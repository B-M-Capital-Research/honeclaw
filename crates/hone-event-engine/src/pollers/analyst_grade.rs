//! AnalystGradePoller — 拉取分析师评级变更事件。
//!
//! 源：FMP `v4/upgrades-downgrades?symbol={TICKER}`。相比关键词兜底，评级是
//! 事实性信号——Sell/Buy、目标价调整都会落到结构化字段里，准确性高。
//!
//! 严重度映射（基于 `action`）：
//! - `downgrade` → High（卖方下调最值得用户立即知道）
//! - `upgrade`   → Medium
//! - `initiated` / `target-raised` / `target-lowered` → Medium
//! - 其他（maintained / reiterated / hold）→ Low
//!
//! id 稳定：`grade:{SYMBOL}:{publishedDate}:{gradingCompany}`。FMP 同一条评级
//! 记录在后续拉取中 `publishedDate`+`gradingCompany` 基本不变，去重安全。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity};
use crate::fmp::FmpClient;
use crate::source::{EventSource, SourceSchedule};
use crate::subscription::SharedRegistry;

pub struct AnalystGradePoller {
    client: FmpClient,
    lookback_days: i64,
    registry: Arc<SharedRegistry>,
    schedule: SourceSchedule,
}

impl AnalystGradePoller {
    pub fn new(client: FmpClient, registry: Arc<SharedRegistry>, schedule: SourceSchedule) -> Self {
        Self {
            client,
            lookback_days: 3,
            registry,
            schedule,
        }
    }

    pub fn with_lookback_days(mut self, days: i64) -> Self {
        self.lookback_days = days;
        self
    }

    /// 按指定 ticker 列表拉评级变更。`EventSource::poll` 调它,从 registry 取
    /// watch pool 后传入;测试可以直接用任意 ticker 列表调本函数(不需要 registry)。
    pub async fn fetch(&self, tickers: &[String]) -> anyhow::Result<Vec<MarketEvent>> {
        let mut out = Vec::new();
        let cutoff = Utc::now() - chrono::Duration::days(self.lookback_days);
        for t in tickers {
            let path = format!("/v4/upgrades-downgrades?symbol={t}");
            match self.client.get_json(&path).await {
                Ok(v) => out.extend(events_from_grades(&v, t, cutoff)),
                Err(e) => tracing::warn!("analyst grade fetch failed for {t}: {e:#}"),
            }
        }
        Ok(out)
    }
}

