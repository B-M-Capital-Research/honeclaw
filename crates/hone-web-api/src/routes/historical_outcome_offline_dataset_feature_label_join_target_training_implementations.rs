//! Immutable registration of a future isolated Stage 51 training-suite implementation.
//!
//! Stage 53 freezes code and a declarative implementation contract only after one exact Stage 52
//! approval. It exposes no callable entrypoint, cannot read training data, fit a model, select on
//! validation, open the sealed holdout, define a reward, create a shadow portfolio, generate an
//! order, access a broker or trade. Independent implementation review is the only next gate.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registration_reviews::{
    HistoricalOutcomeTrainingExperimentRegistrationReview,
    IndependentlyReviewedHistoricalOutcomeTrainingExperimentRegistration,
    independently_reviewed_training_experiment_registrations_for_implementation_registration,
    validate_historical_outcome_training_experiment_registration_review_for_implementation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-training-implementation-registry-v1";
const IMPLEMENTATION_SCHEMA_VERSION: &str = "hone-historical-outcome-training-implementation-v1";
const POLICY_VERSION: &str = "hone-training-implementation-registration-v1-no-execution";
const CONTRACT_SCHEMA_VERSION: &str = "hone-isolated-training-implementation-contract-v1";
const SUITE_VERSION: &str = "hone-nine-target-comparison-suite-v1";
const TARGET_CONTRACT_VERSION: &str = "hone-nine-raw-continuous-outcomes-v1";
const INPUT_SCHEMA_VERSION: &str = "hone-reviewed-training-store-dataset-v1";
const OUTPUT_SCHEMA_VERSION: &str = "hone-unrun-training-artifact-envelope-v1";
const CANONICAL_SERIALIZER_VERSION: &str = "hone-canonical-json-serializer-v1";
const PREPROCESSOR_IMPLEMENTATION_VERSION: &str =
    "v1-train-only-scaling-explicit-missingness-no-imputation";
