//! Independent post-materialization validation of one official joined dataset.
//!
//! This validator intentionally reopens the Stage 45 files itself and never
//! calls the Stage 45 materializer or its artifact-validation helpers. It
//! recomputes the immutable claim, result, dataset, rows, excluded rows and
//! target-commitment fingerprints, then compares the artifact with the exact
//! current Stage 44 admitted candidate. Passing only permits a future
//! training-store-copy admission review; it does not copy, train, reward,
//! shadow, order, access a broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_candidate_admission_reviews::{
    AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
    admitted_join_target_candidates_for_official_joined_dataset_materialization,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_execution_attempts::HistoricalOutcomeFeatureLabelJoinTargetVisibility;
use super::historical_outcome_offline_dataset_feature_label_join_target_official_dataset_materializations::{
    HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult,
    HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-feature-label-join-target-official-dataset-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-official-dataset-output-validation-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-official-dataset-independent-output-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-official-joined-dataset-validator-v1-independent-reopen-and-canonical-recomputation";
const MATERIALIZATION_CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-official-dataset-materialization-claim-v1";
const MATERIALIZATION_RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-official-dataset-materialization-result-v1";
const OFFICIAL_DATASET_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-official-joined-dataset-v1";
const MATERIALIZATION_POLICY_VERSION: &str =
    "hone-feature-label-join-target-official-dataset-create-once-exact-copy-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const MAX_OFFICIAL_DATASET_BYTES: usize = 64 * 1024 * 1024;
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest {
    expected_materialization_id: String,
    expected_materialization_claim_sha256: String,
    expected_materialization_result_sha256: String,
    expected_official_joined_dataset_sha256: String,
    expected_admission_review_sha256: String,
    expected_source_validation_sha256: String,
    expected_source_output_sha256: String,
    expected_recomputed_rows_sha256: String,
    expected_recomputed_excluded_rows_sha256: String,
    expected_recomputed_target_commitments_sha256: String,
    independent_reopen_and_recomputation_confirmed: bool,
    exact_current_admitted_candidate_binding_confirmed: bool,
    validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    no_training_store_copy_training_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationVerdict {
    ValidatedOfficialJoinedDatasetForFutureTrainingStoreCopyAdmissionReview,
    FailedIndependentPostMaterializationValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub materialization_id: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_id: String,
    pub materialization_result_sha256: String,
    pub official_joined_dataset_sha256: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub source_validation_id: String,
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
    pub materialized_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_materializer_and_complete_prior_chain: bool,
    pub exact_current_admission_chain_verified: bool,
    pub materialization_claim_fingerprint_verified: bool,
    pub materialization_result_fingerprint_verified: bool,
    pub official_joined_dataset_fingerprint_verified: bool,
    pub exact_admitted_candidate_copy_verified: bool,
    pub exact_one_to_one_entry_join_and_cardinality_verified: bool,
    pub exact_65_feature_catalog_verified: bool,
    pub point_in_time_and_explicit_missingness_verified: bool,
    pub official_split_purge_and_embargo_verified: bool,
    pub exact_nine_raw_f64_bits_and_commitments_verified: bool,
    pub train_only_target_visibility_verified: bool,
    pub validation_targets_withheld_verified: bool,
    pub sealed_holdout_targets_withheld_verified: bool,
    pub downstream_authority_closed_verified: bool,
    pub recomputed_materialization_claim_sha256: String,
    pub recomputed_materialization_result_sha256: String,
    pub recomputed_official_joined_dataset_sha256: String,
    pub recomputed_rows_sha256: String,
    pub recomputed_excluded_rows_sha256: String,
    pub recomputed_target_commitments_sha256: String,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationVerdict,
    pub official_joined_dataset_independently_validated: bool,
    pub future_training_store_copy_admission_review_eligible: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
    pub admitted_candidate: AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
    pub claim: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    pub result: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult,
    pub official_joined_dataset: HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationItem {
    pub materialization: CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    pub validation:
        Option<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_official_joined_dataset_count: usize,
    pub failed_validation_count: usize,
    pub future_training_store_copy_admission_review_eligible_count: usize,
    pub validation_status: String,
    pub independent_post_materialization_validation_available: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationReadinessSummary
{
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_official_joined_dataset_count: usize,
    pub failed_validation_count: usize,
    pub future_training_store_copy_admission_review_eligible_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
    pub materialization: CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    pub validation: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_official_dataset_output_validations(
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
            warn!(%error, "official joined dataset output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果正式 joined dataset 独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_feature_label_join_target_official_dataset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest>,
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

pub(crate) async fn historical_outcome_feature_label_join_target_official_dataset_output_validation_readiness_summary(
    state: &AppState,
) -> Result<
    HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationReadinessSummary,
    String,
> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationReadinessSummary {
            validation_eligible_count: registry.validation_eligible_count,
            validation_count: registry.validation_count,
            independently_validated_official_joined_dataset_count: registry
                .independently_validated_official_joined_dataset_count,
            failed_validation_count: registry.failed_validation_count,
            future_training_store_copy_admission_review_eligible_count: registry
                .future_training_store_copy_admission_review_eligible_count,
            validation_status: registry.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_official_joined_datasets_for_training_store_copy_admission_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset>, String>
{
    let validations = read_validations(state).await?;
    let index = validations
        .into_iter()
        .map(|record| (record.attempt_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for materialization in completed_materializations(state).await? {
        let Some(validation) = index.get(&materialization.claim.attempt_id).cloned() else {
            continue;
        };
        validate_record(&validation)?;
        if !validation.official_joined_dataset_independently_validated
            || !validation.future_training_store_copy_admission_review_eligible
        {
            continue;
        }
        let audit = independently_audit(&materialization)?;
        if !audit.mismatch_reasons.is_empty()
            || validation.official_joined_dataset_sha256
                != materialization
                    .official_joined_dataset
                    .official_joined_dataset_sha256
            || validation.materialization_result_sha256 != materialization.result.result_sha256
        {
            return Err("已通过的正式 joined dataset 校验与当前不可变工件不再一致".to_string());
        }
        output.push(
            IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
                materialization,
                validation,
            },
        );
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
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry, String>
{
    let materializations = completed_materializations(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let items = materializations
        .into_iter()
        .map(|materialization| {
            let validation = index
                .get(materialization.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationItem {
                validation_eligible: validation.is_none(),
                materialization,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let independently_validated_official_joined_dataset_count = validations
        .iter()
        .filter(|record| record.official_joined_dataset_independently_validated)
        .count();
    let failed_validation_count =
        validations.len() - independently_validated_official_joined_dataset_count;
    let validation_status = if failed_validation_count > 0 {
        "official_joined_dataset_independent_validation_failed_fail_closed"
    } else if independently_validated_official_joined_dataset_count > 0 {
        "official_joined_dataset_independently_validated_waiting_training_store_copy_admission_review"
    } else if validation_eligible_count > 0 {
        "waiting_independent_official_joined_dataset_output_validation"
    } else {
        "waiting_completed_official_joined_dataset_materialization"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count: validations.len(),
        independently_validated_official_joined_dataset_count,
        failed_validation_count,
        future_training_store_copy_admission_review_eligible_count:
            independently_validated_official_joined_dataset_count,
        validation_status: validation_status.to_string(),
        independent_post_materialization_validation_available: true,
        copied_to_training_store: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 46 阶段由物化者和完整上游之外的独立管理员重新打开 claim/result/official joined dataset，独立重算工件、行、排除项和目标承诺摘要并核对精确当前准入候选。通过只开放未来训练库复制准入复核资格；当前不复制、不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.attempt_id == attempt_id)
    {
        return Err("这份正式 joined dataset 已形成不可变独立校验记录，不得重放".to_string());
    }
    let materialization = completed_materializations(state)
        .await?
        .into_iter()
        .find(|value| value.claim.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有完整正式 joined dataset 可供独立校验".to_string())?;
    verify_expected_bindings(&materialization, &request)?;
    let excluded_prior_actor_ids = prior_actor_ids(&materialization);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("正式 joined dataset 校验人不能是物化人或完整上游链任一角色".to_string());
    }
    let audit = independently_audit(&materialization)?;
    let passed = independent && audit.mismatch_reasons.is_empty();
    let review = &materialization.admitted_candidate.admission_review;
    let source_validation = &materialization.admitted_candidate.candidate.validation;
    let dataset = &materialization.official_joined_dataset;
    let mut record = HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        attempt_id: attempt_id.to_string(),
        materialization_id: materialization.claim.materialization_id.clone(),
        materialization_claim_sha256: materialization.claim.claim_sha256.clone(),
        materialization_result_id: materialization.result.result_id.clone(),
        materialization_result_sha256: materialization.result.result_sha256.clone(),
        official_joined_dataset_sha256: dataset.official_joined_dataset_sha256.clone(),
        admission_review_id: review.review_id.clone(),
        admission_review_sha256: review.review_sha256.clone(),
        source_validation_id: source_validation.validation_id.clone(),
        source_validation_sha256: source_validation.validation_sha256.clone(),
        source_output_sha256: source_validation.output_sha256.clone(),
        dataset_id: dataset.dataset_id.clone(),
        dataset_content_sha256: dataset.dataset_content_sha256.clone(),
        dataset_manifest_sha256: dataset.dataset_manifest_sha256.clone(),
        candidate_set_sha256: dataset.candidate_set_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        materialized_by: materialization.claim.materialized_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_materializer_and_complete_prior_chain: independent,
        exact_current_admission_chain_verified: audit.exact_current_admission_chain,
        materialization_claim_fingerprint_verified: audit.claim_fingerprint,
        materialization_result_fingerprint_verified: audit.result_fingerprint,
        official_joined_dataset_fingerprint_verified: audit.dataset_fingerprint,
        exact_admitted_candidate_copy_verified: audit.exact_candidate_copy,
        exact_one_to_one_entry_join_and_cardinality_verified: audit.one_to_one,
        exact_65_feature_catalog_verified: audit.feature_catalog,
        point_in_time_and_explicit_missingness_verified: audit.point_in_time_missingness,
        official_split_purge_and_embargo_verified: audit.split_purge_embargo,
        exact_nine_raw_f64_bits_and_commitments_verified: audit.target_bits_commitments,
        train_only_target_visibility_verified: audit.train_only_visibility,
        validation_targets_withheld_verified: audit.validation_withheld,
        sealed_holdout_targets_withheld_verified: audit.sealed_holdout_withheld,
        downstream_authority_closed_verified: audit.downstream_closed,
        recomputed_materialization_claim_sha256: audit.recomputed_claim_sha256,
        recomputed_materialization_result_sha256: audit.recomputed_result_sha256,
        recomputed_official_joined_dataset_sha256: audit.recomputed_dataset_sha256,
        recomputed_rows_sha256: audit.recomputed_rows_sha256,
        recomputed_excluded_rows_sha256: audit.recomputed_excluded_rows_sha256,
        recomputed_target_commitments_sha256: audit.recomputed_target_commitments_sha256,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationVerdict::ValidatedOfficialJoinedDatasetForFutureTrainingStoreCopyAdmissionReview
        } else {
            HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationVerdict::FailedIndependentPostMaterializationValidation
        },
        official_joined_dataset_independently_validated: passed,
        future_training_store_copy_admission_review_eligible: passed,
        copied_to_training_store: false,
        training_authorized: false,
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
    exact_current_admission_chain: bool,
    claim_fingerprint: bool,
    result_fingerprint: bool,
    dataset_fingerprint: bool,
    exact_candidate_copy: bool,
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
    materialization: &CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
) -> Result<IndependentAudit, String> {
    let admitted = &materialization.admitted_candidate;
    let review = &admitted.admission_review;
    let source_validation = &admitted.candidate.validation;
    let envelope = admitted
        .candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "当前准入候选缺少源输出信封".to_string())?;
    let claim = &materialization.claim;
    let result = &materialization.result;
    let dataset = &materialization.official_joined_dataset;
    let recomputed_claim_sha256 =
        fingerprint_without(claim, &["materialization_id", "claim_sha256"])?;
    let recomputed_result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    let recomputed_dataset_sha256 =
        fingerprint_without(dataset, &["official_joined_dataset_sha256"])?;
    let recomputed_rows_sha256 = hash_serializable(&dataset.rows)?;
    let recomputed_excluded_rows_sha256 = hash_serializable(&dataset.excluded_rows)?;
    let mut target_commitments = dataset
        .rows
        .iter()
        .map(|row| {
            (
                row.dataset_entry_id.clone(),
                row.target_commitment_sha256.clone(),
            )
        })
        .collect::<Vec<_>>();
    target_commitments.sort();
    let recomputed_target_commitments_sha256 = hash_serializable(&target_commitments)?;
    let exact_current_admission_chain = claim.attempt_id == review.attempt_id
        && claim.admission_review_id == review.review_id
        && claim.admission_review_sha256 == review.review_sha256
        && claim.validation_id == source_validation.validation_id
        && claim.validation_sha256 == source_validation.validation_sha256
        && claim.source_claim_sha256 == review.claim_sha256
        && claim.source_result_id == review.result_id
        && claim.source_result_sha256 == review.result_sha256
        && claim.source_output_sha256 == review.output_sha256
        && claim.authorization_review_id == review.authorization_review_id
        && claim.authorization_review_sha256 == review.authorization_review_sha256
        && claim.isolated_runner_id == review.isolated_runner_id
        && claim.isolated_runner_spec_sha256 == review.isolated_runner_spec_sha256
        && claim.implementation_id == review.implementation_id
        && claim.implementation_sha256 == review.implementation_sha256
        && claim.specification_id == review.specification_id
        && claim.specification_sha256 == review.specification_sha256
        && claim.join_specification_sha256 == review.join_specification_sha256
        && claim.target_specification_sha256 == review.target_specification_sha256
        && claim.split_manifest_sha256 == review.split_manifest_sha256
        && claim.feature_bundle_sha256 == review.feature_bundle_sha256
        && claim.combined_artifact_sha256 == review.combined_artifact_sha256
        && claim.dataset_id == review.dataset_id
        && claim.dataset_content_sha256 == review.dataset_content_sha256
        && claim.dataset_manifest_sha256 == review.dataset_manifest_sha256
        && claim.candidate_set_sha256 == review.candidate_set_sha256
        && claim.expected_recomputed_rows_sha256 == source_validation.recomputed_rows_sha256
        && claim.expected_recomputed_excluded_rows_sha256
            == source_validation.recomputed_excluded_rows_sha256
        && claim.expected_recomputed_target_commitments_sha256
            == source_validation.recomputed_target_commitments_sha256
        && claim.expected_dataset_entry_count == review.dataset_entry_count
        && claim.expected_active_row_count == review.active_candidate_row_count
        && claim.expected_excluded_row_count == review.excluded_purge_or_embargo_row_count
        && claim.expected_feature_catalog_count == EXPECTED_FEATURE_COUNT
        && claim.expected_feature_catalog_count == review.feature_catalog_count
        && claim.expected_target_count == EXPECTED_TARGET_COUNT
        && claim.expected_target_count == review.target_count;
    let claim_fingerprint = claim.schema_version == MATERIALIZATION_CLAIM_SCHEMA_VERSION
        && claim.policy_version == MATERIALIZATION_POLICY_VERSION
        && claim.claim_sha256 == recomputed_claim_sha256
        && claim.materialization_id == recomputed_claim_sha256[..32]
        && claim.claim_consumed
        && claim.official_joined_dataset_materialization_started
        && claim.independent_post_materialization_validation_required_confirmed;
    let result_fingerprint = result.schema_version == MATERIALIZATION_RESULT_SCHEMA_VERSION
        && result.policy_version == MATERIALIZATION_POLICY_VERSION
        && result.result_sha256 == recomputed_result_sha256
        && result.result_id == recomputed_result_sha256[..32]
        && result.materialization_id == claim.materialization_id
        && result.claim_sha256 == claim.claim_sha256
        && result.status
            == HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus::CompletedPendingIndependentValidation
        && result.error.is_none()
        && result.official_joined_dataset_sha256.as_deref()
            == Some(dataset.official_joined_dataset_sha256.as_str())
        && result.official_joined_dataset_bytes > 0
        && result.official_joined_dataset_bytes <= MAX_OFFICIAL_DATASET_BYTES
        && result.official_joined_dataset_created
        && result.exact_admitted_candidate_copy_completed;
    let dataset_fingerprint = dataset.schema_version == OFFICIAL_DATASET_SCHEMA_VERSION
        && dataset.official_joined_dataset_sha256 == recomputed_dataset_sha256
        && dataset.materialization_id == claim.materialization_id
        && dataset.attempt_id == claim.attempt_id
        && dataset.admission_review_id == claim.admission_review_id
        && dataset.admission_review_sha256 == claim.admission_review_sha256
        && dataset.validation_id == claim.validation_id
        && dataset.validation_sha256 == claim.validation_sha256
        && dataset.source_claim_sha256 == claim.source_claim_sha256
        && dataset.source_result_id == claim.source_result_id
        && dataset.source_result_sha256 == claim.source_result_sha256
        && dataset.source_output_sha256 == claim.source_output_sha256
        && dataset.authorization_review_id == claim.authorization_review_id
        && dataset.authorization_review_sha256 == claim.authorization_review_sha256
        && dataset.isolated_runner_id == claim.isolated_runner_id
        && dataset.isolated_runner_spec_sha256 == claim.isolated_runner_spec_sha256
        && dataset.implementation_id == claim.implementation_id
        && dataset.implementation_sha256 == claim.implementation_sha256
        && dataset.specification_id == claim.specification_id
        && dataset.specification_sha256 == claim.specification_sha256
        && dataset.join_specification_sha256 == claim.join_specification_sha256
        && dataset.target_specification_sha256 == claim.target_specification_sha256
        && dataset.split_manifest_sha256 == claim.split_manifest_sha256
        && dataset.feature_bundle_sha256 == claim.feature_bundle_sha256
        && dataset.combined_artifact_sha256 == claim.combined_artifact_sha256
        && dataset.dataset_id == claim.dataset_id
        && dataset.dataset_content_sha256 == claim.dataset_content_sha256
        && dataset.dataset_manifest_sha256 == claim.dataset_manifest_sha256
        && dataset.candidate_set_sha256 == claim.candidate_set_sha256
        && dataset.source_recomputed_rows_sha256 == claim.expected_recomputed_rows_sha256
        && dataset.source_recomputed_excluded_rows_sha256
            == claim.expected_recomputed_excluded_rows_sha256
        && dataset.target_commitments_sha256 == claim.expected_recomputed_target_commitments_sha256;
    let exact_candidate_copy = dataset.rows == envelope.rows
        && dataset.excluded_rows == envelope.excluded_rows
        && dataset.exact_admitted_candidate_copy
        && recomputed_rows_sha256 == source_validation.recomputed_rows_sha256
        && recomputed_excluded_rows_sha256 == source_validation.recomputed_excluded_rows_sha256;
    let one_to_one = dataset.dataset_entry_count == review.dataset_entry_count
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
        && dataset.target_commitments_sha256 == recomputed_target_commitments_sha256
        && recomputed_target_commitments_sha256
            == source_validation.recomputed_target_commitments_sha256;
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
    let validation_withheld = dataset.validation_targets_withheld
        && dataset.validation_target_withheld_count
            == dataset
                .rows
                .iter()
                .filter(|row| {
                    row.target_visibility
                        == HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
                })
                .count();
    let sealed_holdout_withheld = dataset.sealed_holdout_targets_withheld
        && dataset.sealed_holdout_target_withheld_count
            == dataset
                .rows
                .iter()
                .filter(|row| {
                    row.target_visibility
                        == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
                })
                .count();
    let downstream_closed = dataset.action_position_and_reward_semantics_absent
        && dataset.official_joined_dataset_created
        && !dataset.independently_validated_after_materialization
        && !dataset.eligible_for_training_store_copy
        && !dataset.copied_to_training_store
        && !dataset.training_authorized
        && !dataset.reward_authorized
        && !dataset.shadow_portfolio_authorized
        && !dataset.order_generation_authorized
        && !dataset.broker_access_authorized
        && !dataset.trading_authorized
        && !result.independent_post_materialization_validation_completed
        && !result.eligible_for_training_store_copy
        && !result.copied_to_training_store
        && !result.training_authorized
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    let mut mismatch_reasons = Vec::new();
    for (matched, reason) in [
        (
            exact_current_admission_chain,
            "current_admission_chain_mismatch",
        ),
        (
            claim_fingerprint,
            "materialization_claim_fingerprint_or_contract_mismatch",
        ),
        (
            result_fingerprint,
            "materialization_result_fingerprint_or_contract_mismatch",
        ),
        (
            dataset_fingerprint,
            "official_joined_dataset_fingerprint_or_binding_mismatch",
        ),
        (
            exact_candidate_copy,
            "official_joined_dataset_not_exact_admitted_candidate_copy",
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
        exact_current_admission_chain,
        claim_fingerprint,
        result_fingerprint,
        dataset_fingerprint,
        exact_candidate_copy,
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
    request: &ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
) -> Result<(), String> {
    if !valid_id(attempt_id)
        || !valid_id(&request.expected_materialization_id)
        || !valid_sha256(&request.expected_materialization_claim_sha256)
        || !valid_sha256(&request.expected_materialization_result_sha256)
        || !valid_sha256(&request.expected_official_joined_dataset_sha256)
        || !valid_sha256(&request.expected_admission_review_sha256)
        || !valid_sha256(&request.expected_source_validation_sha256)
        || !valid_sha256(&request.expected_source_output_sha256)
        || !valid_sha256(&request.expected_recomputed_rows_sha256)
        || !valid_sha256(&request.expected_recomputed_excluded_rows_sha256)
        || !valid_sha256(&request.expected_recomputed_target_commitments_sha256)
    {
        return Err("正式 joined dataset 独立校验请求中的 ID 或 SHA-256 无效".to_string());
    }
    if !request.independent_reopen_and_recomputation_confirmed
        || !request.exact_current_admitted_candidate_binding_confirmed
        || !request.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        || !request.no_training_store_copy_training_or_trading_confirmed
    {
        return Err("必须确认独立重开重算、精确准入绑定、目标隐藏和零下游权限四项边界".to_string());
    }
    Ok(())
}

fn verify_expected_bindings(
    materialization: &CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    request: &ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
) -> Result<(), String> {
    let review = &materialization.admitted_candidate.admission_review;
    let source_validation = &materialization.admitted_candidate.candidate.validation;
    if materialization.claim.materialization_id != request.expected_materialization_id
        || materialization.claim.claim_sha256 != request.expected_materialization_claim_sha256
        || materialization.result.result_sha256 != request.expected_materialization_result_sha256
        || materialization
            .official_joined_dataset
            .official_joined_dataset_sha256
            != request.expected_official_joined_dataset_sha256
        || review.review_sha256 != request.expected_admission_review_sha256
        || source_validation.validation_sha256 != request.expected_source_validation_sha256
        || source_validation.output_sha256 != request.expected_source_output_sha256
        || source_validation.recomputed_rows_sha256 != request.expected_recomputed_rows_sha256
        || source_validation.recomputed_excluded_rows_sha256
            != request.expected_recomputed_excluded_rows_sha256
        || source_validation.recomputed_target_commitments_sha256
            != request.expected_recomputed_target_commitments_sha256
    {
        return Err("正式 joined dataset、物化结果或当前准入源候选绑定已经变化".to_string());
    }
    Ok(())
}

async fn completed_materializations(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset>, String> {
    let candidates =
        admitted_join_target_candidates_for_official_joined_dataset_materialization(state).await?;
    let mut output = Vec::new();
    for candidate in candidates {
        if let Some(value) = read_completed_materialization(state, candidate).await? {
            output.push(value);
        }
    }
    output.sort_by(|left, right| right.result.completed_at.cmp(&left.result.completed_at));
    Ok(output)
}

async fn read_completed_materialization(
    state: &AppState,
    admitted_candidate: AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
) -> Result<Option<CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset>, String> {
    let attempt_id = &admitted_candidate.admission_review.attempt_id;
    let directory = materialization_directory(state, attempt_id);
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
        HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    >(&claim_bytes)
    .map_err(|error| format!("正式 joined dataset claim 无法解析：{error}"))?;
    let result = serde_json::from_slice::<
        HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult,
    >(&result_bytes)
    .map_err(|error| format!("正式 joined dataset result 无法解析：{error}"))?;
    if result.status
        != HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus::CompletedPendingIndependentValidation
    {
        return Ok(None);
    }
    let dataset_sha256 = result
        .official_joined_dataset_sha256
        .as_deref()
        .ok_or_else(|| "正式物化结果缺少 joined dataset SHA-256".to_string())?;
    let bytes =
        tokio::fs::read(directory.join(format!("official-joined-dataset-{dataset_sha256}.json")))
            .await
            .map_err(|error| error.to_string())?;
    if bytes.len() > MAX_OFFICIAL_DATASET_BYTES {
        return Err("正式 joined dataset 超过固定 64 MiB 上限".to_string());
    }
    let official_joined_dataset =
        serde_json::from_slice::<HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset>(&bytes)
            .map_err(|error| format!("正式 joined dataset 无法解析：{error}"))?;
    Ok(Some(
        CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
            admitted_candidate,
            claim,
            result,
            official_joined_dataset,
        },
    ))
}

fn prior_actor_ids(
    materialization: &CompletedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
) -> Vec<String> {
    let mut values = materialization.claim.excluded_prior_actor_ids.clone();
    values.push(materialization.claim.materialized_by.clone());
    values.push(
        materialization
            .admitted_candidate
            .admission_review
            .reviewer_id
            .clone(),
    );
    values.push(
        materialization
            .admitted_candidate
            .candidate
            .validation
            .validated_by
            .clone(),
    );
    values.extend(
        materialization
            .admitted_candidate
            .admission_review
            .excluded_prior_actor_ids
            .clone(),
    );
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded_actor_ids: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded_actor_ids.iter().all(|value| value != actor_id)
}

fn validate_record(
    record: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_materializer_and_complete_prior_chain
        && record.exact_current_admission_chain_verified
        && record.materialization_claim_fingerprint_verified
        && record.materialization_result_fingerprint_verified
        && record.official_joined_dataset_fingerprint_verified
        && record.exact_admitted_candidate_copy_verified
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
    let downstream_closed = !record.copied_to_training_store
        && !record.training_authorized
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256)
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != record_fingerprint(record)?
        || !valid_id(&record.attempt_id)
        || !valid_id(&record.materialization_id)
        || !valid_sha256(&record.materialization_claim_sha256)
        || !valid_id(&record.materialization_result_id)
        || !valid_sha256(&record.materialization_result_sha256)
        || !valid_sha256(&record.official_joined_dataset_sha256)
        || !valid_id(&record.admission_review_id)
        || !valid_sha256(&record.admission_review_sha256)
        || !valid_id(&record.source_validation_id)
        || !valid_sha256(&record.source_validation_sha256)
        || !valid_sha256(&record.source_output_sha256)
        || !valid_id(&record.dataset_id)
        || !valid_sha256(&record.dataset_content_sha256)
        || !valid_sha256(&record.dataset_manifest_sha256)
        || !valid_sha256(&record.candidate_set_sha256)
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || record.validated_by.trim().is_empty()
        || record.materialized_by.trim().is_empty()
        || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || !actor_is_independent(&record.validated_by, &record.excluded_prior_actor_ids)
        || !valid_sha256(&record.recomputed_materialization_claim_sha256)
        || !valid_sha256(&record.recomputed_materialization_result_sha256)
        || !valid_sha256(&record.recomputed_official_joined_dataset_sha256)
        || !valid_sha256(&record.recomputed_rows_sha256)
        || !valid_sha256(&record.recomputed_excluded_rows_sha256)
        || !valid_sha256(&record.recomputed_target_commitments_sha256)
        || record.mismatch_reasons.len() > MAX_MISMATCH_REASONS
        || record.official_joined_dataset_independently_validated != passed
        || record.future_training_store_copy_admission_review_eligible != passed
        || record.official_joined_dataset_independently_validated
            != (record.verdict
                == HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationVerdict::ValidatedOfficialJoinedDatasetForFutureTrainingStoreCopyAdmissionReview)
        || !downstream_closed
    {
        return Err("official joined dataset validation record is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord>, String>
{
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
            return Err("正式 joined dataset 校验目录包含无效 attempt ID".to_string());
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
                HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord,
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
                return Err("正式 joined dataset 独立校验登记含重放、误命名或重复哈希".to_string());
            }
            records.push(record);
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord,
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
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(hash_bytes(&bytes))
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

fn materialization_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("feature-label-join-target-official-dataset-materializations")
        .join(attempt_id)
}

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("feature-label-join-target-official-dataset-output-validations")
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
        "feature-label-join-target-official-dataset-output-validation-{attempt_id}.lock"
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
        .map_err(|_| "已有同一正式 joined dataset 独立校验正在进行".to_string())?;
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

    fn minimal_record()
    -> HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord {
        let mut record = HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationRecord {
            schema_version: RECORD_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            validation_id: String::new(),
            validation_sha256: String::new(),
            attempt_id: "1".repeat(32),
            materialization_id: "2".repeat(32),
            materialization_claim_sha256: "3".repeat(64),
            materialization_result_id: "4".repeat(32),
            materialization_result_sha256: "5".repeat(64),
            official_joined_dataset_sha256: "6".repeat(64),
            admission_review_id: "7".repeat(32),
            admission_review_sha256: "8".repeat(64),
            source_validation_id: "9".repeat(32),
            source_validation_sha256: "a".repeat(64),
            source_output_sha256: "b".repeat(64),
            dataset_id: "c".repeat(32),
            dataset_content_sha256: "d".repeat(64),
            dataset_manifest_sha256: "e".repeat(64),
            candidate_set_sha256: "f".repeat(64),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.into(),
            validator_implementation_sha256: validator_implementation_sha256(),
            validated_at: Utc::now(),
            validated_by: "independent-validator".into(),
            materialized_by: "materializer".into(),
            excluded_prior_actor_ids: vec!["admitter".into(), "materializer".into()],
            validator_independent_from_materializer_and_complete_prior_chain: true,
            exact_current_admission_chain_verified: true,
            materialization_claim_fingerprint_verified: true,
            materialization_result_fingerprint_verified: true,
            official_joined_dataset_fingerprint_verified: true,
            exact_admitted_candidate_copy_verified: true,
            exact_one_to_one_entry_join_and_cardinality_verified: true,
            exact_65_feature_catalog_verified: true,
            point_in_time_and_explicit_missingness_verified: true,
            official_split_purge_and_embargo_verified: true,
            exact_nine_raw_f64_bits_and_commitments_verified: true,
            train_only_target_visibility_verified: true,
            validation_targets_withheld_verified: true,
            sealed_holdout_targets_withheld_verified: true,
            downstream_authority_closed_verified: true,
            recomputed_materialization_claim_sha256: "1".repeat(64),
            recomputed_materialization_result_sha256: "2".repeat(64),
            recomputed_official_joined_dataset_sha256: "3".repeat(64),
            recomputed_rows_sha256: "4".repeat(64),
            recomputed_excluded_rows_sha256: "5".repeat(64),
            recomputed_target_commitments_sha256: "6".repeat(64),
            mismatch_reasons: Vec::new(),
            verdict: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationVerdict::ValidatedOfficialJoinedDatasetForFutureTrainingStoreCopyAdmissionReview,
            official_joined_dataset_independently_validated: true,
            future_training_store_copy_admission_review_eligible: true,
            copied_to_training_store: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        record
    }

    #[test]
    fn validator_must_be_outside_materializer_and_prior_actor_set() {
        let actors = vec!["admitter".to_string(), "materializer".to_string()];
        assert!(actor_is_independent("independent-validator", &actors));
        assert!(!actor_is_independent("materializer", &actors));
    }

    #[test]
    fn valid_record_opens_only_future_copy_admission_review() {
        let record = minimal_record();
        validate_record(&record).unwrap();
        assert!(record.future_training_store_copy_admission_review_eligible);
        assert!(!record.copied_to_training_store);
        assert!(!record.training_authorized);
        assert!(!record.trading_authorized);
    }

    #[test]
    fn validation_hash_binds_verdict() {
        let record = minimal_record();
        let first = record.validation_sha256.clone();
        let mut changed = record;
        changed.verdict = HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationVerdict::FailedIndependentPostMaterializationValidation;
        assert_ne!(first, record_fingerprint(&changed).unwrap());
    }

    #[test]
    fn failed_validation_cannot_open_future_copy_review() {
        let mut record = minimal_record();
        record.exact_65_feature_catalog_verified = false;
        record.mismatch_reasons = vec!["feature_catalog_not_exactly_65".into()];
        record.verdict = HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetOutputValidationVerdict::FailedIndependentPostMaterializationValidation;
        record.official_joined_dataset_independently_validated = false;
        record.future_training_store_copy_admission_review_eligible = false;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        validate_record(&record).unwrap();
    }

    #[test]
    fn passed_record_rejects_training_authority_tamper() {
        let mut record = minimal_record();
        record.training_authorized = true;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn passed_record_requires_all_structural_proofs() {
        let mut record = minimal_record();
        record.validation_targets_withheld_verified = false;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn excluded_actor_ids_are_strictly_sorted_and_unique() {
        assert!(sorted_unique(&["a".into(), "b".into()]));
        assert!(!sorted_unique(&["b".into(), "a".into()]));
        assert!(!sorted_unique(&["a".into(), "a".into()]));
    }

    #[test]
    fn canonical_hash_detects_row_or_commitment_tamper() {
        let rows = vec![("entry".to_string(), "a".repeat(64))];
        let first = hash_serializable(&rows).unwrap();
        let changed = vec![("entry".to_string(), "b".repeat(64))];
        assert_ne!(first, hash_serializable(&changed).unwrap());
    }

    #[test]
    fn request_requires_all_four_boundary_confirmations() {
        let request = ValidateHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest {
            expected_materialization_id: "1".repeat(32),
            expected_materialization_claim_sha256: "2".repeat(64),
            expected_materialization_result_sha256: "3".repeat(64),
            expected_official_joined_dataset_sha256: "4".repeat(64),
            expected_admission_review_sha256: "5".repeat(64),
            expected_source_validation_sha256: "6".repeat(64),
            expected_source_output_sha256: "7".repeat(64),
            expected_recomputed_rows_sha256: "8".repeat(64),
            expected_recomputed_excluded_rows_sha256: "9".repeat(64),
            expected_recomputed_target_commitments_sha256: "a".repeat(64),
            independent_reopen_and_recomputation_confirmed: true,
            exact_current_admitted_candidate_binding_confirmed: true,
            validation_and_sealed_holdout_targets_remain_withheld_confirmed: true,
            no_training_store_copy_training_or_trading_confirmed: false,
        };
        assert!(validate_request(&"b".repeat(32), &request).is_err());
    }
}
