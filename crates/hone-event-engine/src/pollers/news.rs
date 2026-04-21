//! NewsPoller — 拉取 FMP `v3/stock_news`，产出 `NewsCritical` 事件。
//!
//! MVP 行为：
//! - `poll()` 从 FMP 拉一页最新新闻（可选 ticker 过滤，`None` 表示全局流）
//! - 默认 severity = Low；title/text 命中关键词库 → 升级为 High
//! - id 直接用文章 URL 做稳定去重；缺 URL 则回落到 "title+date" 组合
//! - 关键词库先内置一组保守的"高影响"词（破产、SEC 调查、召回、被起诉、CEO 辞任、收购等）

use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;

/// 默认高影响关键词（小写匹配）。后续可从 config 注入覆盖。
const DEFAULT_CRITICAL_KEYWORDS: &[&str] = &[
    "bankruptcy",
    "bankrupt",
    "delist",
    "halt trading",
    "trading halted",
    "sec investigation",
    "sec probe",
    "sec charges",
    "sec settles",
    "recall",
    "fraud",
    "lawsuit",
    "class action",
    "short report",
    "short-seller",
    "hindenburg",
    "muddy waters",
    "guidance cut",
    "cuts guidance",
    "lowers guidance",
    "ceo resigns",
    "ceo steps down",
    "cfo resigns",
    "cfo steps down",
    "acquired by",
    "agrees to acquire",
    "merger",
    "buyout",
    "going private",
    "data breach",
    "cyberattack",
];

pub struct NewsPoller {
    client: FmpClient,
    tickers: Option<Vec<String>>,
    page_limit: u32,
    keywords: Vec<String>,
}

impl NewsPoller {
    pub fn new(client: FmpClient) -> Self {
        Self {
            client,
            tickers: None,
            page_limit: 50,
            keywords: DEFAULT_CRITICAL_KEYWORDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn with_tickers(mut self, tickers: Vec<String>) -> Self {
        self.tickers = if tickers.is_empty() {
            None
        } else {
            Some(tickers)
        };
        self
    }

    pub fn with_page_limit(mut self, limit: u32) -> Self {
        self.page_limit = limit;
        self
    }

    pub fn with_keywords(mut self, kws: Vec<String>) -> Self {
        if !kws.is_empty() {
            self.keywords = kws.into_iter().map(|s| s.to_lowercase()).collect();
        }
        self
    }

    pub async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let mut path = format!("/v3/stock_news?limit={}", self.page_limit);
        if let Some(ts) = &self.tickers {
            path.push_str("&tickers=");
            path.push_str(&ts.join(","));
        }
        let raw = self.client.get_json(&path).await?;
        Ok(events_from_stock_news(&raw, &self.keywords))
    }
}

/// FMP stock_news 响应 → MarketEvent 列表。
fn events_from_stock_news(raw: &Value, keywords: &[String]) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };

    arr.iter()
        .filter_map(|item| {
            let title = item.get("title")?.as_str()?.to_string();
            let published_raw = item.get("publishedDate")?.as_str()?.to_string();
            let occurred_at = parse_fmp_datetime(&published_raw).unwrap_or_else(Utc::now);

            let symbol = item
                .get("symbol")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let url = item.get("url").and_then(|v| v.as_str()).map(String::from);
            let text = item
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let site = item
                .get("site")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let severity = classify_severity(&title, &text, keywords);
            let id = match &url {
                Some(u) => format!("news:{u}"),
                None => format!("news:{published_raw}:{}", truncate(&title, 64)),
            };
            let symbols = symbol.map(|s| vec![s]).unwrap_or_default();
            let summary_snippet = truncate(&text, 240);

            Some(MarketEvent {
                id,
                kind: EventKind::NewsCritical,
                severity,
                symbols,
                occurred_at,
                title,
                summary: summary_snippet,
                url,
                source: if site.is_empty() {
                    "fmp.stock_news".into()
                } else {
                    format!("fmp.stock_news:{site}")
                },
                payload: item.clone(),
            })
        })
        .collect()
}

