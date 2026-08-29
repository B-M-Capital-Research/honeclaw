//! Stage 114 independent admission review for one exact Stage 113 validated observation envelope.
//!
//! Admission makes the exact, still immutable Stage 112 envelope eligible as formal observation
//! evidence for a later ledger-transition specification. It does not mutate the envelope, create a
//! ledger, calculate performance, train a model, generate an order, access a broker or trade.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_output_validations::{
    IndependentlyValidatedControlledShadowObservationEnvelope,
    independently_validated_observation_envelopes_for_stage_114_review,
    revalidated_observation_evidence_for_stage_114_review,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_execution_attempts::UntrustedControlledShadowObservationEnvelope;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-evidence-admission-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-evidence-admission-review-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-validated-observation-envelope-independent-evidence-admission-v1-no-ledger";
const NEXT_GATE: &str = "stage_115_observation_ledger_transition_specification_registration";
const MAX_TEXT_CHARS: usize = 4_000;
const PROVIDER_TIME_LIMITATION: &str =
    "provider_publication_time_unverified_stage_104_custody_time_floor_preserved";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationEvidenceAdmissionVerdict {
    AdmittedForFutureObservationLedgerTransitionSpecificationRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowObservationEvidenceAdmissionRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_stage_113_validation_id: String,
    expected_stage_113_validation_sha256: String,
    expected_stage_112_result_sha256: String,
    expected_stage_112_output_sha256: String,
    expected_stage_111_claim_sha256: String,
    verdict: ControlledShadowObservationEvidenceAdmissionVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_through_stage_113_binding_confirmed: bool,
    reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: bool,
    stage_113_terminal_validation_reopened_rehashed_and_current_confirmed: bool,
    stage_112_envelope_reopened_rehashed_and_reprojected_confirmed: bool,
    exact_stage_104_admitted_input_binding_preserved_confirmed: bool,
    sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed: bool,
    natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed:
        bool,
    provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed: bool,
    admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed: bool,
    approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed:
        bool,
    no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationEvidenceAdmissionReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub stage_111_attempt_id: String,
    pub stage_111_claim_sha256: String,
    pub stage_112_result_id: String,
    pub stage_112_result_sha256: String,
    pub stage_112_output_sha256: String,
    pub stage_113_validation_id: String,
    pub stage_113_validation_sha256: String,
    pub observation_materialization_specification_sha256: String,
    pub stage_104_review_sha256: String,
    pub stage_102_output_sha256: String,
    pub admitted_available_at_utc: DateTime<Utc>,
    pub stage_113_validated_at: DateTime<Utc>,
    pub submitted_at: DateTime<Utc>,
    pub observed_session_count: usize,
    pub observed_price_count: usize,
    pub observed_gap_count: usize,
    pub observed_dividend_count: usize,
    pub observed_split_count: usize,
    pub provider_publication_time_verified: bool,
    pub provider_time_limitation: String,
    pub original_envelope_remains_untrusted_and_immutable: bool,
    pub submitted_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowObservationEvidenceAdmissionVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_current_stage_51_through_stage_113_binding_confirmed: bool,
    pub reviewer_independent_from_validator_executor_and_complete_prior_chain: bool,
    pub stage_113_terminal_validation_reopened_rehashed_and_current_confirmed: bool,
    pub stage_112_envelope_reopened_rehashed_and_reprojected_confirmed: bool,
    pub exact_stage_104_admitted_input_binding_preserved_confirmed: bool,
    pub sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed: bool,
    pub natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed:
        bool,
    pub provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed: bool,
    pub admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed:
        bool,
    pub approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed:
        bool,
    pub no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub current_custody_and_full_reprojection_revalidated: bool,
    pub observation_evidence_admitted: bool,
    pub future_observation_ledger_transition_specification_registration_eligible: bool,
    pub observation_ledger_transition_specification_registered: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub nav_or_performance_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationEvidenceAdmissionItem {
    pub candidate: IndependentlyValidatedControlledShadowObservationEnvelope,
    pub latest_review: Option<ControlledShadowObservationEvidenceAdmissionReview>,
    pub current_binding: bool,
    pub review_eligible: bool,
    pub observation_evidence_admitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationEvidenceAdmissionRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowObservationEvidenceAdmissionItem>,
    pub independently_validated_candidate_count: usize,
    pub review_eligible_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_observation_evidence_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_observation_ledger_transition_specification_registration_eligible_count: usize,
    pub admission_status: String,
    pub next_gate: String,
    pub admission_review_available: bool,
    pub provider_publication_time_verified: bool,
    pub original_envelope_mutated: bool,
    pub ledger_created: bool,
    pub nav_or_performance_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationEvidenceAdmissionReadinessSummary {
    pub independently_validated_candidate_count: usize,
    pub review_eligible_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_observation_evidence_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_observation_ledger_transition_specification_registration_eligible_count: usize,
    pub admission_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct AdmittedControlledShadowObservationEvidence {
    pub candidate: IndependentlyValidatedControlledShadowObservationEnvelope,
    pub admission_review: ControlledShadowObservationEvidenceAdmissionReview,
    pub envelope: UntrustedControlledShadowObservationEnvelope,
}

#[allow(dead_code)]
pub(crate) async fn admitted_controlled_shadow_observation_evidence_for_stage_115(
    state: &AppState,
) -> Result<Vec<AdmittedControlledShadowObservationEvidence>, String> {
    let registry = build_registry(state).await?;
    let mut admitted = Vec::new();
    for item in registry.items {
        let Some(review) = item.latest_review else {
            continue;
        };
        if !item.current_binding
            || !item.observation_evidence_admitted
            || !review.future_observation_ledger_transition_specification_registration_eligible
        {
            continue;
        }
        let revalidated = revalidated_observation_evidence_for_stage_114_review(
            state,
            &item.candidate.claim.attempt_id,
            &item.candidate.validation.validation_sha256,
        )
        .await?;
        if revalidated.evidence.claim.claim_sha256 != item.candidate.claim.claim_sha256
            || revalidated.evidence.result.result_sha256 != item.candidate.result.result_sha256
            || revalidated.evidence.validation.validation_sha256
                != item.candidate.validation.validation_sha256
        {
            return Err("Stage 115 取用时 Stage 113 证据链发生漂移".to_string());
        }
        admitted.push(AdmittedControlledShadowObservationEvidence {
            candidate: item.candidate,
            admission_review: review,
            envelope: revalidated.envelope,
        });
    }
    Ok(admitted)
}

pub(crate) async fn handle_get_controlled_shadow_observation_evidence_admission_reviews(
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
            warn!(%error, "observation evidence admission registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察证据独立准入复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_observation_evidence_admission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowObservationEvidenceAdmissionRequest>,
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

pub(crate) async fn controlled_shadow_observation_evidence_admission_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationEvidenceAdmissionReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowObservationEvidenceAdmissionReadinessSummary {
            independently_validated_candidate_count: value.independently_validated_candidate_count,
            review_eligible_candidate_count: value.review_eligible_candidate_count,
            reviewed_candidate_count: value.reviewed_candidate_count,
            admitted_observation_evidence_count: value.admitted_observation_evidence_count,
            changes_requested_or_rejected_count: value.changes_requested_or_rejected_count,
            future_observation_ledger_transition_specification_registration_eligible_count: value
                .future_observation_ledger_transition_specification_registration_eligible_count,
            admission_status: value.admission_status,
        },
    )
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationEvidenceAdmissionRegistry, String> {
    let candidates =
        independently_validated_observation_envelopes_for_stage_114_review(state).await?;
    let mut items = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let latest_review = latest_review(state, &candidate).await?;
        let admitted = latest_review
            .as_ref()
            .is_some_and(|value| value.observation_evidence_admitted);
        items.push(ControlledShadowObservationEvidenceAdmissionItem {
            current_binding: true,
            review_eligible: !admitted,
            observation_evidence_admitted: admitted,
            candidate,
            latest_review,
        });
    }
    let independently_validated_candidate_count = items.len();
    let review_eligible_candidate_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_candidate_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let admitted_observation_evidence_count = items
        .iter()
        .filter(|item| item.observation_evidence_admitted)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    != ControlledShadowObservationEvidenceAdmissionVerdict::AdmittedForFutureObservationLedgerTransitionSpecificationRegistration
            })
        })
        .count();
    let admission_status = if admitted_observation_evidence_count > 0 {
        "observation_evidence_admitted_waiting_stage_115_ledger_transition_specification"
    } else if changes_requested_or_rejected_count > 0 {
        "observation_evidence_admission_changes_requested_or_rejected"
    } else if independently_validated_candidate_count > 0 {
        "waiting_independent_observation_evidence_admission_review"
    } else {
        "waiting_stage_113_independently_validated_observation_envelope"
    };
    Ok(ControlledShadowObservationEvidenceAdmissionRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        independently_validated_candidate_count,
        review_eligible_candidate_count,
        reviewed_candidate_count,
        admitted_observation_evidence_count,
        changes_requested_or_rejected_count,
        future_observation_ledger_transition_specification_registration_eligible_count:
            admitted_observation_evidence_count,
        admission_status: admission_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        admission_review_available: true,
        provider_publication_time_verified: false,
        original_envelope_mutated: false,
        ledger_created: false,
        nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 114 由 Stage 113 validator、Stage 112 executor 与 Stage 51–113 完整责任链之外的新管理员复核。服务端在写入和读取时重新打开并重哈希 Stage 113 终态与 exact Stage 112 envelope，再执行完整 Stage 113 独立重投影。批准只创建分离、自哈希、追加式证据准入记录，并开放 Stage 115 账本转换规格登记；原 envelope 保持 untrusted/immutable，供应商发布时间仍未验证，不建账、不算净值/绩效、不训练/RL/reward、不生成订单、不接券商、不交易。".to_string(),
    })
}

