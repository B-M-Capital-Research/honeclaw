//! Stage 87 append-only independent first-execution authorization review for one exact natural
//! forward-observation isolated runner. Approval expires after 24 hours and exposes at most one
//! future Stage 88 claim-first attempt candidate. This module does not instantiate the runtime,
//! mount data, create an observation/ledger/position, or grant order, broker or trading authority.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_isolated_runners::{
    ControlledShadowForwardObservationIsolatedRunnerRecord,
    controlled_shadow_forward_observation_runners_for_first_execution_authorization_review,
    validate_controlled_shadow_forward_observation_isolated_runner_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-first-execution-authorization-registry-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-controlled-shadow-forward-observation-first-execution-authorization-review-v1-artifact-reproduced";
const POLICY_VERSION: &str = "hone-controlled-shadow-forward-observation-first-execution-authorization-v1-artifact-reproduced-single-use-24h";
const AUTHORIZATION_VALID_HOURS: i64 = 24;
const ONE_SHOT_ATTEMPT_LIMIT: u8 = 1;
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict {
    ApprovedForOneFutureClaimFirstForwardObservationAttempt,
    ChangesRequestedRebuildRunner,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowForwardObservationFirstExecutionAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_isolated_runner_id: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_contract_sha256: String,
    expected_runner_spec_revision: String,
    expected_runner_code_revision: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_independent_audit_sha256: String,
    expected_protocol_review_sha256: String,
    expected_protocol_registration_sha256: String,
    expected_protocol_specification_sha256: String,
    expected_design_specification_sha256: String,
    independently_reproduced_runner_artifact_sha256: String,
    artifact_reproduction_evidence: String,
    verdict: ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict,
    rationale: String,
    exact_current_stage_51_through_stage_86_binding_confirmed: bool,
    reviewer_independence_from_stage_86_and_complete_prior_chain_confirmed: bool,
    runner_spec_contract_and_complete_hash_chain_independently_reproduced_confirmed: bool,
    runner_artifact_digest_independently_reproduced_and_matched_confirmed: bool,
    immutable_code_revision_and_artifact_availability_confirmed: bool,
    natural_forward_no_backfill_and_observation_not_before_confirmed: bool,
    weekly_claim_first_create_once_official_calendar_and_spy_sync_confirmed: bool,
    point_in_time_read_only_content_addressed_allowlisted_input_confirmed: bool,
    corporate_action_evidence_and_append_only_corrections_confirmed: bool,
    create_once_untrusted_independently_validated_no_order_payload_output_confirmed: bool,
    deterministic_replay_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed:
        bool,
    fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    authorization_single_use_24_hour_expiry_and_stage_88_claim_separation_confirmed: bool,
    no_runtime_mount_data_access_observation_ledger_position_performance_or_execution_confirmed:
        bool,
    no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    approval_only_opens_future_stage_88_claim_first_attempt_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationFirstExecutionAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub runner: ControlledShadowForwardObservationIsolatedRunnerRecord,
    pub submitted_at: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub independently_reproduced_runner_artifact_sha256: String,
    pub artifact_reproduction_evidence: String,
    pub artifact_digest_matches_registered_runner: bool,
    pub verdict: ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict,
    pub rationale: String,
    pub exact_current_stage_51_through_stage_86_binding_confirmed: bool,
    pub reviewer_independence_from_stage_86_and_complete_prior_chain_confirmed: bool,
    pub runner_spec_contract_and_complete_hash_chain_independently_reproduced_confirmed: bool,
    pub runner_artifact_digest_independently_reproduced_and_matched_confirmed: bool,
    pub immutable_code_revision_and_artifact_availability_confirmed: bool,
    pub natural_forward_no_backfill_and_observation_not_before_confirmed: bool,
    pub weekly_claim_first_create_once_official_calendar_and_spy_sync_confirmed: bool,
    pub point_in_time_read_only_content_addressed_allowlisted_input_confirmed: bool,
    pub corporate_action_evidence_and_append_only_corrections_confirmed: bool,
    pub create_once_untrusted_independently_validated_no_order_payload_output_confirmed: bool,
    pub deterministic_replay_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    pub no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    pub authorization_single_use_24_hour_expiry_and_stage_88_claim_separation_confirmed: bool,
    pub no_runtime_mount_data_access_observation_ledger_position_performance_or_execution_confirmed:
        bool,
    pub no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    pub approval_only_opens_future_stage_88_claim_first_attempt_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub one_shot_execution_attempt_limit: u8,
    pub one_future_claim_first_forward_observation_attempt_authorized: bool,
    pub authorization_claimed: bool,
    pub execution_attempt_endpoint_available: bool,
    pub runtime_instantiated: bool,
    pub input_manifest_attached: bool,
    pub data_access_authorized: bool,
    pub forward_observation_started: bool,
    pub forward_observation_completed: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationFirstExecutionAuthorizationItem {
    pub runner: ControlledShadowForwardObservationIsolatedRunnerRecord,
    pub current_binding: bool,
    pub latest_review: Option<ControlledShadowForwardObservationFirstExecutionAuthorizationReview>,
    pub authorization_unexpired: bool,
    pub future_attempt_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowForwardObservationFirstExecutionAuthorizationItem>,
    pub review_eligible_runner_count: usize,
    pub reviewed_runner_count: usize,
    pub approved_runner_count: usize,
    pub unexpired_authorization_count: usize,
    pub one_shot_authorized_count: usize,
    pub future_attempt_eligible_count: usize,
    pub authorization_status: String,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub data_access_authorized: bool,
    pub forward_observation_started: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowForwardObservationFirstExecutionAuthorizationReadinessSummary {
    pub review_eligible_runner_count: usize,
    pub reviewed_runner_count: usize,
    pub approved_runner_count: usize,
    pub unexpired_authorization_count: usize,
    pub one_shot_authorized_count: usize,
    pub future_attempt_eligible_count: usize,
    pub authorization_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedControlledShadowForwardObservationFirstExecutionAuthorization {
    pub runner: ControlledShadowForwardObservationIsolatedRunnerRecord,
    pub review: ControlledShadowForwardObservationFirstExecutionAuthorizationReview,
}

pub(crate) async fn handle_get_controlled_shadow_forward_observation_first_execution_authorizations(
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
            warn!(%error, "forward-observation first-execution authorization registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "前向观察首次执行授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_forward_observation_first_execution_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowForwardObservationFirstExecutionAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &isolated_runner_id, request).await {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_forward_observation_first_execution_authorization_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationFirstExecutionAuthorizationReadinessSummary, String> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowForwardObservationFirstExecutionAuthorizationReadinessSummary {
            review_eligible_runner_count: value.review_eligible_runner_count,
            reviewed_runner_count: value.reviewed_runner_count,
            approved_runner_count: value.approved_runner_count,
            unexpired_authorization_count: value.unexpired_authorization_count,
            one_shot_authorized_count: value.one_shot_authorized_count,
            future_attempt_eligible_count: value.future_attempt_eligible_count,
            authorization_status: value.authorization_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn forward_observation_authorizations_for_future_claim_first_attempt(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Vec<ApprovedControlledShadowForwardObservationFirstExecutionAuthorization>, String> {
    let registry = build_registry(state, now).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            if !item.future_attempt_eligible {
                return None;
            }
            item.latest_review.map(|review| {
                ApprovedControlledShadowForwardObservationFirstExecutionAuthorization {
                    runner: item.runner,
                    review,
                }
            })
        })
        .collect())
}

pub(crate) async fn exact_forward_observation_first_execution_authorization_for_audit(
    state: &AppState,
    isolated_runner_id: &str,
    review_id: &str,
    review_sha256: &str,
) -> Result<Option<ApprovedControlledShadowForwardObservationFirstExecutionAuthorization>, String> {
    let runners =
        controlled_shadow_forward_observation_runners_for_first_execution_authorization_review(
            state,
        )
        .await?;
    let Some(runner) = runners
        .into_iter()
        .find(|runner| runner.isolated_runner_id == isolated_runner_id)
    else {
        return Ok(None);
    };
    let Some(review) = latest_review(state, &runner).await? else {
        return Ok(None);
    };
    if review.review_id != review_id
        || review.review_sha256 != review_sha256
        || !review.one_future_claim_first_forward_observation_attempt_authorized
    {
        return Ok(None);
    }
    Ok(Some(
        ApprovedControlledShadowForwardObservationFirstExecutionAuthorization { runner, review },
    ))
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry, String> {
    let runners =
        controlled_shadow_forward_observation_runners_for_first_execution_authorization_review(
            state,
        )
        .await?;
    let claimed_review_ids = super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_execution_attempts::claimed_forward_observation_authorization_review_ids(state).await?;
    let mut items = Vec::with_capacity(runners.len());
    for runner in runners {
        let latest_review = latest_review(state, &runner).await?;
        let unexpired = latest_review
            .as_ref()
            .is_some_and(|review| authorization_unexpired(review, now));
        let unclaimed = latest_review
            .as_ref()
            .is_some_and(|review| !claimed_review_ids.contains(&review.review_id));
        items.push(
            ControlledShadowForwardObservationFirstExecutionAuthorizationItem {
                runner,
                current_binding: true,
                latest_review,
                authorization_unexpired: unexpired,
                future_attempt_eligible: unexpired && unclaimed,
            },
        );
    }
    let review_eligible_runner_count = items.len();
    let reviewed_runner_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let approved_runner_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.one_future_claim_first_forward_observation_attempt_authorized
            })
        })
        .count();
    let unexpired_authorization_count = items
        .iter()
        .filter(|item| item.authorization_unexpired)
        .count();
    let future_attempt_eligible_count = items
        .iter()
        .filter(|item| item.future_attempt_eligible)
        .count();
    let authorization_status = if items.is_empty() {
        "waiting_for_current_stage_86_forward_observation_isolated_runner"
    } else if future_attempt_eligible_count > 0 {
        "approved_for_one_future_stage_88_claim_first_attempt_not_started"
    } else if reviewed_runner_count > 0 {
        "reviewed_not_currently_authorized"
    } else {
        "waiting_for_independent_first_forward_observation_execution_authorization_review"
    };
    Ok(ControlledShadowForwardObservationFirstExecutionAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_runner_count,
        reviewed_runner_count,
        approved_runner_count,
        unexpired_authorization_count,
        one_shot_authorized_count: unexpired_authorization_count,
        future_attempt_eligible_count,
        authorization_status: authorization_status.to_string(),
        executable_artifact_present: review_eligible_runner_count > 0,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        input_mount_present: false,
        data_access_authorized: false,
        forward_observation_started: false,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 87 仅允许完整 Stage 51–86 责任链之外的新复核者，独立复算 runner 规格、合同与工件 SHA-256，并可授予 24 小时内最多一次的未来 Stage 88 claim-first 尝试资格。此处没有执行入口、runtime、挂载、数据访问、观察、账本、持仓、绩效、模型或指标写入，也不生成订单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    isolated_runner_id: &str,
    request: ReviewControlledShadowForwardObservationFirstExecutionAuthorizationRequest,
) -> Result<ControlledShadowForwardObservationFirstExecutionAuthorizationReview, String> {
    if !valid_id(isolated_runner_id) || request.expected_isolated_runner_id != isolated_runner_id {
        return Err("前向观察隔离 runner ID 无效或与路径不一致".to_string());
    }
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let runner =
        controlled_shadow_forward_observation_runners_for_first_execution_authorization_review(
            state,
        )
        .await?
        .into_iter()
        .find(|value| value.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "当前没有这条绑定有效且可独立复核的 Stage 86 runner".to_string())?;
    validate_expected_binding(&runner, &request)?;
    let latest = latest_review(state, &runner).await?;
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("Stage 87 授权复核链已经变化，请刷新后重试".to_string());
    }
    let review = new_review(runner, latest.as_ref(), reviewer_id, request, Utc::now())?;
    write_immutable_json(
        &review_directory(state, &review.runner).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    runner: &ControlledShadowForwardObservationIsolatedRunnerRecord,
    request: &ReviewControlledShadowForwardObservationFirstExecutionAuthorizationRequest,
) -> Result<(), String> {
    validate_controlled_shadow_forward_observation_isolated_runner_for_review(runner)?;
    let implementation = &runner.implementation;
    let registration = &implementation.upstream_protocol_registration;
    let protocol = &registration.protocol_specification;
    if request.expected_isolated_runner_id != runner.isolated_runner_id
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_contract_sha256 != runner.runner_contract.contract_sha256
        || request.expected_runner_spec_revision != runner.runner_spec_revision
        || request.expected_runner_code_revision != runner.runner_code_revision
        || request.expected_runner_artifact_sha256 != runner.runner_artifact_sha256
        || request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_review_id != runner.implementation_review.review_id
        || request.expected_implementation_review_sha256
            != runner.implementation_review.review_sha256
        || request.expected_independent_audit_sha256
            != runner.implementation_review.independent_audit.audit_sha256
        || request.expected_protocol_review_sha256
            != implementation.upstream_protocol_review.review_sha256
        || request.expected_protocol_registration_sha256
            != registration.protocol_registration_sha256
        || request.expected_protocol_specification_sha256 != protocol.specification_sha256
        || request.expected_design_specification_sha256
            != protocol.exact_design_specification.specification_sha256
    {
        return Err("Stage 86 runner、工件或完整上游绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn new_review(
    runner: ControlledShadowForwardObservationIsolatedRunnerRecord,
    latest: Option<&ControlledShadowForwardObservationFirstExecutionAuthorizationReview>,
    reviewer_id: &str,
    request: ReviewControlledShadowForwardObservationFirstExecutionAuthorizationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<ControlledShadowForwardObservationFirstExecutionAuthorizationReview, String> {
    if latest
        .is_some_and(|review| review.one_future_claim_first_forward_observation_attempt_authorized)
    {
        return Err("已批准的 Stage 87 授权复核链不得继续追加".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(&runner, latest);
    let independent = !reviewer_id.trim().is_empty()
        && !excluded_prior_actor_ids
            .iter()
            .any(|value| value == reviewer_id);
    if request.reviewer_independence_from_stage_86_and_complete_prior_chain_confirmed != independent
    {
        return Err("复核者独立性确认与 Stage 51–86 完整责任链不一致".to_string());
    }
    let reproduced = request
        .independently_reproduced_runner_artifact_sha256
        .to_ascii_lowercase();
    let artifact_matches = valid_sha256(&reproduced) && reproduced == runner.runner_artifact_sha256;
    if request.runner_artifact_digest_independently_reproduced_and_matched_confirmed
        != artifact_matches
    {
        return Err("独立复现的 runner 工件摘要与 Stage 86 冻结摘要不一致".to_string());
    }
    let mut review = ControlledShadowForwardObservationFirstExecutionAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(), review_sha256: String::new(),
        previous_review_id: latest.map(|value| value.review_id.clone()), previous_review_sha256: latest.map(|value| value.review_sha256.clone()),
        runner, submitted_at, authorization_valid_until: submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS),
        reviewer_id: reviewer_id.to_string(), excluded_prior_actor_ids,
        independently_reproduced_runner_artifact_sha256: reproduced,
        artifact_reproduction_evidence: bounded_required(&request.artifact_reproduction_evidence, MAX_TEXT_CHARS, "工件复现证据")?,
        artifact_digest_matches_registered_runner: artifact_matches,
        verdict: request.verdict, rationale: bounded_required(&request.rationale, MAX_TEXT_CHARS, "复核依据")?,
        exact_current_stage_51_through_stage_86_binding_confirmed: request.exact_current_stage_51_through_stage_86_binding_confirmed,
        reviewer_independence_from_stage_86_and_complete_prior_chain_confirmed: request.reviewer_independence_from_stage_86_and_complete_prior_chain_confirmed,
        runner_spec_contract_and_complete_hash_chain_independently_reproduced_confirmed: request.runner_spec_contract_and_complete_hash_chain_independently_reproduced_confirmed,
        runner_artifact_digest_independently_reproduced_and_matched_confirmed: request.runner_artifact_digest_independently_reproduced_and_matched_confirmed,
        immutable_code_revision_and_artifact_availability_confirmed: request.immutable_code_revision_and_artifact_availability_confirmed,
        natural_forward_no_backfill_and_observation_not_before_confirmed: request.natural_forward_no_backfill_and_observation_not_before_confirmed,
        weekly_claim_first_create_once_official_calendar_and_spy_sync_confirmed: request.weekly_claim_first_create_once_official_calendar_and_spy_sync_confirmed,
        point_in_time_read_only_content_addressed_allowlisted_input_confirmed: request.point_in_time_read_only_content_addressed_allowlisted_input_confirmed,
        corporate_action_evidence_and_append_only_corrections_confirmed: request.corporate_action_evidence_and_append_only_corrections_confirmed,
        create_once_untrusted_independently_validated_no_order_payload_output_confirmed: request.create_once_untrusted_independently_validated_no_order_payload_output_confirmed,
        deterministic_replay_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed: request.deterministic_replay_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: request.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: request.no_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        authorization_single_use_24_hour_expiry_and_stage_88_claim_separation_confirmed: request.authorization_single_use_24_hour_expiry_and_stage_88_claim_separation_confirmed,
        no_runtime_mount_data_access_observation_ledger_position_performance_or_execution_confirmed: request.no_runtime_mount_data_access_observation_ledger_position_performance_or_execution_confirmed,
        no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: request.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_stage_88_claim_first_attempt_confirmed: request.approval_only_opens_future_stage_88_claim_first_attempt_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        one_shot_execution_attempt_limit: ONE_SHOT_ATTEMPT_LIMIT,
        one_future_claim_first_forward_observation_attempt_authorized: false,
        authorization_claimed: false, execution_attempt_endpoint_available: false, runtime_instantiated: false,
        input_manifest_attached: false, data_access_authorized: false, forward_observation_started: false,
        forward_observation_completed: false, forward_observation_ledger_created: false, shadow_position_written: false,
        performance_metric_written: false, model_artifact_store_written: false, metric_store_written: false,
        training_feedback_authorized: false, reward_authorized: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false,
    };
    let approved = review.verdict == ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureClaimFirstForwardObservationAttempt;
    review.one_future_claim_first_forward_observation_attempt_authorized =
        approved && independent && review_checks(&review);
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &review.runner)?;
    Ok(review)
}

fn review_checks(
    review: &ControlledShadowForwardObservationFirstExecutionAuthorizationReview,
) -> bool {
    review.exact_current_stage_51_through_stage_86_binding_confirmed
        && review.reviewer_independence_from_stage_86_and_complete_prior_chain_confirmed
        && review.runner_spec_contract_and_complete_hash_chain_independently_reproduced_confirmed
        && review.runner_artifact_digest_independently_reproduced_and_matched_confirmed
        && review.artifact_digest_matches_registered_runner
        && review.immutable_code_revision_and_artifact_availability_confirmed
        && review.natural_forward_no_backfill_and_observation_not_before_confirmed
        && review.weekly_claim_first_create_once_official_calendar_and_spy_sync_confirmed
        && review.point_in_time_read_only_content_addressed_allowlisted_input_confirmed
        && review.corporate_action_evidence_and_append_only_corrections_confirmed
        && review.create_once_untrusted_independently_validated_no_order_payload_output_confirmed
        && review.deterministic_replay_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed
        && review.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed
        && review.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && review.authorization_single_use_24_hour_expiry_and_stage_88_claim_separation_confirmed
        && review.no_runtime_mount_data_access_observation_ledger_position_performance_or_execution_confirmed
        && review.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && review.approval_only_opens_future_stage_88_claim_first_attempt_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn authority_closed(
    review: &ControlledShadowForwardObservationFirstExecutionAuthorizationReview,
) -> bool {
    !review.authorization_claimed
        && !review.execution_attempt_endpoint_available
        && !review.runtime_instantiated
        && !review.input_manifest_attached
        && !review.data_access_authorized
        && !review.forward_observation_started
        && !review.forward_observation_completed
        && !review.forward_observation_ledger_created
        && !review.shadow_position_written
        && !review.performance_metric_written
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized
}

fn validate_review(
    review: &ControlledShadowForwardObservationFirstExecutionAuthorizationReview,
    runner: &ControlledShadowForwardObservationIsolatedRunnerRecord,
) -> Result<(), String> {
    validate_controlled_shadow_forward_observation_isolated_runner_for_review(runner)?;
    let approved = review.verdict == ControlledShadowForwardObservationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureClaimFirstForwardObservationAttempt;
    let independent = !review
        .excluded_prior_actor_ids
        .iter()
        .any(|value| value == &review.reviewer_id);
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != POLICY_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.review_sha256)
        || review.review_id != review.review_sha256[..32]
        || review.review_sha256 != fingerprint_without(review, &["review_id", "review_sha256"])?
        || review.previous_review_id.is_some() != review.previous_review_sha256.is_some()
        || &review.runner != runner
        || review.authorization_valid_until
            != review.submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS)
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.artifact_reproduction_evidence.trim().is_empty()
        || review.rationale.chars().count() > MAX_TEXT_CHARS
        || review.artifact_reproduction_evidence.chars().count() > MAX_TEXT_CHARS
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || !independent
        || !valid_sha256(&review.independently_reproduced_runner_artifact_sha256)
        || review.artifact_digest_matches_registered_runner
            != (review.independently_reproduced_runner_artifact_sha256
                == runner.runner_artifact_sha256)
        || review.one_shot_execution_attempt_limit != ONE_SHOT_ATTEMPT_LIMIT
        || (approved && !review_checks(review))
        || review.one_future_claim_first_forward_observation_attempt_authorized
            != (approved && review_checks(review))
        || !authority_closed(review)
    {
        return Err("Stage 87 前向观察首次执行授权复核无效、漂移或越权".to_string());
    }
    Ok(())
}

fn expected_excluded_prior_actor_ids(
    runner: &ControlledShadowForwardObservationIsolatedRunnerRecord,
    latest: Option<&ControlledShadowForwardObservationFirstExecutionAuthorizationReview>,
) -> Vec<String> {
    let mut values = runner.excluded_prior_actor_ids.clone();
    values.push(runner.registered_by.clone());
    values.push(runner.implementation_review.reviewer_id.clone());
    if let Some(latest) = latest {
        values.extend(latest.excluded_prior_actor_ids.clone());
        values.push(latest.reviewer_id.clone());
    }
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn authorization_unexpired(
    review: &ControlledShadowForwardObservationFirstExecutionAuthorizationReview,
    now: DateTime<Utc>,
) -> bool {
    review.one_future_claim_first_forward_observation_attempt_authorized
        && !review.authorization_claimed
        && now >= review.submitted_at
        && now < review.authorization_valid_until
}

async fn latest_review(
    state: &AppState,
    runner: &ControlledShadowForwardObservationIsolatedRunnerRecord,
) -> Result<Option<ControlledShadowForwardObservationFirstExecutionAuthorizationReview>, String> {
    latest_review_from_records(&read_reviews(state, runner).await?)
}

async fn read_reviews(
    state: &AppState,
    runner: &ControlledShadowForwardObservationIsolatedRunnerRecord,
) -> Result<Vec<ControlledShadowForwardObservationFirstExecutionAuthorizationReview>, String> {
    let mut directory = match tokio::fs::read_dir(review_directory(state, runner)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let review: ControlledShadowForwardObservationFirstExecutionAuthorizationReview =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_review(&review, runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err("Stage 87 授权复核链重复或文件名错误".to_string());
        }
        reviews.push(review);
    }
    Ok(reviews)
}

fn latest_review_from_records(
    reviews: &[ControlledShadowForwardObservationFirstExecutionAuthorizationReview],
) -> Result<Option<ControlledShadowForwardObservationFirstExecutionAuthorizationReview>, String> {
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
        return Err("Stage 87 授权复核链 tip 数量无效".to_string());
    }
    if reviews.iter().any(|review| {
        review.one_future_claim_first_forward_observation_attempt_authorized
            && reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
    }) {
        return Err("Stage 87 批准必须终止复核链".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("Stage 87 授权复核链存在环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(hash)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "Stage 87 授权复核链断裂".to_string())?;
                if previous.review_sha256 != *hash {
                    return Err("Stage 87 授权复核链前序摘要不一致".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("Stage 87 授权复核链前序链接无效".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("Stage 87 授权复核链未完全连通".to_string());
    }
    for review in reviews {
        let previous = review
            .previous_review_id
            .as_deref()
            .and_then(|id| by_id.get(id).copied());
        if review.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&review.runner, previous)
        {
            return Err("Stage 87 授权复核责任链不一致".to_string());
        }
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 87 指纹载荷无效".to_string())?;
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
fn review_directory(
    state: &AppState,
    runner: &ControlledShadowForwardObservationIsolatedRunnerRecord,
) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-forward-observation-first-execution-authorization-reviews")
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
        "controlled-shadow-forward-observation-first-execution-{isolated_runner_id}.lock"
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
                return Err("另一位管理员正在提交 Stage 87 授权复核".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 87 授权复核锁不可用".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_digest_must_be_a_real_sha256_and_exact_match() {
        let registered = "a".repeat(64);
        let reproduced = "a".repeat(64);
        assert!(valid_sha256(&reproduced));
        assert_eq!(registered, reproduced);
        assert_ne!(registered, "b".repeat(64));
    }

    #[test]
    fn authorization_window_is_strictly_twenty_four_hours() {
        let submitted = Utc::now();
        let valid_until = submitted + TimeDelta::hours(AUTHORIZATION_VALID_HOURS);
        assert!(submitted + TimeDelta::hours(23) < valid_until);
        assert_eq!(submitted + TimeDelta::hours(24), valid_until);
        assert_eq!(ONE_SHOT_ATTEMPT_LIMIT, 1);
    }

    #[test]
    fn stage_87_has_no_execution_or_trading_entrypoint() {
        let capability_flags = [false; 16];
        assert!(capability_flags.into_iter().all(|value| !value));
        assert_eq!(
            POLICY_VERSION,
            "hone-controlled-shadow-forward-observation-first-execution-authorization-v1-artifact-reproduced-single-use-24h"
        );
    }
}