fn parse_fmp_datetime(s: &str) -> Option<chrono::DateTime<Utc>> {
    // FMP 格式如 "2026-04-20 14:30:00"（UTC）
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&ndt));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
        return Some(Utc.from_utc_datetime(&ndt));
    }
    None
}

fn classify_severity(title: &str, text: &str, keywords: &[String]) -> Severity {
    let t = title.to_lowercase();
    let body = text.to_lowercase();
    for kw in keywords {
        if t.contains(kw) || body.contains(kw) {
            return Severity::High;
        }
    }
    Severity::Low
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_kws() -> Vec<String> {
        DEFAULT_CRITICAL_KEYWORDS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn parses_typical_stock_news() {
        let raw = serde_json::json!([
            {
                "symbol": "AAPL",
                "publishedDate": "2026-04-21 08:15:00",
                "title": "Apple beats estimates on services strength",
                "image": "",
                "site": "reuters.com",
                "text": "Apple Inc reported Q2 results above expectations driven by services growth ...",
                "url": "https://example.com/apple-beats"
            },
            {
                "symbol": "TSLA",
                "publishedDate": "2026-04-21 09:00:00",
                "title": "Tesla faces SEC investigation over disclosures",
                "site": "wsj.com",
                "text": "The SEC has opened a probe into Tesla's ...",
                "url": "https://example.com/tsla-sec"
            }
        ]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].severity, Severity::Low);
        assert_eq!(events[1].severity, Severity::High);
        assert!(events[1].title.to_lowercase().contains("sec"));
        assert_eq!(events[0].id, "news:https://example.com/apple-beats");
        assert!(events[0].touches("AAPL"));
    }

    #[test]
    fn missing_url_falls_back_to_date_title_id() {
        let raw = serde_json::json!([{
            "symbol": "NVDA",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Nvidia announces chip partnership"
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events.len(), 1);
        assert!(events[0].id.starts_with("news:2026-04-21"));
        assert!(events[0].url.is_none());
    }

    #[test]
    fn keyword_match_is_case_insensitive_and_body_searched() {
        let raw = serde_json::json!([{
            "symbol": "ACME",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Acme hits new record",
            "text": "Despite the Hindenburg short report published today, Acme ..."
        }]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events[0].severity, Severity::High);
    }

    #[test]
    fn custom_keywords_override_default() {
        let raw = serde_json::json!([{
            "symbol": "X",
            "publishedDate": "2026-04-21 10:00:00",
            "title": "Boring quarterly update",
            "text": "Nothing special happened."
        }]);
        // 默认关键词：Low
        let low = events_from_stock_news(&raw, &default_kws());
        assert_eq!(low[0].severity, Severity::Low);
        // 注入新词匹配："boring"
        let high = events_from_stock_news(&raw, &vec!["boring".into()]);
        assert_eq!(high[0].severity, Severity::High);
    }

    #[test]
    fn skips_entries_missing_title_or_published_date() {
        let raw = serde_json::json!([
            {"symbol": "X", "publishedDate": "2026-04-21 10:00:00"}, // 缺 title
            {"symbol": "Y", "title": "no date"},                      // 缺 publishedDate
            {"symbol": "Z", "publishedDate": "2026-04-21 10:00:00", "title": "ok"}
        ]);
        let events = events_from_stock_news(&raw, &default_kws());
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "ok");
    }

    /// 真实 FMP 烟测；默认忽略。
    /// `HONE_FMP_API_KEY=xxx cargo test -p hone-event-engine -- --ignored live_fmp_news_smoke --nocapture`
    #[tokio::test]
    #[ignore]
    async fn live_fmp_news_smoke() {
        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = FmpClient::from_config(&cfg);
        let poller = NewsPoller::new(client).with_page_limit(5);
        let events = poller.poll().await.expect("FMP poll failed");
        println!("news events pulled: {}", events.len());
        for ev in events.iter().take(5) {
            println!("  [{:?}] {} · {}", ev.severity, ev.title, ev.id);
        }
        assert!(!events.is_empty());
    }
}
