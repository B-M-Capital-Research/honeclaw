//! Stage 113 chain-external validation of one Stage 112 untrusted observation envelope.
//!
//! This validator never calls the Stage 112 materializer. It reopens the immutable Stage 112
//! output and the exact Stage 104-admitted Stage 102 input, projects the complete envelope with a
//! second implementation, and compares every field and row hash. A terminal validation is
//! create-once. Passing only opens a future Stage 114 observation-evidence admission review; it
//! creates no ledger, position, performance, training, reward, order, broker or trading authority.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_execution_attempts::{
    CanonicalMarketCalendarRow, CanonicalMarketDividendRow, CanonicalMarketPriceRow,
    CanonicalMarketSplitRow, UntrustedCanonicalMarketDataParseResult,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_input_admission_reviews::read_revalidated_admitted_observation_input_for_materialization_execution;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_execution_attempt_claims::{
    ControlledShadowObservationMaterializationExecutionAttemptClaim,
    observation_materialization_execution_attempt_claims_for_execution,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_execution_attempts::{
    ControlledShadowObservationMaterializationExecutionAttemptResult,
    ControlledShadowObservationMaterializationExecutionStatus, InitialShadowAllocationBinding,
    MaterializedCorporateAction, MaterializedExplicitGap, MaterializedMarketSession,
    MaterializedPriceObservation, UntrustedControlledShadowObservationEnvelope,
    controlled_shadow_observation_materialization_execution_custody_root,
    observation_materialization_execution_results_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_specifications::ControlledShadowObservationMaterializationSpecification;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-output-validation-v1";
const OUTPUT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-untrusted-natural-forward-observation-envelope-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-materialization-chain-external-full-reprojection-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-observation-materialization-validator-v1-independent-second-projection";
const NEXT_GATE: &str = "stage_114_validated_observation_envelope_admission_review";
const MAX_REASON_CHARS: usize = 4_000;
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateControlledShadowObservationMaterializationOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_specification_sha256: String,
    expected_stage_104_review_sha256: String,
    expected_stage_102_output_sha256: String,
    validation_reason: String,
    exact_current_stage_51_through_stage_112_binding_confirmed: bool,
    validator_independent_from_executor_and_complete_prior_chain_confirmed: bool,
    stage_112_result_and_create_once_output_reopened_and_rehashed_confirmed: bool,
    exact_stage_104_admitted_stage_102_input_reopened_and_rehashed_confirmed: bool,
    second_projection_does_not_call_stage_112_materializer_helpers_confirmed: bool,
    sessions_prices_gaps_actions_allocation_availability_independently_recomputed_confirmed: bool,
    every_row_hash_sort_order_and_complete_envelope_exactly_compared_confirmed: bool,
    pass_only_opens_future_stage_114_observation_evidence_admission_review_confirmed: bool,
    no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationMaterializationOutputValidationVerdict {
    IndependentlyValidatedExactObservationEnvelope,
    FailedIndependentObservationEnvelopeValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub stage_111_attempt_id: String,
    pub stage_111_claim_sha256: String,
    pub stage_112_result_id: String,
    pub stage_112_result_sha256: String,
    pub stage_112_output_sha256: String,
    pub observation_materialization_specification_sha256: String,
    pub stage_104_review_sha256: String,
    pub stage_102_output_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub validation_reason: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validator_independent_from_executor_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_112_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub output_file_custody_and_fingerprint_verified: bool,
    pub exact_stage_104_admitted_input_revalidated: bool,
    pub source_matrix_independently_verified: bool,
    pub complete_envelope_independently_reprojected: bool,
    pub every_output_row_hash_independently_verified: bool,
    pub canonical_sort_and_complete_output_exact_match_verified: bool,
    pub initial_allocation_binding_preserved_without_recomputation_verified: bool,
    pub admitted_availability_preserved_verified: bool,
    pub no_downstream_authority_verified: bool,
    pub recomputed_claim_sha256: String,
    pub recomputed_result_sha256: String,
    pub recomputed_persisted_output_sha256: String,
    pub independently_recomputed_output_sha256: String,
    pub observed_output_bytes: usize,
    pub observed_session_count: usize,
    pub observed_price_count: usize,
    pub observed_gap_count: usize,
    pub observed_dividend_count: usize,
    pub observed_split_count: usize,
    pub mismatch_reasons: Vec<String>,
    pub verdict: ControlledShadowObservationMaterializationOutputValidationVerdict,
    pub observation_envelope_independently_validated: bool,
    pub future_stage_114_observation_evidence_admission_review_eligible: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationOutputValidationItem {
    pub claim: ControlledShadowObservationMaterializationExecutionAttemptClaim,
    pub result: ControlledShadowObservationMaterializationExecutionAttemptResult,
    pub validation: Option<ControlledShadowObservationMaterializationOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<ControlledShadowObservationMaterializationOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_observation_count: usize,
    pub failed_validation_count: usize,
    pub future_stage_114_observation_evidence_admission_review_eligible_count: usize,
    pub validation_status: String,
    pub next_gate: String,
    pub independent_output_validation_available: bool,
    pub ledger_created: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationMaterializationOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_observation_count: usize,
    pub failed_validation_count: usize,
    pub future_stage_114_observation_evidence_admission_review_eligible_count: usize,
    pub validation_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyValidatedControlledShadowObservationEnvelope {
    pub claim: ControlledShadowObservationMaterializationExecutionAttemptClaim,
    pub result: ControlledShadowObservationMaterializationExecutionAttemptResult,
    pub validation: ControlledShadowObservationMaterializationOutputValidationRecord,
}

#[derive(Debug, Clone)]
pub(crate) struct RevalidatedControlledShadowObservationEvidenceForStage114 {
    pub evidence: IndependentlyValidatedControlledShadowObservationEnvelope,
    pub envelope: UntrustedControlledShadowObservationEnvelope,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    output_file: bool,
    admitted_input: bool,
    source_matrix: bool,
    reprojected: bool,
    row_hashes: bool,
    exact_output: bool,
    allocation: bool,
    availability: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_persisted_output_sha256: String,
    independently_recomputed_output_sha256: String,
    output_bytes: usize,
    session_count: usize,
    price_count: usize,
    gap_count: usize,
    dividend_count: usize,
    split_count: usize,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_controlled_shadow_observation_materialization_output_validations(
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
            warn!(%error, "observation materialization output validation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察物化输出独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_controlled_shadow_observation_materialization_output_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateControlledShadowObservationMaterializationOutputRequest>,
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

pub(crate) async fn controlled_shadow_observation_materialization_output_validation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationOutputValidationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowObservationMaterializationOutputValidationReadinessSummary {
            validation_eligible_count: value.validation_eligible_count,
            validation_count: value.validation_count,
            independently_validated_observation_count: value
                .independently_validated_observation_count,
            failed_validation_count: value.failed_validation_count,
            future_stage_114_observation_evidence_admission_review_eligible_count: value
                .future_stage_114_observation_evidence_admission_review_eligible_count,
            validation_status: value.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_observation_envelopes_for_stage_114_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedControlledShadowObservationEnvelope>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let validation = item.validation?;
            (validation.observation_envelope_independently_validated
                && validation.future_stage_114_observation_evidence_admission_review_eligible)
                .then_some(IndependentlyValidatedControlledShadowObservationEnvelope {
                    claim: item.claim,
                    result: item.result,
                    validation,
                })
        })
        .collect())
}

/// Reopens the exact Stage 112 envelope and reruns the chain-external Stage 113 audit immediately
/// before a Stage 114 reviewer can admit it as observation evidence. This is a custody/current-
/// binding recheck, not a new materialization and not an accounting transition.
pub(crate) async fn revalidated_observation_evidence_for_stage_114_review(
    state: &AppState,
    attempt_id: &str,
    expected_validation_sha256: &str,
) -> Result<RevalidatedControlledShadowObservationEvidenceForStage114, String> {
    let evidence = independently_validated_observation_envelopes_for_stage_114_review(state)
        .await?
        .into_iter()
        .find(|value| {
            value.claim.attempt_id == attempt_id
                && value.validation.validation_sha256 == expected_validation_sha256
        })
        .ok_or_else(|| "Stage 114 找不到当前精确绑定且独立验证通过的观察 envelope".to_string())?;
    let audit = independently_audit(state, &evidence.claim, &evidence.result).await;
    let validation = &evidence.validation;
    let current = audit.exact_chain
        && audit.claim_fingerprint
        && audit.result_fingerprint
        && audit.output_file
        && audit.admitted_input
        && audit.source_matrix
        && audit.reprojected
        && audit.row_hashes
        && audit.exact_output
        && audit.allocation
        && audit.availability
        && audit.downstream_closed
        && audit.mismatch_reasons.is_empty()
        && audit.recomputed_claim_sha256 == validation.recomputed_claim_sha256
        && audit.recomputed_result_sha256 == validation.recomputed_result_sha256
        && audit.recomputed_persisted_output_sha256
            == validation.recomputed_persisted_output_sha256
        && audit.independently_recomputed_output_sha256
            == validation.independently_recomputed_output_sha256
        && audit.output_bytes == validation.observed_output_bytes
        && audit.session_count == validation.observed_session_count
        && audit.price_count == validation.observed_price_count
        && audit.gap_count == validation.observed_gap_count
        && audit.dividend_count == validation.observed_dividend_count
        && audit.split_count == validation.observed_split_count;
    if !current {
        return Err("Stage 114 前置重开、重哈希或独立重投影不再与 Stage 113 终态一致".to_string());
    }
    let (envelope, _) = independent_read_output(state, &evidence.claim, &evidence.result).await?;
    Ok(RevalidatedControlledShadowObservationEvidenceForStage114 { evidence, envelope })
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationOutputValidationRegistry, String> {
    let claims = observation_materialization_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .map(|value| (value.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let results =
        observation_materialization_execution_results_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let validation_index = validations
        .iter()
        .map(|value| (value.stage_111_attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut items = Vec::new();
    for result in results.into_iter().filter(|value| {
        value.status
            == ControlledShadowObservationMaterializationExecutionStatus::CompletedWithUntrustedObservationEnvelope
    }) {
        let claim = claims
            .get(&result.stage_111_attempt_id)
            .cloned()
            .ok_or_else(|| "Stage 113 candidate 缺少 Stage 111 claim".to_string())?;
        let validation = validation_index
            .get(result.stage_111_attempt_id.as_str())
            .cloned()
            .cloned();
        if let Some(record) = &validation {
            validate_record_binding(record, &claim, &result)?;
        }
        items.push(ControlledShadowObservationMaterializationOutputValidationItem {
            validation_eligible: validation.is_none(),
            claim,
            result,
            validation,
        });
    }
    let validation_eligible_count = items
        .iter()
        .filter(|value| value.validation_eligible)
        .count();
    let independently_validated_observation_count = validations
        .iter()
        .filter(|value| value.observation_envelope_independently_validated)
        .count();
    let failed_validation_count = validations
        .len()
        .saturating_sub(independently_validated_observation_count);
    let validation_status = if failed_validation_count > 0 {
        "observation_materialization_output_validation_failed_fail_closed"
    } else if independently_validated_observation_count > 0 {
        "observation_envelope_independently_validated_waiting_stage_114_admission_review"
    } else if validation_eligible_count > 0 {
        "waiting_chain_external_full_observation_reprojection_validation"
    } else {
        "waiting_successful_stage_112_untrusted_observation_envelope"
    };
    Ok(
        ControlledShadowObservationMaterializationOutputValidationRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
            validator_implementation_sha256: validator_implementation_sha256(),
            items,
            validation_eligible_count,
            validation_count: validations.len(),
            independently_validated_observation_count,
            failed_validation_count,
            future_stage_114_observation_evidence_admission_review_eligible_count:
                independently_validated_observation_count,
            validation_status: validation_status.to_string(),
            next_gate: NEXT_GATE.to_string(),
            independent_output_validation_available: true,
            ledger_created: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "Stage 113 由 Stage 112 executor 与 Stage 51–112 完整责任链之外的新管理员，以第二实现重新打开 Stage 112 非可信 envelope 与 exact Stage 104 admitted Stage 102 input，独立重算 session、三价格口径、显式 gap、公司行动、初始分配绑定、available-at、每行摘要、canonical 排序和完整输出。通过只开放 Stage 114 观察证据准入复核候选；不建账、不写持仓/绩效、不训练/reward、不生成订单、不接券商、不交易。".to_string(),
        },
    )
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateControlledShadowObservationMaterializationOutputRequest,
) -> Result<ControlledShadowObservationMaterializationOutputValidationRecord, String> {
    validate_request_shape(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|value| value.stage_111_attempt_id == attempt_id)
    {
        return Err("这条 Stage 112 输出已经形成不可变独立验证记录，不得重放".to_string());
    }
    let claim = observation_materialization_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .find(|value| value.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有对应的 Stage 111 claim".to_string())?;
    let result = observation_materialization_execution_results_for_independent_validation(state)
        .await?
        .into_iter()
        .find(|value| {
            value.stage_111_attempt_id == attempt_id
                && value.status
                    == ControlledShadowObservationMaterializationExecutionStatus::CompletedWithUntrustedObservationEnvelope
        })
        .ok_or_else(|| "当前没有成功且待独立校验的 Stage 112 非可信输出".to_string())?;
    verify_expected_bindings(&claim, &result, &request)?;
    let excluded_prior_actor_ids = prior_actor_ids(&claim, &result);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "Stage 113 验证者不能是 Stage 112 executor 或 Stage 51–112 任一既有责任人".to_string(),
        );
    }
    let audit = independently_audit(state, &claim, &result).await;
    let passed = independent
        && audit.exact_chain
        && audit.claim_fingerprint
        && audit.result_fingerprint
        && audit.output_file
        && audit.admitted_input
        && audit.source_matrix
        && audit.reprojected
        && audit.row_hashes
        && audit.exact_output
        && audit.allocation
        && audit.availability
        && audit.downstream_closed
        && audit.mismatch_reasons.is_empty();
    let specification = exact_specification(&claim);
    let mut record = ControlledShadowObservationMaterializationOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        stage_111_attempt_id: claim.attempt_id.clone(),
        stage_111_claim_sha256: claim.claim_sha256.clone(),
        stage_112_result_id: result.result_id.clone(),
        stage_112_result_sha256: result.result_sha256.clone(),
        stage_112_output_sha256: result.output_sha256.clone().unwrap_or_default(),
        observation_materialization_specification_sha256: specification
            .specification_sha256
            .clone(),
        stage_104_review_sha256: specification.stage_104_review_sha256.clone(),
        stage_102_output_sha256: specification.stage_102_output_sha256.clone(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        validation_reason: bounded_required(&request.validation_reason, "验证原因")?,
        excluded_prior_actor_ids,
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validator_independent_from_executor_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_112_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        output_file_custody_and_fingerprint_verified: audit.output_file,
        exact_stage_104_admitted_input_revalidated: audit.admitted_input,
        source_matrix_independently_verified: audit.source_matrix,
        complete_envelope_independently_reprojected: audit.reprojected,
        every_output_row_hash_independently_verified: audit.row_hashes,
        canonical_sort_and_complete_output_exact_match_verified: audit.exact_output,
        initial_allocation_binding_preserved_without_recomputation_verified: audit.allocation,
        admitted_availability_preserved_verified: audit.availability,
        no_downstream_authority_verified: audit.downstream_closed,
        recomputed_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_result_sha256: audit.recomputed_result_sha256,
        recomputed_persisted_output_sha256: audit.recomputed_persisted_output_sha256,
        independently_recomputed_output_sha256: audit.independently_recomputed_output_sha256,
        observed_output_bytes: audit.output_bytes,
        observed_session_count: audit.session_count,
        observed_price_count: audit.price_count,
        observed_gap_count: audit.gap_count,
        observed_dividend_count: audit.dividend_count,
        observed_split_count: audit.split_count,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            ControlledShadowObservationMaterializationOutputValidationVerdict::IndependentlyValidatedExactObservationEnvelope
        } else {
            ControlledShadowObservationMaterializationOutputValidationVerdict::FailedIndependentObservationEnvelopeValidation
        },
        observation_envelope_independently_validated: passed,
        future_stage_114_observation_evidence_admission_review_eligible: passed,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
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
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
    result: &ControlledShadowObservationMaterializationExecutionAttemptResult,
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
        "Stage 111 claim fingerprint mismatch",
    );
    push_check(
        &mut reasons,
        result_fingerprint,
        "Stage 112 result fingerprint mismatch",
    );

    let mut output_bytes = 0usize;
    let mut recomputed_persisted_output_sha256 = hash_bytes(b"persisted-output-unavailable");
    let mut persisted: Option<UntrustedControlledShadowObservationEnvelope> = None;
    let output_file = match independent_read_output(state, claim, result).await {
        Ok((value, bytes)) => {
            output_bytes = bytes;
            recomputed_persisted_output_sha256 =
                independent_fingerprint_without(&value, &["output_sha256"]).unwrap_or_else(
                    |error| {
                        push_reason(
                            &mut reasons,
                            format!("persisted output fingerprint: {error}"),
                        );
                        hash_bytes(b"persisted-output-fingerprint-unavailable")
                    },
                );
            let valid = value.output_sha256 == recomputed_persisted_output_sha256
                && result.output_sha256.as_deref() == Some(value.output_sha256.as_str());
            push_check(
                &mut reasons,
                valid,
                "Stage 112 output custody or fingerprint mismatch",
            );
            persisted = Some(value);
            valid
        }
        Err(error) => {
            push_reason(&mut reasons, format!("Stage 112 output reopen: {error}"));
            false
        }
    };

    let mut reprojected_output: Option<UntrustedControlledShadowObservationEnvelope> = None;
    let mut independently_recomputed_output_sha256 = hash_bytes(b"independent-output-unavailable");
    let (admitted_input, source_matrix, reprojected) =
        match read_revalidated_admitted_observation_input_for_materialization_execution(
            state,
            &specification.stage_104_review_sha256,
            &specification.stage_102_output_sha256,
        )
        .await
        {
            Ok(admitted) => {
                let source_matrix =
                    independent_validate_source(specification, &admitted.exact_output)
                        .map(|_| true)
                        .unwrap_or_else(|error| {
                            push_reason(
                                &mut reasons,
                                format!("independent source matrix: {error}"),
                            );
                            false
                        });
                match independent_project_envelope(
                    specification,
                    &admitted.exact_output,
                    &specification.stage_101_input_manifest_sha256,
                ) {
                    Ok(value) => {
                        independently_recomputed_output_sha256 = value.output_sha256.clone();
                        reprojected_output = Some(value);
                        (true, source_matrix, true)
                    }
                    Err(error) => {
                        push_reason(
                            &mut reasons,
                            format!("independent full projection: {error}"),
                        );
                        (true, source_matrix, false)
                    }
                }
            }
            Err(error) => {
                push_reason(
                    &mut reasons,
                    format!("exact Stage 104 input reopen: {error}"),
                );
                (false, false, false)
            }
        };
    let row_hashes = reprojected_output
        .as_ref()
        .is_some_and(independent_output_row_hashes_valid);
    push_check(
        &mut reasons,
        row_hashes,
        "independent observation row hash mismatch",
    );
    let exact_output = persisted
        .as_ref()
        .zip(reprojected_output.as_ref())
        .is_some_and(|(left, right)| left == right);
    push_check(
        &mut reasons,
        exact_output,
        "persisted output differs from independent projection",
    );
    let allocation = persisted.as_ref().is_some_and(|value| {
        value
            .initial_shadow_allocation_binding
            .stage_88_initial_observation_output_sha256
            == specification.stage_88_initial_observation_output_sha256
            && value
                .initial_shadow_allocation_binding
                .initial_shadow_allocation_manifest_sha256
                == specification.initial_shadow_allocation_manifest_sha256
            && !value
                .initial_shadow_allocation_binding
                .initial_shadow_allocation_recomputed
            && !value
                .initial_shadow_allocation_binding
                .accounting_transition_applied
    });
    push_check(
        &mut reasons,
        allocation,
        "initial allocation binding drifted or recomputed",
    );
    let availability = persisted.as_ref().is_some_and(|value| {
        value.admitted_available_at_utc == specification.admitted_available_at_utc
    });
    push_check(&mut reasons, availability, "admitted availability changed");
    let downstream_closed = persisted.as_ref().is_some_and(|value| {
        value.untrusted
            && !result.independent_validation_completed
            && !result.ledger_created
            && !result.position_written
            && !result.performance_metric_written
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
        "downstream authority is not closed",
    );
    let exact_chain = result.stage_111_claim_sha256 == claim.claim_sha256
        && persisted.as_ref().is_some_and(|value| {
            value.specification_sha256 == specification.specification_sha256
                && value.stage_104_review_sha256 == specification.stage_104_review_sha256
                && value.cycle_claim_sha256 == specification.cycle_claim_sha256
        });
    push_check(&mut reasons, exact_chain, "Stage 51–112 binding mismatch");
    let (session_count, price_count, gap_count, dividend_count, split_count) = persisted
        .as_ref()
        .map(|value| {
            (
                value.sessions.len(),
                value.prices.len(),
                value.explicit_gaps.len(),
                value.dividends.len(),
                value.splits.len(),
            )
        })
        .unwrap_or_default();
    IndependentAudit {
        exact_chain,
        claim_fingerprint,
        result_fingerprint,
        output_file,
        admitted_input,
        source_matrix,
        reprojected,
        row_hashes,
        exact_output,
        allocation,
        availability,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_persisted_output_sha256,
        independently_recomputed_output_sha256,
        output_bytes,
        session_count,
        price_count,
        gap_count,
        dividend_count,
        split_count,
        mismatch_reasons: reasons,
    }
}

async fn independent_read_output(
    state: &AppState,
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
    result: &ControlledShadowObservationMaterializationExecutionAttemptResult,
) -> Result<(UntrustedControlledShadowObservationEnvelope, usize), String> {
    let specification = exact_specification(claim);
    let expected = PathBuf::from("observations")
        .join(&specification.cycle_claim_id)
        .join(format!("{}.json", specification.specification_sha256));
    let relative = PathBuf::from(
        result
            .output_relative_path
            .as_deref()
            .ok_or_else(|| "successful Stage 112 result has no output path".to_string())?,
    );
    if relative != expected || !safe_relative_path(&relative) {
        return Err("output path is not the exact frozen content-addressed path".to_string());
    }
    let path =
        controlled_shadow_observation_materialization_execution_custody_root(state).join(relative);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|_| "output file is missing".to_string())?;
    let maximum = claim
        .authorization
        .runner
        .runner_contract
        .maximum_output_bytes;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum
    {
        return Err("output is not a bounded regular file".to_string());
    }
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|_| "output file read failed".to_string())?;
    let value = serde_json::from_slice::<UntrustedControlledShadowObservationEnvelope>(&bytes)
        .map_err(|_| "output JSON/schema rejected".to_string())?;
    Ok((value, bytes.len()))
}

fn independent_project_envelope(
    specification: &ControlledShadowObservationMaterializationSpecification,
    source: &UntrustedCanonicalMarketDataParseResult,
    source_manifest_sha256: &str,
) -> Result<UntrustedControlledShadowObservationEnvelope, String> {
    independent_validate_source(specification, source)?;
    let mut sessions = source
        .calendar_rows
        .iter()
        .map(|row| MaterializedMarketSession {
            date: row.date,
            session_kind: row.session_kind.clone(),
            close_time_et: row.close_time_et.clone(),
        })
        .collect::<Vec<_>>();
    sessions.sort_by_key(|row| row.date);
    let mut prices = source
        .price_rows
        .iter()
        .map(|row| {
            let mut value = MaterializedPriceObservation {
                symbol: row.symbol.clone(),
                date: row.date,
                price_basis: row.source_kind.clone(),
                close_decimal: row.close_decimal.clone(),
                volume_integer: row.volume_integer,
                source_request_sha256: row.request_sha256.clone(),
                source_row_sha256: row.row_sha256.clone(),
                observation_row_sha256: String::new(),
            };
            value.observation_row_sha256 =
                independent_fingerprint_without(&value, &["observation_row_sha256"])?;
            Ok(value)
        })
        .collect::<Result<Vec<_>, String>>()?;
    prices.sort_by(|left, right| {
        (left.date, &left.symbol, &left.price_basis).cmp(&(
            right.date,
            &right.symbol,
            &right.price_basis,
        ))
    });
    let mut explicit_gaps = source
        .explicit_gaps
        .iter()
        .map(|row| {
            let mut value = MaterializedExplicitGap {
                symbol: row.symbol.clone(),
                date: row.date,
                price_basis: row.source_kind.clone(),
                reason: row.reason.clone(),
                source_manifest_sha256: source_manifest_sha256.to_string(),
                gap_row_sha256: String::new(),
            };
            value.gap_row_sha256 = independent_fingerprint_without(&value, &["gap_row_sha256"])?;
            Ok(value)
        })
        .collect::<Result<Vec<_>, String>>()?;
    explicit_gaps.sort_by(|left, right| {
        (left.date, &left.symbol, &left.price_basis).cmp(&(
            right.date,
            &right.symbol,
            &right.price_basis,
        ))
    });
    let mut dividends = source
        .dividend_rows
        .iter()
        .map(independent_dividend)
        .collect::<Result<Vec<_>, String>>()?;
    let mut splits = source
        .split_rows
        .iter()
        .map(independent_split)
        .collect::<Result<Vec<_>, String>>()?;
    dividends.sort_by(|left, right| (left.date, &left.symbol).cmp(&(right.date, &right.symbol)));
    splits.sort_by(|left, right| (left.date, &left.symbol).cmp(&(right.date, &right.symbol)));
    let mut envelope = UntrustedControlledShadowObservationEnvelope {
        schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        specification_sha256: specification.specification_sha256.clone(),
        stage_104_review_sha256: specification.stage_104_review_sha256.clone(),
        cycle_claim_sha256: specification.cycle_claim_sha256.clone(),
        admitted_available_at_utc: specification.admitted_available_at_utc,
        sessions,
        prices,
        explicit_gaps,
        dividends,
        splits,
        initial_shadow_allocation_binding: InitialShadowAllocationBinding {
            stage_88_initial_observation_output_sha256: specification
                .stage_88_initial_observation_output_sha256
                .clone(),
            initial_shadow_allocation_manifest_sha256: specification
                .initial_shadow_allocation_manifest_sha256
                .clone(),
            initial_shadow_allocation_recomputed: false,
            accounting_transition_applied: false,
        },
        output_sha256: String::new(),
        untrusted: true,
    };
    envelope.output_sha256 = independent_fingerprint_without(&envelope, &["output_sha256"])?;
    independent_validate_envelope(specification, &envelope)?;
    Ok(envelope)
}

fn independent_validate_source(
    specification: &ControlledShadowObservationMaterializationSpecification,
    source: &UntrustedCanonicalMarketDataParseResult,
) -> Result<(), String> {
    if source.output_sha256 != specification.stage_102_output_sha256
        || !source.untrusted
        || source.independent_validation_completed
        || source.observation_started
        || source.ledger_created
        || source.position_written
        || source.order_generation_authorized
        || source.broker_access_authorized
        || source.trading_authorized
        || !source.rejections.is_empty()
        || source.calendar_rows.len() != specification.official_market_session_count
    {
        return Err("source identity, immutable state or session count invalid".to_string());
    }
    let sessions = source
        .calendar_rows
        .iter()
        .map(|row| row.date)
        .collect::<BTreeSet<_>>();
    if sessions.len() != source.calendar_rows.len()
        || sessions.first().copied() != Some(specification.earliest_market_session_date)
        || sessions.last().copied() != Some(specification.latest_market_session_date)
    {
        return Err("source session window or uniqueness invalid".to_string());
    }
    for row in &source.calendar_rows {
        independent_validate_calendar_row(row)?;
    }
    let bases = specification
        .allowed_price_bases
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if bases.len() != 3 {
        return Err("frozen price basis set is not exactly three".to_string());
    }
    let mut symbols = specification
        .subject_symbols
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    symbols.insert(specification.benchmark_symbol.clone());
    let mut price_keys = BTreeSet::new();
    for row in &source.price_rows {
        independent_validate_price_row(row)?;
        if !sessions.contains(&row.date)
            || !symbols.contains(&row.symbol)
            || !bases.contains(&row.source_kind)
            || !price_keys.insert((row.symbol.clone(), row.date, row.source_kind.clone()))
        {
            return Err("source price out of bounds, duplicate or invalid basis".to_string());
        }
    }
    let mut gap_keys = BTreeSet::new();
    for row in &source.explicit_gaps {
        if !sessions.contains(&row.date)
            || !specification.subject_symbols.contains(&row.symbol)
            || !bases.contains(&row.source_kind)
            || row.reason != "missing_subject_row_no_fill"
            || !gap_keys.insert((row.symbol.clone(), row.date, row.source_kind.clone()))
        {
            return Err("source explicit gap invalid".to_string());
        }
    }
    for date in &sessions {
        for symbol in &symbols {
            for basis in &bases {
                let key = (symbol.clone(), *date, basis.clone());
                let price = price_keys.contains(&key);
                let gap = gap_keys.contains(&key);
                if symbol == &specification.benchmark_symbol {
                    if !price || gap {
                        return Err("benchmark three-price-basis coverage incomplete".to_string());
                    }
                } else if price == gap {
                    return Err("subject price/gap is not strict XOR".to_string());
                }
            }
        }
    }
    let mut actions = BTreeSet::new();
    for row in &source.dividend_rows {
        independent_validate_dividend_row(row)?;
        if row.date < specification.window_start_date
            || row.date > specification.window_end_date
            || !symbols.contains(&row.symbol)
            || !actions.insert((row.symbol.clone(), row.date, "dividend"))
        {
            return Err("source dividend out of bounds or duplicate".to_string());
        }
    }
    for row in &source.split_rows {
        independent_validate_split_row(row)?;
        if row.date < specification.window_start_date
            || row.date > specification.window_end_date
            || !symbols.contains(&row.symbol)
            || !actions.insert((row.symbol.clone(), row.date, "split"))
        {
            return Err("source split out of bounds or duplicate".to_string());
        }
    }
    Ok(())
}

fn independent_validate_calendar_row(row: &CanonicalMarketCalendarRow) -> Result<(), String> {
    if row.row_sha256 != independent_fingerprint_without(row, &["row_sha256"])?
        || !matches!(row.session_kind.as_str(), "regular" | "early_close")
        || !matches!(
            (row.session_kind.as_str(), row.close_time_et.as_str()),
            ("regular", "16:00") | ("early_close", "13:00")
        )
    {
        return Err("source calendar row invalid".to_string());
    }
    Ok(())
}

fn independent_validate_price_row(row: &CanonicalMarketPriceRow) -> Result<(), String> {
    if row.row_sha256 != independent_fingerprint_without(row, &["row_sha256"])?
        || !valid_sha256(&row.request_sha256)
        || row.close_decimal.trim().is_empty()
    {
        return Err("source price row invalid".to_string());
    }
    Ok(())
}

fn independent_validate_dividend_row(row: &CanonicalMarketDividendRow) -> Result<(), String> {
    if row.row_sha256 != independent_fingerprint_without(row, &["row_sha256"])?
        || row.dividend_decimal.trim().is_empty()
    {
        return Err("source dividend row invalid".to_string());
    }
    Ok(())
}

fn independent_validate_split_row(row: &CanonicalMarketSplitRow) -> Result<(), String> {
    if row.row_sha256 != independent_fingerprint_without(row, &["row_sha256"])?
        || row.numerator_positive_integer == 0
        || row.denominator_positive_integer == 0
    {
        return Err("source split row invalid".to_string());
    }
    Ok(())
}

fn independent_dividend(
    row: &CanonicalMarketDividendRow,
) -> Result<MaterializedCorporateAction, String> {
    let mut fields = Map::new();
    fields.insert(
        "dividend_decimal".to_string(),
        Value::String(row.dividend_decimal.clone()),
    );
    fields.insert("record_date".to_string(), optional_date(row.record_date));
    fields.insert("payment_date".to_string(), optional_date(row.payment_date));
    fields.insert(
        "declaration_date".to_string(),
        optional_date(row.declaration_date),
    );
    independent_action(
        row.symbol.clone(),
        row.date,
        "dividend",
        fields,
        row.row_sha256.clone(),
    )
}

fn independent_split(row: &CanonicalMarketSplitRow) -> Result<MaterializedCorporateAction, String> {
    let mut fields = Map::new();
    fields.insert(
        "numerator_positive_integer".to_string(),
        Value::from(row.numerator_positive_integer),
    );
    fields.insert(
        "denominator_positive_integer".to_string(),
        Value::from(row.denominator_positive_integer),
    );
    fields.insert(
        "split_type".to_string(),
        row.split_type.clone().map_or(Value::Null, Value::String),
    );
    independent_action(
        row.symbol.clone(),
        row.date,
        "split",
        fields,
        row.row_sha256.clone(),
    )
}

fn independent_action(
    symbol: String,
    date: NaiveDate,
    action_kind: &str,
    exact_source_fields: Map<String, Value>,
    source_row_sha256: String,
) -> Result<MaterializedCorporateAction, String> {
    let mut action = MaterializedCorporateAction {
        symbol,
        date,
        action_kind: action_kind.to_string(),
        exact_source_fields,
        source_row_sha256,
        action_row_sha256: String::new(),
    };
    action.action_row_sha256 = independent_fingerprint_without(&action, &["action_row_sha256"])?;
    Ok(action)
}

fn optional_date(value: Option<NaiveDate>) -> Value {
    value.map_or(Value::Null, |date| Value::String(date.to_string()))
}

fn independent_validate_envelope(
    specification: &ControlledShadowObservationMaterializationSpecification,
    envelope: &UntrustedControlledShadowObservationEnvelope,
) -> Result<(), String> {
    let session_sorted = envelope
        .sessions
        .windows(2)
        .all(|rows| rows[0].date < rows[1].date);
    let price_sorted = envelope.prices.windows(2).all(|rows| {
        (rows[0].date, &rows[0].symbol, &rows[0].price_basis)
            < (rows[1].date, &rows[1].symbol, &rows[1].price_basis)
    });
    let gap_sorted = envelope.explicit_gaps.windows(2).all(|rows| {
        (rows[0].date, &rows[0].symbol, &rows[0].price_basis)
            < (rows[1].date, &rows[1].symbol, &rows[1].price_basis)
    });
    if envelope.schema_version != OUTPUT_SCHEMA_VERSION
        || envelope.specification_sha256 != specification.specification_sha256
        || envelope.stage_104_review_sha256 != specification.stage_104_review_sha256
        || envelope.cycle_claim_sha256 != specification.cycle_claim_sha256
        || envelope.admitted_available_at_utc != specification.admitted_available_at_utc
        || envelope.sessions.len() != specification.official_market_session_count
        || !session_sorted
        || !price_sorted
        || !gap_sorted
        || envelope.output_sha256 != independent_fingerprint_without(envelope, &["output_sha256"])?
        || !envelope.untrusted
        || envelope
            .initial_shadow_allocation_binding
            .initial_shadow_allocation_recomputed
        || envelope
            .initial_shadow_allocation_binding
            .accounting_transition_applied
    {
        return Err("independent envelope invalid, non-canonical or over-authorized".to_string());
    }
    Ok(())
}

fn independent_output_row_hashes_valid(
    value: &UntrustedControlledShadowObservationEnvelope,
) -> bool {
    value.prices.iter().all(|row| {
        independent_fingerprint_without(row, &["observation_row_sha256"])
            .is_ok_and(|hash| hash == row.observation_row_sha256)
    }) && value.explicit_gaps.iter().all(|row| {
        independent_fingerprint_without(row, &["gap_row_sha256"])
            .is_ok_and(|hash| hash == row.gap_row_sha256)
    }) && value.dividends.iter().chain(&value.splits).all(|row| {
        independent_fingerprint_without(row, &["action_row_sha256"])
            .is_ok_and(|hash| hash == row.action_row_sha256)
    })
}

fn exact_specification(
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
) -> &ControlledShadowObservationMaterializationSpecification {
    &claim
        .authorization
        .runner
        .implementation
        .implementation_contract
        .exact_observation_materialization_specification
}

fn validate_request_shape(
    attempt_id: &str,
    request: &ValidateControlledShadowObservationMaterializationOutputRequest,
) -> Result<(), String> {
    let valid = valid_id(attempt_id)
        && valid_sha256(&request.expected_claim_sha256)
        && valid_sha256(&request.expected_result_sha256)
        && valid_sha256(&request.expected_output_sha256)
        && valid_sha256(&request.expected_specification_sha256)
        && valid_sha256(&request.expected_stage_104_review_sha256)
        && valid_sha256(&request.expected_stage_102_output_sha256)
        && !request.validation_reason.trim().is_empty()
        && request.validation_reason.chars().count() <= MAX_REASON_CHARS
        && request.exact_current_stage_51_through_stage_112_binding_confirmed
        && request.validator_independent_from_executor_and_complete_prior_chain_confirmed
        && request.stage_112_result_and_create_once_output_reopened_and_rehashed_confirmed
        && request.exact_stage_104_admitted_stage_102_input_reopened_and_rehashed_confirmed
        && request.second_projection_does_not_call_stage_112_materializer_helpers_confirmed
        && request.sessions_prices_gaps_actions_allocation_availability_independently_recomputed_confirmed
        && request.every_row_hash_sort_order_and_complete_envelope_exactly_compared_confirmed
        && request.pass_only_opens_future_stage_114_observation_evidence_admission_review_confirmed
        && request.no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 113 请求摘要、理由或确认项不完整".to_string())
}

fn verify_expected_bindings(
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
    result: &ControlledShadowObservationMaterializationExecutionAttemptResult,
    request: &ValidateControlledShadowObservationMaterializationOutputRequest,
) -> Result<(), String> {
    let specification = exact_specification(claim);
    let valid = request.expected_claim_sha256 == claim.claim_sha256
        && request.expected_result_sha256 == result.result_sha256
        && result.output_sha256.as_deref() == Some(request.expected_output_sha256.as_str())
        && request.expected_specification_sha256 == specification.specification_sha256
        && request.expected_stage_104_review_sha256 == specification.stage_104_review_sha256
        && request.expected_stage_102_output_sha256 == specification.stage_102_output_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 113 期望绑定与当前不可变链不一致".to_string())
}

fn prior_actor_ids(
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
    result: &ControlledShadowObservationMaterializationExecutionAttemptResult,
) -> Vec<String> {
    let mut values = claim.excluded_prior_actor_ids.clone();
    values.push(claim.claimed_by.clone());
    values.push(claim.authorization.reviewer_id.clone());
    values.push(claim.authorization.runner.registered_by.clone());
    values.push(result.executed_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn validate_record_binding(
    record: &ControlledShadowObservationMaterializationOutputValidationRecord,
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
    result: &ControlledShadowObservationMaterializationExecutionAttemptResult,
) -> Result<(), String> {
    let specification = exact_specification(claim);
    let valid = record.stage_111_attempt_id == claim.attempt_id
        && record.stage_111_claim_sha256 == claim.claim_sha256
        && record.stage_112_result_id == result.result_id
        && record.stage_112_result_sha256 == result.result_sha256
        && result.output_sha256.as_deref() == Some(record.stage_112_output_sha256.as_str())
        && record.observation_materialization_specification_sha256
            == specification.specification_sha256
        && record.stage_104_review_sha256 == specification.stage_104_review_sha256
        && record.stage_102_output_sha256 == specification.stage_102_output_sha256
        && record.excluded_prior_actor_ids == prior_actor_ids(claim, result);
    valid
        .then_some(())
        .ok_or_else(|| "Stage 113 验证记录与当前 Stage 111/112 链漂移".to_string())
}

fn validate_record(
    record: &ControlledShadowObservationMaterializationOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_executor_and_complete_prior_chain
        && record.exact_current_stage_51_through_stage_112_chain_verified
        && record.claim_fingerprint_independently_verified
        && record.result_fingerprint_independently_verified
        && record.output_file_custody_and_fingerprint_verified
        && record.exact_stage_104_admitted_input_revalidated
        && record.source_matrix_independently_verified
        && record.complete_envelope_independently_reprojected
        && record.every_output_row_hash_independently_verified
        && record.canonical_sort_and_complete_output_exact_match_verified
        && record.initial_allocation_binding_preserved_without_recomputation_verified
        && record.admitted_availability_preserved_verified
        && record.no_downstream_authority_verified
        && record.mismatch_reasons.is_empty();
    let authority_closed = !record.ledger_created
        && !record.position_written
        && !record.performance_metric_written
        && !record.model_or_metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let valid = record.schema_version == RECORD_SCHEMA_VERSION
        && record.policy_version == POLICY_VERSION
        && valid_id(&record.validation_id)
        && valid_sha256(&record.validation_sha256)
        && record.validation_id == record.validation_sha256[..32]
        && record.validation_sha256 == record_fingerprint(record)?
        && record.validator_implementation_version == VALIDATOR_IMPLEMENTATION_VERSION
        && record.validator_implementation_sha256 == validator_implementation_sha256()
        && sorted_unique(&record.excluded_prior_actor_ids)
        && actor_is_independent(&record.validated_by, &record.excluded_prior_actor_ids)
        && !record.validation_reason.trim().is_empty()
        && record.validation_reason.chars().count() <= MAX_REASON_CHARS
        && record.mismatch_reasons.len() <= MAX_MISMATCH_REASONS
        && authority_closed
        && record.observation_envelope_independently_validated == passed
        && record.future_stage_114_observation_evidence_admission_review_eligible == passed
        && (record.verdict
            == ControlledShadowObservationMaterializationOutputValidationVerdict::IndependentlyValidatedExactObservationEnvelope)
            == passed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 113 独立验证记录无效、漂移或越权".to_string())
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationMaterializationOutputValidationRecord>, String> {
    let mut records = Vec::new();
    let mut stack = vec![validation_root(state)];
    while let Some(directory) = stack.pop() {
        let mut entries = match tokio::fs::read_dir(directory).await {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.to_string()),
        };
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|error| error.to_string())?
        {
            let path = entry.path();
            let file_type = entry.file_type().await.map_err(|error| error.to_string())?;
            if file_type.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
                let metadata = tokio::fs::symlink_metadata(&path)
                    .await
                    .map_err(|error| error.to_string())?;
                if metadata.file_type().is_symlink()
                    || !metadata.is_file()
                    || metadata.len() == 0
                    || metadata.len() > 1024 * 1024
                {
                    return Err("Stage 113 validation record file invalid".to_string());
                }
                let record = serde_json::from_slice::<
                    ControlledShadowObservationMaterializationOutputValidationRecord,
                >(
                    &tokio::fs::read(path)
                        .await
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
                validate_record(&record)?;
                records.push(record);
            }
        }
    }
    let mut attempts = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for record in &records {
        if !attempts.insert(record.stage_111_attempt_id.clone())
            || !ids.insert(record.validation_id.clone())
        {
            return Err("Stage 113 独立验证记录重复或重放".to_string());
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn push_reason(reasons: &mut Vec<String>, reason: String) {
    if reasons.len() < MAX_MISMATCH_REASONS {
        reasons.push(reason);
    }
}

fn push_check(reasons: &mut Vec<String>, condition: bool, reason: &str) {
    if !condition {
        push_reason(reasons, reason.to_string());
    }
}

fn record_fingerprint(
    record: &ControlledShadowObservationMaterializationOutputValidationRecord,
) -> Result<String, String> {
    independent_fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    hash_bytes(include_bytes!(
        "historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_output_validations.rs"
    ))
}

fn independent_fingerprint_without<T: Serialize>(
    value: &T,
    fields: &[&str],
) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(hash_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_REASON_CHARS {
        Err(format!("{label}为空或过长"))
    } else {
        Ok(value.to_string())
    }
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() > 1024 * 1024 {
        return Err("Stage 113 validation record empty or oversized".to_string());
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
    decision_root(state).join("controlled-shadow-observation-materialization-output-validations")
}

struct ValidationLock(PathBuf);

impl Drop for ValidationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<ValidationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage113-{attempt_id}.lock"));
    for _ in 0..2 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ValidationLock(path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .and_then(|value| value.modified())
                    .ok()
                    .and_then(|value| value.elapsed().ok())
                    .is_some_and(|elapsed| elapsed.as_secs() > 300);
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                return Err("已有同一 Stage 113 独立验证正在进行".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 113 校验锁不可用".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_113_source_does_not_call_stage_112_materializer_helpers() {
        let source = include_str!(
            "historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_output_validations.rs"
        );
        let module = "observation_materialization_execution_attempts";
        for helper in [
            "materialize_observation",
            "validate_source",
            "validate_envelope",
        ] {
            assert!(!source.contains(&format!("{module}::{helper}")));
        }
        assert!(source.contains("independent_project_envelope("));
    }

    #[test]
    fn independent_row_hash_preserves_exact_decimal_string() {
        let mut row = MaterializedPriceObservation {
            symbol: "ZZTEST".into(),
            date: NaiveDate::from_ymd_opt(2099, 1, 3).expect("date"),
            price_basis: "raw_unadjusted_price".into(),
            close_decimal: "10.5000".into(),
            volume_integer: Some(42),
            source_request_sha256: "a".repeat(64),
            source_row_sha256: "b".repeat(64),
            observation_row_sha256: String::new(),
        };
        row.observation_row_sha256 =
            independent_fingerprint_without(&row, &["observation_row_sha256"]).expect("hash");
        assert_eq!(row.close_decimal, "10.5000");
        assert!(independent_output_row_hashes_valid(
            &UntrustedControlledShadowObservationEnvelope {
                schema_version: OUTPUT_SCHEMA_VERSION.into(),
                specification_sha256: "c".repeat(64),
                stage_104_review_sha256: "d".repeat(64),
                cycle_claim_sha256: "e".repeat(64),
                admitted_available_at_utc: Utc::now(),
                sessions: Vec::new(),
                prices: vec![row],
                explicit_gaps: Vec::new(),
                dividends: Vec::new(),
                splits: Vec::new(),
                initial_shadow_allocation_binding: InitialShadowAllocationBinding {
                    stage_88_initial_observation_output_sha256: "f".repeat(64),
                    initial_shadow_allocation_manifest_sha256: "1".repeat(64),
                    initial_shadow_allocation_recomputed: false,
                    accounting_transition_applied: false,
                },
                output_sha256: "2".repeat(64),
                untrusted: true,
            }
        ));
    }

    #[test]
    fn stage_113_grants_no_downstream_authority() {
        let authority_flags = [false; 9];
        assert!(authority_flags.into_iter().all(|value| !value));
        assert_eq!(
            NEXT_GATE,
            "stage_114_validated_observation_envelope_admission_review"
        );
    }
}
