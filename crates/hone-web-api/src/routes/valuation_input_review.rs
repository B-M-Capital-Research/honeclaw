//! Independent human admission for SEC-backed valuation inputs.
//!
//! Rating-quality review and valuation-use review are deliberately separate.
//! This module freezes an exact SEC evidence fingerprint together with the
//! supplemental inputs that are required to turn filed facts into a
//! multi-method valuation.  Approval is short lived, fail closed, and never
//! grants training, reward, portfolio, shadow, or trading authority.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::financial_evidence_review::financial_evidence_fingerprint;
use super::investment_decisions::FinancialVerificationState;
use crate::state::AppState;

pub(crate) const VALUATION_INPUT_REVIEW_SCHEMA_VERSION: &str = "hone-sec-valuation-input-review-v1";
pub(crate) const VALUATION_INPUT_REVIEW_POLICY_VERSION: &str =
    "hone-sec-valuation-input-admission-v1";
const MAX_INPUT_AGE_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ValuationInputReviewVerdict {
    ApprovedForValuation,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ValuationInputReviewConfirmations {
    pub official_sources_opened: bool,
    pub sec_financial_values_recomputed: bool,
    pub diluted_share_count_and_corporate_actions_verified: bool,
    pub complete_net_cash_or_debt_verified: bool,
    pub forward_or_midcycle_inputs_verified: bool,
    pub cyclicality_and_normalization_checked: bool,
    pub cross_method_comparability_checked: bool,
    pub no_unresolved_material_issue: bool,
}

impl ValuationInputReviewConfirmations {
    fn all_confirmed(&self) -> bool {
        self.official_sources_opened
            && self.sec_financial_values_recomputed
            && self.diluted_share_count_and_corporate_actions_verified
            && self.complete_net_cash_or_debt_verified
            && self.forward_or_midcycle_inputs_verified
            && self.cyclicality_and_normalization_checked
            && self.cross_method_comparability_checked
            && self.no_unresolved_material_issue
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(crate) struct SupplementalValuationInputs {
    pub input_as_of: String,
    pub currency: String,
    pub diluted_shares_millions: Option<f64>,
    pub complete_net_cash_millions: Option<f64>,
    pub forward_eps: Option<f64>,
    pub forward_revenue_millions: Option<f64>,
    pub normalized_ebit_margin_percent: Option<f64>,
    #[serde(default)]
    pub annual_fcf_history_millions: Vec<f64>,
    #[serde(default)]
    pub source_urls: Vec<String>,
    pub source_note: String,
}

impl SupplementalValuationInputs {
    pub(crate) fn prepared_method_ids(&self) -> Vec<String> {
        let mut methods = Vec::new();
        if self.forward_eps.is_some_and(|value| value > 0.0) {
            methods.push("forward_pe".to_string());
        }
        if self
            .forward_revenue_millions
            .is_some_and(|value| value > 0.0)
            && self
                .normalized_ebit_margin_percent
                .is_some_and(|value| value > 0.0)
        {
            methods.push("ev_ebit".to_string());
        }
        if self.annual_fcf_history_millions.len() >= 3
            && self
                .annual_fcf_history_millions
                .iter()
                .filter(|value| value.is_finite() && **value > 0.0)
                .count()
                >= 2
        {
            methods.push("cycle_adjusted_dcf".to_string());
        }
        methods
    }

    pub(crate) fn missing_inputs(&self) -> Vec<String> {
        let mut missing = Vec::new();
        if self
            .diluted_shares_millions
            .is_none_or(|value| !value.is_finite() || value <= 0.0)
        {
            missing.push("经来源核验的稀释后流通股本".to_string());
        }
        if self
            .complete_net_cash_millions
            .is_none_or(|value| !value.is_finite())
        {
            missing.push("经来源核验的完整净现金或净负债".to_string());
        }
        if self.prepared_method_ids().len() < 2 {
            missing.push("至少两种完整估值方法输入（前瞻 P/E、EV/EBIT、周期调整 DCF）".to_string());
        }
        if self.source_urls.is_empty() {
            missing.push("补充输入的一手或可审计来源链接".to_string());
        }
        if self.source_note.trim().len() < 8 {
            missing.push("补充输入的口径、期间与推导说明".to_string());
        }
        missing
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValuationInputReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub previous_review_id: Option<String>,
    pub symbol: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub financial_evidence_fingerprint_sha256: String,
    pub financial_evidence_snapshot: FinancialVerificationState,
    pub input_fingerprint_sha256: String,
    pub supplemental_inputs: SupplementalValuationInputs,
    pub verdict: ValuationInputReviewVerdict,
    pub rationale: String,
    pub confirmations: ValuationInputReviewConfirmations,
    pub valuation_authorized: bool,
    pub rating_factor_authorized: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub portfolio_action_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub trade_authorized: bool,
    pub old_wang_logic_confirmed: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ValuationInputReviewRequest {
    pub expected_review_id: Option<String>,
    pub expected_financial_evidence_fingerprint_sha256: String,
    pub verdict: ValuationInputReviewVerdict,
    pub rationale: String,
    #[serde(default)]
    pub confirmations: ValuationInputReviewConfirmations,
    #[serde(default)]
    pub supplemental_inputs: SupplementalValuationInputs,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ValuationInputReviewCandidate {
    pub symbol: String,
    pub financial_evidence_fingerprint_sha256: String,
    pub financial_evidence: FinancialVerificationState,
    pub review_status: String,
    pub valuation_authorized: bool,
    pub blocking_reasons: Vec<String>,
    pub prepared_method_ids: Vec<String>,
    pub latest_review: Option<ValuationInputReviewRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ValuationInputReviewResponse {
    pub schema_version: String,
    pub policy_version: String,
    pub generated_at: DateTime<Utc>,
    pub observed: usize,
    pub pending: usize,
    pub authorized: usize,
    pub stale: usize,
    pub candidates: Vec<ValuationInputReviewCandidate>,
    pub scope: String,
    pub rating_factor_authorized: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub portfolio_action_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub trade_authorized: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ValuationInputReviewQuery {
    pub symbol: Option<String>,
}

#[derive(Debug)]
struct ValuationReviewLock {
    path: PathBuf,
}

impl Drop for ValuationReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub(crate) async fn handle_get_valuation_input_reviews(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<ValuationInputReviewQuery>,
) -> Response {
    if let Err(response) = super::public_admin::require_public_admin_for_read(&state, &headers) {
        return response;
    }
    let symbols = match normalized_query_symbols(&state, query.symbol).await {
        Ok(symbols) => symbols,
        Err(error) => return (StatusCode::BAD_REQUEST, error).into_response(),
    };
    match review_response(&state, &symbols).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => {
            warn!(%error, "valuation input review read failed");
            (
                StatusCode::CONFLICT,
                "估值输入复核链不可用，请先检查审计记录",
            )
                .into_response()
        }
    }
}

pub(crate) async fn handle_review_valuation_inputs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(symbol): AxumPath<String>,
    Json(mut request): Json<ValuationInputReviewRequest>,
) -> Response {
    let admin = match super::public_admin::require_public_admin_for_mutation(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let symbol = symbol.trim().to_ascii_uppercase();
    if !valid_symbol(&symbol) {
        return (StatusCode::BAD_REQUEST, "invalid valuation review symbol").into_response();
    }
    let _lock = match acquire_review_lock(&state, &symbol).await {
        Ok(lock) => lock,
        Err(error) if error.contains("already in progress") => {
            return (StatusCode::CONFLICT, error).into_response();
        }
        Err(error) => {
            warn!(%symbol, %error, "valuation input review lock failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "valuation input review unavailable",
            )
                .into_response();
        }
    };
    let states = super::investment_decisions::current_sec_financial_states(
        &state,
        std::slice::from_ref(&symbol),
        Utc::now(),
    )
    .await;
    let Some(evidence) = states.get(&symbol).cloned() else {
        return (
            StatusCode::NOT_FOUND,
            "current SEC financial evidence not found",
        )
            .into_response();
    };
    let current_fingerprint = financial_evidence_fingerprint(&evidence);
    if request.expected_financial_evidence_fingerprint_sha256 != current_fingerprint {
        return (
            StatusCode::CONFLICT,
            "financial evidence changed; reload before submitting",
        )
            .into_response();
    }
    normalize_inputs(&mut request.supplemental_inputs);
    if let Err(error) = validate_inputs(&request.supplemental_inputs, Utc::now().date_naive()) {
        return (StatusCode::BAD_REQUEST, error).into_response();
    }
    let records = match read_review_records(&state, &symbol).await {
        Ok(records) => records,
        Err(error) => {
            warn!(%symbol, %error, "valuation input audit read failed");
            return (StatusCode::CONFLICT, "valuation input audit is invalid").into_response();
        }
    };
    let latest = match latest_review_record(&records) {
        Ok(latest) => latest,
        Err(error) => {
            warn!(%symbol, %error, "valuation input audit chain invalid");
            return (StatusCode::CONFLICT, "valuation input audit is invalid").into_response();
        }
    };
    let previous_review_id = latest.map(|record| record.review_id.clone());
    if request.expected_review_id != previous_review_id {
        return (
            StatusCode::CONFLICT,
            "valuation input review changed; reload before submitting",
        )
            .into_response();
    }
    let rationale = request.rationale.trim().to_string();
    if rationale.len() < 8 || rationale.len() > 10_000 {
        return (
            StatusCode::BAD_REQUEST,
            "valuation input review requires a written rationale",
        )
            .into_response();
    }
    let approved = request.verdict == ValuationInputReviewVerdict::ApprovedForValuation;
    if approved
        && (!request.confirmations.all_confirmed()
            || !request.supplemental_inputs.missing_inputs().is_empty()
            || !input_is_fresh(&request.supplemental_inputs, Utc::now().date_naive())
            || evidence.financial_value_unit.as_deref() != Some("USD_millions"))
    {
        return (
            StatusCode::BAD_REQUEST,
            "valuation approval requires complete inputs, two methods, sources and all confirmations",
        )
            .into_response();
    }
    let submitted_at = Utc::now();
    let input_fingerprint_sha256 =
        supplemental_input_fingerprint(&symbol, &current_fingerprint, &request.supplemental_inputs);
    let record = ValuationInputReviewRecord {
        schema_version: VALUATION_INPUT_REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: VALUATION_INPUT_REVIEW_POLICY_VERSION.to_string(),
        review_id: format!(
            "{}-valuation-input-{}-{}",
            symbol,
            submitted_at.timestamp_millis(),
            Uuid::new_v4()
        ),
        previous_review_id,
        symbol: symbol.clone(),
        submitted_at,
        reviewer_id: admin.user_id,
        financial_evidence_fingerprint_sha256: current_fingerprint,
        financial_evidence_snapshot: evidence,
        input_fingerprint_sha256,
        supplemental_inputs: request.supplemental_inputs,
        verdict: request.verdict,
        rationale,
        confirmations: request.confirmations,
        valuation_authorized: approved,
        rating_factor_authorized: false,
        training_authorized: false,
        reward_authorized: false,
        portfolio_action_authorized: false,
        shadow_portfolio_authorized: false,
        trade_authorized: false,
        old_wang_logic_confirmed: false,
    };
    if let Err(error) = validate_review_record(&record, Utc::now().date_naive()) {
        return (StatusCode::BAD_REQUEST, error).into_response();
    }
    let path = review_directory(&state, &symbol).join(format!("{}.json", record.review_id));
    if let Err(error) = write_immutable_json(&path, &record).await {
        warn!(%symbol, %error, "valuation input audit write failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "valuation input review unavailable",
        )
            .into_response();
    }
    super::valuation_lab::refresh_and_store(&state).await;
    match review_response(&state, std::slice::from_ref(&symbol)).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => {
            warn!(%symbol, %error, "valuation input review response failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "valuation input review saved; refresh the page",
            )
                .into_response()
        }
    }
}

pub(crate) async fn review_outcomes_for_states(
    state: &AppState,
    states: &HashMap<String, FinancialVerificationState>,
) -> HashMap<String, ValuationInputReviewCandidate> {
    let mut outcomes = HashMap::new();
    for (symbol, evidence) in states {
        let candidate = match review_candidate(state, symbol, evidence.clone(), Utc::now()).await {
            Ok(candidate) => candidate,
            Err(error) => {
                warn!(%symbol, %error, "valuation input audit failed closed");
                ValuationInputReviewCandidate {
                    symbol: symbol.clone(),
                    financial_evidence_fingerprint_sha256: financial_evidence_fingerprint(evidence),
                    financial_evidence: evidence.clone(),
                    review_status: "sec_valuation_review_audit_invalid".to_string(),
                    valuation_authorized: false,
                    blocking_reasons: vec![
                        "估值输入复核链无效；修复前禁止使用 SEC 输入估值".to_string(),
                    ],
                    prepared_method_ids: Vec::new(),
                    latest_review: None,
                }
            }
        };
        outcomes.insert(symbol.clone(), candidate);
    }
    outcomes
}

async fn normalized_query_symbols(
    state: &AppState,
    symbol: Option<String>,
) -> Result<Vec<String>, String> {
    if let Some(symbol) = symbol {
        let symbol = symbol.trim().to_ascii_uppercase();
        if !valid_symbol(&symbol) {
            return Err("invalid valuation review symbol".to_string());
        }
        Ok(vec![symbol])
    } else {
        Ok(super::company_ratings::covered_symbols(state).await)
    }
}

async fn review_response(
    state: &AppState,
    symbols: &[String],
) -> Result<ValuationInputReviewResponse, String> {
    let states =
        super::investment_decisions::current_sec_financial_states(state, symbols, Utc::now()).await;
    let mut candidates = Vec::new();
    for (symbol, evidence) in states {
        candidates.push(review_candidate(state, &symbol, evidence, Utc::now()).await?);
    }
    candidates.sort_by(|left, right| {
        right
            .valuation_authorized
            .cmp(&left.valuation_authorized)
            .then_with(|| left.symbol.cmp(&right.symbol))
    });
    let observed = candidates.len();
    let authorized = candidates
        .iter()
        .filter(|candidate| candidate.valuation_authorized)
        .count();
    let stale = candidates
        .iter()
        .filter(|candidate| candidate.review_status.contains("stale"))
        .count();
    let pending = observed.saturating_sub(authorized + stale);
    Ok(ValuationInputReviewResponse {
        schema_version: VALUATION_INPUT_REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: VALUATION_INPUT_REVIEW_POLICY_VERSION.to_string(),
        generated_at: Utc::now(),
        observed,
        pending,
        authorized,
        stale,
        candidates,
        scope: "只授权一个短期有效、证据指纹精确匹配的 SEC 估值输入包进入 HONE 多方法估值；不批准投资结论，也不授权评级财务、训练、奖励、组合、影子组合或交易。".to_string(),
        rating_factor_authorized: false,
        training_authorized: false,
        reward_authorized: false,
        portfolio_action_authorized: false,
        shadow_portfolio_authorized: false,
        trade_authorized: false,
    })
}

async fn review_candidate(
    state: &AppState,
    symbol: &str,
    evidence: FinancialVerificationState,
    now: DateTime<Utc>,
) -> Result<ValuationInputReviewCandidate, String> {
    let fingerprint = financial_evidence_fingerprint(&evidence);
    let records = read_review_records(state, symbol).await?;
    let latest = latest_review_record(&records)?.cloned();
    let (review_status, valuation_authorized, blocking_reasons, prepared_method_ids) =
        review_status_from_latest(&fingerprint, latest.as_ref(), now.date_naive());
    Ok(ValuationInputReviewCandidate {
        symbol: symbol.to_string(),
        financial_evidence_fingerprint_sha256: fingerprint,
        financial_evidence: evidence,
        review_status,
        valuation_authorized,
        blocking_reasons,
        prepared_method_ids,
        latest_review: latest,
    })
}

fn review_status_from_latest(
    fingerprint: &str,
    latest: Option<&ValuationInputReviewRecord>,
    today: NaiveDate,
) -> (String, bool, Vec<String>, Vec<String>) {
    let Some(review) = latest else {
        return (
            "sec_valuation_review_pending".to_string(),
            false,
            vec!["尚未完成独立估值用途复核".to_string()],
            Vec::new(),
        );
    };
    let methods = review.supplemental_inputs.prepared_method_ids();
    if review.financial_evidence_fingerprint_sha256 != fingerprint {
        return (
            "sec_valuation_review_stale_evidence_changed".to_string(),
            false,
            vec!["SEC 财务证据已经变化，旧估值授权自动失效".to_string()],
            methods,
        );
    }
    if review.verdict == ValuationInputReviewVerdict::ApprovedForValuation
        && !input_is_fresh(&review.supplemental_inputs, today)
    {
        return (
            "sec_valuation_review_stale_input_expired".to_string(),
            false,
            vec![format!(
                "估值补充输入超过 {MAX_INPUT_AGE_DAYS} 天，必须重新核验"
            )],
            methods,
        );
    }
    if review.verdict == ValuationInputReviewVerdict::ApprovedForValuation
        && review.valuation_authorized
        && review.confirmations.all_confirmed()
        && review.supplemental_inputs.missing_inputs().is_empty()
    {
        return (
            "sec_human_reviewed_for_valuation".to_string(),
            true,
            Vec::new(),
            methods,
        );
    }
    if review.verdict == ValuationInputReviewVerdict::ChangesRequested {
        return (
            "sec_valuation_review_changes_requested".to_string(),
            false,
            vec![review.rationale.clone()],
            methods,
        );
    }
    (
        "sec_valuation_review_rejected".to_string(),
        false,
        vec![review.rationale.clone()],
        methods,
    )
}

fn normalize_inputs(inputs: &mut SupplementalValuationInputs) {
    inputs.input_as_of = inputs.input_as_of.trim().to_string();
    inputs.currency = inputs.currency.trim().to_ascii_uppercase();
    inputs.source_note = inputs.source_note.trim().to_string();
    inputs.source_urls = inputs
        .source_urls
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
}

fn validate_inputs(inputs: &SupplementalValuationInputs, today: NaiveDate) -> Result<(), String> {
    if inputs.currency != "USD" {
        return Err("valuation supplemental inputs currently require USD".to_string());
    }
    let input_date = NaiveDate::parse_from_str(&inputs.input_as_of, "%Y-%m-%d")
        .map_err(|_| "valuation input_as_of must be YYYY-MM-DD".to_string())?;
    if input_date > today || (today - input_date).num_days() > 60 {
        return Err("valuation input_as_of is future dated or too old to review".to_string());
    }
    for (name, value) in [
        ("diluted_shares_millions", inputs.diluted_shares_millions),
        ("forward_eps", inputs.forward_eps),
        ("forward_revenue_millions", inputs.forward_revenue_millions),
    ] {
        if value.is_some_and(|value| !value.is_finite() || value <= 0.0) {
            return Err(format!("{name} must be a finite positive number"));
        }
    }
    if inputs
        .complete_net_cash_millions
        .is_some_and(|value| !value.is_finite())
    {
        return Err("complete_net_cash_millions must be finite".to_string());
    }
    if inputs
        .normalized_ebit_margin_percent
        .is_some_and(|value| !value.is_finite() || !(-100.0..=100.0).contains(&value))
    {
        return Err("normalized_ebit_margin_percent must be between -100 and 100".to_string());
    }
    if inputs.annual_fcf_history_millions.len() > 20
        || inputs
            .annual_fcf_history_millions
            .iter()
            .any(|value| !value.is_finite())
    {
        return Err("annual FCF history must contain at most 20 finite values".to_string());
    }
    if inputs.source_urls.len() > 20
        || inputs
            .source_urls
            .iter()
            .any(|value| !valid_https_url(value))
    {
        return Err("valuation source URLs must be bounded HTTPS links".to_string());
    }
    if inputs.source_note.len() > 10_000 {
        return Err("valuation source note is too long".to_string());
    }
    Ok(())
}

fn input_is_fresh(inputs: &SupplementalValuationInputs, today: NaiveDate) -> bool {
    NaiveDate::parse_from_str(&inputs.input_as_of, "%Y-%m-%d")
        .ok()
        .is_some_and(|date| date <= today && (today - date).num_days() <= MAX_INPUT_AGE_DAYS)
}

pub(crate) fn supplemental_input_fingerprint(
    symbol: &str,
    financial_fingerprint: &str,
    inputs: &SupplementalValuationInputs,
) -> String {
    let bytes = serde_json::to_vec(&(symbol, financial_fingerprint, inputs))
        .expect("valuation inputs are serializable");
    format!("{:x}", Sha256::digest(bytes))
}

fn validate_review_record(
    record: &ValuationInputReviewRecord,
    today: NaiveDate,
) -> Result<(), String> {
    validate_inputs(&record.supplemental_inputs, today)?;
    if record.schema_version != VALUATION_INPUT_REVIEW_SCHEMA_VERSION
        || record.policy_version != VALUATION_INPUT_REVIEW_POLICY_VERSION
        || !valid_symbol(&record.symbol)
        || record.review_id.trim().is_empty()
        || record.reviewer_id.trim().is_empty()
        || record.rationale.trim().len() < 8
        || record.rationale.len() > 10_000
        || !valid_sha256(&record.financial_evidence_fingerprint_sha256)
        || record.financial_evidence_fingerprint_sha256
            != financial_evidence_fingerprint(&record.financial_evidence_snapshot)
        || !valid_sha256(&record.input_fingerprint_sha256)
        || record.input_fingerprint_sha256
            != supplemental_input_fingerprint(
                &record.symbol,
                &record.financial_evidence_fingerprint_sha256,
                &record.supplemental_inputs,
            )
    {
        return Err("invalid valuation input review record".to_string());
    }
    let approved = record.verdict == ValuationInputReviewVerdict::ApprovedForValuation;
    if approved != record.valuation_authorized
        || (approved
            && (!record.confirmations.all_confirmed()
                || !record.supplemental_inputs.missing_inputs().is_empty()
                || record
                    .financial_evidence_snapshot
                    .financial_value_unit
                    .as_deref()
                    != Some("USD_millions")))
        || record.rating_factor_authorized
        || record.training_authorized
        || record.reward_authorized
        || record.portfolio_action_authorized
        || record.shadow_portfolio_authorized
        || record.trade_authorized
        || record.old_wang_logic_confirmed
    {
        return Err("valuation input review authority exceeded its scope".to_string());
    }
    Ok(())
}

async fn read_review_records(
    state: &AppState,
    symbol: &str,
) -> Result<Vec<ValuationInputReviewRecord>, String> {
    let directory = review_directory(state, symbol);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let record = serde_json::from_slice::<ValuationInputReviewRecord>(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        validate_review_record(&record, record.submitted_at.date_naive())
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if record.symbol != symbol {
            return Err(format!(
                "{}: valuation review symbol mismatch",
                path.display()
            ));
        }
        records.push(record);
    }
    latest_review_record(&records)?;
    Ok(records)
}

fn latest_review_record(
    records: &[ValuationInputReviewRecord],
) -> Result<Option<&ValuationInputReviewRecord>, String> {
    if records.is_empty() {
        return Ok(None);
    }
    let ids = records
        .iter()
        .map(|record| record.review_id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != records.len() {
        return Err("valuation input audit contains duplicate ids".to_string());
    }
    let roots = records
        .iter()
        .filter(|record| record.previous_review_id.is_none())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("valuation input audit chain must have exactly one root".to_string());
    }
    let mut current = roots[0];
    let mut visited = vec![current.review_id.as_str()];
    loop {
        let children = records
            .iter()
            .filter(|record| record.previous_review_id.as_deref() == Some(&current.review_id))
            .collect::<Vec<_>>();
        match children.as_slice() {
            [] => break,
            [child] => {
                if visited.contains(&child.review_id.as_str()) {
                    return Err("valuation input audit chain contains a cycle".to_string());
                }
                visited.push(child.review_id.as_str());
                current = child;
            }
            _ => return Err("valuation input audit chain contains a branch".to_string()),
        }
    }
    if visited.len() != records.len() {
        return Err("valuation input audit chain is disconnected".to_string());
    }
    Ok(Some(current))
}

fn review_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("company_ratings")
        .join("valuation-input-reviews")
}

fn review_directory(state: &AppState, symbol: &str) -> PathBuf {
    review_root(state).join("audit").join(symbol)
}

async fn acquire_review_lock(
    state: &AppState,
    symbol: &str,
) -> Result<ValuationReviewLock, String> {
    let directory = review_root(state).join("locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{symbol}.lock"));
    for attempt in 0..2 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ValuationReviewLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && attempt == 0 => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                    .is_some_and(|age| age >= StdDuration::from_secs(600));
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                return Err("valuation input review is already in progress".to_string());
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err("valuation input review is already in progress".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("valuation input review lock unavailable".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "valuation review path has no parent".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let temp = parent.join(format!(".{}.{}.tmp", Uuid::new_v4(), std::process::id()));
    tokio::fs::write(&temp, bytes)
        .await
        .map_err(|error| error.to_string())?;
    let linked = tokio::fs::hard_link(&temp, path)
        .await
        .map_err(|error| error.to_string());
    let _ = tokio::fs::remove_file(&temp).await;
    linked
}

fn valid_symbol(symbol: &str) -> bool {
    (1..=12).contains(&symbol.len())
        && symbol
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'.')
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_https_url(value: &str) -> bool {
    value.starts_with("https://") && value.len() <= 2_048 && !value.chars().any(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::investment_decisions::MeasurementStatus;

    fn evidence() -> FinancialVerificationState {
        FinancialVerificationState {
            policy_version: "hone-financial-verification-v5-valuation-input-preparation"
                .to_string(),
            status: MeasurementStatus::PartiallyMeasured,
            financial_as_of: Some("2026-08-18".to_string()),
            revenue_growth_percent: Some(20.0),
            gross_margin_percent: Some(50.0),
            gross_margin_change_pp: Some(2.0),
            ebit_margin_percent: Some(30.0),
            fcf_margin_percent: None,
            accounts_receivable_growth_percent: None,
            accounts_payable_growth_percent: None,
            inventory_growth_percent: None,
            property_plant_equipment_growth_percent: None,
            operating_cash_flow_growth_percent: None,
            capital_expenditure_growth_percent: None,
            free_cash_flow_growth_percent: None,
            cash_and_equivalents: Some(4_000.0),
            long_term_debt: Some(1_000.0),
            net_cash: Some(3_000.0),
            current_free_cash_flow: Some(2_000.0),
            prior_free_cash_flow: Some(1_500.0),
            financial_value_unit: Some("USD_millions".to_string()),
            forward_metric_label: None,
            forward_metric_value: None,
            forward_metric_growth_percent: None,
            forward_metric_as_of: None,
            source_claim_ids: vec!["claim-1".to_string()],
            source_urls: vec!["https://www.sec.gov/filing".to_string()],
            source_calculations: vec!["FCF = OCF - capex".to_string()],
            source_claims: Vec::new(),
            quality_warnings: Vec::new(),
            missing_checks: Vec::new(),
        }
    }

    fn inputs(as_of: &str) -> SupplementalValuationInputs {
        SupplementalValuationInputs {
            input_as_of: as_of.to_string(),
            currency: "USD".to_string(),
            diluted_shares_millions: Some(150.0),
            complete_net_cash_millions: Some(3_000.0),
            forward_eps: Some(12.0),
            forward_revenue_millions: Some(20_000.0),
            normalized_ebit_margin_percent: Some(30.0),
            annual_fcf_history_millions: vec![1_000.0, 1_500.0, 2_000.0],
            source_urls: vec![
                "https://www.sec.gov/filing".to_string(),
                "https://investor.example.com/estimate".to_string(),
            ],
            source_note: "已核对稀释股本、净现金和下一财年一致预期口径".to_string(),
        }
    }

    fn confirmations() -> ValuationInputReviewConfirmations {
        ValuationInputReviewConfirmations {
            official_sources_opened: true,
            sec_financial_values_recomputed: true,
            diluted_share_count_and_corporate_actions_verified: true,
            complete_net_cash_or_debt_verified: true,
            forward_or_midcycle_inputs_verified: true,
            cyclicality_and_normalization_checked: true,
            cross_method_comparability_checked: true,
            no_unresolved_material_issue: true,
        }
    }

    fn approved_record(today: NaiveDate) -> ValuationInputReviewRecord {
        let evidence = evidence();
        let financial_fingerprint = financial_evidence_fingerprint(&evidence);
        let supplemental_inputs = inputs(&today.to_string());
        ValuationInputReviewRecord {
            schema_version: VALUATION_INPUT_REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: VALUATION_INPUT_REVIEW_POLICY_VERSION.to_string(),
            review_id: "review-1".to_string(),
            previous_review_id: None,
            symbol: "SNDK".to_string(),
            submitted_at: Utc::now(),
            reviewer_id: "admin".to_string(),
            financial_evidence_fingerprint_sha256: financial_fingerprint.clone(),
            financial_evidence_snapshot: evidence,
            input_fingerprint_sha256: supplemental_input_fingerprint(
                "SNDK",
                &financial_fingerprint,
                &supplemental_inputs,
            ),
            supplemental_inputs,
            verdict: ValuationInputReviewVerdict::ApprovedForValuation,
            rationale: "已完成估值用途独立复核并保留来源".to_string(),
            confirmations: confirmations(),
            valuation_authorized: true,
            rating_factor_authorized: false,
            training_authorized: false,
            reward_authorized: false,
            portfolio_action_authorized: false,
            shadow_portfolio_authorized: false,
            trade_authorized: false,
            old_wang_logic_confirmed: false,
        }
    }

    #[test]
    fn approval_requires_two_methods_and_universal_inputs() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let complete = inputs(&today.to_string());
        assert_eq!(complete.prepared_method_ids().len(), 3);
        assert!(complete.missing_inputs().is_empty());

        let mut incomplete = complete;
        incomplete.diluted_shares_millions = None;
        incomplete.forward_revenue_millions = None;
        incomplete.annual_fcf_history_millions.clear();
        assert_eq!(incomplete.prepared_method_ids(), vec!["forward_pe"]);
        assert!(incomplete.missing_inputs().len() >= 2);
    }

    #[test]
    fn exact_fresh_approval_authorizes_only_valuation() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let record = approved_record(today);
        validate_review_record(&record, today).unwrap();
        let (status, authorized, reasons, methods) = review_status_from_latest(
            &record.financial_evidence_fingerprint_sha256,
            Some(&record),
            today,
        );
        assert_eq!(status, "sec_human_reviewed_for_valuation");
        assert!(authorized);
        assert!(reasons.is_empty());
        assert_eq!(methods.len(), 3);
        assert!(!record.rating_factor_authorized);
        assert!(!record.training_authorized);
        assert!(!record.reward_authorized);
        assert!(!record.portfolio_action_authorized);
        assert!(!record.shadow_portfolio_authorized);
        assert!(!record.trade_authorized);
        assert!(!record.old_wang_logic_confirmed);
    }

    #[test]
    fn evidence_change_and_input_expiry_fail_closed() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let mut record = approved_record(today);
        let (status, authorized, _, _) =
            review_status_from_latest(&"0".repeat(64), Some(&record), today);
        assert_eq!(status, "sec_valuation_review_stale_evidence_changed");
        assert!(!authorized);

        record.supplemental_inputs.input_as_of = "2026-08-13".to_string();
        record.input_fingerprint_sha256 = supplemental_input_fingerprint(
            &record.symbol,
            &record.financial_evidence_fingerprint_sha256,
            &record.supplemental_inputs,
        );
        let (status, authorized, _, _) = review_status_from_latest(
            &record.financial_evidence_fingerprint_sha256,
            Some(&record),
            today,
        );
        assert_eq!(status, "sec_valuation_review_stale_input_expired");
        assert!(!authorized);
    }

    #[test]
    fn fingerprint_binds_financial_evidence_and_supplemental_values() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let original = inputs(&today.to_string());
        let first = supplemental_input_fingerprint("SNDK", &"a".repeat(64), &original);
        let mut changed = original.clone();
        changed.forward_eps = Some(12.5);
        assert_ne!(
            first,
            supplemental_input_fingerprint("SNDK", &"a".repeat(64), &changed)
        );
        assert_ne!(
            first,
            supplemental_input_fingerprint("SNDK", &"b".repeat(64), &original)
        );
    }

    #[test]
    fn audit_chain_rejects_branching() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 21).unwrap();
        let first = approved_record(today);
        let mut second = approved_record(today);
        second.review_id = "review-2".to_string();
        second.previous_review_id = Some(first.review_id.clone());
        let mut branch = approved_record(today);
        branch.review_id = "review-3".to_string();
        branch.previous_review_id = Some(first.review_id.clone());
        assert_eq!(
            latest_review_record(&[first.clone(), second.clone()])
                .unwrap()
                .unwrap()
                .review_id,
            "review-2"
        );
        assert!(latest_review_record(&[first, second, branch]).is_err());
    }
}
