//! One-shot historical outcome-label materialization execution attempts.
//!
//! This stage consumes one exact, current and unexpired stage-seventeen
//! authorization. A create-once claim is persisted before a fixed pure
//! projection copies independently validated raw metrics, provenance and known
//! limitations into an untrusted envelope. The projection receives no ambient
//! filesystem, environment, network, tool, model, production-store, broker or
//! trading capability. The envelope is not a label and requires a later,
//! independent structural/provenance/bitwise validation.

use std::collections::{BTreeSet, HashMap};
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

use super::historical_outcome_dry_run_execution_attempts::HistoricalOutcomeDryRunMetric;
use super::historical_outcome_label_admission_reviews::{
    AdmittedHistoricalOutcomeForLabelMaterialization,
    exact_admitted_historical_outcome_for_label_materialization,
};
use super::historical_outcome_label_materialization_first_execution_authorizations::{
    ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
    current_unexpired_first_execution_authorizations,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-execution-attempt-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-execution-attempt-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-untrusted-envelope-v1";
const EXECUTION_POLICY_VERSION: &str =
    "hone-historical-outcome-label-materialization-one-shot-pure-projection-v1";
const ISOLATION_BACKEND: &str =
    "fixed-raw-validated-outcome-envelope-pure-function-no-ambient-capabilities-v1";
const MAX_ERROR_CHARS: usize = 1_200;
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];

#[derive(Debug)]
struct MaterializationFailure {
    message: String,
    ephemeral_directory_removed: bool,
}

