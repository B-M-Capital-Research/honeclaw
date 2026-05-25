//! EarningsSurprisePoller — 拉取已发布财报的 surprise%。
//!
//! 源：FMP `v3/earnings-surprises/{ticker}`。盘后 16:30 ET 之后由 scheduler
//! 触发。和 `EarningsPoller` 的区别：
//! - `EarningsPoller` 日历/预告（T-1 Medium），不含实际数据
//! - `EarningsSurprisePoller` 实际 vs 预期，含 `actualEarningResult` + `estimatedEarning`
//!
//! 严重度映射：
//! - FMP EPS surprise 只作为“财报已发布”的触发器和 LLM 输入
//! - 只有启用 earnings quality review 且近期 8-K 上下文 + LLM judgement 成功时才产出事件
//! - 不再产出 EPS-only 财报推送；review 失败 / 缺上下文 / 低置信时直接跳过
//!
//! id 稳定：`earnings_surprise:{SYMBOL}:{date}`——一家公司一个季度只有一条。

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde_json::{Value, json};
use tracing::warn;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;
use crate::pollers::earnings_quality::{EarningsQualityReviewer, apply_earnings_quality_review};
use crate::pollers::sec_enrichment::extract_filing_llm_context;
use crate::source::{EventSource, SourceSchedule};
use crate::subscription::SharedRegistry;

const EPS_PERCENT_MIN_DENOMINATOR: f64 = 0.10;
const EPS_ABS_HIGH_DELTA: f64 = 0.05;

pub struct EarningsSurprisePoller {
    client: FmpClient,
    lookback_days: i64,
    high_threshold_pct: f64,
    registry: Arc<SharedRegistry>,
    schedule: SourceSchedule,
    quality_reviewer: Option<Arc<dyn EarningsQualityReviewer>>,
    quality_sec_recent_hours: i64,
    quality_context_max_chars: usize,
    quality_min_review_confidence: f64,
    quality_min_immediate_confidence: f64,
    sec_user_agent: String,
    sec_http: reqwest::Client,
}