async fn review_admission(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewControlledShadowObservationEvidenceAdmissionRequest,
) -> Result<ControlledShadowObservationEvidenceAdmissionReview, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    let revalidated = revalidated_observation_evidence_for_stage_114_review(
        state,
        attempt_id,
        &request.expected_stage_113_validation_sha256,
    )
    .await?;
    let candidate = revalidated.evidence;
    verify_expected_bindings(&candidate, &request)?;
    let latest = latest_review(state, &candidate).await?;
    if latest
        .as_ref()
        .is_some_and(|value| value.observation_evidence_admitted)
    {
        return Err("该精确观察证据已经准入，复核链永久冻结".to_string());
    }
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_previous_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_previous_review_sha256.as_deref()
    {
        return Err("Stage 114 复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&candidate, &latest);
    let independent = actor_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent
        || !request.reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed
    {
        return Err("Stage 114 reviewer 必须独立于 Stage 113 validator、Stage 112 executor 和完整既有责任链".to_string());
    }
    let validation = &candidate.validation;
    let specification = &candidate
        .claim
        .authorization
        .runner
        .implementation
        .upstream_specification_registration
        .specification;
    let mut review = ControlledShadowObservationEvidenceAdmissionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        stage_111_attempt_id: candidate.claim.attempt_id.clone(),
        stage_111_claim_sha256: candidate.claim.claim_sha256.clone(),
        stage_112_result_id: candidate.result.result_id.clone(),
        stage_112_result_sha256: candidate.result.result_sha256.clone(),
        stage_112_output_sha256: candidate.result.output_sha256.clone().unwrap_or_default(),
        stage_113_validation_id: validation.validation_id.clone(),
        stage_113_validation_sha256: validation.validation_sha256.clone(),
        observation_materialization_specification_sha256: specification
            .specification_sha256
            .clone(),
        stage_104_review_sha256: specification.stage_104_review_sha256.clone(),
        stage_102_output_sha256: specification.stage_102_output_sha256.clone(),
        admitted_available_at_utc: revalidated.envelope.admitted_available_at_utc,
        stage_113_validated_at: validation.validated_at,
        submitted_at: Utc::now(),
        observed_session_count: validation.observed_session_count,
        observed_price_count: validation.observed_price_count,
        observed_gap_count: validation.observed_gap_count,
        observed_dividend_count: validation.observed_dividend_count,
        observed_split_count: validation.observed_split_count,
        provider_publication_time_verified: false,
        provider_time_limitation: PROVIDER_TIME_LIMITATION.to_string(),
        original_envelope_remains_untrusted_and_immutable: revalidated.envelope.untrusted,
        submitted_by: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核依据")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        exact_current_stage_51_through_stage_113_binding_confirmed: request
            .exact_current_stage_51_through_stage_113_binding_confirmed,
        reviewer_independent_from_validator_executor_and_complete_prior_chain: independent,
        stage_113_terminal_validation_reopened_rehashed_and_current_confirmed: request
            .stage_113_terminal_validation_reopened_rehashed_and_current_confirmed,
        stage_112_envelope_reopened_rehashed_and_reprojected_confirmed: request
            .stage_112_envelope_reopened_rehashed_and_reprojected_confirmed,
        exact_stage_104_admitted_input_binding_preserved_confirmed: request
            .exact_stage_104_admitted_input_binding_preserved_confirmed,
        sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed:
            request.sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed,
        natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed:
            request.natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed,
        provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed: request
            .provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed,
        admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed:
            request.admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed,
        approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed:
            request.approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed,
        no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed:
            request.no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        current_custody_and_full_reprojection_revalidated: true,
        observation_evidence_admitted: false,
        future_observation_ledger_transition_specification_registration_eligible: false,
        observation_ledger_transition_specification_registered: false,
        ledger_created: false,
        position_written: false,
        nav_or_performance_written: false,
        model_or_metric_store_written: false,
        training_or_rl_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == ControlledShadowObservationEvidenceAdmissionVerdict::AdmittedForFutureObservationLedgerTransitionSpecificationRegistration;
    review.observation_evidence_admitted = approved
        && independent
        && review.current_custody_and_full_reprojection_revalidated
        && review_all_checks(&review)
        && review.original_envelope_remains_untrusted_and_immutable;
    review.future_observation_ledger_transition_specification_registration_eligible =
        review.observation_evidence_admitted;
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

fn review_all_checks(review: &ControlledShadowObservationEvidenceAdmissionReview) -> bool {
    review.exact_current_stage_51_through_stage_113_binding_confirmed
        && review.reviewer_independent_from_validator_executor_and_complete_prior_chain
        && review.stage_113_terminal_validation_reopened_rehashed_and_current_confirmed
        && review.stage_112_envelope_reopened_rehashed_and_reprojected_confirmed
        && review.exact_stage_104_admitted_input_binding_preserved_confirmed
        && review.sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed
        && review.natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed
        && review.provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed
        && review.admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed
        && review.approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed
        && review.no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn verify_expected_bindings(
    candidate: &IndependentlyValidatedControlledShadowObservationEnvelope,
    request: &ReviewControlledShadowObservationEvidenceAdmissionRequest,
) -> Result<(), String> {
    let valid = request.expected_stage_113_validation_id == candidate.validation.validation_id
        && request.expected_stage_113_validation_sha256 == candidate.validation.validation_sha256
        && request.expected_stage_112_result_sha256 == candidate.result.result_sha256
        && candidate.result.output_sha256.as_deref()
            == Some(request.expected_stage_112_output_sha256.as_str())
        && request.expected_stage_111_claim_sha256 == candidate.claim.claim_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 114 预期绑定与当前 Stage 111–113 精确链不一致".to_string())
}

fn validate_request(
    attempt_id: &str,
    request: &ReviewControlledShadowObservationEvidenceAdmissionRequest,
) -> Result<(), String> {
    let previous_pair = request.expected_previous_review_id.is_some()
        == request.expected_previous_review_sha256.is_some();
    let valid = valid_id(attempt_id)
        && previous_pair
        && request
            .expected_previous_review_id
            .as_deref()
            .map_or(true, valid_id)
        && request
            .expected_previous_review_sha256
            .as_deref()
            .map_or(true, valid_sha256)
        && valid_id(&request.expected_stage_113_validation_id)
        && [
            &request.expected_stage_113_validation_sha256,
            &request.expected_stage_112_result_sha256,
            &request.expected_stage_112_output_sha256,
            &request.expected_stage_111_claim_sha256,
        ]
        .into_iter()
        .all(|value| valid_sha256(value));
    valid
        .then_some(())
        .ok_or_else(|| "Stage 114 请求绑定格式无效".to_string())
}

fn excluded_actor_ids(
    candidate: &IndependentlyValidatedControlledShadowObservationEnvelope,
    latest: &Option<ControlledShadowObservationEvidenceAdmissionReview>,
) -> Vec<String> {
    let mut values = candidate.validation.excluded_prior_actor_ids.clone();
    values.push(candidate.validation.validated_by.clone());
    values.push(candidate.result.executed_by.clone());
    values.push(candidate.claim.claimed_by.clone());
    values.push(candidate.claim.authorization.reviewer_id.clone());
    values.push(candidate.claim.authorization.runner.registered_by.clone());
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
    review: &ControlledShadowObservationEvidenceAdmissionReview,
    candidate: &IndependentlyValidatedControlledShadowObservationEnvelope,
) -> Result<(), String> {
    let specification = &candidate
        .claim
        .authorization
        .runner
        .implementation
        .upstream_specification_registration
        .specification;
    let approved = review.verdict
        == ControlledShadowObservationEvidenceAdmissionVerdict::AdmittedForFutureObservationLedgerTransitionSpecificationRegistration;
    let independent = actor_is_independent(&review.submitted_by, &review.excluded_prior_actor_ids);
    let expected_admitted = approved
        && independent
        && review.current_custody_and_full_reprojection_revalidated
        && review.original_envelope_remains_untrusted_and_immutable
        && review_all_checks(review);
    let downstream_closed = !review.observation_ledger_transition_specification_registered
        && !review.ledger_created
        && !review.position_written
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
        && review.previous_review_id.as_deref().map_or(true, valid_id)
        && review
            .previous_review_sha256
            .as_deref()
            .map_or(true, valid_sha256)
        && review.stage_111_attempt_id == candidate.claim.attempt_id
        && review.stage_111_claim_sha256 == candidate.claim.claim_sha256
        && review.stage_112_result_id == candidate.result.result_id
        && review.stage_112_result_sha256 == candidate.result.result_sha256
        && candidate.result.output_sha256.as_deref()
            == Some(review.stage_112_output_sha256.as_str())
        && review.stage_113_validation_id == candidate.validation.validation_id
        && review.stage_113_validation_sha256 == candidate.validation.validation_sha256
        && review.observation_materialization_specification_sha256
            == specification.specification_sha256
        && review.stage_104_review_sha256 == specification.stage_104_review_sha256
        && review.stage_102_output_sha256 == specification.stage_102_output_sha256
        && review.stage_113_validated_at == candidate.validation.validated_at
        && review.submitted_at >= review.stage_113_validated_at
        && review.observed_session_count == candidate.validation.observed_session_count
        && review.observed_price_count == candidate.validation.observed_price_count
        && review.observed_gap_count == candidate.validation.observed_gap_count
        && review.observed_dividend_count == candidate.validation.observed_dividend_count
        && review.observed_split_count == candidate.validation.observed_split_count
        && !review.provider_publication_time_verified
        && review.provider_time_limitation == PROVIDER_TIME_LIMITATION
        && review.original_envelope_remains_untrusted_and_immutable
        && !review.submitted_by.trim().is_empty()
        && sorted_unique(&review.excluded_prior_actor_ids)
        && independent
        && review.reviewer_independent_from_validator_executor_and_complete_prior_chain
            == independent
        && !review.rationale.trim().is_empty()
        && review.rationale.chars().count() <= MAX_TEXT_CHARS
        && !review.known_limitations.trim().is_empty()
        && review.known_limitations.chars().count() <= MAX_TEXT_CHARS
        && (!approved || review_all_checks(review))
        && review.observation_evidence_admitted == expected_admitted
        && review.future_observation_ledger_transition_specification_registration_eligible
            == expected_admitted
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 114 观察证据准入复核记录无效、漂移或越权".to_string())
}

async fn latest_review(
    state: &AppState,
    candidate: &IndependentlyValidatedControlledShadowObservationEnvelope,
) -> Result<Option<ControlledShadowObservationEvidenceAdmissionReview>, String> {
    let directory = review_directory(state, &candidate.claim.attempt_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    // A persisted Stage 114 decision is meaningful only while the exact Stage 113 validation and
    // Stage 112 envelope still reproduce. Re-run the complete independent projection before
    // trusting any review record; a valid self-hash alone must never mask upstream custody drift.
    let revalidated = revalidated_observation_evidence_for_stage_114_review(
        state,
        &candidate.claim.attempt_id,
        &candidate.validation.validation_sha256,
    )
    .await?;
    let mut reviews = Vec::new();
    let mut ids = BTreeSet::new();
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
            || !metadata.file_type().is_file()
            || metadata.len() == 0
            || metadata.len() > 256 * 1024
        {
            return Err("Stage 114 review 文件无效或超限".to_string());
        }
        let review: ControlledShadowObservationEvidenceAdmissionReview = serde_json::from_slice(
            &tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        validate_review(&review, candidate)?;
        if review.admitted_available_at_utc != revalidated.envelope.admitted_available_at_utc {
            return Err(
                "Stage 114 review 的 available_at 与当前重投影 envelope 不一致".to_string(),
            );
        }
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
        {
            return Err("Stage 114 review 文件名或唯一性无效".to_string());
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[ControlledShadowObservationEvidenceAdmissionReview],
) -> Result<Option<ControlledShadowObservationEvidenceAdmissionReview>, String> {
    if reviews.is_empty() {
        return Ok(None);
    }
    let by_id = reviews
        .iter()
        .map(|value| (value.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let tips = reviews
        .iter()
        .filter(|value| {
            !reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(value.review_id.as_str()))
        })
        .collect::<Vec<_>>();
    if tips.len() != 1 {
        return Err("Stage 114 review chain 分叉或缺少唯一链尖".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("Stage 114 review chain 存在循环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "Stage 114 review chain 断裂".to_string())?;
                if previous.review_sha256 != *sha {
                    return Err("Stage 114 previous review SHA-256 不匹配".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("Stage 114 previous review 绑定不完整".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("Stage 114 review chain 含游离记录".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn review_fingerprint(
    review: &ControlledShadowObservationEvidenceAdmissionReview,
) -> Result<String, String> {
    let mut value = serde_json::to_value(review).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 114 fingerprint payload 无效".to_string())?;
    object.remove("review_id");
    object.remove("review_sha256");
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}为空或超过 {MAX_TEXT_CHARS} 字符"))
    } else {
        Ok(value.to_string())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
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
        .join("controlled-shadow-observation-evidence-admission-reviews")
        .join(attempt_id)
}

struct AdmissionLock {
    path: PathBuf,
}

impl Drop for AdmissionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<AdmissionLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage114-{attempt_id}.lock"));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|value| SystemTime::now().duration_since(value).ok())
            .is_some_and(|value| value > StdDuration::from_secs(300));
        if stale {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 114 观察证据准入复核正在进行".to_string())?;
    Ok(AdmissionLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reviewer_must_be_chain_external() {
        let actors = vec!["executor".to_string(), "validator".to_string()];
        assert!(!actor_is_independent("validator", &actors));
        assert!(actor_is_independent("stage114-reviewer", &actors));
    }

    #[test]
    fn provider_time_remains_unverified_and_next_gate_is_only_a_specification() {
        assert!(PROVIDER_TIME_LIMITATION.contains("provider_publication_time_unverified"));
        assert!(NEXT_GATE.contains("specification_registration"));
        assert!(!NEXT_GATE.contains("execution"));
    }

    #[test]
    fn request_requires_exact_hash_shapes_and_previous_pair() {
        let request = ReviewControlledShadowObservationEvidenceAdmissionRequest {
            expected_previous_review_id: None,
            expected_previous_review_sha256: None,
            expected_stage_113_validation_id: "1".repeat(32),
            expected_stage_113_validation_sha256: "2".repeat(64),
            expected_stage_112_result_sha256: "3".repeat(64),
            expected_stage_112_output_sha256: "4".repeat(64),
            expected_stage_111_claim_sha256: "5".repeat(64),
            verdict: ControlledShadowObservationEvidenceAdmissionVerdict::ChangesRequested,
            rationale: "test".into(),
            known_limitations: "provider time unverified".into(),
            exact_current_stage_51_through_stage_113_binding_confirmed: false,
            reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: false,
            stage_113_terminal_validation_reopened_rehashed_and_current_confirmed: false,
            stage_112_envelope_reopened_rehashed_and_reprojected_confirmed: false,
            exact_stage_104_admitted_input_binding_preserved_confirmed: false,
            sessions_prices_gaps_actions_allocation_and_available_at_exactly_preserved_confirmed: false,
            natural_forward_only_no_refetch_fill_substitution_rewrite_correction_or_backfill_confirmed: false,
            provider_publication_time_unverified_and_custody_time_floor_preserved_confirmed: false,
            admission_preserves_original_envelope_and_only_creates_separate_evidence_record_confirmed: false,
            approval_only_opens_future_observation_ledger_transition_specification_registration_confirmed: false,
            no_ledger_position_nav_performance_model_metric_training_rl_reward_order_broker_or_trading_confirmed: false,
            no_unconfirmed_hari_or_old_wang_logic_claimed: false,
        };
        assert!(validate_request(&"a".repeat(32), &request).is_ok());
    }
}
