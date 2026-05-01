//! ExtendedHoursPoller — 美股盘前/盘后 30min 振幅监控。
//!
//! ## 为什么单开一路
//!
//! 现有 `PricePoller` 走 FMP `/v3/quote`,该 endpoint 在盘前/盘后**不更新**
//! `timestamp`,被 `FRESH_QUOTE_MAX_AGE_SECS=15min` 判定为 stale 直接跳过。结果是
//! 用户在 GOOGL 2026-04-29 财报夜盘后整整 16h 完全没收到价格信号,直到 ET 09:30
//! 开盘 PricePoller 才捕捉到累计 +6%。中间 ET 16:00–17:30 三个 30min 窗振幅
//! +4.80% / +4.30% / +3.12% 全程蒙眼。
//!
//! 本通道改打 `/v3/historical-chart/1min/{sym}?extended=true`(POC 验证覆盖
//! ET 04:00–19:59,延迟 ~2 min),每 30min 拉一次,只在 ET pre/post 窗口工作。
//!
//! ## 触发与去重
//!
//! - 振幅 = `(max_high - min_low) / prev_close * 100`(过去 30min 内 1min K 的极值)
//! - 振幅 ≥ `low_pct` → 产出 PriceAlert(系统 `high_pct` 之上 High,之间 Low)
//! - 写进 `payload.changesPercentage` 的是**有符号振幅**:幅度 = amp_pct,
//!   符号取自 `last_price - prev_close`。这样 router 的 `price_high_pct_override`
//!   / `_up_override` / `_down_override` 路径无修改即可命中 —— 用户配 4% 阈值
//!   就能在盘前盘后等价升级 Low → High 即时推。
//! - id 稳定:`extended:{SYM}:{ET_DATE}:{pre|post}`。`EventStore::insert_event`
//!   的 `INSERT IGNORE` 保证同 session 同 ticker 一天最多触达 sink 一次,
//!   后续 30min tick 同 id 自动幂等丢弃 ——「同股同 session 只发一次」由 store
//!   层做掉,本 poller 不存额外状态。
//!
//! ## 窗口判定
//!
//! 用 `chrono_tz::US::Eastern`,DST-aware。pre = Mon-Fri 04:00–09:30 ET,
//! post = Mon-Fri 16:00–20:00 ET。窗口外 `poll()` 立即 `Ok(vec![])` 返回,
//! 不做任何 HTTP 请求。判定纯粹是「市场属性」,与用户 timezone 无关 —— 用户
//! TZ 只影响 quiet_hours / digest_slots 的时间换算,那部分由 dispatch / digest
//! 层各自处理,本 poller 不感知。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Timelike, Utc, Weekday};
use chrono_tz::US::Eastern;
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;
use crate::source::{EventSource, SourceSchedule};
use crate::subscription::SharedRegistry;

/// 默认 30 分钟拉一次。窗口外 poll() 直接 no-op,所以非交易时段几乎零开销。
const DEFAULT_INTERVAL_SECS: u64 = 30 * 60;
/// 每 tick 看过去多久的 1min K(必须 ≥ 拉取间隔以避免漏窗)。
const WINDOW_LOOKBACK_MINS: i64 = 30;
/// 找 prev close 时最多回溯多少个交易日(跨长假兜底)。
const PREV_CLOSE_LOOKBACK_DAYS: i64 = 7;

pub struct ExtendedHoursPoller {
    client: FmpClient,
    registry: Arc<SharedRegistry>,
    schedule: SourceSchedule,
    low_pct: f64,
    high_pct: f64,
    /// (symbol, et_date) → close。同一交易日内重复命中直接复用,避免 N 次
    /// historical-price-full 调用。每天首次 poll 该 ticker 时 miss,触发拉取。
    prev_close_cache: Arc<Mutex<HashMap<String, (NaiveDate, f64)>>>,
}

