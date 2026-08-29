//! Stage 102 one-shot, fail-closed execution of one exact Stage 101 parser claim.
//!
//! `runner.artifact` is never spawned. It is a content-addressed declarative program which binds
//! the already-reviewed parser contract to HONE's trusted deterministic interpreter. Only the
//! exact Stage 94 payload files frozen by Stage 101 are opened, rehashed and parsed. Every valid
//! execution request first consumes the claim with a create-once start marker, then creates one
//! terminal result. A failure consumes the claim permanently; an interrupted execution is
//! terminalized as failed after the frozen wall-clock deadline and can never be retried.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_execution_attempt_claims::{
    ControlledShadowMarketDataParserExecutionAttemptClaim,
    market_data_parser_execution_attempt_claims_for_execution,
    validate_market_data_parser_execution_attempt_claim,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_first_execution_authorizations::read_revalidated_market_data_parser_artifact_for_execution;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_attempts::{
    CanonicalMarketDataRequestDescriptor, controlled_shadow_market_data_receipt_custody_root,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-execution-attempt-registry-v1";
const PROGRAM_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-declarative-program-v1";
const OUTPUT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-untrusted-canonical-market-data-parse-result-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-execution-attempt-result-v1";
const START_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-execution-attempt-start-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-market-data-parser-execution-v1-one-shot-in-process-declarative-fail-closed";
const NEXT_GATE: &str = "stage_103_independent_parser_output_validation";
const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;
const MAX_PAYLOAD_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TOTAL_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;
const MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
const MAX_REASON_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct ExecuteControlledShadowMarketDataParserAttemptRequest {
    expected_claim_sha256: String,
    expected_authorization_review_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_input_manifest_sha256: String,
    execution_reason: String,
    exact_stage_51_through_stage_101_binding_confirmed: bool,
    executor_independent_from_complete_prior_chain_confirmed: bool,
    one_shot_failure_consumes_claim_and_no_retry_confirmed: bool,
    artifact_is_declarative_not_spawned_or_executed_confirmed: bool,
    only_fixed_stage_94_payloads_are_read_only_opened_and_rehashed_confirmed: bool,
    strict_parser_and_cross_source_reconciliation_fail_closed_confirmed: bool,
    output_create_once_untrusted_and_requires_independent_validation_confirmed: bool,
    no_network_environment_secret_tool_subprocess_or_production_io_confirmed: bool,
    no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ControlledShadowMarketDataParserDeclarativeProgram {
    pub schema_version: String,
    pub program_kind: String,
    pub implementation_contract_sha256: String,
    pub parser_specification_sha256: String,
    pub immutable_code_revision: String,
    pub function_ids: Vec<String>,
    pub canonical_schemas: Vec<String>,
    pub no_command_or_entrypoint: bool,
    pub deterministic_in_process_interpreter_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CanonicalMarketCalendarRow {
    pub date: NaiveDate,
    pub session_kind: String,
    pub close_time_et: String,
    pub row_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CanonicalMarketPriceRow {
    pub symbol: String,
    pub date: NaiveDate,
    pub source_kind: String,
    pub close_decimal: String,
    pub volume_integer: Option<u64>,
    pub request_sha256: String,
    pub row_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CanonicalMarketDividendRow {
    pub symbol: String,
    pub date: NaiveDate,
    pub dividend_decimal: String,
    pub record_date: Option<NaiveDate>,
    pub payment_date: Option<NaiveDate>,
    pub declaration_date: Option<NaiveDate>,
    pub row_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CanonicalMarketSplitRow {
    pub symbol: String,
    pub date: NaiveDate,
    pub numerator_positive_integer: u64,
    pub denominator_positive_integer: u64,
    pub split_type: Option<String>,
    pub row_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ExplicitMarketDataGap {
    pub symbol: String,
    pub date: NaiveDate,
    pub source_kind: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UntrustedCanonicalMarketDataParseResult {
    pub schema_version: String,
    pub output_sha256: String,
    pub source_receipt_sha256: String,
    pub parser_spec_sha256: String,
    pub execution_claim_sha256: String,
    pub calendar_rows: Vec<CanonicalMarketCalendarRow>,
    pub price_rows: Vec<CanonicalMarketPriceRow>,
    pub dividend_rows: Vec<CanonicalMarketDividendRow>,
    pub split_rows: Vec<CanonicalMarketSplitRow>,
    pub explicit_gaps: Vec<ExplicitMarketDataGap>,
    pub rejections: Vec<String>,
    pub raw_payload_count: usize,
    pub raw_payload_bytes: usize,
    pub untrusted: bool,
    pub independent_validation_completed: bool,
    pub observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowMarketDataParserExecutionStatus {
    CompletedWithUntrustedOutput,
    FailedClaimConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserExecutionAttemptResult {
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub stage_101_attempt_id: String,
    pub stage_101_claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub executed_by: String,
    pub execution_reason: String,
    pub duration_millis: u64,
    pub status: ControlledShadowMarketDataParserExecutionStatus,
    pub bounded_error_code: Option<String>,
    pub output_sha256: Option<String>,
    pub output_relative_path: Option<String>,
    pub claim_consumed: bool,
    pub artifact_revalidated: bool,
    pub artifact_spawned_or_executed: bool,
    pub raw_payloads_opened: bool,
    pub parser_executed_in_process: bool,
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
struct ControlledShadowMarketDataParserExecutionAttemptStart {
    schema_version: String,
    policy_version: String,
    start_sha256: String,
    stage_101_attempt_id: String,
    stage_101_claim_sha256: String,
    started_at: DateTime<Utc>,
    executed_by: String,
    execution_reason: String,
    one_shot_claim_consumed_before_artifact_or_payload_read: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserExecutionAttemptRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub execution_endpoint_available: bool,
    pub pending_claims: Vec<ControlledShadowMarketDataParserExecutionAttemptClaim>,
    pub results: Vec<ControlledShadowMarketDataParserExecutionAttemptResult>,
    pub pending_claim_count: usize,
    pub terminal_result_count: usize,
    pub successful_untrusted_output_count: usize,
    pub failed_consumed_claim_count: usize,
    pub next_gate: String,
    pub arbitrary_artifact_execution_allowed: bool,
    pub outbound_network_allowed: bool,
    pub independent_validation_completed: bool,
    pub forward_observation_started: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowMarketDataParserExecutionReadinessSummary {
    pub pending_claim_count: usize,
    pub terminal_result_count: usize,
    pub successful_untrusted_output_count: usize,
    pub failed_consumed_claim_count: usize,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_parser_execution_attempts(
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
            warn!(%error, "market-data parser execution registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "行情解析执行表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_execute_controlled_shadow_market_data_parser_attempt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ExecuteControlledShadowMarketDataParserAttemptRequest>,
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
                "执行结果已落盘，但刷新执行表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_parser_execution_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserExecutionReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(ControlledShadowMarketDataParserExecutionReadinessSummary {
        pending_claim_count: registry.pending_claim_count,
        terminal_result_count: registry.terminal_result_count,
        successful_untrusted_output_count: registry.successful_untrusted_output_count,
        failed_consumed_claim_count: registry.failed_consumed_claim_count,
    })
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserExecutionAttemptRegistry, String> {
    let claims = market_data_parser_execution_attempt_claims_for_execution(state).await?;
    let mut results = read_results(state).await?;
    recover_interrupted_attempts(state, &claims, &results).await?;
    results = read_results(state).await?;
    let starts = read_starts(state).await?;
    let completed = results
        .iter()
        .map(|value| value.stage_101_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    let started = starts
        .iter()
        .map(|value| value.stage_101_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    let pending_claims = claims
        .into_iter()
        .filter(|value| attempt_is_pending(&value.attempt_id, &completed, &started))
        .collect::<Vec<_>>();
    let successful = results
        .iter()
        .filter(|value| {
            value.status
                == ControlledShadowMarketDataParserExecutionStatus::CompletedWithUntrustedOutput
        })
        .count();
    let failed = results.len().saturating_sub(successful);
    Ok(ControlledShadowMarketDataParserExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        execution_endpoint_available: true,
        pending_claim_count: pending_claims.len(),
        terminal_result_count: results.len(),
        successful_untrusted_output_count: successful,
        failed_consumed_claim_count: failed,
        pending_claims,
        results,
        next_gate: NEXT_GATE.to_string(),
        arbitrary_artifact_execution_allowed: false,
        outbound_network_allowed: false,
        independent_validation_completed: false,
        forward_observation_started: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 102 对一条 Stage 101 claim 最多执行一次：先重新校验只读内容寻址工件，再把它作为严格声明式程序解释；随后只读打开并重哈希冻结的 Stage 94 载荷，由 HONE 受信任进程内内核确定性解析。成功输出 create-once 且仍为非可信，失败永久消费 claim。没有任意二进制执行、网络、环境变量、secret、工具、子进程、观察、账本、持仓、绩效、模型/训练/reward、订单、券商或交易能力。".to_string(),
    })
}

async fn execute_once(
    state: &AppState,
    executor_id: &str,
    attempt_id: &str,
    request: ExecuteControlledShadowMarketDataParserAttemptRequest,
) -> Result<ControlledShadowMarketDataParserExecutionAttemptResult, String> {
    if !valid_id(attempt_id) {
        return Err("Stage 101 attempt ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_results(state)
        .await?
        .iter()
        .any(|value| value.stage_101_attempt_id == attempt_id)
    {
        return Err("这条 Stage 101 claim 已有终态结果，不能重试".to_string());
    }
    if read_starts(state)
        .await?
        .iter()
        .any(|value| value.stage_101_attempt_id == attempt_id)
    {
        return Err("这条 Stage 101 claim 已经开始过执行，不能重试".to_string());
    }
    let claim = market_data_parser_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .find(|value| value.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有这条 Stage 101 claim".to_string())?;
    validate_execution_request(&claim, executor_id, &request)?;
    let reason = bounded_required(&request.execution_reason, MAX_REASON_CHARS, "执行原因")?;
    let mut start = ControlledShadowMarketDataParserExecutionAttemptStart {
        schema_version: START_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        start_sha256: String::new(),
        stage_101_attempt_id: claim.attempt_id.clone(),
        stage_101_claim_sha256: claim.claim_sha256.clone(),
        started_at: Utc::now(),
        executed_by: executor_id.to_string(),
        execution_reason: reason.clone(),
        one_shot_claim_consumed_before_artifact_or_payload_read: true,
    };
    start.start_sha256 = fingerprint_without(&start, &["start_sha256"])?;
    validate_start(&start, &claim)?;
    write_immutable_json(
        &start_directory(state).join(format!("{}.json", attempt_id)),
        &start,
    )
    .await?;
    let started = Instant::now();
    let execution = tokio::time::timeout(
        StdDuration::from_secs(u64::from(
            claim
                .authorization
                .runner
                .runner_contract
                .maximum_wall_clock_seconds,
        )),
        execute_claim(state, &claim),
    )
    .await
    .unwrap_or_else(|_| {
        Err(ExecutionFailure::new(
            "execution_wall_clock_limit_exceeded",
            "Stage 99 wall-clock ceiling exceeded",
            true,
            true,
            true,
        ))
    });
    let completed_at = Utc::now();
    let duration_millis = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let (
        status,
        error_code,
        output_sha256,
        output_relative_path,
        artifact_revalidated,
        raw_opened,
        parser_executed,
    ) = match execution {
        Ok((output_sha256, output_relative_path)) => (
            ControlledShadowMarketDataParserExecutionStatus::CompletedWithUntrustedOutput,
            None,
            Some(output_sha256),
            Some(output_relative_path),
            true,
            true,
            true,
        ),
        Err(failure) => {
            warn!(attempt_id, error = %failure.detail, "one-shot market-data parser execution failed");
            (
                ControlledShadowMarketDataParserExecutionStatus::FailedClaimConsumed,
                Some(failure.code.to_string()),
                None,
                None,
                failure.artifact_revalidated,
                failure.raw_payloads_opened,
                failure.parser_executed,
            )
        }
    };
    let successful =
        status == ControlledShadowMarketDataParserExecutionStatus::CompletedWithUntrustedOutput;
    let mut result = ControlledShadowMarketDataParserExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        stage_101_attempt_id: claim.attempt_id.clone(),
        stage_101_claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        executed_by: executor_id.to_string(),
        execution_reason: reason,
        duration_millis,
        status,
        bounded_error_code: error_code,
        output_sha256,
        output_relative_path,
        claim_consumed: true,
        artifact_revalidated,
        artifact_spawned_or_executed: false,
        raw_payloads_opened: raw_opened,
        parser_executed_in_process: parser_executed,
        output_untrusted: successful,
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
    validate_result(&result, &claim)?;
    write_immutable_json(
        &result_directory(state).join(format!("{}.json", attempt_id)),
        &result,
    )
    .await?;
    Ok(result)
}

async fn recover_interrupted_attempts(
    state: &AppState,
    claims: &[ControlledShadowMarketDataParserExecutionAttemptClaim],
    existing_results: &[ControlledShadowMarketDataParserExecutionAttemptResult],
) -> Result<(), String> {
    let starts = read_starts(state).await?;
    let claim_index = claims
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let completed = existing_results
        .iter()
        .map(|value| value.stage_101_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    for start in starts {
        if completed.contains(start.stage_101_attempt_id.as_str()) {
            continue;
        }
        let claim = claim_index
            .get(start.stage_101_attempt_id.as_str())
            .ok_or_else(|| "Stage 102 start marker 缺少 Stage 101 claim".to_string())?;
        let deadline = execution_deadline(
            start.started_at,
            claim
                .authorization
                .runner
                .runner_contract
                .maximum_wall_clock_seconds,
        );
        if Utc::now() <= deadline {
            continue;
        }
        let mut result = ControlledShadowMarketDataParserExecutionAttemptResult {
            schema_version: RESULT_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            result_id: String::new(),
            result_sha256: String::new(),
            stage_101_attempt_id: claim.attempt_id.clone(),
            stage_101_claim_sha256: claim.claim_sha256.clone(),
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
            status: ControlledShadowMarketDataParserExecutionStatus::FailedClaimConsumed,
            bounded_error_code: Some("execution_interrupted_without_terminal_result".to_string()),
            output_sha256: None,
            output_relative_path: None,
            claim_consumed: true,
            artifact_revalidated: false,
            artifact_spawned_or_executed: false,
            raw_payloads_opened: false,
            parser_executed_in_process: false,
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
        let path = result_directory(state).join(format!("{}.json", claim.attempt_id));
        match write_immutable_json(&path, &result).await {
            Ok(()) => {}
            Err(error) => {
                if tokio::fs::symlink_metadata(&path).await.is_err() {
                    return Err(error);
                }
            }
        }
    }
    Ok(())
}

struct ExecutionFailure {
    code: &'static str,
    detail: String,
    artifact_revalidated: bool,
    raw_payloads_opened: bool,
    parser_executed: bool,
}

fn attempt_is_pending(
    attempt_id: &str,
    completed: &BTreeSet<&str>,
    started: &BTreeSet<&str>,
) -> bool {
    !completed.contains(attempt_id) && !started.contains(attempt_id)
}

fn execution_deadline(started_at: DateTime<Utc>, maximum_wall_clock_seconds: u32) -> DateTime<Utc> {
    started_at + Duration::seconds(i64::from(maximum_wall_clock_seconds))
}

impl ExecutionFailure {
    fn new(
        code: &'static str,
        detail: impl Into<String>,
        artifact: bool,
        raw: bool,
        parser: bool,
    ) -> Self {
        Self {
            code,
            detail: detail.into(),
            artifact_revalidated: artifact,
            raw_payloads_opened: raw,
            parser_executed: parser,
        }
    }
}

async fn execute_claim(
    state: &AppState,
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
) -> Result<(String, String), ExecutionFailure> {
    let artifact =
        read_revalidated_market_data_parser_artifact_for_execution(state, &claim.authorization)
            .await
            .map_err(|error| {
                ExecutionFailure::new("artifact_revalidation_failed", error, false, false, false)
            })?;
    if artifact.len() > MAX_ARTIFACT_BYTES {
        return Err(ExecutionFailure::new(
            "artifact_contract_rejected",
            "declarative artifact exceeds maximum",
            true,
            false,
            false,
        ));
    }
    let program: ControlledShadowMarketDataParserDeclarativeProgram =
        serde_json::from_slice(&artifact).map_err(|_| {
            ExecutionFailure::new(
                "artifact_contract_rejected",
                "declarative artifact is not strict JSON",
                true,
                false,
                false,
            )
        })?;
    validate_program(&program, claim).map_err(|error| {
        ExecutionFailure::new("artifact_contract_rejected", error, true, false, false)
    })?;
    let payloads = read_fixed_payloads(state, claim).await.map_err(|error| {
        ExecutionFailure::new("raw_payload_integrity_failed", error, true, true, false)
    })?;
    let mut output = parse_payloads(claim, &payloads).map_err(|error| {
        ExecutionFailure::new("parser_contract_rejected", error, true, true, true)
    })?;
    output.output_sha256 = fingerprint_without(&output, &["output_sha256"]).map_err(|error| {
        ExecutionFailure::new("output_fingerprint_failed", error, true, true, true)
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
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err(ExecutionFailure::new(
            "output_size_limit_exceeded",
            "canonical output exceeds 8 MiB",
            true,
            true,
            true,
        ));
    }
    let relative = format!("outputs/{}.json", output.output_sha256);
    write_immutable_bytes(&execution_root(state).join(&relative), &bytes)
        .await
        .map_err(|error| {
            ExecutionFailure::new("output_create_once_write_failed", error, true, true, true)
        })?;
    Ok((output.output_sha256, relative))
}

fn validate_execution_request(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    executor_id: &str,
    request: &ExecuteControlledShadowMarketDataParserAttemptRequest,
) -> Result<(), String> {
    validate_market_data_parser_execution_attempt_claim(claim)?;
    let independent = !executor_id.trim().is_empty()
        && executor_id != claim.claimed_by
        && !claim
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == executor_id);
    let valid = request.expected_claim_sha256 == claim.claim_sha256
        && request.expected_authorization_review_sha256 == claim.authorization.review_sha256
        && request.expected_runner_artifact_sha256 == claim.authorization.server_computed_artifact_sha256
        && request.expected_input_manifest_sha256 == claim.fixed_input_manifest.input_manifest_sha256
        && independent
        && request.exact_stage_51_through_stage_101_binding_confirmed
        && request.executor_independent_from_complete_prior_chain_confirmed == independent
        && request.one_shot_failure_consumes_claim_and_no_retry_confirmed
        && request.artifact_is_declarative_not_spawned_or_executed_confirmed
        && request.only_fixed_stage_94_payloads_are_read_only_opened_and_rehashed_confirmed
        && request.strict_parser_and_cross_source_reconciliation_fail_closed_confirmed
        && request.output_create_once_untrusted_and_requires_independent_validation_confirmed
        && request.no_network_environment_secret_tool_subprocess_or_production_io_confirmed
        && request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 102 执行绑定、独立性或确认项不完整".to_string())
}

fn validate_program(
    program: &ControlledShadowMarketDataParserDeclarativeProgram,
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
) -> Result<(), String> {
    let contract = &claim
        .authorization
        .runner
        .implementation
        .implementation_contract;
    let expected_functions = vec![
        contract.strict_envelope_dispatch_function_id.clone(),
        contract.fmp_price_array_parser_function_id.clone(),
        contract.fmp_dividend_event_parser_function_id.clone(),
        contract.fmp_split_event_parser_function_id.clone(),
        contract.nyse_calendar_table_parser_function_id.clone(),
        contract
            .calendar_subject_spy_reconciliation_function_id
            .clone(),
        contract
            .canonical_row_serialization_and_hash_function_id
            .clone(),
        contract.synthetic_vector_conformance_function_id.clone(),
    ];
    let expected_schemas = vec![
        contract.canonical_calendar_row_schema.clone(),
        contract.canonical_price_row_schema.clone(),
        contract.canonical_dividend_row_schema.clone(),
        contract.canonical_split_row_schema.clone(),
        contract.canonical_parse_result_schema.clone(),
    ];
    let valid = program.schema_version == PROGRAM_SCHEMA_VERSION
        && program.program_kind == "hone_trusted_deterministic_market_data_parser_bindings"
        && program.implementation_contract_sha256 == contract.contract_sha256
        && program.parser_specification_sha256 == contract.parser_specification_sha256
        && program.immutable_code_revision == contract.immutable_code_revision
        && program.function_ids == expected_functions
        && program.canonical_schemas == expected_schemas
        && program.no_command_or_entrypoint
        && program.deterministic_in_process_interpreter_only;
    valid
        .then_some(())
        .ok_or_else(|| "声明式 parser 工件与已批准 Stage 97 合同不一致".to_string())
}

async fn read_fixed_payloads(
    state: &AppState,
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let root = controlled_shadow_market_data_receipt_custody_root(state);
    let expected_prefix =
        PathBuf::from("payloads").join(&claim.fixed_input_manifest.stage_93_claim.attempt_id);
    let mut output = BTreeMap::new();
    let mut total = 0usize;
    for receipt in &claim.fixed_input_manifest.raw_payloads {
        let relative = PathBuf::from(&receipt.raw_payload_relative_path);
        if !safe_relative_path(&relative) || !relative.starts_with(&expected_prefix) {
            return Err("冻结载荷路径越界".to_string());
        }
        let path = root.join(&relative);
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|_| "冻结载荷缺失".to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() > MAX_PAYLOAD_BYTES
        {
            return Err("冻结载荷不是受限常规文件".to_string());
        }
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|_| "冻结载荷读取失败".to_string())?;
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| "载荷总长度溢出".to_string())?;
        if total > MAX_TOTAL_PAYLOAD_BYTES
            || bytes.len() != receipt.response_bytes
            || sha256_bytes(&bytes) != receipt.response_body_sha256
            || receipt.source_document_sha256 != receipt.response_body_sha256
            || output.insert(receipt.source_id.clone(), bytes).is_some()
        {
            return Err("冻结载荷长度、摘要、总量或 source ID 不一致".to_string());
        }
    }
    if output.len() != claim.fixed_input_manifest.raw_payload_count
        || total != claim.fixed_input_manifest.total_response_bytes
    {
        return Err("冻结载荷集合与 Stage 101 清单不一致".to_string());
    }
    Ok(output)
}

fn parse_payloads(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    payloads: &BTreeMap<String, Vec<u8>>,
) -> Result<UntrustedCanonicalMarketDataParseResult, String> {
    let request_index = claim
        .fixed_input_manifest
        .stage_93_claim
        .canonical_requests
        .iter()
        .map(|value| (value.source_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    if request_index.len() != payloads.len() {
        return Err("canonical request 与载荷数量不一致".to_string());
    }
    let from = claim.fixed_input_manifest.window_start_date;
    let to = claim.fixed_input_manifest.window_end_date;
    let mut calendar_rows = Vec::new();
    let mut price_rows = Vec::new();
    let mut dividend_rows = Vec::new();
    let mut split_rows = Vec::new();
    for (source_id, bytes) in payloads {
        let request = request_index
            .get(source_id.as_str())
            .ok_or_else(|| "载荷缺少 canonical request".to_string())?;
        if request.canonical_request_sha256
            != claim
                .fixed_input_manifest
                .raw_payloads
                .iter()
                .find(|value| value.source_id == *source_id)
                .map(|value| value.canonical_request_sha256.as_str())
                .unwrap_or_default()
        {
            return Err("载荷 canonical request 摘要漂移".to_string());
        }
        if source_id == "nyse_official_us_market_calendar" {
            calendar_rows = parse_nyse_calendar(bytes, from, to)?;
            continue;
        }
        let symbol = request_symbol(request)?;
        match request.path.as_str() {
            "/stable/historical-price-eod/full" => price_rows.extend(parse_price_rows(
                bytes,
                &symbol,
                "split_adjusted_price",
                &request.canonical_request_sha256,
                from,
                to,
            )?),
            "/stable/historical-price-eod/non-split-adjusted" => {
                price_rows.extend(parse_price_rows(
                    bytes,
                    &symbol,
                    "raw_unadjusted_price",
                    &request.canonical_request_sha256,
                    from,
                    to,
                )?)
            }
            "/stable/historical-price-eod/dividend-adjusted" => {
                price_rows.extend(parse_price_rows(
                    bytes,
                    &symbol,
                    "dividend_adjusted_price",
                    &request.canonical_request_sha256,
                    from,
                    to,
                )?)
            }
            "/stable/dividends" => {
                dividend_rows.extend(parse_dividend_rows(bytes, &symbol, from, to)?)
            }
            "/stable/splits" => split_rows.extend(parse_split_rows(bytes, &symbol, from, to)?),
            _ => return Err("canonical request path 不在 Stage 95 allowlist".to_string()),
        }
    }
    if calendar_rows.is_empty() {
        return Err("官方日历在冻结窗口内没有交易日".to_string());
    }
    calendar_rows.sort_by_key(|value| value.date);
    price_rows.sort_by(|left, right| {
        (&left.symbol, &left.source_kind, left.date).cmp(&(
            &right.symbol,
            &right.source_kind,
            right.date,
        ))
    });
    dividend_rows
        .sort_by(|left, right| (&left.symbol, left.date).cmp(&(&right.symbol, right.date)));
    split_rows.sort_by(|left, right| (&left.symbol, left.date).cmp(&(&right.symbol, right.date)));
    let explicit_gaps = reconcile_calendar_and_prices(claim, &calendar_rows, &price_rows)?;
    let raw_payload_bytes = payloads.values().map(Vec::len).sum();
    Ok(UntrustedCanonicalMarketDataParseResult {
        schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        output_sha256: String::new(),
        source_receipt_sha256: claim.fixed_input_manifest.stage_93_receipt_sha256.clone(),
        parser_spec_sha256: claim
            .authorization
            .runner
            .implementation
            .implementation_contract
            .parser_specification_sha256
            .clone(),
        execution_claim_sha256: claim.claim_sha256.clone(),
        calendar_rows,
        price_rows,
        dividend_rows,
        split_rows,
        explicit_gaps,
        rejections: Vec::new(),
        raw_payload_count: payloads.len(),
        raw_payload_bytes,
        untrusted: true,
        independent_validation_completed: false,
        observation_started: false,
        ledger_created: false,
        position_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    })
}

fn parse_price_rows(
    bytes: &[u8],
    symbol: &str,
    source_kind: &str,
    request_sha256: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketPriceRow>, String> {
    let rows = strict_json_array(bytes)?;
    if rows.is_empty() {
        return Err(format!("{symbol} {source_kind} 价格序列为空"));
    }
    let mut dates = BTreeSet::new();
    let mut output = Vec::new();
    for value in rows {
        let object = value
            .as_object()
            .ok_or_else(|| "FMP price row 不是 object".to_string())?;
        require_symbol(object.get("symbol"), symbol)?;
        let date = required_window_date(object.get("date"), from, to)?;
        if !dates.insert(date) {
            return Err(format!("{symbol} {source_kind} 出现重复日期"));
        }
        let close_decimal = required_decimal(object.get("close"), true, "close")?;
        let volume_integer = match object.get("volume") {
            None | Some(Value::Null) => None,
            Some(value) => Some(
                value
                    .as_u64()
                    .ok_or_else(|| "volume 必须是非负整数".to_string())?,
            ),
        };
        let mut row = CanonicalMarketPriceRow {
            symbol: symbol.to_string(),
            date,
            source_kind: source_kind.to_string(),
            close_decimal,
            volume_integer,
            request_sha256: request_sha256.to_string(),
            row_sha256: String::new(),
        };
        row.row_sha256 = fingerprint_without(&row, &["row_sha256"])?;
        output.push(row);
    }
    Ok(output)
}

fn parse_dividend_rows(
    bytes: &[u8],
    symbol: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketDividendRow>, String> {
    let rows = strict_json_array(bytes)?;
    let mut dates = BTreeSet::new();
    let mut output = Vec::new();
    for value in rows {
        let object = value
            .as_object()
            .ok_or_else(|| "FMP dividend row 不是 object".to_string())?;
        require_symbol(object.get("symbol"), symbol)?;
        let date = required_window_date(object.get("date"), from, to)?;
        if !dates.insert(date) {
            return Err(format!("{symbol} dividend 出现重复日期"));
        }
        let decimal_value = object.get("dividend").or_else(|| object.get("adjDividend"));
        let mut row = CanonicalMarketDividendRow {
            symbol: symbol.to_string(),
            date,
            dividend_decimal: required_decimal(decimal_value, false, "dividend")?,
            record_date: optional_date(object.get("recordDate"))?,
            payment_date: optional_date(object.get("paymentDate"))?,
            declaration_date: optional_date(object.get("declarationDate"))?,
            row_sha256: String::new(),
        };
        row.row_sha256 = fingerprint_without(&row, &["row_sha256"])?;
        output.push(row);
    }
    Ok(output)
}

fn parse_split_rows(
    bytes: &[u8],
    symbol: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketSplitRow>, String> {
    let rows = strict_json_array(bytes)?;
    let mut dates = BTreeSet::new();
    let mut output = Vec::new();
    for value in rows {
        let object = value
            .as_object()
            .ok_or_else(|| "FMP split row 不是 object".to_string())?;
        require_symbol(object.get("symbol"), symbol)?;
        let date = required_window_date(object.get("date"), from, to)?;
        if !dates.insert(date) {
            return Err(format!("{symbol} split 出现重复日期"));
        }
        let numerator = object
            .get("numerator")
            .and_then(Value::as_u64)
            .filter(|value| *value > 0)
            .ok_or_else(|| "split numerator 必须是正整数".to_string())?;
        let denominator = object
            .get("denominator")
            .and_then(Value::as_u64)
            .filter(|value| *value > 0)
            .ok_or_else(|| "split denominator 必须是正整数".to_string())?;
        let split_type = match object.get("splitType") {
            None | Some(Value::Null) => None,
            Some(Value::String(value)) if !value.trim().is_empty() => {
                Some(value.trim().to_string())
            }
            _ => return Err("splitType 必须是非空 string 或 null".to_string()),
        };
        let mut row = CanonicalMarketSplitRow {
            symbol: symbol.to_string(),
            date,
            numerator_positive_integer: numerator,
            denominator_positive_integer: denominator,
            split_type,
            row_sha256: String::new(),
        };
        row.row_sha256 = fingerprint_without(&row, &["row_sha256"])?;
        output.push(row);
    }
    Ok(output)
}

fn parse_nyse_calendar(
    bytes: &[u8],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketCalendarRow>, String> {
    let html =
        std::str::from_utf8(bytes).map_err(|_| "NYSE calendar 必须是 UTF-8 HTML".to_string())?;
    let document = Html::parse_document(html);
    let table_selector =
        Selector::parse("table").map_err(|_| "calendar selector 无效".to_string())?;
    let row_selector =
        Selector::parse("tr").map_err(|_| "calendar row selector 无效".to_string())?;
    let cell_selector =
        Selector::parse("th,td").map_err(|_| "calendar cell selector 无效".to_string())?;
    for table in document.select(&table_selector) {
        let rows = table
            .select(&row_selector)
            .map(|row| {
                row.select(&cell_selector)
                    .map(|cell| normalized_text(cell.text()))
                    .collect::<Vec<_>>()
            })
            .filter(|cells| !cells.is_empty())
            .collect::<Vec<_>>();
        if rows
            .first()
            .is_some_and(|header| header == &["Date", "Session", "Close ET"])
        {
            return parse_direct_session_table(&rows[1..], from, to);
        }
    }
    parse_nyse_holiday_table(&document, from, to)
}

fn parse_direct_session_table(
    rows: &[Vec<String>],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketCalendarRow>, String> {
    let mut dates = BTreeSet::new();
    let mut output = Vec::new();
    for cells in rows {
        if cells.len() != 3 {
            return Err("NYSE direct calendar row 列数错误".to_string());
        }
        let date = NaiveDate::parse_from_str(&cells[0], "%Y-%m-%d")
            .map_err(|_| "NYSE calendar date 无效".to_string())?;
        if date < from || date > to {
            return Err("NYSE calendar row 超出冻结窗口".to_string());
        }
        if !dates.insert(date) {
            return Err("NYSE calendar 出现重复日期".to_string());
        }
        let session_kind = cells[1].as_str();
        let close = cells[2].as_str();
        if !matches!(
            (session_kind, close),
            ("regular", "16:00") | ("early_close", "13:00")
        ) {
            return Err("NYSE session_kind/close_time_et 无效".to_string());
        }
        output.push(hashed_calendar_row(date, session_kind, close)?);
    }
    Ok(output)
}

fn parse_nyse_holiday_table(
    document: &Html,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketCalendarRow>, String> {
    let table_selector =
        Selector::parse("table").map_err(|_| "calendar selector 无效".to_string())?;
    let row_selector =
        Selector::parse("tr").map_err(|_| "calendar row selector 无效".to_string())?;
    let cell_selector =
        Selector::parse("th,td").map_err(|_| "calendar cell selector 无效".to_string())?;
    let required_years = (from.year()..=to.year()).collect::<BTreeSet<_>>();
    let mut available_years = BTreeSet::new();
    let mut holidays = BTreeSet::new();
    let mut found = false;
    for table in document.select(&table_selector) {
        let rows = table
            .select(&row_selector)
            .map(|row| {
                row.select(&cell_selector)
                    .map(|cell| normalized_text(cell.text()))
                    .collect::<Vec<_>>()
            })
            .filter(|cells| !cells.is_empty())
            .collect::<Vec<_>>();
        let Some(header) = rows.first() else { continue };
        if header.first().map(String::as_str) != Some("Holiday") {
            continue;
        }
        found = true;
        let years = header
            .iter()
            .skip(1)
            .map(|value| {
                value
                    .parse::<i32>()
                    .map_err(|_| "NYSE holiday table year 无效".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        available_years.extend(years.iter().copied());
        for cells in rows.iter().skip(1) {
            if cells.len() != years.len() + 1 {
                return Err("NYSE holiday table row 列数错误".to_string());
            }
            for (index, year) in years.iter().enumerate() {
                if let Some(date) = parse_holiday_cell(&cells[index + 1], *year)? {
                    holidays.insert(date);
                }
            }
        }
    }
    if !found || !required_years.is_subset(&available_years) {
        return Err("NYSE official holiday table 未覆盖冻结窗口年份".to_string());
    }
    let early_closes = parse_early_close_dates(document)?;
    let mut output = Vec::new();
    let mut date = from;
    while date <= to {
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) && !holidays.contains(&date) {
            let early = early_closes.contains(&date);
            output.push(hashed_calendar_row(
                date,
                if early { "early_close" } else { "regular" },
                if early { "13:00" } else { "16:00" },
            )?);
        }
        date = date
            .checked_add_signed(Duration::days(1))
            .ok_or_else(|| "calendar date overflow".to_string())?;
    }
    Ok(output)
}

fn parse_early_close_dates(document: &Html) -> Result<BTreeSet<NaiveDate>, String> {
    let selector =
        Selector::parse("p").map_err(|_| "calendar paragraph selector 无效".to_string())?;
    let pattern = Regex::new(r"(?:Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday),\s+([A-Z][a-z]+\s+\d{1,2}),\s+(\d{4})").map_err(|_| "calendar date regex 无效".to_string())?;
    let mut output = BTreeSet::new();
    for paragraph in document.select(&selector) {
        let text = normalized_text(paragraph.text());
        if !text.contains("close early at 1:00 p.m.") {
            continue;
        }
        for captures in pattern.captures_iter(&text) {
            let date = NaiveDate::parse_from_str(
                &format!("{}, {}", &captures[1], &captures[2]),
                "%B %d, %Y",
            )
            .map_err(|_| "NYSE early-close date 无效".to_string())?;
            output.insert(date);
        }
    }
    Ok(output)
}

fn parse_holiday_cell(value: &str, year: i32) -> Result<Option<NaiveDate>, String> {
    if value.starts_with('—') || value == "-" {
        return Ok(None);
    }
    let without_weekday = value
        .split_once(',')
        .map(|(_, rest)| rest.trim())
        .unwrap_or(value.trim());
    let cleaned = without_weekday
        .split('(')
        .next()
        .unwrap_or(without_weekday)
        .replace('*', "");
    let cleaned = cleaned.trim();
    NaiveDate::parse_from_str(&format!("{cleaned}, {year}"), "%B %d, %Y")
        .map(Some)
        .map_err(|_| format!("NYSE holiday date 无效: {value}"))
}

fn reconcile_calendar_and_prices(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    calendar: &[CanonicalMarketCalendarRow],
    prices: &[CanonicalMarketPriceRow],
) -> Result<Vec<ExplicitMarketDataGap>, String> {
    let sessions = calendar
        .iter()
        .map(|value| value.date)
        .collect::<BTreeSet<_>>();
    let kinds = [
        "split_adjusted_price",
        "raw_unadjusted_price",
        "dividend_adjusted_price",
    ];
    let mut gaps = Vec::new();
    let mut symbols = claim.fixed_input_manifest.subject_symbols.clone();
    if !symbols
        .iter()
        .any(|value| value == &claim.fixed_input_manifest.benchmark_symbol)
    {
        symbols.push(claim.fixed_input_manifest.benchmark_symbol.clone());
    }
    symbols.sort();
    symbols.dedup();
    for symbol in symbols {
        for kind in kinds {
            let dates = prices
                .iter()
                .filter(|row| row.symbol == symbol && row.source_kind == kind)
                .map(|row| row.date)
                .collect::<BTreeSet<_>>();
            if dates.iter().any(|date| !sessions.contains(date)) {
                return Err(format!("{symbol} {kind} 包含非官方交易日"));
            }
            for date in sessions.difference(&dates) {
                if symbol == claim.fixed_input_manifest.benchmark_symbol {
                    return Err(format!("SPY {kind} 未覆盖官方交易日 {date}"));
                }
                gaps.push(ExplicitMarketDataGap {
                    symbol: symbol.clone(),
                    date: *date,
                    source_kind: kind.to_string(),
                    reason: "subject_missing_official_session_no_synthetic_row".to_string(),
                });
            }
        }
    }
    gaps.sort_by(|left, right| {
        (&left.symbol, &left.source_kind, left.date).cmp(&(
            &right.symbol,
            &right.source_kind,
            right.date,
        ))
    });
    Ok(gaps)
}

fn strict_json_array(bytes: &[u8]) -> Result<Vec<Value>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "FMP payload 必须是 UTF-8".to_string())?;
    serde_json::from_str::<Value>(text)
        .map_err(|_| "FMP payload 不是有效 JSON".to_string())?
        .as_array()
        .cloned()
        .ok_or_else(|| "FMP payload 顶层必须是 array".to_string())
}

fn request_symbol(request: &CanonicalMarketDataRequestDescriptor) -> Result<String, String> {
    request
        .query
        .iter()
        .find(|(key, _)| key == "symbol")
        .map(|(_, value)| value.clone())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "canonical request 缺少 symbol".to_string())
}

fn require_symbol(value: Option<&Value>, expected: &str) -> Result<(), String> {
    match value {
        Some(Value::String(value)) if value == expected => Ok(()),
        _ => Err(format!("FMP row symbol 与请求 {expected} 不一致")),
    }
}

fn required_window_date(
    value: Option<&Value>,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<NaiveDate, String> {
    let date = optional_date(value)?.ok_or_else(|| "row 缺少 date".to_string())?;
    if date < from || date > to {
        Err("row date 超出冻结窗口".to_string())
    } else {
        Ok(date)
    }
}

fn optional_date(value: Option<&Value>) -> Result<Option<NaiveDate>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| "date 必须是 YYYY-MM-DD".to_string()),
        _ => Err("date 必须是 string 或 null".to_string()),
    }
}

fn required_decimal(value: Option<&Value>, positive: bool, field: &str) -> Result<String, String> {
    let number = value
        .and_then(Value::as_number)
        .ok_or_else(|| format!("{field} 必须是 JSON number"))?;
    let parsed = number
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| format!("{field} 必须是有限数值"))?;
    if (positive && parsed <= 0.0) || (!positive && parsed < 0.0) {
        return Err(format!("{field} 超出允许范围"));
    }
    Ok(canonical_number(number))
}

fn canonical_number(number: &Number) -> String {
    let value = number.to_string();
    if value.contains('.') && !value.contains(['e', 'E']) {
        value
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        value
    }
}

fn normalized_text<'a>(values: impl Iterator<Item = &'a str>) -> String {
    values
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn hashed_calendar_row(
    date: NaiveDate,
    kind: &str,
    close: &str,
) -> Result<CanonicalMarketCalendarRow, String> {
    let mut row = CanonicalMarketCalendarRow {
        date,
        session_kind: kind.to_string(),
        close_time_et: close.to_string(),
        row_sha256: String::new(),
    };
    row.row_sha256 = fingerprint_without(&row, &["row_sha256"])?;
    Ok(row)
}

fn validate_result(
    result: &ControlledShadowMarketDataParserExecutionAttemptResult,
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
) -> Result<(), String> {
    let successful = result.status
        == ControlledShadowMarketDataParserExecutionStatus::CompletedWithUntrustedOutput;
    let valid = result.schema_version == RESULT_SCHEMA_VERSION
        && result.policy_version == POLICY_VERSION
        && valid_id(&result.result_id)
        && valid_sha256(&result.result_sha256)
        && result.result_id == result.result_sha256[..32]
        && result.result_sha256 == fingerprint_without(result, &["result_id", "result_sha256"])?
        && result.stage_101_attempt_id == claim.attempt_id
        && result.stage_101_claim_sha256 == claim.claim_sha256
        && !result.executed_by.trim().is_empty()
        && !result.execution_reason.trim().is_empty()
        && result.claim_consumed
        && !result.artifact_spawned_or_executed
        && result.output_sha256.is_some() == successful
        && result.output_relative_path.is_some() == successful
        && result.bounded_error_code.is_none() == successful
        && result.output_untrusted == successful
        && !result.independent_validation_completed
        && !result.forward_observation_started
        && !result.ledger_created
        && !result.position_written
        && !result.performance_metric_written
        && !result.model_or_metric_store_written
        && !result.training_feedback_authorized
        && !result.reward_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 102 终态结果无效或越权".to_string())
}

fn validate_start(
    start: &ControlledShadowMarketDataParserExecutionAttemptStart,
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
) -> Result<(), String> {
    let independent = start.executed_by != claim.claimed_by
        && !claim
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &start.executed_by);
    let valid = start.schema_version == START_SCHEMA_VERSION
        && start.policy_version == POLICY_VERSION
        && valid_sha256(&start.start_sha256)
        && start.start_sha256 == fingerprint_without(start, &["start_sha256"])?
        && start.stage_101_attempt_id == claim.attempt_id
        && start.stage_101_claim_sha256 == claim.claim_sha256
        && !start.executed_by.trim().is_empty()
        && independent
        && !start.execution_reason.trim().is_empty()
        && start.execution_reason.chars().count() <= MAX_REASON_CHARS
        && start.one_shot_claim_consumed_before_artifact_or_payload_read;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 102 start marker 无效、漂移或越权".to_string())
}

async fn read_starts(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserExecutionAttemptStart>, String> {
    let mut directory = match tokio::fs::read_dir(start_directory(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let claims = market_data_parser_execution_attempt_claims_for_execution(state).await?;
    let index = claims
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let start: ControlledShadowMarketDataParserExecutionAttemptStart = serde_json::from_slice(
            &tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        let claim = index
            .get(start.stage_101_attempt_id.as_str())
            .ok_or_else(|| "Stage 102 start marker 缺少 Stage 101 claim".to_string())?;
        validate_start(&start, claim)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(start.stage_101_attempt_id.as_str())
            || !ids.insert(start.stage_101_attempt_id.clone())
        {
            return Err("Stage 102 start marker 重复、重放或文件名错误".to_string());
        }
        output.push(start);
    }
    output.sort_by_key(|value| value.started_at);
    Ok(output)
}

pub(crate) async fn market_data_parser_execution_results_for_independent_validation(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserExecutionAttemptResult>, String> {
    read_results(state).await
}

async fn read_results(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserExecutionAttemptResult>, String> {
    let mut directory = match tokio::fs::read_dir(result_directory(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let claims = market_data_parser_execution_attempt_claims_for_execution(state).await?;
    let index = claims
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let result: ControlledShadowMarketDataParserExecutionAttemptResult =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        let claim = index
            .get(result.stage_101_attempt_id.as_str())
            .ok_or_else(|| "Stage 102 结果缺少 Stage 101 claim".to_string())?;
        validate_result(&result, claim)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(result.stage_101_attempt_id.as_str())
            || !ids.insert(result.stage_101_attempt_id.clone())
        {
            return Err("Stage 102 结果重复、重放或文件名错误".to_string());
        }
        output.push(result);
    }
    output.sort_by_key(|value| value.completed_at);
    Ok(output)
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 102 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(sha256_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

fn sha256_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars {
        Err(format!("{label}为空或过长"))
    } else {
        Ok(value.to_string())
    }
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    write_immutable_bytes(
        path,
        &serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?,
    )
    .await
}

async fn write_immutable_bytes(path: &Path, value: &[u8]) -> Result<(), String> {
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
    file.write_all(value)
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
fn execution_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-market-data-parser-execution-attempts")
}
pub(crate) fn controlled_shadow_market_data_parser_execution_custody_root(
    state: &AppState,
) -> PathBuf {
    execution_root(state)
}
fn start_directory(state: &AppState) -> PathBuf {
    execution_root(state).join("starts")
}
fn result_directory(state: &AppState) -> PathBuf {
    execution_root(state).join("results")
}
fn lock_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("governance-locks")
}

struct AttemptLock {
    path: PathBuf,
}
impl Drop for AttemptLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<AttemptLock, String> {
    tokio::fs::create_dir_all(lock_directory(state))
        .await
        .map_err(|error| error.to_string())?;
    let path =
        lock_directory(state).join(format!("market-data-parser-stage-102-{attempt_id}.lock"));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(AttemptLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|value| value.modified().ok())
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
                return Err("已有同一 Stage 102 执行正在进行".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 102 执行锁不可用".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_parser_is_strict_and_hashes_rows() {
        let from = NaiveDate::from_ymd_opt(2099, 1, 3).unwrap();
        let to = NaiveDate::from_ymd_opt(2099, 1, 4).unwrap();
        let rows = parse_price_rows(
            br#"[{"symbol":"ZZTEST","date":"2099-01-03","close":10.50,"volume":100}]"#,
            "ZZTEST",
            "split_adjusted_price",
            &"a".repeat(64),
            from,
            to,
        )
        .unwrap();
        assert_eq!(rows[0].close_decimal, "10.5");
        assert!(valid_sha256(&rows[0].row_sha256));
        assert!(
            parse_price_rows(
                br#"[{"symbol":"ZZTEST","date":"2099-01-03","close":"10.5"}]"#,
                "ZZTEST",
                "split_adjusted_price",
                &"a".repeat(64),
                from,
                to
            )
            .is_err()
        );
    }

    #[test]
    fn duplicate_and_out_of_window_rows_fail_closed() {
        let from = NaiveDate::from_ymd_opt(2099, 1, 3).unwrap();
        let to = NaiveDate::from_ymd_opt(2099, 1, 4).unwrap();
        assert!(parse_price_rows(br#"[{"symbol":"ZZTEST","date":"2099-01-03","close":10.5},{"symbol":"ZZTEST","date":"2099-01-03","close":10.6}]"#, "ZZTEST", "raw_unadjusted_price", &"a".repeat(64), from, to).is_err());
        assert!(
            parse_price_rows(
                br#"[{"symbol":"ZZTEST","date":"2099-01-02","close":10.5}]"#,
                "ZZTEST",
                "raw_unadjusted_price",
                &"a".repeat(64),
                from,
                to
            )
            .is_err()
        );
    }

    #[test]
    fn fmp_dividend_and_split_event_shapes_are_explicit() {
        let from = NaiveDate::from_ymd_opt(2099, 1, 3).unwrap();
        let to = NaiveDate::from_ymd_opt(2099, 1, 4).unwrap();
        let dividends = parse_dividend_rows(
            br#"[{"symbol":"ZZTEST","date":"2099-01-03","dividend":0.25,"recordDate":"2099-01-03","paymentDate":null,"declarationDate":null}]"#,
            "ZZTEST", from, to,
        ).unwrap();
        assert_eq!(dividends[0].dividend_decimal, "0.25");
        let splits = parse_split_rows(
            br#"[{"symbol":"ZZTEST","date":"2099-01-04","numerator":4,"denominator":1,"splitType":"forward"}]"#,
            "ZZTEST", from, to,
        ).unwrap();
        assert_eq!(splits[0].numerator_positive_integer, 4);
        assert!(
            parse_split_rows(
                br#"[{"symbol":"ZZTEST","date":"2099-01-04","splitRatio":"4:1"}]"#,
                "ZZTEST",
                from,
                to,
            )
            .is_err()
        );
    }

    #[test]
    fn direct_nyse_synthetic_table_conforms() {
        let from = NaiveDate::from_ymd_opt(2099, 1, 3).unwrap();
        let html = br#"<table><thead><tr><th>Date</th><th>Session</th><th>Close ET</th></tr></thead><tbody><tr><td>2099-01-03</td><td>regular</td><td>16:00</td></tr></tbody></table>"#;
        let rows = parse_nyse_calendar(html, from, from).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].session_kind, "regular");
    }

    #[test]
    fn actual_nyse_holiday_shape_generates_sessions_and_early_close() {
        let html = br#"<html><body><table><thead><tr><th>Holiday</th><th>2026</th></tr></thead><tbody><tr><th>Thanksgiving Day</th><td>Thursday, November 26***</td></tr></tbody></table><p>*** Each market will close early at 1:00 p.m. (1:15 p.m. for eligible options) on Friday, November 27, 2026 (the day after Thanksgiving).</p></body></html>"#;
        let from = NaiveDate::from_ymd_opt(2026, 11, 25).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 11, 27).unwrap();
        let rows = parse_nyse_calendar(html, from, to).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].session_kind, "early_close");
        assert_eq!(rows[1].close_time_et, "13:00");
    }

    #[test]
    fn paths_cannot_escape_custody() {
        assert!(safe_relative_path(Path::new("payloads/a/file.bin")));
        assert!(!safe_relative_path(Path::new("payloads/a/../secret")));
        assert!(!safe_relative_path(Path::new("/tmp/secret")));
    }

    #[test]
    fn execution_has_no_arbitrary_process_or_trading_authority() {
        assert!(POLICY_VERSION.contains("in-process-declarative"));
        assert!(START_SCHEMA_VERSION.contains("execution-attempt-start"));
        assert_eq!(NEXT_GATE, "stage_103_independent_parser_output_validation");
        assert_eq!(MAX_OUTPUT_BYTES, 8 * 1024 * 1024);
        assert!(MAX_REASON_CHARS > 0);
    }

    #[test]
    fn started_or_completed_attempt_is_never_pending_again() {
        let mut started = BTreeSet::new();
        started.insert("attempt-started");
        let mut completed = BTreeSet::new();
        completed.insert("attempt-completed");
        assert!(!attempt_is_pending("attempt-started", &completed, &started));
        assert!(!attempt_is_pending(
            "attempt-completed",
            &completed,
            &started
        ));
        assert!(attempt_is_pending("attempt-new", &completed, &started));
    }

    #[test]
    fn interrupted_attempt_uses_frozen_wall_clock_deadline() {
        let started_at = DateTime::parse_from_rfc3339("2026-08-27T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(
            execution_deadline(started_at, 300),
            DateTime::parse_from_rfc3339("2026-08-27T12:05:00Z")
                .unwrap()
                .with_timezone(&Utc)
        );
    }
}
