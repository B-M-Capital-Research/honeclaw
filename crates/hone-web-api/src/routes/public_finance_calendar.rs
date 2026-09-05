//! Public finance calendar APIs.
//!
//! The public user owns the actor scope. Calendar images are rendered by the
//! browser and uploaded through the existing public upload endpoint. This
//! module validates both desktop and mobile variants, then persists their
//! paths as structured session metadata beside compatibility image markers.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{Datelike, NaiveDate};
use futures::stream::{self, StreamExt};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{debug, warn};

use hone_core::ActorIdentity;

use crate::routes::json_error;
use crate::state::{AppState, PushEvent};

const FINANCE_CALENDAR_SOURCE: &str = "hone.public.finance_calendar";

/// Single-letter FMP suffixes that address an exchange rather than a US share
/// class, so `SHEL.L` must not be rewritten the way `BRK.B` is.
const FMP_SINGLE_LETTER_EXCHANGE_SUFFIXES: [&str; 4] = ["L", "F", "V", "T"];

/// Status FMP uses when a symbol sits outside the account's subscription. The
/// answer is stable for the life of the plan, so re-asking every calendar build
/// only burns upstream quota.
const FMP_PLAN_REJECTED_MARKER: &str = "HTTP 402";

/// How long an out-of-plan symbol stays skipped. Short enough that a plan
/// upgrade takes effect the same day without a restart.
const FMP_UNSUPPORTED_SYMBOL_TTL: Duration = Duration::from_secs(6 * 60 * 60);
const FMP_EARNINGS_CACHE_TTL: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
struct UnsupportedFmpSymbols {
    entries: HashMap<String, Instant>,
}

static FMP_UNSUPPORTED_SYMBOLS: LazyLock<Mutex<UnsupportedFmpSymbols>> =
    LazyLock::new(|| Mutex::new(UnsupportedFmpSymbols::default()));

#[derive(Default)]
struct CachedFmpEarnings {
    entries: HashMap<String, (Instant, Value)>,
}

static FMP_EARNINGS_CACHE: LazyLock<Mutex<CachedFmpEarnings>> =
    LazyLock::new(|| Mutex::new(CachedFmpEarnings::default()));

#[derive(Debug, Deserialize)]
pub(crate) struct FinanceCalendarQuery {
    pub month: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinanceCalendarSendRequest {
    pub path: Option<String>,
    pub mobile_path: Option<String>,
    pub month: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct FinanceCalendarMonth {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct FinanceCalendarEvent {
    pub date: String,
    pub title: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct FinanceCalendarPayload {
    pub today: String,
    pub month: String,
    pub months: Vec<FinanceCalendarMonth>,
    pub holdings: Vec<String>,
    pub events: Vec<FinanceCalendarEvent>,
    pub earnings_status: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MonthSpec {
    year: i32,
    month: u32,
}

impl MonthSpec {
    fn value(&self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }

    fn label(&self) -> String {
        format!("{}年{}月", self.year, self.month)
    }
}

/// GET /api/public/finance-calendar?month=YYYY-MM
pub(crate) async fn handle_get_finance_calendar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<FinanceCalendarQuery>,
) -> Response {
    let (actor, _) = match require_public_actor(&state, &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let month = match resolve_requested_month(
        query.month.as_deref(),
        hone_core::local_now().date_naive(),
    ) {
        Ok(month) => month,
        Err(error) => return json_error(StatusCode::BAD_REQUEST, error),
    };

    let payload = build_finance_calendar_payload(&state, &actor, &month).await;
    Json(payload).into_response()
}

/// POST /api/public/finance-calendar/send
pub(crate) async fn handle_send_finance_calendar(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<FinanceCalendarSendRequest>,
) -> Response {
    let (actor, user_id) = match require_public_actor(&state, &headers).await {
        Ok(value) => value,
        Err(response) => return response,
    };
    let raw_path = match request
        .path
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(path) => path,
        None => return json_error(StatusCode::BAD_REQUEST, "缺少图片路径"),
    };
    if !raw_path.to_ascii_lowercase().ends_with(".png") {
        return json_error(StatusCode::BAD_REQUEST, "财经日历只接受 PNG 图片");
    }

    let upload_root = crate::routes::public::public_upload_dir(&state, &user_id);
    let oss = crate::cloud_oss::OssClient::from_config(&state.core.config.cloud.oss);
    let validated_path = match crate::routes::public::validate_public_upload_path(
        &upload_root,
        oss.as_ref(),
        &user_id,
        raw_path,
    ) {
        Ok(path) => path,
        Err(response) => return response,
    };
    let raw_mobile_path = match request
        .mobile_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(path) => path,
        None => return json_error(StatusCode::BAD_REQUEST, "缺少移动端财经日历图片路径"),
    };
    if !raw_mobile_path.to_ascii_lowercase().ends_with(".png") {
        return json_error(StatusCode::BAD_REQUEST, "移动端财经日历只接受 PNG 图片");
    }
    let validated_mobile_path = match crate::routes::public::validate_public_upload_path(
        &upload_root,
        oss.as_ref(),
        &user_id,
        raw_mobile_path,
    ) {
        Ok(path) => path,
        Err(response) => return response,
    };

    let month = request
        .month
        .as_deref()
        .and_then(|value| parse_month_spec(value).ok())
        .map(|month| month.value())
        .unwrap_or_else(|| hone_core::local_now().format("%Y-%m").to_string());
    let content =
        finance_calendar_assistant_message(&validated_path, Some(&validated_mobile_path), &month);
    let metadata =
        finance_calendar_message_metadata(&validated_path, &validated_mobile_path, &month);
    let session_id = actor.session_id();
    if state
        .core
        .session_storage
        .load_session(&session_id)
        .await
        .ok()
        .flatten()
        .is_none()
        && let Err(error) = state
            .core
            .session_storage
            .create_session_for_actor(&actor)
            .await
    {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("创建会话失败: {error}"),
        );
    }
    match state
        .core
        .session_storage
        .add_message(&session_id, "assistant", &content, Some(metadata))
        .await
    {
        Ok(true) => {}
        Ok(false) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "会话不可用"),
        Err(error) => {
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("写入会话失败: {error}"),
            );
        }
    }

