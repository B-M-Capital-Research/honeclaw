//! Stage 88 claim-first, single-use natural-forward-observation initialization attempts.
//!
//! One exact unexpired Stage 87 authorization is irreversibly claimed before the submitted
//! initialization manifest is opened or the current executable digest is checked. A successful
//! attempt produces only an untrusted initialization receipt for independent Stage 89 review.
//! It does not attach market data, instantiate a persistent runtime, start an observation, create
//! a ledger or position, calculate performance, or grant order, broker or trading capability.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_execution_authorizations::{
    ApprovedControlledShadowForwardObservationFirstExecutionAuthorization,
    exact_forward_observation_first_execution_authorization_for_audit,
    forward_observation_authorizations_for_future_claim_first_attempt,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-execution-attempt-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-execution-attempt-result-v1";
const MANIFEST_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-initialization-manifest-v1-no-market-data";
const RECEIPT_SCHEMA_VERSION: &str = "hone-controlled-shadow-forward-observation-untrusted-initialization-receipt-v2-reconstructible-manifest";
const EXECUTION_POLICY_VERSION: &str =
    "hone-controlled-shadow-forward-observation-claim-first-one-shot-initialization-v1";
const ISOLATION_BACKEND: &str =
    "hone-in-process-capability-limited-forward-observation-initialization-v1";
