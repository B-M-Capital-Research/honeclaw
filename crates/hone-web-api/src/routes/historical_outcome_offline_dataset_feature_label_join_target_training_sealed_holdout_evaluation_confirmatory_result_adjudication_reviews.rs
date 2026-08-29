//! Independent, append-only adjudication of exact Stage 72 sealed-holdout confirmations.
//!
//! Stage 72 proves only that a frozen output is reproducible through a second implementation.
//! This stage separately reviews statistical sufficiency, economic meaning, limitations and
//! falsification conditions. A quantitative failure can never be overridden by a reviewer.
//! Approval opens only a future controlled shadow-experiment design registration; it does not
//! select or store a model, feed training/reward, run a shadow ledger, order, broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_execution_attempts::HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_output_validations::{
    HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord,
    IndependentlyValidatedHistoricalOutcomeSealedHoldoutConfirmation,
    independently_validated_sealed_holdout_confirmations_for_adjudication_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-confirmatory-result-adjudication-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-confirmatory-result-adjudication-review-v1";
const POLICY_VERSION: &str =
    "hone-sealed-holdout-confirmatory-result-independent-adjudication-v1-no-model-promotion";
const EXPECTED_METRIC_COUNT: usize = 3;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const PASS_STATUS: &str = "untrusted_confirmatory_pass_pending_independent_validation";
const MAX_TEXT_CHARS: usize = 3_600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict {
    ApprovedForFutureControlledShadowExperimentDesignRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_output_validation_id: String,
    expected_output_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_envelope_sha256: String,
    expected_candidate_set_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    expected_sealed_holdout_projection_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    verdict: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict,
    statistical_interpretation: String,
    economic_interpretation: String,
    known_limitations: String,
    falsification_conditions: String,
    next_experiment_constraints: String,
    exact_current_stage_51_through_stage_72_binding_confirmed: bool,
    stage_72_second_implementation_reproducibility_confirmed: bool,
    exact_one_target_one_algorithm_three_frozen_seeds_confirmed: bool,
    all_three_preregistered_seed_tests_and_thresholds_reviewed: bool,
    sample_component_and_multiple_testing_sufficiency_reviewed: bool,
    target_semantics_and_economic_relevance_reviewed: bool,
    effect_size_not_p_value_only_reviewed: bool,
    data_coverage_selection_bias_and_failure_modes_reviewed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    reproducibility_not_profitability_or_generalization_confirmed: bool,
    approval_only_opens_future_controlled_shadow_experiment_design_registration_confirmed: bool,
    no_selection_store_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationCandidate {
    pub source: IndependentlyValidatedHistoricalOutcomeSealedHoldoutConfirmation,
    pub envelope_sha256: String,
    pub confirmation_status: String,
    pub sealed_holdout_row_count: usize,
    pub independent_component_count: usize,
    pub metric_count: usize,
    pub all_three_seeds_passed: bool,
    pub insufficient_evidence: bool,
    pub quantitative_approval_eligible: bool,
    pub quantitative_ineligibility_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub output_validation_id: String,
    pub output_validation_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub envelope_sha256: String,
    pub candidate_set_sha256: String,
    pub training_store_dataset_sha256: String,
    pub selected_algorithm_three_seed_binding_sha256: String,
    pub sealed_holdout_split_commitment_sha256: String,
    pub sealed_holdout_projection_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub confirmation_status: String,
    pub sealed_holdout_row_count: usize,
    pub independent_component_count: usize,
    pub metric_count: usize,
    pub all_three_seeds_passed: bool,
    pub insufficient_evidence: bool,
    pub quantitative_approval_eligible: bool,
    pub quantitative_ineligibility_reasons: Vec<String>,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict,
    pub statistical_interpretation: String,
    pub economic_interpretation: String,
    pub known_limitations: String,
    pub falsification_conditions: String,
    pub next_experiment_constraints: String,
    pub reviewer_independent_from_stage_72_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_72_binding_confirmed: bool,
    pub stage_72_second_implementation_reproducibility_confirmed: bool,
    pub exact_one_target_one_algorithm_three_frozen_seeds_confirmed: bool,
    pub all_three_preregistered_seed_tests_and_thresholds_reviewed: bool,
    pub sample_component_and_multiple_testing_sufficiency_reviewed: bool,
    pub target_semantics_and_economic_relevance_reviewed: bool,
    pub effect_size_not_p_value_only_reviewed: bool,
    pub data_coverage_selection_bias_and_failure_modes_reviewed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub reproducibility_not_profitability_or_generalization_confirmed: bool,
    pub approval_only_opens_future_controlled_shadow_experiment_design_registration_confirmed: bool,
    pub no_selection_store_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub confirmatory_result_adjudicated: bool,
    pub future_controlled_shadow_experiment_design_registration_eligible: bool,
    pub official_candidate_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationItem {
    pub candidate: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationCandidate,
    pub latest_review: Option<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview>,
    pub review_eligible: bool,
    pub confirmatory_result_adjudicated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationItem>,
    pub candidate_count: usize,
    pub quantitative_pass_count: usize,
    pub quantitative_fail_or_insufficient_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_controlled_shadow_experiment_design_registration_eligible_count: usize,
    pub adjudication_status: String,
    pub adjudication_review_available: bool,
    pub official_candidate_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReadinessSummary {
    pub candidate_count: usize,
    pub quantitative_pass_count: usize,
    pub quantitative_fail_or_insufficient_count: usize,
    pub reviewed_count: usize,
    pub approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_controlled_shadow_experiment_design_registration_eligible_count: usize,
    pub adjudication_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign {
    pub candidate: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationCandidate,
    pub review: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview,
}

pub(crate) async fn handle_get_historical_outcome_sealed_holdout_confirmatory_result_adjudications(
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
            warn!(%error, "sealed-holdout confirmatory adjudication registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "sealed-holdout 确认结果裁决暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_sealed_holdout_confirmatory_result_adjudication(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_adjudication(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_sealed_holdout_confirmatory_result_adjudication_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReadinessSummary {
            candidate_count: registry.candidate_count,
            quantitative_pass_count: registry.quantitative_pass_count,
            quantitative_fail_or_insufficient_count: registry
                .quantitative_fail_or_insufficient_count,
            reviewed_count: registry.reviewed_count,
            approved_count: registry.approved_count,
            changes_requested_or_rejected_count: registry.changes_requested_or_rejected_count,
            future_controlled_shadow_experiment_design_registration_eligible_count: registry
                .future_controlled_shadow_experiment_design_registration_eligible_count,
            adjudication_status: registry.adjudication_status,
        },
    )
}

pub(crate) async fn adjudicated_sealed_holdout_confirmations_for_shadow_design_registration(
    state: &AppState,
) -> Result<Vec<AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign>, String> {
    let mut output = Vec::new();
    for source in
        independently_validated_sealed_holdout_confirmations_for_adjudication_review(state).await?
    {
        let candidate = candidate_from_source(source)?;
        if let Some(review) = latest_review(state, &candidate).await?
            && review.confirmatory_result_adjudicated
            && review.future_controlled_shadow_experiment_design_registration_eligible
        {
            output.push(
                AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign {
                    candidate,
                    review,
                },
            );
        }
    }
    output.sort_by(|left, right| left.review.review_id.cmp(&right.review.review_id));
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry, String> {
    let mut items = Vec::new();
    for source in
        independently_validated_sealed_holdout_confirmations_for_adjudication_review(state).await?
    {
        let candidate = candidate_from_source(source)?;
        let latest_review = latest_review(state, &candidate).await?;
        let adjudicated = latest_review
            .as_ref()
            .is_some_and(|review| review.confirmatory_result_adjudicated);
        items.push(
            HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationItem {
                review_eligible: !adjudicated,
                candidate,
                latest_review,
                confirmatory_result_adjudicated: adjudicated,
            },
        );
    }
    items.sort_by(|a, b| {
        a.candidate
            .source
            .attempt
            .claim
            .attempt_id
            .cmp(&b.candidate.source.attempt.claim.attempt_id)
    });
    let candidate_count = items.len();
    let quantitative_pass_count = items
        .iter()
        .filter(|item| item.candidate.quantitative_approval_eligible)
        .count();
    let quantitative_fail_or_insufficient_count =
        candidate_count.saturating_sub(quantitative_pass_count);
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let approved_count = items
        .iter()
        .filter(|item| item.confirmatory_result_adjudicated)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review
                .as_ref()
                .is_some_and(|review| !review.confirmatory_result_adjudicated)
        })
        .count();
    let adjudication_status = if approved_count > 0 {
        "confirmatory_result_adjudicated_waiting_controlled_shadow_experiment_design_registration"
    } else if changes_requested_or_rejected_count > 0 {
        "confirmatory_result_adjudication_changes_requested_or_rejected"
    } else if candidate_count > 0 {
        "waiting_independent_confirmatory_result_adjudication"
    } else {
        "waiting_independently_validated_sealed_holdout_confirmation"
    };
    Ok(HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        candidate_count,
        quantitative_pass_count,
        quantitative_fail_or_insufficient_count,
        review_eligible_count,
        reviewed_count,
        approved_count,
        changes_requested_or_rejected_count,
        future_controlled_shadow_experiment_design_registration_eligible_count: approved_count,
        adjudication_status: adjudication_status.to_string(),
        adjudication_review_available: true,
        official_candidate_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 73 阶段把 Stage 72 的逐位可复现性与经济裁决分开。人工必须复核样本/独立分量、多重检验、效应量、目标经济语义、覆盖偏差、失败模式、局限与证伪条件；定量失败或证据不足不可人工覆盖。批准只开放未来受控影子实验设计登记，不正式选模、不写模型/指标库、不反馈训练或奖励，也不运行影子账本、订单、券商或交易。".to_string(),
    })
}

fn candidate_from_source(
    source: IndependentlyValidatedHistoricalOutcomeSealedHoldoutConfirmation,
) -> Result<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationCandidate, String> {
    let envelope = source
        .attempt
        .result
        .untrusted_confirmation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 72 来源缺少 sealed-holdout 确认信封".to_string())?;
    let envelope_sha256 = hash_serializable(envelope)?;
    let reasons = quantitative_ineligibility_reasons(envelope, &source.validation);
    Ok(
        HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationCandidate {
            confirmation_status: envelope.confirmation_status.clone(),
            sealed_holdout_row_count: envelope.sealed_holdout_row_count,
            independent_component_count: envelope.independent_component_count,
            metric_count: envelope.metrics.len(),
            all_three_seeds_passed: envelope.all_three_seeds_passed,
            insufficient_evidence: envelope.insufficient_evidence,
            quantitative_approval_eligible: reasons.is_empty(),
            quantitative_ineligibility_reasons: reasons,
            source,
            envelope_sha256,
        },
    )
}

fn quantitative_ineligibility_reasons(
    envelope: &HistoricalOutcomeSealedHoldoutEvaluationUntrustedEnvelope,
    validation: &HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if !validation.sealed_holdout_confirmation_independently_validated
        || !validation.future_confirmatory_result_adjudication_review_eligible
    {
        reasons.push("Stage 72 独立复算未通过".to_string());
    }
    if envelope.confirmation_status != PASS_STATUS {
        reasons.push(format!(
            "确认状态不是预登记通过：{}",
            envelope.confirmation_status
        ));
    }
    if envelope.insufficient_evidence || validation.recomputed_insufficient_evidence {
        reasons.push("样本或独立分量证据不足".to_string());
    }
    if !envelope.all_three_seeds_passed || !validation.recomputed_all_three_seeds_passed {
        reasons.push("三个冻结种子没有全部通过".to_string());
    }
    if envelope.metrics.len() != EXPECTED_METRIC_COUNT
        || validation.recomputed_metric_count != EXPECTED_METRIC_COUNT
    {
        reasons.push("指标数量不是精确三项".to_string());
    }
    let seeds = envelope
        .metrics
        .iter()
        .map(|metric| metric.random_seed)
        .collect::<BTreeSet<_>>();
    if seeds != EXPECTED_SEEDS.into_iter().collect() {
        reasons.push("冻结种子集合不是 17/29/43".to_string());
    }
    if envelope.metrics.iter().any(|metric| {
        metric.target_id != envelope.target_id
            || !metric.all_preregistered_thresholds_passed
            || metric.evidence_status != "all_preregistered_thresholds_passed"
            || metric.official_model_selection_metric
    }) {
        reasons.push("存在目标错配、阈值失败、证据状态异常或正式选模指标".to_string());
    }
    if !envelope.one_target_only
        || !envelope.one_algorithm_only
        || envelope.training_or_preprocessing_updated
        || envelope.feedback_reuse_allowed
        || !envelope.output_is_untrusted
        || envelope.official_candidate_selection_completed
        || envelope.global_model_validity_claimed
        || envelope.composite_score_created
    {
        reasons.push("单目标/单算法或零反馈权限边界不成立".to_string());
    }
    reasons.sort();
    reasons.dedup();
    reasons
}

async fn review_adjudication(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRequest,
) -> Result<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview, String> {
    if !valid_id(attempt_id) {
        return Err("sealed-holdout 尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let mut matching =
        independently_validated_sealed_holdout_confirmations_for_adjudication_review(state)
            .await?
            .into_iter()
            .filter(|source| source.attempt.claim.attempt_id == attempt_id)
            .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err("当前没有精确一条可裁决的 Stage 72 确认结果".to_string());
    }
    let candidate = candidate_from_source(matching.remove(0))?;
    validate_expected_binding(&candidate, &request)?;
    let latest = latest_review(state, &candidate).await?;
    if latest
        .as_ref()
        .is_some_and(|review| review.confirmatory_result_adjudicated)
    {
        return Err(
            "该确认结果已裁决通过，复核链永久冻结；下一步只能登记受控影子实验设计".to_string(),
        );
    }
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("确认结果裁决链已经变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&candidate.source.validation, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "裁决人不能是 Stage 72 验证者、Stage 71 执行者、完整上游或此前裁决角色".to_string(),
        );
    }
    let envelope = candidate
        .source
        .attempt
        .result
        .untrusted_confirmation_envelope
        .as_ref()
        .ok_or_else(|| "确认信封缺失".to_string())?;
    let mut review = HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(), review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()), previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: attempt_id.to_string(), output_validation_id: candidate.source.validation.validation_id.clone(), output_validation_sha256: candidate.source.validation.validation_sha256.clone(),
        claim_sha256: candidate.source.validation.claim_sha256.clone(), result_sha256: candidate.source.validation.result_sha256.clone(), output_sha256: candidate.source.validation.output_sha256.clone(), envelope_sha256: candidate.envelope_sha256.clone(),
        candidate_set_sha256: candidate.source.validation.candidate_set_sha256.clone(), training_store_dataset_sha256: candidate.source.validation.training_store_dataset_sha256.clone(),
        selected_algorithm_three_seed_binding_sha256: candidate.source.validation.selected_algorithm_three_seed_binding_sha256.clone(), sealed_holdout_split_commitment_sha256: candidate.source.validation.sealed_holdout_split_commitment_sha256.clone(),
        sealed_holdout_projection_sha256: candidate.source.validation.sealed_holdout_projection_sha256.clone(), feature_order_sha256: candidate.source.validation.feature_order_sha256.clone(), preprocessing_sha256: candidate.source.validation.preprocessing_sha256.clone(),
        target_id: envelope.target_id.clone(), frozen_candidate_algorithm_id: envelope.frozen_candidate_algorithm_id.clone(), confirmation_status: candidate.confirmation_status.clone(),
        sealed_holdout_row_count: candidate.sealed_holdout_row_count, independent_component_count: candidate.independent_component_count, metric_count: candidate.metric_count,
        all_three_seeds_passed: candidate.all_three_seeds_passed, insufficient_evidence: candidate.insufficient_evidence, quantitative_approval_eligible: candidate.quantitative_approval_eligible, quantitative_ineligibility_reasons: candidate.quantitative_ineligibility_reasons.clone(),
        submitted_at: Utc::now(), reviewer_id: reviewer_id.to_string(), excluded_prior_actor_ids, verdict: request.verdict,
        statistical_interpretation: bounded_required(&request.statistical_interpretation, "统计解释")?, economic_interpretation: bounded_required(&request.economic_interpretation, "经济解释")?, known_limitations: bounded_required(&request.known_limitations, "已知局限")?, falsification_conditions: bounded_required(&request.falsification_conditions, "证伪条件")?, next_experiment_constraints: bounded_required(&request.next_experiment_constraints, "下一实验约束")?,
        reviewer_independent_from_stage_72_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_72_binding_confirmed: request.exact_current_stage_51_through_stage_72_binding_confirmed,
        stage_72_second_implementation_reproducibility_confirmed: request.stage_72_second_implementation_reproducibility_confirmed,
        exact_one_target_one_algorithm_three_frozen_seeds_confirmed: request.exact_one_target_one_algorithm_three_frozen_seeds_confirmed,
        all_three_preregistered_seed_tests_and_thresholds_reviewed: request.all_three_preregistered_seed_tests_and_thresholds_reviewed,
        sample_component_and_multiple_testing_sufficiency_reviewed: request.sample_component_and_multiple_testing_sufficiency_reviewed,
        target_semantics_and_economic_relevance_reviewed: request.target_semantics_and_economic_relevance_reviewed,
        effect_size_not_p_value_only_reviewed: request.effect_size_not_p_value_only_reviewed,
        data_coverage_selection_bias_and_failure_modes_reviewed: request.data_coverage_selection_bias_and_failure_modes_reviewed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        reproducibility_not_profitability_or_generalization_confirmed: request.reproducibility_not_profitability_or_generalization_confirmed,
        approval_only_opens_future_controlled_shadow_experiment_design_registration_confirmed: request.approval_only_opens_future_controlled_shadow_experiment_design_registration_confirmed,
        no_selection_store_training_reward_shadow_order_broker_or_trading_confirmed: request.no_selection_store_training_reward_shadow_order_broker_or_trading_confirmed,
        confirmatory_result_adjudicated: false, future_controlled_shadow_experiment_design_registration_eligible: false,
        official_candidate_selection_authorized: false, model_artifact_store_written: false, metric_store_written: false, training_feedback_authorized: false, reward_authorized: false, shadow_portfolio_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
    };
    let approving = review.verdict == HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict::ApprovedForFutureControlledShadowExperimentDesignRegistration;
    if approving && !candidate.quantitative_approval_eligible {
        return Err("定量确认失败或证据不足，人工裁决不能覆盖为通过".to_string());
    }
    let approved = adjudication_approved(
        approving,
        candidate.quantitative_approval_eligible,
        independent,
        review_all_checks(&review),
    );
    review.confirmatory_result_adjudicated = approved;
    review.future_controlled_shadow_experiment_design_registration_eligible = approved;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &candidate)?;
    write_immutable_json(
        &review_directory(state, attempt_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    candidate: &HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationCandidate,
    request: &ReviewHistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationRequest,
) -> Result<(), String> {
    let validation = &candidate.source.validation;
    let envelope = candidate
        .source
        .attempt
        .result
        .untrusted_confirmation_envelope
        .as_ref()
        .ok_or_else(|| "确认信封缺失".to_string())?;
    let exact = request.expected_output_validation_id == validation.validation_id
        && request.expected_output_validation_sha256 == validation.validation_sha256
        && request.expected_claim_sha256 == validation.claim_sha256
        && request.expected_result_sha256 == validation.result_sha256
        && request.expected_output_sha256 == validation.output_sha256
        && request.expected_envelope_sha256 == candidate.envelope_sha256
        && request.expected_candidate_set_sha256 == validation.candidate_set_sha256
        && request.expected_training_store_dataset_sha256
            == validation.training_store_dataset_sha256
        && request.expected_selected_algorithm_three_seed_binding_sha256
            == validation.selected_algorithm_three_seed_binding_sha256
        && request.expected_sealed_holdout_split_commitment_sha256
            == validation.sealed_holdout_split_commitment_sha256
        && request.expected_sealed_holdout_projection_sha256
            == validation.sealed_holdout_projection_sha256
        && request.expected_feature_order_sha256 == validation.feature_order_sha256
        && request.expected_preprocessing_sha256 == validation.preprocessing_sha256
        && request.expected_target_id == envelope.target_id
        && request.expected_frozen_candidate_algorithm_id == envelope.frozen_candidate_algorithm_id;
    exact
        .then_some(())
        .ok_or_else(|| "Stage 51-72 精确绑定已经变化，请刷新后重试".to_string())
}

fn review_all_checks(
    review: &HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview,
) -> bool {
    review.exact_current_stage_51_through_stage_72_binding_confirmed
        && review.stage_72_second_implementation_reproducibility_confirmed
        && review.exact_one_target_one_algorithm_three_frozen_seeds_confirmed
        && review.all_three_preregistered_seed_tests_and_thresholds_reviewed
        && review.sample_component_and_multiple_testing_sufficiency_reviewed
        && review.target_semantics_and_economic_relevance_reviewed
        && review.effect_size_not_p_value_only_reviewed
        && review.data_coverage_selection_bias_and_failure_modes_reviewed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
        && review.reproducibility_not_profitability_or_generalization_confirmed
        && review
            .approval_only_opens_future_controlled_shadow_experiment_design_registration_confirmed
        && review.no_selection_store_training_reward_shadow_order_broker_or_trading_confirmed
}

fn adjudication_approved(
    approving_verdict: bool,
    quantitative_approval_eligible: bool,
    reviewer_independent: bool,
    all_checks: bool,
) -> bool {
    approving_verdict && quantitative_approval_eligible && reviewer_independent && all_checks
}

fn excluded_actor_ids(
    validation: &HistoricalOutcomeSealedHoldoutEvaluationOutputValidationRecord,
    latest: &Option<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview>,
) -> Vec<String> {
    let mut actors = validation.excluded_prior_actor_ids.clone();
    actors.push(validation.validated_by.clone());
    actors.push(validation.invoked_by.clone());
    if let Some(review) = latest {
        actors.extend(review.excluded_prior_actor_ids.clone());
        actors.push(review.reviewer_id.clone());
    }
    actors.retain(|actor| !actor.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn reviewer_is_independent(reviewer_id: &str, excluded_actor_ids: &[String]) -> bool {
    !reviewer_id.trim().is_empty() && excluded_actor_ids.iter().all(|actor| actor != reviewer_id)
}

fn validate_review(
    review: &HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview,
    candidate: &HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationCandidate,
) -> Result<(), String> {
    let validation = &candidate.source.validation;
    let envelope = candidate
        .source
        .attempt
        .result
        .untrusted_confirmation_envelope
        .as_ref()
        .ok_or_else(|| "确认信封缺失".to_string())?;
    let approving = review.verdict == HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict::ApprovedForFutureControlledShadowExperimentDesignRegistration;
    let independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_approved = adjudication_approved(
        approving,
        candidate.quantitative_approval_eligible,
        independent,
        review_all_checks(review),
    );
    let downstream_closed = !review.official_candidate_selection_authorized
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.shadow_portfolio_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let required_base_actors = {
        let mut actors = validation.excluded_prior_actor_ids.clone();
        actors.push(validation.validated_by.clone());
        actors.push(validation.invoked_by.clone());
        actors.retain(|actor| !actor.trim().is_empty());
        actors.sort();
        actors.dedup();
        actors
    };
    let texts_valid = [
        &review.statistical_interpretation,
        &review.economic_interpretation,
        &review.known_limitations,
        &review.falsification_conditions,
        &review.next_experiment_constraints,
    ]
    .into_iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != POLICY_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.review_sha256)
        || review.review_id != review.review_sha256[..32]
        || review.review_sha256 != review_fingerprint(review)?
        || review.previous_review_id.is_some() != review.previous_review_sha256.is_some()
        || review.attempt_id != candidate.source.attempt.claim.attempt_id
        || review.output_validation_id != validation.validation_id
        || review.output_validation_sha256 != validation.validation_sha256
        || review.claim_sha256 != validation.claim_sha256
        || review.result_sha256 != validation.result_sha256
        || review.output_sha256 != validation.output_sha256
        || review.envelope_sha256 != candidate.envelope_sha256
        || review.candidate_set_sha256 != validation.candidate_set_sha256
        || review.training_store_dataset_sha256 != validation.training_store_dataset_sha256
        || review.selected_algorithm_three_seed_binding_sha256
            != validation.selected_algorithm_three_seed_binding_sha256
        || review.sealed_holdout_split_commitment_sha256
            != validation.sealed_holdout_split_commitment_sha256
        || review.sealed_holdout_projection_sha256 != validation.sealed_holdout_projection_sha256
        || review.feature_order_sha256 != validation.feature_order_sha256
        || review.preprocessing_sha256 != validation.preprocessing_sha256
        || review.target_id != envelope.target_id
        || review.frozen_candidate_algorithm_id != envelope.frozen_candidate_algorithm_id
        || review.confirmation_status != candidate.confirmation_status
        || review.sealed_holdout_row_count != candidate.sealed_holdout_row_count
        || review.independent_component_count != candidate.independent_component_count
        || review.metric_count != candidate.metric_count
        || review.all_three_seeds_passed != candidate.all_three_seeds_passed
        || review.insufficient_evidence != candidate.insufficient_evidence
        || review.quantitative_approval_eligible != candidate.quantitative_approval_eligible
        || review.quantitative_ineligibility_reasons != candidate.quantitative_ineligibility_reasons
        || review.reviewer_id.trim().is_empty()
        || review.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || required_base_actors
            .iter()
            .any(|actor| !review.excluded_prior_actor_ids.contains(actor))
        || !independent
        || review.reviewer_independent_from_stage_72_and_complete_prior_chain != independent
        || !texts_valid
        || (approving && (!candidate.quantitative_approval_eligible || !review_all_checks(review)))
        || review.confirmatory_result_adjudicated != expected_approved
        || review.future_controlled_shadow_experiment_design_registration_eligible
            != expected_approved
        || !downstream_closed
    {
        return Err("historical sealed-holdout confirmatory-result adjudication is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn review_fingerprint(
    review: &HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    candidate: &HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationCandidate,
) -> Result<Option<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview>, String> {
    let directory = review_directory(state, &candidate.source.attempt.claim.attempt_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let review: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_review(&review, candidate)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err("确认结果裁决包含文件名不符或重复记录".to_string());
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview],
) -> Result<Option<HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview>, String> {
    if reviews.is_empty() {
        return Ok(None);
    }
    let by_id = reviews
        .iter()
        .map(|review| (review.review_id.as_str(), review))
        .collect::<HashMap<_, _>>();
    let tips = reviews
        .iter()
        .filter(|review| {
            !reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
        })
        .collect::<Vec<_>>();
    if tips.len() != 1 {
        return Err("确认结果裁决链存在分叉或多个链尖".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("确认结果裁决链存在循环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "确认结果裁决链断裂".to_string())?;
                if previous.review_sha256 != *sha {
                    return Err("确认结果裁决前序哈希不一致".to_string());
                }
                if !review
                    .excluded_prior_actor_ids
                    .contains(&previous.reviewer_id)
                    || previous
                        .excluded_prior_actor_ids
                        .iter()
                        .any(|actor| !review.excluded_prior_actor_ids.contains(actor))
                {
                    return Err("确认结果裁决未完整继承此前角色排除集合".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("确认结果裁决前序链接无效".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("确认结果裁决链断裂".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "裁决指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}
fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).map_err(|error| error.to_string())?)
    ))
}
fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("{label}不能超过 {MAX_TEXT_CHARS} 个字符"));
    }
    Ok(value.to_string())
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

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?)
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
fn review_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-sealed-holdout-confirmatory-result-adjudication-reviews")
        .join(attempt_id)
}

struct AdjudicationLock {
    path: PathBuf,
}
impl Drop for AdjudicationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<AdjudicationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-sealed-holdout-confirmatory-adjudication-{attempt_id}.lock"
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
        .map_err(|_| "已有同一确认结果裁决正在进行".to_string())?;
    Ok(AdjudicationLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_review() -> HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview {
        let mut value = HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(), review_id: String::new(), review_sha256: String::new(), previous_review_id: None, previous_review_sha256: None,
            attempt_id: "a".repeat(32), output_validation_id: "b".repeat(32), output_validation_sha256: "c".repeat(64), claim_sha256: "d".repeat(64), result_sha256: "e".repeat(64), output_sha256: "f".repeat(64), envelope_sha256: "1".repeat(64), candidate_set_sha256: "2".repeat(64), training_store_dataset_sha256: "3".repeat(64), selected_algorithm_three_seed_binding_sha256: "4".repeat(64), sealed_holdout_split_commitment_sha256: "5".repeat(64), sealed_holdout_projection_sha256: "6".repeat(64), feature_order_sha256: "7".repeat(64), preprocessing_sha256: "8".repeat(64), target_id: "excess_return_250d".to_string(), frozen_candidate_algorithm_id: "ridge".to_string(), confirmation_status: PASS_STATUS.to_string(), sealed_holdout_row_count: 90, independent_component_count: 12, metric_count: 3, all_three_seeds_passed: true, insufficient_evidence: false, quantitative_approval_eligible: true, quantitative_ineligibility_reasons: vec![], submitted_at: Utc::now(), reviewer_id: "reviewer-new".to_string(), excluded_prior_actor_ids: vec!["executor".to_string(), "validator".to_string()], verdict: HistoricalOutcomeSealedHoldoutConfirmatoryResultAdjudicationVerdict::ApprovedForFutureControlledShadowExperimentDesignRegistration,
            statistical_interpretation: "三种子与预登记阈值均通过，但只代表本次封存样本。".to_string(), economic_interpretation: "目标具有经济含义，仍需前向影子实验验证稳定性。".to_string(), known_limitations: "样本期、市场状态与公司覆盖仍有限。".to_string(), falsification_conditions: "前向窗口失效或效应方向反转即证伪。".to_string(), next_experiment_constraints: "冻结模型、成本口径和停止条件后才能登记。".to_string(), reviewer_independent_from_stage_72_and_complete_prior_chain: true,
            exact_current_stage_51_through_stage_72_binding_confirmed: true, stage_72_second_implementation_reproducibility_confirmed: true, exact_one_target_one_algorithm_three_frozen_seeds_confirmed: true, all_three_preregistered_seed_tests_and_thresholds_reviewed: true, sample_component_and_multiple_testing_sufficiency_reviewed: true, target_semantics_and_economic_relevance_reviewed: true, effect_size_not_p_value_only_reviewed: true, data_coverage_selection_bias_and_failure_modes_reviewed: true, no_unconfirmed_hari_or_old_wang_logic_claimed: true, reproducibility_not_profitability_or_generalization_confirmed: true, approval_only_opens_future_controlled_shadow_experiment_design_registration_confirmed: true, no_selection_store_training_reward_shadow_order_broker_or_trading_confirmed: true,
            confirmatory_result_adjudicated: true, future_controlled_shadow_experiment_design_registration_eligible: true, official_candidate_selection_authorized: false, model_artifact_store_written: false, metric_store_written: false, training_feedback_authorized: false, reward_authorized: false, shadow_portfolio_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        };
        value.review_sha256 = review_fingerprint(&value).unwrap();
        value.review_id = value.review_sha256[..32].to_string();
        value
    }

    #[test]
    fn all_checks_are_required() {
        let mut review = minimal_review();
        assert!(review_all_checks(&review));
        review.effect_size_not_p_value_only_reviewed = false;
        assert!(!review_all_checks(&review));
    }
    #[test]
    fn human_review_cannot_override_quantitative_failure() {
        assert!(!adjudication_approved(true, false, true, true));
        assert!(adjudication_approved(true, true, true, true));
    }
    #[test]
    fn prior_actor_cannot_adjudicate() {
        assert!(!reviewer_is_independent(
            "executor",
            &["executor".to_string()]
        ));
        assert!(reviewer_is_independent("new", &["executor".to_string()]));
    }
    #[test]
    fn fingerprint_changes_with_economic_interpretation() {
        let review = minimal_review();
        let before = review.review_sha256.clone();
        let mut changed = review;
        changed.review_id.clear();
        changed.review_sha256.clear();
        changed.economic_interpretation.push_str("新增约束");
        assert_ne!(before, review_fingerprint(&changed).unwrap());
    }
    #[test]
    fn downstream_authority_is_always_closed() {
        let review = minimal_review();
        assert!(
            !review.official_candidate_selection_authorized
                && !review.training_feedback_authorized
                && !review.shadow_portfolio_authorized
                && !review.trading_authorized
        );
    }
    #[test]
    fn chain_rejects_forks() {
        let first = minimal_review();
        let mut second = minimal_review();
        second.review_id = "9".repeat(32);
        second.review_sha256 = "9".repeat(64);
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some(first.review_sha256.clone());
        let mut fork = second.clone();
        fork.review_id = "0".repeat(32);
        fork.review_sha256 = "0".repeat(64);
        assert!(latest_review_from_records(&[first, second, fork]).is_err());
    }
    #[test]
    fn chain_requires_previous_reviewer_in_exclusions() {
        let first = minimal_review();
        let mut second = minimal_review();
        second.review_id = "9".repeat(32);
        second.review_sha256 = "9".repeat(64);
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some(first.review_sha256.clone());
        second.reviewer_id = "reviewer-two".to_string();
        second
            .excluded_prior_actor_ids
            .retain(|actor| actor != &first.reviewer_id);
        assert!(latest_review_from_records(&[first, second]).is_err());
    }
    #[test]
    fn text_is_required_and_bounded() {
        assert!(bounded_required("", "统计解释").is_err());
        assert!(bounded_required(&"a".repeat(MAX_TEXT_CHARS + 1), "统计解释").is_err());
    }
}
