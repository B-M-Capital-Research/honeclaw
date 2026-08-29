//! Stage 70 append-only independent first-execution review chain for one exact sealed-holdout runner.
//!
//! Approval is valid for 24 hours and grants at most one future isolated invocation of the exact
//! content-addressed runner. This module deliberately has no invocation/claim endpoint, starts no
//! process, reads no sealed-holdout data, evaluates or selects no candidate and grants no reward, shadow,
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_isolated_runners::{
    HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    first_execution_authorization_review_eligible_sealed_holdout_evaluation_runners,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-first-execution-authorization-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-first-execution-authorization-review-v1";
const POLICY_VERSION: &str = "hone-historical-outcome-sealed-holdout-evaluation-first-execution-authorization-v1-single-use-24h";
const AUTHORIZATION_VALID_HOURS: i64 = 24;
const ONE_SHOT_INVOCATION_LIMIT: u8 = 1;
const MAX_RATIONALE_CHARS: usize = 2_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict {
    ApprovedForOneFutureIsolatedSealedHoldoutEvaluationInvocation,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_runner_code_revision: String,
    expected_runner_contract_sha256: String,
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_artifact_sha256: String,
    expected_immutable_code_revision: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_implementation_independent_audit_sha256: String,
    expected_candidate_set_sha256: String,
    expected_stage_66_protocol_review_sha256: String,
    expected_sealed_holdout_evaluation_protocol_sha256: String,
    expected_target_bundle_sha256: String,
    expected_recommendation_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    verdict: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict,
    rationale: String,
    exact_runner_and_complete_upstream_binding_confirmed: bool,
    reviewer_independence_from_complete_prior_chain_confirmed: bool,
    runner_artifact_digest_independently_reproduced: bool,
    immutable_code_revision_reproducible_and_artifact_available_confirmed: bool,
    future_exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_confirmed: bool,
    unprivileged_and_no_new_privileges_confirmed: bool,
    ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: bool,
    fixed_runtime_and_resource_limits_confirmed: bool,
    no_host_environment_variables_or_secrets_confirmed: bool,
    no_network_tools_child_process_production_or_history_access_confirmed: bool,
    fixed_one_algorithm_three_seed_sixty_five_feature_one_target_metrics_bootstrap_holm_and_sample_gates_confirmed:
        bool,
    one_shot_sealed_holdout_only_no_training_tuning_reselection_or_feedback_confirmed: bool,
    exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed:
        bool,
    authorization_single_use_and_24_hour_expiry_confirmed: bool,
    authorization_execution_output_validation_and_selection_separation_confirmed: bool,
    no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    pub submitted_at: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict,
    pub rationale: String,
    pub exact_runner_and_complete_upstream_binding_confirmed: bool,
    pub reviewer_independence_from_complete_prior_chain_confirmed: bool,
    pub runner_artifact_digest_independently_reproduced: bool,
    pub immutable_code_revision_reproducible_and_artifact_available_confirmed: bool,
    pub future_exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_confirmed: bool,
    pub unprivileged_and_no_new_privileges_confirmed: bool,
    pub ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed: bool,
    pub fixed_runtime_and_resource_limits_confirmed: bool,
    pub no_host_environment_variables_or_secrets_confirmed: bool,
    pub no_network_tools_child_process_production_or_history_access_confirmed: bool,
    pub fixed_one_algorithm_three_seed_sixty_five_feature_one_target_metrics_bootstrap_holm_and_sample_gates_confirmed:
        bool,
    pub one_shot_sealed_holdout_only_no_training_tuning_reselection_or_feedback_confirmed: bool,
    pub exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed:
        bool,
    pub authorization_single_use_and_24_hour_expiry_confirmed: bool,
    pub authorization_execution_output_validation_and_selection_separation_confirmed: bool,
    pub no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub one_shot_invocation_limit: u8,
    pub one_future_isolated_sealed_holdout_evaluation_invocation_authorized: bool,
    pub authorization_claimed: bool,
    pub invocation_endpoint_available: bool,
    pub sealed_holdout_feature_access_authorized: bool,
    pub sealed_holdout_label_access_authorized: bool,
    pub evaluation_authorized: bool,
    pub evaluation_started: bool,
    pub evaluation_completed: bool,
    pub candidate_selection_authorized: bool,
    pub untrusted_output_created: bool,
    pub output_validation_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub other_target_or_unscoped_sealed_holdout_access_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationItem {
    pub runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    pub current_binding: bool,
    pub latest_review:
        Option<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview>,
    pub one_future_isolated_sealed_holdout_evaluation_invocation_authorized: bool,
    pub authorization_unexpired: bool,
    pub execution_attempt_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationItem>,
    pub review_eligible_runner_count: usize,
    pub reviewed_runner_count: usize,
    pub approved_runner_count: usize,
    pub unexpired_authorization_count: usize,
    pub one_shot_authorized_count: usize,
    pub execution_attempt_eligible_count: usize,
    pub authorization_status: String,
    pub invocation_endpoint_available: bool,
    pub sealed_holdout_feature_access_authorized: bool,
    pub sealed_holdout_label_access_authorized: bool,
    pub evaluation_authorized: bool,
    pub evaluation_started: bool,
    pub evaluation_completed: bool,
    pub candidate_selection_authorized: bool,
    pub untrusted_output_created: bool,
    pub output_validation_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub other_target_or_unscoped_sealed_holdout_access_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReadinessSummary
{
    pub reviewed_runner_count: usize,
    pub approved_runner_count: usize,
    pub unexpired_authorization_count: usize,
    pub one_shot_authorized_count: usize,
    pub execution_attempt_eligible_count: usize,
    pub authorization_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization {
    pub runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    pub review: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview,
}

pub(crate) async fn handle_get_historical_outcome_sealed_holdout_evaluation_first_execution_authorizations(
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
            warn!(%error, "historical outcome sealed holdout evaluation first execution authorization registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果 sealed-holdout 评估首次执行授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_sealed_holdout_evaluation_first_execution_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<
        ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest,
    >,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_first_execution_authorization(&state, &admin.user_id, &isolated_runner_id, request)
        .await
    {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome sealed holdout evaluation first execution authorization registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果 sealed-holdout 评估首次执行授权复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_sealed_holdout_evaluation_first_execution_authorization_readiness_summary(
    state: &AppState,
) -> Result<
    HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReadinessSummary,
    String,
> {
    let registry = build_registry(state, Utc::now()).await?;
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReadinessSummary {
            reviewed_runner_count: registry.reviewed_runner_count,
            approved_runner_count: registry.approved_runner_count,
            unexpired_authorization_count: registry.unexpired_authorization_count,
            one_shot_authorized_count: registry.one_shot_authorized_count,
            execution_attempt_eligible_count: registry.execution_attempt_eligible_count,
            authorization_status: registry.authorization_status,
        },
    )
}

#[allow(dead_code)] // No caller exists until a separately reviewed execution-attempt gate is added.
pub(crate) async fn execution_attempt_eligible_sealed_holdout_evaluation_authorizations(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Vec<ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization>, String>
{
    let runners =
        first_execution_authorization_review_eligible_sealed_holdout_evaluation_runners(state)
            .await?;
    let mut approved = Vec::new();
    for runner in runners {
        let Some(review) = latest_review(state, &runner).await? else {
            continue;
        };
        if authorization_unexpired(&review, now) {
            approved.push(
                ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization {
                    runner,
                    review,
                },
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

/// Reopens the exact current runner and immutable authorization review for a
/// downstream audit. Expiry and consumption deliberately do not invalidate an
/// audit binding: those properties govern invocation, while a validator must
/// still be able to reproduce the chain that produced an immutable result.
#[allow(dead_code)] // Consumed only by a separately gated Stage 71 execution-attempt audit.
pub(crate) async fn exact_first_execution_authorization_for_sealed_holdout_evaluation_audit(
    state: &AppState,
    isolated_runner_id: &str,
    review_id: &str,
    review_sha256: &str,
) -> Result<
    Option<ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization>,
    String,
> {
    let runners =
        first_execution_authorization_review_eligible_sealed_holdout_evaluation_runners(state)
            .await?;
    let Some(runner) = runners
        .into_iter()
        .find(|runner| runner.isolated_runner_id == isolated_runner_id)
    else {
        return Ok(None);
    };
    let directory = review_directory(state, &runner);
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
        let review: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, &runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err(
                "sealed holdout evaluation first execution authorization audit chain is duplicated or misnamed"
                    .to_string(),
            );
        }
        reviews.push(review);
    }
    // Validate the whole append-only chain before accepting one historical
    // authorization as an audit binding.
    let _ = latest_review_from_records(&reviews)?;
    let Some(review) = reviews.into_iter().find(|review| {
        review.review_id == review_id
            && review.review_sha256 == review_sha256
            && review.one_future_isolated_sealed_holdout_evaluation_invocation_authorized
    }) else {
        return Ok(None);
    };
    Ok(Some(
        ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization {
            runner,
            review,
        },
    ))
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry, String> {
    let runners =
        first_execution_authorization_review_eligible_sealed_holdout_evaluation_runners(state)
            .await?;
    let mut items = Vec::with_capacity(runners.len());
    for runner in runners {
        let latest_review = latest_review(state, &runner).await?;
        let authorized = latest_review.as_ref().is_some_and(|review| {
            review.one_future_isolated_sealed_holdout_evaluation_invocation_authorized
        });
        let unexpired = latest_review
            .as_ref()
            .is_some_and(|review| authorization_unexpired(review, now));
        items.push(
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationItem {
                runner,
                current_binding: true,
                latest_review,
                one_future_isolated_sealed_holdout_evaluation_invocation_authorized: authorized,
                authorization_unexpired: unexpired,
                execution_attempt_eligible: unexpired,
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
        .filter(|item| item.one_future_isolated_sealed_holdout_evaluation_invocation_authorized)
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
    let authorization_status = if items.is_empty() {
        "waiting_for_current_registered_not_run_sealed_holdout_evaluation_runner"
    } else if execution_attempt_eligible_count > 0 {
        "approved_for_one_future_isolated_sealed_holdout_evaluation_invocation_not_started"
    } else if reviewed_runner_count > 0 {
        "reviewed_not_currently_authorized"
    } else {
        "waiting_for_independent_first_execution_authorization_review"
    };
    Ok(HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRegistry {
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
        invocation_endpoint_available: false,
        sealed_holdout_feature_access_authorized: false,
        sealed_holdout_label_access_authorized: false,
        evaluation_authorized: false,
        evaluation_started: false,
        evaluation_completed: false,
        candidate_selection_authorized: false,
        untrusted_output_created: false,
        output_validation_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        other_target_or_unscoped_sealed_holdout_access_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 70 阶段只由完整 Stage 51–69 责任链之外的独立复核者，对精确 runner、一个目标与一种算法的 17/29/43 三个候选工件和冻结统计协议授予 24 小时内、最多一次的未来隔离 sealed-holdout 访问与评估调用资格。授权记录本身没有 claim 或调用入口，不挂载或读取 sealed-holdout 特征/标签、不执行评估、不选择候选、不创建输出；模型/指标库、奖励、影子、订单、券商和交易仍关闭。".to_string(),
    })
}

async fn review_first_execution_authorization(
    state: &AppState,
    reviewer_id: &str,
    isolated_runner_id: &str,
    request: ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview, String> {
    if !valid_id(isolated_runner_id) {
        return Err("sealed-holdout 评估隔离 runner ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!(
            "historical-outcome-sealed-holdout-evaluation-first-execution-{isolated_runner_id}"
        ),
    )
    .await?;
    let runner =
        first_execution_authorization_review_eligible_sealed_holdout_evaluation_runners(state)
            .await?
            .into_iter()
            .find(|runner| runner.isolated_runner_id == isolated_runner_id)
            .ok_or_else(|| {
                "当前没有这条绑定有效且可复核的 sealed-holdout 评估隔离 runner".to_string()
            })?;
    validate_expected_binding(&runner, &request)?;
    let latest = latest_review(state, &runner).await?;
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("首次执行授权复核链已经变化，请刷新后重试".to_string());
    }
    let submitted_at = Utc::now();
    let review = new_review(runner, latest.as_ref(), reviewer_id, request, submitted_at)?;
    write_immutable_json(
        &review_directory(state, &review.runner).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn new_review(
    runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    latest: Option<&HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview, String> {
    if latest.is_some_and(|review| {
        review.one_future_isolated_sealed_holdout_evaluation_invocation_authorized
    }) {
        return Err("已批准的首次执行授权复核链不得继续追加".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(&runner, latest);
    let independent = !excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == reviewer_id);
    if request.reviewer_independence_from_complete_prior_chain_confirmed != independent {
        return Err("复核者独立性确认与完整历史角色链不一致".to_string());
    }
    let mut review = HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        runner,
        submitted_at,
        authorization_valid_until: submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        exact_runner_and_complete_upstream_binding_confirmed: request
            .exact_runner_and_complete_upstream_binding_confirmed,
        reviewer_independence_from_complete_prior_chain_confirmed: request
            .reviewer_independence_from_complete_prior_chain_confirmed,
        runner_artifact_digest_independently_reproduced: request
            .runner_artifact_digest_independently_reproduced,
        immutable_code_revision_reproducible_and_artifact_available_confirmed: request
            .immutable_code_revision_reproducible_and_artifact_available_confirmed,
        future_exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_confirmed: request
            .future_exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_confirmed,
        unprivileged_and_no_new_privileges_confirmed: request
            .unprivileged_and_no_new_privileges_confirmed,
        ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed:
            request
                .ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed,
        fixed_runtime_and_resource_limits_confirmed: request
            .fixed_runtime_and_resource_limits_confirmed,
        no_host_environment_variables_or_secrets_confirmed: request
            .no_host_environment_variables_or_secrets_confirmed,
        no_network_tools_child_process_production_or_history_access_confirmed: request
            .no_network_tools_child_process_production_or_history_access_confirmed,
        fixed_one_algorithm_three_seed_sixty_five_feature_one_target_metrics_bootstrap_holm_and_sample_gates_confirmed: request
            .fixed_one_algorithm_three_seed_sixty_five_feature_one_target_metrics_bootstrap_holm_and_sample_gates_confirmed,
        one_shot_sealed_holdout_only_no_training_tuning_reselection_or_feedback_confirmed: request
            .one_shot_sealed_holdout_only_no_training_tuning_reselection_or_feedback_confirmed,
        exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed: request
            .exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed,
        authorization_single_use_and_24_hour_expiry_confirmed: request
            .authorization_single_use_and_24_hour_expiry_confirmed,
        authorization_execution_output_validation_and_selection_separation_confirmed: request
            .authorization_execution_output_validation_and_selection_separation_confirmed,
        no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
            request
                .no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed,
        one_shot_invocation_limit: ONE_SHOT_INVOCATION_LIMIT,
        one_future_isolated_sealed_holdout_evaluation_invocation_authorized: false,
        authorization_claimed: false,
        invocation_endpoint_available: false,
        sealed_holdout_feature_access_authorized: false,
        sealed_holdout_label_access_authorized: false,
        evaluation_authorized: false,
        evaluation_started: false,
        evaluation_completed: false,
        candidate_selection_authorized: false,
        untrusted_output_created: false,
        output_validation_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        other_target_or_unscoped_sealed_holdout_access_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedSealedHoldoutEvaluationInvocation;
    review.one_future_isolated_sealed_holdout_evaluation_invocation_authorized =
        approved && review_all_checks(&review) && independent;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &review.runner)?;
    Ok(review)
}

fn validate_expected_binding(
    runner: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    request: &ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest,
) -> Result<(), String> {
    let implementation_contract = &runner.implementation.implementation_contract;
    let implementation_review = &runner.implementation_review;
    if runner.isolated_runner_spec_sha256 != request.expected_isolated_runner_spec_sha256
        || runner.runner_artifact_sha256 != request.expected_runner_artifact_sha256
        || runner.runner_code_revision != request.expected_runner_code_revision
        || runner.runner_contract.contract_sha256 != request.expected_runner_contract_sha256
        || runner.implementation.implementation_id != request.expected_implementation_id
        || runner.implementation.implementation_sha256 != request.expected_implementation_sha256
        || implementation_contract.contract_sha256
            != request.expected_implementation_contract_sha256
        || implementation_contract.implementation_artifact_sha256
            != request.expected_implementation_artifact_sha256
        || implementation_contract.immutable_code_revision
            != request.expected_immutable_code_revision
        || runner.implementation_review.review_id != request.expected_implementation_review_id
        || runner.implementation_review.review_sha256
            != request.expected_implementation_review_sha256
        || implementation_review.independent_audit.audit_sha256
            != request.expected_implementation_independent_audit_sha256
        || implementation_contract.candidate_set_sha256 != request.expected_candidate_set_sha256
        || implementation_contract.stage_66_protocol_review_sha256
            != request.expected_stage_66_protocol_review_sha256
        || implementation_contract.sealed_holdout_evaluation_protocol_sha256
            != request.expected_sealed_holdout_evaluation_protocol_sha256
        || implementation_contract.target_bundle_sha256 != request.expected_target_bundle_sha256
        || implementation_contract.recommendation_sha256 != request.expected_recommendation_sha256
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
        return Err("runner 工件、代码、沙箱合同或完整上游绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn review_all_checks(
    review: &HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview,
) -> bool {
    review.exact_runner_and_complete_upstream_binding_confirmed
        && review.reviewer_independence_from_complete_prior_chain_confirmed
        && review.runner_artifact_digest_independently_reproduced
        && review.immutable_code_revision_reproducible_and_artifact_available_confirmed
        && review.future_exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_confirmed
        && review.unprivileged_and_no_new_privileges_confirmed
        && review
            .ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed
        && review.fixed_runtime_and_resource_limits_confirmed
        && review.no_host_environment_variables_or_secrets_confirmed
        && review.no_network_tools_child_process_production_or_history_access_confirmed
        && review.fixed_one_algorithm_three_seed_sixty_five_feature_one_target_metrics_bootstrap_holm_and_sample_gates_confirmed
        && review.one_shot_sealed_holdout_only_no_training_tuning_reselection_or_feedback_confirmed
        && review.exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed
        && review.authorization_single_use_and_24_hour_expiry_confirmed
        && review.authorization_execution_output_validation_and_selection_separation_confirmed
        && review
            .no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn validate_review(
    review: &HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview,
    runner: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
) -> Result<(), String> {
    let approved = review.verdict
        == HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedSealedHoldoutEvaluationInvocation;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let expected_excluded = expected_excluded_prior_actor_ids(runner, None);
    let independent = !review
        .excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == &review.reviewer_id);
    let authority_closed = !review.authorization_claimed
        && !review.invocation_endpoint_available
        && !review.sealed_holdout_feature_access_authorized
        && !review.sealed_holdout_label_access_authorized
        && !review.evaluation_authorized
        && !review.evaluation_started
        && !review.evaluation_completed
        && !review.candidate_selection_authorized
        && !review.untrusted_output_created
        && !review.output_validation_authorized
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.other_target_or_unscoped_sealed_holdout_access_authorized
        && !review.reward_authorized
        && !review.shadow_portfolio_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
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
        || &review.runner != runner
        || review.authorization_valid_until
            != review.submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS)
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || !expected_excluded
            .iter()
            .all(|actor| review.excluded_prior_actor_ids.contains(actor))
        || review.reviewer_independence_from_complete_prior_chain_confirmed != independent
        || (approved && !review_all_checks(review))
        || (approved && !independent)
        || review.one_shot_invocation_limit != ONE_SHOT_INVOCATION_LIMIT
        || review.one_future_isolated_sealed_holdout_evaluation_invocation_authorized
            != (approved && review_all_checks(review) && independent)
        || !authority_closed
    {
        return Err("historical outcome sealed holdout evaluation first execution authorization is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn expected_excluded_prior_actor_ids(
    runner: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
    latest: Option<&HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview>,
) -> Vec<String> {
    let mut actors = runner.excluded_prior_actor_ids.clone();
    actors.push(runner.registered_by.clone());
    actors.push(runner.implementation_review.reviewer_id.clone());
    if let Some(latest) = latest {
        actors.extend(latest.excluded_prior_actor_ids.clone());
        actors.push(latest.reviewer_id.clone());
    }
    actors.sort();
    actors.dedup();
    actors
}

fn authorization_unexpired(
    review: &HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview,
    now: DateTime<Utc>,
) -> bool {
    review.one_future_isolated_sealed_holdout_evaluation_invocation_authorized
        && !review.authorization_claimed
        && now >= review.submitted_at
        && now < review.authorization_valid_until
}

fn review_fingerprint(
    review: &HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    runner: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
) -> Result<Option<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview>, String>
{
    let directory = review_directory(state, runner);
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
        let review: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err(
                "sealed holdout evaluation first execution authorization filename mismatch"
                    .to_string(),
            );
        }
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err(
                "sealed holdout evaluation first execution authorization contains duplicates"
                    .to_string(),
            );
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview],
) -> Result<Option<HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview>, String>
{
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
        review.one_future_isolated_sealed_holdout_evaluation_invocation_authorized
            && reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
    }) {
        return Err(
            "sealed holdout evaluation first execution authorization approval must terminate the chain"
                .to_string(),
        );
    }
    if tips.len() != 1 {
        return Err(
            "sealed holdout evaluation first execution authorization chain has invalid tips"
                .to_string(),
        );
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err(
                "sealed holdout evaluation first execution authorization chain has a cycle"
                    .to_string(),
            );
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(previous_id), Some(previous_sha256)) => {
                let previous = by_id.get(previous_id.as_str()).ok_or_else(|| {
                    "sealed holdout evaluation first execution authorization chain is disconnected"
                        .to_string()
                })?;
                if previous.review_sha256 != *previous_sha256 {
                    return Err(
                        "sealed holdout evaluation first execution authorization previous hash mismatch".to_string(),
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "sealed holdout evaluation first execution authorization previous link is invalid"
                        .to_string(),
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err(
            "sealed holdout evaluation first execution authorization chain is disconnected"
                .to_string(),
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
                "sealed holdout evaluation first execution authorization actor chain mismatch"
                    .to_string(),
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
    runner: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-sealed-holdout-evaluation-first-execution-authorization-reviews")
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

async fn acquire_lock(state: &AppState, name: &str) -> Result<ReviewLock, String> {
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(
            "invalid sealed holdout evaluation first execution authorization lock name".to_string(),
        );
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{name}.lock"));
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
                return Err("另一个管理员正在复核该训练 隔离首次执行授权".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("训练隔离首次执行授权复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_isolated_runners::tests::record as runner_record;

    fn request(
        runner: &HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
        verdict: HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict,
        all_checks: bool,
        latest: Option<&HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview>,
    ) -> ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest {
        let implementation_contract = &runner.implementation.implementation_contract;
        let implementation_review = &runner.implementation_review;
        ReviewHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationRequest {
            expected_review_id: latest.map(|value| value.review_id.clone()),
            expected_review_sha256: latest.map(|value| value.review_sha256.clone()),
            expected_isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
            expected_runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
            expected_runner_code_revision: runner.runner_code_revision.clone(),
            expected_runner_contract_sha256: runner.runner_contract.contract_sha256.clone(),
            expected_implementation_id: runner.implementation.implementation_id.clone(),
            expected_implementation_sha256: runner.implementation.implementation_sha256.clone(),
            expected_implementation_contract_sha256: implementation_contract
                .contract_sha256
                .clone(),
            expected_implementation_artifact_sha256: implementation_contract
                .implementation_artifact_sha256
                .clone(),
            expected_immutable_code_revision: implementation_contract
                .immutable_code_revision
                .clone(),
            expected_implementation_review_id: runner.implementation_review.review_id.clone(),
            expected_implementation_review_sha256: implementation_review.review_sha256.clone(),
            expected_implementation_independent_audit_sha256: implementation_review
                .independent_audit
                .audit_sha256
                .clone(),
            expected_candidate_set_sha256: implementation_contract.candidate_set_sha256.clone(),
            expected_stage_66_protocol_review_sha256: implementation_contract
                .stage_66_protocol_review_sha256
                .clone(),
            expected_sealed_holdout_evaluation_protocol_sha256: implementation_contract
                .sealed_holdout_evaluation_protocol_sha256
                .clone(),
            expected_target_bundle_sha256: implementation_contract.target_bundle_sha256.clone(),
            expected_recommendation_sha256: implementation_contract.recommendation_sha256.clone(),
            expected_selected_algorithm_three_seed_binding_sha256: implementation_contract
                .selected_algorithm_three_seed_binding_sha256
                .clone(),
            expected_sealed_holdout_split_commitment_sha256: implementation_contract
                .sealed_holdout_split_commitment_sha256
                .clone(),
            expected_feature_order_sha256: implementation_contract.feature_order_sha256.clone(),
            expected_preprocessing_sha256: implementation_contract.preprocessing_sha256.clone(),
            expected_target_id: implementation_contract.target_id.clone(),
            expected_frozen_candidate_algorithm_id: implementation_contract
                .frozen_candidate_algorithm_id
                .clone(),
            verdict,
            rationale:
                "independently reproduced the exact artifact and verified the closed sandbox"
                    .to_string(),
            exact_runner_and_complete_upstream_binding_confirmed: all_checks,
            reviewer_independence_from_complete_prior_chain_confirmed: true,
            runner_artifact_digest_independently_reproduced: all_checks,
            immutable_code_revision_reproducible_and_artifact_available_confirmed: all_checks,
            future_exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_confirmed: all_checks,
            unprivileged_and_no_new_privileges_confirmed: all_checks,
            ephemeral_content_addressed_create_once_output_and_independent_validation_confirmed:
                all_checks,
            fixed_runtime_and_resource_limits_confirmed: all_checks,
            no_host_environment_variables_or_secrets_confirmed: all_checks,
            no_network_tools_child_process_production_or_history_access_confirmed: all_checks,
            fixed_one_algorithm_three_seed_sixty_five_feature_one_target_metrics_bootstrap_holm_and_sample_gates_confirmed:
                all_checks,
            one_shot_sealed_holdout_only_no_training_tuning_reselection_or_feedback_confirmed:
                all_checks,
            exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed:
                all_checks,
            authorization_single_use_and_24_hour_expiry_confirmed: all_checks,
            authorization_execution_output_validation_and_selection_separation_confirmed:
                all_checks,
            no_data_read_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
                all_checks,
        }
    }

    fn approved_review(
        runner: HistoricalOutcomeSealedHoldoutEvaluationIsolatedRunnerRecord,
        submitted_at: DateTime<Utc>,
    ) -> HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview {
        let request = request(
            &runner,
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedSealedHoldoutEvaluationInvocation,
            true,
            None,
        );
        new_review(
            runner,
            None,
            "independent-first-execution-reviewer",
            request,
            submitted_at,
        )
        .unwrap()
    }

    #[allow(dead_code)] // Fixture for the separately gated Stage 71 module.
    pub(crate) fn approved_authorization()
    -> ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization {
        let submitted_at = DateTime::from_timestamp(1_760_000_200, 0).unwrap();
        let runner = runner_record();
        let review = approved_review(runner.clone(), submitted_at);
        ApprovedHistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorization {
            runner,
            review,
        }
    }

    fn rehash(
        review: &mut HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationReview,
    ) {
        review.review_sha256 = review_fingerprint(review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
    }

    #[test]
    fn approval_is_single_use_24h_and_still_does_not_execute() {
        let submitted_at = DateTime::from_timestamp(1_760_000_200, 0).unwrap();
        let review = approved_review(runner_record(), submitted_at);
        assert!(review.one_future_isolated_sealed_holdout_evaluation_invocation_authorized);
        assert_eq!(review.one_shot_invocation_limit, 1);
        assert_eq!(
            review.authorization_valid_until,
            submitted_at + TimeDelta::hours(24)
        );
        assert!(authorization_unexpired(
            &review,
            submitted_at + TimeDelta::hours(23)
        ));
        assert!(!authorization_unexpired(
            &review,
            submitted_at + TimeDelta::hours(24)
        ));
        assert!(!review.invocation_endpoint_available);
        assert!(!review.authorization_claimed);
        assert!(!review.sealed_holdout_feature_access_authorized);
        assert!(!review.sealed_holdout_label_access_authorized);
        assert!(!review.evaluation_authorized);
        assert!(!review.evaluation_started);
        assert!(!review.candidate_selection_authorized);
        assert!(!review.untrusted_output_created);
    }

    #[test]
    fn every_approval_check_is_mandatory() {
        let runner = runner_record();
        let request = request(
            &runner,
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedSealedHoldoutEvaluationInvocation,
            false,
            None,
        );
        assert!(new_review(runner, None, "independent-reviewer", request, Utc::now()).is_err());
    }

    #[test]
    fn reviewer_must_be_independent_from_complete_prior_chain() {
        let runner = runner_record();
        for actor in expected_excluded_prior_actor_ids(&runner, None) {
            let request = request(
                &runner,
                HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureIsolatedSealedHoldoutEvaluationInvocation,
                true,
                None,
            );
            assert!(new_review(runner.clone(), None, &actor, request, Utc::now()).is_err());
        }
    }

    #[test]
    fn exact_runner_artifact_and_upstream_binding_are_required() {
        let runner = runner_record();
        let mut runner_drift_request = request(
            &runner,
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ChangesRequested,
            false,
            None,
        );
        runner_drift_request.expected_runner_artifact_sha256 = "b".repeat(64);
        assert!(validate_expected_binding(&runner, &runner_drift_request).is_err());
    }

    #[test]
    fn exact_protocol_target_split_and_independent_audits_are_required() {
        let runner = runner_record();
        let mut target_drift_request = request(
            &runner,
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ChangesRequested,
            false,
            None,
        );
        target_drift_request.expected_sealed_holdout_split_commitment_sha256 = "c".repeat(64);
        assert!(validate_expected_binding(&runner, &target_drift_request).is_err());
        let mut audit_drift_request = request(
            &runner,
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ChangesRequested,
            false,
            None,
        );
        audit_drift_request.expected_implementation_independent_audit_sha256 = "d".repeat(64);
        assert!(validate_expected_binding(&runner, &audit_drift_request).is_err());
    }

    #[test]
    fn runner_and_authorization_keep_sealed_holdout_data_closed_until_execution_claim() {
        let runner = runner_record();
        assert!(
            !runner
                .runner_contract
                .sealed_holdout_features_access_allowed
        );
        assert!(!runner.runner_contract.sealed_holdout_labels_access_allowed);
        let review = approved_review(runner, Utc::now());
        assert!(
            review
                .exact_read_only_one_target_sealed_holdout_and_three_candidate_mounts_and_no_other_data_access_confirmed
        );
        assert!(!review.sealed_holdout_label_access_authorized);
        assert!(!review.evaluation_started);
    }

    #[test]
    fn chain_requires_exact_previous_hash_single_tip_and_actor_history() {
        let runner = runner_record();
        let first_request = request(
            &runner,
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ChangesRequested,
            false,
            None,
        );
        let first = new_review(
            runner.clone(),
            None,
            "reviewer-a",
            first_request,
            DateTime::from_timestamp(1_760_000_200, 0).unwrap(),
        )
        .unwrap();
        let second_request = request(
            &runner,
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ChangesRequested,
            false,
            Some(&first),
        );
        let second = new_review(
            runner,
            Some(&first),
            "reviewer-b",
            second_request,
            DateTime::from_timestamp(1_760_000_300, 0).unwrap(),
        )
        .unwrap();
        assert_eq!(
            latest_review_from_records(&[first.clone(), second.clone()])
                .unwrap()
                .unwrap()
                .review_id,
            second.review_id
        );
        let mut fork = second.clone();
        fork.reviewer_id = "reviewer-c".to_string();
        rehash(&mut fork);
        assert!(latest_review_from_records(&[first, second, fork]).is_err());
    }

    #[test]
    fn approval_is_terminal_and_cannot_be_extended() {
        let runner = runner_record();
        let approved = approved_review(runner.clone(), Utc::now());
        let request = request(
            &runner,
            HistoricalOutcomeSealedHoldoutEvaluationFirstExecutionAuthorizationVerdict::ChangesRequested,
            false,
            Some(&approved),
        );
        assert!(
            new_review(
                runner,
                Some(&approved),
                "another-reviewer",
                request,
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn authorization_expiry_cannot_be_extended_by_tampering() {
        let mut review = approved_review(runner_record(), Utc::now());
        review.authorization_valid_until += TimeDelta::hours(1);
        rehash(&mut review);
        assert!(validate_review(&review, &review.runner).is_err());
    }

    #[test]
    fn any_claim_execution_output_or_downstream_authority_tamper_fails_closed() {
        let mut review = approved_review(runner_record(), Utc::now());
        review.authorization_claimed = true;
        rehash(&mut review);
        assert!(validate_review(&review, &review.runner).is_err());
        review.authorization_claimed = false;
        review.untrusted_output_created = true;
        rehash(&mut review);
        assert!(validate_review(&review, &review.runner).is_err());
        review.untrusted_output_created = false;
        review.metric_store_written = true;
        rehash(&mut review);
        assert!(validate_review(&review, &review.runner).is_err());
        review.metric_store_written = false;
        review.other_target_or_unscoped_sealed_holdout_access_authorized = true;
        rehash(&mut review);
        assert!(validate_review(&review, &review.runner).is_err());
    }
}
