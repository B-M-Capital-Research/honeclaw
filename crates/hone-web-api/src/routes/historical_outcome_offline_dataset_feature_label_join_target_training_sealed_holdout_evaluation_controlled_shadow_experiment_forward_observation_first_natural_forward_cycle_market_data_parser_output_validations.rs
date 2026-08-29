//! Stage 103 chain-external validation of one Stage 102 untrusted market-data parse output.
//!
//! This validator does not call the Stage 102 parser helpers. It reopens the frozen Stage 94 raw
//! payloads, parses them with a second implementation, reconstructs the complete canonical output
//! and compares every field and row hash. A validation record is create-once and terminal. Passing
//! only admits an observation-input candidate for a later gate; it starts no observation and grants
//! no model, reward, order, broker or trading authority.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc, Weekday};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_execution_attempt_claims::{
    ControlledShadowMarketDataParserExecutionAttemptClaim,
    market_data_parser_execution_attempt_claims_for_execution,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_execution_attempts::{
    CanonicalMarketCalendarRow, CanonicalMarketDividendRow, CanonicalMarketPriceRow,
    CanonicalMarketSplitRow, ControlledShadowMarketDataParserExecutionAttemptResult,
    ControlledShadowMarketDataParserExecutionStatus, ExplicitMarketDataGap,
    UntrustedCanonicalMarketDataParseResult,
    controlled_shadow_market_data_parser_execution_custody_root,
    market_data_parser_execution_results_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_attempts::{
    CanonicalMarketDataRequestDescriptor, controlled_shadow_market_data_receipt_custody_root,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-output-validation-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-chain-external-full-reparse-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-market-data-parser-validator-v1-independent-second-implementation";
const OUTPUT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-untrusted-canonical-market-data-parse-result-v1";
const NEXT_GATE: &str = "stage_104_first_natural_forward_cycle_observation_input_admission_review";
const MAX_REASON_CHARS: usize = 4_000;
const MAX_MISMATCH_REASONS: usize = 64;
const MAX_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;
const MAX_PAYLOAD_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TOTAL_PAYLOAD_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateControlledShadowMarketDataParserOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_input_manifest_sha256: String,
    expected_stage_94_validation_sha256: String,
    validation_reason: String,
    exact_current_stage_51_through_stage_102_binding_confirmed: bool,
    validator_independent_from_executor_and_complete_prior_chain_confirmed: bool,
    stage_102_result_output_and_create_once_custody_reopened_confirmed: bool,
    fixed_stage_94_raw_payloads_rehashed_and_independently_reparsed_confirmed: bool,
    second_implementation_does_not_call_stage_102_parser_helpers_confirmed: bool,
    every_canonical_row_hash_and_complete_output_exactly_compared_confirmed: bool,
    official_calendar_spy_coverage_subject_gaps_and_actions_fail_closed_confirmed: bool,
    source_available_at_remains_unverified_confirmed: bool,
    pass_only_opens_future_observation_input_admission_review_confirmed: bool,
    no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowMarketDataParserOutputValidationVerdict {
    IndependentlyValidatedExactCanonicalParseOutput,
    FailedIndependentCanonicalParseOutputValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub stage_102_attempt_id: String,
    pub stage_101_claim_sha256: String,
    pub stage_102_result_id: String,
    pub stage_102_result_sha256: String,
    pub stage_102_output_sha256: String,
    pub stage_101_input_manifest_sha256: String,
    pub stage_94_validation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub validation_reason: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validator_independent_from_executor_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_102_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub output_file_custody_and_fingerprint_verified: bool,
    pub raw_payload_custody_and_fingerprints_verified: bool,
    pub canonical_rows_independently_reparsed: bool,
    pub every_row_hash_independently_verified: bool,
    pub complete_output_exact_match_verified: bool,
    pub official_calendar_and_spy_coverage_verified: bool,
    pub source_available_at_verified: bool,
    pub no_downstream_authority_verified: bool,
    pub recomputed_claim_sha256: String,
    pub recomputed_result_sha256: String,
    pub recomputed_persisted_output_sha256: String,
    pub independently_recomputed_output_sha256: String,
    pub observed_output_bytes: usize,
    pub observed_raw_payload_count: usize,
    pub observed_raw_payload_bytes: usize,
    pub mismatch_reasons: Vec<String>,
    pub verdict: ControlledShadowMarketDataParserOutputValidationVerdict,
    pub canonical_parse_output_independently_validated: bool,
    pub future_observation_input_admission_review_eligible: bool,
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserOutputValidationItem {
    pub claim: ControlledShadowMarketDataParserExecutionAttemptClaim,
    pub result: ControlledShadowMarketDataParserExecutionAttemptResult,
    pub validation: Option<ControlledShadowMarketDataParserOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<ControlledShadowMarketDataParserOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_output_count: usize,
    pub failed_validation_count: usize,
    pub future_observation_input_admission_review_eligible_count: usize,
    pub validation_status: String,
    pub next_gate: String,
    pub independent_output_validation_available: bool,
    pub source_available_at_verified: bool,
    pub forward_observation_started: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowMarketDataParserOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_output_count: usize,
    pub failed_validation_count: usize,
    pub future_observation_input_admission_review_eligible_count: usize,
    pub validation_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyValidatedMarketDataParserOutput {
    pub claim: ControlledShadowMarketDataParserExecutionAttemptClaim,
    pub result: ControlledShadowMarketDataParserExecutionAttemptResult,
    pub validation: ControlledShadowMarketDataParserOutputValidationRecord,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    output_file: bool,
    raw_payloads: bool,
    reparsed: bool,
    row_hashes: bool,
    exact_output: bool,
    calendar_spy: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_persisted_output_sha256: String,
    independently_recomputed_output_sha256: String,
    output_bytes: usize,
    raw_payload_count: usize,
    raw_payload_bytes: usize,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_parser_output_validations(
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
            warn!(%error, "market-data parser output validation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "行情解析输出独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_controlled_shadow_market_data_parser_output_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateControlledShadowMarketDataParserOutputRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match validate_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_parser_output_validation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserOutputValidationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowMarketDataParserOutputValidationReadinessSummary {
            validation_eligible_count: value.validation_eligible_count,
            validation_count: value.validation_count,
            independently_validated_output_count: value.independently_validated_output_count,
            failed_validation_count: value.failed_validation_count,
            future_observation_input_admission_review_eligible_count: value
                .future_observation_input_admission_review_eligible_count,
            validation_status: value.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_market_data_parser_outputs_for_observation_input_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedMarketDataParserOutput>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let validation = item.validation?;
            (validation.canonical_parse_output_independently_validated
                && validation.future_observation_input_admission_review_eligible)
                .then_some(IndependentlyValidatedMarketDataParserOutput {
                    claim: item.claim,
                    result: item.result,
                    validation,
                })
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserOutputValidationRegistry, String> {
    let claims = market_data_parser_execution_attempt_claims_for_execution(state).await?;
    let claim_index = claims
        .into_iter()
        .map(|value| (value.attempt_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let results = market_data_parser_execution_results_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let validation_index = validations
        .iter()
        .map(|value| (value.stage_102_attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let mut items = Vec::new();
    for result in results.into_iter().filter(|value| {
        value.status
            == ControlledShadowMarketDataParserExecutionStatus::CompletedWithUntrustedOutput
    }) {
        let claim = claim_index
            .get(&result.stage_101_attempt_id)
            .cloned()
            .ok_or_else(|| "Stage 103 candidate 缺少 Stage 101 claim".to_string())?;
        let validation = validation_index
            .get(result.stage_101_attempt_id.as_str())
            .cloned()
            .cloned();
        if let Some(record) = &validation {
            validate_record_binding(record, &claim, &result)?;
        }
        items.push(ControlledShadowMarketDataParserOutputValidationItem {
            validation_eligible: validation.is_none(),
            claim,
            result,
            validation,
        });
    }
    let validation_eligible_count = items
        .iter()
        .filter(|value| value.validation_eligible)
        .count();
    let independently_validated_output_count = validations
        .iter()
        .filter(|value| value.canonical_parse_output_independently_validated)
        .count();
    let failed_validation_count = validations
        .len()
        .saturating_sub(independently_validated_output_count);
    let validation_status = if failed_validation_count > 0 {
        "market_data_parser_output_validation_failed_fail_closed"
    } else if independently_validated_output_count > 0 {
        "market_data_parser_output_independently_validated_waiting_observation_input_admission_review"
    } else if validation_eligible_count > 0 {
        "waiting_chain_external_full_reparse_validation"
    } else {
        "waiting_successful_stage_102_untrusted_output"
    };
    Ok(ControlledShadowMarketDataParserOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count: validations.len(),
        independently_validated_output_count,
        failed_validation_count,
        future_observation_input_admission_review_eligible_count:
            independently_validated_output_count,
        validation_status: validation_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        independent_output_validation_available: true,
        source_available_at_verified: false,
        forward_observation_started: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 103 由 Stage 102 executor 与完整 Stage 51–102 责任链之外的新管理员，以第二实现重新打开固定 Stage 94 原始载荷和 Stage 102 非可信输出，独立重算 claim/result/output 指纹、交易日历、三套价格、分红、拆股、SPY 覆盖、标的缺口及每行哈希，并逐字段精确比较。通过只开放 Stage 104 观察输入准入复核候选；source_available_at 仍未验证，也不开始观察、不建账、不写持仓/绩效、不训练/reward、不生成订单、不接券商、不交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateControlledShadowMarketDataParserOutputRequest,
) -> Result<ControlledShadowMarketDataParserOutputValidationRecord, String> {
    validate_request_shape(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|value| value.stage_102_attempt_id == attempt_id)
    {
        return Err("这条 Stage 102 输出已经形成不可变独立验证记录，不得重放".to_string());
    }
    let claim = market_data_parser_execution_attempt_claims_for_execution(state)
        .await?
        .into_iter()
        .find(|value| value.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有对应的 Stage 101 claim".to_string())?;
    let result = market_data_parser_execution_results_for_independent_validation(state)
        .await?
        .into_iter()
        .find(|value| {
            value.stage_101_attempt_id == attempt_id
                && value.status
                    == ControlledShadowMarketDataParserExecutionStatus::CompletedWithUntrustedOutput
        })
        .ok_or_else(|| "当前没有成功且待独立校验的 Stage 102 非可信输出".to_string())?;
    verify_expected_bindings(&claim, &result, &request)?;
    let excluded_prior_actor_ids = prior_actor_ids(&claim, &result);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "Stage 103 验证者不能是 Stage 102 executor 或 Stage 51–102 任一既有责任人".to_string(),
        );
    }
    let audit = independently_audit(state, &claim, &result).await;
    let passed = independent
        && audit.exact_chain
        && audit.claim_fingerprint
        && audit.result_fingerprint
        && audit.output_file
        && audit.raw_payloads
        && audit.reparsed
        && audit.row_hashes
        && audit.exact_output
        && audit.calendar_spy
        && audit.downstream_closed
        && audit.mismatch_reasons.is_empty();
    let mut record = ControlledShadowMarketDataParserOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        stage_102_attempt_id: claim.attempt_id.clone(),
        stage_101_claim_sha256: claim.claim_sha256.clone(),
        stage_102_result_id: result.result_id.clone(),
        stage_102_result_sha256: result.result_sha256.clone(),
        stage_102_output_sha256: result.output_sha256.clone().unwrap_or_default(),
        stage_101_input_manifest_sha256: claim.fixed_input_manifest.input_manifest_sha256.clone(),
        stage_94_validation_sha256: claim
            .fixed_input_manifest
            .stage_94_validation
            .validation_sha256
            .clone(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        validation_reason: bounded_required(
            &request.validation_reason,
            MAX_REASON_CHARS,
            "验证原因",
        )?,
        excluded_prior_actor_ids,
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validator_independent_from_executor_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_102_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        output_file_custody_and_fingerprint_verified: audit.output_file,
        raw_payload_custody_and_fingerprints_verified: audit.raw_payloads,
        canonical_rows_independently_reparsed: audit.reparsed,
        every_row_hash_independently_verified: audit.row_hashes,
        complete_output_exact_match_verified: audit.exact_output,
        official_calendar_and_spy_coverage_verified: audit.calendar_spy,
        source_available_at_verified: false,
        no_downstream_authority_verified: audit.downstream_closed,
        recomputed_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_result_sha256: audit.recomputed_result_sha256,
        recomputed_persisted_output_sha256: audit.recomputed_persisted_output_sha256,
        independently_recomputed_output_sha256: audit.independently_recomputed_output_sha256,
        observed_output_bytes: audit.output_bytes,
        observed_raw_payload_count: audit.raw_payload_count,
        observed_raw_payload_bytes: audit.raw_payload_bytes,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            ControlledShadowMarketDataParserOutputValidationVerdict::IndependentlyValidatedExactCanonicalParseOutput
        } else {
            ControlledShadowMarketDataParserOutputValidationVerdict::FailedIndependentCanonicalParseOutputValidation
        },
        canonical_parse_output_independently_validated: passed,
        future_observation_input_admission_review_eligible: passed,
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
    record.validation_sha256 = record_fingerprint(&record)?;
    record.validation_id = record.validation_sha256[..32].to_string();
    validate_record(&record)?;
    write_immutable_json(
        &validation_root(state)
            .join(attempt_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

async fn independently_audit(
    state: &AppState,
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    result: &ControlledShadowMarketDataParserExecutionAttemptResult,
) -> IndependentAudit {
    let mut reasons = Vec::new();
    let recomputed_claim_sha256 =
        independent_fingerprint_without(claim, &["attempt_id", "claim_sha256"]).unwrap_or_else(
            |error| {
                push_reason(&mut reasons, format!("claim fingerprint: {error}"));
                hash_bytes(b"claim-fingerprint-unavailable")
            },
        );
    let recomputed_result_sha256 =
        independent_fingerprint_without(result, &["result_id", "result_sha256"]).unwrap_or_else(
            |error| {
                push_reason(&mut reasons, format!("result fingerprint: {error}"));
                hash_bytes(b"result-fingerprint-unavailable")
            },
        );
    let claim_fingerprint = recomputed_claim_sha256 == claim.claim_sha256;
    let result_fingerprint = recomputed_result_sha256 == result.result_sha256;
    push_check(
        &mut reasons,
        claim_fingerprint,
        "Stage 101 claim fingerprint mismatch",
    );
    push_check(
        &mut reasons,
        result_fingerprint,
        "Stage 102 result fingerprint mismatch",
    );

    let mut output_bytes_count = 0usize;
    let mut recomputed_persisted_output_sha256 = hash_bytes(b"persisted-output-unavailable");
    let mut persisted_output: Option<UntrustedCanonicalMarketDataParseResult> = None;
    let output_file = match independent_read_output(state, result).await {
        Ok((value, bytes)) => {
            output_bytes_count = bytes;
            recomputed_persisted_output_sha256 =
                independent_fingerprint_without(&value, &["output_sha256"]).unwrap_or_else(
                    |error| {
                        push_reason(
                            &mut reasons,
                            format!("persisted output fingerprint: {error}"),
                        );
                        hash_bytes(b"persisted-output-fingerprint-unavailable")
                    },
                );
            let valid = value.output_sha256 == recomputed_persisted_output_sha256
                && result.output_sha256.as_deref() == Some(value.output_sha256.as_str());
            push_check(
                &mut reasons,
                valid,
                "Stage 102 output custody or fingerprint mismatch",
            );
            persisted_output = Some(value);
            valid
        }
        Err(error) => {
            push_reason(&mut reasons, format!("Stage 102 output reopen: {error}"));
            false
        }
    };

    let mut raw_payload_count = 0usize;
    let mut raw_payload_bytes = 0usize;
    let mut independently_recomputed_output_sha256 = hash_bytes(b"independent-output-unavailable");
    let mut recomputed_output: Option<UntrustedCanonicalMarketDataParseResult> = None;
    let (raw_payloads, reparsed) = match independent_read_payloads(state, claim).await {
        Ok(payloads) => {
            raw_payload_count = payloads.len();
            raw_payload_bytes = payloads.values().map(Vec::len).sum();
            match independent_parse_payloads(claim, &payloads) {
                Ok(mut value) => {
                    independently_recomputed_output_sha256 =
                        independent_fingerprint_without(&value, &["output_sha256"]).unwrap_or_else(
                            |error| {
                                push_reason(
                                    &mut reasons,
                                    format!("independent output fingerprint: {error}"),
                                );
                                hash_bytes(b"independent-output-fingerprint-unavailable")
                            },
                        );
                    value.output_sha256 = independently_recomputed_output_sha256.clone();
                    recomputed_output = Some(value);
                    (true, true)
                }
                Err(error) => {
                    push_reason(&mut reasons, format!("independent full reparse: {error}"));
                    (true, false)
                }
            }
        }
        Err(error) => {
            push_reason(&mut reasons, format!("raw payload reopen: {error}"));
            (false, false)
        }
    };
    let row_hashes = recomputed_output
        .as_ref()
        .is_some_and(independent_row_hashes_valid);
    push_check(
        &mut reasons,
        row_hashes,
        "independent canonical row hash mismatch",
    );
    let exact_output = persisted_output
        .as_ref()
        .zip(recomputed_output.as_ref())
        .is_some_and(|(left, right)| left == right);
    push_check(
        &mut reasons,
        exact_output,
        "persisted output differs from independent reparse",
    );
    let calendar_spy = recomputed_output
        .as_ref()
        .is_some_and(|value| independent_calendar_spy_output_valid(claim, value));
    push_check(
        &mut reasons,
        calendar_spy,
        "official calendar, SPY coverage or explicit subject gaps invalid",
    );
    let downstream_closed = persisted_output.as_ref().is_some_and(|value| {
        value.untrusted
            && !value.independent_validation_completed
            && !value.observation_started
            && !value.ledger_created
            && !value.position_written
            && !value.order_generation_authorized
            && !value.broker_access_authorized
            && !value.trading_authorized
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
            && !result.trading_authorized
    });
    push_check(
        &mut reasons,
        downstream_closed,
        "downstream authority is not closed",
    );
    let exact_chain = result.stage_101_claim_sha256 == claim.claim_sha256
        && persisted_output.as_ref().is_some_and(|value| {
            value.execution_claim_sha256 == claim.claim_sha256
                && value.source_receipt_sha256 == claim.fixed_input_manifest.stage_93_receipt_sha256
                && value.parser_spec_sha256
                    == claim
                        .authorization
                        .runner
                        .implementation
                        .implementation_contract
                        .parser_specification_sha256
        });
    push_check(&mut reasons, exact_chain, "Stage 51–102 binding mismatch");

    IndependentAudit {
        exact_chain,
        claim_fingerprint,
        result_fingerprint,
        output_file,
        raw_payloads,
        reparsed,
        row_hashes,
        exact_output,
        calendar_spy,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_persisted_output_sha256,
        independently_recomputed_output_sha256,
        output_bytes: output_bytes_count,
        raw_payload_count,
        raw_payload_bytes,
        mismatch_reasons: reasons,
    }
}

async fn independent_read_output(
    state: &AppState,
    result: &ControlledShadowMarketDataParserExecutionAttemptResult,
) -> Result<(UntrustedCanonicalMarketDataParseResult, usize), String> {
    let sha = result
        .output_sha256
        .as_deref()
        .ok_or_else(|| "successful result has no output SHA".to_string())?;
    let expected = PathBuf::from("outputs").join(format!("{sha}.json"));
    let relative = PathBuf::from(
        result
            .output_relative_path
            .as_deref()
            .ok_or_else(|| "successful result has no output path".to_string())?,
    );
    if relative != expected || !safe_relative_path(&relative) {
        return Err("output path is not the exact content-addressed path".to_string());
    }
    let path = controlled_shadow_market_data_parser_execution_custody_root(state).join(relative);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|_| "output file is missing".to_string())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_OUTPUT_BYTES
    {
        return Err("output is not a bounded regular file".to_string());
    }
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|_| "output file read failed".to_string())?;
    let value = serde_json::from_slice::<UntrustedCanonicalMarketDataParseResult>(&bytes)
        .map_err(|_| "output JSON/schema rejected".to_string())?;
    Ok((value, bytes.len()))
}

async fn independent_read_payloads(
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
            return Err("raw payload path escapes exact Stage 93 custody".to_string());
        }
        let path = root.join(&relative);
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|_| "raw payload missing".to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() > MAX_PAYLOAD_BYTES
        {
            return Err("raw payload is not a bounded regular file".to_string());
        }
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|_| "raw payload read failed".to_string())?;
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| "raw payload total overflow".to_string())?;
        if total > MAX_TOTAL_PAYLOAD_BYTES
            || bytes.len() != receipt.response_bytes
            || hash_bytes(&bytes) != receipt.response_body_sha256
            || receipt.source_document_sha256 != receipt.response_body_sha256
            || output.insert(receipt.source_id.clone(), bytes).is_some()
        {
            return Err("raw payload length, digest, source or total mismatch".to_string());
        }
    }
    if output.len() != claim.fixed_input_manifest.raw_payload_count
        || total != claim.fixed_input_manifest.total_response_bytes
    {
        return Err("raw payload set differs from fixed input manifest".to_string());
    }
    Ok(output)
}

fn independent_parse_payloads(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    payloads: &BTreeMap<String, Vec<u8>>,
) -> Result<UntrustedCanonicalMarketDataParseResult, String> {
    let requests = claim
        .fixed_input_manifest
        .stage_93_claim
        .canonical_requests
        .iter()
        .map(|value| (value.source_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    if requests.len() != payloads.len() {
        return Err("request/payload cardinality mismatch".to_string());
    }
    let from = claim.fixed_input_manifest.window_start_date;
    let to = claim.fixed_input_manifest.window_end_date;
    let mut calendar_rows = Vec::new();
    let mut price_rows = Vec::new();
    let mut dividend_rows = Vec::new();
    let mut split_rows = Vec::new();
    for (source_id, bytes) in payloads {
        let request = requests
            .get(source_id.as_str())
            .ok_or_else(|| "payload source has no canonical request".to_string())?;
        let receipt = claim
            .fixed_input_manifest
            .raw_payloads
            .iter()
            .find(|value| value.source_id == *source_id)
            .ok_or_else(|| "payload source has no custody receipt".to_string())?;
        if request.canonical_request_sha256 != receipt.canonical_request_sha256 {
            return Err("canonical request digest drift".to_string());
        }
        if source_id == "nyse_official_us_market_calendar" {
            calendar_rows = independent_parse_nyse_calendar(bytes, from, to)?;
            continue;
        }
        let symbol = independent_request_symbol(request)?;
        match request.path.as_str() {
            "/stable/historical-price-eod/full" => price_rows.extend(independent_parse_prices(
                bytes,
                &symbol,
                "split_adjusted_price",
                &request.canonical_request_sha256,
                from,
                to,
            )?),
            "/stable/historical-price-eod/non-split-adjusted" => {
                price_rows.extend(independent_parse_prices(
                    bytes,
                    &symbol,
                    "raw_unadjusted_price",
                    &request.canonical_request_sha256,
                    from,
                    to,
                )?)
            }
            "/stable/historical-price-eod/dividend-adjusted" => {
                price_rows.extend(independent_parse_prices(
                    bytes,
                    &symbol,
                    "dividend_adjusted_price",
                    &request.canonical_request_sha256,
                    from,
                    to,
                )?)
            }
            "/stable/dividends" => {
                dividend_rows.extend(independent_parse_dividends(bytes, &symbol, from, to)?)
            }
            "/stable/splits" => {
                split_rows.extend(independent_parse_splits(bytes, &symbol, from, to)?)
            }
            _ => return Err("request path is outside frozen parser allowlist".to_string()),
        }
    }
    if calendar_rows.is_empty() {
        return Err("official calendar contains no sessions".to_string());
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
    let explicit_gaps = independent_reconcile(claim, &calendar_rows, &price_rows)?;
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
        raw_payload_bytes: payloads.values().map(Vec::len).sum(),
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

fn independent_parse_prices(
    bytes: &[u8],
    symbol: &str,
    source_kind: &str,
    request_sha256: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketPriceRow>, String> {
    let values = independent_json_array(bytes)?;
    if values.is_empty() {
        return Err(format!("{symbol} {source_kind} empty"));
    }
    let mut dates = BTreeSet::new();
    let mut rows = Vec::new();
    for value in values {
        let object = value
            .as_object()
            .ok_or_else(|| "price row is not object".to_string())?;
        independent_require_symbol(object, symbol)?;
        let date = independent_required_date(object.get("date"), from, to)?;
        if !dates.insert(date) {
            return Err(format!("duplicate {symbol} {source_kind} date"));
        }
        let close_decimal = independent_decimal(object.get("close"), true, "close")?;
        let volume_integer = match object.get("volume") {
            None | Some(Value::Null) => None,
            Some(value) => Some(
                value
                    .as_u64()
                    .ok_or_else(|| "volume is not non-negative integer".to_string())?,
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
        row.row_sha256 = independent_fingerprint_without(&row, &["row_sha256"])?;
        rows.push(row);
    }
    Ok(rows)
}

fn independent_parse_dividends(
    bytes: &[u8],
    symbol: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketDividendRow>, String> {
    let mut dates = BTreeSet::new();
    let mut rows = Vec::new();
    for value in independent_json_array(bytes)? {
        let object = value
            .as_object()
            .ok_or_else(|| "dividend row is not object".to_string())?;
        independent_require_symbol(object, symbol)?;
        let date = independent_required_date(object.get("date"), from, to)?;
        if !dates.insert(date) {
            return Err(format!("duplicate {symbol} dividend date"));
        }
        let mut row = CanonicalMarketDividendRow {
            symbol: symbol.to_string(),
            date,
            dividend_decimal: independent_decimal(object.get("dividend"), false, "dividend")?,
            record_date: independent_optional_date(object.get("recordDate"))?,
            payment_date: independent_optional_date(object.get("paymentDate"))?,
            declaration_date: independent_optional_date(object.get("declarationDate"))?,
            row_sha256: String::new(),
        };
        row.row_sha256 = independent_fingerprint_without(&row, &["row_sha256"])?;
        rows.push(row);
    }
    Ok(rows)
}

fn independent_parse_splits(
    bytes: &[u8],
    symbol: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketSplitRow>, String> {
    let mut dates = BTreeSet::new();
    let mut rows = Vec::new();
    for value in independent_json_array(bytes)? {
        let object = value
            .as_object()
            .ok_or_else(|| "split row is not object".to_string())?;
        independent_require_symbol(object, symbol)?;
        let date = independent_required_date(object.get("date"), from, to)?;
        if !dates.insert(date) {
            return Err(format!("duplicate {symbol} split date"));
        }
        let numerator = object
            .get("numerator")
            .and_then(Value::as_u64)
            .filter(|value| *value > 0)
            .ok_or_else(|| "split numerator is not positive integer".to_string())?;
        let denominator = object
            .get("denominator")
            .and_then(Value::as_u64)
            .filter(|value| *value > 0)
            .ok_or_else(|| "split denominator is not positive integer".to_string())?;
        let split_type = match object.get("splitType") {
            None | Some(Value::Null) => None,
            Some(Value::String(value)) if !value.trim().is_empty() => {
                Some(value.trim().to_string())
            }
            _ => return Err("splitType must be non-empty string or null".to_string()),
        };
        let mut row = CanonicalMarketSplitRow {
            symbol: symbol.to_string(),
            date,
            numerator_positive_integer: numerator,
            denominator_positive_integer: denominator,
            split_type,
            row_sha256: String::new(),
        };
        row.row_sha256 = independent_fingerprint_without(&row, &["row_sha256"])?;
        rows.push(row);
    }
    Ok(rows)
}

fn independent_parse_nyse_calendar(
    bytes: &[u8],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketCalendarRow>, String> {
    let html = std::str::from_utf8(bytes).map_err(|_| "NYSE HTML is not UTF-8".to_string())?;
    let document = Html::parse_document(html);
    let table_selector = Selector::parse("table").map_err(|_| "table selector".to_string())?;
    let row_selector = Selector::parse("tr").map_err(|_| "row selector".to_string())?;
    let cell_selector = Selector::parse("th,td").map_err(|_| "cell selector".to_string())?;
    for table in document.select(&table_selector) {
        let rows = independent_html_rows(table, &row_selector, &cell_selector);
        if rows
            .first()
            .is_some_and(|value| value == &["Date", "Session", "Close ET"])
        {
            return independent_direct_sessions(&rows[1..], from, to);
        }
    }
    independent_holiday_sessions(&document, from, to)
}

fn independent_html_rows(
    table: scraper::ElementRef<'_>,
    row_selector: &Selector,
    cell_selector: &Selector,
) -> Vec<Vec<String>> {
    table
        .select(row_selector)
        .map(|row| {
            row.select(cell_selector)
                .map(|cell| independent_normalized_text(cell.text()))
                .collect::<Vec<_>>()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

fn independent_direct_sessions(
    rows: &[Vec<String>],
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketCalendarRow>, String> {
    let mut dates = BTreeSet::new();
    let mut output = Vec::new();
    for cells in rows {
        if cells.len() != 3 {
            return Err("direct NYSE row column count".to_string());
        }
        let date = NaiveDate::parse_from_str(&cells[0], "%Y-%m-%d")
            .map_err(|_| "direct NYSE date".to_string())?;
        if date < from || date > to || !dates.insert(date) {
            return Err("direct NYSE date outside window or duplicate".to_string());
        }
        if !matches!(
            (cells[1].as_str(), cells[2].as_str()),
            ("regular", "16:00") | ("early_close", "13:00")
        ) {
            return Err("direct NYSE session/close".to_string());
        }
        output.push(independent_calendar_row(date, &cells[1], &cells[2])?);
    }
    Ok(output)
}

fn independent_holiday_sessions(
    document: &Html,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<CanonicalMarketCalendarRow>, String> {
    let table_selector = Selector::parse("table").map_err(|_| "table selector".to_string())?;
    let row_selector = Selector::parse("tr").map_err(|_| "row selector".to_string())?;
    let cell_selector = Selector::parse("th,td").map_err(|_| "cell selector".to_string())?;
    let required_years = (from.year()..=to.year()).collect::<BTreeSet<_>>();
    let mut available_years = BTreeSet::new();
    let mut holidays = BTreeSet::new();
    let mut found = false;
    for table in document.select(&table_selector) {
        let rows = independent_html_rows(table, &row_selector, &cell_selector);
        let Some(header) = rows.first() else { continue };
        if header.first().map(String::as_str) != Some("Holiday") {
            continue;
        }
        found = true;
        let years = header
            .iter()
            .skip(1)
            .map(|value| value.parse::<i32>().map_err(|_| "NYSE year".to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        available_years.extend(years.iter().copied());
        for cells in rows.iter().skip(1) {
            if cells.len() != years.len() + 1 {
                return Err("NYSE holiday row column count".to_string());
            }
            for (index, year) in years.iter().enumerate() {
                if let Some(date) = independent_holiday_cell(&cells[index + 1], *year)? {
                    holidays.insert(date);
                }
            }
        }
    }
    if !found || !required_years.is_subset(&available_years) {
        return Err("NYSE holiday table does not cover window years".to_string());
    }
    let early_closes = independent_early_close_dates(document)?;
    let mut output = Vec::new();
    let mut date = from;
    while date <= to {
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) && !holidays.contains(&date) {
            let early = early_closes.contains(&date);
            output.push(independent_calendar_row(
                date,
                if early { "early_close" } else { "regular" },
                if early { "13:00" } else { "16:00" },
            )?);
        }
        date = date
            .checked_add_signed(Duration::days(1))
            .ok_or_else(|| "calendar overflow".to_string())?;
    }
    Ok(output)
}

fn independent_early_close_dates(document: &Html) -> Result<BTreeSet<NaiveDate>, String> {
    let selector = Selector::parse("p").map_err(|_| "paragraph selector".to_string())?;
    let pattern = Regex::new(r"(?:Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday),\s+([A-Z][a-z]+\s+\d{1,2}),\s+(\d{4})")
        .map_err(|_| "early-close regex".to_string())?;
    let mut output = BTreeSet::new();
    for paragraph in document.select(&selector) {
        let text = independent_normalized_text(paragraph.text());
        if !text.contains("close early at 1:00 p.m.") {
            continue;
        }
        for captures in pattern.captures_iter(&text) {
            let date = NaiveDate::parse_from_str(
                &format!("{}, {}", &captures[1], &captures[2]),
                "%B %d, %Y",
            )
            .map_err(|_| "early-close date".to_string())?;
            output.insert(date);
        }
    }
    Ok(output)
}

fn independent_holiday_cell(value: &str, year: i32) -> Result<Option<NaiveDate>, String> {
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
    NaiveDate::parse_from_str(&format!("{}, {year}", cleaned.trim()), "%B %d, %Y")
        .map(Some)
        .map_err(|_| format!("NYSE holiday date: {value}"))
}

fn independent_reconcile(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    calendar: &[CanonicalMarketCalendarRow],
    prices: &[CanonicalMarketPriceRow],
) -> Result<Vec<ExplicitMarketDataGap>, String> {
    let sessions = calendar
        .iter()
        .map(|value| value.date)
        .collect::<BTreeSet<_>>();
    let mut symbols = claim.fixed_input_manifest.subject_symbols.clone();
    if !symbols.contains(&claim.fixed_input_manifest.benchmark_symbol) {
        symbols.push(claim.fixed_input_manifest.benchmark_symbol.clone());
    }
    symbols.sort();
    symbols.dedup();
    let mut gaps = Vec::new();
    for symbol in symbols {
        for source_kind in [
            "split_adjusted_price",
            "raw_unadjusted_price",
            "dividend_adjusted_price",
        ] {
            let dates = prices
                .iter()
                .filter(|row| row.symbol == symbol && row.source_kind == source_kind)
                .map(|row| row.date)
                .collect::<BTreeSet<_>>();
            if dates.iter().any(|date| !sessions.contains(date)) {
                return Err(format!("{symbol} {source_kind} has non-session date"));
            }
            for date in sessions.difference(&dates) {
                if symbol == claim.fixed_input_manifest.benchmark_symbol {
                    return Err(format!("SPY {source_kind} missing {date}"));
                }
                gaps.push(ExplicitMarketDataGap {
                    symbol: symbol.clone(),
                    date: *date,
                    source_kind: source_kind.to_string(),
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

fn independent_row_hashes_valid(value: &UntrustedCanonicalMarketDataParseResult) -> bool {
    value.calendar_rows.iter().all(|row| {
        independent_fingerprint_without(row, &["row_sha256"])
            .is_ok_and(|digest| digest == row.row_sha256)
    }) && value.price_rows.iter().all(|row| {
        independent_fingerprint_without(row, &["row_sha256"])
            .is_ok_and(|digest| digest == row.row_sha256)
    }) && value.dividend_rows.iter().all(|row| {
        independent_fingerprint_without(row, &["row_sha256"])
            .is_ok_and(|digest| digest == row.row_sha256)
    }) && value.split_rows.iter().all(|row| {
        independent_fingerprint_without(row, &["row_sha256"])
            .is_ok_and(|digest| digest == row.row_sha256)
    })
}

fn independent_calendar_spy_output_valid(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    value: &UntrustedCanonicalMarketDataParseResult,
) -> bool {
    let sessions = value
        .calendar_rows
        .iter()
        .map(|row| row.date)
        .collect::<BTreeSet<_>>();
    if sessions.len() != value.calendar_rows.len() || sessions.is_empty() {
        return false;
    }
    let spy = &claim.fixed_input_manifest.benchmark_symbol;
    [
        "split_adjusted_price",
        "raw_unadjusted_price",
        "dividend_adjusted_price",
    ]
    .iter()
    .all(|kind| {
        value
            .price_rows
            .iter()
            .filter(|row| &row.symbol == spy && &row.source_kind == kind)
            .map(|row| row.date)
            .collect::<BTreeSet<_>>()
            == sessions
    })
}

fn independent_json_array(bytes: &[u8]) -> Result<Vec<Value>, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "FMP bytes are not UTF-8".to_string())?;
    serde_json::from_str::<Value>(text)
        .map_err(|_| "FMP payload is not JSON".to_string())?
        .as_array()
        .cloned()
        .ok_or_else(|| "FMP top-level value is not array".to_string())
}

fn independent_request_symbol(
    request: &CanonicalMarketDataRequestDescriptor,
) -> Result<String, String> {
    request
        .query
        .iter()
        .find(|(key, _)| key == "symbol")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "canonical request has no symbol".to_string())
}

fn independent_require_symbol(object: &Map<String, Value>, symbol: &str) -> Result<(), String> {
    match object.get("symbol") {
        Some(Value::String(value)) if value == symbol => Ok(()),
        _ => Err(format!("row symbol differs from {symbol}")),
    }
}

fn independent_required_date(
    value: Option<&Value>,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<NaiveDate, String> {
    let date = independent_optional_date(value)?.ok_or_else(|| "missing date".to_string())?;
    if date < from || date > to {
        Err("date outside frozen window".to_string())
    } else {
        Ok(date)
    }
}

fn independent_optional_date(value: Option<&Value>) -> Result<Option<NaiveDate>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| "date is not YYYY-MM-DD".to_string()),
        _ => Err("date is not string or null".to_string()),
    }
}

fn independent_decimal(
    value: Option<&Value>,
    positive: bool,
    field: &str,
) -> Result<String, String> {
    let number = value
        .and_then(Value::as_number)
        .ok_or_else(|| format!("{field} is not JSON number"))?;
    let parsed = number
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| format!("{field} is not finite"))?;
    if (positive && parsed <= 0.0) || (!positive && parsed < 0.0) {
        return Err(format!("{field} outside allowed range"));
    }
    Ok(independent_canonical_number(number))
}

fn independent_canonical_number(number: &Number) -> String {
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

fn independent_normalized_text<'a>(values: impl Iterator<Item = &'a str>) -> String {
    values
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

fn independent_calendar_row(
    date: NaiveDate,
    session_kind: &str,
    close_time_et: &str,
) -> Result<CanonicalMarketCalendarRow, String> {
    let mut row = CanonicalMarketCalendarRow {
        date,
        session_kind: session_kind.to_string(),
        close_time_et: close_time_et.to_string(),
        row_sha256: String::new(),
    };
    row.row_sha256 = independent_fingerprint_without(&row, &["row_sha256"])?;
    Ok(row)
}

fn validate_request_shape(
    attempt_id: &str,
    request: &ValidateControlledShadowMarketDataParserOutputRequest,
) -> Result<(), String> {
    let valid = valid_id(attempt_id)
        && valid_sha256(&request.expected_claim_sha256)
        && valid_sha256(&request.expected_result_sha256)
        && valid_sha256(&request.expected_output_sha256)
        && valid_sha256(&request.expected_input_manifest_sha256)
        && valid_sha256(&request.expected_stage_94_validation_sha256)
        && !request.validation_reason.trim().is_empty()
        && request.validation_reason.chars().count() <= MAX_REASON_CHARS
        && request.exact_current_stage_51_through_stage_102_binding_confirmed
        && request.validator_independent_from_executor_and_complete_prior_chain_confirmed
        && request.stage_102_result_output_and_create_once_custody_reopened_confirmed
        && request.fixed_stage_94_raw_payloads_rehashed_and_independently_reparsed_confirmed
        && request.second_implementation_does_not_call_stage_102_parser_helpers_confirmed
        && request.every_canonical_row_hash_and_complete_output_exactly_compared_confirmed
        && request.official_calendar_spy_coverage_subject_gaps_and_actions_fail_closed_confirmed
        && request.source_available_at_remains_unverified_confirmed
        && request.pass_only_opens_future_observation_input_admission_review_confirmed
        && request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 103 请求摘要、理由或确认项不完整".to_string())
}

fn verify_expected_bindings(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    result: &ControlledShadowMarketDataParserExecutionAttemptResult,
    request: &ValidateControlledShadowMarketDataParserOutputRequest,
) -> Result<(), String> {
    let valid = request.expected_claim_sha256 == claim.claim_sha256
        && request.expected_result_sha256 == result.result_sha256
        && result.output_sha256.as_deref() == Some(request.expected_output_sha256.as_str())
        && request.expected_input_manifest_sha256
            == claim.fixed_input_manifest.input_manifest_sha256
        && request.expected_stage_94_validation_sha256
            == claim
                .fixed_input_manifest
                .stage_94_validation
                .validation_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 103 期望绑定与当前不可变链不一致".to_string())
}

fn prior_actor_ids(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    result: &ControlledShadowMarketDataParserExecutionAttemptResult,
) -> Vec<String> {
    let mut values = claim.excluded_prior_actor_ids.clone();
    values.push(claim.claimed_by.clone());
    values.push(claim.authorization.reviewer_id.clone());
    values.push(result.executed_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn validate_record_binding(
    record: &ControlledShadowMarketDataParserOutputValidationRecord,
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
    result: &ControlledShadowMarketDataParserExecutionAttemptResult,
) -> Result<(), String> {
    let valid = record.stage_102_attempt_id == claim.attempt_id
        && record.stage_101_claim_sha256 == claim.claim_sha256
        && record.stage_102_result_id == result.result_id
        && record.stage_102_result_sha256 == result.result_sha256
        && result.output_sha256.as_deref() == Some(record.stage_102_output_sha256.as_str())
        && record.stage_101_input_manifest_sha256
            == claim.fixed_input_manifest.input_manifest_sha256
        && record.stage_94_validation_sha256
            == claim
                .fixed_input_manifest
                .stage_94_validation
                .validation_sha256
        && record.excluded_prior_actor_ids == prior_actor_ids(claim, result);
    valid
        .then_some(())
        .ok_or_else(|| "Stage 103 验证记录与当前 Stage 101/102 链漂移".to_string())
}

fn validate_record(
    record: &ControlledShadowMarketDataParserOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_executor_and_complete_prior_chain
        && record.exact_current_stage_51_through_stage_102_chain_verified
        && record.claim_fingerprint_independently_verified
        && record.result_fingerprint_independently_verified
        && record.output_file_custody_and_fingerprint_verified
        && record.raw_payload_custody_and_fingerprints_verified
        && record.canonical_rows_independently_reparsed
        && record.every_row_hash_independently_verified
        && record.complete_output_exact_match_verified
        && record.official_calendar_and_spy_coverage_verified
        && !record.source_available_at_verified
        && record.no_downstream_authority_verified
        && record.mismatch_reasons.is_empty();
    let authority_closed = !record.forward_observation_started
        && !record.ledger_created
        && !record.position_written
        && !record.performance_metric_written
        && !record.model_or_metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let valid = record.schema_version == RECORD_SCHEMA_VERSION
        && record.policy_version == POLICY_VERSION
        && valid_id(&record.validation_id)
        && valid_sha256(&record.validation_sha256)
        && record.validation_id == record.validation_sha256[..32]
        && record.validation_sha256 == record_fingerprint(record)?
        && record.validator_implementation_version == VALIDATOR_IMPLEMENTATION_VERSION
        && record.validator_implementation_sha256 == validator_implementation_sha256()
        && sorted_unique(&record.excluded_prior_actor_ids)
        && actor_is_independent(&record.validated_by, &record.excluded_prior_actor_ids)
        && !record.validation_reason.trim().is_empty()
        && record.validation_reason.chars().count() <= MAX_REASON_CHARS
        && record.mismatch_reasons.len() <= MAX_MISMATCH_REASONS
        && authority_closed
        && record.canonical_parse_output_independently_validated == passed
        && record.future_observation_input_admission_review_eligible == passed
        && (record.verdict
            == ControlledShadowMarketDataParserOutputValidationVerdict::IndependentlyValidatedExactCanonicalParseOutput)
            == passed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 103 独立验证记录无效、漂移或越权".to_string())
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserOutputValidationRecord>, String> {
    let mut records = Vec::new();
    let mut stack = vec![validation_root(state)];
    while let Some(directory) = stack.pop() {
        let mut entries = match tokio::fs::read_dir(directory).await {
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
                let record = serde_json::from_slice::<
                    ControlledShadowMarketDataParserOutputValidationRecord,
                >(
                    &tokio::fs::read(path)
                        .await
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
                validate_record(&record)?;
                records.push(record);
            }
        }
    }
    let mut attempts = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for record in &records {
        if !attempts.insert(record.stage_102_attempt_id.clone())
            || !ids.insert(record.validation_id.clone())
        {
            return Err("Stage 103 独立验证记录重复或重放".to_string());
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn push_reason(reasons: &mut Vec<String>, reason: String) {
    if reasons.len() < MAX_MISMATCH_REASONS {
        reasons.push(reason);
    }
}

fn push_check(reasons: &mut Vec<String>, condition: bool, reason: &str) {
    if !condition {
        push_reason(reasons, reason.to_string());
    }
}

fn record_fingerprint(
    record: &ControlledShadowMarketDataParserOutputValidationRecord,
) -> Result<String, String> {
    independent_fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    hash_bytes(include_bytes!(
        "historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_output_validations.rs"
    ))
}

fn independent_fingerprint_without<T: Serialize>(
    value: &T,
    fields: &[&str],
) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(hash_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
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
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
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

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-market-data-parser-output-validations")
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
    let path = directory.join(format!("stage103-{attempt_id}.lock"));
    for _ in 0..2 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ValidationLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .and_then(|value| value.modified())
                    .ok()
                    .and_then(|value| value.elapsed().ok())
                    .is_some_and(|elapsed| elapsed.as_secs() > 300);
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                return Err("已有同一 Stage 103 独立校验正在进行".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 103 校验锁不可用".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_price_parser_is_strict_and_hashes_rows() {
        let date = NaiveDate::from_ymd_opt(2099, 1, 3).unwrap();
        let rows = independent_parse_prices(
            br#"[{"symbol":"ZZTEST","date":"2099-01-03","close":10.50,"volume":100}]"#,
            "ZZTEST",
            "split_adjusted_price",
            &"a".repeat(64),
            date,
            date,
        )
        .unwrap();
        assert_eq!(rows[0].close_decimal, "10.5");
        assert!(valid_sha256(&rows[0].row_sha256));
        assert!(
            independent_parse_prices(
                br#"[{"symbol":"ZZTEST","date":"2099-01-03","close":"10.5"}]"#,
                "ZZTEST",
                "split_adjusted_price",
                &"a".repeat(64),
                date,
                date,
            )
            .is_err()
        );
    }

    #[test]
    fn independent_action_parsers_reject_inferred_shapes() {
        let date = NaiveDate::from_ymd_opt(2099, 1, 3).unwrap();
        assert!(
            independent_parse_dividends(
                br#"[{"symbol":"ZZTEST","date":"2099-01-03","dividend":0.25}]"#,
                "ZZTEST",
                date,
                date,
            )
            .is_ok()
        );
        assert!(
            independent_parse_splits(
                br#"[{"symbol":"ZZTEST","date":"2099-01-03","splitRatio":"4:1"}]"#,
                "ZZTEST",
                date,
                date,
            )
            .is_err()
        );
    }

    #[test]
    fn independent_nyse_parser_handles_real_holiday_shape() {
        let html = br#"<table><tr><th>Holiday</th><th>2026</th></tr><tr><th>Thanksgiving Day</th><td>Thursday, November 26***</td></tr></table><p>*** Each market will close early at 1:00 p.m. on Friday, November 27, 2026 (the day after Thanksgiving).</p>"#;
        let rows = independent_parse_nyse_calendar(
            html,
            NaiveDate::from_ymd_opt(2026, 11, 25).unwrap(),
            NaiveDate::from_ymd_opt(2026, 11, 27).unwrap(),
        )
        .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].session_kind, "early_close");
        assert_eq!(rows[1].close_time_et, "13:00");
    }

    #[test]
    fn validator_source_does_not_call_stage_102_parser_helpers() {
        let source = include_str!(
            "historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_output_validations.rs"
        );
        let imported_stage_102_symbols = source
            .split("market_data_parser_execution_attempts::{")
            .nth(1)
            .and_then(|value| value.split("};").next())
            .unwrap();
        for forbidden in [
            "parse_payloads",
            "parse_price_rows",
            "parse_dividend_rows",
            "parse_split_rows",
            "parse_nyse_calendar",
            "reconcile_calendar_and_prices",
        ] {
            assert!(
                !imported_stage_102_symbols.contains(forbidden),
                "forbidden helper import: {forbidden}"
            );
        }
        assert!(source.contains("independent_parse_payloads"));
        assert!(source.contains("independent_parse_nyse_calendar"));
    }

    #[test]
    fn stage_103_keeps_observation_and_trading_closed() {
        assert_eq!(
            NEXT_GATE,
            "stage_104_first_natural_forward_cycle_observation_input_admission_review"
        );
        assert!(POLICY_VERSION.contains("chain-external-full-reparse"));
        assert!(!VALIDATOR_IMPLEMENTATION_VERSION.is_empty());
    }
}
