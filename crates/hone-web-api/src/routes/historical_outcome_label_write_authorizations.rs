//! Append-only authorization review for one future formal raw-outcome label write.
//!
//! A passing stage-nineteen materialized envelope may be reviewed for exactly
//! one future create-once write. This module never writes a label and grants no
//! training, reward, shadow, order, broker or trading authority.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_label_materialization_output_validations::{
    IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope,
    independently_validated_materialized_envelopes_for_label_write_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-write-authorization-registry-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-historical-outcome-label-write-authorization-review-v1";
const POLICY_VERSION: &str =
    "hone-historical-outcome-formal-label-write-authorization-v1-single-use-24h";
pub(crate) const FORMAL_LABEL_SCHEMA_VERSION: &str = "hone-historical-outcome-formal-label-v1";
pub(crate) const FORMAL_LABEL_SEMANTICS_VERSION: &str =
    "raw-observed-absolute-and-relative-market-outcomes-no-action-or-reward-v1";
const AUTHORIZATION_VALID_HOURS: i64 = 24;
const ONE_SHOT_LABEL_WRITE_LIMIT: u8 = 1;
const MAX_RATIONALE_CHARS: usize = 2_400;
const ALLOWED_LABEL_FIELDS: [&str; 8] = [
    "asset_symbol",
    "benchmark_symbol",
    "decision_available_at",
    "common_session_count",
    "raw_validated_metrics",
    "source_provenance",
    "known_limitations",
    "immutable_chain_bindings",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeLabelWriteAuthorizationVerdict {
    ApprovedForOneShotFormalLabelWrite,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeLabelWriteAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
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
    verdict: HistoricalOutcomeLabelWriteAuthorizationVerdict,
    rationale: String,
    exact_validated_envelope_binding_confirmed: bool,
    reviewer_independence_confirmed: bool,
    formal_label_schema_confirmed: bool,
    raw_outcome_semantics_only_confirmed: bool,
    exact_metric_bits_and_provenance_confirmed: bool,
    known_limitations_preserved_confirmed: bool,
    create_once_no_overwrite_writer_confirmed: bool,
    single_use_and_expiry_confirmed: bool,
    label_store_isolated_from_training_confirmed: bool,
    no_semantic_inference_or_reward_confirmed: bool,
    no_network_tools_or_unrelated_production_access_confirmed: bool,
    no_training_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelWriteAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub materialization_validation_id: String,
    pub materialization_validation_sha256: String,
    pub materialization_validated_at: DateTime<Utc>,
    pub materialization_validated_by: String,
    pub materialization_attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
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
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub common_session_count: usize,
    pub metric_horizons_market_sessions: Vec<u16>,
    pub metric_start_date: NaiveDate,
    pub metric_end_dates: Vec<NaiveDate>,
    pub known_limitations: String,
    pub formal_label_schema_version: String,
    pub formal_label_semantics_version: String,
    pub label_contract_sha256: String,
    pub allowed_label_fields: Vec<String>,
    pub submitted_at: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeLabelWriteAuthorizationVerdict,
    pub rationale: String,
    pub exact_validated_envelope_binding_confirmed: bool,
    pub reviewer_independence_confirmed: bool,
    pub formal_label_schema_confirmed: bool,
    pub raw_outcome_semantics_only_confirmed: bool,
    pub exact_metric_bits_and_provenance_confirmed: bool,
    pub known_limitations_preserved_confirmed: bool,
    pub create_once_no_overwrite_writer_confirmed: bool,
    pub single_use_and_expiry_confirmed: bool,
    pub label_store_isolated_from_training_confirmed: bool,
    pub no_semantic_inference_or_reward_confirmed: bool,
    pub no_network_tools_or_unrelated_production_access_confirmed: bool,
    pub no_training_shadow_order_broker_or_trading_confirmed: bool,
    pub reviewer_independent_from_complete_prior_chain: bool,
    pub one_shot_label_write_limit: u8,
    pub one_shot_formal_label_write_authorized: bool,
    pub authorization_consumed: bool,
    pub label_writer_endpoint_available: bool,
    pub outcome_label_write_allowed: bool,
    pub outcome_label_written: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelWriteAuthorizationItem {
    pub materialization_validation_id: String,
    pub materialization_validation_sha256: String,
    pub materialization_attempt_id: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub admission_review_sha256: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub snapshot_sha256: String,
    pub protocol_sha256: String,
    pub recomputed_metrics_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub current_binding: bool,
    pub latest_review: Option<HistoricalOutcomeLabelWriteAuthorizationReview>,
    pub review_eligible: bool,
    pub one_shot_formal_label_write_authorized: bool,
    pub authorization_consumed_by_formal_label_writer: bool,
    pub authorization_unexpired: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelWriteAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub formal_label_schema_version: String,
    pub formal_label_semantics_version: String,
    pub label_contract_sha256: String,
    pub allowed_label_fields: Vec<String>,
    pub items: Vec<HistoricalOutcomeLabelWriteAuthorizationItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub one_shot_authorized_count: usize,
    pub unexpired_authorization_count: usize,
    pub authorization_status: String,
    pub label_writer_endpoint_available: bool,
    pub outcome_label_write_allowed: bool,
    pub outcome_label_written: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeLabelWriteAuthorizationReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub one_shot_authorized_count: usize,
    pub unexpired_authorization_count: usize,
    pub authorization_status: String,
    pub outcome_label_write_allowed: bool,
    pub outcome_label_written: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ApprovedHistoricalOutcomeLabelWriteAuthorization {
    pub source: IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope,
    pub review: HistoricalOutcomeLabelWriteAuthorizationReview,
}

pub(crate) async fn handle_get_historical_outcome_label_write_authorizations(
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
            warn!(%error, "historical outcome label write authorization registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果正式标签写入授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_label_write_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(validation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeLabelWriteAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_authorization(&state, &admin.user_id, &validation_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome label write authorization registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果正式标签写入授权复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_label_write_authorization_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelWriteAuthorizationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeLabelWriteAuthorizationReadinessSummary {
        review_eligible_count: registry.review_eligible_count,
        reviewed_count: registry.reviewed_count,
        one_shot_authorized_count: registry.one_shot_authorized_count,
        unexpired_authorization_count: registry.unexpired_authorization_count,
        authorization_status: registry.authorization_status,
        outcome_label_write_allowed: false,
        outcome_label_written: false,
    })
}

#[allow(dead_code)]
pub(crate) async fn current_unexpired_label_write_authorizations(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Vec<ApprovedHistoricalOutcomeLabelWriteAuthorization>, String> {
    let sources =
        independently_validated_materialized_envelopes_for_label_write_review(state).await?;
    let consumed = consumed_authorization_review_ids(state).await?;
    let mut output = Vec::new();
    for source in sources {
        let Some(review) = latest_review(state, &source).await? else {
            continue;
        };
        if authorization_unexpired(&review, now) && !consumed.contains(&review.review_id) {
            output.push(ApprovedHistoricalOutcomeLabelWriteAuthorization { source, review });
        }
    }
    Ok(output)
}

pub(crate) async fn exact_label_write_authorization_for_audit(
    state: &AppState,
    review_id: &str,
    review_sha256: &str,
) -> Result<Option<ApprovedHistoricalOutcomeLabelWriteAuthorization>, String> {
    let sources =
        independently_validated_materialized_envelopes_for_label_write_review(state).await?;
    for source in sources {
        let Some(review) = latest_review(state, &source).await? else {
            continue;
        };
        if review.review_id == review_id
            && review.review_sha256 == review_sha256
            && review.one_shot_formal_label_write_authorized
        {
            return Ok(Some(ApprovedHistoricalOutcomeLabelWriteAuthorization {
                source,
                review,
            }));
        }
    }
    Ok(None)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelWriteAuthorizationRegistry, String> {
    let sources =
        independently_validated_materialized_envelopes_for_label_write_review(state).await?;
    let consumed = consumed_authorization_review_ids(state).await?;
    let now = Utc::now();
    let mut items = Vec::with_capacity(sources.len());
    for source in sources {
        let latest_review = latest_review(state, &source).await?;
        let authorized = latest_review
            .as_ref()
            .is_some_and(|review| review.one_shot_formal_label_write_authorized);
        let unexpired = latest_review.as_ref().is_some_and(|review| {
            authorization_unexpired(review, now) && !consumed.contains(&review.review_id)
        });
        let authorization_consumed_by_formal_label_writer = latest_review
            .as_ref()
            .is_some_and(|review| consumed.contains(&review.review_id));
        let envelope = source
            .attempt
            .result
            .untrusted_envelope
            .as_ref()
            .ok_or_else(|| "标签物化结果缺少未信任原始结果包".to_string())?;
        items.push(HistoricalOutcomeLabelWriteAuthorizationItem {
            materialization_validation_id: source.validation.validation_id.clone(),
            materialization_validation_sha256: source.validation.validation_sha256.clone(),
            materialization_attempt_id: source.attempt.claim.attempt_id.clone(),
            claim_sha256: source.attempt.claim.claim_sha256.clone(),
            result_sha256: source.attempt.result.result_sha256.clone(),
            output_sha256: source.validation.output_sha256.clone(),
            admission_review_sha256: source.attempt.claim.admission_review_sha256.clone(),
            source_validation_sha256: source.attempt.claim.validation_sha256.clone(),
            source_output_sha256: source.attempt.claim.source_output_sha256.clone(),
            snapshot_sha256: source.attempt.claim.snapshot_sha256.clone(),
            protocol_sha256: source.attempt.claim.protocol_sha256.clone(),
            recomputed_metrics_sha256: source.attempt.claim.recomputed_metrics_sha256.clone(),
            asset_symbol: envelope.asset_symbol.clone(),
            benchmark_symbol: envelope.benchmark_symbol.clone(),
            decision_available_at: envelope.decision_available_at,
            current_binding: true,
            review_eligible: true,
            latest_review,
            one_shot_formal_label_write_authorized: authorized,
            authorization_consumed_by_formal_label_writer,
            authorization_unexpired: unexpired,
        });
    }
    items.sort_by(|left, right| {
        right
            .decision_available_at
            .cmp(&left.decision_available_at)
            .then_with(|| {
                left.materialization_validation_id
                    .cmp(&right.materialization_validation_id)
            })
    });
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let one_shot_authorized_count = items
        .iter()
        .filter(|item| item.one_shot_formal_label_write_authorized)
        .count();
    let unexpired_authorization_count = items
        .iter()
        .filter(|item| item.one_shot_formal_label_write_authorized && item.authorization_unexpired)
        .count();
    let authorization_status = if items.is_empty() {
        "waiting_independently_validated_materialized_envelope"
    } else if unexpired_authorization_count > 0 {
        "one_shot_formal_label_write_authorized_not_written"
    } else if reviewed_count > 0 {
        "reviewed_not_currently_authorized"
    } else {
        "waiting_independent_formal_label_write_authorization_review"
    };
    Ok(HistoricalOutcomeLabelWriteAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        formal_label_schema_version: FORMAL_LABEL_SCHEMA_VERSION.to_string(),
        formal_label_semantics_version: FORMAL_LABEL_SEMANTICS_VERSION.to_string(),
        label_contract_sha256: label_contract_sha256(),
        allowed_label_fields: allowed_label_fields(),
        review_eligible_count: items.len(),
        reviewed_count,
        one_shot_authorized_count,
        unexpired_authorization_count,
        items,
        authorization_status: authorization_status.to_string(),
        label_writer_endpoint_available: false,
        outcome_label_write_allowed: false,
        outcome_label_written: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "本阶段只独立复核一条通过第十九阶段校验的精确原始结果包，并最多授予 24 小时内一次未来 create-once 正式标签写入资格。复核没有写入端点，不写标签，不推断方向、动作、仓位或奖励，也不开放训练、影子、订单、券商或交易。".to_string(),
    })
}

async fn consumed_authorization_review_ids(state: &AppState) -> Result<BTreeSet<String>, String> {
    let directory = decision_root(state)
        .join("historical_outcome_formal_label_writes")
        .join("claims");
    let mut entries = match tokio::fs::read_dir(directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut consumed = BTreeSet::new();
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
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        let review_id = value
            .get("authorization_review_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                "formal label write claim is missing authorization review id".to_string()
            })?;
        let consumed_flag = value
            .get("authorization_consumed")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if !valid_id(review_id) || !consumed_flag {
            return Err("formal label write claim has invalid consumption binding".to_string());
        }
        consumed.insert(review_id.to_string());
    }
    Ok(consumed)
}

async fn review_authorization(
    state: &AppState,
    reviewer_id: &str,
    validation_id: &str,
    request: ReviewHistoricalOutcomeLabelWriteAuthorizationRequest,
) -> Result<HistoricalOutcomeLabelWriteAuthorizationReview, String> {
    if !valid_id(validation_id) {
        return Err("标签物化校验 ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-label-write-authorization-{validation_id}"),
    )
    .await?;
    let source = independently_validated_materialized_envelopes_for_label_write_review(state)
        .await?
        .into_iter()
        .find(|source| source.validation.validation_id == validation_id)
        .ok_or_else(|| "当前没有这条绑定有效且通过独立校验的物化结果包".to_string())?;
    verify_expected_bindings(&source, &request)?;
    if request.expected_label_contract_sha256 != label_contract_sha256() {
        return Err("正式标签 schema 或语义合同已经变化，请重新读取后再复核".to_string());
    }
    let previous = latest_review(state, &source).await?;
    if previous.as_ref().map(|value| value.review_id.as_str())
        != request.expected_review_id.as_deref()
        || previous.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("正式标签写入授权复核链尖已经变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = excluded_prior_actor_ids(&source);
    if !reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids) {
        return Err("复核人不能是物化校验人或此前物化、准入、校验、执行链角色".to_string());
    }
    let rationale = request.rationale.trim().to_string();
    if rationale.is_empty() || rationale.chars().count() > MAX_RATIONALE_CHARS {
        return Err("独立复核依据必填且最多 2400 字".to_string());
    }
    let envelope = source
        .attempt
        .result
        .untrusted_envelope
        .as_ref()
        .ok_or_else(|| "标签物化结果缺少未信任原始结果包".to_string())?;
    let metrics = &source.validation.independently_validated_metrics;
    let submitted_at = Utc::now();
    let mut review = HistoricalOutcomeLabelWriteAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: previous.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: previous.as_ref().map(|value| value.review_sha256.clone()),
        materialization_validation_id: source.validation.validation_id.clone(),
        materialization_validation_sha256: source.validation.validation_sha256.clone(),
        materialization_validated_at: source.validation.validated_at,
        materialization_validated_by: source.validation.validated_by.clone(),
        materialization_attempt_id: source.attempt.claim.attempt_id.clone(),
        claim_sha256: source.attempt.claim.claim_sha256.clone(),
        result_id: source.attempt.result.result_id.clone(),
        result_sha256: source.attempt.result.result_sha256.clone(),
        output_sha256: source.validation.output_sha256.clone(),
        admission_review_id: source.attempt.claim.admission_review_id.clone(),
        admission_review_sha256: source.attempt.claim.admission_review_sha256.clone(),
        source_validation_id: source.attempt.claim.validation_id.clone(),
        source_validation_sha256: source.attempt.claim.validation_sha256.clone(),
        source_attempt_id: source.attempt.claim.source_attempt_id.clone(),
        source_output_sha256: source.attempt.claim.source_output_sha256.clone(),
        snapshot_id: source.attempt.claim.snapshot_id.clone(),
        snapshot_sha256: source.attempt.claim.snapshot_sha256.clone(),
        reconstruction_id: source.attempt.claim.reconstruction_id.clone(),
        reconstruction_sha256: source.attempt.claim.reconstruction_sha256.clone(),
        protocol_version: source.attempt.claim.protocol_version.clone(),
        protocol_sha256: source.attempt.claim.protocol_sha256.clone(),
        recomputed_metrics_sha256: source.attempt.claim.recomputed_metrics_sha256.clone(),
        asset_symbol: envelope.asset_symbol.clone(),
        benchmark_symbol: envelope.benchmark_symbol.clone(),
        decision_available_at: envelope.decision_available_at,
        common_session_count: envelope.common_session_count,
        metric_horizons_market_sessions: metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect(),
        metric_start_date: metrics
            .first()
            .ok_or_else(|| "独立校验指标为空".to_string())?
            .start_date,
        metric_end_dates: metrics.iter().map(|metric| metric.end_date).collect(),
        known_limitations: envelope.known_limitations.clone(),
        formal_label_schema_version: FORMAL_LABEL_SCHEMA_VERSION.to_string(),
        formal_label_semantics_version: FORMAL_LABEL_SEMANTICS_VERSION.to_string(),
        label_contract_sha256: label_contract_sha256(),
        allowed_label_fields: allowed_label_fields(),
        submitted_at,
        authorization_valid_until: submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        exact_validated_envelope_binding_confirmed: request
            .exact_validated_envelope_binding_confirmed,
        reviewer_independence_confirmed: request.reviewer_independence_confirmed,
        formal_label_schema_confirmed: request.formal_label_schema_confirmed,
        raw_outcome_semantics_only_confirmed: request.raw_outcome_semantics_only_confirmed,
        exact_metric_bits_and_provenance_confirmed: request
            .exact_metric_bits_and_provenance_confirmed,
        known_limitations_preserved_confirmed: request.known_limitations_preserved_confirmed,
        create_once_no_overwrite_writer_confirmed: request
            .create_once_no_overwrite_writer_confirmed,
        single_use_and_expiry_confirmed: request.single_use_and_expiry_confirmed,
        label_store_isolated_from_training_confirmed: request
            .label_store_isolated_from_training_confirmed,
        no_semantic_inference_or_reward_confirmed: request
            .no_semantic_inference_or_reward_confirmed,
        no_network_tools_or_unrelated_production_access_confirmed: request
            .no_network_tools_or_unrelated_production_access_confirmed,
        no_training_shadow_order_broker_or_trading_confirmed: request
            .no_training_shadow_order_broker_or_trading_confirmed,
        reviewer_independent_from_complete_prior_chain: true,
        one_shot_label_write_limit: ONE_SHOT_LABEL_WRITE_LIMIT,
        one_shot_formal_label_write_authorized: false,
        authorization_consumed: false,
        label_writer_endpoint_available: false,
        outcome_label_write_allowed: false,
        outcome_label_written: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == HistoricalOutcomeLabelWriteAuthorizationVerdict::ApprovedForOneShotFormalLabelWrite;
    review.one_shot_formal_label_write_authorized = approved && all_checks(&review);
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &source)?;
    write_immutable_json(
        &review_directory(state, validation_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn verify_expected_bindings(
    source: &IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope,
    request: &ReviewHistoricalOutcomeLabelWriteAuthorizationRequest,
) -> Result<(), String> {
    if source.validation.validation_sha256 != request.expected_materialization_validation_sha256
        || source.attempt.claim.claim_sha256 != request.expected_claim_sha256
        || source.attempt.result.result_sha256 != request.expected_result_sha256
        || source.validation.output_sha256 != request.expected_output_sha256
        || source.attempt.claim.admission_review_sha256 != request.expected_admission_review_sha256
        || source.attempt.claim.validation_sha256 != request.expected_source_validation_sha256
        || source.attempt.claim.source_output_sha256 != request.expected_source_output_sha256
        || source.attempt.claim.snapshot_sha256 != request.expected_snapshot_sha256
        || source.attempt.claim.protocol_sha256 != request.expected_protocol_sha256
        || source.attempt.claim.recomputed_metrics_sha256
            != request.expected_recomputed_metrics_sha256
    {
        return Err("第十九阶段校验、物化结果或完整上游绑定已经变化".to_string());
    }
    Ok(())
}

fn excluded_prior_actor_ids(
    source: &IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope,
) -> Vec<String> {
    let mut actors = source.validation.excluded_prior_actor_ids.clone();
    actors.push(source.validation.validated_by.clone());
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn reviewer_is_independent(reviewer_id: &str, excluded: &[String]) -> bool {
    !reviewer_id.trim().is_empty() && !excluded.iter().any(|actor| actor == reviewer_id)
}

fn all_checks(review: &HistoricalOutcomeLabelWriteAuthorizationReview) -> bool {
    review.exact_validated_envelope_binding_confirmed
        && review.reviewer_independence_confirmed
        && review.formal_label_schema_confirmed
        && review.raw_outcome_semantics_only_confirmed
        && review.exact_metric_bits_and_provenance_confirmed
        && review.known_limitations_preserved_confirmed
        && review.create_once_no_overwrite_writer_confirmed
        && review.single_use_and_expiry_confirmed
        && review.label_store_isolated_from_training_confirmed
        && review.no_semantic_inference_or_reward_confirmed
        && review.no_network_tools_or_unrelated_production_access_confirmed
        && review.no_training_shadow_order_broker_or_trading_confirmed
}

fn validate_review(
    review: &HistoricalOutcomeLabelWriteAuthorizationReview,
    source: &IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope,
) -> Result<(), String> {
    let approved = review.verdict
        == HistoricalOutcomeLabelWriteAuthorizationVerdict::ApprovedForOneShotFormalLabelWrite;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let authority_closed = !review.authorization_consumed
        && !review.label_writer_endpoint_available
        && !review.outcome_label_write_allowed
        && !review.outcome_label_written
        && !review.decision_training_authorized
        && !review.reward_evidence_authorized
        && !review.shadow_evidence_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let envelope = source
        .attempt
        .result
        .untrusted_envelope
        .as_ref()
        .ok_or_else(|| "标签物化结果缺少未信任原始结果包".to_string())?;
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != POLICY_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.review_sha256)
        || review.review_id != review.review_sha256[..32]
        || review.review_sha256 != review_fingerprint(review)?
        || !previous_pair_valid
        || review.materialization_validation_id != source.validation.validation_id
        || review.materialization_validation_sha256 != source.validation.validation_sha256
        || review.materialization_validated_at != source.validation.validated_at
        || review.materialization_validated_by != source.validation.validated_by
        || review.materialization_attempt_id != source.attempt.claim.attempt_id
        || review.claim_sha256 != source.attempt.claim.claim_sha256
        || review.result_id != source.attempt.result.result_id
        || review.result_sha256 != source.attempt.result.result_sha256
        || review.output_sha256 != source.validation.output_sha256
        || review.admission_review_id != source.attempt.claim.admission_review_id
        || review.admission_review_sha256 != source.attempt.claim.admission_review_sha256
        || review.source_validation_id != source.attempt.claim.validation_id
        || review.source_validation_sha256 != source.attempt.claim.validation_sha256
        || review.source_attempt_id != source.attempt.claim.source_attempt_id
        || review.source_output_sha256 != source.attempt.claim.source_output_sha256
        || review.snapshot_id != source.attempt.claim.snapshot_id
        || review.snapshot_sha256 != source.attempt.claim.snapshot_sha256
        || review.reconstruction_id != source.attempt.claim.reconstruction_id
        || review.reconstruction_sha256 != source.attempt.claim.reconstruction_sha256
        || review.protocol_version != source.attempt.claim.protocol_version
        || review.protocol_sha256 != source.attempt.claim.protocol_sha256
        || review.recomputed_metrics_sha256 != source.attempt.claim.recomputed_metrics_sha256
        || review.asset_symbol != envelope.asset_symbol
        || review.benchmark_symbol != envelope.benchmark_symbol
        || review.decision_available_at != envelope.decision_available_at
        || review.common_session_count != envelope.common_session_count
        || review.metric_horizons_market_sessions != vec![20, 60, 250]
        || review.metric_end_dates.len() != 3
        || review.known_limitations != envelope.known_limitations
        || review.formal_label_schema_version != FORMAL_LABEL_SCHEMA_VERSION
        || review.formal_label_semantics_version != FORMAL_LABEL_SEMANTICS_VERSION
        || review.label_contract_sha256 != label_contract_sha256()
        || review.allowed_label_fields != allowed_label_fields()
        || review.authorization_valid_until
            != review.submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS)
        || review.reviewer_id.trim().is_empty()
        || review.excluded_prior_actor_ids != excluded_prior_actor_ids(source)
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || !reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids)
        || !review.reviewer_independent_from_complete_prior_chain
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || (approved && !all_checks(review))
        || review.one_shot_label_write_limit != ONE_SHOT_LABEL_WRITE_LIMIT
        || review.one_shot_formal_label_write_authorized != (approved && all_checks(review))
        || !authority_closed
    {
        return Err(
            "historical outcome formal label write authorization is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn authorization_unexpired(
    review: &HistoricalOutcomeLabelWriteAuthorizationReview,
    now: DateTime<Utc>,
) -> bool {
    review.one_shot_formal_label_write_authorized
        && !review.authorization_consumed
        && now >= review.submitted_at
        && now < review.authorization_valid_until
}

pub(crate) fn label_contract_sha256() -> String {
    let value = serde_json::json!({
        "schema_version": FORMAL_LABEL_SCHEMA_VERSION,
        "semantics_version": FORMAL_LABEL_SEMANTICS_VERSION,
        "allowed_fields": ALLOWED_LABEL_FIELDS,
        "direction_inferred": false,
        "action_inferred": false,
        "position_inferred": false,
        "reward_inferred": false,
    });
    hash_serializable(&value).expect("static formal label contract is serializable")
}

pub(crate) fn allowed_label_fields() -> Vec<String> {
    ALLOWED_LABEL_FIELDS
        .iter()
        .map(|value| value.to_string())
        .collect()
}

fn review_fingerprint(
    review: &HistoricalOutcomeLabelWriteAuthorizationReview,
) -> Result<String, String> {
    let mut value = serde_json::to_value(review).map_err(|error| error.to_string())?;
    let object = value.as_object_mut().ok_or_else(|| {
        "formal label write authorization fingerprint payload is invalid".to_string()
    })?;
    object.remove("review_id");
    object.remove("review_sha256");
    hash_serializable(&value)
}

async fn latest_review(
    state: &AppState,
    source: &IndependentlyValidatedHistoricalOutcomeLabelMaterializationEnvelope,
) -> Result<Option<HistoricalOutcomeLabelWriteAuthorizationReview>, String> {
    let directory = review_directory(state, &source.validation.validation_id);
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
        let review: HistoricalOutcomeLabelWriteAuthorizationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, source)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err("historical outcome formal label write authorization contains duplicate or filename mismatch".to_string());
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeLabelWriteAuthorizationReview],
) -> Result<Option<HistoricalOutcomeLabelWriteAuthorizationReview>, String> {
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
        return Err(
            "historical outcome formal label write authorization chain has invalid tips"
                .to_string(),
        );
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err(
                "historical outcome formal label write authorization chain has a cycle".to_string(),
            );
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(previous_id), Some(previous_sha256)) => {
                let previous = by_id.get(previous_id.as_str()).ok_or_else(|| {
                    "historical outcome formal label write authorization chain is disconnected"
                        .to_string()
                })?;
                if previous.review_sha256 != *previous_sha256 {
                    return Err("historical outcome formal label write authorization previous hash mismatch".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "historical outcome formal label write authorization previous link is invalid"
                        .to_string(),
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err(
            "historical outcome formal label write authorization chain is disconnected".to_string(),
        );
    }
    Ok(Some(tips[0].clone()))
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_directory(state: &AppState, validation_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical_outcome_label_write_authorizations")
        .join(validation_id)
}

async fn acquire_lock(state: &AppState, name: &str) -> Result<DirectoryLock, String> {
    let root = decision_root(state).join("locks");
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|error| error.to_string())?;
    let path = root.join(name);
    match tokio::fs::create_dir(&path).await {
        Ok(_) => Ok(DirectoryLock { path }),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err("另一条正式标签写入授权复核正在提交，请稍后重试".to_string())
        }
        Err(error) => Err(error.to_string()),
    }
}

struct DirectoryLock {
    path: PathBuf,
}

impl Drop for DirectoryLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.path);
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
            .open(&path)
            .map_err(|error| error.to_string())?;
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_label_contract_is_fixed_and_raw_outcome_only() {
        assert!(valid_sha256(&label_contract_sha256()));
        assert_eq!(
            allowed_label_fields(),
            ALLOWED_LABEL_FIELDS
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
        );
        let fields = allowed_label_fields().join(" ");
        assert!(!fields.contains("action"));
        assert!(!fields.contains("reward"));
        assert!(!fields.contains("position"));
    }

    #[test]
    fn reviewer_must_be_outside_the_complete_prior_chain() {
        let excluded = vec!["admission-admin".to_string(), "validator-admin".to_string()];
        assert!(!reviewer_is_independent("validator-admin", &excluded));
        assert!(reviewer_is_independent("fresh-admin", &excluded));
        assert!(!reviewer_is_independent("", &excluded));
    }

    #[test]
    fn authorization_is_one_shot_and_expires_at_exact_boundary() {
        let submitted_at = Utc::now();
        let review = minimal_review(submitted_at);
        assert!(authorization_unexpired(&review, submitted_at));
        assert!(authorization_unexpired(
            &review,
            submitted_at + TimeDelta::hours(23)
        ));
        assert!(!authorization_unexpired(
            &review,
            submitted_at + TimeDelta::hours(24)
        ));
    }

    #[test]
    fn approval_requires_every_explicit_check() {
        let mut review = minimal_review(Utc::now());
        assert!(all_checks(&review));
        review.label_store_isolated_from_training_confirmed = false;
        assert!(!all_checks(&review));
    }

    #[test]
    fn authorization_record_keeps_writer_and_downstream_authority_closed() {
        let review = minimal_review(Utc::now());
        assert!(review.one_shot_formal_label_write_authorized);
        assert!(!review.label_writer_endpoint_available);
        assert!(!review.outcome_label_write_allowed);
        assert!(!review.outcome_label_written);
        assert!(!review.decision_training_authorized);
        assert!(!review.reward_evidence_authorized);
        assert!(!review.shadow_evidence_authorized);
        assert!(!review.order_generation_authorized);
        assert!(!review.broker_access_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn actor_sets_must_be_sorted_and_unique() {
        assert!(sorted_unique(&["a".to_string(), "b".to_string()]));
        assert!(!sorted_unique(&["b".to_string(), "a".to_string()]));
        assert!(!sorted_unique(&["a".to_string(), "a".to_string()]));
    }

    #[test]
    fn review_fingerprint_binds_the_label_contract_and_every_check() {
        let review = minimal_review(Utc::now());
        let original = review_fingerprint(&review).unwrap();
        let mut changed_contract = review.clone();
        changed_contract.formal_label_semantics_version = "semantic-drift".to_string();
        assert_ne!(original, review_fingerprint(&changed_contract).unwrap());
        let mut missing_check = review;
        missing_check.no_semantic_inference_or_reward_confirmed = false;
        assert_ne!(original, review_fingerprint(&missing_check).unwrap());
    }

    #[test]
    fn review_chain_requires_exact_previous_hash_and_one_tip() {
        let mut first = minimal_review(Utc::now());
        first.review_id = "a".repeat(32);
        first.review_sha256 = "a".repeat(64);
        let mut second = minimal_review(Utc::now());
        second.review_id = "b".repeat(32);
        second.review_sha256 = "b".repeat(64);
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some(first.review_sha256.clone());
        assert_eq!(
            latest_review_from_records(&[first.clone(), second.clone()])
                .unwrap()
                .unwrap()
                .review_id,
            second.review_id
        );

        let mut wrong_hash = second.clone();
        wrong_hash.previous_review_sha256 = Some("c".repeat(64));
        assert!(latest_review_from_records(&[first.clone(), wrong_hash]).is_err());

        let mut branch = second;
        branch.review_id = "d".repeat(32);
        branch.review_sha256 = "d".repeat(64);
        assert!(latest_review_from_records(&[first, branch.clone(), branch]).is_err());
    }

    fn minimal_review(
        submitted_at: DateTime<Utc>,
    ) -> HistoricalOutcomeLabelWriteAuthorizationReview {
        HistoricalOutcomeLabelWriteAuthorizationReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_id: "a".repeat(32),
            review_sha256: "a".repeat(64),
            previous_review_id: None,
            previous_review_sha256: None,
            materialization_validation_id: "b".repeat(32),
            materialization_validation_sha256: "b".repeat(64),
            materialization_validated_at: submitted_at,
            materialization_validated_by: "validator".to_string(),
            materialization_attempt_id: "c".repeat(32),
            claim_sha256: "c".repeat(64),
            result_id: "d".repeat(32),
            result_sha256: "d".repeat(64),
            output_sha256: "e".repeat(64),
            admission_review_id: "f".repeat(32),
            admission_review_sha256: "f".repeat(64),
            source_validation_id: "1".repeat(32),
            source_validation_sha256: "1".repeat(64),
            source_attempt_id: "2".repeat(32),
            source_output_sha256: "2".repeat(64),
            snapshot_id: "3".repeat(32),
            snapshot_sha256: "3".repeat(64),
            reconstruction_id: "4".repeat(32),
            reconstruction_sha256: "4".repeat(64),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "5".repeat(64),
            recomputed_metrics_sha256: "6".repeat(64),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            decision_available_at: submitted_at,
            common_session_count: 251,
            metric_horizons_market_sessions: vec![20, 60, 250],
            metric_start_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            metric_end_dates: vec![
                NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
                NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
            ],
            known_limitations: "raw market outcomes only".to_string(),
            formal_label_schema_version: FORMAL_LABEL_SCHEMA_VERSION.to_string(),
            formal_label_semantics_version: FORMAL_LABEL_SEMANTICS_VERSION.to_string(),
            label_contract_sha256: label_contract_sha256(),
            allowed_label_fields: allowed_label_fields(),
            submitted_at,
            authorization_valid_until: submitted_at + TimeDelta::hours(24),
            reviewer_id: "fresh-admin".to_string(),
            excluded_prior_actor_ids: vec!["validator".to_string()],
            verdict:
                HistoricalOutcomeLabelWriteAuthorizationVerdict::ApprovedForOneShotFormalLabelWrite,
            rationale: "exact raw outcome label contract reviewed".to_string(),
            exact_validated_envelope_binding_confirmed: true,
            reviewer_independence_confirmed: true,
            formal_label_schema_confirmed: true,
            raw_outcome_semantics_only_confirmed: true,
            exact_metric_bits_and_provenance_confirmed: true,
            known_limitations_preserved_confirmed: true,
            create_once_no_overwrite_writer_confirmed: true,
            single_use_and_expiry_confirmed: true,
            label_store_isolated_from_training_confirmed: true,
            no_semantic_inference_or_reward_confirmed: true,
            no_network_tools_or_unrelated_production_access_confirmed: true,
            no_training_shadow_order_broker_or_trading_confirmed: true,
            reviewer_independent_from_complete_prior_chain: true,
            one_shot_label_write_limit: 1,
            one_shot_formal_label_write_authorized: true,
            authorization_consumed: false,
            label_writer_endpoint_available: false,
            outcome_label_write_allowed: false,
            outcome_label_written: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        }
    }
}
