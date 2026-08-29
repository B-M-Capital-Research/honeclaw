//! Daily transcript-informed company ratings for the public chat workspace.
//!
//! The durable snapshot is refreshed at 19:30 Asia/Shanghai. FMP enriches the
//! research baseline when configured; missing upstream data never becomes a
//! zero and is surfaced through `data_status`, coverage, and confidence.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use futures::{StreamExt, stream};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};

use super::investment_decisions::FinancialSourceClaimTrace;
use crate::routes::public_finance_calendar::fetch_fmp_json_once;
use crate::state::AppState;

const CARDS_JSON: &str =
    include_str!("../../../../skills/company-thesis-ratings/references/company-cards.json");
const REFRESH_HOUR: u32 = 19;
const REFRESH_MINUTE: u32 = 30;
const STALE_AFTER_HOURS: i64 = 36;
const METHODOLOGY_VERSION: &str = "hone-company-rating-v9-analyst-consensus-context";
const MARKET_HISTORY_POLICY_VERSION: &str = "hone-market-history-v1-nasdaq-daily-close";
const MARKET_HISTORY_LOOKBACK_DAYS: i64 = 430;
const MARKET_HISTORY_MAX_AGE_DAYS: i64 = 7;
const MARKET_HISTORY_SPLIT_REVIEW_THRESHOLD_PERCENT: f64 = 45.0;
pub(crate) const SHORT_INTEREST_POLICY_VERSION: &str = "hone-short-interest-v1-nasdaq-settlement";
const SHORT_INTEREST_MAX_AGE_DAYS: i64 = 45;
pub(crate) const OPTIONS_POSITIONING_POLICY_VERSION: &str =
    "hone-options-positioning-v1-nasdaq-monthly-open-interest";
const OPTIONS_POSITIONING_MAX_AGE_DAYS: i64 = 5;
pub(crate) const NEWS_ATTENTION_POLICY_VERSION: &str =
    "hone-news-attention-v1-nasdaq-syndicated-14d";
const NEWS_ATTENTION_RESULT_LIMIT: usize = 100;
const NEWS_ATTENTION_WINDOW_DAYS: i64 = 14;
const NEWS_ATTENTION_RECENT_DAYS: i64 = 3;
pub(crate) const INSTITUTIONAL_HOLDINGS_POLICY_VERSION: &str =
    "hone-institutional-holdings-v1-nasdaq-13f-observation";
const INSTITUTIONAL_HOLDINGS_RESULT_LIMIT: usize = 50;
pub(crate) const INSTITUTIONAL_HOLDINGS_MAX_OBSERVATION_AGE_DAYS: i64 = 1;
pub(crate) const ANALYST_CONSENSUS_POLICY_VERSION: &str =
    "hone-analyst-consensus-v1-nasdaq-observation";
pub(crate) const ANALYST_CONSENSUS_MAX_OBSERVATION_AGE_DAYS: i64 = 1;
static COMPANY_RATING_REFRESH_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Debug, Clone, Deserialize)]
struct CardFile {
    companies: Vec<CompanyCard>,
}

