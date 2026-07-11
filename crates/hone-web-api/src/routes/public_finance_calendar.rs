//! Public finance calendar APIs.
//!
//! The public user owns the actor scope. Calendar images are rendered by the
//! browser, uploaded through the existing public upload endpoint, then this
//! module persists a short assistant message containing the uploaded image
//! marker into the user's web chat session.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{Datelike, NaiveDate};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::warn;

use hone_core::ActorIdentity;

use crate::routes::json_error;
use crate::state::{AppState, PushEvent};

const FINANCE_CALENDAR_SOURCE: &str = "hone.public.finance_calendar";

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
    let (actor, _) = match require_public_actor(&state, &headers) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let month = match resolve_requested_month(
        query.month.as_deref(),
        hone_core::beijing_now().date_naive(),
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
    let (actor, user_id) = match require_public_actor(&state, &headers) {
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
    let validated_mobile_path = if let Some(raw_mobile_path) = request
        .mobile_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !raw_mobile_path.to_ascii_lowercase().ends_with(".png") {
            return json_error(StatusCode::BAD_REQUEST, "移动端财经日历只接受 PNG 图片");
        }
        match crate::routes::public::validate_public_upload_path(
            &upload_root,
            oss.as_ref(),
            &user_id,
            raw_mobile_path,
        ) {
            Ok(path) => Some(path),
            Err(response) => return response,
        }
    } else {
        None
    };

    let month = request
        .month
        .as_deref()
        .and_then(|value| parse_month_spec(value).ok())
        .map(|month| month.value())
        .unwrap_or_else(|| hone_core::beijing_now().format("%Y-%m").to_string());
    let content = finance_calendar_assistant_message(
        &validated_path,
        validated_mobile_path.as_deref(),
        &month,
    );
    let session_id = actor.session_id();
    if state
        .core
        .session_storage
        .load_session(&session_id)
        .ok()
        .flatten()
        .is_none()
        && let Err(error) = state.core.session_storage.create_session_for_actor(&actor)
    {
        return json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("创建会话失败: {error}"),
        );
    }
    match state
        .core
        .session_storage
        .add_message(&session_id, "assistant", &content, None)
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
    let holdings = portfolio_calendar_symbols(state, actor);
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
        today: hone_core::beijing_now()
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

