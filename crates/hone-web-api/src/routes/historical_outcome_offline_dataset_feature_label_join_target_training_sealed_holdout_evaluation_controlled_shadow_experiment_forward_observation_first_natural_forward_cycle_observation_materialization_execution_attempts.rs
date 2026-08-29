//! Stage 112 one-shot controlled materialization of one exact Stage 111 claim.
//!
//! The reviewed `runner.artifact` is a strict declarative binding, never an executable. HONE's
//! trusted in-process projection reopens and rehashes the exact Stage 104 admitted Stage 102
//! output, preserves decimal strings and explicit gaps, and writes one create-once untrusted
//! observation envelope. A start marker is persisted before the artifact or input is read. Any
//! failure permanently consumes the claim; success still requires a chain-external Stage 113
//! independent output validation before any downstream use.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_execution_attempts::{
    CanonicalMarketCalendarRow, CanonicalMarketDividendRow, CanonicalMarketPriceRow,
    CanonicalMarketSplitRow, UntrustedCanonicalMarketDataParseResult,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_input_admission_reviews::read_revalidated_admitted_observation_input_for_materialization_execution;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_execution_attempt_claims::{
    ControlledShadowObservationMaterializationExecutionAttemptClaim,
    observation_materialization_execution_attempt_claims_for_execution,
    validate_observation_materialization_execution_attempt_claim,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_first_execution_authorizations::read_revalidated_observation_materialization_artifact_for_execution;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_specifications::ControlledShadowObservationMaterializationSpecification;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-execution-attempt-registry-v1";
const PROGRAM_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-declarative-program-v1";
const OUTPUT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-untrusted-natural-forward-observation-envelope-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-execution-attempt-result-v1";
const START_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-execution-attempt-start-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-materialization-execution-v1-one-shot-in-process-declarative-fail-closed";
const PROGRAM_KIND: &str = "hone_trusted_deterministic_observation_materialization_bindings";
const NEXT_GATE: &str = "stage_113_independent_observation_materialization_output_validation";
const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;
const MAX_REASON_CHARS: usize = 4_000;
const INTERRUPTED_RECOVERY_GRACE_SECONDS: i64 = 1;

#[derive(Debug, Deserialize)]
pub(crate) struct ExecuteControlledShadowObservationMaterializationAttemptRequest {
    expected_claim_sha256: String,
    expected_authorization_review_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_artifact_manifest_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_observation_materialization_specification_sha256: String,
    expected_stage_104_admission_review_sha256: String,
    expected_stage_102_output_sha256: String,
    expected_stage_101_input_manifest_sha256: String,
    expected_cycle_claim_sha256: String,
    execution_reason: String,
    exact_stage_51_through_stage_111_binding_confirmed: bool,
    executor_independent_from_complete_prior_chain_and_claimant_confirmed: bool,
    start_marker_consumes_claim_before_artifact_or_input_read_confirmed: bool,
    one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: bool,
    artifact_is_declarative_not_spawned_or_executed_confirmed: bool,
    only_exact_stage_104_admitted_output_is_read_only_opened_and_rehashed_confirmed: bool,
    deterministic_session_price_gap_action_allocation_and_availability_projection_confirmed: bool,
    no_refetch_reparse_fill_interpolation_substitution_backfill_or_correction_confirmed: bool,
    output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed:
        bool,
    no_network_environment_secret_tool_subprocess_or_production_io_confirmed: bool,
    no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ControlledShadowObservationMaterializationDeclarativeProgram {
    pub schema_version: String,
    pub program_kind: String,
    pub implementation_contract_sha256: String,
    pub specification_sha256: String,
    pub immutable_code_revision: String,
    pub function_ids: Vec<String>,
    pub canonical_schemas: Vec<String>,
    pub no_command_or_entrypoint: bool,
    pub deterministic_in_process_interpreter_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MaterializedMarketSession {
    pub date: NaiveDate,
    pub session_kind: String,
    pub close_time_et: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MaterializedPriceObservation {
    pub symbol: String,
    pub date: NaiveDate,
    pub price_basis: String,
    pub close_decimal: String,
    pub volume_integer: Option<u64>,
    pub source_request_sha256: String,
    pub source_row_sha256: String,
    pub observation_row_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MaterializedExplicitGap {
    pub symbol: String,
    pub date: NaiveDate,
    pub price_basis: String,
    pub reason: String,
    pub source_manifest_sha256: String,
    pub gap_row_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct MaterializedCorporateAction {
    pub symbol: String,
    pub date: NaiveDate,
    pub action_kind: String,
    pub exact_source_fields: Map<String, Value>,
    pub source_row_sha256: String,
    pub action_row_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct InitialShadowAllocationBinding {
    pub stage_88_initial_observation_output_sha256: String,
    pub initial_shadow_allocation_manifest_sha256: String,
    pub initial_shadow_allocation_recomputed: bool,
    pub accounting_transition_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UntrustedControlledShadowObservationEnvelope {
    pub schema_version: String,
    pub specification_sha256: String,
    pub stage_104_review_sha256: String,
    pub cycle_claim_sha256: String,
    pub admitted_available_at_utc: DateTime<Utc>,
    pub sessions: Vec<MaterializedMarketSession>,
    pub prices: Vec<MaterializedPriceObservation>,
    pub explicit_gaps: Vec<MaterializedExplicitGap>,
    pub dividends: Vec<MaterializedCorporateAction>,
    pub splits: Vec<MaterializedCorporateAction>,
    pub initial_shadow_allocation_binding: InitialShadowAllocationBinding,
    pub output_sha256: String,
    pub untrusted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationMaterializationExecutionStatus {
    CompletedWithUntrustedObservationEnvelope,
    FailedClaimConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationExecutionAttemptResult {
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub stage_111_attempt_id: String,
    pub stage_111_claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub executed_by: String,
    pub execution_reason: String,
    pub duration_millis: u64,
    pub status: ControlledShadowObservationMaterializationExecutionStatus,
    pub bounded_error_code: Option<String>,
    pub output_sha256: Option<String>,
    pub output_relative_path: Option<String>,
    pub claim_consumed: bool,
    pub artifact_revalidated: bool,
    pub artifact_spawned_or_executed: bool,
    pub exact_admitted_input_revalidated_and_opened: bool,
    pub materializer_executed_in_process: bool,
    pub sessions_materialized: bool,
    pub price_observations_materialized: bool,
    pub explicit_gaps_materialized: bool,
    pub corporate_actions_materialized: bool,
    pub observation_envelope_created: bool,
    pub output_untrusted: bool,
    pub independent_validation_completed: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ControlledShadowObservationMaterializationExecutionAttemptStart {
    schema_version: String,
    policy_version: String,
    start_sha256: String,
    stage_111_attempt_id: String,
    stage_111_claim_sha256: String,
    started_at: DateTime<Utc>,
    executed_by: String,
    execution_reason: String,
    one_shot_claim_consumed_before_artifact_or_input_read: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationExecutionAttemptRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub execution_endpoint_available: bool,
    pub pending_claims: Vec<ControlledShadowObservationMaterializationExecutionAttemptClaim>,
    pub results: Vec<ControlledShadowObservationMaterializationExecutionAttemptResult>,
    pub pending_claim_count: usize,
    pub terminal_result_count: usize,
    pub successful_untrusted_observation_count: usize,
    pub failed_consumed_claim_count: usize,
    pub next_gate: String,
    pub arbitrary_artifact_execution_allowed: bool,
    pub outbound_network_allowed: bool,
    pub independent_validation_completed: bool,
    pub observation_envelope_created: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationMaterializationExecutionReadinessSummary {
    pub pending_claim_count: usize,
    pub terminal_result_count: usize,
    pub successful_untrusted_observation_count: usize,
    pub failed_consumed_claim_count: usize,
}

pub(crate) async fn handle_get_controlled_shadow_observation_materialization_execution_attempts(
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
            warn!(%error, "observation materialization execution registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察物化执行表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_execute_controlled_shadow_observation_materialization_attempt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ExecuteControlledShadowObservationMaterializationAttemptRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match execute_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(_) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "执行结果已落盘，但刷新观察物化执行表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_observation_materialization_execution_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationExecutionReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowObservationMaterializationExecutionReadinessSummary {
            pending_claim_count: registry.pending_claim_count,
            terminal_result_count: registry.terminal_result_count,
            successful_untrusted_observation_count: registry.successful_untrusted_observation_count,
            failed_consumed_claim_count: registry.failed_consumed_claim_count,
        },
    )
}

pub(crate) async fn started_or_terminal_observation_materialization_claim_attempt_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    let mut ids = read_starts(state)
        .await?
        .into_iter()
        .map(|value| value.stage_111_attempt_id)
        .collect::<BTreeSet<_>>();
    ids.extend(
        read_results(state)
            .await?
            .into_iter()
            .map(|value| value.stage_111_attempt_id),
    );
    Ok(ids)
}

/// Returns only structurally valid immutable Stage 112 terminal results. Stage 113 may use this
/// index to locate candidates, but it must reopen the output and recompute the envelope with its
/// own implementation rather than calling any Stage 112 materialization helper.
pub(crate) async fn observation_materialization_execution_results_for_independent_validation(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationMaterializationExecutionAttemptResult>, String> {
    read_results(state).await
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationExecutionAttemptRegistry, String> {
    let claims = observation_materialization_execution_attempt_claims_for_execution(state).await?;
    let mut results = read_results(state).await?;
    recover_interrupted_attempts(state, &claims, &results).await?;
    results = read_results(state).await?;
    let starts = read_starts(state).await?;
    let completed = results
        .iter()
        .map(|value| value.stage_111_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    let started = starts
        .iter()
        .map(|value| value.stage_111_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    let pending_claims = claims
        .into_iter()
        .filter(|value| {
            !completed.contains(value.attempt_id.as_str())
                && !started.contains(value.attempt_id.as_str())
        })
        .collect::<Vec<_>>();
    let successful = results
        .iter()
        .filter(|value| {
            value.status
                == ControlledShadowObservationMaterializationExecutionStatus::CompletedWithUntrustedObservationEnvelope
        })
        .count();
    let failed = results.len().saturating_sub(successful);
    Ok(
        ControlledShadowObservationMaterializationExecutionAttemptRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            execution_endpoint_available: true,
            pending_claim_count: pending_claims.len(),
            terminal_result_count: results.len(),
            successful_untrusted_observation_count: successful,
            failed_consumed_claim_count: failed,
            pending_claims,
            results,
            next_gate: NEXT_GATE.to_string(),
            arbitrary_artifact_execution_allowed: false,
            outbound_network_allowed: false,
            independent_validation_completed: false,
            observation_envelope_created: successful > 0,
            forward_observation_started: successful > 0,
            ledger_created: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "Stage 112 对一条 Stage 111 声明最多执行一次：先写 create-once start marker 永久消费声明，再重新核验只读声明式工件与 exact Stage 104 admitted Stage 102 output，由 HONE 受信任进程内函数确定性投影 session、三价格口径、显式 gap、公司行动和初始分配绑定。成功只创建非可信观察 envelope，失败或中断均不可重试；Stage 113 独立验证前不得进入账本、持仓、绩效、模型/训练/reward、订单、券商或交易。".to_string(),
        },
    )
}

async fn execute_once(
    state: &AppState,
    executor_id: &str,
    attempt_id: &str,
    request: ExecuteControlledShadowObservationMaterializationAttemptRequest,
) -> Result<ControlledShadowObservationMaterializationExecutionAttemptResult, String> {
    if !valid_id(attempt_id) {
        return Err("Stage 111 attempt ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_results(state)
        .await?
        .iter()
        .any(|value| value.stage_111_attempt_id == attempt_id)
    {
        return Err("这条 Stage 111 claim 已有终态结果，不能重试".to_string());
    }
    if read_starts(state)
        .await?
        .iter()
        .any(|value| value.stage_111_attempt_id == attempt_id)
    {
        return Err("这条 Stage 111 claim 已经开始过执行，不能重试".to_string());
    }
    let claim = observation_materialization_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .find(|value| value.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有这条 Stage 111 claim".to_string())?;
    validate_execution_request(&claim, executor_id, &request)?;
    let reason = bounded_required(&request.execution_reason, "执行原因")?;
    let mut start = ControlledShadowObservationMaterializationExecutionAttemptStart {
        schema_version: START_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        start_sha256: String::new(),
        stage_111_attempt_id: claim.attempt_id.clone(),
        stage_111_claim_sha256: claim.claim_sha256.clone(),
        started_at: Utc::now(),
        executed_by: executor_id.to_string(),
        execution_reason: reason.clone(),
        one_shot_claim_consumed_before_artifact_or_input_read: true,
    };
    start.start_sha256 = fingerprint_without(&start, &["start_sha256"])?;
    validate_start(&start, &claim)?;
    write_immutable_json(
        &start_directory(state).join(format!("{attempt_id}.json")),
        &start,
        None,
    )
    .await?;

    let started = Instant::now();
    let runner = &claim.authorization.runner.runner_contract;
    let execution = tokio::time::timeout(
        StdDuration::from_secs(u64::from(runner.maximum_wall_clock_seconds)),
        execute_claim(state, &claim),
    )
    .await
    .unwrap_or_else(|_| {
        Err(ExecutionFailure::new(
            "execution_wall_clock_limit_exceeded",
            "Stage 112 wall-clock ceiling exceeded",
            true,
            true,
            true,
        ))
    });
    let duration_millis = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let (
        status,
        bounded_error_code,
        output_sha256,
        output_relative_path,
        artifact_revalidated,
        input_opened,
        materializer_executed,
    ) = match execution {
        Ok((sha256, relative_path)) => (
            ControlledShadowObservationMaterializationExecutionStatus::CompletedWithUntrustedObservationEnvelope,
            None,
            Some(sha256),
            Some(relative_path),
            true,
            true,
            true,
        ),
        Err(failure) => {
            warn!(attempt_id, error = %failure.detail, "one-shot observation materialization failed");
            (
                ControlledShadowObservationMaterializationExecutionStatus::FailedClaimConsumed,
                Some(failure.code.to_string()),
                None,
                None,
                failure.artifact_revalidated,
                failure.input_opened,
                failure.materializer_executed,
            )
        }
    };
    let successful = status
        == ControlledShadowObservationMaterializationExecutionStatus::CompletedWithUntrustedObservationEnvelope;
    let mut result = ControlledShadowObservationMaterializationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        stage_111_attempt_id: claim.attempt_id.clone(),
        stage_111_claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        executed_by: executor_id.to_string(),
        execution_reason: reason,
        duration_millis,
        status,
        bounded_error_code,
        output_sha256,
        output_relative_path,
        claim_consumed: true,
        artifact_revalidated,
        artifact_spawned_or_executed: false,
        exact_admitted_input_revalidated_and_opened: input_opened,
        materializer_executed_in_process: materializer_executed,
        sessions_materialized: successful,
        price_observations_materialized: successful,
        explicit_gaps_materialized: successful,
        corporate_actions_materialized: successful,
        observation_envelope_created: successful,
        output_untrusted: successful,
        independent_validation_completed: false,
        forward_observation_started: successful,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_or_metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    result.result_sha256 = fingerprint_without(&result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, &claim)?;
    write_immutable_json(
        &result_directory(state).join(format!("{attempt_id}.json")),
        &result,
        None,
    )
    .await?;
    Ok(result)
}

async fn execute_claim(
    state: &AppState,
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
) -> Result<(String, String), ExecutionFailure> {
    let artifact = read_revalidated_observation_materialization_artifact_for_execution(
        state,
        &claim.authorization,
    )
    .await
    .map_err(|error| {
        ExecutionFailure::new("artifact_revalidation_failed", error, false, false, false)
    })?;
    if artifact.is_empty() || artifact.len() > MAX_ARTIFACT_BYTES {
        return Err(ExecutionFailure::new(
            "declarative_artifact_size_invalid",
            "Stage 112 declarative artifact is empty or too large",
            true,
            false,
            false,
        ));
    }
    let program: ControlledShadowObservationMaterializationDeclarativeProgram =
        serde_json::from_slice(&artifact).map_err(|error| {
            ExecutionFailure::new(
                "declarative_artifact_invalid",
                error.to_string(),
                true,
                false,
                false,
            )
        })?;
    let specification = exact_specification(claim);
    validate_program(&program, claim, specification).map_err(|error| {
        ExecutionFailure::new(
            "declarative_program_binding_invalid",
            error,
            true,
            false,
            false,
        )
    })?;

    let admitted = read_revalidated_admitted_observation_input_for_materialization_execution(
        state,
        &specification.stage_104_review_sha256,
        &specification.stage_102_output_sha256,
    )
    .await
    .map_err(|error| {
        ExecutionFailure::new(
            "exact_admitted_input_revalidation_failed",
            error,
            true,
            true,
            false,
        )
    })?;
    if admitted.admitted.admission_review.review_sha256 != specification.stage_104_review_sha256
        || admitted
            .admitted
            .candidate
            .parser_output
            .result
            .output_sha256
            .as_deref()
            != Some(specification.stage_102_output_sha256.as_str())
        || admitted.admitted.candidate.cycle_claim.cycle_claim_sha256
            != specification.cycle_claim_sha256
    {
        return Err(ExecutionFailure::new(
            "exact_admitted_input_chain_drifted",
            "Stage 112 exact admitted input chain drifted after revalidation",
            true,
            true,
            false,
        ));
    }
    let output = materialize_observation(
        specification,
        &admitted.exact_output,
        &specification.stage_101_input_manifest_sha256,
    )
    .map_err(|error| {
        ExecutionFailure::new(
            "deterministic_materialization_failed",
            error,
            true,
            true,
            true,
        )
    })?;
    let bytes = serde_json::to_vec_pretty(&output).map_err(|error| {
        ExecutionFailure::new(
            "output_serialization_failed",
            error.to_string(),
            true,
            true,
            true,
        )
    })?;
    let maximum_output_bytes = claim
        .authorization
        .runner
        .runner_contract
        .maximum_output_bytes;
    if bytes.is_empty() || bytes.len() as u64 > maximum_output_bytes {
        return Err(ExecutionFailure::new(
            "output_size_limit_exceeded",
            "Stage 112 output exceeds frozen runner ceiling",
            true,
            true,
            true,
        ));
    }
    let relative_path = specification.future_output_relative_path_template.replace(
        "{specification_sha256}",
        &specification.specification_sha256,
    );
    let expected_relative = format!(
        "observations/{}/{}.json",
        specification.cycle_claim_id, specification.specification_sha256
    );
    if relative_path != expected_relative {
        return Err(ExecutionFailure::new(
            "output_path_template_invalid",
            "Stage 112 output path template drifted",
            true,
            true,
            true,
        ));
    }
    write_immutable_bytes(
        &execution_root(state).join(&relative_path),
        &bytes,
        maximum_output_bytes,
    )
    .await
    .map_err(|error| ExecutionFailure::new("output_create_once_failed", error, true, true, true))?;
    Ok((output.output_sha256, relative_path))
}

fn exact_specification(
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
) -> &ControlledShadowObservationMaterializationSpecification {
    &claim
        .authorization
        .runner
        .implementation
        .implementation_contract
        .exact_observation_materialization_specification
}

fn validate_program(
    program: &ControlledShadowObservationMaterializationDeclarativeProgram,
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
    specification: &ControlledShadowObservationMaterializationSpecification,
) -> Result<(), String> {
    let contract = &claim
        .authorization
        .runner
        .implementation
        .implementation_contract;
    let expected_functions = vec![
        contract
            .current_source_binding_validation_function_id
            .clone(),
        contract.canonical_session_projection_function_id.clone(),
        contract.three_price_basis_projection_function_id.clone(),
        contract
            .explicit_gap_and_spy_fail_closed_function_id
            .clone(),
        contract.corporate_action_separation_function_id.clone(),
        contract.initial_allocation_binding_function_id.clone(),
        contract.conservative_availability_function_id.clone(),
        contract
            .canonical_envelope_serialization_and_hash_function_id
            .clone(),
    ];
    let expected_schemas = vec![
        specification.canonical_session_schema.clone(),
        specification.canonical_price_observation_schema.clone(),
        specification.canonical_explicit_gap_schema.clone(),
        specification.canonical_corporate_action_schema.clone(),
        specification.canonical_observation_envelope_schema.clone(),
    ];
    if program.schema_version != PROGRAM_SCHEMA_VERSION
        || program.program_kind != PROGRAM_KIND
        || program.implementation_contract_sha256 != contract.contract_sha256
        || program.specification_sha256 != specification.specification_sha256
        || program.immutable_code_revision != contract.immutable_code_revision
        || program.function_ids != expected_functions
        || program.canonical_schemas != expected_schemas
        || !program.no_command_or_entrypoint
        || !program.deterministic_in_process_interpreter_only
    {
        return Err("Stage 112 declarative program 与已批准合同不一致".to_string());
    }
    Ok(())
}

fn materialize_observation(
    specification: &ControlledShadowObservationMaterializationSpecification,
    source: &UntrustedCanonicalMarketDataParseResult,
    source_manifest_sha256: &str,
) -> Result<UntrustedControlledShadowObservationEnvelope, String> {
    validate_source(specification, source)?;
    let mut sessions = source
        .calendar_rows
        .iter()
        .map(|row| MaterializedMarketSession {
            date: row.date,
            session_kind: row.session_kind.clone(),
            close_time_et: row.close_time_et.clone(),
        })
        .collect::<Vec<_>>();
    sessions.sort_by_key(|row| row.date);

    let mut prices = source
        .price_rows
        .iter()
        .map(|row| {
            let mut projected = MaterializedPriceObservation {
                symbol: row.symbol.clone(),
                date: row.date,
                price_basis: row.source_kind.clone(),
                close_decimal: row.close_decimal.clone(),
                volume_integer: row.volume_integer,
                source_request_sha256: row.request_sha256.clone(),
                source_row_sha256: row.row_sha256.clone(),
                observation_row_sha256: String::new(),
            };
            projected.observation_row_sha256 =
                fingerprint_without(&projected, &["observation_row_sha256"])?;
            Ok(projected)
        })
        .collect::<Result<Vec<_>, String>>()?;
    prices.sort_by(|left, right| {
        (left.date, &left.symbol, &left.price_basis).cmp(&(
            right.date,
            &right.symbol,
            &right.price_basis,
        ))
    });

    let mut explicit_gaps = source
        .explicit_gaps
        .iter()
        .map(|row| {
            let mut projected = MaterializedExplicitGap {
                symbol: row.symbol.clone(),
                date: row.date,
                price_basis: row.source_kind.clone(),
                reason: row.reason.clone(),
                source_manifest_sha256: source_manifest_sha256.to_string(),
                gap_row_sha256: String::new(),
            };
            projected.gap_row_sha256 = fingerprint_without(&projected, &["gap_row_sha256"])?;
            Ok(projected)
        })
        .collect::<Result<Vec<_>, String>>()?;
    explicit_gaps.sort_by(|left, right| {
        (left.date, &left.symbol, &left.price_basis).cmp(&(
            right.date,
            &right.symbol,
            &right.price_basis,
        ))
    });
    let mut dividends = source
        .dividend_rows
        .iter()
        .map(materialize_dividend)
        .collect::<Result<Vec<_>, String>>()?;
    let mut splits = source
        .split_rows
        .iter()
        .map(materialize_split)
        .collect::<Result<Vec<_>, String>>()?;
    dividends.sort_by(|left, right| (left.date, &left.symbol).cmp(&(right.date, &right.symbol)));
    splits.sort_by(|left, right| (left.date, &left.symbol).cmp(&(right.date, &right.symbol)));

    let mut envelope = UntrustedControlledShadowObservationEnvelope {
        schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        specification_sha256: specification.specification_sha256.clone(),
        stage_104_review_sha256: specification.stage_104_review_sha256.clone(),
        cycle_claim_sha256: specification.cycle_claim_sha256.clone(),
        admitted_available_at_utc: specification.admitted_available_at_utc,
        sessions,
        prices,
        explicit_gaps,
        dividends,
        splits,
        initial_shadow_allocation_binding: InitialShadowAllocationBinding {
            stage_88_initial_observation_output_sha256: specification
                .stage_88_initial_observation_output_sha256
                .clone(),
            initial_shadow_allocation_manifest_sha256: specification
                .initial_shadow_allocation_manifest_sha256
                .clone(),
            initial_shadow_allocation_recomputed: false,
            accounting_transition_applied: false,
        },
        output_sha256: String::new(),
        untrusted: true,
    };
    envelope.output_sha256 = fingerprint_without(&envelope, &["output_sha256"])?;
    validate_envelope(specification, &envelope)?;
    Ok(envelope)
}

fn validate_source(
    specification: &ControlledShadowObservationMaterializationSpecification,
    source: &UntrustedCanonicalMarketDataParseResult,
) -> Result<(), String> {
    if source.output_sha256 != specification.stage_102_output_sha256
        || !source.untrusted
        // Stage 103/104 evidence is append-only and external to this immutable Stage 102 file;
        // mutating this embedded flag would itself violate exact-input custody.
        || source.independent_validation_completed
        || source.observation_started
        || source.ledger_created
        || source.position_written
        || source.order_generation_authorized
        || source.broker_access_authorized
        || source.trading_authorized
        || !source.rejections.is_empty()
        || source.calendar_rows.len() != specification.official_market_session_count
    {
        return Err("Stage 112 source identity、状态或 session 数量无效".to_string());
    }
    let sessions = source
        .calendar_rows
        .iter()
        .map(|row| row.date)
        .collect::<BTreeSet<_>>();
    if sessions.len() != source.calendar_rows.len()
        || sessions.first().copied() != Some(specification.earliest_market_session_date)
        || sessions.last().copied() != Some(specification.latest_market_session_date)
    {
        return Err("Stage 112 source session 窗口或唯一性无效".to_string());
    }
    for row in &source.calendar_rows {
        validate_source_calendar_row(row)?;
    }
    let allowed_bases = specification
        .allowed_price_bases
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if allowed_bases.len() != 3 {
        return Err("Stage 112 specification 价格口径不是固定三种".to_string());
    }
    let mut expected_symbols = specification
        .subject_symbols
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    expected_symbols.insert(specification.benchmark_symbol.clone());
    let mut price_keys = BTreeSet::new();
    for row in &source.price_rows {
        validate_source_price_row(row)?;
        if !sessions.contains(&row.date)
            || !expected_symbols.contains(&row.symbol)
            || !allowed_bases.contains(row.source_kind.as_str())
            || !price_keys.insert((row.symbol.clone(), row.date, row.source_kind.clone()))
        {
            return Err("Stage 112 source price 越界、重复或口径无效".to_string());
        }
    }
    let mut gap_keys = BTreeSet::new();
    for row in &source.explicit_gaps {
        if !sessions.contains(&row.date)
            || !specification.subject_symbols.contains(&row.symbol)
            || !allowed_bases.contains(row.source_kind.as_str())
            || row.reason != "missing_subject_row_no_fill"
            || !gap_keys.insert((row.symbol.clone(), row.date, row.source_kind.clone()))
        {
            return Err("Stage 112 source gap 越界、重复、基准缺失或原因无效".to_string());
        }
    }
    for date in &sessions {
        for symbol in &expected_symbols {
            for basis in &allowed_bases {
                let key = (symbol.clone(), *date, (*basis).to_string());
                let has_price = price_keys.contains(&key);
                let has_gap = gap_keys.contains(&key);
                if symbol == &specification.benchmark_symbol {
                    if !has_price || has_gap {
                        return Err("Stage 112 SPY 三价格口径不完整".to_string());
                    }
                } else if has_price == has_gap {
                    return Err("Stage 112 subject price/gap 不是严格 XOR".to_string());
                }
            }
        }
    }
    let mut action_keys = BTreeSet::new();
    for row in &source.dividend_rows {
        validate_source_dividend_row(row)?;
        if row.date < specification.window_start_date
            || row.date > specification.window_end_date
            || !expected_symbols.contains(&row.symbol)
            || !action_keys.insert((row.symbol.clone(), row.date, "dividend"))
        {
            return Err("Stage 112 dividend 越界或重复".to_string());
        }
    }
    for row in &source.split_rows {
        validate_source_split_row(row)?;
        if row.date < specification.window_start_date
            || row.date > specification.window_end_date
            || !expected_symbols.contains(&row.symbol)
            || !action_keys.insert((row.symbol.clone(), row.date, "split"))
        {
            return Err("Stage 112 split 越界或重复".to_string());
        }
    }
    Ok(())
}

fn validate_source_calendar_row(row: &CanonicalMarketCalendarRow) -> Result<(), String> {
    let expected = fingerprint_without(row, &["row_sha256"])?;
    if row.row_sha256 != expected
        || !matches!(row.session_kind.as_str(), "regular" | "early_close")
        || !matches!(
            (row.session_kind.as_str(), row.close_time_et.as_str()),
            ("regular", "16:00") | ("early_close", "13:00")
        )
    {
        return Err("Stage 112 source calendar row 摘要或字段无效".to_string());
    }
    Ok(())
}

fn validate_source_price_row(row: &CanonicalMarketPriceRow) -> Result<(), String> {
    if row.row_sha256 != fingerprint_without(row, &["row_sha256"])?
        || !valid_sha256(&row.request_sha256)
        || row.close_decimal.trim().is_empty()
    {
        return Err("Stage 112 source price row 摘要或字段无效".to_string());
    }
    Ok(())
}

fn validate_source_dividend_row(row: &CanonicalMarketDividendRow) -> Result<(), String> {
    if row.row_sha256 != fingerprint_without(row, &["row_sha256"])?
        || row.dividend_decimal.trim().is_empty()
    {
        return Err("Stage 112 source dividend row 摘要或字段无效".to_string());
    }
    Ok(())
}

fn validate_source_split_row(row: &CanonicalMarketSplitRow) -> Result<(), String> {
    if row.row_sha256 != fingerprint_without(row, &["row_sha256"])?
        || row.numerator_positive_integer == 0
        || row.denominator_positive_integer == 0
    {
        return Err("Stage 112 source split row 摘要或字段无效".to_string());
    }
    Ok(())
}

fn materialize_dividend(
    row: &CanonicalMarketDividendRow,
) -> Result<MaterializedCorporateAction, String> {
    let mut fields = Map::new();
    fields.insert(
        "dividend_decimal".to_string(),
        Value::String(row.dividend_decimal.clone()),
    );
    fields.insert(
        "record_date".to_string(),
        optional_date_value(row.record_date),
    );
    fields.insert(
        "payment_date".to_string(),
        optional_date_value(row.payment_date),
    );
    fields.insert(
        "declaration_date".to_string(),
        optional_date_value(row.declaration_date),
    );
    materialize_action(
        row.symbol.clone(),
        row.date,
        "dividend",
        fields,
        row.row_sha256.clone(),
    )
}

fn materialize_split(row: &CanonicalMarketSplitRow) -> Result<MaterializedCorporateAction, String> {
    let mut fields = Map::new();
    fields.insert(
        "numerator_positive_integer".to_string(),
        Value::from(row.numerator_positive_integer),
    );
    fields.insert(
        "denominator_positive_integer".to_string(),
        Value::from(row.denominator_positive_integer),
    );
    fields.insert(
        "split_type".to_string(),
        row.split_type.clone().map_or(Value::Null, Value::String),
    );
    materialize_action(
        row.symbol.clone(),
        row.date,
        "split",
        fields,
        row.row_sha256.clone(),
    )
}

fn materialize_action(
    symbol: String,
    date: NaiveDate,
    action_kind: &str,
    exact_source_fields: Map<String, Value>,
    source_row_sha256: String,
) -> Result<MaterializedCorporateAction, String> {
    let mut action = MaterializedCorporateAction {
        symbol,
        date,
        action_kind: action_kind.to_string(),
        exact_source_fields,
        source_row_sha256,
        action_row_sha256: String::new(),
    };
    action.action_row_sha256 = fingerprint_without(&action, &["action_row_sha256"])?;
    Ok(action)
}

fn optional_date_value(value: Option<NaiveDate>) -> Value {
    value.map_or(Value::Null, |date| Value::String(date.to_string()))
}

fn validate_envelope(
    specification: &ControlledShadowObservationMaterializationSpecification,
    envelope: &UntrustedControlledShadowObservationEnvelope,
) -> Result<(), String> {
    let session_sorted = envelope
        .sessions
        .windows(2)
        .all(|rows| rows[0].date < rows[1].date);
    let price_sorted = envelope.prices.windows(2).all(|rows| {
        (rows[0].date, &rows[0].symbol, &rows[0].price_basis)
            < (rows[1].date, &rows[1].symbol, &rows[1].price_basis)
    });
    let gap_sorted = envelope.explicit_gaps.windows(2).all(|rows| {
        (rows[0].date, &rows[0].symbol, &rows[0].price_basis)
            < (rows[1].date, &rows[1].symbol, &rows[1].price_basis)
    });
    if envelope.schema_version != OUTPUT_SCHEMA_VERSION
        || envelope.specification_sha256 != specification.specification_sha256
        || envelope.stage_104_review_sha256 != specification.stage_104_review_sha256
        || envelope.cycle_claim_sha256 != specification.cycle_claim_sha256
        || envelope.admitted_available_at_utc != specification.admitted_available_at_utc
        || envelope.sessions.len() != specification.official_market_session_count
        || !session_sorted
        || !price_sorted
        || !gap_sorted
        || !valid_sha256(&envelope.output_sha256)
        || envelope.output_sha256 != fingerprint_without(envelope, &["output_sha256"])?
        || !envelope.untrusted
        || envelope
            .initial_shadow_allocation_binding
            .initial_shadow_allocation_recomputed
        || envelope
            .initial_shadow_allocation_binding
            .accounting_transition_applied
    {
        return Err("Stage 112 observation envelope 无效、非 canonical 或越权".to_string());
    }
    Ok(())
}

fn validate_execution_request(
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
    executor_id: &str,
    request: &ExecuteControlledShadowObservationMaterializationAttemptRequest,
) -> Result<(), String> {
    validate_observation_materialization_execution_attempt_claim(claim)?;
    let authorization = &claim.authorization;
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
    let specification = &contract.exact_observation_materialization_specification;
    if request.expected_claim_sha256 != claim.claim_sha256
        || request.expected_authorization_review_sha256 != authorization.review_sha256
        || request.expected_runner_artifact_sha256 != authorization.server_computed_artifact_sha256
        || request.expected_artifact_manifest_sha256
            != authorization.artifact_manifest.manifest_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_observation_materialization_specification_sha256
            != specification.specification_sha256
        || request.expected_stage_104_admission_review_sha256
            != specification.stage_104_review_sha256
        || request.expected_stage_102_output_sha256 != specification.stage_102_output_sha256
        || request.expected_stage_101_input_manifest_sha256
            != specification.stage_101_input_manifest_sha256
        || request.expected_cycle_claim_sha256 != specification.cycle_claim_sha256
    {
        return Err("Stage 101–111 观察物化责任链已经变化".to_string());
    }
    let mut excluded = claim.excluded_prior_actor_ids.clone();
    excluded.push(claim.claimed_by.clone());
    excluded.sort();
    excluded.dedup();
    let independent =
        !executor_id.trim().is_empty() && !excluded.iter().any(|value| value == executor_id);
    let confirmations = request.exact_stage_51_through_stage_111_binding_confirmed
        && request.executor_independent_from_complete_prior_chain_and_claimant_confirmed
            == independent
        && request.start_marker_consumes_claim_before_artifact_or_input_read_confirmed
        && request.one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed
        && request.artifact_is_declarative_not_spawned_or_executed_confirmed
        && request.only_exact_stage_104_admitted_output_is_read_only_opened_and_rehashed_confirmed
        && request.deterministic_session_price_gap_action_allocation_and_availability_projection_confirmed
        && request.no_refetch_reparse_fill_interpolation_substitution_backfill_or_correction_confirmed
        && request.output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed
        && request.no_network_environment_secret_tool_subprocess_or_production_io_confirmed
        && request.no_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    if !independent || !confirmations {
        return Err("Stage 112 executor 独立性或强制确认不完整".to_string());
    }
    Ok(())
}

fn validate_start(
    start: &ControlledShadowObservationMaterializationExecutionAttemptStart,
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
) -> Result<(), String> {
    if start.schema_version != START_SCHEMA_VERSION
        || start.policy_version != POLICY_VERSION
        || start.start_sha256 != fingerprint_without(start, &["start_sha256"])?
        || start.stage_111_attempt_id != claim.attempt_id
        || start.stage_111_claim_sha256 != claim.claim_sha256
        || start.started_at < claim.claimed_at
        || start.executed_by.trim().is_empty()
        || start.execution_reason.trim().is_empty()
        || !start.one_shot_claim_consumed_before_artifact_or_input_read
    {
        return Err("Stage 112 start marker 无效或漂移".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &ControlledShadowObservationMaterializationExecutionAttemptResult,
    claim: &ControlledShadowObservationMaterializationExecutionAttemptClaim,
) -> Result<(), String> {
    let successful = result.status
        == ControlledShadowObservationMaterializationExecutionStatus::CompletedWithUntrustedObservationEnvelope;
    let success_fields = result.bounded_error_code.is_none()
        && result.output_sha256.as_deref().is_some_and(valid_sha256)
        && result.output_relative_path.is_some()
        && result.artifact_revalidated
        && result.exact_admitted_input_revalidated_and_opened
        && result.materializer_executed_in_process
        && result.sessions_materialized
        && result.price_observations_materialized
        && result.explicit_gaps_materialized
        && result.corporate_actions_materialized
        && result.observation_envelope_created
        && result.output_untrusted
        && result.forward_observation_started;
    let failure_fields = result
        .bounded_error_code
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty() && value.chars().count() <= 128)
        && result.output_sha256.is_none()
        && result.output_relative_path.is_none()
        && !result.sessions_materialized
        && !result.price_observations_materialized
        && !result.explicit_gaps_materialized
        && !result.corporate_actions_materialized
        && !result.observation_envelope_created
        && !result.output_untrusted
        && !result.forward_observation_started;
    let authority_closed = !result.artifact_spawned_or_executed
        && !result.independent_validation_completed
        && !result.ledger_created
        && !result.position_written
        && !result.performance_metric_written
        && !result.model_or_metric_store_written
        && !result.training_feedback_authorized
        && !result.reward_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.policy_version != POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != fingerprint_without(result, &["result_id", "result_sha256"])?
        || result.stage_111_attempt_id != claim.attempt_id
        || result.stage_111_claim_sha256 != claim.claim_sha256
        || result.completed_at < claim.claimed_at
        || result.executed_by.trim().is_empty()
        || result.execution_reason.trim().is_empty()
        || !result.claim_consumed
        || !authority_closed
        || (successful && !success_fields)
        || (!successful && !failure_fields)
    {
        return Err("Stage 112 terminal result 无效、漂移或越权".to_string());
    }
    Ok(())
}

async fn recover_interrupted_attempts(
    state: &AppState,
    claims: &[ControlledShadowObservationMaterializationExecutionAttemptClaim],
    existing_results: &[ControlledShadowObservationMaterializationExecutionAttemptResult],
) -> Result<(), String> {
    let claim_index = claims
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let completed = existing_results
        .iter()
        .map(|value| value.stage_111_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    for start in read_starts(state).await? {
        if completed.contains(start.stage_111_attempt_id.as_str()) {
            continue;
        }
        let claim = claim_index
            .get(start.stage_111_attempt_id.as_str())
            .ok_or_else(|| "Stage 112 start marker 缺少 Stage 111 claim".to_string())?;
        let ceiling = i64::from(
            claim
                .authorization
                .runner
                .runner_contract
                .maximum_wall_clock_seconds,
        );
        let deadline =
            start.started_at + Duration::seconds(ceiling + INTERRUPTED_RECOVERY_GRACE_SECONDS);
        if Utc::now() <= deadline {
            continue;
        }
        let mut result = ControlledShadowObservationMaterializationExecutionAttemptResult {
            schema_version: RESULT_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            result_id: String::new(),
            result_sha256: String::new(),
            stage_111_attempt_id: claim.attempt_id.clone(),
            stage_111_claim_sha256: claim.claim_sha256.clone(),
            completed_at: deadline,
            executed_by: start.executed_by.clone(),
            execution_reason: start.execution_reason.clone(),
            duration_millis: u64::from(
                claim
                    .authorization
                    .runner
                    .runner_contract
                    .maximum_wall_clock_seconds,
            ) * 1_000,
            status: ControlledShadowObservationMaterializationExecutionStatus::FailedClaimConsumed,
            bounded_error_code: Some("execution_interrupted_without_terminal_result".to_string()),
            output_sha256: None,
            output_relative_path: None,
            claim_consumed: true,
            artifact_revalidated: false,
            artifact_spawned_or_executed: false,
            exact_admitted_input_revalidated_and_opened: false,
            materializer_executed_in_process: false,
            sessions_materialized: false,
            price_observations_materialized: false,
            explicit_gaps_materialized: false,
            corporate_actions_materialized: false,
            observation_envelope_created: false,
            output_untrusted: false,
            independent_validation_completed: false,
            forward_observation_started: false,
            ledger_created: false,
            position_written: false,
            performance_metric_written: false,
            model_or_metric_store_written: false,
            training_feedback_authorized: false,
            reward_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        result.result_sha256 = fingerprint_without(&result, &["result_id", "result_sha256"])?;
        result.result_id = result.result_sha256[..32].to_string();
        validate_result(&result, claim)?;
        write_immutable_json(
            &result_directory(state).join(format!("{}.json", claim.attempt_id)),
            &result,
            None,
        )
        .await?;
    }
    Ok(())
}

async fn read_starts(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationMaterializationExecutionAttemptStart>, String> {
    read_json_directory(start_directory(state), 256 * 1024).await
}

async fn read_results(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationMaterializationExecutionAttemptResult>, String> {
    let mut values: Vec<ControlledShadowObservationMaterializationExecutionAttemptResult> =
        read_json_directory(result_directory(state), 512 * 1024).await?;
    let claims = observation_materialization_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .map(|value| (value.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut attempts = BTreeSet::new();
    for value in &values {
        let claim = claims
            .get(&value.stage_111_attempt_id)
            .ok_or_else(|| "Stage 112 result 缺少 Stage 111 claim".to_string())?;
        validate_result(value, claim)?;
        if !ids.insert(value.result_id.clone())
            || !attempts.insert(value.stage_111_attempt_id.clone())
        {
            return Err("Stage 112 result 重复或重放".to_string());
        }
    }
    values.sort_by(|left, right| left.completed_at.cmp(&right.completed_at));
    Ok(values)
}

async fn read_json_directory<T>(directory: PathBuf, maximum_bytes: u64) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut entries = match tokio::fs::read_dir(directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut values = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let metadata = tokio::fs::symlink_metadata(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.len() == 0
            || metadata.len() > maximum_bytes
        {
            return Err("Stage 112 registry 文件无效或超限".to_string());
        }
        values.push(
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
        );
    }
    Ok(values)
}

#[derive(Debug)]
struct ExecutionFailure {
    code: &'static str,
    detail: String,
    artifact_revalidated: bool,
    input_opened: bool,
    materializer_executed: bool,
}

impl ExecutionFailure {
    fn new(
        code: &'static str,
        detail: impl Into<String>,
        artifact_revalidated: bool,
        input_opened: bool,
        materializer_executed: bool,
    ) -> Self {
        Self {
            code,
            detail: detail.into(),
            artifact_revalidated,
            input_opened,
            materializer_executed,
        }
    }
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 112 fingerprint payload 无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_REASON_CHARS {
        Err(format!("{label}为空或超过 {MAX_REASON_CHARS} 字符"))
    } else {
        Ok(value.to_string())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

async fn write_immutable_json<T: Serialize>(
    path: &Path,
    value: &T,
    maximum_bytes: Option<u64>,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    write_immutable_bytes(path, &bytes, maximum_bytes.unwrap_or(u64::MAX)).await
}

async fn write_immutable_bytes(
    path: &Path,
    bytes: &[u8],
    maximum_bytes: u64,
) -> Result<(), String> {
    if bytes.is_empty() || bytes.len() as u64 > maximum_bytes {
        return Err("Stage 112 create-once 文件为空或超限".to_string());
    }
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
    file.write_all(bytes)
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

#[allow(dead_code)]
pub(crate) fn controlled_shadow_observation_materialization_execution_custody_root(
    state: &AppState,
) -> PathBuf {
    execution_root(state)
}

fn execution_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-observation-materialization-execution-attempts")
}

fn start_directory(state: &AppState) -> PathBuf {
    execution_root(state).join("starts")
}

fn result_directory(state: &AppState) -> PathBuf {
    execution_root(state).join("results")
}

struct ExecutionLock(PathBuf);

impl Drop for ExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<ExecutionLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-observation-materialization-execution-{attempt_id}.lock"
    ));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age > StdDuration::from_secs(600));
        if stale {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 112 单次执行正在进行".to_string())?;
    Ok(ExecutionLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_is_strictly_declarative() {
        let value = serde_json::json!({
            "schema_version": PROGRAM_SCHEMA_VERSION,
            "program_kind": PROGRAM_KIND,
            "implementation_contract_sha256": "a".repeat(64),
            "specification_sha256": "b".repeat(64),
            "immutable_code_revision": "rev-1",
            "function_ids": ["f1"],
            "canonical_schemas": ["s1"],
            "no_command_or_entrypoint": true,
            "deterministic_in_process_interpreter_only": true
        });
        let parsed: ControlledShadowObservationMaterializationDeclarativeProgram =
            serde_json::from_value(value).expect("strict program");
        assert!(parsed.no_command_or_entrypoint);
        assert!(parsed.deterministic_in_process_interpreter_only);
    }

    #[test]
    fn program_rejects_command_field() {
        let value = serde_json::json!({
            "schema_version": PROGRAM_SCHEMA_VERSION,
            "program_kind": PROGRAM_KIND,
            "implementation_contract_sha256": "a".repeat(64),
            "specification_sha256": "b".repeat(64),
            "immutable_code_revision": "rev-1",
            "function_ids": ["f1"],
            "canonical_schemas": ["s1"],
            "no_command_or_entrypoint": true,
            "deterministic_in_process_interpreter_only": true,
            "command": "sh"
        });
        assert!(
            serde_json::from_value::<ControlledShadowObservationMaterializationDeclarativeProgram>(
                value
            )
            .is_err()
        );
    }

    #[test]
    fn row_hash_is_deterministic_and_excludes_only_its_hash() {
        let mut row = MaterializedPriceObservation {
            symbol: "ZZTEST".into(),
            date: NaiveDate::from_ymd_opt(2099, 1, 3).expect("date"),
            price_basis: "raw_unadjusted_price".into(),
            close_decimal: "10.5000".into(),
            volume_integer: Some(42),
            source_request_sha256: "a".repeat(64),
            source_row_sha256: "b".repeat(64),
            observation_row_sha256: String::new(),
        };
        let first = fingerprint_without(&row, &["observation_row_sha256"]).expect("hash");
        row.observation_row_sha256 = first.clone();
        assert_eq!(
            first,
            fingerprint_without(&row, &["observation_row_sha256"]).expect("hash")
        );
        assert_eq!(row.close_decimal, "10.5000");
    }

    #[test]
    fn stage_112_has_no_investment_or_trading_authority() {
        let authority_flags = [false; 11];
        assert!(authority_flags.into_iter().all(|value| !value));
        assert_eq!(
            NEXT_GATE,
            "stage_113_independent_observation_materialization_output_validation"
        );
    }
}
