//! Stage 99 create-once registration of an isolated market-data parser runner specification.
//!
//! The record freezes one exact Stage 98 approval, a proposed future artifact identity, immutable
//! code revision, fixed unprivileged runtime contract, read-only content-addressed input and
//! create-once untrusted output. It does not create parser source, an executable artifact, a
//! callable entrypoint, runtime, raw-payload mount/read, parsed rows, observation, ledger,
//! position, performance, training, reward, order, broker or trading capability.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_implementation_reviews::{
    ControlledShadowMarketDataParserImplementationReviewRecord,
    IndependentlyApprovedControlledShadowMarketDataParserImplementation,
    independently_approved_market_data_parser_implementations_for_isolated_runner_registration,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_implementations::{
    ControlledShadowMarketDataParserImplementationContract,
    ControlledShadowMarketDataParserImplementationRecord,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-isolated-runner-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-isolated-runner-specification-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-isolated-runner-zero-capability-contract-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-isolated-runner-create-once-v1-no-execution";
const STATUS: &str = "registered_not_authorized_not_run";
const NEXT_GATE: &str = "chain_external_first_market_data_parser_execution_authorization_review";
const RUNTIME_IDENTITY: &str = "hone-natural-forward-market-data-parser-runtime";
const RUNTIME_VERSION: &str = "v1-proposed-artifact-not-present-no-current-mount";
const FUTURE_INPUT_ENVELOPE: &str =
    "stage_94_validated_raw_market_data_receipt_payloads_v1_read_only_content_addressed";
const FUTURE_OUTPUT_ENVELOPE: &str = "canonical_market_data_parse_result_v1_create_once_untrusted";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;
const MAXIMUM_MEMORY_MIB: u32 = 1_024;
const MAXIMUM_WALL_CLOCK_SECONDS: u32 = 300;
const MAXIMUM_CPU_MILLICORES: u32 = 1_000;
const MAXIMUM_PROCESS_COUNT: u32 = 1;
const MAXIMUM_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowMarketDataParserIsolatedRunnerKind {
    EphemeralDeterministicMarketDataParserSpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowMarketDataParserIsolatedRunnerRequest {
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_independent_audit_sha256: String,
    expected_specification_review_sha256: String,
    expected_specification_registration_sha256: String,
    expected_parser_specification_sha256: String,
    expected_validation_sha256: String,
    expected_receipt_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    runner_name: String,
    runner_kind: ControlledShadowMarketDataParserIsolatedRunnerKind,
    runner_spec_revision: String,
    proposed_runner_code_revision: String,
    proposed_runner_artifact_sha256: String,
    artifact_reproduction_procedure: String,
    rationale: String,
    known_limitations: String,
    future_input_constraints: String,
    future_output_constraints: String,
    exact_current_stage_51_through_stage_98_binding_confirmed: bool,
    registrar_independent_from_stage_98_and_complete_prior_chain_confirmed: bool,
    implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed: bool,
    proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed:
        bool,
    all_eight_parser_functions_and_canonical_schemas_preserved_confirmed: bool,
    future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed:
        bool,
    strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: bool,
    no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
        bool,
    future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    source_available_at_remains_unverified_until_separate_evidence_confirmed: bool,
    fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed:
        bool,
    no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    registration_only_opens_chain_external_first_execution_authorization_review_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserIsolatedRunnerContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub stage_98_implementation_review_id: String,
    pub stage_98_implementation_review_sha256: String,
    pub stage_98_independent_audit_sha256: String,
    pub stage_97_implementation_id: String,
    pub stage_97_implementation_sha256: String,
    pub stage_97_implementation_contract_sha256: String,
    pub stage_96_specification_review_sha256: String,
    pub stage_95_specification_registration_sha256: String,
    pub stage_95_parser_specification_sha256: String,
    pub stage_94_validation_sha256: String,
    pub stage_93_receipt_sha256: String,
    pub stage_93_claim_sha256: String,
    pub stage_93_result_sha256: String,
    pub exact_approved_implementation_contract:
        ControlledShadowMarketDataParserImplementationContract,
    pub runner_spec_revision: String,
    pub proposed_runner_code_revision: String,
    pub proposed_runner_artifact_sha256: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub future_input_envelope: String,
    pub future_output_envelope: String,
    pub next_gate: String,
    pub specification_registered: bool,
    pub future_runner_artifact_identity_bound: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_identity_bound: bool,
    pub runtime_instantiated: bool,
    pub raw_payload_mount_present: bool,
    pub raw_payload_read_allowed: bool,
    pub data_access_authorized: bool,
    pub root_filesystem_read_only_required: bool,
    pub ephemeral_working_directory_required: bool,
    pub run_as_unprivileged_required: bool,
    pub no_new_privileges_required: bool,
    pub future_input_read_only_required: bool,
    pub future_input_content_addressed_required: bool,
    pub future_input_stage_94_validation_required: bool,
    pub future_output_create_once_required: bool,
    pub future_output_untrusted_required: bool,
    pub future_output_independent_validation_required: bool,
    pub future_output_market_interpretation_allowed: bool,
    pub future_output_order_intent_allowed: bool,
    pub environment_inheritance_allowed: bool,
    pub allowed_environment_variables: Vec<String>,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub parsed_row_writes_allowed: bool,
    pub observation_writes_allowed: bool,
    pub ledger_creation_allowed: bool,
    pub position_writes_allowed: bool,
    pub performance_metric_writes_allowed: bool,
    pub model_store_writes_allowed: bool,
    pub metric_store_writes_allowed: bool,
    pub training_feedback_allowed: bool,
    pub reward_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
    pub maximum_parallel_runs: u32,
    pub maximum_memory_mib: u32,
    pub maximum_wall_clock_seconds: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u32,
    pub maximum_output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserIsolatedRunnerRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub implementation: ControlledShadowMarketDataParserImplementationRecord,
    pub implementation_review: ControlledShadowMarketDataParserImplementationReviewRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub runner_name: String,
    pub runner_kind: ControlledShadowMarketDataParserIsolatedRunnerKind,
    pub artifact_reproduction_procedure: String,
    pub rationale: String,
    pub known_limitations: String,
    pub future_input_constraints: String,
    pub future_output_constraints: String,
    pub runner_contract: ControlledShadowMarketDataParserIsolatedRunnerContract,
    pub status: String,
    pub confirmations_complete: bool,
    pub exact_current_stage_51_through_stage_98_binding_confirmed: bool,
    pub registrar_independent_from_stage_98_and_complete_prior_chain_confirmed: bool,
    pub implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed:
        bool,
    pub proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed:
        bool,
    pub all_eight_parser_functions_and_canonical_schemas_preserved_confirmed: bool,
    pub future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed:
        bool,
    pub strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: bool,
    pub no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
        bool,
    pub future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    pub source_available_at_remains_unverified_until_separate_evidence_confirmed: bool,
    pub fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed:
        bool,
    pub no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    pub no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub registration_only_opens_chain_external_first_execution_authorization_review_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub first_execution_authorization_review_eligible: bool,
    pub first_execution_authorized: bool,
    pub raw_payload_accessed: bool,
    pub parsed_calendar_rows_created: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserIsolatedRunnerItem {
    pub runner: ControlledShadowMarketDataParserIsolatedRunnerRecord,
    pub approved_review_binding_current: bool,
    pub first_execution_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserIsolatedRunnerRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_implementations:
        Vec<IndependentlyApprovedControlledShadowMarketDataParserImplementation>,
    pub registration_eligible_count: usize,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub items: Vec<ControlledShadowMarketDataParserIsolatedRunnerItem>,
    pub runner_status: String,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub raw_payload_accessed: bool,
    pub parsed_calendar_rows_created: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowMarketDataParserIsolatedRunnerReadinessSummary {
    pub registration_eligible_count: usize,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_parser_isolated_runners(
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
            warn!(%error, "market-data parser isolated runner registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "行情 parser 隔离 runner 规格登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_market_data_parser_isolated_runner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowMarketDataParserIsolatedRunnerRequest>,
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

pub(crate) async fn controlled_shadow_market_data_parser_isolated_runner_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserIsolatedRunnerReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowMarketDataParserIsolatedRunnerReadinessSummary {
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
pub(crate) async fn market_data_parser_runners_for_first_execution_authorization_review(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserIsolatedRunnerRecord>, String> {
    let approved =
        independently_approved_market_data_parser_implementations_for_isolated_runner_registration(
            state,
        )
        .await?;
    let mut runners = read_runners(state)
        .await?
        .into_iter()
        .filter(|runner| {
            approved.iter().any(|value| binding_matches(runner, value))
                && runner.first_execution_authorization_review_eligible
        })
        .collect::<Vec<_>>();
    runners.sort_by(|left, right| right.registered_at.cmp(&left.registered_at));
    Ok(runners)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserIsolatedRunnerRegistry, String> {
    let approved =
        independently_approved_market_data_parser_implementations_for_isolated_runner_registration(
            state,
        )
        .await?;
    let mut items = read_runners(state)
        .await?
        .into_iter()
        .map(|runner| {
            let current = approved.iter().any(|value| binding_matches(&runner, value));
            ControlledShadowMarketDataParserIsolatedRunnerItem {
                first_execution_authorization_review_eligible: current
                    && runner.first_execution_authorization_review_eligible,
                approved_review_binding_current: current,
                runner,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| right.runner.registered_at.cmp(&left.runner.registered_at));
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
        "waiting_stage_98_independently_approved_market_data_parser_implementation"
    } else if first_execution_authorization_review_eligible_count > 0 {
        "market_data_parser_isolated_runner_registered_not_authorized_not_run"
    } else if registration_eligible_count > 0 {
        "ready_for_market_data_parser_isolated_runner_specification_registration"
    } else {
        "market_data_parser_isolated_runner_binding_stale_fail_closed"
    };
    Ok(ControlledShadowMarketDataParserIsolatedRunnerRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_implementations,
        registration_eligible_count,
        runner_count: items.len(),
        current_binding_runner_count,
        first_execution_authorization_review_eligible_count,
        items,
        runner_status: runner_status.to_string(),
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        raw_payload_accessed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 99 只把 Stage 98 独立批准绑定到未来 runner 工件身份、不可变代码版本、固定非特权 runtime、Stage 94 已验证只读输入和 create-once 不可信输出。当前没有源码、可执行工件、入口、runtime、挂载、载荷读取或解析；下一步只能进入 Stage 100 责任链外首次执行授权复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterControlledShadowMarketDataParserIsolatedRunnerRequest,
) -> Result<ControlledShadowMarketDataParserIsolatedRunnerRecord, String> {
    let _lock = acquire_lock(state, &request.expected_implementation_id).await?;
    let approved =
        independently_approved_market_data_parser_implementations_for_isolated_runner_registration(
            state,
        )
        .await?;
    let value = approved
        .iter()
        .find(|value| value.implementation.implementation_id == request.expected_implementation_id)
        .ok_or_else(|| "当前没有这条绑定有效且已独立批准的 Stage 97 parser 实现".to_string())?;
    validate_expected_binding(value, &request)?;
    if read_runners(state)
        .await?
        .iter()
        .any(|runner| runner.implementation_review.review_id == value.review.review_id)
    {
        return Err("这条 Stage 98 批准已经登记过隔离 runner 规格".to_string());
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
    approved: &IndependentlyApprovedControlledShadowMarketDataParserImplementation,
    request: &RegisterControlledShadowMarketDataParserIsolatedRunnerRequest,
) -> Result<(), String> {
    let implementation = &approved.implementation;
    let review = &approved.review;
    let contract = &implementation.implementation_contract;
    let spec_review = &implementation.upstream_specification_review;
    let registration = &implementation.upstream_specification_registration;
    if request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_review_id != review.review_id
        || request.expected_implementation_review_sha256 != review.review_sha256
        || request.expected_independent_audit_sha256 != review.independent_audit.audit_sha256
        || request.expected_specification_review_sha256 != spec_review.review_sha256
        || request.expected_specification_registration_sha256 != registration.registration_sha256
        || request.expected_parser_specification_sha256
            != registration
                .parser_specification
                .parser_specification_sha256
        || request.expected_validation_sha256 != contract.validation_sha256
        || request.expected_receipt_sha256 != contract.receipt_sha256
        || request.expected_claim_sha256 != contract.claim_sha256
        || request.expected_result_sha256 != contract.result_sha256
    {
        return Err("Stage 93–98 parser 收据、规格、实现、审计或复核绑定已经变化".to_string());
    }
    Ok(())
}

fn new_record(
    approved: &IndependentlyApprovedControlledShadowMarketDataParserImplementation,
    registrar_id: &str,
    request: &RegisterControlledShadowMarketDataParserIsolatedRunnerRequest,
    registered_at: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataParserIsolatedRunnerRecord, String> {
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(approved);
    if registrar_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|value| value == registrar_id)
    {
        return Err("runner 规格登记人必须独立于 Stage 98 和完整 Stage 51–98 责任链".to_string());
    }
    if !request_checks(request) {
        return Err("必须逐项确认 Stage 99 runner 规格、解析语义与零权限边界".to_string());
    }
    if !valid_sha256(&request.proposed_runner_artifact_sha256) {
        return Err("未来 runner 工件 SHA-256 无效".to_string());
    }
    let runner_name = bounded_required(&request.runner_name, MAX_NAME_CHARS, "runner 名称")?;
    let runner_spec_revision = bounded_required(
        &request.runner_spec_revision,
        MAX_REVISION_CHARS,
        "runner 规格版本",
    )?;
    let proposed_runner_code_revision = bounded_required(
        &request.proposed_runner_code_revision,
        MAX_REVISION_CHARS,
        "未来 runner 代码版本",
    )?;
    let mut record = ControlledShadowMarketDataParserIsolatedRunnerRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        isolated_runner_id: String::new(),
        isolated_runner_spec_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        implementation: approved.implementation.clone(),
        implementation_review: approved.review.clone(),
        excluded_prior_actor_ids,
        runner_name,
        runner_kind: request.runner_kind.clone(),
        artifact_reproduction_procedure: bounded_required(
            &request.artifact_reproduction_procedure,
            MAX_TEXT_CHARS,
            "工件复现步骤",
        )?,
        rationale: bounded_required(&request.rationale, MAX_TEXT_CHARS, "登记理由")?,
        known_limitations: bounded_required(
            &request.known_limitations,
            MAX_TEXT_CHARS,
            "已知局限",
        )?,
        future_input_constraints: bounded_required(
            &request.future_input_constraints,
            MAX_TEXT_CHARS,
            "未来输入约束",
        )?,
        future_output_constraints: bounded_required(
            &request.future_output_constraints,
            MAX_TEXT_CHARS,
            "未来输出约束",
        )?,
        runner_contract: runner_contract(
            approved,
            &runner_spec_revision,
            &proposed_runner_code_revision,
            &request.proposed_runner_artifact_sha256,
        )?,
        status: STATUS.to_string(),
        confirmations_complete: true,
        exact_current_stage_51_through_stage_98_binding_confirmed: true,
        registrar_independent_from_stage_98_and_complete_prior_chain_confirmed: true,
        implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed:
            true,
        proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed:
            true,
        all_eight_parser_functions_and_canonical_schemas_preserved_confirmed: true,
        future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed:
            true,
        strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: true,
        no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
            true,
        future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
            true,
        source_available_at_remains_unverified_until_separate_evidence_confirmed: true,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed:
            true,
        no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
            true,
        no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
            true,
        registration_only_opens_chain_external_first_execution_authorization_review_confirmed:
            true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        first_execution_authorization_review_eligible: true,
        first_execution_authorized: false,
        raw_payload_accessed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.isolated_runner_spec_sha256 = runner_fingerprint(&record)?;
    record.isolated_runner_id = record.isolated_runner_spec_sha256[..32].to_string();
    validate_market_data_parser_isolated_runner_for_review(&record)?;
    Ok(record)
}

fn runner_contract(
    approved: &IndependentlyApprovedControlledShadowMarketDataParserImplementation,
    runner_spec_revision: &str,
    proposed_runner_code_revision: &str,
    proposed_runner_artifact_sha256: &str,
) -> Result<ControlledShadowMarketDataParserIsolatedRunnerContract, String> {
    let implementation = &approved.implementation;
    let source = &implementation.implementation_contract;
    let mut contract = ControlledShadowMarketDataParserIsolatedRunnerContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        stage_98_implementation_review_id: approved.review.review_id.clone(),
        stage_98_implementation_review_sha256: approved.review.review_sha256.clone(),
        stage_98_independent_audit_sha256: approved.review.independent_audit.audit_sha256.clone(),
        stage_97_implementation_id: implementation.implementation_id.clone(),
        stage_97_implementation_sha256: implementation.implementation_sha256.clone(),
        stage_97_implementation_contract_sha256: source.contract_sha256.clone(),
        stage_96_specification_review_sha256: implementation
            .upstream_specification_review
            .review_sha256
            .clone(),
        stage_95_specification_registration_sha256: implementation
            .upstream_specification_registration
            .registration_sha256
            .clone(),
        stage_95_parser_specification_sha256: implementation
            .upstream_specification_registration
            .parser_specification
            .parser_specification_sha256
            .clone(),
        stage_94_validation_sha256: source.validation_sha256.clone(),
        stage_93_receipt_sha256: source.receipt_sha256.clone(),
        stage_93_claim_sha256: source.claim_sha256.clone(),
        stage_93_result_sha256: source.result_sha256.clone(),
        exact_approved_implementation_contract: source.clone(),
        runner_spec_revision: runner_spec_revision.to_string(),
        proposed_runner_code_revision: proposed_runner_code_revision.to_string(),
        proposed_runner_artifact_sha256: proposed_runner_artifact_sha256.to_ascii_lowercase(),
        runtime_identity: RUNTIME_IDENTITY.to_string(),
        runtime_version: RUNTIME_VERSION.to_string(),
        future_input_envelope: FUTURE_INPUT_ENVELOPE.to_string(),
        future_output_envelope: FUTURE_OUTPUT_ENVELOPE.to_string(),
        next_gate: NEXT_GATE.to_string(),
        specification_registered: true,
        future_runner_artifact_identity_bound: true,
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_identity_bound: true,
        runtime_instantiated: false,
        raw_payload_mount_present: false,
        raw_payload_read_allowed: false,
        data_access_authorized: false,
        root_filesystem_read_only_required: true,
        ephemeral_working_directory_required: true,
        run_as_unprivileged_required: true,
        no_new_privileges_required: true,
        future_input_read_only_required: true,
        future_input_content_addressed_required: true,
        future_input_stage_94_validation_required: true,
        future_output_create_once_required: true,
        future_output_untrusted_required: true,
        future_output_independent_validation_required: true,
        future_output_market_interpretation_allowed: false,
        future_output_order_intent_allowed: false,
        environment_inheritance_allowed: false,
        allowed_environment_variables: Vec::new(),
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        parsed_row_writes_allowed: false,
        observation_writes_allowed: false,
        ledger_creation_allowed: false,
        position_writes_allowed: false,
        performance_metric_writes_allowed: false,
        model_store_writes_allowed: false,
        metric_store_writes_allowed: false,
        training_feedback_allowed: false,
        reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
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

fn request_checks(request: &RegisterControlledShadowMarketDataParserIsolatedRunnerRequest) -> bool {
    request.exact_current_stage_51_through_stage_98_binding_confirmed
        && request.registrar_independent_from_stage_98_and_complete_prior_chain_confirmed
        && request.implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed
        && request.proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed
        && request.all_eight_parser_functions_and_canonical_schemas_preserved_confirmed
        && request.future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed
        && request.strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed
        && request.no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed
        && request.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && request.source_available_at_remains_unverified_until_separate_evidence_confirmed
        && request.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed
        && request.no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && request.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.registration_only_opens_chain_external_first_execution_authorization_review_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

pub(crate) fn validate_market_data_parser_isolated_runner_for_review(
    record: &ControlledShadowMarketDataParserIsolatedRunnerRecord,
) -> Result<(), String> {
    let contract = &record.runner_contract;
    let closed = !contract.source_artifact_present
        && !contract.executable_artifact_present
        && !contract.callable_entrypoint_present
        && contract.runtime_identity_bound
        && !contract.runtime_instantiated
        && !contract.raw_payload_mount_present
        && !contract.raw_payload_read_allowed
        && !contract.data_access_authorized
        && !contract.environment_inheritance_allowed
        && contract.allowed_environment_variables.is_empty()
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.parsed_row_writes_allowed
        && !contract.observation_writes_allowed
        && !contract.ledger_creation_allowed
        && !contract.position_writes_allowed
        && !contract.performance_metric_writes_allowed
        && !contract.model_store_writes_allowed
        && !contract.metric_store_writes_allowed
        && !contract.training_feedback_allowed
        && !contract.reward_allowed
        && !contract.order_generation_allowed
        && !contract.broker_access_allowed
        && !contract.trading_allowed;
    let record_closed = !record.first_execution_authorized
        && !record.raw_payload_accessed
        && !record.parsed_calendar_rows_created
        && !record.parsed_market_rows_created
        && !record.forward_observation_started
        && !record.ledger_created
        && !record.position_written
        && !record.performance_metric_written
        && !record.model_store_written
        && !record.metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let valid = record.schema_version == RECORD_SCHEMA_VERSION
        && record.policy_version == POLICY_VERSION
        && valid_id(&record.isolated_runner_id)
        && valid_sha256(&record.isolated_runner_spec_sha256)
        && record.isolated_runner_id == record.isolated_runner_spec_sha256[..32]
        && record.isolated_runner_spec_sha256 == runner_fingerprint(record)?
        && record.status == STATUS
        && record.confirmations_complete
        && record.exact_current_stage_51_through_stage_98_binding_confirmed
        && record.registrar_independent_from_stage_98_and_complete_prior_chain_confirmed
        && record
            .implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed
        && record
            .proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed
        && record.all_eight_parser_functions_and_canonical_schemas_preserved_confirmed
        && record
            .future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed
        && record.strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed
        && record
            .no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed
        && record
            .future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && record.source_available_at_remains_unverified_until_separate_evidence_confirmed
        && record
            .fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed
        && record
            .no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && record
            .no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && record.registration_only_opens_chain_external_first_execution_authorization_review_confirmed
        && record.no_unconfirmed_hari_or_old_wang_logic_claimed
        && !record
            .excluded_prior_actor_ids
            .contains(&record.registered_by)
        && sorted_unique(&record.excluded_prior_actor_ids)
        && record.first_execution_authorization_review_eligible
        && contract.schema_version == CONTRACT_SCHEMA_VERSION
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?
        && contract.next_gate == NEXT_GATE
        && contract.specification_registered
        && contract.future_runner_artifact_identity_bound
        && valid_sha256(&contract.proposed_runner_artifact_sha256)
        && contract.root_filesystem_read_only_required
        && contract.ephemeral_working_directory_required
        && contract.run_as_unprivileged_required
        && contract.no_new_privileges_required
        && contract.future_input_read_only_required
        && contract.future_input_content_addressed_required
        && contract.future_input_stage_94_validation_required
        && contract.future_output_create_once_required
        && contract.future_output_untrusted_required
        && contract.future_output_independent_validation_required
        && closed
        && record_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 99 parser runner 规格无效、漂移或越权".to_string())
}

fn binding_matches(
    runner: &ControlledShadowMarketDataParserIsolatedRunnerRecord,
    approved: &IndependentlyApprovedControlledShadowMarketDataParserImplementation,
) -> bool {
    runner.implementation == approved.implementation
        && runner.implementation_review == approved.review
        && validate_market_data_parser_isolated_runner_for_review(runner).is_ok()
}

fn expected_excluded_prior_actor_ids(
    approved: &IndependentlyApprovedControlledShadowMarketDataParserImplementation,
) -> Vec<String> {
    let mut actors = approved.review.excluded_prior_actor_ids.clone();
    actors.extend(approved.implementation.excluded_prior_actor_ids.clone());
    actors.push(approved.implementation.registered_by.clone());
    actors.push(approved.review.reviewer_id.clone());
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn runner_fingerprint(
    record: &ControlledShadowMarketDataParserIsolatedRunnerRecord,
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
        .ok_or_else(|| "Stage 99 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars {
        Err(format!("{label}不能为空且不得超过 {max_chars} 字"))
    } else {
        Ok(value.to_string())
    }
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

async fn read_runners(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserIsolatedRunnerRecord>, String> {
    let directory = runner_root(state);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut runners = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        if entry
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("json")
        {
            let runner: ControlledShadowMarketDataParserIsolatedRunnerRecord =
                serde_json::from_slice(
                    &tokio::fs::read(&path)
                        .await
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
            if path.file_stem().and_then(|value| value.to_str())
                != Some(runner.isolated_runner_id.as_str())
            {
                return Err("Stage 99 runner 文件名与记录 ID 不一致".to_string());
            }
            validate_market_data_parser_isolated_runner_for_review(&runner)?;
            runners.push(runner);
        }
    }
    runners.sort_by(|left, right| left.registered_at.cmp(&right.registered_at));
    let mut ids = BTreeSet::new();
    let mut reviews = BTreeSet::new();
    for runner in &runners {
        if !ids.insert(runner.isolated_runner_id.clone())
            || !reviews.insert(runner.implementation_review.review_id.clone())
        {
            return Err("Stage 99 runner 登记重复或分叉".to_string());
        }
    }
    Ok(runners)
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

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn runner_root(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-first-natural-forward-cycle-market-data-parser-isolated-runners")
}

struct RunnerLock(PathBuf);
impl Drop for RunnerLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, implementation_id: &str) -> Result<RunnerLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-market-data-parser-isolated-runner-{implementation_id}.lock"
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
        .map_err(|_| "已有同一 Stage 99 runner 登记正在进行".to_string())?;
    Ok(RunnerLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_requires_every_zero_capability_confirmation() {
        let request = RegisterControlledShadowMarketDataParserIsolatedRunnerRequest {
            expected_implementation_id: "a".repeat(32), expected_implementation_sha256: "b".repeat(64), expected_implementation_contract_sha256: "c".repeat(64),
            expected_implementation_review_id: "d".repeat(32), expected_implementation_review_sha256: "e".repeat(64), expected_independent_audit_sha256: "f".repeat(64),
            expected_specification_review_sha256: "1".repeat(64), expected_specification_registration_sha256: "2".repeat(64), expected_parser_specification_sha256: "3".repeat(64),
            expected_validation_sha256: "4".repeat(64), expected_receipt_sha256: "5".repeat(64), expected_claim_sha256: "6".repeat(64), expected_result_sha256: "7".repeat(64),
            runner_name: "隔离行情解析器".to_string(), runner_kind: ControlledShadowMarketDataParserIsolatedRunnerKind::EphemeralDeterministicMarketDataParserSpecification,
            runner_spec_revision: "v1".to_string(), proposed_runner_code_revision: "rev-1".to_string(), proposed_runner_artifact_sha256: "8".repeat(64),
            artifact_reproduction_procedure: "未来独立复现后逐位核验。".to_string(), rationale: "冻结未来执行边界。".to_string(), known_limitations: "当前没有工件或真实载荷。".to_string(),
            future_input_constraints: "只读 Stage 94 已验证载荷。".to_string(), future_output_constraints: "create-once 不可信输出。".to_string(),
            exact_current_stage_51_through_stage_98_binding_confirmed: true, registrar_independent_from_stage_98_and_complete_prior_chain_confirmed: true,
            implementation_review_audit_contract_and_parser_specification_hashes_reproduced_confirmed: true,
            proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
            all_eight_parser_functions_and_canonical_schemas_preserved_confirmed: true,
            future_input_only_stage_94_validated_read_only_content_addressed_receipt_payloads_confirmed: true,
            strict_source_calendar_action_numeric_and_failure_semantics_preserved_confirmed: true,
            no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: true,
            future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
            source_available_at_remains_unverified_until_separate_evidence_confirmed: true,
            fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
            no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            registration_only_opens_chain_external_first_execution_authorization_review_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        };
        assert!(request_checks(&request));
    }

    #[test]
    fn proposed_artifact_hash_must_be_exact_sha256() {
        assert!(valid_sha256(&"a".repeat(64)));
        assert!(!valid_sha256(&"a".repeat(63)));
        assert!(!valid_sha256(&format!("{}g", "a".repeat(63))));
    }

    #[test]
    fn resource_contract_is_intentionally_small_and_single_process() {
        assert_eq!(MAXIMUM_MEMORY_MIB, 1_024);
        assert_eq!(MAXIMUM_WALL_CLOCK_SECONDS, 300);
        assert_eq!(MAXIMUM_CPU_MILLICORES, 1_000);
        assert_eq!(MAXIMUM_PROCESS_COUNT, 1);
        assert_eq!(MAXIMUM_OUTPUT_BYTES, 8 * 1024 * 1024);
    }
}
