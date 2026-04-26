//! PricePoller — 批量查 FMP `v3/quote`，按日涨跌幅阈值产出 `PriceAlert`
//! 以及 52 周高/低事件。
//!
//! - `poll()` 对 watch pool（调用方在构造时传入）批量查询
//! - 阈值：`|pct| < low_pct` → 无事件；`low_pct <= |pct| < high_pct` → Low；`|pct| >= high_pct` → High
//! - 52 周：`price >= yearHigh` → `Weekly52High`（Medium）；`price <= yearLow` → `Weekly52Low`（Medium）
//! - id 稳定：`price:{SYM}:{YYYY-MM-DD}` / `52h:{SYM}:{YYYY-MM-DD}` / `52l:{SYM}:{YYYY-MM-DD}`
//!   每交易日最多一次，避免重复推送。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Timelike, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;
use crate::source::{EventSource, SourceSchedule};
use crate::subscription::SharedRegistry;

const FRESH_QUOTE_MAX_AGE_SECS: i64 = 15 * 60;
const CLOSING_QUOTE_MAX_AGE_SECS: i64 = 20 * 60 * 60;
const FUTURE_QUOTE_MAX_SKEW_SECS: i64 = 5 * 60;

pub struct PricePoller {
    client: FmpClient,
    registry: Arc<SharedRegistry>,
    schedule: SourceSchedule,
    low_pct: f64,
    high_pct: f64,
    realert_step_pct: f64,
    /// 52 周高/低的相对容差（0.001 = 触碰 0.1% 内算新高/新低）。
    near_hi_lo_tolerance: f64,
}

impl PricePoller {
    pub fn new(client: FmpClient, registry: Arc<SharedRegistry>, schedule: SourceSchedule) -> Self {
        Self {
            client,
            registry,
            schedule,
            low_pct: 5.0,
            high_pct: 10.0,
            realert_step_pct: 2.0,
            near_hi_lo_tolerance: 0.001,
        }
    }

    pub fn with_thresholds(mut self, low_pct: f64, high_pct: f64) -> Self {
        self.low_pct = low_pct;
        self.high_pct = high_pct;
        self
    }

    pub fn with_realert_step_pct(mut self, step_pct: f64) -> Self {
        self.realert_step_pct = sanitize_realert_step_pct(step_pct);
        self
    }

    /// 按指定 ticker 列表批量查 quote。`EventSource::poll` 内部从 registry
    /// 取 watch_pool 后调它;测试可以直接传任意 ticker。watch_pool 为空时
    /// 调用方应直接返回 Ok(vec![])(本函数会照样发请求,用于显式测试)。
    pub async fn fetch(&self, symbols: &[String]) -> anyhow::Result<Vec<MarketEvent>> {
        if symbols.is_empty() {
            return Ok(vec![]);
        }
        let joined = symbols.join(",");
        let path = format!("/v3/quote/{joined}");
        let raw = self.client.get_json(&path).await?;
        Ok(events_from_quotes_at(
            &raw,
            self.low_pct,
            self.high_pct,
            self.realert_step_pct,
            self.near_hi_lo_tolerance,
            Utc::now(),
        ))
    }
}

#[async_trait]
impl EventSource for PricePoller {
    fn name(&self) -> &str {
        "fmp.price"
    }

    fn schedule(&self) -> SourceSchedule {
        self.schedule.clone()
    }

    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let symbols = self.registry.load().watch_pool();
        if symbols.is_empty() {
            return Ok(vec![]);
        }
        self.fetch(&symbols).await
    }
}

#[cfg(test)]
fn events_from_quotes(raw: &Value, low_pct: f64, high_pct: f64, near_tol: f64) -> Vec<MarketEvent> {
    events_from_quotes_at(raw, low_pct, high_pct, 2.0, near_tol, Utc::now())
}

