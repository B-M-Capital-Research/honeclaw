//! Immutable specification registry for a future historical-outcome dry-run runner.
//!
//! A record freezes the exact reviewed implementation, runner artifact and
//! sandbox resource contract. Registration deliberately provides no callable
//! entrypoint and grants no execution authority. A later independent review
//! must approve a first invocation; this module never invokes a runner,
//! computes outcomes, creates output artifacts, writes labels or trades.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_dry_run_implementations::HistoricalOutcomeDryRunImplementationKind;
use super::historical_outcome_dry_run_run_authorizations::{
    ApprovedHistoricalOutcomeDryRunRunAuthorization,
    isolated_runner_registration_eligible_authorizations,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-isolated-runner-registry-v1";
const RUNNER_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-isolated-runner-v1";
const RUNTIME_POLICY_VERSION: &str =
    "hone-historical-outcome-dry-run-isolated-runtime-v1-non-invoking";
const STATUS: &str = "registered_not_run";
const INPUT_MOUNT_CONTRACT: &str = "未来经独立授权后，只能把当前运行授权复核所绑定的封存行情、七层状态、标签器和协议作为只读输入挂载；不得联网补数、替换输入或读取未绑定资料。";
const OUTPUT_CONTRACT: &str = "未来首次运行只能写入一次性临时目录，输出默认不可信且必须经独立工件校验；当前登记不创建目录、不运行、不计算，也不产生输出工件。";
const INVOCATION_CONTRACT: &str = "本规范没有可调用入口，也不授予首次执行权限；未来调用必须绑定本规范、制品摘要和新的独立执行授权记录。";
const MAX_NAME_CHARS: usize = 120;
const MAX_REVISION_CHARS: usize = 160;
const MAX_WALL_CLOCK_SECONDS: u16 = 300;
const MAX_MEMORY_MIB: u16 = 512;
const MAX_CPU_MILLICORES: u16 = 1_000;
const MAX_PROCESS_COUNT: u8 = 1;
const MAX_OUTPUT_BYTES: u32 = 1_048_576;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeDryRunIsolatedRunnerKind {
    EphemeralDeterministicProcess,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeDryRunIsolatedRunnerRequest {
    dry_run_implementation_id: String,
    expected_run_authorization_review_id: String,
    expected_run_authorization_review_sha256: String,
    expected_implementation_spec_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    runner_name: String,
    runner_kind: HistoricalOutcomeDryRunIsolatedRunnerKind,
    runner_code_revision: String,
    runner_artifact_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeDryRunIsolatedRunnerRecord {
    pub schema_version: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub run_authorization_review_id: String,
    pub run_authorization_review_sha256: String,
    pub run_authorization_reviewer_id: String,
    pub dry_run_implementation_id: String,
    pub dry_run_implementation_spec_sha256: String,
    pub dry_run_implementation_code_revision: String,
    pub dry_run_implementation_kind: HistoricalOutcomeDryRunImplementationKind,
    pub authorization_review_id: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub reconstruction_review_id: String,
    pub labeler_implementation_id: String,
    pub labeler_implementation_spec_sha256: String,
    pub labeler_implementation_review_id: String,
    pub labeler_code_revision: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub requested_from: NaiveDate,
    pub requested_to: NaiveDate,
    pub asset_series_sha256: String,
    pub benchmark_series_sha256: String,
    pub common_session_count: usize,
    pub covered_horizons_market_sessions: Vec<u16>,
    pub runtime_policy_version: String,
    pub runner_name: String,
    pub runner_kind: HistoricalOutcomeDryRunIsolatedRunnerKind,
    pub runner_code_revision: String,
    pub runner_artifact_sha256: String,
    pub status: String,
    pub input_mount_contract: String,
    pub output_contract: String,
    pub invocation_contract: String,
    pub callable_entrypoint_registered: bool,
    pub input_mount_read_only_required: bool,
    pub root_filesystem_read_only_required: bool,
    pub ephemeral_working_directory_required: bool,
    pub output_validation_required: bool,
    pub run_as_unprivileged_required: bool,
    pub no_new_privileges_required: bool,
    pub host_environment_inherited: bool,
    pub allowed_environment_variables: Vec<String>,
    pub secrets_available: bool,
    pub max_wall_clock_seconds: u16,
    pub max_memory_mib: u16,
    pub max_cpu_millicores: u16,
    pub max_process_count: u8,
    pub max_output_bytes: u32,
    pub outbound_network_allowed: bool,
    pub external_tools_allowed: bool,
    pub production_writes_allowed: bool,
    pub historical_state_mutation_allowed: bool,
    pub outcome_label_writes_allowed: bool,
    pub training_writes_allowed: bool,
    pub reward_writes_allowed: bool,
    pub shadow_writes_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub invocation_authorized: bool,
    pub offline_dry_run_enabled: bool,
    pub execution_started: bool,
    pub output_artifact_created: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunIsolatedRunnerItem {
    pub runner: HistoricalOutcomeDryRunIsolatedRunnerRecord,
    pub run_authorization_binding_current: bool,
    pub execution_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunIsolatedRunnerRegistry {
    pub schema_version: String,
    pub runtime_policy_version: String,
    pub eligible_authorizations: Vec<ApprovedHistoricalOutcomeDryRunRunAuthorization>,
    pub allowed_runner_kinds: Vec<HistoricalOutcomeDryRunIsolatedRunnerKind>,
    pub registration_allowed: bool,
    pub current_runtime_artifact_sha256: Option<String>,
    pub current_runtime_git_sha: Option<String>,
    pub current_runtime_build_source: String,
    pub runners: Vec<HistoricalOutcomeDryRunIsolatedRunnerItem>,
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
    pub invocation_authorized: bool,
    pub offline_dry_run_enabled: bool,
    pub execution_started: bool,
    pub output_artifact_created: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeDryRunIsolatedRunnerReadinessSummary {
    pub runner_count: usize,
    pub current_binding_runner_count: usize,
    pub execution_authorization_review_eligible_count: usize,
    pub runner_status: String,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
}

pub(crate) async fn handle_get_historical_outcome_dry_run_isolated_runners(
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
            warn!(%error, "historical outcome dry-run isolated runner registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线试运行隔离执行器注册表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_dry_run_isolated_runner(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeDryRunIsolatedRunnerRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_runner(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome dry-run isolated runner registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线试运行隔离执行器注册表暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_dry_run_isolated_runner_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunIsolatedRunnerReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeDryRunIsolatedRunnerReadinessSummary {
        runner_count: registry.runner_count,
        current_binding_runner_count: registry.current_binding_runner_count,
        execution_authorization_review_eligible_count: registry
            .execution_authorization_review_eligible_count,
        runner_status: registry.runner_status,
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
    })
}

pub(crate) async fn first_execution_authorization_review_eligible_runners(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeDryRunIsolatedRunnerRecord>, String> {
    let authorizations = isolated_runner_registration_eligible_authorizations(state).await?;
    let authorization_index = authorizations
        .iter()
        .map(|authorization| (authorization.review.review_id.as_str(), authorization))
        .collect::<HashMap<_, _>>();
    let mut runners = read_runners(state)
        .await?
        .into_iter()
        .filter(|runner| {
            authorization_index
                .get(runner.run_authorization_review_id.as_str())
                .is_some_and(|authorization| binding_matches(runner, authorization))
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
) -> Result<HistoricalOutcomeDryRunIsolatedRunnerRegistry, String> {
    let current_build = hone_core::current_build_info();
    let eligible_authorizations =
        isolated_runner_registration_eligible_authorizations(state).await?;
    let authorization_index = eligible_authorizations
        .iter()
        .map(|authorization| (authorization.review.review_id.as_str(), authorization))
        .collect::<HashMap<_, _>>();
    let mut runners = read_runners(state)
        .await?
        .into_iter()
        .map(|runner| {
            let run_authorization_binding_current = authorization_index
                .get(runner.run_authorization_review_id.as_str())
                .is_some_and(|authorization| binding_matches(&runner, authorization));
            HistoricalOutcomeDryRunIsolatedRunnerItem {
                execution_authorization_review_eligible: run_authorization_binding_current,
                run_authorization_binding_current,
                runner,
            }
        })
        .collect::<Vec<_>>();
    runners.sort_by(|left, right| {
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
    let runner_count = runners.len();
    let current_binding_runner_count = runners
        .iter()
        .filter(|item| item.run_authorization_binding_current)
        .count();
    let execution_authorization_review_eligible_count = runners
        .iter()
        .filter(|item| item.execution_authorization_review_eligible)
        .count();
    let runner_status = if eligible_authorizations.is_empty() {
        "waiting_for_approved_run_authorization_review"
    } else if runner_count == 0 {
        "waiting_for_immutable_isolated_runner_registration"
    } else if current_binding_runner_count == 0 {
        "registered_binding_stale"
    } else {
        STATUS
    };
    Ok(HistoricalOutcomeDryRunIsolatedRunnerRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        runtime_policy_version: RUNTIME_POLICY_VERSION.to_string(),
        registration_allowed: !eligible_authorizations.is_empty(),
        current_runtime_artifact_sha256: current_build.binary_sha256,
        current_runtime_git_sha: current_build.git_sha,
        current_runtime_build_source: current_build.source.as_str().to_string(),
        eligible_authorizations,
        allowed_runner_kinds: vec![
            HistoricalOutcomeDryRunIsolatedRunnerKind::EphemeralDeterministicProcess,
        ],
        runners,
        runner_count,
        current_binding_runner_count,
        execution_authorization_review_eligible_count,
        runner_status: runner_status.to_string(),
        invocation_authorized: false,
        offline_dry_run_enabled: false,
        execution_started: false,
        output_artifact_created: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "本阶段只冻结未来隔离执行器的制品摘要、资源上限、只读输入与临时输出合同；登记对象没有可调用入口，也没有首次执行权限。运行、输出工件、标签准入、训练、奖励、影子组合、订单、券商和交易仍需后续独立门禁。".to_string(),
    })
}

async fn register_runner(
    state: &AppState,
    admin_id: &str,
    request: RegisterHistoricalOutcomeDryRunIsolatedRunnerRequest,
) -> Result<HistoricalOutcomeDryRunIsolatedRunnerRecord, String> {
    let _lock = acquire_lock(state, "historical-outcome-dry-run-isolated-runner").await?;
    let authorizations = isolated_runner_registration_eligible_authorizations(state).await?;
    let authorization = authorizations
        .into_iter()
        .find(|authorization| {
            authorization.implementation.dry_run_implementation_id
                == request.dry_run_implementation_id
        })
        .ok_or_else(|| "当前没有这条绑定有效且通过独立复核的运行授权".to_string())?;
    if authorization.review.review_id != request.expected_run_authorization_review_id
        || authorization.review.review_sha256 != request.expected_run_authorization_review_sha256
        || authorization
            .implementation
            .dry_run_implementation_spec_sha256
            != request.expected_implementation_spec_sha256
        || authorization.implementation.snapshot_sha256 != request.expected_snapshot_sha256
        || authorization.implementation.protocol_sha256 != request.expected_protocol_sha256
    {
        return Err("运行授权、实现、行情或协议指纹已经变化，请刷新后重试".to_string());
    }
    let runner_name = bounded_required(&request.runner_name, MAX_NAME_CHARS, "执行器名称")?;
    let runner_code_revision = bounded_required(
        &request.runner_code_revision,
        MAX_REVISION_CHARS,
        "执行器代码版本",
    )?;
    if !valid_sha256(&request.runner_artifact_sha256) {
        return Err("执行器制品 SHA-256 无效".to_string());
    }
    let record = new_record(
        &authorization,
        admin_id,
        runner_name,
        request.runner_kind,
        runner_code_revision,
        request.runner_artifact_sha256.to_ascii_lowercase(),
        Utc::now(),
    )?;
    let deduplication_fingerprint = runner_deduplication_fingerprint(&record)?;
    if read_runners(state)
        .await?
        .iter()
        .map(runner_deduplication_fingerprint)
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|existing| existing == &deduplication_fingerprint)
    {
        return Err("完全相同的隔离执行器规范已经登记；制品或规范变化时请登记新版本".to_string());
    }
    write_immutable_json(
        &runner_root(state).join(format!("{}.json", record.isolated_runner_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn new_record(
    authorization: &ApprovedHistoricalOutcomeDryRunRunAuthorization,
    admin_id: &str,
    runner_name: String,
    runner_kind: HistoricalOutcomeDryRunIsolatedRunnerKind,
    runner_code_revision: String,
    runner_artifact_sha256: String,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeDryRunIsolatedRunnerRecord, String> {
    let implementation = &authorization.implementation;
    let review = &authorization.review;
    let mut record = HistoricalOutcomeDryRunIsolatedRunnerRecord {
        schema_version: RUNNER_SCHEMA_VERSION.to_string(),
        isolated_runner_id: String::new(),
        isolated_runner_spec_sha256: String::new(),
        registered_at,
        registered_by: admin_id.to_string(),
        run_authorization_review_id: review.review_id.clone(),
        run_authorization_review_sha256: review.review_sha256.clone(),
        run_authorization_reviewer_id: review.reviewer_id.clone(),
        dry_run_implementation_id: implementation.dry_run_implementation_id.clone(),
        dry_run_implementation_spec_sha256: implementation
            .dry_run_implementation_spec_sha256
            .clone(),
        dry_run_implementation_code_revision: implementation.code_revision.clone(),
        dry_run_implementation_kind: implementation.implementation_kind.clone(),
        authorization_review_id: implementation.authorization_review_id.clone(),
        snapshot_id: implementation.snapshot_id.clone(),
        snapshot_sha256: implementation.snapshot_sha256.clone(),
        reconstruction_id: implementation.reconstruction_id.clone(),
        reconstruction_sha256: implementation.reconstruction_sha256.clone(),
        reconstruction_review_id: implementation.reconstruction_review_id.clone(),
        labeler_implementation_id: implementation.labeler_implementation_id.clone(),
        labeler_implementation_spec_sha256: implementation
            .labeler_implementation_spec_sha256
            .clone(),
        labeler_implementation_review_id: implementation.labeler_implementation_review_id.clone(),
        labeler_code_revision: implementation.labeler_code_revision.clone(),
        protocol_version: implementation.protocol_version.clone(),
        protocol_sha256: implementation.protocol_sha256.clone(),
        asset_symbol: implementation.asset_symbol.clone(),
        benchmark_symbol: implementation.benchmark_symbol.clone(),
        requested_from: implementation.requested_from,
        requested_to: implementation.requested_to,
        asset_series_sha256: implementation.asset_series_sha256.clone(),
        benchmark_series_sha256: implementation.benchmark_series_sha256.clone(),
        common_session_count: implementation.common_session_count,
        covered_horizons_market_sessions: implementation.covered_horizons_market_sessions.clone(),
        runtime_policy_version: RUNTIME_POLICY_VERSION.to_string(),
        runner_name,
        runner_kind,
        runner_code_revision,
        runner_artifact_sha256,
        status: STATUS.to_string(),
        input_mount_contract: INPUT_MOUNT_CONTRACT.to_string(),
        output_contract: OUTPUT_CONTRACT.to_string(),
        invocation_contract: INVOCATION_CONTRACT.to_string(),
        callable_entrypoint_registered: false,
        input_mount_read_only_required: true,
        root_filesystem_read_only_required: true,
        ephemeral_working_directory_required: true,
        output_validation_required: true,
        run_as_unprivileged_required: true,
        no_new_privileges_required: true,
        host_environment_inherited: false,
        allowed_environment_variables: Vec::new(),
        secrets_available: false,
        max_wall_clock_seconds: MAX_WALL_CLOCK_SECONDS,
        max_memory_mib: MAX_MEMORY_MIB,
        max_cpu_millicores: MAX_CPU_MILLICORES,
        max_process_count: MAX_PROCESS_COUNT,
        max_output_bytes: MAX_OUTPUT_BYTES,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        production_writes_allowed: false,
        historical_state_mutation_allowed: false,
        outcome_label_writes_allowed: false,
        training_writes_allowed: false,
        reward_writes_allowed: false,
        shadow_writes_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        invocation_authorized: false,
        offline_dry_run_enabled: false,
        execution_started: false,
        output_artifact_created: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
    };
    record.isolated_runner_spec_sha256 = runner_fingerprint(&record)?;
    record.isolated_runner_id = record.isolated_runner_spec_sha256[..32].to_string();
    validate_record(&record)?;
    Ok(record)
}

fn validate_record(record: &HistoricalOutcomeDryRunIsolatedRunnerRecord) -> Result<(), String> {
    let permissions_closed = !record.callable_entrypoint_registered
        && !record.host_environment_inherited
        && record.allowed_environment_variables.is_empty()
        && !record.secrets_available
        && !record.outbound_network_allowed
        && !record.external_tools_allowed
        && !record.production_writes_allowed
        && !record.historical_state_mutation_allowed
        && !record.outcome_label_writes_allowed
        && !record.training_writes_allowed
        && !record.reward_writes_allowed
        && !record.shadow_writes_allowed
        && !record.order_generation_allowed
        && !record.broker_access_allowed
        && !record.invocation_authorized
        && !record.offline_dry_run_enabled
        && !record.execution_started
        && !record.output_artifact_created
        && !record.outcome_label_generation_enabled
        && !record.decision_training_authorized
        && !record.reward_evidence_authorized
        && !record.shadow_evidence_authorized
        && !record.trading_authorized;
    if record.schema_version != RUNNER_SCHEMA_VERSION
        || !valid_id(&record.isolated_runner_id)
        || !valid_sha256(&record.isolated_runner_spec_sha256)
        || record.isolated_runner_id != record.isolated_runner_spec_sha256[..32]
        || record.isolated_runner_spec_sha256 != runner_fingerprint(record)?
        || record.registered_by.trim().is_empty()
        || !valid_id(&record.run_authorization_review_id)
        || !valid_sha256(&record.run_authorization_review_sha256)
        || record.run_authorization_reviewer_id.trim().is_empty()
        || !valid_id(&record.dry_run_implementation_id)
        || !valid_sha256(&record.dry_run_implementation_spec_sha256)
        || record
            .dry_run_implementation_code_revision
            .trim()
            .is_empty()
        || !valid_id(&record.authorization_review_id)
        || !valid_id(&record.snapshot_id)
        || !valid_sha256(&record.snapshot_sha256)
        || !valid_id(&record.reconstruction_id)
        || !valid_sha256(&record.reconstruction_sha256)
        || !valid_id(&record.reconstruction_review_id)
        || !valid_id(&record.labeler_implementation_id)
        || !valid_sha256(&record.labeler_implementation_spec_sha256)
        || !valid_id(&record.labeler_implementation_review_id)
        || record.labeler_code_revision.trim().is_empty()
        || record.protocol_version.trim().is_empty()
        || !valid_sha256(&record.protocol_sha256)
        || !valid_symbol(&record.asset_symbol)
        || !valid_symbol(&record.benchmark_symbol)
        || record.requested_from > record.requested_to
        || !valid_sha256(&record.asset_series_sha256)
        || !valid_sha256(&record.benchmark_series_sha256)
        || record.common_session_count <= 250
        || record.covered_horizons_market_sessions != vec![20, 60, 250]
        || record.runtime_policy_version != RUNTIME_POLICY_VERSION
        || record.runner_name.trim().is_empty()
        || record.runner_name.chars().count() > MAX_NAME_CHARS
        || record.runner_code_revision.trim().is_empty()
        || record.runner_code_revision.chars().count() > MAX_REVISION_CHARS
        || !valid_sha256(&record.runner_artifact_sha256)
        || record.status != STATUS
        || record.input_mount_contract != INPUT_MOUNT_CONTRACT
        || record.output_contract != OUTPUT_CONTRACT
        || record.invocation_contract != INVOCATION_CONTRACT
        || !record.input_mount_read_only_required
        || !record.root_filesystem_read_only_required
        || !record.ephemeral_working_directory_required
        || !record.output_validation_required
        || !record.run_as_unprivileged_required
        || !record.no_new_privileges_required
        || record.max_wall_clock_seconds != MAX_WALL_CLOCK_SECONDS
        || record.max_memory_mib != MAX_MEMORY_MIB
        || record.max_cpu_millicores != MAX_CPU_MILLICORES
        || record.max_process_count != MAX_PROCESS_COUNT
        || record.max_output_bytes != MAX_OUTPUT_BYTES
        || !permissions_closed
    {
        return Err(
            "historical outcome dry-run isolated runner is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn binding_matches(
    record: &HistoricalOutcomeDryRunIsolatedRunnerRecord,
    authorization: &ApprovedHistoricalOutcomeDryRunRunAuthorization,
) -> bool {
    let implementation = &authorization.implementation;
    let review = &authorization.review;
    record.run_authorization_review_id == review.review_id
        && record.run_authorization_review_sha256 == review.review_sha256
        && record.run_authorization_reviewer_id == review.reviewer_id
        && record.dry_run_implementation_id == implementation.dry_run_implementation_id
        && record.dry_run_implementation_spec_sha256
            == implementation.dry_run_implementation_spec_sha256
        && record.dry_run_implementation_code_revision == implementation.code_revision
        && record.dry_run_implementation_kind == implementation.implementation_kind
        && record.authorization_review_id == implementation.authorization_review_id
        && record.snapshot_id == implementation.snapshot_id
        && record.snapshot_sha256 == implementation.snapshot_sha256
        && record.reconstruction_id == implementation.reconstruction_id
        && record.reconstruction_sha256 == implementation.reconstruction_sha256
        && record.reconstruction_review_id == implementation.reconstruction_review_id
        && record.labeler_implementation_id == implementation.labeler_implementation_id
        && record.labeler_implementation_spec_sha256
            == implementation.labeler_implementation_spec_sha256
        && record.labeler_implementation_review_id
            == implementation.labeler_implementation_review_id
        && record.labeler_code_revision == implementation.labeler_code_revision
        && record.protocol_version == implementation.protocol_version
        && record.protocol_sha256 == implementation.protocol_sha256
        && record.asset_symbol == implementation.asset_symbol
        && record.benchmark_symbol == implementation.benchmark_symbol
        && record.requested_from == implementation.requested_from
        && record.requested_to == implementation.requested_to
        && record.asset_series_sha256 == implementation.asset_series_sha256
        && record.benchmark_series_sha256 == implementation.benchmark_series_sha256
        && record.common_session_count == implementation.common_session_count
        && record.covered_horizons_market_sessions
            == implementation.covered_horizons_market_sessions
}

fn runner_fingerprint(
    record: &HistoricalOutcomeDryRunIsolatedRunnerRecord,
) -> Result<String, String> {
    let mut value = serde_json::to_value(record).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "isolated runner fingerprint payload is invalid".to_string())?;
    object.remove("isolated_runner_id");
    object.remove("isolated_runner_spec_sha256");
    hash_serializable(&value)
}

fn runner_deduplication_fingerprint(
    record: &HistoricalOutcomeDryRunIsolatedRunnerRecord,
) -> Result<String, String> {
    let mut value = serde_json::to_value(record).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "isolated runner deduplication payload is invalid".to_string())?;
    object.remove("isolated_runner_id");
    object.remove("isolated_runner_spec_sha256");
    object.remove("registered_at");
    object.remove("registered_by");
    hash_serializable(&value)
}

async fn read_runners(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeDryRunIsolatedRunnerRecord>, String> {
    let directory = runner_root(state);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    let mut fingerprints = BTreeSet::new();
    while let Some(entry) = entries
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
        let record: HistoricalOutcomeDryRunIsolatedRunnerRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&record)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(record.isolated_runner_id.as_str())
        {
            return Err("historical outcome dry-run isolated runner filename mismatch".to_string());
        }
        if !ids.insert(record.isolated_runner_id.clone())
            || !fingerprints.insert(record.isolated_runner_spec_sha256.clone())
        {
            return Err(
                "historical outcome dry-run isolated runner registry contains duplicates"
                    .to_string(),
            );
        }
        output.push(record);
    }
    Ok(output)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn runner_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-dry-run-isolated-runners")
}

struct HistoricalOutcomeDryRunIsolatedRunnerLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeDryRunIsolatedRunnerLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<HistoricalOutcomeDryRunIsolatedRunnerLock, String> {
    if !valid_id(name) {
        return Err("invalid historical outcome dry-run isolated runner lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{name}.lock"));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(HistoricalOutcomeDryRunIsolatedRunnerLock { path }),
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
                return Err("隔离执行器正在由另一请求登记，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("隔离执行器登记锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid immutable audit path".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
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

fn bounded_required(value: &str, max: usize, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if trimmed.chars().count() > max {
        return Err(format!("{label}无效或过长"));
    }
    Ok(trimmed.to_string())
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn valid_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 24
        && value.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::historical_outcome_dry_run_implementations::HistoricalOutcomeDryRunImplementationRecord;
    use crate::routes::historical_outcome_dry_run_run_authorizations::{
        HistoricalOutcomeDryRunRunAuthorizationReview,
        HistoricalOutcomeDryRunRunAuthorizationVerdict,
    };

    fn authorization() -> ApprovedHistoricalOutcomeDryRunRunAuthorization {
        let implementation = HistoricalOutcomeDryRunImplementationRecord {
            schema_version: "hone-historical-outcome-dry-run-implementation-v1".to_string(),
            dry_run_implementation_id: "implementation-1".to_string(),
            dry_run_implementation_spec_sha256: "a".repeat(64),
            registered_at: DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            registered_by: "implementation-admin".to_string(),
            authorization_review_id: "authorization-review-1".to_string(),
            snapshot_id: "snapshot-1".to_string(),
            snapshot_sha256: "b".repeat(64),
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "c".repeat(64),
            reconstruction_review_id: "reconstruction-review-1".to_string(),
            labeler_implementation_id: "labeler-1".to_string(),
            labeler_implementation_spec_sha256: "d".repeat(64),
            labeler_implementation_review_id: "labeler-review-1".to_string(),
            labeler_code_revision: "oldwang@labeler123".to_string(),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "e".repeat(64),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            requested_from: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            requested_to: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            asset_payload_sha256: "f".repeat(64),
            benchmark_payload_sha256: "1".repeat(64),
            asset_series_sha256: "2".repeat(64),
            benchmark_series_sha256: "3".repeat(64),
            common_session_count: 251,
            covered_horizons_market_sessions: vec![20, 60, 250],
            sandbox_policy_version: "hone-historical-outcome-dry-run-sandbox-v1".to_string(),
            implementation_name: "共同交易日隔离试运行实现".to_string(),
            implementation_kind:
                HistoricalOutcomeDryRunImplementationKind::DeterministicIsolatedCommonSessionReplay,
            code_revision: "oldwang@dryrun123".to_string(),
            status: "registered_not_run".to_string(),
            input_contract: "input".to_string(),
            output_contract: "output".to_string(),
            metrics: vec![
                "asset_return".to_string(),
                "benchmark_return".to_string(),
                "excess_return".to_string(),
                "maximum_drawdown".to_string(),
            ],
            deterministic_replay_required: true,
            isolated_output_required: true,
            future_information_isolation_required: true,
            missing_data_fail_closed_required: true,
            max_parallel_series: 4,
            outbound_network_allowed: false,
            external_tools_allowed: false,
            production_writes_allowed: false,
            historical_state_mutation_allowed: false,
            outcome_label_writes_allowed: false,
            training_writes_allowed: false,
            reward_writes_allowed: false,
            shadow_writes_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            run_authorized: false,
            offline_dry_run_enabled: false,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        };
        let review = HistoricalOutcomeDryRunRunAuthorizationReview {
            schema_version:
                "hone-historical-outcome-dry-run-run-authorization-review-v1".to_string(),
            policy_version: "hone-historical-outcome-dry-run-run-authorization-v1".to_string(),
            review_id: "run-review-1".to_string(),
            review_sha256: "4".repeat(64),
            previous_review_id: None,
            previous_review_sha256: None,
            dry_run_implementation_id: implementation.dry_run_implementation_id.clone(),
            dry_run_implementation_spec_sha256: implementation
                .dry_run_implementation_spec_sha256
                .clone(),
            authorization_review_id: implementation.authorization_review_id.clone(),
            snapshot_id: implementation.snapshot_id.clone(),
            snapshot_sha256: implementation.snapshot_sha256.clone(),
            reconstruction_id: implementation.reconstruction_id.clone(),
            reconstruction_sha256: implementation.reconstruction_sha256.clone(),
            reconstruction_review_id: implementation.reconstruction_review_id.clone(),
            labeler_implementation_id: implementation.labeler_implementation_id.clone(),
            labeler_implementation_spec_sha256: implementation
                .labeler_implementation_spec_sha256
                .clone(),
            labeler_implementation_review_id: implementation
                .labeler_implementation_review_id
                .clone(),
            labeler_code_revision: implementation.labeler_code_revision.clone(),
            protocol_version: implementation.protocol_version.clone(),
            protocol_sha256: implementation.protocol_sha256.clone(),
            sandbox_policy_version: implementation.sandbox_policy_version.clone(),
            implementation_name: implementation.implementation_name.clone(),
            implementation_kind: implementation.implementation_kind.clone(),
            code_revision: implementation.code_revision.clone(),
            implementation_status: implementation.status.clone(),
            implementation_registered_by: implementation.registered_by.clone(),
            submitted_at: DateTime::parse_from_rfc3339("2026-08-22T13:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            reviewer_id: "independent-reviewer".to_string(),
            verdict: HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration,
            rationale: "全部边界已经独立复核。".to_string(),
            implementation_fingerprint_confirmed: true,
            current_upstream_bindings_confirmed: true,
            code_revision_reproducible_confirmed: true,
            sealed_input_read_only_confirmed: true,
            deterministic_common_session_replay_confirmed: true,
            isolated_ephemeral_output_confirmed: true,
            resource_bounds_confirmed: true,
            no_network_or_external_tools_confirmed: true,
            no_production_label_training_reward_shadow_writes_confirmed: true,
            no_order_broker_or_trading_confirmed: true,
            reviewer_independent_from_registrant: true,
            isolated_runner_registration_eligible: true,
            run_authorized: false,
            offline_dry_run_enabled: false,
            execution_started: false,
            output_artifact_created: false,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        ApprovedHistoricalOutcomeDryRunRunAuthorization {
            implementation,
            review,
        }
    }

    fn record() -> HistoricalOutcomeDryRunIsolatedRunnerRecord {
        new_record(
            &authorization(),
            "runner-admin",
            "一次性确定性历史结果执行器".to_string(),
            HistoricalOutcomeDryRunIsolatedRunnerKind::EphemeralDeterministicProcess,
            "oldwang@runner123".to_string(),
            "5".repeat(64),
            DateTime::parse_from_rfc3339("2026-08-22T14:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        )
        .unwrap()
    }

    fn rehash(record: &mut HistoricalOutcomeDryRunIsolatedRunnerRecord) {
        record.isolated_runner_spec_sha256 = runner_fingerprint(record).unwrap();
        record.isolated_runner_id = record.isolated_runner_spec_sha256[..32].to_string();
    }

    #[test]
    fn registration_has_no_entrypoint_and_never_runs() {
        let value = record();
        assert!(validate_record(&value).is_ok());
        assert_eq!(value.status, "registered_not_run");
        assert!(!value.callable_entrypoint_registered);
        assert!(!value.invocation_authorized);
        assert!(!value.offline_dry_run_enabled);
        assert!(!value.execution_started);
        assert!(!value.output_artifact_created);
        assert!(!value.outcome_label_generation_enabled);
        assert!(!value.decision_training_authorized);
        assert!(!value.shadow_evidence_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn runner_fingerprint_detects_tampering() {
        let mut value = record();
        value.max_memory_mib = 1_024;
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn exact_approved_review_binding_is_required() {
        let value = record();
        let mut approved = authorization();
        assert!(binding_matches(&value, &approved));
        approved.review.review_sha256 = "9".repeat(64);
        assert!(!binding_matches(&value, &approved));
    }

    #[test]
    fn executable_or_privileged_runner_is_rejected() {
        let mut value = record();
        value.callable_entrypoint_registered = true;
        value.invocation_authorized = true;
        rehash(&mut value);
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn environment_secrets_or_network_are_rejected() {
        let mut value = record();
        value.allowed_environment_variables = vec!["API_KEY".to_string()];
        value.secrets_available = true;
        value.outbound_network_allowed = true;
        rehash(&mut value);
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn duplicate_spec_is_detected_across_registration_metadata() {
        let first = record();
        let mut second = record();
        second.registered_by = "another-admin".to_string();
        second.registered_at = DateTime::parse_from_rfc3339("2026-08-22T15:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        rehash(&mut second);
        assert_ne!(
            first.isolated_runner_spec_sha256,
            second.isolated_runner_spec_sha256
        );
        assert_eq!(
            runner_deduplication_fingerprint(&first).unwrap(),
            runner_deduplication_fingerprint(&second).unwrap()
        );
    }

    #[test]
    fn output_label_or_trading_authority_is_rejected() {
        let mut value = record();
        value.output_artifact_created = true;
        value.outcome_label_generation_enabled = true;
        value.order_generation_allowed = true;
        value.trading_authorized = true;
        rehash(&mut value);
        assert!(validate_record(&value).is_err());
    }
}
