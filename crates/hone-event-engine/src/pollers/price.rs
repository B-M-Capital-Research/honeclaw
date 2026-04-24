//! PricePoller — 批量查 FMP `v3/quote`，按日涨跌幅阈值产出 `PriceAlert`
//! 以及 52 周高/低事件。
//!
//! - `poll()` 对 watch pool（调用方在构造时传入）批量查询
//! - 阈值：`|pct| < low_pct` → 无事件；`low_pct <= |pct| < high_pct` → Low；`|pct| >= high_pct` → High
//! - 52 周：`price >= yearHigh` → `Weekly52High`（Medium）；`price <= yearLow` → `Weekly52Low`（Medium）
//! - id 稳定：`price:{SYM}:{YYYY-MM-DD}` / `52h:{SYM}:{YYYY-MM-DD}` / `52l:{SYM}:{YYYY-MM-DD}`
//!   每交易日最多一次，避免重复推送。

use chrono::{DateTime, TimeZone, Timelike, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;

const FRESH_QUOTE_MAX_AGE_SECS: i64 = 15 * 60;
const CLOSING_QUOTE_MAX_AGE_SECS: i64 = 20 * 60 * 60;
const FUTURE_QUOTE_MAX_SKEW_SECS: i64 = 5 * 60;

pub struct PricePoller {
    client: FmpClient,
    symbols: Vec<String>,
    low_pct: f64,
    high_pct: f64,
    /// 52 周高/低的相对容差（0.001 = 触碰 0.1% 内算新高/新低）。
    near_hi_lo_tolerance: f64,
}

impl PricePoller {
    pub fn new(client: FmpClient) -> Self {
        Self {
            client,
            symbols: vec![],
            low_pct: 5.0,
            high_pct: 10.0,
            near_hi_lo_tolerance: 0.001,
        }
    }

    pub fn with_symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = symbols;
        self
    }

    pub fn with_thresholds(mut self, low_pct: f64, high_pct: f64) -> Self {
        self.low_pct = low_pct;
        self.high_pct = high_pct;
        self
    }

    pub async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        if self.symbols.is_empty() {
            return Ok(vec![]);
        }
        let joined = self.symbols.join(",");
        let path = format!("/v3/quote/{joined}");
        let raw = self.client.get_json(&path).await?;
        Ok(events_from_quotes_at(
            &raw,
            self.low_pct,
            self.high_pct,
            self.near_hi_lo_tolerance,
            Utc::now(),
        ))
    }
}

#[cfg(test)]
fn events_from_quotes(raw: &Value, low_pct: f64, high_pct: f64, near_tol: f64) -> Vec<MarketEvent> {
    events_from_quotes_at(raw, low_pct, high_pct, near_tol, Utc::now())
}

fn events_from_quotes_at(
    raw: &Value,
    low_pct: f64,
    high_pct: f64,
    near_tol: f64,
    now: DateTime<Utc>,
) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    let mut out = Vec::new();

    for item in arr {
        let Some((quote_time, window)) = quote_time_and_window(item, now) else {
            continue;
        };
        let date_key = quote_time.date_naive().format("%Y-%m-%d").to_string();
        let Some(symbol) = item
            .get("symbol")
            .and_then(|v| v.as_str())
            .map(String::from)
        else {
            continue;
        };
        let price = item.get("price").and_then(|v| v.as_f64());
        let pct = item.get("changesPercentage").and_then(|v| v.as_f64());
        let year_high = item.get("yearHigh").and_then(|v| v.as_f64());
        let year_low = item.get("yearLow").and_then(|v| v.as_f64());

        if let Some(pct) = pct {
            let abs = pct.abs();
            if abs >= low_pct {
                let severity = if window == PriceWindow::Close {
                    closing_move_severity(abs, high_pct)
                } else if abs >= high_pct {
                    Severity::High
                } else {
                    Severity::Low
                };
                let bps = (pct * 100.0).round() as i64;
                let direction = if pct >= 0.0 { "+" } else { "" };
                out.push(MarketEvent {
                    id: format!("{}:{symbol}:{date_key}", window.price_id_prefix()),
                    kind: EventKind::PriceAlert {
                        pct_change_bps: bps,
                        window: window.as_str().into(),
                    },
                    severity,
                    symbols: vec![symbol.clone()],
                    occurred_at: quote_time,
                    title: format!("{symbol} {direction}{pct:.2}%"),
                    summary: price.map(|p| format!("价格 {p:.2}")).unwrap_or_default(),
                    url: None,
                    source: "fmp.quote".into(),
                    payload: item.clone(),
                });
            }
        }

        if let (Some(price), Some(yh)) = (price, year_high) {
            if yh > 0.0 && price >= yh * (1.0 - near_tol) {
                out.push(MarketEvent {
                    id: format!("52h:{symbol}:{date_key}"),
                    kind: EventKind::Weekly52High,
                    severity: Severity::Medium,
                    symbols: vec![symbol.clone()],
                    occurred_at: quote_time,
                    title: format!("{symbol} 触及 52 周新高"),
                    summary: format!("价格 {price:.2} · 年内高 {yh:.2}"),
                    url: None,
                    source: "fmp.quote".into(),
                    payload: item.clone(),
                });
            }
        }
        if let (Some(price), Some(yl)) = (price, year_low) {
            if yl > 0.0 && price <= yl * (1.0 + near_tol) {
                out.push(MarketEvent {
                    id: format!("52l:{symbol}:{date_key}"),
                    kind: EventKind::Weekly52Low,
                    severity: Severity::Medium,
                    symbols: vec![symbol.clone()],
                    occurred_at: quote_time,
                    title: format!("{symbol} 触及 52 周新低"),
                    summary: format!("价格 {price:.2} · 年内低 {yl:.2}"),
                    url: None,
                    source: "fmp.quote".into(),
                    payload: item.clone(),
                });
            }
        }
    }

    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PriceWindow {
    Day,
    Close,
}

