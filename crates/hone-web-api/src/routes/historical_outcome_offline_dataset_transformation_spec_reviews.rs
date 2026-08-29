//! Independent review of one immutable offline-dataset transformation specification.
//!
//! Approval permits only later registration of an isolated transformation implementation.
//! This module never creates a split manifest or feature bundle, performs a join, assigns a
//! target, runs training or grants reward, shadow, order, broker or trading authority.

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

use super::historical_outcome_offline_dataset_transformation_specs::{
    HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
    historical_outcome_offline_dataset_transformation_spec_registry_for_reviews,
    validate_historical_outcome_offline_dataset_transformation_spec_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-spec-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-spec-independent-review-v1";
const POLICY_VERSION: &str =
    "hone-offline-dataset-transformation-spec-independent-review-v1-no-execution";
const REVIEW_CONTRACT_VERSION: &str =
    "hone-offline-dataset-transformation-spec-independent-semantic-audit-v1";
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;
const EXPECTED_FEATURE_COUNT: usize = 65;

const EXPECTED_FEATURES: &[&str] = &[
    "industry_state:demand_driver_state",
    "industry_state:effective_supply_state",
    "industry_state:supply_demand_gap_state",
    "industry_state:bottleneck_state",
    "industry_state:pricing_power_state",
    "industry_state:capacity_state",
    "industry_state:technology_milestone_state",
    "industry_state:industry_cycle_state",
    "company_fundamentals:business_model_quality",
    "company_fundamentals:moat_strength",
    "company_fundamentals:scarcity_strength",
    "company_fundamentals:differentiation_strength",
    "company_fundamentals:value_capture_strength",
    "company_fundamentals:market_share_state",
    "company_fundamentals:product_competitiveness",
    "company_fundamentals:customer_switching_cost",
    "company_fundamentals:customer_concentration_risk",
    "financial_state:revenue_growth_yoy",
    "financial_state:gross_margin",
    "financial_state:gross_margin_yoy_delta",
    "financial_state:operating_margin",
    "financial_state:free_cash_flow",
    "financial_state:free_cash_flow_margin",
    "financial_state:net_cash",
    "financial_state:net_debt",
    "financial_state:accounts_receivable_growth",
    "financial_state:inventory_growth",
    "financial_state:accounts_payable_growth",
    "financial_state:capital_expenditure",
    "financial_state:backlog",
    "financial_state:remaining_performance_obligation",
    "financial_state:annual_recurring_revenue",
    "valuation_state:current_price",
    "valuation_state:market_cap",
    "valuation_state:enterprise_value",
    "valuation_state:forward_pe",
    "valuation_state:ev_ebitda",
    "valuation_state:free_cash_flow_yield",
    "valuation_state:reverse_dcf_implied_growth",
    "valuation_state:bear_fair_value",
    "valuation_state:base_fair_value",
    "valuation_state:bull_fair_value",
    "valuation_state:current_price_to_base_value",
    "crowding_state:price_drawdown_from_high",
    "crowding_state:relative_strength",
    "crowding_state:valuation_percentile",
    "crowding_state:analyst_revision_state",
    "crowding_state:short_interest_state",
    "crowding_state:option_positioning_state",
    "crowding_state:narrative_crowding_state",
    "macro_state:ten_year_treasury_yield",
    "macro_state:thirty_year_treasury_yield",
    "macro_state:policy_rate",
    "macro_state:unemployment_rate",
    "macro_state:nonfarm_payroll_state",
    "macro_state:core_pce_state",
    "macro_state:vix",
    "macro_state:liquidity_state",
    "portfolio_context:position_weight",
    "portfolio_context:theme_weight",
    "portfolio_context:cash_weight",
    "portfolio_context:cost_basis_distance",
    "portfolio_context:thesis_status",
    "portfolio_context:risk_budget_remaining",
    "portfolio_context:holding_horizon",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict {
    ApprovedForFutureIsolatedTransformationImplementationRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecReviewContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub semantic_audit_implementation: String,
    pub required_split_checks: Vec<String>,
    pub required_feature_checks: Vec<String>,
    pub approval_scope: String,
    pub implementation_registration_separate: bool,
    pub transformation_execution_separate: bool,
    pub output_validation_separate: bool,
    pub target_definition_separate: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_transformation_spec_sha256: String,
    expected_transformation_body_sha256: String,
    expected_dataset_content_sha256: String,
    expected_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_governance_review_sha256: String,
    expected_split_specification_sha256: String,
    expected_feature_specification_sha256: String,
    expected_review_contract_sha256: String,
    verdict: HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_specification_binding_confirmed: bool,
    reviewer_independence_confirmed: bool,
    independent_hash_and_schema_reproduction_confirmed: bool,
    transitive_component_identity_and_indivisibility_confirmed: bool,
    chronological_contiguous_boundary_objective_confirmed: bool,
    equal_time_hash_tie_break_only_confirmed: bool,
    market_session_purge_embargo_and_empty_partition_failure_confirmed: bool,
    sealed_holdout_label_isolation_confirmed: bool,
    exact_seven_layer_feature_id_allowlist_confirmed: bool,
    point_in_time_artifact_and_revision_provenance_confirmed: bool,
    qualitative_market_and_portfolio_source_contracts_confirmed: bool,
    explicit_missingness_without_backfill_or_interpolation_confirmed: bool,
    outcome_label_future_and_namespace_smuggling_exclusion_confirmed: bool,
    content_addressed_create_once_outputs_and_later_validation_confirmed: bool,
    review_implementation_execution_target_training_separation_confirmed: bool,
    no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub specification: HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
    pub review_contract: HistoricalOutcomeOfflineDatasetTransformationSpecReviewContract,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub reviewer_independent_from_complete_registration_chain: bool,
    pub verdict: HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_current_specification_binding_confirmed: bool,
    pub reviewer_independence_confirmed: bool,
    pub independent_hash_and_schema_reproduction_confirmed: bool,
    pub transitive_component_identity_and_indivisibility_confirmed: bool,
    pub chronological_contiguous_boundary_objective_confirmed: bool,
    pub equal_time_hash_tie_break_only_confirmed: bool,
    pub market_session_purge_embargo_and_empty_partition_failure_confirmed: bool,
    pub sealed_holdout_label_isolation_confirmed: bool,
    pub exact_seven_layer_feature_id_allowlist_confirmed: bool,
    pub point_in_time_artifact_and_revision_provenance_confirmed: bool,
    pub qualitative_market_and_portfolio_source_contracts_confirmed: bool,
    pub explicit_missingness_without_backfill_or_interpolation_confirmed: bool,
    pub outcome_label_future_and_namespace_smuggling_exclusion_confirmed: bool,
    pub content_addressed_create_once_outputs_and_later_validation_confirmed: bool,
    pub review_implementation_execution_target_training_separation_confirmed: bool,
    pub no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub future_isolated_transformation_implementation_registration_eligible: bool,
    pub transformation_implementation_registered: bool,
    pub split_manifest_generation_authorized: bool,
    pub split_manifest_generated: bool,
    pub feature_bundle_generation_authorized: bool,
    pub feature_bundle_generated: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecReviewItem {
    pub specification: HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
    pub complete_review_actor_ids: Vec<String>,
    pub upstream_binding_current: bool,
    pub latest_review: Option<HistoricalOutcomeOfflineDatasetTransformationSpecReview>,
    pub review_eligible: bool,
    pub future_isolated_transformation_implementation_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_contract: HistoricalOutcomeOfflineDatasetTransformationSpecReviewContract,
    pub items: Vec<HistoricalOutcomeOfflineDatasetTransformationSpecReviewItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub implementation_registration_eligible_count: usize,
    pub review_status: String,
    pub transformation_implementation_registered: bool,
    pub split_manifest_generation_authorized: bool,
    pub split_manifest_generated: bool,
    pub feature_bundle_generation_authorized: bool,
    pub feature_bundle_generated: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationSpecReviewReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub implementation_registration_eligible_count: usize,
    pub review_status: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_spec_reviews(
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
            warn!(%error, "historical outcome transformation spec review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线转换规范独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_offline_dataset_transformation_spec(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(transformation_spec_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &transformation_spec_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome transformation spec review registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线转换规范独立复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_transformation_spec_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeOfflineDatasetTransformationSpecReviewReadinessSummary {
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

pub(crate) async fn approved_historical_outcome_offline_dataset_transformation_spec_reviews(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationSpecReview>, String> {
    let registry = build_registry(state).await?;
    let mut reviews = registry
        .items
        .into_iter()
        .filter(|item| item.future_isolated_transformation_implementation_registration_eligible)
        .filter_map(|item| item.latest_review)
        .collect::<Vec<_>>();
    for review in &reviews {
        validate_review(review)?;
    }
    reviews.sort_by(|left, right| {
        right
            .submitted_at
            .cmp(&left.submitted_at)
            .then_with(|| right.review_id.cmp(&left.review_id))
    });
    Ok(reviews)
}

pub(crate) fn validate_historical_outcome_offline_dataset_transformation_spec_review_for_implementation(
    review: &HistoricalOutcomeOfflineDatasetTransformationSpecReview,
) -> Result<(), String> {
    validate_review(review)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry, String> {
    let source =
        historical_outcome_offline_dataset_transformation_spec_registry_for_reviews(state).await?;
    let contract = review_contract();
    let mut items = Vec::with_capacity(source.items.len());
    for source_item in &source.items {
        let specification = &source_item.specification;
        let reviews = read_reviews_for_spec(state, &specification.transformation_spec_id).await?;
        let complete_review_actor_ids = reviews
            .iter()
            .map(|review| review.reviewer_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let latest_review = reviews.last().cloned();
        if latest_review
            .as_ref()
            .is_some_and(|review| review.specification != *specification)
        {
            return Err(
                "historical outcome transformation spec review binding changed".to_string(),
            );
        }
        let upstream_binding_current = source_item.upstream_binding_current
            && source_item.future_independent_spec_review_eligible;
        let approved = latest_review.as_ref().is_some_and(|review| {
            review.future_isolated_transformation_implementation_registration_eligible
        });
        items.push(
            HistoricalOutcomeOfflineDatasetTransformationSpecReviewItem {
                specification: specification.clone(),
                complete_review_actor_ids,
                upstream_binding_current,
                review_eligible: upstream_binding_current && !approved,
                future_isolated_transformation_implementation_registration_eligible:
                    upstream_binding_current && approved,
                latest_review,
            },
        );
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
                review.future_isolated_transformation_implementation_registration_eligible
            })
        })
        .count();
    let current_binding_approved_count = items
        .iter()
        .filter(|item| item.future_isolated_transformation_implementation_registration_eligible)
        .count();
    let implementation_registration_eligible_count = current_binding_approved_count;
    let review_status = if source.current_binding_registered_count == 0 {
        "waiting_current_transformation_specification"
    } else if current_binding_approved_count > 0 {
        "current_specification_approved_only_for_future_isolated_implementation_registration"
    } else if review_eligible_count > 0 {
        "current_specification_ready_for_independent_review"
    } else {
        "transformation_specification_review_blocked_fail_closed"
    };
    Ok(HistoricalOutcomeOfflineDatasetTransformationSpecReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_contract: contract,
        items,
        review_eligible_count,
        reviewed_count,
        approved_count,
        current_binding_approved_count,
        implementation_registration_eligible_count,
        review_status: review_status.to_string(),
        transformation_implementation_registered: false,
        split_manifest_generation_authorized: false,
        split_manifest_generated: false,
        feature_bundle_generation_authorized: false,
        feature_bundle_generated: false,
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
        scope: "第 26 阶段由独立角色复核一份当前不可变转换规范的确定性、防泄漏、点时来源、具体 feature ID 白名单和缺失语义。批准只允许未来登记隔离转换实现；本阶段不登记实现、不生成 manifest 或 bundle、不连接特征、不定义目标、不训练、不奖励、不建立影子组合、不生成订单，也不接入券商或交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    transformation_spec_id: &str,
    request: ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecReview, String> {
    if reviewer_id.trim().is_empty() {
        return Err("转换规范独立复核人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let source =
        historical_outcome_offline_dataset_transformation_spec_registry_for_reviews(state).await?;
    let source_item = source
        .items
        .iter()
        .find(|item| item.specification.transformation_spec_id == transformation_spec_id)
        .ok_or_else(|| "不可变转换规范不存在".to_string())?;
    if !source_item.upstream_binding_current || !source_item.future_independent_spec_review_eligible
    {
        return Err("只能复核当前完整上游绑定的不可变转换规范，请刷新后重试".to_string());
    }
    let reviews = read_reviews_for_spec(state, transformation_spec_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|review| {
        review.future_isolated_transformation_implementation_registration_eligible
    }) {
        return Err("当前转换规范已经独立批准，不得重复或分叉复核".to_string());
    }
    let review = build_review(
        &source_item.specification,
        prior,
        reviewer_id,
        request,
        Utc::now(),
    )?;
    write_immutable_json(
        &review_root(state)
            .join(transformation_spec_id)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn build_review(
    specification: &HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
    prior: Option<&HistoricalOutcomeOfflineDatasetTransformationSpecReview>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationSpecReview, String> {
    validate_historical_outcome_offline_dataset_transformation_spec_for_review(specification)?;
    independently_audit_specification(specification)?;
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let contract = review_contract();
    if request.expected_review_id != prior.map(|review| review.review_id.clone())
        || request.expected_review_sha256 != prior.map(|review| review.review_sha256.clone())
        || request.expected_transformation_spec_sha256 != specification.transformation_spec_sha256
        || request.expected_transformation_body_sha256 != specification.transformation_body_sha256
        || request.expected_dataset_content_sha256 != specification.subject.dataset_content_sha256
        || request.expected_manifest_sha256 != specification.subject.manifest_sha256
        || request.expected_candidate_set_sha256 != specification.subject.candidate_set_sha256
        || request.expected_governance_review_sha256 != specification.governance_review_sha256
        || request.expected_split_specification_sha256
            != specification
                .split_manifest_specification
                .specification_sha256
        || request.expected_feature_specification_sha256
            != specification
                .feature_bundle_specification
                .specification_sha256
        || request.expected_review_contract_sha256 != contract.contract_sha256
    {
        return Err("转换规范、治理链或独立复核合同绑定已变化，请刷新后重试".to_string());
    }
    if prior.is_some_and(|review| review.specification != *specification) {
        return Err("上一条复核绑定的转换规范与当前规范不一致".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(specification);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == reviewer_id)
    {
        return Err("复核人参与过数据集、治理或规范登记链，必须更换独立复核人".to_string());
    }
    let approval = request.verdict
        == HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict::ApprovedForFutureIsolatedTransformationImplementationRegistration;
    let all_confirmed = request.exact_current_specification_binding_confirmed
        && request.reviewer_independence_confirmed
        && request.independent_hash_and_schema_reproduction_confirmed
        && request.transitive_component_identity_and_indivisibility_confirmed
        && request.chronological_contiguous_boundary_objective_confirmed
        && request.equal_time_hash_tie_break_only_confirmed
        && request.market_session_purge_embargo_and_empty_partition_failure_confirmed
        && request.sealed_holdout_label_isolation_confirmed
        && request.exact_seven_layer_feature_id_allowlist_confirmed
        && request.point_in_time_artifact_and_revision_provenance_confirmed
        && request.qualitative_market_and_portfolio_source_contracts_confirmed
        && request.explicit_missingness_without_backfill_or_interpolation_confirmed
        && request.outcome_label_future_and_namespace_smuggling_exclusion_confirmed
        && request.content_addressed_create_once_outputs_and_later_validation_confirmed
        && request.review_implementation_execution_target_training_separation_confirmed
        && request.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed;
    if approval && !all_confirmed {
        return Err("批准转换规范前必须逐项确认确定性、防泄漏、点时来源和零执行边界".to_string());
    }
    let mut review = HistoricalOutcomeOfflineDatasetTransformationSpecReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: prior.map(|review| review.review_id.clone()),
        previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
        specification: specification.clone(),
        review_contract: contract,
        submitted_at,
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        reviewer_independent_from_complete_registration_chain: true,
        verdict: request.verdict,
        rationale,
        known_limitations,
        exact_current_specification_binding_confirmed: request
            .exact_current_specification_binding_confirmed,
        reviewer_independence_confirmed: request.reviewer_independence_confirmed,
        independent_hash_and_schema_reproduction_confirmed: request
            .independent_hash_and_schema_reproduction_confirmed,
        transitive_component_identity_and_indivisibility_confirmed: request
            .transitive_component_identity_and_indivisibility_confirmed,
        chronological_contiguous_boundary_objective_confirmed: request
            .chronological_contiguous_boundary_objective_confirmed,
        equal_time_hash_tie_break_only_confirmed: request
            .equal_time_hash_tie_break_only_confirmed,
        market_session_purge_embargo_and_empty_partition_failure_confirmed: request
            .market_session_purge_embargo_and_empty_partition_failure_confirmed,
        sealed_holdout_label_isolation_confirmed: request
            .sealed_holdout_label_isolation_confirmed,
        exact_seven_layer_feature_id_allowlist_confirmed: request
            .exact_seven_layer_feature_id_allowlist_confirmed,
        point_in_time_artifact_and_revision_provenance_confirmed: request
            .point_in_time_artifact_and_revision_provenance_confirmed,
        qualitative_market_and_portfolio_source_contracts_confirmed: request
            .qualitative_market_and_portfolio_source_contracts_confirmed,
        explicit_missingness_without_backfill_or_interpolation_confirmed: request
            .explicit_missingness_without_backfill_or_interpolation_confirmed,
        outcome_label_future_and_namespace_smuggling_exclusion_confirmed: request
            .outcome_label_future_and_namespace_smuggling_exclusion_confirmed,
        content_addressed_create_once_outputs_and_later_validation_confirmed: request
            .content_addressed_create_once_outputs_and_later_validation_confirmed,
        review_implementation_execution_target_training_separation_confirmed: request
            .review_implementation_execution_target_training_separation_confirmed,
        no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
            request.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed,
        future_isolated_transformation_implementation_registration_eligible: approval,
        transformation_implementation_registered: false,
        split_manifest_generation_authorized: false,
        split_manifest_generated: false,
        feature_bundle_generation_authorized: false,
        feature_bundle_generated: false,
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

fn independently_audit_specification(
    specification: &HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
) -> Result<(), String> {
    let split = &specification.split_manifest_specification;
    let features = &specification.feature_bundle_specification;
    let split_ok = split.schema_version
        == "hone-deterministic-connected-component-split-manifest-spec-v2-exact-boundaries"
        && split.train_percent == 70
        && split.validation_percent == 15
        && split.sealed_holdout_percent == 15
        && split.purge_embargo_market_sessions == 250
        && split.max_outcome_horizon_market_sessions == 250
        && split
            .component_construction_algorithm
            .contains("union_find")
        && split
            .chronological_order_algorithm
            .starts_with("ascending_component_latest")
        && split
            .boundary_assignment_algorithm
            .contains("enumerate_all")
        && split
            .boundary_objective
            .contains("lexicographically_minimize")
        && split
            .minimum_partition_rule
            .contains("at_least_three_connected_components")
        && split
            .market_session_calendar_rule
            .contains("common_asset_and_spy_market_session")
        && split
            .purge_embargo_algorithm
            .contains("outcome_window_reaches_or_crosses")
        && split
            .empty_partition_after_purge_policy
            .starts_with("fail_closed")
        && split.sealed_holdout_labels_withheld_from_training_worker
        && split.content_addressed_output_required
        && split.create_once_output_required
        && !split.split_assignments_generated;
    let expected_namespaces = vec![
        "industry_state",
        "company_fundamentals",
        "financial_state",
        "valuation_state",
        "crowding_state",
        "macro_state",
        "portfolio_context",
    ];
    let actual_feature_ids = features
        .allowed_features
        .iter()
        .map(|feature| format!("{}:{}", feature.namespace, feature.feature_id))
        .collect::<BTreeSet<_>>();
    let expected_feature_ids = EXPECTED_FEATURES
        .iter()
        .map(|value| (*value).to_string())
        .collect::<BTreeSet<_>>();
    let required_record_fields = [
        "artifact_sha256",
        "source_identity",
        "source_version",
        "observed_at_utc",
        "available_at_utc",
        "is_missing",
        "missingness_reason",
        "accounting_basis",
        "measurement_scope",
    ];
    let forbidden = [
        "raw_validated_metrics",
        "outcome",
        "formal_label",
        "validation",
        "admission",
        "offline_dataset",
        "future_market_data",
        "split_assignment",
    ];
    let features_ok = features.schema_version
        == "hone-point-in-time-seven-layer-feature-bundle-spec-v2-feature-id-allowlist"
        && features.allowed_feature_namespaces == expected_namespaces
        && features.allowed_features.len() == EXPECTED_FEATURE_COUNT
        && actual_feature_ids == expected_feature_ids
        && features.feature_id_must_be_allowlisted
        && features.namespace_cannot_override_feature_semantics
        && required_record_fields.iter().all(|field| {
            features
                .required_feature_record_fields
                .iter()
                .any(|actual| actual == field)
        })
        && forbidden.iter().all(|namespace| {
            features
                .forbidden_feature_namespaces
                .iter()
                .any(|actual| actual == namespace)
        })
        && features.availability_rule.contains("available_at_utc <=")
        && features
            .artifact_revision_policy
            .contains("later_restated_or_revised_versions_are_forbidden")
        && features
            .qualitative_feature_review_policy
            .contains("human_reviewed_evidence_chain")
        && features
            .market_snapshot_policy
            .contains("content_addressed_point_in_time_market_snapshot")
        && features
            .portfolio_snapshot_policy
            .contains("must_not_use_current_holdings")
        && features.missingness_values.len() == 4
        && !features.backfill_allowed
        && !features.interpolation_allowed
        && features.content_addressed_output_required
        && features.create_once_output_required
        && !features.feature_bundle_generated
        && !features.feature_join_performed;
    if !split_ok || !features_ok {
        return Err(
            "independent transformation specification semantic audit failed closed".to_string(),
        );
    }
    Ok(())
}

fn review_contract() -> HistoricalOutcomeOfflineDatasetTransformationSpecReviewContract {
    let mut contract = HistoricalOutcomeOfflineDatasetTransformationSpecReviewContract {
        schema_version: REVIEW_CONTRACT_VERSION.to_string(),
        contract_sha256: String::new(),
        semantic_audit_implementation:
            "hone-independent-transformation-spec-semantic-audit-v1-no-registration-generator_reuse"
                .to_string(),
        required_split_checks: vec![
            "transitive_company_event_source_components_are_indivisible".to_string(),
            "chronological_contiguous_boundary_pair_enumeration_has_exact_integer_objective_and_tie_break"
                .to_string(),
            "common_market_session_250_horizon_purge_and_embargo_fail_closed".to_string(),
            "sealed_holdout_labels_are_inaccessible_to_training_worker".to_string(),
            "manifest_output_is_create_once_content_addressed_and_separately_validated"
                .to_string(),
        ],
        required_feature_checks: vec![
            "exact_seven_namespace_and_65_feature_id_allowlist".to_string(),
            "artifact_source_version_observation_and_availability_provenance".to_string(),
            "historical_artifact_revision_only_no_later_restatement".to_string(),
            "qualitative_market_and_portfolio_features_have_specific_source_contracts"
                .to_string(),
            "explicit_missingness_no_backfill_or_interpolation".to_string(),
            "outcome_label_future_and_namespace_smuggling_fail_closed".to_string(),
        ],
        approval_scope:
            "future_isolated_transformation_implementation_registration_only_no_execution"
                .to_string(),
        implementation_registration_separate: true,
        transformation_execution_separate: true,
        output_validation_separate: true,
        target_definition_separate: true,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static transformation spec review contract is serializable");
    contract
}

fn expected_excluded_prior_actor_ids(
    specification: &HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
) -> Vec<String> {
    let mut actors = specification.excluded_prior_actor_ids.clone();
    actors.push(specification.registered_by.clone());
    actors.push(specification.governance_reviewer_id.clone());
    actors.sort();
    actors.dedup();
    actors
}

fn validate_review(
    review: &HistoricalOutcomeOfflineDatasetTransformationSpecReview,
) -> Result<(), String> {
    validate_historical_outcome_offline_dataset_transformation_spec_for_review(
        &review.specification,
    )?;
    independently_audit_specification(&review.specification)?;
    let approval = review.verdict
        == HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict::ApprovedForFutureIsolatedTransformationImplementationRegistration;
    let all_confirmed = review.exact_current_specification_binding_confirmed
        && review.reviewer_independence_confirmed
        && review.independent_hash_and_schema_reproduction_confirmed
        && review.transitive_component_identity_and_indivisibility_confirmed
        && review.chronological_contiguous_boundary_objective_confirmed
        && review.equal_time_hash_tie_break_only_confirmed
        && review.market_session_purge_embargo_and_empty_partition_failure_confirmed
        && review.sealed_holdout_label_isolation_confirmed
        && review.exact_seven_layer_feature_id_allowlist_confirmed
        && review.point_in_time_artifact_and_revision_provenance_confirmed
        && review.qualitative_market_and_portfolio_source_contracts_confirmed
        && review.explicit_missingness_without_backfill_or_interpolation_confirmed
        && review.outcome_label_future_and_namespace_smuggling_exclusion_confirmed
        && review.content_addressed_create_once_outputs_and_later_validation_confirmed
        && review.review_implementation_execution_target_training_separation_confirmed
        && review.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !review.transformation_implementation_registered
        && !review.split_manifest_generation_authorized
        && !review.split_manifest_generated
        && !review.feature_bundle_generation_authorized
        && !review.feature_bundle_generated
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
        || review.review_contract != review_contract()
        || review.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&review.specification)
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review.reviewer_id.trim().is_empty()
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || !review.reviewer_independent_from_complete_registration_chain
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || review.known_limitations.trim().is_empty()
        || review.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || review.future_isolated_transformation_implementation_registration_eligible
            != (approval && all_confirmed)
        || !authority_closed
    {
        return Err(
            "historical outcome transformation specification review is invalid or over-authorized"
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
        return Err("transformation specification review predecessor is invalid".to_string());
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[HistoricalOutcomeOfflineDatasetTransformationSpecReview],
    transformation_spec_id: &str,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationSpecReview>, String> {
    if reviews.is_empty() {
        return Ok(Vec::new());
    }
    let mut by_id = HashMap::new();
    let mut child_by_parent = HashMap::<String, String>::new();
    let mut roots = Vec::new();
    for review in reviews {
        validate_review(review)?;
        if review.specification.transformation_spec_id != transformation_spec_id
            || by_id
                .insert(review.review_id.clone(), review.clone())
                .is_some()
        {
            return Err(
                "transformation specification review replay or subject mismatch".to_string(),
            );
        }
        match &review.previous_review_id {
            Some(parent) => {
                if child_by_parent
                    .insert(parent.clone(), review.review_id.clone())
                    .is_some()
                {
                    return Err("transformation specification review chain branches".to_string());
                }
            }
            None => roots.push(review.review_id.clone()),
        }
    }
    if roots.len() != 1 {
        return Err("transformation specification review chain has no single root".to_string());
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = roots.pop().unwrap();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err("transformation specification review chain contains a cycle".to_string());
        }
        let review = by_id
            .get(&current)
            .ok_or_else(|| "transformation specification review chain is disconnected".to_string())?
            .clone();
        if let Some(parent_id) = &review.previous_review_id {
            let parent = by_id.get(parent_id).ok_or_else(|| {
                "transformation specification review predecessor is missing".to_string()
            })?;
            if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                return Err(
                    "transformation specification review predecessor hash drift".to_string()
                );
            }
        }
        ordered.push(review);
        match child_by_parent.get(&current) {
            Some(next) => current = next.clone(),
            None => break,
        }
    }
    if ordered.len() != reviews.len() {
        return Err("transformation specification review chain is disconnected".to_string());
    }
    Ok(ordered)
}

async fn read_reviews_for_spec(
    state: &AppState,
    transformation_spec_id: &str,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationSpecReview>, String> {
    let root = review_root(state).join(transformation_spec_id);
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
        let review: HistoricalOutcomeOfflineDatasetTransformationSpecReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("transformation specification review path mismatch".to_string());
        }
        reviews.push(review);
    }
    validate_review_chain(&reviews, transformation_spec_id)
}

fn review_fingerprint(
    review: &HistoricalOutcomeOfflineDatasetTransformationSpecReview,
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
    decision_root(state)
        .join("historical_outcome_offline_dataset_transformation_spec_reviews/reviews")
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
    let path = directory.join("historical-outcome-offline-dataset-transformation-spec-review.lock");
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
                return Err("另一个管理员正在复核离线转换规范".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("离线转换规范复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_governance::HistoricalOutcomeOfflineDatasetGovernanceSubject;
    use crate::routes::historical_outcome_offline_dataset_transformation_specs::{
        DeterministicSplitManifestSpecification, PointInTimeFeatureBundleSpecification,
        PointInTimeFeatureDefinition,
    };

    fn feature_catalog() -> Vec<PointInTimeFeatureDefinition> {
        EXPECTED_FEATURES
            .iter()
            .map(|value| {
                let (namespace, feature_id) = value.split_once(':').unwrap();
                PointInTimeFeatureDefinition {
                    namespace: namespace.to_string(),
                    feature_id: feature_id.to_string(),
                    value_kind: "verified".to_string(),
                    source_authority_contract: "traceable".to_string(),
                }
            })
            .collect()
    }

    pub(crate) fn specification() -> HistoricalOutcomeOfflineDatasetTransformationSpecRecord {
        let subject = HistoricalOutcomeOfflineDatasetGovernanceSubject {
            dataset_id: "a".repeat(32),
            dataset_version: "v1".to_string(),
            version_number: 1,
            dataset_content_sha256: "b".repeat(64),
            manifest_sha256: "c".repeat(64),
            candidate_set_sha256: "d".repeat(64),
            entry_count: 30,
            distinct_symbol_count: 10,
            earliest_decision_available_at: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            latest_decision_available_at: DateTime::from_timestamp(1_710_000_000, 0).unwrap(),
            assembled_at: DateTime::from_timestamp(1_720_000_000, 0).unwrap(),
            assembled_by: "assembler".to_string(),
            complete_actor_ids: vec!["assembler".to_string(), "writer".to_string()],
            distinct_reconstruction_count: 10,
            distinct_snapshot_count: 10,
            raw_outcome_only: true,
            split_assigned: false,
            feature_join_performed: false,
            semantic_targets_assigned: false,
        };
        let split_manifest_specification = DeterministicSplitManifestSpecification {
            schema_version:
                "hone-deterministic-connected-component-split-manifest-spec-v2-exact-boundaries"
                    .to_string(),
            specification_sha256: "e".repeat(64),
            governed_split_policy_version: "split-policy".to_string(),
            governed_split_policy_sha256: "f".repeat(64),
            component_identity_fields: vec!["asset_symbol".to_string()],
            component_construction_algorithm: "deterministic_union_find".to_string(),
            component_identity_algorithm: "sha256".to_string(),
            chronological_order_algorithm: "ascending_component_latest_then_earliest_then_sha256"
                .to_string(),
            boundary_assignment_algorithm: "enumerate_all_boundary_pairs".to_string(),
            boundary_objective: "lexicographically_minimize_exact_integer_deviation".to_string(),
            minimum_partition_rule: "at_least_three_connected_components_then_fail_closed"
                .to_string(),
            market_session_calendar_rule: "common_asset_and_spy_market_session_index".to_string(),
            purge_embargo_algorithm:
                "outcome_window_reaches_or_crosses_transition_and_250_session_embargo".to_string(),
            empty_partition_after_purge_policy: "fail_closed".to_string(),
            train_percent: 70,
            validation_percent: 15,
            sealed_holdout_percent: 15,
            purge_embargo_market_sessions: 250,
            max_outcome_horizon_market_sessions: 250,
            output_manifest_fields: vec!["split".to_string()],
            sealed_holdout_labels_withheld_from_training_worker: true,
            content_addressed_output_required: true,
            create_once_output_required: true,
            split_assignments_generated: false,
        };
        let feature_bundle_specification = PointInTimeFeatureBundleSpecification {
            schema_version:
                "hone-point-in-time-seven-layer-feature-bundle-spec-v2-feature-id-allowlist"
                    .to_string(),
            specification_sha256: "1".repeat(64),
            governed_feature_join_policy_version: "feature-policy".to_string(),
            governed_feature_join_policy_sha256: "2".repeat(64),
            join_key: "entry+time".to_string(),
            allowed_feature_namespaces: vec![
                "industry_state".to_string(),
                "company_fundamentals".to_string(),
                "financial_state".to_string(),
                "valuation_state".to_string(),
                "crowding_state".to_string(),
                "macro_state".to_string(),
                "portfolio_context".to_string(),
            ],
            allowed_features: feature_catalog(),
            feature_id_must_be_allowlisted: true,
            namespace_cannot_override_feature_semantics: true,
            required_feature_record_fields: vec![
                "artifact_sha256".to_string(),
                "source_identity".to_string(),
                "source_version".to_string(),
                "observed_at_utc".to_string(),
                "available_at_utc".to_string(),
                "is_missing".to_string(),
                "missingness_reason".to_string(),
                "accounting_basis".to_string(),
                "measurement_scope".to_string(),
            ],
            availability_rule: "available_at_utc <= decision_available_at_utc".to_string(),
            observation_time_rule: "observed_not_available".to_string(),
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
            missingness_values: vec![
                "not_available_at_decision_time".to_string(),
                "source_missing".to_string(),
                "availability_ambiguous".to_string(),
                "not_applicable".to_string(),
            ],
            missing_or_ambiguous_availability_policy: "fail_closed".to_string(),
            artifact_revision_policy: "later_restated_or_revised_versions_are_forbidden"
                .to_string(),
            qualitative_feature_review_policy: "human_reviewed_evidence_chain".to_string(),
            market_snapshot_policy: "content_addressed_point_in_time_market_snapshot".to_string(),
            portfolio_snapshot_policy: "must_not_use_current_holdings".to_string(),
            output_bundle_fields: vec!["bundle_sha256".to_string()],
            backfill_allowed: false,
            interpolation_allowed: false,
            content_addressed_output_required: true,
            create_once_output_required: true,
            feature_bundle_generated: false,
            feature_join_performed: false,
        };
        let transformation_body_sha256 = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&serde_json::json!({
                    "split_manifest_specification": &split_manifest_specification,
                    "feature_bundle_specification": &feature_bundle_specification,
                }))
                .unwrap()
            )
        );
        let mut record = HistoricalOutcomeOfflineDatasetTransformationSpecRecord {
            schema_version: "hone-historical-outcome-offline-dataset-transformation-spec-v1"
                .to_string(),
            policy_version: "hone-offline-dataset-transformation-spec-registration-v1-no-execution"
                .to_string(),
            transformation_spec_id: String::new(),
            transformation_spec_sha256: String::new(),
            registered_at: DateTime::from_timestamp(1_730_000_000, 0).unwrap(),
            registered_by: "registrar".to_string(),
            subject,
            governance_review_id: "3".repeat(32),
            governance_review_sha256: "4".repeat(64),
            governance_reviewer_id: "governance-reviewer".to_string(),
            governance_known_limitations: "small sample".to_string(),
            excluded_prior_actor_ids: vec![
                "assembler".to_string(),
                "governance-reviewer".to_string(),
                "writer".to_string(),
            ],
            registrar_independent_from_dataset_and_governance_chain: true,
            specification_name: "spec".to_string(),
            code_revision: "git:abc".to_string(),
            rationale: "freeze".to_string(),
            known_limitations: "small sample".to_string(),
            split_manifest_specification,
            feature_bundle_specification,
            transformation_body_sha256,
            status: "registered_not_independently_reviewed_not_run".to_string(),
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
        record.transformation_spec_sha256 = super::fingerprint_without(
            &record,
            &["transformation_spec_id", "transformation_spec_sha256"],
        )
        .unwrap();
        record.transformation_spec_id = record.transformation_spec_sha256[..32].to_string();
        record
    }

    fn request(
        specification: &HistoricalOutcomeOfflineDatasetTransformationSpecRecord,
        prior: Option<&HistoricalOutcomeOfflineDatasetTransformationSpecReview>,
    ) -> ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest {
        ReviewHistoricalOutcomeOfflineDatasetTransformationSpecRequest {
            expected_review_id: prior.map(|review| review.review_id.clone()),
            expected_review_sha256: prior.map(|review| review.review_sha256.clone()),
            expected_transformation_spec_sha256: specification
                .transformation_spec_sha256
                .clone(),
            expected_transformation_body_sha256: specification
                .transformation_body_sha256
                .clone(),
            expected_dataset_content_sha256: specification.subject.dataset_content_sha256.clone(),
            expected_manifest_sha256: specification.subject.manifest_sha256.clone(),
            expected_candidate_set_sha256: specification.subject.candidate_set_sha256.clone(),
            expected_governance_review_sha256: specification.governance_review_sha256.clone(),
            expected_split_specification_sha256: specification
                .split_manifest_specification
                .specification_sha256
                .clone(),
            expected_feature_specification_sha256: specification
                .feature_bundle_specification
                .specification_sha256
                .clone(),
            expected_review_contract_sha256: review_contract().contract_sha256,
            verdict: HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict::ApprovedForFutureIsolatedTransformationImplementationRegistration,
            rationale: "independently reproduced every contract".to_string(),
            known_limitations: "sample and component balance remain unproven".to_string(),
            exact_current_specification_binding_confirmed: true,
            reviewer_independence_confirmed: true,
            independent_hash_and_schema_reproduction_confirmed: true,
            transitive_component_identity_and_indivisibility_confirmed: true,
            chronological_contiguous_boundary_objective_confirmed: true,
            equal_time_hash_tie_break_only_confirmed: true,
            market_session_purge_embargo_and_empty_partition_failure_confirmed: true,
            sealed_holdout_label_isolation_confirmed: true,
            exact_seven_layer_feature_id_allowlist_confirmed: true,
            point_in_time_artifact_and_revision_provenance_confirmed: true,
            qualitative_market_and_portfolio_source_contracts_confirmed: true,
            explicit_missingness_without_backfill_or_interpolation_confirmed: true,
            outcome_label_future_and_namespace_smuggling_exclusion_confirmed: true,
            content_addressed_create_once_outputs_and_later_validation_confirmed: true,
            review_implementation_execution_target_training_separation_confirmed: true,
            no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn approved_review() -> HistoricalOutcomeOfflineDatasetTransformationSpecReview {
        let spec = specification();
        build_review(
            &spec,
            None,
            "transformation-spec-reviewer",
            request(&spec, None),
            DateTime::from_timestamp(1_740_000_000, 0).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn independent_audit_requires_exact_boundaries_and_all_sixty_five_features() {
        let mut spec = specification();
        assert!(independently_audit_specification(&spec).is_ok());
        spec.feature_bundle_specification.allowed_features.pop();
        assert!(independently_audit_specification(&spec).is_err());
        let mut spec = specification();
        spec.split_manifest_specification.boundary_objective = "nearest".to_string();
        assert!(independently_audit_specification(&spec).is_err());
    }

    #[test]
    fn reviewer_must_be_outside_complete_registration_chain() {
        let spec = specification();
        assert!(build_review(&spec, None, "registrar", request(&spec, None), Utc::now()).is_err());
        assert!(
            build_review(
                &spec,
                None,
                "governance-reviewer",
                request(&spec, None),
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn approval_requires_every_confirmation_and_exact_binding() {
        let spec = specification();
        let mut missing = request(&spec, None);
        missing.exact_seven_layer_feature_id_allowlist_confirmed = false;
        assert!(build_review(&spec, None, "reviewer", missing, Utc::now()).is_err());
        let mut stale = request(&spec, None);
        stale.expected_feature_specification_sha256 = "0".repeat(64);
        assert!(build_review(&spec, None, "reviewer", stale, Utc::now()).is_err());
    }

    #[test]
    fn approved_review_only_allows_future_implementation_registration() {
        let spec = specification();
        let review =
            build_review(&spec, None, "reviewer", request(&spec, None), Utc::now()).unwrap();
        assert!(review.future_isolated_transformation_implementation_registration_eligible);
        assert!(!review.transformation_implementation_registered);
        assert!(!review.split_manifest_generation_authorized);
        assert!(!review.feature_bundle_generation_authorized);
        assert!(!review.training_authorized);
        assert!(!review.reward_authorized);
        assert!(!review.shadow_portfolio_authorized);
        assert!(!review.order_generation_authorized);
        assert!(!review.broker_access_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn one_bit_or_authority_tamper_fails_validation() {
        let spec = specification();
        let mut review =
            build_review(&spec, None, "reviewer", request(&spec, None), Utc::now()).unwrap();
        review.split_manifest_generation_authorized = true;
        assert!(validate_review(&review).is_err());
    }

    #[test]
    fn append_only_chain_rejects_branch_or_hash_drift() {
        let spec = specification();
        let mut first_request = request(&spec, None);
        first_request.verdict =
            HistoricalOutcomeOfflineDatasetTransformationSpecReviewVerdict::ChangesRequested;
        let first = build_review(
            &spec,
            None,
            "reviewer-a",
            first_request,
            DateTime::from_timestamp(1_740_000_000, 0).unwrap(),
        )
        .unwrap();
        let second = build_review(
            &spec,
            Some(&first),
            "reviewer-b",
            request(&spec, Some(&first)),
            DateTime::from_timestamp(1_740_000_100, 0).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validate_review_chain(
                &[second.clone(), first.clone()],
                &spec.transformation_spec_id
            )
            .unwrap()
            .len(),
            2
        );
        let mut branch = second.clone();
        branch.review_id = "9".repeat(32);
        branch.review_sha256 = "9".repeat(64);
        assert!(
            validate_review_chain(&[first, second, branch], &spec.transformation_spec_id).is_err()
        );
    }

    #[test]
    fn review_contract_is_content_bound_and_execution_separate() {
        let contract = review_contract();
        assert!(contract.implementation_registration_separate);
        assert!(contract.transformation_execution_separate);
        assert!(contract.output_validation_separate);
        assert!(contract.target_definition_separate);
        assert!(valid_sha256(&contract.contract_sha256));
    }
}
