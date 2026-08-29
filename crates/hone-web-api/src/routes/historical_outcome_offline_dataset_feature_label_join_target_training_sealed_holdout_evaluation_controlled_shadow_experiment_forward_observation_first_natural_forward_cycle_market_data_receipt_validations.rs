//! Stage 94 chain-external independent validation of a Stage 93 raw market-data receipt.
//!
//! This module never performs network access and never parses market rows or trading sessions. It
//! independently reopens the immutable Stage 92/93 chain, reconstructs the redacted request set,
//! re-reads every raw payload, and recomputes every relevant fingerprint. A failed validation is a
//! permanent terminal record; a passing record only allows a future, separately reviewed parser.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_adapter_authorizations::{
    ApprovedControlledShadowMarketDataAdapterAuthorization,
    exact_market_data_adapter_authorization_for_receipt_audit,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_attempts::{
    CanonicalMarketDataRequestDescriptor, CompletedControlledShadowMarketDataReceiptAttempt,
    ControlledShadowMarketDataReceiptClaim, ControlledShadowMarketDataReceiptAttemptResult,
    ControlledShadowMarketDataReceiptAttemptStatus, RawMarketDataSourceReceipt,
    UntrustedRawMarketDataReceipt,
    completed_market_data_receipt_attempts_for_independent_validation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-receipt-validation-registry-v2";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-receipt-independent-validation-v2";
const POLICY_VERSION: &str = "hone-controlled-shadow-market-data-receipt-independent-validation-v2-explicit-actions-no-parsing";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-stage-94-independent-raw-receipt-validator-v2-explicit-actions";
const FMP_ORIGIN: &str = "https://financialmodelingprep.com";
const MAX_SUBJECT_SYMBOLS: usize = 10;
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_TOTAL_RESPONSE_BYTES: usize = 64 * 1024 * 1024;
const MAX_MISMATCH_REASONS: usize = 24;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateControlledShadowMarketDataReceiptRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_receipt_sha256: String,
    expected_adapter_authorization_sha256: String,
    expected_cycle_claim_sha256: String,
    expected_adapter_spec_sha256: String,
    expected_subject_symbol_set_sha256: String,
    expected_time_window_sha256: String,
    expected_canonical_request_set_sha256: String,
    independent_chain_reopen_and_fingerprint_recomputation_confirmed: bool,
    validator_independent_from_executor_stage_92_and_complete_prior_chain_confirmed: bool,
    claim_first_single_terminal_result_and_no_replay_confirmed: bool,
    redacted_fixed_request_set_independently_reconstructed_confirmed: bool,
    every_raw_payload_reopened_size_and_sha256_recomputed_confirmed: bool,
    source_identity_timestamp_and_content_addressed_custody_confirmed: bool,
    credential_absence_from_persisted_artifacts_confirmed: bool,
    successful_http_envelope_only_not_market_truth_confirmed: bool,
    validation_does_not_parse_calendar_or_market_rows_confirmed: bool,
    no_runtime_observation_ledger_position_performance_or_model_metric_confirmed: bool,
    no_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowMarketDataReceiptValidationVerdict {
    IndependentlyValidatedUntrustedRawMarketDataReceipt,
    FailedIndependentRawMarketDataReceiptValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataReceiptValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub receipt_sha256: String,
    pub adapter_authorization_id: String,
    pub adapter_authorization_sha256: String,
    pub cycle_claim_sha256: String,
    pub adapter_spec_sha256: String,
    pub subject_symbol_set_sha256: String,
    pub time_window_sha256: String,
    pub canonical_request_set_sha256: String,
    pub raw_payload_custody_manifest_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub stage_93_executor_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_executor_and_complete_prior_chain: bool,
    pub exact_stage_51_through_stage_93_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub receipt_fingerprint_independently_verified: bool,
    pub canonical_request_set_independently_reconstructed: bool,
    pub claim_first_single_terminal_result_and_no_replay_verified: bool,
    pub raw_payload_count_size_and_sha256_independently_verified: bool,
    pub source_identity_timestamp_and_content_addressed_custody_verified: bool,
    pub persisted_artifacts_verified_credential_free: bool,
    pub successful_http_payload_envelope_verified_without_market_truth_claim: bool,
    pub no_parsing_observation_or_downstream_authority_verified: bool,
    pub mismatch_reasons: Vec<String>,
    pub verdict: ControlledShadowMarketDataReceiptValidationVerdict,
    pub raw_market_data_receipt_independently_validated: bool,
    pub future_market_data_parser_review_eligible: bool,
    pub calendar_window_resolved: bool,
    pub parsed_market_rows_created: bool,
    pub persistent_runtime_instantiated: bool,
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
pub(crate) struct ControlledShadowMarketDataReceiptValidationCandidate {
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub receipt_sha256: String,
    pub adapter_authorization_sha256: String,
    pub cycle_claim_sha256: String,
    pub adapter_spec_sha256: String,
    pub subject_symbols: Vec<String>,
    pub subject_symbol_set_sha256: String,
    pub window_start_date: NaiveDate,
    pub window_end_date: NaiveDate,
    pub time_window_sha256: String,
    pub canonical_request_set_sha256: String,
    pub raw_payload_count: usize,
    pub total_response_bytes: usize,
    pub validator_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataReceiptValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_endpoint_available: bool,
    pub candidates: Vec<ControlledShadowMarketDataReceiptValidationCandidate>,
    pub validations: Vec<ControlledShadowMarketDataReceiptValidationRecord>,
    pub completed_untrusted_receipt_count: usize,
    pub pending_independent_validation_count: usize,
    pub independently_validated_receipt_count: usize,
    pub failed_independent_validation_count: usize,
    pub future_market_data_parser_review_eligible_count: usize,
    pub validation_status: String,
    pub calendar_window_resolved: bool,
    pub parsed_market_rows_created: bool,
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
pub(crate) struct ControlledShadowMarketDataReceiptValidationReadinessSummary {
    pub completed_untrusted_receipt_count: usize,
    pub pending_independent_validation_count: usize,
    pub independently_validated_receipt_count: usize,
    pub failed_independent_validation_count: usize,
    pub future_market_data_parser_review_eligible_count: usize,
    pub validation_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct IndependentlyValidatedControlledShadowMarketDataReceipt {
    pub attempt: CompletedControlledShadowMarketDataReceiptAttempt,
    pub validation: ControlledShadowMarketDataReceiptValidationRecord,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    receipt_fingerprint: bool,
    canonical_request_set: bool,
    claim_first_single_terminal_no_replay: bool,
    raw_payload_integrity: bool,
    source_custody: bool,
    credential_free: bool,
    payload_envelope: bool,
    downstream_closed: bool,
    custody_manifest_sha256: String,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_receipt_validations(
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
            warn!(%error, "market-data receipt validation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "原始行情收据独立验证表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_controlled_shadow_market_data_receipt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateControlledShadowMarketDataReceiptRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match validate_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(_) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "验证已写入，但刷新注册表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_receipt_validation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataReceiptValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowMarketDataReceiptValidationReadinessSummary {
            completed_untrusted_receipt_count: registry.completed_untrusted_receipt_count,
            pending_independent_validation_count: registry.pending_independent_validation_count,
            independently_validated_receipt_count: registry.independently_validated_receipt_count,
            failed_independent_validation_count: registry.failed_independent_validation_count,
            future_market_data_parser_review_eligible_count: registry
                .future_market_data_parser_review_eligible_count,
            validation_status: registry.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_market_data_receipts_for_parser_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedControlledShadowMarketDataReceipt>, String> {
    let attempts = completed_market_data_receipt_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let attempt_index = attempts
        .into_iter()
        .map(|value| (value.claim.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    for validation in validations.into_iter().filter(|value| {
        value.raw_market_data_receipt_independently_validated
            && value.future_market_data_parser_review_eligible
    }) {
        let attempt = attempt_index
            .get(&validation.attempt_id)
            .cloned()
            .ok_or_else(|| "Stage 94 通过记录缺少 Stage 93 原始收据".to_string())?;
        output.push(IndependentlyValidatedControlledShadowMarketDataReceipt {
            attempt,
            validation,
        });
    }
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowMarketDataReceiptValidationRegistry, String> {
    let attempts = completed_market_data_receipt_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let validation_index = validations
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut candidates = Vec::new();
    for attempt in &attempts {
        if validation_index.contains_key(attempt.claim.attempt_id.as_str()) {
            continue;
        }
        let receipt = attempt
            .result
            .untrusted_raw_market_data_receipt
            .as_ref()
            .ok_or_else(|| "Stage 94 候选缺少 Stage 93 原始收据".to_string())?;
        candidates.push(candidate(attempt, receipt));
    }
    let independently_validated_receipt_count = validations
        .iter()
        .filter(|value| value.raw_market_data_receipt_independently_validated)
        .count();
    let failed_independent_validation_count = validations
        .len()
        .saturating_sub(independently_validated_receipt_count);
    let validation_status = if independently_validated_receipt_count > 0 {
        "raw_market_data_receipt_independently_validated_waiting_parser_review"
    } else if failed_independent_validation_count > 0 {
        "raw_market_data_receipt_independent_validation_failed_terminal"
    } else if !candidates.is_empty() {
        "waiting_for_chain_external_raw_market_data_receipt_validation"
    } else {
        "waiting_for_completed_stage_93_untrusted_raw_market_data_receipt"
    };
    Ok(ControlledShadowMarketDataReceiptValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        validation_endpoint_available: true,
        completed_untrusted_receipt_count: attempts.len(),
        pending_independent_validation_count: candidates.len(),
        independently_validated_receipt_count,
        failed_independent_validation_count,
        future_market_data_parser_review_eligible_count: independently_validated_receipt_count,
        candidates,
        validations,
        validation_status: validation_status.into(),
        calendar_window_resolved: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 94 只由 Stage 93 executor、Stage 92 reviewer 与完整既有责任链之外的新管理员，独立重算 claim/result/receipt/规范请求/原始载荷指纹和保管路径。通过只开放未来无行情语义的 parser 规格复核资格；当前不解析交易日或价格、不启动观察、不建账、不写持仓/绩效/模型指标，也不开放训练、reward、订单、券商或交易。".into(),
    })
}

fn candidate(
    attempt: &CompletedControlledShadowMarketDataReceiptAttempt,
    receipt: &UntrustedRawMarketDataReceipt,
) -> ControlledShadowMarketDataReceiptValidationCandidate {
    let mut excluded = attempt.claim.excluded_prior_actor_ids.clone();
    excluded.push(attempt.claim.claimed_by.clone());
    excluded.sort();
    excluded.dedup();
    ControlledShadowMarketDataReceiptValidationCandidate {
        attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        receipt_sha256: receipt.receipt_sha256.clone(),
        adapter_authorization_sha256: attempt.claim.adapter_authorization_sha256.clone(),
        cycle_claim_sha256: attempt.claim.cycle_claim_sha256.clone(),
        adapter_spec_sha256: attempt.claim.adapter_spec_sha256.clone(),
        subject_symbols: attempt.claim.subject_symbols.clone(),
        subject_symbol_set_sha256: attempt.claim.subject_symbol_set_sha256.clone(),
        window_start_date: attempt.claim.window_start_date,
        window_end_date: attempt.claim.window_end_date,
        time_window_sha256: attempt.claim.time_window_sha256.clone(),
        canonical_request_set_sha256: attempt.claim.canonical_request_set_sha256.clone(),
        raw_payload_count: receipt.raw_payload_count,
        total_response_bytes: receipt.total_response_bytes,
        validator_excluded_actor_ids: excluded,
    }
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateControlledShadowMarketDataReceiptRequest,
) -> Result<ControlledShadowMarketDataReceiptValidationRecord, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|value| value.attempt_id == attempt_id)
    {
        return Err("该 Stage 93 原始收据已形成不可覆盖的独立验证记录".into());
    }
    let attempt = completed_market_data_receipt_attempts_for_independent_validation(state)
        .await?
        .into_iter()
        .find(|value| value.claim.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有完整且待验证的 Stage 93 原始行情收据".to_string())?;
    verify_expected_bindings(&attempt, &request)?;
    let authorization = exact_market_data_adapter_authorization_for_receipt_audit(
        state,
        &attempt.claim.adapter_authorization_id,
        &attempt.claim.adapter_authorization_sha256,
    )
    .await?
    .ok_or_else(|| "Stage 94 无法重开精确 Stage 92 授权链".to_string())?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization);
    let independent = !validator_id.trim().is_empty()
        && excluded_prior_actor_ids
            .iter()
            .all(|value| value != validator_id);
    if !independent {
        return Err(
            "Stage 94 验证者不能是 Stage 93 executor、Stage 92 reviewer 或完整既有责任链任一角色"
                .into(),
        );
    }
    let audit = independently_audit(state, &attempt, &authorization).await;
    let passed = independent
        && audit.exact_chain
        && audit.claim_fingerprint
        && audit.result_fingerprint
        && audit.receipt_fingerprint
        && audit.canonical_request_set
        && audit.claim_first_single_terminal_no_replay
        && audit.raw_payload_integrity
        && audit.source_custody
        && audit.credential_free
        && audit.payload_envelope
        && audit.downstream_closed
        && audit.mismatch_reasons.is_empty();
    let receipt = attempt
        .result
        .untrusted_raw_market_data_receipt
        .as_ref()
        .ok_or_else(|| "Stage 94 成功候选缺少 Stage 93 原始收据".to_string())?;
    let mut record = ControlledShadowMarketDataReceiptValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        receipt_sha256: receipt.receipt_sha256.clone(),
        adapter_authorization_id: attempt.claim.adapter_authorization_id.clone(),
        adapter_authorization_sha256: attempt.claim.adapter_authorization_sha256.clone(),
        cycle_claim_sha256: attempt.claim.cycle_claim_sha256.clone(),
        adapter_spec_sha256: attempt.claim.adapter_spec_sha256.clone(),
        subject_symbol_set_sha256: attempt.claim.subject_symbol_set_sha256.clone(),
        time_window_sha256: attempt.claim.time_window_sha256.clone(),
        canonical_request_set_sha256: attempt.claim.canonical_request_set_sha256.clone(),
        raw_payload_custody_manifest_sha256: audit.custody_manifest_sha256,
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.into(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: validator_id.into(),
        stage_93_executor_id: attempt.claim.claimed_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_executor_and_complete_prior_chain: independent,
        exact_stage_51_through_stage_93_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        receipt_fingerprint_independently_verified: audit.receipt_fingerprint,
        canonical_request_set_independently_reconstructed: audit.canonical_request_set,
        claim_first_single_terminal_result_and_no_replay_verified: audit
            .claim_first_single_terminal_no_replay,
        raw_payload_count_size_and_sha256_independently_verified: audit.raw_payload_integrity,
        source_identity_timestamp_and_content_addressed_custody_verified: audit.source_custody,
        persisted_artifacts_verified_credential_free: audit.credential_free,
        successful_http_payload_envelope_verified_without_market_truth_claim: audit
            .payload_envelope,
        no_parsing_observation_or_downstream_authority_verified: audit.downstream_closed,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            ControlledShadowMarketDataReceiptValidationVerdict::IndependentlyValidatedUntrustedRawMarketDataReceipt
        } else {
            ControlledShadowMarketDataReceiptValidationVerdict::FailedIndependentRawMarketDataReceiptValidation
        },
        raw_market_data_receipt_independently_validated: passed,
        future_market_data_parser_review_eligible: passed,
        calendar_window_resolved: false,
        parsed_market_rows_created: false,
        persistent_runtime_instantiated: false,
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
    record.validation_sha256 = record_fingerprint(&record)?;
    record.validation_id = record.validation_sha256[..32].into();
    validate_record(&record)?;
    write_immutable_json(
        &validation_directory(state).join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

async fn independently_audit(
    state: &AppState,
    attempt: &CompletedControlledShadowMarketDataReceiptAttempt,
    authorization: &ApprovedControlledShadowMarketDataAdapterAuthorization,
) -> IndependentAudit {
    let claim = &attempt.claim;
    let result = &attempt.result;
    let receipt = result.untrusted_raw_market_data_receipt.as_ref();
    let mut mismatch_reasons = Vec::new();
    let claim_fingerprint = independent_fingerprint_without(claim, &["attempt_id", "claim_sha256"])
        .is_ok_and(|value| value == claim.claim_sha256 && claim.attempt_id == value[..32]);
    let result_fingerprint =
        independent_fingerprint_without(result, &["result_id", "result_sha256"])
            .is_ok_and(|value| value == result.result_sha256 && result.result_id == value[..32]);
    let receipt_fingerprint = receipt.is_some_and(|value| {
        independent_fingerprint_without(value, &["receipt_sha256"])
            .is_ok_and(|hash| hash == value.receipt_sha256)
    });
    let exact_chain = claim.adapter_authorization_id
        == authorization.authorization.adapter_authorization_id
        && claim.adapter_authorization_sha256
            == authorization.authorization.adapter_authorization_sha256
        && claim.cycle_claim_id == authorization.claim.cycle_claim_id
        && claim.cycle_claim_sha256 == authorization.claim.cycle_claim_sha256
        && claim.adapter_spec_sha256
            == authorization
                .authorization
                .adapter_specification
                .adapter_spec_sha256
        && claim.claimed_at >= authorization.authorization.authorized_not_before
        && claim.claimed_at < authorization.authorization.authorized_valid_until;
    let reconstructed = independently_reconstruct_canonical_requests(
        &claim.subject_symbols,
        claim.window_start_date,
        claim.window_end_date,
    );
    let canonical_request_set = reconstructed.is_ok_and(|value| {
        value == claim.canonical_requests
            && independent_fingerprint(&value)
                .is_ok_and(|hash| hash == claim.canonical_request_set_sha256)
            && independent_fingerprint(&claim.subject_symbols)
                .is_ok_and(|hash| hash == claim.subject_symbol_set_sha256)
            && independent_fingerprint(&(claim.window_start_date, claim.window_end_date))
                .is_ok_and(|hash| hash == claim.time_window_sha256)
    });
    let claim_first_single_terminal_no_replay = claim.claimed_at <= result.completed_at
        && claim.adapter_authorization_consumed
        && claim.claim_first
        && claim.create_once
        && result.adapter_authorization_consumed
        && matches!(
            result.status,
            ControlledShadowMarketDataReceiptAttemptStatus::CompletedWithUntrustedRawMarketDataReceipt
        )
        && result.bounded_error_code.is_none();
    let custody = independently_validate_payloads(state, claim, receipt).await;
    let credential_free =
        persisted_artifacts_are_credential_free(state, claim, result, receipt).await;
    let downstream_closed = receipt.is_some_and(|value| {
        value.output_is_untrusted
            && !value.independent_validation_completed
            && !value.calendar_window_resolved
            && !value.parsed_market_rows_created
            && !value.forward_observation_started
    }) && !result.calendar_window_resolved
        && !result.forward_observation_started
        && !result.ledger_created
        && !result.position_written
        && !result.performance_metric_written
        && !result.model_store_written
        && !result.metric_store_written
        && !result.training_feedback_authorized
        && !result.reward_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;

    push_mismatch(
        &mut mismatch_reasons,
        exact_chain,
        "Stage 51–93 exact chain mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        claim_fingerprint,
        "Stage 93 claim fingerprint mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        result_fingerprint,
        "Stage 93 result fingerprint mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        receipt_fingerprint,
        "Stage 93 receipt fingerprint mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        canonical_request_set,
        "redacted canonical request reconstruction mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        claim_first_single_terminal_no_replay,
        "claim-first or single terminal result mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        custody.integrity,
        "raw payload count, size or SHA-256 mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        custody.source_custody,
        "source identity, time or content-addressed custody mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        custody.payload_envelope,
        "successful HTTP payload envelope is empty or inconsistent",
    );
    push_mismatch(
        &mut mismatch_reasons,
        credential_free,
        "persisted artifact may contain configured credential",
    );
    push_mismatch(
        &mut mismatch_reasons,
        downstream_closed,
        "parsing, observation or downstream authority is not closed",
    );

    IndependentAudit {
        exact_chain,
        claim_fingerprint,
        result_fingerprint,
        receipt_fingerprint,
        canonical_request_set,
        claim_first_single_terminal_no_replay,
        raw_payload_integrity: custody.integrity,
        source_custody: custody.source_custody,
        credential_free,
        payload_envelope: custody.payload_envelope,
        downstream_closed,
        custody_manifest_sha256: custody.manifest_sha256,
        mismatch_reasons,
    }
}

#[derive(Debug)]
struct PayloadAudit {
    integrity: bool,
    source_custody: bool,
    payload_envelope: bool,
    manifest_sha256: String,
}

async fn independently_validate_payloads(
    state: &AppState,
    claim: &ControlledShadowMarketDataReceiptClaim,
    receipt: Option<&UntrustedRawMarketDataReceipt>,
) -> PayloadAudit {
    let Some(receipt) = receipt else {
        return PayloadAudit {
            integrity: false,
            source_custody: false,
            payload_envelope: false,
            manifest_sha256: independent_hash_bytes(&[]),
        };
    };
    let request_index = claim
        .canonical_requests
        .iter()
        .map(|value| (value.source_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut manifest = Vec::new();
    let mut integrity = receipt.source_receipts.len() == claim.expected_request_count;
    let mut source_custody = true;
    let mut payload_envelope = true;
    let mut total = 0usize;
    let mut seen = BTreeSet::new();
    for source in &receipt.source_receipts {
        let Some(request) = request_index.get(source.source_id.as_str()) else {
            source_custody = false;
            continue;
        };
        let expected_relative = format!(
            "payloads/{}/{}-{}.bin",
            claim.attempt_id, source.source_id, source.response_body_sha256
        );
        let bytes = tokio::fs::read(receipt_root(state).join(&expected_relative))
            .await
            .unwrap_or_default();
        let body_sha = independent_hash_bytes(&bytes);
        total = total.saturating_add(bytes.len());
        integrity &= !bytes.is_empty()
            && bytes.len() == source.response_bytes
            && bytes.len() <= MAX_RESPONSE_BYTES
            && body_sha == source.response_body_sha256
            && body_sha == source.source_document_sha256;
        source_custody &= seen.insert(source.source_id.clone())
            && source.raw_payload_relative_path == expected_relative
            && source.canonical_request_sha256 == request.canonical_request_sha256
            && source.http_status == 200
            && source.source_available_at_utc <= source.retrieved_at_utc
            && source.source_available_at_basis == "retrieval_time_fallback_unverified";
        payload_envelope &= payload_envelope_is_sane(source, &bytes);
        manifest.push((
            source.source_id.clone(),
            source.canonical_request_sha256.clone(),
            body_sha,
            bytes.len(),
            expected_relative,
        ));
    }
    manifest.sort();
    integrity &= total == receipt.total_response_bytes
        && total <= MAX_TOTAL_RESPONSE_BYTES
        && receipt.raw_payload_count == receipt.source_receipts.len();
    source_custody &= seen.len() == request_index.len();
    PayloadAudit {
        integrity,
        source_custody,
        payload_envelope,
        manifest_sha256: independent_fingerprint(&manifest)
            .unwrap_or_else(|_| independent_hash_bytes(&[])),
    }
}

fn payload_envelope_is_sane(source: &RawMarketDataSourceReceipt, bytes: &[u8]) -> bool {
    let first = bytes
        .iter()
        .copied()
        .find(|value| !value.is_ascii_whitespace());
    if source.source_id == "nyse_official_us_market_calendar" {
        first == Some(b'<')
    } else if source.source_id.starts_with("fmp_") {
        matches!(first, Some(b'{') | Some(b'['))
    } else {
        false
    }
}

async fn persisted_artifacts_are_credential_free(
    state: &AppState,
    claim: &ControlledShadowMarketDataReceiptClaim,
    result: &ControlledShadowMarketDataReceiptAttemptResult,
    receipt: Option<&UntrustedRawMarketDataReceipt>,
) -> bool {
    let serialized = serde_json::to_vec(&(claim, result, receipt)).unwrap_or_default();
    let keys = state
        .core
        .config
        .fmp
        .effective_key_pool()
        .keys()
        .iter()
        .filter(|value| !value.is_empty())
        .map(|value| value.as_bytes().to_vec())
        .collect::<Vec<_>>();
    let canonical_is_redacted = claim.canonical_requests.iter().all(|request| {
        request
            .query
            .iter()
            .all(|(name, value)| name != "apikey" || value == "REDACTED")
    });
    let json_is_clean = keys
        .iter()
        .all(|key| !serialized.windows(key.len()).any(|window| window == key));
    let mut payloads_are_clean = true;
    if let Some(receipt) = receipt {
        for source in &receipt.source_receipts {
            let bytes =
                tokio::fs::read(receipt_root(state).join(&source.raw_payload_relative_path))
                    .await
                    .unwrap_or_default();
            payloads_are_clean &= keys
                .iter()
                .all(|key| !bytes.windows(key.len()).any(|window| window == key));
        }
    }
    canonical_is_redacted && json_is_clean && payloads_are_clean
}

fn independently_reconstruct_canonical_requests(
    symbols: &[String],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketDataRequestDescriptor>, String> {
    if symbols.is_empty()
        || symbols.len() > MAX_SUBJECT_SYMBOLS
        || !sorted_unique(symbols)
        || symbols.iter().any(|value| !valid_symbol(value))
        || to < from
    {
        return Err("Stage 94 无法独立重建非法标的或时间窗".into());
    }
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
                independent_fingerprint_without(&descriptor, &["canonical_request_sha256"])?;
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
        independent_fingerprint_without(&calendar, &["canonical_request_sha256"])?;
    output.push(calendar);
    output.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    Ok(output)
}

fn verify_expected_bindings(
    attempt: &CompletedControlledShadowMarketDataReceiptAttempt,
    request: &ValidateControlledShadowMarketDataReceiptRequest,
) -> Result<(), String> {
    let receipt = attempt
        .result
        .untrusted_raw_market_data_receipt
        .as_ref()
        .ok_or_else(|| "Stage 94 候选缺少原始收据".to_string())?;
    if request.expected_claim_sha256 != attempt.claim.claim_sha256
        || request.expected_result_sha256 != attempt.result.result_sha256
        || request.expected_receipt_sha256 != receipt.receipt_sha256
        || request.expected_adapter_authorization_sha256
            != attempt.claim.adapter_authorization_sha256
        || request.expected_cycle_claim_sha256 != attempt.claim.cycle_claim_sha256
        || request.expected_adapter_spec_sha256 != attempt.claim.adapter_spec_sha256
        || request.expected_subject_symbol_set_sha256 != attempt.claim.subject_symbol_set_sha256
        || request.expected_time_window_sha256 != attempt.claim.time_window_sha256
        || request.expected_canonical_request_set_sha256
            != attempt.claim.canonical_request_set_sha256
    {
        return Err("Stage 94 预期 Stage 92/93 绑定已经漂移".into());
    }
    Ok(())
}

fn validate_request(
    attempt_id: &str,
    request: &ValidateControlledShadowMarketDataReceiptRequest,
) -> Result<(), String> {
    let hashes = [
        request.expected_claim_sha256.as_str(),
        request.expected_result_sha256.as_str(),
        request.expected_receipt_sha256.as_str(),
        request.expected_adapter_authorization_sha256.as_str(),
        request.expected_cycle_claim_sha256.as_str(),
        request.expected_adapter_spec_sha256.as_str(),
        request.expected_subject_symbol_set_sha256.as_str(),
        request.expected_time_window_sha256.as_str(),
        request.expected_canonical_request_set_sha256.as_str(),
    ];
    if !valid_id(attempt_id) || hashes.iter().any(|value| !valid_sha256(value)) {
        return Err("Stage 94 请求 ID 或 SHA-256 无效".into());
    }
    if !confirmations_complete(request) {
        return Err("必须确认独立重算、原始载荷保管、凭据隔离、禁止解析与全部下游权限关闭".into());
    }
    Ok(())
}

fn confirmations_complete(value: &ValidateControlledShadowMarketDataReceiptRequest) -> bool {
    value.independent_chain_reopen_and_fingerprint_recomputation_confirmed
        && value.validator_independent_from_executor_stage_92_and_complete_prior_chain_confirmed
        && value.claim_first_single_terminal_result_and_no_replay_confirmed
        && value.redacted_fixed_request_set_independently_reconstructed_confirmed
        && value.every_raw_payload_reopened_size_and_sha256_recomputed_confirmed
        && value.source_identity_timestamp_and_content_addressed_custody_confirmed
        && value.credential_absence_from_persisted_artifacts_confirmed
        && value.successful_http_envelope_only_not_market_truth_confirmed
        && value.validation_does_not_parse_calendar_or_market_rows_confirmed
        && value.no_runtime_observation_ledger_position_performance_or_model_metric_confirmed
        && value.no_training_feedback_reward_order_broker_or_trading_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn prior_actor_ids(
    attempt: &CompletedControlledShadowMarketDataReceiptAttempt,
    authorization: &ApprovedControlledShadowMarketDataAdapterAuthorization,
) -> Vec<String> {
    let mut values = attempt.claim.excluded_prior_actor_ids.clone();
    values.extend(authorization.authorization.excluded_prior_actor_ids.clone());
    values.push(authorization.authorization.reviewer_id.clone());
    values.push(attempt.claim.claimed_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn validate_record(
    value: &ControlledShadowMarketDataReceiptValidationRecord,
) -> Result<(), String> {
    let passed = value.validator_independent_from_executor_and_complete_prior_chain
        && value.exact_stage_51_through_stage_93_chain_verified
        && value.claim_fingerprint_independently_verified
        && value.result_fingerprint_independently_verified
        && value.receipt_fingerprint_independently_verified
        && value.canonical_request_set_independently_reconstructed
        && value.claim_first_single_terminal_result_and_no_replay_verified
        && value.raw_payload_count_size_and_sha256_independently_verified
        && value.source_identity_timestamp_and_content_addressed_custody_verified
        && value.persisted_artifacts_verified_credential_free
        && value.successful_http_payload_envelope_verified_without_market_truth_claim
        && value.no_parsing_observation_or_downstream_authority_verified
        && value.mismatch_reasons.is_empty();
    let authority_closed = !value.calendar_window_resolved
        && !value.parsed_market_rows_created
        && !value.persistent_runtime_instantiated
        && !value.forward_observation_started
        && !value.ledger_created
        && !value.position_written
        && !value.performance_metric_written
        && !value.model_store_written
        && !value.metric_store_written
        && !value.training_feedback_authorized
        && !value.reward_authorized
        && !value.order_generation_authorized
        && !value.broker_access_authorized
        && !value.trading_authorized;
    if value.schema_version != RECORD_SCHEMA_VERSION
        || value.policy_version != POLICY_VERSION
        || !valid_id(&value.validation_id)
        || value.validation_sha256 != record_fingerprint(value)?
        || value.validation_id != value.validation_sha256[..32]
        || value.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || value.validator_implementation_sha256 != validator_implementation_sha256()
        || !sorted_unique(&value.excluded_prior_actor_ids)
        || value.excluded_prior_actor_ids.contains(&value.validated_by)
        || !authority_closed
        || value.raw_market_data_receipt_independently_validated != passed
        || value.future_market_data_parser_review_eligible != passed
        || (passed
            != matches!(
                value.verdict,
                ControlledShadowMarketDataReceiptValidationVerdict::IndependentlyValidatedUntrustedRawMarketDataReceipt
            ))
    {
        return Err("Stage 94 原始收据独立验证记录无效、漂移或越权".into());
    }
    Ok(())
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataReceiptValidationRecord>, String> {
    let mut directory = match tokio::fs::read_dir(validation_directory(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    let mut attempts = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let value: ControlledShadowMarketDataReceiptValidationRecord = serde_json::from_slice(
            &tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        validate_record(&value)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(value.validation_id.as_str())
            || !ids.insert(value.validation_id.clone())
            || !attempts.insert(value.attempt_id.clone())
        {
            return Err("Stage 94 验证记录文件名、ID 或 attempt 重复".into());
        }
        output.push(value);
    }
    output.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(output)
}

fn record_fingerprint(
    value: &ControlledShadowMarketDataReceiptValidationRecord,
) -> Result<String, String> {
    independent_fingerprint_without(value, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    independent_hash_bytes(include_bytes!(
        "historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_validations.rs"
    ))
}

fn independent_fingerprint<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(independent_hash_bytes(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}

fn independent_fingerprint_without<T: Serialize>(
    value: &T,
    fields: &[&str],
) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 94 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    independent_fingerprint(&value)
}

fn independent_hash_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn push_mismatch(reasons: &mut Vec<String>, condition: bool, message: &str) {
    if !condition && reasons.len() < MAX_MISMATCH_REASONS {
        reasons.push(message.into());
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

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn receipt_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-first-natural-forward-cycle-market-data-receipts")
}

fn validation_directory(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-first-natural-forward-cycle-market-data-receipt-validations")
}

struct ValidationLock {
    path: PathBuf,
}

impl Drop for ValidationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<ValidationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-market-data-receipt-validation-{attempt_id}.lock"
    ));
    for retry in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ValidationLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|value| value.modified().ok())
                    .and_then(|value| SystemTime::now().duration_since(value).ok())
                    .is_some_and(|age| age > StdDuration::from_secs(120))
                {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                if retry == 7 {
                    return Err("Stage 94 验证正在被另一请求处理".into());
                }
                tokio::time::sleep(StdDuration::from_millis(20 * (retry + 1))).await;
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 94 无法取得验证锁".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ValidateControlledShadowMarketDataReceiptRequest {
        ValidateControlledShadowMarketDataReceiptRequest {
            expected_claim_sha256: "1".repeat(64),
            expected_result_sha256: "2".repeat(64),
            expected_receipt_sha256: "3".repeat(64),
            expected_adapter_authorization_sha256: "4".repeat(64),
            expected_cycle_claim_sha256: "5".repeat(64),
            expected_adapter_spec_sha256: "6".repeat(64),
            expected_subject_symbol_set_sha256: "7".repeat(64),
            expected_time_window_sha256: "8".repeat(64),
            expected_canonical_request_set_sha256: "9".repeat(64),
            independent_chain_reopen_and_fingerprint_recomputation_confirmed: true,
            validator_independent_from_executor_stage_92_and_complete_prior_chain_confirmed: true,
            claim_first_single_terminal_result_and_no_replay_confirmed: true,
            redacted_fixed_request_set_independently_reconstructed_confirmed: true,
            every_raw_payload_reopened_size_and_sha256_recomputed_confirmed: true,
            source_identity_timestamp_and_content_addressed_custody_confirmed: true,
            credential_absence_from_persisted_artifacts_confirmed: true,
            successful_http_envelope_only_not_market_truth_confirmed: true,
            validation_does_not_parse_calendar_or_market_rows_confirmed: true,
            no_runtime_observation_ledger_position_performance_or_model_metric_confirmed: true,
            no_training_feedback_reward_order_broker_or_trading_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn reconstructs_only_redacted_fixed_requests() {
        let values = independently_reconstruct_canonical_requests(
            &["NVDA".into(), "SNDK".into()],
            NaiveDate::from_ymd_opt(2026, 8, 26).unwrap(),
            NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
        )
        .unwrap();
        assert_eq!(values.len(), 16);
        assert!(values.iter().all(|value| value.method == "GET"));
        let json = serde_json::to_string(&values).unwrap();
        for path in [
            "/stable/historical-price-eod/full",
            "/stable/historical-price-eod/non-split-adjusted",
            "/stable/historical-price-eod/dividend-adjusted",
            "/stable/dividends",
            "/stable/splits",
        ] {
            assert!(json.contains(path));
        }
        assert!(!json.contains("/v3/historical-price-full"));
        assert!(
            values
                .iter()
                .all(|value| !serde_json::to_string(value).unwrap().contains("sk-"))
        );
        assert!(
            values
                .iter()
                .filter(|value| value.source_id.starts_with("fmp_"))
                .all(|value| {
                    value
                        .query
                        .iter()
                        .any(|(name, value)| name == "apikey" && value == "REDACTED")
                })
        );
    }

    #[test]
    fn rejects_incomplete_or_malformed_validation_request() {
        let mut value = request();
        assert!(validate_request(&"a".repeat(32), &value).is_ok());
        value.validation_does_not_parse_calendar_or_market_rows_confirmed = false;
        assert!(validate_request(&"a".repeat(32), &value).is_err());
        value.validation_does_not_parse_calendar_or_market_rows_confirmed = true;
        value.expected_receipt_sha256 = "not-a-hash".into();
        assert!(validate_request(&"a".repeat(32), &value).is_err());
    }

    #[test]
    fn envelope_check_is_format_only_and_rejects_empty_payload() {
        let source = RawMarketDataSourceReceipt {
            source_id: "fmp_split_adjusted_price_SNDK".into(),
            canonical_request_sha256: "1".repeat(64),
            response_body_sha256: "2".repeat(64),
            source_document_sha256: "2".repeat(64),
            response_bytes: 2,
            content_type: Some("application/json".into()),
            http_status: 200,
            retrieved_at_utc: Utc::now(),
            source_available_at_utc: Utc::now(),
            source_available_at_basis: "retrieval_time_fallback_unverified".into(),
            raw_payload_relative_path: "payloads/x/y.bin".into(),
        };
        assert!(payload_envelope_is_sane(&source, b"{}"));
        assert!(!payload_envelope_is_sane(&source, b""));
        assert!(!payload_envelope_is_sane(&source, b"<html>"));
    }
}
