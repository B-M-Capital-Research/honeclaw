//! Actor-scoped weekly review and forward calendar.
//!
//! The product intentionally separates a scheduled event from its outcome.
//! Finance-calendar rows prove that a date is on the calendar, while only
//! attributed, confirmed key-event rows are allowed to describe what changed.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use chrono::{Datelike, Days, NaiveDate, Utc};
use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::routes::public_finance_calendar::{
    FinanceCalendarEvent, calendar_events_for_range, portfolio_calendar_symbols,
};
use crate::state::AppState;

/// Pre-generation runs at 19:10 in the runtime timezone, before the 19:30+ rating and
/// signal workers, so the first screen never triggers the FMP fan-out itself.
const REFRESH_HOUR: u32 = 19;
const REFRESH_MINUTE: u32 = 10;
/// Synthetic actor used to pre-generate the shared (no-holdings) brief; its
/// portfolio is always empty, so the scope is exactly the coverage universe.
const PREGEN_USER_ID: &str = "weekly-brief-pregen";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WeeklyBriefWindow {
    pub start: String,
    pub end: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WeeklyBriefItem {
    pub id: String,
    pub date: String,
    pub weekday: String,
    pub phase: String,
    pub category: String,
    pub importance: String,
    pub title: String,
    pub subtitle: String,
    pub ticker: Option<String>,
    pub source_name: String,
    pub source_url: Option<String>,
    pub evidence_status: String,
    pub evidence_note: String,
    pub analysis: String,
    pub attention: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WeeklyBriefPayload {
    pub report_date: String,
    #[serde(alias = "generated_at_beijing")]
    pub generated_at_local: String,
    pub timezone: String,
    pub status: String,
    pub summary: String,
    pub previous_week: WeeklyBriefWindow,
    pub next_week: WeeklyBriefWindow,
    pub ai_outlook: WeeklyBriefWindow,
    pub last_week_items: Vec<WeeklyBriefItem>,
    pub next_week_items: Vec<WeeklyBriefItem>,
    pub ai_outlook_items: Vec<WeeklyBriefItem>,
    pub earnings_status: String,
    pub earnings_scope_count: usize,
    pub holdings: Vec<String>,
    pub errors: Vec<String>,
    pub methodology_note: String,
    pub disclaimer: String,
}

pub(crate) async fn handle_get_weekly_brief(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let user = match crate::routes::public::require_public_user(&state, &headers).await {
        Ok(user) => user,
        Err(response) => return response,
    };
    let actor = match ActorIdentity::new("web", &user.user_id, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => {
            return crate::routes::json_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("构造 actor 失败: {error}"),
            );
        }
    };
    // Prefer today's pre-generated snapshot: it already covers the rating
    // universe, so it is exact for every user whose holdings stay inside that
    // scope. Anything else (missing/stale snapshot, extra holdings) falls back
    // to the existing live path so behaviour never degrades.
    let holdings = portfolio_calendar_symbols(&state, &actor).await;
    if let Some(mut snapshot) = read_snapshot(&state).await {
        let today = hone_core::local_now().format("%Y-%m-%d").to_string();
        if snapshot.report_date == today && holdings_within_coverage(&state, &holdings).await {
            snapshot.holdings = holdings;
            return Json(snapshot).into_response();
        }
    }
    Json(build_weekly_brief(&state, &actor).await).into_response()
}

async fn holdings_within_coverage(state: &AppState, holdings: &[String]) -> bool {
    if holdings.is_empty() {
        return true;
    }
    let covered = super::company_ratings::covered_symbols(state)
        .await
        .into_iter()
        .collect::<BTreeSet<_>>();
    holdings.iter().all(|symbol| covered.contains(symbol))
}

/// Compact overview projection of the pre-generated snapshot. `None` (missing
/// snapshot) degrades to a waiting card — the overview must never trigger the
/// live FMP fan-out.
pub(crate) async fn overview_card(
    state: &AppState,
) -> Option<crate::routes::research_overview::OverviewCard> {
    let payload = read_snapshot(state).await?;
    let mut card = crate::routes::research_overview::OverviewCard::waiting(
        "weekly-brief",
        "周度简报",
        "回顾与前瞻",
    );
    card.report_date = Some(payload.report_date.clone());
    card.status = payload.status.clone();
    card.metric = Some(format!("{} 项下周日程", payload.next_week_items.len()));
    card.summary = Some(crate::routes::research_overview::short_summary(
        &payload.summary,
    ));
    Some(card)
}

/// Pre-generate the shared weekly brief at 19:10 in the runtime timezone. On startup it
/// first checks whether today's snapshot already exists (daily_signals'
/// `latest_is_date` pattern) so a restart never burns the FMP quota again.
pub(crate) async fn weekly_brief_worker(state: Arc<AppState>) {
    if latest_is_today(&state) {
        info!("weekly brief snapshot already generated today; skipping startup refresh");
    } else {
        refresh_and_store(&state).await;
    }
    loop {
        let next = crate::routes::research_store::next_local_refresh(
            Utc::now(),
            REFRESH_HOUR,
            REFRESH_MINUTE,
        );
        info!(next_refresh = %next, "weekly brief worker waiting");
        let wait = (next - Utc::now())
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(60));
        tokio::time::sleep(wait).await;
        refresh_and_store(&state).await;
    }
}

