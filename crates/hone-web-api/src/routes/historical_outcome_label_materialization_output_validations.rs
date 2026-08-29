//! Independent validation of one untrusted label-materialization envelope.
//!
//! A validator who did not participate in materialization or the producing
//! evidence chain binds one exact immutable claim/result/output, reloads the
//! current admitted source, verifies structure and provenance, and compares
//! every frozen metric bitwise. Passing this gate still does not write a label
//! or authorize training, reward, shadow evidence, orders, broker access or
//! trading.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_dry_run_execution_attempts::HistoricalOutcomeDryRunMetric;
use super::historical_outcome_label_admission_reviews::{
    AdmittedHistoricalOutcomeForLabelMaterialization,
    exact_admitted_historical_outcome_for_label_materialization,
};
use super::historical_outcome_label_materialization_execution_attempts::{
    CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt,
    HistoricalOutcomeLabelMaterializationUntrustedEnvelope,
    completed_execution_attempt_for_independent_validation,
    completed_execution_attempts_for_independent_validation,
};
use super::historical_outcome_label_materialization_first_execution_authorizations::exact_first_execution_authorization_for_audit;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-output-validation-v1";
const VALIDATION_POLICY_VERSION: &str =
    "hone-label-materialization-independent-structure-provenance-bitwise-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-independent-materialized-envelope-validator-v1-no-projection-code-reuse";
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeLabelMaterializationOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_admission_review_sha256: String,
    expected_validation_sha256: String,
    expected_source_output_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    expected_recomputed_metrics_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeLabelMaterializationOutputValidationVerdict {
    ValidatedStructureProvenanceAndBitwiseMatch,
    FailedStructureProvenanceOrBitwiseMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub materialization_attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub materialization_implementation_id: String,
    pub materialization_implementation_spec_sha256: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub source_validation_id: String,
    pub source_validation_sha256: String,
    pub source_attempt_id: String,
    pub source_output_sha256: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub recomputed_metrics_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub materialization_invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_materialization_and_prior_chain: bool,
    pub immutable_chain_integrity_verified: bool,
    pub current_admitted_source_binding_verified: bool,
    pub canonical_output_hash_verified: bool,
    pub output_structure_verified: bool,
    pub provenance_match: bool,
    pub exact_metric_bits_match: bool,
    pub known_limitations_match: bool,
    pub independently_validated_metrics: Vec<HistoricalOutcomeDryRunMetric>,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeLabelMaterializationOutputValidationVerdict,
    pub untrusted_envelope_validated: bool,
    pub outcome_label_admission_authorized: bool,
    pub outcome_label_write_allowed: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationOutputValidationItem {
    pub attempt: CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt,
    pub validation: Option<HistoricalOutcomeLabelMaterializationOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeLabelMaterializationOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub validated_envelope_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
    pub output_validation_available: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeLabelMaterializationOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub validated_envelope_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
    pub outcome_label_generation_enabled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope {
    pub attempt: CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt,
    pub validation: HistoricalOutcomeLabelMaterializationOutputValidationRecord,
}

pub(crate) async fn handle_get_historical_outcome_label_materialization_output_validations(
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
            warn!(%error, "historical label materialization output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果标签物化输出独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_label_materialization_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeLabelMaterializationOutputRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match validate_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => {
                warn!(%error, "historical label materialization output validation registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果标签物化输出独立校验暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_label_materialization_output_validation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelMaterializationOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeLabelMaterializationOutputValidationReadinessSummary {
            validation_eligible_count: registry.validation_eligible_count,
            validation_count: registry.validation_count,
            validated_envelope_count: registry.validated_envelope_count,
            failed_validation_count: registry.failed_validation_count,
            validation_status: registry.validation_status,
            outcome_label_generation_enabled: false,
        },
    )
}

pub(crate) async fn independently_validated_materialized_envelopes_for_label_write_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope>, String> {
    let attempts = completed_execution_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let by_attempt = validations
        .into_iter()
        .filter(|record| record.untrusted_envelope_validated)
        .map(|record| (record.materialization_attempt_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for attempt in attempts {
        let Some(validation) = by_attempt.get(&attempt.claim.attempt_id).cloned() else {
            continue;
        };
        if validation.claim_sha256 != attempt.claim.claim_sha256
            || validation.result_id != attempt.result.result_id
            || validation.result_sha256 != attempt.result.result_sha256
            || attempt.result.output_sha256.as_deref() != Some(validation.output_sha256.as_str())
        {
            continue;
        }
        let Some(admission) = exact_admitted_historical_outcome_for_label_materialization(
            state,
            &attempt.claim.source_attempt_id,
            &attempt.claim.admission_review_id,
            &attempt.claim.admission_review_sha256,
            &attempt.claim.validation_sha256,
            &attempt.claim.source_output_sha256,
            &attempt.claim.snapshot_sha256,
            &attempt.claim.protocol_sha256,
        )
        .await?
        else {
            continue;
        };
        if !provenance_matches(
            &attempt,
            &admission,
            attempt
                .result
                .untrusted_envelope
                .as_ref()
                .ok_or_else(|| "标签物化结果缺少未信任原始结果包".to_string())?,
        ) || !metrics_bitwise_equal(
            &validation.independently_validated_metrics,
            &admission.validation.recomputed_metrics,
        ) {
            continue;
        }
        output.push(
            IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope {
                attempt,
                validation,
            },
        );
    }
    output.sort_by(|left, right| {
        right
            .validation
            .validated_at
            .cmp(&left.validation.validated_at)
            .then_with(|| {
                left.attempt
                    .claim
                    .attempt_id
                    .cmp(&right.attempt.claim.attempt_id)
            })
    });
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelMaterializationOutputValidationRegistry, String> {
    let attempts = completed_execution_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.materialization_attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let items = attempts
        .into_iter()
        .map(|attempt| {
            let validation = index
                .get(attempt.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeLabelMaterializationOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let validation_count = validations.len();
    let validated_envelope_count = validations
        .iter()
        .filter(|record| record.untrusted_envelope_validated)
        .count();
    let failed_validation_count = validations
        .iter()
        .filter(|record| !record.untrusted_envelope_validated)
        .count();
    let validation_status = if validated_envelope_count > 0 {
        "materialized_envelope_independently_validated_still_not_label"
    } else if failed_validation_count > 0 {
        "materialized_envelope_validation_failed_fail_closed"
    } else if validation_eligible_count > 0 {
        "waiting_independent_structure_provenance_bitwise_validation"
    } else {
        "waiting_untrusted_materialized_envelope"
    };
    Ok(HistoricalOutcomeLabelMaterializationOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: VALIDATION_POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count,
        validated_envelope_count,
        failed_validation_count,
        validation_status: validation_status.to_string(),
        output_validation_available: true,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "独立管理员只可校验第十八阶段精确 claim/result/output 的结构、来源、已知局限与 20/60/250 日浮点位模式；通过后仍不是结果标签，不开放训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    admin_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeLabelMaterializationOutputRequest,
) -> Result<HistoricalOutcomeLabelMaterializationOutputValidationRecord, String> {
    if !valid_id(attempt_id) {
        return Err("标签物化执行尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-label-materialization-output-validation-{attempt_id}"),
    )
    .await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.materialization_attempt_id == attempt_id)
    {
        return Err("这条标签物化输出已经形成不可变独立校验记录，不得重放".to_string());
    }
    let attempt = completed_execution_attempt_for_independent_validation(
        state,
        attempt_id,
        &request.expected_claim_sha256,
        &request.expected_result_sha256,
        &request.expected_output_sha256,
    )
    .await?
    .ok_or_else(|| "物化 claim、result 或 output 已变化，或没有完整未信任输出".to_string())?;
    if attempt.claim.admission_review_sha256 != request.expected_admission_review_sha256
        || attempt.claim.validation_sha256 != request.expected_validation_sha256
        || attempt.claim.source_output_sha256 != request.expected_source_output_sha256
        || attempt.claim.snapshot_sha256 != request.expected_snapshot_sha256
        || attempt.claim.protocol_sha256 != request.expected_protocol_sha256
        || attempt.claim.recomputed_metrics_sha256 != request.expected_recomputed_metrics_sha256
    {
        return Err("物化输出的准入、校验、来源、快照、协议或指标绑定已经变化".to_string());
    }
    let authorization = exact_first_execution_authorization_for_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "标签物化首次执行授权审计链或当前上游绑定不可用".to_string())?;
    let admission = exact_admitted_historical_outcome_for_label_materialization(
        state,
        &attempt.claim.source_attempt_id,
        &attempt.claim.admission_review_id,
        &attempt.claim.admission_review_sha256,
        &attempt.claim.validation_sha256,
        &attempt.claim.source_output_sha256,
        &attempt.claim.snapshot_sha256,
        &attempt.claim.protocol_sha256,
    )
    .await?
    .ok_or_else(|| "当前准入输出或其独立校验链已失效".to_string())?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization, &admission);
    let independent = validator_is_independent(admin_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("独立校验人不能是物化调用人或任一物化、准入、校验、原执行链角色".to_string());
    }
    let envelope = attempt
        .result
        .untrusted_envelope
        .as_ref()
        .ok_or_else(|| "标签物化结果缺少未信任原始结果包".to_string())?;
    let canonical_output_hash_verified =
        hash_serializable(envelope)? == request.expected_output_sha256;
    let mut mismatch_reasons = structural_mismatch_reasons(&attempt, &admission, envelope);
    if !canonical_output_hash_verified {
        mismatch_reasons.push("canonical_output_sha256_mismatch".to_string());
    }
    let provenance_match = provenance_matches(&attempt, &admission, envelope);
    if !provenance_match {
        mismatch_reasons.push("provenance_mismatch".to_string());
    }
    let exact_metric_bits_match = metrics_bitwise_equal(
        envelope.raw_validated_metrics.as_slice(),
        admission.validation.recomputed_metrics.as_slice(),
    );
    if !exact_metric_bits_match {
        mismatch_reasons.push("exact_metric_bits_mismatch".to_string());
    }
    let known_limitations_match =
        envelope.known_limitations == admission.admission_review.known_limitations;
    if !known_limitations_match {
        mismatch_reasons.push("known_limitations_mismatch".to_string());
    }
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    let output_structure_verified = mismatch_reasons.iter().all(|reason| {
        matches!(
            reason.as_str(),
            "canonical_output_sha256_mismatch"
                | "provenance_mismatch"
                | "exact_metric_bits_mismatch"
                | "known_limitations_mismatch"
        )
    });
    let untrusted_envelope_validated = output_structure_verified
        && canonical_output_hash_verified
        && provenance_match
        && exact_metric_bits_match
        && known_limitations_match;
    let mut record = HistoricalOutcomeLabelMaterializationOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: VALIDATION_POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        materialization_attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        output_sha256: request.expected_output_sha256,
        isolated_runner_id: attempt.claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: attempt.claim.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: attempt.claim.runner_artifact_sha256.clone(),
        authorization_review_id: attempt.claim.authorization_review_id.clone(),
        authorization_review_sha256: attempt.claim.authorization_review_sha256.clone(),
        materialization_implementation_id: attempt.claim.materialization_implementation_id.clone(),
        materialization_implementation_spec_sha256: attempt
            .claim
            .materialization_implementation_spec_sha256
            .clone(),
        admission_review_id: attempt.claim.admission_review_id.clone(),
        admission_review_sha256: attempt.claim.admission_review_sha256.clone(),
        source_validation_id: attempt.claim.validation_id.clone(),
        source_validation_sha256: attempt.claim.validation_sha256.clone(),
        source_attempt_id: attempt.claim.source_attempt_id.clone(),
        source_output_sha256: attempt.claim.source_output_sha256.clone(),
        snapshot_id: attempt.claim.snapshot_id.clone(),
        snapshot_sha256: attempt.claim.snapshot_sha256.clone(),
        reconstruction_id: attempt.claim.reconstruction_id.clone(),
        reconstruction_sha256: attempt.claim.reconstruction_sha256.clone(),
        protocol_version: attempt.claim.protocol_version.clone(),
        protocol_sha256: attempt.claim.protocol_sha256.clone(),
        recomputed_metrics_sha256: attempt.claim.recomputed_metrics_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: admin_id.to_string(),
        materialization_invoked_by: attempt.claim.invoked_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_materialization_and_prior_chain: independent,
        immutable_chain_integrity_verified: true,
        current_admitted_source_binding_verified: true,
        canonical_output_hash_verified,
        output_structure_verified,
        provenance_match,
        exact_metric_bits_match,
        known_limitations_match,
        independently_validated_metrics: admission.validation.recomputed_metrics.clone(),
        mismatch_reasons,
        verdict: if untrusted_envelope_validated {
            HistoricalOutcomeLabelMaterializationOutputValidationVerdict::ValidatedStructureProvenanceAndBitwiseMatch
        } else {
            HistoricalOutcomeLabelMaterializationOutputValidationVerdict::FailedStructureProvenanceOrBitwiseMismatch
        },
        untrusted_envelope_validated,
        outcome_label_admission_authorized: false,
        outcome_label_write_allowed: false,
        outcome_label_written: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.validation_sha256 = validation_fingerprint(&record)?;
    record.validation_id = record.validation_sha256[..32].to_string();
    validate_record(&record)?;
    write_immutable_json(
        &validation_root(state)
            .join(&record.materialization_attempt_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn prior_actor_ids(
    attempt: &CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt,
    authorization: &super::historical_outcome_label_materialization_first_execution_authorizations::ApprovedHistoricalOutcomeLabelMaterializationFirstExecutionAuthorization,
    admission: &AdmittedHistoricalOutcomeForLabelMaterialization,
) -> Vec<String> {
    let runner = &authorization.runner;
    let mut actors = vec![
        attempt.claim.invoked_by.clone(),
        runner.registered_by.clone(),
        authorization.review.reviewer_id.clone(),
        runner.materialization_run_authorization_reviewer_id.clone(),
        runner.materialization_implementation_registered_by.clone(),
        runner.admission_reviewer_id.clone(),
        runner.validated_by.clone(),
        runner.execution_invoked_by.clone(),
        runner.source_runner_registered_by.clone(),
        runner
            .source_first_execution_authorization_reviewer_id
            .clone(),
        runner.source_run_authorization_reviewer_id.clone(),
        admission.admission_review.reviewer_id.clone(),
        admission.validation.validated_by.clone(),
    ];
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn validator_is_independent(validator: &str, prior_actors: &[String]) -> bool {
    !validator.trim().is_empty() && !prior_actors.iter().any(|actor| actor == validator)
}

fn structural_mismatch_reasons(
    attempt: &CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt,
    admission: &AdmittedHistoricalOutcomeForLabelMaterialization,
    envelope: &HistoricalOutcomeLabelMaterializationUntrustedEnvelope,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if envelope.schema_version
        != "hone-historical-outcome-label-materialization-untrusted-envelope-v1"
        || envelope.output_label_schema_version.trim().is_empty()
    {
        reasons.push("envelope_schema_invalid".to_string());
    }
    let horizons = envelope
        .raw_validated_metrics
        .iter()
        .map(|metric| metric.horizon_market_sessions)
        .collect::<Vec<_>>();
    if horizons != REQUIRED_HORIZONS
        || envelope.raw_validated_metrics.iter().any(metric_invalid)
        || envelope.metric_end_dates.len() != REQUIRED_HORIZONS.len()
    {
        reasons.push("metric_structure_invalid".to_string());
    }
    if !envelope.deterministic_projection_only
        || !envelope.exact_metric_bits_preserved
        || !envelope.provenance_preserved
        || !envelope.known_limitations_preserved
        || !envelope.output_is_untrusted
        || envelope.independent_validation_completed
        || envelope.outcome_label_write_allowed
        || envelope.outcome_label_written
        || envelope.direction_inferred
        || envelope.rating_inferred
        || envelope.investment_action_inferred
        || envelope.position_size_inferred
        || envelope.training_target_written
        || envelope.reward_written
        || envelope.shadow_position_written
        || envelope.order_generated
        || envelope.broker_accessed
        || envelope.trade_executed
        || attempt.result.independent_validation_completed
        || attempt.result.outcome_label_admission_authorized
        || attempt.result.outcome_label_write_allowed
        || attempt.result.outcome_label_written
        || attempt.result.training_authorized
        || attempt.result.reward_authorized
        || attempt.result.shadow_portfolio_authorized
        || attempt.result.order_generation_authorized
        || attempt.result.broker_access_authorized
        || attempt.result.trading_authorized
    {
        reasons.push("envelope_or_result_authority_invalid".to_string());
    }
    if hash_serializable(&admission.validation.recomputed_metrics)
        .ok()
        .as_deref()
        != Some(attempt.claim.recomputed_metrics_sha256.as_str())
    {
        reasons.push("source_metric_hash_invalid".to_string());
    }
    reasons
}

fn provenance_matches(
    attempt: &CompletedHistoricalOutcomeLabelMaterializationExecutionAttempt,
    admission: &AdmittedHistoricalOutcomeForLabelMaterialization,
    envelope: &HistoricalOutcomeLabelMaterializationUntrustedEnvelope,
) -> bool {
    envelope.materialization_implementation_id == attempt.claim.materialization_implementation_id
        && envelope.materialization_implementation_spec_sha256
            == attempt.claim.materialization_implementation_spec_sha256
        && envelope.admission_review_id == admission.admission_review.review_id
        && envelope.admission_review_sha256 == admission.admission_review.review_sha256
        && envelope.validation_id == admission.validation.validation_id
        && envelope.validation_sha256 == admission.validation.validation_sha256
        && envelope.source_attempt_id == admission.validation.attempt_id
        && envelope.source_claim_sha256 == admission.validation.claim_sha256
        && envelope.source_result_id == admission.validation.result_id
        && envelope.source_result_sha256 == admission.validation.result_sha256
        && envelope.source_output_sha256 == admission.validation.output_sha256
        && envelope.snapshot_id == admission.snapshot.snapshot_id
        && envelope.snapshot_sha256 == admission.snapshot.snapshot_sha256
        && envelope.reconstruction_id == admission.snapshot.reconstruction_id
        && envelope.reconstruction_sha256 == admission.snapshot.reconstruction_sha256
        && envelope.protocol_version == admission.snapshot.protocol_version
        && envelope.protocol_sha256 == admission.snapshot.protocol_sha256
        && envelope.asset_symbol == admission.snapshot.asset_symbol
        && envelope.benchmark_symbol == admission.snapshot.benchmark_symbol
        && envelope.decision_available_at == admission.snapshot.decision_available_at
        && envelope.common_session_count == admission.snapshot.common_session_count
        && envelope.recomputed_metrics_sha256 == attempt.claim.recomputed_metrics_sha256
}

fn metric_invalid(metric: &HistoricalOutcomeDryRunMetric) -> bool {
    metric.end_date < metric.start_date
        || !metric.asset_return.is_finite()
        || !metric.benchmark_return.is_finite()
        || !metric.excess_return.is_finite()
        || !metric.asset_max_drawdown.is_finite()
        || metric.asset_max_drawdown > 0.0
}

fn metrics_bitwise_equal(
    observed: &[HistoricalOutcomeDryRunMetric],
    expected: &[HistoricalOutcomeDryRunMetric],
) -> bool {
    observed.len() == expected.len()
        && observed.iter().zip(expected).all(|(left, right)| {
            left.horizon_market_sessions == right.horizon_market_sessions
                && left.start_date == right.start_date
                && left.end_date == right.end_date
                && left.asset_return.to_bits() == right.asset_return.to_bits()
                && left.benchmark_return.to_bits() == right.benchmark_return.to_bits()
                && left.excess_return.to_bits() == right.excess_return.to_bits()
                && left.asset_max_drawdown.to_bits() == right.asset_max_drawdown.to_bits()
        })
}

fn validate_record(
    record: &HistoricalOutcomeLabelMaterializationOutputValidationRecord,
) -> Result<(), String> {
    let downstream_closed = !record.outcome_label_admission_authorized
        && !record.outcome_label_write_allowed
        && !record.outcome_label_written
        && !record.training_authorized
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let expected_valid = record.validator_independent_from_materialization_and_prior_chain
        && record.immutable_chain_integrity_verified
        && record.current_admitted_source_binding_verified
        && record.canonical_output_hash_verified
        && record.output_structure_verified
        && record.provenance_match
        && record.exact_metric_bits_match
        && record.known_limitations_match
        && record.mismatch_reasons.is_empty();
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != VALIDATION_POLICY_VERSION
        || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256)
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != validation_fingerprint(record)?
        || !valid_id(&record.materialization_attempt_id)
        || !valid_sha256(&record.claim_sha256)
        || !valid_id(&record.result_id)
        || !valid_sha256(&record.result_sha256)
        || !valid_sha256(&record.output_sha256)
        || !valid_id(&record.isolated_runner_id)
        || !valid_sha256(&record.isolated_runner_spec_sha256)
        || !valid_sha256(&record.runner_artifact_sha256)
        || !valid_id(&record.authorization_review_id)
        || !valid_sha256(&record.authorization_review_sha256)
        || !valid_id(&record.materialization_implementation_id)
        || !valid_sha256(&record.materialization_implementation_spec_sha256)
        || !valid_id(&record.admission_review_id)
        || !valid_sha256(&record.admission_review_sha256)
        || !valid_id(&record.source_validation_id)
        || !valid_sha256(&record.source_validation_sha256)
        || !valid_id(&record.source_attempt_id)
        || !valid_sha256(&record.source_output_sha256)
        || !valid_id(&record.snapshot_id)
        || !valid_sha256(&record.snapshot_sha256)
        || !valid_id(&record.reconstruction_id)
        || !valid_sha256(&record.reconstruction_sha256)
        || record.protocol_version.trim().is_empty()
        || !valid_sha256(&record.protocol_sha256)
        || !valid_sha256(&record.recomputed_metrics_sha256)
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || record.validated_by.trim().is_empty()
        || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.validated_by)
        || record.untrusted_envelope_validated != expected_valid
        || (record.untrusted_envelope_validated
            != (record.verdict
                == HistoricalOutcomeLabelMaterializationOutputValidationVerdict::ValidatedStructureProvenanceAndBitwiseMatch))
        || !downstream_closed
    {
        return Err("historical materialized envelope validation record is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn validation_fingerprint(
    record: &HistoricalOutcomeLabelMaterializationOutputValidationRecord,
) -> Result<String, String> {
    let mut value = serde_json::to_value(record).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "materialized envelope validation fingerprint is invalid".to_string())?;
    object.remove("validation_id");
    object.remove("validation_sha256");
    hash_serializable(&value)
}

fn validator_implementation_sha256() -> String {
    format!(
        "{:x}",
        Sha256::digest(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
    )
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn validation_attempt_ids_unique<'a>(values: impl IntoIterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    values.into_iter().all(|value| seen.insert(value))
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeLabelMaterializationOutputValidationRecord>, String> {
    let root = validation_root(state);
    let mut directories = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    while let Some(directory) = directories.next_entry().await.map_err(|e| e.to_string())? {
        if !directory
            .file_type()
            .await
            .map_err(|e| e.to_string())?
            .is_dir()
        {
            continue;
        }
        let mut entries = tokio::fs::read_dir(directory.path())
            .await
            .map_err(|e| e.to_string())?;
        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let bytes = tokio::fs::read(entry.path())
                .await
                .map_err(|e| e.to_string())?;
            let record: HistoricalOutcomeLabelMaterializationOutputValidationRecord =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            validate_record(&record)?;
            if entry.path().file_stem().and_then(|value| value.to_str())
                != Some(record.validation_id.as_str())
                || directory.file_name().to_string_lossy() != record.materialization_attempt_id
            {
                return Err("historical materialized envelope validation path mismatch".to_string());
            }
            if !ids.insert(record.validation_id.clone())
                || !hashes.insert(record.validation_sha256.clone())
            {
                return Err(
                    "historical materialized envelope validation contains duplicate records"
                        .to_string(),
                );
            }
            output.push(record);
        }
    }
    if !validation_attempt_ids_unique(
        output
            .iter()
            .map(|record| record.materialization_attempt_id.as_str()),
    ) {
        return Err(
            "historical materialized envelope validation contains replayed attempts".to_string(),
        );
    }
    output.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(output)
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(path).await.map_err(|e| e.to_string())?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;
    file.sync_all().await.map_err(|e| e.to_string())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-label-materialization-output-validations")
}

struct ValidationLock {
    path: PathBuf,
}

impl Drop for ValidationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, name: &str) -> Result<ValidationLock, String> {
    if !valid_id(name) {
        return Err("invalid materialized envelope validation lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|e| e.to_string())?;
    let path = directory.join(format!("{name}.lock"));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age > StdDuration::from_secs(300));
        if stale {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    options
        .open(&path)
        .await
        .map_err(|_| "已有标签物化输出独立校验正在进行".to_string())?;
    Ok(ValidationLock { path })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn metrics() -> Vec<HistoricalOutcomeDryRunMetric> {
        [20_u16, 60, 250]
            .into_iter()
            .map(|horizon| HistoricalOutcomeDryRunMetric {
                horizon_market_sessions: horizon,
                start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
                    + chrono::Days::new(u64::from(horizon)),
                asset_return: f64::from(horizon) / 100.0,
                benchmark_return: f64::from(horizon) / 200.0,
                excess_return: f64::from(horizon) / 200.0,
                asset_max_drawdown: -0.1,
            })
            .collect()
    }

    #[test]
    fn bitwise_metric_validation_rejects_one_ulp_tamper() {
        let expected = metrics();
        let mut observed = expected.clone();
        observed[0].asset_return = f64::from_bits(observed[0].asset_return.to_bits() + 1);
        assert!(!metrics_bitwise_equal(&observed, &expected));
        assert!(metrics_bitwise_equal(&expected, &expected));
    }

    #[test]
    fn metric_structure_is_frozen_and_finite() {
        let mut value = metrics().remove(0);
        assert!(!metric_invalid(&value));
        value.asset_max_drawdown = 0.01;
        assert!(metric_invalid(&value));
        value.asset_max_drawdown = f64::NAN;
        assert!(metric_invalid(&value));
    }

    #[test]
    fn validator_must_be_independent_from_every_prior_actor() {
        let actors = vec!["invoker".to_string(), "reviewer".to_string()];
        assert!(validator_is_independent("validator", &actors));
        assert!(!validator_is_independent("invoker", &actors));
        assert!(!validator_is_independent("reviewer", &actors));
        assert!(!validator_is_independent("", &actors));
    }

    #[test]
    fn prior_actor_list_must_be_sorted_and_unique() {
        assert!(sorted_unique(&["a".into(), "b".into(), "c".into()]));
        assert!(!sorted_unique(&["b".into(), "a".into()]));
        assert!(!sorted_unique(&["a".into(), "a".into()]));
    }

    #[test]
    fn replayed_materialization_attempt_validation_is_rejected() {
        assert!(validation_attempt_ids_unique(["attempt-a", "attempt-b"]));
        assert!(!validation_attempt_ids_unique(["attempt-a", "attempt-a"]));
    }

    #[test]
    fn validator_implementation_digest_is_fixed() {
        assert!(valid_sha256(&validator_implementation_sha256()));
        assert_eq!(
            validator_implementation_sha256(),
            format!(
                "{:x}",
                Sha256::digest(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
            )
        );
    }
}
