//! 公司行动两个独立事件源:`CorpActionCalendarPoller`(splits + dividends)
//! 与 `SecFilingsPoller`(SEC filings whitelist)。
//!
//! 历史:这两件事最初共用一个 `CorpActionPoller`,但它们的"调度依赖"完全不同
//! ——日历只看时间(不需要 watch pool),SEC filings 必须按持仓 ticker 逐个拉。
//! 把它们硬塞进同一个 EventSource 会让 `poll()` 无法干净地表达"先拉日历再
//! per-symbol 拉 sec",所以拆成两个 source。两者共用本文件里的纯函数
//! `events_from_splits` / `events_from_dividends` / `events_from_sec_filings`。
//!
//! Severity:splits/dividends=Medium。SEC filings 由 form 决定:
//! 8-K / S-1 → High,10-Q / 10-K → Medium,DEF 14A → Low。
//! id 稳定:`split:{SYM}:{DATE}` / `div:{SYM}:{EXDATE}` / `sec:{SYM}:{ACCESSION}`。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::Value;
use tracing::warn;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;
use crate::source::{EventSource, SourceSchedule};
use crate::subscription::SharedRegistry;

// ─────────────────────────────────────────────────────────────────────────────
// CorpActionCalendarPoller —— splits + dividends 日历
// ─────────────────────────────────────────────────────────────────────────────

pub struct CorpActionCalendarPoller {
    client: FmpClient,
    window_days: i64,
    schedule: SourceSchedule,
}

impl CorpActionCalendarPoller {
    pub fn new(client: FmpClient, schedule: SourceSchedule) -> Self {
        Self {
            client,
            window_days: 30,
            schedule,
        }
    }

    pub fn with_window_days(mut self, days: i64) -> Self {
        self.window_days = days;
        self
    }
}

#[async_trait]
impl EventSource for CorpActionCalendarPoller {
    fn name(&self) -> &str {
        "fmp.corp_action"
    }

    fn schedule(&self) -> SourceSchedule {
        self.schedule.clone()
    }

    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let today = Utc::now().date_naive();
        let to = today + chrono::Duration::days(self.window_days);
        let from_str = today.format("%Y-%m-%d").to_string();
        let to_str = to.format("%Y-%m-%d").to_string();

        let mut out = Vec::new();

        // Splits
        let splits_path = format!("/v3/stock_split_calendar?from={from_str}&to={to_str}");
        match self.client.get_json(&splits_path).await {
            Ok(v) => out.extend(events_from_splits(&v)),
            Err(e) => warn!("split calendar fetch failed: {e:#}"),
        }

        // Dividends
        let div_path = format!("/v3/stock_dividend_calendar?from={from_str}&to={to_str}");
        match self.client.get_json(&div_path).await {
            Ok(v) => out.extend(events_from_dividends(&v)),
            Err(e) => warn!("dividend calendar fetch failed: {e:#}"),
        }

        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SecFilingsPoller —— per-symbol 8-K 拉取
// ─────────────────────────────────────────────────────────────────────────────

pub struct SecFilingsPoller {
    client: FmpClient,
    sec_recent_hours: i64,
    forms: Vec<String>,
    registry: Arc<SharedRegistry>,
    schedule: SourceSchedule,
}

impl SecFilingsPoller {
    pub fn new(client: FmpClient, registry: Arc<SharedRegistry>, schedule: SourceSchedule) -> Self {
        Self {
            client,
            sec_recent_hours: 48,
            forms: default_forms(),
            registry,
            schedule,
        }
    }

    /// SEC filings 的时效性窗口:`fetch` 只保留 `occurred_at` 在过去这么多小时
    /// 内的条目。默认 48h——每天定时跑两次只推"新出现"的 filing,避免把两周前
    /// 的老条目反复推送。真实的幂等性由 `EventStore` 保证;窗口只是减少
    /// "冷启动首次运行时把所有历史 filing 当新事件一次性 dispatch"的冲击。
    pub fn with_sec_recent_hours(mut self, hours: i64) -> Self {
        self.sec_recent_hours = hours;
        self
    }

    /// 覆盖 form whitelist。空 vec 等于关闭整个 poller(`fetch` 返回空)。
    /// 默认见 `default_forms`(8-K / 10-Q / 10-K / S-1 / DEF 14A)。
    pub fn with_forms(mut self, forms: Vec<String>) -> Self {
        self.forms = forms;
        self
    }

