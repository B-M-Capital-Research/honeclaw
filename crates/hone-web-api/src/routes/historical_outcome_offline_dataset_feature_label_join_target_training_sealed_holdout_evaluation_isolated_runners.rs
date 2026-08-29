//! Immutable Stage 69 specification registry for a future isolated sealed-holdout evaluator.
//!
//! Registration is deliberately non-invoking. It freezes one independently approved Stage 68
//! implementation review, one runner artifact, one runtime identity and a zero-ambient-capability
//! sandbox contract. It exposes no callable entrypoint and grants no authority to mount or read
//! sealed-holdout features or labels, evaluate or select a candidate, write model/metric stores,
//! define reward semantics, or reach any trading surface.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementation_reviews::{
    ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview,
    HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord,
    approved_sealed_holdout_evaluation_implementation_reviews_for_runner_registration,
    validate_sealed_holdout_evaluation_implementation_review_for_runner_registration,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementations::HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-isolated-runner-registry-v1";
const RUNNER_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-isolated-runner-v1";
const POLICY_VERSION: &str =
    "hone-sealed-holdout-evaluation-isolated-runner-registration-v1-no-access-no-execution";
const CONTRACT_SCHEMA_VERSION: &str = "hone-sealed-holdout-evaluation-isolated-runner-contract-v1";
const RUNTIME_IDENTITY: &str =
    "hone-isolated-one-target-three-seed-sealed-holdout-evaluation-runtime";