impl ExtendedHoursPoller {
    pub fn new(client: FmpClient, registry: Arc<SharedRegistry>) -> Self {
        Self {
            client,
            registry,
            schedule: SourceSchedule::FixedInterval(Duration::from_secs(DEFAULT_INTERVAL_SECS)),
            low_pct: 2.5,
            high_pct: 6.0,
            prev_close_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_thresholds(mut self, low_pct: f64, high_pct: f64) -> Self {
        self.low_pct = low_pct;
        self.high_pct = high_pct;
        self
    }

    pub fn with_schedule(mut self, schedule: SourceSchedule) -> Self {
        self.schedule = schedule;
        self
    }

    async fn fetch_prev_close(&self, symbol: &str, et_today: NaiveDate) -> anyhow::Result<f64> {
        if let Ok(map) = self.prev_close_cache.lock() {
            if let Some((cached_day, close)) = map.get(symbol) {
                if *cached_day == et_today {
                    return Ok(*close);
                }
            }
        }

        let mut day = et_today;
        for _ in 0..PREV_CLOSE_LOOKBACK_DAYS {
            day = previous_weekday(day);
            let path =
                format!("/v3/historical-price-full/{symbol}?from={day}&to={day}&serietype=line");
            let resp = match self.client.get_json(&path).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!("extended_hours: prev_close fetch {symbol} {day} failed: {e:#}");
                    continue;
                }
            };
            if let Some(close) = resp
                .get("historical")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("close"))
                .and_then(|v| v.as_f64())
            {
                if let Ok(mut map) = self.prev_close_cache.lock() {
                    map.insert(symbol.to_string(), (et_today, close));
                }
                return Ok(close);
            }
        }
        anyhow::bail!("no prev close found for {symbol} after {PREV_CLOSE_LOOKBACK_DAYS} days")
    }

    async fn fetch_window_bars(&self, symbol: &str) -> anyhow::Result<Vec<Bar>> {
        let path = format!("/v3/historical-chart/1min/{symbol}?extended=true");
        let resp = self.client.get_json(&path).await?;
        let arr = match resp.as_array() {
            Some(a) => a.clone(),
            None => return Ok(vec![]),
        };
        Ok(arr.into_iter().filter_map(parse_bar).collect())
    }
}

#[async_trait]
impl EventSource for ExtendedHoursPoller {
    fn name(&self) -> &str {
        "fmp.extended_hours"
    }

    fn schedule(&self) -> SourceSchedule {
        self.schedule.clone()
    }

    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let now = Utc::now();
        let Some((session, et_date)) = current_extended_session(now) else {
            return Ok(vec![]);
        };

        let symbols = self.registry.load().watch_pool();
        if symbols.is_empty() {
            return Ok(vec![]);
        }

