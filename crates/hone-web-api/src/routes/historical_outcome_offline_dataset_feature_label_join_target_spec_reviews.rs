//! Independent, append-only semantic review of a registered feature-label join
//! and continuous-target governance specification.
//!
//! Approval permits only a future isolated implementation-registration step.
//! It never executes the join, opens labels, creates rows, trains, rewards,
//! shadows, drafts orders, accesses a broker or trades.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_specs::{
    CurrentHistoricalOutcomeFeatureLabelJoinTargetSpecification,
    HistoricalOutcomeFeatureLabelJoinSpecification,
    HistoricalOutcomeFeatureLabelJoinTargetSpecRecord, HistoricalOutcomeSemanticTargetDefinition,
    HistoricalOutcomeSemanticTargetSpecification,
    current_historical_outcome_feature_label_join_target_specifications_for_independent_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-spec-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-spec-review-v1";
const REVIEW_CONTRACT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-independent-review-contract-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-independent-semantic-audit-v1";
const POLICY_VERSION: &str = "hone-feature-label-join-target-independent-review-v1-no-execution";
const EXPECTED_REGISTRATION_POLICY_VERSION: &str =
    "hone-feature-label-join-target-governance-registration-v1-no-execution";
const EXPECTED_REGISTRATION_STATUS: &str = "registered_not_independently_reviewed_not_executed";
const EXPECTED_FEATURE_COUNT: usize = 65;
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];
const MAX_RATIONALE_CHARS: usize = 3_000;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict {
    ApprovedForFutureIsolatedJoinTargetImplementationRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecReviewContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub semantic_audit_implementation: String,
    pub required_join_checks: Vec<String>,
    pub required_target_checks: Vec<String>,
    pub approval_scope: String,
    pub primary_target_is_engineering_candidate_not_strategy_truth: bool,
    pub implementation_registration_separate: bool,
    pub join_execution_separate: bool,
    pub output_validation_separate: bool,
    pub training_and_reward_governance_separate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub specification_id: String,
    pub specification_sha256: String,
    pub specification_body_sha256: String,
    pub join_specification_sha256: String,
    pub target_specification_sha256: String,
    pub combined_artifact_sha256: String,
    pub record_hash_independently_reproduced: bool,
    pub specification_body_hash_independently_reproduced: bool,
    pub join_hash_independently_reproduced: bool,
    pub target_hash_independently_reproduced: bool,
    pub exact_current_artifact_binding_reproduced: bool,
    pub exact_feature_catalog_binding_reproduced: bool,
    pub join_cardinality_and_split_semantics_valid: bool,
    pub point_in_time_and_missingness_semantics_valid: bool,
    pub forbidden_input_and_holdout_isolation_valid: bool,
    pub exact_nine_continuous_target_semantics_valid: bool,
    pub primary_and_risk_roles_are_explicit_engineering_candidates: bool,
    pub no_action_position_threshold_ranking_or_reward_semantics: bool,
    pub all_execution_and_downstream_authority_closed: bool,
    pub target_ids: Vec<String>,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeFeatureLabelJoinTargetSpecRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_specification_sha256: String,
    expected_specification_body_sha256: String,
    expected_join_specification_sha256: String,
    expected_target_specification_sha256: String,
    expected_validation_sha256: String,
    expected_combined_artifact_sha256: String,
    expected_review_contract_sha256: String,
    verdict: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_specification_and_artifact_binding_confirmed: bool,
    reviewer_independence_confirmed: bool,
    independent_record_join_target_hash_reproduction_confirmed: bool,
    one_to_one_entry_join_and_duplicate_missing_failure_confirmed: bool,
    purge_embargo_exclusion_and_official_split_authority_confirmed: bool,
    point_in_time_feature_and_explicit_missingness_confirmed: bool,
    forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed: bool,
    split_specific_target_visibility_and_sealed_holdout_confirmed: bool,
    exact_nine_continuous_target_semantics_confirmed: bool,
    primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed: bool,
    exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed: bool,
    review_implementation_execution_and_output_validation_separation_confirmed: bool,
    no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub specification: HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
    pub review_contract: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewContract,
    pub independent_audit: HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub reviewer_independent_from_complete_prior_chain: bool,
    pub verdict: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_current_specification_and_artifact_binding_confirmed: bool,
    pub reviewer_independence_confirmed: bool,
    pub independent_record_join_target_hash_reproduction_confirmed: bool,
    pub one_to_one_entry_join_and_duplicate_missing_failure_confirmed: bool,
    pub purge_embargo_exclusion_and_official_split_authority_confirmed: bool,
    pub point_in_time_feature_and_explicit_missingness_confirmed: bool,
    pub forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed: bool,
    pub split_specific_target_visibility_and_sealed_holdout_confirmed: bool,
    pub exact_nine_continuous_target_semantics_confirmed: bool,
    pub primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed: bool,
    pub exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed: bool,
    pub review_implementation_execution_and_output_validation_separation_confirmed: bool,
    pub no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub future_isolated_join_target_implementation_registration_eligible: bool,
    pub join_target_implementation_registered: bool,
    pub join_execution_authorized: bool,
    pub join_executed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub semantic_target_assigned: bool,
    pub joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecReviewItem {
    pub specification: HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
    pub current_independent_audit: HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub upstream_binding_current: bool,
    pub latest_review: Option<HistoricalOutcomeFeatureLabelJoinTargetSpecReview>,
    pub review_eligible: bool,
    pub future_isolated_join_target_implementation_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_contract: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewContract,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetSpecReviewItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub implementation_registration_eligible_count: usize,
    pub review_status: String,
    pub join_target_implementation_registered: bool,
    pub join_execution_authorized: bool,
    pub join_executed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub semantic_target_assigned: bool,
    pub joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub training_data_access_authorized: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetSpecReviewReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub implementation_registration_eligible_count: usize,
    pub review_status: String,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_spec_reviews(
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
            warn!(%error, "feature-label join/target independent review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "特征—标签连接与目标规范独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_feature_label_join_target_spec(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(specification_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeFeatureLabelJoinTargetSpecRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &specification_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(registry) => Json(registry).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_feature_label_join_target_spec_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetSpecReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetSpecReviewReadinessSummary {
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            approved_count: registry.approved_count,
            current_binding_approved_count: registry.current_binding_approved_count,
            implementation_registration_eligible_count: registry
                .implementation_registration_eligible_count,
            review_status: registry.review_status,
        },
    )
}

pub(crate) async fn approved_historical_outcome_feature_label_join_target_spec_reviews(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetSpecReview>, String> {
    let registry = build_registry(state).await?;
    let mut reviews = registry
        .items
        .into_iter()
        .filter(|item| {
            item.upstream_binding_current
                && item.future_isolated_join_target_implementation_registration_eligible
        })
        .filter_map(|item| item.latest_review)
        .collect::<Vec<_>>();
    for review in &reviews {
        validate_historical_outcome_feature_label_join_target_spec_review_for_implementation(
            review,
        )?;
    }
    reviews.sort_by(|left, right| {
        right
            .submitted_at
            .cmp(&left.submitted_at)
            .then_with(|| right.review_id.cmp(&left.review_id))
    });
    Ok(reviews)
}

pub(crate) fn validate_historical_outcome_feature_label_join_target_spec_review_for_implementation(
    review: &HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
) -> Result<(), String> {
    validate_review(review)?;
    if !review.future_isolated_join_target_implementation_registration_eligible {
        return Err(
            "join/target specification review is not approved for implementation registration"
                .to_string(),
        );
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry, String> {
    let sources =
        current_historical_outcome_feature_label_join_target_specifications_for_independent_review(
            state,
        )
        .await?;
    let contract = review_contract();
    let mut items = Vec::with_capacity(sources.len());
    for source in sources {
        let specification_id = source.specification.specification_id.clone();
        let audit = independently_audit_source(&source)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("当前连接/目标规范独立语义审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews_for_spec(state, &specification_id).await?;
        let complete_review_actor_ids = reviews
            .iter()
            .map(|review| review.reviewer_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|review| {
            review.specification != source.specification || review.independent_audit != audit
        }) {
            return Err("连接/目标规范复核绑定与当前规范或独立审计已变化".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|review| {
            review.future_isolated_join_target_implementation_registration_eligible
        });
        items.push(HistoricalOutcomeFeatureLabelJoinTargetSpecReviewItem {
            specification: source.specification,
            current_independent_audit: audit,
            complete_review_actor_ids,
            upstream_binding_current: true,
            review_eligible: !approved,
            future_isolated_join_target_implementation_registration_eligible: approved,
            latest_review,
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
            item.latest_review.as_ref().is_some_and(|review| {
                review.future_isolated_join_target_implementation_registration_eligible
            })
        })
        .count();
    let current_binding_approved_count = items
        .iter()
        .filter(|item| item.future_isolated_join_target_implementation_registration_eligible)
        .count();
    let review_status = if items.is_empty() {
        "waiting_current_join_target_specification"
    } else if current_binding_approved_count > 0 {
        "current_join_target_specification_approved_only_for_future_isolated_implementation_registration"
    } else if review_eligible_count > 0 {
        "current_join_target_specification_ready_for_independent_review"
    } else {
        "join_target_specification_review_blocked_fail_closed"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetSpecReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_contract: contract,
        items,
        review_eligible_count,
        reviewed_count,
        approved_count,
        current_binding_approved_count,
        implementation_registration_eligible_count: current_binding_approved_count,
        review_status: review_status.to_string(),
        join_target_implementation_registered: false,
        join_execution_authorized: false,
        join_executed: false,
        semantic_target_assignment_authorized: false,
        semantic_target_assigned: false,
        joined_dataset_created: false,
        copied_to_training_store: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_run_allowed: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 37 阶段只由独立角色重算当前 join/target 规范的工件绑定、哈希、连接、防泄漏与九维连续目标语义。批准只允许未来登记隔离实现；当前不登记实现、不执行 join、不打开或分配标签、不创建训练行，也不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    specification_id: &str,
    request: ReviewHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetSpecReview, String> {
    if reviewer_id.trim().is_empty() || !valid_id(specification_id) {
        return Err("连接/目标规范独立复核人或规范 ID 无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let sources =
        current_historical_outcome_feature_label_join_target_specifications_for_independent_review(
            state,
        )
        .await?;
    let source = sources
        .into_iter()
        .find(|source| source.specification.specification_id == specification_id)
        .ok_or_else(|| "当前不可变连接/目标规范不存在或上游绑定已变化".to_string())?;
    let reviews = read_reviews_for_spec(state, specification_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|review| {
        review.future_isolated_join_target_implementation_registration_eligible
    }) {
        return Err("当前连接/目标规范已经独立批准，不得重复或分叉复核".to_string());
    }
    let review = build_review(&source, &reviews, reviewer_id, request, Utc::now())?;
    write_immutable_json(
        &review_root(state)
            .join(specification_id)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn build_review(
    source: &CurrentHistoricalOutcomeFeatureLabelJoinTargetSpecification,
    prior_reviews: &[HistoricalOutcomeFeatureLabelJoinTargetSpecReview],
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetSpecReview, String> {
    let specification = &source.specification;
    let audit = independently_audit_source(source)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("独立语义与指纹审计不通过，不得提交复核".to_string());
    }
    let prior = prior_reviews.last();
    let contract = review_contract();
    if request.expected_review_id != prior.map(|review| review.review_id.clone())
        || request.expected_review_sha256 != prior.map(|review| review.review_sha256.clone())
        || request.expected_specification_sha256 != specification.specification_sha256
        || request.expected_specification_body_sha256 != specification.specification_body_sha256
        || request.expected_join_specification_sha256
            != specification.join_specification.specification_sha256
        || request.expected_target_specification_sha256
            != specification.target_specification.specification_sha256
        || request.expected_validation_sha256 != specification.validation_sha256
        || request.expected_combined_artifact_sha256 != specification.combined_artifact_sha256
        || request.expected_review_contract_sha256 != contract.contract_sha256
    {
        return Err("连接/目标规范、工件、前序复核或复核合同绑定已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(specification, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == reviewer_id)
    {
        return Err(
            "复核人参与过规范登记、上游生产/校验或此前复核，必须更换独立复核人".to_string(),
        );
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let approval = request.verdict
        == HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ApprovedForFutureIsolatedJoinTargetImplementationRegistration;
    let all_confirmed = request_confirmations(&request);
    if approval && !all_confirmed {
        return Err("批准未来实现登记前必须逐项确认全部语义、独立性和零执行边界".to_string());
    }
    let mut review = HistoricalOutcomeFeatureLabelJoinTargetSpecReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: prior.map(|review| review.review_id.clone()),
        previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
        specification: specification.clone(),
        review_contract: contract,
        independent_audit: audit,
        submitted_at,
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        reviewer_independent_from_complete_prior_chain: true,
        verdict: request.verdict,
        rationale,
        known_limitations,
        exact_current_specification_and_artifact_binding_confirmed: request
            .exact_current_specification_and_artifact_binding_confirmed,
        reviewer_independence_confirmed: request.reviewer_independence_confirmed,
        independent_record_join_target_hash_reproduction_confirmed: request
            .independent_record_join_target_hash_reproduction_confirmed,
        one_to_one_entry_join_and_duplicate_missing_failure_confirmed: request
            .one_to_one_entry_join_and_duplicate_missing_failure_confirmed,
        purge_embargo_exclusion_and_official_split_authority_confirmed: request
            .purge_embargo_exclusion_and_official_split_authority_confirmed,
        point_in_time_feature_and_explicit_missingness_confirmed: request
            .point_in_time_feature_and_explicit_missingness_confirmed,
        forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed: request
            .forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed,
        split_specific_target_visibility_and_sealed_holdout_confirmed: request
            .split_specific_target_visibility_and_sealed_holdout_confirmed,
        exact_nine_continuous_target_semantics_confirmed: request
            .exact_nine_continuous_target_semantics_confirmed,
        primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed: request
            .primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed,
        exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed: request
            .exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed,
        review_implementation_execution_and_output_validation_separation_confirmed: request
            .review_implementation_execution_and_output_validation_separation_confirmed,
        no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed: request
            .no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed,
        future_isolated_join_target_implementation_registration_eligible: approval && all_confirmed,
        join_target_implementation_registered: false,
        join_execution_authorized: false,
        join_executed: false,
        semantic_target_assignment_authorized: false,
        semantic_target_assigned: false,
        joined_dataset_created: false,
        copied_to_training_store: false,
        training_data_access_authorized: false,
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

fn independently_audit_source(
    source: &CurrentHistoricalOutcomeFeatureLabelJoinTargetSpecification,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit, String> {
    let specification = &source.specification;
    let pair = &source.validated_artifact_pair;
    let join = &specification.join_specification;
    let target = &specification.target_specification;
    let record_hash_independently_reproduced = specification.specification_sha256
        == fingerprint_without(specification, &["specification_id", "specification_sha256"])?;
    let specification_body_hash_independently_reproduced = specification.specification_body_sha256
        == hash_serializable(&(
            join,
            target,
            specification.specification_name.as_str(),
            specification.code_revision.as_str(),
            specification.rationale.as_str(),
            specification.known_limitations.as_str(),
        ))?;
    let join_hash_independently_reproduced =
        join.specification_sha256 == fingerprint_without(join, &["specification_sha256"])?;
    let target_hash_independently_reproduced =
        target.specification_sha256 == fingerprint_without(target, &["specification_sha256"])?;
    let validation = &pair.validation;
    let bundle = &pair.artifact_pair.feature_bundle;
    let exact_current_artifact_binding_reproduced = specification.transformation_attempt_id
        == validation.transformation_attempt_id
        && specification.validation_id == validation.validation_id
        && specification.validation_sha256 == validation.validation_sha256
        && specification.materialization_id == validation.materialization_id
        && specification.materialization_claim_sha256 == validation.materialization_claim_sha256
        && specification.materialization_result_sha256 == validation.materialization_result_sha256
        && specification.split_manifest_sha256 == validation.split_manifest_sha256
        && specification.feature_bundle_sha256 == validation.feature_bundle_sha256
        && specification.combined_artifact_sha256 == validation.combined_artifact_sha256
        && specification.dataset_id == validation.dataset_id
        && specification.dataset_content_sha256 == validation.dataset_content_sha256
        && specification.dataset_manifest_sha256 == validation.dataset_manifest_sha256
        && specification.candidate_set_sha256 == validation.candidate_set_sha256;
    let exact_feature_catalog_binding_reproduced = join.feature_catalog_count
        == EXPECTED_FEATURE_COUNT
        && join.feature_catalog_count == bundle.feature_catalog_count
        && join.feature_catalog_sha256 == bundle.feature_catalog_sha256
        && join.feature_schema_sha256 == bundle.feature_schema_sha256
        && bundle.records.len()
            == bundle
                .records
                .iter()
                .map(|record| {
                    (
                        record.dataset_entry_id.clone(),
                        record.feature_namespace.clone(),
                        record.feature_id.clone(),
                    )
                })
                .collect::<BTreeSet<_>>()
                .len();
    let join_cardinality_and_split_semantics_valid = join_semantics_exact(join);
    let point_in_time_and_missingness_semantics_valid = join.feature_availability_rule
        == "available_at_utc_must_be_lte_dataset_entry.decision_available_at_and_artifact_source_version_must_match_the_official_bundle"
        && join.explicit_missingness_rule
            == "preserve_every_allowlisted_feature_record; missing_value_remains_null_with_nonempty_reason; no_row_or_feature_drop_due_to_missingness"
        && join.all_allowlisted_feature_records_preserved
        && !join.imputation_allowed
        && !join.interpolation_allowed;
    let forbidden_input_and_holdout_isolation_valid = join.forbidden_join_inputs
        == [
            "post_decision_source",
            "future_market_data",
            "outcome_or_label_as_feature",
            "sealed_holdout_label_for_training_or_tuning",
            "current_portfolio_state",
            "model_generated_backfill",
        ]
        && !join.sealed_holdout_labels_opened
        && !join.join_executed;
    let exact_nine_continuous_target_semantics_valid = target_semantics_exact(target);
    let primary_and_risk_roles_are_explicit_engineering_candidates =
        target.primary_supervised_target_id == "excess_return_250_continuous"
            && target.risk_target_id == "asset_max_drawdown_250_continuous"
            && target
                .target_definitions
                .iter()
                .find(|definition| definition.target_id == target.primary_supervised_target_id)
                .is_some_and(|definition| definition.role == "primary")
            && target
                .target_definitions
                .iter()
                .find(|definition| definition.target_id == target.risk_target_id)
                .is_some_and(|definition| definition.role == "risk");
    let no_action_position_threshold_ranking_or_reward_semantics = target.exact_f64_bits_preserved
        && !target.normalization_allowed
        && !target.winsorization_allowed
        && !target.rank_transform_allowed
        && !target.categorical_action_label_defined
        && !target.buy_hold_sell_threshold_defined
        && !target.portfolio_weight_target_defined
        && !target.scalar_reward_defined
        && !target.semantic_target_assignment_performed;
    let all_execution_and_downstream_authority_closed =
        registration_authority_closed(specification);
    let record_envelope_exact = specification.schema_version
        == "hone-historical-outcome-feature-label-join-target-spec-v1"
        && specification.policy_version == EXPECTED_REGISTRATION_POLICY_VERSION
        && specification.status == EXPECTED_REGISTRATION_STATUS
        && specification.specification_id == specification.specification_sha256[..32]
        && specification.future_independent_spec_review_eligible
        && specification.registrar_independent_from_complete_prior_chain;
    let checks = [
        (record_hash_independently_reproduced, "record_hash_mismatch"),
        (
            specification_body_hash_independently_reproduced,
            "specification_body_hash_mismatch",
        ),
        (join_hash_independently_reproduced, "join_hash_mismatch"),
        (target_hash_independently_reproduced, "target_hash_mismatch"),
        (
            exact_current_artifact_binding_reproduced,
            "current_artifact_binding_mismatch",
        ),
        (
            exact_feature_catalog_binding_reproduced,
            "feature_catalog_binding_mismatch",
        ),
        (
            join_cardinality_and_split_semantics_valid,
            "join_cardinality_or_split_semantics_invalid",
        ),
        (
            point_in_time_and_missingness_semantics_valid,
            "point_in_time_or_missingness_semantics_invalid",
        ),
        (
            forbidden_input_and_holdout_isolation_valid,
            "forbidden_input_or_holdout_isolation_invalid",
        ),
        (
            exact_nine_continuous_target_semantics_valid,
            "continuous_target_semantics_invalid",
        ),
        (
            primary_and_risk_roles_are_explicit_engineering_candidates,
            "primary_or_risk_target_role_invalid",
        ),
        (
            no_action_position_threshold_ranking_or_reward_semantics,
            "action_position_threshold_ranking_or_reward_semantics_present",
        ),
        (
            all_execution_and_downstream_authority_closed,
            "execution_or_downstream_authority_open",
        ),
        (record_envelope_exact, "registration_envelope_invalid"),
    ];
    let mismatch_reasons = checks
        .iter()
        .filter(|(passed, _)| !passed)
        .map(|(_, reason)| (*reason).to_string())
        .collect::<Vec<_>>();
    let target_ids = target
        .target_definitions
        .iter()
        .map(|definition| definition.target_id.clone())
        .collect::<Vec<_>>();
    let mut audit = HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        specification_id: specification.specification_id.clone(),
        specification_sha256: specification.specification_sha256.clone(),
        specification_body_sha256: specification.specification_body_sha256.clone(),
        join_specification_sha256: join.specification_sha256.clone(),
        target_specification_sha256: target.specification_sha256.clone(),
        combined_artifact_sha256: specification.combined_artifact_sha256.clone(),
        record_hash_independently_reproduced,
        specification_body_hash_independently_reproduced,
        join_hash_independently_reproduced,
        target_hash_independently_reproduced,
        exact_current_artifact_binding_reproduced,
        exact_feature_catalog_binding_reproduced,
        join_cardinality_and_split_semantics_valid,
        point_in_time_and_missingness_semantics_valid,
        forbidden_input_and_holdout_isolation_valid,
        exact_nine_continuous_target_semantics_valid,
        primary_and_risk_roles_are_explicit_engineering_candidates,
        no_action_position_threshold_ranking_or_reward_semantics,
        all_execution_and_downstream_authority_closed,
        target_ids,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn join_semantics_exact(join: &HistoricalOutcomeFeatureLabelJoinSpecification) -> bool {
    join.schema_version == "hone-historical-outcome-exact-feature-label-join-spec-v1"
        && join.dataset_entry_key == "dataset.entries.entry_id"
        && join.split_record_key == "official_split_manifest.records.dataset_entry_id"
        && join.feature_record_key_fields == ["dataset_entry_id", "feature_namespace", "feature_id"]
        && join.raw_outcome_record_key == "dataset.entries.entry_id"
        && join.join_cardinality_rule
            == "exactly_one_split_record_and_one_raw_outcome_record_per_dataset_entry; exactly_one_record_per_allowlisted_feature_id; duplicates_or_missing_keys_fail_closed"
        && join.split_authority_rule
            == "official_split_manifest_is_only_split_authority; no_reassignment_or_cross_component_move"
        && join.purged_or_embargoed_row_policy
            == "exclude_from_all_joined_training_or_evaluation_rows_and_preserve_audit_record"
        && join.allowed_label_horizons_market_sessions.as_slice() == REQUIRED_HORIZONS
        && join.one_to_one_outcome_join_required
        && join.all_allowlisted_feature_records_preserved
        && !join.imputation_allowed
        && !join.interpolation_allowed
        && !join.sealed_holdout_labels_opened
        && !join.join_executed
}

fn target_semantics_exact(target: &HistoricalOutcomeSemanticTargetSpecification) -> bool {
    let definitions = expected_target_definitions();
    let order = definitions
        .iter()
        .map(|definition| definition.target_id.clone())
        .collect::<Vec<_>>();
    target.schema_version == "hone-historical-outcome-continuous-target-vector-spec-v1"
        && target.prediction_task
            == "predict_continuous_forward_asset_path_and_benchmark_relative_outcomes_from_point_in_time_features; no_action_inference"
        && target.target_definitions == definitions
        && target.target_vector_order == order
        && target.primary_supervised_target_id == "excess_return_250_continuous"
        && target.risk_target_id == "asset_max_drawdown_250_continuous"
        && target.auxiliary_target_ids.len() == 7
        && target.benchmark_return_role
            == "audit_component_only_for_recomputing_excess_return; not_an_independent_supervised_target"
        && target.duplicate_horizon_policy == "fail_closed"
        && target.missing_horizon_policy
            == "exclude_row_before_split_specific_target_access_and_preserve_failure_audit; no_partial_target_vector"
        && target.train_target_access_policy
            == "future_authorized_training_only_after_join_materialization_and_independent_output_validation"
        && target.validation_target_access_policy
            == "future_authorized_model_selection_evaluator_only_after_model_candidate_freeze"
        && target.sealed_holdout_target_access_policy
            == "future_one_shot_independent_evaluator_only_after_model_and_protocol_freeze; never_training_or_tuning"
        && target.exact_f64_bits_preserved
        && !target.normalization_allowed
        && !target.winsorization_allowed
        && !target.rank_transform_allowed
        && !target.categorical_action_label_defined
        && !target.buy_hold_sell_threshold_defined
        && !target.portfolio_weight_target_defined
        && !target.scalar_reward_defined
        && !target.semantic_target_assignment_performed
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

fn registration_authority_closed(
    specification: &HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
) -> bool {
    !specification.independent_spec_review_completed
        && !specification.join_execution_authorized
        && !specification.feature_join_performed
        && !specification.semantic_target_assignment_authorized
        && !specification.semantic_target_assigned
        && !specification.joined_dataset_created
        && !specification.copied_to_training_store
        && !specification.training_authorized
        && !specification.reward_authorized
        && !specification.shadow_portfolio_authorized
        && !specification.order_generation_authorized
        && !specification.broker_access_authorized
        && !specification.trading_authorized
}

fn request_confirmations(
    request: &ReviewHistoricalOutcomeFeatureLabelJoinTargetSpecRequest,
) -> bool {
    request.exact_current_specification_and_artifact_binding_confirmed
        && request.reviewer_independence_confirmed
        && request.independent_record_join_target_hash_reproduction_confirmed
        && request.one_to_one_entry_join_and_duplicate_missing_failure_confirmed
        && request.purge_embargo_exclusion_and_official_split_authority_confirmed
        && request.point_in_time_feature_and_explicit_missingness_confirmed
        && request.forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed
        && request.split_specific_target_visibility_and_sealed_holdout_confirmed
        && request.exact_nine_continuous_target_semantics_confirmed
        && request.primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed
        && request.exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed
        && request.review_implementation_execution_and_output_validation_separation_confirmed
        && request.no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed
}

fn review_contract() -> HistoricalOutcomeFeatureLabelJoinTargetSpecReviewContract {
    let mut contract = HistoricalOutcomeFeatureLabelJoinTargetSpecReviewContract {
        schema_version: REVIEW_CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        semantic_audit_implementation:
            "hone-independent-join-target-semantic-fingerprint-audit-v1-no-registration-validator-reuse"
                .to_string(),
        required_join_checks: vec![
            "exact_current_official_artifact_and_registration_hash_binding".to_string(),
            "one_dataset_entry_one_split_one_raw_outcome_and_one_each_allowlisted_feature"
                .to_string(),
            "official_split_authority_and_purge_embargo_rows_excluded".to_string(),
            "point_in_time_availability_and_explicit_missingness_without_imputation".to_string(),
            "future_outcome_holdout_current_portfolio_and_model_backfill_excluded".to_string(),
            "split_specific_target_visibility_and_sealed_holdout_isolation".to_string(),
        ],
        required_target_checks: vec![
            "exact_20_60_250_asset_excess_and_max_drawdown_continuous_vector".to_string(),
            "exact_f64_identity_without_normalization_winsorization_or_ranking".to_string(),
            "primary_250_excess_and_risk_250_drawdown_are_engineering_candidates_only"
                .to_string(),
            "no_action_class_buy_hold_sell_threshold_position_target_or_scalar_reward"
                .to_string(),
        ],
        approval_scope:
            "future_isolated_join_target_implementation_registration_only_no_execution"
                .to_string(),
        primary_target_is_engineering_candidate_not_strategy_truth: true,
        implementation_registration_separate: true,
        join_execution_separate: true,
        output_validation_separate: true,
        training_and_reward_governance_separate: true,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static join/target review contract is serializable");
    contract
}

fn expected_excluded_prior_actor_ids(
    specification: &HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
    prior_reviews: &[HistoricalOutcomeFeatureLabelJoinTargetSpecReview],
) -> Vec<String> {
    let mut actors = specification.excluded_prior_actor_ids.clone();
    actors.push(specification.registered_by.clone());
    actors.extend(
        prior_reviews
            .iter()
            .map(|review| review.reviewer_id.clone()),
    );
    actors.retain(|actor| !actor.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn validate_review(
    review: &HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
) -> Result<(), String> {
    let approval = review.verdict
        == HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ApprovedForFutureIsolatedJoinTargetImplementationRegistration;
    let all_confirmed = review.exact_current_specification_and_artifact_binding_confirmed
        && review.reviewer_independence_confirmed
        && review.independent_record_join_target_hash_reproduction_confirmed
        && review.one_to_one_entry_join_and_duplicate_missing_failure_confirmed
        && review.purge_embargo_exclusion_and_official_split_authority_confirmed
        && review.point_in_time_feature_and_explicit_missingness_confirmed
        && review.forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed
        && review.split_specific_target_visibility_and_sealed_holdout_confirmed
        && review.exact_nine_continuous_target_semantics_confirmed
        && review.primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed
        && review.exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed
        && review.review_implementation_execution_and_output_validation_separation_confirmed
        && review.no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !review.join_target_implementation_registered
        && !review.join_execution_authorized
        && !review.join_executed
        && !review.semantic_target_assignment_authorized
        && !review.semantic_target_assigned
        && !review.joined_dataset_created
        && !review.copied_to_training_store
        && !review.training_data_access_authorized
        && !review.training_authorized
        && !review.training_run_allowed
        && !review.reward_authorized
        && !review.shadow_portfolio_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let audit_valid = review.independent_audit.schema_version == AUDIT_SCHEMA_VERSION
        && valid_sha256(&review.independent_audit.audit_sha256)
        && review.independent_audit.audit_sha256
            == fingerprint_without(&review.independent_audit, &["audit_sha256"])?
        && review.independent_audit.mismatch_reasons.is_empty()
        && review
            .independent_audit
            .all_execution_and_downstream_authority_closed;
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != POLICY_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.review_sha256)
        || review.review_id != review.review_sha256[..32]
        || review.review_sha256 != review_fingerprint(review)?
        || review.review_contract != review_contract()
        || !audit_valid
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review.reviewer_id.trim().is_empty()
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || !review.reviewer_independent_from_complete_prior_chain
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || review.known_limitations.trim().is_empty()
        || review.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || review.future_isolated_join_target_implementation_registration_eligible
            != (approval && all_confirmed)
        || !authority_closed
    {
        return Err("join/target specification review is invalid or over-authorized".to_string());
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
        return Err("join/target specification review predecessor is invalid".to_string());
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[HistoricalOutcomeFeatureLabelJoinTargetSpecReview],
    specification_id: &str,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetSpecReview>, String> {
    if reviews.is_empty() {
        return Ok(Vec::new());
    }
    let mut by_id = HashMap::new();
    let mut child_by_parent = HashMap::<String, String>::new();
    let mut roots = Vec::new();
    for review in reviews {
        validate_review(review)?;
        if review.specification.specification_id != specification_id
            || by_id
                .insert(review.review_id.clone(), review.clone())
                .is_some()
        {
            return Err("join/target review replay or subject mismatch".to_string());
        }
        match &review.previous_review_id {
            Some(parent) => {
                if child_by_parent
                    .insert(parent.clone(), review.review_id.clone())
                    .is_some()
                {
                    return Err("join/target review chain branches".to_string());
                }
            }
            None => roots.push(review.review_id.clone()),
        }
    }
    if roots.len() != 1 {
        return Err("join/target review chain has no single root".to_string());
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = roots.pop().unwrap();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err("join/target review chain contains a cycle".to_string());
        }
        let review = by_id
            .get(&current)
            .ok_or_else(|| "join/target review chain is disconnected".to_string())?
            .clone();
        if let Some(parent_id) = &review.previous_review_id {
            let parent = by_id
                .get(parent_id)
                .ok_or_else(|| "join/target review predecessor is missing".to_string())?;
            if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                return Err("join/target review predecessor hash drift".to_string());
            }
        }
        let ancestor_reviews = ordered.clone();
        if review.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&review.specification, &ancestor_reviews)
        {
            return Err("join/target review role-exclusion history mismatch".to_string());
        }
        if review.future_isolated_join_target_implementation_registration_eligible
            && child_by_parent.contains_key(&current)
        {
            return Err(
                "approved join/target review must be the terminal chain record".to_string(),
            );
        }
        ordered.push(review);
        match child_by_parent.get(&current) {
            Some(next) => current = next.clone(),
            None => break,
        }
    }
    if ordered.len() != reviews.len() {
        return Err("join/target review chain is disconnected".to_string());
    }
    Ok(ordered)
}

async fn read_reviews_for_spec(
    state: &AppState,
    specification_id: &str,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetSpecReview>, String> {
    let root = review_root(state).join(specification_id);
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
        let review: HistoricalOutcomeFeatureLabelJoinTargetSpecReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("join/target review path mismatch".to_string());
        }
        reviews.push(review);
    }
    validate_review_chain(&reviews, specification_id)
}

fn review_fingerprint(
    review: &HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
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
    let bytes = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
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

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
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
    decision_root(state).join("historical_outcome_feature_label_join_target_spec_reviews/reviews")
}

struct ReviewLock {
    path: PathBuf,
}

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("historical-outcome-feature-label-join-target-spec-review.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ReviewLock { path }),
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
                return Err("另一个管理员正在复核连接/目标规范".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("连接/目标规范复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn hash() -> String {
        "a".repeat(64)
    }

    fn id() -> String {
        "a".repeat(32)
    }

    fn sample_target_specification() -> HistoricalOutcomeSemanticTargetSpecification {
        let definitions = expected_target_definitions();
        let order = definitions
            .iter()
            .map(|definition| definition.target_id.clone())
            .collect::<Vec<_>>();
        let mut target = HistoricalOutcomeSemanticTargetSpecification {
            schema_version:
                "hone-historical-outcome-continuous-target-vector-spec-v1".to_string(),
            specification_sha256: String::new(),
            prediction_task: "predict_continuous_forward_asset_path_and_benchmark_relative_outcomes_from_point_in_time_features; no_action_inference".to_string(),
            target_definitions: definitions,
            primary_supervised_target_id: "excess_return_250_continuous".to_string(),
            risk_target_id: "asset_max_drawdown_250_continuous".to_string(),
            auxiliary_target_ids: order
                .iter()
                .filter(|id| {
                    id.as_str() != "excess_return_250_continuous"
                        && id.as_str() != "asset_max_drawdown_250_continuous"
                })
                .cloned()
                .collect(),
            benchmark_return_role: "audit_component_only_for_recomputing_excess_return; not_an_independent_supervised_target".to_string(),
            target_vector_order: order,
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
        target.specification_sha256 =
            fingerprint_without(&target, &["specification_sha256"]).unwrap();
        target
    }

    fn sample_join_specification() -> HistoricalOutcomeFeatureLabelJoinSpecification {
        HistoricalOutcomeFeatureLabelJoinSpecification {
            schema_version: "hone-historical-outcome-exact-feature-label-join-spec-v1".into(),
            specification_sha256: hash(),
            dataset_id: id(),
            dataset_content_sha256: hash(),
            dataset_manifest_sha256: hash(),
            candidate_set_sha256: hash(),
            split_manifest_sha256: hash(),
            feature_bundle_sha256: hash(),
            combined_artifact_sha256: hash(),
            dataset_entry_key: "dataset.entries.entry_id".into(),
            split_record_key: "official_split_manifest.records.dataset_entry_id".into(),
            feature_record_key_fields: vec![
                "dataset_entry_id".into(),
                "feature_namespace".into(),
                "feature_id".into(),
            ],
            raw_outcome_record_key: "dataset.entries.entry_id".into(),
            join_cardinality_rule: "exactly_one_split_record_and_one_raw_outcome_record_per_dataset_entry; exactly_one_record_per_allowlisted_feature_id; duplicates_or_missing_keys_fail_closed".into(),
            split_authority_rule: "official_split_manifest_is_only_split_authority; no_reassignment_or_cross_component_move".into(),
            purged_or_embargoed_row_policy: "exclude_from_all_joined_training_or_evaluation_rows_and_preserve_audit_record".into(),
            train_target_visibility_policy: "train".into(),
            validation_target_visibility_policy: "validation".into(),
            sealed_holdout_target_visibility_policy: "sealed".into(),
            feature_availability_rule: "available_at_utc_must_be_lte_dataset_entry.decision_available_at_and_artifact_source_version_must_match_the_official_bundle".into(),
            explicit_missingness_rule: "preserve_every_allowlisted_feature_record; missing_value_remains_null_with_nonempty_reason; no_row_or_feature_drop_due_to_missingness".into(),
            feature_catalog_count: EXPECTED_FEATURE_COUNT,
            feature_catalog_sha256: hash(),
            feature_schema_sha256: hash(),
            allowed_label_horizons_market_sessions: REQUIRED_HORIZONS.to_vec(),
            forbidden_join_inputs: vec![
                "post_decision_source".into(),
                "future_market_data".into(),
                "outcome_or_label_as_feature".into(),
                "sealed_holdout_label_for_training_or_tuning".into(),
                "current_portfolio_state".into(),
                "model_generated_backfill".into(),
            ],
            joined_row_schema_fields: Vec::new(),
            one_to_one_outcome_join_required: true,
            all_allowlisted_feature_records_preserved: true,
            imputation_allowed: false,
            interpolation_allowed: false,
            sealed_holdout_labels_opened: false,
            join_executed: false,
        }
    }

    fn sample_specification() -> HistoricalOutcomeFeatureLabelJoinTargetSpecRecord {
        HistoricalOutcomeFeatureLabelJoinTargetSpecRecord {
            schema_version: "hone-historical-outcome-feature-label-join-target-spec-v1".into(),
            policy_version: EXPECTED_REGISTRATION_POLICY_VERSION.into(),
            specification_id: id(),
            specification_sha256: hash(),
            transformation_attempt_id: id(),
            validation_id: id(),
            validation_sha256: hash(),
            materialization_id: id(),
            materialization_claim_sha256: hash(),
            materialization_result_sha256: hash(),
            split_manifest_sha256: hash(),
            feature_bundle_sha256: hash(),
            combined_artifact_sha256: hash(),
            dataset_id: id(),
            dataset_content_sha256: hash(),
            dataset_manifest_sha256: hash(),
            candidate_set_sha256: hash(),
            registered_at: DateTime::from_timestamp(1_740_000_000, 0).unwrap(),
            registered_by: "registrar".into(),
            excluded_prior_actor_ids: vec!["upstream".into()],
            registrar_independent_from_complete_prior_chain: true,
            specification_name: "sample".into(),
            code_revision: "revision".into(),
            rationale: "rationale".into(),
            known_limitations: "limitations".into(),
            join_specification: sample_join_specification(),
            target_specification: sample_target_specification(),
            specification_body_sha256: hash(),
            status: EXPECTED_REGISTRATION_STATUS.into(),
            exact_validated_artifact_pair_binding_confirmed: true,
            registrar_independence_confirmed: true,
            exact_dataset_entry_one_to_one_join_confirmed: true,
            purged_and_embargoed_rows_excluded_confirmed: true,
            point_in_time_feature_availability_confirmed: true,
            sealed_holdout_target_isolation_confirmed: true,
            exact_raw_metric_bits_without_transform_confirmed: true,
            continuous_target_vector_not_action_or_reward_confirmed: true,
            explicit_missingness_without_imputation_confirmed: true,
            registration_review_execution_separation_confirmed: true,
            no_join_target_assignment_training_reward_shadow_order_broker_or_trading_confirmed:
                true,
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
        }
    }

    fn sample_audit(
        specification: &HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
    ) -> HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit {
        let mut audit = HistoricalOutcomeFeatureLabelJoinTargetIndependentAudit {
            schema_version: AUDIT_SCHEMA_VERSION.into(),
            audit_sha256: String::new(),
            specification_id: specification.specification_id.clone(),
            specification_sha256: specification.specification_sha256.clone(),
            specification_body_sha256: specification.specification_body_sha256.clone(),
            join_specification_sha256: specification
                .join_specification
                .specification_sha256
                .clone(),
            target_specification_sha256: specification
                .target_specification
                .specification_sha256
                .clone(),
            combined_artifact_sha256: specification.combined_artifact_sha256.clone(),
            record_hash_independently_reproduced: true,
            specification_body_hash_independently_reproduced: true,
            join_hash_independently_reproduced: true,
            target_hash_independently_reproduced: true,
            exact_current_artifact_binding_reproduced: true,
            exact_feature_catalog_binding_reproduced: true,
            join_cardinality_and_split_semantics_valid: true,
            point_in_time_and_missingness_semantics_valid: true,
            forbidden_input_and_holdout_isolation_valid: true,
            exact_nine_continuous_target_semantics_valid: true,
            primary_and_risk_roles_are_explicit_engineering_candidates: true,
            no_action_position_threshold_ranking_or_reward_semantics: true,
            all_execution_and_downstream_authority_closed: true,
            target_ids: expected_target_definitions()
                .into_iter()
                .map(|definition| definition.target_id)
                .collect(),
            mismatch_reasons: Vec::new(),
        };
        audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"]).unwrap();
        audit
    }

    fn sample_review(
        specification: &HistoricalOutcomeFeatureLabelJoinTargetSpecRecord,
        reviewer_id: &str,
        prior: Option<&HistoricalOutcomeFeatureLabelJoinTargetSpecReview>,
        verdict: HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict,
        excluded_prior_actor_ids: Vec<String>,
    ) -> HistoricalOutcomeFeatureLabelJoinTargetSpecReview {
        let approved = verdict
            == HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ApprovedForFutureIsolatedJoinTargetImplementationRegistration;
        let mut review = HistoricalOutcomeFeatureLabelJoinTargetSpecReview {
            schema_version: REVIEW_SCHEMA_VERSION.into(),
            policy_version: POLICY_VERSION.into(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id: prior.map(|review| review.review_id.clone()),
            previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
            specification: specification.clone(),
            review_contract: review_contract(),
            independent_audit: sample_audit(specification),
            submitted_at: DateTime::from_timestamp(1_745_000_000, 0).unwrap(),
            reviewer_id: reviewer_id.into(),
            excluded_prior_actor_ids,
            reviewer_independent_from_complete_prior_chain: true,
            verdict,
            rationale: "independent rationale".into(),
            known_limitations: "known limitations".into(),
            exact_current_specification_and_artifact_binding_confirmed: true,
            reviewer_independence_confirmed: true,
            independent_record_join_target_hash_reproduction_confirmed: true,
            one_to_one_entry_join_and_duplicate_missing_failure_confirmed: true,
            purge_embargo_exclusion_and_official_split_authority_confirmed: true,
            point_in_time_feature_and_explicit_missingness_confirmed: true,
            forbidden_future_outcome_holdout_portfolio_and_model_inputs_confirmed: true,
            split_specific_target_visibility_and_sealed_holdout_confirmed: true,
            exact_nine_continuous_target_semantics_confirmed: true,
            primary_and_risk_targets_are_engineering_candidates_not_strategy_truth_confirmed: true,
            exact_f64_identity_without_normalization_ranking_or_thresholds_confirmed: true,
            review_implementation_execution_and_output_validation_separation_confirmed: true,
            no_join_assignment_training_reward_shadow_order_broker_or_trading_confirmed: true,
            future_isolated_join_target_implementation_registration_eligible: approved,
            join_target_implementation_registered: false,
            join_execution_authorized: false,
            join_executed: false,
            semantic_target_assignment_authorized: false,
            semantic_target_assigned: false,
            joined_dataset_created: false,
            copied_to_training_store: false,
            training_data_access_authorized: false,
            training_authorized: false,
            training_run_allowed: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        review
    }

    pub(crate) fn approved_review() -> HistoricalOutcomeFeatureLabelJoinTargetSpecReview {
        let specification = sample_specification();
        sample_review(
            &specification,
            "join-target-spec-reviewer",
            None,
            HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ApprovedForFutureIsolatedJoinTargetImplementationRegistration,
            expected_excluded_prior_actor_ids(&specification, &[]),
        )
    }

    #[test]
    fn continuous_target_vector_has_exact_metrics_horizons_and_roles() {
        let target = sample_target_specification();
        assert!(target_semantics_exact(&target));
        assert_eq!(target.target_definitions.len(), 9);
        assert_eq!(
            target.primary_supervised_target_id,
            "excess_return_250_continuous"
        );
        assert_eq!(target.risk_target_id, "asset_max_drawdown_250_continuous");
    }

    #[test]
    fn target_semantic_drift_fails_independent_audit() {
        let mut target = sample_target_specification();
        target.target_definitions[4].source_selector =
            "raw_validated_metrics[horizon_market_sessions=250].excess_return".to_string();
        assert!(!target_semantics_exact(&target));
    }

    #[test]
    fn action_position_rank_and_reward_targets_are_absent() {
        let target = sample_target_specification();
        assert!(!target.categorical_action_label_defined);
        assert!(!target.buy_hold_sell_threshold_defined);
        assert!(!target.portfolio_weight_target_defined);
        assert!(!target.rank_transform_allowed);
        assert!(!target.scalar_reward_defined);
    }

    #[test]
    fn review_contract_calls_primary_target_an_engineering_candidate() {
        let contract = review_contract();
        assert!(contract.primary_target_is_engineering_candidate_not_strategy_truth);
        assert_eq!(
            contract.approval_scope,
            "future_isolated_join_target_implementation_registration_only_no_execution"
        );
        assert!(contract.join_execution_separate);
        assert!(contract.training_and_reward_governance_separate);
    }

    #[test]
    fn contract_fingerprint_detects_scope_tampering() {
        let mut contract = review_contract();
        let original = contract.contract_sha256.clone();
        contract.approval_scope = "execute_join".to_string();
        assert_ne!(
            original,
            fingerprint_without(&contract, &["contract_sha256"]).unwrap()
        );
    }

    #[test]
    fn required_target_sources_cover_three_metrics_at_three_horizons() {
        let identities = expected_target_definitions()
            .into_iter()
            .map(|definition| {
                (
                    definition.horizon_market_sessions,
                    definition.source_metric_field,
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(identities.len(), 9);
        for horizon in REQUIRED_HORIZONS {
            for metric in ["asset_return", "excess_return", "asset_max_drawdown"] {
                assert!(identities.contains(&(horizon, metric.to_string())));
            }
        }
    }

    #[test]
    fn approved_review_rejects_any_open_training_or_execution_authority() {
        let specification = sample_specification();
        let excluded = expected_excluded_prior_actor_ids(&specification, &[]);
        let mut review = sample_review(
            &specification,
            "reviewer-a",
            None,
            HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ApprovedForFutureIsolatedJoinTargetImplementationRegistration,
            excluded,
        );
        assert!(validate_review(&review).is_ok());
        review.training_authorized = true;
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review).is_err());
    }

    #[test]
    fn review_chain_requires_every_prior_reviewer_in_role_exclusion() {
        let specification = sample_specification();
        let first = sample_review(
            &specification,
            "reviewer-a",
            None,
            HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ChangesRequested,
            expected_excluded_prior_actor_ids(&specification, &[]),
        );
        let second = sample_review(
            &specification,
            "reviewer-b",
            Some(&first),
            HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ChangesRequested,
            expected_excluded_prior_actor_ids(&specification, &[]),
        );
        assert!(validate_review_chain(&[first, second], &specification.specification_id).is_err());
    }

    #[test]
    fn approved_review_is_terminal_even_if_disk_chain_is_tampered() {
        let specification = sample_specification();
        let first = sample_review(
            &specification,
            "reviewer-a",
            None,
            HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ApprovedForFutureIsolatedJoinTargetImplementationRegistration,
            expected_excluded_prior_actor_ids(&specification, &[]),
        );
        let second = sample_review(
            &specification,
            "reviewer-b",
            Some(&first),
            HistoricalOutcomeFeatureLabelJoinTargetSpecReviewVerdict::ChangesRequested,
            expected_excluded_prior_actor_ids(&specification, std::slice::from_ref(&first)),
        );
        assert!(validate_review_chain(&[first, second], &specification.specification_id).is_err());
    }
}
