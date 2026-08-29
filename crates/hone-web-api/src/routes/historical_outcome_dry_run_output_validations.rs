//! Independent structural and deterministic recomputation validation for one
//! completed historical outcome dry-run output.
//!
//! This registry never mutates the execution result. A different administrator
//! must validate the exact immutable claim/result/output against the current
//! sealed snapshot. The recomputation implementation intentionally does not call
//! the execution implementation. A passing record still cannot create labels,
//! training targets, rewards, shadow positions, orders or trades.

use std::collections::{BTreeMap, BTreeSet, HashMap};
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

use super::historical_outcome_dry_run_execution_attempts::{
    CompletedHistoricalOutcomeDryRunExecutionAttempt, HistoricalOutcomeDryRunMetric,
    HistoricalOutcomeDryRunUntrustedOutput, completed_execution_attempt_for_independent_validation,
    completed_execution_attempts_for_independent_validation,
};
use super::historical_outcome_dry_run_first_execution_authorizations::exact_first_execution_authorization_for_audit;
use super::historical_outcome_price_snapshots::{
    HistoricalOutcomePriceSnapshot, current_sealed_snapshot_for_execution,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-dry-run-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-output-validation-v1";
const VALIDATION_POLICY_VERSION: &str =
    "hone-historical-outcome-dry-run-independent-structural-recomputation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-independent-outcome-recomputer-v1-no-execution-code-reuse";
const MAX_INPUT_SERIES_POINTS: usize = 2_048;
const MAX_COMMON_SESSION_INPUTS: usize = 1_024;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeDryRunOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeDryRunOutputValidationVerdict {
    ValidatedDeterministicMatch,
    FailedStructuralOrRecomputationMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeDryRunOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub execution_invoked_by: String,
    pub runner_registered_by: String,
    pub first_execution_authorization_reviewer_id: String,
    pub run_authorization_reviewer_id: String,
    pub validator_independent_from_execution_and_prior_reviewers: bool,
    pub immutable_chain_integrity_verified: bool,
    pub current_sealed_snapshot_binding_verified: bool,
    pub canonical_output_hash_verified: bool,
    pub output_structure_verified: bool,
    pub deterministic_recomputation_match: bool,
    pub recomputed_metrics: Vec<HistoricalOutcomeDryRunMetric>,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeDryRunOutputValidationVerdict,
    pub output_validated: bool,
    pub outcome_label_admission_authorized: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunOutputValidationItem {
    pub attempt: CompletedHistoricalOutcomeDryRunExecutionAttempt,
    pub validation: Option<HistoricalOutcomeDryRunOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeDryRunOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub validated_output_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
    pub output_validation_available: bool,
    pub outcome_label_admission_authorized: bool,
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
pub(crate) struct HistoricalOutcomeDryRunOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub validated_output_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
    pub outcome_label_generation_enabled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeForLabelAdmission {
    pub validation: HistoricalOutcomeDryRunOutputValidationRecord,
    pub snapshot: HistoricalOutcomePriceSnapshot,
}

pub(crate) async fn exact_independently_validated_output_for_label_admission(
    state: &AppState,
    attempt_id: &str,
    validation_id: &str,
    validation_sha256: &str,
    output_sha256: &str,
    snapshot_sha256: &str,
    protocol_sha256: &str,
) -> Result<Option<IndependentlyValidatedHistoricalOutcomeForLabelAdmission>, String> {
    let validation = read_validations(state).await?.into_iter().find(|record| {
        validation_matches_label_admission_request(
            record,
            attempt_id,
            validation_id,
            validation_sha256,
            output_sha256,
            snapshot_sha256,
            protocol_sha256,
        )
    });
    let Some(validation) = validation else {
        return Ok(None);
    };
    validated_output_for_label_admission_from_record(state, validation)
        .await
        .map(Some)
}

pub(crate) fn validation_matches_label_admission_request(
    record: &HistoricalOutcomeDryRunOutputValidationRecord,
    attempt_id: &str,
    validation_id: &str,
    validation_sha256: &str,
    output_sha256: &str,
    snapshot_sha256: &str,
    protocol_sha256: &str,
) -> bool {
    record.attempt_id == attempt_id
        && record.validation_id == validation_id
        && record.validation_sha256 == validation_sha256
        && record.output_sha256 == output_sha256
        && record.snapshot_sha256 == snapshot_sha256
        && record.protocol_sha256 == protocol_sha256
        && record.output_validated
        && record.verdict
            == HistoricalOutcomeDryRunOutputValidationVerdict::ValidatedDeterministicMatch
}

pub(crate) async fn independently_validated_outputs_for_label_admission(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeForLabelAdmission>, String> {
    let mut output = Vec::new();
    for validation in read_validations(state)
        .await?
        .into_iter()
        .filter(|record| record.output_validated)
    {
        output.push(validated_output_for_label_admission_from_record(state, validation).await?);
    }
    Ok(output)
}

async fn validated_output_for_label_admission_from_record(
    state: &AppState,
    validation: HistoricalOutcomeDryRunOutputValidationRecord,
) -> Result<IndependentlyValidatedHistoricalOutcomeForLabelAdmission, String> {
    let attempt = completed_execution_attempt_for_independent_validation(
        state,
        &validation.attempt_id,
        &validation.claim_sha256,
        &validation.result_sha256,
        &validation.output_sha256,
    )
    .await?
    .ok_or_else(|| "已校验输出的 claim、result 或 output 当前无法精确重放".to_string())?;
    let snapshot = current_sealed_snapshot_for_execution(
        state,
        &validation.snapshot_id,
        &validation.snapshot_sha256,
    )
    .await?
    .ok_or_else(|| "已校验输出绑定的封存快照或上游链已失效".to_string())?;
    let output = attempt
        .result
        .untrusted_output
        .as_ref()
        .ok_or_else(|| "已校验执行结果缺少输出".to_string())?;
    let recomputed = independently_recompute_metrics(&snapshot)?;
    if hash_serializable(output)? != validation.output_sha256
        || !structural_mismatch_reasons(&attempt, &snapshot, output).is_empty()
        || !metrics_bitwise_equal(&output.metrics, &validation.recomputed_metrics)
        || !metrics_bitwise_equal(&recomputed, &validation.recomputed_metrics)
    {
        return Err("已校验输出、独立重算记录或当前封存输入不再一致".to_string());
    }
    Ok(IndependentlyValidatedHistoricalOutcomeForLabelAdmission {
        validation,
        snapshot,
    })
}

pub(crate) async fn handle_get_historical_outcome_dry_run_output_validations(
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
            warn!(%error, "historical outcome dry-run output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果独立输出校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_dry_run_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeDryRunOutputRequest>,
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
                warn!(%error, "historical outcome output validation registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果独立输出校验暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_dry_run_output_validation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeDryRunOutputValidationReadinessSummary {
        validation_eligible_count: registry.validation_eligible_count,
        validation_count: registry.validation_count,
        validated_output_count: registry.validated_output_count,
        failed_validation_count: registry.failed_validation_count,
        validation_status: registry.validation_status,
        outcome_label_generation_enabled: false,
    })
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunOutputValidationRegistry, String> {
    let attempts = completed_execution_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let validation_index = validations
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let items = attempts
        .into_iter()
        .map(|attempt| {
            let validation = validation_index
                .get(attempt.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeDryRunOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_count = validations.len();
    let validated_output_count = validations
        .iter()
        .filter(|value| value.output_validated)
        .count();
    let failed_validation_count = validations
        .iter()
        .filter(|value| !value.output_validated)
        .count();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let validation_status = if validated_output_count > 0 {
        "independently_validated_output_waiting_label_admission"
    } else if failed_validation_count > 0 {
        "independent_validation_failed_fail_closed"
    } else if validation_eligible_count > 0 {
        "waiting_independent_structural_recomputation_validation"
    } else {
        "waiting_completed_untrusted_output"
    };
    Ok(HistoricalOutcomeDryRunOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: VALIDATION_POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count,
        validated_output_count,
        failed_validation_count,
        validation_status: validation_status.to_string(),
        output_validation_available: true,
        outcome_label_admission_authorized: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "独立管理员只可对精确 claim/result/output 和当前封存快照执行结构校验与第二套确定性重算；通过后仍需未来独立的结果标签准入复核，当前不生成标签、训练、奖励、影子、订单或交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    admin_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeDryRunOutputRequest,
) -> Result<HistoricalOutcomeDryRunOutputValidationRecord, String> {
    if !valid_id(attempt_id) {
        return Err("执行尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-outcome-output-validation-{attempt_id}"),
    )
    .await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|value| value.attempt_id == attempt_id)
    {
        return Err("这条执行输出已经形成不可变独立校验记录，不得重放".to_string());
    }
    let attempt = completed_execution_attempt_for_independent_validation(
        state,
        attempt_id,
        &request.expected_claim_sha256,
        &request.expected_result_sha256,
        &request.expected_output_sha256,
    )
    .await?
    .ok_or_else(|| "执行 claim、result 或 output 已变化，或该尝试没有完整输出".to_string())?;
    if attempt.claim.snapshot_sha256 != request.expected_snapshot_sha256
        || attempt.claim.protocol_sha256 != request.expected_protocol_sha256
    {
        return Err("封存快照或协议绑定已经变化，请刷新后重试".to_string());
    }
    let authorization = exact_first_execution_authorization_for_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "首次执行授权审计链或当前上游绑定不可用".to_string())?;
    let independent = validator_is_independent(
        admin_id,
        &attempt.claim.invoked_by,
        &authorization.runner.registered_by,
        &authorization.review.reviewer_id,
        &authorization.runner.run_authorization_reviewer_id,
    );
    if !independent {
        return Err("独立校验人不能是执行调用人、运行器登记者或任一上游授权复核人".to_string());
    }
    let snapshot = current_sealed_snapshot_for_execution(
        state,
        &attempt.claim.snapshot_id,
        &attempt.claim.snapshot_sha256,
    )
    .await?
    .ok_or_else(|| "当前封存快照或上游绑定已经失效，独立校验失败关闭".to_string())?;
    let output = attempt
        .result
        .untrusted_output
        .as_ref()
        .ok_or_else(|| "执行结果没有未验证输出".to_string())?;
    let canonical_output_hash_verified =
        hash_serializable(output)? == request.expected_output_sha256;
    let mut mismatch_reasons = structural_mismatch_reasons(&attempt, &snapshot, output);
    if !canonical_output_hash_verified {
        mismatch_reasons.push("canonical_output_sha256_mismatch".to_string());
    }
    let recomputed_metrics = independently_recompute_metrics(&snapshot)?;
    let deterministic_recomputation_match =
        metrics_bitwise_equal(output.metrics.as_slice(), recomputed_metrics.as_slice());
    if !deterministic_recomputation_match {
        mismatch_reasons.push("deterministic_recomputation_mismatch".to_string());
    }
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    let output_structure_verified = mismatch_reasons.iter().all(|reason| {
        reason == "canonical_output_sha256_mismatch"
            || reason == "deterministic_recomputation_mismatch"
    });
    let output_validated = output_structure_verified
        && canonical_output_hash_verified
        && deterministic_recomputation_match;
    let mut record = HistoricalOutcomeDryRunOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: VALIDATION_POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        output_sha256: request.expected_output_sha256,
        isolated_runner_id: attempt.claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: attempt.claim.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: attempt.claim.runner_artifact_sha256.clone(),
        authorization_review_id: attempt.claim.authorization_review_id.clone(),
        authorization_review_sha256: attempt.claim.authorization_review_sha256.clone(),
        snapshot_id: snapshot.snapshot_id.clone(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        protocol_version: snapshot.protocol_version.clone(),
        protocol_sha256: snapshot.protocol_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: admin_id.to_string(),
        execution_invoked_by: attempt.claim.invoked_by.clone(),
        runner_registered_by: authorization.runner.registered_by.clone(),
        first_execution_authorization_reviewer_id: authorization.review.reviewer_id.clone(),
        run_authorization_reviewer_id: authorization.runner.run_authorization_reviewer_id.clone(),
        validator_independent_from_execution_and_prior_reviewers: independent,
        immutable_chain_integrity_verified: true,
        current_sealed_snapshot_binding_verified: true,
        canonical_output_hash_verified,
        output_structure_verified,
        deterministic_recomputation_match,
        recomputed_metrics,
        mismatch_reasons,
        verdict: if output_validated {
            HistoricalOutcomeDryRunOutputValidationVerdict::ValidatedDeterministicMatch
        } else {
            HistoricalOutcomeDryRunOutputValidationVerdict::FailedStructuralOrRecomputationMismatch
        },
        output_validated,
        outcome_label_admission_authorized: false,
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
            .join(&record.attempt_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn structural_mismatch_reasons(
    attempt: &CompletedHistoricalOutcomeDryRunExecutionAttempt,
    snapshot: &HistoricalOutcomePriceSnapshot,
    output: &HistoricalOutcomeDryRunUntrustedOutput,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if output.schema_version != "hone-historical-outcome-dry-run-untrusted-output-v1" {
        reasons.push("output_schema_version_mismatch".to_string());
    }
    if output.snapshot_id != snapshot.snapshot_id
        || output.snapshot_sha256 != snapshot.snapshot_sha256
        || output.snapshot_id != attempt.claim.snapshot_id
    {
        reasons.push("snapshot_binding_mismatch".to_string());
    }
    if output.protocol_version != snapshot.protocol_version
        || output.protocol_sha256 != snapshot.protocol_sha256
        || output.protocol_sha256 != attempt.claim.protocol_sha256
    {
        reasons.push("protocol_binding_mismatch".to_string());
    }
    if output.asset_symbol != snapshot.asset_symbol
        || output.benchmark_symbol != snapshot.benchmark_symbol
        || output.asset_series_sha256 != snapshot.asset_series_sha256
        || output.benchmark_series_sha256 != snapshot.benchmark_series_sha256
        || output.common_session_count != snapshot.common_session_count
    {
        reasons.push("price_series_binding_mismatch".to_string());
    }
    let horizons = output
        .metrics
        .iter()
        .map(|metric| metric.horizon_market_sessions)
        .collect::<Vec<_>>();
    if horizons != vec![20, 60, 250] || output.metrics.iter().any(metric_invalid) {
        reasons.push("metric_structure_invalid".to_string());
    }
    if !output.deterministic_replay_only
        || !output.output_is_untrusted
        || output.outcome_label_written
        || output.training_target_written
        || output.reward_written
        || output.shadow_position_written
        || output.order_generated
        || output.broker_accessed
        || output.trade_executed
    {
        reasons.push("output_permissions_or_provenance_invalid".to_string());
    }
    reasons
}

fn metric_invalid(metric: &HistoricalOutcomeDryRunMetric) -> bool {
    metric.end_date < metric.start_date
        || !metric.asset_return.is_finite()
        || !metric.benchmark_return.is_finite()
        || !metric.excess_return.is_finite()
        || !metric.asset_max_drawdown.is_finite()
        || metric.asset_max_drawdown > 0.0
}

fn validator_is_independent(
    validator: &str,
    invoker: &str,
    runner_registrant: &str,
    first_execution_reviewer: &str,
    run_authorization_reviewer: &str,
) -> bool {
    !validator.trim().is_empty()
        && validator != invoker
        && validator != runner_registrant
        && validator != first_execution_reviewer
        && validator != run_authorization_reviewer
}

fn validation_attempt_ids_unique<'a>(values: impl IntoIterator<Item = &'a str>) -> bool {
    let mut seen = BTreeSet::new();
    values.into_iter().all(|value| seen.insert(value))
}

fn independently_recompute_metrics(
    snapshot: &HistoricalOutcomePriceSnapshot,
) -> Result<Vec<HistoricalOutcomeDryRunMetric>, String> {
    if snapshot.asset_points.len() > MAX_INPUT_SERIES_POINTS
        || snapshot.benchmark_points.len() > MAX_INPUT_SERIES_POINTS
        || snapshot.common_session_count > MAX_COMMON_SESSION_INPUTS
    {
        return Err("封存快照超过独立重算器静态输入上限".to_string());
    }
    let asset_by_date = snapshot
        .asset_points
        .iter()
        .map(|point| (point.date, point.adjusted_close))
        .collect::<BTreeMap<NaiveDate, f64>>();
    let common = snapshot
        .benchmark_points
        .iter()
        .filter_map(|benchmark| {
            asset_by_date
                .get(&benchmark.date)
                .copied()
                .map(|asset| (benchmark.date, asset, benchmark.adjusted_close))
        })
        .collect::<Vec<_>>();
    if common.len() != snapshot.common_session_count || common.len() <= 250 {
        return Err("独立重算器发现共同交易日数量或覆盖不一致".to_string());
    }
    if snapshot.covered_horizons_market_sessions != vec![20, 60, 250] {
        return Err("独立重算器只接受冻结的 20/60/250 窗口".to_string());
    }
    let (start_date, start_asset, start_benchmark) = common[0];
    if start_asset <= 0.0 || start_benchmark <= 0.0 {
        return Err("独立重算器发现非正起始价格".to_string());
    }
    let mut output = Vec::with_capacity(3);
    for horizon in [20_u16, 60_u16, 250_u16] {
        let index = usize::from(horizon);
        let (end_date, end_asset, end_benchmark) = common[index];
        let asset_return = end_asset / start_asset - 1.0;
        let benchmark_return = end_benchmark / start_benchmark - 1.0;
        let mut running_peak = f64::NEG_INFINITY;
        let asset_max_drawdown = common[..=index]
            .iter()
            .map(|(_, asset, _)| {
                running_peak = running_peak.max(*asset);
                *asset / running_peak - 1.0
            })
            .fold(0.0_f64, f64::min);
        let metric = HistoricalOutcomeDryRunMetric {
            horizon_market_sessions: horizon,
            start_date,
            end_date,
            asset_return,
            benchmark_return,
            excess_return: asset_return - benchmark_return,
            asset_max_drawdown,
        };
        if metric_invalid(&metric) {
            return Err("独立重算器产生无效指标".to_string());
        }
        output.push(metric);
    }
    Ok(output)
}

fn metrics_bitwise_equal(
    observed: &[HistoricalOutcomeDryRunMetric],
    recomputed: &[HistoricalOutcomeDryRunMetric],
) -> bool {
    observed.len() == recomputed.len()
        && observed.iter().zip(recomputed).all(|(left, right)| {
            left.horizon_market_sessions == right.horizon_market_sessions
                && left.start_date == right.start_date
                && left.end_date == right.end_date
                && left.asset_return.to_bits() == right.asset_return.to_bits()
                && left.benchmark_return.to_bits() == right.benchmark_return.to_bits()
                && left.excess_return.to_bits() == right.excess_return.to_bits()
                && left.asset_max_drawdown.to_bits() == right.asset_max_drawdown.to_bits()
        })
}

fn validate_record(record: &HistoricalOutcomeDryRunOutputValidationRecord) -> Result<(), String> {
    let downstream_closed = !record.outcome_label_admission_authorized
        && !record.outcome_label_written
        && !record.training_authorized
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let expected_valid = record.validator_independent_from_execution_and_prior_reviewers
        && record.immutable_chain_integrity_verified
        && record.current_sealed_snapshot_binding_verified
        && record.canonical_output_hash_verified
        && record.output_structure_verified
        && record.deterministic_recomputation_match
        && record.mismatch_reasons.is_empty();
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != VALIDATION_POLICY_VERSION
        || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256)
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != validation_fingerprint(record)?
        || !valid_id(&record.attempt_id)
        || !valid_sha256(&record.claim_sha256)
        || !valid_id(&record.result_id)
        || !valid_sha256(&record.result_sha256)
        || !valid_sha256(&record.output_sha256)
        || !valid_id(&record.isolated_runner_id)
        || !valid_sha256(&record.isolated_runner_spec_sha256)
        || !valid_sha256(&record.runner_artifact_sha256)
        || !valid_id(&record.authorization_review_id)
        || !valid_sha256(&record.authorization_review_sha256)
        || !valid_id(&record.snapshot_id)
        || !valid_sha256(&record.snapshot_sha256)
        || record.protocol_version.trim().is_empty()
        || !valid_sha256(&record.protocol_sha256)
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || record.validated_by.trim().is_empty()
        || record.validated_by == record.execution_invoked_by
        || record.validated_by == record.runner_registered_by
        || record.validated_by == record.first_execution_authorization_reviewer_id
        || record.validated_by == record.run_authorization_reviewer_id
        || record.output_validated != expected_valid
        || (record.output_validated
            != (record.verdict
                == HistoricalOutcomeDryRunOutputValidationVerdict::ValidatedDeterministicMatch))
        || !downstream_closed
    {
        return Err(
            "historical outcome independent validation record is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn validation_fingerprint(
    record: &HistoricalOutcomeDryRunOutputValidationRecord,
) -> Result<String, String> {
    let mut value = serde_json::to_value(record).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "validation fingerprint payload is invalid".to_string())?;
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

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeDryRunOutputValidationRecord>, String> {
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
            let record: HistoricalOutcomeDryRunOutputValidationRecord =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            validate_record(&record)?;
            if entry.path().file_stem().and_then(|value| value.to_str())
                != Some(record.validation_id.as_str())
                || directory.file_name().to_string_lossy() != record.attempt_id
            {
                return Err("historical outcome validation path mismatch".to_string());
            }
            if !ids.insert(record.validation_id.clone())
                || !hashes.insert(record.validation_sha256.clone())
            {
                return Err(
                    "historical outcome validation registry contains duplicates or replay"
                        .to_string(),
                );
            }
            output.push(record);
        }
    }
    if !validation_attempt_ids_unique(output.iter().map(|record| record.attempt_id.as_str())) {
        return Err(
            "historical outcome validation registry contains replayed attempts".to_string(),
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
    decision_root(state).join("historical-outcome-dry-run-output-validations")
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
        return Err("invalid historical outcome validation lock name".to_string());
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
        .map_err(|_| "已有独立输出校验正在进行".to_string())?;
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
    use super::super::historical_outcome_price_snapshots::SealedAdjustedClosePoint;
    use super::*;

    fn snapshot() -> HistoricalOutcomePriceSnapshot {
        let points = (0..=250)
            .map(|day| SealedAdjustedClosePoint {
                date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap() + chrono::Days::new(day),
                adjusted_close: 100.0 + day as f64,
            })
            .collect::<Vec<_>>();
        HistoricalOutcomePriceSnapshot {
            schema_version: "test".into(),
            ingestion_policy_version: "test".into(),
            snapshot_id: "snapshot-1".into(),
            snapshot_sha256: "a".repeat(64),
            sealed_at: Utc::now(),
            sealed_by: "admin".into(),
            reconstruction_id: "reconstruction-1".into(),
            reconstruction_sha256: "b".repeat(64),
            reconstruction_review_id: "review-1".into(),
            anchor_candidate_id: "anchor-1".into(),
            anchor_review_id: "anchor-review-1".into(),
            decision_available_at: Utc::now(),
            implementation_id: "implementation-1".into(),
            implementation_spec_sha256: "c".repeat(64),
            implementation_review_id: "implementation-review-1".into(),
            governance_review_id: "governance-1".into(),
            protocol_version: "v1".into(),
            protocol_sha256: "d".repeat(64),
            code_revision: "test".into(),
            provider: "test".into(),
            provider_endpoint_template: "test".into(),
            price_basis: "adjusted close".into(),
            asset_symbol: "TEST".into(),
            benchmark_symbol: "SPY".into(),
            requested_from: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            requested_to: NaiveDate::from_ymd_opt(2025, 9, 8).unwrap(),
            asset_payload_sha256: "e".repeat(64),
            benchmark_payload_sha256: "f".repeat(64),
            asset_series_sha256: "1".repeat(64),
            benchmark_series_sha256: "2".repeat(64),
            asset_points: points.clone(),
            benchmark_points: points,
            common_session_count: 251,
            covered_horizons_market_sessions: vec![20, 60, 250],
            all_protocol_horizons_covered: true,
            outcome_metrics_computed: false,
            label_written: false,
            historical_state_mutated: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        }
    }

    #[test]
    fn independent_recomputation_has_frozen_horizons() {
        let metrics = independently_recompute_metrics(&snapshot()).unwrap();
        assert_eq!(
            metrics
                .iter()
                .map(|m| m.horizon_market_sessions)
                .collect::<Vec<_>>(),
            vec![20, 60, 250]
        );
        assert_eq!(
            metrics[0].asset_return.to_bits(),
            (120.0_f64 / 100.0 - 1.0).to_bits()
        );
        assert_eq!(metrics[2].asset_max_drawdown.to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn bitwise_comparison_rejects_small_tamper() {
        let expected = independently_recompute_metrics(&snapshot()).unwrap();
        let mut tampered = expected.clone();
        tampered[0].asset_return = f64::from_bits(tampered[0].asset_return.to_bits() + 1);
        assert!(!metrics_bitwise_equal(&tampered, &expected));
    }

    #[test]
    fn recomputation_fails_closed_on_non_frozen_horizons() {
        let mut value = snapshot();
        value.covered_horizons_market_sessions = vec![20, 60];
        assert!(independently_recompute_metrics(&value).is_err());
    }

    #[test]
    fn validator_must_be_independent_from_all_execution_roles() {
        assert!(validator_is_independent(
            "validator",
            "invoker",
            "registrant",
            "first-reviewer",
            "run-reviewer"
        ));
        assert!(!validator_is_independent(
            "invoker",
            "invoker",
            "registrant",
            "first-reviewer",
            "run-reviewer"
        ));
        assert!(!validator_is_independent(
            "registrant",
            "invoker",
            "registrant",
            "first-reviewer",
            "run-reviewer"
        ));
        assert!(!validator_is_independent(
            "first-reviewer",
            "invoker",
            "registrant",
            "first-reviewer",
            "run-reviewer"
        ));
        assert!(!validator_is_independent(
            "run-reviewer",
            "invoker",
            "registrant",
            "first-reviewer",
            "run-reviewer"
        ));
    }

    #[test]
    fn replayed_attempt_validation_is_rejected() {
        assert!(validation_attempt_ids_unique(["attempt-a", "attempt-b"]));
        assert!(!validation_attempt_ids_unique(["attempt-a", "attempt-a"]));
    }

    #[test]
    fn structurally_invalid_metric_fails_closed() {
        let mut metric = independently_recompute_metrics(&snapshot())
            .unwrap()
            .remove(0);
        metric.asset_max_drawdown = 0.01;
        assert!(metric_invalid(&metric));
        metric.asset_max_drawdown = f64::NAN;
        assert!(metric_invalid(&metric));
    }
}