async fn refresh_and_store(state: &AppState) {
    let actor = match ActorIdentity::new("web", PREGEN_USER_ID, Option::<String>::None) {
        Ok(actor) => actor,
        Err(error) => {
            warn!(%error, "weekly brief pregen actor unavailable");
            return;
        }
    };
    let payload = build_weekly_brief(state, &actor).await;
    match crate::routes::research_store::write_json_atomic(&snapshot_path(state), &payload).await {
        Ok(()) => info!(status = %payload.status, "weekly brief snapshot refreshed"),
        Err(error) => warn!(%error, "weekly brief snapshot write failed"),
    }
}

fn snapshot_path(state: &AppState) -> PathBuf {
    crate::routes::research_store::data_root(state)
        .join("weekly_brief")
        .join("latest.json")
}

async fn read_snapshot(state: &AppState) -> Option<WeeklyBriefPayload> {
    let bytes = tokio::fs::read(snapshot_path(state)).await.ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn latest_is_today(state: &AppState) -> bool {
    let today = hone_core::local_now().format("%Y-%m-%d").to_string();
    std::fs::read(snapshot_path(state))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<WeeklyBriefPayload>(&bytes).ok())
        .is_some_and(|payload| payload.report_date == today)
}

async fn build_weekly_brief(state: &AppState, actor: &ActorIdentity) -> WeeklyBriefPayload {
    let now = hone_core::local_now();
    let today = now.date_naive();
    let current_week_start = week_start(today);
    let previous_start = current_week_start - Days::new(7);
    let previous_end = current_week_start - Days::new(1);
    let next_start = current_week_start + Days::new(7);
    let next_end = next_start + Days::new(6);
    let ai_outlook_end = today + Days::new(29);

    let holdings = portfolio_calendar_symbols(state, actor).await;
    let mut scope = holdings.iter().cloned().collect::<BTreeSet<_>>();
    scope.extend(super::company_ratings::covered_symbols(state).await);
    let scope = scope.into_iter().collect::<Vec<_>>();
    let calendar = calendar_events_for_range(state, &scope, previous_start, ai_outlook_end).await;

    let mut last_week_items = calendar
        .events
        .iter()
        .filter_map(|event| {
            let date = parse_event_date(event)?;
            (date >= previous_start && date <= previous_end)
                .then(|| calendar_item(event, date, "last_week"))
        })
        .collect::<Vec<_>>();
    let mut next_week_items = calendar
        .events
        .iter()
        .filter_map(|event| {
            let date = parse_event_date(event)?;
            (date >= next_start && date <= next_end)
                .then(|| calendar_item(event, date, "next_week"))
        })
        .collect::<Vec<_>>();
    let mut ai_outlook_items = official_ai_calendar_items(today, ai_outlook_end);
    ai_outlook_items.extend(calendar.events.iter().filter_map(|event| {
        let date = parse_event_date(event)?;
        let ticker = event.ticker.as_deref()?;
        (event.kind == "earnings"
            && date >= today
            && date <= ai_outlook_end
            && is_important_ai_ticker(ticker))
        .then(|| calendar_item(event, date, "ai_outlook"))
    }));

    next_week_items.extend(
        ai_outlook_items
            .iter()
            .filter(|item| {
                parse_item_date(item).is_some_and(|date| date >= next_start && date <= next_end)
            })
            .cloned()
            .map(|mut item| {
                item.phase = "next_week".to_string();
                item
            }),
    );

    let key_snapshot = super::key_event_chain::current_snapshot(state).await;
    for topic in key_snapshot.topics {
        for event in topic.events {
            let date = hone_core::local_time_at(event.published_at).date_naive();
            if date < previous_start
                || date > previous_end
                || event.verification_status != "confirmed"
            {
                continue;
            }
            last_week_items.push(WeeklyBriefItem {
                id: format!("industry:{}", event.id),
                date: date.format("%Y-%m-%d").to_string(),
                weekday: weekday_label(date),
                phase: "last_week".to_string(),
                category: "industry".to_string(),
                importance: if topic.priority <= 5 {
                    "high"
                } else {
                    "medium"
                }
                .to_string(),
                title: event.title,
                subtitle: format!("{} · {}", topic.name, event.source_name),
                ticker: event.tickers.first().cloned(),
                source_name: event.source_name,
                source_url: Some(event.source_url),
                evidence_status: "confirmed".to_string(),
                evidence_note: event.verification_note,
                analysis: event.impact,
                attention: event.next_watch,
            });
        }
    }

    sort_and_dedup(&mut last_week_items);
    sort_and_dedup(&mut next_week_items);
    sort_and_dedup(&mut ai_outlook_items);
    let confirmed = last_week_items
        .iter()
        .filter(|item| item.evidence_status == "confirmed")
        .count();
    let next_high = next_week_items
        .iter()
        .filter(|item| item.importance == "high")
        .count();
    let status = if last_week_items.is_empty() && next_week_items.is_empty() {
        "empty"
    } else if calendar.earnings_status == "ok" {
        "live"
    } else {
        "partial"
    };

    WeeklyBriefPayload {
        report_date: today.format("%Y-%m-%d").to_string(),
        generated_at_local: now.format("%Y-%m-%d %H:%M").to_string(),
        timezone: hone_core::runtime_timezone_name(),
        status: status.to_string(),
        summary: format!(
            "上周收录 {} 项，其中 {} 项为已确认产业变化；下周有 {} 项日程，{} 项列为重点关注。",
            last_week_items.len(),
            confirmed,
            next_week_items.len(),
            next_high
        ),
        previous_week: week_window(previous_start, previous_end, "上周复盘"),
        next_week: week_window(next_start, next_end, "下周关注"),
        ai_outlook: week_window(today, ai_outlook_end, "未来 30 天 AI 前瞻"),
        last_week_items,
        next_week_items,
        ai_outlook_items,
        earnings_status: calendar.earnings_status,
        earnings_scope_count: scope.len(),
        holdings,
        errors: calendar.errors,
        methodology_note: "上周由一手确认的产业变化和已发生的重要日程组成；日程本身不代表数据结果。下周仅列有来源的宏观、财报日程；AI 前瞻优先使用公司 IR 与会议官网，只收录已确认日期，未官宣日期不补造。".to_string(),
        disclaimer: "时间均按运行时时区展示。财报日期可能由公司或交易所调整；投资前应再次核对公司 IR、监管文件和官方发布。".to_string(),
    }
}