    let _ = state.push_tx.send(PushEvent {
        channel: actor.channel,
        user_id: actor.user_id,
        channel_scope: actor.channel_scope,
        event: "push_message".to_string(),
        data: json!({
            "text": content,
            "source": FINANCE_CALENDAR_SOURCE,
            "month": month,
        }),
    });

    Json(json!({ "ok": true, "message": content })).into_response()
}

async fn build_finance_calendar_payload(
    state: &AppState,
    actor: &ActorIdentity,
    month: &MonthSpec,
) -> FinanceCalendarPayload {
    let mut events = macro_events_for_month(month);
    let holdings = portfolio_calendar_symbols(state, actor).await;
    let mut errors = Vec::new();
    let mut earnings_status = "ok".to_string();

    if holdings.is_empty() {
        earnings_status = "empty_portfolio".to_string();
    } else {
        match fetch_earnings_for_symbols(state, &holdings, month).await {
            EarningsFetchOutcome::Ok(items) => events.extend(items),
            EarningsFetchOutcome::MissingKey => {
                earnings_status = "missing_key".to_string();
                errors.push("未配置 FMP API Key，已仅展示内置宏观事件".to_string());
            }
            EarningsFetchOutcome::Partial {
                events: items,
                errors: errs,
            } => {
                earnings_status = "partial".to_string();
                events.extend(items);
                errors.extend(errs);
            }
            EarningsFetchOutcome::Failed(errs) => {
                earnings_status = "failed".to_string();
                errors.extend(errs);
            }
        }
    }

    events.sort_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then_with(|| event_kind_sort_key(&a.kind).cmp(&event_kind_sort_key(&b.kind)))
            .then_with(|| a.title.cmp(&b.title))
    });

    FinanceCalendarPayload {
        today: hone_core::local_now()
            .date_naive()
            .format("%Y-%m-%d")
            .to_string(),
        month: month.value(),
        months: months_for_year(month.year),
        holdings,
        events,
        earnings_status,
        errors,
    }
}

fn event_kind_sort_key(kind: &str) -> u8 {
    match kind {
        "macro" => 0,
        "earnings" => 1,
        _ => 2,
    }
}

async fn require_public_actor(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(ActorIdentity, String), Response> {
    let user = crate::routes::public::require_public_user(state, headers).await?;
    let user_id = user.user_id.clone();
    let actor = ActorIdentity::new("web", &user_id, Option::<String>::None).map_err(|e| {
        json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("构造 actor 失败: {e}"),
        )
    })?;
    Ok((actor, user_id))
}

fn resolve_requested_month(raw: Option<&str>, today: NaiveDate) -> Result<MonthSpec, String> {
    match raw.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => parse_month_spec(value),
        None => Ok(default_month_for_date(today)),
    }
}

fn parse_month_spec(value: &str) -> Result<MonthSpec, String> {
    if value.len() != 7 {
        return Err("month 格式应为 YYYY-MM".to_string());
    }
    let Some((year, month)) = value.split_once('-') else {
        return Err("month 格式应为 YYYY-MM".to_string());
    };
    if year.len() != 4 || month.len() != 2 {
        return Err("month 格式应为 YYYY-MM".to_string());
    }
    let year = year
        .parse::<i32>()
        .map_err(|_| "month 年份无效".to_string())?;
    let month = month
        .parse::<u32>()
        .map_err(|_| "month 月份无效".to_string())?;
    if !(1..=12).contains(&month) {
        return Err("month 月份必须在 01-12".to_string());
    }
    Ok(MonthSpec { year, month })
}

fn default_month_for_date(today: NaiveDate) -> MonthSpec {
    MonthSpec {
        year: today.year(),
        month: today.month(),
    }
}

fn months_for_year(year: i32) -> Vec<FinanceCalendarMonth> {
    (1..=12)
        .map(|month| {
            let spec = MonthSpec { year, month };
            FinanceCalendarMonth {
                value: spec.value(),
                label: spec.label(),
            }
        })
        .collect()
}

fn macro_events_for_month(month: &MonthSpec) -> Vec<FinanceCalendarEvent> {
    macro_seed_events()
        .into_iter()
        .filter(|event| event.date.starts_with(&month.value()))
        .collect()
}

