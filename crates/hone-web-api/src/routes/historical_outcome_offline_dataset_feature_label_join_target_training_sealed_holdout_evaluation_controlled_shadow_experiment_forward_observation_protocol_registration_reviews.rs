//! Stage 83 chain-external, append-only review of one exact Stage 82 forward-observation
//! protocol registration.
//!
//! Approval opens only a future zero-capability observation-implementation specification
//! registration. It does not start observation, create a ledger or position, calculate
//! performance, write model/metric stores, enable feedback/reward, create orders, connect a
//! broker, or trade.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_protocol_registrations::{
    ControlledShadowForwardObservationProtocolRegistration,
    ControlledShadowForwardObservationProtocolSpecification,
    RegisteredControlledShadowForwardObservationProtocolForIndependentReview,
    registered_controlled_shadow_forward_observation_protocols_for_independent_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-registration-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-registration-review-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-chain-external-review-v1-no-observation";
const REGISTRATION_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-registration-v1";
const REGISTRATION_POLICY_VERSION: &str =
    "hone-controlled-shadow-natural-forward-only-protocol-create-once-v1-no-observation";
const PROTOCOL_SPEC_SCHEMA_VERSION: &str = "hone-controlled-shadow-forward-observation-protocol-v1";
const DESIGN_SPEC_SCHEMA_VERSION: &str = "hone-controlled-shadow-experiment-design-spec-v1";
const MAX_TEXT_CHARS: usize = 3_600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowForwardObservationProtocolRegistrationReviewVerdict {
    ApprovedForFutureZeroCapabilityForwardObservationImplementationRegistration,
    ChangesRequiredRebuildForwardObservationProtocol,
    RejectedForwardObservationProtocol,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowForwardObservationProtocolRegistrationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_protocol_registration_id: String,
    expected_protocol_registration_sha256: String,
    expected_protocol_specification_sha256: String,
    expected_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_input_manifest_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_design_specification_sha256: String,
    expected_candidate_set_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    verdict: ControlledShadowForwardObservationProtocolRegistrationReviewVerdict,
    rationale: String,
    natural_forward_assessment: String,
    calendar_and_timing_assessment: String,
    source_custody_and_correction_assessment: String,
    metric_and_stop_assessment: String,
    known_limitations: String,
    future_implementation_constraints: String,
    exact_current_stage_51_through_stage_82_binding_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed: bool,
    observation_not_before_and_no_retroactive_backfill_reviewed: bool,
    weekly_claim_first_create_once_reviewed: bool,
    official_us_market_calendar_half_days_halts_and_spy_sync_reviewed: bool,
    point_in_time_allowlist_content_addressing_and_source_availability_reviewed: bool,
    raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed:
        bool,
    next_full_session_fill_25bps_cost_and_counterfactuals_reviewed: bool,
    long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: bool,
    checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed: bool,
    separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: bool,
    stop_falsification_fail_closed_and_no_in_place_restart_reviewed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed:
        bool,
    no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed:
        bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationProtocolRegistrationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub attempt_id: String,
    pub validation_id: String,
    pub protocol_registration_id: String,
    pub protocol_registration_sha256: String,
    pub protocol_specification_sha256: String,
    pub independently_recomputed_protocol_registration_sha256: String,
    pub independently_recomputed_protocol_specification_sha256: String,
    pub independently_recomputed_exact_design_specification_sha256: String,
    pub validation_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub input_manifest_sha256: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub implementation_contract_sha256: String,
    pub design_specification_sha256: String,
    pub candidate_set_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowForwardObservationProtocolRegistrationReviewVerdict,
    pub rationale: String,
    pub natural_forward_assessment: String,
    pub calendar_and_timing_assessment: String,
    pub source_custody_and_correction_assessment: String,
    pub metric_and_stop_assessment: String,
    pub known_limitations: String,
    pub future_implementation_constraints: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_82_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed: bool,
    pub observation_not_before_and_no_retroactive_backfill_reviewed: bool,
    pub weekly_claim_first_create_once_reviewed: bool,
    pub official_us_market_calendar_half_days_halts_and_spy_sync_reviewed: bool,
    pub point_in_time_allowlist_content_addressing_and_source_availability_reviewed: bool,
    pub raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed:
        bool,
    pub next_full_session_fill_25bps_cost_and_counterfactuals_reviewed: bool,
    pub long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: bool,
    pub checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed: bool,
    pub separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: bool,
    pub stop_falsification_fail_closed_and_no_in_place_restart_reviewed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed:
        bool,
    pub no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed:
        bool,
    pub protocol_registration_independently_approved: bool,
    pub future_zero_capability_forward_observation_implementation_registration_eligible: bool,
    pub forward_observation_authorized: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationProtocolRegistrationReviewItem {
    pub registered_protocol:
        RegisteredControlledShadowForwardObservationProtocolForIndependentReview,
    pub latest_review: Option<ControlledShadowForwardObservationProtocolRegistrationReview>,
    pub review_eligible: bool,
    pub independently_approved: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationProtocolRegistrationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowForwardObservationProtocolRegistrationReviewItem>,
    pub protocol_registered_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_zero_capability_forward_observation_implementation_registration_eligible_count:
        usize,
    pub review_status: String,
    pub forward_observation_authorized: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowForwardObservationProtocolRegistrationReviewReadinessSummary {
    pub protocol_registered_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_zero_capability_forward_observation_implementation_registration_eligible_count:
        usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyApprovedControlledShadowForwardObservationProtocol {
    pub registered_protocol:
        RegisteredControlledShadowForwardObservationProtocolForIndependentReview,
    pub review: ControlledShadowForwardObservationProtocolRegistrationReview,
}

pub(crate) async fn independently_approved_controlled_shadow_forward_observation_protocols_for_implementation_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyApprovedControlledShadowForwardObservationProtocol>, String> {
    let registry = build_registry(state).await?;
    let mut output = registry
        .items
        .into_iter()
        .filter_map(|item| {
            item.latest_review.and_then(|review| {
                (item.independently_approved
                    && review.protocol_registration_independently_approved
                    && review
                        .future_zero_capability_forward_observation_implementation_registration_eligible)
                    .then_some(
                        IndependentlyApprovedControlledShadowForwardObservationProtocol {
                            registered_protocol: item.registered_protocol,
                            review,
                        },
                    )
            })
        })
        .collect::<Vec<_>>();
    for source in &output {
        validate_review(&source.review, &source.registered_protocol)?;
    }
    output.sort_by(|left, right| {
        right
            .review
            .submitted_at
            .cmp(&left.review.submitted_at)
            .then_with(|| right.review.review_id.cmp(&left.review.review_id))
    });
    Ok(output)
}

pub(crate) async fn handle_get_controlled_shadow_forward_observation_protocol_registration_reviews(
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
            warn!(%error, "controlled shadow forward-observation protocol review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "前向观察协议独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_forward_observation_protocol_registration(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(protocol_registration_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowForwardObservationProtocolRegistrationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_protocol(&state, &admin.user_id, &protocol_registration_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_forward_observation_protocol_registration_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationProtocolRegistrationReviewReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(ControlledShadowForwardObservationProtocolRegistrationReviewReadinessSummary {
        protocol_registered_count: value.protocol_registered_count,
        review_eligible_count: value.review_eligible_count,
        reviewed_count: value.reviewed_count,
        independently_approved_count: value.independently_approved_count,
        changes_required_or_rejected_count: value.changes_required_or_rejected_count,
        future_zero_capability_forward_observation_implementation_registration_eligible_count:
            value.future_zero_capability_forward_observation_implementation_registration_eligible_count,
        review_status: value.review_status,
    })
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationProtocolRegistrationReviewRegistry, String> {
    let mut items = Vec::new();
    for registered_protocol in
        registered_controlled_shadow_forward_observation_protocols_for_independent_review(state)
            .await?
    {
        let latest_review = latest_review(state, &registered_protocol).await?;
        let approved = latest_review
            .as_ref()
            .is_some_and(|review| review.protocol_registration_independently_approved);
        items.push(
            ControlledShadowForwardObservationProtocolRegistrationReviewItem {
                registered_protocol,
                latest_review,
                review_eligible: !approved,
                independently_approved: approved,
            },
        );
    }
    items.sort_by(|left, right| {
        left.registered_protocol
            .registration
            .protocol_registration_id
            .cmp(
                &right
                    .registered_protocol
                    .registration
                    .protocol_registration_id,
            )
    });
    let protocol_registered_count = items.len();
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|item| item.independently_approved)
        .count();
    let changes_required_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review
                .as_ref()
                .is_some_and(|review| !review.protocol_registration_independently_approved)
        })
        .count();
    let review_status = if independently_approved_count > 0 {
        "forward_observation_protocol_independently_approved_waiting_zero_capability_implementation_registration"
    } else if changes_required_or_rejected_count > 0 {
        "forward_observation_protocol_changes_required_rebuild_or_rejected"
    } else if protocol_registered_count > 0 {
        "waiting_chain_external_forward_observation_protocol_review"
    } else {
        "waiting_stage_82_forward_observation_protocol_registration"
    };
    Ok(ControlledShadowForwardObservationProtocolRegistrationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        protocol_registered_count,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_required_or_rejected_count,
        future_zero_capability_forward_observation_implementation_registration_eligible_count:
            independently_approved_count,
        review_status: review_status.to_string(),
        forward_observation_authorized: false,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 83 由责任链外新角色独立重算 Stage 82 登记、前向协议与完整 Stage 74 设计指纹，并逐项复核自然前向、禁止回填、claim-first、官方交易日历、SPY 同步、来源保管、公司行动、追加更正、下一完整交易日、单边 25bp、反事实、21/63/126/252 检查点、252/40/12/4 最低门槛、分项指标和停止规则。批准只开放未来零能力观察实现规格登记；当前不观察、不建账、不写持仓或绩效，不写模型/指标库，不开放反馈、reward、订单、券商或交易。".to_string(),
    })
}

async fn review_protocol(
    state: &AppState,
    reviewer_id: &str,
    protocol_registration_id: &str,
    request: ReviewControlledShadowForwardObservationProtocolRegistrationRequest,
) -> Result<ControlledShadowForwardObservationProtocolRegistrationReview, String> {
    if !valid_id(protocol_registration_id) {
        return Err("Stage 82 协议登记 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, protocol_registration_id).await?;
    let mut matches =
        registered_controlled_shadow_forward_observation_protocols_for_independent_review(state)
            .await?
            .into_iter()
            .filter(|value| value.registration.protocol_registration_id == protocol_registration_id)
            .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err("当前没有精确一条可独立复核的 Stage 82 协议登记".to_string());
    }
    let registered = matches.remove(0);
    validate_expected_binding(&registered.registration, &request)?;
    let latest = latest_review(state, &registered).await?;
    if latest
        .as_ref()
        .is_some_and(|review| review.protocol_registration_independently_approved)
    {
        return Err(
            "该协议已独立批准，复核链永久冻结；下一步只能登记零能力观察实现规格".to_string(),
        );
    }
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("前向观察协议复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&registered.registration, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("复核人不能是 Stage 82 登记人或完整 Stage 51–82 责任链角色".to_string());
    }
    let recomputed_registration = independent_registration_fingerprint(&registered.registration)?;
    let recomputed_protocol =
        independent_protocol_fingerprint(&registered.registration.protocol_specification)?;
    let recomputed_design = independent_design_fingerprint(
        &registered
            .registration
            .protocol_specification
            .exact_design_specification,
    )?;
    if recomputed_registration != registered.registration.protocol_registration_sha256
        || recomputed_protocol
            != registered
                .registration
                .protocol_specification
                .specification_sha256
        || recomputed_design != registered.registration.design_specification_sha256
    {
        return Err("Stage 82 登记、前向协议或 Stage 74 设计指纹独立重算不一致".to_string());
    }
    let r = &registered.registration;
    let mut review = ControlledShadowForwardObservationProtocolRegistrationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        attempt_id: r.attempt_id.clone(),
        validation_id: r.validation_id.clone(),
        protocol_registration_id: r.protocol_registration_id.clone(),
        protocol_registration_sha256: r.protocol_registration_sha256.clone(),
        protocol_specification_sha256: r.protocol_specification.specification_sha256.clone(),
        independently_recomputed_protocol_registration_sha256: recomputed_registration,
        independently_recomputed_protocol_specification_sha256: recomputed_protocol,
        independently_recomputed_exact_design_specification_sha256: recomputed_design,
        validation_sha256: r.validation_sha256.clone(),
        claim_sha256: r.claim_sha256.clone(),
        result_sha256: r.result_sha256.clone(),
        output_sha256: r.output_sha256.clone(),
        input_manifest_sha256: r.input_manifest_sha256.clone(),
        authorization_review_sha256: r.authorization_review_sha256.clone(),
        isolated_runner_spec_sha256: r.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: r.runner_artifact_sha256.clone(),
        implementation_contract_sha256: r.implementation_contract_sha256.clone(),
        design_specification_sha256: r.design_specification_sha256.clone(),
        candidate_set_sha256: r.candidate_set_sha256.clone(),
        feature_order_sha256: r.feature_order_sha256.clone(),
        preprocessing_sha256: r.preprocessing_sha256.clone(),
        target_id: r.target_id.clone(),
        frozen_candidate_algorithm_id: r.frozen_candidate_algorithm_id.clone(),
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        natural_forward_assessment: bounded_required(
            &request.natural_forward_assessment,
            "自然前向评估",
        )?,
        calendar_and_timing_assessment: bounded_required(
            &request.calendar_and_timing_assessment,
            "日历与时点评估",
        )?,
        source_custody_and_correction_assessment: bounded_required(
            &request.source_custody_and_correction_assessment,
            "来源保管与更正评估",
        )?,
        metric_and_stop_assessment: bounded_required(
            &request.metric_and_stop_assessment,
            "指标与停止规则评估",
        )?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_implementation_constraints: bounded_required(
            &request.future_implementation_constraints,
            "未来实现约束",
        )?,
        reviewer_independent_from_registrar_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_82_binding_confirmed: request
            .exact_current_stage_51_through_stage_82_binding_confirmed,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_registrar_and_complete_prior_chain_confirmed,
        independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed:
            request.independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed,
        observation_not_before_and_no_retroactive_backfill_reviewed: request
            .observation_not_before_and_no_retroactive_backfill_reviewed,
        weekly_claim_first_create_once_reviewed: request.weekly_claim_first_create_once_reviewed,
        official_us_market_calendar_half_days_halts_and_spy_sync_reviewed: request
            .official_us_market_calendar_half_days_halts_and_spy_sync_reviewed,
        point_in_time_allowlist_content_addressing_and_source_availability_reviewed: request
            .point_in_time_allowlist_content_addressing_and_source_availability_reviewed,
        raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed:
            request.raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed,
        next_full_session_fill_25bps_cost_and_counterfactuals_reviewed: request
            .next_full_session_fill_25bps_cost_and_counterfactuals_reviewed,
        long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: request
            .long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed,
        checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed: request
            .checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed,
        separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: request
            .separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed,
        stop_falsification_fail_closed_and_no_in_place_restart_reviewed: request
            .stop_falsification_fail_closed_and_no_in_place_restart_reviewed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed:
            request.approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed,
        no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed:
            request.no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed,
        protocol_registration_independently_approved: false,
        future_zero_capability_forward_observation_implementation_registration_eligible: false,
        forward_observation_authorized: false,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == ControlledShadowForwardObservationProtocolRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityForwardObservationImplementationRegistration
        && independent
        && all_review_checks(&review);
    review.protocol_registration_independently_approved = approved;
    review.future_zero_capability_forward_observation_implementation_registration_eligible =
        approved;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &registered)?;
    write_immutable_json(
        &review_directory(state, protocol_registration_id)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    r: &ControlledShadowForwardObservationProtocolRegistration,
    q: &ReviewControlledShadowForwardObservationProtocolRegistrationRequest,
) -> Result<(), String> {
    let exact = q.expected_protocol_registration_id == r.protocol_registration_id
        && q.expected_protocol_registration_sha256 == r.protocol_registration_sha256
        && q.expected_protocol_specification_sha256
            == r.protocol_specification.specification_sha256
        && q.expected_validation_sha256 == r.validation_sha256
        && q.expected_claim_sha256 == r.claim_sha256
        && q.expected_result_sha256 == r.result_sha256
        && q.expected_output_sha256 == r.output_sha256
        && q.expected_input_manifest_sha256 == r.input_manifest_sha256
        && q.expected_authorization_review_sha256 == r.authorization_review_sha256
        && q.expected_isolated_runner_spec_sha256 == r.isolated_runner_spec_sha256
        && q.expected_runner_artifact_sha256 == r.runner_artifact_sha256
        && q.expected_implementation_contract_sha256 == r.implementation_contract_sha256
        && q.expected_design_specification_sha256 == r.design_specification_sha256
        && q.expected_candidate_set_sha256 == r.candidate_set_sha256
        && q.expected_feature_order_sha256 == r.feature_order_sha256
        && q.expected_preprocessing_sha256 == r.preprocessing_sha256
        && q.expected_target_id == r.target_id
        && q.expected_frozen_candidate_algorithm_id == r.frozen_candidate_algorithm_id;
    exact
        .then_some(())
        .ok_or_else(|| "Stage 51–82 精确绑定已经变化，请刷新后重试".to_string())
}

fn all_review_checks(r: &ControlledShadowForwardObservationProtocolRegistrationReview) -> bool {
    r.exact_current_stage_51_through_stage_82_binding_confirmed
        && r.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && r.independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed
        && r.observation_not_before_and_no_retroactive_backfill_reviewed
        && r.weekly_claim_first_create_once_reviewed
        && r.official_us_market_calendar_half_days_halts_and_spy_sync_reviewed
        && r.point_in_time_allowlist_content_addressing_and_source_availability_reviewed
        && r.raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed
        && r.next_full_session_fill_25bps_cost_and_counterfactuals_reviewed
        && r.long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed
        && r.checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed
        && r.separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed
        && r.stop_falsification_fail_closed_and_no_in_place_restart_reviewed
        && r.no_unconfirmed_hari_or_old_wang_logic_claimed
        && r.approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed
        && r.no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed
}

fn validate_review(
    review: &ControlledShadowForwardObservationProtocolRegistrationReview,
    registered: &RegisteredControlledShadowForwardObservationProtocolForIndependentReview,
) -> Result<(), String> {
    let r = &registered.registration;
    let approving = review.verdict
        == ControlledShadowForwardObservationProtocolRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityForwardObservationImplementationRegistration;
    let independent =
        reviewer_is_independent(&review.reviewer_id, &review.excluded_prior_actor_ids);
    let expected_approved = approving && independent && all_review_checks(review);
    let required = excluded_actor_ids(r, &None);
    let texts_valid = [
        &review.rationale,
        &review.natural_forward_assessment,
        &review.calendar_and_timing_assessment,
        &review.source_custody_and_correction_assessment,
        &review.metric_and_stop_assessment,
        &review.known_limitations,
        &review.future_implementation_constraints,
    ]
    .into_iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let closed = !review.forward_observation_authorized
        && !review.forward_observation_ledger_created
        && !review.shadow_position_written
        && !review.performance_metric_written
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
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
        || review.attempt_id != r.attempt_id
        || review.validation_id != r.validation_id
        || review.protocol_registration_id != r.protocol_registration_id
        || review.protocol_registration_sha256 != r.protocol_registration_sha256
        || review.protocol_specification_sha256 != r.protocol_specification.specification_sha256
        || review.independently_recomputed_protocol_registration_sha256
            != independent_registration_fingerprint(r)?
        || review.independently_recomputed_protocol_specification_sha256
            != independent_protocol_fingerprint(&r.protocol_specification)?
        || review.independently_recomputed_exact_design_specification_sha256
            != independent_design_fingerprint(&r.protocol_specification.exact_design_specification)?
        || review.validation_sha256 != r.validation_sha256
        || review.claim_sha256 != r.claim_sha256
        || review.result_sha256 != r.result_sha256
        || review.output_sha256 != r.output_sha256
        || review.input_manifest_sha256 != r.input_manifest_sha256
        || review.authorization_review_sha256 != r.authorization_review_sha256
        || review.isolated_runner_spec_sha256 != r.isolated_runner_spec_sha256
        || review.runner_artifact_sha256 != r.runner_artifact_sha256
        || review.implementation_contract_sha256 != r.implementation_contract_sha256
        || review.design_specification_sha256 != r.design_specification_sha256
        || review.candidate_set_sha256 != r.candidate_set_sha256
        || review.feature_order_sha256 != r.feature_order_sha256
        || review.preprocessing_sha256 != r.preprocessing_sha256
        || review.target_id != r.target_id
        || review.frozen_candidate_algorithm_id != r.frozen_candidate_algorithm_id
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || required
            .iter()
            .any(|actor| !review.excluded_prior_actor_ids.contains(actor))
        || !independent
        || review.reviewer_independent_from_registrar_and_complete_prior_chain != independent
        || !texts_valid
        || (approving && !all_review_checks(review))
        || review.protocol_registration_independently_approved != expected_approved
        || review.future_zero_capability_forward_observation_implementation_registration_eligible
            != expected_approved
        || !closed
    {
        return Err("Stage 83 前向协议独立复核无效、漂移或越权".to_string());
    }
    Ok(())
}

fn excluded_actor_ids(
    registration: &ControlledShadowForwardObservationProtocolRegistration,
    latest: &Option<ControlledShadowForwardObservationProtocolRegistrationReview>,
) -> Vec<String> {
    let mut actors = registration.excluded_prior_actor_ids.clone();
    actors.push(registration.registered_by.clone());
    if let Some(review) = latest {
        actors.extend(review.excluded_prior_actor_ids.clone());
        actors.push(review.reviewer_id.clone());
    }
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn reviewer_is_independent(reviewer_id: &str, excluded_actor_ids: &[String]) -> bool {
    !reviewer_id.trim().is_empty() && excluded_actor_ids.iter().all(|value| value != reviewer_id)
}

fn independent_registration_fingerprint(
    value: &ControlledShadowForwardObservationProtocolRegistration,
) -> Result<String, String> {
    if value.schema_version != REGISTRATION_SCHEMA_VERSION
        || value.policy_version != REGISTRATION_POLICY_VERSION
    {
        return Err("Stage 82 登记 schema 或策略版本无效".to_string());
    }
    fingerprint_without(
        value,
        &["protocol_registration_id", "protocol_registration_sha256"],
    )
}

fn independent_protocol_fingerprint(
    value: &ControlledShadowForwardObservationProtocolSpecification,
) -> Result<String, String> {
    if value.schema_version != PROTOCOL_SPEC_SCHEMA_VERSION {
        return Err("Stage 82 前向协议 schema 无效".to_string());
    }
    fingerprint_without(value, &["specification_sha256"])
}

fn independent_design_fingerprint<T: Serialize>(value: &T) -> Result<String, String> {
    let json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    if json.get("schema_version").and_then(|value| value.as_str())
        != Some(DESIGN_SPEC_SCHEMA_VERSION)
    {
        return Err("Stage 74 设计 schema 无效".to_string());
    }
    fingerprint_without(value, &["specification_sha256"])
}

fn review_fingerprint(
    review: &ControlledShadowForwardObservationProtocolRegistrationReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

async fn latest_review(
    state: &AppState,
    registered: &RegisteredControlledShadowForwardObservationProtocolForIndependentReview,
) -> Result<Option<ControlledShadowForwardObservationProtocolRegistrationReview>, String> {
    let directory = review_directory(state, &registered.registration.protocol_registration_id);
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
        let review: ControlledShadowForwardObservationProtocolRegistrationReview =
            serde_json::from_slice(
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
            return Err("Stage 83 复核包含文件名不符或重复记录".to_string());
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[ControlledShadowForwardObservationProtocolRegistrationReview],
) -> Result<Option<ControlledShadowForwardObservationProtocolRegistrationReview>, String> {
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
        .count();
    let tips = reviews
        .iter()
        .filter(|review| {
            !reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
        })
        .collect::<Vec<_>>();
    if roots != 1 || tips.len() != 1 {
        return Err("Stage 83 复核链存在多根、分叉或多个链尖".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("Stage 83 复核链存在循环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "Stage 83 复核链断裂".to_string())?;
                if previous.protocol_registration_independently_approved
                    || previous.review_sha256 != *sha
                    || !review
                        .excluded_prior_actor_ids
                        .contains(&previous.reviewer_id)
                    || previous
                        .excluded_prior_actor_ids
                        .iter()
                        .any(|actor| !review.excluded_prior_actor_ids.contains(actor))
                {
                    return Err("Stage 83 批准终止、前序哈希或角色排除继承不一致".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("Stage 83 前序链接无效".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("Stage 83 复核链断裂".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 83 指纹载荷无效".to_string())?;
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
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"))
    } else {
        Ok(value.to_string())
    }
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

fn review_directory(state: &AppState, protocol_registration_id: &str) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-forward-observation-protocol-registration-reviews")
        .join(protocol_registration_id)
}

struct ReviewLock {
    path: PathBuf,
}

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    protocol_registration_id: &str,
) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-forward-observation-protocol-review-{protocol_registration_id}.lock"
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
        .map_err(|_| "已有同一 Stage 83 协议复核正在进行".to_string())?;
    Ok(ReviewLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_review() -> ControlledShadowForwardObservationProtocolRegistrationReview {
        let mut review = ControlledShadowForwardObservationProtocolRegistrationReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(), review_id: String::new(), review_sha256: String::new(), previous_review_id: None, previous_review_sha256: None,
            attempt_id: "a".repeat(32), validation_id: "b".repeat(32), protocol_registration_id: "c".repeat(32), protocol_registration_sha256: "d".repeat(64), protocol_specification_sha256: "e".repeat(64), independently_recomputed_protocol_registration_sha256: "d".repeat(64), independently_recomputed_protocol_specification_sha256: "e".repeat(64), independently_recomputed_exact_design_specification_sha256: "f".repeat(64), validation_sha256: "1".repeat(64), claim_sha256: "2".repeat(64), result_sha256: "3".repeat(64), output_sha256: "4".repeat(64), input_manifest_sha256: "5".repeat(64), authorization_review_sha256: "6".repeat(64), isolated_runner_spec_sha256: "7".repeat(64), runner_artifact_sha256: "8".repeat(64), implementation_contract_sha256: "9".repeat(64), design_specification_sha256: "f".repeat(64), candidate_set_sha256: "a".repeat(64), feature_order_sha256: "b".repeat(64), preprocessing_sha256: "c".repeat(64), target_id: "excess_return_250d".to_string(), frozen_candidate_algorithm_id: "ridge".to_string(), submitted_at: Utc::now(), reviewer_id: "stage-83-reviewer".to_string(), excluded_prior_actor_ids: vec!["stage-81-validator".to_string(), "stage-82-registrar".to_string()], verdict: ControlledShadowForwardObservationProtocolRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityForwardObservationImplementationRegistration,
            rationale: "协议绑定完整，但仍需零能力实现登记。".to_string(), natural_forward_assessment: "只接受批准后自然到来的观察。".to_string(), calendar_and_timing_assessment: "官方日历、半日市、停牌和 SPY 同步已逐项检查。".to_string(), source_custody_and_correction_assessment: "点时来源内容寻址，公司行动只追加更正。".to_string(), metric_and_stop_assessment: "检查点、样本门槛、分项指标与停止规则保持冻结。".to_string(), known_limitations: "尚无真实前向样本。".to_string(), future_implementation_constraints: "下一阶段仍不得观察、建账或交易。".to_string(), reviewer_independent_from_registrar_and_complete_prior_chain: true,
            exact_current_stage_51_through_stage_82_binding_confirmed: true, reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true, independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed: true, observation_not_before_and_no_retroactive_backfill_reviewed: true, weekly_claim_first_create_once_reviewed: true, official_us_market_calendar_half_days_halts_and_spy_sync_reviewed: true, point_in_time_allowlist_content_addressing_and_source_availability_reviewed: true, raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed: true, next_full_session_fill_25bps_cost_and_counterfactuals_reviewed: true, long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: true, checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed: true, separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: true, stop_falsification_fail_closed_and_no_in_place_restart_reviewed: true, no_unconfirmed_hari_or_old_wang_logic_claimed: true, approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed: true, no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed: true,
            protocol_registration_independently_approved: true, future_zero_capability_forward_observation_implementation_registration_eligible: true, forward_observation_authorized: false, forward_observation_ledger_created: false, shadow_position_written: false, performance_metric_written: false, model_artifact_store_written: false, metric_store_written: false, training_feedback_authorized: false, reward_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        };
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        review
    }

    #[test]
    fn approval_requires_every_protocol_review_check() {
        let mut review = minimal_review();
        assert!(all_review_checks(&review));
        review.observation_not_before_and_no_retroactive_backfill_reviewed = false;
        assert!(!all_review_checks(&review));
    }

    #[test]
    fn registrar_and_prior_roles_are_excluded() {
        let excluded = vec!["registrar".to_string(), "validator".to_string()];
        assert!(!reviewer_is_independent("registrar", &excluded));
        assert!(reviewer_is_independent("new-reviewer", &excluded));
    }

    #[test]
    fn review_fingerprint_binds_natural_forward_assessment() {
        let review = minimal_review();
        let before = review.review_sha256.clone();
        let mut changed = review;
        changed.review_id.clear();
        changed.review_sha256.clear();
        changed.natural_forward_assessment.push_str("篡改");
        assert_ne!(before, review_fingerprint(&changed).unwrap());
    }

    #[test]
    fn approval_keeps_all_execution_authorities_closed() {
        let review = minimal_review();
        assert!(review.protocol_registration_independently_approved);
        assert!(!review.forward_observation_authorized);
        assert!(!review.forward_observation_ledger_created);
        assert!(!review.shadow_position_written);
        assert!(!review.performance_metric_written);
        assert!(!review.order_generation_authorized);
        assert!(!review.broker_access_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn review_chain_rejects_multiple_roots_and_forks() {
        let first = minimal_review();
        let mut second = minimal_review();
        second.review_id = "1".repeat(32);
        second.review_sha256 = "1".repeat(64);
        assert!(latest_review_from_records(&[first.clone(), second]).is_err());
        let mut child = first.clone();
        child.review_id = "2".repeat(32);
        child.review_sha256 = "2".repeat(64);
        child.previous_review_id = Some(first.review_id.clone());
        child.previous_review_sha256 = Some(first.review_sha256.clone());
        child
            .excluded_prior_actor_ids
            .push(first.reviewer_id.clone());
        child.excluded_prior_actor_ids.sort();
        child.excluded_prior_actor_ids.dedup();
        let mut fork = child.clone();
        fork.review_id = "3".repeat(32);
        fork.review_sha256 = "3".repeat(64);
        assert!(latest_review_from_records(&[first, child, fork]).is_err());
    }

    #[test]
    fn approved_review_is_terminal() {
        let first = minimal_review();
        let mut child = minimal_review();
        child.review_id = "4".repeat(32);
        child.review_sha256 = "4".repeat(64);
        child.previous_review_id = Some(first.review_id.clone());
        child.previous_review_sha256 = Some(first.review_sha256.clone());
        child
            .excluded_prior_actor_ids
            .push(first.reviewer_id.clone());
        child.excluded_prior_actor_ids.sort();
        child.excluded_prior_actor_ids.dedup();
        assert!(latest_review_from_records(&[first, child]).is_err());
    }
}
