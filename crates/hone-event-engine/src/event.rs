//! 市场事件核心数据结构。
//!
//! `MarketEvent` 是事件引擎的通用载荷，所有 poller 产出此类型。

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Low,
    Medium,
    High,
}

impl Severity {
    /// 大小比较用的数值秩：Low=0 < Medium=1 < High=2。
    pub fn rank(self) -> u8 {
        match self {
            Severity::Low => 0,
            Severity::Medium => 1,
            Severity::High => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    EarningsUpcoming,
    EarningsReleased,
    EarningsCallTranscript,
    NewsCritical,
    PressRelease,
    PriceAlert {
        pct_change_bps: i64,
        window: String,
    },
    Weekly52High,
    Weekly52Low,
    VolumeSpike,
    Dividend,
    Split,
    Buyback,
    SecFiling {
        form: String,
    },
    AnalystGrade,
    MacroEvent,
    /// 第三方社交源帖子 (Telegram channel 等)。severity 默认 Low,
    /// router 的 LLM 仲裁链路按 `payload.source_class == "uncertain"` 决定是否升 Medium。
    SocialPost,
}

impl EventKind {
    /// 事件保鲜期 —— `quiet_flush` 在 `quiet_hours.to` 时刻触发时，超过此时长仍 hold
    /// 的事件直接 drop 不进合集。`None` = 永不过期（事实性事件，如财报/SEC/分红等
    /// 隔夜仍有阅读价值）。原则：**仅盘中市场微观状态会过期**。
    pub fn shelf_life(&self) -> Option<Duration> {
        match self {
            // 盘中价格/成交量微观状态：过夜失效
            EventKind::PriceAlert { .. } | EventKind::VolumeSpike => Some(Duration::hours(2)),
            // 52 周高低虽然技术性强，跨开盘后仍能用一段时间
            EventKind::Weekly52High | EventKind::Weekly52Low => Some(Duration::hours(8)),
            // 社交流热度衰减快，但少数 trusted 源也可能值得隔夜看
            EventKind::SocialPost => Some(Duration::hours(12)),
            // 事实性事件 —— 永不过期
            EventKind::EarningsUpcoming
            | EventKind::EarningsReleased
            | EventKind::EarningsCallTranscript
            | EventKind::NewsCritical
            | EventKind::PressRelease
            | EventKind::Dividend
            | EventKind::Split
            | EventKind::Buyback
            | EventKind::SecFiling { .. }
            | EventKind::AnalystGrade
            | EventKind::MacroEvent => None,
        }
    }

    /// 事件相对 `now` 是否仍新鲜：`shelf_life=None` 一律 true，否则 `now - occurred_at < ttl`。
    pub fn is_fresh(&self, occurred_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        match self.shelf_life() {
            None => true,
            Some(ttl) => (now - occurred_at) < ttl,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub id: String,
    pub kind: EventKind,
    pub severity: Severity,
    #[serde(default)]
    pub symbols: Vec<String>,
    pub occurred_at: DateTime<Utc>,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub url: Option<String>,
    pub source: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl MarketEvent {
    pub fn touches(&self, symbol: &str) -> bool {
        self.symbols.iter().any(|s| s.eq_ignore_ascii_case(symbol))
    }
}

pub fn is_noop_analyst_grade(event: &MarketEvent) -> bool {
    if !matches!(event.kind, EventKind::AnalystGrade) {
        return false;
    }
    let action = event
        .payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if !matches!(action.as_str(), "hold" | "maintained" | "reiterated") {
        return false;
    }
    let previous = normalized_grade(event.payload.get("previousGrade").and_then(|v| v.as_str()));
    let new = normalized_grade(event.payload.get("newGrade").and_then(|v| v.as_str()));
    !new.is_empty() && previous == new
}

fn normalized_grade(raw: Option<&str>) -> String {
    raw.unwrap_or("")
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shelf_life_per_kind_returns_expected_duration() {
        // 盘中市场微观状态 → 短保鲜期
        assert_eq!(
            EventKind::PriceAlert {
                pct_change_bps: 600,
                window: "1d".into(),
            }
            .shelf_life(),
            Some(Duration::hours(2))
        );
        assert_eq!(
            EventKind::VolumeSpike.shelf_life(),
            Some(Duration::hours(2))
        );
        assert_eq!(
            EventKind::Weekly52High.shelf_life(),
            Some(Duration::hours(8))
        );
        assert_eq!(
            EventKind::Weekly52Low.shelf_life(),
            Some(Duration::hours(8))
        );
        assert_eq!(
            EventKind::SocialPost.shelf_life(),
            Some(Duration::hours(12))
        );

        // 事实性事件 → None,永不过期
        assert_eq!(EventKind::EarningsReleased.shelf_life(), None);
        assert_eq!(EventKind::EarningsUpcoming.shelf_life(), None);
        assert_eq!(EventKind::EarningsCallTranscript.shelf_life(), None);
        assert_eq!(EventKind::NewsCritical.shelf_life(), None);
        assert_eq!(EventKind::PressRelease.shelf_life(), None);
        assert_eq!(EventKind::Dividend.shelf_life(), None);
        assert_eq!(EventKind::Split.shelf_life(), None);
        assert_eq!(EventKind::Buyback.shelf_life(), None);
        assert_eq!(
            EventKind::SecFiling { form: "8-K".into() }.shelf_life(),
            None
        );
        assert_eq!(EventKind::AnalystGrade.shelf_life(), None);
        assert_eq!(EventKind::MacroEvent.shelf_life(), None);
    }

    #[test]
    fn is_fresh_handles_none_and_some_shelf_life() {
        let now = Utc::now();
        // None shelf_life → 永远 fresh
        let earnings_old = now - Duration::days(7);
        assert!(EventKind::EarningsReleased.is_fresh(earnings_old, now));
        // Some shelf_life: 边界内 = fresh
        let price_recent = now - Duration::minutes(60);
        assert!(
            EventKind::PriceAlert {
                pct_change_bps: 600,
                window: "1d".into(),
            }
            .is_fresh(price_recent, now)
        );
        // Some shelf_life: 超过 = stale
        let price_stale = now - Duration::hours(3);
        assert!(
            !EventKind::PriceAlert {
                pct_change_bps: 600,
                window: "1d".into(),
            }
            .is_fresh(price_stale, now)
        );
        // Some shelf_life: 正好等于 ttl 边界 → stale (`<` 严格小于)
        let price_at_boundary = now - Duration::hours(2);
        assert!(
            !EventKind::PriceAlert {
                pct_change_bps: 600,
                window: "1d".into(),
            }
            .is_fresh(price_at_boundary, now)
        );
    }

    #[test]
    fn touches_is_case_insensitive() {
        let ev = MarketEvent {
            id: "earnings:AAPL:2026-04-30".into(),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec!["AAPL".into()],
            occurred_at: Utc::now(),
            title: "Apple earnings".into(),
            summary: String::new(),
            url: None,
            source: "fmp.earning_calendar".into(),
            payload: serde_json::Value::Null,
        };
        assert!(ev.touches("aapl"));
        assert!(ev.touches("AAPL"));
        assert!(!ev.touches("TSLA"));
    }

    #[test]
    fn detects_noop_analyst_grade_hold() {
        let ev = MarketEvent {
            id: "grade:GEV:test".into(),
            kind: EventKind::AnalystGrade,
            severity: Severity::Low,
            symbols: vec!["GEV".into()],
            occurred_at: Utc::now(),
            title: "GEV · RBC Capital hold · Outperform".into(),
            summary: "Outperform → Outperform".into(),
            url: None,
            source: "fmp.upgrades_downgrades".into(),
            payload: serde_json::json!({
                "action": "hold",
                "previousGrade": "Outperform",
                "newGrade": "Outperform"
            }),
        };

        assert!(is_noop_analyst_grade(&ev));
    }

    #[test]
    fn target_change_with_same_rating_is_not_noop_grade() {
        let ev = MarketEvent {
            id: "grade:GEV:test".into(),
            kind: EventKind::AnalystGrade,
            severity: Severity::Medium,
            symbols: vec!["GEV".into()],
            occurred_at: Utc::now(),
            title: "GEV · RBC Capital target-raised · Outperform".into(),
            summary: "Outperform → Outperform".into(),
            url: None,
            source: "fmp.upgrades_downgrades".into(),
            payload: serde_json::json!({
                "action": "target-raised",
                "previousGrade": "Outperform",
                "newGrade": "Outperform"
            }),
        };

        assert!(!is_noop_analyst_grade(&ev));
    }

    #[test]
    fn event_roundtrip_json() {
        let ev = MarketEvent {
            id: "price:NVDA:2026-04-21T15:00".into(),
            kind: EventKind::PriceAlert {
                pct_change_bps: 1200,
                window: "5m".into(),
            },
            severity: Severity::High,
            symbols: vec!["NVDA".into()],
            occurred_at: Utc::now(),
            title: "NVDA +12%".into(),
            summary: "intraday spike".into(),
            url: None,
            source: "fmp.quote".into(),
            payload: serde_json::json!({"price": 940.5}),
        };
        let s = serde_json::to_string(&ev).unwrap();
        let back: MarketEvent = serde_json::from_str(&s).unwrap();
        assert_eq!(back.id, ev.id);
        assert_eq!(back.severity, Severity::High);
    }
}