impl PriceWindow {
    fn as_str(self) -> &'static str {
        match self {
            PriceWindow::Day => "day",
            PriceWindow::Close => "close",
        }
    }

    fn price_id_prefix(self) -> &'static str {
        match self {
            PriceWindow::Day => "price",
            PriceWindow::Close => "price_close",
        }
    }
}

fn quote_time_and_window(item: &Value, now: DateTime<Utc>) -> Option<(DateTime<Utc>, PriceWindow)> {
    let Some(quote_time) = item
        .get("timestamp")
        .and_then(|v| v.as_i64())
        .and_then(|ts| Utc.timestamp_opt(ts, 0).single())
    else {
        return Some((now, PriceWindow::Day));
    };

    let age_secs = now.signed_duration_since(quote_time).num_seconds();
    if age_secs < -FUTURE_QUOTE_MAX_SKEW_SECS {
        return None;
    }

    if is_us_regular_close_quote(quote_time) {
        return (age_secs <= CLOSING_QUOTE_MAX_AGE_SECS)
            .then_some((quote_time, PriceWindow::Close));
    }

    (age_secs <= FRESH_QUOTE_MAX_AGE_SECS).then_some((quote_time, PriceWindow::Day))
}

fn is_us_regular_close_quote(quote_time: DateTime<Utc>) -> bool {
    matches!(quote_time.hour(), 20 | 21) && quote_time.minute() <= 10
}

