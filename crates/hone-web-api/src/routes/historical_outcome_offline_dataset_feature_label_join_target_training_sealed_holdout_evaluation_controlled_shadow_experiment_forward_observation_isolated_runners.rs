//! Stage 86 create-once registration of a content-addressed isolated runner specification for
//! natural forward observation. The record binds an exact Stage 85 approval, executable artifact
//! digest, immutable code revision, runtime identity and future I/O constraints. It creates no
//! callable entrypoint, mount, observation, ledger, position, order, broker or trading ability.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_implementation_reviews::{
    ControlledShadowForwardObservationImplementationReviewRecord,
    IndependentlyApprovedControlledShadowForwardObservationImplementation,
    independently_approved_controlled_shadow_forward_observation_implementations_for_isolated_runner_registration,
    validate_controlled_shadow_forward_observation_implementation_review_for_runner_registration,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_implementations::{
    ControlledShadowForwardObservationImplementationContract,
    ControlledShadowForwardObservationImplementationRecord,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-isolated-runner-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-isolated-runner-specification-v1-artifact-bound";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-isolated-runner-zero-capability-contract-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-forward-observation-isolated-runner-create-once-v1-artifact-bound-no-execution";
const STATUS: &str = "registered_not_authorized_not_run";
const NEXT_GATE: &str = "independent_first_forward_observation_execution_authorization_review";
const RUNTIME_IDENTITY: &str = "hone-natural-forward-observation-runtime";
const RUNTIME_VERSION: &str = "v1-artifact-bound-no-current-mount";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;
const MAXIMUM_MEMORY_MIB: u32 = 2_048;
const MAXIMUM_WALL_CLOCK_SECONDS_PER_CYCLE: u32 = 3_600;
const MAXIMUM_CPU_MILLICORES: u32 = 2_000;
const MAXIMUM_PROCESS_COUNT: u32 = 1;
const MAXIMUM_OUTPUT_BYTES_PER_CYCLE: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowForwardObservationIsolatedRunnerKind {
    EphemeralNaturalForwardObservationSpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowForwardObservationIsolatedRunnerRequest {
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_independent_audit_sha256: String,
    expected_protocol_review_sha256: String,
    expected_protocol_registration_sha256: String,
    expected_protocol_specification_sha256: String,
    expected_design_specification_sha256: String,
    runner_name: String,
    runner_kind: ControlledShadowForwardObservationIsolatedRunnerKind,
    runner_spec_revision: String,
    runner_code_revision: String,
    runner_artifact_sha256: String,
    artifact_reproducibility_procedure: String,
    rationale: String,
    known_limitations: String,
    future_mount_constraints: String,
    future_output_constraints: String,
    exact_current_stage_51_through_stage_85_binding_confirmed: bool,
    registrar_independent_from_stage_85_and_complete_prior_chain_confirmed: bool,
    implementation_review_audit_contract_protocol_and_design_hashes_reproduced_confirmed: bool,
    executable_artifact_digest_code_revision_and_reproduction_procedure_bound_confirmed: bool,
    no_callable_entrypoint_or_current_mount_confirmed: bool,
    natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: bool,
    weekly_claim_first_create_once_official_calendar_and_spy_sync_preserved_confirmed: bool,
    future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: bool,
    corporate_action_evidence_and_append_only_corrections_preserved_confirmed: bool,
    future_create_once_untrusted_independently_validated_output_without_order_intent_confirmed:
        bool,
    deterministic_signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_preserved_confirmed:
        bool,
    fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: bool,
    no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    no_observation_ledger_position_performance_model_metric_training_feedback_reward_order_broker_or_trading_confirmed:
        bool,
    registration_only_opens_independent_first_execution_authorization_review_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationIsolatedRunnerContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub stage_85_implementation_review_id: String,
    pub stage_85_implementation_review_sha256: String,
    pub stage_85_independent_audit_sha256: String,
    pub stage_84_implementation_id: String,
    pub stage_84_implementation_sha256: String,
    pub stage_84_implementation_contract_sha256: String,
    pub stage_83_protocol_review_sha256: String,
    pub stage_82_protocol_registration_sha256: String,
    pub stage_82_protocol_specification_sha256: String,
    pub stage_74_design_specification_sha256: String,
    pub exact_approved_implementation_contract:
        ControlledShadowForwardObservationImplementationContract,
    pub runner_spec_revision: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub future_input_manifest_schema: String,
    pub future_cycle_claim_schema: String,
    pub future_untrusted_observation_schema: String,
    pub next_gate: String,
    pub specification_registered: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_identity_bound: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub market_data_adapter_present: bool,
    pub data_access_authorized: bool,
    pub root_filesystem_read_only_required: bool,
    pub ephemeral_working_directory_required: bool,
    pub run_as_unprivileged_required: bool,
    pub no_new_privileges_required: bool,
    pub future_input_read_only_required: bool,
    pub future_input_point_in_time_required: bool,
    pub future_input_content_addressed_required: bool,
    pub future_input_allowlisted_required: bool,
    pub future_cycle_claim_first_required: bool,
    pub future_cycle_create_once_required: bool,
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
    pub observation_claim_writes_allowed: bool,
    pub forward_observation_writes_allowed: bool,
    pub ledger_creation_allowed: bool,
    pub position_writes_allowed: bool,
    pub performance_metric_writes_allowed: bool,
    pub model_store_writes_allowed: bool,
    pub metric_store_writes_allowed: bool,
    pub training_feedback_allowed: bool,
    pub scalar_reward_defined: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
    pub maximum_parallel_runs: u32,
    pub maximum_memory_mib: u32,
    pub maximum_wall_clock_seconds_per_cycle: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u32,
    pub maximum_output_bytes_per_cycle: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationIsolatedRunnerRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub implementation: ControlledShadowForwardObservationImplementationRecord,
    pub implementation_review: ControlledShadowForwardObservationImplementationReviewRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_85_and_complete_prior_chain: bool,
    pub runner_name: String,
    pub runner_kind: ControlledShadowForwardObservationIsolatedRunnerKind,
    pub runner_spec_revision: String,
    pub runner_code_revision: String,
    pub runner_artifact_sha256: String,
    pub artifact_reproducibility_procedure: String,
    pub rationale: String,
    pub known_limitations: String,
    pub future_mount_constraints: String,
    pub future_output_constraints: String,
    pub runner_contract: ControlledShadowForwardObservationIsolatedRunnerContract,
    pub status: String,
    pub exact_current_stage_51_through_stage_85_binding_confirmed: bool,
    pub registrar_independent_from_stage_85_and_complete_prior_chain_confirmed: bool,
    pub implementation_review_audit_contract_protocol_and_design_hashes_reproduced_confirmed: bool,
    pub executable_artifact_digest_code_revision_and_reproduction_procedure_bound_confirmed: bool,
    pub no_callable_entrypoint_or_current_mount_confirmed: bool,
    pub natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: bool,
    pub weekly_claim_first_create_once_official_calendar_and_spy_sync_preserved_confirmed: bool,
    pub future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: bool,
    pub corporate_action_evidence_and_append_only_corrections_preserved_confirmed: bool,
    pub future_create_once_untrusted_independently_validated_output_without_order_intent_confirmed:
        bool,
    pub deterministic_signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_preserved_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: bool,
    pub no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    pub no_observation_ledger_position_performance_model_metric_training_feedback_reward_order_broker_or_trading_confirmed:
        bool,
    pub registration_only_opens_independent_first_execution_authorization_review_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub first_execution_authorization_review_eligible: bool,
    pub first_execution_authorized: bool,
    pub forward_observation_started: bool,
    pub forward_observation_completed: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationIsolatedRunnerItem {
    pub runner: ControlledShadowForwardObservationIsolatedRunnerRecord,
    pub approved_review_binding_current: bool,
    pub first_execution_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationIsolatedRunnerRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_implementations:
        Vec<IndependentlyApprovedControlledShadowForwardObservationImplementation>,
    pub registration_eligible_count: usize,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub allowed_runner_kinds: Vec<ControlledShadowForwardObservationIsolatedRunnerKind>,
    pub items: Vec<ControlledShadowForwardObservationIsolatedRunnerItem>,
    pub runner_status: String,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub forward_observation_authorized: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowForwardObservationIsolatedRunnerReadinessSummary {
    pub registration_eligible_count: usize,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_forward_observation_isolated_runners(
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
            warn!(%error, "forward-observation isolated runner registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "前向观察隔离 runner 规格登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_forward_observation_isolated_runner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowForwardObservationIsolatedRunnerRequest>,
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

pub(crate) async fn controlled_shadow_forward_observation_isolated_runner_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationIsolatedRunnerReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowForwardObservationIsolatedRunnerReadinessSummary {
            registration_eligible_count: value.registration_eligible_count,
            runner_count: value.runner_count,
            current_binding_runner_count: value.current_binding_runner_count,
            first_execution_authorization_review_eligible_count: value
                .first_execution_authorization_review_eligible_count,
            runner_status: value.runner_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn controlled_shadow_forward_observation_runners_for_first_execution_authorization_review(
    state: &AppState,
) -> Result<Vec<ControlledShadowForwardObservationIsolatedRunnerRecord>, String> {
    let approved = independently_approved_controlled_shadow_forward_observation_implementations_for_isolated_runner_registration(state).await?;
    let mut runners = read_runners(state)
        .await?
        .into_iter()
        .filter(|runner| {
            approved.iter().any(|value| binding_matches(runner, value))
                && runner.first_execution_authorization_review_eligible
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
) -> Result<ControlledShadowForwardObservationIsolatedRunnerRegistry, String> {
    let approved = independently_approved_controlled_shadow_forward_observation_implementations_for_isolated_runner_registration(state).await?;
    let runners = read_runners(state).await?;
    let mut items = runners
        .into_iter()
        .map(|runner| {
            let current = approved.iter().any(|value| binding_matches(&runner, value));
            ControlledShadowForwardObservationIsolatedRunnerItem {
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
    let eligible_implementations = approved
        .iter()
        .filter(|value| !registered_review_ids.contains(value.review.review_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let registration_eligible_count = eligible_implementations.len();
    let current_binding_runner_count = items
        .iter()
        .filter(|item| item.approved_review_binding_current)
        .count();
    let first_execution_authorization_review_eligible_count = items
        .iter()
        .filter(|item| item.first_execution_authorization_review_eligible)
        .count();
    let runner_status = if approved.is_empty() {
        "waiting_stage_85_independently_approved_forward_observation_implementation"
    } else if first_execution_authorization_review_eligible_count > 0 {
        "forward_observation_isolated_runner_registered_not_authorized_not_run"
    } else if registration_eligible_count > 0 {
        "ready_for_forward_observation_isolated_runner_specification_registration"
    } else {
        "forward_observation_isolated_runner_binding_stale_fail_closed"
    };
    Ok(ControlledShadowForwardObservationIsolatedRunnerRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_implementations,
        registration_eligible_count,
        runner_count: items.len(),
        current_binding_runner_count,
        first_execution_authorization_review_eligible_count,
        allowed_runner_kinds: vec![
            ControlledShadowForwardObservationIsolatedRunnerKind::EphemeralNaturalForwardObservationSpecification,
        ],
        items,
        runner_status: runner_status.to_string(),
        executable_artifact_present: current_binding_runner_count > 0,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        input_mount_present: false,
        forward_observation_authorized: false,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 86 把 Stage 85 独立批准绑定到内容寻址可执行工件摘要、不可变代码版本、复现步骤、固定 runtime 身份、未来点时只读输入和 create-once 不可信输出。工件身份已冻结，但 runtime 未实例化，且没有入口、挂载、数据访问、观察、账本、持仓、绩效、订单、券商或交易权限；下一步只可进入 Stage 87 首次执行授权独立复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterControlledShadowForwardObservationIsolatedRunnerRequest,
) -> Result<ControlledShadowForwardObservationIsolatedRunnerRecord, String> {
    let _lock = acquire_lock(state).await?;
    let approved = independently_approved_controlled_shadow_forward_observation_implementations_for_isolated_runner_registration(state).await?;
    let value = approved
        .iter()
        .find(|value| value.implementation.implementation_id == request.expected_implementation_id)
        .ok_or_else(|| "当前没有这条绑定有效且已独立批准的 Stage 84 前向观察实现".to_string())?;
    validate_expected_binding(value, &request)?;
    let existing = read_runners(state).await?;
    if existing
        .iter()
        .any(|runner| runner.implementation_review.review_id == value.review.review_id)
    {
        return Err("这条 Stage 85 批准已经登记过隔离 runner 规格".to_string());
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
    approved: &IndependentlyApprovedControlledShadowForwardObservationImplementation,
    request: &RegisterControlledShadowForwardObservationIsolatedRunnerRequest,
) -> Result<(), String> {
    let implementation = &approved.implementation;
    let review = &approved.review;
    let contract = &implementation.implementation_contract;
    let registration = &implementation.upstream_protocol_registration;
    let protocol = &registration.protocol_specification;
    if request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_review_id != review.review_id
        || request.expected_implementation_review_sha256 != review.review_sha256
        || request.expected_independent_audit_sha256 != review.independent_audit.audit_sha256
        || request.expected_protocol_review_sha256
            != implementation.upstream_protocol_review.review_sha256
        || request.expected_protocol_registration_sha256
            != registration.protocol_registration_sha256
        || request.expected_protocol_specification_sha256 != protocol.specification_sha256
        || request.expected_design_specification_sha256
            != protocol.exact_design_specification.specification_sha256
    {
        return Err("Stage 85 复核、Stage 84 实现、协议、设计或独立审计绑定已经变化".to_string());
    }
    Ok(())
}

fn new_record(
    approved: &IndependentlyApprovedControlledShadowForwardObservationImplementation,
    registrar_id: &str,
    request: &RegisterControlledShadowForwardObservationIsolatedRunnerRequest,
    registered_at: DateTime<Utc>,
) -> Result<ControlledShadowForwardObservationIsolatedRunnerRecord, String> {
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(approved);
    if registrar_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|value| value == registrar_id)
        || !request.registrar_independent_from_stage_85_and_complete_prior_chain_confirmed
    {
        return Err("runner 规格登记人必须独立于 Stage 85 和完整 Stage 51–85 责任链".to_string());
    }
    if !request_checks(request) {
        return Err("必须逐项确认 Stage 86 runner 规格、自然前向与零权限边界".to_string());
    }
    if !valid_sha256(&request.runner_artifact_sha256) {
        return Err("runner 可执行工件 SHA-256 无效".to_string());
    }
    let runner_name = bounded_required(&request.runner_name, MAX_NAME_CHARS, "runner 名称")?;
    let runner_spec_revision = bounded_required(
        &request.runner_spec_revision,
        MAX_REVISION_CHARS,
        "runner 规格版本",
    )?;
    let mut record = ControlledShadowForwardObservationIsolatedRunnerRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        isolated_runner_id: String::new(),
        isolated_runner_spec_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        implementation: approved.implementation.clone(),
        implementation_review: approved.review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_85_and_complete_prior_chain: true,
        runner_name,
        runner_kind: request.runner_kind.clone(),
        runner_spec_revision: runner_spec_revision.clone(),
        runner_code_revision: bounded_required(
            &request.runner_code_revision,
            MAX_REVISION_CHARS,
            "runner 代码版本",
        )?,
        runner_artifact_sha256: request.runner_artifact_sha256.to_ascii_lowercase(),
        artifact_reproducibility_procedure: bounded_required(
            &request.artifact_reproducibility_procedure,
            MAX_TEXT_CHARS,
            "工件复现步骤",
        )?,
        rationale: bounded_required(&request.rationale, MAX_TEXT_CHARS, "登记理由")?,
        known_limitations: bounded_required(
            &request.known_limitations,
            MAX_TEXT_CHARS,
            "已知局限",
        )?,
        future_mount_constraints: bounded_required(
            &request.future_mount_constraints,
            MAX_TEXT_CHARS,
            "未来挂载约束",
        )?,
        future_output_constraints: bounded_required(
            &request.future_output_constraints,
            MAX_TEXT_CHARS,
            "未来输出约束",
        )?,
        runner_contract: runner_contract(approved, &runner_spec_revision)?,
        status: STATUS.to_string(),
        exact_current_stage_51_through_stage_85_binding_confirmed: true,
        registrar_independent_from_stage_85_and_complete_prior_chain_confirmed: true,
        implementation_review_audit_contract_protocol_and_design_hashes_reproduced_confirmed: true,
        executable_artifact_digest_code_revision_and_reproduction_procedure_bound_confirmed: true,
        no_callable_entrypoint_or_current_mount_confirmed: true,
        natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: true,
        weekly_claim_first_create_once_official_calendar_and_spy_sync_preserved_confirmed: true,
        future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: true,
        corporate_action_evidence_and_append_only_corrections_preserved_confirmed: true,
        future_create_once_untrusted_independently_validated_output_without_order_intent_confirmed: true,
        deterministic_signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_preserved_confirmed: true,
        fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: true,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
        no_observation_ledger_position_performance_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
        registration_only_opens_independent_first_execution_authorization_review_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        first_execution_authorization_review_eligible: true,
        first_execution_authorized: false,
        forward_observation_started: false,
        forward_observation_completed: false,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
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
    validate_controlled_shadow_forward_observation_isolated_runner_for_review(&record)?;
    Ok(record)
}

fn runner_contract(
    approved: &IndependentlyApprovedControlledShadowForwardObservationImplementation,
    runner_spec_revision: &str,
) -> Result<ControlledShadowForwardObservationIsolatedRunnerContract, String> {
    let implementation = &approved.implementation;
    let source = &implementation.implementation_contract;
    let registration = &implementation.upstream_protocol_registration;
    let protocol = &registration.protocol_specification;
    let mut contract = ControlledShadowForwardObservationIsolatedRunnerContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        stage_85_implementation_review_id: approved.review.review_id.clone(),
        stage_85_implementation_review_sha256: approved.review.review_sha256.clone(),
        stage_85_independent_audit_sha256: approved.review.independent_audit.audit_sha256.clone(),
        stage_84_implementation_id: implementation.implementation_id.clone(),
        stage_84_implementation_sha256: implementation.implementation_sha256.clone(),
        stage_84_implementation_contract_sha256: source.contract_sha256.clone(),
        stage_83_protocol_review_sha256: implementation
            .upstream_protocol_review
            .review_sha256
            .clone(),
        stage_82_protocol_registration_sha256: registration.protocol_registration_sha256.clone(),
        stage_82_protocol_specification_sha256: protocol.specification_sha256.clone(),
        stage_74_design_specification_sha256: protocol
            .exact_design_specification
            .specification_sha256
            .clone(),
        exact_approved_implementation_contract: source.clone(),
        runner_spec_revision: runner_spec_revision.to_string(),
        runtime_identity: RUNTIME_IDENTITY.to_string(),
        runtime_version: RUNTIME_VERSION.to_string(),
        future_input_manifest_schema: source.canonical_future_input_manifest_schema.clone(),
        future_cycle_claim_schema: source.canonical_future_cycle_claim_schema.clone(),
        future_untrusted_observation_schema: source
            .canonical_future_untrusted_observation_schema
            .clone(),
        next_gate: NEXT_GATE.to_string(),
        specification_registered: true,
        executable_artifact_present: true,
        callable_entrypoint_present: false,
        runtime_identity_bound: true,
        runtime_instantiated: false,
        input_mount_present: false,
        market_data_adapter_present: false,
        data_access_authorized: false,
        root_filesystem_read_only_required: true,
        ephemeral_working_directory_required: true,
        run_as_unprivileged_required: true,
        no_new_privileges_required: true,
        future_input_read_only_required: true,
        future_input_point_in_time_required: true,
        future_input_content_addressed_required: true,
        future_input_allowlisted_required: true,
        future_cycle_claim_first_required: true,
        future_cycle_create_once_required: true,
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
        observation_claim_writes_allowed: false,
        forward_observation_writes_allowed: false,
        ledger_creation_allowed: false,
        position_writes_allowed: false,
        performance_metric_writes_allowed: false,
        model_store_writes_allowed: false,
        metric_store_writes_allowed: false,
        training_feedback_allowed: false,
        scalar_reward_defined: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
        maximum_parallel_runs: 1,
        maximum_memory_mib: MAXIMUM_MEMORY_MIB,
        maximum_wall_clock_seconds_per_cycle: MAXIMUM_WALL_CLOCK_SECONDS_PER_CYCLE,
        maximum_cpu_millicores: MAXIMUM_CPU_MILLICORES,
        maximum_process_count: MAXIMUM_PROCESS_COUNT,
        maximum_output_bytes_per_cycle: MAXIMUM_OUTPUT_BYTES_PER_CYCLE,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn expected_excluded_prior_actor_ids(
    approved: &IndependentlyApprovedControlledShadowForwardObservationImplementation,
) -> Vec<String> {
    let mut values = approved.review.excluded_prior_actor_ids.clone();
    values.extend(approved.implementation.excluded_prior_actor_ids.clone());
    values.push(approved.implementation.registered_by.clone());
    values.push(approved.review.reviewer_id.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn request_checks(
    request: &RegisterControlledShadowForwardObservationIsolatedRunnerRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_85_binding_confirmed
        && request.registrar_independent_from_stage_85_and_complete_prior_chain_confirmed
        && request.implementation_review_audit_contract_protocol_and_design_hashes_reproduced_confirmed
        && request.executable_artifact_digest_code_revision_and_reproduction_procedure_bound_confirmed
        && request.no_callable_entrypoint_or_current_mount_confirmed
        && request.natural_forward_no_backfill_and_observation_not_before_preserved_confirmed
        && request.weekly_claim_first_create_once_official_calendar_and_spy_sync_preserved_confirmed
        && request.future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed
        && request.corporate_action_evidence_and_append_only_corrections_preserved_confirmed
        && request.future_create_once_untrusted_independently_validated_output_without_order_intent_confirmed
        && request.deterministic_signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_preserved_confirmed
        && request.fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed
        && request.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && request.no_observation_ledger_position_performance_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && request.registration_only_opens_independent_first_execution_authorization_review_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn record_checks(record: &ControlledShadowForwardObservationIsolatedRunnerRecord) -> bool {
    record.exact_current_stage_51_through_stage_85_binding_confirmed
        && record.registrar_independent_from_stage_85_and_complete_prior_chain_confirmed
        && record.implementation_review_audit_contract_protocol_and_design_hashes_reproduced_confirmed
        && record.executable_artifact_digest_code_revision_and_reproduction_procedure_bound_confirmed
        && record.no_callable_entrypoint_or_current_mount_confirmed
        && record.natural_forward_no_backfill_and_observation_not_before_preserved_confirmed
        && record.weekly_claim_first_create_once_official_calendar_and_spy_sync_preserved_confirmed
        && record.future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed
        && record.corporate_action_evidence_and_append_only_corrections_preserved_confirmed
        && record.future_create_once_untrusted_independently_validated_output_without_order_intent_confirmed
        && record.deterministic_signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_preserved_confirmed
        && record.fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed
        && record.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && record.no_observation_ledger_position_performance_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && record.registration_only_opens_independent_first_execution_authorization_review_confirmed
        && record.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn authority_closed(record: &ControlledShadowForwardObservationIsolatedRunnerRecord) -> bool {
    let contract = &record.runner_contract;
    contract.executable_artifact_present
        && !contract.callable_entrypoint_present
        && contract.runtime_identity_bound
        && !contract.runtime_instantiated
        && !contract.input_mount_present
        && !contract.market_data_adapter_present
        && !contract.data_access_authorized
        && !contract.environment_inheritance_allowed
        && contract.allowed_environment_variables.is_empty()
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.observation_claim_writes_allowed
        && !contract.forward_observation_writes_allowed
        && !contract.ledger_creation_allowed
        && !contract.position_writes_allowed
        && !contract.performance_metric_writes_allowed
        && !contract.model_store_writes_allowed
        && !contract.metric_store_writes_allowed
        && !contract.training_feedback_allowed
        && !contract.scalar_reward_defined
        && !contract.order_generation_allowed
        && !contract.broker_access_allowed
        && !contract.trading_allowed
        && !record.first_execution_authorized
        && !record.forward_observation_started
        && !record.forward_observation_completed
        && !record.forward_observation_ledger_created
        && !record.shadow_position_written
        && !record.performance_metric_written
        && !record.model_artifact_store_written
        && !record.metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized
}

pub(crate) fn validate_controlled_shadow_forward_observation_isolated_runner_for_review(
    record: &ControlledShadowForwardObservationIsolatedRunnerRecord,
) -> Result<(), String> {
    validate_controlled_shadow_forward_observation_implementation_review_for_runner_registration(
        &record.implementation_review,
    )?;
    let approved = IndependentlyApprovedControlledShadowForwardObservationImplementation {
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
            .future_isolated_forward_observation_runner_specification_registration_eligible
        || record.excluded_prior_actor_ids != expected_excluded_prior_actor_ids(&approved)
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record.registered_by.trim().is_empty()
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &record.registered_by)
        || !record.registrar_independent_from_stage_85_and_complete_prior_chain
        || !valid_sha256(&record.runner_artifact_sha256)
        || !bounded_fields_valid(record)
        || record.runner_contract != runner_contract(&approved, &record.runner_spec_revision)?
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
        || !contract.future_cycle_claim_first_required
        || !contract.future_cycle_create_once_required
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
        return Err("Stage 86 前向观察隔离 runner 规格无效、漂移或越权".to_string());
    }
    Ok(())
}

fn bounded_fields_valid(record: &ControlledShadowForwardObservationIsolatedRunnerRecord) -> bool {
    let revisions = [&record.runner_spec_revision, &record.runner_code_revision];
    let texts = [
        &record.artifact_reproducibility_procedure,
        &record.rationale,
        &record.known_limitations,
        &record.future_mount_constraints,
        &record.future_output_constraints,
    ];
    !record.runner_name.trim().is_empty()
        && record.runner_name.chars().count() <= MAX_NAME_CHARS
        && revisions
            .iter()
            .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_REVISION_CHARS)
        && texts
            .iter()
            .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS)
}

fn binding_matches(
    runner: &ControlledShadowForwardObservationIsolatedRunnerRecord,
    approved: &IndependentlyApprovedControlledShadowForwardObservationImplementation,
) -> bool {
    runner.implementation == approved.implementation
        && runner.implementation_review == approved.review
        && validate_controlled_shadow_forward_observation_isolated_runner_for_review(runner).is_ok()
}

fn runner_fingerprint(
    record: &ControlledShadowForwardObservationIsolatedRunnerRecord,
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
        .ok_or_else(|| "Stage 86 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

async fn read_runners(
    state: &AppState,
) -> Result<Vec<ControlledShadowForwardObservationIsolatedRunnerRecord>, String> {
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
        let runner: ControlledShadowForwardObservationIsolatedRunnerRecord =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_controlled_shadow_forward_observation_isolated_runner_for_review(&runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(runner.isolated_runner_id.as_str())
            || !ids.insert(runner.isolated_runner_id.clone())
            || !fingerprints.insert(runner.isolated_runner_spec_sha256.clone())
            || !review_ids.insert(runner.implementation_review.review_id.clone())
        {
            return Err("Stage 86 runner 规格路径、ID 或 Stage 85 绑定重复".to_string());
        }
        runners.push(runner);
    }
    Ok(runners)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn runner_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-forward-observation-isolated-runners")
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
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
    let path = directory.join("controlled-shadow-forward-observation-isolated-runner.lock");
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|value| SystemTime::now().duration_since(value).ok())
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
        .map_err(|_| "已有 Stage 86 runner 规格登记正在进行".to_string())?;
    Ok(RegistrationLock { path })
}

fn bounded_required(value: &str, limit: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > limit {
        Err(format!("{label}不能为空且不得超过 {limit} 字"))
    } else {
        Ok(value.to_string())
    }
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterControlledShadowForwardObservationIsolatedRunnerRequest {
        RegisterControlledShadowForwardObservationIsolatedRunnerRequest {
            expected_implementation_id: "a".repeat(32),
            expected_implementation_sha256: "a".repeat(64),
            expected_implementation_contract_sha256: "b".repeat(64),
            expected_implementation_review_id: "c".repeat(32),
            expected_implementation_review_sha256: "c".repeat(64),
            expected_independent_audit_sha256: "d".repeat(64),
            expected_protocol_review_sha256: "e".repeat(64),
            expected_protocol_registration_sha256: "f".repeat(64),
            expected_protocol_specification_sha256: "1".repeat(64),
            expected_design_specification_sha256: "2".repeat(64),
            runner_name: "自然前向观察隔离 runner 规格".to_string(),
            runner_kind: ControlledShadowForwardObservationIsolatedRunnerKind::EphemeralNaturalForwardObservationSpecification,
            runner_spec_revision: "forward-observation-runner-spec-v1".to_string(),
            runner_code_revision: "revision-1".to_string(),
            runner_artifact_sha256: "3".repeat(64),
            artifact_reproducibility_procedure: "固定工具链重新构建并逐位核对摘要。".to_string(),
            rationale: "绑定未来自然前向观察工件。".to_string(),
            known_limitations: "尚无真实自然前向样本。".to_string(),
            future_mount_constraints: "授权后仍只允许点时只读挂载。".to_string(),
            future_output_constraints: "输出创建一次且不得含订单意图。".to_string(),
            exact_current_stage_51_through_stage_85_binding_confirmed: true,
            registrar_independent_from_stage_85_and_complete_prior_chain_confirmed: true,
            implementation_review_audit_contract_protocol_and_design_hashes_reproduced_confirmed: true,
            executable_artifact_digest_code_revision_and_reproduction_procedure_bound_confirmed: true,
            no_callable_entrypoint_or_current_mount_confirmed: true,
            natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: true,
            weekly_claim_first_create_once_official_calendar_and_spy_sync_preserved_confirmed: true,
            future_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: true,
            corporate_action_evidence_and_append_only_corrections_preserved_confirmed: true,
            future_create_once_untrusted_independently_validated_output_without_order_intent_confirmed: true,
            deterministic_signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_preserved_confirmed: true,
            fixed_unprivileged_identity_read_only_root_and_bounded_resources_confirmed: true,
            no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_observation_ledger_position_performance_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
            registration_only_opens_independent_first_execution_authorization_review_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn registration_requires_every_confirmation() {
        let mut value = request();
        assert!(request_checks(&value));
        value.natural_forward_no_backfill_and_observation_not_before_preserved_confirmed = false;
        assert!(!request_checks(&value));
    }

    #[test]
    fn artifact_digest_and_text_bounds_fail_closed() {
        assert!(valid_sha256(&"a".repeat(64)));
        assert!(!valid_sha256("not-a-sha256"));
        assert!(bounded_required("", 10, "说明").is_err());
        assert!(bounded_required("有效说明", 10, "说明").is_ok());
    }

    #[test]
    fn fingerprint_binds_runner_artifact_and_forward_semantics() {
        #[derive(Serialize)]
        struct Fixture {
            isolated_runner_id: String,
            isolated_runner_spec_sha256: String,
            artifact: String,
            semantics: String,
        }
        let value = Fixture {
            isolated_runner_id: String::new(),
            isolated_runner_spec_sha256: String::new(),
            artifact: "a".repeat(64),
            semantics: "natural-forward".to_string(),
        };
        let before = fingerprint_without(
            &value,
            &["isolated_runner_id", "isolated_runner_spec_sha256"],
        )
        .unwrap();
        let changed = Fixture {
            semantics: "backfill".to_string(),
            ..value
        };
        assert_ne!(
            before,
            fingerprint_without(
                &changed,
                &["isolated_runner_id", "isolated_runner_spec_sha256"]
            )
            .unwrap()
        );
    }
}