impl MaterializationFailure {
    fn before_staging(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral_directory_removed: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeHistoricalOutcomeLabelMaterializationOnceRequest {
    expected_first_execution_authorization_review_id: String,
    expected_first_execution_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_spec_sha256: String,
    expected_admission_review_sha256: String,
    expected_validation_sha256: String,
    expected_source_output_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    expected_recomputed_metrics_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct HistoricalOutcomeLabelMaterializationUntrustedEnvelope {
    pub schema_version: String,
    pub output_label_schema_version: String,
    pub materialization_implementation_id: String,
    pub materialization_implementation_spec_sha256: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub source_attempt_id: String,
    pub source_claim_sha256: String,
    pub source_result_id: String,
    pub source_result_sha256: String,
    pub source_output_sha256: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub common_session_count: usize,
    pub metric_start_date: NaiveDate,
    pub metric_end_dates: Vec<NaiveDate>,
    pub recomputed_metrics_sha256: String,
    pub raw_validated_metrics: Vec<HistoricalOutcomeDryRunMetric>,
    pub known_limitations: String,
    pub deterministic_projection_only: bool,
    pub exact_metric_bits_preserved: bool,
    pub provenance_preserved: bool,
    pub known_limitations_preserved: bool,
    pub output_is_untrusted: bool,
    pub independent_validation_completed: bool,
    pub outcome_label_write_allowed: bool,
    pub outcome_label_written: bool,
    pub direction_inferred: bool,
    pub rating_inferred: bool,
    pub investment_action_inferred: bool,
    pub position_size_inferred: bool,
    pub training_target_written: bool,
    pub reward_written: bool,
    pub shadow_position_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationExecutionAttemptClaim {
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
    pub materialization_implementation_id: String,
    pub materialization_implementation_spec_sha256: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub source_attempt_id: String,
    pub source_output_sha256: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub recomputed_metrics_sha256: String,
    pub max_wall_clock_seconds: u16,
    pub max_memory_mib: u16,
    pub max_cpu_millicores: u16,
    pub max_process_count: u8,
    pub max_output_bytes: u32,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub isolation_backend: String,
    pub artifact_digest_reverified: bool,
    pub current_admission_chain_revalidated: bool,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub child_process_spawned: bool,
    pub ambient_filesystem_capability_available: bool,
    pub ambient_environment_capability_available: bool,
    pub network_capability_available: bool,
    pub external_tool_capability_available: bool,
    pub production_data_capability_available_to_projection: bool,
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
pub(crate) enum HistoricalOutcomeLabelMaterializationExecutionAttemptStatus {
    CompletedWithUntrustedEnvelope,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: HistoricalOutcomeLabelMaterializationExecutionAttemptStatus,
    pub exit_code: i32,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub output_sha256: Option<String>,
    pub untrusted_envelope: Option<HistoricalOutcomeLabelMaterializationUntrustedEnvelope>,
    pub ephemeral_directory_removed: bool,
    pub independent_validation_completed: bool,
    pub outcome_label_admission_authorized: bool,
    pub outcome_label_write_allowed: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationExecutionAttemptItem {
    pub claim: HistoricalOutcomeLabelMaterializationExecutionAttemptClaim,
    pub result: Option<HistoricalOutcomeLabelMaterializationExecutionAttemptResult>,
    pub current_authorization_binding: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub isolation_backend: String,
    pub invocation_endpoint_available: bool,
    pub invocation_eligible_authorization_count: usize,
    pub attempt_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_envelope_count: usize,
    pub independent_validation_eligible_count: usize,
    pub execution_status: String,
    pub attempts: Vec<HistoricalOutcomeLabelMaterializationExecutionAttemptItem>,
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
pub(crate) struct HistoricalOutcomeLabelMaterializationExecutionAttemptReadinessSummary {
    pub attempt_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_envelope_count: usize,
    pub independent_validation_eligible_count: usize,
    pub execution_status: String,
    pub outcome_label_generation_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt {
    pub claim: HistoricalOutcomeLabelMaterializationExecutionAttemptClaim,
    pub result: HistoricalOutcomeLabelMaterializationExecutionAttemptResult,
}

pub(crate) async fn handle_get_historical_outcome_label_materialization_execution_attempts(
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
            warn!(%error, "historical outcome label materialization execution registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果标签物化执行记录暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_historical_outcome_label_materialization_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeHistoricalOutcomeLabelMaterializationOnceRequest>,
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
                warn!(%error, "historical outcome label materialization execution registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果标签物化执行记录暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_label_materialization_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelMaterializationExecutionAttemptReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeLabelMaterializationExecutionAttemptReadinessSummary {
            attempt_count: registry.attempt_count,
            completed_attempt_count: registry.completed_attempt_count,
            failed_attempt_count: registry.failed_attempt_count,
            untrusted_envelope_count: registry.untrusted_envelope_count,
            independent_validation_eligible_count: registry.independent_validation_eligible_count,
            execution_status: registry.execution_status,
            outcome_label_generation_enabled: false,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn completed_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .into_iter()
        .filter(|result| {
            result.status
                == HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::CompletedWithUntrustedEnvelope
                && result.untrusted_envelope.is_some()
                && result.output_sha256.is_some()
        })
        .map(|result| (result.attempt_id.clone(), result))
        .collect::<HashMap<_, _>>();
    let mut output = claims
        .into_iter()
        .filter_map(|claim| {
            result_index.get(&claim.attempt_id).cloned().map(|result| {
                CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt { claim, result }
            })
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

pub(crate) async fn completed_execution_attempt_for_independent_validation(
    state: &AppState,
    attempt_id: &str,
    claim_sha256: &str,
    result_sha256: &str,
    output_sha256: &str,
) -> Result<Option<CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt>, String> {
    Ok(
        completed_execution_attempts_for_independent_validation(state)
            .await?
            .into_iter()
            .find(|item| {
                item.claim.attempt_id == attempt_id
                    && item.claim.claim_sha256 == claim_sha256
                    && item.result.result_sha256 == result_sha256
                    && item.result.output_sha256.as_deref() == Some(output_sha256)
            }),
    )
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry, String> {
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
            HistoricalOutcomeLabelMaterializationExecutionAttemptItem {
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
                    == HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::CompletedWithUntrustedEnvelope
            })
        })
        .count();
    let failed_attempt_count = attempts
        .iter()
        .filter(|item| {
            item.result.as_ref().is_some_and(|result| {
                result.status
                    == HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::FailedAuthorizationConsumed
            })
        })
        .count();
    let untrusted_envelope_count = attempts
        .iter()
        .filter(|item| {
            item.result
                .as_ref()
                .is_some_and(|result| result.untrusted_envelope.is_some())
        })
        .count();
    let independent_validation_eligible_count = untrusted_envelope_count;
    let execution_status = if attempts.iter().any(|item| item.result.is_none()) {
        "materialization_claimed_incomplete_fail_closed"
    } else if untrusted_envelope_count > 0 {
        "untrusted_envelope_waiting_independent_validation"
    } else if failed_attempt_count > 0 {
        "materialization_failed_authorization_consumed"
    } else if invocation_eligible_authorization_count > 0 {
        "ready_for_one_shot_fixed_materialization"
    } else {
        "waiting_for_unexpired_first_execution_authorization"
    };
    Ok(HistoricalOutcomeLabelMaterializationExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        invocation_endpoint_available: true,
        invocation_eligible_authorization_count,
        attempt_count,
        completed_attempt_count,
        failed_attempt_count,
        untrusted_envelope_count,
        independent_validation_eligible_count,
        execution_status: execution_status.to_string(),
        attempts,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "本阶段最多消费一条当前未过期授权，先不可变 claim，再用无环境、网络、工具、模型或生产能力的固定纯函数逐位封装已独立重算指标、来源和已知局限。输出只是未验证原始信封，不是结果标签，也不能进入训练、奖励、影子、订单或交易。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    admin_id: &str,
    isolated_runner_id: &str,
    request: InvokeHistoricalOutcomeLabelMaterializationOnceRequest,
) -> Result<HistoricalOutcomeLabelMaterializationExecutionAttemptResult, String> {
    if !valid_id(isolated_runner_id) {
        return Err("标签物化隔离 runner ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-outcome-label-materialization-execution-{isolated_runner_id}"),
    )
    .await?;
    let now = Utc::now();
    let authorization = current_unexpired_first_execution_authorizations(state, now)
        .await?
        .into_iter()
        .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "当前没有这条标签物化 runner 的未过期一次性授权".to_string())?;
    verify_request(&authorization, &request)?;
    let (existing_claims, _) = read_attempt_records(state).await?;
    if claim_replays_authorization_or_runner(&existing_claims, &authorization) {
        return Err("这条标签物化 runner 或首次执行授权已经消费，不得重放".to_string());
    }
    let runtime_artifact_sha256 = reverify_current_binary_sha256().await?;
    if runtime_artifact_sha256 != authorization.runner.runner_artifact_sha256 {
        return Err("当前运行制品与独立复核的标签物化 runner SHA-256 不一致".to_string());
    }
    let admission = exact_admitted_historical_outcome_for_label_materialization(
        state,
        &authorization.runner.attempt_id,
        &authorization.runner.admission_review_id,
        &authorization.runner.admission_review_sha256,
        &authorization.runner.validation_sha256,
        &authorization.runner.output_sha256,
        &authorization.runner.snapshot_sha256,
        &authorization.runner.protocol_sha256,
    )
    .await?
    .ok_or_else(|| "标签物化 runner 的当前准入、校验或封存输入链已经失效".to_string())?;
    verify_admission_binding(&authorization, &admission)?;

    let claim = new_claim(&authorization, admin_id, Utc::now())?;
    write_immutable_json(
        &claim_directory(state, &claim.isolated_runner_id)
            .join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let execution = execute_fixed_projection(&authorization, &admission).await;
    let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let result = match execution {
        Ok((envelope, output_sha256, stdout_bytes)) => new_success_result(
            &claim,
            Utc::now(),
            duration_millis,
            envelope,
            output_sha256,
            stdout_bytes,
        )?,
        Err(error) => new_failed_result(&claim, Utc::now(), duration_millis, &error)?,
    };
    write_immutable_json(
        &result_directory(state, &claim.isolated_runner_id)
            .join(format!("{}.json", result.result_id)),
        &result,
    )
    .await?;
    Ok(result)
}

fn claim_replays_authorization_or_runner(
    claims: &[HistoricalOutcomeLabelMaterializationExecutionAttemptClaim],
    authorization: &ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
) -> bool {
    claim_replays_identity(
        claims,
        &authorization.runner.isolated_runner_id,
        &authorization.review.review_id,
    )
}

fn claim_replays_identity(
    claims: &[HistoricalOutcomeLabelMaterializationExecutionAttemptClaim],
    isolated_runner_id: &str,
    authorization_review_id: &str,
) -> bool {
    claims.iter().any(|claim| {
        claim.isolated_runner_id == isolated_runner_id
            || claim.authorization_review_id == authorization_review_id
    })
}

fn verify_request(
    authorization: &ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
    request: &InvokeHistoricalOutcomeLabelMaterializationOnceRequest,
) -> Result<(), String> {
    let runner = &authorization.runner;
    if request.expected_first_execution_authorization_review_id != authorization.review.review_id
        || request.expected_first_execution_authorization_review_sha256
            != authorization.review.review_sha256
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != runner.runner_artifact_sha256
        || request.expected_implementation_spec_sha256
            != runner.materialization_implementation_spec_sha256
        || request.expected_admission_review_sha256 != runner.admission_review_sha256
        || request.expected_validation_sha256 != runner.validation_sha256
        || request.expected_source_output_sha256 != runner.output_sha256
        || request.expected_snapshot_sha256 != runner.snapshot_sha256
        || request.expected_protocol_sha256 != runner.protocol_sha256
        || request.expected_recomputed_metrics_sha256 != runner.recomputed_metrics_sha256
    {
        return Err("标签物化授权、runner 制品或上游证据已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn verify_admission_binding(
    authorization: &ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
    admission: &AdmittedHistoricalOutcomeForLabelMaterialization,
) -> Result<(), String> {
    let runner = &authorization.runner;
    let review = &admission.admission_review;
    let validation = &admission.validation;
    if review.review_id != runner.admission_review_id
        || review.review_sha256 != runner.admission_review_sha256
        || review.reviewer_id != runner.admission_reviewer_id
        || review.known_limitations != runner.admission_known_limitations
        || review.attempt_id != runner.attempt_id
        || review.claim_sha256 != runner.claim_sha256
        || review.result_id != runner.result_id
        || review.result_sha256 != runner.result_sha256
        || review.output_sha256 != runner.output_sha256
        || validation.validation_id != runner.validation_id
        || validation.validation_sha256 != runner.validation_sha256
        || validation.output_sha256 != runner.output_sha256
        || validation.snapshot_id != runner.snapshot_id
        || validation.snapshot_sha256 != runner.snapshot_sha256
        || validation.protocol_version != runner.protocol_version
        || validation.protocol_sha256 != runner.protocol_sha256
        || admission.snapshot.reconstruction_id != runner.reconstruction_id
        || admission.snapshot.reconstruction_sha256 != runner.reconstruction_sha256
        || admission.snapshot.asset_symbol != runner.asset_symbol
        || admission.snapshot.benchmark_symbol != runner.benchmark_symbol
        || admission.snapshot.decision_available_at != runner.decision_available_at
        || admission.snapshot.common_session_count != runner.common_session_count
        || hash_serializable(&validation.recomputed_metrics)? != runner.recomputed_metrics_sha256
        || validation
            .recomputed_metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect::<Vec<_>>()
            != REQUIRED_HORIZONS
    {
        return Err("标签物化 runner 与重新读取的准入/校验链绑定不一致".to_string());
    }
    Ok(())
}

fn new_claim(
    authorization: &ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
    admin_id: &str,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeLabelMaterializationExecutionAttemptClaim, String> {
    let runner = &authorization.runner;
    let review = &authorization.review;
    let mut claim = HistoricalOutcomeLabelMaterializationExecutionAttemptClaim {
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
        materialization_implementation_id: runner.materialization_implementation_id.clone(),
        materialization_implementation_spec_sha256: runner
            .materialization_implementation_spec_sha256
            .clone(),
        admission_review_id: runner.admission_review_id.clone(),
        admission_review_sha256: runner.admission_review_sha256.clone(),
        validation_id: runner.validation_id.clone(),
        validation_sha256: runner.validation_sha256.clone(),
        source_attempt_id: runner.attempt_id.clone(),
        source_output_sha256: runner.output_sha256.clone(),
        snapshot_id: runner.snapshot_id.clone(),
        snapshot_sha256: runner.snapshot_sha256.clone(),
        reconstruction_id: runner.reconstruction_id.clone(),
        reconstruction_sha256: runner.reconstruction_sha256.clone(),
        protocol_version: runner.protocol_version.clone(),
        protocol_sha256: runner.protocol_sha256.clone(),
        recomputed_metrics_sha256: runner.recomputed_metrics_sha256.clone(),
        max_wall_clock_seconds: runner.max_wall_clock_seconds,
        max_memory_mib: runner.max_memory_mib,
        max_cpu_millicores: runner.max_cpu_millicores,
        max_process_count: runner.max_process_count,
        max_output_bytes: runner.max_output_bytes,
        claimed_at,
        invoked_by: admin_id.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        artifact_digest_reverified: true,
        current_admission_chain_revalidated: true,
        authorization_consumed: true,
        invocation_started: true,
        child_process_spawned: false,
        ambient_filesystem_capability_available: false,
        ambient_environment_capability_available: false,
        network_capability_available: false,
        external_tool_capability_available: false,
        production_data_capability_available_to_projection: false,
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

async fn execute_fixed_projection(
    authorization: &ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
    admission: &AdmittedHistoricalOutcomeForLabelMaterialization,
) -> Result<
    (
        HistoricalOutcomeLabelMaterializationUntrustedEnvelope,
        String,
        usize,
    ),
    MaterializationFailure,
> {
    let envelope = materialize_untrusted_envelope(authorization, admission)
        .map_err(MaterializationFailure::before_staging)?;
    let bytes = serde_json::to_vec(&envelope)
        .map_err(|error| MaterializationFailure::before_staging(error.to_string()))?;
    if bytes.len() > authorization.runner.max_output_bytes as usize {
        return Err(MaterializationFailure::before_staging(
            "原始结果信封超过 runner 登记输出上限",
        ));
    }
    let directory = std::env::temp_dir().join(format!(
        "hone-historical-outcome-label-materialization-{}",
        Uuid::new_v4()
    ));
    tokio::fs::create_dir(&directory)
        .await
        .map_err(|error| MaterializationFailure {
            message: format!("一次性物化目录创建失败：{error}"),
            ephemeral_directory_removed: false,
        })?;
    let path = directory.join("untrusted-raw-outcome-envelope.json");
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
            return Err("一次性物化输出回读不一致".to_string());
        }
        Ok::<_, String>(hash_bytes(&read_back))
    }
    .await;
    let cleanup = tokio::fs::remove_dir_all(&directory).await;
    let removed = cleanup.is_ok()
        || tokio::fs::metadata(&directory)
            .await
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
    match (staged, cleanup, removed) {
        (Ok(output_sha256), Ok(()), true) => Ok((envelope, output_sha256, bytes.len())),
        (Ok(_), Err(error), removed) => Err(MaterializationFailure {
            message: format!("一次性物化目录清理失败：{error}"),
            ephemeral_directory_removed: removed,
        }),
        (Err(error), _, removed) => Err(MaterializationFailure {
            message: error,
            ephemeral_directory_removed: removed,
        }),
        (Ok(_), Ok(()), false) => Err(MaterializationFailure {
            message: "一次性物化目录清理状态无法确认".to_string(),
            ephemeral_directory_removed: false,
        }),
    }
}

fn materialize_untrusted_envelope(
    authorization: &ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
    admission: &AdmittedHistoricalOutcomeForLabelMaterialization,
) -> Result<HistoricalOutcomeLabelMaterializationUntrustedEnvelope, String> {
    verify_admission_binding(authorization, admission)?;
    let runner = &authorization.runner;
    let review = &admission.admission_review;
    let metrics = admission.validation.recomputed_metrics.clone();
    if metrics.len() != REQUIRED_HORIZONS.len()
        || metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect::<Vec<_>>()
            != REQUIRED_HORIZONS
        || metrics.iter().any(|metric| {
            !metric.asset_return.is_finite()
                || !metric.benchmark_return.is_finite()
                || !metric.excess_return.is_finite()
                || !metric.asset_max_drawdown.is_finite()
                || metric.asset_max_drawdown > 0.0
        })
    {
        return Err("已验证指标结构不符合冻结物化合同".to_string());
    }
    Ok(HistoricalOutcomeLabelMaterializationUntrustedEnvelope {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_string(),
        output_label_schema_version: runner.output_label_schema_version.clone(),
        materialization_implementation_id: runner.materialization_implementation_id.clone(),
        materialization_implementation_spec_sha256: runner
            .materialization_implementation_spec_sha256
            .clone(),
        admission_review_id: runner.admission_review_id.clone(),
        admission_review_sha256: runner.admission_review_sha256.clone(),
        validation_id: runner.validation_id.clone(),
        validation_sha256: runner.validation_sha256.clone(),
        source_attempt_id: runner.attempt_id.clone(),
        source_claim_sha256: runner.claim_sha256.clone(),
        source_result_id: runner.result_id.clone(),
        source_result_sha256: runner.result_sha256.clone(),
        source_output_sha256: runner.output_sha256.clone(),
        snapshot_id: runner.snapshot_id.clone(),
        snapshot_sha256: runner.snapshot_sha256.clone(),
        reconstruction_id: runner.reconstruction_id.clone(),
        reconstruction_sha256: runner.reconstruction_sha256.clone(),
        protocol_version: runner.protocol_version.clone(),
        protocol_sha256: runner.protocol_sha256.clone(),
        asset_symbol: runner.asset_symbol.clone(),
        benchmark_symbol: runner.benchmark_symbol.clone(),
        decision_available_at: runner.decision_available_at,
        common_session_count: runner.common_session_count,
        metric_start_date: runner.metric_start_date,
        metric_end_dates: runner.metric_end_dates.clone(),
        recomputed_metrics_sha256: runner.recomputed_metrics_sha256.clone(),
        raw_validated_metrics: metrics,
        known_limitations: review.known_limitations.clone(),
        deterministic_projection_only: true,
        exact_metric_bits_preserved: true,
        provenance_preserved: true,
        known_limitations_preserved: true,
        output_is_untrusted: true,
        independent_validation_completed: false,
        outcome_label_write_allowed: false,
        outcome_label_written: false,
        direction_inferred: false,
        rating_inferred: false,
        investment_action_inferred: false,
        position_size_inferred: false,
        training_target_written: false,
        reward_written: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    })
}

fn new_success_result(
    claim: &HistoricalOutcomeLabelMaterializationExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    envelope: HistoricalOutcomeLabelMaterializationUntrustedEnvelope,
    output_sha256: String,
    stdout_bytes: usize,
) -> Result<HistoricalOutcomeLabelMaterializationExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeLabelMaterializationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::CompletedWithUntrustedEnvelope,
        exit_code: 0,
        stdout_sha256: output_sha256.clone(),
        stderr_sha256: hash_bytes(&[]),
        stdout_bytes,
        stderr_bytes: 0,
        output_sha256: Some(output_sha256),
        untrusted_envelope: Some(envelope),
        ephemeral_directory_removed: true,
        independent_validation_completed: false,
        outcome_label_admission_authorized: false,
        outcome_label_write_allowed: false,
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
    claim: &HistoricalOutcomeLabelMaterializationExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    error: &MaterializationFailure,
) -> Result<HistoricalOutcomeLabelMaterializationExecutionAttemptResult, String> {
    let bounded = error
        .message
        .chars()
        .take(MAX_ERROR_CHARS)
        .collect::<String>();
    let stderr = bounded.as_bytes();
    let mut result = HistoricalOutcomeLabelMaterializationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status:
            HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::FailedAuthorizationConsumed,
        exit_code: 1,
        stdout_sha256: hash_bytes(&[]),
        stderr_sha256: hash_bytes(stderr),
        stdout_bytes: 0,
        stderr_bytes: stderr.len(),
        output_sha256: None,
        untrusted_envelope: None,
        ephemeral_directory_removed: error.ephemeral_directory_removed,
        independent_validation_completed: false,
        outcome_label_admission_authorized: false,
        outcome_label_write_allowed: false,
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

fn validate_claim(
    claim: &HistoricalOutcomeLabelMaterializationExecutionAttemptClaim,
) -> Result<(), String> {
    let capabilities_closed = !claim.child_process_spawned
        && !claim.ambient_filesystem_capability_available
        && !claim.ambient_environment_capability_available
        && !claim.network_capability_available
        && !claim.external_tool_capability_available
        && !claim.production_data_capability_available_to_projection
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
        || !valid_id(&claim.materialization_implementation_id)
        || !valid_sha256(&claim.materialization_implementation_spec_sha256)
        || !valid_id(&claim.admission_review_id)
        || !valid_sha256(&claim.admission_review_sha256)
        || !valid_id(&claim.validation_id)
        || !valid_sha256(&claim.validation_sha256)
        || !valid_id(&claim.source_attempt_id)
        || !valid_sha256(&claim.source_output_sha256)
        || !valid_id(&claim.snapshot_id)
        || !valid_sha256(&claim.snapshot_sha256)
        || !valid_id(&claim.reconstruction_id)
        || !valid_sha256(&claim.reconstruction_sha256)
        || claim.protocol_version.trim().is_empty()
        || !valid_sha256(&claim.protocol_sha256)
        || !valid_sha256(&claim.recomputed_metrics_sha256)
        || claim.max_wall_clock_seconds != 300
        || claim.max_memory_mib != 512
        || claim.max_cpu_millicores != 1_000
        || claim.max_process_count != 1
        || claim.max_output_bytes != 1_048_576
        || claim.invoked_by.trim().is_empty()
        || claim.isolation_backend != ISOLATION_BACKEND
        || !claim.artifact_digest_reverified
        || !claim.current_admission_chain_revalidated
        || !claim.authorization_consumed
        || !claim.invocation_started
        || !capabilities_closed
    {
        return Err(
            "historical outcome label materialization claim is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeLabelMaterializationExecutionAttemptResult,
    claim: &HistoricalOutcomeLabelMaterializationExecutionAttemptClaim,
) -> Result<(), String> {
    let downstream_closed = !result.independent_validation_completed
        && !result.outcome_label_admission_authorized
        && !result.outcome_label_write_allowed
        && !result.outcome_label_written
        && !result.training_authorized
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    let output_consistent = match (
        &result.status,
        &result.untrusted_envelope,
        &result.output_sha256,
    ) {
        (
            HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::CompletedWithUntrustedEnvelope,
            Some(envelope),
            Some(output_sha256),
        ) => {
            result.exit_code == 0
                && result.stdout_bytes <= claim.max_output_bytes as usize
                && result.stderr_bytes == 0
                && result.stderr_sha256 == hash_bytes(&[])
                && envelope.output_is_untrusted
                && !envelope.independent_validation_completed
                && !envelope.outcome_label_write_allowed
                && !envelope.outcome_label_written
                && !envelope.direction_inferred
                && !envelope.rating_inferred
                && !envelope.investment_action_inferred
                && !envelope.position_size_inferred
                && !envelope.training_target_written
                && !envelope.reward_written
                && !envelope.shadow_position_written
                && !envelope.order_generated
                && !envelope.broker_accessed
                && !envelope.trade_executed
                && hash_serializable(envelope).is_ok_and(|hash| hash == *output_sha256)
                && result.stdout_sha256 == *output_sha256
        }
        (
            HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::FailedAuthorizationConsumed,
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
            == HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::CompletedWithUntrustedEnvelope
            && !result.ephemeral_directory_removed)
        || !output_consistent
        || !downstream_closed
    {
        return Err("historical outcome label materialization result is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn claim_binding_matches(
    claim: &HistoricalOutcomeLabelMaterializationExecutionAttemptClaim,
    authorization: &ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
) -> bool {
    let runner = &authorization.runner;
    claim.authorization_review_id == authorization.review.review_id
        && claim.authorization_review_sha256 == authorization.review.review_sha256
        && claim.authorization_valid_until == authorization.review.authorization_valid_until
        && claim.isolated_runner_id == runner.isolated_runner_id
        && claim.isolated_runner_spec_sha256 == runner.isolated_runner_spec_sha256
        && claim.runner_artifact_sha256 == runner.runner_artifact_sha256
        && claim.runner_code_revision == runner.runner_code_revision
        && claim.materialization_implementation_id == runner.materialization_implementation_id
        && claim.materialization_implementation_spec_sha256
            == runner.materialization_implementation_spec_sha256
        && claim.admission_review_id == runner.admission_review_id
        && claim.admission_review_sha256 == runner.admission_review_sha256
        && claim.validation_id == runner.validation_id
        && claim.validation_sha256 == runner.validation_sha256
        && claim.source_attempt_id == runner.attempt_id
        && claim.source_output_sha256 == runner.output_sha256
        && claim.snapshot_id == runner.snapshot_id
        && claim.snapshot_sha256 == runner.snapshot_sha256
        && claim.reconstruction_id == runner.reconstruction_id
        && claim.reconstruction_sha256 == runner.reconstruction_sha256
        && claim.protocol_version == runner.protocol_version
        && claim.protocol_sha256 == runner.protocol_sha256
        && claim.recomputed_metrics_sha256 == runner.recomputed_metrics_sha256
}

fn claim_fingerprint(
    claim: &HistoricalOutcomeLabelMaterializationExecutionAttemptClaim,
) -> Result<String, String> {
    let mut value = serde_json::to_value(claim).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "materialization claim fingerprint payload is invalid".to_string())?;
    object.remove("attempt_id");
    object.remove("claim_sha256");
    hash_serializable(&value)
}

fn result_fingerprint(
    result: &HistoricalOutcomeLabelMaterializationExecutionAttemptResult,
) -> Result<String, String> {
    let mut value = serde_json::to_value(result).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "materialization result fingerprint payload is invalid".to_string())?;
    object.remove("result_id");
    object.remove("result_sha256");
    hash_serializable(&value)
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<HistoricalOutcomeLabelMaterializationExecutionAttemptClaim>,
        Vec<HistoricalOutcomeLabelMaterializationExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<HistoricalOutcomeLabelMaterializationExecutionAttemptClaim>(
        &execution_root(state).join("claims"),
    )
    .await?;
    let results = read_nested_json::<HistoricalOutcomeLabelMaterializationExecutionAttemptResult>(
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
                "historical outcome label materialization registry contains replayed claims"
                    .to_string(),
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
        let claim = claim_index
            .get(result.attempt_id.as_str())
            .ok_or_else(|| "materialization result has no immutable claim".to_string())?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_hashes.insert(result.result_sha256.clone())
            || !completed_attempts.insert(result.attempt_id.clone())
        {
            return Err(
                "historical outcome label materialization registry contains duplicate results"
                    .to_string(),
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
    decision_root(state).join("historical-outcome-label-materialization-execution-attempts")
}

fn claim_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("claims").join(runner_id)
}

fn result_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("results").join(runner_id)
}

struct HistoricalOutcomeLabelMaterializationExecutionLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeLabelMaterializationExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<HistoricalOutcomeLabelMaterializationExecutionLock, String> {
    if !valid_id(name) {
        return Err("invalid label materialization execution lock name".to_string());
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
            Ok(_) => return Ok(HistoricalOutcomeLabelMaterializationExecutionLock { path }),
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
                return Err("标签物化首次执行正在由另一请求消费".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("标签物化首次执行锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid label materialization execution path".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path).await.map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            "historical outcome label materialization execution record already exists".to_string()
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
    use super::*;

    fn metric(horizon: u16, end_month: u32, asset_return: f64) -> HistoricalOutcomeDryRunMetric {
        HistoricalOutcomeDryRunMetric {
            horizon_market_sessions: horizon,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, end_month, 3).unwrap(),
            asset_return,
            benchmark_return: asset_return / 2.0,
            excess_return: asset_return / 2.0,
            asset_max_drawdown: -0.1,
        }
    }

    fn envelope() -> HistoricalOutcomeLabelMaterializationUntrustedEnvelope {
        let metrics = vec![metric(20, 2, 0.1), metric(60, 4, 0.2), metric(250, 12, 0.3)];
        HistoricalOutcomeLabelMaterializationUntrustedEnvelope {
            schema_version: ENVELOPE_SCHEMA_VERSION.to_string(),
            output_label_schema_version: "raw-label-v1".to_string(),
            materialization_implementation_id: "implementation-1".to_string(),
            materialization_implementation_spec_sha256: "1".repeat(64),
            admission_review_id: "admission-1".to_string(),
            admission_review_sha256: "2".repeat(64),
            validation_id: "validation-1".to_string(),
            validation_sha256: "3".repeat(64),
            source_attempt_id: "source-attempt-1".to_string(),
            source_claim_sha256: "4".repeat(64),
            source_result_id: "source-result-1".to_string(),
            source_result_sha256: "5".repeat(64),
            source_output_sha256: "6".repeat(64),
            snapshot_id: "snapshot-1".to_string(),
            snapshot_sha256: "7".repeat(64),
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "8".repeat(64),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "9".repeat(64),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            decision_available_at: DateTime::parse_from_rfc3339("2025-01-02T21:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            common_session_count: 251,
            metric_start_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            metric_end_dates: metrics.iter().map(|metric| metric.end_date).collect(),
            recomputed_metrics_sha256: hash_serializable(&metrics).unwrap(),
            raw_validated_metrics: metrics,
            known_limitations: "样本有限，不推断动作。".to_string(),
            deterministic_projection_only: true,
            exact_metric_bits_preserved: true,
            provenance_preserved: true,
            known_limitations_preserved: true,
            output_is_untrusted: true,
            independent_validation_completed: false,
            outcome_label_write_allowed: false,
            outcome_label_written: false,
            direction_inferred: false,
            rating_inferred: false,
            investment_action_inferred: false,
            position_size_inferred: false,
            training_target_written: false,
            reward_written: false,
            shadow_position_written: false,
            order_generated: false,
            broker_accessed: false,
            trade_executed: false,
        }
    }

    fn claim() -> HistoricalOutcomeLabelMaterializationExecutionAttemptClaim {
        let claimed_at = DateTime::parse_from_rfc3339("2026-08-22T16:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let mut claim = HistoricalOutcomeLabelMaterializationExecutionAttemptClaim {
            schema_version: CLAIM_SCHEMA_VERSION.to_string(),
            execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
            attempt_id: String::new(),
            claim_sha256: String::new(),
            authorization_review_id: "authorization-1".to_string(),
            authorization_review_sha256: "a".repeat(64),
            authorization_valid_until: DateTime::parse_from_rfc3339("2026-08-23T15:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            isolated_runner_id: "runner-1".to_string(),
            isolated_runner_spec_sha256: "b".repeat(64),
            runner_artifact_sha256: "c".repeat(64),
            runner_code_revision: "oldwang@materializer".to_string(),
            materialization_implementation_id: "implementation-1".to_string(),
            materialization_implementation_spec_sha256: "d".repeat(64),
            admission_review_id: "admission-1".to_string(),
            admission_review_sha256: "e".repeat(64),
            validation_id: "validation-1".to_string(),
            validation_sha256: "f".repeat(64),
            source_attempt_id: "source-attempt-1".to_string(),
            source_output_sha256: "1".repeat(64),
            snapshot_id: "snapshot-1".to_string(),
            snapshot_sha256: "2".repeat(64),
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "3".repeat(64),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "4".repeat(64),
            recomputed_metrics_sha256: "5".repeat(64),
            max_wall_clock_seconds: 300,
            max_memory_mib: 512,
            max_cpu_millicores: 1_000,
            max_process_count: 1,
            max_output_bytes: 1_048_576,
            claimed_at,
            invoked_by: "executor-admin".to_string(),
            isolation_backend: ISOLATION_BACKEND.to_string(),
            artifact_digest_reverified: true,
            current_admission_chain_revalidated: true,
            authorization_consumed: true,
            invocation_started: true,
            child_process_spawned: false,
            ambient_filesystem_capability_available: false,
            ambient_environment_capability_available: false,
            network_capability_available: false,
            external_tool_capability_available: false,
            production_data_capability_available_to_projection: false,
            historical_state_mutation_allowed: false,
            outcome_label_writes_allowed: false,
            training_writes_allowed: false,
            reward_writes_allowed: false,
            shadow_writes_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        claim
    }

    fn success_result() -> HistoricalOutcomeLabelMaterializationExecutionAttemptResult {
        let claim = claim();
        let envelope = envelope();
        let output_sha256 = hash_serializable(&envelope).unwrap();
        let output_bytes = serde_json::to_vec(&envelope).unwrap().len();
        new_success_result(
            &claim,
            claim.claimed_at,
            1,
            envelope,
            output_sha256.clone(),
            output_bytes,
        )
        .unwrap()
    }

    #[test]
    fn claim_is_consumed_before_fixed_projection_and_has_no_ambient_capabilities() {
        let value = claim();
        assert!(validate_claim(&value).is_ok());
        assert!(value.authorization_consumed);
        assert!(value.invocation_started);
        assert!(!value.child_process_spawned);
        assert!(!value.network_capability_available);
        assert!(!value.outcome_label_writes_allowed);
        assert!(!value.trading_allowed);
    }

    #[test]
    fn successful_output_is_an_untrusted_envelope_not_a_label() {
        let value = success_result();
        assert!(validate_result(&value, &claim()).is_ok());
        let output = value.untrusted_envelope.unwrap();
        assert!(output.output_is_untrusted);
        assert!(!output.independent_validation_completed);
        assert!(!output.outcome_label_written);
        assert!(!output.training_target_written);
        assert!(!output.investment_action_inferred);
    }

    #[test]
    fn exact_metrics_and_limitations_are_preserved() {
        let output = envelope();
        assert_eq!(output.raw_validated_metrics.len(), 3);
        assert_eq!(
            output
                .raw_validated_metrics
                .iter()
                .map(|metric| metric.horizon_market_sessions)
                .collect::<Vec<_>>(),
            REQUIRED_HORIZONS
        );
        assert_eq!(
            hash_serializable(&output.raw_validated_metrics).unwrap(),
            output.recomputed_metrics_sha256
        );
        assert_eq!(output.known_limitations, "样本有限，不推断动作。");
    }

    #[test]
    fn output_tampering_is_rejected() {
        let claim = claim();
        let mut result = success_result();
        result
            .untrusted_envelope
            .as_mut()
            .unwrap()
            .raw_validated_metrics[0]
            .asset_return += f64::EPSILON;
        assert!(validate_result(&result, &claim).is_err());
    }

    #[test]
    fn semantic_or_downstream_authority_is_rejected() {
        let claim = claim();
        let mut result = success_result();
        result
            .untrusted_envelope
            .as_mut()
            .unwrap()
            .investment_action_inferred = true;
        result.outcome_label_write_allowed = true;
        result.training_authorized = true;
        assert!(validate_result(&result, &claim).is_err());
    }

    #[test]
    fn failed_attempt_consumes_authorization_without_output() {
        let claim = claim();
        let result = new_failed_result(
            &claim,
            claim.claimed_at,
            2,
            &MaterializationFailure::before_staging("deterministic failure"),
        )
        .unwrap();
        assert_eq!(
            result.status,
            HistoricalOutcomeLabelMaterializationExecutionAttemptStatus::FailedAuthorizationConsumed
        );
        assert!(result.untrusted_envelope.is_none());
        assert!(validate_result(&result, &claim).is_ok());
    }

    #[test]
    fn claim_fingerprint_detects_tampering() {
        let mut value = claim();
        value.source_output_sha256 = "9".repeat(64);
        assert!(validate_claim(&value).is_err());
    }

    #[test]
    fn result_fingerprint_detects_tampering() {
        let claim = claim();
        let mut value = success_result();
        value.stdout_bytes += 1;
        assert!(validate_result(&value, &claim).is_err());
    }

    #[test]
    fn duplicate_runner_or_authorization_claim_is_a_replay() {
        assert!(claim_replays_identity(
            &[claim()],
            "runner-2",
            "authorization-1"
        ));
        assert!(claim_replays_identity(
            &[claim()],
            "runner-1",
            "authorization-2"
        ));
        assert!(!claim_replays_identity(
            &[claim()],
            "runner-2",
            "authorization-2"
        ));
    }
}