const STATUS: &str = "registered_not_reviewed_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeTrainingImplementationRequest {
    expected_review_id: String,
    expected_review_sha256: String,
    expected_attempt_id: String,
    expected_registration_id: String,
    expected_registration_sha256: String,
    expected_claim_sha256: String,
    expected_result_id: String,
    expected_result_sha256: String,
    expected_suite_specification_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    exact_stage_52_review_and_stage_51_registration_binding_confirmed: bool,
    registrar_independent_from_complete_prior_chain_confirmed: bool,
    immutable_artifact_and_code_revision_confirmed: bool,
    fixed_three_arm_three_seed_implementation_confirmed: bool,
    exact_65_feature_nine_raw_continuous_target_contract_confirmed: bool,
    train_only_preprocessing_and_fit_confirmed: bool,
    validation_selection_and_sealed_holdout_isolation_confirmed: bool,
    per_target_per_seed_metrics_without_composite_masking_confirmed: bool,
    deterministic_replay_and_fixed_resource_ceilings_confirmed: bool,
    no_scalar_reward_action_position_or_ranking_semantics_confirmed: bool,
    implementation_review_runner_and_run_authorization_separation_confirmed: bool,
    no_data_access_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_artifact_sha256: String,
    pub immutable_code_revision: String,
    pub suite_version: String,
    pub suite_specification_sha256: String,
    pub target_contract_version: String,
    pub input_schema_version: String,
    pub output_schema_version: String,
    pub canonical_serializer_version: String,
    pub preprocessor_implementation_version: String,
    pub algorithm_implementation_versions: Vec<String>,
    pub exact_feature_count: u16,
    pub exact_target_count: u8,
    pub exact_random_seeds: Vec<u64>,
    pub reported_metric_ids: Vec<String>,
    pub maximum_wall_clock_seconds: u32,
    pub maximum_memory_mib: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u16,
    pub maximum_output_bytes: u64,
    pub input_contract: String,
    pub output_contract: String,
    pub callable_entrypoint_present: bool,
    pub ambient_environment_available: bool,
    pub environment_variables_allowed: bool,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub external_tools_allowed: bool,
    pub child_process_allowed: bool,
    pub training_store_reads_allowed: bool,
    pub validation_labels_visible_to_fit_worker: bool,
    pub sealed_holdout_labels_visible_to_fit_or_selection_worker: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub scalar_reward_defined: bool,
    pub action_position_or_ranking_semantics_defined: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub approved_registration_review: HistoricalOutcomeTrainingExperimentRegistrationReview,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_complete_prior_chain: bool,
    pub implementation_name: String,
    pub rationale: String,
    pub known_limitations: String,
    pub implementation_contract: HistoricalOutcomeTrainingImplementationContract,
    pub status: String,
    pub exact_stage_52_review_and_stage_51_registration_binding_confirmed: bool,
    pub registrar_independent_from_complete_prior_chain_confirmed: bool,
    pub immutable_artifact_and_code_revision_confirmed: bool,
    pub fixed_three_arm_three_seed_implementation_confirmed: bool,
    pub exact_65_feature_nine_raw_continuous_target_contract_confirmed: bool,
    pub train_only_preprocessing_and_fit_confirmed: bool,
    pub validation_selection_and_sealed_holdout_isolation_confirmed: bool,
    pub per_target_per_seed_metrics_without_composite_masking_confirmed: bool,
    pub deterministic_replay_and_fixed_resource_ceilings_confirmed: bool,
    pub no_scalar_reward_action_position_or_ranking_semantics_confirmed: bool,
    pub implementation_review_runner_and_run_authorization_separation_confirmed: bool,
    pub no_data_access_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub training_implementation_registered: bool,
    pub future_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub isolated_runner_registration_eligible: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub validation_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub model_artifact_created: bool,
    pub metrics_created: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingImplementationItem {
    pub implementation: HistoricalOutcomeTrainingImplementationRecord,
    pub upstream_binding_current: bool,
    pub future_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_reviews: Vec<HistoricalOutcomeTrainingExperimentRegistrationReview>,
    pub items: Vec<HistoricalOutcomeTrainingImplementationItem>,
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub runner_registered: bool,
    pub training_data_access_authorized: bool,
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
pub(crate) struct HistoricalOutcomeTrainingImplementationReadinessSummary {
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_historical_outcome_training_implementations(
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
            warn!(%error, "training implementation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "训练实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_training_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeTrainingImplementationRequest>,
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

pub(crate) async fn historical_outcome_training_implementation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingImplementationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeTrainingImplementationReadinessSummary {
        registration_eligible_count: registry.registration_eligible_count,
        implementation_count: registry.implementation_count,
        current_binding_implementation_count: registry.current_binding_implementation_count,
        independent_implementation_review_eligible_count: registry
            .independent_implementation_review_eligible_count,
        implementation_status: registry.implementation_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn independent_review_eligible_historical_outcome_training_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeTrainingImplementationRecord>, String> {
    let registry = build_registry(state).await?;
    let mut records = registry
        .items
        .into_iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .map(|item| item.implementation)
        .collect::<Vec<_>>();
    for record in &records {
        validate_record(record)?;
    }
    records.sort_by(|left, right| {
        right
            .registered_at
            .cmp(&left.registered_at)
            .then_with(|| right.implementation_id.cmp(&left.implementation_id))
    });
    Ok(records)
}

#[allow(dead_code)]
pub(crate) fn validate_historical_outcome_training_implementation_for_review(
    record: &HistoricalOutcomeTrainingImplementationRecord,
) -> Result<(), String> {
    validate_record(record)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingImplementationRegistry, String> {
    let eligible =
        independently_reviewed_training_experiment_registrations_for_implementation_registration(
            state,
        )
        .await?;
    let eligible_by_review = eligible
        .iter()
        .map(|value| (value.review.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let mut items = read_records(state)
        .await?
        .into_iter()
        .map(|implementation| {
            let upstream_binding_current = eligible_by_review
                .get(
                    implementation
                        .approved_registration_review
                        .review_id
                        .as_str(),
                )
                .is_some_and(|upstream| {
                    implementation.approved_registration_review == upstream.review
                });
            HistoricalOutcomeTrainingImplementationItem {
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
    let registered_review_ids = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .map(|item| {
            item.implementation
                .approved_registration_review
                .review_id
                .as_str()
        })
        .collect::<BTreeSet<_>>();
    let registration_eligible_count = eligible
        .iter()
        .filter(|value| !registered_review_ids.contains(value.review.review_id.as_str()))
        .count();
    let implementation_count = items.len();
    let current_binding_implementation_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .count();
    let implementation_status = if current_binding_implementation_count > 0 {
        "current_training_implementation_registered_not_run_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "approved_training_experiment_registration_ready_for_implementation_registration"
    } else {
        "waiting_stage_52_independently_approved_training_experiment_registration"
    };
    Ok(HistoricalOutcomeTrainingImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_reviews: eligible.into_iter().map(|value| value.review).collect(),
        items,
        registration_eligible_count,
        implementation_count,
        current_binding_implementation_count,
        independent_implementation_review_eligible_count,
        implementation_status: implementation_status.to_string(),
        runner_registered: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 53 只允许完整上游之外的新管理员，为一条精确 Stage 52 独立批准登记冻结训练实现工件、代码版本、三模型臂、三随机种子、65 项特征、九项原始连续目标、逐目标逐种子指标和零能力边界。登记状态固定为 registered_not_reviewed_not_run；下一步只能独立复核实现，不创建 runner、不读取训练数据、不训练、不定义奖励、不影子、不下单、不接券商、不交易。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeTrainingImplementationRequest,
) -> Result<HistoricalOutcomeTrainingImplementationRecord, String> {
    if registrar_id.trim().is_empty() {
        return Err("训练实现登记人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let eligible =
        independently_reviewed_training_experiment_registrations_for_implementation_registration(
            state,
        )
        .await?;
    let upstream = eligible
        .iter()
        .find(|value| value.review.review_id == request.expected_review_id)
        .ok_or_else(|| "当前 Stage 52 独立批准记录不存在，请刷新后重试".to_string())?;
    ensure_review_unregistered(&read_records(state).await?, &upstream.review)?;
    let record = build_record(upstream, registrar_id, request, Utc::now())?;
    write_immutable_json(
        &record_root(state).join(format!("{}.json", record.implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn build_record(
    upstream: &IndependentlyReviewedHistoricalOutcomeTrainingExperimentRegistration,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeTrainingImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeTrainingImplementationRecord, String> {
    let review = &upstream.review;
    validate_historical_outcome_training_experiment_registration_review_for_implementation(review)?;
    let registration = upstream
        .registered_experiment
        .attempt
        .registration
        .as_ref()
        .ok_or_else(|| "Stage 51 registration is missing".to_string())?;
    let result = upstream
        .registered_experiment
        .attempt
        .result
        .as_ref()
        .ok_or_else(|| "Stage 51 registration result is missing".to_string())?;
    if request.expected_review_id != review.review_id
        || request.expected_review_sha256 != review.review_sha256
        || request.expected_attempt_id != review.attempt_id
        || request.expected_registration_id != review.registration_id
        || request.expected_registration_sha256 != review.registration_sha256
        || request.expected_claim_sha256 != review.claim_sha256
        || request.expected_result_id != review.result_id
        || request.expected_result_sha256 != review.result_sha256
        || request.expected_suite_specification_sha256 != review.suite_specification_sha256
        || registration.registration_sha256 != review.registration_sha256
        || result.result_sha256 != review.result_sha256
    {
        return Err("训练实现绑定的 Stage 51–52 链已变化，请刷新后重试".to_string());
    }
    let implementation_name =
        bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    let immutable_code_revision = bounded_required(
        &request.immutable_code_revision,
        MAX_REVISION_CHARS,
        "不可变代码版本",
    )?;
    if !valid_sha256(&request.implementation_artifact_sha256) {
        return Err("训练实现工件 SHA-256 无效".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "登记理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(review);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == registrar_id)
    {
        return Err("训练实现登记人参与过 Stage 51–52 或完整上游，必须更换独立角色".to_string());
    }
    if !request_checks_confirmed(&request) {
        return Err("登记训练实现前必须逐项确认绑定、复现、隔离和零权限边界".to_string());
    }
    let contract = implementation_contract(
        &request.implementation_artifact_sha256,
        &immutable_code_revision,
        &review.suite_specification_sha256,
    );
    let mut record = HistoricalOutcomeTrainingImplementationRecord {
        schema_version: IMPLEMENTATION_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        approved_registration_review: review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_complete_prior_chain: true,
        implementation_name,
        rationale,
        known_limitations,
        implementation_contract: contract,
        status: STATUS.to_string(),
        exact_stage_52_review_and_stage_51_registration_binding_confirmed: request
            .exact_stage_52_review_and_stage_51_registration_binding_confirmed,
        registrar_independent_from_complete_prior_chain_confirmed: request
            .registrar_independent_from_complete_prior_chain_confirmed,
        immutable_artifact_and_code_revision_confirmed: request
            .immutable_artifact_and_code_revision_confirmed,
        fixed_three_arm_three_seed_implementation_confirmed: request
            .fixed_three_arm_three_seed_implementation_confirmed,
        exact_65_feature_nine_raw_continuous_target_contract_confirmed: request
            .exact_65_feature_nine_raw_continuous_target_contract_confirmed,
        train_only_preprocessing_and_fit_confirmed: request
            .train_only_preprocessing_and_fit_confirmed,
        validation_selection_and_sealed_holdout_isolation_confirmed: request
            .validation_selection_and_sealed_holdout_isolation_confirmed,
        per_target_per_seed_metrics_without_composite_masking_confirmed: request
            .per_target_per_seed_metrics_without_composite_masking_confirmed,
        deterministic_replay_and_fixed_resource_ceilings_confirmed: request
            .deterministic_replay_and_fixed_resource_ceilings_confirmed,
        no_scalar_reward_action_position_or_ranking_semantics_confirmed: request
            .no_scalar_reward_action_position_or_ranking_semantics_confirmed,
        implementation_review_runner_and_run_authorization_separation_confirmed: request
            .implementation_review_runner_and_run_authorization_separation_confirmed,
        no_data_access_training_reward_shadow_order_broker_or_trading_confirmed: request
            .no_data_access_training_reward_shadow_order_broker_or_trading_confirmed,
        training_implementation_registered: true,
        future_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        isolated_runner_registration_eligible: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_started: false,
        validation_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_created: false,
        metrics_created: false,
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

fn request_checks_confirmed(
    request: &RegisterHistoricalOutcomeTrainingImplementationRequest,
) -> bool {
    request.exact_stage_52_review_and_stage_51_registration_binding_confirmed
        && request.registrar_independent_from_complete_prior_chain_confirmed
        && request.immutable_artifact_and_code_revision_confirmed
        && request.fixed_three_arm_three_seed_implementation_confirmed
        && request.exact_65_feature_nine_raw_continuous_target_contract_confirmed
        && request.train_only_preprocessing_and_fit_confirmed
        && request.validation_selection_and_sealed_holdout_isolation_confirmed
        && request.per_target_per_seed_metrics_without_composite_masking_confirmed
        && request.deterministic_replay_and_fixed_resource_ceilings_confirmed
        && request.no_scalar_reward_action_position_or_ranking_semantics_confirmed
        && request.implementation_review_runner_and_run_authorization_separation_confirmed
        && request.no_data_access_training_reward_shadow_order_broker_or_trading_confirmed
}

fn implementation_contract(
    implementation_artifact_sha256: &str,
    immutable_code_revision: &str,
    suite_specification_sha256: &str,
) -> HistoricalOutcomeTrainingImplementationContract {
    let mut contract = HistoricalOutcomeTrainingImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_artifact_sha256: implementation_artifact_sha256.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        suite_version: SUITE_VERSION.to_string(),
        suite_specification_sha256: suite_specification_sha256.to_string(),
        target_contract_version: TARGET_CONTRACT_VERSION.to_string(),
        input_schema_version: INPUT_SCHEMA_VERSION.to_string(),
        output_schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        canonical_serializer_version: CANONICAL_SERIALIZER_VERSION.to_string(),
        preprocessor_implementation_version: PREPROCESSOR_IMPLEMENTATION_VERSION.to_string(),
        algorithm_implementation_versions: vec![
            "frozen_zero_prediction_baseline@v1-no-fit".to_string(),
            "ridge_multi_target_regression@v1-train-only-fit".to_string(),
            "gradient_boosted_multi_target_regression@v1-deterministic-train-only-fit".to_string(),
        ],
        exact_feature_count: 65,
        exact_target_count: 9,
        exact_random_seeds: vec![17, 29, 43],
        reported_metric_ids: [
            "mae_per_target_per_seed",
            "spearman_per_target_per_seed",
            "directional_accuracy_per_target_per_seed",
            "calibration_slope_per_target_per_seed",
        ]
        .map(ToString::to_string)
        .to_vec(),
        maximum_wall_clock_seconds: 3_600,
        maximum_memory_mib: 8_192,
        maximum_cpu_millicores: 4_000,
        maximum_process_count: 4,
        maximum_output_bytes: 256 * 1024 * 1024,
        input_contract: "未来仅在独立实现复核、隔离 runner 登记和单次运行授权后读取精确绑定训练副本；拟合与预处理只使用 train，validation 仅用于候选选择，sealed holdout 对拟合和选择不可见。当前登记不读取任何数据。".to_string(),
        output_contract: "未来输出只能是按模型臂和随机种子分离、内容寻址且待独立校验的模型工件与逐目标指标；不得折叠为 reward、动作、仓位或排名。当前登记不创建任何输出。".to_string(),
        callable_entrypoint_present: false,
        ambient_environment_available: false,
        environment_variables_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        child_process_allowed: false,
        training_store_reads_allowed: false,
        validation_labels_visible_to_fit_worker: false,
        sealed_holdout_labels_visible_to_fit_or_selection_worker: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        scalar_reward_defined: false,
        action_position_or_ranking_semantics_defined: false,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static training implementation contract is serializable");
    contract
}

fn expected_excluded_prior_actor_ids(
    review: &HistoricalOutcomeTrainingExperimentRegistrationReview,
) -> Vec<String> {
    let mut actors = review.excluded_prior_actor_ids.clone();
    actors.push(review.reviewer_id.clone());
    actors.sort();
    actors.dedup();
    actors
}

fn ensure_review_unregistered(
    records: &[HistoricalOutcomeTrainingImplementationRecord],
    review: &HistoricalOutcomeTrainingExperimentRegistrationReview,
) -> Result<(), String> {
    if records
        .iter()
        .any(|record| record.approved_registration_review.review_id == review.review_id)
    {
        return Err("该 Stage 52 批准记录已经登记过训练实现，不得重复或覆盖".to_string());
    }
    Ok(())
}

fn validate_record(record: &HistoricalOutcomeTrainingImplementationRecord) -> Result<(), String> {
    validate_historical_outcome_training_experiment_registration_review_for_implementation(
        &record.approved_registration_review,
    )?;
    let contract = &record.implementation_contract;
    let expected_contract = implementation_contract(
        &contract.implementation_artifact_sha256,
        &contract.immutable_code_revision,
        &record
            .approved_registration_review
            .suite_specification_sha256,
    );
    let confirmations = record.exact_stage_52_review_and_stage_51_registration_binding_confirmed
        && record.registrar_independent_from_complete_prior_chain_confirmed
        && record.immutable_artifact_and_code_revision_confirmed
        && record.fixed_three_arm_three_seed_implementation_confirmed
        && record.exact_65_feature_nine_raw_continuous_target_contract_confirmed
        && record.train_only_preprocessing_and_fit_confirmed
        && record.validation_selection_and_sealed_holdout_isolation_confirmed
        && record.per_target_per_seed_metrics_without_composite_masking_confirmed
        && record.deterministic_replay_and_fixed_resource_ceilings_confirmed
        && record.no_scalar_reward_action_position_or_ranking_semantics_confirmed
        && record.implementation_review_runner_and_run_authorization_separation_confirmed
        && record.no_data_access_training_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !record.independent_implementation_review_completed
        && !record.isolated_runner_registration_eligible
        && !record.training_data_access_authorized
        && !record.training_authorized
        && !record.training_started
        && !record.validation_selection_authorized
        && !record.sealed_holdout_access_authorized
        && !record.model_artifact_created
        && !record.metrics_created
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != IMPLEMENTATION_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || record.status != STATUS
        || !valid_id(&record.implementation_id)
        || !valid_sha256(&record.implementation_sha256)
        || record.implementation_id != record.implementation_sha256[..32]
        || record.implementation_sha256 != record_fingerprint(record)?
        || record.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&record.approved_registration_review)
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
        || !valid_sha256(&contract.implementation_artifact_sha256)
        || contract.immutable_code_revision.trim().is_empty()
        || contract.immutable_code_revision.chars().count() > MAX_REVISION_CHARS
        || *contract != expected_contract
        || !confirmations
        || !record.training_implementation_registered
        || !record.future_independent_implementation_review_eligible
        || !authority_closed
    {
        return Err(
            "historical outcome training implementation is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeTrainingImplementationRecord>, String> {
    let root = record_root(state);
    let mut directory = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = tokio::fs::read(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        let record: HistoricalOutcomeTrainingImplementationRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(record.implementation_id.as_str())
        {
            return Err("training implementation path mismatch".to_string());
        }
        validate_record(&record)?;
        records.push(record);
    }
    let mut identities = BTreeSet::new();
    let mut review_ids = BTreeSet::new();
    for record in &records {
        if !identities.insert(record.implementation_id.clone())
            || !review_ids.insert(record.approved_registration_review.review_id.clone())
        {
            return Err("duplicate or replayed training implementation".to_string());
        }
    }
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeTrainingImplementationRecord,
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
    let bytes = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
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
    decision_root(state).join("historical_outcome_training_implementations/records")
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
    let path = directory.join("historical-outcome-training-implementation.lock");
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
                return Err("另一个管理员正在登记训练实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("训练实现登记锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registration_reviews::test_approved_training_experiment_registration_review;

    fn request(
        review: &HistoricalOutcomeTrainingExperimentRegistrationReview,
    ) -> RegisterHistoricalOutcomeTrainingImplementationRequest {
        RegisterHistoricalOutcomeTrainingImplementationRequest {
            expected_review_id: review.review_id.clone(),
            expected_review_sha256: review.review_sha256.clone(),
            expected_attempt_id: review.attempt_id.clone(),
            expected_registration_id: review.registration_id.clone(),
            expected_registration_sha256: review.registration_sha256.clone(),
            expected_claim_sha256: review.claim_sha256.clone(),
            expected_result_id: review.result_id.clone(),
            expected_result_sha256: review.result_sha256.clone(),
            expected_suite_specification_sha256: review.suite_specification_sha256.clone(),
            implementation_name: "隔离三臂九目标确定性训练实现".into(),
            immutable_code_revision: "oldwang-stage53-test-revision".into(),
            implementation_artifact_sha256: "a".repeat(64),
            rationale: "冻结实现工件供另一角色独立复核，不运行训练".into(),
            known_limitations: "未复核、未运行，不证明模型质量、收益或可交易性".into(),
            exact_stage_52_review_and_stage_51_registration_binding_confirmed: true,
            registrar_independent_from_complete_prior_chain_confirmed: true,
            immutable_artifact_and_code_revision_confirmed: true,
            fixed_three_arm_three_seed_implementation_confirmed: true,
            exact_65_feature_nine_raw_continuous_target_contract_confirmed: true,
            train_only_preprocessing_and_fit_confirmed: true,
            validation_selection_and_sealed_holdout_isolation_confirmed: true,
            per_target_per_seed_metrics_without_composite_masking_confirmed: true,
            deterministic_replay_and_fixed_resource_ceilings_confirmed: true,
            no_scalar_reward_action_position_or_ranking_semantics_confirmed: true,
            implementation_review_runner_and_run_authorization_separation_confirmed: true,
            no_data_access_training_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn record() -> HistoricalOutcomeTrainingImplementationRecord {
        let review = test_approved_training_experiment_registration_review();
        let contract = implementation_contract(
            &"a".repeat(64),
            "oldwang-stage53-test-revision",
            &review.suite_specification_sha256,
        );
        let mut record = HistoricalOutcomeTrainingImplementationRecord {
            schema_version: IMPLEMENTATION_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            implementation_id: String::new(),
            implementation_sha256: String::new(),
            registered_at: Utc::now(),
            registered_by: "stage-53-registrar".into(),
            approved_registration_review: review.clone(),
            excluded_prior_actor_ids: expected_excluded_prior_actor_ids(&review),
            registrar_independent_from_complete_prior_chain: true,
            implementation_name: "隔离三臂九目标确定性训练实现".into(),
            rationale: "冻结实现工件供另一角色独立复核，不运行训练".into(),
            known_limitations: "未复核、未运行，不证明模型质量、收益或可交易性".into(),
            implementation_contract: contract,
            status: STATUS.into(),
            exact_stage_52_review_and_stage_51_registration_binding_confirmed: true,
            registrar_independent_from_complete_prior_chain_confirmed: true,
            immutable_artifact_and_code_revision_confirmed: true,
            fixed_three_arm_three_seed_implementation_confirmed: true,
            exact_65_feature_nine_raw_continuous_target_contract_confirmed: true,
            train_only_preprocessing_and_fit_confirmed: true,
            validation_selection_and_sealed_holdout_isolation_confirmed: true,
            per_target_per_seed_metrics_without_composite_masking_confirmed: true,
            deterministic_replay_and_fixed_resource_ceilings_confirmed: true,
            no_scalar_reward_action_position_or_ranking_semantics_confirmed: true,
            implementation_review_runner_and_run_authorization_separation_confirmed: true,
            no_data_access_training_reward_shadow_order_broker_or_trading_confirmed: true,
            training_implementation_registered: true,
            future_independent_implementation_review_eligible: true,
            independent_implementation_review_completed: false,
            isolated_runner_registration_eligible: false,
            training_data_access_authorized: false,
            training_authorized: false,
            training_started: false,
            validation_selection_authorized: false,
            sealed_holdout_access_authorized: false,
            model_artifact_created: false,
            metrics_created: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        record.implementation_sha256 = record_fingerprint(&record).unwrap();
        record.implementation_id = record.implementation_sha256[..32].to_string();
        record
    }

    #[test]
    fn all_twelve_registration_confirmations_are_required() {
        let review = test_approved_training_experiment_registration_review();
        let mut value = request(&review);
        assert!(request_checks_confirmed(&value));
        value.validation_selection_and_sealed_holdout_isolation_confirmed = false;
        assert!(!request_checks_confirmed(&value));
    }

    #[test]
    fn contract_freezes_three_arms_three_seeds_and_nine_targets() {
        let value = record();
        assert_eq!(
            value
                .implementation_contract
                .algorithm_implementation_versions
                .len(),
            3
        );
        assert_eq!(
            value.implementation_contract.exact_random_seeds,
            [17, 29, 43]
        );
        assert_eq!(value.implementation_contract.exact_feature_count, 65);
        assert_eq!(value.implementation_contract.exact_target_count, 9);
    }

    #[test]
    fn registration_is_valid_but_not_reviewed_or_run() {
        let value = record();
        validate_record(&value).unwrap();
        assert_eq!(value.status, STATUS);
        assert!(value.training_implementation_registered);
        assert!(value.future_independent_implementation_review_eligible);
        assert!(!value.independent_implementation_review_completed);
        assert!(!value.training_started);
    }

    #[test]
    fn contract_exposes_no_entrypoint_or_data_access() {
        let contract = record().implementation_contract;
        assert!(!contract.callable_entrypoint_present);
        assert!(!contract.training_store_reads_allowed);
        assert!(!contract.outbound_network_allowed);
        assert!(!contract.sealed_holdout_labels_visible_to_fit_or_selection_worker);
    }

    #[test]
    fn registration_hash_binds_artifact_revision_and_upstream_review() {
        let value = record();
        let first = value.implementation_sha256.clone();
        let mut changed = value;
        changed.implementation_contract.immutable_code_revision = "changed".into();
        changed.approved_registration_review.review_sha256 = "f".repeat(64);
        assert_ne!(first, record_fingerprint(&changed).unwrap());
    }

    #[test]
    fn contract_hash_detects_metric_or_resource_drift() {
        let contract = record().implementation_contract;
        let first = contract.contract_sha256.clone();
        let mut changed = contract;
        changed.reported_metric_ids.pop();
        changed.maximum_memory_mib += 1;
        assert_ne!(
            first,
            fingerprint_without(&changed, &["contract_sha256"]).unwrap()
        );
    }

    #[test]
    fn scalar_reward_action_position_and_rank_are_absent() {
        let contract = record().implementation_contract;
        assert!(!contract.scalar_reward_defined);
        assert!(!contract.action_position_or_ranking_semantics_defined);
    }

    #[test]
    fn next_gate_is_only_independent_implementation_review() {
        let value = record();
        assert!(value.future_independent_implementation_review_eligible);
        assert!(!value.isolated_runner_registration_eligible);
        assert!(!value.training_authorized);
        assert!(!value.reward_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn actor_exclusion_is_sorted_unique_and_hash_bound() {
        let value = record();
        assert!(sorted_unique(&value.excluded_prior_actor_ids));
        assert!(
            !value
                .excluded_prior_actor_ids
                .contains(&value.registered_by)
        );
        let first = value.implementation_sha256.clone();
        let mut changed = value;
        changed.excluded_prior_actor_ids.push("z-upstream".into());
        assert_ne!(first, record_fingerprint(&changed).unwrap());
    }

    #[test]
    fn bounded_text_and_duplicate_registration_fail_closed() {
        assert_eq!(bounded_required("  实现  ", 8, "名称").unwrap(), "实现");
        assert!(bounded_required("   ", 8, "名称").is_err());
        let value = record();
        assert!(
            ensure_review_unregistered(&[value.clone()], &value.approved_registration_review)
                .is_err()
        );
    }
}