fn macro_seed_events() -> Vec<FinanceCalendarEvent> {
    [
        (
            "2026-07-01",
            "ISM 制造业 PMI",
            "运行时时区 22:00 · 6月",
            "ismworld.org",
        ),
        (
            "2026-07-02",
            "美国非农就业报告",
            "运行时时区 20:30 · 6月",
            "bls.gov",
        ),
        (
            "2026-07-06",
            "ISM 服务业 PMI",
            "运行时时区 22:00 · 6月",
            "ismworld.org",
        ),
        (
            "2026-07-07",
            "美国贸易帐",
            "运行时时区 20:30 · 5月",
            "bea.gov",
        ),
        (
            "2026-07-09",
            "FOMC 会议纪要",
            "运行时时区 02:00 · 6月会议",
            "federalreserve.gov",
        ),
        (
            "2026-07-14",
            "美国 CPI",
            "运行时时区 20:30 · 6月",
            "bls.gov",
        ),
        (
            "2026-07-15",
            "美国 PPI",
            "运行时时区 20:30 · 6月",
            "bls.gov",
        ),
        (
            "2026-07-16",
            "美联储褐皮书",
            "运行时时区 02:00",
            "federalreserve.gov",
        ),
        (
            "2026-07-16",
            "美国零售销售",
            "运行时时区 20:30 · 6月",
            "census.gov",
        ),
        (
            "2026-07-17",
            "美国新屋开工",
            "运行时时区 20:30 · 6月",
            "census.gov",
        ),
        (
            "2026-07-17",
            "美国工业产出",
            "运行时时区 21:15 · 6月",
            "federalreserve.gov",
        ),
        (
            "2026-07-24",
            "美国新屋销售",
            "运行时时区 22:00 · 6月",
            "census.gov",
        ),
        (
            "2026-07-27",
            "美国耐用品订单",
            "运行时时区 20:30 · 6月",
            "census.gov",
        ),
        (
            "2026-07-30",
            "FOMC 利率决议与记者会",
            "运行时时区 02:00 / 02:30",
            "federalreserve.gov",
        ),
        (
            "2026-07-30",
            "美国二季度 GDP 初值",
            "运行时时区 20:30",
            "bea.gov",
        ),
        (
            "2026-07-30",
            "美国 PCE 物价指数",
            "运行时时区 20:30 · 6月",
            "bea.gov",
        ),
        (
            "2026-07-31",
            "美国就业成本指数",
            "运行时时区 20:30 · 二季度",
            "bls.gov",
        ),
        (
            "2026-08-03",
            "ISM 制造业 PMI",
            "运行时时区 22:00 · 7月",
            "ismworld.org",
        ),
        (
            "2026-08-05",
            "ISM 服务业 PMI",
            "运行时时区 22:00 · 7月",
            "ismworld.org",
        ),
        (
            "2026-08-05",
            "美国贸易帐",
            "运行时时区 20:30 · 6月",
            "bea.gov",
        ),
        (
            "2026-08-07",
            "美国非农就业报告",
            "运行时时区 20:30 · 7月",
            "bls.gov",
        ),
        (
            "2026-08-12",
            "美国 CPI",
            "运行时时区 20:30 · 7月",
            "bls.gov",
        ),
        (
            "2026-08-13",
            "美国 PPI",
            "运行时时区 20:30 · 7月",
            "bls.gov",
        ),
        (
            "2026-08-14",
            "美国零售销售",
            "运行时时区 20:30 · 7月",
            "census.gov",
        ),
        (
            "2026-08-14",
            "美国工业产出",
            "运行时时区 21:15 · 7月",
            "federalreserve.gov",
        ),
        (
            "2026-08-18",
            "美国新屋开工",
            "运行时时区 20:30 · 7月",
            "census.gov",
        ),
        (
            "2026-08-20",
            "FOMC 会议纪要",
            "运行时时区 02:00 · 7月会议",
            "federalreserve.gov",
        ),
        (
            "2026-08-21",
            "杰克逊霍尔央行年会·美联储主席讲话",
            "运行时时区 22:00 前后 · 年会 8/20-8/22",
            "kansascityfed.org",
        ),
        (
            "2026-08-25",
            "美国新屋销售",
            "运行时时区 22:00 · 7月",
            "census.gov",
        ),
        (
            "2026-08-26",
            "美国耐用品订单",
            "运行时时区 20:30 · 7月",
            "census.gov",
        ),
        (
            "2026-08-27",
            "美国二季度 GDP 修正值",
            "运行时时区 20:30",
            "bea.gov",
        ),
        (
            "2026-08-28",
            "美国 PCE 物价指数",
            "运行时时区 20:30 · 7月",
            "bea.gov",
        ),
        (
            "2026-08-28",
            "密歇根大学消费者信心终值",
            "运行时时区 22:00 · 8月",
            "umich.edu",
        ),
    ]
    .into_iter()
    .map(|(date, title, subtitle, source)| FinanceCalendarEvent {
        date: date.to_string(),
        title: title.to_string(),
        kind: "macro".to_string(),
        ticker: None,
        subtitle: Some(subtitle.to_string()),
        source: source.to_string(),
    })
    .collect()
}

