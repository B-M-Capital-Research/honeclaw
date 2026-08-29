//! Claim-first, one-shot execution of one exact Stage 56 training authorization.
//!
//! Stage 57 fits the frozen three-arm/three-seed suite only on the exact independently validated
//! training-store copy. Validation and sealed-holdout labels remain unavailable. Every invocation
//! consumes its authorization before fitting starts, and every successful output remains an
//! untrusted, content-addressed candidate that requires a separate independent validation gate.

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
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::{
    HistoricalOutcomeTrainingExperimentAlgorithm,
    HistoricalOutcomeTrainingExperimentSuiteSpecification,
    registered_training_experiment_suites_for_independent_review,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_first_execution_authorizations::{
    ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    exact_first_execution_authorization_for_training_audit,
    execution_attempt_eligible_training_authorizations,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copies::HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_output_validations::independently_validated_training_store_copies_for_training_registration_review;
use super::historical_outcome_offline_dataset_transformation_execution_attempts::HistoricalOutcomeOfflineDatasetTransformationSplit;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-training-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str = "hone-historical-outcome-training-execution-claim-v1";
const RESULT_SCHEMA_VERSION: &str = "hone-historical-outcome-training-execution-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str = "hone-untrusted-training-artifact-envelope-v1";
const EXECUTION_POLICY_VERSION: &str = "hone-training-claim-first-one-shot-train-only-fit-v1";
const ISOLATION_BACKEND: &str = "hone-in-process-capability-limited-training-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const MAX_ERROR_CHARS: usize = 1_200;
const RIDGE_LAMBDA: f64 = 0.01;
const NUMERIC_EPSILON: f64 = 1e-12;

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeHistoricalOutcomeTrainingOnceRequest {
    expected_first_execution_authorization_review_id: String,
    expected_first_execution_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_review_sha256: String,
    expected_suite_specification_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_rows_sha256: String,
    expected_excluded_rows_sha256: String,
    expected_target_commitments_sha256: String,
    claim_first_create_once_and_failure_consumes_confirmed: bool,
    exact_read_only_training_store_dataset_only_confirmed: bool,
    train_only_fit_and_explicit_missingness_preserved_confirmed: bool,
    validation_and_sealed_holdout_labels_remain_withheld_confirmed: bool,
    fixed_three_arm_three_seed_suite_confirmed: bool,
    untrusted_content_addressed_output_and_independent_validation_confirmed: bool,
    no_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeTrainingExecutionAttemptClaim {
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
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_review_sha256: String,
    pub suite_specification_sha256: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub isolation_backend: String,
    pub maximum_wall_clock_seconds: u32,
    pub maximum_memory_mib: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u16,
    pub maximum_output_bytes: u64,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub exact_training_store_read_allowed: bool,
    pub train_target_read_allowed: bool,
    pub validation_target_read_allowed: bool,
    pub sealed_holdout_target_read_allowed: bool,
    pub generic_training_store_read_allowed: bool,
    pub production_read_allowed: bool,
    pub production_write_allowed: bool,
    pub network_allowed: bool,
    pub external_tools_allowed: bool,
    pub child_process_spawned: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub reward_write_allowed: bool,
    pub shadow_write_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingFeaturePreprocessingRecord {
    pub feature_id: String,
    pub observed_train_count: usize,
    pub missing_train_count: usize,
    pub train_mean_f64_bits_hex: String,
    pub train_scale_f64_bits_hex: String,
    pub missingness_preserved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingRidgeCoefficient {
    pub feature_id: String,
    pub observed_value_coefficient_f64_bits_hex: String,
    pub missing_indicator_coefficient_f64_bits_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingBoostedStump {
    pub feature_id: Option<String>,
    pub threshold_f64_bits_hex: Option<String>,
    pub low_leaf_f64_bits_hex: String,
    pub high_leaf_f64_bits_hex: String,
    pub missing_leaf_f64_bits_hex: String,
    pub depth: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeTrainingTargetModelBody {
    FrozenZeroPrediction {
        prediction_f64_bits_hex: String,
    },
    RidgeRegression {
        intercept_f64_bits_hex: String,
        l2_regularization_f64_bits_hex: String,
        coefficients: Vec<HistoricalOutcomeTrainingRidgeCoefficient>,
    },
    GradientBoostedRegression {
        learning_rate_f64_bits_hex: String,
        maximum_tree_depth: u8,
        rounds: Vec<HistoricalOutcomeTrainingBoostedStump>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingTargetModel {
    pub target_id: String,
    pub model: HistoricalOutcomeTrainingTargetModelBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingUntrustedModelArtifact {
    pub artifact_sha256: String,
    pub algorithm: HistoricalOutcomeTrainingExperimentAlgorithm,
    pub random_seed: u64,
    pub train_row_count: usize,
    pub exact_feature_count: usize,
    pub exact_target_count: usize,
    pub target_models: Vec<HistoricalOutcomeTrainingTargetModel>,
    pub validation_selected: bool,
    pub sealed_holdout_accessed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingFitDiagnostic {
    pub algorithm: HistoricalOutcomeTrainingExperimentAlgorithm,
    pub random_seed: u64,
    pub target_id: String,
    pub split: String,
    pub mae_f64_bits_hex: String,
    pub spearman_f64_bits_hex: Option<String>,
    pub directional_accuracy_f64_bits_hex: String,
    pub calibration_slope_f64_bits_hex: Option<String>,
    pub model_selection_metric: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingUntrustedArtifactEnvelope {
    pub schema_version: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_review_sha256: String,
    pub suite_specification_sha256: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
    pub feature_order: Vec<String>,
    pub target_order: Vec<String>,
    pub preprocessing: Vec<HistoricalOutcomeTrainingFeaturePreprocessingRecord>,
    pub train_row_count: usize,
    pub validation_row_count_with_targets_withheld: usize,
    pub sealed_holdout_row_count_with_targets_withheld: usize,
    pub model_artifacts: Vec<HistoricalOutcomeTrainingUntrustedModelArtifact>,
    pub fit_diagnostics: Vec<HistoricalOutcomeTrainingFitDiagnostic>,
    pub train_only_fit_completed: bool,
    pub validation_labels_accessed: bool,
    pub validation_selection_completed: bool,
    pub sealed_holdout_labels_accessed: bool,
    pub output_is_untrusted: bool,
    pub independent_output_validation_completed: bool,
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
pub(crate) enum HistoricalOutcomeTrainingExecutionAttemptStatus {
    CompletedWithUntrustedTrainOnlyArtifacts,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeTrainingExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: HistoricalOutcomeTrainingExecutionAttemptStatus,
    pub exit_code: i32,
    pub output_sha256: Option<String>,
    pub output_bytes: usize,
    pub bounded_error: Option<String>,
    pub untrusted_artifact_envelope: Option<HistoricalOutcomeTrainingUntrustedArtifactEnvelope>,
    pub ephemeral_directory_removed: bool,
    pub independent_output_validation_completed: bool,
    pub validation_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingExecutionAttemptItem {
    pub claim: HistoricalOutcomeTrainingExecutionAttemptClaim,
    pub result: Option<HistoricalOutcomeTrainingExecutionAttemptResult>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub isolation_backend: String,
    pub invocation_endpoint_available: bool,
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_artifact_envelope_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
    pub attempts: Vec<HistoricalOutcomeTrainingExecutionAttemptItem>,
    pub validation_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeTrainingExecutionAttemptReadinessSummary {
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_artifact_envelope_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct CompletedHistoricalOutcomeTrainingExecutionAttempt {
    pub claim: HistoricalOutcomeTrainingExecutionAttemptClaim,
    pub result: HistoricalOutcomeTrainingExecutionAttemptResult,
}

#[derive(Debug)]
struct TrainingFailure {
    message: String,
    ephemeral_directory_removed: bool,
}

#[derive(Debug, Clone)]
struct PreparedTrainingData {
    feature_order: Vec<String>,
    target_order: Vec<String>,
    preprocessing: Vec<HistoricalOutcomeTrainingFeaturePreprocessingRecord>,
    train_features: Vec<Vec<Option<f64>>>,
    train_targets: Vec<Vec<f64>>,
    validation_count: usize,
    sealed_holdout_count: usize,
}

pub(crate) async fn handle_get_historical_outcome_training_execution_attempts(
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
            warn!(%error, "historical outcome training execution attempt registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "训练一次性执行尝试暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_historical_outcome_training_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeHistoricalOutcomeTrainingOnceRequest>,
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

pub(crate) async fn historical_outcome_training_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingExecutionAttemptReadinessSummary, String> {
    let registry = build_registry(state, Utc::now()).await?;
    Ok(HistoricalOutcomeTrainingExecutionAttemptReadinessSummary {
        invocation_eligible_authorization_count: registry.invocation_eligible_authorization_count,
        claim_count: registry.claim_count,
        completed_attempt_count: registry.completed_attempt_count,
        failed_attempt_count: registry.failed_attempt_count,
        untrusted_artifact_envelope_count: registry.untrusted_artifact_envelope_count,
        independent_output_validation_eligible_count: registry
            .independent_output_validation_eligible_count,
        execution_status: registry.execution_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn completed_training_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeTrainingExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let claims = claims
        .into_iter()
        .map(|claim| (claim.attempt_id.clone(), claim))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    for result in results {
        let Some(claim) = claims.get(&result.attempt_id).cloned() else {
            return Err("训练执行结果缺少不可变 claim".to_string());
        };
        validate_result(&result, &claim)?;
        if result.status
            == HistoricalOutcomeTrainingExecutionAttemptStatus::CompletedWithUntrustedTrainOnlyArtifacts
        {
            output.push(CompletedHistoricalOutcomeTrainingExecutionAttempt { claim, result });
        }
    }
    output.sort_by(|left, right| left.claim.attempt_id.cmp(&right.claim.attempt_id));
    Ok(output)
}

#[allow(dead_code)]
pub(crate) async fn completed_training_execution_attempt_for_independent_validation(
    state: &AppState,
    attempt_id: &str,
) -> Result<Option<CompletedHistoricalOutcomeTrainingExecutionAttempt>, String> {
    Ok(
        completed_training_execution_attempts_for_independent_validation(state)
            .await?
            .into_iter()
            .find(|attempt| attempt.claim.attempt_id == attempt_id),
    )
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<HistoricalOutcomeTrainingExecutionAttemptRegistry, String> {
    let eligible = execution_attempt_eligible_training_authorizations(state, now).await?;
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .iter()
        .map(|result| (result.attempt_id.as_str(), result))
        .collect::<BTreeMap<_, _>>();
    let attempts = claims
        .iter()
        .map(|claim| {
            let result = result_index
                .get(claim.attempt_id.as_str())
                .copied()
                .cloned();
            if let Some(value) = &result {
                validate_result(value, claim)?;
            }
            Ok(HistoricalOutcomeTrainingExecutionAttemptItem {
                claim: claim.clone(),
                result,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let completed_attempt_count = results
        .iter()
        .filter(|result| {
            result.status
                == HistoricalOutcomeTrainingExecutionAttemptStatus::CompletedWithUntrustedTrainOnlyArtifacts
        })
        .count();
    let failed_attempt_count = results.len().saturating_sub(completed_attempt_count);
    let untrusted_artifact_envelope_count = results
        .iter()
        .filter(|result| result.untrusted_artifact_envelope.is_some())
        .count();
    let independent_output_validation_eligible_count = completed_attempt_count;
    let consumed_authorizations = claims
        .iter()
        .map(|claim| claim.authorization_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let invocation_eligible_authorization_count = eligible
        .iter()
        .filter(|authorization| {
            !consumed_authorizations.contains(authorization.review.review_id.as_str())
        })
        .count();
    let execution_status = if independent_output_validation_eligible_count > 0 {
        "untrusted_train_only_output_pending_independent_validation"
    } else if claims.len() > results.len() {
        "claim_consumed_result_missing_fail_closed"
    } else if failed_attempt_count > 0 {
        "failed_authorization_consumed"
    } else if invocation_eligible_authorization_count > 0 {
        "ready_for_one_claim_first_train_only_attempt"
    } else {
        "waiting_unexpired_unconsumed_stage_56_authorization"
    };
    Ok(HistoricalOutcomeTrainingExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        invocation_endpoint_available: true,
        invocation_eligible_authorization_count,
        claim_count: claims.len(),
        completed_attempt_count,
        failed_attempt_count,
        untrusted_artifact_envelope_count,
        independent_output_validation_eligible_count,
        execution_status: execution_status.to_string(),
        attempts,
        validation_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 57 先不可逆写入 claim，再只读取精确绑定且独立校验的 training-store 副本；只用 train 标签拟合固定三臂三种子，validation 与 sealed holdout 标签继续隐藏。成功输出是待独立校验的内容寻址模型候选和 train-only 诊断，不做 validation 选模，不写模型/指标库、reward、影子仓位、订单、券商或交易。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    admin_id: &str,
    isolated_runner_id: &str,
    request: InvokeHistoricalOutcomeTrainingOnceRequest,
) -> Result<HistoricalOutcomeTrainingExecutionAttemptResult, String> {
    if !valid_id(isolated_runner_id) {
        return Err("训练隔离 runner ID 无效".to_string());
    }
    verify_invocation_confirmations(&request)?;
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let authorization = execution_attempt_eligible_training_authorizations(state, Utc::now())
        .await?
        .into_iter()
        .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "当前没有这条训练 runner 的未过期一次性授权".to_string())?;
    verify_request(&authorization, &request)?;
    let (existing_claims, _) = read_attempt_records(state).await?;
    if existing_claims.iter().any(|claim| {
        claim.isolated_runner_id == isolated_runner_id
            || claim.authorization_review_id == authorization.review.review_id
    }) {
        return Err("这条 runner 或首次执行授权已经消费，不得重放".to_string());
    }
    let runtime_artifact_sha256 = reverify_current_binary_sha256().await?;
    if runtime_artifact_sha256 != authorization.runner.runner_artifact_sha256 {
        return Err("当前运行制品与独立复核的训练 runner SHA-256 不一致".to_string());
    }
    let dataset = exact_training_store_dataset(&authorization, state).await?;
    verify_exact_dataset_binding(&authorization, &dataset)?;
    let suite = exact_training_suite(&authorization, state).await?;
    verify_fixed_suite(&suite)?;
    let prepared = prepare_training_data(&dataset, &suite)?;

    let claim = new_claim(&authorization, admin_id, Utc::now())?;
    write_immutable_json(
        &claim_directory(state, isolated_runner_id).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let execution = execute_fixed_training(&authorization, &suite, prepared).await;
    let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let result = match execution {
        Ok((envelope, output_sha256, output_bytes)) => new_success_result(
            &claim,
            Utc::now(),
            duration_millis,
            envelope,
            output_sha256,
            output_bytes,
        )?,
        Err(error) => new_failed_result(&claim, Utc::now(), duration_millis, &error)?,
    };
    write_immutable_json(
        &result_directory(state, isolated_runner_id).join(format!("{}.json", result.result_id)),
        &result,
    )
    .await?;
    Ok(result)
}

fn verify_invocation_confirmations(
    request: &InvokeHistoricalOutcomeTrainingOnceRequest,
) -> Result<(), String> {
    if !request.claim_first_create_once_and_failure_consumes_confirmed
        || !request.exact_read_only_training_store_dataset_only_confirmed
        || !request.train_only_fit_and_explicit_missingness_preserved_confirmed
        || !request.validation_and_sealed_holdout_labels_remain_withheld_confirmed
        || !request.fixed_three_arm_three_seed_suite_confirmed
        || !request.untrusted_content_addressed_output_and_independent_validation_confirmed
        || !request.no_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err(
            "必须确认一次性消费、精确数据、train-only 拟合、标签隔离和零投资执行权限".to_string(),
        );
    }
    Ok(())
}

fn verify_request(
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    request: &InvokeHistoricalOutcomeTrainingOnceRequest,
) -> Result<(), String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let registration_review = &implementation.approved_registration_review;
    if request.expected_first_execution_authorization_review_id != authorization.review.review_id
        || request.expected_first_execution_authorization_review_sha256
            != authorization.review.review_sha256
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != runner.runner_artifact_sha256
        || request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_review_sha256
            != runner.implementation_review.review_sha256
        || request.expected_suite_specification_sha256
            != registration_review.suite_specification_sha256
        || request.expected_training_store_dataset_sha256
            != registration_review.training_store_dataset_sha256
        || request.expected_rows_sha256 != registration_review.rows_sha256
        || request.expected_excluded_rows_sha256 != registration_review.excluded_rows_sha256
        || request.expected_target_commitments_sha256
            != registration_review.target_commitments_sha256
    {
        return Err("授权、runner、实现、训练副本或固定套件绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

async fn exact_training_store_dataset(
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset, String> {
    let expected = &authorization
        .runner
        .implementation
        .approved_registration_review
        .training_store_dataset_sha256;
    independently_validated_training_store_copies_for_training_registration_review(state)
        .await?
        .into_iter()
        .filter_map(|value| value.copied_dataset.attempt.training_store_dataset)
        .find(|dataset| &dataset.training_store_dataset_sha256 == expected)
        .ok_or_else(|| "精确绑定的独立校验训练副本不再可用".to_string())
}

async fn exact_training_suite(
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingExperimentSuiteSpecification, String> {
    let review = &authorization
        .runner
        .implementation
        .approved_registration_review;
    registered_training_experiment_suites_for_independent_review(state)
        .await?
        .into_iter()
        .filter_map(|registered| registered.attempt.registration)
        .find(|registration| {
            registration.registration_id == review.registration_id
                && registration.registration_sha256 == review.registration_sha256
                && registration.suite_specification.specification_sha256
                    == review.suite_specification_sha256
        })
        .map(|registration| registration.suite_specification)
        .ok_or_else(|| "精确绑定的固定训练套件不再可用".to_string())
}

fn verify_exact_dataset_binding(
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
) -> Result<(), String> {
    let review = &authorization
        .runner
        .implementation
        .approved_registration_review;
    let commitments = dataset
        .rows
        .iter()
        .map(|row| {
            (
                row.dataset_entry_id.clone(),
                row.target_commitment_sha256.clone(),
            )
        })
        .collect::<Vec<_>>();
    if dataset.training_store_dataset_sha256 != review.training_store_dataset_sha256
        || hash_serializable(&dataset.rows)? != review.rows_sha256
        || hash_serializable(&dataset.excluded_rows)? != review.excluded_rows_sha256
        || hash_serializable(&commitments)? != review.target_commitments_sha256
        || dataset.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || dataset.target_count != EXPECTED_TARGET_COUNT
        || !dataset.copied_to_training_store
        || !dataset.exact_official_dataset_copy
        || !dataset.one_to_one_join_preserved
        || !dataset.point_in_time_and_explicit_missingness_preserved
        || !dataset.official_split_purge_and_embargo_preserved
        || !dataset.exact_raw_f64_bits_preserved
        || !dataset.validation_targets_withheld
        || !dataset.sealed_holdout_targets_withheld
        || !dataset.action_position_and_reward_semantics_absent
        || dataset.training_started
        || dataset.reward_authorized
        || dataset.shadow_portfolio_authorized
        || dataset.order_generation_authorized
        || dataset.broker_access_authorized
        || dataset.trading_authorized
    {
        return Err("训练副本与授权完整绑定不一致或携带越权状态".to_string());
    }
    Ok(())
}

fn verify_fixed_suite(
    suite: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
) -> Result<(), String> {
    let algorithms = suite
        .arms
        .iter()
        .map(|arm| arm.algorithm.clone())
        .collect::<Vec<_>>();
    if suite.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || suite.target_count != EXPECTED_TARGET_COUNT
        || suite.target_vector_order.len() != EXPECTED_TARGET_COUNT
        || algorithms
            != vec![
                HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline,
                HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
                HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
            ]
        || suite
            .arms
            .iter()
            .any(|arm| arm.random_seeds != EXPECTED_SEEDS)
        || suite.fit_split != "train"
        || suite.model_selection_split != "validation"
        || suite.sealed_holdout_split != "sealed_holdout"
        || suite.sealed_holdout_access_allowed
        || suite.sealed_holdout_labels_visible_to_training_worker
        || suite.scalar_reward_defined
        || suite.action_position_or_ranking_semantics_defined
        || suite.ambient_environment_available
        || suite.network_available
        || suite.external_tools_available
        || suite.arbitrary_code_allowed
        || suite.production_state_write_available
    {
        return Err("固定训练套件已经漂移或越权".to_string());
    }
    Ok(())
}

fn prepare_training_data(
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    suite: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
) -> Result<PreparedTrainingData, String> {
    let first = dataset
        .rows
        .first()
        .ok_or_else(|| "训练副本没有可用行".to_string())?;
    let mut feature_order = first
        .feature_records
        .iter()
        .map(|feature| feature.feature_id.clone())
        .collect::<Vec<_>>();
    feature_order.sort();
    feature_order.dedup();
    if feature_order.len() != EXPECTED_FEATURE_COUNT {
        return Err("训练副本不是精确 65 项特征".to_string());
    }
    let mut train_features = Vec::new();
    let mut train_targets = Vec::new();
    let mut validation_count = 0;
    let mut sealed_holdout_count = 0;
    for row in &dataset.rows {
        match row.target_visibility {
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets => {
                if row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::Train {
                    return Err("带标签行不属于 train split".to_string());
                }
                train_features.push(parse_feature_row(row, &feature_order)?);
                train_targets.push(parse_target_row(row, &suite.target_vector_order)?);
            }
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld => {
                if row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::Validation
                    || row.target_vector.is_some()
                {
                    return Err("validation 标签隔离已经破坏".to_string());
                }
                validation_count += 1;
            }
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld => {
                if row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
                    || row.target_vector.is_some()
                {
                    return Err("sealed holdout 标签隔离已经破坏".to_string());
                }
                sealed_holdout_count += 1;
            }
        }
    }
    if train_features.is_empty() || train_features.len() != train_targets.len() {
        return Err("train split 没有完整特征与目标".to_string());
    }
    let preprocessing = fit_preprocessing(&feature_order, &mut train_features)?;
    Ok(PreparedTrainingData {
        feature_order,
        target_order: suite.target_vector_order.clone(),
        preprocessing,
        train_features,
        train_targets,
        validation_count,
        sealed_holdout_count,
    })
}

fn parse_feature_row(
    row: &HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    feature_order: &[String],
) -> Result<Vec<Option<f64>>, String> {
    let index = row
        .feature_records
        .iter()
        .map(|feature| (feature.feature_id.as_str(), feature))
        .collect::<BTreeMap<_, _>>();
    if index.len() != feature_order.len() {
        return Err(format!("条目 {} 的特征重复或缺失", row.dataset_entry_id));
    }
    feature_order
        .iter()
        .map(|feature_id| {
            let feature = index
                .get(feature_id.as_str())
                .ok_or_else(|| format!("条目 {} 缺少特征 {feature_id}", row.dataset_entry_id))?;
            if feature.available_at_utc > row.decision_available_at {
                return Err(format!("特征 {feature_id} 晚于决策时点"));
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
                    .map_err(|_| format!("非缺失特征 {feature_id} 不是有限数值"))?;
                if !value.is_finite() {
                    return Err(format!("非缺失特征 {feature_id} 不是有限数值"));
                }
                Ok(Some(value))
            }
        })
        .collect()
}

fn parse_target_row(
    row: &HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    target_order: &[String],
) -> Result<Vec<f64>, String> {
    let values = row
        .target_vector
        .as_ref()
        .ok_or_else(|| "train 行缺少原始目标".to_string())?;
    let index = values
        .iter()
        .map(|value| (value.target_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    if index.len() != EXPECTED_TARGET_COUNT {
        return Err("train 行目标重复或数量错误".to_string());
    }
    target_order
        .iter()
        .map(|target_id| {
            let value = index
                .get(target_id.as_str())
                .ok_or_else(|| format!("train 行缺少目标 {target_id}"))?;
            let bits = u64::from_str_radix(&value.exact_f64_bits_hex, 16)
                .map_err(|_| format!("目标 {target_id} 的 f64 位模式无效"))?;
            let parsed = f64::from_bits(bits);
            if !parsed.is_finite() {
                return Err(format!("目标 {target_id} 不是有限值"));
            }
            Ok(parsed)
        })
        .collect()
}

fn fit_preprocessing(
    feature_order: &[String],
    rows: &mut [Vec<Option<f64>>],
) -> Result<Vec<HistoricalOutcomeTrainingFeaturePreprocessingRecord>, String> {
    let mut records = Vec::with_capacity(feature_order.len());
    for (feature_index, feature_id) in feature_order.iter().enumerate() {
        let observed = rows
            .iter()
            .filter_map(|row| row[feature_index])
            .collect::<Vec<_>>();
        let mean = if observed.is_empty() {
            0.0
        } else {
            observed.iter().sum::<f64>() / observed.len() as f64
        };
        let variance = if observed.len() < 2 {
            0.0
        } else {
            observed
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / observed.len() as f64
        };
        let scale = variance.sqrt().max(1.0);
        for row in rows.iter_mut() {
            if let Some(value) = row[feature_index] {
                row[feature_index] = Some((value - mean) / scale);
            }
        }
        records.push(HistoricalOutcomeTrainingFeaturePreprocessingRecord {
            feature_id: feature_id.clone(),
            observed_train_count: observed.len(),
            missing_train_count: rows.len().saturating_sub(observed.len()),
            train_mean_f64_bits_hex: f64_hex(mean),
            train_scale_f64_bits_hex: f64_hex(scale),
            missingness_preserved: true,
        });
    }
    Ok(records)
}

async fn execute_fixed_training(
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    suite: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
    prepared: PreparedTrainingData,
) -> Result<
    (
        HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
        String,
        usize,
    ),
    TrainingFailure,
> {
    let maximum_output_bytes = authorization.runner.runner_contract.maximum_output_bytes as usize;
    let authorization = authorization.clone();
    let suite = suite.clone();
    let envelope =
        tokio::task::spawn_blocking(move || train_suite(&authorization, &suite, prepared))
            .await
            .map_err(|error| TrainingFailure {
                message: format!("固定训练 worker 异常：{error}"),
                ephemeral_directory_removed: true,
            })?
            .map_err(|message| TrainingFailure {
                message,
                ephemeral_directory_removed: true,
            })?;
    let bytes = serde_json::to_vec(&envelope).map_err(|error| TrainingFailure {
        message: error.to_string(),
        ephemeral_directory_removed: true,
    })?;
    if bytes.len() > maximum_output_bytes {
        return Err(TrainingFailure {
            message: "未验证训练候选超过 runner 输出上限".to_string(),
            ephemeral_directory_removed: true,
        });
    }
    let directory = std::env::temp_dir().join(format!("hone-training-{}", Uuid::new_v4()));
    tokio::fs::create_dir(&directory)
        .await
        .map_err(|error| TrainingFailure {
            message: format!("一次性训练目录创建失败：{error}"),
            ephemeral_directory_removed: false,
        })?;
    let path = directory.join("untrusted-training-artifacts.json");
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
            return Err("一次性训练输出回读不一致".to_string());
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
        (Ok(output_sha256), Ok(()), true) => Ok((envelope, output_sha256, bytes.len())),
        (Ok(_), Err(error), removed) => Err(TrainingFailure {
            message: format!("一次性训练目录清理失败：{error}"),
            ephemeral_directory_removed: removed,
        }),
        (Err(error), _, removed) => Err(TrainingFailure {
            message: error,
            ephemeral_directory_removed: removed,
        }),
        (Ok(_), Ok(()), false) => Err(TrainingFailure {
            message: "一次性训练目录清理状态无法确认".to_string(),
            ephemeral_directory_removed: false,
        }),
    }
}

fn train_suite(
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    suite: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
    data: PreparedTrainingData,
) -> Result<HistoricalOutcomeTrainingUntrustedArtifactEnvelope, String> {
    let mut model_artifacts = Vec::new();
    let mut fit_diagnostics = Vec::new();
    for arm in &suite.arms {
        for seed in &arm.random_seeds {
            let (target_models, predictions) = match arm.algorithm {
                HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline => {
                    train_zero_models(&data)
                }
                HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression => {
                    train_ridge_models(&data)?
                }
                HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression => {
                    train_boosted_models(&data, *seed, arm.max_epochs_or_boosting_rounds, arm.learning_rate_micros, arm.maximum_tree_depth)?
                }
            };
            let mut artifact = HistoricalOutcomeTrainingUntrustedModelArtifact {
                artifact_sha256: String::new(),
                algorithm: arm.algorithm.clone(),
                random_seed: *seed,
                train_row_count: data.train_targets.len(),
                exact_feature_count: data.feature_order.len(),
                exact_target_count: data.target_order.len(),
                target_models,
                validation_selected: false,
                sealed_holdout_accessed: false,
            };
            artifact.artifact_sha256 = fingerprint_without(&artifact, &["artifact_sha256"])?;
            fit_diagnostics.extend(compute_fit_diagnostics(
                &arm.algorithm,
                *seed,
                &data.target_order,
                &data.train_targets,
                &predictions,
            )?);
            model_artifacts.push(artifact);
        }
    }
    let review = &authorization
        .runner
        .implementation
        .approved_registration_review;
    let envelope = HistoricalOutcomeTrainingUntrustedArtifactEnvelope {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_string(),
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        isolated_runner_id: authorization.runner.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: authorization.runner.isolated_runner_spec_sha256.clone(),
        implementation_id: authorization
            .runner
            .implementation
            .implementation_id
            .clone(),
        implementation_sha256: authorization
            .runner
            .implementation
            .implementation_sha256
            .clone(),
        implementation_review_sha256: authorization
            .runner
            .implementation_review
            .review_sha256
            .clone(),
        suite_specification_sha256: review.suite_specification_sha256.clone(),
        training_store_dataset_sha256: review.training_store_dataset_sha256.clone(),
        rows_sha256: review.rows_sha256.clone(),
        excluded_rows_sha256: review.excluded_rows_sha256.clone(),
        target_commitments_sha256: review.target_commitments_sha256.clone(),
        feature_order: data.feature_order,
        target_order: data.target_order,
        preprocessing: data.preprocessing,
        train_row_count: data.train_targets.len(),
        validation_row_count_with_targets_withheld: data.validation_count,
        sealed_holdout_row_count_with_targets_withheld: data.sealed_holdout_count,
        model_artifacts,
        fit_diagnostics,
        train_only_fit_completed: true,
        validation_labels_accessed: false,
        validation_selection_completed: false,
        sealed_holdout_labels_accessed: false,
        output_is_untrusted: true,
        independent_output_validation_completed: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        scalar_reward_written: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    };
    validate_envelope(&envelope)?;
    Ok(envelope)
}

fn train_zero_models(
    data: &PreparedTrainingData,
) -> (Vec<HistoricalOutcomeTrainingTargetModel>, Vec<Vec<f64>>) {
    let predictions = vec![vec![0.0; data.target_order.len()]; data.train_targets.len()];
    let models = data
        .target_order
        .iter()
        .map(|target_id| HistoricalOutcomeTrainingTargetModel {
            target_id: target_id.clone(),
            model: HistoricalOutcomeTrainingTargetModelBody::FrozenZeroPrediction {
                prediction_f64_bits_hex: f64_hex(0.0),
            },
        })
        .collect();
    (models, predictions)
}

fn design_row(features: &[Option<f64>]) -> Vec<f64> {
    let mut row = Vec::with_capacity(1 + features.len() * 2);
    row.push(1.0);
    for value in features {
        row.push(value.unwrap_or(0.0));
        row.push(f64::from(value.is_none()));
    }
    row
}

fn train_ridge_models(
    data: &PreparedTrainingData,
) -> Result<(Vec<HistoricalOutcomeTrainingTargetModel>, Vec<Vec<f64>>), String> {
    let design = data
        .train_features
        .iter()
        .map(|row| design_row(row))
        .collect::<Vec<_>>();
    let dimension = design[0].len();
    let mut xtx = vec![vec![0.0; dimension]; dimension];
    let mut xty = vec![vec![0.0; dimension]; data.target_order.len()];
    for (row, targets) in design.iter().zip(&data.train_targets) {
        for left in 0..dimension {
            for right in 0..dimension {
                xtx[left][right] += row[left] * row[right];
            }
            for target in 0..data.target_order.len() {
                xty[target][left] += row[left] * targets[target];
            }
        }
    }
    for index in 1..dimension {
        xtx[index][index] += RIDGE_LAMBDA;
    }
    let weights = xty
        .iter()
        .map(|rhs| solve_linear_system(&xtx, rhs))
        .collect::<Result<Vec<_>, _>>()?;
    let predictions = design
        .iter()
        .map(|row| {
            weights
                .iter()
                .map(|target_weights| dot(row, target_weights))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let models = weights
        .iter()
        .enumerate()
        .map(|(target, values)| HistoricalOutcomeTrainingTargetModel {
            target_id: data.target_order[target].clone(),
            model: HistoricalOutcomeTrainingTargetModelBody::RidgeRegression {
                intercept_f64_bits_hex: f64_hex(values[0]),
                l2_regularization_f64_bits_hex: f64_hex(RIDGE_LAMBDA),
                coefficients: data
                    .feature_order
                    .iter()
                    .enumerate()
                    .map(
                        |(index, feature_id)| HistoricalOutcomeTrainingRidgeCoefficient {
                            feature_id: feature_id.clone(),
                            observed_value_coefficient_f64_bits_hex: f64_hex(values[1 + index * 2]),
                            missing_indicator_coefficient_f64_bits_hex: f64_hex(
                                values[2 + index * 2],
                            ),
                        },
                    )
                    .collect(),
            },
        })
        .collect();
    Ok((models, predictions))
}

fn solve_linear_system(matrix: &[Vec<f64>], rhs: &[f64]) -> Result<Vec<f64>, String> {
    let size = rhs.len();
    let mut augmented = matrix
        .iter()
        .zip(rhs)
        .map(|(row, value)| {
            let mut output = row.clone();
            output.push(*value);
            output
        })
        .collect::<Vec<_>>();
    for pivot in 0..size {
        let best = (pivot..size)
            .max_by(|left, right| {
                augmented[*left][pivot]
                    .abs()
                    .total_cmp(&augmented[*right][pivot].abs())
            })
            .ok_or_else(|| "ridge 方程缺少 pivot".to_string())?;
        augmented.swap(pivot, best);
        let divisor = augmented[pivot][pivot];
        if divisor.abs() <= NUMERIC_EPSILON {
            return Err("ridge 方程奇异，无法在冻结规则下求解".to_string());
        }
        for column in pivot..=size {
            augmented[pivot][column] /= divisor;
        }
        for row in 0..size {
            if row == pivot {
                continue;
            }
            let factor = augmented[row][pivot];
            if factor.abs() <= NUMERIC_EPSILON {
                continue;
            }
            for column in pivot..=size {
                augmented[row][column] -= factor * augmented[pivot][column];
            }
        }
    }
    let output = augmented
        .into_iter()
        .map(|row| row[size])
        .collect::<Vec<_>>();
    if output.iter().any(|value| !value.is_finite()) {
        return Err("ridge 输出非有限值".to_string());
    }
    Ok(output)
}

fn train_boosted_models(
    data: &PreparedTrainingData,
    seed: u64,
    rounds: u16,
    learning_rate_micros: u32,
    maximum_tree_depth: u8,
) -> Result<(Vec<HistoricalOutcomeTrainingTargetModel>, Vec<Vec<f64>>), String> {
    let learning_rate = f64::from(learning_rate_micros) / 1_000_000.0;
    let mut predictions = vec![vec![0.0; data.target_order.len()]; data.train_targets.len()];
    let thresholds = (0..data.feature_order.len())
        .map(|feature| median_observed(&data.train_features, feature))
        .collect::<Vec<_>>();
    let mut models = Vec::new();
    for target in 0..data.target_order.len() {
        let mut target_rounds = Vec::with_capacity(rounds as usize);
        for round in 0..rounds {
            let residuals = data
                .train_targets
                .iter()
                .zip(&predictions)
                .map(|(actual, predicted)| actual[target] - predicted[target])
                .collect::<Vec<_>>();
            let stump = best_stump(
                &data.feature_order,
                &data.train_features,
                &thresholds,
                &residuals,
                seed,
                round,
            );
            for (row_index, features) in data.train_features.iter().enumerate() {
                predictions[row_index][target] +=
                    learning_rate * stump_prediction(&stump, features);
            }
            target_rounds.push(HistoricalOutcomeTrainingBoostedStump {
                feature_id: stump
                    .feature_index
                    .map(|index| data.feature_order[index].clone()),
                threshold_f64_bits_hex: stump.threshold.map(f64_hex),
                low_leaf_f64_bits_hex: f64_hex(stump.low_leaf),
                high_leaf_f64_bits_hex: f64_hex(stump.high_leaf),
                missing_leaf_f64_bits_hex: f64_hex(stump.missing_leaf),
                depth: u8::from(stump.feature_index.is_some()),
            });
        }
        models.push(HistoricalOutcomeTrainingTargetModel {
            target_id: data.target_order[target].clone(),
            model: HistoricalOutcomeTrainingTargetModelBody::GradientBoostedRegression {
                learning_rate_f64_bits_hex: f64_hex(learning_rate),
                maximum_tree_depth,
                rounds: target_rounds,
            },
        });
    }
    Ok((models, predictions))
}

#[derive(Debug, Clone)]
struct Stump {
    feature_index: Option<usize>,
    threshold: Option<f64>,
    low_leaf: f64,
    high_leaf: f64,
    missing_leaf: f64,
    loss: f64,
}

fn best_stump(
    feature_order: &[String],
    features: &[Vec<Option<f64>>],
    thresholds: &[Option<f64>],
    residuals: &[f64],
    seed: u64,
    round: u16,
) -> Stump {
    let mean = residuals.iter().sum::<f64>() / residuals.len() as f64;
    let mut best = Stump {
        feature_index: None,
        threshold: None,
        low_leaf: mean,
        high_leaf: mean,
        missing_leaf: mean,
        loss: residuals.iter().map(|value| (value - mean).powi(2)).sum(),
    };
    let offset = ((seed + u64::from(round)) % feature_order.len() as u64) as usize;
    for step in 0..feature_order.len() {
        let feature = (offset + step) % feature_order.len();
        let Some(threshold) = thresholds[feature] else {
            continue;
        };
        let mut low = Vec::new();
        let mut high = Vec::new();
        let mut missing = Vec::new();
        for (row, residual) in features.iter().zip(residuals) {
            match row[feature] {
                Some(value) if value <= threshold => low.push(*residual),
                Some(_) => high.push(*residual),
                None => missing.push(*residual),
            }
        }
        if low.is_empty() || high.is_empty() {
            continue;
        }
        let low_mean = mean_slice(&low);
        let high_mean = mean_slice(&high);
        let missing_mean = if missing.is_empty() {
            0.0
        } else {
            mean_slice(&missing)
        };
        let loss = squared_error(&low, low_mean)
            + squared_error(&high, high_mean)
            + squared_error(&missing, missing_mean);
        if loss + NUMERIC_EPSILON < best.loss {
            best = Stump {
                feature_index: Some(feature),
                threshold: Some(threshold),
                low_leaf: low_mean,
                high_leaf: high_mean,
                missing_leaf: missing_mean,
                loss,
            };
        }
    }
    best
}

fn stump_prediction(stump: &Stump, features: &[Option<f64>]) -> f64 {
    match (stump.feature_index, stump.threshold) {
        (Some(index), Some(threshold)) => match features[index] {
            Some(value) if value <= threshold => stump.low_leaf,
            Some(_) => stump.high_leaf,
            None => stump.missing_leaf,
        },
        _ => stump.missing_leaf,
    }
}

fn median_observed(rows: &[Vec<Option<f64>>], feature: usize) -> Option<f64> {
    let mut values = rows
        .iter()
        .filter_map(|row| row[feature])
        .collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    if values.len() < 2 {
        None
    } else {
        Some(values[(values.len() - 1) / 2])
    }
}

fn mean_slice(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn squared_error(values: &[f64], mean: f64) -> f64 {
    values.iter().map(|value| (value - mean).powi(2)).sum()
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn compute_fit_diagnostics(
    algorithm: &HistoricalOutcomeTrainingExperimentAlgorithm,
    seed: u64,
    target_order: &[String],
    actual: &[Vec<f64>],
    predicted: &[Vec<f64>],
) -> Result<Vec<HistoricalOutcomeTrainingFitDiagnostic>, String> {
    if actual.len() != predicted.len() || actual.is_empty() {
        return Err("训练诊断输入行数不一致".to_string());
    }
    (0..target_order.len())
        .map(|target| {
            let y = actual.iter().map(|row| row[target]).collect::<Vec<_>>();
            let p = predicted.iter().map(|row| row[target]).collect::<Vec<_>>();
            let mae = y.iter().zip(&p).map(|(y, p)| (y - p).abs()).sum::<f64>() / y.len() as f64;
            let directional = y
                .iter()
                .zip(&p)
                .filter(|(y, p)| y.signum() == p.signum())
                .count() as f64
                / y.len() as f64;
            Ok(HistoricalOutcomeTrainingFitDiagnostic {
                algorithm: algorithm.clone(),
                random_seed: seed,
                target_id: target_order[target].clone(),
                split: "train_fit_diagnostic_only".to_string(),
                mae_f64_bits_hex: f64_hex(mae),
                spearman_f64_bits_hex: spearman(&y, &p).map(f64_hex),
                directional_accuracy_f64_bits_hex: f64_hex(directional),
                calibration_slope_f64_bits_hex: calibration_slope(&y, &p).map(f64_hex),
                model_selection_metric: false,
            })
        })
        .collect()
}

fn spearman(left: &[f64], right: &[f64]) -> Option<f64> {
    let left_rank = average_ranks(left);
    let right_rank = average_ranks(right);
    pearson(&left_rank, &right_rank)
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
    let left_mean = mean_slice(left);
    let right_mean = mean_slice(right);
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

fn calibration_slope(actual: &[f64], predicted: &[f64]) -> Option<f64> {
    let predicted_mean = mean_slice(predicted);
    let actual_mean = mean_slice(actual);
    let variance = predicted
        .iter()
        .map(|value| (value - predicted_mean).powi(2))
        .sum::<f64>();
    if variance <= NUMERIC_EPSILON {
        return None;
    }
    Some(
        predicted
            .iter()
            .zip(actual)
            .map(|(predicted, actual)| (predicted - predicted_mean) * (actual - actual_mean))
            .sum::<f64>()
            / variance,
    )
}

fn validate_envelope(
    envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
) -> Result<(), String> {
    let artifact_keys = envelope
        .model_artifacts
        .iter()
        .map(|artifact| (format!("{:?}", artifact.algorithm), artifact.random_seed))
        .collect::<BTreeSet<_>>();
    let target_diagnostic_keys = envelope
        .fit_diagnostics
        .iter()
        .map(|metric| {
            (
                format!("{:?}", metric.algorithm),
                metric.random_seed,
                metric.target_id.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    let artifacts_valid = envelope.model_artifacts.iter().all(|artifact| {
        valid_sha256(&artifact.artifact_sha256)
            && artifact.artifact_sha256
                == fingerprint_without(artifact, &["artifact_sha256"]).unwrap_or_default()
            && artifact.train_row_count == envelope.train_row_count
            && artifact.exact_feature_count == EXPECTED_FEATURE_COUNT
            && artifact.exact_target_count == EXPECTED_TARGET_COUNT
            && artifact.target_models.len() == EXPECTED_TARGET_COUNT
            && !artifact.validation_selected
            && !artifact.sealed_holdout_accessed
    });
    let metrics_valid = envelope.fit_diagnostics.iter().all(|metric| {
        metric.split == "train_fit_diagnostic_only"
            && !metric.model_selection_metric
            && valid_f64_hex(&metric.mae_f64_bits_hex)
            && valid_f64_hex(&metric.directional_accuracy_f64_bits_hex)
            && metric
                .spearman_f64_bits_hex
                .as_deref()
                .is_none_or(valid_f64_hex)
            && metric
                .calibration_slope_f64_bits_hex
                .as_deref()
                .is_none_or(valid_f64_hex)
    });
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION
        || envelope.feature_order.len() != EXPECTED_FEATURE_COUNT
        || envelope.target_order.len() != EXPECTED_TARGET_COUNT
        || envelope.preprocessing.len() != EXPECTED_FEATURE_COUNT
        || envelope.model_artifacts.len() != 9
        || artifact_keys.len() != 9
        || envelope.fit_diagnostics.len() != 81
        || target_diagnostic_keys.len() != 81
        || !artifacts_valid
        || !metrics_valid
        || !envelope.train_only_fit_completed
        || envelope.validation_labels_accessed
        || envelope.validation_selection_completed
        || envelope.sealed_holdout_labels_accessed
        || !envelope.output_is_untrusted
        || envelope.independent_output_validation_completed
        || envelope.model_artifact_store_written
        || envelope.metric_store_written
        || envelope.scalar_reward_written
        || envelope.shadow_position_written
        || envelope.order_generated
        || envelope.broker_accessed
        || envelope.trade_executed
    {
        return Err("未验证训练候选结构无效或越权".to_string());
    }
    Ok(())
}

fn new_claim(
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    admin_id: &str,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeTrainingExecutionAttemptClaim, String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let review = &implementation.approved_registration_review;
    let contract = &runner.runner_contract;
    let mut claim = HistoricalOutcomeTrainingExecutionAttemptClaim {
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
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_review_sha256: runner.implementation_review.review_sha256.clone(),
        suite_specification_sha256: review.suite_specification_sha256.clone(),
        training_store_dataset_sha256: review.training_store_dataset_sha256.clone(),
        rows_sha256: review.rows_sha256.clone(),
        excluded_rows_sha256: review.excluded_rows_sha256.clone(),
        target_commitments_sha256: review.target_commitments_sha256.clone(),
        claimed_at,
        invoked_by: admin_id.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        maximum_wall_clock_seconds: contract.maximum_wall_clock_seconds,
        maximum_memory_mib: contract.maximum_memory_mib,
        maximum_cpu_millicores: contract.maximum_cpu_millicores,
        maximum_process_count: contract.maximum_process_count,
        maximum_output_bytes: contract.maximum_output_bytes,
        authorization_consumed: true,
        invocation_started: true,
        exact_training_store_read_allowed: true,
        train_target_read_allowed: true,
        validation_target_read_allowed: false,
        sealed_holdout_target_read_allowed: false,
        generic_training_store_read_allowed: false,
        production_read_allowed: false,
        production_write_allowed: false,
        network_allowed: false,
        external_tools_allowed: false,
        child_process_spawned: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        reward_write_allowed: false,
        shadow_write_allowed: false,
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
    claim: &HistoricalOutcomeTrainingExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    envelope: HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    output_sha256: String,
    output_bytes: usize,
) -> Result<HistoricalOutcomeTrainingExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeTrainingExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeTrainingExecutionAttemptStatus::CompletedWithUntrustedTrainOnlyArtifacts,
        exit_code: 0,
        output_sha256: Some(output_sha256),
        output_bytes,
        bounded_error: None,
        untrusted_artifact_envelope: Some(envelope),
        ephemeral_directory_removed: true,
        independent_output_validation_completed: false,
        validation_selection_authorized: false,
        sealed_holdout_access_authorized: false,
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
    claim: &HistoricalOutcomeTrainingExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    error: &TrainingFailure,
) -> Result<HistoricalOutcomeTrainingExecutionAttemptResult, String> {
    let bounded = error
        .message
        .chars()
        .take(MAX_ERROR_CHARS)
        .collect::<String>();
    let mut result = HistoricalOutcomeTrainingExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeTrainingExecutionAttemptStatus::FailedAuthorizationConsumed,
        exit_code: 1,
        output_sha256: None,
        output_bytes: 0,
        bounded_error: Some(bounded),
        untrusted_artifact_envelope: None,
        ephemeral_directory_removed: error.ephemeral_directory_removed,
        independent_output_validation_completed: false,
        validation_selection_authorized: false,
        sealed_holdout_access_authorized: false,
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

fn validate_claim(claim: &HistoricalOutcomeTrainingExecutionAttemptClaim) -> Result<(), String> {
    let closed = !claim.validation_target_read_allowed
        && !claim.sealed_holdout_target_read_allowed
        && !claim.generic_training_store_read_allowed
        && !claim.production_read_allowed
        && !claim.production_write_allowed
        && !claim.network_allowed
        && !claim.external_tools_allowed
        && !claim.child_process_spawned
        && !claim.model_store_write_allowed
        && !claim.metric_store_write_allowed
        && !claim.reward_write_allowed
        && !claim.shadow_write_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed;
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&claim.attempt_id)
        || !valid_sha256(&claim.claim_sha256)
        || claim.attempt_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != claim_fingerprint(claim)?
        || claim.invoked_by.trim().is_empty()
        || claim.isolation_backend != ISOLATION_BACKEND
        || !claim.authorization_consumed
        || !claim.invocation_started
        || !claim.exact_training_store_read_allowed
        || !claim.train_target_read_allowed
        || !closed
    {
        return Err("训练执行 claim 无效或越权".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeTrainingExecutionAttemptResult,
    claim: &HistoricalOutcomeTrainingExecutionAttemptClaim,
) -> Result<(), String> {
    validate_claim(claim)?;
    let completed = result.status
        == HistoricalOutcomeTrainingExecutionAttemptStatus::CompletedWithUntrustedTrainOnlyArtifacts;
    let shape = if completed {
        result.exit_code == 0
            && result.output_sha256.as_deref().is_some_and(valid_sha256)
            && result.output_bytes > 0
            && result.bounded_error.is_none()
            && result
                .untrusted_artifact_envelope
                .as_ref()
                .is_some_and(|envelope| validate_envelope(envelope).is_ok())
            && result.ephemeral_directory_removed
    } else {
        result.exit_code != 0
            && result.output_sha256.is_none()
            && result.output_bytes == 0
            && result
                .bounded_error
                .as_ref()
                .is_some_and(|value| !value.is_empty())
            && result.untrusted_artifact_envelope.is_none()
    };
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != result_fingerprint(result)?
        || result.attempt_id != claim.attempt_id
        || result.claim_sha256 != claim.claim_sha256
        || !shape
        || result.independent_output_validation_completed
        || result.validation_selection_authorized
        || result.sealed_holdout_access_authorized
        || result.reward_authorized
        || result.shadow_portfolio_authorized
        || result.order_generation_authorized
        || result.broker_access_authorized
        || result.trading_authorized
    {
        return Err("训练执行结果无效或越权".to_string());
    }
    Ok(())
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<HistoricalOutcomeTrainingExecutionAttemptClaim>,
        Vec<HistoricalOutcomeTrainingExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<HistoricalOutcomeTrainingExecutionAttemptClaim>(
        &execution_root(state).join("claims"),
    )
    .await?;
    let results = read_nested_json::<HistoricalOutcomeTrainingExecutionAttemptResult>(
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
            return Err("训练执行 claim 重复，拒绝继续".to_string());
        }
        exact_first_execution_authorization_for_training_audit(
            state,
            &claim.isolated_runner_id,
            &claim.authorization_review_id,
            &claim.authorization_review_sha256,
        )
        .await?
        .ok_or_else(|| "训练执行 claim 的授权链不再可重开".to_string())?;
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
            .ok_or_else(|| "训练执行结果缺少 claim".to_string())?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_attempts.insert(result.attempt_id.clone())
        {
            return Err("训练执行结果重复，拒绝继续".to_string());
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
    claim: &HistoricalOutcomeTrainingExecutionAttemptClaim,
) -> Result<String, String> {
    fingerprint_without(claim, &["attempt_id", "claim_sha256"])
}

fn result_fingerprint(
    result: &HistoricalOutcomeTrainingExecutionAttemptResult,
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

fn f64_hex(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn valid_f64_hex(value: &str) -> bool {
    value.len() == 16
        && u64::from_str_radix(value, 16)
            .ok()
            .map(f64::from_bits)
            .is_some_and(f64::is_finite)
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
    decision_root(state).join("historical-outcome-training-execution-attempts")
}

fn claim_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("claims").join(runner_id)
}

fn result_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("results").join(runner_id)
}

struct TrainingExecutionLock {
    path: PathBuf,
}

impl Drop for TrainingExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, runner_id: &str) -> Result<TrainingExecutionLock, String> {
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
        Ok(_) => Ok(TrainingExecutionLock { path }),
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
            Err("这条训练 runner 正在执行或等待结果，拒绝并发调用".to_string())
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

    fn feature_order() -> Vec<String> {
        (0..EXPECTED_FEATURE_COUNT)
            .map(|index| format!("feature_{index:02}"))
            .collect()
    }

    fn target_order() -> Vec<String> {
        [20_u16, 60, 250]
            .into_iter()
            .flat_map(|horizon| {
                ["asset_return", "excess_return", "asset_max_drawdown"]
                    .into_iter()
                    .map(move |metric| format!("{metric}_{horizon}_continuous"))
            })
            .collect()
    }

    fn prepared() -> PreparedTrainingData {
        let features = feature_order();
        let targets = target_order();
        let train_features = (0..8)
            .map(|row| {
                (0..EXPECTED_FEATURE_COUNT)
                    .map(|feature| {
                        ((row + feature) % 7 != 0)
                            .then_some((row as f64 - 3.5) * (feature as f64 + 1.0) / 100.0)
                    })
                    .collect()
            })
            .collect::<Vec<_>>();
        let train_targets = (0..8)
            .map(|row| {
                (0..EXPECTED_TARGET_COUNT)
                    .map(|target| (row as f64 - 3.5) * (target as f64 + 1.0) / 50.0)
                    .collect()
            })
            .collect();
        PreparedTrainingData {
            preprocessing: features
                .iter()
                .map(
                    |feature_id| HistoricalOutcomeTrainingFeaturePreprocessingRecord {
                        feature_id: feature_id.clone(),
                        observed_train_count: 7,
                        missing_train_count: 1,
                        train_mean_f64_bits_hex: f64_hex(0.0),
                        train_scale_f64_bits_hex: f64_hex(1.0),
                        missingness_preserved: true,
                    },
                )
                .collect(),
            feature_order: features,
            target_order: targets,
            train_features,
            train_targets,
            validation_count: 2,
            sealed_holdout_count: 2,
        }
    }

    #[test]
    fn zero_arm_is_exactly_zero_and_seed_independent() {
        let data = prepared();
        let (models, predictions) = train_zero_models(&data);
        assert_eq!(models.len(), EXPECTED_TARGET_COUNT);
        assert!(predictions.iter().flatten().all(|value| *value == 0.0));
    }

    #[test]
    fn ridge_fits_all_targets_and_preserves_missingness_parameters() {
        let data = prepared();
        let (models, predictions) = train_ridge_models(&data).unwrap();
        assert_eq!(models.len(), EXPECTED_TARGET_COUNT);
        assert_eq!(predictions.len(), data.train_targets.len());
        match &models[0].model {
            HistoricalOutcomeTrainingTargetModelBody::RidgeRegression { coefficients, .. } => {
                assert_eq!(coefficients.len(), EXPECTED_FEATURE_COUNT);
            }
            _ => panic!("expected ridge model"),
        }
    }

    #[test]
    fn boosted_arm_is_deterministic_for_same_seed() {
        let data = prepared();
        let first = train_boosted_models(&data, 17, 12, 50_000, 4).unwrap();
        let second = train_boosted_models(&data, 17, 12, 50_000, 4).unwrap();
        assert_eq!(first.0, second.0);
        assert_eq!(first.1, second.1);
    }

    #[test]
    fn boosted_arm_emits_bounded_depth_and_rounds() {
        let data = prepared();
        let (models, _) = train_boosted_models(&data, 29, 7, 50_000, 4).unwrap();
        match &models[0].model {
            HistoricalOutcomeTrainingTargetModelBody::GradientBoostedRegression {
                maximum_tree_depth,
                rounds,
                ..
            } => {
                assert_eq!(*maximum_tree_depth, 4);
                assert_eq!(rounds.len(), 7);
                assert!(rounds.iter().all(|round| round.depth <= 1));
            }
            _ => panic!("expected boosted model"),
        }
    }

    #[test]
    fn diagnostics_are_train_only_and_not_selection_metrics() {
        let data = prepared();
        let (_, predictions) = train_zero_models(&data);
        let diagnostics = compute_fit_diagnostics(
            &HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline,
            17,
            &data.target_order,
            &data.train_targets,
            &predictions,
        )
        .unwrap();
        assert_eq!(diagnostics.len(), EXPECTED_TARGET_COUNT);
        assert!(diagnostics.iter().all(|value| {
            value.split == "train_fit_diagnostic_only" && !value.model_selection_metric
        }));
    }

    #[test]
    fn rank_metric_rejects_constant_prediction_as_undefined() {
        assert!(spearman(&[1.0, 2.0, 3.0], &[0.0, 0.0, 0.0]).is_none());
        assert!(calibration_slope(&[1.0, 2.0, 3.0], &[0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn linear_solver_recovers_simple_system() {
        let solution = solve_linear_system(&[vec![2.0, 0.0], vec![0.0, 4.0]], &[6.0, 8.0]).unwrap();
        assert!((solution[0] - 3.0).abs() < 1e-12);
        assert!((solution[1] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn f64_encoding_is_exact_and_rejects_non_finite_values() {
        assert_eq!(
            f64::from_bits(u64::from_str_radix(&f64_hex(1.25), 16).unwrap()),
            1.25
        );
        assert!(valid_f64_hex(&f64_hex(-0.25)));
        assert!(!valid_f64_hex(&f64_hex(f64::NAN)));
    }

    #[test]
    fn invocation_confirmations_fail_closed() {
        let mut request = InvokeHistoricalOutcomeTrainingOnceRequest {
            expected_first_execution_authorization_review_id: "1".repeat(32),
            expected_first_execution_authorization_review_sha256: "2".repeat(64),
            expected_isolated_runner_spec_sha256: "3".repeat(64),
            expected_runner_artifact_sha256: "4".repeat(64),
            expected_implementation_id: "5".repeat(32),
            expected_implementation_sha256: "6".repeat(64),
            expected_implementation_review_sha256: "7".repeat(64),
            expected_suite_specification_sha256: "8".repeat(64),
            expected_training_store_dataset_sha256: "9".repeat(64),
            expected_rows_sha256: "a".repeat(64),
            expected_excluded_rows_sha256: "b".repeat(64),
            expected_target_commitments_sha256: "c".repeat(64),
            claim_first_create_once_and_failure_consumes_confirmed: true,
            exact_read_only_training_store_dataset_only_confirmed: true,
            train_only_fit_and_explicit_missingness_preserved_confirmed: true,
            validation_and_sealed_holdout_labels_remain_withheld_confirmed: true,
            fixed_three_arm_three_seed_suite_confirmed: true,
            untrusted_content_addressed_output_and_independent_validation_confirmed: true,
            no_reward_shadow_order_broker_or_trading_confirmed: true,
        };
        assert!(verify_invocation_confirmations(&request).is_ok());
        request.validation_and_sealed_holdout_labels_remain_withheld_confirmed = false;
        assert!(verify_invocation_confirmations(&request).is_err());
    }

    #[test]
    fn failed_result_consumes_claim_without_output_or_authority() {
        let mut claim = HistoricalOutcomeTrainingExecutionAttemptClaim {
            schema_version: CLAIM_SCHEMA_VERSION.to_string(),
            execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
            attempt_id: String::new(),
            claim_sha256: String::new(),
            authorization_review_id: "1".repeat(32),
            authorization_review_sha256: "2".repeat(64),
            authorization_valid_until: Utc::now(),
            isolated_runner_id: "3".repeat(32),
            isolated_runner_spec_sha256: "4".repeat(64),
            runner_artifact_sha256: "5".repeat(64),
            runner_code_revision: "rev".to_string(),
            implementation_id: "6".repeat(32),
            implementation_sha256: "7".repeat(64),
            implementation_review_sha256: "8".repeat(64),
            suite_specification_sha256: "9".repeat(64),
            training_store_dataset_sha256: "a".repeat(64),
            rows_sha256: "b".repeat(64),
            excluded_rows_sha256: "c".repeat(64),
            target_commitments_sha256: "d".repeat(64),
            claimed_at: Utc::now(),
            invoked_by: "admin".to_string(),
            isolation_backend: ISOLATION_BACKEND.to_string(),
            maximum_wall_clock_seconds: 3600,
            maximum_memory_mib: 8192,
            maximum_cpu_millicores: 4000,
            maximum_process_count: 4,
            maximum_output_bytes: 256 * 1024 * 1024,
            authorization_consumed: true,
            invocation_started: true,
            exact_training_store_read_allowed: true,
            train_target_read_allowed: true,
            validation_target_read_allowed: false,
            sealed_holdout_target_read_allowed: false,
            generic_training_store_read_allowed: false,
            production_read_allowed: false,
            production_write_allowed: false,
            network_allowed: false,
            external_tools_allowed: false,
            child_process_spawned: false,
            model_store_write_allowed: false,
            metric_store_write_allowed: false,
            reward_write_allowed: false,
            shadow_write_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        let result = new_failed_result(
            &claim,
            Utc::now(),
            1,
            &TrainingFailure {
                message: "synthetic failure".to_string(),
                ephemeral_directory_removed: true,
            },
        )
        .unwrap();
        assert_eq!(
            result.status,
            HistoricalOutcomeTrainingExecutionAttemptStatus::FailedAuthorizationConsumed
        );
        assert!(result.untrusted_artifact_envelope.is_none());
        assert!(!result.reward_authorized && !result.trading_authorized);
    }
}
