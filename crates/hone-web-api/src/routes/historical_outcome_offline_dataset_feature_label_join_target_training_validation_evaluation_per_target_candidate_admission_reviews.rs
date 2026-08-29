//! Independent, append-only admission review for each exact Stage 64 target result.
//!
//! A target may advance only when its own frozen recommendation has sufficient evidence and the
//! recommended algorithm passed every preregistered threshold on all three frozen seeds. An
//! approval opens only a future sealed-holdout evaluation *protocol review*. It does not expose
//! the sealed holdout, make an official model selection, write a model/metric store, train,
//! reward, shadow, generate an order, access a broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_execution_attempts::{
    HistoricalOutcomeValidationEvaluationMetric,
    HistoricalOutcomeValidationEvaluationPerTargetRecommendation,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::HistoricalOutcomeTrainingExperimentAlgorithm;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_validation_evaluation_output_validations::{
    HistoricalOutcomeValidationEvaluationOutputValidationRecord,
    IndependentlyValidatedHistoricalOutcomeValidationEvaluationOutput,
    independently_validated_validation_evaluation_outputs_for_candidate_admission_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-per-target-candidate-admission-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-validation-evaluation-per-target-candidate-admission-review-v1";
const POLICY_VERSION: &str =
    "hone-validation-evaluation-independent-per-target-candidate-admission-v1-no-holdout-access";
const EXPECTED_METRICS_PER_TARGET: usize = 9;
const EXPECTED_ALGORITHMS_PER_TARGET: usize = 3;
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const RECOMMENDABLE_STATUS: &str =
    "untrusted_per_target_recommendation_pending_independent_validation";
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 2_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict {
    AdmittedForFutureSealedHoldoutEvaluationProtocolReview,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_output_validation_id: String,
    expected_output_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_implementation_sha256: String,
    expected_implementation_review_sha256: String,
    expected_candidate_set_sha256: String,
    expected_upstream_validation_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_validation_projection_sha256: String,
    expected_target_bundle_sha256: String,
    expected_recommendation_sha256: String,
    verdict: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_through_stage_64_binding_confirmed: bool,
    exact_target_only_nine_metrics_three_algorithms_three_seeds_confirmed: bool,
    target_evidence_status_and_thresholds_confirmed: bool,
    recommended_algorithm_and_three_seed_median_confirmed: bool,
    no_cross_target_composite_or_masking_confirmed: bool,
    sealed_holdout_remains_unread_confirmed: bool,
    next_gate_is_protocol_review_not_holdout_execution_confirmed: bool,
    no_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationPerTargetCandidate {
    pub source: IndependentlyValidatedHistoricalOutcomeValidationEvaluationOutput,
    pub target_id: String,
    pub target_bundle_sha256: String,
    pub recommendation_sha256: String,
    pub metrics: Vec<HistoricalOutcomeValidationEvaluationMetric>,
    pub recommendation: HistoricalOutcomeValidationEvaluationPerTargetRecommendation,
    pub exact_nine_metrics_three_algorithms_three_seeds: bool,
    pub recommendation_admissible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub target_id: String,
    pub output_validation_id: String,
    pub output_validation_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_spec_sha256: String,
    pub implementation_sha256: String,
    pub implementation_review_sha256: String,
    pub candidate_set_sha256: String,
    pub upstream_validation_sha256: String,
    pub training_store_dataset_sha256: String,
    pub validation_projection_sha256: String,
    pub target_bundle_sha256: String,
    pub recommendation_sha256: String,
    pub target_metric_count: usize,
    pub target_algorithm_count: usize,
    pub frozen_seed_count: usize,
    pub recommendation_status: String,
    pub recommended_algorithm: Option<String>,
    pub three_seed_median_mae_f64_bits_hex: Option<String>,
    pub all_three_seeds_passed: bool,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub reviewer_independent_from_stage_64_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_64_binding_confirmed: bool,
    pub exact_target_only_nine_metrics_three_algorithms_three_seeds_confirmed: bool,
    pub target_evidence_status_and_thresholds_confirmed: bool,
    pub recommended_algorithm_and_three_seed_median_confirmed: bool,
    pub no_cross_target_composite_or_masking_confirmed: bool,
    pub sealed_holdout_remains_unread_confirmed: bool,
    pub next_gate_is_protocol_review_not_holdout_execution_confirmed: bool,
    pub no_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub per_target_candidate_admitted: bool,
    pub future_sealed_holdout_evaluation_protocol_review_eligible: bool,
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
pub(crate) struct HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionItem {
    pub candidate: HistoricalOutcomeValidationEvaluationPerTargetCandidate,
    pub latest_review:
        Option<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview>,
    pub review_eligible: bool,
    pub per_target_candidate_admitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionItem>,
    pub independently_validated_output_count: usize,
    pub target_candidate_count: usize,
    pub review_eligible_target_count: usize,
    pub reviewed_target_count: usize,
    pub admitted_target_count: usize,
    pub changes_requested_or_rejected_target_count: usize,
    pub insufficient_evidence_target_count: usize,
    pub no_candidate_passed_target_count: usize,
    pub future_sealed_holdout_evaluation_protocol_review_eligible_target_count: usize,
    pub admission_status: String,
    pub per_target_candidate_admission_review_available: bool,
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
pub(crate) struct HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReadinessSummary {
    pub target_candidate_count: usize,
    pub reviewed_target_count: usize,
    pub admitted_target_count: usize,
    pub changes_requested_or_rejected_target_count: usize,
    pub insufficient_evidence_target_count: usize,
    pub no_candidate_passed_target_count: usize,
    pub future_sealed_holdout_evaluation_protocol_review_eligible_target_count: usize,
    pub admission_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate {
    pub candidate: HistoricalOutcomeValidationEvaluationPerTargetCandidate,
    pub admission_review: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview,
}

pub(crate) async fn handle_get_historical_outcome_validation_evaluation_per_target_candidate_admission_reviews(
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
            warn!(%error, "per-target candidate admission registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果逐目标候选准入复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_validation_evaluation_per_target_candidate_admission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath((attempt_id, target_id)): AxumPath<(String, String)>,
    Json(request): Json<
        ReviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRequest,
    >,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_admission(&state, &admin.user_id, &attempt_id, &target_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_validation_evaluation_per_target_candidate_admission_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReadinessSummary, String>
{
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReadinessSummary {
            target_candidate_count: registry.target_candidate_count,
            reviewed_target_count: registry.reviewed_target_count,
            admitted_target_count: registry.admitted_target_count,
            changes_requested_or_rejected_target_count: registry
                .changes_requested_or_rejected_target_count,
            insufficient_evidence_target_count: registry.insufficient_evidence_target_count,
            no_candidate_passed_target_count: registry.no_candidate_passed_target_count,
            future_sealed_holdout_evaluation_protocol_review_eligible_target_count: registry
                .future_sealed_holdout_evaluation_protocol_review_eligible_target_count,
            admission_status: registry.admission_status,
        },
    )
}

pub(crate) async fn admitted_validation_evaluation_per_target_candidates_for_sealed_holdout_protocol_review(
    state: &AppState,
) -> Result<Vec<AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate>, String> {
    let registry = build_registry(state).await?;
    registry
        .items
        .into_iter()
        .filter(|item| item.per_target_candidate_admitted)
        .map(|item| {
            let admission_review = item
                .latest_review
                .ok_or_else(|| "逐目标准入状态缺少不可变复核记录".to_string())?;
            if !admission_review.per_target_candidate_admitted
                || !admission_review.future_sealed_holdout_evaluation_protocol_review_eligible
                || !item.candidate.recommendation_admissible
            {
                return Err("逐目标准入状态与当前候选不一致".to_string());
            }
            Ok(
                AdmittedHistoricalOutcomeValidationEvaluationPerTargetCandidate {
                    candidate: item.candidate,
                    admission_review,
                },
            )
        })
        .collect()
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry, String> {
    let sources =
        independently_validated_validation_evaluation_outputs_for_candidate_admission_review(state)
            .await?;
    let independently_validated_output_count = sources.len();
    let mut items = Vec::new();
    for source in sources {
        for candidate in per_target_candidates(source)? {
            let latest_review = latest_review(state, &candidate).await?;
            let admitted = latest_review
                .as_ref()
                .is_some_and(|review| review.per_target_candidate_admitted);
            items.push(
                HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionItem {
                    review_eligible: !admitted,
                    candidate,
                    latest_review,
                    per_target_candidate_admitted: admitted,
                },
            );
        }
    }
    items.sort_by(|left, right| {
        left.candidate
            .source
            .attempt
            .claim
            .attempt_id
            .cmp(&right.candidate.source.attempt.claim.attempt_id)
            .then_with(|| left.candidate.target_id.cmp(&right.candidate.target_id))
    });
    let target_candidate_count = items.len();
    let review_eligible_target_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_target_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let admitted_target_count = items
        .iter()
        .filter(|item| item.per_target_candidate_admitted)
        .count();
    let changes_requested_or_rejected_target_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    != HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict::AdmittedForFutureSealedHoldoutEvaluationProtocolReview
            })
        })
        .count();
    let insufficient_evidence_target_count = items
        .iter()
        .filter(|item| item.candidate.recommendation.status == "insufficient_evidence")
        .count();
    let no_candidate_passed_target_count = items
        .iter()
        .filter(|item| {
            item.candidate.recommendation.status == "no_candidate_passed_all_three_seeds"
        })
        .count();
    let admission_status = if admitted_target_count > 0 {
        "per_target_candidates_admitted_waiting_sealed_holdout_evaluation_protocol_review"
    } else if changes_requested_or_rejected_target_count > 0 {
        "per_target_candidate_admission_changes_requested_or_rejected"
    } else if target_candidate_count > 0 {
        "waiting_independent_per_target_candidate_admission_review"
    } else {
        "waiting_independently_validated_validation_evaluation_output"
    };
    Ok(HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        independently_validated_output_count,
        target_candidate_count,
        review_eligible_target_count,
        reviewed_target_count,
        admitted_target_count,
        changes_requested_or_rejected_target_count,
        insufficient_evidence_target_count,
        no_candidate_passed_target_count,
        future_sealed_holdout_evaluation_protocol_review_eligible_target_count:
            admitted_target_count,
        admission_status: admission_status.to_string(),
        per_target_candidate_admission_review_available: true,
        official_candidate_selection_authorized: false,
        sealed_holdout_access_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 65 阶段把 Stage 64 独立复算通过的九个目标完全拆开复核。每个目标单独绑定三种算法×三个冻结种子的九项指标、证据状态、三种子门槛和建议哈希；证据不足或无候选三种子全过时不得准入，其他目标不能掩盖。批准只开放未来 sealed-holdout 评估协议复核资格，不读取留出集、不正式选模、不写模型/指标库，也不训练、奖励、影子、订单、券商或交易。".to_string(),
    })
}

fn per_target_candidates(
    source: IndependentlyValidatedHistoricalOutcomeValidationEvaluationOutput,
) -> Result<Vec<HistoricalOutcomeValidationEvaluationPerTargetCandidate>, String> {
    let envelope = source
        .attempt
        .result
        .untrusted_evaluation_envelope
        .as_ref()
        .ok_or_else(|| "Stage 64 来源缺少 validation 评估信封".to_string())?;
    let mut output = Vec::with_capacity(envelope.target_order.len());
    for target_id in &envelope.target_order {
        if !valid_target_id(target_id) {
            return Err(format!("目标 ID {target_id} 无效"));
        }
        let metrics = envelope
            .metrics
            .iter()
            .filter(|metric| &metric.target_id == target_id)
            .cloned()
            .collect::<Vec<_>>();
        let recommendations = envelope
            .per_target_recommendations
            .iter()
            .filter(|recommendation| &recommendation.target_id == target_id)
            .cloned()
            .collect::<Vec<_>>();
        if recommendations.len() != 1 {
            return Err(format!("目标 {target_id} 必须精确绑定一项建议"));
        }
        let recommendation = recommendations[0].clone();
        let exact_shape = exact_target_metric_shape(&metrics);
        let recommendation_sha256 = hash_serializable(&recommendation)?;
        let target_bundle_sha256 = hash_serializable(&serde_json::json!({
            "target_id": target_id,
            "metrics": metrics,
            "recommendation": recommendation,
        }))?;
        let recommendation_admissible = recommendation_is_admissible(&metrics, &recommendation);
        output.push(HistoricalOutcomeValidationEvaluationPerTargetCandidate {
            source: source.clone(),
            target_id: target_id.clone(),
            target_bundle_sha256,
            recommendation_sha256,
            metrics,
            recommendation,
            exact_nine_metrics_three_algorithms_three_seeds: exact_shape,
            recommendation_admissible,
        });
    }
    if output.len() != 9 {
        return Err("Stage 64 来源必须精确拆为九个逐目标候选".to_string());
    }
    Ok(output)
}

fn exact_target_metric_shape(metrics: &[HistoricalOutcomeValidationEvaluationMetric]) -> bool {
    if metrics.len() != EXPECTED_METRICS_PER_TARGET {
        return false;
    }
    let algorithms = metrics
        .iter()
        .filter_map(|metric| serde_json::to_string(&metric.algorithm).ok())
        .collect::<BTreeSet<_>>();
    let seeds = metrics
        .iter()
        .map(|metric| metric.random_seed)
        .collect::<BTreeSet<_>>();
    let pairs = metrics
        .iter()
        .filter_map(|metric| {
            serde_json::to_string(&metric.algorithm)
                .ok()
                .map(|algorithm| (algorithm, metric.random_seed))
        })
        .collect::<BTreeSet<_>>();
    algorithms.len() == EXPECTED_ALGORITHMS_PER_TARGET
        && seeds == EXPECTED_SEEDS.into_iter().collect()
        && pairs.len() == EXPECTED_METRICS_PER_TARGET
        && metrics
            .iter()
            .all(|metric| !metric.official_model_selection_metric)
}

fn recommendation_is_admissible(
    metrics: &[HistoricalOutcomeValidationEvaluationMetric],
    recommendation: &HistoricalOutcomeValidationEvaluationPerTargetRecommendation,
) -> bool {
    if !exact_target_metric_shape(metrics)
        || recommendation.status != RECOMMENDABLE_STATUS
        || !recommendation.all_three_seeds_passed
        || recommendation.official_selection
        || metrics
            .iter()
            .any(|metric| metric.evidence_status == "insufficient_evidence")
    {
        return false;
    }
    let Some(algorithm) = recommendation.recommended_algorithm.as_ref() else {
        return false;
    };
    if !matches!(
        algorithm,
        HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression
            | HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression
    ) {
        return false;
    }
    let selected = metrics
        .iter()
        .filter(|metric| &metric.algorithm == algorithm)
        .collect::<Vec<_>>();
    if selected.len() != EXPECTED_SEEDS.len()
        || selected
            .iter()
            .any(|metric| !metric.all_preregistered_thresholds_passed)
    {
        return false;
    }
    let mut maes = match selected
        .iter()
        .map(|metric| parse_f64_hex(&metric.mae_f64_bits_hex))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(values) => values,
        Err(_) => return false,
    };
    maes.sort_by(f64::total_cmp);
    recommendation.three_seed_median_mae_f64_bits_hex.as_deref() == Some(f64_hex(maes[1]).as_str())
}

fn parse_f64_hex(value: &str) -> Result<f64, String> {
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("f64 bits hex 无效".to_string());
    }
    let bits = u64::from_str_radix(value, 16).map_err(|error| error.to_string())?;
    let parsed = f64::from_bits(bits);
    if !parsed.is_finite() {
        return Err("f64 bits hex 不是有限值".to_string());
    }
    Ok(parsed)
}

fn f64_hex(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

async fn review_admission(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    target_id: &str,
    request: ReviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRequest,
) -> Result<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview, String> {
    if !valid_id(attempt_id) || !valid_target_id(target_id) {
        return Err("validation 评估尝试或目标 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id, target_id).await?;
    let sources =
        independently_validated_validation_evaluation_outputs_for_candidate_admission_review(state)
            .await?;
    let mut matching = Vec::new();
    for source in sources {
        matching.extend(
            per_target_candidates(source)?
                .into_iter()
                .filter(|candidate| {
                    candidate.source.attempt.claim.attempt_id == attempt_id
                        && candidate.target_id == target_id
                }),
        );
    }
    if matching.len() != 1 {
        return Err("当前没有精确一条可复核的逐目标候选".to_string());
    }
    let candidate = matching.remove(0);
    validate_expected_binding(&candidate, &request)?;
    let latest = latest_review(state, &candidate).await?;
    if latest
        .as_ref()
        .is_some_and(|review| review.per_target_candidate_admitted)
    {
        return Err("该逐目标候选已准入，复核链永久冻结；下一步只能进入协议复核".to_string());
    }
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("逐目标候选准入复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限与偏差",
    )?;
    let excluded_prior_actor_ids = excluded_actor_ids(&candidate.source.validation, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "逐目标准入复核人不能是 Stage 64 验证者、Stage 63 执行者、完整上游或此前复核角色"
                .to_string(),
        );
    }
    let validation = &candidate.source.validation;
    let recommended_algorithm = candidate
        .recommendation
        .recommended_algorithm
        .as_ref()
        .map(|value| serde_json::to_string(value))
        .transpose()
        .map_err(|error| error.to_string())?;
    let mut review = HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: attempt_id.to_string(),
        target_id: target_id.to_string(),
        output_validation_id: validation.validation_id.clone(),
        output_validation_sha256: validation.validation_sha256.clone(),
        claim_sha256: validation.claim_sha256.clone(),
        result_sha256: validation.result_sha256.clone(),
        output_sha256: validation.output_sha256.clone(),
        authorization_review_sha256: validation.authorization_review_sha256.clone(),
        isolated_runner_spec_sha256: validation.isolated_runner_spec_sha256.clone(),
        implementation_sha256: validation.implementation_sha256.clone(),
        implementation_review_sha256: validation.implementation_review_sha256.clone(),
        candidate_set_sha256: validation.candidate_set_sha256.clone(),
        upstream_validation_sha256: validation.upstream_validation_sha256.clone(),
        training_store_dataset_sha256: validation.training_store_dataset_sha256.clone(),
        validation_projection_sha256: validation.validation_projection_sha256.clone(),
        target_bundle_sha256: candidate.target_bundle_sha256.clone(),
        recommendation_sha256: candidate.recommendation_sha256.clone(),
        target_metric_count: candidate.metrics.len(),
        target_algorithm_count: candidate
            .metrics
            .iter()
            .filter_map(|metric| serde_json::to_string(&metric.algorithm).ok())
            .collect::<BTreeSet<_>>()
            .len(),
        frozen_seed_count: candidate
            .metrics
            .iter()
            .map(|metric| metric.random_seed)
            .collect::<BTreeSet<_>>()
            .len(),
        recommendation_status: candidate.recommendation.status.clone(),
        recommended_algorithm,
        three_seed_median_mae_f64_bits_hex: candidate
            .recommendation
            .three_seed_median_mae_f64_bits_hex
            .clone(),
        all_three_seeds_passed: candidate.recommendation.all_three_seeds_passed,
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        known_limitations,
        reviewer_independent_from_stage_64_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_64_binding_confirmed: request
            .exact_current_stage_51_through_stage_64_binding_confirmed,
        exact_target_only_nine_metrics_three_algorithms_three_seeds_confirmed: request
            .exact_target_only_nine_metrics_three_algorithms_three_seeds_confirmed,
        target_evidence_status_and_thresholds_confirmed: request
            .target_evidence_status_and_thresholds_confirmed,
        recommended_algorithm_and_three_seed_median_confirmed: request
            .recommended_algorithm_and_three_seed_median_confirmed,
        no_cross_target_composite_or_masking_confirmed: request
            .no_cross_target_composite_or_masking_confirmed,
        sealed_holdout_remains_unread_confirmed: request.sealed_holdout_remains_unread_confirmed,
        next_gate_is_protocol_review_not_holdout_execution_confirmed: request
            .next_gate_is_protocol_review_not_holdout_execution_confirmed,
        no_selection_store_reward_shadow_order_broker_or_trading_confirmed: request
            .no_selection_store_reward_shadow_order_broker_or_trading_confirmed,
        per_target_candidate_admitted: false,
        future_sealed_holdout_evaluation_protocol_review_eligible: false,
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
    let approving = review.verdict
        == HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict::AdmittedForFutureSealedHoldoutEvaluationProtocolReview;
    if approving && !candidate.recommendation_admissible {
        return Err("该目标证据不足或没有候选在三个冻结种子上全部通过，不能准入".to_string());
    }
    let admitted = approving
        && candidate.recommendation_admissible
        && independent
        && review_all_checks(&review);
    review.per_target_candidate_admitted = admitted;
    review.future_sealed_holdout_evaluation_protocol_review_eligible = admitted;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &candidate)?;
    write_immutable_json(
        &review_directory(state, attempt_id, target_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    candidate: &HistoricalOutcomeValidationEvaluationPerTargetCandidate,
    request: &ReviewHistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionRequest,
) -> Result<(), String> {
    let validation = &candidate.source.validation;
    let exact = request.expected_output_validation_id == validation.validation_id
        && request.expected_output_validation_sha256 == validation.validation_sha256
        && request.expected_claim_sha256 == validation.claim_sha256
        && request.expected_result_sha256 == validation.result_sha256
        && request.expected_output_sha256 == validation.output_sha256
        && request.expected_authorization_review_sha256 == validation.authorization_review_sha256
        && request.expected_isolated_runner_spec_sha256 == validation.isolated_runner_spec_sha256
        && request.expected_implementation_sha256 == validation.implementation_sha256
        && request.expected_implementation_review_sha256 == validation.implementation_review_sha256
        && request.expected_candidate_set_sha256 == validation.candidate_set_sha256
        && request.expected_upstream_validation_sha256 == validation.upstream_validation_sha256
        && request.expected_training_store_dataset_sha256
            == validation.training_store_dataset_sha256
        && request.expected_validation_projection_sha256 == validation.validation_projection_sha256
        && request.expected_target_bundle_sha256 == candidate.target_bundle_sha256
        && request.expected_recommendation_sha256 == candidate.recommendation_sha256;
    if exact {
        Ok(())
    } else {
        Err("逐目标候选的 Stage 51-64 精确绑定已经变化，请刷新后重试".to_string())
    }
}

fn review_all_checks(
    review: &HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview,
) -> bool {
    review.exact_current_stage_51_through_stage_64_binding_confirmed
        && review.exact_target_only_nine_metrics_three_algorithms_three_seeds_confirmed
        && review.target_evidence_status_and_thresholds_confirmed
        && review.recommended_algorithm_and_three_seed_median_confirmed
        && review.no_cross_target_composite_or_masking_confirmed
        && review.sealed_holdout_remains_unread_confirmed
        && review.next_gate_is_protocol_review_not_holdout_execution_confirmed
        && review.no_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn excluded_actor_ids(
    validation: &HistoricalOutcomeValidationEvaluationOutputValidationRecord,
    latest: &Option<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview>,
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
    review: &HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview,
    candidate: &HistoricalOutcomeValidationEvaluationPerTargetCandidate,
) -> Result<(), String> {
    let validation = &candidate.source.validation;
    let approving = review.verdict
        == HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict::AdmittedForFutureSealedHoldoutEvaluationProtocolReview;
    let independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_admitted = approving
        && candidate.recommendation_admissible
        && independent
        && review_all_checks(review);
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let downstream_closed = !review.official_candidate_selection_authorized
        && !review.sealed_holdout_access_authorized
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.reward_authorized
        && !review.shadow_portfolio_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let expected_recommended_algorithm = candidate
        .recommendation
        .recommended_algorithm
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| error.to_string())?;
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != POLICY_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.review_sha256)
        || review.review_id != review.review_sha256[..32]
        || review.review_sha256 != review_fingerprint(review)?
        || !previous_pair_valid
        || review.attempt_id != candidate.source.attempt.claim.attempt_id
        || review.target_id != candidate.target_id
        || review.output_validation_id != validation.validation_id
        || review.output_validation_sha256 != validation.validation_sha256
        || review.claim_sha256 != validation.claim_sha256
        || review.result_sha256 != validation.result_sha256
        || review.output_sha256 != validation.output_sha256
        || review.authorization_review_sha256 != validation.authorization_review_sha256
        || review.isolated_runner_spec_sha256 != validation.isolated_runner_spec_sha256
        || review.implementation_sha256 != validation.implementation_sha256
        || review.implementation_review_sha256 != validation.implementation_review_sha256
        || review.candidate_set_sha256 != validation.candidate_set_sha256
        || review.upstream_validation_sha256 != validation.upstream_validation_sha256
        || review.training_store_dataset_sha256 != validation.training_store_dataset_sha256
        || review.validation_projection_sha256 != validation.validation_projection_sha256
        || review.target_bundle_sha256 != candidate.target_bundle_sha256
        || review.recommendation_sha256 != candidate.recommendation_sha256
        || review.target_metric_count != EXPECTED_METRICS_PER_TARGET
        || review.target_algorithm_count != EXPECTED_ALGORITHMS_PER_TARGET
        || review.frozen_seed_count != EXPECTED_SEEDS.len()
        || review.recommendation_status != candidate.recommendation.status
        || review.recommended_algorithm != expected_recommended_algorithm
        || review.three_seed_median_mae_f64_bits_hex
            != candidate.recommendation.three_seed_median_mae_f64_bits_hex
        || review.all_three_seeds_passed != candidate.recommendation.all_three_seeds_passed
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
        || review.reviewer_independent_from_stage_64_and_complete_prior_chain != independent
        || (approving && (!candidate.recommendation_admissible || !review_all_checks(review)))
        || review.per_target_candidate_admitted != expected_admitted
        || review.future_sealed_holdout_evaluation_protocol_review_eligible != expected_admitted
        || !downstream_closed
    {
        return Err(
            "historical per-target candidate admission review is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn review_fingerprint(
    review: &HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    candidate: &HistoricalOutcomeValidationEvaluationPerTargetCandidate,
) -> Result<Option<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview>, String>
{
    let directory = review_directory(
        state,
        &candidate.source.attempt.claim.attempt_id,
        &candidate.target_id,
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
        let review: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, candidate)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err("逐目标候选准入复核包含文件名不符或重复记录".to_string());
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview],
) -> Result<Option<HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview>, String>
{
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
        return Err("逐目标候选准入复核链存在分叉或多个链尖".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("逐目标候选准入复核链存在循环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "逐目标候选准入复核链断裂".to_string())?;
                if previous.review_sha256 != *sha {
                    return Err("逐目标候选准入复核前序哈希不一致".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("逐目标候选准入复核前序链接无效".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("逐目标候选准入复核链断裂".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "逐目标准入指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > max_chars {
        return Err(format!("{label}不能超过 {max_chars} 个字符"));
    }
    Ok(value.to_string())
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

fn review_directory(state: &AppState, attempt_id: &str, target_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-validation-evaluation-per-target-candidate-admission-reviews")
        .join(attempt_id)
        .join(target_id)
}

struct AdmissionLock {
    path: PathBuf,
}

impl Drop for AdmissionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    attempt_id: &str,
    target_id: &str,
) -> Result<AdmissionLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-validation-evaluation-per-target-admission-{attempt_id}-{target_id}.lock"
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
        .map_err(|_| "已有同一逐目标候选准入复核正在进行".to_string())?;
    Ok(AdmissionLock { path })
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_target_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn metric(
        algorithm: HistoricalOutcomeTrainingExperimentAlgorithm,
        random_seed: u64,
    ) -> HistoricalOutcomeValidationEvaluationMetric {
        HistoricalOutcomeValidationEvaluationMetric {
            algorithm,
            random_seed,
            target_id: "excess_return_20d".to_string(),
            validation_row_count: 90,
            independent_component_count: 12,
            mae_f64_bits_hex: "3ff0000000000000".to_string(),
            zero_baseline_mae_f64_bits_hex: "4000000000000000".to_string(),
            relative_mae_improvement_f64_bits_hex: "3fd0000000000000".to_string(),
            component_block_bootstrap_p_value_f64_bits_hex: Some("3fa999999999999a".to_string()),
            holm_adjusted_p_value_f64_bits_hex: Some("3fa999999999999a".to_string()),
            spearman_f64_bits_hex: Some("3fd0000000000000".to_string()),
            directional_accuracy_f64_bits_hex: "3fe3333333333333".to_string(),
            calibration_slope_f64_bits_hex: Some("3ff0000000000000".to_string()),
            evidence_status: "sufficient_evidence".to_string(),
            all_preregistered_thresholds_passed: true,
            official_model_selection_metric: false,
        }
    }

    fn exact_metrics() -> Vec<HistoricalOutcomeValidationEvaluationMetric> {
        use HistoricalOutcomeTrainingExperimentAlgorithm::*;
        [
            FrozenZeroPredictionBaseline,
            RidgeMultiTargetRegression,
            GradientBoostedMultiTargetRegression,
        ]
        .into_iter()
        .flat_map(|algorithm| {
            EXPECTED_SEEDS
                .into_iter()
                .map(move |seed| metric(algorithm.clone(), seed))
        })
        .collect()
    }

    #[test]
    fn exact_target_shape_requires_three_algorithms_and_three_frozen_seeds() {
        assert!(exact_target_metric_shape(&exact_metrics()));
        let mut missing = exact_metrics();
        missing.pop();
        assert!(!exact_target_metric_shape(&missing));
    }

    #[test]
    fn duplicate_algorithm_seed_pair_fails_closed() {
        let mut metrics = exact_metrics();
        metrics[8] = metrics[7].clone();
        assert!(!exact_target_metric_shape(&metrics));
    }

    #[test]
    fn official_metric_flag_fails_closed() {
        let mut metrics = exact_metrics();
        metrics[0].official_model_selection_metric = true;
        assert!(!exact_target_metric_shape(&metrics));
    }

    #[test]
    fn recommendation_requires_exact_three_seed_median_and_all_thresholds() {
        let metrics = exact_metrics();
        let recommendation = HistoricalOutcomeValidationEvaluationPerTargetRecommendation {
            target_id: "excess_return_20d".to_string(),
            status: RECOMMENDABLE_STATUS.to_string(),
            recommended_algorithm: Some(
                HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
            ),
            three_seed_median_mae_f64_bits_hex: Some("3ff0000000000000".to_string()),
            rationale: "冻结规则建议".to_string(),
            all_three_seeds_passed: true,
            official_selection: false,
        };
        assert!(recommendation_is_admissible(&metrics, &recommendation));
        let mut wrong_median = recommendation.clone();
        wrong_median.three_seed_median_mae_f64_bits_hex = Some("4000000000000000".to_string());
        assert!(!recommendation_is_admissible(&metrics, &wrong_median));
        let mut failed_seed = metrics;
        failed_seed[3].all_preregistered_thresholds_passed = false;
        assert!(!recommendation_is_admissible(&failed_seed, &recommendation));
    }

    #[test]
    fn target_ids_are_path_safe_and_bounded() {
        assert!(valid_target_id("excess_return_20d"));
        assert!(!valid_target_id("../holdout"));
        assert!(!valid_target_id("target/other"));
    }

    #[test]
    fn review_chain_rejects_branches() {
        let mut first = minimal_review("1", None, None);
        seal(&mut first);
        let mut left = minimal_review(
            "2",
            Some(first.review_id.clone()),
            Some(first.review_sha256.clone()),
        );
        seal(&mut left);
        let mut right = minimal_review(
            "3",
            Some(first.review_id.clone()),
            Some(first.review_sha256.clone()),
        );
        seal(&mut right);
        assert!(latest_review_from_records(&[first, left, right]).is_err());
    }

    fn minimal_review(
        marker: &str,
        previous_review_id: Option<String>,
        previous_review_sha256: Option<String>,
    ) -> HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview {
        HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id,
            previous_review_sha256,
            attempt_id: "1".repeat(32),
            target_id: "excess_return_20d".to_string(),
            output_validation_id: "2".repeat(32),
            output_validation_sha256: marker.repeat(64),
            claim_sha256: "3".repeat(64),
            result_sha256: "4".repeat(64),
            output_sha256: "5".repeat(64),
            authorization_review_sha256: "6".repeat(64),
            isolated_runner_spec_sha256: "7".repeat(64),
            implementation_sha256: "8".repeat(64),
            implementation_review_sha256: "9".repeat(64),
            candidate_set_sha256: "a".repeat(64),
            upstream_validation_sha256: "b".repeat(64),
            training_store_dataset_sha256: "c".repeat(64),
            validation_projection_sha256: "d".repeat(64),
            target_bundle_sha256: "e".repeat(64),
            recommendation_sha256: "f".repeat(64),
            target_metric_count: 9,
            target_algorithm_count: 3,
            frozen_seed_count: 3,
            recommendation_status: RECOMMENDABLE_STATUS.to_string(),
            recommended_algorithm: Some("\"ridge_multi_target_regression\"".to_string()),
            three_seed_median_mae_f64_bits_hex: Some("3ff0000000000000".to_string()),
            all_three_seeds_passed: true,
            submitted_at: Utc::now(),
            reviewer_id: format!("reviewer-{marker}"),
            excluded_prior_actor_ids: vec!["executor".to_string(), "validator".to_string()],
            verdict: HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionVerdict::ChangesRequested,
            rationale: "需要补充逐目标证据".to_string(),
            known_limitations: "sealed holdout 未读取".to_string(),
            reviewer_independent_from_stage_64_and_complete_prior_chain: true,
            exact_current_stage_51_through_stage_64_binding_confirmed: true,
            exact_target_only_nine_metrics_three_algorithms_three_seeds_confirmed: true,
            target_evidence_status_and_thresholds_confirmed: true,
            recommended_algorithm_and_three_seed_median_confirmed: true,
            no_cross_target_composite_or_masking_confirmed: true,
            sealed_holdout_remains_unread_confirmed: true,
            next_gate_is_protocol_review_not_holdout_execution_confirmed: true,
            no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
            per_target_candidate_admitted: false,
            future_sealed_holdout_evaluation_protocol_review_eligible: false,
            official_candidate_selection_authorized: false,
            sealed_holdout_access_authorized: false,
            model_artifact_store_written: false,
            metric_store_written: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        }
    }

    fn seal(review: &mut HistoricalOutcomeValidationEvaluationPerTargetCandidateAdmissionReview) {
        review.review_sha256 = review_fingerprint(review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
    }

    #[test]
    fn rejection_does_not_open_any_downstream_authority() {
        let mut review = minimal_review("a", None, None);
        seal(&mut review);
        assert!(!review.per_target_candidate_admitted);
        assert!(!review.sealed_holdout_access_authorized);
        assert!(!review.trading_authorized);
    }
}
