//! Stage 131 claim-first declaration for one exact source-artifact receipt attempt.
//!
//! The claim is persisted before any source byte, upload stream, receiver entrypoint, runtime,
//! mount, or input read may exist. It permanently consumes one current Stage 130 authorization;
//! actual receipt remains a separate Stage 132 gate.

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

use super::controlled_shadow_opening_portfolio_source_artifact_receipt_first_execution_authorizations::{
    ApprovedOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization,
    OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
    opening_portfolio_source_artifact_receipt_first_execution_authorizations_for_future_claim,
    validate_opening_portfolio_source_artifact_receipt_first_execution_authorization_for_claim,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-execution-attempt-claim-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-execution-attempt-claim-v1";
const POLICY_VERSION: &str = "hone-opening-portfolio-source-artifact-receipt-execution-attempt-claim-v1-create-once-consumes-stage-130-before-source-byte";
const NEXT_GATE: &str = "stage_132_one_shot_source_artifact_receipt_attempt";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct ClaimOpeningPortfolioSourceArtifactReceiptExecutionAttemptRequest {
    expected_authorization_review_sha256: String,
    expected_isolated_receiver_spec_sha256: String,
    expected_receiver_contract_sha256: String,
    expected_receiver_artifact_sha256: String,
    expected_artifact_manifest_sha256: String,
    expected_artifact_byte_length: u64,
    claim_reason: String,
    exact_current_stage_51_through_stage_130_binding_confirmed: bool,
    claimant_independent_from_stage_130_builder_reviewer_and_complete_prior_chain_confirmed: bool,
    authorization_unexpired_single_use_and_permanently_consumed_before_source_byte_confirmed: bool,
    server_rehashed_receiver_artifact_and_manifest_before_claim_confirmed: bool,
    claim_contains_only_existing_metadata_and_hashes_confirmed: bool,
    no_upload_stream_source_byte_entrypoint_runtime_mount_input_read_or_receipt_confirmed: bool,
    future_stage_132_attempt_one_shot_create_once_untrusted_and_separately_validated_confirmed:
        bool,
    no_retry_release_or_authorization_restoration_after_claim_confirmed: bool,
    no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim {
    pub schema_version: String,
    pub policy_version: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub authorization: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
    pub claimed_at: DateTime<Utc>,
    pub claimed_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub claim_reason: String,
    pub exact_current_stage_51_through_stage_130_binding_confirmed: bool,
    pub claimant_independent_from_stage_130_builder_reviewer_and_complete_prior_chain_confirmed:
        bool,
    pub authorization_unexpired_single_use_and_permanently_consumed_before_source_byte_confirmed:
        bool,
    pub server_rehashed_receiver_artifact_and_manifest_before_claim_confirmed: bool,
    pub claim_contains_only_existing_metadata_and_hashes_confirmed: bool,
    pub no_upload_stream_source_byte_entrypoint_runtime_mount_input_read_or_receipt_confirmed: bool,
    pub future_stage_132_attempt_one_shot_create_once_untrusted_and_separately_validated_confirmed:
        bool,
    pub no_retry_release_or_authorization_restoration_after_claim_confirmed: bool,
    pub no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub authorization_consumed: bool,
    pub create_once: bool,
    pub claim_first: bool,
    pub retry_allowed: bool,
    pub release_allowed: bool,
    pub authorization_restoration_allowed: bool,
    pub task_status: String,
    pub stage_132_receipt_attempt_endpoint_available: bool,
    pub upload_stream_opened: bool,
    pub source_artifact_received_or_read: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub receipt_manifest_created: bool,
    pub opening_portfolio_snapshot_materialized: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimCandidate {
    pub authorization: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
    pub claimant_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub claim_endpoint_available: bool,
    pub eligible_authorizations:
        Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimCandidate>,
    pub claims: Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim>,
    pub authorization_candidate_count: usize,
    pub claim_eligible_count: usize,
    pub claim_count: usize,
    pub authorization_consumed_count: usize,
    pub waiting_for_stage_132_attempt_count: usize,
    pub claim_status: String,
    pub next_gate: String,
    pub stage_132_receipt_attempt_endpoint_available: bool,
    pub upload_stream_opened: bool,
    pub source_artifact_received_or_read: bool,
    pub runtime_instantiated: bool,
    pub receipt_manifest_created: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimReadinessSummary {
    pub authorization_candidate_count: usize,
    pub claim_eligible_count: usize,
    pub claim_count: usize,
    pub authorization_consumed_count: usize,
    pub waiting_for_stage_132_attempt_count: usize,
    pub claim_status: String,
}

pub(crate) async fn handle_get_opening_portfolio_source_artifact_receipt_execution_attempt_claims(
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
            warn!(%error, "Stage 131 source-artifact receipt claim registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "来源工件接收尝试资格占用暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_claim_opening_portfolio_source_artifact_receipt_execution_attempt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(authorization_review_id): AxumPath<String>,
    Json(request): Json<ClaimOpeningPortfolioSourceArtifactReceiptExecutionAttemptRequest>,
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

pub(crate) async fn opening_portfolio_source_artifact_receipt_execution_attempt_claim_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimReadinessSummary, String> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimReadinessSummary {
            authorization_candidate_count: value.authorization_candidate_count,
            claim_eligible_count: value.claim_eligible_count,
            claim_count: value.claim_count,
            authorization_consumed_count: value.authorization_consumed_count,
            waiting_for_stage_132_attempt_count: value.waiting_for_stage_132_attempt_count,
            claim_status: value.claim_status,
        },
    )
}

pub(crate) async fn claimed_opening_portfolio_source_artifact_receipt_first_execution_authorization_review_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    Ok(read_claims(state)
        .await?
        .into_iter()
        .map(|claim| claim.authorization.review_id)
        .collect())
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry, String> {
    let claims = read_claims(state).await?;
    let claimed = claims
        .iter()
        .map(|claim| claim.authorization.review_id.clone())
        .collect::<BTreeSet<_>>();
    let eligible_authorizations =
        opening_portfolio_source_artifact_receipt_first_execution_authorizations_for_future_claim(
            state, now,
        )
        .await?
        .into_iter()
        .filter(|value| !claimed.contains(&value.review.review_id))
        .map(
            |value| OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimCandidate {
                claimant_excluded_actor_ids: expected_excluded_actor_ids(&value.review),
                authorization: value.review,
            },
        )
        .collect::<Vec<_>>();
    let claim_eligible_count = eligible_authorizations.len();
    let claim_count = claims.len();
    let started_or_terminal = super::controlled_shadow_opening_portfolio_source_artifact_receipt_execution_attempts::started_or_terminal_opening_portfolio_source_artifact_receipt_claim_attempt_ids(state).await?;
    let waiting_for_stage_132_attempt_count = claims
        .iter()
        .filter(|claim| !started_or_terminal.contains(&claim.attempt_id))
        .count();
    let claim_status = if waiting_for_stage_132_attempt_count > 0 {
        "stage_130_authorization_consumed_ready_for_stage_132_one_shot_receipt_attempt"
    } else if claim_count > 0 {
        "all_stage_131_claims_started_or_terminal_in_stage_132"
    } else if claim_eligible_count > 0 {
        "eligible_for_claim_first_create_once_source_artifact_receipt_attempt"
    } else {
        "waiting_for_unexpired_unconsumed_stage_130_authorization"
    };
    Ok(OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaimRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        claim_endpoint_available: true,
        authorization_candidate_count: claim_eligible_count + claim_count,
        claim_eligible_count,
        claim_count,
        authorization_consumed_count: claim_count,
        waiting_for_stage_132_attempt_count,
        eligible_authorizations,
        claims,
        claim_status: claim_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        stage_132_receipt_attempt_endpoint_available: true,
        upload_stream_opened: false,
        source_artifact_received_or_read: false,
        runtime_instantiated: false,
        receipt_manifest_created: false,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 131 必须在任何上传流或来源字节之前永久消费一条当前 Stage 130 授权。Stage 132 已提供独立的单次接收门禁；只有尚未开始的 claim 可用，开始或终止后永不重试。claim 记录本身仍保持历史零能力事实。".to_string(),
    })
}

async fn claim_once(
    state: &AppState,
    claimant_id: &str,
    authorization_review_id: &str,
    request: ClaimOpeningPortfolioSourceArtifactReceiptExecutionAttemptRequest,
    claimed_at: DateTime<Utc>,
) -> Result<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim, String> {
    if !valid_id(authorization_review_id) {
        return Err("Stage 130 授权 review ID 无效".to_string());
    }
    let _lock = acquire_lock(state, authorization_review_id).await?;
    if read_claims(state)
        .await?
        .iter()
        .any(|claim| claim.authorization.review_id == authorization_review_id)
    {
        return Err("这条 Stage 130 授权已被永久消费，不能重试或释放".to_string());
    }
    let authorization =
        opening_portfolio_source_artifact_receipt_first_execution_authorizations_for_future_claim(
            state, claimed_at,
        )
        .await?
        .into_iter()
        .find(|value| value.review.review_id == authorization_review_id)
        .ok_or_else(|| "当前没有这条未过期且未消费的 Stage 130 授权".to_string())?;
    validate_expected_binding(&authorization, &request)?;
    let excluded_prior_actor_ids = expected_excluded_actor_ids(&authorization.review);
    let independent = !claimant_id.trim().is_empty()
        && !excluded_prior_actor_ids
            .iter()
            .any(|value| value == claimant_id);
    if independent
        != request
            .claimant_independent_from_stage_130_builder_reviewer_and_complete_prior_chain_confirmed
    {
        return Err("领取人独立性确认与 Stage 51–130 完整责任链不一致".to_string());
    }
    let mut claim = OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        attempt_id: String::new(),
        claim_sha256: String::new(),
        authorization: authorization.review,
        claimed_at,
        claimed_by: claimant_id.to_string(),
        excluded_prior_actor_ids,
        claim_reason: bounded_required(&request.claim_reason, "领取原因")?,
        exact_current_stage_51_through_stage_130_binding_confirmed: request.exact_current_stage_51_through_stage_130_binding_confirmed,
        claimant_independent_from_stage_130_builder_reviewer_and_complete_prior_chain_confirmed: independent,
        authorization_unexpired_single_use_and_permanently_consumed_before_source_byte_confirmed: request.authorization_unexpired_single_use_and_permanently_consumed_before_source_byte_confirmed,
        server_rehashed_receiver_artifact_and_manifest_before_claim_confirmed: request.server_rehashed_receiver_artifact_and_manifest_before_claim_confirmed,
        claim_contains_only_existing_metadata_and_hashes_confirmed: request.claim_contains_only_existing_metadata_and_hashes_confirmed,
        no_upload_stream_source_byte_entrypoint_runtime_mount_input_read_or_receipt_confirmed: request.no_upload_stream_source_byte_entrypoint_runtime_mount_input_read_or_receipt_confirmed,
        future_stage_132_attempt_one_shot_create_once_untrusted_and_separately_validated_confirmed: request.future_stage_132_attempt_one_shot_create_once_untrusted_and_separately_validated_confirmed,
        no_retry_release_or_authorization_restoration_after_claim_confirmed: request.no_retry_release_or_authorization_restoration_after_claim_confirmed,
        no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: request.no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        authorization_consumed: true,
        create_once: true,
        claim_first: true,
        retry_allowed: false,
        release_allowed: false,
        authorization_restoration_allowed: false,
        task_status: "authorization_consumed_not_received_waiting_for_separate_stage_132_attempt".to_string(),
        stage_132_receipt_attempt_endpoint_available: false,
        upload_stream_opened: false,
        source_artifact_received_or_read: false,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        input_mount_present: false,
        receipt_manifest_created: false,
        opening_portfolio_snapshot_materialized: false,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        position_or_cash_written: false,
        nav_or_performance_written: false,
        model_or_metric_store_written: false,
        training_or_rl_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    claim.claim_sha256 = fingerprint_without(&claim, &["attempt_id", "claim_sha256"])?;
    claim.attempt_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim)?;
    write_immutable_json(
        &claim_directory(state).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;
    Ok(claim)
}

fn validate_expected_binding(
    authorization: &ApprovedOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization,
    request: &ClaimOpeningPortfolioSourceArtifactReceiptExecutionAttemptRequest,
) -> Result<(), String> {
    let review = &authorization.review;
    if request.expected_authorization_review_sha256 != review.review_sha256
        || request.expected_isolated_receiver_spec_sha256
            != review.receiver.isolated_receiver_spec_sha256
        || request.expected_receiver_contract_sha256
            != review.receiver.receiver_contract.contract_sha256
        || request.expected_receiver_artifact_sha256 != review.server_computed_artifact_sha256
        || request.expected_artifact_manifest_sha256 != review.artifact_manifest.manifest_sha256
        || request.expected_artifact_byte_length != review.server_observed_artifact_byte_length
    {
        return Err("Stage 129–130 授权、接收器工件或 manifest 已变化".to_string());
    }
    Ok(())
}

pub(crate) fn validate_claim(
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
) -> Result<(), String> {
    validate_opening_portfolio_source_artifact_receipt_first_execution_authorization_for_claim(
        &claim.authorization,
    )?;
    let confirmations = claim.exact_current_stage_51_through_stage_130_binding_confirmed
        && claim.claimant_independent_from_stage_130_builder_reviewer_and_complete_prior_chain_confirmed
        && claim.authorization_unexpired_single_use_and_permanently_consumed_before_source_byte_confirmed
        && claim.server_rehashed_receiver_artifact_and_manifest_before_claim_confirmed
        && claim.claim_contains_only_existing_metadata_and_hashes_confirmed
        && claim.no_upload_stream_source_byte_entrypoint_runtime_mount_input_read_or_receipt_confirmed
        && claim.future_stage_132_attempt_one_shot_create_once_untrusted_and_separately_validated_confirmed
        && claim.no_retry_release_or_authorization_restoration_after_claim_confirmed
        && claim.no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && claim.no_unconfirmed_hari_or_old_wang_logic_claimed;
    let authority_closed = !claim.stage_132_receipt_attempt_endpoint_available
        && !claim.upload_stream_opened
        && !claim.source_artifact_received_or_read
        && !claim.callable_entrypoint_present
        && !claim.runtime_instantiated
        && !claim.input_mount_present
        && !claim.receipt_manifest_created
        && !claim.opening_portfolio_snapshot_materialized
        && !claim.opening_portfolio_snapshot_admitted
        && !claim.financial_event_allowlist_nonempty
        && !claim.ledger_created
        && !claim.position_or_cash_written
        && !claim.nav_or_performance_written
        && !claim.model_or_metric_store_written
        && !claim.training_or_rl_feedback_authorized
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
        || claim.excluded_prior_actor_ids != expected_excluded_actor_ids(&claim.authorization)
        || claim
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &claim.claimed_by)
        || !confirmations
        || !claim.authorization_consumed
        || !claim.create_once
        || !claim.claim_first
        || claim.retry_allowed
        || claim.release_allowed
        || claim.authorization_restoration_allowed
        || claim.task_status
            != "authorization_consumed_not_received_waiting_for_separate_stage_132_attempt"
        || !authority_closed
    {
        return Err("Stage 131 来源工件接收尝试资格占用无效、漂移或越权".to_string());
    }
    Ok(())
}

fn expected_excluded_actor_ids(
    review: &OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
) -> Vec<String> {
    let mut values = review.excluded_prior_actor_ids.clone();
    values.push(review.reviewer_id.clone());
    values.push(review.artifact_manifest.reproduced_by.clone());
    values.push(review.receiver.registered_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

pub(crate) async fn opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim>, String> {
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
        let claim: OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_claim(&claim)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(claim.attempt_id.as_str())
            || !ids.insert(claim.attempt_id.clone())
            || !reviews.insert(claim.authorization.review_id.clone())
        {
            return Err("Stage 131 claim 重复、重放或文件名错误".to_string());
        }
        claims.push(claim);
    }
    claims.sort_by(|left, right| left.claimed_at.cmp(&right.claimed_at));
    Ok(claims)
}

async fn read_claims(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim>, String> {
    opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt(state).await
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 131 指纹载荷无效".to_string())?;
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
    decision_root(state).join("opening-portfolio-source-artifact-receipt-execution-attempt-claims")
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
        "opening-portfolio-source-artifact-receipt-stage-131-{review_id}.lock"
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
        .map_err(|_| "已有同一 Stage 131 claim 正在进行".to_string())?;
    Ok(ClaimLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_131_consumes_stage_130_before_any_source_byte() {
        assert!(POLICY_VERSION.contains("consumes-stage-130-before-source-byte"));
        assert_eq!(
            NEXT_GATE,
            "stage_132_one_shot_source_artifact_receipt_attempt"
        );
    }

    #[test]
    fn stage_131_does_not_open_upload_runtime_or_financial_authority() {
        assert!(!POLICY_VERSION.contains("trading-authorized"));
        assert!(!NEXT_GATE.contains("snapshot"));
    }

    #[test]
    fn identity_formats_are_fail_closed() {
        assert!(valid_sha256(&"a".repeat(64)));
        assert!(!valid_sha256(&"a".repeat(63)));
        assert!(valid_id(&"b".repeat(32)));
        assert!(!valid_id("not-an-id"));
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
            fingerprint_without(&right, &["id", "sha"]).unwrap()
        );
    }
}
