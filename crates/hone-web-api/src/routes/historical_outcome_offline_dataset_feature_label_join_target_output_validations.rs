//! Independent validation of one Stage 42 feature/label join-target output.
//!
//! This module deliberately rebuilds the joined rows from the exact current
//! raw-outcome dataset and independently validated official split/feature
//! artifacts. It does not call the Stage 42 projection or envelope validator.
//! A passing record only makes the untrusted candidate eligible for a future
//! admission review. It does not create an official joined dataset, expose
//! validation/holdout targets, copy data to a training store, train, reward,
//! shadow, order, access a broker or trade.

use std::collections::{BTreeMap, BTreeSet, HashMap};
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
    CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit,
    HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope,
    HistoricalOutcomeFeatureLabelJoinTargetValue,
    HistoricalOutcomeFeatureLabelJoinTargetVisibility,
    completed_join_target_execution_attempt_for_independent_validation,
    completed_join_target_execution_attempts_for_independent_validation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_first_execution_authorizations::{
    ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
    exact_first_execution_authorization_for_join_target_audit,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_specs::{
    HistoricalOutcomeSemanticTargetDefinition, HistoricalOutcomeSemanticTargetSpecification,
};
use super::historical_outcome_offline_dataset_transformation_execution_attempts::{
    HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord,
    HistoricalOutcomeOfflineDatasetTransformationSplit,
};
use super::historical_outcome_offline_dataset_transformation_official_artifact_output_validations::{
    IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
    independently_validated_official_artifact_pairs_for_join_target_specification,
};
use super::historical_outcome_offline_datasets::{
    HistoricalOutcomeOfflineDataset, HistoricalOutcomeOfflineDatasetEntry,
    exact_current_historical_outcome_offline_dataset_for_transformation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-output-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-output-validation-v1";
const POLICY_VERSION: &str = "hone-feature-label-join-target-independent-output-recomputation-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-independent-join-target-validator-v1-no-stage42-projection-or-envelope-validator-reuse";
const ENVELOPE_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-untrusted-candidate-envelope-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];
const MAX_MISMATCH_REASONS: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeFeatureLabelJoinTargetOutputRequest {
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_split_manifest_sha256: String,
    expected_feature_bundle_sha256: String,
    expected_combined_artifact_sha256: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    independent_recomputation_confirmed: bool,
    validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    output_remains_untrusted_pending_admission_confirmed: bool,
    no_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetOutputValidationVerdict {
    ValidatedUntrustedCandidateForFutureAdmissionReview,
    FailedIndependentStructureOrRecomputationMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
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
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub execution_invoked_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_execution_and_complete_prior_chain: bool,
    pub immutable_claim_fingerprint_verified: bool,
    pub immutable_result_fingerprint_verified: bool,
    pub canonical_output_hash_verified: bool,
    pub exact_current_authorization_binding_verified: bool,
    pub exact_current_official_artifact_pair_verified: bool,
    pub exact_current_raw_outcome_dataset_verified: bool,
    pub exact_one_to_one_entry_join_recomputed: bool,
    pub exact_65_feature_catalog_recomputed: bool,
    pub point_in_time_and_explicit_missingness_recomputed: bool,
    pub official_purge_embargo_and_split_recomputed: bool,
    pub exact_nine_raw_f64_target_bits_recomputed: bool,
    pub target_commitments_recomputed: bool,
    pub train_only_target_exposure_verified: bool,
    pub validation_targets_withheld_verified: bool,
    pub sealed_holdout_targets_withheld_verified: bool,
    pub downstream_authority_closed_verified: bool,
    pub recomputed_rows_sha256: String,
    pub recomputed_excluded_rows_sha256: String,
    pub recomputed_target_commitments_sha256: String,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeFeatureLabelJoinTargetOutputValidationVerdict,
    pub untrusted_candidate_independently_validated: bool,
    pub future_candidate_admission_review_eligible: bool,
    pub official_joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOutputValidationItem {
    pub attempt: CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    pub validation: Option<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationItem>,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_untrusted_candidate_count: usize,
    pub failed_validation_count: usize,
    pub future_candidate_admission_review_eligible_count: usize,
    pub validation_status: String,
    pub independent_output_validation_available: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOutputValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub independently_validated_untrusted_candidate_count: usize,
    pub failed_validation_count: usize,
    pub future_candidate_admission_review_eligible_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate {
    pub attempt: CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    pub validation: HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord,
}

#[derive(Debug)]
struct IndependentAudit {
    rows: Vec<HistoricalOutcomeFeatureLabelJoinTargetCandidateRow>,
    excluded_rows: Vec<HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit>,
    target_commitments: Vec<(String, String)>,
    one_to_one: bool,
    feature_catalog_exact: bool,
    point_in_time_missingness_exact: bool,
    split_exact: bool,
    target_bits_exact: bool,
    commitments_exact: bool,
    train_only_exposure: bool,
    validation_withheld: bool,
    sealed_holdout_withheld: bool,
    downstream_closed: bool,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_output_validations(
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
            warn!(%error, "join/target output validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果 join/target 输出独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_feature_label_join_target_output(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeFeatureLabelJoinTargetOutputRequest>,
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

pub(crate) async fn historical_outcome_feature_label_join_target_output_validation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetOutputValidationReadinessSummary {
            validation_eligible_count: registry.validation_eligible_count,
            validation_count: registry.validation_count,
            independently_validated_untrusted_candidate_count: registry
                .independently_validated_untrusted_candidate_count,
            failed_validation_count: registry.failed_validation_count,
            future_candidate_admission_review_eligible_count: registry
                .future_candidate_admission_review_eligible_count,
            validation_status: registry.validation_status,
        },
    )
}

pub(crate) async fn independently_validated_join_target_candidates_for_admission_review(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate>, String> {
    let validations = read_validations(state).await?;
    let mut output = Vec::new();
    for validation in validations.into_iter().filter(|record| {
        record.untrusted_candidate_independently_validated
            && record.future_candidate_admission_review_eligible
    }) {
        validate_record(&validation)?;
        let Some(attempt) = completed_join_target_execution_attempt_for_independent_validation(
            state,
            &validation.attempt_id,
            &validation.claim_sha256,
            &validation.result_sha256,
            &validation.output_sha256,
        )
        .await?
        else {
            continue;
        };
        output.push(
            IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate {
                attempt,
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn exact_independently_validated_join_target_candidate_for_admission_review(
    state: &AppState,
    attempt_id: &str,
    validation_id: &str,
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
) -> Result<Option<IndependentlyValidatedHistoricalOutcomeFeatureLabelJoinTargetCandidate>, String>
{
    let candidates =
        independently_validated_join_target_candidates_for_admission_review(state).await?;
    Ok(candidates.into_iter().find(|value| {
        let record = &value.validation;
        record.attempt_id == attempt_id
            && record.validation_id == validation_id
            && record.validation_sha256 == validation_sha256
            && record.claim_sha256 == claim_sha256
            && record.result_sha256 == result_sha256
            && record.output_sha256 == output_sha256
            && record.authorization_review_sha256 == authorization_review_sha256
            && record.isolated_runner_spec_sha256 == isolated_runner_spec_sha256
            && record.implementation_sha256 == implementation_sha256
            && record.specification_sha256 == specification_sha256
            && record.join_specification_sha256 == join_specification_sha256
            && record.target_specification_sha256 == target_specification_sha256
            && record.split_manifest_sha256 == split_manifest_sha256
            && record.feature_bundle_sha256 == feature_bundle_sha256
            && record.combined_artifact_sha256 == combined_artifact_sha256
            && record.dataset_content_sha256 == dataset_content_sha256
            && record.dataset_manifest_sha256 == dataset_manifest_sha256
            && record.candidate_set_sha256 == candidate_set_sha256
            && record.recomputed_rows_sha256 == recomputed_rows_sha256
            && record.recomputed_excluded_rows_sha256 == recomputed_excluded_rows_sha256
            && record.recomputed_target_commitments_sha256 == recomputed_target_commitments_sha256
    }))
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry, String> {
    let attempts =
        completed_join_target_execution_attempts_for_independent_validation(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let items = attempts
        .into_iter()
        .map(|attempt| {
            let validation = index
                .get(attempt.claim.attempt_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeFeatureLabelJoinTargetOutputValidationItem {
                validation_eligible: validation.is_none(),
                attempt,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let validation_count = validations.len();
    let independently_validated_untrusted_candidate_count = validations
        .iter()
        .filter(|record| record.untrusted_candidate_independently_validated)
        .count();
    let failed_validation_count = validations
        .iter()
        .filter(|record| !record.untrusted_candidate_independently_validated)
        .count();
    let future_candidate_admission_review_eligible_count = validations
        .iter()
        .filter(|record| record.future_candidate_admission_review_eligible)
        .count();
    let validation_status = if failed_validation_count > 0 {
        "join_target_output_validation_failed_fail_closed"
    } else if future_candidate_admission_review_eligible_count > 0 {
        "join_target_untrusted_candidate_independently_validated_waiting_admission_review"
    } else if validation_eligible_count > 0 {
        "waiting_independent_join_target_output_recomputation"
    } else {
        "waiting_completed_join_target_untrusted_candidate"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        items,
        validation_eligible_count,
        validation_count,
        independently_validated_untrusted_candidate_count,
        failed_validation_count,
        future_candidate_admission_review_eligible_count,
        validation_status: validation_status.to_string(),
        independent_output_validation_available: true,
        official_joined_dataset_created: false,
        copied_to_training_store: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 43 阶段由完整上游链之外的独立管理员重新打开精确 claim/result/output、正式 split/feature 工件和当前原始结果数据集，独立重算一对一连接、65 项点时特征、九项原始 f64 位目标、承诺及 validation/封存隐藏。通过只开放未来候选准入复核资格；不创建正式 joined dataset，不复制训练库，不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    admin_id: &str,
    attempt_id: &str,
    request: ValidateHistoricalOutcomeFeatureLabelJoinTargetOutputRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord, String> {
    if !valid_id(attempt_id) {
        return Err("join/target 执行尝试 ID 无效".to_string());
    }
    if !request.independent_recomputation_confirmed
        || !request.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        || !request.output_remains_untrusted_pending_admission_confirmed
        || !request.no_training_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err("必须确认独立重算、目标隐藏、不可信输出和全部下游权限关闭".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.attempt_id == attempt_id)
    {
        return Err("这条 join/target 输出已经形成不可变独立校验记录，不得重放".to_string());
    }
    let attempt = completed_join_target_execution_attempt_for_independent_validation(
        state,
        attempt_id,
        &request.expected_claim_sha256,
        &request.expected_result_sha256,
        &request.expected_output_sha256,
    )
    .await?
    .ok_or_else(|| "join/target claim、result 或 canonical output 已变化".to_string())?;
    verify_expected_request_bindings(&attempt, &request)?;
    let authorization = exact_first_execution_authorization_for_join_target_audit(
        state,
        &attempt.claim.isolated_runner_id,
        &attempt.claim.authorization_review_id,
        &attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "精确 join/target 首次执行授权审计链不可用".to_string())?;
    let dataset = exact_current_historical_outcome_offline_dataset_for_transformation(
        state,
        &attempt.claim.dataset_id,
        &attempt.claim.dataset_content_sha256,
        &attempt.claim.dataset_manifest_sha256,
        &attempt.claim.candidate_set_sha256,
    )
    .await?
    .ok_or_else(|| "精确原始结果数据集已不是完整当前候选集".to_string())?;
    let artifact_pair = exact_artifact_pair(state, &attempt).await?;
    let excluded_prior_actor_ids = prior_actor_ids(&attempt, &authorization, &dataset);
    let independent = actor_is_independent(admin_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("独立校验人不能是执行调用人或完整上游链任一角色".to_string());
    }
    let envelope = attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "join/target 结果缺少不可信候选信封".to_string())?;
    let target_spec = &authorization
        .runner
        .implementation
        .approved_review
        .specification
        .target_specification;
    let audit = independently_recompute(
        &attempt,
        &authorization,
        &artifact_pair,
        &dataset,
        target_spec,
        envelope,
    )?;
    let immutable_claim_fingerprint_verified =
        fingerprint_without(&attempt.claim, &["attempt_id", "claim_sha256"])?
            == attempt.claim.claim_sha256;
    let immutable_result_fingerprint_verified =
        fingerprint_without(&attempt.result, &["result_id", "result_sha256"])?
            == attempt.result.result_sha256;
    let canonical_output_hash_verified =
        hash_serializable(envelope)? == request.expected_output_sha256;
    let exact_current_authorization_binding_verified =
        exact_authorization_binding(&attempt, &authorization);
    let exact_current_official_artifact_pair_verified =
        exact_artifact_binding(&attempt, &artifact_pair, &request);
    let exact_current_raw_outcome_dataset_verified = exact_dataset_binding(&attempt, &dataset);
    let mut mismatch_reasons = audit.mismatch_reasons.clone();
    if !immutable_claim_fingerprint_verified {
        mismatch_reasons.push("immutable_claim_fingerprint_mismatch".to_string());
    }
    if !immutable_result_fingerprint_verified {
        mismatch_reasons.push("immutable_result_fingerprint_mismatch".to_string());
    }
    if !canonical_output_hash_verified {
        mismatch_reasons.push("canonical_output_sha256_mismatch".to_string());
    }
    if !exact_current_authorization_binding_verified {
        mismatch_reasons.push("current_authorization_binding_mismatch".to_string());
    }
    if !exact_current_official_artifact_pair_verified {
        mismatch_reasons.push("current_official_artifact_pair_binding_mismatch".to_string());
    }
    if !exact_current_raw_outcome_dataset_verified {
        mismatch_reasons.push("current_raw_outcome_dataset_binding_mismatch".to_string());
    }
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    mismatch_reasons.truncate(MAX_MISMATCH_REASONS);
    let passed = independent
        && immutable_claim_fingerprint_verified
        && immutable_result_fingerprint_verified
        && canonical_output_hash_verified
        && exact_current_authorization_binding_verified
        && exact_current_official_artifact_pair_verified
        && exact_current_raw_outcome_dataset_verified
        && audit.one_to_one
        && audit.feature_catalog_exact
        && audit.point_in_time_missingness_exact
        && audit.split_exact
        && audit.target_bits_exact
        && audit.commitments_exact
        && audit.train_only_exposure
        && audit.validation_withheld
        && audit.sealed_holdout_withheld
        && audit.downstream_closed
        && mismatch_reasons.is_empty();
    let mut record = HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        attempt_id: attempt.claim.attempt_id.clone(),
        claim_sha256: attempt.claim.claim_sha256.clone(),
        result_id: attempt.result.result_id.clone(),
        result_sha256: attempt.result.result_sha256.clone(),
        output_sha256: request.expected_output_sha256,
        authorization_review_id: attempt.claim.authorization_review_id.clone(),
        authorization_review_sha256: attempt.claim.authorization_review_sha256.clone(),
        isolated_runner_id: attempt.claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: attempt.claim.isolated_runner_spec_sha256.clone(),
        implementation_id: attempt.claim.implementation_id.clone(),
        implementation_sha256: attempt.claim.implementation_sha256.clone(),
        specification_id: attempt.claim.specification_id.clone(),
        specification_sha256: attempt.claim.specification_sha256.clone(),
        join_specification_sha256: attempt.claim.join_specification_sha256.clone(),
        target_specification_sha256: attempt.claim.target_specification_sha256.clone(),
        split_manifest_sha256: attempt.claim.split_manifest_sha256.clone(),
        feature_bundle_sha256: attempt.claim.feature_bundle_sha256.clone(),
        combined_artifact_sha256: attempt.claim.combined_artifact_sha256.clone(),
        dataset_id: dataset.dataset_id.clone(),
        dataset_content_sha256: dataset.dataset_content_sha256.clone(),
        dataset_manifest_sha256: dataset.manifest_sha256.clone(),
        candidate_set_sha256: dataset.candidate_set_sha256.clone(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: admin_id.to_string(),
        execution_invoked_by: attempt.claim.invoked_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_execution_and_complete_prior_chain: independent,
        immutable_claim_fingerprint_verified,
        immutable_result_fingerprint_verified,
        canonical_output_hash_verified,
        exact_current_authorization_binding_verified,
        exact_current_official_artifact_pair_verified,
        exact_current_raw_outcome_dataset_verified,
        exact_one_to_one_entry_join_recomputed: audit.one_to_one,
        exact_65_feature_catalog_recomputed: audit.feature_catalog_exact,
        point_in_time_and_explicit_missingness_recomputed: audit.point_in_time_missingness_exact,
        official_purge_embargo_and_split_recomputed: audit.split_exact,
        exact_nine_raw_f64_target_bits_recomputed: audit.target_bits_exact,
        target_commitments_recomputed: audit.commitments_exact,
        train_only_target_exposure_verified: audit.train_only_exposure,
        validation_targets_withheld_verified: audit.validation_withheld,
        sealed_holdout_targets_withheld_verified: audit.sealed_holdout_withheld,
        downstream_authority_closed_verified: audit.downstream_closed,
        recomputed_rows_sha256: hash_serializable(&audit.rows)?,
        recomputed_excluded_rows_sha256: hash_serializable(&audit.excluded_rows)?,
        recomputed_target_commitments_sha256: hash_serializable(&audit.target_commitments)?,
        mismatch_reasons,
        verdict: if passed {
            HistoricalOutcomeFeatureLabelJoinTargetOutputValidationVerdict::ValidatedUntrustedCandidateForFutureAdmissionReview
        } else {
            HistoricalOutcomeFeatureLabelJoinTargetOutputValidationVerdict::FailedIndependentStructureOrRecomputationMismatch
        },
        untrusted_candidate_independently_validated: passed,
        future_candidate_admission_review_eligible: passed,
        official_joined_dataset_created: false,
        copied_to_training_store: false,
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
            .join(&record.attempt_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn verify_expected_request_bindings(
    attempt: &CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    request: &ValidateHistoricalOutcomeFeatureLabelJoinTargetOutputRequest,
) -> Result<(), String> {
    let output_matches =
        attempt.result.output_sha256.as_deref() == Some(request.expected_output_sha256.as_str());
    if attempt.claim.authorization_review_sha256 != request.expected_authorization_review_sha256
        || attempt.claim.split_manifest_sha256 != request.expected_split_manifest_sha256
        || attempt.claim.feature_bundle_sha256 != request.expected_feature_bundle_sha256
        || attempt.claim.combined_artifact_sha256 != request.expected_combined_artifact_sha256
        || attempt.claim.dataset_content_sha256 != request.expected_dataset_content_sha256
        || attempt.claim.dataset_manifest_sha256 != request.expected_dataset_manifest_sha256
        || attempt.claim.candidate_set_sha256 != request.expected_candidate_set_sha256
        || !output_matches
    {
        return Err("join/target 输出校验请求绑定已漂移".to_string());
    }
    Ok(())
}

async fn exact_artifact_pair(
    state: &AppState,
    attempt: &CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
) -> Result<IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair, String> {
    independently_validated_official_artifact_pairs_for_join_target_specification(state)
        .await?
        .into_iter()
        .find(|pair| {
            pair.validation.validation_id == attempt.claim.validation_id
                && pair.validation.validation_sha256 == attempt.claim.validation_sha256
                && pair.artifact_pair.split_manifest.manifest_sha256
                    == attempt.claim.split_manifest_sha256
                && pair.artifact_pair.feature_bundle.feature_bundle_sha256
                    == attempt.claim.feature_bundle_sha256
                && pair
                    .artifact_pair
                    .result
                    .combined_artifact_sha256
                    .as_deref()
                    == Some(attempt.claim.combined_artifact_sha256.as_str())
        })
        .ok_or_else(|| "精确独立校验正式工件对不可用".to_string())
}

fn independently_recompute(
    attempt: &CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
    artifact_pair: &IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
    dataset: &HistoricalOutcomeOfflineDataset,
    target_spec: &HistoricalOutcomeSemanticTargetSpecification,
    envelope: &HistoricalOutcomeFeatureLabelJoinTargetUntrustedCandidateEnvelope,
) -> Result<IndependentAudit, String> {
    let mut mismatch_reasons = Vec::new();
    let target_contract_valid = independently_validate_target_contract(target_spec);
    if !target_contract_valid {
        mismatch_reasons.push("independent_target_contract_invalid".to_string());
    }
    let manifest = &artifact_pair.artifact_pair.split_manifest;
    let bundle = &artifact_pair.artifact_pair.feature_bundle;
    let split_index = unique_index(
        &manifest.records,
        |record| record.dataset_entry_id.as_str(),
        "official split",
    )?;
    let dataset_index = unique_index(&dataset.entries, |entry| entry.entry_id.as_str(), "dataset")?;
    let one_to_one_keys = split_index.keys().copied().collect::<BTreeSet<_>>()
        == dataset_index.keys().copied().collect::<BTreeSet<_>>();
    let mut features_by_entry: BTreeMap<
        &str,
        Vec<HistoricalOutcomeOfflineDatasetTransformationFeatureCandidateRecord>,
    > = BTreeMap::new();
    let mut feature_keys = BTreeSet::new();
    for feature in &bundle.records {
        if !feature_keys.insert((
            feature.dataset_entry_id.as_str(),
            feature.feature_namespace.as_str(),
            feature.feature_id.as_str(),
        )) {
            mismatch_reasons.push("duplicate_feature_entry_namespace_id".to_string());
        }
        features_by_entry
            .entry(feature.dataset_entry_id.as_str())
            .or_default()
            .push(feature.clone());
    }
    let mut rows = Vec::new();
    let mut excluded_rows = Vec::new();
    let mut target_commitments = Vec::new();
    let mut canonical_feature_ids: Option<BTreeSet<(String, String)>> = None;
    let mut point_in_time_missingness_exact = true;
    let mut split_exact = one_to_one_keys;
    let mut feature_catalog_exact = bundle.feature_catalog_count == EXPECTED_FEATURE_COUNT;
    let mut target_bits_exact = target_contract_valid;
    let mut commitments_exact = true;
    for entry in &dataset.entries {
        let Some(split) = split_index.get(entry.entry_id.as_str()).copied() else {
            split_exact = false;
            mismatch_reasons.push(format!("missing_split:{}", entry.entry_id));
            continue;
        };
        if split.dataset_entry_sha256 != entry.entry_sha256
            || split.decision_available_at != entry.decision_available_at
        {
            split_exact = false;
            mismatch_reasons.push(format!("split_binding_mismatch:{}", entry.entry_id));
        }
        let features = features_by_entry
            .remove(entry.entry_id.as_str())
            .unwrap_or_default();
        if split.purged_or_embargoed {
            if !features.is_empty() {
                split_exact = false;
                mismatch_reasons.push(format!("purged_entry_has_features:{}", entry.entry_id));
            }
            excluded_rows.push(HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit {
                dataset_entry_id: entry.entry_id.clone(),
                dataset_entry_sha256: entry.entry_sha256.clone(),
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
            feature_catalog_exact = false;
            mismatch_reasons.push(format!("feature_count_not_65:{}", entry.entry_id));
        }
        let feature_ids = features
            .iter()
            .map(|feature| {
                (
                    feature.feature_namespace.clone(),
                    feature.feature_id.clone(),
                )
            })
            .collect::<BTreeSet<_>>();
        if feature_ids.len() != EXPECTED_FEATURE_COUNT {
            feature_catalog_exact = false;
        }
        if let Some(expected) = &canonical_feature_ids {
            if expected != &feature_ids {
                feature_catalog_exact = false;
            }
        } else {
            canonical_feature_ids = Some(feature_ids);
        }
        for feature in &features {
            let missingness_valid = if feature.is_missing {
                feature.value.is_none() && !feature.missingness_reason.trim().is_empty()
            } else {
                feature.value.is_some()
            };
            if feature.dataset_entry_id != entry.entry_id
                || feature.available_at_utc > entry.decision_available_at
                || !missingness_valid
            {
                point_in_time_missingness_exact = false;
            }
        }
        let targets = independently_recompute_target_values(entry, target_spec)?;
        if targets.len() != EXPECTED_TARGET_COUNT {
            target_bits_exact = false;
        }
        let commitment = hash_serializable(&targets)?;
        target_commitments.push((entry.entry_id.clone(), commitment.clone()));
        let (visibility, target_vector) = match split.split {
            HistoricalOutcomeOfflineDatasetTransformationSplit::Train => (
                HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets,
                Some(targets),
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
        let source_binding_sha256 = hash_serializable(&serde_json::json!({
            "dataset_entry_id": entry.entry_id,
            "dataset_entry_sha256": entry.entry_sha256,
            "split_record": split,
            "feature_records": features,
            "target_commitment_sha256": commitment,
        }))?;
        rows.push(HistoricalOutcomeFeatureLabelJoinTargetCandidateRow {
            dataset_entry_id: entry.entry_id.clone(),
            dataset_entry_sha256: entry.entry_sha256.clone(),
            split: split.split.clone(),
            component_id: split.component_id.clone(),
            decision_available_at: entry.decision_available_at,
            feature_records: features,
            target_visibility: visibility,
            target_vector,
            target_commitment_sha256: commitment,
            source_binding_sha256,
        });
    }
    if !features_by_entry.is_empty() {
        feature_catalog_exact = false;
        mismatch_reasons.push("orphan_feature_records".to_string());
    }
    rows.sort_by(|left, right| left.dataset_entry_id.cmp(&right.dataset_entry_id));
    excluded_rows.sort_by(|left, right| left.dataset_entry_id.cmp(&right.dataset_entry_id));
    target_commitments.sort();
    if rows != envelope.rows {
        mismatch_reasons.push("independently_recomputed_rows_mismatch".to_string());
    }
    if excluded_rows != envelope.excluded_rows {
        mismatch_reasons.push("independently_recomputed_excluded_rows_mismatch".to_string());
    }
    for row in &envelope.rows {
        let Some(commitment) = target_commitments
            .iter()
            .find(|(entry_id, _)| entry_id == &row.dataset_entry_id)
            .map(|(_, commitment)| commitment)
        else {
            commitments_exact = false;
            continue;
        };
        if commitment != &row.target_commitment_sha256 {
            commitments_exact = false;
        }
    }
    let train_only_exposure = envelope.rows.iter().all(|row| match row.target_visibility {
        HistoricalOutcomeFeatureLabelJoinTargetVisibility::TrainCandidateRawTargets => row
            .target_vector
            .as_ref()
            .is_some_and(|targets| targets.len() == EXPECTED_TARGET_COUNT),
        HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
        | HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld => {
            row.target_vector.is_none()
        }
    });
    let validation_withheld = envelope.validation_targets_withheld
        && envelope
            .rows
            .iter()
            .filter(|row| {
                row.target_visibility
                    == HistoricalOutcomeFeatureLabelJoinTargetVisibility::ValidationTargetsWithheld
            })
            .all(|row| row.target_vector.is_none());
    let sealed_holdout_withheld = envelope.sealed_holdout_targets_withheld
        && manifest.sealed_holdout_labels_withheld
        && envelope
            .rows
            .iter()
            .filter(|row| {
                row.target_visibility
                == HistoricalOutcomeFeatureLabelJoinTargetVisibility::SealedHoldoutTargetsWithheld
            })
            .all(|row| row.target_vector.is_none());
    let downstream_closed = envelope.output_is_untrusted
        && !envelope.independent_output_validation_completed
        && !envelope.official_joined_dataset_created
        && !envelope.copied_to_training_store
        && !envelope.training_started
        && !envelope.reward_written
        && !envelope.shadow_position_written
        && !envelope.order_generated
        && !envelope.broker_accessed
        && !envelope.trade_executed
        && !attempt.result.official_joined_dataset_authorized
        && !attempt.result.training_data_access_authorized
        && !attempt.result.training_authorized
        && !attempt.result.reward_authorized
        && !attempt.result.shadow_portfolio_authorized
        && !attempt.result.order_generation_authorized
        && !attempt.result.broker_access_authorized
        && !attempt.result.trading_authorized;
    let envelope_bindings = envelope.schema_version == ENVELOPE_SCHEMA_VERSION
        && envelope.authorization_review_sha256 == authorization.review.review_sha256
        && envelope.split_manifest_sha256 == manifest.manifest_sha256
        && envelope.feature_bundle_sha256 == bundle.feature_bundle_sha256
        && envelope.dataset_content_sha256 == dataset.dataset_content_sha256
        && envelope.dataset_manifest_sha256 == dataset.manifest_sha256
        && envelope.candidate_set_sha256 == dataset.candidate_set_sha256
        && envelope.dataset_entry_count == dataset.entries.len()
        && envelope.active_candidate_row_count == rows.len()
        && envelope.excluded_purge_or_embargo_row_count == excluded_rows.len()
        && envelope.feature_catalog_count == EXPECTED_FEATURE_COUNT
        && envelope.target_count == EXPECTED_TARGET_COUNT;
    if !envelope_bindings {
        mismatch_reasons.push("envelope_structure_or_binding_mismatch".to_string());
    }
    if !one_to_one_keys || rows.len() + excluded_rows.len() != dataset.entries.len() {
        mismatch_reasons.push("one_to_one_dataset_join_mismatch".to_string());
    }
    if !feature_catalog_exact {
        mismatch_reasons.push("feature_catalog_or_entry_feature_set_mismatch".to_string());
    }
    if !point_in_time_missingness_exact {
        mismatch_reasons.push("point_in_time_or_missingness_mismatch".to_string());
    }
    if !split_exact {
        mismatch_reasons.push("official_split_purge_or_embargo_mismatch".to_string());
    }
    if !target_bits_exact {
        mismatch_reasons.push("nine_raw_target_bits_mismatch".to_string());
    }
    if !commitments_exact {
        mismatch_reasons.push("target_commitment_mismatch".to_string());
    }
    if !train_only_exposure || !validation_withheld || !sealed_holdout_withheld {
        mismatch_reasons.push("target_visibility_or_withholding_mismatch".to_string());
    }
    if !downstream_closed {
        mismatch_reasons.push("downstream_authority_not_closed".to_string());
    }
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    Ok(IndependentAudit {
        one_to_one: one_to_one_keys
            && rows.len() + excluded_rows.len() == dataset.entries.len()
            && rows == envelope.rows
            && excluded_rows == envelope.excluded_rows
            && envelope_bindings,
        feature_catalog_exact,
        point_in_time_missingness_exact,
        split_exact,
        target_bits_exact,
        commitments_exact,
        train_only_exposure,
        validation_withheld,
        sealed_holdout_withheld,
        downstream_closed,
        rows,
        excluded_rows,
        target_commitments,
        mismatch_reasons,
    })
}

fn independently_validate_target_contract(
    target_spec: &HistoricalOutcomeSemanticTargetSpecification,
) -> bool {
    let definitions = target_spec
        .target_definitions
        .iter()
        .map(|definition| (definition.target_id.as_str(), definition))
        .collect::<HashMap<_, _>>();
    target_spec.target_definitions.len() == EXPECTED_TARGET_COUNT
        && target_spec.target_vector_order.len() == EXPECTED_TARGET_COUNT
        && definitions.len() == EXPECTED_TARGET_COUNT
        && target_spec
            .target_vector_order
            .iter()
            .all(|target_id| definitions.contains_key(target_id.as_str()))
        && target_spec
            .target_definitions
            .iter()
            .all(independently_allowed_target_definition)
        && target_spec.exact_f64_bits_preserved
        && !target_spec.normalization_allowed
        && !target_spec.winsorization_allowed
        && !target_spec.rank_transform_allowed
        && !target_spec.categorical_action_label_defined
        && !target_spec.buy_hold_sell_threshold_defined
        && !target_spec.portfolio_weight_target_defined
        && !target_spec.scalar_reward_defined
        && !target_spec.semantic_target_assignment_performed
}

fn independently_allowed_target_definition(
    definition: &HistoricalOutcomeSemanticTargetDefinition,
) -> bool {
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

fn independently_recompute_target_values(
    entry: &HistoricalOutcomeOfflineDatasetEntry,
    target_spec: &HistoricalOutcomeSemanticTargetSpecification,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetValue>, String> {
    let mut metrics = BTreeMap::new();
    for metric in &entry.raw_validated_metrics {
        if !REQUIRED_HORIZONS.contains(&metric.horizon_market_sessions)
            || !metric.asset_return.is_finite()
            || !metric.excess_return.is_finite()
            || !metric.asset_max_drawdown.is_finite()
            || metrics
                .insert(metric.horizon_market_sessions, metric)
                .is_some()
        {
            return Err(format!("条目 {} 原始结果窗口无效或重复", entry.entry_id));
        }
    }
    if metrics.keys().copied().collect::<Vec<_>>() != REQUIRED_HORIZONS {
        return Err(format!("条目 {} 缺少 20/60/250 日结果", entry.entry_id));
    }
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
            let metric = metrics
                .get(&definition.horizon_market_sessions)
                .ok_or_else(|| format!("条目 {} 缺少目标窗口", entry.entry_id))?;
            let bits = match definition.source_metric_field.as_str() {
                "asset_return" => metric.asset_return.to_bits(),
                "excess_return" => metric.excess_return.to_bits(),
                "asset_max_drawdown" => metric.asset_max_drawdown.to_bits(),
                _ => return Err(format!("目标 {target_id} 使用未允许字段")),
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

fn exact_authorization_binding(
    attempt: &CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
) -> bool {
    let runner = &authorization.runner;
    let implementation = &runner.implementation;
    let specification = &implementation.approved_review.specification;
    attempt.claim.authorization_review_id == authorization.review.review_id
        && attempt.claim.authorization_review_sha256 == authorization.review.review_sha256
        && attempt.claim.isolated_runner_id == runner.isolated_runner_id
        && attempt.claim.isolated_runner_spec_sha256 == runner.isolated_runner_spec_sha256
        && attempt.claim.implementation_id == implementation.implementation_id
        && attempt.claim.implementation_sha256 == implementation.implementation_sha256
        && attempt.claim.specification_id == specification.specification_id
        && attempt.claim.specification_sha256 == specification.specification_sha256
        && attempt.claim.join_specification_sha256
            == specification.join_specification.specification_sha256
        && attempt.claim.target_specification_sha256
            == specification.target_specification.specification_sha256
}

fn exact_artifact_binding(
    attempt: &CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    pair: &IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
    request: &ValidateHistoricalOutcomeFeatureLabelJoinTargetOutputRequest,
) -> bool {
    attempt.claim.split_manifest_sha256 == pair.artifact_pair.split_manifest.manifest_sha256
        && attempt.claim.feature_bundle_sha256
            == pair.artifact_pair.feature_bundle.feature_bundle_sha256
        && pair
            .artifact_pair
            .result
            .combined_artifact_sha256
            .as_deref()
            == Some(attempt.claim.combined_artifact_sha256.as_str())
        && request.expected_split_manifest_sha256 == attempt.claim.split_manifest_sha256
        && request.expected_feature_bundle_sha256 == attempt.claim.feature_bundle_sha256
        && request.expected_combined_artifact_sha256 == attempt.claim.combined_artifact_sha256
}

fn exact_dataset_binding(
    attempt: &CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    dataset: &HistoricalOutcomeOfflineDataset,
) -> bool {
    attempt.claim.dataset_id == dataset.dataset_id
        && attempt.claim.dataset_content_sha256 == dataset.dataset_content_sha256
        && attempt.claim.dataset_manifest_sha256 == dataset.manifest_sha256
        && attempt.claim.candidate_set_sha256 == dataset.candidate_set_sha256
        && dataset.entry_count == dataset.entries.len()
        && dataset.complete_candidate_set_frozen
        && dataset.point_in_time_lineage_preserved
        && dataset.entries.iter().all(|entry| {
            entry.raw_outcome_only
                && !entry.feature_vector_present
                && !entry.semantic_target_assigned
                && !entry.split_assigned
                && !entry.reward_present
        })
}

fn prior_actor_ids(
    attempt: &CompletedHistoricalOutcomeFeatureLabelJoinTargetExecutionAttempt,
    authorization: &ApprovedHistoricalOutcomeFeatureLabelJoinTargetFirstExecutionAuthorization,
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

fn actor_is_independent(actor: &str, excluded: &[String]) -> bool {
    !actor.trim().is_empty() && !excluded.iter().any(|value| value == actor)
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
            return Err(format!("{label} 存在重复 entry key"));
        }
    }
    Ok(index)
}

fn validate_record(
    record: &HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord,
) -> Result<(), String> {
    let hashes = [
        &record.validation_sha256,
        &record.claim_sha256,
        &record.result_sha256,
        &record.output_sha256,
        &record.authorization_review_sha256,
        &record.isolated_runner_spec_sha256,
        &record.implementation_sha256,
        &record.specification_sha256,
        &record.join_specification_sha256,
        &record.target_specification_sha256,
        &record.split_manifest_sha256,
        &record.feature_bundle_sha256,
        &record.combined_artifact_sha256,
        &record.dataset_content_sha256,
        &record.dataset_manifest_sha256,
        &record.candidate_set_sha256,
        &record.validator_implementation_sha256,
        &record.recomputed_rows_sha256,
        &record.recomputed_excluded_rows_sha256,
        &record.recomputed_target_commitments_sha256,
    ];
    let ids = [
        &record.validation_id,
        &record.attempt_id,
        &record.result_id,
        &record.authorization_review_id,
        &record.isolated_runner_id,
        &record.implementation_id,
        &record.specification_id,
        &record.dataset_id,
    ];
    let authority_closed = !record.official_joined_dataset_created
        && !record.copied_to_training_store
        && !record.training_authorized
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let passed = record.untrusted_candidate_independently_validated
        == (record.verdict
            == HistoricalOutcomeFeatureLabelJoinTargetOutputValidationVerdict::ValidatedUntrustedCandidateForFutureAdmissionReview)
        && record.future_candidate_admission_review_eligible
            == record.untrusted_candidate_independently_validated;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || ids.iter().any(|id| !valid_id(id))
        || hashes.iter().any(|hash| !valid_sha256(hash))
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != validation_fingerprint(record)?
        || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || !actor_is_independent(&record.validated_by, &record.excluded_prior_actor_ids)
        || record.execution_invoked_by.trim().is_empty()
        || record.mismatch_reasons.len() > MAX_MISMATCH_REASONS
        || !passed
        || !authority_closed
    {
        return Err("join/target 独立输出校验记录无效或越权".to_string());
    }
    if record.untrusted_candidate_independently_validated
        && (!record.validator_independent_from_execution_and_complete_prior_chain
            || !record.immutable_claim_fingerprint_verified
            || !record.immutable_result_fingerprint_verified
            || !record.canonical_output_hash_verified
            || !record.exact_current_authorization_binding_verified
            || !record.exact_current_official_artifact_pair_verified
            || !record.exact_current_raw_outcome_dataset_verified
            || !record.exact_one_to_one_entry_join_recomputed
            || !record.exact_65_feature_catalog_recomputed
            || !record.point_in_time_and_explicit_missingness_recomputed
            || !record.official_purge_embargo_and_split_recomputed
            || !record.exact_nine_raw_f64_target_bits_recomputed
            || !record.target_commitments_recomputed
            || !record.train_only_target_exposure_verified
            || !record.validation_targets_withheld_verified
            || !record.sealed_holdout_targets_withheld_verified
            || !record.downstream_authority_closed_verified
            || !record.mismatch_reasons.is_empty())
    {
        return Err("通过的 join/target 独立校验记录缺少完整证明".to_string());
    }
    Ok(())
}

fn validation_fingerprint(
    record: &HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord,
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
    decision_root(state).join("historical-outcome-feature-label-join-target-output-validations")
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord>, String> {
    let root = validation_root(state);
    let mut directories = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
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
            let record: HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord =
                serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
            validate_record(&record)?;
            let directory_name = directory.file_name().to_string_lossy().to_string();
            if directory_name != record.attempt_id
                || entry.path().file_stem().and_then(|value| value.to_str())
                    != Some(record.validation_id.as_str())
                || !attempts.insert(record.attempt_id.clone())
                || !ids.insert(record.validation_id.clone())
                || !hashes.insert(record.validation_sha256.clone())
            {
                return Err("join/target 独立输出校验登记含重放、误命名或重复哈希".to_string());
            }
            output.push(record);
        }
    }
    output.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(output)
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
        "historical-feature-label-join-target-output-validation-{attempt_id}.lock"
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
        .map_err(|_| "已有同一 join/target 独立输出校验正在进行".to_string())?;
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

    fn minimal_record() -> HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord {
        let mut record = HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord {
            schema_version: RECORD_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            validation_id: String::new(),
            validation_sha256: String::new(),
            attempt_id: "1".repeat(32),
            claim_sha256: "2".repeat(64),
            result_id: "3".repeat(32),
            result_sha256: "4".repeat(64),
            output_sha256: "5".repeat(64),
            authorization_review_id: "6".repeat(32),
            authorization_review_sha256: "7".repeat(64),
            isolated_runner_id: "8".repeat(32),
            isolated_runner_spec_sha256: "9".repeat(64),
            implementation_id: "a".repeat(32),
            implementation_sha256: "b".repeat(64),
            specification_id: "c".repeat(32),
            specification_sha256: "d".repeat(64),
            join_specification_sha256: "e".repeat(64),
            target_specification_sha256: "f".repeat(64),
            split_manifest_sha256: "1".repeat(64),
            feature_bundle_sha256: "2".repeat(64),
            combined_artifact_sha256: "3".repeat(64),
            dataset_id: "4".repeat(32),
            dataset_content_sha256: "5".repeat(64),
            dataset_manifest_sha256: "6".repeat(64),
            candidate_set_sha256: "7".repeat(64),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.into(),
            validator_implementation_sha256: validator_implementation_sha256(),
            validated_at: DateTime::from_timestamp(1_780_000_000, 0).unwrap(),
            validated_by: "independent-validator".into(),
            execution_invoked_by: "executor".into(),
            excluded_prior_actor_ids: vec!["executor".into(), "upstream-reviewer".into()],
            validator_independent_from_execution_and_complete_prior_chain: true,
            immutable_claim_fingerprint_verified: true,
            immutable_result_fingerprint_verified: true,
            canonical_output_hash_verified: true,
            exact_current_authorization_binding_verified: true,
            exact_current_official_artifact_pair_verified: true,
            exact_current_raw_outcome_dataset_verified: true,
            exact_one_to_one_entry_join_recomputed: true,
            exact_65_feature_catalog_recomputed: true,
            point_in_time_and_explicit_missingness_recomputed: true,
            official_purge_embargo_and_split_recomputed: true,
            exact_nine_raw_f64_target_bits_recomputed: true,
            target_commitments_recomputed: true,
            train_only_target_exposure_verified: true,
            validation_targets_withheld_verified: true,
            sealed_holdout_targets_withheld_verified: true,
            downstream_authority_closed_verified: true,
            recomputed_rows_sha256: "8".repeat(64),
            recomputed_excluded_rows_sha256: "9".repeat(64),
            recomputed_target_commitments_sha256: "a".repeat(64),
            mismatch_reasons: Vec::new(),
            verdict: HistoricalOutcomeFeatureLabelJoinTargetOutputValidationVerdict::ValidatedUntrustedCandidateForFutureAdmissionReview,
            untrusted_candidate_independently_validated: true,
            future_candidate_admission_review_eligible: true,
            official_joined_dataset_created: false,
            copied_to_training_store: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        record.validation_sha256 = validation_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        record
    }

    fn rehash(record: &mut HistoricalOutcomeFeatureLabelJoinTargetOutputValidationRecord) {
        record.validation_id.clear();
        record.validation_sha256.clear();
        record.validation_sha256 = validation_fingerprint(record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
    }

    #[test]
    fn valid_independent_validation_record_passes() {
        assert!(validate_record(&minimal_record()).is_ok());
    }

    #[test]
    fn validation_self_hash_tamper_fails() {
        let mut record = minimal_record();
        record.validation_sha256 = "f".repeat(64);
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn validator_cannot_be_an_upstream_actor() {
        assert!(!actor_is_independent(
            "executor",
            &["executor".into(), "reviewer".into()]
        ));
        assert!(actor_is_independent(
            "independent",
            &["executor".into(), "reviewer".into()]
        ));
    }

    #[test]
    fn passed_record_requires_every_independent_proof() {
        let mut record = minimal_record();
        record.target_commitments_recomputed = false;
        rehash(&mut record);
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn passed_record_cannot_open_training_authority() {
        let mut record = minimal_record();
        record.training_authorized = true;
        rehash(&mut record);
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn passed_record_cannot_open_trading_authority() {
        let mut record = minimal_record();
        record.trading_authorized = true;
        rehash(&mut record);
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn failed_verdict_cannot_be_admission_eligible() {
        let mut record = minimal_record();
        record.verdict = HistoricalOutcomeFeatureLabelJoinTargetOutputValidationVerdict::FailedIndependentStructureOrRecomputationMismatch;
        record.untrusted_candidate_independently_validated = false;
        rehash(&mut record);
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn excluded_actor_chain_must_be_sorted_and_unique() {
        let mut record = minimal_record();
        record.excluded_prior_actor_ids = vec!["z".into(), "a".into()];
        rehash(&mut record);
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn canonical_f64_bit_hex_preserves_negative_zero() {
        assert_eq!(format!("{:016x}", (-0.0_f64).to_bits()), "8000000000000000");
    }

    #[test]
    fn duplicate_entry_keys_fail_closed() {
        #[derive(Clone)]
        struct Item(String);
        let items = vec![Item("same".into()), Item("same".into())];
        assert!(unique_index(&items, |item| item.0.as_str(), "test").is_err());
    }
}
