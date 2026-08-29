//! Independent review of one immutable Stage 53 training-suite implementation.
//!
//! Approval opens only future registration of a separate isolated runner specification. This
//! module has no callable implementation entrypoint, data access, training, model selection,
//! reward, shadow portfolio, order, broker or trading capability.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_implementations::{
    HistoricalOutcomeTrainingImplementationRecord,
    independent_review_eligible_historical_outcome_training_implementations,
    validate_historical_outcome_training_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-training-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-training-implementation-independent-review-v1";
const POLICY_VERSION: &str = "hone-training-implementation-independent-review-v1-no-execution";
const REVIEW_CONTRACT_SCHEMA_VERSION: &str =
    "hone-training-implementation-independent-review-contract-v1";
const AUDIT_SCHEMA_VERSION: &str = "hone-training-implementation-independent-audit-v1";
const EXPECTED_IMPLEMENTATION_SCHEMA_VERSION: &str =
    "hone-historical-outcome-training-implementation-v1";
const EXPECTED_IMPLEMENTATION_POLICY_VERSION: &str =
    "hone-training-implementation-registration-v1-no-execution";
const EXPECTED_IMPLEMENTATION_STATUS: &str = "registered_not_reviewed_not_run";
const EXPECTED_CONTRACT_SCHEMA_VERSION: &str = "hone-isolated-training-implementation-contract-v1";
const EXPECTED_SUITE_VERSION: &str = "hone-nine-target-comparison-suite-v1";
const EXPECTED_TARGET_CONTRACT_VERSION: &str = "hone-nine-raw-continuous-outcomes-v1";
const EXPECTED_INPUT_SCHEMA_VERSION: &str = "hone-reviewed-training-store-dataset-v1";
const EXPECTED_OUTPUT_SCHEMA_VERSION: &str = "hone-unrun-training-artifact-envelope-v1";
const EXPECTED_SERIALIZER_VERSION: &str = "hone-canonical-json-serializer-v1";
const EXPECTED_PREPROCESSOR_VERSION: &str =
    "v1-train-only-scaling-explicit-missingness-no-imputation";
