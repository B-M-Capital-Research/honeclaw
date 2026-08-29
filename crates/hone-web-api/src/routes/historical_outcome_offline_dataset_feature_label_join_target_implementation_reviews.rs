//! Independent review of an immutable isolated feature-label join and continuous-target
//! implementation.
//!
//! Approval permits only future registration of a separate isolated runner specification. This
//! module has no implementation entrypoint and cannot read labels, execute a join, create joined
//! rows, train, reward, shadow, draft orders, access a broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_implementations::{
    HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
    independent_review_eligible_historical_outcome_feature_label_join_target_implementations,
    validate_historical_outcome_feature_label_join_target_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-implementation-independent-review-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-implementation-independent-review-v1-no-execution";
const REVIEW_CONTRACT_SCHEMA_VERSION: &str =
    "hone-feature-label-join-target-implementation-independent-review-contract-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-feature-label-join-target-implementation-independent-audit-v1";
const EXPECTED_IMPLEMENTATION_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-implementation-v1";
const EXPECTED_IMPLEMENTATION_POLICY_VERSION: &str =
    "hone-feature-label-join-target-implementation-registration-v1-no-execution";
const EXPECTED_IMPLEMENTATION_STATUS: &str = "registered_not_reviewed_not_run";
const EXPECTED_CONTRACT_SCHEMA_VERSION: &str =
    "hone-feature-label-join-target-isolated-implementation-contract-v1";