fn week_start(date: NaiveDate) -> NaiveDate {
    date - Days::new(date.weekday().num_days_from_monday() as u64)
}

fn week_window(start: NaiveDate, end: NaiveDate, label: &str) -> WeeklyBriefWindow {
    WeeklyBriefWindow {
        start: start.format("%Y-%m-%d").to_string(),
        end: end.format("%Y-%m-%d").to_string(),
        label: label.to_string(),
    }
}

fn parse_event_date(event: &FinanceCalendarEvent) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&event.date, "%Y-%m-%d").ok()
}

fn parse_item_date(item: &WeeklyBriefItem) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&item.date, "%Y-%m-%d").ok()
}

fn official_ai_calendar_items(start: NaiveDate, end: NaiveDate) -> Vec<WeeklyBriefItem> {
    let definitions = [
        (
            "2026-08-23",
            "ai:hot-chips-2026",
            "Hot Chips 2026 开幕",
            "8月23–25日 · Stanford · HBM / HBF / Vera / Rubin / MI400",
            None,
            "Hot Chips 官网",
            "https://hotchips.org/",
            "官方议程已公布。它是本期验证下一代 AI 计算、内存与系统架构能否按路线图推进的高信息密度节点，但技术发布不等于量产或订单兑现。",
            "重点记录 HBM/HBF、Rubin、Vera 与 MI400 的明确参数、封装/互连依赖、量产时间和相对上一版路线图的变化。",
        ),
        (
            "2026-08-26",
            "ai:nvda-fy27-q2",
            "NVIDIA FY27 Q2 财报",
            "NVIDIA IR 已确认 · 2nd Quarter FY27 Financial Results",
            Some("NVDA"),
            "NVIDIA Investor Relations",
            "https://investor.nvidia.com/events-and-presentations/events-and-presentations/event-details/2026/NVIDIA-2nd-Quarter-FY27-Financial-Results/default.aspx",
            "这是验证 AI 算力需求、Blackwell 放量质量与下一阶段产品切换的重要财报节点，不能只看 EPS 是否超预期。",
            "重点核对数据中心收入、毛利率、供给与爬坡、下一季指引、云厂商需求、客户集中度，以及 Blackwell 到 Rubin 的衔接口径。",
        ),
    ];

    definitions
        .into_iter()
        .filter_map(
            |(
                date_text,
                id,
                title,
                subtitle,
                ticker,
                source_name,
                source_url,
                analysis,
                attention,
            )| {
                let date = NaiveDate::parse_from_str(date_text, "%Y-%m-%d").ok()?;
                (date >= start && date <= end).then(|| WeeklyBriefItem {
                    id: id.to_string(),
                    date: date_text.to_string(),
                    weekday: weekday_label(date),
                    phase: "ai_outlook".to_string(),
                    category: if ticker.is_some() {
                        "earnings"
                    } else {
                        "ai_conference"
                    }
                    .to_string(),
                    importance: "high".to_string(),
                    title: title.to_string(),
                    subtitle: subtitle.to_string(),
                    ticker: ticker.map(str::to_string),
                    source_name: source_name.to_string(),
                    source_url: Some(source_url.to_string()),
                    evidence_status: "official_schedule".to_string(),
                    evidence_note:
                        "日期与议程来自主办方或公司官方页面；如官方调整，以最新页面为准。"
                            .to_string(),
                    analysis: analysis.to_string(),
                    attention: attention.to_string(),
                })
            },
        )
        .collect()
}