const EXPECTED_FEATURE_COUNT: u16 = 65;
const EXPECTED_TARGET_COUNT: u8 = 9;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const EXPECTED_MAXIMUM_WALL_CLOCK_SECONDS: u32 = 3_600;
const EXPECTED_MAXIMUM_MEMORY_MIB: u32 = 8_192;
const EXPECTED_MAXIMUM_CPU_MILLICORES: u32 = 4_000;
const EXPECTED_MAXIMUM_PROCESS_COUNT: u16 = 4;
const EXPECTED_MAXIMUM_OUTPUT_BYTES: u64 = 256 * 1024 * 1024;
const MAX_RATIONALE_CHARS: usize = 3_000;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeTrainingImplementationReviewVerdict {
    ApprovedForFutureIsolatedTrainingRunnerRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingImplementationReviewContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub independent_audit_implementation: String,
    pub required_fingerprint_checks: Vec<String>,
    pub required_training_semantic_checks: Vec<String>,
    pub required_sandbox_checks: Vec<String>,
    pub approval_scope: String,
    pub runner_registration_separate: bool,
    pub data_access_authorization_separate: bool,
    pub training_execution_separate: bool,
    pub output_validation_separate: bool,
    pub reward_governance_separate: bool,
    pub targets_remain_engineering_candidates_not_strategy_truth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub implementation_artifact_sha256: String,
    pub immutable_code_revision: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub exact_stage_52_review_and_stage_51_chain_binding_valid: bool,
    pub immutable_artifact_and_code_revision_valid: bool,
    pub fixed_three_arm_three_seed_contract_valid: bool,
    pub exact_65_feature_nine_raw_continuous_target_contract_valid: bool,
    pub train_only_preprocessing_and_fit_valid: bool,
    pub validation_only_selection_and_sealed_holdout_isolation_valid: bool,
    pub per_target_per_seed_metrics_without_composite_masking_valid: bool,
    pub deterministic_replay_and_fixed_resource_ceilings_valid: bool,
    pub no_scalar_reward_action_position_or_ranking_semantics: bool,
    pub no_entrypoint_environment_secret_network_tool_child_process_or_data_access: bool,
    pub all_runner_training_artifact_metric_reward_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeTrainingImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_implementation_artifact_sha256: String,
    expected_immutable_code_revision: String,
    expected_stage_52_review_sha256: String,
    expected_stage_51_registration_sha256: String,
    expected_stage_51_claim_sha256: String,
    expected_stage_51_result_sha256: String,
    expected_suite_specification_sha256: String,
    expected_review_contract_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: HistoricalOutcomeTrainingImplementationReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_implementation_and_complete_upstream_binding_confirmed: bool,
    reviewer_independence_from_registrar_and_complete_prior_chain_confirmed: bool,
    implementation_record_and_contract_hashes_independently_reproduced_confirmed: bool,
    immutable_artifact_digest_and_code_revision_reproducible_confirmed: bool,
    fixed_three_arm_three_seed_implementation_confirmed: bool,
    exact_65_feature_nine_raw_continuous_target_contract_confirmed: bool,
    train_only_preprocessing_and_fit_confirmed: bool,
    validation_only_selection_and_sealed_holdout_isolation_confirmed: bool,
    per_target_per_seed_metrics_without_composite_masking_confirmed: bool,
    deterministic_replay_and_fixed_resource_ceilings_confirmed: bool,
    no_scalar_reward_action_position_or_ranking_semantics_confirmed: bool,
    no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed: bool,
    review_runner_data_access_training_output_validation_and_reward_separation_confirmed: bool,
    no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed:
        bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingImplementationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: HistoricalOutcomeTrainingImplementationRecord,
    pub review_contract: HistoricalOutcomeTrainingImplementationReviewContract,
    pub independent_audit: HistoricalOutcomeTrainingImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub verdict: HistoricalOutcomeTrainingImplementationReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_current_implementation_and_complete_upstream_binding_confirmed: bool,
    pub reviewer_independence_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub implementation_record_and_contract_hashes_independently_reproduced_confirmed: bool,
    pub immutable_artifact_digest_and_code_revision_reproducible_confirmed: bool,
    pub fixed_three_arm_three_seed_implementation_confirmed: bool,
    pub exact_65_feature_nine_raw_continuous_target_contract_confirmed: bool,
    pub train_only_preprocessing_and_fit_confirmed: bool,
    pub validation_only_selection_and_sealed_holdout_isolation_confirmed: bool,
    pub per_target_per_seed_metrics_without_composite_masking_confirmed: bool,
    pub deterministic_replay_and_fixed_resource_ceilings_confirmed: bool,
    pub no_scalar_reward_action_position_or_ranking_semantics_confirmed: bool,
    pub no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed:
        bool,
    pub review_runner_data_access_training_output_validation_and_reward_separation_confirmed: bool,
    pub no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub training_implementation_independently_approved: bool,
    pub future_isolated_training_runner_registration_eligible: bool,
    pub isolated_training_runner_registered: bool,
    pub data_access_authorization_review_eligible: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub validation_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub model_artifact_created: bool,
    pub metrics_created: bool,
    pub output_validation_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingImplementationReviewItem {
    pub implementation: HistoricalOutcomeTrainingImplementationRecord,
    pub current_independent_audit: HistoricalOutcomeTrainingImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub upstream_binding_current: bool,
    pub latest_review: Option<HistoricalOutcomeTrainingImplementationReview>,
    pub review_eligible: bool,
    pub future_isolated_training_runner_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_contract: HistoricalOutcomeTrainingImplementationReviewContract,
    pub items: Vec<HistoricalOutcomeTrainingImplementationReviewItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub current_binding_approved_count: usize,
    pub future_isolated_runner_registration_eligible_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub review_status: String,
    pub isolated_training_runner_registered: bool,
    pub data_access_authorization_review_eligible: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub validation_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub model_artifact_created: bool,
    pub metrics_created: bool,
    pub output_validation_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeTrainingImplementationReviewReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub current_binding_approved_count: usize,
    pub future_isolated_runner_registration_eligible_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeTrainingImplementationReview {
    pub implementation: HistoricalOutcomeTrainingImplementationRecord,
    pub review: HistoricalOutcomeTrainingImplementationReview,
}

pub(crate) async fn handle_get_historical_outcome_training_implementation_reviews(
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
            warn!(%error, "training implementation review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "训练实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_training_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeTrainingImplementationRequest>,
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

pub(crate) async fn historical_outcome_training_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingImplementationReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeTrainingImplementationReviewReadinessSummary {
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            current_binding_approved_count: registry.current_binding_approved_count,
            future_isolated_runner_registration_eligible_count: registry
                .future_isolated_runner_registration_eligible_count,
            changes_requested_or_rejected_count: registry.changes_requested_or_rejected_count,
            review_status: registry.review_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn approved_historical_outcome_training_implementation_reviews(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeTrainingImplementationReview>, String> {
    let implementations =
        independent_review_eligible_historical_outcome_training_implementations(state).await?;
    let mut approved = Vec::new();
    for implementation in implementations {
        let reviews =
            read_reviews_for_implementation(state, &implementation.implementation_id).await?;
        let Some(review) = reviews.last().cloned() else {
            continue;
        };
        if review.implementation != implementation
            || !review.future_isolated_training_runner_registration_eligible
        {
            continue;
        }
        validate_historical_outcome_training_implementation_review_for_runner_registration(
            &review,
        )?;
        approved.push(ApprovedHistoricalOutcomeTrainingImplementationReview {
            implementation,
            review,
        });
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

#[allow(dead_code)]
pub(crate) fn validate_historical_outcome_training_implementation_review_for_runner_registration(
    review: &HistoricalOutcomeTrainingImplementationReview,
) -> Result<(), String> {
    validate_review(review)?;
    if !review.future_isolated_training_runner_registration_eligible {
        return Err("训练实现复核未批准未来隔离 runner 规格登记".to_string());
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingImplementationReviewRegistry, String> {
    let implementations =
        independent_review_eligible_historical_outcome_training_implementations(state).await?;
    let mut items = Vec::with_capacity(implementations.len());
    for implementation in implementations {
        let audit = independently_audit_implementation(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("当前训练实现独立审计失败，晋级已关闭".to_string());
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
            return Err("训练实现复核绑定或独立审计已变化".to_string());
        }
        let approved = latest_review
            .as_ref()
            .is_some_and(|review| review.future_isolated_training_runner_registration_eligible);
        items.push(HistoricalOutcomeTrainingImplementationReviewItem {
            implementation,
            current_independent_audit: audit,
            complete_review_actor_ids,
            upstream_binding_current: true,
            latest_review,
            review_eligible: !approved,
            future_isolated_training_runner_registration_eligible: approved,
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
                .is_some_and(|review| review.future_isolated_training_runner_registration_eligible)
        })
        .count();
    let current_binding_approved_count = items
        .iter()
        .filter(|item| item.future_isolated_training_runner_registration_eligible)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review
                .as_ref()
                .is_some_and(|review| !review.future_isolated_training_runner_registration_eligible)
        })
        .count();
    let review_status = if items.is_empty() {
        "waiting_current_registered_training_implementation"
    } else if current_binding_approved_count > 0 {
        "current_training_implementation_approved_only_for_future_isolated_runner_registration"
    } else if review_eligible_count > 0 {
        "current_training_implementation_ready_for_independent_review"
    } else {
        "training_implementation_review_blocked_fail_closed"
    };
    Ok(HistoricalOutcomeTrainingImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_contract: review_contract(),
        items,
        review_eligible_count,
        reviewed_count,
        approved_count,
        current_binding_approved_count,
        future_isolated_runner_registration_eligible_count: current_binding_approved_count,
        changes_requested_or_rejected_count,
        review_status: review_status.to_string(),
        isolated_training_runner_registered: false,
        data_access_authorization_review_eligible: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_started: false,
        validation_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_created: false,
        metrics_created: false,
        output_validation_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 54 阶段由完整 Stage 51–53 链之外的新角色，独立重算训练实现记录与合同指纹，并复核三模型臂、三种子、65 项特征、九项原始连续目标、train-only 拟合、validation-only 选择、sealed holdout 隔离、逐目标指标、资源上限和零能力边界。批准只开放未来隔离 runner 规格登记；本阶段不登记 runner、不读取数据、不训练、不产出模型或指标、不定义奖励、不影子、不下单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewHistoricalOutcomeTrainingImplementationRequest,
) -> Result<HistoricalOutcomeTrainingImplementationReview, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("训练实现复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementations =
        independent_review_eligible_historical_outcome_training_implementations(state).await?;
    let implementation = implementations
        .iter()
        .find(|value| value.implementation_id == implementation_id)
        .ok_or_else(|| "当前训练实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews_for_implementation(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|review| review.future_isolated_training_runner_registration_eligible) {
        return Err("当前训练实现已经独立批准，不得追加或分叉复核".to_string());
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
    implementation: &HistoricalOutcomeTrainingImplementationRecord,
    prior_reviews: &[HistoricalOutcomeTrainingImplementationReview],
    prior: Option<&HistoricalOutcomeTrainingImplementationReview>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeTrainingImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeTrainingImplementationReview, String> {
    validate_historical_outcome_training_implementation_for_review(implementation)?;
    let independent_audit = independently_audit_implementation(implementation)?;
    if !independent_audit.mismatch_reasons.is_empty() {
        return Err("训练实现独立审计不通过".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let stage_52 = &implementation.approved_registration_review;
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
        || request.expected_stage_52_review_sha256 != stage_52.review_sha256
        || request.expected_stage_51_registration_sha256 != stage_52.registration_sha256
        || request.expected_stage_51_claim_sha256 != stage_52.claim_sha256
        || request.expected_stage_51_result_sha256 != stage_52.result_sha256
        || request.expected_suite_specification_sha256 != stage_52.suite_specification_sha256
        || request.expected_review_contract_sha256 != review_contract.contract_sha256
        || request.expected_independent_audit_sha256 != independent_audit.audit_sha256
    {
        return Err("训练实现、完整上游、独立审计或复核合同已变化，请刷新后重试".to_string());
    }
    if prior.is_some_and(|review| review.implementation != *implementation) {
        return Err("上一条复核绑定的训练实现与当前实现不一致".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == reviewer_id)
    {
        return Err("复核人参与过登记、完整上游或此前复核链，必须更换独立角色".to_string());
    }
    let approval = request.verdict
        == HistoricalOutcomeTrainingImplementationReviewVerdict::ApprovedForFutureIsolatedTrainingRunnerRegistration;
    let all_confirmed = request_all_checks(&request);
    if approval && !all_confirmed {
        return Err("批准训练实现前必须逐项确认指纹、训练语义、沙箱和零执行边界".to_string());
    }
    let mut review = HistoricalOutcomeTrainingImplementationReview {
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
        reviewer_independent_from_registrar_and_complete_prior_chain: true,
        verdict: request.verdict,
        rationale,
        known_limitations,
        exact_current_implementation_and_complete_upstream_binding_confirmed: request
            .exact_current_implementation_and_complete_upstream_binding_confirmed,
        reviewer_independence_from_registrar_and_complete_prior_chain_confirmed: request
            .reviewer_independence_from_registrar_and_complete_prior_chain_confirmed,
        implementation_record_and_contract_hashes_independently_reproduced_confirmed: request
            .implementation_record_and_contract_hashes_independently_reproduced_confirmed,
        immutable_artifact_digest_and_code_revision_reproducible_confirmed: request
            .immutable_artifact_digest_and_code_revision_reproducible_confirmed,
        fixed_three_arm_three_seed_implementation_confirmed: request
            .fixed_three_arm_three_seed_implementation_confirmed,
        exact_65_feature_nine_raw_continuous_target_contract_confirmed: request
            .exact_65_feature_nine_raw_continuous_target_contract_confirmed,
        train_only_preprocessing_and_fit_confirmed: request
            .train_only_preprocessing_and_fit_confirmed,
        validation_only_selection_and_sealed_holdout_isolation_confirmed: request
            .validation_only_selection_and_sealed_holdout_isolation_confirmed,
        per_target_per_seed_metrics_without_composite_masking_confirmed: request
            .per_target_per_seed_metrics_without_composite_masking_confirmed,
        deterministic_replay_and_fixed_resource_ceilings_confirmed: request
            .deterministic_replay_and_fixed_resource_ceilings_confirmed,
        no_scalar_reward_action_position_or_ranking_semantics_confirmed: request
            .no_scalar_reward_action_position_or_ranking_semantics_confirmed,
        no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed:
            request.no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed,
        review_runner_data_access_training_output_validation_and_reward_separation_confirmed:
            request.review_runner_data_access_training_output_validation_and_reward_separation_confirmed,
        no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed:
            request.no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed,
        training_implementation_independently_approved: approval && all_confirmed,
        future_isolated_training_runner_registration_eligible: approval && all_confirmed,
        isolated_training_runner_registered: false,
        data_access_authorization_review_eligible: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_started: false,
        validation_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_created: false,
        metrics_created: false,
        output_validation_authorized: false,
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

fn request_all_checks(request: &ReviewHistoricalOutcomeTrainingImplementationRequest) -> bool {
    request.exact_current_implementation_and_complete_upstream_binding_confirmed
        && request.reviewer_independence_from_registrar_and_complete_prior_chain_confirmed
        && request.implementation_record_and_contract_hashes_independently_reproduced_confirmed
        && request.immutable_artifact_digest_and_code_revision_reproducible_confirmed
        && request.fixed_three_arm_three_seed_implementation_confirmed
        && request.exact_65_feature_nine_raw_continuous_target_contract_confirmed
        && request.train_only_preprocessing_and_fit_confirmed
        && request.validation_only_selection_and_sealed_holdout_isolation_confirmed
        && request.per_target_per_seed_metrics_without_composite_masking_confirmed
        && request.deterministic_replay_and_fixed_resource_ceilings_confirmed
        && request.no_scalar_reward_action_position_or_ranking_semantics_confirmed
        && request.no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed
        && request.review_runner_data_access_training_output_validation_and_reward_separation_confirmed
        && request.no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed
}

fn review_all_checks(review: &HistoricalOutcomeTrainingImplementationReview) -> bool {
    review.exact_current_implementation_and_complete_upstream_binding_confirmed
        && review.reviewer_independence_from_registrar_and_complete_prior_chain_confirmed
        && review.implementation_record_and_contract_hashes_independently_reproduced_confirmed
        && review.immutable_artifact_digest_and_code_revision_reproducible_confirmed
        && review.fixed_three_arm_three_seed_implementation_confirmed
        && review.exact_65_feature_nine_raw_continuous_target_contract_confirmed
        && review.train_only_preprocessing_and_fit_confirmed
        && review.validation_only_selection_and_sealed_holdout_isolation_confirmed
        && review.per_target_per_seed_metrics_without_composite_masking_confirmed
        && review.deterministic_replay_and_fixed_resource_ceilings_confirmed
        && review.no_scalar_reward_action_position_or_ranking_semantics_confirmed
        && review.no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed
        && review.review_runner_data_access_training_output_validation_and_reward_separation_confirmed
        && review.no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed
}

fn independently_audit_implementation(
    implementation: &HistoricalOutcomeTrainingImplementationRecord,
) -> Result<HistoricalOutcomeTrainingImplementationIndependentAudit, String> {
    let contract = &implementation.implementation_contract;
    let stage_52 = &implementation.approved_registration_review;
    let record_hash = valid_id(&implementation.implementation_id)
        && valid_sha256(&implementation.implementation_sha256)
        && implementation.implementation_id == implementation.implementation_sha256[..32]
        && implementation.implementation_sha256
            == fingerprint_without(
                implementation,
                &["implementation_id", "implementation_sha256"],
            )?;
    let contract_hash = valid_sha256(&contract.contract_sha256)
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?;
    let exact_binding = stage_52.future_training_implementation_registration_eligible
        && stage_52.training_experiment_registration_independently_approved
        && valid_id(&stage_52.attempt_id)
        && valid_id(&stage_52.registration_id)
        && valid_sha256(&stage_52.registration_sha256)
        && valid_sha256(&stage_52.claim_sha256)
        && valid_id(&stage_52.result_id)
        && valid_sha256(&stage_52.result_sha256)
        && stage_52.suite_specification_sha256 == contract.suite_specification_sha256;
    let artifact_revision = valid_sha256(&contract.implementation_artifact_sha256)
        && !contract.immutable_code_revision.trim().is_empty();
    let expected_arms = [
        "frozen_zero_prediction_baseline@v1-no-fit",
        "ridge_multi_target_regression@v1-train-only-fit",
        "gradient_boosted_multi_target_regression@v1-deterministic-train-only-fit",
    ];
    let fixed_arms_seeds = contract.schema_version == EXPECTED_CONTRACT_SCHEMA_VERSION
        && contract.suite_version == EXPECTED_SUITE_VERSION
        && contract.algorithm_implementation_versions
            == expected_arms.map(ToString::to_string).to_vec()
        && contract.exact_random_seeds == EXPECTED_SEEDS;
    let exact_features_targets = contract.target_contract_version
        == EXPECTED_TARGET_CONTRACT_VERSION
        && contract.input_schema_version == EXPECTED_INPUT_SCHEMA_VERSION
        && contract.output_schema_version == EXPECTED_OUTPUT_SCHEMA_VERSION
        && contract.canonical_serializer_version == EXPECTED_SERIALIZER_VERSION
        && contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == EXPECTED_TARGET_COUNT;
    let train_only = contract.preprocessor_implementation_version == EXPECTED_PREPROCESSOR_VERSION
        && implementation.train_only_preprocessing_and_fit_confirmed
        && contract.input_contract.contains("拟合与预处理只使用 train")
        && !contract.validation_labels_visible_to_fit_worker;
    let validation_holdout = implementation
        .validation_selection_and_sealed_holdout_isolation_confirmed
        && contract
            .input_contract
            .contains("validation 仅用于候选选择")
        && contract
            .input_contract
            .contains("sealed holdout 对拟合和选择不可见")
        && !contract.sealed_holdout_labels_visible_to_fit_or_selection_worker;
    let expected_metrics = [
        "mae_per_target_per_seed",
        "spearman_per_target_per_seed",
        "directional_accuracy_per_target_per_seed",
        "calibration_slope_per_target_per_seed",
    ];
    let per_target_metrics = implementation
        .per_target_per_seed_metrics_without_composite_masking_confirmed
        && contract.reported_metric_ids == expected_metrics.map(ToString::to_string).to_vec()
        && contract.output_contract.contains("按模型臂和随机种子分离")
        && contract.output_contract.contains("逐目标指标")
        && contract.output_contract.contains("不得折叠为 reward");
    let deterministic_resources = implementation
        .deterministic_replay_and_fixed_resource_ceilings_confirmed
        && contract.maximum_wall_clock_seconds == EXPECTED_MAXIMUM_WALL_CLOCK_SECONDS
        && contract.maximum_memory_mib == EXPECTED_MAXIMUM_MEMORY_MIB
        && contract.maximum_cpu_millicores == EXPECTED_MAXIMUM_CPU_MILLICORES
        && contract.maximum_process_count == EXPECTED_MAXIMUM_PROCESS_COUNT
        && contract.maximum_output_bytes == EXPECTED_MAXIMUM_OUTPUT_BYTES;
    let no_semantics = implementation
        .no_scalar_reward_action_position_or_ranking_semantics_confirmed
        && !contract.scalar_reward_defined
        && !contract.action_position_or_ranking_semantics_defined;
    let sandbox_closed = !contract.callable_entrypoint_present
        && !contract.ambient_environment_available
        && !contract.environment_variables_allowed
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.external_tools_allowed
        && !contract.child_process_allowed
        && !contract.training_store_reads_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed;
    let authority_closed = implementation.schema_version == EXPECTED_IMPLEMENTATION_SCHEMA_VERSION
        && implementation.policy_version == EXPECTED_IMPLEMENTATION_POLICY_VERSION
        && implementation.status == EXPECTED_IMPLEMENTATION_STATUS
        && implementation.training_implementation_registered
        && implementation.future_independent_implementation_review_eligible
        && !implementation.independent_implementation_review_completed
        && !implementation.isolated_runner_registration_eligible
        && !implementation.training_data_access_authorized
        && !implementation.training_authorized
        && !implementation.training_started
        && !implementation.validation_selection_authorized
        && !implementation.sealed_holdout_access_authorized
        && !implementation.model_artifact_created
        && !implementation.metrics_created
        && !implementation.reward_authorized
        && !implementation.shadow_portfolio_authorized
        && !implementation.order_generation_authorized
        && !implementation.broker_access_authorized
        && !implementation.trading_authorized;
    let checks = [
        (record_hash, "implementation_record_hash_mismatch"),
        (contract_hash, "implementation_contract_hash_mismatch"),
        (exact_binding, "stage_51_52_binding_mismatch"),
        (artifact_revision, "artifact_or_revision_invalid"),
        (fixed_arms_seeds, "three_arm_three_seed_contract_mismatch"),
        (
            exact_features_targets,
            "feature_or_target_contract_mismatch",
        ),
        (train_only, "train_only_preprocessing_or_fit_mismatch"),
        (
            validation_holdout,
            "validation_or_sealed_holdout_isolation_mismatch",
        ),
        (
            per_target_metrics,
            "per_target_per_seed_metric_contract_mismatch",
        ),
        (
            deterministic_resources,
            "determinism_or_resource_ceiling_mismatch",
        ),
        (
            no_semantics,
            "reward_action_position_or_rank_semantics_present",
        ),
        (sandbox_closed, "sandbox_or_data_access_open"),
        (
            authority_closed,
            "runner_training_artifact_metric_or_trading_authority_open",
        ),
    ];
    let mismatch_reasons = checks
        .into_iter()
        .filter(|(passed, _)| !passed)
        .map(|(_, reason)| reason.to_string())
        .collect::<Vec<_>>();
    let mut audit = HistoricalOutcomeTrainingImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        implementation_artifact_sha256: contract.implementation_artifact_sha256.clone(),
        immutable_code_revision: contract.immutable_code_revision.clone(),
        implementation_record_hash_independently_reproduced: record_hash,
        implementation_contract_hash_independently_reproduced: contract_hash,
        exact_stage_52_review_and_stage_51_chain_binding_valid: exact_binding,
        immutable_artifact_and_code_revision_valid: artifact_revision,
        fixed_three_arm_three_seed_contract_valid: fixed_arms_seeds,
        exact_65_feature_nine_raw_continuous_target_contract_valid: exact_features_targets,
        train_only_preprocessing_and_fit_valid: train_only,
        validation_only_selection_and_sealed_holdout_isolation_valid: validation_holdout,
        per_target_per_seed_metrics_without_composite_masking_valid: per_target_metrics,
        deterministic_replay_and_fixed_resource_ceilings_valid: deterministic_resources,
        no_scalar_reward_action_position_or_ranking_semantics: no_semantics,
        no_entrypoint_environment_secret_network_tool_child_process_or_data_access: sandbox_closed,
        all_runner_training_artifact_metric_reward_and_trading_authority_closed: authority_closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn review_contract() -> HistoricalOutcomeTrainingImplementationReviewContract {
    let mut contract = HistoricalOutcomeTrainingImplementationReviewContract {
        schema_version: REVIEW_CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        independent_audit_implementation:
            "hone-independent-training-implementation-audit-v1-no-registration-validator-hash-reuse"
                .to_string(),
        required_fingerprint_checks: vec![
            "implementation_record_sha256_independently_reproduced".to_string(),
            "implementation_contract_sha256_independently_reproduced".to_string(),
            "artifact_sha256_and_code_revision_reproducible".to_string(),
            "exact_stage_51_52_chain_binding".to_string(),
        ],
        required_training_semantic_checks: vec![
            "fixed_three_model_arms_and_seeds_17_29_43".to_string(),
            "exact_65_features_and_nine_raw_continuous_targets".to_string(),
            "train_only_preprocessing_and_fit".to_string(),
            "validation_only_selection_and_sealed_holdout_isolation".to_string(),
            "per_target_per_seed_metrics_without_composite_masking".to_string(),
            "no_scalar_reward_action_position_or_ranking_semantics".to_string(),
        ],
        required_sandbox_checks: vec![
            "deterministic_replay_and_fixed_resource_ceilings".to_string(),
            "no_entrypoint_environment_secret_network_tool_or_child_process".to_string(),
            "no_training_store_or_production_data_access".to_string(),
            "no_runner_training_artifact_metrics_reward_shadow_order_broker_or_trading".to_string(),
        ],
        approval_scope:
            "future_isolated_training_runner_specification_registration_only_no_execution"
                .to_string(),
        runner_registration_separate: true,
        data_access_authorization_separate: true,
        training_execution_separate: true,
        output_validation_separate: true,
        reward_governance_separate: true,
        targets_remain_engineering_candidates_not_strategy_truth: true,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static training implementation review contract is serializable");
    contract
}

fn expected_excluded_prior_actor_ids(
    implementation: &HistoricalOutcomeTrainingImplementationRecord,
    prior_reviews: &[HistoricalOutcomeTrainingImplementationReview],
) -> Vec<String> {
    let mut actors = implementation.excluded_prior_actor_ids.clone();
    actors.push(implementation.registered_by.clone());
    actors.push(
        implementation
            .approved_registration_review
            .reviewer_id
            .clone(),
    );
    actors.extend(
        prior_reviews
            .iter()
            .map(|review| review.reviewer_id.clone()),
    );
    actors.sort();
    actors.dedup();
    actors
}

fn validate_review(review: &HistoricalOutcomeTrainingImplementationReview) -> Result<(), String> {
    validate_historical_outcome_training_implementation_for_review(&review.implementation)?;
    let expected_audit = independently_audit_implementation(&review.implementation)?;
    let approval = review.verdict
        == HistoricalOutcomeTrainingImplementationReviewVerdict::ApprovedForFutureIsolatedTrainingRunnerRegistration;
    let authority_closed = !review.isolated_training_runner_registered
        && !review.data_access_authorization_review_eligible
        && !review.training_data_access_authorized
        && !review.training_authorized
        && !review.training_started
        && !review.validation_selection_authorized
        && !review.sealed_holdout_access_authorized
        && !review.model_artifact_created
        && !review.metrics_created
        && !review.output_validation_authorized
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
        || review.independent_audit != expected_audit
        || !review.independent_audit.mismatch_reasons.is_empty()
        || !review
            .independent_audit
            .all_runner_training_artifact_metric_reward_and_trading_authority_closed
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review.reviewer_id.trim().is_empty()
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || !review.reviewer_independent_from_registrar_and_complete_prior_chain
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || review.known_limitations.trim().is_empty()
        || review.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || (approval && !review_all_checks(review))
        || review.training_implementation_independently_approved
            != (approval && review_all_checks(review))
        || review.future_isolated_training_runner_registration_eligible
            != (approval && review_all_checks(review))
        || !authority_closed
    {
        return Err("training implementation review is invalid or over-authorized".to_string());
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
        return Err("training implementation review predecessor is invalid".to_string());
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[HistoricalOutcomeTrainingImplementationReview],
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeTrainingImplementationReview>, String> {
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
            return Err("training implementation review replay or subject mismatch".to_string());
        }
        match &review.previous_review_id {
            Some(parent) => {
                if child_by_parent
                    .insert(parent.clone(), review.review_id.clone())
                    .is_some()
                {
                    return Err("training implementation review chain branches".to_string());
                }
            }
            None => roots.push(review.review_id.clone()),
        }
    }
    if roots.len() != 1 {
        return Err("training implementation review chain has no single root".to_string());
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = roots.pop().unwrap();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err("training implementation review chain contains a cycle".to_string());
        }
        let review = by_id
            .get(&current)
            .ok_or_else(|| "training implementation review chain is disconnected".to_string())?
            .clone();
        if let Some(parent_id) = &review.previous_review_id {
            let parent = by_id.get(parent_id).ok_or_else(|| {
                "training implementation review predecessor is missing".to_string()
            })?;
            if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                return Err("training implementation review predecessor hash drift".to_string());
            }
        }
        if review.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&review.implementation, &ordered)
        {
            return Err(
                "training implementation review role-exclusion history mismatch".to_string(),
            );
        }
        if review.future_isolated_training_runner_registration_eligible
            && child_by_parent.contains_key(&current)
        {
            return Err("approved training implementation review must be terminal".to_string());
        }
        ordered.push(review);
        match child_by_parent.get(&current) {
            Some(next) => current = next.clone(),
            None => break,
        }
    }
    if ordered.len() != reviews.len() {
        return Err("training implementation review chain is disconnected".to_string());
    }
    Ok(ordered)
}

async fn read_reviews_for_implementation(
    state: &AppState,
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeTrainingImplementationReview>, String> {
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
        let review: HistoricalOutcomeTrainingImplementationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("training implementation review path mismatch".to_string());
        }
        reviews.push(review);
    }
    validate_review_chain(&reviews, implementation_id)
}

fn review_fingerprint(
    review: &HistoricalOutcomeTrainingImplementationReview,
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
    decision_root(state).join("historical_outcome_training_implementation_reviews/reviews")
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
        return Err("训练实现 ID 无效".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-outcome-training-implementation-review-{implementation_id}.lock"
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
                return Err("另一个管理员正在复核训练实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("训练实现复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_implementations::tests::record;

    fn request(
        implementation: &HistoricalOutcomeTrainingImplementationRecord,
        prior: Option<&HistoricalOutcomeTrainingImplementationReview>,
    ) -> ReviewHistoricalOutcomeTrainingImplementationRequest {
        let stage_52 = &implementation.approved_registration_review;
        let contract = &implementation.implementation_contract;
        let audit = independently_audit_implementation(implementation).unwrap();
        ReviewHistoricalOutcomeTrainingImplementationRequest {
            expected_previous_review_id: prior.map(|review| review.review_id.clone()),
            expected_previous_review_sha256: prior.map(|review| review.review_sha256.clone()),
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_contract_sha256: contract.contract_sha256.clone(),
            expected_implementation_artifact_sha256: contract.implementation_artifact_sha256.clone(),
            expected_immutable_code_revision: contract.immutable_code_revision.clone(),
            expected_stage_52_review_sha256: stage_52.review_sha256.clone(),
            expected_stage_51_registration_sha256: stage_52.registration_sha256.clone(),
            expected_stage_51_claim_sha256: stage_52.claim_sha256.clone(),
            expected_stage_51_result_sha256: stage_52.result_sha256.clone(),
            expected_suite_specification_sha256: stage_52.suite_specification_sha256.clone(),
            expected_review_contract_sha256: review_contract().contract_sha256,
            expected_independent_audit_sha256: audit.audit_sha256,
            verdict: HistoricalOutcomeTrainingImplementationReviewVerdict::ApprovedForFutureIsolatedTrainingRunnerRegistration,
            rationale: "独立重算训练实现及全部隔离边界，仅准入未来 runner 规格登记".into(),
            known_limitations: "未创建 runner、未读数据、未训练，不证明模型质量、收益或可交易性".into(),
            exact_current_implementation_and_complete_upstream_binding_confirmed: true,
            reviewer_independence_from_registrar_and_complete_prior_chain_confirmed: true,
            implementation_record_and_contract_hashes_independently_reproduced_confirmed: true,
            immutable_artifact_digest_and_code_revision_reproducible_confirmed: true,
            fixed_three_arm_three_seed_implementation_confirmed: true,
            exact_65_feature_nine_raw_continuous_target_contract_confirmed: true,
            train_only_preprocessing_and_fit_confirmed: true,
            validation_only_selection_and_sealed_holdout_isolation_confirmed: true,
            per_target_per_seed_metrics_without_composite_masking_confirmed: true,
            deterministic_replay_and_fixed_resource_ceilings_confirmed: true,
            no_scalar_reward_action_position_or_ranking_semantics_confirmed: true,
            no_entrypoint_environment_secrets_network_tools_child_process_or_data_access_confirmed: true,
            review_runner_data_access_training_output_validation_and_reward_separation_confirmed: true,
            no_runner_data_access_training_artifact_metrics_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn approved_review() -> HistoricalOutcomeTrainingImplementationReview {
        let implementation = record();
        build_review(
            &implementation,
            &[],
            None,
            "stage-54-independent-reviewer",
            request(&implementation, None),
            DateTime::from_timestamp(1_765_100_000, 0).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn independent_audit_reproduces_hashes_and_training_contract() {
        let audit = independently_audit_implementation(&record()).unwrap();
        assert!(audit.mismatch_reasons.is_empty());
        assert!(audit.implementation_record_hash_independently_reproduced);
        assert!(audit.implementation_contract_hash_independently_reproduced);
        assert!(audit.fixed_three_arm_three_seed_contract_valid);
        assert!(audit.exact_65_feature_nine_raw_continuous_target_contract_valid);
    }

    #[test]
    fn independent_audit_rejects_semantic_or_authority_drift() {
        let mut implementation = record();
        implementation.implementation_contract.exact_random_seeds = vec![17, 29, 44];
        assert!(
            !independently_audit_implementation(&implementation)
                .unwrap()
                .mismatch_reasons
                .is_empty()
        );
        let mut implementation = record();
        implementation.training_authorized = true;
        assert!(
            independently_audit_implementation(&implementation)
                .unwrap()
                .mismatch_reasons
                .contains(&"runner_training_artifact_metric_or_trading_authority_open".to_string())
        );
    }

    #[test]
    fn reviewer_must_be_outside_registrar_and_complete_prior_chain() {
        let implementation = record();
        for actor in expected_excluded_prior_actor_ids(&implementation, &[]) {
            assert!(
                build_review(
                    &implementation,
                    &[],
                    None,
                    &actor,
                    request(&implementation, None),
                    Utc::now()
                )
                .is_err()
            );
        }
    }

    #[test]
    fn approval_requires_all_fourteen_confirmations_and_exact_audit_binding() {
        let implementation = record();
        let mut missing = request(&implementation, None);
        missing.per_target_per_seed_metrics_without_composite_masking_confirmed = false;
        assert!(build_review(&implementation, &[], None, "reviewer", missing, Utc::now()).is_err());
        let mut stale = request(&implementation, None);
        stale.expected_independent_audit_sha256 = "0".repeat(64);
        assert!(build_review(&implementation, &[], None, "reviewer", stale, Utc::now()).is_err());
    }

    #[test]
    fn approval_only_opens_future_isolated_runner_registration() {
        let review = approved_review();
        assert!(review.training_implementation_independently_approved);
        assert!(review.future_isolated_training_runner_registration_eligible);
        assert!(!review.isolated_training_runner_registered);
        assert!(!review.training_data_access_authorized);
        assert!(!review.training_started);
        assert!(!review.model_artifact_created);
        assert!(!review.metrics_created);
        assert!(!review.reward_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn one_bit_or_authority_tamper_fails_validation() {
        let mut review = approved_review();
        review.training_started = true;
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
    fn review_chain_binds_predecessor_and_excludes_prior_reviewer() {
        let implementation = record();
        let mut first_request = request(&implementation, None);
        first_request.verdict =
            HistoricalOutcomeTrainingImplementationReviewVerdict::ChangesRequested;
        let first = build_review(
            &implementation,
            &[],
            None,
            "reviewer-a",
            first_request,
            DateTime::from_timestamp(1_765_100_000, 0).unwrap(),
        )
        .unwrap();
        let second = build_review(
            &implementation,
            std::slice::from_ref(&first),
            Some(&first),
            "reviewer-b",
            request(&implementation, Some(&first)),
            DateTime::from_timestamp(1_765_100_100, 0).unwrap(),
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
        let mut drift = second;
        drift.previous_review_sha256 = Some("0".repeat(64));
        drift.review_sha256 = review_fingerprint(&drift).unwrap();
        drift.review_id = drift.review_sha256[..32].to_string();
        assert!(validate_review_chain(&[first, drift], &implementation.implementation_id).is_err());
    }

    #[test]
    fn approved_review_is_terminal() {
        let approved = approved_review();
        let implementation = approved.implementation.clone();
        let mut next_request = request(&implementation, Some(&approved));
        next_request.verdict = HistoricalOutcomeTrainingImplementationReviewVerdict::Rejected;
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
    fn review_contract_keeps_runner_training_reward_and_targets_separate() {
        let contract = review_contract();
        assert!(contract.runner_registration_separate);
        assert!(contract.data_access_authorization_separate);
        assert!(contract.training_execution_separate);
        assert!(contract.output_validation_separate);
        assert!(contract.reward_governance_separate);
        assert!(contract.targets_remain_engineering_candidates_not_strategy_truth);
        assert!(valid_sha256(&contract.contract_sha256));
    }

    #[test]
    fn implementation_and_contract_hashes_bind_artifact_metrics_and_resources() {
        let implementation = record();
        let first_audit = independently_audit_implementation(&implementation)
            .unwrap()
            .audit_sha256;
        let mut changed = implementation;
        changed.implementation_contract.maximum_output_bytes += 1;
        changed.implementation_contract.reported_metric_ids.pop();
        assert_ne!(
            first_audit,
            independently_audit_implementation(&changed)
                .unwrap()
                .audit_sha256
        );
    }
}