#[derive(Debug, Clone, Deserialize)]
struct CompanyCard {
    name: String,
    symbol: String,
    market_scope: String,
    theme: String,
    value_chain: String,
    business_model: String,
    moat: String,
    thesis_summary: String,
    valuation_method: String,
    dimensions_1_to_5: StaticDimensions,
    confidence: String,
    watch_items: Vec<String>,
    risks: Vec<String>,
    falsifiers: Vec<String>,
    source_updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
struct StaticDimensions {
    scarcity: f64,
    pricing_quality: f64,
    visibility: f64,
    execution: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct RatingDimensions {
    pub moat: f64,
    pub scarcity: f64,
    pub fundamentals: f64,
    pub visibility: f64,
    #[serde(default)]
    pub growth_quality: Option<f64>,
    #[serde(default)]
    pub pricing_power: Option<f64>,
    #[serde(default)]
    pub financial_quality: Option<f64>,
    #[serde(default)]
    pub valuation: Option<f64>,
    pub market_confirmation: f64,
    #[serde(default)]
    pub timing: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct RatingMetrics {
    pub revenue_growth_percent: Option<f64>,
    pub forward_revenue_growth_percent: Option<f64>,
    pub gross_margin_percent: Option<f64>,
    pub gross_margin_change_pp: Option<f64>,
    pub ebit_margin_percent: Option<f64>,
    pub fcf_margin_percent: Option<f64>,
    pub net_cash_to_revenue_percent: Option<f64>,
    pub accounts_receivable_growth_percent: Option<f64>,
    pub accounts_payable_growth_percent: Option<f64>,
    pub inventory_growth_percent: Option<f64>,
    pub property_plant_equipment_growth_percent: Option<f64>,
    pub operating_cash_flow_growth_percent: Option<f64>,
    pub capital_expenditure_growth_percent: Option<f64>,
    pub free_cash_flow_growth_percent: Option<f64>,
    pub financial_as_of: Option<String>,
    #[serde(default)]
    pub financial_review_status: Option<String>,
    #[serde(default)]
    pub financial_score_eligible: bool,
    #[serde(default)]
    pub financial_source_claim_ids: Vec<String>,
    #[serde(default)]
    pub financial_source_urls: Vec<String>,
    #[serde(default)]
    pub financial_calculations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub financial_source_claims: Vec<FinancialSourceClaimTrace>,
    #[serde(default)]
    pub financial_quality_warnings: Vec<String>,
    pub forward_metric_label: Option<String>,
    pub forward_metric_value: Option<String>,
    pub forward_metric_growth_percent: Option<f64>,
    pub forward_metric_as_of: Option<String>,
    pub forward_metric_source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DailyValuation {
    pub as_of: String,
    pub generated_at_beijing: String,
    pub currency: String,
    pub bear_case: f64,
    pub base_case: f64,
    pub bull_case: f64,
    pub current_price: f64,
    #[serde(default)]
    pub probability_weighted_value: f64,
    #[serde(default)]
    pub expected_upside_percent: f64,
    #[serde(default)]
    pub method_count: usize,
    #[serde(default)]
    pub confidence: String,
    pub current_position: String,
    pub position_percent: f64,
    pub method: String,
    pub assumptions: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DailyValuationFile {
    report_date: String,
    framework_version: String,
    generated_at: DateTime<Utc>,
    items: Vec<DailyValuationInput>,
}

#[derive(Debug, Clone, Deserialize)]
struct DailyValuationInput {
    symbol: String,
    as_of: String,
    currency: String,
    bear_case: f64,
    base_case: f64,
    bull_case: f64,
    current_price: f64,
    #[serde(default)]
    probability_weighted_value: Option<f64>,
    #[serde(default)]
    expected_upside_percent: Option<f64>,
    #[serde(default)]
    method_count: usize,
    #[serde(default)]
    confidence: String,
    method: String,
    assumptions: Vec<String>,
    sources: Vec<String>,
    review_status: String,
    #[serde(default)]
    input_mode: String,
    #[serde(default)]
    valuation_review_id: Option<String>,
    #[serde(default)]
    valuation_input_fingerprint_sha256: Option<String>,
    #[serde(default)]
    valuation_financial_evidence_fingerprint_sha256: Option<String>,
    #[serde(default)]
    valuation_input_as_of: Option<String>,
}

#[derive(Debug, Clone)]
struct ValuationAuthorizationBinding {
    review_id: String,
    input_fingerprint_sha256: String,
    financial_evidence_fingerprint_sha256: String,
    input_as_of: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CompanyRating {
    pub name: String,
    pub symbol: String,
    pub market_scope: String,
    pub theme: String,
    pub value_chain: String,
    pub score: f64,
    pub light: String,
    pub confidence: String,
    pub data_status: String,
    pub price: Option<f64>,
    pub change_percent: Option<f64>,
    #[serde(default)]
    pub price_avg50: Option<f64>,
    #[serde(default)]
    pub price_avg200: Option<f64>,
    #[serde(default)]
    pub year_low: Option<f64>,
    #[serde(default)]
    pub year_high: Option<f64>,
    #[serde(default)]
    pub market_history: Option<MarketHistorySummary>,
    #[serde(default)]
    pub short_interest: Option<ShortInterestSummary>,
    #[serde(default)]
    pub options_positioning: Option<OptionsPositioningSummary>,
    #[serde(default)]
    pub news_attention: Option<NewsAttentionSummary>,
    #[serde(default)]
    pub institutional_holdings: Option<InstitutionalHoldingsSummary>,
    #[serde(default)]
    pub analyst_consensus: Option<AnalystConsensusSummary>,
    pub market_as_of: Option<String>,
    pub financial_as_of: Option<String>,
    pub thesis_summary: String,
    pub business_model: String,
    pub moat: String,
    pub valuation_method: String,
    #[serde(default)]
    pub valuation: Option<DailyValuation>,
    #[serde(default)]
    pub valuation_unavailable_reason: String,
    pub dimensions: RatingDimensions,
    #[serde(default)]
    pub metrics: RatingMetrics,
    #[serde(default)]
    pub score_cap_reason: String,
    #[serde(default)]
    pub factor_coverage: usize,
    pub watch_items: Vec<String>,
    pub risks: Vec<String>,
    pub falsifiers: Vec<String>,
    pub research_updated_at: String,
    pub data_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MarketHistorySummary {
    pub policy_version: String,
    pub as_of: String,
    pub source: String,
    pub source_url: String,
    pub price_basis: String,
    pub session_count: usize,
    pub latest_close: f64,
    pub average_close_50: Option<f64>,
    pub average_close_200: Option<f64>,
    pub return_20_sessions_percent: Option<f64>,
    pub return_60_sessions_percent: Option<f64>,
    pub drawdown_from_60_session_high_percent: Option<f64>,
    pub recent_5_session_volume_vs_prior_55_percent: Option<f64>,
    pub quality_status: String,
    pub quality_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ShortInterestSummary {
    pub policy_version: String,
    pub as_of: String,
    pub source: String,
    pub source_url: String,
    pub current_shares_short: f64,
    pub previous_shares_short: f64,
    pub change_percent: f64,
    pub average_daily_share_volume: f64,
    pub days_to_cover: f64,
    pub observation_count: usize,
    pub quality_status: String,
    pub quality_warnings: Vec<String>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OptionsPositioningSummary {
    pub policy_version: String,
    pub as_of: String,
    pub source: String,
    pub source_url: String,
    pub expiration_date: String,
    pub days_to_expiration: i64,
    pub spot_price: f64,
    pub call_open_interest: f64,
    pub put_open_interest: f64,
    pub put_call_open_interest_ratio: Option<f64>,
    pub call_volume: f64,
    pub put_volume: f64,
    pub put_call_volume_ratio: Option<f64>,
    pub contract_rows: usize,
    pub quality_status: String,
    pub quality_warnings: Vec<String>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NewsAttentionSummary {
    pub policy_version: String,
    pub as_of: String,
    pub source: String,
    pub source_url: String,
    pub window_days: i64,
    pub recent_window_days: i64,
    pub recent_article_count: usize,
    pub prior_article_count: usize,
    pub recent_daily_rate: f64,
    pub prior_daily_rate: f64,
    pub activity_ratio: Option<f64>,
    pub unique_publishers: usize,
    pub observed_article_count: usize,
    pub oldest_observed_date: String,
    pub result_limit: usize,
    pub truncated_window: bool,
    pub quality_status: String,
    pub quality_warnings: Vec<String>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InstitutionalHoldingsSummary {
    pub policy_version: String,
    pub observed_on: String,
    pub source: String,
    pub source_url: String,
    pub institutional_ownership_percent: f64,
    pub institutional_holders: usize,
    pub total_shares_held: f64,
    pub total_reported_records: usize,
    pub top_sample_rows: usize,
    pub holder_table_truncated: bool,
    pub earliest_report_period: String,
    pub latest_report_period: String,
    pub report_period_count: usize,
    pub latest_period_rows_in_sample: usize,
    pub increased_positions_holders: usize,
    pub increased_positions_shares: f64,
    pub decreased_positions_holders: usize,
    pub decreased_positions_shares: f64,
    pub held_positions_holders: usize,
    pub held_positions_shares: f64,
    pub new_positions_holders: usize,
    pub new_positions_shares: f64,
    pub sold_out_positions_holders: usize,
    pub sold_out_positions_shares: f64,
    pub quality_status: String,
    pub quality_warnings: Vec<String>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AnalystConsensusSummary {
    pub policy_version: String,
    pub observed_on: String,
    pub source: String,
    pub source_url: String,
    pub buy_count: usize,
    pub hold_count: usize,
    pub sell_count: usize,
    pub recommendation_count: usize,
    pub buy_share_percent: f64,
    pub hold_share_percent: f64,
    pub sell_share_percent: f64,
    pub dominant_rating: String,
    pub dominant_count: usize,
    pub dominant_share_percent: f64,
    pub consensus_target_price: f64,
    pub low_target_price: f64,
    pub high_target_price: f64,
    pub target_range_width_percent: f64,
    pub historical_month_count: usize,
    pub quality_status: String,
    pub quality_warnings: Vec<String>,
    pub interpretation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct RatingCoverage {
    pub companies: usize,
    pub quotes: usize,
    /// Financial rows admitted into the dynamic rating factors.
    pub financials: usize,
    /// Point-in-time financial observations shown with provenance, including
    /// rows that remain review-only and therefore do not affect the score.
    #[serde(default)]
    pub financial_observations: usize,
    #[serde(default)]
    pub financials_review_required: usize,
    #[serde(default)]
    pub valuations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CompanyRatingSnapshot {
    pub generated_at: DateTime<Utc>,
    pub generated_at_beijing: String,
    pub next_refresh_at: DateTime<Utc>,
    pub timezone: String,
    pub data_status: String,
    pub methodology_version: String,
    #[serde(default)]
    pub simulation_note: String,
    pub coverage: RatingCoverage,
    pub disclaimer: String,
    pub items: Vec<CompanyRating>,
}

#[derive(Debug, Clone, Default)]
struct QuoteFact {
    price: f64,
    change_percent: Option<f64>,
    avg50: Option<f64>,
    avg200: Option<f64>,
    year_low: Option<f64>,
    year_high: Option<f64>,
    timestamp: Option<i64>,
    as_of: Option<String>,
    source: String,
    market_history: Option<MarketHistorySummary>,
    short_interest: Option<ShortInterestSummary>,
    options_positioning: Option<OptionsPositioningSummary>,
    news_attention: Option<NewsAttentionSummary>,
    institutional_holdings: Option<InstitutionalHoldingsSummary>,
    analyst_consensus: Option<AnalystConsensusSummary>,
}

#[derive(Debug, Clone)]
struct NasdaqDailyBar {
    date: NaiveDate,
    close: f64,
    volume: f64,
}

#[derive(Debug, Clone, Default)]
struct FinancialFact {
    as_of: Option<String>,
    revenue_growth_percent: Option<f64>,
    forward_revenue_growth_percent: Option<f64>,
    gross_margin_percent: Option<f64>,
    gross_margin_change_pp: Option<f64>,
    ebit_margin_percent: Option<f64>,
    fcf_margin_percent: Option<f64>,
    net_cash_to_revenue_percent: Option<f64>,
    accounts_receivable_growth_percent: Option<f64>,
    accounts_payable_growth_percent: Option<f64>,
    inventory_growth_percent: Option<f64>,
    property_plant_equipment_growth_percent: Option<f64>,
    operating_cash_flow_growth_percent: Option<f64>,
    capital_expenditure_growth_percent: Option<f64>,
    free_cash_flow_growth_percent: Option<f64>,
    score_eligible: bool,
    review_status: String,
    sources: Vec<String>,
    source_urls: Vec<String>,
    source_claim_ids: Vec<String>,
    source_calculations: Vec<String>,
    source_claims: Vec<FinancialSourceClaimTrace>,
    quality_warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct FundamentalFile {
    framework_version: String,
    generated_at: DateTime<Utc>,
    items: Vec<FundamentalInput>,
}

#[derive(Debug, Deserialize)]
struct FundamentalInput {
    symbol: String,
    as_of: String,
    revenue_growth_percent: Option<f64>,
    forward_revenue_growth_percent: Option<f64>,
    gross_margin_percent: Option<f64>,
    gross_margin_change_pp: Option<f64>,
    ebit_margin_percent: Option<f64>,
    fcf_margin_percent: Option<f64>,
    net_cash_to_revenue_percent: Option<f64>,
    #[serde(default)]
    accounts_receivable_growth_percent: Option<f64>,
    #[serde(default)]
    accounts_payable_growth_percent: Option<f64>,
    #[serde(default)]
    inventory_growth_percent: Option<f64>,
    #[serde(default)]
    property_plant_equipment_growth_percent: Option<f64>,
    #[serde(default)]
    operating_cash_flow_growth_percent: Option<f64>,
    #[serde(default)]
    capital_expenditure_growth_percent: Option<f64>,
    #[serde(default)]
    free_cash_flow_growth_percent: Option<f64>,
    sources: Vec<String>,
    review_status: String,
}

#[derive(Debug, Deserialize)]
struct ForwardEvidenceFile {
    framework_version: String,
    generated_at: DateTime<Utc>,
    items: Vec<ForwardEvidenceInput>,
}

#[derive(Debug, Deserialize)]
struct ForwardEvidenceInput {
    symbol: String,
    metric_kind: String,
    metric_label: String,
    value_display: String,
    growth_percent: Option<f64>,
    as_of: String,
    source_url: String,
    review_status: String,
}

#[derive(Debug, Clone)]
struct ForwardFact {
    metric_kind: String,
    metric_label: String,
    value_display: String,
    growth_percent: Option<f64>,
    as_of: String,
    source_url: String,
}

pub(crate) async fn handle_get_company_ratings(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) = crate::routes::public::require_public_user(&state, &headers) {
        return response;
    }

    let snapshot = match read_snapshot(&state).await {
        Some(snapshot) => mark_stale_if_needed(normalize_snapshot_contract(snapshot)),
        None => baseline_snapshot(),
    };
    Json(snapshot).into_response()
}

/// Start an immediate best-effort refresh, then wait for 19:30 Beijing each day.
pub(crate) async fn company_rating_worker(state: Arc<AppState>) {
    refresh_and_store(&state).await;
    refresh_position_management(&state).await;
    loop {
        let next = next_refresh(Utc::now());
        let wait = (next - Utc::now())
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(60));
        info!(next_refresh = %next, "company rating worker waiting");
        tokio::time::sleep(wait).await;
        refresh_and_store(&state).await;
        refresh_position_management(&state).await;
    }
}

async fn refresh_position_management(state: &AppState) {
    if let Err(error) = crate::routes::position_management::refresh_all(state).await {
        warn!(%error, "position management refresh after company ratings failed");
    }
}

pub(crate) async fn refresh_and_store(state: &AppState) {
    let requested_at = Utc::now();
    let _refresh_guard = COMPANY_RATING_REFRESH_LOCK.lock().await;
    if read_snapshot(state).await.is_some_and(|snapshot| {
        snapshot_satisfies_refresh_request(snapshot.generated_at, requested_at)
    }) {
        info!(%requested_at, "company rating concurrent refresh coalesced");
        return;
    }
    let fresh = generate_snapshot(state).await;
    let snapshot = if fresh.data_status == "transcript_only" {
        match read_snapshot(state).await {
            Some(prior)
                if prior.methodology_version == METHODOLOGY_VERSION
                    && matches!(prior.data_status.as_str(), "live" | "partial") =>
            {
                let mut prior = normalize_snapshot_contract(prior);
                prior.data_status = "stale".to_string();
                for item in &mut prior.items {
                    if item.data_status != "transcript_only" {
                        item.data_status = "stale".to_string();
                    }
                }
                prior.next_refresh_at = next_refresh(Utc::now());
                prior
            }
            _ => fresh,
        }
    } else {
        fresh
    };
    if let Err(error) = write_snapshot(state, &snapshot).await {
        warn!("company rating snapshot write failed: {error}");
    } else {
        crate::routes::investment_decisions::refresh_from_company_ratings(state, &snapshot).await;
        info!(
            status = %snapshot.data_status,
            quotes = snapshot.coverage.quotes,
            financials = snapshot.coverage.financials,
            "company rating snapshot refreshed"
        );
    }
}

fn snapshot_satisfies_refresh_request(
    snapshot_generated_at: DateTime<Utc>,
    requested_at: DateTime<Utc>,
) -> bool {
    snapshot_generated_at >= requested_at
}

async fn generate_snapshot(state: &AppState) -> CompanyRatingSnapshot {
    let cards = parse_cards();
    if simulation_preview_enabled(state) {
        let (financials, forward_evidence) = simulation_inputs(&cards);
        return snapshot_from_facts(
            cards,
            HashMap::new(),
            financials,
            HashMap::new(),
            forward_evidence,
            true,
        );
    }
    let valuations = read_verified_valuations(state).await;
    let mut financials = read_verified_fundamentals(state).await;
    let forward_evidence = read_verified_forward_evidence(state).await;
    let symbols = cards
        .iter()
        .map(|card| card.symbol.clone())
        .collect::<Vec<_>>();
    let sec_financials = read_sec_financial_evidence(state, &symbols).await;
    for (symbol, fact) in sec_financials {
        // A fresh, fully reviewed FMP bridge keeps precedence. SEC claim rows
        // are currently training-only pending human review, so they may fill a
        // missing observation but must never silently make an existing score
        // eligible or mix two review policies inside one factor.
        financials.entry(symbol).or_insert(fact);
    }
    let pool = state.core.config.fmp.effective_key_pool();
    let mut quotes = if pool.keys().is_empty() {
        HashMap::new()
    } else {
        fetch_quotes(state, pool.keys(), &symbols)
            .await
            .unwrap_or_else(|error| {
                warn!("company rating FMP quotes unavailable: {error}");
                HashMap::new()
            })
    };
    let missing = symbols
        .iter()
        .filter(|symbol| !quotes.contains_key(*symbol))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        let fallback = fetch_nasdaq_quotes(state, &missing).await;
        info!(
            requested = missing.len(),
            received = fallback.len(),
            "company rating Nasdaq quote fallback completed"
        );
        for (symbol, quote) in fallback {
            quotes.entry(symbol).or_insert(quote);
        }
    }
    let (
        histories,
        short_interests,
        options_positioning,
        news_attention,
        institutional_holdings,
        analyst_consensus,
    ) = tokio::join!(
        fetch_nasdaq_market_histories(state, &symbols),
        fetch_nasdaq_short_interests(state, &symbols),
        fetch_nasdaq_options_positioning(state, &symbols),
        fetch_nasdaq_news_attention(state, &symbols),
        fetch_nasdaq_institutional_holdings(state, &symbols),
        fetch_nasdaq_analyst_consensus(state, &symbols),
    );
    info!(
        requested = symbols.len(),
        received = histories.len(),
        "company rating Nasdaq market-history fallback completed"
    );
    for (symbol, history) in histories {
        let Some(quote) = quotes.get_mut(&symbol) else {
            continue;
        };
        if history.quality_status == "usable" {
            quote.avg50 = quote.avg50.or(history.average_close_50);
            quote.avg200 = quote.avg200.or(history.average_close_200);
        }
        quote.market_history = Some(history);
    }
    info!(
        requested = symbols.len(),
        received = short_interests.len(),
        "company rating Nasdaq short-interest context completed"
    );
    for (symbol, short_interest) in short_interests {
        let Some(quote) = quotes.get_mut(&symbol) else {
            continue;
        };
        quote.short_interest = Some(short_interest);
    }
    info!(
        requested = symbols.len(),
        received = options_positioning.len(),
        "company rating Nasdaq options-positioning context completed"
    );
    for (symbol, options) in options_positioning {
        let Some(quote) = quotes.get_mut(&symbol) else {
            continue;
        };
        quote.options_positioning = Some(options);
    }
    info!(
        requested = symbols.len(),
        received = news_attention.len(),
        "company rating Nasdaq syndicated-news attention completed"
    );
    for (symbol, attention) in news_attention {
        let Some(quote) = quotes.get_mut(&symbol) else {
            continue;
        };
        quote.news_attention = Some(attention);
    }
    info!(
        requested = symbols.len(),
        received = institutional_holdings.len(),
        "company rating Nasdaq institutional-holdings context completed"
    );
    for (symbol, holdings) in institutional_holdings {
        let Some(quote) = quotes.get_mut(&symbol) else {
            continue;
        };
        quote.institutional_holdings = Some(holdings);
    }
    info!(
        requested = symbols.len(),
        received = analyst_consensus.len(),
        "company rating Nasdaq analyst-consensus context completed"
    );
    for (symbol, consensus) in analyst_consensus {
        let Some(quote) = quotes.get_mut(&symbol) else {
            continue;
        };
        quote.analyst_consensus = Some(consensus);
    }
    snapshot_from_facts(
        cards,
        quotes,
        financials,
        valuations,
        forward_evidence,
        false,
    )
}

fn parse_cards() -> Vec<CompanyCard> {
    serde_json::from_str::<CardFile>(CARDS_JSON)
        .expect("company rating cards must be valid JSON")
        .companies
}

fn simulation_preview_enabled(state: &AppState) -> bool {
    if state.deployment_mode != "local"
        || !state
            .core
            .config
            .cloud
            .effective_mode()
            .as_str()
            .eq_ignore_ascii_case("local")
    {
        return false;
    }
    std::env::var("HONE_COMPANY_RATING_SIMULATION")
        .ok()
        .is_some_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
}

fn simulation_inputs(
    cards: &[CompanyCard],
) -> (HashMap<String, FinancialFact>, HashMap<String, ForwardFact>) {
    let as_of = Utc::now().with_timezone(&Shanghai).date_naive().to_string();
    let mut financials = HashMap::new();
    let mut forward = HashMap::new();
    for card in cards {
        let execution = card.dimensions_1_to_5.execution;
        let pricing = card.dimensions_1_to_5.pricing_quality;
        let visibility = card.dimensions_1_to_5.visibility;
        let growth =
            (execution * 6.0 + 1.0 + simulation_jitter(&card.symbol, 3) * 5.0).clamp(-12.0, 48.0);
        let forward_growth =
            (growth + (visibility - 3.0) * 2.5 + simulation_jitter(&card.symbol, 5) * 3.0)
                .clamp(-15.0, 55.0);
        let gross_margin = (simulation_theme_gross_margin(&card.theme)
            + (pricing - 3.0) * 3.5
            + simulation_jitter(&card.symbol, 7) * 3.0)
            .clamp(12.0, 82.0);
        let gross_change =
            ((pricing - 3.0) * 1.2 + simulation_jitter(&card.symbol, 11) * 1.8).clamp(-6.0, 6.0);
        let ebit_margin = (gross_margin * 0.43 - 7.0
            + (execution - 3.0) * 2.5
            + simulation_jitter(&card.symbol, 13) * 3.0)
            .clamp(-12.0, 42.0);
        let fcf_margin =
            (ebit_margin * 0.82 + simulation_jitter(&card.symbol, 17) * 4.0).clamp(-15.0, 38.0);
        let net_cash = ((execution - 3.0) * 14.0 + simulation_jitter(&card.symbol, 23) * 24.0)
            .clamp(-75.0, 85.0);
        financials.insert(
            card.symbol.clone(),
            FinancialFact {
                as_of: Some(format!("{as_of} · Codex 模拟")),
                revenue_growth_percent: Some(round1(growth)),
                forward_revenue_growth_percent: Some(round1(forward_growth)),
                gross_margin_percent: Some(round1(gross_margin)),
                gross_margin_change_pp: Some(round1(gross_change)),
                ebit_margin_percent: Some(round1(ebit_margin)),
                fcf_margin_percent: Some(round1(fcf_margin)),
                net_cash_to_revenue_percent: Some(round1(net_cash)),
                accounts_receivable_growth_percent: None,
                accounts_payable_growth_percent: None,
                inventory_growth_percent: None,
                property_plant_equipment_growth_percent: None,
                operating_cash_flow_growth_percent: None,
                capital_expenditure_growth_percent: None,
                free_cash_flow_growth_percent: None,
                score_eligible: true,
                review_status: "simulation".to_string(),
                sources: vec!["Codex 本地情景模拟（非真实数据）".to_string()],
                source_urls: Vec::new(),
                source_claim_ids: Vec::new(),
                source_calculations: Vec::new(),
                source_claims: Vec::new(),
                quality_warnings: Vec::new(),
            },
        );
        let (kind, label) = if card.theme.contains("AI平台") || card.theme.contains("软件") {
            ("rpo", "RPO / ARR 增速")
        } else if card.theme.contains("New Cloud") {
            ("backlog", "积压订单增速")
        } else {
            ("orders", "在手订单/指引增速")
        };
        let forward_metric_growth = ((visibility - 3.0) * 14.0
            + simulation_jitter(&card.symbol, 29) * 8.0)
            .clamp(-20.0, 58.0);
        forward.insert(
            card.symbol.clone(),
            ForwardFact {
                metric_kind: kind.to_string(),
                metric_label: label.to_string(),
                value_display: "Codex 模拟情景".to_string(),
                growth_percent: Some(round1(forward_metric_growth)),
                as_of: format!("{as_of} · 模拟"),
                source_url: "simulation://codex-local-preview".to_string(),
            },
        );
    }
    (financials, forward)
}

fn simulation_theme_gross_margin(theme: &str) -> f64 {
    if theme.contains("AI平台") || theme.contains("软件") || theme.contains("AI医疗") {
        62.0
    } else if theme.contains("算力芯片") || theme.contains("半导体") || theme.contains("存储")
    {
        48.0
    } else if theme.contains("光通信") {
        40.0
    } else if theme.contains("电力") || theme.contains("航天") {
        32.0
    } else {
        38.0
    }
}

fn simulation_jitter(symbol: &str, salt: u32) -> f64 {
    let hash = symbol
        .bytes()
        .fold(salt.wrapping_mul(16777619), |hash, byte| {
            hash.wrapping_mul(16777619) ^ u32::from(byte)
        });
    (f64::from(hash % 2001) / 1000.0) - 1.0
}

fn simulation_factor_score(symbol: &str, grade: f64, salt: u32) -> f64 {
    (structural_anchor(grade) + simulation_jitter(symbol, salt) * 12.0).clamp(15.0, 92.0)
}

fn snapshot_from_facts(
    cards: Vec<CompanyCard>,
    quotes: HashMap<String, QuoteFact>,
    financials: HashMap<String, FinancialFact>,
    valuations: HashMap<String, DailyValuation>,
    forward_evidence: HashMap<String, ForwardFact>,
    simulation: bool,
) -> CompanyRatingSnapshot {
    let quote_count = quotes.len();
    let financial_observation_count = financials.len();
    let financial_count = financials
        .values()
        .filter(|fact| fact.score_eligible)
        .count();
    let financial_review_required_count =
        financial_observation_count.saturating_sub(financial_count);
    let valuation_count = if simulation {
        cards.len()
    } else {
        valuations.len()
    };
    let company_count = cards.len();
    let data_status = if simulation {
        "simulation"
    } else {
        coverage_status(company_count, quote_count, financial_count, valuation_count)
    }
    .to_string();
    let now = Utc::now();
    let themes = cards
        .iter()
        .map(|card| (card.symbol.clone(), card.theme.clone()))
        .collect::<HashMap<_, _>>();
    let mut items = cards
        .into_iter()
        .map(|card| {
            let quote = quotes.get(&card.symbol);
            let financial = financials.get(&card.symbol);
            let valuation = valuations.get(&card.symbol);
            let forward = forward_evidence.get(&card.symbol);
            rating_from_card(
                card,
                quote,
                financial,
                valuation,
                forward,
                &financials,
                &themes,
                simulation,
            )
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    CompanyRatingSnapshot {
        generated_at: now,
        generated_at_beijing: now
            .with_timezone(&Shanghai)
            .format("%Y-%m-%d %H:%M")
            .to_string(),
        next_refresh_at: next_refresh(now),
        timezone: "Asia/Shanghai".to_string(),
        data_status,
        methodology_version: METHODOLOGY_VERSION.to_string(),
        simulation_note: if simulation {
            "Codex 本地模拟预览：以下为非真实数据，仅用于检查评分差异和页面效果，不代表真实行情、财报、订单或估值。"
                .to_string()
        } else {
            String::new()
        },
        coverage: RatingCoverage {
            companies: company_count,
            quotes: quote_count,
            financials: financial_count,
            financial_observations: financial_observation_count,
            financials_review_required: financial_review_required_count,
            valuations: valuation_count,
        },
        disclaimer: if simulation {
            "Codex 模拟预览，不构成真实评级或投资建议；接入真实数据后会整体替换。".to_string()
        } else {
            "研究排序工具，不构成买卖、仓位或收益承诺；缺失数据不会以零值替代。".to_string()
        },
        items,
    }
}

fn rating_from_card(
    card: CompanyCard,
    quote: Option<&QuoteFact>,
    financial: Option<&FinancialFact>,
    valuation: Option<&DailyValuation>,
    forward: Option<&ForwardFact>,
    financials: &HashMap<String, FinancialFact>,
    themes: &HashMap<String, String>,
    simulation: bool,
) -> CompanyRating {
    let financial_for_score = financial.filter(|fact| fact.score_eligible);
    let mut dimensions = RatingDimensions {
        moat: structural_anchor(card.dimensions_1_to_5.pricing_quality),
        scarcity: structural_anchor(card.dimensions_1_to_5.scarcity),
        fundamentals: structural_anchor(card.dimensions_1_to_5.execution),
        visibility: structural_anchor(card.dimensions_1_to_5.visibility),
        growth_quality: None,
        pricing_power: None,
        financial_quality: None,
        valuation: None,
        market_confirmation: 60.0,
        timing: None,
    };
    if let Some(fact) = financial_for_score {
        let peers = peer_facts(&card.theme, financials, themes);
        dimensions.growth_quality = growth_quality_score(fact, &peers);
        dimensions.pricing_power = pricing_power_score(fact, &peers);
        dimensions.financial_quality = financial_quality_score(fact, &peers);
        dimensions.fundamentals = average_present(&[
            dimensions.growth_quality,
            dimensions.pricing_power,
            dimensions.financial_quality,
        ])
        .unwrap_or(dimensions.fundamentals);
    }
    if let Some(fact) = quote {
        dimensions.market_confirmation = market_score(fact);
        dimensions.timing = Some(dimensions.market_confirmation);
    }
    if let Some(fact) = forward {
        dimensions.visibility = visibility_score(dimensions.visibility, fact.growth_percent);
    }

    let valuation = valuation
        .filter(|value| valuation_matches_quote(value, quote))
        .cloned();
    dimensions.valuation = valuation.as_ref().map(valuation_health_score);
    if simulation {
        dimensions.valuation = Some(simulation_factor_score(
            &card.symbol,
            card.dimensions_1_to_5.execution,
            7,
        ));
        dimensions.timing = Some(simulation_factor_score(
            &card.symbol,
            card.dimensions_1_to_5.visibility,
            19,
        ));
    }

    let raw_score = weighted_score(&dimensions);
    let (score, score_cap_reason) = apply_score_caps(raw_score, &dimensions);
    let factor_coverage = factor_coverage(&dimensions);
    let item_status = if simulation {
        "simulation"
    } else {
        match (
            quote.is_some(),
            financial_for_score.is_some(),
            valuation.is_some(),
            forward.is_some(),
        ) {
            (true, true, true, _) => "live",
            (false, false, false, false) => "transcript_only",
            _ => "partial",
        }
    };
    let confidence = effective_confidence(&card.confidence, item_status);
    let mut data_sources = vec!["内部演讲研究卡（压缩观点）".to_string()];
    if let Some(fact) = quote {
        data_sources.push(fact.source.clone());
        if let Some(history) = &fact.market_history {
            data_sources.push(format!(
                "{}（{}，{}）",
                history.source, history.as_of, history.source_url
            ));
        }
        if let Some(short_interest) = &fact.short_interest {
            data_sources.push(format!(
                "{}（{}，{}）",
                short_interest.source, short_interest.as_of, short_interest.source_url
            ));
        }
        if let Some(options) = &fact.options_positioning {
            data_sources.push(format!(
                "{}（{}，{}）",
                options.source, options.as_of, options.source_url
            ));
        }
        if let Some(attention) = &fact.news_attention {
            data_sources.push(format!(
                "{}（{}，{}）",
                attention.source, attention.as_of, attention.source_url
            ));
        }
        if let Some(holdings) = &fact.institutional_holdings {
            data_sources.push(format!(
                "{}（观察日 {}，{}）",
                holdings.source, holdings.observed_on, holdings.source_url
            ));
        }
        if let Some(consensus) = &fact.analyst_consensus {
            data_sources.push(format!(
                "{}（观察日 {}，{}）",
                consensus.source, consensus.observed_on, consensus.source_url
            ));
        }
    }
    if let Some(fact) = financial {
        data_sources.extend(fact.sources.clone());
        if !fact.score_eligible {
            data_sources.push("该财务证据仅供复核，当前不进入评分".to_string());
        }
    }
    if valuation.is_some() {
        data_sources.push("当日多方法三情景估值快照（来源、概率与方法已校验）".to_string());
    }
    if let Some(fact) = forward {
        data_sources.push(format!(
            "已核验前瞻指标：{}（{}，{}）",
            fact.metric_kind, fact.as_of, fact.source_url
        ));
    }
    if simulation {
        data_sources.push("Codex 本地情景模拟（非真实数据）".to_string());
    }

    CompanyRating {
        name: card.name,
        symbol: card.symbol,
        market_scope: card.market_scope,
        theme: card.theme,
        value_chain: card.value_chain,
        score: round1(score),
        light: if item_status == "transcript_only" {
            "unknown"
        } else {
            light_for_score(score)
        }
        .to_string(),
        confidence,
        data_status: item_status.to_string(),
        price: quote.map(|fact| round2(fact.price)),
        change_percent: quote.and_then(|fact| fact.change_percent).map(round2),
        price_avg50: quote.and_then(|fact| fact.avg50).map(round2),
        price_avg200: quote.and_then(|fact| fact.avg200).map(round2),
        year_low: quote.and_then(|fact| fact.year_low).map(round2),
        year_high: quote.and_then(|fact| fact.year_high).map(round2),
        market_history: quote.and_then(|fact| fact.market_history.clone()),
        short_interest: quote.and_then(|fact| fact.short_interest.clone()),
        options_positioning: quote.and_then(|fact| fact.options_positioning.clone()),
        news_attention: quote.and_then(|fact| fact.news_attention.clone()),
        institutional_holdings: quote.and_then(|fact| fact.institutional_holdings.clone()),
        analyst_consensus: quote.and_then(|fact| fact.analyst_consensus.clone()),
        market_as_of: quote.and_then(quote_as_of),
        financial_as_of: financial.and_then(|fact| fact.as_of.clone()),
        thesis_summary: card.thesis_summary,
        business_model: card.business_model,
        moat: card.moat,
        valuation_method: card.valuation_method,
        valuation_unavailable_reason: if simulation {
            "当前展示的是 Codex 模拟估值分，不生成虚构的目标价区间；真实估值仍等待行情与财务链路。"
                .to_string()
        } else {
            valuation_unavailable_reason(valuation.as_ref(), quote, financial_for_score)
        },
        valuation,
        dimensions,
        metrics: rating_metrics(financial, forward),
        score_cap_reason,
        factor_coverage,
        watch_items: card.watch_items,
        risks: card.risks,
        falsifiers: card.falsifiers,
        research_updated_at: card.source_updated_at,
        data_sources,
    }
}

/// Research cards use an ordinal five-grade judgment, not a percentage.
/// Reserve 100 for impossible certainty and keep enough distance between
/// grades for current evidence to move a dimension without erasing the base.
fn structural_anchor(grade: f64) -> f64 {
    match grade.round().clamp(1.0, 5.0) as u8 {
        1 => 25.0,
        2 => 45.0,
        3 => 60.0,
        4 => 75.0,
        _ => 90.0,
    }
}

fn valuation_unavailable_reason(
    valuation: Option<&DailyValuation>,
    quote: Option<&QuoteFact>,
    financial: Option<&FinancialFact>,
) -> String {
    if valuation.is_some() {
        return String::new();
    }
    match (quote.is_some(), financial.is_some()) {
        (false, false) => "本次未取得当日行情与财务输入，估值实验室无法生成可复算结果；没有沿用旧目标价，估值不参与综合分。",
        (false, true) => "本次缺少新鲜行情，无法确认当前价格在悲观/基准/乐观区间的位置；估值不参与综合分。",
        (true, false) => "本次缺少可用财务与一致预期输入，无法完成至少两种方法交叉验证；估值不参与综合分。",
        (true, true) => "本日估值的方法数量、离散度或来源门槛未通过复核；估值不参与综合分。",
    }
    .to_string()
}

fn weighted_score(d: &RatingDimensions) -> f64 {
    let factors = [
        (Some(d.moat), 0.20),
        (Some(d.scarcity), 0.10),
        (d.growth_quality, 0.15),
        (d.pricing_power, 0.10),
        (d.financial_quality, 0.15),
        (Some(d.visibility), 0.10),
        (d.valuation, 0.15),
        (d.timing, 0.05),
    ];
    let (weighted, weights) = factors
        .iter()
        .fold((0.0, 0.0), |(sum, weight), (score, w)| {
            score.map_or((sum, weight), |score| (sum + score * w, weight + w))
        });
    round1(weighted / weights.max(f64::EPSILON))
}

fn peer_facts<'a>(
    theme: &str,
    financials: &'a HashMap<String, FinancialFact>,
    themes: &HashMap<String, String>,
) -> Vec<&'a FinancialFact> {
    let themed = financials
        .iter()
        .filter(|(_, fact)| fact.score_eligible)
        .filter(|(symbol, _)| themes.get(*symbol).is_some_and(|value| value == theme))
        .map(|(_, fact)| fact)
        .collect::<Vec<_>>();
    if themed.len() >= 5 {
        themed
    } else {
        financials
            .values()
            .filter(|fact| fact.score_eligible)
            .collect()
    }
}

fn percentile(
    value: f64,
    peers: &[&FinancialFact],
    metric: fn(&FinancialFact) -> Option<f64>,
) -> f64 {
    let values = peers
        .iter()
        .filter_map(|fact| metric(fact))
        .filter(|value| value.is_finite())
        .collect::<Vec<_>>();
    if values.len() < 3 {
        return 50.0;
    }
    let below = values
        .iter()
        .filter(|candidate| **candidate < value)
        .count() as f64;
    let equal = values
        .iter()
        .filter(|candidate| (**candidate - value).abs() < 1e-9)
        .count() as f64;
    ((below + equal * 0.5) / values.len() as f64 * 100.0).clamp(5.0, 95.0)
}

fn anchor(value: f64, points: &[(f64, f64)]) -> f64 {
    if value <= points[0].0 {
        return points[0].1;
    }
    for pair in points.windows(2) {
        let (x0, y0) = pair[0];
        let (x1, y1) = pair[1];
        if value <= x1 {
            let ratio = (value - x0) / (x1 - x0);
            return y0 + ratio * (y1 - y0);
        }
    }
    points.last().map(|(_, score)| *score).unwrap_or(50.0)
}

fn blend_absolute_peer(
    value: Option<f64>,
    peers: &[&FinancialFact],
    metric: fn(&FinancialFact) -> Option<f64>,
    anchors: &[(f64, f64)],
) -> Option<f64> {
    value.map(|value| {
        (anchor(value, anchors) * 0.6 + percentile(value, peers, metric) * 0.4).clamp(5.0, 95.0)
    })
}

fn average_present(values: &[Option<f64>]) -> Option<f64> {
    let present = values.iter().flatten().copied().collect::<Vec<_>>();
    (!present.is_empty()).then(|| present.iter().sum::<f64>() / present.len() as f64)
}

fn visibility_score(structural: f64, growth_percent: Option<f64>) -> f64 {
    let Some(growth) = growth_percent else {
        return structural;
    };
    let dynamic = anchor(
        growth,
        &[
            (-20.0, 10.0),
            (0.0, 45.0),
            (15.0, 65.0),
            (30.0, 80.0),
            (60.0, 95.0),
        ],
    );
    (structural * 0.45 + dynamic * 0.55).clamp(5.0, 95.0)
}

fn growth_quality_score(fact: &FinancialFact, peers: &[&FinancialFact]) -> Option<f64> {
    average_present(&[
        blend_absolute_peer(
            fact.revenue_growth_percent,
            peers,
            |fact| fact.revenue_growth_percent,
            &[
                (-20.0, 10.0),
                (0.0, 40.0),
                (10.0, 60.0),
                (25.0, 80.0),
                (50.0, 95.0),
            ],
        ),
        blend_absolute_peer(
            fact.forward_revenue_growth_percent,
            peers,
            |fact| fact.forward_revenue_growth_percent,
            &[
                (-15.0, 10.0),
                (0.0, 40.0),
                (10.0, 60.0),
                (25.0, 82.0),
                (50.0, 95.0),
            ],
        ),
    ])
}

fn pricing_power_score(fact: &FinancialFact, peers: &[&FinancialFact]) -> Option<f64> {
    average_present(&[
        blend_absolute_peer(
            fact.gross_margin_percent,
            peers,
            |fact| fact.gross_margin_percent,
            &[(10.0, 15.0), (30.0, 45.0), (50.0, 70.0), (70.0, 90.0)],
        ),
        fact.gross_margin_change_pp.map(|change| {
            anchor(
                change,
                &[
                    (-8.0, 10.0),
                    (-2.0, 35.0),
                    (0.0, 55.0),
                    (2.0, 75.0),
                    (6.0, 95.0),
                ],
            )
        }),
    ])
}

fn financial_quality_score(fact: &FinancialFact, peers: &[&FinancialFact]) -> Option<f64> {
    average_present(&[
        blend_absolute_peer(
            fact.fcf_margin_percent,
            peers,
            |fact| fact.fcf_margin_percent,
            &[
                (-10.0, 5.0),
                (0.0, 35.0),
                (10.0, 60.0),
                (25.0, 82.0),
                (40.0, 95.0),
            ],
        ),
        blend_absolute_peer(
            fact.ebit_margin_percent,
            peers,
            |fact| fact.ebit_margin_percent,
            &[
                (-10.0, 5.0),
                (0.0, 35.0),
                (10.0, 58.0),
                (25.0, 80.0),
                (40.0, 95.0),
            ],
        ),
        fact.net_cash_to_revenue_percent.map(|value| {
            anchor(
                value,
                &[
                    (-100.0, 5.0),
                    (-30.0, 25.0),
                    (0.0, 55.0),
                    (30.0, 75.0),
                    (80.0, 95.0),
                ],
            )
        }),
    ])
}

fn factor_coverage(d: &RatingDimensions) -> usize {
    3 + [
        d.growth_quality,
        d.pricing_power,
        d.financial_quality,
        d.valuation,
        d.timing,
    ]
    .iter()
    .filter(|score| score.is_some())
    .count()
}

fn apply_score_caps(score: f64, d: &RatingDimensions) -> (f64, String) {
    let mut capped = score;
    let mut reasons = Vec::new();
    if d.moat < 45.0 || d.scarcity < 45.0 {
        capped = capped.min(74.9);
        reasons.push("护城河或产品地位偏弱，最高不超过黄灯");
    }
    if d.financial_quality.is_some_and(|value| value < 30.0) {
        capped = capped.min(54.9);
        reasons.push("现金流、盈利或债务质量进入高风险区，最高不超过红灯上沿");
    }
    if d.valuation.is_some_and(|value| value < 20.0) {
        capped = capped.min(74.9);
        reasons.push("估值隐含预期过高，最高不超过黄灯");
    }
    (round1(capped), reasons.join("；"))
}

fn rating_metrics(fact: Option<&FinancialFact>, forward: Option<&ForwardFact>) -> RatingMetrics {
    RatingMetrics {
        revenue_growth_percent: fact.and_then(|value| value.revenue_growth_percent),
        forward_revenue_growth_percent: fact.and_then(|value| value.forward_revenue_growth_percent),
        gross_margin_percent: fact.and_then(|value| value.gross_margin_percent),
        gross_margin_change_pp: fact.and_then(|value| value.gross_margin_change_pp),
        ebit_margin_percent: fact.and_then(|value| value.ebit_margin_percent),
        fcf_margin_percent: fact.and_then(|value| value.fcf_margin_percent),
        net_cash_to_revenue_percent: fact.and_then(|value| value.net_cash_to_revenue_percent),
        accounts_receivable_growth_percent: fact
            .and_then(|value| value.accounts_receivable_growth_percent),
        accounts_payable_growth_percent: fact
            .and_then(|value| value.accounts_payable_growth_percent),
        inventory_growth_percent: fact.and_then(|value| value.inventory_growth_percent),
        property_plant_equipment_growth_percent: fact
            .and_then(|value| value.property_plant_equipment_growth_percent),
        operating_cash_flow_growth_percent: fact
            .and_then(|value| value.operating_cash_flow_growth_percent),
        capital_expenditure_growth_percent: fact
            .and_then(|value| value.capital_expenditure_growth_percent),
        free_cash_flow_growth_percent: fact.and_then(|value| value.free_cash_flow_growth_percent),
        financial_as_of: fact.and_then(|value| value.as_of.clone()),
        financial_review_status: fact.map(|value| value.review_status.clone()),
        financial_score_eligible: fact.is_some_and(|value| value.score_eligible),
        financial_source_claim_ids: fact
            .map(|value| value.source_claim_ids.clone())
            .unwrap_or_default(),
        financial_source_urls: fact
            .map(|value| value.source_urls.clone())
            .unwrap_or_default(),
        financial_calculations: fact
            .map(|value| value.source_calculations.clone())
            .unwrap_or_default(),
        financial_source_claims: fact
            .map(|value| value.source_claims.clone())
            .unwrap_or_default(),
        financial_quality_warnings: fact
            .map(|value| value.quality_warnings.clone())
            .unwrap_or_default(),
        forward_metric_label: forward.map(|value| value.metric_label.clone()),
        forward_metric_value: forward.map(|value| value.value_display.clone()),
        forward_metric_growth_percent: forward.and_then(|value| value.growth_percent),
        forward_metric_as_of: forward.map(|value| value.as_of.clone()),
        forward_metric_source_url: forward.map(|value| value.source_url.clone()),
    }
}

fn market_score(fact: &QuoteFact) -> f64 {
    match (fact.avg50, fact.avg200) {
        (Some(avg50), Some(avg200)) if fact.price >= avg50 && avg50 >= avg200 => 90.0,
        (Some(avg50), Some(avg200)) if fact.price >= avg200 && fact.price < avg50 => 70.0,
        (Some(avg50), Some(avg200)) if fact.price < avg50 && avg50 >= avg200 => 55.0,
        (Some(_), Some(_)) => 35.0,
        (Some(avg50), None) if fact.price >= avg50 => 75.0,
        (Some(_), None) => 45.0,
        _ => 60.0,
    }
}

fn valuation_matches_quote(valuation: &DailyValuation, quote: Option<&QuoteFact>) -> bool {
    quote.is_none_or(|quote| {
        let denominator = quote.price.abs().max(1.0);
        (valuation.current_price - quote.price).abs() / denominator <= 0.05
    })
}

fn valuation_health_score(valuation: &DailyValuation) -> f64 {
    let raw = (50.0 + valuation.expected_upside_percent * 1.5).clamp(5.0, 95.0);
    let confidence_factor = match valuation.confidence.as_str() {
        "high" => 1.0,
        "medium" => 0.85,
        _ => 0.65,
    };
    let confidence_adjusted = 50.0 + (raw - 50.0) * confidence_factor;
    (confidence_adjusted
        - if valuation.method_count >= 3 {
            0.0
        } else {
            3.0
        })
    .clamp(5.0, 95.0)
}

fn effective_confidence(base: &str, status: &str) -> String {
    match status {
        "live" => base.to_string(),
        "partial" if base == "low" => "low".to_string(),
        "partial" => "medium".to_string(),
        _ => "low".to_string(),
    }
}

fn coverage_status(
    companies: usize,
    quotes: usize,
    financials: usize,
    valuations: usize,
) -> &'static str {
    if quotes == companies && financials == companies && valuations == companies {
        "live"
    } else if quotes > 0 || financials > 0 || valuations > 0 {
        "partial"
    } else {
        "transcript_only"
    }
}

fn light_for_score(score: f64) -> &'static str {
    if score >= 75.0 {
        "green"
    } else if score >= 55.0 {
        "yellow"
    } else {
        "red"
    }
}

fn quote_as_of(fact: &QuoteFact) -> Option<String> {
    fact.as_of.clone().or_else(|| {
        fact.timestamp
            .and_then(|seconds| DateTime::from_timestamp(seconds, 0))
            .map(|value| value.to_rfc3339())
    })
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

async fn fetch_quotes(
    state: &AppState,
    keys: &[String],
    symbols: &[String],
) -> Result<HashMap<String, QuoteFact>, String> {
    let joined = symbols.join(",");
    let encoded_symbols = utf8_percent_encode(&joined, NON_ALPHANUMERIC).to_string();
    let base = quote_base_url(&state.core.config.fmp.base_url);
    let mut last_error = String::new();
    for key in keys {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!("{base}/v3/quote/{encoded_symbols}?apikey={encoded_key}");
        match fetch_fmp_json_once(&state.http_client, &url, state.core.config.fmp.timeout).await {
            Ok(value) => return Ok(quotes_from_value(&value)),
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

fn quotes_from_value(value: &Value) -> HashMap<String, QuoteFact> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let symbol = item.get("symbol")?.as_str()?.trim().to_string();
            let price = item.get("price")?.as_f64()?;
            if symbol.is_empty() || !price.is_finite() {
                return None;
            }
            Some((
                symbol,
                QuoteFact {
                    price,
                    change_percent: item.get("changesPercentage").and_then(Value::as_f64),
                    avg50: item.get("priceAvg50").and_then(Value::as_f64),
                    avg200: item.get("priceAvg200").and_then(Value::as_f64),
                    year_low: item.get("yearLow").and_then(Value::as_f64),
                    year_high: item.get("yearHigh").and_then(Value::as_f64),
                    timestamp: item.get("timestamp").and_then(Value::as_i64),
                    as_of: None,
                    source: "FMP 行情快照".to_string(),
                    market_history: None,
                    short_interest: None,
                    options_positioning: None,
                    news_attention: None,
                    institutional_holdings: None,
                    analyst_consensus: None,
                },
            ))
        })
        .collect()
}

async fn fetch_nasdaq_quotes(state: &AppState, symbols: &[String]) -> HashMap<String, QuoteFact> {
    let client = &state.http_client;
    let attempts = stream::iter(symbols.iter().cloned().map(|symbol| async move {
        let encoded = utf8_percent_encode(&symbol, NON_ALPHANUMERIC).to_string();
        let url = format!("https://api.nasdaq.com/api/quote/{encoded}/info?assetclass=stocks");
        let result = async {
            let response = client
                .get(&url)
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (HONE research dashboard)",
                )
                .header(reqwest::header::ACCEPT, "application/json")
                .timeout(Duration::from_secs(12))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            let value = response
                .json::<Value>()
                .await
                .map_err(|error| error.to_string())?;
            nasdaq_quote_from_value(&symbol, &value)
                .ok_or_else(|| "response contained no usable quote".to_string())
        }
        .await;
        (symbol, result)
    }))
    .buffer_unordered(8)
    .collect::<Vec<_>>()
    .await;

    attempts
        .into_iter()
        .filter_map(|(symbol, result)| match result {
            Ok(quote) => Some((symbol, quote)),
            Err(error) => {
                warn!(%symbol, %error, "Nasdaq quote fallback failed");
                None
            }
        })
        .collect()
}

fn nasdaq_quote_from_value(expected_symbol: &str, value: &Value) -> Option<QuoteFact> {
    let data = value.get("data")?;
    let symbol = data.get("symbol")?.as_str()?.trim();
    if !symbol.eq_ignore_ascii_case(expected_symbol) {
        return None;
    }
    let primary = data.get("primaryData")?;
    let price = parse_display_number(primary.get("lastSalePrice")?.as_str()?)?;
    let change_percent = primary
        .get("percentageChange")
        .and_then(Value::as_str)
        .and_then(parse_display_number);
    let as_of = primary
        .get("lastTradeTimestamp")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("{value} · Nasdaq"));
    let (year_low, year_high) = data
        .pointer("/keyStats/fiftyTwoWeekHighLow/value")
        .and_then(Value::as_str)
        .and_then(parse_display_range)
        .unwrap_or((None, None));
    Some(QuoteFact {
        price,
        change_percent,
        avg50: None,
        avg200: None,
        year_low,
        year_high,
        timestamp: None,
        as_of,
        source: "Nasdaq 官方行情降级快照".to_string(),
        market_history: None,
        short_interest: None,
        options_positioning: None,
        news_attention: None,
        institutional_holdings: None,
        analyst_consensus: None,
    })
}

async fn fetch_nasdaq_short_interests(
    state: &AppState,
    symbols: &[String],
) -> HashMap<String, ShortInterestSummary> {
    let client = &state.http_client;
    let requested_through = Utc::now().date_naive();
    let attempts = stream::iter(symbols.iter().cloned().map(|symbol| async move {
        let encoded = utf8_percent_encode(&symbol, NON_ALPHANUMERIC).to_string();
        let url =
            format!("https://api.nasdaq.com/api/quote/{encoded}/short-interest?assetclass=stocks");
        let result = async {
            let response = client
                .get(&url)
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (HONE research dashboard)",
                )
                .header(reqwest::header::ACCEPT, "application/json")
                .timeout(Duration::from_secs(16))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            let value = response
                .json::<Value>()
                .await
                .map_err(|error| error.to_string())?;
            nasdaq_short_interest_from_value(&value, &url, requested_through)
        }
        .await;
        (symbol, result)
    }))
    .buffer_unordered(8)
    .collect::<Vec<_>>()
    .await;

    attempts
        .into_iter()
        .filter_map(|(symbol, result)| match result {
            Ok(summary) => Some((symbol, summary)),
            Err(error) => {
                warn!(%symbol, %error, "Nasdaq short-interest context failed");
                None
            }
        })
        .collect()
}

fn nasdaq_short_interest_from_value(
    value: &Value,
    source_url: &str,
    requested_through: NaiveDate,
) -> Result<ShortInterestSummary, String> {
    let rows = value
        .pointer("/data/shortInterestTable/rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "response contained no Nasdaq short-interest rows".to_string())?;
    let mut by_date = BTreeMap::new();
    for row in rows {
        let Some(date) = row
            .get("settlementDate")
            .and_then(Value::as_str)
            .and_then(|value| NaiveDate::parse_from_str(value.trim(), "%m/%d/%Y").ok())
        else {
            continue;
        };
        let Some(shares_short) = row
            .get("interest")
            .and_then(Value::as_str)
            .and_then(parse_display_number)
            .filter(|value| *value > 0.0)
        else {
            continue;
        };
        let Some(average_daily_share_volume) = row
            .get("avgDailyShareVolume")
            .and_then(Value::as_str)
            .and_then(parse_display_number)
            .filter(|value| *value > 0.0)
        else {
            continue;
        };
        let Some(days_to_cover) = row
            .get("daysToCover")
            .and_then(|value| {
                value
                    .as_f64()
                    .or_else(|| value.as_str().and_then(parse_display_number))
            })
            .filter(|value| *value >= 0.0)
        else {
            continue;
        };
        by_date.insert(
            date,
            (shares_short, average_daily_share_volume, days_to_cover),
        );
    }
    let observations = by_date.into_iter().collect::<Vec<_>>();
    if observations.len() < 2 {
        return Err(
            "response contained fewer than two valid short-interest observations".to_string(),
        );
    }
    let (latest_date, (current_shares_short, average_daily_share_volume, days_to_cover)) =
        observations[observations.len() - 1];
    let (_, (previous_shares_short, _, _)) = observations[observations.len() - 2];
    let change_percent = (current_shares_short / previous_shares_short - 1.0) * 100.0;
    let mut warnings = Vec::new();
    if latest_date > requested_through {
        warnings.push("空头仓位包含请求截止日之后的结算日".to_string());
    }
    let age_days = requested_through
        .signed_duration_since(latest_date)
        .num_days();
    if age_days > SHORT_INTEREST_MAX_AGE_DAYS {
        warnings.push(format!(
            "最近空头仓位结算日距请求日 {age_days} 天，超过45日新鲜度门槛"
        ));
    }
    Ok(ShortInterestSummary {
        policy_version: SHORT_INTEREST_POLICY_VERSION.to_string(),
        as_of: latest_date.to_string(),
        source: "Nasdaq 官方空头仓位结算表".to_string(),
        source_url: source_url.to_string(),
        current_shares_short: round4(current_shares_short),
        previous_shares_short: round4(previous_shares_short),
        change_percent: round4(change_percent),
        average_daily_share_volume: round4(average_daily_share_volume),
        days_to_cover: round4(days_to_cover),
        observation_count: observations.len(),
        quality_status: if warnings.is_empty() {
            "usable"
        } else {
            "review_required"
        }
        .to_string(),
        quality_warnings: warnings,
        interpretation: "空头股数变化和回补天数反映空头一致性与潜在回补压力；高空头既可能代表负面共识，也可能带来挤压风险，不单独表示恐惧、看空正确或投资动作。".to_string(),
    })
}

async fn fetch_nasdaq_options_positioning(
    state: &AppState,
    symbols: &[String],
) -> HashMap<String, OptionsPositioningSummary> {
    let client = &state.http_client;
    let requested_through = Utc::now().date_naive();
    let Some(expiration) = monthly_option_expiration(requested_through) else {
        return HashMap::new();
    };
    let attempts = stream::iter(symbols.iter().cloned().map(|symbol| async move {
        let encoded = utf8_percent_encode(&symbol, NON_ALPHANUMERIC).to_string();
        let url = format!(
            "https://api.nasdaq.com/api/quote/{encoded}/option-chain?assetclass=stocks&limit=2000&fromdate={expiration}&todate={expiration}&excode=oprac&callput=callput&money=all&type=all"
        );
        let result = async {
            let response = client
                .get(&url)
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (HONE research dashboard)",
                )
                .header(reqwest::header::ACCEPT, "application/json")
                .timeout(Duration::from_secs(20))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            let value = response
                .json::<Value>()
                .await
                .map_err(|error| error.to_string())?;
            nasdaq_options_positioning_from_value(
                &value,
                &url,
                requested_through,
                expiration,
            )
        }
        .await;
        (symbol, result)
    }))
    .buffer_unordered(4)
    .collect::<Vec<_>>()
    .await;

    attempts
        .into_iter()
        .filter_map(|(symbol, result)| match result {
            Ok(summary) => Some((symbol, summary)),
            Err(error) => {
                warn!(%symbol, %error, "Nasdaq options-positioning context failed");
                None
            }
        })
        .collect()
}

fn monthly_option_expiration(requested_through: NaiveDate) -> Option<NaiveDate> {
    let base_month = requested_through.year() * 12 + requested_through.month0() as i32;
    (0..4).find_map(|offset| {
        let index = base_month + offset;
        let year = index.div_euclid(12);
        let month = index.rem_euclid(12) as u32 + 1;
        let expiration = (15..=21).find_map(|day| {
            NaiveDate::from_ymd_opt(year, month, day)
                .filter(|date| date.weekday() == chrono::Weekday::Fri)
        })?;
        let days = expiration
            .signed_duration_since(requested_through)
            .num_days();
        (28..=75).contains(&days).then_some(expiration)
    })
}

fn nasdaq_options_positioning_from_value(
    value: &Value,
    source_url: &str,
    requested_through: NaiveDate,
    expected_expiration: NaiveDate,
) -> Result<OptionsPositioningSummary, String> {
    let data = value
        .get("data")
        .ok_or_else(|| "response contained no Nasdaq option-chain data".to_string())?;
    let last_trade = data
        .get("lastTrade")
        .and_then(Value::as_str)
        .ok_or_else(|| "option chain omitted last-trade provenance".to_string())?;
    let (spot_price, as_of) = parse_nasdaq_last_trade(last_trade)
        .ok_or_else(|| "option chain last-trade provenance was invalid".to_string())?;
    let rows = data
        .pointer("/table/rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "response contained no Nasdaq option-chain rows".to_string())?;
    let total_record = data
        .get("totalRecord")
        .and_then(|value| {
            value
                .as_u64()
                .or_else(|| value.as_str().and_then(|value| value.parse::<u64>().ok()))
        })
        .unwrap_or(rows.len() as u64) as usize;
    let expected_label = expected_expiration.format("%B %-d, %Y").to_string();
    let mut saw_expected_group = false;
    let mut call_open_interest = 0.0;
    let mut put_open_interest = 0.0;
    let mut call_volume = 0.0;
    let mut put_volume = 0.0;
    let mut contract_rows = 0usize;
    for row in rows {
        if let Some(group) = row
            .get("expirygroup")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            saw_expected_group |= group == expected_label;
            continue;
        }
        if row
            .get("strike")
            .and_then(Value::as_str)
            .and_then(parse_display_number)
            .is_none()
        {
            continue;
        }
        contract_rows += 1;
        call_open_interest += option_display_number(row.get("c_Openinterest"));
        put_open_interest += option_display_number(row.get("p_Openinterest"));
        call_volume += option_display_number(row.get("c_Volume"));
        put_volume += option_display_number(row.get("p_Volume"));
    }
    if contract_rows == 0 {
        return Err("response contained no valid option contracts".to_string());
    }
    let mut warnings = Vec::new();
    if as_of > requested_through {
        warnings.push("期权链包含请求截止日之后的行情日期".to_string());
    }
    let age_days = requested_through.signed_duration_since(as_of).num_days();
    if age_days > OPTIONS_POSITIONING_MAX_AGE_DAYS {
        warnings.push(format!(
            "期权链行情距请求日 {age_days} 天，超过五日新鲜度门槛"
        ));
    }
    if !saw_expected_group {
        warnings.push("返回链未确认请求的标准月度到期日".to_string());
    }
    if total_record > rows.len() {
        warnings.push(format!(
            "期权链只返回 {}/{} 行，未平仓量合计可能被截断",
            rows.len(),
            total_record
        ));
    }
    if call_open_interest <= 0.0 || put_open_interest <= 0.0 {
        warnings.push("看涨或看跌未平仓量为空，无法形成双边仓位比".to_string());
    }
    let days_to_expiration = expected_expiration.signed_duration_since(as_of).num_days();
    if !(20..=90).contains(&days_to_expiration) {
        warnings.push(format!(
            "到期日距行情日 {days_to_expiration} 天，不在中短期观察窗口"
        ));
    }
    Ok(OptionsPositioningSummary {
        policy_version: OPTIONS_POSITIONING_POLICY_VERSION.to_string(),
        as_of: as_of.to_string(),
        source: "Nasdaq 官方综合期权链".to_string(),
        source_url: source_url.to_string(),
        expiration_date: expected_expiration.to_string(),
        days_to_expiration,
        spot_price: round4(spot_price),
        call_open_interest: round4(call_open_interest),
        put_open_interest: round4(put_open_interest),
        put_call_open_interest_ratio: (call_open_interest > 0.0)
            .then(|| round4(put_open_interest / call_open_interest)),
        call_volume: round4(call_volume),
        put_volume: round4(put_volume),
        put_call_volume_ratio: (call_volume > 0.0).then(|| round4(put_volume / call_volume)),
        contract_rows,
        quality_status: if warnings.is_empty() {
            "usable"
        } else {
            "review_required"
        }
        .to_string(),
        quality_warnings: warnings,
        interpretation: "指定标准月到期日的看跌/看涨未平仓量与成交量反映期权仓位结构；交易可能来自保护、备兑、价差或投机，比例不等同方向判断。本源不提供可验证的隐含波动率或偏斜，因此不推算。".to_string(),
    })
}

fn option_display_number(value: Option<&Value>) -> f64 {
    value
        .and_then(|value| {
            value
                .as_f64()
                .or_else(|| value.as_str().and_then(parse_display_number))
        })
        .filter(|value| value.is_finite() && *value >= 0.0)
        .unwrap_or(0.0)
}

fn parse_nasdaq_last_trade(value: &str) -> Option<(f64, NaiveDate)> {
    let price_text = value.split('$').nth(1)?.split_whitespace().next()?;
    let price = parse_display_number(price_text)?;
    let date_text = value
        .split_once("AS OF")?
        .1
        .trim()
        .trim_end_matches(')')
        .trim();
    let normalized_date = date_text
        .split_whitespace()
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                let mut chars = part.chars();
                chars
                    .next()
                    .map(|first| {
                        first.to_uppercase().collect::<String>()
                            + &chars.as_str().to_ascii_lowercase()
                    })
                    .unwrap_or_default()
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let date = NaiveDate::parse_from_str(&normalized_date, "%b %d, %Y").ok()?;
    Some((price, date))
}

async fn fetch_nasdaq_news_attention(
    state: &AppState,
    symbols: &[String],
) -> HashMap<String, NewsAttentionSummary> {
    let client = &state.http_client;
    let requested_through = Utc::now().date_naive();
    let attempts = stream::iter(symbols.iter().cloned().map(|symbol| async move {
        let query = utf8_percent_encode(&format!("{symbol}|stocks"), NON_ALPHANUMERIC).to_string();
        let url = format!(
            "https://www.nasdaq.com/api/news/topic/articlebysymbol?q={query}&limit={NEWS_ATTENTION_RESULT_LIMIT}"
        );
        let result = async {
            let response = client
                .get(&url)
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (HONE research dashboard)",
                )
                .header(reqwest::header::ACCEPT, "application/json")
                .timeout(Duration::from_secs(20))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            let value = response
                .json::<Value>()
                .await
                .map_err(|error| error.to_string())?;
            nasdaq_news_attention_from_value(&symbol, &value, &url, requested_through)
        }
        .await;
        (symbol, result)
    }))
    .buffer_unordered(4)
    .collect::<Vec<_>>()
    .await;

    attempts
        .into_iter()
        .filter_map(|(symbol, result)| match result {
            Ok(summary) => Some((symbol, summary)),
            Err(error) => {
                warn!(%symbol, %error, "Nasdaq syndicated-news attention failed");
                None
            }
        })
        .collect()
}

fn nasdaq_news_attention_from_value(
    expected_symbol: &str,
    value: &Value,
    source_url: &str,
    requested_through: NaiveDate,
) -> Result<NewsAttentionSummary, String> {
    let rows = value
        .pointer("/data/rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "response contained no Nasdaq syndicated-news rows".to_string())?;
    let expected_relation = format!("{}|stocks", expected_symbol.to_ascii_lowercase());
    let mut by_identity = BTreeMap::new();
    for row in rows {
        let related = row
            .get("related_symbols")
            .and_then(Value::as_array)
            .is_some_and(|symbols| {
                symbols.iter().any(|symbol| {
                    symbol
                        .as_str()
                        .is_some_and(|symbol| symbol.eq_ignore_ascii_case(&expected_relation))
                })
            });
        let primary = row
            .get("primarysymbol")
            .and_then(Value::as_str)
            .is_some_and(|symbol| symbol.eq_ignore_ascii_case(expected_symbol));
        if !related && !primary {
            continue;
        }
        let Some(date) = row
            .get("created")
            .and_then(Value::as_str)
            .and_then(|value| NaiveDate::parse_from_str(value.trim(), "%b %d, %Y").ok())
        else {
            continue;
        };
        let publisher = row
            .get("publisher")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("")
            .to_string();
        let title = row
            .get("title")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or("");
        let identity = row
            .get("id")
            .filter(|value| !value.is_null())
            .map(Value::to_string)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("{date}|{publisher}|{title}"));
        by_identity.insert(identity, (date, publisher));
    }
    let observed = by_identity.into_values().collect::<Vec<_>>();
    let oldest_observed_date = observed
        .iter()
        .filter(|(date, _)| *date <= requested_through)
        .map(|(date, _)| *date)
        .min();
    let window_start = requested_through - chrono::Duration::days(NEWS_ATTENTION_WINDOW_DAYS - 1);
    let recent_start = requested_through - chrono::Duration::days(NEWS_ATTENTION_RECENT_DAYS - 1);
    let prior_days = NEWS_ATTENTION_WINDOW_DAYS - NEWS_ATTENTION_RECENT_DAYS;
    let mut recent_article_count = 0usize;
    let mut prior_article_count = 0usize;
    let mut publishers = BTreeSet::new();
    let mut warnings = Vec::new();
    let mut saw_future_publication = false;
    for (date, publisher) in &observed {
        if *date > requested_through {
            saw_future_publication = true;
            continue;
        }
        if *date < window_start {
            continue;
        }
        if !publisher.is_empty() {
            publishers.insert(publisher.clone());
        }
        if *date >= recent_start {
            recent_article_count += 1;
        } else {
            prior_article_count += 1;
        }
    }
    if saw_future_publication {
        warnings.push("新闻聚合流包含请求截止日之后的发布日期".to_string());
    }
    let truncated_window = rows.len() >= NEWS_ATTENTION_RESULT_LIMIT
        && oldest_observed_date.is_none_or(|date| date > window_start);
    if truncated_window {
        warnings.push("聚合结果达到100条上限但未覆盖完整14日窗口，活跃度只可视为下界".to_string());
    }
    let recent_daily_rate = recent_article_count as f64 / NEWS_ATTENTION_RECENT_DAYS as f64;
    let prior_daily_rate = prior_article_count as f64 / prior_days as f64;
    Ok(NewsAttentionSummary {
        policy_version: NEWS_ATTENTION_POLICY_VERSION.to_string(),
        as_of: requested_through.to_string(),
        source: "Nasdaq 公司新闻聚合发布流（第三方媒体）".to_string(),
        source_url: source_url.to_string(),
        window_days: NEWS_ATTENTION_WINDOW_DAYS,
        recent_window_days: NEWS_ATTENTION_RECENT_DAYS,
        recent_article_count,
        prior_article_count,
        recent_daily_rate: round4(recent_daily_rate),
        prior_daily_rate: round4(prior_daily_rate),
        activity_ratio: (prior_daily_rate > 0.0)
            .then(|| round4(recent_daily_rate / prior_daily_rate)),
        unique_publishers: publishers.len(),
        observed_article_count: observed.len(),
        oldest_observed_date: oldest_observed_date
            .map(|date| date.to_string())
            .unwrap_or_default(),
        result_limit: NEWS_ATTENTION_RESULT_LIMIT,
        truncated_window,
        quality_status: if warnings.is_empty() {
            "usable"
        } else {
            "review_required"
        }
        .to_string(),
        quality_warnings: warnings,
        interpretation: "该指标只计 Nasdaq 聚合页中的媒体发布活跃度，不代表文章观点、事实正确性、投资者情绪或独立新闻数量；同一事件可能被多家媒体重复覆盖，因此不单独形成方向或动作。".to_string(),
    })
}

async fn fetch_nasdaq_institutional_holdings(
    state: &AppState,
    symbols: &[String],
) -> HashMap<String, InstitutionalHoldingsSummary> {
    let client = &state.http_client;
    let observed_on = Utc::now().date_naive();
    let attempts = stream::iter(symbols.iter().cloned().map(|symbol| async move {
        let encoded = utf8_percent_encode(&symbol, NON_ALPHANUMERIC).to_string();
        let url = format!(
            "https://api.nasdaq.com/api/company/{encoded}/institutional-holdings?limit={INSTITUTIONAL_HOLDINGS_RESULT_LIMIT}"
        );
        let result = async {
            let response = client
                .get(&url)
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (HONE research dashboard)",
                )
                .header(reqwest::header::ACCEPT, "application/json")
                .timeout(Duration::from_secs(20))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            let value = response
                .json::<Value>()
                .await
                .map_err(|error| error.to_string())?;
            nasdaq_institutional_holdings_from_value(&value, &url, observed_on)
        }
        .await;
        (symbol, result)
    }))
    .buffer_unordered(4)
    .collect::<Vec<_>>()
    .await;

    attempts
        .into_iter()
        .filter_map(|(symbol, result)| match result {
            Ok(summary) => Some((symbol, summary)),
            Err(error) => {
                warn!(%symbol, %error, "Nasdaq institutional-holdings context failed");
                None
            }
        })
        .collect()
}

fn nasdaq_institutional_holdings_from_value(
    value: &Value,
    source_url: &str,
    observed_on: NaiveDate,
) -> Result<InstitutionalHoldingsSummary, String> {
    let data = value
        .get("data")
        .ok_or_else(|| "response contained no Nasdaq institutional-holdings data".to_string())?;
    let institutional_ownership_percent = data
        .pointer("/ownershipSummary/SharesOutstandingPCT/value")
        .and_then(Value::as_str)
        .and_then(parse_display_number)
        .filter(|value| *value >= 0.0)
        .ok_or_else(|| "institutional ownership percentage was unavailable".to_string())?;
    let transactions = data
        .get("holdingsTransactions")
        .ok_or_else(|| "institutional transaction summary was unavailable".to_string())?;
    let institutional_holders = transactions
        .get("institutionalHolders")
        .and_then(Value::as_str)
        .and_then(parse_leading_display_number)
        .map(|value| value as usize)
        .filter(|value| *value > 0)
        .ok_or_else(|| "institutional holder count was unavailable".to_string())?;
    let total_shares_held = transactions
        .get("sharesHeld")
        .and_then(Value::as_str)
        .and_then(parse_leading_display_number)
        .filter(|value| *value > 0.0)
        .ok_or_else(|| "institutional share total was unavailable".to_string())?;
    let total_reported_records = transactions
        .get("totalRecords")
        .and_then(|value| {
            value.as_u64().map(|value| value as usize).or_else(|| {
                value
                    .as_str()
                    .and_then(parse_leading_display_number)
                    .map(|value| value as usize)
            })
        })
        .filter(|value| *value > 0)
        .ok_or_else(|| "institutional report count was unavailable".to_string())?;
    let holder_rows = transactions
        .pointer("/table/rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "institutional holder table was unavailable".to_string())?;
    let mut report_periods = BTreeSet::new();
    let mut warnings = Vec::new();
    for row in holder_rows {
        let Some(period) = row
            .get("date")
            .and_then(Value::as_str)
            .and_then(|value| NaiveDate::parse_from_str(value.trim(), "%m/%d/%Y").ok())
        else {
            continue;
        };
        if period > observed_on {
            warnings.push("机构持仓表包含观察日之后的报告期".to_string());
        }
        report_periods.insert(period);
    }
    let earliest_report_period = report_periods
        .first()
        .copied()
        .ok_or_else(|| "institutional holder rows contained no report period".to_string())?;
    let latest_report_period = report_periods
        .last()
        .copied()
        .unwrap_or(earliest_report_period);
    let latest_period_rows_in_sample = holder_rows
        .iter()
        .filter(|row| {
            row.get("date")
                .and_then(Value::as_str)
                .and_then(|value| NaiveDate::parse_from_str(value.trim(), "%m/%d/%Y").ok())
                == Some(latest_report_period)
        })
        .count();
    let active_rows = data
        .pointer("/activePositions/rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "institutional active-position summary was unavailable".to_string())?;
    let new_sold_rows = data
        .pointer("/newSoldOutPositions/rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "institutional new/sold-out summary was unavailable".to_string())?;
    let (increased_positions_holders, increased_positions_shares) =
        institutional_position_row(active_rows, "Increased Positions")?;
    let (decreased_positions_holders, decreased_positions_shares) =
        institutional_position_row(active_rows, "Decreased Positions")?;
    let (held_positions_holders, held_positions_shares) =
        institutional_position_row(active_rows, "Held Positions")?;
    let (new_positions_holders, new_positions_shares) =
        institutional_position_row(new_sold_rows, "New Positions")?;
    let (sold_out_positions_holders, sold_out_positions_shares) =
        institutional_position_row(new_sold_rows, "Sold Out Positions")?;
    let active_holders =
        increased_positions_holders + decreased_positions_holders + held_positions_holders;
    let active_shares =
        increased_positions_shares + decreased_positions_shares + held_positions_shares;
    if active_holders != institutional_holders {
        warnings.push("机构持有人总数与增持/减持/持平分类无法对账".to_string());
    }
    if (active_shares - total_shares_held).abs() > 1.0 {
        warnings.push("机构持股总数与增持/减持/持平分类无法对账".to_string());
    }
    if total_reported_records < holder_rows.len() {
        warnings.push("机构记录总数小于返回样本行数".to_string());
    }
    warnings.sort();
    warnings.dedup();

    Ok(InstitutionalHoldingsSummary {
        policy_version: INSTITUTIONAL_HOLDINGS_POLICY_VERSION.to_string(),
        observed_on: observed_on.to_string(),
        source: "Nasdaq 机构持仓聚合表（SEC 13F 报告）".to_string(),
        source_url: source_url.to_string(),
        institutional_ownership_percent: round4(institutional_ownership_percent),
        institutional_holders,
        total_shares_held: round4(total_shares_held),
        total_reported_records,
        top_sample_rows: holder_rows.len(),
        holder_table_truncated: total_reported_records > holder_rows.len(),
        earliest_report_period: earliest_report_period.to_string(),
        latest_report_period: latest_report_period.to_string(),
        report_period_count: report_periods.len(),
        latest_period_rows_in_sample,
        increased_positions_holders,
        increased_positions_shares: round4(increased_positions_shares),
        decreased_positions_holders,
        decreased_positions_shares: round4(decreased_positions_shares),
        held_positions_holders,
        held_positions_shares: round4(held_positions_shares),
        new_positions_holders,
        new_positions_shares: round4(new_positions_shares),
        sold_out_positions_holders,
        sold_out_positions_shares: round4(sold_out_positions_shares),
        quality_status: if warnings.is_empty() {
            "usable"
        } else {
            "review_required"
        }
        .to_string(),
        quality_warnings: warnings,
        interpretation: "机构持仓来自SEC 13F季度披露，通常可在季度结束后45天内提交；Nasdaq聚合表会混合不同机构的不同报告期。这里展示观察日看到的所有持有人分类汇总和前50大记录的报告期分布，不代表机构今天买卖，不据此计算完整机构集中度，也不单独形成方向或动作。".to_string(),
    })
}

fn institutional_position_row(rows: &[Value], label: &str) -> Result<(usize, f64), String> {
    let row = rows
        .iter()
        .find(|row| {
            row.get("positions")
                .and_then(Value::as_str)
                .is_some_and(|value| value.trim() == label)
        })
        .ok_or_else(|| format!("institutional position category {label} was unavailable"))?;
    let holders = row
        .get("holders")
        .and_then(Value::as_str)
        .and_then(parse_display_number)
        .filter(|value| *value >= 0.0)
        .map(|value| value as usize)
        .ok_or_else(|| format!("institutional holder count for {label} was invalid"))?;
    let shares = row
        .get("shares")
        .and_then(Value::as_str)
        .and_then(parse_display_number)
        .filter(|value| *value >= 0.0)
        .ok_or_else(|| format!("institutional share count for {label} was invalid"))?;
    Ok((holders, shares))
}

fn parse_leading_display_number(value: &str) -> Option<f64> {
    value
        .split_whitespace()
        .next()
        .and_then(parse_display_number)
}

async fn fetch_nasdaq_analyst_consensus(
    state: &AppState,
    symbols: &[String],
) -> HashMap<String, AnalystConsensusSummary> {
    let client = &state.http_client;
    let observed_on = Utc::now().date_naive();
    let attempts = stream::iter(symbols.iter().cloned().map(|symbol| async move {
        let encoded = utf8_percent_encode(&symbol, NON_ALPHANUMERIC).to_string();
        let url = format!("https://api.nasdaq.com/api/analyst/{encoded}/targetprice");
        let result = async {
            let response = client
                .get(&url)
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (HONE research dashboard)",
                )
                .header(reqwest::header::ACCEPT, "application/json")
                .timeout(Duration::from_secs(20))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            let value = response
                .json::<Value>()
                .await
                .map_err(|error| error.to_string())?;
            nasdaq_analyst_consensus_from_value(&value, &url, observed_on)
        }
        .await;
        (symbol, result)
    }))
    .buffer_unordered(4)
    .collect::<Vec<_>>()
    .await;

    attempts
        .into_iter()
        .filter_map(|(symbol, result)| match result {
            Ok(summary) => Some((symbol, summary)),
            Err(error) => {
                warn!(%symbol, %error, "Nasdaq analyst-consensus context failed");
                None
            }
        })
        .collect()
}

fn nasdaq_analyst_consensus_from_value(
    value: &Value,
    source_url: &str,
    observed_on: NaiveDate,
) -> Result<AnalystConsensusSummary, String> {
    let data = value
        .get("data")
        .ok_or_else(|| "response contained no Nasdaq analyst-consensus data".to_string())?;
    let overview = data
        .get("consensusOverview")
        .ok_or_else(|| "analyst consensus overview was unavailable".to_string())?;
    let buy_count = parse_nonnegative_usize(overview.get("buy"))
        .ok_or_else(|| "analyst buy count was unavailable".to_string())?;
    let hold_count = parse_nonnegative_usize(overview.get("hold"))
        .ok_or_else(|| "analyst hold count was unavailable".to_string())?;
    let sell_count = parse_nonnegative_usize(overview.get("sell"))
        .ok_or_else(|| "analyst sell count was unavailable".to_string())?;
    let recommendation_count = buy_count + hold_count + sell_count;
    if recommendation_count == 0 {
        return Err("analyst recommendation sample was empty".to_string());
    }
    let consensus_target_price = overview
        .get("priceTarget")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| "consensus target price was unavailable".to_string())?;
    let low_target_price = overview
        .get("lowPriceTarget")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| "low target price was unavailable".to_string())?;
    let high_target_price = overview
        .get("highPriceTarget")
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value > 0.0)
        .ok_or_else(|| "high target price was unavailable".to_string())?;
    let historical = data
        .get("historicalConsensus")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut warnings = Vec::new();
    if recommendation_count < 3 {
        warnings.push("分析师建议样本少于3个，不能视为稳定共识".to_string());
    }
    if !(low_target_price <= consensus_target_price && consensus_target_price <= high_target_price)
    {
        warnings.push("目标价低值、共识值与高值无法按顺序对账".to_string());
    }
    if historical.is_empty() {
        warnings.push("Nasdaq 未返回历史共识序列".to_string());
    }
    if historical.iter().any(|entry| {
        entry
            .pointer("/z/date")
            .and_then(Value::as_str)
            .and_then(|date| NaiveDate::parse_from_str(date.trim(), "%m/%d/%Y").ok())
            .is_some_and(|date| date > observed_on)
    }) {
        warnings.push("历史共识序列包含观察日之后的月份".to_string());
    }
    let total = recommendation_count as f64;
    let (dominant_rating, dominant_count) =
        dominant_analyst_bucket(buy_count, hold_count, sell_count);
    warnings.sort();
    warnings.dedup();

    Ok(AnalystConsensusSummary {
        policy_version: ANALYST_CONSENSUS_POLICY_VERSION.to_string(),
        observed_on: observed_on.to_string(),
        source: "Nasdaq 分析师建议与目标价聚合".to_string(),
        source_url: source_url.to_string(),
        buy_count,
        hold_count,
        sell_count,
        recommendation_count,
        buy_share_percent: round4(buy_count as f64 / total * 100.0),
        hold_share_percent: round4(hold_count as f64 / total * 100.0),
        sell_share_percent: round4(sell_count as f64 / total * 100.0),
        dominant_rating: dominant_rating.to_string(),
        dominant_count,
        dominant_share_percent: round4(dominant_count as f64 / total * 100.0),
        consensus_target_price: round4(consensus_target_price),
        low_target_price: round4(low_target_price),
        high_target_price: round4(high_target_price),
        target_range_width_percent: round4(
            (high_target_price - low_target_price) / consensus_target_price * 100.0,
        ),
        historical_month_count: historical.len(),
        quality_status: if warnings.is_empty() {
            "usable"
        } else {
            "review_required"
        }
        .to_string(),
        quality_warnings: warnings,
        interpretation: "该记录是观察日看到的 Nasdaq 聚合快照：买入/持有/卖出分布可用于识别建议是否集中，但不代表独立样本或真实资金仓位；Nasdaq 未披露目标价贡献者数量、更新时间与逐笔明细，因此目标价仅作背景展示，不进入评分，也不单独形成方向或动作。".to_string(),
    })
}

fn parse_nonnegative_usize(value: Option<&Value>) -> Option<usize> {
    value.and_then(|value| {
        value.as_u64().map(|value| value as usize).or_else(|| {
            value
                .as_str()
                .and_then(parse_display_number)
                .filter(|value| value.is_finite() && *value >= 0.0 && value.fract() == 0.0)
                .map(|value| value as usize)
        })
    })
}

fn dominant_analyst_bucket(buy: usize, hold: usize, sell: usize) -> (&'static str, usize) {
    let maximum = buy.max(hold).max(sell);
    let ties = [buy, hold, sell]
        .into_iter()
        .filter(|count| *count == maximum)
        .count();
    if ties > 1 {
        ("并列", maximum)
    } else if buy == maximum {
        ("买入", maximum)
    } else if hold == maximum {
        ("持有", maximum)
    } else {
        ("卖出", maximum)
    }
}

async fn fetch_nasdaq_market_histories(
    state: &AppState,
    symbols: &[String],
) -> HashMap<String, MarketHistorySummary> {
    let client = &state.http_client;
    let to = Utc::now().date_naive();
    let from = to - chrono::Duration::days(MARKET_HISTORY_LOOKBACK_DAYS);
    let attempts = stream::iter(symbols.iter().cloned().map(|symbol| async move {
        let encoded = utf8_percent_encode(&symbol, NON_ALPHANUMERIC).to_string();
        let url = format!(
            "https://api.nasdaq.com/api/quote/{encoded}/historical?assetclass=stocks&fromdate={from}&todate={to}&limit=5000"
        );
        let result = async {
            let response = client
                .get(&url)
                .header(
                    reqwest::header::USER_AGENT,
                    "Mozilla/5.0 (HONE research dashboard)",
                )
                .header(reqwest::header::ACCEPT, "application/json")
                .timeout(Duration::from_secs(16))
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if !response.status().is_success() {
                return Err(format!("HTTP {}", response.status()));
            }
            let value = response
                .json::<Value>()
                .await
                .map_err(|error| error.to_string())?;
            nasdaq_market_history_from_value(&value, &url, to)
        }
        .await;
        (symbol, result)
    }))
    .buffer_unordered(8)
    .collect::<Vec<_>>()
    .await;

    attempts
        .into_iter()
        .filter_map(|(symbol, result)| match result {
            Ok(history) => Some((symbol, history)),
            Err(error) => {
                warn!(%symbol, %error, "Nasdaq market history fallback failed");
                None
            }
        })
        .collect()
}

fn nasdaq_market_history_from_value(
    value: &Value,
    source_url: &str,
    requested_through: NaiveDate,
) -> Result<MarketHistorySummary, String> {
    let rows = value
        .pointer("/data/tradesTable/rows")
        .and_then(Value::as_array)
        .ok_or_else(|| "response contained no Nasdaq historical rows".to_string())?;
    let mut by_date = BTreeMap::new();
    for row in rows {
        let Some(date) = row
            .get("date")
            .and_then(Value::as_str)
            .and_then(|value| NaiveDate::parse_from_str(value.trim(), "%m/%d/%Y").ok())
        else {
            continue;
        };
        let Some(close) = row
            .get("close")
            .and_then(Value::as_str)
            .and_then(parse_display_number)
            .filter(|value| *value > 0.0)
        else {
            continue;
        };
        let Some(volume) = row
            .get("volume")
            .and_then(Value::as_str)
            .and_then(parse_display_number)
            .filter(|value| *value >= 0.0)
        else {
            continue;
        };
        by_date.insert(
            date,
            NasdaqDailyBar {
                date,
                close,
                volume,
            },
        );
    }
    let bars = by_date.into_values().collect::<Vec<_>>();
    let Some(latest) = bars.last() else {
        return Err("response contained no valid Nasdaq daily bars".to_string());
    };
    let mut warnings = Vec::new();
    if latest.date > requested_through {
        warnings.push("历史行情包含请求截止日之后的交易日".to_string());
    }
    let age_days = requested_through
        .signed_duration_since(latest.date)
        .num_days();
    if age_days > MARKET_HISTORY_MAX_AGE_DAYS {
        warnings.push(format!(
            "最新完整日线距请求日 {age_days} 天，超过七日新鲜度门槛"
        ));
    }
    if bars.len() < 61 {
        warnings.push(format!(
            "只有 {} 个交易日，无法完成 60 日路径验证",
            bars.len()
        ));
    }
    if let Some((left, right, move_percent)) = bars.windows(2).find_map(|pair| {
        let move_percent = (pair[1].close / pair[0].close - 1.0) * 100.0;
        (move_percent.abs() >= MARKET_HISTORY_SPLIT_REVIEW_THRESHOLD_PERCENT).then_some((
            pair[0].date,
            pair[1].date,
            move_percent,
        ))
    }) {
        warnings.push(format!(
            "{left} 至 {right} 收盘价变化 {move_percent:+.1}%，需核对拆股、复权或重大跳空"
        ));
    }
    let closes = bars.iter().map(|bar| bar.close).collect::<Vec<_>>();
    let average_close_50 = trailing_average(&closes, 50);
    let average_close_200 = trailing_average(&closes, 200);
    let return_20_sessions_percent = trailing_return(&closes, 20);
    let return_60_sessions_percent = trailing_return(&closes, 60);
    let drawdown_from_60_session_high_percent = (bars.len() >= 60).then(|| {
        let high = bars[bars.len() - 60..]
            .iter()
            .map(|bar| bar.close)
            .fold(0.0_f64, f64::max);
        (latest.close / high - 1.0) * 100.0
    });
    let recent_5_session_volume_vs_prior_55_percent = (bars.len() >= 60)
        .then(|| {
            let window = &bars[bars.len() - 60..];
            let recent = window[55..].iter().map(|bar| bar.volume).sum::<f64>() / 5.0;
            let prior = window[..55].iter().map(|bar| bar.volume).sum::<f64>() / 55.0;
            (prior > 0.0).then_some(recent / prior * 100.0)
        })
        .flatten();
    Ok(MarketHistorySummary {
        policy_version: MARKET_HISTORY_POLICY_VERSION.to_string(),
        as_of: latest.date.to_string(),
        source: "Nasdaq 官方历史日线表".to_string(),
        source_url: source_url.to_string(),
        price_basis: "Nasdaq 展示的日收盘价；接口未声明复权口径，检测到极端单日跳变时强制复核"
            .to_string(),
        session_count: bars.len(),
        latest_close: round4(latest.close),
        average_close_50: average_close_50.map(round4),
        average_close_200: average_close_200.map(round4),
        return_20_sessions_percent: return_20_sessions_percent.map(round4),
        return_60_sessions_percent: return_60_sessions_percent.map(round4),
        drawdown_from_60_session_high_percent: drawdown_from_60_session_high_percent.map(round4),
        recent_5_session_volume_vs_prior_55_percent: recent_5_session_volume_vs_prior_55_percent
            .map(round4),
        quality_status: if warnings.is_empty() {
            "usable"
        } else {
            "review_required"
        }
        .to_string(),
        quality_warnings: warnings,
    })
}

fn trailing_average(values: &[f64], sessions: usize) -> Option<f64> {
    (values.len() >= sessions)
        .then(|| values[values.len() - sessions..].iter().sum::<f64>() / sessions as f64)
}

fn trailing_return(values: &[f64], sessions: usize) -> Option<f64> {
    (values.len() > sessions).then(|| {
        let start = values[values.len() - 1 - sessions];
        (values.last().copied().unwrap_or(start) / start - 1.0) * 100.0
    })
}

fn parse_display_range(value: &str) -> Option<(Option<f64>, Option<f64>)> {
    let mut values = value
        .split(['-', '–', '—'])
        .filter_map(parse_display_number);
    let first = values.next()?;
    let second = values.next()?;
    let low = first.min(second);
    let high = first.max(second);
    (high > low && low > 0.0).then_some((Some(low), Some(high)))
}

fn parse_display_number(value: &str) -> Option<f64> {
    let normalized = value
        .trim()
        .trim_start_matches('$')
        .trim_end_matches('%')
        .replace(',', "");
    normalized
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

#[cfg(test)]
fn financial_from_value(value: &Value) -> Option<FinancialFact> {
    let rows = value.as_array()?;
    let current = rows.first()?;
    let prior = rows.get(4).or_else(|| rows.get(1));
    let revenue = current.get("revenue").and_then(Value::as_f64);
    let prior_revenue = prior
        .and_then(|row| row.get("revenue"))
        .and_then(Value::as_f64);
    let gross_margin = margin(current, "grossProfit", "revenue");
    let prior_gross_margin = prior.and_then(|row| margin(row, "grossProfit", "revenue"));
    Some(FinancialFact {
        as_of: current
            .get("date")
            .and_then(Value::as_str)
            .map(str::to_string),
        revenue_growth_percent: match (revenue, prior_revenue) {
            (Some(now), Some(previous)) if previous.abs() > f64::EPSILON => {
                Some(round1((now / previous - 1.0) * 100.0))
            }
            _ => None,
        },
        forward_revenue_growth_percent: None,
        gross_margin_percent: gross_margin.map(|value| round1(value * 100.0)),
        gross_margin_change_pp: match (gross_margin, prior_gross_margin) {
            (Some(current), Some(prior)) => Some(round1((current - prior) * 100.0)),
            _ => None,
        },
        ebit_margin_percent: margin(current, "operatingIncome", "revenue")
            .map(|value| round1(value * 100.0)),
        fcf_margin_percent: None,
        net_cash_to_revenue_percent: None,
        accounts_receivable_growth_percent: None,
        accounts_payable_growth_percent: None,
        inventory_growth_percent: None,
        property_plant_equipment_growth_percent: None,
        operating_cash_flow_growth_percent: None,
        capital_expenditure_growth_percent: None,
        free_cash_flow_growth_percent: None,
        score_eligible: true,
        review_status: "computed".to_string(),
        sources: vec!["test fixture".to_string()],
        source_urls: Vec::new(),
        source_claim_ids: Vec::new(),
        source_calculations: Vec::new(),
        source_claims: Vec::new(),
        quality_warnings: Vec::new(),
    })
}

#[cfg(test)]
fn margin(row: &Value, numerator: &str, denominator: &str) -> Option<f64> {
    let numerator = row.get(numerator)?.as_f64()?;
    let denominator = row.get(denominator)?.as_f64()?;
    (denominator.abs() > f64::EPSILON).then_some(numerator / denominator)
}

fn quote_base_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    base.strip_suffix("/v3").unwrap_or(base).to_string()
}

fn snapshot_path(state: &AppState) -> PathBuf {
    std::path::Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("company_ratings")
        .join("daily.json")
}

fn valuation_input_path(state: &AppState) -> PathBuf {
    snapshot_path(state)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("valuations")
        .join("latest.json")
}

fn fundamental_input_path(state: &AppState) -> PathBuf {
    snapshot_path(state)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("fundamentals")
        .join("latest.json")
}

fn forward_evidence_input_path(state: &AppState) -> PathBuf {
    snapshot_path(state)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("forward")
        .join("latest.json")
}

async fn read_verified_forward_evidence(state: &AppState) -> HashMap<String, ForwardFact> {
    let path = forward_evidence_input_path(state);
    let Ok(bytes) = tokio::fs::read(&path).await else {
        return HashMap::new();
    };
    let Ok(file) = serde_json::from_slice::<ForwardEvidenceFile>(&bytes) else {
        warn!(path = %path.display(), "forward evidence file is invalid JSON");
        return HashMap::new();
    };
    let now = Utc::now();
    if file.framework_version != "hone-forward-evidence-v1"
        || file.generated_at > now + chrono::Duration::minutes(5)
        || now - file.generated_at > chrono::Duration::hours(STALE_AFTER_HOURS)
    {
        warn!(path = %path.display(), "forward evidence file failed freshness validation");
        return HashMap::new();
    }
    let today = now.with_timezone(&Shanghai).date_naive();
    file.items
        .into_iter()
        .filter_map(|item| {
            let kind = item.metric_kind.trim().to_ascii_lowercase();
            let as_of = chrono::NaiveDate::parse_from_str(&item.as_of, "%Y-%m-%d").ok()?;
            let valid_kind = matches!(
                kind.as_str(),
                "backlog" | "rpo" | "arr" | "orders" | "book_to_bill" | "guidance"
            );
            if item.review_status != "verified"
                || !valid_kind
                || item.metric_label.trim().is_empty()
                || item.value_display.trim().is_empty()
                || !item.source_url.starts_with("https://")
                || as_of > today
                || today.signed_duration_since(as_of).num_days() > 200
                || item.growth_percent.is_some_and(|value| !value.is_finite())
            {
                return None;
            }
            let symbol = item.symbol.trim().to_uppercase();
            (!symbol.is_empty()).then_some((
                symbol,
                ForwardFact {
                    metric_kind: kind,
                    metric_label: item.metric_label,
                    value_display: item.value_display,
                    growth_percent: item.growth_percent,
                    as_of: item.as_of,
                    source_url: item.source_url,
                },
            ))
        })
        .collect()
}

async fn read_verified_fundamentals(state: &AppState) -> HashMap<String, FinancialFact> {
    let path = fundamental_input_path(state);
    let Ok(bytes) = tokio::fs::read(&path).await else {
        return HashMap::new();
    };
    let Ok(file) = serde_json::from_slice::<FundamentalFile>(&bytes) else {
        warn!(path = %path.display(), "daily fundamental file is invalid JSON");
        return HashMap::new();
    };
    let now = Utc::now();
    if file.framework_version != "hone-fundamentals-v1"
        || file.generated_at > now + chrono::Duration::minutes(5)
        || now - file.generated_at > chrono::Duration::hours(STALE_AFTER_HOURS)
    {
        warn!(path = %path.display(), "daily fundamental file failed freshness validation");
        return HashMap::new();
    }
    let today = now.with_timezone(&Shanghai).date_naive();
    file.items
        .into_iter()
        .filter_map(|item| {
            let as_of = chrono::NaiveDate::parse_from_str(&item.as_of, "%Y-%m-%d").ok()?;
            if item.review_status != "computed"
                || item.sources.len() < 2
                || as_of > today
                || today.signed_duration_since(as_of).num_days() > 200
            {
                return None;
            }
            let symbol = item.symbol.trim().to_uppercase();
            (!symbol.is_empty()).then_some((
                symbol,
                FinancialFact {
                    as_of: Some(item.as_of),
                    revenue_growth_percent: item.revenue_growth_percent,
                    forward_revenue_growth_percent: item.forward_revenue_growth_percent,
                    gross_margin_percent: item.gross_margin_percent,
                    gross_margin_change_pp: item.gross_margin_change_pp,
                    ebit_margin_percent: item.ebit_margin_percent,
                    fcf_margin_percent: item.fcf_margin_percent,
                    net_cash_to_revenue_percent: item.net_cash_to_revenue_percent,
                    accounts_receivable_growth_percent: item.accounts_receivable_growth_percent,
                    accounts_payable_growth_percent: item.accounts_payable_growth_percent,
                    inventory_growth_percent: item.inventory_growth_percent,
                    property_plant_equipment_growth_percent: item
                        .property_plant_equipment_growth_percent,
                    operating_cash_flow_growth_percent: item.operating_cash_flow_growth_percent,
                    capital_expenditure_growth_percent: item.capital_expenditure_growth_percent,
                    free_cash_flow_growth_percent: item.free_cash_flow_growth_percent,
                    score_eligible: true,
                    review_status: "computed".to_string(),
                    sources: item.sources,
                    source_urls: Vec::new(),
                    source_claim_ids: Vec::new(),
                    source_calculations: Vec::new(),
                    source_claims: Vec::new(),
                    quality_warnings: Vec::new(),
                },
            ))
        })
        .collect()
}

async fn read_sec_financial_evidence(
    state: &AppState,
    symbols: &[String],
) -> HashMap<String, FinancialFact> {
    let states =
        super::investment_decisions::current_sec_financial_states(state, symbols, Utc::now()).await;
    super::financial_evidence_review::review_outcomes_for_states(state, &states)
        .await
        .into_iter()
        .filter_map(|(symbol, review)| {
            financial_fact_from_sec_state(
                review.evidence,
                &review.review_status,
                review.score_eligible,
                review.blocking_reasons,
            )
            .map(|fact| (symbol, fact))
        })
        .collect()
}

fn financial_fact_from_sec_state(
    state: super::investment_decisions::FinancialVerificationState,
    review_status: &str,
    score_eligible: bool,
    blocking_reasons: Vec<String>,
) -> Option<FinancialFact> {
    let has_metrics = state.revenue_growth_percent.is_some()
        || state.gross_margin_percent.is_some()
        || state.gross_margin_change_pp.is_some()
        || state.ebit_margin_percent.is_some()
        || state.fcf_margin_percent.is_some()
        || state.accounts_receivable_growth_percent.is_some()
        || state.accounts_payable_growth_percent.is_some()
        || state.inventory_growth_percent.is_some()
        || state.property_plant_equipment_growth_percent.is_some()
        || state.operating_cash_flow_growth_percent.is_some()
        || state.capital_expenditure_growth_percent.is_some()
        || state.free_cash_flow_growth_percent.is_some();
    if !has_metrics {
        return None;
    }
    let mut quality_warnings = state.quality_warnings;
    quality_warnings.extend(blocking_reasons);
    quality_warnings.push(if score_eligible {
        "管理员已按当前证据指纹完成独立财务质量复核；原始警告继续保留供审计".to_string()
    } else {
        "SEC 结构化主张在完成独立财务质量复核前只展示证据，不进入每日评级分".to_string()
    });
    quality_warnings.sort();
    quality_warnings.dedup();
    Some(FinancialFact {
        as_of: state.financial_as_of,
        revenue_growth_percent: state.revenue_growth_percent,
        forward_revenue_growth_percent: None,
        gross_margin_percent: state.gross_margin_percent,
        gross_margin_change_pp: state.gross_margin_change_pp,
        ebit_margin_percent: state.ebit_margin_percent,
        fcf_margin_percent: state.fcf_margin_percent,
        net_cash_to_revenue_percent: None,
        accounts_receivable_growth_percent: state.accounts_receivable_growth_percent,
        accounts_payable_growth_percent: state.accounts_payable_growth_percent,
        inventory_growth_percent: state.inventory_growth_percent,
        property_plant_equipment_growth_percent: state.property_plant_equipment_growth_percent,
        operating_cash_flow_growth_percent: state.operating_cash_flow_growth_percent,
        capital_expenditure_growth_percent: state.capital_expenditure_growth_percent,
        free_cash_flow_growth_percent: state.free_cash_flow_growth_percent,
        score_eligible,
        review_status: review_status.to_string(),
        sources: state
            .source_urls
            .iter()
            .map(|url| format!("SEC XBRL 点时事实（{url}）"))
            .collect(),
        source_urls: state.source_urls,
        source_claim_ids: state.source_claim_ids,
        source_calculations: state.source_calculations,
        source_claims: state.source_claims,
        quality_warnings,
    })
}

async fn read_verified_valuations(state: &AppState) -> HashMap<String, DailyValuation> {
    let path = valuation_input_path(state);
    let Ok(bytes) = tokio::fs::read(&path).await else {
        return HashMap::new();
    };
    let Ok(file) = serde_json::from_slice::<DailyValuationFile>(&bytes) else {
        warn!(path = %path.display(), "daily valuation file is invalid JSON");
        return HashMap::new();
    };
    let now = Utc::now();
    let today = now.with_timezone(&Shanghai).date_naive().to_string();
    let (expected_review_status, strict_input_binding) = match file.framework_version.as_str() {
        "hari-invest-v1" => ("verified", false),
        "hone-valuation-v2" => ("computed", false),
        "hone-valuation-v3-reviewed-input-binding" => ("computed", true),
        _ => ("", false),
    };
    if expected_review_status.is_empty()
        || file.report_date != today
        || file.generated_at > now + chrono::Duration::minutes(5)
        || now - file.generated_at > chrono::Duration::hours(STALE_AFTER_HOURS)
    {
        warn!(path = %path.display(), "daily valuation file failed freshness or framework validation");
        return HashMap::new();
    }

    let reviewed_sec_symbols = file
        .items
        .iter()
        .filter(|item| item.input_mode == "sec_reviewed_supplemental_packet")
        .map(|item| item.symbol.trim().to_ascii_uppercase())
        .collect::<Vec<_>>();
    let authorization_bindings = if reviewed_sec_symbols.is_empty() {
        HashMap::new()
    } else {
        let financial_states = super::investment_decisions::current_sec_financial_states(
            state,
            &reviewed_sec_symbols,
            now,
        )
        .await;
        super::valuation_input_review::review_outcomes_for_states(state, &financial_states)
            .await
            .into_iter()
            .filter_map(|(symbol, candidate)| {
                let record = candidate.latest_review?;
                candidate.valuation_authorized.then_some((
                    symbol,
                    ValuationAuthorizationBinding {
                        review_id: record.review_id,
                        input_fingerprint_sha256: record.input_fingerprint_sha256,
                        financial_evidence_fingerprint_sha256: record
                            .financial_evidence_fingerprint_sha256,
                        input_as_of: record.supplemental_inputs.input_as_of,
                    },
                ))
            })
            .collect::<HashMap<_, _>>()
    };

    file.items
        .into_iter()
        .filter_map(|item| {
            let symbol = item.symbol.trim().to_ascii_uppercase();
            validated_daily_valuation(
                item,
                file.generated_at,
                &today,
                expected_review_status,
                strict_input_binding,
                authorization_bindings.get(&symbol),
            )
        })
        .collect()
}

fn validated_daily_valuation(
    item: DailyValuationInput,
    generated_at: DateTime<Utc>,
    report_date: &str,
    expected_review_status: &str,
    strict_input_binding: bool,
    expected_binding: Option<&ValuationAuthorizationBinding>,
) -> Option<(String, DailyValuation)> {
    let probability_weighted_value = item.probability_weighted_value.unwrap_or(item.base_case);
    let expected_upside_percent = item
        .expected_upside_percent
        .unwrap_or_else(|| (probability_weighted_value / item.current_price - 1.0) * 100.0);
    let valid_numbers = [
        item.bear_case,
        item.base_case,
        item.bull_case,
        item.current_price,
        probability_weighted_value,
    ]
    .into_iter()
    .all(|value| value.is_finite() && value > 0.0);
    let valid_input_binding = valuation_input_binding_is_valid(
        &item,
        report_date,
        expected_review_status,
        strict_input_binding,
        expected_binding,
    );
    if item.review_status != expected_review_status
        || item.as_of != report_date
        || item.symbol.trim().is_empty()
        || item.currency.trim().is_empty()
        || item.method.trim().is_empty()
        || item.assumptions.is_empty()
        || item.sources.len() < 2
        || (expected_review_status == "computed"
            && (item.method_count < 2 || !matches!(item.confidence.as_str(), "high" | "medium")))
        || !valid_input_binding
        || !valid_numbers
        || !expected_upside_percent.is_finite()
        || !(item.bear_case <= item.base_case && item.base_case <= item.bull_case)
        || (item.bull_case - item.bear_case).abs() <= f64::EPSILON
    {
        return None;
    }
    let position_percent =
        (item.current_price - item.bear_case) / (item.bull_case - item.bear_case) * 100.0;
    let current_position = if item.current_price < item.bear_case {
        "低于悲观值"
    } else if item.current_price < item.base_case {
        "悲观—基准之间"
    } else if item.current_price <= item.bull_case {
        "基准—乐观之间"
    } else {
        "高于乐观值"
    };
    let symbol = item.symbol.trim().to_uppercase();
    Some((
        symbol,
        DailyValuation {
            as_of: item.as_of,
            generated_at_beijing: generated_at
                .with_timezone(&Shanghai)
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            currency: item.currency,
            bear_case: round2(item.bear_case),
            base_case: round2(item.base_case),
            bull_case: round2(item.bull_case),
            current_price: round2(item.current_price),
            probability_weighted_value: round2(probability_weighted_value),
            expected_upside_percent: round1(expected_upside_percent),
            method_count: item.method_count.max(1),
            confidence: if item.confidence.is_empty() {
                "medium".to_string()
            } else {
                item.confidence
            },
            current_position: current_position.to_string(),
            position_percent: round1(position_percent),
            method: item.method,
            assumptions: item.assumptions,
            sources: item.sources,
        },
    ))
}

fn valuation_input_binding_is_valid(
    item: &DailyValuationInput,
    report_date: &str,
    expected_review_status: &str,
    strict_input_binding: bool,
    expected_binding: Option<&ValuationAuthorizationBinding>,
) -> bool {
    if expected_review_status != "computed" {
        return true;
    }
    match item.input_mode.as_str() {
        "provider_bundle" | "provider_bundle_plus_sec_observation" => true,
        "sec_reviewed_supplemental_packet" => {
            let valid_review_id = item
                .valuation_review_id
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            let valid_fingerprints = [
                item.valuation_input_fingerprint_sha256.as_deref(),
                item.valuation_financial_evidence_fingerprint_sha256
                    .as_deref(),
            ]
            .into_iter()
            .all(|value| {
                value.is_some_and(|value| {
                    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
                })
            });
            let fresh_input = NaiveDate::parse_from_str(report_date, "%Y-%m-%d")
                .ok()
                .zip(
                    item.valuation_input_as_of
                        .as_deref()
                        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()),
                )
                .is_some_and(|(report_date, input_as_of)| {
                    input_as_of <= report_date && (report_date - input_as_of).num_days() <= 7
                });
            let exact_current_binding = if strict_input_binding {
                expected_binding.is_some_and(|binding| {
                    item.valuation_review_id.as_deref() == Some(binding.review_id.as_str())
                        && item.valuation_input_fingerprint_sha256.as_deref()
                            == Some(binding.input_fingerprint_sha256.as_str())
                        && item
                            .valuation_financial_evidence_fingerprint_sha256
                            .as_deref()
                            == Some(binding.financial_evidence_fingerprint_sha256.as_str())
                        && item.valuation_input_as_of.as_deref()
                            == Some(binding.input_as_of.as_str())
                })
            } else {
                true
            };
            valid_review_id && valid_fingerprints && fresh_input && exact_current_binding
        }
        "" => !strict_input_binding,
        _ => false,
    }
}

async fn read_snapshot(state: &AppState) -> Option<CompanyRatingSnapshot> {
    let bytes = tokio::fs::read(snapshot_path(state)).await.ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub(crate) async fn current_snapshot(state: &AppState) -> CompanyRatingSnapshot {
    read_snapshot(state)
        .await
        .map(normalize_snapshot_contract)
        .map(mark_stale_if_needed)
        .unwrap_or_else(baseline_snapshot)
}

pub(crate) async fn covered_symbols(state: &AppState) -> Vec<String> {
    current_snapshot(state)
        .await
        .items
        .into_iter()
        .map(|item| item.symbol)
        .collect()
}

async fn write_snapshot(state: &AppState, snapshot: &CompanyRatingSnapshot) -> Result<(), String> {
    let path = snapshot_path(state);
    let parent = path
        .parent()
        .ok_or_else(|| "snapshot path has no parent".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let temp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(snapshot).map_err(|error| error.to_string())?;
    tokio::fs::write(&temp, bytes)
        .await
        .map_err(|error| error.to_string())?;
    tokio::fs::rename(&temp, &path)
        .await
        .map_err(|error| error.to_string())
}

fn baseline_snapshot() -> CompanyRatingSnapshot {
    snapshot_from_facts(
        parse_cards(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        false,
    )
}

fn normalize_snapshot_contract(mut snapshot: CompanyRatingSnapshot) -> CompanyRatingSnapshot {
    let legacy_contract = snapshot.methodology_version != METHODOLOGY_VERSION;
    let simulation = snapshot.data_status == "simulation";
    let cards = legacy_contract.then(|| {
        parse_cards()
            .into_iter()
            .map(|card| (card.symbol.clone(), card))
            .collect::<HashMap<_, _>>()
    });
    for item in &mut snapshot.items {
        if legacy_contract {
            item.valuation = None;
            item.dimensions.growth_quality = None;
            item.dimensions.pricing_power = None;
            item.dimensions.financial_quality = None;
            item.dimensions.timing = item
                .market_as_of
                .as_ref()
                .map(|_| item.dimensions.market_confirmation);
            if let Some(card) = cards.as_ref().and_then(|cards| cards.get(&item.symbol)) {
                item.dimensions.moat = structural_anchor(card.dimensions_1_to_5.pricing_quality);
                item.dimensions.scarcity = structural_anchor(card.dimensions_1_to_5.scarcity);
                item.dimensions.visibility = structural_anchor(card.dimensions_1_to_5.visibility);
                item.dimensions.fundamentals = if item.financial_as_of.is_none() {
                    structural_anchor(card.dimensions_1_to_5.execution)
                } else {
                    item.dimensions.fundamentals.clamp(5.0, 95.0)
                };
            }
        }
        if item.valuation.is_none() && !simulation {
            item.dimensions.valuation = None;
            item.valuation_unavailable_reason = valuation_unavailable_reason(
                None,
                item.price
                    .map(|price| QuoteFact {
                        price,
                        ..QuoteFact::default()
                    })
                    .as_ref(),
                item.financial_as_of
                    .as_ref()
                    .map(|as_of| FinancialFact {
                        as_of: Some(as_of.clone()),
                        ..FinancialFact::default()
                    })
                    .as_ref(),
            );
        }
        let (score, cap_reason) =
            apply_score_caps(weighted_score(&item.dimensions), &item.dimensions);
        item.score = score;
        item.score_cap_reason = cap_reason;
        item.factor_coverage = factor_coverage(&item.dimensions);
        item.light = if item.data_status == "transcript_only" {
            "unknown"
        } else {
            light_for_score(item.score)
        }
        .to_string();
    }
    snapshot.methodology_version = METHODOLOGY_VERSION.to_string();
    snapshot.coverage.valuations = if simulation {
        snapshot.coverage.companies
    } else {
        snapshot
            .items
            .iter()
            .filter(|item| item.valuation.is_some())
            .count()
    };
    if !simulation {
        snapshot.data_status = coverage_status(
            snapshot.coverage.companies,
            snapshot.coverage.quotes,
            snapshot.coverage.financials,
            snapshot.coverage.valuations,
        )
        .to_string();
    }
    snapshot
}

fn mark_stale_if_needed(mut snapshot: CompanyRatingSnapshot) -> CompanyRatingSnapshot {
    if snapshot.data_status != "transcript_only"
        && Utc::now() - snapshot.generated_at > chrono::Duration::hours(STALE_AFTER_HOURS)
    {
        snapshot.data_status = "stale".to_string();
        for item in &mut snapshot.items {
            if item.data_status != "transcript_only" {
                item.data_status = "stale".to_string();
            }
        }
    }
    snapshot
}

fn next_refresh(now: DateTime<Utc>) -> DateTime<Utc> {
    let local = now.with_timezone(&Shanghai);
    let today = Shanghai
        .with_ymd_and_hms(
            local.year(),
            local.month(),
            local.day(),
            REFRESH_HOUR,
            REFRESH_MINUTE,
            0,
        )
        .single()
        .expect("Shanghai local time is unambiguous");
    let target = if local < today {
        today
    } else {
        today + chrono::Duration::days(1)
    };
    target.with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use serde_json::json;

    #[test]
    fn embedded_cards_are_valid_and_cover_current_universe() {
        let cards = parse_cards();
        assert_eq!(cards.len(), 52);
        assert!(cards.iter().any(|card| card.symbol == "SNDK"));
        assert!(cards.iter().any(|card| card.symbol == "SKHY"));
        assert!(!cards.iter().any(|card| card.name == "量子板块"));
    }

    fn sec_financial_state(
        revenue_growth_percent: Option<f64>,
    ) -> super::super::investment_decisions::FinancialVerificationState {
        super::super::investment_decisions::FinancialVerificationState {
            policy_version: "hone-financial-verification-v3-sec-projection-quality-gate".into(),
            status: super::super::investment_decisions::MeasurementStatus::PartiallyMeasured,
            financial_as_of: Some("2026-07-30".into()),
            revenue_growth_percent,
            gross_margin_percent: Some(67.9),
            gross_margin_change_pp: Some(-0.9),
            ebit_margin_percent: Some(46.8),
            fcf_margin_percent: None,
            accounts_receivable_growth_percent: Some(15.7),
            accounts_payable_growth_percent: Some(53.0),
            inventory_growth_percent: Some(48.9),
            property_plant_equipment_growth_percent: None,
            operating_cash_flow_growth_percent: Some(34.4),
            capital_expenditure_growth_percent: Some(79.6),
            free_cash_flow_growth_percent: None,
            cash_and_equivalents: None,
            long_term_debt: None,
            net_cash: None,
            current_free_cash_flow: None,
            prior_free_cash_flow: None,
            financial_value_unit: None,
            forward_metric_label: None,
            forward_metric_value: None,
            forward_metric_growth_percent: None,
            forward_metric_as_of: None,
            source_claim_ids: vec!["source-claim:msft-2026:0".into()],
            source_urls: vec!["https://www.sec.gov/Archives/msft-2026.htm".into()],
            source_calculations: vec!["收入同比：281724 → 331839，变化 +17.8%".into()],
            source_claims: Vec::new(),
            quality_warnings: Vec::new(),
            missing_checks: vec!["自由现金流率".into()],
        }
    }

    #[test]
    fn sec_projection_is_visible_but_never_scores_before_human_review() {
        let fact = financial_fact_from_sec_state(
            sec_financial_state(Some(17.8)),
            "sec_structured_pending_human_review",
            false,
            vec!["尚未完成独立财务证据质量复核".into()],
        )
        .unwrap();
        assert!(!fact.score_eligible);
        assert_eq!(fact.review_status, "sec_structured_pending_human_review");
        assert_eq!(fact.revenue_growth_percent, Some(17.8));
        assert_eq!(fact.source_claim_ids.len(), 1);
        assert!(
            fact.quality_warnings
                .iter()
                .any(|warning| warning.contains("不进入每日评级分"))
        );

        let mut financials = HashMap::new();
        financials.insert("MSFT".to_string(), fact);
        let snapshot = snapshot_from_facts(
            parse_cards(),
            HashMap::new(),
            financials,
            HashMap::new(),
            HashMap::new(),
            false,
        );
        assert_eq!(snapshot.coverage.financial_observations, 1);
        assert_eq!(snapshot.coverage.financials, 0);
        assert_eq!(snapshot.coverage.financials_review_required, 1);
        let msft = snapshot
            .items
            .iter()
            .find(|item| item.symbol == "MSFT")
            .unwrap();
        assert_eq!(msft.metrics.revenue_growth_percent, Some(17.8));
        assert!(!msft.metrics.financial_score_eligible);
        assert!(msft.dimensions.growth_quality.is_none());
        assert!(msft.dimensions.pricing_power.is_none());
        assert!(msft.dimensions.financial_quality.is_none());
    }

    #[test]
    fn exact_human_review_admits_sec_projection_to_dynamic_rating_factors() {
        let fact = financial_fact_from_sec_state(
            sec_financial_state(Some(17.8)),
            "sec_human_reviewed_for_rating",
            true,
            Vec::new(),
        )
        .unwrap();
        assert!(fact.score_eligible);
        assert_eq!(fact.review_status, "sec_human_reviewed_for_rating");

        let mut financials = HashMap::new();
        financials.insert("MSFT".to_string(), fact);
        let snapshot = snapshot_from_facts(
            parse_cards(),
            HashMap::new(),
            financials,
            HashMap::new(),
            HashMap::new(),
            false,
        );
        assert_eq!(snapshot.coverage.financial_observations, 1);
        assert_eq!(snapshot.coverage.financials, 1);
        assert_eq!(snapshot.coverage.financials_review_required, 0);
        let msft = snapshot
            .items
            .iter()
            .find(|item| item.symbol == "MSFT")
            .unwrap();
        assert!(msft.metrics.financial_score_eligible);
        assert!(msft.dimensions.growth_quality.is_some());
        assert!(msft.dimensions.pricing_power.is_some());
        assert!(msft.dimensions.financial_quality.is_some());
    }

    #[test]
    fn empty_sec_projection_does_not_create_a_financial_observation() {
        let mut state = sec_financial_state(None);
        state.gross_margin_percent = None;
        state.gross_margin_change_pp = None;
        state.ebit_margin_percent = None;
        state.accounts_receivable_growth_percent = None;
        state.accounts_payable_growth_percent = None;
        state.inventory_growth_percent = None;
        state.operating_cash_flow_growth_percent = None;
        state.capital_expenditure_growth_percent = None;
        assert!(
            financial_fact_from_sec_state(
                state,
                "sec_structured_pending_human_review",
                false,
                Vec::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn codex_simulation_fills_all_factors_without_claiming_live_data() {
        let cards = parse_cards();
        let (financials, forward) = simulation_inputs(&cards);
        let snapshot = snapshot_from_facts(
            cards,
            HashMap::new(),
            financials,
            HashMap::new(),
            forward,
            true,
        );
        assert_eq!(snapshot.data_status, "simulation");
        assert!(snapshot.simulation_note.contains("非真实"));
        assert_eq!(snapshot.coverage.financials, 52);
        assert_eq!(snapshot.coverage.valuations, 52);
        assert!(snapshot.items.iter().all(|item| item.factor_coverage == 8));
        assert!(snapshot.items.iter().all(|item| {
            item.data_status == "simulation"
                && item.valuation.is_none()
                && item.dimensions.valuation.is_some()
                && item.metrics.forward_metric_value.as_deref() == Some("Codex 模拟情景")
        }));
    }

    #[test]
    fn traffic_light_thresholds_are_stable() {
        assert_eq!(light_for_score(75.0), "green");
        assert_eq!(light_for_score(74.9), "yellow");
        assert_eq!(light_for_score(55.0), "yellow");
        assert_eq!(light_for_score(54.9), "red");
    }

    #[test]
    fn concurrent_refresh_is_coalesced_only_by_a_snapshot_created_after_its_request() {
        let request = Utc.with_ymd_and_hms(2026, 8, 13, 9, 0, 0).unwrap();
        assert!(!snapshot_satisfies_refresh_request(
            request - chrono::Duration::milliseconds(1),
            request
        ));
        assert!(snapshot_satisfies_refresh_request(request, request));
        assert!(snapshot_satisfies_refresh_request(
            request + chrono::Duration::milliseconds(1),
            request
        ));
    }

    #[test]
    fn nasdaq_fallback_quote_requires_matching_symbol_and_parses_display_fields() {
        let value = json!({
            "data": {
                "symbol": "SNDK",
                "primaryData": {
                    "lastSalePrice": "$1,271.05",
                    "percentageChange": "+2.35%",
                    "lastTradeTimestamp": "Aug 11, 2026"
                },
                "keyStats": {
                    "fiftyTwoWeekHighLow": { "value": "1,755.00 - 113.46" }
                }
            }
        });
        let quote = nasdaq_quote_from_value("SNDK", &value).expect("valid Nasdaq quote");
        assert_eq!(quote.price, 1271.05);
        assert_eq!(quote.change_percent, Some(2.35));
        assert_eq!(quote.year_low, Some(113.46));
        assert_eq!(quote.year_high, Some(1755.0));
        assert_eq!(quote.as_of.as_deref(), Some("Aug 11, 2026 · Nasdaq"));
        assert_eq!(quote.source, "Nasdaq 官方行情降级快照");
        assert!(nasdaq_quote_from_value("MU", &value).is_none());
    }

    #[test]
    fn display_range_accepts_common_dashes_but_rejects_invalid_ranges() {
        assert_eq!(
            parse_display_range("$113.46 – $1,255.00"),
            Some((Some(113.46), Some(1255.0)))
        );
        assert_eq!(
            parse_display_range("100 — 20"),
            Some((Some(20.0), Some(100.0)))
        );
        assert_eq!(parse_display_range("100 - 100"), None);
        assert_eq!(parse_display_range("N/A"), None);
    }

    #[test]
    fn nasdaq_history_keeps_point_in_time_path_volume_and_source() {
        let start = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let rows = (0..65)
            .map(|index| {
                let date = start + chrono::Duration::days(index);
                json!({
                    "date": date.format("%m/%d/%Y").to_string(),
                    "close": format!("${:.2}", 100.0 + index as f64),
                    "volume": format!("{}", 1_000 + index * 10),
                    "open": "$100.00",
                    "high": "$101.00",
                    "low": "$99.00"
                })
            })
            .rev()
            .collect::<Vec<_>>();
        let requested_through = start + chrono::Duration::days(64);
        let history = nasdaq_market_history_from_value(
            &json!({ "data": { "tradesTable": { "rows": rows } } }),
            "https://api.nasdaq.com/api/quote/SNDK/historical?assetclass=stocks",
            requested_through,
        )
        .expect("usable history");

        assert_eq!(history.policy_version, MARKET_HISTORY_POLICY_VERSION);
        assert_eq!(history.as_of, requested_through.to_string());
        assert_eq!(history.session_count, 65);
        assert_eq!(history.quality_status, "usable");
        assert!(history.quality_warnings.is_empty());
        assert_eq!(history.latest_close, 164.0);
        assert!(history.average_close_50.is_some());
        assert!(history.average_close_200.is_none());
        assert!(
            history
                .return_20_sessions_percent
                .is_some_and(|value| value > 0.0)
        );
        assert!(
            history
                .return_60_sessions_percent
                .is_some_and(|value| value > 0.0)
        );
        assert!(
            history
                .drawdown_from_60_session_high_percent
                .is_some_and(|value| value == 0.0)
        );
        assert!(
            history
                .recent_5_session_volume_vs_prior_55_percent
                .is_some()
        );
        assert!(history.source_url.starts_with("https://"));
        assert!(history.price_basis.contains("未声明复权口径"));
    }

    #[test]
    fn nasdaq_history_requires_review_for_a_split_like_jump() {
        let value = json!({
            "data": { "tradesTable": { "rows": [
                { "date": "08/12/2026", "close": "$200.00", "volume": "1000" },
                { "date": "08/11/2026", "close": "$100.00", "volume": "1000" }
            ] } }
        });
        let history = nasdaq_market_history_from_value(
            &value,
            "https://api.nasdaq.com/api/quote/TEST/historical?assetclass=stocks",
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
        )
        .expect("reviewable history");
        assert_eq!(history.quality_status, "review_required");
        assert!(
            history
                .quality_warnings
                .iter()
                .any(|warning| warning.contains("拆股"))
        );
        assert!(
            history
                .quality_warnings
                .iter()
                .any(|warning| warning.contains("60 日路径"))
        );
    }

    #[test]
    fn nasdaq_short_interest_keeps_two_period_change_and_ambiguous_interpretation() {
        let value = json!({
            "data": { "shortInterestTable": { "rows": [
                {
                    "settlementDate": "07/31/2026",
                    "interest": "6,823,953",
                    "avgDailyShareVolume": "18,073,238",
                    "daysToCover": 1
                },
                {
                    "settlementDate": "07/15/2026",
                    "interest": "7,857,690",
                    "avgDailyShareVolume": "14,000,000",
                    "daysToCover": "1"
                }
            ] } }
        });
        let summary = nasdaq_short_interest_from_value(
            &value,
            "https://api.nasdaq.com/api/quote/SNDK/short-interest?assetclass=stocks",
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
        )
        .expect("usable short-interest history");

        assert_eq!(summary.policy_version, SHORT_INTEREST_POLICY_VERSION);
        assert_eq!(summary.as_of, "2026-07-31");
        assert_eq!(summary.current_shares_short, 6_823_953.0);
        assert_eq!(summary.previous_shares_short, 7_857_690.0);
        assert!((summary.change_percent - -13.1557).abs() < 1e-4);
        assert_eq!(summary.days_to_cover, 1.0);
        assert_eq!(summary.quality_status, "usable");
        assert!(summary.source_url.starts_with("https://"));
        assert!(summary.interpretation.contains("不单独表示恐惧"));
    }

    #[test]
    fn nasdaq_short_interest_marks_stale_context_for_review() {
        let value = json!({
            "data": { "shortInterestTable": { "rows": [
                {
                    "settlementDate": "01/31/2026",
                    "interest": "1,100",
                    "avgDailyShareVolume": "500",
                    "daysToCover": 2
                },
                {
                    "settlementDate": "01/15/2026",
                    "interest": "1,000",
                    "avgDailyShareVolume": "500",
                    "daysToCover": 2
                }
            ] } }
        });
        let summary = nasdaq_short_interest_from_value(
            &value,
            "https://api.nasdaq.com/api/quote/TEST/short-interest?assetclass=stocks",
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
        )
        .expect("reviewable short-interest history");

        assert_eq!(summary.quality_status, "review_required");
        assert!(
            summary
                .quality_warnings
                .iter()
                .any(|warning| warning.contains("45日"))
        );
    }

    #[test]
    fn option_positioning_uses_a_standard_month_and_keeps_ratios_non_scored() {
        let requested = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
        let expiration = monthly_option_expiration(requested).expect("standard expiration");
        assert_eq!(expiration, NaiveDate::from_ymd_opt(2026, 9, 18).unwrap());
        let value = json!({
            "data": {
                "lastTrade": "LAST TRADE: $90.00 (AS OF AUG 12, 2026)",
                "totalRecord": 3,
                "table": { "rows": [
                    { "expirygroup": "September 18, 2026" },
                    {
                        "strike": "$85.00",
                        "c_Openinterest": "1,000",
                        "p_Openinterest": "1,500",
                        "c_Volume": "200",
                        "p_Volume": "250"
                    },
                    {
                        "strike": "$90.00",
                        "c_Openinterest": "500",
                        "p_Openinterest": "750",
                        "c_Volume": "100",
                        "p_Volume": "125"
                    }
                ] }
            }
        });
        let summary = nasdaq_options_positioning_from_value(
            &value,
            "https://api.nasdaq.com/api/quote/SNDK/option-chain?assetclass=stocks",
            requested,
            expiration,
        )
        .expect("usable option positioning");

        assert_eq!(summary.policy_version, OPTIONS_POSITIONING_POLICY_VERSION);
        assert_eq!(summary.as_of, "2026-08-12");
        assert_eq!(summary.days_to_expiration, 37);
        assert_eq!(summary.call_open_interest, 1_500.0);
        assert_eq!(summary.put_open_interest, 2_250.0);
        assert_eq!(summary.put_call_open_interest_ratio, Some(1.5));
        assert_eq!(summary.put_call_volume_ratio, Some(1.25));
        assert_eq!(summary.quality_status, "usable");
        assert!(summary.interpretation.contains("不等同方向判断"));
        assert!(summary.interpretation.contains("不推算"));
    }

    #[test]
    fn option_positioning_requires_complete_chain_coverage() {
        let requested = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
        let expiration = NaiveDate::from_ymd_opt(2026, 9, 18).unwrap();
        let summary = nasdaq_options_positioning_from_value(
            &json!({ "data": {
                "lastTrade": "LAST TRADE: $90.00 (AS OF AUG 12, 2026)",
                "totalRecord": "20",
                "table": { "rows": [
                    { "expirygroup": "September 18, 2026" },
                    { "strike": "$90.00", "c_Openinterest": "10", "p_Openinterest": "20" }
                ] }
            }}),
            "https://api.nasdaq.com/api/quote/TEST/option-chain?assetclass=stocks",
            requested,
            expiration,
        )
        .expect("reviewable option positioning");
        assert_eq!(summary.quality_status, "review_required");
        assert!(
            summary
                .quality_warnings
                .iter()
                .any(|warning| warning.contains("截断"))
        );
    }

    #[test]
    fn news_attention_counts_publication_activity_without_claiming_sentiment() {
        let requested = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
        let value = json!({ "data": { "rows": [
            {
                "id": 1,
                "created": "Aug 12, 2026",
                "publisher": "Publisher A",
                "title": "One",
                "primarysymbol": "SNDK",
                "related_symbols": ["sndk|stocks"]
            },
            {
                "id": 2,
                "created": "Aug 10, 2026",
                "publisher": "Publisher B",
                "title": "Two",
                "primarysymbol": "SNDK",
                "related_symbols": ["SNDK|stocks"]
            },
            {
                "id": 3,
                "created": "Aug 05, 2026",
                "publisher": "Publisher A",
                "title": "Three",
                "primarysymbol": "SNDK",
                "related_symbols": ["SNDK|stocks"]
            },
            {
                "id": 3,
                "created": "Aug 05, 2026",
                "publisher": "Publisher A",
                "title": "Duplicate",
                "primarysymbol": "SNDK",
                "related_symbols": ["SNDK|stocks"]
            },
            {
                "id": 4,
                "created": "Aug 12, 2026",
                "publisher": "Other",
                "title": "Wrong symbol",
                "primarysymbol": "MU",
                "related_symbols": ["MU|stocks"]
            }
        ] }});
        let summary = nasdaq_news_attention_from_value(
            "SNDK",
            &value,
            "https://www.nasdaq.com/api/news/topic/articlebysymbol?q=SNDK%7Cstocks&limit=100",
            requested,
        )
        .expect("usable attention context");

        assert_eq!(summary.policy_version, NEWS_ATTENTION_POLICY_VERSION);
        assert_eq!(summary.recent_article_count, 2);
        assert_eq!(summary.prior_article_count, 1);
        assert_eq!(summary.unique_publishers, 2);
        assert_eq!(summary.observed_article_count, 3);
        assert_eq!(summary.quality_status, "usable");
        assert!(!summary.truncated_window);
        assert!(summary.interpretation.contains("不代表文章观点"));
        assert!(summary.interpretation.contains("不单独形成方向"));
    }

    #[test]
    fn institutional_holdings_reconcile_13f_categories_and_disclose_mixed_periods() {
        let value = json!({ "data": {
            "ownershipSummary": {
                "SharesOutstandingPCT": { "value": "94.07%" }
            },
            "holdingsTransactions": {
                "institutionalHolders": "1,802 Institutional Holders",
                "sharesHeld": "139,304,332 Total Shares Held",
                "totalRecords": 1802,
                "table": { "rows": [
                    { "date": "06/30/2026" },
                    { "date": "06/30/2026" },
                    { "date": "03/31/2026" },
                    { "date": "12/31/2025" }
                ] }
            },
            "activePositions": { "rows": [
                { "positions": "Increased Positions", "holders": "1,234", "shares": "40,143,212" },
                { "positions": "Decreased Positions", "holders": "484", "shares": "25,719,662" },
                { "positions": "Held Positions", "holders": "84", "shares": "73,441,458" }
            ] },
            "newSoldOutPositions": { "rows": [
                { "positions": "New Positions", "holders": "754", "shares": "22,544,032" },
                { "positions": "Sold Out Positions", "holders": "73", "shares": "751,055" }
            ] }
        }});
        let summary = nasdaq_institutional_holdings_from_value(
            &value,
            "https://api.nasdaq.com/api/company/SNDK/institutional-holdings?limit=50",
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
        )
        .expect("usable institutional holdings");

        assert_eq!(
            summary.policy_version,
            INSTITUTIONAL_HOLDINGS_POLICY_VERSION
        );
        assert_eq!(summary.institutional_ownership_percent, 94.07);
        assert_eq!(summary.institutional_holders, 1_802);
        assert_eq!(summary.total_shares_held, 139_304_332.0);
        assert_eq!(summary.report_period_count, 3);
        assert_eq!(summary.earliest_report_period, "2025-12-31");
        assert_eq!(summary.latest_report_period, "2026-06-30");
        assert_eq!(summary.latest_period_rows_in_sample, 2);
        assert!(summary.holder_table_truncated);
        assert_eq!(summary.quality_status, "usable");
        assert!(summary.quality_warnings.is_empty());
        assert!(summary.interpretation.contains("45天"));
        assert!(summary.interpretation.contains("不代表机构今天买卖"));
    }

    #[test]
    fn analyst_consensus_reconciles_distribution_and_discloses_target_limitations() {
        let value = json!({ "data": {
            "consensusOverview": {
                "lowPriceTarget": 130.0,
                "highPriceTarget": 305.0,
                "priceTarget": 218.125,
                "buy": 14,
                "sell": 0,
                "hold": 2
            },
            "historicalConsensus": [
                { "z": { "date": "07/01/2026", "buy": 13, "hold": 3, "sell": 0 } },
                { "z": { "date": "08/01/2026", "buy": 14, "hold": 2, "sell": 0 } }
            ]
        }});
        let summary = nasdaq_analyst_consensus_from_value(
            &value,
            "https://api.nasdaq.com/api/analyst/SNDK/targetprice",
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
        )
        .expect("usable analyst consensus");

        assert_eq!(summary.policy_version, ANALYST_CONSENSUS_POLICY_VERSION);
        assert_eq!(summary.recommendation_count, 16);
        assert_eq!(summary.dominant_rating, "买入");
        assert_eq!(summary.dominant_count, 14);
        assert_eq!(summary.dominant_share_percent, 87.5);
        assert_eq!(summary.consensus_target_price, 218.125);
        assert_eq!(summary.target_range_width_percent, 80.2292);
        assert_eq!(summary.historical_month_count, 2);
        assert_eq!(summary.quality_status, "usable");
        assert!(summary.quality_warnings.is_empty());
        assert!(summary.interpretation.contains("目标价贡献者数量"));
        assert!(summary.interpretation.contains("不进入评分"));
    }

    #[test]
    fn analyst_consensus_small_or_unordered_sample_requires_review() {
        let value = json!({ "data": {
            "consensusOverview": {
                "lowPriceTarget": 120.0,
                "highPriceTarget": 100.0,
                "priceTarget": 110.0,
                "buy": 1,
                "hold": 1,
                "sell": 0
            },
            "historicalConsensus": []
        }});
        let summary = nasdaq_analyst_consensus_from_value(
            &value,
            "https://api.nasdaq.com/api/analyst/TEST/targetprice",
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
        )
        .expect("reviewable analyst consensus");

        assert_eq!(summary.dominant_rating, "并列");
        assert_eq!(summary.quality_status, "review_required");
        assert_eq!(summary.quality_warnings.len(), 3);
    }

    #[test]
    fn institutional_holdings_category_mismatch_is_review_required() {
        let value = json!({ "data": {
            "ownershipSummary": {
                "SharesOutstandingPCT": { "value": "50%" }
            },
            "holdingsTransactions": {
                "institutionalHolders": "3 Institutional Holders",
                "sharesHeld": "100 Total Shares Held",
                "totalRecords": 3,
                "table": { "rows": [
                    { "date": "06/30/2026" },
                    { "date": "06/30/2026" },
                    { "date": "06/30/2026" }
                ] }
            },
            "activePositions": { "rows": [
                { "positions": "Increased Positions", "holders": "1", "shares": "50" },
                { "positions": "Decreased Positions", "holders": "1", "shares": "25" },
                { "positions": "Held Positions", "holders": "0", "shares": "20" }
            ] },
            "newSoldOutPositions": { "rows": [
                { "positions": "New Positions", "holders": "1", "shares": "10" },
                { "positions": "Sold Out Positions", "holders": "1", "shares": "5" }
            ] }
        }});
        let summary = nasdaq_institutional_holdings_from_value(
            &value,
            "https://api.nasdaq.com/api/company/TEST/institutional-holdings?limit=50",
            NaiveDate::from_ymd_opt(2026, 8, 12).unwrap(),
        )
        .expect("reviewable institutional holdings");

        assert_eq!(summary.quality_status, "review_required");
        assert!(
            summary
                .quality_warnings
                .iter()
                .any(|warning| warning.contains("持有人"))
        );
        assert!(
            summary
                .quality_warnings
                .iter()
                .any(|warning| warning.contains("持股总数"))
        );
    }

    #[test]
    fn transcript_only_baseline_never_claims_a_daily_traffic_light() {
        let snapshot = baseline_snapshot();
        assert_eq!(snapshot.data_status, "transcript_only");
        assert!(snapshot.items.iter().all(|item| item.light == "unknown"));
        assert!(snapshot.items.iter().all(|item| item.factor_coverage == 3));
    }

    #[test]
    fn research_grades_use_explicit_non_perfection_anchors() {
        assert_eq!(structural_anchor(1.0), 25.0);
        assert_eq!(structural_anchor(2.0), 45.0);
        assert_eq!(structural_anchor(3.0), 60.0);
        assert_eq!(structural_anchor(4.0), 75.0);
        assert_eq!(structural_anchor(5.0), 90.0);
        assert!(
            baseline_snapshot()
                .items
                .iter()
                .flat_map(|item| [
                    item.dimensions.moat,
                    item.dimensions.scarcity,
                    item.dimensions.fundamentals,
                    item.dimensions.visibility,
                ])
                .all(|score| score <= 90.0)
        );
    }

    #[test]
    fn weighted_score_uses_documented_weights() {
        let dimensions = RatingDimensions {
            moat: 100.0,
            scarcity: 80.0,
            fundamentals: 60.0,
            visibility: 40.0,
            growth_quality: Some(60.0),
            pricing_power: Some(40.0),
            financial_quality: Some(20.0),
            valuation: Some(20.0),
            market_confirmation: 0.0,
            timing: Some(0.0),
        };
        assert_eq!(weighted_score(&dimensions), 51.0);
    }

    #[test]
    fn missing_daily_valuation_is_removed_and_remaining_weights_are_normalized() {
        let dimensions = RatingDimensions {
            moat: 100.0,
            scarcity: 80.0,
            fundamentals: 60.0,
            visibility: 40.0,
            growth_quality: Some(60.0),
            pricing_power: Some(40.0),
            financial_quality: Some(20.0),
            valuation: None,
            market_confirmation: 0.0,
            timing: Some(0.0),
        };
        assert_eq!(weighted_score(&dimensions), 56.5);
    }

    #[test]
    fn legacy_snapshot_cannot_keep_a_numeric_valuation_without_daily_scenarios() {
        let mut snapshot = baseline_snapshot();
        snapshot.methodology_version = "hone-company-rating-v1".to_string();
        snapshot.items[0].dimensions.valuation = Some(100.0);
        snapshot.items[0].score = weighted_score(&snapshot.items[0].dimensions);

        let normalized = normalize_snapshot_contract(snapshot);
        let item = &normalized.items[0];
        assert_eq!(normalized.methodology_version, METHODOLOGY_VERSION);
        assert_eq!(normalized.coverage.valuations, 0);
        assert_eq!(item.dimensions.valuation, None);
        assert_eq!(item.score, weighted_score(&item.dimensions));
        assert!(
            item.valuation_unavailable_reason
                .contains("估值不参与综合分")
        );
    }

    #[test]
    fn verified_daily_valuation_builds_three_cases_and_current_position() {
        let input = DailyValuationInput {
            symbol: "TSM".to_string(),
            as_of: "2026-08-11".to_string(),
            currency: "USD".to_string(),
            bear_case: 180.0,
            base_case: 220.0,
            bull_case: 280.0,
            current_price: 240.0,
            probability_weighted_value: Some(225.6),
            expected_upside_percent: Some(-6.0),
            method_count: 3,
            confidence: "high".to_string(),
            method: "反向 DCF 与 forward P/E 交叉验证".to_string(),
            assumptions: vec!["增长、毛利和资本开支均有明确来源".to_string()],
            sources: vec!["公司 IR".to_string(), "SEC".to_string()],
            review_status: "verified".to_string(),
            input_mode: String::new(),
            valuation_review_id: None,
            valuation_input_fingerprint_sha256: None,
            valuation_financial_evidence_fingerprint_sha256: None,
            valuation_input_as_of: None,
        };
        let (_, valuation) = validated_daily_valuation(
            input,
            Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
            "2026-08-11",
            "verified",
            false,
            None,
        )
        .expect("valid valuation");
        assert_eq!(valuation.current_position, "基准—乐观之间");
        assert_eq!(valuation.position_percent, 60.0);
        assert_eq!(valuation_health_score(&valuation), 41.0);
    }

    #[test]
    fn unverified_or_unordered_valuation_is_rejected() {
        let input = DailyValuationInput {
            symbol: "TSM".to_string(),
            as_of: "2026-08-11".to_string(),
            currency: "USD".to_string(),
            bear_case: 280.0,
            base_case: 220.0,
            bull_case: 180.0,
            current_price: 240.0,
            probability_weighted_value: None,
            expected_upside_percent: None,
            method_count: 0,
            confidence: String::new(),
            method: "旧估值".to_string(),
            assumptions: vec!["旧假设".to_string()],
            sources: vec!["单一来源".to_string()],
            review_status: "draft".to_string(),
            input_mode: String::new(),
            valuation_review_id: None,
            valuation_input_fingerprint_sha256: None,
            valuation_financial_evidence_fingerprint_sha256: None,
            valuation_input_as_of: None,
        };
        assert!(
            validated_daily_valuation(
                input,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "verified",
                false,
                None,
            )
            .is_none()
        );
    }

    #[test]
    fn computed_hone_valuation_uses_a_separate_review_contract() {
        let input = DailyValuationInput {
            symbol: "NVDA".to_string(),
            as_of: "2026-08-11".to_string(),
            currency: "USD".to_string(),
            bear_case: 120.0,
            base_case: 160.0,
            bull_case: 210.0,
            current_price: 150.0,
            probability_weighted_value: Some(165.0),
            expected_upside_percent: Some(10.0),
            method_count: 3,
            confidence: "high".to_string(),
            method: "HONE 多方法估值".to_string(),
            assumptions: vec!["模型参数可复算".to_string()],
            sources: vec!["FMP cash flow".to_string(), "FMP estimates".to_string()],
            review_status: "computed".to_string(),
            input_mode: "provider_bundle".to_string(),
            valuation_review_id: None,
            valuation_input_fingerprint_sha256: None,
            valuation_financial_evidence_fingerprint_sha256: None,
            valuation_input_as_of: None,
        };
        assert!(
            validated_daily_valuation(
                input,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "computed",
                true,
                None,
            )
            .is_some()
        );
    }

    #[test]
    fn reviewed_sec_valuation_requires_exact_fresh_authorization_binding() {
        let input = DailyValuationInput {
            symbol: "SNDK".to_string(),
            as_of: "2026-08-11".to_string(),
            currency: "USD".to_string(),
            bear_case: 90.0,
            base_case: 120.0,
            bull_case: 160.0,
            current_price: 110.0,
            probability_weighted_value: Some(123.0),
            expected_upside_percent: Some(11.8),
            method_count: 3,
            confidence: "high".to_string(),
            method: "HONE 多方法估值".to_string(),
            assumptions: vec!["SEC 一手事实与经复核中周期输入可复算".to_string()],
            sources: vec!["SEC filing".to_string(), "公司 IR".to_string()],
            review_status: "computed".to_string(),
            input_mode: "sec_reviewed_supplemental_packet".to_string(),
            valuation_review_id: Some("SNDK-valuation-input-review-1".to_string()),
            valuation_input_fingerprint_sha256: Some("a".repeat(64)),
            valuation_financial_evidence_fingerprint_sha256: Some("b".repeat(64)),
            valuation_input_as_of: Some("2026-08-08".to_string()),
        };
        let binding = ValuationAuthorizationBinding {
            review_id: "SNDK-valuation-input-review-1".to_string(),
            input_fingerprint_sha256: "a".repeat(64),
            financial_evidence_fingerprint_sha256: "b".repeat(64),
            input_as_of: "2026-08-08".to_string(),
        };
        assert!(
            validated_daily_valuation(
                input.clone(),
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "computed",
                true,
                Some(&binding),
            )
            .is_some()
        );

        let mut missing_review = input.clone();
        missing_review.valuation_review_id = None;
        assert!(
            validated_daily_valuation(
                missing_review,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "computed",
                true,
                Some(&binding),
            )
            .is_none()
        );

        let mut stale = input.clone();
        stale.valuation_input_as_of = Some("2026-08-03".to_string());
        assert!(
            validated_daily_valuation(
                stale,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "computed",
                true,
                Some(&binding),
            )
            .is_none()
        );

        let mut tampered = input;
        tampered.valuation_input_fingerprint_sha256 = Some("c".repeat(64));
        assert!(
            validated_daily_valuation(
                tampered,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "computed",
                true,
                Some(&binding),
            )
            .is_none()
        );
    }

    #[test]
    fn computed_valuation_requires_two_methods_and_current_confidence() {
        let input = DailyValuationInput {
            symbol: "NVDA".to_string(),
            as_of: "2026-08-11".to_string(),
            currency: "USD".to_string(),
            bear_case: 120.0,
            base_case: 160.0,
            bull_case: 210.0,
            current_price: 150.0,
            probability_weighted_value: Some(165.0),
            expected_upside_percent: Some(10.0),
            method_count: 1,
            confidence: "low".to_string(),
            method: "只有 DCF".to_string(),
            assumptions: vec!["模型参数可复算".to_string()],
            sources: vec!["FMP cash flow".to_string(), "FMP estimates".to_string()],
            review_status: "computed".to_string(),
            input_mode: "provider_bundle".to_string(),
            valuation_review_id: None,
            valuation_input_fingerprint_sha256: None,
            valuation_financial_evidence_fingerprint_sha256: None,
            valuation_input_as_of: None,
        };
        assert!(
            validated_daily_valuation(
                input,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "computed",
                true,
                None,
            )
            .is_none()
        );
    }

    #[test]
    fn financial_parser_uses_year_over_year_quarter_when_available() {
        let value = json!([
            {"date":"2026-06-30","revenue":120.0,"grossProfit":60.0,"netIncome":5.0},
            {"date":"2026-03-31","revenue":110.0,"grossProfit":52.0,"netIncome":3.0},
            {"date":"2025-12-31","revenue":105.0,"grossProfit":48.0,"netIncome":1.0},
            {"date":"2025-09-30","revenue":102.0,"grossProfit":47.0,"netIncome":1.0},
            {"date":"2025-06-30","revenue":100.0,"grossProfit":45.0,"netIncome":-1.0}
        ]);
        let fact = financial_from_value(&value).unwrap();
        assert_eq!(fact.as_of.as_deref(), Some("2026-06-30"));
        assert_eq!(fact.revenue_growth_percent, Some(20.0));
        assert_eq!(fact.gross_margin_percent, Some(50.0));
        assert_eq!(fact.gross_margin_change_pp, Some(5.0));
    }

    #[test]
    fn margin_expansion_improves_pricing_power() {
        let peers = vec![
            FinancialFact {
                gross_margin_percent: Some(40.0),
                ..FinancialFact::default()
            },
            FinancialFact {
                gross_margin_percent: Some(50.0),
                ..FinancialFact::default()
            },
            FinancialFact {
                gross_margin_percent: Some(60.0),
                ..FinancialFact::default()
            },
        ];
        let peer_refs = peers.iter().collect::<Vec<_>>();
        let contracting = FinancialFact {
            gross_margin_percent: Some(50.0),
            gross_margin_change_pp: Some(-3.0),
            ..FinancialFact::default()
        };
        let expanding = FinancialFact {
            gross_margin_percent: Some(50.0),
            gross_margin_change_pp: Some(3.0),
            ..FinancialFact::default()
        };
        assert!(
            pricing_power_score(&expanding, &peer_refs)
                > pricing_power_score(&contracting, &peer_refs)
        );
    }

    #[test]
    fn cash_generation_and_net_cash_improve_financial_quality() {
        let peers = vec![
            FinancialFact {
                fcf_margin_percent: Some(-5.0),
                ..FinancialFact::default()
            },
            FinancialFact {
                fcf_margin_percent: Some(10.0),
                ..FinancialFact::default()
            },
            FinancialFact {
                fcf_margin_percent: Some(30.0),
                ..FinancialFact::default()
            },
        ];
        let peer_refs = peers.iter().collect::<Vec<_>>();
        let weak = FinancialFact {
            fcf_margin_percent: Some(-5.0),
            ebit_margin_percent: Some(-2.0),
            net_cash_to_revenue_percent: Some(-60.0),
            ..FinancialFact::default()
        };
        let strong = FinancialFact {
            fcf_margin_percent: Some(30.0),
            ebit_margin_percent: Some(25.0),
            net_cash_to_revenue_percent: Some(40.0),
            ..FinancialFact::default()
        };
        assert!(
            financial_quality_score(&strong, &peer_refs)
                > financial_quality_score(&weak, &peer_refs)
        );
    }

    #[test]
    fn severe_financial_weakness_caps_the_rating_at_red() {
        let dimensions = RatingDimensions {
            moat: 90.0,
            scarcity: 90.0,
            fundamentals: 90.0,
            visibility: 90.0,
            growth_quality: Some(95.0),
            pricing_power: Some(90.0),
            financial_quality: Some(20.0),
            valuation: Some(90.0),
            market_confirmation: 90.0,
            timing: Some(90.0),
        };
        let (score, reason) = apply_score_caps(weighted_score(&dimensions), &dimensions);
        assert_eq!(score, 54.9);
        assert!(reason.contains("现金流"));
    }

    #[test]
    fn verified_forward_growth_can_adjust_visibility_but_absolute_value_alone_cannot() {
        assert_eq!(visibility_score(75.0, None), 75.0);
        assert!(visibility_score(75.0, Some(35.0)) > 75.0);
        assert!(visibility_score(75.0, Some(-20.0)) < 75.0);
    }

    #[test]
    fn next_refresh_is_1930_beijing() {
        let now = Utc.with_ymd_and_hms(2026, 8, 10, 8, 0, 0).unwrap();
        let next = next_refresh(now).with_timezone(&Shanghai);
        assert_eq!((next.hour(), next.minute()), (19, 30));
        assert_eq!(next.day(), 10);
    }
}
