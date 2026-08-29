//! Independent admission review for one exact, independently recomputed
//! historical outcome output.
//!
//! Approval here admits only the exact validated output as an input to a later
//! label-materialization step. It does not write a label, infer a direction,
//! train, reward, create a shadow position, draft an order or trade.

use std::collections::{BTreeSet, HashMap};
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

use super::historical_outcome_dry_run_output_validations::{
    HistoricalOutcomeDryRunOutputValidationRecord,
    IndependentlyValidatedHistoricalOutcomeForLabelAdmission,
    exact_independently_validated_output_for_label_admission,
    independently_validated_outputs_for_label_admission,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-label-admission-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-historical-outcome-label-admission-review-v1";
const POLICY_VERSION: &str =
    "hone-historical-outcome-label-admission-v1-independent-applicability-bias-review";
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 2_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeLabelAdmissionVerdict {
    ApprovedForFutureLabelMaterialization,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeLabelAdmissionRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_validation_id: String,
    expected_validation_sha256: String,
    expected_output_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    verdict: HistoricalOutcomeLabelAdmissionVerdict,
    rationale: String,
    known_limitations: String,
    exact_validation_current_binding_confirmed: bool,
    frozen_protocol_applicability_confirmed: bool,
    complete_horizons_and_common_session_endpoints_confirmed: bool,
    adjusted_close_and_corporate_action_basis_confirmed: bool,
    benchmark_comparability_confirmed: bool,
    event_time_and_future_isolation_confirmed: bool,
    missingness_and_survivorship_bias_reviewed: bool,
    no_manual_metric_override_confirmed: bool,
    label_semantics_and_direction_not_inferred_confirmed: bool,
    downstream_authority_remains_closed_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelAdmissionReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_by: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub common_session_count: usize,
    pub metric_horizons_market_sessions: Vec<u16>,
    pub metric_start_date: NaiveDate,
    pub metric_end_dates: Vec<NaiveDate>,
    pub recomputed_metrics_sha256: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub verdict: HistoricalOutcomeLabelAdmissionVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub reviewer_independent_from_validation_and_execution_chain: bool,
    pub exact_validation_current_binding_confirmed: bool,
    pub frozen_protocol_applicability_confirmed: bool,
    pub complete_horizons_and_common_session_endpoints_confirmed: bool,
    pub adjusted_close_and_corporate_action_basis_confirmed: bool,
    pub benchmark_comparability_confirmed: bool,
    pub event_time_and_future_isolation_confirmed: bool,
    pub missingness_and_survivorship_bias_reviewed: bool,
    pub no_manual_metric_override_confirmed: bool,
    pub label_semantics_and_direction_not_inferred_confirmed: bool,
    pub downstream_authority_remains_closed_confirmed: bool,
    pub outcome_label_input_admitted: bool,
    pub future_label_materialization_eligible: bool,
    pub outcome_label_written: bool,
    pub label_materialization_started: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelAdmissionItem {
    pub validation: HistoricalOutcomeDryRunOutputValidationRecord,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub latest_review: Option<HistoricalOutcomeLabelAdmissionReview>,
    pub current_binding: bool,
    pub review_eligible: bool,
    pub outcome_label_input_admitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelAdmissionRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeLabelAdmissionItem>,
    pub independently_validated_output_count: usize,
    pub review_eligible_output_count: usize,
    pub reviewed_output_count: usize,
    pub admitted_output_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub admission_status: String,
    pub outcome_label_input_admission_available: bool,
    pub outcome_label_materialization_enabled: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeLabelAdmissionReadinessSummary {
    pub independently_validated_output_count: usize,
    pub reviewed_output_count: usize,
    pub admitted_output_count: usize,
    pub rejected_or_changes_requested_count: usize,
    pub admission_status: String,
    pub outcome_label_materialization_enabled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct AdmittedHistoricalOutcomeForLabelMaterialization {
    pub admission_review: HistoricalOutcomeLabelAdmissionReview,
    pub validation: HistoricalOutcomeDryRunOutputValidationRecord,
    pub snapshot: super::historical_outcome_price_snapshots::HistoricalOutcomePriceSnapshot,
}

pub(crate) async fn handle_get_historical_outcome_label_admission_reviews(
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
            warn!(%error, "historical outcome label admission registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果标签准入复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_label_admission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeLabelAdmissionRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_admission(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome label admission registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果标签准入复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_label_admission_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelAdmissionReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeLabelAdmissionReadinessSummary {
        independently_validated_output_count: registry.independently_validated_output_count,
        reviewed_output_count: registry.reviewed_output_count,
        admitted_output_count: registry.admitted_output_count,
        rejected_or_changes_requested_count: registry.changes_requested_or_rejected_count,
        admission_status: registry.admission_status,
        outcome_label_materialization_enabled: false,
    })
}

pub(crate) async fn admitted_historical_outcomes_for_label_materialization(
    state: &AppState,
) -> Result<Vec<AdmittedHistoricalOutcomeForLabelMaterialization>, String> {
    let mut admitted = Vec::new();
    for validated in independently_validated_outputs_for_label_admission(state).await? {
        let Some(review) = latest_review(state, &validated).await? else {
            continue;
        };
        if review.verdict
            == HistoricalOutcomeLabelAdmissionVerdict::ApprovedForFutureLabelMaterialization
            && review.outcome_label_input_admitted
            && review.future_label_materialization_eligible
        {
            admitted.push(AdmittedHistoricalOutcomeForLabelMaterialization {
                admission_review: review,
                validation: validated.validation,
                snapshot: validated.snapshot,
            });
        }
    }
    admitted.sort_by(|left, right| {
        right
            .admission_review
            .submitted_at
            .cmp(&left.admission_review.submitted_at)
            .then_with(|| {
                right
                    .admission_review
                    .review_id
                    .cmp(&left.admission_review.review_id)
            })
    });
    Ok(admitted)
}

pub(crate) async fn exact_admitted_historical_outcome_for_label_materialization(
    state: &AppState,
    attempt_id: &str,
    admission_review_id: &str,
    admission_review_sha256: &str,
    validation_sha256: &str,
    output_sha256: &str,
    snapshot_sha256: &str,
    protocol_sha256: &str,
) -> Result<Option<AdmittedHistoricalOutcomeForLabelMaterialization>, String> {
    Ok(
        admitted_historical_outcomes_for_label_materialization(state)
            .await?
            .into_iter()
            .find(|value| {
                value.admission_review.attempt_id == attempt_id
                    && value.admission_review.review_id == admission_review_id
                    && value.admission_review.review_sha256 == admission_review_sha256
                    && value.validation.validation_sha256 == validation_sha256
                    && value.validation.output_sha256 == output_sha256
                    && value.snapshot.snapshot_sha256 == snapshot_sha256
                    && value.snapshot.protocol_sha256 == protocol_sha256
            }),
    )
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelAdmissionRegistry, String> {
    let validated = independently_validated_outputs_for_label_admission(state).await?;
    let mut items = Vec::with_capacity(validated.len());
    for value in validated {
        let latest_review = latest_review(state, &value).await?;
        let admitted = latest_review
            .as_ref()
            .is_some_and(|review| review.outcome_label_input_admitted);
        items.push(HistoricalOutcomeLabelAdmissionItem {
            validation: value.validation.clone(),
            asset_symbol: value.snapshot.asset_symbol.clone(),
            benchmark_symbol: value.snapshot.benchmark_symbol.clone(),
            decision_available_at: value.snapshot.decision_available_at,
            latest_review,
            current_binding: true,
            review_eligible: true,
            outcome_label_input_admitted: admitted,
        });
    }
    items.sort_by(|left, right| right.decision_available_at.cmp(&left.decision_available_at));
    let independently_validated_output_count = items.len();
    let reviewed_output_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let admitted_output_count = items
        .iter()
        .filter(|item| item.outcome_label_input_admitted)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.verdict != HistoricalOutcomeLabelAdmissionVerdict::ApprovedForFutureLabelMaterialization
            })
        })
        .count();
    let admission_status = if admitted_output_count > 0 {
        "label_input_admitted_waiting_materialization_implementation"
    } else if changes_requested_or_rejected_count > 0 {
        "label_admission_changes_requested_or_rejected"
    } else if independently_validated_output_count > 0 {
        "waiting_independent_label_admission_review"
    } else {
        "waiting_independently_validated_output"
    };
    Ok(HistoricalOutcomeLabelAdmissionRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        independently_validated_output_count,
        review_eligible_output_count: independently_validated_output_count,
        reviewed_output_count,
        admitted_output_count,
        changes_requested_or_rejected_count,
        admission_status: admission_status.to_string(),
        outcome_label_input_admission_available: true,
        outcome_label_materialization_enabled: false,
        outcome_label_written: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第十三阶段由另一名独立管理员审阅已重算一致的精确输出是否适用于冻结结果协议，并记录复权口径、基准可比性、共同交易日端点、未来隔离、缺失与幸存者偏差等局限。批准只接纳该输出作为未来标签物化输入；当前仍不写标签、不训练、不奖励、不建立影子组合、不生成订单或交易。".to_string(),
    })
}

