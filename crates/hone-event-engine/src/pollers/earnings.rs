//! EarningsPoller — 拉取 FMP earning_calendar,只产出一次性的 teaser 事件。
//!
//! **Read-time derivation**(v0.1.46 重构):
//! - Poller 只产出"事实":`earnings:{SYMBOL}:{DATE}` teaser(Medium),id 稳定,
//!   EventStore 去重保证同一场财报只入库一次,Poller 的 cron 漂移不影响推送精度
//! - T-3/T-2/T-1 每日倒计时**不再由 Poller 产出**,改由 `DigestScheduler` 在
//!   每次 flush 时刻根据 `now` 现算(见 `synthesize_countdowns`)——这样用户
//!   重启时机、poller 漂移、跨时区都不会让倒计时 off-by-one
//! - 整条 lifecycle 仍共享 `EventKind::EarningsUpcoming`,用户把它放进
//!   `blocked_kinds` 就能一次静音 teaser + 所有倒计时

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, NaiveDate, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;
use crate::source::{EventSource, SourceSchedule};

pub struct EarningsPoller {
    client: FmpClient,
    window_days: i64,
    schedule: SourceSchedule,
}

impl EarningsPoller {
    pub fn new(client: FmpClient, schedule: SourceSchedule) -> Self {
        Self {
            client,
            window_days: 14,
            schedule,
        }
    }

    pub fn with_window_days(mut self, days: i64) -> Self {
        self.window_days = days;
        self
    }
}

#[async_trait]
impl EventSource for EarningsPoller {
    fn name(&self) -> &str {
        "fmp.earnings"
    }

    fn schedule(&self) -> SourceSchedule {
        self.schedule.clone()
    }

    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let today = Utc::now().date_naive();
        let to = today + ChronoDuration::days(self.window_days);
        let path = format!(
            "/v3/earning_calendar?from={}&to={}",
            today.format("%Y-%m-%d"),
            to.format("%Y-%m-%d")
        );
        let raw = self.client.get_json(&path).await?;
        Ok(events_from_calendar(&raw))
    }
}

/// 纯函数:把 FMP earning_calendar 响应映射为 teaser MarketEvent 列表。
///
/// 每条 earnings 产出一条 `earnings:{SYM}:{DATE}` (Medium) teaser,id 稳定;
/// EventStore 去重保证同一场财报只入库一次。倒计时由 `synthesize_countdowns`
/// 在 digest flush 时刻按 `now` 现算,不在这里产出。
fn events_from_calendar(raw: &Value) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };

    let mut out = Vec::new();
    for item in arr.iter() {
        let Some(symbol) = item
            .get("symbol")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            continue;
        };
        let Some(date_str) = item
            .get("date")
            .and_then(|v| v.as_str())
            .map(str::to_string)
        else {
            continue;
        };
        let Ok(naive) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") else {
            continue;
        };
        let Some(dt) = naive.and_hms_opt(0, 0, 0) else {
            continue;
        };
        let occurred_at = Utc.from_utc_datetime(&dt).to_utc();

        let eps_est = item.get("epsEstimated").and_then(|v| v.as_f64());
        let rev_est = item.get("revenueEstimated").and_then(|v| v.as_f64());
        let summary = match (eps_est, rev_est) {
            (Some(e), Some(r)) => format!("EPS est {e:.2} · Rev est {r:.0}"),
            (Some(e), None) => format!("EPS est {e:.2}"),
            (None, Some(r)) => format!("Rev est {r:.0}"),
            (None, None) => String::new(),
        };

        out.push(MarketEvent {
            id: format!("earnings:{symbol}:{date_str}"),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec![symbol.clone()],
            occurred_at,
            title: format!("{symbol} earnings on {date_str}"),
            summary,
            url: None,
            source: "fmp.earning_calendar".into(),
            payload: item.clone(),
        });
    }
    out
}

