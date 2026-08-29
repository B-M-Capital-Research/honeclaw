//! Stage 122 one-shot projection of admitted observations into non-financial notice candidates.
//!
//! The reviewed artifact is a strict declarative binding and is never spawned. A create-once start
//! marker is persisted before either the artifact or the exact Stage 114 evidence is opened. With
//! no separately admitted opening portfolio snapshot, the only successful output is an untrusted,
//! content-addressed candidate envelope. It does not create a ledger or financial state and must be
//! independently validated by Stage 123 before any later use.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_evidence_admission_reviews::{
    AdmittedControlledShadowObservationEvidence,
    admitted_controlled_shadow_observation_evidence_for_stage_115,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_execution_attempt_claims::{
    ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    observation_ledger_transition_execution_attempt_claims_for_execution,
    validate_observation_ledger_transition_execution_attempt_claim,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_first_execution_authorizations::read_revalidated_observation_ledger_transition_artifact_for_execution;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_implementations::ControlledShadowObservationLedgerTransitionImplementationContract;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_specifications::ControlledShadowObservationLedgerTransitionSpecification;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-execution-attempt-registry-v1";
const PROGRAM_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-declarative-program-v1";
const OUTPUT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-untrusted-non-financial-observation-notice-candidate-envelope-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-execution-attempt-result-v1";
const START_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-execution-attempt-start-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-ledger-transition-execution-v1-one-shot-in-process-no-financial-state";
const PROGRAM_KIND: &str =
    "hone_trusted_deterministic_non_financial_observation_notice_projection_bindings";
const NEXT_GATE: &str =
    "stage_123_independent_non_financial_observation_notice_candidate_validation";
const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;
const MAX_REASON_CHARS: usize = 4_000;
const INTERRUPTED_RECOVERY_GRACE_SECONDS: i64 = 1;

#[derive(Debug, Deserialize)]
pub(crate) struct ExecuteControlledShadowObservationLedgerTransitionAttemptRequest {
    expected_claim_sha256: String,
    expected_authorization_review_sha256: String,
    expected_runner_contract_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_artifact_manifest_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_observation_ledger_transition_specification_sha256: String,
    expected_stage_114_admission_review_sha256: String,
    expected_stage_113_validation_sha256: String,
    expected_stage_112_result_sha256: String,
    expected_stage_112_output_sha256: String,
    expected_stage_111_claim_sha256: String,
    execution_reason: String,
    exact_stage_51_through_stage_121_binding_confirmed: bool,
    executor_independent_from_complete_prior_chain_and_claimant_confirmed: bool,
    start_marker_consumes_claim_before_artifact_or_input_read_confirmed: bool,
    one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: bool,
    artifact_is_declarative_not_spawned_or_executed_confirmed: bool,
    only_exact_stage_114_admitted_output_is_read_only_reopened_and_rehashed_confirmed: bool,
    opening_portfolio_snapshot_absent_no_default_notional_cash_positions_or_shares_confirmed: bool,
    non_financial_notice_allowlist_only_and_no_ledger_event_or_financial_posting_confirmed: bool,
    raw_security_close_and_dividend_adjusted_spy_benchmark_separated_confirmed: bool,
    explicit_gap_blocks_nav_and_corporate_actions_remain_pending_validation_confirmed: bool,
    output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed:
        bool,
    no_network_environment_secret_tool_subprocess_or_production_io_confirmed: bool,
    no_authoritative_financial_state_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ControlledShadowObservationLedgerTransitionDeclarativeProgram {
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
pub(crate) struct CanonicalNonFinancialObservationNoticeCandidate {
    pub notice_id: String,
    pub notice_sha256: String,
    pub event_type: String,
    pub effective_date: NaiveDate,
    pub available_at_utc: DateTime<Utc>,
    pub stage_114_review_sha256: String,
    pub stage_112_output_sha256: String,
    pub source_row_sha256: Option<String>,
    pub symbol: Option<String>,
    pub exact_decimal_fields: BTreeMap<String, String>,
    pub evidence_attributes: BTreeMap<String, String>,
    pub non_financial: bool,
    pub untrusted: bool,
    pub authoritative: bool,
    pub financial_posting_created: bool,
    pub ledger_event_written: bool,
    pub supersedes_notice_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UntrustedNonFinancialObservationNoticeCandidateEnvelope {
    pub schema_version: String,
    pub specification_sha256: String,
    pub stage_114_review_sha256: String,
    pub stage_113_validation_sha256: String,
    pub stage_112_result_sha256: String,
    pub stage_112_output_sha256: String,
    pub stage_111_claim_sha256: String,
    pub admitted_available_at_utc: DateTime<Utc>,
    pub opening_portfolio_snapshot_admitted: bool,
    pub financial_event_allowlist: Vec<String>,
    pub notices: Vec<CanonicalNonFinancialObservationNoticeCandidate>,
    pub candidate_sha256: String,
    pub create_once: bool,
    pub untrusted: bool,
    pub independent_validation_completed: bool,
    pub ledger_created: bool,
    pub authoritative_financial_state_created: bool,
    pub nav_or_performance_calculated: bool,
    pub order_intent_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationLedgerTransitionExecutionStatus {
    CompletedWithUntrustedNonFinancialNoticeCandidate,
    FailedClaimConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionExecutionAttemptResult {
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub stage_121_attempt_id: String,
    pub stage_121_claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub executed_by: String,
    pub execution_reason: String,
    pub duration_millis: u64,
    pub status: ControlledShadowObservationLedgerTransitionExecutionStatus,
    pub bounded_error_code: Option<String>,
    pub candidate_sha256: Option<String>,
    pub candidate_relative_path: Option<String>,
    pub notice_candidate_count: usize,
    pub claim_consumed: bool,
    pub artifact_revalidated: bool,
    pub artifact_spawned_or_executed: bool,
    pub exact_stage_114_input_revalidated_and_opened: bool,
    pub transition_projector_executed_in_process: bool,
    pub candidate_envelope_created: bool,
    pub output_untrusted: bool,
    pub independent_validation_completed: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_created: bool,
    pub ledger_event_written: bool,
    pub financial_posting_created: bool,
    pub position_written: bool,
    pub cash_written: bool,
    pub nav_or_performance_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ControlledShadowObservationLedgerTransitionExecutionAttemptStart {
    schema_version: String,
    policy_version: String,
    start_sha256: String,
    stage_121_attempt_id: String,
    stage_121_claim_sha256: String,
    started_at: DateTime<Utc>,
    executed_by: String,
    execution_reason: String,
    one_shot_claim_consumed_before_artifact_or_input_read: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub execution_endpoint_available: bool,
    pub pending_claims: Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptClaim>,
    pub results: Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptResult>,
    pub pending_claim_count: usize,
    pub terminal_result_count: usize,
    pub successful_untrusted_candidate_count: usize,
    pub failed_consumed_claim_count: usize,
    pub next_gate: String,
    pub arbitrary_artifact_execution_allowed: bool,
    pub outbound_network_allowed: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub independent_validation_completed: bool,
    pub non_financial_notice_candidate_created: bool,
    pub ledger_created: bool,
    pub ledger_event_written: bool,
    pub financial_posting_created: bool,
    pub nav_or_performance_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationLedgerTransitionExecutionReadinessSummary {
    pub pending_claim_count: usize,
    pub terminal_result_count: usize,
    pub successful_untrusted_candidate_count: usize,
    pub failed_consumed_claim_count: usize,
}

pub(crate) async fn handle_get_controlled_shadow_observation_ledger_transition_execution_attempts(
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
            warn!(%error, "observation-ledger-transition execution registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察到账本转换执行表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_execute_controlled_shadow_observation_ledger_transition_attempt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ExecuteControlledShadowObservationLedgerTransitionAttemptRequest>,
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
                "执行结果已落盘，但刷新观察到账本转换执行表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_observation_ledger_transition_execution_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionExecutionReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowObservationLedgerTransitionExecutionReadinessSummary {
            pending_claim_count: registry.pending_claim_count,
            terminal_result_count: registry.terminal_result_count,
            successful_untrusted_candidate_count: registry.successful_untrusted_candidate_count,
            failed_consumed_claim_count: registry.failed_consumed_claim_count,
        },
    )
}

pub(crate) async fn started_or_terminal_observation_ledger_transition_claim_attempt_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    let mut ids = read_starts(state)
        .await?
        .into_iter()
        .map(|value| value.stage_121_attempt_id)
        .collect::<BTreeSet<_>>();
    ids.extend(
        read_results(state)
            .await?
            .into_iter()
            .map(|value| value.stage_121_attempt_id),
    );
    Ok(ids)
}

pub(crate) async fn observation_ledger_transition_execution_results_for_independent_validation(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptResult>, String> {
    read_results(state).await
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry, String> {
    let claims =
        observation_ledger_transition_execution_attempt_claims_for_execution(state).await?;
    let mut results = read_results(state).await?;
    recover_interrupted_attempts(state, &claims, &results).await?;
    results = read_results(state).await?;
    let starts = read_starts(state).await?;
    let completed = results
        .iter()
        .map(|value| value.stage_121_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    let started = starts
        .iter()
        .map(|value| value.stage_121_attempt_id.as_str())
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
                == ControlledShadowObservationLedgerTransitionExecutionStatus::CompletedWithUntrustedNonFinancialNoticeCandidate
        })
        .count();
    let failed = results.len().saturating_sub(successful);
    Ok(ControlledShadowObservationLedgerTransitionExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        execution_endpoint_available: true,
        pending_claim_count: pending_claims.len(),
        terminal_result_count: results.len(),
        successful_untrusted_candidate_count: successful,
        failed_consumed_claim_count: failed,
        pending_claims,
        results,
        next_gate: NEXT_GATE.to_string(),
        arbitrary_artifact_execution_allowed: false,
        outbound_network_allowed: false,
        opening_portfolio_snapshot_admitted: false,
        independent_validation_completed: false,
        non_financial_notice_candidate_created: successful > 0,
        ledger_created: false,
        ledger_event_written: false,
        financial_posting_created: false,
        nav_or_performance_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 122 对一条 Stage 121 声明最多执行一次：先写 create-once start marker 永久消费声明，再重新核验声明式工件和 exact Stage 114 admitted Stage 112 observation envelope。由于没有单独准入的期初组合快照，成功也只创建非可信、非财务的观察通知候选；不写 ledger event、现金、仓位、净值或绩效。失败或中断不可重试，Stage 123 独立验证前不得下游使用。".to_string(),
    })
}

async fn execute_once(
    state: &AppState,
    executor_id: &str,
    attempt_id: &str,
    request: ExecuteControlledShadowObservationLedgerTransitionAttemptRequest,
) -> Result<ControlledShadowObservationLedgerTransitionExecutionAttemptResult, String> {
    if !valid_id(attempt_id) {
        return Err("Stage 121 attempt ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_results(state)
        .await?
        .iter()
        .any(|value| value.stage_121_attempt_id == attempt_id)
    {
        return Err("这条 Stage 121 claim 已有终态结果，不能重试".to_string());
    }
    if read_starts(state)
        .await?
        .iter()
        .any(|value| value.stage_121_attempt_id == attempt_id)
    {
        return Err("这条 Stage 121 claim 已经开始过执行，不能重试".to_string());
    }
    let claim = observation_ledger_transition_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .find(|value| value.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有这条 Stage 121 claim".to_string())?;
    validate_execution_request(&claim, executor_id, &request)?;
    let reason = bounded_required(&request.execution_reason, "执行原因")?;
    let mut start = ControlledShadowObservationLedgerTransitionExecutionAttemptStart {
        schema_version: START_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        start_sha256: String::new(),
        stage_121_attempt_id: claim.attempt_id.clone(),
        stage_121_claim_sha256: claim.claim_sha256.clone(),
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
    let ceiling = u64::from(
        claim
            .authorization
            .runner
            .runner_contract
            .maximum_wall_clock_seconds,
    );
    let execution = tokio::time::timeout(
        StdDuration::from_secs(ceiling),
        execute_claim(state, &claim),
    )
    .await
    .unwrap_or_else(|_| {
        Err(ExecutionFailure::new(
            "execution_wall_clock_limit_exceeded",
            "Stage 122 wall-clock ceiling exceeded",
            true,
            true,
            true,
        ))
    });
    let duration_millis = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let (
        status,
        bounded_error_code,
        candidate_sha256,
        candidate_relative_path,
        notice_candidate_count,
        artifact_revalidated,
        input_opened,
        projector_executed,
    ) = match execution {
        Ok((sha256, relative_path, count)) => (
            ControlledShadowObservationLedgerTransitionExecutionStatus::CompletedWithUntrustedNonFinancialNoticeCandidate,
            None,
            Some(sha256),
            Some(relative_path),
            count,
            true,
            true,
            true,
        ),
        Err(failure) => {
            warn!(attempt_id, error = %failure.detail, "one-shot observation-ledger transition failed");
            (
                ControlledShadowObservationLedgerTransitionExecutionStatus::FailedClaimConsumed,
                Some(failure.code.to_string()),
                None,
                None,
                0,
                failure.artifact_revalidated,
                failure.input_opened,
                failure.projector_executed,
            )
        }
    };
    let successful = status
        == ControlledShadowObservationLedgerTransitionExecutionStatus::CompletedWithUntrustedNonFinancialNoticeCandidate;
    let mut result = ControlledShadowObservationLedgerTransitionExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        stage_121_attempt_id: claim.attempt_id.clone(),
        stage_121_claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        executed_by: executor_id.to_string(),
        execution_reason: reason,
        duration_millis,
        status,
        bounded_error_code,
        candidate_sha256,
        candidate_relative_path,
        notice_candidate_count,
        claim_consumed: true,
        artifact_revalidated,
        artifact_spawned_or_executed: false,
        exact_stage_114_input_revalidated_and_opened: input_opened,
        transition_projector_executed_in_process: projector_executed,
        candidate_envelope_created: successful,
        output_untrusted: successful,
        independent_validation_completed: false,
        opening_portfolio_snapshot_admitted: false,
        ledger_created: false,
        ledger_event_written: false,
        financial_posting_created: false,
        position_written: false,
        cash_written: false,
        nav_or_performance_written: false,
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
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
) -> Result<(String, String, usize), ExecutionFailure> {
    let artifact = read_revalidated_observation_ledger_transition_artifact_for_execution(
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
            "Stage 122 declarative artifact is empty or too large",
            true,
            false,
            false,
        ));
    }
    let program: ControlledShadowObservationLedgerTransitionDeclarativeProgram =
        serde_json::from_slice(&artifact).map_err(|error| {
            ExecutionFailure::new(
                "declarative_artifact_invalid",
                error.to_string(),
                true,
                false,
                false,
            )
        })?;
    let contract = exact_implementation_contract(claim);
    let specification = &contract.exact_observation_ledger_transition_specification;
    validate_program(&program, contract, specification).map_err(|error| {
        ExecutionFailure::new(
            "declarative_program_binding_invalid",
            error,
            true,
            false,
            false,
        )
    })?;

    let admitted = exact_admitted_evidence(state, specification)
        .await
        .map_err(|error| {
            ExecutionFailure::new(
                "exact_stage_114_input_revalidation_failed",
                error,
                true,
                true,
                false,
            )
        })?;
    let output = project_non_financial_notices(specification, &admitted).map_err(|error| {
        ExecutionFailure::new(
            "deterministic_notice_projection_failed",
            error,
            true,
            true,
            true,
        )
    })?;
    let bytes = serde_json::to_vec_pretty(&output).map_err(|error| {
        ExecutionFailure::new(
            "candidate_serialization_failed",
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
            "Stage 122 output exceeds frozen runner ceiling",
            true,
            true,
            true,
        ));
    }
    let relative_path = format!(
        "candidates/{}/{}.json",
        claim.attempt_id, output.candidate_sha256
    );
    write_immutable_bytes(
        &execution_root(state).join(&relative_path),
        &bytes,
        maximum_output_bytes,
    )
    .await
    .map_err(|error| ExecutionFailure::new("output_create_once_failed", error, true, true, true))?;
    Ok((output.candidate_sha256, relative_path, output.notices.len()))
}

fn exact_implementation_contract(
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
) -> &ControlledShadowObservationLedgerTransitionImplementationContract {
    &claim
        .authorization
        .runner
        .runner_contract
        .exact_approved_implementation_contract
}

async fn exact_admitted_evidence(
    state: &AppState,
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
) -> Result<AdmittedControlledShadowObservationEvidence, String> {
    let admitted = admitted_controlled_shadow_observation_evidence_for_stage_115(state)
        .await?
        .into_iter()
        .find(|value| {
            value.admission_review.review_sha256 == specification.stage_114_review_sha256
                && value.envelope.output_sha256 == specification.stage_112_output_sha256
        })
        .ok_or_else(|| "exact Stage 114 admitted observation evidence 不再可用".to_string())?;
    if admitted.candidate.validation.validation_sha256 != specification.stage_113_validation_sha256
        || admitted.candidate.result.result_sha256 != specification.stage_112_result_sha256
        || admitted.candidate.claim.claim_sha256 != specification.stage_111_claim_sha256
        || admitted.envelope.admitted_available_at_utc != specification.admitted_available_at_utc
        || admitted.envelope.output_sha256 != specification.stage_112_output_sha256
    {
        return Err("Stage 111–114 admitted observation evidence chain 漂移".to_string());
    }
    Ok(admitted)
}

fn validate_program(
    program: &ControlledShadowObservationLedgerTransitionDeclarativeProgram,
    contract: &ControlledShadowObservationLedgerTransitionImplementationContract,
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
) -> Result<(), String> {
    let expected_functions = vec![
        contract
            .current_source_binding_validation_function_id
            .clone(),
        contract
            .opening_portfolio_prerequisite_validation_function_id
            .clone(),
        contract
            .non_financial_observation_event_projection_function_id
            .clone(),
        contract
            .raw_close_accounting_and_adjusted_price_separation_function_id
            .clone(),
        contract.explicit_gap_nav_fail_closed_function_id.clone(),
        contract.corporate_action_notice_gating_function_id.clone(),
        contract
            .exact_decimal_idempotency_and_double_entry_function_id
            .clone(),
        contract
            .append_only_correction_and_conservative_availability_function_id
            .clone(),
    ];
    let expected_schemas = vec![
        contract.canonical_event_schema.clone(),
        contract.canonical_double_entry_posting_schema.clone(),
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
        return Err("Stage 122 declarative program 与已批准合同不一致".to_string());
    }
    Ok(())
}

fn project_non_financial_notices(
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
    admitted: &AdmittedControlledShadowObservationEvidence,
) -> Result<UntrustedNonFinancialObservationNoticeCandidateEnvelope, String> {
    validate_projection_boundary(specification, admitted)?;
    let envelope = &admitted.envelope;
    let available_at = specification.admitted_available_at_utc;
    let review_sha = &specification.stage_114_review_sha256;
    let output_sha = &specification.stage_112_output_sha256;
    let mut notices = Vec::new();

    notices.push(new_notice(
        "observation_evidence_anchor",
        specification.earliest_market_session_date,
        available_at,
        review_sha,
        output_sha,
        Some(output_sha.clone()),
        None,
        BTreeMap::new(),
        BTreeMap::from([(
            "evidence_kind".to_string(),
            "exact_stage_114_admitted_observation_envelope".to_string(),
        )]),
    )?);
    for session in &envelope.sessions {
        notices.push(new_notice(
            "market_session_fact",
            session.date,
            available_at,
            review_sha,
            output_sha,
            None,
            None,
            BTreeMap::new(),
            BTreeMap::from([
                ("session_kind".to_string(), session.session_kind.clone()),
                ("close_time_et".to_string(), session.close_time_et.clone()),
            ]),
        )?);
    }
    let subjects = specification
        .subject_symbols
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    for price in &envelope.prices {
        let event_type = if subjects.contains(price.symbol.as_str())
            && price.price_basis == "raw_unadjusted_price"
        {
            Some("security_raw_close_mark_fact")
        } else if price.symbol == specification.benchmark_symbol
            && price.price_basis == "dividend_adjusted_price"
        {
            Some("benchmark_total_return_mark_fact")
        } else {
            None
        };
        if let Some(event_type) = event_type {
            let mut decimals =
                BTreeMap::from([("close_decimal".to_string(), price.close_decimal.clone())]);
            if let Some(volume) = price.volume_integer {
                decimals.insert("volume_integer".to_string(), volume.to_string());
            }
            notices.push(new_notice(
                event_type,
                price.date,
                available_at,
                review_sha,
                output_sha,
                Some(price.source_row_sha256.clone()),
                Some(price.symbol.clone()),
                decimals,
                BTreeMap::from([("price_basis".to_string(), price.price_basis.clone())]),
            )?);
        }
    }
    for gap in &envelope.explicit_gaps {
        notices.push(new_notice(
            "explicit_data_gap_fact",
            gap.date,
            available_at,
            review_sha,
            output_sha,
            Some(gap.gap_row_sha256.clone()),
            Some(gap.symbol.clone()),
            BTreeMap::new(),
            BTreeMap::from([
                ("price_basis".to_string(), gap.price_basis.clone()),
                ("reason".to_string(), gap.reason.clone()),
                ("nav_status".to_string(), "blocked_no_fill".to_string()),
            ]),
        )?);
    }
    for action in &envelope.dividends {
        notices.push(new_notice(
            "dividend_notice_pending_validation",
            action.date,
            available_at,
            review_sha,
            output_sha,
            Some(action.action_row_sha256.clone()),
            Some(action.symbol.clone()),
            BTreeMap::new(),
            BTreeMap::from([(
                "posting_status".to_string(),
                "notice_only_requires_position_and_terms_validation".to_string(),
            )]),
        )?);
    }
    for action in &envelope.splits {
        notices.push(new_notice(
            "split_notice_pending_validation",
            action.date,
            available_at,
            review_sha,
            output_sha,
            Some(action.action_row_sha256.clone()),
            Some(action.symbol.clone()),
            BTreeMap::new(),
            BTreeMap::from([(
                "posting_status".to_string(),
                "notice_only_requires_position_and_terms_validation".to_string(),
            )]),
        )?);
    }
    notices.sort_by(|left, right| canonical_notice_key(left).cmp(&canonical_notice_key(right)));
    if notices
        .windows(2)
        .any(|values| values[0].notice_id == values[1].notice_id)
    {
        return Err("Stage 122 notice candidate idempotency collision".to_string());
    }
    let mut output = UntrustedNonFinancialObservationNoticeCandidateEnvelope {
        schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        specification_sha256: specification.specification_sha256.clone(),
        stage_114_review_sha256: specification.stage_114_review_sha256.clone(),
        stage_113_validation_sha256: specification.stage_113_validation_sha256.clone(),
        stage_112_result_sha256: specification.stage_112_result_sha256.clone(),
        stage_112_output_sha256: specification.stage_112_output_sha256.clone(),
        stage_111_claim_sha256: specification.stage_111_claim_sha256.clone(),
        admitted_available_at_utc: available_at,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist: Vec::new(),
        notices,
        candidate_sha256: String::new(),
        create_once: true,
        untrusted: true,
        independent_validation_completed: false,
        ledger_created: false,
        authoritative_financial_state_created: false,
        nav_or_performance_calculated: false,
        order_intent_created: false,
    };
    output.candidate_sha256 = fingerprint_without(&output, &["candidate_sha256"])?;
    validate_candidate_envelope(specification, &output)?;
    Ok(output)
}

fn validate_projection_boundary(
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
    admitted: &AdmittedControlledShadowObservationEvidence,
) -> Result<(), String> {
    let prerequisite = &specification.opening_portfolio_prerequisite;
    let rules = &specification.mapping_rules;
    let allowed = rules
        .non_financial_event_type_allowlist
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        "observation_evidence_anchor",
        "market_session_fact",
        "security_raw_close_mark_fact",
        "benchmark_total_return_mark_fact",
        "explicit_data_gap_fact",
        "dividend_notice_pending_validation",
        "split_notice_pending_validation",
    ]);
    if !specification.exact_stage_114_admitted_envelope_is_only_observation_input
        || admitted.admission_review.review_sha256 != specification.stage_114_review_sha256
        || admitted.envelope.output_sha256 != specification.stage_112_output_sha256
        || !prerequisite.separately_admitted_opening_portfolio_snapshot_required
        || prerequisite.current_opening_portfolio_snapshot_available
        || prerequisite.default_notional_allowed
        || prerequisite.default_cash_allowed
        || prerequisite.infer_positions_from_subject_symbols_allowed
        || prerequisite.infer_share_quantities_from_prices_or_target_weights_allowed
        || prerequisite.financial_posting_before_opening_snapshot_admission_allowed
        || !rules
            .financial_event_type_allowlist_before_opening_snapshot
            .is_empty()
        || allowed != expected
        || specification.financial_postings_currently_eligible
        || specification.nav_or_performance_currently_eligible
        || specification
            .authority_boundary
            .opening_portfolio_snapshot_present
        || specification.authority_boundary.ledger_created
        || specification.authority_boundary.ledger_event_write_allowed
        || specification.authority_boundary.position_write_allowed
        || specification.authority_boundary.cash_write_allowed
        || specification
            .authority_boundary
            .nav_or_performance_write_allowed
        || specification.authority_boundary.order_generation_allowed
        || specification.authority_boundary.broker_access_allowed
        || specification.authority_boundary.trading_allowed
    {
        return Err("Stage 122 opening-portfolio 或非财务权限边界漂移".to_string());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn new_notice(
    event_type: &str,
    effective_date: NaiveDate,
    available_at_utc: DateTime<Utc>,
    stage_114_review_sha256: &str,
    stage_112_output_sha256: &str,
    source_row_sha256: Option<String>,
    symbol: Option<String>,
    exact_decimal_fields: BTreeMap<String, String>,
    evidence_attributes: BTreeMap<String, String>,
) -> Result<CanonicalNonFinancialObservationNoticeCandidate, String> {
    let identity = serde_json::json!({
        "stage_114_review_sha256": stage_114_review_sha256,
        "stage_112_output_sha256": stage_112_output_sha256,
        "event_type": event_type,
        "source_row_sha256_or_envelope_sha256": source_row_sha256.as_deref().unwrap_or(stage_112_output_sha256),
        "symbol_or_none": symbol.as_deref().unwrap_or("none"),
        "effective_date": effective_date,
    });
    let notice_id = format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&identity).map_err(|error| error.to_string())?)
    );
    let mut notice = CanonicalNonFinancialObservationNoticeCandidate {
        notice_id,
        notice_sha256: String::new(),
        event_type: event_type.to_string(),
        effective_date,
        available_at_utc,
        stage_114_review_sha256: stage_114_review_sha256.to_string(),
        stage_112_output_sha256: stage_112_output_sha256.to_string(),
        source_row_sha256,
        symbol,
        exact_decimal_fields,
        evidence_attributes,
        non_financial: true,
        untrusted: true,
        authoritative: false,
        financial_posting_created: false,
        ledger_event_written: false,
        supersedes_notice_id: None,
    };
    notice.notice_sha256 = fingerprint_without(&notice, &["notice_sha256"])?;
    validate_notice(&notice)?;
    Ok(notice)
}

fn canonical_notice_key(
    notice: &CanonicalNonFinancialObservationNoticeCandidate,
) -> (NaiveDate, u8, String, String) {
    let precedence = match notice.event_type.as_str() {
        "observation_evidence_anchor" => 0,
        "market_session_fact" => 1,
        "dividend_notice_pending_validation" | "split_notice_pending_validation" => 2,
        "explicit_data_gap_fact" => 3,
        "security_raw_close_mark_fact" | "benchmark_total_return_mark_fact" => 4,
        _ => u8::MAX,
    };
    (
        notice.effective_date,
        precedence,
        notice.symbol.clone().unwrap_or_default(),
        notice.notice_id.clone(),
    )
}

fn validate_notice(notice: &CanonicalNonFinancialObservationNoticeCandidate) -> Result<(), String> {
    let allowed = BTreeSet::from([
        "observation_evidence_anchor",
        "market_session_fact",
        "security_raw_close_mark_fact",
        "benchmark_total_return_mark_fact",
        "explicit_data_gap_fact",
        "dividend_notice_pending_validation",
        "split_notice_pending_validation",
    ]);
    if !valid_sha256(&notice.notice_id)
        || !valid_sha256(&notice.notice_sha256)
        || notice.notice_sha256 != fingerprint_without(notice, &["notice_sha256"])?
        || !allowed.contains(notice.event_type.as_str())
        || !valid_sha256(&notice.stage_114_review_sha256)
        || !valid_sha256(&notice.stage_112_output_sha256)
        || notice
            .source_row_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256(value))
        || notice
            .exact_decimal_fields
            .values()
            .any(|value| !valid_exact_decimal(value))
        || !notice.non_financial
        || !notice.untrusted
        || notice.authoritative
        || notice.financial_posting_created
        || notice.ledger_event_written
        || notice.supersedes_notice_id.is_some()
    {
        return Err("Stage 122 non-financial notice candidate 无效或越权".to_string());
    }
    Ok(())
}

fn validate_candidate_envelope(
    specification: &ControlledShadowObservationLedgerTransitionSpecification,
    envelope: &UntrustedNonFinancialObservationNoticeCandidateEnvelope,
) -> Result<(), String> {
    let sorted = envelope
        .notices
        .windows(2)
        .all(|values| canonical_notice_key(&values[0]) < canonical_notice_key(&values[1]));
    for notice in &envelope.notices {
        validate_notice(notice)?;
    }
    if envelope.schema_version != OUTPUT_SCHEMA_VERSION
        || envelope.specification_sha256 != specification.specification_sha256
        || envelope.stage_114_review_sha256 != specification.stage_114_review_sha256
        || envelope.stage_113_validation_sha256 != specification.stage_113_validation_sha256
        || envelope.stage_112_result_sha256 != specification.stage_112_result_sha256
        || envelope.stage_112_output_sha256 != specification.stage_112_output_sha256
        || envelope.stage_111_claim_sha256 != specification.stage_111_claim_sha256
        || envelope.admitted_available_at_utc != specification.admitted_available_at_utc
        || envelope.notices.is_empty()
        || !sorted
        || !valid_sha256(&envelope.candidate_sha256)
        || envelope.candidate_sha256 != fingerprint_without(envelope, &["candidate_sha256"])?
        || envelope.opening_portfolio_snapshot_admitted
        || !envelope.financial_event_allowlist.is_empty()
        || !envelope.create_once
        || !envelope.untrusted
        || envelope.independent_validation_completed
        || envelope.ledger_created
        || envelope.authoritative_financial_state_created
        || envelope.nav_or_performance_calculated
        || envelope.order_intent_created
    {
        return Err("Stage 122 candidate envelope 无效、非 canonical 或越权".to_string());
    }
    Ok(())
}

fn validate_execution_request(
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
    executor_id: &str,
    request: &ExecuteControlledShadowObservationLedgerTransitionAttemptRequest,
) -> Result<(), String> {
    validate_observation_ledger_transition_execution_attempt_claim(claim)?;
    let authorization = &claim.authorization;
    let runner = &authorization.runner;
    let runner_contract = &runner.runner_contract;
    let implementation_contract = &runner_contract.exact_approved_implementation_contract;
    let specification = &implementation_contract.exact_observation_ledger_transition_specification;
    if request.expected_claim_sha256 != claim.claim_sha256
        || request.expected_authorization_review_sha256 != authorization.review_sha256
        || request.expected_runner_contract_sha256 != runner_contract.contract_sha256
        || request.expected_runner_artifact_sha256 != authorization.server_computed_artifact_sha256
        || request.expected_artifact_manifest_sha256
            != authorization.artifact_manifest.manifest_sha256
        || request.expected_implementation_contract_sha256
            != implementation_contract.contract_sha256
        || request.expected_observation_ledger_transition_specification_sha256
            != specification.specification_sha256
        || request.expected_stage_114_admission_review_sha256
            != specification.stage_114_review_sha256
        || request.expected_stage_113_validation_sha256 != specification.stage_113_validation_sha256
        || request.expected_stage_112_result_sha256 != specification.stage_112_result_sha256
        || request.expected_stage_112_output_sha256 != specification.stage_112_output_sha256
        || request.expected_stage_111_claim_sha256 != specification.stage_111_claim_sha256
    {
        return Err("Stage 111–121 观察到账本转换责任链已经变化".to_string());
    }
    let mut excluded = claim.excluded_prior_actor_ids.clone();
    excluded.push(claim.claimed_by.clone());
    excluded.sort();
    excluded.dedup();
    let independent =
        !executor_id.trim().is_empty() && !excluded.iter().any(|value| value == executor_id);
    let confirmations = request.exact_stage_51_through_stage_121_binding_confirmed
        && request.executor_independent_from_complete_prior_chain_and_claimant_confirmed
            == independent
        && request.start_marker_consumes_claim_before_artifact_or_input_read_confirmed
        && request.one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed
        && request.artifact_is_declarative_not_spawned_or_executed_confirmed
        && request.only_exact_stage_114_admitted_output_is_read_only_reopened_and_rehashed_confirmed
        && request.opening_portfolio_snapshot_absent_no_default_notional_cash_positions_or_shares_confirmed
        && request.non_financial_notice_allowlist_only_and_no_ledger_event_or_financial_posting_confirmed
        && request.raw_security_close_and_dividend_adjusted_spy_benchmark_separated_confirmed
        && request.explicit_gap_blocks_nav_and_corporate_actions_remain_pending_validation_confirmed
        && request.output_create_once_content_addressed_untrusted_and_requires_independent_validation_confirmed
        && request.no_network_environment_secret_tool_subprocess_or_production_io_confirmed
        && request.no_authoritative_financial_state_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    if !independent || !confirmations {
        return Err("Stage 122 executor 独立性或强制确认不完整".to_string());
    }
    Ok(())
}

fn validate_start(
    start: &ControlledShadowObservationLedgerTransitionExecutionAttemptStart,
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
) -> Result<(), String> {
    if start.schema_version != START_SCHEMA_VERSION
        || start.policy_version != POLICY_VERSION
        || start.start_sha256 != fingerprint_without(start, &["start_sha256"])?
        || start.stage_121_attempt_id != claim.attempt_id
        || start.stage_121_claim_sha256 != claim.claim_sha256
        || start.started_at < claim.claimed_at
        || start.executed_by.trim().is_empty()
        || start.execution_reason.trim().is_empty()
        || !start.one_shot_claim_consumed_before_artifact_or_input_read
    {
        return Err("Stage 122 start marker 无效或漂移".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &ControlledShadowObservationLedgerTransitionExecutionAttemptResult,
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
) -> Result<(), String> {
    let successful = result.status
        == ControlledShadowObservationLedgerTransitionExecutionStatus::CompletedWithUntrustedNonFinancialNoticeCandidate;
    let success_fields = result.bounded_error_code.is_none()
        && result.candidate_sha256.as_deref().is_some_and(valid_sha256)
        && result.candidate_relative_path.is_some()
        && result.notice_candidate_count > 0
        && result.artifact_revalidated
        && result.exact_stage_114_input_revalidated_and_opened
        && result.transition_projector_executed_in_process
        && result.candidate_envelope_created
        && result.output_untrusted;
    let failure_fields = result
        .bounded_error_code
        .as_ref()
        .is_some_and(|value| !value.trim().is_empty() && value.chars().count() <= 128)
        && result.candidate_sha256.is_none()
        && result.candidate_relative_path.is_none()
        && result.notice_candidate_count == 0
        && !result.candidate_envelope_created
        && !result.output_untrusted;
    let authority_closed = !result.artifact_spawned_or_executed
        && !result.independent_validation_completed
        && !result.opening_portfolio_snapshot_admitted
        && !result.ledger_created
        && !result.ledger_event_written
        && !result.financial_posting_created
        && !result.position_written
        && !result.cash_written
        && !result.nav_or_performance_written
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
        || result.stage_121_attempt_id != claim.attempt_id
        || result.stage_121_claim_sha256 != claim.claim_sha256
        || result.completed_at < claim.claimed_at
        || result.executed_by.trim().is_empty()
        || result.execution_reason.trim().is_empty()
        || !result.claim_consumed
        || !authority_closed
        || (successful && !success_fields)
        || (!successful && !failure_fields)
    {
        return Err("Stage 122 terminal result 无效、漂移或越权".to_string());
    }
    Ok(())
}

async fn recover_interrupted_attempts(
    state: &AppState,
    claims: &[ControlledShadowObservationLedgerTransitionExecutionAttemptClaim],
    existing_results: &[ControlledShadowObservationLedgerTransitionExecutionAttemptResult],
) -> Result<(), String> {
    let claim_index = claims
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let completed = existing_results
        .iter()
        .map(|value| value.stage_121_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    for start in read_starts(state).await? {
        if completed.contains(start.stage_121_attempt_id.as_str()) {
            continue;
        }
        let claim = claim_index
            .get(start.stage_121_attempt_id.as_str())
            .ok_or_else(|| "Stage 122 start marker 缺少 Stage 121 claim".to_string())?;
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
        let mut result = ControlledShadowObservationLedgerTransitionExecutionAttemptResult {
            schema_version: RESULT_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            result_id: String::new(),
            result_sha256: String::new(),
            stage_121_attempt_id: claim.attempt_id.clone(),
            stage_121_claim_sha256: claim.claim_sha256.clone(),
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
            status: ControlledShadowObservationLedgerTransitionExecutionStatus::FailedClaimConsumed,
            bounded_error_code: Some("execution_interrupted_without_terminal_result".to_string()),
            candidate_sha256: None,
            candidate_relative_path: None,
            notice_candidate_count: 0,
            claim_consumed: true,
            artifact_revalidated: false,
            artifact_spawned_or_executed: false,
            exact_stage_114_input_revalidated_and_opened: false,
            transition_projector_executed_in_process: false,
            candidate_envelope_created: false,
            output_untrusted: false,
            independent_validation_completed: false,
            opening_portfolio_snapshot_admitted: false,
            ledger_created: false,
            ledger_event_written: false,
            financial_posting_created: false,
            position_written: false,
            cash_written: false,
            nav_or_performance_written: false,
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
) -> Result<Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptStart>, String> {
    read_json_directory(start_directory(state), 256 * 1024).await
}

async fn read_results(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptResult>, String> {
    let mut values: Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptResult> =
        read_json_directory(result_directory(state), 512 * 1024).await?;
    let claims = observation_ledger_transition_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .map(|value| (value.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut attempts = BTreeSet::new();
    for value in &values {
        let claim = claims
            .get(&value.stage_121_attempt_id)
            .ok_or_else(|| "Stage 122 result 缺少 Stage 121 claim".to_string())?;
        validate_result(value, claim)?;
        if !ids.insert(value.result_id.clone())
            || !attempts.insert(value.stage_121_attempt_id.clone())
        {
            return Err("Stage 122 result 重复或重放".to_string());
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
            return Err("Stage 122 registry 文件无效或超限".to_string());
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
    projector_executed: bool,
}

impl ExecutionFailure {
    fn new(
        code: &'static str,
        detail: impl Into<String>,
        artifact_revalidated: bool,
        input_opened: bool,
        projector_executed: bool,
    ) -> Self {
        Self {
            code,
            detail: detail.into(),
            artifact_revalidated,
            input_opened,
            projector_executed,
        }
    }
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 122 fingerprint payload 无效".to_string())?;
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

fn valid_exact_decimal(value: &str) -> bool {
    if value.is_empty() || value.starts_with('+') || value.contains(['e', 'E']) {
        return false;
    }
    let value = value.strip_prefix('-').unwrap_or(value);
    let mut parts = value.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    !integer.is_empty()
        && integer.bytes().all(|byte| byte.is_ascii_digit())
        && fraction
            .is_none_or(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none()
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
        return Err("Stage 122 create-once 文件为空或超限".to_string());
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
pub(crate) fn controlled_shadow_observation_ledger_transition_execution_custody_root(
    state: &AppState,
) -> PathBuf {
    execution_root(state)
}

fn execution_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-observation-ledger-transition-execution-attempts")
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
    let directory = execution_root(state).join("locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{attempt_id}.lock"));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "这条 Stage 121 claim 正在执行或已被并发锁定".to_string())?;
    Ok(ExecutionLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(value: &str) -> String {
        format!("{:x}", Sha256::digest(value.as_bytes()))
    }

    #[test]
    fn notice_identity_is_deterministic_and_non_financial() {
        let first = new_notice(
            "security_raw_close_mark_fact",
            NaiveDate::from_ymd_opt(2099, 1, 2).unwrap(),
            DateTime::parse_from_rfc3339("2099-01-03T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            &hash("review"),
            &hash("output"),
            Some(hash("row")),
            Some("ZZTEST".to_string()),
            BTreeMap::from([("close_decimal".to_string(), "123.45".to_string())]),
            BTreeMap::from([(
                "price_basis".to_string(),
                "raw_unadjusted_price".to_string(),
            )]),
        )
        .unwrap();
        let second = first.clone();
        assert_eq!(first.notice_id, second.notice_id);
        assert!(first.non_financial && first.untrusted);
        assert!(!first.authoritative && !first.financial_posting_created);
        validate_notice(&first).unwrap();
    }

    #[test]
    fn exact_decimal_rejects_floats_with_exponents_or_missing_digits() {
        assert!(valid_exact_decimal("0"));
        assert!(valid_exact_decimal("-12.340"));
        assert!(!valid_exact_decimal("1e3"));
        assert!(!valid_exact_decimal(".5"));
        assert!(!valid_exact_decimal("1."));
        assert!(!valid_exact_decimal("+1"));
    }

    #[test]
    fn canonical_order_places_actions_and_gaps_before_price_marks() {
        let date = NaiveDate::from_ymd_opt(2099, 1, 2).unwrap();
        let available = DateTime::parse_from_rfc3339("2099-01-03T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let make = |event: &str| {
            new_notice(
                event,
                date,
                available,
                &hash("review"),
                &hash("output"),
                None,
                None,
                BTreeMap::new(),
                BTreeMap::new(),
            )
            .unwrap()
        };
        assert!(
            canonical_notice_key(&make("market_session_fact"))
                < canonical_notice_key(&make("dividend_notice_pending_validation"))
        );
        assert!(
            canonical_notice_key(&make("explicit_data_gap_fact"))
                < canonical_notice_key(&make("security_raw_close_mark_fact"))
        );
    }

    #[test]
    fn next_gate_requires_independent_validation() {
        assert_eq!(
            NEXT_GATE,
            "stage_123_independent_non_financial_observation_notice_candidate_validation"
        );
        assert!(PROGRAM_KIND.contains("non_financial"));
        assert!(POLICY_VERSION.contains("no-financial-state"));
    }
}
