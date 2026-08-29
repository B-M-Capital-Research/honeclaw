//! Claim-first, single-use sealed-holdout confirmation attempts.
//!
//! This stage consumes one exact Stage 70 authorization before reopening any
//! sealed-holdout feature or label. The evaluator receives one target, one
//! frozen algorithm and exactly three frozen seeds. Its output is ephemeral,
//! content addressed and untrusted until a later independent validator has
//! reconstructed it. It cannot tune, select, store, reward, shadow or trade.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::historical_outcome_offline_dataset_feature_label_join_target_execution_attempts::{
    HistoricalOutcomeFeatureLabelJoinTargetVisibility,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::{
    HistoricalOutcomeTrainingExecutionAttemptClaim, HistoricalOutcomeTrainingExecutionAttemptResult,
    HistoricalOutcomeTrainingTargetModel, HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    HistoricalOutcomeTrainingUntrustedModelArtifact,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::HistoricalOutcomeTrainingExperimentAlgorithm;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_first_execution_authorizations::{
    ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    exact_first_execution_authorization_for_sealed_holdout_evaluation_audit,
    execution_attempt_eligible_sealed_holdout_evaluation_authorizations,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementations::HistoricalOutcomeSealedHoldoutEvaluationImplementationContract;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copies::HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_output_validations::{
    IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    independently_validated_training_store_copies_for_training_registration_review,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts::{
    PreparedValidationData, ValidationRow, algorithm_id, calibration_slope,
    component_block_bootstrap_p_value, directional_accuracy, f64_hex,
    holm_adjusted_p_values, mean_absolute_error, parse_and_preprocess_features, parse_f64_hex,
    project_targets, spearman, exact_training_attempt as exact_stage57_training_attempt,
    predict_target,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_first_execution_authorizations::exact_first_execution_authorization_for_validation_evaluation_audit;
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
    "hone-historical-outcome-sealed-holdout-evaluation-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-execution-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-execution-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str =
    "hone-untrusted-sealed-holdout-one-target-three-seed-confirmation-envelope-v1";
const EXECUTION_POLICY_VERSION: &str = "hone-sealed-holdout-claim-first-one-shot-confirmation-v1";
const ISOLATION_BACKEND: &str = "hone-in-process-capability-limited-sealed-holdout-confirmation-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const EXPECTED_METRIC_COUNT: usize = 3;
const MAX_ERROR_CHARS: usize = 1_200;
const NUMERIC_EPSILON: f64 = 1e-12;

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeHistoricalOutcomeSealedHoldoutEvaluationOnceRequest {
    expected_first_execution_authorization_review_id: String,
    expected_first_execution_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_runner_code_revision: String,
    expected_runner_contract_sha256: String,
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_implementation_independent_audit_sha256: String,
    expected_protocol_sha256: String,
    expected_candidate_set_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    claim_first_single_use_and_failure_consumes_confirmed: bool,
    exact_one_target_one_algorithm_three_seed_projection_confirmed: bool,
    sealed_holdout_only_and_no_other_partition_or_target_access_confirmed: bool,
    frozen_metrics_component_bootstrap_holm_and_sample_gates_confirmed: bool,
    no_feedback_tuning_refit_reselection_or_composite_confirmed: bool,
    untrusted_content_addressed_output_and_independent_validation_confirmed: bool,
    no_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub authorization_valid_until: DateTime<Utc>,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub runner_code_revision: String,
    pub runner_contract_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub implementation_review_id: String,
    pub implementation_review_sha256: String,
    pub implementation_independent_audit_sha256: String,
    pub protocol_sha256: String,
    pub candidate_set_sha256: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub target_commitments_sha256: String,
    pub selected_algorithm_three_seed_binding_sha256: String,
    pub sealed_holdout_split_commitment_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub exact_random_seeds: Vec<u64>,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub isolation_backend: String,
    pub claim_first_single_use_and_failure_consumes_confirmed: bool,
    pub exact_one_target_one_algorithm_three_seed_projection_confirmed: bool,
    pub sealed_holdout_only_and_no_other_partition_or_target_access_confirmed: bool,
    pub frozen_metrics_component_bootstrap_holm_and_sample_gates_confirmed: bool,
    pub no_feedback_tuning_refit_reselection_or_composite_confirmed: bool,
    pub untrusted_content_addressed_output_and_independent_validation_confirmed: bool,
    pub no_store_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub sealed_holdout_feature_read_allowed: bool,
    pub sealed_holdout_label_read_allowed: bool,
    pub evaluation_authorized: bool,
    pub validation_or_train_read_allowed: bool,
    pub other_target_read_allowed: bool,
    pub training_or_preprocessing_update_allowed: bool,
    pub candidate_reselection_allowed: bool,
    pub official_candidate_selection_allowed: bool,
    pub ephemeral_output_write_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub ambient_environment_available: bool,
    pub network_available: bool,
    pub external_tools_available: bool,
    pub child_process_allowed: bool,
    pub production_read_allowed: bool,
    pub production_write_allowed: bool,
    pub historical_state_mutation_allowed: bool,
    pub reward_allowed: bool,
    pub shadow_portfolio_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationMetric {
    pub algorithm: HistoricalOutcomeTrainingExperimentAlgorithm,
    pub random_seed: u64,
    pub target_id: String,
    pub sealed_holdout_row_count: usize,
    pub independent_component_count: usize,
    pub mae_f64_bits_hex: String,
    pub zero_baseline_mae_f64_bits_hex: String,
    pub relative_mae_improvement_f64_bits_hex: String,
    pub component_block_bootstrap_p_value_f64_bits_hex: String,
    pub holm_adjusted_p_value_f64_bits_hex: String,
    pub spearman_f64_bits_hex: Option<String>,
    pub directional_accuracy_f64_bits_hex: String,
    pub calibration_slope_f64_bits_hex: Option<String>,
    pub evidence_status: String,
    pub all_preregistered_thresholds_passed: bool,
    pub official_model_selection_metric: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope {
    pub schema_version: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub implementation_review_sha256: String,
    pub implementation_independent_audit_sha256: String,
    pub protocol_sha256: String,
    pub candidate_set_sha256: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub target_commitments_sha256: String,
    pub selected_algorithm_three_seed_binding_sha256: String,
    pub sealed_holdout_split_commitment_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub sealed_holdout_projection_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub exact_random_seeds: Vec<u64>,
    pub candidate_artifact_sha256s: Vec<String>,
    pub sealed_holdout_row_count: usize,
    pub independent_component_count: usize,
    pub exact_metric_count: usize,
    pub exact_candidate_hypothesis_count: usize,
    pub metrics: Vec<HistoricalOutcomeSealedHoldoutEvaluationMetric>,
    pub confirmation_status: String,
    pub all_three_seeds_passed: bool,
    pub insufficient_evidence: bool,
    pub sealed_holdout_features_accessed: bool,
    pub sealed_holdout_labels_accessed: bool,
    pub one_target_only: bool,
    pub one_algorithm_only: bool,
    pub training_or_preprocessing_updated: bool,
    pub feedback_reuse_allowed: bool,
    pub output_is_untrusted: bool,
    pub independent_output_validation_completed: bool,
    pub official_candidate_selection_completed: bool,
    pub composite_score_created: bool,
    pub global_model_validity_claimed: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub scalar_reward_written: bool,
    pub shadow_position_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus {
    CompletedWithUntrustedSealedHoldoutConfirmation,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus,
    pub exit_code: i32,
    pub output_sha256: Option<String>,
    pub output_bytes: usize,
    pub bounded_error: Option<String>,
    pub untrusted_confirmation_envelope:
        Option<HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope>,
    pub ephemeral_directory_removed: bool,
    pub sealed_holdout_features_accessed: bool,
    pub sealed_holdout_labels_accessed: bool,
    pub evaluation_completed: bool,
    pub independent_output_validation_completed: bool,
    pub official_candidate_selection_completed: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptItem {
    pub claim: HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
    pub result: Option<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub isolation_backend: String,
    pub invocation_endpoint_available: bool,
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_confirmation_envelope_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
    pub attempts: Vec<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptItem>,
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
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptReadinessSummary {
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_confirmation_envelope_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt {
    pub claim: HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
    pub result: HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult,
}

#[derive(Debug)]
struct EvaluationFailure {
    message: String,
    ephemeral_directory_removed: bool,
    sealed_holdout_inputs_accessed: bool,
}

impl EvaluationFailure {
    #[cfg(test)]
    fn before_access(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral_directory_removed: true,
            sealed_holdout_inputs_accessed: false,
        }
    }

    fn after_access(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral_directory_removed: true,
            sealed_holdout_inputs_accessed: true,
        }
    }
}

pub(crate) async fn handle_get_historical_outcome_sealed_holdout_evaluation_execution_attempts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_registry(&state, Utc::now()).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => {
            warn!(%error, "sealed-holdout evaluation execution registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "sealed-holdout 一次性评估执行记录暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_historical_outcome_sealed_holdout_evaluation_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeHistoricalOutcomeSealedHoldoutEvaluationOnceRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match invoke_once(&state, &admin.user_id, &isolated_runner_id, request).await {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_sealed_holdout_evaluation_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptReadinessSummary, String> {
    let registry = build_registry(state, Utc::now()).await?;
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptReadinessSummary {
            invocation_eligible_authorization_count: registry
                .invocation_eligible_authorization_count,
            claim_count: registry.claim_count,
            completed_attempt_count: registry.completed_attempt_count,
            failed_attempt_count: registry.failed_attempt_count,
            untrusted_confirmation_envelope_count: registry.untrusted_confirmation_envelope_count,
            independent_output_validation_eligible_count: registry
                .independent_output_validation_eligible_count,
            execution_status: registry.execution_status,
        },
    )
}

pub(crate) async fn completed_sealed_holdout_evaluation_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .into_iter()
        .map(|value| (value.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    Ok(claims.into_iter().filter_map(|claim| {
        let result = result_index.get(&claim.attempt_id)?.clone();
        (result.status == HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus::CompletedWithUntrustedSealedHoldoutConfirmation)
            .then_some(CompletedHistoricalOutcomeSealedHoldoutEvaluationExecutionAttempt { claim, result })
    }).collect())
}

/// Reopens the exact immutable Stage 71 chain and returns the frozen train-only
/// candidate artifacts for a later, separately authorized controlled-shadow
/// execution. Callers must already have persisted their own claim before using
/// this helper because it reopens model inputs.
pub(crate) async fn exact_training_envelope_for_controlled_shadow_execution(
    state: &AppState,
    attempt_id: &str,
    expected_claim_sha256: &str,
    expected_result_sha256: &str,
) -> Result<HistoricalOutcomeTrainingUntrustedArtifactEnvelope, String> {
    let completed =
        completed_sealed_holdout_evaluation_execution_attempts_for_independent_validation(state)
            .await?
            .into_iter()
            .find(|value| {
                value.claim.attempt_id == attempt_id
                    && value.claim.claim_sha256 == expected_claim_sha256
                    && value.result.result_sha256 == expected_result_sha256
            })
            .ok_or_else(|| "找不到受控影子设计精确绑定的 Stage 71 完成记录".to_string())?;
    let authorization = exact_first_execution_authorization_for_sealed_holdout_evaluation_audit(
        state,
        &completed.claim.isolated_runner_id,
        &completed.claim.authorization_review_id,
        &completed.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "Stage 71 的一次性授权链已经无法重开".to_string())?;
    let admitted = exact_admitted_candidate(state, &authorization).await?;
    let (_, training_result) = exact_training_attempt(&admitted, state).await?;
    training_result
        .untrusted_artifact_envelope
        .ok_or_else(|| "Stage 57 训练执行缺少冻结候选模型工件".to_string())
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry, String> {
    let eligible =
        execution_attempt_eligible_sealed_holdout_evaluation_authorizations(state, now).await?;
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let attempts = claims
        .iter()
        .cloned()
        .map(|claim| {
            let result = result_index
                .get(claim.attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptItem { claim, result }
        })
        .collect::<Vec<_>>();
    let completed_attempt_count = results.iter().filter(|value| value.status == HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus::CompletedWithUntrustedSealedHoldoutConfirmation).count();
    let failed_attempt_count = results.iter().filter(|value| value.status == HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus::FailedAuthorizationConsumed).count();
    let untrusted_confirmation_envelope_count = results
        .iter()
        .filter(|value| value.untrusted_confirmation_envelope.is_some())
        .count();
    let execution_status = if !eligible.is_empty() {
        "sealed_holdout_execution_authorized_not_claimed"
    } else if claims.len() > results.len() {
        "sealed_holdout_authorization_claimed_execution_interrupted_fail_closed"
    } else if completed_attempt_count > 0 {
        "untrusted_sealed_holdout_confirmation_waiting_independent_validation"
    } else if failed_attempt_count > 0 {
        "sealed_holdout_authorization_consumed_execution_failed"
    } else {
        "waiting_for_current_unexpired_stage_70_authorization"
    }
    .to_string();
    Ok(HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        invocation_endpoint_available: true,
        invocation_eligible_authorization_count: eligible.len(),
        claim_count: claims.len(),
        completed_attempt_count,
        failed_attempt_count,
        untrusted_confirmation_envelope_count,
        independent_output_validation_eligible_count: untrusted_confirmation_envelope_count,
        execution_status,
        attempts,
        official_candidate_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 71 仅消费一个精确 Stage 70 授权，执行一次单目标、单算法、三冻结种子的 sealed-holdout confirmatory 评估；输出仍不可信，必须独立验证，且不得反馈、选模、写库、定义奖励、模拟或交易。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    admin_id: &str,
    isolated_runner_id: &str,
    request: InvokeHistoricalOutcomeSealedHoldoutEvaluationOnceRequest,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult, String> {
    verify_invocation_confirmations(&request)?;
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let now = Utc::now();
    let authorization =
        execution_attempt_eligible_sealed_holdout_evaluation_authorizations(state, now)
            .await?
            .into_iter()
            .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
            .ok_or_else(|| "没有当前、未过期且未消费的 Stage 70 一次性授权".to_string())?;
    verify_request(&authorization, &request)?;
    let (claims, _) = read_attempt_records(state).await?;
    if claims.iter().any(|claim| {
        claim.authorization_review_id == authorization.review.review_id
            || claim.isolated_runner_id == isolated_runner_id
    }) {
        return Err(
            "这条 sealed-holdout 授权或 runner 已经被 claim；成功、失败或中断都不得重放"
                .to_string(),
        );
    }
    let current_binary_sha256 = reverify_current_binary_sha256().await?;
    if current_binary_sha256 != authorization.runner.runner_artifact_sha256 {
        return Err("当前执行二进制摘要与 Stage 69–70 冻结 runner 工件不一致".to_string());
    }
    let admitted = exact_admitted_candidate(state, &authorization).await?;
    let (_training_claim, training_result) = exact_training_attempt(&admitted, state).await?;
    let training_envelope = training_result
        .untrusted_artifact_envelope
        .as_ref()
        .ok_or_else(|| "Stage 57 训练执行缺少候选模型工件".to_string())?;
    let validated_copy = exact_training_store_copy(&authorization, state).await?;
    let dataset = validated_copy
        .copied_dataset
        .attempt
        .training_store_dataset
        .as_ref()
        .ok_or_else(|| "独立校验训练副本缺少数据集".to_string())?;
    verify_exact_inputs(
        &authorization,
        &admitted,
        &training_result,
        training_envelope,
        dataset,
    )?;
    let claim = new_claim(&authorization, admin_id, now)?;
    write_immutable_json(
        &claim_directory(state, isolated_runner_id).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let execution = match exact_current_historical_outcome_offline_dataset_for_transformation(
        state,
        &dataset.dataset_id,
        &dataset.dataset_content_sha256,
        &dataset.dataset_manifest_sha256,
        &dataset.candidate_set_sha256,
    )
    .await
    {
        Ok(Some(raw_dataset)) => prepare_sealed_holdout_projection(
            dataset,
            &raw_dataset,
            training_envelope,
            &authorization.runner.implementation.implementation_contract,
        )
        .and_then(|prepared| {
            evaluate_fixed_contract(&authorization, &claim, training_envelope, prepared)
        })
        .map_err(EvaluationFailure::after_access),
        Ok(None) => Err(EvaluationFailure::after_access(
            "sealed-holdout label broker 无法重开精确当前原始结果数据集",
        )),
        Err(error) => Err(EvaluationFailure::after_access(format!(
            "sealed-holdout label broker 重开精确数据集失败：{error}"
        ))),
    };
    let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let result = match execution {
        Ok(envelope) => {
            execute_ephemeral_output(&authorization, &claim, envelope, duration_millis).await?
        }
        Err(failure) => new_failed_result(&claim, Utc::now(), duration_millis, &failure)?,
    };
    write_immutable_json(
        &result_directory(state, isolated_runner_id).join(format!("{}.json", result.result_id)),
        &result,
    )
    .await?;
    Ok(result)
}

fn verify_invocation_confirmations(
    request: &InvokeHistoricalOutcomeSealedHoldoutEvaluationOnceRequest,
) -> Result<(), String> {
    if !request.claim_first_single_use_and_failure_consumes_confirmed
        || !request.exact_one_target_one_algorithm_three_seed_projection_confirmed
        || !request.sealed_holdout_only_and_no_other_partition_or_target_access_confirmed
        || !request.frozen_metrics_component_bootstrap_holm_and_sample_gates_confirmed
        || !request.no_feedback_tuning_refit_reselection_or_composite_confirmed
        || !request.untrusted_content_addressed_output_and_independent_validation_confirmed
        || !request.no_store_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err("必须确认一次性消费、单目标三种子、sealed-holdout 隔离、冻结统计、无反馈复用和零投资执行权限".to_string());
    }
    Ok(())
}

fn verify_request(
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    request: &InvokeHistoricalOutcomeSealedHoldoutEvaluationOnceRequest,
) -> Result<(), String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
    let protocol = &implementation.upstream_protocol;
    let review = &runner.implementation_review;
    if request.expected_first_execution_authorization_review_id != authorization.review.review_id
        || request.expected_first_execution_authorization_review_sha256
            != authorization.review.review_sha256
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != runner.runner_artifact_sha256
        || request.expected_runner_code_revision != runner.runner_code_revision
        || request.expected_runner_contract_sha256 != runner.runner_contract.contract_sha256
        || request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_review_id != review.review_id
        || request.expected_implementation_review_sha256 != review.review_sha256
        || request.expected_implementation_independent_audit_sha256
            != review.independent_audit.audit_sha256
        || request.expected_protocol_sha256 != protocol.protocol_sha256
        || request.expected_candidate_set_sha256 != contract.candidate_set_sha256
        || request.expected_training_store_dataset_sha256 != contract.training_store_dataset_sha256
        || request.expected_selected_algorithm_three_seed_binding_sha256
            != contract.selected_algorithm_three_seed_binding_sha256
        || request.expected_sealed_holdout_split_commitment_sha256
            != contract.sealed_holdout_split_commitment_sha256
        || request.expected_target_id != contract.target_id
        || request.expected_frozen_candidate_algorithm_id != contract.frozen_candidate_algorithm_id
    {
        return Err(
            "授权、runner、实现、协议、目标、算法或 sealed-holdout 绑定已经变化，请刷新后重试"
                .to_string(),
        );
    }
    Ok(())
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
        .ok_or_else(|| "找不到 Stage 66 协议精确绑定的当前 Stage 65 逐目标准入记录".to_string())
}

async fn exact_training_attempt(
    admitted: &AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate,
    state: &AppState,
) -> Result<
    (
        HistoricalOutcomeTrainingExecutionAttemptClaim,
        HistoricalOutcomeTrainingExecutionAttemptResult,
    ),
    String,
> {
    let validation_claim = &admitted.candidate.source.attempt.claim;
    let authorization = exact_first_execution_authorization_for_validation_evaluation_audit(
        state,
        &validation_claim.isolated_runner_id,
        &validation_claim.authorization_review_id,
        &validation_claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "Stage 63 validation 评估授权链已经无法重开".to_string())?;
    exact_stage57_training_attempt(&authorization, state).await
}

async fn exact_training_store_copy(
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    state: &AppState,
) -> Result<IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset, String>
{
    let expected = &authorization
        .runner
        .implementation
        .implementation_contract
        .training_store_dataset_sha256;
    independently_validated_training_store_copies_for_training_registration_review(state)
        .await?
        .into_iter()
        .find(|value| {
            value
                .copied_dataset
                .attempt
                .training_store_dataset
                .as_ref()
                .is_some_and(|dataset| &dataset.training_store_dataset_sha256 == expected)
        })
        .ok_or_else(|| "找不到精确绑定且仍独立校验的训练存储副本".to_string())
}

fn verify_exact_inputs(
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    admitted: &AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate,
    training_result: &HistoricalOutcomeTrainingExecutionAttemptResult,
    envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
) -> Result<(), String> {
    let contract = &authorization.runner.implementation.implementation_contract;
    let protocol = &authorization.runner.implementation.upstream_protocol;
    let selected = &admitted.candidate.recommendation.recommended_algorithm;
    let exact = dataset.training_store_dataset_sha256 == contract.training_store_dataset_sha256
        && envelope.training_store_dataset_sha256 == contract.training_store_dataset_sha256
        && envelope.feature_order.len() == EXPECTED_FEATURE_COUNT
        && hash_serializable(&envelope.feature_order)? == contract.feature_order_sha256
        && hash_serializable(&envelope.preprocessing)? == contract.preprocessing_sha256
        && envelope.model_artifacts.len() == 9
        && training_result.untrusted_artifact_envelope.is_some()
        && dataset.candidate_set_sha256 == contract.candidate_set_sha256
        && protocol.target_id == contract.target_id
        && selected
            .as_ref()
            .is_some_and(|value| algorithm_id(value) == contract.frozen_candidate_algorithm_id)
        && contract.exact_random_seeds == EXPECTED_SEEDS
        && dataset.validation_targets_withheld
        && dataset.sealed_holdout_targets_withheld;
    if !exact {
        return Err(
            "sealed-holdout 输入、训练候选、目标、算法、特征或预处理与冻结合同不一致".to_string(),
        );
    }
    Ok(())
}

fn prepare_sealed_holdout_projection(
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    raw_dataset: &HistoricalOutcomeOfflineDataset,
    envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
) -> Result<PreparedValidationData, String> {
    if raw_dataset.dataset_id != dataset.dataset_id
        || raw_dataset.dataset_content_sha256 != dataset.dataset_content_sha256
        || raw_dataset.manifest_sha256 != dataset.dataset_manifest_sha256
        || raw_dataset.candidate_set_sha256 != dataset.candidate_set_sha256
        || envelope.feature_order.len() != EXPECTED_FEATURE_COUNT
        || envelope.preprocessing.len() != EXPECTED_FEATURE_COUNT
    {
        return Err("sealed-holdout label broker 的数据集、特征或预处理漂移".to_string());
    }
    let holdout_rows = dataset
        .rows
        .iter()
        .filter(|row| {
            row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
                && row.split == HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
        })
        .collect::<Vec<_>>();
    if holdout_rows.is_empty() {
        return Err("sealed-holdout 分区没有可评估行".to_string());
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
        return Err("sealed-holdout 与训练/验证 entry 集合重叠".to_string());
    }
    let raw_index = raw_dataset
        .entries
        .iter()
        .filter(|entry| allowed_ids.contains(entry.entry_id.as_str()))
        .map(|entry| (entry.entry_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    if raw_index.len() != allowed_ids.len() {
        return Err("sealed-holdout label broker 缺少精确原始结果条目".to_string());
    }
    let target_order = vec![contract.target_id.clone()];
    let mut rows = Vec::with_capacity(holdout_rows.len());
    for row in holdout_rows {
        let raw = raw_index
            .get(row.dataset_entry_id.as_str())
            .ok_or_else(|| "sealed-holdout 行缺少原始结果".to_string())?;
        if raw.entry_sha256 != row.dataset_entry_sha256
            || raw.decision_available_at != row.decision_available_at
        {
            return Err(format!(
                "sealed-holdout 条目 {} 的点时绑定漂移",
                row.dataset_entry_id
            ));
        }
        rows.push(ValidationRow {
            dataset_entry_id: row.dataset_entry_id.clone(),
            component_id: row.component_id.clone(),
            features: parse_and_preprocess_features(
                row,
                &envelope.feature_order,
                &envelope.preprocessing,
            )?,
            targets: project_targets(raw, &target_order)?,
        });
    }
    rows.sort_by(|left, right| left.dataset_entry_id.cmp(&right.dataset_entry_id));
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
    Ok(PreparedValidationData {
        feature_order: envelope.feature_order.clone(),
        target_order,
        rows,
        projection_sha256,
    })
}

fn evaluate_fixed_contract(
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    claim: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    data: PreparedValidationData,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope, String> {
    let contract = &authorization.runner.implementation.implementation_contract;
    validate_frozen_contract(contract, training_envelope, &data)?;
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
    let zero_mae = mean_absolute_error(&actual, &zero)?;
    let mut selected_artifacts = Vec::new();
    let mut provisional = Vec::new();
    let mut p_values = Vec::new();
    for seed in EXPECTED_SEEDS {
        let artifact = exact_candidate_artifact(training_envelope, &algorithm, seed)?;
        let model = exact_target_model(artifact, &contract.target_id)?;
        let predicted = predict_single_target(model, &data.feature_order, &data.rows)?;
        let mae = mean_absolute_error(&actual, &predicted)?;
        let improvement = if zero_mae.abs() <= NUMERIC_EPSILON {
            0.0
        } else {
            (zero_mae - mae) / zero_mae
        };
        let p = component_block_bootstrap_p_value(
            &data.rows,
            &actual,
            &predicted,
            &zero,
            contract.bootstrap_replications,
            contract.bootstrap_random_seed ^ seed,
        )?;
        selected_artifacts.push(artifact.artifact_sha256.clone());
        p_values.push(p);
        provisional.push((
            seed,
            mae,
            improvement,
            p,
            spearman(&actual, &predicted),
            directional_accuracy(&actual, &predicted),
            calibration_slope(&actual, &predicted),
        ));
    }
    let adjusted = holm_adjusted_p_values(&p_values);
    let insufficient = data.rows.len() < contract.minimum_sealed_holdout_rows as usize
        || components.len() < contract.minimum_independent_components as usize;
    let mut metrics = Vec::with_capacity(EXPECTED_METRIC_COUNT);
    for ((seed, mae, improvement, p, rho, directional, calibration), q) in
        provisional.into_iter().zip(adjusted)
    {
        let passed =
            !insufficient && metric_passes(contract, improvement, q, rho, directional, calibration);
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
    let all_three = !insufficient
        && metrics
            .iter()
            .all(|metric| metric.all_preregistered_thresholds_passed);
    let confirmation_status = if insufficient {
        "insufficient_evidence_fail_closed"
    } else if all_three {
        "untrusted_confirmatory_pass_pending_independent_validation"
    } else {
        "untrusted_confirmatory_fail_pending_independent_validation"
    }
    .to_string();
    let output = HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope {
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
        candidate_artifact_sha256s: selected_artifacts,
        sealed_holdout_row_count: data.rows.len(),
        independent_component_count: components.len(),
        exact_metric_count: metrics.len(),
        exact_candidate_hypothesis_count: metrics.len(),
        metrics,
        confirmation_status,
        all_three_seeds_passed: all_three,
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
    };
    validate_envelope(&output, contract)?;
    Ok(output)
}

fn validate_frozen_contract(
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
    envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    data: &PreparedValidationData,
) -> Result<(), String> {
    let exact = contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == 9
        && contract.exact_candidate_hypothesis_count == 3
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
        && hash_serializable(&envelope.preprocessing)? == contract.preprocessing_sha256
        && data.target_order == [contract.target_id.clone()];
    if !exact {
        return Err("Stage 66–70 冻结 sealed-holdout 统计合同已经漂移".to_string());
    }
    Ok(())
}

pub(crate) fn parse_algorithm_id(
    value: &str,
) -> Result<HistoricalOutcomeTrainingExperimentAlgorithm, String> {
    match value {
        "ridge_multi_target_regression" => {
            Ok(HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression)
        }
        "gradient_boosted_multi_target_regression" => {
            Ok(HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression)
        }
        _ => Err("sealed-holdout 冻结候选算法不是准入的 ridge 或 boosted".to_string()),
    }
}

pub(crate) fn exact_candidate_artifact<'a>(
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
        return Err(format!("冻结算法种子 {seed} 不是精确一个候选工件"));
    }
    Ok(values[0])
}

pub(crate) fn exact_target_model<'a>(
    artifact: &'a HistoricalOutcomeTrainingUntrustedModelArtifact,
    target_id: &str,
) -> Result<&'a HistoricalOutcomeTrainingTargetModel, String> {
    let values = artifact
        .target_models
        .iter()
        .filter(|model| model.target_id == target_id)
        .collect::<Vec<_>>();
    if values.len() != 1 {
        return Err(format!("候选工件不是精确一个目标模型 {target_id}"));
    }
    Ok(values[0])
}

fn predict_single_target(
    model: &HistoricalOutcomeTrainingTargetModel,
    feature_order: &[String],
    rows: &[ValidationRow],
) -> Result<Vec<f64>, String> {
    rows.iter()
        .map(|row| predict_target(model, feature_order, &row.features))
        .collect()
}

fn metric_passes(
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

async fn execute_ephemeral_output(
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    claim: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
    envelope: HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope,
    duration_millis: u64,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult, String> {
    let bytes = serde_json::to_vec(&envelope).map_err(|error| error.to_string())?;
    if bytes.len() > authorization.runner.runner_contract.maximum_output_bytes as usize {
        return new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure::after_access("未验证 sealed-holdout 输出超过 runner 上限"),
        );
    }
    let directory =
        std::env::temp_dir().join(format!("hone-sealed-holdout-evaluation-{}", Uuid::new_v4()));
    if let Err(error) = tokio::fs::create_dir(&directory).await {
        return new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure::after_access(format!("一次性工作目录创建失败：{error}")),
        );
    }
    let path = directory.join("untrusted-sealed-holdout-confirmation.json");
    let staged = async {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
            .map_err(|error| error.to_string())?;
        file.write_all(&bytes)
            .await
            .map_err(|error| error.to_string())?;
        file.sync_all().await.map_err(|error| error.to_string())?;
        let read_back = tokio::fs::read(&path)
            .await
            .map_err(|error| error.to_string())?;
        if read_back != bytes {
            return Err("一次性 sealed-holdout 输出回读不一致".to_string());
        }
        Ok::<_, String>(hash_bytes(&read_back))
    }
    .await;
    let cleanup = tokio::fs::remove_dir_all(&directory).await;
    let removed = cleanup.is_ok()
        || tokio::fs::metadata(&directory)
            .await
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
    match (staged, cleanup, removed) {
        (Ok(output_sha256), Ok(()), true) => new_success_result(
            claim,
            Utc::now(),
            duration_millis,
            envelope,
            output_sha256,
            bytes.len(),
        ),
        (Ok(_), Err(error), removed) => new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure {
                message: error.to_string(),
                ephemeral_directory_removed: removed,
                sealed_holdout_inputs_accessed: true,
            },
        ),
        (Err(error), _, removed) => new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure {
                message: error,
                ephemeral_directory_removed: removed,
                sealed_holdout_inputs_accessed: true,
            },
        ),
        (Ok(_), Ok(()), false) => new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure {
                message: "一次性 sealed-holdout 工作目录清理状态无法确认".to_string(),
                ephemeral_directory_removed: false,
                sealed_holdout_inputs_accessed: true,
            },
        ),
    }
}

fn new_claim(
    authorization: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization,
    invoked_by: &str,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim, String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
    let protocol = &implementation.upstream_protocol;
    let mut claim = HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        attempt_id: String::new(),
        claim_sha256: String::new(),
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        authorization_valid_until: authorization.review.authorization_valid_until,
        isolated_runner_id: runner.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
        runner_code_revision: runner.runner_code_revision.clone(),
        runner_contract_sha256: runner.runner_contract.contract_sha256.clone(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        implementation_review_id: runner.implementation_review.review_id.clone(),
        implementation_review_sha256: runner.implementation_review.review_sha256.clone(),
        implementation_independent_audit_sha256: runner
            .implementation_review
            .independent_audit
            .audit_sha256
            .clone(),
        protocol_sha256: protocol.protocol_sha256.clone(),
        candidate_set_sha256: contract.candidate_set_sha256.clone(),
        training_store_dataset_sha256: contract.training_store_dataset_sha256.clone(),
        rows_sha256: protocol.rows_sha256.clone(),
        target_commitments_sha256: protocol.target_commitments_sha256.clone(),
        selected_algorithm_three_seed_binding_sha256: contract
            .selected_algorithm_three_seed_binding_sha256
            .clone(),
        sealed_holdout_split_commitment_sha256: contract
            .sealed_holdout_split_commitment_sha256
            .clone(),
        target_id: contract.target_id.clone(),
        frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id.clone(),
        exact_random_seeds: contract.exact_random_seeds.clone(),
        claimed_at,
        invoked_by: invoked_by.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        claim_first_single_use_and_failure_consumes_confirmed: true,
        exact_one_target_one_algorithm_three_seed_projection_confirmed: true,
        sealed_holdout_only_and_no_other_partition_or_target_access_confirmed: true,
        frozen_metrics_component_bootstrap_holm_and_sample_gates_confirmed: true,
        no_feedback_tuning_refit_reselection_or_composite_confirmed: true,
        untrusted_content_addressed_output_and_independent_validation_confirmed: true,
        no_store_reward_shadow_order_broker_or_trading_confirmed: true,
        authorization_consumed: true,
        invocation_started: false,
        sealed_holdout_feature_read_allowed: true,
        sealed_holdout_label_read_allowed: true,
        evaluation_authorized: true,
        validation_or_train_read_allowed: false,
        other_target_read_allowed: false,
        training_or_preprocessing_update_allowed: false,
        candidate_reselection_allowed: false,
        official_candidate_selection_allowed: false,
        ephemeral_output_write_allowed: true,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        ambient_environment_available: false,
        network_available: false,
        external_tools_available: false,
        child_process_allowed: false,
        production_read_allowed: false,
        production_write_allowed: false,
        historical_state_mutation_allowed: false,
        reward_allowed: false,
        shadow_portfolio_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = claim_fingerprint(&claim)?;
    claim.attempt_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim)?;
    Ok(claim)
}

fn new_success_result(
    claim: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    envelope: HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope,
    output_sha256: String,
    output_bytes: usize,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(), execution_policy_version: EXECUTION_POLICY_VERSION.to_string(), result_id: String::new(), result_sha256: String::new(), attempt_id: claim.attempt_id.clone(), claim_sha256: claim.claim_sha256.clone(), completed_at, duration_millis,
        status: HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus::CompletedWithUntrustedSealedHoldoutConfirmation, exit_code: 0, output_sha256: Some(output_sha256), output_bytes, bounded_error: None, untrusted_confirmation_envelope: Some(envelope), ephemeral_directory_removed: true,
        sealed_holdout_features_accessed: true, sealed_holdout_labels_accessed: true, evaluation_completed: true, independent_output_validation_completed: false, official_candidate_selection_completed: false, model_artifact_store_written: false, metric_store_written: false, reward_authorized: false, shadow_portfolio_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn new_failed_result(
    claim: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    failure: &EvaluationFailure,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(), execution_policy_version: EXECUTION_POLICY_VERSION.to_string(), result_id: String::new(), result_sha256: String::new(), attempt_id: claim.attempt_id.clone(), claim_sha256: claim.claim_sha256.clone(), completed_at, duration_millis,
        status: HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus::FailedAuthorizationConsumed, exit_code: 1, output_sha256: None, output_bytes: 0, bounded_error: Some(failure.message.chars().take(MAX_ERROR_CHARS).collect()), untrusted_confirmation_envelope: None, ephemeral_directory_removed: failure.ephemeral_directory_removed,
        sealed_holdout_features_accessed: failure.sealed_holdout_inputs_accessed, sealed_holdout_labels_accessed: failure.sealed_holdout_inputs_accessed, evaluation_completed: false, independent_output_validation_completed: false, official_candidate_selection_completed: false, model_artifact_store_written: false, metric_store_written: false, reward_authorized: false, shadow_portfolio_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn validate_claim(
    claim: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
) -> Result<(), String> {
    let closed = claim.authorization_consumed
        && !claim.invocation_started
        && claim.sealed_holdout_feature_read_allowed
        && claim.sealed_holdout_label_read_allowed
        && claim.evaluation_authorized
        && !claim.validation_or_train_read_allowed
        && !claim.other_target_read_allowed
        && !claim.training_or_preprocessing_update_allowed
        && !claim.candidate_reselection_allowed
        && !claim.official_candidate_selection_allowed
        && claim.ephemeral_output_write_allowed
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
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&claim.attempt_id)
        || !valid_sha256(&claim.claim_sha256)
        || claim.attempt_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != claim_fingerprint(claim)?
        || claim.isolation_backend != ISOLATION_BACKEND
        || claim.invoked_by.trim().is_empty()
        || claim.claimed_at >= claim.authorization_valid_until
        || claim.exact_random_seeds != EXPECTED_SEEDS
        || !claim.claim_first_single_use_and_failure_consumes_confirmed
        || !claim.exact_one_target_one_algorithm_three_seed_projection_confirmed
        || !claim.sealed_holdout_only_and_no_other_partition_or_target_access_confirmed
        || !claim.frozen_metrics_component_bootstrap_holm_and_sample_gates_confirmed
        || !claim.no_feedback_tuning_refit_reselection_or_composite_confirmed
        || !claim.untrusted_content_addressed_output_and_independent_validation_confirmed
        || !claim.no_store_reward_shadow_order_broker_or_trading_confirmed
        || !closed
    {
        return Err("sealed-holdout 评估 claim 无效或越权".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult,
    claim: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
) -> Result<(), String> {
    let success = result.status == HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus::CompletedWithUntrustedSealedHoldoutConfirmation;
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != result_fingerprint(result)?
        || result.attempt_id != claim.attempt_id
        || result.claim_sha256 != claim.claim_sha256
        || result.completed_at < claim.claimed_at
        || success != (result.exit_code == 0)
        || success != result.output_sha256.as_deref().is_some_and(valid_sha256)
        || success != result.untrusted_confirmation_envelope.is_some()
        || success != (result.output_bytes > 0)
        || success != result.evaluation_completed
        || result.sealed_holdout_labels_accessed != result.sealed_holdout_features_accessed
        || (success && !result.sealed_holdout_labels_accessed)
        || result.independent_output_validation_completed
        || result.official_candidate_selection_completed
        || result.model_artifact_store_written
        || result.metric_store_written
        || result.reward_authorized
        || result.shadow_portfolio_authorized
        || result.order_generation_authorized
        || result.broker_access_authorized
        || result.trading_authorized
        || (success && !result.ephemeral_directory_removed)
        || (!success && result.bounded_error.as_deref().is_none_or(str::is_empty))
    {
        return Err("sealed-holdout 评估执行结果无效或越权".to_string());
    }
    if let Some(envelope) = &result.untrusted_confirmation_envelope {
        if hash_serializable(envelope)? != result.output_sha256.clone().unwrap_or_default() {
            return Err("sealed-holdout envelope 与输出摘要不一致".to_string());
        }
    }
    Ok(())
}

fn validate_envelope(
    envelope: &HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope,
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
) -> Result<(), String> {
    let metric_seeds = envelope
        .metrics
        .iter()
        .map(|metric| metric.random_seed)
        .collect::<BTreeSet<_>>();
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION
        || envelope.target_id != contract.target_id
        || envelope.frozen_candidate_algorithm_id != contract.frozen_candidate_algorithm_id
        || envelope.exact_random_seeds != EXPECTED_SEEDS
        || metric_seeds != EXPECTED_SEEDS.into_iter().collect()
        || envelope.candidate_artifact_sha256s.len() != EXPECTED_SEEDS.len()
        || envelope
            .candidate_artifact_sha256s
            .iter()
            .any(|value| !valid_sha256(value))
        || envelope.exact_metric_count != EXPECTED_METRIC_COUNT
        || envelope.exact_candidate_hypothesis_count != EXPECTED_METRIC_COUNT
        || envelope.metrics.len() != EXPECTED_METRIC_COUNT
        || !envelope.sealed_holdout_features_accessed
        || !envelope.sealed_holdout_labels_accessed
        || !envelope.one_target_only
        || !envelope.one_algorithm_only
        || envelope.training_or_preprocessing_updated
        || envelope.feedback_reuse_allowed
        || !envelope.output_is_untrusted
        || envelope.independent_output_validation_completed
        || envelope.official_candidate_selection_completed
        || envelope.composite_score_created
        || envelope.global_model_validity_claimed
        || envelope.model_artifact_store_written
        || envelope.metric_store_written
        || envelope.scalar_reward_written
        || envelope.shadow_position_written
        || envelope.order_generated
        || envelope.broker_accessed
        || envelope.trade_executed
        || envelope.metrics.iter().any(|metric| {
            algorithm_id(&metric.algorithm) != contract.frozen_candidate_algorithm_id
                || metric.target_id != contract.target_id
                || metric.official_model_selection_metric
                || !valid_f64_hex(&metric.mae_f64_bits_hex)
                || !valid_f64_hex(&metric.holm_adjusted_p_value_f64_bits_hex)
        })
    {
        return Err("未验证 sealed-holdout confirmation envelope 无效或越权".to_string());
    }
    Ok(())
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim>,
        Vec<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim>(
        &execution_root(state).join("claims"),
    )
    .await?;
    let results =
        read_nested_json::<HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult>(
            &execution_root(state).join("results"),
        )
        .await?;
    let mut claim_ids = BTreeSet::new();
    let mut authorization_ids = BTreeSet::new();
    for claim in &claims {
        validate_claim(claim)?;
        if !claim_ids.insert(claim.attempt_id.clone())
            || !authorization_ids.insert(claim.authorization_review_id.clone())
        {
            return Err("sealed-holdout 评估 claim 重复".to_string());
        }
        exact_first_execution_authorization_for_sealed_holdout_evaluation_audit(
            state,
            &claim.isolated_runner_id,
            &claim.authorization_review_id,
            &claim.authorization_review_sha256,
        )
        .await?
        .ok_or_else(|| "sealed-holdout claim 的授权链不再可重开".to_string())?;
    }
    let claim_index = claims
        .iter()
        .map(|claim| (claim.attempt_id.as_str(), claim))
        .collect::<BTreeMap<_, _>>();
    let mut result_ids = BTreeSet::new();
    let mut result_attempts = BTreeSet::new();
    for result in &results {
        let claim = claim_index
            .get(result.attempt_id.as_str())
            .ok_or_else(|| "sealed-holdout 结果缺少 claim".to_string())?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_attempts.insert(result.attempt_id.clone())
        {
            return Err("sealed-holdout 评估结果重复".to_string());
        }
    }
    Ok((claims, results))
}

async fn read_nested_json<T: for<'de> Deserialize<'de>>(root: &Path) -> Result<Vec<T>, String> {
    let mut stack = vec![root.to_path_buf()];
    let mut values = Vec::new();
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
                values.push(
                    serde_json::from_slice(
                        &tokio::fs::read(&path)
                            .await
                            .map_err(|error| error.to_string())?,
                    )
                    .map_err(|error| error.to_string())?,
                );
            }
        }
    }
    Ok(values)
}

async fn reverify_current_binary_sha256() -> Result<String, String> {
    Ok(hash_bytes(
        &tokio::fs::read(std::env::current_exe().map_err(|error| error.to_string())?)
            .await
            .map_err(|error| error.to_string())?,
    ))
}
fn claim_fingerprint(
    value: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptClaim,
) -> Result<String, String> {
    fingerprint_without(value, &["attempt_id", "claim_sha256"])
}
fn result_fingerprint(
    value: &HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptResult,
) -> Result<String, String> {
    fingerprint_without(value, &["result_id", "result_sha256"])
}
fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint input is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}
fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(hash_bytes(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}
fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}
fn valid_f64_hex(value: &str) -> bool {
    value.len() == 16 && parse_f64_hex(value).is_ok()
}
fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn execution_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-sealed-holdout-evaluation-execution-attempts")
}
fn claim_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("claims").join(runner_id)
}
fn result_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("results").join(runner_id)
}

struct ExecutionLock {
    path: PathBuf,
}
impl Drop for ExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn acquire_lock(state: &AppState, runner_id: &str) -> Result<ExecutionLock, String> {
    let directory = execution_root(state).join("locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{runner_id}.lock"));
    match tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
    {
        Ok(_) => Ok(ExecutionLock { path }),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = tokio::fs::metadata(&path)
                .await
                .map_err(|error| error.to_string())?;
            let modified = metadata.modified().map_err(|error| error.to_string())?;
            if SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default()
                > StdDuration::from_secs(3_900)
            {
                tokio::fs::remove_file(&path)
                    .await
                    .map_err(|error| error.to_string())?;
                return Box::pin(acquire_lock(state, runner_id)).await;
            }
            Err("这条 sealed-holdout runner 正在执行或等待结果".to_string())
        }
        Err(error) => Err(error.to_string()),
    }
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_gate_requires_every_preregistered_threshold() {
        let contract = &super::super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementations::tests::record().implementation_contract;
        assert!(metric_passes(
            contract,
            0.06,
            0.04,
            Some(0.1),
            0.6,
            Some(1.0)
        ));
        assert!(!metric_passes(
            contract,
            0.049,
            0.04,
            Some(0.1),
            0.6,
            Some(1.0)
        ));
        assert!(!metric_passes(
            contract,
            0.06,
            0.051,
            Some(0.1),
            0.6,
            Some(1.0)
        ));
    }

    #[test]
    fn failed_attempt_consumes_claim_and_never_authorizes_downstream() {
        let authorization = super::super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_first_execution_authorizations::tests::approved_authorization();
        let claim = new_claim(
            &authorization,
            "stage71-executor",
            authorization.review.submitted_at,
        )
        .unwrap();
        let result = new_failed_result(
            &claim,
            claim.claimed_at,
            1,
            &EvaluationFailure::before_access("synthetic failure"),
        )
        .unwrap();
        assert_eq!(result.status, HistoricalOutcomeSealedHoldoutEvaluationExecutionAttemptStatus::FailedAuthorizationConsumed);
        assert!(!result.sealed_holdout_labels_accessed);
        assert!(!result.official_candidate_selection_completed);
        assert!(!result.trading_authorized);
    }

    #[test]
    fn claim_tamper_to_other_target_or_feedback_fails_closed() {
        let authorization = super::super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_first_execution_authorizations::tests::approved_authorization();
        let mut claim = new_claim(
            &authorization,
            "stage71-executor",
            authorization.review.submitted_at,
        )
        .unwrap();
        claim.other_target_read_allowed = true;
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        assert!(validate_claim(&claim).is_err());
    }
}
