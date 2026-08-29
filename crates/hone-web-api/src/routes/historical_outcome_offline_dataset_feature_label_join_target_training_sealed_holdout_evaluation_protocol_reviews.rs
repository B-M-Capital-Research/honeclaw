//! Independent review of the per-target sealed-holdout evaluation protocol.
//!
//! Stage 66 freezes how one admitted Stage 65 target may eventually be evaluated exactly once.
//! It does not mount, decrypt, project or execute the sealed holdout. Approval opens only a
//! future implementation-registration gate. Data access, runner registration, one-shot execution,
//! output validation and any model-store decision remain separate future stages.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::HistoricalOutcomeTrainingExperimentAlgorithm;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_per_target_candidate_admission_reviews::{
    AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate,
    HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview,
    admitted_validation_evaluation_per_target_candidates_for_sealed_holdout_protocol_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-protocol-review-registry-v1";
const PROTOCOL_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-protocol-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-protocol-review-v1";
const POLICY_VERSION: &str =
    "hone-sealed-holdout-per-target-protocol-independent-review-v1-no-data-access";
const PROTOCOL_VERSION: &str =
    "hone-sealed-holdout-per-target-three-seed-confirmatory-evaluation-v1";
const EXPECTED_FEATURE_COUNT: usize = 65;
const EXPECTED_TARGET_COUNT: usize = 1;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const EXPECTED_HYPOTHESIS_COUNT: u32 = 3;
const BOOTSTRAP_REPLICATIONS: u32 = 10_000;
const BOOTSTRAP_RANDOM_SEED: u64 = 66_202_608_24;
const FAMILY_WISE_ALPHA_MILLIONTHS: u32 = 50_000;
const MINIMUM_RELATIVE_MAE_IMPROVEMENT_PPM: u32 = 50_000;
const MINIMUM_SEALED_HOLDOUT_ROWS: u32 = 100;
const MINIMUM_INDEPENDENT_COMPONENTS: u32 = 20;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 3_200;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict {
    ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeSealedHoldoutEvaluationProtocolRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_stage_65_admission_review_id: String,
    expected_stage_65_admission_review_sha256: String,
    expected_output_validation_sha256: String,
    expected_candidate_set_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_target_bundle_sha256: String,
    expected_recommendation_sha256: String,
    expected_protocol_sha256: String,
    verdict: HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_through_stage_65_binding_confirmed: bool,
    reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed: bool,
    one_target_one_algorithm_three_frozen_seeds_only_confirmed: bool,
    immutable_candidate_feature_preprocessing_and_target_confirmed: bool,
    sealed_holdout_single_use_and_no_feedback_reuse_confirmed: bool,
    fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: bool,
    all_three_seeds_must_pass_and_failures_remain_visible_confirmed: bool,
    insufficient_sample_fails_closed_confirmed: bool,
    no_cross_target_composite_tuning_refit_or_reselection_confirmed: bool,
    protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed: bool,
    next_gate_is_implementation_registration_not_data_access_confirmed: bool,
    no_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationProtocol {
    pub schema_version: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub attempt_id: String,
    pub target_id: String,
    pub stage_65_admission_review_id: String,
    pub stage_65_admission_review_sha256: String,
    pub output_validation_sha256: String,
    pub candidate_set_sha256: String,
    pub training_store_dataset_sha256: String,
    pub rows_sha256: String,
    pub target_commitments_sha256: String,
    pub validation_projection_sha256: String,
    pub target_bundle_sha256: String,
    pub recommendation_sha256: String,
    pub selected_algorithm_three_seed_binding_sha256: String,
    pub sealed_holdout_split_commitment_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub frozen_candidate_algorithm_id: String,
    pub exact_random_seeds: Vec<u64>,
    pub exact_feature_count: usize,
    pub exact_target_count: usize,
    pub target_vector_order: Vec<String>,
    pub benchmark_algorithm_id: String,
    pub reported_metric_ids: Vec<String>,
    pub bootstrap_unit: String,
    pub bootstrap_replications: u32,
    pub bootstrap_random_seed: u64,
    pub family_wise_error_correction: String,
    pub family_wise_alpha_millionths: u32,
    pub exact_candidate_hypothesis_count: u32,
    pub minimum_relative_mae_improvement_ppm: u32,
    pub minimum_spearman_millionths: i32,
    pub minimum_directional_accuracy_millionths: u32,
    pub minimum_calibration_slope_millionths: u32,
    pub maximum_calibration_slope_millionths: u32,
    pub minimum_sealed_holdout_rows: u32,
    pub minimum_independent_components: u32,
    pub all_three_seeds_must_pass: bool,
    pub one_shot_evaluation_required: bool,
    pub insufficient_sample_rule: String,
    pub confirmatory_decision_rule: String,
    pub no_feedback_reuse_rule: String,
    pub no_composite_score_or_cross_target_masking: bool,
    pub protocol_review_only: bool,
    pub callable_entrypoint_present: bool,
    pub sealed_holdout_features_access_allowed: bool,
    pub sealed_holdout_labels_access_allowed: bool,
    pub training_or_preprocessing_update_allowed: bool,
    pub hyperparameter_or_threshold_tuning_allowed: bool,
    pub candidate_reselection_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub outbound_network_allowed: bool,
    pub secrets_allowed: bool,
    pub scalar_reward_defined: bool,
    pub action_position_or_ranking_semantics_defined: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationProtocolReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub target_id: String,
    pub stage_65_admission_review_id: String,
    pub stage_65_admission_review_sha256: String,
    pub output_validation_sha256: String,
    pub candidate_set_sha256: String,
    pub training_store_dataset_sha256: String,
    pub target_bundle_sha256: String,
    pub recommendation_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub reviewer_independent_from_stage_65_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_65_binding_confirmed: bool,
    pub reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed: bool,
    pub one_target_one_algorithm_three_frozen_seeds_only_confirmed: bool,
    pub immutable_candidate_feature_preprocessing_and_target_confirmed: bool,
    pub sealed_holdout_single_use_and_no_feedback_reuse_confirmed: bool,
    pub fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: bool,
    pub all_three_seeds_must_pass_and_failures_remain_visible_confirmed: bool,
    pub insufficient_sample_fails_closed_confirmed: bool,
    pub no_cross_target_composite_tuning_refit_or_reselection_confirmed: bool,
    pub protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed: bool,
    pub next_gate_is_implementation_registration_not_data_access_confirmed: bool,
    pub no_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub protocol_independently_approved: bool,
    pub future_sealed_holdout_evaluation_implementation_registration_eligible: bool,
    pub official_candidate_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub sealed_holdout_evaluation_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationProtocolSubject {
    pub admitted: AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate,
    pub protocol: HistoricalOutcomeSealedHoldoutEvaluationProtocol,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewItem {
    pub subject: HistoricalOutcomeSealedHoldoutEvaluationProtocolSubject,
    pub latest_review: Option<HistoricalOutcomeSealedHoldoutEvaluationProtocolReview>,
    pub review_eligible: bool,
    pub protocol_independently_approved: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewItem>,
    pub admitted_target_count: usize,
    pub protocol_review_eligible_count: usize,
    pub protocol_reviewed_count: usize,
    pub protocol_independently_approved_count: usize,
    pub protocol_rejected_or_changes_requested_count: usize,
    pub future_sealed_holdout_evaluation_implementation_registration_eligible_count: usize,
    pub protocol_review_status: String,
    pub protocol_review_available: bool,
    pub official_candidate_selection_authorized: bool,
    pub sealed_holdout_access_authorized: bool,
    pub sealed_holdout_evaluation_authorized: bool,
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
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewReadinessSummary {
    pub admitted_target_count: usize,
    pub protocol_reviewed_count: usize,
    pub protocol_independently_approved_count: usize,
    pub protocol_rejected_or_changes_requested_count: usize,
    pub future_sealed_holdout_evaluation_implementation_registration_eligible_count: usize,
    pub protocol_review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeSealedHoldoutEvaluationProtocol {
    pub subject: HistoricalOutcomeSealedHoldoutEvaluationProtocolSubject,
    pub protocol_review: HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
}

pub(crate) async fn handle_get_historical_outcome_sealed_holdout_evaluation_protocol_reviews(
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
            warn!(%error, "sealed-holdout evaluation protocol review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "sealed-holdout 逐目标评估协议复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_sealed_holdout_evaluation_protocol(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath((attempt_id, target_id)): AxumPath<(String, String)>,
    Json(request): Json<ReviewHistoricalOutcomeSealedHoldoutEvaluationProtocolRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_protocol(&state, &admin.user_id, &attempt_id, &target_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_sealed_holdout_evaluation_protocol_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewReadinessSummary {
            admitted_target_count: registry.admitted_target_count,
            protocol_reviewed_count: registry.protocol_reviewed_count,
            protocol_independently_approved_count: registry.protocol_independently_approved_count,
            protocol_rejected_or_changes_requested_count: registry
                .protocol_rejected_or_changes_requested_count,
            future_sealed_holdout_evaluation_implementation_registration_eligible_count: registry
                .future_sealed_holdout_evaluation_implementation_registration_eligible_count,
            protocol_review_status: registry.protocol_review_status,
        },
    )
}

pub(crate) async fn approved_sealed_holdout_evaluation_protocols_for_implementation_registration(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeSealedHoldoutEvaluationProtocol>, String> {
    let registry = build_registry(state).await?;
    let mut approved = Vec::new();
    for item in registry.items {
        let Some(review) = item.latest_review else {
            continue;
        };
        if !item.protocol_independently_approved
            || !review.protocol_independently_approved
            || !review.future_sealed_holdout_evaluation_implementation_registration_eligible
        {
            continue;
        }
        validate_protocol(&item.subject.protocol)?;
        validate_review(&review, &item.subject.protocol)?;
        approved.push(ApprovedHistoricalOutcomeSealedHoldoutEvaluationProtocol {
            subject: item.subject,
            protocol_review: review,
        });
    }
    Ok(approved)
}

pub(crate) fn validate_approved_sealed_holdout_evaluation_protocol_for_implementation_registration(
    protocol: &HistoricalOutcomeSealedHoldoutEvaluationProtocol,
    review: &HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
) -> Result<(), String> {
    validate_protocol(protocol)?;
    validate_review(review, protocol)?;
    if !review.protocol_independently_approved
        || !review.future_sealed_holdout_evaluation_implementation_registration_eligible
        || review.sealed_holdout_access_authorized
        || review.sealed_holdout_evaluation_authorized
        || review.official_candidate_selection_authorized
    {
        return Err("Stage 66 协议未获当前实现登记资格或权限越界".to_string());
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry, String> {
    let admitted =
        admitted_validation_evaluation_per_target_candidates_for_sealed_holdout_protocol_review(
            state,
        )
        .await?;
    let admitted_target_count = admitted.len();
    let mut items = Vec::with_capacity(admitted.len());
    for value in admitted {
        let subject = subject(value)?;
        let latest_review = latest_review(state, &subject).await?;
        let approved = latest_review
            .as_ref()
            .is_some_and(|review| review.protocol_independently_approved);
        items.push(HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewItem {
            review_eligible: !approved,
            subject,
            latest_review,
            protocol_independently_approved: approved,
        });
    }
    items.sort_by(|left, right| {
        left.subject
            .protocol
            .attempt_id
            .cmp(&right.subject.protocol.attempt_id)
            .then_with(|| {
                left.subject
                    .protocol
                    .target_id
                    .cmp(&right.subject.protocol.target_id)
            })
    });
    let protocol_review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let protocol_reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let protocol_independently_approved_count = items
        .iter()
        .filter(|item| item.protocol_independently_approved)
        .count();
    let protocol_rejected_or_changes_requested_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    != HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration
            })
        })
        .count();
    let protocol_review_status = if protocol_independently_approved_count > 0 {
        "sealed_holdout_evaluation_implementation_registration_eligible"
    } else if protocol_rejected_or_changes_requested_count > 0 {
        "sealed_holdout_evaluation_protocol_changes_requested_or_rejected"
    } else if admitted_target_count > 0 {
        "waiting_independent_sealed_holdout_evaluation_protocol_review"
    } else {
        "waiting_stage_65_per_target_candidate_admission"
    };
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            items,
            admitted_target_count,
            protocol_review_eligible_count,
            protocol_reviewed_count,
            protocol_independently_approved_count,
            protocol_rejected_or_changes_requested_count,
            future_sealed_holdout_evaluation_implementation_registration_eligible_count:
                protocol_independently_approved_count,
            protocol_review_status: protocol_review_status.to_string(),
            protocol_review_available: true,
            official_candidate_selection_authorized: false,
            sealed_holdout_access_authorized: false,
            sealed_holdout_evaluation_authorized: false,
            model_artifact_store_written: false,
            metric_store_written: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "第 66 阶段只为每个 Stage 65 已准入目标冻结 confirmatory sealed-holdout 评估协议：一个目标、一个算法族、17/29/43 三个冻结种子、三项 family-wise 假设、固定指标/阈值、一次性评估和无反馈复用。协议复核不挂载、不解密、不投影、不读取也不执行 sealed holdout；批准只开放未来评估实现登记，正式选模、模型/指标库、奖励、影子、订单、券商和交易继续关闭。".to_string(),
        },
    )
}

fn subject(
    admitted: AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationProtocolSubject, String> {
    let protocol = protocol(&admitted)?;
    validate_protocol(&protocol)?;
    Ok(HistoricalOutcomeSealedHoldoutEvaluationProtocolSubject { admitted, protocol })
}

fn protocol(
    admitted: &AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationProtocol, String> {
    let candidate = &admitted.candidate;
    let admission = &admitted.admission_review;
    let envelope = candidate
        .source
        .attempt
        .result
        .untrusted_evaluation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 65 准入候选缺少 validation 评估 envelope".to_string())?;
    let algorithm = candidate
        .recommendation
        .recommended_algorithm
        .as_ref()
        .ok_or_else(|| "Stage 65 准入候选缺少推荐算法".to_string())?;
    if !admission.per_target_candidate_admitted
        || !admission.future_sealed_holdout_evaluation_protocol_review_eligible
        || !candidate.recommendation_admissible
        || candidate.recommendation.official_selection
        || !candidate.recommendation.all_three_seeds_passed
    {
        return Err("Stage 65 逐目标准入记录不是当前协议复核来源".to_string());
    }
    let frozen_candidate_algorithm_id = algorithm_id(algorithm).to_string();
    let selected_algorithm_three_seed_binding_sha256 = hash_serializable(&serde_json::json!({
        "candidate_set_sha256": admission.candidate_set_sha256,
        "algorithm_id": frozen_candidate_algorithm_id,
        "random_seeds": EXPECTED_SEEDS,
        "target_id": candidate.target_id,
    }))?;
    let sealed_holdout_split_commitment_sha256 = hash_serializable(&serde_json::json!({
        "training_store_dataset_sha256": admission.training_store_dataset_sha256,
        "rows_sha256": candidate.source.attempt.claim.rows_sha256,
        "target_commitments_sha256": candidate.source.attempt.claim.target_commitments_sha256,
        "split": "sealed_holdout",
        "target_id": candidate.target_id,
    }))?;
    let mut value = HistoricalOutcomeSealedHoldoutEvaluationProtocol {
        schema_version: PROTOCOL_SCHEMA_VERSION.to_string(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        protocol_sha256: String::new(),
        attempt_id: admission.attempt_id.clone(),
        target_id: candidate.target_id.clone(),
        stage_65_admission_review_id: admission.review_id.clone(),
        stage_65_admission_review_sha256: admission.review_sha256.clone(),
        output_validation_sha256: admission.output_validation_sha256.clone(),
        candidate_set_sha256: admission.candidate_set_sha256.clone(),
        training_store_dataset_sha256: admission.training_store_dataset_sha256.clone(),
        rows_sha256: candidate.source.attempt.claim.rows_sha256.clone(),
        target_commitments_sha256: candidate
            .source
            .attempt
            .claim
            .target_commitments_sha256
            .clone(),
        validation_projection_sha256: admission.validation_projection_sha256.clone(),
        target_bundle_sha256: candidate.target_bundle_sha256.clone(),
        recommendation_sha256: candidate.recommendation_sha256.clone(),
        selected_algorithm_three_seed_binding_sha256,
        sealed_holdout_split_commitment_sha256,
        feature_order_sha256: envelope.feature_order_sha256.clone(),
        preprocessing_sha256: envelope.preprocessing_sha256.clone(),
        frozen_candidate_algorithm_id,
        exact_random_seeds: EXPECTED_SEEDS.to_vec(),
        exact_feature_count: EXPECTED_FEATURE_COUNT,
        exact_target_count: EXPECTED_TARGET_COUNT,
        target_vector_order: vec![candidate.target_id.clone()],
        benchmark_algorithm_id: "frozen_zero_prediction_baseline".to_string(),
        reported_metric_ids: [
            "sealed_holdout_mae_per_seed",
            "paired_relative_mae_improvement_vs_zero_per_seed",
            "component_block_bootstrap_mae_improvement_p_value_per_seed",
            "holm_adjusted_p_value_across_three_seed_tests",
            "sealed_holdout_spearman_per_seed",
            "sealed_holdout_directional_accuracy_per_seed",
            "sealed_holdout_calibration_slope_per_seed",
        ]
        .map(ToString::to_string)
        .to_vec(),
        bootstrap_unit: "official_split_component_id".to_string(),
        bootstrap_replications: BOOTSTRAP_REPLICATIONS,
        bootstrap_random_seed: BOOTSTRAP_RANDOM_SEED,
        family_wise_error_correction:
            "holm_bonferroni_across_exact_three_frozen_seed_tests_for_one_target".to_string(),
        family_wise_alpha_millionths: FAMILY_WISE_ALPHA_MILLIONTHS,
        exact_candidate_hypothesis_count: EXPECTED_HYPOTHESIS_COUNT,
        minimum_relative_mae_improvement_ppm: MINIMUM_RELATIVE_MAE_IMPROVEMENT_PPM,
        minimum_spearman_millionths: 0,
        minimum_directional_accuracy_millionths: 500_000,
        minimum_calibration_slope_millionths: 500_000,
        maximum_calibration_slope_millionths: 1_500_000,
        minimum_sealed_holdout_rows: MINIMUM_SEALED_HOLDOUT_ROWS,
        minimum_independent_components: MINIMUM_INDEPENDENT_COMPONENTS,
        all_three_seeds_must_pass: true,
        one_shot_evaluation_required: true,
        insufficient_sample_rule: "sealed holdout 少于 100 行或少于 20 个独立 official component 时只能输出 insufficient_evidence；不得重开、换阈值、换候选或宣称有效。".to_string(),
        confirmatory_decision_rule: "同一目标的冻结候选算法必须在 17/29/43 三个种子上全部达到相对零预测 MAE 至少改善 5%、三项 component-block bootstrap 检验经 Holm 修正 q<=0.05、Spearman 为正、方向准确率不低于 0.50、校准斜率处于 [0.5,1.5]；任一种子失败即该目标未通过。".to_string(),
        no_feedback_reuse_rule: "sealed-holdout 只允许未来精确协议的一次性 confirmatory 执行；结果不得用于调参、改阈值、重训、换算法、换种子、生成新的候选或再次访问同一留出集。".to_string(),
        no_composite_score_or_cross_target_masking: true,
        protocol_review_only: true,
        callable_entrypoint_present: false,
        sealed_holdout_features_access_allowed: false,
        sealed_holdout_labels_access_allowed: false,
        training_or_preprocessing_update_allowed: false,
        hyperparameter_or_threshold_tuning_allowed: false,
        candidate_reselection_allowed: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        outbound_network_allowed: false,
        secrets_allowed: false,
        scalar_reward_defined: false,
        action_position_or_ranking_semantics_defined: false,
    };
    value.protocol_sha256 = fingerprint_without(&value, &["protocol_sha256"])?;
    Ok(value)
}

async fn review_protocol(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    target_id: &str,
    request: ReviewHistoricalOutcomeSealedHoldoutEvaluationProtocolRequest,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationProtocolReview, String> {
    if !valid_id(attempt_id) || !valid_target_id(target_id) {
        return Err("sealed-holdout 协议 attempt 或 target ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id, target_id).await?;
    let admitted =
        admitted_validation_evaluation_per_target_candidates_for_sealed_holdout_protocol_review(
            state,
        )
        .await?;
    let matching = admitted
        .into_iter()
        .filter(|value| {
            value.admission_review.attempt_id == attempt_id
                && value.candidate.target_id == target_id
        })
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err("当前没有精确一条已准入逐目标候选可复核协议".to_string());
    }
    let subject = subject(matching.into_iter().next().expect("one subject"))?;
    validate_expected_binding(&subject, &request)?;
    let latest = latest_review(state, &subject).await?;
    if latest
        .as_ref()
        .is_some_and(|review| review.protocol_independently_approved)
    {
        return Err("该目标 sealed-holdout 协议已批准，复核链永久冻结".to_string());
    }
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("sealed-holdout 协议复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限与偏差",
    )?;
    let excluded_prior_actor_ids = excluded_actor_ids(&subject.admitted.admission_review, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "协议复核人不能是 Stage 65 复核者、完整 Stage 51-65 上游或此前 Stage 66 复核者"
                .to_string(),
        );
    }
    let protocol = &subject.protocol;
    let mut review = HistoricalOutcomeSealedHoldoutEvaluationProtocolReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: protocol.attempt_id.clone(),
        target_id: protocol.target_id.clone(),
        stage_65_admission_review_id: protocol.stage_65_admission_review_id.clone(),
        stage_65_admission_review_sha256: protocol.stage_65_admission_review_sha256.clone(),
        output_validation_sha256: protocol.output_validation_sha256.clone(),
        candidate_set_sha256: protocol.candidate_set_sha256.clone(),
        training_store_dataset_sha256: protocol.training_store_dataset_sha256.clone(),
        target_bundle_sha256: protocol.target_bundle_sha256.clone(),
        recommendation_sha256: protocol.recommendation_sha256.clone(),
        protocol_version: protocol.protocol_version.clone(),
        protocol_sha256: protocol.protocol_sha256.clone(),
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        known_limitations,
        reviewer_independent_from_stage_65_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_65_binding_confirmed: request
            .exact_current_stage_51_through_stage_65_binding_confirmed,
        reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed,
        one_target_one_algorithm_three_frozen_seeds_only_confirmed: request
            .one_target_one_algorithm_three_frozen_seeds_only_confirmed,
        immutable_candidate_feature_preprocessing_and_target_confirmed: request
            .immutable_candidate_feature_preprocessing_and_target_confirmed,
        sealed_holdout_single_use_and_no_feedback_reuse_confirmed: request
            .sealed_holdout_single_use_and_no_feedback_reuse_confirmed,
        fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: request
            .fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed,
        all_three_seeds_must_pass_and_failures_remain_visible_confirmed: request
            .all_three_seeds_must_pass_and_failures_remain_visible_confirmed,
        insufficient_sample_fails_closed_confirmed: request
            .insufficient_sample_fails_closed_confirmed,
        no_cross_target_composite_tuning_refit_or_reselection_confirmed: request
            .no_cross_target_composite_tuning_refit_or_reselection_confirmed,
        protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed: request
            .protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed,
        next_gate_is_implementation_registration_not_data_access_confirmed: request
            .next_gate_is_implementation_registration_not_data_access_confirmed,
        no_selection_store_reward_shadow_order_broker_or_trading_confirmed: request
            .no_selection_store_reward_shadow_order_broker_or_trading_confirmed,
        protocol_independently_approved: false,
        future_sealed_holdout_evaluation_implementation_registration_eligible: false,
        official_candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        sealed_holdout_evaluation_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approving = review.verdict
        == HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration;
    let approved = approving && independent && review_all_checks(&review);
    review.protocol_independently_approved = approved;
    review.future_sealed_holdout_evaluation_implementation_registration_eligible = approved;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, protocol)?;
    write_immutable_json(
        &review_directory(state, attempt_id, target_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    subject: &HistoricalOutcomeSealedHoldoutEvaluationProtocolSubject,
    request: &ReviewHistoricalOutcomeSealedHoldoutEvaluationProtocolRequest,
) -> Result<(), String> {
    let protocol = &subject.protocol;
    if request.expected_stage_65_admission_review_id == protocol.stage_65_admission_review_id
        && request.expected_stage_65_admission_review_sha256
            == protocol.stage_65_admission_review_sha256
        && request.expected_output_validation_sha256 == protocol.output_validation_sha256
        && request.expected_candidate_set_sha256 == protocol.candidate_set_sha256
        && request.expected_training_store_dataset_sha256 == protocol.training_store_dataset_sha256
        && request.expected_target_bundle_sha256 == protocol.target_bundle_sha256
        && request.expected_recommendation_sha256 == protocol.recommendation_sha256
        && request.expected_protocol_sha256 == protocol.protocol_sha256
    {
        Ok(())
    } else {
        Err("Stage 51-65 或 sealed-holdout 协议精确绑定已经变化，请刷新后重试".to_string())
    }
}

fn validate_protocol(
    protocol: &HistoricalOutcomeSealedHoldoutEvaluationProtocol,
) -> Result<(), String> {
    let downstream_closed = !protocol.callable_entrypoint_present
        && !protocol.sealed_holdout_features_access_allowed
        && !protocol.sealed_holdout_labels_access_allowed
        && !protocol.training_or_preprocessing_update_allowed
        && !protocol.hyperparameter_or_threshold_tuning_allowed
        && !protocol.candidate_reselection_allowed
        && !protocol.model_store_write_allowed
        && !protocol.metric_store_write_allowed
        && !protocol.production_reads_allowed
        && !protocol.production_writes_allowed
        && !protocol.outbound_network_allowed
        && !protocol.secrets_allowed
        && !protocol.scalar_reward_defined
        && !protocol.action_position_or_ranking_semantics_defined;
    let algorithm_valid = matches!(
        protocol.frozen_candidate_algorithm_id.as_str(),
        "ridge_multi_target_regression" | "gradient_boosted_multi_target_regression"
    );
    if protocol.schema_version != PROTOCOL_SCHEMA_VERSION
        || protocol.protocol_version != PROTOCOL_VERSION
        || !valid_sha256(&protocol.protocol_sha256)
        || protocol.protocol_sha256 != fingerprint_without(protocol, &["protocol_sha256"])?
        || !valid_id(&protocol.attempt_id)
        || !valid_target_id(&protocol.target_id)
        || !valid_id(&protocol.stage_65_admission_review_id)
        || [
            protocol.stage_65_admission_review_sha256.as_str(),
            protocol.output_validation_sha256.as_str(),
            protocol.candidate_set_sha256.as_str(),
            protocol.training_store_dataset_sha256.as_str(),
            protocol.rows_sha256.as_str(),
            protocol.target_commitments_sha256.as_str(),
            protocol.validation_projection_sha256.as_str(),
            protocol.target_bundle_sha256.as_str(),
            protocol.recommendation_sha256.as_str(),
            protocol
                .selected_algorithm_three_seed_binding_sha256
                .as_str(),
            protocol.sealed_holdout_split_commitment_sha256.as_str(),
            protocol.feature_order_sha256.as_str(),
            protocol.preprocessing_sha256.as_str(),
        ]
        .iter()
        .any(|value| !valid_sha256(value))
        || !algorithm_valid
        || protocol.exact_random_seeds != EXPECTED_SEEDS
        || protocol.exact_feature_count != EXPECTED_FEATURE_COUNT
        || protocol.exact_target_count != EXPECTED_TARGET_COUNT
        || protocol.target_vector_order != [protocol.target_id.clone()]
        || protocol.benchmark_algorithm_id != "frozen_zero_prediction_baseline"
        || protocol.reported_metric_ids.len() != 7
        || protocol.bootstrap_unit != "official_split_component_id"
        || protocol.bootstrap_replications != BOOTSTRAP_REPLICATIONS
        || protocol.bootstrap_random_seed != BOOTSTRAP_RANDOM_SEED
        || protocol.family_wise_error_correction
            != "holm_bonferroni_across_exact_three_frozen_seed_tests_for_one_target"
        || protocol.family_wise_alpha_millionths != FAMILY_WISE_ALPHA_MILLIONTHS
        || protocol.exact_candidate_hypothesis_count != EXPECTED_HYPOTHESIS_COUNT
        || protocol.minimum_relative_mae_improvement_ppm != MINIMUM_RELATIVE_MAE_IMPROVEMENT_PPM
        || protocol.minimum_spearman_millionths != 0
        || protocol.minimum_directional_accuracy_millionths != 500_000
        || protocol.minimum_calibration_slope_millionths != 500_000
        || protocol.maximum_calibration_slope_millionths != 1_500_000
        || protocol.minimum_sealed_holdout_rows != MINIMUM_SEALED_HOLDOUT_ROWS
        || protocol.minimum_independent_components != MINIMUM_INDEPENDENT_COMPONENTS
        || !protocol.all_three_seeds_must_pass
        || !protocol.one_shot_evaluation_required
        || protocol.insufficient_sample_rule.trim().is_empty()
        || protocol.confirmatory_decision_rule.trim().is_empty()
        || protocol.no_feedback_reuse_rule.trim().is_empty()
        || !protocol.no_composite_score_or_cross_target_masking
        || !protocol.protocol_review_only
        || !downstream_closed
    {
        return Err("sealed-holdout 逐目标评估协议无效或越权".to_string());
    }
    Ok(())
}

fn review_all_checks(review: &HistoricalOutcomeSealedHoldoutEvaluationProtocolReview) -> bool {
    review.exact_current_stage_51_through_stage_65_binding_confirmed
        && review.reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed
        && review.one_target_one_algorithm_three_frozen_seeds_only_confirmed
        && review.immutable_candidate_feature_preprocessing_and_target_confirmed
        && review.sealed_holdout_single_use_and_no_feedback_reuse_confirmed
        && review.fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed
        && review.all_three_seeds_must_pass_and_failures_remain_visible_confirmed
        && review.insufficient_sample_fails_closed_confirmed
        && review.no_cross_target_composite_tuning_refit_or_reselection_confirmed
        && review.protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed
        && review.next_gate_is_implementation_registration_not_data_access_confirmed
        && review.no_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn validate_review(
    review: &HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
    protocol: &HistoricalOutcomeSealedHoldoutEvaluationProtocol,
) -> Result<(), String> {
    validate_protocol(protocol)?;
    let approving = review.verdict
        == HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration;
    let independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_approved = approving && independent && review_all_checks(review);
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let downstream_closed = !review.official_candidate_selection_authorized
        && !review.sealed_holdout_access_authorized
        && !review.sealed_holdout_evaluation_authorized
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
        || review.review_sha256 != review_fingerprint(review)?
        || !previous_pair_valid
        || review.attempt_id != protocol.attempt_id
        || review.target_id != protocol.target_id
        || review.stage_65_admission_review_id != protocol.stage_65_admission_review_id
        || review.stage_65_admission_review_sha256 != protocol.stage_65_admission_review_sha256
        || review.output_validation_sha256 != protocol.output_validation_sha256
        || review.candidate_set_sha256 != protocol.candidate_set_sha256
        || review.training_store_dataset_sha256 != protocol.training_store_dataset_sha256
        || review.target_bundle_sha256 != protocol.target_bundle_sha256
        || review.recommendation_sha256 != protocol.recommendation_sha256
        || review.protocol_version != protocol.protocol_version
        || review.protocol_sha256 != protocol.protocol_sha256
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || review.known_limitations.trim().is_empty()
        || review.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || review.reviewer_id.trim().is_empty()
        || review.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || review.reviewer_independent_from_stage_65_and_complete_prior_chain != independent
        || (approving && !review_all_checks(review))
        || review.protocol_independently_approved != expected_approved
        || review.future_sealed_holdout_evaluation_implementation_registration_eligible
            != expected_approved
        || !downstream_closed
    {
        return Err("sealed-holdout protocol review is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn excluded_actor_ids(
    admission: &HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview,
    latest: &Option<HistoricalOutcomeSealedHoldoutEvaluationProtocolReview>,
) -> Vec<String> {
    let mut actors = admission.excluded_prior_actor_ids.clone();
    actors.push(admission.reviewer_id.clone());
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

fn review_fingerprint(
    review: &HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    subject: &HistoricalOutcomeSealedHoldoutEvaluationProtocolSubject,
) -> Result<Option<HistoricalOutcomeSealedHoldoutEvaluationProtocolReview>, String> {
    let directory = review_directory(
        state,
        &subject.protocol.attempt_id,
        &subject.protocol.target_id,
    );
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
        let bytes = tokio::fs::read(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        let review: HistoricalOutcomeSealedHoldoutEvaluationProtocolReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, &subject.protocol)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err("sealed-holdout 协议复核包含文件名不符或重复记录".to_string());
        }
        reviews.push(review);
    }
    validate_chain_and_latest(
        &reviews,
        &excluded_actor_ids(&subject.admitted.admission_review, &None),
    )
}

fn validate_chain_and_latest(
    reviews: &[HistoricalOutcomeSealedHoldoutEvaluationProtocolReview],
    base_excluded_actor_ids: &[String],
) -> Result<Option<HistoricalOutcomeSealedHoldoutEvaluationProtocolReview>, String> {
    if reviews.is_empty() {
        return Ok(None);
    }
    let by_id = reviews
        .iter()
        .map(|review| (review.review_id.as_str(), review))
        .collect::<HashMap<_, _>>();
    let roots = reviews
        .iter()
        .filter(|review| review.previous_review_id.is_none())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("sealed-holdout 协议复核链必须精确一个根".to_string());
    }
    for review in reviews {
        let expected_excluded = match review.previous_review_id.as_deref() {
            Some(previous_id) => {
                let parent = by_id
                    .get(previous_id)
                    .ok_or_else(|| "sealed-holdout 协议复核链存在断链".to_string())?;
                if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                    return Err("sealed-holdout 协议复核前序哈希不匹配".to_string());
                }
                if parent.protocol_independently_approved {
                    return Err("已批准 sealed-holdout 协议复核后不得追加记录".to_string());
                }
                let mut values = parent.excluded_prior_actor_ids.clone();
                values.push(parent.reviewer_id.clone());
                values.sort();
                values.dedup();
                values
            }
            None => base_excluded_actor_ids.to_vec(),
        };
        if review.excluded_prior_actor_ids != expected_excluded {
            return Err("sealed-holdout 协议复核职责排除链不连续".to_string());
        }
    }
    let tips = reviews
        .iter()
        .filter(|review| {
            !reviews.iter().any(|candidate| {
                candidate.previous_review_id.as_deref() == Some(review.review_id.as_str())
            })
        })
        .collect::<Vec<_>>();
    if tips.len() != 1 {
        return Err("sealed-holdout 协议复核链出现分叉或没有链尖".to_string());
    }
    let mut seen = BTreeSet::new();
    let mut cursor = Some(tips[0]);
    while let Some(review) = cursor {
        if !seen.insert(review.review_id.as_str()) {
            return Err("sealed-holdout 协议复核链出现循环".to_string());
        }
        cursor = review
            .previous_review_id
            .as_deref()
            .and_then(|previous| by_id.get(previous).copied());
    }
    if seen.len() != reviews.len() {
        return Err("sealed-holdout 协议复核链包含断开的记录".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn algorithm_id(algorithm: &HistoricalOutcomeTrainingExperimentAlgorithm) -> &'static str {
    match algorithm {
        HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline => {
            "frozen_zero_prediction_baseline"
        }
        HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression => {
            "ridge_multi_target_regression"
        }
        HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression => {
            "gradient_boosted_multi_target_regression"
        }
    }
}

fn review_directory(state: &AppState, attempt_id: &str, target_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-sealed-holdout-evaluation-protocol-reviews")
        .join(attempt_id)
        .join(target_id)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

async fn acquire_lock(
    state: &AppState,
    attempt_id: &str,
    target_id: &str,
) -> Result<DirectoryLock, String> {
    let path = decision_root(state)
        .join("historical-outcome-sealed-holdout-evaluation-protocol-review-locks")
        .join(format!("{attempt_id}-{target_id}.lock"));
    acquire_directory_lock(&path).await
}

struct DirectoryLock {
    path: PathBuf,
}

impl Drop for DirectoryLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir(&self.path);
    }
}

async fn acquire_directory_lock(path: &Path) -> Result<DirectoryLock, String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    match tokio::fs::create_dir(path).await {
        Ok(()) => Ok(DirectoryLock {
            path: path.to_path_buf(),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = tokio::fs::metadata(path)
                .await
                .map_err(|error| error.to_string())?;
            let stale = metadata
                .modified()
                .ok()
                .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                .is_some_and(|age| age > StdDuration::from_secs(120));
            if stale {
                tokio::fs::remove_dir_all(path)
                    .await
                    .map_err(|error| error.to_string())?;
                tokio::fs::create_dir(path)
                    .await
                    .map_err(|error| error.to_string())?;
                Ok(DirectoryLock {
                    path: path.to_path_buf(),
                })
            } else {
                Err("sealed-holdout 协议复核正在处理中，请稍后重试".to_string())
            }
        }
        Err(error) => Err(error.to_string()),
    }
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    tokio::fs::write(&temporary, bytes)
        .await
        .map_err(|error| error.to_string())?;
    match tokio::fs::hard_link(&temporary, path).await {
        Ok(()) => {
            tokio::fs::remove_file(&temporary)
                .await
                .map_err(|error| error.to_string())?;
            Ok(())
        }
        Err(error) => {
            let _ = tokio::fs::remove_file(&temporary).await;
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                Err("sealed-holdout 协议复核记录已存在，不能覆盖".to_string())
            } else {
                Err(error.to_string())
            }
        }
    }
}

fn fingerprint_without<T: Serialize>(value: &T, keys: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "fingerprint value must be object".to_string())?;
    for key in keys {
        object.remove(*key);
    }
    hash_serializable(&json)
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max: usize, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if trimmed.chars().count() > max {
        return Err(format!("{label}不能超过 {max} 字"));
    }
    Ok(trimmed.to_string())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_target_id(value: &str) -> bool {
    valid_id(value) && value.len() <= 120
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protocol_fixture() -> HistoricalOutcomeSealedHoldoutEvaluationProtocol {
        let mut value = HistoricalOutcomeSealedHoldoutEvaluationProtocol {
            schema_version: PROTOCOL_SCHEMA_VERSION.to_string(),
            protocol_version: PROTOCOL_VERSION.to_string(),
            protocol_sha256: String::new(),
            attempt_id: "attempt-66".to_string(),
            target_id: "excess_return_250_continuous".to_string(),
            stage_65_admission_review_id: "a".repeat(32),
            stage_65_admission_review_sha256: "1".repeat(64),
            output_validation_sha256: "2".repeat(64),
            candidate_set_sha256: "3".repeat(64),
            training_store_dataset_sha256: "4".repeat(64),
            rows_sha256: "5".repeat(64),
            target_commitments_sha256: "6".repeat(64),
            validation_projection_sha256: "7".repeat(64),
            target_bundle_sha256: "8".repeat(64),
            recommendation_sha256: "9".repeat(64),
            selected_algorithm_three_seed_binding_sha256: "a".repeat(64),
            sealed_holdout_split_commitment_sha256: "b".repeat(64),
            feature_order_sha256: "c".repeat(64),
            preprocessing_sha256: "d".repeat(64),
            frozen_candidate_algorithm_id: "ridge_multi_target_regression".to_string(),
            exact_random_seeds: EXPECTED_SEEDS.to_vec(),
            exact_feature_count: EXPECTED_FEATURE_COUNT,
            exact_target_count: EXPECTED_TARGET_COUNT,
            target_vector_order: vec!["excess_return_250_continuous".to_string()],
            benchmark_algorithm_id: "frozen_zero_prediction_baseline".to_string(),
            reported_metric_ids: (0..7).map(|index| format!("metric-{index}")).collect(),
            bootstrap_unit: "official_split_component_id".to_string(),
            bootstrap_replications: BOOTSTRAP_REPLICATIONS,
            bootstrap_random_seed: BOOTSTRAP_RANDOM_SEED,
            family_wise_error_correction:
                "holm_bonferroni_across_exact_three_frozen_seed_tests_for_one_target".to_string(),
            family_wise_alpha_millionths: FAMILY_WISE_ALPHA_MILLIONTHS,
            exact_candidate_hypothesis_count: EXPECTED_HYPOTHESIS_COUNT,
            minimum_relative_mae_improvement_ppm: MINIMUM_RELATIVE_MAE_IMPROVEMENT_PPM,
            minimum_spearman_millionths: 0,
            minimum_directional_accuracy_millionths: 500_000,
            minimum_calibration_slope_millionths: 500_000,
            maximum_calibration_slope_millionths: 1_500_000,
            minimum_sealed_holdout_rows: MINIMUM_SEALED_HOLDOUT_ROWS,
            minimum_independent_components: MINIMUM_INDEPENDENT_COMPONENTS,
            all_three_seeds_must_pass: true,
            one_shot_evaluation_required: true,
            insufficient_sample_rule: "insufficient fails closed".to_string(),
            confirmatory_decision_rule: "all seeds pass".to_string(),
            no_feedback_reuse_rule: "no feedback reuse".to_string(),
            no_composite_score_or_cross_target_masking: true,
            protocol_review_only: true,
            callable_entrypoint_present: false,
            sealed_holdout_features_access_allowed: false,
            sealed_holdout_labels_access_allowed: false,
            training_or_preprocessing_update_allowed: false,
            hyperparameter_or_threshold_tuning_allowed: false,
            candidate_reselection_allowed: false,
            model_store_write_allowed: false,
            metric_store_write_allowed: false,
            production_reads_allowed: false,
            production_writes_allowed: false,
            outbound_network_allowed: false,
            secrets_allowed: false,
            scalar_reward_defined: false,
            action_position_or_ranking_semantics_defined: false,
        };
        value.protocol_sha256 = fingerprint_without(&value, &["protocol_sha256"]).unwrap();
        value
    }

    fn review_fixture(
        protocol: &HistoricalOutcomeSealedHoldoutEvaluationProtocol,
        verdict: HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict,
        reviewer: &str,
        excluded: Vec<String>,
    ) -> HistoricalOutcomeSealedHoldoutEvaluationProtocolReview {
        let approving = verdict
            == HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration;
        let mut value = HistoricalOutcomeSealedHoldoutEvaluationProtocolReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id: None,
            previous_review_sha256: None,
            attempt_id: protocol.attempt_id.clone(),
            target_id: protocol.target_id.clone(),
            stage_65_admission_review_id: protocol.stage_65_admission_review_id.clone(),
            stage_65_admission_review_sha256: protocol.stage_65_admission_review_sha256.clone(),
            output_validation_sha256: protocol.output_validation_sha256.clone(),
            candidate_set_sha256: protocol.candidate_set_sha256.clone(),
            training_store_dataset_sha256: protocol.training_store_dataset_sha256.clone(),
            target_bundle_sha256: protocol.target_bundle_sha256.clone(),
            recommendation_sha256: protocol.recommendation_sha256.clone(),
            protocol_version: protocol.protocol_version.clone(),
            protocol_sha256: protocol.protocol_sha256.clone(),
            submitted_at: Utc::now(),
            reviewer_id: reviewer.to_string(),
            excluded_prior_actor_ids: excluded,
            verdict,
            rationale: "independent protocol review".to_string(),
            known_limitations: "not yet executed".to_string(),
            reviewer_independent_from_stage_65_and_complete_prior_chain: true,
            exact_current_stage_51_through_stage_65_binding_confirmed: true,
            reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed: true,
            one_target_one_algorithm_three_frozen_seeds_only_confirmed: true,
            immutable_candidate_feature_preprocessing_and_target_confirmed: true,
            sealed_holdout_single_use_and_no_feedback_reuse_confirmed: true,
            fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: true,
            all_three_seeds_must_pass_and_failures_remain_visible_confirmed: true,
            insufficient_sample_fails_closed_confirmed: true,
            no_cross_target_composite_tuning_refit_or_reselection_confirmed: true,
            protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed: true,
            next_gate_is_implementation_registration_not_data_access_confirmed: true,
            no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
            protocol_independently_approved: approving,
            future_sealed_holdout_evaluation_implementation_registration_eligible: approving,
            official_candidate_selection_authorized: false,
            sealed_holdout_access_authorized: false,
            sealed_holdout_evaluation_authorized: false,
            model_artifact_store_written: false,
            metric_store_written: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        value.review_sha256 = review_fingerprint(&value).unwrap();
        value.review_id = value.review_sha256[..32].to_string();
        value
    }

    #[test]
    fn protocol_is_one_target_three_seed_and_review_only() {
        let value = protocol_fixture();
        validate_protocol(&value).unwrap();
        assert_eq!(value.exact_target_count, 1);
        assert_eq!(value.exact_random_seeds, [17, 29, 43]);
        assert_eq!(value.exact_candidate_hypothesis_count, 3);
        assert!(value.one_shot_evaluation_required);
        assert!(!value.sealed_holdout_features_access_allowed);
        assert!(!value.sealed_holdout_labels_access_allowed);
        assert!(!value.callable_entrypoint_present);
    }

    #[test]
    fn protocol_hash_or_seed_drift_fails_closed() {
        let mut value = protocol_fixture();
        value.exact_random_seeds = vec![17, 29, 99];
        assert!(validate_protocol(&value).is_err());
        value = protocol_fixture();
        value.protocol_sha256 = "f".repeat(64);
        assert!(validate_protocol(&value).is_err());
    }

    #[test]
    fn protocol_rejects_access_tuning_and_model_store_authority() {
        let mut value = protocol_fixture();
        value.sealed_holdout_labels_access_allowed = true;
        assert!(validate_protocol(&value).is_err());
        value = protocol_fixture();
        value.hyperparameter_or_threshold_tuning_allowed = true;
        assert!(validate_protocol(&value).is_err());
        value = protocol_fixture();
        value.model_store_write_allowed = true;
        assert!(validate_protocol(&value).is_err());
    }

    #[test]
    fn approval_requires_every_protocol_confirmation() {
        let protocol = protocol_fixture();
        let mut review = review_fixture(
            &protocol,
            HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration,
            "reviewer-66",
            vec!["stage-65-reviewer".to_string()],
        );
        review.insufficient_sample_fails_closed_confirmed = false;
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review, &protocol).is_err());
    }

    #[test]
    fn approval_still_keeps_holdout_selection_and_trading_closed() {
        let protocol = protocol_fixture();
        let review = review_fixture(
            &protocol,
            HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration,
            "reviewer-66",
            vec!["stage-65-reviewer".to_string()],
        );
        validate_review(&review, &protocol).unwrap();
        assert!(review.protocol_independently_approved);
        assert!(review.future_sealed_holdout_evaluation_implementation_registration_eligible);
        assert!(!review.sealed_holdout_access_authorized);
        assert!(!review.sealed_holdout_evaluation_authorized);
        assert!(!review.official_candidate_selection_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn reviewer_may_not_overlap_excluded_chain() {
        assert!(!reviewer_is_independent(
            "stage-65-reviewer",
            &["stage-65-reviewer".to_string()]
        ));
        assert!(reviewer_is_independent(
            "new-reviewer",
            &["stage-65-reviewer".to_string()]
        ));
    }

    #[test]
    fn review_chain_rejects_branching_and_approval_children() {
        let protocol = protocol_fixture();
        let root = review_fixture(
            &protocol,
            HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ChangesRequested,
            "reviewer-a",
            vec!["stage-65-reviewer".to_string()],
        );
        let mut child_a = review_fixture(
            &protocol,
            HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::Rejected,
            "reviewer-b",
            vec!["reviewer-a".to_string(), "stage-65-reviewer".to_string()],
        );
        child_a.previous_review_id = Some(root.review_id.clone());
        child_a.previous_review_sha256 = Some(root.review_sha256.clone());
        child_a.review_sha256 = review_fingerprint(&child_a).unwrap();
        child_a.review_id = child_a.review_sha256[..32].to_string();
        let mut child_b = child_a.clone();
        child_b.reviewer_id = "reviewer-c".to_string();
        child_b.review_sha256 = review_fingerprint(&child_b).unwrap();
        child_b.review_id = child_b.review_sha256[..32].to_string();
        assert!(
            validate_chain_and_latest(
                &[root.clone(), child_a, child_b],
                &["stage-65-reviewer".to_string()]
            )
            .is_err()
        );

        let approved_root = review_fixture(
            &protocol,
            HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration,
            "reviewer-a",
            vec!["stage-65-reviewer".to_string()],
        );
        let mut child = review_fixture(
            &protocol,
            HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::Rejected,
            "reviewer-b",
            vec!["reviewer-a".to_string(), "stage-65-reviewer".to_string()],
        );
        child.previous_review_id = Some(approved_root.review_id.clone());
        child.previous_review_sha256 = Some(approved_root.review_sha256.clone());
        child.review_sha256 = review_fingerprint(&child).unwrap();
        child.review_id = child.review_sha256[..32].to_string();
        assert!(
            validate_chain_and_latest(&[approved_root, child], &["stage-65-reviewer".to_string()])
                .is_err()
        );
    }

    #[test]
    fn linear_changes_requested_chain_keeps_latest_tip() {
        let protocol = protocol_fixture();
        let root = review_fixture(
            &protocol,
            HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ChangesRequested,
            "reviewer-a",
            vec!["stage-65-reviewer".to_string()],
        );
        let mut child = review_fixture(
            &protocol,
            HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::Rejected,
            "reviewer-b",
            vec!["reviewer-a".to_string(), "stage-65-reviewer".to_string()],
        );
        child.previous_review_id = Some(root.review_id.clone());
        child.previous_review_sha256 = Some(root.review_sha256.clone());
        child.review_sha256 = review_fingerprint(&child).unwrap();
        child.review_id = child.review_sha256[..32].to_string();
        let latest =
            validate_chain_and_latest(&[root, child.clone()], &["stage-65-reviewer".to_string()])
                .unwrap()
                .unwrap();
        assert_eq!(latest.review_id, child.review_id);
    }
}
