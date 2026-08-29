//! Stage 123 chain-external validation of one Stage 122 non-financial notice candidate.
//!
//! The validator never calls the Stage 122 projector. It reopens the immutable candidate and the
//! exact Stage 114-admitted observation envelope, rebuilds every notice with a second
//! implementation, and compares the complete candidate. Passing only opens a future Stage 124
//! admission review. It never creates an authoritative ledger or portfolio financial state.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_evidence_admission_reviews::{
    AdmittedControlledShadowObservationEvidence,
    admitted_controlled_shadow_observation_evidence_for_stage_115,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_execution_attempt_claims::{
    ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    observation_ledger_transition_execution_attempt_claims_for_execution,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_execution_attempts::{
    CanonicalNonFinancialObservationNoticeCandidate,
    ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
    ControlledShadowObservationLedgerTransitionExecutionStatus,
    UntrustedNonFinancialObservationNoticeCandidateEnvelope,
    controlled_shadow_observation_ledger_transition_execution_custody_root,
    observation_ledger_transition_execution_results_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_specifications::ControlledShadowObservationLedgerTransitionSpecification;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-output-validation-v1";
const OUTPUT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-untrusted-non-financial-observation-notice-candidate-envelope-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-ledger-transition-chain-external-full-candidate-reprojection-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-observation-ledger-transition-validator-v1-independent-second-projection";
const NEXT_GATE: &str =
    "stage_124_validated_non_financial_observation_notice_candidate_admission_review";
const MAX_REASON_CHARS: usize = 4_000;
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateControlledShadowObservationLedgerTransitionOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_candidate_sha256: String,
    expected_specification_sha256: String,
    expected_stage_114_review_sha256: String,
    expected_stage_112_output_sha256: String,
    validation_reason: String,
    exact_current_stage_51_through_stage_122_binding_confirmed: bool,
    validator_independent_from_executor_claimant_and_complete_prior_chain_confirmed: bool,
    stage_122_result_and_create_once_candidate_reopened_and_rehashed_confirmed: bool,
    exact_stage_114_admitted_observation_envelope_reopened_and_rehashed_confirmed: bool,
    second_projection_does_not_call_stage_122_projector_helpers_confirmed: bool,
    every_notice_identity_decimal_hash_sort_and_complete_candidate_exactly_compared_confirmed: bool,
    opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed: bool,
    pass_only_opens_future_stage_124_non_financial_candidate_admission_review_confirmed: bool,
    no_ledger_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationLedgerTransitionOutputValidationVerdict {
    IndependentlyValidatedExactNonFinancialNoticeCandidate,
    FailedIndependentNonFinancialNoticeCandidateValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub stage_121_attempt_id: String,
    pub stage_121_claim_sha256: String,
    pub stage_122_result_id: String,
    pub stage_122_result_sha256: String,
    pub stage_122_candidate_sha256: String,
    pub observation_ledger_transition_specification_sha256: String,
    pub stage_114_review_sha256: String,
    pub stage_112_output_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub validation_reason: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validator_independent_from_executor_claimant_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_122_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub candidate_file_custody_and_fingerprint_verified: bool,
    pub exact_stage_114_admitted_observation_revalidated: bool,
    pub complete_candidate_independently_reprojected: bool,
    pub every_notice_identity_and_hash_independently_verified: bool,
    pub exact_decimal_fields_independently_verified: bool,
    pub canonical_sort_and_complete_candidate_exact_match_verified: bool,
    pub opening_portfolio_absence_and_empty_financial_allowlist_verified: bool,
    pub no_downstream_authority_verified: bool,
    pub recomputed_claim_sha256: String,
    pub recomputed_result_sha256: String,
    pub recomputed_persisted_candidate_sha256: String,
    pub independently_recomputed_candidate_sha256: String,
    pub observed_candidate_bytes: usize,
    pub observed_notice_count: usize,
    pub observed_event_type_counts: BTreeMap<String, usize>,
    pub mismatch_reasons: Vec<String>,
    pub verdict: ControlledShadowObservationLedgerTransitionOutputValidationVerdict,
    pub non_financial_notice_candidate_independently_validated: bool,
    pub future_stage_124_non_financial_candidate_admission_review_eligible: bool,
    pub candidate_remains_untrusted: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub authoritative_ledger_event_created: bool,
    pub position_written: bool,
    pub cash_written: bool,
    pub nav_or_performance_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionOutputValidationItem {
    pub claim: ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    pub result: ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
    pub validation: Option<ControlledShadowObservationLedgerTransitionOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<ControlledShadowObservationLedgerTransitionOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_candidate_count: usize,
    pub failed_validation_count: usize,
    pub future_stage_124_admission_review_eligible_count: usize,
    pub validation_status: String,
    pub next_gate: String,
    pub independent_output_validation_available: bool,
    pub candidate_remains_untrusted: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub authoritative_ledger_event_created: bool,
    pub nav_or_performance_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationLedgerTransitionOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_candidate_count: usize,
    pub failed_validation_count: usize,
    pub future_stage_124_admission_review_eligible_count: usize,
    pub validation_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyValidatedNonFinancialObservationNoticeCandidate {
    pub claim: ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    pub result: ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
    pub validation: ControlledShadowObservationLedgerTransitionOutputValidationRecord,
    pub candidate: UntrustedNonFinancialObservationNoticeCandidateEnvelope,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    candidate_file: bool,
    admitted_observation: bool,
    reprojected: bool,
    notice_hashes: bool,
    exact_decimals: bool,
    exact_candidate: bool,
    opening_boundary: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_persisted_candidate_sha256: String,
    independently_recomputed_candidate_sha256: String,
    candidate_bytes: usize,
    notice_count: usize,
    event_type_counts: BTreeMap<String, usize>,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_controlled_shadow_observation_ledger_transition_output_validations(
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
            warn!(%error, "observation-ledger transition output validation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察到账本转换候选独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_controlled_shadow_observation_ledger_transition_output_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateControlledShadowObservationLedgerTransitionOutputRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match validate_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_observation_ledger_transition_output_validation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowObservationLedgerTransitionOutputValidationReadinessSummary {
            validation_eligible_count: registry.validation_eligible_count,
            validation_count: registry.validation_count,
            independently_validated_candidate_count: registry
                .independently_validated_candidate_count,
            failed_validation_count: registry.failed_validation_count,
            future_stage_124_admission_review_eligible_count: registry
                .future_stage_124_admission_review_eligible_count,
            validation_status: registry.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_non_financial_candidates_for_stage_124_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedNonFinancialObservationNoticeCandidate>, String> {
    let registry = build_registry(state).await?;
    let mut values = Vec::new();
    for item in registry.items {
        let Some(validation) = item.validation else {
            continue;
        };
        if !validation.non_financial_notice_candidate_independently_validated
            || !validation.future_stage_124_non_financial_candidate_admission_review_eligible
        {
            continue;
        }
        let (candidate, _) = independent_read_candidate(state, &item.claim, &item.result).await?;
        values.push(
            IndependentlyValidatedNonFinancialObservationNoticeCandidate {
                claim: item.claim,
                result: item.result,
                validation,
                candidate,
            },
        );
    }
    Ok(values)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionOutputValidationRegistry, String> {
    let claims = observation_ledger_transition_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .map(|value| (value.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let results =
        observation_ledger_transition_execution_results_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let validation_index = validations
        .iter()
        .map(|value| (value.stage_121_attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut items = Vec::new();
    for result in results.into_iter().filter(|value| {
        value.status
            == ControlledShadowObservationLedgerTransitionExecutionStatus::CompletedWithUntrustedNonFinancialNoticeCandidate
    }) {
        let claim = claims
            .get(&result.stage_121_attempt_id)
            .cloned()
            .ok_or_else(|| "Stage 123 candidate 缺少 Stage 121 claim".to_string())?;
        let validation = validation_index
            .get(result.stage_121_attempt_id.as_str())
            .cloned()
            .cloned();
        if let Some(record) = &validation {
            validate_record_binding(record, &claim, &result)?;
        }
        items.push(
            ControlledShadowObservationLedgerTransitionOutputValidationItem {
                validation_eligible: validation.is_none(),
                claim,
                result,
                validation,
            },
        );
    }
    let validation_eligible_count = items
        .iter()
        .filter(|value| value.validation_eligible)
        .count();
    let independently_validated_candidate_count = validations
        .iter()
        .filter(|value| value.non_financial_notice_candidate_independently_validated)
        .count();
    let failed_validation_count = validations
        .len()
        .saturating_sub(independently_validated_candidate_count);
    let validation_status = if failed_validation_count > 0 {
        "observation_ledger_transition_candidate_validation_failed_fail_closed"
    } else if independently_validated_candidate_count > 0 {
        "non_financial_notice_candidate_independently_validated_waiting_stage_124_admission_review"
    } else if validation_eligible_count > 0 {
        "waiting_chain_external_full_non_financial_candidate_reprojection_validation"
    } else {
        "waiting_successful_stage_122_untrusted_non_financial_candidate"
    };
    Ok(
        ControlledShadowObservationLedgerTransitionOutputValidationRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
            validator_implementation_sha256: validator_implementation_sha256(),
            items,
            validation_eligible_count,
            validation_count: validations.len(),
            independently_validated_candidate_count,
            failed_validation_count,
            future_stage_124_admission_review_eligible_count:
                independently_validated_candidate_count,
            validation_status: validation_status.to_string(),
            next_gate: NEXT_GATE.to_string(),
            independent_output_validation_available: true,
            candidate_remains_untrusted: true,
            opening_portfolio_snapshot_admitted: false,
            authoritative_ledger_event_created: false,
            nav_or_performance_written: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "Stage 123 由 Stage 122 executor、Stage 121 claimant 与 Stage 51–122 完整责任链之外的新管理员，使用第二实现重新打开 Stage 122 候选和 exact Stage 114 admitted observation envelope，独立重建每条 notice、精确十进制、摘要、canonical 排序与完整 candidate。通过只开放 Stage 124 非财务候选准入复核；候选仍未受信，不建账、不写持仓/现金/NAV/绩效、不训练/reward、不生成订单、不接券商、不交易。".to_string(),
        },
    )
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateControlledShadowObservationLedgerTransitionOutputRequest,
) -> Result<ControlledShadowObservationLedgerTransitionOutputValidationRecord, String> {
    validate_request_shape(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|value| value.stage_121_attempt_id == attempt_id)
    {
        return Err("这条 Stage 122 候选已经形成不可变独立验证记录，不得重放".to_string());
    }
    let claim = observation_ledger_transition_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .find(|value| value.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有对应的 Stage 121 claim".to_string())?;
    let result = observation_ledger_transition_execution_results_for_independent_validation(state)
        .await?
        .into_iter()
        .find(|value| {
            value.stage_121_attempt_id == attempt_id
                && value.status
                    == ControlledShadowObservationLedgerTransitionExecutionStatus::CompletedWithUntrustedNonFinancialNoticeCandidate
        })
        .ok_or_else(|| "当前没有成功且待独立校验的 Stage 122 非财务候选".to_string())?;
    verify_expected_bindings(&claim, &result, &request)?;
    let excluded_prior_actor_ids = prior_actor_ids(&claim, &result);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "Stage 123 验证者不能是 Stage 122 executor、Stage 121 claimant 或 Stage 51–122 任一既有责任人"
                .to_string(),
        );
    }
    let audit = independently_audit(state, &claim, &result).await;
    let passed = independent
        && audit.exact_chain
        && audit.claim_fingerprint
        && audit.result_fingerprint
        && audit.candidate_file
        && audit.admitted_observation
        && audit.reprojected
        && audit.notice_hashes
        && audit.exact_decimals
        && audit.exact_candidate
        && audit.opening_boundary
        && audit.downstream_closed
        && audit.mismatch_reasons.is_empty();
    let specification = exact_specification(&claim);
    let mut record = ControlledShadowObservationLedgerTransitionOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        stage_121_attempt_id: claim.attempt_id.clone(),
        stage_121_claim_sha256: claim.claim_sha256.clone(),
        stage_122_result_id: result.result_id.clone(),
        stage_122_result_sha256: result.result_sha256.clone(),
        stage_122_candidate_sha256: result.candidate_sha256.clone().unwrap_or_default(),
        observation_ledger_transition_specification_sha256: specification
            .specification_sha256
            .clone(),
        stage_114_review_sha256: specification.stage_114_review_sha256.clone(),
        stage_112_output_sha256: specification.stage_112_output_sha256.clone(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        validation_reason: bounded_required(&request.validation_reason, "验证原因")?,
        excluded_prior_actor_ids,
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validator_independent_from_executor_claimant_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_122_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        candidate_file_custody_and_fingerprint_verified: audit.candidate_file,
        exact_stage_114_admitted_observation_revalidated: audit.admitted_observation,
        complete_candidate_independently_reprojected: audit.reprojected,
        every_notice_identity_and_hash_independently_verified: audit.notice_hashes,
        exact_decimal_fields_independently_verified: audit.exact_decimals,
        canonical_sort_and_complete_candidate_exact_match_verified: audit.exact_candidate,
        opening_portfolio_absence_and_empty_financial_allowlist_verified: audit.opening_boundary,
        no_downstream_authority_verified: audit.downstream_closed,
        recomputed_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_result_sha256: audit.recomputed_result_sha256,
        recomputed_persisted_candidate_sha256: audit.recomputed_persisted_candidate_sha256,
        independently_recomputed_candidate_sha256: audit.independently_recomputed_candidate_sha256,
        observed_candidate_bytes: audit.candidate_bytes,
        observed_notice_count: audit.notice_count,
        observed_event_type_counts: audit.event_type_counts,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            ControlledShadowObservationLedgerTransitionOutputValidationVerdict::IndependentlyValidatedExactNonFinancialNoticeCandidate
        } else {
            ControlledShadowObservationLedgerTransitionOutputValidationVerdict::FailedIndependentNonFinancialNoticeCandidateValidation
        },
        non_financial_notice_candidate_independently_validated: passed,
        future_stage_124_non_financial_candidate_admission_review_eligible: passed,
        candidate_remains_untrusted: true,
        opening_portfolio_snapshot_admitted: false,
        authoritative_ledger_event_created: false,
        position_written: false,
        cash_written: false,
        nav_or_performance_written: false,
        model_or_metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.validation_sha256 = record_fingerprint(&record)?;
    record.validation_id = record.validation_sha256[..32].to_string();
    validate_record(&record)?;
    write_immutable_json(
        &validation_root(state)
            .join(attempt_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

async fn independently_audit(
    state: &AppState,
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    result: &ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
) -> IndependentAudit {
    let mut reasons = Vec::new();
    let specification = exact_specification(claim);
    let recomputed_claim_sha256 =
        independent_fingerprint_without(claim, &["attempt_id", "claim_sha256"]).unwrap_or_else(
            |error| {
                push_reason(&mut reasons, format!("claim fingerprint: {error}"));
                hash_bytes(b"claim-fingerprint-unavailable")
            },
        );
    let recomputed_result_sha256 =
        independent_fingerprint_without(result, &["result_id", "result_sha256"]).unwrap_or_else(
            |error| {
                push_reason(&mut reasons, format!("result fingerprint: {error}"));
                hash_bytes(b"result-fingerprint-unavailable")
            },
        );
    let claim_fingerprint = recomputed_claim_sha256 == claim.claim_sha256;
    let result_fingerprint = recomputed_result_sha256 == result.result_sha256;
    push_check(
        &mut reasons,
        claim_fingerprint,
        "Stage 121 claim fingerprint mismatch",
    );
    push_check(
        &mut reasons,
        result_fingerprint,
        "Stage 122 result fingerprint mismatch",
    );

    let mut candidate_bytes = 0usize;
    let mut recomputed_persisted_candidate_sha256 = hash_bytes(b"persisted-candidate-unavailable");
    let mut persisted: Option<UntrustedNonFinancialObservationNoticeCandidateEnvelope> = None;
    let candidate_file = match independent_read_candidate(state, claim, result).await {
        Ok((value, bytes)) => {
            candidate_bytes = bytes;
            recomputed_persisted_candidate_sha256 =
                independent_fingerprint_without(&value, &["candidate_sha256"]).unwrap_or_else(
                    |error| {
                        push_reason(
                            &mut reasons,
                            format!("persisted candidate fingerprint: {error}"),
                        );
                        hash_bytes(b"persisted-candidate-fingerprint-unavailable")
                    },
                );
            let valid = value.candidate_sha256 == recomputed_persisted_candidate_sha256
                && result.candidate_sha256.as_deref() == Some(value.candidate_sha256.as_str());
            push_check(
                &mut reasons,
                valid,
                "Stage 122 candidate custody or fingerprint mismatch",
            );
            persisted = Some(value);
            valid
        }
        Err(error) => {
            push_reason(&mut reasons, format!("Stage 122 candidate reopen: {error}"));
            false
        }
    };

    let admitted = exact_admitted_evidence(state, specification).await;
    let mut independently_recomputed_candidate_sha256 =
        hash_bytes(b"independent-candidate-unavailable");
    let mut reprojected_candidate = None;
    let (admitted_observation, reprojected) = match admitted {
        Ok(evidence) => match independent_project_candidate(specification, &evidence) {
            Ok(value) => {
                independently_recomputed_candidate_sha256 = value.candidate_sha256.clone();
                reprojected_candidate = Some(value);
                (true, true)
            }
            Err(error) => {
                push_reason(
                    &mut reasons,
                    format!("independent complete candidate projection: {error}"),
                );
                (true, false)
            }
        },
        Err(error) => {
            push_reason(
                &mut reasons,
                format!("exact Stage 114 observation reopen: {error}"),
            );
            (false, false)
        }
    };
    let notice_hashes = persisted
        .as_ref()
        .is_some_and(independent_notice_hashes_and_identities_valid);
    push_check(
        &mut reasons,
        notice_hashes,
        "notice identity or fingerprint mismatch",
    );
    let exact_decimals = persisted.as_ref().is_some_and(|value| {
        value.notices.iter().all(|notice| {
            notice
                .exact_decimal_fields
                .values()
                .all(|decimal| valid_exact_decimal(decimal))
        })
    });
    push_check(
        &mut reasons,
        exact_decimals,
        "notice exact decimal field invalid",
    );
    let exact_candidate = persisted
        .as_ref()
        .zip(reprojected_candidate.as_ref())
        .is_some_and(|(left, right)| left == right);
    push_check(
        &mut reasons,
        exact_candidate,
        "persisted candidate differs from independent second projection",
    );
    let opening_boundary = persisted.as_ref().is_some_and(|value| {
        !value.opening_portfolio_snapshot_admitted
            && value.financial_event_allowlist.is_empty()
            && value.create_once
            && value.untrusted
            && !value.independent_validation_completed
            && !value.ledger_created
            && !value.authoritative_financial_state_created
            && !value.nav_or_performance_calculated
            && !value.order_intent_created
    });
    push_check(
        &mut reasons,
        opening_boundary,
        "opening portfolio or financial-event boundary drifted",
    );
    let downstream_closed = persisted.as_ref().is_some_and(|_| {
        !result.independent_validation_completed
            && !result.opening_portfolio_snapshot_admitted
            && !result.ledger_created
            && !result.ledger_event_written
            && !result.financial_posting_created
            && !result.position_written
            && !result.cash_written
            && !result.nav_or_performance_written
            && !result.model_or_metric_store_written
            && !result.training_feedback_authorized
            && !result.reward_authorized
            && !result.order_generation_authorized
            && !result.broker_access_authorized
            && !result.trading_authorized
    });
    push_check(
        &mut reasons,
        downstream_closed,
        "downstream financial, model or trading authority is not closed",
    );
    let exact_chain = result.stage_121_claim_sha256 == claim.claim_sha256
        && persisted.as_ref().is_some_and(|value| {
            value.specification_sha256 == specification.specification_sha256
                && value.stage_114_review_sha256 == specification.stage_114_review_sha256
                && value.stage_113_validation_sha256 == specification.stage_113_validation_sha256
                && value.stage_112_result_sha256 == specification.stage_112_result_sha256
                && value.stage_112_output_sha256 == specification.stage_112_output_sha256
                && value.stage_111_claim_sha256 == specification.stage_111_claim_sha256
                && value.admitted_available_at_utc == specification.admitted_available_at_utc
        });
    push_check(&mut reasons, exact_chain, "Stage 51–122 binding mismatch");
    let notice_count = persisted.as_ref().map_or(0, |value| value.notices.len());
    let mut event_type_counts = BTreeMap::new();
    if let Some(value) = &persisted {
        for notice in &value.notices {
            *event_type_counts
                .entry(notice.event_type.clone())
                .or_insert(0) += 1;
        }
    }
    IndependentAudit {
        exact_chain,
        claim_fingerprint,
        result_fingerprint,
        candidate_file,
        admitted_observation,
        reprojected,
        notice_hashes,
        exact_decimals,
        exact_candidate,
        opening_boundary,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_persisted_candidate_sha256,
        independently_recomputed_candidate_sha256,
        candidate_bytes,
        notice_count,
        event_type_counts,
        mismatch_reasons: reasons,
    }
}

async fn independent_read_candidate(
    state: &AppState,
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    result: &ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
) -> Result<
    (
        UntrustedNonFinancialObservationNoticeCandidateEnvelope,
        usize,
    ),
    String,
> {
    let candidate_sha256 = result
        .candidate_sha256
        .as_deref()
        .ok_or_else(|| "successful Stage 122 result has no candidate sha256".to_string())?;
    let expected = PathBuf::from("candidates")
        .join(&claim.attempt_id)
        .join(format!("{candidate_sha256}.json"));
    let relative = PathBuf::from(
        result
            .candidate_relative_path
            .as_deref()
            .ok_or_else(|| "successful Stage 122 result has no candidate path".to_string())?,
    );
    if relative != expected || !safe_relative_path(&relative) {
        return Err("candidate path is not the exact content-addressed path".to_string());
    }
    let path = controlled_shadow_observation_ledger_transition_execution_custody_root(state)
        .join(relative);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|_| "candidate file is missing".to_string())?;
    let maximum = claim
        .authorization
        .runner
        .runner_contract
        .maximum_output_bytes;
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err("candidate is not a bounded regular file".to_string());
    }
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|_| "candidate file read failed".to_string())?;
    let value =
        serde_json::from_slice::<UntrustedNonFinancialObservationNoticeCandidateEnvelope>(&bytes)
            .map_err(|_| "candidate JSON/schema rejected".to_string())?;
    Ok((value, bytes.len()))
}

async fn exact_admitted_evidence(
    state: &AppState,
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
) -> Result<AdmittedControlledShadowObservationEvidence, String> {
    let admitted = admitted_controlled_shadow_observation_evidence_for_stage_115(state)
        .await?
        .into_iter()
        .find(|value| {
            value.admission_review.review_sha256 == specification.stage_114_review_sha256
                && value.envelope.output_sha256 == specification.stage_112_output_sha256
        })
        .ok_or_else(|| "exact Stage 114 admitted observation evidence 不再可用".to_string())?;
    if admitted.candidate.validation.validation_sha256 != specification.stage_113_validation_sha256
        || admitted.candidate.result.result_sha256 != specification.stage_112_result_sha256
        || admitted.candidate.claim.claim_sha256 != specification.stage_111_claim_sha256
        || admitted.envelope.admitted_available_at_utc != specification.admitted_available_at_utc
    {
        return Err("Stage 111–114 admitted observation evidence chain 漂移".to_string());
    }
    Ok(admitted)
}

fn independent_project_candidate(
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
    admitted: &AdmittedControlledShadowObservationEvidence,
) -> Result<UntrustedNonFinancialObservationNoticeCandidateEnvelope, String> {
    independent_validate_projection_boundary(specification, admitted)?;
    let envelope = &admitted.envelope;
    let available_at = specification.admitted_available_at_utc;
    let review_sha = &specification.stage_114_review_sha256;
    let output_sha = &specification.stage_112_output_sha256;
    let mut notices = Vec::new();
    notices.push(independent_new_notice(
        "observation_evidence_anchor",
        specification.earliest_market_session_date,
        available_at,
        review_sha,
        output_sha,
        Some(output_sha.clone()),
        None,
        BTreeMap::new(),
        BTreeMap::from([(
            "evidence_kind".to_string(),
            "exact_stage_114_admitted_observation_envelope".to_string(),
        )]),
    )?);
    for session in &envelope.sessions {
        notices.push(independent_new_notice(
            "market_session_fact",
            session.date,
            available_at,
            review_sha,
            output_sha,
            None,
            None,
            BTreeMap::new(),
            BTreeMap::from([
                ("session_kind".to_string(), session.session_kind.clone()),
                ("close_time_et".to_string(), session.close_time_et.clone()),
            ]),
        )?);
    }
    let subjects = specification
        .subject_symbols
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for price in &envelope.prices {
        let event_type = if subjects.contains(price.symbol.as_str())
            && price.price_basis == "raw_unadjusted_price"
        {
            Some("security_raw_close_mark_fact")
        } else if price.symbol == specification.benchmark_symbol
            && price.price_basis == "dividend_adjusted_price"
        {
            Some("benchmark_total_return_mark_fact")
        } else {
            None
        };
        if let Some(event_type) = event_type {
            let mut decimals =
                BTreeMap::from([("close_decimal".to_string(), price.close_decimal.clone())]);
            if let Some(volume) = price.volume_integer {
                decimals.insert("volume_integer".to_string(), volume.to_string());
            }
            notices.push(independent_new_notice(
                event_type,
                price.date,
                available_at,
                review_sha,
                output_sha,
                Some(price.source_row_sha256.clone()),
                Some(price.symbol.clone()),
                decimals,
                BTreeMap::from([("price_basis".to_string(), price.price_basis.clone())]),
            )?);
        }
    }
    for gap in &envelope.explicit_gaps {
        notices.push(independent_new_notice(
            "explicit_data_gap_fact",
            gap.date,
            available_at,
            review_sha,
            output_sha,
            Some(gap.gap_row_sha256.clone()),
            Some(gap.symbol.clone()),
            BTreeMap::new(),
            BTreeMap::from([
                ("price_basis".to_string(), gap.price_basis.clone()),
                ("reason".to_string(), gap.reason.clone()),
                ("nav_status".to_string(), "blocked_no_fill".to_string()),
            ]),
        )?);
    }
    for action in &envelope.dividends {
        notices.push(independent_new_notice(
            "dividend_notice_pending_validation",
            action.date,
            available_at,
            review_sha,
            output_sha,
            Some(action.action_row_sha256.clone()),
            Some(action.symbol.clone()),
            BTreeMap::new(),
            BTreeMap::from([(
                "posting_status".to_string(),
                "notice_only_requires_position_and_terms_validation".to_string(),
            )]),
        )?);
    }
    for action in &envelope.splits {
        notices.push(independent_new_notice(
            "split_notice_pending_validation",
            action.date,
            available_at,
            review_sha,
            output_sha,
            Some(action.action_row_sha256.clone()),
            Some(action.symbol.clone()),
            BTreeMap::new(),
            BTreeMap::from([(
                "posting_status".to_string(),
                "notice_only_requires_position_and_terms_validation".to_string(),
            )]),
        )?);
    }
    notices.sort_by(|left, right| independent_notice_key(left).cmp(&independent_notice_key(right)));
    if notices
        .windows(2)
        .any(|values| values[0].notice_id == values[1].notice_id)
    {
        return Err("independent notice idempotency collision".to_string());
    }
    let mut candidate = UntrustedNonFinancialObservationNoticeCandidateEnvelope {
        schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        specification_sha256: specification.specification_sha256.clone(),
        stage_114_review_sha256: specification.stage_114_review_sha256.clone(),
        stage_113_validation_sha256: specification.stage_113_validation_sha256.clone(),
        stage_112_result_sha256: specification.stage_112_result_sha256.clone(),
        stage_112_output_sha256: specification.stage_112_output_sha256.clone(),
        stage_111_claim_sha256: specification.stage_111_claim_sha256.clone(),
        admitted_available_at_utc: available_at,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist: Vec::new(),
        notices,
        candidate_sha256: String::new(),
        create_once: true,
        untrusted: true,
        independent_validation_completed: false,
        ledger_created: false,
        authoritative_financial_state_created: false,
        nav_or_performance_calculated: false,
        order_intent_created: false,
    };
    candidate.candidate_sha256 =
        independent_fingerprint_without(&candidate, &["candidate_sha256"])?;
    independent_validate_candidate(specification, &candidate)?;
    Ok(candidate)
}

fn independent_validate_projection_boundary(
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
    admitted: &AdmittedControlledShadowObservationEvidence,
) -> Result<(), String> {
    let prerequisite = &specification.opening_portfolio_prerequisite;
    let rules = &specification.mapping_rules;
    let allowed = rules
        .non_financial_event_type_allowlist
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = allowed_event_types();
    if !specification.exact_stage_114_admitted_envelope_is_only_observation_input
        || admitted.admission_review.review_sha256 != specification.stage_114_review_sha256
        || admitted.envelope.output_sha256 != specification.stage_112_output_sha256
        || !prerequisite.separately_admitted_opening_portfolio_snapshot_required
        || prerequisite.current_opening_portfolio_snapshot_available
        || prerequisite.default_notional_allowed
        || prerequisite.default_cash_allowed
        || prerequisite.infer_positions_from_subject_symbols_allowed
        || prerequisite.infer_share_quantities_from_prices_or_target_weights_allowed
        || prerequisite.financial_posting_before_opening_snapshot_admission_allowed
        || !rules
            .financial_event_type_allowlist_before_opening_snapshot
            .is_empty()
        || allowed != expected
        || specification.financial_postings_currently_eligible
        || specification.nav_or_performance_currently_eligible
        || specification
            .authority_boundary
            .opening_portfolio_snapshot_present
        || specification.authority_boundary.ledger_created
        || specification.authority_boundary.ledger_event_write_allowed
        || specification.authority_boundary.position_write_allowed
        || specification.authority_boundary.cash_write_allowed
        || specification
            .authority_boundary
            .nav_or_performance_write_allowed
        || specification.authority_boundary.order_generation_allowed
        || specification.authority_boundary.broker_access_allowed
        || specification.authority_boundary.trading_allowed
    {
        return Err("independent opening-portfolio or non-financial boundary mismatch".to_string());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn independent_new_notice(
    event_type: &str,
    effective_date: NaiveDate,
    available_at_utc: DateTime<Utc>,
    stage_114_review_sha256: &str,
    stage_112_output_sha256: &str,
    source_row_sha256: Option<String>,
    symbol: Option<String>,
    exact_decimal_fields: BTreeMap<String, String>,
    evidence_attributes: BTreeMap<String, String>,
) -> Result<CanonicalNonFinancialObservationNoticeCandidate, String> {
    let identity = serde_json::json!({
        "stage_114_review_sha256": stage_114_review_sha256,
        "stage_112_output_sha256": stage_112_output_sha256,
        "event_type": event_type,
        "source_row_sha256_or_envelope_sha256": source_row_sha256.as_deref().unwrap_or(stage_112_output_sha256),
        "symbol_or_none": symbol.as_deref().unwrap_or("none"),
        "effective_date": effective_date,
    });
    let notice_id = hash_bytes(&serde_json::to_vec(&identity).map_err(|error| error.to_string())?);
    let mut notice = CanonicalNonFinancialObservationNoticeCandidate {
        notice_id,
        notice_sha256: String::new(),
        event_type: event_type.to_string(),
        effective_date,
        available_at_utc,
        stage_114_review_sha256: stage_114_review_sha256.to_string(),
        stage_112_output_sha256: stage_112_output_sha256.to_string(),
        source_row_sha256,
        symbol,
        exact_decimal_fields,
        evidence_attributes,
        non_financial: true,
        untrusted: true,
        authoritative: false,
        financial_posting_created: false,
        ledger_event_written: false,
        supersedes_notice_id: None,
    };
    notice.notice_sha256 = independent_fingerprint_without(&notice, &["notice_sha256"])?;
    independent_validate_notice(&notice)?;
    Ok(notice)
}

fn independent_notice_key(
    notice: &CanonicalNonFinancialObservationNoticeCandidate,
) -> (NaiveDate, u8, String, String) {
    let precedence = match notice.event_type.as_str() {
        "observation_evidence_anchor" => 0,
        "market_session_fact" => 1,
        "dividend_notice_pending_validation" | "split_notice_pending_validation" => 2,
        "explicit_data_gap_fact" => 3,
        "security_raw_close_mark_fact" | "benchmark_total_return_mark_fact" => 4,
        _ => u8::MAX,
    };
    (
        notice.effective_date,
        precedence,
        notice.symbol.clone().unwrap_or_default(),
        notice.notice_id.clone(),
    )
}

fn independent_validate_notice(
    notice: &CanonicalNonFinancialObservationNoticeCandidate,
) -> Result<(), String> {
    let identity = serde_json::json!({
        "stage_114_review_sha256": notice.stage_114_review_sha256,
        "stage_112_output_sha256": notice.stage_112_output_sha256,
        "event_type": notice.event_type,
        "source_row_sha256_or_envelope_sha256": notice.source_row_sha256.as_deref().unwrap_or(&notice.stage_112_output_sha256),
        "symbol_or_none": notice.symbol.as_deref().unwrap_or("none"),
        "effective_date": notice.effective_date,
    });
    let expected_id =
        hash_bytes(&serde_json::to_vec(&identity).map_err(|error| error.to_string())?);
    if notice.notice_id != expected_id
        || !valid_sha256(&notice.notice_sha256)
        || notice.notice_sha256 != independent_fingerprint_without(notice, &["notice_sha256"])?
        || !allowed_event_types().contains(notice.event_type.as_str())
        || !valid_sha256(&notice.stage_114_review_sha256)
        || !valid_sha256(&notice.stage_112_output_sha256)
        || notice
            .source_row_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256(value))
        || notice
            .exact_decimal_fields
            .values()
            .any(|value| !valid_exact_decimal(value))
        || !notice.non_financial
        || !notice.untrusted
        || notice.authoritative
        || notice.financial_posting_created
        || notice.ledger_event_written
        || notice.supersedes_notice_id.is_some()
    {
        return Err("independent non-financial notice validation failed".to_string());
    }
    Ok(())
}

fn independent_validate_candidate(
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
    candidate: &UntrustedNonFinancialObservationNoticeCandidateEnvelope,
) -> Result<(), String> {
    let sorted = candidate
        .notices
        .windows(2)
        .all(|values| independent_notice_key(&values[0]) < independent_notice_key(&values[1]));
    for notice in &candidate.notices {
        independent_validate_notice(notice)?;
    }
    if candidate.schema_version != OUTPUT_SCHEMA_VERSION
        || candidate.specification_sha256 != specification.specification_sha256
        || candidate.stage_114_review_sha256 != specification.stage_114_review_sha256
        || candidate.stage_113_validation_sha256 != specification.stage_113_validation_sha256
        || candidate.stage_112_result_sha256 != specification.stage_112_result_sha256
        || candidate.stage_112_output_sha256 != specification.stage_112_output_sha256
        || candidate.stage_111_claim_sha256 != specification.stage_111_claim_sha256
        || candidate.admitted_available_at_utc != specification.admitted_available_at_utc
        || candidate.notices.is_empty()
        || !sorted
        || candidate.candidate_sha256
            != independent_fingerprint_without(candidate, &["candidate_sha256"])?
        || candidate.opening_portfolio_snapshot_admitted
        || !candidate.financial_event_allowlist.is_empty()
        || !candidate.create_once
        || !candidate.untrusted
        || candidate.independent_validation_completed
        || candidate.ledger_created
        || candidate.authoritative_financial_state_created
        || candidate.nav_or_performance_calculated
        || candidate.order_intent_created
    {
        return Err(
            "independent candidate is invalid, non-canonical or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn independent_notice_hashes_and_identities_valid(
    candidate: &UntrustedNonFinancialObservationNoticeCandidateEnvelope,
) -> bool {
    candidate
        .notices
        .iter()
        .all(|notice| independent_validate_notice(notice).is_ok())
        && candidate
            .notices
            .windows(2)
            .all(|values| independent_notice_key(&values[0]) < independent_notice_key(&values[1]))
}

fn exact_specification(
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
) -> &ControlledShadowObservationLedgerTransitionSpecification {
    &claim
        .authorization
        .runner
        .runner_contract
        .exact_approved_implementation_contract
        .exact_observation_ledger_transition_specification
}

fn validate_request_shape(
    attempt_id: &str,
    request: &ValidateControlledShadowObservationLedgerTransitionOutputRequest,
) -> Result<(), String> {
    let valid = valid_id(attempt_id)
        && valid_sha256(&request.expected_claim_sha256)
        && valid_sha256(&request.expected_result_sha256)
        && valid_sha256(&request.expected_candidate_sha256)
        && valid_sha256(&request.expected_specification_sha256)
        && valid_sha256(&request.expected_stage_114_review_sha256)
        && valid_sha256(&request.expected_stage_112_output_sha256)
        && !request.validation_reason.trim().is_empty()
        && request.validation_reason.chars().count() <= MAX_REASON_CHARS
        && request.exact_current_stage_51_through_stage_122_binding_confirmed
        && request
            .validator_independent_from_executor_claimant_and_complete_prior_chain_confirmed
        && request.stage_122_result_and_create_once_candidate_reopened_and_rehashed_confirmed
        && request
            .exact_stage_114_admitted_observation_envelope_reopened_and_rehashed_confirmed
        && request.second_projection_does_not_call_stage_122_projector_helpers_confirmed
        && request
            .every_notice_identity_decimal_hash_sort_and_complete_candidate_exactly_compared_confirmed
        && request.opening_portfolio_snapshot_absent_and_financial_event_allowlist_empty_confirmed
        && request
            .pass_only_opens_future_stage_124_non_financial_candidate_admission_review_confirmed
        && request
            .no_ledger_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 123 请求摘要、理由或确认项不完整".to_string())
}

fn verify_expected_bindings(
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    result: &ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
    request: &ValidateControlledShadowObservationLedgerTransitionOutputRequest,
) -> Result<(), String> {
    let specification = exact_specification(claim);
    if request.expected_claim_sha256 != claim.claim_sha256
        || request.expected_result_sha256 != result.result_sha256
        || result.candidate_sha256.as_deref() != Some(request.expected_candidate_sha256.as_str())
        || request.expected_specification_sha256 != specification.specification_sha256
        || request.expected_stage_114_review_sha256 != specification.stage_114_review_sha256
        || request.expected_stage_112_output_sha256 != specification.stage_112_output_sha256
        || result.stage_121_claim_sha256 != claim.claim_sha256
    {
        return Err("Stage 121–123 责任链已变化或请求绑定错误".to_string());
    }
    Ok(())
}

fn prior_actor_ids(
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    result: &ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
) -> Vec<String> {
    let mut values = claim.excluded_prior_actor_ids.clone();
    values.push(claim.claimed_by.clone());
    values.push(result.executed_by.clone());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && !excluded.iter().any(|value| value == actor_id)
}

fn validate_record(
    record: &ControlledShadowObservationLedgerTransitionOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.verdict
        == ControlledShadowObservationLedgerTransitionOutputValidationVerdict::IndependentlyValidatedExactNonFinancialNoticeCandidate;
    let all_checks = record.validator_independent_from_executor_claimant_and_complete_prior_chain
        && record.exact_current_stage_51_through_stage_122_chain_verified
        && record.claim_fingerprint_independently_verified
        && record.result_fingerprint_independently_verified
        && record.candidate_file_custody_and_fingerprint_verified
        && record.exact_stage_114_admitted_observation_revalidated
        && record.complete_candidate_independently_reprojected
        && record.every_notice_identity_and_hash_independently_verified
        && record.exact_decimal_fields_independently_verified
        && record.canonical_sort_and_complete_candidate_exact_match_verified
        && record.opening_portfolio_absence_and_empty_financial_allowlist_verified
        && record.no_downstream_authority_verified
        && record.mismatch_reasons.is_empty();
    let authority_closed = record.candidate_remains_untrusted
        && !record.opening_portfolio_snapshot_admitted
        && !record.authoritative_ledger_event_created
        && !record.position_written
        && !record.cash_written
        && !record.nav_or_performance_written
        && !record.model_or_metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256)
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != record_fingerprint(record)?
        || !valid_id(&record.stage_121_attempt_id)
        || !valid_sha256(&record.stage_121_claim_sha256)
        || !valid_id(&record.stage_122_result_id)
        || !valid_sha256(&record.stage_122_result_sha256)
        || !valid_sha256(&record.stage_122_candidate_sha256)
        || !valid_sha256(&record.observation_ledger_transition_specification_sha256)
        || !valid_sha256(&record.stage_114_review_sha256)
        || !valid_sha256(&record.stage_112_output_sha256)
        || record.validated_by.trim().is_empty()
        || record.validation_reason.trim().is_empty()
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || !authority_closed
        || passed != all_checks
        || record.non_financial_notice_candidate_independently_validated != passed
        || record.future_stage_124_non_financial_candidate_admission_review_eligible != passed
    {
        return Err("Stage 123 validation record 无效、漂移或越权".to_string());
    }
    Ok(())
}

fn validate_record_binding(
    record: &ControlledShadowObservationLedgerTransitionOutputValidationRecord,
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    result: &ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
) -> Result<(), String> {
    validate_record(record)?;
    let specification = exact_specification(claim);
    if record.stage_121_attempt_id != claim.attempt_id
        || record.stage_121_claim_sha256 != claim.claim_sha256
        || record.stage_122_result_id != result.result_id
        || record.stage_122_result_sha256 != result.result_sha256
        || result.candidate_sha256.as_deref() != Some(record.stage_122_candidate_sha256.as_str())
        || record.observation_ledger_transition_specification_sha256
            != specification.specification_sha256
        || record.stage_114_review_sha256 != specification.stage_114_review_sha256
        || record.stage_112_output_sha256 != specification.stage_112_output_sha256
    {
        return Err("Stage 123 validation 与当前 Stage 121/122 绑定不一致".to_string());
    }
    Ok(())
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationLedgerTransitionOutputValidationRecord>, String> {
    let mut directories = match tokio::fs::read_dir(validation_root(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut values = Vec::new();
    while let Some(directory) = directories
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if !directory
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let mut entries = tokio::fs::read_dir(directory.path())
            .await
            .map_err(|error| error.to_string())?;
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
                || metadata.len() > 1024 * 1024
            {
                return Err("Stage 123 validation 文件无效或超限".to_string());
            }
            let value = serde_json::from_slice::<
                ControlledShadowObservationLedgerTransitionOutputValidationRecord,
            >(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            validate_record(&value)?;
            values.push(value);
        }
    }
    let mut attempts = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for value in &values {
        if !attempts.insert(value.stage_121_attempt_id.clone())
            || !ids.insert(value.validation_id.clone())
        {
            return Err("Stage 123 validation 重复或分叉".to_string());
        }
    }
    values.sort_by(|left, right| left.validated_at.cmp(&right.validated_at));
    Ok(values)
}

fn validator_implementation_sha256() -> String {
    hash_bytes(
        b"stage123|independent-second-projection|claim-result-candidate-rehash|stage114-reopen|notice-id-hash-decimal-sort|complete-candidate-exact-match|opening-absent|no-financial-authority",
    )
}

fn record_fingerprint(
    record: &ControlledShadowObservationLedgerTransitionOutputValidationRecord,
) -> Result<String, String> {
    independent_fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn independent_fingerprint_without<T: Serialize>(
    value: &T,
    fields: &[&str],
) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 123 fingerprint payload 无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(hash_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

fn allowed_event_types() -> BTreeSet<&'static str> {
    BTreeSet::from([
        "observation_evidence_anchor",
        "market_session_fact",
        "security_raw_close_mark_fact",
        "benchmark_total_return_mark_fact",
        "explicit_data_gap_fact",
        "dividend_notice_pending_validation",
        "split_notice_pending_validation",
    ])
}

fn valid_exact_decimal(value: &str) -> bool {
    if value.is_empty() || value.starts_with('+') || value.contains(['e', 'E']) {
        return false;
    }
    let value = value.strip_prefix('-').unwrap_or(value);
    let mut parts = value.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    !integer.is_empty()
        && integer.bytes().all(|byte| byte.is_ascii_digit())
        && fraction
            .is_none_or(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
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
    if value.is_empty() || value.chars().count() > MAX_REASON_CHARS {
        Err(format!("{label}为空或超过 {MAX_REASON_CHARS} 字符"))
    } else {
        Ok(value.to_string())
    }
}

fn push_reason(reasons: &mut Vec<String>, reason: impl Into<String>) {
    if reasons.len() < MAX_MISMATCH_REASONS {
        reasons.push(reason.into());
    }
}

fn push_check(reasons: &mut Vec<String>, passed: bool, reason: &str) {
    if !passed {
        push_reason(reasons, reason.to_string());
    }
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() > 1024 * 1024 {
        return Err("Stage 123 validation 文件为空或超限".to_string());
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

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-observation-ledger-transition-output-validations")
}

struct ValidationLock(PathBuf);

impl Drop for ValidationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<ValidationLock, String> {
    let directory = validation_root(state).join("locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{attempt_id}.lock"));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "这条 Stage 122 candidate 正在校验或已被并发锁定".to_string())?;
    Ok(ValidationLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(value: &str) -> String {
        hash_bytes(value.as_bytes())
    }

    #[test]
    fn independent_notice_identity_is_deterministic_and_untrusted() {
        let notice = independent_new_notice(
            "security_raw_close_mark_fact",
            NaiveDate::from_ymd_opt(2099, 1, 2).unwrap(),
            DateTime::parse_from_rfc3339("2099-01-03T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            &hash("review"),
            &hash("output"),
            Some(hash("row")),
            Some("ZZTEST".to_string()),
            BTreeMap::from([("close_decimal".to_string(), "123.45".to_string())]),
            BTreeMap::from([(
                "price_basis".to_string(),
                "raw_unadjusted_price".to_string(),
            )]),
        )
        .unwrap();
        assert!(notice.non_financial && notice.untrusted);
        assert!(!notice.authoritative && !notice.ledger_event_written);
        independent_validate_notice(&notice).unwrap();
    }

    #[test]
    fn independent_decimal_validation_rejects_ambiguous_forms() {
        assert!(valid_exact_decimal("0"));
        assert!(valid_exact_decimal("-12.340"));
        assert!(!valid_exact_decimal("1e3"));
        assert!(!valid_exact_decimal(".5"));
        assert!(!valid_exact_decimal("1."));
        assert!(!valid_exact_decimal("+1"));
    }

    #[test]
    fn independent_canonical_order_matches_frozen_precedence() {
        let date = NaiveDate::from_ymd_opt(2099, 1, 2).unwrap();
        let available = DateTime::parse_from_rfc3339("2099-01-03T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let make = |event: &str| {
            independent_new_notice(
                event,
                date,
                available,
                &hash("review"),
                &hash("output"),
                None,
                None,
                BTreeMap::new(),
                BTreeMap::new(),
            )
            .unwrap()
        };
        assert!(
            independent_notice_key(&make("market_session_fact"))
                < independent_notice_key(&make("dividend_notice_pending_validation"))
        );
        assert!(
            independent_notice_key(&make("explicit_data_gap_fact"))
                < independent_notice_key(&make("security_raw_close_mark_fact"))
        );
    }

    #[test]
    fn next_gate_is_admission_review_not_financial_ledger() {
        assert_eq!(
            NEXT_GATE,
            "stage_124_validated_non_financial_observation_notice_candidate_admission_review"
        );
        assert!(POLICY_VERSION.contains("chain-external"));
        assert!(VALIDATOR_IMPLEMENTATION_VERSION.contains("second-projection"));
    }
}
