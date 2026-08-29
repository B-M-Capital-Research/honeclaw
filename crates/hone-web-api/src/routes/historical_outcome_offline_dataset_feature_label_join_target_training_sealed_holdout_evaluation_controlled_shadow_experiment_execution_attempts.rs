//! Claim-first, single-use Stage 80 controlled-shadow initialization attempts.
//!
//! One exact unexpired Stage 79 authorization is irreversibly claimed before
//! the runner digest, frozen candidate artifacts or point-in-time input are
//! opened. A successful attempt creates only a content-addressed, untrusted
//! initial observation envelope. It does not create a portfolio ledger or
//! position, does not calculate not-yet-observed forward performance, and has
//! no order, broker, production-write, feedback, reward or trading capability.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_execution_attempts::{
    HistoricalOutcomeTrainingFeaturePreprocessingRecord,
    HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_first_execution_authorizations::{
    ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    exact_controlled_shadow_first_execution_authorization_for_audit,
    execution_attempt_eligible_controlled_shadow_authorizations,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registrations::ControlledShadowExperimentPortfolioConstraints;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_execution_attempts::{
    exact_candidate_artifact, exact_target_model,
    exact_training_envelope_for_controlled_shadow_execution, parse_algorithm_id,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts::{
    parse_f64_hex, predict_target,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str = "hone-controlled-shadow-experiment-execution-attempt-claim-v1";
const RESULT_SCHEMA_VERSION: &str = "hone-controlled-shadow-experiment-execution-attempt-result-v1";
const INPUT_SCHEMA_VERSION: &str = "controlled_shadow_point_in_time_read_only_input_v1_not_mounted";
const OUTPUT_SCHEMA_VERSION: &str =
    "controlled_shadow_checkpoint_observation_envelope_v1_no_order_intent";
const EXECUTION_POLICY_VERSION: &str =
    "hone-controlled-shadow-claim-first-one-shot-initialization-v1";
const ISOLATION_BACKEND: &str =
    "hone-in-process-capability-limited-controlled-shadow-initialization-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const MAX_INPUT_ROWS: usize = 100;
const MAX_SOURCE_COUNT: usize = 256;
const MAX_ERROR_CHARS: usize = 1_200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowPointInTimeSourceKind {
    SecFiling,
    CompanyInvestorRelations,
    LicensedMarketData,
    ExchangeOfficialData,
    GovernmentOfficialData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowPointInTimeSource {
    pub source_kind: ControlledShadowPointInTimeSourceKind,
    pub source_id: String,
    pub content_sha256: String,
    pub available_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowPointInTimeInputRow {
    pub symbol: String,
    pub frozen_theme_id: String,
    pub security_type: String,
    pub available_at: DateTime<Utc>,
    pub eligible_in_frozen_universe: bool,
    pub tradable_at_signal_cutoff: bool,
    pub adjusted_close_f64_bits_hex: String,
    pub feature_values_f64_bits_hex: Vec<Option<String>>,
    pub source_content_sha256s: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowPointInTimeInputEnvelope {
    pub schema_version: String,
    pub input_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub feature_order: Vec<String>,
    pub preprocessing_sha256: String,
    pub signal_cutoff_at: DateTime<Utc>,
    pub captured_at: DateTime<Utc>,
    pub expected_next_full_market_session_at: DateTime<Utc>,
    pub benchmark_symbol: String,
    pub benchmark_adjusted_close_f64_bits_hex: String,
    pub sources: Vec<ControlledShadowPointInTimeSource>,
    pub rows: Vec<ControlledShadowPointInTimeInputRow>,
    pub point_in_time_read_only: bool,
    pub content_addressed: bool,
    pub allowlisted_sources_only: bool,
    pub no_retroactive_revision: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeControlledShadowExperimentOnceRequest {
    expected_authorization_review_id: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_runner_code_revision: String,
    expected_runner_contract_sha256: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_design_specification_sha256: String,
    expected_candidate_set_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    expected_input_manifest_sha256: String,
    input: ControlledShadowPointInTimeInputEnvelope,
    claim_first_single_use_and_failure_consumes_confirmed: bool,
    exact_stage_51_through_stage_79_binding_confirmed: bool,
    current_binary_digest_reverification_after_claim_confirmed: bool,
    point_in_time_read_only_content_addressed_allowlisted_input_confirmed: bool,
    deterministic_three_seed_long_only_initialization_confirmed: bool,
    no_future_performance_or_checkpoint_fabrication_confirmed: bool,
    create_once_untrusted_output_requires_independent_validation_confirmed: bool,
    no_ledger_position_order_broker_or_trading_confirmed: bool,
    no_model_metric_store_feedback_composite_or_reward_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentExecutionAttemptClaim {
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
    pub runner_contract_sha256: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub design_specification_sha256: String,
    pub candidate_set_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub exact_random_seeds: Vec<u64>,
    pub input_manifest_sha256: String,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub executor_independent_from_stage_79_and_complete_prior_chain: bool,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub input_opened: bool,
    pub point_in_time_input_read_allowed: bool,
    pub ephemeral_untrusted_output_write_allowed: bool,
    pub shadow_ledger_write_allowed: bool,
    pub shadow_position_write_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub training_feedback_allowed: bool,
    pub scalar_reward_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowSeedPrediction {
    pub random_seed: u64,
    pub prediction_f64_bits_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowVirtualAllocationObservation {
    pub symbol: String,
    pub frozen_theme_id: String,
    pub adjusted_close_f64_bits_hex: String,
    pub seed_predictions: Vec<ControlledShadowSeedPrediction>,
    pub mean_prediction_f64_bits_hex: String,
    pub rank: usize,
    pub virtual_target_weight_bps: u32,
    pub pending_next_session_simulated_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentUntrustedInitialObservationEnvelope {
    pub schema_version: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub implementation_contract_sha256: String,
    pub design_specification_sha256: String,
    pub input_manifest_sha256: String,
    pub signal_cutoff_at: DateTime<Utc>,
    pub expected_next_full_market_session_at: DateTime<Utc>,
    pub benchmark_symbol: String,
    pub benchmark_adjusted_close_f64_bits_hex: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub exact_random_seeds: Vec<u64>,
    pub virtual_notional_usd: u64,
    pub virtual_allocations: Vec<ControlledShadowVirtualAllocationObservation>,
    pub virtual_gross_exposure_bps: u32,
    pub virtual_cash_weight_bps: u32,
    pub forward_market_sessions_observed: u16,
    pub minimum_forward_market_sessions: u16,
    pub checkpoint_market_sessions: Vec<u16>,
    pub performance_metrics: Vec<String>,
    pub initialization_only: bool,
    pub output_is_untrusted: bool,
    pub independent_output_validation_completed: bool,
    pub shadow_ledger_created: bool,
    pub shadow_position_written: bool,
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
pub(crate) enum ControlledShadowExperimentExecutionAttemptStatus {
    CompletedWithUntrustedInitialObservation,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: ControlledShadowExperimentExecutionAttemptStatus,
    pub exit_code: i32,
    pub output_sha256: Option<String>,
    pub output_bytes: usize,
    pub bounded_error: Option<String>,
    pub untrusted_initial_observation:
        Option<ControlledShadowExperimentUntrustedInitialObservationEnvelope>,
    pub current_binary_digest_reverified: bool,
    pub point_in_time_input_opened: bool,
    pub initialization_completed: bool,
    pub independent_output_validation_completed: bool,
    pub shadow_ledger_created: bool,
    pub shadow_position_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentExecutionAttemptItem {
    pub claim: ControlledShadowExperimentExecutionAttemptClaim,
    pub result: Option<ControlledShadowExperimentExecutionAttemptResult>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompletedControlledShadowExperimentExecutionAttempt {
    pub claim: ControlledShadowExperimentExecutionAttemptClaim,
    pub result: ControlledShadowExperimentExecutionAttemptResult,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub isolation_backend: String,
    pub invocation_endpoint_available: bool,
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_initial_observation_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
    pub attempts: Vec<ControlledShadowExperimentExecutionAttemptItem>,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowExperimentExecutionAttemptReadinessSummary {
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_initial_observation_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
}

#[derive(Debug)]
struct ShadowExecutionFailure {
    message: String,
    current_binary_digest_reverified: bool,
    point_in_time_input_opened: bool,
}

impl ShadowExecutionFailure {
    fn before_binary(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            current_binary_digest_reverified: false,
            point_in_time_input_opened: false,
        }
    }

    fn after_binary(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            current_binary_digest_reverified: true,
            point_in_time_input_opened: false,
        }
    }

    fn after_input(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            current_binary_digest_reverified: true,
            point_in_time_input_opened: true,
        }
    }
}

pub(crate) async fn handle_get_controlled_shadow_experiment_execution_attempts(
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
            warn!(%error, "controlled-shadow execution registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "受控影子单次执行记录暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_controlled_shadow_experiment_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeControlledShadowExperimentOnceRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match invoke_once(&state, &admin.user_id, &isolated_runner_id, request).await {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_experiment_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowExperimentExecutionAttemptReadinessSummary, String> {
    let registry = build_registry(state, Utc::now()).await?;
    Ok(ControlledShadowExperimentExecutionAttemptReadinessSummary {
        invocation_eligible_authorization_count: registry.invocation_eligible_authorization_count,
        claim_count: registry.claim_count,
        completed_attempt_count: registry.completed_attempt_count,
        failed_attempt_count: registry.failed_attempt_count,
        untrusted_initial_observation_count: registry.untrusted_initial_observation_count,
        independent_output_validation_eligible_count: registry
            .independent_output_validation_eligible_count,
        execution_status: registry.execution_status,
    })
}

pub(crate) async fn claimed_controlled_shadow_authorization_review_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    let (claims, _) = read_attempt_records(state).await?;
    Ok(claims
        .into_iter()
        .map(|claim| claim.authorization_review_id)
        .collect())
}

pub(crate) async fn completed_controlled_shadow_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedControlledShadowExperimentExecutionAttempt>, String> {
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
                == ControlledShadowExperimentExecutionAttemptStatus::CompletedWithUntrustedInitialObservation)
                .then_some(CompletedControlledShadowExperimentExecutionAttempt { claim, result })
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowExperimentExecutionAttemptRegistry, String> {
    let eligible = execution_attempt_eligible_controlled_shadow_authorizations(state, now).await?;
    let (claims, results) = read_attempt_records(state).await?;
    let claimed_authorizations = claims
        .iter()
        .map(|value| value.authorization_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let invocation_eligible_authorization_count = eligible
        .iter()
        .filter(|value| !claimed_authorizations.contains(value.review.review_id.as_str()))
        .count();
    let result_index = results
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let attempts = claims
        .iter()
        .cloned()
        .map(|claim| ControlledShadowExperimentExecutionAttemptItem {
            result: result_index
                .get(claim.attempt_id.as_str())
                .cloned()
                .cloned(),
            claim,
        })
        .collect::<Vec<_>>();
    let completed_attempt_count = results
        .iter()
        .filter(|value| {
            value.status
                == ControlledShadowExperimentExecutionAttemptStatus::CompletedWithUntrustedInitialObservation
        })
        .count();
    let failed_attempt_count = results.len().saturating_sub(completed_attempt_count);
    let untrusted_initial_observation_count = results
        .iter()
        .filter(|value| value.untrusted_initial_observation.is_some())
        .count();
    let execution_status = if invocation_eligible_authorization_count > 0 {
        "controlled_shadow_execution_authorized_not_claimed"
    } else if claims.len() > results.len() {
        "controlled_shadow_authorization_claimed_execution_interrupted_fail_closed"
    } else if completed_attempt_count > 0 {
        "untrusted_initial_shadow_observation_waiting_independent_validation"
    } else if failed_attempt_count > 0 {
        "controlled_shadow_authorization_consumed_execution_failed"
    } else {
        "waiting_for_current_unexpired_stage_79_authorization"
    };
    Ok(ControlledShadowExperimentExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        invocation_endpoint_available: true,
        invocation_eligible_authorization_count,
        claim_count: claims.len(),
        completed_attempt_count,
        failed_attempt_count,
        untrusted_initial_observation_count,
        independent_output_validation_eligible_count: untrusted_initial_observation_count,
        execution_status: execution_status.to_string(),
        attempts,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 80 只在 claim 不可变落盘后消费一次精确 Stage 79 授权，复核当前二进制并读取一份点时、只读、内容寻址、白名单输入，执行冻结三种子信号与只做多虚拟组合初始化。输出是不可信观察信封，不写影子账本或持仓，不虚构未来收益，必须另经 Stage 81 独立验证；无模型/指标库、反馈、reward、订单、券商或交易权限。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    admin_id: &str,
    isolated_runner_id: &str,
    request: InvokeControlledShadowExperimentOnceRequest,
) -> Result<ControlledShadowExperimentExecutionAttemptResult, String> {
    verify_confirmations(&request)?;
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let now = Utc::now();
    let authorization = execution_attempt_eligible_controlled_shadow_authorizations(state, now)
        .await?
        .into_iter()
        .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "没有当前、未过期且未消费的 Stage 79 一次性授权".to_string())?;
    verify_expected_binding(&authorization, &request)?;
    let (claims, _) = read_attempt_records(state).await?;
    if claims.iter().any(|claim| {
        claim.authorization_review_id == authorization.review.review_id
            || claim.isolated_runner_id == isolated_runner_id
    }) {
        return Err(
            "这条 Stage 79 授权或 runner 已经被 claim；成功、失败或中断都不得重放".to_string(),
        );
    }
    let claim = new_claim(
        &authorization,
        admin_id,
        &request.expected_input_manifest_sha256,
        now,
    )?;
    write_immutable_json(
        &claim_directory(state, isolated_runner_id).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let execution = execute_after_claim(
        state,
        &authorization,
        &claim.input_manifest_sha256,
        &request.input,
    )
    .await;
    let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let result = match execution {
        Ok(envelope) => successful_result(&claim, envelope, duration_millis)?,
        Err(failure) => failed_result(&claim, failure, duration_millis)?,
    };
    write_immutable_json(
        &result_directory(state, isolated_runner_id).join(format!("{}.json", result.result_id)),
        &result,
    )
    .await?;
    Ok(result)
}

fn verify_confirmations(
    request: &InvokeControlledShadowExperimentOnceRequest,
) -> Result<(), String> {
    if !request.claim_first_single_use_and_failure_consumes_confirmed
        || !request.exact_stage_51_through_stage_79_binding_confirmed
        || !request.current_binary_digest_reverification_after_claim_confirmed
        || !request.point_in_time_read_only_content_addressed_allowlisted_input_confirmed
        || !request.deterministic_three_seed_long_only_initialization_confirmed
        || !request.no_future_performance_or_checkpoint_fabrication_confirmed
        || !request.create_once_untrusted_output_requires_independent_validation_confirmed
        || !request.no_ledger_position_order_broker_or_trading_confirmed
        || !request.no_model_metric_store_feedback_composite_or_reward_confirmed
    {
        return Err("必须确认 claim-first 单次消费、精确绑定、点时输入、不虚构未来结果、不可信输出和全部投资执行禁区".to_string());
    }
    if !valid_sha256(&request.expected_input_manifest_sha256) {
        return Err("expected_input_manifest_sha256 无效".to_string());
    }
    Ok(())
}

fn verify_expected_binding(
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    request: &InvokeControlledShadowExperimentOnceRequest,
) -> Result<(), String> {
    let runner = &authorization.runner;
    let contract = &runner.implementation.implementation_contract;
    if request.expected_authorization_review_id != authorization.review.review_id
        || request.expected_authorization_review_sha256 != authorization.review.review_sha256
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != runner.runner_artifact_sha256
        || request.expected_runner_code_revision != runner.runner_code_revision
        || request.expected_runner_contract_sha256 != runner.runner_contract.contract_sha256
        || request.expected_implementation_sha256 != runner.implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_design_specification_sha256 != contract.design_specification_sha256
        || request.expected_candidate_set_sha256 != contract.candidate_set_sha256
        || request.expected_feature_order_sha256 != contract.feature_order_sha256
        || request.expected_preprocessing_sha256 != contract.preprocessing_sha256
        || request.expected_target_id != contract.target_id
        || request.expected_frozen_candidate_algorithm_id != contract.frozen_candidate_algorithm_id
    {
        return Err(
            "Stage 51–79 授权、工件、实现、设计、候选或特征绑定已经变化，请刷新后重试".to_string(),
        );
    }
    Ok(())
}

fn new_claim(
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    admin_id: &str,
    input_manifest_sha256: &str,
    claimed_at: DateTime<Utc>,
) -> Result<ControlledShadowExperimentExecutionAttemptClaim, String> {
    let runner = &authorization.runner;
    let contract = &runner.implementation.implementation_contract;
    let mut excluded = authorization.review.excluded_prior_actor_ids.clone();
    excluded.push(authorization.review.reviewer_id.clone());
    excluded.sort();
    excluded.dedup();
    if excluded.iter().any(|value| value == admin_id) {
        return Err(
            "Stage 80 执行者必须独立于 Stage 79 复核者和完整 Stage 51–79 责任链".to_string(),
        );
    }
    let mut claim = ControlledShadowExperimentExecutionAttemptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        attempt_id: Uuid::new_v4().simple().to_string(),
        claim_sha256: String::new(),
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        authorization_valid_until: authorization.review.authorization_valid_until,
        isolated_runner_id: runner.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
        runner_code_revision: runner.runner_code_revision.clone(),
        runner_contract_sha256: runner.runner_contract.contract_sha256.clone(),
        implementation_sha256: runner.implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        design_specification_sha256: contract.design_specification_sha256.clone(),
        candidate_set_sha256: contract.candidate_set_sha256.clone(),
        feature_order_sha256: contract.feature_order_sha256.clone(),
        preprocessing_sha256: contract.preprocessing_sha256.clone(),
        target_id: contract.target_id.clone(),
        frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id.clone(),
        exact_random_seeds: contract.random_seeds.clone(),
        input_manifest_sha256: input_manifest_sha256.to_string(),
        claimed_at,
        invoked_by: admin_id.to_string(),
        excluded_prior_actor_ids: excluded,
        executor_independent_from_stage_79_and_complete_prior_chain: true,
        authorization_consumed: true,
        invocation_started: true,
        input_opened: false,
        point_in_time_input_read_allowed: true,
        ephemeral_untrusted_output_write_allowed: true,
        shadow_ledger_write_allowed: false,
        shadow_position_write_allowed: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        training_feedback_allowed: false,
        scalar_reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = claim_fingerprint(&claim)?;
    validate_claim(&claim)?;
    Ok(claim)
}

async fn execute_after_claim(
    state: &AppState,
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    claimed_input_manifest_sha256: &str,
    input: &ControlledShadowPointInTimeInputEnvelope,
) -> Result<ControlledShadowExperimentUntrustedInitialObservationEnvelope, ShadowExecutionFailure> {
    let binary_sha256 = reverify_current_binary_sha256()
        .await
        .map_err(ShadowExecutionFailure::before_binary)?;
    if binary_sha256 != authorization.runner.runner_artifact_sha256 {
        return Err(ShadowExecutionFailure::before_binary(
            "当前执行二进制摘要与 Stage 78–79 独立复核工件不一致",
        ));
    }
    let registration = &authorization
        .runner
        .implementation
        .upstream_design_registration;
    let training_envelope = exact_training_envelope_for_controlled_shadow_execution(
        state,
        &registration.attempt_id,
        &registration.claim_sha256,
        &registration.result_sha256,
    )
    .await
    .map_err(ShadowExecutionFailure::after_binary)?;
    if input.input_manifest_sha256 != claimed_input_manifest_sha256 {
        return Err(ShadowExecutionFailure::after_input(
            "claim 绑定的 input manifest 与实际点时输入不一致",
        ));
    }
    validate_input(authorization, &training_envelope, input)
        .map_err(ShadowExecutionFailure::after_input)?;
    let envelope = project_initial_observation(authorization, &training_envelope, input)
        .map_err(ShadowExecutionFailure::after_input)?;
    let output_bytes = serde_json::to_vec(&envelope)
        .map_err(|error| ShadowExecutionFailure::after_input(error.to_string()))?
        .len() as u64;
    if output_bytes > authorization.runner.runner_contract.maximum_output_bytes {
        return Err(ShadowExecutionFailure::after_input(
            "不可信输出超过 Stage 78 冻结字节上限",
        ));
    }
    Ok(envelope)
}

fn validate_input(
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    training: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    input: &ControlledShadowPointInTimeInputEnvelope,
) -> Result<(), String> {
    let contract = &authorization.runner.implementation.implementation_contract;
    let design = &contract.exact_design_specification;
    if input.schema_version != INPUT_SCHEMA_VERSION
        || input.input_manifest_sha256 != input_fingerprint(input)?
        || input.candidate_set_sha256 != contract.candidate_set_sha256
        || input.feature_order != training.feature_order
        || input.feature_order.len() != EXPECTED_FEATURE_COUNT
        || hash_serializable(&input.feature_order)? != contract.feature_order_sha256
        || input.preprocessing_sha256 != contract.preprocessing_sha256
        || hash_serializable(&training.preprocessing)? != contract.preprocessing_sha256
        || input.benchmark_symbol != design.benchmark_symbol
        || input.captured_at > input.signal_cutoff_at
        || input.expected_next_full_market_session_at <= input.signal_cutoff_at
        || !input.point_in_time_read_only
        || !input.content_addressed
        || !input.allowlisted_sources_only
        || !input.no_retroactive_revision
        || input.sources.is_empty()
        || input.sources.len() > MAX_SOURCE_COUNT
        || input.rows.is_empty()
        || input.rows.len() > MAX_INPUT_ROWS
    {
        return Err(
            "点时输入 manifest、时间、候选、65 项特征、预处理、基准或只读白名单约束无效"
                .to_string(),
        );
    }
    let benchmark = parse_f64_hex(&input.benchmark_adjusted_close_f64_bits_hex)?;
    if !benchmark.is_finite() || benchmark <= 0.0 {
        return Err("基准调整后价格无效".to_string());
    }
    let mut source_hashes = BTreeSet::new();
    for source in &input.sources {
        if source.source_id.trim().is_empty()
            || !valid_sha256(&source.content_sha256)
            || source.available_at > input.signal_cutoff_at
            || !source_hashes.insert(source.content_sha256.clone())
        {
            return Err("输入来源为空、重复、非内容寻址或在信号截止后才可得".to_string());
        }
    }
    let mut symbols = BTreeSet::new();
    for row in &input.rows {
        if !valid_symbol(&row.symbol)
            || !symbols.insert(row.symbol.clone())
            || row.frozen_theme_id.trim().is_empty()
            || row.frozen_theme_id.len() > 80
            || row.security_type != "us_listed_common_stock"
            || row.available_at > input.signal_cutoff_at
            || row.feature_values_f64_bits_hex.len() != EXPECTED_FEATURE_COUNT
            || row.source_content_sha256s.is_empty()
            || !sorted_unique(&row.source_content_sha256s)
            || row
                .source_content_sha256s
                .iter()
                .any(|value| !source_hashes.contains(value))
        {
            return Err(format!("{} 的证券、点时、特征或来源绑定无效", row.symbol));
        }
        let price = parse_f64_hex(&row.adjusted_close_f64_bits_hex)?;
        if !price.is_finite() || price <= 0.0 {
            return Err(format!("{} 的调整后价格无效", row.symbol));
        }
        for value in row.feature_values_f64_bits_hex.iter().flatten() {
            let parsed = parse_f64_hex(value)?;
            if !parsed.is_finite() {
                return Err(format!("{} 包含非有限特征", row.symbol));
            }
        }
    }
    Ok(())
}

fn project_initial_observation(
    authorization: &ApprovedControlledShadowExperimentFirstExecutionAuthorization,
    training: &HistoricalOutcomeTrainingUntrustedArtifactEnvelope,
    input: &ControlledShadowPointInTimeInputEnvelope,
) -> Result<ControlledShadowExperimentUntrustedInitialObservationEnvelope, String> {
    let contract = &authorization.runner.implementation.implementation_contract;
    let design = &contract.exact_design_specification;
    let algorithm = parse_algorithm_id(&contract.frozen_candidate_algorithm_id)?;
    let mut observations = Vec::new();
    for row in input
        .rows
        .iter()
        .filter(|value| value.eligible_in_frozen_universe && value.tradable_at_signal_cutoff)
    {
        let features =
            preprocess_features(&row.feature_values_f64_bits_hex, &training.preprocessing)?;
        let mut seed_predictions = Vec::new();
        let mut sum = 0.0;
        for seed in EXPECTED_SEEDS {
            let artifact = exact_candidate_artifact(training, &algorithm, seed)?;
            let model = exact_target_model(artifact, &contract.target_id)?;
            let prediction = predict_target(model, &training.feature_order, &features)?;
            sum += prediction;
            seed_predictions.push(ControlledShadowSeedPrediction {
                random_seed: seed,
                prediction_f64_bits_hex: f64_hex(prediction),
            });
        }
        observations.push((row, seed_predictions, sum / EXPECTED_SEEDS.len() as f64));
    }
    observations.sort_by(|left, right| {
        right
            .2
            .total_cmp(&left.2)
            .then_with(|| left.0.symbol.cmp(&right.0.symbol))
    });
    let weights = deterministic_virtual_weights(
        &observations
            .iter()
            .map(|(row, _, mean)| (row.frozen_theme_id.clone(), *mean))
            .collect::<Vec<_>>(),
        &design.portfolio_constraints,
    );
    let gross = weights.iter().copied().sum::<u32>();
    let mut virtual_allocations = Vec::new();
    for (index, ((row, seed_predictions, mean), weight)) in
        observations.into_iter().zip(weights).enumerate()
    {
        virtual_allocations.push(ControlledShadowVirtualAllocationObservation {
            symbol: row.symbol.clone(),
            frozen_theme_id: row.frozen_theme_id.clone(),
            adjusted_close_f64_bits_hex: row.adjusted_close_f64_bits_hex.clone(),
            seed_predictions,
            mean_prediction_f64_bits_hex: f64_hex(mean),
            rank: index + 1,
            virtual_target_weight_bps: weight,
            pending_next_session_simulated_execution: weight > 0,
        });
    }
    if gross > design.portfolio_constraints.maximum_gross_exposure_bps
        || 10_000_u32.saturating_sub(gross) < design.portfolio_constraints.minimum_cash_weight_bps
    {
        return Err("冻结只做多组合上限或现金下限被突破".to_string());
    }
    Ok(
        ControlledShadowExperimentUntrustedInitialObservationEnvelope {
            schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
            authorization_review_sha256: authorization.review.review_sha256.clone(),
            isolated_runner_spec_sha256: authorization.runner.isolated_runner_spec_sha256.clone(),
            runner_artifact_sha256: authorization.runner.runner_artifact_sha256.clone(),
            implementation_contract_sha256: contract.contract_sha256.clone(),
            design_specification_sha256: contract.design_specification_sha256.clone(),
            input_manifest_sha256: input.input_manifest_sha256.clone(),
            signal_cutoff_at: input.signal_cutoff_at,
            expected_next_full_market_session_at: input.expected_next_full_market_session_at,
            benchmark_symbol: input.benchmark_symbol.clone(),
            benchmark_adjusted_close_f64_bits_hex: input
                .benchmark_adjusted_close_f64_bits_hex
                .clone(),
            target_id: contract.target_id.clone(),
            frozen_candidate_algorithm_id: contract.frozen_candidate_algorithm_id.clone(),
            exact_random_seeds: contract.random_seeds.clone(),
            virtual_notional_usd: design.portfolio_constraints.virtual_notional_usd,
            virtual_allocations,
            virtual_gross_exposure_bps: gross,
            virtual_cash_weight_bps: 10_000_u32.saturating_sub(gross),
            forward_market_sessions_observed: 0,
            minimum_forward_market_sessions: design
                .observation_contract
                .minimum_forward_market_sessions,
            checkpoint_market_sessions: design
                .observation_contract
                .checkpoint_market_sessions
                .clone(),
            performance_metrics: Vec::new(),
            initialization_only: true,
            output_is_untrusted: true,
            independent_output_validation_completed: false,
            shadow_ledger_created: false,
            shadow_position_written: false,
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

fn deterministic_virtual_weights(
    ranked_theme_and_score: &[(String, f64)],
    constraints: &ControlledShadowExperimentPortfolioConstraints,
) -> Vec<u32> {
    let maximum_positions = usize::from(constraints.maximum_position_count);
    let available_gross = constraints
        .maximum_gross_exposure_bps
        .min(10_000_u32.saturating_sub(constraints.minimum_cash_weight_bps));
    let mut gross = 0_u32;
    let mut theme_weights = BTreeMap::<String, u32>::new();
    ranked_theme_and_score
        .iter()
        .enumerate()
        .map(|(index, (theme_id, score))| {
            let current_theme_weight = *theme_weights.get(theme_id).unwrap_or(&0);
            let weight = if index < maximum_positions && *score > 0.0 {
                constraints
                    .maximum_single_name_weight_bps
                    .min(available_gross.saturating_sub(gross))
                    .min(
                        constraints
                            .maximum_theme_weight_bps
                            .saturating_sub(current_theme_weight),
                    )
            } else {
                0
            };
            gross = gross.saturating_add(weight);
            theme_weights.insert(
                theme_id.clone(),
                current_theme_weight.saturating_add(weight),
            );
            weight
        })
        .collect()
}

fn preprocess_features(
    values: &[Option<String>],
    preprocessing: &[HistoricalOutcomeTrainingFeaturePreprocessingRecord],
) -> Result<Vec<Option<f64>>, String> {
    if values.len() != EXPECTED_FEATURE_COUNT || preprocessing.len() != EXPECTED_FEATURE_COUNT {
        return Err("点时特征或冻结预处理不是精确 65 项".to_string());
    }
    values
        .iter()
        .zip(preprocessing)
        .map(|(value, prep)| match value {
            None if prep.missingness_preserved => Ok(None),
            None => Err(format!("{} 未保留缺失语义", prep.feature_id)),
            Some(value) => {
                let raw = parse_f64_hex(value)?;
                let mean = parse_f64_hex(&prep.train_mean_f64_bits_hex)?;
                let scale = parse_f64_hex(&prep.train_scale_f64_bits_hex)?;
                if !raw.is_finite() || scale <= 0.0 {
                    return Err(format!("{} 的数值或尺度无效", prep.feature_id));
                }
                Ok(Some((raw - mean) / scale))
            }
        })
        .collect()
}

fn successful_result(
    claim: &ControlledShadowExperimentExecutionAttemptClaim,
    envelope: ControlledShadowExperimentUntrustedInitialObservationEnvelope,
    duration_millis: u64,
) -> Result<ControlledShadowExperimentExecutionAttemptResult, String> {
    let output_bytes = serde_json::to_vec(&envelope)
        .map_err(|error| error.to_string())?
        .len();
    let mut result = ControlledShadowExperimentExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: Uuid::new_v4().simple().to_string(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        duration_millis,
        status: ControlledShadowExperimentExecutionAttemptStatus::CompletedWithUntrustedInitialObservation,
        exit_code: 0,
        output_sha256: Some(hash_serializable(&envelope)?),
        output_bytes,
        bounded_error: None,
        untrusted_initial_observation: Some(envelope),
        current_binary_digest_reverified: true,
        point_in_time_input_opened: true,
        initialization_completed: true,
        independent_output_validation_completed: false,
        shadow_ledger_created: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    validate_result(&result, claim)?;
    Ok(result)
}

fn failed_result(
    claim: &ControlledShadowExperimentExecutionAttemptClaim,
    failure: ShadowExecutionFailure,
    duration_millis: u64,
) -> Result<ControlledShadowExperimentExecutionAttemptResult, String> {
    let mut result = ControlledShadowExperimentExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: Uuid::new_v4().simple().to_string(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        duration_millis,
        status: ControlledShadowExperimentExecutionAttemptStatus::FailedAuthorizationConsumed,
        exit_code: 1,
        output_sha256: None,
        output_bytes: 0,
        bounded_error: Some(failure.message.chars().take(MAX_ERROR_CHARS).collect()),
        untrusted_initial_observation: None,
        current_binary_digest_reverified: failure.current_binary_digest_reverified,
        point_in_time_input_opened: failure.point_in_time_input_opened,
        initialization_completed: false,
        independent_output_validation_completed: false,
        shadow_ledger_created: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    validate_result(&result, claim)?;
    Ok(result)
}

fn validate_claim(value: &ControlledShadowExperimentExecutionAttemptClaim) -> Result<(), String> {
    let authority_closed = !value.shadow_ledger_write_allowed
        && !value.shadow_position_write_allowed
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
        || value.claim_sha256 != claim_fingerprint(value)?
        || !valid_sha256(&value.authorization_review_sha256)
        || !valid_sha256(&value.runner_artifact_sha256)
        || !valid_sha256(&value.input_manifest_sha256)
        || value.exact_random_seeds != EXPECTED_SEEDS
        || !sorted_unique(&value.excluded_prior_actor_ids)
        || value
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &value.invoked_by)
        || !value.executor_independent_from_stage_79_and_complete_prior_chain
        || !value.authorization_consumed
        || !value.invocation_started
        || value.input_opened
        || !value.point_in_time_input_read_allowed
        || !value.ephemeral_untrusted_output_write_allowed
        || !authority_closed
    {
        return Err("Stage 80 claim 无效、漂移或越权".to_string());
    }
    Ok(())
}

fn validate_result(
    value: &ControlledShadowExperimentExecutionAttemptResult,
    claim: &ControlledShadowExperimentExecutionAttemptClaim,
) -> Result<(), String> {
    let no_authority = !value.independent_output_validation_completed
        && !value.shadow_ledger_created
        && !value.shadow_position_written
        && !value.order_generated
        && !value.broker_accessed
        && !value.trade_executed;
    let completed = value.status
        == ControlledShadowExperimentExecutionAttemptStatus::CompletedWithUntrustedInitialObservation;
    if value.schema_version != RESULT_SCHEMA_VERSION
        || value.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&value.result_id)
        || value.result_sha256 != result_fingerprint(value)?
        || value.attempt_id != claim.attempt_id
        || value.claim_sha256 != claim.claim_sha256
        || !no_authority
        || completed != value.untrusted_initial_observation.is_some()
        || completed != value.output_sha256.is_some()
        || completed != value.initialization_completed
        || (completed
            && (!value.current_binary_digest_reverified || !value.point_in_time_input_opened))
        || (!completed && value.exit_code == 0)
    {
        return Err("Stage 80 result 无效、漂移或越权".to_string());
    }
    if let Some(envelope) = &value.untrusted_initial_observation {
        if value.output_sha256.as_deref() != Some(hash_serializable(envelope)?.as_str())
            || !envelope.initialization_only
            || !envelope.output_is_untrusted
            || envelope.independent_output_validation_completed
            || envelope.forward_market_sessions_observed != 0
            || !envelope.performance_metrics.is_empty()
            || envelope.shadow_ledger_created
            || envelope.shadow_position_written
            || envelope.model_store_written
            || envelope.metric_store_written
            || envelope.training_feedback_written
            || envelope.scalar_reward_written
            || envelope.order_intent_present
            || envelope.broker_payload_present
            || envelope.trade_executed
        {
            return Err("Stage 80 不可信观察信封包含未来指标或越权载荷".to_string());
        }
    }
    Ok(())
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<ControlledShadowExperimentExecutionAttemptClaim>,
        Vec<ControlledShadowExperimentExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<ControlledShadowExperimentExecutionAttemptClaim>(
        &execution_root(state).join("claims"),
    )
    .await?;
    let results = read_nested_json::<ControlledShadowExperimentExecutionAttemptResult>(
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
            return Err("Stage 80 claim 重复".to_string());
        }
        exact_controlled_shadow_first_execution_authorization_for_audit(
            state,
            &claim.isolated_runner_id,
            &claim.authorization_review_id,
            &claim.authorization_review_sha256,
        )
        .await?
        .ok_or_else(|| "Stage 80 claim 的 Stage 79 授权链无法重开".to_string())?;
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
            .ok_or_else(|| "Stage 80 result 缺少 claim".to_string())?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_attempts.insert(result.attempt_id.clone())
        {
            return Err("Stage 80 result 重复".to_string());
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

fn input_fingerprint(value: &ControlledShadowPointInTimeInputEnvelope) -> Result<String, String> {
    fingerprint_without(value, &["input_manifest_sha256"])
}

fn claim_fingerprint(
    value: &ControlledShadowExperimentExecutionAttemptClaim,
) -> Result<String, String> {
    fingerprint_without(value, &["attempt_id", "claim_sha256"])
}

fn result_fingerprint(
    value: &ControlledShadowExperimentExecutionAttemptResult,
) -> Result<String, String> {
    fingerprint_without(value, &["result_id", "result_sha256"])
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

fn f64_hex(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 12
        && value.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'.' || byte == b'-'
        })
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
    decision_root(state).join("controlled-shadow-experiment-execution-attempts")
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
                return Err("另一个 Stage 80 单次执行正在处理该 runner".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 80 执行锁不可用".to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_first_execution_authorizations::tests::approved_review;

    fn authorization() -> ApprovedControlledShadowExperimentFirstExecutionAuthorization {
        let review = approved_review();
        ApprovedControlledShadowExperimentFirstExecutionAuthorization {
            runner: review.runner.clone(),
            review,
        }
    }

    #[test]
    fn claim_is_content_addressed_consumes_authorization_and_keeps_all_authority_closed() {
        let value = new_claim(
            &authorization(),
            "stage-80-executor",
            &"b".repeat(64),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(value.claim_sha256, claim_fingerprint(&value).unwrap());
        assert!(value.authorization_consumed);
        assert!(!value.input_opened);
        assert!(!value.shadow_ledger_write_allowed);
        assert!(!value.shadow_position_write_allowed);
        assert!(!value.order_generation_allowed);
        assert!(!value.broker_access_allowed);
        assert!(!value.trading_allowed);
    }

    #[test]
    fn executor_must_be_outside_stage_79_and_complete_prior_chain() {
        let authorization = authorization();
        assert!(
            new_claim(
                &authorization,
                &authorization.review.reviewer_id,
                &"b".repeat(64),
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn failed_attempt_consumes_authorization_without_output_or_execution_authority() {
        let claim = new_claim(
            &authorization(),
            "stage-80-executor",
            &"b".repeat(64),
            Utc::now(),
        )
        .unwrap();
        let result = failed_result(
            &claim,
            ShadowExecutionFailure::before_binary("digest mismatch"),
            1,
        )
        .unwrap();
        assert_eq!(
            result.status,
            ControlledShadowExperimentExecutionAttemptStatus::FailedAuthorizationConsumed
        );
        assert!(result.untrusted_initial_observation.is_none());
        assert!(!result.shadow_ledger_created);
        assert!(!result.shadow_position_written);
        assert!(!result.order_generated);
        assert!(!result.trade_executed);
    }

    #[test]
    fn input_manifest_is_content_addressed_and_tampering_is_detected() {
        let mut input = ControlledShadowPointInTimeInputEnvelope {
            schema_version: INPUT_SCHEMA_VERSION.to_string(),
            input_manifest_sha256: String::new(),
            candidate_set_sha256: "a".repeat(64),
            feature_order: (0..EXPECTED_FEATURE_COUNT)
                .map(|index| format!("f{index}"))
                .collect(),
            preprocessing_sha256: "c".repeat(64),
            signal_cutoff_at: Utc::now(),
            captured_at: Utc::now(),
            expected_next_full_market_session_at: Utc::now() + chrono::TimeDelta::days(1),
            benchmark_symbol: "SPY".to_string(),
            benchmark_adjusted_close_f64_bits_hex: f64_hex(500.0),
            sources: Vec::new(),
            rows: Vec::new(),
            point_in_time_read_only: true,
            content_addressed: true,
            allowlisted_sources_only: true,
            no_retroactive_revision: true,
        };
        input.input_manifest_sha256 = input_fingerprint(&input).unwrap();
        assert_eq!(
            input.input_manifest_sha256,
            input_fingerprint(&input).unwrap()
        );
        input.benchmark_symbol = "QQQ".to_string();
        assert_ne!(
            input.input_manifest_sha256,
            input_fingerprint(&input).unwrap()
        );
    }

    #[test]
    fn preprocessing_preserves_missingness_and_uses_frozen_mean_scale() {
        let values = vec![Some(f64_hex(4.0)); EXPECTED_FEATURE_COUNT];
        let preprocessing = (0..EXPECTED_FEATURE_COUNT)
            .map(
                |index| HistoricalOutcomeTrainingFeaturePreprocessingRecord {
                    feature_id: format!("f{index}"),
                    observed_train_count: 10,
                    missing_train_count: 0,
                    train_mean_f64_bits_hex: f64_hex(2.0),
                    train_scale_f64_bits_hex: f64_hex(2.0),
                    missingness_preserved: true,
                },
            )
            .collect::<Vec<_>>();
        let projected = preprocess_features(&values, &preprocessing).unwrap();
        assert!(projected.iter().all(|value| *value == Some(1.0)));
    }

    #[test]
    fn deterministic_weights_enforce_name_theme_gross_cash_and_position_caps() {
        let constraints = authorization()
            .runner
            .implementation
            .implementation_contract
            .exact_design_specification
            .portfolio_constraints;
        let ranked = (0..12)
            .map(|index| {
                (
                    if index < 8 { "memory" } else { "optical" }.to_string(),
                    12.0 - index as f64,
                )
            })
            .collect::<Vec<_>>();
        let weights = deterministic_virtual_weights(&ranked, &constraints);
        assert!(weights.iter().all(|weight| *weight <= 500));
        assert_eq!(weights.iter().take(8).sum::<u32>(), 2_000);
        assert_eq!(weights.iter().skip(8).sum::<u32>(), 1_000);
        assert_eq!(weights.iter().filter(|weight| **weight > 0).count(), 6);
        assert!(weights.iter().sum::<u32>() <= 6_000);
        assert!(10_000 - weights.iter().sum::<u32>() >= 4_000);
    }

    #[test]
    fn successful_output_cannot_claim_future_performance_or_write_positions_orders_or_trades() {
        let claim = new_claim(
            &authorization(),
            "stage-80-executor",
            &"b".repeat(64),
            Utc::now(),
        )
        .unwrap();
        let auth = authorization();
        let design = &auth
            .runner
            .implementation
            .implementation_contract
            .exact_design_specification;
        let envelope = ControlledShadowExperimentUntrustedInitialObservationEnvelope {
            schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
            authorization_review_sha256: auth.review.review_sha256.clone(),
            isolated_runner_spec_sha256: auth.runner.isolated_runner_spec_sha256.clone(),
            runner_artifact_sha256: auth.runner.runner_artifact_sha256.clone(),
            implementation_contract_sha256: auth
                .runner
                .implementation
                .implementation_contract
                .contract_sha256
                .clone(),
            design_specification_sha256: auth
                .runner
                .implementation
                .implementation_contract
                .design_specification_sha256
                .clone(),
            input_manifest_sha256: "b".repeat(64),
            signal_cutoff_at: Utc::now(),
            expected_next_full_market_session_at: Utc::now() + chrono::TimeDelta::days(1),
            benchmark_symbol: "SPY".to_string(),
            benchmark_adjusted_close_f64_bits_hex: f64_hex(500.0),
            target_id: auth
                .runner
                .implementation
                .implementation_contract
                .target_id
                .clone(),
            frozen_candidate_algorithm_id: auth
                .runner
                .implementation
                .implementation_contract
                .frozen_candidate_algorithm_id
                .clone(),
            exact_random_seeds: EXPECTED_SEEDS.to_vec(),
            virtual_notional_usd: design.portfolio_constraints.virtual_notional_usd,
            virtual_allocations: Vec::new(),
            virtual_gross_exposure_bps: 0,
            virtual_cash_weight_bps: 10_000,
            forward_market_sessions_observed: 0,
            minimum_forward_market_sessions: 252,
            checkpoint_market_sessions: vec![21, 63, 126, 252],
            performance_metrics: Vec::new(),
            initialization_only: true,
            output_is_untrusted: true,
            independent_output_validation_completed: false,
            shadow_ledger_created: false,
            shadow_position_written: false,
            model_store_written: false,
            metric_store_written: false,
            training_feedback_written: false,
            scalar_reward_written: false,
            order_intent_present: false,
            broker_payload_present: false,
            trade_executed: false,
        };
        let result = successful_result(&claim, envelope, 1).unwrap();
        assert!(validate_result(&result, &claim).is_ok());
        assert!(
            result
                .untrusted_initial_observation
                .as_ref()
                .unwrap()
                .performance_metrics
                .is_empty()
        );
        assert!(!result.shadow_position_written);
        assert!(!result.order_generated);
        assert!(!result.trade_executed);
    }
}
