//! EarningsPoller — 拉取 FMP earning_calendar，产出 EarningsUpcoming 事件。
//!
//! 行为：
//! - 拉取 [today, today+window_days] 的财报日历（window_days 默认 14,可由 config 调整）
//! - 每条记录首次发现时生成一条稳定 id `earnings:{SYMBOL}:{DATE}` 的 Medium 预告 →
//!   进 digest 合并推送；EventStore 去重保证同一场财报只推一次,不会随日子临近重复刷屏
//! - 对距今 T-3/T-2/T-1 的财报,额外每日发一条倒计时事件,id 带 `:countdown:N` 后缀避免
//!   被 store dedup 跨日折叠；T-1 severity = High(立即推,赶在盘前),T-2/T-3 = Medium
//!   (进 digest)
//! - 由于倒计时也走 `EventKind::EarningsUpcoming`,用户只要 `blocked_kinds` 包含
//!   `earnings_upcoming`,初次预告 + 每日倒计时会被一并静音

use chrono::{Duration as ChronoDuration, NaiveDate, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;

pub struct EarningsPoller {
    client: FmpClient,
    window_days: i64,
}

impl EarningsPoller {
    pub fn new(client: FmpClient) -> Self {
        Self {
            client,
            window_days: 14,
        }
    }

    pub fn with_window_days(mut self, days: i64) -> Self {
        self.window_days = days;
        self
    }

    /// 拉取一次窗口内的财报日历，返回事件列表。
    pub async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let today = Utc::now().date_naive();
        let to = today + ChronoDuration::days(self.window_days);
        let path = format!(
            "/v3/earning_calendar?from={}&to={}",
            today.format("%Y-%m-%d"),
            to.format("%Y-%m-%d")
        );
        let raw = self.client.get_json(&path).await?;
        Ok(events_from_calendar(&raw, today))
    }
}

