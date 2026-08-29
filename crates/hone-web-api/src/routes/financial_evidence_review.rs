//! Human quality review for SEC-derived financial projections.
//!
//! This is deliberately separate from thesis and causal-effect review.  It
//! answers one narrow question: may this exact, fingerprinted financial
//! projection participate in the daily rating factors?  A review never
//! confirms old-Wang logic, creates a training label, values a company, or
//! authorizes a portfolio/trading action.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

#[cfg(test)]
use super::investment_decisions::FinancialSourceClaimTrace;
use super::investment_decisions::FinancialVerificationState;
use crate::state::AppState;

pub(crate) const FINANCIAL_REVIEW_SCHEMA_VERSION: &str = "hone-sec-financial-evidence-review-v1";
pub(crate) const FINANCIAL_REVIEW_POLICY_VERSION: &str = "hone-sec-financial-rating-admission-v1";
pub(crate) const FINANCIAL_REVIEW_SELECTION_POLICY_VERSION: &str =
    "hone-financial-review-readiness-batch-v1";
const DEFAULT_FINANCIAL_REVIEW_BATCH_SIZE: usize = 5;
const MAX_FINANCIAL_REVIEW_BATCH_SIZE: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FinancialEvidenceReviewVerdict {
    ApprovedForRating,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct FinancialEvidenceReviewConfirmations {
    pub official_filings_opened: bool,
    pub identity_periods_and_units_verified: bool,
    pub calculations_recomputed: bool,
    pub corporate_actions_and_restatements_checked: bool,
    pub quality_warnings_resolved: bool,
    pub no_unresolved_material_issue: bool,
}

impl FinancialEvidenceReviewConfirmations {
    fn all_confirmed(&self) -> bool {
        self.official_filings_opened
            && self.identity_periods_and_units_verified
            && self.calculations_recomputed
            && self.corporate_actions_and_restatements_checked
            && self.quality_warnings_resolved
            && self.no_unresolved_material_issue
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FinancialEvidenceReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub previous_review_id: Option<String>,
    pub symbol: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub evidence_fingerprint_sha256: String,
    pub evidence_snapshot: FinancialVerificationState,
    pub verdict: FinancialEvidenceReviewVerdict,
    pub rationale: String,
    pub confirmations: FinancialEvidenceReviewConfirmations,
    pub rating_factor_authorized: bool,
    pub valuation_authorized: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub portfolio_action_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub trade_authorized: bool,
    pub old_wang_logic_confirmed: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinancialEvidenceReviewRequest {
    pub expected_review_id: Option<String>,
    pub expected_evidence_fingerprint_sha256: String,
    pub verdict: FinancialEvidenceReviewVerdict,
    pub rationale: String,
    #[serde(default)]
    pub confirmations: FinancialEvidenceReviewConfirmations,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FinancialEvidenceReviewCandidate {
    pub symbol: String,
    pub evidence_fingerprint_sha256: String,
    pub evidence: FinancialVerificationState,
    pub review_status: String,
    pub score_eligible: bool,
    pub blocking_reasons: Vec<String>,
    pub latest_review: Option<FinancialEvidenceReviewRecord>,
    pub review_priority_rank: u8,
    pub review_priority_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FinancialEvidenceReviewSummary {
    pub observed: usize,
    pub pending: usize,
    pub approved_for_rating: usize,
    pub changes_requested: usize,
    pub rejected: usize,
    pub stale_after_evidence_change: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FinancialEvidenceReviewResponse {
    pub schema_version: String,
    pub policy_version: String,
    pub generated_at: DateTime<Utc>,
    pub summary: FinancialEvidenceReviewSummary,
    pub candidates: Vec<FinancialEvidenceReviewCandidate>,
    pub selection_mode: String,
    pub selection_policy_version: String,
    pub selection_scope: String,
    pub eligible_queue: usize,
    pub returned: usize,
    pub scope: String,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub portfolio_action_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub trade_authorized: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FinancialEvidenceReviewQuery {
    pub symbol: Option<String>,
    pub selection: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FinancialReviewSelection {
    ActiveBatch,
    FullQueue,
}

impl FinancialReviewSelection {
    fn as_str(self) -> &'static str {
        match self {
            Self::ActiveBatch => "active_batch",
            Self::FullQueue => "full_queue",
        }
    }
}

#[derive(Debug)]
struct FinancialReviewLock {
    path: PathBuf,
}

impl Drop for FinancialReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub(crate) async fn handle_get_financial_evidence_reviews(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<FinancialEvidenceReviewQuery>,
) -> Response {
    if let Err(response) = super::public_admin::require_public_admin_for_read(&state, &headers) {
        return response;
    }
    let symbols = match normalized_query_symbols(&state, query.symbol).await {
        Ok(symbols) => symbols,
        Err(error) => return (StatusCode::BAD_REQUEST, error).into_response(),
    };
    let selection = match normalized_selection(query.selection, symbols.len() == 1) {
        Ok(selection) => selection,
        Err(error) => return (StatusCode::BAD_REQUEST, error).into_response(),
    };
    let limit = normalized_selection_limit(query.limit, selection);
    match financial_review_response(&state, &symbols, selection, limit).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => {
            warn!(%error, "financial evidence review read failed");
            (
                StatusCode::CONFLICT,
                "财务证据复核链不可用，请先检查审计记录",
            )
                .into_response()
        }
    }
}

pub(crate) async fn handle_review_financial_evidence(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(symbol): AxumPath<String>,
    Json(request): Json<FinancialEvidenceReviewRequest>,
) -> Response {
    let admin = match super::public_admin::require_public_admin_for_mutation(&state, &headers) {
        Ok(admin) => admin,
        Err(response) => return response,
    };
    let symbol = symbol.trim().to_ascii_uppercase();
    if !valid_symbol(&symbol) {
        return (StatusCode::BAD_REQUEST, "invalid financial review symbol").into_response();
    }
    let _lock = match acquire_review_lock(&state, &symbol).await {
        Ok(lock) => lock,
        Err(error) if error.contains("already in progress") => {
            return (StatusCode::CONFLICT, error).into_response();
        }
        Err(error) => {
            warn!(%symbol, %error, "financial evidence review lock failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "financial evidence review unavailable",
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
    if request.expected_evidence_fingerprint_sha256 != current_fingerprint {
        return (
            StatusCode::CONFLICT,
            "financial evidence changed; reload before submitting",
        )
            .into_response();
    }
    let records = match read_review_records(&state, &symbol).await {
        Ok(records) => records,
        Err(error) => {
            warn!(%symbol, %error, "financial evidence audit read failed");
            return (StatusCode::CONFLICT, "financial evidence audit is invalid").into_response();
        }
    };
    let latest = match latest_review_record(&records) {
        Ok(latest) => latest,
        Err(error) => {
            warn!(%symbol, %error, "financial evidence audit chain invalid");
            return (StatusCode::CONFLICT, "financial evidence audit is invalid").into_response();
        }
    };
    let previous_review_id = latest.map(|record| record.review_id.clone());
    if request.expected_review_id != previous_review_id {
        return (
            StatusCode::CONFLICT,
            "financial evidence review changed; reload before submitting",
        )
            .into_response();
    }
    let rationale = request.rationale.trim().to_string();
    if rationale.len() < 8 || rationale.len() > 10_000 {
        return (
            StatusCode::BAD_REQUEST,
            "financial evidence review requires a written rationale",
        )
            .into_response();
    }
    if request.verdict == FinancialEvidenceReviewVerdict::ApprovedForRating
        && !request.confirmations.all_confirmed()
    {
        return (
            StatusCode::BAD_REQUEST,
            "rating approval requires every financial quality confirmation",
        )
            .into_response();
    }
    let submitted_at = Utc::now();
    let rating_factor_authorized =
        request.verdict == FinancialEvidenceReviewVerdict::ApprovedForRating;
    let record = FinancialEvidenceReviewRecord {
        schema_version: FINANCIAL_REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: FINANCIAL_REVIEW_POLICY_VERSION.to_string(),
        review_id: format!(
            "{}-financial-{}-{}",
            symbol,
            submitted_at.timestamp_millis(),
            Uuid::new_v4()
        ),
        previous_review_id,
        symbol: symbol.clone(),
        submitted_at,
        reviewer_id: admin.user_id,
        evidence_fingerprint_sha256: current_fingerprint,
        evidence_snapshot: evidence,
        verdict: request.verdict,
        rationale,
        confirmations: request.confirmations,
        rating_factor_authorized,
        valuation_authorized: false,
        training_authorized: false,
        reward_authorized: false,
        portfolio_action_authorized: false,
        shadow_portfolio_authorized: false,
        trade_authorized: false,
        old_wang_logic_confirmed: false,
    };
    if let Err(error) = validate_review_record(&record) {
        return (StatusCode::BAD_REQUEST, error).into_response();
    }
    let path = review_directory(&state, &symbol).join(format!("{}.json", record.review_id));
    if let Err(error) = write_immutable_json(&path, &record).await {
        warn!(%symbol, %error, "financial evidence audit write failed");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "financial evidence review unavailable",
        )
            .into_response();
    }
    // The stored rating snapshot is the user-facing projection. Rebuild it
    // immediately so an accepted/rejected review cannot disagree with the
    // dashboard until the next scheduled run.
    super::company_ratings::refresh_and_store(&state).await;
    match financial_review_response(
        &state,
        std::slice::from_ref(&symbol),
        FinancialReviewSelection::FullQueue,
        1,
    )
    .await
    {
        Ok(response) => Json(response).into_response(),
        Err(error) => {
            warn!(%symbol, %error, "financial evidence review response failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "financial evidence review saved; refresh the page",
            )
                .into_response()
        }
    }
}

pub(crate) async fn review_outcomes_for_states(
    state: &AppState,
    states: &HashMap<String, FinancialVerificationState>,
) -> HashMap<String, FinancialEvidenceReviewCandidate> {
    let mut outcomes = HashMap::new();
    for (symbol, evidence) in states {
        match review_candidate(state, symbol, evidence.clone()).await {
            Ok(candidate) => {
                outcomes.insert(symbol.clone(), candidate);
            }
            Err(error) => {
                warn!(%symbol, %error, "financial evidence audit failed closed");
                outcomes.insert(
                    symbol.clone(),
                    FinancialEvidenceReviewCandidate {
                        symbol: symbol.clone(),
                        evidence_fingerprint_sha256: financial_evidence_fingerprint(evidence),
                        evidence: evidence.clone(),
                        review_status: "sec_review_audit_invalid".to_string(),
                        score_eligible: false,
                        blocking_reasons: vec![
                            "财务证据复核链无效；在修复审计记录前禁止进入每日评级分".to_string(),
                        ],
                        latest_review: None,
                        review_priority_rank: 0,
                        review_priority_reasons: vec![
                            "审计链异常，需要先恢复证据完整性".to_string(),
                        ],
                    },
                );
            }
        }
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
            return Err("invalid financial review symbol".to_string());
        }
        Ok(vec![symbol])
    } else {
        Ok(super::company_ratings::covered_symbols(state).await)
    }
}

async fn financial_review_response(
    state: &AppState,
    symbols: &[String],
    selection: FinancialReviewSelection,
    limit: usize,
) -> Result<FinancialEvidenceReviewResponse, String> {
    let states =
        super::investment_decisions::current_sec_financial_states(state, symbols, Utc::now()).await;
    let mut candidates = Vec::new();
    for (symbol, evidence) in states {
        candidates.push(review_candidate(state, &symbol, evidence).await?);
    }
    candidates.sort_by(compare_review_candidates);
    let summary = summarize_candidates(&candidates);
    let eligible_queue = candidates
        .iter()
        .filter(|candidate| candidate_is_actionable(candidate))
        .count();
    let candidates = match selection {
        FinancialReviewSelection::ActiveBatch => candidates
            .into_iter()
            .filter(candidate_is_actionable)
            .take(limit)
            .collect(),
        FinancialReviewSelection::FullQueue => candidates,
    };
    let returned = candidates.len();
    Ok(FinancialEvidenceReviewResponse {
        schema_version: FINANCIAL_REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: FINANCIAL_REVIEW_POLICY_VERSION.to_string(),
        generated_at: Utc::now(),
        summary,
        candidates,
        selection_mode: selection.as_str().to_string(),
        selection_policy_version: FINANCIAL_REVIEW_SELECTION_POLICY_VERSION.to_string(),
        selection_scope: "优先级只表示财务证据审核准备度，不表示投资吸引力。证据变化和要求修正优先；其后优先材料更完整、异常更少且可重算项目更多的待审公司。".to_string(),
        eligible_queue,
        returned,
        scope: "这里只确认当前 SEC 财务口径和计算能否进入每日评级因子；不确认投资逻辑，不授权估值、训练、奖励、组合、影子组合或交易。".to_string(),
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
) -> Result<FinancialEvidenceReviewCandidate, String> {
    let fingerprint = financial_evidence_fingerprint(&evidence);
    let records = read_review_records(state, symbol).await?;
    let latest = latest_review_record(&records)?.cloned();
    let (review_status, score_eligible, blocking_reasons) =
        review_status_from_latest(&fingerprint, latest.as_ref());
    let (review_priority_rank, review_priority_reasons) =
        review_priority(&review_status, &evidence);
    Ok(FinancialEvidenceReviewCandidate {
        symbol: symbol.to_string(),
        evidence_fingerprint_sha256: fingerprint,
        evidence,
        review_status,
        score_eligible,
        blocking_reasons,
        latest_review: latest,
        review_priority_rank,
        review_priority_reasons,
    })
}

fn normalized_selection(
    selection: Option<String>,
    single_symbol: bool,
) -> Result<FinancialReviewSelection, String> {
    match selection.as_deref().map(str::trim) {
        None | Some("") if single_symbol => Ok(FinancialReviewSelection::FullQueue),
        None | Some("") | Some("active_batch") => Ok(FinancialReviewSelection::ActiveBatch),
        Some("full_queue") => Ok(FinancialReviewSelection::FullQueue),
        Some(_) => Err("financial review selection must be active_batch or full_queue".to_string()),
    }
}

fn normalized_selection_limit(limit: Option<usize>, selection: FinancialReviewSelection) -> usize {
    match selection {
        FinancialReviewSelection::ActiveBatch => limit
            .unwrap_or(DEFAULT_FINANCIAL_REVIEW_BATCH_SIZE)
            .clamp(1, MAX_FINANCIAL_REVIEW_BATCH_SIZE),
        FinancialReviewSelection::FullQueue => usize::MAX,
    }
}

fn candidate_is_actionable(candidate: &FinancialEvidenceReviewCandidate) -> bool {
    matches!(
        candidate.review_status.as_str(),
        "sec_review_audit_invalid"
            | "sec_review_stale_evidence_changed"
            | "sec_review_changes_requested"
            | "sec_structured_pending_human_review"
    )
}

fn review_priority(
    review_status: &str,
    evidence: &FinancialVerificationState,
) -> (u8, Vec<String>) {
    let mut reasons = Vec::new();
    let rank = match review_status {
        "sec_review_audit_invalid" => {
            reasons.push("审计链异常，需要先恢复证据完整性".to_string());
            0
        }
        "sec_review_stale_evidence_changed" => {
            reasons.push("底层财务证据已变化，旧审核自动失效".to_string());
            1
        }
        "sec_review_changes_requested" => {
            reasons.push("已有明确修正意见，优先闭环".to_string());
            2
        }
        "sec_structured_pending_human_review" => {
            reasons.push("尚未完成人工财务质量审核".to_string());
            3
        }
        "sec_review_rejected" => 4,
        "sec_human_reviewed_for_rating" => 5,
        _ => 6,
    };
    if evidence.missing_checks.is_empty() {
        reasons.push("当前结构化财务检查项齐全".to_string());
    } else {
        reasons.push(format!("仍缺 {} 项检查", evidence.missing_checks.len()));
    }
    if evidence.quality_warnings.is_empty() {
        reasons.push("没有未解释的自动质量警告".to_string());
    } else {
        reasons.push(format!(
            "仍有 {} 条质量警告",
            evidence.quality_warnings.len()
        ));
    }
    reasons.push(format!(
        "已保留 {} 条逐项来源声明",
        evidence.source_claims.len()
    ));
    (rank, reasons)
}

fn compare_review_candidates(
    left: &FinancialEvidenceReviewCandidate,
    right: &FinancialEvidenceReviewCandidate,
) -> std::cmp::Ordering {
    left.review_priority_rank
        .cmp(&right.review_priority_rank)
        .then_with(|| {
            left.evidence
                .missing_checks
                .len()
                .cmp(&right.evidence.missing_checks.len())
        })
        .then_with(|| {
            left.evidence
                .quality_warnings
                .len()
                .cmp(&right.evidence.quality_warnings.len())
        })
        .then_with(|| {
            right
                .evidence
                .source_claims
                .len()
                .cmp(&left.evidence.source_claims.len())
        })
        .then_with(|| left.symbol.cmp(&right.symbol))
}

fn review_status_from_latest(
    fingerprint: &str,
    latest: Option<&FinancialEvidenceReviewRecord>,
) -> (String, bool, Vec<String>) {
    match latest {
        None => (
            "sec_structured_pending_human_review".to_string(),
            false,
            vec!["尚未完成独立财务证据质量复核".to_string()],
        ),
        Some(review) if review.evidence_fingerprint_sha256 != fingerprint => (
            "sec_review_stale_evidence_changed".to_string(),
            false,
            vec!["SEC 证据或计算已经变化，旧复核自动失效".to_string()],
        ),
        Some(review)
            if review.verdict == FinancialEvidenceReviewVerdict::ApprovedForRating
                && review.rating_factor_authorized
                && review.confirmations.all_confirmed() =>
        {
            (
                "sec_human_reviewed_for_rating".to_string(),
                true,
                Vec::new(),
            )
        }
        Some(review) if review.verdict == FinancialEvidenceReviewVerdict::ChangesRequested => (
            "sec_review_changes_requested".to_string(),
            false,
            vec![review.rationale.clone()],
        ),
        Some(review) => (
            "sec_review_rejected".to_string(),
            false,
            vec![review.rationale.clone()],
        ),
    }
}

fn summarize_candidates(
    candidates: &[FinancialEvidenceReviewCandidate],
) -> FinancialEvidenceReviewSummary {
    FinancialEvidenceReviewSummary {
        observed: candidates.len(),
        pending: candidates
            .iter()
            .filter(|item| item.review_status == "sec_structured_pending_human_review")
            .count(),
        approved_for_rating: candidates.iter().filter(|item| item.score_eligible).count(),
        changes_requested: candidates
            .iter()
            .filter(|item| item.review_status == "sec_review_changes_requested")
            .count(),
        rejected: candidates
            .iter()
            .filter(|item| item.review_status == "sec_review_rejected")
            .count(),
        stale_after_evidence_change: candidates
            .iter()
            .filter(|item| item.review_status == "sec_review_stale_evidence_changed")
            .count(),
    }
}

pub(crate) fn financial_evidence_fingerprint(evidence: &FinancialVerificationState) -> String {
    let mut canonical = evidence.clone();
    canonical.source_claim_ids.sort();
    canonical.source_claim_ids.dedup();
    canonical.source_urls.sort();
    canonical.source_urls.dedup();
    canonical.source_calculations.sort();
    canonical.source_calculations.dedup();
    canonical.quality_warnings.sort();
    canonical.quality_warnings.dedup();
    canonical.missing_checks.sort();
    canonical.missing_checks.dedup();
    let bytes = serde_json::to_vec(&canonical).expect("financial evidence is serializable");
    format!("{:x}", Sha256::digest(bytes))
}

fn validate_review_record(record: &FinancialEvidenceReviewRecord) -> Result<(), String> {
    if record.schema_version != FINANCIAL_REVIEW_SCHEMA_VERSION
        || record.policy_version != FINANCIAL_REVIEW_POLICY_VERSION
        || !valid_symbol(&record.symbol)
        || record.review_id.trim().is_empty()
        || record.reviewer_id.trim().is_empty()
        || record.rationale.trim().len() < 8
        || record.rationale.len() > 10_000
        || record.evidence_snapshot.source_claim_ids.is_empty()
        || record.evidence_snapshot.source_urls.is_empty()
        || !valid_sha256(&record.evidence_fingerprint_sha256)
        || record.evidence_fingerprint_sha256
            != financial_evidence_fingerprint(&record.evidence_snapshot)
    {
        return Err("invalid financial evidence review record".to_string());
    }
    let approved = record.verdict == FinancialEvidenceReviewVerdict::ApprovedForRating;
    if approved != record.rating_factor_authorized
        || (approved && !record.confirmations.all_confirmed())
        || record.valuation_authorized
        || record.training_authorized
        || record.reward_authorized
        || record.portfolio_action_authorized
        || record.shadow_portfolio_authorized
        || record.trade_authorized
        || record.old_wang_logic_confirmed
    {
        return Err("financial review authority exceeded its scope".to_string());
    }
    Ok(())
}

async fn read_review_records(
    state: &AppState,
    symbol: &str,
) -> Result<Vec<FinancialEvidenceReviewRecord>, String> {
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
        let record = serde_json::from_slice::<FinancialEvidenceReviewRecord>(&bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        validate_review_record(&record).map_err(|error| format!("{}: {error}", path.display()))?;
        if record.symbol != symbol {
            return Err(format!(
                "{}: financial review symbol mismatch",
                path.display()
            ));
        }
        records.push(record);
    }
    latest_review_record(&records)?;
    Ok(records)
}

fn latest_review_record(
    records: &[FinancialEvidenceReviewRecord],
) -> Result<Option<&FinancialEvidenceReviewRecord>, String> {
    if records.is_empty() {
        return Ok(None);
    }
    let ids = records
        .iter()
        .map(|record| record.review_id.as_str())
        .collect::<BTreeSet<_>>();
    if ids.len() != records.len() {
        return Err("financial review audit contains duplicate ids".to_string());
    }
    let roots = records
        .iter()
        .filter(|record| record.previous_review_id.is_none())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("financial review audit chain must have exactly one root".to_string());
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
                    return Err("financial review audit chain contains a cycle".to_string());
                }
                visited.push(child.review_id.as_str());
                current = child;
            }
            _ => return Err("financial review audit chain contains a branch".to_string()),
        }
    }
    if visited.len() != records.len() {
        return Err("financial review audit chain is disconnected".to_string());
    }
    Ok(Some(current))
}

fn review_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("company_ratings")
        .join("financial-evidence-reviews")
}

fn review_directory(state: &AppState, symbol: &str) -> PathBuf {
    review_root(state).join("audit").join(symbol)
}

async fn acquire_review_lock(
    state: &AppState,
    symbol: &str,
) -> Result<FinancialReviewLock, String> {
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
            Ok(_) => return Ok(FinancialReviewLock { path }),
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
                return Err("financial evidence review is already in progress".to_string());
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                return Err("financial evidence review is already in progress".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("financial evidence review lock unavailable".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "financial review path has no parent".to_string())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::investment_decisions::MeasurementStatus;

    fn evidence() -> FinancialVerificationState {
        FinancialVerificationState {
            policy_version: "hone-financial-verification-v3-sec-projection-quality-gate"
                .to_string(),
            status: MeasurementStatus::PartiallyMeasured,
            financial_as_of: Some("2026-07-30".to_string()),
            revenue_growth_percent: Some(17.8),
            gross_margin_percent: Some(67.9),
            gross_margin_change_pp: Some(-0.9),
            ebit_margin_percent: Some(46.8),
            fcf_margin_percent: None,
            accounts_receivable_growth_percent: Some(15.7),
            accounts_payable_growth_percent: None,
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
            source_claim_ids: vec!["claim-b".to_string(), "claim-a".to_string()],
            source_urls: vec!["https://www.sec.gov/filing".to_string()],
            source_calculations: vec!["收入同比 17.8%".to_string()],
            source_claims: Vec::new(),
            quality_warnings: vec!["待人工复核".to_string()],
            missing_checks: vec!["净现金".to_string()],
        }
    }

    fn confirmations() -> FinancialEvidenceReviewConfirmations {
        FinancialEvidenceReviewConfirmations {
            official_filings_opened: true,
            identity_periods_and_units_verified: true,
            calculations_recomputed: true,
            corporate_actions_and_restatements_checked: true,
            quality_warnings_resolved: true,
            no_unresolved_material_issue: true,
        }
    }

    fn record(
        id: &str,
        previous: Option<&str>,
        verdict: FinancialEvidenceReviewVerdict,
    ) -> FinancialEvidenceReviewRecord {
        let evidence = evidence();
        let approved = verdict == FinancialEvidenceReviewVerdict::ApprovedForRating;
        FinancialEvidenceReviewRecord {
            schema_version: FINANCIAL_REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: FINANCIAL_REVIEW_POLICY_VERSION.to_string(),
            review_id: id.to_string(),
            previous_review_id: previous.map(str::to_string),
            symbol: "MSFT".to_string(),
            submitted_at: Utc::now(),
            reviewer_id: "admin".to_string(),
            evidence_fingerprint_sha256: financial_evidence_fingerprint(&evidence),
            evidence_snapshot: evidence,
            verdict,
            rationale: "已逐项核对官方财报和计算口径".to_string(),
            confirmations: if approved {
                confirmations()
            } else {
                Default::default()
            },
            rating_factor_authorized: approved,
            valuation_authorized: false,
            training_authorized: false,
            reward_authorized: false,
            portfolio_action_authorized: false,
            shadow_portfolio_authorized: false,
            trade_authorized: false,
            old_wang_logic_confirmed: false,
        }
    }

    #[test]
    fn fingerprint_is_order_independent_but_changes_with_financial_value() {
        let original = evidence();
        let mut reordered = original.clone();
        reordered.source_claim_ids.reverse();
        assert_eq!(
            financial_evidence_fingerprint(&original),
            financial_evidence_fingerprint(&reordered)
        );
        reordered.revenue_growth_percent = Some(18.0);
        assert_ne!(
            financial_evidence_fingerprint(&original),
            financial_evidence_fingerprint(&reordered)
        );
    }

    #[test]
    fn fingerprint_binds_each_source_claim_accounting_basis_and_original_unit() {
        let mut original = evidence();
        original.policy_version = "hone-financial-verification-v4-source-claim-trace".to_string();
        original.source_claim_ids = vec!["claim-a".to_string()];
        original.source_claims = vec![FinancialSourceClaimTrace {
            claim_id: "claim-a".to_string(),
            metric_id: "revenue".to_string(),
            metric_basis: "IFRS:RevenueFromContractsWithCustomers".to_string(),
            period: "FY2025".to_string(),
            numeric_value: 20_000.0,
            unit: "EUR_millions".to_string(),
            source_url: "https://www.sec.gov/filing".to_string(),
            published_at: "2026-03-05T12:00:00Z".parse().unwrap(),
        }];
        let mut changed_unit = original.clone();
        changed_unit.source_claims[0].unit = "USD_millions".to_string();
        assert_ne!(
            financial_evidence_fingerprint(&original),
            financial_evidence_fingerprint(&changed_unit)
        );
        let mut changed_basis = original.clone();
        changed_basis.source_claims[0].metric_basis = "US_GAAP:Revenue".to_string();
        assert_ne!(
            financial_evidence_fingerprint(&original),
            financial_evidence_fingerprint(&changed_basis)
        );
    }

    #[test]
    fn approval_requires_all_checks_and_never_expands_authority() {
        let valid = record(
            "review-1",
            None,
            FinancialEvidenceReviewVerdict::ApprovedForRating,
        );
        validate_review_record(&valid).unwrap();
        assert!(valid.rating_factor_authorized);
        assert!(!valid.valuation_authorized);
        assert!(!valid.training_authorized);
        assert!(!valid.reward_authorized);
        assert!(!valid.portfolio_action_authorized);
        assert!(!valid.shadow_portfolio_authorized);
        assert!(!valid.trade_authorized);
        assert!(!valid.old_wang_logic_confirmed);

        let mut incomplete = valid.clone();
        incomplete.confirmations.quality_warnings_resolved = false;
        assert!(validate_review_record(&incomplete).is_err());
    }

    #[test]
    fn audit_chain_rejects_branches_and_returns_one_tip() {
        let first = record("review-1", None, FinancialEvidenceReviewVerdict::Rejected);
        let second = record(
            "review-2",
            Some("review-1"),
            FinancialEvidenceReviewVerdict::ApprovedForRating,
        );
        assert_eq!(
            latest_review_record(&[first.clone(), second.clone()])
                .unwrap()
                .unwrap()
                .review_id,
            "review-2"
        );
        let branch = record(
            "review-3",
            Some("review-1"),
            FinancialEvidenceReviewVerdict::ChangesRequested,
        );
        assert!(latest_review_record(&[first, second, branch]).is_err());
    }

    #[test]
    fn stale_approved_review_cannot_be_reused_for_changed_evidence() {
        let approved = record(
            "review-1",
            None,
            FinancialEvidenceReviewVerdict::ApprovedForRating,
        );
        let mut changed = approved.evidence_snapshot.clone();
        changed.financial_as_of = Some("2026-08-01".to_string());
        assert_ne!(
            approved.evidence_fingerprint_sha256,
            financial_evidence_fingerprint(&changed)
        );
        let changed_fingerprint = financial_evidence_fingerprint(&changed);
        let (status, eligible, reasons) =
            review_status_from_latest(&changed_fingerprint, Some(&approved));
        assert_eq!(status, "sec_review_stale_evidence_changed");
        assert!(!eligible);
        assert!(reasons[0].contains("自动失效"));
    }

    #[test]
    fn only_exact_approved_tip_admits_rating_factors() {
        let approved = record(
            "review-1",
            None,
            FinancialEvidenceReviewVerdict::ApprovedForRating,
        );
        let (status, eligible, reasons) =
            review_status_from_latest(&approved.evidence_fingerprint_sha256, Some(&approved));
        assert_eq!(status, "sec_human_reviewed_for_rating");
        assert!(eligible);
        assert!(reasons.is_empty());

        let rejected = record("review-2", None, FinancialEvidenceReviewVerdict::Rejected);
        let (status, eligible, _) =
            review_status_from_latest(&rejected.evidence_fingerprint_sha256, Some(&rejected));
        assert_eq!(status, "sec_review_rejected");
        assert!(!eligible);
    }

    #[test]
    fn active_batch_defaults_to_five_and_full_queue_ignores_limit() {
        assert_eq!(
            normalized_selection(None, false).unwrap(),
            FinancialReviewSelection::ActiveBatch
        );
        assert_eq!(
            normalized_selection(None, true).unwrap(),
            FinancialReviewSelection::FullQueue
        );
        assert_eq!(
            normalized_selection_limit(None, FinancialReviewSelection::ActiveBatch),
            5
        );
        assert_eq!(
            normalized_selection_limit(Some(200), FinancialReviewSelection::ActiveBatch),
            20
        );
        assert_eq!(
            normalized_selection_limit(Some(1), FinancialReviewSelection::FullQueue),
            usize::MAX
        );
        assert!(normalized_selection(Some("investment_priority".to_string()), false).is_err());
    }

    #[test]
    fn review_readiness_priority_is_explicit_and_not_an_investment_rank() {
        let mut complete = evidence();
        complete.missing_checks.clear();
        complete.quality_warnings.clear();
        complete.source_claims = vec![FinancialSourceClaimTrace {
            claim_id: "claim-a".to_string(),
            metric_id: "revenue".to_string(),
            metric_basis: "US_GAAP:Revenue".to_string(),
            period: "Q2-2026".to_string(),
            numeric_value: 76_441.0,
            unit: "USD_millions".to_string(),
            source_url: "https://www.sec.gov/filing".to_string(),
            published_at: "2026-07-30T12:00:00Z".parse().unwrap(),
        }];
        let (stale_rank, stale_reasons) =
            review_priority("sec_review_stale_evidence_changed", &complete);
        let (pending_rank, pending_reasons) =
            review_priority("sec_structured_pending_human_review", &complete);
        let (approved_rank, _) = review_priority("sec_human_reviewed_for_rating", &complete);
        assert!(stale_rank < pending_rank);
        assert!(pending_rank < approved_rank);
        assert!(
            stale_reasons
                .iter()
                .any(|reason| reason.contains("自动失效"))
        );
        assert!(
            pending_reasons
                .iter()
                .any(|reason| reason.contains("检查项齐全"))
        );
        assert!(
            pending_reasons
                .iter()
                .any(|reason| reason.contains("1 条逐项来源"))
        );
    }
}
