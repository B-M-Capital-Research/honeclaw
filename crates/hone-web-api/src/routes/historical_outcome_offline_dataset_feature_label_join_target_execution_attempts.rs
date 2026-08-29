//! One-shot isolated feature-label join/target execution attempts.
//!
//! One exact, current and unexpired Stage 41 authorization is consumed by a
//! create-once claim before the fixed projection runs. The projection receives
//! only the exact independently validated official split/feature artifacts and
//! their exact current raw-outcome dataset. It emits a content-addressed,
//! untrusted candidate envelope for a later independent validator.
//!
//! Only train rows contain the nine raw f64 target bit patterns. Validation and
//! sealed-holdout target values remain withheld and are represented only by a
//! commitment. No generic label or training store is opened; no official
//! joined dataset, training row, reward, shadow position, order, broker access
//! or trade is created. Success and failure both consume the authorization.

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
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::historical_outcome_offline_dataset_feature_label_join_target_first_execution_authorizations::{
    ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
    execution_attempt_eligible_join_target_authorizations,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_specs::{
    HistoricalOutcomeFeatureLabelJoinSpecification, HistoricalOutcomeSemanticTargetDefinition,
    HistoricalOutcomeSemanticTargetSpecification,
};
use super::historical_outcome_offline_dataset_transformation_execution_attempts::{
    HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord,
    HistoricalOutcomeOfflineDatasetTransformationSplit,
};
use super::historical_outcome_offline_dataset_transformation_official_artifact_materializations::{
    HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
};
use super::historical_outcome_offline_dataset_transformation_official_artifact_output_validations::{
    IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
    independently_validated_official_artifact_pairs_for_join_target_specification,
};
use super::historical_outcome_offline_datasets::{
    HistoricalOutcomeOfflineDataset, exact_current_historical_outcome_offline_dataset_for_transformation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-execution-attempt-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-execution-attempt-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-execution-attempt-result-v1";
const ENVELOPE_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-untrusted-candidate-envelope-v1";
const EXECUTION_POLICY_VERSION: &str = "hone-feature-label-join-target-one-shot-pure-projection-v1";
const ISOLATION_BACKEND: &str =
    "fixed-sealed-feature-label-join-target-pure-function-no-ambient-capabilities-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];
const MAX_ERROR_CHARS: usize = 1_200;

#[derive(Debug)]
struct JoinTargetFailure {
    message: String,
    ephemeral_directory_removed: bool,
}

impl JoinTargetFailure {
    fn before_staging(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            ephemeral_directory_removed: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct InvokeHistoricalOutcomeFeatureLabelJoinTargetOnceRequest {
    expected_first_execution_authorization_review_id: String,
    expected_first_execution_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_runner_code_revision: String,
    expected_implementation_id: String,
    expected_implementation_sha256: String,
    expected_implementation_review_sha256: String,
    expected_specification_id: String,
    expected_specification_sha256: String,
    expected_specification_body_sha256: String,
    expected_join_specification_sha256: String,
    expected_target_specification_sha256: String,
    expected_validation_id: String,
    expected_validation_sha256: String,
    expected_split_manifest_sha256: String,
    expected_feature_bundle_sha256: String,
    expected_combined_artifact_sha256: String,
    expected_dataset_id: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    create_once_claim_and_failure_consumes_confirmed: bool,
    exact_one_to_one_join_and_nine_raw_target_projection_confirmed: bool,
    validation_and_sealed_holdout_target_values_withheld_confirmed: bool,
    no_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetVisibility {
    TrainCandidateRawTargets,
    ValidationTargetsWithheld,
    SealedHoldoutTargetsWithheld,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetValue {
    pub target_id: String,
    pub horizon_market_sessions: u16,
    pub source_metric_field: String,
    pub value_kind: String,
    pub unit: String,
    pub role: String,
    pub exact_f64_bits_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetCandidateRow {
    pub dataset_entry_id: String,
    pub dataset_entry_sha256: String,
    pub split: HistoricalOutcomeOfflineDatasetTransformationSplit,
    pub component_id: String,
    pub decision_available_at: DateTime<Utc>,
    pub feature_records: Vec<HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord>,
    pub target_visibility: HistoricalOutcomeFeatureLabelJoinTargetVisibility,
    pub target_vector: Option<Vec<HistoricalOutcomeFeatureLabelJoinTargetValue>>,
    pub target_commitment_sha256: String,
    pub source_binding_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit {
    pub dataset_entry_id: String,
    pub dataset_entry_sha256: String,
    pub split: HistoricalOutcomeOfflineDatasetTransformationSplit,
    pub component_id: String,
    pub purge_reason: String,
    pub feature_record_count: usize,
    pub target_values_opened: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope {
    pub schema_version: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub runner_id: String,
    pub runner_spec_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_review_sha256: String,
    pub specification_id: String,
    pub specification_sha256: String,
    pub specification_body_sha256: String,
    pub join_specification_sha256: String,
    pub target_specification_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub split_manifest_sha256: String,
    pub feature_bundle_sha256: String,
    pub combined_artifact_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub dataset_entry_count: usize,
    pub active_candidate_row_count: usize,
    pub excluded_purge_or_embargo_row_count: usize,
    pub feature_catalog_count: usize,
    pub target_count: usize,
    pub train_target_vector_count: usize,
    pub validation_target_withheld_count: usize,
    pub sealed_holdout_target_withheld_count: usize,
    pub rows: Vec<HistoricalOutcomeFeatureLabelJoinTargetCandidateRow>,
    pub excluded_rows: Vec<HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit>,
    pub one_to_one_join_satisfied: bool,
    pub exact_raw_f64_bits_preserved: bool,
    pub official_split_authority_preserved: bool,
    pub point_in_time_and_explicit_missingness_preserved: bool,
    pub validation_targets_withheld: bool,
    pub sealed_holdout_targets_withheld: bool,
    pub output_is_untrusted: bool,
    pub independent_output_validation_completed: bool,
    pub official_joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub training_started: bool,
    pub reward_written: bool,
    pub shadow_position_written: bool,
    pub order_generated: bool,
    pub broker_accessed: bool,
    pub trade_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim {
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
    pub specification_review_id: String,
    pub specification_review_sha256: String,
    pub specification_id: String,
    pub specification_sha256: String,
    pub specification_body_sha256: String,
    pub join_specification_sha256: String,
    pub target_specification_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub split_manifest_sha256: String,
    pub feature_bundle_sha256: String,
    pub combined_artifact_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub max_wall_clock_seconds: u16,
    pub max_memory_mib: u32,
    pub max_cpu_millicores: u16,
    pub max_process_count: u8,
    pub max_output_bytes: u32,
    pub claimed_at: DateTime<Utc>,
    pub invoked_by: String,
    pub isolation_backend: String,
    pub artifact_digest_reverified: bool,
    pub exact_current_complete_upstream_chain_revalidated: bool,
    pub exact_official_artifact_pair_and_dataset_revalidated: bool,
    pub authorization_consumed: bool,
    pub invocation_started: bool,
    pub exact_bound_raw_outcome_read_allowed: bool,
    pub generic_label_store_read_allowed: bool,
    pub training_store_read_allowed: bool,
    pub child_process_spawned: bool,
    pub ambient_filesystem_capability_available: bool,
    pub ambient_environment_capability_available: bool,
    pub network_capability_available: bool,
    pub external_tool_capability_available: bool,
    pub production_data_capability_available: bool,
    pub historical_state_mutation_allowed: bool,
    pub official_joined_dataset_write_allowed: bool,
    pub training_write_allowed: bool,
    pub reward_write_allowed: bool,
    pub shadow_write_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus {
    CompletedWithUntrustedJoinedTargetCandidateEnvelope,
    FailedAuthorizationConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub duration_millis: u64,
    pub status: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus,
    pub exit_code: i32,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub bounded_error: Option<String>,
    pub output_sha256: Option<String>,
    pub untrusted_candidate_envelope:
        Option<HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope>,
    pub ephemeral_directory_removed: bool,
    pub independent_output_validation_completed: bool,
    pub official_joined_dataset_authorized: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptItem {
    pub claim: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim,
    pub result: Option<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult>,
    pub current_authorization_binding: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry {
    pub schema_version: String,
    pub execution_policy_version: String,
    pub isolation_backend: String,
    pub invocation_endpoint_available: bool,
    pub invocation_eligible_authorization_count: usize,
    pub eligible_authorizations:
        Vec<ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization>,
    pub attempt_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_candidate_envelope_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
    pub attempts: Vec<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptItem>,
    pub official_joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptReadinessSummary {
    pub invocation_eligible_authorization_count: usize,
    pub attempt_count: usize,
    pub completed_attempt_count: usize,
    pub failed_attempt_count: usize,
    pub untrusted_candidate_envelope_count: usize,
    pub independent_output_validation_eligible_count: usize,
    pub execution_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt {
    pub claim: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim,
    pub result: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult,
}

#[derive(Debug, Clone)]
struct RawMetricBits {
    asset_return: u64,
    excess_return: u64,
    asset_max_drawdown: u64,
}

#[derive(Debug, Clone)]
struct RawOutcomeEntryInput {
    dataset_entry_id: String,
    dataset_entry_sha256: String,
    decision_available_at: DateTime<Utc>,
    metrics: BTreeMap<u16, RawMetricBits>,
}

#[derive(Debug, Clone)]
struct ProjectionBindings {
    authorization_review_id: String,
    authorization_review_sha256: String,
    runner_id: String,
    runner_spec_sha256: String,
    implementation_id: String,
    implementation_sha256: String,
    implementation_review_sha256: String,
    specification_id: String,
    specification_sha256: String,
    specification_body_sha256: String,
    join_specification_sha256: String,
    target_specification_sha256: String,
    validation_id: String,
    validation_sha256: String,
    split_manifest_sha256: String,
    feature_bundle_sha256: String,
    combined_artifact_sha256: String,
    dataset_id: String,
    dataset_content_sha256: String,
    dataset_manifest_sha256: String,
    candidate_set_sha256: String,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_execution_attempts(
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
            warn!(%error, "historical outcome join/target execution registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果 join/target 一次性执行记录暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_invoke_historical_outcome_feature_label_join_target_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_runner_id): AxumPath<String>,
    Json(request): Json<InvokeHistoricalOutcomeFeatureLabelJoinTargetOnceRequest>,
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

pub(crate) async fn historical_outcome_feature_label_join_target_execution_attempt_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptReadinessSummary {
            invocation_eligible_authorization_count: registry
                .invocation_eligible_authorization_count,
            attempt_count: registry.attempt_count,
            completed_attempt_count: registry.completed_attempt_count,
            failed_attempt_count: registry.failed_attempt_count,
            untrusted_candidate_envelope_count: registry.untrusted_candidate_envelope_count,
            independent_output_validation_eligible_count: registry
                .independent_output_validation_eligible_count,
            execution_status: registry.execution_status,
        },
    )
}

pub(crate) async fn completed_join_target_execution_attempts_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt>, String> {
    let (claims, results) = read_attempt_records(state).await?;
    let result_index = results
        .into_iter()
        .filter(|result| {
            result.status
                == HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus::CompletedWithUntrustedJoinedTargetCandidateEnvelope
                && result.untrusted_candidate_envelope.is_some()
                && result.output_sha256.is_some()
        })
        .map(|result| (result.attempt_id.clone(), result))
        .collect::<HashMap<_, _>>();
    let mut output = claims
        .into_iter()
        .filter_map(|claim| {
            result_index.get(&claim.attempt_id).cloned().map(|result| {
                CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt { claim, result }
            })
        })
        .collect::<Vec<_>>();
    output.sort_by(|left, right| right.claim.claimed_at.cmp(&left.claim.claimed_at));
    Ok(output)
}

pub(crate) async fn completed_join_target_execution_attempt_for_independent_validation(
    state: &AppState,
    attempt_id: &str,
    claim_sha256: &str,
    result_sha256: &str,
    output_sha256: &str,
) -> Result<Option<CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt>, String> {
    Ok(
        completed_join_target_execution_attempts_for_independent_validation(state)
            .await?
            .into_iter()
            .find(|attempt| {
                attempt.claim.attempt_id == attempt_id
                    && attempt.claim.claim_sha256 == claim_sha256
                    && attempt.result.result_sha256 == result_sha256
                    && attempt.result.output_sha256.as_deref() == Some(output_sha256)
                    && attempt
                        .result
                        .untrusted_candidate_envelope
                        .as_ref()
                        .and_then(|envelope| hash_serializable(envelope).ok())
                        .is_some_and(|hash| hash == output_sha256)
            }),
    )
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry, String> {
    let authorizations =
        execution_attempt_eligible_join_target_authorizations(state, Utc::now()).await?;
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
            HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptItem {
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
    let eligible_authorizations = authorizations
        .into_iter()
        .filter(|authorization| !consumed.contains(authorization.review.review_id.as_str()))
        .collect::<Vec<_>>();
    let invocation_eligible_authorization_count = eligible_authorizations.len();
    let attempt_count = attempts.len();
    let completed_attempt_count = attempts
        .iter()
        .filter(|item| {
            item.result.as_ref().is_some_and(|result| {
                result.status
                    == HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus::CompletedWithUntrustedJoinedTargetCandidateEnvelope
            })
        })
        .count();
    let failed_attempt_count = attempts
        .iter()
        .filter(|item| {
            item.result.as_ref().is_some_and(|result| {
                result.status
                    == HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus::FailedAuthorizationConsumed
            })
        })
        .count();
    let untrusted_candidate_envelope_count = attempts
        .iter()
        .filter(|item| {
            item.result
                .as_ref()
                .is_some_and(|result| result.untrusted_candidate_envelope.is_some())
        })
        .count();
    let independent_output_validation_eligible_count = untrusted_candidate_envelope_count;
    let execution_status = if attempts.iter().any(|item| item.result.is_none()) {
        "join_target_claimed_incomplete_fail_closed"
    } else if untrusted_candidate_envelope_count > 0 {
        "untrusted_join_target_candidate_waiting_independent_output_validation"
    } else if failed_attempt_count > 0 {
        "join_target_execution_failed_authorization_consumed"
    } else if invocation_eligible_authorization_count > 0 {
        "ready_for_one_shot_isolated_join_target_projection"
    } else {
        "waiting_for_unexpired_unclaimed_first_execution_authorization"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptRegistry {
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
        independent_output_validation_eligible_count,
        execution_status: execution_status.to_string(),
        attempts,
        official_joined_dataset_created: false,
        copied_to_training_store: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 42 阶段最多消费一条当前、未过期且从未 claim 的 join/target 授权；先写不可变 claim，再只读取精确绑定的正式 split、65 项点时 feature bundle 和当前原始结果数据集。train 行投影九项原始 f64 位模式，validation 与 sealed holdout 只保留承诺。成功或失败都消费授权；输出仍不可信，不是正式 joined dataset 或训练数据，不训练、不奖励、不建立影子组合、不生成订单、不访问券商或交易。".to_string(),
    })
}

async fn invoke_once(
    state: &AppState,
    admin_id: &str,
    isolated_runner_id: &str,
    request: InvokeHistoricalOutcomeFeatureLabelJoinTargetOnceRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult, String> {
    if !valid_id(isolated_runner_id) {
        return Err("join/target 隔离 runner ID 无效".to_string());
    }
    verify_invocation_confirmations(&request)?;
    let _lock = acquire_lock(state, isolated_runner_id).await?;
    let authorization = execution_attempt_eligible_join_target_authorizations(state, Utc::now())
        .await?
        .into_iter()
        .find(|value| value.runner.isolated_runner_id == isolated_runner_id)
        .ok_or_else(|| "当前没有这条 join/target runner 的未过期一次性授权".to_string())?;
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
        return Err("当前运行制品与独立复核的 join/target runner SHA-256 不一致".to_string());
    }
    let specification = &authorization
        .runner
        .implementation
        .approved_review
        .specification;
    let artifact_pair = exact_artifact_pair(state, specification).await?;
    let dataset = exact_current_historical_outcome_offline_dataset_for_transformation(
        state,
        &specification.dataset_id,
        &specification.dataset_content_sha256,
        &specification.dataset_manifest_sha256,
        &specification.candidate_set_sha256,
    )
    .await?
    .ok_or_else(|| "join/target 绑定的原始结果数据集不再是精确当前完整集合".to_string())?;
    verify_exact_inputs(&authorization, &artifact_pair, &dataset)?;
    let raw_inputs = raw_outcome_inputs(&dataset)?;
    let bindings = projection_bindings(&authorization);

    let claim = new_claim(&authorization, admin_id, Utc::now())?;
    write_immutable_json(
        &claim_directory(state, isolated_runner_id).join(format!("{}.json", claim.attempt_id)),
        &claim,
    )
    .await?;

    let started = Instant::now();
    let execution = execute_fixed_projection(
        &authorization,
        &bindings,
        &raw_inputs,
        &artifact_pair.artifact_pair.split_manifest,
        &artifact_pair.artifact_pair.feature_bundle,
        &specification.join_specification,
        &specification.target_specification,
    )
    .await;
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

fn verify_invocation_confirmations(
    request: &InvokeHistoricalOutcomeFeatureLabelJoinTargetOnceRequest,
) -> Result<(), String> {
    if !request.create_once_claim_and_failure_consumes_confirmed
        || !request.exact_one_to_one_join_and_nine_raw_target_projection_confirmed
        || !request.validation_and_sealed_holdout_target_values_withheld_confirmed
        || !request.no_training_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err("必须确认一次性消费、精确连接、目标隔离和零下游权限".to_string());
    }
    Ok(())
}

fn verify_request(
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
    request: &InvokeHistoricalOutcomeFeatureLabelJoinTargetOnceRequest,
) -> Result<(), String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let implementation_review = &runner.implementation_review;
    let specification_review = &implementation.approved_review;
    let specification = &specification_review.specification;
    if request.expected_first_execution_authorization_review_id != authorization.review.review_id
        || request.expected_first_execution_authorization_review_sha256
            != authorization.review.review_sha256
        || request.expected_isolated_runner_spec_sha256 != runner.isolated_runner_spec_sha256
        || request.expected_runner_artifact_sha256 != runner.runner_artifact_sha256
        || request.expected_runner_code_revision != runner.runner_code_revision
        || request.expected_implementation_id != implementation.implementation_id
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_review_sha256 != implementation_review.review_sha256
        || request.expected_specification_id != specification.specification_id
        || request.expected_specification_sha256 != specification.specification_sha256
        || request.expected_specification_body_sha256 != specification.specification_body_sha256
        || request.expected_join_specification_sha256
            != specification.join_specification.specification_sha256
        || request.expected_target_specification_sha256
            != specification.target_specification.specification_sha256
        || request.expected_validation_id != specification.validation_id
        || request.expected_validation_sha256 != specification.validation_sha256
        || request.expected_split_manifest_sha256 != specification.split_manifest_sha256
        || request.expected_feature_bundle_sha256 != specification.feature_bundle_sha256
        || request.expected_combined_artifact_sha256 != specification.combined_artifact_sha256
        || request.expected_dataset_id != specification.dataset_id
        || request.expected_dataset_content_sha256 != specification.dataset_content_sha256
        || request.expected_dataset_manifest_sha256 != specification.dataset_manifest_sha256
        || request.expected_candidate_set_sha256 != specification.candidate_set_sha256
    {
        return Err(
            "授权、runner、实现、规范、正式工件或数据集绑定已经变化，请刷新后重试".to_string(),
        );
    }
    Ok(())
}

async fn exact_artifact_pair(
    state: &AppState,
    specification: &super::historical_outcome_offline_dataset_feature_label_join_target_specs::HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
) -> Result<IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair, String> {
    independently_validated_official_artifact_pairs_for_join_target_specification(state)
        .await?
        .into_iter()
        .find(|pair| {
            pair.validation.validation_id == specification.validation_id
                && pair.validation.validation_sha256 == specification.validation_sha256
                && pair.artifact_pair.split_manifest.manifest_sha256
                    == specification.split_manifest_sha256
                && pair.artifact_pair.feature_bundle.feature_bundle_sha256
                    == specification.feature_bundle_sha256
                && pair
                    .artifact_pair
                    .result
                    .combined_artifact_sha256
                    .as_deref()
                    == Some(specification.combined_artifact_sha256.as_str())
        })
        .ok_or_else(|| "找不到规范精确绑定且仍有效的独立校验正式工件对".to_string())
}

fn verify_exact_inputs(
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
    pair: &IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
    dataset: &HistoricalOutcomeOfflineDataset,
) -> Result<(), String> {
    let specification = &authorization
        .runner
        .implementation
        .approved_review
        .specification;
    let manifest = &pair.artifact_pair.split_manifest;
    let bundle = &pair.artifact_pair.feature_bundle;
    let exact = pair.validation.validation_id == specification.validation_id
        && pair.validation.validation_sha256 == specification.validation_sha256
        && manifest.manifest_sha256 == specification.split_manifest_sha256
        && bundle.feature_bundle_sha256 == specification.feature_bundle_sha256
        && pair
            .artifact_pair
            .result
            .combined_artifact_sha256
            .as_deref()
            == Some(specification.combined_artifact_sha256.as_str())
        && dataset.dataset_id == specification.dataset_id
        && dataset.dataset_content_sha256 == specification.dataset_content_sha256
        && dataset.manifest_sha256 == specification.dataset_manifest_sha256
        && dataset.candidate_set_sha256 == specification.candidate_set_sha256
        && manifest.dataset_id == dataset.dataset_id
        && manifest.dataset_content_sha256 == dataset.dataset_content_sha256
        && manifest.dataset_manifest_sha256 == dataset.manifest_sha256
        && manifest.candidate_set_sha256 == dataset.candidate_set_sha256
        && bundle.dataset_id == dataset.dataset_id
        && bundle.dataset_content_sha256 == dataset.dataset_content_sha256
        && bundle.dataset_manifest_sha256 == dataset.manifest_sha256
        && bundle.candidate_set_sha256 == dataset.candidate_set_sha256
        && dataset.entries.iter().all(|entry| {
            entry.raw_outcome_only
                && !entry.feature_vector_present
                && !entry.semantic_target_assigned
                && !entry.split_assigned
                && !entry.reward_present
        });
    if !exact {
        return Err("重新打开的正式工件或数据集与授权完整绑定不一致".to_string());
    }
    Ok(())
}

fn raw_outcome_inputs(
    dataset: &HistoricalOutcomeOfflineDataset,
) -> Result<Vec<RawOutcomeEntryInput>, String> {
    let mut ids = BTreeSet::new();
    dataset
        .entries
        .iter()
        .map(|entry| {
            if !ids.insert(entry.entry_id.clone())
                || entry.raw_validated_metrics.len() != REQUIRED_HORIZONS.len()
            {
                return Err(format!(
                    "数据集条目 {} 重复或结果窗口不完整",
                    entry.entry_id
                ));
            }
            let mut metrics = BTreeMap::new();
            for metric in &entry.raw_validated_metrics {
                if !REQUIRED_HORIZONS.contains(&metric.horizon_market_sessions)
                    || !metric.asset_return.is_finite()
                    || !metric.excess_return.is_finite()
                    || !metric.asset_max_drawdown.is_finite()
                    || metrics
                        .insert(
                            metric.horizon_market_sessions,
                            RawMetricBits {
                                asset_return: metric.asset_return.to_bits(),
                                excess_return: metric.excess_return.to_bits(),
                                asset_max_drawdown: metric.asset_max_drawdown.to_bits(),
                            },
                        )
                        .is_some()
                {
                    return Err(format!(
                        "数据集条目 {} 的原始结果无效或重复",
                        entry.entry_id
                    ));
                }
            }
            if metrics.keys().copied().collect::<Vec<_>>() != REQUIRED_HORIZONS {
                return Err(format!(
                    "数据集条目 {} 的 20/60/250 日窗口不完整",
                    entry.entry_id
                ));
            }
            Ok(RawOutcomeEntryInput {
                dataset_entry_id: entry.entry_id.clone(),
                dataset_entry_sha256: entry.entry_sha256.clone(),
                decision_available_at: entry.decision_available_at,
                metrics,
            })
        })
        .collect()
}

fn projection_bindings(
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
) -> ProjectionBindings {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let specification = &implementation.approved_review.specification;
    ProjectionBindings {
        authorization_review_id: authorization.review.review_id.clone(),
        authorization_review_sha256: authorization.review.review_sha256.clone(),
        runner_id: runner.isolated_runner_id.clone(),
        runner_spec_sha256: runner.isolated_runner_spec_sha256.clone(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_review_sha256: runner.implementation_review.review_sha256.clone(),
        specification_id: specification.specification_id.clone(),
        specification_sha256: specification.specification_sha256.clone(),
        specification_body_sha256: specification.specification_body_sha256.clone(),
        join_specification_sha256: specification
            .join_specification
            .specification_sha256
            .clone(),
        target_specification_sha256: specification
            .target_specification
            .specification_sha256
            .clone(),
        validation_id: specification.validation_id.clone(),
        validation_sha256: specification.validation_sha256.clone(),
        split_manifest_sha256: specification.split_manifest_sha256.clone(),
        feature_bundle_sha256: specification.feature_bundle_sha256.clone(),
        combined_artifact_sha256: specification.combined_artifact_sha256.clone(),
        dataset_id: specification.dataset_id.clone(),
        dataset_content_sha256: specification.dataset_content_sha256.clone(),
        dataset_manifest_sha256: specification.dataset_manifest_sha256.clone(),
        candidate_set_sha256: specification.candidate_set_sha256.clone(),
    }
}

fn new_claim(
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
    admin_id: &str,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim, String> {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let implementation_review = &runner.implementation_review;
    let specification_review = &implementation.approved_review;
    let specification = &specification_review.specification;
    let contract = &runner.runner_contract;
    let mut claim = HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim {
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
        implementation_review_id: implementation_review.review_id.clone(),
        implementation_review_sha256: implementation_review.review_sha256.clone(),
        specification_review_id: specification_review.review_id.clone(),
        specification_review_sha256: specification_review.review_sha256.clone(),
        specification_id: specification.specification_id.clone(),
        specification_sha256: specification.specification_sha256.clone(),
        specification_body_sha256: specification.specification_body_sha256.clone(),
        join_specification_sha256: specification
            .join_specification
            .specification_sha256
            .clone(),
        target_specification_sha256: specification
            .target_specification
            .specification_sha256
            .clone(),
        validation_id: specification.validation_id.clone(),
        validation_sha256: specification.validation_sha256.clone(),
        split_manifest_sha256: specification.split_manifest_sha256.clone(),
        feature_bundle_sha256: specification.feature_bundle_sha256.clone(),
        combined_artifact_sha256: specification.combined_artifact_sha256.clone(),
        dataset_id: specification.dataset_id.clone(),
        dataset_content_sha256: specification.dataset_content_sha256.clone(),
        dataset_manifest_sha256: specification.dataset_manifest_sha256.clone(),
        candidate_set_sha256: specification.candidate_set_sha256.clone(),
        max_wall_clock_seconds: contract.maximum_wall_clock_seconds,
        max_memory_mib: contract.maximum_memory_mebibytes,
        max_cpu_millicores: contract.maximum_cpu_millicores,
        max_process_count: contract.maximum_process_count,
        max_output_bytes: contract.maximum_output_bytes,
        claimed_at,
        invoked_by: admin_id.to_string(),
        isolation_backend: ISOLATION_BACKEND.to_string(),
        artifact_digest_reverified: true,
        exact_current_complete_upstream_chain_revalidated: true,
        exact_official_artifact_pair_and_dataset_revalidated: true,
        authorization_consumed: true,
        invocation_started: true,
        exact_bound_raw_outcome_read_allowed: true,
        generic_label_store_read_allowed: false,
        training_store_read_allowed: false,
        child_process_spawned: false,
        ambient_filesystem_capability_available: false,
        ambient_environment_capability_available: false,
        network_capability_available: false,
        external_tool_capability_available: false,
        production_data_capability_available: false,
        historical_state_mutation_allowed: false,
        official_joined_dataset_write_allowed: false,
        training_write_allowed: false,
        reward_write_allowed: false,
        shadow_write_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = claim_fingerprint(&claim)?;
    claim.attempt_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim)?;
    Ok(claim)
}

async fn execute_fixed_projection(
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
    bindings: &ProjectionBindings,
    raw_entries: &[RawOutcomeEntryInput],
    manifest: &HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
    bundle: &HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    join_spec: &HistoricalOutcomeFeatureLabelJoinSpecification,
    target_spec: &HistoricalOutcomeSemanticTargetSpecification,
) -> Result<
    (
        HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope,
        String,
        usize,
    ),
    JoinTargetFailure,
> {
    let envelope = project_untrusted_candidate(
        bindings,
        raw_entries,
        manifest,
        bundle,
        join_spec,
        target_spec,
    )
    .map_err(JoinTargetFailure::before_staging)?;
    let bytes = serde_json::to_vec(&envelope)
        .map_err(|error| JoinTargetFailure::before_staging(error.to_string()))?;
    if bytes.len() > authorization.runner.runner_contract.maximum_output_bytes as usize {
        return Err(JoinTargetFailure::before_staging(
            "未验证 join/target 候选超过 runner 登记输出上限",
        ));
    }
    let directory = std::env::temp_dir().join(format!("hone-join-target-{}", Uuid::new_v4()));
    tokio::fs::create_dir(&directory)
        .await
        .map_err(|error| JoinTargetFailure {
            message: format!("一次性 join/target 目录创建失败：{error}"),
            ephemeral_directory_removed: false,
        })?;
    let path = directory.join("untrusted-joined-target-candidate.json");
    let staged = async {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
            .map_err(|error| error.to_string())?;
        file.write_all(&bytes)
            .await
            .map_err(|error| error.to_string())?;
        file.sync_all().await.map_err(|error| error.to_string())?;
        let read_back = tokio::fs::read(&path)
            .await
            .map_err(|error| error.to_string())?;
        if read_back != bytes {
            return Err("一次性 join/target 输出回读不一致".to_string());
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
        (Ok(_), Err(error), removed) => Err(JoinTargetFailure {
            message: format!("一次性 join/target 目录清理失败：{error}"),
            ephemeral_directory_removed: removed,
        }),
        (Err(error), _, removed) => Err(JoinTargetFailure {
            message: error,
            ephemeral_directory_removed: removed,
        }),
        (Ok(_), Ok(()), false) => Err(JoinTargetFailure {
            message: "一次性 join/target 目录清理状态无法确认".to_string(),
            ephemeral_directory_removed: false,
        }),
    }
}

fn project_untrusted_candidate(
    bindings: &ProjectionBindings,
    raw_entries: &[RawOutcomeEntryInput],
    manifest: &HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
    bundle: &HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    join_spec: &HistoricalOutcomeFeatureLabelJoinSpecification,
    target_spec: &HistoricalOutcomeSemanticTargetSpecification,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope, String> {
    validate_projection_contract(bindings, manifest, bundle, join_spec, target_spec)?;
    if raw_entries.is_empty() || raw_entries.len() != manifest.records.len() {
        return Err("原始结果条目与 official split manifest 不是完整一对一关系".to_string());
    }
    let raw_index = unique_index(
        raw_entries,
        |entry| entry.dataset_entry_id.as_str(),
        "原始结果",
    )?;
    let split_index = unique_index(
        &manifest.records,
        |record| record.dataset_entry_id.as_str(),
        "official split",
    )?;
    if raw_index.keys().copied().collect::<BTreeSet<_>>()
        != split_index.keys().copied().collect::<BTreeSet<_>>()
    {
        return Err("原始结果与 official split entry key 集合不一致".to_string());
    }
    let mut features_by_entry: BTreeMap<&str, Vec<_>> = BTreeMap::new();
    let mut feature_keys = BTreeSet::new();
    for feature in &bundle.records {
        if !feature_keys.insert((
            feature.dataset_entry_id.as_str(),
            feature.feature_namespace.as_str(),
            feature.feature_id.as_str(),
        )) {
            return Err("official feature bundle 存在重复 entry/namespace/feature key".to_string());
        }
        features_by_entry
            .entry(feature.dataset_entry_id.as_str())
            .or_default()
            .push(feature);
    }
    let mut canonical_feature_ids: Option<BTreeSet<(&str, &str)>> = None;
    let mut rows = Vec::new();
    let mut excluded_rows = Vec::new();
    for raw in raw_entries {
        let split = split_index
            .get(raw.dataset_entry_id.as_str())
            .ok_or_else(|| "原始结果缺少 official split record".to_string())?;
        if split.dataset_entry_sha256 != raw.dataset_entry_sha256
            || split.decision_available_at != raw.decision_available_at
        {
            return Err(format!("条目 {} 的 split 绑定漂移", raw.dataset_entry_id));
        }
        let features = features_by_entry
            .remove(raw.dataset_entry_id.as_str())
            .unwrap_or_default();
        if split.purged_or_embargoed {
            if !features.is_empty() {
                return Err(format!(
                    "purge/embargo 条目 {} 不得含 feature record",
                    raw.dataset_entry_id
                ));
            }
            excluded_rows.push(HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit {
                dataset_entry_id: raw.dataset_entry_id.clone(),
                dataset_entry_sha256: raw.dataset_entry_sha256.clone(),
                split: split.split.clone(),
                component_id: split.component_id.clone(),
                purge_reason: split
                    .purge_reason
                    .clone()
                    .unwrap_or_else(|| "purged_or_embargoed".to_string()),
                feature_record_count: 0,
                target_values_opened: false,
            });
            continue;
        }
        if features.len() != EXPECTED_FEATURE_COUNT {
            return Err(format!(
                "条目 {} 不是恰好 65 个 feature record",
                raw.dataset_entry_id
            ));
        }
        if features.iter().any(|feature| {
            feature.available_at_utc > raw.decision_available_at
                || (feature.is_missing
                    && (feature.value.is_some() || feature.missingness_reason.trim().is_empty()))
                || (!feature.is_missing && feature.value.is_none())
        }) {
            return Err(format!(
                "条目 {} 含未来特征或缺失语义漂移",
                raw.dataset_entry_id
            ));
        }
        let feature_ids = features
            .iter()
            .map(|feature| {
                (
                    feature.feature_namespace.as_str(),
                    feature.feature_id.as_str(),
                )
            })
            .collect::<BTreeSet<_>>();
        if feature_ids.len() != EXPECTED_FEATURE_COUNT {
            return Err(format!(
                "条目 {} 的 feature 目录键重复",
                raw.dataset_entry_id
            ));
        }
        if let Some(expected) = &canonical_feature_ids {
            if expected != &feature_ids {
                return Err("不同条目的 65 项 feature 目录不一致".to_string());
            }
        } else {
            canonical_feature_ids = Some(feature_ids);
        }
        let target_values = target_values(raw, target_spec)?;
        let target_commitment_sha256 = hash_serializable(&target_values)?;
        let (target_visibility, target_vector) = match split.split {
            HistoricalOutcomeOfflineDatasetTransformationSplit::Train => (
                HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets,
                Some(target_values),
            ),
            HistoricalOutcomeOfflineDatasetTransformationSplit::Validation => (
                HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld,
                None,
            ),
            HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout => (
                HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld,
                None,
            ),
        };
        let feature_records = features.into_iter().cloned().collect::<Vec<_>>();
        let source_binding_sha256 = hash_serializable(&serde_json::json!({
            "dataset_entry_id": raw.dataset_entry_id,
            "dataset_entry_sha256": raw.dataset_entry_sha256,
            "split_record": split,
            "feature_records": feature_records,
            "target_commitment_sha256": target_commitment_sha256,
        }))?;
        rows.push(HistoricalOutcomeFeatureLabelJoinTargetCandidateRow {
            dataset_entry_id: raw.dataset_entry_id.clone(),
            dataset_entry_sha256: raw.dataset_entry_sha256.clone(),
            split: split.split.clone(),
            component_id: split.component_id.clone(),
            decision_available_at: raw.decision_available_at,
            feature_records,
            target_visibility,
            target_vector,
            target_commitment_sha256,
            source_binding_sha256,
        });
    }
    if !features_by_entry.is_empty() {
        return Err("official feature bundle 含不属于当前数据集或已 purge 条目的记录".to_string());
    }
    let train_target_vector_count = rows
        .iter()
        .filter(|row| row.target_vector.is_some())
        .count();
    let validation_target_withheld_count = rows
        .iter()
        .filter(|row| {
            row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
        })
        .count();
    let sealed_holdout_target_withheld_count = rows
        .iter()
        .filter(|row| {
            row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
        })
        .count();
    let mut envelope = HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_string(),
        authorization_review_id: bindings.authorization_review_id.clone(),
        authorization_review_sha256: bindings.authorization_review_sha256.clone(),
        runner_id: bindings.runner_id.clone(),
        runner_spec_sha256: bindings.runner_spec_sha256.clone(),
        implementation_id: bindings.implementation_id.clone(),
        implementation_sha256: bindings.implementation_sha256.clone(),
        implementation_review_sha256: bindings.implementation_review_sha256.clone(),
        specification_id: bindings.specification_id.clone(),
        specification_sha256: bindings.specification_sha256.clone(),
        specification_body_sha256: bindings.specification_body_sha256.clone(),
        join_specification_sha256: bindings.join_specification_sha256.clone(),
        target_specification_sha256: bindings.target_specification_sha256.clone(),
        validation_id: bindings.validation_id.clone(),
        validation_sha256: bindings.validation_sha256.clone(),
        split_manifest_sha256: bindings.split_manifest_sha256.clone(),
        feature_bundle_sha256: bindings.feature_bundle_sha256.clone(),
        combined_artifact_sha256: bindings.combined_artifact_sha256.clone(),
        dataset_id: bindings.dataset_id.clone(),
        dataset_content_sha256: bindings.dataset_content_sha256.clone(),
        dataset_manifest_sha256: bindings.dataset_manifest_sha256.clone(),
        candidate_set_sha256: bindings.candidate_set_sha256.clone(),
        dataset_entry_count: raw_entries.len(),
        active_candidate_row_count: rows.len(),
        excluded_purge_or_embargo_row_count: excluded_rows.len(),
        feature_catalog_count: EXPECTED_FEATURE_COUNT,
        target_count: EXPECTED_TARGET_COUNT,
        train_target_vector_count,
        validation_target_withheld_count,
        sealed_holdout_target_withheld_count,
        rows,
        excluded_rows,
        one_to_one_join_satisfied: true,
        exact_raw_f64_bits_preserved: true,
        official_split_authority_preserved: true,
        point_in_time_and_explicit_missingness_preserved: true,
        validation_targets_withheld: true,
        sealed_holdout_targets_withheld: true,
        output_is_untrusted: true,
        independent_output_validation_completed: false,
        official_joined_dataset_created: false,
        copied_to_training_store: false,
        training_started: false,
        reward_written: false,
        shadow_position_written: false,
        order_generated: false,
        broker_accessed: false,
        trade_executed: false,
    };
    envelope
        .rows
        .sort_by(|left, right| left.dataset_entry_id.cmp(&right.dataset_entry_id));
    envelope
        .excluded_rows
        .sort_by(|left, right| left.dataset_entry_id.cmp(&right.dataset_entry_id));
    validate_envelope(&envelope)?;
    Ok(envelope)
}

fn validate_projection_contract(
    bindings: &ProjectionBindings,
    manifest: &HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
    bundle: &HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    join_spec: &HistoricalOutcomeFeatureLabelJoinSpecification,
    target_spec: &HistoricalOutcomeSemanticTargetSpecification,
) -> Result<(), String> {
    let target_ids = target_spec
        .target_definitions
        .iter()
        .map(|definition| definition.target_id.as_str())
        .collect::<BTreeSet<_>>();
    let target_order = target_spec
        .target_vector_order
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let target_contract = target_spec.target_definitions.len() == EXPECTED_TARGET_COUNT
        && target_spec.target_vector_order.len() == EXPECTED_TARGET_COUNT
        && target_ids.len() == EXPECTED_TARGET_COUNT
        && target_ids == target_order
        && target_spec
            .target_definitions
            .iter()
            .all(target_definition_allowed)
        && target_spec.exact_f64_bits_preserved
        && !target_spec.normalization_allowed
        && !target_spec.winsorization_allowed
        && !target_spec.rank_transform_allowed
        && !target_spec.categorical_action_label_defined
        && !target_spec.buy_hold_sell_threshold_defined
        && !target_spec.portfolio_weight_target_defined
        && !target_spec.scalar_reward_defined
        && !target_spec.semantic_target_assignment_performed;
    let join_contract = join_spec.one_to_one_outcome_join_required
        && join_spec.all_allowlisted_feature_records_preserved
        && join_spec.feature_catalog_count == EXPECTED_FEATURE_COUNT
        && join_spec.allowed_label_horizons_market_sessions == REQUIRED_HORIZONS
        && !join_spec.imputation_allowed
        && !join_spec.interpolation_allowed
        && !join_spec.sealed_holdout_labels_opened
        && !join_spec.join_executed;
    let exact_bindings = manifest.manifest_sha256 == bindings.split_manifest_sha256
        && bundle.feature_bundle_sha256 == bindings.feature_bundle_sha256
        && join_spec.specification_sha256 == bindings.join_specification_sha256
        && target_spec.specification_sha256 == bindings.target_specification_sha256
        && join_spec.dataset_id == bindings.dataset_id
        && join_spec.dataset_content_sha256 == bindings.dataset_content_sha256
        && join_spec.dataset_manifest_sha256 == bindings.dataset_manifest_sha256
        && join_spec.candidate_set_sha256 == bindings.candidate_set_sha256
        && join_spec.split_manifest_sha256 == bindings.split_manifest_sha256
        && join_spec.feature_bundle_sha256 == bindings.feature_bundle_sha256
        && join_spec.combined_artifact_sha256 == bindings.combined_artifact_sha256
        && bundle.feature_catalog_count == EXPECTED_FEATURE_COUNT
        && bundle.feature_catalog_sha256 == join_spec.feature_catalog_sha256
        && bundle.feature_schema_sha256 == join_spec.feature_schema_sha256;
    if !target_contract || !join_contract || !exact_bindings {
        return Err("join/target 投影合同、九项目标或正式工件绑定漂移".to_string());
    }
    Ok(())
}

fn target_definition_allowed(definition: &HistoricalOutcomeSemanticTargetDefinition) -> bool {
    REQUIRED_HORIZONS.contains(&definition.horizon_market_sessions)
        && matches!(
            definition.source_metric_field.as_str(),
            "asset_return" | "excess_return" | "asset_max_drawdown"
        )
        && definition.value_kind == "f64_exact_bits"
        && definition.unit == "decimal_return"
        && definition.transformation == "identity_no_normalization_no_winsorization_no_ranking"
        && definition.target_id
            == format!(
                "{}_{}_continuous",
                definition.source_metric_field, definition.horizon_market_sessions
            )
}

fn target_values(
    raw: &RawOutcomeEntryInput,
    target_spec: &HistoricalOutcomeSemanticTargetSpecification,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetValue>, String> {
    let definitions = target_spec
        .target_definitions
        .iter()
        .map(|definition| (definition.target_id.as_str(), definition))
        .collect::<HashMap<_, _>>();
    target_spec
        .target_vector_order
        .iter()
        .map(|target_id| {
            let definition = definitions
                .get(target_id.as_str())
                .ok_or_else(|| format!("目标 {target_id} 缺少冻结定义"))?;
            let metric = raw
                .metrics
                .get(&definition.horizon_market_sessions)
                .ok_or_else(|| format!("条目 {} 缺少目标窗口", raw.dataset_entry_id))?;
            let bits = match definition.source_metric_field.as_str() {
                "asset_return" => metric.asset_return,
                "excess_return" => metric.excess_return,
                "asset_max_drawdown" => metric.asset_max_drawdown,
                _ => return Err(format!("目标 {target_id} 使用未允许原始字段")),
            };
            Ok(HistoricalOutcomeFeatureLabelJoinTargetValue {
                target_id: definition.target_id.clone(),
                horizon_market_sessions: definition.horizon_market_sessions,
                source_metric_field: definition.source_metric_field.clone(),
                value_kind: definition.value_kind.clone(),
                unit: definition.unit.clone(),
                role: definition.role.clone(),
                exact_f64_bits_hex: format!("{bits:016x}"),
            })
        })
        .collect()
}

fn unique_index<'a, T, F>(
    values: &'a [T],
    key: F,
    label: &str,
) -> Result<BTreeMap<&'a str, &'a T>, String>
where
    F: Fn(&'a T) -> &'a str,
{
    let mut index = BTreeMap::new();
    for value in values {
        if index.insert(key(value), value).is_some() {
            return Err(format!("{label}存在重复 entry key"));
        }
    }
    Ok(index)
}

fn validate_envelope(
    envelope: &HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope,
) -> Result<(), String> {
    let row_ids = envelope
        .rows
        .iter()
        .map(|row| row.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    let excluded_ids = envelope
        .excluded_rows
        .iter()
        .map(|row| row.dataset_entry_id.as_str())
        .collect::<BTreeSet<_>>();
    let rows_safe = envelope.rows.iter().all(|row| {
        row.feature_records.len() == EXPECTED_FEATURE_COUNT
            && valid_sha256(&row.target_commitment_sha256)
            && valid_sha256(&row.source_binding_sha256)
            && row.feature_records.iter().all(|feature| {
                feature.dataset_entry_id == row.dataset_entry_id
                    && feature.available_at_utc <= row.decision_available_at
            })
            && match row.target_visibility {
                HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets => row
                    .target_vector
                    .as_ref()
                    .is_some_and(|values| values.len() == EXPECTED_TARGET_COUNT),
                HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
                | HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld => {
                    row.target_vector.is_none()
                }
            }
    });
    let excluded_safe = envelope
        .excluded_rows
        .iter()
        .all(|row| row.feature_record_count == 0 && !row.target_values_opened);
    let authority_closed = envelope.output_is_untrusted
        && !envelope.independent_output_validation_completed
        && !envelope.official_joined_dataset_created
        && !envelope.copied_to_training_store
        && !envelope.training_started
        && !envelope.reward_written
        && !envelope.shadow_position_written
        && !envelope.order_generated
        && !envelope.broker_accessed
        && !envelope.trade_executed;
    let counts = envelope.active_candidate_row_count == envelope.rows.len()
        && envelope.excluded_purge_or_embargo_row_count == envelope.excluded_rows.len()
        && envelope.dataset_entry_count
            == envelope.active_candidate_row_count
                + envelope.excluded_purge_or_embargo_row_count
        && envelope.train_target_vector_count
            == envelope
                .rows
                .iter()
                .filter(|row| row.target_vector.is_some())
                .count()
        && envelope.validation_target_withheld_count
            == envelope
                .rows
                .iter()
                .filter(|row| {
                    row.target_visibility
                        == HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
                })
                .count()
        && envelope.sealed_holdout_target_withheld_count
            == envelope
                .rows
                .iter()
                .filter(|row| {
                    row.target_visibility
                        == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
                })
                .count();
    let hashes = [
        &envelope.authorization_review_sha256,
        &envelope.runner_spec_sha256,
        &envelope.implementation_sha256,
        &envelope.implementation_review_sha256,
        &envelope.specification_sha256,
        &envelope.specification_body_sha256,
        &envelope.join_specification_sha256,
        &envelope.target_specification_sha256,
        &envelope.validation_sha256,
        &envelope.split_manifest_sha256,
        &envelope.feature_bundle_sha256,
        &envelope.combined_artifact_sha256,
        &envelope.dataset_content_sha256,
        &envelope.dataset_manifest_sha256,
        &envelope.candidate_set_sha256,
    ];
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION
        || envelope.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || envelope.target_count != EXPECTED_TARGET_COUNT
        || row_ids.len() != envelope.rows.len()
        || excluded_ids.len() != envelope.excluded_rows.len()
        || !row_ids.is_disjoint(&excluded_ids)
        || hashes.iter().any(|hash| !valid_sha256(hash))
        || !rows_safe
        || !excluded_safe
        || !counts
        || !envelope.one_to_one_join_satisfied
        || !envelope.exact_raw_f64_bits_preserved
        || !envelope.official_split_authority_preserved
        || !envelope.point_in_time_and_explicit_missingness_preserved
        || !envelope.validation_targets_withheld
        || !envelope.sealed_holdout_targets_withheld
        || !authority_closed
    {
        return Err("未验证 join/target 候选结构无效或越权".to_string());
    }
    Ok(())
}

fn new_success_result(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    envelope: HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope,
    output_sha256: String,
    stdout_bytes: usize,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult, String> {
    let mut result = HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus::CompletedWithUntrustedJoinedTargetCandidateEnvelope,
        exit_code: 0,
        stdout_sha256: output_sha256.clone(),
        stderr_sha256: hash_bytes(&[]),
        stdout_bytes,
        stderr_bytes: 0,
        bounded_error: None,
        output_sha256: Some(output_sha256),
        untrusted_candidate_envelope: Some(envelope),
        ephemeral_directory_removed: true,
        independent_output_validation_completed: false,
        official_joined_dataset_authorized: false,
        training_data_access_authorized: false,
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
    claim: &HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim,
    completed_at: DateTime<Utc>,
    duration_millis: u64,
    error: &JoinTargetFailure,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult, String> {
    let bounded = error
        .message
        .chars()
        .take(MAX_ERROR_CHARS)
        .collect::<String>();
    let mut result = HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        execution_policy_version: EXECUTION_POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        attempt_id: claim.attempt_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at,
        duration_millis,
        status: HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus::FailedAuthorizationConsumed,
        exit_code: 1,
        stdout_sha256: hash_bytes(&[]),
        stderr_sha256: hash_bytes(bounded.as_bytes()),
        stdout_bytes: 0,
        stderr_bytes: bounded.len(),
        bounded_error: Some(bounded),
        output_sha256: None,
        untrusted_candidate_envelope: None,
        ephemeral_directory_removed: error.ephemeral_directory_removed,
        independent_output_validation_completed: false,
        official_joined_dataset_authorized: false,
        training_data_access_authorized: false,
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
    claim: &HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim,
) -> Result<(), String> {
    let closed = !claim.generic_label_store_read_allowed
        && !claim.training_store_read_allowed
        && !claim.child_process_spawned
        && !claim.ambient_filesystem_capability_available
        && !claim.ambient_environment_capability_available
        && !claim.network_capability_available
        && !claim.external_tool_capability_available
        && !claim.production_data_capability_available
        && !claim.historical_state_mutation_allowed
        && !claim.official_joined_dataset_write_allowed
        && !claim.training_write_allowed
        && !claim.reward_write_allowed
        && !claim.shadow_write_allowed
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
        &claim.specification_review_sha256,
        &claim.specification_sha256,
        &claim.specification_body_sha256,
        &claim.join_specification_sha256,
        &claim.target_specification_sha256,
        &claim.validation_sha256,
        &claim.split_manifest_sha256,
        &claim.feature_bundle_sha256,
        &claim.combined_artifact_sha256,
        &claim.dataset_content_sha256,
        &claim.dataset_manifest_sha256,
        &claim.candidate_set_sha256,
    ];
    let ids = [
        &claim.attempt_id,
        &claim.authorization_review_id,
        &claim.isolated_runner_id,
        &claim.implementation_id,
        &claim.implementation_review_id,
        &claim.specification_review_id,
        &claim.specification_id,
        &claim.validation_id,
        &claim.dataset_id,
    ];
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.execution_policy_version != EXECUTION_POLICY_VERSION
        || ids.iter().any(|id| !valid_id(id))
        || hashes.iter().any(|hash| !valid_sha256(hash))
        || claim.attempt_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != claim_fingerprint(claim)?
        || claim.claimed_at >= claim.authorization_valid_until
        || claim.invoked_by.trim().is_empty()
        || claim.isolation_backend != ISOLATION_BACKEND
        || claim.max_wall_clock_seconds != 300
        || claim.max_memory_mib != 4_096
        || claim.max_cpu_millicores != 1_000
        || claim.max_process_count != 1
        || claim.max_output_bytes != 8_388_608
        || !claim.artifact_digest_reverified
        || !claim.exact_current_complete_upstream_chain_revalidated
        || !claim.exact_official_artifact_pair_and_dataset_revalidated
        || !claim.authorization_consumed
        || !claim.invocation_started
        || !claim.exact_bound_raw_outcome_read_allowed
        || !closed
    {
        return Err("join/target 执行 claim 无效或越权".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult,
    claim: &HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim,
) -> Result<(), String> {
    let downstream_closed = !result.independent_output_validation_completed
        && !result.official_joined_dataset_authorized
        && !result.training_data_access_authorized
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
            HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus::CompletedWithUntrustedJoinedTargetCandidateEnvelope,
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
            HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus::FailedAuthorizationConsumed,
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
        return Err("join/target 执行结果无效或越权".to_string());
    }
    Ok(())
}

fn claim_binding_matches(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim,
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
) -> bool {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let implementation_review = &runner.implementation_review;
    let specification_review = &implementation.approved_review;
    let specification = &specification_review.specification;
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
        && claim.specification_review_id == specification_review.review_id
        && claim.specification_review_sha256 == specification_review.review_sha256
        && claim.specification_id == specification.specification_id
        && claim.specification_sha256 == specification.specification_sha256
        && claim.specification_body_sha256 == specification.specification_body_sha256
        && claim.join_specification_sha256 == specification.join_specification.specification_sha256
        && claim.target_specification_sha256
            == specification.target_specification.specification_sha256
        && claim.validation_id == specification.validation_id
        && claim.validation_sha256 == specification.validation_sha256
        && claim.split_manifest_sha256 == specification.split_manifest_sha256
        && claim.feature_bundle_sha256 == specification.feature_bundle_sha256
        && claim.combined_artifact_sha256 == specification.combined_artifact_sha256
        && claim.dataset_id == specification.dataset_id
        && claim.dataset_content_sha256 == specification.dataset_content_sha256
        && claim.dataset_manifest_sha256 == specification.dataset_manifest_sha256
        && claim.candidate_set_sha256 == specification.candidate_set_sha256
}

fn claim_fingerprint(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim,
) -> Result<String, String> {
    fingerprint_without(claim, &["attempt_id", "claim_sha256"])
}

fn result_fingerprint(
    result: &HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult,
) -> Result<String, String> {
    fingerprint_without(result, &["result_id", "result_sha256"])
}

async fn read_attempt_records(
    state: &AppState,
) -> Result<
    (
        Vec<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim>,
        Vec<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult>,
    ),
    String,
> {
    let claims = read_nested_json::<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptClaim>(
        &execution_root(state).join("claims"),
    )
    .await?;
    let results =
        read_nested_json::<HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptResult>(
            &execution_root(state).join("results"),
        )
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
            return Err("join/target 执行登记含重放 claim".to_string());
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
            .ok_or_else(|| "join/target 结果缺少不可变 claim".to_string())?;
        validate_result(result, claim)?;
        if !result_ids.insert(result.result_id.clone())
            || !result_hashes.insert(result.result_sha256.clone())
            || !completed.insert(result.attempt_id.clone())
        {
            return Err("join/target 执行登记含重复结果".to_string());
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
    decision_root(state).join("historical-outcome-feature-label-join-target-execution-attempts")
}

fn claim_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("claims").join(runner_id)
}

fn result_directory(state: &AppState, runner_id: &str) -> PathBuf {
    execution_root(state).join("results").join(runner_id)
}

struct JoinTargetExecutionLock {
    path: PathBuf,
}

impl Drop for JoinTargetExecutionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    runner_id: &str,
) -> Result<JoinTargetExecutionLock, String> {
    let directory = execution_root(state).join("locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{runner_id}.lock"));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(JoinTargetExecutionLock { path }),
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
                return Err("另一个管理员正在领取这条 join/target 一次性授权".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("join/target 一次性执行锁不可用".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "immutable json path has no parent".to_string())?;
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
    file.sync_all().await.map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_first_execution_authorizations::tests::approved_authorization;
    use crate::routes::historical_outcome_offline_dataset_transformation_execution_attempts::{
        HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit,
        HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord,
    };

    struct ProjectionFixture {
        authorization: ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
        bindings: ProjectionBindings,
        raw_entries: Vec<RawOutcomeEntryInput>,
        manifest: HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
        bundle: HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
        join_spec: HistoricalOutcomeFeatureLabelJoinSpecification,
        target_spec: HistoricalOutcomeSemanticTargetSpecification,
    }

    fn h(character: char) -> String {
        character.to_string().repeat(64)
    }

    fn raw_entry(id: &str, entry_sha256: &str, decision_at: DateTime<Utc>) -> RawOutcomeEntryInput {
        let metrics = REQUIRED_HORIZONS
            .into_iter()
            .enumerate()
            .map(|(index, horizon)| {
                let base = (index + 1) as f64 / 10.0;
                (
                    horizon,
                    RawMetricBits {
                        asset_return: base.to_bits(),
                        excess_return: (base - 0.03).to_bits(),
                        asset_max_drawdown: (-base / 2.0).to_bits(),
                    },
                )
            })
            .collect();
        RawOutcomeEntryInput {
            dataset_entry_id: id.to_string(),
            dataset_entry_sha256: entry_sha256.to_string(),
            decision_available_at: decision_at,
            metrics,
        }
    }

    fn split_record(
        raw: &RawOutcomeEntryInput,
        split: HistoricalOutcomeOfflineDatasetTransformationSplit,
        index: usize,
    ) -> HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord {
        HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord {
            dataset_entry_id: raw.dataset_entry_id.clone(),
            dataset_entry_sha256: raw.dataset_entry_sha256.clone(),
            component_id: format!("component-{index}"),
            component_order_index: index,
            split,
            boundary_objective_tuple: [1, 2, 3, 4],
            purged_or_embargoed: false,
            purge_reason: None,
            price_snapshot_sha256: h('9'),
            common_market_session_index: index,
            decision_session_date: NaiveDate::from_ymd_opt(2025, 1, 2 + index as u32).unwrap(),
            outcome_horizon_250_end_date: NaiveDate::from_ymd_opt(2026, 1, 2 + index as u32)
                .unwrap(),
            decision_available_at: raw.decision_available_at,
        }
    }

    fn feature_records(
        raw: &RawOutcomeEntryInput,
    ) -> Vec<HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord> {
        (0..EXPECTED_FEATURE_COUNT)
            .map(
                |index| HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord {
                    dataset_entry_id: raw.dataset_entry_id.clone(),
                    feature_id: format!("feature_{index:02}"),
                    feature_namespace: "point_in_time_fundamentals".to_string(),
                    value_encoding: "explicit_missingness".to_string(),
                    value: None,
                    value_kind: "f64".to_string(),
                    unit: "ratio".to_string(),
                    period_start_utc: None,
                    period_end_utc: None,
                    accounting_basis: "not_available".to_string(),
                    measurement_scope: "issuer".to_string(),
                    is_missing: true,
                    missingness_reason: "source_not_available_at_decision_time".to_string(),
                    artifact_sha256: h('8'),
                    source_identity: "sealed-test-source".to_string(),
                    source_version: "v1".to_string(),
                    observed_at_utc: raw.decision_available_at,
                    available_at_utc: raw.decision_available_at,
                    transformation_id: "fixed_projection_v1".to_string(),
                    transformation_sha256: h('7'),
                },
            )
            .collect()
    }

    fn fixture() -> ProjectionFixture {
        let authorization = approved_authorization();
        let specification = &authorization
            .runner
            .implementation
            .approved_review
            .specification;
        let bindings = projection_bindings(&authorization);
        let decision_at = DateTime::from_timestamp(1_750_000_000, 0).unwrap();
        let raw_entries = vec![
            raw_entry("train-entry", &h('1'), decision_at),
            raw_entry("validation-entry", &h('2'), decision_at),
            raw_entry("sealed-entry", &h('3'), decision_at),
        ];
        let records = vec![
            split_record(
                &raw_entries[0],
                HistoricalOutcomeOfflineDatasetTransformationSplit::Train,
                0,
            ),
            split_record(
                &raw_entries[1],
                HistoricalOutcomeOfflineDatasetTransformationSplit::Validation,
                1,
            ),
            split_record(
                &raw_entries[2],
                HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout,
                2,
            ),
        ];
        let manifest = HistoricalOutcomeOfflineDatasetOfficialSplitManifest {
            schema_version: "official-split-test-v1".to_string(),
            manifest_sha256: bindings.split_manifest_sha256.clone(),
            materialization_id: "materialization-test".to_string(),
            transformation_attempt_id: "transformation-attempt-test".to_string(),
            admission_review_id: "admission-review-test".to_string(),
            admission_review_sha256: h('4'),
            validation_id: bindings.validation_id.clone(),
            validation_sha256: bindings.validation_sha256.clone(),
            source_output_sha256: h('5'),
            dataset_id: bindings.dataset_id.clone(),
            dataset_content_sha256: bindings.dataset_content_sha256.clone(),
            dataset_manifest_sha256: bindings.dataset_manifest_sha256.clone(),
            candidate_set_sha256: bindings.candidate_set_sha256.clone(),
            transformation_spec_sha256: h('6'),
            split_specification_sha256: h('7'),
            source_split_manifest_candidate_sha256: h('8'),
            boundary_audit: HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit {
                schema_version: "boundary-audit-test-v1".to_string(),
                candidate_pair_count: 3,
                selected_train_component_end_exclusive: 1,
                selected_validation_component_end_exclusive: 2,
                pre_purge_train_entry_count: 1,
                pre_purge_validation_entry_count: 1,
                pre_purge_sealed_holdout_entry_count: 1,
                objective_tuple: [1, 2, 3, 4],
                all_candidate_objectives_sha256: h('9'),
                audit_sha256: h('a'),
            },
            records,
            entry_count: 3,
            component_count: 3,
            sealed_holdout_labels_withheld: true,
            exact_validated_candidate_copy: true,
            independently_validated_after_materialization: true,
            eligible_for_feature_join: true,
            semantic_target_assigned: false,
            training_authorized: false,
        };
        let features = raw_entries
            .iter()
            .flat_map(feature_records)
            .collect::<Vec<_>>();
        let join_spec = specification.join_specification.clone();
        let target_spec = specification.target_specification.clone();
        let bundle = HistoricalOutcomeOfflineDatasetOfficialFeatureBundle {
            schema_version: "official-feature-bundle-test-v1".to_string(),
            feature_bundle_sha256: bindings.feature_bundle_sha256.clone(),
            materialization_id: "materialization-test".to_string(),
            transformation_attempt_id: "transformation-attempt-test".to_string(),
            admission_review_id: "admission-review-test".to_string(),
            admission_review_sha256: h('4'),
            validation_id: bindings.validation_id.clone(),
            validation_sha256: bindings.validation_sha256.clone(),
            source_output_sha256: h('5'),
            dataset_id: bindings.dataset_id.clone(),
            dataset_content_sha256: bindings.dataset_content_sha256.clone(),
            dataset_manifest_sha256: bindings.dataset_manifest_sha256.clone(),
            candidate_set_sha256: bindings.candidate_set_sha256.clone(),
            transformation_spec_sha256: h('6'),
            feature_specification_sha256: h('7'),
            source_feature_bundle_candidate_sha256: h('8'),
            source_exclusion_audit_sha256: h('9'),
            feature_catalog_count: EXPECTED_FEATURE_COUNT,
            feature_catalog_sha256: join_spec.feature_catalog_sha256.clone(),
            feature_schema_sha256: join_spec.feature_schema_sha256.clone(),
            records: features,
            exclusion_audit: Vec::new(),
            explicit_missingness_preserved: true,
            no_imputation_or_backfill_performed: true,
            outcome_future_and_current_portfolio_sources_excluded: true,
            exact_validated_candidate_copy: true,
            independently_validated_after_materialization: true,
            joined_to_outcome_labels: false,
            semantic_target_assigned: false,
            copied_to_training_store: false,
            training_authorized: false,
        };
        ProjectionFixture {
            authorization,
            bindings,
            raw_entries,
            manifest,
            bundle,
            join_spec,
            target_spec,
        }
    }

    fn project(
        fixture: &ProjectionFixture,
    ) -> Result<HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope, String> {
        project_untrusted_candidate(
            &fixture.bindings,
            &fixture.raw_entries,
            &fixture.manifest,
            &fixture.bundle,
            &fixture.join_spec,
            &fixture.target_spec,
        )
    }

    #[test]
    fn exact_join_exposes_only_train_raw_target_bits() {
        let fixture = fixture();
        let envelope = project(&fixture).unwrap();
        assert_eq!(envelope.rows.len(), 3);
        assert_eq!(envelope.train_target_vector_count, 1);
        assert_eq!(envelope.validation_target_withheld_count, 1);
        assert_eq!(envelope.sealed_holdout_target_withheld_count, 1);
        let train = envelope
            .rows
            .iter()
            .find(|row| row.split == HistoricalOutcomeOfflineDatasetTransformationSplit::Train)
            .unwrap();
        let targets = train.target_vector.as_ref().unwrap();
        assert_eq!(targets.len(), EXPECTED_TARGET_COUNT);
        assert_eq!(targets[0].exact_f64_bits_hex.len(), 16);
        assert!(envelope.rows.iter().filter(|row| row.split != HistoricalOutcomeOfflineDatasetTransformationSplit::Train).all(|row| row.target_vector.is_none()));
        assert!(!envelope.independent_output_validation_completed);
        assert!(!envelope.official_joined_dataset_created);
        assert!(!envelope.training_started);
        assert!(!envelope.trade_executed);
    }

    #[test]
    fn duplicate_or_missing_entry_keys_fail_closed() {
        let mut duplicate = fixture();
        duplicate.raw_entries[1].dataset_entry_id =
            duplicate.raw_entries[0].dataset_entry_id.clone();
        assert!(project(&duplicate).is_err());

        let mut missing = fixture();
        missing.manifest.records.pop();
        assert!(project(&missing).is_err());
    }

    #[test]
    fn missing_duplicate_or_future_feature_fails_closed() {
        let mut missing = fixture();
        missing.bundle.records.pop();
        assert!(project(&missing).is_err());

        let mut duplicate = fixture();
        duplicate.bundle.records[1].feature_id = duplicate.bundle.records[0].feature_id.clone();
        assert!(project(&duplicate).is_err());

        let mut future = fixture();
        future.bundle.records[0].available_at_utc =
            future.raw_entries[0].decision_available_at + chrono::TimeDelta::seconds(1);
        assert!(project(&future).is_err());
    }

    #[test]
    fn missingness_semantics_cannot_be_ambiguous() {
        let mut empty_reason = fixture();
        empty_reason.bundle.records[0].missingness_reason.clear();
        assert!(project(&empty_reason).is_err());

        let mut missing_without_value = fixture();
        missing_without_value.bundle.records[0].is_missing = false;
        assert!(project(&missing_without_value).is_err());
    }

    #[test]
    fn purge_rows_never_open_features_or_targets() {
        let mut purged = fixture();
        let record = &mut purged.manifest.records[2];
        record.purged_or_embargoed = true;
        record.purge_reason = Some("purge_overlap_250d".to_string());
        purged
            .bundle
            .records
            .retain(|feature| feature.dataset_entry_id != "sealed-entry");
        let envelope = project(&purged).unwrap();
        assert_eq!(envelope.rows.len(), 2);
        assert_eq!(envelope.excluded_rows.len(), 1);
        assert!(!envelope.excluded_rows[0].target_values_opened);
        assert_eq!(envelope.excluded_rows[0].feature_record_count, 0);

        let mut leaked = fixture();
        leaked.manifest.records[2].purged_or_embargoed = true;
        leaked.manifest.records[2].purge_reason = Some("purge_overlap_250d".to_string());
        assert!(project(&leaked).is_err());
    }

    #[test]
    fn target_contract_rejects_transforms_actions_and_rewards() {
        let mut normalized = fixture();
        normalized.target_spec.normalization_allowed = true;
        assert!(project(&normalized).is_err());

        let mut action = fixture();
        action.target_spec.buy_hold_sell_threshold_defined = true;
        assert!(project(&action).is_err());

        let mut reward = fixture();
        reward.target_spec.scalar_reward_defined = true;
        assert!(project(&reward).is_err());
    }

    #[test]
    fn create_once_claim_is_exactly_bound_and_has_no_downstream_authority() {
        let fixture = fixture();
        let claimed_at = fixture.authorization.review.submitted_at + chrono::TimeDelta::hours(1);
        let claim = new_claim(&fixture.authorization, "stage-42-admin", claimed_at).unwrap();
        assert!(claim.authorization_consumed);
        assert!(claim.exact_bound_raw_outcome_read_allowed);
        assert!(!claim.generic_label_store_read_allowed);
        assert!(!claim.training_store_read_allowed);
        assert!(!claim.training_write_allowed);
        assert!(!claim.order_generation_allowed);
        assert!(!claim.trading_allowed);
        assert!(claim_binding_matches(&claim, &fixture.authorization));

        let mut tampered = claim;
        tampered.training_write_allowed = true;
        tampered.claim_sha256 = claim_fingerprint(&tampered).unwrap();
        tampered.attempt_id = tampered.claim_sha256[..32].to_string();
        assert!(validate_claim(&tampered).is_err());
    }

    #[test]
    fn success_result_remains_untrusted_and_downstream_closed() {
        let fixture = fixture();
        let envelope = project(&fixture).unwrap();
        let claimed_at = fixture.authorization.review.submitted_at + chrono::TimeDelta::hours(1);
        let claim = new_claim(&fixture.authorization, "stage-42-admin", claimed_at).unwrap();
        let output_sha256 = hash_serializable(&envelope).unwrap();
        let stdout_bytes = serde_json::to_vec(&envelope).unwrap().len();
        let result = new_success_result(
            &claim,
            claimed_at + chrono::TimeDelta::seconds(1),
            1_000,
            envelope,
            output_sha256,
            stdout_bytes,
        )
        .unwrap();
        assert!(!result.independent_output_validation_completed);
        assert!(!result.official_joined_dataset_authorized);
        assert!(!result.training_data_access_authorized);
        assert!(!result.training_authorized);
        assert!(!result.reward_authorized);
        assert!(!result.trading_authorized);

        let mut tampered = result;
        tampered.training_authorized = true;
        tampered.result_sha256 = result_fingerprint(&tampered).unwrap();
        tampered.result_id = tampered.result_sha256[..32].to_string();
        assert!(validate_result(&tampered, &claim).is_err());
    }

    #[test]
    fn failure_consumes_claim_and_bounds_error() {
        let fixture = fixture();
        let claimed_at = fixture.authorization.review.submitted_at + chrono::TimeDelta::hours(1);
        let claim = new_claim(&fixture.authorization, "stage-42-admin", claimed_at).unwrap();
        let failure = JoinTargetFailure::before_staging("x".repeat(MAX_ERROR_CHARS + 50));
        let result = new_failed_result(
            &claim,
            claimed_at + chrono::TimeDelta::seconds(1),
            1_000,
            &failure,
        )
        .unwrap();
        assert_eq!(result.status, HistoricalOutcomeFeatureLabelJoinTargetExecutionAttemptStatus::FailedAuthorizationConsumed);
        assert_eq!(
            result.bounded_error.unwrap().chars().count(),
            MAX_ERROR_CHARS
        );
        assert!(claim.authorization_consumed);
    }
}
