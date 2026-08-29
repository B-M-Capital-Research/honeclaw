//! Stage 124 independent admission review for one exact Stage 123 validated non-financial notice candidate.
//!
//! Admission creates a separate, append-only formal non-financial observation-evidence record. It
//! never mutates the Stage 122 candidate and cannot create an opening portfolio, ledger event,
//! position, cash, NAV, performance, training signal, order, broker connection, or trade. Approval
//! only opens Stage 125 governance-specification work for an externally sourced opening portfolio.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_output_validations::{
    IndependentlyValidatedNonFinancialObservationNoticeCandidate,
    independently_validated_non_financial_candidates_for_stage_124_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-candidate-admission-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-candidate-admission-review-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-independently-validated-non-financial-observation-candidate-admission-v1-no-financial-state";
const NEXT_GATE: &str =
    "stage_125_opening_portfolio_snapshot_governance_specification_registration";
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationLedgerTransitionCandidateAdmissionVerdict {
    AdmittedAsFormalNonFinancialObservationEvidenceForFutureOpeningPortfolioGovernance,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_stage_123_validation_id: String,
    expected_stage_123_validation_sha256: String,
    expected_stage_122_result_sha256: String,
    expected_stage_122_candidate_sha256: String,
    expected_stage_121_claim_sha256: String,
    expected_stage_114_review_sha256: String,
    expected_stage_112_output_sha256: String,
    verdict: ControlledShadowObservationLedgerTransitionCandidateAdmissionVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_through_stage_123_binding_confirmed: bool,
    reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain_confirmed: bool,
    stage_123_terminal_validation_reopened_rehashed_and_current_confirmed: bool,
    stage_122_candidate_reopened_rehashed_and_exact_match_confirmed: bool,
    exact_stage_114_admitted_observation_binding_preserved_confirmed: bool,
    every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed: bool,
    admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed:
        bool,
    opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed:
        bool,
    approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed:
        bool,
    no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionCandidateAdmissionReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub stage_121_attempt_id: String,
    pub stage_121_claim_sha256: String,
    pub stage_122_result_id: String,
    pub stage_122_result_sha256: String,
    pub stage_122_candidate_sha256: String,
    pub stage_123_validation_id: String,
    pub stage_123_validation_sha256: String,
    pub stage_114_review_sha256: String,
    pub stage_112_output_sha256: String,
    pub stage_123_validated_at: DateTime<Utc>,
    pub submitted_at: DateTime<Utc>,
    pub submitted_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowObservationLedgerTransitionCandidateAdmissionVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub notice_count: usize,
    pub event_type_counts: BTreeMap<String, usize>,
    pub exact_current_stage_51_through_stage_123_binding_confirmed: bool,
    pub reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain: bool,
    pub stage_123_terminal_validation_reopened_rehashed_and_current_confirmed: bool,
    pub stage_122_candidate_reopened_rehashed_and_exact_match_confirmed: bool,
    pub exact_stage_114_admitted_observation_binding_preserved_confirmed: bool,
    pub every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed: bool,
    pub admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed:
        bool,
    pub opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed:
        bool,
    pub approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed:
        bool,
    pub no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub current_custody_and_full_stage_123_validation_revalidated: bool,
    pub original_candidate_remains_untrusted_and_immutable: bool,
    pub formal_non_financial_observation_evidence_admitted: bool,
    pub future_stage_125_opening_portfolio_snapshot_governance_specification_eligible: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub authoritative_ledger_event_created: bool,
    pub position_written: bool,
    pub cash_written: bool,
    pub nav_or_performance_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionCandidateAdmissionItem {
    pub candidate: IndependentlyValidatedNonFinancialObservationNoticeCandidate,
    pub latest_review: Option<ControlledShadowObservationLedgerTransitionCandidateAdmissionReview>,
    pub current_binding: bool,
    pub review_eligible: bool,
    pub formal_non_financial_observation_evidence_admitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowObservationLedgerTransitionCandidateAdmissionItem>,
    pub independently_validated_candidate_count: usize,
    pub review_eligible_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_non_financial_observation_evidence_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_stage_125_opening_portfolio_snapshot_governance_specification_eligible_count: usize,
    pub admission_status: String,
    pub next_gate: String,
    pub admission_review_available: bool,
    pub candidate_remains_untrusted: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub authoritative_ledger_event_created: bool,
    pub nav_or_performance_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationLedgerTransitionCandidateAdmissionReadinessSummary {
    pub review_eligible_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_non_financial_observation_evidence_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_stage_125_opening_portfolio_snapshot_governance_specification_eligible_count: usize,
    pub admission_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct AdmittedNonFinancialObservationEvidenceForStage125 {
    pub candidate: IndependentlyValidatedNonFinancialObservationNoticeCandidate,
    pub admission_review: ControlledShadowObservationLedgerTransitionCandidateAdmissionReview,
}

pub(crate) async fn handle_get_controlled_shadow_observation_ledger_transition_candidate_admission_reviews(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_registry(&state).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => {
            warn!(%error, "Stage 124 candidate admission registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "非财务观察候选独立准入复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_observation_ledger_transition_candidate_admission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_admission(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_observation_ledger_transition_candidate_admission_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionCandidateAdmissionReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(ControlledShadowObservationLedgerTransitionCandidateAdmissionReadinessSummary {
        review_eligible_candidate_count: value.review_eligible_candidate_count,
        reviewed_candidate_count: value.reviewed_candidate_count,
        admitted_non_financial_observation_evidence_count: value
            .admitted_non_financial_observation_evidence_count,
        changes_requested_or_rejected_count: value.changes_requested_or_rejected_count,
        future_stage_125_opening_portfolio_snapshot_governance_specification_eligible_count: value
            .future_stage_125_opening_portfolio_snapshot_governance_specification_eligible_count,
        admission_status: value.admission_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn admitted_non_financial_observation_evidence_for_stage_125(
    state: &AppState,
) -> Result<Vec<AdmittedNonFinancialObservationEvidenceForStage125>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let review = item.latest_review?;
            (item.current_binding
                && item.formal_non_financial_observation_evidence_admitted
                && review
                    .future_stage_125_opening_portfolio_snapshot_governance_specification_eligible)
                .then_some(AdmittedNonFinancialObservationEvidenceForStage125 {
                    candidate: item.candidate,
                    admission_review: review,
                })
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry, String> {
    let candidates =
        independently_validated_non_financial_candidates_for_stage_124_review(state).await?;
    let mut items = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let latest_review = latest_review(state, &candidate).await?;
        let admitted = latest_review
            .as_ref()
            .is_some_and(|value| value.formal_non_financial_observation_evidence_admitted);
        items.push(
            ControlledShadowObservationLedgerTransitionCandidateAdmissionItem {
                current_binding: true,
                review_eligible: !admitted,
                formal_non_financial_observation_evidence_admitted: admitted,
                candidate,
                latest_review,
            },
        );
    }
    let independently_validated_candidate_count = items.len();
    let review_eligible_candidate_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_candidate_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let admitted_non_financial_observation_evidence_count = items
        .iter()
        .filter(|item| item.formal_non_financial_observation_evidence_admitted)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    != ControlledShadowObservationLedgerTransitionCandidateAdmissionVerdict::AdmittedAsFormalNonFinancialObservationEvidenceForFutureOpeningPortfolioGovernance
            })
        })
        .count();
    let admission_status = if admitted_non_financial_observation_evidence_count > 0 {
        "formal_non_financial_observation_evidence_admitted_waiting_stage_125_opening_portfolio_governance"
    } else if changes_requested_or_rejected_count > 0 {
        "non_financial_observation_candidate_admission_changes_requested_or_rejected"
    } else if independently_validated_candidate_count > 0 {
        "waiting_stage_124_independent_candidate_admission_review"
    } else {
        "waiting_stage_123_independently_validated_non_financial_candidate"
    };
    Ok(ControlledShadowObservationLedgerTransitionCandidateAdmissionRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        independently_validated_candidate_count,
        review_eligible_candidate_count,
        reviewed_candidate_count,
        admitted_non_financial_observation_evidence_count,
        changes_requested_or_rejected_count,
        future_stage_125_opening_portfolio_snapshot_governance_specification_eligible_count:
            admitted_non_financial_observation_evidence_count,
        admission_status: admission_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        admission_review_available: true,
        candidate_remains_untrusted: true,
        opening_portfolio_snapshot_admitted: false,
        authoritative_ledger_event_created: false,
        nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 124 由 Stage 123 validator、Stage 122 executor、Stage 121 claimant 与 Stage 51–123 完整责任链之外的新管理员复核。服务端在写入和读取时重开 Stage 123 终态及 exact Stage 122 candidate，并通过 Stage 123 独立读取链再次核验。批准只创建分离、自哈希、追加式的正式非财务观察证据记录；原 candidate 继续 untrusted/immutable。批准后只开放 Stage 125 外部来源期初组合快照治理规格，不补造期初组合，不创建权威账本事件、持仓、现金、净值/绩效、模型/训练/RL/reward、订单、券商或交易权限。".to_string(),
    })
}

async fn review_admission(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest,
) -> Result<ControlledShadowObservationLedgerTransitionCandidateAdmissionReview, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    let candidate = exact_current_candidate(state, attempt_id, &request).await?;
    let latest = latest_review(state, &candidate).await?;
    if latest
        .as_ref()
        .is_some_and(|value| value.formal_non_financial_observation_evidence_admitted)
    {
        return Err("该精确 Stage 123 candidate 已正式准入，复核链永久冻结".to_string());
    }
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_previous_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_previous_review_sha256.as_deref()
    {
        return Err("Stage 124 复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&candidate, &latest);
    let independent = actor_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent
        || !request
            .reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain_confirmed
    {
        return Err(
            "Stage 124 reviewer 必须独立于 validator、executor、claimant 与完整既有责任链"
                .to_string(),
        );
    }
    let mut review = ControlledShadowObservationLedgerTransitionCandidateAdmissionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        stage_121_attempt_id: candidate.claim.attempt_id.clone(),
        stage_121_claim_sha256: candidate.claim.claim_sha256.clone(),
        stage_122_result_id: candidate.result.result_id.clone(),
        stage_122_result_sha256: candidate.result.result_sha256.clone(),
        stage_122_candidate_sha256: candidate.candidate.candidate_sha256.clone(),
        stage_123_validation_id: candidate.validation.validation_id.clone(),
        stage_123_validation_sha256: candidate.validation.validation_sha256.clone(),
        stage_114_review_sha256: candidate.validation.stage_114_review_sha256.clone(),
        stage_112_output_sha256: candidate.validation.stage_112_output_sha256.clone(),
        stage_123_validated_at: candidate.validation.validated_at,
        submitted_at: Utc::now(),
        submitted_by: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核依据")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        notice_count: candidate.candidate.notices.len(),
        event_type_counts: event_type_counts(&candidate),
        exact_current_stage_51_through_stage_123_binding_confirmed: request
            .exact_current_stage_51_through_stage_123_binding_confirmed,
        reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain: independent,
        stage_123_terminal_validation_reopened_rehashed_and_current_confirmed: request
            .stage_123_terminal_validation_reopened_rehashed_and_current_confirmed,
        stage_122_candidate_reopened_rehashed_and_exact_match_confirmed: request
            .stage_122_candidate_reopened_rehashed_and_exact_match_confirmed,
        exact_stage_114_admitted_observation_binding_preserved_confirmed: request
            .exact_stage_114_admitted_observation_binding_preserved_confirmed,
        every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed: request
            .every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed,
        admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed:
            request.admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed,
        opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed:
            request.opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed,
        approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed:
            request.approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed,
        no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed:
            request.no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        current_custody_and_full_stage_123_validation_revalidated: true,
        original_candidate_remains_untrusted_and_immutable: candidate.candidate.untrusted,
        formal_non_financial_observation_evidence_admitted: false,
        future_stage_125_opening_portfolio_snapshot_governance_specification_eligible: false,
        opening_portfolio_snapshot_admitted: false,
        authoritative_ledger_event_created: false,
        position_written: false,
        cash_written: false,
        nav_or_performance_written: false,
        model_or_metric_store_written: false,
        training_or_rl_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == ControlledShadowObservationLedgerTransitionCandidateAdmissionVerdict::AdmittedAsFormalNonFinancialObservationEvidenceForFutureOpeningPortfolioGovernance;
    review.formal_non_financial_observation_evidence_admitted = approved
        && review.current_custody_and_full_stage_123_validation_revalidated
        && review.original_candidate_remains_untrusted_and_immutable
        && review_all_checks(&review);
    review.future_stage_125_opening_portfolio_snapshot_governance_specification_eligible =
        review.formal_non_financial_observation_evidence_admitted;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &candidate)?;
    write_immutable_json(
        &review_directory(state, attempt_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

async fn exact_current_candidate(
    state: &AppState,
    attempt_id: &str,
    request: &ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest,
) -> Result<IndependentlyValidatedNonFinancialObservationNoticeCandidate, String> {
    let candidates =
        independently_validated_non_financial_candidates_for_stage_124_review(state).await?;
    let candidate = candidates
        .into_iter()
        .find(|value| value.claim.attempt_id == attempt_id)
        .ok_or_else(|| "未找到当前可复核的 Stage 123 独立验证 candidate".to_string())?;
    let valid = candidate.validation.validation_id == request.expected_stage_123_validation_id
        && candidate.validation.validation_sha256 == request.expected_stage_123_validation_sha256
        && candidate.result.result_sha256 == request.expected_stage_122_result_sha256
        && candidate.candidate.candidate_sha256 == request.expected_stage_122_candidate_sha256
        && candidate.claim.claim_sha256 == request.expected_stage_121_claim_sha256
        && candidate.validation.stage_114_review_sha256 == request.expected_stage_114_review_sha256
        && candidate.validation.stage_112_output_sha256 == request.expected_stage_112_output_sha256;
    valid
        .then_some(candidate)
        .ok_or_else(|| "Stage 124 预期绑定与当前 Stage 114/112/121–123 精确链不一致".to_string())
}

fn review_all_checks(
    review: &ControlledShadowObservationLedgerTransitionCandidateAdmissionReview,
) -> bool {
    review.exact_current_stage_51_through_stage_123_binding_confirmed
        && review.reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain
        && review.stage_123_terminal_validation_reopened_rehashed_and_current_confirmed
        && review.stage_122_candidate_reopened_rehashed_and_exact_match_confirmed
        && review.exact_stage_114_admitted_observation_binding_preserved_confirmed
        && review.every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed
        && review.admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed
        && review.opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed
        && review.approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed
        && review.no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn event_type_counts(
    candidate: &IndependentlyValidatedNonFinancialObservationNoticeCandidate,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for notice in &candidate.candidate.notices {
        *counts.entry(notice.event_type.clone()).or_default() += 1;
    }
    counts
}

fn excluded_actor_ids(
    candidate: &IndependentlyValidatedNonFinancialObservationNoticeCandidate,
    latest: &Option<ControlledShadowObservationLedgerTransitionCandidateAdmissionReview>,
) -> Vec<String> {
    let mut values = candidate.validation.excluded_prior_actor_ids.clone();
    values.push(candidate.validation.validated_by.clone());
    values.push(candidate.result.executed_by.clone());
    values.push(candidate.claim.claimed_by.clone());
    values.extend(candidate.claim.excluded_prior_actor_ids.clone());
    if let Some(review) = latest {
        values.extend(review.excluded_prior_actor_ids.clone());
        values.push(review.submitted_by.clone());
    }
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn validate_review(
    review: &ControlledShadowObservationLedgerTransitionCandidateAdmissionReview,
    candidate: &IndependentlyValidatedNonFinancialObservationNoticeCandidate,
) -> Result<(), String> {
    let approved = review.verdict
        == ControlledShadowObservationLedgerTransitionCandidateAdmissionVerdict::AdmittedAsFormalNonFinancialObservationEvidenceForFutureOpeningPortfolioGovernance;
    let expected_admitted = approved
        && actor_is_independent(&review.submitted_by, &review.excluded_prior_actor_ids)
        && review.current_custody_and_full_stage_123_validation_revalidated
        && review.original_candidate_remains_untrusted_and_immutable
        && review_all_checks(review);
    let downstream_closed = !review.opening_portfolio_snapshot_admitted
        && !review.authoritative_ledger_event_created
        && !review.position_written
        && !review.cash_written
        && !review.nav_or_performance_written
        && !review.model_or_metric_store_written
        && !review.training_or_rl_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let valid = review.schema_version == REVIEW_SCHEMA_VERSION
        && review.policy_version == POLICY_VERSION
        && valid_id(&review.review_id)
        && valid_sha256(&review.review_sha256)
        && review.review_id == review.review_sha256[..32]
        && review.review_sha256 == review_fingerprint(review)?
        && review.previous_review_id.is_some() == review.previous_review_sha256.is_some()
        && review.previous_review_id.as_deref().is_none_or(valid_id)
        && review
            .previous_review_sha256
            .as_deref()
            .is_none_or(valid_sha256)
        && review.stage_121_attempt_id == candidate.claim.attempt_id
        && review.stage_121_claim_sha256 == candidate.claim.claim_sha256
        && review.stage_122_result_id == candidate.result.result_id
        && review.stage_122_result_sha256 == candidate.result.result_sha256
        && review.stage_122_candidate_sha256 == candidate.candidate.candidate_sha256
        && review.stage_123_validation_id == candidate.validation.validation_id
        && review.stage_123_validation_sha256 == candidate.validation.validation_sha256
        && review.stage_114_review_sha256 == candidate.validation.stage_114_review_sha256
        && review.stage_112_output_sha256 == candidate.validation.stage_112_output_sha256
        && review.stage_123_validated_at == candidate.validation.validated_at
        && review.submitted_at >= review.stage_123_validated_at
        && review.notice_count == candidate.candidate.notices.len()
        && review.event_type_counts == event_type_counts(candidate)
        && !review.event_type_counts.is_empty()
        && review.original_candidate_remains_untrusted_and_immutable
        && candidate.candidate.financial_event_allowlist.is_empty()
        && !candidate.candidate.opening_portfolio_snapshot_admitted
        && expected_admitted == review.formal_non_financial_observation_evidence_admitted
        && review.future_stage_125_opening_portfolio_snapshot_governance_specification_eligible
            == review.formal_non_financial_observation_evidence_admitted
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 124 admission review 完整性校验失败".to_string())
}

async fn latest_review(
    state: &AppState,
    candidate: &IndependentlyValidatedNonFinancialObservationNoticeCandidate,
) -> Result<Option<ControlledShadowObservationLedgerTransitionCandidateAdmissionReview>, String> {
    let reviews = read_reviews(state, &candidate.claim.attempt_id).await?;
    if reviews.is_empty() {
        return Ok(None);
    }
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    let mut referenced = BTreeSet::new();
    let by_hash = reviews
        .iter()
        .map(|value| (value.review_sha256.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    for review in &reviews {
        validate_review(review, candidate)?;
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err("Stage 124 admission review 出现重复记录".to_string());
        }
        let previous_review = if let Some(previous) = review.previous_review_sha256.as_deref() {
            let previous_review = by_hash
                .get(previous)
                .copied()
                .ok_or_else(|| "Stage 124 admission review 链断裂".to_string())?;
            if review.previous_review_id.as_deref() != Some(previous_review.review_id.as_str()) {
                return Err("Stage 124 admission review 前序 ID/hash 不一致".to_string());
            }
            if previous_review.formal_non_financial_observation_evidence_admitted {
                return Err("Stage 124 已批准终态之后不得追加复核记录".to_string());
            }
            if !referenced.insert(previous.to_string()) {
                return Err("Stage 124 admission review 链分叉或断裂".to_string());
            }
            Some((*previous_review).clone())
        } else {
            None
        };
        if review.excluded_prior_actor_ids != excluded_actor_ids(candidate, &previous_review) {
            return Err("Stage 124 admission review 责任链排除集合不完整或漂移".to_string());
        }
    }
    let roots = reviews
        .iter()
        .filter(|value| value.previous_review_sha256.is_none())
        .count();
    let tips = reviews
        .iter()
        .filter(|value| !referenced.contains(&value.review_sha256))
        .collect::<Vec<_>>();
    if roots != 1 || tips.len() != 1 {
        return Err("Stage 124 admission review 链不是唯一线性链".to_string());
    }
    Ok(Some((*tips[0]).clone()))
}

async fn read_reviews(
    state: &AppState,
    attempt_id: &str,
) -> Result<Vec<ControlledShadowObservationLedgerTransitionCandidateAdmissionReview>, String> {
    let mut entries = match tokio::fs::read_dir(review_directory(state, attempt_id)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut values = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let metadata = tokio::fs::symlink_metadata(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() == 0
            || metadata.len() > MAX_FILE_BYTES
        {
            return Err("Stage 124 admission review 文件无效或超限".to_string());
        }
        values.push(
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
        );
    }
    Ok(values)
}

fn validate_request(
    attempt_id: &str,
    request: &ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest,
) -> Result<(), String> {
    let valid = valid_id(attempt_id)
        && request.expected_previous_review_id.is_some()
            == request.expected_previous_review_sha256.is_some()
        && request
            .expected_previous_review_id
            .as_deref()
            .is_none_or(valid_id)
        && request
            .expected_previous_review_sha256
            .as_deref()
            .is_none_or(valid_sha256)
        && valid_id(&request.expected_stage_123_validation_id)
        && [
            &request.expected_stage_123_validation_sha256,
            &request.expected_stage_122_result_sha256,
            &request.expected_stage_122_candidate_sha256,
            &request.expected_stage_121_claim_sha256,
            &request.expected_stage_114_review_sha256,
            &request.expected_stage_112_output_sha256,
        ]
        .into_iter()
        .all(|value| valid_sha256(value));
    valid
        .then_some(())
        .ok_or_else(|| "Stage 124 请求绑定格式无效".to_string())
}

fn review_fingerprint(
    review: &ControlledShadowObservationLedgerTransitionCandidateAdmissionReview,
) -> Result<String, String> {
    let mut value = serde_json::to_value(review).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 124 review fingerprint payload 无效".to_string())?;
    object.remove("review_id");
    object.remove("review_sha256");
    Ok(hash_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn hash_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}为空或超过 {MAX_TEXT_CHARS} 字符"))
    } else {
        Ok(value.to_string())
    }
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("Stage 124 admission review 文件为空或超限".to_string());
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-observation-ledger-transition-candidate-admission-reviews")
        .join(attempt_id)
}

struct AdmissionLock(PathBuf);

impl Drop for AdmissionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<AdmissionLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage124-{attempt_id}.lock"));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 124 candidate admission 正在进行".to_string())?;
    Ok(AdmissionLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewer_must_be_external_to_full_prior_chain() {
        let excluded = vec![
            "validator".to_string(),
            "executor".to_string(),
            "claimant".to_string(),
        ];
        assert!(!actor_is_independent("validator", &excluded));
        assert!(actor_is_independent("stage124-reviewer", &excluded));
    }

    #[test]
    fn approval_only_opens_opening_portfolio_governance_specification() {
        assert!(NEXT_GATE.contains("opening_portfolio_snapshot_governance_specification"));
        assert!(!NEXT_GATE.contains("execution"));
        assert!(!NEXT_GATE.contains("trade"));
    }

    #[test]
    fn request_requires_exact_hashes_and_linear_previous_pair() {
        let request = ReviewControlledShadowObservationLedgerTransitionCandidateAdmissionRequest {
            expected_previous_review_id: None,
            expected_previous_review_sha256: None,
            expected_stage_123_validation_id: "1".repeat(32),
            expected_stage_123_validation_sha256: "2".repeat(64),
            expected_stage_122_result_sha256: "3".repeat(64),
            expected_stage_122_candidate_sha256: "4".repeat(64),
            expected_stage_121_claim_sha256: "5".repeat(64),
            expected_stage_114_review_sha256: "6".repeat(64),
            expected_stage_112_output_sha256: "7".repeat(64),
            verdict: ControlledShadowObservationLedgerTransitionCandidateAdmissionVerdict::ChangesRequested,
            rationale: "test".into(),
            known_limitations: "opening portfolio absent".into(),
            exact_current_stage_51_through_stage_123_binding_confirmed: false,
            reviewer_independent_from_validator_executor_claimant_and_complete_prior_chain_confirmed: false,
            stage_123_terminal_validation_reopened_rehashed_and_current_confirmed: false,
            stage_122_candidate_reopened_rehashed_and_exact_match_confirmed: false,
            exact_stage_114_admitted_observation_binding_preserved_confirmed: false,
            every_non_financial_notice_identity_decimal_hash_and_order_preserved_confirmed: false,
            admission_creates_separate_formal_non_financial_evidence_record_without_mutating_candidate_confirmed: false,
            opening_portfolio_absent_financial_allowlist_empty_and_no_authoritative_ledger_event_confirmed: false,
            approval_only_opens_stage_125_opening_portfolio_snapshot_governance_specification_confirmed: false,
            no_position_cash_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: false,
            no_unconfirmed_hari_or_old_wang_logic_claimed: false,
        };
        assert!(validate_request(&"a".repeat(32), &request).is_ok());
    }

    #[test]
    fn empty_financial_state_is_the_only_stage_124_boundary() {
        assert!(POLICY_VERSION.ends_with("no-financial-state"));
        assert!(REVIEW_SCHEMA_VERSION.contains("admission-review-v1"));
    }
}
