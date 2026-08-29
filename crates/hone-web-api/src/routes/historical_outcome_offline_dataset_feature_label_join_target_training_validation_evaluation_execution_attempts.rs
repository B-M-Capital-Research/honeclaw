//! Claim-first, one-shot validation-evaluation execution attempts.
//!
//! One exact, current and unexpired Stage 62 authorization is consumed by an
//! immutable claim before any validation feature or label is projected into
//! the evaluation worker. The worker receives only validation rows, the exact
//! nine independently reproduced train-only model candidates and the frozen
//! Stage 59 statistical contract. Sealed-holdout rows and labels never enter
//! the worker input.
//!
//! A successful attempt emits a content-addressed, untrusted per-target metric
//! envelope for a later independent validator. It does not make an official
//! model selection, write a model or metric store, define reward, simulate a
//! portfolio, generate an order, access a broker or trade. Success, failure and
//! interruption all consume the authorization.

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
    HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    HistoricalOutcomeFeatureLabelJoinTargetVisibility,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::{
    HistoricalOutcomeTrainingBoostedStump, HistoricalOutcomeTrainingExecutionAttemptClaim,
    HistoricalOutcomeTrainingExecutionAttemptResult, HistoricalOutcomeTrainingTargetModel,
    HistoricalOutcomeTrainingTargetModelBody,
    HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    HistoricalOutcomeTrainingUntrustedModelArtifact,
    completed_training_execution_attempt_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::HistoricalOutcomeTrainingExperimentAlgorithm;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copies::HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_output_validations::{
    IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    independently_validated_training_store_copies_for_training_registration_review,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_first_execution_authorizations::{
    ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    exact_first_execution_authorization_for_validation_evaluation_audit,
    execution_attempt_eligible_validation_evaluation_authorizations,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_implementations::HistoricalOutcomeValidationEvaluationImplementationContract;
use super::historical_outcome_offline_dataset_transformation_execution_attempts::HistoricalOutcomeOfflineDatasetTransformationSplit;
use super::historical_outcome_offline_datasets::{
    HistoricalOutcomeOfflineDataset, exact_current_historical_outcome_offline_dataset_for_transformation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-execution-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-execution-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str = "hone-untrusted-validation-evaluation-envelope-v1";
const EXECUTION_POLICY_VERSION: &str =
    "hone-validation-evaluation-claim-first-one-shot-frozen-statistics-v1";
const ISOLATION_BACKEND: &str = "hone-in-process-capability-limited-validation-evaluation-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const EXPECTED_ARTIFACT_COUNT: usize = 9;
const EXPECTED_METRIC_COUNT: usize = 81;
const EXPECTED_HYPOTHESIS_COUNT: usize = 54;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];
const MAX_ERROR_CHARS: usize = 1_200;
const NUMERIC_EPSILON: f64 = 1e-12;

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeHistoricalOutcomeValidationEvaluationOnceRequest {
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
    expected_candidate_set_sha256: String,
    expected_upstream_validation_sha256: String,
    expected_upstream_output_sha256: String,
    claim_first_create_once_and_failure_consumes_confirmed: bool,
    exact_validation_features_labels_and_nine_candidates_only_confirmed: bool,
    frozen_metrics_component_bootstrap_and_holm_confirmed: bool,
    no_seed_shopping_tuning_composite_or_global_claim_confirmed: bool,
    validation_only_no_training_update_and_sealed_holdout_hidden_confirmed: bool,
    untrusted_content_addressed_output_and_independent_validation_confirmed: bool,
    no_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationExecutionAttemptClaim {
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
    pub candidate_set_sha256: String,
    pub upstream_validation_sha256: String,
    pub upstream_output_sha256: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub isolation_backend: String,
    pub claim_first_create_once_and_failure_consumes_confirmed: bool,
    pub exact_validation_features_labels_and_nine_candidates_only_confirmed: bool,
    pub frozen_metrics_component_bootstrap_and_holm_confirmed: bool,
    pub no_seed_shopping_tuning_composite_or_global_claim_confirmed: bool,
    pub validation_only_no_training_update_and_sealed_holdout_hidden_confirmed: bool,
    pub untrusted_content_addressed_output_and_independent_validation_confirmed: bool,
    pub no_store_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub validation_feature_read_allowed: bool,
    pub validation_label_read_allowed: bool,
    pub evaluation_authorized: bool,
    pub sealed_holdout_feature_read_allowed: bool,
    pub sealed_holdout_label_read_allowed: bool,
    pub training_or_preprocessing_update_allowed: bool,
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
pub(crate) struct HistoricalOutcomeValidationEvaluationMetric {
    pub algorithm: HistoricalOutcomeTrainingExperimentAlgorithm,
    pub random_seed: u64,
    pub target_id: String,
    pub validation_row_count: usize,
    pub independent_component_count: usize,
    pub mae_f64_bits_hex: String,
    pub zero_baseline_mae_f64_bits_hex: String,
    pub relative_mae_improvement_f64_bits_hex: String,
    pub component_block_bootstrap_p_value_f64_bits_hex: Option<String>,
    pub holm_adjusted_p_value_f64_bits_hex: Option<String>,
    pub spearman_f64_bits_hex: Option<String>,
    pub directional_accuracy_f64_bits_hex: String,
    pub calibration_slope_f64_bits_hex: Option<String>,
    pub evidence_status: String,
    pub all_preregistered_thresholds_passed: bool,
    pub official_model_selection_metric: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationPerTargetRecommendation {
    pub target_id: String,
    pub status: String,
    pub recommended_algorithm: Option<HistoricalOutcomeTrainingExperimentAlgorithm>,
    pub three_seed_median_mae_f64_bits_hex: Option<String>,
    pub rationale: String,
    pub all_three_seeds_passed: bool,
    pub official_selection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationUntrustedEnvelope {
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
    pub upstream_validation_sha256: String,
    pub upstream_output_sha256: String,
    pub training_store_dataset_sha256: String,
    pub candidate_set_sha256: String,
    pub rows_sha256: String,
    pub excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
    pub validation_projection_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_order: Vec<String>,
    pub validation_row_count: usize,
    pub independent_component_count: usize,
    pub exact_artifact_count: usize,
    pub exact_metric_count: usize,
    pub exact_candidate_hypothesis_count: usize,
    pub metrics: Vec<HistoricalOutcomeValidationEvaluationMetric>,
    pub per_target_recommendations:
        Vec<HistoricalOutcomeValidationEvaluationPerTargetRecommendation>,
    pub validation_features_accessed: bool,
    pub validation_labels_accessed: bool,
    pub validation_evaluation_completed: bool,
    pub sealed_holdout_features_accessed: bool,
    pub sealed_holdout_labels_accessed: bool,
    pub training_or_preprocessing_updated: bool,
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
pub(crate) enum HistoricalOutcomeValidationEvaluationExecutionAttemptStatus {
    CompletedWithUntrustedValidationEvaluation,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: HistoricalOutcomeValidationEvaluationExecutionAttemptStatus,
    pub exit_code: i32,
    pub output_sha256: Option<String>,
    pub output_bytes: usize,
    pub bounded_error: Option<String>,
    pub untrusted_evaluation_envelope:
        Option<HistoricalOutcomeValidationEvaluationUntrustedEnvelope>,
    pub ephemeral_directory_removed: bool,
    pub validation_features_accessed: bool,
    pub validation_labels_accessed: bool,
    pub evaluation_completed: bool,
    pub sealed_holdout_features_accessed: bool,
    pub sealed_holdout_labels_accessed: bool,
    pub training_or_preprocessing_updated: bool,
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
pub(crate) struct HistoricalOutcomeValidationEvaluationExecutionAttemptItem {
    pub claim: HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
    pub result: Option<HistoricalOutcomeValidationEvaluationExecutionAttemptResult>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub isolation_backend: String,
    pub invocation_endpoint_available: bool,
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_evaluation_envelope_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
    pub attempts: Vec<HistoricalOutcomeValidationEvaluationExecutionAttemptItem>,
    pub sealed_holdout_access_authorized: bool,
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
pub(crate) struct HistoricalOutcomeValidationEvaluationExecutionAttemptReadinessSummary {
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_evaluation_envelope_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt {
    pub claim: HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
    pub result: HistoricalOutcomeValidationEvaluationExecutionAttemptResult,
}

#[derive(Debug)]
struct EvaluationFailure {
    message: String,
    ephemeral_directory_removed: bool,
    validation_inputs_accessed: bool,
}

impl EvaluationFailure {
    #[cfg(test)]
    fn before_staging(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral_directory_removed: true,
            validation_inputs_accessed: false,
        }
    }

    fn after_validation_access(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral_directory_removed: true,
            validation_inputs_accessed: true,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ValidationRow {
    pub(crate) dataset_entry_id: String,
    pub(crate) component_id: String,
    pub(crate) features: Vec<Option<f64>>,
    pub(crate) targets: Vec<f64>,
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedValidationData {
    pub(crate) feature_order: Vec<String>,
    pub(crate) target_order: Vec<String>,
    pub(crate) rows: Vec<ValidationRow>,
    pub(crate) projection_sha256: String,
}

pub(crate) async fn handle_get_historical_outcome_validation_evaluation_execution_attempts(
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
            warn!(%error, "historical outcome validation evaluation execution registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果 validation 评估执行尝试暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_historical_outcome_validation_evaluation_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeHistoricalOutcomeValidationEvaluationOnceRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
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

pub(crate) async fn historical_outcome_validation_evaluation_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeValidationEvaluationExecutionAttemptReadinessSummary, String> {
    let registry = build_registry(state, Utc::now()).await?;
    Ok(
        HistoricalOutcomeValidationEvaluationExecutionAttemptReadinessSummary {
            invocation_eligible_authorization_count: registry
                .invocation_eligible_authorization_count,
            claim_count: registry.claim_count,
            completed_attempt_count: registry.completed_attempt_count,
            failed_attempt_count: registry.failed_attempt_count,
            untrusted_evaluation_envelope_count: registry.untrusted_evaluation_envelope_count,
            independent_output_validation_eligible_count: registry
                .independent_output_validation_eligible_count,
            execution_status: registry.execution_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn completed_validation_evaluation_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let claims = claims
        .into_iter()
        .map(|claim| (claim.attempt_id.clone(), claim))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    for result in results {
        let Some(claim) = claims.get(&result.attempt_id).cloned() else {
            return Err("validation 评估结果缺少不可变 claim".to_string());
        };
        validate_result(&result, &claim)?;
        if result.status
            == HistoricalOutcomeValidationEvaluationExecutionAttemptStatus::CompletedWithUntrustedValidationEvaluation
        {
            output.push(CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt {
                claim,
                result,
            });
        }
    }
    output.sort_by(|left, right| left.claim.attempt_id.cmp(&right.claim.attempt_id));
    Ok(output)
}

#[allow(dead_code)]
pub(crate) async fn completed_validation_evaluation_execution_attempt_for_independent_validation(
    state: &AppState,
    attempt_id: &str,
) -> Result<Option<CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt>, String> {
    Ok(
        completed_validation_evaluation_execution_attempts_for_independent_validation(state)
            .await?
            .into_iter()
            .find(|attempt| attempt.claim.attempt_id == attempt_id),
    )
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry, String> {
    let eligible =
        execution_attempt_eligible_validation_evaluation_authorizations(state, now).await?;
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .iter()
        .map(|result| (result.attempt_id.as_str(), result))
        .collect::<BTreeMap<_, _>>();
    let attempts = claims
        .iter()
        .map(
            |claim| HistoricalOutcomeValidationEvaluationExecutionAttemptItem {
                claim: claim.clone(),
                result: result_index
                    .get(claim.attempt_id.as_str())
                    .cloned()
                    .cloned(),
            },
        )
        .collect::<Vec<_>>();
    let completed_attempt_count = results
        .iter()
        .filter(|result| {
            result.status
                == HistoricalOutcomeValidationEvaluationExecutionAttemptStatus::CompletedWithUntrustedValidationEvaluation
        })
        .count();
    let failed_attempt_count = results.len().saturating_sub(completed_attempt_count);
    let untrusted_evaluation_envelope_count = results
        .iter()
        .filter(|result| result.untrusted_evaluation_envelope.is_some())
        .count();
    let execution_status = if !eligible.is_empty() {
        "authorized_for_claim_first_one_shot_validation_evaluation"
    } else if claims
        .iter()
        .any(|claim| !result_index.contains_key(claim.attempt_id.as_str()))
    {
        "authorization_consumed_result_pending"
    } else if completed_attempt_count > 0 {
        "untrusted_validation_evaluation_output_pending_independent_validation"
    } else if failed_attempt_count > 0 {
        "validation_evaluation_failed_authorization_consumed"
    } else {
        "waiting_for_unexpired_stage_62_authorization"
    }
    .to_string();
    Ok(HistoricalOutcomeValidationEvaluationExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        invocation_endpoint_available: true,
        invocation_eligible_authorization_count: eligible.len(),
        claim_count: claims.len(),
        completed_attempt_count,
        failed_attempt_count,
        untrusted_evaluation_envelope_count,
        independent_output_validation_eligible_count: completed_attempt_count,
        execution_status,
        attempts,
        sealed_holdout_access_authorized: false,
        official_candidate_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 63 只允许精确 Stage 62 授权在 claim-first 后消费一次：validation 特征/标签仅投影给固定评估 worker，sealed holdout 永久隐藏；成功输出仍是不可信逐目标指标与建议，必须另经独立校验，不写模型/指标库或产生投资执行权限。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    admin_id: &str,
    isolated_runner_id: &str,
    request: InvokeHistoricalOutcomeValidationEvaluationOnceRequest,
) -> Result<HistoricalOutcomeValidationEvaluationExecutionAttemptResult, String> {
    if !valid_id(isolated_runner_id) {
        return Err("validation 评估隔离 runner ID 无效".to_string());
    }
    verify_invocation_confirmations(&request)?;
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let authorization =
        execution_attempt_eligible_validation_evaluation_authorizations(state, Utc::now())
            .await?
            .into_iter()
            .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
            .ok_or_else(|| "当前没有这条 validation 评估 runner 的未过期一次性授权".to_string())?;
    verify_request(&authorization, &request)?;
    let (existing_claims, _) = read_attempt_records(state).await?;
    if existing_claims.iter().any(|claim| {
        claim.isolated_runner_id == isolated_runner_id
            || claim.authorization_review_id == authorization.review.review_id
    }) {
        return Err("这条 validation 评估 runner 或首次执行授权已经消费，不得重放".to_string());
    }
    let runtime_artifact_sha256 = reverify_current_binary_sha256().await?;
    if runtime_artifact_sha256 != authorization.runner.runner_artifact_sha256 {
        return Err("当前运行制品与独立复核的 validation 评估 runner SHA-256 不一致".to_string());
    }
    let training_attempt = exact_training_attempt(&authorization, state).await?;
    let training_envelope = training_attempt
        .1
        .untrusted_artifact_envelope
        .as_ref()
        .ok_or_else(|| "Stage 57 训练输出缺少九候选 envelope".to_string())?;
    let validated_copy = exact_training_store_copy(&authorization, state).await?;
    let dataset = validated_copy
        .copied_dataset
        .attempt
        .training_store_dataset
        .as_ref()
        .ok_or_else(|| "独立校验训练副本缺少数据集".to_string())?;
    verify_exact_inputs(
        &authorization,
        &training_attempt.0,
        &training_attempt.1,
        training_envelope,
        dataset,
    )?;
    let claim = new_claim(&authorization, admin_id, Utc::now())?;
    write_immutable_json(
        &claim_directory(state, isolated_runner_id).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    // The immutable claim is deliberately persisted before the raw outcome dataset is
    // reopened. The host-side label broker may see the full encrypted-at-rest dataset,
    // but the fixed evaluation worker below receives validation rows only.
    let execution = match exact_current_historical_outcome_offline_dataset_for_transformation(
        state,
        &dataset.dataset_id,
        &dataset.dataset_content_sha256,
        &dataset.dataset_manifest_sha256,
        &dataset.candidate_set_sha256,
    )
    .await
    {
        Ok(Some(raw_dataset)) => prepare_validation_projection(
            dataset,
            &raw_dataset,
            training_envelope,
            &authorization.runner.implementation.implementation_contract,
        )
        .and_then(|prepared| {
            evaluate_fixed_contract(&authorization, &claim, training_envelope, prepared)
        })
        .map_err(EvaluationFailure::after_validation_access),
        Ok(None) => Err(EvaluationFailure::after_validation_access(
            "validation label broker 无法重开精确当前原始结果数据集",
        )),
        Err(error) => Err(EvaluationFailure::after_validation_access(format!(
            "validation label broker 重开精确当前原始结果数据集失败：{error}"
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
    request: &InvokeHistoricalOutcomeValidationEvaluationOnceRequest,
) -> Result<(), String> {
    if !request.claim_first_create_once_and_failure_consumes_confirmed
        || !request.exact_validation_features_labels_and_nine_candidates_only_confirmed
        || !request.frozen_metrics_component_bootstrap_and_holm_confirmed
        || !request.no_seed_shopping_tuning_composite_or_global_claim_confirmed
        || !request.validation_only_no_training_update_and_sealed_holdout_hidden_confirmed
        || !request.untrusted_content_addressed_output_and_independent_validation_confirmed
        || !request.no_store_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err(
            "必须确认一次性消费、精确 validation 输入、冻结统计、holdout 隔离和零投资执行权限"
                .to_string(),
        );
    }
    Ok(())
}

fn verify_request(
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    request: &InvokeHistoricalOutcomeValidationEvaluationOnceRequest,
) -> Result<(), String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
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
        || request.expected_candidate_set_sha256 != contract.candidate_set_sha256
        || request.expected_upstream_validation_sha256
            != implementation.upstream_validation.validation_sha256
        || request.expected_upstream_output_sha256
            != implementation.upstream_validation.output_sha256
    {
        return Err(
            "授权、runner、评估实现、候选集合或完整上游绑定已经变化，请刷新后重试".to_string(),
        );
    }
    Ok(())
}

pub(crate) async fn exact_training_attempt(
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    state: &AppState,
) -> Result<
    (
        HistoricalOutcomeTrainingExecutionAttemptClaim,
        HistoricalOutcomeTrainingExecutionAttemptResult,
    ),
    String,
> {
    let validation = &authorization.runner.implementation.upstream_validation;
    let attempt = completed_training_execution_attempt_for_independent_validation(
        state,
        &validation.attempt_id,
    )
    .await?
    .ok_or_else(|| "Stage 57 训练执行产物已不存在".to_string())?;
    if attempt.claim.claim_sha256 != validation.claim_sha256
        || attempt.result.result_sha256 != validation.result_sha256
        || attempt.result.output_sha256.as_deref() != Some(validation.output_sha256.as_str())
    {
        return Err("Stage 57–58 训练产物绑定已经漂移".to_string());
    }
    Ok((attempt.claim, attempt.result))
}

async fn exact_training_store_copy(
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
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
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    training_claim: &HistoricalOutcomeTrainingExecutionAttemptClaim,
    training_result: &HistoricalOutcomeTrainingExecutionAttemptResult,
    envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
) -> Result<(), String> {
    let implementation = &authorization.runner.implementation;
    let contract = &implementation.implementation_contract;
    let validation = &implementation.upstream_validation;
    let exact = training_claim.claim_sha256 == validation.claim_sha256
        && training_result.result_sha256 == validation.result_sha256
        && training_result.output_sha256.as_deref() == Some(validation.output_sha256.as_str())
        && envelope.training_store_dataset_sha256 == contract.training_store_dataset_sha256
        && dataset.training_store_dataset_sha256 == contract.training_store_dataset_sha256
        && envelope.rows_sha256 == validation.rows_sha256
        && envelope.excluded_rows_sha256 == validation.excluded_rows_sha256
        && envelope.target_commitments_sha256 == validation.target_commitments_sha256
        && envelope.feature_order.len() == EXPECTED_FEATURE_COUNT
        && envelope.target_order == contract.target_vector_order
        && envelope.model_artifacts.len() == EXPECTED_ARTIFACT_COUNT
        && dataset.validation_targets_withheld
        && dataset.sealed_holdout_targets_withheld
        && dataset.rows.iter().all(|row| match row.target_visibility {
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets => {
                row.split == HistoricalOutcomeOfflineDatasetTransformationSplit::Train
                    && row.target_vector.is_some()
            }
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld => {
                row.split == HistoricalOutcomeOfflineDatasetTransformationSplit::Validation
                    && row.target_vector.is_none()
            }
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld => {
                row.split == HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
                    && row.target_vector.is_none()
            }
        });
    if !exact {
        return Err("validation 评估输入、训练候选或 holdout 隔离与冻结合同不一致".to_string());
    }
    Ok(())
}

fn prepare_validation_projection(
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    raw_dataset: &HistoricalOutcomeOfflineDataset,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
) -> Result<PreparedValidationData, String> {
    if raw_dataset.dataset_id != dataset.dataset_id
        || raw_dataset.dataset_content_sha256 != dataset.dataset_content_sha256
        || raw_dataset.manifest_sha256 != dataset.dataset_manifest_sha256
        || raw_dataset.candidate_set_sha256 != dataset.candidate_set_sha256
        || training_envelope.feature_order.len() != EXPECTED_FEATURE_COUNT
        || training_envelope.preprocessing.len() != EXPECTED_FEATURE_COUNT
        || training_envelope.target_order.len() != EXPECTED_TARGET_COUNT
        || training_envelope.target_order != contract.target_vector_order
    {
        return Err("validation label broker 的数据集、特征、预处理或目标顺序漂移".to_string());
    }
    let validation_rows = dataset
        .rows
        .iter()
        .filter(|row| {
            row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
        })
        .collect::<Vec<_>>();
    if validation_rows.is_empty() {
        return Err("validation 分区没有可评估行".to_string());
    }
    let allowed_ids = validation_rows
        .iter()
        .map(|row| row.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    let sealed_ids = dataset
        .rows
        .iter()
        .filter(|row| {
            row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
        })
        .map(|row| row.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    if !allowed_ids.is_disjoint(&sealed_ids) {
        return Err("validation 与 sealed holdout entry 集合重叠".to_string());
    }
    let raw_index = raw_dataset
        .entries
        .iter()
        .filter(|entry| allowed_ids.contains(entry.entry_id.as_str()))
        .map(|entry| (entry.entry_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    if raw_index.len() != allowed_ids.len() {
        return Err("validation label broker 缺少精确原始结果条目".to_string());
    }
    let mut rows = Vec::with_capacity(validation_rows.len());
    for row in validation_rows {
        let raw = raw_index
            .get(row.dataset_entry_id.as_str())
            .ok_or_else(|| "validation 行缺少原始结果".to_string())?;
        if raw.entry_sha256 != row.dataset_entry_sha256
            || raw.decision_available_at != row.decision_available_at
            || raw.raw_validated_metrics.len() != REQUIRED_HORIZONS.len()
        {
            return Err(format!(
                "validation 条目 {} 的点时绑定漂移",
                row.dataset_entry_id
            ));
        }
        let features = parse_and_preprocess_features(
            row,
            &training_envelope.feature_order,
            &training_envelope.preprocessing,
        )?;
        let targets = project_targets(raw, &training_envelope.target_order)?;
        rows.push(ValidationRow {
            dataset_entry_id: row.dataset_entry_id.clone(),
            component_id: row.component_id.clone(),
            features,
            targets,
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
        feature_order: training_envelope.feature_order.clone(),
        target_order: training_envelope.target_order.clone(),
        rows,
        projection_sha256,
    })
}

pub(crate) fn parse_and_preprocess_features(
    row: &HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    feature_order: &[String],
    preprocessing: &[super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::HistoricalOutcomeTrainingFeaturePreprocessingRecord],
) -> Result<Vec<Option<f64>>, String> {
    let index = row
        .feature_records
        .iter()
        .map(|feature| (feature.feature_id.as_str(), feature))
        .collect::<BTreeMap<_, _>>();
    let preprocessing = preprocessing
        .iter()
        .map(|value| (value.feature_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    if index.len() != EXPECTED_FEATURE_COUNT || preprocessing.len() != EXPECTED_FEATURE_COUNT {
        return Err(format!(
            "validation 条目 {} 的特征或预处理不完整",
            row.dataset_entry_id
        ));
    }
    feature_order
        .iter()
        .map(|feature_id| {
            let feature = index
                .get(feature_id.as_str())
                .ok_or_else(|| format!("validation 行缺少特征 {feature_id}"))?;
            let prep = preprocessing
                .get(feature_id.as_str())
                .ok_or_else(|| format!("训练产物缺少预处理 {feature_id}"))?;
            if feature.available_at_utc > row.decision_available_at || !prep.missingness_preserved {
                return Err(format!("特征 {feature_id} 的点时或缺失语义漂移"));
            }
            if feature.is_missing {
                if feature.value.is_some() {
                    return Err(format!("缺失特征 {feature_id} 携带值"));
                }
                Ok(None)
            } else {
                let value = feature
                    .value
                    .as_deref()
                    .ok_or_else(|| format!("非缺失特征 {feature_id} 没有值"))?
                    .parse::<f64>()
                    .map_err(|_| format!("特征 {feature_id} 不是有限值"))?;
                let mean = parse_f64_hex(&prep.train_mean_f64_bits_hex)?;
                let scale = parse_f64_hex(&prep.train_scale_f64_bits_hex)?;
                if !value.is_finite() || scale <= 0.0 {
                    return Err(format!("特征 {feature_id} 或预处理尺度无效"));
                }
                Ok(Some((value - mean) / scale))
            }
        })
        .collect()
}

pub(crate) fn project_targets(
    entry: &super::historical_outcome_offline_datasets::HistoricalOutcomeOfflineDatasetEntry,
    target_order: &[String],
) -> Result<Vec<f64>, String> {
    let metrics = entry
        .raw_validated_metrics
        .iter()
        .map(|metric| (metric.horizon_market_sessions, metric))
        .collect::<BTreeMap<_, _>>();
    if metrics.len() != REQUIRED_HORIZONS.len() {
        return Err("validation 原始结果窗口不完整".to_string());
    }
    target_order
        .iter()
        .map(|target_id| {
            let (metric_name, horizon) = parse_target_id(target_id)?;
            let metric = metrics
                .get(&horizon)
                .ok_or_else(|| format!("validation 缺少 {horizon} 日结果"))?;
            let value = match metric_name {
                "asset_return" => metric.asset_return,
                "excess_return" => metric.excess_return,
                "asset_max_drawdown" => metric.asset_max_drawdown,
                _ => return Err(format!("未知 validation 目标 {target_id}")),
            };
            if !value.is_finite() {
                return Err(format!("validation 目标 {target_id} 不是有限值"));
            }
            Ok(value)
        })
        .collect()
}

fn parse_target_id(target_id: &str) -> Result<(&str, u16), String> {
    for metric in ["asset_return", "excess_return", "asset_max_drawdown"] {
        if let Some(value) = target_id
            .strip_prefix(&format!("{metric}_"))
            .and_then(|value| value.strip_suffix("_continuous"))
        {
            let horizon = value
                .parse::<u16>()
                .map_err(|_| format!("目标 horizon 无效：{target_id}"))?;
            if REQUIRED_HORIZONS.contains(&horizon) {
                return Ok((metric, horizon));
            }
        }
    }
    Err(format!("目标 ID 不属于冻结九目标：{target_id}"))
}

fn evaluate_fixed_contract(
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    claim: &HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    data: PreparedValidationData,
) -> Result<HistoricalOutcomeValidationEvaluationUntrustedEnvelope, String> {
    let contract = &authorization.runner.implementation.implementation_contract;
    validate_frozen_contract(contract, training_envelope, &data)?;
    let components = data
        .rows
        .iter()
        .map(|row| row.component_id.clone())
        .collect::<BTreeSet<_>>();
    let mut predictions = BTreeMap::new();
    for artifact in &training_envelope.model_artifacts {
        validate_candidate_artifact(artifact, contract)?;
        predictions.insert(
            (
                algorithm_id(&artifact.algorithm).to_string(),
                artifact.random_seed,
            ),
            predict_artifact(artifact, &data)?,
        );
    }
    if predictions.len() != EXPECTED_ARTIFACT_COUNT {
        return Err("validation 评估不是精确九候选".to_string());
    }
    let mut metrics = Vec::with_capacity(EXPECTED_METRIC_COUNT);
    let mut hypothesis_positions = Vec::with_capacity(EXPECTED_HYPOTHESIS_COUNT);
    let mut hypothesis_p_values = Vec::with_capacity(EXPECTED_HYPOTHESIS_COUNT);
    for algorithm in [
        HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline,
        HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
        HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
    ] {
        for seed in EXPECTED_SEEDS {
            let predicted = predictions
                .get(&(algorithm_id(&algorithm).to_string(), seed))
                .ok_or_else(|| "validation 评估缺少算法/种子预测".to_string())?;
            let baseline = predictions
                .get(&("frozen_zero_prediction_baseline".to_string(), seed))
                .ok_or_else(|| "validation 评估缺少同种子零预测基线".to_string())?;
            for target in 0..data.target_order.len() {
                let actual = data
                    .rows
                    .iter()
                    .map(|row| row.targets[target])
                    .collect::<Vec<_>>();
                let candidate = predicted.iter().map(|row| row[target]).collect::<Vec<_>>();
                let zero = baseline.iter().map(|row| row[target]).collect::<Vec<_>>();
                let mae = mean_absolute_error(&actual, &candidate)?;
                let zero_mae = mean_absolute_error(&actual, &zero)?;
                let improvement = if zero_mae.abs() <= NUMERIC_EPSILON {
                    0.0
                } else {
                    (zero_mae - mae) / zero_mae
                };
                let is_benchmark = algorithm
                    == HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline;
                let p_value = if is_benchmark {
                    None
                } else {
                    Some(component_block_bootstrap_p_value(
                        &data.rows,
                        &actual,
                        &candidate,
                        &zero,
                        contract.bootstrap_replications,
                        contract.bootstrap_random_seed
                            ^ seed
                            ^ u64::try_from(target).unwrap_or_default(),
                    )?)
                };
                let position = metrics.len();
                if let Some(value) = p_value {
                    hypothesis_positions.push(position);
                    hypothesis_p_values.push(value);
                }
                metrics.push(HistoricalOutcomeValidationEvaluationMetric {
                    algorithm: algorithm.clone(),
                    random_seed: seed,
                    target_id: data.target_order[target].clone(),
                    validation_row_count: data.rows.len(),
                    independent_component_count: components.len(),
                    mae_f64_bits_hex: f64_hex(mae),
                    zero_baseline_mae_f64_bits_hex: f64_hex(zero_mae),
                    relative_mae_improvement_f64_bits_hex: f64_hex(improvement),
                    component_block_bootstrap_p_value_f64_bits_hex: p_value.map(f64_hex),
                    holm_adjusted_p_value_f64_bits_hex: None,
                    spearman_f64_bits_hex: spearman(&actual, &candidate).map(f64_hex),
                    directional_accuracy_f64_bits_hex: f64_hex(directional_accuracy(
                        &actual, &candidate,
                    )),
                    calibration_slope_f64_bits_hex: calibration_slope(&actual, &candidate)
                        .map(f64_hex),
                    evidence_status: if data.rows.len() < contract.minimum_validation_rows as usize
                        || components.len() < contract.minimum_independent_components as usize
                    {
                        "insufficient_evidence".to_string()
                    } else if is_benchmark {
                        "benchmark_only".to_string()
                    } else {
                        "pending_holm_adjustment".to_string()
                    },
                    all_preregistered_thresholds_passed: false,
                    official_model_selection_metric: false,
                });
            }
        }
    }
    if metrics.len() != EXPECTED_METRIC_COUNT
        || hypothesis_p_values.len() != EXPECTED_HYPOTHESIS_COUNT
        || contract.exact_candidate_hypothesis_count as usize != EXPECTED_HYPOTHESIS_COUNT
    {
        return Err("逐目标逐种子指标或 54 项候选假设数量漂移".to_string());
    }
    let adjusted = holm_adjusted_p_values(&hypothesis_p_values);
    for (position, adjusted) in hypothesis_positions.into_iter().zip(adjusted) {
        let metric = &mut metrics[position];
        metric.holm_adjusted_p_value_f64_bits_hex = Some(f64_hex(adjusted));
        if metric.evidence_status != "insufficient_evidence" {
            metric.all_preregistered_thresholds_passed = metric_passes(metric, contract)?;
            metric.evidence_status = if metric.all_preregistered_thresholds_passed {
                "all_preregistered_thresholds_passed"
            } else {
                "one_or_more_preregistered_thresholds_failed"
            }
            .to_string();
        }
    }
    let recommendations = per_target_recommendations(&metrics, contract, &data.target_order)?;
    let mut output = HistoricalOutcomeValidationEvaluationUntrustedEnvelope {
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
        upstream_validation_sha256: claim.upstream_validation_sha256.clone(),
        upstream_output_sha256: claim.upstream_output_sha256.clone(),
        training_store_dataset_sha256: claim.training_store_dataset_sha256.clone(),
        candidate_set_sha256: claim.candidate_set_sha256.clone(),
        rows_sha256: claim.rows_sha256.clone(),
        excluded_rows_sha256: claim.excluded_rows_sha256.clone(),
        target_commitments_sha256: claim.target_commitments_sha256.clone(),
        validation_projection_sha256: data.projection_sha256,
        feature_order_sha256: hash_serializable(&data.feature_order)?,
        preprocessing_sha256: hash_serializable(&training_envelope.preprocessing)?,
        target_order: data.target_order,
        validation_row_count: data.rows.len(),
        independent_component_count: components.len(),
        exact_artifact_count: training_envelope.model_artifacts.len(),
        exact_metric_count: metrics.len(),
        exact_candidate_hypothesis_count: EXPECTED_HYPOTHESIS_COUNT,
        metrics,
        per_target_recommendations: recommendations,
        validation_features_accessed: true,
        validation_labels_accessed: true,
        validation_evaluation_completed: true,
        sealed_holdout_features_accessed: false,
        sealed_holdout_labels_accessed: false,
        training_or_preprocessing_updated: false,
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
    // Keep the mutable binding explicit so future additions cannot bypass the
    // final validation by returning a separately constructed object.
    output.output_is_untrusted = true;
    Ok(output)
}

fn validate_frozen_contract(
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    data: &PreparedValidationData,
) -> Result<(), String> {
    let feature_order_sha256 = hash_serializable(&training_envelope.feature_order)?;
    let preprocessing_sha256 = hash_serializable(&training_envelope.preprocessing)?;
    let exact = contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == EXPECTED_TARGET_COUNT
        && contract.exact_artifact_count == EXPECTED_ARTIFACT_COUNT
        && contract.exact_random_seeds == EXPECTED_SEEDS
        && contract.bootstrap_unit == "official_split_component_id"
        && contract.bootstrap_replications == 10_000
        && contract.family_wise_error_correction
            == "holm_bonferroni_across_54_candidate_target_seed_tests"
        && contract.exact_candidate_hypothesis_count as usize == EXPECTED_HYPOTHESIS_COUNT
        && contract.all_three_seeds_must_pass
        && contract.no_composite_score_or_global_model_validity_claim
        && !contract.sealed_holdout_features_access_allowed
        && !contract.sealed_holdout_labels_access_allowed
        && !contract.training_or_preprocessing_update_allowed
        && !contract.hyperparameter_or_threshold_tuning_allowed
        && !contract.candidate_selection_allowed
        && !contract.model_store_write_allowed
        && !contract.metric_store_write_allowed
        && feature_order_sha256 == contract.feature_order_sha256
        && preprocessing_sha256 == contract.preprocessing_sha256
        && training_envelope.feature_order == data.feature_order
        && training_envelope.target_order == data.target_order
        && training_envelope.model_artifacts.len() == EXPECTED_ARTIFACT_COUNT;
    if !exact {
        return Err("Stage 59 冻结统计合同或 Stage 57 九候选已经漂移".to_string());
    }
    Ok(())
}

fn validate_candidate_artifact(
    artifact: &HistoricalOutcomeTrainingUntrustedModelArtifact,
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
) -> Result<(), String> {
    let binding = contract.candidate_bindings.iter().find(|binding| {
        binding.algorithm_id == algorithm_id(&artifact.algorithm)
            && binding.random_seed == artifact.random_seed
    });
    if binding.is_none_or(|binding| {
        binding.artifact_sha256 != artifact.artifact_sha256
            || binding.exact_target_model_count != EXPECTED_TARGET_COUNT
    }) || artifact.exact_feature_count != EXPECTED_FEATURE_COUNT
        || artifact.exact_target_count != EXPECTED_TARGET_COUNT
        || artifact.target_models.len() != EXPECTED_TARGET_COUNT
        || artifact.validation_selected
        || artifact.sealed_holdout_accessed
    {
        return Err("候选模型工件与 Stage 59 绑定不一致或已经越权".to_string());
    }
    Ok(())
}

pub(crate) fn predict_artifact(
    artifact: &HistoricalOutcomeTrainingUntrustedModelArtifact,
    data: &PreparedValidationData,
) -> Result<Vec<Vec<f64>>, String> {
    let models = artifact
        .target_models
        .iter()
        .map(|model| (model.target_id.as_str(), model))
        .collect::<BTreeMap<_, _>>();
    if models.len() != data.target_order.len() {
        return Err("候选模型目标重复或缺失".to_string());
    }
    data.rows
        .iter()
        .map(|row| {
            data.target_order
                .iter()
                .map(|target_id| {
                    predict_target(
                        models
                            .get(target_id.as_str())
                            .ok_or_else(|| format!("候选缺少目标模型 {target_id}"))?,
                        &data.feature_order,
                        &row.features,
                    )
                })
                .collect()
        })
        .collect()
}

pub(crate) fn predict_target(
    model: &HistoricalOutcomeTrainingTargetModel,
    feature_order: &[String],
    features: &[Option<f64>],
) -> Result<f64, String> {
    let feature_index = feature_order
        .iter()
        .enumerate()
        .map(|(index, value)| (value.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let value = match &model.model {
        HistoricalOutcomeTrainingTargetModelBody::FrozenZeroPrediction {
            prediction_f64_bits_hex,
        } => parse_f64_hex(prediction_f64_bits_hex)?,
        HistoricalOutcomeTrainingTargetModelBody::RidgeRegression {
            intercept_f64_bits_hex,
            coefficients,
            ..
        } => {
            if coefficients.len() != feature_order.len() {
                return Err("ridge 系数不是精确 65 项".to_string());
            }
            let mut value = parse_f64_hex(intercept_f64_bits_hex)?;
            for coefficient in coefficients {
                let index = *feature_index
                    .get(coefficient.feature_id.as_str())
                    .ok_or_else(|| "ridge 系数引用未知特征".to_string())?;
                value += match features[index] {
                    Some(feature) => {
                        feature
                            * parse_f64_hex(&coefficient.observed_value_coefficient_f64_bits_hex)?
                    }
                    None => parse_f64_hex(&coefficient.missing_indicator_coefficient_f64_bits_hex)?,
                };
            }
            value
        }
        HistoricalOutcomeTrainingTargetModelBody::GradientBoostedRegression {
            learning_rate_f64_bits_hex,
            rounds,
            ..
        } => {
            let learning_rate = parse_f64_hex(learning_rate_f64_bits_hex)?;
            let mut prediction = 0.0;
            for stump in rounds {
                prediction += learning_rate * predict_stump(stump, &feature_index, features)?;
            }
            prediction
        }
    };
    if !value.is_finite() {
        return Err("候选模型预测不是有限值".to_string());
    }
    Ok(value)
}

fn predict_stump(
    stump: &HistoricalOutcomeTrainingBoostedStump,
    feature_index: &BTreeMap<&str, usize>,
    features: &[Option<f64>],
) -> Result<f64, String> {
    let missing = parse_f64_hex(&stump.missing_leaf_f64_bits_hex)?;
    let (Some(feature_id), Some(threshold_hex)) =
        (&stump.feature_id, &stump.threshold_f64_bits_hex)
    else {
        return Ok(missing);
    };
    let index = *feature_index
        .get(feature_id.as_str())
        .ok_or_else(|| "boosted stump 引用未知特征".to_string())?;
    let threshold = parse_f64_hex(threshold_hex)?;
    match features[index] {
        Some(value) if value <= threshold => parse_f64_hex(&stump.low_leaf_f64_bits_hex),
        Some(_) => parse_f64_hex(&stump.high_leaf_f64_bits_hex),
        None => Ok(missing),
    }
}

pub(crate) fn component_block_bootstrap_p_value(
    rows: &[ValidationRow],
    actual: &[f64],
    candidate: &[f64],
    baseline: &[f64],
    replications: u32,
    seed: u64,
) -> Result<f64, String> {
    if rows.len() != actual.len()
        || actual.len() != candidate.len()
        || actual.len() != baseline.len()
        || rows.is_empty()
        || replications == 0
    {
        return Err("component bootstrap 输入无效".to_string());
    }
    let mut blocks = BTreeMap::<&str, Vec<f64>>::new();
    for (index, row) in rows.iter().enumerate() {
        blocks.entry(row.component_id.as_str()).or_default().push(
            (actual[index] - baseline[index]).abs() - (actual[index] - candidate[index]).abs(),
        );
    }
    let block_means = blocks
        .values()
        .map(|values| values.iter().sum::<f64>() / values.len() as f64)
        .collect::<Vec<_>>();
    if block_means.is_empty() {
        return Err("component bootstrap 没有独立 block".to_string());
    }
    let mut rng = DeterministicRng::new(seed);
    let mut non_positive = 0_u32;
    for _ in 0..replications {
        let mean = (0..block_means.len())
            .map(|_| block_means[rng.index(block_means.len())])
            .sum::<f64>()
            / block_means.len() as f64;
        if mean <= 0.0 {
            non_positive += 1;
        }
    }
    Ok((f64::from(non_positive) + 1.0) / (f64::from(replications) + 1.0))
}

#[derive(Debug, Clone)]
struct DeterministicRng(u64);

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    fn index(&mut self, length: usize) -> usize {
        (self.next() % length as u64) as usize
    }
}

pub(crate) fn holm_adjusted_p_values(values: &[f64]) -> Vec<f64> {
    let mut ordered = values.iter().copied().enumerate().collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.1.total_cmp(&right.1).then(left.0.cmp(&right.0)));
    let mut adjusted = vec![1.0; values.len()];
    let mut running = 0.0_f64;
    for (rank, (index, value)) in ordered.into_iter().enumerate() {
        let multiplier = values.len().saturating_sub(rank) as f64;
        running = running.max((value * multiplier).min(1.0));
        adjusted[index] = running;
    }
    adjusted
}

fn metric_passes(
    metric: &HistoricalOutcomeValidationEvaluationMetric,
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
) -> Result<bool, String> {
    let improvement = parse_f64_hex(&metric.relative_mae_improvement_f64_bits_hex)?;
    let q = metric
        .holm_adjusted_p_value_f64_bits_hex
        .as_deref()
        .map(parse_f64_hex)
        .transpose()?
        .unwrap_or(1.0);
    let spearman = metric
        .spearman_f64_bits_hex
        .as_deref()
        .map(parse_f64_hex)
        .transpose()?;
    let directional = parse_f64_hex(&metric.directional_accuracy_f64_bits_hex)?;
    let calibration = metric
        .calibration_slope_f64_bits_hex
        .as_deref()
        .map(parse_f64_hex)
        .transpose()?;
    Ok(
        improvement >= f64::from(contract.minimum_relative_mae_improvement_ppm) / 1_000_000.0
            && q <= f64::from(contract.family_wise_alpha_millionths) / 1_000_000.0
            && spearman.is_some_and(|value| {
                value >= f64::from(contract.minimum_spearman_millionths) / 1_000_000.0
            })
            && directional
                >= f64::from(contract.minimum_directional_accuracy_millionths) / 1_000_000.0
            && calibration.is_some_and(|value| {
                value >= f64::from(contract.minimum_calibration_slope_millionths) / 1_000_000.0
                    && value
                        <= f64::from(contract.maximum_calibration_slope_millionths) / 1_000_000.0
            }),
    )
}

fn per_target_recommendations(
    metrics: &[HistoricalOutcomeValidationEvaluationMetric],
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
    target_order: &[String],
) -> Result<Vec<HistoricalOutcomeValidationEvaluationPerTargetRecommendation>, String> {
    target_order
        .iter()
        .map(|target_id| {
            let target_metrics = metrics
                .iter()
                .filter(|metric| &metric.target_id == target_id)
                .collect::<Vec<_>>();
            if target_metrics.len() != 9 {
                return Err(format!("目标 {target_id} 不是精确九项指标"));
            }
            if target_metrics
                .iter()
                .any(|metric| metric.evidence_status == "insufficient_evidence")
            {
                return Ok(HistoricalOutcomeValidationEvaluationPerTargetRecommendation {
                    target_id: target_id.clone(),
                    status: "insufficient_evidence".to_string(),
                    recommended_algorithm: None,
                    three_seed_median_mae_f64_bits_hex: None,
                    rationale: "validation 行数或独立 component 未达到预注册门槛；不得选模或声明有效。".to_string(),
                    all_three_seeds_passed: false,
                    official_selection: false,
                });
            }
            let mut candidates = Vec::new();
            for algorithm in [
                HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
                HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
            ] {
                let values = target_metrics
                    .iter()
                    .filter(|metric| metric.algorithm == algorithm)
                    .collect::<Vec<_>>();
                if values.len() == EXPECTED_SEEDS.len()
                    && values
                        .iter()
                        .all(|metric| metric.all_preregistered_thresholds_passed)
                {
                    let mut maes = values
                        .iter()
                        .map(|metric| parse_f64_hex(&metric.mae_f64_bits_hex))
                        .collect::<Result<Vec<_>, _>>()?;
                    maes.sort_by(f64::total_cmp);
                    candidates.push((algorithm, maes[1]));
                }
            }
            if candidates.is_empty() {
                return Ok(HistoricalOutcomeValidationEvaluationPerTargetRecommendation {
                    target_id: target_id.clone(),
                    status: "no_candidate_passed_all_three_seeds".to_string(),
                    recommended_algorithm: None,
                    three_seed_median_mae_f64_bits_hex: None,
                    rationale: "ridge 与 boosted 均未在三个冻结种子上同时通过全部预注册门槛。".to_string(),
                    all_three_seeds_passed: false,
                    official_selection: false,
                });
            }
            candidates.sort_by(|left, right| {
                left.1.total_cmp(&right.1).then_with(|| {
                    let preferred = contract.tie_break_preferred_algorithm_id.as_str();
                    let left_rank = usize::from(algorithm_id(&left.0) != preferred);
                    let right_rank = usize::from(algorithm_id(&right.0) != preferred);
                    left_rank.cmp(&right_rank)
                })
            });
            let selected = candidates.remove(0);
            Ok(HistoricalOutcomeValidationEvaluationPerTargetRecommendation {
                target_id: target_id.clone(),
                status: "untrusted_per_target_recommendation_pending_independent_validation"
                    .to_string(),
                recommended_algorithm: Some(selected.0),
                three_seed_median_mae_f64_bits_hex: Some(f64_hex(selected.1)),
                rationale: "仅按预注册三种子全部通过、validation MAE 中位数和固定 ridge 平局规则形成不可信建议；尚非正式选择。".to_string(),
                all_three_seeds_passed: true,
                official_selection: false,
            })
        })
        .collect()
}

pub(crate) fn mean_absolute_error(actual: &[f64], predicted: &[f64]) -> Result<f64, String> {
    if actual.len() != predicted.len() || actual.is_empty() {
        return Err("MAE 输入无效".to_string());
    }
    Ok(actual
        .iter()
        .zip(predicted)
        .map(|(actual, predicted)| (actual - predicted).abs())
        .sum::<f64>()
        / actual.len() as f64)
}

pub(crate) fn directional_accuracy(actual: &[f64], predicted: &[f64]) -> f64 {
    let direction = |value: f64| {
        if value > NUMERIC_EPSILON {
            1_i8
        } else if value < -NUMERIC_EPSILON {
            -1_i8
        } else {
            0_i8
        }
    };
    actual
        .iter()
        .zip(predicted)
        .filter(|(actual, predicted)| direction(**actual) == direction(**predicted))
        .count() as f64
        / actual.len() as f64
}

pub(crate) fn spearman(left: &[f64], right: &[f64]) -> Option<f64> {
    pearson(&average_ranks(left), &average_ranks(right))
}

fn average_ranks(values: &[f64]) -> Vec<f64> {
    let mut order = values.iter().copied().enumerate().collect::<Vec<_>>();
    order.sort_by(|left, right| left.1.total_cmp(&right.1).then(left.0.cmp(&right.0)));
    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < order.len() {
        let mut end = start + 1;
        while end < order.len() && order[end].1.to_bits() == order[start].1.to_bits() {
            end += 1;
        }
        let rank = (start + end - 1) as f64 / 2.0 + 1.0;
        for index in start..end {
            ranks[order[index].0] = rank;
        }
        start = end;
    }
    ranks
}

fn pearson(left: &[f64], right: &[f64]) -> Option<f64> {
    if left.len() != right.len() || left.len() < 2 {
        return None;
    }
    let left_mean = left.iter().sum::<f64>() / left.len() as f64;
    let right_mean = right.iter().sum::<f64>() / right.len() as f64;
    let covariance = left
        .iter()
        .zip(right)
        .map(|(left, right)| (left - left_mean) * (right - right_mean))
        .sum::<f64>();
    let left_variance = left
        .iter()
        .map(|value| (value - left_mean).powi(2))
        .sum::<f64>();
    let right_variance = right
        .iter()
        .map(|value| (value - right_mean).powi(2))
        .sum::<f64>();
    let denominator = (left_variance * right_variance).sqrt();
    (denominator > NUMERIC_EPSILON).then_some(covariance / denominator)
}

pub(crate) fn calibration_slope(actual: &[f64], predicted: &[f64]) -> Option<f64> {
    if actual.len() != predicted.len() || actual.len() < 2 {
        return None;
    }
    let predicted_mean = predicted.iter().sum::<f64>() / predicted.len() as f64;
    let actual_mean = actual.iter().sum::<f64>() / actual.len() as f64;
    let denominator = predicted
        .iter()
        .map(|value| (value - predicted_mean).powi(2))
        .sum::<f64>();
    if denominator <= NUMERIC_EPSILON {
        return None;
    }
    Some(
        predicted
            .iter()
            .zip(actual)
            .map(|(predicted, actual)| (predicted - predicted_mean) * (actual - actual_mean))
            .sum::<f64>()
            / denominator,
    )
}

async fn execute_ephemeral_output(
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    claim: &HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
    envelope: HistoricalOutcomeValidationEvaluationUntrustedEnvelope,
    duration_millis: u64,
) -> Result<HistoricalOutcomeValidationEvaluationExecutionAttemptResult, String> {
    let bytes = match serde_json::to_vec(&envelope) {
        Ok(bytes) => bytes,
        Err(error) => {
            return new_failed_result(
                claim,
                Utc::now(),
                duration_millis,
                &EvaluationFailure::after_validation_access(format!(
                    "未验证 validation 评估输出序列化失败：{error}"
                )),
            );
        }
    };
    if bytes.len() > authorization.runner.runner_contract.maximum_output_bytes as usize {
        return new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure::after_validation_access(
                "未验证 validation 评估输出超过 runner 上限",
            ),
        );
    }
    let directory =
        std::env::temp_dir().join(format!("hone-validation-evaluation-{}", Uuid::new_v4()));
    if let Err(error) = tokio::fs::create_dir(&directory).await {
        return new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure::after_validation_access(format!(
                "一次性 validation 评估目录创建失败：{error}"
            )),
        );
    }
    let path = directory.join("untrusted-validation-evaluation.json");
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
            return Err("一次性 validation 评估输出回读不一致".to_string());
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
                message: format!("一次性 validation 评估目录清理失败：{error}"),
                ephemeral_directory_removed: removed,
                validation_inputs_accessed: true,
            },
        ),
        (Err(error), _, removed) => new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure {
                message: error,
                ephemeral_directory_removed: removed,
                validation_inputs_accessed: true,
            },
        ),
        (Ok(_), Ok(()), false) => new_failed_result(
            claim,
            Utc::now(),
            duration_millis,
            &EvaluationFailure {
                message: "一次性 validation 评估目录清理状态无法确认".to_string(),
                ephemeral_directory_removed: false,
                validation_inputs_accessed: true,
            },
        ),
    }
}

fn new_claim(
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    invoked_by: &str,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeValidationEvaluationExecutionAttemptClaim, String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
    let validation = &implementation.upstream_validation;
    let mut claim = HistoricalOutcomeValidationEvaluationExecutionAttemptClaim {
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
        candidate_set_sha256: contract.candidate_set_sha256.clone(),
        upstream_validation_sha256: validation.validation_sha256.clone(),
        upstream_output_sha256: validation.output_sha256.clone(),
        training_store_dataset_sha256: validation.training_store_dataset_sha256.clone(),
        rows_sha256: validation.rows_sha256.clone(),
        excluded_rows_sha256: validation.excluded_rows_sha256.clone(),
        target_commitments_sha256: validation.target_commitments_sha256.clone(),
        claimed_at,
        invoked_by: invoked_by.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        claim_first_create_once_and_failure_consumes_confirmed: true,
        exact_validation_features_labels_and_nine_candidates_only_confirmed: true,
        frozen_metrics_component_bootstrap_and_holm_confirmed: true,
        no_seed_shopping_tuning_composite_or_global_claim_confirmed: true,
        validation_only_no_training_update_and_sealed_holdout_hidden_confirmed: true,
        untrusted_content_addressed_output_and_independent_validation_confirmed: true,
        no_store_reward_shadow_order_broker_or_trading_confirmed: true,
        authorization_consumed: true,
        invocation_started: false,
        validation_feature_read_allowed: true,
        validation_label_read_allowed: true,
        evaluation_authorized: true,
        sealed_holdout_feature_read_allowed: false,
        sealed_holdout_label_read_allowed: false,
        training_or_preprocessing_update_allowed: false,
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
    claim: &HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    envelope: HistoricalOutcomeValidationEvaluationUntrustedEnvelope,
    output_sha256: String,
    output_bytes: usize,
) -> Result<HistoricalOutcomeValidationEvaluationExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeValidationEvaluationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeValidationEvaluationExecutionAttemptStatus::CompletedWithUntrustedValidationEvaluation,
        exit_code: 0,
        output_sha256: Some(output_sha256),
        output_bytes,
        bounded_error: None,
        untrusted_evaluation_envelope: Some(envelope),
        ephemeral_directory_removed: true,
        validation_features_accessed: true,
        validation_labels_accessed: true,
        evaluation_completed: true,
        sealed_holdout_features_accessed: false,
        sealed_holdout_labels_accessed: false,
        training_or_preprocessing_updated: false,
        independent_output_validation_completed: false,
        official_candidate_selection_completed: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn new_failed_result(
    claim: &HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    failure: &EvaluationFailure,
) -> Result<HistoricalOutcomeValidationEvaluationExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeValidationEvaluationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status:
            HistoricalOutcomeValidationEvaluationExecutionAttemptStatus::FailedAuthorizationConsumed,
        exit_code: 1,
        output_sha256: None,
        output_bytes: 0,
        bounded_error: Some(failure.message.chars().take(MAX_ERROR_CHARS).collect()),
        untrusted_evaluation_envelope: None,
        ephemeral_directory_removed: failure.ephemeral_directory_removed,
        validation_features_accessed: failure.validation_inputs_accessed,
        validation_labels_accessed: failure.validation_inputs_accessed,
        evaluation_completed: false,
        sealed_holdout_features_accessed: false,
        sealed_holdout_labels_accessed: false,
        training_or_preprocessing_updated: false,
        independent_output_validation_completed: false,
        official_candidate_selection_completed: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn validate_claim(
    claim: &HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
) -> Result<(), String> {
    let boundary_closed = claim.authorization_consumed
        && !claim.invocation_started
        && claim.validation_feature_read_allowed
        && claim.validation_label_read_allowed
        && claim.evaluation_authorized
        && !claim.sealed_holdout_feature_read_allowed
        && !claim.sealed_holdout_label_read_allowed
        && !claim.training_or_preprocessing_update_allowed
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
        || !claim.claim_first_create_once_and_failure_consumes_confirmed
        || !claim.exact_validation_features_labels_and_nine_candidates_only_confirmed
        || !claim.frozen_metrics_component_bootstrap_and_holm_confirmed
        || !claim.no_seed_shopping_tuning_composite_or_global_claim_confirmed
        || !claim.validation_only_no_training_update_and_sealed_holdout_hidden_confirmed
        || !claim.untrusted_content_addressed_output_and_independent_validation_confirmed
        || !claim.no_store_reward_shadow_order_broker_or_trading_confirmed
        || !boundary_closed
    {
        return Err("validation 评估 claim 无效或越权".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeValidationEvaluationExecutionAttemptResult,
    claim: &HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
) -> Result<(), String> {
    let success = result.status
        == HistoricalOutcomeValidationEvaluationExecutionAttemptStatus::CompletedWithUntrustedValidationEvaluation;
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
        || success != result.untrusted_evaluation_envelope.is_some()
        || success != (result.output_bytes > 0)
        || (success && !result.validation_features_accessed)
        || result.validation_labels_accessed != result.validation_features_accessed
        || success != result.evaluation_completed
        || result.sealed_holdout_features_accessed
        || result.sealed_holdout_labels_accessed
        || result.training_or_preprocessing_updated
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
        return Err("validation 评估执行结果无效或越权".to_string());
    }
    if let Some(envelope) = &result.untrusted_evaluation_envelope {
        if hash_serializable(envelope)? != result.output_sha256.clone().unwrap_or_default() {
            return Err("validation 评估 envelope 与输出摘要不一致".to_string());
        }
    }
    Ok(())
}

fn validate_envelope(
    envelope: &HistoricalOutcomeValidationEvaluationUntrustedEnvelope,
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
) -> Result<(), String> {
    let target_ids = envelope
        .metrics
        .iter()
        .map(|metric| metric.target_id.as_str())
        .collect::<BTreeSet<_>>();
    let metric_keys = envelope
        .metrics
        .iter()
        .map(|metric| {
            (
                algorithm_id(&metric.algorithm),
                metric.random_seed,
                metric.target_id.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    let expected_metric_keys = [
        "frozen_zero_prediction_baseline",
        "ridge_multi_target_regression",
        "gradient_boosted_multi_target_regression",
    ]
    .into_iter()
    .flat_map(|algorithm| {
        EXPECTED_SEEDS.into_iter().flat_map(move |seed| {
            contract
                .target_vector_order
                .iter()
                .map(move |target| (algorithm, seed, target.as_str()))
        })
    })
    .collect::<BTreeSet<_>>();
    let recommendation_targets = envelope
        .per_target_recommendations
        .iter()
        .map(|recommendation| recommendation.target_id.as_str())
        .collect::<BTreeSet<_>>();
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION
        || !valid_sha256(&envelope.validation_projection_sha256)
        || !valid_sha256(&envelope.feature_order_sha256)
        || !valid_sha256(&envelope.preprocessing_sha256)
        || envelope.target_order.len() != EXPECTED_TARGET_COUNT
        || envelope.target_order != contract.target_vector_order
        || envelope.feature_order_sha256 != contract.feature_order_sha256
        || envelope.preprocessing_sha256 != contract.preprocessing_sha256
        || target_ids.len() != EXPECTED_TARGET_COUNT
        || metric_keys != expected_metric_keys
        || recommendation_targets != target_ids
        || envelope.exact_artifact_count != EXPECTED_ARTIFACT_COUNT
        || envelope.exact_metric_count != EXPECTED_METRIC_COUNT
        || envelope.metrics.len() != EXPECTED_METRIC_COUNT
        || envelope.exact_candidate_hypothesis_count != EXPECTED_HYPOTHESIS_COUNT
        || envelope.per_target_recommendations.len() != EXPECTED_TARGET_COUNT
        || !envelope.validation_features_accessed
        || !envelope.validation_labels_accessed
        || !envelope.validation_evaluation_completed
        || envelope.sealed_holdout_features_accessed
        || envelope.sealed_holdout_labels_accessed
        || envelope.training_or_preprocessing_updated
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
            !EXPECTED_SEEDS.contains(&metric.random_seed)
                || !valid_f64_hex(&metric.mae_f64_bits_hex)
                || !valid_f64_hex(&metric.zero_baseline_mae_f64_bits_hex)
                || !valid_f64_hex(&metric.relative_mae_improvement_f64_bits_hex)
                || metric.official_model_selection_metric
        })
        || envelope
            .per_target_recommendations
            .iter()
            .any(|recommendation| recommendation.official_selection)
        || contract.no_composite_score_or_global_model_validity_claim != true
    {
        return Err("未验证 validation 评估 envelope 无效或越权".to_string());
    }
    Ok(())
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<HistoricalOutcomeValidationEvaluationExecutionAttemptClaim>,
        Vec<HistoricalOutcomeValidationEvaluationExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<HistoricalOutcomeValidationEvaluationExecutionAttemptClaim>(
        &execution_root(state).join("claims"),
    )
    .await?;
    let results = read_nested_json::<HistoricalOutcomeValidationEvaluationExecutionAttemptResult>(
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
            return Err("validation 评估 claim 重复，拒绝继续".to_string());
        }
        exact_first_execution_authorization_for_validation_evaluation_audit(
            state,
            &claim.isolated_runner_id,
            &claim.authorization_review_id,
            &claim.authorization_review_sha256,
        )
        .await?
        .ok_or_else(|| "validation 评估 claim 的授权链不再可重开".to_string())?;
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
            .ok_or_else(|| "validation 评估结果缺少 claim".to_string())?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_attempts.insert(result.attempt_id.clone())
        {
            return Err("validation 评估结果重复，拒绝继续".to_string());
        }
    }
    Ok((claims, results))
}

async fn read_nested_json<T>(root: &Path) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
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
                let bytes = tokio::fs::read(&path)
                    .await
                    .map_err(|error| error.to_string())?;
                values.push(serde_json::from_slice(&bytes).map_err(|error| error.to_string())?);
            }
        }
    }
    Ok(values)
}

async fn reverify_current_binary_sha256() -> Result<String, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let bytes = tokio::fs::read(executable)
        .await
        .map_err(|error| error.to_string())?;
    Ok(hash_bytes(&bytes))
}

fn claim_fingerprint(
    claim: &HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
) -> Result<String, String> {
    fingerprint_without(claim, &["attempt_id", "claim_sha256"])
}

fn result_fingerprint(
    result: &HistoricalOutcomeValidationEvaluationExecutionAttemptResult,
) -> Result<String, String> {
    fingerprint_without(result, &["result_id", "result_sha256"])
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
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(crate) fn f64_hex(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

pub(crate) fn parse_f64_hex(value: &str) -> Result<f64, String> {
    let bits = u64::from_str_radix(value, 16).map_err(|_| "f64 位模式无效".to_string())?;
    let value = f64::from_bits(bits);
    if !value.is_finite() {
        return Err("f64 位模式不是有限值".to_string());
    }
    Ok(value)
}

fn valid_f64_hex(value: &str) -> bool {
    value.len() == 16 && parse_f64_hex(value).is_ok()
}

pub(crate) fn algorithm_id(
    algorithm: &HistoricalOutcomeTrainingExperimentAlgorithm,
) -> &'static str {
    match algorithm {
        HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline => {
            "frozen_zero_prediction_baseline"
        }
        HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression => {
            "ridge_multi_target_regression"
        }
        HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression => {
            "gradient_boosted_multi_target_regression"
        }
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn execution_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-validation-evaluation-execution-attempts")
}

fn claim_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("claims").join(runner_id)
}

fn result_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("results").join(runner_id)
}

struct ValidationEvaluationExecutionLock {
    path: PathBuf,
}

impl Drop for ValidationEvaluationExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    runner_id: &str,
) -> Result<ValidationEvaluationExecutionLock, String> {
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
        Ok(_) => Ok(ValidationEvaluationExecutionLock { path }),
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
            Err("这条 validation 评估 runner 正在执行或等待结果，拒绝并发调用".to_string())
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
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(component_count: usize, rows_per_component: usize) -> Vec<ValidationRow> {
        (0..component_count)
            .flat_map(|component| {
                (0..rows_per_component).map(move |row| {
                    let value = (component * rows_per_component + row) as f64 / 100.0 + 0.1;
                    ValidationRow {
                        dataset_entry_id: format!("entry-{component:02}-{row:02}"),
                        component_id: format!("component-{component:02}"),
                        features: vec![Some(value); EXPECTED_FEATURE_COUNT],
                        targets: vec![value; EXPECTED_TARGET_COUNT],
                    }
                })
            })
            .collect()
    }

    #[test]
    fn component_bootstrap_is_deterministic_and_rewards_lower_error() {
        let rows = rows(20, 5);
        let actual = rows.iter().map(|row| row.targets[0]).collect::<Vec<_>>();
        let candidate = actual.iter().map(|value| value * 0.95).collect::<Vec<_>>();
        let baseline = vec![0.0; actual.len()];
        let first =
            component_block_bootstrap_p_value(&rows, &actual, &candidate, &baseline, 10_000, 59)
                .unwrap();
        let second =
            component_block_bootstrap_p_value(&rows, &actual, &candidate, &baseline, 10_000, 59)
                .unwrap();
        assert_eq!(first.to_bits(), second.to_bits());
        assert!(first < 0.01);
    }

    #[test]
    fn component_bootstrap_preserves_whole_components() {
        let rows = rows(2, 2);
        let actual = vec![1.0, 1.0, -1.0, -1.0];
        let candidate = vec![0.0, 0.0, 0.0, 0.0];
        let baseline = vec![0.0, 0.0, 0.0, 0.0];
        let value =
            component_block_bootstrap_p_value(&rows, &actual, &candidate, &baseline, 100, 1)
                .unwrap();
        assert_eq!(value, 1.0);
    }

    #[test]
    fn holm_adjustment_is_monotone_and_family_wise() {
        let adjusted = holm_adjusted_p_values(&[0.001, 0.01, 0.04]);
        assert_eq!(adjusted, vec![0.003, 0.02, 0.04]);
    }

    #[test]
    fn target_parser_accepts_only_frozen_nine_targets() {
        assert_eq!(
            parse_target_id("excess_return_250_continuous").unwrap(),
            ("excess_return", 250)
        );
        assert!(parse_target_id("reward_250_continuous").is_err());
        assert!(parse_target_id("asset_return_30_continuous").is_err());
    }

    #[test]
    fn metric_helpers_preserve_degenerate_diagnostics() {
        let actual = [1.0, 2.0, 3.0];
        let constant = [0.0, 0.0, 0.0];
        assert!(spearman(&actual, &constant).is_none());
        assert!(calibration_slope(&actual, &constant).is_none());
        assert_eq!(directional_accuracy(&actual, &constant), 0.0);
    }

    #[test]
    fn claim_is_claim_first_and_keeps_downstream_closed() {
        let mut claim = synthetic_claim();
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        validate_claim(&claim).unwrap();
        assert!(claim.authorization_consumed);
        assert!(!claim.invocation_started);
        assert!(claim.validation_label_read_allowed);
        assert!(!claim.sealed_holdout_label_read_allowed);
        assert!(!claim.official_candidate_selection_allowed);
        assert!(!claim.trading_allowed);
    }

    #[test]
    fn claim_tamper_to_holdout_or_store_fails_closed() {
        let mut claim = synthetic_claim();
        claim.sealed_holdout_label_read_allowed = true;
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        assert!(validate_claim(&claim).is_err());
        claim.sealed_holdout_label_read_allowed = false;
        claim.model_store_write_allowed = true;
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        assert!(validate_claim(&claim).is_err());
    }

    #[test]
    fn failed_result_records_validation_access_boundary_and_consumes_claim() {
        let mut claim = synthetic_claim();
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        let result = new_failed_result(
            &claim,
            claim.claimed_at,
            1,
            &EvaluationFailure::before_staging("synthetic failure"),
        )
        .unwrap();
        assert_eq!(
            result.status,
            HistoricalOutcomeValidationEvaluationExecutionAttemptStatus::FailedAuthorizationConsumed
        );
        assert!(!result.validation_labels_accessed);
        assert!(!result.evaluation_completed);

        let accessed_result = new_failed_result(
            &claim,
            claim.claimed_at,
            2,
            &EvaluationFailure::after_validation_access("synthetic post-access failure"),
        )
        .unwrap();
        assert!(accessed_result.validation_features_accessed);
        assert!(accessed_result.validation_labels_accessed);
        assert!(!accessed_result.evaluation_completed);
    }

    #[test]
    fn result_tamper_to_selection_or_trading_fails_closed() {
        let mut claim = synthetic_claim();
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        let mut result = new_failed_result(
            &claim,
            claim.claimed_at,
            1,
            &EvaluationFailure::before_staging("synthetic failure"),
        )
        .unwrap();
        result.official_candidate_selection_completed = true;
        result.result_sha256 = result_fingerprint(&result).unwrap();
        result.result_id = result.result_sha256[..32].to_string();
        assert!(validate_result(&result, &claim).is_err());
        result.official_candidate_selection_completed = false;
        result.trading_authorized = true;
        result.result_sha256 = result_fingerprint(&result).unwrap();
        result.result_id = result.result_sha256[..32].to_string();
        assert!(validate_result(&result, &claim).is_err());
    }

    #[test]
    fn prediction_uses_frozen_zero_model_exactly() {
        let model = HistoricalOutcomeTrainingTargetModel {
            target_id: "asset_return_20_continuous".to_string(),
            model: HistoricalOutcomeTrainingTargetModelBody::FrozenZeroPrediction {
                prediction_f64_bits_hex: f64_hex(0.0),
            },
        };
        assert_eq!(
            predict_target(&model, &["feature".to_string()], &[Some(10.0)]).unwrap(),
            0.0
        );
    }

    fn synthetic_claim() -> HistoricalOutcomeValidationEvaluationExecutionAttemptClaim {
        HistoricalOutcomeValidationEvaluationExecutionAttemptClaim {
            schema_version: CLAIM_SCHEMA_VERSION.to_string(),
            execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
            attempt_id: String::new(),
            claim_sha256: String::new(),
            authorization_review_id: "1".repeat(32),
            authorization_review_sha256: "1".repeat(64),
            authorization_valid_until: DateTime::parse_from_rfc3339("2026-08-25T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            isolated_runner_id: "2".repeat(32),
            isolated_runner_spec_sha256: "2".repeat(64),
            runner_artifact_sha256: "3".repeat(64),
            runner_code_revision: "revision".to_string(),
            runner_contract_sha256: "4".repeat(64),
            implementation_id: "5".repeat(32),
            implementation_sha256: "5".repeat(64),
            implementation_contract_sha256: "6".repeat(64),
            implementation_review_id: "7".repeat(32),
            implementation_review_sha256: "7".repeat(64),
            implementation_independent_audit_sha256: "8".repeat(64),
            candidate_set_sha256: "9".repeat(64),
            upstream_validation_sha256: "a".repeat(64),
            upstream_output_sha256: "b".repeat(64),
            training_store_dataset_sha256: "c".repeat(64),
            rows_sha256: "d".repeat(64),
            excluded_rows_sha256: "e".repeat(64),
            target_commitments_sha256: "f".repeat(64),
            claimed_at: DateTime::parse_from_rfc3339("2026-08-24T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            invoked_by: "stage63-executor".to_string(),
            isolation_backend: ISOLATION_BACKEND.to_string(),
            claim_first_create_once_and_failure_consumes_confirmed: true,
            exact_validation_features_labels_and_nine_candidates_only_confirmed: true,
            frozen_metrics_component_bootstrap_and_holm_confirmed: true,
            no_seed_shopping_tuning_composite_or_global_claim_confirmed: true,
            validation_only_no_training_update_and_sealed_holdout_hidden_confirmed: true,
            untrusted_content_addressed_output_and_independent_validation_confirmed: true,
            no_store_reward_shadow_order_broker_or_trading_confirmed: true,
            authorization_consumed: true,
            invocation_started: false,
            validation_feature_read_allowed: true,
            validation_label_read_allowed: true,
            evaluation_authorized: true,
            sealed_holdout_feature_read_allowed: false,
            sealed_holdout_label_read_allowed: false,
            training_or_preprocessing_update_allowed: false,
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
        }
    }
}