/// 纯函数：把 FMP earning_calendar 响应映射为 MarketEvent 列表。
///
/// 每条 earnings 至多产出两条事件:
/// 1. 首次预告 `earnings:{SYM}:{DATE}` (Medium) —— store dedup 保证只推一次
/// 2. 若 `days_until ∈ [1, 3]`,额外发倒计时 `earnings:{SYM}:{DATE}:countdown:{N}`
///    (T-1 = High,T-2/T-3 = Medium) —— id 随 N 变,每天都能穿过 dedup
fn events_from_calendar(raw: &Value, today: NaiveDate) -> Vec<MarketEvent> {
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

        // 1) 初次预告(id 稳定,跨日 dedup)。
        out.push(MarketEvent {
            id: format!("earnings:{symbol}:{date_str}"),
            kind: EventKind::EarningsUpcoming,
            severity: Severity::Medium,
            symbols: vec![symbol.clone()],
            occurred_at,
            title: format!("{symbol} earnings on {date_str}"),
            summary: summary.clone(),
            url: None,
            source: "fmp.earning_calendar".into(),
            payload: item.clone(),
        });

        // 2) T-3 / T-2 / T-1 倒计时(id 含 N,每日穿过 dedup)。
        let days_until = (naive - today).num_days();
        if (1..=3).contains(&days_until) {
            let (severity, phrasing) = match days_until {
                1 => (Severity::High, "tomorrow".to_string()),
                n => (Severity::Medium, format!("in {n} days")),
            };
            out.push(MarketEvent {
                id: format!("earnings:{symbol}:{date_str}:countdown:{days_until}"),
                kind: EventKind::EarningsUpcoming,
                severity,
                symbols: vec![symbol.clone()],
                occurred_at,
                title: format!("{symbol} earnings {phrasing} ({date_str})"),
                summary: summary.clone(),
                url: None,
                source: "fmp.earning_calendar".into(),
                payload: item.clone(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用的"今天"——远早于所有测试里出现的 earnings 日期,避免意外触发倒计时。
    fn far_today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()
    }

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
        let events = events_from_calendar(&raw, far_today());
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, "earnings:AAPL:2026-04-30");
        assert!(events[0].touches("AAPL"));
        assert_eq!(events[0].severity, Severity::Medium);
        assert_eq!(events[0].source, "fmp.earning_calendar");
        assert!(events[0].summary.contains("EPS est 1.52"));
    }

    #[test]
    fn empty_or_invalid_input_returns_empty() {
        assert!(events_from_calendar(&serde_json::json!({}), far_today()).is_empty());
        assert!(events_from_calendar(&serde_json::json!([]), far_today()).is_empty());
    }

    #[test]
    fn skips_items_missing_required_fields() {
        let raw = serde_json::json!([
            {"date": "2026-04-30"},                  // 缺 symbol
            {"symbol": "AAPL"},                       // 缺 date
            {"symbol": "TSLA", "date": "not-a-date"}, // 非法 date
            {"symbol": "NVDA", "date": "2026-05-01"} // 合法
        ]);
        let events = events_from_calendar(&raw, far_today());
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
        let events = events_from_calendar(&raw, far_today());
        // events_from_calendar 本身不做去重（留给 EventStore）；但 id 必须稳定。
        assert_eq!(events[0].id, events[1].id);
        assert_ne!(events[0].id, events[2].id);
    }

    #[test]
    fn t_minus_3_emits_extra_countdown_event_as_medium() {
        let raw = serde_json::json!([
            {"symbol": "AAPL", "date": "2026-04-30", "epsEstimated": 1.52}
        ]);
        let today = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap(); // T-3
        let events = events_from_calendar(&raw, today);
        assert_eq!(events.len(), 2, "expect base event + T-3 countdown");

        let countdown = events
            .iter()
            .find(|e| e.id.ends_with(":countdown:3"))
            .expect("countdown T-3 event");
        assert_eq!(countdown.severity, Severity::Medium);
        assert!(countdown.title.contains("in 3 days"));
        assert!(countdown.title.contains("2026-04-30"));
        assert!(countdown.summary.contains("EPS est 1.52"));
    }

    #[test]
    fn t_minus_1_countdown_upgrades_to_high() {
        let raw = serde_json::json!([
            {"symbol": "MSFT", "date": "2026-04-30"}
        ]);
        let today = NaiveDate::from_ymd_opt(2026, 4, 29).unwrap(); // T-1
        let events = events_from_calendar(&raw, today);
        assert_eq!(events.len(), 2);

        let countdown = events
            .iter()
            .find(|e| e.id.ends_with(":countdown:1"))
            .expect("countdown T-1 event");
        assert_eq!(countdown.severity, Severity::High);
        assert!(countdown.title.contains("tomorrow"));
    }

    #[test]
    fn countdown_suppressed_beyond_three_days() {
        let raw = serde_json::json!([
            {"symbol": "NVDA", "date": "2026-04-30"}
        ]);
        let today = NaiveDate::from_ymd_opt(2026, 4, 26).unwrap(); // T-4
        let events = events_from_calendar(&raw, today);
        assert_eq!(events.len(), 1, "only base event, no countdown");
        assert!(!events[0].id.contains(":countdown:"));
    }

    #[test]
    fn countdown_suppressed_on_earnings_day_or_after() {
        let raw = serde_json::json!([
            {"symbol": "GOOG", "date": "2026-04-30"}
        ]);
        // T-0 (earnings day itself): EarningsSurprisePoller 接手实际推送,这里不再发倒计时
        let events = events_from_calendar(&raw, NaiveDate::from_ymd_opt(2026, 4, 30).unwrap());
        assert_eq!(events.len(), 1);
        assert!(!events[0].id.contains(":countdown:"));

        // T+1 (已过期): 同样不发
        let events = events_from_calendar(&raw, NaiveDate::from_ymd_opt(2026, 5, 1).unwrap());
        assert_eq!(events.len(), 1);
        assert!(!events[0].id.contains(":countdown:"));
    }

    #[test]
    fn countdown_ids_change_per_day_so_store_dedup_doesnt_fold_them() {
        let raw = serde_json::json!([
            {"symbol": "AMD", "date": "2026-04-30"}
        ]);
        let t_minus_3 = events_from_calendar(&raw, NaiveDate::from_ymd_opt(2026, 4, 27).unwrap());
        let t_minus_2 = events_from_calendar(&raw, NaiveDate::from_ymd_opt(2026, 4, 28).unwrap());
        let t_minus_1 = events_from_calendar(&raw, NaiveDate::from_ymd_opt(2026, 4, 29).unwrap());

        let id_3 = &t_minus_3
            .iter()
            .find(|e| e.id.contains(":countdown:"))
            .unwrap()
            .id;
        let id_2 = &t_minus_2
            .iter()
            .find(|e| e.id.contains(":countdown:"))
            .unwrap()
            .id;
        let id_1 = &t_minus_1
            .iter()
            .find(|e| e.id.contains(":countdown:"))
            .unwrap()
            .id;
        assert_ne!(id_3, id_2);
        assert_ne!(id_2, id_1);
        assert_ne!(id_3, id_1);
    }

    /// 真实 FMP 烟测；默认忽略。
    ///
    /// 触发：`HONE_FMP_API_KEY=xxx cargo test -p hone-event-engine \
    ///        --  --ignored live_fmp_earnings_smoke --nocapture`
    #[tokio::test]
    #[ignore]
    async fn live_fmp_earnings_smoke() {
        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = crate::fmp::FmpClient::from_config(&cfg);
        let poller = EarningsPoller::new(client);
        let events = poller.poll().await.expect("FMP poll failed");
        println!("earnings events pulled: {}", events.len());
        for ev in events.iter().take(5) {
            println!("  {} · {} · {}", ev.id, ev.title, ev.summary);
        }
        assert!(!events.is_empty(), "14 天窗口内应至少有 1 条财报");
    }
}