pub(crate) async fn portfolio_calendar_symbols(
    state: &AppState,
    actor: &ActorIdentity,
) -> Vec<String> {
    let portfolio_storage =
        hone_memory::PortfolioStorage::new(&state.core.config.storage.portfolio_dir);
    let Ok(Some(portfolio)) = portfolio_storage.load(actor).await else {
        return Vec::new();
    };
    calendar_symbols_from_holdings(&portfolio.holdings)
}

fn calendar_symbols_from_holdings(holdings: &[hone_memory::portfolio::Holding]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    for holding in holdings {
        let raw = holding
            .underlying
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| holding.symbol.trim());
        let symbol = normalize_calendar_symbol(raw);
        if !symbol.is_empty() {
            seen.insert(symbol);
        }
    }
    seen.into_iter().collect()
}

fn normalize_calendar_symbol(raw: &str) -> String {
    let cleaned = raw
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        .collect::<String>()
        .to_ascii_uppercase();

    // FMP addresses a US share class with a dash (`BRK-B`) and a non-US listing
    // with an exchange suffix (`0700.HK`). Portfolios store the dotted share
    // class, which FMP rejects with HTTP 402, so translate only that form and
    // leave every exchange suffix alone.
    let Some((base, suffix)) = cleaned.rsplit_once('.') else {
        return cleaned;
    };
    if base.is_empty()
        || base.contains('.')
        || suffix.len() != 1
        || !suffix.chars().all(|ch| ch.is_ascii_alphabetic())
        || FMP_SINGLE_LETTER_EXCHANGE_SUFFIXES.contains(&suffix)
    {
        return cleaned;
    }
    format!("{base}-{suffix}")
}

enum EarningsFetchOutcome {
    Ok(Vec<FinanceCalendarEvent>),
    MissingKey,
    Partial {
        events: Vec<FinanceCalendarEvent>,
        errors: Vec<String>,
    },
    Failed(Vec<String>),
}

