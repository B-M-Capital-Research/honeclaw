//! Independent, append-only review of one exact Stage 74 controlled shadow-experiment
//! design registration.
//!
//! Approval is deliberately narrow: it opens only a future zero-capability shadow
//! implementation-specification registration. It does not select or materialize a model,
//! enable reward, start a shadow ledger, create a position or order, connect a broker or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registrations::{
    ControlledShadowExperimentDesignRegistration,
    ControlledShadowExperimentDesignSpecification,
    RegisteredControlledShadowExperimentDesignForIndependentReview,
    registered_controlled_shadow_experiment_designs_for_independent_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-design-registration-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-design-registration-review-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-experiment-design-independent-review-v1-no-shadow-run";
const REGISTRATION_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-design-registration-v1";
const REGISTRATION_POLICY_VERSION: &str =
    "hone-controlled-shadow-experiment-design-create-once-v1-no-shadow-run";
const SPEC_SCHEMA_VERSION: &str = "hone-controlled-shadow-experiment-design-spec-v1";
const MAX_TEXT_CHARS: usize = 3_600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowExperimentDesignRegistrationReviewVerdict {
    ApprovedForFutureZeroCapabilityShadowImplementationRegistration,
    ChangesRequestedRequiresNewDesignRegistration,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowExperimentDesignRegistrationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_registration_id: String,
    expected_registration_sha256: String,
    expected_adjudication_review_id: String,
    expected_adjudication_review_sha256: String,
    expected_output_validation_id: String,
    expected_output_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_envelope_sha256: String,
    expected_candidate_set_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_design_specification_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    verdict: ControlledShadowExperimentDesignRegistrationReviewVerdict,
    rationale: String,
    risk_assessment: String,
    known_limitations: String,
    falsification_assessment: String,
    future_implementation_constraints: String,
    exact_current_stage_51_through_stage_74_binding_confirmed: bool,
    independent_recomputation_of_registration_and_design_fingerprints_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    experimental_candidate_not_official_model_selection_confirmed: bool,
    point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed: bool,
    benchmark_and_all_counterfactual_semantics_reviewed: bool,
    signal_timing_execution_cost_dividends_and_rebalance_reviewed: bool,
    long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: bool,
    minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed: bool,
    separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: bool,
    stop_rules_falsification_and_no_in_place_restart_reviewed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed: bool,
    no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentDesignRegistrationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub adjudication_review_id: String,
    pub adjudication_review_sha256: String,
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
    pub design_specification_sha256: String,
    pub independently_recomputed_registration_sha256: String,
    pub independently_recomputed_design_specification_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowExperimentDesignRegistrationReviewVerdict,
    pub rationale: String,
    pub risk_assessment: String,
    pub known_limitations: String,
    pub falsification_assessment: String,
    pub future_implementation_constraints: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_74_binding_confirmed: bool,
    pub independent_recomputation_of_registration_and_design_fingerprints_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub experimental_candidate_not_official_model_selection_confirmed: bool,
    pub point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed: bool,
    pub benchmark_and_all_counterfactual_semantics_reviewed: bool,
    pub signal_timing_execution_cost_dividends_and_rebalance_reviewed: bool,
    pub long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: bool,
    pub minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed: bool,
    pub separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: bool,
    pub stop_rules_falsification_and_no_in_place_restart_reviewed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed:
        bool,
    pub no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: bool,
    pub design_registration_independently_approved: bool,
    pub future_zero_capability_shadow_implementation_registration_eligible: bool,
    pub official_model_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_implementation_registered: bool,
    pub shadow_run_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentDesignRegistrationReviewItem {
    pub registered_design: RegisteredControlledShadowExperimentDesignForIndependentReview,
    pub latest_review: Option<ControlledShadowExperimentDesignRegistrationReview>,
    pub review_eligible: bool,
    pub independently_approved: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentDesignRegistrationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowExperimentDesignRegistrationReviewItem>,
    pub registered_design_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_zero_capability_shadow_implementation_registration_eligible_count: usize,
    pub review_status: String,
    pub independent_review_available: bool,
    pub official_model_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_implementation_registered: bool,
    pub shadow_run_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowExperimentDesignRegistrationReviewReadinessSummary {
    pub registered_design_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_zero_capability_shadow_implementation_registration_eligible_count: usize,
    pub review_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct IndependentlyReviewedControlledShadowExperimentDesign {
    pub registered_design: RegisteredControlledShadowExperimentDesignForIndependentReview,
    pub review: ControlledShadowExperimentDesignRegistrationReview,
}

pub(crate) async fn handle_get_controlled_shadow_experiment_design_registration_reviews(
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
            warn!(%error, "controlled shadow experiment design review registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "受控影子实验设计独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_experiment_design_registration(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowExperimentDesignRegistrationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_design(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_experiment_design_registration_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowExperimentDesignRegistrationReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowExperimentDesignRegistrationReviewReadinessSummary {
            registered_design_count: registry.registered_design_count,
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            independently_approved_count: registry.independently_approved_count,
            changes_requested_or_rejected_count: registry.changes_requested_or_rejected_count,
            future_zero_capability_shadow_implementation_registration_eligible_count: registry
                .future_zero_capability_shadow_implementation_registration_eligible_count,
            review_status: registry.review_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_reviewed_controlled_shadow_experiment_designs_for_implementation_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyReviewedControlledShadowExperimentDesign>, String> {
    let mut output = Vec::new();
    for registered_design in
        registered_controlled_shadow_experiment_designs_for_independent_review(state).await?
    {
        if let Some(review) = latest_review(state, &registered_design).await?
            && review.design_registration_independently_approved
            && review.future_zero_capability_shadow_implementation_registration_eligible
        {
            output.push(IndependentlyReviewedControlledShadowExperimentDesign {
                registered_design,
                review,
            });
        }
    }
    output.sort_by(|left, right| right.review.submitted_at.cmp(&left.review.submitted_at));
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowExperimentDesignRegistrationReviewRegistry, String> {
    let mut items = Vec::new();
    for registered_design in
        registered_controlled_shadow_experiment_designs_for_independent_review(state).await?
    {
        let latest_review = latest_review(state, &registered_design).await?;
        let approved = latest_review
            .as_ref()
            .is_some_and(|review| review.design_registration_independently_approved);
        items.push(ControlledShadowExperimentDesignRegistrationReviewItem {
            review_eligible: !approved,
            registered_design,
            latest_review,
            independently_approved: approved,
        });
    }
    items.sort_by(|left, right| {
        left.registered_design
            .registration
            .registration_id
            .cmp(&right.registered_design.registration.registration_id)
    });
    let registered_design_count = items.len();
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|item| item.independently_approved)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review
                .as_ref()
                .is_some_and(|review| !review.design_registration_independently_approved)
        })
        .count();
    let review_status = if independently_approved_count > 0 {
        "shadow_design_independently_approved_waiting_zero_capability_implementation_registration"
    } else if changes_requested_or_rejected_count > 0 {
        "shadow_design_changes_requested_requires_new_registration_or_rejected"
    } else if registered_design_count > 0 {
        "waiting_independent_shadow_design_review"
    } else {
        "waiting_stage_74_shadow_design_registration"
    };
    Ok(
        ControlledShadowExperimentDesignRegistrationReviewRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            items,
            registered_design_count,
            review_eligible_count,
            reviewed_count,
            independently_approved_count,
            changes_requested_or_rejected_count,
            future_zero_capability_shadow_implementation_registration_eligible_count:
                independently_approved_count,
            review_status: review_status.to_string(),
            independent_review_available: true,
            official_model_selection_authorized: false,
            model_artifact_store_written: false,
            metric_store_written: false,
            training_feedback_authorized: false,
            reward_authorized: false,
            shadow_implementation_registered: false,
            shadow_run_authorized: false,
            shadow_ledger_enabled: false,
            shadow_position_written: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "第 75 阶段由责任链外新角色独立重算 Stage 74 登记与设计指纹，并复核点时/退市偏差、反事实、信号时点、成本、分红、组合边界、观察门槛、分项指标、多重检验和停止规则。批准只开放未来零能力影子实现规格登记，不正式选模、不写模型/指标库、不反馈训练或奖励，也不启动影子账本、持仓、订单、券商或交易。".to_string(),
        },
    )
}

async fn review_design(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewControlledShadowExperimentDesignRegistrationRequest,
) -> Result<ControlledShadowExperimentDesignRegistrationReview, String> {
    if !valid_id(attempt_id) {
        return Err("sealed-holdout 尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let mut matching =
        registered_controlled_shadow_experiment_designs_for_independent_review(state)
            .await?
            .into_iter()
            .filter(|registered| registered.registration.attempt_id == attempt_id)
            .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err("当前没有精确一条可独立复核的 Stage 74 设计登记".to_string());
    }
    let registered_design = matching.remove(0);
    validate_expected_binding(&registered_design, &request)?;
    let latest = latest_review(state, &registered_design).await?;
    if latest
        .as_ref()
        .is_some_and(|review| review.design_registration_independently_approved)
    {
        return Err(
            "该设计已独立批准，复核链永久冻结；下一步只能登记零能力影子实现规格".to_string(),
        );
    }
    if latest.as_ref().map(|review| review.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|review| review.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("影子设计复核链已经变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&registered_design.registration, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "复核人不能是 Stage 74 登记人、Stage 73 裁决者或完整上游责任链角色".to_string(),
        );
    }
    let recomputed_design_sha256 =
        independent_design_fingerprint(&registered_design.registration.design_specification)?;
    let recomputed_registration_sha256 =
        independent_registration_fingerprint(&registered_design.registration)?;
    if recomputed_design_sha256
        != registered_design
            .registration
            .design_specification
            .specification_sha256
        || recomputed_registration_sha256 != registered_design.registration.registration_sha256
    {
        return Err("Stage 74 登记或设计指纹独立重算不一致".to_string());
    }
    let mut review = ControlledShadowExperimentDesignRegistrationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: attempt_id.to_string(),
        registration_id: registered_design.registration.registration_id.clone(),
        registration_sha256: registered_design.registration.registration_sha256.clone(),
        adjudication_review_id: registered_design.registration.adjudication_review_id.clone(),
        adjudication_review_sha256: registered_design
            .registration
            .adjudication_review_sha256
            .clone(),
        output_validation_id: registered_design.registration.output_validation_id.clone(),
        output_validation_sha256: registered_design
            .registration
            .output_validation_sha256
            .clone(),
        claim_sha256: registered_design.registration.claim_sha256.clone(),
        result_sha256: registered_design.registration.result_sha256.clone(),
        output_sha256: registered_design.registration.output_sha256.clone(),
        envelope_sha256: registered_design.registration.envelope_sha256.clone(),
        candidate_set_sha256: registered_design.registration.candidate_set_sha256.clone(),
        training_store_dataset_sha256: registered_design
            .registration
            .training_store_dataset_sha256
            .clone(),
        selected_algorithm_three_seed_binding_sha256: registered_design
            .registration
            .selected_algorithm_three_seed_binding_sha256
            .clone(),
        sealed_holdout_split_commitment_sha256: registered_design
            .registration
            .sealed_holdout_split_commitment_sha256
            .clone(),
        sealed_holdout_projection_sha256: registered_design
            .registration
            .sealed_holdout_projection_sha256
            .clone(),
        feature_order_sha256: registered_design.registration.feature_order_sha256.clone(),
        preprocessing_sha256: registered_design.registration.preprocessing_sha256.clone(),
        design_specification_sha256: registered_design
            .registration
            .design_specification
            .specification_sha256
            .clone(),
        independently_recomputed_registration_sha256: recomputed_registration_sha256,
        independently_recomputed_design_specification_sha256: recomputed_design_sha256,
        target_id: registered_design.registration.target_id.clone(),
        frozen_candidate_algorithm_id: registered_design
            .registration
            .frozen_candidate_algorithm_id
            .clone(),
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        risk_assessment: bounded_required(&request.risk_assessment, "风险评估")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        falsification_assessment: bounded_required(
            &request.falsification_assessment,
            "证伪评估",
        )?,
        future_implementation_constraints: bounded_required(
            &request.future_implementation_constraints,
            "未来实现约束",
        )?,
        reviewer_independent_from_registrar_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_74_binding_confirmed: request
            .exact_current_stage_51_through_stage_74_binding_confirmed,
        independent_recomputation_of_registration_and_design_fingerprints_confirmed: request
            .independent_recomputation_of_registration_and_design_fingerprints_confirmed,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_registrar_and_complete_prior_chain_confirmed,
        experimental_candidate_not_official_model_selection_confirmed: request
            .experimental_candidate_not_official_model_selection_confirmed,
        point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed: request
            .point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed,
        benchmark_and_all_counterfactual_semantics_reviewed: request
            .benchmark_and_all_counterfactual_semantics_reviewed,
        signal_timing_execution_cost_dividends_and_rebalance_reviewed: request
            .signal_timing_execution_cost_dividends_and_rebalance_reviewed,
        long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: request
            .long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed,
        minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed: request
            .minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed,
        separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: request
            .separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed,
        stop_rules_falsification_and_no_in_place_restart_reviewed: request
            .stop_rules_falsification_and_no_in_place_restart_reviewed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed:
            request.approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed,
        no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: request
            .no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed,
        design_registration_independently_approved: false,
        future_zero_capability_shadow_implementation_registration_eligible: false,
        official_model_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        shadow_implementation_registered: false,
        shadow_run_authorized: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == ControlledShadowExperimentDesignRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityShadowImplementationRegistration
        && independent
        && review_all_checks(&review);
    review.design_registration_independently_approved = approved;
    review.future_zero_capability_shadow_implementation_registration_eligible = approved;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &registered_design)?;
    write_immutable_json(
        &review_directory(state, attempt_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    registered: &RegisteredControlledShadowExperimentDesignForIndependentReview,
    request: &ReviewControlledShadowExperimentDesignRegistrationRequest,
) -> Result<(), String> {
    let registration = &registered.registration;
    let exact = request.expected_registration_id == registration.registration_id
        && request.expected_registration_sha256 == registration.registration_sha256
        && request.expected_adjudication_review_id == registration.adjudication_review_id
        && request.expected_adjudication_review_sha256 == registration.adjudication_review_sha256
        && request.expected_output_validation_id == registration.output_validation_id
        && request.expected_output_validation_sha256 == registration.output_validation_sha256
        && request.expected_claim_sha256 == registration.claim_sha256
        && request.expected_result_sha256 == registration.result_sha256
        && request.expected_output_sha256 == registration.output_sha256
        && request.expected_envelope_sha256 == registration.envelope_sha256
        && request.expected_candidate_set_sha256 == registration.candidate_set_sha256
        && request.expected_selected_algorithm_three_seed_binding_sha256
            == registration.selected_algorithm_three_seed_binding_sha256
        && request.expected_design_specification_sha256
            == registration.design_specification.specification_sha256
        && request.expected_target_id == registration.target_id
        && request.expected_frozen_candidate_algorithm_id
            == registration.frozen_candidate_algorithm_id;
    exact
        .then_some(())
        .ok_or_else(|| "Stage 51-74 精确绑定已经变化，请刷新后重试".to_string())
}

fn review_all_checks(review: &ControlledShadowExperimentDesignRegistrationReview) -> bool {
    review.exact_current_stage_51_through_stage_74_binding_confirmed
        && review.independent_recomputation_of_registration_and_design_fingerprints_confirmed
        && review.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && review.experimental_candidate_not_official_model_selection_confirmed
        && review.point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed
        && review.benchmark_and_all_counterfactual_semantics_reviewed
        && review.signal_timing_execution_cost_dividends_and_rebalance_reviewed
        && review.long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed
        && review.minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed
        && review.separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed
        && review.stop_rules_falsification_and_no_in_place_restart_reviewed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
        && review
            .approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed
        && review.no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed
}

fn excluded_actor_ids(
    registration: &ControlledShadowExperimentDesignRegistration,
    latest: &Option<ControlledShadowExperimentDesignRegistrationReview>,
) -> Vec<String> {
    let mut actors = registration.excluded_prior_actor_ids.clone();
    actors.push(registration.registered_by.clone());
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
    review: &ControlledShadowExperimentDesignRegistrationReview,
    registered: &RegisteredControlledShadowExperimentDesignForIndependentReview,
) -> Result<(), String> {
    let registration = &registered.registration;
    let approving = review.verdict
        == ControlledShadowExperimentDesignRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityShadowImplementationRegistration;
    let independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_approved = approving && independent && review_all_checks(review);
    let required_actors = excluded_actor_ids(registration, &None);
    let texts_valid = [
        &review.rationale,
        &review.risk_assessment,
        &review.known_limitations,
        &review.falsification_assessment,
        &review.future_implementation_constraints,
    ]
    .into_iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let downstream_closed = !review.official_model_selection_authorized
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.shadow_implementation_registered
        && !review.shadow_run_authorized
        && !review.shadow_ledger_enabled
        && !review.shadow_position_written
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
        || review.attempt_id != registration.attempt_id
        || review.registration_id != registration.registration_id
        || review.registration_sha256 != registration.registration_sha256
        || review.adjudication_review_id != registration.adjudication_review_id
        || review.adjudication_review_sha256 != registration.adjudication_review_sha256
        || review.output_validation_id != registration.output_validation_id
        || review.output_validation_sha256 != registration.output_validation_sha256
        || review.claim_sha256 != registration.claim_sha256
        || review.result_sha256 != registration.result_sha256
        || review.output_sha256 != registration.output_sha256
        || review.envelope_sha256 != registration.envelope_sha256
        || review.candidate_set_sha256 != registration.candidate_set_sha256
        || review.training_store_dataset_sha256 != registration.training_store_dataset_sha256
        || review.selected_algorithm_three_seed_binding_sha256
            != registration.selected_algorithm_three_seed_binding_sha256
        || review.sealed_holdout_split_commitment_sha256
            != registration.sealed_holdout_split_commitment_sha256
        || review.sealed_holdout_projection_sha256 != registration.sealed_holdout_projection_sha256
        || review.feature_order_sha256 != registration.feature_order_sha256
        || review.preprocessing_sha256 != registration.preprocessing_sha256
        || review.design_specification_sha256
            != registration.design_specification.specification_sha256
        || review.independently_recomputed_registration_sha256
            != independent_registration_fingerprint(registration)?
        || review.independently_recomputed_design_specification_sha256
            != independent_design_fingerprint(&registration.design_specification)?
        || review.target_id != registration.target_id
        || review.frozen_candidate_algorithm_id != registration.frozen_candidate_algorithm_id
        || review.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || required_actors
            .iter()
            .any(|actor| !review.excluded_prior_actor_ids.contains(actor))
        || !independent
        || review.reviewer_independent_from_registrar_and_complete_prior_chain != independent
        || !texts_valid
        || (approving && !review_all_checks(review))
        || review.design_registration_independently_approved != expected_approved
        || review.future_zero_capability_shadow_implementation_registration_eligible
            != expected_approved
        || !downstream_closed
    {
        return Err(
            "controlled shadow experiment design review is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn independent_design_fingerprint(
    specification: &ControlledShadowExperimentDesignSpecification,
) -> Result<String, String> {
    if specification.schema_version != SPEC_SCHEMA_VERSION {
        return Err("影子实验设计 schema 无效".to_string());
    }
    fingerprint_without(specification, &["specification_sha256"])
}

fn independent_registration_fingerprint(
    registration: &ControlledShadowExperimentDesignRegistration,
) -> Result<String, String> {
    if registration.schema_version != REGISTRATION_SCHEMA_VERSION
        || registration.policy_version != REGISTRATION_POLICY_VERSION
    {
        return Err("影子实验设计登记 schema 或策略版本无效".to_string());
    }
    fingerprint_without(registration, &["registration_id", "registration_sha256"])
}

fn review_fingerprint(
    review: &ControlledShadowExperimentDesignRegistrationReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    registered: &RegisteredControlledShadowExperimentDesignForIndependentReview,
) -> Result<Option<ControlledShadowExperimentDesignRegistrationReview>, String> {
    let directory = review_directory(state, &registered.registration.attempt_id);
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
        let review: ControlledShadowExperimentDesignRegistrationReview = serde_json::from_slice(
            &tokio::fs::read(entry.path())
                .await
                .map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        validate_review(&review, registered)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err("影子实验设计复核包含文件名不符或重复记录".to_string());
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[ControlledShadowExperimentDesignRegistrationReview],
) -> Result<Option<ControlledShadowExperimentDesignRegistrationReview>, String> {
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
        return Err("影子实验设计复核链存在分叉或多个链尖".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("影子实验设计复核链存在循环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "影子实验设计复核链断裂".to_string())?;
                if previous.review_sha256 != *sha {
                    return Err("影子实验设计复核前序哈希不一致".to_string());
                }
                if !review
                    .excluded_prior_actor_ids
                    .contains(&previous.reviewer_id)
                    || previous
                        .excluded_prior_actor_ids
                        .iter()
                        .any(|actor| !review.excluded_prior_actor_ids.contains(actor))
                {
                    return Err("影子实验设计复核未完整继承此前角色排除集合".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("影子实验设计复核前序链接无效".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("影子实验设计复核链断裂".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "影子设计复核指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
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
        .join("controlled-shadow-experiment-design-registration-reviews")
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
        "controlled-shadow-experiment-design-review-{attempt_id}.lock"
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
        .map_err(|_| "已有同一影子实验设计复核正在进行".to_string())?;
    Ok(ReviewLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_review() -> ControlledShadowExperimentDesignRegistrationReview {
        let mut review = ControlledShadowExperimentDesignRegistrationReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(), review_id: String::new(), review_sha256: String::new(), previous_review_id: None, previous_review_sha256: None,
            attempt_id: "a".repeat(32), registration_id: "b".repeat(32), registration_sha256: "c".repeat(64), adjudication_review_id: "d".repeat(32), adjudication_review_sha256: "e".repeat(64), output_validation_id: "f".repeat(32), output_validation_sha256: "1".repeat(64), claim_sha256: "2".repeat(64), result_sha256: "3".repeat(64), output_sha256: "4".repeat(64), envelope_sha256: "5".repeat(64), candidate_set_sha256: "6".repeat(64), training_store_dataset_sha256: "7".repeat(64), selected_algorithm_three_seed_binding_sha256: "8".repeat(64), sealed_holdout_split_commitment_sha256: "9".repeat(64), sealed_holdout_projection_sha256: "a".repeat(64), feature_order_sha256: "b".repeat(64), preprocessing_sha256: "c".repeat(64), design_specification_sha256: "d".repeat(64), independently_recomputed_registration_sha256: "c".repeat(64), independently_recomputed_design_specification_sha256: "d".repeat(64), target_id: "excess_return_250d".to_string(), frozen_candidate_algorithm_id: "ridge".to_string(), submitted_at: Utc::now(), reviewer_id: "reviewer-new".to_string(), excluded_prior_actor_ids: vec!["registrar".to_string(), "stage73-reviewer".to_string()], verdict: ControlledShadowExperimentDesignRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityShadowImplementationRegistration,
            rationale: "设计绑定完整，进入下一层实现登记仍需继续隔离。".to_string(), risk_assessment: "风险预算保守，但仍需验证市场状态和退市偏差。".to_string(), known_limitations: "尚无真实前向样本，阈值只是工程候选。".to_string(), falsification_assessment: "泄漏、语义变化或覆盖不足立即停止。".to_string(), future_implementation_constraints: "实现必须零网络、只读输入且无账本入口。".to_string(), reviewer_independent_from_registrar_and_complete_prior_chain: true,
            exact_current_stage_51_through_stage_74_binding_confirmed: true, independent_recomputation_of_registration_and_design_fingerprints_confirmed: true, reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true, experimental_candidate_not_official_model_selection_confirmed: true, point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed: true, benchmark_and_all_counterfactual_semantics_reviewed: true, signal_timing_execution_cost_dividends_and_rebalance_reviewed: true, long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: true, minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed: true, separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: true, stop_rules_falsification_and_no_in_place_restart_reviewed: true, no_unconfirmed_hari_or_old_wang_logic_claimed: true, approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed: true, no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: true,
            design_registration_independently_approved: true, future_zero_capability_shadow_implementation_registration_eligible: true, official_model_selection_authorized: false, model_artifact_store_written: false, metric_store_written: false, training_feedback_authorized: false, reward_authorized: false, shadow_implementation_registered: false, shadow_run_authorized: false, shadow_ledger_enabled: false, shadow_position_written: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        };
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        review
    }

    #[test]
    fn approval_requires_every_review_check() {
        let mut review = minimal_review();
        assert!(review_all_checks(&review));
        review.point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed = false;
        assert!(!review_all_checks(&review));
    }

    #[test]
    fn registrar_or_prior_reviewer_cannot_review() {
        let excluded = vec!["registrar".to_string(), "prior-reviewer".to_string()];
        assert!(!reviewer_is_independent("registrar", &excluded));
        assert!(!reviewer_is_independent("prior-reviewer", &excluded));
        assert!(reviewer_is_independent("new-reviewer", &excluded));
    }

    #[test]
    fn review_fingerprint_binds_risk_assessment() {
        let review = minimal_review();
        let before = review.review_sha256.clone();
        let mut changed = review;
        changed.review_id.clear();
        changed.review_sha256.clear();
        changed.risk_assessment.push_str("新增风险");
        assert_ne!(before, review_fingerprint(&changed).unwrap());
    }

    #[test]
    fn approved_review_keeps_every_execution_authority_closed() {
        let review = minimal_review();
        assert!(review.design_registration_independently_approved);
        assert!(!review.shadow_implementation_registered);
        assert!(!review.shadow_run_authorized);
        assert!(!review.shadow_ledger_enabled);
        assert!(!review.shadow_position_written);
        assert!(!review.order_generation_authorized);
        assert!(!review.broker_access_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn review_chain_rejects_forks() {
        let first = minimal_review();
        let mut second = minimal_review();
        second.review_id = "e".repeat(32);
        second.review_sha256 = "e".repeat(64);
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some(first.review_sha256.clone());
        let mut fork = second.clone();
        fork.review_id = "f".repeat(32);
        fork.review_sha256 = "f".repeat(64);
        assert!(latest_review_from_records(&[first, second, fork]).is_err());
    }

    #[test]
    fn review_chain_requires_previous_reviewer_exclusion() {
        let first = minimal_review();
        let mut second = minimal_review();
        second.review_id = "e".repeat(32);
        second.review_sha256 = "e".repeat(64);
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some(first.review_sha256.clone());
        second.reviewer_id = "reviewer-two".to_string();
        second
            .excluded_prior_actor_ids
            .retain(|actor| actor != &first.reviewer_id);
        assert!(latest_review_from_records(&[first, second]).is_err());
    }

    #[test]
    fn required_text_is_bounded() {
        assert!(bounded_required("", "复核理由").is_err());
        assert!(bounded_required(&"a".repeat(MAX_TEXT_CHARS + 1), "复核理由").is_err());
    }
}
