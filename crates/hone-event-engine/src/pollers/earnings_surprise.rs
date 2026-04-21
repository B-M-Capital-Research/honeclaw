//! EarningsSurprisePoller — 拉取已发布财报的 surprise%。
//!
//! 源：FMP `v3/earnings-surprises/{ticker}`。盘后 16:30 ET 之后由 scheduler
//! 触发。和 `EarningsPoller` 的区别：
//! - `EarningsPoller` 日历/预告（T-1 Medium），不含实际数据
//! - `EarningsSurprisePoller` 实际 vs 预期，含 `actualEarningResult` + `estimatedEarning`
//!
//! 严重度映射：
//! - `|surprise_pct| >= 5%` → High（显著 beat / miss）
//! - 其他                   → Medium
//!
//! id 稳定：`earnings_surprise:{SYMBOL}:{date}`——一家公司一个季度只有一条。

use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;

pub struct EarningsSurprisePoller {
    client: FmpClient,
    lookback_days: i64,
    high_threshold_pct: f64,
}

impl EarningsSurprisePoller {
    pub fn new(client: FmpClient) -> Self {
        Self {
            client,
            lookback_days: 3,
            high_threshold_pct: 5.0,
        }
    }

    pub fn with_lookback_days(mut self, days: i64) -> Self {
        self.lookback_days = days;
        self
    }

    pub fn with_high_threshold_pct(mut self, pct: f64) -> Self {
        self.high_threshold_pct = pct;
        self
    }

    /// 按 watch pool 拉每一只票的最新 surprise。只保留 `lookback_days` 窗口内的。
    pub async fn poll(&self, tickers: &[String]) -> anyhow::Result<Vec<MarketEvent>> {
        let mut out = Vec::new();
        let cutoff = Utc::now() - chrono::Duration::days(self.lookback_days);
        for t in tickers {
            let path = format!("/v3/earnings-surprises/{t}");
            match self.client.get_json(&path).await {
                Ok(v) => out.extend(events_from_surprises(
                    &v,
                    t,
                    cutoff,
                    self.high_threshold_pct,
                )),
                Err(e) => tracing::warn!("earnings surprise fetch failed for {t}: {e:#}"),
            }
        }
        Ok(out)
    }
}

fn events_from_surprises(
    raw: &Value,
    ticker: &str,
    cutoff: DateTime<Utc>,
    high_pct: f64,
) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter()
        .filter_map(|item| {
            let date = item.get("date").and_then(|v| v.as_str())?.to_string();
            let naive = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?;
            let occurred_at = Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0)?);
            if occurred_at < cutoff {
                return None;
            }
            let actual = item.get("actualEarningResult").and_then(|v| v.as_f64())?;
            let est = item.get("estimatedEarning").and_then(|v| v.as_f64())?;
            if est.abs() < f64::EPSILON {
                return None;
            }
            let pct = (actual - est) / est.abs() * 100.0;
            let severity = if pct.abs() >= high_pct {
                Severity::High
            } else {
                Severity::Medium
            };
            let direction = if pct >= 0.0 {
                "超预期"
            } else {
                "不及预期"
            };
            // FMP /v3/earnings-surprises 本身不返回 press release 链接;
            // 指向 Yahoo 的 press-releases 页面作为通用兜底 —— 用户点进去就能看到
            // 该公司最新(通常就是当日)的财报新闻稿,无需注册且免费。
            let url = Some(format!(
                "https://finance.yahoo.com/quote/{ticker}/press-releases/"
            ));
            Some(MarketEvent {
                id: format!("earnings_surprise:{ticker}:{date}"),
                kind: EventKind::EarningsReleased,
                severity,
                symbols: vec![ticker.to_string()],
                occurred_at,
                title: format!("{ticker} 财报 {direction} {pct:+.1}%"),
                summary: format!("实际 {actual:.2} / 预期 {est:.2}"),
                url,
                source: "fmp.earnings_surprises".into(),
                payload: item.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surprise(date_offset: i64, actual: f64, est: f64) -> Value {
        let d = (Utc::now() - chrono::Duration::days(date_offset))
            .format("%Y-%m-%d")
            .to_string();
        serde_json::json!({
            "date": d,
            "symbol": "AAPL",
            "actualEarningResult": actual,
            "estimatedEarning": est,
        })
    }

    #[test]
    fn large_beat_is_high() {
        let raw = serde_json::json!([surprise(0, 2.30, 2.00)]);
        let events =
            events_from_surprises(&raw, "AAPL", Utc::now() - chrono::Duration::days(7), 5.0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, Severity::High);
        assert!(events[0].title.contains("超预期"));
        assert!(events[0].summary.contains("2.30"));
        assert_eq!(
            events[0].id,
            format!(
                "earnings_surprise:AAPL:{}",
                events[0].occurred_at.format("%Y-%m-%d")
            )
        );
    }

    #[test]
    fn released_event_carries_press_release_link() {
        let raw = serde_json::json!([surprise(0, 2.30, 2.00)]);
        let events =
            events_from_surprises(&raw, "AAPL", Utc::now() - chrono::Duration::days(7), 5.0);
        let url = events[0].url.as_ref().expect("press-release url");
        assert!(url.contains("AAPL"));
        assert!(url.starts_with("https://"));
        assert!(url.contains("press-releases"));
    }

    #[test]
    fn small_beat_is_medium() {
        let raw = serde_json::json!([surprise(0, 2.03, 2.00)]);
        let events =
            events_from_surprises(&raw, "AAPL", Utc::now() - chrono::Duration::days(7), 5.0);
        assert_eq!(events[0].severity, Severity::Medium);
    }

    #[test]
    fn large_miss_is_high() {
        let raw = serde_json::json!([surprise(0, 1.70, 2.00)]);
        let events =
            events_from_surprises(&raw, "AAPL", Utc::now() - chrono::Duration::days(7), 5.0);
        assert_eq!(events[0].severity, Severity::High);
        assert!(events[0].title.contains("不及预期"));
    }

    #[test]
    fn stale_surprise_is_dropped() {
        let raw = serde_json::json!([surprise(90, 2.30, 2.00)]);
        let events =
            events_from_surprises(&raw, "AAPL", Utc::now() - chrono::Duration::days(3), 5.0);
        assert!(events.is_empty());
    }

    #[test]
    fn zero_estimate_is_skipped() {
        let raw = serde_json::json!([surprise(0, 2.30, 0.0)]);
        let events =
            events_from_surprises(&raw, "AAPL", Utc::now() - chrono::Duration::days(3), 5.0);
        assert!(events.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn live_fmp_earnings_surprise_smoke() {
        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = FmpClient::from_config(&cfg);
        let poller = EarningsSurprisePoller::new(client).with_lookback_days(90);
        let events = poller
            .poll(&["AAPL".into(), "NVDA".into()])
            .await
            .expect("FMP poll failed");
        println!("earnings surprise events pulled: {}", events.len());
        for ev in events.iter().take(5) {
            println!("  [{:?}] {} · {}", ev.severity, ev.title, ev.summary);
        }
    }
}
