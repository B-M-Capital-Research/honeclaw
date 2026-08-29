//! Independent, append-only review of one immutable Stage 67 sealed-holdout evaluator.
//!
//! Approval opens only a future isolated-runner specification registration gate. This module
//! cannot read or mount the sealed holdout, evaluate a candidate, select a model,
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementations::{
    HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
    HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    independent_review_eligible_sealed_holdout_evaluation_implementations,
    validate_historical_outcome_sealed_holdout_evaluation_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-implementation-independent-review-v1";
const POLICY_VERSION: &str =
    "hone-sealed-holdout-evaluation-implementation-independent-review-v1-no-access-no-evaluation";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-sealed-holdout-evaluation-implementation-independent-audit-v1";
const EXPECTED_CONTRACT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-implementation-contract-v1";
const EXPECTED_PROTOCOL_VERSION: &str =
    "hone-sealed-holdout-one-target-three-seed-confirmatory-evaluator-v1";
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 1;
const EXPECTED_HYPOTHESIS_COUNT: u32 = 3;
const EXPECTED_BOOTSTRAP_SEED: u64 = 66_202_608_24;
const MAX_RATIONALE_CHARS: usize = 3_000;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict {
    ApprovedForFutureIsolatedSealedHoldoutEvaluationRunnerRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub sealed_holdout_evaluation_protocol_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub sealed_holdout_evaluation_protocol_hash_independently_reproduced: bool,
    pub exact_current_stage_51_through_stage_67_binding_valid: bool,
    pub exact_one_algorithm_three_seed_one_target_contract_valid: bool,
    pub exact_65_feature_one_target_order_valid: bool,
    pub per_target_per_seed_metric_contract_valid: bool,
    pub paired_component_block_bootstrap_holm_contract_valid: bool,
    pub minimum_effect_diagnostics_and_sample_gates_valid: bool,
    pub all_three_seed_no_shopping_no_composite_contract_valid: bool,
    pub one_shot_no_feedback_create_once_untrusted_output_contract_valid: bool,
    pub all_access_evaluation_selection_store_reward_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_sealed_holdout_evaluation_protocol_sha256: String,
    expected_implementation_artifact_sha256: String,
    expected_immutable_code_revision: String,
    expected_stage_66_protocol_review_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_through_stage_67_chain_confirmed: bool,
    reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: bool,
    implementation_contract_and_protocol_hashes_independently_reproduced_confirmed: bool,
    exact_one_artifact_one_algorithm_three_seed_matrix_confirmed: bool,
    exact_65_feature_one_target_and_metric_contract_confirmed: bool,
    component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: bool,
    minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: bool,
    no_seed_shopping_tuning_refit_reselection_or_composite_masking_confirmed: bool,
    one_shot_no_feedback_create_once_untrusted_output_confirmed: bool,
    independent_runner_authorization_and_output_validation_separation_confirmed: bool,
    no_entrypoint_mount_adapter_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
        bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    pub independent_audit: HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub exact_current_stage_51_through_stage_67_chain_confirmed: bool,
    pub reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed:
        bool,
    pub implementation_contract_and_protocol_hashes_independently_reproduced_confirmed: bool,
    pub exact_one_artifact_one_algorithm_three_seed_matrix_confirmed: bool,
    pub exact_65_feature_one_target_and_metric_contract_confirmed: bool,
    pub component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: bool,
    pub minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: bool,
    pub no_seed_shopping_tuning_refit_reselection_or_composite_masking_confirmed: bool,
    pub one_shot_no_feedback_create_once_untrusted_output_confirmed: bool,
    pub independent_runner_authorization_and_output_validation_separation_confirmed: bool,
    pub no_entrypoint_mount_adapter_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub sealed_holdout_evaluation_implementation_independently_approved: bool,
    pub future_isolated_runner_registration_eligible: bool,
    pub isolated_runner_registered: bool,
    pub sealed_holdout_features_access_authorized: bool,
    pub evaluation_authorized: bool,
    pub evaluation_started: bool,
    pub evaluation_completed: bool,
    pub official_candidate_selection_authorized: bool,
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
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewItem {
    pub implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    pub current_independent_audit:
        HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_isolated_runner_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_isolated_runner_registration_eligible_count: usize,
    pub review_status: String,
    pub sealed_holdout_features_access_authorized: bool,
    pub evaluation_authorized: bool,
    pub evaluation_started: bool,
    pub official_candidate_selection_authorized: bool,
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
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_isolated_runner_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview {
    pub implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    pub review: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord,
}

pub(crate) async fn handle_get_historical_outcome_sealed_holdout_evaluation_implementation_reviews(
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
            warn!(%error, "sealed holdout evaluation implementation review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "sealed-holdout 评估实现复核表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_sealed_holdout_evaluation_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest>,
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

pub(crate) async fn historical_outcome_sealed_holdout_evaluation_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewReadinessSummary {
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

#[allow(dead_code)] // Consumed by the separately gated future Stage 69 runner registry.
pub(crate) async fn approved_sealed_holdout_evaluation_implementation_reviews_for_runner_registration(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview>, String> {
    let implementations =
        independent_review_eligible_sealed_holdout_evaluation_implementations(state).await?;
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
        validate_sealed_holdout_evaluation_implementation_review_for_runner_registration(&review)?;
        approved.push(
            ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview {
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
pub(crate) fn validate_sealed_holdout_evaluation_implementation_review_for_runner_registration(
    review: &HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord,
) -> Result<(), String> {
    validate_review(review)?;
    if !review.future_isolated_runner_registration_eligible {
        return Err("sealed-holdout 评估实现复核未批准未来隔离 runner 规格登记".to_string());
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRegistry, String> {
    let implementations =
        independent_review_eligible_sealed_holdout_evaluation_implementations(state).await?;
    let mut items = Vec::with_capacity(implementations.len());
    for implementation in implementations {
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("当前 sealed-holdout 评估实现独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|value| {
            value.implementation != implementation || value.independent_audit != audit
        }) {
            return Err("sealed-holdout 评估实现复核绑定或审计已变化".to_string());
        }
        let approved = latest_review
            .as_ref()
            .is_some_and(|value| value.future_isolated_runner_registration_eligible);
        items.push(
            HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewItem {
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
        "waiting_current_sealed_holdout_evaluation_implementation"
    } else if independently_approved_count > 0 {
        "current_sealed_holdout_evaluation_implementation_approved_only_for_future_isolated_runner_registration"
    } else if review_eligible_count > 0 {
        "current_sealed_holdout_evaluation_implementation_ready_for_independent_review"
    } else {
        "sealed_holdout_evaluation_implementation_review_blocked_fail_closed"
    };
    Ok(HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_requested_or_rejected_count,
        future_isolated_runner_registration_eligible_count: independently_approved_count,
        review_status: review_status.to_string(),
        sealed_holdout_features_access_authorized: false,
        evaluation_authorized: false,
        evaluation_started: false,
        official_candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 68 阶段由 Stage 51–67 完整链之外的新角色独立重算实现、合同与 Stage 66 协议哈希，并核对单目标×单算法×17/29/43 三种子、固定指标与门槛、10,000 次 component-block bootstrap、三项 Holm 修正、样本不足失败关闭、one-shot 无反馈及零权限边界。批准只开放未来隔离 runner 规格登记；当前无入口、挂载、adapter、sealed holdout 访问或评估，不选模、不写模型或指标库、不定义奖励、不影子、不下单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("sealed-holdout 评估实现复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementation =
        independent_review_eligible_sealed_holdout_evaluation_implementations(state)
            .await?
            .into_iter()
            .find(|value| value.implementation_id == implementation_id)
            .ok_or_else(|| "当前 sealed-holdout 评估实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|value| value.future_isolated_runner_registration_eligible) {
        return Err("当前 sealed-holdout 评估实现已经独立批准，不得追加或分叉复核".to_string());
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
    implementation: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    prior_reviews: &[HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord],
    prior: Option<&HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord, String> {
    validate_historical_outcome_sealed_holdout_evaluation_implementation_for_review(
        implementation,
    )?;
    let audit = independently_audit(implementation)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("sealed-holdout 评估实现独立审计不通过".to_string());
    }
    let contract = &implementation.implementation_contract;
    if request.expected_previous_review_id != prior.map(|value| value.review_id.clone())
        || request.expected_previous_review_sha256 != prior.map(|value| value.review_sha256.clone())
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_sealed_holdout_evaluation_protocol_sha256
            != contract.sealed_holdout_evaluation_protocol_sha256
        || request.expected_implementation_artifact_sha256
            != contract.implementation_artifact_sha256
        || request.expected_immutable_code_revision != contract.immutable_code_revision
        || request.expected_stage_66_protocol_review_sha256
            != implementation.upstream_protocol_review.review_sha256
        || request.expected_independent_audit_sha256 != audit.audit_sha256
    {
        return Err("sealed-holdout 评估实现、上游、审计或复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|value| value == reviewer_id)
    {
        return Err("复核人参与过 Stage 51–67 或此前复核链，必须更换独立角色".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let approved = request.verdict == HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedSealedHoldoutEvaluationRunnerRegistration;
    let all_confirmed = request_checks(&request);
    if approved && !all_confirmed {
        return Err("批准前必须逐项确认哈希、统计协议、角色隔离和零能力边界".to_string());
    }
    let mut review = HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord {
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
        exact_current_stage_51_through_stage_67_chain_confirmed: request.exact_current_stage_51_through_stage_67_chain_confirmed,
        reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: request.reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed,
        implementation_contract_and_protocol_hashes_independently_reproduced_confirmed: request.implementation_contract_and_protocol_hashes_independently_reproduced_confirmed,
        exact_one_artifact_one_algorithm_three_seed_matrix_confirmed: request.exact_one_artifact_one_algorithm_three_seed_matrix_confirmed,
        exact_65_feature_one_target_and_metric_contract_confirmed: request.exact_65_feature_one_target_and_metric_contract_confirmed,
        component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: request.component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed,
        minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: request.minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed,
        no_seed_shopping_tuning_refit_reselection_or_composite_masking_confirmed: request.no_seed_shopping_tuning_refit_reselection_or_composite_masking_confirmed,
        one_shot_no_feedback_create_once_untrusted_output_confirmed: request.one_shot_no_feedback_create_once_untrusted_output_confirmed,
        independent_runner_authorization_and_output_validation_separation_confirmed: request.independent_runner_authorization_and_output_validation_separation_confirmed,
        no_entrypoint_mount_adapter_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: request.no_entrypoint_mount_adapter_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed,
        sealed_holdout_evaluation_implementation_independently_approved: approved && all_confirmed,
        future_isolated_runner_registration_eligible: approved && all_confirmed,
        isolated_runner_registered: false,
        sealed_holdout_features_access_authorized: false,
        evaluation_authorized: false,
        evaluation_started: false,
        evaluation_completed: false,
        official_candidate_selection_authorized: false,
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
    implementation: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit, String> {
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
    let protocol_hash = valid_sha256(&implementation.upstream_protocol.protocol_sha256)
        && implementation.upstream_protocol.protocol_sha256
            == fingerprint_without(&implementation.upstream_protocol, &["protocol_sha256"])?;
    let protocol = &implementation.upstream_protocol;
    let binding = contract.stage_66_protocol_review_id
        == implementation.upstream_protocol_review.review_id
        && contract.stage_66_protocol_review_sha256
            == implementation.upstream_protocol_review.review_sha256
        && contract.sealed_holdout_evaluation_protocol_sha256 == protocol.protocol_sha256
        && contract.stage_65_admission_review_sha256 == protocol.stage_65_admission_review_sha256
        && contract.output_validation_sha256 == protocol.output_validation_sha256
        && contract.candidate_set_sha256 == protocol.candidate_set_sha256
        && contract.training_store_dataset_sha256 == protocol.training_store_dataset_sha256
        && contract.target_bundle_sha256 == protocol.target_bundle_sha256
        && contract.recommendation_sha256 == protocol.recommendation_sha256
        && contract.selected_algorithm_three_seed_binding_sha256
            == protocol.selected_algorithm_three_seed_binding_sha256
        && contract.sealed_holdout_split_commitment_sha256
            == protocol.sealed_holdout_split_commitment_sha256
        && contract.feature_order_sha256 == protocol.feature_order_sha256
        && contract.preprocessing_sha256 == protocol.preprocessing_sha256;
    let matrix = contract.implementation_artifact_sha256.len() == 64
        && valid_sha256(&contract.implementation_artifact_sha256)
        && !contract.immutable_code_revision.trim().is_empty()
        && contract.frozen_candidate_algorithm_id == protocol.frozen_candidate_algorithm_id
        && contract.exact_random_seeds == EXPECTED_SEEDS;
    let targets = contract.schema_version == EXPECTED_CONTRACT_SCHEMA_VERSION
        && contract.implementation_protocol_version == EXPECTED_PROTOCOL_VERSION
        && contract.exact_feature_count == EXPECTED_FEATURE_COUNT
        && contract.exact_target_count == EXPECTED_TARGET_COUNT
        && contract.exact_random_seeds == EXPECTED_SEEDS
        && contract.target_id == protocol.target_id
        && valid_sha256(&contract.feature_order_sha256)
        && valid_sha256(&contract.preprocessing_sha256);
    let metrics = contract.reported_metric_ids == expected_metrics();
    let bootstrap = contract.bootstrap_unit == "official_split_component_id"
        && contract.bootstrap_replications == 10_000
        && contract.bootstrap_random_seed == EXPECTED_BOOTSTRAP_SEED
        && contract.family_wise_error_correction
            == "holm_bonferroni_across_exact_three_frozen_seed_tests_for_one_target"
        && contract.family_wise_alpha_millionths == 50_000
        && contract.exact_candidate_hypothesis_count == EXPECTED_HYPOTHESIS_COUNT;
    let gates = contract.minimum_relative_mae_improvement_ppm == 50_000
        && contract.minimum_spearman_millionths == 0
        && contract.minimum_directional_accuracy_millionths == 500_000
        && contract.minimum_calibration_slope_millionths == 500_000
        && contract.maximum_calibration_slope_millionths == 1_500_000
        && contract.minimum_sealed_holdout_rows == 100
        && contract.minimum_independent_components == 20;
    let robustness = contract.all_three_seeds_must_pass
        && contract.no_composite_score_or_cross_target_masking
        && !contract.hyperparameter_or_threshold_tuning_allowed
        && !contract.candidate_reselection_allowed;
    let one_shot = contract.one_shot_evaluation_required
        && contract.future_output_create_once
        && contract.future_output_independent_validation_required
        && contract.no_feedback_reuse
        && contract.insufficient_sample_fails_closed
        && implementation
            .one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed
        && implementation
            .future_output_create_once_untrusted_and_independent_validation_required_confirmed;
    let closed = authority_closed(contract, implementation);
    let checks = [
        ("implementation_record_hash", record_hash),
        ("implementation_contract_hash", contract_hash),
        ("sealed_holdout_evaluation_protocol_hash", protocol_hash),
        ("stage_51_stage_67_binding", binding),
        ("one_algorithm_three_seed_matrix", matrix),
        ("features_targets", targets),
        ("reported_metrics", metrics),
        ("bootstrap_holm", bootstrap),
        ("effect_diagnostics_sample_gates", gates),
        ("all_seed_no_composite", robustness),
        ("one_shot_no_feedback_output", one_shot),
        ("zero_authority", closed),
    ];
    let mismatch_reasons = checks
        .into_iter()
        .filter_map(|(name, ok)| (!ok).then_some(name.to_string()))
        .collect::<Vec<_>>();
    let mut audit = HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        sealed_holdout_evaluation_protocol_sha256: protocol.protocol_sha256.clone(),
        implementation_record_hash_independently_reproduced: record_hash,
        implementation_contract_hash_independently_reproduced: contract_hash,
        sealed_holdout_evaluation_protocol_hash_independently_reproduced: protocol_hash,
        exact_current_stage_51_through_stage_67_binding_valid: binding,
        exact_one_algorithm_three_seed_one_target_contract_valid: matrix,
        exact_65_feature_one_target_order_valid: targets,
        per_target_per_seed_metric_contract_valid: metrics,
        paired_component_block_bootstrap_holm_contract_valid: bootstrap,
        minimum_effect_diagnostics_and_sample_gates_valid: gates,
        all_three_seed_no_shopping_no_composite_contract_valid: robustness,
        one_shot_no_feedback_create_once_untrusted_output_contract_valid: one_shot,
        all_access_evaluation_selection_store_reward_and_trading_authority_closed: closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn authority_closed(
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
    implementation: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
) -> bool {
    !contract.callable_entrypoint_present
        && !contract.input_mount_present
        && !contract.sealed_holdout_data_adapter_present
        && !contract.sealed_holdout_features_access_allowed
        && !contract.sealed_holdout_labels_access_allowed
        && !contract.sealed_holdout_evaluation_allowed
        && !contract.training_or_preprocessing_update_allowed
        && !contract.hyperparameter_or_threshold_tuning_allowed
        && !contract.candidate_reselection_allowed
        && !contract.official_candidate_selection_allowed
        && !contract.model_store_write_allowed
        && !contract.metric_store_write_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.outbound_network_allowed
        && !contract.environment_inheritance_allowed
        && !contract.secrets_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.scalar_reward_defined
        && !contract.action_position_or_ranking_semantics_defined
        && !implementation.sealed_holdout_access_authorized
        && !implementation.sealed_holdout_evaluation_authorized
        && !implementation.official_candidate_selection_authorized
        && !implementation.sealed_holdout_access_authorized
        && !implementation.model_artifact_store_written
        && !implementation.metric_store_written
        && !implementation.reward_authorized
        && !implementation.shadow_portfolio_authorized
        && !implementation.order_generation_authorized
        && !implementation.broker_access_authorized
        && !implementation.trading_authorized
}

fn expected_metrics() -> Vec<String> {
    [
        "sealed_holdout_mae_per_seed",
        "paired_relative_mae_improvement_vs_zero_per_seed",
        "component_block_bootstrap_mae_improvement_p_value_per_seed",
        "holm_adjusted_p_value_across_three_seed_tests",
        "sealed_holdout_spearman_per_seed",
        "sealed_holdout_directional_accuracy_per_seed",
        "sealed_holdout_calibration_slope_per_seed",
    ]
    .map(ToString::to_string)
    .to_vec()
}

fn request_checks(
    request: &ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_67_chain_confirmed
        && request.reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed
        && request.implementation_contract_and_protocol_hashes_independently_reproduced_confirmed
        && request.exact_one_artifact_one_algorithm_three_seed_matrix_confirmed
        && request.exact_65_feature_one_target_and_metric_contract_confirmed
        && request.component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed
        && request.minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed
        && request.no_seed_shopping_tuning_refit_reselection_or_composite_masking_confirmed
        && request.one_shot_no_feedback_create_once_untrusted_output_confirmed
        && request.independent_runner_authorization_and_output_validation_separation_confirmed
        && request.no_entrypoint_mount_adapter_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn review_checks(
    review: &HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord,
) -> bool {
    review.exact_current_stage_51_through_stage_67_chain_confirmed
        && review.reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed
        && review.implementation_contract_and_protocol_hashes_independently_reproduced_confirmed
        && review.exact_one_artifact_one_algorithm_three_seed_matrix_confirmed
        && review.exact_65_feature_one_target_and_metric_contract_confirmed
        && review.component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed
        && review.minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed
        && review.no_seed_shopping_tuning_refit_reselection_or_composite_masking_confirmed
        && review.one_shot_no_feedback_create_once_untrusted_output_confirmed
        && review.independent_runner_authorization_and_output_validation_separation_confirmed
        && review.no_entrypoint_mount_adapter_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn expected_excluded_actor_ids(
    implementation: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    prior_reviews: &[HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord],
) -> Vec<String> {
    let mut values = implementation.excluded_prior_actor_ids.clone();
    values.push(implementation.registered_by.clone());
    values.push(implementation.upstream_protocol_review.reviewer_id.clone());
    values.extend(prior_reviews.iter().map(|value| value.reviewer_id.clone()));
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn validate_review(
    review: &HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord,
) -> Result<(), String> {
    validate_historical_outcome_sealed_holdout_evaluation_implementation_for_review(
        &review.implementation,
    )?;
    let expected_audit = independently_audit(&review.implementation)?;
    let approved = review.verdict == HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedSealedHoldoutEvaluationRunnerRegistration;
    let closed = !review.isolated_runner_registered
        && !review.sealed_holdout_features_access_authorized
        && !review.evaluation_authorized
        && !review.evaluation_started
        && !review.evaluation_completed
        && !review.official_candidate_selection_authorized
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
        || review.sealed_holdout_evaluation_implementation_independently_approved
            != (approved && review_checks(review))
        || review.future_isolated_runner_registration_eligible
            != (approved && review_checks(review))
        || !closed
    {
        return Err(
            "sealed holdout evaluation implementation review is invalid or over-authorized"
                .to_string(),
        );
    }
    if review.previous_review_id.is_some() != review.previous_review_sha256.is_some() {
        return Err(
            "sealed holdout evaluation implementation review predecessor is invalid".to_string(),
        );
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord],
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord>, String> {
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
                "sealed holdout evaluation implementation review replay or subject mismatch"
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
                        "sealed holdout evaluation implementation review chain branches"
                            .to_string(),
                    );
                }
            }
            None => roots.push(review.review_id.clone()),
        }
    }
    if roots.len() != 1 {
        return Err(
            "sealed holdout evaluation implementation review chain has no single root".to_string(),
        );
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = roots.pop().unwrap();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err(
                "sealed holdout evaluation implementation review chain contains a cycle"
                    .to_string(),
            );
        }
        let review = by_id
            .get(&current)
            .ok_or_else(|| {
                "sealed holdout evaluation implementation review chain is disconnected".to_string()
            })?
            .clone();
        if let Some(parent_id) = &review.previous_review_id {
            let parent = by_id.get(parent_id).ok_or_else(|| {
                "sealed holdout evaluation implementation review predecessor is missing".to_string()
            })?;
            if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                return Err(
                    "sealed holdout evaluation implementation review predecessor hash drift"
                        .to_string(),
                );
            }
        }
        if review.excluded_prior_actor_ids
            != expected_excluded_actor_ids(&review.implementation, &ordered)
        {
            return Err(
                "sealed holdout evaluation implementation review role-exclusion history mismatch"
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
            "sealed holdout evaluation implementation review chain is disconnected".to_string(),
        );
    }
    Ok(ordered)
}

async fn read_reviews(
    state: &AppState,
    implementation_id: &str,
) -> Result<Vec<HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord>, String> {
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
        let review: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err(
                "sealed holdout evaluation implementation review path mismatch".to_string(),
            );
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
        .join("historical_outcome_sealed_holdout_evaluation_implementation_reviews/reviews")
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
        "sealed-holdout-evaluation-implementation-review-{implementation_id}.lock"
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
                return Err("另一个管理员正在复核该 sealed-holdout 评估实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("sealed-holdout 评估实现复核锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn record() -> HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord {
        let mut value = super::super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_implementations::tests::record();
        value.upstream_protocol.reported_metric_ids = expected_metrics();
        value.upstream_protocol.protocol_sha256 =
            fingerprint_without(&value.upstream_protocol, &["protocol_sha256"]).unwrap();
        value.upstream_protocol_review.protocol_sha256 =
            value.upstream_protocol.protocol_sha256.clone();
        value.upstream_protocol_review.review_sha256 = fingerprint_without(
            &value.upstream_protocol_review,
            &["review_id", "review_sha256"],
        )
        .unwrap();
        value.upstream_protocol_review.review_id =
            value.upstream_protocol_review.review_sha256[..32].to_string();
        value.implementation_contract.stage_66_protocol_review_id =
            value.upstream_protocol_review.review_id.clone();
        value
            .implementation_contract
            .stage_66_protocol_review_sha256 = value.upstream_protocol_review.review_sha256.clone();
        value
            .implementation_contract
            .sealed_holdout_evaluation_protocol_sha256 =
            value.upstream_protocol.protocol_sha256.clone();
        value.implementation_contract.reported_metric_ids = expected_metrics();
        value.implementation_contract.contract_sha256 =
            fingerprint_without(&value.implementation_contract, &["contract_sha256"]).unwrap();
        value.implementation_sha256 =
            fingerprint_without(&value, &["implementation_id", "implementation_sha256"]).unwrap();
        value.implementation_id = value.implementation_sha256[..32].to_string();
        value
    }

    pub(crate) fn approved() -> ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview
    {
        let implementation = record();
        let audit = independently_audit(&implementation).unwrap();
        let review = build_review(
            &implementation,
            &[],
            None,
            "stage-68-reviewer",
            request(
                &implementation,
                &audit,
                HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedSealedHoldoutEvaluationRunnerRegistration,
                None,
            ),
            Utc::now(),
        )
        .unwrap();
        ApprovedHistoricalOutcomeSealedHoldoutEvaluationImplementationReview {
            implementation,
            review,
        }
    }

    fn request(
        value: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
        audit: &HistoricalOutcomeSealedHoldoutEvaluationImplementationIndependentAudit,
        verdict: HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict,
        prior: Option<&HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewRecord>,
    ) -> ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest {
        ReviewHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest {
            expected_previous_review_id: prior.map(|item| item.review_id.clone()),
            expected_previous_review_sha256: prior.map(|item| item.review_sha256.clone()),
            expected_implementation_sha256: value.implementation_sha256.clone(),
            expected_implementation_contract_sha256: value
                .implementation_contract
                .contract_sha256
                .clone(),
            expected_sealed_holdout_evaluation_protocol_sha256: value
                .upstream_protocol
                .protocol_sha256
                .clone(),
            expected_implementation_artifact_sha256: value
                .implementation_contract
                .implementation_artifact_sha256
                .clone(),
            expected_immutable_code_revision: value
                .implementation_contract
                .immutable_code_revision
                .clone(),
            expected_stage_66_protocol_review_sha256: value
                .upstream_protocol_review
                .review_sha256
                .clone(),
            expected_independent_audit_sha256: audit.audit_sha256.clone(),
            verdict,
            rationale: "独立复算哈希与固定统计契约，未接触 sealed holdout".to_string(),
            known_limitations: "没有 runner、访问授权、评估或输出".to_string(),
            exact_current_stage_51_through_stage_67_chain_confirmed: true,
            reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: true,
            implementation_contract_and_protocol_hashes_independently_reproduced_confirmed: true,
            exact_one_artifact_one_algorithm_three_seed_matrix_confirmed: true,
            exact_65_feature_one_target_and_metric_contract_confirmed: true,
            component_block_bootstrap_holm_fixed_seed_and_sample_rules_confirmed: true,
            minimum_effect_rank_direction_calibration_and_all_seed_gates_confirmed: true,
            no_seed_shopping_tuning_refit_reselection_or_composite_masking_confirmed: true,
            one_shot_no_feedback_create_once_untrusted_output_confirmed: true,
            independent_runner_authorization_and_output_validation_separation_confirmed: true,
            no_entrypoint_mount_adapter_holdout_access_evaluation_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    #[test]
    fn independent_audit_reproduces_record_contract_protocol_and_semantics() {
        let audit = independently_audit(&record()).unwrap();
        assert!(audit.mismatch_reasons.is_empty());
        assert!(audit.implementation_record_hash_independently_reproduced);
        assert!(audit.implementation_contract_hash_independently_reproduced);
        assert!(audit.sealed_holdout_evaluation_protocol_hash_independently_reproduced);
        assert!(audit.exact_current_stage_51_through_stage_67_binding_valid);
        assert!(audit.exact_one_algorithm_three_seed_one_target_contract_valid);
        assert!(audit.one_shot_no_feedback_create_once_untrusted_output_contract_valid);
    }

    #[test]
    fn independent_audit_fails_on_artifact_tamper() {
        let mut value = record();
        value.implementation_contract.implementation_artifact_sha256 = "0".repeat(64);
        let audit = independently_audit(&value).unwrap();
        assert!(
            audit
                .mismatch_reasons
                .contains(&"implementation_record_hash".to_string())
        );
        assert!(
            audit
                .mismatch_reasons
                .contains(&"implementation_contract_hash".to_string())
        );
    }

    #[test]
    fn independent_audit_fails_on_protocol_or_holm_drift() {
        let mut value = record();
        value
            .implementation_contract
            .exact_candidate_hypothesis_count = 4;
        let audit = independently_audit(&value).unwrap();
        assert!(
            audit
                .mismatch_reasons
                .contains(&"bootstrap_holm".to_string())
        );
    }

    #[test]
    fn approved_review_only_opens_future_runner_registration() {
        let value = record();
        let audit = independently_audit(&value).unwrap();
        let review = build_review(
            &value,
            &[],
            None,
            "stage-68-reviewer",
            request(
                &value,
                &audit,
                HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedSealedHoldoutEvaluationRunnerRegistration,
                None,
            ),
            Utc::now(),
        )
        .unwrap();
        assert!(review.future_isolated_runner_registration_eligible);
        assert!(!review.sealed_holdout_access_authorized);
        assert!(!review.evaluation_authorized);
        assert!(!review.official_candidate_selection_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn approval_fails_when_one_confirmation_is_missing() {
        let value = record();
        let audit = independently_audit(&value).unwrap();
        let mut input = request(
            &value,
            &audit,
            HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedSealedHoldoutEvaluationRunnerRegistration,
            None,
        );
        input.one_shot_no_feedback_create_once_untrusted_output_confirmed = false;
        assert!(build_review(&value, &[], None, "stage-68-reviewer", input, Utc::now(),).is_err());
    }

    #[test]
    fn reviewer_must_be_outside_registrar_upstream_and_prior_reviewers() {
        let value = record();
        let audit = independently_audit(&value).unwrap();
        assert!(build_review(
            &value,
            &[],
            None,
            &value.registered_by,
            request(
                &value,
                &audit,
                HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ChangesRequested,
                None,
            ),
            Utc::now(),
        )
        .is_err());
    }

    #[test]
    fn changes_requested_can_be_followed_by_independent_correction_review() {
        let value = record();
        let audit = independently_audit(&value).unwrap();
        let first = build_review(
            &value,
            &[],
            None,
            "stage-68-reviewer-a",
            request(
                &value,
                &audit,
                HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ChangesRequested,
                None,
            ),
            Utc::now(),
        )
        .unwrap();
        let second = build_review(
            &value,
            std::slice::from_ref(&first),
            Some(&first),
            "stage-68-reviewer-b",
            request(
                &value,
                &audit,
                HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedSealedHoldoutEvaluationRunnerRegistration,
                Some(&first),
            ),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            second.previous_review_id.as_deref(),
            Some(first.review_id.as_str())
        );
        assert!(second.future_isolated_runner_registration_eligible);
        assert!(second.excluded_prior_actor_ids.contains(&first.reviewer_id));
    }

    #[test]
    fn approved_review_is_terminal_in_chain() {
        let value = record();
        let audit = independently_audit(&value).unwrap();
        let approved = build_review(
            &value,
            &[],
            None,
            "stage-68-reviewer-a",
            request(
                &value,
                &audit,
                HistoricalOutcomeSealedHoldoutEvaluationImplementationReviewVerdict::ApprovedForFutureIsolatedSealedHoldoutEvaluationRunnerRegistration,
                None,
            ),
            Utc::now(),
        )
        .unwrap();
        let mut child = approved.clone();
        child.review_id = "f".repeat(32);
        child.review_sha256 = "f".repeat(64);
        child.previous_review_id = Some(approved.review_id.clone());
        child.previous_review_sha256 = Some(approved.review_sha256.clone());
        assert!(validate_review_chain(&[approved, child], &value.implementation_id).is_err());
    }
}
