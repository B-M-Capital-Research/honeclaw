//! Immutable registration of one feature-label join and semantic-target
//! governance specification for an independently validated official offline
//! artifact pair.
//!
//! Registration freezes the exact join keys, split-specific label visibility,
//! point-in-time and missingness rules, and a continuous forward-outcome target
//! vector. It does not execute a join, assign a target to any row, create a
//! training dataset, train, reward, shadow, order, access a broker or trade.

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

use super::historical_outcome_offline_dataset_transformation_official_artifact_output_validations::{
    IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
    independently_validated_official_artifact_pairs_for_join_target_specification,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-spec-registry-v1";
const RECORD_SCHEMA_VERSION: &str = "hone-historical-outcome-feature-label-join-target-spec-v1";
const JOIN_SPEC_SCHEMA_VERSION: &str = "hone-historical-outcome-exact-feature-label-join-spec-v1";
const TARGET_SPEC_SCHEMA_VERSION: &str = "hone-historical-outcome-continuous-target-vector-spec-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-governance-registration-v1-no-execution";
const STATUS: &str = "registered_not_independently_reviewed_not_executed";
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];
const EXPECTED_FEATURE_COUNT: usize = 65;
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 3_000;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeFeatureLabelJoinTargetSpecRequest {
    expected_validation_id: String,
    expected_validation_sha256: String,
    expected_materialization_id: String,
    expected_materialization_claim_sha256: String,
    expected_materialization_result_sha256: String,
    expected_split_manifest_sha256: String,
    expected_feature_bundle_sha256: String,
    expected_combined_artifact_sha256: String,
    expected_dataset_id: String,
    expected_dataset_content_sha256: String,
    expected_dataset_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    specification_name: String,
    code_revision: String,
    rationale: String,
    known_limitations: String,
    exact_validated_artifact_pair_binding_confirmed: bool,
    registrar_independence_confirmed: bool,
    exact_dataset_entry_one_to_one_join_confirmed: bool,
    purged_and_embargoed_rows_excluded_confirmed: bool,
    point_in_time_feature_availability_confirmed: bool,
    sealed_holdout_target_isolation_confirmed: bool,
    exact_raw_metric_bits_without_transform_confirmed: bool,
    continuous_target_vector_not_action_or_reward_confirmed: bool,
    explicit_missingness_without_imputation_confirmed: bool,
    registration_review_execution_separation_confirmed: bool,
    no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub split_manifest_sha256: String,
    pub feature_bundle_sha256: String,
    pub combined_artifact_sha256: String,
    pub dataset_entry_key: String,
    pub split_record_key: String,
    pub feature_record_key_fields: Vec<String>,
    pub raw_outcome_record_key: String,
    pub join_cardinality_rule: String,
    pub split_authority_rule: String,
    pub purged_or_embargoed_row_policy: String,
    pub train_target_visibility_policy: String,
    pub validation_target_visibility_policy: String,
    pub sealed_holdout_target_visibility_policy: String,
    pub feature_availability_rule: String,
    pub explicit_missingness_rule: String,
    pub feature_catalog_count: usize,
    pub feature_catalog_sha256: String,
    pub feature_schema_sha256: String,
    pub allowed_label_horizons_market_sessions: Vec<u16>,
    pub forbidden_join_inputs: Vec<String>,
    pub joined_row_schema_fields: Vec<String>,
    pub one_to_one_outcome_join_required: bool,
    pub all_allowlisted_feature_records_preserved: bool,
    pub imputation_allowed: bool,
    pub interpolation_allowed: bool,
    pub sealed_holdout_labels_opened: bool,
    pub join_executed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSemanticTargetDefinition {
    pub target_id: String,
    pub horizon_market_sessions: u16,
    pub source_metric_field: String,
    pub source_selector: String,
    pub value_kind: String,
    pub unit: String,
    pub transformation: String,
    pub role: String,
    pub semantics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSemanticTargetSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub prediction_task: String,
    pub target_definitions: Vec<HistoricalOutcomeSemanticTargetDefinition>,
    pub primary_supervised_target_id: String,
    pub risk_target_id: String,
    pub auxiliary_target_ids: Vec<String>,
    pub benchmark_return_role: String,
    pub target_vector_order: Vec<String>,
    pub duplicate_horizon_policy: String,
    pub missing_horizon_policy: String,
    pub train_target_access_policy: String,
    pub validation_target_access_policy: String,
    pub sealed_holdout_target_access_policy: String,
    pub exact_f64_bits_preserved: bool,
    pub normalization_allowed: bool,
    pub winsorization_allowed: bool,
    pub rank_transform_allowed: bool,
    pub categorical_action_label_defined: bool,
    pub buy_hold_sell_threshold_defined: bool,
    pub portfolio_weight_target_defined: bool,
    pub scalar_reward_defined: bool,
    pub semantic_target_assignment_performed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub specification_id: String,
    pub specification_sha256: String,
    pub transformation_attempt_id: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub materialization_id: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_sha256: String,
    pub split_manifest_sha256: String,
    pub feature_bundle_sha256: String,
    pub combined_artifact_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_complete_prior_chain: bool,
    pub specification_name: String,
    pub code_revision: String,
    pub rationale: String,
    pub known_limitations: String,
    pub join_specification: HistoricalOutcomeFeatureLabelJoinSpecification,
    pub target_specification: HistoricalOutcomeSemanticTargetSpecification,
    pub specification_body_sha256: String,
    pub status: String,
    pub exact_validated_artifact_pair_binding_confirmed: bool,
    pub registrar_independence_confirmed: bool,
    pub exact_dataset_entry_one_to_one_join_confirmed: bool,
    pub purged_and_embargoed_rows_excluded_confirmed: bool,
    pub point_in_time_feature_availability_confirmed: bool,
    pub sealed_holdout_target_isolation_confirmed: bool,
    pub exact_raw_metric_bits_without_transform_confirmed: bool,
    pub continuous_target_vector_not_action_or_reward_confirmed: bool,
    pub explicit_missingness_without_imputation_confirmed: bool,
    pub registration_review_execution_separation_confirmed: bool,
    pub no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub future_independent_spec_review_eligible: bool,
    pub independent_spec_review_completed: bool,
    pub join_execution_authorized: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub semantic_target_assigned: bool,
    pub joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecEligibleSubject {
    pub transformation_attempt_id: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub materialization_id: String,
    pub materialization_claim_sha256: String,
    pub materialization_result_sha256: String,
    pub split_manifest_sha256: String,
    pub feature_bundle_sha256: String,
    pub combined_artifact_sha256: String,
    pub dataset_id: String,
    pub dataset_content_sha256: String,
    pub dataset_manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub feature_catalog_count: usize,
    pub feature_catalog_sha256: String,
    pub feature_schema_sha256: String,
    pub registered_specification: Option<HistoricalOutcomeFeatureLabelJoinTargetSpecRecord>,
    pub registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub subjects: Vec<HistoricalOutcomeFeatureLabelJoinTargetSpecEligibleSubject>,
    pub registration_eligible_count: usize,
    pub specification_count: usize,
    pub current_binding_specification_count: usize,
    pub stale_or_mismatched_specification_count: usize,
    pub independent_review_eligible_count: usize,
    pub registration_status: String,
    pub registration_enabled: bool,
    pub independent_review_enabled: bool,
    pub join_execution_enabled: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecReadinessSummary {
    pub registration_eligible_count: usize,
    pub specification_count: usize,
    pub current_binding_specification_count: usize,
    pub stale_or_mismatched_specification_count: usize,
    pub independent_review_eligible_count: usize,
    pub registration_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CurrentHistoricalOutcomeFeatureLabelJoinTargetSpecification {
    pub specification: HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
    pub validated_artifact_pair:
        IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_specs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_registry(&state).await {
        Ok(registry) => Json(registry).into_response(),
        Err(error) => {
            warn!(%error, "feature-label join/target spec registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "特征—标签连接与目标治理规范暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_feature_label_join_target_spec(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<RegisterHistoricalOutcomeFeatureLabelJoinTargetSpecRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(registry) => (StatusCode::CREATED, Json(registry)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_feature_label_join_target_spec_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetSpecReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetSpecReadinessSummary {
            registration_eligible_count: registry.registration_eligible_count,
            specification_count: registry.specification_count,
            current_binding_specification_count: registry.current_binding_specification_count,
            stale_or_mismatched_specification_count: registry
                .stale_or_mismatched_specification_count,
            independent_review_eligible_count: registry.independent_review_eligible_count,
            registration_status: registry.registration_status,
        },
    )
}

pub(crate) async fn current_historical_outcome_feature_label_join_target_specifications_for_independent_review(
    state: &AppState,
) -> Result<Vec<CurrentHistoricalOutcomeFeatureLabelJoinTargetSpecification>, String> {
    let pairs =
        independently_validated_official_artifact_pairs_for_join_target_specification(state)
            .await?;
    let specifications = read_specifications(state).await?;
    let specification_index = specifications
        .into_iter()
        .map(|record| (record.transformation_attempt_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut current = Vec::new();
    for pair in pairs {
        let attempt_id = pair.validation.transformation_attempt_id.clone();
        let Some(specification) = specification_index.get(&attempt_id).cloned() else {
            continue;
        };
        verify_record_binding(&specification, &pair)?;
        if !specification.future_independent_spec_review_eligible
            || specification.independent_spec_review_completed
        {
            continue;
        }
        current.push(
            CurrentHistoricalOutcomeFeatureLabelJoinTargetSpecification {
                specification,
                validated_artifact_pair: pair,
            },
        );
    }
    current.sort_by(|left, right| {
        right
            .specification
            .registered_at
            .cmp(&left.specification.registered_at)
            .then_with(|| {
                right
                    .specification
                    .specification_id
                    .cmp(&left.specification.specification_id)
            })
    });
    Ok(current)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry, String> {
    let validated_pairs =
        independently_validated_official_artifact_pairs_for_join_target_specification(state)
            .await?;
    let specifications = read_specifications(state).await?;
    let index = specifications
        .iter()
        .map(|record| (record.transformation_attempt_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let current_attempts = validated_pairs
        .iter()
        .map(|pair| pair.validation.transformation_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    let stale_or_mismatched_specification_count = specifications
        .iter()
        .filter(|record| !current_attempts.contains(record.transformation_attempt_id.as_str()))
        .count();
    let mut subjects = Vec::new();
    for pair in validated_pairs {
        let record = index
            .get(pair.validation.transformation_attempt_id.as_str())
            .cloned()
            .cloned();
        if let Some(record) = &record {
            verify_record_binding(record, &pair)?;
        }
        subjects.push(subject_from_pair(&pair, record));
    }
    let registration_eligible_count = subjects
        .iter()
        .filter(|subject| subject.registration_eligible)
        .count();
    let current_binding_specification_count = subjects
        .iter()
        .filter(|subject| subject.registered_specification.is_some())
        .count();
    let independent_review_eligible_count = subjects
        .iter()
        .filter(|subject| {
            subject
                .registered_specification
                .as_ref()
                .is_some_and(|record| {
                    record.future_independent_spec_review_eligible
                        && !record.independent_spec_review_completed
                })
        })
        .count();
    let registration_status = if stale_or_mismatched_specification_count > 0 {
        "join_target_specification_stale_or_mismatched_fail_closed"
    } else if independent_review_eligible_count > 0 {
        "join_target_specification_registered_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "waiting_join_target_specification_registration"
    } else {
        "waiting_independently_validated_official_artifact_pair"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetSpecRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        subjects,
        registration_eligible_count,
        specification_count: specifications.len(),
        current_binding_specification_count,
        stale_or_mismatched_specification_count,
        independent_review_eligible_count,
        registration_status: registration_status.to_string(),
        registration_enabled: registration_eligible_count > 0,
        independent_review_enabled: false,
        join_execution_enabled: false,
        semantic_target_assignment_enabled: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 36 阶段只登记精确数据条目连接、分区标签可见性、点时/缺失规则和连续前瞻结果目标向量。登记后仍须独立复核；当前不执行 join、不分配目标、不创建训练行，也不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

fn subject_from_pair(
    pair: &IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
    registered_specification: Option<HistoricalOutcomeFeatureLabelJoinTargetSpecRecord>,
) -> HistoricalOutcomeFeatureLabelJoinTargetSpecEligibleSubject {
    let artifact_pair = &pair.artifact_pair;
    HistoricalOutcomeFeatureLabelJoinTargetSpecEligibleSubject {
        transformation_attempt_id: pair.validation.transformation_attempt_id.clone(),
        validation_id: pair.validation.validation_id.clone(),
        validation_sha256: pair.validation.validation_sha256.clone(),
        materialization_id: pair.validation.materialization_id.clone(),
        materialization_claim_sha256: pair.validation.materialization_claim_sha256.clone(),
        materialization_result_sha256: pair.validation.materialization_result_sha256.clone(),
        split_manifest_sha256: pair.validation.split_manifest_sha256.clone(),
        feature_bundle_sha256: pair.validation.feature_bundle_sha256.clone(),
        combined_artifact_sha256: pair.validation.combined_artifact_sha256.clone(),
        dataset_id: pair.validation.dataset_id.clone(),
        dataset_content_sha256: pair.validation.dataset_content_sha256.clone(),
        dataset_manifest_sha256: pair.validation.dataset_manifest_sha256.clone(),
        candidate_set_sha256: pair.validation.candidate_set_sha256.clone(),
        feature_catalog_count: artifact_pair.feature_bundle.feature_catalog_count,
        feature_catalog_sha256: artifact_pair.feature_bundle.feature_catalog_sha256.clone(),
        feature_schema_sha256: artifact_pair.feature_bundle.feature_schema_sha256.clone(),
        registration_eligible: registered_specification.is_none(),
        registered_specification,
    }
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    attempt_id: &str,
    request: RegisterHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetSpecRecord, String> {
    validate_request(attempt_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_specifications(state)
        .await?
        .iter()
        .any(|record| record.transformation_attempt_id == attempt_id)
    {
        return Err("该正式工件对已经登记不可变 join/target 治理规范，不得重放".to_string());
    }
    let pair = independently_validated_official_artifact_pairs_for_join_target_specification(state)
        .await?
        .into_iter()
        .find(|pair| pair.validation.transformation_attempt_id == attempt_id)
        .ok_or_else(|| "当前没有这组已独立校验正式工件可供登记".to_string())?;
    verify_expected_binding(&pair, &request)?;
    let excluded_prior_actor_ids = prior_actor_ids(&pair);
    let registrar_independent = actor_is_independent(registrar_id, &excluded_prior_actor_ids);
    if !registrar_independent {
        return Err("join/target 规范登记者不能是正式工件校验人或完整上游参与者".to_string());
    }
    let join_specification = build_join_specification(&pair)?;
    let target_specification = build_target_specification()?;
    let specification_body_sha256 = hash_serializable(&(
        &join_specification,
        &target_specification,
        request.specification_name.trim(),
        request.code_revision.trim(),
        request.rationale.trim(),
        request.known_limitations.trim(),
    ))?;
    let validation = &pair.validation;
    let mut record = HistoricalOutcomeFeatureLabelJoinTargetSpecRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        specification_id: String::new(),
        specification_sha256: String::new(),
        transformation_attempt_id: attempt_id.to_string(),
        validation_id: validation.validation_id.clone(),
        validation_sha256: validation.validation_sha256.clone(),
        materialization_id: validation.materialization_id.clone(),
        materialization_claim_sha256: validation.materialization_claim_sha256.clone(),
        materialization_result_sha256: validation.materialization_result_sha256.clone(),
        split_manifest_sha256: validation.split_manifest_sha256.clone(),
        feature_bundle_sha256: validation.feature_bundle_sha256.clone(),
        combined_artifact_sha256: validation.combined_artifact_sha256.clone(),
        dataset_id: validation.dataset_id.clone(),
        dataset_content_sha256: validation.dataset_content_sha256.clone(),
        dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(),
        candidate_set_sha256: validation.candidate_set_sha256.clone(),
        registered_at: Utc::now(),
        registered_by: registrar_id.to_string(),
        excluded_prior_actor_ids,
        registrar_independent_from_complete_prior_chain: registrar_independent,
        specification_name: request.specification_name.trim().to_string(),
        code_revision: request.code_revision.trim().to_string(),
        rationale: request.rationale.trim().to_string(),
        known_limitations: request.known_limitations.trim().to_string(),
        join_specification,
        target_specification,
        specification_body_sha256,
        status: STATUS.to_string(),
        exact_validated_artifact_pair_binding_confirmed: request
            .exact_validated_artifact_pair_binding_confirmed,
        registrar_independence_confirmed: request.registrar_independence_confirmed,
        exact_dataset_entry_one_to_one_join_confirmed: request
            .exact_dataset_entry_one_to_one_join_confirmed,
        purged_and_embargoed_rows_excluded_confirmed: request
            .purged_and_embargoed_rows_excluded_confirmed,
        point_in_time_feature_availability_confirmed: request
            .point_in_time_feature_availability_confirmed,
        sealed_holdout_target_isolation_confirmed: request
            .sealed_holdout_target_isolation_confirmed,
        exact_raw_metric_bits_without_transform_confirmed: request
            .exact_raw_metric_bits_without_transform_confirmed,
        continuous_target_vector_not_action_or_reward_confirmed: request
            .continuous_target_vector_not_action_or_reward_confirmed,
        explicit_missingness_without_imputation_confirmed: request
            .explicit_missingness_without_imputation_confirmed,
        registration_review_execution_separation_confirmed: request
            .registration_review_execution_separation_confirmed,
        no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed: request
            .no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed,
        future_independent_spec_review_eligible: true,
        independent_spec_review_completed: false,
        join_execution_authorized: false,
        feature_join_performed: false,
        semantic_target_assignment_authorized: false,
        semantic_target_assigned: false,
        joined_dataset_created: false,
        copied_to_training_store: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.specification_sha256 = record_fingerprint(&record)?;
    record.specification_id = record.specification_sha256[..32].to_string();
    validate_record(&record)?;
    let path = specification_root(state)
        .join(attempt_id)
        .join(format!("{}.json", record.specification_id));
    write_immutable_json(&path, &record).await?;
    Ok(record)
}

fn build_join_specification(
    pair: &IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
) -> Result<HistoricalOutcomeFeatureLabelJoinSpecification, String> {
    let validation = &pair.validation;
    let bundle = &pair.artifact_pair.feature_bundle;
    let mut specification = HistoricalOutcomeFeatureLabelJoinSpecification {
        schema_version: JOIN_SPEC_SCHEMA_VERSION.to_string(),
        specification_sha256: String::new(),
        dataset_id: validation.dataset_id.clone(),
        dataset_content_sha256: validation.dataset_content_sha256.clone(),
        dataset_manifest_sha256: validation.dataset_manifest_sha256.clone(),
        candidate_set_sha256: validation.candidate_set_sha256.clone(),
        split_manifest_sha256: validation.split_manifest_sha256.clone(),
        feature_bundle_sha256: validation.feature_bundle_sha256.clone(),
        combined_artifact_sha256: validation.combined_artifact_sha256.clone(),
        dataset_entry_key: "dataset.entries.entry_id".to_string(),
        split_record_key: "official_split_manifest.records.dataset_entry_id".to_string(),
        feature_record_key_fields: vec![
            "dataset_entry_id".to_string(),
            "feature_namespace".to_string(),
            "feature_id".to_string(),
        ],
        raw_outcome_record_key: "dataset.entries.entry_id".to_string(),
        join_cardinality_rule: "exactly_one_split_record_and_one_raw_outcome_record_per_dataset_entry; exactly_one_record_per_allowlisted_feature_id; duplicates_or_missing_keys_fail_closed".to_string(),
        split_authority_rule: "official_split_manifest_is_only_split_authority; no_reassignment_or_cross_component_move".to_string(),
        purged_or_embargoed_row_policy: "exclude_from_all_joined_training_or_evaluation_rows_and_preserve_audit_record".to_string(),
        train_target_visibility_policy: "future_separately_authorized_training_runtime_may_read_train_targets_only_after_join_output_independent_validation".to_string(),
        validation_target_visibility_policy: "future_separately_authorized_evaluator_only; never_available_to_feature_transformation".to_string(),
        sealed_holdout_target_visibility_policy: "cryptographically_and_operationally_sealed_until_model_and_evaluation_protocol_are_frozen; never_available_to_training_or_tuning".to_string(),
        feature_availability_rule: "available_at_utc_must_be_lte_dataset_entry.decision_available_at_and_artifact_source_version_must_match_the_official_bundle".to_string(),
        explicit_missingness_rule: "preserve_every_allowlisted_feature_record; missing_value_remains_null_with_nonempty_reason; no_row_or_feature_drop_due_to_missingness".to_string(),
        feature_catalog_count: bundle.feature_catalog_count,
        feature_catalog_sha256: bundle.feature_catalog_sha256.clone(),
        feature_schema_sha256: bundle.feature_schema_sha256.clone(),
        allowed_label_horizons_market_sessions: REQUIRED_HORIZONS.to_vec(),
        forbidden_join_inputs: vec![
            "post_decision_source".to_string(),
            "future_market_data".to_string(),
            "outcome_or_label_as_feature".to_string(),
            "sealed_holdout_label_for_training_or_tuning".to_string(),
            "current_portfolio_state".to_string(),
            "model_generated_backfill".to_string(),
        ],
        joined_row_schema_fields: vec![
            "dataset_entry_id".to_string(),
            "dataset_entry_sha256".to_string(),
            "split".to_string(),
            "component_id".to_string(),
            "decision_available_at".to_string(),
            "feature_records".to_string(),
            "target_vector".to_string(),
            "source_binding_sha256".to_string(),
        ],
        one_to_one_outcome_join_required: true,
        all_allowlisted_feature_records_preserved: true,
        imputation_allowed: false,
        interpolation_allowed: false,
        sealed_holdout_labels_opened: false,
        join_executed: false,
    };
    specification.specification_sha256 = join_spec_fingerprint(&specification)?;
    validate_join_specification(&specification)?;
    Ok(specification)
}

fn build_target_specification() -> Result<HistoricalOutcomeSemanticTargetSpecification, String> {
    let definitions = expected_target_definitions();
    let target_vector_order = definitions
        .iter()
        .map(|definition| definition.target_id.clone())
        .collect::<Vec<_>>();
    let primary_supervised_target_id = "excess_return_250_continuous".to_string();
    let risk_target_id = "asset_max_drawdown_250_continuous".to_string();
    let auxiliary_target_ids = target_vector_order
        .iter()
        .filter(|target_id| {
            *target_id != &primary_supervised_target_id && *target_id != &risk_target_id
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut specification = HistoricalOutcomeSemanticTargetSpecification {
        schema_version: TARGET_SPEC_SCHEMA_VERSION.to_string(),
        specification_sha256: String::new(),
        prediction_task: "predict_continuous_forward_asset_path_and_benchmark_relative_outcomes_from_point_in_time_features; no_action_inference".to_string(),
        target_definitions: definitions,
        primary_supervised_target_id,
        risk_target_id,
        auxiliary_target_ids,
        benchmark_return_role: "audit_component_only_for_recomputing_excess_return; not_an_independent_supervised_target".to_string(),
        target_vector_order,
        duplicate_horizon_policy: "fail_closed".to_string(),
        missing_horizon_policy: "exclude_row_before_split_specific_target_access_and_preserve_failure_audit; no_partial_target_vector".to_string(),
        train_target_access_policy: "future_authorized_training_only_after_join_materialization_and_independent_output_validation".to_string(),
        validation_target_access_policy: "future_authorized_model_selection_evaluator_only_after_model_candidate_freeze".to_string(),
        sealed_holdout_target_access_policy: "future_one_shot_independent_evaluator_only_after_model_and_protocol_freeze; never_training_or_tuning".to_string(),
        exact_f64_bits_preserved: true,
        normalization_allowed: false,
        winsorization_allowed: false,
        rank_transform_allowed: false,
        categorical_action_label_defined: false,
        buy_hold_sell_threshold_defined: false,
        portfolio_weight_target_defined: false,
        scalar_reward_defined: false,
        semantic_target_assignment_performed: false,
    };
    specification.specification_sha256 = target_spec_fingerprint(&specification)?;
    validate_target_specification(&specification)?;
    Ok(specification)
}

fn expected_target_definitions() -> Vec<HistoricalOutcomeSemanticTargetDefinition> {
    let mut definitions = Vec::new();
    for horizon in REQUIRED_HORIZONS {
        definitions.push(target_definition(
            horizon,
            "asset_return",
            if horizon == 250 {
                "auxiliary"
            } else {
                "path_auxiliary"
            },
            "asset total return over the exact common-session horizon",
        ));
        definitions.push(target_definition(
            horizon,
            "excess_return",
            if horizon == 250 {
                "primary"
            } else {
                "path_auxiliary"
            },
            "asset return minus benchmark return over the exact common-session horizon",
        ));
        definitions.push(target_definition(
            horizon,
            "asset_max_drawdown",
            if horizon == 250 {
                "risk"
            } else {
                "path_risk_auxiliary"
            },
            "minimum peak-to-trough asset drawdown observed inside the exact horizon",
        ));
    }
    definitions
}

fn target_definition(
    horizon: u16,
    metric: &str,
    role: &str,
    semantics: &str,
) -> HistoricalOutcomeSemanticTargetDefinition {
    HistoricalOutcomeSemanticTargetDefinition {
        target_id: format!("{metric}_{horizon}_continuous"),
        horizon_market_sessions: horizon,
        source_metric_field: metric.to_string(),
        source_selector: format!(
            "raw_validated_metrics[horizon_market_sessions={horizon}].{metric}"
        ),
        value_kind: "f64_exact_bits".to_string(),
        unit: "decimal_return".to_string(),
        transformation: "identity_no_normalization_no_winsorization_no_ranking".to_string(),
        role: role.to_string(),
        semantics: semantics.to_string(),
    }
}

fn validate_request(
    attempt_id: &str,
    request: &RegisterHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
) -> Result<(), String> {
    let text_valid = bounded_nonempty(&request.specification_name, MAX_NAME_CHARS)
        && bounded_nonempty(&request.code_revision, MAX_REVISION_CHARS)
        && bounded_nonempty(&request.rationale, MAX_RATIONALE_CHARS)
        && bounded_nonempty(&request.known_limitations, MAX_LIMITATIONS_CHARS);
    if !valid_id(attempt_id)
        || !valid_id(&request.expected_validation_id)
        || !valid_id(&request.expected_materialization_id)
        || !valid_id(&request.expected_dataset_id)
        || !all_sha256(&[
            &request.expected_validation_sha256,
            &request.expected_materialization_claim_sha256,
            &request.expected_materialization_result_sha256,
            &request.expected_split_manifest_sha256,
            &request.expected_feature_bundle_sha256,
            &request.expected_combined_artifact_sha256,
            &request.expected_dataset_content_sha256,
            &request.expected_dataset_manifest_sha256,
            &request.expected_candidate_set_sha256,
        ])
        || !text_valid
    {
        return Err("join/target 规范请求中的 ID、SHA-256 或说明字段无效".to_string());
    }
    if !request.exact_validated_artifact_pair_binding_confirmed
        || !request.registrar_independence_confirmed
        || !request.exact_dataset_entry_one_to_one_join_confirmed
        || !request.purged_and_embargoed_rows_excluded_confirmed
        || !request.point_in_time_feature_availability_confirmed
        || !request.sealed_holdout_target_isolation_confirmed
        || !request.exact_raw_metric_bits_without_transform_confirmed
        || !request.continuous_target_vector_not_action_or_reward_confirmed
        || !request.explicit_missingness_without_imputation_confirmed
        || !request.registration_review_execution_separation_confirmed
        || !request
            .no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed
    {
        return Err("必须逐项确认连接、目标、防泄漏、独立性和零执行权限边界".to_string());
    }
    Ok(())
}

fn verify_expected_binding(
    pair: &IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
    request: &RegisterHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
) -> Result<(), String> {
    let validation = &pair.validation;
    if validation.validation_id != request.expected_validation_id
        || validation.validation_sha256 != request.expected_validation_sha256
        || validation.materialization_id != request.expected_materialization_id
        || validation.materialization_claim_sha256 != request.expected_materialization_claim_sha256
        || validation.materialization_result_sha256
            != request.expected_materialization_result_sha256
        || validation.split_manifest_sha256 != request.expected_split_manifest_sha256
        || validation.feature_bundle_sha256 != request.expected_feature_bundle_sha256
        || validation.combined_artifact_sha256 != request.expected_combined_artifact_sha256
        || validation.dataset_id != request.expected_dataset_id
        || validation.dataset_content_sha256 != request.expected_dataset_content_sha256
        || validation.dataset_manifest_sha256 != request.expected_dataset_manifest_sha256
        || validation.candidate_set_sha256 != request.expected_candidate_set_sha256
    {
        return Err("已校验正式工件或数据集绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn verify_record_binding(
    record: &HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
    pair: &IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
) -> Result<(), String> {
    validate_record(record)?;
    let validation = &pair.validation;
    if record.transformation_attempt_id != validation.transformation_attempt_id
        || record.validation_id != validation.validation_id
        || record.validation_sha256 != validation.validation_sha256
        || record.materialization_id != validation.materialization_id
        || record.materialization_claim_sha256 != validation.materialization_claim_sha256
        || record.materialization_result_sha256 != validation.materialization_result_sha256
        || record.split_manifest_sha256 != validation.split_manifest_sha256
        || record.feature_bundle_sha256 != validation.feature_bundle_sha256
        || record.combined_artifact_sha256 != validation.combined_artifact_sha256
        || record.dataset_id != validation.dataset_id
        || record.dataset_content_sha256 != validation.dataset_content_sha256
        || record.dataset_manifest_sha256 != validation.dataset_manifest_sha256
        || record.candidate_set_sha256 != validation.candidate_set_sha256
    {
        return Err("已登记 join/target 规范与当前独立校验正式工件不一致".to_string());
    }
    Ok(())
}

fn prior_actor_ids(
    pair: &IndependentlyValidatedHistoricalOutcomeOfflineDatasetOfficialArtifactPair,
) -> Vec<String> {
    let mut actors = pair.validation.excluded_prior_actor_ids.clone();
    actors.push(pair.validation.validated_by.clone());
    actors.push(pair.validation.materialized_by.clone());
    actors.retain(|actor| !actor.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn actor_is_independent(actor_id: &str, excluded_actor_ids: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded_actor_ids.iter().all(|actor| actor != actor_id)
}

fn validate_join_specification(
    specification: &HistoricalOutcomeFeatureLabelJoinSpecification,
) -> Result<(), String> {
    let exact_fields = specification.feature_record_key_fields
        == ["dataset_entry_id", "feature_namespace", "feature_id"];
    let horizons = specification
        .allowed_label_horizons_market_sessions
        .as_slice()
        == REQUIRED_HORIZONS;
    let static_contract_exact = specification.join_cardinality_rule
        == "exactly_one_split_record_and_one_raw_outcome_record_per_dataset_entry; exactly_one_record_per_allowlisted_feature_id; duplicates_or_missing_keys_fail_closed"
        && specification.split_authority_rule
            == "official_split_manifest_is_only_split_authority; no_reassignment_or_cross_component_move"
        && specification.purged_or_embargoed_row_policy
            == "exclude_from_all_joined_training_or_evaluation_rows_and_preserve_audit_record"
        && specification.train_target_visibility_policy
            == "future_separately_authorized_training_runtime_may_read_train_targets_only_after_join_output_independent_validation"
        && specification.validation_target_visibility_policy
            == "future_separately_authorized_evaluator_only; never_available_to_feature_transformation"
        && specification.sealed_holdout_target_visibility_policy
            == "cryptographically_and_operationally_sealed_until_model_and_evaluation_protocol_are_frozen; never_available_to_training_or_tuning"
        && specification.feature_availability_rule
            == "available_at_utc_must_be_lte_dataset_entry.decision_available_at_and_artifact_source_version_must_match_the_official_bundle"
        && specification.explicit_missingness_rule
            == "preserve_every_allowlisted_feature_record; missing_value_remains_null_with_nonempty_reason; no_row_or_feature_drop_due_to_missingness"
        && specification.forbidden_join_inputs
            == [
                "post_decision_source",
                "future_market_data",
                "outcome_or_label_as_feature",
                "sealed_holdout_label_for_training_or_tuning",
                "current_portfolio_state",
                "model_generated_backfill",
            ]
        && specification.joined_row_schema_fields
            == [
                "dataset_entry_id",
                "dataset_entry_sha256",
                "split",
                "component_id",
                "decision_available_at",
                "feature_records",
                "target_vector",
                "source_binding_sha256",
            ];
    if specification.schema_version != JOIN_SPEC_SCHEMA_VERSION
        || !valid_sha256(&specification.specification_sha256)
        || specification.specification_sha256 != join_spec_fingerprint(specification)?
        || !valid_id(&specification.dataset_id)
        || !all_sha256(&[
            &specification.dataset_content_sha256,
            &specification.dataset_manifest_sha256,
            &specification.candidate_set_sha256,
            &specification.split_manifest_sha256,
            &specification.feature_bundle_sha256,
            &specification.combined_artifact_sha256,
            &specification.feature_catalog_sha256,
            &specification.feature_schema_sha256,
        ])
        || specification.dataset_entry_key != "dataset.entries.entry_id"
        || specification.split_record_key != "official_split_manifest.records.dataset_entry_id"
        || specification.raw_outcome_record_key != "dataset.entries.entry_id"
        || !exact_fields
        || specification.feature_catalog_count != EXPECTED_FEATURE_COUNT
        || !horizons
        || !static_contract_exact
        || !specification.one_to_one_outcome_join_required
        || !specification.all_allowlisted_feature_records_preserved
        || specification.imputation_allowed
        || specification.interpolation_allowed
        || specification.sealed_holdout_labels_opened
        || specification.join_executed
    {
        return Err("feature-label join specification is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn validate_target_specification(
    specification: &HistoricalOutcomeSemanticTargetSpecification,
) -> Result<(), String> {
    let expected = build_expected_target_identity_set();
    let actual = specification
        .target_definitions
        .iter()
        .map(|definition| {
            (
                definition.target_id.clone(),
                definition.horizon_market_sessions,
                definition.source_metric_field.clone(),
            )
        })
        .collect::<BTreeSet<_>>();
    let order = specification
        .target_definitions
        .iter()
        .map(|definition| definition.target_id.clone())
        .collect::<Vec<_>>();
    let definitions_valid = specification.target_definitions == expected_target_definitions();
    let static_contract_exact = specification.prediction_task
        == "predict_continuous_forward_asset_path_and_benchmark_relative_outcomes_from_point_in_time_features; no_action_inference"
        && specification.benchmark_return_role
            == "audit_component_only_for_recomputing_excess_return; not_an_independent_supervised_target"
        && specification.duplicate_horizon_policy == "fail_closed"
        && specification.missing_horizon_policy
            == "exclude_row_before_split_specific_target_access_and_preserve_failure_audit; no_partial_target_vector"
        && specification.train_target_access_policy
            == "future_authorized_training_only_after_join_materialization_and_independent_output_validation"
        && specification.validation_target_access_policy
            == "future_authorized_model_selection_evaluator_only_after_model_candidate_freeze"
        && specification.sealed_holdout_target_access_policy
            == "future_one_shot_independent_evaluator_only_after_model_and_protocol_freeze; never_training_or_tuning";
    if specification.schema_version != TARGET_SPEC_SCHEMA_VERSION
        || !valid_sha256(&specification.specification_sha256)
        || specification.specification_sha256 != target_spec_fingerprint(specification)?
        || specification.target_definitions.len() != 9
        || actual != expected
        || order != specification.target_vector_order
        || !specification
            .target_vector_order
            .contains(&specification.primary_supervised_target_id)
        || specification.primary_supervised_target_id != "excess_return_250_continuous"
        || specification.risk_target_id != "asset_max_drawdown_250_continuous"
        || specification.auxiliary_target_ids.len() != 7
        || !definitions_valid
        || !static_contract_exact
        || !specification.exact_f64_bits_preserved
        || specification.normalization_allowed
        || specification.winsorization_allowed
        || specification.rank_transform_allowed
        || specification.categorical_action_label_defined
        || specification.buy_hold_sell_threshold_defined
        || specification.portfolio_weight_target_defined
        || specification.scalar_reward_defined
        || specification.semantic_target_assignment_performed
    {
        return Err("semantic target specification is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn build_expected_target_identity_set() -> BTreeSet<(String, u16, String)> {
    REQUIRED_HORIZONS
        .into_iter()
        .flat_map(|horizon| {
            ["asset_return", "excess_return", "asset_max_drawdown"]
                .into_iter()
                .map(move |metric| {
                    (
                        format!("{metric}_{horizon}_continuous"),
                        horizon,
                        metric.to_string(),
                    )
                })
        })
        .collect()
}

fn validate_record(
    record: &HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
) -> Result<(), String> {
    validate_join_specification(&record.join_specification)?;
    validate_target_specification(&record.target_specification)?;
    let confirmations = record.exact_validated_artifact_pair_binding_confirmed
        && record.registrar_independence_confirmed
        && record.exact_dataset_entry_one_to_one_join_confirmed
        && record.purged_and_embargoed_rows_excluded_confirmed
        && record.point_in_time_feature_availability_confirmed
        && record.sealed_holdout_target_isolation_confirmed
        && record.exact_raw_metric_bits_without_transform_confirmed
        && record.continuous_target_vector_not_action_or_reward_confirmed
        && record.explicit_missingness_without_imputation_confirmed
        && record.registration_review_execution_separation_confirmed
        && record
            .no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !record.independent_spec_review_completed
        && !record.join_execution_authorized
        && !record.feature_join_performed
        && !record.semantic_target_assignment_authorized
        && !record.semantic_target_assigned
        && !record.joined_dataset_created
        && !record.copied_to_training_store
        && !record.training_authorized
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let expected_body_sha256 = hash_serializable(&(
        &record.join_specification,
        &record.target_specification,
        record.specification_name.as_str(),
        record.code_revision.as_str(),
        record.rationale.as_str(),
        record.known_limitations.as_str(),
    ))?;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.specification_id)
        || !valid_sha256(&record.specification_sha256)
        || record.specification_id != record.specification_sha256[..32]
        || record.specification_sha256 != record_fingerprint(record)?
        || !valid_id(&record.transformation_attempt_id)
        || !valid_id(&record.validation_id)
        || !valid_id(&record.materialization_id)
        || !valid_id(&record.dataset_id)
        || !all_sha256(&[
            &record.validation_sha256,
            &record.materialization_claim_sha256,
            &record.materialization_result_sha256,
            &record.split_manifest_sha256,
            &record.feature_bundle_sha256,
            &record.combined_artifact_sha256,
            &record.dataset_content_sha256,
            &record.dataset_manifest_sha256,
            &record.candidate_set_sha256,
            &record.specification_body_sha256,
        ])
        || record.specification_body_sha256 != expected_body_sha256
        || !bounded_nonempty(&record.specification_name, MAX_NAME_CHARS)
        || !bounded_nonempty(&record.code_revision, MAX_REVISION_CHARS)
        || !bounded_nonempty(&record.rationale, MAX_RATIONALE_CHARS)
        || !bounded_nonempty(&record.known_limitations, MAX_LIMITATIONS_CHARS)
        || record.status != STATUS
        || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || !record.registrar_independent_from_complete_prior_chain
        || !actor_is_independent(&record.registered_by, &record.excluded_prior_actor_ids)
        || !confirmations
        || !record.future_independent_spec_review_eligible
        || !authority_closed
    {
        return Err("join/target specification record is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_specifications(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetSpecRecord>, String> {
    let root = specification_root(state);
    let mut records = Vec::new();
    let mut attempts = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(records),
        Err(error) => return Err(error.to_string()),
    };
    while let Some(attempt) = attempts
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if !attempt
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let attempt_id = attempt.file_name().to_string_lossy().to_string();
        if !valid_id(&attempt_id) {
            return Err("join/target 规范目录包含无效 attempt ID".to_string());
        }
        let mut files = tokio::fs::read_dir(attempt.path())
            .await
            .map_err(|error| error.to_string())?;
        let mut count = 0usize;
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
            {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let record =
                serde_json::from_slice::<HistoricalOutcomeFeatureLabelJoinTargetSpecRecord>(
                    &tokio::fs::read(&path)
                        .await
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| format!("join/target 规范无法解析：{error}"))?;
            validate_record(&record)?;
            if record.transformation_attempt_id != attempt_id
                || path.file_stem().and_then(|value| value.to_str())
                    != Some(record.specification_id.as_str())
            {
                return Err("join/target 规范目录或文件名与记录身份不一致".to_string());
            }
            count += 1;
            records.push(record);
        }
        if count > 1 {
            return Err("同一正式工件 attempt 存在多个 join/target 规范".to_string());
        }
    }
    records.sort_by(|left, right| {
        right
            .registered_at
            .cmp(&left.registered_at)
            .then_with(|| right.specification_id.cmp(&left.specification_id))
    });
    Ok(records)
}

fn join_spec_fingerprint(
    specification: &HistoricalOutcomeFeatureLabelJoinSpecification,
) -> Result<String, String> {
    fingerprint_without(specification, &["specification_sha256"])
}

fn target_spec_fingerprint(
    specification: &HistoricalOutcomeSemanticTargetSpecification,
) -> Result<String, String> {
    fingerprint_without(specification, &["specification_sha256"])
}

fn record_fingerprint(
    record: &HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["specification_id", "specification_sha256"])
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "join/target fingerprint input is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_nonempty(value: &str, max_chars: usize) -> bool {
    let value = value.trim();
    !value.is_empty() && value.chars().count() <= max_chars
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

fn all_sha256(values: &[&String]) -> bool {
    values.iter().all(|value| valid_sha256(value))
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn specification_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-feature-label-join-target-specs")
}

struct RegistrationLock {
    path: PathBuf,
}

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-outcome-feature-label-join-target-spec-{attempt_id}.lock"
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
        .map_err(|_| "已有同一 join/target 规范登记正在进行".to_string())?;
    Ok(RegistrationLock { path })
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

    #[test]
    fn target_vector_is_continuous_and_contains_no_action_or_reward() {
        let specification = build_target_specification().unwrap();
        assert_eq!(specification.target_definitions.len(), 9);
        assert_eq!(
            specification.primary_supervised_target_id,
            "excess_return_250_continuous"
        );
        assert!(!specification.categorical_action_label_defined);
        assert!(!specification.buy_hold_sell_threshold_defined);
        assert!(!specification.scalar_reward_defined);
        assert!(!specification.semantic_target_assignment_performed);
    }

    #[test]
    fn target_spec_rejects_duplicate_or_missing_metric_identity() {
        let mut specification = build_target_specification().unwrap();
        specification.target_definitions[0].target_id =
            specification.target_definitions[1].target_id.clone();
        specification.specification_sha256 = target_spec_fingerprint(&specification).unwrap();
        assert!(validate_target_specification(&specification).is_err());
    }

    #[test]
    fn target_hash_binds_primary_semantics() {
        let specification = build_target_specification().unwrap();
        let first = target_spec_fingerprint(&specification).unwrap();
        let mut changed = specification;
        changed.primary_supervised_target_id = "asset_return_250_continuous".to_string();
        assert_ne!(first, target_spec_fingerprint(&changed).unwrap());
    }

    #[test]
    fn rehashed_target_source_selector_tampering_is_rejected() {
        let mut specification = build_target_specification().unwrap();
        specification.target_definitions[0].source_selector =
            "raw_validated_metrics[horizon_market_sessions=250].asset_return".to_string();
        specification.specification_sha256 = target_spec_fingerprint(&specification).unwrap();
        assert!(validate_target_specification(&specification).is_err());
    }

    #[test]
    fn registrar_must_be_outside_complete_prior_chain() {
        let actors = vec!["materializer".to_string(), "validator".to_string()];
        assert!(actor_is_independent("registrar", &actors));
        assert!(!actor_is_independent("validator", &actors));
    }

    #[test]
    fn expected_target_identity_set_has_three_metrics_for_each_horizon() {
        let identities = build_expected_target_identity_set();
        assert_eq!(identities.len(), 9);
        for horizon in REQUIRED_HORIZONS {
            assert_eq!(
                identities
                    .iter()
                    .filter(|(_, current_horizon, _)| *current_horizon == horizon)
                    .count(),
                3
            );
        }
    }

    #[test]
    fn exact_float_and_no_transform_contract_is_fail_closed() {
        let mut specification = build_target_specification().unwrap();
        specification.normalization_allowed = true;
        specification.specification_sha256 = target_spec_fingerprint(&specification).unwrap();
        assert!(validate_target_specification(&specification).is_err());
    }
}
