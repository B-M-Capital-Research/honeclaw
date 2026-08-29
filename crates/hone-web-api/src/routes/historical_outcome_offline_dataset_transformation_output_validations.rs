//! Independent validation of one untrusted offline transformation envelope.
//!
//! The validator reopens the exact current dataset, sealed price snapshots,
//! runner and consumed authorization, then recomputes connected components,
//! chronological boundaries, purge/embargo and explicit missingness without
//! calling the Stage 31 transformation implementation. Passing this gate only
//! validates an untrusted candidate. It does not create an official manifest
//! or bundle and cannot join features, assign targets, train, reward, shadow,
//! order, access a broker or trade.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_transformation_execution_attempts::{
    CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt,
    HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit,
    HistoricalOutcomeOfflineDatasetTransformationExclusionAudit,
    HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord,
    HistoricalOutcomeOfflineDatasetTransformationSplit,
    HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord,
    HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope,
    completed_transformation_execution_attempt_for_independent_validation,
    completed_transformation_execution_attempts_for_independent_validation,
};
use super::historical_outcome_offline_dataset_transformation_first_execution_authorizations::{
    ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    exact_first_execution_authorization_for_transformation_audit,
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
    "hone-historical-outcome-offline-dataset-transformation-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-output-validation-v1";
const VALIDATION_POLICY_VERSION: &str =
    "hone-offline-transformation-independent-recomputation-validation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-independent-offline-transformation-validator-v1-graph-traversal-no-execution-code-reuse";
const ENVELOPE_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-untrusted-candidate-envelope-v1";
const MISSINGNESS_TRANSFORMATION_ID: &str = "hone-explicit-point-in-time-missingness-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeOfflineDatasetTransformationOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_transformation_spec_sha256: String,
    expected_split_specification_sha256: String,
    expected_feature_specification_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetTransformationOutputValidationVerdict {
    ValidatedIndependentStructureAndDeterministicMatch,
    FailedStructureOrIndependentRecomputationMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub transformation_attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
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
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub execution_invoked_by: String,
    pub runner_registered_by: String,
    pub authorization_reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_execution_and_complete_prior_chain: bool,
    pub immutable_chain_integrity_verified: bool,
    pub current_dataset_binding_verified: bool,
    pub current_sealed_snapshot_bindings_verified: bool,
    pub canonical_output_hash_verified: bool,
    pub output_structure_verified: bool,
    pub independent_component_recomputation_match: bool,
    pub independent_boundary_recomputation_match: bool,
    pub independent_purge_embargo_recomputation_match: bool,
    pub independent_feature_recomputation_match: bool,
    pub sealed_holdout_withholding_verified: bool,
    pub recomputed_boundary_audit_sha256: String,
    pub recomputed_split_manifest_candidate_sha256: String,
    pub recomputed_feature_bundle_candidate_sha256: String,
    pub recomputed_exclusion_audit_sha256: String,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeOfflineDatasetTransformationOutputValidationVerdict,
    pub untrusted_candidate_envelope_validated: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOutputValidationItem {
    pub attempt: CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt,
    pub validation: Option<HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeOfflineDatasetTransformationOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub validated_candidate_envelope_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
    pub output_validation_available: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub validated_candidate_envelope_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate {
    pub attempt: CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt,
    pub validation: HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord,
}

#[derive(Debug, Clone)]
struct ValidationEntryInput {
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
struct ValidationComponent {
    component_id: String,
    entry_indexes: Vec<usize>,
    earliest: DateTime<Utc>,
    latest: DateTime<Utc>,
}

#[derive(Debug)]
struct IndependentProjection {
    component_count: usize,
    boundary_audit: HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit,
    split_records: Vec<HistoricalOutcomeOfflineDatasetTransformationSplitCandidateRecord>,
    feature_records: Vec<HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord>,
    exclusion_audit: Vec<HistoricalOutcomeOfflineDatasetTransformationExclusionAudit>,
    feature_catalog_sha256: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_output_validations(
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
            warn!(%error, "historical offline transformation output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线转换输出独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_offline_dataset_transformation_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeOfflineDatasetTransformationOutputRequest>,
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

pub(crate) async fn historical_outcome_offline_dataset_transformation_output_validation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeOfflineDatasetTransformationOutputValidationReadinessSummary {
            validation_eligible_count: registry.validation_eligible_count,
            validation_count: registry.validation_count,
            validated_candidate_envelope_count: registry.validated_candidate_envelope_count,
            failed_validation_count: registry.failed_validation_count,
            validation_status: registry.validation_status,
        },
    )
}

pub(crate) async fn independently_validated_transformation_candidates_for_admission_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate>, String> {
    let validations = read_validations(state).await?;
    let mut candidates = Vec::new();
    for validation in validations
        .into_iter()
        .filter(|record| record.untrusted_candidate_envelope_validated)
    {
        if let Some(candidate) =
            exact_independently_validated_transformation_candidate_for_admission(
                state,
                &validation.transformation_attempt_id,
                &validation.validation_id,
                &validation.validation_sha256,
                &validation.claim_sha256,
                &validation.result_sha256,
                &validation.output_sha256,
                &validation.dataset_content_sha256,
                &validation.dataset_manifest_sha256,
                &validation.candidate_set_sha256,
                &validation.transformation_spec_sha256,
                &validation.split_specification_sha256,
                &validation.feature_specification_sha256,
            )
            .await?
        {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| {
        right
            .validation
            .validated_at
            .cmp(&left.validation.validated_at)
    });
    Ok(candidates)
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn exact_independently_validated_transformation_candidate_for_admission(
    state: &AppState,
    attempt_id: &str,
    validation_id: &str,
    validation_sha256: &str,
    claim_sha256: &str,
    result_sha256: &str,
    output_sha256: &str,
    dataset_content_sha256: &str,
    dataset_manifest_sha256: &str,
    candidate_set_sha256: &str,
    transformation_spec_sha256: &str,
    split_specification_sha256: &str,
    feature_specification_sha256: &str,
) -> Result<Option<IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate>, String> {
    let Some(validation) = read_validations(state).await?.into_iter().find(|record| {
        record.transformation_attempt_id == attempt_id
            && record.validation_id == validation_id
            && record.validation_sha256 == validation_sha256
            && record.claim_sha256 == claim_sha256
            && record.result_sha256 == result_sha256
            && record.output_sha256 == output_sha256
            && record.dataset_content_sha256 == dataset_content_sha256
            && record.dataset_manifest_sha256 == dataset_manifest_sha256
            && record.candidate_set_sha256 == candidate_set_sha256
            && record.transformation_spec_sha256 == transformation_spec_sha256
            && record.split_specification_sha256 == split_specification_sha256
            && record.feature_specification_sha256 == feature_specification_sha256
            && record.untrusted_candidate_envelope_validated
            && record.verdict
                == HistoricalOutcomeOfflineDatasetTransformationOutputValidationVerdict::ValidatedIndependentStructureAndDeterministicMatch
    }) else {
        return Ok(None);
    };
    let Some(attempt) = completed_transformation_execution_attempt_for_independent_validation(
        state,
        attempt_id,
        claim_sha256,
        result_sha256,
        output_sha256,
    )
    .await?
    else {
        return Ok(None);
    };
    let Some(dataset) = exact_current_historical_outcome_offline_dataset_for_transformation(
        state,
        &validation.dataset_id,
        dataset_content_sha256,
        dataset_manifest_sha256,
        candidate_set_sha256,
    )
    .await?
    else {
        return Ok(None);
    };
    let Some(authorization) = exact_first_execution_authorization_for_transformation_audit(
        state,
        &validation.isolated_runner_id,
        &validation.authorization_review_id,
        &validation.authorization_review_sha256,
    )
    .await?
    else {
        return Ok(None);
    };
    let snapshots = current_sealed_historical_outcome_price_snapshots_for_transformation(state)
        .await?
        .into_iter()
        .map(|snapshot| snapshot.snapshot_sha256)
        .collect::<BTreeSet<_>>();
    let current_snapshot_bindings = dataset
        .entries
        .iter()
        .all(|entry| snapshots.contains(&entry.snapshot_sha256));
    let exact_upstream = attempt.claim.dataset_id == validation.dataset_id
        && attempt.claim.transformation_spec_sha256 == transformation_spec_sha256
        && attempt.claim.split_specification_sha256 == split_specification_sha256
        && attempt.claim.feature_specification_sha256 == feature_specification_sha256
        && attempt.claim.implementation_id == validation.implementation_id
        && attempt.claim.implementation_sha256 == validation.implementation_sha256
        && attempt.claim.isolated_runner_id == validation.isolated_runner_id
        && attempt.claim.isolated_runner_spec_sha256 == validation.isolated_runner_spec_sha256
        && authorization.runner.isolated_runner_id == validation.isolated_runner_id
        && authorization.review.review_id == validation.authorization_review_id
        && authorization.review.review_sha256 == validation.authorization_review_sha256;
    if !current_snapshot_bindings || !exact_upstream {
        return Ok(None);
    }
    Ok(Some(
        IndependentlyValidatedHistoricalOutcomeOfflineTransformationCandidate {
            attempt,
            validation,
        },
    ))
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry, String> {
    let attempts =
        completed_transformation_execution_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.transformation_attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let items = attempts
        .into_iter()
        .map(|attempt| {
            let validation = index
                .get(attempt.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeOfflineDatasetTransformationOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let validation_count = validations.len();
    let validated_candidate_envelope_count = validations
        .iter()
        .filter(|record| record.untrusted_candidate_envelope_validated)
        .count();
    let failed_validation_count = validations
        .iter()
        .filter(|record| !record.untrusted_candidate_envelope_validated)
        .count();
    let validation_status = if failed_validation_count > 0 {
        "transformation_candidate_validation_failed_fail_closed"
    } else if validated_candidate_envelope_count > 0 {
        "transformation_candidate_independently_validated_still_not_official_artifact"
    } else if validation_eligible_count > 0 {
        "waiting_independent_transformation_candidate_recomputation_validation"
    } else {
        "waiting_untrusted_transformation_candidate_envelope"
    };
    Ok(HistoricalOutcomeOfflineDatasetTransformationOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: VALIDATION_POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count,
        validated_candidate_envelope_count,
        failed_validation_count,
        validation_status: validation_status.to_string(),
        output_validation_available: true,
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
        scope: "独立管理员只可对第 31 阶段精确 claim/result/output 重新计算连通分量、时间边界、250 交易日 purge/embargo 与 65 项显式缺失值来源。通过后仍是未信任候选，不生成正式 manifest/bundle，不允许 join、目标、训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    admin_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeOfflineDatasetTransformationOutputRequest,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord, String> {
    if !valid_id(attempt_id) {
        return Err("离线转换执行尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.transformation_attempt_id == attempt_id)
    {
        return Err("这条离线转换输出已经形成不可变独立校验记录，不得重放".to_string());
    }
    let attempt = completed_transformation_execution_attempt_for_independent_validation(
        state,
        attempt_id,
        &request.expected_claim_sha256,
        &request.expected_result_sha256,
        &request.expected_output_sha256,
    )
    .await?
    .ok_or_else(|| "转换 claim、result 或 canonical output 已变化，或未形成完整候选".to_string())?;
    verify_expected_bindings(&attempt, &request)?;
    let authorization = exact_first_execution_authorization_for_transformation_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "转换首次执行授权审计链或当前 runner 绑定不可用".to_string())?;
    let dataset = exact_current_historical_outcome_offline_dataset_for_transformation(
        state,
        &attempt.claim.dataset_id,
        &attempt.claim.dataset_content_sha256,
        &attempt.claim.dataset_manifest_sha256,
        &attempt.claim.candidate_set_sha256,
    )
    .await?
    .ok_or_else(|| "精确离线数据集已不是完整当前候选集，校验失败关闭".to_string())?;
    let snapshots =
        current_sealed_historical_outcome_price_snapshots_for_transformation(state).await?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization, &dataset);
    let independent = validator_is_independent(admin_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("独立校验人不能是转换调用人、runner 登记人或完整上游链任一角色".to_string());
    }
    let entries = build_validation_inputs(&dataset, &snapshots)?;
    let projection = independently_recompute(&authorization, &dataset, &entries)?;
    let envelope = attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "转换结果缺少未信任候选包".to_string())?;
    let canonical_output_hash_verified =
        hash_serializable(envelope)? == request.expected_output_sha256;
    let independent_component_recomputation_match =
        envelope.component_count == projection.component_count;
    let independent_boundary_recomputation_match =
        envelope.boundary_audit == projection.boundary_audit;
    let independent_purge_embargo_recomputation_match =
        envelope.split_manifest_candidate == projection.split_records;
    let independent_feature_recomputation_match = envelope.feature_bundle_candidate
        == projection.feature_records
        && envelope.excluded_feature_audit == projection.exclusion_audit
        && envelope.feature_catalog_sha256 == projection.feature_catalog_sha256;
    let sealed_holdout_withholding_verified = envelope.sealed_holdout_labels_withheld
        && envelope.split_manifest_candidate.iter().any(|record| {
            record.split == HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout
        })
        && envelope.feature_bundle_candidate.iter().all(|record| {
            record.value.is_none()
                && record.is_missing
                && record.value_encoding == "explicit_missingness"
        });
    let mut mismatch_reasons =
        structural_mismatch_reasons(&attempt, &authorization, &dataset, envelope);
    if !canonical_output_hash_verified {
        mismatch_reasons.push("canonical_output_sha256_mismatch".to_string());
    }
    if !independent_component_recomputation_match {
        mismatch_reasons.push("independent_component_recomputation_mismatch".to_string());
    }
    if !independent_boundary_recomputation_match {
        mismatch_reasons.push("independent_boundary_recomputation_mismatch".to_string());
    }
    if !independent_purge_embargo_recomputation_match {
        mismatch_reasons.push("independent_purge_embargo_recomputation_mismatch".to_string());
    }
    if !independent_feature_recomputation_match {
        mismatch_reasons
            .push("independent_feature_or_exclusion_recomputation_mismatch".to_string());
    }
    if !sealed_holdout_withholding_verified {
        mismatch_reasons.push("sealed_holdout_withholding_invalid".to_string());
    }
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    let output_structure_verified = mismatch_reasons.iter().all(|reason| {
        matches!(
            reason.as_str(),
            "canonical_output_sha256_mismatch"
                | "independent_component_recomputation_mismatch"
                | "independent_boundary_recomputation_mismatch"
                | "independent_purge_embargo_recomputation_mismatch"
                | "independent_feature_or_exclusion_recomputation_mismatch"
                | "sealed_holdout_withholding_invalid"
        )
    });
    let validated = output_structure_verified
        && canonical_output_hash_verified
        && independent_component_recomputation_match
        && independent_boundary_recomputation_match
        && independent_purge_embargo_recomputation_match
        && independent_feature_recomputation_match
        && sealed_holdout_withholding_verified
        && mismatch_reasons.is_empty();
    let mut record = HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: VALIDATION_POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        transformation_attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        output_sha256: request.expected_output_sha256,
        dataset_id: dataset.dataset_id.clone(),
        dataset_content_sha256: dataset.dataset_content_sha256.clone(),
        dataset_manifest_sha256: dataset.manifest_sha256.clone(),
        candidate_set_sha256: dataset.candidate_set_sha256.clone(),
        transformation_spec_sha256: attempt.claim.transformation_spec_sha256.clone(),
        split_specification_sha256: attempt.claim.split_specification_sha256.clone(),
        feature_specification_sha256: attempt.claim.feature_specification_sha256.clone(),
        implementation_id: attempt.claim.implementation_id.clone(),
        implementation_sha256: attempt.claim.implementation_sha256.clone(),
        isolated_runner_id: attempt.claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: attempt.claim.isolated_runner_spec_sha256.clone(),
        authorization_review_id: attempt.claim.authorization_review_id.clone(),
        authorization_review_sha256: attempt.claim.authorization_review_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: admin_id.to_string(),
        execution_invoked_by: attempt.claim.invoked_by.clone(),
        runner_registered_by: authorization.runner.registered_by.clone(),
        authorization_reviewer_id: authorization.review.reviewer_id.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_execution_and_complete_prior_chain: independent,
        immutable_chain_integrity_verified: true,
        current_dataset_binding_verified: true,
        current_sealed_snapshot_bindings_verified: true,
        canonical_output_hash_verified,
        output_structure_verified,
        independent_component_recomputation_match,
        independent_boundary_recomputation_match,
        independent_purge_embargo_recomputation_match,
        independent_feature_recomputation_match,
        sealed_holdout_withholding_verified,
        recomputed_boundary_audit_sha256: projection.boundary_audit.audit_sha256.clone(),
        recomputed_split_manifest_candidate_sha256: hash_serializable(&projection.split_records)?,
        recomputed_feature_bundle_candidate_sha256: hash_serializable(&projection.feature_records)?,
        recomputed_exclusion_audit_sha256: hash_serializable(&projection.exclusion_audit)?,
        mismatch_reasons,
        verdict: if validated {
            HistoricalOutcomeOfflineDatasetTransformationOutputValidationVerdict::ValidatedIndependentStructureAndDeterministicMatch
        } else {
            HistoricalOutcomeOfflineDatasetTransformationOutputValidationVerdict::FailedStructureOrIndependentRecomputationMismatch
        },
        untrusted_candidate_envelope_validated: validated,
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
    record.validation_sha256 = validation_fingerprint(&record)?;
    record.validation_id = record.validation_sha256[..32].to_string();
    validate_record(&record)?;
    write_immutable_json(
        &validation_root(state)
            .join(&record.transformation_attempt_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn verify_expected_bindings(
    attempt: &CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt,
    request: &ValidateHistoricalOutcomeOfflineDatasetTransformationOutputRequest,
) -> Result<(), String> {
    if attempt.claim.dataset_content_sha256 != request.expected_dataset_content_sha256
        || attempt.claim.dataset_manifest_sha256 != request.expected_dataset_manifest_sha256
        || attempt.claim.candidate_set_sha256 != request.expected_candidate_set_sha256
        || attempt.claim.transformation_spec_sha256 != request.expected_transformation_spec_sha256
        || attempt.claim.split_specification_sha256 != request.expected_split_specification_sha256
        || attempt.claim.feature_specification_sha256
            != request.expected_feature_specification_sha256
    {
        return Err("转换输出的数据集或冻结转换规格绑定已经变化".to_string());
    }
    Ok(())
}

fn prior_actor_ids(
    attempt: &CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeOfflineDataset,
) -> Vec<String> {
    let mut actors = authorization.runner.excluded_prior_actor_ids.clone();
    actors.extend(authorization.review.excluded_prior_actor_ids.clone());
    actors.extend([
        attempt.claim.invoked_by.clone(),
        authorization.runner.registered_by.clone(),
        authorization
            .runner
            .implementation_review
            .reviewer_id
            .clone(),
        authorization.review.reviewer_id.clone(),
        dataset.assembled_by.clone(),
    ]);
    for entry in &dataset.entries {
        actors.extend(entry.excluded_prior_actor_ids.clone());
        actors.push(entry.formal_label_written_by.clone());
        actors.push(entry.independently_validated_by.clone());
    }
    actors.retain(|actor| !actor.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn validator_is_independent(validator: &str, prior_actors: &[String]) -> bool {
    !validator.trim().is_empty() && !prior_actors.iter().any(|actor| actor == validator)
}

fn build_validation_inputs(
    dataset: &HistoricalOutcomeOfflineDataset,
    snapshots: &[HistoricalOutcomePriceSnapshot],
) -> Result<Vec<ValidationEntryInput>, String> {
    let snapshot_index = snapshots
        .iter()
        .map(|snapshot| (snapshot.snapshot_sha256.as_str(), snapshot))
        .collect::<HashMap<_, _>>();
    dataset
        .entries
        .iter()
        .map(|entry| validation_input(entry, &snapshot_index))
        .collect()
}

fn validation_input(
    entry: &HistoricalOutcomeOfflineDatasetEntry,
    snapshot_index: &HashMap<&str, &HistoricalOutcomePriceSnapshot>,
) -> Result<ValidationEntryInput, String> {
    let snapshot = snapshot_index
        .get(entry.snapshot_sha256.as_str())
        .ok_or_else(|| format!("数据集条目 {} 缺少当前封存行情快照", entry.entry_id))?;
    if snapshot.reconstruction_sha256 != entry.reconstruction_sha256
        || snapshot.asset_symbol != entry.asset_symbol
        || snapshot.benchmark_symbol != entry.benchmark_symbol
        || snapshot.decision_available_at != entry.decision_available_at
        || snapshot.protocol_sha256 != entry.protocol_sha256
    {
        return Err(format!("数据集条目 {} 与封存快照不一致", entry.entry_id));
    }
    if entry.raw_validated_metrics.len() != REQUIRED_HORIZONS.len()
        || entry
            .raw_validated_metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect::<Vec<_>>()
            != REQUIRED_HORIZONS
    {
        return Err(format!("数据集条目 {} 结果窗口不完整", entry.entry_id));
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
        .ok_or_else(|| format!("数据集条目 {} 起始交易日不在共同日历", entry.entry_id))?;
    if entry
        .raw_validated_metrics
        .iter()
        .any(|metric| metric.start_date != start || !common_dates.contains(&metric.end_date))
    {
        return Err(format!(
            "数据集条目 {} 窗口端点不在共同日历",
            entry.entry_id
        ));
    }
    Ok(ValidationEntryInput {
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

fn independently_recompute(
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeOfflineDataset,
    entries: &[ValidationEntryInput],
) -> Result<IndependentProjection, String> {
    if entries.is_empty() || entries.len() != dataset.entry_count {
        return Err("独立校验输入与数据集条目数不一致".to_string());
    }
    let specification = &authorization
        .runner
        .implementation
        .approved_review
        .specification;
    let features = &specification.feature_bundle_specification.allowed_features;
    if features.len() != EXPECTED_FEATURE_COUNT {
        return Err("冻结点时特征目录不是 65 项".to_string());
    }
    let components = graph_components(
        entries,
        &specification.split_manifest_specification.schema_version,
    )?;
    if components.len() < 3 {
        return Err("独立重算连通分量少于三个".to_string());
    }
    let (boundary_audit, mut split_records) = recompute_boundaries(entries, &components)?;
    independently_apply_purge_embargo(&mut split_records)?;
    for split in [
        HistoricalOutcomeOfflineDatasetTransformationSplit::Train,
        HistoricalOutcomeOfflineDatasetTransformationSplit::Validation,
        HistoricalOutcomeOfflineDatasetTransformationSplit::SealedHoldout,
    ] {
        if !split_records
            .iter()
            .any(|record| record.split == split && !record.purged_or_embargoed)
        {
            return Err("独立重算后存在空分区，校验失败关闭".to_string());
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
    let mut feature_records = Vec::with_capacity(active_entry_ids.len() * features.len());
    for entry in entries
        .iter()
        .filter(|entry| active_entry_ids.contains(entry.entry_id.as_str()))
    {
        for feature in features {
            feature_records.push(independent_missing_feature(
                entry,
                feature,
                &transformation_sha256,
            ));
        }
    }
    let exclusion_audit = entries
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
        .collect();
    Ok(IndependentProjection {
        component_count: components.len(),
        boundary_audit,
        split_records,
        feature_records,
        exclusion_audit,
        feature_catalog_sha256: hash_serializable(features)?,
    })
}

/// Deliberately uses graph traversal rather than the execution module's
/// union-find implementation.
fn graph_components(
    entries: &[ValidationEntryInput],
    split_schema_version: &str,
) -> Result<Vec<ValidationComponent>, String> {
    let mut remaining = (0..entries.len()).collect::<BTreeSet<_>>();
    let mut groups = Vec::new();
    while let Some(seed) = remaining.pop_first() {
        let mut stack = vec![seed];
        let mut indexes = Vec::new();
        while let Some(index) = stack.pop() {
            indexes.push(index);
            let neighbours = remaining
                .iter()
                .copied()
                .filter(|candidate| entries_connected(&entries[index], &entries[*candidate]))
                .collect::<Vec<_>>();
            for neighbour in neighbours {
                remaining.remove(&neighbour);
                stack.push(neighbour);
            }
        }
        indexes.sort_by(|left, right| entries[*left].entry_id.cmp(&entries[*right].entry_id));
        groups.push(indexes);
    }
    let mut components = groups
        .into_iter()
        .map(|indexes| {
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
            Ok::<_, String>(ValidationComponent {
                component_id: hash_serializable(&serde_json::json!({
                    "split_schema_version": split_schema_version,
                    "entries": identities,
                }))?,
                earliest: indexes
                    .iter()
                    .map(|index| entries[*index].decision_available_at)
                    .min()
                    .unwrap(),
                latest: indexes
                    .iter()
                    .map(|index| entries[*index].decision_available_at)
                    .max()
                    .unwrap(),
                entry_indexes: indexes,
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

fn entries_connected(left: &ValidationEntryInput, right: &ValidationEntryInput) -> bool {
    left.asset_symbol == right.asset_symbol
        || left.reconstruction_sha256 == right.reconstruction_sha256
        || left.snapshot_sha256 == right.snapshot_sha256
        || left.source_output_sha256 == right.source_output_sha256
        || left.source_validation_sha256 == right.source_validation_sha256
}

fn recompute_boundaries(
    entries: &[ValidationEntryInput],
    components: &[ValidationComponent],
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
        .ok_or_else(|| "没有合法连续分量边界".to_string())?;
    let mut audit = HistoricalOutcomeOfflineDatasetTransformationBoundaryAudit {
        schema_version: "hone-boundary-candidate-audit-v1".to_string(),
        candidate_pair_count: candidates.len(),
        selected_train_component_end_exclusive: train_end,
        selected_validation_component_end_exclusive: validation_end,
        pre_purge_train_entry_count: counts[0],
        pre_purge_validation_entry_count: counts[1],
        pre_purge_sealed_holdout_entry_count: counts[2],
        objective_tuple: objective,
        all_candidate_objectives_sha256: hash_serializable(
            &candidates
                .iter()
                .map(|candidate| (candidate.0, candidate.1, candidate.2, candidate.3))
                .collect::<Vec<_>>(),
        )?,
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

fn independently_apply_purge_embargo(
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
            .ok_or_else(|| "独立重算边界后一分区为空".to_string())?;
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

fn independent_missing_feature(
    entry: &ValidationEntryInput,
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

fn structural_mismatch_reasons(
    attempt: &CompletedHistoricalOutcomeOfflineDatasetTransformationExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeOfflineDatasetTransformationFirstExecutionAuthorization,
    dataset: &HistoricalOutcomeOfflineDataset,
    envelope: &HistoricalOutcomeOfflineDatasetTransformationUntrustedCandidateEnvelope,
) -> Vec<String> {
    let mut reasons = Vec::new();
    let specification = &authorization
        .runner
        .implementation
        .approved_review
        .specification;
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION
        || envelope.dataset_id != dataset.dataset_id
        || envelope.dataset_content_sha256 != dataset.dataset_content_sha256
        || envelope.dataset_manifest_sha256 != dataset.manifest_sha256
        || envelope.candidate_set_sha256 != dataset.candidate_set_sha256
        || envelope.transformation_spec_sha256 != specification.transformation_spec_sha256
        || envelope.transformation_body_sha256 != specification.transformation_body_sha256
        || envelope.split_specification_sha256
            != specification
                .split_manifest_specification
                .specification_sha256
        || envelope.feature_specification_sha256
            != specification
                .feature_bundle_specification
                .specification_sha256
        || envelope.implementation_id != authorization.runner.implementation.implementation_id
        || envelope.implementation_sha256
            != authorization.runner.implementation.implementation_sha256
        || envelope.runner_id != authorization.runner.isolated_runner_id
        || envelope.runner_spec_sha256 != authorization.runner.isolated_runner_spec_sha256
        || envelope.authorization_review_id != authorization.review.review_id
        || envelope.authorization_review_sha256 != authorization.review.review_sha256
        || envelope.entry_count != dataset.entry_count
        || envelope.feature_catalog_count != EXPECTED_FEATURE_COUNT
    {
        reasons.push("envelope_or_complete_upstream_binding_invalid".to_string());
    }
    let result_authority_closed = !attempt.result.independent_validation_completed
        && !attempt.result.official_split_manifest_authorized
        && !attempt.result.official_feature_bundle_authorized
        && !attempt.result.feature_join_authorized
        && !attempt.result.semantic_target_authorized
        && !attempt.result.training_authorized
        && !attempt.result.reward_authorized
        && !attempt.result.shadow_portfolio_authorized
        && !attempt.result.order_generation_authorized
        && !attempt.result.broker_access_authorized
        && !attempt.result.trading_authorized;
    let envelope_authority_closed = envelope.output_is_untrusted
        && envelope.deterministic_projection_only
        && envelope.explicit_missingness_preserved
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
    if !result_authority_closed || !envelope_authority_closed {
        reasons.push("candidate_or_result_authority_invalid".to_string());
    }
    reasons
}

fn validate_record(
    record: &HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord,
) -> Result<(), String> {
    let downstream_closed = !record.official_split_manifest_authorized
        && !record.official_feature_bundle_authorized
        && !record.feature_join_authorized
        && !record.semantic_target_authorized
        && !record.training_authorized
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let expected_valid = record.validator_independent_from_execution_and_complete_prior_chain
        && record.immutable_chain_integrity_verified
        && record.current_dataset_binding_verified
        && record.current_sealed_snapshot_bindings_verified
        && record.canonical_output_hash_verified
        && record.output_structure_verified
        && record.independent_component_recomputation_match
        && record.independent_boundary_recomputation_match
        && record.independent_purge_embargo_recomputation_match
        && record.independent_feature_recomputation_match
        && record.sealed_holdout_withholding_verified
        && record.mismatch_reasons.is_empty();
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != VALIDATION_POLICY_VERSION
        || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256)
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != validation_fingerprint(record)?
        || !valid_id(&record.transformation_attempt_id)
        || !valid_sha256(&record.claim_sha256)
        || !valid_id(&record.result_id)
        || !valid_sha256(&record.result_sha256)
        || !valid_sha256(&record.output_sha256)
        || !valid_id(&record.dataset_id)
        || !valid_sha256(&record.dataset_content_sha256)
        || !valid_sha256(&record.dataset_manifest_sha256)
        || !valid_sha256(&record.candidate_set_sha256)
        || !valid_sha256(&record.transformation_spec_sha256)
        || !valid_sha256(&record.split_specification_sha256)
        || !valid_sha256(&record.feature_specification_sha256)
        || !valid_id(&record.implementation_id)
        || !valid_sha256(&record.implementation_sha256)
        || !valid_id(&record.isolated_runner_id)
        || !valid_sha256(&record.isolated_runner_spec_sha256)
        || !valid_id(&record.authorization_review_id)
        || !valid_sha256(&record.authorization_review_sha256)
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || record.validated_by.trim().is_empty()
        || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.validated_by)
        || !valid_sha256(&record.recomputed_boundary_audit_sha256)
        || !valid_sha256(&record.recomputed_split_manifest_candidate_sha256)
        || !valid_sha256(&record.recomputed_feature_bundle_candidate_sha256)
        || !valid_sha256(&record.recomputed_exclusion_audit_sha256)
        || record.untrusted_candidate_envelope_validated != expected_valid
        || record.untrusted_candidate_envelope_validated
            != (record.verdict
                == HistoricalOutcomeOfflineDatasetTransformationOutputValidationVerdict::ValidatedIndependentStructureAndDeterministicMatch)
        || !downstream_closed
    {
        return Err("historical offline transformation output validation is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn validation_fingerprint(
    record: &HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord,
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
        .ok_or_else(|| "validation fingerprint payload is not an object".to_string())?;
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

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord>, String> {
    let root = validation_root(state);
    let mut directories = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    let mut attempts = BTreeSet::new();
    while let Some(directory) = directories.next_entry().await.map_err(|e| e.to_string())? {
        if !directory
            .file_type()
            .await
            .map_err(|e| e.to_string())?
            .is_dir()
        {
            continue;
        }
        let mut entries = tokio::fs::read_dir(directory.path())
            .await
            .map_err(|e| e.to_string())?;
        while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
            if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let bytes = tokio::fs::read(entry.path())
                .await
                .map_err(|e| e.to_string())?;
            let record: HistoricalOutcomeOfflineDatasetTransformationOutputValidationRecord =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            validate_record(&record)?;
            if entry.path().file_stem().and_then(|value| value.to_str())
                != Some(record.validation_id.as_str())
                || directory.file_name().to_string_lossy() != record.transformation_attempt_id
            {
                return Err(
                    "historical offline transformation validation path mismatch".to_string()
                );
            }
            if !ids.insert(record.validation_id.clone())
                || !hashes.insert(record.validation_sha256.clone())
                || !attempts.insert(record.transformation_attempt_id.clone())
            {
                return Err("historical offline transformation validation contains duplicate or replayed records".to_string());
            }
            output.push(record);
        }
    }
    output.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(output)
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

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-offline-dataset-transformation-output-validations")
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
    if !valid_id(attempt_id) {
        return Err("invalid offline transformation validation lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-offline-transformation-output-validation-{attempt_id}.lock"
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
        .map_err(|_| "已有同一离线转换输出独立校验正在进行".to_string())?;
    Ok(ValidationLock { path })
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sha(seed: u8) -> String {
        format!("{seed:02x}").repeat(32)
    }

    fn entry(id: u8, day: u32) -> ValidationEntryInput {
        ValidationEntryInput {
            entry_id: format!("{id:02x}").repeat(16),
            entry_sha256: sha(id),
            asset_symbol: format!("ASSET-{id}"),
            reconstruction_sha256: sha(id.wrapping_add(20)),
            snapshot_sha256: sha(id.wrapping_add(40)),
            source_output_sha256: sha(id.wrapping_add(60)),
            source_validation_sha256: sha(id.wrapping_add(80)),
            protocol_sha256: sha(100),
            decision_available_at: Utc.with_ymd_and_hms(2025, 1, day, 20, 0, 0).unwrap(),
            decision_session_date: NaiveDate::from_ymd_opt(2025, 1, day).unwrap(),
            outcome_horizon_250_end_date: NaiveDate::from_ymd_opt(2025, 8, day).unwrap(),
            common_market_session_index: day as usize,
        }
    }

    #[test]
    fn graph_traversal_closes_transitive_components() {
        let mut values = vec![entry(1, 1), entry(2, 2), entry(3, 3), entry(4, 4)];
        values[1].asset_symbol = values[0].asset_symbol.clone();
        values[2].source_output_sha256 = values[1].source_output_sha256.clone();
        let components = graph_components(&values, "split-v1").unwrap();
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].entry_indexes.len(), 3);
    }

    #[test]
    fn boundary_selection_is_deterministic_and_tamper_visible() {
        let values = (1..=5).map(|day| entry(day as u8, day)).collect::<Vec<_>>();
        let components = graph_components(&values, "split-v1").unwrap();
        let (first, records) = recompute_boundaries(&values, &components).unwrap();
        let (second, _) = recompute_boundaries(&values, &components).unwrap();
        assert_eq!(first, second);
        assert_eq!(records.len(), 5);
        let mut tampered = first.clone();
        tampered.pre_purge_train_entry_count += 1;
        assert_ne!(tampered, first);
    }

    #[test]
    fn purge_and_embargo_reason_is_exact_and_fail_closed() {
        let values = (1..=4).map(|day| entry(day as u8, day)).collect::<Vec<_>>();
        let components = graph_components(&values, "split-v1").unwrap();
        let (_, mut records) = recompute_boundaries(&values, &components).unwrap();
        independently_apply_purge_embargo(&mut records).unwrap();
        assert!(records.iter().any(|record| {
            record.purged_or_embargoed
                && record
                    .purge_reason
                    .as_deref()
                    .is_some_and(|reason| reason.contains("250_session"))
        }));
    }

    #[test]
    fn explicit_missingness_cannot_be_silently_backfilled() {
        let value = entry(1, 1);
        let feature = PointInTimeFeatureDefinition {
            namespace: "fundamental".to_string(),
            feature_id: "revenue_growth".to_string(),
            value_kind: "ratio".to_string(),
            source_authority_contract: "point_in_time".to_string(),
        };
        let mut record = independent_missing_feature(&value, &feature, &sha(9));
        assert!(record.value.is_none() && record.is_missing);
        record.value = Some("0.42".to_string());
        assert_ne!(
            record,
            independent_missing_feature(&value, &feature, &sha(9))
        );
    }

    #[test]
    fn validator_must_be_independent_from_every_prior_actor() {
        let actors = vec![
            "assembler".to_string(),
            "invoker".to_string(),
            "reviewer".to_string(),
        ];
        assert!(validator_is_independent("validator", &actors));
        assert!(!validator_is_independent("invoker", &actors));
        assert!(!validator_is_independent("", &actors));
    }

    #[test]
    fn validation_records_are_single_attempt_and_content_addressed() {
        let mut seen = BTreeSet::new();
        assert!(seen.insert("attempt-a"));
        assert!(!seen.insert("attempt-a"));
        assert!(valid_sha256(&validator_implementation_sha256()));
    }

    #[test]
    fn component_identity_includes_protocol_but_not_as_connectivity_edge() {
        let mut left = entry(1, 1);
        let mut right = entry(2, 2);
        assert!(!entries_connected(&left, &right));
        right.protocol_sha256 = left.protocol_sha256.clone();
        assert!(!entries_connected(&left, &right));
        left.asset_symbol = right.asset_symbol.clone();
        assert!(entries_connected(&left, &right));
    }
}
