//! Stage 110 chain-external first-execution authorization review for one exact Stage 109
//! observation-materialization runner specification.
//!
//! The review is fail-closed: the server must find a read-only regular artifact and a self-hashed
//! reproduction manifest in the runner's derived content-addressed custody directory, then hash the
//! artifact itself. Approval expires after 24 hours and only exposes one future Stage 111
//! claim-first attempt candidate. This module never executes the artifact, mounts or reads a Stage
//! 104 admitted input, creates an observation envelope, or grants portfolio/trading power.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_isolated_runners::{
    ControlledShadowObservationMaterializationIsolatedRunnerRecord,
    observation_materialization_runners_for_first_execution_authorization_review,
    validate_observation_materialization_isolated_runner_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-first-execution-authorization-registry-v1";
const MANIFEST_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-reproduced-artifact-manifest-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-controlled-shadow-observation-materialization-first-execution-authorization-review-v1-server-rehashed-artifact";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-materialization-first-execution-authorization-v1-server-rehashed-single-use-24h";
const AUTHORIZATION_VALID_HOURS: i64 = 24;
const ONE_SHOT_ATTEMPT_LIMIT: u8 = 1;
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const ARTIFACT_FILE_NAME: &str = "runner.artifact";
const MANIFEST_FILE_NAME: &str = "manifest.json";
const NEXT_GATE: &str = "stage_111_claim_first_observation_materialization_execution_attempt";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationMaterializationFirstExecutionAuthorizationVerdict {
    ApprovedForOneFutureClaimFirstObservationMaterializationAttempt,
    ChangesRequestedRebuildArtifact,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationReproducedArtifactManifest {
    pub schema_version: String,
    pub manifest_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_contract_sha256: String,
    pub runner_spec_revision: String,
    pub runner_code_revision: String,
    pub runner_artifact_sha256: String,
    pub artifact_byte_length: u64,
    pub artifact_file_name: String,
    pub artifact_media_type: String,
    pub source_bundle_sha256: String,
    pub artifact_reproduction_procedure_sha256: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub reproduced_at: DateTime<Utc>,
    pub reproduced_by: String,
    pub source_and_artifact_reproduced_from_immutable_revision: bool,
    pub artifact_is_read_only_regular_file: bool,
    pub artifact_was_not_executed: bool,
    pub stage_104_admitted_input_was_not_read: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationArtifactInspection {
    pub custody_locator: String,
    pub manifest_present: bool,
    pub artifact_present: bool,
    pub manifest: Option<ControlledShadowObservationMaterializationReproducedArtifactManifest>,
    pub server_computed_artifact_sha256: Option<String>,
    pub server_observed_artifact_byte_length: Option<u64>,
    pub artifact_verified: bool,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowObservationMaterializationFirstExecutionAuthorizationRequest
{
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
    expected_implementation_contract_sha256: String,
    expected_implementation_review_id: String,
    expected_implementation_review_sha256: String,
    expected_independent_audit_sha256: String,
    expected_specification_review_sha256: String,
    expected_specification_registration_sha256: String,
    expected_observation_materialization_specification_sha256: String,
    expected_stage_104_admission_review_sha256: String,
    expected_stage_103_validation_sha256: String,
    expected_stage_102_result_sha256: String,
    expected_stage_102_output_sha256: String,
    expected_stage_101_claim_sha256: String,
    expected_stage_101_input_manifest_sha256: String,
    expected_cycle_claim_sha256: String,
    expected_artifact_manifest_sha256: String,
    artifact_reproduction_review_evidence: String,
    sandbox_contract_review_evidence: String,
    verdict: ControlledShadowObservationMaterializationFirstExecutionAuthorizationVerdict,
    rationale: String,
    exact_current_stage_51_through_stage_109_binding_confirmed: bool,
    reviewer_independent_from_stage_109_builder_and_complete_prior_chain_confirmed: bool,
    server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: bool,
    self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: bool,
    artifact_builder_and_reviewer_separation_confirmed: bool,
    all_eight_observation_materialization_functions_and_canonical_schemas_remain_bound_confirmed:
        bool,
    session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed:
        bool,
    no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed:
        bool,
    fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed: bool,
    future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    provider_publication_time_remains_unverified_until_separate_evidence_confirmed: bool,
    authorization_single_use_24_hour_expiry_and_stage_111_claim_separation_confirmed: bool,
    no_runtime_entrypoint_mount_input_read_observation_materialization_execution_or_observations_confirmed:
        bool,
    no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    approval_only_opens_future_stage_111_claim_first_attempt_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub runner: ControlledShadowObservationMaterializationIsolatedRunnerRecord,
    pub artifact_manifest: ControlledShadowObservationMaterializationReproducedArtifactManifest,
    pub submitted_at: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub server_computed_artifact_sha256: String,
    pub server_observed_artifact_byte_length: u64,
    pub artifact_reproduction_review_evidence: String,
    pub sandbox_contract_review_evidence: String,
    pub verdict: ControlledShadowObservationMaterializationFirstExecutionAuthorizationVerdict,
    pub rationale: String,
    pub exact_current_stage_51_through_stage_109_binding_confirmed: bool,
    pub reviewer_independent_from_stage_109_builder_and_complete_prior_chain_confirmed: bool,
    pub server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: bool,
    pub self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: bool,
    pub artifact_builder_and_reviewer_separation_confirmed: bool,
    pub all_eight_observation_materialization_functions_and_canonical_schemas_remain_bound_confirmed:
        bool,
    pub session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed:
        bool,
    pub no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    pub future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed: bool,
    pub future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    pub provider_publication_time_remains_unverified_until_separate_evidence_confirmed: bool,
    pub authorization_single_use_24_hour_expiry_and_stage_111_claim_separation_confirmed: bool,
    pub no_runtime_entrypoint_mount_input_read_observation_materialization_execution_or_observations_confirmed:
        bool,
    pub no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    pub no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub approval_only_opens_future_stage_111_claim_first_attempt_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub one_shot_execution_attempt_limit: u8,
    pub one_future_claim_first_observation_materialization_attempt_authorized: bool,
    pub authorization_claimed: bool,
    pub execution_attempt_endpoint_available: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub input_read: bool,
    pub observation_materialization_executed: bool,
    pub sessions_materialized: bool,
    pub price_observations_materialized: bool,
    pub observation_materialized: bool,
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
pub(crate) struct ControlledShadowObservationMaterializationFirstExecutionAuthorizationItem {
    pub runner: ControlledShadowObservationMaterializationIsolatedRunnerRecord,
    pub artifact_inspection: ControlledShadowObservationMaterializationArtifactInspection,
    pub latest_review:
        Option<ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview>,
    pub authorization_unexpired: bool,
    pub future_claim_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationFirstExecutionAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowObservationMaterializationFirstExecutionAuthorizationItem>,
    pub runner_count: usize,
    pub artifact_verified_runner_count: usize,
    pub artifact_pending_runner_count: usize,
    pub review_eligible_runner_count: usize,
    pub reviewed_runner_count: usize,
    pub approved_runner_count: usize,
    pub unexpired_authorization_count: usize,
    pub one_shot_authorized_count: usize,
    pub future_claim_eligible_count: usize,
    pub authorization_status: String,
    pub next_gate: String,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub input_read: bool,
    pub observation_materialization_executed: bool,
    pub sessions_materialized: bool,
    pub price_observations_materialized: bool,
    pub observation_materialized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationMaterializationFirstExecutionAuthorizationReadinessSummary
{
    pub runner_count: usize,
    pub artifact_verified_runner_count: usize,
    pub artifact_pending_runner_count: usize,
    pub review_eligible_runner_count: usize,
    pub reviewed_runner_count: usize,
    pub approved_runner_count: usize,
    pub unexpired_authorization_count: usize,
    pub one_shot_authorized_count: usize,
    pub future_claim_eligible_count: usize,
    pub authorization_status: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ApprovedControlledShadowObservationMaterializationFirstExecutionAuthorization {
    pub runner: ControlledShadowObservationMaterializationIsolatedRunnerRecord,
    pub review: ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview,
}

pub(crate) async fn handle_get_controlled_shadow_observation_materialization_first_execution_authorizations(
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
            warn!(%error, "observation materialization first-execution authorization registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察物化首次执行授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_observation_materialization_first_execution_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<
        ReviewControlledShadowObservationMaterializationFirstExecutionAuthorizationRequest,
    >,
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

pub(crate) async fn controlled_shadow_observation_materialization_first_execution_authorization_readiness_summary(
    state: &AppState,
) -> Result<
    ControlledShadowObservationMaterializationFirstExecutionAuthorizationReadinessSummary,
    String,
> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowObservationMaterializationFirstExecutionAuthorizationReadinessSummary {
            runner_count: value.runner_count,
            artifact_verified_runner_count: value.artifact_verified_runner_count,
            artifact_pending_runner_count: value.artifact_pending_runner_count,
            review_eligible_runner_count: value.review_eligible_runner_count,
            reviewed_runner_count: value.reviewed_runner_count,
            approved_runner_count: value.approved_runner_count,
            unexpired_authorization_count: value.unexpired_authorization_count,
            one_shot_authorized_count: value.one_shot_authorized_count,
            future_claim_eligible_count: value.future_claim_eligible_count,
            authorization_status: value.authorization_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn observation_materialization_first_execution_authorizations_for_future_claim(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<
    Vec<ApprovedControlledShadowObservationMaterializationFirstExecutionAuthorization>,
    String,
> {
    let registry = build_registry(state, now).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            if !item.future_claim_eligible {
                return None;
            }
            item.latest_review.map(|review| {
                ApprovedControlledShadowObservationMaterializationFirstExecutionAuthorization {
                    runner: item.runner,
                    review,
                }
            })
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowObservationMaterializationFirstExecutionAuthorizationRegistry, String> {
    let runners =
        observation_materialization_runners_for_first_execution_authorization_review(state).await?;
    let claimed_review_ids = super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_execution_attempt_claims::claimed_observation_materialization_first_execution_authorization_review_ids(state).await?;
    let mut items = Vec::with_capacity(runners.len());
    for runner in runners {
        let artifact_inspection = inspect_artifact(state, &runner).await?;
        let latest_review = latest_review(state, &runner).await?;
        let authorization_unexpired = latest_review.as_ref().is_some_and(|review| {
            authorization_unexpired(review, now)
                && artifact_inspection_matches_review(&artifact_inspection, review)
        });
        let authorization_claimed = latest_review
            .as_ref()
            .is_some_and(|review| claimed_review_ids.contains(&review.review_id));
        items.push(
            ControlledShadowObservationMaterializationFirstExecutionAuthorizationItem {
                runner,
                artifact_inspection,
                latest_review,
                authorization_unexpired,
                future_claim_eligible: authorization_unexpired && !authorization_claimed,
            },
        );
    }
    let runner_count = items.len();
    let artifact_verified_runner_count = items
        .iter()
        .filter(|item| item.artifact_inspection.artifact_verified)
        .count();
    let artifact_pending_runner_count = runner_count.saturating_sub(artifact_verified_runner_count);
    let review_eligible_runner_count = artifact_verified_runner_count;
    let reviewed_runner_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let approved_runner_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.one_future_claim_first_observation_materialization_attempt_authorized
            })
        })
        .count();
    let unexpired_authorization_count = items
        .iter()
        .filter(|item| item.authorization_unexpired)
        .count();
    let future_claim_eligible_count = items
        .iter()
        .filter(|item| item.future_claim_eligible)
        .count();
    let authorization_status = if runner_count == 0 {
        "waiting_for_current_stage_109_observation_materialization_isolated_runner"
    } else if future_claim_eligible_count > 0 {
        "approved_for_one_future_stage_111_claim_first_attempt_not_started"
    } else if artifact_verified_runner_count == 0 {
        "waiting_for_server_verifiable_reproduced_runner_artifact"
    } else if reviewed_runner_count > 0 {
        "reviewed_not_currently_authorized"
    } else {
        "waiting_for_independent_first_observation_materialization_execution_authorization_review"
    };
    Ok(
        ControlledShadowObservationMaterializationFirstExecutionAuthorizationRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            items,
            runner_count,
            artifact_verified_runner_count,
            artifact_pending_runner_count,
            review_eligible_runner_count,
            reviewed_runner_count,
            approved_runner_count,
            unexpired_authorization_count,
            one_shot_authorized_count: unexpired_authorization_count,
            future_claim_eligible_count,
            authorization_status: authorization_status.to_string(),
            next_gate: NEXT_GATE.to_string(),
            callable_entrypoint_present: false,
            runtime_instantiated: false,
            input_mount_present: false,
            input_read: false,
            observation_materialization_executed: false,
            sessions_materialized: false,
            price_observations_materialized: false,
            observation_materialized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "Stage 110 只允许完整 Stage 51–109 责任链之外的新复核者，在服务端看到只读常规工件及自哈希 manifest、重新计算工件 SHA-256 并核对代码版本、复现步骤和隔离合同后，授予 24 小时内最多一次的未来 Stage 111 claim-first 资格。当前没有入口、runtime、载荷挂载/读取、观察物化执行、观察输出、观察、账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易能力。".to_string(),
        },
    )
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    isolated_runner_id: &str,
    request: ReviewControlledShadowObservationMaterializationFirstExecutionAuthorizationRequest,
) -> Result<ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview, String> {
    if !valid_id(isolated_runner_id) || request.expected_isolated_runner_id != isolated_runner_id {
        return Err("Stage 109 隔离 runner ID 无效或与路径不一致".to_string());
    }
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let runner =
        observation_materialization_runners_for_first_execution_authorization_review(state)
            .await?
            .into_iter()
            .find(|value| value.isolated_runner_id == isolated_runner_id)
            .ok_or_else(|| "当前没有这条绑定有效且可独立复核的 Stage 109 runner".to_string())?;
    validate_expected_binding(&runner, &request)?;
    let artifact = inspect_artifact(state, &runner).await?;
    if !artifact.artifact_verified {
        return Err("服务端尚未找到并核验只读内容寻址 runner 工件与 manifest".to_string());
    }
    let manifest = artifact
        .manifest
        .clone()
        .ok_or_else(|| "已核验工件缺少 manifest".to_string())?;
    if request.expected_artifact_manifest_sha256 != manifest.manifest_sha256 {
        return Err("工件 manifest 已变化，请刷新后重试".to_string());
    }
    let latest = latest_review(state, &runner).await?;
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("Stage 110 授权复核链已经变化，请刷新后重试".to_string());
    }
    let review = new_review(
        runner,
        manifest,
        artifact
            .server_computed_artifact_sha256
            .ok_or_else(|| "缺少服务端工件摘要".to_string())?,
        artifact
            .server_observed_artifact_byte_length
            .ok_or_else(|| "缺少服务端工件长度".to_string())?,
        latest.as_ref(),
        reviewer_id,
        request,
        Utc::now(),
    )?;
    write_immutable_json(
        &review_directory(state, &review.runner).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
    request: &ReviewControlledShadowObservationMaterializationFirstExecutionAuthorizationRequest,
) -> Result<(), String> {
    validate_observation_materialization_isolated_runner_for_review(runner)?;
    let contract = &runner.runner_contract;
    let implementation = &runner.implementation;
    let implementation_contract = &implementation.implementation_contract;
    let spec_review = &implementation.upstream_specification_review;
    let registration = &implementation.upstream_specification_registration;
    let specification = &implementation_contract.exact_observation_materialization_specification;
    if request.expected_isolated_runner_id != runner.isolated_runner_id
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_contract_sha256 != contract.contract_sha256
        || request.expected_runner_spec_revision != contract.runner_spec_revision
        || request.expected_runner_code_revision != contract.proposed_runner_code_revision
        || request.expected_runner_artifact_sha256 != contract.proposed_runner_artifact_sha256
        || request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256
            != implementation_contract.contract_sha256
        || request.expected_implementation_review_id != runner.implementation_review.review_id
        || request.expected_implementation_review_sha256
            != runner.implementation_review.review_sha256
        || request.expected_independent_audit_sha256
            != runner.implementation_review.independent_audit.audit_sha256
        || request.expected_specification_review_sha256 != spec_review.review_sha256
        || request.expected_specification_registration_sha256 != registration.registration_sha256
        || request.expected_observation_materialization_specification_sha256
            != registration.specification.specification_sha256
        || request.expected_stage_104_admission_review_sha256
            != specification.stage_104_review_sha256
        || request.expected_stage_103_validation_sha256 != specification.stage_103_validation_sha256
        || request.expected_stage_102_result_sha256 != specification.stage_102_result_sha256
        || request.expected_stage_102_output_sha256 != specification.stage_102_output_sha256
        || request.expected_stage_101_claim_sha256 != specification.stage_101_claim_sha256
        || request.expected_stage_101_input_manifest_sha256
            != specification.stage_101_input_manifest_sha256
        || request.expected_cycle_claim_sha256 != specification.cycle_claim_sha256
    {
        return Err("Stage 101–109 runner、工件身份或完整上游绑定已经变化".to_string());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn new_review(
    runner: ControlledShadowObservationMaterializationIsolatedRunnerRecord,
    artifact_manifest: ControlledShadowObservationMaterializationReproducedArtifactManifest,
    server_computed_artifact_sha256: String,
    server_observed_artifact_byte_length: u64,
    latest: Option<&ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview>,
    reviewer_id: &str,
    request: ReviewControlledShadowObservationMaterializationFirstExecutionAuthorizationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview, String> {
    if latest.is_some_and(|review| {
        review.one_future_claim_first_observation_materialization_attempt_authorized
    }) {
        return Err("已批准的 Stage 110 授权复核链不得继续追加".to_string());
    }
    let excluded_prior_actor_ids =
        expected_excluded_prior_actor_ids(&runner, &artifact_manifest, latest);
    let independent = !reviewer_id.trim().is_empty()
        && !excluded_prior_actor_ids
            .iter()
            .any(|value| value == reviewer_id);
    if request.reviewer_independent_from_stage_109_builder_and_complete_prior_chain_confirmed
        != independent
    {
        return Err("复核者独立性确认与 Stage 51–109 及工件构建者责任链不一致".to_string());
    }
    let builder_separated = artifact_manifest.reproduced_by != reviewer_id;
    if request.artifact_builder_and_reviewer_separation_confirmed != builder_separated {
        return Err("工件构建者与 Stage 110 复核者分离确认不一致".to_string());
    }
    let mut review = ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.map(|value| value.review_id.clone()),
        previous_review_sha256: latest.map(|value| value.review_sha256.clone()),
        runner,
        artifact_manifest,
        submitted_at,
        authorization_valid_until: submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        server_computed_artifact_sha256,
        server_observed_artifact_byte_length,
        artifact_reproduction_review_evidence: bounded_required(
            &request.artifact_reproduction_review_evidence,
            MAX_TEXT_CHARS,
            "工件复现复核证据",
        )?,
        sandbox_contract_review_evidence: bounded_required(
            &request.sandbox_contract_review_evidence,
            MAX_TEXT_CHARS,
            "隔离合同复核证据",
        )?,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, MAX_TEXT_CHARS, "复核依据")?,
        exact_current_stage_51_through_stage_109_binding_confirmed: request
            .exact_current_stage_51_through_stage_109_binding_confirmed,
        reviewer_independent_from_stage_109_builder_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_stage_109_builder_and_complete_prior_chain_confirmed,
        server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: request
            .server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed,
        self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: request
            .self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed,
        artifact_builder_and_reviewer_separation_confirmed: request
            .artifact_builder_and_reviewer_separation_confirmed,
        all_eight_observation_materialization_functions_and_canonical_schemas_remain_bound_confirmed: request
            .all_eight_observation_materialization_functions_and_canonical_schemas_remain_bound_confirmed,
        session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed: request
            .session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed,
        no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed: request
            .no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
            request.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed,
        future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed:
            request.future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed,
        future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
            request.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed,
        provider_publication_time_remains_unverified_until_separate_evidence_confirmed: request
            .provider_publication_time_remains_unverified_until_separate_evidence_confirmed,
        authorization_single_use_24_hour_expiry_and_stage_111_claim_separation_confirmed: request
            .authorization_single_use_24_hour_expiry_and_stage_111_claim_separation_confirmed,
        no_runtime_entrypoint_mount_input_read_observation_materialization_execution_or_observations_confirmed: request
            .no_runtime_entrypoint_mount_input_read_observation_materialization_execution_or_observations_confirmed,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: request
            .no_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
            request.no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_stage_111_claim_first_attempt_confirmed: request
            .approval_only_opens_future_stage_111_claim_first_attempt_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        one_shot_execution_attempt_limit: ONE_SHOT_ATTEMPT_LIMIT,
        one_future_claim_first_observation_materialization_attempt_authorized: false,
        authorization_claimed: false,
        execution_attempt_endpoint_available: false,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        input_mount_present: false,
        input_read: false,
        observation_materialization_executed: false,
        sessions_materialized: false,
        price_observations_materialized: false,
        observation_materialized: false,
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
    if review.artifact_manifest.reproduced_at > submitted_at {
        return Err("工件 manifest 的复现时间不能晚于复核时间".to_string());
    }
    let approved = review.verdict
        == ControlledShadowObservationMaterializationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureClaimFirstObservationMaterializationAttempt;
    review.one_future_claim_first_observation_materialization_attempt_authorized =
        approved && independent && review_checks(&review);
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &review.runner)?;
    Ok(review)
}

fn review_checks(
    review: &ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview,
) -> bool {
    review.exact_current_stage_51_through_stage_109_binding_confirmed
        && review.reviewer_independent_from_stage_109_builder_and_complete_prior_chain_confirmed
        && review.server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed
        && review.self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed
        && review.artifact_builder_and_reviewer_separation_confirmed
        && review.all_eight_observation_materialization_functions_and_canonical_schemas_remain_bound_confirmed
        && review.session_price_basis_gap_action_allocation_availability_and_failure_semantics_preserved_confirmed
        && review.no_overwrite_backfill_forward_fill_interpolation_substitution_or_inferred_actions_confirmed
        && review.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed
        && review.future_input_only_stage_104_admitted_read_only_content_addressed_output_confirmed
        && review.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && review.provider_publication_time_remains_unverified_until_separate_evidence_confirmed
        && review.authorization_single_use_24_hour_expiry_and_stage_111_claim_separation_confirmed
        && review.no_runtime_entrypoint_mount_input_read_observation_materialization_execution_or_observations_confirmed
        && review.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && review.no_observation_envelope_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && review.approval_only_opens_future_stage_111_claim_first_attempt_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn authority_closed(
    review: &ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview,
) -> bool {
    !review.authorization_claimed
        && !review.execution_attempt_endpoint_available
        && !review.callable_entrypoint_present
        && !review.runtime_instantiated
        && !review.input_mount_present
        && !review.input_read
        && !review.observation_materialization_executed
        && !review.sessions_materialized
        && !review.price_observations_materialized
        && !review.observation_materialized
        && !review.ledger_created
        && !review.position_written
        && !review.performance_metric_written
        && !review.model_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized
}

#[allow(dead_code)]
pub(crate) fn validate_observation_materialization_first_execution_authorization_for_claim(
    review: &ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview,
) -> Result<(), String> {
    validate_review(review, &review.runner)
}

fn validate_review(
    review: &ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview,
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
) -> Result<(), String> {
    validate_observation_materialization_isolated_runner_for_review(runner)?;
    validate_manifest(&review.artifact_manifest, runner)?;
    let approved = review.verdict
        == ControlledShadowObservationMaterializationFirstExecutionAuthorizationVerdict::ApprovedForOneFutureClaimFirstObservationMaterializationAttempt;
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
        || review.artifact_manifest.reproduced_at > review.submitted_at
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review
            .artifact_reproduction_review_evidence
            .trim()
            .is_empty()
        || review.sandbox_contract_review_evidence.trim().is_empty()
        || review.rationale.chars().count() > MAX_TEXT_CHARS
        || review.artifact_reproduction_review_evidence.chars().count() > MAX_TEXT_CHARS
        || review.sandbox_contract_review_evidence.chars().count() > MAX_TEXT_CHARS
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || !independent
        || review.artifact_manifest.reproduced_by == review.reviewer_id
        || review.server_computed_artifact_sha256 != review.artifact_manifest.runner_artifact_sha256
        || review.server_observed_artifact_byte_length
            != review.artifact_manifest.artifact_byte_length
        || review.one_shot_execution_attempt_limit != ONE_SHOT_ATTEMPT_LIMIT
        || (approved && !review_checks(review))
        || review.one_future_claim_first_observation_materialization_attempt_authorized
            != (approved && review_checks(review))
        || !authority_closed(review)
    {
        return Err("Stage 110 观察物化首次执行授权复核无效、漂移或越权".to_string());
    }
    Ok(())
}

async fn inspect_artifact(
    state: &AppState,
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
) -> Result<ControlledShadowObservationMaterializationArtifactInspection, String> {
    let directory = artifact_directory(state, runner);
    let custody_locator = format!(
        "controlled-shadow-observation-materialization-reproduced-artifacts/{}/{}",
        runner.isolated_runner_id, runner.runner_contract.proposed_runner_artifact_sha256
    );
    let manifest_path = directory.join(MANIFEST_FILE_NAME);
    let artifact_path = directory.join(ARTIFACT_FILE_NAME);
    let manifest_metadata = match tokio::fs::symlink_metadata(&manifest_path).await {
        Ok(value) => Some(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.to_string()),
    };
    let artifact_metadata = match tokio::fs::symlink_metadata(&artifact_path).await {
        Ok(value) => Some(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.to_string()),
    };
    let artifact_present = artifact_metadata.is_some();
    let Some(manifest_metadata) = manifest_metadata else {
        return Ok(pending_inspection(
            custody_locator,
            false,
            artifact_present,
            "reproduction_manifest_missing",
        ));
    };
    let Some(artifact_metadata) = artifact_metadata else {
        return Ok(pending_inspection(
            custody_locator,
            true,
            false,
            "reproduced_artifact_missing",
        ));
    };
    if !safe_read_only_regular_file(&manifest_metadata, MAX_MANIFEST_BYTES)
        || !safe_read_only_regular_file(&artifact_metadata, MAX_ARTIFACT_BYTES)
    {
        return Ok(pending_inspection(
            custody_locator,
            true,
            true,
            "artifact_or_manifest_not_read_only_regular_file",
        ));
    }
    let manifest_bytes = tokio::fs::read(&manifest_path)
        .await
        .map_err(|error| error.to_string())?;
    let manifest: ControlledShadowObservationMaterializationReproducedArtifactManifest =
        match serde_json::from_slice(&manifest_bytes) {
            Ok(value) => value,
            Err(_) => {
                return Ok(pending_inspection(
                    custody_locator,
                    true,
                    true,
                    "reproduction_manifest_invalid_json",
                ));
            }
        };
    if validate_manifest(&manifest, runner).is_err() {
        return Ok(
            ControlledShadowObservationMaterializationArtifactInspection {
                custody_locator,
                manifest_present: true,
                artifact_present: true,
                manifest: Some(manifest),
                server_computed_artifact_sha256: None,
                server_observed_artifact_byte_length: Some(artifact_metadata.len()),
                artifact_verified: false,
                status: "reproduction_manifest_binding_invalid".to_string(),
            },
        );
    }
    let artifact_bytes = tokio::fs::read(&artifact_path)
        .await
        .map_err(|error| error.to_string())?;
    let artifact_sha256 = sha256_bytes(&artifact_bytes);
    let artifact_verified = artifact_sha256 == manifest.runner_artifact_sha256
        && artifact_metadata.len() == manifest.artifact_byte_length;
    Ok(
        ControlledShadowObservationMaterializationArtifactInspection {
            custody_locator,
            manifest_present: true,
            artifact_present: true,
            manifest: Some(manifest),
            server_computed_artifact_sha256: Some(artifact_sha256),
            server_observed_artifact_byte_length: Some(artifact_metadata.len()),
            artifact_verified,
            status: if artifact_verified {
                "server_rehashed_artifact_verified_not_executed"
            } else {
                "artifact_digest_or_length_mismatch"
            }
            .to_string(),
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn read_revalidated_observation_materialization_artifact_for_execution(
    state: &AppState,
    review: &ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview,
) -> Result<Vec<u8>, String> {
    validate_observation_materialization_first_execution_authorization_for_claim(review)?;
    let inspection = inspect_artifact(state, &review.runner).await?;
    if !artifact_inspection_matches_review(&inspection, review) {
        return Err("Stage 110 工件或 manifest 在未来 Stage 112 执行前复核失败".to_string());
    }
    let path = artifact_directory(state, &review.runner).join(ARTIFACT_FILE_NAME);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|_| "未来 Stage 112 无法重新打开固定观察物化工件".to_string())?;
    if !safe_read_only_regular_file(&metadata, MAX_ARTIFACT_BYTES) {
        return Err("未来 Stage 112 固定观察物化工件不再是只读常规文件".to_string());
    }
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|_| "未来 Stage 112 无法读取固定观察物化工件".to_string())?;
    if sha256_bytes(&bytes) != review.server_computed_artifact_sha256
        || bytes.len() as u64 != review.server_observed_artifact_byte_length
    {
        return Err("未来 Stage 112 固定观察物化工件摘要或长度漂移".to_string());
    }
    Ok(bytes)
}

fn safe_read_only_regular_file(metadata: &std::fs::Metadata, maximum_bytes: u64) -> bool {
    !metadata.file_type().is_symlink()
        && metadata.is_file()
        && metadata.permissions().readonly()
        && metadata.len() > 0
        && metadata.len() <= maximum_bytes
}

fn artifact_inspection_matches_review(
    inspection: &ControlledShadowObservationMaterializationArtifactInspection,
    review: &ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview,
) -> bool {
    inspection.artifact_verified
        && inspection
            .manifest
            .as_ref()
            .is_some_and(|manifest| manifest == &review.artifact_manifest)
        && inspection.server_computed_artifact_sha256.as_deref()
            == Some(review.server_computed_artifact_sha256.as_str())
        && inspection.server_observed_artifact_byte_length
            == Some(review.server_observed_artifact_byte_length)
}

fn pending_inspection(
    custody_locator: String,
    manifest_present: bool,
    artifact_present: bool,
    status: &str,
) -> ControlledShadowObservationMaterializationArtifactInspection {
    ControlledShadowObservationMaterializationArtifactInspection {
        custody_locator,
        manifest_present,
        artifact_present,
        manifest: None,
        server_computed_artifact_sha256: None,
        server_observed_artifact_byte_length: None,
        artifact_verified: false,
        status: status.to_string(),
    }
}

fn validate_manifest(
    manifest: &ControlledShadowObservationMaterializationReproducedArtifactManifest,
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
) -> Result<(), String> {
    let contract = &runner.runner_contract;
    let valid = manifest.schema_version == MANIFEST_SCHEMA_VERSION
        && valid_sha256(&manifest.manifest_sha256)
        && manifest.manifest_sha256 == fingerprint_without(manifest, &["manifest_sha256"])?
        && manifest.isolated_runner_id == runner.isolated_runner_id
        && manifest.isolated_runner_spec_sha256 == runner.isolated_runner_spec_sha256
        && manifest.runner_contract_sha256 == contract.contract_sha256
        && manifest.runner_spec_revision == contract.runner_spec_revision
        && manifest.runner_code_revision == contract.proposed_runner_code_revision
        && manifest.runner_artifact_sha256 == contract.proposed_runner_artifact_sha256
        && manifest.artifact_byte_length > 0
        && manifest.artifact_byte_length <= MAX_ARTIFACT_BYTES
        && manifest.artifact_file_name == ARTIFACT_FILE_NAME
        && !manifest.artifact_media_type.trim().is_empty()
        && valid_sha256(&manifest.source_bundle_sha256)
        && manifest.artifact_reproduction_procedure_sha256
            == sha256_bytes(runner.artifact_reproduction_procedure.as_bytes())
        && manifest.runtime_identity == contract.runtime_identity
        && manifest.runtime_version == contract.runtime_version
        && !manifest.reproduced_by.trim().is_empty()
        && manifest.source_and_artifact_reproduced_from_immutable_revision
        && manifest.artifact_is_read_only_regular_file
        && manifest.artifact_was_not_executed
        && manifest.stage_104_admitted_input_was_not_read;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 110 工件复现 manifest 无效或与 Stage 109 漂移".to_string())
}

fn expected_excluded_prior_actor_ids(
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
    manifest: &ControlledShadowObservationMaterializationReproducedArtifactManifest,
    latest: Option<&ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview>,
) -> Vec<String> {
    let mut values = runner.excluded_prior_actor_ids.clone();
    values.push(runner.registered_by.clone());
    values.push(runner.implementation_review.reviewer_id.clone());
    values.push(manifest.reproduced_by.clone());
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
    review: &ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview,
    now: DateTime<Utc>,
) -> bool {
    review.one_future_claim_first_observation_materialization_attempt_authorized
        && !review.authorization_claimed
        && now >= review.submitted_at
        && now < review.authorization_valid_until
}

async fn latest_review(
    state: &AppState,
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
) -> Result<
    Option<ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview>,
    String,
> {
    latest_review_from_records(&read_reviews(state, runner).await?)
}

async fn read_reviews(
    state: &AppState,
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
) -> Result<Vec<ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview>, String>
{
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
        let review: ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview =
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
            return Err("Stage 110 授权复核链重复或文件名错误".to_string());
        }
        reviews.push(review);
    }
    Ok(reviews)
}

fn latest_review_from_records(
    reviews: &[ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview],
) -> Result<
    Option<ControlledShadowObservationMaterializationFirstExecutionAuthorizationReview>,
    String,
> {
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
        return Err("Stage 110 授权复核链 tip 数量无效".to_string());
    }
    if reviews.iter().any(|review| {
        review.one_future_claim_first_observation_materialization_attempt_authorized
            && reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
    }) {
        return Err("Stage 110 批准必须终止复核链".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("Stage 110 授权复核链存在环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(hash)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "Stage 110 授权复核链断裂".to_string())?;
                if previous.review_sha256 != *hash {
                    return Err("Stage 110 授权复核链前序摘要不一致".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("Stage 110 授权复核链前序链接无效".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("Stage 110 授权复核链未完全连通".to_string());
    }
    for review in reviews {
        let previous = review
            .previous_review_id
            .as_deref()
            .and_then(|id| by_id.get(id).copied());
        if review.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(
                &review.runner,
                &review.artifact_manifest,
                previous,
            )
        {
            return Err("Stage 110 授权复核责任链不一致".to_string());
        }
    }
    Ok(Some(tips[0].clone()))
}

fn sha256_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 110 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(sha256_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
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

fn artifact_directory(
    state: &AppState,
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-observation-materialization-reproduced-artifacts")
        .join(&runner.isolated_runner_id)
        .join(&runner.runner_contract.proposed_runner_artifact_sha256)
}

fn review_directory(
    state: &AppState,
    runner: &ControlledShadowObservationMaterializationIsolatedRunnerRecord,
) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-observation-materialization-first-execution-authorization-reviews")
        .join(&runner.isolated_runner_id)
}

struct ReviewLock(PathBuf);
impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, isolated_runner_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-observation-materialization-first-execution-{isolated_runner_id}.lock"
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
        .map_err(|_| "已有同一 Stage 110 授权复核正在进行".to_string())?;
    Ok(ReviewLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_is_single_use_and_strictly_twenty_four_hours() {
        let submitted = Utc::now();
        let valid_until = submitted + TimeDelta::hours(AUTHORIZATION_VALID_HOURS);
        assert!(submitted + TimeDelta::hours(23) < valid_until);
        assert_eq!(submitted + TimeDelta::hours(24), valid_until);
        assert_eq!(ONE_SHOT_ATTEMPT_LIMIT, 1);
    }

    #[test]
    fn artifact_limits_are_bounded_and_content_addressed() {
        assert_eq!(ARTIFACT_FILE_NAME, "runner.artifact");
        assert_eq!(MANIFEST_FILE_NAME, "manifest.json");
        assert_eq!(MAX_MANIFEST_BYTES, 64 * 1024);
        assert_eq!(MAX_ARTIFACT_BYTES, 16 * 1024 * 1024);
        assert_eq!(sha256_bytes(b"runner"), sha256_bytes(b"runner"));
        assert_ne!(sha256_bytes(b"runner"), sha256_bytes(b"other"));
    }

    #[test]
    fn stage_110_has_no_materialization_or_trading_entrypoint() {
        let capability_flags = [false; 20];
        assert!(capability_flags.into_iter().all(|value| !value));
        assert_eq!(
            NEXT_GATE,
            "stage_111_claim_first_observation_materialization_execution_attempt"
        );
    }

    #[test]
    fn artifact_files_must_be_nonempty_read_only_regular_files() {
        let directory = std::env::temp_dir().join(format!(
            "hone-stage-110-metadata-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&directory).expect("create temp directory");
        let file = directory.join("artifact");
        std::fs::write(&file, b"runner").expect("write artifact");
        let writable = std::fs::symlink_metadata(&file).expect("writable metadata");
        assert!(!safe_read_only_regular_file(&writable, 64));
        let mut permissions = writable.permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&file, permissions).expect("set read only");
        let read_only = std::fs::symlink_metadata(&file).expect("read-only metadata");
        assert!(safe_read_only_regular_file(&read_only, 64));
        assert!(!safe_read_only_regular_file(&read_only, 2));
        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir(&directory);
    }
}
