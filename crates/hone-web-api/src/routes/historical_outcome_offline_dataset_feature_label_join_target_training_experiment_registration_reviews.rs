//! Independent review of one exact Stage 51 training-experiment registration.
//!
//! Approval is deliberately narrow: it only makes the immutable registration
//! eligible for a future, separately governed training-implementation
//! registration. This module never creates a runner, authorizes or starts
//! training, defines a reward, creates a shadow portfolio, generates an order,
//! accesses a broker or trades.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::{
    HistoricalOutcomeTrainingExperimentAlgorithm,
    HistoricalOutcomeTrainingExperimentRegistrationStatus,
    HistoricalOutcomeTrainingExperimentSuiteSpecification,
    RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview,
    registered_training_experiment_suites_for_independent_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-training-experiment-registration-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-training-experiment-registration-review-v1";
const POLICY_VERSION: &str =
    "hone-training-experiment-registration-independent-review-v1-no-training";
const CLAIM_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-experiment-registration-claim-v1";
const SPEC_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-experiment-suite-spec-v1";
const REGISTRATION_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-experiment-registration-v1";
const RESULT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-training-experiment-registration-result-v1";
const REGISTRATION_POLICY_VERSION: &str =
    "hone-training-experiment-registration-claim-first-create-once-v1-no-run";
