//! Stage 79 append-only independent first-execution authorization review for one exact
//! controlled-shadow isolated runner specification.
//!
//! Stage 78 registered one exact runner artifact and code revision without a callable entrypoint or
//! current mount. This review independently reproduces the artifact digest, verifies artifact
//! availability and replays the complete upstream hash chain. Approval is valid for 24 hours and exposes at most one future
//! claim-first Stage 80 execution-attempt eligibility. This module has no claim or execution
//! endpoint, attaches no input, starts no shadow run, creates no ledger or position and grants no
//! order, broker or trading authority.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_isolated_runners::{
    ControlledShadowExperimentIsolatedRunnerRecord,
    controlled_shadow_runners_for_first_execution_authorization_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-first-execution-authorization-registry-v2-artifact-reviewed";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-first-execution-authorization-review-v2-artifact-reviewed";
const POLICY_VERSION: &str = "hone-controlled-shadow-experiment-first-execution-authorization-v2-artifact-reviewed-single-use-24h";
const AUTHORIZATION_VALID_HOURS: i64 = 24;
const ONE_SHOT_EXECUTION_ATTEMPT_LIMIT: u8 = 1;
const MAX_RATIONALE_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowExperimentFirstExecutionAuthorizationVerdict {
    ApprovedForOneFutureIsolatedControlledShadowExecutionAttempt,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_contract_sha256: String,
    expected_runner_spec_revision: String,
    expected_runner_code_revision: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_independent_audit_sha256: String,
    expected_design_review_sha256: String,
    expected_design_registration_sha256: String,
    expected_design_specification_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    verdict: ControlledShadowExperimentFirstExecutionAuthorizationVerdict,
    rationale: String,
    exact_current_stage_51_through_stage_78_binding_confirmed: bool,
    reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed: bool,
    runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed: bool,
    runner_artifact_digest_independently_reproduced: bool,
    immutable_code_revision_reproducible_and_artifact_available_confirmed: bool,
    no_callable_entrypoint_or_current_mount_confirmed: bool,
    future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: bool,
    future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed: bool,
    deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed:
        bool,
    fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    no_model_metric_store_training_feedback_composite_or_reward_confirmed: bool,
    authorization_single_use_and_24_hour_expiry_confirmed: bool,
    authorization_claim_execution_and_output_validation_separation_confirmed: bool,
    no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed: bool,
    approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentFirstExecutionAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub runner: ControlledShadowExperimentIsolatedRunnerRecord,
    pub submitted_at: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowExperimentFirstExecutionAuthorizationVerdict,
    pub rationale: String,
    pub exact_current_stage_51_through_stage_78_binding_confirmed: bool,
    pub reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed: bool,
    pub runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed:
        bool,
    pub runner_artifact_digest_independently_reproduced: bool,
    pub immutable_code_revision_reproducible_and_artifact_available_confirmed: bool,
    pub no_callable_entrypoint_or_current_mount_confirmed: bool,
    pub future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed:
        bool,
    pub future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed:
        bool,
    pub deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    pub no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    pub no_model_metric_store_training_feedback_composite_or_reward_confirmed: bool,
    pub authorization_single_use_and_24_hour_expiry_confirmed: bool,
    pub authorization_claim_execution_and_output_validation_separation_confirmed: bool,
    pub no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed: bool,
    pub approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub one_shot_execution_attempt_limit: u8,
    pub one_future_isolated_controlled_shadow_execution_attempt_authorized: bool,
    pub authorization_claimed: bool,
    pub execution_attempt_endpoint_available: bool,
    pub input_manifest_attached: bool,
    pub point_in_time_input_access_authorized: bool,
    pub shadow_execution_authorized: bool,
    pub shadow_run_started: bool,
    pub shadow_run_completed: bool,
    pub untrusted_output_created: bool,
    pub independent_output_validation_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentFirstExecutionAuthorizationItem {
    pub runner: ControlledShadowExperimentIsolatedRunnerRecord,
    pub current_binding: bool,
    pub latest_review: Option<ControlledShadowExperimentFirstExecutionAuthorizationReview>,
    pub one_future_isolated_controlled_shadow_execution_attempt_authorized: bool,
    pub authorization_unexpired: bool,
    pub execution_attempt_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentFirstExecutionAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowExperimentFirstExecutionAuthorizationItem>,
    pub review_eligible_runner_count: usize,
    pub reviewed_runner_count: usize,
    pub approved_runner_count: usize,
    pub unexpired_authorization_count: usize,
    pub one_shot_authorized_count: usize,
    pub execution_attempt_eligible_count: usize,
    pub authorization_status: String,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub execution_attempt_endpoint_available: bool,
    pub input_manifest_attached: bool,
    pub point_in_time_input_access_authorized: bool,
    pub shadow_execution_authorized: bool,
    pub shadow_run_started: bool,
    pub shadow_run_completed: bool,
    pub untrusted_output_created: bool,
    pub independent_output_validation_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowExperimentFirstExecutionAuthorizationReadinessSummary {
    pub reviewed_runner_count: usize,
    pub approved_runner_count: usize,
    pub unexpired_authorization_count: usize,
    pub one_shot_authorized_count: usize,
    pub execution_attempt_eligible_count: usize,
    pub authorization_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedControlledShadowExperimentFirstExecutionAuthorization {
    pub runner: ControlledShadowExperimentIsolatedRunnerRecord,
    pub review: ControlledShadowExperimentFirstExecutionAuthorizationReview,
}

pub(crate) async fn handle_get_controlled_shadow_experiment_first_execution_authorizations(
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
            warn!(%error, "controlled shadow first execution authorization registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "受控影子首次执行授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_experiment_first_execution_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_first_execution_authorization(&state, &admin.user_id, &isolated_runner_id, request)
        .await
    {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_experiment_first_execution_authorization_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowExperimentFirstExecutionAuthorizationReadinessSummary, String> {
    let registry = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowExperimentFirstExecutionAuthorizationReadinessSummary {
            reviewed_runner_count: registry.reviewed_runner_count,
            approved_runner_count: registry.approved_runner_count,
            unexpired_authorization_count: registry.unexpired_authorization_count,
            one_shot_authorized_count: registry.one_shot_authorized_count,
            execution_attempt_eligible_count: registry.execution_attempt_eligible_count,
            authorization_status: registry.authorization_status,
        },
    )
}

#[allow(dead_code)] // Consumed only by the separately gated future Stage 80 claim-first attempt.
pub(crate) async fn execution_attempt_eligible_controlled_shadow_authorizations(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Vec<ApprovedControlledShadowExperimentFirstExecutionAuthorization>, String> {
    let runners = controlled_shadow_runners_for_first_execution_authorization_review(state).await?;
    let mut approved = Vec::new();
    for runner in runners {
        let Some(review) = latest_review(state, &runner).await? else {
            continue;
        };
        if authorization_unexpired(&review, now) {
            approved.push(
                ApprovedControlledShadowExperimentFirstExecutionAuthorization { runner, review },
            );
        }
    }
    approved.sort_by(|left, right| {
        left.runner
            .isolated_runner_id
            .cmp(&right.runner.isolated_runner_id)
    });
    Ok(approved)
}

#[allow(dead_code)] // Reserved for a future Stage 80 attempt audit; it does not execute anything.
pub(crate) async fn exact_controlled_shadow_first_execution_authorization_for_audit(
    state: &AppState,
    isolated_runner_id: &str,
    review_id: &str,
    review_sha256: &str,
) -> Result<Option<ApprovedControlledShadowExperimentFirstExecutionAuthorization>, String> {
    let runners = controlled_shadow_runners_for_first_execution_authorization_review(state).await?;
    let Some(runner) = runners
        .into_iter()
        .find(|value| value.isolated_runner_id == isolated_runner_id)
    else {
        return Ok(None);
    };
    let reviews = read_reviews(state, &runner).await?;
    let _ = latest_review_from_records(&reviews)?;
    let Some(review) = reviews.into_iter().find(|value| {
        value.review_id == review_id
            && value.review_sha256 == review_sha256
            && value.one_future_isolated_controlled_shadow_execution_attempt_authorized
    }) else {
        return Ok(None);
    };
    Ok(Some(
        ApprovedControlledShadowExperimentFirstExecutionAuthorization { runner, review },
    ))
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowExperimentFirstExecutionAuthorizationRegistry, String> {
    let runners = controlled_shadow_runners_for_first_execution_authorization_review(state).await?;
    let claimed_authorizations = super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_execution_attempts::claimed_controlled_shadow_authorization_review_ids(state).await?;
    let mut items = Vec::with_capacity(runners.len());
    for runner in runners {
        let latest_review = latest_review(state, &runner).await?;
        let authorized = latest_review.as_ref().is_some_and(|review| {
            review.one_future_isolated_controlled_shadow_execution_attempt_authorized
        });
        let unexpired = latest_review
            .as_ref()
            .is_some_and(|review| authorization_unexpired(review, now));
        let execution_attempt_eligible = unexpired
            && latest_review
                .as_ref()
                .is_some_and(|review| !claimed_authorizations.contains(&review.review_id));
        items.push(ControlledShadowExperimentFirstExecutionAuthorizationItem {
            runner,
            current_binding: true,
            latest_review,
            one_future_isolated_controlled_shadow_execution_attempt_authorized: authorized,
            authorization_unexpired: unexpired,
            execution_attempt_eligible,
        });
    }
    let review_eligible_runner_count = items.len();
    let reviewed_runner_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let approved_runner_count = items
        .iter()
        .filter(|item| item.one_future_isolated_controlled_shadow_execution_attempt_authorized)
        .count();
    let unexpired_authorization_count = items
        .iter()
        .filter(|item| item.authorization_unexpired)
        .count();
    let one_shot_authorized_count = unexpired_authorization_count;
    let execution_attempt_eligible_count = items
        .iter()
        .filter(|item| item.execution_attempt_eligible)
        .count();
    let artifact_bound_runner_present = items.iter().any(|item| item.current_binding);
    let authorization_status = if items.is_empty() {
        "waiting_for_current_registered_not_run_controlled_shadow_runner_specification"
    } else if execution_attempt_eligible_count > 0 {
        "approved_for_one_future_claim_first_controlled_shadow_execution_attempt_not_started"
    } else if reviewed_runner_count > 0 {
        "reviewed_not_currently_authorized"
    } else {
        "waiting_for_independent_first_shadow_execution_authorization_review"
    };
    Ok(ControlledShadowExperimentFirstExecutionAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_runner_count,
        reviewed_runner_count,
        approved_runner_count,
        unexpired_authorization_count,
        one_shot_authorized_count,
        execution_attempt_eligible_count,
        authorization_status: authorization_status.to_string(),
        executable_artifact_present: artifact_bound_runner_present,
        callable_entrypoint_present: false,
        runtime_present: artifact_bound_runner_present,
        input_mount_present: false,
        execution_attempt_endpoint_available: false,
        input_manifest_attached: false,
        point_in_time_input_access_authorized: false,
        shadow_execution_authorized: false,
        shadow_run_started: false,
        shadow_run_completed: false,
        untrusted_output_created: false,
        independent_output_validation_authorized: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 79 阶段只由 Stage 51–78 完整责任链之外的新角色独立复算精确 runner 规格、可执行工件摘要、代码版本、合同与全部上游指纹，并可授予 24 小时内最多一次的未来 Stage 80 claim-first 影子执行尝试资格。当前没有 callable entrypoint 或 mount，也不附加输入、不创建 claim、不执行影子运行、不写账本/持仓/模型/指标，不生成订单、不接券商或交易。".to_string(),
    })
}

async fn review_first_execution_authorization(
    state: &AppState,
    reviewer_id: &str,
    isolated_runner_id: &str,
    request: ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest,
) -> Result<ControlledShadowExperimentFirstExecutionAuthorizationReview, String> {
    if !valid_id(isolated_runner_id) {
        return Err("受控影子隔离 runner 规格 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let runner = controlled_shadow_runners_for_first_execution_authorization_review(state)
        .await?
        .into_iter()
        .find(|value| value.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "当前没有这条绑定有效且可独立复核的受控影子 runner 规格".to_string())?;
    validate_expected_binding(&runner, &request)?;
    let latest = latest_review(state, &runner).await?;
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("首次影子执行授权复核链已经变化，请刷新后重试".to_string());
    }
    let review = new_review(runner, latest.as_ref(), reviewer_id, request, Utc::now())?;
    write_immutable_json(
        &review_directory(state, &review.runner).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn new_review(
    runner: ControlledShadowExperimentIsolatedRunnerRecord,
    latest: Option<&ControlledShadowExperimentFirstExecutionAuthorizationReview>,
    reviewer_id: &str,
    request: ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<ControlledShadowExperimentFirstExecutionAuthorizationReview, String> {
    if latest.is_some_and(|review| {
        review.one_future_isolated_controlled_shadow_execution_attempt_authorized
    }) {
        return Err("已批准的首次影子执行授权复核链不得继续追加".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(&runner, latest);
    let independent = !excluded_prior_actor_ids
        .iter()
        .any(|value| value == reviewer_id);
    if request.reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed != independent
    {
        return Err("复核者独立性确认与 Stage 51–78 完整责任链不一致".to_string());
    }
    let mut review = ControlledShadowExperimentFirstExecutionAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.map(|value| value.review_id.clone()),
        previous_review_sha256: latest.map(|value| value.review_sha256.clone()),
        runner,
        submitted_at,
        authorization_valid_until: submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        exact_current_stage_51_through_stage_78_binding_confirmed: request
            .exact_current_stage_51_through_stage_78_binding_confirmed,
        reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed: request
            .reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed,
        runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed:
            request
                .runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed,
        runner_artifact_digest_independently_reproduced: request
            .runner_artifact_digest_independently_reproduced,
        immutable_code_revision_reproducible_and_artifact_available_confirmed: request
            .immutable_code_revision_reproducible_and_artifact_available_confirmed,
        no_callable_entrypoint_or_current_mount_confirmed: request
            .no_callable_entrypoint_or_current_mount_confirmed,
        future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed:
            request
                .future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed,
        future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed:
            request
                .future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed,
        deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed:
            request
                .deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
            request
                .fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: request
            .no_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_model_metric_store_training_feedback_composite_or_reward_confirmed: request
            .no_model_metric_store_training_feedback_composite_or_reward_confirmed,
        authorization_single_use_and_24_hour_expiry_confirmed: request
            .authorization_single_use_and_24_hour_expiry_confirmed,
        authorization_claim_execution_and_output_validation_separation_confirmed: request
            .authorization_claim_execution_and_output_validation_separation_confirmed,
        no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed: request
            .no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed,
        approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed: request
            .approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        one_shot_execution_attempt_limit: ONE_SHOT_EXECUTION_ATTEMPT_LIMIT,
        one_future_isolated_controlled_shadow_execution_attempt_authorized: false,
        authorization_claimed: false,
        execution_attempt_endpoint_available: false,
        input_manifest_attached: false,
        point_in_time_input_access_authorized: false,
        shadow_execution_authorized: false,
        shadow_run_started: false,
        shadow_run_completed: false,
        untrusted_output_created: false,
        independent_output_validation_authorized: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedControlledShadowExecutionAttempt;
    review.one_future_isolated_controlled_shadow_execution_attempt_authorized =
        approved && review_all_checks(&review) && independent;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &review.runner)?;
    Ok(review)
}

fn validate_expected_binding(
    runner: &ControlledShadowExperimentIsolatedRunnerRecord,
    request: &ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest,
) -> Result<(), String> {
    let contract = &runner.runner_contract;
    let implementation = &runner.implementation;
    let implementation_contract = &implementation.implementation_contract;
    let implementation_review = &runner.implementation_review;
    if runner.isolated_runner_spec_sha256 != request.expected_isolated_runner_spec_sha256
        || contract.contract_sha256 != request.expected_runner_contract_sha256
        || runner.runner_spec_revision != request.expected_runner_spec_revision
        || runner.runner_code_revision != request.expected_runner_code_revision
        || runner.runner_artifact_sha256 != request.expected_runner_artifact_sha256
        || implementation.implementation_id != request.expected_implementation_id
        || implementation.implementation_sha256 != request.expected_implementation_sha256
        || implementation_contract.contract_sha256
            != request.expected_implementation_contract_sha256
        || implementation_review.review_id != request.expected_implementation_review_id
        || implementation_review.review_sha256 != request.expected_implementation_review_sha256
        || implementation_review.independent_audit.audit_sha256
            != request.expected_independent_audit_sha256
        || contract.stage_75_design_review_sha256 != request.expected_design_review_sha256
        || contract.stage_74_design_registration_sha256
            != request.expected_design_registration_sha256
        || contract.design_specification_sha256 != request.expected_design_specification_sha256
        || implementation_contract.selected_algorithm_three_seed_binding_sha256
            != request.expected_selected_algorithm_three_seed_binding_sha256
        || implementation_contract.sealed_holdout_split_commitment_sha256
            != request.expected_sealed_holdout_split_commitment_sha256
        || implementation_contract.feature_order_sha256 != request.expected_feature_order_sha256
        || implementation_contract.preprocessing_sha256 != request.expected_preprocessing_sha256
        || implementation_contract.target_id != request.expected_target_id
        || implementation_contract.frozen_candidate_algorithm_id
            != request.expected_frozen_candidate_algorithm_id
    {
        return Err("Stage 78 runner 规格、合同或完整上游绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn review_all_checks(review: &ControlledShadowExperimentFirstExecutionAuthorizationReview) -> bool {
    review.exact_current_stage_51_through_stage_78_binding_confirmed
        && review.reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed
        && review
            .runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed
        && review.runner_artifact_digest_independently_reproduced
        && review.immutable_code_revision_reproducible_and_artifact_available_confirmed
        && review.no_callable_entrypoint_or_current_mount_confirmed
        && review
            .future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed
        && review
            .future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed
        && review
            .deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed
        && review
            .fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed
        && review.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && review.no_model_metric_store_training_feedback_composite_or_reward_confirmed
        && review.authorization_single_use_and_24_hour_expiry_confirmed
        && review.authorization_claim_execution_and_output_validation_separation_confirmed
        && review.no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed
        && review.approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn authority_closed(review: &ControlledShadowExperimentFirstExecutionAuthorizationReview) -> bool {
    !review.authorization_claimed
        && !review.execution_attempt_endpoint_available
        && !review.input_manifest_attached
        && !review.point_in_time_input_access_authorized
        && !review.shadow_execution_authorized
        && !review.shadow_run_started
        && !review.shadow_run_completed
        && !review.untrusted_output_created
        && !review.independent_output_validation_authorized
        && !review.shadow_ledger_enabled
        && !review.shadow_position_written
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized
}

fn validate_review(
    review: &ControlledShadowExperimentFirstExecutionAuthorizationReview,
    runner: &ControlledShadowExperimentIsolatedRunnerRecord,
) -> Result<(), String> {
    let approved = review.verdict
        == ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedControlledShadowExecutionAttempt;
    let independent = !review
        .excluded_prior_actor_ids
        .iter()
        .any(|value| value == &review.reviewer_id);
    let expected_base_actors = expected_excluded_prior_actor_ids(runner, None);
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != POLICY_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.review_sha256)
        || review.review_id != review.review_sha256[..32]
        || review.review_sha256 != review_fingerprint(review)?
        || review.previous_review_id.is_some() != review.previous_review_sha256.is_some()
        || review
            .previous_review_id
            .as_deref()
            .is_some_and(|value| !valid_id(value))
        || review
            .previous_review_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256(value))
        || &review.runner != runner
        || review.authorization_valid_until
            != review.submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS)
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || !expected_base_actors
            .iter()
            .all(|value| review.excluded_prior_actor_ids.contains(value))
        || review.reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed
            != independent
        || (approved && (!independent || !review_all_checks(review)))
        || review.one_shot_execution_attempt_limit != ONE_SHOT_EXECUTION_ATTEMPT_LIMIT
        || review.one_future_isolated_controlled_shadow_execution_attempt_authorized
            != (approved && independent && review_all_checks(review))
        || !authority_closed(review)
    {
        return Err(
            "controlled shadow first execution authorization is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn expected_excluded_prior_actor_ids(
    runner: &ControlledShadowExperimentIsolatedRunnerRecord,
    latest: Option<&ControlledShadowExperimentFirstExecutionAuthorizationReview>,
) -> Vec<String> {
    let mut actors = runner.excluded_prior_actor_ids.clone();
    actors.push(runner.registered_by.clone());
    actors.push(runner.implementation_review.reviewer_id.clone());
    if let Some(latest) = latest {
        actors.extend(latest.excluded_prior_actor_ids.clone());
        actors.push(latest.reviewer_id.clone());
    }
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn authorization_unexpired(
    review: &ControlledShadowExperimentFirstExecutionAuthorizationReview,
    now: DateTime<Utc>,
) -> bool {
    review.one_future_isolated_controlled_shadow_execution_attempt_authorized
        && !review.authorization_claimed
        && now >= review.submitted_at
        && now < review.authorization_valid_until
}

fn review_fingerprint(
    review: &ControlledShadowExperimentFirstExecutionAuthorizationReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    runner: &ControlledShadowExperimentIsolatedRunnerRecord,
) -> Result<Option<ControlledShadowExperimentFirstExecutionAuthorizationReview>, String> {
    let reviews = read_reviews(state, runner).await?;
    latest_review_from_records(&reviews)
}

async fn read_reviews(
    state: &AppState,
    runner: &ControlledShadowExperimentIsolatedRunnerRecord,
) -> Result<Vec<ControlledShadowExperimentFirstExecutionAuthorizationReview>, String> {
    let directory = review_directory(state, runner);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
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
        let review: ControlledShadowExperimentFirstExecutionAuthorizationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err(
                "controlled shadow first execution authorization chain is duplicated or misnamed"
                    .to_string(),
            );
        }
        reviews.push(review);
    }
    Ok(reviews)
}

fn latest_review_from_records(
    reviews: &[ControlledShadowExperimentFirstExecutionAuthorizationReview],
) -> Result<Option<ControlledShadowExperimentFirstExecutionAuthorizationReview>, String> {
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
    if reviews.iter().any(|review| {
        review.one_future_isolated_controlled_shadow_execution_attempt_authorized
            && reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
    }) {
        return Err(
            "controlled shadow first execution authorization approval must terminate the chain"
                .to_string(),
        );
    }
    if tips.len() != 1 {
        return Err(
            "controlled shadow first execution authorization chain has invalid tips".to_string(),
        );
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err(
                "controlled shadow first execution authorization chain has a cycle".to_string(),
            );
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(previous_id), Some(previous_sha256)) => {
                let previous = by_id.get(previous_id.as_str()).ok_or_else(|| {
                    "controlled shadow first execution authorization chain is disconnected"
                        .to_string()
                })?;
                if previous.review_sha256 != *previous_sha256 {
                    return Err(
                        "controlled shadow first execution authorization previous hash mismatch"
                            .to_string(),
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "controlled shadow first execution authorization previous link is invalid"
                        .to_string(),
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err(
            "controlled shadow first execution authorization chain is disconnected".to_string(),
        );
    }
    for review in reviews {
        let previous = review
            .previous_review_id
            .as_deref()
            .and_then(|id| by_id.get(id).copied());
        if review.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&review.runner, previous)
        {
            return Err(
                "controlled shadow first execution authorization actor chain mismatch".to_string(),
            );
        }
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    let bytes = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max_chars: usize, field: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars {
        return Err(format!("{field}为空或过长"));
    }
    Ok(value.to_string())
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

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_directory(
    state: &AppState,
    runner: &ControlledShadowExperimentIsolatedRunnerRecord,
) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-experiment-first-execution-authorization-reviews")
        .join(&runner.isolated_runner_id)
}

struct ReviewLock {
    path: PathBuf,
}

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, isolated_runner_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-experiment-first-execution-{isolated_runner_id}.lock"
    ));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ReviewLock { path }),
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
                return Err("另一位管理员正在提交首次影子执行授权复核".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("首次影子执行授权复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_isolated_runners::tests::record;

    fn request(
        runner: &ControlledShadowExperimentIsolatedRunnerRecord,
        verdict: ControlledShadowExperimentFirstExecutionAuthorizationVerdict,
    ) -> ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest {
        let implementation = &runner.implementation;
        let contract = &implementation.implementation_contract;
        ReviewControlledShadowExperimentFirstExecutionAuthorizationRequest {
            expected_review_id: None,
            expected_review_sha256: None,
            expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
            expected_runner_contract_sha256: runner.runner_contract.contract_sha256.clone(),
            expected_runner_spec_revision: runner.runner_spec_revision.clone(),
            expected_runner_code_revision: runner.runner_code_revision.clone(),
            expected_runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
            expected_implementation_id: implementation.implementation_id.clone(),
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_contract_sha256: contract.contract_sha256.clone(),
            expected_implementation_review_id: runner.implementation_review.review_id.clone(),
            expected_implementation_review_sha256: runner
                .implementation_review
                .review_sha256
                .clone(),
            expected_independent_audit_sha256: runner
                .implementation_review
                .independent_audit
                .audit_sha256
                .clone(),
            expected_design_review_sha256: runner
                .runner_contract
                .stage_75_design_review_sha256
                .clone(),
            expected_design_registration_sha256: runner
                .runner_contract
                .stage_74_design_registration_sha256
                .clone(),
            expected_design_specification_sha256: runner
                .runner_contract
                .design_specification_sha256
                .clone(),
            expected_selected_algorithm_three_seed_binding_sha256: contract
                .selected_algorithm_three_seed_binding_sha256
                .clone(),
            expected_sealed_holdout_split_commitment_sha256: contract
                .sealed_holdout_split_commitment_sha256
                .clone(),
            expected_feature_order_sha256: contract.feature_order_sha256.clone(),
            expected_preprocessing_sha256: contract.preprocessing_sha256.clone(),
            expected_target_id: contract.target_id.clone(),
            expected_frozen_candidate_algorithm_id: contract
                .frozen_candidate_algorithm_id
                .clone(),
            verdict,
            rationale: "独立复算 Stage 78 规格与完整上游；只开放未来一次 claim-first 尝试"
                .to_string(),
            exact_current_stage_51_through_stage_78_binding_confirmed: true,
            reviewer_independence_from_stage_78_and_complete_prior_chain_confirmed: true,
            runner_specification_contract_and_complete_hash_chain_independently_reproduced_confirmed: true,
            runner_artifact_digest_independently_reproduced: true,
            immutable_code_revision_reproducible_and_artifact_available_confirmed: true,
            no_callable_entrypoint_or_current_mount_confirmed: true,
            future_single_use_point_in_time_read_only_content_addressed_allowlisted_input_confirmed: true,
            future_create_once_untrusted_independently_validated_no_order_payload_output_confirmed: true,
            deterministic_replay_long_only_caps_costs_counterfactuals_observations_and_stop_rules_confirmed: true,
            fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: true,
            no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_model_metric_store_training_feedback_composite_or_reward_confirmed: true,
            authorization_single_use_and_24_hour_expiry_confirmed: true,
            authorization_claim_execution_and_output_validation_separation_confirmed: true,
            no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_stage_80_claim_first_execution_attempt_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    pub(crate) fn approved_review() -> ControlledShadowExperimentFirstExecutionAuthorizationReview {
        let runner = record();
        let request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedControlledShadowExecutionAttempt,
        );
        new_review(runner, None, "stage-79-reviewer", request, Utc::now()).unwrap()
    }

    #[test]
    fn approval_is_content_addressed_and_exactly_bound() {
        let review = approved_review();
        assert_eq!(review.review_id, review.review_sha256[..32]);
        assert_eq!(
            review
                .runner
                .runner_contract
                .stage_76_implementation_contract_sha256,
            review
                .runner
                .implementation
                .implementation_contract
                .contract_sha256
        );
        assert!(validate_review(&review, &review.runner).is_ok());
    }

    #[test]
    fn review_binds_available_artifact_but_does_not_open_entrypoint_or_mount() {
        let review = approved_review();
        assert!(review.runner.runner_contract.executable_artifact_present);
        assert!(!review.runner.runner_contract.callable_entrypoint_present);
        assert!(review.runner.runner_contract.runtime_present);
        assert!(!review.runner.runner_contract.input_mount_present);
        assert!(review.runner_artifact_digest_independently_reproduced);
        assert!(review.immutable_code_revision_reproducible_and_artifact_available_confirmed);
        assert!(review.no_callable_entrypoint_or_current_mount_confirmed);
    }

    #[test]
    fn approval_is_one_shot_and_expires_after_twenty_four_hours() {
        let review = approved_review();
        assert_eq!(review.one_shot_execution_attempt_limit, 1);
        assert!(authorization_unexpired(
            &review,
            review.submitted_at + TimeDelta::hours(23)
        ));
        assert!(!authorization_unexpired(
            &review,
            review.submitted_at + TimeDelta::hours(24)
        ));
    }

    #[test]
    fn reviewer_must_be_outside_stage_51_through_stage_78_chain() {
        let runner = record();
        let request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedControlledShadowExecutionAttempt,
        );
        assert!(
            new_review(
                runner.clone(),
                None,
                &runner.registered_by,
                request,
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn every_approval_confirmation_is_required() {
        let runner = record();
        let mut request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedControlledShadowExecutionAttempt,
        );
        request.no_input_attachment_shadow_run_ledger_position_order_broker_or_trading_confirmed =
            false;
        assert!(new_review(runner, None, "stage-79-reviewer", request, Utc::now()).is_err());
    }

    #[test]
    fn changes_requested_does_not_authorize_future_attempt() {
        let runner = record();
        let request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ChangesRequested,
        );
        let review = new_review(runner, None, "stage-79-reviewer", request, Utc::now()).unwrap();
        assert!(!review.one_future_isolated_controlled_shadow_execution_attempt_authorized);
    }

    #[test]
    fn stale_expected_binding_is_rejected() {
        let runner = record();
        let mut request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ChangesRequested,
        );
        request.expected_runner_contract_sha256 = "0".repeat(64);
        assert!(validate_expected_binding(&runner, &request).is_err());
    }

    #[test]
    fn runner_artifact_or_code_revision_drift_is_rejected() {
        let runner = record();
        let mut artifact_drift_request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ChangesRequested,
        );
        artifact_drift_request.expected_runner_artifact_sha256 = "0".repeat(64);
        assert!(validate_expected_binding(&runner, &artifact_drift_request).is_err());

        let mut revision_drift_request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ChangesRequested,
        );
        revision_drift_request.expected_runner_code_revision = "drifted-code-revision".to_string();
        assert!(validate_expected_binding(&runner, &revision_drift_request).is_err());
    }

    #[test]
    fn tampering_with_authority_is_rejected() {
        let mut review = approved_review();
        review.shadow_execution_authorized = true;
        assert!(validate_review(&review, &review.runner).is_err());
    }

    #[test]
    fn approval_must_terminate_append_only_chain() {
        let first = approved_review();
        let runner = first.runner.clone();
        let mut request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ChangesRequested,
        );
        request.expected_review_id = Some(first.review_id.clone());
        request.expected_review_sha256 = Some(first.review_sha256.clone());
        assert!(
            new_review(
                runner,
                Some(&first),
                "another-stage-79-reviewer",
                request,
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn append_only_chain_preserves_prior_reviewer_exclusion() {
        let runner = record();
        let first_request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ChangesRequested,
        );
        let first = new_review(
            runner.clone(),
            None,
            "stage-79-reviewer-a",
            first_request,
            Utc::now(),
        )
        .unwrap();
        let mut second_request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::Rejected,
        );
        second_request.expected_review_id = Some(first.review_id.clone());
        second_request.expected_review_sha256 = Some(first.review_sha256.clone());
        let second = new_review(
            runner,
            Some(&first),
            "stage-79-reviewer-b",
            second_request,
            Utc::now(),
        )
        .unwrap();
        assert!(
            second
                .excluded_prior_actor_ids
                .contains(&"stage-79-reviewer-a".to_string())
        );
        assert!(latest_review_from_records(&[first, second]).is_ok());
    }

    #[test]
    fn all_execution_ledger_store_order_broker_and_trading_authority_remains_closed() {
        let review = approved_review();
        assert!(review.one_future_isolated_controlled_shadow_execution_attempt_authorized);
        assert!(authority_closed(&review));
        assert!(!review.execution_attempt_endpoint_available);
        assert!(!review.input_manifest_attached);
        assert!(!review.shadow_run_started);
        assert!(!review.shadow_ledger_enabled);
        assert!(!review.shadow_position_written);
        assert!(!review.order_generation_authorized);
        assert!(!review.broker_access_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn bounded_rationale_is_enforced() {
        let runner = record();
        let mut request = request(
            &runner,
            ControlledShadowExperimentFirstExecutionAuthorizationVerdict::ChangesRequested,
        );
        request.rationale = "x".repeat(MAX_RATIONALE_CHARS + 1);
        assert!(new_review(runner, None, "stage-79-reviewer", request, Utc::now()).is_err());
    }
}
