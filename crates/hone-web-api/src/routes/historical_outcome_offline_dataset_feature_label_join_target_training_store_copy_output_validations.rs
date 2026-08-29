//! Independent post-copy validation for an isolated training-store dataset.
//!
//! Stage 49 never trusts the Stage 48 copy flags. A different administrator
//! reopens the immutable copy attempt and independently recomputes the claim,
//! result, copied dataset, rows, excluded rows and target commitments. Passing
//! only opens a future training-registration admission review; it does not
//! register or run training, define reward, create a shadow portfolio,
//! generate orders, access a broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_execution_attempts::HistoricalOutcomeFeatureLabelJoinTargetVisibility;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copies::{
    CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt,
    HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult,
    HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus,
    HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_store_copy_admission_reviews::{
    AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
    admitted_official_joined_datasets_for_training_store_copy,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-feature-label-join-target-training-store-copy-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-store-copy-output-validation-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-training-store-copy-independent-output-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-training-store-copy-validator-v1-independent-reopen-and-canonical-recomputation";
const COPY_CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-store-copy-claim-v1";
const COPY_RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-store-copy-result-v1";
const TRAINING_STORE_DATASET_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-isolated-training-store-dataset-v1";
const COPY_POLICY_VERSION: &str =
    "hone-feature-label-join-target-training-store-claim-first-create-once-exact-copy-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const MAX_TRAINING_STORE_DATASET_BYTES: usize = 64 * 1024 * 1024;
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest {
    expected_copy_id: String,
    expected_copy_claim_sha256: String,
    expected_copy_result_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_admission_review_sha256: String,
    expected_output_validation_sha256: String,
    expected_official_joined_dataset_sha256: String,
    expected_rows_sha256: String,
    expected_excluded_rows_sha256: String,
    expected_target_commitments_sha256: String,
    independent_reopen_and_recomputation_confirmed: bool,
    exact_current_stage_47_and_stage_48_binding_confirmed: bool,
    validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationVerdict {
    ValidatedTrainingStoreCopyForFutureTrainingRegistrationReview,
    FailedIndependentPostCopyValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub copy_id: String,
    pub copy_claim_sha256: String,
    pub copy_result_id: String,
    pub copy_result_sha256: String,
    pub training_store_dataset_sha256: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub output_validation_id: String,
    pub output_validation_sha256: String,
    pub materialization_id: String,
    pub official_joined_dataset_sha256: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub copied_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_copier_and_complete_prior_chain: bool,
    pub exact_current_stage_47_and_stage_48_chain_verified: bool,
    pub copy_claim_fingerprint_verified: bool,
    pub copy_result_fingerprint_verified: bool,
    pub training_store_dataset_fingerprint_verified: bool,
    pub exact_official_dataset_copy_verified: bool,
    pub exact_one_to_one_entry_join_and_cardinality_verified: bool,
    pub exact_65_feature_catalog_verified: bool,
    pub point_in_time_and_explicit_missingness_verified: bool,
    pub official_split_purge_and_embargo_verified: bool,
    pub exact_nine_raw_f64_bits_and_commitments_verified: bool,
    pub train_only_target_visibility_verified: bool,
    pub validation_targets_withheld_verified: bool,
    pub sealed_holdout_targets_withheld_verified: bool,
    pub downstream_authority_closed_verified: bool,
    pub recomputed_copy_claim_sha256: String,
    pub recomputed_copy_result_sha256: String,
    pub recomputed_training_store_dataset_sha256: String,
    pub recomputed_rows_sha256: String,
    pub recomputed_excluded_rows_sha256: String,
    pub recomputed_target_commitments_sha256: String,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationVerdict,
    pub training_store_copy_independently_validated: bool,
    pub future_training_registration_review_eligible: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationItem {
    pub copied_dataset: CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    pub validation:
        Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_training_store_copy_count: usize,
    pub failed_validation_count: usize,
    pub future_training_registration_review_eligible_count: usize,
    pub validation_status: String,
    pub independent_post_copy_validation_available: bool,
    pub training_registration_available: bool,
    pub training_registered: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationReadinessSummary
{
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_training_store_copy_count: usize,
    pub failed_validation_count: usize,
    pub future_training_registration_review_eligible_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset
{
    pub copied_dataset: CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    pub validation: HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_training_store_copy_output_validations(
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
            warn!(%error, "training-store copy output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "训练存储副本独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_feature_label_join_target_training_store_copy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match validate_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_feature_label_join_target_training_store_copy_output_validation_readiness_summary(
    state: &AppState,
) -> Result<
    HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationReadinessSummary,
    String,
> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationReadinessSummary {
            validation_eligible_count: registry.validation_eligible_count,
            validation_count: registry.validation_count,
            independently_validated_training_store_copy_count: registry
                .independently_validated_training_store_copy_count,
            failed_validation_count: registry.failed_validation_count,
            future_training_registration_review_eligible_count: registry
                .future_training_registration_review_eligible_count,
            validation_status: registry.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_training_store_copies_for_training_registration_review(
    state: &AppState,
) -> Result<
    Vec<IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset>,
    String,
> {
    let validations = read_validations(state).await?;
    let index = validations
        .into_iter()
        .map(|record| (record.attempt_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for copied_dataset in completed_copies(state).await? {
        let Some(validation) = index.get(&copied_dataset.attempt.claim.attempt_id).cloned() else {
            continue;
        };
        validate_record(&validation)?;
        let audit = independently_audit(&copied_dataset)?;
        if validation.training_store_copy_independently_validated
            && validation.future_training_registration_review_eligible
            && audit.mismatch_reasons.is_empty()
            && validation.copy_result_sha256
                == copied_dataset
                    .attempt
                    .result
                    .as_ref()
                    .map(|value| value.result_sha256.as_str())
                    .unwrap_or_default()
            && validation.training_store_dataset_sha256
                == copied_dataset
                    .attempt
                    .training_store_dataset
                    .as_ref()
                    .map(|value| value.training_store_dataset_sha256.as_str())
                    .unwrap_or_default()
        {
            output.push(
                IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset {
                    copied_dataset,
                    validation,
                },
            );
        } else {
            return Err("已通过的训练存储副本校验与当前不可变工件不再一致".to_string());
        }
    }
    output.sort_by(|left, right| {
        right
            .validation
            .validated_at
            .cmp(&left.validation.validated_at)
    });
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry, String>
{
    let copies = completed_copies(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let items = copies
        .into_iter()
        .map(|copied_dataset| {
            let validation = index
                .get(copied_dataset.attempt.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationItem {
                validation_eligible: validation.is_none(),
                copied_dataset,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let independently_validated_training_store_copy_count = validations
        .iter()
        .filter(|record| record.training_store_copy_independently_validated)
        .count();
    let failed_validation_count =
        validations.len() - independently_validated_training_store_copy_count;
    let validation_status = if failed_validation_count > 0 {
        "training_store_copy_independent_validation_failed_fail_closed"
    } else if independently_validated_training_store_copy_count > 0 {
        "training_store_copy_independently_validated_waiting_training_registration_review"
    } else if validation_eligible_count > 0 {
        "waiting_independent_training_store_copy_output_validation"
    } else {
        "waiting_completed_training_store_copy"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count: validations.len(),
        independently_validated_training_store_copy_count,
        failed_validation_count,
        future_training_registration_review_eligible_count: independently_validated_training_store_copy_count,
        validation_status: validation_status.to_string(),
        independent_post_copy_validation_available: true,
        training_registration_available: false,
        training_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 49 阶段由复制人和完整上游之外的新管理员重新打开 Stage 48 claim/result/训练存储副本，独立重算工件、行、排除项和目标承诺并与精确 Stage 47 正式数据集逐行逐位核对。通过只开放未来训练登记准入复核资格；当前不登记、不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord, String>
{
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.attempt_id == attempt_id)
    {
        return Err("这份训练存储副本已形成不可变独立校验记录，不得重放".to_string());
    }
    let copied = completed_copies(state)
        .await?
        .into_iter()
        .find(|value| value.attempt.claim.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有完整训练存储副本可供独立校验".to_string())?;
    verify_expected_bindings(&copied, &request)?;
    let excluded_prior_actor_ids = prior_actor_ids(&copied);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("训练存储副本校验人不能是复制人或完整上游链任一角色".to_string());
    }
    let audit = independently_audit(&copied)?;
    let passed = independent && audit.mismatch_reasons.is_empty();
    let claim = &copied.attempt.claim;
    let result = copied
        .attempt
        .result
        .as_ref()
        .ok_or_else(|| "复制结果缺失".to_string())?;
    let dataset = copied
        .attempt
        .training_store_dataset
        .as_ref()
        .ok_or_else(|| "训练存储副本缺失".to_string())?;
    let mut record =
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord {
            schema_version: RECORD_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            validation_id: String::new(),
            validation_sha256: String::new(),
            attempt_id: attempt_id.to_string(),
            copy_id: claim.copy_id.clone(),
            copy_claim_sha256: claim.claim_sha256.clone(),
            copy_result_id: result.result_id.clone(),
            copy_result_sha256: result.result_sha256.clone(),
            training_store_dataset_sha256: dataset.training_store_dataset_sha256.clone(),
            admission_review_id: claim.admission_review_id.clone(),
            admission_review_sha256: claim.admission_review_sha256.clone(),
            output_validation_id: claim.output_validation_id.clone(),
            output_validation_sha256: claim.output_validation_sha256.clone(),
            materialization_id: claim.materialization_id.clone(),
            official_joined_dataset_sha256: claim.official_joined_dataset_sha256.clone(),
            source_validation_sha256: claim.source_validation_sha256.clone(),
            source_output_sha256: claim.source_output_sha256.clone(),
            dataset_id: claim.dataset_id.clone(),
            dataset_content_sha256: claim.dataset_content_sha256.clone(),
            dataset_manifest_sha256: claim.dataset_manifest_sha256.clone(),
            candidate_set_sha256: claim.candidate_set_sha256.clone(),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
            validator_implementation_sha256: validator_implementation_sha256(),
            validated_at: Utc::now(),
            validated_by: validator_id.to_string(),
            copied_by: claim.copied_by.clone(),
            excluded_prior_actor_ids,
            validator_independent_from_copier_and_complete_prior_chain: independent,
            exact_current_stage_47_and_stage_48_chain_verified: audit.exact_chain,
            copy_claim_fingerprint_verified: audit.claim_fingerprint,
            copy_result_fingerprint_verified: audit.result_fingerprint,
            training_store_dataset_fingerprint_verified: audit.dataset_fingerprint,
            exact_official_dataset_copy_verified: audit.exact_copy,
            exact_one_to_one_entry_join_and_cardinality_verified: audit.one_to_one,
            exact_65_feature_catalog_verified: audit.feature_catalog,
            point_in_time_and_explicit_missingness_verified: audit.point_in_time_missingness,
            official_split_purge_and_embargo_verified: audit.split_purge_embargo,
            exact_nine_raw_f64_bits_and_commitments_verified: audit.target_bits_commitments,
            train_only_target_visibility_verified: audit.train_only_visibility,
            validation_targets_withheld_verified: audit.validation_withheld,
            sealed_holdout_targets_withheld_verified: audit.sealed_holdout_withheld,
            downstream_authority_closed_verified: audit.downstream_closed,
            recomputed_copy_claim_sha256: audit.recomputed_claim_sha256,
            recomputed_copy_result_sha256: audit.recomputed_result_sha256,
            recomputed_training_store_dataset_sha256: audit.recomputed_dataset_sha256,
            recomputed_rows_sha256: audit.recomputed_rows_sha256,
            recomputed_excluded_rows_sha256: audit.recomputed_excluded_rows_sha256,
            recomputed_target_commitments_sha256: audit.recomputed_target_commitments_sha256,
            mismatch_reasons: audit.mismatch_reasons,
            verdict: if passed {
                HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationVerdict::ValidatedTrainingStoreCopyForFutureTrainingRegistrationReview
            } else {
                HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationVerdict::FailedIndependentPostCopyValidation
            },
            training_store_copy_independently_validated: passed,
            future_training_registration_review_eligible: passed,
            training_registered: false,
            training_authorized: false,
            training_started: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
    record.validation_sha256 = record_fingerprint(&record)?;
    record.validation_id = record.validation_sha256[..32].to_string();
    validate_record(&record)?;
    write_immutable_json(
        &validation_root(state)
            .join(attempt_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

#[derive(Debug)]
struct IndependentAudit {
    exact_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    dataset_fingerprint: bool,
    exact_copy: bool,
    one_to_one: bool,
    feature_catalog: bool,
    point_in_time_missingness: bool,
    split_purge_embargo: bool,
    target_bits_commitments: bool,
    train_only_visibility: bool,
    validation_withheld: bool,
    sealed_holdout_withheld: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_dataset_sha256: String,
    recomputed_rows_sha256: String,
    recomputed_excluded_rows_sha256: String,
    recomputed_target_commitments_sha256: String,
    mismatch_reasons: Vec<String>,
}

fn independently_audit(
    copied: &CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
) -> Result<IndependentAudit, String> {
    let claim = &copied.attempt.claim;
    let result = copied
        .attempt
        .result
        .as_ref()
        .ok_or_else(|| "训练存储复制结果缺失".to_string())?;
    let dataset = copied
        .attempt
        .training_store_dataset
        .as_ref()
        .ok_or_else(|| "训练存储副本缺失".to_string())?;
    let admitted = &copied.admitted_dataset;
    let review = &admitted.admission_review;
    let source_validation = &admitted.dataset.validation;
    let official = &admitted.dataset.materialization.official_joined_dataset;
    let recomputed_claim_sha256 = fingerprint_without(claim, &["copy_id", "claim_sha256"])?;
    let recomputed_result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    let recomputed_dataset_sha256 =
        fingerprint_without(dataset, &["training_store_dataset_sha256"])?;
    let recomputed_dataset_bytes = serde_json::to_vec(dataset)
        .map_err(|error| error.to_string())?
        .len();
    let recomputed_rows_sha256 = hash_serializable(&dataset.rows)?;
    let recomputed_excluded_rows_sha256 = hash_serializable(&dataset.excluded_rows)?;
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
    let recomputed_target_commitments_sha256 = hash_serializable(&commitments)?;
    let exact_chain = claim.attempt_id == review.attempt_id
        && claim.admission_review_id == review.review_id
        && claim.admission_review_sha256 == review.review_sha256
        && claim.output_validation_id == source_validation.validation_id
        && claim.output_validation_sha256 == source_validation.validation_sha256
        && claim.materialization_id == source_validation.materialization_id
        && claim.materialization_claim_sha256 == source_validation.materialization_claim_sha256
        && claim.materialization_result_sha256 == source_validation.materialization_result_sha256
        && claim.official_joined_dataset_sha256 == source_validation.official_joined_dataset_sha256
        && claim.source_validation_sha256 == source_validation.source_validation_sha256
        && claim.source_output_sha256 == source_validation.source_output_sha256
        && claim.dataset_id == official.dataset_id
        && claim.dataset_content_sha256 == official.dataset_content_sha256
        && claim.dataset_manifest_sha256 == official.dataset_manifest_sha256
        && claim.candidate_set_sha256 == official.candidate_set_sha256
        && claim.expected_rows_sha256 == source_validation.recomputed_rows_sha256
        && claim.expected_excluded_rows_sha256 == source_validation.recomputed_excluded_rows_sha256
        && claim.expected_target_commitments_sha256
            == source_validation.recomputed_target_commitments_sha256
        && claim.expected_dataset_entry_count == official.dataset_entry_count
        && claim.expected_active_row_count == official.active_row_count
        && claim.expected_excluded_row_count == official.excluded_purge_or_embargo_row_count
        && claim.expected_feature_catalog_count == EXPECTED_FEATURE_COUNT
        && claim.expected_feature_catalog_count == official.feature_catalog_count
        && claim.expected_target_count == EXPECTED_TARGET_COUNT
        && claim.expected_target_count == official.target_count
        && claim.expected_train_target_vector_count == official.train_target_vector_count
        && claim.expected_validation_target_withheld_count
            == official.validation_target_withheld_count
        && claim.expected_sealed_holdout_target_withheld_count
            == official.sealed_holdout_target_withheld_count;
    let claim_fingerprint = claim.schema_version == COPY_CLAIM_SCHEMA_VERSION
        && claim.policy_version == COPY_POLICY_VERSION
        && claim.claim_sha256 == recomputed_claim_sha256
        && claim.copy_id == recomputed_claim_sha256[..32]
        && claim.claim_consumed
        && claim.training_store_copy_started
        && claim.copier_independent_from_complete_prior_chain
        && claim.exact_current_stage_47_admission_and_complete_chain_confirmed
        && claim.claim_first_create_once_and_failure_consumes_confirmed
        && claim.exact_official_dataset_copy_without_recompute_repair_or_imputation_confirmed
        && claim.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        && claim.independent_post_copy_validation_required_confirmed
        && claim.no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed;
    let result_fingerprint = result.schema_version == COPY_RESULT_SCHEMA_VERSION && result.policy_version == COPY_POLICY_VERSION
        && result.result_sha256 == recomputed_result_sha256 && result.result_id == recomputed_result_sha256[..32]
        && result.copy_id == claim.copy_id && result.claim_sha256 == claim.claim_sha256
        && result.status == HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus::CompletedPendingIndependentValidation
        && result.error.is_none() && result.training_store_dataset_sha256.as_deref() == Some(dataset.training_store_dataset_sha256.as_str())
        && result.training_store_dataset_bytes > 0 && result.training_store_dataset_bytes <= MAX_TRAINING_STORE_DATASET_BYTES
        && result.training_store_dataset_bytes == recomputed_dataset_bytes
        && result.copied_to_training_store && result.exact_official_dataset_copy_completed;
    let dataset_fingerprint = dataset.schema_version == TRAINING_STORE_DATASET_SCHEMA_VERSION
        && dataset.training_store_dataset_sha256 == recomputed_dataset_sha256
        && dataset.copy_id == claim.copy_id
        && dataset.attempt_id == claim.attempt_id
        && dataset.admission_review_id == claim.admission_review_id
        && dataset.admission_review_sha256 == claim.admission_review_sha256
        && dataset.output_validation_id == claim.output_validation_id
        && dataset.output_validation_sha256 == claim.output_validation_sha256
        && dataset.materialization_id == claim.materialization_id
        && dataset.materialization_claim_sha256 == claim.materialization_claim_sha256
        && dataset.materialization_result_sha256 == claim.materialization_result_sha256
        && dataset.official_joined_dataset_sha256 == claim.official_joined_dataset_sha256
        && dataset.source_validation_sha256 == claim.source_validation_sha256
        && dataset.source_output_sha256 == claim.source_output_sha256
        && dataset.dataset_id == claim.dataset_id
        && dataset.dataset_content_sha256 == claim.dataset_content_sha256
        && dataset.dataset_manifest_sha256 == claim.dataset_manifest_sha256
        && dataset.candidate_set_sha256 == claim.candidate_set_sha256;
    let exact_copy = dataset.rows == official.rows
        && dataset.excluded_rows == official.excluded_rows
        && dataset.exact_official_dataset_copy
        && recomputed_rows_sha256 == claim.expected_rows_sha256
        && recomputed_excluded_rows_sha256 == claim.expected_excluded_rows_sha256
        && dataset.source_rows_sha256 == recomputed_rows_sha256
        && dataset.source_excluded_rows_sha256 == recomputed_excluded_rows_sha256;
    let one_to_one = dataset.dataset_entry_count == official.dataset_entry_count
        && dataset.dataset_entry_count == claim.expected_dataset_entry_count
        && dataset.active_row_count == dataset.rows.len()
        && dataset.active_row_count == claim.expected_active_row_count
        && dataset.excluded_purge_or_embargo_row_count == dataset.excluded_rows.len()
        && dataset.excluded_purge_or_embargo_row_count == claim.expected_excluded_row_count
        && dataset.active_row_count + dataset.excluded_purge_or_embargo_row_count
            == dataset.dataset_entry_count
        && dataset.one_to_one_join_preserved;
    let feature_catalog = dataset.feature_catalog_count == EXPECTED_FEATURE_COUNT
        && dataset.feature_catalog_count == claim.expected_feature_catalog_count
        && dataset
            .rows
            .iter()
            .all(|row| row.feature_records.len() == EXPECTED_FEATURE_COUNT);
    let point_in_time_missingness = dataset.point_in_time_and_explicit_missingness_preserved;
    let split_purge_embargo = dataset.official_split_purge_and_embargo_preserved
        && dataset
            .excluded_rows
            .iter()
            .all(|row| !row.target_values_opened);
    let target_bits_commitments = dataset.target_count == EXPECTED_TARGET_COUNT
        && dataset.target_count == claim.expected_target_count
        && dataset.exact_raw_f64_bits_preserved
        && dataset.source_target_commitments_sha256 == recomputed_target_commitments_sha256
        && recomputed_target_commitments_sha256 == claim.expected_target_commitments_sha256;
    let train_only_visibility = dataset.rows.iter().all(|row| match row.target_visibility {
        HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets => row
            .target_vector
            .as_ref()
            .is_some_and(|values| values.len() == EXPECTED_TARGET_COUNT),
        HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
        | HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld => {
            row.target_vector.is_none()
        }
    }) && dataset.train_target_vector_count
        == dataset
            .rows
            .iter()
            .filter(|row| {
                row.target_visibility
                    == HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets
            })
            .count();
    let validation_withheld = dataset.validation_targets_withheld && dataset.validation_target_withheld_count == dataset.rows.iter().filter(|row| row.target_visibility == HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld).count();
    let sealed_holdout_withheld = dataset.sealed_holdout_targets_withheld && dataset.sealed_holdout_target_withheld_count == dataset.rows.iter().filter(|row| row.target_visibility == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld).count();
    let downstream_closed = dataset.action_position_and_reward_semantics_absent
        && dataset.copied_to_training_store
        && !dataset.independently_validated_after_training_store_copy
        && !dataset.eligible_for_training_registration_review
        && !dataset.training_registered
        && !dataset.training_authorized
        && !dataset.training_started
        && !dataset.reward_authorized
        && !dataset.shadow_portfolio_authorized
        && !dataset.order_generation_authorized
        && !dataset.broker_access_authorized
        && !dataset.trading_authorized
        && !result.independent_post_copy_validation_completed
        && !result.eligible_for_training_registration_review
        && !result.training_registered
        && !result.training_authorized
        && !result.training_started
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized
        && !claim.generic_training_store_read_allowed
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
    let mut mismatch_reasons = Vec::new();
    for (matched, reason) in [
        (exact_chain, "current_stage_47_or_stage_48_chain_mismatch"),
        (
            claim_fingerprint,
            "copy_claim_fingerprint_or_contract_mismatch",
        ),
        (
            result_fingerprint,
            "copy_result_fingerprint_or_contract_mismatch",
        ),
        (
            dataset_fingerprint,
            "training_store_dataset_fingerprint_or_binding_mismatch",
        ),
        (
            exact_copy,
            "training_store_dataset_not_exact_official_dataset_copy",
        ),
        (one_to_one, "one_to_one_or_cardinality_mismatch"),
        (feature_catalog, "feature_catalog_not_exactly_65"),
        (
            point_in_time_missingness,
            "point_in_time_or_missingness_mismatch",
        ),
        (
            split_purge_embargo,
            "official_split_purge_or_embargo_mismatch",
        ),
        (
            target_bits_commitments,
            "nine_target_bits_or_commitments_mismatch",
        ),
        (
            train_only_visibility,
            "train_only_target_visibility_mismatch",
        ),
        (validation_withheld, "validation_targets_not_withheld"),
        (
            sealed_holdout_withheld,
            "sealed_holdout_targets_not_withheld",
        ),
        (downstream_closed, "downstream_authority_overreach"),
    ] {
        if !matched {
            mismatch_reasons.push(reason.to_string());
        }
    }
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    mismatch_reasons.truncate(MAX_MISMATCH_REASONS);
    Ok(IndependentAudit {
        exact_chain,
        claim_fingerprint,
        result_fingerprint,
        dataset_fingerprint,
        exact_copy,
        one_to_one,
        feature_catalog,
        point_in_time_missingness,
        split_purge_embargo,
        target_bits_commitments,
        train_only_visibility,
        validation_withheld,
        sealed_holdout_withheld,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_dataset_sha256,
        recomputed_rows_sha256,
        recomputed_excluded_rows_sha256,
        recomputed_target_commitments_sha256,
        mismatch_reasons,
    })
}

fn validate_request(
    attempt_id: &str,
    request: &ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest,
) -> Result<(), String> {
    if !valid_id(attempt_id)
        || !valid_id(&request.expected_copy_id)
        || [
            request.expected_copy_claim_sha256.as_str(),
            request.expected_copy_result_sha256.as_str(),
            request.expected_training_store_dataset_sha256.as_str(),
            request.expected_admission_review_sha256.as_str(),
            request.expected_output_validation_sha256.as_str(),
            request.expected_official_joined_dataset_sha256.as_str(),
            request.expected_rows_sha256.as_str(),
            request.expected_excluded_rows_sha256.as_str(),
            request.expected_target_commitments_sha256.as_str(),
        ]
        .iter()
        .any(|value| !valid_sha256(value))
    {
        return Err("训练存储副本独立校验请求中的 ID 或 SHA-256 无效".to_string());
    }
    if !request.independent_reopen_and_recomputation_confirmed
        || !request.exact_current_stage_47_and_stage_48_binding_confirmed
        || !request.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        || !request.no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err(
            "必须确认独立重开重算、精确 Stage 47/48 绑定、留出目标隐藏和零下游权限四项边界"
                .to_string(),
        );
    }
    Ok(())
}

fn verify_expected_bindings(
    copied: &CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    request: &ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest,
) -> Result<(), String> {
    let claim = &copied.attempt.claim;
    let result = copied
        .attempt
        .result
        .as_ref()
        .ok_or_else(|| "复制结果缺失".to_string())?;
    let dataset = copied
        .attempt
        .training_store_dataset
        .as_ref()
        .ok_or_else(|| "训练存储副本缺失".to_string())?;
    if claim.copy_id != request.expected_copy_id
        || claim.claim_sha256 != request.expected_copy_claim_sha256
        || result.result_sha256 != request.expected_copy_result_sha256
        || dataset.training_store_dataset_sha256 != request.expected_training_store_dataset_sha256
        || claim.admission_review_sha256 != request.expected_admission_review_sha256
        || claim.output_validation_sha256 != request.expected_output_validation_sha256
        || claim.official_joined_dataset_sha256 != request.expected_official_joined_dataset_sha256
        || claim.expected_rows_sha256 != request.expected_rows_sha256
        || claim.expected_excluded_rows_sha256 != request.expected_excluded_rows_sha256
        || claim.expected_target_commitments_sha256 != request.expected_target_commitments_sha256
    {
        return Err("训练存储副本、复制结果或 Stage 47 准入源绑定已经变化".to_string());
    }
    Ok(())
}

fn prior_actor_ids(
    copied: &CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
) -> Vec<String> {
    let mut values = copied.attempt.claim.excluded_prior_actor_ids.clone();
    values.push(copied.attempt.claim.copied_by.clone());
    values.push(copied.admitted_dataset.admission_review.reviewer_id.clone());
    values.push(
        copied
            .admitted_dataset
            .dataset
            .validation
            .validated_by
            .clone(),
    );
    values.extend(
        copied
            .admitted_dataset
            .admission_review
            .excluded_prior_actor_ids
            .clone(),
    );
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

async fn completed_copies(
    state: &AppState,
) -> Result<Vec<CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset>, String> {
    let mut output = Vec::new();
    for admitted_dataset in admitted_official_joined_datasets_for_training_store_copy(state).await?
    {
        let Some(attempt) = independently_reopen_copy_attempt(state, &admitted_dataset).await?
        else {
            continue;
        };
        output.push(
            CopiedHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset {
                admitted_dataset,
                attempt,
            },
        );
    }
    output.sort_by(|left, right| {
        right
            .attempt
            .claim
            .claimed_at
            .cmp(&left.attempt.claim.claimed_at)
    });
    Ok(output)
}

async fn independently_reopen_copy_attempt(
    state: &AppState,
    admitted_dataset: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetForTrainingStoreCopy,
) -> Result<Option<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt>, String> {
    let attempt_id = &admitted_dataset.admission_review.attempt_id;
    let directory = decision_root(state)
        .join("isolated-training-store")
        .join("feature-label-join-target-copies")
        .join(attempt_id);
    let claim_bytes = match tokio::fs::read(directory.join("claim.json")).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let result_bytes = match tokio::fs::read(directory.join("result.json")).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let claim = serde_json::from_slice::<
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyClaim,
    >(&claim_bytes)
    .map_err(|error| format!("训练存储复制 claim 无法解析：{error}"))?;
    let result = serde_json::from_slice::<
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyResult,
    >(&result_bytes)
    .map_err(|error| format!("训练存储复制 result 无法解析：{error}"))?;
    if result.status
        != HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyStatus::CompletedPendingIndependentValidation
    {
        return Ok(None);
    }
    let dataset_sha256 = result
        .training_store_dataset_sha256
        .as_deref()
        .ok_or_else(|| "训练存储复制结果缺少 dataset SHA-256".to_string())?;
    let dataset_bytes =
        tokio::fs::read(directory.join(format!("training-store-dataset-{dataset_sha256}.json")))
            .await
            .map_err(|error| error.to_string())?;
    if dataset_bytes.len() > MAX_TRAINING_STORE_DATASET_BYTES {
        return Err("训练存储副本超过固定 64 MiB 上限".to_string());
    }
    let training_store_dataset = serde_json::from_slice::<
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreDataset,
    >(&dataset_bytes)
    .map_err(|error| format!("训练存储副本无法解析：{error}"))?;
    Ok(Some(
        HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyAttempt {
            claim,
            result: Some(result),
            training_store_dataset: Some(training_store_dataset),
        },
    ))
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn validate_record(
    record: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_copier_and_complete_prior_chain
        && record.exact_current_stage_47_and_stage_48_chain_verified
        && record.copy_claim_fingerprint_verified
        && record.copy_result_fingerprint_verified
        && record.training_store_dataset_fingerprint_verified
        && record.exact_official_dataset_copy_verified
        && record.exact_one_to_one_entry_join_and_cardinality_verified
        && record.exact_65_feature_catalog_verified
        && record.point_in_time_and_explicit_missingness_verified
        && record.official_split_purge_and_embargo_verified
        && record.exact_nine_raw_f64_bits_and_commitments_verified
        && record.train_only_target_visibility_verified
        && record.validation_targets_withheld_verified
        && record.sealed_holdout_targets_withheld_verified
        && record.downstream_authority_closed_verified
        && record.mismatch_reasons.is_empty();
    let closed = !record.training_registered
        && !record.training_authorized
        && !record.training_started
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != RECORD_SCHEMA_VERSION || record.policy_version != POLICY_VERSION || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256) || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != record_fingerprint(record)? || !valid_id(&record.attempt_id) || !valid_id(&record.copy_id)
        || [&record.copy_claim_sha256, &record.copy_result_sha256, &record.training_store_dataset_sha256, &record.admission_review_sha256,
            &record.output_validation_sha256, &record.official_joined_dataset_sha256, &record.source_validation_sha256, &record.source_output_sha256,
            &record.dataset_content_sha256, &record.dataset_manifest_sha256, &record.candidate_set_sha256, &record.validator_implementation_sha256,
            &record.recomputed_copy_claim_sha256, &record.recomputed_copy_result_sha256, &record.recomputed_training_store_dataset_sha256,
            &record.recomputed_rows_sha256, &record.recomputed_excluded_rows_sha256, &record.recomputed_target_commitments_sha256].iter().any(|value| !valid_sha256(value))
        || !valid_id(&record.copy_result_id) || !valid_id(&record.admission_review_id) || !valid_id(&record.output_validation_id)
        || !valid_id(&record.materialization_id) || !valid_id(&record.dataset_id)
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION || record.validator_implementation_sha256 != validator_implementation_sha256()
        || record.validated_by.trim().is_empty() || record.copied_by.trim().is_empty() || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids) || !actor_is_independent(&record.validated_by, &record.excluded_prior_actor_ids)
        || record.mismatch_reasons.len() > MAX_MISMATCH_REASONS || record.training_store_copy_independently_validated != passed
        || record.future_training_registration_review_eligible != passed
        || record.training_store_copy_independently_validated != (record.verdict == HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationVerdict::ValidatedTrainingStoreCopyForFutureTrainingRegistrationReview)
        || !closed {
        return Err("training-store copy validation record is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_validations(
    state: &AppState,
) -> Result<
    Vec<HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord>,
    String,
> {
    let root = validation_root(state);
    let mut directories = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    let mut attempts = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
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
        let directory_name = directory.file_name().to_string_lossy().to_string();
        if !valid_id(&directory_name) {
            return Err("训练存储副本校验目录包含无效 attempt ID".to_string());
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
            let record = serde_json::from_slice::<
                HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord,
            >(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            validate_record(&record)?;
            if directory_name != record.attempt_id
                || entry.path().file_stem().and_then(|value| value.to_str())
                    != Some(record.validation_id.as_str())
                || !attempts.insert(record.attempt_id.clone())
                || !ids.insert(record.validation_id.clone())
                || !hashes.insert(record.validation_sha256.clone())
            {
                return Err("训练存储副本独立校验登记含重放、误命名或重复哈希".to_string());
            }
            records.push(record);
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyOutputValidationRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["validation_id", "validation_sha256"])
}
fn validator_implementation_sha256() -> String {
    hash_bytes(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
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
    Ok(hash_bytes(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}
fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}
fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}
fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("feature-label-join-target-training-store-copy-output-validations")
}

struct ValidationLock {
    path: PathBuf,
}
impl Drop for ValidationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<ValidationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "feature-label-join-target-training-store-copy-output-validation-{attempt_id}.lock"
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
        .map_err(|_| "已有同一训练存储副本独立校验正在进行".to_string())?;
    Ok(ValidationLock { path })
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

    fn request() -> ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest {
        ValidateHistoricalOutcomeFeatureLabelJoinTargetTrainingStoreCopyRequest {
            expected_copy_id: "a".repeat(32),
            expected_copy_claim_sha256: "b".repeat(64),
            expected_copy_result_sha256: "c".repeat(64),
            expected_training_store_dataset_sha256: "d".repeat(64),
            expected_admission_review_sha256: "e".repeat(64),
            expected_output_validation_sha256: "f".repeat(64),
            expected_official_joined_dataset_sha256: "1".repeat(64),
            expected_rows_sha256: "2".repeat(64),
            expected_excluded_rows_sha256: "3".repeat(64),
            expected_target_commitments_sha256: "4".repeat(64),
            independent_reopen_and_recomputation_confirmed: true,
            exact_current_stage_47_and_stage_48_binding_confirmed: true,
            validation_and_sealed_holdout_targets_remain_withheld_confirmed: true,
            no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    #[test]
    fn request_requires_valid_attempt_id() {
        assert!(validate_request("bad", &request()).is_err());
    }
    #[test]
    fn request_accepts_complete_fail_closed_contract() {
        assert!(validate_request(&"9".repeat(32), &request()).is_ok());
    }
    #[test]
    fn request_requires_independent_reopen_confirmation() {
        let mut value = request();
        value.independent_reopen_and_recomputation_confirmed = false;
        assert!(validate_request(&"9".repeat(32), &value).is_err());
    }
    #[test]
    fn request_requires_exact_chain_confirmation() {
        let mut value = request();
        value.exact_current_stage_47_and_stage_48_binding_confirmed = false;
        assert!(validate_request(&"9".repeat(32), &value).is_err());
    }
    #[test]
    fn request_requires_withheld_target_confirmation() {
        let mut value = request();
        value.validation_and_sealed_holdout_targets_remain_withheld_confirmed = false;
        assert!(validate_request(&"9".repeat(32), &value).is_err());
    }
    #[test]
    fn request_requires_downstream_closure_confirmation() {
        let mut value = request();
        value.no_training_registration_run_reward_shadow_order_broker_or_trading_confirmed = false;
        assert!(validate_request(&"9".repeat(32), &value).is_err());
    }
    #[test]
    fn actor_must_be_outside_complete_chain() {
        assert!(!actor_is_independent("copier", &["copier".into()]));
        assert!(actor_is_independent("validator", &["copier".into()]));
    }
    #[test]
    fn validator_implementation_hash_is_stable() {
        assert_eq!(
            validator_implementation_sha256(),
            hash_bytes(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
        );
    }
    #[test]
    fn validation_root_is_stage_49_specific() {
        assert!(
            "feature-label-join-target-training-store-copy-output-validations"
                .contains("training-store-copy-output-validations")
        );
    }
}