async fn fetch_earnings_for_symbols(
    state: &AppState,
    symbols: &[String],
    month: &MonthSpec,
) -> EarningsFetchOutcome {
    let start = NaiveDate::from_ymd_opt(month.year, month.month, 1)
        .expect("validated calendar month must have a first day");
    let next_month = if month.month == 12 {
        NaiveDate::from_ymd_opt(month.year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(month.year, month.month + 1, 1)
    }
    .expect("validated calendar month must have a next month");
    fetch_earnings_for_symbols_in_range(state, symbols, start, next_month - chrono::Days::new(1))
        .await
}

pub(crate) struct FinanceCalendarRangeResult {
    pub events: Vec<FinanceCalendarEvent>,
    pub earnings_status: String,
    pub errors: Vec<String>,
}

pub(crate) async fn calendar_events_for_range(
    state: &AppState,
    symbols: &[String],
    start: NaiveDate,
    end: NaiveDate,
) -> FinanceCalendarRangeResult {
    let mut events = macro_seed_events()
        .into_iter()
        .filter(|event| {
            NaiveDate::parse_from_str(&event.date, "%Y-%m-%d")
                .map(|date| date >= start && date <= end)
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    let earnings_status = if symbols.is_empty() {
        "empty_scope".to_string()
    } else {
        match fetch_earnings_for_symbols_in_range(state, symbols, start, end).await {
            EarningsFetchOutcome::Ok(items) => {
                events.extend(items);
                "ok".to_string()
            }
            EarningsFetchOutcome::MissingKey => {
                errors.push("未配置 FMP API Key，重点公司财报日期未进入本期简报".to_string());
                "missing_key".to_string()
            }
            EarningsFetchOutcome::Partial {
                events: items,
                errors: errs,
            } => {
                events.extend(items);
                errors.extend(errs);
                "partial".to_string()
            }
            EarningsFetchOutcome::Failed(errs) => {
                errors.extend(errs);
                "failed".to_string()
            }
        }
    };
    events.sort_by(|left, right| {
        left.date
            .cmp(&right.date)
            .then_with(|| event_kind_sort_key(&left.kind).cmp(&event_kind_sort_key(&right.kind)))
            .then_with(|| left.title.cmp(&right.title))
    });
    events.dedup_by(|left, right| {
        left.date == right.date && left.kind == right.kind && left.title == right.title
    });
    FinanceCalendarRangeResult {
        events,
        earnings_status,
        errors,
    }
}

async fn fetch_earnings_for_symbols_in_range(
    state: &AppState,
    symbols: &[String],
    start: NaiveDate,
    end: NaiveDate,
) -> EarningsFetchOutcome {
    let pool = state.core.config.fmp.effective_key_pool();
    let keys = pool.keys();
    if keys.is_empty() {
        return EarningsFetchOutcome::MissingKey;
    }

    let mut events = Vec::new();
    let mut errors = Vec::new();
    let mut unsupported = Vec::new();
    let requests = symbols
        .iter()
        .filter_map(|symbol| {
            if fmp_symbol_is_known_unsupported(symbol) {
                unsupported.push(symbol.clone());
                None
            } else {
                Some(symbol.clone())
            }
        })
        .collect::<Vec<_>>();
    let results = stream::iter(requests.into_iter().map(|symbol| async move {
        let result = fetch_symbol_earnings(state, keys, &symbol).await;
        (symbol, result)
    }))
    .buffer_unordered(8)
    .collect::<Vec<_>>()
    .await;
    for (symbol, result) in results {
        match result {
            Ok(value) => events.extend(earnings_events_from_value_in_range(
                &symbol, &value, start, end,
            )),
            Err(error) if fmp_error_is_plan_rejection(&error) => {
                // Not a failure to retry: the plan will answer the same way
                // until it is upgraded, so remember it and stop asking.
                remember_unsupported_fmp_symbol(&symbol);
                debug!(%symbol, "finance calendar FMP symbol outside subscription: {error}");
                unsupported.push(symbol);
            }
            Err(error) => {
                warn!(%symbol, "finance calendar FMP earnings fetch failed: {error}");
                errors.push(format!("{symbol}: {error}"));
            }
        }
    }

    if !unsupported.is_empty() {
        errors.push(format!(
            "以下标的不在当前 FMP 订阅覆盖范围内，已跳过财报日期：{}",
            unsupported.join("、")
        ));
    }

    if errors.is_empty() {
        EarningsFetchOutcome::Ok(events)
    } else if events.is_empty() {
        EarningsFetchOutcome::Failed(errors)
    } else {
        EarningsFetchOutcome::Partial { events, errors }
    }
}

fn fmp_error_is_plan_rejection(error: &str) -> bool {
    error.contains(FMP_PLAN_REJECTED_MARKER)
}

fn fmp_symbol_is_known_unsupported(symbol: &str) -> bool {
    let Ok(mut cache) = FMP_UNSUPPORTED_SYMBOLS.lock() else {
        return false;
    };
    let now = Instant::now();
    cache
        .entries
        .retain(|_, seen_at| now.duration_since(*seen_at) < FMP_UNSUPPORTED_SYMBOL_TTL);
    cache.entries.contains_key(symbol)
}

fn remember_unsupported_fmp_symbol(symbol: &str) {
    if let Ok(mut cache) = FMP_UNSUPPORTED_SYMBOLS.lock() {
        cache.entries.insert(symbol.to_string(), Instant::now());
    }
}

/// FMP error bodies arrive as free-form text; keep them on one log line.
fn collapse_fmp_body_whitespace(body: &str) -> String {
    body.split_whitespace().collect::<Vec<_>>().join(" ")
}

async fn fetch_symbol_earnings(
    state: &AppState,
    keys: &[String],
    symbol: &str,
) -> Result<Value, String> {
    let stable_base = stable_fmp_base_url(&state.core.config.fmp.base_url);
    let cache_key = format!("{stable_base}|{symbol}");
    if let Some(value) = cached_fmp_earnings(&cache_key) {
        return Ok(value);
    }
    let encoded_symbol = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
    let url_base = format!("{stable_base}/stable/earnings?symbol={encoded_symbol}");
    let mut last_error = String::new();
    for key in keys {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("{url_base}&apikey={encoded_key}");
        match fetch_fmp_json_once(&state.http_client, &url, state.core.config.fmp.timeout).await {
            Ok(value) => {
                remember_fmp_earnings(cache_key.clone(), value.clone());
                return Ok(value);
            }
            Err(error) => last_error = error,
        }
    }
    Err(if last_error.is_empty() {
        "FMP 请求失败".to_string()
    } else {
        last_error
    })
}

fn cached_fmp_earnings(key: &str) -> Option<Value> {
    let Ok(mut cache) = FMP_EARNINGS_CACHE.lock() else {
        return None;
    };
    let now = Instant::now();
    cache
        .entries
        .retain(|_, (seen_at, _)| now.duration_since(*seen_at) < FMP_EARNINGS_CACHE_TTL);
    cache.entries.get(key).map(|(_, value)| value.clone())
}

fn remember_fmp_earnings(key: String, value: Value) {
    if let Ok(mut cache) = FMP_EARNINGS_CACHE.lock() {
        cache.entries.insert(key, (Instant::now(), value));
    }
}

pub(crate) async fn fetch_fmp_json_once(
    http: &reqwest::Client,
    url: &str,
    timeout_secs: u64,
) -> Result<Value, String> {
    let response = http
        .get(url)
        .timeout(Duration::from_secs(timeout_secs))
        .send()
        .await
        .map_err(|error| sanitize_fmp_error(&format!("FMP 请求失败: {error}")))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| sanitize_fmp_error(&format!("FMP 响应读取失败: {error}")))?;
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return Err(format!("FMP Key 无效（HTTP {status}）"));
    }
    // FMP answers plan and quota rejections with plain text, not JSON. Parsing
    // before checking the status turned every one of them into a misleading
    // "JSON 解析失败: expected value at line 1 column 1".
    if !status.is_success() {
        return Err(sanitize_fmp_error(&format!(
            "FMP 请求被拒绝（HTTP {}）: {}",
            status.as_u16(),
            collapse_fmp_body_whitespace(&body)
        )));
    }
    let value: Value = serde_json::from_str(&body)
        .map_err(|error| sanitize_fmp_error(&format!("FMP JSON 解析失败: {error}")))?;
    if let Some(message) = value.get("Error Message").and_then(|value| value.as_str()) {
        return Err(sanitize_fmp_error(message));
    }
    Ok(value)
}

/// FMP `stable/` 端点的 base。配置里的 `fmp.base_url` 是 `https://financialmodelingprep.com/api`
/// （也可能带 `/v3`），而 stable 端点挂在 host 根下的 `/stable/...`——两个后缀都要剥掉。
///
/// `pub(crate)`：company_facts 的 worker 也要拼 stable 端点。这段逻辑已经在仓库里被
/// 抄过三遍，第四遍抄错一次就是一整段功能静默失效（详见 company_facts.rs 的 URL 测试）。
pub(crate) fn stable_fmp_base_url(base_url: &str) -> String {
    let mut base = base_url.trim_end_matches('/').to_string();
    for suffix in ["/api/v3", "/api"] {
        if let Some(stripped) = base.strip_suffix(suffix) {
            base = stripped.to_string();
            break;
        }
    }
    base.trim_end_matches('/').to_string()
}

#[cfg(test)]
fn earnings_events_from_value(
    requested_symbol: &str,
    value: &Value,
    month: &MonthSpec,
) -> Vec<FinanceCalendarEvent> {
    let start = NaiveDate::from_ymd_opt(month.year, month.month, 1)
        .expect("validated calendar month must have a first day");
    let next_month = if month.month == 12 {
        NaiveDate::from_ymd_opt(month.year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(month.year, month.month + 1, 1)
    }
    .expect("validated calendar month must have a next month");
    earnings_events_from_value_in_range(
        requested_symbol,
        value,
        start,
        next_month - chrono::Days::new(1),
    )
}

fn earnings_events_from_value_in_range(
    requested_symbol: &str,
    value: &Value,
    start: NaiveDate,
    end: NaiveDate,
) -> Vec<FinanceCalendarEvent> {
    let items = match value.as_array() {
        Some(items) => items,
        None => return Vec::new(),
    };
    let mut dedup = BTreeMap::<(String, String), FinanceCalendarEvent>::new();
    for item in items {
        let Some(date) = earnings_date_from_item(item) else {
            continue;
        };
        if date < start || date > end {
            continue;
        }
        let date_text = date.format("%Y-%m-%d").to_string();
        let symbol = item
            .get("symbol")
            .and_then(|value| value.as_str())
            .map(normalize_calendar_symbol)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| requested_symbol.to_string());
        let subtitle = earnings_subtitle_from_item(item);
        dedup
            .entry((date_text.clone(), symbol.clone()))
            .or_insert(FinanceCalendarEvent {
                date: date_text,
                title: format!("{symbol} 财报"),
                kind: "earnings".to_string(),
                ticker: Some(symbol),
                subtitle,
                source: "fmp.stable.earnings".to_string(),
            });
    }
    dedup.into_values().collect()
}

fn earnings_date_from_item(item: &Value) -> Option<NaiveDate> {
    for key in ["date", "reportedDate", "reportDate", "epsDate"] {
        let Some(value) = item.get(key).and_then(|value| value.as_str()) else {
            continue;
        };
        if let Ok(date) = NaiveDate::parse_from_str(value.get(0..10).unwrap_or(value), "%Y-%m-%d") {
            return Some(date);
        }
    }
    None
}

fn earnings_subtitle_from_item(item: &Value) -> Option<String> {
    let time = item
        .get("time")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let fiscal = item
        .get("fiscalDateEnding")
        .or_else(|| item.get("period"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match (time, fiscal) {
        (Some(time), Some(fiscal)) => Some(format!("{time} · {fiscal}")),
        (Some(time), None) => Some(time.to_string()),
        (None, Some(fiscal)) => Some(fiscal.to_string()),
        (None, None) => None,
    }
}

fn finance_calendar_image_marker(path: &str) -> String {
    if path.trim().starts_with("oss://") {
        path.trim().to_string()
    } else {
        format!("file://{}", path.trim().trim_start_matches("file://"))
    }
}

fn finance_calendar_message_metadata(
    desktop_path: &str,
    mobile_path: &str,
    month: &str,
) -> HashMap<String, Value> {
    HashMap::from([
        (
            "source".to_string(),
            Value::String(FINANCE_CALENDAR_SOURCE.to_string()),
        ),
        (
            "finance_calendar".to_string(),
            json!({
                "month": month,
                "desktop_path": desktop_path,
                "mobile_path": mobile_path,
            }),
        ),
    ])
}

fn finance_calendar_assistant_message(
    desktop_path: &str,
    mobile_path: Option<&str>,
    month: &str,
) -> String {
    let desktop_marker = finance_calendar_image_marker(desktop_path);
    match mobile_path {
        Some(path) => format!(
            "这是你的 {month} 财经日历：\n\n{desktop_marker}\n\n{}",
            finance_calendar_image_marker(path)
        ),
        None => format!("这是你的 {month} 财经日历：\n\n{desktop_marker}"),
    }
}

fn sanitize_fmp_error(message: &str) -> String {
    let mut out = message.to_string();
    for key in ["apikey", "api_key", "apiKey"] {
        let needle = format!("{key}=");
        let mut search_from = 0;
        while let Some(relative_index) = out[search_from..].find(&needle) {
            let index = search_from + relative_index;
            let value_start = index + key.len() + 1;
            let value_end = out[value_start..]
                .char_indices()
                .find_map(|(idx, ch)| (ch == '&' || ch.is_whitespace()).then_some(idx))
                .map(|idx| value_start + idx)
                .unwrap_or(out.len());
            out.replace_range(value_start..value_end, "<redacted>");
            search_from = value_start + "<redacted>".len();
        }
    }
    if out.chars().count() > 240 {
        out.chars().take(240).collect::<String>() + "..."
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hone_memory::portfolio::Holding;

    #[test]
    fn finance_calendar_month_parser_accepts_yyyy_mm() {
        assert_eq!(
            parse_month_spec("2026-07").expect("month"),
            MonthSpec {
                year: 2026,
                month: 7
            }
        );
        assert!(parse_month_spec("2026-7").is_err());
        assert!(parse_month_spec("2026-13").is_err());
        assert!(parse_month_spec("bad").is_err());
    }

    #[test]
    fn sanitize_fmp_error_redacts_each_key_without_reprocessing_replacement() {
        let message = "request failed: https://example.test/stable/earnings?symbol=SPY&apikey=secret-one api_key=secret-two&apiKey=secret-three";

        let sanitized = sanitize_fmp_error(message);

        assert_eq!(
            sanitized,
            "request failed: https://example.test/stable/earnings?symbol=SPY&apikey=<redacted> api_key=<redacted>&apiKey=<redacted>"
        );
        assert!(!sanitized.contains("secret-"));
    }

    #[test]
    fn finance_calendar_default_month_is_always_current_month() {
        assert_eq!(
            default_month_for_date(NaiveDate::from_ymd_opt(2026, 6, 23).unwrap()),
            MonthSpec {
                year: 2026,
                month: 6
            }
        );
        assert_eq!(
            default_month_for_date(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
            MonthSpec {
                year: 2026,
                month: 6
            }
        );
        assert_eq!(
            default_month_for_date(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
            MonthSpec {
                year: 2026,
                month: 12
            }
        );
    }

    #[test]
    fn finance_calendar_macro_seed_filters_july_events() {
        let july = MonthSpec {
            year: 2026,
            month: 7,
        };
        let events = macro_events_for_month(&july);
        assert_eq!(events.len(), 17);
        assert_eq!(events[0].date, "2026-07-01");
        assert!(events.iter().any(|event| event.title.contains("非农")));
        assert!(events.iter().any(|event| event.title.contains("CPI")));
        assert!(events.iter().any(|event| event.title.contains("利率决议")));
        assert!(events.iter().all(|event| event.subtitle.is_some()));

        let august = MonthSpec {
            year: 2026,
            month: 8,
        };
        let august_events = macro_events_for_month(&august);
        assert_eq!(august_events.len(), 16);
        assert_eq!(august_events[0].date, "2026-08-03");
        assert!(
            august_events
                .iter()
                .any(|event| event.title.contains("非农"))
        );
        assert!(
            august_events
                .iter()
                .any(|event| event.title.contains("CPI"))
        );
        assert!(
            august_events
                .iter()
                .any(|event| event.title.contains("杰克逊霍尔"))
        );
        assert!(
            august_events
                .iter()
                .all(|event| event.date.starts_with("2026-08"))
        );
        assert!(august_events.iter().all(|event| event.subtitle.is_some()));

        let september = MonthSpec {
            year: 2026,
            month: 9,
        };
        assert!(macro_events_for_month(&september).is_empty());
    }

    #[test]
    fn finance_calendar_symbols_prefer_option_underlying_and_dedupe() {
        let holdings = vec![
            Holding {
                symbol: "AAPL".to_string(),
                asset_type: "stock".to_string(),
                shares: 1.0,
                avg_cost: 100.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                weight: None,
                name: None,
                tracking_only: None,
            },
            Holding {
                symbol: "AAPL250117C00100000".to_string(),
                asset_type: "option".to_string(),
                shares: 1.0,
                avg_cost: 1.0,
                underlying: Some("aapl".to_string()),
                option_type: Some("call".to_string()),
                strike_price: Some(100.0),
                expiration_date: Some("2025-01-17".to_string()),
                contract_multiplier: Some(100.0),
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                weight: None,
                name: None,
                tracking_only: Some(true),
            },
            Holding {
                symbol: "BRK.B".to_string(),
                asset_type: "stock".to_string(),
                shares: 1.0,
                avg_cost: 1.0,
                underlying: None,
                option_type: None,
                strike_price: None,
                expiration_date: None,
                contract_multiplier: None,
                holding_horizon: None,
                strategy_notes: None,
                notes: None,
                weight: None,
                name: None,
                tracking_only: Some(true),
            },
        ];
        assert_eq!(
            calendar_symbols_from_holdings(&holdings),
            vec!["AAPL".to_string(), "BRK-B".to_string()]
        );
    }

    #[test]
    fn finance_calendar_symbol_rewrites_share_class_but_keeps_exchange_suffix() {
        // FMP serves BRK-B and rejects BRK.B with HTTP 402.
        assert_eq!(normalize_calendar_symbol("brk.b"), "BRK-B");
        assert_eq!(normalize_calendar_symbol("BF.A"), "BF-A");

        // Exchange suffixes must survive untouched, including the single-letter
        // ones that would otherwise look like a share class.
        assert_eq!(normalize_calendar_symbol("0700.HK"), "0700.HK");
        assert_eq!(normalize_calendar_symbol("688167.SH"), "688167.SH");
        assert_eq!(normalize_calendar_symbol("SHEL.L"), "SHEL.L");
        assert_eq!(normalize_calendar_symbol("AAPL"), "AAPL");
        assert_eq!(normalize_calendar_symbol("BRK-B"), "BRK-B");

        // Degenerate input must not grow a dash out of nowhere.
        assert_eq!(normalize_calendar_symbol(".B"), ".B");
        assert_eq!(normalize_calendar_symbol("A.B.C"), "A.B.C");
        assert_eq!(normalize_calendar_symbol("BRK.12"), "BRK.12");
    }

    #[test]
    fn fmp_plan_rejection_is_reported_with_its_status_not_as_a_parse_error() {
        // Regression: the body of an FMP 402 is plain text, and parsing it
        // before checking the status reported "JSON 解析失败" for what is
        // really a subscription limit.
        let rejection = sanitize_fmp_error(&format!(
            "FMP 请求被拒绝（HTTP {}）: {}",
            402,
            collapse_fmp_body_whitespace(
                "Premium Query Parameter: 'Special Endpoint :\n  This value set for 'symbol' is not available"
            )
        ));

        assert!(rejection.contains("HTTP 402"));
        assert!(!rejection.contains("JSON 解析失败"));
        assert!(!rejection.contains('\n'));
        assert!(fmp_error_is_plan_rejection(&rejection));
        assert!(!fmp_error_is_plan_rejection(
            "FMP JSON 解析失败: expected value at line 1 column 1"
        ));
        assert!(!fmp_error_is_plan_rejection("FMP Key 无效（HTTP 401）"));
    }

    #[test]
    fn unsupported_fmp_symbols_are_remembered_and_expire() {
        let symbol = "0700.HK-test-remembered";
        assert!(!fmp_symbol_is_known_unsupported(symbol));

        remember_unsupported_fmp_symbol(symbol);
        assert!(fmp_symbol_is_known_unsupported(symbol));

        // An entry older than the TTL must not keep a symbol suppressed after a
        // plan upgrade.
        {
            let mut cache = FMP_UNSUPPORTED_SYMBOLS.lock().expect("cache");
            let expired = Instant::now()
                .checked_sub(FMP_UNSUPPORTED_SYMBOL_TTL + Duration::from_secs(1))
                .expect("expired instant");
            cache.entries.insert(symbol.to_string(), expired);
        }
        assert!(!fmp_symbol_is_known_unsupported(symbol));
    }

    #[test]
    fn finance_calendar_parses_fmp_earnings_items_for_month() {
        let raw = json!([
            {"symbol":"AAPL","date":"2026-07-30","time":"amc","fiscalDateEnding":"2026-06-30"},
            {"symbol":"AAPL","date":"2026-08-01","time":"bmo"},
            {"symbol":"MSFT","reportedDate":"2026-07-24T00:00:00.000Z"}
        ]);
        let july = MonthSpec {
            year: 2026,
            month: 7,
        };
        let events = earnings_events_from_value("AAPL", &raw, &july);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].date, "2026-07-24");
        assert_eq!(events[1].date, "2026-07-30");
        assert_eq!(events[1].subtitle.as_deref(), Some("amc · 2026-06-30"));
    }

    #[test]
    fn finance_calendar_assistant_message_uses_image_marker() {
        let local = finance_calendar_assistant_message(
            "/tmp/calendar.png",
            Some("/tmp/calendar-mobile.png"),
            "2026-07",
        );
        assert!(local.contains("file:///tmp/calendar.png"));
        assert!(local.contains("file:///tmp/calendar-mobile.png"));
        let oss = finance_calendar_assistant_message(
            "oss://bucket/users/a/calendar.png",
            None,
            "2026-07",
        );
        assert!(oss.contains("oss://bucket/users/a/calendar.png"));
    }

    #[test]
    fn finance_calendar_metadata_persists_both_image_variants() {
        let metadata = finance_calendar_message_metadata(
            "/tmp/calendar.png",
            "/tmp/calendar-mobile-v4.png",
            "2026-07",
        );
        let calendar = metadata.get("finance_calendar").expect("calendar metadata");

        assert_eq!(calendar["month"], "2026-07");
        assert_eq!(calendar["desktop_path"], "/tmp/calendar.png");
        assert_eq!(calendar["mobile_path"], "/tmp/calendar-mobile-v4.png");
        assert_eq!(metadata["source"], FINANCE_CALENDAR_SOURCE);
    }

    #[test]
    fn finance_calendar_stable_fmp_base_strips_api_suffix() {
        assert_eq!(
            stable_fmp_base_url("https://financialmodelingprep.com/api"),
            "https://financialmodelingprep.com"
        );
        assert_eq!(
            stable_fmp_base_url("https://financialmodelingprep.com/api/v3"),
            "https://financialmodelingprep.com"
        );
        assert_eq!(
            stable_fmp_base_url("https://example.com/fmp"),
            "https://example.com/fmp"
        );
    }
}
