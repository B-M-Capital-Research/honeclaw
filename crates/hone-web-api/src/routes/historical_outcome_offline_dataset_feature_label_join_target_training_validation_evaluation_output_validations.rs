//! Independent, create-once validation of Stage 63 validation-evaluation outputs.
//!
//! A validator outside the executor and complete prior actor chain reopens the immutable
//! Stage 63 claim/result, exact Stage 62 authorization, independently validated training
//! artifacts and training-store copy, plus the exact raw-outcome dataset. This module does
//! not call the Stage 63 projection, prediction, metric, bootstrap, Holm or recommendation
//! helpers. It independently reconstructs the validation-only projection and bitwise
//! recomputes all 81 metrics, 54 candidate hypotheses and nine per-target recommendations.
//! Sealed-holdout rows remain excluded. A pass is still an untrusted validation-evaluation
//! result and opens only a future per-target candidate-admission review gate.

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
    HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    HistoricalOutcomeFeatureLabelJoinTargetVisibility,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::{
    HistoricalOutcomeTrainingBoostedStump, HistoricalOutcomeTrainingTargetModel,
    HistoricalOutcomeTrainingTargetModelBody, HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    HistoricalOutcomeTrainingUntrustedModelArtifact,
    completed_training_execution_attempt_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::HistoricalOutcomeTrainingExperimentAlgorithm;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copies::HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_output_validations::independently_validated_training_store_copies_for_training_registration_review;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts::{
    CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt,
    HistoricalOutcomeValidationEvaluationExecutionAttemptStatus,
    HistoricalOutcomeValidationEvaluationMetric,
    HistoricalOutcomeValidationEvaluationPerTargetRecommendation,
    HistoricalOutcomeValidationEvaluationUntrustedEnvelope,
    completed_validation_evaluation_execution_attempt_for_independent_validation,
    completed_validation_evaluation_execution_attempts_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_first_execution_authorizations::{
    ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    exact_first_execution_authorization_for_validation_evaluation_audit,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_implementations::HistoricalOutcomeValidationEvaluationImplementationContract;
use super::historical_outcome_offline_dataset_transformation_execution_attempts::HistoricalOutcomeOfflineDatasetTransformationSplit;
use super::historical_outcome_offline_datasets::{
    HistoricalOutcomeOfflineDataset,
    exact_current_historical_outcome_offline_dataset_for_transformation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-output-validation-v1";
const POLICY_VERSION: &str =
    "hone-validation-evaluation-output-independent-full-bitwise-recomputation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-validation-evaluation-output-validator-v1-second-projection-prediction-statistics-path";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-execution-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-execution-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str = "hone-untrusted-validation-evaluation-envelope-v1";
const EXECUTION_POLICY_VERSION: &str =
    "hone-validation-evaluation-claim-first-one-shot-frozen-statistics-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const EXPECTED_ARTIFACT_COUNT: usize = 9;
const EXPECTED_METRIC_COUNT: usize = 81;
const EXPECTED_HYPOTHESIS_COUNT: usize = 54;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];
const NUMERIC_EPSILON: f64 = 1e-12;
const MAX_MISMATCH_REASONS: usize = 96;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeValidationEvaluationOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_implementation_sha256: String,
    expected_implementation_review_sha256: String,
    expected_candidate_set_sha256: String,
    expected_upstream_validation_sha256: String,
    expected_upstream_output_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_rows_sha256: String,
    expected_excluded_rows_sha256: String,
    expected_target_commitments_sha256: String,
    expected_validation_projection_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    independent_reopen_and_second_implementation_recomputation_confirmed: bool,
    exact_current_stage_51_through_stage_63_binding_confirmed: bool,
    exact_validation_projection_and_nine_candidate_predictions_confirmed: bool,
    all_eighty_one_metrics_fifty_four_hypotheses_and_nine_recommendations_bitwise_recomputed_confirmed:
        bool,
    sealed_holdout_remains_unread_confirmed: bool,
    no_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeValidationEvaluationOutputValidationVerdict {
    IndependentlyValidatedUntrustedValidationEvaluation,
    FailedIndependentValidationEvaluationOutputValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationOutputValidationRecord {
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
    pub candidate_set_sha256: String,
    pub upstream_validation_sha256: String,
    pub upstream_output_sha256: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
    pub validation_projection_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_execution_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_63_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub envelope_fingerprint_independently_verified: bool,
    pub exact_training_artifacts_store_dataset_and_contract_verified: bool,
    pub validation_projection_independently_reconstructed: bool,
    pub exact_nine_candidate_predictions_bitwise_recomputed: bool,
    pub exact_eighty_one_metrics_bitwise_recomputed: bool,
    pub exact_fifty_four_component_bootstrap_and_holm_tests_bitwise_recomputed: bool,
    pub exact_nine_per_target_recommendations_bitwise_recomputed: bool,
    pub sealed_holdout_non_access_verified: bool,
    pub no_selection_or_downstream_authority_verified: bool,
    pub recomputed_claim_sha256: String,
    pub recomputed_result_sha256: String,
    pub recomputed_output_sha256: String,
    pub recomputed_envelope_sha256: String,
    pub recomputed_validation_projection_sha256: String,
    pub recomputed_metric_count: usize,
    pub recomputed_candidate_hypothesis_count: usize,
    pub recomputed_per_target_recommendation_count: usize,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeValidationEvaluationOutputValidationVerdict,
    pub validation_evaluation_output_independently_validated: bool,
    pub future_per_target_candidate_admission_review_eligible: bool,
    pub official_candidate_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationOutputValidationItem {
    pub attempt: CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt,
    pub validation: Option<HistoricalOutcomeValidationEvaluationOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeValidationEvaluationOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_untrusted_envelope_count: usize,
    pub failed_validation_count: usize,
    pub future_per_target_candidate_admission_review_eligible_count: usize,
    pub validation_status: String,
    pub independent_output_validation_available: bool,
    pub official_candidate_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
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
pub(crate) struct HistoricalOutcomeValidationEvaluationOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_untrusted_envelope_count: usize,
    pub failed_validation_count: usize,
    pub future_per_target_candidate_admission_review_eligible_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeValidationEvaluationOutput {
    pub attempt: CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt,
    pub validation: HistoricalOutcomeValidationEvaluationOutputValidationRecord,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentValidationRow {
    pub dataset_entry_id: String,
    pub component_id: String,
    pub features: Vec<Option<f64>>,
    pub targets: Vec<f64>,
}

#[derive(Debug, Clone)]
struct IndependentValidationData {
    feature_order: Vec<String>,
    target_order: Vec<String>,
    rows: Vec<IndependentValidationRow>,
    projection_sha256: String,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    envelope_fingerprint: bool,
    exact_inputs: bool,
    projection_exact: bool,
    predictions_exact: bool,
    metrics_exact: bool,
    hypothesis_tests_exact: bool,
    recommendations_exact: bool,
    sealed_holdout_non_access: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_output_sha256: String,
    recomputed_envelope_sha256: String,
    recomputed_validation_projection_sha256: String,
    recomputed_metric_count: usize,
    recomputed_candidate_hypothesis_count: usize,
    recomputed_per_target_recommendation_count: usize,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_historical_outcome_validation_evaluation_output_validations(
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
            warn!(%error, "historical outcome validation evaluation output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果 validation 评估输出独立验证暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_validation_evaluation_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeValidationEvaluationOutputRequest>,
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

pub(crate) async fn historical_outcome_validation_evaluation_output_validation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeValidationEvaluationOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeValidationEvaluationOutputValidationReadinessSummary {
            validation_eligible_count: registry.validation_eligible_count,
            validation_count: registry.validation_count,
            independently_validated_untrusted_envelope_count: registry
                .independently_validated_untrusted_envelope_count,
            failed_validation_count: registry.failed_validation_count,
            future_per_target_candidate_admission_review_eligible_count: registry
                .future_per_target_candidate_admission_review_eligible_count,
            validation_status: registry.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_validation_evaluation_outputs_for_candidate_admission_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeValidationEvaluationOutput>, String> {
    let validations = read_validations(state).await?;
    let index = validations
        .into_iter()
        .map(|record| (record.attempt_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for attempt in
        completed_validation_evaluation_execution_attempts_for_independent_validation(state).await?
    {
        let Some(validation) = index.get(&attempt.claim.attempt_id).cloned() else {
            continue;
        };
        validate_record(&validation)?;
        if validation.validation_evaluation_output_independently_validated
            && validation.future_per_target_candidate_admission_review_eligible
            && validation.claim_sha256 == attempt.claim.claim_sha256
            && validation.result_sha256 == attempt.result.result_sha256
        {
            output.push(
                IndependentlyValidatedHistoricalOutcomeValidationEvaluationOutput {
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
) -> Result<HistoricalOutcomeValidationEvaluationOutputValidationRegistry, String> {
    let attempts =
        completed_validation_evaluation_execution_attempts_for_independent_validation(state)
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
            HistoricalOutcomeValidationEvaluationOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let independently_validated_untrusted_envelope_count = validations
        .iter()
        .filter(|record| record.validation_evaluation_output_independently_validated)
        .count();
    let failed_validation_count = validations
        .len()
        .saturating_sub(independently_validated_untrusted_envelope_count);
    let validation_status = if failed_validation_count > 0 {
        "validation_evaluation_output_independent_validation_failed_fail_closed"
    } else if independently_validated_untrusted_envelope_count > 0 {
        "validation_evaluation_output_independently_validated_waiting_per_target_candidate_admission_review"
    } else if validation_eligible_count > 0 {
        "waiting_independent_validation_evaluation_output_recomputation"
    } else {
        "waiting_completed_validation_evaluation_execution_attempt"
    };
    Ok(HistoricalOutcomeValidationEvaluationOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count: validations.len(),
        independently_validated_untrusted_envelope_count,
        failed_validation_count,
        future_per_target_candidate_admission_review_eligible_count:
            independently_validated_untrusted_envelope_count,
        validation_status: validation_status.to_string(),
        independent_output_validation_available: true,
        official_candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 64 阶段由 Stage 63 执行者和完整上游之外的新管理员重开 claim/result、九候选、独立校验训练副本和原始结果数据集，以第二套实现重构 validation-only 投影，并逐位复算 81 指标、54 项 component bootstrap/Holm 检验和 9 项逐目标建议。通过仍只是待逐目标准入复核的不可信评估；sealed holdout、正式选模、模型/指标库、奖励、影子、订单、券商和交易继续关闭。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeValidationEvaluationOutputRequest,
) -> Result<HistoricalOutcomeValidationEvaluationOutputValidationRecord, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.attempt_id == attempt_id)
    {
        return Err("这次 validation 评估已经形成不可变独立验证记录，不得重放".to_string());
    }
    let attempt = completed_validation_evaluation_execution_attempt_for_independent_validation(
        state, attempt_id,
    )
    .await?
    .ok_or_else(|| "当前没有完整且未验证的 validation 评估执行产物".to_string())?;
    verify_expected_bindings(&attempt, &request)?;
    let authorization = exact_first_execution_authorization_for_validation_evaluation_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "validation 评估执行授权完整链已经无法重开".to_string())?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("validation 评估输出验证人不能是执行人或完整上游链任一角色".to_string());
    }
    let training_attempt = exact_training_attempt(state, &authorization).await?;
    let training_envelope = training_attempt
        .result
        .untrusted_artifact_envelope
        .as_ref()
        .ok_or_else(|| "精确 Stage 57 训练执行缺少九候选工件".to_string())?;
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
        training_envelope,
        &dataset,
        &raw_dataset,
    )?;
    let passed = independent && audit.mismatch_reasons.is_empty();
    let envelope = attempt
        .result
        .untrusted_evaluation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 63 结果缺少 validation 评估 envelope".to_string())?;
    let mut record = HistoricalOutcomeValidationEvaluationOutputValidationRecord {
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
        candidate_set_sha256: attempt.claim.candidate_set_sha256.clone(),
        upstream_validation_sha256: attempt.claim.upstream_validation_sha256.clone(),
        upstream_output_sha256: attempt.claim.upstream_output_sha256.clone(),
        training_store_dataset_sha256: attempt.claim.training_store_dataset_sha256.clone(),
        rows_sha256: attempt.claim.rows_sha256.clone(),
        excluded_rows_sha256: attempt.claim.excluded_rows_sha256.clone(),
        target_commitments_sha256: attempt.claim.target_commitments_sha256.clone(),
        validation_projection_sha256: envelope.validation_projection_sha256.clone(),
        feature_order_sha256: envelope.feature_order_sha256.clone(),
        preprocessing_sha256: envelope.preprocessing_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        invoked_by: attempt.claim.invoked_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_execution_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_63_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        envelope_fingerprint_independently_verified: audit.envelope_fingerprint,
        exact_training_artifacts_store_dataset_and_contract_verified: audit.exact_inputs,
        validation_projection_independently_reconstructed: audit.projection_exact,
        exact_nine_candidate_predictions_bitwise_recomputed: audit.predictions_exact,
        exact_eighty_one_metrics_bitwise_recomputed: audit.metrics_exact,
        exact_fifty_four_component_bootstrap_and_holm_tests_bitwise_recomputed: audit
            .hypothesis_tests_exact,
        exact_nine_per_target_recommendations_bitwise_recomputed: audit.recommendations_exact,
        sealed_holdout_non_access_verified: audit.sealed_holdout_non_access,
        no_selection_or_downstream_authority_verified: audit.downstream_closed,
        recomputed_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_result_sha256: audit.recomputed_result_sha256,
        recomputed_output_sha256: audit.recomputed_output_sha256,
        recomputed_envelope_sha256: audit.recomputed_envelope_sha256,
        recomputed_validation_projection_sha256: audit.recomputed_validation_projection_sha256,
        recomputed_metric_count: audit.recomputed_metric_count,
        recomputed_candidate_hypothesis_count: audit.recomputed_candidate_hypothesis_count,
        recomputed_per_target_recommendation_count: audit
            .recomputed_per_target_recommendation_count,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            HistoricalOutcomeValidationEvaluationOutputValidationVerdict::IndependentlyValidatedUntrustedValidationEvaluation
        } else {
            HistoricalOutcomeValidationEvaluationOutputValidationVerdict::FailedIndependentValidationEvaluationOutputValidation
        },
        validation_evaluation_output_independently_validated: passed,
        future_per_target_candidate_admission_review_eligible: passed,
        official_candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
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

async fn exact_training_attempt(
    state: &AppState,
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
) -> Result<super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::CompletedHistoricalOutcomeTrainingExecutionAttempt, String>{
    let upstream = &authorization.runner.implementation.upstream_validation;
    let attempt = completed_training_execution_attempt_for_independent_validation(
        state,
        &upstream.attempt_id,
    )
    .await?
    .ok_or_else(|| "精确 Stage 57 训练执行已不存在".to_string())?;
    if attempt.claim.claim_sha256 != upstream.claim_sha256
        || attempt.result.result_sha256 != upstream.result_sha256
        || attempt.result.output_sha256.as_deref() != Some(upstream.output_sha256.as_str())
    {
        return Err("Stage 57–58 训练产物绑定已经漂移".to_string());
    }
    Ok(attempt)
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
    attempt: &CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    raw_dataset: &HistoricalOutcomeOfflineDataset,
) -> Result<IndependentAudit, String> {
    let claim = &attempt.claim;
    let result = &attempt.result;
    let envelope = result
        .untrusted_evaluation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 63 结果缺少未验证评估 envelope".to_string())?;
    let contract = &authorization.runner.implementation.implementation_contract;
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
        && claim.implementation_contract_sha256 == contract.contract_sha256
        && claim.implementation_review_sha256
            == authorization.runner.implementation_review.review_sha256
        && claim.implementation_independent_audit_sha256
            == authorization
                .runner
                .implementation_review
                .independent_audit
                .audit_sha256
        && claim.candidate_set_sha256 == contract.candidate_set_sha256
        && claim.upstream_validation_sha256
            == authorization
                .runner
                .implementation
                .upstream_validation
                .validation_sha256
        && claim.upstream_output_sha256
            == authorization
                .runner
                .implementation
                .upstream_validation
                .output_sha256
        && claim.training_store_dataset_sha256 == contract.training_store_dataset_sha256;
    let claim_fingerprint = claim.schema_version == CLAIM_SCHEMA_VERSION
        && claim.execution_policy_version == EXECUTION_POLICY_VERSION
        && claim.claim_sha256 == recomputed_claim_sha256
        && claim.attempt_id == recomputed_claim_sha256[..32]
        && claim.authorization_consumed
        && claim.validation_feature_read_allowed
        && claim.validation_label_read_allowed
        && claim.evaluation_authorized
        && !claim.sealed_holdout_feature_read_allowed
        && !claim.sealed_holdout_label_read_allowed
        && !claim.training_or_preprocessing_update_allowed
        && !claim.official_candidate_selection_allowed
        && !claim.model_store_write_allowed
        && !claim.metric_store_write_allowed
        && !claim.network_available
        && !claim.external_tools_available
        && !claim.child_process_allowed
        && !claim.production_read_allowed
        && !claim.production_write_allowed
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
            == HistoricalOutcomeValidationEvaluationExecutionAttemptStatus::CompletedWithUntrustedValidationEvaluation
        && result.exit_code == 0
        && result.output_sha256.as_deref() == Some(recomputed_output_sha256.as_str())
        && result.output_bytes == serde_json::to_vec(envelope).map_err(|error| error.to_string())?.len()
        && result.bounded_error.is_none()
        && result.ephemeral_directory_removed
        && result.validation_features_accessed
        && result.validation_labels_accessed
        && result.evaluation_completed
        && !result.sealed_holdout_features_accessed
        && !result.sealed_holdout_labels_accessed
        && !result.training_or_preprocessing_updated
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
        && envelope.upstream_validation_sha256 == claim.upstream_validation_sha256
        && envelope.upstream_output_sha256 == claim.upstream_output_sha256
        && envelope.training_store_dataset_sha256 == claim.training_store_dataset_sha256
        && envelope.candidate_set_sha256 == claim.candidate_set_sha256
        && envelope.rows_sha256 == claim.rows_sha256
        && envelope.excluded_rows_sha256 == claim.excluded_rows_sha256
        && envelope.target_commitments_sha256 == claim.target_commitments_sha256;
    let recomputed_rows_sha256 = hash_serializable(&dataset.rows)?;
    let recomputed_excluded_rows_sha256 = hash_serializable(&dataset.excluded_rows)?;
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
        && recomputed_excluded_rows_sha256 == claim.excluded_rows_sha256
        && hash_serializable(&commitments)? == claim.target_commitments_sha256
        && training_envelope.training_store_dataset_sha256 == claim.training_store_dataset_sha256
        && training_envelope.rows_sha256 == claim.rows_sha256
        && training_envelope.excluded_rows_sha256 == claim.excluded_rows_sha256
        && training_envelope.target_commitments_sha256 == claim.target_commitments_sha256
        && training_envelope.model_artifacts.len() == EXPECTED_ARTIFACT_COUNT
        && training_envelope.feature_order.len() == EXPECTED_FEATURE_COUNT
        && training_envelope.preprocessing.len() == EXPECTED_FEATURE_COUNT
        && training_envelope.target_order.len() == EXPECTED_TARGET_COUNT
        && contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == EXPECTED_TARGET_COUNT
        && contract.exact_artifact_count == EXPECTED_ARTIFACT_COUNT
        && contract.exact_random_seeds == EXPECTED_SEEDS
        && contract.exact_candidate_hypothesis_count as usize == EXPECTED_HYPOTHESIS_COUNT;
    let data = independently_prepare_validation_projection(
        dataset,
        raw_dataset,
        training_envelope,
        contract,
    )?;
    let recomputed =
        independently_recompute_envelope(claim, training_envelope, contract, data.clone())?;
    let recomputed_envelope_sha256 = hash_serializable(&recomputed)?;
    let projection_exact = envelope.validation_projection_sha256 == data.projection_sha256
        && envelope.validation_row_count == data.rows.len()
        && envelope.feature_order_sha256 == hash_serializable(&data.feature_order)?
        && envelope.preprocessing_sha256 == hash_serializable(&training_envelope.preprocessing)?
        && envelope.target_order == data.target_order;
    let predictions_exact = recomputed.metrics.iter().all(|metric| {
        valid_f64_hex(&metric.mae_f64_bits_hex)
            && valid_f64_hex(&metric.zero_baseline_mae_f64_bits_hex)
    }) && recomputed.exact_artifact_count == EXPECTED_ARTIFACT_COUNT;
    let metrics_exact = recomputed_output_sha256 == recomputed_envelope_sha256
        && envelope.metrics == recomputed.metrics
        && recomputed.metrics.len() == EXPECTED_METRIC_COUNT;
    let hypothesis_tests_exact = envelope
        .metrics
        .iter()
        .filter(|metric| {
            metric
                .component_block_bootstrap_p_value_f64_bits_hex
                .is_some()
        })
        .count()
        == EXPECTED_HYPOTHESIS_COUNT
        && recomputed
            .metrics
            .iter()
            .filter(|metric| {
                metric
                    .component_block_bootstrap_p_value_f64_bits_hex
                    .is_some()
            })
            .count()
            == EXPECTED_HYPOTHESIS_COUNT
        && envelope
            .metrics
            .iter()
            .zip(&recomputed.metrics)
            .all(|(left, right)| {
                left.component_block_bootstrap_p_value_f64_bits_hex
                    == right.component_block_bootstrap_p_value_f64_bits_hex
                    && left.holm_adjusted_p_value_f64_bits_hex
                        == right.holm_adjusted_p_value_f64_bits_hex
                    && left.all_preregistered_thresholds_passed
                        == right.all_preregistered_thresholds_passed
                    && left.evidence_status == right.evidence_status
            });
    let recommendations_exact = envelope.per_target_recommendations
        == recomputed.per_target_recommendations
        && recomputed.per_target_recommendations.len() == EXPECTED_TARGET_COUNT;
    let sealed_holdout_non_access = dataset.rows.iter().all(|row| {
        row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
            || (row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
                && row.target_vector.is_none())
    }) && !envelope.sealed_holdout_features_accessed
        && !envelope.sealed_holdout_labels_accessed
        && !recomputed.sealed_holdout_features_accessed
        && !recomputed.sealed_holdout_labels_accessed;
    let downstream_closed = envelope.validation_evaluation_completed
        && envelope.output_is_untrusted
        && !envelope.independent_output_validation_completed
        && !envelope.official_candidate_selection_completed
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
        (exact_chain, "stage_51_through_stage_63_chain_mismatch"),
        (
            claim_fingerprint,
            "validation_evaluation_claim_fingerprint_or_capability_mismatch",
        ),
        (
            result_fingerprint,
            "validation_evaluation_result_fingerprint_or_shape_mismatch",
        ),
        (
            envelope_fingerprint,
            "validation_evaluation_envelope_binding_mismatch",
        ),
        (
            exact_inputs,
            "training_artifacts_dataset_or_frozen_contract_mismatch",
        ),
        (projection_exact, "validation_projection_bitwise_mismatch"),
        (
            predictions_exact,
            "nine_candidate_prediction_recomputation_failed",
        ),
        (
            metrics_exact,
            "eighty_one_validation_metrics_bitwise_mismatch",
        ),
        (
            hypothesis_tests_exact,
            "fifty_four_component_bootstrap_or_holm_bitwise_mismatch",
        ),
        (
            recommendations_exact,
            "nine_per_target_recommendations_bitwise_mismatch",
        ),
        (
            sealed_holdout_non_access,
            "sealed_holdout_non_access_boundary_mismatch",
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
        exact_inputs,
        projection_exact,
        predictions_exact,
        metrics_exact,
        hypothesis_tests_exact,
        recommendations_exact,
        sealed_holdout_non_access,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_output_sha256,
        recomputed_envelope_sha256,
        recomputed_validation_projection_sha256: data.projection_sha256,
        recomputed_metric_count: recomputed.metrics.len(),
        recomputed_candidate_hypothesis_count: recomputed
            .metrics
            .iter()
            .filter(|metric| {
                metric
                    .component_block_bootstrap_p_value_f64_bits_hex
                    .is_some()
            })
            .count(),
        recomputed_per_target_recommendation_count: recomputed.per_target_recommendations.len(),
        mismatch_reasons,
    })
}

fn independently_prepare_validation_projection(
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    raw_dataset: &HistoricalOutcomeOfflineDataset,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
) -> Result<IndependentValidationData, String> {
    if raw_dataset.dataset_id != dataset.dataset_id
        || raw_dataset.dataset_content_sha256 != dataset.dataset_content_sha256
        || raw_dataset.manifest_sha256 != dataset.dataset_manifest_sha256
        || raw_dataset.candidate_set_sha256 != dataset.candidate_set_sha256
        || training_envelope.feature_order.len() != EXPECTED_FEATURE_COUNT
        || training_envelope.preprocessing.len() != EXPECTED_FEATURE_COUNT
        || training_envelope.target_order != contract.target_vector_order
    {
        return Err("独立 validation 投影的精确输入绑定漂移".to_string());
    }
    let mut validation_rows = dataset
        .rows
        .iter()
        .filter(|row| {
            row.split == HistoricalOutcomeOfflineDatasetTransformationSplit::Validation
                && row.target_visibility
                    == HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
                && row.target_vector.is_none()
        })
        .collect::<Vec<_>>();
    validation_rows.sort_by(|left, right| left.dataset_entry_id.cmp(&right.dataset_entry_id));
    if validation_rows.is_empty() {
        return Err("validation 分区没有可独立复算的行".to_string());
    }
    let allowed = validation_rows
        .iter()
        .map(|row| row.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    let sealed = dataset
        .rows
        .iter()
        .filter(|row| {
            row.split == HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
        })
        .map(|row| row.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    if !allowed.is_disjoint(&sealed) {
        return Err("validation 与 sealed holdout entry 重叠".to_string());
    }
    let raw_index = raw_dataset
        .entries
        .iter()
        .map(|entry| (entry.entry_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let preprocessing = training_envelope
        .preprocessing
        .iter()
        .map(|record| (record.feature_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::with_capacity(validation_rows.len());
    for row in validation_rows {
        let raw = raw_index
            .get(row.dataset_entry_id.as_str())
            .ok_or_else(|| "独立 validation 投影缺少原始结果条目".to_string())?;
        if raw.entry_sha256 != row.dataset_entry_sha256
            || raw.decision_available_at != row.decision_available_at
            || raw.raw_validated_metrics.len() != REQUIRED_HORIZONS.len()
        {
            return Err(format!(
                "validation 条目 {} 点时绑定漂移",
                row.dataset_entry_id
            ));
        }
        let features =
            independent_preprocess_row(row, &training_envelope.feature_order, &preprocessing)?;
        let targets = independent_project_targets(raw, &training_envelope.target_order)?;
        rows.push(IndependentValidationRow {
            dataset_entry_id: row.dataset_entry_id.clone(),
            component_id: row.component_id.clone(),
            features,
            targets,
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
    Ok(IndependentValidationData {
        feature_order: training_envelope.feature_order.clone(),
        target_order: training_envelope.target_order.clone(),
        rows,
        projection_sha256,
    })
}

pub(crate) fn independent_preprocess_row<'a>(
    row: &HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    feature_order: &[String],
    preprocessing: &BTreeMap<&'a str, &'a super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::HistoricalOutcomeTrainingFeaturePreprocessingRecord>,
) -> Result<Vec<Option<f64>>, String> {
    let features = row
        .feature_records
        .iter()
        .map(|feature| (feature.feature_id.as_str(), feature))
        .collect::<BTreeMap<_, _>>();
    if features.len() != EXPECTED_FEATURE_COUNT || preprocessing.len() != EXPECTED_FEATURE_COUNT {
        return Err("validation 特征或冻结预处理不是精确 65 项".to_string());
    }
    let mut output = Vec::with_capacity(feature_order.len());
    for feature_id in feature_order {
        let feature = features
            .get(feature_id.as_str())
            .ok_or_else(|| format!("validation 行缺少特征 {feature_id}"))?;
        let prep = preprocessing
            .get(feature_id.as_str())
            .ok_or_else(|| format!("训练工件缺少预处理 {feature_id}"))?;
        if feature.available_at_utc > row.decision_available_at || !prep.missingness_preserved {
            return Err(format!("特征 {feature_id} 点时或缺失语义漂移"));
        }
        if feature.is_missing {
            if feature.value.is_some() {
                return Err(format!("缺失特征 {feature_id} 携带数值"));
            }
            output.push(None);
        } else {
            let raw = feature
                .value
                .as_deref()
                .ok_or_else(|| format!("非缺失特征 {feature_id} 没有数值"))?
                .parse::<f64>()
                .map_err(|_| format!("特征 {feature_id} 不是数值"))?;
            let mean = parse_f64_hex(&prep.train_mean_f64_bits_hex)?;
            let scale = parse_f64_hex(&prep.train_scale_f64_bits_hex)?;
            if !raw.is_finite() || scale <= 0.0 {
                return Err(format!("特征 {feature_id} 或尺度无效"));
            }
            output.push(Some((raw - mean) / scale));
        }
    }
    Ok(output)
}

pub(crate) fn independent_project_targets(
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
    let mut output = Vec::with_capacity(target_order.len());
    for target_id in target_order {
        let (metric_id, horizon) = independent_parse_target_id(target_id)?;
        let metric = metrics
            .get(&horizon)
            .ok_or_else(|| format!("validation 缺少 {horizon} 日结果"))?;
        let value = match metric_id {
            "asset_return" => metric.asset_return,
            "excess_return" => metric.excess_return,
            "asset_max_drawdown" => metric.asset_max_drawdown,
            _ => return Err(format!("未知目标 {target_id}")),
        };
        if !value.is_finite() {
            return Err(format!("目标 {target_id} 不是有限值"));
        }
        output.push(value);
    }
    Ok(output)
}

fn independent_parse_target_id(target_id: &str) -> Result<(&str, u16), String> {
    for metric in ["asset_return", "excess_return", "asset_max_drawdown"] {
        let prefix = format!("{metric}_");
        if let Some(value) = target_id
            .strip_prefix(&prefix)
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
    Err(format!("目标不属于冻结九目标：{target_id}"))
}

fn independently_recompute_envelope(
    claim: &super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts::HistoricalOutcomeValidationEvaluationExecutionAttemptClaim,
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
    data: IndependentValidationData,
) -> Result<HistoricalOutcomeValidationEvaluationUntrustedEnvelope, String> {
    independent_validate_contract(training_envelope, contract, &data)?;
    let components = data
        .rows
        .iter()
        .map(|row| row.component_id.clone())
        .collect::<BTreeSet<_>>();
    let mut predictions = BTreeMap::new();
    for artifact in &training_envelope.model_artifacts {
        independent_validate_artifact(artifact, contract)?;
        let key = (
            independent_algorithm_id(&artifact.algorithm).to_string(),
            artifact.random_seed,
        );
        if predictions
            .insert(key, independent_predict_artifact(artifact, &data)?)
            .is_some()
        {
            return Err("九候选包含重复算法/种子".to_string());
        }
    }
    if predictions.len() != EXPECTED_ARTIFACT_COUNT {
        return Err("独立评估不是精确九候选".to_string());
    }
    let algorithms = [
        HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline,
        HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
        HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
    ];
    let mut metrics = Vec::with_capacity(EXPECTED_METRIC_COUNT);
    let mut candidate_positions = Vec::with_capacity(EXPECTED_HYPOTHESIS_COUNT);
    let mut candidate_p_values = Vec::with_capacity(EXPECTED_HYPOTHESIS_COUNT);
    for algorithm in algorithms {
        for seed in EXPECTED_SEEDS {
            let predicted = predictions
                .get(&(independent_algorithm_id(&algorithm).to_string(), seed))
                .ok_or_else(|| "独立评估缺少算法/种子预测".to_string())?;
            let zero = predictions
                .get(&("frozen_zero_prediction_baseline".to_string(), seed))
                .ok_or_else(|| "独立评估缺少同种子零基线".to_string())?;
            for target_index in 0..data.target_order.len() {
                let actual = data
                    .rows
                    .iter()
                    .map(|row| row.targets[target_index])
                    .collect::<Vec<_>>();
                let candidate = predicted
                    .iter()
                    .map(|row| row[target_index])
                    .collect::<Vec<_>>();
                let baseline = zero.iter().map(|row| row[target_index]).collect::<Vec<_>>();
                let mae = independent_mae(&actual, &candidate)?;
                let zero_mae = independent_mae(&actual, &baseline)?;
                let improvement = if zero_mae.abs() <= NUMERIC_EPSILON {
                    0.0
                } else {
                    (zero_mae - mae) / zero_mae
                };
                let is_baseline = algorithm
                    == HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline;
                let p_value = if is_baseline {
                    None
                } else {
                    Some(independent_component_bootstrap(
                        &data.rows,
                        &actual,
                        &candidate,
                        &baseline,
                        contract.bootstrap_replications,
                        contract.bootstrap_random_seed ^ seed ^ target_index as u64,
                    )?)
                };
                let position = metrics.len();
                if let Some(value) = p_value {
                    candidate_positions.push(position);
                    candidate_p_values.push(value);
                }
                metrics.push(HistoricalOutcomeValidationEvaluationMetric {
                    algorithm: algorithm.clone(),
                    random_seed: seed,
                    target_id: data.target_order[target_index].clone(),
                    validation_row_count: data.rows.len(),
                    independent_component_count: components.len(),
                    mae_f64_bits_hex: f64_hex(mae),
                    zero_baseline_mae_f64_bits_hex: f64_hex(zero_mae),
                    relative_mae_improvement_f64_bits_hex: f64_hex(improvement),
                    component_block_bootstrap_p_value_f64_bits_hex: p_value.map(f64_hex),
                    holm_adjusted_p_value_f64_bits_hex: None,
                    spearman_f64_bits_hex: independent_spearman(&actual, &candidate).map(f64_hex),
                    directional_accuracy_f64_bits_hex: f64_hex(independent_directional_accuracy(
                        &actual, &candidate,
                    )),
                    calibration_slope_f64_bits_hex: independent_calibration_slope(
                        &actual, &candidate,
                    )
                    .map(f64_hex),
                    evidence_status: if data.rows.len() < contract.minimum_validation_rows as usize
                        || components.len() < contract.minimum_independent_components as usize
                    {
                        "insufficient_evidence".to_string()
                    } else if is_baseline {
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
        || candidate_p_values.len() != EXPECTED_HYPOTHESIS_COUNT
    {
        return Err("独立评估指标或候选假设数量漂移".to_string());
    }
    let adjusted = independent_holm(&candidate_p_values);
    for (position, q) in candidate_positions.into_iter().zip(adjusted) {
        let metric = &mut metrics[position];
        metric.holm_adjusted_p_value_f64_bits_hex = Some(f64_hex(q));
        if metric.evidence_status != "insufficient_evidence" {
            metric.all_preregistered_thresholds_passed =
                independent_metric_passes(metric, contract)?;
            metric.evidence_status = if metric.all_preregistered_thresholds_passed {
                "all_preregistered_thresholds_passed"
            } else {
                "one_or_more_preregistered_thresholds_failed"
            }
            .to_string();
        }
    }
    let recommendations = independent_recommendations(&metrics, contract, &data.target_order)?;
    Ok(HistoricalOutcomeValidationEvaluationUntrustedEnvelope {
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
    })
}

fn independent_validate_contract(
    training_envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
    data: &IndependentValidationData,
) -> Result<(), String> {
    let valid = contract.exact_feature_count == EXPECTED_FEATURE_COUNT
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
        && hash_serializable(&training_envelope.feature_order)? == contract.feature_order_sha256
        && hash_serializable(&training_envelope.preprocessing)? == contract.preprocessing_sha256
        && training_envelope.feature_order == data.feature_order
        && training_envelope.target_order == data.target_order;
    if !valid {
        return Err("冻结评估合同或九候选工件漂移".to_string());
    }
    Ok(())
}

fn independent_validate_artifact(
    artifact: &HistoricalOutcomeTrainingUntrustedModelArtifact,
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
) -> Result<(), String> {
    let binding = contract.candidate_bindings.iter().find(|binding| {
        binding.algorithm_id == independent_algorithm_id(&artifact.algorithm)
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
        return Err("候选模型工件与冻结绑定不一致或越权".to_string());
    }
    Ok(())
}

fn independent_predict_artifact(
    artifact: &HistoricalOutcomeTrainingUntrustedModelArtifact,
    data: &IndependentValidationData,
) -> Result<Vec<Vec<f64>>, String> {
    let models = artifact
        .target_models
        .iter()
        .map(|model| (model.target_id.as_str(), model))
        .collect::<BTreeMap<_, _>>();
    if models.len() != EXPECTED_TARGET_COUNT {
        return Err("候选目标模型重复或缺失".to_string());
    }
    let feature_positions = data
        .feature_order
        .iter()
        .enumerate()
        .map(|(index, feature)| (feature.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::with_capacity(data.rows.len());
    for row in &data.rows {
        let mut predictions = Vec::with_capacity(EXPECTED_TARGET_COUNT);
        for target_id in &data.target_order {
            let model = models
                .get(target_id.as_str())
                .ok_or_else(|| format!("候选缺少目标模型 {target_id}"))?;
            predictions.push(independent_predict_target(
                model,
                &feature_positions,
                &row.features,
            )?);
        }
        output.push(predictions);
    }
    Ok(output)
}

pub(crate) fn independent_predict_target(
    model: &HistoricalOutcomeTrainingTargetModel,
    feature_positions: &BTreeMap<&str, usize>,
    features: &[Option<f64>],
) -> Result<f64, String> {
    let prediction = match &model.model {
        HistoricalOutcomeTrainingTargetModelBody::FrozenZeroPrediction {
            prediction_f64_bits_hex,
        } => parse_f64_hex(prediction_f64_bits_hex)?,
        HistoricalOutcomeTrainingTargetModelBody::RidgeRegression {
            intercept_f64_bits_hex,
            coefficients,
            ..
        } => {
            if coefficients.len() != EXPECTED_FEATURE_COUNT {
                return Err("ridge 不是精确 65 项系数".to_string());
            }
            let mut sum = parse_f64_hex(intercept_f64_bits_hex)?;
            for coefficient in coefficients {
                let index = *feature_positions
                    .get(coefficient.feature_id.as_str())
                    .ok_or_else(|| "ridge 引用未知特征".to_string())?;
                match features[index] {
                    Some(value) => {
                        sum += value
                            * parse_f64_hex(&coefficient.observed_value_coefficient_f64_bits_hex)?;
                    }
                    None => {
                        sum +=
                            parse_f64_hex(&coefficient.missing_indicator_coefficient_f64_bits_hex)?;
                    }
                }
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
                sum += rate * independent_predict_stump(stump, feature_positions, features)?;
            }
            sum
        }
    };
    if !prediction.is_finite() {
        return Err("候选预测不是有限值".to_string());
    }
    Ok(prediction)
}

fn independent_predict_stump(
    stump: &HistoricalOutcomeTrainingBoostedStump,
    feature_positions: &BTreeMap<&str, usize>,
    features: &[Option<f64>],
) -> Result<f64, String> {
    let missing = parse_f64_hex(&stump.missing_leaf_f64_bits_hex)?;
    let (Some(feature_id), Some(threshold)) = (&stump.feature_id, &stump.threshold_f64_bits_hex)
    else {
        return Ok(missing);
    };
    let index = *feature_positions
        .get(feature_id.as_str())
        .ok_or_else(|| "boosted stump 引用未知特征".to_string())?;
    match features[index] {
        Some(value) if value <= parse_f64_hex(threshold)? => {
            parse_f64_hex(&stump.low_leaf_f64_bits_hex)
        }
        Some(_) => parse_f64_hex(&stump.high_leaf_f64_bits_hex),
        None => Ok(missing),
    }
}

pub(crate) fn independent_component_bootstrap(
    rows: &[IndependentValidationRow],
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
        return Err("独立 component bootstrap 输入无效".to_string());
    }
    let mut block_values = BTreeMap::<&str, (f64, usize)>::new();
    for index in 0..rows.len() {
        let delta =
            (actual[index] - baseline[index]).abs() - (actual[index] - candidate[index]).abs();
        let entry = block_values
            .entry(rows[index].component_id.as_str())
            .or_insert((0.0, 0));
        entry.0 += delta;
        entry.1 += 1;
    }
    let block_means = block_values
        .values()
        .map(|(sum, count)| *sum / *count as f64)
        .collect::<Vec<_>>();
    if block_means.is_empty() {
        return Err("独立 component bootstrap 没有 block".to_string());
    }
    let mut state = seed.max(1);
    let mut non_positive = 0_u32;
    for _ in 0..replications {
        let mut sum = 0.0;
        for _ in 0..block_means.len() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            sum += block_means[(state % block_means.len() as u64) as usize];
        }
        if sum / block_means.len() as f64 <= 0.0 {
            non_positive += 1;
        }
    }
    Ok((f64::from(non_positive) + 1.0) / (f64::from(replications) + 1.0))
}

pub(crate) fn independent_holm(values: &[f64]) -> Vec<f64> {
    let mut order = (0..values.len()).collect::<Vec<_>>();
    order.sort_by(|left, right| {
        values[*left]
            .total_cmp(&values[*right])
            .then(left.cmp(right))
    });
    let mut output = vec![1.0; values.len()];
    let mut floor = 0.0_f64;
    for (rank, original_index) in order.into_iter().enumerate() {
        let raw = values[original_index] * (values.len() - rank) as f64;
        floor = floor.max(raw.min(1.0));
        output[original_index] = floor;
    }
    output
}

fn independent_metric_passes(
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

fn independent_recommendations(
    metrics: &[HistoricalOutcomeValidationEvaluationMetric],
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
    target_order: &[String],
) -> Result<Vec<HistoricalOutcomeValidationEvaluationPerTargetRecommendation>, String> {
    let mut output = Vec::with_capacity(target_order.len());
    for target_id in target_order {
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
            output.push(
                HistoricalOutcomeValidationEvaluationPerTargetRecommendation {
                    target_id: target_id.clone(),
                    status: "insufficient_evidence".to_string(),
                    recommended_algorithm: None,
                    three_seed_median_mae_f64_bits_hex: None,
                    rationale:
                        "validation 行数或独立 component 未达到预注册门槛；不得选模或声明有效。"
                            .to_string(),
                    all_three_seeds_passed: false,
                    official_selection: false,
                },
            );
            continue;
        }
        let mut candidates = Vec::new();
        for algorithm in [
            HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
            HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
        ] {
            let selected = target_metrics
                .iter()
                .filter(|metric| metric.algorithm == algorithm)
                .collect::<Vec<_>>();
            if selected.len() == EXPECTED_SEEDS.len()
                && selected
                    .iter()
                    .all(|metric| metric.all_preregistered_thresholds_passed)
            {
                let mut maes = selected
                    .iter()
                    .map(|metric| parse_f64_hex(&metric.mae_f64_bits_hex))
                    .collect::<Result<Vec<_>, _>>()?;
                maes.sort_by(f64::total_cmp);
                candidates.push((algorithm, maes[1]));
            }
        }
        if candidates.is_empty() {
            output.push(
                HistoricalOutcomeValidationEvaluationPerTargetRecommendation {
                    target_id: target_id.clone(),
                    status: "no_candidate_passed_all_three_seeds".to_string(),
                    recommended_algorithm: None,
                    three_seed_median_mae_f64_bits_hex: None,
                    rationale: "ridge 与 boosted 均未在三个冻结种子上同时通过全部预注册门槛。"
                        .to_string(),
                    all_three_seeds_passed: false,
                    official_selection: false,
                },
            );
            continue;
        }
        candidates.sort_by(|left, right| {
            left.1.total_cmp(&right.1).then_with(|| {
                let preferred = contract.tie_break_preferred_algorithm_id.as_str();
                let left_rank = usize::from(independent_algorithm_id(&left.0) != preferred);
                let right_rank = usize::from(independent_algorithm_id(&right.0) != preferred);
                left_rank.cmp(&right_rank)
            })
        });
        let selected = candidates.remove(0);
        output.push(HistoricalOutcomeValidationEvaluationPerTargetRecommendation {
            target_id: target_id.clone(),
            status: "untrusted_per_target_recommendation_pending_independent_validation".to_string(),
            recommended_algorithm: Some(selected.0),
            three_seed_median_mae_f64_bits_hex: Some(f64_hex(selected.1)),
            rationale: "仅按预注册三种子全部通过、validation MAE 中位数和固定 ridge 平局规则形成不可信建议；尚非正式选择。".to_string(),
            all_three_seeds_passed: true,
            official_selection: false,
        });
    }
    Ok(output)
}

pub(crate) fn independent_mae(actual: &[f64], predicted: &[f64]) -> Result<f64, String> {
    if actual.len() != predicted.len() || actual.is_empty() {
        return Err("MAE 输入无效".to_string());
    }
    let mut sum = 0.0;
    for index in 0..actual.len() {
        sum += (actual[index] - predicted[index]).abs();
    }
    Ok(sum / actual.len() as f64)
}

pub(crate) fn independent_directional_accuracy(actual: &[f64], predicted: &[f64]) -> f64 {
    let sign = |value: f64| {
        if value > NUMERIC_EPSILON {
            1_i8
        } else if value < -NUMERIC_EPSILON {
            -1_i8
        } else {
            0_i8
        }
    };
    let mut matches = 0_usize;
    for index in 0..actual.len() {
        if sign(actual[index]) == sign(predicted[index]) {
            matches += 1;
        }
    }
    matches as f64 / actual.len() as f64
}

pub(crate) fn independent_spearman(left: &[f64], right: &[f64]) -> Option<f64> {
    independent_pearson(&independent_ranks(left), &independent_ranks(right))
}

fn independent_ranks(values: &[f64]) -> Vec<f64> {
    let mut order = (0..values.len()).collect::<Vec<_>>();
    order.sort_by(|left, right| {
        values[*left]
            .total_cmp(&values[*right])
            .then(left.cmp(right))
    });
    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < order.len() {
        let mut end = start + 1;
        while end < order.len() && values[order[end]].to_bits() == values[order[start]].to_bits() {
            end += 1;
        }
        let rank = (start + end - 1) as f64 / 2.0 + 1.0;
        for position in start..end {
            ranks[order[position]] = rank;
        }
        start = end;
    }
    ranks
}

fn independent_pearson(left: &[f64], right: &[f64]) -> Option<f64> {
    if left.len() != right.len() || left.len() < 2 {
        return None;
    }
    let left_mean = left.iter().sum::<f64>() / left.len() as f64;
    let right_mean = right.iter().sum::<f64>() / right.len() as f64;
    let mut covariance = 0.0;
    let mut left_variance = 0.0;
    let mut right_variance = 0.0;
    for index in 0..left.len() {
        let left_delta = left[index] - left_mean;
        let right_delta = right[index] - right_mean;
        covariance += left_delta * right_delta;
        left_variance += left_delta.powi(2);
        right_variance += right_delta.powi(2);
    }
    let denominator = (left_variance * right_variance).sqrt();
    (denominator > NUMERIC_EPSILON).then_some(covariance / denominator)
}

pub(crate) fn independent_calibration_slope(actual: &[f64], predicted: &[f64]) -> Option<f64> {
    if actual.len() != predicted.len() || actual.len() < 2 {
        return None;
    }
    let predicted_mean = predicted.iter().sum::<f64>() / predicted.len() as f64;
    let actual_mean = actual.iter().sum::<f64>() / actual.len() as f64;
    let mut denominator = 0.0;
    let mut numerator = 0.0;
    for index in 0..actual.len() {
        let prediction_delta = predicted[index] - predicted_mean;
        denominator += prediction_delta.powi(2);
        numerator += prediction_delta * (actual[index] - actual_mean);
    }
    (denominator > NUMERIC_EPSILON).then_some(numerator / denominator)
}

pub(crate) fn independent_algorithm_id(
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

fn validate_request(
    attempt_id: &str,
    request: &ValidateHistoricalOutcomeValidationEvaluationOutputRequest,
) -> Result<(), String> {
    let hashes = [
        request.expected_claim_sha256.as_str(),
        request.expected_result_sha256.as_str(),
        request.expected_output_sha256.as_str(),
        request.expected_authorization_review_sha256.as_str(),
        request.expected_isolated_runner_spec_sha256.as_str(),
        request.expected_implementation_sha256.as_str(),
        request.expected_implementation_review_sha256.as_str(),
        request.expected_candidate_set_sha256.as_str(),
        request.expected_upstream_validation_sha256.as_str(),
        request.expected_upstream_output_sha256.as_str(),
        request.expected_training_store_dataset_sha256.as_str(),
        request.expected_rows_sha256.as_str(),
        request.expected_excluded_rows_sha256.as_str(),
        request.expected_target_commitments_sha256.as_str(),
        request.expected_validation_projection_sha256.as_str(),
        request.expected_feature_order_sha256.as_str(),
        request.expected_preprocessing_sha256.as_str(),
    ];
    if !valid_id(attempt_id) || hashes.iter().any(|value| !valid_sha256(value)) {
        return Err("validation 评估输出独立验证请求中的 ID 或 SHA-256 无效".to_string());
    }
    if !request.independent_reopen_and_second_implementation_recomputation_confirmed
        || !request.exact_current_stage_51_through_stage_63_binding_confirmed
        || !request.exact_validation_projection_and_nine_candidate_predictions_confirmed
        || !request.all_eighty_one_metrics_fifty_four_hypotheses_and_nine_recommendations_bitwise_recomputed_confirmed
        || !request.sealed_holdout_remains_unread_confirmed
        || !request.no_selection_store_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err("必须确认第二实现复算、完整链、validation 投影、81/54/9 逐位核对、holdout 隔离和零下游权限".to_string());
    }
    Ok(())
}

fn verify_expected_bindings(
    attempt: &CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt,
    request: &ValidateHistoricalOutcomeValidationEvaluationOutputRequest,
) -> Result<(), String> {
    let claim = &attempt.claim;
    let result = &attempt.result;
    let envelope = result
        .untrusted_evaluation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 63 结果缺少评估 envelope".to_string())?;
    if claim.claim_sha256 != request.expected_claim_sha256
        || result.result_sha256 != request.expected_result_sha256
        || result.output_sha256.as_deref() != Some(request.expected_output_sha256.as_str())
        || claim.authorization_review_sha256 != request.expected_authorization_review_sha256
        || claim.isolated_runner_spec_sha256 != request.expected_isolated_runner_spec_sha256
        || claim.implementation_sha256 != request.expected_implementation_sha256
        || claim.implementation_review_sha256 != request.expected_implementation_review_sha256
        || claim.candidate_set_sha256 != request.expected_candidate_set_sha256
        || claim.upstream_validation_sha256 != request.expected_upstream_validation_sha256
        || claim.upstream_output_sha256 != request.expected_upstream_output_sha256
        || claim.training_store_dataset_sha256 != request.expected_training_store_dataset_sha256
        || claim.rows_sha256 != request.expected_rows_sha256
        || claim.excluded_rows_sha256 != request.expected_excluded_rows_sha256
        || claim.target_commitments_sha256 != request.expected_target_commitments_sha256
        || envelope.validation_projection_sha256 != request.expected_validation_projection_sha256
        || envelope.feature_order_sha256 != request.expected_feature_order_sha256
        || envelope.preprocessing_sha256 != request.expected_preprocessing_sha256
    {
        return Err(
            "validation 评估 claim/result、输出或完整上游绑定已变化，请刷新后重试".to_string(),
        );
    }
    Ok(())
}

fn prior_actor_ids(
    attempt: &CompletedHistoricalOutcomeValidationEvaluationExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeValidationEvaluationFirstExecutionAuthorization,
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
    record: &HistoricalOutcomeValidationEvaluationOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_execution_and_complete_prior_chain
        && record.exact_current_stage_51_through_stage_63_chain_verified
        && record.claim_fingerprint_independently_verified
        && record.result_fingerprint_independently_verified
        && record.envelope_fingerprint_independently_verified
        && record.exact_training_artifacts_store_dataset_and_contract_verified
        && record.validation_projection_independently_reconstructed
        && record.exact_nine_candidate_predictions_bitwise_recomputed
        && record.exact_eighty_one_metrics_bitwise_recomputed
        && record.exact_fifty_four_component_bootstrap_and_holm_tests_bitwise_recomputed
        && record.exact_nine_per_target_recommendations_bitwise_recomputed
        && record.sealed_holdout_non_access_verified
        && record.no_selection_or_downstream_authority_verified
        && record.recomputed_metric_count == EXPECTED_METRIC_COUNT
        && record.recomputed_candidate_hypothesis_count == EXPECTED_HYPOTHESIS_COUNT
        && record.recomputed_per_target_recommendation_count == EXPECTED_TARGET_COUNT
        && record.mismatch_reasons.is_empty();
    let closed = !record.official_candidate_selection_authorized
        && !record.sealed_holdout_access_authorized
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
        record.candidate_set_sha256.as_str(),
        record.upstream_validation_sha256.as_str(),
        record.upstream_output_sha256.as_str(),
        record.training_store_dataset_sha256.as_str(),
        record.rows_sha256.as_str(),
        record.excluded_rows_sha256.as_str(),
        record.target_commitments_sha256.as_str(),
        record.validation_projection_sha256.as_str(),
        record.feature_order_sha256.as_str(),
        record.preprocessing_sha256.as_str(),
        record.validator_implementation_sha256.as_str(),
        record.recomputed_claim_sha256.as_str(),
        record.recomputed_result_sha256.as_str(),
        record.recomputed_output_sha256.as_str(),
        record.recomputed_envelope_sha256.as_str(),
        record.recomputed_validation_projection_sha256.as_str(),
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
        || !record.excluded_prior_actor_ids.iter().any(|actor| actor == &record.invoked_by)
        || record.mismatch_reasons.len() > MAX_MISMATCH_REASONS
        || record.validation_evaluation_output_independently_validated != passed
        || record.future_per_target_candidate_admission_review_eligible != passed
        || record.validation_evaluation_output_independently_validated
            != (record.verdict
                == HistoricalOutcomeValidationEvaluationOutputValidationVerdict::IndependentlyValidatedUntrustedValidationEvaluation)
        || !closed
    {
        return Err("validation evaluation output validation record is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeValidationEvaluationOutputValidationRecord>, String> {
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
            return Err("validation 评估输出验证目录包含无效 attempt ID".to_string());
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
                HistoricalOutcomeValidationEvaluationOutputValidationRecord,
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
                return Err("validation 评估输出独立验证含重放、误命名或重复哈希".to_string());
            }
            records.push(record);
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeValidationEvaluationOutputValidationRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    hash_bytes(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
}

pub(crate) fn fingerprint_without<T: Serialize>(
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
    hash_serializable(&value)
}

pub(crate) fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(hash_bytes(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
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
    value.len() == 16
        && value.bytes().all(|value| value.is_ascii_hexdigit())
        && parse_f64_hex(value).is_ok()
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
    decision_root(state).join("validation-evaluation-output-validations")
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
        "validation-evaluation-output-validation-{attempt_id}.lock"
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
        .map_err(|_| "已有同一 validation 评估输出独立验证正在进行".to_string())?;
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

    fn request() -> ValidateHistoricalOutcomeValidationEvaluationOutputRequest {
        ValidateHistoricalOutcomeValidationEvaluationOutputRequest {
            expected_claim_sha256: "a".repeat(64),
            expected_result_sha256: "b".repeat(64),
            expected_output_sha256: "c".repeat(64),
            expected_authorization_review_sha256: "d".repeat(64),
            expected_isolated_runner_spec_sha256: "e".repeat(64),
            expected_implementation_sha256: "f".repeat(64),
            expected_implementation_review_sha256: "1".repeat(64),
            expected_candidate_set_sha256: "2".repeat(64),
            expected_upstream_validation_sha256: "3".repeat(64),
            expected_upstream_output_sha256: "4".repeat(64),
            expected_training_store_dataset_sha256: "5".repeat(64),
            expected_rows_sha256: "6".repeat(64),
            expected_excluded_rows_sha256: "7".repeat(64),
            expected_target_commitments_sha256: "8".repeat(64),
            expected_validation_projection_sha256: "9".repeat(64),
            expected_feature_order_sha256: "a".repeat(64),
            expected_preprocessing_sha256: "b".repeat(64),
            independent_reopen_and_second_implementation_recomputation_confirmed: true,
            exact_current_stage_51_through_stage_63_binding_confirmed: true,
            exact_validation_projection_and_nine_candidate_predictions_confirmed: true,
            all_eighty_one_metrics_fifty_four_hypotheses_and_nine_recommendations_bitwise_recomputed_confirmed: true,
            sealed_holdout_remains_unread_confirmed: true,
            no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    fn validation_rows() -> Vec<IndependentValidationRow> {
        vec![
            IndependentValidationRow {
                dataset_entry_id: "entry-a".to_string(),
                component_id: "component-a".to_string(),
                features: vec![Some(1.0); EXPECTED_FEATURE_COUNT],
                targets: vec![0.1; EXPECTED_TARGET_COUNT],
            },
            IndependentValidationRow {
                dataset_entry_id: "entry-b".to_string(),
                component_id: "component-b".to_string(),
                features: vec![Some(-1.0); EXPECTED_FEATURE_COUNT],
                targets: vec![-0.2; EXPECTED_TARGET_COUNT],
            },
        ]
    }

    fn valid_record() -> HistoricalOutcomeValidationEvaluationOutputValidationRecord {
        let mut record = HistoricalOutcomeValidationEvaluationOutputValidationRecord {
            schema_version: RECORD_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            validation_id: String::new(),
            validation_sha256: String::new(),
            attempt_id: "a".repeat(32),
            claim_sha256: "b".repeat(64),
            result_id: "c".repeat(32),
            result_sha256: "d".repeat(64),
            output_sha256: "e".repeat(64),
            authorization_review_id: "f".repeat(32),
            authorization_review_sha256: "1".repeat(64),
            isolated_runner_id: "2".repeat(32),
            isolated_runner_spec_sha256: "3".repeat(64),
            implementation_id: "4".repeat(32),
            implementation_sha256: "5".repeat(64),
            implementation_review_sha256: "6".repeat(64),
            candidate_set_sha256: "7".repeat(64),
            upstream_validation_sha256: "8".repeat(64),
            upstream_output_sha256: "9".repeat(64),
            training_store_dataset_sha256: "a".repeat(64),
            rows_sha256: "b".repeat(64),
            excluded_rows_sha256: "c".repeat(64),
            target_commitments_sha256: "d".repeat(64),
            validation_projection_sha256: "e".repeat(64),
            feature_order_sha256: "f".repeat(64),
            preprocessing_sha256: "1".repeat(64),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
            validator_implementation_sha256: validator_implementation_sha256(),
            validated_at: Utc::now(),
            validated_by: "validator".to_string(),
            invoked_by: "executor".to_string(),
            excluded_prior_actor_ids: vec!["executor".to_string(), "reviewer".to_string()],
            validator_independent_from_execution_and_complete_prior_chain: true,
            exact_current_stage_51_through_stage_63_chain_verified: true,
            claim_fingerprint_independently_verified: true,
            result_fingerprint_independently_verified: true,
            envelope_fingerprint_independently_verified: true,
            exact_training_artifacts_store_dataset_and_contract_verified: true,
            validation_projection_independently_reconstructed: true,
            exact_nine_candidate_predictions_bitwise_recomputed: true,
            exact_eighty_one_metrics_bitwise_recomputed: true,
            exact_fifty_four_component_bootstrap_and_holm_tests_bitwise_recomputed: true,
            exact_nine_per_target_recommendations_bitwise_recomputed: true,
            sealed_holdout_non_access_verified: true,
            no_selection_or_downstream_authority_verified: true,
            recomputed_claim_sha256: "2".repeat(64),
            recomputed_result_sha256: "3".repeat(64),
            recomputed_output_sha256: "4".repeat(64),
            recomputed_envelope_sha256: "4".repeat(64),
            recomputed_validation_projection_sha256: "5".repeat(64),
            recomputed_metric_count: EXPECTED_METRIC_COUNT,
            recomputed_candidate_hypothesis_count: EXPECTED_HYPOTHESIS_COUNT,
            recomputed_per_target_recommendation_count: EXPECTED_TARGET_COUNT,
            mismatch_reasons: Vec::new(),
            verdict: HistoricalOutcomeValidationEvaluationOutputValidationVerdict::IndependentlyValidatedUntrustedValidationEvaluation,
            validation_evaluation_output_independently_validated: true,
            future_per_target_candidate_admission_review_eligible: true,
            official_candidate_selection_authorized: false,
            sealed_holdout_access_authorized: false,
            model_artifact_store_written: false,
            metric_store_written: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        record
    }

    #[test]
    fn request_requires_all_confirmations_and_valid_hashes() {
        let mut value = request();
        assert!(validate_request(&"a".repeat(32), &value).is_ok());
        value.sealed_holdout_remains_unread_confirmed = false;
        assert!(validate_request(&"a".repeat(32), &value).is_err());
        value.sealed_holdout_remains_unread_confirmed = true;
        value.expected_output_sha256 = "bad".to_string();
        assert!(validate_request(&"a".repeat(32), &value).is_err());
    }

    #[test]
    fn validator_must_be_outside_complete_actor_chain() {
        let excluded = vec!["executor".to_string(), "reviewer".to_string()];
        assert!(!actor_is_independent("executor", &excluded));
        assert!(actor_is_independent("validator", &excluded));
    }

    #[test]
    fn second_bootstrap_path_is_deterministic_and_component_bounded() {
        let rows = validation_rows();
        let actual = vec![0.1, -0.2];
        let candidate = vec![0.08, -0.15];
        let baseline = vec![0.0, 0.0];
        let left = independent_component_bootstrap(&rows, &actual, &candidate, &baseline, 500, 17)
            .unwrap();
        let right = independent_component_bootstrap(&rows, &actual, &candidate, &baseline, 500, 17)
            .unwrap();
        assert_eq!(left.to_bits(), right.to_bits());
        assert!((0.0..=1.0).contains(&left));
    }

    #[test]
    fn second_holm_path_is_monotone_in_sorted_order() {
        let input = vec![0.01, 0.04, 0.02, 0.50];
        let output = independent_holm(&input);
        assert_eq!(output.len(), input.len());
        let mut order = (0..input.len()).collect::<Vec<_>>();
        order.sort_by(|left, right| input[*left].total_cmp(&input[*right]));
        assert!(
            order
                .windows(2)
                .all(|pair| output[pair[0]] <= output[pair[1]])
        );
    }

    #[test]
    fn bitwise_encoding_rejects_one_ulp_equivalence() {
        let value = 1.0_f64;
        let next = f64::from_bits(value.to_bits() + 1);
        assert_ne!(f64_hex(value), f64_hex(next));
    }

    #[test]
    fn rank_and_correlation_second_path_handle_ties() {
        let ranks = independent_ranks(&[1.0, 1.0, 3.0]);
        assert_eq!(ranks, vec![1.5, 1.5, 3.0]);
        assert_eq!(independent_spearman(&[1.0, 2.0], &[1.0, 2.0]), Some(1.0));
    }

    #[test]
    fn valid_record_keeps_selection_and_execution_closed() {
        let record = valid_record();
        assert!(validate_record(&record).is_ok());
        assert!(record.validation_evaluation_output_independently_validated);
        assert!(!record.official_candidate_selection_authorized);
        assert!(!record.trading_authorized);
    }

    #[test]
    fn failed_record_can_be_immutable_without_downstream_eligibility() {
        let mut record = valid_record();
        record.exact_eighty_one_metrics_bitwise_recomputed = false;
        record.mismatch_reasons =
            vec!["eighty_one_validation_metrics_bitwise_mismatch".to_string()];
        record.verdict = HistoricalOutcomeValidationEvaluationOutputValidationVerdict::FailedIndependentValidationEvaluationOutputValidation;
        record.validation_evaluation_output_independently_validated = false;
        record.future_per_target_candidate_admission_review_eligible = false;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        assert!(validate_record(&record).is_ok());
    }

    #[test]
    fn count_or_authority_tamper_is_rejected() {
        let mut record = valid_record();
        record.recomputed_candidate_hypothesis_count -= 1;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        assert!(validate_record(&record).is_err());

        let mut record = valid_record();
        record.reward_authorized = true;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn independent_mae_and_directional_accuracy_are_exact() {
        assert_eq!(independent_mae(&[1.0, -1.0], &[0.5, -0.5]).unwrap(), 0.5);
        assert_eq!(
            independent_directional_accuracy(&[1.0, -1.0], &[0.5, -0.5]),
            1.0
        );
    }
}
