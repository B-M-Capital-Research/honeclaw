//! Stage 121 claim-first declaration for one exact observation-ledger-transition execution attempt.
//!
//! A claim permanently consumes one currently valid Stage 120 authorization before any runner
//! entrypoint, runtime, Stage 114 admitted output mount/read, or candidate output can exist. The record is
//! metadata-only and embeds the complete server-validated authorization chain. Execution remains a
//! separate Stage 122 gate; the immutable Stage 121 record itself never gains execution authority.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_first_execution_authorizations::{
    ApprovedControlledShadowObservationLedgerTransitionFirstExecutionAuthorization,
    ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationReview,
    observation_ledger_transition_first_execution_authorizations_for_future_claim,
    validate_observation_ledger_transition_first_execution_authorization_for_claim,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-execution-attempt-claim-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-execution-attempt-claim-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-ledger-transition-execution-attempt-claim-v1-create-once-consumes-stage-120-before-execution";
const NEXT_GATE: &str = "stage_122_single_claim_observation_ledger_transition_execution_attempt";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct ClaimControlledShadowObservationLedgerTransitionExecutionAttemptRequest {
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_contract_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_artifact_manifest_sha256: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_review_sha256: String,
    expected_observation_ledger_transition_specification_sha256: String,
    expected_stage_114_admission_review_sha256: String,
    expected_stage_113_validation_sha256: String,
    expected_stage_112_result_sha256: String,
    expected_stage_111_claim_sha256: String,
    claim_reason: String,
    exact_current_stage_51_through_stage_120_binding_confirmed: bool,
    claimant_independent_from_stage_120_and_complete_prior_chain_confirmed: bool,
    authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed: bool,
    current_server_rehashed_artifact_and_manifest_binding_confirmed: bool,
    exact_stage_114_admitted_output_remains_content_addressed_read_only_and_unread_confirmed: bool,
    claim_contains_only_existing_metadata_and_hashes_confirmed: bool,
    no_entrypoint_runtime_input_mount_input_read_or_observation_ledger_transition_execution_confirmed:
        bool,
    future_candidate_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed:
        bool,
    no_retry_release_or_authorization_restoration_after_claim_confirmed: bool,
    no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionExecutionAttemptClaim {
    pub schema_version: String,
    pub policy_version: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub authorization: ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationReview,
    pub claimed_at: DateTime<Utc>,
    pub claimed_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub claimant_independent_from_stage_120_and_complete_prior_chain: bool,
    pub claim_reason: String,
    pub exact_current_stage_51_through_stage_120_binding_confirmed: bool,
    pub authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed:
        bool,
    pub current_server_rehashed_artifact_and_manifest_binding_confirmed: bool,
    pub exact_stage_114_admitted_output_remains_content_addressed_read_only_and_unread_confirmed:
        bool,
    pub claim_contains_only_existing_metadata_and_hashes_confirmed: bool,
    pub no_entrypoint_runtime_input_mount_input_read_or_observation_ledger_transition_execution_confirmed:
        bool,
    pub future_candidate_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed:
        bool,
    pub no_retry_release_or_authorization_restoration_after_claim_confirmed: bool,
    pub no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub authorization_consumed: bool,
    pub create_once: bool,
    pub claim_first: bool,
    pub retry_allowed: bool,
    pub release_allowed: bool,
    pub authorization_restoration_allowed: bool,
    pub task_status: String,
    pub execution_attempt_endpoint_available: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub input_read: bool,
    pub observation_ledger_transition_executed: bool,
    pub candidate_notice_created: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_event_written: bool,
    pub output_written: bool,
    pub cash_written: bool,
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
pub(crate) struct ControlledShadowObservationLedgerTransitionExecutionAttemptClaimCandidate {
    pub authorization: ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationReview,
    pub claimant_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionExecutionAttemptClaimRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub claim_endpoint_available: bool,
    pub eligible_authorizations:
        Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptClaimCandidate>,
    pub claims: Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptClaim>,
    pub authorization_candidate_count: usize,
    pub claim_eligible_count: usize,
    pub claim_count: usize,
    pub authorization_consumed_count: usize,
    pub waiting_for_stage_122_execution_count: usize,
    pub claim_status: String,
    pub next_gate: String,
    pub execution_attempt_endpoint_available: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub input_read: bool,
    pub observation_ledger_transition_executed: bool,
    pub candidate_notice_created: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_event_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationLedgerTransitionExecutionAttemptClaimReadinessSummary {
    pub authorization_candidate_count: usize,
    pub claim_eligible_count: usize,
    pub claim_count: usize,
    pub authorization_consumed_count: usize,
    pub waiting_for_stage_122_execution_count: usize,
    pub claim_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_observation_ledger_transition_execution_attempt_claims(
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
            warn!(%error, "observation-ledger-transition execution-attempt claim registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察到账本转换单次执行尝试声明暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_claim_controlled_shadow_observation_ledger_transition_execution_attempt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(authorization_review_id): AxumPath<String>,
    Json(request): Json<ClaimControlledShadowObservationLedgerTransitionExecutionAttemptRequest>,
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

pub(crate) async fn controlled_shadow_observation_ledger_transition_execution_attempt_claim_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionExecutionAttemptClaimReadinessSummary, String>
{
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowObservationLedgerTransitionExecutionAttemptClaimReadinessSummary {
            authorization_candidate_count: value.authorization_candidate_count,
            claim_eligible_count: value.claim_eligible_count,
            claim_count: value.claim_count,
            authorization_consumed_count: value.authorization_consumed_count,
            waiting_for_stage_122_execution_count: value.waiting_for_stage_122_execution_count,
            claim_status: value.claim_status,
        },
    )
}

pub(crate) async fn claimed_observation_ledger_transition_first_execution_authorization_review_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    Ok(
        observation_ledger_transition_execution_attempt_claims_for_execution(state)
            .await?
            .into_iter()
            .map(|value| value.authorization.review_id)
            .collect(),
    )
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowObservationLedgerTransitionExecutionAttemptClaimRegistry, String> {
    let claims =
        observation_ledger_transition_execution_attempt_claims_for_execution(state).await?;
    let claimed = claims
        .iter()
        .map(|value| value.authorization.review_id.clone())
        .collect::<BTreeSet<_>>();
    let active =
        observation_ledger_transition_first_execution_authorizations_for_future_claim(state, now)
            .await?;
    let eligible_authorizations = active
        .into_iter()
        .filter(|authorization| !claimed.contains(&authorization.review.review_id))
        .map(|authorization| {
            ControlledShadowObservationLedgerTransitionExecutionAttemptClaimCandidate {
                claimant_excluded_actor_ids: expected_excluded_actor_ids(&authorization.review),
                authorization: authorization.review,
            }
        })
        .collect::<Vec<_>>();
    let claim_count = claims.len();
    let claim_eligible_count = eligible_authorizations.len();
    let stage_122_consumed = super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_execution_attempts::started_or_terminal_observation_ledger_transition_claim_attempt_ids(state).await?;
    let stage_122_summary = super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_execution_attempts::controlled_shadow_observation_ledger_transition_execution_readiness_summary(state).await?;
    let waiting_for_stage_122_execution_count = claims
        .iter()
        .filter(|value| !stage_122_consumed.contains(&value.attempt_id))
        .count();
    let claim_status = if waiting_for_stage_122_execution_count > 0 {
        "stage_120_authorization_consumed_waiting_for_separate_stage_122_execution_attempt"
    } else if stage_122_summary.successful_untrusted_candidate_count > 0 {
        "stage_122_untrusted_non_financial_candidate_waiting_independent_validation"
    } else if stage_122_summary.failed_consumed_claim_count > 0 {
        "stage_122_failed_claim_consumed_no_retry"
    } else if claim_eligible_count > 0 {
        "eligible_for_claim_first_create_once_observation_ledger_transition_attempt_declaration"
    } else {
        "waiting_for_unexpired_unconsumed_stage_120_authorization"
    };
    Ok(
        ControlledShadowObservationLedgerTransitionExecutionAttemptClaimRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            claim_endpoint_available: true,
            authorization_candidate_count: claim_eligible_count + claim_count,
            claim_eligible_count,
            claim_count,
            authorization_consumed_count: claims.len(),
            waiting_for_stage_122_execution_count,
            eligible_authorizations,
            claims,
            claim_status: claim_status.to_string(),
            next_gate: NEXT_GATE.to_string(),
            execution_attempt_endpoint_available: true,
            callable_entrypoint_present: false,
            runtime_instantiated: false,
            input_mount_present: false,
            input_read: false,
            observation_ledger_transition_executed: false,
            candidate_notice_created: stage_122_summary.successful_untrusted_candidate_count > 0,
            opening_portfolio_snapshot_admitted: false,
            ledger_event_written: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "Stage 121 在任何入口、runtime、Stage 114 已准入输出挂载/读取或观察到账本转换执行之前，以 create-once 元数据声明永久消费一条未过期 Stage 120 授权。声明自身不运行工件、不读取输入；Stage 122 只允许一次受控、声明式、进程内投影，并且在缺少期初组合时最多创建未受信的非财务观察通知候选。已消费授权不得恢复、释放或重试。".to_string(),
        },
    )
}

async fn claim_once(
    state: &AppState,
    claimant_id: &str,
    authorization_review_id: &str,
    request: ClaimControlledShadowObservationLedgerTransitionExecutionAttemptRequest,
    claimed_at: DateTime<Utc>,
) -> Result<ControlledShadowObservationLedgerTransitionExecutionAttemptClaim, String> {
    if !valid_id(authorization_review_id) {
        return Err("Stage 120 授权 review ID 无效".to_string());
    }
    let _lock = acquire_lock(state, authorization_review_id).await?;
    if observation_ledger_transition_execution_attempt_claims_for_execution(state)
        .await?
        .iter()
        .any(|value| value.authorization.review_id == authorization_review_id)
    {
        return Err("这条 Stage 120 授权已经被永久消费，不能重试或释放".to_string());
    }
    let authorization =
        observation_ledger_transition_first_execution_authorizations_for_future_claim(
            state, claimed_at,
        )
        .await?
        .into_iter()
        .find(|value| value.review.review_id == authorization_review_id)
        .ok_or_else(|| "当前没有这条未过期且未消费的 Stage 120 授权".to_string())?;
    validate_expected_binding(&authorization, &request)?;
    let excluded_prior_actor_ids = expected_excluded_actor_ids(&authorization.review);
    let independent = !claimant_id.trim().is_empty()
        && !excluded_prior_actor_ids
            .iter()
            .any(|value| value == claimant_id);
    if request.claimant_independent_from_stage_120_and_complete_prior_chain_confirmed != independent
    {
        return Err("声明人独立性确认与完整 Stage 51–120 责任链不一致".to_string());
    }
    let mut claim = ControlledShadowObservationLedgerTransitionExecutionAttemptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        attempt_id: String::new(),
        claim_sha256: String::new(),
        authorization: authorization.review,
        claimed_at,
        claimed_by: claimant_id.to_string(),
        excluded_prior_actor_ids,
        claimant_independent_from_stage_120_and_complete_prior_chain: independent,
        claim_reason: bounded_required(&request.claim_reason, MAX_TEXT_CHARS, "声明原因")?,
        exact_current_stage_51_through_stage_120_binding_confirmed: request.exact_current_stage_51_through_stage_120_binding_confirmed,
        authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed: request.authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed,
        current_server_rehashed_artifact_and_manifest_binding_confirmed: request.current_server_rehashed_artifact_and_manifest_binding_confirmed,
        exact_stage_114_admitted_output_remains_content_addressed_read_only_and_unread_confirmed: request.exact_stage_114_admitted_output_remains_content_addressed_read_only_and_unread_confirmed,
        claim_contains_only_existing_metadata_and_hashes_confirmed: request.claim_contains_only_existing_metadata_and_hashes_confirmed,
        no_entrypoint_runtime_input_mount_input_read_or_observation_ledger_transition_execution_confirmed: request.no_entrypoint_runtime_input_mount_input_read_or_observation_ledger_transition_execution_confirmed,
        future_candidate_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed: request.future_candidate_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed,
        no_retry_release_or_authorization_restoration_after_claim_confirmed: request.no_retry_release_or_authorization_restoration_after_claim_confirmed,
        no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed: request.no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        authorization_consumed: true,
        create_once: true,
        claim_first: true,
        retry_allowed: false,
        release_allowed: false,
        authorization_restoration_allowed: false,
        task_status: "authorization_consumed_not_executed_waiting_for_separate_stage_122_attempt".to_string(),
        execution_attempt_endpoint_available: false,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        input_mount_present: false,
        input_read: false,
        observation_ledger_transition_executed: false,
        candidate_notice_created: false,
        opening_portfolio_snapshot_admitted: false,
        ledger_event_written: false,
        output_written: false,
        cash_written: false,
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
    validate_observation_ledger_transition_execution_attempt_claim(&claim)?;
    write_immutable_json(
        &claim_directory(state).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;
    Ok(claim)
}

fn validate_expected_binding(
    authorization: &ApprovedControlledShadowObservationLedgerTransitionFirstExecutionAuthorization,
    request: &ClaimControlledShadowObservationLedgerTransitionExecutionAttemptRequest,
) -> Result<(), String> {
    let review = &authorization.review;
    let runner = &review.runner;
    let implementation = &runner.implementation;
    let contract = &implementation.implementation_contract;
    let specification = &contract.exact_observation_ledger_transition_specification;
    if request.expected_authorization_review_sha256 != review.review_sha256
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_contract_sha256 != runner.runner_contract.contract_sha256
        || request.expected_runner_artifact_sha256 != review.server_computed_artifact_sha256
        || request.expected_artifact_manifest_sha256 != review.artifact_manifest.manifest_sha256
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_review_sha256
            != runner.implementation_review.review_sha256
        || request.expected_observation_ledger_transition_specification_sha256
            != specification.specification_sha256
        || request.expected_stage_114_admission_review_sha256
            != specification.stage_114_review_sha256
        || request.expected_stage_113_validation_sha256 != specification.stage_113_validation_sha256
        || request.expected_stage_112_result_sha256 != specification.stage_112_result_sha256
        || request.expected_stage_111_claim_sha256 != specification.stage_111_claim_sha256
    {
        return Err("Stage 111–120 授权、工件或固定输入责任链已经变化".to_string());
    }
    Ok(())
}

pub(crate) fn validate_observation_ledger_transition_execution_attempt_claim(
    claim: &ControlledShadowObservationLedgerTransitionExecutionAttemptClaim,
) -> Result<(), String> {
    validate_observation_ledger_transition_first_execution_authorization_for_claim(
        &claim.authorization,
    )?;
    let expected_excluded = expected_excluded_actor_ids(&claim.authorization);
    let independent = !claim
        .excluded_prior_actor_ids
        .iter()
        .any(|value| value == &claim.claimed_by);
    let confirmations = claim.exact_current_stage_51_through_stage_120_binding_confirmed
        && claim.authorization_unexpired_single_use_and_permanently_consumed_before_execution_confirmed
        && claim.current_server_rehashed_artifact_and_manifest_binding_confirmed
        && claim.exact_stage_114_admitted_output_remains_content_addressed_read_only_and_unread_confirmed
        && claim.claim_contains_only_existing_metadata_and_hashes_confirmed
        && claim.no_entrypoint_runtime_input_mount_input_read_or_observation_ledger_transition_execution_confirmed
        && claim.future_candidate_output_create_once_content_addressed_untrusted_and_independently_validated_confirmed
        && claim.no_retry_release_or_authorization_restoration_after_claim_confirmed
        && claim.no_authoritative_ledger_event_position_cash_nav_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && claim.no_unconfirmed_hari_or_old_wang_logic_claimed;
    let authority_closed = !claim.execution_attempt_endpoint_available
        && !claim.callable_entrypoint_present
        && !claim.runtime_instantiated
        && !claim.input_mount_present
        && !claim.input_read
        && !claim.observation_ledger_transition_executed
        && !claim.candidate_notice_created
        && !claim.opening_portfolio_snapshot_admitted
        && !claim.ledger_event_written
        && !claim.output_written
        && !claim.cash_written
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
        || !claim.claimant_independent_from_stage_120_and_complete_prior_chain
        || !confirmations
        || !claim.authorization_consumed
        || !claim.create_once
        || !claim.claim_first
        || claim.retry_allowed
        || claim.release_allowed
        || claim.authorization_restoration_allowed
        || claim.task_status
            != "authorization_consumed_not_executed_waiting_for_separate_stage_122_attempt"
        || !authority_closed
    {
        return Err("Stage 121 观察到账本转换单次尝试声明无效、漂移或越权".to_string());
    }
    Ok(())
}

fn expected_excluded_actor_ids(
    review: &ControlledShadowObservationLedgerTransitionFirstExecutionAuthorizationReview,
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

pub(crate) async fn observation_ledger_transition_execution_attempt_claims_for_execution(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationLedgerTransitionExecutionAttemptClaim>, String> {
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
        let claim: ControlledShadowObservationLedgerTransitionExecutionAttemptClaim =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_observation_ledger_transition_execution_attempt_claim(&claim)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(claim.attempt_id.as_str())
            || !ids.insert(claim.attempt_id.clone())
            || !reviews.insert(claim.authorization.review_id.clone())
        {
            return Err("Stage 121 声明重复、重放或文件名错误".to_string());
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
        .ok_or_else(|| "Stage 121 指纹载荷无效".to_string())?;
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
    decision_root(state)
        .join("controlled-shadow-observation-ledger-transition-execution-attempt-claims")
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
        "controlled-shadow-observation-ledger-transition-stage-111-{review_id}.lock"
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
        .map_err(|_| "已有同一 Stage 121 声明正在进行".to_string())?;
    Ok(ClaimLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_121_consumes_stage_120_before_separate_stage_122() {
        assert!(POLICY_VERSION.contains("consumes-stage-120-before-execution"));
        assert_eq!(
            NEXT_GATE,
            "stage_122_single_claim_observation_ledger_transition_execution_attempt"
        );
    }

    #[test]
    fn stage_121_registry_never_opens_execution_or_financial_authority() {
        assert_eq!(
            NEXT_GATE,
            "stage_122_single_claim_observation_ledger_transition_execution_attempt"
        );
        assert!(POLICY_VERSION.contains("create-once"));
        assert!(!POLICY_VERSION.contains("trading-authorized"));
        assert!(!POLICY_VERSION.contains("retry"));
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
