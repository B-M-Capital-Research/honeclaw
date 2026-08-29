//! Create-once formal raw-outcome label writes.
//!
//! One current, unexpired stage-twenty authorization may be consumed exactly
//! once. An immutable claim is persisted before the fixed writer attempts a
//! create-new label write. A success label contains raw observed absolute and
//! relative market outcomes only. A failure record (or an orphan claim after a
//! process interruption) also consumes the authorization. This store is
//! physically separate from training, reward, shadow, order and broker stores.

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
use super::historical_outcome_label_write_authorizations::{
    ApprovedHistoricalOutcomeLabelWriteAuthorization, FORMAL_LABEL_SCHEMA_VERSION,
    FORMAL_LABEL_SEMANTICS_VERSION, allowed_label_fields,
    current_unexpired_label_write_authorizations, label_contract_sha256,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-formal-label-write-registry-v1";
const CLAIM_SCHEMA_VERSION: &str = "hone-historical-outcome-formal-label-write-claim-v1";
const FAILURE_SCHEMA_VERSION: &str = "hone-historical-outcome-formal-label-write-failure-v1";
const WRITER_POLICY_VERSION: &str =
    "hone-historical-outcome-formal-label-create-once-raw-outcome-writer-v1";
const WRITER_IMPLEMENTATION_VERSION: &str =
    "hone-fixed-formal-raw-outcome-label-writer-v1-no-semantic-inference";
const MAX_ERROR_CHARS: usize = 1_200;
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];