        let mut out = Vec::new();
        for symbol in &symbols {
            let prev_close = match self.fetch_prev_close(symbol, et_date).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("extended_hours: skip {symbol}, no prev_close: {e:#}");
                    continue;
                }
            };
            let bars = match self.fetch_window_bars(symbol).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::warn!("extended_hours: 1min K fetch {symbol} failed: {e:#}");
                    continue;
                }
            };
            let recent = filter_recent_bars(&bars, now, WINDOW_LOOKBACK_MINS);
            if let Some(event) = build_event_if_threshold_met(
                symbol,
                session,
                et_date,
                prev_close,
                &recent,
                self.low_pct,
                self.high_pct,
                now,
            ) {
                out.push(event);
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone)]
struct Bar {
    /// ET 本地壁钟时间(FMP 这条 endpoint 直接给 ET local string,无 TZ 后缀)。
    timestamp_et: NaiveDateTime,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

fn parse_bar(item: Value) -> Option<Bar> {
    let date = item.get("date").and_then(|v| v.as_str())?;
    let ts = NaiveDateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S").ok()?;
    Some(Bar {
        timestamp_et: ts,
        high: item.get("high").and_then(|v| v.as_f64())?,
        low: item.get("low").and_then(|v| v.as_f64())?,
        close: item.get("close").and_then(|v| v.as_f64())?,
        volume: item.get("volume").and_then(|v| v.as_f64()).unwrap_or(0.0),
    })
}

fn filter_recent_bars(bars: &[Bar], now: DateTime<Utc>, lookback_mins: i64) -> Vec<Bar> {
    let cutoff_et = (now - chrono::Duration::minutes(lookback_mins))
        .with_timezone(&Eastern)
        .naive_local();
    bars.iter()
        .filter(|b| b.timestamp_et >= cutoff_et)
        .cloned()
        .collect()
}

/// 判断当前是否在 ET pre / post 窗口内,返回 (session_tag, et_date_for_id)。
///
/// pre 用「当天 ET 日期」(04:00–09:30 ET 都属同一天的 pre)。
/// post 用「当天 ET 日期」(16:00–20:00 ET 都属同一天的 post)。
/// 周末与节假日不进入(节假日靠 prev_close 拉到空 → 整体 skip,不在此函数兜底)。
fn current_extended_session(now: DateTime<Utc>) -> Option<(&'static str, NaiveDate)> {
    let et = now.with_timezone(&Eastern);
    if matches!(et.weekday(), Weekday::Sat | Weekday::Sun) {
        return None;
    }
    let h = et.hour();
    let m = et.minute();
    let date = et.date_naive();
    // pre: 04:00 ≤ t < 09:30
    if h >= 4 && (h < 9 || (h == 9 && m < 30)) {
        return Some(("pre", date));
    }
    // post: 16:00 ≤ t < 20:00
    if (16..20).contains(&h) {
        return Some(("post", date));
    }
    None
}

fn previous_weekday(d: NaiveDate) -> NaiveDate {
    let mut prev = d - chrono::Duration::days(1);
    while matches!(prev.weekday(), Weekday::Sat | Weekday::Sun) {
        prev -= chrono::Duration::days(1);
    }
    prev
}

#[allow(clippy::too_many_arguments)]
fn build_event_if_threshold_met(
    symbol: &str,
    session: &'static str,
    et_date: NaiveDate,
    prev_close: f64,
    bars: &[Bar],
    low_pct: f64,
    high_pct: f64,
    now: DateTime<Utc>,
) -> Option<MarketEvent> {
    if prev_close <= 0.0 || bars.is_empty() {
        return None;
    }
    let max_high = bars.iter().map(|b| b.high).fold(f64::MIN, f64::max);
    let min_low = bars.iter().map(|b| b.low).fold(f64::MAX, f64::min);
    if !max_high.is_finite() || !min_low.is_finite() {
        return None;
    }
    let amp_pct = (max_high - min_low) / prev_close * 100.0;
    if amp_pct < low_pct {
        return None;
    }
    let last_bar = bars.iter().max_by_key(|b| b.timestamp_et)?;
    let last_price = last_bar.close;
    let net_chg_pct = (last_price - prev_close) / prev_close * 100.0;
    let direction_sign = if net_chg_pct >= 0.0 { 1.0 } else { -1.0 };
    let signed_amp_pct = amp_pct * direction_sign;
    let volume_30min: f64 = bars.iter().map(|b| b.volume).sum();

    let severity = if amp_pct >= high_pct {
        Severity::High
    } else {
        Severity::Low
    };

    let session_label = match session {
        "pre" => "盘前",
        "post" => "盘后",
        _ => "扩展时段",
    };
    let dir_text = if net_chg_pct >= 0.0 { "+" } else { "" };
    let title = format!("{symbol} {session_label} 振幅 {amp_pct:.2}% · 现价 {last_price:.2}");
    let summary = format!(
        "30min 振幅 {amp_pct:.2}%(高 {max_high:.2} / 低 {min_low:.2})· 现价 {last_price:.2} · 较昨收 {dir_text}{net_chg_pct:.2}%"
    );

    let payload = serde_json::json!({
        // 字段名与 FMP /v3/quote 对齐 —— router 的 price_override_threshold
        // 直接读 changesPercentage 做 per-actor 升级判断。这里写 signed_amp_pct
        // (符号 = 净涨跌方向, 幅度 = 振幅)以保留方向感同时让用户阈值生效。
        "changesPercentage": signed_amp_pct,
        "price": last_price,
        // 扩展字段 —— renderer / 调试用。
        "amp_pct": amp_pct,
        "net_chg_pct": net_chg_pct,
        "hi": max_high,
        "lo": min_low,
        "prev_close": prev_close,
        "volume_30min": volume_30min,
        "session": session,
        "et_date": et_date.to_string(),
    });

    let pct_change_bps = (signed_amp_pct * 100.0).round() as i64;

    Some(MarketEvent {
        id: format!("extended:{symbol}:{et_date}:{session}"),
        kind: EventKind::PriceAlert {
            pct_change_bps,
            window: session.to_string(),
        },
        severity,
        symbols: vec![symbol.to_string()],
        occurred_at: now,
        title,
        summary,
        url: None,
        source: "fmp.extended_hours".into(),
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn et(year: i32, mon: u32, day: u32, hour: u32, min: u32) -> DateTime<Utc> {
        Eastern
            .with_ymd_and_hms(year, mon, day, hour, min, 0)
            .single()
            .expect("valid ET datetime")
            .with_timezone(&Utc)
    }

    #[test]
    fn pre_market_window_detected_on_weekday() {
        // 2026-04-30 (Thursday) 04:30 ET → pre
        let now = et(2026, 4, 30, 4, 30);
        let (session, date) = current_extended_session(now).unwrap();
        assert_eq!(session, "pre");
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 4, 30).unwrap());
    }

    #[test]
    fn post_market_window_detected_on_weekday() {
        let now = et(2026, 4, 30, 18, 0);
        let (session, _date) = current_extended_session(now).unwrap();
        assert_eq!(session, "post");
    }

    #[test]
    fn regular_session_returns_none() {
        // 11:00 ET on a weekday
        let now = et(2026, 4, 30, 11, 0);
        assert!(current_extended_session(now).is_none());
    }

    #[test]
    fn weekend_returns_none_even_in_window_hour() {
        // 2026-05-02 is a Saturday
        let now = et(2026, 5, 2, 5, 0);
        assert!(current_extended_session(now).is_none());
    }

    #[test]
    fn pre_market_boundary_at_0930_excludes_regular_open() {
        let exactly_0930 = et(2026, 4, 30, 9, 30);
        assert!(
            current_extended_session(exactly_0930).is_none(),
            "09:30 ET 是常规开盘,不应再算 pre"
        );
        let just_before = et(2026, 4, 30, 9, 29);
        assert_eq!(current_extended_session(just_before).unwrap().0, "pre");
    }

    #[test]
    fn post_market_boundary_at_2000_excludes_after_hours_close() {
        let exactly_2000 = et(2026, 4, 30, 20, 0);
        assert!(current_extended_session(exactly_2000).is_none());
        let just_before = et(2026, 4, 30, 19, 59);
        assert_eq!(current_extended_session(just_before).unwrap().0, "post");
    }

    #[test]
    fn previous_weekday_skips_weekend() {
        let mon = NaiveDate::from_ymd_opt(2026, 5, 4).unwrap(); // Mon
        assert_eq!(
            previous_weekday(mon),
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap() // Fri
        );
        let tue = NaiveDate::from_ymd_opt(2026, 5, 5).unwrap();
        assert_eq!(
            previous_weekday(tue),
            NaiveDate::from_ymd_opt(2026, 5, 4).unwrap()
        );
    }

    fn make_bar(et_str: &str, high: f64, low: f64, close: f64, volume: f64) -> Bar {
        Bar {
            timestamp_et: NaiveDateTime::parse_from_str(et_str, "%Y-%m-%d %H:%M:%S").unwrap(),
            high,
            low,
            close,
            volume,
        }
    }

    #[test]
    fn amp_below_low_threshold_yields_no_event() {
        let bars = vec![
            make_bar("2026-04-30 04:01:00", 100.5, 100.0, 100.2, 1_000.0),
            make_bar("2026-04-30 04:02:00", 100.8, 100.3, 100.6, 1_500.0),
        ];
        let now = et(2026, 4, 30, 4, 5);
        let event = build_event_if_threshold_met(
            "AAPL",
            "pre",
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
            100.0, // prev_close — amp = (100.8-100.0)/100 = 0.8% < 2.5%
            &bars,
            2.5,
            6.0,
            now,
        );
        assert!(event.is_none());
    }

    #[test]
    fn amp_in_low_band_yields_low_severity() {
        let bars = vec![
            make_bar("2026-04-30 04:01:00", 103.0, 100.0, 102.5, 50_000.0),
            make_bar("2026-04-30 04:02:00", 103.5, 102.0, 103.0, 60_000.0),
        ];
        let now = et(2026, 4, 30, 4, 5);
        let event = build_event_if_threshold_met(
            "AAPL",
            "pre",
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
            100.0, // amp = (103.5-100.0)/100 = 3.5%
            &bars,
            2.5,
            6.0,
            now,
        )
        .unwrap();
        assert_eq!(event.severity, Severity::Low);
        assert_eq!(event.id, "extended:AAPL:2026-04-30:pre");
        match &event.kind {
            EventKind::PriceAlert { window, .. } => assert_eq!(window, "pre"),
            _ => panic!("expected PriceAlert"),
        }
    }

    #[test]
    fn amp_above_high_threshold_yields_high_severity() {
        let bars = vec![
            make_bar("2026-04-30 16:01:00", 108.0, 100.0, 107.5, 200_000.0),
            make_bar("2026-04-30 16:02:00", 109.0, 107.0, 108.5, 250_000.0),
        ];
        let now = et(2026, 4, 30, 16, 5);
        let event = build_event_if_threshold_met(
            "GOOGL",
            "post",
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
            100.0, // amp = (109-100)/100 = 9%
            &bars,
            2.5,
            6.0,
            now,
        )
        .unwrap();
        assert_eq!(event.severity, Severity::High);
        assert_eq!(event.id, "extended:GOOGL:2026-04-30:post");
        let payload = &event.payload;
        // signed_amp = +9% (last_price 108.5 > prev_close 100)
        let cp = payload.get("changesPercentage").unwrap().as_f64().unwrap();
        assert!((cp - 9.0).abs() < 0.001);
        assert_eq!(
            payload.get("session").and_then(|v| v.as_str()),
            Some("post")
        );
        assert!((payload.get("amp_pct").unwrap().as_f64().unwrap() - 9.0).abs() < 0.001);
    }

    #[test]
    fn signed_amp_uses_net_direction_sign_not_amp_sign() {
        // 盘后先冲 +5%(106) 再回落到 -3%(97);amp = (106-97)/100 = 9%,
        // 净 chg 为负 → signed_amp = -9%。这是路由层 down_override 命中的关键场景。
        let bars = vec![
            make_bar("2026-04-30 16:01:00", 106.0, 99.0, 105.0, 100_000.0),
            make_bar("2026-04-30 16:30:00", 100.0, 97.0, 97.5, 150_000.0),
        ];
        let now = et(2026, 4, 30, 16, 32);
        let event = build_event_if_threshold_met(
            "TSLA",
            "post",
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
            100.0,
            &bars,
            2.5,
            6.0,
            now,
        )
        .unwrap();
        let cp = event
            .payload
            .get("changesPercentage")
            .unwrap()
            .as_f64()
            .unwrap();
        assert!(cp < 0.0, "净跌应给 changesPercentage 负号");
        assert!((cp.abs() - 9.0).abs() < 0.001);
    }

    #[test]
    fn empty_bars_produce_no_event() {
        let event = build_event_if_threshold_met(
            "AAPL",
            "pre",
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
            100.0,
            &[],
            2.5,
            6.0,
            Utc::now(),
        );
        assert!(event.is_none());
    }

    #[test]
    fn zero_prev_close_produces_no_event() {
        let bars = vec![make_bar("2026-04-30 04:01:00", 100.0, 90.0, 95.0, 1.0)];
        let event = build_event_if_threshold_met(
            "AAPL",
            "pre",
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
            0.0,
            &bars,
            2.5,
            6.0,
            Utc::now(),
        );
        assert!(event.is_none());
    }

    #[test]
    fn filter_recent_bars_drops_old_bars() {
        let now = et(2026, 4, 30, 4, 30);
        let bars = vec![
            // 04:00 ET = 30min ago → keep (cutoff is exactly 04:00)
            make_bar("2026-04-30 04:00:00", 100.0, 99.0, 99.5, 1.0),
            // 03:55 ET → drop
            make_bar("2026-04-30 03:55:00", 100.0, 99.0, 99.5, 1.0),
            // 04:25 ET → keep
            make_bar("2026-04-30 04:25:00", 100.0, 99.0, 99.5, 1.0),
        ];
        let kept = filter_recent_bars(&bars, now, 30);
        assert_eq!(kept.len(), 2);
        assert!(
            kept.iter()
                .all(|b| b.timestamp_et.format("%H:%M").to_string() != "03:55")
        );
    }
}