const SUITE_VERSION: &str = "hone-nine-target-comparison-suite-v1";
const TARGET_CONTRACT_VERSION: &str = "hone-nine-raw-continuous-outcomes-v1";
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 2_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeTrainingExperimentRegistrationReviewVerdict {
    ApprovedForFutureTrainingImplementationRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_registration_id: String,
    expected_registration_sha256: String,
    expected_claim_sha256: String,
    expected_result_id: String,
    expected_result_sha256: String,
    expected_admission_review_id: String,
    expected_admission_review_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_rows_sha256: String,
    expected_excluded_rows_sha256: String,
    expected_target_commitments_sha256: String,
    expected_suite_specification_sha256: String,
    verdict: HistoricalOutcomeTrainingExperimentRegistrationReviewVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_registration_and_complete_chain_confirmed: bool,
    immutable_claim_registration_result_and_suite_hashes_confirmed: bool,
    claim_first_create_once_success_and_registered_not_run_confirmed: bool,
    registrar_and_reviewer_independence_confirmed: bool,
    fixed_three_arm_three_seed_suite_confirmed: bool,
    exact_65_feature_nine_raw_continuous_target_contract_confirmed: bool,
    train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: bool,
    per_target_per_seed_metrics_without_composite_masking_confirmed: bool,
    fixed_resource_ceilings_and_deterministic_replay_confirmed: bool,
    no_scalar_reward_action_position_or_ranking_semantics_confirmed: bool,
    implementation_registration_runner_and_run_authorization_remain_separate_confirmed: bool,
    no_training_run_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub copy_output_validation_id: String,
    pub copy_output_validation_sha256: String,
    pub copy_id: String,
    pub training_store_dataset_sha256: String,
    pub source_official_joined_dataset_sha256: String,
    pub source_dataset_id: String,
    pub source_dataset_content_sha256: String,
    pub source_dataset_manifest_sha256: String,
    pub source_candidate_set_sha256: String,
    pub rows_sha256: String,
    pub excluded_rows_sha256: String,
    pub target_commitments_sha256: String,
    pub dataset_entry_count: usize,
    pub active_row_count: usize,
    pub excluded_row_count: usize,
    pub feature_catalog_count: usize,
    pub target_count: usize,
    pub train_target_vector_count: usize,
    pub validation_target_withheld_count: usize,
    pub sealed_holdout_target_withheld_count: usize,
    pub suite_version: String,
    pub suite_specification_sha256: String,
    pub arm_count: usize,
    pub random_seeds: Vec<u64>,
    pub registered_at: DateTime<Utc>,
    pub reviewed_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: HistoricalOutcomeTrainingExperimentRegistrationReviewVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_registration_and_complete_chain_confirmed: bool,
    pub immutable_claim_registration_result_and_suite_hashes_confirmed: bool,
    pub claim_first_create_once_success_and_registered_not_run_confirmed: bool,
    pub registrar_and_reviewer_independence_confirmed: bool,
    pub fixed_three_arm_three_seed_suite_confirmed: bool,
    pub exact_65_feature_nine_raw_continuous_target_contract_confirmed: bool,
    pub train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: bool,
    pub per_target_per_seed_metrics_without_composite_masking_confirmed: bool,
    pub fixed_resource_ceilings_and_deterministic_replay_confirmed: bool,
    pub no_scalar_reward_action_position_or_ranking_semantics_confirmed: bool,
    pub implementation_registration_runner_and_run_authorization_remain_separate_confirmed: bool,
    pub no_training_run_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub training_experiment_registration_independently_approved: bool,
    pub future_training_implementation_registration_eligible: bool,
    pub training_implementation_registered: bool,
    pub runner_registered: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationReviewItem {
    pub registered_experiment: RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview,
    pub latest_review: Option<HistoricalOutcomeTrainingExperimentRegistrationReview>,
    pub current_binding: bool,
    pub review_eligible: bool,
    pub independently_approved: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeTrainingExperimentRegistrationReviewItem>,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub rejected_or_changes_requested_count: usize,
    pub future_training_implementation_registration_eligible_count: usize,
    pub review_status: String,
    pub training_implementation_registered: bool,
    pub runner_registered: bool,
    pub training_authorized: bool,
    pub training_started: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeTrainingExperimentRegistrationReviewReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub rejected_or_changes_requested_count: usize,
    pub future_training_implementation_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct IndependentlyReviewedHistoricalOutcomeTrainingExperimentRegistration {
    pub registered_experiment: RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview,
    pub review: HistoricalOutcomeTrainingExperimentRegistrationReview,
}

pub(crate) async fn handle_get_historical_outcome_training_experiment_registration_reviews(
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
            warn!(%error, "training experiment registration review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "训练实验登记独立复核注册表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_training_experiment_registration(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_registration(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_training_experiment_registration_review_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeTrainingExperimentRegistrationReviewReadinessSummary {
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            independently_approved_count: registry.independently_approved_count,
            rejected_or_changes_requested_count: registry.rejected_or_changes_requested_count,
            future_training_implementation_registration_eligible_count: registry
                .future_training_implementation_registration_eligible_count,
            review_status: registry.review_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_reviewed_training_experiment_registrations_for_implementation_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyReviewedHistoricalOutcomeTrainingExperimentRegistration>, String> {
    let mut output = Vec::new();
    for registered_experiment in
        registered_training_experiment_suites_for_independent_review(state).await?
    {
        if let Some(review) = latest_review(state, &registered_experiment).await?
            && review.training_experiment_registration_independently_approved
            && review.future_training_implementation_registration_eligible
        {
            output.push(
                IndependentlyReviewedHistoricalOutcomeTrainingExperimentRegistration {
                    registered_experiment,
                    review,
                },
            );
        }
    }
    output.sort_by(|left, right| right.review.reviewed_at.cmp(&left.review.reviewed_at));
    Ok(output)
}

pub(crate) fn validate_historical_outcome_training_experiment_registration_review_for_implementation(
    review: &HistoricalOutcomeTrainingExperimentRegistrationReview,
) -> Result<(), String> {
    let downstream_closed = !review.training_implementation_registered
        && !review.runner_registered
        && !review.training_authorized
        && !review.training_started
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
        || review.verdict
            != HistoricalOutcomeTrainingExperimentRegistrationReviewVerdict::ApprovedForFutureTrainingImplementationRegistration
        || !review.training_experiment_registration_independently_approved
        || !review.future_training_implementation_registration_eligible
        || !review_all_checks(review)
        || review.suite_version != SUITE_VERSION
        || review.arm_count != 3
        || review.random_seeds != [17, 29, 43]
        || review.feature_catalog_count != 65
        || review.target_count != 9
        || review.reviewer_id.trim().is_empty()
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || !review.reviewer_independent_from_registrar_and_complete_prior_chain
        || !downstream_closed
    {
        return Err(
            "training experiment registration review is not eligible for implementation registration"
                .to_string(),
        );
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry, String> {
    let mut items = Vec::new();
    for registered_experiment in
        registered_training_experiment_suites_for_independent_review(state).await?
    {
        independently_validate_registered_experiment(&registered_experiment)?;
        let latest_review = latest_review(state, &registered_experiment).await?;
        let independently_approved = latest_review
            .as_ref()
            .is_some_and(|review| review.training_experiment_registration_independently_approved);
        items.push(HistoricalOutcomeTrainingExperimentRegistrationReviewItem {
            current_binding: true,
            review_eligible: !independently_approved,
            registered_experiment,
            latest_review,
            independently_approved,
        });
    }
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|item| item.independently_approved)
        .count();
    let rejected_or_changes_requested_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                !review.training_experiment_registration_independently_approved
            })
        })
        .count();
    let future_training_implementation_registration_eligible_count = items
        .iter()
        .filter(|item| {
            item.latest_review
                .as_ref()
                .is_some_and(|review| review.future_training_implementation_registration_eligible)
        })
        .count();
    let review_status = if independently_approved_count > 0 {
        "registration_independently_approved_waiting_training_implementation_registration"
    } else if rejected_or_changes_requested_count > 0 {
        "registration_review_rejected_or_changes_requested"
    } else if review_eligible_count > 0 {
        "waiting_independent_training_experiment_registration_review"
    } else {
        "waiting_stage_51_training_experiment_registration"
    };
    Ok(HistoricalOutcomeTrainingExperimentRegistrationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        rejected_or_changes_requested_count,
        future_training_implementation_registration_eligible_count,
        review_status: review_status.to_string(),
        training_implementation_registered: false,
        runner_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 52 由 Stage 51 登记人、Stage 50 复核者与完整上游之外的新管理员，重新计算 claim、实验规范、registration、result 及上游绑定。批准只开放未来训练实现登记；本阶段不创建 runner、不授权或启动训练，也不开放奖励、影子、订单、券商或交易。".to_string(),
    })
}

async fn review_registration(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest,
) -> Result<HistoricalOutcomeTrainingExperimentRegistrationReview, String> {
    if !valid_id(attempt_id) {
        return Err("训练实验登记尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let registered_experiment = registered_training_experiment_suites_for_independent_review(state)
        .await?
        .into_iter()
        .find(|value| exact_request_binding(value, attempt_id, &request))
        .ok_or_else(|| "当前没有这条精确绑定且完成的 Stage 51 训练实验登记".to_string())?;
    independently_validate_registered_experiment(&registered_experiment)?;
    let latest = latest_review(state, &registered_experiment).await?;
    if latest
        .as_ref()
        .is_some_and(|review| review.training_experiment_registration_independently_approved)
    {
        return Err("该登记已独立批准，复核链永久冻结；下一步只能进入训练实现登记".to_string());
    }
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("训练实验登记复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限与偏差",
    )?;
    let excluded_prior_actor_ids = excluded_actor_ids(&registered_experiment, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "复核人不能是 Stage 51 登记人、Stage 50 复核者、完整上游或此前复核角色".to_string(),
        );
    }
    let claim = &registered_experiment.attempt.claim;
    let registration = registered_experiment
        .attempt
        .registration
        .as_ref()
        .ok_or_else(|| "训练实验登记记录缺失".to_string())?;
    let result = registered_experiment
        .attempt
        .result
        .as_ref()
        .ok_or_else(|| "训练实验登记结果缺失".to_string())?;
    let mut review = HistoricalOutcomeTrainingExperimentRegistrationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: claim.attempt_id.clone(),
        registration_id: registration.registration_id.clone(),
        registration_sha256: registration.registration_sha256.clone(),
        claim_sha256: claim.claim_sha256.clone(),
        result_id: result.result_id.clone(),
        result_sha256: result.result_sha256.clone(),
        admission_review_id: claim.admission_review_id.clone(),
        admission_review_sha256: claim.admission_review_sha256.clone(),
        copy_output_validation_id: claim.copy_output_validation_id.clone(),
        copy_output_validation_sha256: claim.copy_output_validation_sha256.clone(),
        copy_id: claim.copy_id.clone(),
        training_store_dataset_sha256: claim.training_store_dataset_sha256.clone(),
        source_official_joined_dataset_sha256: claim.source_official_joined_dataset_sha256.clone(),
        source_dataset_id: claim.source_dataset_id.clone(),
        source_dataset_content_sha256: claim.source_dataset_content_sha256.clone(),
        source_dataset_manifest_sha256: claim.source_dataset_manifest_sha256.clone(),
        source_candidate_set_sha256: claim.source_candidate_set_sha256.clone(),
        rows_sha256: registration.rows_sha256.clone(),
        excluded_rows_sha256: registration.excluded_rows_sha256.clone(),
        target_commitments_sha256: registration.target_commitments_sha256.clone(),
        dataset_entry_count: registration.dataset_entry_count,
        active_row_count: registration.active_row_count,
        excluded_row_count: registration.excluded_row_count,
        feature_catalog_count: registration.suite_specification.feature_catalog_count,
        target_count: registration.suite_specification.target_count,
        train_target_vector_count: claim.expected_train_target_vector_count,
        validation_target_withheld_count: claim.expected_validation_target_withheld_count,
        sealed_holdout_target_withheld_count: claim.expected_sealed_holdout_target_withheld_count,
        suite_version: registration.suite_specification.suite_version.clone(),
        suite_specification_sha256: registration
            .suite_specification
            .specification_sha256
            .clone(),
        arm_count: registration.suite_specification.arms.len(),
        random_seeds: vec![17, 29, 43],
        registered_at: registration.registered_at,
        reviewed_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale,
        known_limitations,
        reviewer_independent_from_registrar_and_complete_prior_chain: independent,
        exact_current_stage_51_registration_and_complete_chain_confirmed: request
            .exact_current_stage_51_registration_and_complete_chain_confirmed,
        immutable_claim_registration_result_and_suite_hashes_confirmed: request
            .immutable_claim_registration_result_and_suite_hashes_confirmed,
        claim_first_create_once_success_and_registered_not_run_confirmed: request
            .claim_first_create_once_success_and_registered_not_run_confirmed,
        registrar_and_reviewer_independence_confirmed: request
            .registrar_and_reviewer_independence_confirmed,
        fixed_three_arm_three_seed_suite_confirmed: request
            .fixed_three_arm_three_seed_suite_confirmed,
        exact_65_feature_nine_raw_continuous_target_contract_confirmed: request
            .exact_65_feature_nine_raw_continuous_target_contract_confirmed,
        train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: request
            .train_fit_validation_selection_and_sealed_holdout_isolation_confirmed,
        per_target_per_seed_metrics_without_composite_masking_confirmed: request
            .per_target_per_seed_metrics_without_composite_masking_confirmed,
        fixed_resource_ceilings_and_deterministic_replay_confirmed: request
            .fixed_resource_ceilings_and_deterministic_replay_confirmed,
        no_scalar_reward_action_position_or_ranking_semantics_confirmed: request
            .no_scalar_reward_action_position_or_ranking_semantics_confirmed,
        implementation_registration_runner_and_run_authorization_remain_separate_confirmed: request
            .implementation_registration_runner_and_run_authorization_remain_separate_confirmed,
        no_training_run_reward_shadow_order_broker_or_trading_confirmed: request
            .no_training_run_reward_shadow_order_broker_or_trading_confirmed,
        training_experiment_registration_independently_approved: false,
        future_training_implementation_registration_eligible: false,
        training_implementation_registered: false,
        runner_registered: false,
        training_authorized: false,
        training_started: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == HistoricalOutcomeTrainingExperimentRegistrationReviewVerdict::ApprovedForFutureTrainingImplementationRegistration;
    review.training_experiment_registration_independently_approved =
        approved && review_all_checks(&review) && independent;
    review.future_training_implementation_registration_eligible =
        review.training_experiment_registration_independently_approved;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &registered_experiment)?;
    write_immutable_json(
        &review_directory(state, attempt_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn exact_request_binding(
    registered: &RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview,
    attempt_id: &str,
    request: &ReviewHistoricalOutcomeTrainingExperimentRegistrationRequest,
) -> bool {
    let claim = &registered.attempt.claim;
    let Some(registration) = registered.attempt.registration.as_ref() else {
        return false;
    };
    let Some(result) = registered.attempt.result.as_ref() else {
        return false;
    };
    claim.attempt_id == attempt_id
        && registration.registration_id == request.expected_registration_id
        && registration.registration_sha256 == request.expected_registration_sha256
        && claim.claim_sha256 == request.expected_claim_sha256
        && result.result_id == request.expected_result_id
        && result.result_sha256 == request.expected_result_sha256
        && claim.admission_review_id == request.expected_admission_review_id
        && claim.admission_review_sha256 == request.expected_admission_review_sha256
        && claim.training_store_dataset_sha256 == request.expected_training_store_dataset_sha256
        && registration.rows_sha256 == request.expected_rows_sha256
        && registration.excluded_rows_sha256 == request.expected_excluded_rows_sha256
        && registration.target_commitments_sha256 == request.expected_target_commitments_sha256
        && registration.suite_specification.specification_sha256
            == request.expected_suite_specification_sha256
}

fn review_all_checks(review: &HistoricalOutcomeTrainingExperimentRegistrationReview) -> bool {
    review.exact_current_stage_51_registration_and_complete_chain_confirmed
        && review.immutable_claim_registration_result_and_suite_hashes_confirmed
        && review.claim_first_create_once_success_and_registered_not_run_confirmed
        && review.registrar_and_reviewer_independence_confirmed
        && review.fixed_three_arm_three_seed_suite_confirmed
        && review.exact_65_feature_nine_raw_continuous_target_contract_confirmed
        && review.train_fit_validation_selection_and_sealed_holdout_isolation_confirmed
        && review.per_target_per_seed_metrics_without_composite_masking_confirmed
        && review.fixed_resource_ceilings_and_deterministic_replay_confirmed
        && review.no_scalar_reward_action_position_or_ranking_semantics_confirmed
        && review.implementation_registration_runner_and_run_authorization_remain_separate_confirmed
        && review.no_training_run_reward_shadow_order_broker_or_trading_confirmed
}

fn independently_validate_registered_experiment(
    registered: &RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview,
) -> Result<(), String> {
    let claim = &registered.attempt.claim;
    let registration = registered
        .attempt
        .registration
        .as_ref()
        .ok_or_else(|| "training experiment registration record is missing".to_string())?;
    let result = registered
        .attempt
        .result
        .as_ref()
        .ok_or_else(|| "training experiment registration result is missing".to_string())?;
    independently_validate_specification(&registration.suite_specification)?;
    let admission = &registered.admitted_dataset.admission_review;
    let claim_closed = claim.claim_consumed
        && claim.exact_registration_directory_write_allowed
        && !claim.generic_training_store_read_allowed
        && !claim.generic_training_store_write_allowed
        && !claim.training_run_allowed
        && !claim.reward_allowed
        && !claim.shadow_portfolio_allowed
        && !claim.order_generation_allowed
        && !claim.broker_access_allowed
        && !claim.trading_allowed;
    let registration_closed = !registration.independently_reviewed_after_registration
        && registration.future_independent_registration_review_eligible
        && !registration.runner_registered
        && !registration.training_authorized
        && !registration.training_started
        && !registration.reward_authorized
        && !registration.shadow_portfolio_authorized
        && !registration.order_generation_authorized
        && !registration.broker_access_authorized
        && !registration.trading_authorized;
    let result_closed = !result.independent_registration_review_completed
        && result.future_independent_registration_review_eligible
        && !result.runner_registered
        && !result.training_authorized
        && !result.training_started
        && !result.reward_authorized
        && !result.shadow_portfolio_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    let exact_checks = claim.exact_current_stage_50_admission_and_complete_chain_confirmed
        && claim.registrar_independent_from_complete_prior_chain
        && claim.claim_first_create_once_and_failure_consumes_confirmed
        && claim.fixed_three_arm_three_seed_suite_confirmed
        && claim.train_fit_validation_selection_and_sealed_holdout_isolation_confirmed
        && claim.exact_65_feature_nine_raw_target_contract_confirmed
        && claim.no_scalar_reward_action_position_or_ranking_semantics_confirmed
        && claim.independent_registration_review_required_before_training_authorization_confirmed
        && claim.no_training_run_reward_shadow_order_broker_or_trading_confirmed;
    if claim.schema_version != CLAIM_SCHEMA_VERSION
        || claim.policy_version != REGISTRATION_POLICY_VERSION
        || !valid_id(&claim.registration_id)
        || !valid_sha256(&claim.claim_sha256)
        || claim.registration_id != claim.claim_sha256[..32]
        || claim.claim_sha256 != fingerprint_without(claim, &["registration_id", "claim_sha256"])?
        || claim.attempt_id != admission.attempt_id
        || claim.admission_review_id != admission.review_id
        || claim.admission_review_sha256 != admission.review_sha256
        || claim.copy_output_validation_id != admission.copy_output_validation_id
        || claim.copy_output_validation_sha256 != admission.copy_output_validation_sha256
        || claim.copy_id != admission.copy_id
        || claim.training_store_dataset_sha256 != admission.training_store_dataset_sha256
        || claim.source_official_joined_dataset_sha256 != admission.official_joined_dataset_sha256
        || claim.source_dataset_id != admission.dataset_id
        || claim.source_dataset_content_sha256 != admission.dataset_content_sha256
        || claim.source_dataset_manifest_sha256 != admission.dataset_manifest_sha256
        || claim.source_candidate_set_sha256 != admission.candidate_set_sha256
        || claim.expected_rows_sha256 != admission.recomputed_rows_sha256
        || claim.expected_excluded_rows_sha256 != admission.recomputed_excluded_rows_sha256
        || claim.expected_target_commitments_sha256
            != admission.recomputed_target_commitments_sha256
        || claim.expected_dataset_entry_count != admission.dataset_entry_count
        || claim.expected_active_row_count != admission.active_row_count
        || claim.expected_excluded_row_count != admission.excluded_purge_or_embargo_row_count
        || claim.expected_feature_catalog_count != 65
        || claim.expected_target_count != 9
        || claim.expected_train_target_vector_count != admission.train_target_vector_count
        || claim.expected_validation_target_withheld_count
            != admission.validation_target_withheld_count
        || claim.expected_sealed_holdout_target_withheld_count
            != admission.sealed_holdout_target_withheld_count
        || claim.suite_specification_sha256 != registration.suite_specification.specification_sha256
        || claim.registered_by.trim().is_empty()
        || claim.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&claim.excluded_prior_actor_ids)
        || claim
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &claim.registered_by)
        || !admission.training_registration_candidate_admitted
        || !admission.future_create_once_training_registration_eligible
        || !exact_checks
        || !claim_closed
    {
        return Err("independent claim or admitted-upstream validation failed".to_string());
    }
    if registration.schema_version != REGISTRATION_SCHEMA_VERSION
        || registration.policy_version != REGISTRATION_POLICY_VERSION
        || registration.registration_id != claim.registration_id
        || registration.claim_sha256 != claim.claim_sha256
        || !valid_sha256(&registration.registration_sha256)
        || registration.registration_sha256
            != fingerprint_without(registration, &["registration_sha256"])?
        || registration.attempt_id != claim.attempt_id
        || registration.admission_review_id != claim.admission_review_id
        || registration.admission_review_sha256 != claim.admission_review_sha256
        || registration.copy_output_validation_id != claim.copy_output_validation_id
        || registration.copy_output_validation_sha256 != claim.copy_output_validation_sha256
        || registration.copy_id != claim.copy_id
        || registration.training_store_dataset_sha256 != claim.training_store_dataset_sha256
        || registration.rows_sha256 != claim.expected_rows_sha256
        || registration.excluded_rows_sha256 != claim.expected_excluded_rows_sha256
        || registration.target_commitments_sha256 != claim.expected_target_commitments_sha256
        || registration.dataset_entry_count != claim.expected_dataset_entry_count
        || registration.active_row_count != claim.expected_active_row_count
        || registration.excluded_row_count != claim.expected_excluded_row_count
        || registration.active_row_count + registration.excluded_row_count
            != registration.dataset_entry_count
        || registration.experiment_name != claim.experiment_name
        || registration.research_hypothesis != claim.research_hypothesis
        || registration.known_limitations != claim.known_limitations
        || registration.registered_at != claim.claimed_at
        || registration.registered_by != claim.registered_by
        || registration.excluded_prior_actor_ids != claim.excluded_prior_actor_ids
        || registration.status != "registered_not_run"
        || !registration.training_experiment_registered
        || !registration_closed
    {
        return Err("independent registration validation failed".to_string());
    }
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.policy_version != REGISTRATION_POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != fingerprint_without(result, &["result_id", "result_sha256"])?
        || result.registration_id != claim.registration_id
        || result.claim_sha256 != claim.claim_sha256
        || result.status
            != HistoricalOutcomeTrainingExperimentRegistrationStatus::CompletedPendingIndependentReview
        || result.error.is_some()
        || result.registration_sha256.as_deref()
            != Some(registration.registration_sha256.as_str())
        || !result.training_experiment_registered
        || !result_closed
    {
        return Err("independent registration result validation failed".to_string());
    }
    Ok(())
}

fn independently_validate_specification(
    specification: &HistoricalOutcomeTrainingExperimentSuiteSpecification,
) -> Result<(), String> {
    let expected_targets = [20_u16, 60, 250]
        .into_iter()
        .flat_map(|horizon| {
            ["asset_return", "excess_return", "asset_max_drawdown"]
                .into_iter()
                .map(move |metric| format!("{metric}_{horizon}_continuous"))
        })
        .collect::<Vec<_>>();
    let expected_metrics = [
        "mae_per_target_per_seed",
        "spearman_per_target_per_seed",
        "directional_accuracy_per_target_per_seed",
        "calibration_slope_per_target_per_seed",
    ];
    let expected_algorithms = [
        HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline,
        HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
        HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
    ];
    let exact_arms =
        specification.arms.len() == 3
            && specification.arms.iter().zip(expected_algorithms).all(
                |(arm, expected_algorithm)| {
                    arm.algorithm == expected_algorithm
                        && arm.random_seeds == [17, 29, 43]
                        && arm.deterministic_replay_required
                },
            )
            && specification.arms[0].role == "non_trained_zero_prediction_baseline"
            && specification.arms[0].max_epochs_or_boosting_rounds == 0
            && specification.arms[0].learning_rate_micros == 0
            && specification.arms[0].l2_regularization_micros == 0
            && specification.arms[0].maximum_tree_depth == 0
            && specification.arms[1].role == "interpretable_linear_baseline"
            && specification.arms[1].max_epochs_or_boosting_rounds == 1
            && specification.arms[1].learning_rate_micros == 0
            && specification.arms[1].l2_regularization_micros == 10_000
            && specification.arms[1].maximum_tree_depth == 0
            && specification.arms[2].role == "nonlinear_tabular_candidate"
            && specification.arms[2].max_epochs_or_boosting_rounds == 400
            && specification.arms[2].learning_rate_micros == 50_000
            && specification.arms[2].l2_regularization_micros == 10_000
            && specification.arms[2].maximum_tree_depth == 4;
    let capabilities_closed = !specification.sealed_holdout_access_allowed
        && !specification.sealed_holdout_labels_visible_to_training_worker
        && !specification.scalar_reward_defined
        && !specification.action_position_or_ranking_semantics_defined
        && !specification.ambient_environment_available
        && !specification.network_available
        && !specification.external_tools_available
        && !specification.arbitrary_code_allowed
        && !specification.production_state_write_available;
    if specification.schema_version != SPEC_SCHEMA_VERSION
        || specification.suite_version != SUITE_VERSION
        || !valid_sha256(&specification.specification_sha256)
        || specification.specification_sha256
            != fingerprint_without(specification, &["specification_sha256"])?
        || specification.feature_catalog_count != 65
        || specification.target_count != 9
        || specification.target_contract_version != TARGET_CONTRACT_VERSION
        || specification.target_vector_order != expected_targets
        || specification.primary_supervised_target_id != "excess_return_250_continuous"
        || specification.risk_target_id != "asset_max_drawdown_250_continuous"
        || !exact_arms
        || specification.fit_split != "train"
        || specification.model_selection_split != "validation"
        || specification.sealed_holdout_split != "sealed_holdout"
        || specification.feature_preprocessing_contract
            != "fit scaling statistics on train only; preserve explicit missingness; no imputation, future backfill or cross-split fitting"
        || specification.objective_contract
            != "predict all nine raw continuous outcomes without collapsing them into a scalar reward, action, position or rank label"
        || specification.model_selection_contract
            != "report every target and every seed separately; validation may select candidates only after model artifact freeze; no composite score may hide a failing target"
        || specification.reported_metric_ids != expected_metrics.map(ToString::to_string)
        || specification.resource_ceilings.maximum_wall_clock_seconds != 3_600
        || specification.resource_ceilings.maximum_memory_mib != 8_192
        || specification.resource_ceilings.maximum_cpu_millicores != 4_000
        || specification.resource_ceilings.maximum_process_count != 4
        || specification.resource_ceilings.maximum_output_bytes != 256 * 1024 * 1024
        || !capabilities_closed
    {
        return Err(
            "independent suite validation detected drift or over-authorization".to_string(),
        );
    }
    Ok(())
}

fn excluded_actor_ids(
    registered: &RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview,
    latest: &Option<HistoricalOutcomeTrainingExperimentRegistrationReview>,
) -> Vec<String> {
    let mut actors = registered.attempt.claim.excluded_prior_actor_ids.clone();
    actors.push(registered.attempt.claim.registered_by.clone());
    actors.extend(
        registered
            .admitted_dataset
            .admission_review
            .excluded_prior_actor_ids
            .clone(),
    );
    actors.push(
        registered
            .admitted_dataset
            .admission_review
            .reviewer_id
            .clone(),
    );
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
    review: &HistoricalOutcomeTrainingExperimentRegistrationReview,
    registered: &RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview,
) -> Result<(), String> {
    independently_validate_registered_experiment(registered)?;
    let claim = &registered.attempt.claim;
    let registration = registered.attempt.registration.as_ref().unwrap();
    let result = registered.attempt.result.as_ref().unwrap();
    let approved = review.verdict
        == HistoricalOutcomeTrainingExperimentRegistrationReviewVerdict::ApprovedForFutureTrainingImplementationRegistration;
    let expected_independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_approved = approved && review_all_checks(review) && expected_independent;
    let downstream_closed = !review.training_implementation_registered
        && !review.runner_registered
        && !review.training_authorized
        && !review.training_started
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
        || review.previous_review_id.is_some() != review.previous_review_sha256.is_some()
        || review.attempt_id != claim.attempt_id
        || review.registration_id != registration.registration_id
        || review.registration_sha256 != registration.registration_sha256
        || review.claim_sha256 != claim.claim_sha256
        || review.result_id != result.result_id
        || review.result_sha256 != result.result_sha256
        || review.admission_review_id != claim.admission_review_id
        || review.admission_review_sha256 != claim.admission_review_sha256
        || review.copy_output_validation_id != claim.copy_output_validation_id
        || review.copy_output_validation_sha256 != claim.copy_output_validation_sha256
        || review.copy_id != claim.copy_id
        || review.training_store_dataset_sha256 != claim.training_store_dataset_sha256
        || review.source_official_joined_dataset_sha256
            != claim.source_official_joined_dataset_sha256
        || review.source_dataset_id != claim.source_dataset_id
        || review.source_dataset_content_sha256 != claim.source_dataset_content_sha256
        || review.source_dataset_manifest_sha256 != claim.source_dataset_manifest_sha256
        || review.source_candidate_set_sha256 != claim.source_candidate_set_sha256
        || review.rows_sha256 != registration.rows_sha256
        || review.excluded_rows_sha256 != registration.excluded_rows_sha256
        || review.target_commitments_sha256 != registration.target_commitments_sha256
        || review.dataset_entry_count != registration.dataset_entry_count
        || review.active_row_count != registration.active_row_count
        || review.excluded_row_count != registration.excluded_row_count
        || review.feature_catalog_count != 65
        || review.target_count != 9
        || review.train_target_vector_count != claim.expected_train_target_vector_count
        || review.validation_target_withheld_count
            != claim.expected_validation_target_withheld_count
        || review.sealed_holdout_target_withheld_count
            != claim.expected_sealed_holdout_target_withheld_count
        || review.suite_version != SUITE_VERSION
        || review.suite_specification_sha256
            != registration.suite_specification.specification_sha256
        || review.arm_count != 3
        || review.random_seeds != [17, 29, 43]
        || review.registered_at != registration.registered_at
        || review.reviewer_id.trim().is_empty()
        || review.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || review.known_limitations.trim().is_empty()
        || review.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || review.reviewer_independent_from_registrar_and_complete_prior_chain
            != expected_independent
        || (approved && !review_all_checks(review))
        || review.training_experiment_registration_independently_approved != expected_approved
        || review.future_training_implementation_registration_eligible != expected_approved
        || !downstream_closed
    {
        return Err(
            "training experiment registration review is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn review_fingerprint(
    review: &HistoricalOutcomeTrainingExperimentRegistrationReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    registered: &RegisteredHistoricalOutcomeTrainingExperimentForIndependentReview,
) -> Result<Option<HistoricalOutcomeTrainingExperimentRegistrationReview>, String> {
    let directory = review_directory(state, &registered.attempt.claim.attempt_id);
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
        let review: HistoricalOutcomeTrainingExperimentRegistrationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, registered)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("training experiment registration review filename mismatch".to_string());
        }
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err(
                "training experiment registration review contains duplicate records".to_string(),
            );
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeTrainingExperimentRegistrationReview],
) -> Result<Option<HistoricalOutcomeTrainingExperimentRegistrationReview>, String> {
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
        return Err("training experiment registration review chain has invalid tips".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("training experiment registration review chain has a cycle".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id.get(id.as_str()).ok_or_else(|| {
                    "training experiment registration review chain is disconnected".to_string()
                })?;
                if previous.review_sha256 != *sha {
                    return Err(
                        "training experiment registration previous hash mismatch".to_string()
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err("training experiment registration previous link is invalid".to_string());
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err("training experiment registration review chain is disconnected".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "training registration review fingerprint payload is invalid".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    let bytes = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
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

fn review_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-training-experiment-registration-reviews")
        .join(attempt_id)
}

struct ReviewLock {
    path: PathBuf,
}

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-training-experiment-registration-review-{attempt_id}.lock"
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
        .map_err(|_| "已有同一训练实验登记复核正在进行".to_string())?;
    Ok(ReviewLock { path })
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_specification() -> HistoricalOutcomeTrainingExperimentSuiteSpecification {
        use super::super::historical_outcome_offline_dataset_feature_label_join_target_training_experiment_registrations::{
            HistoricalOutcomeTrainingExperimentArm,
            HistoricalOutcomeTrainingExperimentResourceCeilings,
        };
        let seeds = vec![17, 29, 43];
        let mut specification = HistoricalOutcomeTrainingExperimentSuiteSpecification {
            schema_version: SPEC_SCHEMA_VERSION.into(),
            suite_version: SUITE_VERSION.into(),
            specification_sha256: String::new(),
            feature_catalog_count: 65,
            target_count: 9,
            target_contract_version: TARGET_CONTRACT_VERSION.into(),
            target_vector_order: [20_u16, 60, 250]
                .into_iter()
                .flat_map(|horizon| {
                    ["asset_return", "excess_return", "asset_max_drawdown"]
                        .into_iter()
                        .map(move |metric| format!("{metric}_{horizon}_continuous"))
                })
                .collect(),
            primary_supervised_target_id: "excess_return_250_continuous".into(),
            risk_target_id: "asset_max_drawdown_250_continuous".into(),
            arms: vec![
                HistoricalOutcomeTrainingExperimentArm {
                    algorithm: HistoricalOutcomeTrainingExperimentAlgorithm::FrozenZeroPredictionBaseline,
                    role: "non_trained_zero_prediction_baseline".into(), random_seeds: seeds.clone(), max_epochs_or_boosting_rounds: 0, learning_rate_micros: 0, l2_regularization_micros: 0, maximum_tree_depth: 0, deterministic_replay_required: true,
                },
                HistoricalOutcomeTrainingExperimentArm {
                    algorithm: HistoricalOutcomeTrainingExperimentAlgorithm::RidgeMultiTargetRegression,
                    role: "interpretable_linear_baseline".into(), random_seeds: seeds.clone(), max_epochs_or_boosting_rounds: 1, learning_rate_micros: 0, l2_regularization_micros: 10_000, maximum_tree_depth: 0, deterministic_replay_required: true,
                },
                HistoricalOutcomeTrainingExperimentArm {
                    algorithm: HistoricalOutcomeTrainingExperimentAlgorithm::GradientBoostedMultiTargetRegression,
                    role: "nonlinear_tabular_candidate".into(), random_seeds: seeds, max_epochs_or_boosting_rounds: 400, learning_rate_micros: 50_000, l2_regularization_micros: 10_000, maximum_tree_depth: 4, deterministic_replay_required: true,
                },
            ],
            fit_split: "train".into(), model_selection_split: "validation".into(), sealed_holdout_split: "sealed_holdout".into(),
            feature_preprocessing_contract: "fit scaling statistics on train only; preserve explicit missingness; no imputation, future backfill or cross-split fitting".into(),
            objective_contract: "predict all nine raw continuous outcomes without collapsing them into a scalar reward, action, position or rank label".into(),
            model_selection_contract: "report every target and every seed separately; validation may select candidates only after model artifact freeze; no composite score may hide a failing target".into(),
            reported_metric_ids: ["mae_per_target_per_seed", "spearman_per_target_per_seed", "directional_accuracy_per_target_per_seed", "calibration_slope_per_target_per_seed"].map(ToString::to_string).to_vec(),
            sealed_holdout_access_allowed: false, sealed_holdout_labels_visible_to_training_worker: false, scalar_reward_defined: false, action_position_or_ranking_semantics_defined: false,
            resource_ceilings: HistoricalOutcomeTrainingExperimentResourceCeilings { maximum_wall_clock_seconds: 3_600, maximum_memory_mib: 8_192, maximum_cpu_millicores: 4_000, maximum_process_count: 4, maximum_output_bytes: 256 * 1024 * 1024 },
            ambient_environment_available: false, network_available: false, external_tools_available: false, arbitrary_code_allowed: false, production_state_write_available: false,
        };
        specification.specification_sha256 =
            fingerprint_without(&specification, &["specification_sha256"]).unwrap();
        specification
    }

    pub(super) fn minimal_review() -> HistoricalOutcomeTrainingExperimentRegistrationReview {
        HistoricalOutcomeTrainingExperimentRegistrationReview {
            schema_version: REVIEW_SCHEMA_VERSION.into(), policy_version: POLICY_VERSION.into(), review_id: String::new(), review_sha256: String::new(), previous_review_id: None, previous_review_sha256: None,
            attempt_id: "1".repeat(32), registration_id: "2".repeat(32), registration_sha256: "3".repeat(64), claim_sha256: "4".repeat(64), result_id: "5".repeat(32), result_sha256: "6".repeat(64), admission_review_id: "7".repeat(32), admission_review_sha256: "8".repeat(64), copy_output_validation_id: "9".repeat(32), copy_output_validation_sha256: "a".repeat(64), copy_id: "b".repeat(32), training_store_dataset_sha256: "c".repeat(64), source_official_joined_dataset_sha256: "d".repeat(64), source_dataset_id: "e".repeat(32), source_dataset_content_sha256: "f".repeat(64), source_dataset_manifest_sha256: "1".repeat(64), source_candidate_set_sha256: "2".repeat(64), rows_sha256: "3".repeat(64), excluded_rows_sha256: "4".repeat(64), target_commitments_sha256: "5".repeat(64),
            dataset_entry_count: 9, active_row_count: 6, excluded_row_count: 3, feature_catalog_count: 65, target_count: 9, train_target_vector_count: 4, validation_target_withheld_count: 2, sealed_holdout_target_withheld_count: 3,
            suite_version: SUITE_VERSION.into(), suite_specification_sha256: "6".repeat(64), arm_count: 3, random_seeds: vec![17, 29, 43], registered_at: Utc::now(), reviewed_at: Utc::now(), reviewer_id: "stage-52-reviewer".into(), excluded_prior_actor_ids: vec!["registrar".into(), "stage-50-reviewer".into()], verdict: HistoricalOutcomeTrainingExperimentRegistrationReviewVerdict::ApprovedForFutureTrainingImplementationRegistration, rationale: "独立重算并逐项复核通过".into(), known_limitations: "只批准未来训练实现登记，不确认模型质量、策略收益或训练授权".into(), reviewer_independent_from_registrar_and_complete_prior_chain: true,
            exact_current_stage_51_registration_and_complete_chain_confirmed: true, immutable_claim_registration_result_and_suite_hashes_confirmed: true, claim_first_create_once_success_and_registered_not_run_confirmed: true, registrar_and_reviewer_independence_confirmed: true, fixed_three_arm_three_seed_suite_confirmed: true, exact_65_feature_nine_raw_continuous_target_contract_confirmed: true, train_fit_validation_selection_and_sealed_holdout_isolation_confirmed: true, per_target_per_seed_metrics_without_composite_masking_confirmed: true, fixed_resource_ceilings_and_deterministic_replay_confirmed: true, no_scalar_reward_action_position_or_ranking_semantics_confirmed: true, implementation_registration_runner_and_run_authorization_remain_separate_confirmed: true, no_training_run_reward_shadow_order_broker_or_trading_confirmed: true,
            training_experiment_registration_independently_approved: true, future_training_implementation_registration_eligible: true, training_implementation_registered: false, runner_registered: false, training_authorized: false, training_started: false, reward_authorized: false, shadow_portfolio_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        }
    }

    #[test]
    fn approval_requires_all_twelve_confirmations() {
        let mut review = minimal_review();
        assert!(review_all_checks(&review));
        review.per_target_per_seed_metrics_without_composite_masking_confirmed = false;
        assert!(!review_all_checks(&review));
    }

    #[test]
    fn independent_suite_review_detects_arm_seed_and_holdout_drift() {
        let specification = expected_specification();
        assert!(independently_validate_specification(&specification).is_ok());
        let mut changed = specification;
        changed.arms[2].random_seeds = vec![17, 29, 44];
        changed.sealed_holdout_access_allowed = true;
        changed.specification_sha256 =
            fingerprint_without(&changed, &["specification_sha256"]).unwrap();
        assert!(independently_validate_specification(&changed).is_err());
    }

    #[test]
    fn reviewer_must_be_independent_from_registrar_and_upstream() {
        let actors = vec!["registrar".to_string(), "stage-50-reviewer".to_string()];
        assert!(reviewer_is_independent("stage-52-reviewer", &actors));
        assert!(!reviewer_is_independent("registrar", &actors));
    }

    #[test]
    fn review_hash_binds_registration_suite_and_result() {
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.suite_specification_sha256 = "9".repeat(64);
        changed.result_sha256 = "a".repeat(64);
        assert_ne!(first, review_fingerprint(&changed).unwrap());
    }

    #[test]
    fn review_hash_binds_implementation_and_training_boundary() {
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.training_implementation_registered = true;
        changed.training_started = true;
        assert_ne!(first, review_fingerprint(&changed).unwrap());
    }

    #[test]
    fn review_chain_rejects_forks() {
        let mut root = minimal_review();
        root.review_sha256 = review_fingerprint(&root).unwrap();
        root.review_id = root.review_sha256[..32].to_string();
        let mut left = root.clone();
        left.previous_review_id = Some(root.review_id.clone());
        left.previous_review_sha256 = Some(root.review_sha256.clone());
        left.reviewed_at += chrono::Duration::seconds(1);
        left.review_sha256 = review_fingerprint(&left).unwrap();
        left.review_id = left.review_sha256[..32].to_string();
        let mut right = left.clone();
        right.rationale = "另一条分叉".into();
        right.review_sha256 = review_fingerprint(&right).unwrap();
        right.review_id = right.review_sha256[..32].to_string();
        assert!(latest_review_from_records(&[root, left, right]).is_err());
    }

    #[test]
    fn review_chain_rejects_disconnected_records() {
        let mut root = minimal_review();
        root.review_sha256 = review_fingerprint(&root).unwrap();
        root.review_id = root.review_sha256[..32].to_string();
        let mut orphan = root.clone();
        orphan.rationale = "孤立记录".into();
        orphan.review_sha256 = review_fingerprint(&orphan).unwrap();
        orphan.review_id = orphan.review_sha256[..32].to_string();
        assert!(latest_review_from_records(&[root, orphan]).is_err());
    }

    #[test]
    fn approval_only_opens_future_implementation_registration() {
        let review = minimal_review();
        assert!(review.training_experiment_registration_independently_approved);
        assert!(review.future_training_implementation_registration_eligible);
        assert!(!review.training_implementation_registered);
        assert!(!review.runner_registered);
        assert!(!review.training_authorized);
        assert!(!review.training_started);
        assert!(!review.reward_authorized);
        assert!(!review.shadow_portfolio_authorized);
        assert!(!review.order_generation_authorized);
        assert!(!review.broker_access_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn required_text_is_trimmed_and_bounded() {
        assert_eq!(bounded_required("  通过  ", 8, "依据").unwrap(), "通过");
        assert!(bounded_required("   ", 8, "依据").is_err());
        assert!(bounded_required("123456789", 8, "依据").is_err());
    }

    #[test]
    fn actor_exclusion_is_sorted_unique_and_hash_bound() {
        assert!(sorted_unique(&["a".into(), "b".into()]));
        assert!(!sorted_unique(&["b".into(), "a".into()]));
        let review = minimal_review();
        let first = review_fingerprint(&review).unwrap();
        let mut changed = review;
        changed.excluded_prior_actor_ids.push("z-upstream".into());
        assert_ne!(first, review_fingerprint(&changed).unwrap());
    }
}

#[cfg(test)]
pub(crate) fn test_approved_training_experiment_registration_review()
-> HistoricalOutcomeTrainingExperimentRegistrationReview {
    let mut review = tests::minimal_review();
    review.review_sha256 = review_fingerprint(&review).expect("test review is serializable");
    review.review_id = review.review_sha256[..32].to_string();
    review
}
