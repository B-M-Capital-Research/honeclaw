//! Daily, reproducible valuation research for the public HONE workspace.
//!
//! Numeric assumptions in this module are HONE-owned analytical defaults. The
//! Hari framework constrains evidence quality and falsifiability; it does not
//! provide fixed discount rates, multiples, or price targets.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Semaphore;
use tracing::{info, warn};

use crate::routes::public_finance_calendar::fetch_fmp_json_once;
use crate::state::AppState;

const CARDS_JSON: &str =
    include_str!("../../../../skills/company-thesis-ratings/references/company-cards.json");
const REFRESH_HOUR: u32 = 19;
const REFRESH_MINUTE: u32 = 20;
const STALE_AFTER_HOURS: i64 = 36;
const MAX_QUOTE_AGE_DAYS: i64 = 4;
const MAX_FINANCIAL_AGE_DAYS: i64 = 200;
const MIN_METHODS_FOR_RATING: usize = 2;
const MAX_BASE_METHOD_DISPERSION_PERCENT: f64 = 50.0;

#[derive(Debug, Clone, Deserialize)]
struct CardFile {
    companies: Vec<ValuationCompany>,
}

#[derive(Debug, Clone, Deserialize)]
struct ValuationCompany {
    name: String,
    symbol: String,
    market_scope: String,
    theme: String,
    valuation_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValuationEvidence {
    pub label: String,
    pub display_value: String,
    pub as_of: String,
    pub source: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValuationScenario {
    pub id: String,
    pub label: String,
    pub initial_growth_rate: f64,
    pub discount_rate: f64,
    pub terminal_growth_rate: f64,
    pub probability: f64,
    pub dcf_value: Option<f64>,
    pub multiple_value: Option<f64>,
    pub methods: Vec<ValuationMethodResult>,
    pub fair_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValuationMethodResult {
    pub id: String,
    pub label: String,
    pub value: f64,
    pub weight: f64,
    pub metric: String,
    pub assumption: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValuationCrossCheck {
    pub status: String,
    pub method_count: usize,
    pub dispersion_percent: Option<f64>,
    pub forward_eps: Option<f64>,
    pub forward_pe: Option<f64>,
    pub pe_value: Option<f64>,
    pub dcf_value: Option<f64>,
    pub gap_percent: Option<f64>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReverseDcfResult {
    pub status: String,
    pub implied_growth_rate: Option<f64>,
    pub implied_forward_eps: Option<f64>,
    pub implied_forward_pe: Option<f64>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValuationLabItem {
    pub symbol: String,
    pub name: String,
    pub market_scope: String,
    pub theme: String,
    pub status: String,
    pub confidence: String,
    pub eligible_for_rating: bool,
    pub unavailable_reason: String,
    pub currency: String,
    pub current_price: Option<f64>,
    pub market_as_of: Option<String>,
    pub financial_as_of: Option<String>,
    pub normalized_fcf: Option<f64>,
    pub normalized_fcf_per_share: Option<f64>,
    pub net_cash_per_share: Option<f64>,
    pub historical_fcf_growth_rate: Option<f64>,
    #[serde(default)]
    pub revenue_growth_percent: Option<f64>,
    #[serde(default)]
    pub forward_revenue_growth_percent: Option<f64>,
    #[serde(default)]
    pub gross_margin_percent: Option<f64>,
    #[serde(default)]
    pub gross_margin_change_pp: Option<f64>,
    #[serde(default)]
    pub ebit_margin_percent: Option<f64>,
    #[serde(default)]
    pub fcf_margin_percent: Option<f64>,
    #[serde(default)]
    pub net_cash_to_revenue_percent: Option<f64>,
    pub current_position: String,
    pub position_percent: Option<f64>,
    pub method: String,
    pub valuation_profile: String,
    pub company_method_hint: String,
    pub scenarios: Vec<ValuationScenario>,
    pub probability_weighted_value: Option<f64>,
    pub expected_upside_percent: Option<f64>,
    pub reverse_dcf: Option<ReverseDcfResult>,
    pub cross_check: Option<ValuationCrossCheck>,
    pub assumptions: Vec<String>,
    pub evidence: Vec<ValuationEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValuationLabSnapshot {
    pub report_date: String,
    pub generated_at: DateTime<Utc>,
    #[serde(alias = "generated_at_beijing")]
    pub generated_at_local: String,
    pub next_refresh_at: DateTime<Utc>,
    pub timezone: String,
    pub methodology_version: String,
    pub status: String,
    pub coverage: ValuationCoverage,
    pub summary: String,
    pub methodology_note: String,
    pub items: Vec<ValuationLabItem>,
    pub disclaimer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ValuationCoverage {
    pub companies: usize,
    pub calculated: usize,
    pub cross_checked: usize,
    pub eligible_for_rating: usize,
}

#[derive(Debug, Clone, Default)]
struct QuoteInput {
    price: f64,
    shares: f64,
    as_of: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct FinancialInput {
    as_of: Option<String>,
    current_fcf: Option<f64>,
    prior_fcf: Option<f64>,
    annual_fcf_history: Vec<f64>,
    net_cash: Option<f64>,
    contract_liabilities: Option<f64>,
    forward_eps: Option<f64>,
    forward_revenue: Option<f64>,
    current_revenue: Option<f64>,
    prior_revenue: Option<f64>,
    current_ebit: Option<f64>,
    normalized_ebit_margin: Option<f64>,
    current_gross_margin: Option<f64>,
    prior_gross_margin: Option<f64>,
    current_ebit_margin: Option<f64>,
    current_fcf_margin: Option<f64>,
    net_cash_to_revenue: Option<f64>,
    forward_revenue_growth: Option<f64>,
}

#[derive(Debug, Clone, Default)]
struct IncomeWindows {
    as_of: Option<String>,
    current_revenue: Option<f64>,
    prior_revenue: Option<f64>,
    current_ebit: Option<f64>,
    normalized_ebit_margin: Option<f64>,
    current_gross_margin: Option<f64>,
    prior_gross_margin: Option<f64>,
    current_ebit_margin: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValuationProfile {
    CyclicalManufacturing,
    ProfitableGrowth,
    RevenueTransition,
}

impl ValuationProfile {
    fn id(self) -> &'static str {
        match self {
            Self::CyclicalManufacturing => "cyclical_manufacturing",
            Self::ProfitableGrowth => "profitable_growth",
            Self::RevenueTransition => "revenue_transition",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::CyclicalManufacturing => "周期制造：前瞻 P/E + EV/EBIT + 周期调整 DCF",
            Self::ProfitableGrowth => "盈利成长：前瞻 P/E + DCF + EV/EBIT",
            Self::RevenueTransition => "收入转型：EV/S + DCF/盈利拐点交叉验证",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ScenarioConfig {
    id: &'static str,
    label: &'static str,
    probability: f64,
    earnings_factor: f64,
    revenue_factor: f64,
    margin_factor: f64,
    discount_rate: f64,
    terminal_growth_rate: f64,
}

#[derive(Debug, Serialize)]
struct RatingValuationFile<'a> {
    report_date: &'a str,
    framework_version: &'static str,
    generated_at: DateTime<Utc>,
    items: Vec<RatingValuationItem>,
}

#[derive(Debug, Serialize)]
struct RatingValuationItem {
    symbol: String,
    as_of: String,
    currency: String,
    bear_case: f64,
    base_case: f64,
    bull_case: f64,
    current_price: f64,
    probability_weighted_value: f64,
    expected_upside_percent: f64,
    method_count: usize,
    confidence: String,
    method: String,
    assumptions: Vec<String>,
    sources: Vec<String>,
    review_status: String,
}

#[derive(Debug, Serialize)]
struct RatingFundamentalFile<'a> {
    report_date: &'a str,
    framework_version: &'static str,
    generated_at: DateTime<Utc>,
    items: Vec<RatingFundamentalItem>,
}

#[derive(Debug, Serialize)]
struct RatingFundamentalItem {
    symbol: String,
    as_of: String,
    revenue_growth_percent: Option<f64>,
    forward_revenue_growth_percent: Option<f64>,
    gross_margin_percent: Option<f64>,
    gross_margin_change_pp: Option<f64>,
    ebit_margin_percent: Option<f64>,
    fcf_margin_percent: Option<f64>,
    net_cash_to_revenue_percent: Option<f64>,
    sources: Vec<String>,
    review_status: &'static str,
}

pub(crate) async fn handle_get_valuation_lab(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = crate::routes::public::require_public_user(&state, &headers) {
        return response;
    }
    let snapshot = read_snapshot(&state)
        .await
        .map(mark_stale_if_needed)
        .unwrap_or_else(unavailable_snapshot);
    Json(snapshot).into_response()
}

/// Compact overview projection of the latest stored snapshot. `None` when no
/// snapshot file exists yet; the aggregator renders a waiting card instead.
pub(crate) async fn overview_card(
    state: &AppState,
) -> Option<crate::routes::research_overview::OverviewCard> {
    let snapshot = mark_stale_if_needed(read_snapshot(state).await?);
    let mut card = crate::routes::research_overview::OverviewCard::waiting(
        "valuation-lab",
        "估值实验室",
        "三情景估值",
    );
    card.report_date = Some(snapshot.report_date.clone());
    card.status = snapshot.status.clone();
    card.metric = Some(format!("{} 家覆盖", snapshot.coverage.companies));
    card.summary = Some(crate::routes::research_overview::short_summary(
        &snapshot.summary,
    ));
    card.generated_at = Some(snapshot.generated_at);
    Some(card)
}

pub(crate) async fn valuation_lab_worker(state: Arc<AppState>) {
    refresh_and_store(&state).await;
    loop {
        let next = next_refresh(Utc::now());
        let wait = (next - Utc::now())
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(60));
        info!(next_refresh = %next, "valuation lab worker waiting");
        tokio::time::sleep(wait).await;
        refresh_and_store(&state).await;
    }
}

async fn refresh_and_store(state: &AppState) {
    let snapshot = generate_snapshot(state).await;
    if let Err(error) = write_snapshot(state, &snapshot).await {
        warn!("valuation lab snapshot write failed: {error}");
        return;
    }
    if let Err(error) = write_rating_valuations(state, &snapshot).await {
        warn!("valuation lab rating input write failed: {error}");
        return;
    }
    if let Err(error) = write_rating_fundamentals(state, &snapshot).await {
        warn!("valuation lab fundamental input write failed: {error}");
        return;
    }
    info!(
        calculated = snapshot.coverage.calculated,
        eligible = snapshot.coverage.eligible_for_rating,
        status = %snapshot.status,
        "valuation lab snapshot refreshed"
    );
    crate::routes::company_ratings::refresh_and_store(state).await;
}

async fn generate_snapshot(state: &AppState) -> ValuationLabSnapshot {
    let companies = parse_companies();
    let now = Utc::now();
    let report_date = hone_core::local_time_at(now).date_naive();
    let keys = state.core.config.fmp.effective_key_pool().keys().to_vec();
    if keys.is_empty() {
        return snapshot_from_inputs(
            companies,
            HashMap::new(),
            HashMap::new(),
            now,
            "FMP 未配置，无法核验当日价格、现金流和股本。".to_string(),
        );
    }

    let symbols = companies
        .iter()
        .map(|item| item.symbol.clone())
        .collect::<Vec<_>>();
    let quotes = fetch_quotes(state, &keys, &symbols)
        .await
        .unwrap_or_else(|error| {
            warn!("valuation quotes unavailable: {error}");
            HashMap::new()
        });
    let financials = fetch_financial_inputs(state, &keys, &symbols, report_date).await;
    let source_error = if quotes.is_empty() {
        "当日行情源不可用。".to_string()
    } else if financials.is_empty() {
        "财务与预期数据源不可用。".to_string()
    } else {
        String::new()
    };
    snapshot_from_inputs(companies, quotes, financials, now, source_error)
}

fn parse_companies() -> Vec<ValuationCompany> {
    serde_json::from_str::<CardFile>(CARDS_JSON)
        .expect("company valuation cards must be valid JSON")
        .companies
}

fn snapshot_from_inputs(
    companies: Vec<ValuationCompany>,
    quotes: HashMap<String, QuoteInput>,
    financials: HashMap<String, FinancialInput>,
    now: DateTime<Utc>,
    source_error: String,
) -> ValuationLabSnapshot {
    let report_date = hone_core::local_time_at(now).date_naive();
    let mut items = companies
        .into_iter()
        .map(|company| {
            let symbol = company.symbol.clone();
            valuation_item(
                company,
                quotes.get(&symbol),
                financials.get(&symbol),
                report_date,
                &source_error,
            )
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| {
        b.eligible_for_rating
            .cmp(&a.eligible_for_rating)
            .then_with(|| {
                b.position_percent
                    .partial_cmp(&a.position_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    let coverage = ValuationCoverage {
        companies: items.len(),
        calculated: items
            .iter()
            .filter(|item| !item.scenarios.is_empty())
            .count(),
        cross_checked: items
            .iter()
            .filter(|item| {
                item.cross_check
                    .as_ref()
                    .is_some_and(|value| value.status != "unavailable")
            })
            .count(),
        eligible_for_rating: items.iter().filter(|item| item.eligible_for_rating).count(),
    };
    let status = if coverage.eligible_for_rating == coverage.companies && coverage.companies > 0 {
        "live"
    } else if coverage.calculated > 0 {
        "partial"
    } else {
        "data_unavailable"
    };
    let summary = if coverage.eligible_for_rating > 0 {
        format!(
            "已对 {} 家公司完成估值，其中 {} 家通过当日数据与交叉验证门槛，可进入公司评级。",
            coverage.calculated, coverage.eligible_for_rating
        )
    } else if coverage.calculated > 0 {
        format!(
            "已形成 {} 家多方法估值草案，但尚无公司同时通过价格新鲜度和方法离散度门槛。",
            coverage.calculated
        )
    } else {
        "当前没有满足核心输入门槛的公司；系统没有沿用旧目标价或生成演示估值。".to_string()
    };
    ValuationLabSnapshot {
        report_date: report_date.to_string(),
        generated_at: now,
        generated_at_local: hone_core::local_time_at(now).format("%Y-%m-%d %H:%M").to_string(),
        next_refresh_at: next_refresh(now),
        timezone: hone_core::runtime_timezone_name(),
        methodology_version: "hone-valuation-v2".to_string(),
        status: status.to_string(),
        coverage,
        summary,
        methodology_note: "按公司商业模式选择估值组合：周期制造以前瞻 P/E 为主、EV/EBIT 与周期调整 DCF 交叉验证；盈利成长结合前瞻 P/E、DCF 与 EV/EBIT；收入转型公司以 EV/S 为主，并要求第二方法验证。三情景按 20%/55%/25% 概率汇总，同时反推当前股价隐含的 EPS、P/E 或现金流增长。所有参数都是 HONE 可复算模型假设，不冒充老王固定倍数。".to_string(),
        items,
        disclaimer: "估值是研究区间，不是目标价承诺、买卖指令或收益保证；不适用的商业模式和缺失数据会明确留空。".to_string(),
    }
}

fn valuation_item(
    company: ValuationCompany,
    quote: Option<&QuoteInput>,
    financial: Option<&FinancialInput>,
    report_date: NaiveDate,
    source_error: &str,
) -> ValuationLabItem {
    let mut item = empty_item(&company, source_error);
    let profile = profile_for_company(&company);
    item.valuation_profile = profile.id().to_string();
    item.method = profile.label().to_string();
    if let Some(financial) = financial {
        item.financial_as_of = financial.as_of.clone();
        item.revenue_growth_percent =
            ratio_change(financial.current_revenue, financial.prior_revenue)
                .map(|value| round1(value * 100.0));
        item.forward_revenue_growth_percent = financial
            .forward_revenue_growth
            .map(|value| round1(value * 100.0));
        item.gross_margin_percent = financial
            .current_gross_margin
            .map(|value| round1(value * 100.0));
        item.gross_margin_change_pp =
            match (financial.current_gross_margin, financial.prior_gross_margin) {
                (Some(current), Some(prior)) => Some(round1((current - prior) * 100.0)),
                _ => None,
            };
        item.ebit_margin_percent = financial
            .current_ebit_margin
            .map(|value| round1(value * 100.0));
        item.fcf_margin_percent = financial
            .current_fcf_margin
            .map(|value| round1(value * 100.0));
        item.net_cash_to_revenue_percent = financial
            .net_cash_to_revenue
            .map(|value| round1(value * 100.0));
    }
    let Some(quote) = quote else {
        item.unavailable_reason = if source_error.is_empty() {
            "缺少当日行情或流通股本。"
        } else {
            source_error
        }
        .to_string();
        return item;
    };
    item.current_price = Some(round2(quote.price));
    item.market_as_of = quote.as_of.clone();
    if !date_is_fresh(quote.as_of.as_deref(), report_date, MAX_QUOTE_AGE_DAYS) {
        item.unavailable_reason = "行情日期过旧或无法确认，不能定位当前价格。".to_string();
        return item;
    }
    let Some(financial) = financial else {
        item.unavailable_reason = "缺少现金流、利润表、资产负债表或下一财年一致预期。".to_string();
        return item;
    };
    if !date_is_fresh(
        financial.as_of.as_deref(),
        report_date,
        MAX_FINANCIAL_AGE_DAYS,
    ) {
        item.unavailable_reason = "财务数据超过 200 天，不能作为当日估值输入。".to_string();
        return item;
    }
    let Some(raw_net_cash) = financial.net_cash else {
        item.unavailable_reason = "净现金或净负债输入不完整。".to_string();
        return item;
    };
    let net_cash = if profile == ValuationProfile::CyclicalManufacturing {
        raw_net_cash - financial.contract_liabilities.unwrap_or(0.0)
    } else {
        raw_net_cash
    };
    if quote.shares <= 0.0 {
        item.unavailable_reason = "稀释股本不满足正值条件。".to_string();
        return item;
    }

    let net_cash_per_share = net_cash / quote.shares;
    let historical_growth = match (financial.current_fcf, financial.prior_fcf) {
        (Some(current), Some(prior)) if current > 0.0 && prior > 0.0 => {
            Some((current / prior - 1.0).clamp(-0.20, 0.35))
        }
        _ => None,
    };
    let normalized_fcf = normalized_positive_cashflow(&financial.annual_fcf_history);
    let configs = scenario_configs(profile);
    let scenarios = configs
        .iter()
        .map(|config| {
            build_scenario(
                profile,
                *config,
                quote,
                financial,
                net_cash_per_share,
                normalized_fcf,
                historical_growth,
            )
        })
        .collect::<Vec<_>>();
    if scenarios
        .iter()
        .any(|scenario| scenario.methods.len() < MIN_METHODS_FOR_RATING)
    {
        item.unavailable_reason = format!(
            "{} 至少需要两种可复算方法；当前 EPS、EBIT、收入或正自由现金流不足。",
            profile.label()
        );
        item.scenarios = scenarios;
        item.net_cash_per_share = Some(round2(net_cash_per_share));
        item.assumptions = assumptions_for_item(profile, financial, normalized_fcf);
        item.evidence = evidence_for_item(&company.symbol, quote, financial, net_cash);
        return item;
    }
    if scenarios
        .iter()
        .any(|value| !value.fair_value.is_finite() || value.fair_value <= 0.0)
        || !(scenarios[0].fair_value < scenarios[1].fair_value
            && scenarios[1].fair_value < scenarios[2].fair_value)
    {
        item.unavailable_reason = "三情景结果没有形成有序正值区间。".to_string();
        return item;
    }
    let base = &scenarios[1];
    let cross_check = build_cross_check(base, financial.forward_eps);
    let gap_ok = cross_check
        .dispersion_percent
        .is_some_and(|gap| gap <= MAX_BASE_METHOD_DISPERSION_PERCENT);
    let reverse = build_reverse_valuation(profile, quote, financial, net_cash_per_share, base);
    let eligible = gap_ok;
    let probability_weighted_value = scenarios
        .iter()
        .map(|scenario| scenario.fair_value * scenario.probability)
        .sum::<f64>();
    let expected_upside = probability_weighted_value / quote.price - 1.0;
    let position_percent = (quote.price - scenarios[0].fair_value)
        / (scenarios[2].fair_value - scenarios[0].fair_value)
        * 100.0;
    let current_position = range_position(
        quote.price,
        scenarios[0].fair_value,
        scenarios[1].fair_value,
        scenarios[2].fair_value,
    );
    item.status = if eligible { "ready" } else { "review_required" }.to_string();
    item.confidence = if eligible
        && cross_check
            .dispersion_percent
            .is_some_and(|gap| gap <= 25.0)
        && base.methods.len() == 3
    {
        "high"
    } else if eligible {
        "medium"
    } else {
        "low"
    }
    .to_string();
    item.eligible_for_rating = eligible;
    item.unavailable_reason = if eligible {
        String::new()
    } else if !gap_ok {
        "基准情景各方法估值离散度超过 50%，不能自动进入公司评级。".to_string()
    } else {
        "估值方法覆盖不足，需要人工复核。".to_string()
    };
    item.normalized_fcf = normalized_fcf.map(round2);
    item.normalized_fcf_per_share = normalized_fcf.map(|value| round2(value / quote.shares));
    item.net_cash_per_share = Some(round2(net_cash_per_share));
    item.historical_fcf_growth_rate = historical_growth.map(round4);
    item.current_position = current_position.to_string();
    item.position_percent = Some(round1(position_percent));
    item.scenarios = scenarios;
    item.probability_weighted_value = Some(round2(probability_weighted_value));
    item.expected_upside_percent = Some(round1(expected_upside * 100.0));
    item.reverse_dcf = Some(reverse);
    item.cross_check = Some(cross_check);
    item.assumptions = assumptions_for_item(profile, financial, normalized_fcf);
    item.evidence = evidence_for_item(&company.symbol, quote, financial, net_cash);
    item
}

fn empty_item(company: &ValuationCompany, source_error: &str) -> ValuationLabItem {
    ValuationLabItem {
        symbol: company.symbol.clone(),
        name: company.name.clone(),
        market_scope: company.market_scope.clone(),
        theme: company.theme.clone(),
        status: "unavailable".to_string(),
        confidence: "low".to_string(),
        eligible_for_rating: false,
        unavailable_reason: source_error.to_string(),
        currency: "USD".to_string(),
        current_price: None,
        market_as_of: None,
        financial_as_of: None,
        normalized_fcf: None,
        normalized_fcf_per_share: None,
        net_cash_per_share: None,
        historical_fcf_growth_rate: None,
        revenue_growth_percent: None,
        forward_revenue_growth_percent: None,
        gross_margin_percent: None,
        gross_margin_change_pp: None,
        ebit_margin_percent: None,
        fcf_margin_percent: None,
        net_cash_to_revenue_percent: None,
        current_position: "无法判断".to_string(),
        position_percent: None,
        method: "按商业模式选择多方法估值".to_string(),
        valuation_profile: String::new(),
        company_method_hint: company.valuation_method.clone(),
        scenarios: Vec::new(),
        probability_weighted_value: None,
        expected_upside_percent: None,
        reverse_dcf: None,
        cross_check: None,
        assumptions: Vec::new(),
        evidence: Vec::new(),
    }
}

fn profile_for_company(company: &ValuationCompany) -> ValuationProfile {
    let method = company.valuation_method.as_str();
    if method.contains("中周期")
        || method.contains("跨 WFE")
        || company.theme.contains("存储")
        || company.theme.contains("半导体设备")
    {
        ValuationProfile::CyclicalManufacturing
    } else if method.contains("EV/S")
        || method.contains("商业化里程碑")
        || company.theme.contains("New Cloud")
        || company.theme.contains("AI医疗")
        || company.theme.contains("航天")
    {
        ValuationProfile::RevenueTransition
    } else {
        ValuationProfile::ProfitableGrowth
    }
}

fn scenario_configs(profile: ValuationProfile) -> [ScenarioConfig; 3] {
    let rates = match profile {
        ValuationProfile::CyclicalManufacturing => [(0.125, 0.02), (0.105, 0.025), (0.10, 0.03)],
        ValuationProfile::ProfitableGrowth => [(0.12, 0.02), (0.10, 0.03), (0.09, 0.035)],
        ValuationProfile::RevenueTransition => [(0.13, 0.02), (0.11, 0.025), (0.10, 0.03)],
    };
    [
        ScenarioConfig {
            id: "bear",
            label: "悲观",
            probability: 0.20,
            earnings_factor: 0.75,
            revenue_factor: 0.90,
            margin_factor: 0.75,
            discount_rate: rates[0].0,
            terminal_growth_rate: rates[0].1,
        },
        ScenarioConfig {
            id: "base",
            label: "基准",
            probability: 0.55,
            earnings_factor: 1.0,
            revenue_factor: 1.0,
            margin_factor: 1.0,
            discount_rate: rates[1].0,
            terminal_growth_rate: rates[1].1,
        },
        ScenarioConfig {
            id: "bull",
            label: "乐观",
            probability: 0.25,
            earnings_factor: 1.25,
            revenue_factor: 1.10,
            margin_factor: 1.20,
            discount_rate: rates[2].0,
            terminal_growth_rate: rates[2].1,
        },
    ]
}

fn build_scenario(
    profile: ValuationProfile,
    config: ScenarioConfig,
    quote: &QuoteInput,
    financial: &FinancialInput,
    net_cash_per_share: f64,
    normalized_fcf: Option<f64>,
    historical_growth: Option<f64>,
) -> ValuationScenario {
    let forward_growth = forward_revenue_growth(financial)
        .unwrap_or(0.08)
        .clamp(-0.20, 0.50);
    let initial_growth = match config.id {
        "bear" => historical_growth.unwrap_or(forward_growth) - 0.08,
        "bull" => historical_growth.unwrap_or(forward_growth) + 0.08,
        _ => historical_growth.unwrap_or(forward_growth),
    }
    .clamp(-0.20, 0.45);
    let mut methods = Vec::new();

    if let Some(eps) = financial.forward_eps.filter(|value| *value > 0.0) {
        let multiple = forward_pe_multiple(profile, config.id, forward_growth);
        let scenario_eps = eps * config.earnings_factor;
        methods.push(ValuationMethodResult {
            id: "forward_pe".to_string(),
            label: "前瞻 P/E".to_string(),
            value: scenario_eps * multiple,
            weight: method_weight(profile, "forward_pe"),
            metric: format!("{multiple:.1}x × ${scenario_eps:.2} EPS"),
            assumption: "下一财年 EPS 随情景调整；周期型公司使用中周期而非峰值倍数。".to_string(),
        });
    }

    if profile != ValuationProfile::RevenueTransition {
        if let (Some(forward_revenue), Some(margin)) = (
            financial.forward_revenue.filter(|value| *value > 0.0),
            financial
                .normalized_ebit_margin
                .filter(|value| *value > 0.0),
        ) {
            let multiple = ev_ebit_multiple(profile, config.id, forward_growth);
            let scenario_margin = (margin * config.margin_factor).clamp(0.01, 0.60);
            let ebit = forward_revenue * config.revenue_factor * scenario_margin;
            let value = (ebit * multiple + net_cash_per_share * quote.shares) / quote.shares;
            if value.is_finite() && value > 0.0 {
                methods.push(ValuationMethodResult {
                    id: "ev_ebit".to_string(),
                    label: "EV/EBIT".to_string(),
                    value,
                    weight: method_weight(profile, "ev_ebit"),
                    metric: format!(
                        "{multiple:.1}x × {:.1}% EBIT margin",
                        scenario_margin * 100.0
                    ),
                    assumption: "折旧与制造资产消耗保留在 EBIT 中，不以 EBITDA 淡化资本成本。"
                        .to_string(),
                });
            }
        }
    } else if let Some(forward_revenue) = financial.forward_revenue.filter(|value| *value > 0.0) {
        let multiple =
            ev_sales_multiple(config.id, forward_growth, financial.normalized_ebit_margin);
        let enterprise_value = forward_revenue * config.revenue_factor * multiple;
        let value = (enterprise_value + net_cash_per_share * quote.shares) / quote.shares;
        if value.is_finite() && value > 0.0 {
            methods.push(ValuationMethodResult {
                id: "ev_sales".to_string(),
                label: "EV/S".to_string(),
                value,
                weight: method_weight(profile, "ev_sales"),
                metric: format!("{multiple:.1}x × forward revenue"),
                assumption: "仅用于尚处盈利转型期的公司，并由现金流或盈利拐点方法交叉验证。"
                    .to_string(),
            });
        }
    }

    let dcf = match (financial.current_fcf, normalized_fcf) {
        (Some(current), Some(normalized)) if current > 0.0 && normalized > 0.0 => {
            if profile == ValuationProfile::CyclicalManufacturing
                && financial.annual_fcf_history.len() >= 3
            {
                let path = cycle_adjusted_fcf_path(current, normalized, config.id);
                Some(dcf_from_fcff_path(
                    &path,
                    quote.shares,
                    net_cash_per_share,
                    config.discount_rate,
                    config.terminal_growth_rate,
                ))
            } else {
                Some(dcf_per_share(
                    current / quote.shares,
                    net_cash_per_share,
                    initial_growth,
                    config.discount_rate,
                    config.terminal_growth_rate,
                ))
            }
        }
        _ => None,
    }
    .filter(|value| value.is_finite() && *value > 0.0);
    if let Some(value) = dcf {
        methods.push(ValuationMethodResult {
            id: "cycle_adjusted_dcf".to_string(),
            label: if profile == ValuationProfile::CyclicalManufacturing {
                "周期调整 DCF"
            } else {
                "DCF"
            }
            .to_string(),
            value,
            weight: method_weight(profile, "dcf"),
            metric: format!(
                "WACC {:.1}% · 永续 {:.1}%",
                config.discount_rate * 100.0,
                config.terminal_growth_rate * 100.0
            ),
            assumption: if profile == ValuationProfile::CyclicalManufacturing {
                "五年 FCFF 从当前盈利向历史中周期现金流收敛，避免永久化周期高点。"
            } else {
                "五年增长逐步向终值增长率收敛，不永久外推当前高增长。"
            }
            .to_string(),
        });
    }

    let total_weight = methods.iter().map(|method| method.weight).sum::<f64>();
    if total_weight > 0.0 {
        for method in &mut methods {
            method.weight /= total_weight;
        }
    }
    let fair_value = methods
        .iter()
        .map(|method| method.value * method.weight)
        .sum::<f64>();
    let dcf_value = methods
        .iter()
        .find(|method| method.id == "cycle_adjusted_dcf")
        .map(|method| round2(method.value));
    let multiple_value = methods
        .iter()
        .find(|method| method.id == "forward_pe" || method.id == "ev_sales")
        .map(|method| round2(method.value));
    for method in &mut methods {
        method.value = round2(method.value);
        method.weight = round4(method.weight);
    }
    ValuationScenario {
        id: config.id.to_string(),
        label: config.label.to_string(),
        probability: config.probability,
        initial_growth_rate: round4(initial_growth),
        discount_rate: config.discount_rate,
        terminal_growth_rate: config.terminal_growth_rate,
        dcf_value,
        multiple_value,
        methods,
        fair_value: round2(fair_value),
    }
}

fn method_weight(profile: ValuationProfile, method: &str) -> f64 {
    match (profile, method) {
        (ValuationProfile::CyclicalManufacturing, "forward_pe") => 0.60,
        (ValuationProfile::CyclicalManufacturing, "ev_ebit") => 0.25,
        (ValuationProfile::CyclicalManufacturing, "dcf") => 0.15,
        (ValuationProfile::ProfitableGrowth, "forward_pe") => 0.40,
        (ValuationProfile::ProfitableGrowth, "ev_ebit") => 0.20,
        (ValuationProfile::ProfitableGrowth, "dcf") => 0.40,
        (ValuationProfile::RevenueTransition, "ev_sales") => 0.60,
        (ValuationProfile::RevenueTransition, "forward_pe") => 0.15,
        (ValuationProfile::RevenueTransition, "dcf") => 0.25,
        _ => 0.0,
    }
}

fn forward_pe_multiple(profile: ValuationProfile, scenario: &str, growth: f64) -> f64 {
    if profile == ValuationProfile::CyclicalManufacturing {
        return match scenario {
            "bear" => 8.0,
            "bull" => 13.0,
            _ => 10.5,
        };
    }
    let base = (18.0 + growth.max(0.0) * 35.0).clamp(14.0, 35.0);
    match scenario {
        "bear" => base * 0.75,
        "bull" => base * 1.25,
        _ => base,
    }
}

fn ev_ebit_multiple(profile: ValuationProfile, scenario: &str, growth: f64) -> f64 {
    if profile == ValuationProfile::CyclicalManufacturing {
        return match scenario {
            "bear" => 6.0,
            "bull" => 10.0,
            _ => 8.0,
        };
    }
    let base = (14.0 + growth.max(0.0) * 20.0).clamp(10.0, 24.0);
    match scenario {
        "bear" => base * 0.75,
        "bull" => base * 1.25,
        _ => base,
    }
}

fn ev_sales_multiple(scenario: &str, growth: f64, margin: Option<f64>) -> f64 {
    let base =
        (2.0 + growth.max(0.0) * 8.0 + margin.unwrap_or(0.0).max(0.0) * 8.0).clamp(1.5, 10.0);
    match scenario {
        "bear" => base * 0.60,
        "bull" => base * 1.40,
        _ => base,
    }
}

fn forward_revenue_growth(financial: &FinancialInput) -> Option<f64> {
    let current = financial.current_revenue.filter(|value| *value > 0.0)?;
    if let Some(forward) = financial.forward_revenue.filter(|value| *value > 0.0) {
        Some(forward / current - 1.0)
    } else {
        financial
            .prior_revenue
            .filter(|value| *value > 0.0)
            .map(|prior| current / prior - 1.0)
    }
}

fn normalized_positive_cashflow(history: &[f64]) -> Option<f64> {
    let mut values = history
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value > 0.0)
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }
    values.sort_by(|a, b| a.total_cmp(b));
    let middle = values.len() / 2;
    Some(if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    })
}

fn cycle_adjusted_fcf_path(current: f64, normalized: f64, scenario: &str) -> [f64; 5] {
    let (start_factor, end_factor) = match scenario {
        "bear" => (0.70, 0.75),
        "bull" => (1.10, 1.25),
        _ => (0.95, 1.0),
    };
    let start = current * start_factor;
    let end = normalized * end_factor;
    std::array::from_fn(|index| {
        let progress = index as f64 / 4.0;
        start + (end - start) * progress
    })
}

fn dcf_from_fcff_path(
    path: &[f64; 5],
    shares: f64,
    net_cash_per_share: f64,
    discount_rate: f64,
    terminal_growth_rate: f64,
) -> f64 {
    if shares <= 0.0 || discount_rate <= terminal_growth_rate {
        return f64::NAN;
    }
    let present = path
        .iter()
        .enumerate()
        .map(|(index, cashflow)| cashflow / (1.0 + discount_rate).powi(index as i32 + 1))
        .sum::<f64>();
    let terminal = path[4] * (1.0 + terminal_growth_rate)
        / (discount_rate - terminal_growth_rate)
        / (1.0 + discount_rate).powi(5);
    (present + terminal) / shares + net_cash_per_share
}

fn assumptions_for_item(
    profile: ValuationProfile,
    financial: &FinancialInput,
    normalized_fcf: Option<f64>,
) -> Vec<String> {
    let mut assumptions = vec![
        format!("估值类型：{}", profile.label()),
        "悲观/基准/乐观概率采用 20%/55%/25% 的 HONE 可审计默认值".to_string(),
        "所有可用方法先按类型权重加权；缺失方法不会填零，其余权重重新归一".to_string(),
    ];
    if profile == ValuationProfile::CyclicalManufacturing {
        assumptions.push("前瞻 P/E / EV/EBIT / 周期调整 DCF 原始权重为 60%/25%/15%".to_string());
        assumptions
            .push("周期 DCF 从当前 FCFF 向最多五个年度窗口的中位正常化 FCFF 回归".to_string());
    } else if profile == ValuationProfile::ProfitableGrowth {
        assumptions.push("前瞻 P/E / EV/EBIT / DCF 原始权重为 40%/20%/40%".to_string());
    } else {
        assumptions
            .push("EV/S / 前瞻 P/E / DCF 原始权重为 60%/15%/25%，且必须有第二方法".to_string());
    }
    if let Some(value) = normalized_fcf {
        assumptions.push(format!(
            "历史年度自由现金流正值中位数为 {:.2} 亿美元",
            value / 100_000_000.0
        ));
    }
    if profile == ValuationProfile::CyclicalManufacturing
        && financial.contract_liabilities.unwrap_or(0.0) > 0.0
    {
        assumptions.push(format!(
            "净现金已扣除约 {:.2} 亿美元合同负债/客户预付款，避免把履约义务当作闲置现金",
            financial.contract_liabilities.unwrap_or(0.0) / 100_000_000.0
        ));
    }
    assumptions
}

fn dcf_per_share(
    starting_fcf_per_share: f64,
    net_cash_per_share: f64,
    initial_growth: f64,
    discount_rate: f64,
    terminal_growth: f64,
) -> f64 {
    if starting_fcf_per_share <= 0.0 || discount_rate <= terminal_growth {
        return f64::NAN;
    }
    let mut fcf = starting_fcf_per_share;
    let mut present = 0.0;
    for year in 1..=5 {
        let progress = (year - 1) as f64 / 4.0;
        let growth = initial_growth + (terminal_growth - initial_growth) * progress;
        fcf *= 1.0 + growth;
        present += fcf / (1.0 + discount_rate).powi(year);
    }
    let terminal = fcf * (1.0 + terminal_growth) / (discount_rate - terminal_growth);
    present + terminal / (1.0 + discount_rate).powi(5) + net_cash_per_share
}

fn reverse_dcf(
    price: f64,
    fcf_per_share: f64,
    net_cash_per_share: f64,
    discount_rate: f64,
    terminal_growth: f64,
) -> ReverseDcfResult {
    let mut low = -0.20;
    let mut high = 0.60;
    let low_value = dcf_per_share(
        fcf_per_share,
        net_cash_per_share,
        low,
        discount_rate,
        terminal_growth,
    );
    let high_value = dcf_per_share(
        fcf_per_share,
        net_cash_per_share,
        high,
        discount_rate,
        terminal_growth,
    );
    if !low_value.is_finite() || !high_value.is_finite() || price < low_value || price > high_value
    {
        return ReverseDcfResult {
            status: "out_of_range".to_string(),
            implied_growth_rate: None,
            implied_forward_eps: None,
            implied_forward_pe: None,
            explanation: format!(
                "当前价格不在 -20% 至 60% 起始增长假设可解释的 DCF 区间（{low_value:.2}–{high_value:.2} 美元）。"
            ),
        };
    }
    for _ in 0..80 {
        let mid = (low + high) / 2.0;
        if dcf_per_share(
            fcf_per_share,
            net_cash_per_share,
            mid,
            discount_rate,
            terminal_growth,
        ) < price
        {
            low = mid;
        } else {
            high = mid;
        }
    }
    let implied = (low + high) / 2.0;
    ReverseDcfResult {
        status: "solved".to_string(),
        implied_growth_rate: Some(round4(implied)),
        implied_forward_eps: None,
        implied_forward_pe: None,
        explanation: format!(
            "按 {:.1}% 折现率和 {:.1}% 终值增长率，当前价格隐含未来五年起始自由现金流增速约 {:.1}%。",
            discount_rate * 100.0,
            terminal_growth * 100.0,
            implied * 100.0,
        ),
    }
}

fn build_cross_check(base: &ValuationScenario, forward_eps: Option<f64>) -> ValuationCrossCheck {
    if base.methods.len() < MIN_METHODS_FOR_RATING {
        return ValuationCrossCheck {
            status: "unavailable".to_string(),
            method_count: base.methods.len(),
            dispersion_percent: None,
            forward_eps,
            forward_pe: None,
            pe_value: None,
            dcf_value: base.dcf_value,
            gap_percent: None,
            explanation: "基准情景少于两种可复算方法，不能完成交叉验证。".to_string(),
        };
    }
    let min = base
        .methods
        .iter()
        .map(|method| method.value)
        .fold(f64::INFINITY, f64::min);
    let max = base
        .methods
        .iter()
        .map(|method| method.value)
        .fold(f64::NEG_INFINITY, f64::max);
    let dispersion = (max - min) / base.fair_value.max(1.0) * 100.0;
    let pe_value = base
        .methods
        .iter()
        .find(|method| method.id == "forward_pe")
        .map(|method| method.value);
    let pe = pe_value
        .zip(forward_eps.filter(|value| *value > 0.0))
        .map(|(value, eps)| value / eps);
    ValuationCrossCheck {
        status: if dispersion <= MAX_BASE_METHOD_DISPERSION_PERCENT {
            "consistent"
        } else {
            "divergent"
        }
        .to_string(),
        method_count: base.methods.len(),
        dispersion_percent: Some(round1(dispersion)),
        forward_eps: forward_eps.map(round2),
        forward_pe: pe.map(round1),
        pe_value: pe_value.map(round2),
        dcf_value: base.dcf_value,
        gap_percent: Some(round1(dispersion)),
        explanation: if dispersion <= MAX_BASE_METHOD_DISPERSION_PERCENT {
            format!(
                "基准情景 {} 种方法的最高/最低结果离散度为 {:.1}%，通过交叉验证门槛。",
                base.methods.len(),
                dispersion
            )
        } else {
            format!(
                "基准情景 {} 种方法的最高/最低结果离散度为 {:.1}%，方法分歧过大。",
                base.methods.len(),
                dispersion
            )
        },
    }
}

fn build_reverse_valuation(
    profile: ValuationProfile,
    quote: &QuoteInput,
    financial: &FinancialInput,
    net_cash_per_share: f64,
    base: &ValuationScenario,
) -> ReverseDcfResult {
    let forward_growth = forward_revenue_growth(financial)
        .unwrap_or(0.08)
        .clamp(-0.20, 0.50);
    let base_pe = forward_pe_multiple(profile, "base", forward_growth);
    let implied_forward_eps = (base_pe > 0.0).then_some(quote.price / base_pe);
    let implied_forward_pe = financial
        .forward_eps
        .filter(|value| *value > 0.0)
        .map(|eps| quote.price / eps);
    let mut parts = Vec::new();
    if let Some(value) = implied_forward_eps {
        parts.push(format!(
            "按基准 {base_pe:.1} 倍 P/E，股价隐含下一财年 EPS ${value:.2}"
        ));
    }
    if let Some(value) = implied_forward_pe {
        parts.push(format!("按一致预期 EPS，当前隐含 {:.1} 倍前瞻 P/E", value));
    }
    let implied_growth_rate = financial
        .current_fcf
        .filter(|value| *value > 0.0)
        .and_then(|fcf| {
            let reverse = reverse_dcf(
                quote.price,
                fcf / quote.shares,
                net_cash_per_share,
                base.discount_rate,
                base.terminal_growth_rate,
            );
            if let Some(growth) = reverse.implied_growth_rate {
                parts.push(format!("DCF 隐含起始增长约 {:.1}%", growth * 100.0));
            }
            reverse.implied_growth_rate
        });
    ReverseDcfResult {
        status: if parts.is_empty() {
            "unavailable"
        } else {
            "solved"
        }
        .to_string(),
        implied_growth_rate,
        implied_forward_eps: implied_forward_eps.map(round2),
        implied_forward_pe: implied_forward_pe.map(round1),
        explanation: if parts.is_empty() {
            "缺少正 EPS 与正自由现金流，无法反推当前价格隐含假设。".to_string()
        } else {
            format!("{}。", parts.join("；"))
        },
    }
}

fn range_position(price: f64, bear: f64, base: f64, bull: f64) -> &'static str {
    if price < bear {
        "低于悲观值"
    } else if price < base {
        "悲观—基准之间"
    } else if price <= bull {
        "基准—乐观之间"
    } else {
        "高于乐观值"
    }
}

fn date_is_fresh(value: Option<&str>, today: NaiveDate, max_age_days: i64) -> bool {
    value
        .and_then(|value| value.get(..10))
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
        .is_some_and(|date| date <= today && (today - date).num_days() <= max_age_days)
}

fn evidence_for_item(
    symbol: &str,
    quote: &QuoteInput,
    financial: &FinancialInput,
    net_cash: f64,
) -> Vec<ValuationEvidence> {
    let encoded = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
    let mut evidence = vec![
        ValuationEvidence {
            label: "当前价格与股本".to_string(),
            display_value: format!(
                "${:.2} · {:.2} 亿股",
                quote.price,
                quote.shares / 100_000_000.0
            ),
            as_of: quote.as_of.clone().unwrap_or_default(),
            source: "FMP Stock Quote".to_string(),
            source_url: format!("https://financialmodelingprep.com/stable/quote?symbol={encoded}"),
        },
        ValuationEvidence {
            label: "调整后净现金 /（净负债）".to_string(),
            display_value: format!("{:.2} 亿美元", net_cash / 100_000_000.0),
            as_of: financial.as_of.clone().unwrap_or_default(),
            source: "FMP Balance Sheet Statement".to_string(),
            source_url: format!(
                "https://financialmodelingprep.com/stable/balance-sheet-statement?symbol={encoded}"
            ),
        },
    ];
    if let Some(fcf) = financial.current_fcf {
        evidence.push(ValuationEvidence {
            label: "最近四季度自由现金流".to_string(),
            display_value: format!("{:.2} 亿美元", fcf / 100_000_000.0),
            as_of: financial.as_of.clone().unwrap_or_default(),
            source: "FMP Cash Flow Statement".to_string(),
            source_url: format!(
                "https://financialmodelingprep.com/stable/cash-flow-statement?symbol={encoded}"
            ),
        });
    }
    if let (Some(revenue), Some(ebit)) = (financial.current_revenue, financial.current_ebit) {
        evidence.push(ValuationEvidence {
            label: "最近四季度收入与 EBIT".to_string(),
            display_value: format!(
                "{:.2} / {:.2} 亿美元",
                revenue / 100_000_000.0,
                ebit / 100_000_000.0
            ),
            as_of: financial.as_of.clone().unwrap_or_default(),
            source: "FMP Income Statement".to_string(),
            source_url: format!(
                "https://financialmodelingprep.com/stable/income-statement?symbol={encoded}"
            ),
        });
    }
    if let Some(eps) = financial.forward_eps {
        evidence.push(ValuationEvidence {
            label: "下一财年 EPS 一致预期".to_string(),
            display_value: format!("${eps:.2}"),
            as_of: quote.as_of.clone().unwrap_or_default(),
            source: "FMP Financial Estimates".to_string(),
            source_url: format!(
                "https://financialmodelingprep.com/stable/analyst-estimates?symbol={encoded}"
            ),
        });
    }
    if let Some(revenue) = financial.forward_revenue {
        evidence.push(ValuationEvidence {
            label: "下一财年收入一致预期".to_string(),
            display_value: format!("{:.2} 亿美元", revenue / 100_000_000.0),
            as_of: quote.as_of.clone().unwrap_or_default(),
            source: "FMP Financial Estimates".to_string(),
            source_url: format!(
                "https://financialmodelingprep.com/stable/analyst-estimates?symbol={encoded}"
            ),
        });
    }
    evidence
}

async fn fetch_quotes(
    state: &AppState,
    keys: &[String],
    symbols: &[String],
) -> Result<HashMap<String, QuoteInput>, String> {
    let joined = symbols.join(",");
    let encoded_symbols = utf8_percent_encode(&joined, NONALPHANUMERIC_COMPAT).to_string();
    let base = stable_base_url(&state.core.config.fmp.base_url);
    let legacy = quote_base_url(&state.core.config.fmp.base_url);
    let mut last_error = String::new();
    for key in keys {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        for url in [
            format!("{base}/stable/batch-quote?symbols={encoded_symbols}&apikey={encoded_key}"),
            format!("{legacy}/v3/quote/{encoded_symbols}?apikey={encoded_key}"),
        ] {
            match fetch_fmp_json_once(&state.http_client, &url, state.core.config.fmp.timeout).await
            {
                Ok(value) => {
                    let parsed = quotes_from_value(&value);
                    if !parsed.is_empty() {
                        return Ok(parsed);
                    }
                    last_error = "quote response contained no usable rows".to_string();
                }
                Err(error) => last_error = error,
            }
        }
    }
    Err(last_error)
}

// The standard NON_ALPHANUMERIC set escapes commas; both stable and legacy
// quote endpoints accept the encoded comma form.
const NONALPHANUMERIC_COMPAT: &percent_encoding::AsciiSet = NON_ALPHANUMERIC;

fn quotes_from_value(value: &Value) -> HashMap<String, QuoteInput> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let symbol = row.get("symbol")?.as_str()?.trim().to_uppercase();
            let price = row.get("price")?.as_f64()?;
            let shares = row
                .get("sharesOutstanding")
                .and_then(Value::as_f64)
                .or_else(|| {
                    row.get("marketCap")
                        .and_then(Value::as_f64)
                        .filter(|_| price > 0.0)
                        .map(|market_cap| market_cap / price)
                })?;
            if symbol.is_empty() || price <= 0.0 || shares <= 0.0 {
                return None;
            }
            let as_of = row
                .get("timestamp")
                .and_then(Value::as_i64)
                .and_then(|timestamp| DateTime::from_timestamp(timestamp, 0))
                .map(|date| date.date_naive().to_string());
            Some((
                symbol,
                QuoteInput {
                    price,
                    shares,
                    as_of,
                },
            ))
        })
        .collect()
}

async fn fetch_financial_inputs(
    state: &AppState,
    keys: &[String],
    symbols: &[String],
    report_date: NaiveDate,
) -> HashMap<String, FinancialInput> {
    let semaphore = Arc::new(Semaphore::new(6));
    let mut set = tokio::task::JoinSet::new();
    for (index, symbol) in symbols.iter().cloned().enumerate() {
        let permit = semaphore.clone().acquire_owned().await;
        let client = state.http_client.clone();
        let key = keys[index % keys.len()].clone();
        let base = stable_base_url(&state.core.config.fmp.base_url);
        let timeout = state.core.config.fmp.timeout;
        set.spawn(async move {
            let _permit = permit.ok()?;
            let encoded_symbol = utf8_percent_encode(&symbol, NON_ALPHANUMERIC).to_string();
            let encoded_key = utf8_percent_encode(&key, NON_ALPHANUMERIC).to_string();
            let cash_url = format!("{base}/stable/cash-flow-statement?symbol={encoded_symbol}&period=quarter&limit=20&apikey={encoded_key}");
            let income_url = format!("{base}/stable/income-statement?symbol={encoded_symbol}&period=quarter&limit=20&apikey={encoded_key}");
            let balance_url = format!("{base}/stable/balance-sheet-statement?symbol={encoded_symbol}&period=quarter&limit=1&apikey={encoded_key}");
            let estimates_url = format!("{base}/stable/analyst-estimates?symbol={encoded_symbol}&period=annual&page=0&limit=6&apikey={encoded_key}");
            let (cash, income, balance, estimates) = tokio::join!(
                fetch_fmp_json_once(&client, &cash_url, timeout),
                fetch_fmp_json_once(&client, &income_url, timeout),
                fetch_fmp_json_once(&client, &balance_url, timeout),
                fetch_fmp_json_once(&client, &estimates_url, timeout),
            );
            let input = financial_from_values(
                cash.ok().as_ref(),
                income.ok().as_ref(),
                balance.ok().as_ref(),
                estimates.ok().as_ref(),
                report_date,
            );
            if input.current_fcf.is_none()
                && input.current_revenue.is_none()
                && input.net_cash.is_none()
            {
                return None;
            }
            Some((symbol, input))
        });
    }
    let mut result = HashMap::new();
    while let Some(joined) = set.join_next().await {
        if let Ok(Some((symbol, input))) = joined {
            result.insert(symbol, input);
        }
    }
    result
}

fn financial_from_values(
    cash: Option<&Value>,
    income: Option<&Value>,
    balance: Option<&Value>,
    estimates: Option<&Value>,
    report_date: NaiveDate,
) -> FinancialInput {
    let (cash_as_of, current_fcf, prior_fcf, annual_fcf_history) = cashflow_windows(cash);
    let income = income_windows(income);
    let (raw_net_cash, contract_liabilities) = balance_inputs(balance);
    let (forward_eps, forward_revenue) = estimates_from_value(estimates, report_date);
    let current_fcf_margin = ratio(current_fcf, income.current_revenue);
    let net_cash_to_revenue = ratio(raw_net_cash, income.current_revenue);
    let forward_revenue_growth = ratio_change(forward_revenue, income.current_revenue);
    FinancialInput {
        as_of: cash_as_of.or(income.as_of),
        current_fcf,
        prior_fcf,
        annual_fcf_history,
        net_cash: raw_net_cash,
        contract_liabilities,
        forward_eps,
        forward_revenue,
        current_revenue: income.current_revenue,
        prior_revenue: income.prior_revenue,
        current_ebit: income.current_ebit,
        normalized_ebit_margin: income.normalized_ebit_margin,
        current_gross_margin: income.current_gross_margin,
        prior_gross_margin: income.prior_gross_margin,
        current_ebit_margin: income.current_ebit_margin,
        current_fcf_margin,
        net_cash_to_revenue,
        forward_revenue_growth,
    }
}

fn ratio(numerator: Option<f64>, denominator: Option<f64>) -> Option<f64> {
    match (numerator, denominator) {
        (Some(numerator), Some(denominator))
            if numerator.is_finite() && denominator.is_finite() && denominator > 0.0 =>
        {
            Some(numerator / denominator)
        }
        _ => None,
    }
}

fn ratio_change(current: Option<f64>, prior: Option<f64>) -> Option<f64> {
    ratio(current, prior).map(|value| value - 1.0)
}

fn cashflow_windows(value: Option<&Value>) -> (Option<String>, Option<f64>, Option<f64>, Vec<f64>) {
    let Some(rows) = value.and_then(Value::as_array) else {
        return (None, None, None, Vec::new());
    };
    let values = rows
        .iter()
        .filter_map(|row| {
            let date = row.get("date")?.as_str()?.to_string();
            let fcf = row
                .get("freeCashFlow")
                .and_then(Value::as_f64)
                .or_else(|| {
                    Some(
                        row.get("operatingCashFlow")?.as_f64()?
                            + row.get("capitalExpenditure")?.as_f64()?,
                    )
                })?;
            fcf.is_finite().then_some((date, fcf))
        })
        .collect::<Vec<_>>();
    let current = (values.len() >= 4).then(|| values.iter().take(4).map(|(_, value)| value).sum());
    let prior =
        (values.len() >= 8).then(|| values.iter().skip(4).take(4).map(|(_, value)| value).sum());
    let annual_history = values
        .chunks(4)
        .filter(|chunk| chunk.len() == 4)
        .map(|chunk| chunk.iter().map(|(_, value)| value).sum::<f64>())
        .collect();
    (
        values.first().map(|(date, _)| date.clone()),
        current,
        prior,
        annual_history,
    )
}

fn income_windows(value: Option<&Value>) -> IncomeWindows {
    let Some(rows) = value.and_then(Value::as_array) else {
        return IncomeWindows::default();
    };
    let values = rows
        .iter()
        .filter_map(|row| {
            let date = row.get("date")?.as_str()?.to_string();
            let revenue = row.get("revenue")?.as_f64()?;
            let ebit = row
                .get("operatingIncome")
                .and_then(Value::as_f64)
                .or_else(|| row.get("ebit").and_then(Value::as_f64));
            let gross_profit = row.get("grossProfit").and_then(Value::as_f64);
            (revenue.is_finite() && revenue > 0.0).then_some((date, revenue, ebit, gross_profit))
        })
        .collect::<Vec<_>>();
    let annual = values
        .chunks(4)
        .filter(|chunk| chunk.len() == 4)
        .map(|chunk| {
            let revenue = chunk.iter().map(|(_, value, _, _)| value).sum::<f64>();
            let ebit = chunk
                .iter()
                .map(|(_, _, value, _)| *value)
                .collect::<Option<Vec<_>>>()
                .map(|values| values.into_iter().sum::<f64>());
            let gross_profit = chunk
                .iter()
                .map(|(_, _, _, value)| *value)
                .collect::<Option<Vec<_>>>()
                .map(|values| values.into_iter().sum::<f64>());
            (revenue, ebit, gross_profit)
        })
        .collect::<Vec<_>>();
    let current = annual.first().cloned();
    let prior = annual.get(1).cloned();
    let mut margins = annual
        .iter()
        .filter_map(|(revenue, ebit, _)| ratio(*ebit, Some(*revenue)))
        .filter(|margin| margin.is_finite() && *margin > 0.0)
        .collect::<Vec<_>>();
    margins.sort_by(|a, b| a.total_cmp(b));
    let normalized_ebit_margin = if margins.is_empty() {
        None
    } else {
        Some(margins[margins.len() / 2])
    };
    IncomeWindows {
        as_of: values.first().map(|(date, _, _, _)| date.clone()),
        current_revenue: current.as_ref().map(|(revenue, _, _)| *revenue),
        prior_revenue: prior.as_ref().map(|(revenue, _, _)| *revenue),
        current_ebit: current.as_ref().and_then(|(_, ebit, _)| *ebit),
        normalized_ebit_margin,
        current_gross_margin: current
            .as_ref()
            .and_then(|(revenue, _, gross)| ratio(*gross, Some(*revenue))),
        prior_gross_margin: prior
            .as_ref()
            .and_then(|(revenue, _, gross)| ratio(*gross, Some(*revenue))),
        current_ebit_margin: current
            .as_ref()
            .and_then(|(revenue, ebit, _)| ratio(*ebit, Some(*revenue))),
    }
}

fn balance_inputs(value: Option<&Value>) -> (Option<f64>, Option<f64>) {
    let Some(row) = value
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
    else {
        return (None, None);
    };
    let Some(cash) = row
        .get("cashAndShortTermInvestments")
        .and_then(Value::as_f64)
        .or_else(|| {
            Some(
                row.get("cashAndCashEquivalents")?.as_f64()?
                    + row
                        .get("shortTermInvestments")
                        .and_then(Value::as_f64)
                        .unwrap_or(0.0),
            )
        })
    else {
        return (None, None);
    };
    let Some(debt) = row.get("totalDebt").and_then(Value::as_f64).or_else(|| {
        Some(
            row.get("shortTermDebt")
                .and_then(Value::as_f64)
                .unwrap_or(0.0)
                + row
                    .get("longTermDebt")
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0),
        )
    }) else {
        return (None, None);
    };
    let contract_liabilities = row
        .get("deferredRevenue")
        .and_then(Value::as_f64)
        .or_else(|| row.get("contractLiabilities").and_then(Value::as_f64))
        .or_else(|| {
            let current = row
                .get("deferredRevenueCurrent")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            let non_current = row
                .get("deferredRevenueNonCurrent")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            ((current + non_current) > 0.0).then_some(current + non_current)
        })
        .filter(|value| value.is_finite() && *value > 0.0);
    (Some(cash - debt), contract_liabilities)
}

fn estimates_from_value(
    value: Option<&Value>,
    report_date: NaiveDate,
) -> (Option<f64>, Option<f64>) {
    let Some(rows) = value.and_then(Value::as_array) else {
        return (None, None);
    };
    let mut candidates = rows
        .iter()
        .filter_map(|row| {
            let date = NaiveDate::parse_from_str(row.get("date")?.as_str()?, "%Y-%m-%d").ok()?;
            let eps = row.get("estimatedEpsAvg").and_then(Value::as_f64);
            let revenue = row.get("estimatedRevenueAvg").and_then(Value::as_f64);
            (date > report_date).then_some((date, eps, revenue))
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(date, _, _)| *date);
    candidates
        .iter()
        .find(|(date, _, _)| (*date - report_date).num_days() >= 180)
        .or_else(|| candidates.first())
        .map(|(_, eps, revenue)| {
            (
                (*eps).filter(|value| value.is_finite() && *value > 0.0),
                (*revenue).filter(|value| value.is_finite() && *value > 0.0),
            )
        })
        .unwrap_or((None, None))
}

fn snapshot_path(state: &AppState) -> PathBuf {
    crate::routes::research_store::data_root(state)
        .join("valuation_lab")
        .join("daily.json")
}

fn rating_valuation_path(state: &AppState) -> PathBuf {
    crate::routes::research_store::data_root(state)
        .join("company_ratings")
        .join("valuations")
        .join("latest.json")
}

fn rating_fundamental_path(state: &AppState) -> PathBuf {
    crate::routes::research_store::data_root(state)
        .join("company_ratings")
        .join("fundamentals")
        .join("latest.json")
}

async fn read_snapshot(state: &AppState) -> Option<ValuationLabSnapshot> {
    let bytes = tokio::fs::read(snapshot_path(state)).await.ok()?;
    serde_json::from_slice(&bytes).ok()
}

async fn write_snapshot(state: &AppState, snapshot: &ValuationLabSnapshot) -> Result<(), String> {
    atomic_write_json(&snapshot_path(state), snapshot).await
}

async fn write_rating_valuations(
    state: &AppState,
    snapshot: &ValuationLabSnapshot,
) -> Result<(), String> {
    let items = snapshot
        .items
        .iter()
        .filter(|item| item.eligible_for_rating && item.scenarios.len() == 3)
        .filter_map(|item| {
            let current_price = item.current_price?;
            let probability_weighted_value = item.probability_weighted_value?;
            let expected_upside_percent = item.expected_upside_percent?;
            let method_count = item.cross_check.as_ref().map(|check| check.method_count)?;
            Some(RatingValuationItem {
                symbol: item.symbol.clone(),
                as_of: snapshot.report_date.clone(),
                currency: item.currency.clone(),
                bear_case: item.scenarios[0].fair_value,
                base_case: item.scenarios[1].fair_value,
                bull_case: item.scenarios[2].fair_value,
                current_price,
                probability_weighted_value,
                expected_upside_percent,
                method_count,
                confidence: item.confidence.clone(),
                method: item.method.clone(),
                assumptions: item.assumptions.clone(),
                sources: item
                    .evidence
                    .iter()
                    .map(|evidence| {
                        format!(
                            "{} · {} · {}",
                            evidence.source, evidence.as_of, evidence.source_url
                        )
                    })
                    .collect(),
                review_status: "computed".to_string(),
            })
        })
        .collect();
    let output = RatingValuationFile {
        report_date: &snapshot.report_date,
        framework_version: "hone-valuation-v2",
        generated_at: snapshot.generated_at,
        items,
    };
    atomic_write_json(&rating_valuation_path(state), &output).await
}

async fn write_rating_fundamentals(
    state: &AppState,
    snapshot: &ValuationLabSnapshot,
) -> Result<(), String> {
    let items = snapshot
        .items
        .iter()
        .filter_map(|item| {
            let as_of = item.financial_as_of.clone()?;
            let has_metrics = item.revenue_growth_percent.is_some()
                || item.forward_revenue_growth_percent.is_some()
                || item.gross_margin_percent.is_some()
                || item.gross_margin_change_pp.is_some()
                || item.ebit_margin_percent.is_some()
                || item.fcf_margin_percent.is_some()
                || item.net_cash_to_revenue_percent.is_some();
            has_metrics.then(|| RatingFundamentalItem {
                symbol: item.symbol.clone(),
                as_of,
                revenue_growth_percent: item.revenue_growth_percent,
                forward_revenue_growth_percent: item.forward_revenue_growth_percent,
                gross_margin_percent: item.gross_margin_percent,
                gross_margin_change_pp: item.gross_margin_change_pp,
                ebit_margin_percent: item.ebit_margin_percent,
                fcf_margin_percent: item.fcf_margin_percent,
                net_cash_to_revenue_percent: item.net_cash_to_revenue_percent,
                sources: vec![
                    "FMP income-statement · https://financialmodelingprep.com/stable/income-statement".to_string(),
                    "FMP cash-flow-statement · https://financialmodelingprep.com/stable/cash-flow-statement".to_string(),
                    "FMP balance-sheet-statement · https://financialmodelingprep.com/stable/balance-sheet-statement".to_string(),
                    "FMP analyst-estimates · https://financialmodelingprep.com/stable/analyst-estimates".to_string(),
                ],
                review_status: "computed",
            })
        })
        .collect();
    let output = RatingFundamentalFile {
        report_date: &snapshot.report_date,
        framework_version: "hone-fundamentals-v1",
        generated_at: snapshot.generated_at,
        items,
    };
    atomic_write_json(&rating_fundamental_path(state), &output).await
}

async fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    crate::routes::research_store::write_json_atomic(path, value)
        .await
        .map_err(|error| error.to_string())
}

fn unavailable_snapshot() -> ValuationLabSnapshot {
    snapshot_from_inputs(
        parse_companies(),
        HashMap::new(),
        HashMap::new(),
        Utc::now(),
        "尚未生成估值快照。".to_string(),
    )
}

fn mark_stale_if_needed(mut snapshot: ValuationLabSnapshot) -> ValuationLabSnapshot {
    if Utc::now() - snapshot.generated_at > chrono::Duration::hours(STALE_AFTER_HOURS) {
        snapshot.status = "stale".to_string();
        snapshot.summary = "估值快照超过 36 小时，仅保留历史研究用途，不进入当日评级。".to_string();
        for item in &mut snapshot.items {
            item.eligible_for_rating = false;
            if item.status == "ready" {
                item.status = "stale".to_string();
            }
        }
        snapshot.coverage.eligible_for_rating = 0;
    }
    snapshot
}

fn next_refresh(now: DateTime<Utc>) -> DateTime<Utc> {
    crate::routes::research_store::next_local_refresh(now, REFRESH_HOUR, REFRESH_MINUTE)
}

fn stable_base_url(base_url: &str) -> String {
    let mut base = base_url.trim_end_matches('/').to_string();
    for suffix in ["/api/v3", "/api"] {
        if let Some(stripped) = base.strip_suffix(suffix) {
            base = stripped.to_string();
            break;
        }
    }
    base.trim_end_matches('/').to_string()
}

fn quote_base_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    base.strip_suffix("/v3").unwrap_or(base).to_string()
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}
fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dcf_is_monotonic_and_reverse_dcf_recovers_growth() {
        let low = dcf_per_share(5.0, 2.0, 0.02, 0.10, 0.03);
        let high = dcf_per_share(5.0, 2.0, 0.20, 0.10, 0.03);
        assert!(high > low);
        let price = dcf_per_share(5.0, 2.0, 0.12, 0.10, 0.03);
        let reverse = reverse_dcf(price, 5.0, 2.0, 0.10, 0.03);
        assert_eq!(reverse.status, "solved");
        assert!((reverse.implied_growth_rate.expect("growth") - 0.12).abs() < 0.001);
    }

    #[test]
    fn quarterly_cashflow_requires_two_complete_years() {
        let rows = (0..8)
            .map(|index| json!({"date": format!("2026-{:02}-01", 8 - index), "freeCashFlow": 10.0 + index as f64}))
            .collect::<Vec<_>>();
        let (date, current, prior, history) = cashflow_windows(Some(&Value::Array(rows)));
        assert_eq!(date.as_deref(), Some("2026-08-01"));
        assert_eq!(current, Some(46.0));
        assert_eq!(prior, Some(62.0));
        assert_eq!(history, vec![46.0, 62.0]);
        let short = json!([{"date":"2026-08-01","freeCashFlow":10.0}]);
        assert_eq!(cashflow_windows(Some(&short)).1, None);
    }

    #[test]
    fn negative_cashflow_never_becomes_a_price_target() {
        let company = ValuationCompany {
            name: "Test".to_string(),
            symbol: "TEST".to_string(),
            market_scope: "US".to_string(),
            theme: "test".to_string(),
            valuation_method: "DCF".to_string(),
        };
        let quote = QuoteInput {
            price: 20.0,
            shares: 100.0,
            as_of: Some("2026-08-11".to_string()),
        };
        let financial = FinancialInput {
            as_of: Some("2026-06-30".to_string()),
            current_fcf: Some(-10.0),
            prior_fcf: Some(8.0),
            annual_fcf_history: vec![-10.0, 8.0],
            net_cash: Some(2.0),
            contract_liabilities: None,
            forward_eps: Some(1.0),
            forward_revenue: None,
            current_revenue: None,
            prior_revenue: None,
            current_ebit: None,
            normalized_ebit_margin: None,
            ..FinancialInput::default()
        };
        let item = valuation_item(
            company,
            Some(&quote),
            Some(&financial),
            NaiveDate::from_ymd_opt(2026, 8, 11).unwrap(),
            "",
        );
        assert!(
            item.scenarios
                .iter()
                .all(|scenario| scenario.methods.len() < 2)
        );
        assert!(!item.eligible_for_rating);
    }

    #[test]
    fn stale_quote_fails_closed() {
        assert!(!date_is_fresh(
            Some("2026-08-01"),
            NaiveDate::from_ymd_opt(2026, 8, 11).unwrap(),
            4
        ));
        assert!(date_is_fresh(
            Some("2026-08-08"),
            NaiveDate::from_ymd_opt(2026, 8, 11).unwrap(),
            4
        ));
    }

    #[test]
    fn forward_eps_uses_nearest_future_year() {
        let value = json!([
            {"date":"2026-06-30","estimatedEpsAvg":2.0},
            {"date":"2028-12-31","estimatedEpsAvg":5.0,"estimatedRevenueAvg":500.0},
            {"date":"2027-12-31","estimatedEpsAvg":4.0,"estimatedRevenueAvg":400.0}
        ]);
        let estimates =
            estimates_from_value(Some(&value), NaiveDate::from_ymd_opt(2026, 8, 11).unwrap());
        assert_eq!(estimates, (Some(4.0), Some(400.0)));
    }

    #[test]
    fn forward_estimate_skips_an_almost_finished_fiscal_year() {
        let value = json!([
            {"date":"2026-12-31","estimatedEpsAvg":3.0,"estimatedRevenueAvg":300.0},
            {"date":"2027-12-31","estimatedEpsAvg":4.0,"estimatedRevenueAvg":400.0}
        ]);
        let estimates =
            estimates_from_value(Some(&value), NaiveDate::from_ymd_opt(2026, 8, 11).unwrap());
        assert_eq!(estimates, (Some(4.0), Some(400.0)));
    }

    #[test]
    fn company_method_hint_selects_the_valuation_profile() {
        let storage = ValuationCompany {
            name: "SanDisk".to_string(),
            symbol: "SNDK".to_string(),
            market_scope: "US".to_string(),
            theme: "存储/企业级 SSD".to_string(),
            valuation_method: "中周期 forward P/E 与 EV/EBIT".to_string(),
        };
        let transition = ValuationCompany {
            name: "Rocket".to_string(),
            symbol: "RKLB".to_string(),
            market_scope: "US".to_string(),
            theme: "航天/能源材料".to_string(),
            valuation_method: "分部 EV/S 与订单质量".to_string(),
        };
        assert_eq!(
            profile_for_company(&storage),
            ValuationProfile::CyclicalManufacturing
        );
        assert_eq!(
            profile_for_company(&transition),
            ValuationProfile::RevenueTransition
        );
    }

    #[test]
    fn cyclical_profile_uses_report_style_three_method_weights() {
        let quote = QuoteInput {
            price: 120.0,
            shares: 100.0,
            as_of: Some("2026-08-11".to_string()),
        };
        let financial = FinancialInput {
            as_of: Some("2026-06-30".to_string()),
            current_fcf: Some(1_200.0),
            prior_fcf: Some(700.0),
            annual_fcf_history: vec![1_200.0, 700.0, 500.0, 800.0, 600.0],
            net_cash: Some(320.0),
            contract_liabilities: Some(20.0),
            forward_eps: Some(12.0),
            forward_revenue: Some(3_400.0),
            current_revenue: Some(3_000.0),
            prior_revenue: Some(2_500.0),
            current_ebit: Some(1_500.0),
            normalized_ebit_margin: Some(0.40),
            ..FinancialInput::default()
        };
        let scenario = build_scenario(
            ValuationProfile::CyclicalManufacturing,
            scenario_configs(ValuationProfile::CyclicalManufacturing)[1],
            &quote,
            &financial,
            3.0,
            normalized_positive_cashflow(&financial.annual_fcf_history),
            Some(0.20),
        );
        assert_eq!(scenario.methods.len(), 3);
        assert_eq!(scenario.methods[0].id, "forward_pe");
        assert_eq!(scenario.methods[0].weight, 0.60);
        assert_eq!(scenario.methods[1].id, "ev_ebit");
        assert_eq!(scenario.methods[1].weight, 0.25);
        assert_eq!(scenario.methods[2].id, "cycle_adjusted_dcf");
        assert_eq!(scenario.methods[2].weight, 0.15);
    }

    #[test]
    fn cyclical_dcf_reverts_peak_cashflow_toward_midcycle() {
        let path = cycle_adjusted_fcf_path(120.0, 60.0, "base");
        assert!(path[0] > path[4]);
        assert_eq!(round1(path[0]), 114.0);
        assert_eq!(round1(path[4]), 60.0);
    }

    #[test]
    fn balance_contract_liability_is_separate_from_raw_net_cash() {
        let balance = json!([{
            "cashAndShortTermInvestments": 100.0,
            "totalDebt": 20.0,
            "deferredRevenue": 15.0
        }]);
        assert_eq!(balance_inputs(Some(&balance)), (Some(80.0), Some(15.0)));
    }

    #[test]
    fn same_day_cyclical_item_is_multi_method_and_rating_eligible() {
        let company = ValuationCompany {
            name: "SanDisk".to_string(),
            symbol: "SNDK".to_string(),
            market_scope: "US".to_string(),
            theme: "存储/企业级 SSD".to_string(),
            valuation_method: "中周期 forward P/E 与 EV/EBIT".to_string(),
        };
        let quote = QuoteInput {
            price: 120.0,
            shares: 100.0,
            as_of: Some("2026-08-11".to_string()),
        };
        let financial = FinancialInput {
            as_of: Some("2026-06-30".to_string()),
            current_fcf: Some(1_200.0),
            prior_fcf: Some(700.0),
            annual_fcf_history: vec![1_200.0, 700.0, 500.0, 800.0, 600.0],
            net_cash: Some(320.0),
            contract_liabilities: Some(20.0),
            forward_eps: Some(12.0),
            forward_revenue: Some(3_400.0),
            current_revenue: Some(3_000.0),
            prior_revenue: Some(2_500.0),
            current_ebit: Some(1_500.0),
            normalized_ebit_margin: Some(0.40),
            ..FinancialInput::default()
        };
        let item = valuation_item(
            company,
            Some(&quote),
            Some(&financial),
            NaiveDate::from_ymd_opt(2026, 8, 11).unwrap(),
            "",
        );
        assert!(item.eligible_for_rating);
        assert_eq!(item.valuation_profile, "cyclical_manufacturing");
        assert_eq!(item.scenarios.len(), 3);
        assert_eq!(item.cross_check.as_ref().unwrap().method_count, 3);
        assert!(item.probability_weighted_value.is_some());
        assert!(item.expected_upside_percent.is_some());
    }

    #[test]
    fn profitable_growth_item_uses_three_independent_methods() {
        let company = ValuationCompany {
            name: "Microsoft".to_string(),
            symbol: "MSFT".to_string(),
            market_scope: "US".to_string(),
            theme: "云与企业软件".to_string(),
            valuation_method: "forward P/E、EV/EBIT 与 DCF 交叉验证".to_string(),
        };
        let quote = QuoteInput {
            price: 500.0,
            shares: 7_400.0,
            as_of: Some("2026-08-11".to_string()),
        };
        let financial = FinancialInput {
            as_of: Some("2026-06-30".to_string()),
            current_fcf: Some(150_000.0),
            prior_fcf: Some(140_000.0),
            annual_fcf_history: vec![150_000.0, 140_000.0, 132_000.0],
            net_cash: Some(40_000.0),
            contract_liabilities: Some(25_000.0),
            forward_eps: Some(18.0),
            forward_revenue: Some(320_000.0),
            current_revenue: Some(280_000.0),
            prior_revenue: Some(250_000.0),
            current_ebit: Some(125_000.0),
            normalized_ebit_margin: Some(0.42),
            ..FinancialInput::default()
        };
        let item = valuation_item(
            company,
            Some(&quote),
            Some(&financial),
            NaiveDate::from_ymd_opt(2026, 8, 11).unwrap(),
            "",
        );
        let base = &item.scenarios[1];
        let method_ids = base
            .methods
            .iter()
            .map(|method| method.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(item.valuation_profile, "profitable_growth");
        assert_eq!(
            method_ids,
            vec!["forward_pe", "ev_ebit", "cycle_adjusted_dcf"]
        );
        assert_eq!(base.methods[0].weight, 0.40);
        assert_eq!(base.methods[1].weight, 0.20);
        assert_eq!(base.methods[2].weight, 0.40);
        assert!(item.eligible_for_rating);
    }
}
