//! Create-once registration of one Stage 78 controlled-shadow isolated runner specification.
//!
//! Registration freezes an exact Stage 77 approval, one reproducible executable artifact and code
//! revision, future read-only input and create-once untrusted output envelopes, a fixed runtime
//! identity and resource ceilings. It creates no callable entrypoint, data mount, shadow ledger,
//! position, order, broker connection or trading capability.

use std::collections::BTreeSet;
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_implementation_reviews::{
    ControlledShadowExperimentImplementationReviewRecord,
    IndependentlyReviewedControlledShadowExperimentImplementation,
    independently_reviewed_controlled_shadow_experiment_implementations_for_runner_registration,
    validate_review,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_implementations::{
    ControlledShadowExperimentImplementationContract, ControlledShadowExperimentImplementationRecord,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-isolated-runner-specification-registry-v2";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-isolated-runner-specification-v2-artifact-bound";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-isolated-runner-zero-capability-contract-v2-artifact-bound";
const POLICY_VERSION: &str = "hone-controlled-shadow-experiment-isolated-runner-specification-create-once-v2-artifact-bound-no-execution";
const STATUS: &str = "registered_not_authorized_not_run";
const NEXT_GATE: &str = "independent_first_shadow_execution_authorization_review";
const RUNTIME_IDENTITY: &str = "hone-controlled-shadow-forward-replay-runtime";
const RUNTIME_VERSION: &str = "v1-artifact-bound-no-current-mount";
const FUTURE_INPUT_ENVELOPE: &str =
    "controlled_shadow_point_in_time_read_only_input_v1_not_mounted";
const FUTURE_OUTPUT_ENVELOPE: &str =
    "controlled_shadow_checkpoint_observation_envelope_v1_no_order_intent";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;
const MAXIMUM_MEMORY_MIB: u32 = 2_048;
const MAXIMUM_WALL_CLOCK_SECONDS: u32 = 3_600;
const MAXIMUM_CPU_MILLICORES: u32 = 2_000;
const MAXIMUM_PROCESS_COUNT: u32 = 1;
const MAXIMUM_OUTPUT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowExperimentIsolatedRunnerKind {
    EphemeralDeterministicForwardReplaySpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowExperimentIsolatedRunnerRequest {
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_independent_audit_sha256: String,
    expected_design_review_sha256: String,
    expected_design_registration_sha256: String,
    expected_design_specification_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    runner_name: String,
    runner_kind: ControlledShadowExperimentIsolatedRunnerKind,
    runner_spec_revision: String,
    runner_code_revision: String,
    runner_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    future_mount_constraints: String,
    future_output_constraints: String,
    exact_current_stage_51_through_stage_77_binding_confirmed: bool,
    registrar_independent_from_stage_77_and_complete_prior_chain_confirmed: bool,
    implementation_review_audit_contract_and_design_hashes_reproduced_confirmed: bool,
    runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: bool,
    no_callable_entrypoint_or_current_mount_confirmed: bool,
    future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: bool,
    future_create_once_untrusted_independently_validated_output_confirmed: bool,
    deterministic_replay_long_only_caps_costs_counterfactuals_and_stop_rules_preserved_confirmed:
        bool,
    fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: bool,
    no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    no_model_metric_store_training_feedback_composite_or_reward_confirmed: bool,
    no_shadow_run_ledger_position_order_broker_or_trading_confirmed: bool,
    registration_only_opens_independent_first_execution_authorization_review_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentIsolatedRunnerContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub stage_77_implementation_review_id: String,
    pub stage_77_implementation_review_sha256: String,
    pub stage_77_independent_audit_sha256: String,
    pub stage_76_implementation_id: String,
    pub stage_76_implementation_sha256: String,
    pub stage_76_implementation_contract_sha256: String,
    pub stage_75_design_review_sha256: String,
    pub stage_74_design_registration_sha256: String,
    pub design_specification_sha256: String,
    pub exact_approved_implementation_contract: ControlledShadowExperimentImplementationContract,
    pub runner_spec_revision: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub future_input_envelope: String,
    pub future_output_envelope: String,
    pub next_gate: String,
    pub specification_registered: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub data_access_authorized: bool,
    pub root_filesystem_read_only_required: bool,
    pub ephemeral_working_directory_required: bool,
    pub run_as_unprivileged_required: bool,
    pub no_new_privileges_required: bool,
    pub future_input_read_only_required: bool,
    pub future_input_point_in_time_required: bool,
    pub future_input_content_addressed_required: bool,
    pub future_input_allowlisted_required: bool,
    pub future_output_create_once_required: bool,
    pub future_output_untrusted_required: bool,
    pub future_output_independent_validation_required: bool,
    pub future_output_order_intent_allowed: bool,
    pub future_output_broker_payload_allowed: bool,
    pub environment_inheritance_allowed: bool,
    pub allowed_environment_variables: Vec<String>,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub model_store_writes_allowed: bool,
    pub metric_store_writes_allowed: bool,
    pub training_feedback_allowed: bool,
    pub scalar_reward_defined: bool,
    pub maximum_parallel_runs: u32,
    pub maximum_memory_mib: u32,
    pub maximum_wall_clock_seconds: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u32,
    pub maximum_output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentIsolatedRunnerRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub implementation: ControlledShadowExperimentImplementationRecord,
    pub implementation_review: ControlledShadowExperimentImplementationReviewRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_77_and_complete_prior_chain: bool,
    pub runner_name: String,
    pub runner_kind: ControlledShadowExperimentIsolatedRunnerKind,
    pub runner_spec_revision: String,
    pub runner_code_revision: String,
    pub runner_artifact_sha256: String,
    pub rationale: String,
    pub known_limitations: String,
    pub future_mount_constraints: String,
    pub future_output_constraints: String,
    pub runner_contract: ControlledShadowExperimentIsolatedRunnerContract,
    pub status: String,
    pub exact_current_stage_51_through_stage_77_binding_confirmed: bool,
    pub registrar_independent_from_stage_77_and_complete_prior_chain_confirmed: bool,
    pub implementation_review_audit_contract_and_design_hashes_reproduced_confirmed: bool,
    pub runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: bool,
    pub no_callable_entrypoint_or_current_mount_confirmed: bool,
    pub future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: bool,
    pub future_create_once_untrusted_independently_validated_output_confirmed: bool,
    pub deterministic_replay_long_only_caps_costs_counterfactuals_and_stop_rules_preserved_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: bool,
    pub no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    pub no_model_metric_store_training_feedback_composite_or_reward_confirmed: bool,
    pub no_shadow_run_ledger_position_order_broker_or_trading_confirmed: bool,
    pub registration_only_opens_independent_first_execution_authorization_review_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub first_execution_authorization_review_eligible: bool,
    pub first_execution_authorized: bool,
    pub shadow_run_started: bool,
    pub shadow_run_completed: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentIsolatedRunnerItem {
    pub runner: ControlledShadowExperimentIsolatedRunnerRecord,
    pub approved_review_binding_current: bool,
    pub first_execution_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentIsolatedRunnerRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_implementations:
        Vec<IndependentlyReviewedControlledShadowExperimentImplementation>,
    pub registration_eligible_count: usize,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub allowed_runner_kinds: Vec<ControlledShadowExperimentIsolatedRunnerKind>,
    pub items: Vec<ControlledShadowExperimentIsolatedRunnerItem>,
    pub runner_status: String,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub shadow_run_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowExperimentIsolatedRunnerReadinessSummary {
    pub registration_eligible_count: usize,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_experiment_isolated_runners(
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
            warn!(%error, "controlled shadow isolated runner specification read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "受控影子隔离 runner 规格登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_experiment_isolated_runner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowExperimentIsolatedRunnerRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    if implementation_id != request.expected_implementation_id {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "实现 ID 与路径不一致");
    }
    match register_once(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_experiment_isolated_runner_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowExperimentIsolatedRunnerReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(ControlledShadowExperimentIsolatedRunnerReadinessSummary {
        registration_eligible_count: registry.registration_eligible_count,
        runner_count: registry.runner_count,
        current_binding_runner_count: registry.current_binding_runner_count,
        first_execution_authorization_review_eligible_count: registry
            .first_execution_authorization_review_eligible_count,
        runner_status: registry.runner_status,
    })
}

#[allow(dead_code)] // Consumed by the separately gated future Stage 79 authorization review.
pub(crate) async fn controlled_shadow_runners_for_first_execution_authorization_review(
    state: &AppState,
) -> Result<Vec<ControlledShadowExperimentIsolatedRunnerRecord>, String> {
    let approved = independently_reviewed_controlled_shadow_experiment_implementations_for_runner_registration(state).await?;
    let runners = read_runners(state).await?;
    let mut current = runners
        .into_iter()
        .filter(|runner| {
            approved.iter().any(|value| binding_matches(runner, value))
                && runner.first_execution_authorization_review_eligible
        })
        .collect::<Vec<_>>();
    current.sort_by(|left, right| {
        right
            .registered_at
            .cmp(&left.registered_at)
            .then_with(|| right.isolated_runner_id.cmp(&left.isolated_runner_id))
    });
    Ok(current)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowExperimentIsolatedRunnerRegistry, String> {
    let approved = independently_reviewed_controlled_shadow_experiment_implementations_for_runner_registration(state).await?;
    let runners = read_runners(state).await?;
    let mut items = runners
        .into_iter()
        .map(|runner| {
            let current = approved.iter().any(|value| binding_matches(&runner, value));
            ControlledShadowExperimentIsolatedRunnerItem {
                first_execution_authorization_review_eligible: current
                    && runner.first_execution_authorization_review_eligible,
                approved_review_binding_current: current,
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
    let registered_review_ids = items
        .iter()
        .map(|item| item.runner.implementation_review.review_id.as_str())
        .collect::<BTreeSet<_>>();
    let registration_eligible_count = approved
        .iter()
        .filter(|value| !registered_review_ids.contains(value.review.review_id.as_str()))
        .count();
    let eligible_implementations = approved
        .iter()
        .filter(|value| !registered_review_ids.contains(value.review.review_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let current_binding_runner_count = items
        .iter()
        .filter(|item| item.approved_review_binding_current)
        .count();
    let first_execution_authorization_review_eligible_count = items
        .iter()
        .filter(|item| item.first_execution_authorization_review_eligible)
        .count();
    let runner_status = if approved.is_empty() {
        "waiting_stage_77_independently_approved_implementation"
    } else if first_execution_authorization_review_eligible_count > 0 {
        "isolated_runner_specification_registered_not_authorized_not_run"
    } else if registration_eligible_count > 0 {
        "ready_for_isolated_runner_specification_registration"
    } else {
        "isolated_runner_specification_binding_stale_fail_closed"
    };
    Ok(ControlledShadowExperimentIsolatedRunnerRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_implementations,
        registration_eligible_count,
        runner_count: items.len(),
        current_binding_runner_count,
        first_execution_authorization_review_eligible_count,
        allowed_runner_kinds: vec![
            ControlledShadowExperimentIsolatedRunnerKind::EphemeralDeterministicForwardReplaySpecification,
        ],
        items,
        runner_status: runner_status.to_string(),
        executable_artifact_present: current_binding_runner_count > 0,
        callable_entrypoint_present: false,
        runtime_present: current_binding_runner_count > 0,
        input_mount_present: false,
        shadow_run_authorized: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 78 阶段把 Stage 77 独立批准的零能力实现登记为内容寻址、代码版本与可执行工件摘要均冻结的隔离 runner 规格，并冻结未来只读点时输入、一次性不可信输出、固定非特权身份与资源上限。当前没有 callable entrypoint、挂载、数据访问、影子运行、影子账本、持仓、订单、券商或交易权限；下一步仅可进入独立首次影子执行授权复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterControlledShadowExperimentIsolatedRunnerRequest,
) -> Result<ControlledShadowExperimentIsolatedRunnerRecord, String> {
    let _lock = acquire_lock(state).await?;
    let approved = independently_reviewed_controlled_shadow_experiment_implementations_for_runner_registration(state).await?;
    let value = approved
        .iter()
        .find(|value| value.implementation.implementation_id == request.expected_implementation_id)
        .ok_or_else(|| "当前没有这条绑定有效且已独立批准的受控影子实现".to_string())?;
    validate_expected_binding(value, &request)?;
    let existing = read_runners(state).await?;
    if existing
        .iter()
        .any(|runner| runner.implementation_review.review_id == value.review.review_id)
    {
        return Err("这条 Stage 77 批准已经登记过隔离 runner 规格".to_string());
    }
    let record = new_record(value, registrar_id, &request, Utc::now())?;
    write_immutable_json(
        &runner_root(state).join(format!("{}.json", record.isolated_runner_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn validate_expected_binding(
    approved: &IndependentlyReviewedControlledShadowExperimentImplementation,
    request: &RegisterControlledShadowExperimentIsolatedRunnerRequest,
) -> Result<(), String> {
    let implementation = &approved.implementation;
    let review = &approved.review;
    let contract = &implementation.implementation_contract;
    if request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_review_id != review.review_id
        || request.expected_implementation_review_sha256 != review.review_sha256
        || request.expected_independent_audit_sha256 != review.independent_audit.audit_sha256
        || request.expected_design_review_sha256 != contract.stage_75_design_review_sha256
        || request.expected_design_registration_sha256
            != contract.stage_74_design_registration_sha256
        || request.expected_design_specification_sha256 != contract.design_specification_sha256
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
            "Stage 77 复核、实现、设计、算法、特征或 sealed split 绑定已经变化".to_string(),
        );
    }
    Ok(())
}

fn new_record(
    approved: &IndependentlyReviewedControlledShadowExperimentImplementation,
    registrar_id: &str,
    request: &RegisterControlledShadowExperimentIsolatedRunnerRequest,
    registered_at: DateTime<Utc>,
) -> Result<ControlledShadowExperimentIsolatedRunnerRecord, String> {
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(approved);
    if registrar_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|value| value == registrar_id)
        || !request.registrar_independent_from_stage_77_and_complete_prior_chain_confirmed
    {
        return Err("runner 规格登记人必须独立于 Stage 77 和完整 Stage 51–76 责任链".to_string());
    }
    if !request_checks(request) {
        return Err("必须逐项确认受控影子 runner 规格边界；登记不得授权访问或执行".to_string());
    }
    let runner_name = bounded_required(&request.runner_name, MAX_NAME_CHARS, "runner 名称")?;
    let runner_spec_revision = bounded_required(
        &request.runner_spec_revision,
        MAX_REVISION_CHARS,
        "runner 规格版本",
    )?;
    let runner_code_revision = bounded_required(
        &request.runner_code_revision,
        MAX_REVISION_CHARS,
        "runner 代码版本",
    )?;
    if !valid_sha256(&request.runner_artifact_sha256) {
        return Err("runner 可执行工件 SHA-256 无效".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_TEXT_CHARS, "登记理由")?;
    let known_limitations =
        bounded_required(&request.known_limitations, MAX_TEXT_CHARS, "已知局限")?;
    let future_mount_constraints = bounded_required(
        &request.future_mount_constraints,
        MAX_TEXT_CHARS,
        "未来挂载约束",
    )?;
    let future_output_constraints = bounded_required(
        &request.future_output_constraints,
        MAX_TEXT_CHARS,
        "未来输出约束",
    )?;
    let mut record = ControlledShadowExperimentIsolatedRunnerRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        isolated_runner_id: String::new(),
        isolated_runner_spec_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        implementation: approved.implementation.clone(),
        implementation_review: approved.review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_77_and_complete_prior_chain: true,
        runner_name,
        runner_kind: request.runner_kind.clone(),
        runner_spec_revision: runner_spec_revision.clone(),
        runner_code_revision,
        runner_artifact_sha256: request.runner_artifact_sha256.to_ascii_lowercase(),
        rationale,
        known_limitations,
        future_mount_constraints,
        future_output_constraints,
        runner_contract: runner_contract(approved, &runner_spec_revision)?,
        status: STATUS.to_string(),
        exact_current_stage_51_through_stage_77_binding_confirmed: true,
        registrar_independent_from_stage_77_and_complete_prior_chain_confirmed: true,
        implementation_review_audit_contract_and_design_hashes_reproduced_confirmed: true,
        runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: true,
        no_callable_entrypoint_or_current_mount_confirmed: true,
        future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: true,
        future_create_once_untrusted_independently_validated_output_confirmed: true,
        deterministic_replay_long_only_caps_costs_counterfactuals_and_stop_rules_preserved_confirmed:
            true,
        fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: true,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
        no_model_metric_store_training_feedback_composite_or_reward_confirmed: true,
        no_shadow_run_ledger_position_order_broker_or_trading_confirmed: true,
        registration_only_opens_independent_first_execution_authorization_review_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        first_execution_authorization_review_eligible: true,
        first_execution_authorized: false,
        shadow_run_started: false,
        shadow_run_completed: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
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
    approved: &IndependentlyReviewedControlledShadowExperimentImplementation,
    runner_spec_revision: &str,
) -> Result<ControlledShadowExperimentIsolatedRunnerContract, String> {
    let implementation = &approved.implementation;
    let source = &implementation.implementation_contract;
    let mut contract = ControlledShadowExperimentIsolatedRunnerContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        stage_77_implementation_review_id: approved.review.review_id.clone(),
        stage_77_implementation_review_sha256: approved.review.review_sha256.clone(),
        stage_77_independent_audit_sha256: approved.review.independent_audit.audit_sha256.clone(),
        stage_76_implementation_id: implementation.implementation_id.clone(),
        stage_76_implementation_sha256: implementation.implementation_sha256.clone(),
        stage_76_implementation_contract_sha256: source.contract_sha256.clone(),
        stage_75_design_review_sha256: source.stage_75_design_review_sha256.clone(),
        stage_74_design_registration_sha256: source.stage_74_design_registration_sha256.clone(),
        design_specification_sha256: source.design_specification_sha256.clone(),
        exact_approved_implementation_contract: source.clone(),
        runner_spec_revision: runner_spec_revision.to_string(),
        runtime_identity: RUNTIME_IDENTITY.to_string(),
        runtime_version: RUNTIME_VERSION.to_string(),
        future_input_envelope: FUTURE_INPUT_ENVELOPE.to_string(),
        future_output_envelope: FUTURE_OUTPUT_ENVELOPE.to_string(),
        next_gate: NEXT_GATE.to_string(),
        specification_registered: true,
        executable_artifact_present: true,
        callable_entrypoint_present: false,
        runtime_present: true,
        input_mount_present: false,
        data_access_authorized: false,
        root_filesystem_read_only_required: true,
        ephemeral_working_directory_required: true,
        run_as_unprivileged_required: true,
        no_new_privileges_required: true,
        future_input_read_only_required: true,
        future_input_point_in_time_required: true,
        future_input_content_addressed_required: true,
        future_input_allowlisted_required: true,
        future_output_create_once_required: true,
        future_output_untrusted_required: true,
        future_output_independent_validation_required: true,
        future_output_order_intent_allowed: false,
        future_output_broker_payload_allowed: false,
        environment_inheritance_allowed: false,
        allowed_environment_variables: Vec::new(),
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        model_store_writes_allowed: false,
        metric_store_writes_allowed: false,
        training_feedback_allowed: false,
        scalar_reward_defined: false,
        maximum_parallel_runs: 1,
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
    approved: &IndependentlyReviewedControlledShadowExperimentImplementation,
) -> Vec<String> {
    let mut values = approved.review.excluded_prior_actor_ids.clone();
    values.extend(approved.implementation.excluded_prior_actor_ids.clone());
    values.push(approved.implementation.registered_by.clone());
    values.push(
        approved
            .implementation
            .upstream_design_review
            .reviewer_id
            .clone(),
    );
    values.push(
        approved
            .implementation
            .upstream_design_registration
            .registered_by
            .clone(),
    );
    values.push(approved.review.reviewer_id.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn request_checks(request: &RegisterControlledShadowExperimentIsolatedRunnerRequest) -> bool {
    request.exact_current_stage_51_through_stage_77_binding_confirmed
        && request.registrar_independent_from_stage_77_and_complete_prior_chain_confirmed
        && request.implementation_review_audit_contract_and_design_hashes_reproduced_confirmed
        && request.runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed
        && request.no_callable_entrypoint_or_current_mount_confirmed
        && request.future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed
        && request.future_create_once_untrusted_independently_validated_output_confirmed
        && request.deterministic_replay_long_only_caps_costs_counterfactuals_and_stop_rules_preserved_confirmed
        && request.fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed
        && request.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && request.no_model_metric_store_training_feedback_composite_or_reward_confirmed
        && request.no_shadow_run_ledger_position_order_broker_or_trading_confirmed
        && request.registration_only_opens_independent_first_execution_authorization_review_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn record_checks(record: &ControlledShadowExperimentIsolatedRunnerRecord) -> bool {
    record.exact_current_stage_51_through_stage_77_binding_confirmed
        && record.registrar_independent_from_stage_77_and_complete_prior_chain_confirmed
        && record.implementation_review_audit_contract_and_design_hashes_reproduced_confirmed
        && record.runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed
        && record.no_callable_entrypoint_or_current_mount_confirmed
        && record.future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed
        && record.future_create_once_untrusted_independently_validated_output_confirmed
        && record.deterministic_replay_long_only_caps_costs_counterfactuals_and_stop_rules_preserved_confirmed
        && record.fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed
        && record.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && record.no_model_metric_store_training_feedback_composite_or_reward_confirmed
        && record.no_shadow_run_ledger_position_order_broker_or_trading_confirmed
        && record.registration_only_opens_independent_first_execution_authorization_review_confirmed
        && record.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn authority_closed(record: &ControlledShadowExperimentIsolatedRunnerRecord) -> bool {
    let contract = &record.runner_contract;
    contract.executable_artifact_present
        && !contract.callable_entrypoint_present
        && contract.runtime_present
        && !contract.input_mount_present
        && !contract.data_access_authorized
        && !contract.environment_inheritance_allowed
        && contract.allowed_environment_variables.is_empty()
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.model_store_writes_allowed
        && !contract.metric_store_writes_allowed
        && !contract.training_feedback_allowed
        && !contract.scalar_reward_defined
        && !record.first_execution_authorized
        && !record.shadow_run_started
        && !record.shadow_run_completed
        && !record.shadow_ledger_enabled
        && !record.shadow_position_written
        && !record.model_artifact_store_written
        && !record.metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized
}

fn validate_record(record: &ControlledShadowExperimentIsolatedRunnerRecord) -> Result<(), String> {
    validate_review(&record.implementation_review)?;
    let approved = IndependentlyReviewedControlledShadowExperimentImplementation {
        implementation: record.implementation.clone(),
        review: record.implementation_review.clone(),
    };
    let contract = &record.runner_contract;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.isolated_runner_id)
        || !valid_sha256(&record.isolated_runner_spec_sha256)
        || record.isolated_runner_id != record.isolated_runner_spec_sha256[..32]
        || record.isolated_runner_spec_sha256 != runner_fingerprint(record)?
        || record.implementation_review.implementation != record.implementation
        || !record
            .implementation_review
            .future_isolated_shadow_runner_specification_registration_eligible
        || record.excluded_prior_actor_ids != expected_excluded_prior_actor_ids(&approved)
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record.registered_by.trim().is_empty()
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &record.registered_by)
        || !record.registrar_independent_from_stage_77_and_complete_prior_chain
        || record.runner_name.trim().is_empty()
        || record.runner_name.chars().count() > MAX_NAME_CHARS
        || record.runner_spec_revision.trim().is_empty()
        || record.runner_spec_revision.chars().count() > MAX_REVISION_CHARS
        || record.runner_code_revision.trim().is_empty()
        || record.runner_code_revision.chars().count() > MAX_REVISION_CHARS
        || !valid_sha256(&record.runner_artifact_sha256)
        || record.rationale.trim().is_empty()
        || record.rationale.chars().count() > MAX_TEXT_CHARS
        || record.known_limitations.trim().is_empty()
        || record.known_limitations.chars().count() > MAX_TEXT_CHARS
        || record.future_mount_constraints.trim().is_empty()
        || record.future_mount_constraints.chars().count() > MAX_TEXT_CHARS
        || record.future_output_constraints.trim().is_empty()
        || record.future_output_constraints.chars().count() > MAX_TEXT_CHARS
        || record.runner_contract != runner_contract(&approved, &record.runner_spec_revision)?
        || contract.future_input_envelope != FUTURE_INPUT_ENVELOPE
        || contract.future_output_envelope != FUTURE_OUTPUT_ENVELOPE
        || contract.next_gate != NEXT_GATE
        || !contract.specification_registered
        || !contract.root_filesystem_read_only_required
        || !contract.ephemeral_working_directory_required
        || !contract.run_as_unprivileged_required
        || !contract.no_new_privileges_required
        || !contract.future_input_read_only_required
        || !contract.future_input_point_in_time_required
        || !contract.future_input_content_addressed_required
        || !contract.future_input_allowlisted_required
        || !contract.future_output_create_once_required
        || !contract.future_output_untrusted_required
        || !contract.future_output_independent_validation_required
        || contract.future_output_order_intent_allowed
        || contract.future_output_broker_payload_allowed
        || contract.maximum_parallel_runs != 1
        || contract.maximum_process_count != 1
        || record.status != STATUS
        || !record.first_execution_authorization_review_eligible
        || !record_checks(record)
        || !authority_closed(record)
    {
        return Err(
            "controlled shadow isolated runner specification is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn binding_matches(
    runner: &ControlledShadowExperimentIsolatedRunnerRecord,
    approved: &IndependentlyReviewedControlledShadowExperimentImplementation,
) -> bool {
    runner.implementation == approved.implementation
        && runner.implementation_review == approved.review
        && validate_record(runner).is_ok()
}

fn runner_fingerprint(
    record: &ControlledShadowExperimentIsolatedRunnerRecord,
) -> Result<String, String> {
    fingerprint_without(
        record,
        &["isolated_runner_id", "isolated_runner_spec_sha256"],
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
) -> Result<Vec<ControlledShadowExperimentIsolatedRunnerRecord>, String> {
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
        let runner: ControlledShadowExperimentIsolatedRunnerRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(runner.isolated_runner_id.as_str())
        {
            return Err(
                "controlled shadow isolated runner specification path mismatch".to_string(),
            );
        }
        if !ids.insert(runner.isolated_runner_id.clone())
            || !fingerprints.insert(runner.isolated_runner_spec_sha256.clone())
            || !review_ids.insert(runner.implementation_review.review_id.clone())
        {
            return Err(
                "controlled shadow isolated runner specification registry contains duplicates"
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
    decision_root(state).join("controlled-shadow-experiment-isolated-runner-specifications")
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
    let path = directory.join("controlled-shadow-experiment-isolated-runner-specification.lock");
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
                return Err("另一个管理员正在登记受控影子 runner 规格".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("受控影子 runner 规格登记锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_implementation_reviews::tests::approved_review;

    fn approved() -> IndependentlyReviewedControlledShadowExperimentImplementation {
        let review = approved_review();
        IndependentlyReviewedControlledShadowExperimentImplementation {
            implementation: review.implementation.clone(),
            review,
        }
    }

    fn request(
        approved: &IndependentlyReviewedControlledShadowExperimentImplementation,
    ) -> RegisterControlledShadowExperimentIsolatedRunnerRequest {
        let implementation = &approved.implementation;
        let review = &approved.review;
        let contract = &implementation.implementation_contract;
        RegisterControlledShadowExperimentIsolatedRunnerRequest {
            expected_implementation_id: implementation.implementation_id.clone(),
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_contract_sha256: contract.contract_sha256.clone(),
            expected_implementation_review_id: review.review_id.clone(),
            expected_implementation_review_sha256: review.review_sha256.clone(),
            expected_independent_audit_sha256: review.independent_audit.audit_sha256.clone(),
            expected_design_review_sha256: contract.stage_75_design_review_sha256.clone(),
            expected_design_registration_sha256: contract.stage_74_design_registration_sha256.clone(),
            expected_design_specification_sha256: contract.design_specification_sha256.clone(),
            expected_selected_algorithm_three_seed_binding_sha256: contract.selected_algorithm_three_seed_binding_sha256.clone(),
            expected_sealed_holdout_split_commitment_sha256: contract.sealed_holdout_split_commitment_sha256.clone(),
            expected_feature_order_sha256: contract.feature_order_sha256.clone(),
            expected_preprocessing_sha256: contract.preprocessing_sha256.clone(),
            expected_target_id: contract.target_id.clone(),
            expected_frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id.clone(),
            runner_name: "stage78 controlled shadow isolated runner specification".to_string(),
            runner_kind: ControlledShadowExperimentIsolatedRunnerKind::EphemeralDeterministicForwardReplaySpecification,
            runner_spec_revision: "stage78-spec-revision-v1".to_string(),
            runner_code_revision: "stage78-runner-code-revision-v1".to_string(),
            runner_artifact_sha256: "a".repeat(64),
            rationale: "冻结未来受控影子前向复演 runner 规格，不授权执行".to_string(),
            known_limitations: "当前没有 callable entrypoint、挂载或数据访问".to_string(),
            future_mount_constraints: "未来仅允许内容寻址、白名单、点时、只读输入".to_string(),
            future_output_constraints: "未来输出必须一次写入、不可信、独立验证且不含订单载荷".to_string(),
            exact_current_stage_51_through_stage_77_binding_confirmed: true,
            registrar_independent_from_stage_77_and_complete_prior_chain_confirmed: true,
            implementation_review_audit_contract_and_design_hashes_reproduced_confirmed: true,
            runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed: true,
            no_callable_entrypoint_or_current_mount_confirmed: true,
            future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: true,
            future_create_once_untrusted_independently_validated_output_confirmed: true,
            deterministic_replay_long_only_caps_costs_counterfactuals_and_stop_rules_preserved_confirmed: true,
            fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: true,
            no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_model_metric_store_training_feedback_composite_or_reward_confirmed: true,
            no_shadow_run_ledger_position_order_broker_or_trading_confirmed: true,
            registration_only_opens_independent_first_execution_authorization_review_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    pub(crate) fn record() -> ControlledShadowExperimentIsolatedRunnerRecord {
        let approved = approved();
        let request = request(&approved);
        new_record(&approved, "stage-78-registrar", &request, Utc::now()).unwrap()
    }

    #[test]
    fn registration_is_content_addressed_and_exactly_bound() {
        let value = record();
        assert_eq!(
            value.isolated_runner_id,
            value.isolated_runner_spec_sha256[..32]
        );
        assert_eq!(
            value.runner_contract.exact_approved_implementation_contract,
            value.implementation.implementation_contract
        );
        assert!(validate_record(&value).is_ok());
    }

    #[test]
    fn registration_binds_artifact_and_runtime_but_has_no_entrypoint_mount_or_access() {
        let value = record();
        assert!(value.runner_contract.executable_artifact_present);
        assert!(!value.runner_contract.callable_entrypoint_present);
        assert!(value.runner_contract.runtime_present);
        assert!(!value.runner_contract.input_mount_present);
        assert!(!value.runner_contract.data_access_authorized);
        assert!(valid_sha256(&value.runner_artifact_sha256));
        assert_eq!(value.status, STATUS);
    }

    #[test]
    fn future_mount_output_and_resource_limits_are_frozen() {
        let value = record();
        let contract = value.runner_contract;
        assert!(contract.future_input_read_only_required);
        assert!(contract.future_input_point_in_time_required);
        assert!(contract.future_input_content_addressed_required);
        assert!(contract.future_input_allowlisted_required);
        assert!(contract.future_output_create_once_required);
        assert!(contract.future_output_untrusted_required);
        assert!(contract.future_output_independent_validation_required);
        assert_eq!(contract.maximum_parallel_runs, 1);
        assert_eq!(contract.maximum_process_count, 1);
    }

    #[test]
    fn registrar_must_be_outside_stage_51_through_stage_77_chain() {
        let approved = approved();
        let request = request(&approved);
        assert!(
            new_record(
                &approved,
                &approved.review.reviewer_id,
                &request,
                Utc::now()
            )
            .is_err()
        );
        assert!(
            new_record(
                &approved,
                &approved.implementation.registered_by,
                &request,
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn every_confirmation_is_required() {
        let approved = approved();
        let mut request = request(&approved);
        request.runner_artifact_code_runtime_protocol_and_serialization_immutable_confirmed = false;
        assert!(new_record(&approved, "stage-78-registrar", &request, Utc::now()).is_err());
    }

    #[test]
    fn stale_expected_binding_is_rejected() {
        let approved = approved();
        let mut request = request(&approved);
        request.expected_implementation_review_sha256 = "0".repeat(64);
        assert!(validate_expected_binding(&approved, &request).is_err());
    }

    #[test]
    fn invalid_runner_artifact_digest_is_rejected() {
        let approved = approved();
        let mut request = request(&approved);
        request.runner_artifact_sha256 = "not-a-sha256".to_string();
        assert!(new_record(&approved, "stage-78-registrar", &request, Utc::now()).is_err());
    }

    #[test]
    fn tampering_any_frozen_contract_field_is_rejected() {
        let mut value = record();
        value.runner_contract.outbound_network_allowed = true;
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn only_next_gate_is_independent_first_execution_authorization_review() {
        let value = record();
        assert!(value.first_execution_authorization_review_eligible);
        assert_eq!(value.runner_contract.next_gate, NEXT_GATE);
        assert!(!value.first_execution_authorized);
        assert!(!value.shadow_run_started);
    }

    #[test]
    fn all_model_feedback_shadow_order_broker_and_trading_authority_remains_closed() {
        let value = record();
        assert!(!value.model_artifact_store_written);
        assert!(!value.metric_store_written);
        assert!(!value.training_feedback_authorized);
        assert!(!value.reward_authorized);
        assert!(!value.shadow_ledger_enabled);
        assert!(!value.shadow_position_written);
        assert!(!value.order_generation_authorized);
        assert!(!value.broker_access_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn bounded_text_is_enforced() {
        let approved = approved();
        let mut request = request(&approved);
        request.rationale = "x".repeat(MAX_TEXT_CHARS + 1);
        assert!(new_record(&approved, "stage-78-registrar", &request, Utc::now()).is_err());
    }
}
