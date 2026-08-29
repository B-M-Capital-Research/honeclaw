//! Independent governance review for one immutable historical-outcome dataset.
//!
//! This stage freezes the future split and point-in-time feature-join contracts.
//! It deliberately does not assign a split, join a feature, infer a target, run
//! training or grant reward, shadow, order, broker or trading authority.

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

use super::historical_outcome_offline_datasets::{
    HistoricalOutcomeOfflineDataset, historical_outcome_offline_dataset_registry_for_governance,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-governance-registry-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-governance-review-v1";
const POLICY_VERSION: &str =
    "hone-historical-outcome-offline-dataset-governance-v1-pit-component-isolated";
const SPLIT_POLICY_VERSION: &str =
    "hone-future-historical-outcome-split-policy-v1-company-event-source-components";
const FEATURE_JOIN_POLICY_VERSION: &str =
    "hone-future-point-in-time-feature-join-policy-v1-fail-closed";
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;
const TRAIN_PERCENT: u8 = 70;
const VALIDATION_PERCENT: u8 = 15;
const SEALED_HOLDOUT_PERCENT: u8 = 15;
const MAX_OUTCOME_HORIZON_MARKET_SESSIONS: u16 = 250;
const PURGE_EMBARGO_MARKET_SESSIONS: u16 = 250;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetGovernanceVerdict {
    ApprovedForSplitAndPointInTimeFeatureJoinSpecRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetSplitPolicy {
    pub policy_version: String,
    pub policy_sha256: String,
    pub connected_component_axes: Vec<String>,
    pub component_rule: String,
    pub deterministic_assignment_algorithm: String,
    pub train_percent: u8,
    pub validation_percent: u8,
    pub sealed_holdout_percent: u8,
    pub temporal_order_required: bool,
    pub max_outcome_horizon_market_sessions: u16,
    pub purge_embargo_market_sessions: u16,
    pub sealed_holdout_labels_withheld_from_training_worker: bool,
    pub assignments_created_by_this_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetFeatureJoinPolicy {
    pub policy_version: String,
    pub policy_sha256: String,
    pub availability_rule: String,
    pub required_feature_provenance_fields: Vec<String>,
    pub forbidden_feature_namespaces: Vec<String>,
    pub missing_or_ambiguous_availability_policy: String,
    pub backfill_or_interpolation_allowed: bool,
    pub immutable_feature_bundle_required: bool,
    pub independent_feature_bundle_review_required: bool,
    pub feature_join_performed_by_this_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetGovernanceSubject {
    pub dataset_id: String,
    pub dataset_version: String,
    pub version_number: usize,
    pub dataset_content_sha256: String,
    pub manifest_sha256: String,
    pub candidate_set_sha256: String,
    pub entry_count: usize,
    pub distinct_symbol_count: usize,
    pub earliest_decision_available_at: DateTime<Utc>,
    pub latest_decision_available_at: DateTime<Utc>,
    pub assembled_at: DateTime<Utc>,
    pub assembled_by: String,
    pub complete_actor_ids: Vec<String>,
    pub distinct_reconstruction_count: usize,
    pub distinct_snapshot_count: usize,
    pub raw_outcome_only: bool,
    pub split_assigned: bool,
    pub feature_join_performed: bool,
    pub semantic_targets_assigned: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_dataset_content_sha256: String,
    expected_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_split_policy_sha256: String,
    expected_feature_join_policy_sha256: String,
    verdict: HistoricalOutcomeOfflineDatasetGovernanceVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_dataset_binding_confirmed: bool,
    reviewer_independence_confirmed: bool,
    complete_candidate_and_lineage_confirmed: bool,
    company_event_source_component_isolation_confirmed: bool,
    deterministic_split_and_sealed_holdout_confirmed: bool,
    temporal_order_and_max_horizon_embargo_confirmed: bool,
    point_in_time_feature_availability_confirmed: bool,
    immutable_feature_provenance_confirmed: bool,
    outcome_and_label_feature_exclusion_confirmed: bool,
    missing_or_ambiguous_availability_fail_closed_confirmed: bool,
    no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetGovernanceReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub subject: HistoricalOutcomeOfflineDatasetGovernanceSubject,
    pub split_policy: HistoricalOutcomeOfflineDatasetSplitPolicy,
    pub feature_join_policy: HistoricalOutcomeOfflineDatasetFeatureJoinPolicy,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub reviewer_independent_from_complete_dataset_chain: bool,
    pub verdict: HistoricalOutcomeOfflineDatasetGovernanceVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_current_dataset_binding_confirmed: bool,
    pub reviewer_independence_confirmed: bool,
    pub complete_candidate_and_lineage_confirmed: bool,
    pub company_event_source_component_isolation_confirmed: bool,
    pub deterministic_split_and_sealed_holdout_confirmed: bool,
    pub temporal_order_and_max_horizon_embargo_confirmed: bool,
    pub point_in_time_feature_availability_confirmed: bool,
    pub immutable_feature_provenance_confirmed: bool,
    pub outcome_and_label_feature_exclusion_confirmed: bool,
    pub missing_or_ambiguous_availability_fail_closed_confirmed: bool,
    pub no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub future_transformation_spec_registration_eligible: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetGovernanceItem {
    pub subject: HistoricalOutcomeOfflineDatasetGovernanceSubject,
    pub complete_review_actor_ids: Vec<String>,
    pub current_binding: bool,
    pub latest_review: Option<HistoricalOutcomeOfflineDatasetGovernanceReview>,
    pub review_eligible: bool,
    pub future_transformation_spec_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetGovernanceRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub split_policy: HistoricalOutcomeOfflineDatasetSplitPolicy,
    pub feature_join_policy: HistoricalOutcomeOfflineDatasetFeatureJoinPolicy,
    pub items: Vec<HistoricalOutcomeOfflineDatasetGovernanceItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub governance_status: String,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetGovernanceReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub governance_status: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_governance(
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
            warn!(%error, "historical outcome offline dataset governance registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线数据集治理复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_offline_dataset_governance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(dataset_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &dataset_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome offline dataset governance reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线数据集治理复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_governance_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetGovernanceReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeOfflineDatasetGovernanceReadinessSummary {
        review_eligible_count: registry.review_eligible_count,
        reviewed_count: registry.reviewed_count,
        approved_count: registry.approved_count,
        current_binding_approved_count: registry.current_binding_approved_count,
        governance_status: registry.governance_status,
    })
}

pub(crate) async fn historical_outcome_offline_dataset_governance_registry_for_transformation_specs(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetGovernanceRegistry, String> {
    build_registry(state).await
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetGovernanceRegistry, String> {
    let source = historical_outcome_offline_dataset_registry_for_governance(state).await?;
    let split_policy = split_policy();
    let feature_join_policy = feature_join_policy();
    let mut items = Vec::with_capacity(source.datasets.len());
    for dataset in &source.datasets {
        let subject = subject_from_dataset(dataset);
        let reviews = read_reviews_for_dataset(state, &dataset.dataset_id).await?;
        let complete_review_actor_ids = reviews
            .iter()
            .map(|review| review.reviewer_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let latest_review = reviews.last().cloned();
        if latest_review
            .as_ref()
            .is_some_and(|review| review.subject != subject)
        {
            return Err(
                "historical outcome offline dataset governance subject binding changed".to_string(),
            );
        }
        let (current_binding, review_eligible, future_spec_eligible) = governance_item_status(
            &dataset.candidate_set_sha256,
            &source.current_candidate_set_sha256,
            latest_review.as_ref(),
        );
        items.push(HistoricalOutcomeOfflineDatasetGovernanceItem {
            subject,
            complete_review_actor_ids,
            current_binding,
            review_eligible,
            latest_review,
            future_transformation_spec_registration_eligible: future_spec_eligible,
        });
    }
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let approved_count = items
        .iter()
        .filter(|item| {
            item.latest_review
                .as_ref()
                .is_some_and(|review| review.future_transformation_spec_registration_eligible)
        })
        .count();
    let current_binding_approved_count = items
        .iter()
        .filter(|item| item.future_transformation_spec_registration_eligible)
        .count();
    let governance_status = if source.current_binding_dataset_count == 0 {
        "waiting_current_bound_offline_dataset"
    } else if current_binding_approved_count > 0 {
        "current_dataset_approved_only_for_future_transformation_spec_registration"
    } else if review_eligible_count > 0 {
        "current_dataset_ready_for_independent_governance_review"
    } else {
        "current_dataset_governance_review_blocked_fail_closed"
    };
    Ok(HistoricalOutcomeOfflineDatasetGovernanceRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        split_policy,
        feature_join_policy,
        items,
        review_eligible_count,
        reviewed_count,
        approved_count,
        current_binding_approved_count,
        governance_status: governance_status.to_string(),
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
        scope: "对当前版本化原始结果数据集实施独立、追加式治理复核，只冻结未来按公司/历史事件/来源连通分量隔离的确定性切分规范和严格点时特征连接规范。批准仅允许后续登记新的转换规范；本阶段不分配样本、不连接特征、不生成语义目标、不训练、不奖励、不建立影子组合、不生成订单，也不接入券商或交易。".to_string(),
    })
}

fn governance_item_status(
    dataset_candidate_set_sha256: &str,
    current_candidate_set_sha256: &str,
    latest_review: Option<&HistoricalOutcomeOfflineDatasetGovernanceReview>,
) -> (bool, bool, bool) {
    let current_binding = dataset_candidate_set_sha256 == current_candidate_set_sha256;
    let approved =
        latest_review.is_some_and(|review| review.future_transformation_spec_registration_eligible);
    (
        current_binding,
        current_binding && !approved,
        current_binding && approved,
    )
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    dataset_id: &str,
    request: ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest,
) -> Result<HistoricalOutcomeOfflineDatasetGovernanceReview, String> {
    if reviewer_id.trim().is_empty() {
        return Err("数据集治理复核人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let source = historical_outcome_offline_dataset_registry_for_governance(state).await?;
    let dataset = source
        .datasets
        .iter()
        .find(|dataset| dataset.dataset_id == dataset_id)
        .ok_or_else(|| "离线数据集不存在".to_string())?;
    if dataset.candidate_set_sha256 != source.current_candidate_set_sha256 {
        return Err("只能复核当前完整候选集合绑定的数据集，请刷新后重试".to_string());
    }
    let reviews = read_reviews_for_dataset(state, dataset_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|review| review.future_transformation_spec_registration_eligible) {
        return Err("当前数据集治理已经批准，不得重复或分叉复核".to_string());
    }
    let review = build_review(
        &subject_from_dataset(dataset),
        prior,
        reviewer_id,
        request,
        Utc::now(),
    )?;
    write_immutable_json(
        &review_root(state)
            .join(dataset_id)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn build_review(
    subject: &HistoricalOutcomeOfflineDatasetGovernanceSubject,
    prior: Option<&HistoricalOutcomeOfflineDatasetGovernanceReview>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeOfflineDatasetGovernanceReview, String> {
    let rationale = request.rationale.trim();
    let known_limitations = request.known_limitations.trim();
    if rationale.is_empty()
        || rationale.chars().count() > MAX_RATIONALE_CHARS
        || known_limitations.is_empty()
        || known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
    {
        return Err("复核理由或已知局限为空或过长".to_string());
    }
    if request.expected_review_id != prior.map(|review| review.review_id.clone())
        || request.expected_review_sha256 != prior.map(|review| review.review_sha256.clone())
        || request.expected_dataset_content_sha256 != subject.dataset_content_sha256
        || request.expected_manifest_sha256 != subject.manifest_sha256
        || request.expected_candidate_set_sha256 != subject.candidate_set_sha256
    {
        return Err("数据集或追加式复核链绑定已变化，请刷新后重试".to_string());
    }
    if prior.is_some_and(|review| review.subject != *subject) {
        return Err("上一条治理复核绑定的数据集对象与当前对象不一致".to_string());
    }
    let split_policy = split_policy();
    let feature_join_policy = feature_join_policy();
    if request.expected_split_policy_sha256 != split_policy.policy_sha256
        || request.expected_feature_join_policy_sha256 != feature_join_policy.policy_sha256
    {
        return Err("防泄漏切分或点时特征连接规范已变化，请刷新后重试".to_string());
    }
    let reviewer_independent = !subject
        .complete_actor_ids
        .iter()
        .any(|actor| actor == reviewer_id);
    if !reviewer_independent {
        return Err(
            "复核人参与过数据集装配、标签写入、校验或前序链路，必须更换独立复核人".to_string(),
        );
    }
    let approval = request.verdict
        == HistoricalOutcomeOfflineDatasetGovernanceVerdict::ApprovedForSplitAndPointInTimeFeatureJoinSpecRegistration;
    let all_confirmed = request.exact_current_dataset_binding_confirmed
        && request.reviewer_independence_confirmed
        && request.complete_candidate_and_lineage_confirmed
        && request.company_event_source_component_isolation_confirmed
        && request.deterministic_split_and_sealed_holdout_confirmed
        && request.temporal_order_and_max_horizon_embargo_confirmed
        && request.point_in_time_feature_availability_confirmed
        && request.immutable_feature_provenance_confirmed
        && request.outcome_and_label_feature_exclusion_confirmed
        && request.missing_or_ambiguous_availability_fail_closed_confirmed
        && request.no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed;
    if approval && (!all_confirmed || !subject_is_raw_and_untransformed(subject)) {
        return Err(
            "批准治理复核前必须逐项确认完整谱系、防泄漏切分、点时特征和零执行边界".to_string(),
        );
    }
    let mut review = HistoricalOutcomeOfflineDatasetGovernanceReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: prior.map(|review| review.review_id.clone()),
        previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
        subject: subject.clone(),
        split_policy,
        feature_join_policy,
        submitted_at,
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids: subject.complete_actor_ids.clone(),
        reviewer_independent_from_complete_dataset_chain: reviewer_independent,
        verdict: request.verdict,
        rationale: rationale.to_string(),
        known_limitations: known_limitations.to_string(),
        exact_current_dataset_binding_confirmed: request.exact_current_dataset_binding_confirmed,
        reviewer_independence_confirmed: request.reviewer_independence_confirmed,
        complete_candidate_and_lineage_confirmed: request.complete_candidate_and_lineage_confirmed,
        company_event_source_component_isolation_confirmed: request
            .company_event_source_component_isolation_confirmed,
        deterministic_split_and_sealed_holdout_confirmed: request
            .deterministic_split_and_sealed_holdout_confirmed,
        temporal_order_and_max_horizon_embargo_confirmed: request
            .temporal_order_and_max_horizon_embargo_confirmed,
        point_in_time_feature_availability_confirmed: request
            .point_in_time_feature_availability_confirmed,
        immutable_feature_provenance_confirmed: request.immutable_feature_provenance_confirmed,
        outcome_and_label_feature_exclusion_confirmed: request
            .outcome_and_label_feature_exclusion_confirmed,
        missing_or_ambiguous_availability_fail_closed_confirmed: request
            .missing_or_ambiguous_availability_fail_closed_confirmed,
        no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed: request
            .no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed,
        future_transformation_spec_registration_eligible: approval,
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
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review)?;
    Ok(review)
}

fn subject_from_dataset(
    dataset: &HistoricalOutcomeOfflineDataset,
) -> HistoricalOutcomeOfflineDatasetGovernanceSubject {
    let mut actors = BTreeSet::new();
    actors.insert(dataset.assembled_by.clone());
    for entry in &dataset.entries {
        actors.insert(entry.formal_label_written_by.clone());
        actors.insert(entry.independently_validated_by.clone());
        actors.extend(entry.excluded_prior_actor_ids.iter().cloned());
    }
    HistoricalOutcomeOfflineDatasetGovernanceSubject {
        dataset_id: dataset.dataset_id.clone(),
        dataset_version: dataset.dataset_version.clone(),
        version_number: dataset.version_number,
        dataset_content_sha256: dataset.dataset_content_sha256.clone(),
        manifest_sha256: dataset.manifest_sha256.clone(),
        candidate_set_sha256: dataset.candidate_set_sha256.clone(),
        entry_count: dataset.entry_count,
        distinct_symbol_count: dataset.distinct_symbol_count,
        earliest_decision_available_at: dataset.earliest_decision_available_at,
        latest_decision_available_at: dataset.latest_decision_available_at,
        assembled_at: dataset.assembled_at,
        assembled_by: dataset.assembled_by.clone(),
        complete_actor_ids: actors.into_iter().collect(),
        distinct_reconstruction_count: dataset
            .entries
            .iter()
            .map(|entry| entry.reconstruction_sha256.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        distinct_snapshot_count: dataset
            .entries
            .iter()
            .map(|entry| entry.snapshot_sha256.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        raw_outcome_only: dataset.entries.iter().all(|entry| entry.raw_outcome_only),
        split_assigned: dataset.entries.iter().any(|entry| entry.split_assigned),
        feature_join_performed: dataset.feature_join_performed,
        semantic_targets_assigned: dataset.semantic_targets_assigned,
    }
}

fn split_policy() -> HistoricalOutcomeOfflineDatasetSplitPolicy {
    let mut policy = HistoricalOutcomeOfflineDatasetSplitPolicy {
        policy_version: SPLIT_POLICY_VERSION.to_string(),
        policy_sha256: String::new(),
        connected_component_axes: vec![
            "asset_symbol".to_string(),
            "historical_event_identity:reconstruction_sha256+snapshot_sha256+decision_available_at".to_string(),
            "source_family_identity:source_output_sha256+source_validation_sha256+protocol_sha256".to_string(),
        ],
        component_rule: "transitive_connected_components_are_indivisible_and_may_exist_in_exactly_one_of_train_validation_sealed_holdout".to_string(),
        deterministic_assignment_algorithm:
            "stable_sha256_of_policy_version_and_complete_component_identity".to_string(),
        train_percent: TRAIN_PERCENT,
        validation_percent: VALIDATION_PERCENT,
        sealed_holdout_percent: SEALED_HOLDOUT_PERCENT,
        temporal_order_required: true,
        max_outcome_horizon_market_sessions: MAX_OUTCOME_HORIZON_MARKET_SESSIONS,
        purge_embargo_market_sessions: PURGE_EMBARGO_MARKET_SESSIONS,
        sealed_holdout_labels_withheld_from_training_worker: true,
        assignments_created_by_this_review: false,
    };
    policy.policy_sha256 = fingerprint_without(&policy, &["policy_sha256"])
        .expect("static split policy is serializable");
    policy
}

fn feature_join_policy() -> HistoricalOutcomeOfflineDatasetFeatureJoinPolicy {
    let mut policy = HistoricalOutcomeOfflineDatasetFeatureJoinPolicy {
        policy_version: FEATURE_JOIN_POLICY_VERSION.to_string(),
        policy_sha256: String::new(),
        availability_rule:
            "feature.available_at_utc <= historical_decision.decision_available_at_utc".to_string(),
        required_feature_provenance_fields: vec![
            "artifact_sha256".to_string(),
            "source_identity".to_string(),
            "source_version".to_string(),
            "available_at_utc".to_string(),
        ],
        forbidden_feature_namespaces: vec![
            "raw_validated_metrics".to_string(),
            "outcome".to_string(),
            "formal_label".to_string(),
            "validation".to_string(),
            "admission".to_string(),
            "offline_dataset".to_string(),
            "future_market_data".to_string(),
            "split_assignment".to_string(),
        ],
        missing_or_ambiguous_availability_policy: "fail_closed_exclude_feature".to_string(),
        backfill_or_interpolation_allowed: false,
        immutable_feature_bundle_required: true,
        independent_feature_bundle_review_required: true,
        feature_join_performed_by_this_review: false,
    };
    policy.policy_sha256 = fingerprint_without(&policy, &["policy_sha256"])
        .expect("static feature join policy is serializable");
    policy
}

fn subject_is_raw_and_untransformed(
    subject: &HistoricalOutcomeOfflineDatasetGovernanceSubject,
) -> bool {
    subject.entry_count > 0
        && subject.raw_outcome_only
        && !subject.split_assigned
        && !subject.feature_join_performed
        && !subject.semantic_targets_assigned
}

fn validate_review(review: &HistoricalOutcomeOfflineDatasetGovernanceReview) -> Result<(), String> {
    let approval = review.verdict
        == HistoricalOutcomeOfflineDatasetGovernanceVerdict::ApprovedForSplitAndPointInTimeFeatureJoinSpecRegistration;
    let all_confirmed = review.exact_current_dataset_binding_confirmed
        && review.reviewer_independence_confirmed
        && review.complete_candidate_and_lineage_confirmed
        && review.company_event_source_component_isolation_confirmed
        && review.deterministic_split_and_sealed_holdout_confirmed
        && review.temporal_order_and_max_horizon_embargo_confirmed
        && review.point_in_time_feature_availability_confirmed
        && review.immutable_feature_provenance_confirmed
        && review.outcome_and_label_feature_exclusion_confirmed
        && review.missing_or_ambiguous_availability_fail_closed_confirmed
        && review.no_split_join_target_training_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !review.split_assignment_authorized
        && !review.split_assignment_performed
        && !review.feature_join_authorized
        && !review.feature_join_performed
        && !review.semantic_target_assignment_authorized
        && !review.training_authorized
        && !review.training_run_allowed
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
        || review.split_policy != split_policy()
        || review.feature_join_policy != feature_join_policy()
        || review.excluded_prior_actor_ids != review.subject.complete_actor_ids
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || !review.reviewer_independent_from_complete_dataset_chain
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || review.known_limitations.trim().is_empty()
        || review.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || !subject_is_raw_and_untransformed(&review.subject)
        || review.future_transformation_spec_registration_eligible != (approval && all_confirmed)
        || !authority_closed
    {
        return Err(
            "historical outcome offline dataset governance review is invalid or over-authorized"
                .to_string(),
        );
    }
    if review.previous_review_id.is_some() != review.previous_review_sha256.is_some()
        || review
            .previous_review_id
            .as_deref()
            .is_some_and(|value| !valid_id(value))
        || review
            .previous_review_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256(value))
    {
        return Err(
            "historical outcome offline dataset governance predecessor is invalid".to_string(),
        );
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[HistoricalOutcomeOfflineDatasetGovernanceReview],
    dataset_id: &str,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetGovernanceReview>, String> {
    if reviews.is_empty() {
        return Ok(Vec::new());
    }
    let mut by_id = HashMap::new();
    let mut child_by_parent = HashMap::<String, String>::new();
    let mut root_ids = Vec::new();
    for review in reviews {
        validate_review(review)?;
        if review.subject.dataset_id != dataset_id
            || by_id
                .insert(review.review_id.clone(), review.clone())
                .is_some()
        {
            return Err(
                "historical outcome offline dataset governance review replay or subject mismatch"
                    .to_string(),
            );
        }
        match &review.previous_review_id {
            Some(parent) => {
                if child_by_parent
                    .insert(parent.clone(), review.review_id.clone())
                    .is_some()
                {
                    return Err(
                        "historical outcome offline dataset governance review chain forked"
                            .to_string(),
                    );
                }
            }
            None => root_ids.push(review.review_id.clone()),
        }
    }
    if root_ids.len() != 1 {
        return Err(
            "historical outcome offline dataset governance review chain has invalid root"
                .to_string(),
        );
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = root_ids.pop().expect("one root");
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err(
                "historical outcome offline dataset governance review chain cycles".to_string(),
            );
        }
        let review = by_id.get(&current).ok_or_else(|| {
            "historical outcome offline dataset governance review chain disconnected".to_string()
        })?;
        if let Some(previous) = ordered.last() {
            let previous: &HistoricalOutcomeOfflineDatasetGovernanceReview = previous;
            if review.previous_review_id.as_deref() != Some(previous.review_id.as_str())
                || review.previous_review_sha256.as_deref() != Some(previous.review_sha256.as_str())
                || review.submitted_at < previous.submitted_at
                || review.subject != previous.subject
            {
                return Err(
                    "historical outcome offline dataset governance review chain binding is invalid"
                        .to_string(),
                );
            }
        }
        ordered.push(review.clone());
        let Some(next) = child_by_parent.get(&current) else {
            break;
        };
        current = next.clone();
    }
    if ordered.len() != reviews.len() {
        return Err(
            "historical outcome offline dataset governance review chain disconnected".to_string(),
        );
    }
    Ok(ordered)
}

async fn read_reviews_for_dataset(
    state: &AppState,
    dataset_id: &str,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetGovernanceReview>, String> {
    let root = review_root(state).join(dataset_id);
    let mut directory = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
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
        let review: HistoricalOutcomeOfflineDatasetGovernanceReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("historical outcome offline dataset governance path mismatch".to_string());
        }
        reviews.push(review);
    }
    validate_review_chain(&reviews, dataset_id)
}

fn review_fingerprint(
    review: &HistoricalOutcomeOfflineDatasetGovernanceReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
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

fn review_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical_outcome_offline_dataset_governance/reviews")
}

struct GovernanceLock {
    path: PathBuf,
}

impl Drop for GovernanceLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<GovernanceLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("historical-outcome-offline-dataset-governance.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(GovernanceLock { path }),
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
                return Err("另一个管理员正在提交离线数据集治理复核".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("离线数据集治理复核锁获取失败".to_string())
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
    use super::*;

    fn subject() -> HistoricalOutcomeOfflineDatasetGovernanceSubject {
        HistoricalOutcomeOfflineDatasetGovernanceSubject {
            dataset_id: "a".repeat(32),
            dataset_version: "historical-raw-outcomes-v1-000001-aaaaaaaaaaaa".to_string(),
            version_number: 1,
            dataset_content_sha256: "a".repeat(64),
            manifest_sha256: "b".repeat(64),
            candidate_set_sha256: "c".repeat(64),
            entry_count: 3,
            distinct_symbol_count: 2,
            earliest_decision_available_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            latest_decision_available_at: DateTime::from_timestamp(1_710_000_000, 0).unwrap(),
            assembled_at: DateTime::from_timestamp(1_720_000_000, 0).unwrap(),
            assembled_by: "assembler".to_string(),
            complete_actor_ids: vec![
                "assembler".to_string(),
                "label-validator".to_string(),
                "writer".to_string(),
            ],
            distinct_reconstruction_count: 3,
            distinct_snapshot_count: 3,
            raw_outcome_only: true,
            split_assigned: false,
            feature_join_performed: false,
            semantic_targets_assigned: false,
        }
    }

    fn request(
        subject: &HistoricalOutcomeOfflineDatasetGovernanceSubject,
        prior: Option<&HistoricalOutcomeOfflineDatasetGovernanceReview>,
        verdict: HistoricalOutcomeOfflineDatasetGovernanceVerdict,
    ) -> ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest {
        ReviewHistoricalOutcomeOfflineDatasetGovernanceRequest {
            expected_review_id: prior.map(|review| review.review_id.clone()),
            expected_review_sha256: prior.map(|review| review.review_sha256.clone()),
            expected_dataset_content_sha256: subject.dataset_content_sha256.clone(),
            expected_manifest_sha256: subject.manifest_sha256.clone(),
            expected_candidate_set_sha256: subject.candidate_set_sha256.clone(),
            expected_split_policy_sha256: split_policy().policy_sha256,
            expected_feature_join_policy_sha256: feature_join_policy().policy_sha256,
            verdict,
            rationale: "完整复核了不可变谱系和防泄漏边界".to_string(),
            known_limitations: "当前样本量仍小；本复核不创建切分或特征".to_string(),
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
        }
    }

    #[test]
    fn policy_hashes_bind_component_split_embargo_and_point_in_time_rules() {
        let split = split_policy();
        let features = feature_join_policy();
        assert_eq!(
            split.train_percent + split.validation_percent + split.sealed_holdout_percent,
            100
        );
        assert_eq!(split.purge_embargo_market_sessions, 250);
        assert_eq!(split.max_outcome_horizon_market_sessions, 250);
        assert!(!split.assignments_created_by_this_review);
        assert!(features.availability_rule.contains("<="));
        assert!(!features.backfill_or_interpolation_allowed);
        assert!(!features.feature_join_performed_by_this_review);
    }

    #[test]
    fn approval_requires_every_confirmation_and_raw_untransformed_subject() {
        let subject = subject();
        let mut missing = request(
            &subject,
            None,
            HistoricalOutcomeOfflineDatasetGovernanceVerdict::ApprovedForSplitAndPointInTimeFeatureJoinSpecRegistration,
        );
        missing.point_in_time_feature_availability_confirmed = false;
        assert!(build_review(&subject, None, "independent", missing, Utc::now()).is_err());
        let approved = build_review(
            &subject,
            None,
            "independent",
            request(
                &subject,
                None,
                HistoricalOutcomeOfflineDatasetGovernanceVerdict::ApprovedForSplitAndPointInTimeFeatureJoinSpecRegistration,
            ),
            Utc::now(),
        )
        .unwrap();
        assert!(approved.future_transformation_spec_registration_eligible);
    }

    #[test]
    fn reviewer_must_be_independent_from_every_dataset_actor() {
        let subject = subject();
        assert!(
            build_review(
                &subject,
                None,
                "writer",
                request(
                    &subject,
                    None,
                    HistoricalOutcomeOfflineDatasetGovernanceVerdict::ChangesRequested
                ),
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn exact_dataset_policy_and_current_tip_binding_are_required() {
        let subject = subject();
        let mut stale = request(
            &subject,
            None,
            HistoricalOutcomeOfflineDatasetGovernanceVerdict::ChangesRequested,
        );
        stale.expected_split_policy_sha256 = "f".repeat(64);
        assert!(build_review(&subject, None, "independent", stale, Utc::now()).is_err());
    }

    #[test]
    fn append_only_review_chain_rejects_forks_tampering_and_subject_changes() {
        let subject = subject();
        let first = build_review(
            &subject,
            None,
            "independent-a",
            request(
                &subject,
                None,
                HistoricalOutcomeOfflineDatasetGovernanceVerdict::ChangesRequested,
            ),
            DateTime::from_timestamp(1_730_000_000, 0).unwrap(),
        )
        .unwrap();
        let second = build_review(
            &subject,
            Some(&first),
            "independent-b",
            request(
                &subject,
                Some(&first),
                HistoricalOutcomeOfflineDatasetGovernanceVerdict::Rejected,
            ),
            DateTime::from_timestamp(1_730_000_001, 0).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validate_review_chain(&[second.clone(), first.clone()], &subject.dataset_id)
                .unwrap()
                .len(),
            2
        );
        let mut fork = second;
        fork.review_id = "d".repeat(32);
        fork.review_sha256 = "d".repeat(64);
        assert!(validate_review_chain(&[first, fork], &subject.dataset_id).is_err());
    }

    #[test]
    fn governance_approval_grants_only_future_spec_registration_eligibility() {
        let subject = subject();
        let review = build_review(
            &subject,
            None,
            "independent",
            request(
                &subject,
                None,
                HistoricalOutcomeOfflineDatasetGovernanceVerdict::ApprovedForSplitAndPointInTimeFeatureJoinSpecRegistration,
            ),
            Utc::now(),
        )
        .unwrap();
        assert!(review.future_transformation_spec_registration_eligible);
        assert!(!review.split_assignment_authorized);
        assert!(!review.feature_join_authorized);
        assert!(!review.semantic_target_assignment_authorized);
        assert!(!review.training_authorized);
        assert!(!review.reward_authorized);
        assert!(!review.shadow_portfolio_authorized);
        assert!(!review.order_generation_authorized);
        assert!(!review.broker_access_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn a_new_candidate_set_makes_a_prior_dataset_approval_non_current() {
        let subject = subject();
        let review = build_review(
            &subject,
            None,
            "independent",
            request(
                &subject,
                None,
                HistoricalOutcomeOfflineDatasetGovernanceVerdict::ApprovedForSplitAndPointInTimeFeatureJoinSpecRegistration,
            ),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            governance_item_status(
                &subject.candidate_set_sha256,
                &subject.candidate_set_sha256,
                Some(&review),
            ),
            (true, false, true),
        );
        assert_eq!(
            governance_item_status(
                &subject.candidate_set_sha256,
                &"d".repeat(64),
                Some(&review),
            ),
            (false, false, false),
        );
    }
}