const RUNTIME_VERSION: &str = "v1-no-ambient-capabilities-no-current-mount";
const STATUS: &str = "registered_not_run";
const NEXT_GATE: &str = "independent_one_shot_access_and_execution_authorization_review";
const INPUT_MOUNT_CONTRACT: &str = "当前登记不挂载任何 sealed-holdout 特征、标签或模型候选工件。未来只有新的链外独立、限时、一次性访问与执行授权，才可以只读挂载本记录精确绑定的一个目标 sealed-holdout 特征/标签分区和一种算法的 17/29/43 三个训练候选；禁止训练、重新拟合、预处理更新、候选重选、跨目标读取、反馈复用、联网补数及宿主环境继承。";
const OUTPUT_CONTRACT: &str = "未来一次执行只能向唯一隔离目录 create-once 写入内容寻址、尚未经独立校验的单目标三种子确认性指标、component-block bootstrap/Holm 诊断与逐种子通过状态；不得覆盖、重试或反馈复用，不得写正式模型/指标/训练/生产存储，不得生成跨目标综合结论、标量 reward、动作、仓位或排名。当前登记不创建输出。";
const INVOCATION_CONTRACT: &str = "本记录没有可调用入口，不授予 sealed-holdout 挂载、特征/标签访问、评估或正式选择权限。未来任何一次调用必须精确绑定本 runner、Stage 68 复核、Stage 67 实现、Stage 66 协议、目标、算法、三种子、sealed split commitment 与冻结统计合同，并先取得另一角色批准的一次性访问与执行授权。";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;
const MAXIMUM_PARALLEL_EVALUATIONS: u8 = 1;
const MAXIMUM_MEMORY_MIB: u32 = 8_192;
const MAXIMUM_WALL_CLOCK_SECONDS: u32 = 3_600;
const MAXIMUM_CPU_MILLICORES: u32 = 4_000;
const MAXIMUM_PROCESS_COUNT: u16 = 4;
const MAXIMUM_OUTPUT_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind {
    EphemeralDeterministicOneTargetThreeSeedSealedHoldoutEvaluator,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest {
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_independent_audit_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_artifact_sha256: String,
    expected_immutable_code_revision: String,
    expected_stage_66_protocol_review_sha256: String,
    expected_sealed_holdout_evaluation_protocol_sha256: String,
    expected_target_bundle_sha256: String,
    expected_recommendation_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    runner_name: String,
    runner_kind: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind,
    runner_code_revision: String,
    runner_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_through_stage_68_binding_confirmed: bool,
    registrar_independent_from_stage_68_and_complete_prior_chain_confirmed: bool,
    runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: bool,
    future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed: bool,
    training_validation_cross_target_and_feedback_isolation_confirmed: bool,
    one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed: bool,
    create_once_untrusted_output_and_independent_validation_confirmed: bool,
    fixed_runtime_identity_and_bounded_resource_contract_confirmed: bool,
    no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
        bool,
    registration_access_authorization_execution_and_output_validation_separation_confirmed: bool,
    no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
        bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub stage_68_implementation_review_sha256: String,
    pub stage_67_implementation_sha256: String,
    pub stage_66_protocol_review_sha256: String,
    pub sealed_holdout_evaluation_protocol_sha256: String,
    pub selected_algorithm_three_seed_binding_sha256: String,
    pub sealed_holdout_split_commitment_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub exact_random_seeds: Vec<u64>,
    pub canonical_input_projection_schema: String,
    pub canonical_untrusted_output_schema: String,
    pub input_mount_contract: String,
    pub output_contract: String,
    pub invocation_contract: String,
    pub next_gate: String,
    pub callable_entrypoint_registered: bool,
    pub current_sealed_holdout_mount_present: bool,
    pub current_candidate_artifact_mount_present: bool,
    pub future_exact_sealed_holdout_read_only_mount_required: bool,
    pub future_exact_three_candidate_artifact_read_only_mount_required: bool,
    pub root_filesystem_read_only_required: bool,
    pub ephemeral_working_directory_required: bool,
    pub content_addressed_create_once_output_required: bool,
    pub independent_output_validation_required: bool,
    pub one_shot_evaluation_required: bool,
    pub no_feedback_reuse_required: bool,
    pub run_as_unprivileged_required: bool,
    pub no_new_privileges_required: bool,
    pub host_environment_inherited: bool,
    pub allowed_environment_variables: Vec<String>,
    pub secrets_available: bool,
    pub outbound_network_allowed: bool,
    pub external_tools_allowed: bool,
    pub child_process_allowed: bool,
    pub sealed_holdout_features_access_allowed: bool,
    pub sealed_holdout_labels_access_allowed: bool,
    pub training_or_preprocessing_update_allowed: bool,
    pub candidate_reselection_allowed: bool,
    pub cross_target_read_or_composite_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub training_store_writes_allowed: bool,
    pub model_artifact_store_writes_allowed: bool,
    pub metric_store_writes_allowed: bool,
    pub future_untrusted_one_target_three_seed_confirmation_envelope_required: bool,
    pub no_composite_score_or_global_model_validity_claim_required: bool,
    pub maximum_parallel_evaluations: u8,
    pub maximum_memory_mib: u32,
    pub maximum_wall_clock_seconds: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u16,
    pub maximum_output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    pub implementation_review: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_68_and_complete_prior_chain: bool,
    pub runner_name: String,
    pub runner_kind: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind,
    pub runner_code_revision: String,
    pub runner_artifact_sha256: String,
    pub rationale: String,
    pub known_limitations: String,
    pub runner_contract: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerContract,
    pub status: String,
    pub exact_current_stage_51_through_stage_68_binding_confirmed: bool,
    pub registrar_independent_from_stage_68_and_complete_prior_chain_confirmed: bool,
    pub runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: bool,
    pub future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed: bool,
    pub training_validation_cross_target_and_feedback_isolation_confirmed: bool,
    pub one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed: bool,
    pub create_once_untrusted_output_and_independent_validation_confirmed: bool,
    pub fixed_runtime_identity_and_bounded_resource_contract_confirmed: bool,
    pub no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
        bool,
    pub registration_access_authorization_execution_and_output_validation_separation_confirmed:
        bool,
    pub no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub first_execution_authorization_review_eligible: bool,
    pub first_execution_authorized: bool,
    pub sealed_holdout_feature_access_authorized: bool,
    pub sealed_holdout_label_access_authorized: bool,
    pub evaluation_authorized: bool,
    pub evaluation_started: bool,
    pub evaluation_completed: bool,
    pub official_candidate_selection_authorized: bool,
    pub untrusted_output_created: bool,
    pub output_validation_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerItem {
    pub runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    pub approved_review_binding_current: bool,
    pub first_execution_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_reviews: Vec<ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview>,
    pub allowed_runner_kinds: Vec<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind>,
    pub registration_allowed: bool,
    pub items: Vec<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerItem>,
    pub registration_eligible_count: usize,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
    pub callable_entrypoint_registered: bool,
    pub current_input_mount_present: bool,
    pub first_execution_authorized: bool,
    pub sealed_holdout_feature_access_authorized: bool,
    pub sealed_holdout_label_access_authorized: bool,
    pub evaluation_authorized: bool,
    pub evaluation_started: bool,
    pub official_candidate_selection_authorized: bool,
    pub untrusted_output_created: bool,
    pub output_validation_authorized: bool,
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
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerReadinessSummary {
    pub registration_eligible_count: usize,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
}

pub(crate) async fn handle_get_historical_outcome_sealed_holdout_evaluation_isolated_runners(
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
            warn!(%error, "sealed-holdout evaluation isolated runner registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "sealed-holdout 评估隔离 runner 规范登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_sealed_holdout_evaluation_isolated_runner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest>,
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

pub(crate) async fn historical_outcome_sealed_holdout_evaluation_isolated_runner_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerReadinessSummary {
            registration_eligible_count: registry.registration_eligible_count,
            runner_count: registry.runner_count,
            current_binding_runner_count: registry.current_binding_runner_count,
            first_execution_authorization_review_eligible_count: registry
                .first_execution_authorization_review_eligible_count,
            runner_status: registry.runner_status,
        },
    )
}

#[allow(dead_code)] // Consumed by the separately gated future Stage 70 authorization review.
pub(crate) async fn first_execution_authorization_review_eligible_sealed_holdout_evaluation_runners(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord>, String> {
    let approved = Box::pin(
        approved_sealed_holdout_evaluation_implementation_reviews_for_runner_registration(state),
    )
    .await?;
    let index = approved
        .iter()
        .map(|value| (value.review.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let mut runners = read_runners(state)
        .await?
        .into_iter()
        .filter(|runner| {
            index
                .get(runner.implementation_review.review_id.as_str())
                .is_some_and(|value| binding_matches(runner, value))
        })
        .collect::<Vec<_>>();
    runners.sort_by(|left, right| {
        right
            .registered_at
            .cmp(&left.registered_at)
            .then_with(|| right.isolated_runner_id.cmp(&left.isolated_runner_id))
    });
    Ok(runners)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry, String> {
    let eligible_reviews = Box::pin(
        approved_sealed_holdout_evaluation_implementation_reviews_for_runner_registration(state),
    )
    .await?;
    let review_index = eligible_reviews
        .iter()
        .map(|value| (value.review.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let runners = read_runners(state).await?;
    let registered_review_ids = runners
        .iter()
        .map(|runner| runner.implementation_review.review_id.as_str())
        .collect::<BTreeSet<_>>();
    let registration_eligible_count = eligible_reviews
        .iter()
        .filter(|value| !registered_review_ids.contains(value.review.review_id.as_str()))
        .count();
    let mut items = runners
        .into_iter()
        .map(|runner| {
            let approved_review_binding_current = review_index
                .get(runner.implementation_review.review_id.as_str())
                .is_some_and(|value| binding_matches(&runner, value));
            HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerItem {
                first_execution_authorization_review_eligible: approved_review_binding_current,
                approved_review_binding_current,
                runner,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .runner
            .registered_at
            .cmp(&left.runner.registered_at)
            .then_with(|| {
                right
                    .runner
                    .isolated_runner_id
                    .cmp(&left.runner.isolated_runner_id)
            })
    });
    let runner_count = items.len();
    let current_binding_runner_count = items
        .iter()
        .filter(|item| item.approved_review_binding_current)
        .count();
    let first_execution_authorization_review_eligible_count = items
        .iter()
        .filter(|item| item.first_execution_authorization_review_eligible)
        .count();
    let runner_status = if eligible_reviews.is_empty() {
        "waiting_for_current_approved_sealed_holdout_evaluation_implementation_review"
    } else if runner_count == 0 {
        "waiting_for_immutable_sealed_holdout_evaluation_isolated_runner_registration"
    } else if current_binding_runner_count == 0 {
        "registered_binding_stale"
    } else {
        STATUS
    };
    Ok(HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_allowed: registration_eligible_count > 0,
        eligible_reviews,
        allowed_runner_kinds: vec![
            HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind::EphemeralDeterministicOneTargetThreeSeedSealedHoldoutEvaluator,
        ],
        items,
        registration_eligible_count,
        runner_count,
        current_binding_runner_count,
        first_execution_authorization_review_eligible_count,
        runner_status: runner_status.to_string(),
        callable_entrypoint_registered: false,
        current_input_mount_present: false,
        first_execution_authorized: false,
        sealed_holdout_feature_access_authorized: false,
        sealed_holdout_label_access_authorized: false,
        evaluation_authorized: false,
        evaluation_started: false,
        official_candidate_selection_authorized: false,
        untrusted_output_created: false,
        output_validation_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 69 阶段只为当前 Stage 68 独立批准实现登记一个不可变、内容寻址、无入口的 sealed-holdout 评估隔离 runner 规格；精确冻结 Stage 51–68、目标、算法、17/29/43 三种子、65/1、特征/预处理、sealed split commitment、统计协议、未来只读输入、create-once 不可信输出和静态资源上限。唯一下一门禁是链外独立的一次性访问与执行授权复核；当前没有挂载、留出集访问、评估、输出、正式选模、模型/指标库、奖励、影子、订单、券商或交易权限。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord, String> {
    let _lock = acquire_lock(state).await?;
    let approved = Box::pin(
        approved_sealed_holdout_evaluation_implementation_reviews_for_runner_registration(state),
    )
    .await?;
    let value = approved
        .iter()
        .find(|value| value.implementation.implementation_id == request.expected_implementation_id)
        .ok_or_else(|| "当前没有这条绑定有效且已独立批准的 sealed-holdout 评估实现".to_string())?;
    validate_expected_binding(value, &request)?;
    let existing = read_runners(state).await?;
    if existing
        .iter()
        .any(|runner| runner.implementation_review.review_id == value.review.review_id)
    {
        return Err("这条 Stage 68 批准已经登记过 sealed-holdout 隔离 runner".to_string());
    }
    let runner_name = bounded_required(&request.runner_name, MAX_NAME_CHARS, "runner 名称")?;
    let runner_code_revision = bounded_required(
        &request.runner_code_revision,
        MAX_REVISION_CHARS,
        "runner 代码版本",
    )?;
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "登记理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    if !valid_sha256(&request.runner_artifact_sha256) {
        return Err("runner 工件 SHA-256 无效".to_string());
    }
    let record = new_record(
        value,
        registrar_id,
        runner_name,
        request.runner_kind.clone(),
        runner_code_revision,
        request.runner_artifact_sha256.to_ascii_lowercase(),
        rationale,
        known_limitations,
        &request,
        Utc::now(),
    )?;
    let dedup = runner_deduplication_fingerprint(&record)?;
    if existing
        .iter()
        .map(runner_deduplication_fingerprint)
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|fingerprint| fingerprint == &dedup)
    {
        return Err("完全相同的 sealed-holdout 评估隔离 runner 规范已经登记".to_string());
    }
    write_immutable_json(
        &runner_root(state).join(format!("{}.json", record.isolated_runner_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn validate_expected_binding(
    approved: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview,
    request: &RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest,
) -> Result<(), String> {
    let implementation = &approved.implementation;
    let review = &approved.review;
    let contract = &implementation.implementation_contract;
    if request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_review_id != review.review_id
        || request.expected_implementation_review_sha256 != review.review_sha256
        || request.expected_independent_audit_sha256 != review.independent_audit.audit_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_artifact_sha256
            != contract.implementation_artifact_sha256
        || request.expected_immutable_code_revision != contract.immutable_code_revision
        || request.expected_stage_66_protocol_review_sha256
            != contract.stage_66_protocol_review_sha256
        || request.expected_sealed_holdout_evaluation_protocol_sha256
            != contract.sealed_holdout_evaluation_protocol_sha256
        || request.expected_target_bundle_sha256 != contract.target_bundle_sha256
        || request.expected_recommendation_sha256 != contract.recommendation_sha256
        || request.expected_selected_algorithm_three_seed_binding_sha256
            != contract.selected_algorithm_three_seed_binding_sha256
        || request.expected_sealed_holdout_split_commitment_sha256
            != contract.sealed_holdout_split_commitment_sha256
        || request.expected_feature_order_sha256 != contract.feature_order_sha256
        || request.expected_preprocessing_sha256 != contract.preprocessing_sha256
        || request.expected_target_id != contract.target_id
        || request.expected_frozen_candidate_algorithm_id != contract.frozen_candidate_algorithm_id
    {
        return Err(
            "Stage 68 复核、实现、协议、目标、算法、特征或 sealed split 绑定已经变化".to_string(),
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn new_record(
    approved: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview,
    registrar_id: &str,
    runner_name: String,
    runner_kind: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind,
    runner_code_revision: String,
    runner_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    request: &RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord, String> {
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(approved);
    if registrar_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == registrar_id)
        || !request.registrar_independent_from_stage_68_and_complete_prior_chain_confirmed
    {
        return Err("runner 登记人必须独立于 Stage 68 和完整 Stage 51–67 责任链".to_string());
    }
    if !request.exact_current_stage_51_through_stage_68_binding_confirmed
        || !request.runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed
        || !request.future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed
        || !request.training_validation_cross_target_and_feedback_isolation_confirmed
        || !request.one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed
        || !request.create_once_untrusted_output_and_independent_validation_confirmed
        || !request.fixed_runtime_identity_and_bounded_resource_contract_confirmed
        || !request.no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed
        || !request.registration_access_authorization_execution_and_output_validation_separation_confirmed
        || !request.no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err(
            "必须逐项确认 sealed-holdout runner 规范边界，登记不得授权访问或评估"
                .to_string(),
        );
    }
    let mut record = HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord {
        schema_version: RUNNER_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        isolated_runner_id: String::new(),
        isolated_runner_spec_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        implementation: approved.implementation.clone(),
        implementation_review: approved.review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_68_and_complete_prior_chain: true,
        runner_name,
        runner_kind,
        runner_code_revision,
        runner_artifact_sha256,
        rationale,
        known_limitations,
        runner_contract: runner_contract(approved)?,
        status: STATUS.to_string(),
        exact_current_stage_51_through_stage_68_binding_confirmed: true,
        registrar_independent_from_stage_68_and_complete_prior_chain_confirmed: true,
        runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: true,
        future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed: true,
        training_validation_cross_target_and_feedback_isolation_confirmed: true,
        one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed: true,
        create_once_untrusted_output_and_independent_validation_confirmed: true,
        fixed_runtime_identity_and_bounded_resource_contract_confirmed: true,
        no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
            true,
        registration_access_authorization_execution_and_output_validation_separation_confirmed:
            true,
        no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
            true,
        first_execution_authorization_review_eligible: true,
        first_execution_authorized: false,
        sealed_holdout_feature_access_authorized: false,
        sealed_holdout_label_access_authorized: false,
        evaluation_authorized: false,
        evaluation_started: false,
        evaluation_completed: false,
        official_candidate_selection_authorized: false,
        untrusted_output_created: false,
        output_validation_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.isolated_runner_spec_sha256 = runner_fingerprint(&record)?;
    record.isolated_runner_id = record.isolated_runner_spec_sha256[..32].to_string();
    validate_record(&record)?;
    Ok(record)
}

fn runner_contract(
    approved: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerContract, String> {
    let implementation = &approved.implementation;
    let source = &implementation.implementation_contract;
    let mut contract = HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        runtime_identity: RUNTIME_IDENTITY.to_string(),
        runtime_version: RUNTIME_VERSION.to_string(),
        stage_68_implementation_review_sha256: approved.review.review_sha256.clone(),
        stage_67_implementation_sha256: implementation.implementation_sha256.clone(),
        stage_66_protocol_review_sha256: source.stage_66_protocol_review_sha256.clone(),
        sealed_holdout_evaluation_protocol_sha256: source
            .sealed_holdout_evaluation_protocol_sha256
            .clone(),
        selected_algorithm_three_seed_binding_sha256: source
            .selected_algorithm_three_seed_binding_sha256
            .clone(),
        sealed_holdout_split_commitment_sha256: source
            .sealed_holdout_split_commitment_sha256
            .clone(),
        feature_order_sha256: source.feature_order_sha256.clone(),
        preprocessing_sha256: source.preprocessing_sha256.clone(),
        target_id: source.target_id.clone(),
        frozen_candidate_algorithm_id: source.frozen_candidate_algorithm_id.clone(),
        exact_random_seeds: source.exact_random_seeds.clone(),
        canonical_input_projection_schema: source.canonical_input_projection_schema.clone(),
        canonical_untrusted_output_schema: source.canonical_untrusted_output_schema.clone(),
        input_mount_contract: INPUT_MOUNT_CONTRACT.to_string(),
        output_contract: OUTPUT_CONTRACT.to_string(),
        invocation_contract: INVOCATION_CONTRACT.to_string(),
        next_gate: NEXT_GATE.to_string(),
        callable_entrypoint_registered: false,
        current_sealed_holdout_mount_present: false,
        current_candidate_artifact_mount_present: false,
        future_exact_sealed_holdout_read_only_mount_required: true,
        future_exact_three_candidate_artifact_read_only_mount_required: true,
        root_filesystem_read_only_required: true,
        ephemeral_working_directory_required: true,
        content_addressed_create_once_output_required: true,
        independent_output_validation_required: true,
        one_shot_evaluation_required: true,
        no_feedback_reuse_required: true,
        run_as_unprivileged_required: true,
        no_new_privileges_required: true,
        host_environment_inherited: false,
        allowed_environment_variables: Vec::new(),
        secrets_available: false,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        child_process_allowed: false,
        sealed_holdout_features_access_allowed: false,
        sealed_holdout_labels_access_allowed: false,
        training_or_preprocessing_update_allowed: false,
        candidate_reselection_allowed: false,
        cross_target_read_or_composite_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        training_store_writes_allowed: false,
        model_artifact_store_writes_allowed: false,
        metric_store_writes_allowed: false,
        future_untrusted_one_target_three_seed_confirmation_envelope_required: true,
        no_composite_score_or_global_model_validity_claim_required: true,
        maximum_parallel_evaluations: MAXIMUM_PARALLEL_EVALUATIONS,
        maximum_memory_mib: MAXIMUM_MEMORY_MIB,
        maximum_wall_clock_seconds: MAXIMUM_WALL_CLOCK_SECONDS,
        maximum_cpu_millicores: MAXIMUM_CPU_MILLICORES,
        maximum_process_count: MAXIMUM_PROCESS_COUNT,
        maximum_output_bytes: MAXIMUM_OUTPUT_BYTES,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn expected_excluded_prior_actor_ids(
    approved: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview,
) -> Vec<String> {
    let mut actors = approved.implementation.excluded_prior_actor_ids.clone();
    actors.push(approved.implementation.registered_by.clone());
    actors.push(
        approved
            .implementation
            .upstream_protocol_review
            .reviewer_id
            .clone(),
    );
    actors.push(approved.review.reviewer_id.clone());
    actors.extend(approved.review.excluded_prior_actor_ids.clone());
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn validate_record(
    record: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
) -> Result<(), String> {
    validate_sealed_holdout_evaluation_implementation_review_for_runner_registration(
        &record.implementation_review,
    )?;
    let authority_closed = !record.first_execution_authorized
        && !record.sealed_holdout_feature_access_authorized
        && !record.sealed_holdout_label_access_authorized
        && !record.evaluation_authorized
        && !record.evaluation_started
        && !record.evaluation_completed
        && !record.official_candidate_selection_authorized
        && !record.untrusted_output_created
        && !record.output_validation_authorized
        && !record.model_artifact_store_written
        && !record.metric_store_written
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let confirmations_complete = record.exact_current_stage_51_through_stage_68_binding_confirmed
        && record.registrar_independent_from_stage_68_and_complete_prior_chain_confirmed
        && record.runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed
        && record.future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed
        && record.training_validation_cross_target_and_feedback_isolation_confirmed
        && record.one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed
        && record.create_once_untrusted_output_and_independent_validation_confirmed
        && record.fixed_runtime_identity_and_bounded_resource_contract_confirmed
        && record.no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed
        && record.registration_access_authorization_execution_and_output_validation_separation_confirmed
        && record.no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed;
    let approved = ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview {
        implementation: record.implementation.clone(),
        review: record.implementation_review.clone(),
    };
    if record.schema_version != RUNNER_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.isolated_runner_id)
        || !valid_sha256(&record.isolated_runner_spec_sha256)
        || record.isolated_runner_id != record.isolated_runner_spec_sha256[..32]
        || record.isolated_runner_spec_sha256 != runner_fingerprint(record)?
        || record.implementation_review.implementation != record.implementation
        || !record
            .implementation_review
            .future_isolated_runner_registration_eligible
        || record.excluded_prior_actor_ids != expected_excluded_prior_actor_ids(&approved)
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record.registered_by.trim().is_empty()
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.registered_by)
        || !record.registrar_independent_from_stage_68_and_complete_prior_chain
        || record.runner_name.trim().is_empty()
        || record.runner_name.chars().count() > MAX_NAME_CHARS
        || record.runner_code_revision.trim().is_empty()
        || record.runner_code_revision.chars().count() > MAX_REVISION_CHARS
        || !valid_sha256(&record.runner_artifact_sha256)
        || record.rationale.trim().is_empty()
        || record.rationale.chars().count() > MAX_RATIONALE_CHARS
        || record.known_limitations.trim().is_empty()
        || record.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || record.runner_contract != runner_contract(&approved)?
        || record.status != STATUS
        || !record.first_execution_authorization_review_eligible
        || !confirmations_complete
        || !authority_closed
    {
        return Err(
            "historical outcome sealed-holdout evaluation isolated runner is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn binding_matches(
    runner: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    approved: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview,
) -> bool {
    runner.implementation == approved.implementation
        && runner.implementation_review == approved.review
        && validate_record(runner).is_ok()
}

fn runner_fingerprint(
    record: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
) -> Result<String, String> {
    fingerprint_without(
        record,
        &["isolated_runner_id", "isolated_runner_spec_sha256"],
    )
}

fn runner_deduplication_fingerprint(
    record: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
) -> Result<String, String> {
    fingerprint_without(
        record,
        &[
            "isolated_runner_id",
            "isolated_runner_spec_sha256",
            "registered_at",
            "registered_by",
        ],
    )
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

async fn read_runners(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord>, String> {
    let root = runner_root(state);
    let mut directory = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut runners = Vec::new();
    let mut ids = BTreeSet::new();
    let mut fingerprints = BTreeSet::new();
    let mut review_ids = BTreeSet::new();
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
        let runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(runner.isolated_runner_id.as_str())
        {
            return Err("sealed-holdout evaluation isolated runner path mismatch".to_string());
        }
        if !ids.insert(runner.isolated_runner_id.clone())
            || !fingerprints.insert(runner.isolated_runner_spec_sha256.clone())
            || !review_ids.insert(runner.implementation_review.review_id.clone())
        {
            return Err(
                "sealed-holdout evaluation isolated runner registry contains duplicates"
                    .to_string(),
            );
        }
        runners.push(runner);
    }
    Ok(runners)
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

fn runner_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-sealed-holdout-evaluation-isolated-runners")
}

struct RunnerLock {
    path: PathBuf,
}

impl Drop for RunnerLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<RunnerLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("sealed-holdout-evaluation-isolated-runner.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(RunnerLock { path }),
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
                return Err("另一个管理员正在登记 sealed-holdout 评估隔离 runner".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("sealed-holdout 评估隔离 runner 登记锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementation_reviews::tests::approved;

    fn request(
        approved: &ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview,
    ) -> RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest {
        let implementation = &approved.implementation;
        let contract = &implementation.implementation_contract;
        RegisterHistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRequest {
            expected_implementation_id: implementation.implementation_id.clone(),
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_review_id: approved.review.review_id.clone(),
            expected_implementation_review_sha256: approved.review.review_sha256.clone(),
            expected_independent_audit_sha256: approved
                .review
                .independent_audit
                .audit_sha256
                .clone(),
            expected_implementation_contract_sha256: contract.contract_sha256.clone(),
            expected_implementation_artifact_sha256: contract
                .implementation_artifact_sha256
                .clone(),
            expected_immutable_code_revision: contract.immutable_code_revision.clone(),
            expected_stage_66_protocol_review_sha256: contract
                .stage_66_protocol_review_sha256
                .clone(),
            expected_sealed_holdout_evaluation_protocol_sha256: contract
                .sealed_holdout_evaluation_protocol_sha256
                .clone(),
            expected_target_bundle_sha256: contract.target_bundle_sha256.clone(),
            expected_recommendation_sha256: contract.recommendation_sha256.clone(),
            expected_selected_algorithm_three_seed_binding_sha256: contract
                .selected_algorithm_three_seed_binding_sha256
                .clone(),
            expected_sealed_holdout_split_commitment_sha256: contract
                .sealed_holdout_split_commitment_sha256
                .clone(),
            expected_feature_order_sha256: contract.feature_order_sha256.clone(),
            expected_preprocessing_sha256: contract.preprocessing_sha256.clone(),
            expected_target_id: contract.target_id.clone(),
            expected_frozen_candidate_algorithm_id: contract
                .frozen_candidate_algorithm_id
                .clone(),
            runner_name: "stage69 sealed-holdout evaluator runner".to_string(),
            runner_kind: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerKind::EphemeralDeterministicOneTargetThreeSeedSealedHoldoutEvaluator,
            runner_code_revision: "stage69-runner-revision".to_string(),
            runner_artifact_sha256: "a".repeat(64),
            rationale: "冻结未来单目标三种子的隔离 sealed-holdout runner 规格".to_string(),
            known_limitations: "当前无入口、挂载、留出集访问、评估或输出".to_string(),
            exact_current_stage_51_through_stage_68_binding_confirmed: true,
            registrar_independent_from_stage_68_and_complete_prior_chain_confirmed: true,
            runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: true,
            future_exact_read_only_one_target_holdout_and_three_candidate_mounts_confirmed: true,
            training_validation_cross_target_and_feedback_isolation_confirmed: true,
            one_algorithm_three_seed_metrics_bootstrap_holm_and_sample_gates_confirmed: true,
            create_once_untrusted_output_and_independent_validation_confirmed: true,
            fixed_runtime_identity_and_bounded_resource_contract_confirmed: true,
            no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: true,
            registration_access_authorization_execution_and_output_validation_separation_confirmed: true,
            no_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn record() -> HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord {
        let approved = approved();
        let request = request(&approved);
        new_record(
            &approved,
            "stage-69-registrar",
            request.runner_name.clone(),
            request.runner_kind.clone(),
            request.runner_code_revision.clone(),
            request.runner_artifact_sha256.clone(),
            request.rationale.clone(),
            request.known_limitations.clone(),
            &request,
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn contract_has_no_entrypoint_current_mount_or_access() {
        let approved = approved();
        let contract = runner_contract(&approved).unwrap();
        assert!(!contract.callable_entrypoint_registered);
        assert!(!contract.current_sealed_holdout_mount_present);
        assert!(!contract.current_candidate_artifact_mount_present);
        assert!(!contract.sealed_holdout_features_access_allowed);
        assert!(!contract.sealed_holdout_labels_access_allowed);
        assert!(contract.one_shot_evaluation_required);
        assert!(contract.no_feedback_reuse_required);
    }

    #[test]
    fn registration_is_content_addressed_and_valid() {
        let value = record();
        assert_eq!(
            value.isolated_runner_id,
            value.isolated_runner_spec_sha256[..32]
        );
        assert!(validate_record(&value).is_ok());
        assert_eq!(value.status, STATUS);
    }

    #[test]
    fn registration_keeps_all_authority_closed() {
        let value = record();
        assert!(!value.sealed_holdout_feature_access_authorized);
        assert!(!value.sealed_holdout_label_access_authorized);
        assert!(!value.evaluation_authorized);
        assert!(!value.official_candidate_selection_authorized);
        assert!(!value.model_artifact_store_written);
        assert!(!value.metric_store_written);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn registrar_must_be_outside_complete_chain() {
        let approved = approved();
        let request = request(&approved);
        let actor = approved.review.reviewer_id.clone();
        assert!(
            new_record(
                &approved,
                &actor,
                request.runner_name.clone(),
                request.runner_kind.clone(),
                request.runner_code_revision.clone(),
                request.runner_artifact_sha256.clone(),
                request.rationale.clone(),
                request.known_limitations.clone(),
                &request,
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn every_confirmation_is_required() {
        let approved = approved();
        let mut request = request(&approved);
        request.training_validation_cross_target_and_feedback_isolation_confirmed = false;
        assert!(
            new_record(
                &approved,
                "stage-69-registrar",
                request.runner_name.clone(),
                request.runner_kind.clone(),
                request.runner_code_revision.clone(),
                request.runner_artifact_sha256.clone(),
                request.rationale.clone(),
                request.known_limitations.clone(),
                &request,
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn stale_expected_binding_is_rejected() {
        let approved = approved();
        let mut request = request(&approved);
        request.expected_sealed_holdout_split_commitment_sha256 = "f".repeat(64);
        assert!(validate_expected_binding(&approved, &request).is_err());
    }

    #[test]
    fn authority_escalation_or_tampering_is_rejected() {
        let mut value = record();
        value.evaluation_authorized = true;
        value.isolated_runner_spec_sha256 = runner_fingerprint(&value).unwrap();
        value.isolated_runner_id = value.isolated_runner_spec_sha256[..32].to_string();
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn deduplication_ignores_actor_and_timestamp_but_not_artifact() {
        let left = record();
        let mut right = left.clone();
        right.registered_by = "another-independent-registrar".to_string();
        right.registered_at += chrono::Duration::seconds(1);
        assert_eq!(
            runner_deduplication_fingerprint(&left).unwrap(),
            runner_deduplication_fingerprint(&right).unwrap()
        );
        right.runner_artifact_sha256 = "b".repeat(64);
        assert_ne!(
            runner_deduplication_fingerprint(&left).unwrap(),
            runner_deduplication_fingerprint(&right).unwrap()
        );
    }
}
