//! Immutable specification registry for a future isolated Stage 51 training-suite runner.
//!
//! Registration is deliberately non-invoking. It freezes one independently approved
//! implementation, runner artifact, runtime identity and zero-ambient-capability sandbox contract.
//! It exposes no callable entrypoint and grants no authority to read a training-store dataset,
//! fit or select a model, open sealed holdout labels, emit artifacts or metrics, define a reward,
//! create a shadow portfolio, generate an order, access a broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_implementation_reviews::{
    ApprovedHistoricalOutcomeTrainingImplementationReview,
    HistoricalOutcomeTrainingImplementationReview,
    approved_historical_outcome_training_implementation_reviews,
    validate_historical_outcome_training_implementation_review_for_runner_registration,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_implementations::HistoricalOutcomeTrainingImplementationRecord;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-training-isolated-runner-registry-v1";
const RUNNER_SCHEMA_VERSION: &str = "hone-historical-outcome-training-isolated-runner-v1";
const POLICY_VERSION: &str = "hone-training-isolated-runner-registration-v1-no-execution";
const CONTRACT_SCHEMA_VERSION: &str = "hone-training-isolated-runner-contract-v1";
const RUNTIME_IDENTITY: &str = "hone-isolated-nine-target-training-runtime";
const RUNTIME_VERSION: &str = "v1-no-ambient-capabilities";
const STATUS: &str = "registered_not_run";
const INPUT_MOUNT_CONTRACT: &str = "未来经新的独立首次执行授权后，只能把本记录精确绑定且已经逐行逐位校验的 training-store dataset 作为只读输入：train 可用于拟合，validation 只用于候选选择，sealed holdout 对拟合与选择 worker 保持隐藏；不得联网补数、替换输入、读取其它训练/标签/生产资料或继承宿主环境。";
const OUTPUT_CONTRACT: &str = "未来首次执行只能向一次性隔离目录 create-once 写入内容寻址、尚未经独立校验的模型候选与逐目标逐种子指标；不得覆盖、写入训练或生产存储、折叠为标量 reward、生成动作/仓位/排名或宣称策略有效。当前登记不创建目录、模型或指标。";
const INVOCATION_CONTRACT: &str = "本记录没有可调用入口且不授予执行权限；未来任何首次调用必须绑定本 runner 规范与工件摘要，并通过新的独立首次执行授权复核。";
const NEXT_GATE: &str = "independent_first_execution_authorization_review";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;
const MAXIMUM_PARALLEL_EXPERIMENTS: u8 = 1;
const MAXIMUM_MEMORY_MIB: u32 = 8_192;
const MAXIMUM_WALL_CLOCK_SECONDS: u32 = 3_600;
const MAXIMUM_CPU_MILLICORES: u32 = 4_000;
const MAXIMUM_PROCESS_COUNT: u16 = 4;
const MAXIMUM_OUTPUT_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeTrainingIsolatedRunnerKind {
    EphemeralDeterministicTrainingProcess,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest {
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_artifact_sha256: String,
    expected_immutable_code_revision: String,
    expected_stage_52_review_sha256: String,
    expected_stage_51_registration_sha256: String,
    expected_stage_51_claim_sha256: String,
    expected_stage_51_result_sha256: String,
    expected_suite_specification_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_rows_sha256: String,
    expected_excluded_rows_sha256: String,
    expected_target_commitments_sha256: String,
    expected_review_contract_sha256: String,
    expected_independent_audit_sha256: String,
    runner_name: String,
    runner_kind: HistoricalOutcomeTrainingIsolatedRunnerKind,
    runner_code_revision: String,
    runner_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    exact_current_approved_review_and_complete_upstream_binding_confirmed: bool,
    registrar_independence_confirmed: bool,
    runner_artifact_and_code_revision_immutable_confirmed: bool,
    exact_read_only_training_input_and_content_addressed_create_once_output_confirmed: bool,
    train_validation_and_sealed_holdout_mount_isolation_confirmed: bool,
    fixed_runtime_identity_and_bounded_resource_contract_confirmed: bool,
    no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
        bool,
    registration_first_execution_and_output_validation_separation_confirmed: bool,
    no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingIsolatedRunnerContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub input_mount_contract: String,
    pub output_contract: String,
    pub invocation_contract: String,
    pub next_gate: String,
    pub callable_entrypoint_registered: bool,
    pub input_mount_read_only_required: bool,
    pub root_filesystem_read_only_required: bool,
    pub ephemeral_working_directory_required: bool,
    pub content_addressed_create_once_output_required: bool,
    pub independent_output_validation_required: bool,
    pub run_as_unprivileged_required: bool,
    pub no_new_privileges_required: bool,
    pub host_environment_inherited: bool,
    pub allowed_environment_variables: Vec<String>,
    pub secrets_available: bool,
    pub outbound_network_allowed: bool,
    pub external_tools_allowed: bool,
    pub child_process_allowed: bool,
    pub exact_training_dataset_mount_registered: bool,
    pub training_store_reads_allowed: bool,
    pub validation_labels_visible_to_fit_worker: bool,
    pub sealed_holdout_labels_visible_to_fit_or_selection_worker: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub training_store_writes_allowed: bool,
    pub model_artifact_store_writes_allowed: bool,
    pub metric_store_writes_allowed: bool,
    pub maximum_parallel_experiments: u8,
    pub maximum_memory_mib: u32,
    pub maximum_wall_clock_seconds: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u16,
    pub maximum_output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingIsolatedRunnerRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub implementation: HistoricalOutcomeTrainingImplementationRecord,
    pub implementation_review: HistoricalOutcomeTrainingImplementationReview,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_complete_approval_chain: bool,
    pub runner_name: String,
    pub runner_kind: HistoricalOutcomeTrainingIsolatedRunnerKind,
    pub runner_code_revision: String,
    pub runner_artifact_sha256: String,
    pub rationale: String,
    pub known_limitations: String,
    pub runner_contract: HistoricalOutcomeTrainingIsolatedRunnerContract,
    pub status: String,
    pub exact_current_approved_review_and_complete_upstream_binding_confirmed: bool,
    pub registrar_independence_confirmed: bool,
    pub runner_artifact_and_code_revision_immutable_confirmed: bool,
    pub exact_read_only_training_input_and_content_addressed_create_once_output_confirmed: bool,
    pub train_validation_and_sealed_holdout_mount_isolation_confirmed: bool,
    pub fixed_runtime_identity_and_bounded_resource_contract_confirmed: bool,
    pub no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
        bool,
    pub registration_first_execution_and_output_validation_separation_confirmed: bool,
    pub no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub first_execution_authorization_review_eligible: bool,
    pub first_execution_authorized: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub training_started: bool,
    pub validation_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub model_artifact_created: bool,
    pub metrics_created: bool,
    pub output_validation_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingIsolatedRunnerItem {
    pub runner: HistoricalOutcomeTrainingIsolatedRunnerRecord,
    pub approved_review_binding_current: bool,
    pub first_execution_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingIsolatedRunnerRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_reviews: Vec<ApprovedHistoricalOutcomeTrainingImplementationReview>,
    pub allowed_runner_kinds: Vec<HistoricalOutcomeTrainingIsolatedRunnerKind>,
    pub registration_allowed: bool,
    pub items: Vec<HistoricalOutcomeTrainingIsolatedRunnerItem>,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
    pub callable_entrypoint_registered: bool,
    pub first_execution_authorized: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub model_artifact_created: bool,
    pub metrics_created: bool,
    pub output_validation_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeTrainingIsolatedRunnerReadinessSummary {
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_training_isolated_runners(
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
            warn!(%error, "historical outcome training isolated runner registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果训练隔离 runner 规范登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_training_isolated_runner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome training isolated runner registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果训练隔离 runner 规范登记表暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_training_isolated_runner_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingIsolatedRunnerReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeTrainingIsolatedRunnerReadinessSummary {
        runner_count: registry.runner_count,
        current_binding_runner_count: registry.current_binding_runner_count,
        first_execution_authorization_review_eligible_count: registry
            .first_execution_authorization_review_eligible_count,
        runner_status: registry.runner_status,
    })
}

#[allow(dead_code)] // Consumed by the separately gated Stage 56 review route.
pub(crate) async fn first_execution_authorization_review_eligible_training_runners(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeTrainingIsolatedRunnerRecord>, String> {
    let approved = approved_historical_outcome_training_implementation_reviews(state).await?;
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
) -> Result<HistoricalOutcomeTrainingIsolatedRunnerRegistry, String> {
    let eligible_reviews =
        approved_historical_outcome_training_implementation_reviews(state).await?;
    let review_index = eligible_reviews
        .iter()
        .map(|value| (value.review.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let mut items = read_runners(state)
        .await?
        .into_iter()
        .map(|runner| {
            let approved_review_binding_current = review_index
                .get(runner.implementation_review.review_id.as_str())
                .is_some_and(|value| binding_matches(&runner, value));
            HistoricalOutcomeTrainingIsolatedRunnerItem {
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
        "waiting_for_current_approved_training_implementation_review"
    } else if runner_count == 0 {
        "waiting_for_immutable_isolated_training_runner_registration"
    } else if current_binding_runner_count == 0 {
        "registered_binding_stale"
    } else {
        STATUS
    };
    Ok(HistoricalOutcomeTrainingIsolatedRunnerRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_allowed: !eligible_reviews.is_empty(),
        eligible_reviews,
        allowed_runner_kinds: vec![HistoricalOutcomeTrainingIsolatedRunnerKind::EphemeralDeterministicTrainingProcess],
        items,
        runner_count,
        current_binding_runner_count,
        first_execution_authorization_review_eligible_count,
        runner_status: runner_status.to_string(),
        callable_entrypoint_registered: false,
        first_execution_authorized: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_started: false,
        model_artifact_created: false,
        metrics_created: false,
        output_validation_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 55 阶段只登记绑定当前 Stage 54 独立批准实现的训练隔离 runner 规范，冻结 runner 工件、代码、运行时、未来只读精确训练输入、create-once 内容寻址候选输出、train/validation/sealed-holdout 边界和静态资源上限；不提供调用入口。唯一下一门禁是独立首次执行授权复核；当前不读取训练数据、不拟合或选择模型、不打开 sealed holdout、不创建模型或指标，也不定义奖励、不影子、不下单、不接券商、不交易。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest,
) -> Result<HistoricalOutcomeTrainingIsolatedRunnerRecord, String> {
    let _lock = acquire_lock(state).await?;
    let approved = approved_historical_outcome_training_implementation_reviews(state).await?;
    let value = approved
        .iter()
        .find(|value| value.implementation.implementation_id == request.expected_implementation_id)
        .ok_or_else(|| "当前没有这条绑定有效且已独立批准的训练实现".to_string())?;
    validate_expected_binding(value, &request)?;
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
    if read_runners(state)
        .await?
        .iter()
        .map(runner_deduplication_fingerprint)
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|existing| existing == &dedup)
    {
        return Err(
            "完全相同的训练隔离 runner 规范已经登记；工件或规范变化时请登记新版本".to_string(),
        );
    }
    write_immutable_json(
        &runner_root(state).join(format!("{}.json", record.isolated_runner_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn validate_expected_binding(
    approved: &ApprovedHistoricalOutcomeTrainingImplementationReview,
    request: &RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest,
) -> Result<(), String> {
    let implementation = &approved.implementation;
    let review = &approved.review;
    let stage_52 = &implementation.approved_registration_review;
    let contract = &implementation.implementation_contract;
    if request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_review_id != review.review_id
        || request.expected_implementation_review_sha256 != review.review_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_artifact_sha256
            != contract.implementation_artifact_sha256
        || request.expected_immutable_code_revision != contract.immutable_code_revision
        || request.expected_stage_52_review_sha256 != stage_52.review_sha256
        || request.expected_stage_51_registration_sha256 != stage_52.registration_sha256
        || request.expected_stage_51_claim_sha256 != stage_52.claim_sha256
        || request.expected_stage_51_result_sha256 != stage_52.result_sha256
        || request.expected_suite_specification_sha256 != stage_52.suite_specification_sha256
        || request.expected_training_store_dataset_sha256 != stage_52.training_store_dataset_sha256
        || request.expected_rows_sha256 != stage_52.rows_sha256
        || request.expected_excluded_rows_sha256 != stage_52.excluded_rows_sha256
        || request.expected_target_commitments_sha256 != stage_52.target_commitments_sha256
        || request.expected_review_contract_sha256 != review.review_contract.contract_sha256
        || request.expected_independent_audit_sha256 != review.independent_audit.audit_sha256
    {
        return Err("实现独立复核、实现工件或完整上游指纹已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn new_record(
    approved: &ApprovedHistoricalOutcomeTrainingImplementationReview,
    registrar_id: &str,
    runner_name: String,
    runner_kind: HistoricalOutcomeTrainingIsolatedRunnerKind,
    runner_code_revision: String,
    runner_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    request: &RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeTrainingIsolatedRunnerRecord, String> {
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(approved);
    if registrar_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == registrar_id)
        || !request.registrar_independence_confirmed
    {
        return Err("runner 登记人必须独立于完整规范、实现与复核链".to_string());
    }
    if !request.exact_current_approved_review_and_complete_upstream_binding_confirmed
        || !request.runner_artifact_and_code_revision_immutable_confirmed
        || !request.exact_read_only_training_input_and_content_addressed_create_once_output_confirmed
        || !request.train_validation_and_sealed_holdout_mount_isolation_confirmed
        || !request.fixed_runtime_identity_and_bounded_resource_contract_confirmed
        || !request.no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed
        || !request.registration_first_execution_and_output_validation_separation_confirmed
        || !request.no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err("必须逐项确认 runner 规范边界，登记仍不得授权执行或下游用途".to_string());
    }
    let mut record = HistoricalOutcomeTrainingIsolatedRunnerRecord {
        schema_version: RUNNER_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        isolated_runner_id: String::new(),
        isolated_runner_spec_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        implementation: approved.implementation.clone(),
        implementation_review: approved.review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_complete_approval_chain: true,
        runner_name,
        runner_kind,
        runner_code_revision,
        runner_artifact_sha256,
        rationale,
        known_limitations,
        runner_contract: runner_contract(),
        status: STATUS.to_string(),
        exact_current_approved_review_and_complete_upstream_binding_confirmed: true,
        registrar_independence_confirmed: true,
        runner_artifact_and_code_revision_immutable_confirmed: true,
        exact_read_only_training_input_and_content_addressed_create_once_output_confirmed: true,
        train_validation_and_sealed_holdout_mount_isolation_confirmed: true,
        fixed_runtime_identity_and_bounded_resource_contract_confirmed: true,
        no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: true,
        registration_first_execution_and_output_validation_separation_confirmed: true,
        no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed: true,
        first_execution_authorization_review_eligible: true,
        first_execution_authorized: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_run_allowed: false,
        training_started: false,
        validation_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_created: false,
        metrics_created: false,
        output_validation_authorized: false,
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

fn runner_contract() -> HistoricalOutcomeTrainingIsolatedRunnerContract {
    let mut contract = HistoricalOutcomeTrainingIsolatedRunnerContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        runtime_identity: RUNTIME_IDENTITY.to_string(),
        runtime_version: RUNTIME_VERSION.to_string(),
        input_mount_contract: INPUT_MOUNT_CONTRACT.to_string(),
        output_contract: OUTPUT_CONTRACT.to_string(),
        invocation_contract: INVOCATION_CONTRACT.to_string(),
        next_gate: NEXT_GATE.to_string(),
        callable_entrypoint_registered: false,
        input_mount_read_only_required: true,
        root_filesystem_read_only_required: true,
        ephemeral_working_directory_required: true,
        content_addressed_create_once_output_required: true,
        independent_output_validation_required: true,
        run_as_unprivileged_required: true,
        no_new_privileges_required: true,
        host_environment_inherited: false,
        allowed_environment_variables: Vec::new(),
        secrets_available: false,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        child_process_allowed: false,
        exact_training_dataset_mount_registered: false,
        training_store_reads_allowed: false,
        validation_labels_visible_to_fit_worker: false,
        sealed_holdout_labels_visible_to_fit_or_selection_worker: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        training_store_writes_allowed: false,
        model_artifact_store_writes_allowed: false,
        metric_store_writes_allowed: false,
        maximum_parallel_experiments: MAXIMUM_PARALLEL_EXPERIMENTS,
        maximum_memory_mib: MAXIMUM_MEMORY_MIB,
        maximum_wall_clock_seconds: MAXIMUM_WALL_CLOCK_SECONDS,
        maximum_cpu_millicores: MAXIMUM_CPU_MILLICORES,
        maximum_process_count: MAXIMUM_PROCESS_COUNT,
        maximum_output_bytes: MAXIMUM_OUTPUT_BYTES,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static runner contract serializes");
    contract
}

fn expected_excluded_prior_actor_ids(
    approved: &ApprovedHistoricalOutcomeTrainingImplementationReview,
) -> Vec<String> {
    let mut actors = approved.implementation.excluded_prior_actor_ids.clone();
    actors.push(approved.implementation.registered_by.clone());
    actors.push(approved.review.reviewer_id.clone());
    actors.extend(approved.review.excluded_prior_actor_ids.clone());
    actors.sort();
    actors.dedup();
    actors
}

fn validate_record(record: &HistoricalOutcomeTrainingIsolatedRunnerRecord) -> Result<(), String> {
    validate_historical_outcome_training_implementation_review_for_runner_registration(
        &record.implementation_review,
    )?;
    let authority_closed = !record.first_execution_authorized
        && !record.training_data_access_authorized
        && !record.training_authorized
        && !record.training_run_allowed
        && !record.training_started
        && !record.validation_selection_authorized
        && !record.sealed_holdout_access_authorized
        && !record.model_artifact_created
        && !record.metrics_created
        && !record.output_validation_authorized
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let confirmations_complete = record
        .exact_current_approved_review_and_complete_upstream_binding_confirmed
        && record.registrar_independence_confirmed
        && record.runner_artifact_and_code_revision_immutable_confirmed
        && record.exact_read_only_training_input_and_content_addressed_create_once_output_confirmed
        && record.train_validation_and_sealed_holdout_mount_isolation_confirmed
        && record.fixed_runtime_identity_and_bounded_resource_contract_confirmed
        && record.no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed
        && record.registration_first_execution_and_output_validation_separation_confirmed
        && record.no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed;
    let implementation_contract = &record.implementation.implementation_contract;
    let resource_contract_current = record.runner_contract.maximum_wall_clock_seconds
        == implementation_contract.maximum_wall_clock_seconds
        && record.runner_contract.maximum_memory_mib == implementation_contract.maximum_memory_mib
        && record.runner_contract.maximum_cpu_millicores
            == implementation_contract.maximum_cpu_millicores
        && record.runner_contract.maximum_process_count
            == implementation_contract.maximum_process_count
        && record.runner_contract.maximum_output_bytes
            == implementation_contract.maximum_output_bytes;
    if record.schema_version != RUNNER_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.isolated_runner_id)
        || !valid_sha256(&record.isolated_runner_spec_sha256)
        || record.isolated_runner_id != record.isolated_runner_spec_sha256[..32]
        || record.isolated_runner_spec_sha256 != runner_fingerprint(record)?
        || record.implementation_review.implementation != record.implementation
        || !record
            .implementation_review
            .future_isolated_training_runner_registration_eligible
        || record.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(
                &ApprovedHistoricalOutcomeTrainingImplementationReview {
                    implementation: record.implementation.clone(),
                    review: record.implementation_review.clone(),
                },
            )
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record.registered_by.trim().is_empty()
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.registered_by)
        || !record.registrar_independent_from_complete_approval_chain
        || record.runner_name.trim().is_empty()
        || record.runner_name.chars().count() > MAX_NAME_CHARS
        || record.runner_code_revision.trim().is_empty()
        || record.runner_code_revision.chars().count() > MAX_REVISION_CHARS
        || !valid_sha256(&record.runner_artifact_sha256)
        || record.rationale.trim().is_empty()
        || record.rationale.chars().count() > MAX_RATIONALE_CHARS
        || record.known_limitations.trim().is_empty()
        || record.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || record.runner_contract != runner_contract()
        || !resource_contract_current
        || record.status != STATUS
        || !record.first_execution_authorization_review_eligible
        || !confirmations_complete
        || !authority_closed
    {
        return Err(
            "historical outcome training isolated runner is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn binding_matches(
    runner: &HistoricalOutcomeTrainingIsolatedRunnerRecord,
    approved: &ApprovedHistoricalOutcomeTrainingImplementationReview,
) -> bool {
    runner.implementation == approved.implementation
        && runner.implementation_review == approved.review
        && validate_record(runner).is_ok()
}

fn runner_fingerprint(
    record: &HistoricalOutcomeTrainingIsolatedRunnerRecord,
) -> Result<String, String> {
    fingerprint_without(
        record,
        &["isolated_runner_id", "isolated_runner_spec_sha256"],
    )
}

fn runner_deduplication_fingerprint(
    record: &HistoricalOutcomeTrainingIsolatedRunnerRecord,
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
) -> Result<Vec<HistoricalOutcomeTrainingIsolatedRunnerRecord>, String> {
    let root = runner_root(state);
    let mut directory = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut runners = Vec::new();
    let mut ids = BTreeSet::new();
    let mut fingerprints = BTreeSet::new();
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
        let runner: HistoricalOutcomeTrainingIsolatedRunnerRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(runner.isolated_runner_id.as_str())
        {
            return Err("training isolated runner path mismatch".to_string());
        }
        if !ids.insert(runner.isolated_runner_id.clone())
            || !fingerprints.insert(runner.isolated_runner_spec_sha256.clone())
        {
            return Err("training isolated runner registry contains duplicates".to_string());
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
    decision_root(state).join("historical-outcome-training-isolated-runners")
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
    let path = directory.join("historical-outcome-training-isolated-runner.lock");
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
                return Err("另一个管理员正在登记训练隔离 runner 规范".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("训练隔离 runner 登记锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_implementation_reviews::tests::approved_review;

    fn approved() -> ApprovedHistoricalOutcomeTrainingImplementationReview {
        let review = approved_review();
        ApprovedHistoricalOutcomeTrainingImplementationReview {
            implementation: review.implementation.clone(),
            review,
        }
    }

    fn request(
        approved: &ApprovedHistoricalOutcomeTrainingImplementationReview,
    ) -> RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest {
        let implementation = &approved.implementation;
        let stage_52 = &implementation.approved_registration_review;
        let contract = &implementation.implementation_contract;
        RegisterHistoricalOutcomeTrainingIsolatedRunnerRequest {
            expected_implementation_id: implementation.implementation_id.clone(),
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_review_id: approved.review.review_id.clone(),
            expected_implementation_review_sha256: approved.review.review_sha256.clone(),
            expected_implementation_contract_sha256: contract.contract_sha256.clone(),
            expected_implementation_artifact_sha256: contract.implementation_artifact_sha256.clone(),
            expected_immutable_code_revision: contract.immutable_code_revision.clone(),
            expected_stage_52_review_sha256: stage_52.review_sha256.clone(),
            expected_stage_51_registration_sha256: stage_52.registration_sha256.clone(),
            expected_stage_51_claim_sha256: stage_52.claim_sha256.clone(),
            expected_stage_51_result_sha256: stage_52.result_sha256.clone(),
            expected_suite_specification_sha256: stage_52.suite_specification_sha256.clone(),
            expected_training_store_dataset_sha256: stage_52.training_store_dataset_sha256.clone(),
            expected_rows_sha256: stage_52.rows_sha256.clone(),
            expected_excluded_rows_sha256: stage_52.excluded_rows_sha256.clone(),
            expected_target_commitments_sha256: stage_52.target_commitments_sha256.clone(),
            expected_review_contract_sha256: approved.review.review_contract.contract_sha256.clone(),
            expected_independent_audit_sha256: approved.review.independent_audit.audit_sha256.clone(),
            runner_name: "deterministic nine-target training runner".to_string(),
            runner_kind: HistoricalOutcomeTrainingIsolatedRunnerKind::EphemeralDeterministicTrainingProcess,
            runner_code_revision: "oldwang@training-runner123".to_string(),
            runner_artifact_sha256: "a".repeat(64),
            rationale: "freeze the reviewed implementation in a no-entrypoint sandbox".to_string(),
            known_limitations: "registered only; never executed and produces no data".to_string(),
            exact_current_approved_review_and_complete_upstream_binding_confirmed: true,
            registrar_independence_confirmed: true,
            runner_artifact_and_code_revision_immutable_confirmed: true,
            exact_read_only_training_input_and_content_addressed_create_once_output_confirmed: true,
            train_validation_and_sealed_holdout_mount_isolation_confirmed: true,
            fixed_runtime_identity_and_bounded_resource_contract_confirmed: true,
            no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: true,
            registration_first_execution_and_output_validation_separation_confirmed: true,
            no_data_read_training_model_metrics_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn record() -> HistoricalOutcomeTrainingIsolatedRunnerRecord {
        let approved = approved();
        let request = request(&approved);
        new_record(
            &approved,
            "isolated-runner-registrar",
            request.runner_name.clone(),
            request.runner_kind.clone(),
            request.runner_code_revision.clone(),
            request.runner_artifact_sha256.clone(),
            request.rationale.clone(),
            request.known_limitations.clone(),
            &request,
            DateTime::from_timestamp(1_760_000_100, 0).unwrap(),
        )
        .unwrap()
    }

    fn rehash(value: &mut HistoricalOutcomeTrainingIsolatedRunnerRecord) {
        value.isolated_runner_spec_sha256 = runner_fingerprint(value).unwrap();
        value.isolated_runner_id = value.isolated_runner_spec_sha256[..32].to_string();
    }

    #[test]
    fn exact_approved_review_and_complete_upstream_are_bound() {
        let runner = record();
        let mut approved = ApprovedHistoricalOutcomeTrainingImplementationReview {
            implementation: runner.implementation.clone(),
            review: runner.implementation_review.clone(),
        };
        assert!(binding_matches(&runner, &approved));
        approved.review.review_sha256 = "b".repeat(64);
        assert!(!binding_matches(&runner, &approved));
    }

    #[test]
    fn runner_freezes_runtime_resource_and_zero_ambient_capabilities() {
        let runner = record();
        assert_eq!(runner.runner_contract, runner_contract());
        assert_eq!(runner.runner_contract.maximum_parallel_experiments, 1);
        assert_eq!(runner.runner_contract.maximum_memory_mib, 8_192);
        assert!(!runner.runner_contract.host_environment_inherited);
        assert!(
            !runner
                .runner_contract
                .exact_training_dataset_mount_registered
        );
        assert!(!runner.runner_contract.training_store_reads_allowed);
        assert!(!runner.runner_contract.production_reads_allowed);
    }

    #[test]
    fn request_binds_exact_dataset_rows_targets_and_training_suite() {
        let approved = approved();
        let mut stale = request(&approved);
        let stage_52 = &approved.implementation.approved_registration_review;
        assert_eq!(
            stale.expected_training_store_dataset_sha256,
            stage_52.training_store_dataset_sha256
        );
        assert_eq!(stale.expected_rows_sha256, stage_52.rows_sha256);
        assert_eq!(
            stale.expected_excluded_rows_sha256,
            stage_52.excluded_rows_sha256
        );
        assert_eq!(
            stale.expected_target_commitments_sha256,
            stage_52.target_commitments_sha256
        );
        assert_eq!(
            stale.expected_suite_specification_sha256,
            stage_52.suite_specification_sha256
        );

        stale.expected_training_store_dataset_sha256 = "e".repeat(64);
        assert!(validate_expected_binding(&approved, &stale).is_err());
    }

    #[test]
    fn registrar_must_be_independent_from_complete_chain() {
        let approved = approved();
        for actor in expected_excluded_prior_actor_ids(&approved) {
            let request = request(&approved);
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
    }

    #[test]
    fn registration_has_no_entrypoint_and_never_runs() {
        let runner = record();
        assert_eq!(runner.status, "registered_not_run");
        assert!(!runner.runner_contract.callable_entrypoint_registered);
        assert!(!runner.first_execution_authorized);
        assert!(!runner.training_data_access_authorized);
        assert!(!runner.training_started);
        assert!(!runner.model_artifact_created);
        assert!(!runner.metrics_created);
        assert!(validate_record(&runner).is_ok());
    }

    #[test]
    fn registration_cannot_create_models_metrics_or_open_holdout() {
        let mut runner = record();
        runner.model_artifact_created = true;
        rehash(&mut runner);
        assert!(validate_record(&runner).is_err());

        let mut runner = record();
        runner.metrics_created = true;
        rehash(&mut runner);
        assert!(validate_record(&runner).is_err());

        let mut runner = record();
        runner.sealed_holdout_access_authorized = true;
        rehash(&mut runner);
        assert!(validate_record(&runner).is_err());
    }

    #[test]
    fn only_next_gate_is_independent_first_execution_authorization_review() {
        let runner = record();
        assert!(runner.first_execution_authorization_review_eligible);
        assert_eq!(runner.runner_contract.next_gate, NEXT_GATE);
        assert!(!runner.training_data_access_authorized);
        assert!(!runner.training_authorized);
        assert!(!runner.trading_authorized);
    }

    #[test]
    fn stale_binding_or_missing_confirmation_is_rejected() {
        let approved = approved();
        let mut stale = request(&approved);
        stale.expected_implementation_review_sha256 = "c".repeat(64);
        assert!(validate_expected_binding(&approved, &stale).is_err());
        let mut missing = request(&approved);
        missing.fixed_runtime_identity_and_bounded_resource_contract_confirmed = false;
        assert!(
            new_record(
                &approved,
                "isolated-runner-registrar",
                missing.runner_name.clone(),
                missing.runner_kind.clone(),
                missing.runner_code_revision.clone(),
                missing.runner_artifact_sha256.clone(),
                missing.rationale.clone(),
                missing.known_limitations.clone(),
                &missing,
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn any_artifact_sandbox_or_downstream_authority_tamper_fails_closed() {
        let mut runner = record();
        runner.runner_artifact_sha256 = "d".repeat(64);
        assert!(validate_record(&runner).is_err());
        rehash(&mut runner);
        assert!(validate_record(&runner).is_ok());
        runner.runner_contract.outbound_network_allowed = true;
        rehash(&mut runner);
        assert!(validate_record(&runner).is_err());
        runner.runner_contract = runner_contract();
        runner.runner_contract.training_store_reads_allowed = true;
        rehash(&mut runner);
        assert!(validate_record(&runner).is_err());
        runner.runner_contract = runner_contract();
        runner.training_authorized = true;
        rehash(&mut runner);
        assert!(validate_record(&runner).is_err());
    }

    #[test]
    fn create_once_deduplication_ignores_only_registration_actor_and_time() {
        let first = record();
        let mut replay = first.clone();
        replay.registered_by = "another-independent-registrar".to_string();
        replay.registered_at = DateTime::from_timestamp(1_760_000_200, 0).unwrap();
        rehash(&mut replay);
        assert_ne!(
            first.isolated_runner_spec_sha256,
            replay.isolated_runner_spec_sha256
        );
        assert_eq!(
            runner_deduplication_fingerprint(&first).unwrap(),
            runner_deduplication_fingerprint(&replay).unwrap()
        );
    }
}
