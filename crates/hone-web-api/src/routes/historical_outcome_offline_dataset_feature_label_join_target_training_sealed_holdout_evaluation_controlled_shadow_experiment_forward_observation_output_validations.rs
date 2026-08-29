//! Stage 89 chain-external validation for Stage 88 zero-market initialization receipts.
//!
//! The validator reconstructs the exact Stage 88 manifest and expected receipt from immutable
//! upstream records. It never opens a market-data source, starts a runtime or observation, creates
//! a ledger/position/performance record, or grants model, reward, order, broker or trade authority.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_execution_attempts::{
    CompletedControlledShadowForwardObservationExecutionAttempt,
    ControlledShadowForwardObservationExecutionAttemptStatus,
    ControlledShadowForwardObservationInitializationManifest,
    ControlledShadowForwardObservationUntrustedInitializationReceipt,
    completed_forward_observation_execution_attempts_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_execution_authorizations::{
    ApprovedControlledShadowForwardObservationFirstExecutionAuthorization,
    exact_forward_observation_first_execution_authorization_for_audit,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-output-validation-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-forward-observation-zero-market-receipt-chain-external-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str = "hone-controlled-shadow-forward-observation-zero-market-receipt-validator-v1-independent-reconstruction";
const MANIFEST_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-initialization-manifest-v1-no-market-data";
const RECEIPT_SCHEMA_VERSION: &str = "hone-controlled-shadow-forward-observation-untrusted-initialization-receipt-v2-reconstructible-manifest";
const MAX_CLOCK_SKEW_MINUTES: i64 = 5;
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateControlledShadowForwardObservationOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_protocol_specification_sha256: String,
    expected_design_specification_sha256: String,
    expected_initial_observation_validation_sha256: String,
    expected_initialization_manifest_sha256: String,
    independent_reopen_and_manifest_receipt_reconstruction_confirmed: bool,
    exact_current_stage_51_through_stage_88_binding_confirmed: bool,
    validator_independent_from_executor_stage_87_and_complete_prior_chain_confirmed: bool,
    claim_first_ordering_and_single_terminal_result_confirmed: bool,
    zero_market_data_natural_forward_only_and_no_backfill_confirmed: bool,
    official_calendar_https_content_hash_and_spy_confirmed: bool,
    zero_runtime_observation_ledger_position_and_performance_confirmed: bool,
    no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    validation_only_opens_future_first_natural_forward_cycle_review_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowForwardObservationOutputValidationVerdict {
    IndependentlyValidatedZeroMarketInitializationReceipt,
    FailedIndependentZeroMarketInitializationReceiptValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowForwardObservationOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub implementation_contract_sha256: String,
    pub protocol_specification_sha256: String,
    pub design_specification_sha256: String,
    pub initial_observation_validation_sha256: String,
    pub initialization_manifest_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_executor_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_88_chain_verified: bool,
    pub claim_fingerprint_independently_verified: bool,
    pub result_fingerprint_independently_verified: bool,
    pub receipt_fingerprint_independently_verified: bool,
    pub initialization_manifest_fingerprint_independently_reconstructed: bool,
    pub expected_receipt_independently_reconstructed: bool,
    pub claim_first_ordering_verified: bool,
    pub single_terminal_result_and_no_replay_verified: bool,
    pub exact_runner_binary_attestation_binding_verified: bool,
    pub natural_forward_protocol_and_time_bounds_verified: bool,
    pub official_calendar_and_spy_contract_verified: bool,
    pub zero_market_rows_sessions_runtime_observation_ledger_position_and_performance_verified:
        bool,
    pub no_downstream_authority_verified: bool,
    pub recomputed_claim_sha256: String,
    pub recomputed_result_sha256: String,
    pub recomputed_receipt_sha256: String,
    pub reconstructed_initialization_manifest_sha256: String,
    pub mismatch_reasons: Vec<String>,
    pub verdict: ControlledShadowForwardObservationOutputValidationVerdict,
    pub initialization_receipt_independently_validated: bool,
    pub future_first_natural_forward_cycle_authorization_review_eligible: bool,
    pub persistent_runtime_instantiated: bool,
    pub market_data_accessed: bool,
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
pub(crate) struct ControlledShadowForwardObservationOutputValidationItem {
    pub attempt: CompletedControlledShadowForwardObservationExecutionAttempt,
    pub validation: Option<ControlledShadowForwardObservationOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<ControlledShadowForwardObservationOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_initialization_receipt_count: usize,
    pub failed_validation_count: usize,
    pub future_first_natural_forward_cycle_authorization_review_eligible_count: usize,
    pub validation_status: String,
    pub independent_output_validation_available: bool,
    pub persistent_runtime_instantiated: bool,
    pub market_data_accessed: bool,
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
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowForwardObservationOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_initialization_receipt_count: usize,
    pub failed_validation_count: usize,
    pub future_first_natural_forward_cycle_authorization_review_eligible_count: usize,
    pub validation_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct IndependentlyValidatedControlledShadowForwardObservationInitialization {
    pub attempt: CompletedControlledShadowForwardObservationExecutionAttempt,
    pub validation: ControlledShadowForwardObservationOutputValidationRecord,
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    receipt_fingerprint: bool,
    manifest_fingerprint: bool,
    expected_receipt: bool,
    claim_first_ordering: bool,
    single_terminal_result_no_replay: bool,
    binary_attestation_binding: bool,
    natural_forward_time_bounds: bool,
    official_calendar_spy: bool,
    zero_runtime_and_data: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_receipt_sha256: String,
    reconstructed_manifest_sha256: String,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_controlled_shadow_forward_observation_output_validations(
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
            warn!(%error, "forward-observation output validation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "前向观察零行情初始化收据独立验证暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_controlled_shadow_forward_observation_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateControlledShadowForwardObservationOutputRequest>,
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

pub(crate) async fn controlled_shadow_forward_observation_output_validation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationOutputValidationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowForwardObservationOutputValidationReadinessSummary {
            validation_eligible_count: value.validation_eligible_count,
            validation_count: value.validation_count,
            independently_validated_initialization_receipt_count: value
                .independently_validated_initialization_receipt_count,
            failed_validation_count: value.failed_validation_count,
            future_first_natural_forward_cycle_authorization_review_eligible_count: value
                .future_first_natural_forward_cycle_authorization_review_eligible_count,
            validation_status: value.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_forward_observation_initializations_for_first_cycle_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedControlledShadowForwardObservationInitialization>, String> {
    let registry = build_registry(state).await?;
    let mut values = registry
        .items
        .into_iter()
        .filter_map(|item| {
            let validation = item.validation?;
            (validation.initialization_receipt_independently_validated
                && validation.future_first_natural_forward_cycle_authorization_review_eligible)
                .then_some(
                    IndependentlyValidatedControlledShadowForwardObservationInitialization {
                        attempt: item.attempt,
                        validation,
                    },
                )
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| {
        left.validation
            .validation_id
            .cmp(&right.validation.validation_id)
    });
    Ok(values)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationOutputValidationRegistry, String> {
    let attempts =
        completed_forward_observation_execution_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.attempt_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    let items = attempts
        .into_iter()
        .map(|attempt| {
            let validation = index
                .get(attempt.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            ControlledShadowForwardObservationOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let independently_validated_initialization_receipt_count = validations
        .iter()
        .filter(|record| record.initialization_receipt_independently_validated)
        .count();
    let failed_validation_count = validations
        .len()
        .saturating_sub(independently_validated_initialization_receipt_count);
    let validation_status = if failed_validation_count > 0 {
        "forward_observation_zero_market_initialization_validation_failed_fail_closed"
    } else if independently_validated_initialization_receipt_count > 0 {
        "zero_market_initialization_receipt_independently_validated_waiting_first_natural_forward_cycle_review"
    } else if validation_eligible_count > 0 {
        "waiting_chain_external_zero_market_initialization_receipt_validation"
    } else {
        "waiting_completed_stage_88_zero_market_initialization_receipt"
    };
    Ok(ControlledShadowForwardObservationOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count: validations.len(),
        independently_validated_initialization_receipt_count,
        failed_validation_count,
        future_first_natural_forward_cycle_authorization_review_eligible_count:
            independently_validated_initialization_receipt_count,
        validation_status: validation_status.to_string(),
        independent_output_validation_available: true,
        persistent_runtime_instantiated: false,
        market_data_accessed: false,
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
        scope: "Stage 89 由 Stage 88 executor、Stage 87 reviewer 和完整 Stage 51–88 责任链之外的新管理员，独立重建零行情初始化 manifest 与预期收据，并验证 claim-first、单一终态、自然前向、官方日历/SPY 和全部零权限位。通过只开放未来首个自然前向周期授权复核资格；当前仍不创建 runtime、读取行情、开始观察、写账本/持仓/绩效、训练/reward、订单、券商或交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateControlledShadowForwardObservationOutputRequest,
) -> Result<ControlledShadowForwardObservationOutputValidationRecord, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.attempt_id == attempt_id)
    {
        return Err("这次 Stage 88 初始化收据已经形成不可变独立验证记录，不得重放".to_string());
    }
    let attempt =
        completed_forward_observation_execution_attempts_for_independent_validation(state)
            .await?
            .into_iter()
            .find(|value| value.claim.attempt_id == attempt_id)
            .ok_or_else(|| "当前没有完整且待独立验证的 Stage 88 初始化收据".to_string())?;
    verify_expected_bindings(&attempt, &request)?;
    let authorization = exact_forward_observation_first_execution_authorization_for_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "Stage 88 绑定的 Stage 87 授权完整链已经无法重开".to_string())?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "Stage 89 验证者不能是 Stage 88 executor、Stage 87 reviewer 或完整 Stage 51–88 责任链任一角色"
                .to_string(),
        );
    }
    let audit = independently_audit(&attempt, &authorization);
    let passed = independent
        && audit.exact_chain
        && audit.claim_fingerprint
        && audit.result_fingerprint
        && audit.receipt_fingerprint
        && audit.manifest_fingerprint
        && audit.expected_receipt
        && audit.claim_first_ordering
        && audit.single_terminal_result_no_replay
        && audit.binary_attestation_binding
        && audit.natural_forward_time_bounds
        && audit.official_calendar_spy
        && audit.zero_runtime_and_data
        && audit.downstream_closed
        && audit.mismatch_reasons.is_empty();
    let receipt = attempt
        .result
        .untrusted_initialization_receipt
        .as_ref()
        .ok_or_else(|| "Stage 88 成功结果缺少不可信初始化收据".to_string())?;
    let mut record = ControlledShadowForwardObservationOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        output_sha256: attempt.result.output_sha256.clone().unwrap_or_default(),
        authorization_review_id: attempt.claim.authorization_review_id.clone(),
        authorization_review_sha256: attempt.claim.authorization_review_sha256.clone(),
        isolated_runner_id: attempt.claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: attempt.claim.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: attempt.claim.runner_artifact_sha256.clone(),
        implementation_contract_sha256: attempt.claim.implementation_contract_sha256.clone(),
        protocol_specification_sha256: attempt.claim.protocol_specification_sha256.clone(),
        design_specification_sha256: attempt.claim.design_specification_sha256.clone(),
        initial_observation_validation_sha256: attempt
            .claim
            .initial_observation_validation_sha256
            .clone(),
        initialization_manifest_sha256: receipt.initialization_manifest_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        invoked_by: attempt.claim.invoked_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_executor_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_88_chain_verified: audit.exact_chain,
        claim_fingerprint_independently_verified: audit.claim_fingerprint,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        receipt_fingerprint_independently_verified: audit.receipt_fingerprint,
        initialization_manifest_fingerprint_independently_reconstructed: audit.manifest_fingerprint,
        expected_receipt_independently_reconstructed: audit.expected_receipt,
        claim_first_ordering_verified: audit.claim_first_ordering,
        single_terminal_result_and_no_replay_verified: audit.single_terminal_result_no_replay,
        exact_runner_binary_attestation_binding_verified: audit.binary_attestation_binding,
        natural_forward_protocol_and_time_bounds_verified: audit.natural_forward_time_bounds,
        official_calendar_and_spy_contract_verified: audit.official_calendar_spy,
        zero_market_rows_sessions_runtime_observation_ledger_position_and_performance_verified:
            audit.zero_runtime_and_data,
        no_downstream_authority_verified: audit.downstream_closed,
        recomputed_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_result_sha256: audit.recomputed_result_sha256,
        recomputed_receipt_sha256: audit.recomputed_receipt_sha256,
        reconstructed_initialization_manifest_sha256: audit.reconstructed_manifest_sha256,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            ControlledShadowForwardObservationOutputValidationVerdict::IndependentlyValidatedZeroMarketInitializationReceipt
        } else {
            ControlledShadowForwardObservationOutputValidationVerdict::FailedIndependentZeroMarketInitializationReceiptValidation
        },
        initialization_receipt_independently_validated: passed,
        future_first_natural_forward_cycle_authorization_review_eligible: passed,
        persistent_runtime_instantiated: false,
        market_data_accessed: false,
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

fn independently_audit(
    attempt: &CompletedControlledShadowForwardObservationExecutionAttempt,
    authorization: &ApprovedControlledShadowForwardObservationFirstExecutionAuthorization,
) -> IndependentAudit {
    let mut reasons = Vec::new();
    let recomputed_claim_sha256 =
        independent_fingerprint_without(&attempt.claim, &["attempt_id", "claim_sha256"])
            .unwrap_or_else(|error| {
                reasons.push(format!("claim fingerprint recomputation failed: {error}"));
                hash_bytes(b"claim-fingerprint-unavailable")
            });
    let recomputed_result_sha256 =
        independent_fingerprint_without(&attempt.result, &["result_id", "result_sha256"])
            .unwrap_or_else(|error| {
                reasons.push(format!("result fingerprint recomputation failed: {error}"));
                hash_bytes(b"result-fingerprint-unavailable")
            });
    let receipt = attempt.result.untrusted_initialization_receipt.as_ref();
    let recomputed_receipt_sha256 = receipt
        .and_then(|value| independent_hash_serializable(value).ok())
        .unwrap_or_else(|| hash_bytes(b"receipt-unavailable"));
    let manifest = receipt.map(reconstruct_manifest);
    let reconstructed_manifest_sha256 = manifest
        .as_ref()
        .and_then(|value| independent_fingerprint_without(value, &["manifest_sha256"]).ok())
        .unwrap_or_else(|| hash_bytes(b"manifest-unavailable"));

    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
    let registration = &implementation.upstream_protocol_registration;
    let protocol = &registration.protocol_specification;
    let exact_chain = attempt.claim.authorization_review_id == authorization.review.review_id
        && attempt.claim.authorization_review_sha256 == authorization.review.review_sha256
        && attempt.claim.authorization_valid_until
            == authorization.review.authorization_valid_until
        && attempt.claim.isolated_runner_id == runner.isolated_runner_id
        && attempt.claim.isolated_runner_spec_sha256 == runner.isolated_runner_spec_sha256
        && attempt.claim.runner_contract_sha256 == runner.runner_contract.contract_sha256
        && attempt.claim.runner_code_revision == runner.runner_code_revision
        && attempt.claim.runner_artifact_sha256 == runner.runner_artifact_sha256
        && attempt.claim.implementation_id == implementation.implementation_id
        && attempt.claim.implementation_sha256 == implementation.implementation_sha256
        && attempt.claim.implementation_contract_sha256 == contract.contract_sha256
        && attempt.claim.implementation_review_sha256 == runner.implementation_review.review_sha256
        && attempt.claim.protocol_review_sha256
            == implementation.upstream_protocol_review.review_sha256
        && attempt.claim.protocol_registration_sha256 == registration.protocol_registration_sha256
        && attempt.claim.protocol_specification_sha256 == protocol.specification_sha256
        && attempt.claim.design_specification_sha256
            == protocol.exact_design_specification.specification_sha256
        && attempt.claim.initial_observation_validation_sha256 == contract.validation_sha256;
    let claim_fingerprint = recomputed_claim_sha256 == attempt.claim.claim_sha256
        && attempt.claim.attempt_id == attempt.claim.claim_sha256[..32];
    let result_fingerprint = recomputed_result_sha256 == attempt.result.result_sha256
        && attempt.result.result_id == attempt.result.result_sha256[..32];
    let receipt_fingerprint =
        attempt.result.output_sha256.as_deref() == Some(recomputed_receipt_sha256.as_str());
    let manifest_fingerprint = manifest.as_ref().is_some_and(|value| {
        value.manifest_sha256 == reconstructed_manifest_sha256
            && value.manifest_sha256 == attempt.claim.initialization_manifest_sha256
    });
    let expected_receipt = match (&manifest, receipt) {
        (Some(manifest), Some(receipt)) => {
            independently_expected_receipt(authorization, manifest) == *receipt
        }
        _ => false,
    };
    let claim_first_ordering = !attempt.claim.initialization_manifest_opened
        && attempt.claim.authorization_consumed
        && attempt.claim.invocation_started
        && attempt.result.started_at >= attempt.claim.claimed_at
        && attempt.result.finished_at >= attempt.result.started_at;
    let single_terminal_result_no_replay = attempt.result.attempt_id == attempt.claim.attempt_id
        && attempt.result.claim_sha256 == attempt.claim.claim_sha256
        && attempt.result.status
            == ControlledShadowForwardObservationExecutionAttemptStatus::CompletedWithUntrustedInitializationReceipt
        && attempt.result.exit_code == 0
        && attempt.result.failure_reason.is_none();
    let binary_attestation_binding = attempt.result.current_binary_digest_reverified
        && receipt.is_some_and(|value| {
            value.runner_artifact_sha256 == attempt.claim.runner_artifact_sha256
                && value.runner_artifact_sha256 == runner.runner_artifact_sha256
        });
    let natural_forward_time_bounds = manifest.as_ref().is_some_and(|value| {
        value.requested_at >= value.observation_not_before
            && value.requested_at
                >= attempt.result.started_at - TimeDelta::minutes(MAX_CLOCK_SKEW_MINUTES)
            && value.requested_at
                <= attempt.result.started_at + TimeDelta::minutes(MAX_CLOCK_SKEW_MINUTES)
            && value.observation_not_before == protocol.observation_not_before
            && value.signal_cadence == protocol.signal_cadence
            && value.first_eligible_signal_rule == protocol.first_eligible_signal_rule
            && value.natural_forward_only
            && !value.retroactive_backfill_allowed
            && value.point_in_time_content_addressed_allowlisted_sources_required
            && value.synchronized_security_and_benchmark_observation_required
            && value.initialization_only
    });
    let official_calendar_spy = manifest.as_ref().is_some_and(|value| {
        value.official_market_calendar == protocol.official_market_calendar
            && value
                .official_market_calendar_source_url
                .starts_with("https://")
            && valid_sha256(&value.official_market_calendar_content_sha256)
            && value.benchmark_symbol == "SPY"
            && value.benchmark_symbol == protocol.benchmark_symbol
    });
    let zero_runtime_and_data = receipt.is_some_and(|value| {
        !value.market_data_rows_attached
            && value.natural_forward_market_sessions_observed == 0
            && !value.persistent_runtime_instantiated
            && !value.input_manifest_attached
            && !value.market_data_accessed
            && !value.forward_observation_started
            && !value.ledger_created
            && !value.position_written
            && !value.performance_metric_written
    }) && !attempt.result.forward_observation_started
        && !attempt.result.ledger_created
        && !attempt.result.position_written
        && !attempt.result.performance_metric_written;
    let downstream_closed = receipt.is_some_and(|value| {
        value.output_is_untrusted
            && !value.independent_output_validation_completed
            && !value.model_store_written
            && !value.metric_store_written
            && !value.training_feedback_written
            && !value.scalar_reward_written
            && !value.order_intent_present
            && !value.broker_payload_present
            && !value.trade_executed
    }) && !attempt.result.independent_output_validation_completed
        && !attempt.result.model_store_written
        && !attempt.result.metric_store_written
        && !attempt.result.training_feedback_written
        && !attempt.result.scalar_reward_written
        && !attempt.result.order_generated
        && !attempt.result.broker_accessed
        && !attempt.result.trade_executed;

    push_mismatch(
        &mut reasons,
        exact_chain,
        "Stage 51–88 exact chain mismatch",
    );
    push_mismatch(
        &mut reasons,
        claim_fingerprint,
        "Stage 88 claim fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        result_fingerprint,
        "Stage 88 result fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        receipt_fingerprint,
        "Stage 88 receipt fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        manifest_fingerprint,
        "initialization manifest reconstruction mismatch",
    );
    push_mismatch(
        &mut reasons,
        expected_receipt,
        "independently reconstructed receipt mismatch",
    );
    push_mismatch(
        &mut reasons,
        claim_first_ordering,
        "claim-first ordering evidence mismatch",
    );
    push_mismatch(
        &mut reasons,
        single_terminal_result_no_replay,
        "terminal result or replay contract mismatch",
    );
    push_mismatch(
        &mut reasons,
        binary_attestation_binding,
        "runner binary attestation binding mismatch",
    );
    push_mismatch(
        &mut reasons,
        natural_forward_time_bounds,
        "natural-forward protocol or time bounds mismatch",
    );
    push_mismatch(
        &mut reasons,
        official_calendar_spy,
        "official calendar or SPY contract mismatch",
    );
    push_mismatch(
        &mut reasons,
        zero_runtime_and_data,
        "market data, runtime, observation, ledger, position or performance is not zero",
    );
    push_mismatch(
        &mut reasons,
        downstream_closed,
        "downstream authority is not closed",
    );

    IndependentAudit {
        exact_chain,
        claim_fingerprint,
        result_fingerprint,
        receipt_fingerprint,
        manifest_fingerprint,
        expected_receipt,
        claim_first_ordering,
        single_terminal_result_no_replay,
        binary_attestation_binding,
        natural_forward_time_bounds,
        official_calendar_spy,
        zero_runtime_and_data,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_receipt_sha256,
        reconstructed_manifest_sha256,
        mismatch_reasons: reasons,
    }
}

fn reconstruct_manifest(
    receipt: &ControlledShadowForwardObservationUntrustedInitializationReceipt,
) -> ControlledShadowForwardObservationInitializationManifest {
    ControlledShadowForwardObservationInitializationManifest {
        schema_version: MANIFEST_SCHEMA_VERSION.to_string(),
        manifest_sha256: receipt.initialization_manifest_sha256.clone(),
        requested_at: receipt.requested_at,
        observation_not_before: receipt.observation_not_before,
        signal_cadence: receipt.signal_cadence.clone(),
        first_eligible_signal_rule: receipt.first_eligible_signal_rule.clone(),
        official_market_calendar: receipt.official_market_calendar.clone(),
        official_market_calendar_source_url: receipt.official_market_calendar_source_url.clone(),
        official_market_calendar_content_sha256: receipt
            .official_market_calendar_content_sha256
            .clone(),
        benchmark_symbol: receipt.benchmark_symbol.clone(),
        initial_observation_validation_sha256: receipt
            .initial_observation_validation_sha256
            .clone(),
        natural_forward_only: receipt.natural_forward_only,
        retroactive_backfill_allowed: receipt.retroactive_backfill_allowed,
        market_data_rows_attached: receipt.market_data_rows_attached,
        point_in_time_content_addressed_allowlisted_sources_required: receipt
            .point_in_time_content_addressed_allowlisted_sources_required,
        synchronized_security_and_benchmark_observation_required: receipt
            .synchronized_security_and_benchmark_observation_required,
        initialization_only: receipt.initialization_only,
    }
}

fn independently_expected_receipt(
    authorization: &ApprovedControlledShadowForwardObservationFirstExecutionAuthorization,
    manifest: &ControlledShadowForwardObservationInitializationManifest,
) -> ControlledShadowForwardObservationUntrustedInitializationReceipt {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let protocol = &implementation
        .upstream_protocol_registration
        .protocol_specification;
    ControlledShadowForwardObservationUntrustedInitializationReceipt {
        schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
        implementation_contract_sha256: implementation
            .implementation_contract
            .contract_sha256
            .clone(),
        protocol_specification_sha256: protocol.specification_sha256.clone(),
        design_specification_sha256: protocol
            .exact_design_specification
            .specification_sha256
            .clone(),
        initialization_manifest_sha256: manifest.manifest_sha256.clone(),
        observation_not_before: manifest.observation_not_before,
        requested_at: manifest.requested_at,
        signal_cadence: manifest.signal_cadence.clone(),
        first_eligible_signal_rule: manifest.first_eligible_signal_rule.clone(),
        official_market_calendar: manifest.official_market_calendar.clone(),
        official_market_calendar_source_url: manifest.official_market_calendar_source_url.clone(),
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
    }
}

fn verify_expected_bindings(
    attempt: &CompletedControlledShadowForwardObservationExecutionAttempt,
    request: &ValidateControlledShadowForwardObservationOutputRequest,
) -> Result<(), String> {
    let output = attempt.result.output_sha256.as_deref().unwrap_or_default();
    if request.expected_claim_sha256 != attempt.claim.claim_sha256
        || request.expected_result_sha256 != attempt.result.result_sha256
        || request.expected_output_sha256 != output
        || request.expected_authorization_review_sha256 != attempt.claim.authorization_review_sha256
        || request.expected_isolated_runner_spec_sha256 != attempt.claim.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != attempt.claim.runner_artifact_sha256
        || request.expected_implementation_contract_sha256
            != attempt.claim.implementation_contract_sha256
        || request.expected_protocol_specification_sha256
            != attempt.claim.protocol_specification_sha256
        || request.expected_design_specification_sha256 != attempt.claim.design_specification_sha256
        || request.expected_initial_observation_validation_sha256
            != attempt.claim.initial_observation_validation_sha256
        || request.expected_initialization_manifest_sha256
            != attempt.claim.initialization_manifest_sha256
    {
        return Err("Stage 88 claim/result/receipt 或 Stage 51–87 预期绑定已经变化".to_string());
    }
    Ok(())
}

fn validate_request(
    attempt_id: &str,
    request: &ValidateControlledShadowForwardObservationOutputRequest,
) -> Result<(), String> {
    let hashes = [
        request.expected_claim_sha256.as_str(),
        request.expected_result_sha256.as_str(),
        request.expected_output_sha256.as_str(),
        request.expected_authorization_review_sha256.as_str(),
        request.expected_isolated_runner_spec_sha256.as_str(),
        request.expected_runner_artifact_sha256.as_str(),
        request.expected_implementation_contract_sha256.as_str(),
        request.expected_protocol_specification_sha256.as_str(),
        request.expected_design_specification_sha256.as_str(),
        request
            .expected_initial_observation_validation_sha256
            .as_str(),
        request.expected_initialization_manifest_sha256.as_str(),
    ];
    if !valid_id(attempt_id) || hashes.iter().any(|value| !valid_sha256(value)) {
        return Err("Stage 89 请求的 ID 或 SHA-256 无效".to_string());
    }
    let confirmed = request.independent_reopen_and_manifest_receipt_reconstruction_confirmed
        && request.exact_current_stage_51_through_stage_88_binding_confirmed
        && request.validator_independent_from_executor_stage_87_and_complete_prior_chain_confirmed
        && request.claim_first_ordering_and_single_terminal_result_confirmed
        && request.zero_market_data_natural_forward_only_and_no_backfill_confirmed
        && request.official_calendar_https_content_hash_and_spy_confirmed
        && request.zero_runtime_observation_ledger_position_and_performance_confirmed
        && request.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && request.validation_only_opens_future_first_natural_forward_cycle_review_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    if !confirmed {
        return Err(
            "必须确认责任链外重建、claim-first、零行情、自然前向和全部下游权限关闭".to_string(),
        );
    }
    Ok(())
}

fn prior_actor_ids(
    attempt: &CompletedControlledShadowForwardObservationExecutionAttempt,
    authorization: &ApprovedControlledShadowForwardObservationFirstExecutionAuthorization,
) -> Vec<String> {
    let mut values = attempt.claim.excluded_prior_actor_ids.clone();
    values.extend(authorization.review.excluded_prior_actor_ids.clone());
    values.push(authorization.review.reviewer_id.clone());
    values.push(attempt.claim.invoked_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn validate_record(
    record: &ControlledShadowForwardObservationOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_executor_and_complete_prior_chain
        && record.exact_current_stage_51_through_stage_88_chain_verified
        && record.claim_fingerprint_independently_verified
        && record.result_fingerprint_independently_verified
        && record.receipt_fingerprint_independently_verified
        && record.initialization_manifest_fingerprint_independently_reconstructed
        && record.expected_receipt_independently_reconstructed
        && record.claim_first_ordering_verified
        && record.single_terminal_result_and_no_replay_verified
        && record.exact_runner_binary_attestation_binding_verified
        && record.natural_forward_protocol_and_time_bounds_verified
        && record.official_calendar_and_spy_contract_verified
        && record
            .zero_market_rows_sessions_runtime_observation_ledger_position_and_performance_verified
        && record.no_downstream_authority_verified
        && record.mismatch_reasons.is_empty();
    let authority_closed = !record.persistent_runtime_instantiated
        && !record.market_data_accessed
        && !record.forward_observation_started
        && !record.ledger_created
        && !record.position_written
        && !record.performance_metric_written
        && !record.model_store_written
        && !record.metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.validation_id)
        || record.validation_sha256 != record_fingerprint(record)?
        || record.validation_id != record.validation_sha256[..32]
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &record.validated_by)
        || !authority_closed
        || record.initialization_receipt_independently_validated != passed
        || record.future_first_natural_forward_cycle_authorization_review_eligible != passed
        || (passed
            != (record.verdict
                == ControlledShadowForwardObservationOutputValidationVerdict::IndependentlyValidatedZeroMarketInitializationReceipt))
    {
        return Err("Stage 89 独立验证记录无效、漂移或越权".to_string());
    }
    Ok(())
}

fn push_mismatch(reasons: &mut Vec<String>, condition: bool, message: &str) {
    if !condition && reasons.len() < MAX_MISMATCH_REASONS {
        reasons.push(message.to_string());
    }
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<ControlledShadowForwardObservationOutputValidationRecord>, String> {
    let mut records = Vec::new();
    let mut stack = vec![validation_root(state)];
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
                let record: ControlledShadowForwardObservationOutputValidationRecord =
                    serde_json::from_slice(
                        &tokio::fs::read(&path)
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
        if !attempts.insert(record.attempt_id.clone()) || !ids.insert(record.validation_id.clone())
        {
            return Err("Stage 89 独立验证记录重复".to_string());
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn record_fingerprint(
    record: &ControlledShadowForwardObservationOutputValidationRecord,
) -> Result<String, String> {
    independent_fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    hash_bytes(include_bytes!(
        "historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_output_validations.rs"
    ))
}

fn independent_fingerprint_without<T: Serialize>(
    value: &T,
    fields: &[&str],
) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    independent_hash_serializable(&value)
}

fn independent_hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
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

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-forward-observation-output-validations")
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
    let directory = validation_root(state).join("locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{attempt_id}.lock"));
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "另一个 Stage 89 验证正在处理该 attempt".to_string()
            } else {
                error.to_string()
            }
        })?;
    file.sync_all().await.map_err(|error| error.to_string())?;
    Ok(ValidationLock { path })
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

    #[test]
    fn reconstructed_manifest_fingerprint_detects_protocol_flag_tampering() {
        let now = Utc::now();
        let mut receipt = ControlledShadowForwardObservationUntrustedInitializationReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION.to_string(),
            authorization_review_sha256: "a".repeat(64),
            isolated_runner_spec_sha256: "b".repeat(64),
            runner_artifact_sha256: "c".repeat(64),
            implementation_contract_sha256: "d".repeat(64),
            protocol_specification_sha256: "e".repeat(64),
            design_specification_sha256: "f".repeat(64),
            initialization_manifest_sha256: String::new(),
            observation_not_before: now,
            requested_at: now,
            signal_cadence: "weekly".to_string(),
            first_eligible_signal_rule: "next_full_market_session".to_string(),
            official_market_calendar: "NYSE official calendar".to_string(),
            official_market_calendar_source_url: "https://www.nyse.com/markets/hours-calendars"
                .to_string(),
            official_market_calendar_content_sha256: "1".repeat(64),
            benchmark_symbol: "SPY".to_string(),
            initial_observation_validation_sha256: "2".repeat(64),
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
        let manifest = reconstruct_manifest(&receipt);
        receipt.initialization_manifest_sha256 =
            independent_fingerprint_without(&manifest, &["manifest_sha256"]).unwrap();
        let original = receipt.initialization_manifest_sha256.clone();
        receipt.retroactive_backfill_allowed = true;
        let tampered = reconstruct_manifest(&receipt);
        assert_ne!(
            original,
            independent_fingerprint_without(&tampered, &["manifest_sha256"]).unwrap()
        );
    }

    #[test]
    fn validator_role_must_be_outside_executor_and_prior_chain() {
        assert!(!actor_is_independent(
            "stage-88-executor",
            &[
                "stage-87-reviewer".to_string(),
                "stage-88-executor".to_string()
            ]
        ));
        assert!(actor_is_independent(
            "stage-89-validator",
            &[
                "stage-87-reviewer".to_string(),
                "stage-88-executor".to_string()
            ]
        ));
    }

    #[test]
    fn validator_implementation_identity_is_content_addressed() {
        assert!(valid_sha256(&validator_implementation_sha256()));
        assert_ne!(
            validator_implementation_sha256(),
            hash_bytes(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
        );
    }
}
