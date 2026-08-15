//! Daily transcript-informed company ratings for the public chat workspace.
//!
//! The durable snapshot is refreshed at 19:30 Asia/Shanghai. FMP enriches the
//! research baseline when configured; missing upstream data never becomes a
//! zero and is surfaced through `data_status`, coverage, and confidence.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use chrono_tz::Asia::Shanghai;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};

use crate::routes::public_finance_calendar::fetch_fmp_json_once;
use crate::state::AppState;

const CARDS_JSON: &str =
    include_str!("../../../../skills/company-thesis-ratings/references/company-cards.json");
const REFRESH_HOUR: u32 = 19;
const REFRESH_MINUTE: u32 = 30;
const STALE_AFTER_HOURS: i64 = 36;
const METHODOLOGY_VERSION: &str = "hone-company-rating-v5";

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
    pub financial_as_of: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct RatingCoverage {
    pub companies: usize,
    pub quotes: usize,
    pub financials: usize,
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
    timestamp: Option<i64>,
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

/// Compact overview projection of the latest stored snapshot. `None` when no
/// snapshot file exists yet; the aggregator renders a waiting card instead.
pub(crate) async fn overview_card(
    state: &AppState,
) -> Option<crate::routes::research_overview::OverviewCard> {
    let snapshot = read_snapshot(state).await?;
    let snapshot = mark_stale_if_needed(normalize_snapshot_contract(snapshot));
    let mut card = crate::routes::research_overview::OverviewCard::waiting(
        "company-ratings",
        "公司评级",
        "52 家研究基线",
    );
    card.report_date = Some(
        snapshot
            .generated_at
            .with_timezone(&Shanghai)
            .format("%Y-%m-%d")
            .to_string(),
    );
    card.status = match snapshot.data_status.as_str() {
        "live" | "partial" | "stale" => snapshot.data_status.clone(),
        // simulation / transcript_only are research-baseline modes.
        _ => "baseline".to_string(),
    };
    card.metric = Some(format!("{} 家覆盖", snapshot.coverage.companies));
    card.generated_at = Some(snapshot.generated_at);
    Some(card)
}

/// Start an immediate best-effort refresh, then wait for 19:30 Beijing each day.
pub(crate) async fn company_rating_worker(state: Arc<AppState>) {
    refresh_and_store(&state).await;
    loop {
        let next = next_refresh(Utc::now());
        let wait = (next - Utc::now())
            .to_std()
            .unwrap_or_else(|_| Duration::from_secs(60));
        info!(next_refresh = %next, "company rating worker waiting");
        tokio::time::sleep(wait).await;
        refresh_and_store(&state).await;
    }
}

pub(crate) async fn refresh_and_store(state: &AppState) {
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
        info!(
            status = %snapshot.data_status,
            quotes = snapshot.coverage.quotes,
            financials = snapshot.coverage.financials,
            "company rating snapshot refreshed"
        );
    }
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
    let financials = read_verified_fundamentals(state).await;
    let forward_evidence = read_verified_forward_evidence(state).await;
    let pool = state.core.config.fmp.effective_key_pool();
    if pool.keys().is_empty() {
        return snapshot_from_facts(
            cards,
            HashMap::new(),
            financials,
            valuations,
            forward_evidence,
            false,
        );
    }

    let symbols = cards
        .iter()
        .map(|card| card.symbol.clone())
        .collect::<Vec<_>>();
    let quotes = fetch_quotes(state, pool.keys(), &symbols)
        .await
        .unwrap_or_else(|error| {
            warn!("company rating quotes unavailable: {error}");
            HashMap::new()
        });
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
    let financial_count = financials.len();
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
    if let Some(fact) = financial {
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
            financial.is_some(),
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
    if quote.is_some() {
        data_sources.push("FMP 行情快照".to_string());
    }
    if financial.is_some() {
        data_sources.push("FMP 最近季度财务报表".to_string());
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
            valuation_unavailable_reason(valuation.as_ref(), quote, financial)
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
        .filter(|(symbol, _)| themes.get(*symbol).is_some_and(|value| value == theme))
        .map(|(_, fact)| fact)
        .collect::<Vec<_>>();
    if themed.len() >= 5 {
        themed
    } else {
        financials.values().collect()
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
        financial_as_of: fact.and_then(|value| value.as_of.clone()),
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
    fact.timestamp
        .and_then(|seconds| DateTime::from_timestamp(seconds, 0))
        .map(|value| value.to_rfc3339())
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
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
                    timestamp: item.get("timestamp").and_then(Value::as_i64),
                },
            ))
        })
        .collect()
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
    crate::routes::research_store::data_root(state)
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
                },
            ))
        })
        .collect()
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
    let expected_review_status = match file.framework_version.as_str() {
        "hari-invest-v1" => "verified",
        "hone-valuation-v2" => "computed",
        _ => "",
    };
    if expected_review_status.is_empty()
        || file.report_date != today
        || file.generated_at > now + chrono::Duration::minutes(5)
        || now - file.generated_at > chrono::Duration::hours(STALE_AFTER_HOURS)
    {
        warn!(path = %path.display(), "daily valuation file failed freshness or framework validation");
        return HashMap::new();
    }

    file.items
        .into_iter()
        .filter_map(|item| {
            validated_daily_valuation(item, file.generated_at, &today, expected_review_status)
        })
        .collect()
}

fn validated_daily_valuation(
    item: DailyValuationInput,
    generated_at: DateTime<Utc>,
    report_date: &str,
    expected_review_status: &str,
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
    if item.review_status != expected_review_status
        || item.as_of != report_date
        || item.symbol.trim().is_empty()
        || item.currency.trim().is_empty()
        || item.method.trim().is_empty()
        || item.assumptions.is_empty()
        || item.sources.len() < 2
        || (expected_review_status == "computed"
            && (item.method_count < 2 || !matches!(item.confidence.as_str(), "high" | "medium")))
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
    crate::routes::research_store::write_json_atomic(&snapshot_path(state), snapshot)
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
    crate::routes::research_store::next_beijing_refresh(now, REFRESH_HOUR, REFRESH_MINUTE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike};
    use serde_json::json;

    #[test]
    fn embedded_cards_are_valid_and_cover_current_universe() {
        let cards = parse_cards();
        assert_eq!(cards.len(), 52);
        assert!(cards.iter().any(|card| card.symbol == "SNDK"));
        assert!(cards.iter().any(|card| card.symbol == "SKHY"));
        assert!(!cards.iter().any(|card| card.name == "量子板块"));
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
        };
        let (_, valuation) = validated_daily_valuation(
            input,
            Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
            "2026-08-11",
            "verified",
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
        };
        assert!(
            validated_daily_valuation(
                input,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "verified",
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
        };
        assert!(
            validated_daily_valuation(
                input,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "computed",
            )
            .is_some()
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
        };
        assert!(
            validated_daily_valuation(
                input,
                Utc.with_ymd_and_hms(2026, 8, 11, 10, 0, 0).unwrap(),
                "2026-08-11",
                "computed",
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
