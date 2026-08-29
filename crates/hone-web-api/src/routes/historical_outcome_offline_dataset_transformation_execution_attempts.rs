//! One-shot isolated offline-dataset transformation execution attempts.
//!
//! One exact, current and unexpired Stage 30 authorization is consumed by a
//! create-once claim before any transformation work begins.  The fixed
//! in-process projection receives only already sealed records and emits a
//! content-addressed *untrusted candidate envelope*.  It never reads fresh
//! data, exposes holdout labels, creates an official split manifest or feature
//! bundle, assigns a semantic target, trains, rewards, shadows, orders, brokers
//! or trades.  Success and failure both consume the authorization.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::historical_outcome_offline_dataset_transformation_first_execution_authorizations::{
    ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    execution_attempt_eligible_transformation_authorizations,
};
use super::historical_outcome_offline_dataset_transformation_specs::PointInTimeFeatureDefinition;
use super::historical_outcome_offline_datasets::{
    HistoricalOutcomeOfflineDataset, HistoricalOutcomeOfflineDatasetEntry,
    exact_current_historical_outcome_offline_dataset_for_transformation,
};
use super::historical_outcome_price_snapshots::{
    HistoricalOutcomePriceSnapshot,
    current_sealed_historical_outcome_price_snapshots_for_transformation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-execution-attempt-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-execution-attempt-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-untrusted-candidate-envelope-v1";
const EXECUTION_POLICY_VERSION: &str =
    "hone-offline-dataset-transformation-one-shot-pure-projection-v1";
const ISOLATION_BACKEND: &str =
    "fixed-sealed-dataset-transformation-pure-function-no-ambient-capabilities-v1";
const MISSINGNESS_TRANSFORMATION_ID: &str = "hone-explicit-point-in-time-missingness-v1";
const MAX_ERROR_CHARS: usize = 1_200;
const EXPECTED_FEATURE_COUNT: usize = 65;
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];

#[derive(Debug)]
struct TransformationFailure {
    message: String,
    ephemeral_directory_removed: bool,
}

