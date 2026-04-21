//! MacroPoller — 拉取 FMP `v3/economic_calendar`，产出 `MacroEvent` 事件。
//!
//! - 默认窗口：今天 → +7 天
//! - Severity：默认 Low；命中高影响清单（CPI/FOMC/Nonfarm 等）升 High
//! - symbols 为空（宏观事件靠 `GlobalSubscription` 分发给所有 actor）
//! - id 稳定：`macro:{COUNTRY}:{DATE}:{EVENT_SLUG}`

use chrono::{NaiveDateTime, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;

/// 默认高影响宏观事件名关键词（小写匹配事件标题）。
const DEFAULT_HIGH_MACRO_KEYWORDS: &[&str] = &[
    "cpi",
    "ppi",
    "core pce",
    "pce",
    "nonfarm",
    "non-farm",
    "unemployment rate",
    "jobless claims",
    "fomc",
    "fed interest rate",
    "federal funds",
    "gdp",
    "ism manufacturing",
    "ism services",
    "retail sales",
    "consumer confidence",
];

pub struct MacroPoller {
    client: FmpClient,
    window_days: i64,
    keywords: Vec<String>,
}

impl MacroPoller {
    pub fn new(client: FmpClient) -> Self {
        Self {
            client,
            window_days: 7,
            keywords: DEFAULT_HIGH_MACRO_KEYWORDS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    pub fn with_window_days(mut self, days: i64) -> Self {
        self.window_days = days;
        self
    }

    pub fn with_keywords(mut self, kws: Vec<String>) -> Self {
        if !kws.is_empty() {
            self.keywords = kws.into_iter().map(|s| s.to_lowercase()).collect();
        }
        self
    }

    pub async fn poll(&self) -> anyhow::Result<Vec<MarketEvent>> {
        let today = Utc::now().date_naive();
        let to = today + chrono::Duration::days(self.window_days);
        let path = format!(
            "/v3/economic_calendar?from={}&to={}",
            today.format("%Y-%m-%d"),
            to.format("%Y-%m-%d")
        );
        let raw = self.client.get_json(&path).await?;
        Ok(events_from_calendar(&raw, &self.keywords))
    }
}

fn events_from_calendar(raw: &Value, keywords: &[String]) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter()
        .filter_map(|item| {
            let event_name = item.get("event")?.as_str()?.to_string();
            let date_raw = item.get("date")?.as_str()?.to_string();
            let country = item
                .get("country")
                .and_then(|v| v.as_str())
                .unwrap_or("GLOBAL")
                .to_string();
            let occurred_at = parse_fmp_datetime(&date_raw).unwrap_or_else(Utc::now);

            let severity = classify(&event_name, keywords);
            let slug = slugify(&event_name);
            let date_key = date_raw.chars().take(10).collect::<String>();

            // 汇总核心数值（estimate/actual/previous）形成简短摘要
            let estimate = item.get("estimate").and_then(|v| v.as_f64());
            let previous = item.get("previous").and_then(|v| v.as_f64());
            let actual = item.get("actual").and_then(|v| v.as_f64());
            let mut summary_parts: Vec<String> = Vec::new();
            if let Some(v) = actual {
                summary_parts.push(format!("actual {v}"));
            }
            if let Some(v) = estimate {
                summary_parts.push(format!("est {v}"));
            }
            if let Some(v) = previous {
                summary_parts.push(format!("prev {v}"));
            }
            let summary = summary_parts.join(" · ");

            Some(MarketEvent {
                id: format!("macro:{country}:{date_key}:{slug}"),
                kind: EventKind::MacroEvent,
                severity,
                symbols: vec![],
                occurred_at,
                title: format!("[{country}] {event_name}"),
                summary,
                url: None,
                source: "fmp.economic_calendar".into(),
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

fn classify(event_name: &str, keywords: &[String]) -> Severity {
    let n = event_name.to_lowercase();
    for kw in keywords {
        if n.contains(kw) {
            return Severity::High;
        }
    }
    Severity::Low
}

fn slugify(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kws() -> Vec<String> {
        DEFAULT_HIGH_MACRO_KEYWORDS
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn parses_calendar_and_classifies() {
        let raw = serde_json::json!([
            {
                "event": "CPI YoY",
                "date": "2026-05-13 12:30:00",
                "country": "US",
                "actual": null,
                "estimate": 3.1,
                "previous": 3.2
            },
            {
                "event": "Building Permits",
                "date": "2026-05-14 12:30:00",
                "country": "US",
                "estimate": 1450000.0
            }
        ]);
        let events = events_from_calendar(&raw, &kws());
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].severity, Severity::High);
        assert_eq!(events[1].severity, Severity::Low);
        assert!(events[0].symbols.is_empty());
        assert!(events[0].id.starts_with("macro:US:2026-05-13:cpi"));
        assert!(events[0].summary.contains("est 3.1"));
    }

    #[test]
    fn empty_or_invalid_is_empty() {
        assert!(events_from_calendar(&serde_json::json!({}), &kws()).is_empty());
        assert!(events_from_calendar(&serde_json::json!([]), &kws()).is_empty());
    }

    #[test]
    fn missing_country_defaults_to_global() {
        let raw = serde_json::json!([{"event": "Core PCE", "date": "2026-05-30"}]);
        let events = events_from_calendar(&raw, &kws());
        assert_eq!(events.len(), 1);
        assert!(events[0].id.starts_with("macro:GLOBAL:"));
        assert_eq!(events[0].severity, Severity::High);
    }

    #[test]
    fn slugify_is_deterministic() {
        assert_eq!(slugify("Core PCE YoY"), "core-pce-yoy");
        assert_eq!(slugify("FOMC!!!"), "fomc");
    }

    #[tokio::test]
    #[ignore]
    async fn live_fmp_macro_smoke() {
        let key = std::env::var("HONE_FMP_API_KEY").expect("需要 HONE_FMP_API_KEY");
        let cfg = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = FmpClient::from_config(&cfg);
        let poller = MacroPoller::new(client);
        let events = poller.poll().await.expect("FMP poll failed");
        println!("macro events pulled: {}", events.len());
        for ev in events.iter().take(10) {
            println!("  [{:?}] {} · {}", ev.severity, ev.title, ev.summary);
        }
    }
}
