//! Independent admission review before any official joined dataset may be copied
//! into a future training store.
//!
//! Approval is deliberately narrow: it only makes one exact, independently
//! validated Stage 46 dataset eligible for a separate create-once copy gate.
//! This module never copies data, starts training, defines a reward, creates a
//! shadow portfolio, generates an order, accesses a broker or trades.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_official_dataset_output_validations::{
    HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord,
    IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    independently_validated_official_joined_datasets_for_training_store_copy_admission_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-feature-label-join-target-training-store-copy-admission-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-store-copy-admission-review-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-training-store-copy-independent-admission-v1-no-copy";
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 2_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict {
    ApprovedForFutureCreateOnceTrainingStoreCopy,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_materialization_id: String,
    expected_materialization_claim_sha256: String,
    expected_materialization_result_sha256: String,
    expected_official_joined_dataset_sha256: String,
    expected_output_validation_id: String,
    expected_output_validation_sha256: String,
    expected_admission_review_sha256: String,
    expected_source_validation_sha256: String,
    expected_source_output_sha256: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_recomputed_rows_sha256: String,
    expected_recomputed_excluded_rows_sha256: String,
    expected_recomputed_target_commitments_sha256: String,
    verdict: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_stage_46_validation_and_complete_chain_confirmed: bool,
    immutable_official_dataset_fingerprint_confirmed: bool,
    exact_one_to_one_entry_join_and_cardinality_confirmed: bool,
    exact_65_feature_catalog_confirmed: bool,
    point_in_time_and_explicit_missingness_confirmed: bool,
    official_split_purge_and_embargo_confirmed: bool,
    exact_nine_raw_f64_bits_and_commitments_confirmed: bool,
    validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    schema_contract_suitable_for_future_copy_only_confirmed: bool,
    no_action_position_or_reward_semantics_confirmed: bool,
    create_once_copy_and_post_copy_validation_remain_separate_confirmed: bool,
    no_copy_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub materialization_id: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_id: String,
    pub materialization_result_sha256: String,
    pub official_joined_dataset_sha256: String,
    pub output_validation_id: String,
    pub output_validation_sha256: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub source_validation_id: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub recomputed_rows_sha256: String,
    pub recomputed_excluded_rows_sha256: String,
    pub recomputed_target_commitments_sha256: String,
    pub dataset_entry_count: usize,
    pub active_row_count: usize,
    pub excluded_purge_or_embargo_row_count: usize,
    pub feature_catalog_count: usize,
    pub target_count: usize,
    pub train_target_vector_count: usize,
    pub validation_target_withheld_count: usize,
    pub sealed_holdout_target_withheld_count: usize,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub reviewer_independent_from_stage_46_and_complete_prior_chain: bool,
    pub exact_current_stage_46_validation_and_complete_chain_confirmed: bool,
    pub immutable_official_dataset_fingerprint_confirmed: bool,
    pub exact_one_to_one_entry_join_and_cardinality_confirmed: bool,
    pub exact_65_feature_catalog_confirmed: bool,
    pub point_in_time_and_explicit_missingness_confirmed: bool,
    pub official_split_purge_and_embargo_confirmed: bool,
    pub exact_nine_raw_f64_bits_and_commitments_confirmed: bool,
    pub validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    pub schema_contract_suitable_for_future_copy_only_confirmed: bool,
    pub no_action_position_or_reward_semantics_confirmed: bool,
    pub create_once_copy_and_post_copy_validation_remain_separate_confirmed: bool,
    pub no_copy_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub training_store_copy_candidate_admitted: bool,
    pub future_create_once_training_store_copy_eligible: bool,
    pub training_store_copy_started: bool,
    pub copied_to_training_store: bool,
    pub independently_validated_after_training_store_copy: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionItem {
    pub dataset: IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    pub latest_review:
        Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview>,
    pub current_binding: bool,
    pub review_eligible: bool,
    pub training_store_copy_candidate_admitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionItem>,
    pub independently_validated_official_joined_dataset_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub admitted_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_create_once_training_store_copy_eligible_count: usize,
    pub admission_status: String,
    pub training_store_copy_admission_review_available: bool,
    pub training_store_copy_enabled: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReadinessSummary
{
    pub independently_validated_official_joined_dataset_count: usize,
    pub reviewed_count: usize,
    pub admitted_count: usize,
    pub rejected_or_changes_requested_count: usize,
    pub future_create_once_training_store_copy_eligible_count: usize,
    pub admission_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy
{
    pub dataset: IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    pub admission_review: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_training_store_copy_admission_reviews(
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
            warn!(%error, "training-store copy admission registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果训练存储复制准入复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_feature_label_join_target_training_store_copy_admission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<
        ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRequest,
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

pub(crate) async fn historical_outcome_feature_label_join_target_training_store_copy_admission_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReadinessSummary, String>
{
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReadinessSummary {
            independently_validated_official_joined_dataset_count: registry
                .independently_validated_official_joined_dataset_count,
            reviewed_count: registry.reviewed_count,
            admitted_count: registry.admitted_count,
            rejected_or_changes_requested_count: registry.changes_requested_or_rejected_count,
            future_create_once_training_store_copy_eligible_count: registry
                .future_create_once_training_store_copy_eligible_count,
            admission_status: registry.admission_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn admitted_official_joined_datasets_for_training_store_copy(
    state: &AppState,
) -> Result<
    Vec<AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy>,
    String,
> {
    let datasets =
        independently_validated_official_joined_datasets_for_training_store_copy_admission_review(
            state,
        )
        .await?;
    let mut output = Vec::new();
    for dataset in datasets {
        let Some(review) = latest_review(state, &dataset).await? else {
            continue;
        };
        if review.training_store_copy_candidate_admitted
            && review.future_create_once_training_store_copy_eligible
            && review.verdict
                == HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict::ApprovedForFutureCreateOnceTrainingStoreCopy
        {
            validate_review(&review, &dataset)?;
            output.push(
                AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy {
                    dataset,
                    admission_review: review,
                },
            );
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

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry, String> {
    let datasets =
        independently_validated_official_joined_datasets_for_training_store_copy_admission_review(
            state,
        )
        .await?;
    let mut items = Vec::with_capacity(datasets.len());
    for dataset in datasets {
        let latest_review = latest_review(state, &dataset).await?;
        let admitted = latest_review
            .as_ref()
            .is_some_and(|review| review.training_store_copy_candidate_admitted);
        items.push(
            HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionItem {
                dataset,
                latest_review,
                current_binding: true,
                review_eligible: !admitted,
                training_store_copy_candidate_admitted: admitted,
            },
        );
    }
    let independently_validated_official_joined_dataset_count = items.len();
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let admitted_count = items
        .iter()
        .filter(|item| item.training_store_copy_candidate_admitted)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    != HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict::ApprovedForFutureCreateOnceTrainingStoreCopy
            })
        })
        .count();
    let admission_status = if admitted_count > 0 {
        "training_store_copy_candidate_admitted_waiting_separate_create_once_copy"
    } else if changes_requested_or_rejected_count > 0 {
        "training_store_copy_admission_changes_requested_or_rejected"
    } else if independently_validated_official_joined_dataset_count > 0 {
        "waiting_independent_training_store_copy_admission_review"
    } else {
        "waiting_independently_validated_official_joined_dataset"
    };
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            items,
            independently_validated_official_joined_dataset_count,
            review_eligible_count,
            reviewed_count,
            admitted_count,
            changes_requested_or_rejected_count,
            future_create_once_training_store_copy_eligible_count: admitted_count,
            admission_status: admission_status.to_string(),
            training_store_copy_admission_review_available: true,
            training_store_copy_enabled: false,
            copied_to_training_store: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "第 47 阶段由 Stage 46 校验者、Stage 45 物化者及完整上游之外的独立管理员，复核精确正式 joined dataset 的数据合同、泄漏边界、九项目标承诺和未来复制边界。批准只开放另一个 create-once 训练存储复制门禁；本阶段不复制、不训练、奖励、影子、订单、券商或交易。".to_string(),
        },
    )
}

async fn review_admission(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview, String> {
    if !valid_id(attempt_id) {
        return Err("join/target 执行尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let dataset =
        independently_validated_official_joined_datasets_for_training_store_copy_admission_review(
            state,
        )
        .await?
        .into_iter()
        .find(|value| exact_request_binding(value, attempt_id, &request))
        .ok_or_else(|| "当前没有这条精确绑定且独立校验通过的正式 joined dataset".to_string())?;
    let latest = latest_review(state, &dataset).await?;
    if latest
        .as_ref()
        .is_some_and(|review| review.training_store_copy_candidate_admitted)
    {
        return Err(
            "该正式数据集已准入，复核链已经永久冻结；后续必须进入独立 create-once 复制门禁"
                .to_string(),
        );
    }
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("训练存储复制准入复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限与偏差",
    )?;
    let excluded_prior_actor_ids = excluded_actor_ids(&dataset.validation, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "复制准入复核人不能是 Stage 46 校验者、Stage 45 物化者、完整上游或此前复核角色"
                .to_string(),
        );
    }
    let validation = &dataset.validation;
    let materialization = &dataset.materialization;
    let official = &materialization.official_joined_dataset;
    let mut review = HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: validation.attempt_id.clone(),
        materialization_id: validation.materialization_id.clone(),
        materialization_claim_sha256: validation.materialization_claim_sha256.clone(),
        materialization_result_id: validation.materialization_result_id.clone(),
        materialization_result_sha256: validation.materialization_result_sha256.clone(),
        official_joined_dataset_sha256: validation.official_joined_dataset_sha256.clone(),
        output_validation_id: validation.validation_id.clone(),
        output_validation_sha256: validation.validation_sha256.clone(),
        admission_review_id: validation.admission_review_id.clone(),
        admission_review_sha256: validation.admission_review_sha256.clone(),
        source_validation_id: validation.source_validation_id.clone(),
        source_validation_sha256: validation.source_validation_sha256.clone(),
        source_output_sha256: validation.source_output_sha256.clone(),
        dataset_id: validation.dataset_id.clone(),
        dataset_content_sha256: validation.dataset_content_sha256.clone(),
        dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(),
        candidate_set_sha256: validation.candidate_set_sha256.clone(),
        recomputed_rows_sha256: validation.recomputed_rows_sha256.clone(),
        recomputed_excluded_rows_sha256: validation.recomputed_excluded_rows_sha256.clone(),
        recomputed_target_commitments_sha256: validation
            .recomputed_target_commitments_sha256
            .clone(),
        dataset_entry_count: official.dataset_entry_count,
        active_row_count: official.active_row_count,
        excluded_purge_or_embargo_row_count: official.excluded_purge_or_embargo_row_count,
        feature_catalog_count: official.feature_catalog_count,
        target_count: official.target_count,
        train_target_vector_count: official.train_target_vector_count,
        validation_target_withheld_count: official.validation_target_withheld_count,
        sealed_holdout_target_withheld_count: official.sealed_holdout_target_withheld_count,
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        known_limitations,
        reviewer_independent_from_stage_46_and_complete_prior_chain: independent,
        exact_current_stage_46_validation_and_complete_chain_confirmed: request
            .exact_current_stage_46_validation_and_complete_chain_confirmed,
        immutable_official_dataset_fingerprint_confirmed: request
            .immutable_official_dataset_fingerprint_confirmed,
        exact_one_to_one_entry_join_and_cardinality_confirmed: request
            .exact_one_to_one_entry_join_and_cardinality_confirmed,
        exact_65_feature_catalog_confirmed: request.exact_65_feature_catalog_confirmed,
        point_in_time_and_explicit_missingness_confirmed: request
            .point_in_time_and_explicit_missingness_confirmed,
        official_split_purge_and_embargo_confirmed: request
            .official_split_purge_and_embargo_confirmed,
        exact_nine_raw_f64_bits_and_commitments_confirmed: request
            .exact_nine_raw_f64_bits_and_commitments_confirmed,
        validation_and_sealed_holdout_targets_remain_withheld_confirmed: request
            .validation_and_sealed_holdout_targets_remain_withheld_confirmed,
        schema_contract_suitable_for_future_copy_only_confirmed: request
            .schema_contract_suitable_for_future_copy_only_confirmed,
        no_action_position_or_reward_semantics_confirmed: request
            .no_action_position_or_reward_semantics_confirmed,
        create_once_copy_and_post_copy_validation_remain_separate_confirmed: request
            .create_once_copy_and_post_copy_validation_remain_separate_confirmed,
        no_copy_training_reward_shadow_order_broker_or_trading_confirmed: request
            .no_copy_training_reward_shadow_order_broker_or_trading_confirmed,
        training_store_copy_candidate_admitted: false,
        future_create_once_training_store_copy_eligible: false,
        training_store_copy_started: false,
        copied_to_training_store: false,
        independently_validated_after_training_store_copy: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict::ApprovedForFutureCreateOnceTrainingStoreCopy;
    review.training_store_copy_candidate_admitted =
        approved && review_all_checks(&review) && independent;
    review.future_create_once_training_store_copy_eligible =
        review.training_store_copy_candidate_admitted;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &dataset)?;
    write_immutable_json(
        &review_directory(state, attempt_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn exact_request_binding(
    dataset: &IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    attempt_id: &str,
    request: &ReviewHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionRequest,
) -> bool {
    let validation = &dataset.validation;
    validation.attempt_id == attempt_id
        && validation.materialization_id == request.expected_materialization_id
        && validation.materialization_claim_sha256 == request.expected_materialization_claim_sha256
        && validation.materialization_result_sha256
            == request.expected_materialization_result_sha256
        && validation.official_joined_dataset_sha256
            == request.expected_official_joined_dataset_sha256
        && validation.validation_id == request.expected_output_validation_id
        && validation.validation_sha256 == request.expected_output_validation_sha256
        && validation.admission_review_sha256 == request.expected_admission_review_sha256
        && validation.source_validation_sha256 == request.expected_source_validation_sha256
        && validation.source_output_sha256 == request.expected_source_output_sha256
        && validation.dataset_content_sha256 == request.expected_dataset_content_sha256
        && validation.dataset_manifest_sha256 == request.expected_dataset_manifest_sha256
        && validation.candidate_set_sha256 == request.expected_candidate_set_sha256
        && validation.recomputed_rows_sha256 == request.expected_recomputed_rows_sha256
        && validation.recomputed_excluded_rows_sha256
            == request.expected_recomputed_excluded_rows_sha256
        && validation.recomputed_target_commitments_sha256
            == request.expected_recomputed_target_commitments_sha256
}

fn review_all_checks(
    review: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview,
) -> bool {
    review.exact_current_stage_46_validation_and_complete_chain_confirmed
        && review.immutable_official_dataset_fingerprint_confirmed
        && review.exact_one_to_one_entry_join_and_cardinality_confirmed
        && review.exact_65_feature_catalog_confirmed
        && review.point_in_time_and_explicit_missingness_confirmed
        && review.official_split_purge_and_embargo_confirmed
        && review.exact_nine_raw_f64_bits_and_commitments_confirmed
        && review.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        && review.schema_contract_suitable_for_future_copy_only_confirmed
        && review.no_action_position_or_reward_semantics_confirmed
        && review.create_once_copy_and_post_copy_validation_remain_separate_confirmed
        && review.no_copy_training_reward_shadow_order_broker_or_trading_confirmed
}

fn excluded_actor_ids(
    validation: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord,
    latest: &Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview>,
) -> Vec<String> {
    let mut actors = validation.excluded_prior_actor_ids.clone();
    actors.push(validation.validated_by.clone());
    actors.push(validation.materialized_by.clone());
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
    review: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview,
    dataset: &IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
) -> Result<(), String> {
    let validation = &dataset.validation;
    let official = &dataset.materialization.official_joined_dataset;
    let approved = review.verdict
        == HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict::ApprovedForFutureCreateOnceTrainingStoreCopy;
    let expected_independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_admitted = approved && review_all_checks(review) && expected_independent;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let downstream_closed = !review.training_store_copy_started
        && !review.copied_to_training_store
        && !review.independently_validated_after_training_store_copy
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
        || review.materialization_id != validation.materialization_id
        || review.materialization_claim_sha256 != validation.materialization_claim_sha256
        || review.materialization_result_id != validation.materialization_result_id
        || review.materialization_result_sha256 != validation.materialization_result_sha256
        || review.official_joined_dataset_sha256 != validation.official_joined_dataset_sha256
        || review.output_validation_id != validation.validation_id
        || review.output_validation_sha256 != validation.validation_sha256
        || review.admission_review_id != validation.admission_review_id
        || review.admission_review_sha256 != validation.admission_review_sha256
        || review.source_validation_id != validation.source_validation_id
        || review.source_validation_sha256 != validation.source_validation_sha256
        || review.source_output_sha256 != validation.source_output_sha256
        || review.dataset_id != validation.dataset_id
        || review.dataset_content_sha256 != validation.dataset_content_sha256
        || review.dataset_manifest_sha256 != validation.dataset_manifest_sha256
        || review.candidate_set_sha256 != validation.candidate_set_sha256
        || review.recomputed_rows_sha256 != validation.recomputed_rows_sha256
        || review.recomputed_excluded_rows_sha256 != validation.recomputed_excluded_rows_sha256
        || review.recomputed_target_commitments_sha256
            != validation.recomputed_target_commitments_sha256
        || review.dataset_entry_count != official.dataset_entry_count
        || review.active_row_count != official.active_row_count
        || review.excluded_purge_or_embargo_row_count
            != official.excluded_purge_or_embargo_row_count
        || review.feature_catalog_count != official.feature_catalog_count
        || review.target_count != official.target_count
        || review.train_target_vector_count != official.train_target_vector_count
        || review.validation_target_withheld_count != official.validation_target_withheld_count
        || review.sealed_holdout_target_withheld_count
            != official.sealed_holdout_target_withheld_count
        || review.feature_catalog_count != 65
        || review.target_count != 9
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
        || review.reviewer_independent_from_stage_46_and_complete_prior_chain
            != expected_independent
        || (approved && !review_all_checks(review))
        || review.training_store_copy_candidate_admitted != expected_admitted
        || review.future_create_once_training_store_copy_eligible != expected_admitted
        || !downstream_closed
    {
        return Err(
            "historical training-store copy admission review is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn review_fingerprint(
    review: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    dataset: &IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
) -> Result<Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview>, String>
{
    let directory = review_directory(state, &dataset.validation.attempt_id);
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
        let review: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, dataset)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("historical training-store copy admission filename mismatch".to_string());
        }
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err(
                "historical training-store copy admission contains duplicate records".to_string(),
            );
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview],
) -> Result<Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview>, String>
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
    if tips.len() != 1 {
        return Err("historical training-store copy admission chain has invalid tips".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("historical training-store copy admission chain has a cycle".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id.get(id.as_str()).ok_or_else(|| {
                    "historical training-store copy admission chain is disconnected".to_string()
                })?;
                if previous.review_sha256 != *sha {
                    return Err(
                        "historical training-store copy admission previous hash mismatch"
                            .to_string(),
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "historical training-store copy admission previous link is invalid".to_string(),
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err("historical training-store copy admission chain is disconnected".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "copy admission fingerprint payload is invalid".to_string())?;
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
        .join("historical-outcome-feature-label-join-target-training-store-copy-admission-reviews")
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
        "historical-join-target-training-store-copy-admission-{attempt_id}.lock"
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
        .map_err(|_| "已有同一训练存储复制准入复核正在进行".to_string())?;
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

    fn minimal_review() -> HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview {
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionReview {
            schema_version: REVIEW_SCHEMA_VERSION.into(), policy_version: POLICY_VERSION.into(), review_id: String::new(), review_sha256: String::new(), previous_review_id: None, previous_review_sha256: None,
            attempt_id: "1".repeat(32), materialization_id: "2".repeat(32), materialization_claim_sha256: "3".repeat(64), materialization_result_id: "4".repeat(32), materialization_result_sha256: "5".repeat(64), official_joined_dataset_sha256: "6".repeat(64),
            output_validation_id: "7".repeat(32), output_validation_sha256: "8".repeat(64), admission_review_id: "9".repeat(32), admission_review_sha256: "a".repeat(64), source_validation_id: "b".repeat(32), source_validation_sha256: "c".repeat(64), source_output_sha256: "d".repeat(64),
            dataset_id: "e".repeat(32), dataset_content_sha256: "f".repeat(64), dataset_manifest_sha256: "1".repeat(64), candidate_set_sha256: "2".repeat(64), recomputed_rows_sha256: "3".repeat(64), recomputed_excluded_rows_sha256: "4".repeat(64), recomputed_target_commitments_sha256: "5".repeat(64),
            dataset_entry_count: 6, active_row_count: 3, excluded_purge_or_embargo_row_count: 3, feature_catalog_count: 65, target_count: 9, train_target_vector_count: 1, validation_target_withheld_count: 1, sealed_holdout_target_withheld_count: 1,
            submitted_at: Utc::now(), reviewer_id: "reviewer".into(), excluded_prior_actor_ids: vec!["materializer".into(), "validator".into()], verdict: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAdmissionVerdict::ApprovedForFutureCreateOnceTrainingStoreCopy,
            rationale: "逐项复核通过".into(), known_limitations: "只确认数据合同可进入未来复制门禁，不确认模型质量或策略收益".into(), reviewer_independent_from_stage_46_and_complete_prior_chain: true,
            exact_current_stage_46_validation_and_complete_chain_confirmed: true, immutable_official_dataset_fingerprint_confirmed: true, exact_one_to_one_entry_join_and_cardinality_confirmed: true, exact_65_feature_catalog_confirmed: true, point_in_time_and_explicit_missingness_confirmed: true, official_split_purge_and_embargo_confirmed: true, exact_nine_raw_f64_bits_and_commitments_confirmed: true, validation_and_sealed_holdout_targets_remain_withheld_confirmed: true, schema_contract_suitable_for_future_copy_only_confirmed: true, no_action_position_or_reward_semantics_confirmed: true, create_once_copy_and_post_copy_validation_remain_separate_confirmed: true, no_copy_training_reward_shadow_order_broker_or_trading_confirmed: true,
            training_store_copy_candidate_admitted: true, future_create_once_training_store_copy_eligible: true, training_store_copy_started: false, copied_to_training_store: false, independently_validated_after_training_store_copy: false, training_authorized: false, reward_authorized: false, shadow_portfolio_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        }
    }

    #[test]
    fn approval_requires_every_copy_admission_check() {
        let mut review = minimal_review();
        assert!(review_all_checks(&review));
        review.schema_contract_suitable_for_future_copy_only_confirmed = false;
        assert!(!review_all_checks(&review));
    }

    #[test]
    fn reviewer_must_be_outside_stage_46_and_complete_chain() {
        let actors = vec!["materializer".to_string(), "validator".to_string()];
        assert!(reviewer_is_independent("new-reviewer", &actors));
        assert!(!reviewer_is_independent("validator", &actors));
    }

    #[test]
    fn review_hash_binds_exact_validation_and_dataset() {
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.output_validation_sha256 = "9".repeat(64);
        assert_ne!(first, review_fingerprint(&changed).unwrap());
    }

    #[test]
    fn review_hash_binds_target_commitments_and_copy_boundary() {
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.copied_to_training_store = true;
        changed.recomputed_target_commitments_sha256 = "a".repeat(64);
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
    fn required_text_is_trimmed_and_bounded() {
        assert_eq!(bounded_required("  通过  ", 8, "依据").unwrap(), "通过");
        assert!(bounded_required("   ", 8, "依据").is_err());
        assert!(bounded_required("123456789", 8, "依据").is_err());
    }

    #[test]
    fn actor_exclusion_must_be_sorted_unique_and_is_hash_bound() {
        assert!(sorted_unique(&["a".into(), "b".into()]));
        assert!(!sorted_unique(&["b".into(), "a".into()]));
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.excluded_prior_actor_ids.push("z-upstream".into());
        assert_ne!(first, review_fingerprint(&changed).unwrap());
    }

    #[test]
    fn approval_never_implies_copy_training_or_trading() {
        let review = minimal_review();
        assert!(review.training_store_copy_candidate_admitted);
        assert!(review.future_create_once_training_store_copy_eligible);
        assert!(!review.training_store_copy_started);
        assert!(!review.copied_to_training_store);
        assert!(!review.training_authorized);
        assert!(!review.reward_authorized);
        assert!(!review.shadow_portfolio_authorized);
        assert!(!review.order_generation_authorized);
        assert!(!review.broker_access_authorized);
        assert!(!review.trading_authorized);
    }
}
