//! Independent post-materialization validation of the official offline split
//! manifest and feature bundle.
//!
//! This validator deliberately does not call the Stage 34 materializer or its
//! validation helpers. It reopens the immutable claim, result and official
//! artifacts, recomputes their fingerprints, and compares every copied field
//! with the exact current admitted source candidate. Passing only permits a
//! future join/target-governance specification to be registered; it does not
//! join labels, assign a target, train, reward, shadow, order, access a broker
//! or trade.

use std::collections::HashMap;
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
};
use super::historical_outcome_offline_dataset_transformation_official_artifact_materializations::{
    HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult,
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-transformation-official-artifact-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-official-artifact-output-validation-v1";
const POLICY_VERSION: &str =
    "hone-offline-transformation-official-artifact-independent-output-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-official-artifact-validator-v1-independent-field-and-fingerprint-recomputation";
const MATERIALIZATION_POLICY_VERSION: &str =
    "hone-offline-transformation-official-artifact-create-once-exact-copy-v1";
const MATERIALIZATION_CLAIM_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-transformation-official-artifact-materialization-claim-v1";
const MATERIALIZATION_RESULT_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-transformation-official-artifact-materialization-result-v1";
const SPLIT_MANIFEST_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-official-split-manifest-v1";
const FEATURE_BUNDLE_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-official-feature-bundle-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const MAX_TOTAL_ARTIFACT_BYTES: usize = 32 * 1024 * 1024;
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest {
    expected_materialization_id: String,
    expected_materialization_claim_sha256: String,
    expected_materialization_result_sha256: String,
    expected_admission_review_sha256: String,
    expected_source_validation_sha256: String,
    expected_source_output_sha256: String,
    expected_split_manifest_sha256: String,
    expected_feature_bundle_sha256: String,
    expected_combined_artifact_sha256: String,
    exact_artifact_pair_binding_confirmed: bool,
    independent_validator_confirmed: bool,
    no_join_target_training_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationVerdict
{
    ValidatedExactOfficialArtifactPair,
    FailedOfficialArtifactStructureOrBindingMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord
{
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub transformation_attempt_id: String,
    pub materialization_id: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_id: String,
    pub materialization_result_sha256: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub source_validation_id: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub transformation_spec_sha256: String,
    pub split_specification_sha256: String,
    pub feature_specification_sha256: String,
    pub split_manifest_sha256: String,
    pub feature_bundle_sha256: String,
    pub combined_artifact_sha256: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub materialized_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_materializer_and_complete_prior_chain: bool,
    pub exact_current_admission_and_source_candidate_verified: bool,
    pub materialization_claim_fingerprint_verified: bool,
    pub materialization_result_fingerprint_verified: bool,
    pub split_manifest_fingerprint_verified: bool,
    pub feature_bundle_fingerprint_verified: bool,
    pub combined_artifact_fingerprint_verified: bool,
    pub exact_split_candidate_copy_verified: bool,
    pub exact_feature_candidate_copy_verified: bool,
    pub sealed_holdout_withholding_verified: bool,
    pub explicit_missingness_and_exclusion_verified: bool,
    pub downstream_authority_closed_verified: bool,
    pub recomputed_materialization_claim_sha256: String,
    pub recomputed_materialization_result_sha256: String,
    pub recomputed_split_manifest_sha256: String,
    pub recomputed_feature_bundle_sha256: String,
    pub recomputed_combined_artifact_sha256: String,
    pub mismatch_reasons: Vec<String>,
    pub verdict:
        HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationVerdict,
    pub official_artifact_pair_independently_validated: bool,
    pub future_feature_label_join_specification_registration_eligible: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assigned: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair {
    pub admitted_candidate: AdmittedHistoricalOutcomeOfflineTransformationCandidate,
    pub claim: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    pub result: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult,
    pub split_manifest: HistoricalOutcomeOfflineDatasetOfficialSplitManifest,
    pub feature_bundle: HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationItem
{
    pub artifact_pair: CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair,
    pub validation:
        Option<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry
{
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items:
        Vec<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_artifact_pair_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
    pub independent_official_artifact_validation_enabled: bool,
    pub future_join_specification_registration_enabled: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationReadinessSummary
{
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_artifact_pair_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair {
    pub artifact_pair: CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair,
    pub validation:
        HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_official_artifact_output_validations(
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
            warn!(%error, "official artifact output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线转换正式工件独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_offline_dataset_transformation_official_artifacts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<
        ValidateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
    >,
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

pub(crate) async fn historical_outcome_offline_dataset_transformation_official_artifact_output_validation_readiness_summary(
    state: &AppState,
) -> Result<
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationReadinessSummary,
    String,
> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationReadinessSummary {
        validation_eligible_count: registry.validation_eligible_count,
        validation_count: registry.validation_count,
        independently_validated_artifact_pair_count: registry.independently_validated_artifact_pair_count,
        failed_validation_count: registry.failed_validation_count,
        validation_status: registry.validation_status,
    })
}

pub(crate) async fn independently_validated_official_artifact_pairs_for_join_target_specification(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair>, String>
{
    let validations = read_validations(state).await?;
    let validation_index = validations
        .into_iter()
        .map(|record| (record.transformation_attempt_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut validated_pairs = Vec::new();
    for artifact_pair in completed_official_artifact_pairs(state).await? {
        let attempt_id = artifact_pair.claim.transformation_attempt_id.clone();
        let Some(validation) = validation_index.get(&attempt_id).cloned() else {
            continue;
        };
        validate_record(&validation)?;
        if !validation.official_artifact_pair_independently_validated
            || !validation.future_feature_label_join_specification_registration_eligible
        {
            continue;
        }
        let audit = independently_audit_pair(&artifact_pair)?;
        let exact_validation_binding = validation.materialization_id
            == artifact_pair.claim.materialization_id
            && validation.materialization_claim_sha256 == artifact_pair.claim.claim_sha256
            && validation.materialization_result_id == artifact_pair.result.result_id
            && validation.materialization_result_sha256 == artifact_pair.result.result_sha256
            && validation.split_manifest_sha256 == artifact_pair.split_manifest.manifest_sha256
            && validation.feature_bundle_sha256
                == artifact_pair.feature_bundle.feature_bundle_sha256
            && artifact_pair.result.combined_artifact_sha256.as_deref()
                == Some(validation.combined_artifact_sha256.as_str());
        if !exact_validation_binding || !audit.mismatch_reasons.is_empty() {
            return Err("已通过的正式工件校验与当前工件或准入源候选不再精确一致".to_string());
        }
        validated_pairs.push(
            IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair {
                artifact_pair,
                validation,
            },
        );
    }
    validated_pairs.sort_by(|left, right| {
        right
            .validation
            .validated_at
            .cmp(&left.validation.validated_at)
    });
    Ok(validated_pairs)
}

async fn build_registry(
    state: &AppState,
) -> Result<
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry,
    String,
> {
    let pairs = completed_official_artifact_pairs(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.transformation_attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let items = pairs
        .into_iter()
        .map(|artifact_pair| {
            let validation = index
                .get(artifact_pair.claim.transformation_attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationItem {
                validation_eligible: validation.is_none(),
                artifact_pair,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let validation_count = validations.len();
    let independently_validated_artifact_pair_count = items
        .iter()
        .filter(|item| {
            item.validation
                .as_ref()
                .is_some_and(|record| record.official_artifact_pair_independently_validated)
        })
        .count();
    let failed_validation_count = validations
        .iter()
        .filter(|record| !record.official_artifact_pair_independently_validated)
        .count();
    let validation_status = if failed_validation_count > 0 {
        "official_artifact_output_validation_failed_fail_closed"
    } else if independently_validated_artifact_pair_count > 0 {
        "official_artifact_pair_independently_validated_waiting_join_target_governance"
    } else if validation_eligible_count > 0 {
        "waiting_independent_official_artifact_output_validation"
    } else {
        "waiting_completed_official_artifact_materialization"
    };
    Ok(HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count,
        independently_validated_artifact_pair_count,
        failed_validation_count,
        validation_status: validation_status.to_string(),
        independent_official_artifact_validation_enabled: true,
        future_join_specification_registration_enabled:
            independently_validated_artifact_pair_count > 0,
        feature_join_enabled: false,
        semantic_target_assignment_enabled: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 35 阶段由另一名独立管理员重新读取正式 claim/result/manifest/feature bundle，并独立重算五类摘要、逐字段对照精确当前源候选。通过只开放未来 join/target 治理规范登记资格；当前不 join、不定义目标、不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
) -> Result<
    HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord,
    String,
> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.transformation_attempt_id == attempt_id)
    {
        return Err("这组正式工件已经形成不可变独立校验记录，不得重放".to_string());
    }
    let pair = completed_official_artifact_pairs(state)
        .await?
        .into_iter()
        .find(|pair| pair.claim.transformation_attempt_id == attempt_id)
        .ok_or_else(|| "当前没有这组完整正式 manifest/feature bundle 可供独立校验".to_string())?;
    verify_expected_bindings(&pair, &request)?;
    let excluded_prior_actor_ids = prior_actor_ids(&pair);
    let independent = actor_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "正式工件独立校验人不能是物化人、准入人、此前校验/执行或完整上游角色".to_string(),
        );
    }
    let audit = independently_audit_pair(&pair)?;
    let validated = independent && audit.mismatch_reasons.is_empty();
    let candidate = &pair.admitted_candidate;
    let review = &candidate.admission_review;
    let source_validation = &candidate.candidate.validation;
    let result = &pair.result;
    let mut record =
        HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord {
            schema_version: RECORD_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            validation_id: String::new(),
            validation_sha256: String::new(),
            transformation_attempt_id: attempt_id.to_string(),
            materialization_id: request.expected_materialization_id,
            materialization_claim_sha256: request.expected_materialization_claim_sha256,
            materialization_result_id: result.result_id.clone(),
            materialization_result_sha256: request.expected_materialization_result_sha256,
            admission_review_id: review.review_id.clone(),
            admission_review_sha256: request.expected_admission_review_sha256,
            source_validation_id: source_validation.validation_id.clone(),
            source_validation_sha256: request.expected_source_validation_sha256,
            source_output_sha256: request.expected_source_output_sha256,
            dataset_id: source_validation.dataset_id.clone(),
            dataset_content_sha256: source_validation.dataset_content_sha256.clone(),
            dataset_manifest_sha256: source_validation.dataset_manifest_sha256.clone(),
            candidate_set_sha256: source_validation.candidate_set_sha256.clone(),
            transformation_spec_sha256: source_validation.transformation_spec_sha256.clone(),
            split_specification_sha256: source_validation.split_specification_sha256.clone(),
            feature_specification_sha256: source_validation.feature_specification_sha256.clone(),
            split_manifest_sha256: request.expected_split_manifest_sha256,
            feature_bundle_sha256: request.expected_feature_bundle_sha256,
            combined_artifact_sha256: request.expected_combined_artifact_sha256,
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
            validator_implementation_sha256: validator_implementation_sha256(),
            validated_at: Utc::now(),
            validated_by: validator_id.to_string(),
            materialized_by: pair.claim.materialized_by.clone(),
            excluded_prior_actor_ids,
            validator_independent_from_materializer_and_complete_prior_chain: independent,
            exact_current_admission_and_source_candidate_verified: audit.exact_current_binding,
            materialization_claim_fingerprint_verified: audit.claim_verified,
            materialization_result_fingerprint_verified: audit.result_verified,
            split_manifest_fingerprint_verified: audit.manifest_verified,
            feature_bundle_fingerprint_verified: audit.bundle_verified,
            combined_artifact_fingerprint_verified: audit.combined_verified,
            exact_split_candidate_copy_verified: audit.exact_split_copy,
            exact_feature_candidate_copy_verified: audit.exact_feature_copy,
            sealed_holdout_withholding_verified: audit.sealed_holdout_verified,
            explicit_missingness_and_exclusion_verified: audit.missingness_and_exclusion_verified,
            downstream_authority_closed_verified: audit.downstream_closed,
            recomputed_materialization_claim_sha256: audit.recomputed_claim_sha256,
            recomputed_materialization_result_sha256: audit.recomputed_result_sha256,
            recomputed_split_manifest_sha256: audit.recomputed_manifest_sha256,
            recomputed_feature_bundle_sha256: audit.recomputed_bundle_sha256,
            recomputed_combined_artifact_sha256: audit.recomputed_combined_sha256,
            mismatch_reasons: audit.mismatch_reasons,
            verdict: if validated {
                HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationVerdict::ValidatedExactOfficialArtifactPair
            } else {
                HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationVerdict::FailedOfficialArtifactStructureOrBindingMismatch
            },
            official_artifact_pair_independently_validated: validated,
            future_feature_label_join_specification_registration_eligible: validated,
            feature_join_performed: false,
            semantic_target_assigned: false,
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
    exact_current_binding: bool,
    claim_verified: bool,
    result_verified: bool,
    manifest_verified: bool,
    bundle_verified: bool,
    combined_verified: bool,
    exact_split_copy: bool,
    exact_feature_copy: bool,
    sealed_holdout_verified: bool,
    missingness_and_exclusion_verified: bool,
    downstream_closed: bool,
    recomputed_claim_sha256: String,
    recomputed_result_sha256: String,
    recomputed_manifest_sha256: String,
    recomputed_bundle_sha256: String,
    recomputed_combined_sha256: String,
    mismatch_reasons: Vec<String>,
}

fn independently_audit_pair(
    pair: &CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair,
) -> Result<IndependentAudit, String> {
    let candidate = &pair.admitted_candidate;
    let review = &candidate.admission_review;
    let source_validation = &candidate.candidate.validation;
    let envelope = candidate
        .candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "当前准入候选缺少源输出包".to_string())?;
    let claim = &pair.claim;
    let result = &pair.result;
    let manifest = &pair.split_manifest;
    let bundle = &pair.feature_bundle;
    let recomputed_claim_sha256 =
        fingerprint_without(claim, &["materialization_id", "claim_sha256"])?;
    let recomputed_result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    let recomputed_manifest_sha256 = fingerprint_without(manifest, &["manifest_sha256"])?;
    let recomputed_bundle_sha256 = fingerprint_without(bundle, &["feature_bundle_sha256"])?;
    let recomputed_combined_sha256 = hash_serializable(&(
        recomputed_manifest_sha256.as_str(),
        recomputed_bundle_sha256.as_str(),
        recomputed_claim_sha256.as_str(),
    ))?;
    let exact_current_binding = claim.transformation_attempt_id == review.transformation_attempt_id
        && claim.admission_review_id == review.review_id
        && claim.admission_review_sha256 == review.review_sha256
        && claim.validation_id == source_validation.validation_id
        && claim.validation_sha256 == source_validation.validation_sha256
        && claim.source_output_sha256 == source_validation.output_sha256
        && claim.dataset_id == source_validation.dataset_id
        && claim.dataset_content_sha256 == source_validation.dataset_content_sha256
        && claim.dataset_manifest_sha256 == source_validation.dataset_manifest_sha256
        && claim.candidate_set_sha256 == source_validation.candidate_set_sha256
        && claim.transformation_spec_sha256 == source_validation.transformation_spec_sha256
        && claim.split_specification_sha256 == source_validation.split_specification_sha256
        && claim.feature_specification_sha256 == source_validation.feature_specification_sha256
        && claim.expected_split_manifest_candidate_sha256
            == source_validation.recomputed_split_manifest_candidate_sha256
        && claim.expected_feature_bundle_candidate_sha256
            == source_validation.recomputed_feature_bundle_candidate_sha256
        && claim.expected_exclusion_audit_sha256
            == source_validation.recomputed_exclusion_audit_sha256;
    let claim_verified = claim.schema_version == MATERIALIZATION_CLAIM_SCHEMA_VERSION
        && claim.policy_version == MATERIALIZATION_POLICY_VERSION
        && claim.claim_sha256 == recomputed_claim_sha256
        && claim.materialization_id == recomputed_claim_sha256[..32]
        && claim.claim_consumed
        && claim.official_artifact_materialization_started
        && claim.exact_copy_only_confirmed
        && claim.create_once_and_failure_consumes_confirmed
        && claim.no_join_target_training_or_trading_confirmed
        && claim.independent_output_validation_required_confirmed;
    let result_verified = result.schema_version == MATERIALIZATION_RESULT_SCHEMA_VERSION
        && result.policy_version == MATERIALIZATION_POLICY_VERSION
        && result.result_sha256 == recomputed_result_sha256
        && result.result_id == recomputed_result_sha256[..32]
        && result.materialization_id == claim.materialization_id
        && result.claim_sha256 == claim.claim_sha256
        && result.status
            == HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus::CompletedPendingIndependentValidation
        && result.error.is_none()
        && result.official_split_manifest_created
        && result.official_feature_bundle_created
        && result.exact_validated_candidate_copy_completed
        && result.total_artifact_bytes > 0
        && result.total_artifact_bytes <= MAX_TOTAL_ARTIFACT_BYTES;
    let manifest_verified = manifest.schema_version == SPLIT_MANIFEST_SCHEMA_VERSION
        && manifest.manifest_sha256 == recomputed_manifest_sha256
        && result.split_manifest_sha256.as_deref() == Some(manifest.manifest_sha256.as_str())
        && manifest.materialization_id == claim.materialization_id
        && manifest.transformation_attempt_id == claim.transformation_attempt_id
        && manifest.admission_review_id == claim.admission_review_id
        && manifest.admission_review_sha256 == claim.admission_review_sha256
        && manifest.validation_id == claim.validation_id
        && manifest.validation_sha256 == claim.validation_sha256
        && manifest.source_output_sha256 == claim.source_output_sha256
        && manifest.dataset_id == claim.dataset_id
        && manifest.dataset_content_sha256 == claim.dataset_content_sha256
        && manifest.dataset_manifest_sha256 == claim.dataset_manifest_sha256
        && manifest.candidate_set_sha256 == claim.candidate_set_sha256
        && manifest.transformation_spec_sha256 == claim.transformation_spec_sha256
        && manifest.split_specification_sha256 == claim.split_specification_sha256
        && manifest.source_split_manifest_candidate_sha256
            == claim.expected_split_manifest_candidate_sha256;
    let bundle_verified = bundle.schema_version == FEATURE_BUNDLE_SCHEMA_VERSION
        && bundle.feature_bundle_sha256 == recomputed_bundle_sha256
        && result.feature_bundle_sha256.as_deref() == Some(bundle.feature_bundle_sha256.as_str())
        && bundle.materialization_id == claim.materialization_id
        && bundle.transformation_attempt_id == claim.transformation_attempt_id
        && bundle.admission_review_id == claim.admission_review_id
        && bundle.admission_review_sha256 == claim.admission_review_sha256
        && bundle.validation_id == claim.validation_id
        && bundle.validation_sha256 == claim.validation_sha256
        && bundle.source_output_sha256 == claim.source_output_sha256
        && bundle.dataset_id == claim.dataset_id
        && bundle.dataset_content_sha256 == claim.dataset_content_sha256
        && bundle.dataset_manifest_sha256 == claim.dataset_manifest_sha256
        && bundle.candidate_set_sha256 == claim.candidate_set_sha256
        && bundle.transformation_spec_sha256 == claim.transformation_spec_sha256
        && bundle.feature_specification_sha256 == claim.feature_specification_sha256
        && bundle.source_feature_bundle_candidate_sha256
            == claim.expected_feature_bundle_candidate_sha256
        && bundle.source_exclusion_audit_sha256 == claim.expected_exclusion_audit_sha256;
    let combined_verified =
        result.combined_artifact_sha256.as_deref() == Some(recomputed_combined_sha256.as_str());
    let exact_split_copy = manifest.boundary_audit == envelope.boundary_audit
        && manifest.records == envelope.split_manifest_candidate
        && manifest.entry_count == envelope.entry_count
        && manifest.component_count == envelope.component_count
        && hash_serializable(&manifest.records)?
            == source_validation.recomputed_split_manifest_candidate_sha256;
    let exact_feature_copy = bundle.feature_catalog_count == EXPECTED_FEATURE_COUNT
        && bundle.feature_catalog_count == envelope.feature_catalog_count
        && bundle.feature_catalog_sha256 == envelope.feature_catalog_sha256
        && bundle.feature_schema_sha256 == envelope.feature_schema_sha256
        && bundle.records == envelope.feature_bundle_candidate
        && bundle.exclusion_audit == envelope.excluded_feature_audit
        && hash_serializable(&bundle.records)?
            == source_validation.recomputed_feature_bundle_candidate_sha256
        && hash_serializable(&bundle.exclusion_audit)?
            == source_validation.recomputed_exclusion_audit_sha256;
    let sealed_holdout_verified =
        manifest.sealed_holdout_labels_withheld && envelope.sealed_holdout_labels_withheld;
    let missingness_and_exclusion_verified = bundle.explicit_missingness_preserved
        && bundle.no_imputation_or_backfill_performed
        && bundle.outcome_future_and_current_portfolio_sources_excluded
        && bundle.records.iter().all(|record| {
            record.value.is_none()
                && record.is_missing
                && record.value_encoding == "explicit_missingness"
        });
    let canonical_artifact_bytes = serde_json::to_vec(manifest)
        .map_err(|error| error.to_string())?
        .len()
        + serde_json::to_vec(bundle)
            .map_err(|error| error.to_string())?
            .len();
    let downstream_closed = !claim.feature_join_allowed
        && !claim.semantic_target_assignment_allowed
        && !claim.training_allowed
        && !claim.reward_allowed
        && !claim.shadow_portfolio_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed
        && !result.independent_output_validation_completed
        && !result.official_artifacts_eligible_for_feature_join
        && !result.feature_join_performed
        && !result.semantic_target_assigned
        && !result.training_authorized
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized
        && !manifest.independently_validated_after_materialization
        && !manifest.eligible_for_feature_join
        && !manifest.semantic_target_assigned
        && !manifest.training_authorized
        && !bundle.independently_validated_after_materialization
        && !bundle.joined_to_outcome_labels
        && !bundle.semantic_target_assigned
        && !bundle.copied_to_training_store
        && !bundle.training_authorized
        && result.total_artifact_bytes == canonical_artifact_bytes;
    let mut mismatch_reasons = Vec::new();
    push_mismatch(
        &mut mismatch_reasons,
        exact_current_binding,
        "exact_current_admission_or_source_binding_mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        claim_verified,
        "materialization_claim_fingerprint_or_contract_mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        result_verified,
        "materialization_result_fingerprint_or_contract_mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        manifest_verified,
        "official_split_manifest_fingerprint_or_binding_mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        bundle_verified,
        "official_feature_bundle_fingerprint_or_binding_mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        combined_verified,
        "combined_artifact_fingerprint_mismatch",
    );
    push_mismatch(
        &mut mismatch_reasons,
        exact_split_copy,
        "official_split_manifest_not_exact_validated_candidate_copy",
    );
    push_mismatch(
        &mut mismatch_reasons,
        exact_feature_copy,
        "official_feature_bundle_not_exact_validated_candidate_copy",
    );
    push_mismatch(
        &mut mismatch_reasons,
        sealed_holdout_verified,
        "sealed_holdout_withholding_invalid",
    );
    push_mismatch(
        &mut mismatch_reasons,
        missingness_and_exclusion_verified,
        "explicit_missingness_or_exclusion_invalid",
    );
    push_mismatch(
        &mut mismatch_reasons,
        downstream_closed,
        "official_artifact_or_materialization_authority_overreach",
    );
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    mismatch_reasons.truncate(MAX_MISMATCH_REASONS);
    Ok(IndependentAudit {
        exact_current_binding,
        claim_verified,
        result_verified,
        manifest_verified,
        bundle_verified,
        combined_verified,
        exact_split_copy,
        exact_feature_copy,
        sealed_holdout_verified,
        missingness_and_exclusion_verified,
        downstream_closed,
        recomputed_claim_sha256,
        recomputed_result_sha256,
        recomputed_manifest_sha256,
        recomputed_bundle_sha256,
        recomputed_combined_sha256,
        mismatch_reasons,
    })
}

fn push_mismatch(reasons: &mut Vec<String>, matched: bool, reason: &str) {
    if !matched {
        reasons.push(reason.to_string());
    }
}

fn validate_request(
    attempt_id: &str,
    request: &ValidateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
) -> Result<(), String> {
    if !valid_id(attempt_id)
        || !valid_id(&request.expected_materialization_id)
        || !valid_sha256(&request.expected_materialization_claim_sha256)
        || !valid_sha256(&request.expected_materialization_result_sha256)
        || !valid_sha256(&request.expected_admission_review_sha256)
        || !valid_sha256(&request.expected_source_validation_sha256)
        || !valid_sha256(&request.expected_source_output_sha256)
        || !valid_sha256(&request.expected_split_manifest_sha256)
        || !valid_sha256(&request.expected_feature_bundle_sha256)
        || !valid_sha256(&request.expected_combined_artifact_sha256)
    {
        return Err("正式工件校验请求中的 ID 或 SHA-256 无效".to_string());
    }
    if !request.exact_artifact_pair_binding_confirmed
        || !request.independent_validator_confirmed
        || !request.no_join_target_training_or_trading_confirmed
    {
        return Err("必须确认精确工件绑定、角色独立和零下游权限三项边界".to_string());
    }
    Ok(())
}

fn verify_expected_bindings(
    pair: &CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair,
    request: &ValidateHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactsRequest,
) -> Result<(), String> {
    let review = &pair.admitted_candidate.admission_review;
    let source_validation = &pair.admitted_candidate.candidate.validation;
    if pair.claim.materialization_id != request.expected_materialization_id
        || pair.claim.claim_sha256 != request.expected_materialization_claim_sha256
        || pair.result.result_sha256 != request.expected_materialization_result_sha256
        || review.review_sha256 != request.expected_admission_review_sha256
        || source_validation.validation_sha256 != request.expected_source_validation_sha256
        || source_validation.output_sha256 != request.expected_source_output_sha256
        || pair.split_manifest.manifest_sha256 != request.expected_split_manifest_sha256
        || pair.feature_bundle.feature_bundle_sha256 != request.expected_feature_bundle_sha256
        || pair.result.combined_artifact_sha256.as_deref()
            != Some(request.expected_combined_artifact_sha256.as_str())
    {
        return Err("正式工件、物化结果或当前源候选绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

async fn completed_official_artifact_pairs(
    state: &AppState,
) -> Result<Vec<CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair>, String>
{
    let candidates =
        admitted_transformation_candidates_for_official_artifact_materialization(state).await?;
    let mut pairs = Vec::new();
    for candidate in candidates {
        if let Some(pair) = read_completed_pair(state, candidate).await? {
            pairs.push(pair);
        }
    }
    pairs.sort_by(|left, right| right.result.completed_at.cmp(&left.result.completed_at));
    Ok(pairs)
}

async fn read_completed_pair(
    state: &AppState,
    admitted_candidate: AdmittedHistoricalOutcomeOfflineTransformationCandidate,
) -> Result<
    Option<CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair>,
    String,
> {
    let attempt_id = &admitted_candidate
        .admission_review
        .transformation_attempt_id;
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
        HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationClaim,
    >(&claim_bytes)
    .map_err(|error| format!("正式物化 claim 无法解析：{error}"))?;
    let result = serde_json::from_slice::<
        HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationResult,
    >(&result_bytes)
    .map_err(|error| format!("正式物化 result 无法解析：{error}"))?;
    if result.status
        != HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactMaterializationStatus::CompletedPendingIndependentValidation
    {
        return Ok(None);
    }
    let manifest_sha256 = result
        .split_manifest_sha256
        .as_deref()
        .ok_or_else(|| "正式物化结果缺少 split manifest SHA-256".to_string())?;
    let feature_bundle_sha256 = result
        .feature_bundle_sha256
        .as_deref()
        .ok_or_else(|| "正式物化结果缺少 feature bundle SHA-256".to_string())?;
    let split_manifest =
        serde_json::from_slice::<HistoricalOutcomeOfflineDatasetOfficialSplitManifest>(
            &tokio::fs::read(directory.join(format!("split-manifest-{manifest_sha256}.json")))
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| format!("正式 split manifest 无法解析：{error}"))?;
    let feature_bundle = serde_json::from_slice::<
        HistoricalOutcomeOfflineDatasetOfficialFeatureBundle,
    >(
        &tokio::fs::read(directory.join(format!("feature-bundle-{feature_bundle_sha256}.json")))
            .await
            .map_err(|error| error.to_string())?,
    )
    .map_err(|error| format!("正式 feature bundle 无法解析：{error}"))?;
    Ok(Some(
        CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair {
            admitted_candidate,
            claim,
            result,
            split_manifest,
            feature_bundle,
        },
    ))
}

fn prior_actor_ids(
    pair: &CompletedHistoricalOutcomeOfflineDatasetTransformationOfficialArtifactPair,
) -> Vec<String> {
    let mut actors = pair.claim.excluded_prior_actor_ids.clone();
    actors.push(pair.claim.materialized_by.clone());
    actors.push(pair.admitted_candidate.admission_review.reviewer_id.clone());
    actors.push(
        pair.admitted_candidate
            .candidate
            .validation
            .validated_by
            .clone(),
    );
    actors.extend(
        pair.admitted_candidate
            .admission_review
            .excluded_prior_actor_ids
            .clone(),
    );
    actors.retain(|actor| !actor.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn actor_is_independent(actor_id: &str, excluded_actor_ids: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded_actor_ids.iter().all(|actor| actor != actor_id)
}

fn validate_record(
    record: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord,
) -> Result<(), String> {
    let expected_valid = record.validator_independent_from_materializer_and_complete_prior_chain
        && record.exact_current_admission_and_source_candidate_verified
        && record.materialization_claim_fingerprint_verified
        && record.materialization_result_fingerprint_verified
        && record.split_manifest_fingerprint_verified
        && record.feature_bundle_fingerprint_verified
        && record.combined_artifact_fingerprint_verified
        && record.exact_split_candidate_copy_verified
        && record.exact_feature_candidate_copy_verified
        && record.sealed_holdout_withholding_verified
        && record.explicit_missingness_and_exclusion_verified
        && record.downstream_authority_closed_verified
        && record.mismatch_reasons.is_empty();
    let downstream_closed = !record.feature_join_performed
        && !record.semantic_target_assigned
        && !record.copied_to_training_store
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
        || !valid_id(&record.transformation_attempt_id)
        || !valid_id(&record.materialization_id)
        || !valid_sha256(&record.materialization_claim_sha256)
        || !valid_id(&record.materialization_result_id)
        || !valid_sha256(&record.materialization_result_sha256)
        || !valid_id(&record.admission_review_id)
        || !valid_sha256(&record.admission_review_sha256)
        || !valid_id(&record.source_validation_id)
        || !valid_sha256(&record.source_validation_sha256)
        || !valid_sha256(&record.source_output_sha256)
        || !valid_id(&record.dataset_id)
        || !valid_sha256(&record.dataset_content_sha256)
        || !valid_sha256(&record.dataset_manifest_sha256)
        || !valid_sha256(&record.candidate_set_sha256)
        || !valid_sha256(&record.transformation_spec_sha256)
        || !valid_sha256(&record.split_specification_sha256)
        || !valid_sha256(&record.feature_specification_sha256)
        || !valid_sha256(&record.split_manifest_sha256)
        || !valid_sha256(&record.feature_bundle_sha256)
        || !valid_sha256(&record.combined_artifact_sha256)
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || record.validated_by.trim().is_empty()
        || record.materialized_by.trim().is_empty()
        || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || !actor_is_independent(&record.validated_by, &record.excluded_prior_actor_ids)
        || !valid_sha256(&record.recomputed_materialization_claim_sha256)
        || !valid_sha256(&record.recomputed_materialization_result_sha256)
        || !valid_sha256(&record.recomputed_split_manifest_sha256)
        || !valid_sha256(&record.recomputed_feature_bundle_sha256)
        || !valid_sha256(&record.recomputed_combined_artifact_sha256)
        || record.mismatch_reasons.len() > MAX_MISMATCH_REASONS
        || record.official_artifact_pair_independently_validated != expected_valid
        || record.future_feature_label_join_specification_registration_eligible != expected_valid
        || record.official_artifact_pair_independently_validated
            != (record.verdict
                == HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationVerdict::ValidatedExactOfficialArtifactPair)
        || !downstream_closed
    {
        return Err("official artifact output validation record is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_validations(
    state: &AppState,
) -> Result<
    Vec<HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord>,
    String,
> {
    let root = validation_root(state);
    let mut records = Vec::new();
    let mut attempts = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(records),
        Err(error) => return Err(error.to_string()),
    };
    while let Some(attempt_entry) = attempts
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if !attempt_entry
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let attempt_id = attempt_entry.file_name().to_string_lossy().to_string();
        if !valid_id(&attempt_id) {
            return Err("正式工件独立校验目录包含无效 attempt ID".to_string());
        }
        let mut files = tokio::fs::read_dir(attempt_entry.path())
            .await
            .map_err(|error| error.to_string())?;
        let mut attempt_record_count = 0usize;
        while let Some(entry) = files
            .next_entry()
            .await
            .map_err(|error| error.to_string())?
        {
            if !entry
                .file_type()
                .await
                .map_err(|error| error.to_string())?
                .is_file()
                || entry.path().extension().and_then(|value| value.to_str()) != Some("json")
            {
                continue;
            }
            attempt_record_count += 1;
            let record = serde_json::from_slice::<
                HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord,
            >(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            validate_record(&record)?;
            if record.transformation_attempt_id != attempt_id
                || entry.file_name().to_string_lossy() != format!("{}.json", record.validation_id)
            {
                return Err("正式工件独立校验记录路径或文件名与内容不一致".to_string());
            }
            records.push(record);
        }
        if attempt_record_count > 1 {
            return Err("同一正式工件 attempt 存在多条校验记录，失败关闭".to_string());
        }
    }
    records.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn validator_implementation_sha256() -> String {
    format!(
        "{:x}",
        Sha256::digest(VALIDATOR_IMPLEMENTATION_VERSION.as_bytes())
    )
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "official artifact validation fingerprint is not an object".to_string())?;
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

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join(
        "historical-outcome-offline-dataset-transformation-official-artifact-output-validations",
    )
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
        "historical-offline-transformation-official-artifact-output-validation-{attempt_id}.lock"
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
        .map_err(|_| "已有同一正式工件独立校验正在进行".to_string())?;
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
    -> HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord {
        let mut record = HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationRecord {
            schema_version: RECORD_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            validation_id: String::new(),
            validation_sha256: String::new(),
            transformation_attempt_id: "1".repeat(32),
            materialization_id: "2".repeat(32),
            materialization_claim_sha256: "3".repeat(64),
            materialization_result_id: "4".repeat(32),
            materialization_result_sha256: "5".repeat(64),
            admission_review_id: "6".repeat(32),
            admission_review_sha256: "7".repeat(64),
            source_validation_id: "8".repeat(32),
            source_validation_sha256: "9".repeat(64),
            source_output_sha256: "a".repeat(64),
            dataset_id: "b".repeat(32),
            dataset_content_sha256: "c".repeat(64),
            dataset_manifest_sha256: "d".repeat(64),
            candidate_set_sha256: "e".repeat(64),
            transformation_spec_sha256: "f".repeat(64),
            split_specification_sha256: "1".repeat(64),
            feature_specification_sha256: "2".repeat(64),
            split_manifest_sha256: "3".repeat(64),
            feature_bundle_sha256: "4".repeat(64),
            combined_artifact_sha256: "5".repeat(64),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.into(),
            validator_implementation_sha256: validator_implementation_sha256(),
            validated_at: Utc::now(),
            validated_by: "independent-validator".into(),
            materialized_by: "materializer".into(),
            excluded_prior_actor_ids: vec!["materializer".into(), "reviewer".into()],
            validator_independent_from_materializer_and_complete_prior_chain: true,
            exact_current_admission_and_source_candidate_verified: true,
            materialization_claim_fingerprint_verified: true,
            materialization_result_fingerprint_verified: true,
            split_manifest_fingerprint_verified: true,
            feature_bundle_fingerprint_verified: true,
            combined_artifact_fingerprint_verified: true,
            exact_split_candidate_copy_verified: true,
            exact_feature_candidate_copy_verified: true,
            sealed_holdout_withholding_verified: true,
            explicit_missingness_and_exclusion_verified: true,
            downstream_authority_closed_verified: true,
            recomputed_materialization_claim_sha256: "6".repeat(64),
            recomputed_materialization_result_sha256: "7".repeat(64),
            recomputed_split_manifest_sha256: "8".repeat(64),
            recomputed_feature_bundle_sha256: "9".repeat(64),
            recomputed_combined_artifact_sha256: "a".repeat(64),
            mismatch_reasons: Vec::new(),
            verdict: HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationVerdict::ValidatedExactOfficialArtifactPair,
            official_artifact_pair_independently_validated: true,
            future_feature_label_join_specification_registration_eligible: true,
            feature_join_performed: false,
            semantic_target_assigned: false,
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
        let actors = vec!["materializer".to_string(), "reviewer".to_string()];
        assert!(actor_is_independent("independent-validator", &actors));
        assert!(!actor_is_independent("materializer", &actors));
    }

    #[test]
    fn validated_pair_opens_only_future_join_specification_registration() {
        let record = minimal_record();
        validate_record(&record).unwrap();
        assert!(record.future_feature_label_join_specification_registration_eligible);
        assert!(!record.feature_join_performed);
        assert!(!record.semantic_target_assigned);
        assert!(!record.training_authorized);
        assert!(!record.trading_authorized);
    }

    #[test]
    fn failed_check_cannot_remain_validated() {
        let mut record = minimal_record();
        record.exact_feature_candidate_copy_verified = false;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn validation_hash_binds_downstream_authority() {
        let record = minimal_record();
        let first = record_fingerprint(&record).unwrap();
        let mut changed = record.clone();
        changed.training_authorized = true;
        assert_ne!(first, record_fingerprint(&changed).unwrap());
    }

    #[test]
    fn independent_artifact_fingerprint_detects_field_tampering() {
        let original = serde_json::json!({
            "schema_version": SPLIT_MANIFEST_SCHEMA_VERSION,
            "manifest_sha256": "",
            "records": ["train", "validation", "sealed_holdout"],
            "training_authorized": false,
        });
        let first = fingerprint_without(&original, &["manifest_sha256"]).unwrap();
        let mut changed = original;
        changed["training_authorized"] = serde_json::Value::Bool(true);
        assert_ne!(
            first,
            fingerprint_without(&changed, &["manifest_sha256"]).unwrap()
        );
    }

    #[test]
    fn mismatch_reasons_are_fail_closed() {
        let mut record = minimal_record();
        record.mismatch_reasons = vec!["combined_artifact_fingerprint_mismatch".into()];
        record.verdict = HistoricalOutcomeOfflineDatasetTransformationOfficialArtifactOutputValidationVerdict::FailedOfficialArtifactStructureOrBindingMismatch;
        record.official_artifact_pair_independently_validated = false;
        record.future_feature_label_join_specification_registration_eligible = false;
        record.validation_sha256 = record_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        validate_record(&record).unwrap();
    }
}
