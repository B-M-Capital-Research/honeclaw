//! 市场事件核心数据结构。
//!
//! `MarketEvent` 是事件引擎的通用载荷，所有 poller 产出此类型。

use chrono::{DateTime, Utc};
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
    PortfolioPreMarket,
    PortfolioPostMarket,
    /// 第三方社交源帖子 (Telegram channel 等)。severity 默认 Low,
    /// router 的 LLM 仲裁链路按 `payload.source_class == "uncertain"` 决定是否升 Medium。
    SocialPost,
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
