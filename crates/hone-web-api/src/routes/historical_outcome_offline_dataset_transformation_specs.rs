//! Immutable registration of one future offline-dataset transformation specification.
//!
//! A registered specification freezes how a later, separately reviewed implementation
//! would construct a leakage-safe split manifest and point-in-time feature bundle. This
//! module does not assign a split, join a feature, create a target, run training or grant
//! reward, shadow, order, broker or trading authority.

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

use super::historical_outcome_offline_dataset_governance::{
    HistoricalOutcomeOfflineDatasetFeatureJoinPolicy,
    HistoricalOutcomeOfflineDatasetGovernanceItem, HistoricalOutcomeOfflineDatasetGovernanceReview,
    HistoricalOutcomeOfflineDatasetGovernanceSubject, HistoricalOutcomeOfflineDatasetSplitPolicy,
    historical_outcome_offline_dataset_governance_registry_for_transformation_specs,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-spec-registry-v1";
const SPEC_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-transformation-spec-v1";
const POLICY_VERSION: &str =
    "hone-offline-dataset-transformation-spec-registration-v1-no-execution";
const SPLIT_MANIFEST_SPEC_VERSION: &str =
    "hone-deterministic-connected-component-split-manifest-spec-v2-exact-boundaries";
const FEATURE_BUNDLE_SPEC_VERSION: &str =
    "hone-point-in-time-seven-layer-feature-bundle-spec-v2-feature-id-allowlist";
const STATUS: &str = "registered_not_independently_reviewed_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct DeterministicSplitManifestSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub governed_split_policy_version: String,
    pub governed_split_policy_sha256: String,
    pub component_identity_fields: Vec<String>,
    pub component_construction_algorithm: String,
    pub component_identity_algorithm: String,
    pub chronological_order_algorithm: String,
    pub boundary_assignment_algorithm: String,
    pub boundary_objective: String,
    pub minimum_partition_rule: String,
    pub market_session_calendar_rule: String,
    pub purge_embargo_algorithm: String,
    pub empty_partition_after_purge_policy: String,
    pub train_percent: u8,
    pub validation_percent: u8,
    pub sealed_holdout_percent: u8,
    pub purge_embargo_market_sessions: u16,
    pub max_outcome_horizon_market_sessions: u16,
    pub output_manifest_fields: Vec<String>,
    pub sealed_holdout_labels_withheld_from_training_worker: bool,
    pub content_addressed_output_required: bool,
    pub create_once_output_required: bool,
    pub split_assignments_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PointInTimeFeatureDefinition {
    pub namespace: String,
    pub feature_id: String,
    pub value_kind: String,
    pub source_authority_contract: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PointInTimeFeatureBundleSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub governed_feature_join_policy_version: String,
    pub governed_feature_join_policy_sha256: String,
    pub join_key: String,
    pub allowed_feature_namespaces: Vec<String>,
    pub allowed_features: Vec<PointInTimeFeatureDefinition>,
    pub feature_id_must_be_allowlisted: bool,
    pub namespace_cannot_override_feature_semantics: bool,
    pub required_feature_record_fields: Vec<String>,
    pub availability_rule: String,
    pub observation_time_rule: String,
    pub forbidden_feature_namespaces: Vec<String>,
    pub missingness_values: Vec<String>,
    pub missing_or_ambiguous_availability_policy: String,
    pub artifact_revision_policy: String,
    pub qualitative_feature_review_policy: String,
    pub market_snapshot_policy: String,
    pub portfolio_snapshot_policy: String,
    pub output_bundle_fields: Vec<String>,
    pub backfill_allowed: bool,
    pub interpolation_allowed: bool,
    pub content_addressed_output_required: bool,
    pub create_once_output_required: bool,
    pub feature_bundle_generated: bool,
    pub feature_join_performed: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest {
    expected_dataset_content_sha256: String,
    expected_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_governance_review_id: String,
    expected_governance_review_sha256: String,
    expected_split_policy_sha256: String,
    expected_feature_join_policy_sha256: String,
    specification_name: String,
    code_revision: String,
    rationale: String,
    known_limitations: String,
    exact_dataset_and_governance_binding_confirmed: bool,
    registrar_independence_confirmed: bool,
    transitive_component_isolation_confirmed: bool,
    chronological_boundaries_and_hash_tie_break_confirmed: bool,
    purge_embargo_and_sealed_holdout_confirmed: bool,
    point_in_time_availability_and_provenance_confirmed: bool,
    seven_layer_namespace_allowlist_confirmed: bool,
    label_outcome_and_future_information_exclusion_confirmed: bool,
    missingness_fail_closed_without_imputation_confirmed: bool,
    registration_review_execution_separation_confirmed: bool,
    no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub transformation_spec_id: String,
    pub transformation_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub subject: HistoricalOutcomeOfflineDatasetGovernanceSubject,
    pub governance_review_id: String,
    pub governance_review_sha256: String,
    pub governance_reviewer_id: String,
    pub governance_known_limitations: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_dataset_and_governance_chain: bool,
    pub specification_name: String,
    pub code_revision: String,
    pub rationale: String,
    pub known_limitations: String,
    pub split_manifest_specification: DeterministicSplitManifestSpecification,
    pub feature_bundle_specification: PointInTimeFeatureBundleSpecification,
    pub transformation_body_sha256: String,
    pub status: String,
    pub exact_dataset_and_governance_binding_confirmed: bool,
    pub registrar_independence_confirmed: bool,
    pub transitive_component_isolation_confirmed: bool,
    pub chronological_boundaries_and_hash_tie_break_confirmed: bool,
    pub purge_embargo_and_sealed_holdout_confirmed: bool,
    pub point_in_time_availability_and_provenance_confirmed: bool,
    pub seven_layer_namespace_allowlist_confirmed: bool,
    pub label_outcome_and_future_information_exclusion_confirmed: bool,
    pub missingness_fail_closed_without_imputation_confirmed: bool,
    pub registration_review_execution_separation_confirmed: bool,
    pub no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub future_independent_spec_review_eligible: bool,
    pub independent_spec_review_completed: bool,
    pub split_assignment_authorized: bool,
    pub split_assignment_performed: bool,
    pub feature_join_authorized: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecEligibleSubject {
    pub subject: HistoricalOutcomeOfflineDatasetGovernanceSubject,
    pub governance_review_id: String,
    pub governance_review_sha256: String,
    pub governance_reviewer_id: String,
    pub split_policy_sha256: String,
    pub feature_join_policy_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecItem {
    pub specification: HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
    pub upstream_binding_current: bool,
    pub future_independent_spec_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub split_manifest_specification: DeterministicSplitManifestSpecification,
    pub feature_bundle_specification: PointInTimeFeatureBundleSpecification,
    pub eligible_subjects: Vec<HistoricalOutcomeOfflineDatasetTransformationSpecEligibleSubject>,
    pub items: Vec<HistoricalOutcomeOfflineDatasetTransformationSpecItem>,
    pub registration_eligible_count: usize,
    pub registered_count: usize,
    pub current_binding_registered_count: usize,
    pub independent_review_eligible_count: usize,
    pub transformation_spec_status: String,
    pub split_assignment_authorized: bool,
    pub split_assignment_performed: bool,
    pub feature_join_authorized: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecReadinessSummary {
    pub registration_eligible_count: usize,
    pub registered_count: usize,
    pub current_binding_registered_count: usize,
    pub independent_review_eligible_count: usize,
    pub transformation_spec_status: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_specs(
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
            warn!(%error, "historical outcome offline dataset transformation spec registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线数据集转换规范注册表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_offline_dataset_transformation_spec(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(dataset_id): AxumPath<String>,
    Json(request): Json<RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &dataset_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome offline dataset transformation spec registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线数据集转换规范注册表暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_transformation_spec_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeOfflineDatasetTransformationSpecReadinessSummary {
            registration_eligible_count: registry.registration_eligible_count,
            registered_count: registry.registered_count,
            current_binding_registered_count: registry.current_binding_registered_count,
            independent_review_eligible_count: registry.independent_review_eligible_count,
            transformation_spec_status: registry.transformation_spec_status,
        },
    )
}

pub(crate) async fn historical_outcome_offline_dataset_transformation_spec_registry_for_reviews(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecRegistry, String> {
    build_registry(state).await
}

pub(crate) fn validate_historical_outcome_offline_dataset_transformation_spec_for_review(
    record: &HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
) -> Result<(), String> {
    validate_record(record)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecRegistry, String> {
    let governance =
        historical_outcome_offline_dataset_governance_registry_for_transformation_specs(state)
            .await?;
    let approved = governance
        .items
        .iter()
        .filter_map(approved_current_item)
        .collect::<Vec<_>>();
    let approved_index = approved
        .iter()
        .map(|(item, review)| (review.review_id.as_str(), (*item, *review)))
        .collect::<HashMap<_, _>>();
    let existing = read_specs(state).await?;
    let mut items = existing
        .into_iter()
        .map(|specification| {
            let upstream_binding_current = approved_index
                .get(specification.governance_review_id.as_str())
                .is_some_and(|(item, review)| binding_matches(&specification, item, review));
            HistoricalOutcomeOfflineDatasetTransformationSpecItem {
                future_independent_spec_review_eligible: upstream_binding_current,
                upstream_binding_current,
                specification,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .specification
            .registered_at
            .cmp(&left.specification.registered_at)
            .then_with(|| {
                right
                    .specification
                    .transformation_spec_id
                    .cmp(&left.specification.transformation_spec_id)
            })
    });
    let registered_governance_reviews = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .map(|item| item.specification.governance_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let eligible_subjects = approved
        .iter()
        .filter(|(_, review)| !registered_governance_reviews.contains(review.review_id.as_str()))
        .map(|(item, review)| eligible_subject(item, review))
        .collect::<Vec<_>>();
    let registration_eligible_count = eligible_subjects.len();
    let registered_count = items.len();
    let current_binding_registered_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let independent_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_spec_review_eligible)
        .count();
    let transformation_spec_status = if approved.is_empty() {
        "waiting_current_governance_approval"
    } else if current_binding_registered_count == 0 {
        "ready_for_immutable_transformation_spec_registration"
    } else {
        STATUS
    };
    Ok(HistoricalOutcomeOfflineDatasetTransformationSpecRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        split_manifest_specification: split_manifest_specification(
            &governance.split_policy,
        ),
        feature_bundle_specification: feature_bundle_specification(
            &governance.feature_join_policy,
        ),
        eligible_subjects,
        items,
        registration_eligible_count,
        registered_count,
        current_binding_registered_count,
        independent_review_eligible_count,
        transformation_spec_status: transformation_spec_status.to_string(),
        split_assignment_authorized: false,
        split_assignment_performed: false,
        feature_join_authorized: false,
        feature_join_performed: false,
        semantic_target_assignment_authorized: false,
        training_authorized: false,
        training_run_allowed: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 25 阶段只登记与当前治理批准精确绑定、内容寻址且不可覆盖的未来转换规范：一份确定性防泄漏切分 manifest 合同和一份七层点时特征 bundle 合同。登记、独立复核与执行严格分离；本阶段不产生切分、不连接特征、不生成目标、不训练、不奖励、不建立影子组合、不生成订单，也不接入券商或交易。".to_string(),
    })
}

fn approved_current_item(
    item: &HistoricalOutcomeOfflineDatasetGovernanceItem,
) -> Option<(
    &HistoricalOutcomeOfflineDatasetGovernanceItem,
    &HistoricalOutcomeOfflineDatasetGovernanceReview,
)> {
    let review = item.latest_review.as_ref()?;
    (item.current_binding
        && item.future_transformation_spec_registration_eligible
        && review.future_transformation_spec_registration_eligible)
        .then_some((item, review))
}

fn eligible_subject(
    item: &HistoricalOutcomeOfflineDatasetGovernanceItem,
    review: &HistoricalOutcomeOfflineDatasetGovernanceReview,
) -> HistoricalOutcomeOfflineDatasetTransformationSpecEligibleSubject {
    HistoricalOutcomeOfflineDatasetTransformationSpecEligibleSubject {
        subject: item.subject.clone(),
        governance_review_id: review.review_id.clone(),
        governance_review_sha256: review.review_sha256.clone(),
        governance_reviewer_id: review.reviewer_id.clone(),
        split_policy_sha256: review.split_policy.policy_sha256.clone(),
        feature_join_policy_sha256: review.feature_join_policy.policy_sha256.clone(),
    }
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    dataset_id: &str,
    request: RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecRecord, String> {
    if registrar_id.trim().is_empty() {
        return Err("转换规范登记人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let governance =
        historical_outcome_offline_dataset_governance_registry_for_transformation_specs(state)
            .await?;
    let (item, review) = governance
        .items
        .iter()
        .filter_map(approved_current_item)
        .find(|(item, _)| item.subject.dataset_id == dataset_id)
        .ok_or_else(|| "当前数据集尚无有效治理批准，不能登记转换规范".to_string())?;
    if read_specs(state)
        .await?
        .iter()
        .any(|record| record.governance_review_id == review.review_id)
    {
        return Err("当前治理批准已经登记过转换规范，不得重复或覆盖".to_string());
    }
    let record = build_record(item, review, registrar_id, request, Utc::now())?;
    write_immutable_json(
        &spec_root(state).join(format!("{}.json", record.transformation_spec_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn build_record(
    item: &HistoricalOutcomeOfflineDatasetGovernanceItem,
    review: &HistoricalOutcomeOfflineDatasetGovernanceReview,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecRecord, String> {
    if !item.current_binding
        || !item.future_transformation_spec_registration_eligible
        || !review.future_transformation_spec_registration_eligible
        || request.expected_dataset_content_sha256 != item.subject.dataset_content_sha256
        || request.expected_manifest_sha256 != item.subject.manifest_sha256
        || request.expected_candidate_set_sha256 != item.subject.candidate_set_sha256
        || request.expected_governance_review_id != review.review_id
        || request.expected_governance_review_sha256 != review.review_sha256
        || request.expected_split_policy_sha256 != review.split_policy.policy_sha256
        || request.expected_feature_join_policy_sha256 != review.feature_join_policy.policy_sha256
    {
        return Err("数据集、治理复核或策略绑定已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(item);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == registrar_id)
    {
        return Err("登记人参与过数据集或治理复核链，必须更换独立登记人".to_string());
    }
    let specification_name =
        bounded_required(&request.specification_name, MAX_NAME_CHARS, "转换规范名称")?;
    let code_revision = bounded_required(&request.code_revision, MAX_REVISION_CHARS, "代码版本")?;
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "登记理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let all_confirmed = request.exact_dataset_and_governance_binding_confirmed
        && request.registrar_independence_confirmed
        && request.transitive_component_isolation_confirmed
        && request.chronological_boundaries_and_hash_tie_break_confirmed
        && request.purge_embargo_and_sealed_holdout_confirmed
        && request.point_in_time_availability_and_provenance_confirmed
        && request.seven_layer_namespace_allowlist_confirmed
        && request.label_outcome_and_future_information_exclusion_confirmed
        && request.missingness_fail_closed_without_imputation_confirmed
        && request.registration_review_execution_separation_confirmed
        && request.no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed;
    if !all_confirmed {
        return Err("登记转换规范前必须逐项确认防泄漏、点时来源和零执行边界".to_string());
    }
    let split_manifest_specification = split_manifest_specification(&review.split_policy);
    let feature_bundle_specification = feature_bundle_specification(&review.feature_join_policy);
    let transformation_body_sha256 = hash_serializable(&serde_json::json!({
        "split_manifest_specification": &split_manifest_specification,
        "feature_bundle_specification": &feature_bundle_specification,
    }))?;
    let mut record = HistoricalOutcomeOfflineDatasetTransformationSpecRecord {
        schema_version: SPEC_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        transformation_spec_id: String::new(),
        transformation_spec_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        subject: item.subject.clone(),
        governance_review_id: review.review_id.clone(),
        governance_review_sha256: review.review_sha256.clone(),
        governance_reviewer_id: review.reviewer_id.clone(),
        governance_known_limitations: review.known_limitations.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_dataset_and_governance_chain: true,
        specification_name,
        code_revision,
        rationale,
        known_limitations,
        split_manifest_specification,
        feature_bundle_specification,
        transformation_body_sha256,
        status: STATUS.to_string(),
        exact_dataset_and_governance_binding_confirmed: request
            .exact_dataset_and_governance_binding_confirmed,
        registrar_independence_confirmed: request.registrar_independence_confirmed,
        transitive_component_isolation_confirmed: request.transitive_component_isolation_confirmed,
        chronological_boundaries_and_hash_tie_break_confirmed: request
            .chronological_boundaries_and_hash_tie_break_confirmed,
        purge_embargo_and_sealed_holdout_confirmed: request
            .purge_embargo_and_sealed_holdout_confirmed,
        point_in_time_availability_and_provenance_confirmed: request
            .point_in_time_availability_and_provenance_confirmed,
        seven_layer_namespace_allowlist_confirmed: request
            .seven_layer_namespace_allowlist_confirmed,
        label_outcome_and_future_information_exclusion_confirmed: request
            .label_outcome_and_future_information_exclusion_confirmed,
        missingness_fail_closed_without_imputation_confirmed: request
            .missingness_fail_closed_without_imputation_confirmed,
        registration_review_execution_separation_confirmed: request
            .registration_review_execution_separation_confirmed,
        no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: request
            .no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed,
        future_independent_spec_review_eligible: true,
        independent_spec_review_completed: false,
        split_assignment_authorized: false,
        split_assignment_performed: false,
        feature_join_authorized: false,
        feature_join_performed: false,
        semantic_target_assignment_authorized: false,
        training_authorized: false,
        training_run_allowed: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.transformation_spec_sha256 = record_fingerprint(&record)?;
    record.transformation_spec_id = record.transformation_spec_sha256[..32].to_string();
    validate_record(&record)?;
    Ok(record)
}

fn split_manifest_specification(
    policy: &HistoricalOutcomeOfflineDatasetSplitPolicy,
) -> DeterministicSplitManifestSpecification {
    let mut specification = DeterministicSplitManifestSpecification {
        schema_version: SPLIT_MANIFEST_SPEC_VERSION.to_string(),
        specification_sha256: String::new(),
        governed_split_policy_version: policy.policy_version.clone(),
        governed_split_policy_sha256: policy.policy_sha256.clone(),
        component_identity_fields: vec![
            "asset_symbol".to_string(),
            "reconstruction_sha256".to_string(),
            "snapshot_sha256".to_string(),
            "decision_available_at".to_string(),
            "source_output_sha256".to_string(),
            "source_validation_sha256".to_string(),
            "protocol_sha256".to_string(),
        ],
        component_construction_algorithm:
            "deterministic_union_find_over_shared_company_historical_event_or_source_family_identity"
                .to_string(),
        component_identity_algorithm:
            "sha256_of_policy_version_plus_canonical_sorted_complete_component_entries"
                .to_string(),
        chronological_order_algorithm:
            "ascending_component_latest_decision_available_at_then_earliest_then_component_sha256"
                .to_string(),
        boundary_assignment_algorithm:
            "enumerate_all_chronological_contiguous_component_boundary_pairs_and_choose_the_lexicographically_minimal_objective"
                .to_string(),
        boundary_objective:
            "lexicographically_minimize_sum_absolute_integer_deviation_then_max_absolute_integer_deviation_then_earlier_train_boundary_index_then_earlier_validation_boundary_index_where_each_deviation_is_abs(100*partition_entry_count-target_percent*total_entry_count)"
                .to_string(),
        minimum_partition_rule:
            "require_at_least_three_connected_components_and_at_least_one_component_in_each_partition_before_and_after_purge_embargo_otherwise_fail_closed_without_manifest"
                .to_string(),
        market_session_calendar_rule:
            "use_the_frozen_common_asset_and_spy_market_session_index_bound_to_each_dataset_entry_no_wall_clock_day_approximation"
                .to_string(),
        purge_embargo_algorithm:
            "at_each_transition_purge_every_previous_partition_entry_whose_frozen_250_session_outcome_window_reaches_or_crosses_the_first_decision_session_of_the_next_partition_and_embargo_every_next_partition_entry_within_250_common_market_sessions_after_that_transition"
                .to_string(),
        empty_partition_after_purge_policy:
            "fail_closed_without_manifest_and_record_the_boundary_candidate_audit".to_string(),
        train_percent: policy.train_percent,
        validation_percent: policy.validation_percent,
        sealed_holdout_percent: policy.sealed_holdout_percent,
        purge_embargo_market_sessions: policy.purge_embargo_market_sessions,
        max_outcome_horizon_market_sessions: policy.max_outcome_horizon_market_sessions,
        output_manifest_fields: vec![
            "dataset_id".to_string(),
            "dataset_content_sha256".to_string(),
            "governance_review_sha256".to_string(),
            "transformation_spec_sha256".to_string(),
            "component_id".to_string(),
            "entry_id".to_string(),
            "split".to_string(),
            "component_order_index".to_string(),
            "boundary_objective_tuple".to_string(),
            "purged_or_embargoed".to_string(),
            "purge_reason".to_string(),
            "common_market_session_index".to_string(),
            "decision_available_at".to_string(),
            "boundary_candidate_audit_sha256".to_string(),
        ],
        sealed_holdout_labels_withheld_from_training_worker: policy
            .sealed_holdout_labels_withheld_from_training_worker,
        content_addressed_output_required: true,
        create_once_output_required: true,
        split_assignments_generated: false,
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])
            .expect("static split manifest specification is serializable");
    specification
}

fn feature_bundle_specification(
    policy: &HistoricalOutcomeOfflineDatasetFeatureJoinPolicy,
) -> PointInTimeFeatureBundleSpecification {
    let mut specification = PointInTimeFeatureBundleSpecification {
        schema_version: FEATURE_BUNDLE_SPEC_VERSION.to_string(),
        specification_sha256: String::new(),
        governed_feature_join_policy_version: policy.policy_version.clone(),
        governed_feature_join_policy_sha256: policy.policy_sha256.clone(),
        join_key: "dataset_entry_id+historical_decision_available_at".to_string(),
        allowed_feature_namespaces: vec![
            "industry_state".to_string(),
            "company_fundamentals".to_string(),
            "financial_state".to_string(),
            "valuation_state".to_string(),
            "crowding_state".to_string(),
            "macro_state".to_string(),
            "portfolio_context".to_string(),
        ],
        allowed_features: point_in_time_feature_catalog(),
        feature_id_must_be_allowlisted: true,
        namespace_cannot_override_feature_semantics: true,
        required_feature_record_fields: vec![
            "dataset_entry_id".to_string(),
            "feature_id".to_string(),
            "feature_namespace".to_string(),
            "value_encoding".to_string(),
            "value".to_string(),
            "value_kind".to_string(),
            "unit".to_string(),
            "period_start_utc".to_string(),
            "period_end_utc".to_string(),
            "accounting_basis".to_string(),
            "measurement_scope".to_string(),
            "is_missing".to_string(),
            "missingness_reason".to_string(),
            "artifact_sha256".to_string(),
            "source_identity".to_string(),
            "source_version".to_string(),
            "observed_at_utc".to_string(),
            "available_at_utc".to_string(),
            "transformation_id".to_string(),
            "transformation_sha256".to_string(),
        ],
        availability_rule: policy.availability_rule.clone(),
        observation_time_rule: "observed_at_utc_may_not_imply_availability_before_available_at_utc"
            .to_string(),
        forbidden_feature_namespaces: policy.forbidden_feature_namespaces.clone(),
        missingness_values: vec![
            "not_available_at_decision_time".to_string(),
            "source_missing".to_string(),
            "availability_ambiguous".to_string(),
            "not_applicable".to_string(),
        ],
        missing_or_ambiguous_availability_policy: policy
            .missing_or_ambiguous_availability_policy
            .clone(),
        artifact_revision_policy:
            "use_only_the_exact_artifact_version_available_at_the_historical_decision_time_later_restated_or_revised_versions_are_forbidden"
                .to_string(),
        qualitative_feature_review_policy:
            "qualitative_features_require_an_immutable_human_reviewed_evidence_chain_available_at_the_historical_decision_time"
                .to_string(),
        market_snapshot_policy:
            "market_features_require_a_frozen_content_addressed_point_in_time_market_snapshot_with_an_explicit_observation_timestamp"
                .to_string(),
        portfolio_snapshot_policy:
            "portfolio_context_requires_an_actor_scoped_immutable_historical_portfolio_snapshot_available_at_the_decision_time_and_must_not_use_current_holdings"
                .to_string(),
        output_bundle_fields: vec![
            "dataset_id".to_string(),
            "dataset_content_sha256".to_string(),
            "split_manifest_sha256".to_string(),
            "transformation_spec_sha256".to_string(),
            "feature_catalog_sha256".to_string(),
            "feature_schema_sha256".to_string(),
            "feature_records".to_string(),
            "excluded_feature_audit".to_string(),
            "bundle_sha256".to_string(),
        ],
        backfill_allowed: false,
        interpolation_allowed: false,
        content_addressed_output_required: true,
        create_once_output_required: true,
        feature_bundle_generated: false,
        feature_join_performed: false,
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])
            .expect("static point-in-time feature specification is serializable");
    specification
}

fn point_in_time_feature_catalog() -> Vec<PointInTimeFeatureDefinition> {
    let mut definitions = Vec::new();
    let mut extend = |namespace: &str, value_kind: &str, authority: &str, feature_ids: &[&str]| {
        definitions.extend(
            feature_ids
                .iter()
                .map(|feature_id| PointInTimeFeatureDefinition {
                    namespace: namespace.to_string(),
                    feature_id: (*feature_id).to_string(),
                    value_kind: value_kind.to_string(),
                    source_authority_contract: authority.to_string(),
                }),
        );
    };
    extend(
        "industry_state",
        "reviewed_state_or_verified_numeric",
        "point_in_time_primary_industry_evidence_or_human_reviewed_causal_state",
        &[
            "demand_driver_state",
            "effective_supply_state",
            "supply_demand_gap_state",
            "bottleneck_state",
            "pricing_power_state",
            "capacity_state",
            "technology_milestone_state",
            "industry_cycle_state",
        ],
    );
    extend(
        "company_fundamentals",
        "human_reviewed_ordinal_state",
        "immutable_human_reviewed_company_thesis_with_traceable_evidence",
        &[
            "business_model_quality",
            "moat_strength",
            "scarcity_strength",
            "differentiation_strength",
            "value_capture_strength",
            "market_share_state",
            "product_competitiveness",
            "customer_switching_cost",
            "customer_concentration_risk",
        ],
    );
    extend(
        "financial_state",
        "verified_numeric_or_explicit_missingness",
        "point_in_time_primary_filing_or_issuer_source_with_accounting_and_period_provenance",
        &[
            "revenue_growth_yoy",
            "gross_margin",
            "gross_margin_yoy_delta",
            "operating_margin",
            "free_cash_flow",
            "free_cash_flow_margin",
            "net_cash",
            "net_debt",
            "accounts_receivable_growth",
            "inventory_growth",
            "accounts_payable_growth",
            "capital_expenditure",
            "backlog",
            "remaining_performance_obligation",
            "annual_recurring_revenue",
        ],
    );
    extend(
        "valuation_state",
        "verified_numeric_or_reviewed_scenario",
        "frozen_market_snapshot_plus_independently_reviewed_point_in_time_valuation_input_chain",
        &[
            "current_price",
            "market_cap",
            "enterprise_value",
            "forward_pe",
            "ev_ebitda",
            "free_cash_flow_yield",
            "reverse_dcf_implied_growth",
            "bear_fair_value",
            "base_fair_value",
            "bull_fair_value",
            "current_price_to_base_value",
        ],
    );
    extend(
        "crowding_state",
        "verified_numeric_or_reviewed_state",
        "frozen_point_in_time_market_or_positioning_snapshot_with_no_future_price_data",
        &[
            "price_drawdown_from_high",
            "relative_strength",
            "valuation_percentile",
            "analyst_revision_state",
            "short_interest_state",
            "option_positioning_state",
            "narrative_crowding_state",
        ],
    );
    extend(
        "macro_state",
        "verified_numeric_or_reviewed_state",
        "point_in_time_official_macro_release_or_frozen_market_snapshot",
        &[
            "ten_year_treasury_yield",
            "thirty_year_treasury_yield",
            "policy_rate",
            "unemployment_rate",
            "nonfarm_payroll_state",
            "core_pce_state",
            "vix",
            "liquidity_state",
        ],
    );
    extend(
        "portfolio_context",
        "actor_scoped_historical_value_or_state",
        "immutable_actor_scoped_historical_portfolio_snapshot_never_current_holdings",
        &[
            "position_weight",
            "theme_weight",
            "cash_weight",
            "cost_basis_distance",
            "thesis_status",
            "risk_budget_remaining",
            "holding_horizon",
        ],
    );
    definitions
}

fn binding_matches(
    record: &HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
    item: &HistoricalOutcomeOfflineDatasetGovernanceItem,
    review: &HistoricalOutcomeOfflineDatasetGovernanceReview,
) -> bool {
    item.current_binding
        && item.future_transformation_spec_registration_eligible
        && review.future_transformation_spec_registration_eligible
        && record.subject == item.subject
        && record.governance_review_id == review.review_id
        && record.governance_review_sha256 == review.review_sha256
        && record.excluded_prior_actor_ids == expected_excluded_prior_actor_ids(item)
        && record.split_manifest_specification == split_manifest_specification(&review.split_policy)
        && record.feature_bundle_specification
            == feature_bundle_specification(&review.feature_join_policy)
}

fn expected_excluded_prior_actor_ids(
    item: &HistoricalOutcomeOfflineDatasetGovernanceItem,
) -> Vec<String> {
    let mut actor_ids = item.subject.complete_actor_ids.clone();
    actor_ids.extend(item.complete_review_actor_ids.iter().cloned());
    actor_ids.sort();
    actor_ids.dedup();
    actor_ids
}

fn validate_record(
    record: &HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
) -> Result<(), String> {
    let confirmations_complete = record.exact_dataset_and_governance_binding_confirmed
        && record.registrar_independence_confirmed
        && record.transitive_component_isolation_confirmed
        && record.chronological_boundaries_and_hash_tie_break_confirmed
        && record.purge_embargo_and_sealed_holdout_confirmed
        && record.point_in_time_availability_and_provenance_confirmed
        && record.seven_layer_namespace_allowlist_confirmed
        && record.label_outcome_and_future_information_exclusion_confirmed
        && record.missingness_fail_closed_without_imputation_confirmed
        && record.registration_review_execution_separation_confirmed
        && record.no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !record.independent_spec_review_completed
        && !record.split_assignment_authorized
        && !record.split_assignment_performed
        && !record.feature_join_authorized
        && !record.feature_join_performed
        && !record.semantic_target_assignment_authorized
        && !record.training_authorized
        && !record.training_run_allowed
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let expected_body_sha256 = hash_serializable(&serde_json::json!({
        "split_manifest_specification": &record.split_manifest_specification,
        "feature_bundle_specification": &record.feature_bundle_specification,
    }))?;
    if record.schema_version != SPEC_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.transformation_spec_id)
        || !valid_sha256(&record.transformation_spec_sha256)
        || record.transformation_spec_id != record.transformation_spec_sha256[..32]
        || record.transformation_spec_sha256 != record_fingerprint(record)?
        || record.registered_by.trim().is_empty()
        || !valid_id(&record.subject.dataset_id)
        || !valid_sha256(&record.subject.dataset_content_sha256)
        || !valid_sha256(&record.subject.manifest_sha256)
        || !valid_sha256(&record.subject.candidate_set_sha256)
        || !valid_id(&record.governance_review_id)
        || !valid_sha256(&record.governance_review_sha256)
        || record.governance_reviewer_id.trim().is_empty()
        || record.governance_known_limitations.trim().is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.registered_by)
        || !record.registrar_independent_from_dataset_and_governance_chain
        || record.specification_name.trim().is_empty()
        || record.specification_name.chars().count() > MAX_NAME_CHARS
        || record.code_revision.trim().is_empty()
        || record.code_revision.chars().count() > MAX_REVISION_CHARS
        || record.rationale.trim().is_empty()
        || record.rationale.chars().count() > MAX_RATIONALE_CHARS
        || record.known_limitations.trim().is_empty()
        || record.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || record.transformation_body_sha256 != expected_body_sha256
        || record.status != STATUS
        || !confirmations_complete
        || !record.future_independent_spec_review_eligible
        || !authority_closed
        || record
            .split_manifest_specification
            .split_assignments_generated
        || record.feature_bundle_specification.feature_bundle_generated
        || record.feature_bundle_specification.feature_join_performed
    {
        return Err(
            "historical outcome offline dataset transformation spec is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

async fn read_specs(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationSpecRecord>, String> {
    let root = spec_root(state);
    let mut directory = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    let mut governance_reviews = BTreeSet::new();
    while let Some(entry) = directory
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
        let record: HistoricalOutcomeOfflineDatasetTransformationSpecRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&record)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(record.transformation_spec_id.as_str())
            || !governance_reviews.insert(record.governance_review_id.clone())
        {
            return Err(
                "historical outcome offline dataset transformation spec replay or path mismatch"
                    .to_string(),
            );
        }
        records.push(record);
    }
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
) -> Result<String, String> {
    fingerprint_without(
        record,
        &["transformation_spec_id", "transformation_spec_sha256"],
    )
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
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max_chars: usize, field: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars {
        return Err(format!("{field}为空或过长"));
    }
    Ok(value.to_string())
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

fn spec_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical_outcome_offline_dataset_transformation_specs/records")
}

struct RegistrationLock {
    path: PathBuf,
}

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("historical-outcome-offline-dataset-transformation-spec.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(RegistrationLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                    .is_some_and(|age| age > StdDuration::from_secs(600));
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                if attempt < 7 {
                    tokio::time::sleep(StdDuration::from_millis(25)).await;
                    continue;
                }
                return Err("另一个管理员正在登记离线数据集转换规范".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("离线数据集转换规范登记锁获取失败".to_string())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::super::historical_outcome_offline_dataset_governance::HistoricalOutcomeOfflineDatasetGovernanceVerdict;
    use super::*;

    fn subject() -> HistoricalOutcomeOfflineDatasetGovernanceSubject {
        HistoricalOutcomeOfflineDatasetGovernanceSubject {
            dataset_id: "a".repeat(32),
            dataset_version: "historical-raw-outcomes-v1-000001-aaaaaaaaaaaa".to_string(),
            version_number: 1,
            dataset_content_sha256: "a".repeat(64),
            manifest_sha256: "b".repeat(64),
            candidate_set_sha256: "c".repeat(64),
            entry_count: 4,
            distinct_symbol_count: 2,
            earliest_decision_available_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            latest_decision_available_at: DateTime::from_timestamp(1_710_000_000, 0).unwrap(),
            assembled_at: DateTime::from_timestamp(1_720_000_000, 0).unwrap(),
            assembled_by: "assembler".to_string(),
            complete_actor_ids: vec!["assembler".to_string(), "validator".to_string()],
            distinct_reconstruction_count: 4,
            distinct_snapshot_count: 4,
            raw_outcome_only: true,
            split_assigned: false,
            feature_join_performed: false,
            semantic_targets_assigned: false,
        }
    }

    fn review(
        subject: &HistoricalOutcomeOfflineDatasetGovernanceSubject,
    ) -> HistoricalOutcomeOfflineDatasetGovernanceReview {
        let split_policy = HistoricalOutcomeOfflineDatasetSplitPolicy {
            policy_version: "split-v1".to_string(),
            policy_sha256: "d".repeat(64),
            connected_component_axes: vec!["asset_symbol".to_string()],
            component_rule: "indivisible".to_string(),
            deterministic_assignment_algorithm: "stable_sha256".to_string(),
            train_percent: 70,
            validation_percent: 15,
            sealed_holdout_percent: 15,
            temporal_order_required: true,
            max_outcome_horizon_market_sessions: 250,
            purge_embargo_market_sessions: 250,
            sealed_holdout_labels_withheld_from_training_worker: true,
            assignments_created_by_this_review: false,
        };
        let feature_join_policy = HistoricalOutcomeOfflineDatasetFeatureJoinPolicy {
            policy_version: "feature-v1".to_string(),
            policy_sha256: "e".repeat(64),
            availability_rule:
                "feature.available_at_utc <= historical_decision.decision_available_at_utc"
                    .to_string(),
            required_feature_provenance_fields: vec!["artifact_sha256".to_string()],
            forbidden_feature_namespaces: vec!["outcome".to_string(), "formal_label".to_string()],
            missing_or_ambiguous_availability_policy: "fail_closed_exclude_feature".to_string(),
            backfill_or_interpolation_allowed: false,
            immutable_feature_bundle_required: true,
            independent_feature_bundle_review_required: true,
            feature_join_performed_by_this_review: false,
        };
        HistoricalOutcomeOfflineDatasetGovernanceReview {
            schema_version: "review-v1".to_string(),
            policy_version: "policy-v1".to_string(),
            review_id: "f".repeat(32),
            review_sha256: "f".repeat(64),
            previous_review_id: None,
            previous_review_sha256: None,
            subject: subject.clone(),
            split_policy,
            feature_join_policy,
            submitted_at: DateTime::from_timestamp(1_730_000_000, 0).unwrap(),
            reviewer_id: "governance-reviewer".to_string(),
            excluded_prior_actor_ids: subject.complete_actor_ids.clone(),
            reviewer_independent_from_complete_dataset_chain: true,
            verdict: HistoricalOutcomeOfflineDatasetGovernanceVerdict::ApprovedForSplitAndPointInTimeFeatureJoinSpecRegistration,
            rationale: "approved".to_string(),
            known_limitations: "small sample".to_string(),
            exact_current_dataset_binding_confirmed: true,
            reviewer_independence_confirmed: true,
            complete_candidate_and_lineage_confirmed: true,
            company_event_source_component_isolation_confirmed: true,
            deterministic_split_and_sealed_holdout_confirmed: true,
            temporal_order_and_max_horizon_embargo_confirmed: true,
            point_in_time_feature_availability_confirmed: true,
            immutable_feature_provenance_confirmed: true,
            outcome_and_label_feature_exclusion_confirmed: true,
            missing_or_ambiguous_availability_fail_closed_confirmed: true,
            no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: true,
            future_transformation_spec_registration_eligible: true,
            split_assignment_authorized: false,
            split_assignment_performed: false,
            feature_join_authorized: false,
            feature_join_performed: false,
            semantic_target_assignment_authorized: false,
            training_authorized: false,
            training_run_allowed: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        }
    }

    fn item(
        subject: &HistoricalOutcomeOfflineDatasetGovernanceSubject,
        review: HistoricalOutcomeOfflineDatasetGovernanceReview,
    ) -> HistoricalOutcomeOfflineDatasetGovernanceItem {
        HistoricalOutcomeOfflineDatasetGovernanceItem {
            subject: subject.clone(),
            complete_review_actor_ids: vec!["governance-reviewer".to_string()],
            current_binding: true,
            latest_review: Some(review),
            review_eligible: false,
            future_transformation_spec_registration_eligible: true,
        }
    }

    fn request(
        subject: &HistoricalOutcomeOfflineDatasetGovernanceSubject,
        review: &HistoricalOutcomeOfflineDatasetGovernanceReview,
    ) -> RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest {
        RegisterHistoricalOutcomeOfflineDatasetTransformationSpecRequest {
            expected_dataset_content_sha256: subject.dataset_content_sha256.clone(),
            expected_manifest_sha256: subject.manifest_sha256.clone(),
            expected_candidate_set_sha256: subject.candidate_set_sha256.clone(),
            expected_governance_review_id: review.review_id.clone(),
            expected_governance_review_sha256: review.review_sha256.clone(),
            expected_split_policy_sha256: review.split_policy.policy_sha256.clone(),
            expected_feature_join_policy_sha256: review.feature_join_policy.policy_sha256.clone(),
            specification_name: "deterministic PIT transformation".to_string(),
            code_revision: "git:abc123".to_string(),
            rationale: "freeze a reviewable transformation contract".to_string(),
            known_limitations: "small sample and feature source coverage remain incomplete"
                .to_string(),
            exact_dataset_and_governance_binding_confirmed: true,
            registrar_independence_confirmed: true,
            transitive_component_isolation_confirmed: true,
            chronological_boundaries_and_hash_tie_break_confirmed: true,
            purge_embargo_and_sealed_holdout_confirmed: true,
            point_in_time_availability_and_provenance_confirmed: true,
            seven_layer_namespace_allowlist_confirmed: true,
            label_outcome_and_future_information_exclusion_confirmed: true,
            missingness_fail_closed_without_imputation_confirmed: true,
            registration_review_execution_separation_confirmed: true,
            no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    #[test]
    fn static_specs_freeze_chronology_components_sealed_holdout_and_seven_layers() {
        let subject = subject();
        let review = review(&subject);
        let split = split_manifest_specification(&review.split_policy);
        let features = feature_bundle_specification(&review.feature_join_policy);
        assert_eq!(
            split.train_percent + split.validation_percent + split.sealed_holdout_percent,
            100
        );
        assert_eq!(split.purge_embargo_market_sessions, 250);
        assert!(split.chronological_order_algorithm.contains("ascending"));
        assert!(split.boundary_objective.contains("lexicographically"));
        assert!(
            split
                .minimum_partition_rule
                .contains("require_at_least_three")
        );
        assert!(
            split
                .market_session_calendar_rule
                .contains("frozen_common_asset_and_spy_market_session_index")
        );
        assert!(
            split
                .empty_partition_after_purge_policy
                .contains("fail_closed_without_manifest")
        );
        assert!(!split.split_assignments_generated);
        assert_eq!(features.allowed_feature_namespaces.len(), 7);
        assert_eq!(features.allowed_features.len(), 65);
        assert!(features.feature_id_must_be_allowlisted);
        assert!(
            features
                .portfolio_snapshot_policy
                .contains("must_not_use_current_holdings")
        );
        assert!(!features.backfill_allowed);
        assert!(!features.feature_bundle_generated);
        assert!(!features.feature_join_performed);
    }

    #[test]
    fn registration_requires_exact_upstream_binding_and_every_confirmation() {
        let subject = subject();
        let review = review(&subject);
        let item = item(&subject, review.clone());
        let mut stale = request(&subject, &review);
        stale.expected_manifest_sha256 = "0".repeat(64);
        assert!(build_record(&item, &review, "registrar", stale, Utc::now()).is_err());
        let mut missing = request(&subject, &review);
        missing.point_in_time_availability_and_provenance_confirmed = false;
        assert!(build_record(&item, &review, "registrar", missing, Utc::now()).is_err());
    }

    #[test]
    fn registrar_must_be_independent_from_dataset_and_governance_reviewer() {
        let subject = subject();
        let review = review(&subject);
        let item = item(&subject, review.clone());
        assert!(
            build_record(
                &item,
                &review,
                "assembler",
                request(&subject, &review),
                Utc::now()
            )
            .is_err()
        );
        assert!(
            build_record(
                &item,
                &review,
                "governance-reviewer",
                request(&subject, &review),
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn valid_registration_is_content_addressed_and_only_review_eligible() {
        let subject = subject();
        let review = review(&subject);
        let item = item(&subject, review.clone());
        let record = build_record(
            &item,
            &review,
            "registrar",
            request(&subject, &review),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            record.transformation_spec_id,
            record.transformation_spec_sha256[..32]
        );
        assert!(record.future_independent_spec_review_eligible);
        assert!(!record.independent_spec_review_completed);
        assert!(!record.split_assignment_authorized);
        assert!(!record.feature_join_authorized);
        assert!(!record.training_authorized);
        assert!(!record.reward_authorized);
        assert!(!record.shadow_portfolio_authorized);
        assert!(!record.order_generation_authorized);
        assert!(!record.broker_access_authorized);
        assert!(!record.trading_authorized);
    }

    #[test]
    fn tampering_with_registered_contract_fails_validation() {
        let subject = subject();
        let review = review(&subject);
        let item = item(&subject, review.clone());
        let mut record = build_record(
            &item,
            &review,
            "registrar",
            request(&subject, &review),
            Utc::now(),
        )
        .unwrap();
        record.feature_bundle_specification.backfill_allowed = true;
        assert!(validate_record(&record).is_err());
    }

    #[test]
    fn upstream_drift_revokes_current_review_eligibility() {
        let subject = subject();
        let review = review(&subject);
        let item = item(&subject, review.clone());
        let record = build_record(
            &item,
            &review,
            "registrar",
            request(&subject, &review),
            Utc::now(),
        )
        .unwrap();
        assert!(binding_matches(&record, &item, &review));
        let mut stale_item = item.clone();
        stale_item.current_binding = false;
        assert!(!binding_matches(&record, &stale_item, &review));

        let mut actor_drift = item;
        actor_drift
            .complete_review_actor_ids
            .push("earlier-governance-reviewer".to_string());
        assert!(!binding_matches(&record, &actor_drift, &review));
    }

    #[test]
    fn transformation_body_hash_binds_both_specs() {
        let subject = subject();
        let review = review(&subject);
        let item = item(&subject, review.clone());
        let record = build_record(
            &item,
            &review,
            "registrar",
            request(&subject, &review),
            Utc::now(),
        )
        .unwrap();
        let expected = hash_serializable(&serde_json::json!({
            "split_manifest_specification": &record.split_manifest_specification,
            "feature_bundle_specification": &record.feature_bundle_specification,
        }))
        .unwrap();
        assert_eq!(record.transformation_body_sha256, expected);
    }
}
