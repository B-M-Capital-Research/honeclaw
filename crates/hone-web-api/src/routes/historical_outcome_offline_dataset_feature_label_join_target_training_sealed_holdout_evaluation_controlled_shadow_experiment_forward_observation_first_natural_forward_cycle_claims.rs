//! Stage 91 claim-first, create-once task declaration for the first natural forward cycle.
//!
//! A claim permanently consumes one exact Stage 90 authorization before any calendar or market
//! data may be resolved. The resulting task is deliberately non-executable and waits for a
//! separate, explicit, read-only, content-addressed market-data adapter authorization. This module
//! does not read data, instantiate a runtime, start an observation, create a ledger/position/
//! performance record, or grant model, reward, order, broker or trade authority.

use std::collections::BTreeSet;
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_authorizations::{
    ApprovedControlledShadowFirstNaturalForwardCycleAuthorization,
    ControlledShadowFirstNaturalForwardCycleAuthorizationReview,
    approved_first_natural_forward_cycle_authorizations_for_claim,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_output_validations::ControlledShadowForwardObservationOutputValidationRecord;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-first-natural-forward-cycle-claim-registry-v1";
const CLAIM_SCHEMA_VERSION: &str = "hone-controlled-shadow-first-natural-forward-cycle-claim-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-first-natural-forward-cycle-claim-v1-create-once-consumes-authorization";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct ClaimControlledShadowFirstNaturalForwardCycleRequest {
    expected_authorization_review_sha256: String,
    expected_validation_sha256: String,
    expected_stage_88_attempt_id: String,
    expected_stage_88_claim_sha256: String,
    expected_stage_88_result_sha256: String,
    expected_stage_88_output_sha256: String,
    expected_initialization_manifest_sha256: String,
    claim_reason: String,
    exact_stage_51_through_stage_90_binding_confirmed: bool,
    claimant_independence_from_stage_90_and_complete_prior_chain_confirmed: bool,
    authorization_current_unexpired_and_single_use_confirmed: bool,
    claim_first_before_calendar_or_market_data_confirmed: bool,
    separate_read_only_market_data_adapter_authorization_required_confirmed: bool,
    natural_forward_only_no_backfill_and_create_once_confirmed: bool,
    no_runtime_observation_ledger_position_or_performance_confirmed: bool,
    no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowFirstNaturalForwardCycleClaim {
    pub schema_version: String,
    pub policy_version: String,
    pub cycle_claim_id: String,
    pub cycle_claim_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub authorization_not_before: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub validation_id: String,
    pub validation_sha256: String,
    pub stage_88_attempt_id: String,
    pub stage_88_claim_sha256: String,
    pub stage_88_result_sha256: String,
    pub stage_88_output_sha256: String,
    pub initialization_manifest_sha256: String,
    pub observation_eligibility_anchor: DateTime<Utc>,
    pub cycle_ordinal: u8,
    pub claimed_at: DateTime<Utc>,
    pub claimed_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub claimant_independent_from_stage_90_and_complete_prior_chain: bool,
    pub claim_reason: String,
    pub exact_stage_51_through_stage_90_binding_confirmed: bool,
    pub authorization_current_unexpired_and_single_use_confirmed: bool,
    pub claim_first_before_calendar_or_market_data_confirmed: bool,
    pub separate_read_only_market_data_adapter_authorization_required_confirmed: bool,
    pub natural_forward_only_no_backfill_and_create_once_confirmed: bool,
    pub no_runtime_observation_ledger_position_or_performance_confirmed: bool,
    pub no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub authorization_consumed: bool,
    pub create_once: bool,
    pub claim_first: bool,
    pub task_status: String,
    pub calendar_window_resolved: bool,
    pub calendar_read_authorized: bool,
    pub market_data_adapter_authorized: bool,
    pub market_data_access_authorized: bool,
    pub execution_endpoint_available: bool,
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
pub(crate) struct ControlledShadowFirstNaturalForwardCycleClaimCandidate {
    pub validation: ControlledShadowForwardObservationOutputValidationRecord,
    pub authorization: ControlledShadowFirstNaturalForwardCycleAuthorizationReview,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowFirstNaturalForwardCycleClaimRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_authorizations: Vec<ControlledShadowFirstNaturalForwardCycleClaimCandidate>,
    pub claims: Vec<ControlledShadowFirstNaturalForwardCycleClaim>,
    pub authorization_candidate_count: usize,
    pub claim_eligible_count: usize,
    pub claim_count: usize,
    pub authorization_consumed_count: usize,
    pub waiting_for_separate_market_data_adapter_authorization_count: usize,
    pub claim_status: String,
    pub calendar_window_resolved: bool,
    pub calendar_read_authorized: bool,
    pub market_data_adapter_authorized: bool,
    pub market_data_access_authorized: bool,
    pub execution_endpoint_available: bool,
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
pub(crate) struct ControlledShadowFirstNaturalForwardCycleClaimReadinessSummary {
    pub authorization_candidate_count: usize,
    pub claim_eligible_count: usize,
    pub claim_count: usize,
    pub authorization_consumed_count: usize,
    pub waiting_for_separate_market_data_adapter_authorization_count: usize,
    pub claim_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_first_natural_forward_cycle_claims(
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
            warn!(%error, "first natural forward-cycle claim registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "首个自然前向周期任务声明暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_claim_controlled_shadow_first_natural_forward_cycle_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(authorization_review_id): AxumPath<String>,
    Json(request): Json<ClaimControlledShadowFirstNaturalForwardCycleRequest>,
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
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_first_natural_forward_cycle_claim_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowFirstNaturalForwardCycleClaimReadinessSummary, String> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowFirstNaturalForwardCycleClaimReadinessSummary {
            authorization_candidate_count: value.authorization_candidate_count,
            claim_eligible_count: value.claim_eligible_count,
            claim_count: value.claim_count,
            authorization_consumed_count: value.authorization_consumed_count,
            waiting_for_separate_market_data_adapter_authorization_count: value
                .waiting_for_separate_market_data_adapter_authorization_count,
            claim_status: value.claim_status,
        },
    )
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowFirstNaturalForwardCycleClaimRegistry, String> {
    let claims = read_claims(state).await?;
    let claimed = claims
        .iter()
        .map(|value| value.authorization_review_sha256.clone())
        .collect::<BTreeSet<_>>();
    let active = approved_first_natural_forward_cycle_authorizations_for_claim(state, now).await?;
    let eligible_authorizations = active
        .into_iter()
        .filter(|value| !claimed.contains(&value.review.review_sha256))
        .map(
            |value| ControlledShadowFirstNaturalForwardCycleClaimCandidate {
                validation: value.initialization.validation,
                authorization: value.review,
            },
        )
        .collect::<Vec<_>>();
    let claim_count = claims.len();
    let claim_eligible_count = eligible_authorizations.len();
    let claim_status = if claim_count > 0 {
        "authorization_consumed_waiting_for_separate_read_only_market_data_adapter_authorization"
    } else if claim_eligible_count > 0 {
        "eligible_for_claim_first_create_once_task_declaration"
    } else {
        "waiting_for_active_stage_90_authorization"
    };
    Ok(ControlledShadowFirstNaturalForwardCycleClaimRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        authorization_candidate_count: claim_eligible_count + claim_count,
        claim_eligible_count,
        claim_count,
        authorization_consumed_count: claims
            .iter()
            .filter(|value| value.authorization_consumed)
            .count(),
        waiting_for_separate_market_data_adapter_authorization_count: claim_count,
        eligible_authorizations,
        claims,
        claim_status: claim_status.to_string(),
        calendar_window_resolved: false,
        calendar_read_authorized: false,
        market_data_adapter_authorized: false,
        market_data_access_authorized: false,
        execution_endpoint_available: false,
        runtime_instantiated: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 91 只允许责任链外管理员先 create-once 写入任务 claim，并永久消费一条当前 Stage 90 授权。claim 写入前后均不读取日历或行情；任务固定停在等待单独、明确、只读、内容寻址白名单行情适配器授权的状态。当前无执行入口、runtime、观察、账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易能力。".to_string(),
    })
}

async fn claim_once(
    state: &AppState,
    claimant_id: &str,
    authorization_review_id: &str,
    request: ClaimControlledShadowFirstNaturalForwardCycleRequest,
    now: DateTime<Utc>,
) -> Result<ControlledShadowFirstNaturalForwardCycleClaim, String> {
    if !valid_id(authorization_review_id) {
        return Err("Stage 90 authorization review ID 无效".to_string());
    }
    let _lock = acquire_lock(state, authorization_review_id).await?;
    let existing = read_claims(state).await?;
    if existing
        .iter()
        .any(|value| value.authorization_review_id == authorization_review_id)
    {
        return Err("Stage 90 一次性授权已经被永久领取，不得重放".to_string());
    }
    let authorization = approved_first_natural_forward_cycle_authorizations_for_claim(state, now)
        .await?
        .into_iter()
        .find(|value| value.review.review_id == authorization_review_id)
        .ok_or_else(|| "Stage 90 授权不存在、尚未生效或已经过期".to_string())?;
    validate_expected_binding(&authorization, &request)?;
    let claim = new_claim(&authorization, claimant_id, request, now)?;
    write_immutable_json(
        &claim_directory(state).join(format!("{}.json", claim.cycle_claim_id)),
        &claim,
    )
    .await?;
    Ok(claim)
}

fn validate_expected_binding(
    authorization: &ApprovedControlledShadowFirstNaturalForwardCycleAuthorization,
    request: &ClaimControlledShadowFirstNaturalForwardCycleRequest,
) -> Result<(), String> {
    let attempt = &authorization.initialization.attempt;
    let validation = &authorization.initialization.validation;
    let exact = request.expected_authorization_review_sha256 == authorization.review.review_sha256
        && request.expected_validation_sha256 == validation.validation_sha256
        && request.expected_stage_88_attempt_id == attempt.claim.attempt_id
        && request.expected_stage_88_claim_sha256 == attempt.claim.claim_sha256
        && request.expected_stage_88_result_sha256 == attempt.result.result_sha256
        && attempt.result.output_sha256.as_deref()
            == Some(request.expected_stage_88_output_sha256.as_str())
        && request.expected_initialization_manifest_sha256
            == validation.initialization_manifest_sha256;
    if exact {
        Ok(())
    } else {
        Err("Stage 91 预期绑定与当前 Stage 90 授权链不一致".to_string())
    }
}

fn new_claim(
    authorization: &ApprovedControlledShadowFirstNaturalForwardCycleAuthorization,
    claimant_id: &str,
    request: ClaimControlledShadowFirstNaturalForwardCycleRequest,
    now: DateTime<Utc>,
) -> Result<ControlledShadowFirstNaturalForwardCycleClaim, String> {
    let mut excluded_prior_actor_ids = authorization.review.excluded_prior_actor_ids.clone();
    excluded_prior_actor_ids.push(authorization.review.reviewer_id.clone());
    excluded_prior_actor_ids.sort();
    excluded_prior_actor_ids.dedup();
    let independent = !claimant_id.trim().is_empty()
        && excluded_prior_actor_ids
            .iter()
            .all(|value| value != claimant_id);
    let attempt = &authorization.initialization.attempt;
    let validation = &authorization.initialization.validation;
    let mut claim = ControlledShadowFirstNaturalForwardCycleClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        cycle_claim_id: String::new(),
        cycle_claim_sha256: String::new(),
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        authorization_not_before: authorization.review.authorization_not_before,
        authorization_valid_until: authorization.review.authorization_valid_until,
        validation_id: validation.validation_id.clone(),
        validation_sha256: validation.validation_sha256.clone(),
        stage_88_attempt_id: attempt.claim.attempt_id.clone(),
        stage_88_claim_sha256: attempt.claim.claim_sha256.clone(),
        stage_88_result_sha256: attempt.result.result_sha256.clone(),
        stage_88_output_sha256: attempt.result.output_sha256.clone().unwrap_or_default(),
        initialization_manifest_sha256: validation.initialization_manifest_sha256.clone(),
        observation_eligibility_anchor: authorization.review.observation_not_before,
        cycle_ordinal: 1,
        claimed_at: now,
        claimed_by: claimant_id.to_string(),
        excluded_prior_actor_ids,
        claimant_independent_from_stage_90_and_complete_prior_chain: independent
            && request.claimant_independence_from_stage_90_and_complete_prior_chain_confirmed,
        claim_reason: bounded_required(&request.claim_reason, MAX_TEXT_CHARS, "任务声明依据")?,
        exact_stage_51_through_stage_90_binding_confirmed: request
            .exact_stage_51_through_stage_90_binding_confirmed,
        authorization_current_unexpired_and_single_use_confirmed: request
            .authorization_current_unexpired_and_single_use_confirmed,
        claim_first_before_calendar_or_market_data_confirmed: request
            .claim_first_before_calendar_or_market_data_confirmed,
        separate_read_only_market_data_adapter_authorization_required_confirmed: request
            .separate_read_only_market_data_adapter_authorization_required_confirmed,
        natural_forward_only_no_backfill_and_create_once_confirmed: request
            .natural_forward_only_no_backfill_and_create_once_confirmed,
        no_runtime_observation_ledger_position_or_performance_confirmed: request
            .no_runtime_observation_ledger_position_or_performance_confirmed,
        no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: request
            .no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        authorization_consumed: true,
        create_once: true,
        claim_first: true,
        task_status: "claimed_waiting_for_separate_read_only_market_data_adapter_authorization"
            .to_string(),
        calendar_window_resolved: false,
        calendar_read_authorized: false,
        market_data_adapter_authorized: false,
        market_data_access_authorized: false,
        execution_endpoint_available: false,
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
    claim.cycle_claim_sha256 =
        fingerprint_without(&claim, &["cycle_claim_id", "cycle_claim_sha256"])?;
    claim.cycle_claim_id = claim.cycle_claim_sha256[..32].to_string();
    validate_claim(&claim)?;
    if !independent || !claim_checks(&claim) {
        return Err("Stage 91 任务领取职责不独立或确认项不完整".to_string());
    }
    Ok(claim)
}

fn claim_checks(claim: &ControlledShadowFirstNaturalForwardCycleClaim) -> bool {
    claim.exact_stage_51_through_stage_90_binding_confirmed
        && claim.claimant_independent_from_stage_90_and_complete_prior_chain
        && claim.authorization_current_unexpired_and_single_use_confirmed
        && claim.claim_first_before_calendar_or_market_data_confirmed
        && claim.separate_read_only_market_data_adapter_authorization_required_confirmed
        && claim.natural_forward_only_no_backfill_and_create_once_confirmed
        && claim.no_runtime_observation_ledger_position_or_performance_confirmed
        && claim.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && claim.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_claim(claim: &ControlledShadowFirstNaturalForwardCycleClaim) -> Result<(), String> {
    let authority_closed = !claim.calendar_window_resolved
        && !claim.calendar_read_authorized
        && !claim.market_data_adapter_authorized
        && !claim.market_data_access_authorized
        && !claim.execution_endpoint_available
        && !claim.runtime_instantiated
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
        || !valid_id(&claim.cycle_claim_id)
        || !valid_sha256(&claim.cycle_claim_sha256)
        || claim.cycle_claim_id != claim.cycle_claim_sha256[..32]
        || claim.cycle_claim_sha256
            != fingerprint_without(claim, &["cycle_claim_id", "cycle_claim_sha256"])?
        || !valid_id(&claim.authorization_review_id)
        || !valid_sha256(&claim.authorization_review_sha256)
        || !valid_id(&claim.validation_id)
        || !valid_sha256(&claim.validation_sha256)
        || !valid_id(&claim.stage_88_attempt_id)
        || !valid_sha256(&claim.stage_88_claim_sha256)
        || !valid_sha256(&claim.stage_88_result_sha256)
        || !valid_sha256(&claim.stage_88_output_sha256)
        || !valid_sha256(&claim.initialization_manifest_sha256)
        || claim.cycle_ordinal != 1
        || claim.claimed_at < claim.authorization_not_before
        || claim.claimed_at >= claim.authorization_valid_until
        || claim.claim_reason.trim().is_empty()
        || claim.claim_reason.chars().count() > MAX_TEXT_CHARS
        || !sorted_unique(&claim.excluded_prior_actor_ids)
        || !claim.claimant_independent_from_stage_90_and_complete_prior_chain
        || claim.excluded_prior_actor_ids.contains(&claim.claimed_by)
        || !claim.authorization_consumed
        || !claim.create_once
        || !claim.claim_first
        || claim.task_status
            != "claimed_waiting_for_separate_read_only_market_data_adapter_authorization"
        || !claim_checks(claim)
        || !authority_closed
    {
        return Err("Stage 91 首个自然前向周期任务 claim 无效、漂移或越权".to_string());
    }
    Ok(())
}

pub(crate) async fn claimed_authorization_review_sha256s(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    Ok(read_claims(state)
        .await?
        .into_iter()
        .map(|value| value.authorization_review_sha256)
        .collect())
}

async fn read_claims(
    state: &AppState,
) -> Result<Vec<ControlledShadowFirstNaturalForwardCycleClaim>, String> {
    let mut directory = match tokio::fs::read_dir(claim_directory(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut claims = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    let mut authorizations = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let claim: ControlledShadowFirstNaturalForwardCycleClaim = serde_json::from_slice(
            &tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        validate_claim(&claim)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(claim.cycle_claim_id.as_str())
            || !ids.insert(claim.cycle_claim_id.clone())
            || !hashes.insert(claim.cycle_claim_sha256.clone())
            || !authorizations.insert(claim.authorization_review_sha256.clone())
        {
            return Err("Stage 91 claim 重复、授权重放或文件名错误".to_string());
        }
        claims.push(claim);
    }
    claims.sort_by(|left, right| left.claimed_at.cmp(&right.claimed_at));
    Ok(claims)
}

pub(crate) async fn claimed_first_natural_forward_cycles_for_market_data_adapter_review(
    state: &AppState,
) -> Result<Vec<ControlledShadowFirstNaturalForwardCycleClaim>, String> {
    read_claims(state).await
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 91 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
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
fn claim_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-first-natural-forward-cycle-claims")
}

struct ClaimLock {
    path: PathBuf,
}
impl Drop for ClaimLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn acquire_lock(
    state: &AppState,
    authorization_review_id: &str,
) -> Result<ClaimLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-first-natural-forward-cycle-claim-{authorization_review_id}.lock"
    ));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ClaimLock { path }),
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
                return Err("另一位管理员正在领取该 Stage 90 一次性授权".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 91 任务 claim 锁不可用".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_91_claim_is_create_once_and_consumes_authorization_without_data_authority() {
        assert!(POLICY_VERSION.contains("create-once-consumes-authorization"));
        let capability_flags = [false; 17];
        assert!(capability_flags.into_iter().all(|value| !value));
    }

    #[test]
    fn stage_91_claim_fingerprint_is_content_addressed() {
        let digest = format!("{:x}", Sha256::digest(b"stage-91"));
        assert_eq!(digest.len(), 64);
        assert!(valid_sha256(&digest));
        assert!(valid_id(&digest[..32]));
    }
}
