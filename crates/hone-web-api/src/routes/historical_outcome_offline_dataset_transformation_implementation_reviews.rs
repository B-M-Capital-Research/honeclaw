//! Independent review of one immutable isolated offline-dataset transformation implementation.
//!
//! Approval permits only future registration of a separate isolated transformation runner
//! specification. This module has no callable implementation entrypoint and cannot execute a
//! transformation, generate a manifest or feature bundle, join features, assign a target, train,
//! reward, shadow, generate orders, access a broker or trade.

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

use super::historical_outcome_offline_dataset_transformation_implementations::{
    HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
    independent_review_eligible_historical_outcome_offline_dataset_transformation_implementations,
    validate_historical_outcome_offline_dataset_transformation_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-implementation-independent-review-v1";
const POLICY_VERSION: &str =
    "hone-offline-dataset-transformation-implementation-independent-review-v1-no-execution";
const REVIEW_CONTRACT_VERSION: &str =
    "hone-offline-dataset-transformation-implementation-independent-audit-v1";
const EXPECTED_SPLIT_IMPLEMENTATION_ID: &str =
    "hone-deterministic-connected-component-boundary-enumerator";
const EXPECTED_SPLIT_IMPLEMENTATION_VERSION: &str = "v1-exact-integer-objective";
const EXPECTED_FEATURE_IMPLEMENTATION_ID: &str = "hone-point-in-time-feature-allowlist-extractor";
const EXPECTED_FEATURE_IMPLEMENTATION_VERSION: &str = "v1-exact-65-feature-ids";
const EXPECTED_SERIALIZER_VERSION: &str = "hone-canonical-json-serializer-v1";
const EXPECTED_INPUT_SCHEMA_VERSION: &str = "hone-sealed-outcome-transformation-input-v1";
const EXPECTED_OUTPUT_SCHEMA_VERSION: &str = "hone-unmaterialized-transformation-output-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict {
    ApprovedForFutureIsolatedTransformationRunnerRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationReviewContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub independent_audit_implementation: String,
    pub required_artifact_checks: Vec<String>,
    pub required_sandbox_checks: Vec<String>,
    pub approval_scope: String,
    pub runner_registration_separate: bool,
    pub execution_authorization_separate: bool,
    pub transformation_execution_separate: bool,
    pub output_validation_separate: bool,
    pub target_definition_separate: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_artifact_sha256: String,
    expected_immutable_code_revision: String,
    expected_specification_review_sha256: String,
    expected_transformation_spec_sha256: String,
    expected_transformation_body_sha256: String,
    expected_split_specification_sha256: String,
    expected_feature_specification_sha256: String,
    expected_dataset_content_sha256: String,
    expected_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_governance_review_sha256: String,
    expected_review_contract_sha256: String,
    verdict: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_implementation_and_upstream_binding_confirmed: bool,
    reviewer_independence_confirmed: bool,
    artifact_digest_independently_reproduced_confirmed: bool,
    immutable_code_revision_reproducible_confirmed: bool,
    deterministic_split_implementation_matches_specification_confirmed: bool,
    exact_65_feature_implementation_matches_allowlist_confirmed: bool,
    canonical_serializer_and_schema_determinism_confirmed: bool,
    sealed_read_only_input_and_create_once_output_contract_confirmed: bool,
    bounded_resource_contract_confirmed: bool,
    no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
        bool,
    review_runner_execution_output_target_and_training_separation_confirmed: bool,
    no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
    pub review_contract: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewContract,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub reviewer_independent_from_complete_registration_chain: bool,
    pub verdict: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_current_implementation_and_upstream_binding_confirmed: bool,
    pub reviewer_independence_confirmed: bool,
    pub artifact_digest_independently_reproduced_confirmed: bool,
    pub immutable_code_revision_reproducible_confirmed: bool,
    pub deterministic_split_implementation_matches_specification_confirmed: bool,
    pub exact_65_feature_implementation_matches_allowlist_confirmed: bool,
    pub canonical_serializer_and_schema_determinism_confirmed: bool,
    pub sealed_read_only_input_and_create_once_output_contract_confirmed: bool,
    pub bounded_resource_contract_confirmed: bool,
    pub no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
        bool,
    pub review_runner_execution_output_target_and_training_separation_confirmed: bool,
    pub no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub future_isolated_transformation_runner_registration_eligible: bool,
    pub transformation_runner_registered: bool,
    pub transformation_execution_authorized: bool,
    pub transformation_execution_started: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationReviewItem {
    pub implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
    pub complete_review_actor_ids: Vec<String>,
    pub upstream_binding_current: bool,
    pub latest_review: Option<HistoricalOutcomeOfflineDatasetTransformationImplementationReview>,
    pub review_eligible: bool,
    pub future_isolated_transformation_runner_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeOfflineDatasetTransformationImplementationReview {
    pub implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
    pub review: HistoricalOutcomeOfflineDatasetTransformationImplementationReview,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_contract: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewContract,
    pub items: Vec<HistoricalOutcomeOfflineDatasetTransformationImplementationReviewItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub runner_registration_eligible_count: usize,
    pub review_status: String,
    pub transformation_runner_registered: bool,
    pub transformation_execution_authorized: bool,
    pub transformation_execution_started: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationReviewReadinessSummary
{
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub runner_registration_eligible_count: usize,
    pub review_status: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_implementation_reviews(
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
            warn!(%error, "historical outcome transformation implementation review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果隔离转换实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_offline_dataset_transformation_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &implementation_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome transformation implementation review registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果隔离转换实现独立复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_transformation_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationImplementationReviewReadinessSummary, String>
{
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeOfflineDatasetTransformationImplementationReviewReadinessSummary {
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            approved_count: registry.approved_count,
            current_binding_approved_count: registry.current_binding_approved_count,
            runner_registration_eligible_count: registry.runner_registration_eligible_count,
            review_status: registry.review_status,
        },
    )
}

pub(crate) async fn approved_historical_outcome_offline_dataset_transformation_implementation_reviews(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeOfflineDatasetTransformationImplementationReview>, String>
{
    let registry = build_registry(state).await?;
    let mut approved = registry
        .items
        .into_iter()
        .filter(|item| item.future_isolated_transformation_runner_registration_eligible)
        .filter_map(|item| {
            item.latest_review.map(|review| {
                ApprovedHistoricalOutcomeOfflineDatasetTransformationImplementationReview {
                    implementation: item.implementation,
                    review,
                }
            })
        })
        .collect::<Vec<_>>();
    approved.sort_by(|left, right| {
        right
            .review
            .submitted_at
            .cmp(&left.review.submitted_at)
            .then_with(|| right.review.review_id.cmp(&left.review.review_id))
    });
    Ok(approved)
}

pub(crate) fn validate_historical_outcome_offline_dataset_transformation_implementation_review_for_runner_registration(
    review: &HistoricalOutcomeOfflineDatasetTransformationImplementationReview,
) -> Result<(), String> {
    validate_review(review)?;
    if !review.future_isolated_transformation_runner_registration_eligible {
        return Err(
            "transformation implementation review is not approved for runner registration"
                .to_string(),
        );
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry, String> {
    let implementations =
        independent_review_eligible_historical_outcome_offline_dataset_transformation_implementations(
            state,
        )
        .await?;
    let mut items = Vec::with_capacity(implementations.len());
    for implementation in implementations {
        let reviews =
            read_reviews_for_implementation(state, &implementation.implementation_id).await?;
        let complete_review_actor_ids = reviews
            .iter()
            .map(|review| review.reviewer_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let latest_review = reviews.last().cloned();
        if latest_review
            .as_ref()
            .is_some_and(|review| review.implementation != implementation)
        {
            return Err(
                "historical outcome transformation implementation review binding changed"
                    .to_string(),
            );
        }
        let approved = latest_review.as_ref().is_some_and(|review| {
            review.future_isolated_transformation_runner_registration_eligible
        });
        items.push(
            HistoricalOutcomeOfflineDatasetTransformationImplementationReviewItem {
                implementation,
                complete_review_actor_ids,
                upstream_binding_current: true,
                review_eligible: !approved,
                future_isolated_transformation_runner_registration_eligible: approved,
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
                review.future_isolated_transformation_runner_registration_eligible
            })
        })
        .count();
    let current_binding_approved_count = items
        .iter()
        .filter(|item| item.future_isolated_transformation_runner_registration_eligible)
        .count();
    let runner_registration_eligible_count = current_binding_approved_count;
    let review_status = if items.is_empty() {
        "waiting_current_registered_transformation_implementation"
    } else if current_binding_approved_count > 0 {
        "current_implementation_approved_only_for_future_isolated_runner_registration"
    } else if review_eligible_count > 0 {
        "current_implementation_ready_for_independent_review"
    } else {
        "transformation_implementation_review_blocked_fail_closed"
    };
    Ok(HistoricalOutcomeOfflineDatasetTransformationImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_contract: review_contract(),
        items,
        review_eligible_count,
        reviewed_count,
        approved_count,
        current_binding_approved_count,
        runner_registration_eligible_count,
        review_status: review_status.to_string(),
        transformation_runner_registered: false,
        transformation_execution_authorized: false,
        transformation_execution_started: false,
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
        scope: "第 28 阶段由独立角色复核一份当前 registered_not_run 隔离转换实现的工件、代码、确定性算法、序列化/schema 和零能力沙箱。批准只允许未来登记隔离转换 runner 规范；本阶段不登记 runner、不执行、不生成 manifest/bundle、不 join、不写目标、不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationImplementationReview, String> {
    if reviewer_id.trim().is_empty() {
        return Err("隔离转换实现独立复核人无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementations =
        independent_review_eligible_historical_outcome_offline_dataset_transformation_implementations(
            state,
        )
        .await?;
    let implementation = implementations
        .iter()
        .find(|value| value.implementation_id == implementation_id)
        .ok_or_else(|| "当前隔离转换实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews_for_implementation(state, implementation_id).await?;
    let prior = reviews.last();
    if prior
        .is_some_and(|review| review.future_isolated_transformation_runner_registration_eligible)
    {
        return Err("当前隔离转换实现已经独立批准，不得重复或分叉复核".to_string());
    }
    let review = build_review(implementation, prior, reviewer_id, request, Utc::now())?;
    write_immutable_json(
        &review_root(state)
            .join(implementation_id)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn build_review(
    implementation: &HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
    prior: Option<&HistoricalOutcomeOfflineDatasetTransformationImplementationReview>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationImplementationReview, String> {
    validate_historical_outcome_offline_dataset_transformation_implementation_for_review(
        implementation,
    )?;
    independently_audit_implementation(implementation)?;
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let spec = &implementation.approved_review.specification;
    let contract = &implementation.implementation_contract;
    let review_contract = review_contract();
    if request.expected_previous_review_id != prior.map(|review| review.review_id.clone())
        || request.expected_previous_review_sha256
            != prior.map(|review| review.review_sha256.clone())
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_implementation_artifact_sha256
            != contract.implementation_artifact_sha256
        || request.expected_immutable_code_revision != contract.immutable_code_revision
        || request.expected_specification_review_sha256
            != implementation.approved_review.review_sha256
        || request.expected_transformation_spec_sha256 != spec.transformation_spec_sha256
        || request.expected_transformation_body_sha256 != spec.transformation_body_sha256
        || request.expected_split_specification_sha256
            != spec.split_manifest_specification.specification_sha256
        || request.expected_feature_specification_sha256
            != spec.feature_bundle_specification.specification_sha256
        || request.expected_dataset_content_sha256 != spec.subject.dataset_content_sha256
        || request.expected_manifest_sha256 != spec.subject.manifest_sha256
        || request.expected_candidate_set_sha256 != spec.subject.candidate_set_sha256
        || request.expected_governance_review_sha256 != spec.governance_review_sha256
        || request.expected_review_contract_sha256 != review_contract.contract_sha256
    {
        return Err("隔离转换实现、上游绑定或独立复核合同已变化，请刷新后重试".to_string());
    }
    if prior.is_some_and(|review| review.implementation != *implementation) {
        return Err("上一条复核绑定的实现与当前实现不一致".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(implementation);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == reviewer_id)
    {
        return Err("复核人参与过完整上游或实现登记链，必须更换独立复核人".to_string());
    }
    let approval = request.verdict
        == HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict::ApprovedForFutureIsolatedTransformationRunnerRegistration;
    let all_confirmed = request_all_checks(&request);
    if approval && !all_confirmed {
        return Err("批准隔离转换实现前必须逐项确认工件、确定性、沙箱和零执行边界".to_string());
    }
    let mut review = HistoricalOutcomeOfflineDatasetTransformationImplementationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: prior.map(|review| review.review_id.clone()),
        previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
        implementation: implementation.clone(),
        review_contract,
        submitted_at,
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        reviewer_independent_from_complete_registration_chain: true,
        verdict: request.verdict,
        rationale,
        known_limitations,
        exact_current_implementation_and_upstream_binding_confirmed: request
            .exact_current_implementation_and_upstream_binding_confirmed,
        reviewer_independence_confirmed: request.reviewer_independence_confirmed,
        artifact_digest_independently_reproduced_confirmed: request
            .artifact_digest_independently_reproduced_confirmed,
        immutable_code_revision_reproducible_confirmed: request
            .immutable_code_revision_reproducible_confirmed,
        deterministic_split_implementation_matches_specification_confirmed: request
            .deterministic_split_implementation_matches_specification_confirmed,
        exact_65_feature_implementation_matches_allowlist_confirmed: request
            .exact_65_feature_implementation_matches_allowlist_confirmed,
        canonical_serializer_and_schema_determinism_confirmed: request
            .canonical_serializer_and_schema_determinism_confirmed,
        sealed_read_only_input_and_create_once_output_contract_confirmed: request
            .sealed_read_only_input_and_create_once_output_contract_confirmed,
        bounded_resource_contract_confirmed: request.bounded_resource_contract_confirmed,
        no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed:
            request.no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed,
        review_runner_execution_output_target_and_training_separation_confirmed: request
            .review_runner_execution_output_target_and_training_separation_confirmed,
        no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
            request.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed,
        future_isolated_transformation_runner_registration_eligible: approval && all_confirmed,
        transformation_runner_registered: false,
        transformation_execution_authorized: false,
        transformation_execution_started: false,
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

fn request_all_checks(
    request: &ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
) -> bool {
    request.exact_current_implementation_and_upstream_binding_confirmed
        && request.reviewer_independence_confirmed
        && request.artifact_digest_independently_reproduced_confirmed
        && request.immutable_code_revision_reproducible_confirmed
        && request.deterministic_split_implementation_matches_specification_confirmed
        && request.exact_65_feature_implementation_matches_allowlist_confirmed
        && request.canonical_serializer_and_schema_determinism_confirmed
        && request.sealed_read_only_input_and_create_once_output_contract_confirmed
        && request.bounded_resource_contract_confirmed
        && request.no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed
        && request.review_runner_execution_output_target_and_training_separation_confirmed
        && request.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed
}

fn review_all_checks(
    review: &HistoricalOutcomeOfflineDatasetTransformationImplementationReview,
) -> bool {
    review.exact_current_implementation_and_upstream_binding_confirmed
        && review.reviewer_independence_confirmed
        && review.artifact_digest_independently_reproduced_confirmed
        && review.immutable_code_revision_reproducible_confirmed
        && review.deterministic_split_implementation_matches_specification_confirmed
        && review.exact_65_feature_implementation_matches_allowlist_confirmed
        && review.canonical_serializer_and_schema_determinism_confirmed
        && review.sealed_read_only_input_and_create_once_output_contract_confirmed
        && review.bounded_resource_contract_confirmed
        && review.no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed
        && review.review_runner_execution_output_target_and_training_separation_confirmed
        && review.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed
}

fn independently_audit_implementation(
    implementation: &HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
) -> Result<(), String> {
    let contract = &implementation.implementation_contract;
    let spec = &implementation.approved_review.specification;
    let artifact_ok = valid_sha256(&contract.implementation_artifact_sha256)
        && !contract.immutable_code_revision.trim().is_empty()
        && contract.split_implementation_id == EXPECTED_SPLIT_IMPLEMENTATION_ID
        && contract.split_implementation_version == EXPECTED_SPLIT_IMPLEMENTATION_VERSION
        && contract.feature_implementation_id == EXPECTED_FEATURE_IMPLEMENTATION_ID
        && contract.feature_implementation_version == EXPECTED_FEATURE_IMPLEMENTATION_VERSION
        && contract.canonical_serializer_version == EXPECTED_SERIALIZER_VERSION
        && contract.input_schema_version == EXPECTED_INPUT_SCHEMA_VERSION
        && contract.output_schema_version == EXPECTED_OUTPUT_SCHEMA_VERSION
        && spec.split_manifest_specification.schema_version
            == "hone-deterministic-connected-component-split-manifest-spec-v2-exact-boundaries"
        && spec.feature_bundle_specification.schema_version
            == "hone-point-in-time-seven-layer-feature-bundle-spec-v2-feature-id-allowlist"
        && spec.feature_bundle_specification.allowed_features.len() == EXPECTED_FEATURE_COUNT
        && contract.maximum_parallel_subjects == 1
        && contract.maximum_memory_mebibytes == 2_048;
    let sandbox_closed = !contract.callable_entrypoint_present
        && !contract.environment_inheritance_allowed
        && !contract.environment_variables_allowed
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.external_tools_allowed
        && !contract.child_process_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.historical_state_mutation_allowed
        && contract.input_contract.contains("当前登记不读取任何输入")
        && contract
            .output_contract
            .contains("当前登记不生成、不连接、不写入任何输出");
    let authority_closed = implementation.status == "registered_not_run"
        && implementation.future_independent_implementation_review_eligible
        && !implementation.independent_implementation_review_completed
        && !implementation.split_manifest_generation_authorized
        && !implementation.split_manifest_generated
        && !implementation.feature_bundle_generation_authorized
        && !implementation.feature_bundle_generated
        && !implementation.feature_join_authorized
        && !implementation.feature_join_performed
        && !implementation.semantic_target_assignment_authorized
        && !implementation.training_authorized
        && !implementation.training_run_allowed
        && !implementation.reward_authorized
        && !implementation.shadow_portfolio_authorized
        && !implementation.order_generation_authorized
        && !implementation.broker_access_authorized
        && !implementation.trading_authorized;
    if !artifact_ok || !sandbox_closed || !authority_closed {
        return Err("independent transformation implementation audit failed closed".to_string());
    }
    Ok(())
}

fn review_contract() -> HistoricalOutcomeOfflineDatasetTransformationImplementationReviewContract {
    let mut contract = HistoricalOutcomeOfflineDatasetTransformationImplementationReviewContract {
        schema_version: REVIEW_CONTRACT_VERSION.to_string(),
        contract_sha256: String::new(),
        independent_audit_implementation:
            "hone-independent-transformation-implementation-audit-v1-no-registrar-generator-reuse"
                .to_string(),
        required_artifact_checks: vec![
            "artifact_sha256_is_independently_reproduced".to_string(),
            "immutable_code_revision_is_reproducible".to_string(),
            "split_implementation_matches_exact_boundary_specification".to_string(),
            "feature_implementation_matches_exact_65_feature_id_allowlist".to_string(),
            "canonical_serializer_and_fixed_schemas_are_deterministic".to_string(),
        ],
        required_sandbox_checks: vec![
            "sealed_read_only_input_and_create_once_content_addressed_output".to_string(),
            "static_single_subject_and_2048_mib_resource_limits".to_string(),
            "no_entrypoint_environment_variables_secrets_network_tools_or_child_process"
                .to_string(),
            "no_production_read_write_or_historical_mutation".to_string(),
            "no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading"
                .to_string(),
        ],
        approval_scope:
            "future_isolated_transformation_runner_specification_registration_only_no_execution"
                .to_string(),
        runner_registration_separate: true,
        execution_authorization_separate: true,
        transformation_execution_separate: true,
        output_validation_separate: true,
        target_definition_separate: true,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static transformation implementation review contract is serializable");
    contract
}

fn expected_excluded_prior_actor_ids(
    implementation: &HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
) -> Vec<String> {
    let mut actors = implementation.excluded_prior_actor_ids.clone();
    actors.push(implementation.registered_by.clone());
    actors.push(implementation.approved_review.reviewer_id.clone());
    actors.sort();
    actors.dedup();
    actors
}

fn validate_review(
    review: &HistoricalOutcomeOfflineDatasetTransformationImplementationReview,
) -> Result<(), String> {
    validate_historical_outcome_offline_dataset_transformation_implementation_for_review(
        &review.implementation,
    )?;
    independently_audit_implementation(&review.implementation)?;
    let approval = review.verdict
        == HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict::ApprovedForFutureIsolatedTransformationRunnerRegistration;
    let authority_closed = !review.transformation_runner_registered
        && !review.transformation_execution_authorized
        && !review.transformation_execution_started
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
            != expected_excluded_prior_actor_ids(&review.implementation)
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
        || (approval && !review_all_checks(review))
        || review.future_isolated_transformation_runner_registration_eligible
            != (approval && review_all_checks(review))
        || !authority_closed
    {
        return Err(
            "historical outcome transformation implementation review is invalid or over-authorized"
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
        return Err("transformation implementation review predecessor is invalid".to_string());
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[HistoricalOutcomeOfflineDatasetTransformationImplementationReview],
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationImplementationReview>, String> {
    if reviews.is_empty() {
        return Ok(Vec::new());
    }
    let mut by_id = HashMap::new();
    let mut child_by_parent = HashMap::<String, String>::new();
    let mut roots = Vec::new();
    for review in reviews {
        validate_review(review)?;
        if review.implementation.implementation_id != implementation_id
            || by_id
                .insert(review.review_id.clone(), review.clone())
                .is_some()
        {
            return Err(
                "transformation implementation review replay or subject mismatch".to_string(),
            );
        }
        match &review.previous_review_id {
            Some(parent) => {
                if child_by_parent
                    .insert(parent.clone(), review.review_id.clone())
                    .is_some()
                {
                    return Err("transformation implementation review chain branches".to_string());
                }
            }
            None => roots.push(review.review_id.clone()),
        }
    }
    if roots.len() != 1 {
        return Err("transformation implementation review chain has no single root".to_string());
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = roots.pop().unwrap();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err("transformation implementation review chain contains a cycle".to_string());
        }
        let review = by_id
            .get(&current)
            .ok_or_else(|| {
                "transformation implementation review chain is disconnected".to_string()
            })?
            .clone();
        if let Some(parent_id) = &review.previous_review_id {
            let parent = by_id.get(parent_id).ok_or_else(|| {
                "transformation implementation review predecessor is missing".to_string()
            })?;
            if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                return Err(
                    "transformation implementation review predecessor hash drift".to_string(),
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
        return Err("transformation implementation review chain is disconnected".to_string());
    }
    Ok(ordered)
}

async fn read_reviews_for_implementation(
    state: &AppState,
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationImplementationReview>, String> {
    let root = review_root(state).join(implementation_id);
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
        let review: HistoricalOutcomeOfflineDatasetTransformationImplementationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("transformation implementation review path mismatch".to_string());
        }
        reviews.push(review);
    }
    validate_review_chain(&reviews, implementation_id)
}

fn review_fingerprint(
    review: &HistoricalOutcomeOfflineDatasetTransformationImplementationReview,
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
        .join("historical_outcome_offline_dataset_transformation_implementation_reviews/reviews")
}

struct ReviewLock {
    path: PathBuf,
}

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, implementation_id: &str) -> Result<ReviewLock, String> {
    if !valid_id(implementation_id) {
        return Err("隔离转换实现 ID 无效".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-outcome-transformation-implementation-review-{implementation_id}.lock"
    ));
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
                return Err("另一个管理员正在复核隔离转换实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("隔离转换实现复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_transformation_implementations::tests::record;

    fn request(
        implementation: &HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
        prior: Option<&HistoricalOutcomeOfflineDatasetTransformationImplementationReview>,
    ) -> ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest {
        let spec = &implementation.approved_review.specification;
        let contract = &implementation.implementation_contract;
        ReviewHistoricalOutcomeOfflineDatasetTransformationImplementationRequest {
            expected_previous_review_id: prior.map(|review| review.review_id.clone()),
            expected_previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_contract_sha256: contract.contract_sha256.clone(),
            expected_implementation_artifact_sha256: contract
                .implementation_artifact_sha256
                .clone(),
            expected_immutable_code_revision: contract.immutable_code_revision.clone(),
            expected_specification_review_sha256: implementation
                .approved_review
                .review_sha256
                .clone(),
            expected_transformation_spec_sha256: spec.transformation_spec_sha256.clone(),
            expected_transformation_body_sha256: spec.transformation_body_sha256.clone(),
            expected_split_specification_sha256: spec
                .split_manifest_specification
                .specification_sha256
                .clone(),
            expected_feature_specification_sha256: spec
                .feature_bundle_specification
                .specification_sha256
                .clone(),
            expected_dataset_content_sha256: spec.subject.dataset_content_sha256.clone(),
            expected_manifest_sha256: spec.subject.manifest_sha256.clone(),
            expected_candidate_set_sha256: spec.subject.candidate_set_sha256.clone(),
            expected_governance_review_sha256: spec.governance_review_sha256.clone(),
            expected_review_contract_sha256: review_contract().contract_sha256,
            verdict: HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict::ApprovedForFutureIsolatedTransformationRunnerRegistration,
            rationale: "independently reproduced the artifact and closed sandbox contract".to_string(),
            known_limitations: "implementation has not been registered as a runner or executed".to_string(),
            exact_current_implementation_and_upstream_binding_confirmed: true,
            reviewer_independence_confirmed: true,
            artifact_digest_independently_reproduced_confirmed: true,
            immutable_code_revision_reproducible_confirmed: true,
            deterministic_split_implementation_matches_specification_confirmed: true,
            exact_65_feature_implementation_matches_allowlist_confirmed: true,
            canonical_serializer_and_schema_determinism_confirmed: true,
            sealed_read_only_input_and_create_once_output_contract_confirmed: true,
            bounded_resource_contract_confirmed: true,
            no_entrypoint_environment_secrets_network_tools_child_process_or_production_access_confirmed: true,
            review_runner_execution_output_target_and_training_separation_confirmed: true,
            no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn approved_review()
    -> ApprovedHistoricalOutcomeOfflineDatasetTransformationImplementationReview {
        let implementation = record();
        let review = build_review(
            &implementation,
            None,
            "implementation-reviewer",
            request(&implementation, None),
            DateTime::from_timestamp(1_760_000_000, 0).unwrap(),
        )
        .unwrap();
        ApprovedHistoricalOutcomeOfflineDatasetTransformationImplementationReview {
            implementation,
            review,
        }
    }

    #[test]
    fn independent_audit_requires_exact_artifact_algorithms_and_closed_sandbox() {
        let implementation = record();
        assert!(independently_audit_implementation(&implementation).is_ok());
        let mut tampered = implementation.clone();
        tampered.implementation_contract.maximum_parallel_subjects = 2;
        assert!(independently_audit_implementation(&tampered).is_err());
    }

    #[test]
    fn reviewer_must_be_outside_complete_registration_chain() {
        let implementation = record();
        for actor in [
            "implementation-registrar",
            "transformation-spec-reviewer",
            "registrar",
        ] {
            assert!(
                build_review(
                    &implementation,
                    None,
                    actor,
                    request(&implementation, None),
                    Utc::now()
                )
                .is_err()
            );
        }
    }

    #[test]
    fn approval_requires_every_confirmation_and_exact_binding() {
        let implementation = record();
        let mut missing = request(&implementation, None);
        missing.artifact_digest_independently_reproduced_confirmed = false;
        assert!(build_review(&implementation, None, "reviewer", missing, Utc::now()).is_err());
        let mut stale = request(&implementation, None);
        stale.expected_implementation_artifact_sha256 = "0".repeat(64);
        assert!(build_review(&implementation, None, "reviewer", stale, Utc::now()).is_err());
    }

    #[test]
    fn approval_only_allows_future_runner_registration() {
        let implementation = record();
        let review = build_review(
            &implementation,
            None,
            "implementation-reviewer",
            request(&implementation, None),
            Utc::now(),
        )
        .unwrap();
        assert!(review.future_isolated_transformation_runner_registration_eligible);
        assert!(!review.transformation_runner_registered);
        assert!(!review.transformation_execution_authorized);
        assert!(!review.split_manifest_generation_authorized);
        assert!(!review.training_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn one_bit_or_authority_tamper_fails_validation() {
        let implementation = record();
        let mut review = build_review(
            &implementation,
            None,
            "implementation-reviewer",
            request(&implementation, None),
            Utc::now(),
        )
        .unwrap();
        review.feature_bundle_generation_authorized = true;
        assert!(validate_review(&review).is_err());
    }

    #[test]
    fn append_only_chain_rejects_branch_or_hash_drift() {
        let implementation = record();
        let mut first_request = request(&implementation, None);
        first_request.verdict =
            HistoricalOutcomeOfflineDatasetTransformationImplementationReviewVerdict::ChangesRequested;
        let first = build_review(
            &implementation,
            None,
            "reviewer-a",
            first_request,
            DateTime::from_timestamp(1_760_000_000, 0).unwrap(),
        )
        .unwrap();
        let second = build_review(
            &implementation,
            Some(&first),
            "reviewer-b",
            request(&implementation, Some(&first)),
            DateTime::from_timestamp(1_760_000_100, 0).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validate_review_chain(
                &[second.clone(), first.clone()],
                &implementation.implementation_id
            )
            .unwrap()
            .len(),
            2
        );
        let mut branch = second.clone();
        branch.review_id = "9".repeat(32);
        branch.review_sha256 = "9".repeat(64);
        assert!(
            validate_review_chain(&[first, second, branch], &implementation.implementation_id)
                .is_err()
        );
    }

    #[test]
    fn review_contract_is_content_bound_and_all_later_gates_are_separate() {
        let contract = review_contract();
        assert!(contract.runner_registration_separate);
        assert!(contract.execution_authorization_separate);
        assert!(contract.transformation_execution_separate);
        assert!(contract.output_validation_separate);
        assert!(contract.target_definition_separate);
        assert!(valid_sha256(&contract.contract_sha256));
    }
}