fn is_important_ai_ticker(ticker: &str) -> bool {
    matches!(
        ticker.to_ascii_uppercase().as_str(),
        "NVDA"
            | "AMD"
            | "AVGO"
            | "MRVL"
            | "MSFT"
            | "AMZN"
            | "META"
            | "GOOG"
            | "GOOGL"
            | "ORCL"
            | "TSM"
            | "MU"
            | "SNOW"
            | "PLTR"
            | "ANET"
            | "DELL"
            | "HPE"
            | "CRM"
    )
}

fn calendar_item(event: &FinanceCalendarEvent, date: NaiveDate, phase: &str) -> WeeklyBriefItem {
    let category = calendar_category(event);
    let future = phase != "last_week";
    let (analysis, attention) = calendar_analysis(category, future, event.ticker.as_deref());
    WeeklyBriefItem {
        id: format!("calendar:{}:{}", event.date, event.title),
        date: event.date.clone(),
        weekday: weekday_label(date),
        phase: phase.to_string(),
        category: category.to_string(),
        importance: calendar_importance(event).to_string(),
        title: event.title.clone(),
        subtitle: event.subtitle.clone().unwrap_or_default(),
        ticker: event.ticker.clone(),
        source_name: event.source.clone(),
        source_url: source_url(&event.source),
        evidence_status: if future {
            "scheduled"
        } else {
            "schedule_passed"
        }
        .to_string(),
        evidence_note: if future {
            "有来源的未来日程；日期仍可能调整，不代表结果已知。"
        } else {
            "该日程已发生，但本条没有附带公布结果；需核对官方原文后再判断。"
        }
        .to_string(),
        analysis,
        attention,
    }
}

