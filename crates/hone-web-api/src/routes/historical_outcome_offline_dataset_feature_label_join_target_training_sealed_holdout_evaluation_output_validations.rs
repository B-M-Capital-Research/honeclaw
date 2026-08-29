//! Independent, create-once validation of Stage 71 sealed-holdout confirmations.
//!
//! A validator outside the executor and the complete Stage 51–71 actor chain
//! reopens the immutable claim/result, exact consumed authorization, frozen
//! candidate artifacts, independently validated training-store copy and exact
//! raw outcome dataset. It uses the Stage 64 second implementation path rather
//! than any Stage 71 projection, prediction or statistic helper. A pass only
//! makes the confirmation eligible for a future adjudication review; it does
//! not select, store, reward, shadow, order, broker or trade.

use std::collections::{BTreeMap, BTreeSet, HashMap};
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

use super::historical_outcome_offline_dataset_feature_label_join_target_execution_attempts::{
    HistoricalOutcomeFeatureLabelJoinTargetVisibility,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::{
    HistoricalOutcomeTrainingTargetModel, HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    HistoricalOutcomeTrainingUntrustedModelArtifact,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::HistoricalOutcomeTrainingExperimentAlgorithm;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_execution_attempts::{
    CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt,
    HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
    HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus,
    HistoricalOutcomeSealedHoldoutEvaluationMetric,
    HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope,
    completed_sealed_holdout_evaluation_execution_attempts_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_first_execution_authorizations::{
    ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    exact_first_execution_authorization_for_sealed_holdout_evaluation_audit,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementations::HistoricalOutcomeSealedHoldoutEvaluationImplementationContract;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copies::HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_output_validations::independently_validated_training_store_copies_for_training_registration_review;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts::exact_training_attempt as exact_stage57_training_attempt;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_first_execution_authorizations::exact_first_execution_authorization_for_validation_evaluation_audit;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_output_validations::{
    IndependentValidationRow, f64_hex, fingerprint_without, hash_serializable,
    independent_calibration_slope, independent_component_bootstrap,
    independent_directional_accuracy, independent_holm, independent_mae,
    independent_predict_target, independent_preprocess_row, independent_project_targets,
    independent_spearman,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_per_target_candidate_admission_reviews::{
    AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate,
    admitted_validation_evaluation_per_target_candidates_for_sealed_holdout_protocol_review,
};
use super::historical_outcome_offline_dataset_transformation_execution_attempts::HistoricalOutcomeOfflineDatasetTransformationSplit;
use super::historical_outcome_offline_datasets::{
    HistoricalOutcomeOfflineDataset, exact_current_historical_outcome_offline_dataset_for_transformation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-output-validation-v1";
const POLICY_VERSION: &str = "hone-sealed-holdout-output-independent-full-bitwise-recomputation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-sealed-holdout-output-validator-v1-stage64-second-statistics-path";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-execution-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-execution-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str =
    "hone-untrusted-sealed-holdout-one-target-three-seed-confirmation-envelope-v1";
const EXECUTION_POLICY_VERSION: &str = "hone-sealed-holdout-claim-first-one-shot-confirmation-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_METRIC_COUNT: usize = 3;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const NUMERIC_EPSILON: f64 = 1e-12;
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_implementation_sha256: String,
    expected_implementation_review_sha256: String,
    expected_implementation_independent_audit_sha256: String,
    expected_protocol_sha256: String,
    expected_candidate_set_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    expected_sealed_holdout_projection_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    independent_reopen_and_second_implementation_recomputation_confirmed: bool,
    exact_current_stage_51_through_stage_71_binding_confirmed: bool,
    claim_first_authorization_consumption_and_no_replay_confirmed: bool,
    exact_one_target_one_algorithm_three_seed_prediction_recomputation_confirmed: bool,
    exact_three_metrics_component_bootstrap_holm_and_thresholds_bitwise_recomputed_confirmed: bool,
    output_remains_untrusted_pending_future_adjudication_confirmed: bool,
    no_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeSealedHoldoutEvaluationOutputValidationVerdict {
    IndependentlyValidatedUntrustedSealedHoldoutConfirmation,
    FailedIndependentSealedHoldoutOutputValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_review_sha256: String,
    pub implementation_independent_audit_sha256: String,
    pub protocol_sha256: String,
    pub candidate_set_sha256: String,
    pub training_store_dataset_sha256: String,
    pub selected_algorithm_three_seed_binding_sha256: String,
    pub sealed_holdout_split_commitment_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub sealed_holdout_projection_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_execution_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_71_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub envelope_fingerprint_independently_verified: bool,
    pub authorization_consumption_and_no_replay_verified: bool,
    pub exact_training_artifacts_store_dataset_and_contract_verified: bool,
    pub sealed_holdout_projection_independently_reconstructed: bool,
    pub exact_three_candidate_predictions_bitwise_recomputed: bool,
    pub exact_three_metrics_bitwise_recomputed: bool,
    pub exact_three_component_bootstrap_and_holm_tests_bitwise_recomputed: bool,
    pub exact_preregistered_sample_and_threshold_gates_recomputed: bool,
    pub no_selection_or_downstream_authority_verified: bool,
    pub recomputed_claim_sha256: String,
    pub recomputed_result_sha256: String,
    pub recomputed_output_sha256: String,
    pub recomputed_envelope_sha256: String,
    pub recomputed_sealed_holdout_projection_sha256: String,
    pub recomputed_metric_count: usize,
    pub recomputed_candidate_hypothesis_count: usize,
    pub recomputed_all_three_seeds_passed: bool,
    pub recomputed_insufficient_evidence: bool,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeSealedHoldoutEvaluationOutputValidationVerdict,
    pub sealed_holdout_confirmation_independently_validated: bool,
    pub future_confirmatory_result_adjudication_review_eligible: bool,
    pub official_candidate_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationOutputValidationItem {
    pub attempt: CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt,
    pub validation: Option<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_untrusted_confirmation_count: usize,
    pub failed_validation_count: usize,
    pub future_confirmatory_result_adjudication_review_eligible_count: usize,
    pub validation_status: String,
    pub independent_output_validation_available: bool,
    pub official_candidate_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_untrusted_confirmation_count: usize,
    pub failed_validation_count: usize,
    pub future_confirmatory_result_adjudication_review_eligible_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeSealedHoldoutConfirmation {
    pub attempt: CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt,
    pub validation: HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord,
}

#[derive(Debug, Clone)]
struct IndependentHoldoutData {
    feature_order: Vec<String>,
    rows: Vec<IndependentValidationRow>,
    projection_sha256: String,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    envelope_fingerprint: bool,
    authorization_consumption: bool,
    exact_inputs: bool,
    projection_exact: bool,
    predictions_exact: bool,
    metrics_exact: bool,
    hypothesis_tests_exact: bool,
    threshold_gates_exact: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_output_sha256: String,
    recomputed_envelope_sha256: String,
    recomputed_projection_sha256: String,
    recomputed_metric_count: usize,
    recomputed_hypothesis_count: usize,
    recomputed_all_three_seeds_passed: bool,
    recomputed_insufficient_evidence: bool,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_historical_outcome_sealed_holdout_evaluation_output_validations(
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
            warn!(%error, "sealed-holdout output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "sealed-holdout 输出独立验证暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_sealed_holdout_evaluation_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
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

pub(crate) async fn historical_outcome_sealed_holdout_evaluation_output_validation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationOutputValidationReadinessSummary {
            validation_eligible_count: registry.validation_eligible_count,
            validation_count: registry.validation_count,
            independently_validated_untrusted_confirmation_count: registry
                .independently_validated_untrusted_confirmation_count,
            failed_validation_count: registry.failed_validation_count,
            future_confirmatory_result_adjudication_review_eligible_count: registry
                .future_confirmatory_result_adjudication_review_eligible_count,
            validation_status: registry.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_sealed_holdout_confirmations_for_adjudication_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeSealedHoldoutConfirmation>, String> {
    let records = read_validations(state).await?;
    let index = records
        .into_iter()
        .map(|record| (record.attempt_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for attempt in
        completed_sealed_holdout_evaluation_execution_attempts_for_independent_validation(state)
            .await?
    {
        let Some(validation) = index.get(&attempt.claim.attempt_id).cloned() else {
            continue;
        };
        validate_record(&validation)?;
        if validation.sealed_holdout_confirmation_independently_validated
            && validation.future_confirmatory_result_adjudication_review_eligible
            && validation.claim_sha256 == attempt.claim.claim_sha256
            && validation.result_sha256 == attempt.result.result_sha256
        {
            output.push(
                IndependentlyValidatedHistoricalOutcomeSealedHoldoutConfirmation {
                    attempt,
                    validation,
                },
            );
        }
    }
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry, String> {
    let attempts =
        completed_sealed_holdout_evaluation_execution_attempts_for_independent_validation(state)
            .await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let items = attempts
        .into_iter()
        .map(|attempt| {
            let validation = index
                .get(attempt.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeSealedHoldoutEvaluationOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let independently_validated_untrusted_confirmation_count = validations
        .iter()
        .filter(|record| record.sealed_holdout_confirmation_independently_validated)
        .count();
    let failed_validation_count = validations
        .len()
        .saturating_sub(independently_validated_untrusted_confirmation_count);
    let validation_status = if failed_validation_count > 0 {
        "sealed_holdout_output_independent_validation_failed_fail_closed"
    } else if independently_validated_untrusted_confirmation_count > 0 {
        "sealed_holdout_confirmation_independently_validated_waiting_adjudication_review"
    } else if validation_eligible_count > 0 {
        "waiting_independent_sealed_holdout_output_recomputation"
    } else {
        "waiting_completed_sealed_holdout_execution_attempt"
    };
    Ok(HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count: validations.len(),
        independently_validated_untrusted_confirmation_count,
        failed_validation_count,
        future_confirmatory_result_adjudication_review_eligible_count:
            independently_validated_untrusted_confirmation_count,
        validation_status: validation_status.to_string(),
        independent_output_validation_available: true,
        official_candidate_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 72 阶段由 Stage 71 执行者和完整 Stage 51–71 责任链之外的新管理员，使用 Stage 64 第二实现路径重开精确 claim/result、已消费授权、冻结候选、独立验证训练副本与原始结果数据集，重构单目标 sealed-holdout 投影并逐位复算三种子指标。通过只开放未来确认结果裁决复核；正式选模、模型/指标库、奖励、影子、订单、券商和交易继续关闭。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.attempt_id == attempt_id)
    {
        return Err("这次 sealed-holdout 输出已经形成不可变独立验证记录，不得重放".to_string());
    }
    let attempt =
        completed_sealed_holdout_evaluation_execution_attempts_for_independent_validation(state)
            .await?
            .into_iter()
            .find(|value| value.claim.attempt_id == attempt_id)
            .ok_or_else(|| "当前没有完整且待独立验证的 sealed-holdout 执行产物".to_string())?;
    verify_expected_bindings(&attempt, &request)?;
    let authorization = exact_first_execution_authorization_for_sealed_holdout_evaluation_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "sealed-holdout 执行授权完整链已经无法重开".to_string())?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("sealed-holdout 输出验证人不能是执行人或完整上游链任一角色".to_string());
    }
    let admitted = exact_admitted_candidate(state, &authorization).await?;
    let validation_authorization =
        exact_first_execution_authorization_for_validation_evaluation_audit(
            state,
            &admitted.candidate.source.attempt.claim.isolated_runner_id,
            &admitted
                .candidate
                .source
                .attempt
                .claim
                .authorization_review_id,
            &admitted
                .candidate
                .source
                .attempt
                .claim
                .authorization_review_sha256,
        )
        .await?
        .ok_or_else(|| "Stage 63 validation 评估授权链已经无法重开".to_string())?;
    let (_, training_result) =
        exact_stage57_training_attempt(&validation_authorization, state).await?;
    let training_envelope = training_result
        .untrusted_artifact_envelope
        .as_ref()
        .ok_or_else(|| "精确 Stage 57 训练执行缺少冻结候选工件".to_string())?;
    let dataset =
        exact_training_store_dataset(state, &attempt.claim.training_store_dataset_sha256).await?;
    let raw_dataset = exact_current_historical_outcome_offline_dataset_for_transformation(
        state,
        &dataset.dataset_id,
        &dataset.dataset_content_sha256,
        &dataset.dataset_manifest_sha256,
        &dataset.candidate_set_sha256,
    )
    .await?
    .ok_or_else(|| "独立验证器无法重开精确原始结果数据集".to_string())?;
    let audit = independently_audit(
        &attempt,
        &authorization,
        &admitted,
        training_envelope,
        &dataset,
        &raw_dataset,
    )?;
    let passed = independent && audit.mismatch_reasons.is_empty();
    let envelope = attempt
        .result
        .untrusted_confirmation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 71 结果缺少 sealed-holdout confirmation envelope".to_string())?;
    let mut record = HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        output_sha256: attempt.result.output_sha256.clone().unwrap_or_default(),
        authorization_review_id: attempt.claim.authorization_review_id.clone(),
        authorization_review_sha256: attempt.claim.authorization_review_sha256.clone(),
        isolated_runner_id: attempt.claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: attempt.claim.isolated_runner_spec_sha256.clone(),
        implementation_id: attempt.claim.implementation_id.clone(),
        implementation_sha256: attempt.claim.implementation_sha256.clone(),
        implementation_review_sha256: attempt.claim.implementation_review_sha256.clone(),
        implementation_independent_audit_sha256: attempt
            .claim
            .implementation_independent_audit_sha256
            .clone(),
        protocol_sha256: attempt.claim.protocol_sha256.clone(),
        candidate_set_sha256: attempt.claim.candidate_set_sha256.clone(),
        training_store_dataset_sha256: attempt.claim.training_store_dataset_sha256.clone(),
        selected_algorithm_three_seed_binding_sha256: attempt
            .claim
            .selected_algorithm_three_seed_binding_sha256
            .clone(),
        sealed_holdout_split_commitment_sha256: attempt
            .claim
            .sealed_holdout_split_commitment_sha256
            .clone(),
        target_id: attempt.claim.target_id.clone(),
        frozen_candidate_algorithm_id: attempt.claim.frozen_candidate_algorithm_id.clone(),
        sealed_holdout_projection_sha256: envelope.sealed_holdout_projection_sha256.clone(),
        feature_order_sha256: envelope.feature_order_sha256.clone(),
        preprocessing_sha256: envelope.preprocessing_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        invoked_by: attempt.claim.invoked_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_execution_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_71_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        envelope_fingerprint_independently_verified: audit.envelope_fingerprint,
        authorization_consumption_and_no_replay_verified: audit.authorization_consumption,
        exact_training_artifacts_store_dataset_and_contract_verified: audit.exact_inputs,
        sealed_holdout_projection_independently_reconstructed: audit.projection_exact,
        exact_three_candidate_predictions_bitwise_recomputed: audit.predictions_exact,
        exact_three_metrics_bitwise_recomputed: audit.metrics_exact,
        exact_three_component_bootstrap_and_holm_tests_bitwise_recomputed: audit
            .hypothesis_tests_exact,
        exact_preregistered_sample_and_threshold_gates_recomputed: audit.threshold_gates_exact,
        no_selection_or_downstream_authority_verified: audit.downstream_closed,
        recomputed_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_result_sha256: audit.recomputed_result_sha256,
        recomputed_output_sha256: audit.recomputed_output_sha256,
        recomputed_envelope_sha256: audit.recomputed_envelope_sha256,
        recomputed_sealed_holdout_projection_sha256: audit.recomputed_projection_sha256,
        recomputed_metric_count: audit.recomputed_metric_count,
        recomputed_candidate_hypothesis_count: audit.recomputed_hypothesis_count,
        recomputed_all_three_seeds_passed: audit.recomputed_all_three_seeds_passed,
        recomputed_insufficient_evidence: audit.recomputed_insufficient_evidence,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            HistoricalOutcomeSealedHoldoutEvaluationOutputValidationVerdict::IndependentlyValidatedUntrustedSealedHoldoutConfirmation
        } else {
            HistoricalOutcomeSealedHoldoutEvaluationOutputValidationVerdict::FailedIndependentSealedHoldoutOutputValidation
        },
        sealed_holdout_confirmation_independently_validated: passed,
        future_confirmatory_result_adjudication_review_eligible: passed,
        official_candidate_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
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

async fn exact_admitted_candidate(
    state: &AppState,
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
) -> Result<AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate, String> {
    let protocol = &authorization.runner.implementation.upstream_protocol;
    admitted_validation_evaluation_per_target_candidates_for_sealed_holdout_protocol_review(state)
        .await?
        .into_iter()
        .find(|value| {
            value.candidate.source.attempt.claim.attempt_id == protocol.attempt_id
                && value.candidate.target_id == protocol.target_id
                && value.admission_review.review_id == protocol.stage_65_admission_review_id
                && value.admission_review.review_sha256 == protocol.stage_65_admission_review_sha256
                && value.candidate.target_bundle_sha256 == protocol.target_bundle_sha256
                && value.candidate.recommendation_sha256 == protocol.recommendation_sha256
        })
        .ok_or_else(|| "找不到 Stage 66 协议精确绑定的 Stage 65 逐目标准入记录".to_string())
}

async fn exact_training_store_dataset(
    state: &AppState,
    expected_sha256: &str,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset, String> {
    independently_validated_training_store_copies_for_training_registration_review(state)
        .await?
        .into_iter()
        .filter_map(|value| value.copied_dataset.attempt.training_store_dataset)
        .find(|dataset| dataset.training_store_dataset_sha256 == expected_sha256)
        .ok_or_else(|| "精确绑定的独立验证训练副本已经不可用".to_string())
}

fn independently_audit(
    attempt: &CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    admitted: &AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    raw_dataset: &HistoricalOutcomeOfflineDataset,
) -> Result<IndependentAudit, String> {
    let claim = &attempt.claim;
    let result = &attempt.result;
    let envelope = result
        .untrusted_confirmation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 71 结果缺少 confirmation envelope".to_string())?;
    let contract = &authorization.runner.implementation.implementation_contract;
    let protocol = &authorization.runner.implementation.upstream_protocol;
    let recomputed_claim_sha256 = fingerprint_without(claim, &["attempt_id", "claim_sha256"])?;
    let recomputed_result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    let recomputed_output_sha256 = hash_serializable(envelope)?;
    let exact_chain = claim.authorization_review_id == authorization.review.review_id
        && claim.authorization_review_sha256 == authorization.review.review_sha256
        && claim.isolated_runner_id == authorization.runner.isolated_runner_id
        && claim.isolated_runner_spec_sha256 == authorization.runner.isolated_runner_spec_sha256
        && claim.runner_artifact_sha256 == authorization.runner.runner_artifact_sha256
        && claim.runner_code_revision == authorization.runner.runner_code_revision
        && claim.runner_contract_sha256 == authorization.runner.runner_contract.contract_sha256
        && claim.implementation_id == authorization.runner.implementation.implementation_id
        && claim.implementation_sha256 == authorization.runner.implementation.implementation_sha256
        && claim.implementation_contract_sha256
            == authorization
                .runner
                .implementation
                .implementation_contract
                .contract_sha256
        && claim.implementation_review_id == authorization.runner.implementation_review.review_id
        && claim.implementation_review_sha256
            == authorization.runner.implementation_review.review_sha256
        && claim.implementation_independent_audit_sha256
            == authorization
                .runner
                .implementation_review
                .independent_audit
                .audit_sha256
        && claim.protocol_sha256 == protocol.protocol_sha256
        && claim.target_id == protocol.target_id
        && claim.target_id == admitted.candidate.target_id
        && claim.selected_algorithm_three_seed_binding_sha256
            == protocol.selected_algorithm_three_seed_binding_sha256
        && claim.sealed_holdout_split_commitment_sha256
            == protocol.sealed_holdout_split_commitment_sha256;
    let claim_fingerprint = claim.schema_version == CLAIM_SCHEMA_VERSION
        && claim.execution_policy_version == EXECUTION_POLICY_VERSION
        && claim.claim_sha256 == recomputed_claim_sha256
        && claim.attempt_id == recomputed_claim_sha256[..32]
        && claim.authorization_consumed
        && claim.invocation_started
        && claim.sealed_holdout_feature_read_allowed
        && claim.sealed_holdout_label_read_allowed
        && claim.evaluation_authorized
        && claim.ephemeral_output_write_allowed
        && !claim.validation_or_train_read_allowed
        && !claim.other_target_read_allowed
        && !claim.training_or_preprocessing_update_allowed
        && !claim.candidate_reselection_allowed
        && !claim.official_candidate_selection_allowed
        && !claim.model_store_write_allowed
        && !claim.metric_store_write_allowed
        && !claim.ambient_environment_available
        && !claim.network_available
        && !claim.external_tools_available
        && !claim.child_process_allowed
        && !claim.production_read_allowed
        && !claim.production_write_allowed
        && !claim.historical_state_mutation_allowed
        && !claim.reward_allowed
        && !claim.shadow_portfolio_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed;
    let result_fingerprint = result.schema_version == RESULT_SCHEMA_VERSION
        && result.execution_policy_version == EXECUTION_POLICY_VERSION
        && result.result_sha256 == recomputed_result_sha256
        && result.result_id == recomputed_result_sha256[..32]
        && result.attempt_id == claim.attempt_id
        && result.claim_sha256 == claim.claim_sha256
        && result.status
            == HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus::CompletedWithUntrustedSealedHoldoutConfirmation
        && result.exit_code == 0
        && result.output_sha256.as_deref() == Some(recomputed_output_sha256.as_str())
        && result.output_bytes
            == serde_json::to_vec(envelope)
                .map_err(|error| error.to_string())?
                .len()
        && result.bounded_error.is_none()
        && result.ephemeral_directory_removed
        && result.sealed_holdout_features_accessed
        && result.sealed_holdout_labels_accessed
        && result.evaluation_completed
        && !result.independent_output_validation_completed
        && !result.official_candidate_selection_completed
        && !result.model_artifact_store_written
        && !result.metric_store_written
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    let envelope_fingerprint = envelope.schema_version == ENVELOPE_SCHEMA_VERSION
        && envelope.authorization_review_id == claim.authorization_review_id
        && envelope.authorization_review_sha256 == claim.authorization_review_sha256
        && envelope.isolated_runner_id == claim.isolated_runner_id
        && envelope.isolated_runner_spec_sha256 == claim.isolated_runner_spec_sha256
        && envelope.implementation_id == claim.implementation_id
        && envelope.implementation_sha256 == claim.implementation_sha256
        && envelope.implementation_contract_sha256 == claim.implementation_contract_sha256
        && envelope.implementation_review_sha256 == claim.implementation_review_sha256
        && envelope.implementation_independent_audit_sha256
            == claim.implementation_independent_audit_sha256
        && envelope.protocol_sha256 == claim.protocol_sha256
        && envelope.candidate_set_sha256 == claim.candidate_set_sha256
        && envelope.training_store_dataset_sha256 == claim.training_store_dataset_sha256
        && envelope.rows_sha256 == claim.rows_sha256
        && envelope.target_commitments_sha256 == claim.target_commitments_sha256
        && envelope.selected_algorithm_three_seed_binding_sha256
            == claim.selected_algorithm_three_seed_binding_sha256
        && envelope.sealed_holdout_split_commitment_sha256
            == claim.sealed_holdout_split_commitment_sha256
        && envelope.target_id == claim.target_id
        && envelope.frozen_candidate_algorithm_id == claim.frozen_candidate_algorithm_id;
    let authorization_consumption = claim.authorization_consumed
        && claim.claim_first_single_use_and_failure_consumes_confirmed
        && claim.claimed_at <= result.completed_at
        && claim.claimed_at <= claim.authorization_valid_until
        && claim.authorization_valid_until == authorization.review.authorization_valid_until;
    let recomputed_rows_sha256 = hash_serializable(&dataset.rows)?;
    let mut commitments = dataset
        .rows
        .iter()
        .map(|row| {
            (
                row.dataset_entry_id.clone(),
                row.target_commitment_sha256.clone(),
            )
        })
        .collect::<Vec<_>>();
    commitments.sort();
    let exact_inputs = dataset.training_store_dataset_sha256
        == fingerprint_without(dataset, &["training_store_dataset_sha256"])?
        && dataset.training_store_dataset_sha256 == claim.training_store_dataset_sha256
        && recomputed_rows_sha256 == claim.rows_sha256
        && hash_serializable(&commitments)? == claim.target_commitments_sha256
        && dataset.candidate_set_sha256 == claim.candidate_set_sha256
        && training_envelope.training_store_dataset_sha256 == claim.training_store_dataset_sha256
        && training_envelope.model_artifacts.len() == 9
        && training_envelope.feature_order.len() == EXPECTED_FEATURE_COUNT
        && training_envelope.preprocessing.len() == EXPECTED_FEATURE_COUNT
        && contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == 9
        && contract.exact_candidate_hypothesis_count == EXPECTED_METRIC_COUNT as u32
        && contract.exact_random_seeds == EXPECTED_SEEDS;
    let data = independently_prepare_holdout_projection(
        dataset,
        raw_dataset,
        training_envelope,
        contract,
    )?;
    let recomputed =
        independently_recompute_envelope(claim, training_envelope, contract, data.clone())?;
    let recomputed_envelope_sha256 = hash_serializable(&recomputed)?;
    let projection_exact = envelope.sealed_holdout_projection_sha256 == data.projection_sha256
        && envelope.sealed_holdout_row_count == data.rows.len()
        && envelope.feature_order_sha256 == hash_serializable(&data.feature_order)?
        && envelope.preprocessing_sha256 == hash_serializable(&training_envelope.preprocessing)?;
    let predictions_exact = recomputed.candidate_artifact_sha256s
        == envelope.candidate_artifact_sha256s
        && recomputed.candidate_artifact_sha256s.len() == EXPECTED_SEEDS.len();
    let metrics_exact = recomputed_output_sha256 == recomputed_envelope_sha256
        && envelope.metrics == recomputed.metrics
        && recomputed.metrics.len() == EXPECTED_METRIC_COUNT;
    let hypothesis_tests_exact =
        envelope
            .metrics
            .iter()
            .zip(&recomputed.metrics)
            .all(|(left, right)| {
                left.component_block_bootstrap_p_value_f64_bits_hex
                    == right.component_block_bootstrap_p_value_f64_bits_hex
                    && left.holm_adjusted_p_value_f64_bits_hex
                        == right.holm_adjusted_p_value_f64_bits_hex
            });
    let threshold_gates_exact = envelope.all_three_seeds_passed
        == recomputed.all_three_seeds_passed
        && envelope.insufficient_evidence == recomputed.insufficient_evidence
        && envelope.confirmation_status == recomputed.confirmation_status
        && envelope
            .metrics
            .iter()
            .zip(&recomputed.metrics)
            .all(|(left, right)| {
                left.evidence_status == right.evidence_status
                    && left.all_preregistered_thresholds_passed
                        == right.all_preregistered_thresholds_passed
            });
    let downstream_closed = envelope.output_is_untrusted
        && !envelope.independent_output_validation_completed
        && !envelope.official_candidate_selection_completed
        && !envelope.feedback_reuse_allowed
        && !envelope.training_or_preprocessing_updated
        && !envelope.composite_score_created
        && !envelope.global_model_validity_claimed
        && !envelope.model_artifact_store_written
        && !envelope.metric_store_written
        && !envelope.scalar_reward_written
        && !envelope.shadow_position_written
        && !envelope.order_generated
        && !envelope.broker_accessed
        && !envelope.trade_executed;
    let mut mismatch_reasons = Vec::new();
    for (matched, reason) in [
        (exact_chain, "stage_51_through_stage_71_chain_mismatch"),
        (
            claim_fingerprint,
            "sealed_holdout_claim_fingerprint_or_capability_mismatch",
        ),
        (
            result_fingerprint,
            "sealed_holdout_result_fingerprint_or_shape_mismatch",
        ),
        (
            envelope_fingerprint,
            "sealed_holdout_envelope_binding_mismatch",
        ),
        (
            authorization_consumption,
            "authorization_consumption_or_no_replay_mismatch",
        ),
        (
            exact_inputs,
            "training_artifacts_dataset_or_frozen_contract_mismatch",
        ),
        (
            projection_exact,
            "sealed_holdout_projection_bitwise_mismatch",
        ),
        (
            predictions_exact,
            "three_candidate_prediction_recomputation_failed",
        ),
        (
            metrics_exact,
            "three_sealed_holdout_metrics_bitwise_mismatch",
        ),
        (
            hypothesis_tests_exact,
            "component_bootstrap_or_holm_bitwise_mismatch",
        ),
        (
            threshold_gates_exact,
            "preregistered_sample_or_threshold_gate_mismatch",
        ),
        (
            downstream_closed,
            "candidate_selection_or_downstream_authority_overreach",
        ),
    ] {
        if !matched {
            mismatch_reasons.push(reason.to_string());
        }
    }
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    mismatch_reasons.truncate(MAX_MISMATCH_REASONS);
    Ok(IndependentAudit {
        exact_chain,
        claim_fingerprint,
        result_fingerprint,
        envelope_fingerprint,
        authorization_consumption,
        exact_inputs,
        projection_exact,
        predictions_exact,
        metrics_exact,
        hypothesis_tests_exact,
        threshold_gates_exact,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_output_sha256,
        recomputed_envelope_sha256,
        recomputed_projection_sha256: data.projection_sha256,
        recomputed_metric_count: recomputed.metrics.len(),
        recomputed_hypothesis_count: recomputed.metrics.len(),
        recomputed_all_three_seeds_passed: recomputed.all_three_seeds_passed,
        recomputed_insufficient_evidence: recomputed.insufficient_evidence,
        mismatch_reasons,
    })
}

fn independently_prepare_holdout_projection(
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    raw_dataset: &HistoricalOutcomeOfflineDataset,
    envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
) -> Result<IndependentHoldoutData, String> {
    if raw_dataset.dataset_id != dataset.dataset_id
        || raw_dataset.dataset_content_sha256 != dataset.dataset_content_sha256
        || raw_dataset.manifest_sha256 != dataset.dataset_manifest_sha256
        || raw_dataset.candidate_set_sha256 != dataset.candidate_set_sha256
        || envelope.feature_order.len() != EXPECTED_FEATURE_COUNT
        || envelope.preprocessing.len() != EXPECTED_FEATURE_COUNT
        || hash_serializable(&envelope.feature_order)? != contract.feature_order_sha256
        || hash_serializable(&envelope.preprocessing)? != contract.preprocessing_sha256
    {
        return Err("独立 sealed-holdout 投影的数据集、特征或预处理绑定漂移".to_string());
    }
    let mut holdout_rows = dataset
        .rows
        .iter()
        .filter(|row| {
            row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
                && row.split == HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
                && row.target_vector.is_none()
        })
        .collect::<Vec<_>>();
    holdout_rows.sort_by(|left, right| left.dataset_entry_id.cmp(&right.dataset_entry_id));
    if holdout_rows.is_empty() {
        return Err("独立验证没有找到精确 sealed-holdout 行".to_string());
    }
    let allowed_ids = holdout_rows
        .iter()
        .map(|row| row.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    let non_holdout_ids = dataset
        .rows
        .iter()
        .filter(|row| {
            row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
        })
        .map(|row| row.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    if !allowed_ids.is_disjoint(&non_holdout_ids) {
        return Err("独立验证发现 sealed-holdout 与训练/验证 entry 重叠".to_string());
    }
    let raw_index = raw_dataset
        .entries
        .iter()
        .filter(|entry| allowed_ids.contains(entry.entry_id.as_str()))
        .map(|entry| (entry.entry_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    if raw_index.len() != allowed_ids.len() {
        return Err("独立验证无法重开全部 sealed-holdout 原始结果".to_string());
    }
    let preprocessing = envelope
        .preprocessing
        .iter()
        .map(|value| (value.feature_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let target_order = vec![contract.target_id.clone()];
    let mut rows = Vec::with_capacity(holdout_rows.len());
    for row in holdout_rows {
        let raw = raw_index
            .get(row.dataset_entry_id.as_str())
            .ok_or_else(|| "独立 sealed-holdout 行缺少原始结果".to_string())?;
        if raw.entry_sha256 != row.dataset_entry_sha256
            || raw.decision_available_at != row.decision_available_at
        {
            return Err(format!(
                "sealed-holdout 条目 {} 点时绑定漂移",
                row.dataset_entry_id
            ));
        }
        rows.push(IndependentValidationRow {
            dataset_entry_id: row.dataset_entry_id.clone(),
            component_id: row.component_id.clone(),
            features: independent_preprocess_row(row, &envelope.feature_order, &preprocessing)?,
            targets: independent_project_targets(raw, &target_order)?,
        });
    }
    let projection_sha256 = hash_serializable(
        &rows
            .iter()
            .map(|row| {
                (
                    &row.dataset_entry_id,
                    &row.component_id,
                    row.features
                        .iter()
                        .map(|value| value.map(f64::to_bits))
                        .collect::<Vec<_>>(),
                    row.targets
                        .iter()
                        .map(|value| value.to_bits())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>(),
    )?;
    Ok(IndependentHoldoutData {
        feature_order: envelope.feature_order.clone(),
        rows,
        projection_sha256,
    })
}

fn independently_recompute_envelope(
    claim: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
    data: IndependentHoldoutData,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope, String> {
    validate_independent_frozen_contract(contract, training_envelope)?;
    let algorithm = parse_algorithm_id(&contract.frozen_candidate_algorithm_id)?;
    let components = data
        .rows
        .iter()
        .map(|row| row.component_id.clone())
        .collect::<BTreeSet<_>>();
    let actual = data
        .rows
        .iter()
        .map(|row| row.targets[0])
        .collect::<Vec<_>>();
    let zero = vec![0.0; actual.len()];
    let zero_mae = independent_mae(&actual, &zero)?;
    let feature_positions = data
        .feature_order
        .iter()
        .enumerate()
        .map(|(index, value)| (value.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut candidate_artifact_sha256s = Vec::new();
    let mut provisional = Vec::new();
    let mut p_values = Vec::new();
    for seed in EXPECTED_SEEDS {
        let artifact = exact_candidate_artifact(training_envelope, &algorithm, seed)?;
        let model = exact_target_model(artifact, &contract.target_id)?;
        let predicted = data
            .rows
            .iter()
            .map(|row| independent_predict_target(model, &feature_positions, &row.features))
            .collect::<Result<Vec<_>, _>>()?;
        let mae = independent_mae(&actual, &predicted)?;
        let improvement = if zero_mae.abs() <= NUMERIC_EPSILON {
            0.0
        } else {
            (zero_mae - mae) / zero_mae
        };
        let p = independent_component_bootstrap(
            &data.rows,
            &actual,
            &predicted,
            &zero,
            contract.bootstrap_replications,
            contract.bootstrap_random_seed ^ seed,
        )?;
        candidate_artifact_sha256s.push(artifact.artifact_sha256.clone());
        p_values.push(p);
        provisional.push((
            seed,
            mae,
            improvement,
            p,
            independent_spearman(&actual, &predicted),
            independent_directional_accuracy(&actual, &predicted),
            independent_calibration_slope(&actual, &predicted),
        ));
    }
    let adjusted = independent_holm(&p_values);
    let insufficient = data.rows.len() < contract.minimum_sealed_holdout_rows as usize
        || components.len() < contract.minimum_independent_components as usize;
    let mut metrics = Vec::with_capacity(EXPECTED_METRIC_COUNT);
    for ((seed, mae, improvement, p, rho, directional, calibration), q) in
        provisional.into_iter().zip(adjusted)
    {
        let passed = !insufficient
            && independent_metric_passes(contract, improvement, q, rho, directional, calibration);
        metrics.push(HistoricalOutcomeSealedHoldoutEvaluationMetric {
            algorithm: algorithm.clone(),
            random_seed: seed,
            target_id: contract.target_id.clone(),
            sealed_holdout_row_count: data.rows.len(),
            independent_component_count: components.len(),
            mae_f64_bits_hex: f64_hex(mae),
            zero_baseline_mae_f64_bits_hex: f64_hex(zero_mae),
            relative_mae_improvement_f64_bits_hex: f64_hex(improvement),
            component_block_bootstrap_p_value_f64_bits_hex: f64_hex(p),
            holm_adjusted_p_value_f64_bits_hex: f64_hex(q),
            spearman_f64_bits_hex: rho.map(f64_hex),
            directional_accuracy_f64_bits_hex: f64_hex(directional),
            calibration_slope_f64_bits_hex: calibration.map(f64_hex),
            evidence_status: if insufficient {
                "insufficient_evidence"
            } else if passed {
                "all_preregistered_thresholds_passed"
            } else {
                "one_or_more_preregistered_thresholds_failed"
            }
            .to_string(),
            all_preregistered_thresholds_passed: passed,
            official_model_selection_metric: false,
        });
    }
    let all_three_seeds_passed = !insufficient
        && metrics
            .iter()
            .all(|metric| metric.all_preregistered_thresholds_passed);
    let confirmation_status = if insufficient {
        "insufficient_evidence_fail_closed"
    } else if all_three_seeds_passed {
        "untrusted_confirmatory_pass_pending_independent_validation"
    } else {
        "untrusted_confirmatory_fail_pending_independent_validation"
    }
    .to_string();
    Ok(HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_string(),
        authorization_review_id: claim.authorization_review_id.clone(),
        authorization_review_sha256: claim.authorization_review_sha256.clone(),
        isolated_runner_id: claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256.clone(),
        implementation_id: claim.implementation_id.clone(),
        implementation_sha256: claim.implementation_sha256.clone(),
        implementation_contract_sha256: claim.implementation_contract_sha256.clone(),
        implementation_review_sha256: claim.implementation_review_sha256.clone(),
        implementation_independent_audit_sha256: claim
            .implementation_independent_audit_sha256
            .clone(),
        protocol_sha256: claim.protocol_sha256.clone(),
        candidate_set_sha256: claim.candidate_set_sha256.clone(),
        training_store_dataset_sha256: claim.training_store_dataset_sha256.clone(),
        rows_sha256: claim.rows_sha256.clone(),
        target_commitments_sha256: claim.target_commitments_sha256.clone(),
        selected_algorithm_three_seed_binding_sha256: claim
            .selected_algorithm_three_seed_binding_sha256
            .clone(),
        sealed_holdout_split_commitment_sha256: claim
            .sealed_holdout_split_commitment_sha256
            .clone(),
        feature_order_sha256: hash_serializable(&data.feature_order)?,
        preprocessing_sha256: hash_serializable(&training_envelope.preprocessing)?,
        sealed_holdout_projection_sha256: data.projection_sha256,
        target_id: claim.target_id.clone(),
        frozen_candidate_algorithm_id: claim.frozen_candidate_algorithm_id.clone(),
        exact_random_seeds: EXPECTED_SEEDS.to_vec(),
        candidate_artifact_sha256s,
        sealed_holdout_row_count: data.rows.len(),
        independent_component_count: components.len(),
        exact_metric_count: metrics.len(),
        exact_candidate_hypothesis_count: metrics.len(),
        metrics,
        confirmation_status,
        all_three_seeds_passed,
        insufficient_evidence: insufficient,
        sealed_holdout_features_accessed: true,
        sealed_holdout_labels_accessed: true,
        one_target_only: true,
        one_algorithm_only: true,
        training_or_preprocessing_updated: false,
        feedback_reuse_allowed: false,
        output_is_untrusted: true,
        independent_output_validation_completed: false,
        official_candidate_selection_completed: false,
        composite_score_created: false,
        global_model_validity_claimed: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        scalar_reward_written: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    })
}

fn validate_independent_frozen_contract(
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
    envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
) -> Result<(), String> {
    let exact = contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == 9
        && contract.exact_candidate_hypothesis_count == EXPECTED_METRIC_COUNT as u32
        && contract.exact_random_seeds == EXPECTED_SEEDS
        && contract.bootstrap_unit == "official_split_component_id"
        && contract.bootstrap_replications == 10_000
        && contract.family_wise_error_correction
            == "holm_bonferroni_across_exact_three_frozen_seed_tests_for_one_target"
        && contract.all_three_seeds_must_pass
        && contract.one_shot_evaluation_required
        && contract.no_feedback_reuse
        && contract.insufficient_sample_fails_closed
        && contract.no_composite_score_or_cross_target_masking
        && !contract.training_or_preprocessing_update_allowed
        && !contract.hyperparameter_or_threshold_tuning_allowed
        && !contract.candidate_reselection_allowed
        && !contract.official_candidate_selection_allowed
        && hash_serializable(&envelope.feature_order)? == contract.feature_order_sha256
        && hash_serializable(&envelope.preprocessing)? == contract.preprocessing_sha256;
    if !exact {
        return Err("独立验证发现 Stage 66–70 冻结统计合同漂移".to_string());
    }
    Ok(())
}

fn parse_algorithm_id(value: &str) -> Result<HistoricalOutcomeTrainingExperimentAlgorithm, String> {
    match value {
        "ridge_multi_target_regression" => {
            Ok(HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression)
        }
        "gradient_boosted_multi_target_regression" => {
            Ok(HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression)
        }
        _ => Err("独立验证不接受未准入的候选算法".to_string()),
    }
}

fn exact_candidate_artifact<'a>(
    envelope: &'a HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    algorithm: &HistoricalOutcomeTrainingExperimentAlgorithm,
    seed: u64,
) -> Result<&'a HistoricalOutcomeTrainingUntrustedModelArtifact, String> {
    let values = envelope
        .model_artifacts
        .iter()
        .filter(|artifact| {
            &artifact.algorithm == algorithm
                && artifact.random_seed == seed
                && !artifact.validation_selected
                && !artifact.sealed_holdout_accessed
        })
        .collect::<Vec<_>>();
    if values.len() != 1 {
        return Err(format!("独立验证要求算法种子 {seed} 精确一个冻结工件"));
    }
    Ok(values[0])
}

fn exact_target_model<'a>(
    artifact: &'a HistoricalOutcomeTrainingUntrustedModelArtifact,
    target_id: &str,
) -> Result<&'a HistoricalOutcomeTrainingTargetModel, String> {
    let values = artifact
        .target_models
        .iter()
        .filter(|model| model.target_id == target_id)
        .collect::<Vec<_>>();
    if values.len() != 1 {
        return Err(format!("独立验证要求精确一个目标模型 {target_id}"));
    }
    Ok(values[0])
}

fn independent_metric_passes(
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
    improvement: f64,
    q: f64,
    rho: Option<f64>,
    directional: f64,
    calibration: Option<f64>,
) -> bool {
    improvement >= f64::from(contract.minimum_relative_mae_improvement_ppm) / 1_000_000.0
        && q <= f64::from(contract.family_wise_alpha_millionths) / 1_000_000.0
        && rho.is_some_and(|value| {
            value >= f64::from(contract.minimum_spearman_millionths) / 1_000_000.0
        })
        && directional >= f64::from(contract.minimum_directional_accuracy_millionths) / 1_000_000.0
        && calibration.is_some_and(|value| {
            value >= f64::from(contract.minimum_calibration_slope_millionths) / 1_000_000.0
                && value <= f64::from(contract.maximum_calibration_slope_millionths) / 1_000_000.0
        })
}

fn validate_request(
    attempt_id: &str,
    request: &ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest,
) -> Result<(), String> {
    let hashes = [
        request.expected_claim_sha256.as_str(),
        request.expected_result_sha256.as_str(),
        request.expected_output_sha256.as_str(),
        request.expected_authorization_review_sha256.as_str(),
        request.expected_isolated_runner_spec_sha256.as_str(),
        request.expected_implementation_sha256.as_str(),
        request.expected_implementation_review_sha256.as_str(),
        request
            .expected_implementation_independent_audit_sha256
            .as_str(),
        request.expected_protocol_sha256.as_str(),
        request.expected_candidate_set_sha256.as_str(),
        request.expected_training_store_dataset_sha256.as_str(),
        request
            .expected_selected_algorithm_three_seed_binding_sha256
            .as_str(),
        request
            .expected_sealed_holdout_split_commitment_sha256
            .as_str(),
        request.expected_sealed_holdout_projection_sha256.as_str(),
        request.expected_feature_order_sha256.as_str(),
        request.expected_preprocessing_sha256.as_str(),
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
        return Err("sealed-holdout 输出验证请求的 ID、SHA-256、目标或算法无效".to_string());
    }
    if !request.independent_reopen_and_second_implementation_recomputation_confirmed
        || !request.exact_current_stage_51_through_stage_71_binding_confirmed
        || !request.claim_first_authorization_consumption_and_no_replay_confirmed
        || !request.exact_one_target_one_algorithm_three_seed_prediction_recomputation_confirmed
        || !request
            .exact_three_metrics_component_bootstrap_holm_and_thresholds_bitwise_recomputed_confirmed
        || !request.output_remains_untrusted_pending_future_adjudication_confirmed
        || !request.no_selection_store_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err("必须确认独立复算、完整链、单次授权、三种子统计和零下游权限".to_string());
    }
    Ok(())
}

fn verify_expected_bindings(
    attempt: &CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt,
    request: &ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest,
) -> Result<(), String> {
    let claim = &attempt.claim;
    let result = &attempt.result;
    let envelope = result
        .untrusted_confirmation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 71 结果缺少 confirmation envelope".to_string())?;
    if claim.claim_sha256 != request.expected_claim_sha256
        || result.result_sha256 != request.expected_result_sha256
        || result.output_sha256.as_deref() != Some(request.expected_output_sha256.as_str())
        || claim.authorization_review_sha256 != request.expected_authorization_review_sha256
        || claim.isolated_runner_spec_sha256 != request.expected_isolated_runner_spec_sha256
        || claim.implementation_sha256 != request.expected_implementation_sha256
        || claim.implementation_review_sha256 != request.expected_implementation_review_sha256
        || claim.implementation_independent_audit_sha256
            != request.expected_implementation_independent_audit_sha256
        || claim.protocol_sha256 != request.expected_protocol_sha256
        || claim.candidate_set_sha256 != request.expected_candidate_set_sha256
        || claim.training_store_dataset_sha256 != request.expected_training_store_dataset_sha256
        || claim.selected_algorithm_three_seed_binding_sha256
            != request.expected_selected_algorithm_three_seed_binding_sha256
        || claim.sealed_holdout_split_commitment_sha256
            != request.expected_sealed_holdout_split_commitment_sha256
        || claim.target_id != request.expected_target_id
        || claim.frozen_candidate_algorithm_id != request.expected_frozen_candidate_algorithm_id
        || envelope.sealed_holdout_projection_sha256
            != request.expected_sealed_holdout_projection_sha256
        || envelope.feature_order_sha256 != request.expected_feature_order_sha256
        || envelope.preprocessing_sha256 != request.expected_preprocessing_sha256
    {
        return Err("sealed-holdout claim/result、输出或冻结绑定已变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn prior_actor_ids(
    attempt: &CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
) -> Vec<String> {
    let mut values = authorization.review.excluded_prior_actor_ids.clone();
    values.extend(authorization.runner.excluded_prior_actor_ids.clone());
    values.extend(
        authorization
            .runner
            .implementation_review
            .excluded_prior_actor_ids
            .clone(),
    );
    values.extend(
        authorization
            .runner
            .implementation
            .excluded_prior_actor_ids
            .clone(),
    );
    values.push(attempt.claim.invoked_by.clone());
    values.push(authorization.review.reviewer_id.clone());
    values.push(authorization.runner.registered_by.clone());
    values.push(
        authorization
            .runner
            .implementation_review
            .reviewer_id
            .clone(),
    );
    values.push(authorization.runner.implementation.registered_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn validate_record(
    record: &HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_execution_and_complete_prior_chain
        && record.exact_current_stage_51_through_stage_71_chain_verified
        && record.claim_fingerprint_independently_verified
        && record.result_fingerprint_independently_verified
        && record.envelope_fingerprint_independently_verified
        && record.authorization_consumption_and_no_replay_verified
        && record.exact_training_artifacts_store_dataset_and_contract_verified
        && record.sealed_holdout_projection_independently_reconstructed
        && record.exact_three_candidate_predictions_bitwise_recomputed
        && record.exact_three_metrics_bitwise_recomputed
        && record.exact_three_component_bootstrap_and_holm_tests_bitwise_recomputed
        && record.exact_preregistered_sample_and_threshold_gates_recomputed
        && record.no_selection_or_downstream_authority_verified
        && record.recomputed_metric_count == EXPECTED_METRIC_COUNT
        && record.recomputed_candidate_hypothesis_count == EXPECTED_METRIC_COUNT
        && record.mismatch_reasons.is_empty();
    let closed = !record.official_candidate_selection_authorized
        && !record.model_artifact_store_written
        && !record.metric_store_written
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let hashes = [
        record.validation_sha256.as_str(),
        record.claim_sha256.as_str(),
        record.result_sha256.as_str(),
        record.output_sha256.as_str(),
        record.authorization_review_sha256.as_str(),
        record.isolated_runner_spec_sha256.as_str(),
        record.implementation_sha256.as_str(),
        record.implementation_review_sha256.as_str(),
        record.implementation_independent_audit_sha256.as_str(),
        record.protocol_sha256.as_str(),
        record.candidate_set_sha256.as_str(),
        record.training_store_dataset_sha256.as_str(),
        record.selected_algorithm_three_seed_binding_sha256.as_str(),
        record.sealed_holdout_split_commitment_sha256.as_str(),
        record.sealed_holdout_projection_sha256.as_str(),
        record.feature_order_sha256.as_str(),
        record.preprocessing_sha256.as_str(),
        record.validator_implementation_sha256.as_str(),
        record.recomputed_claim_sha256.as_str(),
        record.recomputed_result_sha256.as_str(),
        record.recomputed_output_sha256.as_str(),
        record.recomputed_envelope_sha256.as_str(),
        record.recomputed_sealed_holdout_projection_sha256.as_str(),
    ];
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.validation_id)
        || hashes.iter().any(|value| !valid_sha256(value))
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != record_fingerprint(record)?
        || !valid_id(&record.attempt_id)
        || !valid_id(&record.result_id)
        || !valid_id(&record.authorization_review_id)
        || !valid_id(&record.isolated_runner_id)
        || !valid_id(&record.implementation_id)
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || record.validated_by.trim().is_empty()
        || record.invoked_by.trim().is_empty()
        || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || !actor_is_independent(&record.validated_by, &record.excluded_prior_actor_ids)
        || !record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.invoked_by)
        || record.mismatch_reasons.len() > MAX_MISMATCH_REASONS
        || record.sealed_holdout_confirmation_independently_validated != passed
        || record.future_confirmatory_result_adjudication_review_eligible != passed
        || record.sealed_holdout_confirmation_independently_validated
            != (record.verdict
                == HistoricalOutcomeSealedHoldoutEvaluationOutputValidationVerdict::IndependentlyValidatedUntrustedSealedHoldoutConfirmation)
        || !closed
    {
        return Err("sealed-holdout output validation record is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord>, String> {
    let root = validation_root(state);
    let mut directories = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    let mut attempts = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
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
        let directory_name = directory.file_name().to_string_lossy().to_string();
        if !valid_id(&directory_name) {
            return Err("sealed-holdout 输出验证目录包含无效 attempt ID".to_string());
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
            let record = serde_json::from_slice::<
                HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord,
            >(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            validate_record(&record)?;
            if directory_name != record.attempt_id
                || entry.path().file_stem().and_then(|value| value.to_str())
                    != Some(record.validation_id.as_str())
                || !attempts.insert(record.attempt_id.clone())
                || !ids.insert(record.validation_id.clone())
                || !hashes.insert(record.validation_sha256.clone())
            {
                return Err("sealed-holdout 输出独立验证含重放、误命名或重复哈希".to_string());
            }
            records.push(record);
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    hash_bytes(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
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
    decision_root(state).join("sealed-holdout-evaluation-output-validations")
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
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "sealed-holdout-evaluation-output-validation-{attempt_id}.lock"
    ));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age > StdDuration::from_secs(300));
        if stale {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 sealed-holdout 输出独立验证正在进行".to_string())?;
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

    fn request() -> ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest {
        ValidateHistoricalOutcomeSealedHoldoutEvaluationOutputRequest {
            expected_claim_sha256: "a".repeat(64),
            expected_result_sha256: "b".repeat(64),
            expected_output_sha256: "c".repeat(64),
            expected_authorization_review_sha256: "d".repeat(64),
            expected_isolated_runner_spec_sha256: "e".repeat(64),
            expected_implementation_sha256: "f".repeat(64),
            expected_implementation_review_sha256: "1".repeat(64),
            expected_implementation_independent_audit_sha256: "2".repeat(64),
            expected_protocol_sha256: "3".repeat(64),
            expected_candidate_set_sha256: "4".repeat(64),
            expected_training_store_dataset_sha256: "5".repeat(64),
            expected_selected_algorithm_three_seed_binding_sha256: "6".repeat(64),
            expected_sealed_holdout_split_commitment_sha256: "7".repeat(64),
            expected_target_id: "asset_return_20_market_sessions".to_string(),
            expected_frozen_candidate_algorithm_id: "ridge_multi_target_regression".to_string(),
            expected_sealed_holdout_projection_sha256: "8".repeat(64),
            expected_feature_order_sha256: "9".repeat(64),
            expected_preprocessing_sha256: "0".repeat(64),
            independent_reopen_and_second_implementation_recomputation_confirmed: true,
            exact_current_stage_51_through_stage_71_binding_confirmed: true,
            claim_first_authorization_consumption_and_no_replay_confirmed: true,
            exact_one_target_one_algorithm_three_seed_prediction_recomputation_confirmed: true,
            exact_three_metrics_component_bootstrap_holm_and_thresholds_bitwise_recomputed_confirmed: true,
            output_remains_untrusted_pending_future_adjudication_confirmed: true,
            no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    #[test]
    fn request_requires_all_fail_closed_confirmations() {
        let mut value = request();
        value.output_remains_untrusted_pending_future_adjudication_confirmed = false;
        assert!(validate_request(&"a".repeat(32), &value).is_err());
    }

    #[test]
    fn request_accepts_exact_stage_72_contract() {
        assert!(validate_request(&"a".repeat(32), &request()).is_ok());
    }

    #[test]
    fn complete_prior_chain_actor_cannot_validate() {
        let excluded = vec!["executor".to_string(), "reviewer".to_string()];
        assert!(!actor_is_independent("executor", &excluded));
        assert!(actor_is_independent("new-independent-validator", &excluded));
    }

    #[test]
    fn algorithm_gate_accepts_only_preregistered_families() {
        assert!(parse_algorithm_id("ridge_multi_target_regression").is_ok());
        assert!(parse_algorithm_id("gradient_boosted_multi_target_regression").is_ok());
        assert!(parse_algorithm_id("adaptive_after_holdout").is_err());
    }
}
