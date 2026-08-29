//! Create-once materialization of one exact admitted offline-transformation
//! candidate into an official split manifest and feature bundle.
//!
//! The operation writes a claim before copying the already independently
//! validated candidate.  It does not recompute features, join them to outcome
//! labels, assign a target, train, reward, shadow, order, access a broker or
//! trade.  The two official artifacts remain unusable until a later,
//! independent output validation gate verifies their exact contents.

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

use super::historical_outcome_offline_dataset_transformation_candidate_admission_reviews::{
    AdmittedHistoricalOutcomeOfflineTransformationCandidate,
    admitted_transformation_candidates_for_official_artifact_materialization,
    exact_admitted_transformation_candidate_for_official_artifact_materialization,
};
use super::historical_outcome_offline_dataset_transformation_execution_attempts::{
    HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit,
    HistoricalOutcomeOfflineDatasetTransformationExclusionAudit,
    HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord,
    HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-transformation-official-artifact-materialization-registry-v1";
const CLAIM_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-transformation-official-artifact-materialization-claim-v1";
const RESULT_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-transformation-official-artifact-materialization-result-v1";
const SPLIT_MANIFEST_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-official-split-manifest-v1";
const FEATURE_BUNDLE_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-official-feature-bundle-v1";
const POLICY_VERSION: &str =
    "hone-offline-transformation-official-artifact-create-once-exact-copy-v1";
const MAX_TOTAL_ARTIFACT_BYTES: usize = 32 * 1024 * 1024;
const EXPECTED_FEATURE_COUNT: usize = 65;
const MAX_ERROR_CHARS: usize = 1_200;

#[derive(Debug, Deserialize)]
pub(crate) struct MaterializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest {
    expected_admission_review_id: String,
    expected_admission_review_sha256: String,
    expected_validation_sha256: String,
    expected_output_sha256: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_transformation_spec_sha256: String,
    expected_split_specification_sha256: String,
    expected_feature_specification_sha256: String,
    exact_copy_only_confirmed: bool,
    create_once_and_failure_consumes_confirmed: bool,
    no_join_target_training_or_trading_confirmed: bool,
    independent_output_validation_required_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetOfficialSplitManifest {
    pub schema_version: String,
    pub manifest_sha256: String,
    pub materialization_id: String,
    pub transformation_attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub source_output_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub transformation_spec_sha256: String,
    pub split_specification_sha256: String,
    pub source_split_manifest_candidate_sha256: String,
    pub boundary_audit: HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit,
    pub records: Vec<HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord>,
    pub entry_count: usize,
    pub component_count: usize,
    pub sealed_holdout_labels_withheld: bool,
    pub exact_validated_candidate_copy: bool,
    pub independently_validated_after_materialization: bool,
    pub eligible_for_feature_join: bool,
    pub semantic_target_assigned: bool,
    pub training_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetOfficialFeatureBundle {
    pub schema_version: String,
    pub feature_bundle_sha256: String,
    pub materialization_id: String,
    pub transformation_attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub source_output_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub transformation_spec_sha256: String,
    pub feature_specification_sha256: String,
    pub source_feature_bundle_candidate_sha256: String,
    pub source_exclusion_audit_sha256: String,
    pub feature_catalog_count: usize,
    pub feature_catalog_sha256: String,
    pub feature_schema_sha256: String,
    pub records: Vec<HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord>,
    pub exclusion_audit: Vec<HistoricalOutcomeOfflineDatasetTransformationExclusionAudit>,
    pub explicit_missingness_preserved: bool,
    pub no_imputation_or_backfill_performed: bool,
    pub outcome_future_and_current_portfolio_sources_excluded: bool,
    pub exact_validated_candidate_copy: bool,
    pub independently_validated_after_materialization: bool,
    pub joined_to_outcome_labels: bool,
    pub semantic_target_assigned: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim
{
    pub schema_version: String,
    pub policy_version: String,
    pub materialization_id: String,
    pub claim_sha256: String,
    pub transformation_attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub source_claim_sha256: String,
    pub source_result_sha256: String,
    pub source_output_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub transformation_spec_sha256: String,
    pub split_specification_sha256: String,
    pub feature_specification_sha256: String,
    pub expected_split_manifest_candidate_sha256: String,
    pub expected_feature_bundle_candidate_sha256: String,
    pub expected_exclusion_audit_sha256: String,
    pub materialized_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub claimed_at: DateTime<Utc>,
    pub exact_current_admission_chain_revalidated: bool,
    pub materializer_independent_from_complete_prior_chain: bool,
    pub exact_copy_only_confirmed: bool,
    pub create_once_and_failure_consumes_confirmed: bool,
    pub no_join_target_training_or_trading_confirmed: bool,
    pub independent_output_validation_required_confirmed: bool,
    pub claim_consumed: bool,
    pub official_artifact_materialization_started: bool,
    pub ambient_environment_available: bool,
    pub network_available: bool,
    pub external_tools_available: bool,
    pub production_state_write_available: bool,
    pub historical_state_mutation_allowed: bool,
    pub feature_join_allowed: bool,
    pub semantic_target_assignment_allowed: bool,
    pub training_allowed: bool,
    pub reward_allowed: bool,
    pub shadow_portfolio_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus {
    CompletedPendingIndependentValidation,
    FailedClaimConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult
{
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub materialization_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub status: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus,
    pub error: Option<String>,
    pub split_manifest_sha256: Option<String>,
    pub feature_bundle_sha256: Option<String>,
    pub combined_artifact_sha256: Option<String>,
    pub total_artifact_bytes: usize,
    pub official_split_manifest_created: bool,
    pub official_feature_bundle_created: bool,
    pub exact_validated_candidate_copy_completed: bool,
    pub independent_output_validation_completed: bool,
    pub official_artifacts_eligible_for_feature_join: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assigned: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationAttempt
{
    pub claim: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    pub result:
        Option<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult>,
    pub split_manifest: Option<HistoricalOutcomeOfflineDatasetOfficialSplitManifest>,
    pub feature_bundle: Option<HistoricalOutcomeOfflineDatasetOfficialFeatureBundle>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationItem {
    pub admitted_candidate: AdmittedHistoricalOutcomeOfflineTransformationCandidate,
    pub attempt:
        Option<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationAttempt>,
    pub materialization_eligible: bool,
    pub official_artifacts_created_pending_independent_validation: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry
{
    pub schema_version: String,
    pub policy_version: String,
    pub items:
        Vec<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationItem>,
    pub admitted_candidate_count: usize,
    pub materialization_eligible_candidate_count: usize,
    pub claimed_candidate_count: usize,
    pub completed_materialization_count: usize,
    pub failed_or_incomplete_materialization_count: usize,
    pub unvalidated_official_artifact_pair_count: usize,
    pub materialization_status: String,
    pub official_artifact_materialization_enabled: bool,
    pub independent_official_artifact_validation_enabled: bool,
    pub feature_join_enabled: bool,
    pub semantic_target_assignment_enabled: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationReadinessSummary
{
    pub admitted_candidate_count: usize,
    pub claimed_candidate_count: usize,
    pub completed_materialization_count: usize,
    pub failed_or_incomplete_materialization_count: usize,
    pub unvalidated_official_artifact_pair_count: usize,
    pub materialization_status: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_official_artifact_materializations(
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
            warn!(%error, "historical offline transformation official artifact materialization registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线转换正式工件物化暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_materialize_historical_outcome_offline_dataset_transformation_official_artifacts_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<
        MaterializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
    >,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match materialize_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_transformation_official_artifact_materialization_readiness_summary(
    state: &AppState,
) -> Result<
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationReadinessSummary,
    String,
> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationReadinessSummary {
        admitted_candidate_count: registry.admitted_candidate_count,
        claimed_candidate_count: registry.claimed_candidate_count,
        completed_materialization_count: registry.completed_materialization_count,
        failed_or_incomplete_materialization_count: registry.failed_or_incomplete_materialization_count,
        unvalidated_official_artifact_pair_count: registry.unvalidated_official_artifact_pair_count,
        materialization_status: registry.materialization_status,
    })
}

pub(crate) fn official_artifact_materialization_claim_exists(
    state: &AppState,
    attempt_id: &str,
) -> bool {
    materialization_directory(state, attempt_id)
        .join("claim.json")
        .exists()
}

async fn build_registry(
    state: &AppState,
) -> Result<
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry,
    String,
> {
    let admitted =
        admitted_transformation_candidates_for_official_artifact_materialization(state).await?;
    let mut items = Vec::with_capacity(admitted.len());
    for candidate in admitted {
        let attempt = read_attempt(state, &candidate).await?;
        let completed = attempt.as_ref().is_some_and(attempt_completed);
        items.push(
            HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationItem {
                materialization_eligible: attempt.is_none(),
                official_artifacts_created_pending_independent_validation: completed,
                admitted_candidate: candidate,
                attempt,
            },
        );
    }
    let admitted_candidate_count = items.len();
    let materialization_eligible_candidate_count = items
        .iter()
        .filter(|item| item.materialization_eligible)
        .count();
    let claimed_candidate_count = items.iter().filter(|item| item.attempt.is_some()).count();
    let completed_materialization_count = items
        .iter()
        .filter(|item| item.attempt.as_ref().is_some_and(attempt_completed))
        .count();
    let failed_or_incomplete_materialization_count = items
        .iter()
        .filter(|item| {
            item.attempt
                .as_ref()
                .is_some_and(|attempt| !attempt_completed(attempt))
        })
        .count();
    let materialization_status = if failed_or_incomplete_materialization_count > 0 {
        "official_artifact_materialization_failed_or_incomplete_fail_closed"
    } else if completed_materialization_count > 0 {
        "official_artifacts_created_waiting_independent_output_validation"
    } else if materialization_eligible_candidate_count > 0 {
        "ready_for_create_once_official_artifact_materialization"
    } else {
        "waiting_admitted_transformation_candidate"
    };
    Ok(HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        admitted_candidate_count,
        materialization_eligible_candidate_count,
        claimed_candidate_count,
        completed_materialization_count,
        failed_or_incomplete_materialization_count,
        unvalidated_official_artifact_pair_count: completed_materialization_count,
        materialization_status: materialization_status.to_string(),
        official_artifact_materialization_enabled: materialization_eligible_candidate_count > 0,
        independent_official_artifact_validation_enabled: false,
        feature_join_enabled: false,
        semantic_target_assignment_enabled: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 34 阶段只消费一条当前独立准入候选并先写 create-once claim，再把已验证候选精确复制为内容寻址的正式 split manifest 与 feature bundle。产物仍未经过物化后独立校验，不能 join、定义目标、训练、奖励、影子、下单、访问券商或交易。".to_string(),
    })
}

fn attempt_completed(
    attempt: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationAttempt,
) -> bool {
    attempt.result.as_ref().is_some_and(|result| {
        result.status
            == HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus::CompletedPendingIndependentValidation
            && result.official_split_manifest_created
            && result.official_feature_bundle_created
            && !result.independent_output_validation_completed
            && attempt.split_manifest.is_some()
            && attempt.feature_bundle.is_some()
    })
}

async fn materialize_once(
    state: &AppState,
    materialized_by: &str,
    attempt_id: &str,
    request: MaterializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
) -> Result<
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult,
    String,
> {
    if !valid_id(attempt_id) {
        return Err("离线转换执行尝试 ID 无效".to_string());
    }
    if !request.exact_copy_only_confirmed
        || !request.create_once_and_failure_consumes_confirmed
        || !request.no_join_target_training_or_trading_confirmed
        || !request.independent_output_validation_required_confirmed
    {
        return Err("必须确认精确复制、失败消费和独立校验四项边界".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let admitted = exact_admitted_transformation_candidate_for_official_artifact_materialization(
        state,
        attempt_id,
        &request.expected_admission_review_id,
        &request.expected_admission_review_sha256,
        &request.expected_validation_sha256,
        &request.expected_output_sha256,
        &request.expected_dataset_content_sha256,
        &request.expected_dataset_manifest_sha256,
        &request.expected_candidate_set_sha256,
        &request.expected_transformation_spec_sha256,
        &request.expected_split_specification_sha256,
        &request.expected_feature_specification_sha256,
    )
    .await?
    .ok_or_else(|| "当前没有这条精确绑定且已独立准入的离线转换候选".to_string())?;
    if read_attempt(state, &admitted).await?.is_some() {
        return Err("这条候选已经领取过一次性正式工件物化 claim，不能重放".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&admitted);
    if !actor_is_independent(materialized_by, &excluded_prior_actor_ids) {
        return Err("正式工件物化人不能是准入复核、输出校验、执行或完整上游角色".to_string());
    }
    let claim = new_claim(
        &admitted,
        materialized_by,
        excluded_prior_actor_ids,
        &request,
        Utc::now(),
    )?;
    let directory = materialization_directory(state, attempt_id);
    write_immutable_json(&directory.join("claim.json"), &claim).await?;

    let materialization = build_artifacts(&claim, &admitted);
    let result = match materialization {
        Ok((manifest, bundle)) => {
            validate_manifest(&manifest, &claim, &admitted)?;
            validate_feature_bundle(&bundle, &claim, &admitted)?;
            let manifest_bytes =
                serde_json::to_vec(&manifest).map_err(|error| error.to_string())?;
            let bundle_bytes = serde_json::to_vec(&bundle).map_err(|error| error.to_string())?;
            let total_artifact_bytes = manifest_bytes.len() + bundle_bytes.len();
            if total_artifact_bytes > MAX_TOTAL_ARTIFACT_BYTES {
                new_failed_result(&claim, "正式工件超过固定 32 MiB 上限", total_artifact_bytes)?
            } else {
                write_immutable_json(
                    &directory.join(format!("split-manifest-{}.json", manifest.manifest_sha256)),
                    &manifest,
                )
                .await?;
                write_immutable_json(
                    &directory.join(format!(
                        "feature-bundle-{}.json",
                        bundle.feature_bundle_sha256
                    )),
                    &bundle,
                )
                .await?;
                new_success_result(&claim, &manifest, &bundle, total_artifact_bytes)?
            }
        }
        Err(error) => new_failed_result(&claim, &error, 0)?,
    };
    write_immutable_json(&directory.join("result.json"), &result).await?;
    Ok(result)
}

fn new_claim(
    admitted: &AdmittedHistoricalOutcomeOfflineTransformationCandidate,
    materialized_by: &str,
    excluded_prior_actor_ids: Vec<String>,
    request: &MaterializeHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim, String>
{
    let review = &admitted.admission_review;
    let validation = &admitted.candidate.validation;
    let mut claim =
        HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim {
            schema_version: CLAIM_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            materialization_id: String::new(),
            claim_sha256: String::new(),
            transformation_attempt_id: review.transformation_attempt_id.clone(),
            admission_review_id: review.review_id.clone(),
            admission_review_sha256: review.review_sha256.clone(),
            validation_id: validation.validation_id.clone(),
            validation_sha256: validation.validation_sha256.clone(),
            source_claim_sha256: validation.claim_sha256.clone(),
            source_result_sha256: validation.result_sha256.clone(),
            source_output_sha256: validation.output_sha256.clone(),
            dataset_id: validation.dataset_id.clone(),
            dataset_content_sha256: validation.dataset_content_sha256.clone(),
            dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(),
            candidate_set_sha256: validation.candidate_set_sha256.clone(),
            transformation_spec_sha256: validation.transformation_spec_sha256.clone(),
            split_specification_sha256: validation.split_specification_sha256.clone(),
            feature_specification_sha256: validation.feature_specification_sha256.clone(),
            expected_split_manifest_candidate_sha256: validation
                .recomputed_split_manifest_candidate_sha256
                .clone(),
            expected_feature_bundle_candidate_sha256: validation
                .recomputed_feature_bundle_candidate_sha256
                .clone(),
            expected_exclusion_audit_sha256: validation.recomputed_exclusion_audit_sha256.clone(),
            materialized_by: materialized_by.to_string(),
            excluded_prior_actor_ids,
            claimed_at,
            exact_current_admission_chain_revalidated: true,
            materializer_independent_from_complete_prior_chain: true,
            exact_copy_only_confirmed: request.exact_copy_only_confirmed,
            create_once_and_failure_consumes_confirmed: request
                .create_once_and_failure_consumes_confirmed,
            no_join_target_training_or_trading_confirmed: request
                .no_join_target_training_or_trading_confirmed,
            independent_output_validation_required_confirmed: request
                .independent_output_validation_required_confirmed,
            claim_consumed: true,
            official_artifact_materialization_started: true,
            ambient_environment_available: false,
            network_available: false,
            external_tools_available: false,
            production_state_write_available: false,
            historical_state_mutation_allowed: false,
            feature_join_allowed: false,
            semantic_target_assignment_allowed: false,
            training_allowed: false,
            reward_allowed: false,
            shadow_portfolio_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
    claim.claim_sha256 = fingerprint_without(&claim, &["materialization_id", "claim_sha256"])?;
    claim.materialization_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim, admitted)?;
    Ok(claim)
}

fn build_artifacts(
    claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    admitted: &AdmittedHistoricalOutcomeOfflineTransformationCandidate,
) -> Result<
    (
        HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
        HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    ),
    String,
> {
    let envelope = admitted
        .candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "已准入候选缺少源输出包".to_string())?;
    let review = &admitted.admission_review;
    let validation = &admitted.candidate.validation;
    let mut manifest = HistoricalOutcomeOfflineDatasetOfficialSplitManifest {
        schema_version: SPLIT_MANIFEST_SCHEMA_VERSION.to_string(),
        manifest_sha256: String::new(),
        materialization_id: claim.materialization_id.clone(),
        transformation_attempt_id: review.transformation_attempt_id.clone(),
        admission_review_id: review.review_id.clone(),
        admission_review_sha256: review.review_sha256.clone(),
        validation_id: validation.validation_id.clone(),
        validation_sha256: validation.validation_sha256.clone(),
        source_output_sha256: validation.output_sha256.clone(),
        dataset_id: validation.dataset_id.clone(),
        dataset_content_sha256: validation.dataset_content_sha256.clone(),
        dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(),
        candidate_set_sha256: validation.candidate_set_sha256.clone(),
        transformation_spec_sha256: validation.transformation_spec_sha256.clone(),
        split_specification_sha256: validation.split_specification_sha256.clone(),
        source_split_manifest_candidate_sha256: validation
            .recomputed_split_manifest_candidate_sha256
            .clone(),
        boundary_audit: envelope.boundary_audit.clone(),
        records: envelope.split_manifest_candidate.clone(),
        entry_count: envelope.entry_count,
        component_count: envelope.component_count,
        sealed_holdout_labels_withheld: true,
        exact_validated_candidate_copy: true,
        independently_validated_after_materialization: false,
        eligible_for_feature_join: false,
        semantic_target_assigned: false,
        training_authorized: false,
    };
    manifest.manifest_sha256 = fingerprint_without(&manifest, &["manifest_sha256"])?;

    let mut bundle = HistoricalOutcomeOfflineDatasetOfficialFeatureBundle {
        schema_version: FEATURE_BUNDLE_SCHEMA_VERSION.to_string(),
        feature_bundle_sha256: String::new(),
        materialization_id: claim.materialization_id.clone(),
        transformation_attempt_id: review.transformation_attempt_id.clone(),
        admission_review_id: review.review_id.clone(),
        admission_review_sha256: review.review_sha256.clone(),
        validation_id: validation.validation_id.clone(),
        validation_sha256: validation.validation_sha256.clone(),
        source_output_sha256: validation.output_sha256.clone(),
        dataset_id: validation.dataset_id.clone(),
        dataset_content_sha256: validation.dataset_content_sha256.clone(),
        dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(),
        candidate_set_sha256: validation.candidate_set_sha256.clone(),
        transformation_spec_sha256: validation.transformation_spec_sha256.clone(),
        feature_specification_sha256: validation.feature_specification_sha256.clone(),
        source_feature_bundle_candidate_sha256: validation
            .recomputed_feature_bundle_candidate_sha256
            .clone(),
        source_exclusion_audit_sha256: validation.recomputed_exclusion_audit_sha256.clone(),
        feature_catalog_count: envelope.feature_catalog_count,
        feature_catalog_sha256: envelope.feature_catalog_sha256.clone(),
        feature_schema_sha256: envelope.feature_schema_sha256.clone(),
        records: envelope.feature_bundle_candidate.clone(),
        exclusion_audit: envelope.excluded_feature_audit.clone(),
        explicit_missingness_preserved: true,
        no_imputation_or_backfill_performed: true,
        outcome_future_and_current_portfolio_sources_excluded: true,
        exact_validated_candidate_copy: true,
        independently_validated_after_materialization: false,
        joined_to_outcome_labels: false,
        semantic_target_assigned: false,
        copied_to_training_store: false,
        training_authorized: false,
    };
    bundle.feature_bundle_sha256 = fingerprint_without(&bundle, &["feature_bundle_sha256"])?;
    Ok((manifest, bundle))
}

fn new_success_result(
    claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    manifest: &HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
    bundle: &HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    total_artifact_bytes: usize,
) -> Result<
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult,
    String,
> {
    let combined_artifact_sha256 = hash_serializable(&(
        manifest.manifest_sha256.as_str(),
        bundle.feature_bundle_sha256.as_str(),
        claim.claim_sha256.as_str(),
    ))?;
    let mut result = HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        materialization_id: claim.materialization_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        status: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus::CompletedPendingIndependentValidation,
        error: None,
        split_manifest_sha256: Some(manifest.manifest_sha256.clone()),
        feature_bundle_sha256: Some(bundle.feature_bundle_sha256.clone()),
        combined_artifact_sha256: Some(combined_artifact_sha256),
        total_artifact_bytes,
        official_split_manifest_created: true,
        official_feature_bundle_created: true,
        exact_validated_candidate_copy_completed: true,
        independent_output_validation_completed: false,
        official_artifacts_eligible_for_feature_join: false,
        feature_join_performed: false,
        semantic_target_assigned: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    finalize_result(&mut result)?;
    validate_result(&result, claim, Some(manifest), Some(bundle))?;
    Ok(result)
}

fn new_failed_result(
    claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    error: &str,
    total_artifact_bytes: usize,
) -> Result<
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult,
    String,
> {
    let mut result = HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        materialization_id: claim.materialization_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        status: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus::FailedClaimConsumed,
        error: Some(error.chars().take(MAX_ERROR_CHARS).collect()),
        split_manifest_sha256: None,
        feature_bundle_sha256: None,
        combined_artifact_sha256: None,
        total_artifact_bytes,
        official_split_manifest_created: false,
        official_feature_bundle_created: false,
        exact_validated_candidate_copy_completed: false,
        independent_output_validation_completed: false,
        official_artifacts_eligible_for_feature_join: false,
        feature_join_performed: false,
        semantic_target_assigned: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    finalize_result(&mut result)?;
    validate_result(&result, claim, None, None)?;
    Ok(result)
}

fn finalize_result(
    result: &mut HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult,
) -> Result<(), String> {
    result.result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].to_string();
    Ok(())
}

fn validate_claim(
    claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    admitted: &AdmittedHistoricalOutcomeOfflineTransformationCandidate,
) -> Result<(), String> {
    let review = &admitted.admission_review;
    let validation = &admitted.candidate.validation;
    let downstream_closed = !claim.ambient_environment_available
        && !claim.network_available
        && !claim.external_tools_available
        && !claim.production_state_write_available
        && !claim.historical_state_mutation_allowed
        && !claim.feature_join_allowed
        && !claim.semantic_target_assignment_allowed
        && !claim.training_allowed
        && !claim.reward_allowed
        && !claim.shadow_portfolio_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed;
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.policy_version != POLICY_VERSION
        || !valid_id(&claim.materialization_id)
        || !valid_sha256(&claim.claim_sha256)
        || claim.materialization_id != claim.claim_sha256[..32]
        || claim.claim_sha256
            != fingerprint_without(claim, &["materialization_id", "claim_sha256"])?
        || claim.transformation_attempt_id != review.transformation_attempt_id
        || claim.admission_review_id != review.review_id
        || claim.admission_review_sha256 != review.review_sha256
        || claim.validation_id != validation.validation_id
        || claim.validation_sha256 != validation.validation_sha256
        || claim.source_claim_sha256 != validation.claim_sha256
        || claim.source_result_sha256 != validation.result_sha256
        || claim.source_output_sha256 != validation.output_sha256
        || claim.dataset_id != validation.dataset_id
        || claim.dataset_content_sha256 != validation.dataset_content_sha256
        || claim.dataset_manifest_sha256 != validation.dataset_manifest_sha256
        || claim.candidate_set_sha256 != validation.candidate_set_sha256
        || claim.transformation_spec_sha256 != validation.transformation_spec_sha256
        || claim.split_specification_sha256 != validation.split_specification_sha256
        || claim.feature_specification_sha256 != validation.feature_specification_sha256
        || claim.expected_split_manifest_candidate_sha256
            != validation.recomputed_split_manifest_candidate_sha256
        || claim.expected_feature_bundle_candidate_sha256
            != validation.recomputed_feature_bundle_candidate_sha256
        || claim.expected_exclusion_audit_sha256 != validation.recomputed_exclusion_audit_sha256
        || claim.materialized_by.trim().is_empty()
        || claim.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&claim.excluded_prior_actor_ids)
        || !actor_is_independent(&claim.materialized_by, &claim.excluded_prior_actor_ids)
        || !claim.exact_current_admission_chain_revalidated
        || !claim.materializer_independent_from_complete_prior_chain
        || !claim.exact_copy_only_confirmed
        || !claim.create_once_and_failure_consumes_confirmed
        || !claim.no_join_target_training_or_trading_confirmed
        || !claim.independent_output_validation_required_confirmed
        || !claim.claim_consumed
        || !claim.official_artifact_materialization_started
        || !downstream_closed
    {
        return Err("historical offline official artifact materialization claim is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn validate_manifest(
    manifest: &HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
    claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    admitted: &AdmittedHistoricalOutcomeOfflineTransformationCandidate,
) -> Result<(), String> {
    let envelope = admitted
        .candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "validated candidate envelope is missing".to_string())?;
    let closed = !manifest.independently_validated_after_materialization
        && !manifest.eligible_for_feature_join
        && !manifest.semantic_target_assigned
        && !manifest.training_authorized;
    if manifest.schema_version != SPLIT_MANIFEST_SCHEMA_VERSION
        || !valid_sha256(&manifest.manifest_sha256)
        || manifest.manifest_sha256 != fingerprint_without(manifest, &["manifest_sha256"])?
        || manifest.materialization_id != claim.materialization_id
        || manifest.transformation_attempt_id != claim.transformation_attempt_id
        || manifest.admission_review_id != claim.admission_review_id
        || manifest.admission_review_sha256 != claim.admission_review_sha256
        || manifest.validation_id != claim.validation_id
        || manifest.validation_sha256 != claim.validation_sha256
        || manifest.source_output_sha256 != claim.source_output_sha256
        || manifest.dataset_id != claim.dataset_id
        || manifest.dataset_content_sha256 != claim.dataset_content_sha256
        || manifest.dataset_manifest_sha256 != claim.dataset_manifest_sha256
        || manifest.candidate_set_sha256 != claim.candidate_set_sha256
        || manifest.transformation_spec_sha256 != claim.transformation_spec_sha256
        || manifest.split_specification_sha256 != claim.split_specification_sha256
        || manifest.source_split_manifest_candidate_sha256
            != claim.expected_split_manifest_candidate_sha256
        || manifest.boundary_audit != envelope.boundary_audit
        || manifest.records != envelope.split_manifest_candidate
        || manifest.entry_count != envelope.entry_count
        || manifest.component_count != envelope.component_count
        || !manifest.sealed_holdout_labels_withheld
        || !manifest.exact_validated_candidate_copy
        || !closed
    {
        return Err("official split manifest is invalid, drifted or over-authorized".to_string());
    }
    Ok(())
}

fn validate_feature_bundle(
    bundle: &HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    admitted: &AdmittedHistoricalOutcomeOfflineTransformationCandidate,
) -> Result<(), String> {
    let envelope = admitted
        .candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "validated candidate envelope is missing".to_string())?;
    let closed = !bundle.independently_validated_after_materialization
        && !bundle.joined_to_outcome_labels
        && !bundle.semantic_target_assigned
        && !bundle.copied_to_training_store
        && !bundle.training_authorized;
    if bundle.schema_version != FEATURE_BUNDLE_SCHEMA_VERSION
        || !valid_sha256(&bundle.feature_bundle_sha256)
        || bundle.feature_bundle_sha256 != fingerprint_without(bundle, &["feature_bundle_sha256"])?
        || bundle.materialization_id != claim.materialization_id
        || bundle.transformation_attempt_id != claim.transformation_attempt_id
        || bundle.admission_review_id != claim.admission_review_id
        || bundle.admission_review_sha256 != claim.admission_review_sha256
        || bundle.validation_id != claim.validation_id
        || bundle.validation_sha256 != claim.validation_sha256
        || bundle.source_output_sha256 != claim.source_output_sha256
        || bundle.dataset_id != claim.dataset_id
        || bundle.dataset_content_sha256 != claim.dataset_content_sha256
        || bundle.dataset_manifest_sha256 != claim.dataset_manifest_sha256
        || bundle.candidate_set_sha256 != claim.candidate_set_sha256
        || bundle.transformation_spec_sha256 != claim.transformation_spec_sha256
        || bundle.feature_specification_sha256 != claim.feature_specification_sha256
        || bundle.source_feature_bundle_candidate_sha256
            != claim.expected_feature_bundle_candidate_sha256
        || bundle.source_exclusion_audit_sha256 != claim.expected_exclusion_audit_sha256
        || bundle.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || bundle.feature_catalog_count != envelope.feature_catalog_count
        || bundle.feature_catalog_sha256 != envelope.feature_catalog_sha256
        || bundle.feature_schema_sha256 != envelope.feature_schema_sha256
        || bundle.records != envelope.feature_bundle_candidate
        || bundle.exclusion_audit != envelope.excluded_feature_audit
        || !bundle.explicit_missingness_preserved
        || !bundle.no_imputation_or_backfill_performed
        || !bundle.outcome_future_and_current_portfolio_sources_excluded
        || !bundle.exact_validated_candidate_copy
        || !closed
    {
        return Err("official feature bundle is invalid, drifted or over-authorized".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult,
    claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    manifest: Option<&HistoricalOutcomeOfflineDatasetOfficialSplitManifest>,
    bundle: Option<&HistoricalOutcomeOfflineDatasetOfficialFeatureBundle>,
) -> Result<(), String> {
    let success = result.status
        == HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus::CompletedPendingIndependentValidation;
    let success_fields = if success {
        let manifest =
            manifest.ok_or_else(|| "successful materialization misses manifest".to_string())?;
        let bundle =
            bundle.ok_or_else(|| "successful materialization misses bundle".to_string())?;
        result.error.is_none()
            && result.split_manifest_sha256.as_deref() == Some(manifest.manifest_sha256.as_str())
            && result.feature_bundle_sha256.as_deref()
                == Some(bundle.feature_bundle_sha256.as_str())
            && result.combined_artifact_sha256.as_deref()
                == Some(
                    hash_serializable(&(
                        manifest.manifest_sha256.as_str(),
                        bundle.feature_bundle_sha256.as_str(),
                        claim.claim_sha256.as_str(),
                    ))?
                    .as_str(),
                )
            && result.total_artifact_bytes > 0
            && result.total_artifact_bytes <= MAX_TOTAL_ARTIFACT_BYTES
            && result.official_split_manifest_created
            && result.official_feature_bundle_created
            && result.exact_validated_candidate_copy_completed
    } else {
        manifest.is_none()
            && bundle.is_none()
            && result.error.as_ref().is_some_and(|value| !value.is_empty())
            && result.split_manifest_sha256.is_none()
            && result.feature_bundle_sha256.is_none()
            && result.combined_artifact_sha256.is_none()
            && !result.official_split_manifest_created
            && !result.official_feature_bundle_created
            && !result.exact_validated_candidate_copy_completed
    };
    let downstream_closed = !result.independent_output_validation_completed
        && !result.official_artifacts_eligible_for_feature_join
        && !result.feature_join_performed
        && !result.semantic_target_assigned
        && !result.training_authorized
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.policy_version != POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != fingerprint_without(result, &["result_id", "result_sha256"])?
        || result.materialization_id != claim.materialization_id
        || result.claim_sha256 != claim.claim_sha256
        || !success_fields
        || !downstream_closed
    {
        return Err(
            "official artifact materialization result is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

async fn read_attempt(
    state: &AppState,
    admitted: &AdmittedHistoricalOutcomeOfflineTransformationCandidate,
) -> Result<
    Option<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationAttempt>,
    String,
> {
    let directory =
        materialization_directory(state, &admitted.admission_review.transformation_attempt_id);
    let claim_bytes = match tokio::fs::read(directory.join("claim.json")).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let claim: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim =
        serde_json::from_slice(&claim_bytes).map_err(|error| error.to_string())?;
    validate_claim(&claim, admitted)?;
    let result_bytes = match tokio::fs::read(directory.join("result.json")).await {
        Ok(value) => Some(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.to_string()),
    };
    let Some(result_bytes) = result_bytes else {
        return Ok(Some(
            HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationAttempt {
                claim,
                result: None,
                split_manifest: None,
                feature_bundle: None,
            },
        ));
    };
    let result: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult =
        serde_json::from_slice(&result_bytes).map_err(|error| error.to_string())?;
    if result.status
        == HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus::CompletedPendingIndependentValidation
    {
        let manifest_sha256 = result
            .split_manifest_sha256
            .as_deref()
            .ok_or_else(|| "materialization result misses manifest hash".to_string())?;
        let bundle_sha256 = result
            .feature_bundle_sha256
            .as_deref()
            .ok_or_else(|| "materialization result misses bundle hash".to_string())?;
        let manifest: HistoricalOutcomeOfflineDatasetOfficialSplitManifest = serde_json::from_slice(
            &tokio::fs::read(directory.join(format!("split-manifest-{manifest_sha256}.json")))
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        let bundle: HistoricalOutcomeOfflineDatasetOfficialFeatureBundle = serde_json::from_slice(
            &tokio::fs::read(directory.join(format!("feature-bundle-{bundle_sha256}.json")))
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        validate_manifest(&manifest, &claim, admitted)?;
        validate_feature_bundle(&bundle, &claim, admitted)?;
        validate_result(&result, &claim, Some(&manifest), Some(&bundle))?;
        Ok(Some(HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationAttempt {
            claim,
            result: Some(result),
            split_manifest: Some(manifest),
            feature_bundle: Some(bundle),
        }))
    } else {
        validate_result(&result, &claim, None, None)?;
        Ok(Some(HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationAttempt {
            claim,
            result: Some(result),
            split_manifest: None,
            feature_bundle: None,
        }))
    }
}

fn excluded_actor_ids(
    admitted: &AdmittedHistoricalOutcomeOfflineTransformationCandidate,
) -> Vec<String> {
    let review = &admitted.admission_review;
    let validation = &admitted.candidate.validation;
    let mut actors = review.excluded_prior_actor_ids.clone();
    actors.push(review.reviewer_id.clone());
    actors.push(validation.validated_by.clone());
    actors.push(validation.execution_invoked_by.clone());
    actors.push(validation.runner_registered_by.clone());
    actors.push(validation.authorization_reviewer_id.clone());
    actors.retain(|actor| !actor.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn actor_is_independent(actor_id: &str, excluded_actor_ids: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded_actor_ids.iter().all(|value| value != actor_id)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "official artifact fingerprint payload is invalid".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn materialization_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join(
            "historical-outcome-offline-dataset-transformation-official-artifact-materializations",
        )
        .join(attempt_id)
}

struct MaterializationLock {
    path: PathBuf,
}

impl Drop for MaterializationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<MaterializationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-offline-transformation-official-artifact-materialization-{attempt_id}.lock"
    ));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age > StdDuration::from_secs(300));
        if stale {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一正式工件物化正在进行".to_string())?;
    Ok(MaterializationLock { path })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_claim()
    -> HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim {
        let mut claim =
            HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim {
                schema_version: CLAIM_SCHEMA_VERSION.into(),
                policy_version: POLICY_VERSION.into(),
                materialization_id: String::new(),
                claim_sha256: String::new(),
                transformation_attempt_id: "1".repeat(32),
                admission_review_id: "2".repeat(32),
                admission_review_sha256: "3".repeat(64),
                validation_id: "4".repeat(32),
                validation_sha256: "5".repeat(64),
                source_claim_sha256: "6".repeat(64),
                source_result_sha256: "7".repeat(64),
                source_output_sha256: "8".repeat(64),
                dataset_id: "9".repeat(32),
                dataset_content_sha256: "a".repeat(64),
                dataset_manifest_sha256: "b".repeat(64),
                candidate_set_sha256: "c".repeat(64),
                transformation_spec_sha256: "d".repeat(64),
                split_specification_sha256: "e".repeat(64),
                feature_specification_sha256: "f".repeat(64),
                expected_split_manifest_candidate_sha256: "1".repeat(64),
                expected_feature_bundle_candidate_sha256: "2".repeat(64),
                expected_exclusion_audit_sha256: "3".repeat(64),
                materialized_by: "materializer".into(),
                excluded_prior_actor_ids: vec!["admitter".into(), "validator".into()],
                claimed_at: Utc::now(),
                exact_current_admission_chain_revalidated: true,
                materializer_independent_from_complete_prior_chain: true,
                exact_copy_only_confirmed: true,
                create_once_and_failure_consumes_confirmed: true,
                no_join_target_training_or_trading_confirmed: true,
                independent_output_validation_required_confirmed: true,
                claim_consumed: true,
                official_artifact_materialization_started: true,
                ambient_environment_available: false,
                network_available: false,
                external_tools_available: false,
                production_state_write_available: false,
                historical_state_mutation_allowed: false,
                feature_join_allowed: false,
                semantic_target_assignment_allowed: false,
                training_allowed: false,
                reward_allowed: false,
                shadow_portfolio_allowed: false,
                order_generation_allowed: false,
                broker_access_allowed: false,
                trading_allowed: false,
            };
        claim.claim_sha256 =
            fingerprint_without(&claim, &["materialization_id", "claim_sha256"]).unwrap();
        claim.materialization_id = claim.claim_sha256[..32].to_string();
        claim
    }

    fn minimal_manifest(
        claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    ) -> HistoricalOutcomeOfflineDatasetOfficialSplitManifest {
        let mut manifest = HistoricalOutcomeOfflineDatasetOfficialSplitManifest {
            schema_version: SPLIT_MANIFEST_SCHEMA_VERSION.into(),
            manifest_sha256: String::new(),
            materialization_id: claim.materialization_id.clone(),
            transformation_attempt_id: claim.transformation_attempt_id.clone(),
            admission_review_id: claim.admission_review_id.clone(),
            admission_review_sha256: claim.admission_review_sha256.clone(),
            validation_id: claim.validation_id.clone(),
            validation_sha256: claim.validation_sha256.clone(),
            source_output_sha256: claim.source_output_sha256.clone(),
            dataset_id: claim.dataset_id.clone(),
            dataset_content_sha256: claim.dataset_content_sha256.clone(),
            dataset_manifest_sha256: claim.dataset_manifest_sha256.clone(),
            candidate_set_sha256: claim.candidate_set_sha256.clone(),
            transformation_spec_sha256: claim.transformation_spec_sha256.clone(),
            split_specification_sha256: claim.split_specification_sha256.clone(),
            source_split_manifest_candidate_sha256: claim
                .expected_split_manifest_candidate_sha256
                .clone(),
            boundary_audit: HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit {
                schema_version: "audit".into(),
                candidate_pair_count: 1,
                selected_train_component_end_exclusive: 1,
                selected_validation_component_end_exclusive: 1,
                pre_purge_train_entry_count: 1,
                pre_purge_validation_entry_count: 1,
                pre_purge_sealed_holdout_entry_count: 1,
                objective_tuple: [1, 2, 3, 4],
                all_candidate_objectives_sha256: "4".repeat(64),
                audit_sha256: "5".repeat(64),
            },
            records: Vec::new(),
            entry_count: 3,
            component_count: 3,
            sealed_holdout_labels_withheld: true,
            exact_validated_candidate_copy: true,
            independently_validated_after_materialization: false,
            eligible_for_feature_join: false,
            semantic_target_assigned: false,
            training_authorized: false,
        };
        manifest.manifest_sha256 = fingerprint_without(&manifest, &["manifest_sha256"]).unwrap();
        manifest
    }

    fn minimal_bundle(
        claim: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    ) -> HistoricalOutcomeOfflineDatasetOfficialFeatureBundle {
        let mut bundle = HistoricalOutcomeOfflineDatasetOfficialFeatureBundle {
            schema_version: FEATURE_BUNDLE_SCHEMA_VERSION.into(),
            feature_bundle_sha256: String::new(),
            materialization_id: claim.materialization_id.clone(),
            transformation_attempt_id: claim.transformation_attempt_id.clone(),
            admission_review_id: claim.admission_review_id.clone(),
            admission_review_sha256: claim.admission_review_sha256.clone(),
            validation_id: claim.validation_id.clone(),
            validation_sha256: claim.validation_sha256.clone(),
            source_output_sha256: claim.source_output_sha256.clone(),
            dataset_id: claim.dataset_id.clone(),
            dataset_content_sha256: claim.dataset_content_sha256.clone(),
            dataset_manifest_sha256: claim.dataset_manifest_sha256.clone(),
            candidate_set_sha256: claim.candidate_set_sha256.clone(),
            transformation_spec_sha256: claim.transformation_spec_sha256.clone(),
            feature_specification_sha256: claim.feature_specification_sha256.clone(),
            source_feature_bundle_candidate_sha256: claim
                .expected_feature_bundle_candidate_sha256
                .clone(),
            source_exclusion_audit_sha256: claim.expected_exclusion_audit_sha256.clone(),
            feature_catalog_count: EXPECTED_FEATURE_COUNT,
            feature_catalog_sha256: "6".repeat(64),
            feature_schema_sha256: "7".repeat(64),
            records: Vec::new(),
            exclusion_audit: Vec::new(),
            explicit_missingness_preserved: true,
            no_imputation_or_backfill_performed: true,
            outcome_future_and_current_portfolio_sources_excluded: true,
            exact_validated_candidate_copy: true,
            independently_validated_after_materialization: false,
            joined_to_outcome_labels: false,
            semantic_target_assigned: false,
            copied_to_training_store: false,
            training_authorized: false,
        };
        bundle.feature_bundle_sha256 =
            fingerprint_without(&bundle, &["feature_bundle_sha256"]).unwrap();
        bundle
    }

    #[test]
    fn materializer_must_be_outside_complete_prior_actor_set() {
        let actors = vec!["admitter".to_string(), "validator".to_string()];
        assert!(actor_is_independent("materializer", &actors));
        assert!(!actor_is_independent("validator", &actors));
    }

    #[test]
    fn claim_hash_binds_downstream_authority_fields() {
        let claim = minimal_claim();
        let original =
            fingerprint_without(&claim, &["materialization_id", "claim_sha256"]).unwrap();
        let mut tampered = claim;
        tampered.training_allowed = true;
        assert_ne!(
            original,
            fingerprint_without(&tampered, &["materialization_id", "claim_sha256"]).unwrap()
        );
    }

    #[test]
    fn official_artifacts_remain_unvalidated_and_unjoined() {
        let claim = minimal_claim();
        let manifest = minimal_manifest(&claim);
        let bundle = minimal_bundle(&claim);
        assert!(!manifest.independently_validated_after_materialization);
        assert!(!manifest.eligible_for_feature_join);
        assert!(!bundle.independently_validated_after_materialization);
        assert!(!bundle.joined_to_outcome_labels);
        assert!(!bundle.training_authorized);
    }

    #[test]
    fn successful_result_never_opens_downstream_authority() {
        let claim = minimal_claim();
        let manifest = minimal_manifest(&claim);
        let bundle = minimal_bundle(&claim);
        let result = new_success_result(&claim, &manifest, &bundle, 1_024).unwrap();
        assert!(result.official_split_manifest_created);
        assert!(result.official_feature_bundle_created);
        assert!(!result.independent_output_validation_completed);
        assert!(!result.training_authorized);
        assert!(!result.trading_authorized);
    }

    #[test]
    fn artifact_fingerprint_detects_one_field_tamper() {
        let claim = minimal_claim();
        let manifest = minimal_manifest(&claim);
        let original = manifest.manifest_sha256.clone();
        let mut tampered = manifest;
        tampered.training_authorized = true;
        assert_ne!(
            original,
            fingerprint_without(&tampered, &["manifest_sha256"]).unwrap()
        );
    }
}
