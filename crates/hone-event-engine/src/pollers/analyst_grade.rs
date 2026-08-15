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
//!
//! ## 汇总文扇出防御（2026-08 MU 事故）
//!
//! FMP 会把 TheFly「多股汇总文」（如 "Buy/Sell: Wall Street's top 10 stock
//! calls" / "Micron upgraded, Cisco downgraded"）里提到的**每一家券商动作**全部
//! 挂到标题第一只票的 symbol 下，产生成组的假 downgrade/upgrade（其他公司的
//! 评级动作被错配到本 ticker）。2026-08-14 MU 因此收到 6 条假 High 下调。
//! `collapse_roundup_fanout` 在源头识别这类组（标题命中汇总模式，或同一
//! `newsURL` 扇出 ≥3 条），坍缩成**一条 Medium 汇总事件**，原始 rows 保留在
//! payload 供核查；不整组丢弃是为了保留「真的被集体下调」时的信号。

use std::cmp::Reverse;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde_json::Value;

use crate::event::{EventKind, MarketEvent, Severity, is_user_visible_url};
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
        let mut events = Vec::new();
        let cutoff = Utc::now() - chrono::Duration::days(self.lookback_days);
        for t in tickers {
            let path = format!("/v4/upgrades-downgrades?symbol={t}");
            match self.client.get_json(&path).await {
                Ok(response_json) => events.extend(events_from_grades(&response_json, t, cutoff)),
                Err(e) => tracing::warn!("analyst grade fetch failed for {t}: {e:#}"),
            }
        }
        Ok(events)
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

pub(crate) fn events_from_grades(
    raw: &Value,
    ticker: &str,
    cutoff: DateTime<Utc>,
) -> Vec<MarketEvent> {
    let grade_items = match raw.as_array() {
        Some(items) => items,
        None => return vec![],
    };
    let events: Vec<MarketEvent> = grade_items
        .iter()
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
                .filter(|url| is_user_visible_url(url))
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
        .collect();
    let events = collapse_roundup_fanout(events, ticker);
    order_analyst_fanout_groups(events)
}

/// 同一 `newsURL` 扇出到本 ticker 的评级 rows 达到该值时，即使标题不像
/// 汇总文也按污染组处理——正规单公司报道极少在一篇文章里产出 ≥3 条评级记录。
const ROUNDUP_FANOUT_MIN_ROWS: usize = 3;

/// 标题是否是「多股汇总文」。命中即认为该文章下所有 rows 存在跨股错配风险。
fn is_roundup_news_title(title: &str) -> bool {
    let lower = title.to_ascii_lowercase();
    lower.contains("top analyst calls")
        || lower.contains("stock calls")
        || lower.contains("buy/sell:")
        || lower.contains("wall street's top")
        || lower.contains("opening bell")
        || (lower.contains(" upgraded") && lower.contains(" downgraded"))
}

/// 把汇总文扇出组坍缩成一条 Medium 汇总事件；非污染组原样保留。
fn collapse_roundup_fanout(events: Vec<MarketEvent>, ticker: &str) -> Vec<MarketEvent> {
    use std::collections::HashMap;

    let mut group_sizes: HashMap<String, usize> = HashMap::new();
    for event in &events {
        if let Some(url) = payload_news_url(event) {
            *group_sizes.entry(url.to_string()).or_default() += 1;
        }
    }

    let mut collapsed: HashMap<String, Vec<MarketEvent>> = HashMap::new();
    let mut ordered_urls: Vec<String> = Vec::new();
    let mut out: Vec<MarketEvent> = Vec::new();
    for event in events {
        let contaminated = payload_news_url(&event).is_some_and(|url| {
            let news_title = event
                .payload
                .get("newsTitle")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            is_roundup_news_title(news_title)
                || group_sizes.get(url).copied().unwrap_or(0) >= ROUNDUP_FANOUT_MIN_ROWS
        });
        if contaminated {
            let url = payload_news_url(&event).unwrap_or_default().to_string();
            if !collapsed.contains_key(&url) {
                ordered_urls.push(url.clone());
                // 占位保持文章在原序列中的相对位置。
                out.push(placeholder_event(&url));
            }
            collapsed.entry(url).or_default().push(event);
        } else {
            out.push(event);
        }
    }
    out.into_iter()
        .map(|event| {
            if event.source == ROUNDUP_PLACEHOLDER_SOURCE {
                let rows = collapsed.remove(&event.id).unwrap_or_default();
                roundup_summary_event(ticker, rows)
            } else {
                event
            }
        })
        .collect()
}