fn calendar_category(event: &FinanceCalendarEvent) -> &'static str {
    if event.kind == "earnings" {
        return "earnings";
    }
    let text = format!(
        "{} {}",
        event.title,
        event.subtitle.as_deref().unwrap_or_default()
    )
    .to_uppercase();
    if text.contains("FOMC") || text.contains("联储") || text.contains("利率") {
        "policy"
    } else if text.contains("CPI") || text.contains("PPI") || text.contains("PCE") {
        "inflation"
    } else if text.contains("非农") || text.contains("就业") || text.contains("失业") {
        "labor"
    } else if text.contains("GDP")
        || text.contains("PMI")
        || text.contains("零售")
        || text.contains("工业")
        || text.contains("耐用品")
    {
        "growth"
    } else {
        "macro"
    }
}

fn calendar_importance(event: &FinanceCalendarEvent) -> &'static str {
    let title = event.title.to_uppercase();
    if event.kind == "earnings"
        || title.contains("FOMC")
        || title.contains("CPI")
        || title.contains("PCE")
        || title.contains("非农")
        || title.contains("GDP")
        || title.contains("杰克逊霍尔")
    {
        "high"
    } else {
        "medium"
    }
}

fn calendar_analysis(category: &str, future: bool, ticker: Option<&str>) -> (String, String) {
    let tense = if future { "本次" } else { "该项" };
    match category {
        "earnings" => (
            format!(
                "{}财报是验证收入质量、利润率和指引能否兑现市场预期的节点。",
                ticker.map(|value| format!("{value} ")).unwrap_or_default()
            ),
            "重点核对公司 IR 的实际值、下一期指引、订单/资本开支与管理层口径变化，不只看 EPS 是否超预期。".to_string(),
        ),
        "policy" => (
            format!("{tense}政策事件会改变利率路径和风险资产估值的折现率。"),
            "关注声明措辞、投票分歧和市场降息路径的变化；不要只看是否维持利率。".to_string(),
        ),
        "inflation" => (
            format!("{tense}通胀数据影响实际利率、长久期资产估值与降息预期。"),
            "同时看核心分项、住房/服务价格和前值修订，判断粘性而非只看单月同比。".to_string(),
        ),
        "labor" => (
            format!("{tense}就业数据决定软着陆、需求降温与再通胀风险之间的权衡。"),
            "结合新增就业、失业率、劳动参与率和薪资增速，避免用一个 headline 下结论。".to_string(),
        ),
        "growth" => (
            format!("{tense}增长数据用于验证终端需求和企业盈利周期是否转向。"),
            "关注新订单、价格和库存等分项，并与企业指引和实际消费交叉验证。".to_string(),
        ),
        _ => (
            format!("{tense}宏观日程可能影响市场对增长、通胀或流动性的定价。"),
            "公布后核对官方原文、前值修订和市场预期差，再评估对持仓的传导。".to_string(),
        ),
    }
}

