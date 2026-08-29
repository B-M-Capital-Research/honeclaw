//! Claim-first, create-once registration of one exact Stage 50 admitted
//! training experiment suite.
//!
//! Registration freezes the dataset binding, three comparison arms, three
//! deterministic seeds, target semantics, split isolation and static resource
//! ceilings. It never runs training, opens sealed holdout labels, defines a
//! scalar reward, creates a shadow portfolio, generates an order, accesses a
//! broker or trades. A completed registration remains unusable until a later
//! independent registration review approves a separately governed runner.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_registration_admission_reviews::{
    AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
    admitted_training_store_datasets_for_training_registration,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-feature-label-join-target-training-experiment-registration-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-experiment-registration-claim-v1";
const SPEC_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-experiment-suite-spec-v1";
const REGISTRATION_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-experiment-registration-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-experiment-registration-result-v1";
const POLICY_VERSION: &str =
    "hone-training-experiment-registration-claim-first-create-once-v1-no-run";
const SUITE_VERSION: &str = "hone-nine-target-comparison-suite-v1";
const TARGET_CONTRACT_VERSION: &str = "hone-nine-raw-continuous-outcomes-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const MAX_NAME_CHARS: usize = 120;
const MAX_HYPOTHESIS_CHARS: usize = 1_200;
const MAX_LIMITATIONS_CHARS: usize = 2_400;
const MAX_ERROR_CHARS: usize = 1_200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeTrainingExperimentAlgorithm {
    FrozenZeroPredictionBaseline,
    RidgeMultiTargetRegression,
    GradientBoostedMultiTargetRegression,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingExperimentArm {
    pub algorithm: HistoricalOutcomeTrainingExperimentAlgorithm,
    pub role: String,
    pub random_seeds: Vec<u64>,
    pub max_epochs_or_boosting_rounds: u16,
    pub learning_rate_micros: u32,
    pub l2_regularization_micros: u32,
    pub maximum_tree_depth: u8,
    pub deterministic_replay_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingExperimentResourceCeilings {
    pub maximum_wall_clock_seconds: u32,
    pub maximum_memory_mib: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u16,
    pub maximum_output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingExperimentSuiteSpecification {
    pub schema_version: String,
    pub suite_version: String,
    pub specification_sha256: String,
    pub feature_catalog_count: usize,
    pub target_count: usize,
    pub target_contract_version: String,
    pub target_vector_order: Vec<String>,
    pub primary_supervised_target_id: String,
    pub risk_target_id: String,
    pub arms: Vec<HistoricalOutcomeTrainingExperimentArm>,
    pub fit_split: String,
    pub model_selection_split: String,
    pub sealed_holdout_split: String,
    pub feature_preprocessing_contract: String,
    pub objective_contract: String,
    pub model_selection_contract: String,
    pub reported_metric_ids: Vec<String>,
    pub sealed_holdout_access_allowed: bool,
    pub sealed_holdout_labels_visible_to_training_worker: bool,
    pub scalar_reward_defined: bool,
    pub action_position_or_ranking_semantics_defined: bool,
    pub resource_ceilings: HistoricalOutcomeTrainingExperimentResourceCeilings,
    pub ambient_environment_available: bool,
    pub network_available: bool,
    pub external_tools_available: bool,
    pub arbitrary_code_allowed: bool,
    pub production_state_write_available: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeTrainingExperimentSuiteRequest {
    expected_admission_review_id: String,
    expected_admission_review_sha256: String,
    expected_copy_output_validation_id: String,
    expected_copy_output_validation_sha256: String,
    expected_copy_id: String,
    expected_training_store_dataset_sha256: String,
    expected_recomputed_rows_sha256: String,
    expected_recomputed_excluded_rows_sha256: String,
    expected_recomputed_target_commitments_sha256: String,
    experiment_name: String,
    research_hypothesis: String,
    known_limitations: String,
    exact_current_stage_50_admission_and_complete_chain_confirmed: bool,
    claim_first_create_once_and_failure_consumes_confirmed: bool,
    fixed_three_arm_three_seed_suite_confirmed: bool,
    train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: bool,
    exact_65_feature_nine_raw_target_contract_confirmed: bool,
    no_scalar_reward_action_position_or_ranking_semantics_confirmed: bool,
    independent_registration_review_required_before_training_authorization_confirmed: bool,
    no_training_run_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationClaim {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_id: String,
    pub claim_sha256: String,
    pub attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub copy_output_validation_id: String,
    pub copy_output_validation_sha256: String,
    pub copy_id: String,
    pub training_store_dataset_sha256: String,
    pub source_official_joined_dataset_sha256: String,
    pub source_dataset_id: String,
    pub source_dataset_content_sha256: String,
    pub source_dataset_manifest_sha256: String,
    pub source_candidate_set_sha256: String,
    pub expected_rows_sha256: String,
    pub expected_excluded_rows_sha256: String,
    pub expected_target_commitments_sha256: String,
    pub expected_dataset_entry_count: usize,
    pub expected_active_row_count: usize,
    pub expected_excluded_row_count: usize,
    pub expected_feature_catalog_count: usize,
    pub expected_target_count: usize,
    pub expected_train_target_vector_count: usize,
    pub expected_validation_target_withheld_count: usize,
    pub expected_sealed_holdout_target_withheld_count: usize,
    pub experiment_name: String,
    pub research_hypothesis: String,
    pub known_limitations: String,
    pub suite_specification_sha256: String,
    pub registered_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub claimed_at: DateTime<Utc>,
    pub exact_current_stage_50_admission_and_complete_chain_confirmed: bool,
    pub registrar_independent_from_complete_prior_chain: bool,
    pub claim_first_create_once_and_failure_consumes_confirmed: bool,
    pub fixed_three_arm_three_seed_suite_confirmed: bool,
    pub train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: bool,
    pub exact_65_feature_nine_raw_target_contract_confirmed: bool,
    pub no_scalar_reward_action_position_or_ranking_semantics_confirmed: bool,
    pub independent_registration_review_required_before_training_authorization_confirmed: bool,
    pub no_training_run_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub claim_consumed: bool,
    pub exact_registration_directory_write_allowed: bool,
    pub generic_training_store_read_allowed: bool,
    pub generic_training_store_write_allowed: bool,
    pub training_run_allowed: bool,
    pub reward_allowed: bool,
    pub shadow_portfolio_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub claim_sha256: String,
    pub attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub copy_output_validation_id: String,
    pub copy_output_validation_sha256: String,
    pub copy_id: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
    pub dataset_entry_count: usize,
    pub active_row_count: usize,
    pub excluded_row_count: usize,
    pub experiment_name: String,
    pub research_hypothesis: String,
    pub known_limitations: String,
    pub suite_specification: HistoricalOutcomeTrainingExperimentSuiteSpecification,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub status: String,
    pub training_experiment_registered: bool,
    pub independently_reviewed_after_registration: bool,
    pub future_independent_registration_review_eligible: bool,
    pub runner_registered: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeTrainingExperimentRegistrationStatus {
    CompletedPendingIndependentReview,
    FailedRegistration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationResult {
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub registration_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub status: HistoricalOutcomeTrainingExperimentRegistrationStatus,
    pub error: Option<String>,
    pub registration_sha256: Option<String>,
    pub training_experiment_registered: bool,
    pub independent_registration_review_completed: bool,
    pub future_independent_registration_review_eligible: bool,
    pub runner_registered: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationAttempt {
    pub claim: HistoricalOutcomeTrainingExperimentRegistrationClaim,
    pub result: Option<HistoricalOutcomeTrainingExperimentRegistrationResult>,
    pub registration: Option<HistoricalOutcomeTrainingExperimentRegistrationRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationItem {
    pub admitted_dataset:
        AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
    pub attempt: Option<HistoricalOutcomeTrainingExperimentRegistrationAttempt>,
    pub registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub suite_version: String,
    pub items: Vec<HistoricalOutcomeTrainingExperimentRegistrationItem>,
    pub admitted_candidate_count: usize,
    pub registration_eligible_count: usize,
    pub claim_count: usize,
    pub completed_registration_count: usize,
    pub failed_or_incomplete_registration_count: usize,
    pub pending_independent_registration_review_count: usize,
    pub registration_status: String,
    pub create_once_registration_available: bool,
    pub training_experiment_registered: bool,
    pub independent_registration_review_completed: bool,
    pub runner_registered: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationReadinessSummary {
    pub admitted_candidate_count: usize,
    pub claim_count: usize,
    pub completed_registration_count: usize,
    pub failed_or_incomplete_registration_count: usize,
    pub pending_independent_registration_review_count: usize,
    pub registration_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview {
    pub admitted_dataset:
        AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
    pub attempt: HistoricalOutcomeTrainingExperimentRegistrationAttempt,
}

pub(crate) async fn handle_get_historical_outcome_training_experiment_registrations(
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
            warn!(%error, "training experiment registration registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "训练实验登记注册表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_training_experiment_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<RegisterHistoricalOutcomeTrainingExperimentSuiteRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_training_experiment_registration_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeTrainingExperimentRegistrationReadinessSummary {
            admitted_candidate_count: registry.admitted_candidate_count,
            claim_count: registry.claim_count,
            completed_registration_count: registry.completed_registration_count,
            failed_or_incomplete_registration_count: registry
                .failed_or_incomplete_registration_count,
            pending_independent_registration_review_count: registry
                .pending_independent_registration_review_count,
            registration_status: registry.registration_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn registered_training_experiment_suites_for_independent_review(
    state: &AppState,
) -> Result<Vec<RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview>, String> {
    let admitted = admitted_training_store_datasets_for_training_registration(state).await?;
    let mut output = Vec::new();
    for admitted_dataset in admitted {
        let Some(attempt) = read_attempt(state, &admitted_dataset).await? else {
            continue;
        };
        if registration_completed(&attempt) {
            output.push(
                RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview {
                    admitted_dataset,
                    attempt,
                },
            );
        }
    }
    output.sort_by(|left, right| {
        right
            .attempt
            .claim
            .claimed_at
            .cmp(&left.attempt.claim.claimed_at)
    });
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationRegistry, String> {
    let admitted = admitted_training_store_datasets_for_training_registration(state).await?;
    let mut items = Vec::with_capacity(admitted.len());
    for admitted_dataset in admitted {
        let attempt = read_attempt(state, &admitted_dataset).await?;
        items.push(HistoricalOutcomeTrainingExperimentRegistrationItem {
            registration_eligible: attempt.is_none(),
            admitted_dataset,
            attempt,
        });
    }
    let admitted_candidate_count = items.len();
    let registration_eligible_count = items
        .iter()
        .filter(|item| item.registration_eligible)
        .count();
    let claim_count = items.iter().filter(|item| item.attempt.is_some()).count();
    let completed_registration_count = items
        .iter()
        .filter(|item| item.attempt.as_ref().is_some_and(registration_completed))
        .count();
    let failed_or_incomplete_registration_count = items
        .iter()
        .filter(|item| {
            item.attempt
                .as_ref()
                .is_some_and(|attempt| !registration_completed(attempt))
        })
        .count();
    let pending_independent_registration_review_count = completed_registration_count;
    let registration_status = if completed_registration_count > 0 {
        "training_experiment_registered_not_run_waiting_independent_review"
    } else if failed_or_incomplete_registration_count > 0 {
        "training_experiment_registration_failed_or_incomplete_claim_consumed"
    } else if admitted_candidate_count > 0 {
        "waiting_claim_first_create_once_training_experiment_registration"
    } else {
        "waiting_stage_50_training_registration_admission"
    };
    Ok(HistoricalOutcomeTrainingExperimentRegistrationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        suite_version: SUITE_VERSION.to_string(),
        items,
        admitted_candidate_count,
        registration_eligible_count,
        claim_count,
        completed_registration_count,
        failed_or_incomplete_registration_count,
        pending_independent_registration_review_count,
        registration_status: registration_status.to_string(),
        create_once_registration_available: true,
        training_experiment_registered: completed_registration_count > 0,
        independent_registration_review_completed: false,
        runner_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 51 阶段只允许完整上游之外的新管理员，对一条精确 Stage 50 已准入训练存储副本先 claim，再 create-once 登记固定三模型臂、三随机种子、九项目标和静态资源上限的离线实验套件。登记状态固定为 registered_not_run，仍须另一角色独立复核；本阶段不运行训练、不定义奖励、不影子、不下单、不接券商、不交易。".to_string(),
    })
}

fn registration_completed(
    attempt: &HistoricalOutcomeTrainingExperimentRegistrationAttempt,
) -> bool {
    attempt.result.as_ref().is_some_and(|result| {
        result.status
            == HistoricalOutcomeTrainingExperimentRegistrationStatus::CompletedPendingIndependentReview
            && result.training_experiment_registered
            && result.future_independent_registration_review_eligible
            && !result.independent_registration_review_completed
            && attempt.registration.is_some()
    })
}

async fn register_once(
    state: &AppState,
    registered_by: &str,
    attempt_id: &str,
    request: RegisterHistoricalOutcomeTrainingExperimentSuiteRequest,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationResult, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    let admitted = admitted_training_store_datasets_for_training_registration(state)
        .await?
        .into_iter()
        .find(|value| exact_request_binding(value, attempt_id, &request))
        .ok_or_else(|| "当前没有这条精确绑定且通过 Stage 50 准入的训练存储副本".to_string())?;
    if read_attempt(state, &admitted).await?.is_some() {
        return Err("这条训练存储副本已经领取过一次性训练实验登记 claim，不能重放".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&admitted);
    if !actor_is_independent(registered_by, &excluded_prior_actor_ids) {
        return Err(
            "训练实验登记者不能是 Stage 50 复核人、Stage 49 校验者、Stage 48 复制者或完整上游角色"
                .to_string(),
        );
    }
    let specification = fixed_suite_specification()?;
    let claim = new_claim(
        &admitted,
        registered_by,
        excluded_prior_actor_ids,
        &request,
        &specification,
        Utc::now(),
    )?;
    let directory = registration_directory(state, attempt_id);
    write_immutable_json(&directory.join("claim.json"), &claim).await?;

    let registration = new_registration(&admitted, &claim, specification)?;
    let path = directory.join(format!(
        "registration-{}.json",
        registration.registration_sha256
    ));
    let result = match write_immutable_json(&path, &registration).await {
        Ok(()) => new_success_result(&claim, &registration)?,
        Err(error) => new_failed_result(&claim, &format!("训练实验登记写入失败：{error}"))?,
    };
    write_immutable_json(&directory.join("result.json"), &result).await?;
    Ok(result)
}

fn validate_request(
    attempt_id: &str,
    request: &RegisterHistoricalOutcomeTrainingExperimentSuiteRequest,
) -> Result<(), String> {
    if !valid_id(attempt_id)
        || !valid_id(&request.expected_admission_review_id)
        || !valid_id(&request.expected_copy_output_validation_id)
        || !valid_id(&request.expected_copy_id)
        || !all_sha256(&[
            &request.expected_admission_review_sha256,
            &request.expected_copy_output_validation_sha256,
            &request.expected_training_store_dataset_sha256,
            &request.expected_recomputed_rows_sha256,
            &request.expected_recomputed_excluded_rows_sha256,
            &request.expected_recomputed_target_commitments_sha256,
        ])
    {
        return Err("训练实验登记请求中的 ID 或 SHA-256 无效".to_string());
    }
    bounded_required(&request.experiment_name, MAX_NAME_CHARS, "实验名称")?;
    bounded_required(
        &request.research_hypothesis,
        MAX_HYPOTHESIS_CHARS,
        "研究假设",
    )?;
    bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    if !registration_checks_confirmed(request) {
        return Err("必须确认当前准入链、一次性失败消费、固定对照套件、切分隔离、65/9 合同、无奖励语义、后续独立复核和零运行权限八项边界".to_string());
    }
    Ok(())
}

fn registration_checks_confirmed(
    request: &RegisterHistoricalOutcomeTrainingExperimentSuiteRequest,
) -> bool {
    request.exact_current_stage_50_admission_and_complete_chain_confirmed
        && request.claim_first_create_once_and_failure_consumes_confirmed
        && request.fixed_three_arm_three_seed_suite_confirmed
        && request.train_fit_validation_selection_and_sealed_holdout_isolation_confirmed
        && request.exact_65_feature_nine_raw_target_contract_confirmed
        && request.no_scalar_reward_action_position_or_ranking_semantics_confirmed
        && request.independent_registration_review_required_before_training_authorization_confirmed
        && request.no_training_run_reward_shadow_order_broker_or_trading_confirmed
}

fn exact_request_binding(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
    attempt_id: &str,
    request: &RegisterHistoricalOutcomeTrainingExperimentSuiteRequest,
) -> bool {
    let review = &admitted.admission_review;
    review.attempt_id == attempt_id
        && review.review_id == request.expected_admission_review_id
        && review.review_sha256 == request.expected_admission_review_sha256
        && review.copy_output_validation_id == request.expected_copy_output_validation_id
        && review.copy_output_validation_sha256 == request.expected_copy_output_validation_sha256
        && review.copy_id == request.expected_copy_id
        && review.training_store_dataset_sha256 == request.expected_training_store_dataset_sha256
        && review.recomputed_rows_sha256 == request.expected_recomputed_rows_sha256
        && review.recomputed_excluded_rows_sha256
            == request.expected_recomputed_excluded_rows_sha256
        && review.recomputed_target_commitments_sha256
            == request.expected_recomputed_target_commitments_sha256
}

fn fixed_suite_specification()
-> Result<HistoricalOutcomeTrainingExperimentSuiteSpecification, String> {
    let seeds = vec![17, 29, 43];
    let arms = vec![
        HistoricalOutcomeTrainingExperimentArm {
            algorithm: HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline,
            role: "non_trained_zero_prediction_baseline".to_string(),
            random_seeds: seeds.clone(),
            max_epochs_or_boosting_rounds: 0,
            learning_rate_micros: 0,
            l2_regularization_micros: 0,
            maximum_tree_depth: 0,
            deterministic_replay_required: true,
        },
        HistoricalOutcomeTrainingExperimentArm {
            algorithm: HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
            role: "interpretable_linear_baseline".to_string(),
            random_seeds: seeds.clone(),
            max_epochs_or_boosting_rounds: 1,
            learning_rate_micros: 0,
            l2_regularization_micros: 10_000,
            maximum_tree_depth: 0,
            deterministic_replay_required: true,
        },
        HistoricalOutcomeTrainingExperimentArm {
            algorithm:
                HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
            role: "nonlinear_tabular_candidate".to_string(),
            random_seeds: seeds,
            max_epochs_or_boosting_rounds: 400,
            learning_rate_micros: 50_000,
            l2_regularization_micros: 10_000,
            maximum_tree_depth: 4,
            deterministic_replay_required: true,
        },
    ];
    let mut specification = HistoricalOutcomeTrainingExperimentSuiteSpecification {
        schema_version: SPEC_SCHEMA_VERSION.to_string(),
        suite_version: SUITE_VERSION.to_string(),
        specification_sha256: String::new(),
        feature_catalog_count: EXPECTED_FEATURE_COUNT,
        target_count: EXPECTED_TARGET_COUNT,
        target_contract_version: TARGET_CONTRACT_VERSION.to_string(),
        target_vector_order: [20_u16, 60, 250]
            .into_iter()
            .flat_map(|horizon| {
                ["asset_return", "excess_return", "asset_max_drawdown"]
                    .into_iter()
                    .map(move |metric| format!("{metric}_{horizon}_continuous"))
            })
            .collect(),
        primary_supervised_target_id: "excess_return_250_continuous".to_string(),
        risk_target_id: "asset_max_drawdown_250_continuous".to_string(),
        arms,
        fit_split: "train".to_string(),
        model_selection_split: "validation".to_string(),
        sealed_holdout_split: "sealed_holdout".to_string(),
        feature_preprocessing_contract: "fit scaling statistics on train only; preserve explicit missingness; no imputation, future backfill or cross-split fitting".to_string(),
        objective_contract: "predict all nine raw continuous outcomes without collapsing them into a scalar reward, action, position or rank label".to_string(),
        model_selection_contract: "report every target and every seed separately; validation may select candidates only after model artifact freeze; no composite score may hide a failing target".to_string(),
        reported_metric_ids: vec![
            "mae_per_target_per_seed".to_string(),
            "spearman_per_target_per_seed".to_string(),
            "directional_accuracy_per_target_per_seed".to_string(),
            "calibration_slope_per_target_per_seed".to_string(),
        ],
        sealed_holdout_access_allowed: false,
        sealed_holdout_labels_visible_to_training_worker: false,
        scalar_reward_defined: false,
        action_position_or_ranking_semantics_defined: false,
        resource_ceilings: HistoricalOutcomeTrainingExperimentResourceCeilings {
            maximum_wall_clock_seconds: 3_600,
            maximum_memory_mib: 8_192,
            maximum_cpu_millicores: 4_000,
            maximum_process_count: 4,
            maximum_output_bytes: 256 * 1024 * 1024,
        },
        ambient_environment_available: false,
        network_available: false,
        external_tools_available: false,
        arbitrary_code_allowed: false,
        production_state_write_available: false,
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])?;
    validate_specification(&specification)?;
    Ok(specification)
}

fn validate_specification(
    specification: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
) -> Result<(), String> {
    let expected_targets = [20_u16, 60, 250]
        .into_iter()
        .flat_map(|horizon| {
            ["asset_return", "excess_return", "asset_max_drawdown"]
                .into_iter()
                .map(move |metric| format!("{metric}_{horizon}_continuous"))
        })
        .collect::<Vec<_>>();
    let expected_algorithms = [
        HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline,
        HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
        HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
    ];
    let fixed_arms = specification.arms.len() == 3
        && specification
            .arms
            .iter()
            .zip(expected_algorithms)
            .all(|(arm, algorithm)| {
                arm.algorithm == algorithm
                    && arm.random_seeds == [17, 29, 43]
                    && arm.deterministic_replay_required
            });
    let downstream_closed = !specification.sealed_holdout_access_allowed
        && !specification.sealed_holdout_labels_visible_to_training_worker
        && !specification.scalar_reward_defined
        && !specification.action_position_or_ranking_semantics_defined
        && !specification.ambient_environment_available
        && !specification.network_available
        && !specification.external_tools_available
        && !specification.arbitrary_code_allowed
        && !specification.production_state_write_available;
    if specification.schema_version != SPEC_SCHEMA_VERSION
        || specification.suite_version != SUITE_VERSION
        || !valid_sha256(&specification.specification_sha256)
        || specification.specification_sha256
            != fingerprint_without(specification, &["specification_sha256"])?
        || specification.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || specification.target_count != EXPECTED_TARGET_COUNT
        || specification.target_contract_version != TARGET_CONTRACT_VERSION
        || specification.target_vector_order != expected_targets
        || specification.primary_supervised_target_id != "excess_return_250_continuous"
        || specification.risk_target_id != "asset_max_drawdown_250_continuous"
        || !fixed_arms
        || specification.fit_split != "train"
        || specification.model_selection_split != "validation"
        || specification.sealed_holdout_split != "sealed_holdout"
        || specification
            .feature_preprocessing_contract
            .trim()
            .is_empty()
        || specification.objective_contract.trim().is_empty()
        || specification.model_selection_contract.trim().is_empty()
        || specification.reported_metric_ids.len() != 4
        || specification.resource_ceilings.maximum_wall_clock_seconds != 3_600
        || specification.resource_ceilings.maximum_memory_mib != 8_192
        || specification.resource_ceilings.maximum_cpu_millicores != 4_000
        || specification.resource_ceilings.maximum_process_count != 4
        || specification.resource_ceilings.maximum_output_bytes != 256 * 1024 * 1024
        || !downstream_closed
    {
        return Err(
            "training experiment suite specification is invalid, drifted or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn new_claim(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
    registered_by: &str,
    excluded_prior_actor_ids: Vec<String>,
    request: &RegisterHistoricalOutcomeTrainingExperimentSuiteRequest,
    specification: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationClaim, String> {
    let review = &admitted.admission_review;
    let copied = admitted
        .dataset
        .copied_dataset
        .attempt
        .training_store_dataset
        .as_ref()
        .ok_or_else(|| "训练存储副本缺失".to_string())?;
    let mut claim = HistoricalOutcomeTrainingExperimentRegistrationClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_id: String::new(),
        claim_sha256: String::new(),
        attempt_id: review.attempt_id.clone(),
        admission_review_id: review.review_id.clone(),
        admission_review_sha256: review.review_sha256.clone(),
        copy_output_validation_id: review.copy_output_validation_id.clone(),
        copy_output_validation_sha256: review.copy_output_validation_sha256.clone(),
        copy_id: review.copy_id.clone(),
        training_store_dataset_sha256: review.training_store_dataset_sha256.clone(),
        source_official_joined_dataset_sha256: review.official_joined_dataset_sha256.clone(),
        source_dataset_id: review.dataset_id.clone(),
        source_dataset_content_sha256: review.dataset_content_sha256.clone(),
        source_dataset_manifest_sha256: review.dataset_manifest_sha256.clone(),
        source_candidate_set_sha256: review.candidate_set_sha256.clone(),
        expected_rows_sha256: review.recomputed_rows_sha256.clone(),
        expected_excluded_rows_sha256: review.recomputed_excluded_rows_sha256.clone(),
        expected_target_commitments_sha256: review.recomputed_target_commitments_sha256.clone(),
        expected_dataset_entry_count: review.dataset_entry_count,
        expected_active_row_count: review.active_row_count,
        expected_excluded_row_count: review.excluded_purge_or_embargo_row_count,
        expected_feature_catalog_count: review.feature_catalog_count,
        expected_target_count: review.target_count,
        expected_train_target_vector_count: review.train_target_vector_count,
        expected_validation_target_withheld_count: review.validation_target_withheld_count,
        expected_sealed_holdout_target_withheld_count: review.sealed_holdout_target_withheld_count,
        experiment_name: bounded_required(&request.experiment_name, MAX_NAME_CHARS, "实验名称")?,
        research_hypothesis: bounded_required(
            &request.research_hypothesis,
            MAX_HYPOTHESIS_CHARS,
            "研究假设",
        )?,
        known_limitations: bounded_required(
            &request.known_limitations,
            MAX_LIMITATIONS_CHARS,
            "已知局限",
        )?,
        suite_specification_sha256: specification.specification_sha256.clone(),
        registered_by: registered_by.to_string(),
        excluded_prior_actor_ids,
        claimed_at,
        exact_current_stage_50_admission_and_complete_chain_confirmed: request
            .exact_current_stage_50_admission_and_complete_chain_confirmed,
        registrar_independent_from_complete_prior_chain: true,
        claim_first_create_once_and_failure_consumes_confirmed: request
            .claim_first_create_once_and_failure_consumes_confirmed,
        fixed_three_arm_three_seed_suite_confirmed: request
            .fixed_three_arm_three_seed_suite_confirmed,
        train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: request
            .train_fit_validation_selection_and_sealed_holdout_isolation_confirmed,
        exact_65_feature_nine_raw_target_contract_confirmed: request
            .exact_65_feature_nine_raw_target_contract_confirmed,
        no_scalar_reward_action_position_or_ranking_semantics_confirmed: request
            .no_scalar_reward_action_position_or_ranking_semantics_confirmed,
        independent_registration_review_required_before_training_authorization_confirmed: request
            .independent_registration_review_required_before_training_authorization_confirmed,
        no_training_run_reward_shadow_order_broker_or_trading_confirmed: request
            .no_training_run_reward_shadow_order_broker_or_trading_confirmed,
        claim_consumed: true,
        exact_registration_directory_write_allowed: true,
        generic_training_store_read_allowed: false,
        generic_training_store_write_allowed: false,
        training_run_allowed: false,
        reward_allowed: false,
        shadow_portfolio_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = fingerprint_without(&claim, &["registration_id", "claim_sha256"])?;
    claim.registration_id = claim.claim_sha256[..32].to_string();
    validate_claim(
        &claim,
        admitted,
        specification,
        copied.feature_catalog_count,
    )?;
    Ok(claim)
}

fn validate_claim(
    claim: &HistoricalOutcomeTrainingExperimentRegistrationClaim,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
    specification: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
    copied_feature_count: usize,
) -> Result<(), String> {
    let review = &admitted.admission_review;
    let independent = actor_is_independent(&claim.registered_by, &claim.excluded_prior_actor_ids);
    let capabilities_closed = !claim.generic_training_store_read_allowed
        && !claim.generic_training_store_write_allowed
        && !claim.training_run_allowed
        && !claim.reward_allowed
        && !claim.shadow_portfolio_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed;
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.policy_version != POLICY_VERSION
        || !valid_id(&claim.registration_id)
        || !valid_sha256(&claim.claim_sha256)
        || claim.registration_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != fingerprint_without(claim, &["registration_id", "claim_sha256"])?
        || claim.attempt_id != review.attempt_id
        || claim.admission_review_id != review.review_id
        || claim.admission_review_sha256 != review.review_sha256
        || claim.copy_output_validation_id != review.copy_output_validation_id
        || claim.copy_output_validation_sha256 != review.copy_output_validation_sha256
        || claim.copy_id != review.copy_id
        || claim.training_store_dataset_sha256 != review.training_store_dataset_sha256
        || claim.source_official_joined_dataset_sha256 != review.official_joined_dataset_sha256
        || claim.source_dataset_id != review.dataset_id
        || claim.source_dataset_content_sha256 != review.dataset_content_sha256
        || claim.source_dataset_manifest_sha256 != review.dataset_manifest_sha256
        || claim.source_candidate_set_sha256 != review.candidate_set_sha256
        || claim.expected_rows_sha256 != review.recomputed_rows_sha256
        || claim.expected_excluded_rows_sha256 != review.recomputed_excluded_rows_sha256
        || claim.expected_target_commitments_sha256 != review.recomputed_target_commitments_sha256
        || claim.expected_dataset_entry_count != review.dataset_entry_count
        || claim.expected_active_row_count != review.active_row_count
        || claim.expected_excluded_row_count != review.excluded_purge_or_embargo_row_count
        || claim.expected_feature_catalog_count != EXPECTED_FEATURE_COUNT
        || claim.expected_feature_catalog_count != copied_feature_count
        || claim.expected_target_count != EXPECTED_TARGET_COUNT
        || claim.expected_train_target_vector_count != review.train_target_vector_count
        || claim.expected_validation_target_withheld_count
            != review.validation_target_withheld_count
        || claim.expected_sealed_holdout_target_withheld_count
            != review.sealed_holdout_target_withheld_count
        || claim.experiment_name.is_empty()
        || claim.research_hypothesis.is_empty()
        || claim.known_limitations.is_empty()
        || claim.suite_specification_sha256 != specification.specification_sha256
        || claim.registered_by.trim().is_empty()
        || claim.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&claim.excluded_prior_actor_ids)
        || !independent
        || !claim.registrar_independent_from_complete_prior_chain
        || !claim.exact_current_stage_50_admission_and_complete_chain_confirmed
        || !claim.claim_first_create_once_and_failure_consumes_confirmed
        || !claim.fixed_three_arm_three_seed_suite_confirmed
        || !claim.train_fit_validation_selection_and_sealed_holdout_isolation_confirmed
        || !claim.exact_65_feature_nine_raw_target_contract_confirmed
        || !claim.no_scalar_reward_action_position_or_ranking_semantics_confirmed
        || !claim.independent_registration_review_required_before_training_authorization_confirmed
        || !claim.no_training_run_reward_shadow_order_broker_or_trading_confirmed
        || !claim.claim_consumed
        || !claim.exact_registration_directory_write_allowed
        || !review.training_registration_candidate_admitted
        || !review.future_create_once_training_registration_eligible
        || review.training_registered
        || review.training_authorized
        || review.training_started
        || !capabilities_closed
    {
        return Err(
            "training experiment registration claim is invalid, drifted or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn new_registration(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
    claim: &HistoricalOutcomeTrainingExperimentRegistrationClaim,
    specification: HistoricalOutcomeTrainingExperimentSuiteSpecification,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationRecord, String> {
    let mut registration = HistoricalOutcomeTrainingExperimentRegistrationRecord {
        schema_version: REGISTRATION_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_id: claim.registration_id.clone(),
        registration_sha256: String::new(),
        claim_sha256: claim.claim_sha256.clone(),
        attempt_id: claim.attempt_id.clone(),
        admission_review_id: claim.admission_review_id.clone(),
        admission_review_sha256: claim.admission_review_sha256.clone(),
        copy_output_validation_id: claim.copy_output_validation_id.clone(),
        copy_output_validation_sha256: claim.copy_output_validation_sha256.clone(),
        copy_id: claim.copy_id.clone(),
        training_store_dataset_sha256: claim.training_store_dataset_sha256.clone(),
        rows_sha256: claim.expected_rows_sha256.clone(),
        excluded_rows_sha256: claim.expected_excluded_rows_sha256.clone(),
        target_commitments_sha256: claim.expected_target_commitments_sha256.clone(),
        dataset_entry_count: claim.expected_dataset_entry_count,
        active_row_count: claim.expected_active_row_count,
        excluded_row_count: claim.expected_excluded_row_count,
        experiment_name: claim.experiment_name.clone(),
        research_hypothesis: claim.research_hypothesis.clone(),
        known_limitations: claim.known_limitations.clone(),
        suite_specification: specification,
        registered_at: claim.claimed_at,
        registered_by: claim.registered_by.clone(),
        excluded_prior_actor_ids: claim.excluded_prior_actor_ids.clone(),
        status: "registered_not_run".to_string(),
        training_experiment_registered: true,
        independently_reviewed_after_registration: false,
        future_independent_registration_review_eligible: true,
        runner_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    registration.registration_sha256 =
        fingerprint_without(&registration, &["registration_sha256"])?;
    validate_registration(&registration, claim)?;
    let copied_feature_count = admitted
        .dataset
        .copied_dataset
        .attempt
        .training_store_dataset
        .as_ref()
        .ok_or_else(|| "训练存储副本缺失".to_string())?
        .feature_catalog_count;
    validate_claim(
        claim,
        admitted,
        &registration.suite_specification,
        copied_feature_count,
    )?;
    Ok(registration)
}

fn validate_registration(
    registration: &HistoricalOutcomeTrainingExperimentRegistrationRecord,
    claim: &HistoricalOutcomeTrainingExperimentRegistrationClaim,
) -> Result<(), String> {
    validate_specification(&registration.suite_specification)?;
    let downstream_closed = !registration.independently_reviewed_after_registration
        && registration.future_independent_registration_review_eligible
        && !registration.runner_registered
        && !registration.training_authorized
        && !registration.training_started
        && !registration.reward_authorized
        && !registration.shadow_portfolio_authorized
        && !registration.order_generation_authorized
        && !registration.broker_access_authorized
        && !registration.trading_authorized;
    if registration.schema_version != REGISTRATION_SCHEMA_VERSION
        || registration.policy_version != POLICY_VERSION
        || registration.registration_id != claim.registration_id
        || !valid_sha256(&registration.registration_sha256)
        || registration.registration_sha256
            != fingerprint_without(registration, &["registration_sha256"])?
        || registration.claim_sha256 != claim.claim_sha256
        || registration.attempt_id != claim.attempt_id
        || registration.admission_review_id != claim.admission_review_id
        || registration.admission_review_sha256 != claim.admission_review_sha256
        || registration.copy_output_validation_id != claim.copy_output_validation_id
        || registration.copy_output_validation_sha256 != claim.copy_output_validation_sha256
        || registration.copy_id != claim.copy_id
        || registration.training_store_dataset_sha256 != claim.training_store_dataset_sha256
        || registration.rows_sha256 != claim.expected_rows_sha256
        || registration.excluded_rows_sha256 != claim.expected_excluded_rows_sha256
        || registration.target_commitments_sha256 != claim.expected_target_commitments_sha256
        || registration.dataset_entry_count != claim.expected_dataset_entry_count
        || registration.active_row_count != claim.expected_active_row_count
        || registration.excluded_row_count != claim.expected_excluded_row_count
        || registration.active_row_count + registration.excluded_row_count
            != registration.dataset_entry_count
        || registration.experiment_name != claim.experiment_name
        || registration.research_hypothesis != claim.research_hypothesis
        || registration.known_limitations != claim.known_limitations
        || registration.suite_specification.specification_sha256 != claim.suite_specification_sha256
        || registration.registered_at != claim.claimed_at
        || registration.registered_by != claim.registered_by
        || registration.excluded_prior_actor_ids != claim.excluded_prior_actor_ids
        || registration.status != "registered_not_run"
        || !registration.training_experiment_registered
        || !downstream_closed
    {
        return Err(
            "training experiment registration is invalid, drifted or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn new_success_result(
    claim: &HistoricalOutcomeTrainingExperimentRegistrationClaim,
    registration: &HistoricalOutcomeTrainingExperimentRegistrationRecord,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationResult, String> {
    let mut result = HistoricalOutcomeTrainingExperimentRegistrationResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        registration_id: claim.registration_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        status:
            HistoricalOutcomeTrainingExperimentRegistrationStatus::CompletedPendingIndependentReview,
        error: None,
        registration_sha256: Some(registration.registration_sha256.clone()),
        training_experiment_registered: true,
        independent_registration_review_completed: false,
        future_independent_registration_review_eligible: true,
        runner_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    finalize_result(&mut result)?;
    validate_result(&result, claim, Some(registration))?;
    Ok(result)
}

fn new_failed_result(
    claim: &HistoricalOutcomeTrainingExperimentRegistrationClaim,
    error: &str,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationResult, String> {
    let error = bounded_error(error);
    let mut result = HistoricalOutcomeTrainingExperimentRegistrationResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        registration_id: claim.registration_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        status: HistoricalOutcomeTrainingExperimentRegistrationStatus::FailedRegistration,
        error: Some(error),
        registration_sha256: None,
        training_experiment_registered: false,
        independent_registration_review_completed: false,
        future_independent_registration_review_eligible: false,
        runner_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    finalize_result(&mut result)?;
    validate_result(&result, claim, None)?;
    Ok(result)
}

fn finalize_result(
    result: &mut HistoricalOutcomeTrainingExperimentRegistrationResult,
) -> Result<(), String> {
    result.result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].to_string();
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeTrainingExperimentRegistrationResult,
    claim: &HistoricalOutcomeTrainingExperimentRegistrationClaim,
    registration: Option<&HistoricalOutcomeTrainingExperimentRegistrationRecord>,
) -> Result<(), String> {
    let success = result.status
        == HistoricalOutcomeTrainingExperimentRegistrationStatus::CompletedPendingIndependentReview;
    let success_fields = if success {
        let registration = registration
            .ok_or_else(|| "successful registration result misses registration".to_string())?;
        result.error.is_none()
            && result.registration_sha256.as_deref()
                == Some(registration.registration_sha256.as_str())
            && result.training_experiment_registered
            && result.future_independent_registration_review_eligible
    } else {
        registration.is_none()
            && result.error.as_ref().is_some_and(|value| !value.is_empty())
            && result.registration_sha256.is_none()
            && !result.training_experiment_registered
            && !result.future_independent_registration_review_eligible
    };
    let downstream_closed = !result.independent_registration_review_completed
        && !result.runner_registered
        && !result.training_authorized
        && !result.training_started
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.policy_version != POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != fingerprint_without(result, &["result_id", "result_sha256"])?
        || result.registration_id != claim.registration_id
        || result.claim_sha256 != claim.claim_sha256
        || !success_fields
        || !downstream_closed
    {
        return Err(
            "training experiment registration result is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

async fn read_attempt(
    state: &AppState,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
) -> Result<Option<HistoricalOutcomeTrainingExperimentRegistrationAttempt>, String> {
    let directory = registration_directory(state, &admitted.admission_review.attempt_id);
    let claim_bytes = match tokio::fs::read(directory.join("claim.json")).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let claim: HistoricalOutcomeTrainingExperimentRegistrationClaim =
        serde_json::from_slice(&claim_bytes).map_err(|error| error.to_string())?;
    let specification = fixed_suite_specification()?;
    let copied_feature_count = admitted
        .dataset
        .copied_dataset
        .attempt
        .training_store_dataset
        .as_ref()
        .ok_or_else(|| "训练存储副本缺失".to_string())?
        .feature_catalog_count;
    validate_claim(&claim, admitted, &specification, copied_feature_count)?;
    let result_bytes = match tokio::fs::read(directory.join("result.json")).await {
        Ok(value) => Some(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.to_string()),
    };
    let result: Option<HistoricalOutcomeTrainingExperimentRegistrationResult> = result_bytes
        .as_ref()
        .map(|bytes| serde_json::from_slice(bytes).map_err(|error| error.to_string()))
        .transpose()?;
    let registration = if let Some(result) = &result {
        match result.registration_sha256.as_deref() {
            Some(sha256) => {
                let bytes = tokio::fs::read(directory.join(format!("registration-{sha256}.json")))
                    .await
                    .map_err(|error| error.to_string())?;
                let registration: HistoricalOutcomeTrainingExperimentRegistrationRecord =
                    serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
                validate_registration(&registration, &claim)?;
                Some(registration)
            }
            None => None,
        }
    } else {
        None
    };
    if let Some(result) = &result {
        validate_result(result, &claim, registration.as_ref())?;
    }
    Ok(Some(
        HistoricalOutcomeTrainingExperimentRegistrationAttempt {
            claim,
            result,
            registration,
        },
    ))
}

fn excluded_actor_ids(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDatasetForTrainingRegistration,
) -> Vec<String> {
    let mut values = admitted.admission_review.excluded_prior_actor_ids.clone();
    values.push(admitted.admission_review.reviewer_id.clone());
    values.push(admitted.dataset.validation.validated_by.clone());
    values.push(admitted.dataset.validation.copied_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded_prior_actor_ids: &[String]) -> bool {
    !actor_id.trim().is_empty()
        && excluded_prior_actor_ids
            .iter()
            .all(|value| value != actor_id)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let map = json
        .as_object_mut()
        .ok_or_else(|| "fingerprinted value must be an object".to_string())?;
    for field in fields {
        map.remove(*field);
    }
    let bytes = serde_json::to_vec(&json).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > max_chars || value.chars().any(char::is_control) {
        return Err(format!("{label}格式无效或超过 {max_chars} 个字符"));
    }
    Ok(value.to_string())
}

fn bounded_error(value: &str) -> String {
    value.trim().chars().take(MAX_ERROR_CHARS).collect()
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn all_sha256(values: &[&str]) -> bool {
    values.iter().all(|value| valid_sha256(value))
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn registration_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("isolated-training-store")
        .join("training-experiment-registrations")
        .join(attempt_id)
}

struct RegistrationLock {
    path: PathBuf,
}

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<RegistrationLock, String> {
    let directory = registration_directory(state, attempt_id);
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(".registration.lock");
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
        .map_err(|_| "已有同一训练实验登记正在进行".to_string())?;
    Ok(RegistrationLock { path })
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

    fn request() -> RegisterHistoricalOutcomeTrainingExperimentSuiteRequest {
        RegisterHistoricalOutcomeTrainingExperimentSuiteRequest {
            expected_admission_review_id: "1".repeat(32),
            expected_admission_review_sha256: "2".repeat(64),
            expected_copy_output_validation_id: "3".repeat(32),
            expected_copy_output_validation_sha256: "4".repeat(64),
            expected_copy_id: "5".repeat(32),
            expected_training_store_dataset_sha256: "6".repeat(64),
            expected_recomputed_rows_sha256: "7".repeat(64),
            expected_recomputed_excluded_rows_sha256: "8".repeat(64),
            expected_recomputed_target_commitments_sha256: "9".repeat(64),
            experiment_name: "三模型九目标首轮对照".to_string(),
            research_hypothesis: "检验点时特征对九项连续结果是否具有跨种子稳定的样本外解释力"
                .to_string(),
            known_limitations: "只登记工程实验；样本规模、选择偏差和经济可交易性仍未验证"
                .to_string(),
            exact_current_stage_50_admission_and_complete_chain_confirmed: true,
            claim_first_create_once_and_failure_consumes_confirmed: true,
            fixed_three_arm_three_seed_suite_confirmed: true,
            train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: true,
            exact_65_feature_nine_raw_target_contract_confirmed: true,
            no_scalar_reward_action_position_or_ranking_semantics_confirmed: true,
            independent_registration_review_required_before_training_authorization_confirmed: true,
            no_training_run_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    fn claim() -> HistoricalOutcomeTrainingExperimentRegistrationClaim {
        let mut claim = HistoricalOutcomeTrainingExperimentRegistrationClaim {
            schema_version: CLAIM_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            registration_id: String::new(),
            claim_sha256: String::new(),
            attempt_id: "a".repeat(32),
            admission_review_id: "b".repeat(32),
            admission_review_sha256: "c".repeat(64),
            copy_output_validation_id: "d".repeat(32),
            copy_output_validation_sha256: "e".repeat(64),
            copy_id: "f".repeat(32),
            training_store_dataset_sha256: "1".repeat(64),
            source_official_joined_dataset_sha256: "2".repeat(64),
            source_dataset_id: "3".repeat(32),
            source_dataset_content_sha256: "4".repeat(64),
            source_dataset_manifest_sha256: "5".repeat(64),
            source_candidate_set_sha256: "6".repeat(64),
            expected_rows_sha256: "7".repeat(64),
            expected_excluded_rows_sha256: "8".repeat(64),
            expected_target_commitments_sha256: "9".repeat(64),
            expected_dataset_entry_count: 6,
            expected_active_row_count: 3,
            expected_excluded_row_count: 3,
            expected_feature_catalog_count: 65,
            expected_target_count: 9,
            expected_train_target_vector_count: 1,
            expected_validation_target_withheld_count: 1,
            expected_sealed_holdout_target_withheld_count: 1,
            experiment_name: "首轮对照".into(),
            research_hypothesis: "检验稳定性".into(),
            known_limitations: "只登记不训练".into(),
            suite_specification_sha256: fixed_suite_specification().unwrap().specification_sha256,
            registered_by: "registrar".into(),
            excluded_prior_actor_ids: vec!["reviewer".into(), "validator".into()],
            claimed_at: Utc::now(),
            exact_current_stage_50_admission_and_complete_chain_confirmed: true,
            registrar_independent_from_complete_prior_chain: true,
            claim_first_create_once_and_failure_consumes_confirmed: true,
            fixed_three_arm_three_seed_suite_confirmed: true,
            train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: true,
            exact_65_feature_nine_raw_target_contract_confirmed: true,
            no_scalar_reward_action_position_or_ranking_semantics_confirmed: true,
            independent_registration_review_required_before_training_authorization_confirmed: true,
            no_training_run_reward_shadow_order_broker_or_trading_confirmed: true,
            claim_consumed: true,
            exact_registration_directory_write_allowed: true,
            generic_training_store_read_allowed: false,
            generic_training_store_write_allowed: false,
            training_run_allowed: false,
            reward_allowed: false,
            shadow_portfolio_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
        claim.claim_sha256 =
            fingerprint_without(&claim, &["registration_id", "claim_sha256"]).unwrap();
        claim.registration_id = claim.claim_sha256[..32].to_string();
        claim
    }

    fn registration_for(
        claim: &HistoricalOutcomeTrainingExperimentRegistrationClaim,
    ) -> HistoricalOutcomeTrainingExperimentRegistrationRecord {
        let mut registration = HistoricalOutcomeTrainingExperimentRegistrationRecord {
            schema_version: REGISTRATION_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            registration_id: claim.registration_id.clone(),
            registration_sha256: String::new(),
            claim_sha256: claim.claim_sha256.clone(),
            attempt_id: claim.attempt_id.clone(),
            admission_review_id: claim.admission_review_id.clone(),
            admission_review_sha256: claim.admission_review_sha256.clone(),
            copy_output_validation_id: claim.copy_output_validation_id.clone(),
            copy_output_validation_sha256: claim.copy_output_validation_sha256.clone(),
            copy_id: claim.copy_id.clone(),
            training_store_dataset_sha256: claim.training_store_dataset_sha256.clone(),
            rows_sha256: claim.expected_rows_sha256.clone(),
            excluded_rows_sha256: claim.expected_excluded_rows_sha256.clone(),
            target_commitments_sha256: claim.expected_target_commitments_sha256.clone(),
            dataset_entry_count: 6,
            active_row_count: 3,
            excluded_row_count: 3,
            experiment_name: claim.experiment_name.clone(),
            research_hypothesis: claim.research_hypothesis.clone(),
            known_limitations: claim.known_limitations.clone(),
            suite_specification: fixed_suite_specification().unwrap(),
            registered_at: claim.claimed_at,
            registered_by: claim.registered_by.clone(),
            excluded_prior_actor_ids: claim.excluded_prior_actor_ids.clone(),
            status: "registered_not_run".into(),
            training_experiment_registered: true,
            independently_reviewed_after_registration: false,
            future_independent_registration_review_eligible: true,
            runner_registered: false,
            training_authorized: false,
            training_started: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        registration.registration_sha256 =
            fingerprint_without(&registration, &["registration_sha256"]).unwrap();
        registration
    }

    fn registration() -> HistoricalOutcomeTrainingExperimentRegistrationRecord {
        registration_for(&claim())
    }

    #[test]
    fn fixed_suite_has_three_arms_and_three_deterministic_seeds() {
        let suite = fixed_suite_specification().unwrap();
        assert_eq!(suite.arms.len(), 3);
        assert!(
            suite
                .arms
                .iter()
                .all(|arm| arm.random_seeds == [17, 29, 43])
        );
        assert!(
            suite
                .arms
                .iter()
                .all(|arm| arm.deterministic_replay_required)
        );
    }

    #[test]
    fn suite_preserves_nine_raw_targets_and_holdout_seal() {
        let suite = fixed_suite_specification().unwrap();
        assert_eq!(suite.target_vector_order.len(), 9);
        assert!(!suite.sealed_holdout_access_allowed);
        assert!(!suite.sealed_holdout_labels_visible_to_training_worker);
        assert!(!suite.scalar_reward_defined);
        assert!(!suite.action_position_or_ranking_semantics_defined);
    }

    #[test]
    fn request_requires_all_eight_registration_boundaries() {
        let mut value = request();
        assert!(registration_checks_confirmed(&value));
        value.independent_registration_review_required_before_training_authorization_confirmed =
            false;
        assert!(!registration_checks_confirmed(&value));
    }

    #[test]
    fn actor_must_be_independent_from_stage_50_and_complete_chain() {
        let actors = vec!["reviewer".to_string(), "validator".to_string()];
        assert!(actor_is_independent("registrar", &actors));
        assert!(!actor_is_independent("reviewer", &actors));
    }

    #[test]
    fn claim_hash_binds_dataset_suite_and_permissions() {
        let value = claim();
        let first = value.claim_sha256.clone();
        let mut changed = value;
        changed.training_store_dataset_sha256 = "a".repeat(64);
        changed.training_run_allowed = true;
        assert_ne!(
            first,
            fingerprint_without(&changed, &["registration_id", "claim_sha256"]).unwrap()
        );
    }

    #[test]
    fn registration_is_registered_not_run_and_review_gated() {
        let claim = claim();
        let value = registration_for(&claim);
        validate_registration(&value, &claim).unwrap();
        assert_eq!(value.status, "registered_not_run");
        assert!(value.future_independent_registration_review_eligible);
        assert!(!value.runner_registered);
        assert!(!value.training_authorized);
        assert!(!value.training_started);
    }

    #[test]
    fn registration_hash_binds_hypothesis_and_target_contract() {
        let value = registration();
        let first = value.registration_sha256.clone();
        let mut changed = value;
        changed.research_hypothesis = "另一假设".into();
        changed.suite_specification.scalar_reward_defined = true;
        assert_ne!(
            first,
            fingerprint_without(&changed, &["registration_sha256"]).unwrap()
        );
    }

    #[test]
    fn successful_result_never_authorizes_training_or_trading() {
        let claim = claim();
        let registration = registration_for(&claim);
        let result = new_success_result(&claim, &registration).unwrap();
        assert!(result.training_experiment_registered);
        assert!(result.future_independent_registration_review_eligible);
        assert!(!result.training_authorized);
        assert!(!result.training_started);
        assert!(!result.reward_authorized);
        assert!(!result.trading_authorized);
    }

    #[test]
    fn failed_result_consumes_claim_without_opening_review_or_run() {
        let claim = claim();
        let result = new_failed_result(&claim, "write failed").unwrap();
        assert_eq!(
            result.status,
            HistoricalOutcomeTrainingExperimentRegistrationStatus::FailedRegistration
        );
        assert!(!result.training_experiment_registered);
        assert!(!result.future_independent_registration_review_eligible);
        assert!(!result.training_started);
    }

    #[test]
    fn bounded_text_and_actor_sets_fail_closed() {
        assert_eq!(bounded_required("  实验  ", 8, "名称").unwrap(), "实验");
        assert!(bounded_required("   ", 8, "名称").is_err());
        assert!(sorted_unique(&["a".into(), "b".into()]));
        assert!(!sorted_unique(&["b".into(), "a".into()]));
    }
}
