//! One-shot capability-isolated historical outcome dry-run execution attempts.
//!
//! The endpoint consumes one exact, unexpired first-execution authorization
//! before invoking a bounded deterministic function over one revalidated sealed
//! price snapshot. The function receives no filesystem, environment, network,
//! tool, model, production-store, broker or trading capability. Its canonical
//! output is staged through an ephemeral directory, hashed and persisted as
//! untrusted evidence. It is never admitted as an outcome label here.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::historical_outcome_dry_run_first_execution_authorizations::{
    ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization,
    current_unexpired_first_execution_authorizations,
};
use super::historical_outcome_price_snapshots::{
    HistoricalOutcomePriceSnapshot, current_sealed_snapshot_for_execution,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-dry-run-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-execution-attempt-claim-v1";
const RESULT_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-execution-attempt-result-v1";
const OUTPUT_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-untrusted-output-v1";
const EXECUTION_POLICY_VERSION: &str =
    "hone-historical-outcome-dry-run-one-shot-capability-isolated-v1";
const ISOLATION_BACKEND: &str = "bounded-pure-function-no-ambient-capabilities-v1";
const MAX_ERROR_CHARS: usize = 1_200;
const MAX_INPUT_SERIES_POINTS: usize = 2_048;
const MAX_COMMON_SESSION_INPUTS: usize = 1_024;

#[derive(Debug)]
struct CapabilityExecutionFailure {
    message: String,
    ephemeral_directory_removed: bool,
}

impl CapabilityExecutionFailure {
    fn before_staging(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral_directory_removed: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeHistoricalOutcomeDryRunRequest {
    expected_first_execution_authorization_review_id: String,
    expected_first_execution_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct HistoricalOutcomeDryRunMetric {
    pub horizon_market_sessions: u16,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub asset_return: f64,
    pub benchmark_return: f64,
    pub excess_return: f64,
    pub asset_max_drawdown: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct HistoricalOutcomeDryRunUntrustedOutput {
    pub schema_version: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub asset_series_sha256: String,
    pub benchmark_series_sha256: String,
    pub common_session_count: usize,
    pub metrics: Vec<HistoricalOutcomeDryRunMetric>,
    pub deterministic_replay_only: bool,
    pub output_is_untrusted: bool,
    pub outcome_label_written: bool,
    pub training_target_written: bool,
    pub reward_written: bool,
    pub shadow_position_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeDryRunExecutionAttemptClaim {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub authorization_valid_until: DateTime<Utc>,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub runner_code_revision: String,
    pub dry_run_implementation_id: String,
    pub dry_run_implementation_spec_sha256: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub labeler_implementation_id: String,
    pub labeler_implementation_spec_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub max_wall_clock_seconds: u16,
    pub max_memory_mib: u16,
    pub max_cpu_millicores: u16,
    pub max_process_count: u8,
    pub max_output_bytes: u32,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub isolation_backend: String,
    pub artifact_digest_reverified: bool,
    pub sealed_snapshot_revalidated: bool,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub child_process_spawned: bool,
    pub ambient_filesystem_capability_available: bool,
    pub ambient_environment_capability_available: bool,
    pub network_capability_available: bool,
    pub external_tool_capability_available: bool,
    pub production_write_capability_available: bool,
    pub historical_state_mutation_allowed: bool,
    pub outcome_label_writes_allowed: bool,
    pub training_writes_allowed: bool,
    pub reward_writes_allowed: bool,
    pub shadow_writes_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeDryRunExecutionAttemptStatus {
    CompletedWithUntrustedOutput,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeDryRunExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: HistoricalOutcomeDryRunExecutionAttemptStatus,
    pub exit_code: i32,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub output_sha256: Option<String>,
    pub untrusted_output: Option<HistoricalOutcomeDryRunUntrustedOutput>,
    pub ephemeral_directory_removed: bool,
    pub output_structural_validation_completed: bool,
    pub output_independent_validation_authorized: bool,
    pub outcome_label_admission_authorized: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunExecutionAttemptItem {
    pub claim: HistoricalOutcomeDryRunExecutionAttemptClaim,
    pub result: Option<HistoricalOutcomeDryRunExecutionAttemptResult>,
    pub current_authorization_binding: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub isolation_backend: String,
    pub invocation_endpoint_available: bool,
    pub invocation_eligible_authorization_count: usize,
    pub attempt_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_output_count: usize,
    pub execution_status: String,
    pub attempts: Vec<HistoricalOutcomeDryRunExecutionAttemptItem>,
    pub output_independent_validation_authorized: bool,
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
pub(crate) struct HistoricalOutcomeDryRunExecutionAttemptReadinessSummary {
    pub attempt_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_output_count: usize,
    pub execution_status: String,
    pub outcome_label_generation_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompletedHistoricalOutcomeDryRunExecutionAttempt {
    pub claim: HistoricalOutcomeDryRunExecutionAttemptClaim,
    pub result: HistoricalOutcomeDryRunExecutionAttemptResult,
}

pub(crate) async fn handle_get_historical_outcome_dry_run_execution_attempts(
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
            warn!(%error, "historical outcome dry-run execution attempt registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线试运行执行记录暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_historical_outcome_dry_run_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeHistoricalOutcomeDryRunRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match invoke_once(&state, &admin.user_id, &isolated_runner_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome execution registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线试运行执行记录暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_dry_run_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunExecutionAttemptReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeDryRunExecutionAttemptReadinessSummary {
        attempt_count: registry.attempt_count,
        completed_attempt_count: registry.completed_attempt_count,
        failed_attempt_count: registry.failed_attempt_count,
        untrusted_output_count: registry.untrusted_output_count,
        execution_status: registry.execution_status,
        outcome_label_generation_enabled: false,
    })
}

pub(crate) async fn completed_execution_attempt_for_independent_validation(
    state: &AppState,
    attempt_id: &str,
    expected_claim_sha256: &str,
    expected_result_sha256: &str,
    expected_output_sha256: &str,
) -> Result<Option<CompletedHistoricalOutcomeDryRunExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let Some(claim) = claims.into_iter().find(|value| {
        value.attempt_id == attempt_id && value.claim_sha256 == expected_claim_sha256
    }) else {
        return Ok(None);
    };
    let Some(result) = results.into_iter().find(|value| {
        value.attempt_id == attempt_id
            && value.result_sha256 == expected_result_sha256
            && value.output_sha256.as_deref() == Some(expected_output_sha256)
    }) else {
        return Ok(None);
    };
    if result.status != HistoricalOutcomeDryRunExecutionAttemptStatus::CompletedWithUntrustedOutput
        || result.untrusted_output.is_none()
    {
        return Ok(None);
    }
    Ok(Some(CompletedHistoricalOutcomeDryRunExecutionAttempt {
        claim,
        result,
    }))
}

pub(crate) async fn completed_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeDryRunExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .into_iter()
        .filter(|result| {
            result.status
                == HistoricalOutcomeDryRunExecutionAttemptStatus::CompletedWithUntrustedOutput
                && result.untrusted_output.is_some()
                && result.output_sha256.is_some()
        })
        .map(|result| (result.attempt_id.clone(), result))
        .collect::<HashMap<_, _>>();
    let mut output = claims
        .into_iter()
        .filter_map(|claim| {
            result_index
                .get(&claim.attempt_id)
                .cloned()
                .map(|result| CompletedHistoricalOutcomeDryRunExecutionAttempt { claim, result })
        })
        .collect::<Vec<_>>();
    output.sort_by(|left, right| {
        right
            .claim
            .claimed_at
            .cmp(&left.claim.claimed_at)
            .then_with(|| right.claim.attempt_id.cmp(&left.claim.attempt_id))
    });
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunExecutionAttemptRegistry, String> {
    let now = Utc::now();
    let authorizations = current_unexpired_first_execution_authorizations(state, now).await?;
    let authorization_index = authorizations
        .iter()
        .map(|value| (value.review.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .into_iter()
        .map(|result| (result.attempt_id.clone(), result))
        .collect::<HashMap<_, _>>();
    let mut attempts = claims
        .into_iter()
        .map(|claim| {
            let current_authorization_binding = authorization_index
                .get(claim.authorization_review_id.as_str())
                .is_some_and(|authorization| claim_binding_matches(&claim, authorization));
            let result = result_index.get(&claim.attempt_id).cloned();
            HistoricalOutcomeDryRunExecutionAttemptItem {
                claim,
                result,
                current_authorization_binding,
            }
        })
        .collect::<Vec<_>>();
    attempts.sort_by(|left, right| {
        right
            .claim
            .claimed_at
            .cmp(&left.claim.claimed_at)
            .then_with(|| right.claim.attempt_id.cmp(&left.claim.attempt_id))
    });
    let consumed_reviews = attempts
        .iter()
        .map(|item| item.claim.authorization_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let invocation_eligible_authorization_count = authorizations
        .iter()
        .filter(|authorization| !consumed_reviews.contains(authorization.review.review_id.as_str()))
        .count();
    let attempt_count = attempts.len();
    let completed_attempt_count = attempts
        .iter()
        .filter(|item| {
            item.result.as_ref().is_some_and(|result| {
                result.status
                    == HistoricalOutcomeDryRunExecutionAttemptStatus::CompletedWithUntrustedOutput
            })
        })
        .count();
    let failed_attempt_count = attempts
        .iter()
        .filter(|item| {
            item.result.as_ref().is_some_and(|result| {
                result.status
                    == HistoricalOutcomeDryRunExecutionAttemptStatus::FailedAuthorizationConsumed
            })
        })
        .count();
    let untrusted_output_count = attempts
        .iter()
        .filter(|item| {
            item.result
                .as_ref()
                .is_some_and(|result| result.untrusted_output.is_some())
        })
        .count();
    let execution_status = if attempts.iter().any(|item| item.result.is_none()) {
        "execution_claimed_incomplete_fail_closed"
    } else if untrusted_output_count > 0 {
        "untrusted_output_waiting_independent_validation"
    } else if failed_attempt_count > 0 {
        "execution_failed_authorization_consumed"
    } else if invocation_eligible_authorization_count > 0 {
        "ready_for_one_shot_capability_isolated_invocation"
    } else {
        "waiting_for_unexpired_first_execution_authorization"
    };
    Ok(HistoricalOutcomeDryRunExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        invocation_endpoint_available: true,
        invocation_eligible_authorization_count,
        attempt_count,
        completed_attempt_count,
        failed_attempt_count,
        untrusted_output_count,
        execution_status: execution_status.to_string(),
        attempts,
        output_independent_validation_authorized: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "本阶段最多消费一条当前未过期授权，真实运行一次无环境、文件、网络、工具或生产写入能力的确定性历史回放。输出经一次性临时目录落盘、限长并哈希后仍是不可信工件；尚不能成为结果标签、训练、奖励、影子、订单或交易证据。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    admin_id: &str,
    isolated_runner_id: &str,
    request: InvokeHistoricalOutcomeDryRunRequest,
) -> Result<HistoricalOutcomeDryRunExecutionAttemptResult, String> {
    if !valid_id(isolated_runner_id) {
        return Err("隔离执行器 ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-outcome-execution-{isolated_runner_id}"),
    )
    .await?;
    let now = Utc::now();
    let authorization = current_unexpired_first_execution_authorizations(state, now)
        .await?
        .into_iter()
        .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "当前没有这条执行器的未过期一次性授权".to_string())?;
    verify_request(&authorization, &request)?;
    let (existing_claims, _) = read_attempt_records(state).await?;
    if existing_claims.iter().any(|claim| {
        claim.isolated_runner_id == isolated_runner_id
            || claim.authorization_review_id == authorization.review.review_id
    }) {
        return Err("这条执行器或首次执行授权已经消费，不得重放".to_string());
    }
    let runtime_artifact_sha256 = reverify_current_binary_sha256().await?;
    if runtime_artifact_sha256 != authorization.runner.runner_artifact_sha256 {
        return Err("当前运行制品与独立复核的执行器 SHA-256 不一致，拒绝调用".to_string());
    }
    let snapshot = current_sealed_snapshot_for_execution(
        state,
        &authorization.runner.snapshot_id,
        &authorization.runner.snapshot_sha256,
    )
    .await?
    .ok_or_else(|| "封存行情快照已经失效或绑定变化，拒绝调用".to_string())?;
    verify_snapshot_binding(&authorization, &snapshot)?;

    let claimed_at = Utc::now();
    let claim = new_claim(&authorization, admin_id, claimed_at)?;
    write_immutable_json(
        &claim_directory(state, &claim.isolated_runner_id)
            .join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let execution = execute_capability_isolated(&snapshot, claim.max_output_bytes).await;
    let completed_at = Utc::now();
    let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let result = match execution {
        Ok((output, output_sha256, stdout_bytes)) => new_success_result(
            &claim,
            completed_at,
            duration_millis,
            output,
            output_sha256,
            stdout_bytes,
        )?,
        Err(error) => new_failed_result(&claim, completed_at, duration_millis, &error)?,
    };
    write_immutable_json(
        &result_directory(state, &claim.isolated_runner_id)
            .join(format!("{}.json", result.result_id)),
        &result,
    )
    .await?;
    Ok(result)
}

fn verify_request(
    authorization: &ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization,
    request: &InvokeHistoricalOutcomeDryRunRequest,
) -> Result<(), String> {
    if request.expected_first_execution_authorization_review_id != authorization.review.review_id
        || request.expected_first_execution_authorization_review_sha256
            != authorization.review.review_sha256
        || request.expected_isolated_runner_spec_sha256
            != authorization.runner.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != authorization.runner.runner_artifact_sha256
        || request.expected_snapshot_sha256 != authorization.runner.snapshot_sha256
        || request.expected_protocol_sha256 != authorization.runner.protocol_sha256
    {
        return Err("首次执行授权、执行器制品或上游证据已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn verify_snapshot_binding(
    authorization: &ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization,
    snapshot: &HistoricalOutcomePriceSnapshot,
) -> Result<(), String> {
    let runner = &authorization.runner;
    if snapshot.snapshot_id != runner.snapshot_id
        || snapshot.snapshot_sha256 != runner.snapshot_sha256
        || snapshot.reconstruction_id != runner.reconstruction_id
        || snapshot.reconstruction_sha256 != runner.reconstruction_sha256
        || snapshot.implementation_id != runner.labeler_implementation_id
        || snapshot.implementation_spec_sha256 != runner.labeler_implementation_spec_sha256
        || snapshot.protocol_version != runner.protocol_version
        || snapshot.protocol_sha256 != runner.protocol_sha256
        || snapshot.asset_symbol != runner.asset_symbol
        || snapshot.benchmark_symbol != runner.benchmark_symbol
        || snapshot.asset_series_sha256 != runner.asset_series_sha256
        || snapshot.benchmark_series_sha256 != runner.benchmark_series_sha256
        || snapshot.common_session_count != runner.common_session_count
        || snapshot.covered_horizons_market_sessions != runner.covered_horizons_market_sessions
    {
        return Err("执行器与重新读取的封存行情快照绑定不一致".to_string());
    }
    if snapshot.asset_points.len() > MAX_INPUT_SERIES_POINTS
        || snapshot.benchmark_points.len() > MAX_INPUT_SERIES_POINTS
        || snapshot.common_session_count > MAX_COMMON_SESSION_INPUTS
    {
        return Err("封存行情快照超过能力隔离执行器的静态输入上限".to_string());
    }
    Ok(())
}

fn new_claim(
    authorization: &ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization,
    admin_id: &str,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeDryRunExecutionAttemptClaim, String> {
    let runner = &authorization.runner;
    let review = &authorization.review;
    let mut claim = HistoricalOutcomeDryRunExecutionAttemptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        attempt_id: String::new(),
        claim_sha256: String::new(),
        authorization_review_id: review.review_id.clone(),
        authorization_review_sha256: review.review_sha256.clone(),
        authorization_valid_until: review.authorization_valid_until,
        isolated_runner_id: runner.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
        runner_code_revision: runner.runner_code_revision.clone(),
        dry_run_implementation_id: runner.dry_run_implementation_id.clone(),
        dry_run_implementation_spec_sha256: runner.dry_run_implementation_spec_sha256.clone(),
        snapshot_id: runner.snapshot_id.clone(),
        snapshot_sha256: runner.snapshot_sha256.clone(),
        reconstruction_id: runner.reconstruction_id.clone(),
        reconstruction_sha256: runner.reconstruction_sha256.clone(),
        labeler_implementation_id: runner.labeler_implementation_id.clone(),
        labeler_implementation_spec_sha256: runner.labeler_implementation_spec_sha256.clone(),
        protocol_version: runner.protocol_version.clone(),
        protocol_sha256: runner.protocol_sha256.clone(),
        max_wall_clock_seconds: runner.max_wall_clock_seconds,
        max_memory_mib: runner.max_memory_mib,
        max_cpu_millicores: runner.max_cpu_millicores,
        max_process_count: runner.max_process_count,
        max_output_bytes: runner.max_output_bytes,
        claimed_at,
        invoked_by: admin_id.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        artifact_digest_reverified: true,
        sealed_snapshot_revalidated: true,
        authorization_consumed: true,
        invocation_started: true,
        child_process_spawned: false,
        ambient_filesystem_capability_available: false,
        ambient_environment_capability_available: false,
        network_capability_available: false,
        external_tool_capability_available: false,
        production_write_capability_available: false,
        historical_state_mutation_allowed: false,
        outcome_label_writes_allowed: false,
        training_writes_allowed: false,
        reward_writes_allowed: false,
        shadow_writes_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = claim_fingerprint(&claim)?;
    claim.attempt_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim)?;
    Ok(claim)
}

async fn execute_capability_isolated(
    snapshot: &HistoricalOutcomePriceSnapshot,
    max_output_bytes: u32,
) -> Result<(HistoricalOutcomeDryRunUntrustedOutput, String, usize), CapabilityExecutionFailure> {
    let output =
        compute_untrusted_output(snapshot).map_err(CapabilityExecutionFailure::before_staging)?;
    let bytes = serde_json::to_vec(&output)
        .map_err(|error| CapabilityExecutionFailure::before_staging(error.to_string()))?;
    if bytes.len() > max_output_bytes as usize {
        return Err(CapabilityExecutionFailure::before_staging(
            "确定性输出超过执行器登记上限",
        ));
    }
    let directory = std::env::temp_dir().join(format!(
        "hone-historical-outcome-dry-run-{}",
        Uuid::new_v4()
    ));
    tokio::fs::create_dir(&directory)
        .await
        .map_err(|error| CapabilityExecutionFailure {
            message: format!("一次性输出目录创建失败：{error}"),
            ephemeral_directory_removed: false,
        })?;
    let path = directory.join("untrusted-output.json");
    let staged = async {
        let mut options = tokio::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = options
            .open(&path)
            .await
            .map_err(|error| error.to_string())?;
        use tokio::io::AsyncWriteExt;
        file.write_all(&bytes)
            .await
            .map_err(|error| error.to_string())?;
        file.sync_all().await.map_err(|error| error.to_string())?;
        let read_back = tokio::fs::read(&path)
            .await
            .map_err(|error| error.to_string())?;
        if read_back != bytes {
            return Err("一次性输出回读与内存输出不一致".to_string());
        }
        Ok::<_, String>(format!("{:x}", Sha256::digest(&read_back)))
    }
    .await;
    let cleanup = tokio::fs::remove_dir_all(&directory).await;
    let ephemeral_directory_removed = cleanup.is_ok()
        || tokio::fs::metadata(&directory)
            .await
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
    match (staged, cleanup, ephemeral_directory_removed) {
        (Ok(output_sha256), Ok(()), true) => Ok((output, output_sha256, bytes.len())),
        (Ok(_), Err(error), removed) => Err(CapabilityExecutionFailure {
            message: format!("一次性输出目录清理失败：{error}"),
            ephemeral_directory_removed: removed,
        }),
        (Err(error), _, removed) => Err(CapabilityExecutionFailure {
            message: error,
            ephemeral_directory_removed: removed,
        }),
        (Ok(_), Ok(()), false) => Err(CapabilityExecutionFailure {
            message: "一次性输出目录清理状态无法确认".to_string(),
            ephemeral_directory_removed: false,
        }),
    }
}

fn compute_untrusted_output(
    snapshot: &HistoricalOutcomePriceSnapshot,
) -> Result<HistoricalOutcomeDryRunUntrustedOutput, String> {
    if snapshot.asset_points.len() > MAX_INPUT_SERIES_POINTS
        || snapshot.benchmark_points.len() > MAX_INPUT_SERIES_POINTS
        || snapshot.common_session_count > MAX_COMMON_SESSION_INPUTS
    {
        return Err("封存行情快照超过能力隔离执行器的静态输入上限".to_string());
    }
    let benchmark = snapshot
        .benchmark_points
        .iter()
        .map(|point| (point.date, point.adjusted_close))
        .collect::<BTreeMap<_, _>>();
    let common = snapshot
        .asset_points
        .iter()
        .filter_map(|asset| {
            benchmark
                .get(&asset.date)
                .copied()
                .map(|benchmark_close| (asset.date, asset.adjusted_close, benchmark_close))
        })
        .collect::<Vec<_>>();
    if common.len() != snapshot.common_session_count || common.len() <= 250 {
        return Err("共同交易日序列与封存快照不一致或覆盖不足".to_string());
    }
    let (start_date, start_asset, start_benchmark) = common[0];
    let mut metrics = Vec::new();
    for horizon in &snapshot.covered_horizons_market_sessions {
        let index = usize::from(*horizon);
        let Some((end_date, end_asset, end_benchmark)) = common.get(index).copied() else {
            return Err("共同交易日序列不足以计算登记窗口".to_string());
        };
        let asset_return = end_asset / start_asset - 1.0;
        let benchmark_return = end_benchmark / start_benchmark - 1.0;
        let mut peak = start_asset;
        let mut max_drawdown = 0.0_f64;
        for (_, price, _) in &common[..=index] {
            peak = peak.max(*price);
            max_drawdown = max_drawdown.min(*price / peak - 1.0);
        }
        if !asset_return.is_finite() || !benchmark_return.is_finite() || !max_drawdown.is_finite() {
            return Err("确定性结果包含非有限数值".to_string());
        }
        metrics.push(HistoricalOutcomeDryRunMetric {
            horizon_market_sessions: *horizon,
            start_date,
            end_date,
            asset_return,
            benchmark_return,
            excess_return: asset_return - benchmark_return,
            asset_max_drawdown: max_drawdown,
        });
    }
    if metrics.len() != 3
        || metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect::<Vec<_>>()
            != vec![20, 60, 250]
    {
        return Err("执行器只接受预冻结的 20/60/250 共同交易日窗口".to_string());
    }
    Ok(HistoricalOutcomeDryRunUntrustedOutput {
        schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        snapshot_id: snapshot.snapshot_id.clone(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        protocol_version: snapshot.protocol_version.clone(),
        protocol_sha256: snapshot.protocol_sha256.clone(),
        asset_symbol: snapshot.asset_symbol.clone(),
        benchmark_symbol: snapshot.benchmark_symbol.clone(),
        asset_series_sha256: snapshot.asset_series_sha256.clone(),
        benchmark_series_sha256: snapshot.benchmark_series_sha256.clone(),
        common_session_count: common.len(),
        metrics,
        deterministic_replay_only: true,
        output_is_untrusted: true,
        outcome_label_written: false,
        training_target_written: false,
        reward_written: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    })
}

fn new_success_result(
    claim: &HistoricalOutcomeDryRunExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    output: HistoricalOutcomeDryRunUntrustedOutput,
    output_sha256: String,
    stdout_bytes: usize,
) -> Result<HistoricalOutcomeDryRunExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeDryRunExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeDryRunExecutionAttemptStatus::CompletedWithUntrustedOutput,
        exit_code: 0,
        stdout_sha256: output_sha256.clone(),
        stderr_sha256: hash_bytes(&[]),
        stdout_bytes,
        stderr_bytes: 0,
        output_sha256: Some(output_sha256),
        untrusted_output: Some(output),
        ephemeral_directory_removed: true,
        output_structural_validation_completed: false,
        output_independent_validation_authorized: false,
        outcome_label_admission_authorized: false,
        outcome_label_written: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn new_failed_result(
    claim: &HistoricalOutcomeDryRunExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    error: &CapabilityExecutionFailure,
) -> Result<HistoricalOutcomeDryRunExecutionAttemptResult, String> {
    let bounded = error
        .message
        .chars()
        .take(MAX_ERROR_CHARS)
        .collect::<String>();
    let stderr = bounded.as_bytes();
    let mut result = HistoricalOutcomeDryRunExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeDryRunExecutionAttemptStatus::FailedAuthorizationConsumed,
        exit_code: 1,
        stdout_sha256: hash_bytes(&[]),
        stderr_sha256: hash_bytes(stderr),
        stdout_bytes: 0,
        stderr_bytes: stderr.len(),
        output_sha256: None,
        untrusted_output: None,
        ephemeral_directory_removed: error.ephemeral_directory_removed,
        output_structural_validation_completed: false,
        output_independent_validation_authorized: false,
        outcome_label_admission_authorized: false,
        outcome_label_written: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn validate_claim(claim: &HistoricalOutcomeDryRunExecutionAttemptClaim) -> Result<(), String> {
    let capabilities_closed = !claim.child_process_spawned
        && !claim.ambient_filesystem_capability_available
        && !claim.ambient_environment_capability_available
        && !claim.network_capability_available
        && !claim.external_tool_capability_available
        && !claim.production_write_capability_available
        && !claim.historical_state_mutation_allowed
        && !claim.outcome_label_writes_allowed
        && !claim.training_writes_allowed
        && !claim.reward_writes_allowed
        && !claim.shadow_writes_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed;
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&claim.attempt_id)
        || !valid_sha256(&claim.claim_sha256)
        || claim.attempt_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != claim_fingerprint(claim)?
        || !valid_id(&claim.authorization_review_id)
        || !valid_sha256(&claim.authorization_review_sha256)
        || claim.claimed_at >= claim.authorization_valid_until
        || !valid_id(&claim.isolated_runner_id)
        || !valid_sha256(&claim.isolated_runner_spec_sha256)
        || !valid_sha256(&claim.runner_artifact_sha256)
        || claim.runner_code_revision.trim().is_empty()
        || !valid_id(&claim.dry_run_implementation_id)
        || !valid_sha256(&claim.dry_run_implementation_spec_sha256)
        || !valid_id(&claim.snapshot_id)
        || !valid_sha256(&claim.snapshot_sha256)
        || !valid_id(&claim.reconstruction_id)
        || !valid_sha256(&claim.reconstruction_sha256)
        || !valid_id(&claim.labeler_implementation_id)
        || !valid_sha256(&claim.labeler_implementation_spec_sha256)
        || claim.protocol_version.trim().is_empty()
        || !valid_sha256(&claim.protocol_sha256)
        || claim.max_wall_clock_seconds != 300
        || claim.max_memory_mib != 512
        || claim.max_cpu_millicores != 1_000
        || claim.max_process_count != 1
        || claim.max_output_bytes != 1_048_576
        || claim.invoked_by.trim().is_empty()
        || claim.isolation_backend != ISOLATION_BACKEND
        || !claim.artifact_digest_reverified
        || !claim.sealed_snapshot_revalidated
        || !claim.authorization_consumed
        || !claim.invocation_started
        || !capabilities_closed
    {
        return Err("historical outcome execution claim is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeDryRunExecutionAttemptResult,
    claim: &HistoricalOutcomeDryRunExecutionAttemptClaim,
) -> Result<(), String> {
    let downstream_closed = !result.output_structural_validation_completed
        && !result.output_independent_validation_authorized
        && !result.outcome_label_admission_authorized
        && !result.outcome_label_written
        && !result.training_authorized
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    let output_consistent = match (
        &result.status,
        &result.untrusted_output,
        &result.output_sha256,
    ) {
        (
            HistoricalOutcomeDryRunExecutionAttemptStatus::CompletedWithUntrustedOutput,
            Some(output),
            Some(output_sha256),
        ) => {
            result.exit_code == 0
                && result.stdout_bytes <= claim.max_output_bytes as usize
                && result.stderr_bytes == 0
                && result.stderr_sha256 == hash_bytes(&[])
                && output.output_is_untrusted
                && !output.outcome_label_written
                && !output.training_target_written
                && !output.reward_written
                && !output.shadow_position_written
                && !output.order_generated
                && !output.broker_accessed
                && !output.trade_executed
                && hash_serializable(output).is_ok_and(|hash| hash == *output_sha256)
                && result.stdout_sha256 == *output_sha256
        }
        (
            HistoricalOutcomeDryRunExecutionAttemptStatus::FailedAuthorizationConsumed,
            None,
            None,
        ) => {
            result.exit_code != 0
                && result.stdout_bytes == 0
                && result.stdout_sha256 == hash_bytes(&[])
                && result.stderr_bytes <= MAX_ERROR_CHARS * 4
        }
        _ => false,
    };
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != result_fingerprint(result)?
        || result.attempt_id != claim.attempt_id
        || result.claim_sha256 != claim.claim_sha256
        || result.completed_at < claim.claimed_at
        || result.duration_millis > u64::from(claim.max_wall_clock_seconds) * 1_000
        || !valid_sha256(&result.stdout_sha256)
        || !valid_sha256(&result.stderr_sha256)
        || result
            .output_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256(value))
        || (result.status
            == HistoricalOutcomeDryRunExecutionAttemptStatus::CompletedWithUntrustedOutput
            && !result.ephemeral_directory_removed)
        || !output_consistent
        || !downstream_closed
    {
        return Err(
            "historical outcome execution result is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn claim_binding_matches(
    claim: &HistoricalOutcomeDryRunExecutionAttemptClaim,
    authorization: &ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization,
) -> bool {
    claim.authorization_review_id == authorization.review.review_id
        && claim.authorization_review_sha256 == authorization.review.review_sha256
        && claim.authorization_valid_until == authorization.review.authorization_valid_until
        && claim.isolated_runner_id == authorization.runner.isolated_runner_id
        && claim.isolated_runner_spec_sha256 == authorization.runner.isolated_runner_spec_sha256
        && claim.runner_artifact_sha256 == authorization.runner.runner_artifact_sha256
        && claim.runner_code_revision == authorization.runner.runner_code_revision
        && claim.dry_run_implementation_id == authorization.runner.dry_run_implementation_id
        && claim.dry_run_implementation_spec_sha256
            == authorization.runner.dry_run_implementation_spec_sha256
        && claim.snapshot_id == authorization.runner.snapshot_id
        && claim.snapshot_sha256 == authorization.runner.snapshot_sha256
        && claim.reconstruction_id == authorization.runner.reconstruction_id
        && claim.reconstruction_sha256 == authorization.runner.reconstruction_sha256
        && claim.labeler_implementation_id == authorization.runner.labeler_implementation_id
        && claim.labeler_implementation_spec_sha256
            == authorization.runner.labeler_implementation_spec_sha256
        && claim.protocol_version == authorization.runner.protocol_version
        && claim.protocol_sha256 == authorization.runner.protocol_sha256
}

fn claim_fingerprint(
    claim: &HistoricalOutcomeDryRunExecutionAttemptClaim,
) -> Result<String, String> {
    let mut value = serde_json::to_value(claim).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "execution claim fingerprint payload is invalid".to_string())?;
    object.remove("attempt_id");
    object.remove("claim_sha256");
    hash_serializable(&value)
}

fn result_fingerprint(
    result: &HistoricalOutcomeDryRunExecutionAttemptResult,
) -> Result<String, String> {
    let mut value = serde_json::to_value(result).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "execution result fingerprint payload is invalid".to_string())?;
    object.remove("result_id");
    object.remove("result_sha256");
    hash_serializable(&value)
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<HistoricalOutcomeDryRunExecutionAttemptClaim>,
        Vec<HistoricalOutcomeDryRunExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<HistoricalOutcomeDryRunExecutionAttemptClaim>(
        &execution_root(state).join("claims"),
    )
    .await?;
    let results = read_nested_json::<HistoricalOutcomeDryRunExecutionAttemptResult>(
        &execution_root(state).join("results"),
    )
    .await?;
    let mut claim_ids = BTreeSet::new();
    let mut claim_hashes = BTreeSet::new();
    let mut runner_ids = BTreeSet::new();
    let mut authorization_ids = BTreeSet::new();
    for claim in &claims {
        validate_claim(claim)?;
        if !claim_ids.insert(claim.attempt_id.clone())
            || !claim_hashes.insert(claim.claim_sha256.clone())
            || !runner_ids.insert(claim.isolated_runner_id.clone())
            || !authorization_ids.insert(claim.authorization_review_id.clone())
        {
            return Err(
                "historical outcome execution registry contains replayed claims".to_string(),
            );
        }
    }
    let claim_index = claims
        .iter()
        .map(|claim| (claim.attempt_id.as_str(), claim))
        .collect::<HashMap<_, _>>();
    let mut result_ids = BTreeSet::new();
    let mut result_hashes = BTreeSet::new();
    let mut completed_attempts = BTreeSet::new();
    for result in &results {
        let claim = claim_index.get(result.attempt_id.as_str()).ok_or_else(|| {
            "historical outcome execution result has no immutable claim".to_string()
        })?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_hashes.insert(result.result_sha256.clone())
            || !completed_attempts.insert(result.attempt_id.clone())
        {
            return Err(
                "historical outcome execution registry contains duplicate results".to_string(),
            );
        }
    }
    Ok((claims, results))
}

async fn read_nested_json<T>(root: &Path) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut directories = match tokio::fs::read_dir(root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
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
            let bytes = tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?;
            output.push(serde_json::from_slice(&bytes).map_err(|error| error.to_string())?);
        }
    }
    Ok(output)
}

async fn reverify_current_binary_sha256() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut file = File::open(executable).map_err(|error| error.to_string())?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok::<_, String>(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|error| error.to_string())?
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn execution_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-dry-run-execution-attempts")
}

fn claim_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("claims").join(runner_id)
}

fn result_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("results").join(runner_id)
}

struct HistoricalOutcomeDryRunExecutionLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeDryRunExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<HistoricalOutcomeDryRunExecutionLock, String> {
    if !valid_id(name) {
        return Err("invalid historical outcome execution lock name".to_string());
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
            Ok(_) => return Ok(HistoricalOutcomeDryRunExecutionLock { path }),
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
                return Err("首次执行正在由另一请求消费，请稍后读取结果".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("首次执行锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid historical outcome execution path".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path).await.map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            "historical outcome execution record already exists".to_string()
        } else {
            error.to_string()
        }
    })?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::super::historical_outcome_price_snapshots::SealedAdjustedClosePoint;
    use super::*;

    fn points(values: &[f64]) -> Vec<SealedAdjustedClosePoint> {
        let start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        values
            .iter()
            .enumerate()
            .map(|(index, value)| SealedAdjustedClosePoint {
                date: start + chrono::Duration::days(index as i64),
                adjusted_close: *value,
            })
            .collect()
    }

    fn snapshot() -> HistoricalOutcomePriceSnapshot {
        let asset = (0..=250)
            .map(|index| {
                if index == 30 {
                    80.0
                } else {
                    100.0 + index as f64
                }
            })
            .collect::<Vec<_>>();
        let benchmark = (0..=250)
            .map(|index| 100.0 + index as f64 * 0.5)
            .collect::<Vec<_>>();
        HistoricalOutcomePriceSnapshot {
            schema_version: "hone-historical-outcome-price-snapshot-v1".to_string(),
            ingestion_policy_version: "hone-historical-outcome-price-ingestion-v1".to_string(),
            snapshot_id: "snapshot-test".to_string(),
            snapshot_sha256: "a".repeat(64),
            sealed_at: Utc::now(),
            sealed_by: "admin".to_string(),
            reconstruction_id: "reconstruction-test".to_string(),
            reconstruction_sha256: "b".repeat(64),
            reconstruction_review_id: "reconstruction-review".to_string(),
            anchor_candidate_id: "anchor-test".to_string(),
            anchor_review_id: "anchor-review".to_string(),
            decision_available_at: Utc::now(),
            implementation_id: "labeler-test".to_string(),
            implementation_spec_sha256: "c".repeat(64),
            implementation_review_id: "labeler-review".to_string(),
            governance_review_id: "governance-review".to_string(),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "d".repeat(64),
            code_revision: "revision".to_string(),
            provider: "Financial Modeling Prep".to_string(),
            provider_endpoint_template: "template".to_string(),
            price_basis: "FMP adjusted close".to_string(),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            requested_from: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            requested_to: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            asset_payload_sha256: "e".repeat(64),
            benchmark_payload_sha256: "f".repeat(64),
            asset_series_sha256: "1".repeat(64),
            benchmark_series_sha256: "2".repeat(64),
            asset_points: points(&asset),
            benchmark_points: points(&benchmark),
            common_session_count: 251,
            covered_horizons_market_sessions: vec![20, 60, 250],
            all_protocol_horizons_covered: true,
            outcome_metrics_computed: false,
            label_written: false,
            historical_state_mutated: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        }
    }

    fn claim() -> HistoricalOutcomeDryRunExecutionAttemptClaim {
        let mut value = HistoricalOutcomeDryRunExecutionAttemptClaim {
            schema_version: CLAIM_SCHEMA_VERSION.to_string(),
            execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
            attempt_id: String::new(),
            claim_sha256: String::new(),
            authorization_review_id: "authorization-review".to_string(),
            authorization_review_sha256: "a".repeat(64),
            authorization_valid_until: DateTime::parse_from_rfc3339("2026-08-23T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            isolated_runner_id: "runner-test".to_string(),
            isolated_runner_spec_sha256: "b".repeat(64),
            runner_artifact_sha256: "c".repeat(64),
            runner_code_revision: "revision".to_string(),
            dry_run_implementation_id: "implementation-test".to_string(),
            dry_run_implementation_spec_sha256: "d".repeat(64),
            snapshot_id: "snapshot-test".to_string(),
            snapshot_sha256: "e".repeat(64),
            reconstruction_id: "reconstruction-test".to_string(),
            reconstruction_sha256: "f".repeat(64),
            labeler_implementation_id: "labeler-test".to_string(),
            labeler_implementation_spec_sha256: "1".repeat(64),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "2".repeat(64),
            max_wall_clock_seconds: 300,
            max_memory_mib: 512,
            max_cpu_millicores: 1_000,
            max_process_count: 1,
            max_output_bytes: 1_048_576,
            claimed_at: DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            invoked_by: "operator".to_string(),
            isolation_backend: ISOLATION_BACKEND.to_string(),
            artifact_digest_reverified: true,
            sealed_snapshot_revalidated: true,
            authorization_consumed: true,
            invocation_started: true,
            child_process_spawned: false,
            ambient_filesystem_capability_available: false,
            ambient_environment_capability_available: false,
            network_capability_available: false,
            external_tool_capability_available: false,
            production_write_capability_available: false,
            historical_state_mutation_allowed: false,
            outcome_label_writes_allowed: false,
            training_writes_allowed: false,
            reward_writes_allowed: false,
            shadow_writes_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
        value.claim_sha256 = claim_fingerprint(&value).unwrap();
        value.attempt_id = value.claim_sha256[..32].to_string();
        value
    }

    #[test]
    fn deterministic_output_uses_common_sessions_and_fixed_horizons() {
        let output = compute_untrusted_output(&snapshot()).unwrap();
        assert_eq!(output.metrics.len(), 3);
        assert_eq!(output.metrics[0].horizon_market_sessions, 20);
        assert!((output.metrics[0].asset_return - 0.2).abs() < 1e-12);
        assert!((output.metrics[0].benchmark_return - 0.1).abs() < 1e-12);
        assert!((output.metrics[1].asset_max_drawdown - (80.0 / 129.0 - 1.0)).abs() < 1e-12);
        assert!(output.output_is_untrusted);
        assert!(!output.outcome_label_written);
    }

    #[test]
    fn missing_common_session_coverage_fails_closed() {
        let mut value = snapshot();
        value.asset_points.truncate(250);
        value.common_session_count = 250;
        assert!(compute_untrusted_output(&value).is_err());
    }

    #[test]
    fn oversized_input_fails_before_capability_isolated_staging() {
        let mut value = snapshot();
        let last = value.asset_points.last().cloned().unwrap();
        value.asset_points.resize(MAX_INPUT_SERIES_POINTS + 1, last);
        assert!(compute_untrusted_output(&value).is_err());
    }

    #[test]
    fn successful_result_remains_untrusted_and_non_authorizing() {
        let claim = claim();
        let output = compute_untrusted_output(&snapshot()).unwrap();
        let output_sha = hash_serializable(&output).unwrap();
        let result = new_success_result(
            &claim,
            claim.claimed_at + chrono::Duration::milliseconds(10),
            10,
            output,
            output_sha,
            1_000,
        )
        .unwrap();
        assert_eq!(
            result.status,
            HistoricalOutcomeDryRunExecutionAttemptStatus::CompletedWithUntrustedOutput
        );
        assert!(!result.output_structural_validation_completed);
        assert!(!result.outcome_label_admission_authorized);
        assert!(!result.training_authorized);
        assert!(!result.trading_authorized);
    }

    #[test]
    fn failed_result_consumes_authorization_without_output() {
        let claim = claim();
        let result = new_failed_result(
            &claim,
            claim.claimed_at + chrono::Duration::milliseconds(5),
            5,
            &CapabilityExecutionFailure::before_staging("fixture failure"),
        )
        .unwrap();
        assert_eq!(
            result.status,
            HistoricalOutcomeDryRunExecutionAttemptStatus::FailedAuthorizationConsumed
        );
        assert!(result.untrusted_output.is_none());
        assert!(claim.authorization_consumed);
    }

    #[test]
    fn cleanup_failure_is_preserved_in_failed_result() {
        let claim = claim();
        let result = new_failed_result(
            &claim,
            claim.claimed_at + chrono::Duration::milliseconds(5),
            5,
            &CapabilityExecutionFailure {
                message: "cleanup failure".to_string(),
                ephemeral_directory_removed: false,
            },
        )
        .unwrap();
        assert!(!result.ephemeral_directory_removed);
        assert_eq!(
            result.status,
            HistoricalOutcomeDryRunExecutionAttemptStatus::FailedAuthorizationConsumed
        );
    }

    #[test]
    fn claim_tampering_is_rejected() {
        let mut value = claim();
        value.network_capability_available = true;
        assert!(validate_claim(&value).is_err());
    }

    #[test]
    fn output_tampering_is_rejected() {
        let claim = claim();
        let output = compute_untrusted_output(&snapshot()).unwrap();
        let output_sha = hash_serializable(&output).unwrap();
        let mut result = new_success_result(
            &claim,
            claim.claimed_at + chrono::Duration::milliseconds(10),
            10,
            output,
            output_sha,
            1_000,
        )
        .unwrap();
        result.training_authorized = true;
        assert!(validate_result(&result, &claim).is_err());
    }

    #[test]
    fn duration_and_output_limits_fail_closed() {
        let claim = claim();
        let output = compute_untrusted_output(&snapshot()).unwrap();
        let output_sha = hash_serializable(&output).unwrap();
        let result = new_success_result(
            &claim,
            claim.claimed_at + chrono::Duration::seconds(301),
            301_000,
            output,
            output_sha,
            1_000,
        )
        .unwrap_err();
        assert!(result.contains("invalid"));
    }
}
