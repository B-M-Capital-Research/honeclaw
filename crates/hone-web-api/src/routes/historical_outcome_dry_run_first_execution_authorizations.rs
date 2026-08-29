//! Append-only review chain for one exact isolated runner's first execution.
//!
//! An approval is short-lived and grants at most one future invocation of the
//! exact runner artifact. This module deliberately exposes no invocation
//! endpoint and never starts a process, creates an output artifact, writes a
//! label, trains, rewards, shadows, drafts an order or trades.

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

use super::historical_outcome_dry_run_implementations::HistoricalOutcomeDryRunImplementationKind;
use super::historical_outcome_dry_run_isolated_runners::{
    HistoricalOutcomeDryRunIsolatedRunnerKind, HistoricalOutcomeDryRunIsolatedRunnerRecord,
    first_execution_authorization_review_eligible_runners,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-dry-run-first-execution-authorization-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-dry-run-first-execution-authorization-review-v1";
const POLICY_VERSION: &str =
    "hone-historical-outcome-dry-run-first-execution-authorization-v1-single-use-24h";
const AUTHORIZATION_VALID_HOURS: i64 = 24;
const MAX_RATIONALE_CHARS: usize = 2_400;
const ONE_SHOT_INVOCATION_LIMIT: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict {
    ApprovedForOneShotFirstExecution,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeDryRunFirstExecutionAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_run_authorization_review_sha256: String,
    expected_implementation_spec_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    verdict: HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict,
    rationale: String,
    runner_spec_fingerprint_confirmed: bool,
    current_upstream_bindings_confirmed: bool,
    artifact_digest_independently_verified: bool,
    artifact_reproducible_and_available_confirmed: bool,
    sealed_inputs_and_root_read_only_confirmed: bool,
    unprivileged_no_new_privileges_confirmed: bool,
    ephemeral_output_and_validation_confirmed: bool,
    resource_limits_confirmed: bool,
    no_host_environment_or_secrets_confirmed: bool,
    no_network_or_external_tools_confirmed: bool,
    no_production_history_label_training_reward_shadow_writes_confirmed: bool,
    no_order_broker_or_trading_confirmed: bool,
    single_use_and_expiry_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeDryRunFirstExecutionAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub isolated_runner_registered_by: String,
    pub run_authorization_review_id: String,
    pub run_authorization_review_sha256: String,
    pub dry_run_implementation_id: String,
    pub dry_run_implementation_spec_sha256: String,
    pub dry_run_implementation_code_revision: String,
    pub dry_run_implementation_kind: HistoricalOutcomeDryRunImplementationKind,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub labeler_implementation_id: String,
    pub labeler_implementation_spec_sha256: String,
    pub labeler_code_revision: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub runtime_policy_version: String,
    pub runner_name: String,
    pub runner_kind: HistoricalOutcomeDryRunIsolatedRunnerKind,
    pub runner_code_revision: String,
    pub runner_artifact_sha256: String,
    pub runner_status: String,
    pub max_wall_clock_seconds: u16,
    pub max_memory_mib: u16,
    pub max_cpu_millicores: u16,
    pub max_process_count: u8,
    pub max_output_bytes: u32,
    pub submitted_at: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub verdict: HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict,
    pub rationale: String,
    pub runner_spec_fingerprint_confirmed: bool,
    pub current_upstream_bindings_confirmed: bool,
    pub artifact_digest_independently_verified: bool,
    pub artifact_reproducible_and_available_confirmed: bool,
    pub sealed_inputs_and_root_read_only_confirmed: bool,
    pub unprivileged_no_new_privileges_confirmed: bool,
    pub ephemeral_output_and_validation_confirmed: bool,
    pub resource_limits_confirmed: bool,
    pub no_host_environment_or_secrets_confirmed: bool,
    pub no_network_or_external_tools_confirmed: bool,
    pub no_production_history_label_training_reward_shadow_writes_confirmed: bool,
    pub no_order_broker_or_trading_confirmed: bool,
    pub single_use_and_expiry_confirmed: bool,
    pub reviewer_independent_from_runner_registrant: bool,
    pub one_shot_invocation_limit: u8,
    pub one_shot_first_execution_authorized: bool,
    pub authorization_consumed: bool,
    pub invocation_endpoint_available: bool,
    pub offline_dry_run_enabled: bool,
    pub execution_started: bool,
    pub output_artifact_created: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunFirstExecutionAuthorizationItem {
    pub runner: HistoricalOutcomeDryRunIsolatedRunnerRecord,
    pub current_binding: bool,
    pub latest_review: Option<HistoricalOutcomeDryRunFirstExecutionAuthorizationReview>,
    pub one_shot_first_execution_authorized: bool,
    pub authorization_unexpired: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeDryRunFirstExecutionAuthorizationItem>,
    pub review_eligible_runner_count: usize,
    pub reviewed_runner_count: usize,
    pub one_shot_first_execution_authorized_count: usize,
    pub unexpired_authorization_count: usize,
    pub authorization_status: String,
    pub invocation_endpoint_available: bool,
    pub offline_dry_run_enabled: bool,
    pub execution_started: bool,
    pub output_artifact_created: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeDryRunFirstExecutionAuthorizationReadinessSummary {
    pub reviewed_runner_count: usize,
    pub one_shot_first_execution_authorized_count: usize,
    pub unexpired_authorization_count: usize,
    pub authorization_status: String,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization {
    pub runner: HistoricalOutcomeDryRunIsolatedRunnerRecord,
    pub review: HistoricalOutcomeDryRunFirstExecutionAuthorizationReview,
}

pub(crate) async fn handle_get_historical_outcome_dry_run_first_execution_authorizations(
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
            warn!(%error, "historical outcome dry-run first execution authorization registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线试运行首次执行授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_dry_run_first_execution_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeDryRunFirstExecutionAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_first_execution_authorization(&state, &admin.user_id, &isolated_runner_id, request)
        .await
    {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome dry-run first execution authorization registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线试运行首次执行授权复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_dry_run_first_execution_authorization_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunFirstExecutionAuthorizationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeDryRunFirstExecutionAuthorizationReadinessSummary {
            reviewed_runner_count: registry.reviewed_runner_count,
            one_shot_first_execution_authorized_count: registry
                .one_shot_first_execution_authorized_count,
            unexpired_authorization_count: registry.unexpired_authorization_count,
            authorization_status: registry.authorization_status,
            offline_dry_run_enabled: false,
            outcome_label_generation_enabled: false,
        },
    )
}

pub(crate) async fn current_unexpired_first_execution_authorizations(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Vec<ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization>, String> {
    let runners = first_execution_authorization_review_eligible_runners(state).await?;
    let mut approved = Vec::new();
    for runner in runners {
        let Some(review) = latest_review(state, &runner).await? else {
            continue;
        };
        if authorization_unexpired(&review, now) {
            approved.push(ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization {
                runner,
                review,
            });
        }
    }
    approved.sort_by(|left, right| {
        left.runner
            .asset_symbol
            .cmp(&right.runner.asset_symbol)
            .then_with(|| {
                left.runner
                    .isolated_runner_id
                    .cmp(&right.runner.isolated_runner_id)
            })
    });
    Ok(approved)
}

pub(crate) async fn exact_first_execution_authorization_for_audit(
    state: &AppState,
    isolated_runner_id: &str,
    review_id: &str,
    review_sha256: &str,
) -> Result<Option<ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization>, String> {
    let runners = first_execution_authorization_review_eligible_runners(state).await?;
    let Some(runner) = runners
        .into_iter()
        .find(|value| value.isolated_runner_id == isolated_runner_id)
    else {
        return Ok(None);
    };
    let Some(review) = latest_review(state, &runner).await? else {
        return Ok(None);
    };
    if review.review_id != review_id
        || review.review_sha256 != review_sha256
        || !review.one_shot_first_execution_authorized
    {
        return Ok(None);
    }
    Ok(Some(
        ApprovedHistoricalOutcomeDryRunFirstExecutionAuthorization { runner, review },
    ))
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry, String> {
    let runners = first_execution_authorization_review_eligible_runners(state).await?;
    let now = Utc::now();
    let mut items = Vec::with_capacity(runners.len());
    for runner in runners {
        let latest_review = latest_review(state, &runner).await?;
        let authorized = latest_review.as_ref().is_some_and(|review| {
            review.verdict
                == HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution
                && review.one_shot_first_execution_authorized
        });
        let unexpired = latest_review
            .as_ref()
            .is_some_and(|review| authorization_unexpired(review, now));
        items.push(HistoricalOutcomeDryRunFirstExecutionAuthorizationItem {
            runner,
            current_binding: true,
            latest_review,
            one_shot_first_execution_authorized: authorized,
            authorization_unexpired: unexpired,
        });
    }
    let review_eligible_runner_count = items.len();
    let reviewed_runner_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let one_shot_first_execution_authorized_count = items
        .iter()
        .filter(|item| item.one_shot_first_execution_authorized)
        .count();
    let unexpired_authorization_count = items
        .iter()
        .filter(|item| item.one_shot_first_execution_authorized && item.authorization_unexpired)
        .count();
    let authorization_status = if items.is_empty() {
        "waiting_for_current_registered_not_run_isolated_runner"
    } else if unexpired_authorization_count > 0 {
        "approved_for_one_shot_first_execution_not_started"
    } else if reviewed_runner_count > 0 {
        "reviewed_not_currently_authorized"
    } else {
        "waiting_for_independent_first_execution_authorization_review"
    };
    Ok(HistoricalOutcomeDryRunFirstExecutionAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_runner_count,
        reviewed_runner_count,
        one_shot_first_execution_authorized_count,
        unexpired_authorization_count,
        authorization_status: authorization_status.to_string(),
        invocation_endpoint_available: false,
        offline_dry_run_enabled: false,
        execution_started: false,
        output_artifact_created: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "本阶段只对一个精确执行器制品授予 24 小时内、最多一次的未来首次执行资格。审批记录没有调用端点，不启动进程、不创建输出，也不允许结果直接成为标签、训练、奖励、影子、订单或交易证据。".to_string(),
    })
}

async fn review_first_execution_authorization(
    state: &AppState,
    reviewer_id: &str,
    isolated_runner_id: &str,
    request: ReviewHistoricalOutcomeDryRunFirstExecutionAuthorizationRequest,
) -> Result<HistoricalOutcomeDryRunFirstExecutionAuthorizationReview, String> {
    if !valid_id(isolated_runner_id) {
        return Err("隔离执行器 ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-outcome-first-execution-{isolated_runner_id}"),
    )
    .await?;
    let runner = first_execution_authorization_review_eligible_runners(state)
        .await?
        .into_iter()
        .find(|runner| runner.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "当前没有这条绑定有效且可复核的隔离执行器规范".to_string())?;
    if runner.isolated_runner_spec_sha256 != request.expected_isolated_runner_spec_sha256
        || runner.runner_artifact_sha256 != request.expected_runner_artifact_sha256
        || runner.run_authorization_review_sha256
            != request.expected_run_authorization_review_sha256
        || runner.dry_run_implementation_spec_sha256 != request.expected_implementation_spec_sha256
        || runner.snapshot_sha256 != request.expected_snapshot_sha256
        || runner.protocol_sha256 != request.expected_protocol_sha256
    {
        return Err("执行器制品、运行复核或上游证据指纹已经变化，请刷新后重试".to_string());
    }
    let latest = latest_review(state, &runner).await?;
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("首次执行授权复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let submitted_at = Utc::now();
    let mut review = HistoricalOutcomeDryRunFirstExecutionAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        isolated_runner_id: runner.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
        isolated_runner_registered_by: runner.registered_by.clone(),
        run_authorization_review_id: runner.run_authorization_review_id.clone(),
        run_authorization_review_sha256: runner.run_authorization_review_sha256.clone(),
        dry_run_implementation_id: runner.dry_run_implementation_id.clone(),
        dry_run_implementation_spec_sha256: runner.dry_run_implementation_spec_sha256.clone(),
        dry_run_implementation_code_revision: runner.dry_run_implementation_code_revision.clone(),
        dry_run_implementation_kind: runner.dry_run_implementation_kind.clone(),
        snapshot_id: runner.snapshot_id.clone(),
        snapshot_sha256: runner.snapshot_sha256.clone(),
        reconstruction_id: runner.reconstruction_id.clone(),
        reconstruction_sha256: runner.reconstruction_sha256.clone(),
        labeler_implementation_id: runner.labeler_implementation_id.clone(),
        labeler_implementation_spec_sha256: runner.labeler_implementation_spec_sha256.clone(),
        labeler_code_revision: runner.labeler_code_revision.clone(),
        protocol_version: runner.protocol_version.clone(),
        protocol_sha256: runner.protocol_sha256.clone(),
        runtime_policy_version: runner.runtime_policy_version.clone(),
        runner_name: runner.runner_name.clone(),
        runner_kind: runner.runner_kind.clone(),
        runner_code_revision: runner.runner_code_revision.clone(),
        runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
        runner_status: runner.status.clone(),
        max_wall_clock_seconds: runner.max_wall_clock_seconds,
        max_memory_mib: runner.max_memory_mib,
        max_cpu_millicores: runner.max_cpu_millicores,
        max_process_count: runner.max_process_count,
        max_output_bytes: runner.max_output_bytes,
        submitted_at,
        authorization_valid_until: submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS),
        reviewer_id: reviewer_id.to_string(),
        verdict: request.verdict,
        rationale,
        runner_spec_fingerprint_confirmed: request.runner_spec_fingerprint_confirmed,
        current_upstream_bindings_confirmed: request.current_upstream_bindings_confirmed,
        artifact_digest_independently_verified: request.artifact_digest_independently_verified,
        artifact_reproducible_and_available_confirmed: request
            .artifact_reproducible_and_available_confirmed,
        sealed_inputs_and_root_read_only_confirmed: request
            .sealed_inputs_and_root_read_only_confirmed,
        unprivileged_no_new_privileges_confirmed: request.unprivileged_no_new_privileges_confirmed,
        ephemeral_output_and_validation_confirmed: request
            .ephemeral_output_and_validation_confirmed,
        resource_limits_confirmed: request.resource_limits_confirmed,
        no_host_environment_or_secrets_confirmed: request.no_host_environment_or_secrets_confirmed,
        no_network_or_external_tools_confirmed: request.no_network_or_external_tools_confirmed,
        no_production_history_label_training_reward_shadow_writes_confirmed: request
            .no_production_history_label_training_reward_shadow_writes_confirmed,
        no_order_broker_or_trading_confirmed: request.no_order_broker_or_trading_confirmed,
        single_use_and_expiry_confirmed: request.single_use_and_expiry_confirmed,
        reviewer_independent_from_runner_registrant: reviewer_id != runner.registered_by,
        one_shot_invocation_limit: ONE_SHOT_INVOCATION_LIMIT,
        one_shot_first_execution_authorized: false,
        authorization_consumed: false,
        invocation_endpoint_available: false,
        offline_dry_run_enabled: false,
        execution_started: false,
        output_artifact_created: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution;
    review.one_shot_first_execution_authorized = approved
        && review_all_checks(&review)
        && review.reviewer_independent_from_runner_registrant;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &runner)?;
    write_immutable_json(
        &review_directory(state, &runner).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn review_all_checks(review: &HistoricalOutcomeDryRunFirstExecutionAuthorizationReview) -> bool {
    review.runner_spec_fingerprint_confirmed
        && review.current_upstream_bindings_confirmed
        && review.artifact_digest_independently_verified
        && review.artifact_reproducible_and_available_confirmed
        && review.sealed_inputs_and_root_read_only_confirmed
        && review.unprivileged_no_new_privileges_confirmed
        && review.ephemeral_output_and_validation_confirmed
        && review.resource_limits_confirmed
        && review.no_host_environment_or_secrets_confirmed
        && review.no_network_or_external_tools_confirmed
        && review.no_production_history_label_training_reward_shadow_writes_confirmed
        && review.no_order_broker_or_trading_confirmed
        && review.single_use_and_expiry_confirmed
}

fn validate_review(
    review: &HistoricalOutcomeDryRunFirstExecutionAuthorizationReview,
    runner: &HistoricalOutcomeDryRunIsolatedRunnerRecord,
) -> Result<(), String> {
    let approved = review.verdict
        == HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let execution_closed = !review.authorization_consumed
        && !review.invocation_endpoint_available
        && !review.offline_dry_run_enabled
        && !review.execution_started
        && !review.output_artifact_created
        && !review.outcome_label_generation_enabled
        && !review.decision_training_authorized
        && !review.reward_evidence_authorized
        && !review.shadow_evidence_authorized
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
        || review.isolated_runner_id != runner.isolated_runner_id
        || review.isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || review.isolated_runner_registered_by != runner.registered_by
        || review.run_authorization_review_id != runner.run_authorization_review_id
        || review.run_authorization_review_sha256 != runner.run_authorization_review_sha256
        || review.dry_run_implementation_id != runner.dry_run_implementation_id
        || review.dry_run_implementation_spec_sha256 != runner.dry_run_implementation_spec_sha256
        || review.dry_run_implementation_code_revision
            != runner.dry_run_implementation_code_revision
        || review.dry_run_implementation_kind != runner.dry_run_implementation_kind
        || review.snapshot_id != runner.snapshot_id
        || review.snapshot_sha256 != runner.snapshot_sha256
        || review.reconstruction_id != runner.reconstruction_id
        || review.reconstruction_sha256 != runner.reconstruction_sha256
        || review.labeler_implementation_id != runner.labeler_implementation_id
        || review.labeler_implementation_spec_sha256 != runner.labeler_implementation_spec_sha256
        || review.labeler_code_revision != runner.labeler_code_revision
        || review.protocol_version != runner.protocol_version
        || review.protocol_sha256 != runner.protocol_sha256
        || review.runtime_policy_version != runner.runtime_policy_version
        || review.runner_name != runner.runner_name
        || review.runner_kind != runner.runner_kind
        || review.runner_code_revision != runner.runner_code_revision
        || review.runner_artifact_sha256 != runner.runner_artifact_sha256
        || review.runner_status != runner.status
        || review.max_wall_clock_seconds != runner.max_wall_clock_seconds
        || review.max_memory_mib != runner.max_memory_mib
        || review.max_cpu_millicores != runner.max_cpu_millicores
        || review.max_process_count != runner.max_process_count
        || review.max_output_bytes != runner.max_output_bytes
        || review.authorization_valid_until
            != review.submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS)
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || (approved && !review_all_checks(review))
        || (approved && !review.reviewer_independent_from_runner_registrant)
        || review.reviewer_independent_from_runner_registrant
            != (review.reviewer_id != review.isolated_runner_registered_by)
        || review.one_shot_invocation_limit != ONE_SHOT_INVOCATION_LIMIT
        || review.one_shot_first_execution_authorized
            != (approved
                && review_all_checks(review)
                && review.reviewer_independent_from_runner_registrant)
        || !execution_closed
    {
        return Err(
            "historical outcome dry-run first execution authorization is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn authorization_unexpired(
    review: &HistoricalOutcomeDryRunFirstExecutionAuthorizationReview,
    now: DateTime<Utc>,
) -> bool {
    review.one_shot_first_execution_authorized
        && !review.authorization_consumed
        && now >= review.submitted_at
        && now < review.authorization_valid_until
}

fn review_fingerprint(
    review: &HistoricalOutcomeDryRunFirstExecutionAuthorizationReview,
) -> Result<String, String> {
    let mut value = serde_json::to_value(review).map_err(|error| error.to_string())?;
    let object = value.as_object_mut().ok_or_else(|| {
        "first execution authorization fingerprint payload is invalid".to_string()
    })?;
    object.remove("review_id");
    object.remove("review_sha256");
    hash_serializable(&value)
}

async fn latest_review(
    state: &AppState,
    runner: &HistoricalOutcomeDryRunIsolatedRunnerRecord,
) -> Result<Option<HistoricalOutcomeDryRunFirstExecutionAuthorizationReview>, String> {
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
        let review: HistoricalOutcomeDryRunFirstExecutionAuthorizationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, runner)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err(
                "historical outcome first execution authorization filename mismatch".to_string(),
            );
        }
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err(
                "historical outcome first execution authorization contains duplicates".to_string(),
            );
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeDryRunFirstExecutionAuthorizationReview],
) -> Result<Option<HistoricalOutcomeDryRunFirstExecutionAuthorizationReview>, String> {
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
            "historical outcome first execution authorization chain has invalid tips".to_string(),
        );
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err(
                "historical outcome first execution authorization chain has a cycle".to_string(),
            );
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(previous_id), Some(previous_sha256)) => {
                let previous = by_id.get(previous_id.as_str()).ok_or_else(|| {
                    "historical outcome first execution authorization chain is disconnected"
                        .to_string()
                })?;
                if previous.review_sha256 != *previous_sha256 {
                    return Err(
                        "historical outcome first execution authorization previous hash mismatch"
                            .to_string(),
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "historical outcome first execution authorization previous link is invalid"
                        .to_string(),
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err(
            "historical outcome first execution authorization chain is disconnected".to_string(),
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

fn review_directory(
    state: &AppState,
    runner: &HistoricalOutcomeDryRunIsolatedRunnerRecord,
) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-dry-run-first-execution-authorization-reviews")
        .join(&runner.isolated_runner_id)
}

struct HistoricalOutcomeDryRunFirstExecutionAuthorizationLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeDryRunFirstExecutionAuthorizationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<HistoricalOutcomeDryRunFirstExecutionAuthorizationLock, String> {
    if !valid_id(name) {
        return Err(
            "invalid historical outcome first execution authorization lock name".to_string(),
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
            Ok(_) => {
                return Ok(HistoricalOutcomeDryRunFirstExecutionAuthorizationLock { path });
            }
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
                return Err("首次执行授权正在由另一请求复核，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("首次执行授权复核锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid immutable audit path".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
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

fn bounded_required(value: &str, max: usize, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if trimmed.chars().count() > max {
        return Err(format!("{label}无效或过长"));
    }
    Ok(trimmed.to_string())
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn runner() -> HistoricalOutcomeDryRunIsolatedRunnerRecord {
        HistoricalOutcomeDryRunIsolatedRunnerRecord {
            schema_version: "hone-historical-outcome-dry-run-isolated-runner-v1".to_string(),
            isolated_runner_id: "runner-1".to_string(),
            isolated_runner_spec_sha256: "a".repeat(64),
            registered_at: DateTime::parse_from_rfc3339("2026-08-22T14:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            registered_by: "runner-admin".to_string(),
            run_authorization_review_id: "run-review-1".to_string(),
            run_authorization_review_sha256: "b".repeat(64),
            run_authorization_reviewer_id: "run-reviewer".to_string(),
            dry_run_implementation_id: "implementation-1".to_string(),
            dry_run_implementation_spec_sha256: "c".repeat(64),
            dry_run_implementation_code_revision: "oldwang@dryrun123".to_string(),
            dry_run_implementation_kind:
                HistoricalOutcomeDryRunImplementationKind::DeterministicIsolatedCommonSessionReplay,
            authorization_review_id: "authorization-review-1".to_string(),
            snapshot_id: "snapshot-1".to_string(),
            snapshot_sha256: "d".repeat(64),
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "e".repeat(64),
            reconstruction_review_id: "reconstruction-review-1".to_string(),
            labeler_implementation_id: "labeler-1".to_string(),
            labeler_implementation_spec_sha256: "f".repeat(64),
            labeler_implementation_review_id: "labeler-review-1".to_string(),
            labeler_code_revision: "oldwang@labeler123".to_string(),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "1".repeat(64),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            requested_from: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            requested_to: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            asset_series_sha256: "2".repeat(64),
            benchmark_series_sha256: "3".repeat(64),
            common_session_count: 251,
            covered_horizons_market_sessions: vec![20, 60, 250],
            runtime_policy_version:
                "hone-historical-outcome-dry-run-isolated-runtime-v1-non-invoking".to_string(),
            runner_name: "一次性确定性历史结果执行器".to_string(),
            runner_kind: HistoricalOutcomeDryRunIsolatedRunnerKind::EphemeralDeterministicProcess,
            runner_code_revision: "oldwang@runner123".to_string(),
            runner_artifact_sha256: "4".repeat(64),
            status: "registered_not_run".to_string(),
            input_mount_contract: "input".to_string(),
            output_contract: "output".to_string(),
            invocation_contract: "invocation".to_string(),
            callable_entrypoint_registered: false,
            input_mount_read_only_required: true,
            root_filesystem_read_only_required: true,
            ephemeral_working_directory_required: true,
            output_validation_required: true,
            run_as_unprivileged_required: true,
            no_new_privileges_required: true,
            host_environment_inherited: false,
            allowed_environment_variables: Vec::new(),
            secrets_available: false,
            max_wall_clock_seconds: 300,
            max_memory_mib: 512,
            max_cpu_millicores: 1_000,
            max_process_count: 1,
            max_output_bytes: 1_048_576,
            outbound_network_allowed: false,
            external_tools_allowed: false,
            production_writes_allowed: false,
            historical_state_mutation_allowed: false,
            outcome_label_writes_allowed: false,
            training_writes_allowed: false,
            reward_writes_allowed: false,
            shadow_writes_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            invocation_authorized: false,
            offline_dry_run_enabled: false,
            execution_started: false,
            output_artifact_created: false,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        }
    }

    fn review(
        verdict: HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict,
        all_checks: bool,
        reviewer: &str,
    ) -> HistoricalOutcomeDryRunFirstExecutionAuthorizationReview {
        let runner = runner();
        let submitted_at = DateTime::parse_from_rfc3339("2026-08-22T15:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let approved = verdict
            == HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution;
        let independent = reviewer != runner.registered_by;
        let mut value = HistoricalOutcomeDryRunFirstExecutionAuthorizationReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id: None,
            previous_review_sha256: None,
            isolated_runner_id: runner.isolated_runner_id.clone(),
            isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
            isolated_runner_registered_by: runner.registered_by.clone(),
            run_authorization_review_id: runner.run_authorization_review_id.clone(),
            run_authorization_review_sha256: runner.run_authorization_review_sha256.clone(),
            dry_run_implementation_id: runner.dry_run_implementation_id.clone(),
            dry_run_implementation_spec_sha256: runner.dry_run_implementation_spec_sha256.clone(),
            dry_run_implementation_code_revision: runner
                .dry_run_implementation_code_revision
                .clone(),
            dry_run_implementation_kind: runner.dry_run_implementation_kind.clone(),
            snapshot_id: runner.snapshot_id.clone(),
            snapshot_sha256: runner.snapshot_sha256.clone(),
            reconstruction_id: runner.reconstruction_id.clone(),
            reconstruction_sha256: runner.reconstruction_sha256.clone(),
            labeler_implementation_id: runner.labeler_implementation_id.clone(),
            labeler_implementation_spec_sha256: runner.labeler_implementation_spec_sha256.clone(),
            labeler_code_revision: runner.labeler_code_revision.clone(),
            protocol_version: runner.protocol_version.clone(),
            protocol_sha256: runner.protocol_sha256.clone(),
            runtime_policy_version: runner.runtime_policy_version.clone(),
            runner_name: runner.runner_name.clone(),
            runner_kind: runner.runner_kind.clone(),
            runner_code_revision: runner.runner_code_revision.clone(),
            runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
            runner_status: runner.status.clone(),
            max_wall_clock_seconds: runner.max_wall_clock_seconds,
            max_memory_mib: runner.max_memory_mib,
            max_cpu_millicores: runner.max_cpu_millicores,
            max_process_count: runner.max_process_count,
            max_output_bytes: runner.max_output_bytes,
            submitted_at,
            authorization_valid_until: submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS),
            reviewer_id: reviewer.to_string(),
            verdict,
            rationale: "已独立复核制品、上游证据、沙箱和一次性边界。".to_string(),
            runner_spec_fingerprint_confirmed: all_checks,
            current_upstream_bindings_confirmed: all_checks,
            artifact_digest_independently_verified: all_checks,
            artifact_reproducible_and_available_confirmed: all_checks,
            sealed_inputs_and_root_read_only_confirmed: all_checks,
            unprivileged_no_new_privileges_confirmed: all_checks,
            ephemeral_output_and_validation_confirmed: all_checks,
            resource_limits_confirmed: all_checks,
            no_host_environment_or_secrets_confirmed: all_checks,
            no_network_or_external_tools_confirmed: all_checks,
            no_production_history_label_training_reward_shadow_writes_confirmed: all_checks,
            no_order_broker_or_trading_confirmed: all_checks,
            single_use_and_expiry_confirmed: all_checks,
            reviewer_independent_from_runner_registrant: independent,
            one_shot_invocation_limit: ONE_SHOT_INVOCATION_LIMIT,
            one_shot_first_execution_authorized: approved && all_checks && independent,
            authorization_consumed: false,
            invocation_endpoint_available: false,
            offline_dry_run_enabled: false,
            execution_started: false,
            output_artifact_created: false,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        rehash(&mut value);
        value
    }

    fn rehash(review: &mut HistoricalOutcomeDryRunFirstExecutionAuthorizationReview) {
        review.review_sha256 = review_fingerprint(review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
    }

    #[test]
    fn approval_is_single_use_and_time_bounded_but_does_not_execute() {
        let value = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution,
            true,
            "independent-reviewer",
        );
        assert!(validate_review(&value, &runner()).is_ok());
        assert!(value.one_shot_first_execution_authorized);
        assert_eq!(value.one_shot_invocation_limit, 1);
        assert_eq!(
            value.authorization_valid_until - value.submitted_at,
            TimeDelta::hours(24)
        );
        assert!(!value.invocation_endpoint_available);
        assert!(!value.offline_dry_run_enabled);
        assert!(!value.execution_started);
        assert!(!value.output_artifact_created);
    }

    #[test]
    fn runner_registrant_cannot_approve_its_first_execution() {
        let value = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution,
            true,
            "runner-admin",
        );
        assert!(validate_review(&value, &runner()).is_err());
    }

    #[test]
    fn approval_requires_every_check() {
        let value = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution,
            false,
            "independent-reviewer",
        );
        assert!(validate_review(&value, &runner()).is_err());
    }

    #[test]
    fn fingerprint_detects_audit_tampering() {
        let mut value = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ChangesRequested,
            false,
            "reviewer",
        );
        value.rationale = "篡改".to_string();
        assert!(validate_review(&value, &runner()).is_err());
    }

    #[test]
    fn exact_runner_artifact_binding_is_required() {
        let value = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution,
            true,
            "independent-reviewer",
        );
        let mut changed = runner();
        changed.runner_artifact_sha256 = "9".repeat(64);
        assert!(validate_review(&value, &changed).is_err());
    }

    #[test]
    fn chain_requires_exact_previous_hash_and_single_tip() {
        let first = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ChangesRequested,
            false,
            "reviewer-a",
        );
        let mut second = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution,
            true,
            "reviewer-b",
        );
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some(first.review_sha256.clone());
        rehash(&mut second);
        assert_eq!(
            latest_review_from_records(&[first.clone(), second.clone()])
                .unwrap()
                .unwrap()
                .review_id,
            second.review_id
        );
        let mut fork = second;
        fork.reviewer_id = "reviewer-c".to_string();
        rehash(&mut fork);
        assert!(latest_review_from_records(&[first, fork.clone(), fork]).is_err());
    }

    #[test]
    fn output_label_or_trading_authority_is_rejected() {
        let mut value = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution,
            true,
            "independent-reviewer",
        );
        value.execution_started = true;
        value.output_artifact_created = true;
        value.outcome_label_generation_enabled = true;
        value.trading_authorized = true;
        rehash(&mut value);
        assert!(validate_review(&value, &runner()).is_err());
    }

    #[test]
    fn authorization_expires_fail_closed() {
        let value = review(
            HistoricalOutcomeDryRunFirstExecutionAuthorizationVerdict::ApprovedForOneShotFirstExecution,
            true,
            "independent-reviewer",
        );
        assert!(authorization_unexpired(
            &value,
            value.submitted_at + TimeDelta::hours(23)
        ));
        assert!(!authorization_unexpired(
            &value,
            value.authorization_valid_until
        ));
    }
}