#[async_trait]
impl EventSource for AnalystGradePoller {
    fn name(&self) -> &str {
        "fmp.analyst_grade"
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

fn events_from_grades(raw: &Value, ticker: &str, cutoff: DateTime<Utc>) -> Vec<MarketEvent> {
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return vec![],
    };
    arr.iter()
        .filter_map(|item| {
            let published = item.get("publishedDate").and_then(|v| v.as_str())?;
            let occurred_at = parse_fmp_datetime(published)?;
            if occurred_at < cutoff {
                return None;
            }
            let grading_company = item
                .get("gradingCompany")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let action = item
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let new_grade = item
                .get("newGrade")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let prev_grade = item
                .get("previousGrade")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let target_change = target_change_from_news_title(
                item.get("newsTitle").and_then(|v| v.as_str()).unwrap_or(""),
            );
            let severity = severity_from_action(&action, target_change.as_ref());
            let title = format!(
                "{ticker} · {grading_company} {}",
                summarize_action(&action, &new_grade, &prev_grade, target_change.as_ref())
            );
            let summary = summarize_payload(&new_grade, &prev_grade, target_change.as_ref());
            let url = item
                .get("newsURL")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Some(MarketEvent {
                id: format!("grade:{ticker}:{published}:{grading_company}"),
                kind: EventKind::AnalystGrade,
                severity,
                symbols: vec![ticker.to_string()],
                occurred_at,
                title,
                summary,
                url,
                source: "fmp.upgrades_downgrades".into(),
                payload: item.clone(),
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TargetChange {
    direction: TargetDirection,
    new_target: Option<String>,
    old_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TargetDirection {
    Raised,
    Lowered,
}

fn severity_from_action(action: &str, target_change: Option<&TargetChange>) -> Severity {
    if target_change.is_some() && matches!(action, "hold" | "maintained" | "reiterated" | "") {
        return Severity::Medium;
    }
    match action {
        "downgrade" => Severity::High,
        "upgrade" | "initiated" | "target-raised" | "target-lowered" => Severity::Medium,
        _ => Severity::Low,
    }
}

fn summarize_action(
    action: &str,
    new_grade: &str,
    prev_grade: &str,
    target_change: Option<&TargetChange>,
) -> String {
    if let Some(target_change) = target_change {
        let direction = match target_change.direction {
            TargetDirection::Raised => "上调目标价",
            TargetDirection::Lowered => "下调目标价",
        };
        let target = format_target_transition(target_change);
        let rating = if new_grade.is_empty() {
            String::new()
        } else {
            format!(" · 评级 {new_grade}")
        };
        return if target.is_empty() {
            format!("{direction}{rating}")
        } else {
            format!("{direction} {target}{rating}")
        };
    }
    match action {
        "downgrade" => format!("下调至 {new_grade}（原 {prev_grade}）"),
        "upgrade" => format!("上调至 {new_grade}（原 {prev_grade}）"),
        "initiated" => format!("首次覆盖 {new_grade}"),
        "target-raised" => format!("上调目标价 · 评级 {new_grade}"),
        "target-lowered" => format!("下调目标价 · 评级 {new_grade}"),
        "maintained" | "reiterated" => format!("维持 {new_grade}"),
        other if !other.is_empty() => format!("{other} · {new_grade}"),
        _ => new_grade.to_string(),
    }
}

fn summarize_payload(
    new_grade: &str,
    prev_grade: &str,
    target_change: Option<&TargetChange>,
) -> String {
    if let Some(target_change) = target_change {
        let target = format_target_transition(target_change);
        let rating = if prev_grade.is_empty() && new_grade.is_empty() {
            String::new()
        } else if prev_grade.trim().eq_ignore_ascii_case(new_grade.trim()) {
            format!("评级 {new_grade}")
        } else {
            format!("评级 {prev_grade} → {new_grade}")
        };
        return match (target.is_empty(), rating.is_empty()) {
            (false, false) => format!("目标价 {target} · {rating}"),
            (false, true) => format!("目标价 {target}"),
            (true, false) => rating,
            (true, true) => String::new(),
        };
    }
    format!("{prev_grade} → {new_grade}")
}

fn format_target_transition(target_change: &TargetChange) -> String {
    match (&target_change.old_target, &target_change.new_target) {
        (Some(old), Some(new)) => format!("{old} → {new}"),
        (None, Some(new)) => format!("至 {new}"),
        (Some(old), None) => format!("原 {old}"),
        (None, None) => String::new(),
    }
}

fn target_change_from_news_title(title: &str) -> Option<TargetChange> {
    let lower = title.to_ascii_lowercase();
    let direction = if lower.contains("price target raised")
        || lower.contains("target raised")
        || lower.contains("raises price target")
    {
        TargetDirection::Raised
    } else if lower.contains("price target lowered")
        || lower.contains("target lowered")
        || lower.contains("lowers price target")
    {
        TargetDirection::Lowered
    } else {
        return None;
    };
    let amounts = dollar_amounts(title);
    Some(TargetChange {
        direction,
        new_target: amounts.first().cloned(),
        old_target: amounts.get(1).cloned(),
    })
}

fn dollar_amounts(title: &str) -> Vec<String> {
    let chars: Vec<char> = title.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] != '$' {
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < chars.len() && (chars[i].is_ascii_digit() || matches!(chars[i], '.' | ',')) {
            i += 1;
        }
        if i > start + 1 {
            out.push(chars[start..i].iter().collect());
        }
    }
    out
}

fn parse_fmp_datetime(s: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }
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

    fn sample_grade(action: &str, days_ago: i64) -> Value {
        let d = (Utc::now() - chrono::Duration::days(days_ago))
            .format("%Y-%m-%dT%H:%M:%S.000Z")
            .to_string();
        serde_json::json!({
            "symbol": "AAPL",
            "publishedDate": d,
            "newsURL": "https://example.com/r",
            "newsTitle": "Title",
            "newGrade": "Buy",
            "previousGrade": "Hold",
            "gradingCompany": "Goldman Sachs",
            "action": action,
        })
    }

    #[test]
    fn downgrade_maps_to_high() {
        let raw = serde_json::json!([sample_grade("downgrade", 0)]);
        let events = events_from_grades(&raw, "AAPL", Utc::now() - chrono::Duration::days(7));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, Severity::High);
        assert!(events[0].title.contains("下调"));
        assert!(events[0].id.starts_with("grade:AAPL:"));
    }

    #[test]
    fn upgrade_maps_to_medium() {
        let raw = serde_json::json!([sample_grade("upgrade", 0)]);
        let events = events_from_grades(&raw, "AAPL", Utc::now() - chrono::Duration::days(7));
        assert_eq!(events[0].severity, Severity::Medium);
        assert!(events[0].title.contains("上调"));
    }

    #[test]
    fn maintained_is_low() {
        let raw = serde_json::json!([sample_grade("maintained", 0)]);
        let events = events_from_grades(&raw, "AAPL", Utc::now() - chrono::Duration::days(7));
        assert_eq!(events[0].severity, Severity::Low);
    }

    #[test]
    fn hold_with_price_target_change_is_medium_and_readable() {
        let mut row = sample_grade("hold", 0);
        row["newGrade"] = Value::String("Overweight".into());
        row["previousGrade"] = Value::String("Overweight".into());
        row["newsTitle"] =
            Value::String("Alphabet price target raised to $405 from $360 at Barclays".into());
        row["gradingCompany"] = Value::String("Barclays".into());
        let raw = serde_json::json!([row]);

        let events = events_from_grades(&raw, "GOOGL", Utc::now() - chrono::Duration::days(7));

        assert_eq!(events[0].severity, Severity::Medium);
        assert!(
            events[0]
                .title
                .contains("GOOGL · Barclays 上调目标价 $360 → $405 · 评级 Overweight"),
            "title = {}",
            events[0].title
        );
        assert_eq!(events[0].summary, "目标价 $360 → $405 · 评级 Overweight");
    }

    #[test]
    fn cutoff_filters_stale_rows() {
        let raw = serde_json::json!([sample_grade("downgrade", 30)]);
        let events = events_from_grades(&raw, "AAPL", Utc::now() - chrono::Duration::days(3));
        assert!(events.is_empty());
    }

    #[test]
    fn missing_published_date_is_skipped() {
        let raw = serde_json::json!([{"symbol": "AAPL", "action": "upgrade"}]);
        let events = events_from_grades(&raw, "AAPL", Utc::now() - chrono::Duration::days(3));
        assert!(events.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn live_fmp_analyst_grade_smoke() {
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
        let poller = AnalystGradePoller::new(
            client,
            registry,
            SourceSchedule::FixedInterval(std::time::Duration::from_secs(60)),
        )
        .with_lookback_days(14);
        let events = poller
            .fetch(&["AAPL".into(), "NVDA".into()])
            .await
            .expect("FMP poll failed");
        println!("analyst grade events pulled: {}", events.len());
        for ev in events.iter().take(10) {
            println!("  [{:?}] {} · {}", ev.severity, ev.title, ev.summary);
        }
    }
}