async fn review_admission(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewHistoricalOutcomeLabelAdmissionRequest,
) -> Result<HistoricalOutcomeLabelAdmissionReview, String> {
    if !valid_id(attempt_id) {
        return Err("执行尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-outcome-label-admission-{attempt_id}"),
    )
    .await?;
    let validated = exact_independently_validated_output_for_label_admission(
        state,
        attempt_id,
        &request.expected_validation_id,
        &request.expected_validation_sha256,
        &request.expected_output_sha256,
        &request.expected_snapshot_sha256,
        &request.expected_protocol_sha256,
    )
    .await?
    .ok_or_else(|| "当前没有这条精确绑定且独立重算一致的输出".to_string())?;
    let latest = latest_review(state, &validated).await?;
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("结果标签准入复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限与偏差",
    )?;
    let validation = &validated.validation;
    let metrics = &validation.recomputed_metrics;
    let independent = admission_reviewer_is_independent(reviewer_id, validation);
    if !independent {
        return Err(
            "标签准入复核人不能是输出校验人、执行人、执行器登记者或任一授权复核人".to_string(),
        );
    }
    let mut review = HistoricalOutcomeLabelAdmissionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: validation.attempt_id.clone(),
        claim_sha256: validation.claim_sha256.clone(),
        result_id: validation.result_id.clone(),
        result_sha256: validation.result_sha256.clone(),
        output_sha256: validation.output_sha256.clone(),
        validation_id: validation.validation_id.clone(),
        validation_sha256: validation.validation_sha256.clone(),
        validator_implementation_version: validation.validator_implementation_version.clone(),
        validator_implementation_sha256: validation.validator_implementation_sha256.clone(),
        validated_by: validation.validated_by.clone(),
        snapshot_id: validated.snapshot.snapshot_id.clone(),
        snapshot_sha256: validated.snapshot.snapshot_sha256.clone(),
        reconstruction_id: validated.snapshot.reconstruction_id.clone(),
        reconstruction_sha256: validated.snapshot.reconstruction_sha256.clone(),
        protocol_version: validated.snapshot.protocol_version.clone(),
        protocol_sha256: validated.snapshot.protocol_sha256.clone(),
        asset_symbol: validated.snapshot.asset_symbol.clone(),
        benchmark_symbol: validated.snapshot.benchmark_symbol.clone(),
        decision_available_at: validated.snapshot.decision_available_at,
        common_session_count: validated.snapshot.common_session_count,
        metric_horizons_market_sessions: metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect(),
        metric_start_date: metrics
            .first()
            .ok_or_else(|| "独立重算记录没有指标".to_string())?
            .start_date,
        metric_end_dates: metrics.iter().map(|metric| metric.end_date).collect(),
        recomputed_metrics_sha256: hash_serializable(metrics)?,
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        verdict: request.verdict,
        rationale,
        known_limitations,
        reviewer_independent_from_validation_and_execution_chain: independent,
        exact_validation_current_binding_confirmed: request
            .exact_validation_current_binding_confirmed,
        frozen_protocol_applicability_confirmed: request.frozen_protocol_applicability_confirmed,
        complete_horizons_and_common_session_endpoints_confirmed: request
            .complete_horizons_and_common_session_endpoints_confirmed,
        adjusted_close_and_corporate_action_basis_confirmed: request
            .adjusted_close_and_corporate_action_basis_confirmed,
        benchmark_comparability_confirmed: request.benchmark_comparability_confirmed,
        event_time_and_future_isolation_confirmed: request
            .event_time_and_future_isolation_confirmed,
        missingness_and_survivorship_bias_reviewed: request
            .missingness_and_survivorship_bias_reviewed,
        no_manual_metric_override_confirmed: request.no_manual_metric_override_confirmed,
        label_semantics_and_direction_not_inferred_confirmed: request
            .label_semantics_and_direction_not_inferred_confirmed,
        downstream_authority_remains_closed_confirmed: request
            .downstream_authority_remains_closed_confirmed,
        outcome_label_input_admitted: false,
        future_label_materialization_eligible: false,
        outcome_label_written: false,
        label_materialization_started: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == HistoricalOutcomeLabelAdmissionVerdict::ApprovedForFutureLabelMaterialization;
    review.outcome_label_input_admitted = approved && review_all_checks(&review) && independent;
    review.future_label_materialization_eligible = review.outcome_label_input_admitted;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &validated)?;
    write_immutable_json(
        &review_directory(state, attempt_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn review_all_checks(review: &HistoricalOutcomeLabelAdmissionReview) -> bool {
    review.exact_validation_current_binding_confirmed
        && review.frozen_protocol_applicability_confirmed
        && review.complete_horizons_and_common_session_endpoints_confirmed
        && review.adjusted_close_and_corporate_action_basis_confirmed
        && review.benchmark_comparability_confirmed
        && review.event_time_and_future_isolation_confirmed
        && review.missingness_and_survivorship_bias_reviewed
        && review.no_manual_metric_override_confirmed
        && review.label_semantics_and_direction_not_inferred_confirmed
        && review.downstream_authority_remains_closed_confirmed
}

fn admission_reviewer_is_independent(
    reviewer_id: &str,
    validation: &HistoricalOutcomeDryRunOutputValidationRecord,
) -> bool {
    !reviewer_id.trim().is_empty()
        && reviewer_id != validation.validated_by
        && reviewer_id != validation.execution_invoked_by
        && reviewer_id != validation.runner_registered_by
        && reviewer_id != validation.first_execution_authorization_reviewer_id
        && reviewer_id != validation.run_authorization_reviewer_id
}

fn validate_review(
    review: &HistoricalOutcomeLabelAdmissionReview,
    validated: &IndependentlyValidatedHistoricalOutcomeForLabelAdmission,
) -> Result<(), String> {
    let validation = &validated.validation;
    let snapshot = &validated.snapshot;
    let approved = review.verdict
        == HistoricalOutcomeLabelAdmissionVerdict::ApprovedForFutureLabelMaterialization;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let downstream_closed = !review.outcome_label_written
        && !review.label_materialization_started
        && !review.training_authorized
        && !review.reward_authorized
        && !review.shadow_portfolio_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let expected_independence = admission_reviewer_is_independent(&review.reviewer_id, validation);
    let expected_admitted = approved && review_all_checks(review) && expected_independence;
    let expected_horizons = validation
        .recomputed_metrics
        .iter()
        .map(|metric| metric.horizon_market_sessions)
        .collect::<Vec<_>>();
    let expected_end_dates = validation
        .recomputed_metrics
        .iter()
        .map(|metric| metric.end_date)
        .collect::<Vec<_>>();
    let expected_start_date = validation
        .recomputed_metrics
        .first()
        .map(|metric| metric.start_date);
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != POLICY_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.review_sha256)
        || review.review_id != review.review_sha256[..32]
        || review.review_sha256 != review_fingerprint(review)?
        || !previous_pair_valid
        || review
            .previous_review_id
            .as_deref()
            .is_some_and(|value| !valid_id(value))
        || review
            .previous_review_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256(value))
        || review.attempt_id != validation.attempt_id
        || review.claim_sha256 != validation.claim_sha256
        || review.result_id != validation.result_id
        || review.result_sha256 != validation.result_sha256
        || review.output_sha256 != validation.output_sha256
        || review.validation_id != validation.validation_id
        || review.validation_sha256 != validation.validation_sha256
        || review.validator_implementation_version != validation.validator_implementation_version
        || review.validator_implementation_sha256 != validation.validator_implementation_sha256
        || review.validated_by != validation.validated_by
        || review.snapshot_id != snapshot.snapshot_id
        || review.snapshot_sha256 != snapshot.snapshot_sha256
        || review.reconstruction_id != snapshot.reconstruction_id
        || review.reconstruction_sha256 != snapshot.reconstruction_sha256
        || review.protocol_version != snapshot.protocol_version
        || review.protocol_sha256 != snapshot.protocol_sha256
        || review.asset_symbol != snapshot.asset_symbol
        || review.benchmark_symbol != snapshot.benchmark_symbol
        || review.decision_available_at != snapshot.decision_available_at
        || review.common_session_count != snapshot.common_session_count
        || review.metric_horizons_market_sessions != expected_horizons
        || review.metric_horizons_market_sessions != vec![20, 60, 250]
        || Some(review.metric_start_date) != expected_start_date
        || review.metric_end_dates != expected_end_dates
        || review.recomputed_metrics_sha256 != hash_serializable(&validation.recomputed_metrics)?
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || review.known_limitations.trim().is_empty()
        || review.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || review.reviewer_independent_from_validation_and_execution_chain != expected_independence
        || (approved && !review_all_checks(review))
        || review.outcome_label_input_admitted != expected_admitted
        || review.future_label_materialization_eligible != expected_admitted
        || !downstream_closed
    {
        return Err(
            "historical outcome label admission review is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn review_fingerprint(review: &HistoricalOutcomeLabelAdmissionReview) -> Result<String, String> {
    let mut value = serde_json::to_value(review).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "label admission review fingerprint payload is invalid".to_string())?;
    object.remove("review_id");
    object.remove("review_sha256");
    hash_serializable(&value)
}

async fn latest_review(
    state: &AppState,
    validated: &IndependentlyValidatedHistoricalOutcomeForLabelAdmission,
) -> Result<Option<HistoricalOutcomeLabelAdmissionReview>, String> {
    let directory = review_directory(state, &validated.validation.attempt_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = tokio::fs::read(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        let review: HistoricalOutcomeLabelAdmissionReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, validated)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("historical outcome label admission filename mismatch".to_string());
        }
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err(
                "historical outcome label admission contains duplicate records".to_string(),
            );
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeLabelAdmissionReview],
) -> Result<Option<HistoricalOutcomeLabelAdmissionReview>, String> {
    if reviews.is_empty() {
        return Ok(None);
    }
    let by_id = reviews
        .iter()
        .map(|review| (review.review_id.as_str(), review))
        .collect::<HashMap<_, _>>();
    let tips = reviews
        .iter()
        .filter(|review| {
            !reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
        })
        .collect::<Vec<_>>();
    if tips.len() != 1 {
        return Err("historical outcome label admission chain has invalid tips".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("historical outcome label admission chain has a cycle".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(previous_id), Some(previous_sha256)) => {
                let previous = by_id.get(previous_id.as_str()).ok_or_else(|| {
                    "historical outcome label admission chain is disconnected".to_string()
                })?;
                if previous.review_sha256 != *previous_sha256 {
                    return Err(
                        "historical outcome label admission previous hash mismatch".to_string()
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "historical outcome label admission previous link is invalid".to_string(),
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err("historical outcome label admission chain is disconnected".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > max_chars {
        return Err(format!("{label}不能超过 {max_chars} 个字符"));
    }
    Ok(value.to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options
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

fn review_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-label-admission-reviews")
        .join(attempt_id)
}

struct AdmissionLock {
    path: PathBuf,
}

impl Drop for AdmissionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, name: &str) -> Result<AdmissionLock, String> {
    if !valid_id(name) {
        return Err("invalid historical outcome label admission lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
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
        .map_err(|_| "已有结果标签准入复核正在进行".to_string())?;
    Ok(AdmissionLock { path })
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::super::historical_outcome_dry_run_execution_attempts::HistoricalOutcomeDryRunMetric;
    use super::super::historical_outcome_dry_run_output_validations::{
        HistoricalOutcomeDryRunOutputValidationVerdict, validation_matches_label_admission_request,
    };
    use super::super::historical_outcome_price_snapshots::{
        HistoricalOutcomePriceSnapshot, SealedAdjustedClosePoint,
    };
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

    fn validation() -> HistoricalOutcomeDryRunOutputValidationRecord {
        HistoricalOutcomeDryRunOutputValidationRecord {
            schema_version: "test".into(),
            policy_version: "test".into(),
            validation_id: "validation-1".into(),
            validation_sha256: "a".repeat(64),
            attempt_id: "attempt-1".into(),
            claim_sha256: "b".repeat(64),
            result_id: "result-1".into(),
            result_sha256: "c".repeat(64),
            output_sha256: "d".repeat(64),
            isolated_runner_id: "runner-1".into(),
            isolated_runner_spec_sha256: "e".repeat(64),
            runner_artifact_sha256: "f".repeat(64),
            authorization_review_id: "authorization-1".into(),
            authorization_review_sha256: "1".repeat(64),
            snapshot_id: "snapshot-1".into(),
            snapshot_sha256: "2".repeat(64),
            protocol_version: "protocol-v1".into(),
            protocol_sha256: "3".repeat(64),
            validator_implementation_version: "validator-v1".into(),
            validator_implementation_sha256: "4".repeat(64),
            validated_at: Utc::now(),
            validated_by: "validator".into(),
            execution_invoked_by: "invoker".into(),
            runner_registered_by: "registrant".into(),
            first_execution_authorization_reviewer_id: "first-reviewer".into(),
            run_authorization_reviewer_id: "run-reviewer".into(),
            validator_independent_from_execution_and_prior_reviewers: true,
            immutable_chain_integrity_verified: true,
            current_sealed_snapshot_binding_verified: true,
            canonical_output_hash_verified: true,
            output_structure_verified: true,
            deterministic_recomputation_match: true,
            recomputed_metrics: metrics(),
            mismatch_reasons: vec![],
            verdict: HistoricalOutcomeDryRunOutputValidationVerdict::ValidatedDeterministicMatch,
            output_validated: true,
            outcome_label_admission_authorized: false,
            outcome_label_written: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        }
    }

    fn validated() -> IndependentlyValidatedHistoricalOutcomeForLabelAdmission {
        let validation = validation();
        let points = (0..=250)
            .map(|day| SealedAdjustedClosePoint {
                date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap() + chrono::Days::new(day),
                adjusted_close: 100.0 + day as f64,
            })
            .collect::<Vec<_>>();
        let snapshot = HistoricalOutcomePriceSnapshot {
            schema_version: "test".into(),
            ingestion_policy_version: "test".into(),
            snapshot_id: validation.snapshot_id.clone(),
            snapshot_sha256: validation.snapshot_sha256.clone(),
            sealed_at: Utc::now(),
            sealed_by: "admin".into(),
            reconstruction_id: "reconstruction-1".into(),
            reconstruction_sha256: "5".repeat(64),
            reconstruction_review_id: "reconstruction-review-1".into(),
            anchor_candidate_id: "anchor-1".into(),
            anchor_review_id: "anchor-review-1".into(),
            decision_available_at: Utc::now(),
            implementation_id: "implementation-1".into(),
            implementation_spec_sha256: "6".repeat(64),
            implementation_review_id: "implementation-review-1".into(),
            governance_review_id: "governance-1".into(),
            protocol_version: validation.protocol_version.clone(),
            protocol_sha256: validation.protocol_sha256.clone(),
            code_revision: "test".into(),
            provider: "test".into(),
            provider_endpoint_template: "test".into(),
            price_basis: "adjusted close".into(),
            asset_symbol: "TEST".into(),
            benchmark_symbol: "SPY".into(),
            requested_from: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            requested_to: NaiveDate::from_ymd_opt(2025, 9, 8).unwrap(),
            asset_payload_sha256: "7".repeat(64),
            benchmark_payload_sha256: "8".repeat(64),
            asset_series_sha256: "9".repeat(64),
            benchmark_series_sha256: "a".repeat(64),
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
        };
        IndependentlyValidatedHistoricalOutcomeForLabelAdmission {
            validation,
            snapshot,
        }
    }

    fn approved_review() -> (
        HistoricalOutcomeLabelAdmissionReview,
        IndependentlyValidatedHistoricalOutcomeForLabelAdmission,
    ) {
        let validated = validated();
        let validation = &validated.validation;
        let snapshot = &validated.snapshot;
        let metrics = &validation.recomputed_metrics;
        let mut review = HistoricalOutcomeLabelAdmissionReview {
            schema_version: REVIEW_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id: None,
            previous_review_sha256: None,
            attempt_id: validation.attempt_id.clone(),
            claim_sha256: validation.claim_sha256.clone(),
            result_id: validation.result_id.clone(),
            result_sha256: validation.result_sha256.clone(),
            output_sha256: validation.output_sha256.clone(),
            validation_id: validation.validation_id.clone(),
            validation_sha256: validation.validation_sha256.clone(),
            validator_implementation_version: validation.validator_implementation_version.clone(),
            validator_implementation_sha256: validation.validator_implementation_sha256.clone(),
            validated_by: validation.validated_by.clone(),
            snapshot_id: snapshot.snapshot_id.clone(),
            snapshot_sha256: snapshot.snapshot_sha256.clone(),
            reconstruction_id: snapshot.reconstruction_id.clone(),
            reconstruction_sha256: snapshot.reconstruction_sha256.clone(),
            protocol_version: snapshot.protocol_version.clone(),
            protocol_sha256: snapshot.protocol_sha256.clone(),
            asset_symbol: snapshot.asset_symbol.clone(),
            benchmark_symbol: snapshot.benchmark_symbol.clone(),
            decision_available_at: snapshot.decision_available_at,
            common_session_count: snapshot.common_session_count,
            metric_horizons_market_sessions: vec![20, 60, 250],
            metric_start_date: metrics[0].start_date,
            metric_end_dates: metrics.iter().map(|metric| metric.end_date).collect(),
            recomputed_metrics_sha256: hash_serializable(metrics).unwrap(),
            submitted_at: Utc::now(),
            reviewer_id: "admission-reviewer".into(),
            verdict: HistoricalOutcomeLabelAdmissionVerdict::ApprovedForFutureLabelMaterialization,
            rationale: "逐项核对通过".into(),
            known_limitations: "单标的历史样本仍存在幸存者偏差，不据此推断策略收益".into(),
            reviewer_independent_from_validation_and_execution_chain: true,
            exact_validation_current_binding_confirmed: true,
            frozen_protocol_applicability_confirmed: true,
            complete_horizons_and_common_session_endpoints_confirmed: true,
            adjusted_close_and_corporate_action_basis_confirmed: true,
            benchmark_comparability_confirmed: true,
            event_time_and_future_isolation_confirmed: true,
            missingness_and_survivorship_bias_reviewed: true,
            no_manual_metric_override_confirmed: true,
            label_semantics_and_direction_not_inferred_confirmed: true,
            downstream_authority_remains_closed_confirmed: true,
            outcome_label_input_admitted: true,
            future_label_materialization_eligible: true,
            outcome_label_written: false,
            label_materialization_started: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        (review, validated)
    }

    #[test]
    fn approved_review_requires_every_applicability_and_bias_check() {
        let (mut review, validated) = approved_review();
        validate_review(&review, &validated).unwrap();
        review.missingness_and_survivorship_bias_reviewed = false;
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review, &validated).is_err());
    }

    #[test]
    fn admission_reviewer_must_be_independent_from_every_prior_role() {
        let validation = validation();
        assert!(admission_reviewer_is_independent(
            "new-reviewer",
            &validation
        ));
        for prior in [
            "validator",
            "invoker",
            "registrant",
            "first-reviewer",
            "run-reviewer",
        ] {
            assert!(!admission_reviewer_is_independent(prior, &validation));
        }
    }

    #[test]
    fn exact_validation_binding_tamper_is_rejected() {
        let (mut review, validated) = approved_review();
        review.validation_sha256 = "0".repeat(64);
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review, &validated).is_err());
    }

    #[test]
    fn failed_or_unvalidated_output_cannot_enter_admission_review() {
        let mut value = validation();
        value.output_validated = false;
        value.verdict =
            HistoricalOutcomeDryRunOutputValidationVerdict::FailedStructuralOrRecomputationMismatch;
        assert!(!validation_matches_label_admission_request(
            &value,
            &value.attempt_id,
            &value.validation_id,
            &value.validation_sha256,
            &value.output_sha256,
            &value.snapshot_sha256,
            &value.protocol_sha256,
        ));
    }

    #[test]
    fn downstream_authority_tamper_is_rejected() {
        let (mut review, validated) = approved_review();
        review.training_authorized = true;
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review, &validated).is_err());
    }

    #[test]
    fn linear_review_chain_is_accepted_and_fork_is_rejected() {
        let (first, _) = approved_review();
        let mut second = first.clone();
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some(first.review_sha256.clone());
        second.submitted_at += chrono::TimeDelta::seconds(1);
        second.verdict = HistoricalOutcomeLabelAdmissionVerdict::ChangesRequested;
        second.outcome_label_input_admitted = false;
        second.future_label_materialization_eligible = false;
        second.review_sha256 = review_fingerprint(&second).unwrap();
        second.review_id = second.review_sha256[..32].to_string();
        assert_eq!(
            latest_review_from_records(&[first.clone(), second.clone()])
                .unwrap()
                .unwrap()
                .review_id,
            second.review_id
        );
        let mut fork = second.clone();
        fork.submitted_at += chrono::TimeDelta::seconds(1);
        fork.previous_review_id = Some(first.review_id.clone());
        fork.previous_review_sha256 = Some(first.review_sha256.clone());
        fork.review_sha256 = review_fingerprint(&fork).unwrap();
        fork.review_id = fork.review_sha256[..32].to_string();
        assert!(latest_review_from_records(&[first, second, fork]).is_err());
    }

    #[test]
    fn review_chain_rejects_previous_hash_mismatch() {
        let (first, _) = approved_review();
        let mut second = first.clone();
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some("0".repeat(64));
        second.submitted_at += chrono::TimeDelta::seconds(1);
        second.review_sha256 = review_fingerprint(&second).unwrap();
        second.review_id = second.review_sha256[..32].to_string();
        assert!(latest_review_from_records(&[first, second]).is_err());
    }

    #[test]
    fn replayed_review_record_is_rejected() {
        let (first, _) = approved_review();
        assert!(latest_review_from_records(&[first.clone(), first]).is_err());
    }
}