const MAX_CLOCK_SKEW_MINUTES: i64 = 5;
const MAX_ERROR_CHARS: usize = 1_200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowForwardObservationInitializationManifest {
    pub schema_version: String,
    pub manifest_sha256: String,
    pub requested_at: DateTime<Utc>,
    pub observation_not_before: DateTime<Utc>,
    pub signal_cadence: String,
    pub first_eligible_signal_rule: String,
    pub official_market_calendar: String,
    pub official_market_calendar_source_url: String,
    pub official_market_calendar_content_sha256: String,
    pub benchmark_symbol: String,
    pub initial_observation_validation_sha256: String,
    pub natural_forward_only: bool,
    pub retroactive_backfill_allowed: bool,
    pub market_data_rows_attached: bool,
    pub point_in_time_content_addressed_allowlisted_sources_required: bool,
    pub synchronized_security_and_benchmark_observation_required: bool,
    pub initialization_only: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeControlledShadowForwardObservationOnceRequest {
    expected_authorization_review_id: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_contract_sha256: String,
    expected_runner_code_revision: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_review_sha256: String,
    expected_protocol_review_sha256: String,
    expected_protocol_registration_sha256: String,
    expected_protocol_specification_sha256: String,
    expected_design_specification_sha256: String,
    expected_initial_observation_validation_sha256: String,
    expected_initialization_manifest_sha256: String,
    initialization_manifest: ControlledShadowForwardObservationInitializationManifest,
    claim_first_single_use_and_failure_consumes_confirmed: bool,
    exact_current_stage_51_through_stage_87_binding_confirmed: bool,
    executor_independent_from_stage_87_and_complete_prior_chain_confirmed: bool,
    current_binary_digest_reverification_after_claim_confirmed: bool,
    natural_forward_observation_not_before_and_no_backfill_confirmed: bool,
    official_calendar_and_spy_synchronization_confirmed: bool,
    initialization_manifest_contains_no_market_data_confirmed: bool,
    initialization_receipt_is_untrusted_and_requires_independent_validation_confirmed: bool,
    no_runtime_mount_data_access_observation_ledger_position_or_performance_confirmed: bool,
    no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowForwardObservationExecutionAttemptClaim {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub authorization_valid_until: DateTime<Utc>,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_contract_sha256: String,
    pub runner_code_revision: String,
    pub runner_artifact_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub implementation_review_sha256: String,
    pub protocol_review_sha256: String,
    pub protocol_registration_sha256: String,
    pub protocol_specification_sha256: String,
    pub design_specification_sha256: String,
    pub initial_observation_validation_sha256: String,
    pub initialization_manifest_sha256: String,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub executor_independent_from_stage_87_and_complete_prior_chain: bool,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub initialization_manifest_opened: bool,
    pub persistent_runtime_instantiation_allowed: bool,
    pub input_mount_allowed: bool,
    pub market_data_access_allowed: bool,
    pub forward_observation_write_allowed: bool,
    pub ledger_write_allowed: bool,
    pub position_write_allowed: bool,
    pub performance_metric_write_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub training_feedback_allowed: bool,
    pub scalar_reward_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowForwardObservationUntrustedInitializationReceipt {
    pub schema_version: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub implementation_contract_sha256: String,
    pub protocol_specification_sha256: String,
    pub design_specification_sha256: String,
    pub initialization_manifest_sha256: String,
    pub observation_not_before: DateTime<Utc>,
    pub requested_at: DateTime<Utc>,
    pub signal_cadence: String,
    pub first_eligible_signal_rule: String,
    pub official_market_calendar: String,
    pub official_market_calendar_source_url: String,
    pub official_market_calendar_content_sha256: String,
    pub benchmark_symbol: String,
    pub initial_observation_validation_sha256: String,
    pub natural_forward_only: bool,
    pub retroactive_backfill_allowed: bool,
    pub point_in_time_content_addressed_allowlisted_sources_required: bool,
    pub synchronized_security_and_benchmark_observation_required: bool,
    pub initialization_only: bool,
    pub output_is_untrusted: bool,
    pub independent_output_validation_completed: bool,
    pub market_data_rows_attached: bool,
    pub natural_forward_market_sessions_observed: u32,
    pub persistent_runtime_instantiated: bool,
    pub input_manifest_attached: bool,
    pub market_data_accessed: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_written: bool,
    pub scalar_reward_written: bool,
    pub order_intent_present: bool,
    pub broker_payload_present: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowForwardObservationExecutionAttemptStatus {
    CompletedWithUntrustedInitializationReceipt,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowForwardObservationExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub status: ControlledShadowForwardObservationExecutionAttemptStatus,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub isolation_backend: String,
    pub exit_code: i32,
    pub failure_reason: Option<String>,
    pub current_binary_digest_reverified: bool,
    pub initialization_manifest_validated: bool,
    pub initialization_completed: bool,
    pub untrusted_initialization_receipt:
        Option<ControlledShadowForwardObservationUntrustedInitializationReceipt>,
    pub output_sha256: Option<String>,
    pub independent_output_validation_completed: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_written: bool,
    pub scalar_reward_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationExecutionAttemptItem {
    pub claim: ControlledShadowForwardObservationExecutionAttemptClaim,
    pub result: Option<ControlledShadowForwardObservationExecutionAttemptResult>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CompletedControlledShadowForwardObservationExecutionAttempt {
    pub claim: ControlledShadowForwardObservationExecutionAttemptClaim,
    pub result: ControlledShadowForwardObservationExecutionAttemptResult,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub attempts: Vec<ControlledShadowForwardObservationExecutionAttemptItem>,
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub interrupted_count: usize,
    pub independent_validation_eligible_count: usize,
    pub execution_status: String,
    pub persistent_runtime_instantiated: bool,
    pub market_data_accessed: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_written: bool,
    pub scalar_reward_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowForwardObservationExecutionAttemptReadinessSummary {
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
    pub interrupted_count: usize,
    pub independent_validation_eligible_count: usize,
    pub execution_status: String,
}

#[derive(Debug, Clone)]
struct ForwardObservationAttemptBindings {
    authorization_review_id: String,
    authorization_review_sha256: String,
    authorization_valid_until: DateTime<Utc>,
    isolated_runner_id: String,
    isolated_runner_spec_sha256: String,
    runner_contract_sha256: String,
    runner_code_revision: String,
    runner_artifact_sha256: String,
    implementation_id: String,
    implementation_sha256: String,
    implementation_contract_sha256: String,
    implementation_review_sha256: String,
    protocol_review_sha256: String,
    protocol_registration_sha256: String,
    protocol_specification_sha256: String,
    design_specification_sha256: String,
    initial_observation_validation_sha256: String,
    observation_not_before: DateTime<Utc>,
    signal_cadence: String,
    first_eligible_signal_rule: String,
    official_market_calendar: String,
    benchmark_symbol: String,
    excluded_prior_actor_ids: Vec<String>,
}

pub(crate) async fn handle_get_controlled_shadow_forward_observation_execution_attempts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_registry(&state, Utc::now()).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => {
            warn!(%error, "forward-observation execution attempt registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "前向观察初始化尝试暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_controlled_shadow_forward_observation_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeControlledShadowForwardObservationOnceRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match invoke_once(&state, &admin.user_id, &isolated_runner_id, request).await {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_forward_observation_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationExecutionAttemptReadinessSummary, String> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowForwardObservationExecutionAttemptReadinessSummary {
            invocation_eligible_authorization_count: value.invocation_eligible_authorization_count,
            claim_count: value.claim_count,
            completed_count: value.completed_count,
            failed_count: value.failed_count,
            interrupted_count: value.interrupted_count,
            independent_validation_eligible_count: value.independent_validation_eligible_count,
            execution_status: value.execution_status,
        },
    )
}

pub(crate) async fn claimed_forward_observation_authorization_review_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    let (claims, _) = read_attempt_records(state).await?;
    Ok(claims
        .into_iter()
        .map(|claim| claim.authorization_review_id)
        .collect())
}

#[allow(dead_code)]
pub(crate) async fn completed_forward_observation_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedControlledShadowForwardObservationExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .into_iter()
        .map(|value| (value.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    Ok(claims
        .into_iter()
        .filter_map(|claim| {
            let result = result_index.get(&claim.attempt_id)?.clone();
            (result.status
                == ControlledShadowForwardObservationExecutionAttemptStatus::CompletedWithUntrustedInitializationReceipt)
                .then_some(CompletedControlledShadowForwardObservationExecutionAttempt { claim, result })
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowForwardObservationExecutionAttemptRegistry, String> {
    let authorizations =
        forward_observation_authorizations_for_future_claim_first_attempt(state, now).await?;
    let (claims, results) = read_attempt_records(state).await?;
    let claimed = claims
        .iter()
        .map(|value| value.authorization_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let invocation_eligible_authorization_count = authorizations
        .iter()
        .filter(|value| !claimed.contains(value.review.review_id.as_str()))
        .count();
    let result_index = results
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let attempts = claims
        .iter()
        .cloned()
        .map(
            |claim| ControlledShadowForwardObservationExecutionAttemptItem {
                result: result_index
                    .get(claim.attempt_id.as_str())
                    .map(|value| (*value).clone()),
                claim,
            },
        )
        .collect::<Vec<_>>();
    let completed_count = results
        .iter()
        .filter(|value| value.status == ControlledShadowForwardObservationExecutionAttemptStatus::CompletedWithUntrustedInitializationReceipt)
        .count();
    let failed_count = results.len().saturating_sub(completed_count);
    let interrupted_count = claims.len().saturating_sub(results.len());
    let execution_status = if invocation_eligible_authorization_count > 0 {
        "forward_observation_initialization_authorized_not_claimed"
    } else if interrupted_count > 0 {
        "forward_observation_authorization_claimed_initialization_interrupted_fail_closed"
    } else if completed_count > 0 {
        "untrusted_forward_observation_initialization_receipt_waiting_for_independent_validation"
    } else if failed_count > 0 {
        "forward_observation_initialization_failed_authorization_consumed"
    } else {
        "waiting_for_current_unexpired_stage_87_authorization"
    };
    Ok(ControlledShadowForwardObservationExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        attempts,
        invocation_eligible_authorization_count,
        claim_count: claims.len(),
        completed_count,
        failed_count,
        interrupted_count,
        independent_validation_eligible_count: completed_count,
        execution_status: execution_status.to_string(),
        persistent_runtime_instantiated: false,
        market_data_accessed: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_store_written: false,
        metric_store_written: false,
        training_feedback_written: false,
        scalar_reward_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
        scope: "Stage 88 只先不可逆 claim 一条精确 Stage 87 授权，再核对当前二进制与无行情初始化 manifest；成功仅生成等待 Stage 89 独立验证的不可信初始化收据。当前不实例化持久 runtime、不挂载或读取行情、不开始自然前向观察、不创建账本/持仓/绩效，不写模型/指标或反馈 reward，也不生成订单、不接券商、不交易。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    executor_id: &str,
    isolated_runner_id: &str,
    request: InvokeControlledShadowForwardObservationOnceRequest,
) -> Result<ControlledShadowForwardObservationExecutionAttemptResult, String> {
    validate_request_confirmations(&request)?;
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let now = Utc::now();
    let authorization =
        forward_observation_authorizations_for_future_claim_first_attempt(state, now)
            .await?
            .into_iter()
            .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
            .ok_or_else(|| "没有当前、未过期且未消费的 Stage 87 精确授权".to_string())?;
    validate_expected_binding(&authorization, &request)?;
    let bindings = bindings_from_authorization(&authorization);
    let (claims, _) = read_attempt_records(state).await?;
    if claims.iter().any(|claim| {
        claim.authorization_review_id == authorization.review.review_id
            || claim.isolated_runner_id == isolated_runner_id
    }) {
        return Err(
            "这条 Stage 87 授权或 runner 已被 claim；成功、失败或中断都不得重放".to_string(),
        );
    }
    let claim = new_claim(
        &bindings,
        executor_id,
        &request.expected_initialization_manifest_sha256,
        now,
    )?;
    write_immutable_json(
        &claim_directory(state, isolated_runner_id).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let started_at = Utc::now();
    let execution = execute_after_claim(
        &bindings,
        &claim.initialization_manifest_sha256,
        &request.initialization_manifest,
        started_at,
    )
    .await;
    let duration_millis = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
    let result = match execution {
        Ok(receipt) => successful_result(&claim, receipt, started_at, duration_millis)?,
        Err(error) => failed_result(&claim, error, started_at, duration_millis)?,
    };
    write_immutable_json(
        &result_directory(state, isolated_runner_id).join(format!("{}.json", result.result_id)),
        &result,
    )
    .await?;
    Ok(result)
}

fn validate_request_confirmations(
    request: &InvokeControlledShadowForwardObservationOnceRequest,
) -> Result<(), String> {
    let confirmed = request.claim_first_single_use_and_failure_consumes_confirmed
        && request.exact_current_stage_51_through_stage_87_binding_confirmed
        && request.executor_independent_from_stage_87_and_complete_prior_chain_confirmed
        && request.current_binary_digest_reverification_after_claim_confirmed
        && request.natural_forward_observation_not_before_and_no_backfill_confirmed
        && request.official_calendar_and_spy_synchronization_confirmed
        && request.initialization_manifest_contains_no_market_data_confirmed
        && request
            .initialization_receipt_is_untrusted_and_requires_independent_validation_confirmed
        && request
            .no_runtime_mount_data_access_observation_ledger_position_or_performance_confirmed
        && request.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    if !confirmed {
        return Err(
            "必须确认 claim-first、完整绑定、自然前向、无行情初始化、不可信收据及全部执行禁区"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_expected_binding(
    authorization: &ApprovedControlledShadowForwardObservationFirstExecutionAuthorization,
    request: &InvokeControlledShadowForwardObservationOnceRequest,
) -> Result<(), String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
    let registration = &implementation.upstream_protocol_registration;
    let protocol = &registration.protocol_specification;
    if request.expected_authorization_review_id != authorization.review.review_id
        || request.expected_authorization_review_sha256 != authorization.review.review_sha256
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_contract_sha256 != runner.runner_contract.contract_sha256
        || request.expected_runner_code_revision != runner.runner_code_revision
        || request.expected_runner_artifact_sha256 != runner.runner_artifact_sha256
        || request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_review_sha256
            != runner.implementation_review.review_sha256
        || request.expected_protocol_review_sha256
            != implementation.upstream_protocol_review.review_sha256
        || request.expected_protocol_registration_sha256
            != registration.protocol_registration_sha256
        || request.expected_protocol_specification_sha256 != protocol.specification_sha256
        || request.expected_design_specification_sha256
            != protocol.exact_design_specification.specification_sha256
        || request.expected_initial_observation_validation_sha256 != contract.validation_sha256
        || request.expected_initialization_manifest_sha256
            != request.initialization_manifest.manifest_sha256
    {
        return Err(
            "Stage 87 授权、runner、协议、初始观察或初始化 manifest 绑定已经变化".to_string(),
        );
    }
    Ok(())
}

fn bindings_from_authorization(
    authorization: &ApprovedControlledShadowForwardObservationFirstExecutionAuthorization,
) -> ForwardObservationAttemptBindings {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
    let registration = &implementation.upstream_protocol_registration;
    let protocol = &registration.protocol_specification;
    let mut excluded = authorization.review.excluded_prior_actor_ids.clone();
    excluded.push(authorization.review.reviewer_id.clone());
    excluded.sort();
    excluded.dedup();
    ForwardObservationAttemptBindings {
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        authorization_valid_until: authorization.review.authorization_valid_until,
        isolated_runner_id: runner.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
        runner_contract_sha256: runner.runner_contract.contract_sha256.clone(),
        runner_code_revision: runner.runner_code_revision.clone(),
        runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        implementation_review_sha256: runner.implementation_review.review_sha256.clone(),
        protocol_review_sha256: implementation
            .upstream_protocol_review
            .review_sha256
            .clone(),
        protocol_registration_sha256: registration.protocol_registration_sha256.clone(),
        protocol_specification_sha256: protocol.specification_sha256.clone(),
        design_specification_sha256: protocol
            .exact_design_specification
            .specification_sha256
            .clone(),
        initial_observation_validation_sha256: contract.validation_sha256.clone(),
        observation_not_before: protocol.observation_not_before,
        signal_cadence: protocol.signal_cadence.clone(),
        first_eligible_signal_rule: protocol.first_eligible_signal_rule.clone(),
        official_market_calendar: protocol.official_market_calendar.clone(),
        benchmark_symbol: protocol.benchmark_symbol.clone(),
        excluded_prior_actor_ids: excluded,
    }
}

fn new_claim(
    bindings: &ForwardObservationAttemptBindings,
    executor_id: &str,
    manifest_sha256: &str,
    claimed_at: DateTime<Utc>,
) -> Result<ControlledShadowForwardObservationExecutionAttemptClaim, String> {
    let independent = !executor_id.trim().is_empty()
        && !bindings
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == executor_id);
    if !independent {
        return Err(
            "Stage 88 执行者必须独立于 Stage 87 复核者和完整 Stage 51–87 责任链".to_string(),
        );
    }
    let mut claim = ControlledShadowForwardObservationExecutionAttemptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        attempt_id: String::new(),
        claim_sha256: String::new(),
        authorization_review_id: bindings.authorization_review_id.clone(),
        authorization_review_sha256: bindings.authorization_review_sha256.clone(),
        authorization_valid_until: bindings.authorization_valid_until,
        isolated_runner_id: bindings.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: bindings.isolated_runner_spec_sha256.clone(),
        runner_contract_sha256: bindings.runner_contract_sha256.clone(),
        runner_code_revision: bindings.runner_code_revision.clone(),
        runner_artifact_sha256: bindings.runner_artifact_sha256.clone(),
        implementation_id: bindings.implementation_id.clone(),
        implementation_sha256: bindings.implementation_sha256.clone(),
        implementation_contract_sha256: bindings.implementation_contract_sha256.clone(),
        implementation_review_sha256: bindings.implementation_review_sha256.clone(),
        protocol_review_sha256: bindings.protocol_review_sha256.clone(),
        protocol_registration_sha256: bindings.protocol_registration_sha256.clone(),
        protocol_specification_sha256: bindings.protocol_specification_sha256.clone(),
        design_specification_sha256: bindings.design_specification_sha256.clone(),
        initial_observation_validation_sha256: bindings
            .initial_observation_validation_sha256
            .clone(),
        initialization_manifest_sha256: manifest_sha256.to_ascii_lowercase(),
        claimed_at,
        invoked_by: executor_id.to_string(),
        excluded_prior_actor_ids: bindings.excluded_prior_actor_ids.clone(),
        executor_independent_from_stage_87_and_complete_prior_chain: independent,
        authorization_consumed: true,
        invocation_started: true,
        initialization_manifest_opened: false,
        persistent_runtime_instantiation_allowed: false,
        input_mount_allowed: false,
        market_data_access_allowed: false,
        forward_observation_write_allowed: false,
        ledger_write_allowed: false,
        position_write_allowed: false,
        performance_metric_write_allowed: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        training_feedback_allowed: false,
        scalar_reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = fingerprint_without(&claim, &["attempt_id", "claim_sha256"])?;
    claim.attempt_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim)?;
    Ok(claim)
}

async fn execute_after_claim(
    bindings: &ForwardObservationAttemptBindings,
    claimed_manifest_sha256: &str,
    manifest: &ControlledShadowForwardObservationInitializationManifest,
    now: DateTime<Utc>,
) -> Result<ControlledShadowForwardObservationUntrustedInitializationReceipt, String> {
    let binary_sha256 = reverify_current_binary_sha256().await?;
    if binary_sha256 != bindings.runner_artifact_sha256 {
        return Err("当前执行二进制摘要与 Stage 86/87 冻结 runner 工件不一致".to_string());
    }
    validate_manifest(bindings, claimed_manifest_sha256, manifest, now)?;
    Ok(
        ControlledShadowForwardObservationUntrustedInitializationReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
            authorization_review_sha256: bindings.authorization_review_sha256.clone(),
            isolated_runner_spec_sha256: bindings.isolated_runner_spec_sha256.clone(),
            runner_artifact_sha256: bindings.runner_artifact_sha256.clone(),
            implementation_contract_sha256: bindings.implementation_contract_sha256.clone(),
            protocol_specification_sha256: bindings.protocol_specification_sha256.clone(),
            design_specification_sha256: bindings.design_specification_sha256.clone(),
            initialization_manifest_sha256: manifest.manifest_sha256.clone(),
            observation_not_before: manifest.observation_not_before,
            requested_at: manifest.requested_at,
            signal_cadence: manifest.signal_cadence.clone(),
            first_eligible_signal_rule: manifest.first_eligible_signal_rule.clone(),
            official_market_calendar: manifest.official_market_calendar.clone(),
            official_market_calendar_source_url: manifest
                .official_market_calendar_source_url
                .clone(),
            official_market_calendar_content_sha256: manifest
                .official_market_calendar_content_sha256
                .clone(),
            benchmark_symbol: manifest.benchmark_symbol.clone(),
            initial_observation_validation_sha256: manifest
                .initial_observation_validation_sha256
                .clone(),
            natural_forward_only: manifest.natural_forward_only,
            retroactive_backfill_allowed: manifest.retroactive_backfill_allowed,
            point_in_time_content_addressed_allowlisted_sources_required: manifest
                .point_in_time_content_addressed_allowlisted_sources_required,
            synchronized_security_and_benchmark_observation_required: manifest
                .synchronized_security_and_benchmark_observation_required,
            initialization_only: true,
            output_is_untrusted: true,
            independent_output_validation_completed: false,
            market_data_rows_attached: false,
            natural_forward_market_sessions_observed: 0,
            persistent_runtime_instantiated: false,
            input_manifest_attached: false,
            market_data_accessed: false,
            forward_observation_started: false,
            ledger_created: false,
            position_written: false,
            performance_metric_written: false,
            model_store_written: false,
            metric_store_written: false,
            training_feedback_written: false,
            scalar_reward_written: false,
            order_intent_present: false,
            broker_payload_present: false,
            trade_executed: false,
        },
    )
}

fn validate_manifest(
    bindings: &ForwardObservationAttemptBindings,
    claimed_manifest_sha256: &str,
    manifest: &ControlledShadowForwardObservationInitializationManifest,
    now: DateTime<Utc>,
) -> Result<(), String> {
    let current_hash = manifest_fingerprint(manifest)?;
    let requested_is_current = manifest.requested_at
        >= now - TimeDelta::minutes(MAX_CLOCK_SKEW_MINUTES)
        && manifest.requested_at <= now + TimeDelta::minutes(MAX_CLOCK_SKEW_MINUTES);
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION
        || !valid_sha256(&manifest.manifest_sha256)
        || manifest.manifest_sha256 != current_hash
        || manifest.manifest_sha256 != claimed_manifest_sha256
        || manifest.observation_not_before != bindings.observation_not_before
        || manifest.requested_at < bindings.observation_not_before
        || !requested_is_current
        || manifest.signal_cadence != bindings.signal_cadence
        || manifest.first_eligible_signal_rule != bindings.first_eligible_signal_rule
        || manifest.official_market_calendar != bindings.official_market_calendar
        || !manifest
            .official_market_calendar_source_url
            .starts_with("https://")
        || !valid_sha256(&manifest.official_market_calendar_content_sha256)
        || manifest.benchmark_symbol != bindings.benchmark_symbol
        || manifest.initial_observation_validation_sha256
            != bindings.initial_observation_validation_sha256
        || !manifest.natural_forward_only
        || manifest.retroactive_backfill_allowed
        || manifest.market_data_rows_attached
        || !manifest.point_in_time_content_addressed_allowlisted_sources_required
        || !manifest.synchronized_security_and_benchmark_observation_required
        || !manifest.initialization_only
    {
        return Err(
            "Stage 88 初始化 manifest 无效、回填、过期、含行情或与冻结协议不一致".to_string(),
        );
    }
    Ok(())
}

fn successful_result(
    claim: &ControlledShadowForwardObservationExecutionAttemptClaim,
    receipt: ControlledShadowForwardObservationUntrustedInitializationReceipt,
    started_at: DateTime<Utc>,
    duration_millis: u64,
) -> Result<ControlledShadowForwardObservationExecutionAttemptResult, String> {
    let output_sha256 = hash_serializable(&receipt)?;
    let mut result = ControlledShadowForwardObservationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        status: ControlledShadowForwardObservationExecutionAttemptStatus::CompletedWithUntrustedInitializationReceipt,
        started_at,
        finished_at: Utc::now(),
        duration_millis,
        isolation_backend: ISOLATION_BACKEND.to_string(),
        exit_code: 0,
        failure_reason: None,
        current_binary_digest_reverified: true,
        initialization_manifest_validated: true,
        initialization_completed: true,
        untrusted_initialization_receipt: Some(receipt),
        output_sha256: Some(output_sha256),
        independent_output_validation_completed: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_store_written: false,
        metric_store_written: false,
        training_feedback_written: false,
        scalar_reward_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    };
    result.result_sha256 = fingerprint_without(&result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn failed_result(
    claim: &ControlledShadowForwardObservationExecutionAttemptClaim,
    error: String,
    started_at: DateTime<Utc>,
    duration_millis: u64,
) -> Result<ControlledShadowForwardObservationExecutionAttemptResult, String> {
    let mut result = ControlledShadowForwardObservationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        status:
            ControlledShadowForwardObservationExecutionAttemptStatus::FailedAuthorizationConsumed,
        started_at,
        finished_at: Utc::now(),
        duration_millis,
        isolation_backend: ISOLATION_BACKEND.to_string(),
        exit_code: 1,
        failure_reason: Some(error.chars().take(MAX_ERROR_CHARS).collect()),
        current_binary_digest_reverified: false,
        initialization_manifest_validated: false,
        initialization_completed: false,
        untrusted_initialization_receipt: None,
        output_sha256: None,
        independent_output_validation_completed: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_store_written: false,
        metric_store_written: false,
        training_feedback_written: false,
        scalar_reward_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    };
    result.result_sha256 = fingerprint_without(&result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn validate_claim(
    value: &ControlledShadowForwardObservationExecutionAttemptClaim,
) -> Result<(), String> {
    let authority_closed = !value.persistent_runtime_instantiation_allowed
        && !value.input_mount_allowed
        && !value.market_data_access_allowed
        && !value.forward_observation_write_allowed
        && !value.ledger_write_allowed
        && !value.position_write_allowed
        && !value.performance_metric_write_allowed
        && !value.model_store_write_allowed
        && !value.metric_store_write_allowed
        && !value.training_feedback_allowed
        && !value.scalar_reward_allowed
        && !value.order_generation_allowed
        && !value.broker_access_allowed
        && !value.trading_allowed;
    if value.schema_version != CLAIM_SCHEMA_VERSION
        || value.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&value.attempt_id)
        || !valid_sha256(&value.claim_sha256)
        || value.attempt_id != value.claim_sha256[..32]
        || value.claim_sha256 != fingerprint_without(value, &["attempt_id", "claim_sha256"])?
        || !valid_sha256(&value.authorization_review_sha256)
        || !valid_sha256(&value.runner_artifact_sha256)
        || !valid_sha256(&value.initialization_manifest_sha256)
        || !sorted_unique(&value.excluded_prior_actor_ids)
        || value
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &value.invoked_by)
        || !value.executor_independent_from_stage_87_and_complete_prior_chain
        || !value.authorization_consumed
        || !value.invocation_started
        || value.initialization_manifest_opened
        || !authority_closed
    {
        return Err("Stage 88 claim 无效、漂移或越权".to_string());
    }
    Ok(())
}

fn validate_result(
    value: &ControlledShadowForwardObservationExecutionAttemptResult,
    claim: &ControlledShadowForwardObservationExecutionAttemptClaim,
) -> Result<(), String> {
    let completed = value.status
        == ControlledShadowForwardObservationExecutionAttemptStatus::CompletedWithUntrustedInitializationReceipt;
    let authority_closed = !value.independent_output_validation_completed
        && !value.forward_observation_started
        && !value.ledger_created
        && !value.position_written
        && !value.performance_metric_written
        && !value.model_store_written
        && !value.metric_store_written
        && !value.training_feedback_written
        && !value.scalar_reward_written
        && !value.order_generated
        && !value.broker_accessed
        && !value.trade_executed;
    if value.schema_version != RESULT_SCHEMA_VERSION
        || value.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&value.result_id)
        || !valid_sha256(&value.result_sha256)
        || value.result_id != value.result_sha256[..32]
        || value.result_sha256 != fingerprint_without(value, &["result_id", "result_sha256"])?
        || value.attempt_id != claim.attempt_id
        || value.claim_sha256 != claim.claim_sha256
        || !authority_closed
        || completed != value.untrusted_initialization_receipt.is_some()
        || completed != value.output_sha256.is_some()
        || completed != value.initialization_completed
        || (completed
            && (!value.current_binary_digest_reverified
                || !value.initialization_manifest_validated))
        || (!completed && value.exit_code == 0)
    {
        return Err("Stage 88 result 无效、漂移或越权".to_string());
    }
    if let Some(receipt) = &value.untrusted_initialization_receipt {
        if value.output_sha256.as_deref() != Some(hash_serializable(receipt)?.as_str())
            || !receipt.initialization_only
            || !receipt
                .official_market_calendar_source_url
                .starts_with("https://")
            || !receipt.natural_forward_only
            || receipt.retroactive_backfill_allowed
            || !receipt.point_in_time_content_addressed_allowlisted_sources_required
            || !receipt.synchronized_security_and_benchmark_observation_required
            || !receipt.output_is_untrusted
            || receipt.independent_output_validation_completed
            || receipt.market_data_rows_attached
            || receipt.natural_forward_market_sessions_observed != 0
            || receipt.persistent_runtime_instantiated
            || receipt.input_manifest_attached
            || receipt.market_data_accessed
            || receipt.forward_observation_started
            || receipt.ledger_created
            || receipt.position_written
            || receipt.performance_metric_written
            || receipt.model_store_written
            || receipt.metric_store_written
            || receipt.training_feedback_written
            || receipt.scalar_reward_written
            || receipt.order_intent_present
            || receipt.broker_payload_present
            || receipt.trade_executed
        {
            return Err("Stage 88 不可信初始化收据包含行情、观察、绩效或越权载荷".to_string());
        }
    }
    Ok(())
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<ControlledShadowForwardObservationExecutionAttemptClaim>,
        Vec<ControlledShadowForwardObservationExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<ControlledShadowForwardObservationExecutionAttemptClaim>(
        &execution_root(state).join("claims"),
    )
    .await?;
    let results = read_nested_json::<ControlledShadowForwardObservationExecutionAttemptResult>(
        &execution_root(state).join("results"),
    )
    .await?;
    let mut attempt_ids = BTreeSet::new();
    let mut authorization_ids = BTreeSet::new();
    for claim in &claims {
        validate_claim(claim)?;
        if !attempt_ids.insert(claim.attempt_id.clone())
            || !authorization_ids.insert(claim.authorization_review_id.clone())
        {
            return Err("Stage 88 claim 重复".to_string());
        }
        exact_forward_observation_first_execution_authorization_for_audit(
            state,
            &claim.isolated_runner_id,
            &claim.authorization_review_id,
            &claim.authorization_review_sha256,
        )
        .await?
        .ok_or_else(|| "Stage 88 claim 的 Stage 87 授权链无法重开".to_string())?;
    }
    let claim_index = claims
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut result_ids = BTreeSet::new();
    let mut result_attempts = BTreeSet::new();
    for result in &results {
        let claim = claim_index
            .get(result.attempt_id.as_str())
            .ok_or_else(|| "Stage 88 result 缺少 claim".to_string())?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_attempts.insert(result.attempt_id.clone())
        {
            return Err("Stage 88 result 重复".to_string());
        }
    }
    Ok((claims, results))
}

async fn read_nested_json<T: for<'de> Deserialize<'de>>(root: &Path) -> Result<Vec<T>, String> {
    let mut stack = vec![root.to_path_buf()];
    let mut values = Vec::new();
    while let Some(directory) = stack.pop() {
        let mut entries = match tokio::fs::read_dir(&directory).await {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.to_string()),
        };
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|error| error.to_string())?
        {
            let path = entry.path();
            let file_type = entry.file_type().await.map_err(|error| error.to_string())?;
            if file_type.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
                values.push(
                    serde_json::from_slice(
                        &tokio::fs::read(&path)
                            .await
                            .map_err(|error| error.to_string())?,
                    )
                    .map_err(|error| error.to_string())?,
                );
            }
        }
    }
    Ok(values)
}

async fn reverify_current_binary_sha256() -> Result<String, String> {
    Ok(hash_bytes(
        &tokio::fs::read(std::env::current_exe().map_err(|error| error.to_string())?)
            .await
            .map_err(|error| error.to_string())?,
    ))
}

fn manifest_fingerprint(
    value: &ControlledShadowForwardObservationInitializationManifest,
) -> Result<String, String> {
    fingerprint_without(value, &["manifest_sha256"])
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(hash_bytes(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn execution_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-forward-observation-execution-attempts")
}

fn claim_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("claims").join(runner_id)
}

fn result_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("results").join(runner_id)
}

struct ExecutionLock {
    path: PathBuf,
}

impl Drop for ExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, runner_id: &str) -> Result<ExecutionLock, String> {
    let directory = execution_root(state).join("locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{runner_id}.lock"));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ExecutionLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                    .is_some_and(|age| age > StdDuration::from_secs(3_600));
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                if attempt < 7 {
                    tokio::time::sleep(StdDuration::from_millis(25)).await;
                    continue;
                }
                return Err("另一个 Stage 88 初始化尝试正在处理该 runner".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 88 初始化尝试锁不可用".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings(now: DateTime<Utc>) -> ForwardObservationAttemptBindings {
        let hash = "a".repeat(64);
        ForwardObservationAttemptBindings {
            authorization_review_id: "b".repeat(32),
            authorization_review_sha256: hash.clone(),
            authorization_valid_until: now + TimeDelta::hours(24),
            isolated_runner_id: "c".repeat(32),
            isolated_runner_spec_sha256: hash.clone(),
            runner_contract_sha256: hash.clone(),
            runner_code_revision: "revision".to_string(),
            runner_artifact_sha256: hash.clone(),
            implementation_id: "d".repeat(32),
            implementation_sha256: hash.clone(),
            implementation_contract_sha256: hash.clone(),
            implementation_review_sha256: hash.clone(),
            protocol_review_sha256: hash.clone(),
            protocol_registration_sha256: hash.clone(),
            protocol_specification_sha256: hash.clone(),
            design_specification_sha256: hash.clone(),
            initial_observation_validation_sha256: hash,
            observation_not_before: now - TimeDelta::hours(1),
            signal_cadence: "weekly".to_string(),
            first_eligible_signal_rule: "next_full_market_session".to_string(),
            official_market_calendar: "NYSE official calendar".to_string(),
            benchmark_symbol: "SPY".to_string(),
            excluded_prior_actor_ids: vec!["prior".to_string(), "reviewer".to_string()],
        }
    }

    fn manifest(
        bindings: &ForwardObservationAttemptBindings,
        now: DateTime<Utc>,
    ) -> ControlledShadowForwardObservationInitializationManifest {
        let mut value = ControlledShadowForwardObservationInitializationManifest {
            schema_version: MANIFEST_SCHEMA_VERSION.to_string(),
            manifest_sha256: String::new(),
            requested_at: now,
            observation_not_before: bindings.observation_not_before,
            signal_cadence: bindings.signal_cadence.clone(),
            first_eligible_signal_rule: bindings.first_eligible_signal_rule.clone(),
            official_market_calendar: bindings.official_market_calendar.clone(),
            official_market_calendar_source_url: "https://www.nyse.com/markets/hours-calendars"
                .to_string(),
            official_market_calendar_content_sha256: "e".repeat(64),
            benchmark_symbol: bindings.benchmark_symbol.clone(),
            initial_observation_validation_sha256: bindings
                .initial_observation_validation_sha256
                .clone(),
            natural_forward_only: true,
            retroactive_backfill_allowed: false,
            market_data_rows_attached: false,
            point_in_time_content_addressed_allowlisted_sources_required: true,
            synchronized_security_and_benchmark_observation_required: true,
            initialization_only: true,
        };
        value.manifest_sha256 = manifest_fingerprint(&value).unwrap();
        value
    }

    #[test]
    fn claim_is_content_addressed_consumes_authorization_and_keeps_every_authority_closed() {
        let now = Utc::now();
        let bindings = bindings(now);
        let manifest = manifest(&bindings, now);
        let claim = new_claim(&bindings, "executor", &manifest.manifest_sha256, now).unwrap();
        assert_eq!(claim.attempt_id, claim.claim_sha256[..32]);
        assert!(claim.authorization_consumed);
        assert!(validate_claim(&claim).is_ok());
        assert!(!claim.market_data_access_allowed);
        assert!(!claim.forward_observation_write_allowed);
        assert!(!claim.order_generation_allowed);
        assert!(!claim.trading_allowed);
    }

    #[test]
    fn manifest_rejects_backfill_market_data_and_stale_requests() {
        let now = Utc::now();
        let bindings = bindings(now);
        let mut value = manifest(&bindings, now);
        assert!(validate_manifest(&bindings, &value.manifest_sha256, &value, now).is_ok());
        value.retroactive_backfill_allowed = true;
        value.manifest_sha256 = manifest_fingerprint(&value).unwrap();
        assert!(validate_manifest(&bindings, &value.manifest_sha256, &value, now).is_err());
        value.retroactive_backfill_allowed = false;
        value.market_data_rows_attached = true;
        value.manifest_sha256 = manifest_fingerprint(&value).unwrap();
        assert!(validate_manifest(&bindings, &value.manifest_sha256, &value, now).is_err());
        value.market_data_rows_attached = false;
        value.requested_at = now - TimeDelta::minutes(MAX_CLOCK_SKEW_MINUTES + 1);
        value.manifest_sha256 = manifest_fingerprint(&value).unwrap();
        assert!(validate_manifest(&bindings, &value.manifest_sha256, &value, now).is_err());
    }

    #[test]
    fn failed_attempt_consumes_authorization_without_receipt_or_execution_authority() {
        let now = Utc::now();
        let bindings = bindings(now);
        let manifest = manifest(&bindings, now);
        let claim = new_claim(&bindings, "executor", &manifest.manifest_sha256, now).unwrap();
        let result = failed_result(&claim, "digest mismatch".to_string(), now, 1).unwrap();
        assert_eq!(result.exit_code, 1);
        assert!(result.untrusted_initialization_receipt.is_none());
        assert!(!result.forward_observation_started);
        assert!(!result.order_generated);
        assert!(!result.trade_executed);
        assert!(validate_result(&result, &claim).is_ok());
    }

    #[test]
    fn successful_receipt_has_zero_market_sessions_and_no_runtime_ledger_position_or_trade() {
        let now = Utc::now();
        let bindings = bindings(now);
        let manifest = manifest(&bindings, now);
        let claim = new_claim(&bindings, "executor", &manifest.manifest_sha256, now).unwrap();
        let receipt = ControlledShadowForwardObservationUntrustedInitializationReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
            authorization_review_sha256: bindings.authorization_review_sha256.clone(),
            isolated_runner_spec_sha256: bindings.isolated_runner_spec_sha256.clone(),
            runner_artifact_sha256: bindings.runner_artifact_sha256.clone(),
            implementation_contract_sha256: bindings.implementation_contract_sha256.clone(),
            protocol_specification_sha256: bindings.protocol_specification_sha256.clone(),
            design_specification_sha256: bindings.design_specification_sha256.clone(),
            initialization_manifest_sha256: manifest.manifest_sha256.clone(),
            observation_not_before: manifest.observation_not_before,
            requested_at: manifest.requested_at,
            signal_cadence: manifest.signal_cadence.clone(),
            first_eligible_signal_rule: manifest.first_eligible_signal_rule.clone(),
            official_market_calendar: manifest.official_market_calendar.clone(),
            official_market_calendar_source_url: manifest
                .official_market_calendar_source_url
                .clone(),
            official_market_calendar_content_sha256: manifest
                .official_market_calendar_content_sha256
                .clone(),
            benchmark_symbol: manifest.benchmark_symbol.clone(),
            initial_observation_validation_sha256: manifest
                .initial_observation_validation_sha256
                .clone(),
            natural_forward_only: true,
            retroactive_backfill_allowed: false,
            point_in_time_content_addressed_allowlisted_sources_required: true,
            synchronized_security_and_benchmark_observation_required: true,
            initialization_only: true,
            output_is_untrusted: true,
            independent_output_validation_completed: false,
            market_data_rows_attached: false,
            natural_forward_market_sessions_observed: 0,
            persistent_runtime_instantiated: false,
            input_manifest_attached: false,
            market_data_accessed: false,
            forward_observation_started: false,
            ledger_created: false,
            position_written: false,
            performance_metric_written: false,
            model_store_written: false,
            metric_store_written: false,
            training_feedback_written: false,
            scalar_reward_written: false,
            order_intent_present: false,
            broker_payload_present: false,
            trade_executed: false,
        };
        let result = successful_result(&claim, receipt, now, 1).unwrap();
        assert!(validate_result(&result, &claim).is_ok());
        let receipt = result.untrusted_initialization_receipt.unwrap();
        assert_eq!(receipt.natural_forward_market_sessions_observed, 0);
        assert!(!receipt.persistent_runtime_instantiated);
        assert!(!receipt.ledger_created);
        assert!(!receipt.position_written);
        assert!(!receipt.trade_executed);
    }
}
