//! Create-once materialization of one exact Stage 44 admitted feature/label
//! join-target candidate into an official joined dataset.
//!
//! The claim is persisted before the admitted candidate is copied. Success,
//! failure or interruption consumes the eligibility. The materializer does not
//! recompute, repair, impute or reinterpret any row. A successful artifact is
//! still unusable until a later independent post-materialization validator
//! reopens it. This stage does not copy to a training store, train, reward,
//! shadow, generate orders, access a broker or trade.

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
    exact_admitted_join_target_candidate_for_official_joined_dataset_materialization,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_execution_attempts::{
    HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
    HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit,
    HistoricalOutcomeFeatureLabelJoinTargetVisibility,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-feature-label-join-target-official-dataset-materialization-registry-v1";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-official-dataset-materialization-claim-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-official-dataset-materialization-result-v1";
const OFFICIAL_DATASET_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-official-joined-dataset-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-official-dataset-create-once-exact-copy-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const MAX_OFFICIAL_DATASET_BYTES: usize = 64 * 1024 * 1024;
const MAX_ERROR_CHARS: usize = 1_200;

#[derive(Debug, Deserialize)]
pub(crate) struct MaterializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest {
    expected_admission_review_id: String,
    expected_admission_review_sha256: String,
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
    exact_admitted_candidate_copy_only_confirmed: bool,
    create_once_and_failure_consumes_confirmed: bool,
    validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    independent_post_materialization_validation_required_confirmed: bool,
    no_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
    pub schema_version: String,
    pub official_joined_dataset_sha256: String,
    pub materialization_id: String,
    pub attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub source_claim_sha256: String,
    pub source_result_id: String,
    pub source_result_sha256: String,
    pub source_output_sha256: String,
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
    pub source_recomputed_rows_sha256: String,
    pub source_recomputed_excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
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
    pub exact_admitted_candidate_copy: bool,
    pub one_to_one_join_preserved: bool,
    pub point_in_time_and_explicit_missingness_preserved: bool,
    pub official_split_purge_and_embargo_preserved: bool,
    pub exact_raw_f64_bits_preserved: bool,
    pub validation_targets_withheld: bool,
    pub sealed_holdout_targets_withheld: bool,
    pub action_position_and_reward_semantics_absent: bool,
    pub official_joined_dataset_created: bool,
    pub independently_validated_after_materialization: bool,
    pub eligible_for_training_store_copy: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim {
    pub schema_version: String,
    pub policy_version: String,
    pub materialization_id: String,
    pub claim_sha256: String,
    pub attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub source_claim_sha256: String,
    pub source_result_id: String,
    pub source_result_sha256: String,
    pub source_output_sha256: String,
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
    pub expected_recomputed_rows_sha256: String,
    pub expected_recomputed_excluded_rows_sha256: String,
    pub expected_recomputed_target_commitments_sha256: String,
    pub expected_dataset_entry_count: usize,
    pub expected_active_row_count: usize,
    pub expected_excluded_row_count: usize,
    pub expected_feature_catalog_count: usize,
    pub expected_target_count: usize,
    pub materialized_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub claimed_at: DateTime<Utc>,
    pub exact_current_admission_chain_revalidated: bool,
    pub materializer_independent_from_complete_prior_chain: bool,
    pub exact_admitted_candidate_copy_only_confirmed: bool,
    pub create_once_and_failure_consumes_confirmed: bool,
    pub validation_and_sealed_holdout_targets_remain_withheld_confirmed: bool,
    pub independent_post_materialization_validation_required_confirmed: bool,
    pub no_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub claim_consumed: bool,
    pub official_joined_dataset_materialization_started: bool,
    pub ambient_environment_available: bool,
    pub network_available: bool,
    pub external_tools_available: bool,
    pub production_state_write_available: bool,
    pub historical_state_mutation_allowed: bool,
    pub training_store_write_allowed: bool,
    pub training_allowed: bool,
    pub reward_allowed: bool,
    pub shadow_portfolio_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus {
    CompletedPendingIndependentValidation,
    FailedClaimConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult {
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub materialization_id: String,
    pub claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub status: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus,
    pub error: Option<String>,
    pub official_joined_dataset_sha256: Option<String>,
    pub official_joined_dataset_bytes: usize,
    pub official_joined_dataset_created: bool,
    pub exact_admitted_candidate_copy_completed: bool,
    pub independent_post_materialization_validation_completed: bool,
    pub eligible_for_training_store_copy: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationAttempt {
    pub claim: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    pub result: Option<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult>,
    pub official_joined_dataset: Option<HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationItem {
    pub admitted_candidate: AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
    pub attempt:
        Option<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationAttempt>,
    pub materialization_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationItem>,
    pub admitted_candidate_count: usize,
    pub materialization_eligible_count: usize,
    pub claim_count: usize,
    pub completed_materialization_count: usize,
    pub failed_materialization_count: usize,
    pub pending_independent_validation_count: usize,
    pub materialization_status: String,
    pub create_once_materialization_available: bool,
    pub official_joined_dataset_created: bool,
    pub independently_validated_after_materialization: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationReadinessSummary
{
    pub admitted_candidate_count: usize,
    pub materialization_eligible_count: usize,
    pub claim_count: usize,
    pub completed_materialization_count: usize,
    pub failed_materialization_count: usize,
    pub pending_independent_validation_count: usize,
    pub materialization_status: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct MaterializedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
    pub admitted_candidate: AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
    pub attempt: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationAttempt,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_official_dataset_materializations(
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
            warn!(%error, "official joined dataset materialization registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果正式 joined dataset 物化暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_materialize_historical_outcome_feature_label_join_target_official_dataset_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<MaterializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest>,
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

pub(crate) async fn historical_outcome_feature_label_join_target_official_dataset_materialization_readiness_summary(
    state: &AppState,
) -> Result<
    HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationReadinessSummary,
    String,
> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationReadinessSummary {
            admitted_candidate_count: registry.admitted_candidate_count,
            materialization_eligible_count: registry.materialization_eligible_count,
            claim_count: registry.claim_count,
            completed_materialization_count: registry.completed_materialization_count,
            failed_materialization_count: registry.failed_materialization_count,
            pending_independent_validation_count: registry.pending_independent_validation_count,
            materialization_status: registry.materialization_status,
        },
    )
}

#[allow(dead_code)] // Stage 46 will be the separately implemented post-materialization validator.
pub(crate) async fn materialized_official_joined_datasets_for_independent_validation(
    state: &AppState,
) -> Result<Vec<MaterializedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset>, String> {
    let admitted =
        admitted_join_target_candidates_for_official_joined_dataset_materialization(state).await?;
    let mut values = Vec::new();
    for candidate in admitted {
        let Some(attempt) = read_attempt(state, &candidate).await? else {
            continue;
        };
        if attempt_completed(&attempt) {
            values.push(
                MaterializedHistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
                    admitted_candidate: candidate,
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
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry, String> {
    let admitted =
        admitted_join_target_candidates_for_official_joined_dataset_materialization(state).await?;
    let mut items = Vec::with_capacity(admitted.len());
    for candidate in admitted {
        let attempt = read_attempt(state, &candidate).await?;
        items.push(
            HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationItem {
                materialization_eligible: attempt.is_none(),
                admitted_candidate: candidate,
                attempt,
            },
        );
    }
    let admitted_candidate_count = items.len();
    let materialization_eligible_count = items
        .iter()
        .filter(|item| item.materialization_eligible)
        .count();
    let claim_count = items.iter().filter(|item| item.attempt.is_some()).count();
    let completed_materialization_count = items
        .iter()
        .filter(|item| item.attempt.as_ref().is_some_and(attempt_completed))
        .count();
    let failed_materialization_count = items
        .iter()
        .filter(|item| {
            item.attempt
                .as_ref()
                .and_then(|attempt| attempt.result.as_ref())
                .is_some_and(|result| {
                    result.status
                        == HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus::FailedClaimConsumed
                })
        })
        .count();
    let pending_independent_validation_count = completed_materialization_count;
    let materialization_status = if failed_materialization_count > 0 {
        "official_joined_dataset_materialization_failed_claim_consumed"
    } else if pending_independent_validation_count > 0 {
        "official_joined_dataset_created_waiting_independent_post_materialization_validation"
    } else if claim_count > 0 {
        "official_joined_dataset_materialization_claim_consumed_waiting_result"
    } else if materialization_eligible_count > 0 {
        "waiting_create_once_official_joined_dataset_materialization"
    } else {
        "waiting_independently_admitted_join_target_candidate"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        admitted_candidate_count,
        materialization_eligible_count,
        claim_count,
        completed_materialization_count,
        failed_materialization_count,
        pending_independent_validation_count,
        materialization_status: materialization_status.to_string(),
        create_once_materialization_available: true,
        official_joined_dataset_created: completed_materialization_count > 0,
        independently_validated_after_materialization: false,
        copied_to_training_store: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 45 阶段只允许独立角色对一条精确 Stage 44 已准入候选先 claim、再 create-once 原样物化为 official joined dataset；失败也消费资格。成功产物仍待另一实现独立重开校验，不复制训练库、不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

fn attempt_completed(
    attempt: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationAttempt,
) -> bool {
    attempt.result.as_ref().is_some_and(|result| {
        result.status
            == HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus::CompletedPendingIndependentValidation
            && result.official_joined_dataset_created
            && !result.independent_post_materialization_validation_completed
            && attempt.official_joined_dataset.is_some()
    })
}

async fn materialize_once(
    state: &AppState,
    materialized_by: &str,
    attempt_id: &str,
    request: MaterializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult, String> {
    if !valid_id(attempt_id) {
        return Err("join/target 执行尝试 ID 无效".to_string());
    }
    if !request.exact_admitted_candidate_copy_only_confirmed
        || !request.create_once_and_failure_consumes_confirmed
        || !request.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        || !request.independent_post_materialization_validation_required_confirmed
        || !request.no_training_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err(
            "必须确认精确复制、失败消费、留出标签继续隐藏、后置独立校验和零下游权限五项边界"
                .to_string(),
        );
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let admitted =
        exact_admitted_join_target_candidate_for_official_joined_dataset_materialization(
            state,
            attempt_id,
            &request.expected_admission_review_id,
            &request.expected_admission_review_sha256,
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
        .ok_or_else(|| "当前没有这条精确绑定且已独立准入的 join/target 候选".to_string())?;
    if read_attempt(state, &admitted).await?.is_some() {
        return Err("这条候选已经领取过一次性正式 joined dataset 物化 claim，不能重放".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&admitted);
    if !actor_is_independent(materialized_by, &excluded_prior_actor_ids) {
        return Err(
            "正式 joined dataset 物化人不能是准入复核、输出校验、执行或完整上游角色".to_string(),
        );
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

    let built = build_official_dataset(&claim, &admitted);
    let result = match built {
        Ok(dataset) => {
            validate_official_dataset(&dataset, &claim, &admitted)?;
            let bytes = serde_json::to_vec(&dataset).map_err(|error| error.to_string())?;
            if bytes.len() > MAX_OFFICIAL_DATASET_BYTES {
                new_failed_result(
                    &claim,
                    "正式 joined dataset 超过固定 64 MiB 上限",
                    bytes.len(),
                )?
            } else {
                let artifact_path = directory.join(format!(
                    "official-joined-dataset-{}.json",
                    dataset.official_joined_dataset_sha256
                ));
                match write_immutable_json(&artifact_path, &dataset).await {
                    Ok(()) => new_success_result(&claim, &dataset, bytes.len())?,
                    Err(error) => new_failed_result(
                        &claim,
                        &format!("正式 joined dataset 写入失败：{error}"),
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

fn new_claim(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
    materialized_by: &str,
    excluded_prior_actor_ids: Vec<String>,
    request: &MaterializeHistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetRequest,
    claimed_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim, String> {
    let review = &admitted.admission_review;
    let validation = &admitted.candidate.validation;
    let mut claim = HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim {
        schema_version: CLAIM_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        materialization_id: String::new(),
        claim_sha256: String::new(),
        attempt_id: review.attempt_id.clone(),
        admission_review_id: review.review_id.clone(),
        admission_review_sha256: review.review_sha256.clone(),
        validation_id: review.validation_id.clone(),
        validation_sha256: review.validation_sha256.clone(),
        source_claim_sha256: review.claim_sha256.clone(),
        source_result_id: review.result_id.clone(),
        source_result_sha256: review.result_sha256.clone(),
        source_output_sha256: review.output_sha256.clone(),
        authorization_review_id: review.authorization_review_id.clone(),
        authorization_review_sha256: review.authorization_review_sha256.clone(),
        isolated_runner_id: review.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: review.isolated_runner_spec_sha256.clone(),
        implementation_id: review.implementation_id.clone(),
        implementation_sha256: review.implementation_sha256.clone(),
        specification_id: review.specification_id.clone(),
        specification_sha256: review.specification_sha256.clone(),
        join_specification_sha256: review.join_specification_sha256.clone(),
        target_specification_sha256: review.target_specification_sha256.clone(),
        split_manifest_sha256: review.split_manifest_sha256.clone(),
        feature_bundle_sha256: review.feature_bundle_sha256.clone(),
        combined_artifact_sha256: review.combined_artifact_sha256.clone(),
        dataset_id: review.dataset_id.clone(),
        dataset_content_sha256: review.dataset_content_sha256.clone(),
        dataset_manifest_sha256: review.dataset_manifest_sha256.clone(),
        candidate_set_sha256: review.candidate_set_sha256.clone(),
        expected_recomputed_rows_sha256: validation.recomputed_rows_sha256.clone(),
        expected_recomputed_excluded_rows_sha256: validation
            .recomputed_excluded_rows_sha256
            .clone(),
        expected_recomputed_target_commitments_sha256: validation
            .recomputed_target_commitments_sha256
            .clone(),
        expected_dataset_entry_count: review.dataset_entry_count,
        expected_active_row_count: review.active_candidate_row_count,
        expected_excluded_row_count: review.excluded_purge_or_embargo_row_count,
        expected_feature_catalog_count: review.feature_catalog_count,
        expected_target_count: review.target_count,
        materialized_by: materialized_by.to_string(),
        excluded_prior_actor_ids,
        claimed_at,
        exact_current_admission_chain_revalidated: true,
        materializer_independent_from_complete_prior_chain: true,
        exact_admitted_candidate_copy_only_confirmed: request
            .exact_admitted_candidate_copy_only_confirmed,
        create_once_and_failure_consumes_confirmed: request
            .create_once_and_failure_consumes_confirmed,
        validation_and_sealed_holdout_targets_remain_withheld_confirmed: request
            .validation_and_sealed_holdout_targets_remain_withheld_confirmed,
        independent_post_materialization_validation_required_confirmed: request
            .independent_post_materialization_validation_required_confirmed,
        no_training_reward_shadow_order_broker_or_trading_confirmed: request
            .no_training_reward_shadow_order_broker_or_trading_confirmed,
        claim_consumed: true,
        official_joined_dataset_materialization_started: true,
        ambient_environment_available: false,
        network_available: false,
        external_tools_available: false,
        production_state_write_available: false,
        historical_state_mutation_allowed: false,
        training_store_write_allowed: false,
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

fn build_official_dataset(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset, String> {
    let review = &admitted.admission_review;
    let envelope = admitted
        .candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "已准入 join/target 候选缺少源输出信封".to_string())?;
    let mut target_commitments = envelope
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
    let rows_sha256 = hash_serializable(&envelope.rows)?;
    let excluded_rows_sha256 = hash_serializable(&envelope.excluded_rows)?;
    let target_commitments_sha256 = hash_serializable(&target_commitments)?;
    if rows_sha256 != claim.expected_recomputed_rows_sha256
        || excluded_rows_sha256 != claim.expected_recomputed_excluded_rows_sha256
        || target_commitments_sha256 != claim.expected_recomputed_target_commitments_sha256
    {
        return Err("源候选内容与独立重算哈希不一致，禁止物化".to_string());
    }
    let mut dataset = HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
        schema_version: OFFICIAL_DATASET_SCHEMA_VERSION.to_string(),
        official_joined_dataset_sha256: String::new(),
        materialization_id: claim.materialization_id.clone(),
        attempt_id: claim.attempt_id.clone(),
        admission_review_id: claim.admission_review_id.clone(),
        admission_review_sha256: claim.admission_review_sha256.clone(),
        validation_id: claim.validation_id.clone(),
        validation_sha256: claim.validation_sha256.clone(),
        source_claim_sha256: claim.source_claim_sha256.clone(),
        source_result_id: claim.source_result_id.clone(),
        source_result_sha256: claim.source_result_sha256.clone(),
        source_output_sha256: claim.source_output_sha256.clone(),
        authorization_review_id: claim.authorization_review_id.clone(),
        authorization_review_sha256: claim.authorization_review_sha256.clone(),
        isolated_runner_id: claim.isolated_runner_id.clone(),
        isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256.clone(),
        implementation_id: claim.implementation_id.clone(),
        implementation_sha256: claim.implementation_sha256.clone(),
        specification_id: claim.specification_id.clone(),
        specification_sha256: claim.specification_sha256.clone(),
        join_specification_sha256: claim.join_specification_sha256.clone(),
        target_specification_sha256: claim.target_specification_sha256.clone(),
        split_manifest_sha256: claim.split_manifest_sha256.clone(),
        feature_bundle_sha256: claim.feature_bundle_sha256.clone(),
        combined_artifact_sha256: claim.combined_artifact_sha256.clone(),
        dataset_id: claim.dataset_id.clone(),
        dataset_content_sha256: claim.dataset_content_sha256.clone(),
        dataset_manifest_sha256: claim.dataset_manifest_sha256.clone(),
        candidate_set_sha256: claim.candidate_set_sha256.clone(),
        source_recomputed_rows_sha256: rows_sha256,
        source_recomputed_excluded_rows_sha256: excluded_rows_sha256,
        target_commitments_sha256,
        dataset_entry_count: review.dataset_entry_count,
        active_row_count: envelope.active_candidate_row_count,
        excluded_purge_or_embargo_row_count: envelope.excluded_purge_or_embargo_row_count,
        feature_catalog_count: envelope.feature_catalog_count,
        target_count: envelope.target_count,
        train_target_vector_count: envelope.train_target_vector_count,
        validation_target_withheld_count: envelope.validation_target_withheld_count,
        sealed_holdout_target_withheld_count: envelope.sealed_holdout_target_withheld_count,
        rows: envelope.rows.clone(),
        excluded_rows: envelope.excluded_rows.clone(),
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
    };
    dataset.official_joined_dataset_sha256 =
        fingerprint_without(&dataset, &["official_joined_dataset_sha256"])?;
    Ok(dataset)
}

fn new_success_result(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    bytes: usize,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult, String> {
    let mut result = HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        materialization_id: claim.materialization_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        status: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus::CompletedPendingIndependentValidation,
        error: None,
        official_joined_dataset_sha256: Some(dataset.official_joined_dataset_sha256.clone()),
        official_joined_dataset_bytes: bytes,
        official_joined_dataset_created: true,
        exact_admitted_candidate_copy_completed: true,
        independent_post_materialization_validation_completed: false,
        eligible_for_training_store_copy: false,
        copied_to_training_store: false,
        training_authorized: false,
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
    claim: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    error: &str,
    bytes: usize,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult, String> {
    let mut result = HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        materialization_id: claim.materialization_id.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        status: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus::FailedClaimConsumed,
        error: Some(error.chars().take(MAX_ERROR_CHARS).collect()),
        official_joined_dataset_sha256: None,
        official_joined_dataset_bytes: bytes,
        official_joined_dataset_created: false,
        exact_admitted_candidate_copy_completed: false,
        independent_post_materialization_validation_completed: false,
        eligible_for_training_store_copy: false,
        copied_to_training_store: false,
        training_authorized: false,
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
    result: &mut HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult,
) -> Result<(), String> {
    result.result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].to_string();
    Ok(())
}

fn validate_claim(
    claim: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
) -> Result<(), String> {
    let review = &admitted.admission_review;
    let downstream_closed = !claim.ambient_environment_available
        && !claim.network_available
        && !claim.external_tools_available
        && !claim.production_state_write_available
        && !claim.historical_state_mutation_allowed
        && !claim.training_store_write_allowed
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
        || claim.attempt_id != review.attempt_id
        || claim.admission_review_id != review.review_id
        || claim.admission_review_sha256 != review.review_sha256
        || claim.validation_id != review.validation_id
        || claim.validation_sha256 != review.validation_sha256
        || claim.source_claim_sha256 != review.claim_sha256
        || claim.source_result_id != review.result_id
        || claim.source_result_sha256 != review.result_sha256
        || claim.source_output_sha256 != review.output_sha256
        || claim.authorization_review_id != review.authorization_review_id
        || claim.authorization_review_sha256 != review.authorization_review_sha256
        || claim.isolated_runner_id != review.isolated_runner_id
        || claim.isolated_runner_spec_sha256 != review.isolated_runner_spec_sha256
        || claim.implementation_id != review.implementation_id
        || claim.implementation_sha256 != review.implementation_sha256
        || claim.specification_id != review.specification_id
        || claim.specification_sha256 != review.specification_sha256
        || claim.join_specification_sha256 != review.join_specification_sha256
        || claim.target_specification_sha256 != review.target_specification_sha256
        || claim.split_manifest_sha256 != review.split_manifest_sha256
        || claim.feature_bundle_sha256 != review.feature_bundle_sha256
        || claim.combined_artifact_sha256 != review.combined_artifact_sha256
        || claim.dataset_id != review.dataset_id
        || claim.dataset_content_sha256 != review.dataset_content_sha256
        || claim.dataset_manifest_sha256 != review.dataset_manifest_sha256
        || claim.candidate_set_sha256 != review.candidate_set_sha256
        || claim.expected_recomputed_rows_sha256 != review.recomputed_rows_sha256
        || claim.expected_recomputed_excluded_rows_sha256 != review.recomputed_excluded_rows_sha256
        || claim.expected_recomputed_target_commitments_sha256
            != review.recomputed_target_commitments_sha256
        || claim.expected_dataset_entry_count != review.dataset_entry_count
        || claim.expected_active_row_count != review.active_candidate_row_count
        || claim.expected_excluded_row_count != review.excluded_purge_or_embargo_row_count
        || claim.expected_feature_catalog_count != EXPECTED_FEATURE_COUNT
        || claim.expected_feature_catalog_count != review.feature_catalog_count
        || claim.expected_target_count != EXPECTED_TARGET_COUNT
        || claim.expected_target_count != review.target_count
        || claim.materialized_by.trim().is_empty()
        || claim.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&claim.excluded_prior_actor_ids)
        || !actor_is_independent(&claim.materialized_by, &claim.excluded_prior_actor_ids)
        || !claim.exact_current_admission_chain_revalidated
        || !claim.materializer_independent_from_complete_prior_chain
        || !claim.exact_admitted_candidate_copy_only_confirmed
        || !claim.create_once_and_failure_consumes_confirmed
        || !claim.validation_and_sealed_holdout_targets_remain_withheld_confirmed
        || !claim.independent_post_materialization_validation_required_confirmed
        || !claim.no_training_reward_shadow_order_broker_or_trading_confirmed
        || !claim.claim_consumed
        || !claim.official_joined_dataset_materialization_started
        || !review.join_target_candidate_admitted
        || !review.future_create_once_official_joined_dataset_materialization_eligible
        || !downstream_closed
    {
        return Err(
            "official joined dataset materialization claim is invalid, drifted or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_official_dataset(
    dataset: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset,
    claim: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
) -> Result<(), String> {
    let envelope = admitted
        .candidate
        .attempt
        .result
        .untrusted_candidate_envelope
        .as_ref()
        .ok_or_else(|| "已准入 join/target 候选缺少源输出信封".to_string())?;
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
    let closed = !dataset.independently_validated_after_materialization
        && !dataset.eligible_for_training_store_copy
        && !dataset.copied_to_training_store
        && !dataset.training_authorized
        && !dataset.reward_authorized
        && !dataset.shadow_portfolio_authorized
        && !dataset.order_generation_authorized
        && !dataset.broker_access_authorized
        && !dataset.trading_authorized;
    if dataset.schema_version != OFFICIAL_DATASET_SCHEMA_VERSION
        || !valid_sha256(&dataset.official_joined_dataset_sha256)
        || dataset.official_joined_dataset_sha256
            != fingerprint_without(dataset, &["official_joined_dataset_sha256"])?
        || dataset.materialization_id != claim.materialization_id
        || dataset.attempt_id != claim.attempt_id
        || dataset.admission_review_id != claim.admission_review_id
        || dataset.admission_review_sha256 != claim.admission_review_sha256
        || dataset.validation_id != claim.validation_id
        || dataset.validation_sha256 != claim.validation_sha256
        || dataset.source_claim_sha256 != claim.source_claim_sha256
        || dataset.source_result_id != claim.source_result_id
        || dataset.source_result_sha256 != claim.source_result_sha256
        || dataset.source_output_sha256 != claim.source_output_sha256
        || dataset.authorization_review_id != claim.authorization_review_id
        || dataset.authorization_review_sha256 != claim.authorization_review_sha256
        || dataset.isolated_runner_id != claim.isolated_runner_id
        || dataset.isolated_runner_spec_sha256 != claim.isolated_runner_spec_sha256
        || dataset.implementation_id != claim.implementation_id
        || dataset.implementation_sha256 != claim.implementation_sha256
        || dataset.specification_id != claim.specification_id
        || dataset.specification_sha256 != claim.specification_sha256
        || dataset.join_specification_sha256 != claim.join_specification_sha256
        || dataset.target_specification_sha256 != claim.target_specification_sha256
        || dataset.split_manifest_sha256 != claim.split_manifest_sha256
        || dataset.feature_bundle_sha256 != claim.feature_bundle_sha256
        || dataset.combined_artifact_sha256 != claim.combined_artifact_sha256
        || dataset.dataset_id != claim.dataset_id
        || dataset.dataset_content_sha256 != claim.dataset_content_sha256
        || dataset.dataset_manifest_sha256 != claim.dataset_manifest_sha256
        || dataset.candidate_set_sha256 != claim.candidate_set_sha256
        || dataset.source_recomputed_rows_sha256 != claim.expected_recomputed_rows_sha256
        || dataset.source_recomputed_excluded_rows_sha256
            != claim.expected_recomputed_excluded_rows_sha256
        || dataset.target_commitments_sha256 != claim.expected_recomputed_target_commitments_sha256
        || dataset.target_commitments_sha256 != hash_serializable(&commitments)?
        || dataset.dataset_entry_count != claim.expected_dataset_entry_count
        || dataset.active_row_count != claim.expected_active_row_count
        || dataset.excluded_purge_or_embargo_row_count != claim.expected_excluded_row_count
        || dataset.active_row_count + dataset.excluded_purge_or_embargo_row_count
            != dataset.dataset_entry_count
        || dataset.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || dataset.target_count != EXPECTED_TARGET_COUNT
        || dataset.rows != envelope.rows
        || dataset.excluded_rows != envelope.excluded_rows
        || !dataset.exact_admitted_candidate_copy
        || !dataset.one_to_one_join_preserved
        || !dataset.point_in_time_and_explicit_missingness_preserved
        || !dataset.official_split_purge_and_embargo_preserved
        || !dataset.exact_raw_f64_bits_preserved
        || !dataset.validation_targets_withheld
        || !dataset.sealed_holdout_targets_withheld
        || !dataset.action_position_and_reward_semantics_absent
        || !dataset.official_joined_dataset_created
        || !target_visibility_valid
        || !feature_catalog_valid
        || !dataset
            .excluded_rows
            .iter()
            .all(|row| !row.target_values_opened)
        || !closed
    {
        return Err("official joined dataset is invalid, drifted or over-authorized".to_string());
    }
    Ok(())
}

fn validate_result(
    result: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult,
    claim: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    dataset: Option<&HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset>,
) -> Result<(), String> {
    let success = result.status
        == HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus::CompletedPendingIndependentValidation;
    let success_fields = if success {
        let dataset = dataset.ok_or_else(|| {
            "successful materialization misses official joined dataset".to_string()
        })?;
        result.error.is_none()
            && result.official_joined_dataset_sha256.as_deref()
                == Some(dataset.official_joined_dataset_sha256.as_str())
            && result.official_joined_dataset_bytes > 0
            && result.official_joined_dataset_bytes <= MAX_OFFICIAL_DATASET_BYTES
            && result.official_joined_dataset_created
            && result.exact_admitted_candidate_copy_completed
    } else {
        dataset.is_none()
            && result.error.as_ref().is_some_and(|value| !value.is_empty())
            && result.official_joined_dataset_sha256.is_none()
            && !result.official_joined_dataset_created
            && !result.exact_admitted_candidate_copy_completed
    };
    let downstream_closed = !result.independent_post_materialization_validation_completed
        && !result.eligible_for_training_store_copy
        && !result.copied_to_training_store
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
            "official joined dataset materialization result is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

async fn read_attempt(
    state: &AppState,
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
) -> Result<
    Option<HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationAttempt>,
    String,
> {
    let directory = materialization_directory(state, &admitted.admission_review.attempt_id);
    let claim_bytes = match tokio::fs::read(directory.join("claim.json")).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let claim: HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim =
        serde_json::from_slice(&claim_bytes).map_err(|error| error.to_string())?;
    validate_claim(&claim, admitted)?;
    let result_bytes = match tokio::fs::read(directory.join("result.json")).await {
        Ok(value) => Some(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.to_string()),
    };
    let result: Option<
        HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationResult,
    > = result_bytes
        .as_ref()
        .map(|bytes| serde_json::from_slice(bytes).map_err(|error| error.to_string()))
        .transpose()?;
    let official_joined_dataset = if let Some(result) = &result {
        match result.official_joined_dataset_sha256.as_deref() {
            Some(sha256) => {
                let bytes = tokio::fs::read(
                    directory.join(format!("official-joined-dataset-{sha256}.json")),
                )
                .await
                .map_err(|error| error.to_string())?;
                let dataset: HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset =
                    serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
                validate_official_dataset(&dataset, &claim, admitted)?;
                Some(dataset)
            }
            None => None,
        }
    } else {
        None
    };
    if let Some(result) = &result {
        validate_result(result, &claim, official_joined_dataset.as_ref())?;
    }
    Ok(Some(
        HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationAttempt {
            claim,
            result,
            official_joined_dataset,
        },
    ))
}

fn excluded_actor_ids(
    admitted: &AdmittedHistoricalOutcomeFeatureLabelJoinTargetCandidate,
) -> Vec<String> {
    let mut values = admitted.admission_review.excluded_prior_actor_ids.clone();
    values.push(admitted.admission_review.reviewer_id.clone());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded_actor_ids: &[String]) -> bool {
    !actor_id.trim().is_empty() && !excluded_actor_ids.iter().any(|value| value == actor_id)
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

fn materialization_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("feature-label-join-target-official-dataset-materializations")
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
    let root = materialization_directory(state, attempt_id);
    tokio::fs::create_dir_all(&root)
        .await
        .map_err(|error| error.to_string())?;
    let path = root.join(".materialization.lock");
    let mut stale_lock_removed = false;
    loop {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(MaterializationLock { path }),
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
                return Err("这条候选已有正式 joined dataset 物化正在进行".to_string());
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

    fn minimal_claim() -> HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim
    {
        let mut claim =
            HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim {
                schema_version: CLAIM_SCHEMA_VERSION.into(),
                policy_version: POLICY_VERSION.into(),
                materialization_id: String::new(),
                claim_sha256: String::new(),
                attempt_id: "1".repeat(32),
                admission_review_id: "2".repeat(32),
                admission_review_sha256: "3".repeat(64),
                validation_id: "4".repeat(32),
                validation_sha256: "5".repeat(64),
                source_claim_sha256: "6".repeat(64),
                source_result_id: "7".repeat(32),
                source_result_sha256: "8".repeat(64),
                source_output_sha256: "9".repeat(64),
                authorization_review_id: "a".repeat(32),
                authorization_review_sha256: "b".repeat(64),
                isolated_runner_id: "c".repeat(32),
                isolated_runner_spec_sha256: "d".repeat(64),
                implementation_id: "e".repeat(32),
                implementation_sha256: "f".repeat(64),
                specification_id: "1".repeat(32),
                specification_sha256: "2".repeat(64),
                join_specification_sha256: "3".repeat(64),
                target_specification_sha256: "4".repeat(64),
                split_manifest_sha256: "5".repeat(64),
                feature_bundle_sha256: "6".repeat(64),
                combined_artifact_sha256: "7".repeat(64),
                dataset_id: "8".repeat(32),
                dataset_content_sha256: "9".repeat(64),
                dataset_manifest_sha256: "a".repeat(64),
                candidate_set_sha256: "b".repeat(64),
                expected_recomputed_rows_sha256: hash_serializable(&Vec::<
                    HistoricalOutcomeFeatureLabelJoinTargetCandidateRow,
                >::new())
                .unwrap(),
                expected_recomputed_excluded_rows_sha256: hash_serializable(&Vec::<
                    HistoricalOutcomeFeatureLabelJoinTargetExcludedRowAudit,
                >::new(
                ))
                .unwrap(),
                expected_recomputed_target_commitments_sha256: hash_serializable(&Vec::<(
                    String,
                    String,
                )>::new(
                ))
                .unwrap(),
                expected_dataset_entry_count: 0,
                expected_active_row_count: 0,
                expected_excluded_row_count: 0,
                expected_feature_catalog_count: EXPECTED_FEATURE_COUNT,
                expected_target_count: EXPECTED_TARGET_COUNT,
                materialized_by: "materializer".into(),
                excluded_prior_actor_ids: vec!["admitter".into(), "validator".into()],
                claimed_at: Utc::now(),
                exact_current_admission_chain_revalidated: true,
                materializer_independent_from_complete_prior_chain: true,
                exact_admitted_candidate_copy_only_confirmed: true,
                create_once_and_failure_consumes_confirmed: true,
                validation_and_sealed_holdout_targets_remain_withheld_confirmed: true,
                independent_post_materialization_validation_required_confirmed: true,
                no_training_reward_shadow_order_broker_or_trading_confirmed: true,
                claim_consumed: true,
                official_joined_dataset_materialization_started: true,
                ambient_environment_available: false,
                network_available: false,
                external_tools_available: false,
                production_state_write_available: false,
                historical_state_mutation_allowed: false,
                training_store_write_allowed: false,
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

    fn minimal_dataset(
        claim: &HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationClaim,
    ) -> HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
        let mut dataset = HistoricalOutcomeFeatureLabelJoinTargetOfficialDataset {
            schema_version: OFFICIAL_DATASET_SCHEMA_VERSION.into(),
            official_joined_dataset_sha256: String::new(),
            materialization_id: claim.materialization_id.clone(),
            attempt_id: claim.attempt_id.clone(),
            admission_review_id: claim.admission_review_id.clone(),
            admission_review_sha256: claim.admission_review_sha256.clone(),
            validation_id: claim.validation_id.clone(),
            validation_sha256: claim.validation_sha256.clone(),
            source_claim_sha256: claim.source_claim_sha256.clone(),
            source_result_id: claim.source_result_id.clone(),
            source_result_sha256: claim.source_result_sha256.clone(),
            source_output_sha256: claim.source_output_sha256.clone(),
            authorization_review_id: claim.authorization_review_id.clone(),
            authorization_review_sha256: claim.authorization_review_sha256.clone(),
            isolated_runner_id: claim.isolated_runner_id.clone(),
            isolated_runner_spec_sha256: claim.isolated_runner_spec_sha256.clone(),
            implementation_id: claim.implementation_id.clone(),
            implementation_sha256: claim.implementation_sha256.clone(),
            specification_id: claim.specification_id.clone(),
            specification_sha256: claim.specification_sha256.clone(),
            join_specification_sha256: claim.join_specification_sha256.clone(),
            target_specification_sha256: claim.target_specification_sha256.clone(),
            split_manifest_sha256: claim.split_manifest_sha256.clone(),
            feature_bundle_sha256: claim.feature_bundle_sha256.clone(),
            combined_artifact_sha256: claim.combined_artifact_sha256.clone(),
            dataset_id: claim.dataset_id.clone(),
            dataset_content_sha256: claim.dataset_content_sha256.clone(),
            dataset_manifest_sha256: claim.dataset_manifest_sha256.clone(),
            candidate_set_sha256: claim.candidate_set_sha256.clone(),
            source_recomputed_rows_sha256: claim.expected_recomputed_rows_sha256.clone(),
            source_recomputed_excluded_rows_sha256: claim
                .expected_recomputed_excluded_rows_sha256
                .clone(),
            target_commitments_sha256: claim.expected_recomputed_target_commitments_sha256.clone(),
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
        };
        dataset.official_joined_dataset_sha256 =
            fingerprint_without(&dataset, &["official_joined_dataset_sha256"]).unwrap();
        dataset
    }

    #[test]
    fn materializer_must_be_outside_complete_prior_actor_set() {
        let actors = vec!["admitter".to_string(), "validator".to_string()];
        assert!(actor_is_independent("materializer", &actors));
        assert!(!actor_is_independent("validator", &actors));
    }

    #[test]
    fn claim_hash_binds_training_and_trading_boundaries() {
        let claim = minimal_claim();
        let first = fingerprint_without(&claim, &["materialization_id", "claim_sha256"]).unwrap();
        let mut changed = claim;
        changed.training_store_write_allowed = true;
        assert_ne!(
            first,
            fingerprint_without(&changed, &["materialization_id", "claim_sha256"]).unwrap()
        );
    }

    #[test]
    fn official_dataset_remains_unvalidated_and_not_trainable() {
        let dataset = minimal_dataset(&minimal_claim());
        assert!(dataset.official_joined_dataset_created);
        assert!(!dataset.independently_validated_after_materialization);
        assert!(!dataset.eligible_for_training_store_copy);
        assert!(!dataset.training_authorized);
        assert!(!dataset.trading_authorized);
    }

    #[test]
    fn successful_result_never_opens_downstream_authority() {
        let claim = minimal_claim();
        let dataset = minimal_dataset(&claim);
        let result = new_success_result(&claim, &dataset, 1_024).unwrap();
        assert!(result.official_joined_dataset_created);
        assert!(!result.independent_post_materialization_validation_completed);
        assert!(!result.copied_to_training_store);
        assert!(!result.training_authorized);
        assert!(!result.trading_authorized);
    }

    #[test]
    fn official_dataset_fingerprint_detects_target_commitment_tamper() {
        let dataset = minimal_dataset(&minimal_claim());
        let first = dataset.official_joined_dataset_sha256.clone();
        let mut changed = dataset;
        changed.target_commitments_sha256 = "f".repeat(64);
        assert_ne!(
            first,
            fingerprint_without(&changed, &["official_joined_dataset_sha256"]).unwrap()
        );
    }

    #[test]
    fn result_hash_binds_official_dataset_digest() {
        let claim = minimal_claim();
        let dataset = minimal_dataset(&claim);
        let result = new_success_result(&claim, &dataset, 1_024).unwrap();
        let first = result.result_sha256.clone();
        let mut changed = result;
        changed.official_joined_dataset_sha256 = Some("e".repeat(64));
        assert_ne!(
            first,
            fingerprint_without(&changed, &["result_id", "result_sha256"]).unwrap()
        );
    }

    #[test]
    fn excluded_actor_ids_must_be_strictly_sorted_and_unique() {
        assert!(sorted_unique(&["a".into(), "b".into()]));
        assert!(!sorted_unique(&["b".into(), "a".into()]));
        assert!(!sorted_unique(&["a".into(), "a".into()]));
    }

    #[test]
    fn failed_result_consumes_claim_without_creating_dataset() {
        let claim = minimal_claim();
        let result = new_failed_result(&claim, "copy failed", 0).unwrap();
        assert_eq!(result.status, HistoricalOutcomeFeatureLabelJoinTargetOfficialDatasetMaterializationStatus::FailedClaimConsumed);
        assert!(!result.official_joined_dataset_created);
        assert!(result.official_joined_dataset_sha256.is_none());
        assert!(!result.training_authorized);
    }
}
