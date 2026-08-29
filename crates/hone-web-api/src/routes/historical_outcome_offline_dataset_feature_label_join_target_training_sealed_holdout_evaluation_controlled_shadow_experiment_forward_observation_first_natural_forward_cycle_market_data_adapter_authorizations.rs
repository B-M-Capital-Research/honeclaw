//! Stage 92 independent authorization of a bounded read-only market-data adapter contract.
//!
//! Approval only freezes which HTTPS sources, methods, data classes and custody proofs a future
//! claim-first receipt may use. It does not make a request, resolve a calendar window, expose an
//! execution endpoint, instantiate a runtime, start observation, create a ledger/position/
//! performance record, or grant model, reward, order, broker or trade authority.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_claims::{
    ControlledShadowFirstNaturalForwardCycleClaim,
    claimed_first_natural_forward_cycles_for_market_data_adapter_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-controlled-shadow-first-natural-forward-cycle-market-data-adapter-authorization-registry-v2";
const AUTHORIZATION_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-first-natural-forward-cycle-market-data-adapter-authorization-v2";
const ADAPTER_SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-read-only-market-data-adapter-spec-v2";
const POLICY_VERSION: &str = "hone-controlled-shadow-market-data-adapter-contract-review-v2-explicit-corporate-actions-no-data-access";
const ADAPTER_ID: &str = "hone-forward-observation-market-data-adapter-v2-explicit-actions";
const MAX_TEXT_CHARS: usize = 4_000;
const AUTHORIZATION_TTL_HOURS: i64 = 7 * 24;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowMarketDataAdapterAuthorizationVerdict {
    ApprovedForFutureClaimFirstReadOnlyMarketDataReceipt,
    RejectedMarketDataAdapterContract,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowMarketDataAdapterAuthorizationRequest {
    expected_cycle_claim_sha256: String,
    expected_authorization_review_sha256: String,
    expected_validation_sha256: String,
    expected_initialization_manifest_sha256: String,
    verdict: ControlledShadowMarketDataAdapterAuthorizationVerdict,
    rationale: String,
    source_allowlist_assessment: String,
    credential_and_request_minimization_assessment: String,
    content_addressing_and_custody_assessment: String,
    known_limitations: String,
    future_receipt_constraints: String,
    exact_stage_51_through_stage_91_binding_confirmed: bool,
    reviewer_independent_from_claimant_and_complete_prior_chain_confirmed: bool,
    fixed_get_only_https_origin_and_path_allowlist_confirmed: bool,
    calendar_security_spy_price_dividend_split_only_confirmed: bool,
    exact_future_symbol_set_and_time_window_must_be_content_addressed_confirmed: bool,
    credentials_never_persisted_forwarded_or_returned_confirmed: bool,
    request_response_source_and_retrieval_time_hashes_required_confirmed: bool,
    natural_forward_only_no_backfill_or_history_rewrite_confirmed: bool,
    approval_only_opens_future_claim_first_read_only_receipt_confirmed: bool,
    no_data_request_calendar_resolution_or_runtime_started_confirmed: bool,
    no_observation_ledger_position_performance_or_model_metric_write_confirmed: bool,
    no_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowReadOnlyMarketDataAdapterSpec {
    pub schema_version: String,
    pub adapter_id: String,
    pub adapter_spec_sha256: String,
    pub allowed_http_methods: Vec<String>,
    pub allowed_https_origin_and_path_prefixes: Vec<String>,
    pub allowed_query_parameter_names: Vec<String>,
    pub credential_query_parameter_name: String,
    pub credential_redaction_required: bool,
    pub credential_excluded_from_canonical_request_sha256: bool,
    pub allowed_data_classes: Vec<String>,
    pub benchmark_symbol: String,
    pub exact_future_subject_symbol_set_content_hash_required: bool,
    pub exact_future_time_window_content_hash_required: bool,
    pub request_sha256_required: bool,
    pub response_body_sha256_required: bool,
    pub source_document_sha256_required: bool,
    pub retrieved_at_utc_required: bool,
    pub source_available_at_utc_required: bool,
    pub raw_payload_retention_required: bool,
    pub append_only_correction_required: bool,
    pub credentials_may_be_persisted: bool,
    pub credentials_may_be_returned: bool,
    pub redirects_allowed: bool,
    pub non_https_allowed: bool,
    pub arbitrary_url_allowed: bool,
    pub arbitrary_symbol_allowed: bool,
    pub retroactive_backfill_allowed: bool,
    pub maximum_response_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataAdapterAuthorization {
    pub schema_version: String,
    pub policy_version: String,
    pub adapter_authorization_id: String,
    pub adapter_authorization_sha256: String,
    pub cycle_claim_id: String,
    pub cycle_claim_sha256: String,
    pub upstream_authorization_review_sha256: String,
    pub validation_sha256: String,
    pub initialization_manifest_sha256: String,
    pub adapter_specification: ControlledShadowReadOnlyMarketDataAdapterSpec,
    pub submitted_at: DateTime<Utc>,
    pub authorized_not_before: DateTime<Utc>,
    pub authorized_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub reviewer_independent_from_claimant_and_complete_prior_chain: bool,
    pub verdict: ControlledShadowMarketDataAdapterAuthorizationVerdict,
    pub rationale: String,
    pub source_allowlist_assessment: String,
    pub credential_and_request_minimization_assessment: String,
    pub content_addressing_and_custody_assessment: String,
    pub known_limitations: String,
    pub future_receipt_constraints: String,
    pub exact_stage_51_through_stage_91_binding_confirmed: bool,
    pub reviewer_independent_from_claimant_and_complete_prior_chain_confirmed: bool,
    pub fixed_get_only_https_origin_and_path_allowlist_confirmed: bool,
    pub calendar_security_spy_price_dividend_split_only_confirmed: bool,
    pub exact_future_symbol_set_and_time_window_must_be_content_addressed_confirmed: bool,
    pub credentials_never_persisted_forwarded_or_returned_confirmed: bool,
    pub request_response_source_and_retrieval_time_hashes_required_confirmed: bool,
    pub natural_forward_only_no_backfill_or_history_rewrite_confirmed: bool,
    pub approval_only_opens_future_claim_first_read_only_receipt_confirmed: bool,
    pub no_data_request_calendar_resolution_or_runtime_started_confirmed: bool,
    pub no_observation_ledger_position_performance_or_model_metric_write_confirmed: bool,
    pub no_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub adapter_contract_authorized: bool,
    pub future_claim_first_read_only_market_data_receipt_eligible: bool,
    pub market_data_request_made: bool,
    pub calendar_window_resolved: bool,
    pub market_data_accessed: bool,
    pub runtime_instantiated: bool,
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
pub(crate) struct ControlledShadowMarketDataAdapterAuthorizationItem {
    pub claim: ControlledShadowFirstNaturalForwardCycleClaim,
    pub authorization: Option<ControlledShadowMarketDataAdapterAuthorization>,
    pub review_eligible: bool,
    pub adapter_contract_authorized: bool,
    pub future_claim_first_read_only_market_data_receipt_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataAdapterAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub adapter_specification: ControlledShadowReadOnlyMarketDataAdapterSpec,
    pub items: Vec<ControlledShadowMarketDataAdapterAuthorizationItem>,
    pub claimed_task_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub rejected_count: usize,
    pub active_authorization_count: usize,
    pub future_claim_first_read_only_market_data_receipt_eligible_count: usize,
    pub authorization_status: String,
    pub market_data_request_made: bool,
    pub calendar_window_resolved: bool,
    pub market_data_accessed: bool,
    pub runtime_instantiated: bool,
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
pub(crate) struct ControlledShadowMarketDataAdapterAuthorizationReadinessSummary {
    pub claimed_task_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub rejected_count: usize,
    pub active_authorization_count: usize,
    pub future_claim_first_read_only_market_data_receipt_eligible_count: usize,
    pub authorization_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ApprovedControlledShadowMarketDataAdapterAuthorization {
    pub claim: ControlledShadowFirstNaturalForwardCycleClaim,
    pub authorization: ControlledShadowMarketDataAdapterAuthorization,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_adapter_authorizations(
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
            warn!(%error, "market-data adapter authorization registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "首个自然前向周期只读行情适配器授权表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_market_data_adapter_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(cycle_claim_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowMarketDataAdapterAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &cycle_claim_id, request, Utc::now()).await {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_adapter_authorization_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataAdapterAuthorizationReadinessSummary, String> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowMarketDataAdapterAuthorizationReadinessSummary {
            claimed_task_count: value.claimed_task_count,
            review_eligible_count: value.review_eligible_count,
            reviewed_count: value.reviewed_count,
            approved_count: value.approved_count,
            rejected_count: value.rejected_count,
            active_authorization_count: value.active_authorization_count,
            future_claim_first_read_only_market_data_receipt_eligible_count: value
                .future_claim_first_read_only_market_data_receipt_eligible_count,
            authorization_status: value.authorization_status,
        },
    )
}

#[allow(dead_code)] // Deliberate Stage 93 hand-off; Stage 92 must not consume the authorization.
pub(crate) async fn approved_market_data_adapter_authorizations_for_future_receipt(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Vec<ApprovedControlledShadowMarketDataAdapterAuthorization>, String> {
    let claims = claimed_first_natural_forward_cycles_for_market_data_adapter_review(state).await?;
    let authorizations = read_authorizations(state).await?;
    let consumed = super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_attempts::claimed_market_data_receipt_adapter_authorization_ids(state).await?;
    let mut approved = Vec::new();
    for authorization in authorizations {
        if !authorization_is_active(&authorization, now)
            || consumed.contains(&authorization.adapter_authorization_id)
        {
            continue;
        }
        let claim = claims
            .iter()
            .find(|value| {
                value.cycle_claim_id == authorization.cycle_claim_id
                    && value.cycle_claim_sha256 == authorization.cycle_claim_sha256
            })
            .ok_or_else(|| "Stage 92 授权引用的 Stage 91 claim 不存在或已漂移".to_string())?;
        approved.push(ApprovedControlledShadowMarketDataAdapterAuthorization {
            claim: claim.clone(),
            authorization,
        });
    }
    Ok(approved)
}

pub(crate) async fn exact_market_data_adapter_authorization_for_receipt_audit(
    state: &AppState,
    adapter_authorization_id: &str,
    adapter_authorization_sha256: &str,
) -> Result<Option<ApprovedControlledShadowMarketDataAdapterAuthorization>, String> {
    let authorization = read_authorizations(state).await?.into_iter().find(|value| {
        value.adapter_authorization_id == adapter_authorization_id
            && value.adapter_authorization_sha256 == adapter_authorization_sha256
            && value.adapter_contract_authorized
    });
    let Some(authorization) = authorization else {
        return Ok(None);
    };
    let claim = claimed_first_natural_forward_cycles_for_market_data_adapter_review(state)
        .await?
        .into_iter()
        .find(|value| {
            value.cycle_claim_id == authorization.cycle_claim_id
                && value.cycle_claim_sha256 == authorization.cycle_claim_sha256
        })
        .ok_or_else(|| "Stage 94 无法重开 Stage 92 授权绑定的 Stage 91 claim".to_string())?;
    Ok(Some(
        ApprovedControlledShadowMarketDataAdapterAuthorization {
            claim,
            authorization,
        },
    ))
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataAdapterAuthorizationRegistry, String> {
    let claims = claimed_first_natural_forward_cycles_for_market_data_adapter_review(state).await?;
    let authorizations = read_authorizations(state).await?;
    let consumed = super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_attempts::claimed_market_data_receipt_adapter_authorization_ids(state).await?;
    let mut items = Vec::new();
    for claim in claims {
        let authorization = authorizations
            .iter()
            .find(|value| value.cycle_claim_sha256 == claim.cycle_claim_sha256)
            .cloned();
        let active = authorization.as_ref().is_some_and(|value| {
            authorization_is_active(value, now)
                && !consumed.contains(&value.adapter_authorization_id)
        });
        items.push(ControlledShadowMarketDataAdapterAuthorizationItem {
            review_eligible: authorization.is_none(),
            adapter_contract_authorized: authorization
                .as_ref()
                .is_some_and(|value| value.adapter_contract_authorized),
            future_claim_first_read_only_market_data_receipt_eligible: active,
            claim,
            authorization,
        });
    }
    let reviewed_count = authorizations.len();
    let approved_count = authorizations
        .iter()
        .filter(|value| value.adapter_contract_authorized)
        .count();
    let rejected_count = reviewed_count.saturating_sub(approved_count);
    let active_authorization_count = authorizations
        .iter()
        .filter(|value| {
            authorization_is_active(value, now)
                && !consumed.contains(&value.adapter_authorization_id)
        })
        .count();
    let review_eligible_count = items.iter().filter(|value| value.review_eligible).count();
    let authorization_status = if active_authorization_count > 0 {
        "adapter_contract_approved_waiting_for_future_claim_first_read_only_receipt"
    } else if approved_count > 0 {
        "adapter_contract_approval_expired_without_data_access"
    } else if rejected_count > 0 {
        "adapter_contract_rejected_no_data_access"
    } else if review_eligible_count > 0 {
        "waiting_for_chain_external_read_only_adapter_contract_review"
    } else {
        "waiting_for_stage_91_claim"
    };
    Ok(ControlledShadowMarketDataAdapterAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        adapter_specification: canonical_adapter_specification()?,
        claimed_task_count: items.len(),
        review_eligible_count,
        reviewed_count,
        approved_count,
        rejected_count,
        active_authorization_count,
        future_claim_first_read_only_market_data_receipt_eligible_count:
            active_authorization_count,
        items,
        authorization_status: authorization_status.to_string(),
        market_data_request_made: false,
        calendar_window_resolved: false,
        market_data_accessed: false,
        runtime_instantiated: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 92 只允许责任链外管理员复核固定只读行情适配器合同。批准仅在 7 天内开放未来 claim-first、create-once 数据收据资格，以覆盖周末和休市日；批准本身不解析日历、不发起 HTTP 请求、不读取行情、不实例化 runtime、不开始观察、不建账、不写持仓或绩效，也不开放模型/指标、训练、reward、订单、券商或交易能力。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    cycle_claim_id: &str,
    request: ReviewControlledShadowMarketDataAdapterAuthorizationRequest,
    now: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataAdapterAuthorization, String> {
    if !valid_id(cycle_claim_id) {
        return Err("Stage 91 cycle claim ID 无效".to_string());
    }
    let _lock = acquire_lock(state, cycle_claim_id).await?;
    let existing = read_authorizations(state).await?;
    if existing
        .iter()
        .any(|value| value.cycle_claim_id == cycle_claim_id)
    {
        return Err("该 Stage 91 claim 已完成不可覆盖的 Stage 92 复核".to_string());
    }
    let claim = claimed_first_natural_forward_cycles_for_market_data_adapter_review(state)
        .await?
        .into_iter()
        .find(|value| value.cycle_claim_id == cycle_claim_id)
        .ok_or_else(|| "Stage 91 claim 不存在".to_string())?;
    validate_expected_binding(&claim, &request)?;
    let authorization = new_authorization(&claim, reviewer_id, request, now)?;
    write_immutable_json(
        &authorization_directory(state)
            .join(format!("{}.json", authorization.adapter_authorization_id)),
        &authorization,
    )
    .await?;
    Ok(authorization)
}

fn validate_expected_binding(
    claim: &ControlledShadowFirstNaturalForwardCycleClaim,
    request: &ReviewControlledShadowMarketDataAdapterAuthorizationRequest,
) -> Result<(), String> {
    if request.expected_cycle_claim_sha256 == claim.cycle_claim_sha256
        && request.expected_authorization_review_sha256 == claim.authorization_review_sha256
        && request.expected_validation_sha256 == claim.validation_sha256
        && request.expected_initialization_manifest_sha256 == claim.initialization_manifest_sha256
    {
        Ok(())
    } else {
        Err("Stage 92 预期绑定与当前 Stage 91 claim 不一致".to_string())
    }
}

fn new_authorization(
    claim: &ControlledShadowFirstNaturalForwardCycleClaim,
    reviewer_id: &str,
    request: ReviewControlledShadowMarketDataAdapterAuthorizationRequest,
    now: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataAdapterAuthorization, String> {
    let mut excluded_prior_actor_ids = claim.excluded_prior_actor_ids.clone();
    excluded_prior_actor_ids.push(claim.claimed_by.clone());
    excluded_prior_actor_ids.sort();
    excluded_prior_actor_ids.dedup();
    let independent = !reviewer_id.trim().is_empty()
        && excluded_prior_actor_ids
            .iter()
            .all(|value| value != reviewer_id);
    let approved = matches!(
        request.verdict,
        ControlledShadowMarketDataAdapterAuthorizationVerdict::ApprovedForFutureClaimFirstReadOnlyMarketDataReceipt
    );
    let mut authorization = ControlledShadowMarketDataAdapterAuthorization {
        schema_version: AUTHORIZATION_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        adapter_authorization_id: String::new(),
        adapter_authorization_sha256: String::new(),
        cycle_claim_id: claim.cycle_claim_id.clone(),
        cycle_claim_sha256: claim.cycle_claim_sha256.clone(),
        upstream_authorization_review_sha256: claim.authorization_review_sha256.clone(),
        validation_sha256: claim.validation_sha256.clone(),
        initialization_manifest_sha256: claim.initialization_manifest_sha256.clone(),
        adapter_specification: canonical_adapter_specification()?,
        submitted_at: now,
        authorized_not_before: now,
        authorized_valid_until: now + Duration::hours(AUTHORIZATION_TTL_HOURS),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        reviewer_independent_from_claimant_and_complete_prior_chain: independent
            && request.reviewer_independent_from_claimant_and_complete_prior_chain_confirmed,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        source_allowlist_assessment: bounded_required(
            &request.source_allowlist_assessment,
            "来源白名单评估",
        )?,
        credential_and_request_minimization_assessment: bounded_required(
            &request.credential_and_request_minimization_assessment,
            "凭据与请求最小化评估",
        )?,
        content_addressing_and_custody_assessment: bounded_required(
            &request.content_addressing_and_custody_assessment,
            "内容寻址与证据保管评估",
        )?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_receipt_constraints: bounded_required(
            &request.future_receipt_constraints,
            "未来收据约束",
        )?,
        exact_stage_51_through_stage_91_binding_confirmed: request
            .exact_stage_51_through_stage_91_binding_confirmed,
        reviewer_independent_from_claimant_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_claimant_and_complete_prior_chain_confirmed,
        fixed_get_only_https_origin_and_path_allowlist_confirmed: request
            .fixed_get_only_https_origin_and_path_allowlist_confirmed,
        calendar_security_spy_price_dividend_split_only_confirmed: request
            .calendar_security_spy_price_dividend_split_only_confirmed,
        exact_future_symbol_set_and_time_window_must_be_content_addressed_confirmed: request
            .exact_future_symbol_set_and_time_window_must_be_content_addressed_confirmed,
        credentials_never_persisted_forwarded_or_returned_confirmed: request
            .credentials_never_persisted_forwarded_or_returned_confirmed,
        request_response_source_and_retrieval_time_hashes_required_confirmed: request
            .request_response_source_and_retrieval_time_hashes_required_confirmed,
        natural_forward_only_no_backfill_or_history_rewrite_confirmed: request
            .natural_forward_only_no_backfill_or_history_rewrite_confirmed,
        approval_only_opens_future_claim_first_read_only_receipt_confirmed: request
            .approval_only_opens_future_claim_first_read_only_receipt_confirmed,
        no_data_request_calendar_resolution_or_runtime_started_confirmed: request
            .no_data_request_calendar_resolution_or_runtime_started_confirmed,
        no_observation_ledger_position_performance_or_model_metric_write_confirmed: request
            .no_observation_ledger_position_performance_or_model_metric_write_confirmed,
        no_training_feedback_reward_order_broker_or_trading_confirmed: request
            .no_training_feedback_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        adapter_contract_authorized: approved,
        future_claim_first_read_only_market_data_receipt_eligible: approved,
        market_data_request_made: false,
        calendar_window_resolved: false,
        market_data_accessed: false,
        runtime_instantiated: false,
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
    authorization.adapter_authorization_sha256 = fingerprint_without(
        &authorization,
        &["adapter_authorization_id", "adapter_authorization_sha256"],
    )?;
    authorization.adapter_authorization_id =
        authorization.adapter_authorization_sha256[..32].to_string();
    validate_authorization(&authorization)?;
    if !independent || !review_checks(&authorization) {
        return Err("Stage 92 复核职责不独立或确认项不完整".to_string());
    }
    Ok(authorization)
}

fn canonical_adapter_specification() -> Result<ControlledShadowReadOnlyMarketDataAdapterSpec, String>
{
    let mut specification = ControlledShadowReadOnlyMarketDataAdapterSpec {
        schema_version: ADAPTER_SPEC_SCHEMA_VERSION.to_string(),
        adapter_id: ADAPTER_ID.to_string(),
        adapter_spec_sha256: String::new(),
        allowed_http_methods: vec!["GET".to_string()],
        allowed_https_origin_and_path_prefixes: vec![
            "https://financialmodelingprep.com/stable/dividends".to_string(),
            "https://financialmodelingprep.com/stable/historical-price-eod/dividend-adjusted"
                .to_string(),
            "https://financialmodelingprep.com/stable/historical-price-eod/full".to_string(),
            "https://financialmodelingprep.com/stable/historical-price-eod/non-split-adjusted"
                .to_string(),
            "https://financialmodelingprep.com/stable/splits".to_string(),
            "https://www.nyse.com/markets/hours-calendars".to_string(),
        ],
        allowed_query_parameter_names: vec![
            "apikey".to_string(),
            "from".to_string(),
            "symbol".to_string(),
            "to".to_string(),
        ],
        credential_query_parameter_name: "apikey".to_string(),
        credential_redaction_required: true,
        credential_excluded_from_canonical_request_sha256: true,
        allowed_data_classes: vec![
            "dividend_adjusted_close".to_string(),
            "dividend_event".to_string(),
            "official_us_market_calendar".to_string(),
            "ohlcv".to_string(),
            "raw_unadjusted_close".to_string(),
            "split_adjusted_close".to_string(),
            "split_event".to_string(),
        ],
        benchmark_symbol: "SPY".to_string(),
        exact_future_subject_symbol_set_content_hash_required: true,
        exact_future_time_window_content_hash_required: true,
        request_sha256_required: true,
        response_body_sha256_required: true,
        source_document_sha256_required: true,
        retrieved_at_utc_required: true,
        source_available_at_utc_required: true,
        raw_payload_retention_required: true,
        append_only_correction_required: true,
        credentials_may_be_persisted: false,
        credentials_may_be_returned: false,
        redirects_allowed: false,
        non_https_allowed: false,
        arbitrary_url_allowed: false,
        arbitrary_symbol_allowed: false,
        retroactive_backfill_allowed: false,
        maximum_response_bytes: 16 * 1024 * 1024,
    };
    specification.adapter_spec_sha256 =
        fingerprint_without(&specification, &["adapter_spec_sha256"])?;
    Ok(specification)
}

fn review_checks(value: &ControlledShadowMarketDataAdapterAuthorization) -> bool {
    value.exact_stage_51_through_stage_91_binding_confirmed
        && value.reviewer_independent_from_claimant_and_complete_prior_chain_confirmed
        && value.fixed_get_only_https_origin_and_path_allowlist_confirmed
        && value.calendar_security_spy_price_dividend_split_only_confirmed
        && value.exact_future_symbol_set_and_time_window_must_be_content_addressed_confirmed
        && value.credentials_never_persisted_forwarded_or_returned_confirmed
        && value.request_response_source_and_retrieval_time_hashes_required_confirmed
        && value.natural_forward_only_no_backfill_or_history_rewrite_confirmed
        && value.approval_only_opens_future_claim_first_read_only_receipt_confirmed
        && value.no_data_request_calendar_resolution_or_runtime_started_confirmed
        && value.no_observation_ledger_position_performance_or_model_metric_write_confirmed
        && value.no_training_feedback_reward_order_broker_or_trading_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn authorization_is_active(
    value: &ControlledShadowMarketDataAdapterAuthorization,
    now: DateTime<Utc>,
) -> bool {
    value.adapter_contract_authorized
        && value.future_claim_first_read_only_market_data_receipt_eligible
        && value.authorized_not_before <= now
        && now < value.authorized_valid_until
}

fn validate_adapter_specification(
    value: &ControlledShadowReadOnlyMarketDataAdapterSpec,
) -> Result<(), String> {
    if value.schema_version != ADAPTER_SPEC_SCHEMA_VERSION
        || value.adapter_id != ADAPTER_ID
        || !valid_sha256(&value.adapter_spec_sha256)
        || value.adapter_spec_sha256 != fingerprint_without(value, &["adapter_spec_sha256"])?
        || value.allowed_http_methods != ["GET"]
        || value.allowed_https_origin_and_path_prefixes
            != [
                "https://financialmodelingprep.com/stable/dividends",
                "https://financialmodelingprep.com/stable/historical-price-eod/dividend-adjusted",
                "https://financialmodelingprep.com/stable/historical-price-eod/full",
                "https://financialmodelingprep.com/stable/historical-price-eod/non-split-adjusted",
                "https://financialmodelingprep.com/stable/splits",
                "https://www.nyse.com/markets/hours-calendars",
            ]
        || value.allowed_query_parameter_names != ["apikey", "from", "symbol", "to"]
        || value.credential_query_parameter_name != "apikey"
        || !value.credential_redaction_required
        || !value.credential_excluded_from_canonical_request_sha256
        || value.allowed_data_classes
            != [
                "dividend_adjusted_close",
                "dividend_event",
                "official_us_market_calendar",
                "ohlcv",
                "raw_unadjusted_close",
                "split_adjusted_close",
                "split_event",
            ]
        || value.benchmark_symbol != "SPY"
        || !value.exact_future_subject_symbol_set_content_hash_required
        || !value.exact_future_time_window_content_hash_required
        || !value.request_sha256_required
        || !value.response_body_sha256_required
        || !value.source_document_sha256_required
        || !value.retrieved_at_utc_required
        || !value.source_available_at_utc_required
        || !value.raw_payload_retention_required
        || !value.append_only_correction_required
        || value.credentials_may_be_persisted
        || value.credentials_may_be_returned
        || value.redirects_allowed
        || value.non_https_allowed
        || value.arbitrary_url_allowed
        || value.arbitrary_symbol_allowed
        || value.retroactive_backfill_allowed
        || value.maximum_response_bytes != 16 * 1024 * 1024
    {
        return Err("Stage 92 固定只读行情适配器规格无效或漂移".to_string());
    }
    Ok(())
}

fn validate_authorization(
    value: &ControlledShadowMarketDataAdapterAuthorization,
) -> Result<(), String> {
    validate_adapter_specification(&value.adapter_specification)?;
    let authority_closed = !value.market_data_request_made
        && !value.calendar_window_resolved
        && !value.market_data_accessed
        && !value.runtime_instantiated
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
    let approved = matches!(
        value.verdict,
        ControlledShadowMarketDataAdapterAuthorizationVerdict::ApprovedForFutureClaimFirstReadOnlyMarketDataReceipt
    );
    if value.schema_version != AUTHORIZATION_SCHEMA_VERSION
        || value.policy_version != POLICY_VERSION
        || !valid_id(&value.adapter_authorization_id)
        || !valid_sha256(&value.adapter_authorization_sha256)
        || value.adapter_authorization_id != value.adapter_authorization_sha256[..32]
        || value.adapter_authorization_sha256
            != fingerprint_without(
                value,
                &["adapter_authorization_id", "adapter_authorization_sha256"],
            )?
        || !valid_id(&value.cycle_claim_id)
        || !valid_sha256(&value.cycle_claim_sha256)
        || !valid_sha256(&value.upstream_authorization_review_sha256)
        || !valid_sha256(&value.validation_sha256)
        || !valid_sha256(&value.initialization_manifest_sha256)
        || value.authorized_not_before != value.submitted_at
        || value.authorized_valid_until
            != value.authorized_not_before + Duration::hours(AUTHORIZATION_TTL_HOURS)
        || value.reviewer_id.trim().is_empty()
        || !sorted_unique(&value.excluded_prior_actor_ids)
        || value.excluded_prior_actor_ids.contains(&value.reviewer_id)
        || !value.reviewer_independent_from_claimant_and_complete_prior_chain
        || !review_checks(value)
        || value.adapter_contract_authorized != approved
        || !authority_closed
    {
        return Err("Stage 92 只读行情适配器授权无效、漂移或越权".to_string());
    }
    Ok(())
}

async fn read_authorizations(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataAdapterAuthorization>, String> {
    let mut directory = match tokio::fs::read_dir(authorization_directory(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut values = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    let mut claims = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let value: ControlledShadowMarketDataAdapterAuthorization = serde_json::from_slice(
            &tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        validate_authorization(&value)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(value.adapter_authorization_id.as_str())
            || !ids.insert(value.adapter_authorization_id.clone())
            || !hashes.insert(value.adapter_authorization_sha256.clone())
            || !claims.insert(value.cycle_claim_sha256.clone())
        {
            return Err("Stage 92 授权重复、claim 重放或文件名错误".to_string());
        }
        values.push(value);
    }
    values.sort_by(|left, right| left.submitted_at.cmp(&right.submitted_at));
    Ok(values)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 92 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
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
fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
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
fn authorization_directory(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-first-natural-forward-cycle-market-data-adapter-authorizations")
}

struct AuthorizationLock {
    path: PathBuf,
}
impl Drop for AuthorizationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn acquire_lock(state: &AppState, cycle_claim_id: &str) -> Result<AuthorizationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-market-data-adapter-authorization-{cycle_claim_id}.lock"
    ));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(AuthorizationLock { path }),
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
                return Err("另一位管理员正在复核该 Stage 91 任务".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 92 行情适配器授权锁不可用".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_92_adapter_contract_is_fixed_read_only_and_non_executing() {
        let spec = canonical_adapter_specification().expect("spec");
        validate_adapter_specification(&spec).expect("valid spec");
        assert_eq!(spec.allowed_http_methods, ["GET"]);
        assert_eq!(
            spec.allowed_query_parameter_names,
            ["apikey", "from", "symbol", "to"]
        );
        assert!(spec.credential_redaction_required);
        assert!(spec.credential_excluded_from_canonical_request_sha256);
        assert_eq!(spec.benchmark_symbol, "SPY");
        assert!(!spec.arbitrary_url_allowed);
        assert!(!spec.arbitrary_symbol_allowed);
        assert!(!spec.retroactive_backfill_allowed);
        let capability_flags = [false; 15];
        assert!(capability_flags.into_iter().all(|value| !value));
    }

    #[test]
    fn stage_92_adapter_contract_is_content_addressed() {
        let spec = canonical_adapter_specification().expect("spec");
        assert!(valid_sha256(&spec.adapter_spec_sha256));
        assert_eq!(
            spec.adapter_spec_sha256,
            fingerprint_without(&spec, &["adapter_spec_sha256"]).expect("fingerprint")
        );
    }

    #[test]
    fn stage_92_authorization_expires_without_implicitly_accessing_data() {
        let now = Utc::now();
        assert!(now < now + Duration::hours(AUTHORIZATION_TTL_HOURS));
        assert!(POLICY_VERSION.ends_with("no-data-access"));
    }
}