fn closing_move_severity(abs_pct: f64, high_pct: f64) -> Severity {
    if abs_pct >= high_pct {
        Severity::High
    } else {
        Severity::Low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_low_pct_emits_nothing() {
        let raw = serde_json::json!([
            {"symbol": "AAPL", "price": 200.0, "changesPercentage": 2.5,
             "yearHigh": 250.0, "yearLow": 150.0}
        ]);
        let events = events_from_quotes(&raw, 5.0, 10.0, 0.001);
        assert!(events.is_empty());
    }

    #[test]
    fn mid_range_pct_is_low_severity_price_alert() {
        let raw = serde_json::json!([
            {"symbol": "AAPL", "price": 200.0, "changesPercentage": 7.0,
             "yearHigh": 250.0, "yearLow": 150.0}
        ]);
        let events = events_from_quotes(&raw, 5.0, 10.0, 0.001);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, Severity::Low);
        match &events[0].kind {
            EventKind::PriceAlert {
                pct_change_bps,
                window,
            } => {
                assert_eq!(*pct_change_bps, 700);
                assert_eq!(window, "day");
            }
            _ => panic!("expected PriceAlert"),
        }
    }

    #[test]
    fn above_high_pct_is_high_severity() {
        let raw = serde_json::json!([
            {"symbol": "TSLA", "price": 300.0, "changesPercentage": -12.3,
             "yearHigh": 400.0, "yearLow": 200.0}
        ]);
        let events = events_from_quotes(&raw, 5.0, 10.0, 0.001);
        // 只返回 PriceAlert High（-12.3 触发），价格离 yearLow 还远
        assert!(events.iter().any(|e| e.severity == Severity::High));
        assert!(
            events
                .iter()
                .any(|e| matches!(e.kind, EventKind::PriceAlert { .. }))
        );
    }

    #[test]
    fn touches_year_high_emits_52h_event() {
        let raw = serde_json::json!([
            {"symbol": "NVDA", "price": 1000.0, "changesPercentage": 1.0,
             "yearHigh": 1000.0, "yearLow": 400.0}
        ]);
        let events = events_from_quotes(&raw, 5.0, 10.0, 0.001);
        assert!(
            events
                .iter()
                .any(|e| matches!(e.kind, EventKind::Weekly52High))
        );
        let hi = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::Weekly52High))
            .unwrap();
        assert_eq!(hi.severity, Severity::Medium);
        assert!(hi.id.starts_with("52h:NVDA:"));
    }

    #[test]
    fn touches_year_low_emits_52l_event() {
        let raw = serde_json::json!([
            {"symbol": "BOO", "price": 50.0, "changesPercentage": -1.0,
             "yearHigh": 200.0, "yearLow": 50.0}
        ]);
        let events = events_from_quotes(&raw, 5.0, 10.0, 0.001);
        let lo = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::Weekly52Low))
            .unwrap();
        assert_eq!(lo.severity, Severity::Medium);
        assert!(lo.id.starts_with("52l:BOO:"));
    }

    #[test]
    fn missing_price_or_pct_is_safe() {
        let raw = serde_json::json!([
            {"symbol": "X"},                                  // 全空
            {"symbol": "Y", "price": 10.0},                   // 无 pct 无高低
            {"symbol": "Z", "changesPercentage": 20.0}        // 无 price
        ]);
        let events = events_from_quotes(&raw, 5.0, 10.0, 0.001);
        // Z 仍能产出 PriceAlert（price 只影响 summary）
        assert!(events.iter().all(|e| !e.id.starts_with("52")));
        assert!(events.iter().any(|e| e.symbols[0] == "Z"));
    }

    #[test]
    fn quote_timestamp_drives_price_event_date_and_occurrence() {
        let quote_time = Utc.with_ymd_and_hms(2026, 4, 22, 13, 32, 40).unwrap();
        let now = quote_time + chrono::Duration::seconds(2);
        let raw = serde_json::json!([
            {"symbol": "BE", "price": 229.75, "changesPercentage": 4.01,
             "timestamp": quote_time.timestamp(), "yearHigh": 235.35, "yearLow": 16.05}
        ]);
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 0.001, now);
        let price = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::PriceAlert { .. }))
            .unwrap();
        assert_eq!(price.id, "price:BE:2026-04-22");
        assert_eq!(price.occurred_at, quote_time);
    }

    #[test]
    fn stale_non_close_quote_is_ignored() {
        let quote_time = Utc.with_ymd_and_hms(2026, 4, 22, 13, 32, 40).unwrap();
        let now = quote_time + chrono::Duration::hours(4);
        let raw = serde_json::json!([
            {"symbol": "BE", "price": 229.75, "changesPercentage": 8.0,
             "timestamp": quote_time.timestamp(), "yearHigh": 235.35, "yearLow": 16.05}
        ]);
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 0.001, now);
        assert!(events.is_empty());
    }

    #[test]
    fn close_quote_above_high_pct_gets_close_id_and_high_severity() {
        let close_time = Utc.with_ymd_and_hms(2026, 4, 22, 20, 0, 1).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 4, 23, 0, 2, 42).unwrap();
        let raw = serde_json::json!([
            {"symbol": "AMD", "price": 303.46, "changesPercentage": 6.66807,
             "timestamp": close_time.timestamp(), "yearHigh": 304.10, "yearLow": 90.12}
        ]);
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 0.001, now);
        let price = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::PriceAlert { .. }))
            .unwrap();
        assert_eq!(price.id, "price_close:AMD:2026-04-22");
        assert_eq!(price.severity, Severity::High);
        assert_eq!(price.occurred_at, close_time);
        match &price.kind {
            EventKind::PriceAlert { window, .. } => assert_eq!(window, "close"),
            _ => panic!("expected PriceAlert"),
        }
    }

    #[test]
    fn close_quote_below_high_pct_remains_low_severity() {
        let close_time = Utc.with_ymd_and_hms(2026, 4, 22, 20, 0, 1).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 4, 23, 0, 2, 42).unwrap();
        let raw = serde_json::json!([
            {"symbol": "AMD", "price": 303.46, "changesPercentage": 3.5,
             "timestamp": close_time.timestamp(), "yearHigh": 500.10, "yearLow": 90.12}
        ]);
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 0.001, now);
        let price = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::PriceAlert { .. }))
            .unwrap();
        assert_eq!(price.id, "price_close:AMD:2026-04-22");
        assert_eq!(price.severity, Severity::Low);
    }

    #[test]
    fn very_old_close_quote_is_ignored() {
        let close_time = Utc.with_ymd_and_hms(2026, 4, 22, 20, 0, 1).unwrap();
        let now = close_time + chrono::Duration::hours(24);
        let raw = serde_json::json!([
            {"symbol": "AMD", "price": 303.46, "changesPercentage": 6.66807,
             "timestamp": close_time.timestamp(), "yearHigh": 304.10, "yearLow": 90.12}
        ]);
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 0.001, now);
        assert!(events.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn live_fmp_price_smoke() {
        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = FmpClient::from_config(&cfg);
        let poller = PricePoller::new(client)
            .with_symbols(vec!["AAPL".into(), "MSFT".into(), "NVDA".into()])
            .with_thresholds(0.1, 5.0); // 很敏感，确保能看到产出
        let events = poller.poll().await.expect("FMP poll failed");
        println!("price events pulled: {}", events.len());
        for ev in events.iter().take(10) {
            println!("  [{:?}] {} · {}", ev.severity, ev.title, ev.summary);
        }
    }
}
