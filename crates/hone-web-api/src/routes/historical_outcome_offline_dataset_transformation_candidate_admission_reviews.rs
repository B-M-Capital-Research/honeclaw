//! Independent admission review for one exact, independently validated offline
//! transformation candidate.
//!
//! Approval here creates only future create-once official-artifact
//! materialization eligibility for the exact candidate. It does not create an
//! official split manifest or feature bundle, join features, assign a target,
//! train, reward, shadow, order, access a broker or trade.

use std::collections::{BTreeSet, HashMap};
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

use super::historical_outcome_offline_dataset_transformation_output_validations::{
    HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord,
    IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate,
    exact_independently_validated_transformation_candidate_for_admission,
    independently_validated_transformation_candidates_for_admission_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-candidate-admission-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-candidate-admission-review-v1";
const POLICY_VERSION: &str =
    "hone-offline-transformation-candidate-independent-admission-v1-no-materialization";
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 2_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict {
    ApprovedForFutureCreateOnceOfficialArtifactMaterialization,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_validation_id: String,
    expected_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_transformation_spec_sha256: String,
    expected_split_specification_sha256: String,
    expected_feature_specification_sha256: String,
    verdict: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_candidate_and_validation_chain_confirmed: bool,
    transitive_component_isolation_confirmed: bool,
    deterministic_chronological_boundary_and_full_objective_audit_confirmed: bool,
    purge_embargo_and_non_empty_partitions_confirmed: bool,
    sealed_holdout_labels_withheld_confirmed: bool,
    point_in_time_feature_allowlist_and_provenance_confirmed: bool,
    explicit_missingness_without_imputation_confirmed: bool,
    outcome_future_and_current_portfolio_exclusion_confirmed: bool,
    official_artifact_contract_and_create_once_scope_confirmed: bool,
    admission_materialization_and_output_validation_separation_confirmed: bool,
    downstream_authority_remains_closed_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub transformation_attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub transformation_spec_sha256: String,
    pub split_specification_sha256: String,
    pub feature_specification_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub recomputed_boundary_audit_sha256: String,
    pub recomputed_split_manifest_candidate_sha256: String,
    pub recomputed_feature_bundle_candidate_sha256: String,
    pub recomputed_exclusion_audit_sha256: String,
    pub entry_count: usize,
    pub component_count: usize,
    pub feature_catalog_count: usize,
    pub split_record_count: usize,
    pub feature_record_count: usize,
    pub exclusion_audit_record_count: usize,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub reviewer_independent_from_validation_execution_and_complete_prior_chain: bool,
    pub exact_current_candidate_and_validation_chain_confirmed: bool,
    pub transitive_component_isolation_confirmed: bool,
    pub deterministic_chronological_boundary_and_full_objective_audit_confirmed: bool,
    pub purge_embargo_and_non_empty_partitions_confirmed: bool,
    pub sealed_holdout_labels_withheld_confirmed: bool,
    pub point_in_time_feature_allowlist_and_provenance_confirmed: bool,
    pub explicit_missingness_without_imputation_confirmed: bool,
    pub outcome_future_and_current_portfolio_exclusion_confirmed: bool,
    pub official_artifact_contract_and_create_once_scope_confirmed: bool,
    pub admission_materialization_and_output_validation_separation_confirmed: bool,
    pub downstream_authority_remains_closed_confirmed: bool,
    pub transformation_candidate_admitted: bool,
    pub future_create_once_official_artifact_materialization_eligible: bool,
    pub official_artifact_materialization_started: bool,
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
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionItem {
    pub candidate: IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate,
    pub latest_review:
        Option<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview>,
    pub current_binding: bool,
    pub review_eligible: bool,
    pub transformation_candidate_admitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionItem>,
    pub independently_validated_candidate_count: usize,
    pub review_eligible_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_candidate_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub admission_status: String,
    pub candidate_admission_review_available: bool,
    pub official_artifact_materialization_enabled: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReadinessSummary {
    pub independently_validated_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_candidate_count: usize,
    pub rejected_or_changes_requested_count: usize,
    pub admission_status: String,
    pub official_artifact_materialization_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdmittedHistoricalOutcomeOfflineTransformationCandidate {
    pub candidate: IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate,
    pub admission_review: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_candidate_admission_reviews(
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
            warn!(%error, "historical offline transformation candidate admission registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线转换候选准入复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_offline_dataset_transformation_candidate_admission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<
        ReviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRequest,
    >,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_admission(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_transformation_candidate_admission_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReadinessSummary, String>
{
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReadinessSummary {
            independently_validated_candidate_count: registry
                .independently_validated_candidate_count,
            reviewed_candidate_count: registry.reviewed_candidate_count,
            admitted_candidate_count: registry.admitted_candidate_count,
            rejected_or_changes_requested_count: registry.changes_requested_or_rejected_count,
            admission_status: registry.admission_status,
            official_artifact_materialization_enabled: false,
        },
    )
}

pub(crate) async fn admitted_transformation_candidates_for_official_artifact_materialization(
    state: &AppState,
) -> Result<Vec<AdmittedHistoricalOutcomeOfflineTransformationCandidate>, String> {
    let candidates =
        independently_validated_transformation_candidates_for_admission_review(state).await?;
    let mut admitted = Vec::new();
    for candidate in candidates {
        let Some(admission_review) = latest_review(state, &candidate).await? else {
            continue;
        };
        if admission_review.transformation_candidate_admitted
            && admission_review.future_create_once_official_artifact_materialization_eligible
            && admission_review.verdict
                == HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialArtifactMaterialization
        {
            validate_review(&admission_review, &candidate)?;
            admitted.push(AdmittedHistoricalOutcomeOfflineTransformationCandidate {
                candidate,
                admission_review,
            });
        }
    }
    admitted.sort_by(|left, right| {
        right
            .admission_review
            .submitted_at
            .cmp(&left.admission_review.submitted_at)
            .then_with(|| {
                right
                    .admission_review
                    .review_id
                    .cmp(&left.admission_review.review_id)
            })
    });
    Ok(admitted)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn exact_admitted_transformation_candidate_for_official_artifact_materialization(
    state: &AppState,
    attempt_id: &str,
    admission_review_id: &str,
    admission_review_sha256: &str,
    validation_sha256: &str,
    output_sha256: &str,
    dataset_content_sha256: &str,
    dataset_manifest_sha256: &str,
    candidate_set_sha256: &str,
    transformation_spec_sha256: &str,
    split_specification_sha256: &str,
    feature_specification_sha256: &str,
) -> Result<Option<AdmittedHistoricalOutcomeOfflineTransformationCandidate>, String> {
    let candidates =
        admitted_transformation_candidates_for_official_artifact_materialization(state).await?;
    Ok(candidates.into_iter().find(|value| {
        let review = &value.admission_review;
        review.transformation_attempt_id == attempt_id
            && review.review_id == admission_review_id
            && review.review_sha256 == admission_review_sha256
            && review.validation_sha256 == validation_sha256
            && review.output_sha256 == output_sha256
            && review.dataset_content_sha256 == dataset_content_sha256
            && review.dataset_manifest_sha256 == dataset_manifest_sha256
            && review.candidate_set_sha256 == candidate_set_sha256
            && review.transformation_spec_sha256 == transformation_spec_sha256
            && review.split_specification_sha256 == split_specification_sha256
            && review.feature_specification_sha256 == feature_specification_sha256
    }))
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry, String> {
    let candidates =
        independently_validated_transformation_candidates_for_admission_review(state).await?;
    let mut items = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let latest_review = latest_review(state, &candidate).await?;
        let admitted = latest_review
            .as_ref()
            .is_some_and(|review| review.transformation_candidate_admitted);
        items.push(
            HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionItem {
                candidate,
                latest_review,
                current_binding: true,
                review_eligible: true,
                transformation_candidate_admitted: admitted,
            },
        );
    }
    let independently_validated_candidate_count = items.len();
    let reviewed_candidate_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let admitted_candidate_count = items
        .iter()
        .filter(|item| item.transformation_candidate_admitted)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    != HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialArtifactMaterialization
            })
        })
        .count();
    let admission_status = if admitted_candidate_count > 0 {
        "candidate_admitted_waiting_separate_create_once_official_artifact_materialization"
    } else if changes_requested_or_rejected_count > 0 {
        "candidate_admission_changes_requested_or_rejected"
    } else if independently_validated_candidate_count > 0 {
        "waiting_independent_transformation_candidate_admission_review"
    } else {
        "waiting_independently_validated_transformation_candidate"
    };
    Ok(
        HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            items,
            independently_validated_candidate_count,
            review_eligible_candidate_count: independently_validated_candidate_count,
            reviewed_candidate_count,
            admitted_candidate_count,
            changes_requested_or_rejected_count,
            admission_status: admission_status.to_string(),
            candidate_admission_review_available: true,
            official_artifact_materialization_enabled: false,
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
            scope: "第 33 阶段由另一名独立管理员复核一条精确、当前且已独立重算通过的候选。批准只授予未来 create-once 正式 manifest/feature bundle 物化资格；本阶段不物化、不 join、不定义目标、不训练、不奖励，也不开放影子、订单、券商或交易。".to_string(),
        },
    )
}

async fn review_admission(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewHistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionRequest,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview, String> {
    if !valid_id(attempt_id) {
        return Err("离线转换执行尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    if super::historical_outcome_offline_dataset_transformation_official_artifact_materializations::official_artifact_materialization_claim_exists(
        state,
        attempt_id,
    ) {
        return Err(
            "正式工件物化 claim 已存在，候选准入复核链已经永久冻结，不能继续追加复核"
                .to_string(),
        );
    }
    let candidate = exact_independently_validated_transformation_candidate_for_admission(
        state,
        attempt_id,
        &request.expected_validation_id,
        &request.expected_validation_sha256,
        &request.expected_claim_sha256,
        &request.expected_result_sha256,
        &request.expected_output_sha256,
        &request.expected_dataset_content_sha256,
        &request.expected_dataset_manifest_sha256,
        &request.expected_candidate_set_sha256,
        &request.expected_transformation_spec_sha256,
        &request.expected_split_specification_sha256,
        &request.expected_feature_specification_sha256,
    )
    .await?
    .ok_or_else(|| "当前没有这条精确绑定且独立校验通过的离线转换候选".to_string())?;
    let latest = latest_review(state, &candidate).await?;
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("离线转换候选准入复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限与偏差",
    )?;
    let excluded_prior_actor_ids = excluded_actor_ids(&candidate.validation, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("候选准入复核人不能是输出校验、执行、完整上游或此前准入复核角色".to_string());
    }
    let envelope = candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "离线转换候选缺少未信任输出包".to_string())?;
    let validation = &candidate.validation;
    let mut review = HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        transformation_attempt_id: validation.transformation_attempt_id.clone(),
        claim_sha256: validation.claim_sha256.clone(),
        result_id: validation.result_id.clone(),
        result_sha256: validation.result_sha256.clone(),
        output_sha256: validation.output_sha256.clone(),
        validation_id: validation.validation_id.clone(),
        validation_sha256: validation.validation_sha256.clone(),
        dataset_id: validation.dataset_id.clone(),
        dataset_content_sha256: validation.dataset_content_sha256.clone(),
        dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(),
        candidate_set_sha256: validation.candidate_set_sha256.clone(),
        transformation_spec_sha256: validation.transformation_spec_sha256.clone(),
        split_specification_sha256: validation.split_specification_sha256.clone(),
        feature_specification_sha256: validation.feature_specification_sha256.clone(),
        implementation_id: validation.implementation_id.clone(),
        implementation_sha256: validation.implementation_sha256.clone(),
        isolated_runner_id: validation.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: validation.isolated_runner_spec_sha256.clone(),
        authorization_review_id: validation.authorization_review_id.clone(),
        authorization_review_sha256: validation.authorization_review_sha256.clone(),
        recomputed_boundary_audit_sha256: validation.recomputed_boundary_audit_sha256.clone(),
        recomputed_split_manifest_candidate_sha256: validation
            .recomputed_split_manifest_candidate_sha256
            .clone(),
        recomputed_feature_bundle_candidate_sha256: validation
            .recomputed_feature_bundle_candidate_sha256
            .clone(),
        recomputed_exclusion_audit_sha256: validation.recomputed_exclusion_audit_sha256.clone(),
        entry_count: envelope.entry_count,
        component_count: envelope.component_count,
        feature_catalog_count: envelope.feature_catalog_count,
        split_record_count: envelope.split_manifest_candidate.len(),
        feature_record_count: envelope.feature_bundle_candidate.len(),
        exclusion_audit_record_count: envelope.excluded_feature_audit.len(),
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        known_limitations,
        reviewer_independent_from_validation_execution_and_complete_prior_chain: independent,
        exact_current_candidate_and_validation_chain_confirmed: request
            .exact_current_candidate_and_validation_chain_confirmed,
        transitive_component_isolation_confirmed: request.transitive_component_isolation_confirmed,
        deterministic_chronological_boundary_and_full_objective_audit_confirmed: request
            .deterministic_chronological_boundary_and_full_objective_audit_confirmed,
        purge_embargo_and_non_empty_partitions_confirmed: request
            .purge_embargo_and_non_empty_partitions_confirmed,
        sealed_holdout_labels_withheld_confirmed: request.sealed_holdout_labels_withheld_confirmed,
        point_in_time_feature_allowlist_and_provenance_confirmed: request
            .point_in_time_feature_allowlist_and_provenance_confirmed,
        explicit_missingness_without_imputation_confirmed: request
            .explicit_missingness_without_imputation_confirmed,
        outcome_future_and_current_portfolio_exclusion_confirmed: request
            .outcome_future_and_current_portfolio_exclusion_confirmed,
        official_artifact_contract_and_create_once_scope_confirmed: request
            .official_artifact_contract_and_create_once_scope_confirmed,
        admission_materialization_and_output_validation_separation_confirmed: request
            .admission_materialization_and_output_validation_separation_confirmed,
        downstream_authority_remains_closed_confirmed: request
            .downstream_authority_remains_closed_confirmed,
        transformation_candidate_admitted: false,
        future_create_once_official_artifact_materialization_eligible: false,
        official_artifact_materialization_started: false,
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
    };
    let approved = review.verdict
        == HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialArtifactMaterialization;
    review.transformation_candidate_admitted =
        approved && review_all_checks(&review) && independent;
    review.future_create_once_official_artifact_materialization_eligible =
        review.transformation_candidate_admitted;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &candidate)?;
    write_immutable_json(
        &review_directory(state, attempt_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn review_all_checks(
    review: &HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview,
) -> bool {
    review.exact_current_candidate_and_validation_chain_confirmed
        && review.transitive_component_isolation_confirmed
        && review.deterministic_chronological_boundary_and_full_objective_audit_confirmed
        && review.purge_embargo_and_non_empty_partitions_confirmed
        && review.sealed_holdout_labels_withheld_confirmed
        && review.point_in_time_feature_allowlist_and_provenance_confirmed
        && review.explicit_missingness_without_imputation_confirmed
        && review.outcome_future_and_current_portfolio_exclusion_confirmed
        && review.official_artifact_contract_and_create_once_scope_confirmed
        && review.admission_materialization_and_output_validation_separation_confirmed
        && review.downstream_authority_remains_closed_confirmed
}

fn excluded_actor_ids(
    validation: &HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord,
    latest: &Option<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview>,
) -> Vec<String> {
    let mut actors = validation.excluded_prior_actor_ids.clone();
    actors.push(validation.validated_by.clone());
    actors.push(validation.execution_invoked_by.clone());
    actors.push(validation.runner_registered_by.clone());
    actors.push(validation.authorization_reviewer_id.clone());
    if let Some(review) = latest {
        actors.extend(review.excluded_prior_actor_ids.clone());
        actors.push(review.reviewer_id.clone());
    }
    actors.retain(|actor| !actor.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn reviewer_is_independent(reviewer_id: &str, excluded_actor_ids: &[String]) -> bool {
    !reviewer_id.trim().is_empty() && excluded_actor_ids.iter().all(|actor| actor != reviewer_id)
}

fn validate_review(
    review: &HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview,
    candidate: &IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate,
) -> Result<(), String> {
    let validation = &candidate.validation;
    let envelope = candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "validated candidate envelope is missing".to_string())?;
    let approved = review.verdict
        == HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialArtifactMaterialization;
    let expected_independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_admitted = approved && review_all_checks(review) && expected_independent;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let downstream_closed = !review.official_artifact_materialization_started
        && !review.official_split_manifest_created
        && !review.official_feature_bundle_created
        && !review.feature_join_performed
        && !review.semantic_target_assigned
        && !review.training_authorized
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
        || review.transformation_attempt_id != validation.transformation_attempt_id
        || review.claim_sha256 != validation.claim_sha256
        || review.result_id != validation.result_id
        || review.result_sha256 != validation.result_sha256
        || review.output_sha256 != validation.output_sha256
        || review.validation_id != validation.validation_id
        || review.validation_sha256 != validation.validation_sha256
        || review.dataset_id != validation.dataset_id
        || review.dataset_content_sha256 != validation.dataset_content_sha256
        || review.dataset_manifest_sha256 != validation.dataset_manifest_sha256
        || review.candidate_set_sha256 != validation.candidate_set_sha256
        || review.transformation_spec_sha256 != validation.transformation_spec_sha256
        || review.split_specification_sha256 != validation.split_specification_sha256
        || review.feature_specification_sha256 != validation.feature_specification_sha256
        || review.implementation_id != validation.implementation_id
        || review.implementation_sha256 != validation.implementation_sha256
        || review.isolated_runner_id != validation.isolated_runner_id
        || review.isolated_runner_spec_sha256 != validation.isolated_runner_spec_sha256
        || review.authorization_review_id != validation.authorization_review_id
        || review.authorization_review_sha256 != validation.authorization_review_sha256
        || review.recomputed_boundary_audit_sha256 != validation.recomputed_boundary_audit_sha256
        || review.recomputed_split_manifest_candidate_sha256
            != validation.recomputed_split_manifest_candidate_sha256
        || review.recomputed_feature_bundle_candidate_sha256
            != validation.recomputed_feature_bundle_candidate_sha256
        || review.recomputed_exclusion_audit_sha256 != validation.recomputed_exclusion_audit_sha256
        || review.entry_count != envelope.entry_count
        || review.component_count != envelope.component_count
        || review.feature_catalog_count != envelope.feature_catalog_count
        || review.split_record_count != envelope.split_manifest_candidate.len()
        || review.feature_record_count != envelope.feature_bundle_candidate.len()
        || review.exclusion_audit_record_count != envelope.excluded_feature_audit.len()
        || review.reviewer_id.trim().is_empty()
        || review.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || review.known_limitations.trim().is_empty()
        || review.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || review.reviewer_independent_from_validation_execution_and_complete_prior_chain
            != expected_independent
        || (approved && !review_all_checks(review))
        || review.transformation_candidate_admitted != expected_admitted
        || review.future_create_once_official_artifact_materialization_eligible != expected_admitted
        || !downstream_closed
    {
        return Err("historical offline transformation candidate admission review is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn review_fingerprint(
    review: &HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    candidate: &IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate,
) -> Result<Option<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview>, String> {
    let directory = review_directory(state, &candidate.validation.transformation_attempt_id);
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
        let review: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, candidate)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err(
                "historical offline transformation candidate admission filename mismatch"
                    .to_string(),
            );
        }
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err(
                "historical offline transformation candidate admission contains duplicate records"
                    .to_string(),
            );
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview],
) -> Result<Option<HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview>, String> {
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
            "historical offline transformation candidate admission chain has invalid tips"
                .to_string(),
        );
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err(
                "historical offline transformation candidate admission chain has a cycle"
                    .to_string(),
            );
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(previous_id), Some(previous_sha256)) => {
                let previous = by_id.get(previous_id.as_str()).ok_or_else(|| {
                    "historical offline transformation candidate admission chain is disconnected".to_string()
                })?;
                if previous.review_sha256 != *previous_sha256 {
                    return Err("historical offline transformation candidate admission previous hash mismatch".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err(
                "historical offline transformation candidate admission previous link is invalid"
                    .to_string(),
            ),
        };
    }
    if visited.len() != reviews.len() {
        return Err(
            "historical offline transformation candidate admission chain is disconnected"
                .to_string(),
        );
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "candidate admission fingerprint payload is invalid".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > max_chars {
        return Err(format!("{label}不能超过 {max_chars} 个字符"));
    }
    Ok(value.to_string())
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

fn review_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-offline-dataset-transformation-candidate-admission-reviews")
        .join(attempt_id)
}

struct AdmissionLock {
    path: PathBuf,
}

impl Drop for AdmissionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<AdmissionLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-offline-transformation-candidate-admission-{attempt_id}.lock"
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
        .map_err(|_| "已有同一离线转换候选准入复核正在进行".to_string())?;
    Ok(AdmissionLock { path })
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_review() -> HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview {
        HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionReview {
            schema_version: REVIEW_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id: None,
            previous_review_sha256: None,
            transformation_attempt_id: "1".repeat(32),
            claim_sha256: "2".repeat(64),
            result_id: "3".repeat(32),
            result_sha256: "4".repeat(64),
            output_sha256: "5".repeat(64),
            validation_id: "6".repeat(32),
            validation_sha256: "7".repeat(64),
            dataset_id: "8".repeat(32),
            dataset_content_sha256: "9".repeat(64),
            dataset_manifest_sha256: "a".repeat(64),
            candidate_set_sha256: "b".repeat(64),
            transformation_spec_sha256: "c".repeat(64),
            split_specification_sha256: "d".repeat(64),
            feature_specification_sha256: "e".repeat(64),
            implementation_id: "f".repeat(32),
            implementation_sha256: "1".repeat(64),
            isolated_runner_id: "2".repeat(32),
            isolated_runner_spec_sha256: "3".repeat(64),
            authorization_review_id: "4".repeat(32),
            authorization_review_sha256: "5".repeat(64),
            recomputed_boundary_audit_sha256: "6".repeat(64),
            recomputed_split_manifest_candidate_sha256: "7".repeat(64),
            recomputed_feature_bundle_candidate_sha256: "8".repeat(64),
            recomputed_exclusion_audit_sha256: "9".repeat(64),
            entry_count: 3,
            component_count: 3,
            feature_catalog_count: 65,
            split_record_count: 3,
            feature_record_count: 195,
            exclusion_audit_record_count: 3,
            submitted_at: Utc::now(),
            reviewer_id: "reviewer".into(),
            excluded_prior_actor_ids: vec!["prior".into(), "validator".into()],
            verdict: HistoricalOutcomeOfflineDatasetTransformationCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialArtifactMaterialization,
            rationale: "逐项复核通过".into(),
            known_limitations: "样本量仍有限，正式产物仍需独立物化和再校验".into(),
            reviewer_independent_from_validation_execution_and_complete_prior_chain: true,
            exact_current_candidate_and_validation_chain_confirmed: true,
            transitive_component_isolation_confirmed: true,
            deterministic_chronological_boundary_and_full_objective_audit_confirmed: true,
            purge_embargo_and_non_empty_partitions_confirmed: true,
            sealed_holdout_labels_withheld_confirmed: true,
            point_in_time_feature_allowlist_and_provenance_confirmed: true,
            explicit_missingness_without_imputation_confirmed: true,
            outcome_future_and_current_portfolio_exclusion_confirmed: true,
            official_artifact_contract_and_create_once_scope_confirmed: true,
            admission_materialization_and_output_validation_separation_confirmed: true,
            downstream_authority_remains_closed_confirmed: true,
            transformation_candidate_admitted: true,
            future_create_once_official_artifact_materialization_eligible: true,
            official_artifact_materialization_started: false,
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
        }
    }

    #[test]
    fn approval_requires_every_admission_check() {
        let mut review = minimal_review();
        assert!(review_all_checks(&review));
        review.explicit_missingness_without_imputation_confirmed = false;
        assert!(!review_all_checks(&review));
    }

    #[test]
    fn reviewer_must_be_outside_complete_prior_actor_set() {
        let actors = vec!["executor".to_string(), "validator".to_string()];
        assert!(reviewer_is_independent("new-reviewer", &actors));
        assert!(!reviewer_is_independent("validator", &actors));
    }

    #[test]
    fn review_hash_binds_all_materialization_eligibility_fields() {
        let mut review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        review.official_split_manifest_created = true;
        assert_ne!(first, review_fingerprint(&review).unwrap());
    }

    #[test]
    fn admission_chain_rejects_forks() {
        let mut root = minimal_review();
        root.review_sha256 = review_fingerprint(&root).unwrap();
        root.review_id = root.review_sha256[..32].to_string();
        let mut left = root.clone();
        left.previous_review_id = Some(root.review_id.clone());
        left.previous_review_sha256 = Some(root.review_sha256.clone());
        left.submitted_at = root.submitted_at + chrono::Duration::seconds(1);
        left.review_sha256 = review_fingerprint(&left).unwrap();
        left.review_id = left.review_sha256[..32].to_string();
        let mut right = left.clone();
        right.rationale = "另一条分叉".into();
        right.review_sha256 = review_fingerprint(&right).unwrap();
        right.review_id = right.review_sha256[..32].to_string();
        assert!(latest_review_from_records(&[root, left, right]).is_err());
    }
}