impl EarningsSurprisePoller {
    pub fn new(client: FmpClient, registry: Arc<SharedRegistry>, schedule: SourceSchedule) -> Self {
        let sec_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client should build");
        Self {
            client,
            lookback_days: 3,
            high_threshold_pct: 5.0,
            registry,
            schedule,
            quality_reviewer: None,
            quality_sec_recent_hours: 72,
            quality_context_max_chars: 9_000,
            quality_min_review_confidence: 0.65,
            quality_min_immediate_confidence: 0.9,
            sec_user_agent: String::new(),
            sec_http,
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

    pub fn with_quality_reviewer(
        mut self,
        reviewer: Arc<dyn EarningsQualityReviewer>,
        sec_recent_hours: i64,
        context_max_chars: usize,
        min_review_confidence: f64,
        min_immediate_confidence: f64,
        sec_user_agent: impl Into<String>,
    ) -> Self {
        self.quality_reviewer = Some(reviewer);
        self.quality_sec_recent_hours = sec_recent_hours.max(1);
        self.quality_context_max_chars = context_max_chars.max(1);
        self.quality_min_review_confidence = min_review_confidence.clamp(0.0, 1.0);
        self.quality_min_immediate_confidence = min_immediate_confidence.clamp(0.0, 1.0);
        self.sec_user_agent = sec_user_agent.into();
        self.sec_http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(self.sec_user_agent.clone())
            .build()
            .expect("reqwest client should build");
        self
    }

    /// 按指定 ticker 列表拉每一只票的最新 surprise。`EventSource::poll` 调它,
    /// 测试也可以直接用任意 ticker 列表调本函数。
    pub async fn fetch(&self, tickers: &[String]) -> anyhow::Result<Vec<MarketEvent>> {
        let mut events = Vec::new();
        let cutoff = Utc::now() - chrono::Duration::days(self.lookback_days);
        for t in tickers {
            let path = format!("/v3/earnings-surprises/{t}");
            match self.client.get_json(&path).await {
                Ok(response_json) => {
                    let candidates =
                        events_from_surprises(&response_json, t, cutoff, self.high_threshold_pct);
                    if self.quality_reviewer.is_none() {
                        if !candidates.is_empty() {
                            warn!(
                                symbol = %t,
                                candidates = candidates.len(),
                                degraded = true,
                                "earnings surprise EPS-only candidates skipped because quality reviewer is unavailable"
                            );
                        }
                        continue;
                    }
                    for mut event in candidates {
                        if self.apply_quality_review(&mut event).await {
                            events.push(event);
                        }
                    }
                }
                Err(e) => tracing::warn!("earnings surprise fetch failed for {t}: {e:#}"),
            }
        }
        Ok(events)
    }

    async fn apply_quality_review(&self, event: &mut MarketEvent) -> bool {
        let Some(reviewer) = &self.quality_reviewer else {
            return false;
        };
        let ticker = match event.symbols.first() {
            Some(ticker) => ticker.clone(),
            None => return false,
        };
        let Some(context) = self
            .fetch_recent_earnings_context(&ticker, event.occurred_at)
            .await
        else {
            return false;
        };
        let Some(review) = reviewer.review(event, &context.context).await else {
            return false;
        };
        let applied = apply_earnings_quality_review(
            event,
            review,
            Some(context.url),
            self.quality_min_review_confidence,
            self.quality_min_immediate_confidence,
        );
        if !applied {
            warn!(
                event_id = %event.id,
                degraded = true,
                "earnings quality review not applied; EPS-only candidate skipped"
            );
        }
        applied
    }

    async fn fetch_recent_earnings_context(
        &self,
        ticker: &str,
        occurred_at: DateTime<Utc>,
    ) -> Option<EarningsReviewContext> {
        if self.sec_user_agent.trim().is_empty() {
            warn!(
                symbol = %ticker,
                degraded = true,
                "earnings quality review skipped SEC fetch because User-Agent is empty"
            );
            return None;
        }

        let path = format!("/v3/sec_filings/{ticker}?type=8-K&page=0");
        let raw = match self.client.get_json(&path).await {
            Ok(raw) => raw,
            Err(e) => {
                warn!(
                    symbol = %ticker,
                    degraded = true,
                    "earnings quality review SEC filing lookup failed: {e:#}"
                );
                return None;
            }
        };
        let (url, accepted_at) =
            select_recent_8k_url(&raw, occurred_at, self.quality_sec_recent_hours)?;

        let response = match self.sec_http.get(&url).send().await {
            Ok(response) => response,
            Err(e) => {
                warn!(
                    symbol = %ticker,
                    url = %url,
                    degraded = true,
                    "earnings quality review SEC fetch failed: {e:#}"
                );
                return None;
            }
        };
        if !response.status().is_success() {
            warn!(
                symbol = %ticker,
                url = %url,
                status = %response.status(),
                degraded = true,
                "earnings quality review SEC fetch non-2xx"
            );
            return None;
        }
        let html = match response.text().await {
            Ok(html) => html,
            Err(e) => {
                warn!(
                    symbol = %ticker,
                    url = %url,
                    degraded = true,
                    "earnings quality review SEC body read failed: {e:#}"
                );
                return None;
            }
        };
        let context =
            extract_filing_llm_context(&html, "8-K", ticker, self.quality_context_max_chars);
        if context.trim().is_empty() {
            warn!(
                symbol = %ticker,
                url = %url,
                accepted_at = %accepted_at,
                degraded = true,
                "earnings quality review SEC excerpt is empty"
            );
            return None;
        }
        Some(EarningsReviewContext { url, context })
    }
}

#[async_trait]
impl EventSource for EarningsSurprisePoller {
    fn name(&self) -> &str {
        "fmp.earnings_surprise"
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

fn events_from_surprises(
    raw: &Value,
    ticker: &str,
    cutoff: DateTime<Utc>,
    high_pct: f64,
) -> Vec<MarketEvent> {
    let surprise_items = match raw.as_array() {
        Some(items) => items,
        None => return vec![],
    };
    surprise_items
        .iter()
        .filter_map(|item| event_from_surprise_item(item, ticker, cutoff, high_pct))
        .collect()
}

fn event_from_surprise_item(
    item: &Value,
    ticker: &str,
    cutoff: DateTime<Utc>,
    high_pct: f64,
) -> Option<MarketEvent> {
    let (date, occurred_at) = surprise_item_date(item)?;
    if occurred_at < cutoff {
        return None;
    }
    let eps = eps_surprise_from_item(item, high_pct)?;
    let payload = eps_annotated_payload(item, &eps);
    Some(MarketEvent {
        id: format!("earnings_surprise:{ticker}:{date}"),
        kind: EventKind::EarningsReleased,
        severity: eps.severity,
        symbols: vec![ticker.to_string()],
        occurred_at,
        title: format!("{ticker} 财报 {}", eps.title_fragment),
        summary: eps.summary,
        url: Some(press_release_fallback_url(ticker)),
        source: "fmp.earnings_surprises".into(),
        payload,
    })
}

fn surprise_item_date(item: &Value) -> Option<(String, DateTime<Utc>)> {
    let date = item.get("date").and_then(|v| v.as_str())?.to_string();
    let naive = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?;
    let occurred_at = Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0)?);
    Some((date, occurred_at))
}

fn eps_surprise_from_item(item: &Value, high_pct: f64) -> Option<EpsSurprisePresentation> {
    let actual = item.get("actualEarningResult").and_then(|v| v.as_f64())?;
    let est = item.get("estimatedEarning").and_then(|v| v.as_f64())?;
    eps_surprise_presentation(actual, est, high_pct)
}

fn press_release_fallback_url(ticker: &str) -> String {
    // FMP /v3/earnings-surprises 本身不返回 press release 链接;
    // 指向 Yahoo 的 press-releases 页面作为通用兜底。
    format!("https://finance.yahoo.com/quote/{ticker}/press-releases/")
}

fn eps_annotated_payload(item: &Value, eps: &EpsSurprisePresentation) -> Value {
    let mut payload = item.clone();
    if let Some(obj) = payload.as_object_mut() {
        obj.insert("computed_eps_delta".into(), json!(eps.delta));
        obj.insert("computed_eps_surprise_pct".into(), json!(eps.pct));
        obj.insert(
            "computed_eps_title_uses_pct".into(),
            json!(eps.title_uses_pct),
        );
    }
    payload
}

#[derive(Debug, Clone)]
struct EpsSurprisePresentation {
    title_fragment: String,
    summary: String,
    severity: Severity,
    pct: f64,
    delta: f64,
    title_uses_pct: bool,
}

#[derive(Debug, Clone)]
struct EarningsReviewContext {
    url: String,
    context: String,
}

fn eps_surprise_presentation(
    actual: f64,
    est: f64,
    high_pct: f64,
) -> Option<EpsSurprisePresentation> {
    if est.abs() < f64::EPSILON {
        return None;
    }
    let delta = actual - est;
    let pct = delta / est.abs() * 100.0;
    let title_uses_pct = actual >= 0.0 && est >= EPS_PERCENT_MIN_DENOMINATOR;
    let title_fragment = if title_uses_pct {
        let direction = if delta >= 0.0 {
            "超预期"
        } else {
            "不及预期"
        };
        format!("{direction} {pct:+.1}%")
    } else {
        format!("{} EPS差 {delta:+.2}", eps_delta_label(actual, est, delta))
    };
    let high = if title_uses_pct {
        pct.abs() >= high_pct
    } else {
        delta.abs() >= EPS_ABS_HIGH_DELTA
    };
    let summary = if title_uses_pct {
        format!("EPS 实际 {actual:.2} / 预期 {est:.2}")
    } else {
        format!("EPS 实际 {actual:.2} / 预期 {est:.2}；差值 {delta:+.2}")
    };
    Some(EpsSurprisePresentation {
        title_fragment,
        summary,
        severity: if high {
            Severity::High
        } else {
            Severity::Medium
        },
        pct,
        delta,
        title_uses_pct,
    })
}

fn eps_delta_label(actual: f64, est: f64, delta: f64) -> &'static str {
    if actual >= 0.0 && est < 0.0 {
        "扭亏超预期"
    } else if actual < 0.0 && est >= 0.0 {
        "转亏不及预期"
    } else if actual < 0.0 || est < 0.0 {
        if delta >= 0.0 {
            "亏损少于预期"
        } else {
            "亏损多于预期"
        }
    } else if delta >= 0.0 {
        "EPS高于预期"
    } else {
        "EPS低于预期"
    }
}

fn select_recent_8k_url(
    raw: &Value,
    occurred_at: DateTime<Utc>,
    recent_hours: i64,
) -> Option<(String, DateTime<Utc>)> {
    let filing_items = raw.as_array()?;
    let max_delta_secs = recent_hours.max(1) * 60 * 60;
    filing_items
        .iter()
        .filter_map(|item| recent_8k_candidate(item, occurred_at, max_delta_secs))
        .min_by_key(|(_, _, delta_secs)| *delta_secs)
        .map(|(url, accepted_at, _)| (url, accepted_at))
}

fn recent_8k_candidate(
    item: &Value,
    occurred_at: DateTime<Utc>,
    max_delta_secs: i64,
) -> Option<(String, DateTime<Utc>, i64)> {
    if item.get("type").and_then(Value::as_str).unwrap_or("") != "8-K" {
        return None;
    }
    let url = sec_filing_url(item)?;
    let accepted = item
        .get("acceptedDate")
        .and_then(Value::as_str)
        .or_else(|| item.get("fillingDate").and_then(Value::as_str))
        .or_else(|| item.get("date").and_then(Value::as_str))?;
    let accepted_at = parse_fmp_datetime(accepted)?;
    let delta_secs = (accepted_at - occurred_at).num_seconds().abs();
    if delta_secs > max_delta_secs {
        return None;
    }
    Some((url, accepted_at, delta_secs))
}

fn sec_filing_url(item: &Value) -> Option<String> {
    let url = item
        .get("finalLink")
        .and_then(Value::as_str)
        .or_else(|| item.get("link").and_then(Value::as_str))?;
    if url.trim().is_empty() {
        return None;
    }
    Some(url.to_string())
}

fn parse_fmp_datetime(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&ndt));
    }
    if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0)?));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::pollers::earnings_quality::EarningsQualityReview;

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
        assert!(events[0].summary.contains("EPS 实际 2.30 / 预期 2.00"));
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
    fn negative_near_zero_eps_uses_abs_delta_not_misleading_pct() {
        let raw = serde_json::json!([surprise(0, -0.0018, -0.02411)]);
        let events =
            events_from_surprises(&raw, "CAI", Utc::now() - chrono::Duration::days(7), 5.0);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, Severity::Medium);
        assert!(events[0].title.contains("亏损少于预期"));
        assert!(!events[0].title.contains("92.5%"));
        assert!(events[0].summary.contains("差值 +0.02"));
        assert_eq!(
            events[0]
                .payload
                .get("computed_eps_title_uses_pct")
                .and_then(Value::as_bool),
            Some(false)
        );
    }

    #[test]
    fn negative_eps_miss_uses_loss_language_not_pct_title() {
        let raw = serde_json::json!([surprise(0, -0.19, -0.05)]);
        let events =
            events_from_surprises(&raw, "AAOI", Utc::now() - chrono::Duration::days(7), 5.0);
        assert_eq!(events[0].severity, Severity::High);
        assert!(events[0].title.contains("亏损多于预期"));
        assert!(!events[0].title.contains("-280.0%"));
        assert!(events[0].summary.contains("差值 -0.14"));
    }

    #[test]
    fn selects_nearest_recent_8k_for_quality_review_context() {
        let occurred_at = Utc.with_ymd_and_hms(2026, 5, 8, 0, 0, 0).unwrap();
        let raw = serde_json::json!([
            {
                "type": "8-K",
                "acceptedDate": "2026-05-07 22:00:00",
                "finalLink": "https://sec.gov/old.htm"
            },
            {
                "type": "10-Q",
                "acceptedDate": "2026-05-08 19:00:00",
                "finalLink": "https://sec.gov/q.htm"
            },
            {
                "type": "8-K",
                "acceptedDate": "2026-05-08 21:00:00",
                "finalLink": "https://sec.gov/earnings.htm"
            }
        ]);
        let (url, accepted_at) = select_recent_8k_url(&raw, occurred_at, 72).expect("recent 8-K");
        assert_eq!(url, "https://sec.gov/old.htm");
        assert_eq!(
            accepted_at,
            Utc.with_ymd_and_hms(2026, 5, 7, 22, 0, 0).unwrap()
        );
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
    async fn fetch_without_quality_reviewer_skips_eps_only_candidates() {
        use crate::subscription::SubscriptionRegistry;

        let (base_url, routes) = spawn_test_http_server();
        routes.lock().unwrap().insert(
            "/v3/earnings-surprises/TEST".into(),
            serde_json::json!([surprise(0, -0.0018, -0.02411)]).to_string(),
        );

        let client = FmpClient::from_config(&hone_core::config::FmpConfig {
            api_key: "test-key".into(),
            api_keys: vec![],
            base_url,
            timeout: 5,
        });
        let registry = Arc::new(SharedRegistry::from_registry(SubscriptionRegistry::new()));
        let poller = EarningsSurprisePoller::new(
            client,
            registry,
            SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
        );

        let events = poller.fetch(&["TEST".into()]).await.expect("fetch");
        assert!(events.is_empty());
    }

    #[test]
    fn quality_review_applies_successful_earnings_event() {
        let raw = serde_json::json!([surprise(0, 0.18, 0.10)]);
        let mut events =
            events_from_surprises(&raw, "TEST", Utc::now() - chrono::Duration::days(3), 5.0);
        assert_eq!(events.len(), 1);
        let mut event = events.remove(0);
        let filing_url = "https://sec.example.test/filing.htm".to_string();

        let applied = apply_earnings_quality_review(
            &mut event,
            EarningsQualityReview {
                conclusion: "positive".into(),
                route: "immediate".into(),
                confidence: 0.95,
                headline_zh: "营收毛利现金流改善".into(),
                summary_zh: "收入、毛利率和经营现金流同步改善".into(),
                evidence: vec!["收入增长79%".into(), "经营现金流转正".into()],
                risks: vec![],
                override_eps_only: true,
            },
            Some(filing_url.clone()),
            0.65,
            0.9,
        );

        assert!(applied);
        assert_eq!(event.severity, Severity::High);
        assert!(event.title.contains("营收毛利现金流改善"));
        assert_eq!(event.url.as_deref(), Some(filing_url.as_str()));
        assert!(
            event
                .payload
                .get("earnings_quality_review_applied")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
    }

    fn spawn_test_http_server() -> (String, Arc<Mutex<HashMap<String, String>>>) {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind test http");
        let addr = listener.local_addr().expect("local addr");
        let routes: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let server_routes = routes.clone();
        std::thread::spawn(move || {
            for socket in listener.incoming() {
                let Ok(mut socket) = socket else {
                    continue;
                };
                let routes = server_routes.clone();
                std::thread::spawn(move || {
                    let mut buf = [0u8; 4096];
                    let n = socket.read(&mut buf).unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let path = request
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap_or("/")
                        .split('?')
                        .next()
                        .unwrap_or("/")
                        .to_string();
                    let body = routes.lock().unwrap().get(&path).cloned();
                    let (status, body) = match body {
                        Some(body) => ("200 OK", body),
                        None => ("404 Not Found", "not found".into()),
                    };
                    let response = format!(
                        "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(response.as_bytes());
                });
            }
        });
        (format!("http://{addr}"), routes)
    }

    #[tokio::test]
    #[ignore]
    async fn live_fmp_earnings_surprise_smoke() {
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
        let poller = EarningsSurprisePoller::new(
            client,
            registry,
            SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
        )
        .with_lookback_days(90);
        let events = poller
            .fetch(&["AAPL".into(), "NVDA".into()])
            .await
            .expect("FMP poll failed");
        println!("earnings surprise events pulled: {}", events.len());
        for ev in events.iter().take(5) {
            println!("  [{:?}] {} · {}", ev.severity, ev.title, ev.summary);
        }
    }
}