    /// 拉取某 ticker 在 form whitelist 上的最近 SEC filings。每个 form 一次
    /// HTTP(FMP `/v3/sec_filings` 必须按 type 过滤,不支持一次取多 type)。
    /// 单个 form fetch 失败只 warn 不中断 —— 一个 form 的 transient 错误不该
    /// 让整个 ticker 这一 tick 失踪。
    pub async fn fetch(&self, ticker: &str) -> anyhow::Result<Vec<MarketEvent>> {
        let cutoff = Utc::now() - chrono::Duration::hours(self.sec_recent_hours);
        let mut out = Vec::new();
        for form in &self.forms {
            let encoded = encode_form(form);
            let path = format!("/v3/sec_filings/{ticker}?type={encoded}&page=0");
            match self.client.get_json(&path).await {
                Ok(raw) => out.extend(
                    events_from_sec_filings(&raw, ticker)
                        .into_iter()
                        .filter(|e| e.occurred_at >= cutoff),
                ),
                Err(e) => warn!(
                    poller = "fmp.sec_filings",
                    symbol = %ticker,
                    form = %form,
                    degraded = true,
                    "form fetch failed: {e:#}"
                ),
            }
        }
        Ok(out)
    }
}

fn default_forms() -> Vec<String> {
    vec![
        "8-K".into(),
        "10-Q".into(),
        "10-K".into(),
        "S-1".into(),
        "DEF 14A".into(),
    ]
}

/// 极简 URL 编码:本仓库 form 名用到的唯一非 ASCII-safe 字符就是空格("DEF 14A")。
/// 引入完整 percent-encoding crate 收益不抵 dep。
fn encode_form(form: &str) -> String {
    form.replace(' ', "%20")
}

#[async_trait]
impl EventSource for SecFilingsPoller {
    fn name(&self) -> &str {
        "fmp.sec_filings"
    }

    fn schedule(&self) -> SourceSchedule {
        self.schedule.clone()
    }

    async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let symbols = self.registry.load().watch_pool();
        if symbols.is_empty() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        for sym in &symbols {
            match self.fetch(sym).await {
                Ok(v) => out.extend(v),
                Err(e) => warn!(
                    poller = "fmp.sec_filings",
                    symbol = %sym,
                    degraded = true,
                    "per-symbol fetch failed: {e:#}"
                ),
            }
        }
        Ok(out)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 纯函数:FMP JSON → MarketEvent
// ─────────────────────────────────────────────────────────────────────────────

fn events_from_splits(raw: &Value) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter()
        .filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.to_string();
            let date = item.get("date")?.as_str()?.to_string();
            let naive = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?;
            let occurred_at = Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0)?);
            let numerator = item.get("numerator").and_then(|v| v.as_f64());
            let denominator = item.get("denominator").and_then(|v| v.as_f64());
            let ratio = match (numerator, denominator) {
                (Some(n), Some(d)) if d > 0.0 => format!("{n}-for-{d}"),
                _ => String::new(),
            };
            Some(MarketEvent {
                id: format!("split:{symbol}:{date}"),
                kind: EventKind::Split,
                severity: Severity::Medium,
                symbols: vec![symbol.clone()],
                occurred_at,
                title: format!("{symbol} stock split on {date}"),
                summary: ratio,
                url: None,
                source: "fmp.stock_split_calendar".into(),
                payload: item.clone(),
            })
        })
        .collect()
}

fn events_from_dividends(raw: &Value) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter()
        .filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.to_string();
            let date = item.get("date")?.as_str()?.to_string();
            let naive = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?;
            let occurred_at = Utc.from_utc_datetime(&naive.and_hms_opt(0, 0, 0)?);
            let dividend = item.get("dividend").and_then(|v| v.as_f64());
            let summary = dividend.map(|d| format!("股息 {d:.4}")).unwrap_or_default();
            Some(MarketEvent {
                id: format!("div:{symbol}:{date}"),
                kind: EventKind::Dividend,
                severity: Severity::Medium,
                symbols: vec![symbol.clone()],
                occurred_at,
                title: format!("{symbol} dividend ex-date {date}"),
                summary,
                url: None,
                source: "fmp.stock_dividend_calendar".into(),
                payload: item.clone(),
            })
        })
        .collect()
}

fn events_from_sec_filings(raw: &Value, ticker: &str) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter()
        .filter_map(|item| {
            let form = item
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let accession = item
                .get("finalLink")
                .and_then(|v| v.as_str())
                .or_else(|| item.get("link").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();
            if accession.is_empty() {
                return None;
            }
            let filed = item
                .get("fillingDate")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let accepted = item
                .get("acceptedDate")
                .and_then(|v| v.as_str())
                .unwrap_or(filed);
            let occurred_at = parse_fmp_datetime(accepted).unwrap_or_else(Utc::now);
            // Severity 按 form 业务影响排序:
            // - 8-K(突发披露)/ S-1(IPO 或追加发行,稀释信号)→ High
            // - 10-Q(季报)/ 10-K(年报)→ Medium(数字本身已被 PriceAlert 覆盖,
            //   这里推送是为了 LLM 摘要里的 backlog / 资本配置等业务信号)
            // - DEF 14A(委托书)→ Low(治理/薪酬,影响最间接)
            let severity = match form.as_str() {
                "8-K" | "S-1" => Severity::High,
                "10-Q" | "10-K" => Severity::Medium,
                "DEF 14A" => Severity::Low,
                _ => Severity::Medium,
            };
            Some(MarketEvent {
                id: format!("sec:{ticker}:{accession}"),
                kind: EventKind::SecFiling { form: form.clone() },
                severity,
                symbols: vec![ticker.to_string()],
                occurred_at,
                title: format!("{ticker} filed {form}"),
                summary: filed.to_string(),
                url: Some(accession.clone()),
                source: "fmp.sec_filings".into(),
                payload: item.clone(),
            })
        })
        .collect()
}

