//! Independent, create-once validation of Stage 57 train-only artifacts.
//!
//! Stage 58 reopens one immutable training claim/result and the exact independently validated
//! training-store copy and frozen experiment suite. A different administrator then recomputes
//! preprocessing, all nine model artifacts and all eighty-one train-only diagnostics without
//! calling the Stage 57 fitting or metric helpers. Every floating-point value is compared through
//! its serialized bit pattern. Validation and sealed-holdout targets stay withheld, and a pass
//! opens only a future validation-evaluation implementation-registration gate.

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
    CompletedHistoricalOutcomeTrainingExecutionAttempt,
    HistoricalOutcomeTrainingBoostedStump, HistoricalOutcomeTrainingExecutionAttemptClaim,
    HistoricalOutcomeTrainingExecutionAttemptStatus, HistoricalOutcomeTrainingFitDiagnostic,
    HistoricalOutcomeTrainingFeaturePreprocessingRecord, HistoricalOutcomeTrainingRidgeCoefficient,
    HistoricalOutcomeTrainingTargetModel, HistoricalOutcomeTrainingTargetModelBody,
    HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    HistoricalOutcomeTrainingUntrustedModelArtifact,
    completed_training_execution_attempt_for_independent_validation,
    completed_training_execution_attempts_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::{
    HistoricalOutcomeTrainingExperimentAlgorithm,
    HistoricalOutcomeTrainingExperimentSuiteSpecification,
    registered_training_experiment_suites_for_independent_review,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_first_execution_authorizations::{
    ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    exact_first_execution_authorization_for_training_audit,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copies::HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_output_validations::independently_validated_training_store_copies_for_training_registration_review;
use super::historical_outcome_offline_dataset_transformation_execution_attempts::HistoricalOutcomeOfflineDatasetTransformationSplit;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-training-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str = "hone-historical-outcome-training-output-validation-v1";
const POLICY_VERSION: &str =
    "hone-training-output-independent-full-bitwise-recomputation-v1-no-validation-label-access";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-training-output-validator-v1-independent-nine-artifact-eighty-one-diagnostic-recompute";
const CLAIM_SCHEMA_VERSION: &str = "hone-historical-outcome-training-execution-claim-v1";
const RESULT_SCHEMA_VERSION: &str = "hone-historical-outcome-training-execution-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str = "hone-untrusted-training-artifact-envelope-v1";
const EXECUTION_POLICY_VERSION: &str = "hone-training-claim-first-one-shot-train-only-fit-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const EXPECTED_ARTIFACT_COUNT: usize = 9;
const EXPECTED_DIAGNOSTIC_COUNT: usize = 81;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const RIDGE_LAMBDA: f64 = 0.01;
const NUMERIC_EPSILON: f64 = 1e-12;
const MAX_MISMATCH_REASONS: usize = 96;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeTrainingOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_implementation_sha256: String,
    expected_implementation_review_sha256: String,
    expected_suite_specification_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_rows_sha256: String,
    expected_excluded_rows_sha256: String,
    expected_target_commitments_sha256: String,
    independent_reopen_and_second_implementation_recomputation_confirmed: bool,
    exact_current_stage_51_through_stage_57_binding_confirmed: bool,
    all_nine_model_artifacts_and_eighty_one_diagnostics_bitwise_recomputed_confirmed: bool,
    validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    no_model_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeTrainingOutputValidationVerdict {
    IndependentlyValidatedTrainOnlyArtifacts,
    FailedIndependentTrainingOutputValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingOutputValidationRecord {
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
    pub suite_specification_sha256: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_execution_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_57_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub envelope_fingerprint_independently_verified: bool,
    pub exact_training_store_dataset_and_suite_verified: bool,
    pub exact_65_feature_preprocessing_bitwise_recomputed: bool,
    pub exact_nine_model_artifacts_bitwise_recomputed: bool,
    pub exact_eighty_one_train_only_diagnostics_bitwise_recomputed: bool,
    pub validation_targets_withheld_verified: bool,
    pub sealed_holdout_targets_withheld_verified: bool,
    pub no_model_selection_or_downstream_authority_verified: bool,
    pub recomputed_claim_sha256: String,
    pub recomputed_result_sha256: String,
    pub recomputed_output_sha256: String,
    pub recomputed_envelope_sha256: String,
    pub recomputed_model_artifact_count: usize,
    pub recomputed_fit_diagnostic_count: usize,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeTrainingOutputValidationVerdict,
    pub training_output_independently_validated: bool,
    pub future_validation_evaluation_implementation_registration_eligible: bool,
    pub validation_selection_authorized: bool,
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
pub(crate) struct HistoricalOutcomeTrainingOutputValidationItem {
    pub attempt: CompletedHistoricalOutcomeTrainingExecutionAttempt,
    pub validation: Option<HistoricalOutcomeTrainingOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeTrainingOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_train_only_artifact_envelope_count: usize,
    pub failed_validation_count: usize,
    pub future_validation_evaluation_implementation_registration_eligible_count: usize,
    pub validation_status: String,
    pub independent_output_validation_available: bool,
    pub validation_selection_authorized: bool,
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
pub(crate) struct HistoricalOutcomeTrainingOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_train_only_artifact_envelope_count: usize,
    pub failed_validation_count: usize,
    pub future_validation_evaluation_implementation_registration_eligible_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeTrainingOutput {
    pub attempt: CompletedHistoricalOutcomeTrainingExecutionAttempt,
    pub validation: HistoricalOutcomeTrainingOutputValidationRecord,
}

#[derive(Debug)]
struct IndependentTrainingData {
    feature_order: Vec<String>,
    target_order: Vec<String>,
    preprocessing: Vec<HistoricalOutcomeTrainingFeaturePreprocessingRecord>,
    train_features: Vec<Vec<Option<f64>>>,
    train_targets: Vec<Vec<f64>>,
    validation_count: usize,
    sealed_holdout_count: usize,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    envelope_fingerprint: bool,
    exact_dataset_and_suite: bool,
    preprocessing_exact: bool,
    artifacts_exact: bool,
    diagnostics_exact: bool,
    validation_withheld: bool,
    sealed_holdout_withheld: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_output_sha256: String,
    recomputed_envelope_sha256: String,
    recomputed_model_artifact_count: usize,
    recomputed_fit_diagnostic_count: usize,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_historical_outcome_training_output_validations(
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
            warn!(%error, "historical outcome training output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果训练产物独立验证暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_training_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeTrainingOutputRequest>,
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

pub(crate) async fn historical_outcome_training_output_validation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeTrainingOutputValidationReadinessSummary {
        validation_eligible_count: registry.validation_eligible_count,
        validation_count: registry.validation_count,
        independently_validated_train_only_artifact_envelope_count: registry
            .independently_validated_train_only_artifact_envelope_count,
        failed_validation_count: registry.failed_validation_count,
        future_validation_evaluation_implementation_registration_eligible_count: registry
            .future_validation_evaluation_implementation_registration_eligible_count,
        validation_status: registry.validation_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_training_outputs_for_validation_evaluation_implementation_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeTrainingOutput>, String> {
    let validations = read_validations(state).await?;
    let index = validations
        .into_iter()
        .map(|record| (record.attempt_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for attempt in completed_training_execution_attempts_for_independent_validation(state).await? {
        let Some(validation) = index.get(&attempt.claim.attempt_id).cloned() else {
            continue;
        };
        validate_record(&validation)?;
        if validation.training_output_independently_validated
            && validation.future_validation_evaluation_implementation_registration_eligible
            && validation.claim_sha256 == attempt.claim.claim_sha256
            && validation.result_sha256 == attempt.result.result_sha256
        {
            output.push(IndependentlyValidatedHistoricalOutcomeTrainingOutput {
                attempt,
                validation,
            });
        }
    }
    Ok(output)
}

#[allow(dead_code)]
pub(crate) fn validate_historical_outcome_training_output_validation_for_evaluation_implementation(
    record: &HistoricalOutcomeTrainingOutputValidationRecord,
) -> Result<(), String> {
    validate_record(record)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingOutputValidationRegistry, String> {
    let attempts = completed_training_execution_attempts_for_independent_validation(state).await?;
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
            HistoricalOutcomeTrainingOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let independently_validated_train_only_artifact_envelope_count = validations
        .iter()
        .filter(|record| record.training_output_independently_validated)
        .count();
    let failed_validation_count = validations
        .len()
        .saturating_sub(independently_validated_train_only_artifact_envelope_count);
    let validation_status = if failed_validation_count > 0 {
        "training_output_independent_validation_failed_fail_closed"
    } else if independently_validated_train_only_artifact_envelope_count > 0 {
        "training_output_independently_validated_waiting_validation_evaluation_implementation_registration"
    } else if validation_eligible_count > 0 {
        "waiting_independent_training_output_recomputation_validation"
    } else {
        "waiting_completed_training_execution_attempt"
    };
    Ok(HistoricalOutcomeTrainingOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count: validations.len(),
        independently_validated_train_only_artifact_envelope_count,
        failed_validation_count,
        future_validation_evaluation_implementation_registration_eligible_count:
            independently_validated_train_only_artifact_envelope_count,
        validation_status: validation_status.to_string(),
        independent_output_validation_available: true,
        validation_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 58 阶段由执行者和完整上游之外的新管理员重开 Stage 57 claim/result、精确训练副本和固定套件，使用第二套实现独立复算 65 项预处理、9 个模型工件和 81 项 train-only 诊断并逐位比对。通过只开放未来验证评估实现登记资格；validation/holdout 标签、选模、工件库、指标库、奖励、影子、订单、券商和交易继续关闭。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeTrainingOutputRequest,
) -> Result<HistoricalOutcomeTrainingOutputValidationRecord, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.attempt_id == attempt_id)
    {
        return Err("这次训练执行已经形成不可变独立验证记录，不得重放".to_string());
    }
    let attempt =
        completed_training_execution_attempt_for_independent_validation(state, attempt_id)
            .await?
            .ok_or_else(|| "当前没有完整且未验证的训练执行产物".to_string())?;
    verify_expected_bindings(&attempt, &request)?;
    let authorization = exact_first_execution_authorization_for_training_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "训练执行授权完整链已经无法重开".to_string())?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("训练产物验证人不能是执行人或完整上游链任一角色".to_string());
    }
    let dataset = exact_training_store_dataset(state, &attempt.claim).await?;
    let suite = exact_training_suite(state, &authorization).await?;
    let audit = independently_audit(&attempt, &authorization, &dataset, &suite)?;
    let passed = independent && audit.mismatch_reasons.is_empty();
    let output_sha256 = attempt
        .result
        .output_sha256
        .clone()
        .ok_or_else(|| "训练执行结果缺少输出 SHA-256".to_string())?;
    let mut record = HistoricalOutcomeTrainingOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        attempt_id: attempt_id.to_string(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        output_sha256,
        authorization_review_id: attempt.claim.authorization_review_id.clone(),
        authorization_review_sha256: attempt.claim.authorization_review_sha256.clone(),
        isolated_runner_id: attempt.claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: attempt.claim.isolated_runner_spec_sha256.clone(),
        implementation_id: attempt.claim.implementation_id.clone(),
        implementation_sha256: attempt.claim.implementation_sha256.clone(),
        implementation_review_sha256: attempt.claim.implementation_review_sha256.clone(),
        suite_specification_sha256: attempt.claim.suite_specification_sha256.clone(),
        training_store_dataset_sha256: attempt.claim.training_store_dataset_sha256.clone(),
        rows_sha256: attempt.claim.rows_sha256.clone(),
        excluded_rows_sha256: attempt.claim.excluded_rows_sha256.clone(),
        target_commitments_sha256: attempt.claim.target_commitments_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        invoked_by: attempt.claim.invoked_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_execution_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_57_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        envelope_fingerprint_independently_verified: audit.envelope_fingerprint,
        exact_training_store_dataset_and_suite_verified: audit.exact_dataset_and_suite,
        exact_65_feature_preprocessing_bitwise_recomputed: audit.preprocessing_exact,
        exact_nine_model_artifacts_bitwise_recomputed: audit.artifacts_exact,
        exact_eighty_one_train_only_diagnostics_bitwise_recomputed: audit.diagnostics_exact,
        validation_targets_withheld_verified: audit.validation_withheld,
        sealed_holdout_targets_withheld_verified: audit.sealed_holdout_withheld,
        no_model_selection_or_downstream_authority_verified: audit.downstream_closed,
        recomputed_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_result_sha256: audit.recomputed_result_sha256,
        recomputed_output_sha256: audit.recomputed_output_sha256,
        recomputed_envelope_sha256: audit.recomputed_envelope_sha256,
        recomputed_model_artifact_count: audit.recomputed_model_artifact_count,
        recomputed_fit_diagnostic_count: audit.recomputed_fit_diagnostic_count,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            HistoricalOutcomeTrainingOutputValidationVerdict::IndependentlyValidatedTrainOnlyArtifacts
        } else {
            HistoricalOutcomeTrainingOutputValidationVerdict::FailedIndependentTrainingOutputValidation
        },
        training_output_independently_validated: passed,
        future_validation_evaluation_implementation_registration_eligible: passed,
        validation_selection_authorized: false,
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

fn independently_audit(
    attempt: &CompletedHistoricalOutcomeTrainingExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    suite: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
) -> Result<IndependentAudit, String> {
    let claim = &attempt.claim;
    let result = &attempt.result;
    let envelope = result
        .untrusted_artifact_envelope
        .as_ref()
        .ok_or_else(|| "训练执行结果缺少未验证工件信封".to_string())?;
    let review = &authorization
        .runner
        .implementation
        .approved_registration_review;
    let recomputed_claim_sha256 = fingerprint_without(claim, &["attempt_id", "claim_sha256"])?;
    let recomputed_result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    let recomputed_output_sha256 = hash_serializable(envelope)?;
    let recomputed_envelope_sha256 = recomputed_output_sha256.clone();
    let exact_chain = claim.authorization_review_id == authorization.review.review_id
        && claim.authorization_review_sha256 == authorization.review.review_sha256
        && claim.isolated_runner_id == authorization.runner.isolated_runner_id
        && claim.isolated_runner_spec_sha256 == authorization.runner.isolated_runner_spec_sha256
        && claim.runner_artifact_sha256 == authorization.runner.runner_artifact_sha256
        && claim.runner_code_revision == authorization.runner.runner_code_revision
        && claim.implementation_id == authorization.runner.implementation.implementation_id
        && claim.implementation_sha256 == authorization.runner.implementation.implementation_sha256
        && claim.implementation_review_sha256
            == authorization.runner.implementation_review.review_sha256
        && claim.suite_specification_sha256 == review.suite_specification_sha256
        && claim.training_store_dataset_sha256 == review.training_store_dataset_sha256
        && claim.rows_sha256 == review.rows_sha256
        && claim.excluded_rows_sha256 == review.excluded_rows_sha256
        && claim.target_commitments_sha256 == review.target_commitments_sha256;
    let claim_fingerprint = claim.schema_version == CLAIM_SCHEMA_VERSION
        && claim.execution_policy_version == EXECUTION_POLICY_VERSION
        && claim.claim_sha256 == recomputed_claim_sha256
        && claim.attempt_id == recomputed_claim_sha256[..32]
        && claim.authorization_consumed
        && claim.invocation_started
        && claim.exact_training_store_read_allowed
        && claim.train_target_read_allowed
        && !claim.validation_target_read_allowed
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
    let result_fingerprint = result.schema_version == RESULT_SCHEMA_VERSION
        && result.execution_policy_version == EXECUTION_POLICY_VERSION
        && result.result_sha256 == recomputed_result_sha256
        && result.result_id == recomputed_result_sha256[..32]
        && result.attempt_id == claim.attempt_id
        && result.claim_sha256 == claim.claim_sha256
        && result.status
            == HistoricalOutcomeTrainingExecutionAttemptStatus::CompletedWithUntrustedTrainOnlyArtifacts
        && result.exit_code == 0
        && result.output_sha256.as_deref() == Some(recomputed_output_sha256.as_str())
        && result.output_bytes == serde_json::to_vec(envelope).map_err(|error| error.to_string())?.len()
        && result.bounded_error.is_none()
        && result.ephemeral_directory_removed
        && !result.independent_output_validation_completed
        && !result.validation_selection_authorized
        && !result.sealed_holdout_access_authorized
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
        && envelope.implementation_review_sha256 == claim.implementation_review_sha256
        && envelope.suite_specification_sha256 == claim.suite_specification_sha256
        && envelope.training_store_dataset_sha256 == claim.training_store_dataset_sha256
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
    let recomputed_target_commitments_sha256 = hash_serializable(&commitments)?;
    let recomputed_training_store_dataset_sha256 =
        fingerprint_without(dataset, &["training_store_dataset_sha256"])?;
    let recomputed_suite_specification_sha256 =
        fingerprint_without(suite, &["specification_sha256"])?;
    let exact_dataset_and_suite = dataset.training_store_dataset_sha256
        == recomputed_training_store_dataset_sha256
        && dataset.training_store_dataset_sha256 == claim.training_store_dataset_sha256
        && recomputed_rows_sha256 == claim.rows_sha256
        && recomputed_excluded_rows_sha256 == claim.excluded_rows_sha256
        && recomputed_target_commitments_sha256 == claim.target_commitments_sha256
        && suite.specification_sha256 == recomputed_suite_specification_sha256
        && suite.specification_sha256 == claim.suite_specification_sha256
        && suite.feature_catalog_count == EXPECTED_FEATURE_COUNT
        && suite.target_count == EXPECTED_TARGET_COUNT
        && suite.target_vector_order.len() == EXPECTED_TARGET_COUNT
        && suite.arms.len() == 3
        && suite
            .arms
            .iter()
            .all(|arm| arm.random_seeds == EXPECTED_SEEDS)
        && suite.fit_split == "train"
        && suite.model_selection_split == "validation"
        && suite.sealed_holdout_split == "sealed_holdout"
        && !suite.sealed_holdout_access_allowed
        && !suite.sealed_holdout_labels_visible_to_training_worker
        && !suite.scalar_reward_defined
        && !suite.action_position_or_ranking_semantics_defined;
    let recomputed = independently_recompute_envelope(authorization, dataset, suite)?;
    let preprocessing_exact = envelope.feature_order == recomputed.feature_order
        && envelope.target_order == recomputed.target_order
        && envelope.preprocessing == recomputed.preprocessing
        && envelope.train_row_count == recomputed.train_row_count
        && envelope.validation_row_count_with_targets_withheld
            == recomputed.validation_row_count_with_targets_withheld
        && envelope.sealed_holdout_row_count_with_targets_withheld
            == recomputed.sealed_holdout_row_count_with_targets_withheld;
    let artifacts_exact = envelope.model_artifacts == recomputed.model_artifacts
        && recomputed.model_artifacts.len() == EXPECTED_ARTIFACT_COUNT;
    let diagnostics_exact = envelope.fit_diagnostics == recomputed.fit_diagnostics
        && recomputed.fit_diagnostics.len() == EXPECTED_DIAGNOSTIC_COUNT;
    let validation_withheld = dataset.rows.iter().all(|row| {
        row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::Validation
            || (row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
                && row.target_vector.is_none())
    }) && !envelope.validation_labels_accessed
        && !recomputed.validation_labels_accessed;
    let sealed_holdout_withheld = dataset.rows.iter().all(|row| {
        row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
            || (row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
                && row.target_vector.is_none())
    }) && !envelope.sealed_holdout_labels_accessed
        && !recomputed.sealed_holdout_labels_accessed;
    let downstream_closed = envelope.train_only_fit_completed
        && !envelope.validation_selection_completed
        && envelope.output_is_untrusted
        && !envelope.independent_output_validation_completed
        && !envelope.model_artifact_store_written
        && !envelope.metric_store_written
        && !envelope.scalar_reward_written
        && !envelope.shadow_position_written
        && !envelope.order_generated
        && !envelope.broker_accessed
        && !envelope.trade_executed;
    let mut mismatch_reasons = Vec::new();
    for (matched, reason) in [
        (exact_chain, "stage_51_through_stage_57_chain_mismatch"),
        (
            claim_fingerprint,
            "training_claim_fingerprint_or_capability_mismatch",
        ),
        (
            result_fingerprint,
            "training_result_fingerprint_or_shape_mismatch",
        ),
        (envelope_fingerprint, "training_envelope_binding_mismatch"),
        (
            exact_dataset_and_suite,
            "training_dataset_or_frozen_suite_mismatch",
        ),
        (
            preprocessing_exact,
            "sixty_five_feature_preprocessing_bitwise_mismatch",
        ),
        (artifacts_exact, "nine_model_artifacts_bitwise_mismatch"),
        (
            diagnostics_exact,
            "eighty_one_train_diagnostics_bitwise_mismatch",
        ),
        (validation_withheld, "validation_targets_not_withheld"),
        (
            sealed_holdout_withheld,
            "sealed_holdout_targets_not_withheld",
        ),
        (
            downstream_closed,
            "model_selection_or_downstream_authority_overreach",
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
        exact_dataset_and_suite,
        preprocessing_exact,
        artifacts_exact,
        diagnostics_exact,
        validation_withheld,
        sealed_holdout_withheld,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_output_sha256,
        recomputed_envelope_sha256,
        recomputed_model_artifact_count: recomputed.model_artifacts.len(),
        recomputed_fit_diagnostic_count: recomputed.fit_diagnostics.len(),
        mismatch_reasons,
    })
}

fn independently_recompute_envelope(
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    suite: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
) -> Result<HistoricalOutcomeTrainingUntrustedArtifactEnvelope, String> {
    let data = independently_prepare_training_data(dataset, suite)?;
    let mut model_artifacts = Vec::new();
    let mut fit_diagnostics = Vec::new();
    for arm in &suite.arms {
        for seed in &arm.random_seeds {
            let (target_models, predictions) = match arm.algorithm {
                HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline => {
                    independent_zero_models(&data)
                }
                HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression => {
                    independent_ridge_models(&data)?
                }
                HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression => {
                    independent_boosted_models(
                        &data,
                        *seed,
                        arm.max_epochs_or_boosting_rounds,
                        arm.learning_rate_micros,
                        arm.maximum_tree_depth,
                    )?
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
            fit_diagnostics.extend(independent_fit_diagnostics(
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
    Ok(HistoricalOutcomeTrainingUntrustedArtifactEnvelope {
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
    })
}

fn independently_prepare_training_data(
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    suite: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
) -> Result<IndependentTrainingData, String> {
    let first = dataset
        .rows
        .first()
        .ok_or_else(|| "训练副本没有可供独立复算的行".to_string())?;
    let mut feature_order = first
        .feature_records
        .iter()
        .map(|feature| feature.feature_id.clone())
        .collect::<Vec<_>>();
    feature_order.sort();
    feature_order.dedup();
    if feature_order.len() != EXPECTED_FEATURE_COUNT {
        return Err("独立复算要求精确 65 项特征".to_string());
    }
    let mut train_features = Vec::new();
    let mut train_targets = Vec::new();
    let mut validation_count = 0;
    let mut sealed_holdout_count = 0;
    for row in &dataset.rows {
        match row.target_visibility {
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets => {
                if row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::Train {
                    return Err("带目标的行不属于 train split".to_string());
                }
                train_features.push(independent_parse_feature_row(row, &feature_order)?);
                train_targets.push(independent_parse_target_row(
                    row,
                    &suite.target_vector_order,
                )?);
            }
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld => {
                if row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::Validation
                    || row.target_vector.is_some()
                {
                    return Err("独立复算发现 validation 标签泄露".to_string());
                }
                validation_count += 1;
            }
            HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld => {
                if row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
                    || row.target_vector.is_some()
                {
                    return Err("独立复算发现 sealed holdout 标签泄露".to_string());
                }
                sealed_holdout_count += 1;
            }
        }
    }
    if train_features.is_empty() || train_features.len() != train_targets.len() {
        return Err("train split 特征与目标不完整".to_string());
    }
    let preprocessing = independent_fit_preprocessing(&feature_order, &mut train_features)?;
    Ok(IndependentTrainingData {
        feature_order,
        target_order: suite.target_vector_order.clone(),
        preprocessing,
        train_features,
        train_targets,
        validation_count,
        sealed_holdout_count,
    })
}

fn independent_parse_feature_row(
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
                    return Err(format!("缺失特征 {feature_id} 携带数值"));
                }
                Ok(None)
            } else {
                let value = feature
                    .value
                    .as_deref()
                    .ok_or_else(|| format!("非缺失特征 {feature_id} 没有数值"))?
                    .parse::<f64>()
                    .map_err(|_| format!("特征 {feature_id} 不是有限数值"))?;
                if !value.is_finite() {
                    return Err(format!("特征 {feature_id} 不是有限数值"));
                }
                Ok(Some(value))
            }
        })
        .collect()
}

fn independent_parse_target_row(
    row: &HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    target_order: &[String],
) -> Result<Vec<f64>, String> {
    let values = row
        .target_vector
        .as_ref()
        .ok_or_else(|| "train 行没有目标向量".to_string())?;
    let index = values
        .iter()
        .map(|value| (value.target_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    if index.len() != EXPECTED_TARGET_COUNT {
        return Err("train 行目标数量或唯一性错误".to_string());
    }
    target_order
        .iter()
        .map(|target_id| {
            let raw = index
                .get(target_id.as_str())
                .ok_or_else(|| format!("train 行缺少目标 {target_id}"))?;
            let bits = u64::from_str_radix(&raw.exact_f64_bits_hex, 16)
                .map_err(|_| format!("目标 {target_id} 位模式无效"))?;
            let value = f64::from_bits(bits);
            if !value.is_finite() {
                return Err(format!("目标 {target_id} 不是有限值"));
            }
            Ok(value)
        })
        .collect()
}

fn independent_fit_preprocessing(
    feature_order: &[String],
    rows: &mut [Vec<Option<f64>>],
) -> Result<Vec<HistoricalOutcomeTrainingFeaturePreprocessingRecord>, String> {
    let mut output = Vec::with_capacity(feature_order.len());
    for (column, feature_id) in feature_order.iter().enumerate() {
        let observed = rows
            .iter()
            .filter_map(|row| row[column])
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
            if let Some(value) = row[column] {
                row[column] = Some((value - mean) / scale);
            }
        }
        output.push(HistoricalOutcomeTrainingFeaturePreprocessingRecord {
            feature_id: feature_id.clone(),
            observed_train_count: observed.len(),
            missing_train_count: rows.len().saturating_sub(observed.len()),
            train_mean_f64_bits_hex: f64_hex(mean),
            train_scale_f64_bits_hex: f64_hex(scale),
            missingness_preserved: true,
        });
    }
    Ok(output)
}

fn independent_zero_models(
    data: &IndependentTrainingData,
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

fn independent_design_row(features: &[Option<f64>]) -> Vec<f64> {
    let mut output = Vec::with_capacity(1 + features.len() * 2);
    output.push(1.0);
    for value in features {
        output.push(value.unwrap_or(0.0));
        output.push(f64::from(value.is_none()));
    }
    output
}

fn independent_ridge_models(
    data: &IndependentTrainingData,
) -> Result<(Vec<HistoricalOutcomeTrainingTargetModel>, Vec<Vec<f64>>), String> {
    let design = data
        .train_features
        .iter()
        .map(|row| independent_design_row(row))
        .collect::<Vec<_>>();
    let dimension = design[0].len();
    let mut gram = vec![vec![0.0; dimension]; dimension];
    let mut projected_targets = vec![vec![0.0; dimension]; data.target_order.len()];
    for (row, targets) in design.iter().zip(&data.train_targets) {
        for left in 0..dimension {
            for right in 0..dimension {
                gram[left][right] += row[left] * row[right];
            }
            for target in 0..data.target_order.len() {
                projected_targets[target][left] += row[left] * targets[target];
            }
        }
    }
    for diagonal in 1..dimension {
        gram[diagonal][diagonal] += RIDGE_LAMBDA;
    }
    let weights = projected_targets
        .iter()
        .map(|rhs| independent_gauss_jordan(&gram, rhs))
        .collect::<Result<Vec<_>, _>>()?;
    let predictions = design
        .iter()
        .map(|row| {
            weights
                .iter()
                .map(|target_weights| independent_dot(row, target_weights))
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

fn independent_gauss_jordan(matrix: &[Vec<f64>], rhs: &[f64]) -> Result<Vec<f64>, String> {
    let size = rhs.len();
    let mut augmented = matrix
        .iter()
        .zip(rhs)
        .map(|(row, value)| {
            let mut values = row.clone();
            values.push(*value);
            values
        })
        .collect::<Vec<_>>();
    for pivot in 0..size {
        let best = (pivot..size)
            .max_by(|left, right| {
                augmented[*left][pivot]
                    .abs()
                    .total_cmp(&augmented[*right][pivot].abs())
            })
            .ok_or_else(|| "独立 ridge 复算缺少 pivot".to_string())?;
        augmented.swap(pivot, best);
        let divisor = augmented[pivot][pivot];
        if divisor.abs() <= NUMERIC_EPSILON {
            return Err("独立 ridge 复算矩阵奇异".to_string());
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
        return Err("独立 ridge 复算得到非有限值".to_string());
    }
    Ok(output)
}

#[derive(Debug, Clone)]
struct IndependentStump {
    feature_index: Option<usize>,
    threshold: Option<f64>,
    low_leaf: f64,
    high_leaf: f64,
    missing_leaf: f64,
    loss: f64,
}

fn independent_boosted_models(
    data: &IndependentTrainingData,
    seed: u64,
    rounds: u16,
    learning_rate_micros: u32,
    maximum_tree_depth: u8,
) -> Result<(Vec<HistoricalOutcomeTrainingTargetModel>, Vec<Vec<f64>>), String> {
    let learning_rate = f64::from(learning_rate_micros) / 1_000_000.0;
    let mut predictions = vec![vec![0.0; data.target_order.len()]; data.train_targets.len()];
    let thresholds = (0..data.feature_order.len())
        .map(|feature| independent_lower_median(&data.train_features, feature))
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
            let stump = independent_best_stump(
                &data.feature_order,
                &data.train_features,
                &thresholds,
                &residuals,
                seed,
                round,
            );
            for (row_index, features) in data.train_features.iter().enumerate() {
                predictions[row_index][target] +=
                    learning_rate * independent_stump_prediction(&stump, features);
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

fn independent_best_stump(
    feature_order: &[String],
    features: &[Vec<Option<f64>>],
    thresholds: &[Option<f64>],
    residuals: &[f64],
    seed: u64,
    round: u16,
) -> IndependentStump {
    let mean = residuals.iter().sum::<f64>() / residuals.len() as f64;
    let mut best = IndependentStump {
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
        let low_mean = independent_mean(&low);
        let high_mean = independent_mean(&high);
        let missing_mean = if missing.is_empty() {
            0.0
        } else {
            independent_mean(&missing)
        };
        let loss = independent_squared_error(&low, low_mean)
            + independent_squared_error(&high, high_mean)
            + independent_squared_error(&missing, missing_mean);
        if loss + NUMERIC_EPSILON < best.loss {
            best = IndependentStump {
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

fn independent_stump_prediction(stump: &IndependentStump, features: &[Option<f64>]) -> f64 {
    match (stump.feature_index, stump.threshold) {
        (Some(index), Some(threshold)) => match features[index] {
            Some(value) if value <= threshold => stump.low_leaf,
            Some(_) => stump.high_leaf,
            None => stump.missing_leaf,
        },
        _ => stump.missing_leaf,
    }
}

fn independent_lower_median(rows: &[Vec<Option<f64>>], feature: usize) -> Option<f64> {
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

fn independent_mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn independent_squared_error(values: &[f64], mean: f64) -> f64 {
    values.iter().map(|value| (value - mean).powi(2)).sum()
}

fn independent_dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn independent_fit_diagnostics(
    algorithm: &HistoricalOutcomeTrainingExperimentAlgorithm,
    seed: u64,
    target_order: &[String],
    actual: &[Vec<f64>],
    predicted: &[Vec<f64>],
) -> Result<Vec<HistoricalOutcomeTrainingFitDiagnostic>, String> {
    if actual.len() != predicted.len() || actual.is_empty() {
        return Err("独立诊断复算输入行数不一致".to_string());
    }
    (0..target_order.len())
        .map(|target| {
            let y = actual.iter().map(|row| row[target]).collect::<Vec<_>>();
            let p = predicted.iter().map(|row| row[target]).collect::<Vec<_>>();
            let mae = y
                .iter()
                .zip(&p)
                .map(|(actual, predicted)| (actual - predicted).abs())
                .sum::<f64>()
                / y.len() as f64;
            let directional = y
                .iter()
                .zip(&p)
                .filter(|(actual, predicted)| actual.signum() == predicted.signum())
                .count() as f64
                / y.len() as f64;
            Ok(HistoricalOutcomeTrainingFitDiagnostic {
                algorithm: algorithm.clone(),
                random_seed: seed,
                target_id: target_order[target].clone(),
                split: "train_fit_diagnostic_only".to_string(),
                mae_f64_bits_hex: f64_hex(mae),
                spearman_f64_bits_hex: independent_spearman(&y, &p).map(f64_hex),
                directional_accuracy_f64_bits_hex: f64_hex(directional),
                calibration_slope_f64_bits_hex: independent_calibration_slope(&y, &p).map(f64_hex),
                model_selection_metric: false,
            })
        })
        .collect()
}

fn independent_spearman(left: &[f64], right: &[f64]) -> Option<f64> {
    let left_ranks = independent_average_ranks(left);
    let right_ranks = independent_average_ranks(right);
    independent_pearson(&left_ranks, &right_ranks)
}

fn independent_average_ranks(values: &[f64]) -> Vec<f64> {
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

fn independent_pearson(left: &[f64], right: &[f64]) -> Option<f64> {
    let left_mean = independent_mean(left);
    let right_mean = independent_mean(right);
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

fn independent_calibration_slope(actual: &[f64], predicted: &[f64]) -> Option<f64> {
    let predicted_mean = independent_mean(predicted);
    let actual_mean = independent_mean(actual);
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

async fn exact_training_store_dataset(
    state: &AppState,
    claim: &HistoricalOutcomeTrainingExecutionAttemptClaim,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset, String> {
    independently_validated_training_store_copies_for_training_registration_review(state)
        .await?
        .into_iter()
        .filter_map(|value| value.copied_dataset.attempt.training_store_dataset)
        .find(|dataset| {
            dataset.training_store_dataset_sha256 == claim.training_store_dataset_sha256
        })
        .ok_or_else(|| "精确绑定的独立验证训练副本已经不可用".to_string())
}

async fn exact_training_suite(
    state: &AppState,
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
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
        .ok_or_else(|| "精确绑定的冻结训练套件已经不可用".to_string())
}

fn validate_request(
    attempt_id: &str,
    request: &ValidateHistoricalOutcomeTrainingOutputRequest,
) -> Result<(), String> {
    if !valid_id(attempt_id)
        || [
            request.expected_claim_sha256.as_str(),
            request.expected_result_sha256.as_str(),
            request.expected_output_sha256.as_str(),
            request.expected_authorization_review_sha256.as_str(),
            request.expected_isolated_runner_spec_sha256.as_str(),
            request.expected_implementation_sha256.as_str(),
            request.expected_implementation_review_sha256.as_str(),
            request.expected_suite_specification_sha256.as_str(),
            request.expected_training_store_dataset_sha256.as_str(),
            request.expected_rows_sha256.as_str(),
            request.expected_excluded_rows_sha256.as_str(),
            request.expected_target_commitments_sha256.as_str(),
        ]
        .iter()
        .any(|value| !valid_sha256(value))
    {
        return Err("训练产物独立验证请求中的 ID 或 SHA-256 无效".to_string());
    }
    if !request.independent_reopen_and_second_implementation_recomputation_confirmed
        || !request.exact_current_stage_51_through_stage_57_binding_confirmed
        || !request.all_nine_model_artifacts_and_eighty_one_diagnostics_bitwise_recomputed_confirmed
        || !request.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        || !request.no_model_selection_store_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err(
            "必须确认独立第二实现复算、完整链、9 工件/81 诊断逐位核对、留出标签隐藏和零下游权限"
                .to_string(),
        );
    }
    Ok(())
}

fn verify_expected_bindings(
    attempt: &CompletedHistoricalOutcomeTrainingExecutionAttempt,
    request: &ValidateHistoricalOutcomeTrainingOutputRequest,
) -> Result<(), String> {
    let claim = &attempt.claim;
    let result = &attempt.result;
    if claim.claim_sha256 != request.expected_claim_sha256
        || result.result_sha256 != request.expected_result_sha256
        || result.output_sha256.as_deref() != Some(request.expected_output_sha256.as_str())
        || claim.authorization_review_sha256 != request.expected_authorization_review_sha256
        || claim.isolated_runner_spec_sha256 != request.expected_isolated_runner_spec_sha256
        || claim.implementation_sha256 != request.expected_implementation_sha256
        || claim.implementation_review_sha256 != request.expected_implementation_review_sha256
        || claim.suite_specification_sha256 != request.expected_suite_specification_sha256
        || claim.training_store_dataset_sha256 != request.expected_training_store_dataset_sha256
        || claim.rows_sha256 != request.expected_rows_sha256
        || claim.excluded_rows_sha256 != request.expected_excluded_rows_sha256
        || claim.target_commitments_sha256 != request.expected_target_commitments_sha256
    {
        return Err("训练 claim/result、输出或完整上游绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn prior_actor_ids(
    attempt: &CompletedHistoricalOutcomeTrainingExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeTrainingFirstExecutionAuthorization,
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

fn validate_record(record: &HistoricalOutcomeTrainingOutputValidationRecord) -> Result<(), String> {
    let passed = record.validator_independent_from_execution_and_complete_prior_chain
        && record.exact_current_stage_51_through_stage_57_chain_verified
        && record.claim_fingerprint_independently_verified
        && record.result_fingerprint_independently_verified
        && record.envelope_fingerprint_independently_verified
        && record.exact_training_store_dataset_and_suite_verified
        && record.exact_65_feature_preprocessing_bitwise_recomputed
        && record.exact_nine_model_artifacts_bitwise_recomputed
        && record.exact_eighty_one_train_only_diagnostics_bitwise_recomputed
        && record.validation_targets_withheld_verified
        && record.sealed_holdout_targets_withheld_verified
        && record.no_model_selection_or_downstream_authority_verified
        && record.recomputed_model_artifact_count == EXPECTED_ARTIFACT_COUNT
        && record.recomputed_fit_diagnostic_count == EXPECTED_DIAGNOSTIC_COUNT
        && record.mismatch_reasons.is_empty();
    let closed = !record.validation_selection_authorized
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
        record.suite_specification_sha256.as_str(),
        record.training_store_dataset_sha256.as_str(),
        record.rows_sha256.as_str(),
        record.excluded_rows_sha256.as_str(),
        record.target_commitments_sha256.as_str(),
        record.validator_implementation_sha256.as_str(),
        record.recomputed_claim_sha256.as_str(),
        record.recomputed_result_sha256.as_str(),
        record.recomputed_output_sha256.as_str(),
        record.recomputed_envelope_sha256.as_str(),
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
        || record.training_output_independently_validated != passed
        || record.future_validation_evaluation_implementation_registration_eligible != passed
        || record.training_output_independently_validated
            != (record.verdict
                == HistoricalOutcomeTrainingOutputValidationVerdict::IndependentlyValidatedTrainOnlyArtifacts)
        || !closed
    {
        return Err("training output validation record is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeTrainingOutputValidationRecord>, String> {
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
            return Err("训练产物验证目录包含无效 attempt ID".to_string());
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
            let record = serde_json::from_slice::<HistoricalOutcomeTrainingOutputValidationRecord>(
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
                return Err("训练产物独立验证登记含重放、误命名或重复哈希".to_string());
            }
            records.push(record);
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeTrainingOutputValidationRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    hash_bytes(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
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

fn f64_hex(value: f64) -> String {
    format!("{:016x}", value.to_bits())
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
    decision_root(state).join("training-output-validations")
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
    let path = directory.join(format!("training-output-validation-{attempt_id}.lock"));
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
        .map_err(|_| "已有同一训练产物独立验证正在进行".to_string())?;
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
pub(crate) mod tests {
    use super::*;

    fn request() -> ValidateHistoricalOutcomeTrainingOutputRequest {
        ValidateHistoricalOutcomeTrainingOutputRequest {
            expected_claim_sha256: "a".repeat(64),
            expected_result_sha256: "b".repeat(64),
            expected_output_sha256: "c".repeat(64),
            expected_authorization_review_sha256: "d".repeat(64),
            expected_isolated_runner_spec_sha256: "e".repeat(64),
            expected_implementation_sha256: "f".repeat(64),
            expected_implementation_review_sha256: "1".repeat(64),
            expected_suite_specification_sha256: "2".repeat(64),
            expected_training_store_dataset_sha256: "3".repeat(64),
            expected_rows_sha256: "4".repeat(64),
            expected_excluded_rows_sha256: "5".repeat(64),
            expected_target_commitments_sha256: "6".repeat(64),
            independent_reopen_and_second_implementation_recomputation_confirmed: true,
            exact_current_stage_51_through_stage_57_binding_confirmed: true,
            all_nine_model_artifacts_and_eighty_one_diagnostics_bitwise_recomputed_confirmed: true,
            validation_and_sealed_holdout_targets_remain_withheld_confirmed: true,
            no_model_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    fn synthetic_data() -> IndependentTrainingData {
        let feature_order = (0..EXPECTED_FEATURE_COUNT)
            .map(|index| format!("feature_{index:02}"))
            .collect::<Vec<_>>();
        let target_order = (0..EXPECTED_TARGET_COUNT)
            .map(|index| format!("target_{index:02}"))
            .collect::<Vec<_>>();
        let train_features = (0..8)
            .map(|row| {
                (0..EXPECTED_FEATURE_COUNT)
                    .map(|column| {
                        if (row + column) % 11 == 0 {
                            None
                        } else {
                            Some((row as f64 - 3.5) * (column as f64 + 1.0) / 10.0)
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let train_targets = (0..8)
            .map(|row| {
                (0..EXPECTED_TARGET_COUNT)
                    .map(|target| (row as f64 - 3.0) * (target as f64 + 1.0) / 100.0)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        IndependentTrainingData {
            feature_order: feature_order.clone(),
            target_order,
            preprocessing: feature_order
                .into_iter()
                .map(
                    |feature_id| HistoricalOutcomeTrainingFeaturePreprocessingRecord {
                        feature_id,
                        observed_train_count: 8,
                        missing_train_count: 0,
                        train_mean_f64_bits_hex: f64_hex(0.0),
                        train_scale_f64_bits_hex: f64_hex(1.0),
                        missingness_preserved: true,
                    },
                )
                .collect(),
            train_features,
            train_targets,
            validation_count: 2,
            sealed_holdout_count: 2,
        }
    }

    pub(crate) fn valid_record() -> HistoricalOutcomeTrainingOutputValidationRecord {
        let mut record = HistoricalOutcomeTrainingOutputValidationRecord {
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
            suite_specification_sha256: "7".repeat(64),
            training_store_dataset_sha256: "8".repeat(64),
            rows_sha256: "9".repeat(64),
            excluded_rows_sha256: "a".repeat(64),
            target_commitments_sha256: "b".repeat(64),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
            validator_implementation_sha256: validator_implementation_sha256(),
            validated_at: Utc::now(),
            validated_by: "validator".to_string(),
            invoked_by: "executor".to_string(),
            excluded_prior_actor_ids: vec!["executor".to_string(), "reviewer".to_string()],
            validator_independent_from_execution_and_complete_prior_chain: true,
            exact_current_stage_51_through_stage_57_chain_verified: true,
            claim_fingerprint_independently_verified: true,
            result_fingerprint_independently_verified: true,
            envelope_fingerprint_independently_verified: true,
            exact_training_store_dataset_and_suite_verified: true,
            exact_65_feature_preprocessing_bitwise_recomputed: true,
            exact_nine_model_artifacts_bitwise_recomputed: true,
            exact_eighty_one_train_only_diagnostics_bitwise_recomputed: true,
            validation_targets_withheld_verified: true,
            sealed_holdout_targets_withheld_verified: true,
            no_model_selection_or_downstream_authority_verified: true,
            recomputed_claim_sha256: "c".repeat(64),
            recomputed_result_sha256: "d".repeat(64),
            recomputed_output_sha256: "e".repeat(64),
            recomputed_envelope_sha256: "e".repeat(64),
            recomputed_model_artifact_count: EXPECTED_ARTIFACT_COUNT,
            recomputed_fit_diagnostic_count: EXPECTED_DIAGNOSTIC_COUNT,
            mismatch_reasons: Vec::new(),
            verdict: HistoricalOutcomeTrainingOutputValidationVerdict::IndependentlyValidatedTrainOnlyArtifacts,
            training_output_independently_validated: true,
            future_validation_evaluation_implementation_registration_eligible: true,
            validation_selection_authorized: false,
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
    fn request_requires_all_five_confirmations() {
        let mut value = request();
        assert!(validate_request(&"a".repeat(32), &value).is_ok());
        value.validation_and_sealed_holdout_targets_remain_withheld_confirmed = false;
        assert!(validate_request(&"a".repeat(32), &value).is_err());
    }

    #[test]
    fn request_rejects_invalid_binding_hash() {
        let mut value = request();
        value.expected_output_sha256 = "not-a-sha".to_string();
        assert!(validate_request(&"a".repeat(32), &value).is_err());
    }

    #[test]
    fn validator_must_be_outside_complete_actor_chain() {
        let excluded = vec!["executor".to_string(), "reviewer".to_string()];
        assert!(!actor_is_independent("executor", &excluded));
        assert!(actor_is_independent("validator", &excluded));
    }

    #[test]
    fn f64_bit_encoding_distinguishes_one_ulp() {
        let value = 1.0_f64;
        let next = f64::from_bits(value.to_bits() + 1);
        assert_ne!(f64_hex(value), f64_hex(next));
    }

    #[test]
    fn zero_baseline_is_exact_and_deterministic() {
        let data = synthetic_data();
        let (left_models, left_predictions) = independent_zero_models(&data);
        let (right_models, right_predictions) = independent_zero_models(&data);
        assert_eq!(left_models, right_models);
        assert_eq!(left_predictions, right_predictions);
        assert!(left_predictions.iter().flatten().all(|value| *value == 0.0));
    }

    #[test]
    fn ridge_second_implementation_is_deterministic() {
        let data = synthetic_data();
        let left = independent_ridge_models(&data).unwrap();
        let right = independent_ridge_models(&data).unwrap();
        assert_eq!(left.0, right.0);
        assert_eq!(left.1, right.1);
        assert_eq!(left.0.len(), EXPECTED_TARGET_COUNT);
    }

    #[test]
    fn boosted_second_implementation_honors_seed_and_round_count() {
        let data = synthetic_data();
        let left = independent_boosted_models(&data, 17, 3, 100_000, 1).unwrap();
        let same = independent_boosted_models(&data, 17, 3, 100_000, 1).unwrap();
        let different = independent_boosted_models(&data, 29, 3, 100_000, 1).unwrap();
        assert_eq!(left, same);
        assert_ne!(left.0, different.0);
    }

    #[test]
    fn train_only_diagnostics_are_per_target_and_not_selection_metrics() {
        let data = synthetic_data();
        let (_, predictions) = independent_zero_models(&data);
        let diagnostics = independent_fit_diagnostics(
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
    fn valid_pass_record_keeps_every_downstream_capability_closed() {
        let record = valid_record();
        assert!(validate_record(&record).is_ok());
        assert!(record.training_output_independently_validated);
        assert!(!record.validation_selection_authorized);
        assert!(!record.trading_authorized);
    }

    #[test]
    fn record_tamper_or_authority_escalation_is_rejected() {
        let mut record = valid_record();
        record.reward_authorized = true;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        assert!(validate_record(&record).is_err());

        let mut record = valid_record();
        record.recomputed_fit_diagnostic_count -= 1;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        assert!(validate_record(&record).is_err());
    }
}
