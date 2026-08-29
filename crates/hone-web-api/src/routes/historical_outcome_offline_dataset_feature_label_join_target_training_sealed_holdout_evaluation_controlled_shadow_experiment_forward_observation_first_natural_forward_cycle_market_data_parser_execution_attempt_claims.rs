//! Stage 101 claim-first declaration for one exact market-data parser execution attempt.
//!
//! The immutable claim permanently consumes one unexpired Stage 100 authorization before any
//! parser entrypoint, runtime, raw-payload mount or read can exist. It freezes the exact Stage 94
//! independently validated input set by embedding only the already-persisted claim/receipt
//! metadata and hashes. This module never opens a raw payload, executes the parser, creates parsed
//! rows, starts observation, or grants portfolio/trading authority.

use std::collections::BTreeSet;
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_first_execution_authorizations::{
    ApprovedControlledShadowMarketDataParserFirstExecutionAuthorization,
    ControlledShadowMarketDataParserFirstExecutionAuthorizationReview,
    market_data_parser_first_execution_authorizations_for_future_claim,
    validate_market_data_parser_first_execution_authorization_for_claim,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_attempts::{
    ControlledShadowMarketDataReceiptClaim, RawMarketDataSourceReceipt,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_validations::{
    ControlledShadowMarketDataReceiptValidationRecord,
    IndependentlyValidatedControlledShadowMarketDataReceipt,
    independently_validated_market_data_receipts_for_parser_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-execution-attempt-claim-registry-v1";
const INPUT_MANIFEST_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-stage-94-fixed-input-manifest-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-execution-attempt-claim-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-market-data-parser-execution-attempt-claim-v1-create-once-consumes-stage-100-before-execution";
const NEXT_GATE: &str = "stage_102_single_claim_parser_execution_attempt";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct ClaimControlledShadowMarketDataParserExecutionAttemptRequest {
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_artifact_manifest_sha256: String,
    expected_stage_94_validation_sha256: String,
    expected_stage_93_claim_sha256: String,
    expected_stage_93_result_sha256: String,
    expected_stage_93_receipt_sha256: String,
    expected_canonical_request_set_sha256: String,
    expected_fixed_input_manifest_sha256: String,
    claim_reason: String,
    exact_current_stage_51_through_stage_100_binding_confirmed: bool,
    claimant_independent_from_stage_100_and_complete_prior_chain_confirmed: bool,
    authorization_unexpired_single_use_and_consumed_before_execution_confirmed: bool,
    current_server_rehashed_artifact_and_manifest_binding_confirmed: bool,
    fixed_stage_94_validated_input_set_content_addressed_and_read_only_confirmed: bool,
    claim_contains_metadata_and_hashes_but_does_not_open_raw_payloads_confirmed: bool,
    no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed: bool,
    future_output_create_once_untrusted_and_independently_validated_confirmed: bool,
    no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserFixedInputManifest {
    pub schema_version: String,
    pub input_manifest_sha256: String,
    pub stage_94_validation: ControlledShadowMarketDataReceiptValidationRecord,
    pub stage_93_claim: ControlledShadowMarketDataReceiptClaim,
    pub stage_93_result_sha256: String,
    pub stage_93_receipt_sha256: String,
    pub subject_symbols: Vec<String>,
    pub benchmark_symbol: String,
    pub window_start_date: NaiveDate,
    pub window_end_date: NaiveDate,
    pub subject_symbol_set_sha256: String,
    pub time_window_sha256: String,
    pub canonical_request_set_sha256: String,
    pub raw_payload_custody_manifest_sha256: String,
    pub raw_payloads: Vec<RawMarketDataSourceReceipt>,
    pub raw_payload_count: usize,
    pub total_response_bytes: usize,
    pub input_metadata_only: bool,
    pub raw_payloads_opened_by_claim: bool,
    pub fixed_stage_94_independently_validated_input: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserExecutionAttemptClaim {
    pub schema_version: String,
    pub policy_version: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub authorization: ControlledShadowMarketDataParserFirstExecutionAuthorizationReview,
    pub fixed_input_manifest: ControlledShadowMarketDataParserFixedInputManifest,
    pub claimed_at: DateTime<Utc>,
    pub claimed_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub claimant_independent_from_stage_100_and_complete_prior_chain: bool,
    pub claim_reason: String,
    pub exact_current_stage_51_through_stage_100_binding_confirmed: bool,
    pub authorization_unexpired_single_use_and_consumed_before_execution_confirmed: bool,
    pub current_server_rehashed_artifact_and_manifest_binding_confirmed: bool,
    pub fixed_stage_94_validated_input_set_content_addressed_and_read_only_confirmed: bool,
    pub claim_contains_metadata_and_hashes_but_does_not_open_raw_payloads_confirmed: bool,
    pub no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed: bool,
    pub future_output_create_once_untrusted_and_independently_validated_confirmed: bool,
    pub no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub authorization_consumed: bool,
    pub create_once: bool,
    pub claim_first: bool,
    pub task_status: String,
    pub execution_attempt_endpoint_available: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub raw_payload_mount_present: bool,
    pub raw_payload_read: bool,
    pub parser_executed: bool,
    pub parsed_calendar_rows_created: bool,
    pub parsed_market_rows_created: bool,
    pub output_written: bool,
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
pub(crate) struct ControlledShadowMarketDataParserExecutionAttemptClaimCandidate {
    pub authorization: ControlledShadowMarketDataParserFirstExecutionAuthorizationReview,
    pub fixed_input_manifest: ControlledShadowMarketDataParserFixedInputManifest,
    pub claimant_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserExecutionAttemptClaimRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub claim_endpoint_available: bool,
    pub eligible_authorizations:
        Vec<ControlledShadowMarketDataParserExecutionAttemptClaimCandidate>,
    pub claims: Vec<ControlledShadowMarketDataParserExecutionAttemptClaim>,
    pub authorization_candidate_count: usize,
    pub claim_eligible_count: usize,
    pub claim_count: usize,
    pub authorization_consumed_count: usize,
    pub waiting_for_stage_102_execution_count: usize,
    pub claim_status: String,
    pub next_gate: String,
    pub execution_attempt_endpoint_available: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub raw_payload_mount_present: bool,
    pub raw_payload_read: bool,
    pub parser_executed: bool,
    pub parsed_calendar_rows_created: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowMarketDataParserExecutionAttemptClaimReadinessSummary {
    pub authorization_candidate_count: usize,
    pub claim_eligible_count: usize,
    pub claim_count: usize,
    pub authorization_consumed_count: usize,
    pub waiting_for_stage_102_execution_count: usize,
    pub claim_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_parser_execution_attempt_claims(
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
            warn!(%error, "market-data parser execution-attempt claim registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "行情 parser 单次尝试声明暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_claim_controlled_shadow_market_data_parser_execution_attempt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(authorization_review_id): AxumPath<String>,
    Json(request): Json<ClaimControlledShadowMarketDataParserExecutionAttemptRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match claim_once(
        &state,
        &admin.user_id,
        &authorization_review_id,
        request,
        Utc::now(),
    )
    .await
    {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_parser_execution_attempt_claim_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserExecutionAttemptClaimReadinessSummary, String> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowMarketDataParserExecutionAttemptClaimReadinessSummary {
            authorization_candidate_count: value.authorization_candidate_count,
            claim_eligible_count: value.claim_eligible_count,
            claim_count: value.claim_count,
            authorization_consumed_count: value.authorization_consumed_count,
            waiting_for_stage_102_execution_count: value.waiting_for_stage_102_execution_count,
            claim_status: value.claim_status,
        },
    )
}

pub(crate) async fn claimed_market_data_parser_first_execution_authorization_review_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    Ok(
        market_data_parser_execution_attempt_claims_for_execution(state)
            .await?
            .into_iter()
            .map(|value| value.authorization.review_id)
            .collect(),
    )
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataParserExecutionAttemptClaimRegistry, String> {
    let claims = market_data_parser_execution_attempt_claims_for_execution(state).await?;
    let claimed = claims
        .iter()
        .map(|value| value.authorization.review_id.clone())
        .collect::<BTreeSet<_>>();
    let active =
        market_data_parser_first_execution_authorizations_for_future_claim(state, now).await?;
    let sources = independently_validated_market_data_receipts_for_parser_review(state).await?;
    let mut eligible_authorizations = Vec::new();
    for authorization in active {
        if claimed.contains(&authorization.review.review_id) {
            continue;
        }
        let source = exact_stage_94_source(&authorization, &sources)?;
        eligible_authorizations.push(
            ControlledShadowMarketDataParserExecutionAttemptClaimCandidate {
                claimant_excluded_actor_ids: expected_excluded_actor_ids(&authorization.review),
                fixed_input_manifest: build_fixed_input_manifest(&authorization, source)?,
                authorization: authorization.review,
            },
        );
    }
    let claim_count = claims.len();
    let claim_eligible_count = eligible_authorizations.len();
    let claim_status = if claim_count > 0 {
        "stage_100_authorization_consumed_waiting_for_separate_stage_102_execution_attempt"
    } else if claim_eligible_count > 0 {
        "eligible_for_claim_first_create_once_parser_attempt_declaration"
    } else {
        "waiting_for_unexpired_unconsumed_stage_100_authorization"
    };
    Ok(ControlledShadowMarketDataParserExecutionAttemptClaimRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        claim_endpoint_available: true,
        authorization_candidate_count: claim_eligible_count + claim_count,
        claim_eligible_count,
        claim_count,
        authorization_consumed_count: claims.len(),
        waiting_for_stage_102_execution_count: claims.len(),
        eligible_authorizations,
        claims,
        claim_status: claim_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        execution_attempt_endpoint_available: false,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        raw_payload_mount_present: false,
        raw_payload_read: false,
        parser_executed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 101 在任何 parser 入口、runtime、载荷挂载或读取出现之前，以 create-once 声明永久消费一条未过期 Stage 100 授权，并冻结同一当前工件与精确 Stage 94 已验证输入清单。声明只保存既有元数据和摘要，不打开原始载荷、不执行 parser、不创建解析行；后续最多只能另建 Stage 102 单次执行尝试。".to_string(),
    })
}

async fn claim_once(
    state: &AppState,
    claimant_id: &str,
    authorization_review_id: &str,
    request: ClaimControlledShadowMarketDataParserExecutionAttemptRequest,
    claimed_at: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataParserExecutionAttemptClaim, String> {
    if !valid_id(authorization_review_id) {
        return Err("Stage 100 授权 review ID 无效".to_string());
    }
    let _lock = acquire_lock(state, authorization_review_id).await?;
    if market_data_parser_execution_attempt_claims_for_execution(state)
        .await?
        .iter()
        .any(|value| value.authorization.review_id == authorization_review_id)
    {
        return Err("这条 Stage 100 授权已经被永久消费".to_string());
    }
    let authorization =
        market_data_parser_first_execution_authorizations_for_future_claim(state, claimed_at)
            .await?
            .into_iter()
            .find(|value| value.review.review_id == authorization_review_id)
            .ok_or_else(|| "当前没有这条未过期且未消费的 Stage 100 授权".to_string())?;
    validate_expected_binding(&authorization, &request)?;
    let sources = independently_validated_market_data_receipts_for_parser_review(state).await?;
    let source = exact_stage_94_source(&authorization, &sources)?;
    let fixed_input_manifest = build_fixed_input_manifest(&authorization, source)?;
    if request.expected_fixed_input_manifest_sha256 != fixed_input_manifest.input_manifest_sha256 {
        return Err("固定 Stage 94 输入清单已经变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(&authorization.review);
    let independent = !claimant_id.trim().is_empty()
        && !excluded_prior_actor_ids
            .iter()
            .any(|value| value == claimant_id);
    if request.claimant_independent_from_stage_100_and_complete_prior_chain_confirmed != independent
    {
        return Err("声明人独立性确认与完整 Stage 51–100 责任链不一致".to_string());
    }
    let mut claim = ControlledShadowMarketDataParserExecutionAttemptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        attempt_id: String::new(),
        claim_sha256: String::new(),
        authorization: authorization.review,
        fixed_input_manifest,
        claimed_at,
        claimed_by: claimant_id.to_string(),
        excluded_prior_actor_ids,
        claimant_independent_from_stage_100_and_complete_prior_chain: independent,
        claim_reason: bounded_required(&request.claim_reason, MAX_TEXT_CHARS, "声明原因")?,
        exact_current_stage_51_through_stage_100_binding_confirmed: request.exact_current_stage_51_through_stage_100_binding_confirmed,
        authorization_unexpired_single_use_and_consumed_before_execution_confirmed: request.authorization_unexpired_single_use_and_consumed_before_execution_confirmed,
        current_server_rehashed_artifact_and_manifest_binding_confirmed: request.current_server_rehashed_artifact_and_manifest_binding_confirmed,
        fixed_stage_94_validated_input_set_content_addressed_and_read_only_confirmed: request.fixed_stage_94_validated_input_set_content_addressed_and_read_only_confirmed,
        claim_contains_metadata_and_hashes_but_does_not_open_raw_payloads_confirmed: request.claim_contains_metadata_and_hashes_but_does_not_open_raw_payloads_confirmed,
        no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed: request.no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed,
        future_output_create_once_untrusted_and_independently_validated_confirmed: request.future_output_create_once_untrusted_and_independently_validated_confirmed,
        no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        authorization_consumed: true,
        create_once: true,
        claim_first: true,
        task_status: "authorization_consumed_not_executed_waiting_for_separate_stage_102_attempt".to_string(),
        execution_attempt_endpoint_available: false,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        raw_payload_mount_present: false,
        raw_payload_read: false,
        parser_executed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        output_written: false,
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
    claim.claim_sha256 = fingerprint_without(&claim, &["attempt_id", "claim_sha256"])?;
    claim.attempt_id = claim.claim_sha256[..32].to_string();
    validate_market_data_parser_execution_attempt_claim(&claim)?;
    write_immutable_json(
        &claim_directory(state).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;
    Ok(claim)
}

fn exact_stage_94_source<'a>(
    authorization: &ApprovedControlledShadowMarketDataParserFirstExecutionAuthorization,
    sources: &'a [IndependentlyValidatedControlledShadowMarketDataReceipt],
) -> Result<&'a IndependentlyValidatedControlledShadowMarketDataReceipt, String> {
    let expected = &authorization
        .runner
        .implementation
        .implementation_contract
        .validation_sha256;
    let matches = sources
        .iter()
        .filter(|value| value.validation.validation_sha256 == *expected)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("Stage 101 必须且只能绑定一条当前 Stage 94 独立验证输入".to_string());
    }
    Ok(matches[0])
}

fn build_fixed_input_manifest(
    authorization: &ApprovedControlledShadowMarketDataParserFirstExecutionAuthorization,
    source: &IndependentlyValidatedControlledShadowMarketDataReceipt,
) -> Result<ControlledShadowMarketDataParserFixedInputManifest, String> {
    let receipt = source
        .attempt
        .result
        .untrusted_raw_market_data_receipt
        .as_ref()
        .ok_or_else(|| "Stage 94 输入缺少 Stage 93 原始收据".to_string())?;
    let contract = &authorization.runner.implementation.implementation_contract;
    if source.validation.validation_sha256 != contract.validation_sha256
        || source.attempt.claim.claim_sha256 != contract.claim_sha256
        || source.attempt.result.result_sha256 != contract.result_sha256
        || receipt.receipt_sha256 != contract.receipt_sha256
    {
        return Err("Stage 94 输入与当前 parser runner 合同绑定不一致".to_string());
    }
    let mut manifest = ControlledShadowMarketDataParserFixedInputManifest {
        schema_version: INPUT_MANIFEST_SCHEMA_VERSION.to_string(),
        input_manifest_sha256: String::new(),
        stage_94_validation: source.validation.clone(),
        stage_93_claim: source.attempt.claim.clone(),
        stage_93_result_sha256: source.attempt.result.result_sha256.clone(),
        stage_93_receipt_sha256: receipt.receipt_sha256.clone(),
        subject_symbols: source.attempt.claim.subject_symbols.clone(),
        benchmark_symbol: source.attempt.claim.benchmark_symbol.clone(),
        window_start_date: source.attempt.claim.window_start_date,
        window_end_date: source.attempt.claim.window_end_date,
        subject_symbol_set_sha256: source.attempt.claim.subject_symbol_set_sha256.clone(),
        time_window_sha256: source.attempt.claim.time_window_sha256.clone(),
        canonical_request_set_sha256: source.attempt.claim.canonical_request_set_sha256.clone(),
        raw_payload_custody_manifest_sha256: source
            .validation
            .raw_payload_custody_manifest_sha256
            .clone(),
        raw_payloads: receipt.source_receipts.clone(),
        raw_payload_count: receipt.raw_payload_count,
        total_response_bytes: receipt.total_response_bytes,
        input_metadata_only: true,
        raw_payloads_opened_by_claim: false,
        fixed_stage_94_independently_validated_input: true,
    };
    manifest.input_manifest_sha256 = fingerprint_without(&manifest, &["input_manifest_sha256"])?;
    validate_fixed_input_manifest(&manifest, &authorization.review)?;
    Ok(manifest)
}

fn validate_expected_binding(
    authorization: &ApprovedControlledShadowMarketDataParserFirstExecutionAuthorization,
    request: &ClaimControlledShadowMarketDataParserExecutionAttemptRequest,
) -> Result<(), String> {
    let review = &authorization.review;
    let contract = &authorization.runner.implementation.implementation_contract;
    if request.expected_authorization_review_sha256 != review.review_sha256
        || request.expected_isolated_runner_spec_sha256
            != authorization.runner.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != review.server_computed_artifact_sha256
        || request.expected_artifact_manifest_sha256 != review.artifact_manifest.manifest_sha256
        || request.expected_stage_94_validation_sha256 != contract.validation_sha256
        || request.expected_stage_93_claim_sha256 != contract.claim_sha256
        || request.expected_stage_93_result_sha256 != contract.result_sha256
        || request.expected_stage_93_receipt_sha256 != contract.receipt_sha256
        || request.expected_canonical_request_set_sha256
            != authorization
                .runner
                .implementation
                .upstream_specification_registration
                .parser_specification
                .canonical_request_set_sha256
    {
        return Err("Stage 94–100 授权、工件或固定输入绑定已经变化".to_string());
    }
    Ok(())
}

fn validate_fixed_input_manifest(
    manifest: &ControlledShadowMarketDataParserFixedInputManifest,
    authorization: &ControlledShadowMarketDataParserFirstExecutionAuthorizationReview,
) -> Result<(), String> {
    let contract = &authorization.runner.implementation.implementation_contract;
    let valid = manifest.schema_version == INPUT_MANIFEST_SCHEMA_VERSION
        && valid_sha256(&manifest.input_manifest_sha256)
        && manifest.input_manifest_sha256
            == fingerprint_without(manifest, &["input_manifest_sha256"])?
        && manifest.stage_94_validation.validation_sha256 == contract.validation_sha256
        && manifest.stage_93_claim.claim_sha256 == contract.claim_sha256
        && manifest.stage_93_result_sha256 == contract.result_sha256
        && manifest.stage_93_receipt_sha256 == contract.receipt_sha256
        && manifest.subject_symbols == manifest.stage_93_claim.subject_symbols
        && manifest.benchmark_symbol == manifest.stage_93_claim.benchmark_symbol
        && manifest.window_start_date == manifest.stage_93_claim.window_start_date
        && manifest.window_end_date == manifest.stage_93_claim.window_end_date
        && manifest.subject_symbol_set_sha256 == manifest.stage_93_claim.subject_symbol_set_sha256
        && manifest.time_window_sha256 == manifest.stage_93_claim.time_window_sha256
        && manifest.canonical_request_set_sha256
            == manifest.stage_93_claim.canonical_request_set_sha256
        && manifest.raw_payload_custody_manifest_sha256
            == manifest
                .stage_94_validation
                .raw_payload_custody_manifest_sha256
        && manifest.raw_payload_count == manifest.raw_payloads.len()
        && manifest.raw_payload_count > 0
        && manifest.raw_payloads.iter().all(|value| {
            valid_sha256(&value.response_body_sha256)
                && valid_sha256(&value.source_document_sha256)
                && valid_sha256(&value.canonical_request_sha256)
                && !value.raw_payload_relative_path.trim().is_empty()
        })
        && manifest.input_metadata_only
        && !manifest.raw_payloads_opened_by_claim
        && manifest.fixed_stage_94_independently_validated_input;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 101 固定输入清单无效或漂移".to_string())
}

pub(crate) fn validate_market_data_parser_execution_attempt_claim(
    claim: &ControlledShadowMarketDataParserExecutionAttemptClaim,
) -> Result<(), String> {
    validate_market_data_parser_first_execution_authorization_for_claim(&claim.authorization)?;
    validate_fixed_input_manifest(&claim.fixed_input_manifest, &claim.authorization)?;
    let expected_excluded = expected_excluded_actor_ids(&claim.authorization);
    let independent = !claim
        .excluded_prior_actor_ids
        .iter()
        .any(|value| value == &claim.claimed_by);
    let confirmations = claim.exact_current_stage_51_through_stage_100_binding_confirmed
        && claim.authorization_unexpired_single_use_and_consumed_before_execution_confirmed
        && claim.current_server_rehashed_artifact_and_manifest_binding_confirmed
        && claim.fixed_stage_94_validated_input_set_content_addressed_and_read_only_confirmed
        && claim.claim_contains_metadata_and_hashes_but_does_not_open_raw_payloads_confirmed
        && claim.no_entrypoint_runtime_mount_payload_read_parser_execution_or_parsed_rows_confirmed
        && claim.future_output_create_once_untrusted_and_independently_validated_confirmed
        && claim.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && claim.no_unconfirmed_hari_or_old_wang_logic_claimed;
    let authority_closed = !claim.execution_attempt_endpoint_available
        && !claim.callable_entrypoint_present
        && !claim.runtime_instantiated
        && !claim.raw_payload_mount_present
        && !claim.raw_payload_read
        && !claim.parser_executed
        && !claim.parsed_calendar_rows_created
        && !claim.parsed_market_rows_created
        && !claim.output_written
        && !claim.forward_observation_started
        && !claim.ledger_created
        && !claim.position_written
        && !claim.performance_metric_written
        && !claim.model_store_written
        && !claim.metric_store_written
        && !claim.training_feedback_authorized
        && !claim.reward_authorized
        && !claim.order_generation_authorized
        && !claim.broker_access_authorized
        && !claim.trading_authorized;
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.policy_version != POLICY_VERSION
        || !valid_id(&claim.attempt_id)
        || !valid_sha256(&claim.claim_sha256)
        || claim.attempt_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != fingerprint_without(claim, &["attempt_id", "claim_sha256"])?
        || claim.claimed_at < claim.authorization.submitted_at
        || claim.claimed_at >= claim.authorization.authorization_valid_until
        || claim.claimed_by.trim().is_empty()
        || claim.claim_reason.trim().is_empty()
        || claim.claim_reason.chars().count() > MAX_TEXT_CHARS
        || claim.excluded_prior_actor_ids != expected_excluded
        || !independent
        || !claim.claimant_independent_from_stage_100_and_complete_prior_chain
        || !confirmations
        || !claim.authorization_consumed
        || !claim.create_once
        || !claim.claim_first
        || claim.task_status
            != "authorization_consumed_not_executed_waiting_for_separate_stage_102_attempt"
        || !authority_closed
    {
        return Err("Stage 101 parser 单次尝试声明无效、漂移或越权".to_string());
    }
    Ok(())
}

fn expected_excluded_actor_ids(
    review: &ControlledShadowMarketDataParserFirstExecutionAuthorizationReview,
) -> Vec<String> {
    let mut values = review.excluded_prior_actor_ids.clone();
    values.push(review.reviewer_id.clone());
    values.push(review.artifact_manifest.reproduced_by.clone());
    values.push(review.runner.registered_by.clone());
    values.push(review.runner.implementation_review.reviewer_id.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

pub(crate) async fn market_data_parser_execution_attempt_claims_for_execution(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserExecutionAttemptClaim>, String> {
    let mut directory = match tokio::fs::read_dir(claim_directory(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut claims = Vec::new();
    let mut ids = BTreeSet::new();
    let mut reviews = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let claim: ControlledShadowMarketDataParserExecutionAttemptClaim = serde_json::from_slice(
            &tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        validate_market_data_parser_execution_attempt_claim(&claim)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(claim.attempt_id.as_str())
            || !ids.insert(claim.attempt_id.clone())
            || !reviews.insert(claim.authorization.review_id.clone())
        {
            return Err("Stage 101 声明重复、重放或文件名错误".to_string());
        }
        claims.push(claim);
    }
    claims.sort_by(|left, right| left.claimed_at.cmp(&right.claimed_at));
    Ok(claims)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 101 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?,)
    ))
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars {
        Err(format!("{label}为空或过长"))
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

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn claim_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-market-data-parser-execution-attempt-claims")
}

struct ClaimLock(PathBuf);
impl Drop for ClaimLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
async fn acquire_lock(state: &AppState, review_id: &str) -> Result<ClaimLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-market-data-parser-stage-101-{review_id}.lock"
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
        .map_err(|_| "已有同一 Stage 101 声明正在进行".to_string())?;
    Ok(ClaimLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_101_is_claim_first_and_stage_102_is_separate() {
        assert!(POLICY_VERSION.contains("consumes-stage-100-before-execution"));
        assert_eq!(NEXT_GATE, "stage_102_single_claim_parser_execution_attempt");
    }

    #[test]
    fn fingerprints_ignore_only_declared_identity_fields() {
        #[derive(Serialize)]
        struct Sample {
            id: String,
            sha: String,
            value: u8,
        }
        let left = Sample {
            id: "a".into(),
            sha: "b".into(),
            value: 7,
        };
        let right = Sample {
            id: "x".into(),
            sha: "y".into(),
            value: 7,
        };
        assert_eq!(
            fingerprint_without(&left, &["id", "sha"]).unwrap(),
            fingerprint_without(&right, &["id", "sha"]).unwrap(),
        );
    }

    #[test]
    fn identity_formats_are_fail_closed() {
        assert!(valid_sha256(&"a".repeat(64)));
        assert!(!valid_sha256(&"a".repeat(63)));
        assert!(valid_id(&"b".repeat(32)));
        assert!(!valid_id("not-an-id"));
    }
}
