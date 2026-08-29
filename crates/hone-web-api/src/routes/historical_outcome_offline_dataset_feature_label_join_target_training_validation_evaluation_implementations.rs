//! Immutable registration of a future validation-evaluation implementation.
//!
//! Stage 59 binds one independently reproduced Stage 58 train-only output to a frozen,
//! content-addressed evaluation contract before any validation label can be opened. Registration
//! has no callable entrypoint and cannot evaluate, tune, select, write a model/metric store, access
//! the sealed holdout, define reward semantics or obtain investment execution authority.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::HistoricalOutcomeTrainingUntrustedArtifactEnvelope;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::HistoricalOutcomeTrainingExperimentAlgorithm;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_output_validations::{
    HistoricalOutcomeTrainingOutputValidationRecord,
    IndependentlyValidatedHistoricalOutcomeTrainingOutput,
    independently_validated_training_outputs_for_validation_evaluation_implementation_registration,
    validate_historical_outcome_training_output_validation_for_evaluation_implementation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-implementation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-implementation-v1";
const POLICY_VERSION: &str =
    "hone-validation-evaluation-implementation-registration-v1-freeze-before-label-access";
const CONTRACT_SCHEMA_VERSION: &str = "hone-validation-evaluation-implementation-contract-v1";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-validation-evaluator-v1-per-target-three-seed-component-block-holm-gate";
const STATUS: &str = "registered_not_reviewed_not_run";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const EXPECTED_ARTIFACT_COUNT: usize = 9;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const BOOTSTRAP_REPLICATIONS: u32 = 10_000;
const BOOTSTRAP_RANDOM_SEED: u64 = 59_202_608_23;
const FAMILY_WISE_ALPHA_MILLIONTHS: u32 = 50_000;
const MINIMUM_RELATIVE_MAE_IMPROVEMENT_PPM: u32 = 50_000;
const MINIMUM_VALIDATION_ROWS: u32 = 100;
const MINIMUM_INDEPENDENT_COMPONENTS: u32 = 20;
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeValidationEvaluationImplementationRequest {
    expected_validation_id: String,
    expected_validation_sha256: String,
    expected_attempt_id: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_suite_specification_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_rows_sha256: String,
    expected_excluded_rows_sha256: String,
    expected_target_commitments_sha256: String,
    expected_candidate_set_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    exact_stage_58_validation_and_stage_57_output_binding_confirmed: bool,
    registrar_independent_from_complete_prior_chain_confirmed: bool,
    immutable_artifact_revision_and_protocol_confirmed: bool,
    evaluation_rules_frozen_before_validation_label_access_confirmed: bool,
    all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed: bool,
    zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed: bool,
    no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed: bool,
    validation_only_and_sealed_holdout_isolation_confirmed: bool,
    independent_review_runner_and_one_shot_authorization_required_confirmed: bool,
    no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationCandidateBinding {
    pub algorithm_id: String,
    pub random_seed: u64,
    pub artifact_sha256: String,
    pub exact_target_model_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_protocol_version: String,
    pub implementation_artifact_sha256: String,
    pub immutable_code_revision: String,
    pub upstream_validation_sha256: String,
    pub upstream_output_sha256: String,
    pub suite_specification_sha256: String,
    pub training_store_dataset_sha256: String,
    pub candidate_set_sha256: String,
    pub candidate_bindings: Vec<HistoricalOutcomeValidationEvaluationCandidateBinding>,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_vector_order: Vec<String>,
    pub primary_supervised_target_id: String,
    pub risk_target_id: String,
    pub benchmark_algorithm_id: String,
    pub candidate_algorithm_ids: Vec<String>,
    pub exact_feature_count: usize,
    pub exact_target_count: usize,
    pub exact_artifact_count: usize,
    pub exact_random_seeds: Vec<u64>,
    pub reported_metric_ids: Vec<String>,
    pub bootstrap_unit: String,
    pub bootstrap_replications: u32,
    pub bootstrap_random_seed: u64,
    pub family_wise_error_correction: String,
    pub family_wise_alpha_millionths: u32,
    pub exact_candidate_hypothesis_count: u32,
    pub minimum_relative_mae_improvement_ppm: u32,
    pub minimum_spearman_millionths: i32,
    pub minimum_directional_accuracy_millionths: u32,
    pub minimum_calibration_slope_millionths: u32,
    pub maximum_calibration_slope_millionths: u32,
    pub minimum_validation_rows: u32,
    pub minimum_independent_components: u32,
    pub all_three_seeds_must_pass: bool,
    pub tie_break_preferred_algorithm_id: String,
    pub seed_robustness_rule: String,
    pub per_target_admission_rule: String,
    pub tie_break_rule: String,
    pub insufficient_sample_rule: String,
    pub no_composite_score_or_global_model_validity_claim: bool,
    pub callable_entrypoint_present: bool,
    pub validation_features_access_allowed: bool,
    pub validation_labels_access_allowed: bool,
    pub sealed_holdout_features_access_allowed: bool,
    pub sealed_holdout_labels_access_allowed: bool,
    pub training_or_preprocessing_update_allowed: bool,
    pub hyperparameter_or_threshold_tuning_allowed: bool,
    pub candidate_selection_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub outbound_network_allowed: bool,
    pub secrets_allowed: bool,
    pub scalar_reward_defined: bool,
    pub action_position_or_ranking_semantics_defined: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub upstream_validation: HistoricalOutcomeTrainingOutputValidationRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_complete_prior_chain: bool,
    pub implementation_name: String,
    pub rationale: String,
    pub known_limitations: String,
    pub implementation_contract: HistoricalOutcomeValidationEvaluationImplementationContract,
    pub status: String,
    pub exact_stage_58_validation_and_stage_57_output_binding_confirmed: bool,
    pub registrar_independent_from_complete_prior_chain_confirmed: bool,
    pub immutable_artifact_revision_and_protocol_confirmed: bool,
    pub evaluation_rules_frozen_before_validation_label_access_confirmed: bool,
    pub all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed: bool,
    pub zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed: bool,
    pub no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed: bool,
    pub validation_only_and_sealed_holdout_isolation_confirmed: bool,
    pub independent_review_runner_and_one_shot_authorization_required_confirmed: bool,
    pub no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub validation_evaluation_implementation_registered: bool,
    pub future_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub isolated_runner_registration_eligible: bool,
    pub validation_label_access_authorized: bool,
    pub evaluation_started: bool,
    pub evaluation_completed: bool,
    pub candidate_selection_authorized: bool,
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
pub(crate) struct HistoricalOutcomeValidationEvaluationEligibleOutput {
    pub validation: HistoricalOutcomeTrainingOutputValidationRecord,
    pub candidate_bindings: Vec<HistoricalOutcomeValidationEvaluationCandidateBinding>,
    pub candidate_set_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_vector_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationItem {
    pub implementation: HistoricalOutcomeValidationEvaluationImplementationRecord,
    pub upstream_binding_current: bool,
    pub future_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_outputs: Vec<HistoricalOutcomeValidationEvaluationEligibleOutput>,
    pub items: Vec<HistoricalOutcomeValidationEvaluationImplementationItem>,
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub validation_label_access_authorized: bool,
    pub evaluation_started: bool,
    pub candidate_selection_authorized: bool,
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
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationReadinessSummary {
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_historical_outcome_validation_evaluation_implementations(
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
            warn!(%error, "validation evaluation implementation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "validation 评估实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_validation_evaluation_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeValidationEvaluationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_validation_evaluation_implementation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeValidationEvaluationImplementationReadinessSummary {
            registration_eligible_count: registry.registration_eligible_count,
            implementation_count: registry.implementation_count,
            current_binding_implementation_count: registry.current_binding_implementation_count,
            independent_implementation_review_eligible_count: registry
                .independent_implementation_review_eligible_count,
            implementation_status: registry.implementation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independent_review_eligible_validation_evaluation_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeValidationEvaluationImplementationRecord>, String> {
    let registry = build_registry(state).await?;
    let records = registry
        .items
        .into_iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .map(|item| item.implementation)
        .collect::<Vec<_>>();
    for record in &records {
        validate_record(record)?;
    }
    Ok(records)
}

#[allow(dead_code)]
pub(crate) fn validate_historical_outcome_validation_evaluation_implementation_for_review(
    record: &HistoricalOutcomeValidationEvaluationImplementationRecord,
) -> Result<(), String> {
    validate_record(record)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationRegistry, String> {
    let eligible = eligible_sources(state).await?;
    let eligible_by_validation = eligible
        .iter()
        .map(|source| (source.validation.validation_id.as_str(), source))
        .collect::<HashMap<_, _>>();
    let mut items = read_records(state)
        .await?
        .into_iter()
        .map(|implementation| {
            let upstream_binding_current = eligible_by_validation
                .get(implementation.upstream_validation.validation_id.as_str())
                .is_some_and(|source| record_matches_source(&implementation, source));
            HistoricalOutcomeValidationEvaluationImplementationItem {
                future_independent_implementation_review_eligible: upstream_binding_current,
                upstream_binding_current,
                implementation,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .implementation
            .registered_at
            .cmp(&left.implementation.registered_at)
            .then_with(|| {
                right
                    .implementation
                    .implementation_id
                    .cmp(&left.implementation.implementation_id)
            })
    });
    let registered_validation_ids = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .map(|item| {
            item.implementation
                .upstream_validation
                .validation_id
                .as_str()
        })
        .collect::<BTreeSet<_>>();
    let registration_eligible_count = eligible
        .iter()
        .filter(|source| {
            !registered_validation_ids.contains(source.validation.validation_id.as_str())
        })
        .count();
    let current_binding_implementation_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .count();
    let implementation_status = if current_binding_implementation_count > 0 {
        "validation_evaluation_implementation_registered_not_run_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "independently_validated_training_output_ready_for_evaluation_implementation_registration"
    } else {
        "waiting_stage_58_independently_validated_training_output"
    };
    Ok(HistoricalOutcomeValidationEvaluationImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_outputs: eligible,
        implementation_count: items.len(),
        current_binding_implementation_count,
        independent_implementation_review_eligible_count,
        items,
        registration_eligible_count,
        implementation_status: implementation_status.to_string(),
        validation_label_access_authorized: false,
        evaluation_started: false,
        candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 59 阶段只在读取 validation 标签之前，把精确 Stage 58 验证记录、9 个冻结模型工件、9 项目标和固定评估协议登记为不可变实现。协议逐目标逐种子报告，零预测作配对基准，使用 component block bootstrap 与 Holm 修正，禁止 seed shopping、调参和综合分遮蔽。当前没有入口、标签访问、评估、选模、sealed holdout、模型/指标库、奖励、影子、订单、券商或交易权限。".to_string(),
    })
}

async fn eligible_sources(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeValidationEvaluationEligibleOutput>, String> {
    independently_validated_training_outputs_for_validation_evaluation_implementation_registration(
        state,
    )
    .await?
    .into_iter()
    .map(source_from_upstream)
    .collect()
}

fn source_from_upstream(
    upstream: IndependentlyValidatedHistoricalOutcomeTrainingOutput,
) -> Result<HistoricalOutcomeValidationEvaluationEligibleOutput, String> {
    validate_historical_outcome_training_output_validation_for_evaluation_implementation(
        &upstream.validation,
    )?;
    let envelope = exact_envelope(&upstream)?;
    let candidate_bindings = candidate_bindings(envelope)?;
    let candidate_set_sha256 = hash_serializable(&candidate_bindings)?;
    Ok(HistoricalOutcomeValidationEvaluationEligibleOutput {
        validation: upstream.validation.clone(),
        candidate_bindings,
        candidate_set_sha256,
        feature_order_sha256: hash_serializable(&envelope.feature_order)?,
        preprocessing_sha256: hash_serializable(&envelope.preprocessing)?,
        target_vector_order: envelope.target_order.clone(),
    })
}

fn exact_envelope(
    upstream: &IndependentlyValidatedHistoricalOutcomeTrainingOutput,
) -> Result<&HistoricalOutcomeTrainingUntrustedArtifactEnvelope, String> {
    let envelope = upstream
        .attempt
        .result
        .untrusted_artifact_envelope
        .as_ref()
        .ok_or_else(|| "Stage 57 训练结果缺少冻结模型工件 envelope".to_string())?;
    if upstream.attempt.claim.attempt_id != upstream.validation.attempt_id
        || upstream.attempt.claim.claim_sha256 != upstream.validation.claim_sha256
        || upstream.attempt.result.result_sha256 != upstream.validation.result_sha256
        || upstream.attempt.result.output_sha256.as_deref()
            != Some(upstream.validation.output_sha256.as_str())
        || envelope.model_artifacts.len() != EXPECTED_ARTIFACT_COUNT
        || envelope.feature_order.len() != EXPECTED_FEATURE_COUNT
        || envelope.preprocessing.len() != EXPECTED_FEATURE_COUNT
        || envelope.target_order != expected_target_order()
        || envelope.model_artifacts.iter().any(|artifact| {
            artifact.train_row_count == 0
                || artifact.exact_feature_count != EXPECTED_FEATURE_COUNT
                || artifact.exact_target_count != EXPECTED_TARGET_COUNT
                || artifact
                    .target_models
                    .iter()
                    .map(|model| model.target_id.as_str())
                    .ne(envelope.target_order.iter().map(String::as_str))
                || artifact.validation_selected
                || artifact.sealed_holdout_accessed
        })
        || envelope.validation_labels_accessed
        || envelope.validation_selection_completed
        || envelope.sealed_holdout_labels_accessed
        || envelope.model_artifact_store_written
        || envelope.metric_store_written
        || envelope.scalar_reward_written
        || envelope.shadow_position_written
        || envelope.order_generated
        || envelope.broker_accessed
        || envelope.trade_executed
    {
        return Err("Stage 57–58 输出链无效、计数异常或越权".to_string());
    }
    Ok(envelope)
}

fn candidate_bindings(
    envelope: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
) -> Result<Vec<HistoricalOutcomeValidationEvaluationCandidateBinding>, String> {
    let mut values = envelope
        .model_artifacts
        .iter()
        .map(
            |artifact| HistoricalOutcomeValidationEvaluationCandidateBinding {
                algorithm_id: algorithm_id(&artifact.algorithm).to_string(),
                random_seed: artifact.random_seed,
                artifact_sha256: artifact.artifact_sha256.clone(),
                exact_target_model_count: artifact.target_models.len(),
            },
        )
        .collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.algorithm_id
            .cmp(&right.algorithm_id)
            .then(left.random_seed.cmp(&right.random_seed))
    });
    validate_candidate_bindings(&values)?;
    Ok(values)
}

fn algorithm_id(algorithm: &HistoricalOutcomeTrainingExperimentAlgorithm) -> &'static str {
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

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeValidationEvaluationImplementationRequest,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationRecord, String> {
    if registrar_id.trim().is_empty() {
        return Err("validation 评估实现登记人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let eligible = eligible_sources(state).await?;
    let source = eligible
        .iter()
        .find(|source| source.validation.validation_id == request.expected_validation_id)
        .ok_or_else(|| "当前 Stage 58 独立验证输出不存在，请刷新后重试".to_string())?;
    ensure_validation_unregistered(&read_records(state).await?, &source.validation)?;
    let record = build_record(source, registrar_id, request, Utc::now())?;
    write_immutable_json(
        &record_root(state).join(format!("{}.json", record.implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn build_record(
    source: &HistoricalOutcomeValidationEvaluationEligibleOutput,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeValidationEvaluationImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationRecord, String> {
    validate_historical_outcome_training_output_validation_for_evaluation_implementation(
        &source.validation,
    )?;
    validate_expected_bindings(source, &request)?;
    let implementation_name =
        bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    let immutable_code_revision = bounded_required(
        &request.immutable_code_revision,
        MAX_REVISION_CHARS,
        "不可变代码版本",
    )?;
    if !valid_sha256(&request.implementation_artifact_sha256) {
        return Err("validation 评估实现工件 SHA-256 无效".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "登记理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    if !request_checks_confirmed(&request) {
        return Err("登记前必须逐项确认预注册、独立角色、统计协议和零权限边界".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(&source.validation);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == registrar_id)
    {
        return Err("评估实现登记人参与过 Stage 51–58 或完整上游，必须更换独立角色".to_string());
    }
    let contract = implementation_contract(
        source,
        &request.implementation_artifact_sha256,
        &immutable_code_revision,
    )?;
    let mut record = HistoricalOutcomeValidationEvaluationImplementationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        upstream_validation: source.validation.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_complete_prior_chain: true,
        implementation_name,
        rationale,
        known_limitations,
        implementation_contract: contract,
        status: STATUS.to_string(),
        exact_stage_58_validation_and_stage_57_output_binding_confirmed: request
            .exact_stage_58_validation_and_stage_57_output_binding_confirmed,
        registrar_independent_from_complete_prior_chain_confirmed: request
            .registrar_independent_from_complete_prior_chain_confirmed,
        immutable_artifact_revision_and_protocol_confirmed: request
            .immutable_artifact_revision_and_protocol_confirmed,
        evaluation_rules_frozen_before_validation_label_access_confirmed: request
            .evaluation_rules_frozen_before_validation_label_access_confirmed,
        all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed: request
            .all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed,
        zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed: request
            .zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed,
        no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed: request
            .no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed,
        validation_only_and_sealed_holdout_isolation_confirmed: request
            .validation_only_and_sealed_holdout_isolation_confirmed,
        independent_review_runner_and_one_shot_authorization_required_confirmed: request
            .independent_review_runner_and_one_shot_authorization_required_confirmed,
        no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed: request
            .no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed,
        validation_evaluation_implementation_registered: true,
        future_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        isolated_runner_registration_eligible: false,
        validation_label_access_authorized: false,
        evaluation_started: false,
        evaluation_completed: false,
        candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.implementation_sha256 = record_fingerprint(&record)?;
    record.implementation_id = record.implementation_sha256[..32].to_string();
    validate_record(&record)?;
    Ok(record)
}

fn implementation_contract(
    source: &HistoricalOutcomeValidationEvaluationEligibleOutput,
    implementation_artifact_sha256: &str,
    immutable_code_revision: &str,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationContract, String> {
    let mut contract = HistoricalOutcomeValidationEvaluationImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_protocol_version: IMPLEMENTATION_PROTOCOL_VERSION.to_string(),
        implementation_artifact_sha256: implementation_artifact_sha256.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        upstream_validation_sha256: source.validation.validation_sha256.clone(),
        upstream_output_sha256: source.validation.output_sha256.clone(),
        suite_specification_sha256: source.validation.suite_specification_sha256.clone(),
        training_store_dataset_sha256: source.validation.training_store_dataset_sha256.clone(),
        candidate_set_sha256: source.candidate_set_sha256.clone(),
        candidate_bindings: source.candidate_bindings.clone(),
        feature_order_sha256: source.feature_order_sha256.clone(),
        preprocessing_sha256: source.preprocessing_sha256.clone(),
        target_vector_order: source.target_vector_order.clone(),
        primary_supervised_target_id: "excess_return_250_continuous".to_string(),
        risk_target_id: "asset_max_drawdown_250_continuous".to_string(),
        benchmark_algorithm_id: "frozen_zero_prediction_baseline".to_string(),
        candidate_algorithm_ids: vec![
            "gradient_boosted_multi_target_regression".to_string(),
            "ridge_multi_target_regression".to_string(),
        ],
        exact_feature_count: EXPECTED_FEATURE_COUNT,
        exact_target_count: EXPECTED_TARGET_COUNT,
        exact_artifact_count: EXPECTED_ARTIFACT_COUNT,
        exact_random_seeds: EXPECTED_SEEDS.to_vec(),
        reported_metric_ids: [
            "validation_mae_per_target_per_seed",
            "paired_relative_mae_improvement_vs_zero_per_target_per_seed",
            "component_block_bootstrap_mae_improvement_p_value_per_target_per_seed",
            "holm_adjusted_p_value_across_54_candidate_target_seed_tests",
            "validation_spearman_per_target_per_seed",
            "validation_directional_accuracy_per_target_per_seed",
            "validation_calibration_slope_per_target_per_seed",
        ]
        .map(ToString::to_string)
        .to_vec(),
        bootstrap_unit: "official_split_component_id".to_string(),
        bootstrap_replications: BOOTSTRAP_REPLICATIONS,
        bootstrap_random_seed: BOOTSTRAP_RANDOM_SEED,
        family_wise_error_correction: "holm_bonferroni_across_54_candidate_target_seed_tests"
            .to_string(),
        family_wise_alpha_millionths: FAMILY_WISE_ALPHA_MILLIONTHS,
        exact_candidate_hypothesis_count: 54,
        minimum_relative_mae_improvement_ppm: MINIMUM_RELATIVE_MAE_IMPROVEMENT_PPM,
        minimum_spearman_millionths: 0,
        minimum_directional_accuracy_millionths: 500_000,
        minimum_calibration_slope_millionths: 500_000,
        maximum_calibration_slope_millionths: 1_500_000,
        minimum_validation_rows: MINIMUM_VALIDATION_ROWS,
        minimum_independent_components: MINIMUM_INDEPENDENT_COMPONENTS,
        all_three_seeds_must_pass: true,
        tie_break_preferred_algorithm_id: "ridge_multi_target_regression".to_string(),
        seed_robustness_rule: "一个 algorithm-target 只有三个冻结种子都达到预注册 MAE 改善、Holm 修正显著性和非退化诊断时才可进入未来逐目标候选；不得挑选表现最好的 seed。".to_string(),
        per_target_admission_rule: "每个目标独立判定：相对零预测 MAE 至少改善 5%，54 项配对 component-block bootstrap 检验经 Holm 修正后 q<=0.05，Spearman 为正、方向准确率不低于 0.50、校准斜率处于 [0.5,1.5]；失败目标必须显式保留，不得由其他目标抵消。".to_string(),
        tie_break_rule: "同一目标若 ridge 与 boosted 均满足全部三种子门槛，只按三个种子的 validation MAE 中位数选择；精确相等时固定优先 ridge。该规则冻结后不得改写。".to_string(),
        insufficient_sample_rule: "validation 少于 100 行或少于 20 个独立 official component 时只输出 insufficient_evidence，不得选模或声明有效。".to_string(),
        no_composite_score_or_global_model_validity_claim: true,
        callable_entrypoint_present: false,
        validation_features_access_allowed: false,
        validation_labels_access_allowed: false,
        sealed_holdout_features_access_allowed: false,
        sealed_holdout_labels_access_allowed: false,
        training_or_preprocessing_update_allowed: false,
        hyperparameter_or_threshold_tuning_allowed: false,
        candidate_selection_allowed: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        outbound_network_allowed: false,
        secrets_allowed: false,
        scalar_reward_defined: false,
        action_position_or_ranking_semantics_defined: false,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn validate_expected_bindings(
    source: &HistoricalOutcomeValidationEvaluationEligibleOutput,
    request: &RegisterHistoricalOutcomeValidationEvaluationImplementationRequest,
) -> Result<(), String> {
    let validation = &source.validation;
    if request.expected_validation_id != validation.validation_id
        || request.expected_validation_sha256 != validation.validation_sha256
        || request.expected_attempt_id != validation.attempt_id
        || request.expected_claim_sha256 != validation.claim_sha256
        || request.expected_result_sha256 != validation.result_sha256
        || request.expected_output_sha256 != validation.output_sha256
        || request.expected_suite_specification_sha256 != validation.suite_specification_sha256
        || request.expected_training_store_dataset_sha256
            != validation.training_store_dataset_sha256
        || request.expected_rows_sha256 != validation.rows_sha256
        || request.expected_excluded_rows_sha256 != validation.excluded_rows_sha256
        || request.expected_target_commitments_sha256 != validation.target_commitments_sha256
        || request.expected_candidate_set_sha256 != source.candidate_set_sha256
    {
        return Err("Stage 57–58 输出、候选集合或完整上游绑定已变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn request_checks_confirmed(
    request: &RegisterHistoricalOutcomeValidationEvaluationImplementationRequest,
) -> bool {
    request.exact_stage_58_validation_and_stage_57_output_binding_confirmed
        && request.registrar_independent_from_complete_prior_chain_confirmed
        && request.immutable_artifact_revision_and_protocol_confirmed
        && request.evaluation_rules_frozen_before_validation_label_access_confirmed
        && request.all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed
        && request.zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed
        && request.no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed
        && request.validation_only_and_sealed_holdout_isolation_confirmed
        && request.independent_review_runner_and_one_shot_authorization_required_confirmed
        && request.no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn expected_excluded_prior_actor_ids(
    validation: &HistoricalOutcomeTrainingOutputValidationRecord,
) -> Vec<String> {
    let mut values = validation.excluded_prior_actor_ids.clone();
    values.push(validation.validated_by.clone());
    values.push(validation.invoked_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn validate_candidate_bindings(
    bindings: &[HistoricalOutcomeValidationEvaluationCandidateBinding],
) -> Result<(), String> {
    let expected_algorithms = [
        "frozen_zero_prediction_baseline",
        "gradient_boosted_multi_target_regression",
        "ridge_multi_target_regression",
    ];
    let mut tuples = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    if bindings.windows(2).any(|pair| {
        (pair[0].algorithm_id.as_str(), pair[0].random_seed)
            >= (pair[1].algorithm_id.as_str(), pair[1].random_seed)
    }) {
        return Err("validation 评估候选工件必须按算法和种子规范排序".to_string());
    }
    for binding in bindings {
        if !expected_algorithms.contains(&binding.algorithm_id.as_str())
            || !EXPECTED_SEEDS.contains(&binding.random_seed)
            || !valid_sha256(&binding.artifact_sha256)
            || binding.exact_target_model_count != EXPECTED_TARGET_COUNT
            || !tuples.insert((binding.algorithm_id.clone(), binding.random_seed))
            || !hashes.insert(binding.artifact_sha256.clone())
        {
            return Err("validation 评估候选工件矩阵无效、重复或不完整".to_string());
        }
    }
    if bindings.len() != EXPECTED_ARTIFACT_COUNT
        || expected_algorithms.iter().any(|algorithm| {
            EXPECTED_SEEDS
                .iter()
                .any(|seed| !tuples.contains(&(algorithm.to_string(), *seed)))
        })
    {
        return Err("validation 评估必须精确绑定三算法 × 三种子的 9 个工件".to_string());
    }
    Ok(())
}

fn expected_target_order() -> Vec<String> {
    [20_u16, 60, 250]
        .into_iter()
        .flat_map(|horizon| {
            ["asset_return", "excess_return", "asset_max_drawdown"]
                .into_iter()
                .map(move |metric| format!("{metric}_{horizon}_continuous"))
        })
        .collect()
}

fn record_matches_source(
    record: &HistoricalOutcomeValidationEvaluationImplementationRecord,
    source: &HistoricalOutcomeValidationEvaluationEligibleOutput,
) -> bool {
    record.upstream_validation == source.validation
        && record.implementation_contract.candidate_bindings == source.candidate_bindings
        && record.implementation_contract.candidate_set_sha256 == source.candidate_set_sha256
        && record.implementation_contract.feature_order_sha256 == source.feature_order_sha256
        && record.implementation_contract.preprocessing_sha256 == source.preprocessing_sha256
        && record.implementation_contract.target_vector_order == source.target_vector_order
}

fn ensure_validation_unregistered(
    records: &[HistoricalOutcomeValidationEvaluationImplementationRecord],
    validation: &HistoricalOutcomeTrainingOutputValidationRecord,
) -> Result<(), String> {
    if records
        .iter()
        .any(|record| record.upstream_validation.validation_id == validation.validation_id)
    {
        return Err("该 Stage 58 验证记录已经登记过评估实现，不得重复或覆盖".to_string());
    }
    Ok(())
}

fn validate_record(
    record: &HistoricalOutcomeValidationEvaluationImplementationRecord,
) -> Result<(), String> {
    validate_historical_outcome_training_output_validation_for_evaluation_implementation(
        &record.upstream_validation,
    )?;
    validate_candidate_bindings(&record.implementation_contract.candidate_bindings)?;
    let source = HistoricalOutcomeValidationEvaluationEligibleOutput {
        validation: record.upstream_validation.clone(),
        candidate_bindings: record.implementation_contract.candidate_bindings.clone(),
        candidate_set_sha256: hash_serializable(
            &record.implementation_contract.candidate_bindings,
        )?,
        feature_order_sha256: record.implementation_contract.feature_order_sha256.clone(),
        preprocessing_sha256: record.implementation_contract.preprocessing_sha256.clone(),
        target_vector_order: record.implementation_contract.target_vector_order.clone(),
    };
    let expected_contract = implementation_contract(
        &source,
        &record
            .implementation_contract
            .implementation_artifact_sha256,
        &record.implementation_contract.immutable_code_revision,
    )?;
    let confirmations = record.exact_stage_58_validation_and_stage_57_output_binding_confirmed
        && record.registrar_independent_from_complete_prior_chain_confirmed
        && record.immutable_artifact_revision_and_protocol_confirmed
        && record.evaluation_rules_frozen_before_validation_label_access_confirmed
        && record.all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed
        && record.zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed
        && record.no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed
        && record.validation_only_and_sealed_holdout_isolation_confirmed
        && record.independent_review_runner_and_one_shot_authorization_required_confirmed
        && record.no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !record.independent_implementation_review_completed
        && !record.isolated_runner_registration_eligible
        && !record.validation_label_access_authorized
        && !record.evaluation_started
        && !record.evaluation_completed
        && !record.candidate_selection_authorized
        && !record.sealed_holdout_access_authorized
        && !record.model_artifact_store_written
        && !record.metric_store_written
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let contract = &record.implementation_contract;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || record.status != STATUS
        || !valid_id(&record.implementation_id)
        || !valid_sha256(&record.implementation_sha256)
        || record.implementation_id != record.implementation_sha256[..32]
        || record.implementation_sha256 != record_fingerprint(record)?
        || record.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&record.upstream_validation)
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record.registered_by.trim().is_empty()
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.registered_by)
        || !record.registrar_independent_from_complete_prior_chain
        || record.implementation_name.trim().is_empty()
        || record.implementation_name.chars().count() > MAX_NAME_CHARS
        || record.rationale.trim().is_empty()
        || record.rationale.chars().count() > MAX_RATIONALE_CHARS
        || record.known_limitations.trim().is_empty()
        || record.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || !valid_sha256(&contract.contract_sha256)
        || contract.contract_sha256 != fingerprint_without(contract, &["contract_sha256"])?
        || !valid_sha256(&contract.implementation_artifact_sha256)
        || !valid_sha256(&contract.upstream_validation_sha256)
        || !valid_sha256(&contract.upstream_output_sha256)
        || !valid_sha256(&contract.suite_specification_sha256)
        || !valid_sha256(&contract.training_store_dataset_sha256)
        || !valid_sha256(&contract.candidate_set_sha256)
        || !valid_sha256(&contract.feature_order_sha256)
        || !valid_sha256(&contract.preprocessing_sha256)
        || contract.immutable_code_revision.trim().is_empty()
        || contract.immutable_code_revision.chars().count() > MAX_REVISION_CHARS
        || contract.target_vector_order != expected_target_order()
        || *contract != expected_contract
        || !confirmations
        || !record.validation_evaluation_implementation_registered
        || !record.future_independent_implementation_review_eligible
        || !authority_closed
    {
        return Err(
            "validation evaluation implementation is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeValidationEvaluationImplementationRecord>, String> {
    let root = record_root(state);
    let mut directory = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    let mut validations = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let record =
            serde_json::from_slice::<HistoricalOutcomeValidationEvaluationImplementationRecord>(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_record(&record)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(record.implementation_id.as_str())
            || !ids.insert(record.implementation_id.clone())
            || !hashes.insert(record.implementation_sha256.clone())
            || !validations.insert(record.upstream_validation.validation_id.clone())
        {
            return Err("validation 评估实现登记含误命名、重复哈希或重放".to_string());
        }
        records.push(record);
    }
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeValidationEvaluationImplementationRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["implementation_id", "implementation_sha256"])
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
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, max_chars: usize, field: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars {
        return Err(format!("{field}为空或过长"));
    }
    Ok(value.to_string())
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

fn record_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical_outcome_validation_evaluation_implementations/records")
}

struct RegistrationLock {
    path: PathBuf,
}

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("historical-outcome-validation-evaluation-implementation.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(RegistrationLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                    .is_some_and(|age| age > StdDuration::from_secs(600));
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                if attempt < 7 {
                    tokio::time::sleep(StdDuration::from_millis(25)).await;
                    continue;
                }
                return Err("另一个管理员正在登记 validation 评估实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("validation 评估实现登记锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_output_validations::tests::valid_record as valid_validation_record;

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

    fn bindings() -> Vec<HistoricalOutcomeValidationEvaluationCandidateBinding> {
        [
            "frozen_zero_prediction_baseline",
            "gradient_boosted_multi_target_regression",
            "ridge_multi_target_regression",
        ]
        .into_iter()
        .flat_map(|algorithm| {
            EXPECTED_SEEDS.into_iter().map(move |seed| {
                HistoricalOutcomeValidationEvaluationCandidateBinding {
                    algorithm_id: algorithm.to_string(),
                    random_seed: seed,
                    artifact_sha256: hash_serializable(&(algorithm, seed)).unwrap(),
                    exact_target_model_count: EXPECTED_TARGET_COUNT,
                }
            })
        })
        .collect()
    }

    fn source() -> HistoricalOutcomeValidationEvaluationEligibleOutput {
        let candidate_bindings = bindings();
        HistoricalOutcomeValidationEvaluationEligibleOutput {
            validation: valid_validation_record(),
            candidate_set_sha256: hash_serializable(&candidate_bindings).unwrap(),
            candidate_bindings,
            feature_order_sha256: "c".repeat(64),
            preprocessing_sha256: "d".repeat(64),
            target_vector_order: target_order(),
        }
    }

    fn request(
        source: &HistoricalOutcomeValidationEvaluationEligibleOutput,
    ) -> RegisterHistoricalOutcomeValidationEvaluationImplementationRequest {
        RegisterHistoricalOutcomeValidationEvaluationImplementationRequest {
            expected_validation_id: source.validation.validation_id.clone(),
            expected_validation_sha256: source.validation.validation_sha256.clone(),
            expected_attempt_id: source.validation.attempt_id.clone(),
            expected_claim_sha256: source.validation.claim_sha256.clone(),
            expected_result_sha256: source.validation.result_sha256.clone(),
            expected_output_sha256: source.validation.output_sha256.clone(),
            expected_suite_specification_sha256: source
                .validation
                .suite_specification_sha256
                .clone(),
            expected_training_store_dataset_sha256: source
                .validation
                .training_store_dataset_sha256
                .clone(),
            expected_rows_sha256: source.validation.rows_sha256.clone(),
            expected_excluded_rows_sha256: source.validation.excluded_rows_sha256.clone(),
            expected_target_commitments_sha256: source.validation.target_commitments_sha256.clone(),
            expected_candidate_set_sha256: source.candidate_set_sha256.clone(),
            implementation_name: "frozen validation evaluator".to_string(),
            immutable_code_revision: "git:stage59-test".to_string(),
            implementation_artifact_sha256: "e".repeat(64),
            rationale: "先冻结逐目标评估协议，再允许未来独立评估".to_string(),
            known_limitations: "当前只有协议登记，没有真实 validation 评估".to_string(),
            exact_stage_58_validation_and_stage_57_output_binding_confirmed: true,
            registrar_independent_from_complete_prior_chain_confirmed: true,
            immutable_artifact_revision_and_protocol_confirmed: true,
            evaluation_rules_frozen_before_validation_label_access_confirmed: true,
            all_nine_artifacts_targets_seeds_and_metrics_reported_separately_confirmed: true,
            zero_baseline_paired_component_block_bootstrap_and_holm_correction_confirmed: true,
            no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed: true,
            validation_only_and_sealed_holdout_isolation_confirmed: true,
            independent_review_runner_and_one_shot_authorization_required_confirmed: true,
            no_label_access_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn record() -> HistoricalOutcomeValidationEvaluationImplementationRecord {
        let source = source();
        build_record(&source, "stage59-registrar", request(&source), Utc::now()).unwrap()
    }

    #[test]
    fn candidate_matrix_requires_exact_three_by_three_artifacts() {
        let mut values = bindings();
        assert!(validate_candidate_bindings(&values).is_ok());
        values.pop();
        assert!(validate_candidate_bindings(&values).is_err());

        let mut values = bindings();
        values.swap(0, 1);
        assert!(validate_candidate_bindings(&values).is_err());
    }

    #[test]
    fn protocol_is_frozen_before_label_access_and_has_no_composite_score() {
        let source = source();
        let contract = implementation_contract(&source, &"e".repeat(64), "revision").unwrap();
        assert_eq!(contract.bootstrap_replications, 10_000);
        assert_eq!(contract.exact_candidate_hypothesis_count, 54);
        assert!(contract.all_three_seeds_must_pass);
        assert_eq!(
            contract.family_wise_error_correction,
            "holm_bonferroni_across_54_candidate_target_seed_tests"
        );
        assert!(contract.no_composite_score_or_global_model_validity_claim);
        assert!(!contract.validation_labels_access_allowed);
        assert!(!contract.candidate_selection_allowed);
        assert!(!contract.sealed_holdout_labels_access_allowed);
    }

    #[test]
    fn every_confirmation_is_required() {
        let source = source();
        let mut value = request(&source);
        assert!(request_checks_confirmed(&value));
        value.no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed = false;
        assert!(!request_checks_confirmed(&value));
    }

    #[test]
    fn exact_candidate_set_binding_rejects_drift() {
        let source = source();
        let mut value = request(&source);
        assert!(validate_expected_bindings(&source, &value).is_ok());
        value.expected_candidate_set_sha256 = "f".repeat(64);
        assert!(validate_expected_bindings(&source, &value).is_err());
    }

    #[test]
    fn registrar_must_be_outside_stage_58_and_complete_prior_chain() {
        let source = source();
        let excluded = expected_excluded_prior_actor_ids(&source.validation);
        assert!(excluded.contains(&source.validation.validated_by));
        assert!(excluded.contains(&source.validation.invoked_by));
        let failure = build_record(
            &source,
            &source.validation.validated_by,
            request(&source),
            Utc::now(),
        );
        assert!(failure.is_err());
    }

    #[test]
    fn valid_registration_keeps_all_labels_selection_and_trading_closed() {
        let value = record();
        assert!(validate_record(&value).is_ok());
        assert!(value.future_independent_implementation_review_eligible);
        assert!(!value.validation_label_access_authorized);
        assert!(!value.candidate_selection_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn tamper_or_authority_escalation_is_rejected_even_after_rehash() {
        let mut value = record();
        value
            .implementation_contract
            .minimum_relative_mae_improvement_ppm = 0;
        value.implementation_contract.contract_sha256 =
            fingerprint_without(&value.implementation_contract, &["contract_sha256"]).unwrap();
        value.implementation_sha256 = record_fingerprint(&value).unwrap();
        value.implementation_id = value.implementation_sha256[..32].to_string();
        assert!(validate_record(&value).is_err());

        let mut value = record();
        value.validation_label_access_authorized = true;
        value.implementation_sha256 = record_fingerprint(&value).unwrap();
        value.implementation_id = value.implementation_sha256[..32].to_string();
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn one_validation_can_register_only_once() {
        let value = record();
        assert!(ensure_validation_unregistered(&[], &value.upstream_validation).is_ok());
        assert!(
            ensure_validation_unregistered(&[value.clone()], &value.upstream_validation,).is_err()
        );
    }
}
