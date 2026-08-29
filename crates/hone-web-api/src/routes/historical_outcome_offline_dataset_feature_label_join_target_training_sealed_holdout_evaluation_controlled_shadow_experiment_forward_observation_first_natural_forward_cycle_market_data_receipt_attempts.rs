//! Stage 93 claim-first, create-once read-only market-data receipt attempt.
//!
//! The endpoint persists an immutable claim before issuing any request. It can then make one
//! bounded GET set per server-derived symbol plus SPY and one fixed NYSE calendar GET. The fixed
//! set separately retains split-adjusted, raw-unadjusted and dividend-adjusted prices plus explicit
//! dividend and split events. Raw responses
//! are retained by content hash. A receipt is untrusted source evidence only: it is not parsed into
//! sessions, observations, positions, performance, model metrics, rewards, orders or trades.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Days, NaiveDate, Utc};
use chrono_tz::America::New_York;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_adapter_authorizations::{
    ApprovedControlledShadowMarketDataAdapterAuthorization,
    approved_market_data_adapter_authorizations_for_future_receipt,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_output_validations::independently_validated_forward_observation_initializations_for_first_cycle_review;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_output_validations::independently_validated_controlled_shadow_initial_observations_for_protocol_registration;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-receipt-attempt-registry-v2";
const CLAIM_SCHEMA_VERSION: &str = "hone-controlled-shadow-market-data-receipt-claim-v2";
const RESULT_SCHEMA_VERSION: &str = "hone-controlled-shadow-market-data-receipt-result-v2";
const RECEIPT_SCHEMA_VERSION: &str = "hone-controlled-shadow-untrusted-raw-market-data-receipt-v2";
const POLICY_VERSION: &str = "hone-controlled-shadow-market-data-receipt-v2-explicit-actions-claim-first-single-use-untrusted-raw";
const FMP_ORIGIN: &str = "https://financialmodelingprep.com";
const NYSE_CALENDAR_URL: &str = "https://www.nyse.com/markets/hours-calendars";
const MAX_SUBJECT_SYMBOLS: usize = 10;
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_TOTAL_RESPONSE_BYTES: usize = 64 * 1024 * 1024;
const MAX_TEXT_CHARS: usize = 4_000;
const URL_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

#[derive(Debug, Deserialize)]
pub(crate) struct ClaimAndReadControlledShadowMarketDataReceiptRequest {
    expected_adapter_authorization_sha256: String,
    expected_cycle_claim_sha256: String,
    expected_adapter_spec_sha256: String,
    expected_subject_symbol_set_sha256: String,
    expected_time_window_sha256: String,
    execution_reason: String,
    claim_first_single_use_and_failure_consumes_authorization_confirmed: bool,
    exact_stage_51_through_stage_92_binding_confirmed: bool,
    executor_independent_from_stage_92_and_complete_prior_chain_confirmed: bool,
    fixed_get_https_path_and_query_allowlist_confirmed: bool,
    server_derived_subject_symbols_and_spy_only_confirmed: bool,
    natural_forward_window_content_addressed_no_backfill_confirmed: bool,
    credential_redacted_not_persisted_returned_or_logged_confirmed: bool,
    raw_payload_hashes_timestamps_and_custody_retained_confirmed: bool,
    receipt_untrusted_pending_independent_validation_confirmed: bool,
    no_parsed_calendar_observation_ledger_position_performance_or_model_metric_confirmed: bool,
    no_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CanonicalMarketDataRequestDescriptor {
    pub source_id: String,
    pub method: String,
    pub https_origin: String,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub data_classes: Vec<String>,
    pub canonical_request_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataReceiptClaim {
    pub schema_version: String,
    pub policy_version: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub adapter_authorization_id: String,
    pub adapter_authorization_sha256: String,
    pub cycle_claim_id: String,
    pub cycle_claim_sha256: String,
    pub adapter_spec_sha256: String,
    pub subject_symbols: Vec<String>,
    pub subject_symbol_set_sha256: String,
    pub benchmark_symbol: String,
    pub window_start_date: NaiveDate,
    pub window_end_date: NaiveDate,
    pub time_window_sha256: String,
    pub canonical_requests: Vec<CanonicalMarketDataRequestDescriptor>,
    pub canonical_request_set_sha256: String,
    pub expected_request_count: usize,
    pub claimed_at: DateTime<Utc>,
    pub claimed_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub execution_reason: String,
    pub adapter_authorization_consumed: bool,
    pub claim_first: bool,
    pub create_once: bool,
    pub confirmations_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RawMarketDataSourceReceipt {
    pub source_id: String,
    pub canonical_request_sha256: String,
    pub response_body_sha256: String,
    pub source_document_sha256: String,
    pub response_bytes: usize,
    pub content_type: Option<String>,
    pub http_status: u16,
    pub retrieved_at_utc: DateTime<Utc>,
    pub source_available_at_utc: DateTime<Utc>,
    pub source_available_at_basis: String,
    pub raw_payload_relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UntrustedRawMarketDataReceipt {
    pub schema_version: String,
    pub receipt_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub subject_symbol_set_sha256: String,
    pub time_window_sha256: String,
    pub canonical_request_set_sha256: String,
    pub source_receipts: Vec<RawMarketDataSourceReceipt>,
    pub total_response_bytes: usize,
    pub raw_payload_count: usize,
    pub output_is_untrusted: bool,
    pub independent_validation_completed: bool,
    pub calendar_window_resolved: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowMarketDataReceiptAttemptStatus {
    CompletedWithUntrustedRawMarketDataReceipt,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataReceiptAttemptResult {
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: ControlledShadowMarketDataReceiptAttemptStatus,
    pub bounded_error_code: Option<String>,
    pub untrusted_raw_market_data_receipt: Option<UntrustedRawMarketDataReceipt>,
    pub adapter_authorization_consumed: bool,
    pub market_data_request_made: bool,
    pub market_data_accessed: bool,
    pub calendar_window_resolved: bool,
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
pub(crate) struct ControlledShadowMarketDataReceiptCandidate {
    pub adapter_authorization_id: String,
    pub adapter_authorization_sha256: String,
    pub cycle_claim_sha256: String,
    pub adapter_spec_sha256: String,
    pub subject_symbols: Vec<String>,
    pub subject_symbol_set_sha256: String,
    pub benchmark_symbol: String,
    pub window_start_date: NaiveDate,
    pub window_end_date: NaiveDate,
    pub time_window_sha256: String,
    pub expected_request_count: usize,
    pub executor_excluded_actor_ids: Vec<String>,
    pub fmp_configured: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataReceiptAttemptItem {
    pub claim: ControlledShadowMarketDataReceiptClaim,
    pub result: Option<ControlledShadowMarketDataReceiptAttemptResult>,
    pub interrupted_after_claim: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataReceiptAttemptRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub invocation_endpoint_available: bool,
    pub eligible_authorizations: Vec<ControlledShadowMarketDataReceiptCandidate>,
    pub items: Vec<ControlledShadowMarketDataReceiptAttemptItem>,
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_untrusted_receipt_count: usize,
    pub failed_authorization_consumed_count: usize,
    pub interrupted_authorization_consumed_count: usize,
    pub independent_validation_eligible_count: usize,
    pub receipt_status: String,
    pub calendar_window_resolved: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowMarketDataReceiptReadinessSummary {
    pub invocation_eligible_authorization_count: usize,
    pub claim_count: usize,
    pub completed_untrusted_receipt_count: usize,
    pub failed_authorization_consumed_count: usize,
    pub interrupted_authorization_consumed_count: usize,
    pub independent_validation_eligible_count: usize,
    pub receipt_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CompletedControlledShadowMarketDataReceiptAttempt {
    pub claim: ControlledShadowMarketDataReceiptClaim,
    pub result: ControlledShadowMarketDataReceiptAttemptResult,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_receipt_attempts(
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
            warn!(%error, "market-data receipt registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "只读行情原始收据表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_claim_and_read_controlled_shadow_market_data_receipt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(adapter_authorization_id): AxumPath<String>,
    Json(request): Json<ClaimAndReadControlledShadowMarketDataReceiptRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match claim_and_read_once(
        &state,
        &admin.user_id,
        &adapter_authorization_id,
        request,
        Utc::now(),
    )
    .await
    {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => Json(value).into_response(),
            Err(_) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "收据已写入，但刷新注册表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_receipt_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataReceiptReadinessSummary, String> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(ControlledShadowMarketDataReceiptReadinessSummary {
        invocation_eligible_authorization_count: value.invocation_eligible_authorization_count,
        claim_count: value.claim_count,
        completed_untrusted_receipt_count: value.completed_untrusted_receipt_count,
        failed_authorization_consumed_count: value.failed_authorization_consumed_count,
        interrupted_authorization_consumed_count: value.interrupted_authorization_consumed_count,
        independent_validation_eligible_count: value.independent_validation_eligible_count,
        receipt_status: value.receipt_status,
    })
}

pub(crate) async fn claimed_market_data_receipt_adapter_authorization_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    Ok(read_claims(state)
        .await?
        .into_iter()
        .map(|value| value.adapter_authorization_id)
        .collect())
}

pub(crate) async fn completed_market_data_receipt_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedControlledShadowMarketDataReceiptAttempt>, String> {
    let claims = read_claims(state).await?;
    let results = read_results(state).await?;
    let result_index = results
        .into_iter()
        .map(|value| (value.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut completed = Vec::new();
    for claim in claims {
        let Some(result) = result_index.get(&claim.attempt_id).cloned() else {
            continue;
        };
        if !matches!(
            result.status,
            ControlledShadowMarketDataReceiptAttemptStatus::CompletedWithUntrustedRawMarketDataReceipt
        ) {
            continue;
        }
        let receipt = result
            .untrusted_raw_market_data_receipt
            .as_ref()
            .ok_or_else(|| "Stage 93 完成结果缺少原始行情收据".to_string())?;
        validate_receipt(receipt, &claim)?;
        completed.push(CompletedControlledShadowMarketDataReceiptAttempt { claim, result });
    }
    completed.sort_by(|left, right| left.claim.claimed_at.cmp(&right.claim.claimed_at));
    Ok(completed)
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataReceiptAttemptRegistry, String> {
    let claims = read_claims(state).await?;
    let results = read_results(state).await?;
    let claim_index = claims
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    for result in &results {
        let claim = claim_index
            .get(result.attempt_id.as_str())
            .ok_or_else(|| "Stage 93 结果缺少对应 claim".to_string())?;
        if result.claim_sha256 != claim.claim_sha256 {
            return Err("Stage 93 结果与 claim 哈希绑定漂移".into());
        }
        if let Some(receipt) = &result.untrusted_raw_market_data_receipt {
            validate_receipt(receipt, claim)?;
            validate_raw_payload_custody(state, receipt, claim).await?;
        }
    }
    let result_index = results
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let consumed = claims
        .iter()
        .map(|value| value.adapter_authorization_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut eligible_authorizations = Vec::new();
    for approved in
        approved_market_data_adapter_authorizations_for_future_receipt(state, now).await?
    {
        if !consumed.contains(approved.authorization.adapter_authorization_id.as_str()) {
            eligible_authorizations.push(build_candidate(state, &approved, now).await?);
        }
    }
    let items = claims
        .into_iter()
        .map(|claim| {
            let result = result_index
                .get(claim.attempt_id.as_str())
                .cloned()
                .cloned();
            ControlledShadowMarketDataReceiptAttemptItem {
                interrupted_after_claim: result.is_none(),
                claim,
                result,
            }
        })
        .collect::<Vec<_>>();
    let completed_untrusted_receipt_count = results.iter().filter(|value| matches!(value.status, ControlledShadowMarketDataReceiptAttemptStatus::CompletedWithUntrustedRawMarketDataReceipt)).count();
    let failed_authorization_consumed_count = results
        .iter()
        .filter(|value| {
            matches!(
                value.status,
                ControlledShadowMarketDataReceiptAttemptStatus::FailedAuthorizationConsumed
            )
        })
        .count();
    let interrupted_authorization_consumed_count = items
        .iter()
        .filter(|value| value.interrupted_after_claim)
        .count();
    let receipt_status = if completed_untrusted_receipt_count > 0 {
        "untrusted_raw_market_data_receipt_waiting_for_independent_validation"
    } else if failed_authorization_consumed_count + interrupted_authorization_consumed_count > 0 {
        "authorization_consumed_without_complete_receipt"
    } else if !eligible_authorizations.is_empty() {
        "ready_for_claim_first_single_read"
    } else {
        "waiting_for_active_stage_92_authorization_and_natural_forward_window"
    };
    Ok(ControlledShadowMarketDataReceiptAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        invocation_endpoint_available: true,
        invocation_eligible_authorization_count: eligible_authorizations.len(),
        claim_count: items.len(),
        completed_untrusted_receipt_count,
        failed_authorization_consumed_count,
        interrupted_authorization_consumed_count,
        independent_validation_eligible_count: completed_untrusted_receipt_count,
        eligible_authorizations,
        items,
        receipt_status: receipt_status.into(),
        calendar_window_resolved: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 93 仅在不可覆盖 claim 写入后，对服务端冻结的标的集合、SPY 与 NYSE 日历各执行一次固定 HTTPS GET，并以内容哈希 create-once 保管原始载荷。成功结果仍是待独立验证的外部原始证据，不解析交易日、不生成观察/持仓/绩效/模型指标，也不开放训练、reward、订单、券商或交易能力；失败或进程中断同样永久消耗授权。".into(),
    })
}

async fn build_candidate(
    state: &AppState,
    approved: &ApprovedControlledShadowMarketDataAdapterAuthorization,
    now: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataReceiptCandidate, String> {
    let subject_symbols = derive_subject_symbols(state, &approved.claim.validation_sha256).await?;
    let subject_symbol_set_sha256 = fingerprint(&subject_symbols)?;
    let window_start_date = approved
        .authorization
        .authorized_not_before
        .with_timezone(&New_York)
        .date_naive()
        .checked_add_days(Days::new(1))
        .ok_or_else(|| "Stage 93 起始日期溢出".to_string())?;
    let window_end_date = now.with_timezone(&New_York).date_naive();
    if window_end_date < window_start_date {
        return Err("Stage 93 尚未进入授权后的首个自然日前向窗口".into());
    }
    let time_window_sha256 = fingerprint(&(window_start_date, window_end_date))?;
    let request_count =
        canonical_requests(&subject_symbols, window_start_date, window_end_date)?.len();
    let mut excluded = approved.authorization.excluded_prior_actor_ids.clone();
    excluded.push(approved.authorization.reviewer_id.clone());
    excluded.sort();
    excluded.dedup();
    Ok(ControlledShadowMarketDataReceiptCandidate {
        adapter_authorization_id: approved.authorization.adapter_authorization_id.clone(),
        adapter_authorization_sha256: approved.authorization.adapter_authorization_sha256.clone(),
        cycle_claim_sha256: approved.claim.cycle_claim_sha256.clone(),
        adapter_spec_sha256: approved
            .authorization
            .adapter_specification
            .adapter_spec_sha256
            .clone(),
        subject_symbols,
        subject_symbol_set_sha256,
        benchmark_symbol: "SPY".into(),
        window_start_date,
        window_end_date,
        time_window_sha256,
        expected_request_count: request_count,
        executor_excluded_actor_ids: excluded,
        fmp_configured: state.core.config.fmp.effective_key_pool().first().is_some()
            && normalized_fmp_origin(&state.core.config.fmp.base_url) == FMP_ORIGIN,
    })
}

async fn derive_subject_symbols(
    state: &AppState,
    stage_89_validation_sha256: &str,
) -> Result<Vec<String>, String> {
    let forward =
        independently_validated_forward_observation_initializations_for_first_cycle_review(state)
            .await?
            .into_iter()
            .find(|value| value.validation.validation_sha256 == stage_89_validation_sha256)
            .ok_or_else(|| "Stage 93 无法绑定 Stage 89 独立验证".to_string())?;
    let initial_sha = forward.validation.initial_observation_validation_sha256;
    let initial =
        independently_validated_controlled_shadow_initial_observations_for_protocol_registration(
            state,
        )
        .await?
        .into_iter()
        .find(|value| value.validation.validation_sha256 == initial_sha)
        .ok_or_else(|| "Stage 93 无法绑定 Stage 81 初始影子组合".to_string())?;
    let envelope = initial
        .attempt
        .result
        .untrusted_initial_observation
        .as_ref()
        .ok_or_else(|| "Stage 81 初始影子组合缺少未信任观察载荷".to_string())?;
    let mut symbols = envelope
        .virtual_allocations
        .iter()
        .map(|value| value.symbol.trim().to_ascii_uppercase())
        .collect::<Vec<_>>();
    symbols.sort();
    symbols.dedup();
    if symbols.is_empty()
        || symbols.len() > MAX_SUBJECT_SYMBOLS
        || symbols.iter().any(|value| !valid_symbol(value))
    {
        return Err("Stage 93 服务端推导的标的集合为空、过大或包含非法代码".into());
    }
    Ok(symbols)
}

async fn claim_and_read_once(
    state: &AppState,
    executor_id: &str,
    adapter_authorization_id: &str,
    request: ClaimAndReadControlledShadowMarketDataReceiptRequest,
    now: DateTime<Utc>,
) -> Result<(), String> {
    if !valid_id(adapter_authorization_id) {
        return Err("Stage 92 adapter authorization ID 无效".into());
    }
    let _lock = acquire_lock(state, adapter_authorization_id).await?;
    if read_claims(state)
        .await?
        .iter()
        .any(|value| value.adapter_authorization_id == adapter_authorization_id)
    {
        return Err("该 Stage 92 授权已被不可覆盖地消耗".into());
    }
    let approved = approved_market_data_adapter_authorizations_for_future_receipt(state, now)
        .await?
        .into_iter()
        .find(|value| value.authorization.adapter_authorization_id == adapter_authorization_id)
        .ok_or_else(|| "Stage 92 授权不存在、未批准或已过期".to_string())?;
    let candidate = build_candidate(state, &approved, now).await?;
    validate_request(&candidate, &request, executor_id)?;
    let key = state
        .core
        .config
        .fmp
        .effective_key_pool()
        .first()
        .map(str::to_string)
        .ok_or_else(|| "FMP API Key 未配置；未写 claim、未发起请求".to_string())?;
    if normalized_fmp_origin(&state.core.config.fmp.base_url) != FMP_ORIGIN {
        return Err("FMP base_url 不是 Stage 92 固定来源；未写 claim、未发起请求".into());
    }
    let canonical = canonical_requests(
        &candidate.subject_symbols,
        candidate.window_start_date,
        candidate.window_end_date,
    )?;
    let mut excluded = candidate.executor_excluded_actor_ids.clone();
    excluded.sort();
    excluded.dedup();
    let mut claim = ControlledShadowMarketDataReceiptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        attempt_id: String::new(),
        claim_sha256: String::new(),
        adapter_authorization_id: candidate.adapter_authorization_id,
        adapter_authorization_sha256: candidate.adapter_authorization_sha256,
        cycle_claim_id: approved.claim.cycle_claim_id,
        cycle_claim_sha256: candidate.cycle_claim_sha256,
        adapter_spec_sha256: candidate.adapter_spec_sha256,
        subject_symbols: candidate.subject_symbols,
        subject_symbol_set_sha256: candidate.subject_symbol_set_sha256,
        benchmark_symbol: "SPY".into(),
        window_start_date: candidate.window_start_date,
        window_end_date: candidate.window_end_date,
        time_window_sha256: candidate.time_window_sha256,
        canonical_request_set_sha256: fingerprint(&canonical)?,
        expected_request_count: canonical.len(),
        canonical_requests: canonical,
        claimed_at: now,
        claimed_by: executor_id.into(),
        excluded_prior_actor_ids: excluded,
        execution_reason: bounded_required(&request.execution_reason, "执行理由")?,
        adapter_authorization_consumed: true,
        claim_first: true,
        create_once: true,
        confirmations_complete: confirmations_complete(&request),
    };
    claim.claim_sha256 = fingerprint_without(&claim, &["attempt_id", "claim_sha256"])?;
    claim.attempt_id = claim.claim_sha256[..32].into();
    validate_claim(&claim)?;
    write_immutable_json(
        &claim_directory(state).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let outcome = fetch_all_raw_payloads(state, &claim, &key).await;
    let result = match outcome {
        Ok(receipt) => new_result(&claim, now, started.elapsed(), Some(receipt), None)?,
        Err(code) => new_result(&claim, now, started.elapsed(), None, Some(code))?,
    };
    write_immutable_json(
        &result_directory(state).join(format!("{}.json", result.result_id)),
        &result,
    )
    .await?;
    Ok(())
}

fn validate_request(
    candidate: &ControlledShadowMarketDataReceiptCandidate,
    request: &ClaimAndReadControlledShadowMarketDataReceiptRequest,
    executor_id: &str,
) -> Result<(), String> {
    if request.expected_adapter_authorization_sha256 != candidate.adapter_authorization_sha256
        || request.expected_cycle_claim_sha256 != candidate.cycle_claim_sha256
        || request.expected_adapter_spec_sha256 != candidate.adapter_spec_sha256
        || request.expected_subject_symbol_set_sha256 != candidate.subject_symbol_set_sha256
        || request.expected_time_window_sha256 != candidate.time_window_sha256
    {
        return Err("Stage 93 预期绑定已漂移".into());
    }
    if executor_id.trim().is_empty()
        || candidate
            .executor_excluded_actor_ids
            .iter()
            .any(|value| value == executor_id)
    {
        return Err("Stage 93 执行人与 Stage 92 或完整上游责任链不独立".into());
    }
    if !confirmations_complete(request) {
        return Err("Stage 93 风险确认项不完整".into());
    }
    Ok(())
}

fn confirmations_complete(value: &ClaimAndReadControlledShadowMarketDataReceiptRequest) -> bool {
    value.claim_first_single_use_and_failure_consumes_authorization_confirmed
        && value.exact_stage_51_through_stage_92_binding_confirmed
        && value.executor_independent_from_stage_92_and_complete_prior_chain_confirmed
        && value.fixed_get_https_path_and_query_allowlist_confirmed
        && value.server_derived_subject_symbols_and_spy_only_confirmed
        && value.natural_forward_window_content_addressed_no_backfill_confirmed
        && value.credential_redacted_not_persisted_returned_or_logged_confirmed
        && value.raw_payload_hashes_timestamps_and_custody_retained_confirmed
        && value.receipt_untrusted_pending_independent_validation_confirmed
        && value
            .no_parsed_calendar_observation_ledger_position_performance_or_model_metric_confirmed
        && value.no_training_feedback_reward_order_broker_or_trading_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn canonical_requests(
    symbols: &[String],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketDataRequestDescriptor>, String> {
    let mut requested = symbols.to_vec();
    if !requested.iter().any(|value| value == "SPY") {
        requested.push("SPY".into());
    }
    requested.sort();
    requested.dedup();
    let mut output = Vec::new();
    for symbol in requested {
        for (source_kind, path, data_classes) in [
            (
                "split_adjusted_price",
                "/stable/historical-price-eod/full",
                vec!["ohlcv", "split_adjusted_close"],
            ),
            (
                "raw_unadjusted_price",
                "/stable/historical-price-eod/non-split-adjusted",
                vec!["ohlcv", "raw_unadjusted_close"],
            ),
            (
                "dividend_adjusted_price",
                "/stable/historical-price-eod/dividend-adjusted",
                vec!["dividend_adjusted_close", "ohlcv"],
            ),
            ("dividends", "/stable/dividends", vec!["dividend_event"]),
            ("splits", "/stable/splits", vec!["split_event"]),
        ] {
            let mut descriptor = CanonicalMarketDataRequestDescriptor {
                source_id: format!("fmp_{source_kind}_{symbol}"),
                method: "GET".into(),
                https_origin: FMP_ORIGIN.into(),
                path: path.into(),
                query: vec![
                    ("apikey".into(), "REDACTED".into()),
                    ("from".into(), from.to_string()),
                    ("symbol".into(), symbol.clone()),
                    ("to".into(), to.to_string()),
                ],
                data_classes: data_classes.into_iter().map(str::to_string).collect(),
                canonical_request_sha256: String::new(),
            };
            descriptor.canonical_request_sha256 =
                fingerprint_without(&descriptor, &["canonical_request_sha256"])?;
            output.push(descriptor);
        }
    }
    let mut calendar = CanonicalMarketDataRequestDescriptor {
        source_id: "nyse_official_us_market_calendar".into(),
        method: "GET".into(),
        https_origin: "https://www.nyse.com".into(),
        path: "/markets/hours-calendars".into(),
        query: Vec::new(),
        data_classes: vec!["official_us_market_calendar".into()],
        canonical_request_sha256: String::new(),
    };
    calendar.canonical_request_sha256 =
        fingerprint_without(&calendar, &["canonical_request_sha256"])?;
    output.push(calendar);
    output.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    Ok(output)
}

async fn fetch_all_raw_payloads(
    state: &AppState,
    claim: &ControlledShadowMarketDataReceiptClaim,
    key: &str,
) -> Result<UntrustedRawMarketDataReceipt, String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(StdDuration::from_secs(
            state.core.config.fmp.timeout.clamp(5, 120),
        ))
        .build()
        .map_err(|_| "http_client_configuration_failed".to_string())?;
    let mut receipts = Vec::new();
    let mut total = 0usize;
    for descriptor in &claim.canonical_requests {
        let url = wire_url(descriptor, key)?;
        let mut response = client
            .get(url)
            .send()
            .await
            .map_err(|_| "http_request_failed".to_string())?;
        let status = response.status();
        if status.is_redirection() {
            return Err("http_redirect_rejected".into());
        }
        if !status.is_success() {
            return Err("http_status_not_success".into());
        }
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.chars().take(200).collect::<String>());
        let retrieved_at = Utc::now();
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| "http_response_stream_failed".to_string())?
        {
            if body.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES
                || total.saturating_add(chunk.len()) > MAX_TOTAL_RESPONSE_BYTES
            {
                return Err("http_response_size_limit_exceeded".into());
            }
            total += chunk.len();
            body.extend_from_slice(&chunk);
        }
        let body_sha = sha256_bytes(&body);
        let relative = format!(
            "payloads/{}/{}-{}.bin",
            claim.attempt_id, descriptor.source_id, body_sha
        );
        write_immutable_bytes(&receipt_root(state).join(&relative), &body)
            .await
            .map_err(|_| "raw_payload_create_once_write_failed".to_string())?;
        receipts.push(RawMarketDataSourceReceipt {
            source_id: descriptor.source_id.clone(),
            canonical_request_sha256: descriptor.canonical_request_sha256.clone(),
            response_body_sha256: body_sha.clone(),
            source_document_sha256: body_sha,
            response_bytes: body.len(),
            content_type,
            http_status: status.as_u16(),
            retrieved_at_utc: retrieved_at,
            source_available_at_utc: retrieved_at,
            source_available_at_basis: "retrieval_time_fallback_unverified".into(),
            raw_payload_relative_path: relative,
        });
    }
    let mut receipt = UntrustedRawMarketDataReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION.into(),
        receipt_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        subject_symbol_set_sha256: claim.subject_symbol_set_sha256.clone(),
        time_window_sha256: claim.time_window_sha256.clone(),
        canonical_request_set_sha256: claim.canonical_request_set_sha256.clone(),
        raw_payload_count: receipts.len(),
        source_receipts: receipts,
        total_response_bytes: total,
        output_is_untrusted: true,
        independent_validation_completed: false,
        calendar_window_resolved: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
    };
    receipt.receipt_sha256 = fingerprint_without(&receipt, &["receipt_sha256"])?;
    validate_receipt(&receipt, claim)?;
    Ok(receipt)
}

fn wire_url(
    descriptor: &CanonicalMarketDataRequestDescriptor,
    key: &str,
) -> Result<String, String> {
    if descriptor.method != "GET" {
        return Err("request_contract_drift".into());
    }
    if descriptor.source_id == "nyse_official_us_market_calendar" {
        if descriptor.https_origin != "https://www.nyse.com"
            || descriptor.path != "/markets/hours-calendars"
            || !descriptor.query.is_empty()
        {
            return Err("request_contract_drift".into());
        }
        return Ok(NYSE_CALENDAR_URL.into());
    }
    let allowed_paths = [
        "/stable/dividends",
        "/stable/historical-price-eod/dividend-adjusted",
        "/stable/historical-price-eod/full",
        "/stable/historical-price-eod/non-split-adjusted",
        "/stable/splits",
    ];
    if descriptor.https_origin != FMP_ORIGIN || !allowed_paths.contains(&descriptor.path.as_str()) {
        return Err("request_contract_drift".into());
    }
    if descriptor.query.len() != 4
        || descriptor
            .query
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            != ["apikey", "from", "symbol", "to"]
        || descriptor.query[0].1 != "REDACTED"
        || !valid_symbol(&descriptor.query[2].1)
        || NaiveDate::parse_from_str(&descriptor.query[1].1, "%Y-%m-%d").is_err()
        || NaiveDate::parse_from_str(&descriptor.query[3].1, "%Y-%m-%d").is_err()
    {
        return Err("request_contract_drift".into());
    }
    let query = descriptor
        .query
        .iter()
        .map(|(name, value)| {
            let value = if name == "apikey" { key } else { value };
            format!(
                "{}={}",
                utf8_percent_encode(name, URL_ENCODE_SET),
                utf8_percent_encode(value, URL_ENCODE_SET)
            )
        })
        .collect::<Vec<_>>()
        .join("&");
    Ok(format!("{FMP_ORIGIN}{}?{query}", descriptor.path))
}

fn normalized_fmp_origin(base_url: &str) -> String {
    let mut base = base_url.trim_end_matches('/').to_string();
    for suffix in ["/api/v3", "/api"] {
        if let Some(stripped) = base.strip_suffix(suffix) {
            base = stripped.to_string();
            break;
        }
    }
    base.trim_end_matches('/').to_string()
}

fn new_result(
    claim: &ControlledShadowMarketDataReceiptClaim,
    started_at: DateTime<Utc>,
    duration: StdDuration,
    receipt: Option<UntrustedRawMarketDataReceipt>,
    error: Option<String>,
) -> Result<ControlledShadowMarketDataReceiptAttemptResult, String> {
    let completed = receipt.is_some();
    let mut result = ControlledShadowMarketDataReceiptAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: started_at + chrono::Duration::from_std(duration).unwrap_or_default(),
        duration_millis: duration.as_millis().min(u64::MAX as u128) as u64,
        status: if completed {
            ControlledShadowMarketDataReceiptAttemptStatus::CompletedWithUntrustedRawMarketDataReceipt
        } else {
            ControlledShadowMarketDataReceiptAttemptStatus::FailedAuthorizationConsumed
        },
        bounded_error_code: error,
        untrusted_raw_market_data_receipt: receipt,
        adapter_authorization_consumed: true,
        market_data_request_made: true,
        market_data_accessed: completed,
        calendar_window_resolved: false,
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
    result.result_sha256 = fingerprint_without(&result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].into();
    validate_result(&result)?;
    Ok(result)
}

fn validate_claim(value: &ControlledShadowMarketDataReceiptClaim) -> Result<(), String> {
    let expected_requests = canonical_requests(
        &value.subject_symbols,
        value.window_start_date,
        value.window_end_date,
    )?;
    if value.schema_version != CLAIM_SCHEMA_VERSION
        || value.policy_version != POLICY_VERSION
        || !valid_id(&value.attempt_id)
        || !valid_sha256(&value.claim_sha256)
        || value.attempt_id != value.claim_sha256[..32]
        || value.claim_sha256 != fingerprint_without(value, &["attempt_id", "claim_sha256"])?
        || !valid_id(&value.adapter_authorization_id)
        || !valid_sha256(&value.adapter_authorization_sha256)
        || !valid_id(&value.cycle_claim_id)
        || !valid_sha256(&value.cycle_claim_sha256)
        || !valid_sha256(&value.adapter_spec_sha256)
        || value.subject_symbols.is_empty()
        || value.subject_symbols.len() > MAX_SUBJECT_SYMBOLS
        || !sorted_unique(&value.subject_symbols)
        || value.subject_symbol_set_sha256 != fingerprint(&value.subject_symbols)?
        || value.benchmark_symbol != "SPY"
        || value.window_end_date < value.window_start_date
        || value.time_window_sha256
            != fingerprint(&(value.window_start_date, value.window_end_date))?
        || value.canonical_requests != expected_requests
        || value.canonical_request_set_sha256 != fingerprint(&value.canonical_requests)?
        || value.expected_request_count != value.canonical_requests.len()
        || value.claimed_by.trim().is_empty()
        || value.excluded_prior_actor_ids.contains(&value.claimed_by)
        || !value.adapter_authorization_consumed
        || !value.claim_first
        || !value.create_once
        || !value.confirmations_complete
    {
        return Err("Stage 93 claim 无效、漂移或越权".into());
    }
    Ok(())
}

fn validate_receipt(
    value: &UntrustedRawMarketDataReceipt,
    claim: &ControlledShadowMarketDataReceiptClaim,
) -> Result<(), String> {
    let expected_request_hashes = claim
        .canonical_requests
        .iter()
        .map(|item| item.canonical_request_sha256.as_str())
        .collect::<BTreeSet<_>>();
    let received_request_hashes = value
        .source_receipts
        .iter()
        .map(|item| item.canonical_request_sha256.as_str())
        .collect::<BTreeSet<_>>();
    if value.schema_version != RECEIPT_SCHEMA_VERSION
        || value.receipt_sha256 != fingerprint_without(value, &["receipt_sha256"])?
        || value.attempt_id != claim.attempt_id
        || value.claim_sha256 != claim.claim_sha256
        || value.source_receipts.len() != claim.expected_request_count
        || value.raw_payload_count != value.source_receipts.len()
        || value.total_response_bytes
            != value
                .source_receipts
                .iter()
                .map(|item| item.response_bytes)
                .sum::<usize>()
        || expected_request_hashes != received_request_hashes
        || !value.output_is_untrusted
        || value.independent_validation_completed
        || value.calendar_window_resolved
        || value.parsed_market_rows_created
        || value.forward_observation_started
        || value.source_receipts.iter().any(|item| {
            !valid_sha256(&item.response_body_sha256)
                || item.response_body_sha256 != item.source_document_sha256
                || item.response_bytes > MAX_RESPONSE_BYTES
        })
    {
        return Err("Stage 93 原始收据无效、漂移或越权".into());
    }
    Ok(())
}

fn validate_result(value: &ControlledShadowMarketDataReceiptAttemptResult) -> Result<(), String> {
    let completed = matches!(
        value.status,
        ControlledShadowMarketDataReceiptAttemptStatus::CompletedWithUntrustedRawMarketDataReceipt
    );
    if value.schema_version != RESULT_SCHEMA_VERSION
        || value.policy_version != POLICY_VERSION
        || !valid_id(&value.result_id)
        || !valid_sha256(&value.result_sha256)
        || value.result_id != value.result_sha256[..32]
        || value.result_sha256 != fingerprint_without(value, &["result_id", "result_sha256"])?
        || completed != value.untrusted_raw_market_data_receipt.is_some()
        || completed == value.bounded_error_code.is_some()
        || value
            .bounded_error_code
            .as_ref()
            .is_some_and(|code| !allowed_error_code(code))
        || !value.adapter_authorization_consumed
        || !value.market_data_request_made
        || value.market_data_accessed != completed
        || value.calendar_window_resolved
        || value.forward_observation_started
        || value.ledger_created
        || value.position_written
        || value.performance_metric_written
        || value.model_store_written
        || value.metric_store_written
        || value.training_feedback_authorized
        || value.reward_authorized
        || value.order_generation_authorized
        || value.broker_access_authorized
        || value.trading_authorized
    {
        return Err("Stage 93 终态结果无效、漂移或越权".into());
    }
    Ok(())
}

async fn read_claims(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataReceiptClaim>, String> {
    let values = read_json_directory(claim_directory(state), validate_claim).await?;
    if !unique_by(&values, |value| value.attempt_id.clone())
        || !unique_by(&values, |value| value.adapter_authorization_id.clone())
    {
        return Err("Stage 93 claim 重复或授权重放".into());
    }
    Ok(values)
}
async fn read_results(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataReceiptAttemptResult>, String> {
    let values = read_json_directory(result_directory(state), validate_result).await?;
    if !unique_by(&values, |value| value.result_id.clone())
        || !unique_by(&values, |value| value.attempt_id.clone())
    {
        return Err("Stage 93 结果重复或 attempt 重放".into());
    }
    Ok(values)
}
async fn read_json_directory<T>(
    directory_path: PathBuf,
    validate: fn(&T) -> Result<(), String>,
) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de> + Clone,
{
    let mut directory = match tokio::fs::read_dir(directory_path).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let value = serde_json::from_slice::<T>(
            &tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        validate(&value)?;
        output.push(value);
    }
    Ok(output)
}

fn fingerprint<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}
fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 93 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    fingerprint(&value)
}
fn sha256_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}为空或过长"))
    } else {
        Ok(value.into())
    }
}
fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn valid_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
}
fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}
fn unique_by<T, F>(values: &[T], mut key: F) -> bool
where
    F: FnMut(&T) -> String,
{
    let mut keys = BTreeSet::new();
    values.iter().all(|value| keys.insert(key(value)))
}
fn allowed_error_code(value: &str) -> bool {
    matches!(
        value,
        "http_client_configuration_failed"
            | "http_request_failed"
            | "http_redirect_rejected"
            | "http_status_not_success"
            | "http_response_stream_failed"
            | "http_response_size_limit_exceeded"
            | "raw_payload_create_once_write_failed"
            | "request_contract_drift"
    )
}

async fn validate_raw_payload_custody(
    state: &AppState,
    receipt: &UntrustedRawMarketDataReceipt,
    claim: &ControlledShadowMarketDataReceiptClaim,
) -> Result<(), String> {
    for source in &receipt.source_receipts {
        let expected_relative = format!(
            "payloads/{}/{}-{}.bin",
            claim.attempt_id, source.source_id, source.response_body_sha256
        );
        if source.raw_payload_relative_path != expected_relative {
            return Err("Stage 93 原始载荷路径不符合内容寻址合同".into());
        }
        let bytes = tokio::fs::read(receipt_root(state).join(&expected_relative))
            .await
            .map_err(|_| "Stage 93 原始载荷缺失".to_string())?;
        if bytes.len() != source.response_bytes
            || sha256_bytes(&bytes) != source.response_body_sha256
        {
            return Err("Stage 93 原始载荷大小或哈希已漂移".into());
        }
    }
    Ok(())
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
fn receipt_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-first-natural-forward-cycle-market-data-receipts")
}

pub(crate) fn controlled_shadow_market_data_receipt_custody_root(state: &AppState) -> PathBuf {
    receipt_root(state)
}
fn claim_directory(state: &AppState) -> PathBuf {
    receipt_root(state).join("claims")
}
fn result_directory(state: &AppState) -> PathBuf {
    receipt_root(state).join("results")
}

struct AttemptLock {
    path: PathBuf,
}
impl Drop for AttemptLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn acquire_lock(state: &AppState, id: &str) -> Result<AttemptLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("controlled-shadow-market-data-receipt-{id}.lock"));
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
                    .and_then(|m| m.modified().ok())
                    .and_then(|m| SystemTime::now().duration_since(m).ok())
                    .is_some_and(|age| age > StdDuration::from_secs(600));
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                if attempt < 7 {
                    tokio::time::sleep(StdDuration::from_millis(25)).await;
                    continue;
                }
                return Err("另一位管理员正在消耗该 Stage 92 授权".into());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 93 收据锁不可用".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_requests_are_fixed_redacted_and_content_addressed() {
        let from = NaiveDate::from_ymd_opt(2026, 8, 27).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 28).unwrap();
        let requests = canonical_requests(&["SNDK".into()], from, to).unwrap();
        assert_eq!(requests.len(), 11);
        let json = serde_json::to_string(&requests).unwrap();
        assert!(json.contains("REDACTED"));
        assert!(!json.contains("secret"));
        for path in [
            "/stable/historical-price-eod/full",
            "/stable/historical-price-eod/non-split-adjusted",
            "/stable/historical-price-eod/dividend-adjusted",
            "/stable/dividends",
            "/stable/splits",
        ] {
            assert!(requests.iter().any(|value| value.path == path));
        }
        assert!(!json.contains("/v3/historical-price-full"));
        assert!(requests.iter().all(|value| value.method == "GET" && valid_sha256(&value.canonical_request_sha256)));
    }

    #[test]
    fn wire_url_injects_credential_only_after_canonical_claim() {
        let descriptor = canonical_requests(
            &["SNDK".into()],
            NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
            NaiveDate::from_ymd_opt(2026, 8, 28).unwrap(),
        )
        .unwrap()
        .into_iter()
        .find(|value| value.source_id == "fmp_raw_unadjusted_price_SNDK")
        .unwrap();
        assert!(
            !serde_json::to_string(&descriptor)
                .unwrap()
                .contains("top-secret")
        );
        assert!(
            wire_url(&descriptor, "top-secret")
                .unwrap()
                .contains("apikey=top-secret")
        );
    }

    #[test]
    fn failed_terminal_result_consumes_authorization_without_downstream_authority() {
        let mut claim = ControlledShadowMarketDataReceiptClaim {
            schema_version: CLAIM_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            attempt_id: String::new(),
            claim_sha256: String::new(),
            adapter_authorization_id: "a".repeat(32),
            adapter_authorization_sha256: "b".repeat(64),
            cycle_claim_id: "c".repeat(32),
            cycle_claim_sha256: "d".repeat(64),
            adapter_spec_sha256: "e".repeat(64),
            subject_symbols: vec!["SNDK".into()],
            subject_symbol_set_sha256: fingerprint(&vec!["SNDK"]).unwrap(),
            benchmark_symbol: "SPY".into(),
            window_start_date: NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
            window_end_date: NaiveDate::from_ymd_opt(2026, 8, 28).unwrap(),
            time_window_sha256: fingerprint(&(
                NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
                NaiveDate::from_ymd_opt(2026, 8, 28).unwrap(),
            ))
            .unwrap(),
            canonical_requests: Vec::new(),
            canonical_request_set_sha256: fingerprint(
                &Vec::<CanonicalMarketDataRequestDescriptor>::new(),
            )
            .unwrap(),
            expected_request_count: 0,
            claimed_at: Utc::now(),
            claimed_by: "executor".into(),
            excluded_prior_actor_ids: vec!["reviewer".into()],
            execution_reason: "test".into(),
            adapter_authorization_consumed: true,
            claim_first: true,
            create_once: true,
            confirmations_complete: true,
        };
        claim.claim_sha256 = fingerprint_without(&claim, &["attempt_id", "claim_sha256"]).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].into();
        let result = new_result(
            &claim,
            Utc::now(),
            StdDuration::from_millis(1),
            None,
            Some("http_request_failed".into()),
        )
        .unwrap();
        assert!(matches!(
            result.status,
            ControlledShadowMarketDataReceiptAttemptStatus::FailedAuthorizationConsumed
        ));
        assert!(result.adapter_authorization_consumed && result.market_data_request_made);
        assert!(
            !result.market_data_accessed
                && !result.forward_observation_started
                && !result.trading_authorized
        );
    }
}
