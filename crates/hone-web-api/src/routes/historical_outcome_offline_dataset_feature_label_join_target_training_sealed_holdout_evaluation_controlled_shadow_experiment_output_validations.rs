//! Chain-external Stage 81 validation for Stage 80 initial shadow observations.
//!
//! The validator must resubmit the exact content-addressed point-in-time input
//! and independently rebuild preprocessing, model prediction, ranking and all
//! five portfolio caps. A validation record is create-once. Passing only proves
//! reproducibility of the zero-forward-session initialization envelope; it does
//! not create a ledger, position, performance metric, order or trading authority.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::{
    HistoricalOutcomeTrainingFeaturePreprocessingRecord, HistoricalOutcomeTrainingTargetModel,
    HistoricalOutcomeTrainingTargetModelBody, HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::HistoricalOutcomeTrainingExperimentAlgorithm;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_execution_attempts::{
    CompletedControlledShadowExperimentExecutionAttempt,
    ControlledShadowExperimentUntrustedInitialObservationEnvelope,
    ControlledShadowPointInTimeInputEnvelope, ControlledShadowSeedPrediction,
    ControlledShadowVirtualAllocationObservation,
    completed_controlled_shadow_execution_attempts_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_first_execution_authorizations::{
    ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    exact_controlled_shadow_first_execution_authorization_for_audit,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registrations::ControlledShadowExperimentPortfolioConstraints;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_execution_attempts::exact_training_envelope_for_controlled_shadow_execution;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts::parse_f64_hex;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str = "hone-controlled-shadow-experiment-output-validation-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-initial-observation-chain-external-bitwise-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-controlled-shadow-initial-observation-validator-v1-independent-projection-path";
const INPUT_SCHEMA_VERSION: &str = "controlled_shadow_point_in_time_read_only_input_v1_not_mounted";
const OUTPUT_SCHEMA_VERSION: &str =
    "controlled_shadow_checkpoint_observation_envelope_v1_no_order_intent";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const MAX_INPUT_ROWS: usize = 100;
const MAX_SOURCE_COUNT: usize = 256;
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateControlledShadowExperimentOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_design_specification_sha256: String,
    expected_candidate_set_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    expected_input_manifest_sha256: String,
    input: ControlledShadowPointInTimeInputEnvelope,
    independent_reopen_and_second_implementation_recomputation_confirmed: bool,
    exact_current_stage_51_through_stage_80_binding_confirmed: bool,
    validator_independent_from_executor_and_complete_prior_chain_confirmed: bool,
    exact_content_addressed_point_in_time_input_resubmitted_confirmed: bool,
    exact_three_seed_predictions_ranking_and_five_caps_recomputed_confirmed: bool,
    zero_forward_sessions_and_no_performance_fabrication_confirmed: bool,
    validated_output_remains_untrusted_pending_forward_observation_confirmed: bool,
    no_ledger_position_store_feedback_reward_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowExperimentOutputValidationVerdict {
    IndependentlyValidatedUntrustedInitialObservation,
    FailedIndependentInitialObservationValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub input_manifest_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub implementation_contract_sha256: String,
    pub design_specification_sha256: String,
    pub candidate_set_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_execution_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_80_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub original_envelope_fingerprint_independently_verified: bool,
    pub input_manifest_fingerprint_independently_verified: bool,
    pub exact_training_artifact_and_frozen_contract_verified: bool,
    pub exact_three_seed_predictions_bitwise_recomputed: bool,
    pub exact_ranking_and_tie_break_recomputed: bool,
    pub single_name_theme_gross_cash_and_position_caps_recomputed: bool,
    pub zero_forward_sessions_and_no_performance_verified: bool,
    pub no_downstream_authority_verified: bool,
    pub recomputed_claim_sha256: String,
    pub recomputed_result_sha256: String,
    pub recomputed_original_envelope_sha256: String,
    pub recomputed_input_manifest_sha256: String,
    pub independently_recomputed_output_sha256: String,
    pub independently_recomputed_allocation_count: usize,
    pub independently_recomputed_virtual_gross_exposure_bps: u32,
    pub independently_recomputed_virtual_cash_weight_bps: u32,
    pub mismatch_reasons: Vec<String>,
    pub verdict: ControlledShadowExperimentOutputValidationVerdict,
    pub initial_observation_independently_validated: bool,
    pub future_forward_observation_protocol_registration_eligible: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentOutputValidationItem {
    pub attempt: CompletedControlledShadowExperimentExecutionAttempt,
    pub validation: Option<ControlledShadowExperimentOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<ControlledShadowExperimentOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_initial_observation_count: usize,
    pub failed_validation_count: usize,
    pub future_forward_observation_protocol_registration_eligible_count: usize,
    pub validation_status: String,
    pub independent_output_validation_available: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowExperimentOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_initial_observation_count: usize,
    pub failed_validation_count: usize,
    pub future_forward_observation_protocol_registration_eligible_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyValidatedControlledShadowInitialObservation {
    pub attempt: CompletedControlledShadowExperimentExecutionAttempt,
    pub validation: ControlledShadowExperimentOutputValidationRecord,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    original_envelope_fingerprint: bool,
    input_manifest_fingerprint: bool,
    exact_training_inputs: bool,
    predictions_exact: bool,
    ranking_exact: bool,
    caps_exact: bool,
    zero_forward_no_performance: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_original_envelope_sha256: String,
    recomputed_input_manifest_sha256: String,
    independently_recomputed_output_sha256: String,
    independently_recomputed_allocation_count: usize,
    independently_recomputed_virtual_gross_exposure_bps: u32,
    independently_recomputed_virtual_cash_weight_bps: u32,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_controlled_shadow_experiment_output_validations(
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
            warn!(%error, "controlled-shadow output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "受控影子初始观察独立校验记录暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_controlled_shadow_experiment_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateControlledShadowExperimentOutputRequest>,
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

pub(crate) async fn controlled_shadow_experiment_output_validation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowExperimentOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(ControlledShadowExperimentOutputValidationReadinessSummary {
        validation_eligible_count: registry.validation_eligible_count,
        validation_count: registry.validation_count,
        independently_validated_initial_observation_count: registry
            .independently_validated_initial_observation_count,
        failed_validation_count: registry.failed_validation_count,
        future_forward_observation_protocol_registration_eligible_count: registry
            .future_forward_observation_protocol_registration_eligible_count,
        validation_status: registry.validation_status,
    })
}

pub(crate) async fn independently_validated_controlled_shadow_initial_observations_for_protocol_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedControlledShadowInitialObservation>, String> {
    let registry = build_registry(state).await?;
    let mut output = registry
        .items
        .into_iter()
        .filter_map(|item| {
            let validation = item.validation?;
            (validation.verdict
                == ControlledShadowExperimentOutputValidationVerdict::IndependentlyValidatedUntrustedInitialObservation
                && validation.initial_observation_independently_validated
                && validation.future_forward_observation_protocol_registration_eligible)
                .then_some(IndependentlyValidatedControlledShadowInitialObservation {
                    attempt: item.attempt,
                    validation,
                })
        })
        .collect::<Vec<_>>();
    output.sort_by(|left, right| {
        left.validation
            .validation_id
            .cmp(&right.validation.validation_id)
    });
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowExperimentOutputValidationRegistry, String> {
    let attempts =
        completed_controlled_shadow_execution_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.attempt_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let items = attempts
        .into_iter()
        .map(|attempt| {
            let validation = index
                .get(attempt.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            ControlledShadowExperimentOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let independently_validated_initial_observation_count = validations
        .iter()
        .filter(|record| record.initial_observation_independently_validated)
        .count();
    let failed_validation_count = validations
        .len()
        .saturating_sub(independently_validated_initial_observation_count);
    let validation_status = if failed_validation_count > 0 {
        "controlled_shadow_initial_observation_independent_validation_failed_fail_closed"
    } else if independently_validated_initial_observation_count > 0 {
        "initial_observation_independently_validated_waiting_forward_observation_protocol"
    } else if validation_eligible_count > 0 {
        "waiting_chain_external_initial_observation_recomputation"
    } else {
        "waiting_completed_stage_80_initial_observation"
    };
    Ok(ControlledShadowExperimentOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count: validations.len(),
        independently_validated_initial_observation_count,
        failed_validation_count,
        future_forward_observation_protocol_registration_eligible_count:
            independently_validated_initial_observation_count,
        validation_status: validation_status.to_string(),
        independent_output_validation_available: true,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 81 由 Stage 80 执行者和完整 Stage 51–80 责任链之外的新管理员，重新提交同一内容寻址点时输入，以不复用 Stage 80 投影、预测或权重函数的第二实现逐位复算三种子预测、排序和单股/主题/总敞口/现金/数量五重上限。通过只证明 0 前向观察日初始化可复现，并最多开放未来前向观察协议登记；不创建账本/持仓/绩效、模型/指标、反馈/reward、订单、券商或交易能力。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateControlledShadowExperimentOutputRequest,
) -> Result<ControlledShadowExperimentOutputValidationRecord, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.attempt_id == attempt_id)
    {
        return Err("这次 Stage 80 初始观察已经形成不可变独立校验记录，不得重放".to_string());
    }
    let attempt = completed_controlled_shadow_execution_attempts_for_independent_validation(state)
        .await?
        .into_iter()
        .find(|value| value.claim.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有完整且待独立校验的 Stage 80 初始观察".to_string())?;
    verify_expected_bindings(&attempt, &request)?;
    let authorization = exact_controlled_shadow_first_execution_authorization_for_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "Stage 80 绑定的 Stage 79 授权完整链已经无法重开".to_string())?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "Stage 81 校验者不能是 Stage 80 执行者或完整 Stage 51–80 责任链任一角色".to_string(),
        );
    }
    let registration = &authorization
        .runner
        .implementation
        .upstream_design_registration;
    let training = exact_training_envelope_for_controlled_shadow_execution(
        state,
        &registration.attempt_id,
        &registration.claim_sha256,
        &registration.result_sha256,
    )
    .await?;
    let audit = independently_audit(&attempt, &authorization, &training, &request.input);
    let passed = independent && audit.mismatch_reasons.is_empty();
    let mut record = ControlledShadowExperimentOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        output_sha256: attempt.result.output_sha256.clone().unwrap_or_default(),
        input_manifest_sha256: attempt.claim.input_manifest_sha256.clone(),
        authorization_review_id: attempt.claim.authorization_review_id.clone(),
        authorization_review_sha256: attempt.claim.authorization_review_sha256.clone(),
        isolated_runner_id: attempt.claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: attempt.claim.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: attempt.claim.runner_artifact_sha256.clone(),
        implementation_contract_sha256: attempt.claim.implementation_contract_sha256.clone(),
        design_specification_sha256: attempt.claim.design_specification_sha256.clone(),
        candidate_set_sha256: attempt.claim.candidate_set_sha256.clone(),
        feature_order_sha256: attempt.claim.feature_order_sha256.clone(),
        preprocessing_sha256: attempt.claim.preprocessing_sha256.clone(),
        target_id: attempt.claim.target_id.clone(),
        frozen_candidate_algorithm_id: attempt.claim.frozen_candidate_algorithm_id.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        invoked_by: attempt.claim.invoked_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_execution_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_80_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        original_envelope_fingerprint_independently_verified: audit.original_envelope_fingerprint,
        input_manifest_fingerprint_independently_verified: audit.input_manifest_fingerprint,
        exact_training_artifact_and_frozen_contract_verified: audit.exact_training_inputs,
        exact_three_seed_predictions_bitwise_recomputed: audit.predictions_exact,
        exact_ranking_and_tie_break_recomputed: audit.ranking_exact,
        single_name_theme_gross_cash_and_position_caps_recomputed: audit.caps_exact,
        zero_forward_sessions_and_no_performance_verified: audit.zero_forward_no_performance,
        no_downstream_authority_verified: audit.downstream_closed,
        recomputed_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_result_sha256: audit.recomputed_result_sha256,
        recomputed_original_envelope_sha256: audit.recomputed_original_envelope_sha256,
        recomputed_input_manifest_sha256: audit.recomputed_input_manifest_sha256,
        independently_recomputed_output_sha256: audit.independently_recomputed_output_sha256,
        independently_recomputed_allocation_count: audit.independently_recomputed_allocation_count,
        independently_recomputed_virtual_gross_exposure_bps: audit
            .independently_recomputed_virtual_gross_exposure_bps,
        independently_recomputed_virtual_cash_weight_bps: audit
            .independently_recomputed_virtual_cash_weight_bps,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            ControlledShadowExperimentOutputValidationVerdict::IndependentlyValidatedUntrustedInitialObservation
        } else {
            ControlledShadowExperimentOutputValidationVerdict::FailedIndependentInitialObservationValidation
        },
        initial_observation_independently_validated: passed,
        future_forward_observation_protocol_registration_eligible: passed,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
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

fn independently_audit(
    attempt: &CompletedControlledShadowExperimentExecutionAttempt,
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    training: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    input: &ControlledShadowPointInTimeInputEnvelope,
) -> IndependentAudit {
    let mut reasons = Vec::new();
    let recomputed_claim_sha256 =
        independent_fingerprint_without(&attempt.claim, &["attempt_id", "claim_sha256"])
            .unwrap_or_else(|error| {
                reasons.push(format!("claim fingerprint recomputation failed: {error}"));
                hash_bytes(b"claim-fingerprint-unavailable")
            });
    let recomputed_result_sha256 =
        independent_fingerprint_without(&attempt.result, &["result_id", "result_sha256"])
            .unwrap_or_else(|error| {
                reasons.push(format!("result fingerprint recomputation failed: {error}"));
                hash_bytes(b"result-fingerprint-unavailable")
            });
    let original = attempt.result.untrusted_initial_observation.as_ref();
    let recomputed_original_envelope_sha256 = original
        .and_then(|value| independent_hash_serializable(value).ok())
        .unwrap_or_else(|| hash_bytes(b"original-envelope-unavailable"));
    let recomputed_input_manifest_sha256 =
        independent_fingerprint_without(input, &["input_manifest_sha256"]).unwrap_or_else(
            |error| {
                reasons.push(format!("input manifest recomputation failed: {error}"));
                hash_bytes(b"input-manifest-unavailable")
            },
        );
    let claim_fingerprint = recomputed_claim_sha256 == attempt.claim.claim_sha256;
    let result_fingerprint = recomputed_result_sha256 == attempt.result.result_sha256;
    let original_envelope_fingerprint = attempt.result.output_sha256.as_deref()
        == Some(recomputed_original_envelope_sha256.as_str());
    let input_manifest_fingerprint = recomputed_input_manifest_sha256
        == attempt.claim.input_manifest_sha256
        && recomputed_input_manifest_sha256 == input.input_manifest_sha256;
    push_mismatch(
        &mut reasons,
        claim_fingerprint,
        "Stage 80 claim fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        result_fingerprint,
        "Stage 80 result fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        original_envelope_fingerprint,
        "Stage 80 original envelope fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        input_manifest_fingerprint,
        "resubmitted point-in-time input manifest mismatch",
    );

    let exact_chain = independent_exact_chain_matches(attempt, authorization);
    push_mismatch(&mut reasons, exact_chain, "Stage 51-80 binding mismatch");
    let exact_training_inputs = independent_validate_input_and_training(
        authorization,
        training,
        input,
        &recomputed_input_manifest_sha256,
    )
    .map(|_| true)
    .unwrap_or_else(|error| {
        reasons.push(error);
        false
    });

    let recomputed = if exact_training_inputs {
        independent_recompute_initial_observation(authorization, training, input)
    } else {
        Err("input or frozen training contract invalid".to_string())
    };
    let (
        independently_recomputed_output_sha256,
        independently_recomputed_allocation_count,
        independently_recomputed_virtual_gross_exposure_bps,
        independently_recomputed_virtual_cash_weight_bps,
        predictions_exact,
        ranking_exact,
        caps_exact,
    ) = match (recomputed, original) {
        (Ok(value), Some(original)) => {
            let output_sha = independent_hash_serializable(&value)
                .unwrap_or_else(|_| hash_bytes(b"recomputed-output-unavailable"));
            let predictions_exact = independent_predictions_equal(original, &value);
            let ranking_exact = independent_ranking_equal(original, &value);
            let caps_exact = independent_caps_equal(original, &value, authorization);
            push_mismatch(
                &mut reasons,
                predictions_exact,
                "three-seed predictions mismatch",
            );
            push_mismatch(&mut reasons, ranking_exact, "ranking or tie-break mismatch");
            push_mismatch(&mut reasons, caps_exact, "five portfolio caps mismatch");
            push_mismatch(
                &mut reasons,
                attempt.result.output_sha256.as_deref() == Some(output_sha.as_str()),
                "independently recomputed output hash mismatch",
            );
            (
                output_sha,
                value.virtual_allocations.len(),
                value.virtual_gross_exposure_bps,
                value.virtual_cash_weight_bps,
                predictions_exact,
                ranking_exact,
                caps_exact,
            )
        }
        (Err(error), _) => {
            reasons.push(format!("independent recomputation failed: {error}"));
            (
                hash_bytes(b"independent-output-unavailable"),
                0,
                0,
                0,
                false,
                false,
                false,
            )
        }
        (_, None) => {
            reasons.push("Stage 80 result lacks initial observation envelope".to_string());
            (
                hash_bytes(b"independent-output-unavailable"),
                0,
                0,
                0,
                false,
                false,
                false,
            )
        }
    };
    let zero_forward_no_performance = original.is_some_and(|value| {
        value.initialization_only
            && value.output_is_untrusted
            && !value.independent_output_validation_completed
            && value.forward_market_sessions_observed == 0
            && value.performance_metrics.is_empty()
    });
    let downstream_closed = original.is_some_and(|value| {
        !value.shadow_ledger_created
            && !value.shadow_position_written
            && !value.model_store_written
            && !value.metric_store_written
            && !value.training_feedback_written
            && !value.scalar_reward_written
            && !value.order_intent_present
            && !value.broker_payload_present
            && !value.trade_executed
            && !attempt.result.shadow_ledger_created
            && !attempt.result.shadow_position_written
            && !attempt.result.order_generated
            && !attempt.result.broker_accessed
            && !attempt.result.trade_executed
    });
    push_mismatch(
        &mut reasons,
        zero_forward_no_performance,
        "future performance or non-initialization claim present",
    );
    push_mismatch(
        &mut reasons,
        downstream_closed,
        "downstream authority is not closed",
    );
    reasons.truncate(MAX_MISMATCH_REASONS);
    IndependentAudit {
        exact_chain,
        claim_fingerprint,
        result_fingerprint,
        original_envelope_fingerprint,
        input_manifest_fingerprint,
        exact_training_inputs,
        predictions_exact,
        ranking_exact,
        caps_exact,
        zero_forward_no_performance,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_original_envelope_sha256,
        recomputed_input_manifest_sha256,
        independently_recomputed_output_sha256,
        independently_recomputed_allocation_count,
        independently_recomputed_virtual_gross_exposure_bps,
        independently_recomputed_virtual_cash_weight_bps,
        mismatch_reasons: reasons,
    }
}

fn independent_exact_chain_matches(
    attempt: &CompletedControlledShadowExperimentExecutionAttempt,
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
) -> bool {
    let contract = &authorization.runner.implementation.implementation_contract;
    attempt.claim.authorization_review_id == authorization.review.review_id
        && attempt.claim.authorization_review_sha256 == authorization.review.review_sha256
        && attempt.claim.isolated_runner_id == authorization.runner.isolated_runner_id
        && attempt.claim.isolated_runner_spec_sha256
            == authorization.runner.isolated_runner_spec_sha256
        && attempt.claim.runner_artifact_sha256 == authorization.runner.runner_artifact_sha256
        && attempt.claim.runner_code_revision == authorization.runner.runner_code_revision
        && attempt.claim.runner_contract_sha256
            == authorization.runner.runner_contract.contract_sha256
        && attempt.claim.implementation_sha256
            == authorization.runner.implementation.implementation_sha256
        && attempt.claim.implementation_contract_sha256 == contract.contract_sha256
        && attempt.claim.design_specification_sha256 == contract.design_specification_sha256
        && attempt.claim.candidate_set_sha256 == contract.candidate_set_sha256
        && attempt.claim.feature_order_sha256 == contract.feature_order_sha256
        && attempt.claim.preprocessing_sha256 == contract.preprocessing_sha256
        && attempt.claim.target_id == contract.target_id
        && attempt.claim.frozen_candidate_algorithm_id == contract.frozen_candidate_algorithm_id
        && attempt.claim.exact_random_seeds == EXPECTED_SEEDS
        && attempt.result.claim_sha256 == attempt.claim.claim_sha256
}

fn independent_validate_input_and_training(
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    training: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    input: &ControlledShadowPointInTimeInputEnvelope,
    recomputed_input_manifest_sha256: &str,
) -> Result<(), String> {
    let contract = &authorization.runner.implementation.implementation_contract;
    let design = &contract.exact_design_specification;
    if input.schema_version != INPUT_SCHEMA_VERSION
        || input.input_manifest_sha256 != recomputed_input_manifest_sha256
        || input.input_manifest_sha256.is_empty()
        || input.candidate_set_sha256 != contract.candidate_set_sha256
        || input.feature_order != training.feature_order
        || input.feature_order.len() != EXPECTED_FEATURE_COUNT
        || independent_hash_serializable(&input.feature_order)? != contract.feature_order_sha256
        || input.preprocessing_sha256 != contract.preprocessing_sha256
        || independent_hash_serializable(&training.preprocessing)? != contract.preprocessing_sha256
        || input.benchmark_symbol != design.benchmark_symbol
        || input.captured_at > input.signal_cutoff_at
        || input.expected_next_full_market_session_at <= input.signal_cutoff_at
        || !input.point_in_time_read_only
        || !input.content_addressed
        || !input.allowlisted_sources_only
        || !input.no_retroactive_revision
        || input.sources.is_empty()
        || input.sources.len() > MAX_SOURCE_COUNT
        || input.rows.is_empty()
        || input.rows.len() > MAX_INPUT_ROWS
    {
        return Err("independent input/training contract validation failed".to_string());
    }
    let benchmark = parse_f64_hex(&input.benchmark_adjusted_close_f64_bits_hex)?;
    if !benchmark.is_finite() || benchmark <= 0.0 {
        return Err("independent benchmark validation failed".to_string());
    }
    let mut source_hashes = BTreeSet::new();
    for source in &input.sources {
        if source.source_id.trim().is_empty()
            || !valid_sha256(&source.content_sha256)
            || source.available_at > input.signal_cutoff_at
            || !source_hashes.insert(source.content_sha256.clone())
        {
            return Err("independent source allowlist/content validation failed".to_string());
        }
    }
    let mut symbols = BTreeSet::new();
    for row in &input.rows {
        if !valid_symbol(&row.symbol)
            || !symbols.insert(row.symbol.clone())
            || row.frozen_theme_id.trim().is_empty()
            || row.frozen_theme_id.len() > 80
            || row.security_type != "us_listed_common_stock"
            || row.available_at > input.signal_cutoff_at
            || row.feature_values_f64_bits_hex.len() != EXPECTED_FEATURE_COUNT
            || row.source_content_sha256s.is_empty()
            || !sorted_unique(&row.source_content_sha256s)
            || row
                .source_content_sha256s
                .iter()
                .any(|value| !source_hashes.contains(value))
        {
            return Err(format!(
                "independent row validation failed for {}",
                row.symbol
            ));
        }
        let price = parse_f64_hex(&row.adjusted_close_f64_bits_hex)?;
        if !price.is_finite() || price <= 0.0 {
            return Err(format!("invalid price for {}", row.symbol));
        }
        for value in row.feature_values_f64_bits_hex.iter().flatten() {
            if !parse_f64_hex(value)?.is_finite() {
                return Err(format!("non-finite feature for {}", row.symbol));
            }
        }
    }
    Ok(())
}

fn independent_recompute_initial_observation(
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    training: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    input: &ControlledShadowPointInTimeInputEnvelope,
) -> Result<ControlledShadowExperimentUntrustedInitialObservationEnvelope, String> {
    let contract = &authorization.runner.implementation.implementation_contract;
    let design = &contract.exact_design_specification;
    let algorithm = independent_parse_algorithm(&contract.frozen_candidate_algorithm_id)?;
    let mut ranked = Vec::new();
    for row in &input.rows {
        if !row.eligible_in_frozen_universe || !row.tradable_at_signal_cutoff {
            continue;
        }
        let features = independent_preprocess_features(
            &row.feature_values_f64_bits_hex,
            &training.preprocessing,
        )?;
        let mut seed_predictions = Vec::new();
        let mut total = 0.0;
        for seed in EXPECTED_SEEDS {
            let artifact = training
                .model_artifacts
                .iter()
                .find(|value| value.algorithm == algorithm && value.random_seed == seed)
                .ok_or_else(|| format!("missing frozen candidate artifact for seed {seed}"))?;
            let model = artifact
                .target_models
                .iter()
                .find(|value| value.target_id == contract.target_id)
                .ok_or_else(|| format!("missing target model {}", contract.target_id))?;
            let prediction = independent_predict_target(model, &training.feature_order, &features)?;
            total += prediction;
            seed_predictions.push(ControlledShadowSeedPrediction {
                random_seed: seed,
                prediction_f64_bits_hex: f64_hex(prediction),
            });
        }
        ranked.push((row, seed_predictions, total / EXPECTED_SEEDS.len() as f64));
    }
    ranked.sort_by(|left, right| {
        right
            .2
            .total_cmp(&left.2)
            .then_with(|| left.0.symbol.cmp(&right.0.symbol))
    });
    let themes_and_scores = ranked
        .iter()
        .map(|(row, _, score)| (row.frozen_theme_id.clone(), *score))
        .collect::<Vec<_>>();
    let weights = independent_virtual_weights(&themes_and_scores, &design.portfolio_constraints);
    let gross = weights.iter().copied().sum::<u32>();
    let virtual_allocations = ranked
        .into_iter()
        .zip(weights)
        .enumerate()
        .map(|(index, ((row, seed_predictions, mean), weight))| {
            ControlledShadowVirtualAllocationObservation {
                symbol: row.symbol.clone(),
                frozen_theme_id: row.frozen_theme_id.clone(),
                adjusted_close_f64_bits_hex: row.adjusted_close_f64_bits_hex.clone(),
                seed_predictions,
                mean_prediction_f64_bits_hex: f64_hex(mean),
                rank: index + 1,
                virtual_target_weight_bps: weight,
                pending_next_session_simulated_execution: weight > 0,
            }
        })
        .collect();
    Ok(
        ControlledShadowExperimentUntrustedInitialObservationEnvelope {
            schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
            authorization_review_sha256: authorization.review.review_sha256.clone(),
            isolated_runner_spec_sha256: authorization.runner.isolated_runner_spec_sha256.clone(),
            runner_artifact_sha256: authorization.runner.runner_artifact_sha256.clone(),
            implementation_contract_sha256: contract.contract_sha256.clone(),
            design_specification_sha256: contract.design_specification_sha256.clone(),
            input_manifest_sha256: input.input_manifest_sha256.clone(),
            signal_cutoff_at: input.signal_cutoff_at,
            expected_next_full_market_session_at: input.expected_next_full_market_session_at,
            benchmark_symbol: input.benchmark_symbol.clone(),
            benchmark_adjusted_close_f64_bits_hex: input
                .benchmark_adjusted_close_f64_bits_hex
                .clone(),
            target_id: contract.target_id.clone(),
            frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id.clone(),
            exact_random_seeds: contract.random_seeds.clone(),
            virtual_notional_usd: design.portfolio_constraints.virtual_notional_usd,
            virtual_allocations,
            virtual_gross_exposure_bps: gross,
            virtual_cash_weight_bps: 10_000_u32.saturating_sub(gross),
            forward_market_sessions_observed: 0,
            minimum_forward_market_sessions: design
                .observation_contract
                .minimum_forward_market_sessions,
            checkpoint_market_sessions: design
                .observation_contract
                .checkpoint_market_sessions
                .clone(),
            performance_metrics: Vec::new(),
            initialization_only: true,
            output_is_untrusted: true,
            independent_output_validation_completed: false,
            shadow_ledger_created: false,
            shadow_position_written: false,
            model_store_written: false,
            metric_store_written: false,
            training_feedback_written: false,
            scalar_reward_written: false,
            order_intent_present: false,
            broker_payload_present: false,
            trade_executed: false,
        },
    )
}

fn independent_preprocess_features(
    values: &[Option<String>],
    preprocessing: &[HistoricalOutcomeTrainingFeaturePreprocessingRecord],
) -> Result<Vec<Option<f64>>, String> {
    if values.len() != EXPECTED_FEATURE_COUNT || preprocessing.len() != EXPECTED_FEATURE_COUNT {
        return Err("independent preprocessing expected exactly 65 features".to_string());
    }
    let mut projected = Vec::with_capacity(EXPECTED_FEATURE_COUNT);
    for index in 0..EXPECTED_FEATURE_COUNT {
        let prep = &preprocessing[index];
        match &values[index] {
            None if prep.missingness_preserved => projected.push(None),
            None => return Err(format!("{} missingness is not preserved", prep.feature_id)),
            Some(raw) => {
                let value = parse_f64_hex(raw)?;
                let mean = parse_f64_hex(&prep.train_mean_f64_bits_hex)?;
                let scale = parse_f64_hex(&prep.train_scale_f64_bits_hex)?;
                if !value.is_finite() || !mean.is_finite() || !scale.is_finite() || scale <= 0.0 {
                    return Err(format!("{} preprocessing is invalid", prep.feature_id));
                }
                projected.push(Some((value - mean) / scale));
            }
        }
    }
    Ok(projected)
}

fn independent_predict_target(
    model: &HistoricalOutcomeTrainingTargetModel,
    feature_order: &[String],
    features: &[Option<f64>],
) -> Result<f64, String> {
    if feature_order.len() != features.len() {
        return Err("feature order and values differ".to_string());
    }
    let indexes = feature_order
        .iter()
        .enumerate()
        .map(|(index, feature)| (feature.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let prediction = match &model.model {
        HistoricalOutcomeTrainingTargetModelBody::FrozenZeroPrediction {
            prediction_f64_bits_hex,
        } => parse_f64_hex(prediction_f64_bits_hex)?,
        HistoricalOutcomeTrainingTargetModelBody::RidgeRegression {
            intercept_f64_bits_hex,
            coefficients,
            ..
        } => {
            if coefficients.len() != feature_order.len() {
                return Err("ridge coefficient count differs".to_string());
            }
            let mut sum = parse_f64_hex(intercept_f64_bits_hex)?;
            for coefficient in coefficients {
                let index = *indexes
                    .get(coefficient.feature_id.as_str())
                    .ok_or_else(|| "ridge references unknown feature".to_string())?;
                sum += match features[index] {
                    Some(value) => {
                        value * parse_f64_hex(&coefficient.observed_value_coefficient_f64_bits_hex)?
                    }
                    None => parse_f64_hex(&coefficient.missing_indicator_coefficient_f64_bits_hex)?,
                };
            }
            sum
        }
        HistoricalOutcomeTrainingTargetModelBody::GradientBoostedRegression {
            learning_rate_f64_bits_hex,
            rounds,
            ..
        } => {
            let rate = parse_f64_hex(learning_rate_f64_bits_hex)?;
            let mut sum = 0.0;
            for stump in rounds {
                let missing = parse_f64_hex(&stump.missing_leaf_f64_bits_hex)?;
                let leaf = match (&stump.feature_id, &stump.threshold_f64_bits_hex) {
                    (Some(feature_id), Some(threshold)) => {
                        let index = *indexes.get(feature_id.as_str()).ok_or_else(|| {
                            "boosted stump references unknown feature".to_string()
                        })?;
                        match features[index] {
                            Some(value) if value <= parse_f64_hex(threshold)? => {
                                parse_f64_hex(&stump.low_leaf_f64_bits_hex)?
                            }
                            Some(_) => parse_f64_hex(&stump.high_leaf_f64_bits_hex)?,
                            None => missing,
                        }
                    }
                    _ => missing,
                };
                sum += rate * leaf;
            }
            sum
        }
    };
    if !prediction.is_finite() {
        return Err("independent prediction is non-finite".to_string());
    }
    Ok(prediction)
}

fn independent_virtual_weights(
    ranked_theme_and_score: &[(String, f64)],
    constraints: &ControlledShadowExperimentPortfolioConstraints,
) -> Vec<u32> {
    let maximum_positions = usize::from(constraints.maximum_position_count);
    let spendable = constraints
        .maximum_gross_exposure_bps
        .min(10_000_u32.saturating_sub(constraints.minimum_cash_weight_bps));
    let mut assigned = 0_u32;
    let mut themes = BTreeMap::<String, u32>::new();
    let mut result = Vec::with_capacity(ranked_theme_and_score.len());
    for (index, (theme, score)) in ranked_theme_and_score.iter().enumerate() {
        let theme_assigned = themes.get(theme).copied().unwrap_or(0);
        let weight = if index >= maximum_positions || *score <= 0.0 {
            0
        } else {
            constraints
                .maximum_single_name_weight_bps
                .min(spendable.saturating_sub(assigned))
                .min(
                    constraints
                        .maximum_theme_weight_bps
                        .saturating_sub(theme_assigned),
                )
        };
        result.push(weight);
        assigned = assigned.saturating_add(weight);
        themes.insert(theme.clone(), theme_assigned.saturating_add(weight));
    }
    result
}

fn independent_predictions_equal(
    original: &ControlledShadowExperimentUntrustedInitialObservationEnvelope,
    recomputed: &ControlledShadowExperimentUntrustedInitialObservationEnvelope,
) -> bool {
    original.virtual_allocations.len() == recomputed.virtual_allocations.len()
        && original
            .virtual_allocations
            .iter()
            .zip(&recomputed.virtual_allocations)
            .all(|(left, right)| {
                left.symbol == right.symbol
                    && left.seed_predictions == right.seed_predictions
                    && left.mean_prediction_f64_bits_hex == right.mean_prediction_f64_bits_hex
            })
}

fn independent_ranking_equal(
    original: &ControlledShadowExperimentUntrustedInitialObservationEnvelope,
    recomputed: &ControlledShadowExperimentUntrustedInitialObservationEnvelope,
) -> bool {
    original.virtual_allocations.len() == recomputed.virtual_allocations.len()
        && original
            .virtual_allocations
            .iter()
            .zip(&recomputed.virtual_allocations)
            .all(|(left, right)| {
                left.symbol == right.symbol
                    && left.frozen_theme_id == right.frozen_theme_id
                    && left.rank == right.rank
                    && left.adjusted_close_f64_bits_hex == right.adjusted_close_f64_bits_hex
            })
}

fn independent_caps_equal(
    original: &ControlledShadowExperimentUntrustedInitialObservationEnvelope,
    recomputed: &ControlledShadowExperimentUntrustedInitialObservationEnvelope,
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
) -> bool {
    let constraints = &authorization
        .runner
        .implementation
        .implementation_contract
        .exact_design_specification
        .portfolio_constraints;
    let mut theme_weights = BTreeMap::<&str, u32>::new();
    for allocation in &recomputed.virtual_allocations {
        if allocation.virtual_target_weight_bps > constraints.maximum_single_name_weight_bps {
            return false;
        }
        *theme_weights
            .entry(allocation.frozen_theme_id.as_str())
            .or_default() += allocation.virtual_target_weight_bps;
    }
    let active = recomputed
        .virtual_allocations
        .iter()
        .filter(|value| value.virtual_target_weight_bps > 0)
        .count();
    original
        .virtual_allocations
        .iter()
        .map(|value| value.virtual_target_weight_bps)
        .eq(recomputed
            .virtual_allocations
            .iter()
            .map(|value| value.virtual_target_weight_bps))
        && original.virtual_gross_exposure_bps == recomputed.virtual_gross_exposure_bps
        && original.virtual_cash_weight_bps == recomputed.virtual_cash_weight_bps
        && recomputed.virtual_gross_exposure_bps <= constraints.maximum_gross_exposure_bps
        && recomputed.virtual_cash_weight_bps >= constraints.minimum_cash_weight_bps
        && active <= usize::from(constraints.maximum_position_count)
        && theme_weights
            .values()
            .all(|value| *value <= constraints.maximum_theme_weight_bps)
}

fn independent_parse_algorithm(
    value: &str,
) -> Result<HistoricalOutcomeTrainingExperimentAlgorithm, String> {
    match value {
        "ridge_multi_target_regression" => {
            Ok(HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression)
        }
        "gradient_boosted_multi_target_regression" => {
            Ok(HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression)
        }
        _ => Err("unsupported frozen candidate algorithm".to_string()),
    }
}

fn verify_expected_bindings(
    attempt: &CompletedControlledShadowExperimentExecutionAttempt,
    request: &ValidateControlledShadowExperimentOutputRequest,
) -> Result<(), String> {
    let claim = &attempt.claim;
    let result = &attempt.result;
    if claim.claim_sha256 != request.expected_claim_sha256
        || result.result_sha256 != request.expected_result_sha256
        || result.output_sha256.as_deref() != Some(request.expected_output_sha256.as_str())
        || claim.authorization_review_sha256 != request.expected_authorization_review_sha256
        || claim.isolated_runner_spec_sha256 != request.expected_isolated_runner_spec_sha256
        || claim.runner_artifact_sha256 != request.expected_runner_artifact_sha256
        || claim.implementation_contract_sha256 != request.expected_implementation_contract_sha256
        || claim.design_specification_sha256 != request.expected_design_specification_sha256
        || claim.candidate_set_sha256 != request.expected_candidate_set_sha256
        || claim.feature_order_sha256 != request.expected_feature_order_sha256
        || claim.preprocessing_sha256 != request.expected_preprocessing_sha256
        || claim.target_id != request.expected_target_id
        || claim.frozen_candidate_algorithm_id != request.expected_frozen_candidate_algorithm_id
        || claim.input_manifest_sha256 != request.expected_input_manifest_sha256
        || request.input.input_manifest_sha256 != request.expected_input_manifest_sha256
    {
        return Err("Stage 80 claim/result、输出、输入或冻结绑定已变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn validate_request(
    attempt_id: &str,
    request: &ValidateControlledShadowExperimentOutputRequest,
) -> Result<(), String> {
    let hashes = [
        request.expected_claim_sha256.as_str(),
        request.expected_result_sha256.as_str(),
        request.expected_output_sha256.as_str(),
        request.expected_authorization_review_sha256.as_str(),
        request.expected_isolated_runner_spec_sha256.as_str(),
        request.expected_runner_artifact_sha256.as_str(),
        request.expected_implementation_contract_sha256.as_str(),
        request.expected_design_specification_sha256.as_str(),
        request.expected_candidate_set_sha256.as_str(),
        request.expected_feature_order_sha256.as_str(),
        request.expected_preprocessing_sha256.as_str(),
        request.expected_input_manifest_sha256.as_str(),
    ];
    if !valid_id(attempt_id)
        || hashes.iter().any(|value| !valid_sha256(value))
        || request.expected_target_id.trim().is_empty()
        || request.expected_target_id.len() > 160
        || !matches!(
            request.expected_frozen_candidate_algorithm_id.as_str(),
            "ridge_multi_target_regression" | "gradient_boosted_multi_target_regression"
        )
    {
        return Err("Stage 81 请求的 ID、SHA-256、目标或算法无效".to_string());
    }
    if !request.independent_reopen_and_second_implementation_recomputation_confirmed
        || !request.exact_current_stage_51_through_stage_80_binding_confirmed
        || !request.validator_independent_from_executor_and_complete_prior_chain_confirmed
        || !request.exact_content_addressed_point_in_time_input_resubmitted_confirmed
        || !request.exact_three_seed_predictions_ranking_and_five_caps_recomputed_confirmed
        || !request.zero_forward_sessions_and_no_performance_fabrication_confirmed
        || !request.validated_output_remains_untrusted_pending_forward_observation_confirmed
        || !request.no_ledger_position_store_feedback_reward_order_broker_or_trading_confirmed
    {
        return Err("必须确认责任链外复算、同一输入、零前向绩效和全部下游权限关闭".to_string());
    }
    Ok(())
}

fn prior_actor_ids(
    attempt: &CompletedControlledShadowExperimentExecutionAttempt,
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
) -> Vec<String> {
    let mut values = attempt.claim.excluded_prior_actor_ids.clone();
    values.extend(authorization.review.excluded_prior_actor_ids.clone());
    values.push(attempt.claim.invoked_by.clone());
    values.push(authorization.review.reviewer_id.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn validate_record(
    record: &ControlledShadowExperimentOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_execution_and_complete_prior_chain
        && record.exact_current_stage_51_through_stage_80_chain_verified
        && record.claim_fingerprint_independently_verified
        && record.result_fingerprint_independently_verified
        && record.original_envelope_fingerprint_independently_verified
        && record.input_manifest_fingerprint_independently_verified
        && record.exact_training_artifact_and_frozen_contract_verified
        && record.exact_three_seed_predictions_bitwise_recomputed
        && record.exact_ranking_and_tie_break_recomputed
        && record.single_name_theme_gross_cash_and_position_caps_recomputed
        && record.zero_forward_sessions_and_no_performance_verified
        && record.no_downstream_authority_verified
        && record.mismatch_reasons.is_empty();
    let authority_closed = !record.forward_observation_ledger_created
        && !record.shadow_position_written
        && !record.performance_metric_written
        && !record.model_artifact_store_written
        && !record.metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.validation_id)
        || record.validation_sha256 != record_fingerprint(record)?
        || record.validation_id != record.validation_sha256[..32]
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &record.validated_by)
        || !authority_closed
        || record.initial_observation_independently_validated != passed
        || record.future_forward_observation_protocol_registration_eligible != passed
        || (passed
            != (record.verdict
                == ControlledShadowExperimentOutputValidationVerdict::IndependentlyValidatedUntrustedInitialObservation))
    {
        return Err("Stage 81 独立校验记录无效、漂移或越权".to_string());
    }
    Ok(())
}

fn push_mismatch(reasons: &mut Vec<String>, condition: bool, message: &str) {
    if !condition && reasons.len() < MAX_MISMATCH_REASONS {
        reasons.push(message.to_string());
    }
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<ControlledShadowExperimentOutputValidationRecord>, String> {
    let mut records = Vec::new();
    let root = validation_root(state);
    let mut stack = vec![root];
    while let Some(directory) = stack.pop() {
        let mut entries = match tokio::fs::read_dir(&directory).await {
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
                let record: ControlledShadowExperimentOutputValidationRecord =
                    serde_json::from_slice(
                        &tokio::fs::read(&path)
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
        if !attempts.insert(record.attempt_id.clone()) || !ids.insert(record.validation_id.clone())
        {
            return Err("Stage 81 独立校验记录重复".to_string());
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn record_fingerprint(
    record: &ControlledShadowExperimentOutputValidationRecord,
) -> Result<String, String> {
    independent_fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    hash_bytes(include_bytes!(
        "historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_output_validations.rs"
    ))
}

fn independent_fingerprint_without<T: Serialize>(
    value: &T,
    fields: &[&str],
) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    independent_hash_serializable(&value)
}

fn independent_hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(hash_bytes(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn f64_hex(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 12
        && value.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-'
        })
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-experiment-output-validations")
}

struct ValidationLock {
    path: PathBuf,
}

impl Drop for ValidationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<ValidationLock, String> {
    let directory = validation_root(state).join("locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{attempt_id}.lock"));
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "另一个 Stage 81 校验正在处理该 attempt".to_string()
            } else {
                error.to_string()
            }
        })?;
    file.sync_all().await.map_err(|error| error.to_string())?;
    Ok(ValidationLock { path })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::{
        HistoricalOutcomeTrainingBoostedStump, HistoricalOutcomeTrainingRidgeCoefficient,
    };

    #[test]
    fn second_preprocessing_path_preserves_missingness_and_frozen_scale() {
        let values = vec![Some(f64_hex(6.0)); EXPECTED_FEATURE_COUNT];
        let preprocessing = (0..EXPECTED_FEATURE_COUNT)
            .map(
                |index| HistoricalOutcomeTrainingFeaturePreprocessingRecord {
                    feature_id: format!("f{index}"),
                    observed_train_count: 10,
                    missing_train_count: 0,
                    train_mean_f64_bits_hex: f64_hex(2.0),
                    train_scale_f64_bits_hex: f64_hex(2.0),
                    missingness_preserved: true,
                },
            )
            .collect::<Vec<_>>();
        assert!(
            independent_preprocess_features(&values, &preprocessing)
                .unwrap()
                .iter()
                .all(|value| *value == Some(2.0))
        );
    }

    #[test]
    fn second_ridge_prediction_path_uses_observed_and_missing_coefficients() {
        let model = HistoricalOutcomeTrainingTargetModel {
            target_id: "target".to_string(),
            model: HistoricalOutcomeTrainingTargetModelBody::RidgeRegression {
                intercept_f64_bits_hex: f64_hex(1.0),
                l2_regularization_f64_bits_hex: f64_hex(0.1),
                coefficients: vec![
                    HistoricalOutcomeTrainingRidgeCoefficient {
                        feature_id: "a".to_string(),
                        observed_value_coefficient_f64_bits_hex: f64_hex(2.0),
                        missing_indicator_coefficient_f64_bits_hex: f64_hex(3.0),
                    },
                    HistoricalOutcomeTrainingRidgeCoefficient {
                        feature_id: "b".to_string(),
                        observed_value_coefficient_f64_bits_hex: f64_hex(4.0),
                        missing_indicator_coefficient_f64_bits_hex: f64_hex(5.0),
                    },
                ],
            },
        };
        assert_eq!(
            independent_predict_target(
                &model,
                &["a".to_string(), "b".to_string()],
                &[Some(2.0), None],
            )
            .unwrap(),
            10.0
        );
    }

    #[test]
    fn second_boosted_prediction_path_handles_low_high_and_missing_leaves() {
        let model = HistoricalOutcomeTrainingTargetModel {
            target_id: "target".to_string(),
            model: HistoricalOutcomeTrainingTargetModelBody::GradientBoostedRegression {
                learning_rate_f64_bits_hex: f64_hex(0.5),
                maximum_tree_depth: 1,
                rounds: vec![HistoricalOutcomeTrainingBoostedStump {
                    feature_id: Some("a".to_string()),
                    threshold_f64_bits_hex: Some(f64_hex(1.0)),
                    low_leaf_f64_bits_hex: f64_hex(2.0),
                    high_leaf_f64_bits_hex: f64_hex(4.0),
                    missing_leaf_f64_bits_hex: f64_hex(6.0),
                    depth: 1,
                }],
            },
        };
        assert_eq!(
            independent_predict_target(&model, &["a".to_string()], &[Some(2.0)]).unwrap(),
            2.0
        );
        assert_eq!(
            independent_predict_target(&model, &["a".to_string()], &[None]).unwrap(),
            3.0
        );
    }

    #[test]
    fn second_weight_path_enforces_all_five_caps() {
        let constraints = ControlledShadowExperimentPortfolioConstraints {
            virtual_notional_usd: 1_000_000,
            long_only: true,
            common_stock_only: true,
            options_allowed: false,
            leverage_allowed: false,
            shorting_allowed: false,
            maximum_single_name_weight_bps: 500,
            maximum_theme_weight_bps: 2_000,
            maximum_gross_exposure_bps: 6_000,
            minimum_cash_weight_bps: 4_000,
            maximum_position_count: 10,
        };
        let ranked = (0..14)
            .map(|index| {
                (
                    if index < 7 { "memory" } else { "optical" }.to_string(),
                    14.0 - index as f64,
                )
            })
            .collect::<Vec<_>>();
        let weights = independent_virtual_weights(&ranked, &constraints);
        assert!(weights.iter().all(|weight| *weight <= 500));
        assert_eq!(weights.iter().take(7).sum::<u32>(), 2_000);
        assert_eq!(weights.iter().skip(7).sum::<u32>(), 1_500);
        assert_eq!(weights.iter().filter(|weight| **weight > 0).count(), 7);
        assert!(weights.iter().sum::<u32>() <= 6_000);
        assert!(10_000 - weights.iter().sum::<u32>() >= 4_000);
    }

    #[test]
    fn input_manifest_tampering_changes_independent_fingerprint() {
        let mut input = ControlledShadowPointInTimeInputEnvelope {
            schema_version: INPUT_SCHEMA_VERSION.to_string(),
            input_manifest_sha256: String::new(),
            candidate_set_sha256: "a".repeat(64),
            feature_order: Vec::new(),
            preprocessing_sha256: "b".repeat(64),
            signal_cutoff_at: Utc::now(),
            captured_at: Utc::now(),
            expected_next_full_market_session_at: Utc::now(),
            benchmark_symbol: "SPY".to_string(),
            benchmark_adjusted_close_f64_bits_hex: f64_hex(500.0),
            sources: Vec::new(),
            rows: Vec::new(),
            point_in_time_read_only: true,
            content_addressed: true,
            allowlisted_sources_only: true,
            no_retroactive_revision: true,
        };
        input.input_manifest_sha256 =
            independent_fingerprint_without(&input, &["input_manifest_sha256"]).unwrap();
        let original = input.input_manifest_sha256.clone();
        input.benchmark_symbol = "QQQ".to_string();
        assert_ne!(
            original,
            independent_fingerprint_without(&input, &["input_manifest_sha256"]).unwrap()
        );
    }

    #[test]
    fn validator_role_must_be_outside_executor_and_prior_chain() {
        assert!(!actor_is_independent(
            "stage-80-executor",
            &[
                "stage-79-reviewer".to_string(),
                "stage-80-executor".to_string()
            ]
        ));
        assert!(actor_is_independent(
            "stage-81-validator",
            &[
                "stage-79-reviewer".to_string(),
                "stage-80-executor".to_string()
            ]
        ));
    }

    #[test]
    fn validator_implementation_identity_is_stable_and_content_addressed() {
        assert!(valid_sha256(&validator_implementation_sha256()));
        assert_ne!(
            validator_implementation_sha256(),
            hash_bytes(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
        );
    }
}