fn weekday_label(date: NaiveDate) -> String {
    match date.weekday().num_days_from_monday() {
        0 => "周一",
        1 => "周二",
        2 => "周三",
        3 => "周四",
        4 => "周五",
        5 => "周六",
        _ => "周日",
    }
    .to_string()
}

fn source_url(source: &str) -> Option<String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        Some(source.to_string())
    } else if source == "fmp.stable.earnings" {
        Some("https://financialmodelingprep.com/developer/docs/stable/earnings-company".to_string())
    } else if source.contains('.') {
        Some(format!("https://{source}"))
    } else {
        None
    }
}

fn sort_and_dedup(items: &mut Vec<WeeklyBriefItem>) {
    items.sort_by(|left, right| {
        left.date
            .cmp(&right.date)
            .then_with(|| {
                importance_rank(&left.importance).cmp(&importance_rank(&right.importance))
            })
            .then_with(|| left.category.cmp(&right.category))
            .then_with(|| left.ticker.cmp(&right.ticker))
            .then_with(|| {
                evidence_rank(&left.evidence_status).cmp(&evidence_rank(&right.evidence_status))
            })
            .then_with(|| left.title.cmp(&right.title))
    });
    items.dedup_by(|left, right| {
        left.date == right.date
            && ((left.category == "earnings"
                && right.category == "earnings"
                && left.ticker == right.ticker)
                || left.title == right.title)
    });
}

fn importance_rank(value: &str) -> u8 {
    if value == "high" { 0 } else { 1 }
}

fn evidence_rank(value: &str) -> u8 {
    match value {
        "official_schedule" => 0,
        "confirmed" => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_windows_use_local_monday_to_sunday() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
        let current = week_start(today);
        assert_eq!(current.to_string(), "2026-08-10");
        assert_eq!((current - Days::new(7)).to_string(), "2026-08-03");
        assert_eq!((current + Days::new(13)).to_string(), "2026-08-23");
    }

    #[test]
    fn passed_calendar_never_claims_an_outcome() {
        let event = FinanceCalendarEvent {
            date: "2026-08-07".to_string(),
            title: "美国非农就业报告".to_string(),
            kind: "macro".to_string(),
            ticker: None,
            subtitle: None,
            source: "bls.gov".to_string(),
        };
        let item = calendar_item(
            &event,
            NaiveDate::from_ymd_opt(2026, 8, 7).unwrap(),
            "last_week",
        );
        assert_eq!(item.evidence_status, "schedule_passed");
        assert!(item.evidence_note.contains("没有附带公布结果"));
    }

    #[test]
    fn earnings_are_high_importance_and_analysis_is_source_bounded() {
        let event = FinanceCalendarEvent {
            date: "2026-08-18".to_string(),
            title: "NVDA 财报".to_string(),
            kind: "earnings".to_string(),
            ticker: Some("NVDA".to_string()),
            subtitle: None,
            source: "fmp.stable.earnings".to_string(),
        };
        let item = calendar_item(
            &event,
            NaiveDate::from_ymd_opt(2026, 8, 18).unwrap(),
            "next_week",
        );
        assert_eq!(item.importance, "high");
        assert_eq!(item.evidence_status, "scheduled");
        assert!(item.analysis.contains("NVDA"));
        assert!(item.attention.contains("公司 IR"));
    }

    #[test]
    fn official_ai_calendar_is_source_bounded_and_horizon_filtered() {
        let start = NaiveDate::from_ymd_opt(2026, 8, 11).unwrap();
        let end = start + Days::new(29);
        let items = official_ai_calendar_items(start, end);
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|item| item.title.contains("Hot Chips")));
        assert!(
            items
                .iter()
                .any(|item| item.ticker.as_deref() == Some("NVDA"))
        );
        assert!(
            items
                .iter()
                .all(|item| item.evidence_status == "official_schedule")
        );
        assert!(official_ai_calendar_items(end + Days::new(1), end + Days::new(30)).is_empty());
    }
}