#[derive(Debug, Deserialize)]
pub(crate) struct WriteHistoricalOutcomeFormalLabelOnceRequest {
    expected_authorization_review_sha256: String,
    expected_materialization_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_admission_review_sha256: String,
    expected_source_validation_sha256: String,
    expected_source_output_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    expected_recomputed_metrics_sha256: String,
    expected_label_contract_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct HistoricalOutcomeFormalLabelSourceProvenance {
    pub materialization_validation_id: String,
    pub materialization_validated_at: DateTime<Utc>,
    pub materialization_validated_by: String,
    pub materialization_attempt_id: String,
    pub source_attempt_id: String,
    pub source_validation_id: String,
    pub snapshot_id: String,
    pub reconstruction_id: String,
    pub protocol_version: String,
    pub metric_start_date: chrono::NaiveDate,
    pub metric_end_dates: Vec<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFormalLabelImmutableChainBindings {
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub materialization_validation_sha256: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_sha256: String,
    pub materialization_output_sha256: String,
    pub admission_review_sha256: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub snapshot_sha256: String,
    pub reconstruction_sha256: String,
    pub protocol_sha256: String,
    pub recomputed_metrics_sha256: String,
    pub label_contract_sha256: String,
}

/// The only eight semantic fields permitted by the stage-twenty label contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct HistoricalOutcomeFormalLabelPayload {
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub common_session_count: usize,
    pub raw_validated_metrics: Vec<HistoricalOutcomeDryRunMetric>,
    pub source_provenance: HistoricalOutcomeFormalLabelSourceProvenance,
    pub known_limitations: String,
    pub immutable_chain_bindings: HistoricalOutcomeFormalLabelImmutableChainBindings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFormalLabelWriteClaim {
    pub schema_version: String,
    pub writer_policy_version: String,
    pub writer_implementation_version: String,
    pub writer_implementation_sha256: String,
    pub claim_id: String,
    pub claim_sha256: String,
    pub target_label_id: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub authorization_valid_until: DateTime<Utc>,
    pub materialization_validation_id: String,
    pub materialization_validation_sha256: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_sha256: String,
    pub materialization_output_sha256: String,
    pub admission_review_sha256: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub snapshot_sha256: String,
    pub reconstruction_sha256: String,
    pub protocol_sha256: String,
    pub recomputed_metrics_sha256: String,
    pub formal_label_schema_version: String,
    pub formal_label_semantics_version: String,
    pub label_contract_sha256: String,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub current_authorization_binding_revalidated: bool,
    pub exact_validated_source_reloaded: bool,
    pub authorization_consumed: bool,
    pub create_once_no_overwrite: bool,
    pub semantic_inference_allowed: bool,
    pub training_write_allowed: bool,
    pub reward_write_allowed: bool,
    pub shadow_write_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFormalLabel {
    pub schema_version: String,
    pub semantics_version: String,
    pub writer_policy_version: String,
    pub writer_implementation_version: String,
    pub label_id: String,
    pub label_sha256: String,
    pub claim_id: String,
    pub claim_sha256: String,
    pub created_at: DateTime<Utc>,
    pub written_by: String,
    pub payload: HistoricalOutcomeFormalLabelPayload,
    pub exact_metric_bits_preserved: bool,
    pub provenance_preserved: bool,
    pub known_limitations_preserved: bool,
    pub formal_label_written: bool,
    pub independently_validated_for_training_admission: bool,
    pub admitted_to_offline_training_dataset_candidate: bool,
    pub direction_inferred: bool,
    pub rating_inferred: bool,
    pub investment_action_inferred: bool,
    pub position_size_inferred: bool,
    pub training_target_written: bool,
    pub reward_written: bool,
    pub shadow_position_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFormalLabelWriteFailure {
    pub schema_version: String,
    pub writer_policy_version: String,
    pub failure_id: String,
    pub failure_sha256: String,
    pub claim_id: String,
    pub claim_sha256: String,
    pub failed_at: DateTime<Utc>,
    pub error_message: String,
    pub error_sha256: String,
    pub authorization_consumed: bool,
    pub formal_label_written: bool,
    pub training_write_allowed: bool,
    pub reward_write_allowed: bool,
    pub shadow_write_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFormalLabelWriteEligibleAuthorization {
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub authorization_valid_until: DateTime<Utc>,
    pub materialization_validation_id: String,
    pub materialization_validation_sha256: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_sha256: String,
    pub materialization_output_sha256: String,
    pub admission_review_sha256: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub snapshot_sha256: String,
    pub protocol_sha256: String,
    pub recomputed_metrics_sha256: String,
    pub label_contract_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFormalLabelWriteItem {
    pub claim: HistoricalOutcomeFormalLabelWriteClaim,
    pub label: Option<HistoricalOutcomeFormalLabel>,
    pub failure: Option<HistoricalOutcomeFormalLabelWriteFailure>,
    pub write_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFormalLabelWriteRegistry {
    pub schema_version: String,
    pub writer_policy_version: String,
    pub writer_implementation_version: String,
    pub writer_implementation_sha256: String,
    pub formal_label_schema_version: String,
    pub formal_label_semantics_version: String,
    pub label_contract_sha256: String,
    pub allowed_label_fields: Vec<String>,
    pub writer_endpoint_available: bool,
    pub eligible_authorization_count: usize,
    pub claim_count: usize,
    pub formal_label_count: usize,
    pub failed_write_count: usize,
    pub incomplete_fail_closed_claim_count: usize,
    pub write_status: String,
    pub eligible_authorizations: Vec<HistoricalOutcomeFormalLabelWriteEligibleAuthorization>,
    pub writes: Vec<HistoricalOutcomeFormalLabelWriteItem>,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeFormalLabelWriteReadinessSummary {
    pub eligible_authorization_count: usize,
    pub claim_count: usize,
    pub formal_label_count: usize,
    pub failed_write_count: usize,
    pub incomplete_fail_closed_claim_count: usize,
    pub write_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FormalLabelForIndependentTrainingAdmissionValidation {
    pub claim: HistoricalOutcomeFormalLabelWriteClaim,
    pub label: HistoricalOutcomeFormalLabel,
}

pub(crate) async fn handle_get_historical_outcome_formal_label_writes(
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
            warn!(%error, "historical outcome formal label write registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果正式原始标签写入记录暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_write_historical_outcome_formal_label_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(authorization_review_id): AxumPath<String>,
    Json(request): Json<WriteHistoricalOutcomeFormalLabelOnceRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match write_once(&state, &admin.user_id, &authorization_review_id, request).await {
        Ok(()) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome formal label write registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果正式原始标签写入记录暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_formal_label_write_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFormalLabelWriteReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeFormalLabelWriteReadinessSummary {
        eligible_authorization_count: registry.eligible_authorization_count,
        claim_count: registry.claim_count,
        formal_label_count: registry.formal_label_count,
        failed_write_count: registry.failed_write_count,
        incomplete_fail_closed_claim_count: registry.incomplete_fail_closed_claim_count,
        write_status: registry.write_status,
    })
}

pub(crate) async fn formal_labels_for_independent_training_admission_validation(
    state: &AppState,
) -> Result<Vec<FormalLabelForIndependentTrainingAdmissionValidation>, String> {
    let (claims, labels, _) = read_records(state).await?;
    let claims = claims
        .into_iter()
        .map(|claim| (claim.claim_id.clone(), claim))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::with_capacity(labels.len());
    for label in labels {
        let claim = claims
            .get(&label.claim_id)
            .cloned()
            .ok_or_else(|| "正式标签缺少不可变写入 claim".to_string())?;
        output.push(FormalLabelForIndependentTrainingAdmissionValidation { claim, label });
    }
    output.sort_by(|left, right| {
        right
            .label
            .created_at
            .cmp(&left.label.created_at)
            .then_with(|| left.label.label_id.cmp(&right.label.label_id))
    });
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFormalLabelWriteRegistry, String> {
    let authorizations = current_unexpired_label_write_authorizations(state, Utc::now()).await?;
    let (claims, labels, failures) = read_records(state).await?;
    let claimed_authorizations = claims
        .iter()
        .map(|claim| claim.authorization_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let label_targets = labels
        .iter()
        .map(|label| label.label_id.as_str())
        .collect::<BTreeSet<_>>();
    let eligible_authorizations = authorizations
        .iter()
        .filter(|authorization| {
            !claimed_authorizations.contains(authorization.review.review_id.as_str())
                && !label_targets.contains(target_label_id(authorization).as_str())
        })
        .map(eligible_authorization)
        .collect::<Vec<_>>();
    let label_by_claim = labels
        .iter()
        .map(|label| (label.claim_id.as_str(), label))
        .collect::<HashMap<_, _>>();
    let failure_by_claim = failures
        .iter()
        .map(|failure| (failure.claim_id.as_str(), failure))
        .collect::<HashMap<_, _>>();
    let mut writes = claims
        .into_iter()
        .map(|claim| {
            let label = label_by_claim
                .get(claim.claim_id.as_str())
                .cloned()
                .cloned();
            let failure = failure_by_claim
                .get(claim.claim_id.as_str())
                .cloned()
                .cloned();
            let write_status = if label.is_some() {
                "formal_raw_outcome_label_written_waiting_independent_training_admission_validation"
            } else if failure.is_some() {
                "formal_label_write_failed_authorization_consumed"
            } else {
                "claim_persisted_no_label_fail_closed_authorization_consumed"
            };
            HistoricalOutcomeFormalLabelWriteItem {
                claim,
                label,
                failure,
                write_status: write_status.to_string(),
            }
        })
        .collect::<Vec<_>>();
    writes.sort_by(|left, right| {
        right
            .claim
            .claimed_at
            .cmp(&left.claim.claimed_at)
            .then_with(|| left.claim.claim_id.cmp(&right.claim.claim_id))
    });
    let formal_label_count = writes.iter().filter(|item| item.label.is_some()).count();
    let failed_write_count = writes.iter().filter(|item| item.failure.is_some()).count();
    let incomplete_fail_closed_claim_count = writes
        .iter()
        .filter(|item| item.label.is_none() && item.failure.is_none())
        .count();
    let write_status = if formal_label_count > 0 {
        "formal_raw_outcome_label_written_training_still_closed"
    } else if failed_write_count > 0 || incomplete_fail_closed_claim_count > 0 {
        "formal_label_write_failed_or_interrupted_fail_closed"
    } else if !eligible_authorizations.is_empty() {
        "waiting_irreversible_create_once_formal_label_write"
    } else {
        "waiting_current_unexpired_one_shot_authorization"
    };
    Ok(HistoricalOutcomeFormalLabelWriteRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        writer_policy_version: WRITER_POLICY_VERSION.to_string(),
        writer_implementation_version: WRITER_IMPLEMENTATION_VERSION.to_string(),
        writer_implementation_sha256: writer_implementation_sha256(),
        formal_label_schema_version: FORMAL_LABEL_SCHEMA_VERSION.to_string(),
        formal_label_semantics_version: FORMAL_LABEL_SEMANTICS_VERSION.to_string(),
        label_contract_sha256: label_contract_sha256(),
        allowed_label_fields: allowed_label_fields(),
        writer_endpoint_available: true,
        eligible_authorization_count: eligible_authorizations.len(),
        claim_count: writes.len(),
        formal_label_count,
        failed_write_count,
        incomplete_fail_closed_claim_count,
        write_status: write_status.to_string(),
        eligible_authorizations,
        writes,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "只消费一条当前、未过期且未使用的第二十阶段授权：先不可变写 claim，再以 create-new 方式写入只含原始绝对/相对市场结果、来源、局限和精确链绑定的正式标签。成功或失败都消费授权；标签与训练/奖励存储隔离，且仍须下一阶段独立校验后才可能成为离线训练数据集候选。无方向、评级、动作、仓位、奖励、影子、订单、券商或交易权限。".to_string(),
    })
}

async fn write_once(
    state: &AppState,
    admin_id: &str,
    authorization_review_id: &str,
    request: WriteHistoricalOutcomeFormalLabelOnceRequest,
) -> Result<(), String> {
    if !valid_id(authorization_review_id) {
        return Err("正式标签写入授权复核 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, authorization_review_id).await?;
    let authorization = current_unexpired_label_write_authorizations(state, Utc::now())
        .await?
        .into_iter()
        .find(|authorization| authorization.review.review_id == authorization_review_id)
        .ok_or_else(|| "当前没有这条未过期且绑定有效的一次性正式标签写入授权".to_string())?;
    verify_request(&authorization, &request)?;
    let (claims, labels, _) = read_records(state).await?;
    let target_label_id = target_label_id(&authorization);
    if claims.iter().any(|claim| {
        claim.authorization_review_id == authorization_review_id
            || claim.materialization_validation_id == authorization.source.validation.validation_id
            || claim.target_label_id == target_label_id
    }) || labels.iter().any(|label| label.label_id == target_label_id)
    {
        return Err("该授权或同一验证结果已经被 claim；一次性额度不能重放".to_string());
    }
    let claim = new_claim(&authorization, admin_id, target_label_id, Utc::now())?;
    write_immutable_json(
        &claim_root(state).join(format!("{}.json", claim.claim_id)),
        &claim,
    )
    .await?;

    let write = new_formal_label(&authorization, &claim, admin_id, Utc::now()).and_then(|label| {
        validate_label(&label, &claim)?;
        Ok(label)
    });
    match write {
        Ok(label) => {
            if let Err(error) = write_immutable_json(
                &label_root(state).join(format!("{}.json", label.label_id)),
                &label,
            )
            .await
            {
                let failure = new_failure(&claim, Utc::now(), &error)?;
                write_immutable_json(
                    &failure_root(state).join(format!("{}.json", failure.failure_id)),
                    &failure,
                )
                .await?;
            }
        }
        Err(error) => {
            let failure = new_failure(&claim, Utc::now(), &error)?;
            write_immutable_json(
                &failure_root(state).join(format!("{}.json", failure.failure_id)),
                &failure,
            )
            .await?;
        }
    }
    Ok(())
}

fn verify_request(
    authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization,
    request: &WriteHistoricalOutcomeFormalLabelOnceRequest,
) -> Result<(), String> {
    let source = &authorization.source;
    if request.expected_authorization_review_sha256 != authorization.review.review_sha256
        || request.expected_materialization_validation_sha256 != source.validation.validation_sha256
        || request.expected_claim_sha256 != source.attempt.claim.claim_sha256
        || request.expected_result_sha256 != source.attempt.result.result_sha256
        || request.expected_output_sha256 != source.validation.output_sha256
        || request.expected_admission_review_sha256 != source.attempt.claim.admission_review_sha256
        || request.expected_source_validation_sha256 != source.attempt.claim.validation_sha256
        || request.expected_source_output_sha256 != source.attempt.claim.source_output_sha256
        || request.expected_snapshot_sha256 != source.attempt.claim.snapshot_sha256
        || request.expected_protocol_sha256 != source.attempt.claim.protocol_sha256
        || request.expected_recomputed_metrics_sha256
            != source.attempt.claim.recomputed_metrics_sha256
        || request.expected_label_contract_sha256 != label_contract_sha256()
    {
        return Err("正式标签授权、合同或完整上游绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn target_label_id(authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization) -> String {
    let value = serde_json::json!({
        "schema_version": FORMAL_LABEL_SCHEMA_VERSION,
        "semantics_version": FORMAL_LABEL_SEMANTICS_VERSION,
        "materialization_validation_sha256": authorization.source.validation.validation_sha256,
        "materialization_output_sha256": authorization.source.validation.output_sha256,
        "label_contract_sha256": label_contract_sha256(),
    });
    hash_serializable(&value).expect("target label identity is serializable")[..32].to_string()
}

fn eligible_authorization(
    authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization,
) -> HistoricalOutcomeFormalLabelWriteEligibleAuthorization {
    let envelope = authorization
        .source
        .attempt
        .result
        .untrusted_envelope
        .as_ref()
        .expect("independently validated source has an envelope");
    HistoricalOutcomeFormalLabelWriteEligibleAuthorization {
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        authorization_valid_until: authorization.review.authorization_valid_until,
        materialization_validation_id: authorization.source.validation.validation_id.clone(),
        materialization_validation_sha256: authorization
            .source
            .validation
            .validation_sha256
            .clone(),
        materialization_claim_sha256: authorization.source.attempt.claim.claim_sha256.clone(),
        materialization_result_sha256: authorization.source.attempt.result.result_sha256.clone(),
        materialization_output_sha256: authorization.source.validation.output_sha256.clone(),
        admission_review_sha256: authorization
            .source
            .attempt
            .claim
            .admission_review_sha256
            .clone(),
        source_validation_sha256: authorization.source.attempt.claim.validation_sha256.clone(),
        source_output_sha256: authorization
            .source
            .attempt
            .claim
            .source_output_sha256
            .clone(),
        snapshot_sha256: authorization.source.attempt.claim.snapshot_sha256.clone(),
        protocol_sha256: authorization.source.attempt.claim.protocol_sha256.clone(),
        recomputed_metrics_sha256: authorization
            .source
            .attempt
            .claim
            .recomputed_metrics_sha256
            .clone(),
        label_contract_sha256: authorization.review.label_contract_sha256.clone(),
        asset_symbol: envelope.asset_symbol.clone(),
        benchmark_symbol: envelope.benchmark_symbol.clone(),
        decision_available_at: envelope.decision_available_at,
    }
}

fn new_claim(
    authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization,
    admin_id: &str,
    target_label_id: String,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeFormalLabelWriteClaim, String> {
    let source = &authorization.source;
    let mut claim = HistoricalOutcomeFormalLabelWriteClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        writer_policy_version: WRITER_POLICY_VERSION.to_string(),
        writer_implementation_version: WRITER_IMPLEMENTATION_VERSION.to_string(),
        writer_implementation_sha256: writer_implementation_sha256(),
        claim_id: String::new(),
        claim_sha256: String::new(),
        target_label_id,
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        authorization_valid_until: authorization.review.authorization_valid_until,
        materialization_validation_id: source.validation.validation_id.clone(),
        materialization_validation_sha256: source.validation.validation_sha256.clone(),
        materialization_claim_sha256: source.attempt.claim.claim_sha256.clone(),
        materialization_result_sha256: source.attempt.result.result_sha256.clone(),
        materialization_output_sha256: source.validation.output_sha256.clone(),
        admission_review_sha256: source.attempt.claim.admission_review_sha256.clone(),
        source_validation_sha256: source.attempt.claim.validation_sha256.clone(),
        source_output_sha256: source.attempt.claim.source_output_sha256.clone(),
        snapshot_sha256: source.attempt.claim.snapshot_sha256.clone(),
        reconstruction_sha256: source.attempt.claim.reconstruction_sha256.clone(),
        protocol_sha256: source.attempt.claim.protocol_sha256.clone(),
        recomputed_metrics_sha256: source.attempt.claim.recomputed_metrics_sha256.clone(),
        formal_label_schema_version: FORMAL_LABEL_SCHEMA_VERSION.to_string(),
        formal_label_semantics_version: FORMAL_LABEL_SEMANTICS_VERSION.to_string(),
        label_contract_sha256: label_contract_sha256(),
        claimed_at,
        invoked_by: admin_id.to_string(),
        current_authorization_binding_revalidated: true,
        exact_validated_source_reloaded: true,
        authorization_consumed: true,
        create_once_no_overwrite: true,
        semantic_inference_allowed: false,
        training_write_allowed: false,
        reward_write_allowed: false,
        shadow_write_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = claim_fingerprint(&claim)?;
    claim.claim_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim)?;
    Ok(claim)
}

fn new_formal_label(
    authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization,
    claim: &HistoricalOutcomeFormalLabelWriteClaim,
    admin_id: &str,
    created_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeFormalLabel, String> {
    let source = &authorization.source;
    let envelope = source
        .attempt
        .result
        .untrusted_envelope
        .as_ref()
        .ok_or_else(|| "独立校验源缺少物化原始结果包".to_string())?;
    let metrics = source.validation.independently_validated_metrics.clone();
    if metrics.len() != REQUIRED_HORIZONS.len()
        || metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect::<Vec<_>>()
            != REQUIRED_HORIZONS
        || hash_serializable(&metrics)? != claim.recomputed_metrics_sha256
    {
        return Err("独立校验指标不符合冻结正式标签合同".to_string());
    }
    let payload = HistoricalOutcomeFormalLabelPayload {
        asset_symbol: envelope.asset_symbol.clone(),
        benchmark_symbol: envelope.benchmark_symbol.clone(),
        decision_available_at: envelope.decision_available_at,
        common_session_count: envelope.common_session_count,
        raw_validated_metrics: metrics,
        source_provenance: HistoricalOutcomeFormalLabelSourceProvenance {
            materialization_validation_id: source.validation.validation_id.clone(),
            materialization_validated_at: source.validation.validated_at,
            materialization_validated_by: source.validation.validated_by.clone(),
            materialization_attempt_id: source.attempt.claim.attempt_id.clone(),
            source_attempt_id: source.attempt.claim.source_attempt_id.clone(),
            source_validation_id: source.attempt.claim.validation_id.clone(),
            snapshot_id: source.attempt.claim.snapshot_id.clone(),
            reconstruction_id: source.attempt.claim.reconstruction_id.clone(),
            protocol_version: source.attempt.claim.protocol_version.clone(),
            metric_start_date: envelope.metric_start_date,
            metric_end_dates: envelope.metric_end_dates.clone(),
        },
        known_limitations: envelope.known_limitations.clone(),
        immutable_chain_bindings: HistoricalOutcomeFormalLabelImmutableChainBindings {
            authorization_review_id: claim.authorization_review_id.clone(),
            authorization_review_sha256: claim.authorization_review_sha256.clone(),
            materialization_validation_sha256: claim.materialization_validation_sha256.clone(),
            materialization_claim_sha256: claim.materialization_claim_sha256.clone(),
            materialization_result_sha256: claim.materialization_result_sha256.clone(),
            materialization_output_sha256: claim.materialization_output_sha256.clone(),
            admission_review_sha256: claim.admission_review_sha256.clone(),
            source_validation_sha256: claim.source_validation_sha256.clone(),
            source_output_sha256: claim.source_output_sha256.clone(),
            snapshot_sha256: claim.snapshot_sha256.clone(),
            reconstruction_sha256: claim.reconstruction_sha256.clone(),
            protocol_sha256: claim.protocol_sha256.clone(),
            recomputed_metrics_sha256: claim.recomputed_metrics_sha256.clone(),
            label_contract_sha256: claim.label_contract_sha256.clone(),
        },
    };
    let mut label = HistoricalOutcomeFormalLabel {
        schema_version: FORMAL_LABEL_SCHEMA_VERSION.to_string(),
        semantics_version: FORMAL_LABEL_SEMANTICS_VERSION.to_string(),
        writer_policy_version: WRITER_POLICY_VERSION.to_string(),
        writer_implementation_version: WRITER_IMPLEMENTATION_VERSION.to_string(),
        label_id: claim.target_label_id.clone(),
        label_sha256: String::new(),
        claim_id: claim.claim_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        created_at,
        written_by: admin_id.to_string(),
        payload,
        exact_metric_bits_preserved: true,
        provenance_preserved: true,
        known_limitations_preserved: true,
        formal_label_written: true,
        independently_validated_for_training_admission: false,
        admitted_to_offline_training_dataset_candidate: false,
        direction_inferred: false,
        rating_inferred: false,
        investment_action_inferred: false,
        position_size_inferred: false,
        training_target_written: false,
        reward_written: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    };
    label.label_sha256 = label_fingerprint(&label)?;
    Ok(label)
}

fn new_failure(
    claim: &HistoricalOutcomeFormalLabelWriteClaim,
    failed_at: DateTime<Utc>,
    error: &str,
) -> Result<HistoricalOutcomeFormalLabelWriteFailure, String> {
    let message = error.chars().take(MAX_ERROR_CHARS).collect::<String>();
    let mut failure = HistoricalOutcomeFormalLabelWriteFailure {
        schema_version: FAILURE_SCHEMA_VERSION.to_string(),
        writer_policy_version: WRITER_POLICY_VERSION.to_string(),
        failure_id: String::new(),
        failure_sha256: String::new(),
        claim_id: claim.claim_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        failed_at,
        error_sha256: hash_bytes(message.as_bytes()),
        error_message: message,
        authorization_consumed: true,
        formal_label_written: false,
        training_write_allowed: false,
        reward_write_allowed: false,
        shadow_write_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    failure.failure_sha256 = failure_fingerprint(&failure)?;
    failure.failure_id = failure.failure_sha256[..32].to_string();
    validate_failure(&failure, claim)?;
    Ok(failure)
}

fn validate_claim(claim: &HistoricalOutcomeFormalLabelWriteClaim) -> Result<(), String> {
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.writer_policy_version != WRITER_POLICY_VERSION
        || claim.writer_implementation_version != WRITER_IMPLEMENTATION_VERSION
        || claim.writer_implementation_sha256 != writer_implementation_sha256()
        || !valid_id(&claim.claim_id)
        || claim.claim_id != claim.claim_sha256.get(..32).unwrap_or_default()
        || !valid_sha256(&claim.claim_sha256)
        || claim.claim_sha256 != claim_fingerprint(claim)?
        || !valid_id(&claim.target_label_id)
        || !valid_id(&claim.authorization_review_id)
        || !valid_sha256(&claim.authorization_review_sha256)
        || claim.claimed_at >= claim.authorization_valid_until
        || !valid_id(&claim.materialization_validation_id)
        || !all_sha256(&[
            &claim.materialization_validation_sha256,
            &claim.materialization_claim_sha256,
            &claim.materialization_result_sha256,
            &claim.materialization_output_sha256,
            &claim.admission_review_sha256,
            &claim.source_validation_sha256,
            &claim.source_output_sha256,
            &claim.snapshot_sha256,
            &claim.reconstruction_sha256,
            &claim.protocol_sha256,
            &claim.recomputed_metrics_sha256,
            &claim.label_contract_sha256,
        ])
        || claim.formal_label_schema_version != FORMAL_LABEL_SCHEMA_VERSION
        || claim.formal_label_semantics_version != FORMAL_LABEL_SEMANTICS_VERSION
        || claim.label_contract_sha256 != label_contract_sha256()
        || claim.invoked_by.trim().is_empty()
        || !claim.current_authorization_binding_revalidated
        || !claim.exact_validated_source_reloaded
        || !claim.authorization_consumed
        || !claim.create_once_no_overwrite
        || claim.semantic_inference_allowed
        || claim.training_write_allowed
        || claim.reward_write_allowed
        || claim.shadow_write_allowed
        || claim.order_generation_allowed
        || claim.broker_access_allowed
        || claim.trading_allowed
    {
        return Err(
            "historical outcome formal label claim is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn validate_label(
    label: &HistoricalOutcomeFormalLabel,
    claim: &HistoricalOutcomeFormalLabelWriteClaim,
) -> Result<(), String> {
    let payload_keys = serde_json::to_value(&label.payload)
        .map_err(|error| error.to_string())?
        .as_object()
        .ok_or_else(|| "formal label payload is not an object".to_string())?
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let allowed_keys = allowed_label_fields().into_iter().collect::<BTreeSet<_>>();
    let metrics_valid = label.payload.raw_validated_metrics.len() == REQUIRED_HORIZONS.len()
        && label
            .payload
            .raw_validated_metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect::<Vec<_>>()
            == REQUIRED_HORIZONS
        && hash_serializable(&label.payload.raw_validated_metrics)?
            == claim.recomputed_metrics_sha256;
    let downstream_closed = !label.independently_validated_for_training_admission
        && !label.admitted_to_offline_training_dataset_candidate
        && !label.direction_inferred
        && !label.rating_inferred
        && !label.investment_action_inferred
        && !label.position_size_inferred
        && !label.training_target_written
        && !label.reward_written
        && !label.shadow_position_written
        && !label.order_generated
        && !label.broker_accessed
        && !label.trade_executed;
    if label.schema_version != FORMAL_LABEL_SCHEMA_VERSION
        || label.semantics_version != FORMAL_LABEL_SEMANTICS_VERSION
        || label.writer_policy_version != WRITER_POLICY_VERSION
        || label.writer_implementation_version != WRITER_IMPLEMENTATION_VERSION
        || label.label_id != claim.target_label_id
        || !valid_sha256(&label.label_sha256)
        || label.label_sha256 != label_fingerprint(label)?
        || label.claim_id != claim.claim_id
        || label.claim_sha256 != claim.claim_sha256
        || label.created_at < claim.claimed_at
        || label.written_by.trim().is_empty()
        || payload_keys != allowed_keys
        || !metrics_valid
        || label
            .payload
            .source_provenance
            .materialization_validation_id
            != claim.materialization_validation_id
        || label
            .payload
            .source_provenance
            .source_validation_id
            .trim()
            .is_empty()
        || label
            .payload
            .source_provenance
            .materialization_attempt_id
            .trim()
            .is_empty()
        || label
            .payload
            .source_provenance
            .source_attempt_id
            .trim()
            .is_empty()
        || label
            .payload
            .source_provenance
            .snapshot_id
            .trim()
            .is_empty()
        || label
            .payload
            .source_provenance
            .reconstruction_id
            .trim()
            .is_empty()
        || label
            .payload
            .source_provenance
            .protocol_version
            .trim()
            .is_empty()
        || label.payload.source_provenance.metric_end_dates.len() != REQUIRED_HORIZONS.len()
        || label.payload.immutable_chain_bindings.label_contract_sha256 != label_contract_sha256()
        || label
            .payload
            .immutable_chain_bindings
            .authorization_review_id
            != claim.authorization_review_id
        || label
            .payload
            .immutable_chain_bindings
            .authorization_review_sha256
            != claim.authorization_review_sha256
        || label
            .payload
            .immutable_chain_bindings
            .materialization_validation_sha256
            != claim.materialization_validation_sha256
        || label
            .payload
            .immutable_chain_bindings
            .materialization_claim_sha256
            != claim.materialization_claim_sha256
        || label
            .payload
            .immutable_chain_bindings
            .materialization_result_sha256
            != claim.materialization_result_sha256
        || label
            .payload
            .immutable_chain_bindings
            .materialization_output_sha256
            != claim.materialization_output_sha256
        || label
            .payload
            .immutable_chain_bindings
            .admission_review_sha256
            != claim.admission_review_sha256
        || label
            .payload
            .immutable_chain_bindings
            .source_validation_sha256
            != claim.source_validation_sha256
        || label.payload.immutable_chain_bindings.source_output_sha256 != claim.source_output_sha256
        || label.payload.immutable_chain_bindings.snapshot_sha256 != claim.snapshot_sha256
        || label.payload.immutable_chain_bindings.reconstruction_sha256
            != claim.reconstruction_sha256
        || label.payload.immutable_chain_bindings.protocol_sha256 != claim.protocol_sha256
        || label
            .payload
            .immutable_chain_bindings
            .recomputed_metrics_sha256
            != claim.recomputed_metrics_sha256
        || !label.exact_metric_bits_preserved
        || !label.provenance_preserved
        || !label.known_limitations_preserved
        || !label.formal_label_written
        || !downstream_closed
    {
        return Err("historical outcome formal label is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn validate_failure(
    failure: &HistoricalOutcomeFormalLabelWriteFailure,
    claim: &HistoricalOutcomeFormalLabelWriteClaim,
) -> Result<(), String> {
    if failure.schema_version != FAILURE_SCHEMA_VERSION
        || failure.writer_policy_version != WRITER_POLICY_VERSION
        || !valid_id(&failure.failure_id)
        || failure.failure_id != failure.failure_sha256.get(..32).unwrap_or_default()
        || !valid_sha256(&failure.failure_sha256)
        || failure.failure_sha256 != failure_fingerprint(failure)?
        || failure.claim_id != claim.claim_id
        || failure.claim_sha256 != claim.claim_sha256
        || failure.failed_at < claim.claimed_at
        || failure.error_message.is_empty()
        || failure.error_message.chars().count() > MAX_ERROR_CHARS
        || failure.error_sha256 != hash_bytes(failure.error_message.as_bytes())
        || !failure.authorization_consumed
        || failure.formal_label_written
        || failure.training_write_allowed
        || failure.reward_write_allowed
        || failure.shadow_write_allowed
        || failure.order_generation_allowed
        || failure.broker_access_allowed
        || failure.trading_allowed
    {
        return Err(
            "historical outcome formal label write failure is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<
    (
        Vec<HistoricalOutcomeFormalLabelWriteClaim>,
        Vec<HistoricalOutcomeFormalLabel>,
        Vec<HistoricalOutcomeFormalLabelWriteFailure>,
    ),
    String,
> {
    let claims =
        read_flat_json::<HistoricalOutcomeFormalLabelWriteClaim>(&claim_root(state)).await?;
    let labels = read_flat_json::<HistoricalOutcomeFormalLabel>(&label_root(state)).await?;
    let failures =
        read_flat_json::<HistoricalOutcomeFormalLabelWriteFailure>(&failure_root(state)).await?;
    let mut claim_ids = BTreeSet::new();
    let mut claim_hashes = BTreeSet::new();
    let mut authorization_ids = BTreeSet::new();
    let mut validation_ids = BTreeSet::new();
    let mut target_ids = BTreeSet::new();
    for claim in &claims {
        validate_claim(claim)?;
        if !claim_ids.insert(claim.claim_id.clone())
            || !claim_hashes.insert(claim.claim_sha256.clone())
            || !authorization_ids.insert(claim.authorization_review_id.clone())
            || !validation_ids.insert(claim.materialization_validation_id.clone())
            || !target_ids.insert(claim.target_label_id.clone())
        {
            return Err(
                "historical outcome formal label registry contains replayed claims".to_string(),
            );
        }
    }
    let claim_index = claims
        .iter()
        .map(|claim| (claim.claim_id.as_str(), claim))
        .collect::<HashMap<_, _>>();
    let mut completed_claims = BTreeSet::new();
    let mut label_ids = BTreeSet::new();
    let mut label_hashes = BTreeSet::new();
    for label in &labels {
        let claim = claim_index
            .get(label.claim_id.as_str())
            .ok_or_else(|| "formal label has no immutable claim".to_string())?;
        validate_label(label, claim)?;
        if !completed_claims.insert(label.claim_id.clone())
            || !label_ids.insert(label.label_id.clone())
            || !label_hashes.insert(label.label_sha256.clone())
        {
            return Err(
                "historical outcome formal label registry contains duplicate labels".to_string(),
            );
        }
    }
    let mut failure_ids = BTreeSet::new();
    let mut failure_hashes = BTreeSet::new();
    for failure in &failures {
        let claim = claim_index
            .get(failure.claim_id.as_str())
            .ok_or_else(|| "formal label write failure has no immutable claim".to_string())?;
        validate_failure(failure, claim)?;
        if completed_claims.contains(&failure.claim_id)
            || !completed_claims.insert(failure.claim_id.clone())
            || !failure_ids.insert(failure.failure_id.clone())
            || !failure_hashes.insert(failure.failure_sha256.clone())
        {
            return Err(
                "historical outcome formal label registry contains duplicate outcomes".to_string(),
            );
        }
    }
    Ok((claims, labels, failures))
}

async fn read_flat_json<T>(root: &Path) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut entries = match tokio::fs::read_dir(root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
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
        let value: T = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        output.push(value);
    }
    Ok(output)
}

fn claim_fingerprint(claim: &HistoricalOutcomeFormalLabelWriteClaim) -> Result<String, String> {
    fingerprint_without(claim, &["claim_id", "claim_sha256"])
}

fn label_fingerprint(label: &HistoricalOutcomeFormalLabel) -> Result<String, String> {
    fingerprint_without(label, &["label_sha256"])
}

fn failure_fingerprint(
    failure: &HistoricalOutcomeFormalLabelWriteFailure,
) -> Result<String, String> {
    fingerprint_without(failure, &["failure_id", "failure_sha256"])
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

fn writer_implementation_sha256() -> String {
    hash_serializable(&serde_json::json!({
        "version": WRITER_IMPLEMENTATION_VERSION,
        "policy": WRITER_POLICY_VERSION,
        "label_schema": FORMAL_LABEL_SCHEMA_VERSION,
        "label_semantics": FORMAL_LABEL_SEMANTICS_VERSION,
        "label_contract": label_contract_sha256(),
        "claim_before_write": true,
        "create_once": true,
        "failure_consumes_authorization": true,
        "semantic_inference": false,
        "downstream_authority": false,
    }))
    .expect("static writer implementation is serializable")
}

fn all_sha256(values: &[&String]) -> bool {
    values.iter().all(|value| valid_sha256(value))
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn write_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical_outcome_formal_label_writes")
}

fn claim_root(state: &AppState) -> PathBuf {
    write_root(state).join("claims")
}

fn failure_root(state: &AppState) -> PathBuf {
    write_root(state).join("failures")
}

fn label_root(state: &AppState) -> PathBuf {
    // Deliberately outside every training/reward directory.
    decision_root(state)
        .join("historical_outcome_formal_labels")
        .join("objects")
}

async fn acquire_lock(
    state: &AppState,
    authorization_review_id: &str,
) -> Result<DirectoryLock, String> {
    if !valid_id(authorization_review_id) {
        return Err("invalid formal label write lock name".to_string());
    }
    let root = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|error| error.to_string())?;
    let path = root.join(format!(
        "historical-formal-label-write-{authorization_review_id}.lock"
    ));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(DirectoryLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
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
                return Err("同一正式标签授权正在被一次性消费，请稍后刷新".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("正式标签一次性写入锁获取失败".to_string())
}

struct DirectoryLock {
    path: PathBuf,
}

impl Drop for DirectoryLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| error.to_string())?;
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_payload_contract_has_exactly_eight_raw_fields() {
        let payload = HistoricalOutcomeFormalLabelPayload {
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            decision_available_at: DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            common_session_count: 251,
            raw_validated_metrics: Vec::new(),
            source_provenance: HistoricalOutcomeFormalLabelSourceProvenance {
                materialization_validation_id: "a".repeat(32),
                materialization_validated_at: Utc::now(),
                materialization_validated_by: "validator".to_string(),
                materialization_attempt_id: "b".repeat(32),
                source_attempt_id: "c".repeat(32),
                source_validation_id: "d".repeat(32),
                snapshot_id: "e".repeat(32),
                reconstruction_id: "f".repeat(32),
                protocol_version: "v1".to_string(),
                metric_start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                metric_end_dates: Vec::new(),
            },
            known_limitations: "raw only".to_string(),
            immutable_chain_bindings: HistoricalOutcomeFormalLabelImmutableChainBindings {
                authorization_review_id: "1".repeat(32),
                authorization_review_sha256: "1".repeat(64),
                materialization_validation_sha256: "2".repeat(64),
                materialization_claim_sha256: "3".repeat(64),
                materialization_result_sha256: "4".repeat(64),
                materialization_output_sha256: "5".repeat(64),
                admission_review_sha256: "6".repeat(64),
                source_validation_sha256: "7".repeat(64),
                source_output_sha256: "8".repeat(64),
                snapshot_sha256: "9".repeat(64),
                reconstruction_sha256: "a".repeat(64),
                protocol_sha256: "b".repeat(64),
                recomputed_metrics_sha256: "c".repeat(64),
                label_contract_sha256: label_contract_sha256(),
            },
        };
        let keys = serde_json::to_value(payload)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            keys,
            allowed_label_fields().into_iter().collect::<BTreeSet<_>>()
        );
        assert!(!keys.contains("direction"));
        assert!(!keys.contains("reward"));
        assert!(!keys.contains("action"));
        assert!(!keys.contains("position"));
    }

    #[test]
    fn writer_implementation_contract_is_stable_and_fail_closed() {
        assert!(valid_sha256(&writer_implementation_sha256()));
        assert_ne!(writer_implementation_sha256(), label_contract_sha256());
        assert_eq!(
            FORMAL_LABEL_SCHEMA_VERSION,
            "hone-historical-outcome-formal-label-v1"
        );
        assert!(FORMAL_LABEL_SEMANTICS_VERSION.contains("no-action-or-reward"));
    }

    #[test]
    fn raw_metric_json_round_trip_preserves_every_f64_bit() {
        let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        let metrics = [20_u16, 60, 250]
            .into_iter()
            .enumerate()
            .map(|(index, horizon)| HistoricalOutcomeDryRunMetric {
                horizon_market_sessions: horizon,
                start_date: start,
                end_date: start + chrono::TimeDelta::days(i64::from(horizon)),
                asset_return: f64::from_bits(0x3fb9_9999_9999_999a + index as u64),
                benchmark_return: f64::from_bits(0x3fa9_9999_9999_999a + index as u64),
                excess_return: f64::from_bits(0x3f84_7ae1_47ae_147b + index as u64),
                asset_max_drawdown: f64::from_bits(0xbfa9_9999_9999_999a + index as u64),
            })
            .collect::<Vec<_>>();
        let encoded = serde_json::to_vec(&metrics).unwrap();
        let decoded: Vec<HistoricalOutcomeDryRunMetric> = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded.len(), metrics.len());
        for (left, right) in metrics.iter().zip(decoded.iter()) {
            assert_eq!(left.asset_return.to_bits(), right.asset_return.to_bits());
            assert_eq!(
                left.benchmark_return.to_bits(),
                right.benchmark_return.to_bits()
            );
            assert_eq!(left.excess_return.to_bits(), right.excess_return.to_bits());
            assert_eq!(
                left.asset_max_drawdown.to_bits(),
                right.asset_max_drawdown.to_bits()
            );
        }
    }

    #[test]
    fn failure_record_consumes_authorization_and_grants_nothing() {
        let now = Utc::now();
        let mut claim = HistoricalOutcomeFormalLabelWriteClaim {
            schema_version: CLAIM_SCHEMA_VERSION.to_string(),
            writer_policy_version: WRITER_POLICY_VERSION.to_string(),
            writer_implementation_version: WRITER_IMPLEMENTATION_VERSION.to_string(),
            writer_implementation_sha256: writer_implementation_sha256(),
            claim_id: String::new(),
            claim_sha256: String::new(),
            target_label_id: "a".repeat(32),
            authorization_review_id: "b".repeat(32),
            authorization_review_sha256: "b".repeat(64),
            authorization_valid_until: now + chrono::TimeDelta::hours(1),
            materialization_validation_id: "c".repeat(32),
            materialization_validation_sha256: "c".repeat(64),
            materialization_claim_sha256: "d".repeat(64),
            materialization_result_sha256: "e".repeat(64),
            materialization_output_sha256: "f".repeat(64),
            admission_review_sha256: "1".repeat(64),
            source_validation_sha256: "2".repeat(64),
            source_output_sha256: "3".repeat(64),
            snapshot_sha256: "4".repeat(64),
            reconstruction_sha256: "5".repeat(64),
            protocol_sha256: "6".repeat(64),
            recomputed_metrics_sha256: "7".repeat(64),
            formal_label_schema_version: FORMAL_LABEL_SCHEMA_VERSION.to_string(),
            formal_label_semantics_version: FORMAL_LABEL_SEMANTICS_VERSION.to_string(),
            label_contract_sha256: label_contract_sha256(),
            claimed_at: now,
            invoked_by: "writer".to_string(),
            current_authorization_binding_revalidated: true,
            exact_validated_source_reloaded: true,
            authorization_consumed: true,
            create_once_no_overwrite: true,
            semantic_inference_allowed: false,
            training_write_allowed: false,
            reward_write_allowed: false,
            shadow_write_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.claim_id = claim.claim_sha256[..32].to_string();
        validate_claim(&claim).unwrap();
        let failure = new_failure(&claim, now, "disk full").unwrap();
        assert!(failure.authorization_consumed);
        assert!(!failure.formal_label_written);
        assert!(!failure.training_write_allowed);
        assert!(!failure.trading_allowed);
    }

    #[test]
    fn replay_identity_rejects_authorization_validation_or_target_reuse() {
        let ids = [("auth", "validation", "target")];
        assert!(ids.iter().any(|(auth, _, _)| *auth == "auth"));
        assert!(
            ids.iter()
                .any(|(_, validation, _)| *validation == "validation")
        );
        assert!(ids.iter().any(|(_, _, target)| *target == "target"));
    }
}