const ROUNDUP_PLACEHOLDER_SOURCE: &str = "__roundup_placeholder__";

fn placeholder_event(url: &str) -> MarketEvent {
    MarketEvent {
        id: url.to_string(),
        kind: EventKind::AnalystGrade,
        severity: Severity::Low,
        symbols: vec![],
        occurred_at: Utc::now(),
        title: String::new(),
        summary: String::new(),
        url: None,
        source: ROUNDUP_PLACEHOLDER_SOURCE.into(),
        payload: Value::Null,
    }
}

fn payload_news_url(event: &MarketEvent) -> Option<&str> {
    event
        .payload
        .get("newsURL")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|url| !url.is_empty())
}

/// 一组污染 rows → 一条 Medium 汇总事件。
///
/// 计数规则：downgrade/upgrade 只有评级**真的变化**才计入下调/上调；
/// prev==new 的「脏 upgrade/downgrade」计为重申；initiated/initialise 计首评。
fn roundup_summary_event(ticker: &str, rows: Vec<MarketEvent>) -> MarketEvent {
    let mut downgrades: Vec<String> = Vec::new();
    let mut upgrades: Vec<String> = Vec::new();
    let mut initiations: Vec<String> = Vec::new();
    let mut reiterations: Vec<String> = Vec::new();
    for row in &rows {
        let firm = row
            .payload
            .get("gradingCompany")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let action = row
            .payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let prev = row
            .payload
            .get("previousGrade")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let new = row
            .payload
            .get("newGrade")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        let changed = !prev.is_empty() && !new.is_empty() && !prev.eq_ignore_ascii_case(new);
        match action.as_str() {
            "downgrade" if changed => downgrades.push(format!("{firm} {prev}→{new}")),
            "upgrade" if changed => upgrades.push(format!("{firm} {prev}→{new}")),
            "initiated" | "initialise" => initiations.push(firm),
            _ => reiterations.push(firm),
        }
    }
    let mut parts: Vec<String> = Vec::new();
    if !downgrades.is_empty() {
        parts.push(format!("{} 下调", downgrades.len()));
    }
    if !upgrades.is_empty() {
        parts.push(format!("{} 上调", upgrades.len()));
    }
    if !initiations.is_empty() {
        parts.push(format!("{} 首评", initiations.len()));
    }
    if !reiterations.is_empty() {
        parts.push(format!("{} 重申", reiterations.len()));
    }
    let news_title = rows
        .first()
        .and_then(|r| r.payload.get("newsTitle"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let news_url = rows.first().and_then(payload_news_url).unwrap_or_default();
    let occurred_at = rows
        .iter()
        .map(|r| r.occurred_at)
        .max()
        .unwrap_or_else(Utc::now);
    let published_key = occurred_at.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

    let mut summary_lines: Vec<String> = Vec::new();
    if !downgrades.is_empty() {
        summary_lines.push(format!("下调: {}", downgrades.join("、")));
    }
    if !upgrades.is_empty() {
        summary_lines.push(format!("上调: {}", upgrades.join("、")));
    }
    if !initiations.is_empty() {
        summary_lines.push(format!("首评: {}", initiations.join("、")));
    }
    if !reiterations.is_empty() {
        summary_lines.push(format!("重申: {}", reiterations.join("、")));
    }
    summary_lines.push(format!(
        "来源《{news_title}》为多股汇总文，评级归属存在跨股错配可能，请以券商原文核实。"
    ));

    let url = rows.iter().find_map(|r| r.url.clone());
    let rows_payload: Vec<Value> = rows.iter().map(|r| r.payload.clone()).collect();
    MarketEvent {
        id: format!("grade_roundup:{ticker}:{published_key}"),
        kind: EventKind::AnalystGrade,
        severity: Severity::Medium,
        symbols: vec![ticker.to_string()],
        occurred_at,
        title: format!(
            "{ticker} · 券商动作汇总 {}（多股汇总文，谨慎核实）",
            parts.join(" / ")
        ),
        summary: summary_lines.join("\n"),
        url,
        source: "fmp.upgrades_downgrades".into(),
        payload: serde_json::json!({
            "hone_analyst_roundup": true,
            "newsTitle": news_title,
            "newsURL": news_url,
            "counts": {
                "downgrade": downgrades.len(),
                "upgrade": upgrades.len(),
                "initiated": initiations.len(),
                "reiterated": reiterations.len(),
            },
            "rows": rows_payload,
        }),
    }
}

fn order_analyst_fanout_groups(events: Vec<MarketEvent>) -> Vec<MarketEvent> {
    let mut ordered = Vec::with_capacity(events.len());
    let mut used = vec![false; events.len()];

    for i in 0..events.len() {
        if used[i] {
            continue;
        }
        let Some(key) = analyst_fanout_key(&events[i]) else {
            used[i] = true;
            ordered.push(events[i].clone());
            continue;
        };
        let mut group = Vec::new();
        for j in i..events.len() {
            if !used[j] && analyst_fanout_key(&events[j]).as_deref() == Some(key.as_str()) {
                group.push(j);
            }
        }
        group.sort_by_key(|idx| Reverse(analyst_signal_rank(&events[*idx])));
        for idx in group {
            used[idx] = true;
            ordered.push(events[idx].clone());
        }
    }

    ordered
}

fn analyst_fanout_key(event: &MarketEvent) -> Option<String> {
    let symbol = event.symbols.first()?.trim().to_ascii_uppercase();
    let url = event
        .payload
        .get("newsURL")
        .and_then(|v| v.as_str())
        .or(event.url.as_deref())?
        .trim();
    if symbol.is_empty() || url.is_empty() {
        return None;
    }
    Some(format!("{symbol}\n{url}"))
}

fn analyst_signal_rank(event: &MarketEvent) -> i32 {
    let action = event
        .payload
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let previous = event
        .payload
        .get("previousGrade")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let new = event
        .payload
        .get("newGrade")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let changed_rating =
        !previous.is_empty() && !new.is_empty() && !previous.eq_ignore_ascii_case(new);
    let has_target_change = target_change_from_news_title(
        event
            .payload
            .get("newsTitle")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    )
    .is_some();

    match action.as_str() {
        "downgrade" if changed_rating => 500,
        "upgrade" if changed_rating => 450,
        _ if has_target_change => 400,
        "downgrade" | "upgrade" if previous.is_empty() && !new.is_empty() => 300,
        "initiated" | "initialise" if !new.is_empty() => 250,
        _ if changed_rating => 200,
        _ => 0,
    }
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
    let mut amounts = Vec::new();
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
            amounts.push(chars[start..i].iter().collect());
        }
    }
    amounts
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
    fn same_news_url_roundup_fanout_collapses_to_one_medium_summary() {
        let published = Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
        let row = |firm: &str, action: &str, previous: Option<&str>, new: &str| {
            serde_json::json!({
                "symbol": "AMD",
                "publishedDate": published,
                "newsURL": "https://thefly.com/ajax/news_get.php?id=4346982",
                "newsTitle": "AMD upgraded, Reddit downgraded: Wall Street's top analyst calls",
                "newGrade": new,
                "previousGrade": previous,
                "gradingCompany": firm,
                "action": action,
            })
        };
        let raw = serde_json::json!([
            row("Needham", "upgrade", None, "Buy"),
            row("Citigroup", "initialise", Some("Neutral"), "Neutral"),
            row("Jefferies", "downgrade", Some("Buy"), "Hold"),
            row("Oppenheimer", "downgrade", Some("Perform"), "Perform"),
            row("BTIG", "upgrade", None, "Buy")
        ]);

        let events = events_from_grades(&raw, "AMD", Utc::now() - chrono::Duration::days(7));

        assert_eq!(events.len(), 1, "roundup fanout must collapse to one event");
        let summary_event = &events[0];
        assert_eq!(summary_event.severity, Severity::Medium);
        assert!(summary_event.id.starts_with("grade_roundup:AMD:"));
        assert!(
            summary_event.title.contains("1 下调"),
            "title = {}",
            summary_event.title
        );
        assert!(summary_event.summary.contains("Jefferies Buy→Hold"));
        assert!(summary_event.summary.contains("跨股错配"));
        assert_eq!(
            summary_event.payload.get("hone_analyst_roundup"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            summary_event.payload["rows"].as_array().map(|r| r.len()),
            Some(5)
        );
    }

    /// 2026-08-14 MU 生产事故回归：FMP 把两篇 TheFly 汇总文里其他公司的评级
    /// 动作错配到 MU，产出 6 条假 High 下调。修复后必须是 0 条 High、每篇
    /// 汇总文一条 Medium 汇总。
    #[test]
    fn mu_2026_08_14_roundup_incident_produces_no_high_events() {
        let d1 = (Utc::now() - chrono::Duration::hours(20))
            .format("%Y-%m-%dT%H:%M:%S.000Z")
            .to_string();
        let d2 = (Utc::now() - chrono::Duration::hours(18))
            .format("%Y-%m-%dT%H:%M:%S.000Z")
            .to_string();
        let row = |published: &str,
                   url: &str,
                   news_title: &str,
                   firm: &str,
                   action: &str,
                   previous: Option<&str>,
                   new: &str| {
            serde_json::json!({
                "symbol": "MU",
                "publishedDate": published,
                "newsURL": url,
                "newsTitle": news_title,
                "newGrade": new,
                "previousGrade": previous,
                "gradingCompany": firm,
                "action": action,
            })
        };
        let u1 = "https://thefly.com/ajax/news_get.php?id=4411792";
        let t1 = "Micron upgraded, Cisco downgraded: Wall Street's top analyst calls";
        let u2 = "https://thefly.com/ajax/news_get.php?id=4411877";
        let t2 = "Buy/Sell: Wall Street's top 10 stock calls this week";
        let raw = serde_json::json!([
            row(
                &d2,
                u2,
                t2,
                "HSBC",
                "downgrade",
                Some("Hold"),
                "Underperform"
            ),
            row(
                &d2,
                u2,
                t2,
                "Wells Fargo",
                "downgrade",
                Some("Overweight"),
                "Underweight"
            ),
            row(
                &d2,
                u2,
                t2,
                "Morgan Stanley",
                "upgrade",
                Some("Overweight"),
                "Overweight"
            ),
            row(
                &d2,
                u2,
                t2,
                "Jefferies",
                "downgrade",
                Some("Buy"),
                "Underperform"
            ),
            row(
                &d1,
                u1,
                t1,
                "Wedbush",
                "downgrade",
                Some("Outperform"),
                "Neutral"
            ),
            row(
                &d1,
                u1,
                t1,
                "Wolfe Research",
                "downgrade",
                Some("Outperform"),
                "Peer Perform"
            ),
            row(&d1, u1, t1, "HSBC", "downgrade", Some("Buy"), "Hold"),
            row(
                &d1,
                u1,
                t1,
                "Bernstein",
                "upgrade",
                Some("Outperform"),
                "Outperform"
            ),
            row(
                &d1,
                u1,
                t1,
                "Wells Fargo",
                "upgrade",
                Some("Overweight"),
                "Overweight"
            ),
            row(
                &d1,
                u1,
                t1,
                "Seaport Global",
                "initialise",
                Some("Buy"),
                "Buy"
            ),
            row(
                &d1,
                u1,
                t1,
                "RBC Capital",
                "initialise",
                Some("Outperform"),
                "Outperform"
            ),
            row(
                &d1,
                u1,
                t1,
                "BMO Capital",
                "initialise",
                Some("Outperform"),
                "Outperform"
            ),
            row(
                &d1,
                u1,
                t1,
                "Jefferies",
                "initialise",
                Value::Null.as_str(),
                "Buy"
            ),
        ]);

        let events = events_from_grades(&raw, "MU", Utc::now() - chrono::Duration::days(3));

        assert_eq!(events.len(), 2, "two roundup articles → two summary events");
        assert!(
            events.iter().all(|e| e.severity != Severity::High),
            "no High events may survive roundup collapse"
        );
        assert!(events.iter().all(|e| e.id.starts_with("grade_roundup:MU:")));
        let article_one = events
            .iter()
            .find(|e| e.payload["newsURL"].as_str() == Some(u1))
            .expect("summary for article 4411792");
        assert!(
            article_one.title.contains("3 下调"),
            "title = {}",
            article_one.title
        );
        assert!(
            article_one.title.contains("4 首评"),
            "title = {}",
            article_one.title
        );
        // 脏 upgrade（Overweight→Overweight）必须计入重申而不是上调。
        assert!(
            article_one.title.contains("2 重申"),
            "title = {}",
            article_one.title
        );
        assert!(
            !article_one.title.contains("上调"),
            "title = {}",
            article_one.title
        );
    }

    /// 单公司正规标题、单行记录：不允许被坍缩或降级。
    #[test]
    fn genuine_single_stock_actions_are_untouched() {
        let published = Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
        let raw = serde_json::json!([
            {
                "symbol": "MU",
                "publishedDate": published,
                "newsURL": "https://thefly.com/ajax/news_get.php?id=4411598",
                "newsTitle": "Micron upgraded to Buy from Neutral at New Street",
                "newGrade": "Buy",
                "previousGrade": "Neutral",
                "gradingCompany": "New Street",
                "action": "upgrade",
            },
            {
                "symbol": "GEV",
                "publishedDate": published,
                "newsURL": "https://www.streetinsider.com/ec_earnings/william-blair-adds-gev",
                "newsTitle": "William Blair Adds GE Vernova (GEV) to Conviction List",
                "newGrade": "Outperform",
                "previousGrade": "Buy",
                "gradingCompany": "William Blair",
                "action": "downgrade",
            },
        ]);

        let events = events_from_grades(&raw, "MU", Utc::now() - chrono::Duration::days(7));

        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.id.starts_with("grade:MU:")));
        assert_eq!(
            events[1].severity,
            Severity::High,
            "real downgrade stays High"
        );
    }

    /// 非汇总标题但同 URL 扇出 ≥3 条：按污染组坍缩（阈值规则兜底）。
    #[test]
    fn non_roundup_title_with_heavy_fanout_still_collapses() {
        let published = Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
        let row = |firm: &str| {
            serde_json::json!({
                "symbol": "NVDA",
                "publishedDate": published,
                "newsURL": "https://example.com/analysts-react",
                "newsTitle": "Analysts react to results",
                "newGrade": "Sell",
                "previousGrade": "Buy",
                "gradingCompany": firm,
                "action": "downgrade",
            })
        };
        let raw = serde_json::json!([row("A"), row("B"), row("C")]);

        let events = events_from_grades(&raw, "NVDA", Utc::now() - chrono::Duration::days(7));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].severity, Severity::Medium);
        assert!(events[0].title.contains("3 下调"));
    }

    #[test]
    fn thefly_ajax_news_url_is_hidden_but_kept_in_payload() {
        let published = Utc::now().format("%Y-%m-%dT%H:%M:%S.000Z").to_string();
        let raw = serde_json::json!([{
            "symbol": "AMD",
            "publishedDate": published,
            "newsURL": "https://www.thefly.com/ajax/news_get.php?id=4357265",
            "newsTitle": "AMD price target raised to $300 from $250 at Example",
            "newGrade": "Buy",
            "previousGrade": "Buy",
            "gradingCompany": "Example",
            "action": "hold",
        }]);

        let events = events_from_grades(&raw, "AMD", Utc::now() - chrono::Duration::days(7));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].url, None);
        assert_eq!(
            events[0].payload.get("newsURL").and_then(|v| v.as_str()),
            Some("https://www.thefly.com/ajax/news_get.php?id=4357265")
        );
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
        let fmp_config = hone_core::config::FmpConfig {
            api_key: key,
            api_keys: vec![],
            base_url: "https://financialmodelingprep.com/api".into(),
            timeout: 30,
        };
        let client = FmpClient::from_config(&fmp_config);
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
        for event in events.iter().take(10) {
            println!(
                "  [{:?}] {} · {}",
                event.severity, event.title, event.summary
            );
        }
    }
}
