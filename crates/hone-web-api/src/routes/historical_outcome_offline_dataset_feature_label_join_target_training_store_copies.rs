//! Claim-first, create-once copy of one exact Stage 47 admitted official joined
//! dataset into an isolated training store.
//!
//! The claim is persisted before any bytes are copied. Success, failure or
//! interruption consumes the admission. A successful copy remains unusable
//! until a later independent post-copy validator reopens the source and copy.
//! This module does not register or run training, define reward, create a shadow
//! portfolio, generate orders, access a broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_execution_attempts::{
    HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit,
    HistoricalOutcomeFeatureLabelJoinTargetVisibility,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_official_dataset_materializations::HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_admission_reviews::{
    AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
    admitted_official_joined_datasets_for_training_store_copy,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-store-copy-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-store-copy-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-store-copy-result-v1";
const TRAINING_STORE_DATASET_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-isolated-training-store-dataset-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-training-store-claim-first-create-once-exact-copy-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const MAX_TRAINING_STORE_DATASET_BYTES: usize = 64 * 1024 * 1024;
const MAX_ERROR_CHARS: usize = 1_200;

#[derive(Debug, Deserialize)]
pub(crate) struct CopyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreRequest {
    expected_admission_review_id: String,
    expected_admission_review_sha256: String,
    expected_output_validation_id: String,
    expected_output_validation_sha256: String,
    expected_materialization_id: String,
    expected_materialization_claim_sha256: String,
    expected_materialization_result_sha256: String,
    expected_official_joined_dataset_sha256: String,
    expected_source_validation_sha256: String,
    expected_source_output_sha256: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_rows_sha256: String,
    expected_excluded_rows_sha256: String,
    expected_target_commitments_sha256: String,
    exact_current_stage_47_admission_and_complete_chain_confirmed: bool,
    claim_first_create_once_and_failure_consumes_confirmed: bool,
    exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed: bool,
    validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    independent_post_copy_validation_required_confirmed: bool,
    no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim {
    pub schema_version: String,
    pub policy_version: String,
    pub copy_id: String,
    pub claim_sha256: String,
    pub attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub output_validation_id: String,
    pub output_validation_sha256: String,
    pub materialization_id: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_sha256: String,
    pub official_joined_dataset_sha256: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub expected_rows_sha256: String,
    pub expected_excluded_rows_sha256: String,
    pub expected_target_commitments_sha256: String,
    pub expected_dataset_entry_count: usize,
    pub expected_active_row_count: usize,
    pub expected_excluded_row_count: usize,
    pub expected_feature_catalog_count: usize,
    pub expected_target_count: usize,
    pub expected_train_target_vector_count: usize,
    pub expected_validation_target_withheld_count: usize,
    pub expected_sealed_holdout_target_withheld_count: usize,
    pub copied_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub claimed_at: DateTime<Utc>,
    pub exact_current_stage_47_admission_and_complete_chain_confirmed: bool,
    pub copier_independent_from_complete_prior_chain: bool,
    pub claim_first_create_once_and_failure_consumes_confirmed: bool,
    pub exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed: bool,
    pub validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    pub independent_post_copy_validation_required_confirmed: bool,
    pub no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub claim_consumed: bool,
    pub training_store_copy_started: bool,
    pub exact_target_directory_write_allowed: bool,
    pub generic_training_store_read_allowed: bool,
    pub generic_training_store_write_allowed: bool,
    pub ambient_environment_available: bool,
    pub network_available: bool,
    pub external_tools_available: bool,
    pub production_state_write_available: bool,
    pub historical_state_mutation_allowed: bool,
    pub training_registration_allowed: bool,
    pub training_run_allowed: bool,
    pub reward_allowed: bool,
    pub shadow_portfolio_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset {
    pub schema_version: String,
    pub training_store_dataset_sha256: String,
    pub copy_id: String,
    pub attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub output_validation_id: String,
    pub output_validation_sha256: String,
    pub materialization_id: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_sha256: String,
    pub official_joined_dataset_sha256: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub source_rows_sha256: String,
    pub source_excluded_rows_sha256: String,
    pub source_target_commitments_sha256: String,
    pub dataset_entry_count: usize,
    pub active_row_count: usize,
    pub excluded_purge_or_embargo_row_count: usize,
    pub feature_catalog_count: usize,
    pub target_count: usize,
    pub train_target_vector_count: usize,
    pub validation_target_withheld_count: usize,
    pub sealed_holdout_target_withheld_count: usize,
    pub rows: Vec<HistoricalOutcomeFeatureLabelJoinTargetCandidateRow>,
    pub excluded_rows: Vec<HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit>,
    pub exact_official_dataset_copy: bool,
    pub one_to_one_join_preserved: bool,
    pub point_in_time_and_explicit_missingness_preserved: bool,
    pub official_split_purge_and_embargo_preserved: bool,
    pub exact_raw_f64_bits_preserved: bool,
    pub validation_targets_withheld: bool,
    pub sealed_holdout_targets_withheld: bool,
    pub action_position_and_reward_semantics_absent: bool,
    pub copied_to_training_store: bool,
    pub independently_validated_after_training_store_copy: bool,
    pub eligible_for_training_registration_review: bool,
    pub training_registered: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus {
    CompletedPendingIndependentValidation,
    FailedClaimConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult {
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub copy_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub status: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus,
    pub error: Option<String>,
    pub training_store_dataset_sha256: Option<String>,
    pub training_store_dataset_bytes: usize,
    pub copied_to_training_store: bool,
    pub exact_official_dataset_copy_completed: bool,
    pub independent_post_copy_validation_completed: bool,
    pub eligible_for_training_registration_review: bool,
    pub training_registered: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt {
    pub claim: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    pub result: Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult>,
    pub training_store_dataset: Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyItem {
    pub admitted_dataset:
        AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
    pub attempt: Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt>,
    pub copy_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyItem>,
    pub admitted_dataset_count: usize,
    pub copy_eligible_count: usize,
    pub claim_count: usize,
    pub completed_copy_count: usize,
    pub failed_copy_count: usize,
    pub pending_independent_post_copy_validation_count: usize,
    pub copy_status: String,
    pub create_once_copy_available: bool,
    pub copied_to_training_store: bool,
    pub independently_validated_after_training_store_copy: bool,
    pub training_registration_available: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyReadinessSummary {
    pub admitted_dataset_count: usize,
    pub copy_eligible_count: usize,
    pub claim_count: usize,
    pub completed_copy_count: usize,
    pub failed_copy_count: usize,
    pub pending_independent_post_copy_validation_count: usize,
    pub copy_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset {
    pub admitted_dataset:
        AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
    pub attempt: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_training_store_copies(
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
            warn!(%error, "training-store copy registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果训练存储复制暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_copy_historical_outcome_feature_label_join_target_training_store_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<CopyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match copy_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_feature_label_join_target_training_store_copy_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyReadinessSummary {
            admitted_dataset_count: registry.admitted_dataset_count,
            copy_eligible_count: registry.copy_eligible_count,
            claim_count: registry.claim_count,
            completed_copy_count: registry.completed_copy_count,
            failed_copy_count: registry.failed_copy_count,
            pending_independent_post_copy_validation_count: registry
                .pending_independent_post_copy_validation_count,
            copy_status: registry.copy_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn copied_training_store_datasets_for_independent_validation(
    state: &AppState,
) -> Result<Vec<CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset>, String> {
    let admitted = admitted_official_joined_datasets_for_training_store_copy(state).await?;
    let mut values = Vec::new();
    for dataset in admitted {
        let Some(attempt) = read_attempt(state, &dataset).await? else {
            continue;
        };
        if attempt_completed(&attempt) {
            values.push(
                CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset {
                    admitted_dataset: dataset,
                    attempt,
                },
            );
        }
    }
    values.sort_by(|left, right| {
        right
            .attempt
            .claim
            .claimed_at
            .cmp(&left.attempt.claim.claimed_at)
    });
    Ok(values)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry, String> {
    let admitted = admitted_official_joined_datasets_for_training_store_copy(state).await?;
    let mut items = Vec::with_capacity(admitted.len());
    for dataset in admitted {
        let attempt = read_attempt(state, &dataset).await?;
        items.push(
            HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyItem {
                copy_eligible: attempt.is_none(),
                admitted_dataset: dataset,
                attempt,
            },
        );
    }
    let admitted_dataset_count = items.len();
    let copy_eligible_count = items.iter().filter(|item| item.copy_eligible).count();
    let claim_count = items.iter().filter(|item| item.attempt.is_some()).count();
    let completed_copy_count = items
        .iter()
        .filter(|item| item.attempt.as_ref().is_some_and(attempt_completed))
        .count();
    let failed_copy_count = items
        .iter()
        .filter(|item| {
            item.attempt
                .as_ref()
                .and_then(|attempt| attempt.result.as_ref())
                .is_some_and(|result| {
                    result.status
                        == HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus::FailedClaimConsumed
                })
        })
        .count();
    let pending_independent_post_copy_validation_count = completed_copy_count;
    let copy_status = if failed_copy_count > 0 {
        "training_store_copy_failed_claim_consumed"
    } else if pending_independent_post_copy_validation_count > 0 {
        "copied_to_training_store_waiting_independent_post_copy_validation"
    } else if claim_count > 0 {
        "training_store_copy_claim_consumed_waiting_result"
    } else if copy_eligible_count > 0 {
        "waiting_create_once_training_store_copy"
    } else {
        "waiting_independently_admitted_official_joined_dataset"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        admitted_dataset_count,
        copy_eligible_count,
        claim_count,
        completed_copy_count,
        failed_copy_count,
        pending_independent_post_copy_validation_count,
        copy_status: copy_status.to_string(),
        create_once_copy_available: true,
        copied_to_training_store: completed_copy_count > 0,
        independently_validated_after_training_store_copy: false,
        training_registration_available: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 48 阶段只允许独立角色对一条精确 Stage 47 已准入正式 joined dataset 先 claim，再 create-once 原样复制到隔离训练存储；失败也消费资格。复制后仍需另一实现独立逐行逐位校验，当前不能登记或运行训练，也不开放奖励、影子、订单、券商或交易。".to_string(),
    })
}

fn attempt_completed(
    attempt: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt,
) -> bool {
    attempt.result.as_ref().is_some_and(|result| {
        result.status
            == HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus::CompletedPendingIndependentValidation
            && result.copied_to_training_store
            && !result.independent_post_copy_validation_completed
            && attempt.training_store_dataset.is_some()
    })
}

async fn copy_once(
    state: &AppState,
    copied_by: &str,
    attempt_id: &str,
    request: CopyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult, String> {
    if !valid_id(attempt_id) {
        return Err("join/target 执行尝试 ID 无效".to_string());
    }
    if !request.exact_current_stage_47_admission_and_complete_chain_confirmed
        || !request.claim_first_create_once_and_failure_consumes_confirmed
        || !request.exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed
        || !request.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        || !request.independent_post_copy_validation_required_confirmed
        || !request.no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err(
            "必须确认当前准入链、一次性失败消费、精确复制、留出目标隐藏、复制后独立校验和零下游权限六项边界"
                .to_string(),
        );
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let admitted = admitted_official_joined_datasets_for_training_store_copy(state)
        .await?
        .into_iter()
        .find(|value| exact_request_binding(value, attempt_id, &request))
        .ok_or_else(|| "当前没有这条精确绑定且已独立准入的正式 joined dataset".to_string())?;
    if read_attempt(state, &admitted).await?.is_some() {
        return Err("这条正式数据集已经领取过一次性训练存储复制 claim，不能重放".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&admitted);
    if !actor_is_independent(copied_by, &excluded_prior_actor_ids) {
        return Err(
            "训练存储复制人不能是 Stage 47 准入、Stage 46 校验、Stage 45 物化或完整上游角色"
                .to_string(),
        );
    }
    let claim = new_claim(
        &admitted,
        copied_by,
        excluded_prior_actor_ids,
        &request,
        Utc::now(),
    )?;
    let directory = copy_directory(state, attempt_id);
    write_immutable_json(&directory.join("claim.json"), &claim).await?;

    let built = build_training_store_dataset(&claim, &admitted);
    let result = match built {
        Ok(dataset) => {
            validate_training_store_dataset(
                &dataset,
                &claim,
                &admitted.dataset.materialization.official_joined_dataset,
            )?;
            let bytes = serde_json::to_vec(&dataset).map_err(|error| error.to_string())?;
            if bytes.len() > MAX_TRAINING_STORE_DATASET_BYTES {
                new_failed_result(&claim, "训练存储数据集超过固定 64 MiB 上限", bytes.len())?
            } else {
                let artifact_path = directory.join(format!(
                    "training-store-dataset-{}.json",
                    dataset.training_store_dataset_sha256
                ));
                match write_immutable_json(&artifact_path, &dataset).await {
                    Ok(()) => new_success_result(&claim, &dataset, bytes.len())?,
                    Err(error) => new_failed_result(
                        &claim,
                        &format!("训练存储数据集写入失败：{error}"),
                        bytes.len(),
                    )?,
                }
            }
        }
        Err(error) => new_failed_result(&claim, &error, 0)?,
    };
    write_immutable_json(&directory.join("result.json"), &result).await?;
    Ok(result)
}

fn exact_request_binding(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
    attempt_id: &str,
    request: &CopyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreRequest,
) -> bool {
    let review = &admitted.admission_review;
    let validation = &admitted.dataset.validation;
    review.attempt_id == attempt_id
        && review.review_id == request.expected_admission_review_id
        && review.review_sha256 == request.expected_admission_review_sha256
        && validation.validation_id == request.expected_output_validation_id
        && validation.validation_sha256 == request.expected_output_validation_sha256
        && validation.materialization_id == request.expected_materialization_id
        && validation.materialization_claim_sha256 == request.expected_materialization_claim_sha256
        && validation.materialization_result_sha256
            == request.expected_materialization_result_sha256
        && validation.official_joined_dataset_sha256
            == request.expected_official_joined_dataset_sha256
        && validation.source_validation_sha256 == request.expected_source_validation_sha256
        && validation.source_output_sha256 == request.expected_source_output_sha256
        && validation.dataset_content_sha256 == request.expected_dataset_content_sha256
        && validation.dataset_manifest_sha256 == request.expected_dataset_manifest_sha256
        && validation.candidate_set_sha256 == request.expected_candidate_set_sha256
        && validation.recomputed_rows_sha256 == request.expected_rows_sha256
        && validation.recomputed_excluded_rows_sha256 == request.expected_excluded_rows_sha256
        && validation.recomputed_target_commitments_sha256
            == request.expected_target_commitments_sha256
}

fn new_claim(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
    copied_by: &str,
    excluded_prior_actor_ids: Vec<String>,
    request: &CopyHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreRequest,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim, String> {
    let review = &admitted.admission_review;
    let validation = &admitted.dataset.validation;
    let official = &admitted.dataset.materialization.official_joined_dataset;
    let mut claim = HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        copy_id: String::new(),
        claim_sha256: String::new(),
        attempt_id: review.attempt_id.clone(),
        admission_review_id: review.review_id.clone(),
        admission_review_sha256: review.review_sha256.clone(),
        output_validation_id: validation.validation_id.clone(),
        output_validation_sha256: validation.validation_sha256.clone(),
        materialization_id: validation.materialization_id.clone(),
        materialization_claim_sha256: validation.materialization_claim_sha256.clone(),
        materialization_result_sha256: validation.materialization_result_sha256.clone(),
        official_joined_dataset_sha256: validation.official_joined_dataset_sha256.clone(),
        source_validation_sha256: validation.source_validation_sha256.clone(),
        source_output_sha256: validation.source_output_sha256.clone(),
        dataset_id: validation.dataset_id.clone(),
        dataset_content_sha256: validation.dataset_content_sha256.clone(),
        dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(),
        candidate_set_sha256: validation.candidate_set_sha256.clone(),
        expected_rows_sha256: validation.recomputed_rows_sha256.clone(),
        expected_excluded_rows_sha256: validation.recomputed_excluded_rows_sha256.clone(),
        expected_target_commitments_sha256: validation.recomputed_target_commitments_sha256.clone(),
        expected_dataset_entry_count: official.dataset_entry_count,
        expected_active_row_count: official.active_row_count,
        expected_excluded_row_count: official.excluded_purge_or_embargo_row_count,
        expected_feature_catalog_count: official.feature_catalog_count,
        expected_target_count: official.target_count,
        expected_train_target_vector_count: official.train_target_vector_count,
        expected_validation_target_withheld_count: official.validation_target_withheld_count,
        expected_sealed_holdout_target_withheld_count: official
            .sealed_holdout_target_withheld_count,
        copied_by: copied_by.to_string(),
        excluded_prior_actor_ids,
        claimed_at,
        exact_current_stage_47_admission_and_complete_chain_confirmed: request
            .exact_current_stage_47_admission_and_complete_chain_confirmed,
        copier_independent_from_complete_prior_chain: true,
        claim_first_create_once_and_failure_consumes_confirmed: request
            .claim_first_create_once_and_failure_consumes_confirmed,
        exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed: request
            .exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed,
        validation_and_sealed_holdout_targets_remain_withheld_confirmed: request
            .validation_and_sealed_holdout_targets_remain_withheld_confirmed,
        independent_post_copy_validation_required_confirmed: request
            .independent_post_copy_validation_required_confirmed,
        no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed: request
            .no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed,
        claim_consumed: true,
        training_store_copy_started: true,
        exact_target_directory_write_allowed: true,
        generic_training_store_read_allowed: false,
        generic_training_store_write_allowed: false,
        ambient_environment_available: false,
        network_available: false,
        external_tools_available: false,
        production_state_write_available: false,
        historical_state_mutation_allowed: false,
        training_registration_allowed: false,
        training_run_allowed: false,
        reward_allowed: false,
        shadow_portfolio_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    claim.claim_sha256 = fingerprint_without(&claim, &["copy_id", "claim_sha256"])?;
    claim.copy_id = claim.claim_sha256[..32].to_string();
    validate_claim(&claim, admitted)?;
    Ok(claim)
}

fn build_training_store_dataset(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset, String> {
    let official = &admitted.dataset.materialization.official_joined_dataset;
    let rows_sha256 = hash_serializable(&official.rows)?;
    let excluded_rows_sha256 = hash_serializable(&official.excluded_rows)?;
    let mut commitments = official
        .rows
        .iter()
        .map(|row| {
            (
                row.dataset_entry_id.clone(),
                row.target_commitment_sha256.clone(),
            )
        })
        .collect::<Vec<_>>();
    commitments.sort();
    let target_commitments_sha256 = hash_serializable(&commitments)?;
    if rows_sha256 != claim.expected_rows_sha256
        || excluded_rows_sha256 != claim.expected_excluded_rows_sha256
        || target_commitments_sha256 != claim.expected_target_commitments_sha256
    {
        return Err("正式 joined dataset 内容与 Stage 47 准入绑定不一致，禁止复制".to_string());
    }
    let mut dataset = HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset {
        schema_version: TRAINING_STORE_DATASET_SCHEMA_VERSION.to_string(),
        training_store_dataset_sha256: String::new(),
        copy_id: claim.copy_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        admission_review_id: claim.admission_review_id.clone(),
        admission_review_sha256: claim.admission_review_sha256.clone(),
        output_validation_id: claim.output_validation_id.clone(),
        output_validation_sha256: claim.output_validation_sha256.clone(),
        materialization_id: claim.materialization_id.clone(),
        materialization_claim_sha256: claim.materialization_claim_sha256.clone(),
        materialization_result_sha256: claim.materialization_result_sha256.clone(),
        official_joined_dataset_sha256: claim.official_joined_dataset_sha256.clone(),
        source_validation_sha256: claim.source_validation_sha256.clone(),
        source_output_sha256: claim.source_output_sha256.clone(),
        dataset_id: claim.dataset_id.clone(),
        dataset_content_sha256: claim.dataset_content_sha256.clone(),
        dataset_manifest_sha256: claim.dataset_manifest_sha256.clone(),
        candidate_set_sha256: claim.candidate_set_sha256.clone(),
        source_rows_sha256: rows_sha256,
        source_excluded_rows_sha256: excluded_rows_sha256,
        source_target_commitments_sha256: target_commitments_sha256,
        dataset_entry_count: official.dataset_entry_count,
        active_row_count: official.active_row_count,
        excluded_purge_or_embargo_row_count: official.excluded_purge_or_embargo_row_count,
        feature_catalog_count: official.feature_catalog_count,
        target_count: official.target_count,
        train_target_vector_count: official.train_target_vector_count,
        validation_target_withheld_count: official.validation_target_withheld_count,
        sealed_holdout_target_withheld_count: official.sealed_holdout_target_withheld_count,
        rows: official.rows.clone(),
        excluded_rows: official.excluded_rows.clone(),
        exact_official_dataset_copy: true,
        one_to_one_join_preserved: true,
        point_in_time_and_explicit_missingness_preserved: true,
        official_split_purge_and_embargo_preserved: true,
        exact_raw_f64_bits_preserved: true,
        validation_targets_withheld: true,
        sealed_holdout_targets_withheld: true,
        action_position_and_reward_semantics_absent: true,
        copied_to_training_store: true,
        independently_validated_after_training_store_copy: false,
        eligible_for_training_registration_review: false,
        training_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    dataset.training_store_dataset_sha256 =
        fingerprint_without(&dataset, &["training_store_dataset_sha256"])?;
    Ok(dataset)
}

fn new_success_result(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    bytes: usize,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult, String> {
    let mut result = HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        copy_id: claim.copy_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        status: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus::CompletedPendingIndependentValidation,
        error: None,
        training_store_dataset_sha256: Some(dataset.training_store_dataset_sha256.clone()),
        training_store_dataset_bytes: bytes,
        copied_to_training_store: true,
        exact_official_dataset_copy_completed: true,
        independent_post_copy_validation_completed: false,
        eligible_for_training_registration_review: false,
        training_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    finalize_result(&mut result)?;
    validate_result(&result, claim, Some(dataset))?;
    Ok(result)
}

fn new_failed_result(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    error: &str,
    bytes: usize,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult, String> {
    let mut result = HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        copy_id: claim.copy_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        status: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus::FailedClaimConsumed,
        error: Some(error.chars().take(MAX_ERROR_CHARS).collect()),
        training_store_dataset_sha256: None,
        training_store_dataset_bytes: bytes,
        copied_to_training_store: false,
        exact_official_dataset_copy_completed: false,
        independent_post_copy_validation_completed: false,
        eligible_for_training_registration_review: false,
        training_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    finalize_result(&mut result)?;
    validate_result(&result, claim, None)?;
    Ok(result)
}

fn finalize_result(
    result: &mut HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult,
) -> Result<(), String> {
    result.result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].to_string();
    Ok(())
}

fn validate_claim(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
) -> Result<(), String> {
    let review = &admitted.admission_review;
    let validation = &admitted.dataset.validation;
    let official = &admitted.dataset.materialization.official_joined_dataset;
    let downstream_closed = !claim.generic_training_store_read_allowed
        && !claim.generic_training_store_write_allowed
        && !claim.ambient_environment_available
        && !claim.network_available
        && !claim.external_tools_available
        && !claim.production_state_write_available
        && !claim.historical_state_mutation_allowed
        && !claim.training_registration_allowed
        && !claim.training_run_allowed
        && !claim.reward_allowed
        && !claim.shadow_portfolio_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed;
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.policy_version != POLICY_VERSION
        || !valid_id(&claim.copy_id)
        || !valid_sha256(&claim.claim_sha256)
        || claim.copy_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != fingerprint_without(claim, &["copy_id", "claim_sha256"])?
        || claim.attempt_id != review.attempt_id
        || claim.admission_review_id != review.review_id
        || claim.admission_review_sha256 != review.review_sha256
        || claim.output_validation_id != validation.validation_id
        || claim.output_validation_sha256 != validation.validation_sha256
        || claim.materialization_id != validation.materialization_id
        || claim.materialization_claim_sha256 != validation.materialization_claim_sha256
        || claim.materialization_result_sha256 != validation.materialization_result_sha256
        || claim.official_joined_dataset_sha256 != validation.official_joined_dataset_sha256
        || claim.source_validation_sha256 != validation.source_validation_sha256
        || claim.source_output_sha256 != validation.source_output_sha256
        || claim.dataset_id != validation.dataset_id
        || claim.dataset_content_sha256 != validation.dataset_content_sha256
        || claim.dataset_manifest_sha256 != validation.dataset_manifest_sha256
        || claim.candidate_set_sha256 != validation.candidate_set_sha256
        || claim.expected_rows_sha256 != validation.recomputed_rows_sha256
        || claim.expected_excluded_rows_sha256 != validation.recomputed_excluded_rows_sha256
        || claim.expected_target_commitments_sha256
            != validation.recomputed_target_commitments_sha256
        || claim.expected_dataset_entry_count != official.dataset_entry_count
        || claim.expected_active_row_count != official.active_row_count
        || claim.expected_excluded_row_count != official.excluded_purge_or_embargo_row_count
        || claim.expected_feature_catalog_count != EXPECTED_FEATURE_COUNT
        || claim.expected_feature_catalog_count != official.feature_catalog_count
        || claim.expected_target_count != EXPECTED_TARGET_COUNT
        || claim.expected_target_count != official.target_count
        || claim.expected_train_target_vector_count != official.train_target_vector_count
        || claim.expected_validation_target_withheld_count
            != official.validation_target_withheld_count
        || claim.expected_sealed_holdout_target_withheld_count
            != official.sealed_holdout_target_withheld_count
        || claim.copied_by.trim().is_empty()
        || claim.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&claim.excluded_prior_actor_ids)
        || !actor_is_independent(&claim.copied_by, &claim.excluded_prior_actor_ids)
        || !claim.exact_current_stage_47_admission_and_complete_chain_confirmed
        || !claim.copier_independent_from_complete_prior_chain
        || !claim.claim_first_create_once_and_failure_consumes_confirmed
        || !claim.exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed
        || !claim.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        || !claim.independent_post_copy_validation_required_confirmed
        || !claim.no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed
        || !claim.claim_consumed
        || !claim.training_store_copy_started
        || !claim.exact_target_directory_write_allowed
        || !review.training_store_copy_candidate_admitted
        || !review.future_create_once_training_store_copy_eligible
        || !downstream_closed
    {
        return Err("training-store copy claim is invalid, drifted or over-authorized".to_string());
    }
    Ok(())
}

fn validate_training_store_dataset(
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    claim: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    official: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
) -> Result<(), String> {
    let mut commitments = dataset
        .rows
        .iter()
        .map(|row| {
            (
                row.dataset_entry_id.clone(),
                row.target_commitment_sha256.clone(),
            )
        })
        .collect::<Vec<_>>();
    commitments.sort();
    let target_visibility_valid = dataset.rows.iter().all(|row| match row.target_visibility {
        HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets => row
            .target_vector
            .as_ref()
            .is_some_and(|values| values.len() == EXPECTED_TARGET_COUNT),
        HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
        | HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld => {
            row.target_vector.is_none()
        }
    });
    let feature_catalog_valid = dataset
        .rows
        .iter()
        .all(|row| row.feature_records.len() == EXPECTED_FEATURE_COUNT);
    let downstream_closed = !dataset.independently_validated_after_training_store_copy
        && !dataset.eligible_for_training_registration_review
        && !dataset.training_registered
        && !dataset.training_authorized
        && !dataset.training_started
        && !dataset.reward_authorized
        && !dataset.shadow_portfolio_authorized
        && !dataset.order_generation_authorized
        && !dataset.broker_access_authorized
        && !dataset.trading_authorized;
    if dataset.schema_version != TRAINING_STORE_DATASET_SCHEMA_VERSION
        || !valid_sha256(&dataset.training_store_dataset_sha256)
        || dataset.training_store_dataset_sha256
            != fingerprint_without(dataset, &["training_store_dataset_sha256"])?
        || dataset.copy_id != claim.copy_id
        || dataset.attempt_id != claim.attempt_id
        || dataset.admission_review_id != claim.admission_review_id
        || dataset.admission_review_sha256 != claim.admission_review_sha256
        || dataset.output_validation_id != claim.output_validation_id
        || dataset.output_validation_sha256 != claim.output_validation_sha256
        || dataset.materialization_id != claim.materialization_id
        || dataset.materialization_claim_sha256 != claim.materialization_claim_sha256
        || dataset.materialization_result_sha256 != claim.materialization_result_sha256
        || dataset.official_joined_dataset_sha256 != claim.official_joined_dataset_sha256
        || dataset.source_validation_sha256 != claim.source_validation_sha256
        || dataset.source_output_sha256 != claim.source_output_sha256
        || dataset.dataset_id != claim.dataset_id
        || dataset.dataset_content_sha256 != claim.dataset_content_sha256
        || dataset.dataset_manifest_sha256 != claim.dataset_manifest_sha256
        || dataset.candidate_set_sha256 != claim.candidate_set_sha256
        || dataset.source_rows_sha256 != claim.expected_rows_sha256
        || dataset.source_rows_sha256 != hash_serializable(&dataset.rows)?
        || dataset.source_excluded_rows_sha256 != claim.expected_excluded_rows_sha256
        || dataset.source_excluded_rows_sha256 != hash_serializable(&dataset.excluded_rows)?
        || dataset.source_target_commitments_sha256 != claim.expected_target_commitments_sha256
        || dataset.source_target_commitments_sha256 != hash_serializable(&commitments)?
        || dataset.dataset_entry_count != claim.expected_dataset_entry_count
        || dataset.active_row_count != claim.expected_active_row_count
        || dataset.excluded_purge_or_embargo_row_count != claim.expected_excluded_row_count
        || dataset.active_row_count + dataset.excluded_purge_or_embargo_row_count
            != dataset.dataset_entry_count
        || dataset.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || dataset.target_count != EXPECTED_TARGET_COUNT
        || dataset.train_target_vector_count != claim.expected_train_target_vector_count
        || dataset.validation_target_withheld_count
            != claim.expected_validation_target_withheld_count
        || dataset.sealed_holdout_target_withheld_count
            != claim.expected_sealed_holdout_target_withheld_count
        || dataset.rows != official.rows
        || dataset.excluded_rows != official.excluded_rows
        || !dataset.exact_official_dataset_copy
        || !dataset.one_to_one_join_preserved
        || !dataset.point_in_time_and_explicit_missingness_preserved
        || !dataset.official_split_purge_and_embargo_preserved
        || !dataset.exact_raw_f64_bits_preserved
        || !dataset.validation_targets_withheld
        || !dataset.sealed_holdout_targets_withheld
        || !dataset.action_position_and_reward_semantics_absent
        || !dataset.copied_to_training_store
        || !target_visibility_valid
        || !feature_catalog_valid
        || !dataset
            .excluded_rows
            .iter()
            .all(|row| !row.target_values_opened)
        || !downstream_closed
    {
        return Err("training-store dataset is invalid, drifted or over-authorized".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult,
    claim: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    dataset: Option<&HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset>,
) -> Result<(), String> {
    let success = result.status
        == HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus::CompletedPendingIndependentValidation;
    let success_fields = if success {
        let dataset =
            dataset.ok_or_else(|| "successful copy misses training-store dataset".to_string())?;
        result.error.is_none()
            && result.training_store_dataset_sha256.as_deref()
                == Some(dataset.training_store_dataset_sha256.as_str())
            && result.training_store_dataset_bytes > 0
            && result.training_store_dataset_bytes <= MAX_TRAINING_STORE_DATASET_BYTES
            && result.copied_to_training_store
            && result.exact_official_dataset_copy_completed
    } else {
        dataset.is_none()
            && result.error.as_ref().is_some_and(|value| !value.is_empty())
            && result.training_store_dataset_sha256.is_none()
            && !result.copied_to_training_store
            && !result.exact_official_dataset_copy_completed
    };
    let downstream_closed = !result.independent_post_copy_validation_completed
        && !result.eligible_for_training_registration_review
        && !result.training_registered
        && !result.training_authorized
        && !result.training_started
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
        || result.copy_id != claim.copy_id
        || result.claim_sha256 != claim.claim_sha256
        || !success_fields
        || !downstream_closed
    {
        return Err("training-store copy result is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_attempt(
    state: &AppState,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
) -> Result<Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt>, String> {
    let directory = copy_directory(state, &admitted.admission_review.attempt_id);
    let claim_bytes = match tokio::fs::read(directory.join("claim.json")).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let claim: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim =
        serde_json::from_slice(&claim_bytes).map_err(|error| error.to_string())?;
    validate_claim(&claim, admitted)?;
    let result_bytes = match tokio::fs::read(directory.join("result.json")).await {
        Ok(value) => Some(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.to_string()),
    };
    let result: Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult> =
        result_bytes
            .as_ref()
            .map(|bytes| serde_json::from_slice(bytes).map_err(|error| error.to_string()))
            .transpose()?;
    let training_store_dataset = if let Some(result) = &result {
        match result.training_store_dataset_sha256.as_deref() {
            Some(sha256) => {
                let bytes = tokio::fs::read(
                    directory.join(format!("training-store-dataset-{sha256}.json")),
                )
                .await
                .map_err(|error| error.to_string())?;
                let dataset: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset =
                    serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
                validate_training_store_dataset(
                    &dataset,
                    &claim,
                    &admitted.dataset.materialization.official_joined_dataset,
                )?;
                Some(dataset)
            }
            None => None,
        }
    } else {
        None
    };
    if let Some(result) = &result {
        validate_result(result, &claim, training_store_dataset.as_ref())?;
    }
    Ok(Some(
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt {
            claim,
            result,
            training_store_dataset,
        },
    ))
}

fn excluded_actor_ids(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
) -> Vec<String> {
    let mut values = admitted.admission_review.excluded_prior_actor_ids.clone();
    values.push(admitted.admission_review.reviewer_id.clone());
    values.push(admitted.dataset.validation.validated_by.clone());
    values.push(admitted.dataset.validation.materialized_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded_actor_ids: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded_actor_ids.iter().all(|value| value != actor_id)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let map = json
        .as_object_mut()
        .ok_or_else(|| "fingerprinted value must be an object".to_string())?;
    for field in fields {
        map.remove(*field);
    }
    hash_serializable(&json)
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

fn copy_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("isolated-training-store")
        .join("feature-label-join-target-copies")
        .join(attempt_id)
}

struct CopyLock {
    path: PathBuf,
}

impl Drop for CopyLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<CopyLock, String> {
    let root = copy_directory(state, attempt_id);
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|error| error.to_string())?;
    let path = root.join(".copy.lock");
    let mut stale_lock_removed = false;
    loop {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(CopyLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = !stale_lock_removed
                    && std::fs::metadata(&path)
                        .and_then(|metadata| metadata.modified())
                        .ok()
                        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                        .is_some_and(|age| age > StdDuration::from_secs(300));
                if stale {
                    tokio::fs::remove_file(&path)
                        .await
                        .map_err(|remove_error| remove_error.to_string())?;
                    stale_lock_removed = true;
                    continue;
                }
                return Err("这条正式数据集已有训练存储复制正在进行".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
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

    fn minimal_claim() -> HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim {
        let rows = Vec::<HistoricalOutcomeFeatureLabelJoinTargetCandidateRow>::new();
        let excluded = Vec::<HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit>::new();
        let commitments = Vec::<(String, String)>::new();
        let mut claim = HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim {
            schema_version: CLAIM_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            copy_id: String::new(),
            claim_sha256: String::new(),
            attempt_id: "1".repeat(32),
            admission_review_id: "2".repeat(32),
            admission_review_sha256: "3".repeat(64),
            output_validation_id: "4".repeat(32),
            output_validation_sha256: "5".repeat(64),
            materialization_id: "6".repeat(32),
            materialization_claim_sha256: "7".repeat(64),
            materialization_result_sha256: "8".repeat(64),
            official_joined_dataset_sha256: "9".repeat(64),
            source_validation_sha256: "a".repeat(64),
            source_output_sha256: "b".repeat(64),
            dataset_id: "c".repeat(32),
            dataset_content_sha256: "d".repeat(64),
            dataset_manifest_sha256: "e".repeat(64),
            candidate_set_sha256: "f".repeat(64),
            expected_rows_sha256: hash_serializable(&rows).unwrap(),
            expected_excluded_rows_sha256: hash_serializable(&excluded).unwrap(),
            expected_target_commitments_sha256: hash_serializable(&commitments).unwrap(),
            expected_dataset_entry_count: 0,
            expected_active_row_count: 0,
            expected_excluded_row_count: 0,
            expected_feature_catalog_count: EXPECTED_FEATURE_COUNT,
            expected_target_count: EXPECTED_TARGET_COUNT,
            expected_train_target_vector_count: 0,
            expected_validation_target_withheld_count: 0,
            expected_sealed_holdout_target_withheld_count: 0,
            copied_by: "copier".into(),
            excluded_prior_actor_ids: vec!["admitter".into(), "validator".into()],
            claimed_at: Utc::now(),
            exact_current_stage_47_admission_and_complete_chain_confirmed: true,
            copier_independent_from_complete_prior_chain: true,
            claim_first_create_once_and_failure_consumes_confirmed: true,
            exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed: true,
            validation_and_sealed_holdout_targets_remain_withheld_confirmed: true,
            independent_post_copy_validation_required_confirmed: true,
            no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed: true,
            claim_consumed: true,
            training_store_copy_started: true,
            exact_target_directory_write_allowed: true,
            generic_training_store_read_allowed: false,
            generic_training_store_write_allowed: false,
            ambient_environment_available: false,
            network_available: false,
            external_tools_available: false,
            production_state_write_available: false,
            historical_state_mutation_allowed: false,
            training_registration_allowed: false,
            training_run_allowed: false,
            reward_allowed: false,
            shadow_portfolio_allowed: false,
            order_generation_allowed: false,
            broker_access_allowed: false,
            trading_allowed: false,
        };
        claim.claim_sha256 = fingerprint_without(&claim, &["copy_id", "claim_sha256"]).unwrap();
        claim.copy_id = claim.claim_sha256[..32].to_string();
        claim
    }

    fn minimal_official(
        claim: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    ) -> HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
        HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
            schema_version: "official".into(),
            official_joined_dataset_sha256: claim.official_joined_dataset_sha256.clone(),
            materialization_id: claim.materialization_id.clone(),
            attempt_id: claim.attempt_id.clone(),
            admission_review_id: "0".repeat(32),
            admission_review_sha256: "0".repeat(64),
            validation_id: "0".repeat(32),
            validation_sha256: "0".repeat(64),
            source_claim_sha256: "0".repeat(64),
            source_result_id: "0".repeat(32),
            source_result_sha256: "0".repeat(64),
            source_output_sha256: claim.source_output_sha256.clone(),
            authorization_review_id: "0".repeat(32),
            authorization_review_sha256: "0".repeat(64),
            isolated_runner_id: "0".repeat(32),
            isolated_runner_spec_sha256: "0".repeat(64),
            implementation_id: "0".repeat(32),
            implementation_sha256: "0".repeat(64),
            specification_id: "0".repeat(32),
            specification_sha256: "0".repeat(64),
            join_specification_sha256: "0".repeat(64),
            target_specification_sha256: "0".repeat(64),
            split_manifest_sha256: "0".repeat(64),
            feature_bundle_sha256: "0".repeat(64),
            combined_artifact_sha256: "0".repeat(64),
            dataset_id: claim.dataset_id.clone(),
            dataset_content_sha256: claim.dataset_content_sha256.clone(),
            dataset_manifest_sha256: claim.dataset_manifest_sha256.clone(),
            candidate_set_sha256: claim.candidate_set_sha256.clone(),
            source_recomputed_rows_sha256: claim.expected_rows_sha256.clone(),
            source_recomputed_excluded_rows_sha256: claim.expected_excluded_rows_sha256.clone(),
            target_commitments_sha256: claim.expected_target_commitments_sha256.clone(),
            dataset_entry_count: 0,
            active_row_count: 0,
            excluded_purge_or_embargo_row_count: 0,
            feature_catalog_count: EXPECTED_FEATURE_COUNT,
            target_count: EXPECTED_TARGET_COUNT,
            train_target_vector_count: 0,
            validation_target_withheld_count: 0,
            sealed_holdout_target_withheld_count: 0,
            rows: Vec::new(),
            excluded_rows: Vec::new(),
            exact_admitted_candidate_copy: true,
            one_to_one_join_preserved: true,
            point_in_time_and_explicit_missingness_preserved: true,
            official_split_purge_and_embargo_preserved: true,
            exact_raw_f64_bits_preserved: true,
            validation_targets_withheld: true,
            sealed_holdout_targets_withheld: true,
            action_position_and_reward_semantics_absent: true,
            official_joined_dataset_created: true,
            independently_validated_after_materialization: false,
            eligible_for_training_store_copy: false,
            copied_to_training_store: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        }
    }

    fn minimal_training_dataset(
        claim: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    ) -> HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset {
        let mut dataset = HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset {
            schema_version: TRAINING_STORE_DATASET_SCHEMA_VERSION.into(),
            training_store_dataset_sha256: String::new(),
            copy_id: claim.copy_id.clone(),
            attempt_id: claim.attempt_id.clone(),
            admission_review_id: claim.admission_review_id.clone(),
            admission_review_sha256: claim.admission_review_sha256.clone(),
            output_validation_id: claim.output_validation_id.clone(),
            output_validation_sha256: claim.output_validation_sha256.clone(),
            materialization_id: claim.materialization_id.clone(),
            materialization_claim_sha256: claim.materialization_claim_sha256.clone(),
            materialization_result_sha256: claim.materialization_result_sha256.clone(),
            official_joined_dataset_sha256: claim.official_joined_dataset_sha256.clone(),
            source_validation_sha256: claim.source_validation_sha256.clone(),
            source_output_sha256: claim.source_output_sha256.clone(),
            dataset_id: claim.dataset_id.clone(),
            dataset_content_sha256: claim.dataset_content_sha256.clone(),
            dataset_manifest_sha256: claim.dataset_manifest_sha256.clone(),
            candidate_set_sha256: claim.candidate_set_sha256.clone(),
            source_rows_sha256: claim.expected_rows_sha256.clone(),
            source_excluded_rows_sha256: claim.expected_excluded_rows_sha256.clone(),
            source_target_commitments_sha256: claim.expected_target_commitments_sha256.clone(),
            dataset_entry_count: 0,
            active_row_count: 0,
            excluded_purge_or_embargo_row_count: 0,
            feature_catalog_count: EXPECTED_FEATURE_COUNT,
            target_count: EXPECTED_TARGET_COUNT,
            train_target_vector_count: 0,
            validation_target_withheld_count: 0,
            sealed_holdout_target_withheld_count: 0,
            rows: Vec::new(),
            excluded_rows: Vec::new(),
            exact_official_dataset_copy: true,
            one_to_one_join_preserved: true,
            point_in_time_and_explicit_missingness_preserved: true,
            official_split_purge_and_embargo_preserved: true,
            exact_raw_f64_bits_preserved: true,
            validation_targets_withheld: true,
            sealed_holdout_targets_withheld: true,
            action_position_and_reward_semantics_absent: true,
            copied_to_training_store: true,
            independently_validated_after_training_store_copy: false,
            eligible_for_training_registration_review: false,
            training_registered: false,
            training_authorized: false,
            training_started: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        dataset.training_store_dataset_sha256 =
            fingerprint_without(&dataset, &["training_store_dataset_sha256"]).unwrap();
        dataset
    }

    #[test]
    fn claim_hash_binds_exact_target_directory_permission() {
        let claim = minimal_claim();
        let mut changed = claim.clone();
        changed.exact_target_directory_write_allowed = false;
        assert_ne!(
            claim.claim_sha256,
            fingerprint_without(&changed, &["copy_id", "claim_sha256"]).unwrap()
        );
    }

    #[test]
    fn claim_keeps_generic_store_and_training_capabilities_closed() {
        let claim = minimal_claim();
        assert!(claim.exact_target_directory_write_allowed);
        assert!(!claim.generic_training_store_read_allowed);
        assert!(!claim.generic_training_store_write_allowed);
        assert!(!claim.training_registration_allowed);
        assert!(!claim.training_run_allowed);
    }

    #[test]
    fn exact_empty_copy_is_valid_and_waits_for_independent_validation() {
        let claim = minimal_claim();
        let official = minimal_official(&claim);
        let dataset = minimal_training_dataset(&claim);
        validate_training_store_dataset(&dataset, &claim, &official).unwrap();
        assert!(dataset.copied_to_training_store);
        assert!(!dataset.independently_validated_after_training_store_copy);
        assert!(!dataset.eligible_for_training_registration_review);
    }

    #[test]
    fn copied_dataset_rejects_any_training_authorization() {
        let claim = minimal_claim();
        let official = minimal_official(&claim);
        let mut dataset = minimal_training_dataset(&claim);
        dataset.training_authorized = true;
        dataset.training_store_dataset_sha256 =
            fingerprint_without(&dataset, &["training_store_dataset_sha256"]).unwrap();
        assert!(validate_training_store_dataset(&dataset, &claim, &official).is_err());
    }

    #[test]
    fn copied_dataset_rejects_a_changed_source_hash() {
        let claim = minimal_claim();
        let official = minimal_official(&claim);
        let mut dataset = minimal_training_dataset(&claim);
        dataset.source_rows_sha256 = "0".repeat(64);
        dataset.training_store_dataset_sha256 =
            fingerprint_without(&dataset, &["training_store_dataset_sha256"]).unwrap();
        assert!(validate_training_store_dataset(&dataset, &claim, &official).is_err());
    }

    #[test]
    fn success_result_never_implies_training_or_trading() {
        let claim = minimal_claim();
        let dataset = minimal_training_dataset(&claim);
        let result = new_success_result(&claim, &dataset, 1).unwrap();
        assert!(result.copied_to_training_store);
        assert!(!result.independent_post_copy_validation_completed);
        assert!(!result.training_registered);
        assert!(!result.training_authorized);
        assert!(!result.training_started);
        assert!(!result.trading_authorized);
    }

    #[test]
    fn failed_result_consumes_claim_without_dataset() {
        let claim = minimal_claim();
        let result = new_failed_result(&claim, "copy failed", 0).unwrap();
        assert_eq!(
            result.status,
            HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus::FailedClaimConsumed
        );
        assert!(!result.copied_to_training_store);
        assert!(result.training_store_dataset_sha256.is_none());
        validate_result(&result, &claim, None).unwrap();
    }

    #[test]
    fn result_hash_binds_post_copy_gate() {
        let claim = minimal_claim();
        let dataset = minimal_training_dataset(&claim);
        let result = new_success_result(&claim, &dataset, 1).unwrap();
        let mut changed = result.clone();
        changed.eligible_for_training_registration_review = true;
        assert_ne!(
            result.result_sha256,
            fingerprint_without(&changed, &["result_id", "result_sha256"]).unwrap()
        );
    }

    #[test]
    fn copier_must_be_outside_complete_prior_chain() {
        let excluded = vec!["admitter".to_string(), "validator".to_string()];
        assert!(!actor_is_independent("validator", &excluded));
        assert!(actor_is_independent("copier", &excluded));
    }
}