fn parse_fmp_datetime(s: &str) -> Option<chrono::DateTime<Utc>> {
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

    #[test]
    fn parses_splits() {
        let raw = serde_json::json!([
            {"date": "2026-05-01", "symbol": "AAPL", "numerator": 4.0, "denominator": 1.0}
        ]);
        let events = events_from_splits(&raw);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "split:AAPL:2026-05-01");
        assert!(events[0].summary.contains("4-for-1"));
        assert_eq!(events[0].severity, Severity::Medium);
    }

    #[test]
    fn parses_dividends() {
        let raw = serde_json::json!([
            {"date": "2026-05-10", "symbol": "MSFT", "dividend": 0.75}
        ]);
        let events = events_from_dividends(&raw);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "div:MSFT:2026-05-10");
        assert!(events[0].summary.contains("0.7500"));
    }

    #[test]
    fn sec_8k_maps_to_high_severity() {
        let raw = serde_json::json!([
            {
                "symbol": "TSLA",
                "type": "8-K",
                "fillingDate": "2026-04-20",
                "acceptedDate": "2026-04-20 16:01:00",
                "finalLink": "https://sec.gov/x/y/z.htm"
            }
        ]);
        let events = events_from_sec_filings(&raw, "TSLA");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, Severity::High);
        match &events[0].kind {
            EventKind::SecFiling { form } => assert_eq!(form, "8-K"),
            _ => panic!("expected SecFiling kind"),
        }
        assert!(events[0].id.starts_with("sec:TSLA:"));
    }

    #[test]
    fn sec_10q_is_medium() {
        let raw = serde_json::json!([
            {
                "symbol": "TSLA",
                "type": "10-Q",
                "fillingDate": "2026-04-20",
                "finalLink": "https://sec.gov/q.htm"
            }
        ]);
        let events = events_from_sec_filings(&raw, "TSLA");
        assert_eq!(events[0].severity, Severity::Medium);
    }

    #[test]
    fn sec_severity_mapping_per_form() {
        for (form, want) in [
            ("8-K", Severity::High),
            ("S-1", Severity::High),
            ("10-Q", Severity::Medium),
            ("10-K", Severity::Medium),
            ("DEF 14A", Severity::Low),
            ("4", Severity::Medium), // 兜底
        ] {
            let raw = serde_json::json!([
                {
                    "symbol": "X",
                    "type": form,
                    "fillingDate": "2026-04-20",
                    "finalLink": format!("https://sec.gov/{form}.htm"),
                }
            ]);
            let events = events_from_sec_filings(&raw, "X");
            assert_eq!(
                events[0].severity, want,
                "form={form} expected {want:?}, got {:?}",
                events[0].severity
            );
        }
    }

    #[test]
    fn encode_form_replaces_space() {
        assert_eq!(encode_form("8-K"), "8-K");
        assert_eq!(encode_form("DEF 14A"), "DEF%2014A");
        assert_eq!(encode_form("10-Q"), "10-Q");
    }

    #[test]
    fn default_forms_covers_whitelist() {
        let f = default_forms();
        for needed in &["8-K", "10-Q", "10-K", "S-1", "DEF 14A"] {
            assert!(
                f.iter().any(|x| x == needed),
                "default_forms missing {needed}"
            );
        }
    }

    #[test]
    fn skips_missing_required_fields() {
        let splits = events_from_splits(&serde_json::json!([
            {"symbol": "AAPL"},         // 缺 date
            {"date": "2026-05-01"}      // 缺 symbol
        ]));
        assert!(splits.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn live_fmp_corp_action_smoke() {
        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = FmpClient::from_config(&cfg);
        let poller = CorpActionCalendarPoller::new(
            client,
            SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
        );
        let events = poller.poll().await.expect("FMP poll failed");
        println!("corp_action events pulled: {}", events.len());
        for ev in events.iter().take(5) {
            println!("  [{:?}] {} · {}", ev.severity, ev.id, ev.summary);
        }
    }
}
