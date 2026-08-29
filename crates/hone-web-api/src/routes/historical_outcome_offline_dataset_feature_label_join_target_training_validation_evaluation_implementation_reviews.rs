//! Independent, append-only review of one immutable Stage 59 validation evaluator.
//!
//! Approval opens only a future isolated-runner specification registration gate. This module
//! cannot read validation labels, evaluate candidates, select a model, access the sealed holdout,
//! write model/metric stores, define reward semantics, or reach any trading surface.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_implementations::{
    HistoricalOutcomeValidationEvaluationImplementationContract,
    HistoricalOutcomeValidationEvaluationImplementationRecord,
    independent_review_eligible_validation_evaluation_implementations,
    validate_historical_outcome_validation_evaluation_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-implementation-independent-review-v1";
const POLICY_VERSION: &str =
    "hone-validation-evaluation-implementation-independent-review-v1-no-evaluation";
const AUDIT_SCHEMA_VERSION: &str = "hone-validation-evaluation-implementation-independent-audit-v1";
const EXPECTED_CONTRACT_SCHEMA_VERSION: &str =
    "hone-validation-evaluation-implementation-contract-v1";
const EXPECTED_PROTOCOL_VERSION: &str =
    "hone-validation-evaluator-v1-per-target-three-seed-component-block-holm-gate";
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 9;
const EXPECTED_ARTIFACT_COUNT: usize = 9;
const MAX_RATIONALE_CHARS: usize = 3_000;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeValidationEvaluationImplementationReviewVerdict {
    ApprovedForFutureIsolatedValidationEvaluationRunnerRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub candidate_set_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub candidate_set_hash_independently_reproduced: bool,
    pub exact_stage_58_validation_and_stage_57_output_binding_valid: bool,
    pub exact_three_algorithm_three_seed_nine_artifact_matrix_valid: bool,
    pub exact_65_feature_nine_target_order_valid: bool,
    pub per_target_per_seed_metric_contract_valid: bool,
    pub paired_component_block_bootstrap_holm_contract_valid: bool,
    pub minimum_effect_diagnostics_and_sample_gates_valid: bool,
    pub all_three_seed_no_shopping_no_composite_contract_valid: bool,
    pub rules_frozen_before_label_access_valid: bool,
    pub all_evaluation_selection_store_reward_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeValidationEvaluationImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_candidate_set_sha256: String,
    expected_implementation_artifact_sha256: String,
    expected_immutable_code_revision: String,
    expected_upstream_validation_sha256: String,
    expected_upstream_output_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: HistoricalOutcomeValidationEvaluationImplementationReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_stage_57_through_stage_59_chain_confirmed: bool,
    reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: bool,
    implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed: bool,
    exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed: bool,
    exact_65_feature_nine_target_and_per_target_metric_contract_confirmed: bool,
    component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: bool,
    minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: bool,
    no_seed_shopping_tuning_or_composite_masking_confirmed: bool,
    rules_frozen_before_validation_label_access_confirmed: bool,
    independent_runner_authorization_and_output_validation_separation_confirmed: bool,
    no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
        bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: HistoricalOutcomeValidationEvaluationImplementationRecord,
    pub independent_audit: HistoricalOutcomeValidationEvaluationImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeValidationEvaluationImplementationReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_stage_57_through_stage_59_chain_confirmed: bool,
    pub reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed:
        bool,
    pub implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed: bool,
    pub exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed: bool,
    pub exact_65_feature_nine_target_and_per_target_metric_contract_confirmed: bool,
    pub component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: bool,
    pub minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: bool,
    pub no_seed_shopping_tuning_or_composite_masking_confirmed: bool,
    pub rules_frozen_before_validation_label_access_confirmed: bool,
    pub independent_runner_authorization_and_output_validation_separation_confirmed: bool,
    pub no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub validation_evaluation_implementation_independently_approved: bool,
    pub future_isolated_runner_registration_eligible: bool,
    pub isolated_runner_registered: bool,
    pub validation_label_access_authorized: bool,
    pub evaluation_authorized: bool,
    pub evaluation_started: bool,
    pub evaluation_completed: bool,
    pub candidate_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationReviewItem {
    pub implementation: HistoricalOutcomeValidationEvaluationImplementationRecord,
    pub current_independent_audit:
        HistoricalOutcomeValidationEvaluationImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<HistoricalOutcomeValidationEvaluationImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_isolated_runner_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeValidationEvaluationImplementationReviewItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_isolated_runner_registration_eligible_count: usize,
    pub review_status: String,
    pub validation_label_access_authorized: bool,
    pub evaluation_authorized: bool,
    pub evaluation_started: bool,
    pub candidate_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeValidationEvaluationImplementationReviewReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_isolated_runner_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeValidationEvaluationImplementationReview {
    pub implementation: HistoricalOutcomeValidationEvaluationImplementationRecord,
    pub review: HistoricalOutcomeValidationEvaluationImplementationReviewRecord,
}

pub(crate) async fn handle_get_historical_outcome_validation_evaluation_implementation_reviews(
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
            warn!(%error, "validation evaluation implementation review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "validation 评估实现复核表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_validation_evaluation_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeValidationEvaluationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &implementation_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_validation_evaluation_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeValidationEvaluationImplementationReviewReadinessSummary {
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            independently_approved_count: registry.independently_approved_count,
            changes_requested_or_rejected_count: registry.changes_requested_or_rejected_count,
            future_isolated_runner_registration_eligible_count: registry
                .future_isolated_runner_registration_eligible_count,
            review_status: registry.review_status,
        },
    )
}

#[allow(dead_code)] // Consumed by the separately gated Stage 61 runner registry.
pub(crate) async fn approved_validation_evaluation_implementation_reviews_for_runner_registration(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeValidationEvaluationImplementationReview>, String> {
    let implementations =
        independent_review_eligible_validation_evaluation_implementations(state).await?;
    let mut approved = Vec::new();
    for implementation in implementations {
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let Some(review) = reviews.last().cloned() else {
            continue;
        };
        if review.implementation != implementation
            || !review.future_isolated_runner_registration_eligible
        {
            continue;
        }
        validate_validation_evaluation_implementation_review_for_runner_registration(&review)?;
        approved.push(
            ApprovedHistoricalOutcomeValidationEvaluationImplementationReview {
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

#[allow(dead_code)]
pub(crate) fn validate_validation_evaluation_implementation_review_for_runner_registration(
    review: &HistoricalOutcomeValidationEvaluationImplementationReviewRecord,
) -> Result<(), String> {
    validate_review(review)?;
    if !review.future_isolated_runner_registration_eligible {
        return Err("validation 评估实现复核未批准未来隔离 runner 规格登记".to_string());
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationReviewRegistry, String> {
    let implementations =
        independent_review_eligible_validation_evaluation_implementations(state).await?;
    let mut items = Vec::with_capacity(implementations.len());
    for implementation in implementations {
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("当前 validation 评估实现独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|value| {
            value.implementation != implementation || value.independent_audit != audit
        }) {
            return Err("validation 评估实现复核绑定或审计已变化".to_string());
        }
        let approved = latest_review
            .as_ref()
            .is_some_and(|value| value.future_isolated_runner_registration_eligible);
        items.push(
            HistoricalOutcomeValidationEvaluationImplementationReviewItem {
                implementation,
                current_independent_audit: audit,
                complete_review_actor_ids: reviews
                    .iter()
                    .map(|value| value.reviewer_id.clone())
                    .collect(),
                latest_review,
                review_eligible: !approved,
                future_isolated_runner_registration_eligible: approved,
            },
        );
    }
    let review_eligible_count = items.iter().filter(|value| value.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|value| value.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|value| value.future_isolated_runner_registration_eligible)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|value| {
            value
                .latest_review
                .as_ref()
                .is_some_and(|review| !review.future_isolated_runner_registration_eligible)
        })
        .count();
    let review_status = if items.is_empty() {
        "waiting_current_validation_evaluation_implementation"
    } else if independently_approved_count > 0 {
        "current_validation_evaluation_implementation_approved_only_for_future_isolated_runner_registration"
    } else if review_eligible_count > 0 {
        "current_validation_evaluation_implementation_ready_for_independent_review"
    } else {
        "validation_evaluation_implementation_review_blocked_fail_closed"
    };
    Ok(HistoricalOutcomeValidationEvaluationImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_requested_or_rejected_count,
        future_isolated_runner_registration_eligible_count: independently_approved_count,
        review_status: review_status.to_string(),
        validation_label_access_authorized: false,
        evaluation_authorized: false,
        evaluation_started: false,
        candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 60 阶段由 Stage 57–59 完整链之外的新角色独立重算实现、合同与候选集合哈希，并核对三算法×三种子×九目标、逐目标指标、10,000 次 component-block bootstrap、54 项 Holm 修正、最小效果/样本/三种子门槛和零权限边界。批准只开放未来隔离 runner 规格登记；当前不读标签、不评估、不选模、不访问 sealed holdout、不写模型或指标库、不定义奖励、不影子、不下单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewHistoricalOutcomeValidationEvaluationImplementationRequest,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationReviewRecord, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("validation 评估实现复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementation = independent_review_eligible_validation_evaluation_implementations(state)
        .await?
        .into_iter()
        .find(|value| value.implementation_id == implementation_id)
        .ok_or_else(|| "当前 validation 评估实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|value| value.future_isolated_runner_registration_eligible) {
        return Err("当前 validation 评估实现已经独立批准，不得追加或分叉复核".to_string());
    }
    let review = build_review(
        &implementation,
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
    implementation: &HistoricalOutcomeValidationEvaluationImplementationRecord,
    prior_reviews: &[HistoricalOutcomeValidationEvaluationImplementationReviewRecord],
    prior: Option<&HistoricalOutcomeValidationEvaluationImplementationReviewRecord>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeValidationEvaluationImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationReviewRecord, String> {
    validate_historical_outcome_validation_evaluation_implementation_for_review(implementation)?;
    let audit = independently_audit(implementation)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("validation 评估实现独立审计不通过".to_string());
    }
    let contract = &implementation.implementation_contract;
    if request.expected_previous_review_id != prior.map(|value| value.review_id.clone())
        || request.expected_previous_review_sha256 != prior.map(|value| value.review_sha256.clone())
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_candidate_set_sha256 != contract.candidate_set_sha256
        || request.expected_implementation_artifact_sha256
            != contract.implementation_artifact_sha256
        || request.expected_immutable_code_revision != contract.immutable_code_revision
        || request.expected_upstream_validation_sha256
            != implementation.upstream_validation.validation_sha256
        || request.expected_upstream_output_sha256
            != implementation.upstream_validation.output_sha256
        || request.expected_independent_audit_sha256 != audit.audit_sha256
    {
        return Err("validation 评估实现、上游、审计或复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|value| value == reviewer_id)
    {
        return Err("复核人参与过 Stage 57–59 或此前复核链，必须更换独立角色".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let approved = request.verdict == HistoricalOutcomeValidationEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedValidationEvaluationRunnerRegistration;
    let all_confirmed = request_checks(&request);
    if approved && !all_confirmed {
        return Err("批准前必须逐项确认哈希、统计协议、角色隔离和零能力边界".to_string());
    }
    let mut review = HistoricalOutcomeValidationEvaluationImplementationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: prior.map(|value| value.review_id.clone()),
        previous_review_sha256: prior.map(|value| value.review_sha256.clone()),
        implementation: implementation.clone(),
        independent_audit: audit,
        submitted_at,
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        known_limitations,
        exact_stage_57_through_stage_59_chain_confirmed: request.exact_stage_57_through_stage_59_chain_confirmed,
        reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: request.reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed,
        implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed: request.implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed,
        exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed: request.exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed,
        exact_65_feature_nine_target_and_per_target_metric_contract_confirmed: request.exact_65_feature_nine_target_and_per_target_metric_contract_confirmed,
        component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: request.component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed,
        minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: request.minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed,
        no_seed_shopping_tuning_or_composite_masking_confirmed: request.no_seed_shopping_tuning_or_composite_masking_confirmed,
        rules_frozen_before_validation_label_access_confirmed: request.rules_frozen_before_validation_label_access_confirmed,
        independent_runner_authorization_and_output_validation_separation_confirmed: request.independent_runner_authorization_and_output_validation_separation_confirmed,
        no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: request.no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed,
        validation_evaluation_implementation_independently_approved: approved && all_confirmed,
        future_isolated_runner_registration_eligible: approved && all_confirmed,
        isolated_runner_registered: false,
        validation_label_access_authorized: false,
        evaluation_authorized: false,
        evaluation_started: false,
        evaluation_completed: false,
        candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review)?;
    Ok(review)
}

fn independently_audit(
    implementation: &HistoricalOutcomeValidationEvaluationImplementationRecord,
) -> Result<HistoricalOutcomeValidationEvaluationImplementationIndependentAudit, String> {
    let contract = &implementation.implementation_contract;
    let record_hash = valid_id(&implementation.implementation_id)
        && implementation.implementation_id == implementation.implementation_sha256[..32]
        && implementation.implementation_sha256
            == fingerprint_without(
                implementation,
                &["implementation_id", "implementation_sha256"],
            )?;
    let contract_hash = valid_sha256(&contract.contract_sha256)
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?;
    let candidate_hash = valid_sha256(&contract.candidate_set_sha256)
        && contract.candidate_set_sha256 == hash_serializable(&contract.candidate_bindings)?;
    let binding = contract.upstream_validation_sha256
        == implementation.upstream_validation.validation_sha256
        && contract.upstream_output_sha256 == implementation.upstream_validation.output_sha256
        && contract.suite_specification_sha256
            == implementation
                .upstream_validation
                .suite_specification_sha256
        && contract.training_store_dataset_sha256
            == implementation
                .upstream_validation
                .training_store_dataset_sha256;
    let matrix = exact_candidate_matrix(contract);
    let targets = contract.schema_version == EXPECTED_CONTRACT_SCHEMA_VERSION
        && contract.implementation_protocol_version == EXPECTED_PROTOCOL_VERSION
        && contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == EXPECTED_TARGET_COUNT
        && contract.exact_artifact_count == EXPECTED_ARTIFACT_COUNT
        && contract.exact_random_seeds == EXPECTED_SEEDS
        && contract.target_vector_order == expected_target_order()
        && valid_sha256(&contract.feature_order_sha256)
        && valid_sha256(&contract.preprocessing_sha256);
    let metrics = contract.reported_metric_ids == expected_metrics();
    let bootstrap = contract.bootstrap_unit == "official_split_component_id"
        && contract.bootstrap_replications == 10_000
        && contract.bootstrap_random_seed == 59_202_608_23
        && contract.family_wise_error_correction
            == "holm_bonferroni_across_54_candidate_target_seed_tests"
        && contract.family_wise_alpha_millionths == 50_000
        && contract.exact_candidate_hypothesis_count == 54;
    let gates = contract.minimum_relative_mae_improvement_ppm == 50_000
        && contract.minimum_spearman_millionths == 0
        && contract.minimum_directional_accuracy_millionths == 500_000
        && contract.minimum_calibration_slope_millionths == 500_000
        && contract.maximum_calibration_slope_millionths == 1_500_000
        && contract.minimum_validation_rows == 100
        && contract.minimum_independent_components == 20
        && contract.tie_break_preferred_algorithm_id == "ridge_multi_target_regression";
    let robustness = contract.all_three_seeds_must_pass
        && contract.no_composite_score_or_global_model_validity_claim
        && !contract.seed_robustness_rule.trim().is_empty()
        && !contract.per_target_admission_rule.trim().is_empty()
        && !contract.tie_break_rule.trim().is_empty()
        && !contract.insufficient_sample_rule.trim().is_empty();
    let frozen = implementation.evaluation_rules_frozen_before_validation_label_access_confirmed
        && implementation.no_seed_shopping_hyperparameter_tuning_or_composite_masking_confirmed;
    let closed = authority_closed(contract, implementation);
    let checks = [
        ("implementation_record_hash", record_hash),
        ("implementation_contract_hash", contract_hash),
        ("candidate_set_hash", candidate_hash),
        ("stage_58_stage_57_binding", binding),
        ("candidate_matrix", matrix),
        ("features_targets", targets),
        ("reported_metrics", metrics),
        ("bootstrap_holm", bootstrap),
        ("effect_diagnostics_sample_gates", gates),
        ("all_seed_no_composite", robustness),
        ("frozen_before_labels", frozen),
        ("zero_authority", closed),
    ];
    let mismatch_reasons = checks
        .into_iter()
        .filter_map(|(name, ok)| (!ok).then_some(name.to_string()))
        .collect::<Vec<_>>();
    let mut audit = HistoricalOutcomeValidationEvaluationImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        candidate_set_sha256: contract.candidate_set_sha256.clone(),
        implementation_record_hash_independently_reproduced: record_hash,
        implementation_contract_hash_independently_reproduced: contract_hash,
        candidate_set_hash_independently_reproduced: candidate_hash,
        exact_stage_58_validation_and_stage_57_output_binding_valid: binding,
        exact_three_algorithm_three_seed_nine_artifact_matrix_valid: matrix,
        exact_65_feature_nine_target_order_valid: targets,
        per_target_per_seed_metric_contract_valid: metrics,
        paired_component_block_bootstrap_holm_contract_valid: bootstrap,
        minimum_effect_diagnostics_and_sample_gates_valid: gates,
        all_three_seed_no_shopping_no_composite_contract_valid: robustness,
        rules_frozen_before_label_access_valid: frozen,
        all_evaluation_selection_store_reward_and_trading_authority_closed: closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn exact_candidate_matrix(
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
) -> bool {
    let algorithms = [
        "frozen_zero_prediction_baseline",
        "gradient_boosted_multi_target_regression",
        "ridge_multi_target_regression",
    ];
    let tuples = contract
        .candidate_bindings
        .iter()
        .map(|value| (value.algorithm_id.as_str(), value.random_seed))
        .collect::<BTreeSet<_>>();
    let hashes = contract
        .candidate_bindings
        .iter()
        .map(|value| value.artifact_sha256.as_str())
        .collect::<BTreeSet<_>>();
    contract.candidate_bindings.len() == EXPECTED_ARTIFACT_COUNT
        && hashes.len() == EXPECTED_ARTIFACT_COUNT
        && contract.candidate_bindings.iter().all(|value| {
            valid_sha256(&value.artifact_sha256)
                && value.exact_target_model_count == EXPECTED_TARGET_COUNT
        })
        && algorithms.iter().all(|algorithm| {
            EXPECTED_SEEDS
                .iter()
                .all(|seed| tuples.contains(&(algorithm, *seed)))
        })
}

fn authority_closed(
    contract: &HistoricalOutcomeValidationEvaluationImplementationContract,
    implementation: &HistoricalOutcomeValidationEvaluationImplementationRecord,
) -> bool {
    !contract.callable_entrypoint_present
        && !contract.validation_features_access_allowed
        && !contract.validation_labels_access_allowed
        && !contract.sealed_holdout_features_access_allowed
        && !contract.sealed_holdout_labels_access_allowed
        && !contract.training_or_preprocessing_update_allowed
        && !contract.hyperparameter_or_threshold_tuning_allowed
        && !contract.candidate_selection_allowed
        && !contract.model_store_write_allowed
        && !contract.metric_store_write_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.outbound_network_allowed
        && !contract.secrets_allowed
        && !contract.scalar_reward_defined
        && !contract.action_position_or_ranking_semantics_defined
        && !implementation.validation_label_access_authorized
        && !implementation.evaluation_started
        && !implementation.evaluation_completed
        && !implementation.candidate_selection_authorized
        && !implementation.sealed_holdout_access_authorized
        && !implementation.model_artifact_store_written
        && !implementation.metric_store_written
        && !implementation.reward_authorized
        && !implementation.shadow_portfolio_authorized
        && !implementation.order_generation_authorized
        && !implementation.broker_access_authorized
        && !implementation.trading_authorized
}

fn expected_target_order() -> Vec<String> {
    [20_u16, 60, 250]
        .into_iter()
        .flat_map(|horizon| {
            ["asset_return", "excess_return", "asset_max_drawdown"]
                .into_iter()
                .map(move |metric| format!("{metric}_{horizon}_continuous"))
        })
        .collect()
}

fn expected_metrics() -> Vec<String> {
    [
        "validation_mae_per_target_per_seed",
        "paired_relative_mae_improvement_vs_zero_per_target_per_seed",
        "component_block_bootstrap_mae_improvement_p_value_per_target_per_seed",
        "holm_adjusted_p_value_across_54_candidate_target_seed_tests",
        "validation_spearman_per_target_per_seed",
        "validation_directional_accuracy_per_target_per_seed",
        "validation_calibration_slope_per_target_per_seed",
    ]
    .map(ToString::to_string)
    .to_vec()
}

fn request_checks(
    request: &ReviewHistoricalOutcomeValidationEvaluationImplementationRequest,
) -> bool {
    request.exact_stage_57_through_stage_59_chain_confirmed
        && request.reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed
        && request.implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed
        && request.exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed
        && request.exact_65_feature_nine_target_and_per_target_metric_contract_confirmed
        && request.component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed
        && request.minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed
        && request.no_seed_shopping_tuning_or_composite_masking_confirmed
        && request.rules_frozen_before_validation_label_access_confirmed
        && request.independent_runner_authorization_and_output_validation_separation_confirmed
        && request.no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn review_checks(review: &HistoricalOutcomeValidationEvaluationImplementationReviewRecord) -> bool {
    review.exact_stage_57_through_stage_59_chain_confirmed
        && review.reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed
        && review.implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed
        && review.exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed
        && review.exact_65_feature_nine_target_and_per_target_metric_contract_confirmed
        && review.component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed
        && review.minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed
        && review.no_seed_shopping_tuning_or_composite_masking_confirmed
        && review.rules_frozen_before_validation_label_access_confirmed
        && review.independent_runner_authorization_and_output_validation_separation_confirmed
        && review.no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn expected_excluded_actor_ids(
    implementation: &HistoricalOutcomeValidationEvaluationImplementationRecord,
    prior_reviews: &[HistoricalOutcomeValidationEvaluationImplementationReviewRecord],
) -> Vec<String> {
    let mut values = implementation.excluded_prior_actor_ids.clone();
    values.push(implementation.registered_by.clone());
    values.push(implementation.upstream_validation.validated_by.clone());
    values.push(implementation.upstream_validation.invoked_by.clone());
    values.extend(prior_reviews.iter().map(|value| value.reviewer_id.clone()));
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn validate_review(
    review: &HistoricalOutcomeValidationEvaluationImplementationReviewRecord,
) -> Result<(), String> {
    validate_historical_outcome_validation_evaluation_implementation_for_review(
        &review.implementation,
    )?;
    let expected_audit = independently_audit(&review.implementation)?;
    let approved = review.verdict == HistoricalOutcomeValidationEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedValidationEvaluationRunnerRegistration;
    let closed = !review.isolated_runner_registered
        && !review.validation_label_access_authorized
        && !review.evaluation_authorized
        && !review.evaluation_started
        && !review.evaluation_completed
        && !review.candidate_selection_authorized
        && !review.sealed_holdout_access_authorized
        && !review.model_artifact_store_written
        && !review.metric_store_written
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
        || review.review_sha256 != fingerprint_without(review, &["review_id", "review_sha256"])?
        || review.independent_audit != expected_audit
        || !review.independent_audit.mismatch_reasons.is_empty()
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review.reviewer_id.trim().is_empty()
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &review.reviewer_id)
        || review.rationale.trim().is_empty()
        || review.known_limitations.trim().is_empty()
        || (approved && !review_checks(review))
        || review.validation_evaluation_implementation_independently_approved
            != (approved && review_checks(review))
        || review.future_isolated_runner_registration_eligible
            != (approved && review_checks(review))
        || !closed
    {
        return Err(
            "validation evaluation implementation review is invalid or over-authorized".to_string(),
        );
    }
    if review.previous_review_id.is_some() != review.previous_review_sha256.is_some() {
        return Err(
            "validation evaluation implementation review predecessor is invalid".to_string(),
        );
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[HistoricalOutcomeValidationEvaluationImplementationReviewRecord],
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeValidationEvaluationImplementationReviewRecord>, String> {
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
                "validation evaluation implementation review replay or subject mismatch"
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
                        "validation evaluation implementation review chain branches".to_string()
                    );
                }
            }
            None => roots.push(review.review_id.clone()),
        }
    }
    if roots.len() != 1 {
        return Err(
            "validation evaluation implementation review chain has no single root".to_string(),
        );
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = roots.pop().unwrap();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err(
                "validation evaluation implementation review chain contains a cycle".to_string(),
            );
        }
        let review = by_id
            .get(&current)
            .ok_or_else(|| {
                "validation evaluation implementation review chain is disconnected".to_string()
            })?
            .clone();
        if let Some(parent_id) = &review.previous_review_id {
            let parent = by_id.get(parent_id).ok_or_else(|| {
                "validation evaluation implementation review predecessor is missing".to_string()
            })?;
            if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                return Err(
                    "validation evaluation implementation review predecessor hash drift"
                        .to_string(),
                );
            }
        }
        if review.excluded_prior_actor_ids
            != expected_excluded_actor_ids(&review.implementation, &ordered)
        {
            return Err(
                "validation evaluation implementation review role-exclusion history mismatch"
                    .to_string(),
            );
        }
        if review.future_isolated_runner_registration_eligible
            && child_by_parent.contains_key(&current)
        {
            return Err("approved review must be terminal".to_string());
        }
        ordered.push(review);
        match child_by_parent.get(&current) {
            Some(next) => current = next.clone(),
            None => break,
        }
    }
    if ordered.len() != reviews.len() {
        return Err(
            "validation evaluation implementation review chain is disconnected".to_string(),
        );
    }
    Ok(ordered)
}

async fn read_reviews(
    state: &AppState,
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeValidationEvaluationImplementationReviewRecord>, String> {
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
        let review: HistoricalOutcomeValidationEvaluationImplementationReviewRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("validation evaluation implementation review path mismatch".to_string());
        }
        reviews.push(review);
    }
    validate_review_chain(&reviews, implementation_id)
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
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
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
        .join("historical_outcome_validation_evaluation_implementation_reviews/reviews")
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
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "validation-evaluation-implementation-review-{implementation_id}.lock"
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
                    .and_then(|value| value.modified().ok())
                    .and_then(|value| SystemTime::now().duration_since(value).ok())
                    .is_some_and(|age| age > StdDuration::from_secs(600));
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                if attempt < 7 {
                    tokio::time::sleep(StdDuration::from_millis(25)).await;
                    continue;
                }
                return Err("另一个管理员正在复核该 validation 评估实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("validation 评估实现复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_implementations::tests::record as implementation_record;

    fn request(
        implementation: &HistoricalOutcomeValidationEvaluationImplementationRecord,
    ) -> ReviewHistoricalOutcomeValidationEvaluationImplementationRequest {
        let audit = independently_audit(implementation).unwrap();
        ReviewHistoricalOutcomeValidationEvaluationImplementationRequest {
            expected_previous_review_id: None,
            expected_previous_review_sha256: None,
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_contract_sha256: implementation.implementation_contract.contract_sha256.clone(),
            expected_candidate_set_sha256: implementation.implementation_contract.candidate_set_sha256.clone(),
            expected_implementation_artifact_sha256: implementation.implementation_contract.implementation_artifact_sha256.clone(),
            expected_immutable_code_revision: implementation.implementation_contract.immutable_code_revision.clone(),
            expected_upstream_validation_sha256: implementation.upstream_validation.validation_sha256.clone(),
            expected_upstream_output_sha256: implementation.upstream_validation.output_sha256.clone(),
            expected_independent_audit_sha256: audit.audit_sha256,
            verdict: HistoricalOutcomeValidationEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedValidationEvaluationRunnerRegistration,
            rationale: "独立重算并确认预注册评估协议".to_string(),
            known_limitations: "仍未注册 runner，未读取 validation 标签或执行评估".to_string(),
            exact_stage_57_through_stage_59_chain_confirmed: true,
            reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: true,
            implementation_contract_and_candidate_set_hashes_independently_reproduced_confirmed: true,
            exact_nine_artifact_three_algorithm_three_seed_matrix_confirmed: true,
            exact_65_feature_nine_target_and_per_target_metric_contract_confirmed: true,
            component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: true,
            minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: true,
            no_seed_shopping_tuning_or_composite_masking_confirmed: true,
            rules_frozen_before_validation_label_access_confirmed: true,
            independent_runner_authorization_and_output_validation_separation_confirmed: true,
            no_entrypoint_label_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn review() -> HistoricalOutcomeValidationEvaluationImplementationReviewRecord {
        let implementation = implementation_record();
        build_review(
            &implementation,
            &[],
            None,
            "stage60-reviewer",
            request(&implementation),
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn independent_audit_reproduces_all_three_hashes_and_semantics() {
        let audit = independently_audit(&implementation_record()).unwrap();
        assert!(audit.mismatch_reasons.is_empty());
        assert!(audit.implementation_record_hash_independently_reproduced);
        assert!(audit.implementation_contract_hash_independently_reproduced);
        assert!(audit.candidate_set_hash_independently_reproduced);
        assert!(audit.paired_component_block_bootstrap_holm_contract_valid);
    }

    #[test]
    fn reviewer_must_be_outside_complete_prior_chain() {
        let implementation = implementation_record();
        let actor = implementation.registered_by.clone();
        assert!(
            build_review(
                &implementation,
                &[],
                None,
                &actor,
                request(&implementation),
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn approval_requires_every_confirmation() {
        let implementation = implementation_record();
        let mut value = request(&implementation);
        value.no_seed_shopping_tuning_or_composite_masking_confirmed = false;
        assert!(
            build_review(
                &implementation,
                &[],
                None,
                "stage60-reviewer",
                value,
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn changes_requested_does_not_open_runner_registration() {
        let implementation = implementation_record();
        let mut value = request(&implementation);
        value.verdict =
            HistoricalOutcomeValidationEvaluationImplementationReviewVerdict::ChangesRequested;
        value.no_seed_shopping_tuning_or_composite_masking_confirmed = false;
        let review = build_review(
            &implementation,
            &[],
            None,
            "stage60-reviewer",
            value,
            Utc::now(),
        )
        .unwrap();
        assert!(!review.future_isolated_runner_registration_eligible);
    }

    #[test]
    fn approval_keeps_labels_evaluation_selection_and_trading_closed() {
        let value = review();
        assert!(validate_review(&value).is_ok());
        assert!(value.future_isolated_runner_registration_eligible);
        assert!(!value.validation_label_access_authorized);
        assert!(!value.evaluation_authorized);
        assert!(!value.candidate_selection_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn tampering_or_authority_escalation_is_rejected() {
        let mut value = review();
        value.evaluation_authorized = true;
        value.review_sha256 = fingerprint_without(&value, &["review_id", "review_sha256"]).unwrap();
        value.review_id = value.review_sha256[..32].to_string();
        assert!(validate_review(&value).is_err());
    }

    #[test]
    fn review_chain_rejects_children_after_approval() {
        let first = review();
        let implementation = first.implementation.clone();
        let mut second_request = request(&implementation);
        second_request.expected_previous_review_id = Some(first.review_id.clone());
        second_request.expected_previous_review_sha256 = Some(first.review_sha256.clone());
        let second = build_review(
            &implementation,
            &[first.clone()],
            Some(&first),
            "stage60-reviewer-2",
            second_request,
            Utc::now(),
        )
        .unwrap();
        assert!(
            validate_review_chain(&[first, second], &implementation.implementation_id).is_err()
        );
    }

    #[test]
    fn review_chain_rejects_two_children_from_one_changes_requested_tip() {
        let implementation = implementation_record();
        let mut first_request = request(&implementation);
        first_request.verdict =
            HistoricalOutcomeValidationEvaluationImplementationReviewVerdict::ChangesRequested;
        let first = build_review(
            &implementation,
            &[],
            None,
            "stage60-reviewer",
            first_request,
            Utc::now(),
        )
        .unwrap();
        let child = |reviewer: &str| {
            let mut value = request(&implementation);
            value.expected_previous_review_id = Some(first.review_id.clone());
            value.expected_previous_review_sha256 = Some(first.review_sha256.clone());
            build_review(
                &implementation,
                std::slice::from_ref(&first),
                Some(&first),
                reviewer,
                value,
                Utc::now(),
            )
            .unwrap()
        };
        let second = child("stage60-reviewer-2");
        let third = child("stage60-reviewer-3");
        assert!(
            validate_review_chain(&[first, second, third], &implementation.implementation_id,)
                .is_err()
        );
    }
}