/// 根据一批已入库的 earnings teaser + 当前本地日期,现算出 T-3/T-2/T-1 倒计时
/// "虚拟事件"列表。用于 `DigestScheduler` 在 flush 时刻覆盖到每个 actor 的推送
/// payload 上;这些事件**不入库**,不会触发 dedup,天然幂等。
///
/// 输入 `teasers` 应是 `EventStore::list_upcoming_earnings` 的结果(今天到未来
/// 若干天的 `EarningsUpcoming` 事件);本函数只负责"从 occurred_at 推导 N"的
/// 纯计算,不做 SQL。
///
/// Id 带 `synth:` 前缀 + 当日日期,保证:
/// - 不与真实入库事件 id 冲突
/// - 同一天同一场财报只会产一条倒计时(render 侧 dedup 依赖 id)
///
/// Severity 统一 Medium:T-1 不再升 High,因为 digest flush 本身就是在用户
/// 配置的 pre_market/post_market 时刻触发——T-1 teaser 在 pre_market 那晚的
/// 19:00 CN flush 里恰好是"明早盘前提醒",不需要再绕过 digest。
pub fn synthesize_countdowns(teasers: &[MarketEvent], today: NaiveDate) -> Vec<MarketEvent> {
    let mut out = Vec::new();
    for t in teasers {
        if !matches!(t.kind, EventKind::EarningsUpcoming) {
            continue;
        }
        let event_date = t.occurred_at.date_naive();
        let days_until = (event_date - today).num_days();
        if !(1..=3).contains(&days_until) {
            continue;
        }
        let Some(symbol) = t.symbols.first().cloned() else {
            continue;
        };
        let date_str = event_date.format("%Y-%m-%d").to_string();
        let today_str = today.format("%Y-%m-%d").to_string();
        let phrasing = if days_until == 1 {
            "tomorrow".to_string()
        } else {
            format!("in {days_until} days")
        };
        out.push(MarketEvent {
            id: format!("synth:earnings:{symbol}:{date_str}:countdown:{today_str}"),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec![symbol.clone()],
            occurred_at: t.occurred_at,
            title: format!("{symbol} earnings {phrasing} ({date_str})"),
            summary: t.summary.clone(),
            url: None,
            source: "digest.synth.earnings_countdown".into(),
            payload: t.payload.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_calendar_response() {
        let raw = serde_json::json!([
            {
                "date": "2026-04-30",
                "symbol": "AAPL",
                "eps": null,
                "epsEstimated": 1.52,
                "time": "amc",
                "revenue": null,
                "revenueEstimated": 95000000000.0,
                "updatedFromDate": "2026-04-20",
                "fiscalDateEnding": "2026-03-31"
            },
            {
                "date": "2026-05-01",
                "symbol": "MSFT",
                "epsEstimated": 2.91,
                "revenueEstimated": 68000000000.0
            }
        ]);
        let events = events_from_calendar(&raw);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, "earnings:AAPL:2026-04-30");
        assert!(events[0].touches("AAPL"));
        assert_eq!(events[0].severity, Severity::Medium);
        assert_eq!(events[0].source, "fmp.earning_calendar");
        assert!(events[0].summary.contains("EPS est 1.52"));
    }

    #[test]
    fn poller_never_emits_countdown_events() {
        // v0.1.46 起 poller 只产 teaser,倒计时由 DigestScheduler 现算
        let raw = serde_json::json!([
            {"symbol": "AAPL", "date": "2026-04-30"},
            {"symbol": "MSFT", "date": "2026-05-02"},
            {"symbol": "NVDA", "date": "2026-05-07"}
        ]);
        let events = events_from_calendar(&raw);
        assert_eq!(events.len(), 3);
        for ev in &events {
            assert!(
                !ev.id.contains(":countdown:"),
                "poller 不应再产出 countdown 事件: {}",
                ev.id
            );
            assert_eq!(ev.severity, Severity::Medium);
        }
    }

    #[test]
    fn empty_or_invalid_input_returns_empty() {
        assert!(events_from_calendar(&serde_json::json!({})).is_empty());
        assert!(events_from_calendar(&serde_json::json!([])).is_empty());
    }

    #[test]
    fn skips_items_missing_required_fields() {
        let raw = serde_json::json!([
            {"date": "2026-04-30"},                  // 缺 symbol
            {"symbol": "AAPL"},                       // 缺 date
            {"symbol": "TSLA", "date": "not-a-date"}, // 非法 date
            {"symbol": "NVDA", "date": "2026-05-01"} // 合法
        ]);
        let events = events_from_calendar(&raw);
        assert_eq!(events.len(), 1);
        assert!(events[0].touches("NVDA"));
    }

    #[test]
    fn event_ids_are_stable_and_unique_per_symbol_date() {
        let raw = serde_json::json!([
            {"symbol": "AAPL", "date": "2026-04-30"},
            {"symbol": "AAPL", "date": "2026-04-30"}, // 重复输入
            {"symbol": "AAPL", "date": "2026-07-30"}
        ]);
        let events = events_from_calendar(&raw);
        assert_eq!(events[0].id, events[1].id);
        assert_ne!(events[0].id, events[2].id);
    }

    fn fake_teaser(symbol: &str, date: &str) -> MarketEvent {
        let naive = NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap();
        let dt = naive.and_hms_opt(0, 0, 0).unwrap();
        MarketEvent {
            id: format!("earnings:{symbol}:{date}"),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec![symbol.into()],
            occurred_at: Utc.from_utc_datetime(&dt),
            title: format!("{symbol} earnings on {date}"),
            summary: "EPS est 1.00".into(),
            url: None,
            source: "fmp.earning_calendar".into(),
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn synth_emits_t_minus_1_2_3_for_upcoming_teaser() {
        let teaser = fake_teaser("AAPL", "2026-04-30");
        let today = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap(); // T-3
        let synth = synthesize_countdowns(&[teaser.clone()], today);
        assert_eq!(synth.len(), 1);
        assert!(
            synth[0]
                .id
                .contains("synth:earnings:AAPL:2026-04-30:countdown:2026-04-27")
        );
        assert!(synth[0].title.contains("in 3 days"));
        assert_eq!(synth[0].severity, Severity::Medium);
        assert_eq!(synth[0].source, "digest.synth.earnings_countdown");

        let t2 = synthesize_countdowns(
            &[teaser.clone()],
            NaiveDate::from_ymd_opt(2026, 4, 28).unwrap(),
        );
        assert_eq!(t2.len(), 1);
        assert!(t2[0].title.contains("in 2 days"));

        let t1 = synthesize_countdowns(&[teaser], NaiveDate::from_ymd_opt(2026, 4, 29).unwrap());
        assert_eq!(t1.len(), 1);
        assert!(t1[0].title.contains("tomorrow"));
        assert_eq!(
            t1[0].severity,
            Severity::Medium,
            "T-1 不再升 High,靠 flush 定时即可"
        );
    }

    #[test]
    fn synth_suppressed_outside_window() {
        let teaser = fake_teaser("NVDA", "2026-04-30");
        // T-4:太远
        let t4 = synthesize_countdowns(
            &[teaser.clone()],
            NaiveDate::from_ymd_opt(2026, 4, 26).unwrap(),
        );
        assert!(t4.is_empty());
        // T-0:财报当日,EarningsSurprisePoller 接手
        let t0 = synthesize_countdowns(
            &[teaser.clone()],
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        );
        assert!(t0.is_empty());
        // T+1:已过期
        let tp1 = synthesize_countdowns(&[teaser], NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
        assert!(tp1.is_empty());
    }

    #[test]
    fn synth_ids_embed_today_so_per_day_renderings_dont_dedupe_each_other() {
        let teaser = fake_teaser("AMD", "2026-04-30");
        let t3 = synthesize_countdowns(
            &[teaser.clone()],
            NaiveDate::from_ymd_opt(2026, 4, 27).unwrap(),
        );
        let t2 = synthesize_countdowns(
            &[teaser.clone()],
            NaiveDate::from_ymd_opt(2026, 4, 28).unwrap(),
        );
        let t1 = synthesize_countdowns(&[teaser], NaiveDate::from_ymd_opt(2026, 4, 29).unwrap());
        assert_ne!(t3[0].id, t2[0].id);
        assert_ne!(t2[0].id, t1[0].id);
    }

    #[test]
    fn synth_ignores_non_earnings_events() {
        let mut wrong_kind = fake_teaser("AAPL", "2026-04-30");
        wrong_kind.kind = EventKind::NewsCritical;
        let synth =
            synthesize_countdowns(&[wrong_kind], NaiveDate::from_ymd_opt(2026, 4, 29).unwrap());
        assert!(synth.is_empty());
    }

    /// 真实 FMP 烟测；默认忽略。
    ///
    /// 触发：`HONE_FMP_API_KEY=xxx cargo test -p hone-event-engine \
    ///        --  --ignored live_fmp_earnings_smoke --nocapture`
    #[tokio::test]
    #[ignore]
    async fn live_fmp_earnings_smoke() {
        use std::time::Duration;
        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = crate::fmp::FmpClient::from_config(&cfg);
        let poller = EarningsPoller::new(
            client,
            SourceSchedule::FixedInterval(Duration::from_secs(60)),
        );
        let events = poller.poll().await.expect("FMP poll failed");
        println!("earnings events pulled: {}", events.len());
        for ev in events.iter().take(5) {
            println!("  {} · {} · {}", ev.id, ev.title, ev.summary);
        }
        assert!(!events.is_empty(), "14 天窗口内应至少有 1 条财报");
    }
}
