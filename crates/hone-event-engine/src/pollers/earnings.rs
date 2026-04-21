//! EarningsPoller — 拉取 FMP earning_calendar，产出 EarningsUpcoming 事件。
//!
//! MVP 行为：
//! - 拉取 [today, today+14d] 的财报日历
//! - 每条记录生成一个稳定 id（`earnings:{SYMBOL}:{DATE}`）用于去重
//! - 默认 severity = Medium（T-1 预告级别），T-1 升级留到 scheduler 层处理

use chrono::{Duration as ChronoDuration, TimeZone, Utc};
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
        Ok(events_from_calendar(&raw))
    }
}

/// 纯函数：把 FMP earning_calendar 响应映射为 MarketEvent。
fn events_from_calendar(raw: &Value) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };

    arr.iter()
        .filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.to_string();
            let date_str = item.get("date")?.as_str()?.to_string();
            // FMP 返回的 date 是 YYYY-MM-DD；我们落到 UTC 当天 00:00。
            let naive = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok()?;
            let occurred_at = Utc
                .from_utc_datetime(&naive.and_hms_opt(0, 0, 0)?)
                .to_utc();

            let eps_est = item.get("epsEstimated").and_then(|v| v.as_f64());
            let rev_est = item.get("revenueEstimated").and_then(|v| v.as_f64());
            let summary = match (eps_est, rev_est) {
                (Some(e), Some(r)) => format!("EPS est {e:.2} · Rev est {r:.0}"),
                (Some(e), None) => format!("EPS est {e:.2}"),
                (None, Some(r)) => format!("Rev est {r:.0}"),
                (None, None) => String::new(),
            };

            Some(MarketEvent {
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
            })
        })
        .collect()
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
        // events_from_calendar 本身不做去重（留给 EventStore）；但 id 必须稳定。
        assert_eq!(events[0].id, events[1].id);
        assert_ne!(events[0].id, events[2].id);
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