fn events_from_quotes_at(
    raw: &Value,
    low_pct: f64,
    high_pct: f64,
    realert_step_pct: f64,
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
                let step_pct = sanitize_realert_step_pct(realert_step_pct);
                let severity = if window == PriceWindow::Close {
                    closing_move_severity(abs, high_pct)
                } else if abs >= high_pct {
                    Severity::High
                } else {
                    Severity::Low
                };
                let bps = (pct * 100.0).round() as i64;
                let direction = if pct >= 0.0 { "+" } else { "" };
                let lane = price_lane(pct, low_pct, high_pct, step_pct, window);
                let payload = price_payload(item, pct, price, &date_key, lane.as_ref());
                out.push(MarketEvent {
                    id: lane
                        .as_ref()
                        .map(|lane| lane.event_id(&symbol, &date_key, window))
                        .unwrap_or_else(|| {
                            format!("{}:{symbol}:{date_key}", window.price_id_prefix())
                        }),
                    kind: EventKind::PriceAlert {
                        pct_change_bps: bps,
                        window: window.as_str().into(),
                    },
                    severity,
                    symbols: vec![symbol.clone()],
                    occurred_at: quote_time,
                    title: price_title(&symbol, pct, lane.as_ref(), direction),
                    summary: price_summary(price, pct, lane.as_ref()),
                    url: None,
                    source: "fmp.quote".into(),
                    payload,
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

#[derive(Debug, Clone)]
enum PriceLane {
    Low,
    Band {
        direction: &'static str,
        band_bps: i64,
        next_band_bps: i64,
    },
    Close,
}

impl PriceLane {
    fn event_id(&self, symbol: &str, date_key: &str, window: PriceWindow) -> String {
        match self {
            PriceLane::Low => format!("price_low:{symbol}:{date_key}"),
            PriceLane::Band {
                direction,
                band_bps,
                ..
            } => format!("price_band:{symbol}:{date_key}:{direction}:{band_bps}"),
            PriceLane::Close => format!("{}:{symbol}:{date_key}", window.price_id_prefix()),
        }
    }
}

fn price_lane(
    pct: f64,
    low_pct: f64,
    high_pct: f64,
    realert_step_pct: f64,
    window: PriceWindow,
) -> Option<PriceLane> {
    if window == PriceWindow::Close {
        return Some(PriceLane::Close);
    }
    let abs = pct.abs();
    if abs < low_pct {
        return None;
    }
    if abs < high_pct {
        return Some(PriceLane::Low);
    }
    let direction = if pct >= 0.0 { "up" } else { "down" };
    let band_pct =
        high_pct + ((abs - high_pct) / realert_step_pct).floor().max(0.0) * realert_step_pct;
    let band_bps = (band_pct * 100.0).round() as i64;
    let next_band_bps = ((band_pct + realert_step_pct) * 100.0).round() as i64;
    Some(PriceLane::Band {
        direction,
        band_bps,
        next_band_bps,
    })
}

fn sanitize_realert_step_pct(step_pct: f64) -> f64 {
    if step_pct.is_finite() && step_pct > 0.0 {
        step_pct
    } else {
        2.0
    }
}

fn price_payload(
    item: &Value,
    pct: f64,
    price: Option<f64>,
    date_key: &str,
    lane: Option<&PriceLane>,
) -> Value {
    let mut payload = item.clone();
    let Some(obj) = payload.as_object_mut() else {
        return payload;
    };
    obj.insert(
        "hone_price_trade_date".into(),
        Value::String(date_key.to_string()),
    );
    obj.insert("hone_price_pct".into(), json_number(pct));
    if let Some(price) = price {
        obj.insert("hone_price".into(), json_number(price));
    }
    match lane {
        Some(PriceLane::Low) => {
            obj.insert("hone_price_event_scope".into(), Value::String("low".into()));
        }
        Some(PriceLane::Band {
            direction,
            band_bps,
            next_band_bps,
        }) => {
            obj.insert(
                "hone_price_event_scope".into(),
                Value::String("band".into()),
            );
            obj.insert(
                "hone_price_direction".into(),
                Value::String((*direction).to_string()),
            );
            obj.insert("hone_price_band_bps".into(), Value::from(*band_bps));
            obj.insert(
                "hone_price_next_band_bps".into(),
                Value::from(*next_band_bps),
            );
        }
        Some(PriceLane::Close) => {
            obj.insert(
                "hone_price_event_scope".into(),
                Value::String("close".into()),
            );
        }
        None => {}
    }
    payload
}

fn price_title(symbol: &str, pct: f64, lane: Option<&PriceLane>, direction_prefix: &str) -> String {
    match lane {
        Some(PriceLane::Band {
            direction,
            band_bps,
            ..
        }) => {
            let sign = if *direction == "up" { "+" } else { "-" };
            format!("{symbol} 跨过 {sign}{}% 档", format_bps(*band_bps))
        }
        _ => format!("{symbol} {direction_prefix}{pct:.2}%"),
    }
}

fn price_summary(price: Option<f64>, pct: f64, lane: Option<&PriceLane>) -> String {
    let price_text = price
        .map(|p| format!("当前 {p:.2}"))
        .unwrap_or_else(|| "当前价格未知".into());
    let move_text = if pct >= 0.0 {
        format!("日涨 +{pct:.2}%")
    } else {
        format!("日跌 {pct:.2}%")
    };
    match lane {
        Some(PriceLane::Band {
            direction,
            next_band_bps,
            ..
        }) => {
            let sign = if *direction == "up" { "+" } else { "-" };
            format!(
                "{price_text}，{move_text}，下一档 {sign}{}%",
                format_bps(*next_band_bps)
            )
        }
        _ => format!("{price_text}，{move_text}"),
    }
}

fn format_bps(bps: i64) -> String {
    let pct = bps as f64 / 100.0;
    if bps % 100 == 0 {
        format!("{pct:.0}")
    } else {
        format!("{pct:.2}")
    }
}

fn json_number(n: f64) -> Value {
    serde_json::Number::from_f64(n)
        .map(Value::Number)
        .unwrap_or(Value::Null)
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
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 2.0, 0.001, now);
        let price = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::PriceAlert { .. }))
            .unwrap();
        assert_eq!(price.id, "price_low:BE:2026-04-22");
        assert_eq!(price.occurred_at, quote_time);
        assert_eq!(
            price
                .payload
                .get("hone_price_event_scope")
                .and_then(|v| v.as_str()),
            Some("low")
        );
    }

    #[test]
    fn intraday_high_move_uses_directional_band_id() {
        let quote_time = Utc.with_ymd_and_hms(2026, 4, 24, 13, 45, 0).unwrap();
        let now = quote_time + chrono::Duration::seconds(2);
        let raw = serde_json::json!([
            {"symbol": "AAOI", "price": 146.24, "changesPercentage": 6.18,
             "timestamp": quote_time.timestamp(), "yearHigh": 173.41, "yearLow": 11.86}
        ]);
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 2.0, 0.001, now);
        let price = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::PriceAlert { .. }))
            .unwrap();
        assert_eq!(price.id, "price_band:AAOI:2026-04-24:up:600");
        assert_eq!(price.severity, Severity::High);
        assert_eq!(
            price
                .payload
                .get("hone_price_next_band_bps")
                .and_then(|v| v.as_i64()),
            Some(800)
        );
    }

    #[test]
    fn jump_open_uses_highest_reached_band_not_all_intermediate_bands() {
        let quote_time = Utc.with_ymd_and_hms(2026, 4, 24, 13, 30, 0).unwrap();
        let now = quote_time + chrono::Duration::seconds(2);
        let raw = serde_json::json!([
            {"symbol": "AMD", "price": 342.53, "changesPercentage": 12.18,
             "timestamp": quote_time.timestamp(), "yearHigh": 360.0, "yearLow": 90.12}
        ]);
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 2.0, 0.001, now);
        let price = events
            .iter()
            .find(|e| matches!(e.kind, EventKind::PriceAlert { .. }))
            .unwrap();
        assert_eq!(price.id, "price_band:AMD:2026-04-24:up:1200");
    }

    #[test]
    fn stale_non_close_quote_is_ignored() {
        let quote_time = Utc.with_ymd_and_hms(2026, 4, 22, 13, 32, 40).unwrap();
        let now = quote_time + chrono::Duration::hours(4);
        let raw = serde_json::json!([
            {"symbol": "BE", "price": 229.75, "changesPercentage": 8.0,
             "timestamp": quote_time.timestamp(), "yearHigh": 235.35, "yearLow": 16.05}
        ]);
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 2.0, 0.001, now);
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
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 2.0, 0.001, now);
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
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 2.0, 0.001, now);
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
        let events = events_from_quotes_at(&raw, 2.5, 6.0, 2.0, 0.001, now);
        assert!(events.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn live_fmp_price_smoke() {
        use crate::subscription::SubscriptionRegistry;

        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = FmpClient::from_config(&cfg);
        let registry = Arc::new(SharedRegistry::from_registry(SubscriptionRegistry::new()));
        let poller = PricePoller::new(
            client,
            registry,
            SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
        )
        .with_thresholds(0.1, 5.0); // 很敏感，确保能看到产出
        let events = poller
            .fetch(&["AAPL".into(), "MSFT".into(), "NVDA".into()])
            .await
            .expect("FMP poll failed");
        println!("price events pulled: {}", events.len());
        for ev in events.iter().take(10) {
            println!("  [{:?}] {} · {}", ev.severity, ev.title, ev.summary);
        }
    }
}
