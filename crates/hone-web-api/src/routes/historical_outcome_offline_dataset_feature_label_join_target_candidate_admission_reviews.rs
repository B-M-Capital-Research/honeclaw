//! Independent admission review for one exact Stage 43 join/target candidate.
//!
//! Approval creates only future create-once official joined-dataset
//! materialization eligibility. It does not materialize a dataset, expose
//! validation or sealed-holdout targets, copy to a training store, train,
//! reward, shadow, generate an order, access a broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_output_validations::{
    HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord,
    IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
    exact_independently_validated_join_target_candidate_for_admission_review,
    independently_validated_join_target_candidates_for_admission_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-candidate-admission-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-candidate-admission-review-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-candidate-independent-admission-v1-no-materialization";
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 2_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict {
    ApprovedForFutureCreateOnceOfficialJoinedDatasetMaterialization,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_validation_id: String,
    expected_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_implementation_sha256: String,
    expected_specification_sha256: String,
    expected_join_specification_sha256: String,
    expected_target_specification_sha256: String,
    expected_split_manifest_sha256: String,
    expected_feature_bundle_sha256: String,
    expected_combined_artifact_sha256: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_recomputed_rows_sha256: String,
    expected_recomputed_excluded_rows_sha256: String,
    expected_recomputed_target_commitments_sha256: String,
    verdict: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_candidate_validation_and_complete_chain_confirmed: bool,
    exact_one_to_one_entry_join_and_cardinality_confirmed: bool,
    exact_65_feature_catalog_confirmed: bool,
    point_in_time_and_explicit_missingness_confirmed: bool,
    official_split_purge_and_embargo_confirmed: bool,
    train_only_target_visibility_confirmed: bool,
    validation_targets_withheld_confirmed: bool,
    sealed_holdout_targets_withheld_confirmed: bool,
    exact_nine_raw_f64_bits_and_commitments_confirmed: bool,
    no_action_position_or_reward_semantics_confirmed: bool,
    create_once_materialization_and_post_materialization_validation_separation_confirmed: bool,
    downstream_authority_remains_closed_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_id: String,
    pub isolated_runner_spec_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub specification_id: String,
    pub specification_sha256: String,
    pub join_specification_sha256: String,
    pub target_specification_sha256: String,
    pub split_manifest_sha256: String,
    pub feature_bundle_sha256: String,
    pub combined_artifact_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub recomputed_rows_sha256: String,
    pub recomputed_excluded_rows_sha256: String,
    pub recomputed_target_commitments_sha256: String,
    pub dataset_entry_count: usize,
    pub active_candidate_row_count: usize,
    pub excluded_purge_or_embargo_row_count: usize,
    pub feature_catalog_count: usize,
    pub target_count: usize,
    pub train_target_vector_count: usize,
    pub validation_target_withheld_count: usize,
    pub sealed_holdout_target_withheld_count: usize,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub reviewer_independent_from_validator_executor_and_complete_prior_chain: bool,
    pub exact_current_candidate_validation_and_complete_chain_confirmed: bool,
    pub exact_one_to_one_entry_join_and_cardinality_confirmed: bool,
    pub exact_65_feature_catalog_confirmed: bool,
    pub point_in_time_and_explicit_missingness_confirmed: bool,
    pub official_split_purge_and_embargo_confirmed: bool,
    pub train_only_target_visibility_confirmed: bool,
    pub validation_targets_withheld_confirmed: bool,
    pub sealed_holdout_targets_withheld_confirmed: bool,
    pub exact_nine_raw_f64_bits_and_commitments_confirmed: bool,
    pub no_action_position_or_reward_semantics_confirmed: bool,
    pub create_once_materialization_and_post_materialization_validation_separation_confirmed: bool,
    pub downstream_authority_remains_closed_confirmed: bool,
    pub join_target_candidate_admitted: bool,
    pub future_create_once_official_joined_dataset_materialization_eligible: bool,
    pub official_joined_dataset_materialization_started: bool,
    pub official_joined_dataset_created: bool,
    pub independently_validated_after_materialization: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionItem {
    pub candidate: IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
    pub latest_review: Option<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview>,
    pub current_binding: bool,
    pub review_eligible: bool,
    pub join_target_candidate_admitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionItem>,
    pub independently_validated_candidate_count: usize,
    pub review_eligible_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_candidate_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_official_joined_dataset_materialization_eligible_count: usize,
    pub admission_status: String,
    pub candidate_admission_review_available: bool,
    pub official_joined_dataset_materialization_enabled: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReadinessSummary {
    pub independently_validated_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_candidate_count: usize,
    pub rejected_or_changes_requested_count: usize,
    pub future_official_joined_dataset_materialization_eligible_count: usize,
    pub admission_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate {
    pub candidate: IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
    pub admission_review: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_candidate_admission_reviews(
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
            warn!(%error, "join/target candidate admission registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果 join/target 候选准入复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_feature_label_join_target_candidate_admission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRequest>,
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

pub(crate) async fn historical_outcome_feature_label_join_target_candidate_admission_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReadinessSummary {
            independently_validated_candidate_count: registry
                .independently_validated_candidate_count,
            reviewed_candidate_count: registry.reviewed_candidate_count,
            admitted_candidate_count: registry.admitted_candidate_count,
            rejected_or_changes_requested_count: registry.changes_requested_or_rejected_count,
            future_official_joined_dataset_materialization_eligible_count: registry
                .future_official_joined_dataset_materialization_eligible_count,
            admission_status: registry.admission_status,
        },
    )
}

pub(crate) async fn admitted_join_target_candidates_for_official_joined_dataset_materialization(
    state: &AppState,
) -> Result<Vec<AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate>, String> {
    let candidates =
        independently_validated_join_target_candidates_for_admission_review(state).await?;
    let mut output = Vec::new();
    for candidate in candidates {
        let Some(review) = latest_review(state, &candidate).await? else {
            continue;
        };
        if review.join_target_candidate_admitted
            && review.future_create_once_official_joined_dataset_materialization_eligible
            && review.verdict == HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialJoinedDatasetMaterialization
        {
            validate_review(&review, &candidate)?;
            output.push(AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate { candidate, admission_review: review });
        }
    }
    output.sort_by(|left, right| {
        right
            .admission_review
            .submitted_at
            .cmp(&left.admission_review.submitted_at)
    });
    Ok(output)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn exact_admitted_join_target_candidate_for_official_joined_dataset_materialization(
    state: &AppState,
    attempt_id: &str,
    admission_review_id: &str,
    admission_review_sha256: &str,
    validation_sha256: &str,
    claim_sha256: &str,
    result_sha256: &str,
    output_sha256: &str,
    authorization_review_sha256: &str,
    isolated_runner_spec_sha256: &str,
    implementation_sha256: &str,
    specification_sha256: &str,
    join_specification_sha256: &str,
    target_specification_sha256: &str,
    split_manifest_sha256: &str,
    feature_bundle_sha256: &str,
    combined_artifact_sha256: &str,
    dataset_content_sha256: &str,
    dataset_manifest_sha256: &str,
    candidate_set_sha256: &str,
    recomputed_rows_sha256: &str,
    recomputed_excluded_rows_sha256: &str,
    recomputed_target_commitments_sha256: &str,
) -> Result<Option<AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate>, String> {
    let candidates =
        admitted_join_target_candidates_for_official_joined_dataset_materialization(state).await?;
    Ok(candidates.into_iter().find(|value| {
        let review = &value.admission_review;
        review.attempt_id == attempt_id
            && review.review_id == admission_review_id
            && review.review_sha256 == admission_review_sha256
            && review.validation_sha256 == validation_sha256
            && review.claim_sha256 == claim_sha256
            && review.result_sha256 == result_sha256
            && review.output_sha256 == output_sha256
            && review.authorization_review_sha256 == authorization_review_sha256
            && review.isolated_runner_spec_sha256 == isolated_runner_spec_sha256
            && review.implementation_sha256 == implementation_sha256
            && review.specification_sha256 == specification_sha256
            && review.join_specification_sha256 == join_specification_sha256
            && review.target_specification_sha256 == target_specification_sha256
            && review.split_manifest_sha256 == split_manifest_sha256
            && review.feature_bundle_sha256 == feature_bundle_sha256
            && review.combined_artifact_sha256 == combined_artifact_sha256
            && review.dataset_content_sha256 == dataset_content_sha256
            && review.dataset_manifest_sha256 == dataset_manifest_sha256
            && review.candidate_set_sha256 == candidate_set_sha256
            && review.recomputed_rows_sha256 == recomputed_rows_sha256
            && review.recomputed_excluded_rows_sha256 == recomputed_excluded_rows_sha256
            && review.recomputed_target_commitments_sha256 == recomputed_target_commitments_sha256
    }))
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry, String> {
    let candidates =
        independently_validated_join_target_candidates_for_admission_review(state).await?;
    let mut items = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let latest_review = latest_review(state, &candidate).await?;
        let admitted = latest_review
            .as_ref()
            .is_some_and(|review| review.join_target_candidate_admitted);
        let terminal = admitted;
        items.push(
            HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionItem {
                candidate,
                latest_review,
                current_binding: true,
                review_eligible: !terminal,
                join_target_candidate_admitted: admitted,
            },
        );
    }
    let independently_validated_candidate_count = items.len();
    let review_eligible_candidate_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_candidate_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let admitted_candidate_count = items
        .iter()
        .filter(|item| item.join_target_candidate_admitted)
        .count();
    let changes_requested_or_rejected_count = items.iter().filter(|item| item.latest_review.as_ref().is_some_and(|review| review.verdict != HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialJoinedDatasetMaterialization)).count();
    let future_official_joined_dataset_materialization_eligible_count = admitted_candidate_count;
    let admission_status = if admitted_candidate_count > 0 {
        "join_target_candidate_admitted_waiting_separate_create_once_official_joined_dataset_materialization"
    } else if changes_requested_or_rejected_count > 0 {
        "join_target_candidate_admission_changes_requested_or_rejected"
    } else if independently_validated_candidate_count > 0 {
        "waiting_independent_join_target_candidate_admission_review"
    } else {
        "waiting_independently_validated_join_target_candidate"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        independently_validated_candidate_count,
        review_eligible_candidate_count,
        reviewed_candidate_count,
        admitted_candidate_count,
        changes_requested_or_rejected_count,
        future_official_joined_dataset_materialization_eligible_count,
        admission_status: admission_status.to_string(),
        candidate_admission_review_available: true,
        official_joined_dataset_materialization_enabled: false,
        official_joined_dataset_created: false,
        copied_to_training_store: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 44 阶段由 Stage 43 校验者、Stage 42 执行者、完整上游链及此前准入复核人之外的独立管理员，复核精确候选的可用性、泄漏边界、九项目标承诺和正式数据集合同。批准只开放未来 create-once 正式 joined dataset 物化资格；本阶段不物化、不复制训练库、不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn review_admission(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewHistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview, String> {
    if !valid_id(attempt_id) {
        return Err("join/target 执行尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let candidate = exact_independently_validated_join_target_candidate_for_admission_review(
        state,
        attempt_id,
        &request.expected_validation_id,
        &request.expected_validation_sha256,
        &request.expected_claim_sha256,
        &request.expected_result_sha256,
        &request.expected_output_sha256,
        &request.expected_authorization_review_sha256,
        &request.expected_isolated_runner_spec_sha256,
        &request.expected_implementation_sha256,
        &request.expected_specification_sha256,
        &request.expected_join_specification_sha256,
        &request.expected_target_specification_sha256,
        &request.expected_split_manifest_sha256,
        &request.expected_feature_bundle_sha256,
        &request.expected_combined_artifact_sha256,
        &request.expected_dataset_content_sha256,
        &request.expected_dataset_manifest_sha256,
        &request.expected_candidate_set_sha256,
        &request.expected_recomputed_rows_sha256,
        &request.expected_recomputed_excluded_rows_sha256,
        &request.expected_recomputed_target_commitments_sha256,
    )
    .await?
    .ok_or_else(|| "当前没有这条精确绑定且独立校验通过的 join/target 候选".to_string())?;
    let latest = latest_review(state, &candidate).await?;
    if latest
        .as_ref()
        .is_some_and(|review| review.join_target_candidate_admitted)
    {
        return Err(
            "该候选已准入，复核链已经永久冻结；后续必须进入独立的 create-once 物化门禁".to_string(),
        );
    }
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("join/target 候选准入复核链已经变化，请刷新后重试".to_string());
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
        return Err(
            "候选准入复核人不能是 Stage 43 校验者、Stage 42 执行者、完整上游或此前准入复核角色"
                .to_string(),
        );
    }
    let envelope = candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "join/target 候选缺少不可信输出信封".to_string())?;
    let validation = &candidate.validation;
    let mut review = HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(), review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: validation.attempt_id.clone(), claim_sha256: validation.claim_sha256.clone(),
        result_id: validation.result_id.clone(), result_sha256: validation.result_sha256.clone(), output_sha256: validation.output_sha256.clone(),
        validation_id: validation.validation_id.clone(), validation_sha256: validation.validation_sha256.clone(),
        authorization_review_id: validation.authorization_review_id.clone(), authorization_review_sha256: validation.authorization_review_sha256.clone(),
        isolated_runner_id: validation.isolated_runner_id.clone(), isolated_runner_spec_sha256: validation.isolated_runner_spec_sha256.clone(),
        implementation_id: validation.implementation_id.clone(), implementation_sha256: validation.implementation_sha256.clone(),
        specification_id: validation.specification_id.clone(), specification_sha256: validation.specification_sha256.clone(),
        join_specification_sha256: validation.join_specification_sha256.clone(), target_specification_sha256: validation.target_specification_sha256.clone(),
        split_manifest_sha256: validation.split_manifest_sha256.clone(), feature_bundle_sha256: validation.feature_bundle_sha256.clone(), combined_artifact_sha256: validation.combined_artifact_sha256.clone(),
        dataset_id: validation.dataset_id.clone(), dataset_content_sha256: validation.dataset_content_sha256.clone(), dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(), candidate_set_sha256: validation.candidate_set_sha256.clone(),
        recomputed_rows_sha256: validation.recomputed_rows_sha256.clone(), recomputed_excluded_rows_sha256: validation.recomputed_excluded_rows_sha256.clone(), recomputed_target_commitments_sha256: validation.recomputed_target_commitments_sha256.clone(),
        dataset_entry_count: envelope.dataset_entry_count, active_candidate_row_count: envelope.active_candidate_row_count,
        excluded_purge_or_embargo_row_count: envelope.excluded_purge_or_embargo_row_count, feature_catalog_count: envelope.feature_catalog_count,
        target_count: envelope.target_count, train_target_vector_count: envelope.train_target_vector_count,
        validation_target_withheld_count: envelope.validation_target_withheld_count, sealed_holdout_target_withheld_count: envelope.sealed_holdout_target_withheld_count,
        submitted_at: Utc::now(), reviewer_id: reviewer_id.to_string(), excluded_prior_actor_ids, verdict: request.verdict,
        rationale, known_limitations,
        reviewer_independent_from_validator_executor_and_complete_prior_chain: independent,
        exact_current_candidate_validation_and_complete_chain_confirmed: request.exact_current_candidate_validation_and_complete_chain_confirmed,
        exact_one_to_one_entry_join_and_cardinality_confirmed: request.exact_one_to_one_entry_join_and_cardinality_confirmed,
        exact_65_feature_catalog_confirmed: request.exact_65_feature_catalog_confirmed,
        point_in_time_and_explicit_missingness_confirmed: request.point_in_time_and_explicit_missingness_confirmed,
        official_split_purge_and_embargo_confirmed: request.official_split_purge_and_embargo_confirmed,
        train_only_target_visibility_confirmed: request.train_only_target_visibility_confirmed,
        validation_targets_withheld_confirmed: request.validation_targets_withheld_confirmed,
        sealed_holdout_targets_withheld_confirmed: request.sealed_holdout_targets_withheld_confirmed,
        exact_nine_raw_f64_bits_and_commitments_confirmed: request.exact_nine_raw_f64_bits_and_commitments_confirmed,
        no_action_position_or_reward_semantics_confirmed: request.no_action_position_or_reward_semantics_confirmed,
        create_once_materialization_and_post_materialization_validation_separation_confirmed: request.create_once_materialization_and_post_materialization_validation_separation_confirmed,
        downstream_authority_remains_closed_confirmed: request.downstream_authority_remains_closed_confirmed,
        join_target_candidate_admitted: false, future_create_once_official_joined_dataset_materialization_eligible: false,
        official_joined_dataset_materialization_started: false, official_joined_dataset_created: false,
        independently_validated_after_materialization: false, copied_to_training_store: false,
        training_authorized: false, reward_authorized: false, shadow_portfolio_authorized: false,
        order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
    };
    let approved = review.verdict == HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialJoinedDatasetMaterialization;
    review.join_target_candidate_admitted = approved && review_all_checks(&review) && independent;
    review.future_create_once_official_joined_dataset_materialization_eligible =
        review.join_target_candidate_admitted;
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
    review: &HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview,
) -> bool {
    review.exact_current_candidate_validation_and_complete_chain_confirmed
        && review.exact_one_to_one_entry_join_and_cardinality_confirmed
        && review.exact_65_feature_catalog_confirmed
        && review.point_in_time_and_explicit_missingness_confirmed
        && review.official_split_purge_and_embargo_confirmed
        && review.train_only_target_visibility_confirmed
        && review.validation_targets_withheld_confirmed
        && review.sealed_holdout_targets_withheld_confirmed
        && review.exact_nine_raw_f64_bits_and_commitments_confirmed
        && review.no_action_position_or_reward_semantics_confirmed
        && review
            .create_once_materialization_and_post_materialization_validation_separation_confirmed
        && review.downstream_authority_remains_closed_confirmed
}

fn excluded_actor_ids(
    validation: &HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord,
    latest: &Option<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview>,
) -> Vec<String> {
    let mut actors = validation.excluded_prior_actor_ids.clone();
    actors.push(validation.validated_by.clone());
    actors.push(validation.execution_invoked_by.clone());
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
    review: &HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview,
    candidate: &IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
) -> Result<(), String> {
    let validation = &candidate.validation;
    let envelope = candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "validated join/target candidate envelope is missing".to_string())?;
    let approved = review.verdict == HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialJoinedDatasetMaterialization;
    let expected_independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_admitted = approved && review_all_checks(review) && expected_independent;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let downstream_closed = !review.official_joined_dataset_materialization_started
        && !review.official_joined_dataset_created
        && !review.independently_validated_after_materialization
        && !review.copied_to_training_store
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
        || review.attempt_id != validation.attempt_id
        || review.claim_sha256 != validation.claim_sha256
        || review.result_id != validation.result_id
        || review.result_sha256 != validation.result_sha256
        || review.output_sha256 != validation.output_sha256
        || review.validation_id != validation.validation_id
        || review.validation_sha256 != validation.validation_sha256
        || review.authorization_review_id != validation.authorization_review_id
        || review.authorization_review_sha256 != validation.authorization_review_sha256
        || review.isolated_runner_id != validation.isolated_runner_id
        || review.isolated_runner_spec_sha256 != validation.isolated_runner_spec_sha256
        || review.implementation_id != validation.implementation_id
        || review.implementation_sha256 != validation.implementation_sha256
        || review.specification_id != validation.specification_id
        || review.specification_sha256 != validation.specification_sha256
        || review.join_specification_sha256 != validation.join_specification_sha256
        || review.target_specification_sha256 != validation.target_specification_sha256
        || review.split_manifest_sha256 != validation.split_manifest_sha256
        || review.feature_bundle_sha256 != validation.feature_bundle_sha256
        || review.combined_artifact_sha256 != validation.combined_artifact_sha256
        || review.dataset_id != validation.dataset_id
        || review.dataset_content_sha256 != validation.dataset_content_sha256
        || review.dataset_manifest_sha256 != validation.dataset_manifest_sha256
        || review.candidate_set_sha256 != validation.candidate_set_sha256
        || review.recomputed_rows_sha256 != validation.recomputed_rows_sha256
        || review.recomputed_excluded_rows_sha256 != validation.recomputed_excluded_rows_sha256
        || review.recomputed_target_commitments_sha256
            != validation.recomputed_target_commitments_sha256
        || review.dataset_entry_count != envelope.dataset_entry_count
        || review.active_candidate_row_count != envelope.active_candidate_row_count
        || review.excluded_purge_or_embargo_row_count
            != envelope.excluded_purge_or_embargo_row_count
        || review.feature_catalog_count != envelope.feature_catalog_count
        || review.target_count != envelope.target_count
        || review.train_target_vector_count != envelope.train_target_vector_count
        || review.validation_target_withheld_count != envelope.validation_target_withheld_count
        || review.sealed_holdout_target_withheld_count
            != envelope.sealed_holdout_target_withheld_count
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
        || review.reviewer_independent_from_validator_executor_and_complete_prior_chain
            != expected_independent
        || (approved && !review_all_checks(review))
        || review.join_target_candidate_admitted != expected_admitted
        || review.future_create_once_official_joined_dataset_materialization_eligible
            != expected_admitted
        || !downstream_closed
    {
        return Err(
            "historical join/target candidate admission review is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn review_fingerprint(
    review: &HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    candidate: &IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
) -> Result<Option<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview>, String> {
    let directory = review_directory(state, &candidate.validation.attempt_id);
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
        let review: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, candidate)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("historical join/target candidate admission filename mismatch".to_string());
        }
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err(
                "historical join/target candidate admission contains duplicate records".to_string(),
            );
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview],
) -> Result<Option<HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview>, String> {
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
            "historical join/target candidate admission chain has invalid tips".to_string(),
        );
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("historical join/target candidate admission chain has a cycle".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id.get(id.as_str()).ok_or_else(|| {
                    "historical join/target candidate admission chain is disconnected".to_string()
                })?;
                if previous.review_sha256 != *sha {
                    return Err(
                        "historical join/target candidate admission previous hash mismatch"
                            .to_string(),
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "historical join/target candidate admission previous link is invalid"
                        .to_string(),
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err("historical join/target candidate admission chain is disconnected".to_string());
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
        .join("historical-outcome-feature-label-join-target-candidate-admission-reviews")
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
        "historical-join-target-candidate-admission-{attempt_id}.lock"
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
        .map_err(|_| "已有同一 join/target 候选准入复核正在进行".to_string())?;
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

    fn minimal_review() -> HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview {
        HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionReview {
            schema_version: REVIEW_SCHEMA_VERSION.into(), policy_version: POLICY_VERSION.into(), review_id: String::new(), review_sha256: String::new(), previous_review_id: None, previous_review_sha256: None,
            attempt_id: "1".repeat(32), claim_sha256: "2".repeat(64), result_id: "3".repeat(32), result_sha256: "4".repeat(64), output_sha256: "5".repeat(64), validation_id: "6".repeat(32), validation_sha256: "7".repeat(64),
            authorization_review_id: "8".repeat(32), authorization_review_sha256: "9".repeat(64), isolated_runner_id: "a".repeat(32), isolated_runner_spec_sha256: "b".repeat(64), implementation_id: "c".repeat(32), implementation_sha256: "d".repeat(64), specification_id: "e".repeat(32), specification_sha256: "f".repeat(64),
            join_specification_sha256: "1".repeat(64), target_specification_sha256: "2".repeat(64), split_manifest_sha256: "3".repeat(64), feature_bundle_sha256: "4".repeat(64), combined_artifact_sha256: "5".repeat(64), dataset_id: "6".repeat(32), dataset_content_sha256: "7".repeat(64), dataset_manifest_sha256: "8".repeat(64), candidate_set_sha256: "9".repeat(64), recomputed_rows_sha256: "a".repeat(64), recomputed_excluded_rows_sha256: "b".repeat(64), recomputed_target_commitments_sha256: "c".repeat(64),
            dataset_entry_count: 6, active_candidate_row_count: 3, excluded_purge_or_embargo_row_count: 3, feature_catalog_count: 65, target_count: 9, train_target_vector_count: 1, validation_target_withheld_count: 1, sealed_holdout_target_withheld_count: 1,
            submitted_at: Utc::now(), reviewer_id: "reviewer".into(), excluded_prior_actor_ids: vec!["executor".into(), "validator".into()],
            verdict: HistoricalOutcomeFeatureLabelJoinTargetCandidateAdmissionVerdict::ApprovedForFutureCreateOnceOfficialJoinedDatasetMaterialization,
            rationale: "逐项复核通过".into(), known_limitations: "样本量仍有限，正式数据集仍需独立物化和再校验".into(), reviewer_independent_from_validator_executor_and_complete_prior_chain: true,
            exact_current_candidate_validation_and_complete_chain_confirmed: true, exact_one_to_one_entry_join_and_cardinality_confirmed: true, exact_65_feature_catalog_confirmed: true, point_in_time_and_explicit_missingness_confirmed: true, official_split_purge_and_embargo_confirmed: true, train_only_target_visibility_confirmed: true, validation_targets_withheld_confirmed: true, sealed_holdout_targets_withheld_confirmed: true, exact_nine_raw_f64_bits_and_commitments_confirmed: true, no_action_position_or_reward_semantics_confirmed: true, create_once_materialization_and_post_materialization_validation_separation_confirmed: true, downstream_authority_remains_closed_confirmed: true,
            join_target_candidate_admitted: true, future_create_once_official_joined_dataset_materialization_eligible: true, official_joined_dataset_materialization_started: false, official_joined_dataset_created: false, independently_validated_after_materialization: false, copied_to_training_store: false, training_authorized: false, reward_authorized: false, shadow_portfolio_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        }
    }

    #[test]
    fn approval_requires_every_admission_check() {
        let mut review = minimal_review();
        assert!(review_all_checks(&review));
        review.validation_targets_withheld_confirmed = false;
        assert!(!review_all_checks(&review));
    }
    #[test]
    fn reviewer_must_be_outside_validator_executor_and_complete_chain() {
        let actors = vec!["executor".to_string(), "validator".to_string()];
        assert!(reviewer_is_independent("new-reviewer", &actors));
        assert!(!reviewer_is_independent("validator", &actors));
    }
    #[test]
    fn review_hash_binds_materialization_and_training_boundaries() {
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.copied_to_training_store = true;
        assert_ne!(first, review_fingerprint(&changed).unwrap());
    }
    #[test]
    fn review_hash_binds_exact_target_commitments() {
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.recomputed_target_commitments_sha256 = "d".repeat(64);
        assert_ne!(first, review_fingerprint(&changed).unwrap());
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
    #[test]
    fn required_text_is_trimmed_and_bounded() {
        assert_eq!(bounded_required("  通过  ", 8, "依据").unwrap(), "通过");
        assert!(bounded_required("   ", 8, "依据").is_err());
        assert!(bounded_required("123456789", 8, "依据").is_err());
    }
    #[test]
    fn excluded_actor_ids_must_be_strictly_sorted_and_unique() {
        assert!(sorted_unique(&["a".into(), "b".into()]));
        assert!(!sorted_unique(&["b".into(), "a".into()]));
        assert!(!sorted_unique(&["a".into(), "a".into()]));
    }
    #[test]
    fn admission_chain_rejects_disconnected_records() {
        let mut root = minimal_review();
        root.review_sha256 = review_fingerprint(&root).unwrap();
        root.review_id = root.review_sha256[..32].to_string();
        let mut orphan = root.clone();
        orphan.rationale = "孤立记录".into();
        orphan.review_sha256 = review_fingerprint(&orphan).unwrap();
        orphan.review_id = orphan.review_sha256[..32].to_string();
        assert!(latest_review_from_records(&[root, orphan]).is_err());
    }
    #[test]
    fn review_hash_binds_independence_and_actor_exclusion() {
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.excluded_prior_actor_ids.push("z-upstream".into());
        assert_ne!(first, review_fingerprint(&changed).unwrap());
    }
}