fn require_public_actor(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(ActorIdentity, String), Response> {
    let user = crate::routes::public::require_public_user(state, headers)?;
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
            "北京时间 22:00 · 6月",
            "ismworld.org",
        ),
        (
            "2026-07-02",
            "美国非农就业报告",
            "北京时间 20:30 · 6月",
            "bls.gov",
        ),
        (
            "2026-07-06",
            "ISM 服务业 PMI",
            "北京时间 22:00 · 6月",
            "ismworld.org",
        ),
        (
            "2026-07-07",
            "美国贸易帐",
            "北京时间 20:30 · 5月",
            "bea.gov",
        ),
        (
            "2026-07-09",
            "FOMC 会议纪要",
            "北京时间 02:00 · 6月会议",
            "federalreserve.gov",
        ),
        ("2026-07-14", "美国 CPI", "北京时间 20:30 · 6月", "bls.gov"),
        ("2026-07-15", "美国 PPI", "北京时间 20:30 · 6月", "bls.gov"),
        (
            "2026-07-16",
            "美联储褐皮书",
            "北京时间 02:00",
            "federalreserve.gov",
        ),
        (
            "2026-07-16",
            "美国零售销售",
            "北京时间 20:30 · 6月",
            "census.gov",
        ),
        (
            "2026-07-17",
            "美国新屋开工",
            "北京时间 20:30 · 6月",
            "census.gov",
        ),
        (
            "2026-07-17",
            "美国工业产出",
            "北京时间 21:15 · 6月",
            "federalreserve.gov",
        ),
        (
            "2026-07-24",
            "美国新屋销售",
            "北京时间 22:00 · 6月",
            "census.gov",
        ),
        (
            "2026-07-27",
            "美国耐用品订单",
            "北京时间 20:30 · 6月",
            "census.gov",
        ),
        (
            "2026-07-30",
            "FOMC 利率决议与记者会",
            "北京时间 02:00 / 02:30",
            "federalreserve.gov",
        ),
        (
            "2026-07-30",
            "美国二季度 GDP 初值",
            "北京时间 20:30",
            "bea.gov",
        ),
        (
            "2026-07-30",
            "美国 PCE 物价指数",
            "北京时间 20:30 · 6月",
            "bea.gov",
        ),
        (
            "2026-07-31",
            "美国就业成本指数",
            "北京时间 20:30 · 二季度",
            "bls.gov",
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

fn portfolio_calendar_symbols(state: &AppState, actor: &ActorIdentity) -> Vec<String> {
    let portfolio_storage =
        hone_memory::PortfolioStorage::new(&state.core.config.storage.portfolio_dir);
    let Ok(Some(portfolio)) = portfolio_storage.load(actor) else {
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
    raw.trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        .collect::<String>()
        .to_ascii_uppercase()
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
    let pool = state.core.config.fmp.effective_key_pool();
    let keys = pool.keys();
    if keys.is_empty() {
        return EarningsFetchOutcome::MissingKey;
    }

    let mut events = Vec::new();
    let mut errors = Vec::new();
    for symbol in symbols {
        match fetch_symbol_earnings(state, keys, symbol).await {
            Ok(value) => events.extend(earnings_events_from_value(symbol, &value, month)),
            Err(error) => {
                warn!(%symbol, "finance calendar FMP earnings fetch failed: {error}");
                errors.push(format!("{symbol}: {error}"));
            }
        }
    }

    if errors.is_empty() {
        EarningsFetchOutcome::Ok(events)
    } else if events.is_empty() {
        EarningsFetchOutcome::Failed(errors)
    } else {
        EarningsFetchOutcome::Partial { events, errors }
    }
}

async fn fetch_symbol_earnings(
    state: &AppState,
    keys: &[String],
    symbol: &str,
) -> Result<Value, String> {
    let stable_base = stable_fmp_base_url(&state.core.config.fmp.base_url);
    let encoded_symbol = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
    let url_base = format!("{stable_base}/stable/earnings?symbol={encoded_symbol}");
    let mut last_error = String::new();
    for key in keys {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("{url_base}&apikey={encoded_key}");
        match fetch_fmp_json_once(&state.http_client, &url, state.core.config.fmp.timeout).await {
            Ok(value) => return Ok(value),
            Err(error) => last_error = error,
        }
    }
    Err(if last_error.is_empty() {
        "FMP 请求失败".to_string()
    } else {
        last_error
    })
}

async fn fetch_fmp_json_once(
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
    let value: Value = serde_json::from_str(&body)
        .map_err(|error| sanitize_fmp_error(&format!("FMP JSON 解析失败: {error}")))?;
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return Err(format!("FMP Key 无效（HTTP {status}）"));
    }
    if let Some(message) = value.get("Error Message").and_then(|value| value.as_str()) {
        return Err(sanitize_fmp_error(message));
    }
    Ok(value)
}

fn stable_fmp_base_url(base_url: &str) -> String {
    let mut base = base_url.trim_end_matches('/').to_string();
    for suffix in ["/api/v3", "/api"] {
        if let Some(stripped) = base.strip_suffix(suffix) {
            base = stripped.to_string();
            break;
        }
    }
    base.trim_end_matches('/').to_string()
}

fn earnings_events_from_value(
    requested_symbol: &str,
    value: &Value,
    month: &MonthSpec,
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
        if date.year() != month.year || date.month() != month.month {
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
        while let Some(index) = out.find(&format!("{key}=")) {
            let value_start = index + key.len() + 1;
            let value_end = out[value_start..]
                .char_indices()
                .find_map(|(idx, ch)| (ch == '&' || ch.is_whitespace()).then_some(idx))
                .map(|idx| value_start + idx)
                .unwrap_or(out.len());
            out.replace_range(value_start..value_end, "<redacted>");
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
        assert!(macro_events_for_month(&august).is_empty());
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
                tracking_only: Some(true),
            },
        ];
        assert_eq!(
            calendar_symbols_from_holdings(&holdings),
            vec!["AAPL".to_string(), "BRK.B".to_string()]
        );
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
