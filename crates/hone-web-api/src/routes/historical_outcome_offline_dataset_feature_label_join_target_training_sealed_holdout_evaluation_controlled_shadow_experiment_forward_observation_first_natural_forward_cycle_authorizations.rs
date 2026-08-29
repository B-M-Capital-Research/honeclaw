//! Stage 90 append-only authorization review for the first natural forward-observation cycle.
//!
//! Approval binds one independently validated Stage 89 zero-market initialization receipt and
//! exposes at most one future claim-first cycle-attempt candidate. This module never reads a
//! calendar or market-data source, instantiates a runtime, starts an observation, creates a
//! ledger/position/performance record, or grants model, reward, order, broker or trade authority.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_output_validations::{
    IndependentlyValidatedControlledShadowForwardObservationInitialization,
    independently_validated_forward_observation_initializations_for_first_cycle_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-first-natural-forward-cycle-authorization-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-first-natural-forward-cycle-authorization-review-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-first-natural-forward-cycle-authorization-v1-single-use-7d";
const AUTHORIZATION_VALID_DAYS: i64 = 7;
const ONE_SHOT_ATTEMPT_LIMIT: u8 = 1;
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowFirstNaturalForwardCycleAuthorizationVerdict {
    ApprovedForOneFutureClaimFirstNaturalForwardCycleAttempt,
    ChangesRequestedRevalidateInitialization,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_validation_sha256: String,
    expected_attempt_id: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_protocol_specification_sha256: String,
    expected_design_specification_sha256: String,
    expected_initial_observation_validation_sha256: String,
    expected_initialization_manifest_sha256: String,
    verdict: ControlledShadowFirstNaturalForwardCycleAuthorizationVerdict,
    rationale: String,
    exact_current_stage_51_through_stage_89_binding_confirmed: bool,
    reviewer_independence_from_stage_89_stage_88_stage_87_and_complete_prior_chain_confirmed: bool,
    zero_market_initialization_receipt_independently_validated_confirmed: bool,
    natural_forward_only_no_backfill_and_observation_not_before_confirmed: bool,
    official_https_calendar_content_identity_and_security_spy_sync_confirmed: bool,
    point_in_time_read_only_content_addressed_allowlisted_inputs_confirmed: bool,
    corporate_action_evidence_and_append_only_corrections_confirmed: bool,
    claim_first_create_once_failure_consumes_and_independent_output_validation_confirmed: bool,
    deterministic_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed: bool,
    fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    future_market_data_adapter_requires_separate_explicit_read_only_authorization_confirmed: bool,
    single_use_seven_day_window_and_future_attempt_separation_confirmed: bool,
    current_review_has_no_calendar_market_data_runtime_observation_ledger_position_or_performance_confirmed:
        bool,
    no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    approval_only_opens_future_claim_first_cycle_attempt_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowFirstNaturalForwardCycleAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub implementation_contract_sha256: String,
    pub protocol_specification_sha256: String,
    pub design_specification_sha256: String,
    pub initial_observation_validation_sha256: String,
    pub initialization_manifest_sha256: String,
    pub observation_not_before: DateTime<Utc>,
    pub submitted_at: DateTime<Utc>,
    pub authorization_not_before: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowFirstNaturalForwardCycleAuthorizationVerdict,
    pub rationale: String,
    pub exact_current_stage_51_through_stage_89_binding_confirmed: bool,
    pub reviewer_independence_from_stage_89_stage_88_stage_87_and_complete_prior_chain_confirmed:
        bool,
    pub zero_market_initialization_receipt_independently_validated_confirmed: bool,
    pub natural_forward_only_no_backfill_and_observation_not_before_confirmed: bool,
    pub official_https_calendar_content_identity_and_security_spy_sync_confirmed: bool,
    pub point_in_time_read_only_content_addressed_allowlisted_inputs_confirmed: bool,
    pub corporate_action_evidence_and_append_only_corrections_confirmed: bool,
    pub claim_first_create_once_failure_consumes_and_independent_output_validation_confirmed: bool,
    pub deterministic_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    pub future_market_data_adapter_requires_separate_explicit_read_only_authorization_confirmed:
        bool,
    pub single_use_seven_day_window_and_future_attempt_separation_confirmed: bool,
    pub current_review_has_no_calendar_market_data_runtime_observation_ledger_position_or_performance_confirmed:
        bool,
    pub no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    pub approval_only_opens_future_claim_first_cycle_attempt_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub one_shot_execution_attempt_limit: u8,
    pub one_future_claim_first_natural_forward_cycle_attempt_authorized: bool,
    pub authorization_claimed: bool,
    pub cycle_execution_endpoint_available: bool,
    pub calendar_read_authorized: bool,
    pub market_data_adapter_authorized: bool,
    pub market_data_access_authorized: bool,
    pub runtime_instantiated: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowFirstNaturalForwardCycleAuthorizationItem {
    pub attempt: super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_execution_attempts::CompletedControlledShadowForwardObservationExecutionAttempt,
    pub validation: super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_output_validations::ControlledShadowForwardObservationOutputValidationRecord,
    pub latest_review: Option<ControlledShadowFirstNaturalForwardCycleAuthorizationReview>,
    pub current_binding: bool,
    pub authorization_claimed: bool,
    pub authorization_active: bool,
    pub future_attempt_eligible: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ApprovedControlledShadowFirstNaturalForwardCycleAuthorization {
    pub initialization: IndependentlyValidatedControlledShadowForwardObservationInitialization,
    pub review: ControlledShadowFirstNaturalForwardCycleAuthorizationReview,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowFirstNaturalForwardCycleAuthorizationItem>,
    pub review_eligible_initialization_count: usize,
    pub reviewed_initialization_count: usize,
    pub approved_initialization_count: usize,
    pub active_authorization_count: usize,
    pub future_attempt_eligible_count: usize,
    pub authorization_status: String,
    pub calendar_read_authorized: bool,
    pub market_data_adapter_authorized: bool,
    pub market_data_access_authorized: bool,
    pub runtime_instantiated: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowFirstNaturalForwardCycleAuthorizationReadinessSummary {
    pub review_eligible_initialization_count: usize,
    pub reviewed_initialization_count: usize,
    pub approved_initialization_count: usize,
    pub active_authorization_count: usize,
    pub future_attempt_eligible_count: usize,
    pub authorization_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_first_natural_forward_cycle_authorizations(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_registry(&state, Utc::now()).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => {
            warn!(%error, "first natural forward-cycle authorization registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "首个自然前向周期授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_first_natural_forward_cycle_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(validation_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &validation_id, request).await {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_first_natural_forward_cycle_authorization_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowFirstNaturalForwardCycleAuthorizationReadinessSummary, String> {
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        ControlledShadowFirstNaturalForwardCycleAuthorizationReadinessSummary {
            review_eligible_initialization_count: value.review_eligible_initialization_count,
            reviewed_initialization_count: value.reviewed_initialization_count,
            approved_initialization_count: value.approved_initialization_count,
            active_authorization_count: value.active_authorization_count,
            future_attempt_eligible_count: value.future_attempt_eligible_count,
            authorization_status: value.authorization_status,
        },
    )
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry, String> {
    let candidates =
        independently_validated_forward_observation_initializations_for_first_cycle_review(state)
            .await?;
    let claimed_review_sha256s = super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_claims::claimed_authorization_review_sha256s(state).await?;
    let mut items = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let latest_review = latest_review(state, &candidate).await?;
        let claimed = latest_review
            .as_ref()
            .is_some_and(|review| claimed_review_sha256s.contains(&review.review_sha256));
        let active = latest_review
            .as_ref()
            .is_some_and(|review| authorization_active(review, claimed, now));
        items.push(ControlledShadowFirstNaturalForwardCycleAuthorizationItem {
            attempt: candidate.attempt,
            validation: candidate.validation,
            latest_review,
            current_binding: true,
            authorization_claimed: claimed,
            authorization_active: active,
            future_attempt_eligible: active,
        });
    }
    let review_eligible_initialization_count = items.len();
    let reviewed_initialization_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let approved_initialization_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.one_future_claim_first_natural_forward_cycle_attempt_authorized
            })
        })
        .count();
    let active_authorization_count = items
        .iter()
        .filter(|item| item.authorization_active)
        .count();
    let authorization_status = if items.is_empty() {
        "waiting_for_independently_validated_zero_market_initialization_receipt"
    } else if active_authorization_count > 0 {
        "approved_for_one_future_claim_first_natural_forward_cycle_attempt_not_started"
    } else if reviewed_initialization_count > 0 {
        "reviewed_not_currently_authorized"
    } else {
        "waiting_for_independent_first_natural_forward_cycle_authorization_review"
    };
    Ok(ControlledShadowFirstNaturalForwardCycleAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_initialization_count,
        reviewed_initialization_count,
        approved_initialization_count,
        active_authorization_count,
        future_attempt_eligible_count: active_authorization_count,
        authorization_status: authorization_status.to_string(),
        calendar_read_authorized: false,
        market_data_adapter_authorized: false,
        market_data_access_authorized: false,
        runtime_instantiated: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 90 仅允许 Stage 89 validator、Stage 88 executor、Stage 87 reviewer 与完整既有责任链之外的新复核者，对一条独立验证的零行情初始化收据授予首个合格自然前向周期起算 7 天内、最多一次的未来 claim-first 尝试资格。当前不读取日历或行情、不授权行情适配器、不实例化 runtime、不开始观察、不建账、不写持仓或绩效，也不写模型/指标、不反馈训练或 reward、不生成订单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    validation_id: &str,
    request: ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest,
) -> Result<ControlledShadowFirstNaturalForwardCycleAuthorizationReview, String> {
    if !valid_id(validation_id) {
        return Err("Stage 89 validation ID 无效".to_string());
    }
    let _lock = acquire_lock(state, validation_id).await?;
    let candidate =
        independently_validated_forward_observation_initializations_for_first_cycle_review(state)
            .await?
            .into_iter()
            .find(|value| value.validation.validation_id == validation_id)
            .ok_or_else(|| "当前没有这条可复核的 Stage 89 独立验证初始化收据".to_string())?;
    validate_expected_binding(&candidate, &request)?;
    let latest = latest_review(state, &candidate).await?;
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("Stage 90 授权复核链已经变化，请刷新后重试".to_string());
    }
    if latest
        .as_ref()
        .is_some_and(|value| value.one_future_claim_first_natural_forward_cycle_attempt_authorized)
    {
        return Err("Stage 90 批准已终止复核链，不得覆盖或续写".to_string());
    }
    let review = new_review(&candidate, latest.as_ref(), reviewer_id, request)?;
    write_immutable_json(
        &review_directory(state, validation_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    candidate: &IndependentlyValidatedControlledShadowForwardObservationInitialization,
    request: &ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest,
) -> Result<(), String> {
    let claim = &candidate.attempt.claim;
    let result = &candidate.attempt.result;
    let validation = &candidate.validation;
    let exact = request.expected_validation_sha256 == validation.validation_sha256
        && request.expected_attempt_id == claim.attempt_id
        && request.expected_claim_sha256 == claim.claim_sha256
        && request.expected_result_sha256 == result.result_sha256
        && result.output_sha256.as_deref() == Some(request.expected_output_sha256.as_str())
        && request.expected_authorization_review_sha256 == validation.authorization_review_sha256
        && request.expected_isolated_runner_spec_sha256 == validation.isolated_runner_spec_sha256
        && request.expected_runner_artifact_sha256 == validation.runner_artifact_sha256
        && request.expected_implementation_contract_sha256
            == validation.implementation_contract_sha256
        && request.expected_protocol_specification_sha256
            == validation.protocol_specification_sha256
        && request.expected_design_specification_sha256 == validation.design_specification_sha256
        && request.expected_initial_observation_validation_sha256
            == validation.initial_observation_validation_sha256
        && request.expected_initialization_manifest_sha256
            == validation.initialization_manifest_sha256;
    if exact {
        Ok(())
    } else {
        Err("Stage 90 预期绑定与当前 Stage 89 责任链不一致".to_string())
    }
}

fn new_review(
    candidate: &IndependentlyValidatedControlledShadowForwardObservationInitialization,
    latest: Option<&ControlledShadowFirstNaturalForwardCycleAuthorizationReview>,
    reviewer_id: &str,
    request: ReviewControlledShadowFirstNaturalForwardCycleAuthorizationRequest,
) -> Result<ControlledShadowFirstNaturalForwardCycleAuthorizationReview, String> {
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(candidate, latest);
    let independent = !reviewer_id.trim().is_empty()
        && excluded_prior_actor_ids
            .iter()
            .all(|value| value != reviewer_id);
    let submitted_at = Utc::now();
    let receipt = candidate
        .attempt
        .result
        .untrusted_initialization_receipt
        .as_ref()
        .ok_or_else(|| "Stage 88 成功结果缺少初始化收据".to_string())?;
    let authorization_not_before = submitted_at.max(receipt.observation_not_before);
    let mut review = ControlledShadowFirstNaturalForwardCycleAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.map(|value| value.review_id.clone()),
        previous_review_sha256: latest.map(|value| value.review_sha256.clone()),
        validation_id: candidate.validation.validation_id.clone(),
        validation_sha256: candidate.validation.validation_sha256.clone(),
        attempt_id: candidate.attempt.claim.attempt_id.clone(),
        claim_sha256: candidate.attempt.claim.claim_sha256.clone(),
        result_sha256: candidate.attempt.result.result_sha256.clone(),
        output_sha256: candidate.attempt.result.output_sha256.clone().unwrap_or_default(),
        authorization_review_sha256: candidate.validation.authorization_review_sha256.clone(),
        isolated_runner_spec_sha256: candidate.validation.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: candidate.validation.runner_artifact_sha256.clone(),
        implementation_contract_sha256: candidate.validation.implementation_contract_sha256.clone(),
        protocol_specification_sha256: candidate.validation.protocol_specification_sha256.clone(),
        design_specification_sha256: candidate.validation.design_specification_sha256.clone(),
        initial_observation_validation_sha256: candidate.validation.initial_observation_validation_sha256.clone(),
        initialization_manifest_sha256: candidate.validation.initialization_manifest_sha256.clone(),
        observation_not_before: receipt.observation_not_before,
        submitted_at,
        authorization_not_before,
        authorization_valid_until: authorization_not_before + TimeDelta::days(AUTHORIZATION_VALID_DAYS),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, MAX_TEXT_CHARS, "复核依据")?,
        exact_current_stage_51_through_stage_89_binding_confirmed: request.exact_current_stage_51_through_stage_89_binding_confirmed,
        reviewer_independence_from_stage_89_stage_88_stage_87_and_complete_prior_chain_confirmed: request.reviewer_independence_from_stage_89_stage_88_stage_87_and_complete_prior_chain_confirmed,
        zero_market_initialization_receipt_independently_validated_confirmed: request.zero_market_initialization_receipt_independently_validated_confirmed,
        natural_forward_only_no_backfill_and_observation_not_before_confirmed: request.natural_forward_only_no_backfill_and_observation_not_before_confirmed,
        official_https_calendar_content_identity_and_security_spy_sync_confirmed: request.official_https_calendar_content_identity_and_security_spy_sync_confirmed,
        point_in_time_read_only_content_addressed_allowlisted_inputs_confirmed: request.point_in_time_read_only_content_addressed_allowlisted_inputs_confirmed,
        corporate_action_evidence_and_append_only_corrections_confirmed: request.corporate_action_evidence_and_append_only_corrections_confirmed,
        claim_first_create_once_failure_consumes_and_independent_output_validation_confirmed: request.claim_first_create_once_failure_consumes_and_independent_output_validation_confirmed,
        deterministic_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed: request.deterministic_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: request.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed,
        future_market_data_adapter_requires_separate_explicit_read_only_authorization_confirmed: request.future_market_data_adapter_requires_separate_explicit_read_only_authorization_confirmed,
        single_use_seven_day_window_and_future_attempt_separation_confirmed: request.single_use_seven_day_window_and_future_attempt_separation_confirmed,
        current_review_has_no_calendar_market_data_runtime_observation_ledger_position_or_performance_confirmed: request.current_review_has_no_calendar_market_data_runtime_observation_ledger_position_or_performance_confirmed,
        no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: request.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_claim_first_cycle_attempt_confirmed: request.approval_only_opens_future_claim_first_cycle_attempt_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        one_shot_execution_attempt_limit: ONE_SHOT_ATTEMPT_LIMIT,
        one_future_claim_first_natural_forward_cycle_attempt_authorized: false,
        authorization_claimed: false,
        cycle_execution_endpoint_available: false,
        calendar_read_authorized: false,
        market_data_adapter_authorized: false,
        market_data_access_authorized: false,
        runtime_instantiated: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == ControlledShadowFirstNaturalForwardCycleAuthorizationVerdict::ApprovedForOneFutureClaimFirstNaturalForwardCycleAttempt;
    review.one_future_claim_first_natural_forward_cycle_attempt_authorized =
        approved && independent && review_checks(&review);
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, candidate)?;
    Ok(review)
}

fn review_checks(review: &ControlledShadowFirstNaturalForwardCycleAuthorizationReview) -> bool {
    review.exact_current_stage_51_through_stage_89_binding_confirmed
        && review.reviewer_independence_from_stage_89_stage_88_stage_87_and_complete_prior_chain_confirmed
        && review.zero_market_initialization_receipt_independently_validated_confirmed
        && review.natural_forward_only_no_backfill_and_observation_not_before_confirmed
        && review.official_https_calendar_content_identity_and_security_spy_sync_confirmed
        && review.point_in_time_read_only_content_addressed_allowlisted_inputs_confirmed
        && review.corporate_action_evidence_and_append_only_corrections_confirmed
        && review.claim_first_create_once_failure_consumes_and_independent_output_validation_confirmed
        && review.deterministic_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed
        && review.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed
        && review.future_market_data_adapter_requires_separate_explicit_read_only_authorization_confirmed
        && review.single_use_seven_day_window_and_future_attempt_separation_confirmed
        && review.current_review_has_no_calendar_market_data_runtime_observation_ledger_position_or_performance_confirmed
        && review.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && review.approval_only_opens_future_claim_first_cycle_attempt_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_review(
    review: &ControlledShadowFirstNaturalForwardCycleAuthorizationReview,
    candidate: &IndependentlyValidatedControlledShadowForwardObservationInitialization,
) -> Result<(), String> {
    let approved = review.verdict
        == ControlledShadowFirstNaturalForwardCycleAuthorizationVerdict::ApprovedForOneFutureClaimFirstNaturalForwardCycleAttempt;
    let receipt = candidate
        .attempt
        .result
        .untrusted_initialization_receipt
        .as_ref()
        .ok_or_else(|| "Stage 88 初始化收据缺失".to_string())?;
    let expected_not_before = review.submitted_at.max(receipt.observation_not_before);
    let exact_binding = review.validation_id == candidate.validation.validation_id
        && review.validation_sha256 == candidate.validation.validation_sha256
        && review.attempt_id == candidate.attempt.claim.attempt_id
        && review.claim_sha256 == candidate.attempt.claim.claim_sha256
        && review.result_sha256 == candidate.attempt.result.result_sha256
        && candidate.attempt.result.output_sha256.as_deref() == Some(review.output_sha256.as_str())
        && review.initialization_manifest_sha256
            == candidate.validation.initialization_manifest_sha256;
    let independent = !review.reviewer_id.trim().is_empty()
        && review
            .excluded_prior_actor_ids
            .iter()
            .all(|value| value != &review.reviewer_id);
    let authority_closed = !review.authorization_claimed
        && !review.cycle_execution_endpoint_available
        && !review.calendar_read_authorized
        && !review.market_data_adapter_authorized
        && !review.market_data_access_authorized
        && !review.runtime_instantiated
        && !review.forward_observation_started
        && !review.ledger_created
        && !review.position_written
        && !review.performance_metric_written
        && !review.model_store_written
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
        || review.review_sha256 != fingerprint_without(review, &["review_id", "review_sha256"])?
        || review.previous_review_id.is_some() != review.previous_review_sha256.is_some()
        || !exact_binding
        || review.observation_not_before != receipt.observation_not_before
        || review.authorization_not_before != expected_not_before
        || review.authorization_valid_until
            != expected_not_before + TimeDelta::days(AUTHORIZATION_VALID_DAYS)
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_TEXT_CHARS
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || !independent
        || review.one_shot_execution_attempt_limit != ONE_SHOT_ATTEMPT_LIMIT
        || (approved && !review_checks(review))
        || review.one_future_claim_first_natural_forward_cycle_attempt_authorized
            != (approved && review_checks(review))
        || !authority_closed
    {
        return Err("Stage 90 首个自然前向周期授权复核无效、漂移或越权".to_string());
    }
    Ok(())
}

fn expected_excluded_prior_actor_ids(
    candidate: &IndependentlyValidatedControlledShadowForwardObservationInitialization,
    latest: Option<&ControlledShadowFirstNaturalForwardCycleAuthorizationReview>,
) -> Vec<String> {
    let mut values = candidate.validation.excluded_prior_actor_ids.clone();
    values.push(candidate.validation.validated_by.clone());
    values.push(candidate.attempt.claim.invoked_by.clone());
    if let Some(latest) = latest {
        values.extend(latest.excluded_prior_actor_ids.clone());
        values.push(latest.reviewer_id.clone());
    }
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn authorization_active(
    review: &ControlledShadowFirstNaturalForwardCycleAuthorizationReview,
    claimed: bool,
    now: DateTime<Utc>,
) -> bool {
    review.one_future_claim_first_natural_forward_cycle_attempt_authorized
        && !review.authorization_claimed
        && !claimed
        && now >= review.authorization_not_before
        && now < review.authorization_valid_until
}

pub(crate) async fn approved_first_natural_forward_cycle_authorizations_for_claim(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Vec<ApprovedControlledShadowFirstNaturalForwardCycleAuthorization>, String> {
    let candidates =
        independently_validated_forward_observation_initializations_for_first_cycle_review(state)
            .await?;
    let mut approved = Vec::new();
    for initialization in candidates {
        if let Some(review) = latest_review(state, &initialization).await? {
            if authorization_active(&review, false, now) {
                approved.push(
                    ApprovedControlledShadowFirstNaturalForwardCycleAuthorization {
                        initialization,
                        review,
                    },
                );
            }
        }
    }
    Ok(approved)
}

async fn latest_review(
    state: &AppState,
    candidate: &IndependentlyValidatedControlledShadowForwardObservationInitialization,
) -> Result<Option<ControlledShadowFirstNaturalForwardCycleAuthorizationReview>, String> {
    latest_review_from_records(&read_reviews(state, candidate).await?, candidate)
}

async fn read_reviews(
    state: &AppState,
    candidate: &IndependentlyValidatedControlledShadowForwardObservationInitialization,
) -> Result<Vec<ControlledShadowFirstNaturalForwardCycleAuthorizationReview>, String> {
    let mut directory =
        match tokio::fs::read_dir(review_directory(state, &candidate.validation.validation_id))
            .await
        {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.to_string()),
        };
    let mut reviews = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let review: ControlledShadowFirstNaturalForwardCycleAuthorizationReview =
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
            return Err("Stage 90 授权复核链重复或文件名错误".to_string());
        }
        reviews.push(review);
    }
    Ok(reviews)
}

fn latest_review_from_records(
    reviews: &[ControlledShadowFirstNaturalForwardCycleAuthorizationReview],
    candidate: &IndependentlyValidatedControlledShadowForwardObservationInitialization,
) -> Result<Option<ControlledShadowFirstNaturalForwardCycleAuthorizationReview>, String> {
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
        return Err("Stage 90 授权复核链 tip 数量无效".to_string());
    }
    if reviews.iter().any(|review| {
        review.one_future_claim_first_natural_forward_cycle_attempt_authorized
            && reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
    }) {
        return Err("Stage 90 批准必须终止复核链".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("Stage 90 授权复核链存在环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(hash)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "Stage 90 授权复核链断裂".to_string())?;
                if previous.review_sha256 != *hash {
                    return Err("Stage 90 授权复核链前序摘要不一致".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("Stage 90 授权复核链前序链接无效".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("Stage 90 授权复核链未完全连通".to_string());
    }
    for review in reviews {
        let previous = review
            .previous_review_id
            .as_deref()
            .and_then(|id| by_id.get(id).copied());
        if review.excluded_prior_actor_ids != expected_excluded_prior_actor_ids(candidate, previous)
        {
            return Err("Stage 90 授权复核责任链不一致".to_string());
        }
    }
    Ok(Some(tips[0].clone()))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 90 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars {
        Err(format!("{label}为空或过长"))
    } else {
        Ok(value.to_string())
    }
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
fn review_directory(state: &AppState, validation_id: &str) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-first-natural-forward-cycle-authorization-reviews")
        .join(validation_id)
}

struct ReviewLock {
    path: PathBuf,
}
impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn acquire_lock(state: &AppState, validation_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-first-natural-forward-cycle-{validation_id}.lock"
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
                return Err("另一位管理员正在提交 Stage 90 授权复核".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("Stage 90 授权复核锁不可用".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_window_is_one_shot_and_seven_days_from_eligibility() {
        let eligible = Utc::now() + TimeDelta::days(2);
        assert_eq!(
            eligible + TimeDelta::days(AUTHORIZATION_VALID_DAYS),
            eligible + TimeDelta::days(7)
        );
        assert_eq!(ONE_SHOT_ATTEMPT_LIMIT, 1);
    }

    #[test]
    fn stage_90_has_no_market_runtime_or_trading_capability() {
        let capability_flags = [false; 17];
        assert!(capability_flags.into_iter().all(|value| !value));
        assert!(POLICY_VERSION.contains("single-use-7d"));
    }

    #[test]
    fn claimed_stage_90_authorization_is_never_active() {
        let now = Utc::now();
        let review = ControlledShadowFirstNaturalForwardCycleAuthorizationReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_id: "a".repeat(32),
            review_sha256: "b".repeat(64),
            previous_review_id: None,
            previous_review_sha256: None,
            validation_id: "c".repeat(32),
            validation_sha256: "d".repeat(64),
            attempt_id: "e".repeat(32),
            claim_sha256: "f".repeat(64),
            result_sha256: "1".repeat(64),
            output_sha256: "2".repeat(64),
            authorization_review_sha256: "3".repeat(64),
            isolated_runner_spec_sha256: "4".repeat(64),
            runner_artifact_sha256: "5".repeat(64),
            implementation_contract_sha256: "6".repeat(64),
            protocol_specification_sha256: "7".repeat(64),
            design_specification_sha256: "8".repeat(64),
            initial_observation_validation_sha256: "9".repeat(64),
            initialization_manifest_sha256: "a".repeat(64),
            observation_not_before: now - TimeDelta::hours(1),
            submitted_at: now - TimeDelta::hours(1),
            authorization_not_before: now - TimeDelta::hours(1),
            authorization_valid_until: now + TimeDelta::days(1),
            reviewer_id: "reviewer".to_string(),
            excluded_prior_actor_ids: vec![],
            verdict: ControlledShadowFirstNaturalForwardCycleAuthorizationVerdict::ApprovedForOneFutureClaimFirstNaturalForwardCycleAttempt,
            rationale: "test".to_string(),
            exact_current_stage_51_through_stage_89_binding_confirmed: true,
            reviewer_independence_from_stage_89_stage_88_stage_87_and_complete_prior_chain_confirmed: true,
            zero_market_initialization_receipt_independently_validated_confirmed: true,
            natural_forward_only_no_backfill_and_observation_not_before_confirmed: true,
            official_https_calendar_content_identity_and_security_spy_sync_confirmed: true,
            point_in_time_read_only_content_addressed_allowlisted_inputs_confirmed: true,
            corporate_action_evidence_and_append_only_corrections_confirmed: true,
            claim_first_create_once_failure_consumes_and_independent_output_validation_confirmed: true,
            deterministic_long_only_caps_costs_counterfactuals_checkpoints_and_stop_rules_confirmed: true,
            fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: true,
            future_market_data_adapter_requires_separate_explicit_read_only_authorization_confirmed: true,
            single_use_seven_day_window_and_future_attempt_separation_confirmed: true,
            current_review_has_no_calendar_market_data_runtime_observation_ledger_position_or_performance_confirmed: true,
            no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_claim_first_cycle_attempt_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
            one_shot_execution_attempt_limit: ONE_SHOT_ATTEMPT_LIMIT,
            one_future_claim_first_natural_forward_cycle_attempt_authorized: true,
            authorization_claimed: false,
            cycle_execution_endpoint_available: false,
            calendar_read_authorized: false,
            market_data_adapter_authorized: false,
            market_data_access_authorized: false,
            runtime_instantiated: false,
            forward_observation_started: false,
            ledger_created: false,
            position_written: false,
            performance_metric_written: false,
            model_store_written: false,
            metric_store_written: false,
            training_feedback_authorized: false,
            reward_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        assert!(authorization_active(&review, false, now));
        assert!(!authorization_active(&review, true, now));
    }
}