impl TransformationFailure {
    fn before_staging(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral_directory_removed: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeHistoricalOutcomeOfflineDatasetTransformationOnceRequest {
    expected_first_execution_authorization_review_id: String,
    expected_first_execution_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_runner_code_revision: String,
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_review_sha256: String,
    expected_transformation_spec_sha256: String,
    expected_transformation_body_sha256: String,
    expected_split_specification_sha256: String,
    expected_feature_specification_sha256: String,
    expected_dataset_id: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetTransformationSplit {
    Train,
    Validation,
    SealedHoldout,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit {
    pub schema_version: String,
    pub candidate_pair_count: usize,
    pub selected_train_component_end_exclusive: usize,
    pub selected_validation_component_end_exclusive: usize,
    pub pre_purge_train_entry_count: usize,
    pub pre_purge_validation_entry_count: usize,
    pub pre_purge_sealed_holdout_entry_count: usize,
    pub objective_tuple: [u64; 4],
    pub all_candidate_objectives_sha256: String,
    pub audit_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord {
    pub dataset_entry_id: String,
    pub dataset_entry_sha256: String,
    pub component_id: String,
    pub component_order_index: usize,
    pub split: HistoricalOutcomeOfflineDatasetTransformationSplit,
    pub boundary_objective_tuple: [u64; 4],
    pub purged_or_embargoed: bool,
    pub purge_reason: Option<String>,
    pub price_snapshot_sha256: String,
    pub common_market_session_index: usize,
    pub decision_session_date: NaiveDate,
    pub outcome_horizon_250_end_date: NaiveDate,
    pub decision_available_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord {
    pub dataset_entry_id: String,
    pub feature_id: String,
    pub feature_namespace: String,
    pub value_encoding: String,
    pub value: Option<String>,
    pub value_kind: String,
    pub unit: String,
    pub period_start_utc: Option<DateTime<Utc>>,
    pub period_end_utc: Option<DateTime<Utc>>,
    pub accounting_basis: String,
    pub measurement_scope: String,
    pub is_missing: bool,
    pub missingness_reason: String,
    pub artifact_sha256: String,
    pub source_identity: String,
    pub source_version: String,
    pub observed_at_utc: DateTime<Utc>,
    pub available_at_utc: DateTime<Utc>,
    pub transformation_id: String,
    pub transformation_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationExclusionAudit {
    pub dataset_entry_id: String,
    pub purged_or_embargoed: bool,
    pub outcome_and_label_namespaces_excluded: bool,
    pub post_decision_and_future_sources_excluded: bool,
    pub current_portfolio_state_excluded: bool,
    pub missing_values_not_backfilled_or_interpolated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope {
    pub schema_version: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub governance_review_sha256: String,
    pub transformation_spec_sha256: String,
    pub transformation_body_sha256: String,
    pub split_specification_sha256: String,
    pub feature_specification_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub runner_id: String,
    pub runner_spec_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub entry_count: usize,
    pub component_count: usize,
    pub feature_catalog_count: usize,
    pub feature_catalog_sha256: String,
    pub feature_schema_sha256: String,
    pub boundary_audit: HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit,
    pub split_manifest_candidate:
        Vec<HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord>,
    pub feature_bundle_candidate:
        Vec<HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord>,
    pub excluded_feature_audit: Vec<HistoricalOutcomeOfflineDatasetTransformationExclusionAudit>,
    pub sealed_holdout_labels_withheld: bool,
    pub deterministic_projection_only: bool,
    pub explicit_missingness_preserved: bool,
    pub output_is_untrusted: bool,
    pub independent_validation_completed: bool,
    pub official_split_manifest_created: bool,
    pub official_feature_bundle_created: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assigned: bool,
    pub copied_to_training_store: bool,
    pub training_started: bool,
    pub reward_written: bool,
    pub shadow_position_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub authorization_valid_until: DateTime<Utc>,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub runner_code_revision: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_review_id: String,
    pub implementation_review_sha256: String,
    pub transformation_spec_id: String,
    pub transformation_spec_sha256: String,
    pub transformation_body_sha256: String,
    pub split_specification_sha256: String,
    pub feature_specification_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub governance_review_id: String,
    pub governance_review_sha256: String,
    pub max_wall_clock_seconds: u16,
    pub max_memory_mib: u32,
    pub max_cpu_millicores: u16,
    pub max_process_count: u8,
    pub max_output_bytes: u32,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub isolation_backend: String,
    pub artifact_digest_reverified: bool,
    pub current_complete_upstream_chain_revalidated: bool,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub child_process_spawned: bool,
    pub ambient_filesystem_capability_available: bool,
    pub ambient_environment_capability_available: bool,
    pub network_capability_available: bool,
    pub external_tool_capability_available: bool,
    pub production_data_capability_available_to_projection: bool,
    pub historical_state_mutation_allowed: bool,
    pub official_manifest_or_bundle_writes_allowed: bool,
    pub semantic_target_writes_allowed: bool,
    pub training_writes_allowed: bool,
    pub reward_writes_allowed: bool,
    pub shadow_writes_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus {
    CompletedWithUntrustedCandidateEnvelope,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus,
    pub exit_code: i32,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub bounded_error: Option<String>,
    pub output_sha256: Option<String>,
    pub untrusted_candidate_envelope:
        Option<HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope>,
    pub ephemeral_directory_removed: bool,
    pub independent_validation_completed: bool,
    pub official_split_manifest_authorized: bool,
    pub official_feature_bundle_authorized: bool,
    pub feature_join_authorized: bool,
    pub semantic_target_authorized: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptItem {
    pub claim: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
    pub result: Option<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult>,
    pub current_authorization_binding: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub isolation_backend: String,
    pub invocation_endpoint_available: bool,
    pub invocation_eligible_authorization_count: usize,
    pub eligible_authorizations:
        Vec<ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization>,
    pub attempt_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_candidate_envelope_count: usize,
    pub independent_validation_eligible_count: usize,
    pub execution_status: String,
    pub attempts: Vec<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptItem>,
    pub official_split_manifest_created: bool,
    pub official_feature_bundle_created: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assigned: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptReadinessSummary {
    pub invocation_eligible_authorization_count: usize,
    pub attempt_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_candidate_envelope_count: usize,
    pub independent_validation_eligible_count: usize,
    pub execution_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt {
    pub claim: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
    pub result: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult,
}

#[derive(Debug, Clone)]
struct TransformationEntryInput {
    entry_id: String,
    entry_sha256: String,
    asset_symbol: String,
    reconstruction_sha256: String,
    snapshot_sha256: String,
    source_output_sha256: String,
    source_validation_sha256: String,
    protocol_sha256: String,
    decision_available_at: DateTime<Utc>,
    decision_session_date: NaiveDate,
    outcome_horizon_250_end_date: NaiveDate,
    common_market_session_index: usize,
}

#[derive(Debug, Clone)]
struct TransformationComponent {
    component_id: String,
    entry_indexes: Vec<usize>,
    earliest: DateTime<Utc>,
    latest: DateTime<Utc>,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_execution_attempts(
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
            warn!(%error, "historical outcome offline transformation execution registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线转换执行记录暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_historical_outcome_offline_dataset_transformation_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeHistoricalOutcomeOfflineDatasetTransformationOnceRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match invoke_once(&state, &admin.user_id, &isolated_runner_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_transformation_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptReadinessSummary {
            invocation_eligible_authorization_count: registry
                .invocation_eligible_authorization_count,
            attempt_count: registry.attempt_count,
            completed_attempt_count: registry.completed_attempt_count,
            failed_attempt_count: registry.failed_attempt_count,
            untrusted_candidate_envelope_count: registry.untrusted_candidate_envelope_count,
            independent_validation_eligible_count: registry.independent_validation_eligible_count,
            execution_status: registry.execution_status,
        },
    )
}

pub(crate) async fn completed_transformation_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .into_iter()
        .filter(|result| {
            result.status
                == HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus::CompletedWithUntrustedCandidateEnvelope
                && result.untrusted_candidate_envelope.is_some()
                && result.output_sha256.is_some()
        })
        .map(|result| (result.attempt_id.clone(), result))
        .collect::<HashMap<_, _>>();
    let mut output = claims
        .into_iter()
        .filter_map(|claim| {
            result_index.get(&claim.attempt_id).cloned().map(|result| {
                CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt {
                    claim,
                    result,
                }
            })
        })
        .collect::<Vec<_>>();
    output.sort_by(|left, right| right.claim.claimed_at.cmp(&left.claim.claimed_at));
    Ok(output)
}

pub(crate) async fn completed_transformation_execution_attempt_for_independent_validation(
    state: &AppState,
    attempt_id: &str,
    claim_sha256: &str,
    result_sha256: &str,
    output_sha256: &str,
) -> Result<Option<CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt>, String>
{
    let attempts =
        completed_transformation_execution_attempts_for_independent_validation(state).await?;
    Ok(attempts.into_iter().find(|attempt| {
        attempt.claim.attempt_id == attempt_id
            && attempt.claim.claim_sha256 == claim_sha256
            && attempt.result.result_sha256 == result_sha256
            && attempt.result.output_sha256.as_deref() == Some(output_sha256)
            && attempt
                .result
                .untrusted_candidate_envelope
                .as_ref()
                .and_then(|envelope| serde_json::to_vec(envelope).ok())
                .is_some_and(|bytes| hash_bytes(&bytes) == output_sha256)
    }))
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry, String> {
    let authorizations =
        execution_attempt_eligible_transformation_authorizations(state, Utc::now()).await?;
    let authorization_index = authorizations
        .iter()
        .map(|value| (value.review.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .into_iter()
        .map(|result| (result.attempt_id.clone(), result))
        .collect::<HashMap<_, _>>();
    let mut attempts = claims
        .into_iter()
        .map(|claim| {
            let current_authorization_binding = authorization_index
                .get(claim.authorization_review_id.as_str())
                .is_some_and(|authorization| claim_binding_matches(&claim, authorization));
            let result = result_index.get(&claim.attempt_id).cloned();
            HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptItem {
                claim,
                result,
                current_authorization_binding,
            }
        })
        .collect::<Vec<_>>();
    attempts.sort_by(|left, right| right.claim.claimed_at.cmp(&left.claim.claimed_at));
    let consumed = attempts
        .iter()
        .map(|item| item.claim.authorization_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let invocation_eligible_authorization_count = authorizations
        .iter()
        .filter(|authorization| !consumed.contains(authorization.review.review_id.as_str()))
        .count();
    let eligible_authorizations = authorizations
        .into_iter()
        .filter(|authorization| !consumed.contains(authorization.review.review_id.as_str()))
        .collect::<Vec<_>>();
    let attempt_count = attempts.len();
    let completed_attempt_count = attempts
        .iter()
        .filter(|item| item.result.as_ref().is_some_and(|result| {
            result.status == HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus::CompletedWithUntrustedCandidateEnvelope
        }))
        .count();
    let failed_attempt_count = attempts
        .iter()
        .filter(|item| item.result.as_ref().is_some_and(|result| {
            result.status == HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus::FailedAuthorizationConsumed
        }))
        .count();
    let untrusted_candidate_envelope_count = attempts
        .iter()
        .filter(|item| {
            item.result
                .as_ref()
                .is_some_and(|result| result.untrusted_candidate_envelope.is_some())
        })
        .count();
    let independent_validation_eligible_count = untrusted_candidate_envelope_count;
    let execution_status = if attempts.iter().any(|item| item.result.is_none()) {
        "transformation_claimed_incomplete_fail_closed"
    } else if untrusted_candidate_envelope_count > 0 {
        "untrusted_transformation_candidate_waiting_independent_validation"
    } else if failed_attempt_count > 0 {
        "transformation_failed_authorization_consumed"
    } else if invocation_eligible_authorization_count > 0 {
        "ready_for_one_shot_isolated_transformation"
    } else {
        "waiting_for_unexpired_unclaimed_first_execution_authorization"
    };
    Ok(HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        invocation_endpoint_available: true,
        invocation_eligible_authorization_count,
        eligible_authorizations,
        attempt_count,
        completed_attempt_count,
        failed_attempt_count,
        untrusted_candidate_envelope_count,
        independent_validation_eligible_count,
        execution_status: execution_status.to_string(),
        attempts,
        official_split_manifest_created: false,
        official_feature_bundle_created: false,
        feature_join_performed: false,
        semantic_target_assigned: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 31 阶段最多消费一条当前未过期且从未 claim 的授权；先写不可变 claim，再用固定纯函数生成待独立校验的切分与显式缺失特征候选。成功或失败都消费授权。输出不是正式 manifest/bundle，不连接特征、不定义目标、不进入训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    admin_id: &str,
    isolated_runner_id: &str,
    request: InvokeHistoricalOutcomeOfflineDatasetTransformationOnceRequest,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult, String> {
    if !valid_id(isolated_runner_id) {
        return Err("隔离转换 runner ID 无效".to_string());
    }
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let authorization = execution_attempt_eligible_transformation_authorizations(state, Utc::now())
        .await?
        .into_iter()
        .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "当前没有这条 runner 的未过期一次性授权".to_string())?;
    verify_request(&authorization, &request)?;
    let (existing_claims, _) = read_attempt_records(state).await?;
    if existing_claims.iter().any(|claim| {
        claim.isolated_runner_id == isolated_runner_id
            || claim.authorization_review_id == authorization.review.review_id
    }) {
        return Err("这条 runner 或首次执行授权已经消费，不得重放".to_string());
    }
    let runtime_artifact_sha256 = reverify_current_binary_sha256().await?;
    if runtime_artifact_sha256 != authorization.runner.runner_artifact_sha256 {
        return Err("当前运行制品与独立复核的 runner SHA-256 不一致".to_string());
    }
    let specification = &authorization
        .runner
        .implementation
        .approved_review
        .specification;
    let subject = &specification.subject;
    let dataset = exact_current_historical_outcome_offline_dataset_for_transformation(
        state,
        &subject.dataset_id,
        &subject.dataset_content_sha256,
        &subject.manifest_sha256,
        &subject.candidate_set_sha256,
    )
    .await?
    .ok_or_else(|| "隔离转换绑定的数据集已不再是精确当前完整集合".to_string())?;
    verify_dataset_binding(&authorization, &dataset)?;
    let snapshots =
        current_sealed_historical_outcome_price_snapshots_for_transformation(state).await?;
    let entries = build_transformation_inputs(&dataset, &snapshots)?;

    let claim = new_claim(&authorization, admin_id, Utc::now())?;
    write_immutable_json(
        &claim_directory(state, isolated_runner_id).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let execution = execute_fixed_transformation(&authorization, &dataset, &entries).await;
    let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let result = match execution {
        Ok((envelope, output_sha256, stdout_bytes)) => new_success_result(
            &claim,
            Utc::now(),
            duration_millis,
            envelope,
            output_sha256,
            stdout_bytes,
        )?,
        Err(error) => new_failed_result(&claim, Utc::now(), duration_millis, &error)?,
    };
    write_immutable_json(
        &result_directory(state, isolated_runner_id).join(format!("{}.json", result.result_id)),
        &result,
    )
    .await?;
    Ok(result)
}

fn verify_request(
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    request: &InvokeHistoricalOutcomeOfflineDatasetTransformationOnceRequest,
) -> Result<(), String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let specification = &implementation.approved_review.specification;
    let subject = &specification.subject;
    if request.expected_first_execution_authorization_review_id != authorization.review.review_id
        || request.expected_first_execution_authorization_review_sha256
            != authorization.review.review_sha256
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != runner.runner_artifact_sha256
        || request.expected_runner_code_revision != runner.runner_code_revision
        || request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_review_sha256
            != runner.implementation_review.review_sha256
        || request.expected_transformation_spec_sha256 != specification.transformation_spec_sha256
        || request.expected_transformation_body_sha256 != specification.transformation_body_sha256
        || request.expected_split_specification_sha256
            != specification
                .split_manifest_specification
                .specification_sha256
        || request.expected_feature_specification_sha256
            != specification
                .feature_bundle_specification
                .specification_sha256
        || request.expected_dataset_id != subject.dataset_id
        || request.expected_dataset_content_sha256 != subject.dataset_content_sha256
        || request.expected_dataset_manifest_sha256 != subject.manifest_sha256
        || request.expected_candidate_set_sha256 != subject.candidate_set_sha256
    {
        return Err("授权、runner、实现、规范或数据集绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn verify_dataset_binding(
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeOfflineDataset,
) -> Result<(), String> {
    let specification = &authorization
        .runner
        .implementation
        .approved_review
        .specification;
    let subject = &specification.subject;
    if dataset.dataset_id != subject.dataset_id
        || dataset.dataset_content_sha256 != subject.dataset_content_sha256
        || dataset.manifest_sha256 != subject.manifest_sha256
        || dataset.candidate_set_sha256 != subject.candidate_set_sha256
        || dataset.entry_count != subject.entry_count
        || dataset.distinct_symbol_count != subject.distinct_symbol_count
        || dataset.earliest_decision_available_at != subject.earliest_decision_available_at
        || dataset.latest_decision_available_at != subject.latest_decision_available_at
        || dataset.entries.iter().any(|entry| {
            !entry.raw_outcome_only
                || entry.feature_vector_present
                || entry.semantic_target_assigned
                || entry.split_assigned
                || entry.reward_present
        })
    {
        return Err("重新读取的数据集与 runner 的完整上游绑定不一致".to_string());
    }
    Ok(())
}

fn build_transformation_inputs(
    dataset: &HistoricalOutcomeOfflineDataset,
    snapshots: &[HistoricalOutcomePriceSnapshot],
) -> Result<Vec<TransformationEntryInput>, String> {
    let snapshot_index = snapshots
        .iter()
        .map(|snapshot| (snapshot.snapshot_sha256.as_str(), snapshot))
        .collect::<HashMap<_, _>>();
    dataset
        .entries
        .iter()
        .map(|entry| transformation_input(entry, &snapshot_index))
        .collect()
}

fn transformation_input(
    entry: &HistoricalOutcomeOfflineDatasetEntry,
    snapshot_index: &HashMap<&str, &HistoricalOutcomePriceSnapshot>,
) -> Result<TransformationEntryInput, String> {
    let snapshot = snapshot_index
        .get(entry.snapshot_sha256.as_str())
        .ok_or_else(|| format!("数据集条目 {} 缺少当前封存行情快照", entry.entry_id))?;
    if snapshot.reconstruction_sha256 != entry.reconstruction_sha256
        || snapshot.asset_symbol != entry.asset_symbol
        || snapshot.benchmark_symbol != entry.benchmark_symbol
        || snapshot.decision_available_at != entry.decision_available_at
    {
        return Err(format!(
            "数据集条目 {} 与封存行情快照绑定不一致",
            entry.entry_id
        ));
    }
    if entry.raw_validated_metrics.len() != REQUIRED_HORIZONS.len()
        || entry
            .raw_validated_metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect::<Vec<_>>()
            != REQUIRED_HORIZONS
    {
        return Err(format!(
            "数据集条目 {} 的冻结结果窗口不完整",
            entry.entry_id
        ));
    }
    let asset_dates = snapshot
        .asset_points
        .iter()
        .map(|point| point.date)
        .collect::<BTreeSet<_>>();
    let common_dates = snapshot
        .benchmark_points
        .iter()
        .filter_map(|point| asset_dates.contains(&point.date).then_some(point.date))
        .collect::<Vec<_>>();
    let start = entry.raw_validated_metrics[0].start_date;
    let common_market_session_index = common_dates
        .iter()
        .position(|date| *date == start)
        .ok_or_else(|| {
            format!(
                "数据集条目 {} 的判断起始交易日不在封存共同日历",
                entry.entry_id
            )
        })?;
    for metric in &entry.raw_validated_metrics {
        if metric.start_date != start || !common_dates.contains(&metric.end_date) {
            return Err(format!(
                "数据集条目 {} 的结果窗口端点不在封存共同日历",
                entry.entry_id
            ));
        }
    }
    Ok(TransformationEntryInput {
        entry_id: entry.entry_id.clone(),
        entry_sha256: entry.entry_sha256.clone(),
        asset_symbol: entry.asset_symbol.clone(),
        reconstruction_sha256: entry.reconstruction_sha256.clone(),
        snapshot_sha256: entry.snapshot_sha256.clone(),
        source_output_sha256: entry.source_output_sha256.clone(),
        source_validation_sha256: entry.source_validation_sha256.clone(),
        protocol_sha256: entry.protocol_sha256.clone(),
        decision_available_at: entry.decision_available_at,
        decision_session_date: start,
        outcome_horizon_250_end_date: entry.raw_validated_metrics[2].end_date,
        common_market_session_index,
    })
}

fn new_claim(
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    admin_id: &str,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim, String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let specification = &implementation.approved_review.specification;
    let subject = &specification.subject;
    let contract = &runner.runner_contract;
    let mut claim = HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        attempt_id: String::new(),
        claim_sha256: String::new(),
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        authorization_valid_until: authorization.review.authorization_valid_until,
        isolated_runner_id: runner.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: runner.runner_artifact_sha256.clone(),
        runner_code_revision: runner.runner_code_revision.clone(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_review_id: runner.implementation_review.review_id.clone(),
        implementation_review_sha256: runner.implementation_review.review_sha256.clone(),
        transformation_spec_id: specification.transformation_spec_id.clone(),
        transformation_spec_sha256: specification.transformation_spec_sha256.clone(),
        transformation_body_sha256: specification.transformation_body_sha256.clone(),
        split_specification_sha256: specification
            .split_manifest_specification
            .specification_sha256
            .clone(),
        feature_specification_sha256: specification
            .feature_bundle_specification
            .specification_sha256
            .clone(),
        dataset_id: subject.dataset_id.clone(),
        dataset_content_sha256: subject.dataset_content_sha256.clone(),
        dataset_manifest_sha256: subject.manifest_sha256.clone(),
        candidate_set_sha256: subject.candidate_set_sha256.clone(),
        governance_review_id: specification.governance_review_id.clone(),
        governance_review_sha256: specification.governance_review_sha256.clone(),
        max_wall_clock_seconds: contract.maximum_wall_clock_seconds,
        max_memory_mib: contract.maximum_memory_mebibytes,
        max_cpu_millicores: contract.maximum_cpu_millicores,
        max_process_count: contract.maximum_process_count,
        max_output_bytes: contract.maximum_output_bytes,
        claimed_at,
        invoked_by: admin_id.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        artifact_digest_reverified: true,
        current_complete_upstream_chain_revalidated: true,
        authorization_consumed: true,
        invocation_started: true,
        child_process_spawned: false,
        ambient_filesystem_capability_available: false,
        ambient_environment_capability_available: false,
        network_capability_available: false,
        external_tool_capability_available: false,
        production_data_capability_available_to_projection: false,
        historical_state_mutation_allowed: false,
        official_manifest_or_bundle_writes_allowed: false,
        semantic_target_writes_allowed: false,
        training_writes_allowed: false,
        reward_writes_allowed: false,
        shadow_writes_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = claim_fingerprint(&claim)?;
    claim.attempt_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim)?;
    Ok(claim)
}

async fn execute_fixed_transformation(
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeOfflineDataset,
    entries: &[TransformationEntryInput],
) -> Result<
    (
        HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope,
        String,
        usize,
    ),
    TransformationFailure,
> {
    let envelope = transform_to_untrusted_candidate(authorization, dataset, entries)
        .map_err(TransformationFailure::before_staging)?;
    let bytes = serde_json::to_vec(&envelope)
        .map_err(|error| TransformationFailure::before_staging(error.to_string()))?;
    if bytes.len() > authorization.runner.runner_contract.maximum_output_bytes as usize {
        return Err(TransformationFailure::before_staging(
            "未验证转换候选超过 runner 登记输出上限",
        ));
    }
    let directory = std::env::temp_dir().join(format!(
        "hone-offline-dataset-transformation-{}",
        Uuid::new_v4()
    ));
    tokio::fs::create_dir(&directory)
        .await
        .map_err(|error| TransformationFailure {
            message: format!("一次性转换目录创建失败：{error}"),
            ephemeral_directory_removed: false,
        })?;
    let path = directory.join("untrusted-transformation-candidate.json");
    let staged = async {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
            .map_err(|error| error.to_string())?;
        use tokio::io::AsyncWriteExt;
        file.write_all(&bytes)
            .await
            .map_err(|error| error.to_string())?;
        file.sync_all().await.map_err(|error| error.to_string())?;
        let read_back = tokio::fs::read(&path)
            .await
            .map_err(|error| error.to_string())?;
        if read_back != bytes {
            return Err("一次性转换输出回读不一致".to_string());
        }
        Ok::<_, String>(hash_bytes(&read_back))
    }
    .await;
    let cleanup = tokio::fs::remove_dir_all(&directory).await;
    let removed = cleanup.is_ok()
        || tokio::fs::metadata(&directory)
            .await
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
    match (staged, cleanup, removed) {
        (Ok(output_sha256), Ok(()), true) => Ok((envelope, output_sha256, bytes.len())),
        (Ok(_), Err(error), removed) => Err(TransformationFailure {
            message: format!("一次性转换目录清理失败：{error}"),
            ephemeral_directory_removed: removed,
        }),
        (Err(error), _, removed) => Err(TransformationFailure {
            message: error,
            ephemeral_directory_removed: removed,
        }),
        (Ok(_), Ok(()), false) => Err(TransformationFailure {
            message: "一次性转换目录清理状态无法确认".to_string(),
            ephemeral_directory_removed: false,
        }),
    }
}

fn transform_to_untrusted_candidate(
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeOfflineDataset,
    entries: &[TransformationEntryInput],
) -> Result<HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope, String> {
    if entries.len() != dataset.entry_count || entries.is_empty() {
        return Err("转换输入条目数与封存数据集不一致".to_string());
    }
    let specification = &authorization
        .runner
        .implementation
        .approved_review
        .specification;
    let feature_catalog = &specification.feature_bundle_specification.allowed_features;
    if feature_catalog.len() != EXPECTED_FEATURE_COUNT {
        return Err("点时特征允许列表不是冻结的 65 项".to_string());
    }
    let components = connected_components(
        entries,
        &specification.split_manifest_specification.schema_version,
    )?;
    if components.len() < 3 {
        return Err("连通分量少于三个，不能形成互斥 train/validation/sealed holdout".to_string());
    }
    let (boundary_audit, mut split_records) = select_and_apply_boundaries(entries, &components)?;
    apply_purge_and_embargo(&mut split_records)?;
    for split in [
        HistoricalOutcomeOfflineDatasetTransformationSplit::Train,
        HistoricalOutcomeOfflineDatasetTransformationSplit::Validation,
        HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout,
    ] {
        if !split_records
            .iter()
            .any(|record| record.split == split && !record.purged_or_embargoed)
        {
            return Err("250 交易日 purge/embargo 后存在空分区，转换失败关闭".to_string());
        }
    }
    let transformation_sha256 = hash_serializable(&serde_json::json!({
        "transformation_id": MISSINGNESS_TRANSFORMATION_ID,
        "feature_specification_sha256": specification.feature_bundle_specification.specification_sha256,
        "backfill_allowed": false,
        "interpolation_allowed": false,
    }))?;
    let active_entry_ids = split_records
        .iter()
        .filter(|record| !record.purged_or_embargoed)
        .map(|record| record.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut feature_bundle_candidate =
        Vec::with_capacity(active_entry_ids.len() * feature_catalog.len());
    for entry in entries
        .iter()
        .filter(|entry| active_entry_ids.contains(entry.entry_id.as_str()))
    {
        for feature in feature_catalog {
            feature_bundle_candidate.push(explicit_missing_feature(
                entry,
                feature,
                &transformation_sha256,
            ));
        }
    }
    let excluded_feature_audit = entries
        .iter()
        .map(
            |entry| HistoricalOutcomeOfflineDatasetTransformationExclusionAudit {
                dataset_entry_id: entry.entry_id.clone(),
                purged_or_embargoed: !active_entry_ids.contains(entry.entry_id.as_str()),
                outcome_and_label_namespaces_excluded: true,
                post_decision_and_future_sources_excluded: true,
                current_portfolio_state_excluded: true,
                missing_values_not_backfilled_or_interpolated: true,
            },
        )
        .collect::<Vec<_>>();
    let envelope = HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_string(),
        dataset_id: dataset.dataset_id.clone(),
        dataset_content_sha256: dataset.dataset_content_sha256.clone(),
        dataset_manifest_sha256: dataset.manifest_sha256.clone(),
        candidate_set_sha256: dataset.candidate_set_sha256.clone(),
        governance_review_sha256: specification.governance_review_sha256.clone(),
        transformation_spec_sha256: specification.transformation_spec_sha256.clone(),
        transformation_body_sha256: specification.transformation_body_sha256.clone(),
        split_specification_sha256: specification
            .split_manifest_specification
            .specification_sha256
            .clone(),
        feature_specification_sha256: specification
            .feature_bundle_specification
            .specification_sha256
            .clone(),
        implementation_id: authorization
            .runner
            .implementation
            .implementation_id
            .clone(),
        implementation_sha256: authorization
            .runner
            .implementation
            .implementation_sha256
            .clone(),
        runner_id: authorization.runner.isolated_runner_id.clone(),
        runner_spec_sha256: authorization.runner.isolated_runner_spec_sha256.clone(),
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        entry_count: entries.len(),
        component_count: components.len(),
        feature_catalog_count: feature_catalog.len(),
        feature_catalog_sha256: hash_serializable(feature_catalog)?,
        feature_schema_sha256: specification
            .feature_bundle_specification
            .specification_sha256
            .clone(),
        boundary_audit,
        split_manifest_candidate: split_records,
        feature_bundle_candidate,
        excluded_feature_audit,
        sealed_holdout_labels_withheld: true,
        deterministic_projection_only: true,
        explicit_missingness_preserved: true,
        output_is_untrusted: true,
        independent_validation_completed: false,
        official_split_manifest_created: false,
        official_feature_bundle_created: false,
        feature_join_performed: false,
        semantic_target_assigned: false,
        copied_to_training_store: false,
        training_started: false,
        reward_written: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    };
    validate_envelope(&envelope)?;
    Ok(envelope)
}

fn explicit_missing_feature(
    entry: &TransformationEntryInput,
    feature: &PointInTimeFeatureDefinition,
    transformation_sha256: &str,
) -> HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord {
    HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord {
        dataset_entry_id: entry.entry_id.clone(),
        feature_id: feature.feature_id.clone(),
        feature_namespace: feature.namespace.clone(),
        value_encoding: "explicit_missingness".to_string(),
        value: None,
        value_kind: feature.value_kind.clone(),
        unit: String::new(),
        period_start_utc: None,
        period_end_utc: None,
        accounting_basis: "not_available".to_string(),
        measurement_scope: "point_in_time_reconstruction_has_no_deterministic_allowlisted_value"
            .to_string(),
        is_missing: true,
        missingness_reason: "availability_ambiguous".to_string(),
        artifact_sha256: entry.reconstruction_sha256.clone(),
        source_identity: format!(
            "historical_state_reconstruction:{}",
            entry.reconstruction_sha256
        ),
        source_version: "hone-historical-state-components-v1-seven-layer".to_string(),
        observed_at_utc: entry.decision_available_at,
        available_at_utc: entry.decision_available_at,
        transformation_id: MISSINGNESS_TRANSFORMATION_ID.to_string(),
        transformation_sha256: transformation_sha256.to_string(),
    }
}

fn connected_components(
    entries: &[TransformationEntryInput],
    split_schema_version: &str,
) -> Result<Vec<TransformationComponent>, String> {
    let mut parent = (0..entries.len()).collect::<Vec<_>>();
    for left in 0..entries.len() {
        for right in (left + 1)..entries.len() {
            if entries_connected(&entries[left], &entries[right]) {
                union(&mut parent, left, right);
            }
        }
    }
    let mut groups = BTreeMap::<usize, Vec<usize>>::new();
    for index in 0..entries.len() {
        let root = find(&mut parent, index);
        groups.entry(root).or_default().push(index);
    }
    let mut components = groups
        .into_values()
        .map(|mut indexes| {
            indexes.sort_by(|left, right| entries[*left].entry_id.cmp(&entries[*right].entry_id));
            let identities = indexes
                .iter()
                .map(|index| {
                    let entry = &entries[*index];
                    serde_json::json!({
                        "asset_symbol": entry.asset_symbol,
                        "reconstruction_sha256": entry.reconstruction_sha256,
                        "snapshot_sha256": entry.snapshot_sha256,
                        "decision_available_at": entry.decision_available_at,
                        "source_output_sha256": entry.source_output_sha256,
                        "source_validation_sha256": entry.source_validation_sha256,
                        "protocol_sha256": entry.protocol_sha256,
                    })
                })
                .collect::<Vec<_>>();
            let component_id = hash_serializable(&serde_json::json!({
                "split_schema_version": split_schema_version,
                "entries": identities,
            }))?;
            let earliest = indexes
                .iter()
                .map(|index| entries[*index].decision_available_at)
                .min()
                .unwrap();
            let latest = indexes
                .iter()
                .map(|index| entries[*index].decision_available_at)
                .max()
                .unwrap();
            Ok::<_, String>(TransformationComponent {
                component_id,
                entry_indexes: indexes,
                earliest,
                latest,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    components.sort_by(|left, right| {
        left.latest
            .cmp(&right.latest)
            .then_with(|| left.earliest.cmp(&right.earliest))
            .then_with(|| left.component_id.cmp(&right.component_id))
    });
    Ok(components)
}

fn entries_connected(left: &TransformationEntryInput, right: &TransformationEntryInput) -> bool {
    left.asset_symbol == right.asset_symbol
        || left.reconstruction_sha256 == right.reconstruction_sha256
        || left.snapshot_sha256 == right.snapshot_sha256
        || left.source_output_sha256 == right.source_output_sha256
        || left.source_validation_sha256 == right.source_validation_sha256
}

fn find(parent: &mut [usize], index: usize) -> usize {
    if parent[index] != index {
        parent[index] = find(parent, parent[index]);
    }
    parent[index]
}

fn union(parent: &mut [usize], left: usize, right: usize) {
    let left_root = find(parent, left);
    let right_root = find(parent, right);
    if left_root != right_root {
        let (low, high) = if left_root < right_root {
            (left_root, right_root)
        } else {
            (right_root, left_root)
        };
        parent[high] = low;
    }
}

fn select_and_apply_boundaries(
    entries: &[TransformationEntryInput],
    components: &[TransformationComponent],
) -> Result<
    (
        HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit,
        Vec<HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord>,
    ),
    String,
> {
    let total = entries.len() as u64;
    let mut candidates = Vec::<(usize, usize, [u64; 4], [usize; 3])>::new();
    for train_end in 1..(components.len() - 1) {
        for validation_end in (train_end + 1)..components.len() {
            let counts = [
                components[..train_end]
                    .iter()
                    .map(|component| component.entry_indexes.len())
                    .sum(),
                components[train_end..validation_end]
                    .iter()
                    .map(|component| component.entry_indexes.len())
                    .sum(),
                components[validation_end..]
                    .iter()
                    .map(|component| component.entry_indexes.len())
                    .sum(),
            ];
            let deviations = [
                (100 * counts[0] as i64 - 70 * total as i64).unsigned_abs(),
                (100 * counts[1] as i64 - 15 * total as i64).unsigned_abs(),
                (100 * counts[2] as i64 - 15 * total as i64).unsigned_abs(),
            ];
            let objective = [
                deviations.iter().sum(),
                *deviations.iter().max().unwrap(),
                train_end as u64,
                validation_end as u64,
            ];
            candidates.push((train_end, validation_end, objective, counts));
        }
    }
    candidates.sort_by_key(|candidate| candidate.2);
    let (train_end, validation_end, objective, counts) = candidates
        .first()
        .cloned()
        .ok_or_else(|| "没有合法的连续分量边界".to_string())?;
    let all_candidate_objectives_sha256 = hash_serializable(
        &candidates
            .iter()
            .map(|candidate| (candidate.0, candidate.1, candidate.2, candidate.3))
            .collect::<Vec<_>>(),
    )?;
    let mut audit = HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit {
        schema_version: "hone-boundary-candidate-audit-v1".to_string(),
        candidate_pair_count: candidates.len(),
        selected_train_component_end_exclusive: train_end,
        selected_validation_component_end_exclusive: validation_end,
        pre_purge_train_entry_count: counts[0],
        pre_purge_validation_entry_count: counts[1],
        pre_purge_sealed_holdout_entry_count: counts[2],
        objective_tuple: objective,
        all_candidate_objectives_sha256,
        audit_sha256: String::new(),
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    let mut records = Vec::with_capacity(entries.len());
    for (component_order_index, component) in components.iter().enumerate() {
        let split = if component_order_index < train_end {
            HistoricalOutcomeOfflineDatasetTransformationSplit::Train
        } else if component_order_index < validation_end {
            HistoricalOutcomeOfflineDatasetTransformationSplit::Validation
        } else {
            HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
        };
        for index in &component.entry_indexes {
            let entry = &entries[*index];
            records.push(
                HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord {
                    dataset_entry_id: entry.entry_id.clone(),
                    dataset_entry_sha256: entry.entry_sha256.clone(),
                    component_id: component.component_id.clone(),
                    component_order_index,
                    split: split.clone(),
                    boundary_objective_tuple: objective,
                    purged_or_embargoed: false,
                    purge_reason: None,
                    price_snapshot_sha256: entry.snapshot_sha256.clone(),
                    common_market_session_index: entry.common_market_session_index,
                    decision_session_date: entry.decision_session_date,
                    outcome_horizon_250_end_date: entry.outcome_horizon_250_end_date,
                    decision_available_at: entry.decision_available_at,
                },
            );
        }
    }
    records.sort_by(|left, right| left.dataset_entry_id.cmp(&right.dataset_entry_id));
    Ok((audit, records))
}

fn apply_purge_and_embargo(
    records: &mut [HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord],
) -> Result<(), String> {
    for (previous, next, label) in [
        (
            HistoricalOutcomeOfflineDatasetTransformationSplit::Train,
            HistoricalOutcomeOfflineDatasetTransformationSplit::Validation,
            "train_to_validation",
        ),
        (
            HistoricalOutcomeOfflineDatasetTransformationSplit::Validation,
            HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout,
            "validation_to_sealed_holdout",
        ),
    ] {
        let boundary = records
            .iter()
            .filter(|record| record.split == next)
            .min_by(|left, right| {
                left.decision_session_date
                    .cmp(&right.decision_session_date)
                    .then_with(|| left.dataset_entry_id.cmp(&right.dataset_entry_id))
            })
            .cloned()
            .ok_or_else(|| "边界后一分区为空".to_string())?;
        for record in records.iter_mut().filter(|record| record.split == previous) {
            if record.outcome_horizon_250_end_date >= boundary.decision_session_date {
                record.purged_or_embargoed = true;
                record.purge_reason = Some(format!(
                    "{label}:prior_250_session_outcome_crosses_boundary"
                ));
            }
        }
        for record in records.iter_mut().filter(|record| record.split == next) {
            if record.decision_session_date <= boundary.outcome_horizon_250_end_date {
                record.purged_or_embargoed = true;
                record.purge_reason =
                    Some(format!("{label}:next_entry_within_250_session_embargo"));
            }
        }
    }
    Ok(())
}

fn validate_envelope(
    envelope: &HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope,
) -> Result<(), String> {
    let active_entries = envelope
        .split_manifest_candidate
        .iter()
        .filter(|record| !record.purged_or_embargoed)
        .count();
    let feature_keys = envelope
        .feature_bundle_candidate
        .iter()
        .map(|record| {
            (
                record.dataset_entry_id.as_str(),
                record.feature_namespace.as_str(),
                record.feature_id.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    let feature_rows_safe = envelope.feature_bundle_candidate.iter().all(|record| {
        record.value_encoding == "explicit_missingness"
            && record.value.is_none()
            && record.is_missing
            && record.missingness_reason == "availability_ambiguous"
            && record.available_at_utc == record.observed_at_utc
            && valid_sha256(&record.artifact_sha256)
            && valid_sha256(&record.transformation_sha256)
    });
    let authority_closed = envelope.output_is_untrusted
        && !envelope.independent_validation_completed
        && !envelope.official_split_manifest_created
        && !envelope.official_feature_bundle_created
        && !envelope.feature_join_performed
        && !envelope.semantic_target_assigned
        && !envelope.copied_to_training_store
        && !envelope.training_started
        && !envelope.reward_written
        && !envelope.shadow_position_written
        && !envelope.order_generated
        && !envelope.broker_accessed
        && !envelope.trade_executed;
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION
        || envelope.entry_count != envelope.split_manifest_candidate.len()
        || envelope.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || envelope.feature_bundle_candidate.len() != active_entries * EXPECTED_FEATURE_COUNT
        || feature_keys.len() != envelope.feature_bundle_candidate.len()
        || envelope.excluded_feature_audit.len() != envelope.entry_count
        || !envelope.sealed_holdout_labels_withheld
        || !envelope.deterministic_projection_only
        || !envelope.explicit_missingness_preserved
        || !feature_rows_safe
        || !authority_closed
        || !valid_sha256(&envelope.boundary_audit.audit_sha256)
    {
        return Err("未验证转换候选结构无效或越权".to_string());
    }
    Ok(())
}

fn new_success_result(
    claim: &HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    envelope: HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope,
    output_sha256: String,
    stdout_bytes: usize,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus::CompletedWithUntrustedCandidateEnvelope,
        exit_code: 0,
        stdout_sha256: output_sha256.clone(),
        stderr_sha256: hash_bytes(&[]),
        stdout_bytes,
        stderr_bytes: 0,
        bounded_error: None,
        output_sha256: Some(output_sha256),
        untrusted_candidate_envelope: Some(envelope),
        ephemeral_directory_removed: true,
        independent_validation_completed: false,
        official_split_manifest_authorized: false,
        official_feature_bundle_authorized: false,
        feature_join_authorized: false,
        semantic_target_authorized: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn new_failed_result(
    claim: &HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    error: &TransformationFailure,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult, String> {
    let bounded = error
        .message
        .chars()
        .take(MAX_ERROR_CHARS)
        .collect::<String>();
    let mut result = HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus::FailedAuthorizationConsumed,
        exit_code: 1,
        stdout_sha256: hash_bytes(&[]),
        stderr_sha256: hash_bytes(bounded.as_bytes()),
        stdout_bytes: 0,
        stderr_bytes: bounded.len(),
        bounded_error: Some(bounded),
        output_sha256: None,
        untrusted_candidate_envelope: None,
        ephemeral_directory_removed: error.ephemeral_directory_removed,
        independent_validation_completed: false,
        official_split_manifest_authorized: false,
        official_feature_bundle_authorized: false,
        feature_join_authorized: false,
        semantic_target_authorized: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    result.result_sha256 = result_fingerprint(&result)?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn validate_claim(
    claim: &HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
) -> Result<(), String> {
    let capabilities_closed = !claim.child_process_spawned
        && !claim.ambient_filesystem_capability_available
        && !claim.ambient_environment_capability_available
        && !claim.network_capability_available
        && !claim.external_tool_capability_available
        && !claim.production_data_capability_available_to_projection
        && !claim.historical_state_mutation_allowed
        && !claim.official_manifest_or_bundle_writes_allowed
        && !claim.semantic_target_writes_allowed
        && !claim.training_writes_allowed
        && !claim.reward_writes_allowed
        && !claim.shadow_writes_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed;
    let hashes = [
        &claim.claim_sha256,
        &claim.authorization_review_sha256,
        &claim.isolated_runner_spec_sha256,
        &claim.runner_artifact_sha256,
        &claim.implementation_sha256,
        &claim.implementation_review_sha256,
        &claim.transformation_spec_sha256,
        &claim.transformation_body_sha256,
        &claim.split_specification_sha256,
        &claim.feature_specification_sha256,
        &claim.dataset_content_sha256,
        &claim.dataset_manifest_sha256,
        &claim.candidate_set_sha256,
        &claim.governance_review_sha256,
    ];
    let ids = [
        &claim.attempt_id,
        &claim.authorization_review_id,
        &claim.isolated_runner_id,
        &claim.implementation_id,
        &claim.implementation_review_id,
        &claim.transformation_spec_id,
        &claim.dataset_id,
        &claim.governance_review_id,
    ];
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.execution_policy_version != EXECUTION_POLICY_VERSION
        || ids.iter().any(|value| !valid_id(value))
        || hashes.iter().any(|value| !valid_sha256(value))
        || claim.attempt_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != claim_fingerprint(claim)?
        || claim.claimed_at >= claim.authorization_valid_until
        || claim.invoked_by.trim().is_empty()
        || claim.isolation_backend != ISOLATION_BACKEND
        || claim.max_wall_clock_seconds != 300
        || claim.max_memory_mib != 2_048
        || claim.max_cpu_millicores != 1_000
        || claim.max_process_count != 1
        || claim.max_output_bytes != 8_388_608
        || !claim.artifact_digest_reverified
        || !claim.current_complete_upstream_chain_revalidated
        || !claim.authorization_consumed
        || !claim.invocation_started
        || !capabilities_closed
    {
        return Err("离线转换执行 claim 无效或越权".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult,
    claim: &HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
) -> Result<(), String> {
    let downstream_closed = !result.independent_validation_completed
        && !result.official_split_manifest_authorized
        && !result.official_feature_bundle_authorized
        && !result.feature_join_authorized
        && !result.semantic_target_authorized
        && !result.training_authorized
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    let output_consistent = match (
        &result.status,
        &result.untrusted_candidate_envelope,
        &result.output_sha256,
        &result.bounded_error,
    ) {
        (
            HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus::CompletedWithUntrustedCandidateEnvelope,
            Some(envelope),
            Some(output_sha256),
            None,
        ) => {
            result.exit_code == 0
                && result.stdout_bytes <= claim.max_output_bytes as usize
                && result.stderr_bytes == 0
                && result.stderr_sha256 == hash_bytes(&[])
                && validate_envelope(envelope).is_ok()
                && hash_serializable(envelope).is_ok_and(|hash| hash == *output_sha256)
                && result.stdout_sha256 == *output_sha256
        }
        (
            HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus::FailedAuthorizationConsumed,
            None,
            None,
            Some(error),
        ) => {
            result.exit_code != 0
                && result.stdout_bytes == 0
                && result.stdout_sha256 == hash_bytes(&[])
                && result.stderr_bytes == error.len()
                && result.stderr_sha256 == hash_bytes(error.as_bytes())
                && error.chars().count() <= MAX_ERROR_CHARS
        }
        _ => false,
    };
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.execution_policy_version != EXECUTION_POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != result_fingerprint(result)?
        || result.attempt_id != claim.attempt_id
        || result.claim_sha256 != claim.claim_sha256
        || result.completed_at < claim.claimed_at
        || result.duration_millis > u64::from(claim.max_wall_clock_seconds) * 1_000
        || !valid_sha256(&result.stdout_sha256)
        || !valid_sha256(&result.stderr_sha256)
        || !output_consistent
        || !downstream_closed
    {
        return Err("离线转换执行结果无效或越权".to_string());
    }
    Ok(())
}

fn claim_binding_matches(
    claim: &HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
) -> bool {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let implementation_review = &runner.implementation_review;
    let specification = &implementation.approved_review.specification;
    let subject = &specification.subject;
    claim.authorization_review_id == authorization.review.review_id
        && claim.authorization_review_sha256 == authorization.review.review_sha256
        && claim.authorization_valid_until == authorization.review.authorization_valid_until
        && claim.isolated_runner_id == runner.isolated_runner_id
        && claim.isolated_runner_spec_sha256 == runner.isolated_runner_spec_sha256
        && claim.runner_artifact_sha256 == runner.runner_artifact_sha256
        && claim.runner_code_revision == runner.runner_code_revision
        && claim.implementation_id == implementation.implementation_id
        && claim.implementation_sha256 == implementation.implementation_sha256
        && claim.implementation_review_id == implementation_review.review_id
        && claim.implementation_review_sha256 == implementation_review.review_sha256
        && claim.transformation_spec_id == specification.transformation_spec_id
        && claim.transformation_spec_sha256 == specification.transformation_spec_sha256
        && claim.transformation_body_sha256 == specification.transformation_body_sha256
        && claim.split_specification_sha256
            == specification
                .split_manifest_specification
                .specification_sha256
        && claim.feature_specification_sha256
            == specification
                .feature_bundle_specification
                .specification_sha256
        && claim.dataset_id == subject.dataset_id
        && claim.dataset_content_sha256 == subject.dataset_content_sha256
        && claim.dataset_manifest_sha256 == subject.manifest_sha256
        && claim.candidate_set_sha256 == subject.candidate_set_sha256
        && claim.governance_review_id == specification.governance_review_id
        && claim.governance_review_sha256 == specification.governance_review_sha256
}

fn claim_fingerprint(
    claim: &HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
) -> Result<String, String> {
    fingerprint_without(claim, &["attempt_id", "claim_sha256"])
}

fn result_fingerprint(
    result: &HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult,
) -> Result<String, String> {
    fingerprint_without(result, &["result_id", "result_sha256"])
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim>,
        Vec<HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<
        HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim,
    >(&execution_root(state).join("claims"))
    .await?;
    let results = read_nested_json::<
        HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptResult,
    >(&execution_root(state).join("results"))
    .await?;
    let mut claim_ids = BTreeSet::new();
    let mut claim_hashes = BTreeSet::new();
    let mut runner_ids = BTreeSet::new();
    let mut authorization_ids = BTreeSet::new();
    for claim in &claims {
        validate_claim(claim)?;
        if !claim_ids.insert(claim.attempt_id.clone())
            || !claim_hashes.insert(claim.claim_sha256.clone())
            || !runner_ids.insert(claim.isolated_runner_id.clone())
            || !authorization_ids.insert(claim.authorization_review_id.clone())
        {
            return Err("离线转换执行登记含重放 claim".to_string());
        }
    }
    let claim_index = claims
        .iter()
        .map(|claim| (claim.attempt_id.as_str(), claim))
        .collect::<HashMap<_, _>>();
    let mut result_ids = BTreeSet::new();
    let mut result_hashes = BTreeSet::new();
    let mut completed = BTreeSet::new();
    for result in &results {
        let claim = claim_index
            .get(result.attempt_id.as_str())
            .ok_or_else(|| "离线转换结果缺少不可变 claim".to_string())?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_hashes.insert(result.result_sha256.clone())
            || !completed.insert(result.attempt_id.clone())
        {
            return Err("离线转换执行登记含重复结果".to_string());
        }
    }
    Ok((claims, results))
}

async fn read_nested_json<T>(root: &Path) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut directories = match tokio::fs::read_dir(root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    while let Some(directory) = directories
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if !directory
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let mut entries = tokio::fs::read_dir(directory.path())
            .await
            .map_err(|error| error.to_string())?;
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
            output.push(serde_json::from_slice(&bytes).map_err(|error| error.to_string())?);
        }
    }
    Ok(output)
}

async fn reverify_current_binary_sha256() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        let executable = std::env::current_exe().map_err(|error| error.to_string())?;
        let mut file = File::open(executable).map_err(|error| error.to_string())?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok::<_, String>(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|error| error.to_string())?
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(hash_bytes(&bytes))
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .chars()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, '-' | '_'))
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn execution_root(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-offline-dataset-transformation-execution-attempts")
}

fn claim_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("claims").join(runner_id)
}

fn result_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("results").join(runner_id)
}

struct TransformationExecutionLock {
    path: PathBuf,
}

impl Drop for TransformationExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    runner_id: &str,
) -> Result<TransformationExecutionLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-outcome-transformation-execution-{runner_id}.lock"
    ));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(TransformationExecutionLock { path }),
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
                return Err("隔离转换首次执行正在由另一请求消费".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("隔离转换首次执行锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid transformation execution path".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "离线转换执行记录已存在，不得覆盖".to_string()
            } else {
                error.to_string()
            }
        })?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        seed: char,
        symbol: &str,
        year: i32,
        session_index: usize,
    ) -> TransformationEntryInput {
        let date = NaiveDate::from_ymd_opt(year, 1, 2).unwrap();
        TransformationEntryInput {
            entry_id: seed.to_string().repeat(32),
            entry_sha256: seed.to_string().repeat(64),
            asset_symbol: symbol.to_string(),
            reconstruction_sha256: seed.to_string().repeat(64),
            snapshot_sha256: seed.to_string().repeat(64),
            source_output_sha256: seed.to_string().repeat(64),
            source_validation_sha256: seed.to_string().repeat(64),
            protocol_sha256: "f".repeat(64),
            decision_available_at: DateTime::from_naive_utc_and_offset(
                date.and_hms_opt(21, 0, 0).unwrap(),
                Utc,
            ),
            decision_session_date: date,
            outcome_horizon_250_end_date: NaiveDate::from_ymd_opt(year, 12, 30).unwrap(),
            common_market_session_index: session_index,
        }
    }

    fn spaced_inputs() -> Vec<TransformationEntryInput> {
        vec![
            input('1', "AAA", 2018, 10),
            input('2', "BBB", 2020, 20),
            input('3', "CCC", 2022, 30),
            input('4', "DDD", 2024, 40),
            input('5', "EEE", 2026, 50),
            input('6', "FFF", 2028, 60),
        ]
    }

    #[test]
    fn connected_components_keep_same_company_and_source_family_indivisible() {
        let mut entries = spaced_inputs();
        entries[1].asset_symbol = entries[0].asset_symbol.clone();
        entries[3].source_output_sha256 = entries[2].source_output_sha256.clone();
        let components = connected_components(&entries, "split-v1").unwrap();
        assert_eq!(components.len(), 4);
        assert!(
            components
                .iter()
                .any(|component| component.entry_indexes.len() == 2)
        );
    }

    #[test]
    fn boundary_objective_is_deterministic_and_components_never_cross_splits() {
        let entries = spaced_inputs();
        let components = connected_components(&entries, "split-v1").unwrap();
        let (first_audit, first_records) =
            select_and_apply_boundaries(&entries, &components).unwrap();
        let (second_audit, second_records) =
            select_and_apply_boundaries(&entries, &components).unwrap();
        assert_eq!(first_audit, second_audit);
        assert_eq!(first_records, second_records);
        let by_component =
            first_records
                .iter()
                .fold(BTreeMap::<_, BTreeSet<_>>::new(), |mut map, record| {
                    map.entry(&record.component_id)
                        .or_default()
                        .insert(&record.split);
                    map
                });
        assert!(by_component.values().all(|splits| splits.len() == 1));
    }

    #[test]
    fn purge_and_embargo_fail_closed_when_horizons_overlap_boundaries() {
        let mut entries = spaced_inputs();
        for entry in &mut entries {
            entry.outcome_horizon_250_end_date = NaiveDate::from_ymd_opt(2030, 1, 2).unwrap();
        }
        let components = connected_components(&entries, "split-v1").unwrap();
        let (_, mut records) = select_and_apply_boundaries(&entries, &components).unwrap();
        apply_purge_and_embargo(&mut records).unwrap();
        assert!(records.iter().any(|record| record.purged_or_embargoed));
        assert!(!records.iter().any(|record| {
            record.split == HistoricalOutcomeOfflineDatasetTransformationSplit::Validation
                && !record.purged_or_embargoed
        }));
    }

    #[test]
    fn explicit_feature_missingness_never_backfills_a_value() {
        let entry = input('a', "AAA", 2020, 1);
        let definition = PointInTimeFeatureDefinition {
            namespace: "company_fundamentals".to_string(),
            feature_id: "moat_strength".to_string(),
            value_kind: "human_reviewed_ordinal_state".to_string(),
            source_authority_contract: "reviewed".to_string(),
        };
        let feature = explicit_missing_feature(&entry, &definition, &"a".repeat(64));
        assert!(feature.is_missing);
        assert!(feature.value.is_none());
        assert_eq!(feature.missingness_reason, "availability_ambiguous");
    }

    #[test]
    fn failed_result_consumes_claim_and_has_no_downstream_authority() {
        let claimed_at = Utc::now();
        let mut claim = HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim {
            schema_version: CLAIM_SCHEMA_VERSION.to_string(),
            execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
            attempt_id: String::new(),
            claim_sha256: String::new(),
            authorization_review_id: "authorization-1".to_string(),
            authorization_review_sha256: "a".repeat(64),
            authorization_valid_until: claimed_at + chrono::Duration::hours(1),
            isolated_runner_id: "runner-1".to_string(),
            isolated_runner_spec_sha256: "b".repeat(64),
            runner_artifact_sha256: "c".repeat(64),
            runner_code_revision: "oldwang@stage31".to_string(),
            implementation_id: "implementation-1".to_string(),
            implementation_sha256: "d".repeat(64),
            implementation_review_id: "review-1".to_string(),
            implementation_review_sha256: "e".repeat(64),
            transformation_spec_id: "spec-1".to_string(),
            transformation_spec_sha256: "f".repeat(64),
            transformation_body_sha256: "1".repeat(64),
            split_specification_sha256: "2".repeat(64),
            feature_specification_sha256: "3".repeat(64),
            dataset_id: "dataset-1".to_string(),
            dataset_content_sha256: "4".repeat(64),
            dataset_manifest_sha256: "5".repeat(64),
            candidate_set_sha256: "6".repeat(64),
            governance_review_id: "governance-1".to_string(),
            governance_review_sha256: "7".repeat(64),
            max_wall_clock_seconds: 300,
            max_memory_mib: 2_048,
            max_cpu_millicores: 1_000,
            max_process_count: 1,
            max_output_bytes: 8_388_608,
            claimed_at,
            invoked_by: "executor-admin".to_string(),
            isolation_backend: ISOLATION_BACKEND.to_string(),
            artifact_digest_reverified: true,
            current_complete_upstream_chain_revalidated: true,
            authorization_consumed: true,
            invocation_started: true,
            child_process_spawned: false,
            ambient_filesystem_capability_available: false,
            ambient_environment_capability_available: false,
            network_capability_available: false,
            external_tool_capability_available: false,
            production_data_capability_available_to_projection: false,
            historical_state_mutation_allowed: false,
            official_manifest_or_bundle_writes_allowed: false,
            semantic_target_writes_allowed: false,
            training_writes_allowed: false,
            reward_writes_allowed: false,
            shadow_writes_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        assert!(validate_claim(&claim).is_ok());
        let result = new_failed_result(
            &claim,
            claimed_at,
            1,
            &TransformationFailure::before_staging("deterministic failure"),
        )
        .unwrap();
        assert_eq!(result.status, HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptStatus::FailedAuthorizationConsumed);
        assert!(result.untrusted_candidate_envelope.is_none());
        assert!(!result.training_authorized);
    }

    #[test]
    fn claim_and_result_fingerprints_reject_tampering() {
        let claimed_at = Utc::now();
        let mut claim = HistoricalOutcomeOfflineDatasetTransformationExecutionAttemptClaim {
            schema_version: CLAIM_SCHEMA_VERSION.to_string(),
            execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
            attempt_id: String::new(),
            claim_sha256: String::new(),
            authorization_review_id: "authorization-1".to_string(),
            authorization_review_sha256: "a".repeat(64),
            authorization_valid_until: claimed_at + chrono::Duration::hours(1),
            isolated_runner_id: "runner-1".to_string(),
            isolated_runner_spec_sha256: "b".repeat(64),
            runner_artifact_sha256: "c".repeat(64),
            runner_code_revision: "oldwang@stage31".to_string(),
            implementation_id: "implementation-1".to_string(),
            implementation_sha256: "d".repeat(64),
            implementation_review_id: "review-1".to_string(),
            implementation_review_sha256: "e".repeat(64),
            transformation_spec_id: "spec-1".to_string(),
            transformation_spec_sha256: "f".repeat(64),
            transformation_body_sha256: "1".repeat(64),
            split_specification_sha256: "2".repeat(64),
            feature_specification_sha256: "3".repeat(64),
            dataset_id: "dataset-1".to_string(),
            dataset_content_sha256: "4".repeat(64),
            dataset_manifest_sha256: "5".repeat(64),
            candidate_set_sha256: "6".repeat(64),
            governance_review_id: "governance-1".to_string(),
            governance_review_sha256: "7".repeat(64),
            max_wall_clock_seconds: 300,
            max_memory_mib: 2_048,
            max_cpu_millicores: 1_000,
            max_process_count: 1,
            max_output_bytes: 8_388_608,
            claimed_at,
            invoked_by: "executor-admin".to_string(),
            isolation_backend: ISOLATION_BACKEND.to_string(),
            artifact_digest_reverified: true,
            current_complete_upstream_chain_revalidated: true,
            authorization_consumed: true,
            invocation_started: true,
            child_process_spawned: false,
            ambient_filesystem_capability_available: false,
            ambient_environment_capability_available: false,
            network_capability_available: false,
            external_tool_capability_available: false,
            production_data_capability_available_to_projection: false,
            historical_state_mutation_allowed: false,
            official_manifest_or_bundle_writes_allowed: false,
            semantic_target_writes_allowed: false,
            training_writes_allowed: false,
            reward_writes_allowed: false,
            shadow_writes_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
        claim.claim_sha256 = claim_fingerprint(&claim).unwrap();
        claim.attempt_id = claim.claim_sha256[..32].to_string();
        let mut tampered = claim.clone();
        tampered.dataset_content_sha256 = "8".repeat(64);
        assert!(validate_claim(&tampered).is_err());
        let mut result = new_failed_result(
            &claim,
            claimed_at,
            1,
            &TransformationFailure::before_staging("failure"),
        )
        .unwrap();
        result.stderr_bytes += 1;
        assert!(validate_result(&result, &claim).is_err());
    }
}
