//! CorpActionPoller — 拉取公司行动类事件：拆股 / 分红 / SEC filings。
//!
//! MVP 行为：
//! - `v3/stock_split_calendar` + `v3/stock_dividend_calendar`：未来窗口日历
//! - `v3/sec_filings/{ticker}?type=8-K`：最近 8-K（需知 ticker）
//! - Severity：splits/dividends=Medium，8-K=High
//! - id 稳定：`split:{SYM}:{DATE}` / `div:{SYM}:{EXDATE}` / `sec:{SYM}:{ACCESSION}`
//!
//! 目前只实现日历（不需要 ticker），SEC filings 需要逐 ticker 拉，留给调用方
//! 用 `fetch_sec_filings(ticker)` 做批量。

use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;

pub struct CorpActionPoller {
    client: FmpClient,
    window_days: i64,
    sec_recent_hours: i64,
}

impl CorpActionPoller {
    pub fn new(client: FmpClient) -> Self {
        Self {
            client,
            window_days: 30,
            sec_recent_hours: 48,
        }
    }

    pub fn with_window_days(mut self, days: i64) -> Self {
        self.window_days = days;
        self
    }

    /// SEC 8-K 的时效性窗口:`fetch_sec_filings` 只保留 `occurred_at` 在
    /// 过去这么多小时内的条目。默认 48h——每天定时跑两次也只推"新出现"的 8-K,
    /// 避免把两周前的老 filing 反复推送。真实的幂等性由上层 `EventStore` 保证;
    /// 这里的窗口只是减少"冷启动首次运行时把所有历史 8-K 当新事件一次性 dispatch"
    /// 的冲击。
    pub fn with_sec_recent_hours(mut self, hours: i64) -> Self {
        self.sec_recent_hours = hours;
        self
    }

    /// 拉取拆股 + 分红日历，合并为事件列表。
    pub async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let today = Utc::now().date_naive();
        let to = today + chrono::Duration::days(self.window_days);
        let from_str = today.format("%Y-%m-%d").to_string();
        let to_str = to.format("%Y-%m-%d").to_string();

        let mut out = Vec::new();

        // Splits
        let splits_path =
            format!("/v3/stock_split_calendar?from={from_str}&to={to_str}");
        match self.client.get_json(&splits_path).await {
            Ok(v) => out.extend(events_from_splits(&v)),
            Err(e) => tracing::warn!("split calendar fetch failed: {e:#}"),
        }

        // Dividends
        let div_path =
            format!("/v3/stock_dividend_calendar?from={from_str}&to={to_str}");
        match self.client.get_json(&div_path).await {
            Ok(v) => out.extend(events_from_dividends(&v)),
            Err(e) => tracing::warn!("dividend calendar fetch failed: {e:#}"),
        }

        Ok(out)
    }

    /// 拉取某 ticker 的最近 SEC 8-K。仅保留 `occurred_at` 在
    /// `sec_recent_hours` 窗口内的;更老的记录忽略——上游已经见过,不值得再推。
    pub async fn fetch_sec_filings(&self, ticker: &str) -> anyhow::Result<Vec<MarketEvent>> {
        let path = format!("/v3/sec_filings/{ticker}?type=8-K&page=0");
        let raw = self.client.get_json(&path).await?;
        let cutoff = Utc::now() - chrono::Duration::hours(self.sec_recent_hours);
        Ok(events_from_sec_filings(&raw, ticker)
            .into_iter()
            .filter(|e| e.occurred_at >= cutoff)
            .collect())
    }
}

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
            let summary = dividend
                .map(|d| format!("股息 {d:.4}"))
                .unwrap_or_default();
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
            let filed = item.get("fillingDate").and_then(|v| v.as_str()).unwrap_or("");
            let accepted = item
                .get("acceptedDate")
                .and_then(|v| v.as_str())
                .unwrap_or(filed);
            let occurred_at = parse_fmp_datetime(accepted).unwrap_or_else(Utc::now);
            let severity = if form == "8-K" {
                Severity::High
            } else {
                Severity::Medium
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
        let poller = CorpActionPoller::new(client);
        let events = poller.poll().await.expect("FMP poll failed");
        println!("corp_action events pulled: {}", events.len());
        for ev in events.iter().take(5) {
            println!("  [{:?}] {} · {}", ev.severity, ev.id, ev.summary);
        }
    }
}