const EXPECTED_JOIN_IMPLEMENTATION_ID: &str = "hone-exact-entry-feature-outcome-join-enumerator";
const EXPECTED_JOIN_IMPLEMENTATION_VERSION: &str = "v1-one-to-one-fail-closed";
const EXPECTED_TARGET_IMPLEMENTATION_ID: &str = "hone-raw-outcome-continuous-target-projector";
const EXPECTED_TARGET_IMPLEMENTATION_VERSION: &str = "v1-exact-f64-bits-nine-targets";
const EXPECTED_SERIALIZER_VERSION: &str = "hone-canonical-json-serializer-v1";
const EXPECTED_INPUT_SCHEMA_VERSION: &str = "hone-reviewed-feature-label-join-target-input-v1";
const EXPECTED_OUTPUT_SCHEMA_VERSION: &str = "hone-unmaterialized-joined-target-envelope-v1";
const EXPECTED_FEATURE_COUNT: u16 = 65;
const EXPECTED_TARGET_COUNT: u8 = 9;
const EXPECTED_HORIZONS: [u16; 3] = [20, 60, 250];
const EXPECTED_MAXIMUM_PARALLEL_DATASETS: u8 = 1;
const EXPECTED_MAXIMUM_MEMORY_MEBIBYTES: u32 = 4_096;
const MAX_RATIONALE_CHARS: usize = 3_000;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict {
    ApprovedForFutureIsolatedJoinTargetRunnerRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub independent_audit_implementation: String,
    pub required_fingerprint_checks: Vec<String>,
    pub required_semantic_checks: Vec<String>,
    pub required_sandbox_checks: Vec<String>,
    pub approval_scope: String,
    pub runner_registration_separate: bool,
    pub first_execution_authorization_separate: bool,
    pub join_execution_separate: bool,
    pub output_validation_separate: bool,
    pub training_and_reward_governance_separate: bool,
    pub targets_remain_engineering_candidates_not_strategy_truth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub implementation_artifact_sha256: String,
    pub immutable_code_revision: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub exact_current_review_specification_artifact_and_dataset_binding_valid: bool,
    pub exact_one_to_one_join_implementation_valid: bool,
    pub exact_nine_raw_f64_target_projection_valid: bool,
    pub point_in_time_missingness_purge_embargo_and_split_isolation_valid: bool,
    pub sealed_holdout_inaccessible_to_training_and_tuning: bool,
    pub canonical_serializer_schema_and_resource_contract_valid: bool,
    pub no_action_position_threshold_rank_or_reward_semantics: bool,
    pub no_entrypoint_environment_secret_network_tool_child_process_or_data_store_access: bool,
    pub all_runner_execution_training_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_artifact_sha256: String,
    expected_immutable_code_revision: String,
    expected_specification_review_sha256: String,
    expected_specification_review_audit_sha256: String,
    expected_specification_sha256: String,
    expected_specification_body_sha256: String,
    expected_join_specification_sha256: String,
    expected_target_specification_sha256: String,
    expected_combined_artifact_sha256: String,
    expected_dataset_content_sha256: String,
    expected_review_contract_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_implementation_and_complete_upstream_binding_confirmed: bool,
    reviewer_independence_from_complete_prior_chain_confirmed: bool,
    implementation_record_and_contract_hashes_independently_reproduced_confirmed: bool,
    implementation_artifact_digest_and_code_revision_reproducible_confirmed: bool,
    exact_one_to_one_join_and_fail_closed_key_semantics_confirmed: bool,
    exact_nine_raw_f64_target_projection_without_transform_confirmed: bool,
    point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: bool,
    sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed: bool,
    canonical_serializer_fixed_schemas_and_resource_limits_confirmed: bool,
    no_action_position_threshold_rank_or_reward_semantics_confirmed: bool,
    no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed:
        bool,
    review_runner_authorization_execution_output_validation_and_training_separation_confirmed: bool,
    no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
    pub review_contract: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewContract,
    pub independent_audit: HistoricalOutcomeFeatureLabelJoinTargetImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub reviewer_independent_from_complete_prior_chain: bool,
    pub verdict: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_current_implementation_and_complete_upstream_binding_confirmed: bool,
    pub reviewer_independence_from_complete_prior_chain_confirmed: bool,
    pub implementation_record_and_contract_hashes_independently_reproduced_confirmed: bool,
    pub implementation_artifact_digest_and_code_revision_reproducible_confirmed: bool,
    pub exact_one_to_one_join_and_fail_closed_key_semantics_confirmed: bool,
    pub exact_nine_raw_f64_target_projection_without_transform_confirmed: bool,
    pub point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: bool,
    pub sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed: bool,
    pub canonical_serializer_fixed_schemas_and_resource_limits_confirmed: bool,
    pub no_action_position_threshold_rank_or_reward_semantics_confirmed: bool,
    pub no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed:
        bool,
    pub review_runner_authorization_execution_output_validation_and_training_separation_confirmed:
        bool,
    pub no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub future_isolated_join_target_runner_registration_eligible: bool,
    pub isolated_runner_registered: bool,
    pub first_execution_authorization_review_eligible: bool,
    pub label_access_authorized: bool,
    pub join_execution_authorized: bool,
    pub join_executed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub semantic_target_assigned: bool,
    pub joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub output_validation_authorized: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewItem {
    pub implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
    pub current_independent_audit:
        HistoricalOutcomeFeatureLabelJoinTargetImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub upstream_binding_current: bool,
    pub latest_review: Option<HistoricalOutcomeFeatureLabelJoinTargetImplementationReview>,
    pub review_eligible: bool,
    pub future_isolated_join_target_runner_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_contract: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewContract,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub runner_registration_eligible_count: usize,
    pub review_status: String,
    pub isolated_runner_registered: bool,
    pub first_execution_authorization_review_eligible: bool,
    pub label_access_authorized: bool,
    pub join_execution_authorized: bool,
    pub join_executed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub semantic_target_assigned: bool,
    pub joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub output_validation_authorized: bool,
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
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub runner_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeFeatureLabelJoinTargetImplementationReview {
    pub implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
    pub review: HistoricalOutcomeFeatureLabelJoinTargetImplementationReview,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_implementation_reviews(
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
            warn!(%error, "feature-label join/target implementation review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "特征—标签连接与目标实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_feature_label_join_target_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &implementation_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_feature_label_join_target_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewReadinessSummary {
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            approved_count: registry.approved_count,
            current_binding_approved_count: registry.current_binding_approved_count,
            runner_registration_eligible_count: registry.runner_registration_eligible_count,
            review_status: registry.review_status,
        },
    )
}

pub(crate) async fn approved_historical_outcome_feature_label_join_target_implementation_reviews(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeFeatureLabelJoinTargetImplementationReview>, String> {
    let implementations =
        independent_review_eligible_historical_outcome_feature_label_join_target_implementations(
            state,
        )
        .await?;
    let mut approved = Vec::new();
    for implementation in implementations {
        let reviews =
            read_reviews_for_implementation(state, &implementation.implementation_id).await?;
        let Some(review) = reviews.last().cloned() else {
            continue;
        };
        if review.implementation != implementation
            || !review.future_isolated_join_target_runner_registration_eligible
        {
            continue;
        }
        validate_historical_outcome_feature_label_join_target_implementation_review_for_runner_registration(
            &review,
        )?;
        approved.push(
            ApprovedHistoricalOutcomeFeatureLabelJoinTargetImplementationReview {
                implementation,
                review,
            },
        );
    }
    approved.sort_by(|left, right| {
        right
            .review
            .submitted_at
            .cmp(&left.review.submitted_at)
            .then_with(|| right.review.review_id.cmp(&left.review.review_id))
    });
    Ok(approved)
}

pub(crate) fn validate_historical_outcome_feature_label_join_target_implementation_review_for_runner_registration(
    review: &HistoricalOutcomeFeatureLabelJoinTargetImplementationReview,
) -> Result<(), String> {
    validate_review(review)?;
    if !review.future_isolated_join_target_runner_registration_eligible {
        return Err("join/target 实现复核未批准未来隔离 runner 规格登记".to_string());
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry, String> {
    let implementations =
        independent_review_eligible_historical_outcome_feature_label_join_target_implementations(
            state,
        )
        .await?;
    let mut items = Vec::with_capacity(implementations.len());
    for implementation in implementations {
        let audit = independently_audit_implementation(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("当前 join/target 实现独立审计失败，晋级已关闭".to_string());
        }
        let reviews =
            read_reviews_for_implementation(state, &implementation.implementation_id).await?;
        let complete_review_actor_ids = reviews
            .iter()
            .map(|review| review.reviewer_id.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|review| {
            review.implementation != implementation || review.independent_audit != audit
        }) {
            return Err("join/target 实现复核绑定或独立审计已变化".to_string());
        }
        let approved = latest_review
            .as_ref()
            .is_some_and(|review| review.future_isolated_join_target_runner_registration_eligible);
        items.push(
            HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewItem {
                implementation,
                current_independent_audit: audit,
                complete_review_actor_ids,
                upstream_binding_current: true,
                latest_review,
                review_eligible: !approved,
                future_isolated_join_target_runner_registration_eligible: approved,
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
                review.future_isolated_join_target_runner_registration_eligible
            })
        })
        .count();
    let current_binding_approved_count = items
        .iter()
        .filter(|item| item.future_isolated_join_target_runner_registration_eligible)
        .count();
    let review_status = if items.is_empty() {
        "waiting_current_registered_join_target_implementation"
    } else if current_binding_approved_count > 0 {
        "current_join_target_implementation_approved_only_for_future_isolated_runner_registration"
    } else if review_eligible_count > 0 {
        "current_join_target_implementation_ready_for_independent_review"
    } else {
        "join_target_implementation_review_blocked_fail_closed"
    };
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_contract: review_contract(),
            items,
            review_eligible_count,
            reviewed_count,
            approved_count,
            current_binding_approved_count,
            runner_registration_eligible_count: current_binding_approved_count,
            review_status: review_status.to_string(),
            isolated_runner_registered: false,
            first_execution_authorization_review_eligible: false,
            label_access_authorized: false,
            join_execution_authorized: false,
            join_executed: false,
            semantic_target_assignment_authorized: false,
            semantic_target_assigned: false,
            joined_dataset_created: false,
            copied_to_training_store: false,
            output_validation_authorized: false,
            training_data_access_authorized: false,
            training_authorized: false,
            training_run_allowed: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "第 39 阶段由独立角色重算当前 join/target 实现记录与合同指纹，并复核工件/代码、一对一连接、九维原始目标、sealed holdout、序列化/schema、资源和零能力沙箱。批准只允许未来登记隔离 runner 规范；本阶段不登记 runner、不读取标签、不执行 join、不创建 joined/training rows，也不训练、奖励、影子、订单、券商或交易。".to_string(),
        },
    )
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationReview, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("join/target 实现独立复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementations =
        independent_review_eligible_historical_outcome_feature_label_join_target_implementations(
            state,
        )
        .await?;
    let implementation = implementations
        .iter()
        .find(|value| value.implementation_id == implementation_id)
        .ok_or_else(|| "当前 join/target 隔离实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews_for_implementation(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|review| review.future_isolated_join_target_runner_registration_eligible) {
        return Err("当前 join/target 实现已经独立批准，不得追加或分叉复核".to_string());
    }
    let review = build_review(
        implementation,
        &reviews,
        prior,
        reviewer_id,
        request,
        Utc::now(),
    )?;
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
    implementation: &HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
    prior_reviews: &[HistoricalOutcomeFeatureLabelJoinTargetImplementationReview],
    prior: Option<&HistoricalOutcomeFeatureLabelJoinTargetImplementationReview>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationReview, String> {
    validate_historical_outcome_feature_label_join_target_implementation_for_review(
        implementation,
    )?;
    let independent_audit = independently_audit_implementation(implementation)?;
    if !independent_audit.mismatch_reasons.is_empty() {
        return Err("join/target 实现独立审计不通过".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let specification_review = &implementation.approved_review;
    let specification = &specification_review.specification;
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
        || request.expected_specification_review_sha256 != specification_review.review_sha256
        || request.expected_specification_review_audit_sha256
            != specification_review.independent_audit.audit_sha256
        || request.expected_specification_sha256 != specification.specification_sha256
        || request.expected_specification_body_sha256 != specification.specification_body_sha256
        || request.expected_join_specification_sha256
            != specification.join_specification.specification_sha256
        || request.expected_target_specification_sha256
            != specification.target_specification.specification_sha256
        || request.expected_combined_artifact_sha256 != specification.combined_artifact_sha256
        || request.expected_dataset_content_sha256 != specification.dataset_content_sha256
        || request.expected_review_contract_sha256 != review_contract.contract_sha256
        || request.expected_independent_audit_sha256 != independent_audit.audit_sha256
    {
        return Err(
            "join/target 实现、完整上游、独立审计或复核合同已变化，请刷新后重试".to_string(),
        );
    }
    if prior.is_some_and(|review| review.implementation != *implementation) {
        return Err("上一条复核绑定的实现与当前实现不一致".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == reviewer_id)
    {
        return Err("复核人参与过完整上游或此前复核链，必须更换独立角色".to_string());
    }
    let approval = request.verdict
        == HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict::ApprovedForFutureIsolatedJoinTargetRunnerRegistration;
    let all_confirmed = request_all_checks(&request);
    if approval && !all_confirmed {
        return Err("批准 join/target 实现前必须逐项确认指纹、语义、沙箱和零执行边界".to_string());
    }
    let mut review = HistoricalOutcomeFeatureLabelJoinTargetImplementationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: prior.map(|review| review.review_id.clone()),
        previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
        implementation: implementation.clone(),
        review_contract,
        independent_audit,
        submitted_at,
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        reviewer_independent_from_complete_prior_chain: true,
        verdict: request.verdict,
        rationale,
        known_limitations,
        exact_current_implementation_and_complete_upstream_binding_confirmed: request
            .exact_current_implementation_and_complete_upstream_binding_confirmed,
        reviewer_independence_from_complete_prior_chain_confirmed: request
            .reviewer_independence_from_complete_prior_chain_confirmed,
        implementation_record_and_contract_hashes_independently_reproduced_confirmed: request
            .implementation_record_and_contract_hashes_independently_reproduced_confirmed,
        implementation_artifact_digest_and_code_revision_reproducible_confirmed: request
            .implementation_artifact_digest_and_code_revision_reproducible_confirmed,
        exact_one_to_one_join_and_fail_closed_key_semantics_confirmed: request
            .exact_one_to_one_join_and_fail_closed_key_semantics_confirmed,
        exact_nine_raw_f64_target_projection_without_transform_confirmed: request
            .exact_nine_raw_f64_target_projection_without_transform_confirmed,
        point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: request
            .point_in_time_missingness_purge_embargo_and_split_isolation_confirmed,
        sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed:
            request.sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed,
        canonical_serializer_fixed_schemas_and_resource_limits_confirmed: request
            .canonical_serializer_fixed_schemas_and_resource_limits_confirmed,
        no_action_position_threshold_rank_or_reward_semantics_confirmed: request
            .no_action_position_threshold_rank_or_reward_semantics_confirmed,
        no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed:
            request.no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed,
        review_runner_authorization_execution_output_validation_and_training_separation_confirmed:
            request.review_runner_authorization_execution_output_validation_and_training_separation_confirmed,
        no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed:
            request.no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed,
        future_isolated_join_target_runner_registration_eligible: approval && all_confirmed,
        isolated_runner_registered: false,
        first_execution_authorization_review_eligible: false,
        label_access_authorized: false,
        join_execution_authorized: false,
        join_executed: false,
        semantic_target_assignment_authorized: false,
        semantic_target_assigned: false,
        joined_dataset_created: false,
        copied_to_training_store: false,
        output_validation_authorized: false,
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

fn request_all_checks(
    request: &ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
) -> bool {
    request.exact_current_implementation_and_complete_upstream_binding_confirmed
        && request.reviewer_independence_from_complete_prior_chain_confirmed
        && request.implementation_record_and_contract_hashes_independently_reproduced_confirmed
        && request.implementation_artifact_digest_and_code_revision_reproducible_confirmed
        && request.exact_one_to_one_join_and_fail_closed_key_semantics_confirmed
        && request.exact_nine_raw_f64_target_projection_without_transform_confirmed
        && request.point_in_time_missingness_purge_embargo_and_split_isolation_confirmed
        && request.sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed
        && request.canonical_serializer_fixed_schemas_and_resource_limits_confirmed
        && request.no_action_position_threshold_rank_or_reward_semantics_confirmed
        && request.no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed
        && request.review_runner_authorization_execution_output_validation_and_training_separation_confirmed
        && request.no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed
}

fn review_all_checks(review: &HistoricalOutcomeFeatureLabelJoinTargetImplementationReview) -> bool {
    review.exact_current_implementation_and_complete_upstream_binding_confirmed
        && review.reviewer_independence_from_complete_prior_chain_confirmed
        && review.implementation_record_and_contract_hashes_independently_reproduced_confirmed
        && review.implementation_artifact_digest_and_code_revision_reproducible_confirmed
        && review.exact_one_to_one_join_and_fail_closed_key_semantics_confirmed
        && review.exact_nine_raw_f64_target_projection_without_transform_confirmed
        && review.point_in_time_missingness_purge_embargo_and_split_isolation_confirmed
        && review.sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed
        && review.canonical_serializer_fixed_schemas_and_resource_limits_confirmed
        && review.no_action_position_threshold_rank_or_reward_semantics_confirmed
        && review.no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed
        && review.review_runner_authorization_execution_output_validation_and_training_separation_confirmed
        && review.no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed
}

fn independently_audit_implementation(
    implementation: &HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationIndependentAudit, String> {
    let contract = &implementation.implementation_contract;
    let specification_review = &implementation.approved_review;
    let specification = &specification_review.specification;
    let record_hash_independently_reproduced = valid_id(&implementation.implementation_id)
        && valid_sha256(&implementation.implementation_sha256)
        && implementation.implementation_id == implementation.implementation_sha256[..32]
        && implementation.implementation_sha256
            == fingerprint_without(
                implementation,
                &["implementation_id", "implementation_sha256"],
            )?;
    let contract_hash_independently_reproduced = valid_sha256(&contract.contract_sha256)
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?;
    let exact_binding_valid = specification_review
        .future_isolated_join_target_implementation_registration_eligible
        && specification_review
            .independent_audit
            .mismatch_reasons
            .is_empty()
        && specification.combined_artifact_sha256
            == specification.join_specification.combined_artifact_sha256
        && specification.dataset_id == specification.join_specification.dataset_id
        && specification.dataset_content_sha256
            == specification.join_specification.dataset_content_sha256;
    let exact_join_valid = contract.schema_version == EXPECTED_CONTRACT_SCHEMA_VERSION
        && contract.join_implementation_id == EXPECTED_JOIN_IMPLEMENTATION_ID
        && contract.join_implementation_version == EXPECTED_JOIN_IMPLEMENTATION_VERSION
        && specification
            .join_specification
            .one_to_one_outcome_join_required
        && specification
            .join_specification
            .all_allowlisted_feature_records_preserved
        && !specification.join_specification.imputation_allowed
        && !specification.join_specification.interpolation_allowed
        && specification.join_specification.feature_catalog_count
            == usize::from(EXPECTED_FEATURE_COUNT);
    let target = &specification.target_specification;
    let exact_target_valid = contract.target_implementation_id == EXPECTED_TARGET_IMPLEMENTATION_ID
        && contract.target_implementation_version == EXPECTED_TARGET_IMPLEMENTATION_VERSION
        && contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == EXPECTED_TARGET_COUNT
        && contract.exact_horizons_market_sessions == EXPECTED_HORIZONS
        && target.target_definitions.len() == usize::from(EXPECTED_TARGET_COUNT)
        && target.exact_f64_bits_preserved
        && !target.normalization_allowed
        && !target.winsorization_allowed
        && !target.rank_transform_allowed
        && !target.semantic_target_assignment_performed;
    let point_in_time_valid = implementation
        .point_in_time_missingness_purge_embargo_and_split_isolation_confirmed
        && specification
            .join_specification
            .feature_availability_rule
            .contains("decision_available_at")
        && specification
            .join_specification
            .explicit_missingness_rule
            .contains("no_row_or_feature_drop")
        && specification
            .join_specification
            .purged_or_embargoed_row_policy
            .contains("exclude_from_all");
    let sealed_holdout_inaccessible = implementation
        .sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed
        && !specification
            .join_specification
            .sealed_holdout_labels_opened
        && target
            .sealed_holdout_target_access_policy
            .contains("never_training_or_tuning");
    let serializer_schema_resource_valid = contract.canonical_serializer_version
        == EXPECTED_SERIALIZER_VERSION
        && contract.input_schema_version == EXPECTED_INPUT_SCHEMA_VERSION
        && contract.output_schema_version == EXPECTED_OUTPUT_SCHEMA_VERSION
        && contract.maximum_parallel_datasets == EXPECTED_MAXIMUM_PARALLEL_DATASETS
        && contract.maximum_memory_mebibytes == EXPECTED_MAXIMUM_MEMORY_MEBIBYTES
        && contract
            .input_contract
            .contains("当前登记不读取任何输入或标签")
        && contract
            .output_contract
            .contains("当前登记不执行连接、不分配目标、不创建任何输出");
    let no_action_semantics = !target.categorical_action_label_defined
        && !target.buy_hold_sell_threshold_defined
        && !target.portfolio_weight_target_defined
        && !target.rank_transform_allowed
        && !target.scalar_reward_defined;
    let sandbox_closed = valid_sha256(&contract.implementation_artifact_sha256)
        && !contract.immutable_code_revision.trim().is_empty()
        && !contract.callable_entrypoint_present
        && !contract.environment_inheritance_allowed
        && !contract.environment_variables_allowed
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.external_tools_allowed
        && !contract.child_process_allowed
        && !contract.label_store_reads_allowed
        && !contract.training_store_reads_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.historical_state_mutation_allowed;
    let authority_closed = implementation.schema_version == EXPECTED_IMPLEMENTATION_SCHEMA_VERSION
        && implementation.policy_version == EXPECTED_IMPLEMENTATION_POLICY_VERSION
        && implementation.status == EXPECTED_IMPLEMENTATION_STATUS
        && implementation.future_independent_implementation_review_eligible
        && !implementation.independent_implementation_review_completed
        && !implementation.isolated_runner_registration_eligible
        && !implementation.label_access_authorized
        && !implementation.join_execution_authorized
        && !implementation.join_executed
        && !implementation.semantic_target_assignment_authorized
        && !implementation.semantic_target_assigned
        && !implementation.joined_dataset_created
        && !implementation.copied_to_training_store
        && !implementation.output_validation_authorized
        && !implementation.training_data_access_authorized
        && !implementation.training_authorized
        && !implementation.training_run_allowed
        && !implementation.reward_authorized
        && !implementation.shadow_portfolio_authorized
        && !implementation.order_generation_authorized
        && !implementation.broker_access_authorized
        && !implementation.trading_authorized;
    let checks = [
        (
            record_hash_independently_reproduced,
            "implementation_record_hash_mismatch",
        ),
        (
            contract_hash_independently_reproduced,
            "implementation_contract_hash_mismatch",
        ),
        (exact_binding_valid, "current_upstream_binding_mismatch"),
        (exact_join_valid, "exact_join_contract_mismatch"),
        (exact_target_valid, "exact_target_projection_mismatch"),
        (
            point_in_time_valid,
            "point_in_time_or_split_isolation_mismatch",
        ),
        (
            sealed_holdout_inaccessible,
            "sealed_holdout_access_mismatch",
        ),
        (
            serializer_schema_resource_valid,
            "serializer_schema_or_resource_mismatch",
        ),
        (
            no_action_semantics,
            "action_position_rank_or_reward_semantics_present",
        ),
        (sandbox_closed, "sandbox_or_data_store_access_open"),
        (
            authority_closed,
            "runner_execution_training_or_trading_authority_open",
        ),
    ];
    let mismatch_reasons = checks
        .into_iter()
        .filter(|(passed, _)| !passed)
        .map(|(_, reason)| reason.to_string())
        .collect::<Vec<_>>();
    let mut audit = HistoricalOutcomeFeatureLabelJoinTargetImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        implementation_artifact_sha256: contract.implementation_artifact_sha256.clone(),
        immutable_code_revision: contract.immutable_code_revision.clone(),
        implementation_record_hash_independently_reproduced: record_hash_independently_reproduced,
        implementation_contract_hash_independently_reproduced:
            contract_hash_independently_reproduced,
        exact_current_review_specification_artifact_and_dataset_binding_valid: exact_binding_valid,
        exact_one_to_one_join_implementation_valid: exact_join_valid,
        exact_nine_raw_f64_target_projection_valid: exact_target_valid,
        point_in_time_missingness_purge_embargo_and_split_isolation_valid: point_in_time_valid,
        sealed_holdout_inaccessible_to_training_and_tuning: sealed_holdout_inaccessible,
        canonical_serializer_schema_and_resource_contract_valid: serializer_schema_resource_valid,
        no_action_position_threshold_rank_or_reward_semantics: no_action_semantics,
        no_entrypoint_environment_secret_network_tool_child_process_or_data_store_access:
            sandbox_closed,
        all_runner_execution_training_and_trading_authority_closed: authority_closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn review_contract() -> HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewContract {
    let mut contract = HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewContract {
        schema_version: REVIEW_CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        independent_audit_implementation:
            "hone-independent-join-target-implementation-audit-v1-no-registration-validator-reuse"
                .to_string(),
        required_fingerprint_checks: vec![
            "implementation_record_sha256_independently_reproduced".to_string(),
            "implementation_contract_sha256_independently_reproduced".to_string(),
            "artifact_sha256_and_code_revision_reproducible".to_string(),
            "exact_current_review_specification_artifact_dataset_binding".to_string(),
        ],
        required_semantic_checks: vec![
            "strict_one_to_one_entry_join_and_fail_closed_keys".to_string(),
            "exact_nine_raw_f64_targets_at_20_60_250_sessions".to_string(),
            "point_in_time_missingness_purge_embargo_and_official_split_isolation".to_string(),
            "sealed_holdout_never_training_tuning_or_model_selection".to_string(),
            "no_action_position_threshold_rank_or_scalar_reward".to_string(),
        ],
        required_sandbox_checks: vec![
            "canonical_serializer_fixed_schemas_and_static_resource_limits".to_string(),
            "no_entrypoint_environment_secret_network_tool_or_child_process".to_string(),
            "no_label_training_store_or_production_access".to_string(),
            "no_runner_join_rows_training_reward_shadow_order_broker_or_trading".to_string(),
        ],
        approval_scope:
            "future_isolated_join_target_runner_specification_registration_only_no_execution"
                .to_string(),
        runner_registration_separate: true,
        first_execution_authorization_separate: true,
        join_execution_separate: true,
        output_validation_separate: true,
        training_and_reward_governance_separate: true,
        targets_remain_engineering_candidates_not_strategy_truth: true,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static join/target implementation review contract is serializable");
    contract
}

fn expected_excluded_prior_actor_ids(
    implementation: &HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
    prior_reviews: &[HistoricalOutcomeFeatureLabelJoinTargetImplementationReview],
) -> Vec<String> {
    let mut actors = implementation.excluded_prior_actor_ids.clone();
    actors.push(implementation.registered_by.clone());
    actors.push(implementation.approved_review.reviewer_id.clone());
    actors.extend(
        prior_reviews
            .iter()
            .map(|review| review.reviewer_id.clone()),
    );
    actors.sort();
    actors.dedup();
    actors
}

fn validate_review(
    review: &HistoricalOutcomeFeatureLabelJoinTargetImplementationReview,
) -> Result<(), String> {
    validate_historical_outcome_feature_label_join_target_implementation_for_review(
        &review.implementation,
    )?;
    let expected_audit = independently_audit_implementation(&review.implementation)?;
    let approval = review.verdict
        == HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict::ApprovedForFutureIsolatedJoinTargetRunnerRegistration;
    let authority_closed = !review.isolated_runner_registered
        && !review.first_execution_authorization_review_eligible
        && !review.label_access_authorized
        && !review.join_execution_authorized
        && !review.join_executed
        && !review.semantic_target_assignment_authorized
        && !review.semantic_target_assigned
        && !review.joined_dataset_created
        && !review.copied_to_training_store
        && !review.output_validation_authorized
        && !review.training_data_access_authorized
        && !review.training_authorized
        && !review.training_run_allowed
        && !review.reward_authorized
        && !review.shadow_portfolio_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let audit_valid = review.independent_audit == expected_audit
        && review.independent_audit.mismatch_reasons.is_empty()
        && review
            .independent_audit
            .all_runner_execution_training_and_trading_authority_closed;
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
        || (approval && !review_all_checks(review))
        || review.future_isolated_join_target_runner_registration_eligible
            != (approval && review_all_checks(review))
        || !authority_closed
    {
        return Err("join/target implementation review is invalid or over-authorized".to_string());
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
        return Err("join/target implementation review predecessor is invalid".to_string());
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[HistoricalOutcomeFeatureLabelJoinTargetImplementationReview],
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetImplementationReview>, String> {
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
            return Err("join/target implementation review replay or subject mismatch".to_string());
        }
        match &review.previous_review_id {
            Some(parent) => {
                if child_by_parent
                    .insert(parent.clone(), review.review_id.clone())
                    .is_some()
                {
                    return Err("join/target implementation review chain branches".to_string());
                }
            }
            None => roots.push(review.review_id.clone()),
        }
    }
    if roots.len() != 1 {
        return Err("join/target implementation review chain has no single root".to_string());
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = roots.pop().unwrap();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err("join/target implementation review chain contains a cycle".to_string());
        }
        let review = by_id
            .get(&current)
            .ok_or_else(|| "join/target implementation review chain is disconnected".to_string())?
            .clone();
        if let Some(parent_id) = &review.previous_review_id {
            let parent = by_id.get(parent_id).ok_or_else(|| {
                "join/target implementation review predecessor is missing".to_string()
            })?;
            if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                return Err("join/target implementation review predecessor hash drift".to_string());
            }
        }
        if review.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&review.implementation, &ordered)
        {
            return Err(
                "join/target implementation review role-exclusion history mismatch".to_string(),
            );
        }
        if review.future_isolated_join_target_runner_registration_eligible
            && child_by_parent.contains_key(&current)
        {
            return Err("approved join/target implementation review must be terminal".to_string());
        }
        ordered.push(review);
        match child_by_parent.get(&current) {
            Some(next) => current = next.clone(),
            None => break,
        }
    }
    if ordered.len() != reviews.len() {
        return Err("join/target implementation review chain is disconnected".to_string());
    }
    Ok(ordered)
}

async fn read_reviews_for_implementation(
    state: &AppState,
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetImplementationReview>, String> {
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
        let review: HistoricalOutcomeFeatureLabelJoinTargetImplementationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("join/target implementation review path mismatch".to_string());
        }
        reviews.push(review);
    }
    validate_review_chain(&reviews, implementation_id)
}

fn review_fingerprint(
    review: &HistoricalOutcomeFeatureLabelJoinTargetImplementationReview,
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
        .join("historical_outcome_feature_label_join_target_implementation_reviews/reviews")
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
        return Err("join/target 隔离实现 ID 无效".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-outcome-feature-label-join-target-implementation-review-{implementation_id}.lock"
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
                return Err("另一个管理员正在复核 join/target 隔离实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("join/target 隔离实现复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_implementations::tests::record;

    fn request(
        implementation: &HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
        prior: Option<&HistoricalOutcomeFeatureLabelJoinTargetImplementationReview>,
    ) -> ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest {
        let specification_review = &implementation.approved_review;
        let specification = &specification_review.specification;
        let contract = &implementation.implementation_contract;
        let audit = independently_audit_implementation(implementation).unwrap();
        ReviewHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest {
            expected_previous_review_id: prior.map(|review| review.review_id.clone()),
            expected_previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_contract_sha256: contract.contract_sha256.clone(),
            expected_implementation_artifact_sha256: contract.implementation_artifact_sha256.clone(),
            expected_immutable_code_revision: contract.immutable_code_revision.clone(),
            expected_specification_review_sha256: specification_review.review_sha256.clone(),
            expected_specification_review_audit_sha256: specification_review.independent_audit.audit_sha256.clone(),
            expected_specification_sha256: specification.specification_sha256.clone(),
            expected_specification_body_sha256: specification.specification_body_sha256.clone(),
            expected_join_specification_sha256: specification.join_specification.specification_sha256.clone(),
            expected_target_specification_sha256: specification.target_specification.specification_sha256.clone(),
            expected_combined_artifact_sha256: specification.combined_artifact_sha256.clone(),
            expected_dataset_content_sha256: specification.dataset_content_sha256.clone(),
            expected_review_contract_sha256: review_contract().contract_sha256,
            expected_independent_audit_sha256: audit.audit_sha256,
            verdict: HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict::ApprovedForFutureIsolatedJoinTargetRunnerRegistration,
            rationale: "independently reproduced implementation and sandbox contracts".into(),
            known_limitations: "runner is absent and engineering targets remain empirically unproven".into(),
            exact_current_implementation_and_complete_upstream_binding_confirmed: true,
            reviewer_independence_from_complete_prior_chain_confirmed: true,
            implementation_record_and_contract_hashes_independently_reproduced_confirmed: true,
            implementation_artifact_digest_and_code_revision_reproducible_confirmed: true,
            exact_one_to_one_join_and_fail_closed_key_semantics_confirmed: true,
            exact_nine_raw_f64_target_projection_without_transform_confirmed: true,
            point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: true,
            sealed_holdout_labels_inaccessible_to_training_tuning_and_model_selection_confirmed: true,
            canonical_serializer_fixed_schemas_and_resource_limits_confirmed: true,
            no_action_position_threshold_rank_or_reward_semantics_confirmed: true,
            no_entrypoint_environment_secrets_network_tools_child_process_or_data_store_access_confirmed: true,
            review_runner_authorization_execution_output_validation_and_training_separation_confirmed: true,
            no_runner_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn approved_review() -> HistoricalOutcomeFeatureLabelJoinTargetImplementationReview {
        let implementation = record();
        build_review(
            &implementation,
            &[],
            None,
            "join-target-implementation-reviewer",
            request(&implementation, None),
            DateTime::from_timestamp(1_765_000_000, 0).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn independent_audit_reproduces_record_contract_and_exact_semantics() {
        let audit = independently_audit_implementation(&record()).unwrap();
        assert!(audit.mismatch_reasons.is_empty());
        assert!(audit.implementation_record_hash_independently_reproduced);
        assert!(audit.implementation_contract_hash_independently_reproduced);
        assert!(audit.exact_one_to_one_join_implementation_valid);
        assert!(audit.exact_nine_raw_f64_target_projection_valid);
    }

    #[test]
    fn independent_audit_rejects_contract_semantic_or_authority_drift() {
        let mut implementation = record();
        implementation.implementation_contract.exact_target_count = 8;
        let audit = independently_audit_implementation(&implementation).unwrap();
        assert!(!audit.mismatch_reasons.is_empty());
        let mut implementation = record();
        implementation.join_execution_authorized = true;
        let audit = independently_audit_implementation(&implementation).unwrap();
        assert!(
            audit
                .mismatch_reasons
                .contains(&"runner_execution_training_or_trading_authority_open".to_string())
        );
    }

    #[test]
    fn reviewer_must_be_outside_complete_prior_chain() {
        let implementation = record();
        for actor in expected_excluded_prior_actor_ids(&implementation, &[]) {
            assert!(
                build_review(
                    &implementation,
                    &[],
                    None,
                    &actor,
                    request(&implementation, None),
                    Utc::now(),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn approval_requires_every_confirmation_and_exact_audit_binding() {
        let implementation = record();
        let mut missing = request(&implementation, None);
        missing.exact_nine_raw_f64_target_projection_without_transform_confirmed = false;
        assert!(build_review(&implementation, &[], None, "reviewer", missing, Utc::now()).is_err());
        let mut stale = request(&implementation, None);
        stale.expected_independent_audit_sha256 = "0".repeat(64);
        assert!(build_review(&implementation, &[], None, "reviewer", stale, Utc::now()).is_err());
    }

    #[test]
    fn approval_only_allows_future_runner_registration() {
        let review = approved_review();
        assert!(review.future_isolated_join_target_runner_registration_eligible);
        assert!(!review.isolated_runner_registered);
        assert!(!review.label_access_authorized);
        assert!(!review.join_execution_authorized);
        assert!(!review.training_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn one_bit_or_authority_tamper_fails_validation() {
        let mut review = approved_review();
        review.training_data_access_authorized = true;
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review).is_err());
        let mut review = approved_review();
        review.independent_audit.audit_sha256 = "0".repeat(64);
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review).is_err());
    }

    #[test]
    fn review_chain_requires_prior_reviewer_exclusion_and_exact_hash() {
        let implementation = record();
        let mut first_request = request(&implementation, None);
        first_request.verdict =
            HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict::ChangesRequested;
        let first = build_review(
            &implementation,
            &[],
            None,
            "reviewer-a",
            first_request,
            DateTime::from_timestamp(1_765_000_000, 0).unwrap(),
        )
        .unwrap();
        let second = build_review(
            &implementation,
            std::slice::from_ref(&first),
            Some(&first),
            "reviewer-b",
            request(&implementation, Some(&first)),
            DateTime::from_timestamp(1_765_000_100, 0).unwrap(),
        )
        .unwrap();
        assert_eq!(
            validate_review_chain(
                &[second.clone(), first.clone()],
                &implementation.implementation_id,
            )
            .unwrap()
            .len(),
            2
        );
        let mut drift = second;
        drift.previous_review_sha256 = Some("0".repeat(64));
        drift.review_sha256 = review_fingerprint(&drift).unwrap();
        drift.review_id = drift.review_sha256[..32].to_string();
        assert!(validate_review_chain(&[first, drift], &implementation.implementation_id).is_err());
    }

    #[test]
    fn approved_review_is_terminal_even_if_chain_is_tampered() {
        let implementation = record();
        let approved = approved_review();
        let mut next_request = request(&implementation, Some(&approved));
        next_request.verdict =
            HistoricalOutcomeFeatureLabelJoinTargetImplementationReviewVerdict::ChangesRequested;
        let next = build_review(
            &implementation,
            std::slice::from_ref(&approved),
            Some(&approved),
            "reviewer-after-approval",
            next_request,
            Utc::now(),
        )
        .unwrap();
        assert!(
            validate_review_chain(&[approved, next], &implementation.implementation_id).is_err()
        );
    }

    #[test]
    fn review_contract_keeps_targets_as_engineering_candidates_and_all_gates_separate() {
        let contract = review_contract();
        assert!(contract.targets_remain_engineering_candidates_not_strategy_truth);
        assert!(contract.runner_registration_separate);
        assert!(contract.first_execution_authorization_separate);
        assert!(contract.join_execution_separate);
        assert!(contract.output_validation_separate);
        assert!(contract.training_and_reward_governance_separate);
        assert!(valid_sha256(&contract.contract_sha256));
    }
}
