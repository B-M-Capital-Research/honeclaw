//! Independent, append-only review of one Stage 76 zero-capability controlled-shadow
//! implementation specification.
//!
//! Approval opens only a future isolated runner-specification registration gate. This module has
//! no executable artifact, entrypoint, runtime, data mount, ledger, position, order, broker or
//! trading capability and cannot start a controlled-shadow experiment.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_implementations::{
    ControlledShadowExperimentImplementationContract,
    ControlledShadowExperimentImplementationRecord,
    independently_reviewable_controlled_shadow_experiment_implementations,
    validate_controlled_shadow_experiment_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-implementation-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-implementation-independent-audit-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-experiment-implementation-independent-review-v1-no-runner-no-shadow-run";
const EXPECTED_CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-zero-capability-implementation-contract-v1";
const EXPECTED_PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-forward-replay-specification-v1-not-executable";
const EXPECTED_SEEDS: [u64; 3] = [17, 29, 43];
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowExperimentImplementationReviewVerdict {
    ApprovedForFutureIsolatedShadowRunnerSpecificationRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub design_review_sha256: String,
    pub design_registration_sha256: String,
    pub design_specification_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub design_review_hash_independently_reproduced: bool,
    pub design_registration_hash_independently_reproduced: bool,
    pub design_specification_hash_independently_reproduced: bool,
    pub exact_current_stage_51_through_stage_76_binding_valid: bool,
    pub deterministic_replay_function_and_schema_contract_valid: bool,
    pub point_in_time_universe_delisting_and_no_lookahead_contract_valid: bool,
    pub execution_cost_dividend_rebalance_and_counterfactual_contract_valid: bool,
    pub long_only_caps_cash_floor_and_instrument_boundary_valid: bool,
    pub observation_checkpoint_metric_multiple_testing_and_stop_contract_valid: bool,
    pub create_once_untrusted_output_and_no_order_payload_contract_valid: bool,
    pub all_runtime_store_feedback_shadow_order_broker_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowExperimentImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_design_review_sha256: String,
    expected_design_registration_sha256: String,
    expected_design_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: ControlledShadowExperimentImplementationReviewVerdict,
    rationale: String,
    implementation_verification_notes: String,
    risk_assessment: String,
    known_limitations: String,
    future_runner_constraints: String,
    exact_current_stage_51_through_stage_76_binding_confirmed: bool,
    reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed: bool,
    implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed:
        bool,
    pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed: bool,
    point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed: bool,
    signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed: bool,
    long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed: bool,
    observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed: bool,
    deterministic_stop_falsification_and_no_in_place_restart_confirmed: bool,
    future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed:
        bool,
    no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    no_model_metric_store_training_feedback_composite_or_reward_confirmed: bool,
    no_shadow_run_ledger_position_order_broker_or_trading_confirmed: bool,
    approval_only_opens_future_isolated_runner_specification_registration_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: ControlledShadowExperimentImplementationRecord,
    pub independent_audit: ControlledShadowExperimentImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowExperimentImplementationReviewVerdict,
    pub rationale: String,
    pub implementation_verification_notes: String,
    pub risk_assessment: String,
    pub known_limitations: String,
    pub future_runner_constraints: String,
    pub reviewer_independent_from_stage_76_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_76_binding_confirmed: bool,
    pub reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed: bool,
    pub implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed:
        bool,
    pub pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed: bool,
    pub point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed: bool,
    pub signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed: bool,
    pub long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed: bool,
    pub observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed: bool,
    pub deterministic_stop_falsification_and_no_in_place_restart_confirmed: bool,
    pub future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed:
        bool,
    pub no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    pub no_model_metric_store_training_feedback_composite_or_reward_confirmed: bool,
    pub no_shadow_run_ledger_position_order_broker_or_trading_confirmed: bool,
    pub approval_only_opens_future_isolated_runner_specification_registration_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_implementation_independently_approved: bool,
    pub future_isolated_shadow_runner_specification_registration_eligible: bool,
    pub isolated_shadow_runner_registered: bool,
    pub runner_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub production_read_authorized: bool,
    pub production_write_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_run_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentImplementationReviewItem {
    pub implementation: ControlledShadowExperimentImplementationRecord,
    pub current_independent_audit: ControlledShadowExperimentImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<ControlledShadowExperimentImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_isolated_shadow_runner_specification_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowExperimentImplementationReviewItem>,
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_isolated_shadow_runner_specification_registration_eligible_count: usize,
    pub review_status: String,
    pub isolated_shadow_runner_registered: bool,
    pub runner_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub shadow_run_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowExperimentImplementationReviewReadinessSummary {
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_isolated_shadow_runner_specification_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyReviewedControlledShadowExperimentImplementation {
    pub implementation: ControlledShadowExperimentImplementationRecord,
    pub review: ControlledShadowExperimentImplementationReviewRecord,
}

pub(crate) async fn handle_get_controlled_shadow_experiment_implementation_reviews(
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
            warn!(%error, "controlled shadow implementation independent review read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "受控影子零能力实现独立复核表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_experiment_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowExperimentImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
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

pub(crate) async fn controlled_shadow_experiment_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowExperimentImplementationReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowExperimentImplementationReviewReadinessSummary {
            implementation_count: registry.implementation_count,
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            independently_approved_count: registry.independently_approved_count,
            changes_requested_or_rejected_count: registry.changes_requested_or_rejected_count,
            future_isolated_shadow_runner_specification_registration_eligible_count: registry
                .future_isolated_shadow_runner_specification_registration_eligible_count,
            review_status: registry.review_status,
        },
    )
}

#[allow(dead_code)] // Consumed by the separately gated future Stage 78 runner registry.
pub(crate) async fn independently_reviewed_controlled_shadow_experiment_implementations_for_runner_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyReviewedControlledShadowExperimentImplementation>, String> {
    let implementations =
        independently_reviewable_controlled_shadow_experiment_implementations(state).await?;
    let mut approved = Vec::new();
    for implementation in implementations {
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let Some(review) = reviews.last().cloned() else {
            continue;
        };
        if review.implementation != implementation
            || !review.future_isolated_shadow_runner_specification_registration_eligible
        {
            continue;
        }
        validate_review(&review)?;
        approved.push(
            IndependentlyReviewedControlledShadowExperimentImplementation {
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

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowExperimentImplementationReviewRegistry, String> {
    let implementations =
        independently_reviewable_controlled_shadow_experiment_implementations(state).await?;
    let mut items = Vec::with_capacity(implementations.len());
    for implementation in implementations {
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("当前受控影子零能力实现独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|value| {
            value.implementation != implementation || value.independent_audit != audit
        }) {
            return Err("受控影子实现复核绑定或独立审计已经变化".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|value| {
            value.future_isolated_shadow_runner_specification_registration_eligible
        });
        items.push(ControlledShadowExperimentImplementationReviewItem {
            implementation,
            current_independent_audit: audit,
            complete_review_actor_ids: reviews
                .iter()
                .map(|value| value.reviewer_id.clone())
                .collect(),
            latest_review,
            review_eligible: !approved,
            future_isolated_shadow_runner_specification_registration_eligible: approved,
        });
    }
    let review_eligible_count = items.iter().filter(|value| value.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|value| value.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|value| value.future_isolated_shadow_runner_specification_registration_eligible)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|value| {
            value.latest_review.as_ref().is_some_and(|review| {
                !review.future_isolated_shadow_runner_specification_registration_eligible
            })
        })
        .count();
    let review_status = if items.is_empty() {
        "waiting_stage_76_zero_capability_shadow_implementation"
    } else if independently_approved_count > 0 {
        "zero_capability_shadow_implementation_independently_approved_only_for_future_isolated_runner_specification_registration"
    } else if review_eligible_count > 0 {
        "zero_capability_shadow_implementation_ready_for_independent_review"
    } else {
        "zero_capability_shadow_implementation_review_blocked_fail_closed"
    };
    Ok(ControlledShadowExperimentImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_count: items.len(),
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_requested_or_rejected_count,
        future_isolated_shadow_runner_specification_registration_eligible_count:
            independently_approved_count,
        items,
        review_status: review_status.to_string(),
        isolated_shadow_runner_registered: false,
        runner_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        shadow_run_authorized: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 77 阶段由 Stage 51–76 完整责任链之外的新角色独立重算实现记录、实现合同、Stage 75 复核、Stage 74 登记与设计规格指纹，并逐项复核点时输入、确定性信号/组合/成本/反事实/检查点/停止、不可信输出和全部零能力位。批准只开放未来隔离 runner 规格登记；当前没有 runner、工件、入口、runtime、挂载、生产读写、影子账本、持仓、订单、券商或交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewControlledShadowExperimentImplementationRequest,
) -> Result<ControlledShadowExperimentImplementationReviewRecord, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("受控影子实现复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementation =
        independently_reviewable_controlled_shadow_experiment_implementations(state)
            .await?
            .into_iter()
            .find(|value| value.implementation_id == implementation_id)
            .ok_or_else(|| "当前受控影子零能力实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|value| {
        value.future_isolated_shadow_runner_specification_registration_eligible
    }) {
        return Err("当前实现已经独立批准，不得追加或分叉复核".to_string());
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
    implementation: &ControlledShadowExperimentImplementationRecord,
    prior_reviews: &[ControlledShadowExperimentImplementationReviewRecord],
    prior: Option<&ControlledShadowExperimentImplementationReviewRecord>,
    reviewer_id: &str,
    request: ReviewControlledShadowExperimentImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<ControlledShadowExperimentImplementationReviewRecord, String> {
    validate_controlled_shadow_experiment_implementation_for_review(implementation)?;
    let audit = independently_audit(implementation)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("受控影子零能力实现独立审计不通过".to_string());
    }
    let contract = &implementation.implementation_contract;
    let registration = &implementation.upstream_design_registration;
    let design_review = &implementation.upstream_design_review;
    if request.expected_previous_review_id != prior.map(|value| value.review_id.clone())
        || request.expected_previous_review_sha256 != prior.map(|value| value.review_sha256.clone())
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_design_review_sha256 != design_review.review_sha256
        || request.expected_design_registration_sha256 != registration.registration_sha256
        || request.expected_design_specification_sha256
            != registration.design_specification.specification_sha256
        || request.expected_independent_audit_sha256 != audit.audit_sha256
    {
        return Err("受控影子实现、设计、审计或复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|value| value == reviewer_id)
    {
        return Err("复核人参与过 Stage 51–76 或此前复核链，必须更换独立角色".to_string());
    }
    let approved = request.verdict
        == ControlledShadowExperimentImplementationReviewVerdict::ApprovedForFutureIsolatedShadowRunnerSpecificationRegistration;
    let all_confirmed = request_checks(&request);
    if approved && !all_confirmed {
        return Err("批准前必须逐项确认独立指纹、确定性语义、角色隔离和零能力边界".to_string());
    }
    let mut review = ControlledShadowExperimentImplementationReviewRecord {
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
        rationale: bounded_required(&request.rationale, "复核理由")?,
        implementation_verification_notes: bounded_required(
            &request.implementation_verification_notes,
            "实现核验说明",
        )?,
        risk_assessment: bounded_required(&request.risk_assessment, "风险评估")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_runner_constraints: bounded_required(
            &request.future_runner_constraints,
            "未来 runner 约束",
        )?,
        reviewer_independent_from_stage_76_and_complete_prior_chain: true,
        exact_current_stage_51_through_stage_76_binding_confirmed: request
            .exact_current_stage_51_through_stage_76_binding_confirmed,
        reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed,
        implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed: request.implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed,
        pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed: request
            .pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed,
        point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed: request
            .point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed,
        signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed: request
            .signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed,
        long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed: request
            .long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed,
        observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed: request
            .observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed,
        deterministic_stop_falsification_and_no_in_place_restart_confirmed: request
            .deterministic_stop_falsification_and_no_in_place_restart_confirmed,
        future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed: request.future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: request
            .no_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_model_metric_store_training_feedback_composite_or_reward_confirmed: request
            .no_model_metric_store_training_feedback_composite_or_reward_confirmed,
        no_shadow_run_ledger_position_order_broker_or_trading_confirmed: request
            .no_shadow_run_ledger_position_order_broker_or_trading_confirmed,
        approval_only_opens_future_isolated_runner_specification_registration_confirmed: request
            .approval_only_opens_future_isolated_runner_specification_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_implementation_independently_approved: approved && all_confirmed,
        future_isolated_shadow_runner_specification_registration_eligible: approved
            && all_confirmed,
        isolated_shadow_runner_registered: false,
        runner_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        input_mount_present: false,
        production_read_authorized: false,
        production_write_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        shadow_run_authorized: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
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
    implementation: &ControlledShadowExperimentImplementationRecord,
) -> Result<ControlledShadowExperimentImplementationIndependentAudit, String> {
    let contract = &implementation.implementation_contract;
    let registration = &implementation.upstream_design_registration;
    let design_review = &implementation.upstream_design_review;
    let specification = &registration.design_specification;
    let record_hash = valid_id(&implementation.implementation_id)
        && implementation.implementation_id == implementation.implementation_sha256[..32]
        && implementation.implementation_sha256
            == fingerprint_without(
                implementation,
                &["implementation_id", "implementation_sha256"],
            )?;
    let contract_hash = valid_sha256(&contract.contract_sha256)
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?;
    let design_review_hash = valid_id(&design_review.review_id)
        && design_review.review_id == design_review.review_sha256[..32]
        && design_review.review_sha256
            == fingerprint_without(design_review, &["review_id", "review_sha256"])?;
    let registration_hash = valid_id(&registration.registration_id)
        && registration.registration_id == registration.registration_sha256[..32]
        && registration.registration_sha256
            == fingerprint_without(registration, &["registration_id", "registration_sha256"])?;
    let specification_hash = valid_sha256(&specification.specification_sha256)
        && specification.specification_sha256
            == fingerprint_without(specification, &["specification_sha256"])?;
    let binding = contract.stage_75_design_review_id == design_review.review_id
        && contract.stage_75_design_review_sha256 == design_review.review_sha256
        && contract.stage_74_design_registration_id == registration.registration_id
        && contract.stage_74_design_registration_sha256 == registration.registration_sha256
        && contract.design_specification_sha256 == specification.specification_sha256
        && contract.exact_design_specification == *specification
        && design_review.registration_id == registration.registration_id
        && design_review.registration_sha256 == registration.registration_sha256
        && design_review.design_specification_sha256 == specification.specification_sha256
        && design_review.independently_recomputed_registration_sha256
            == registration.registration_sha256
        && design_review.independently_recomputed_design_specification_sha256
            == specification.specification_sha256
        && contract.adjudication_review_sha256 == registration.adjudication_review_sha256
        && contract.output_validation_sha256 == registration.output_validation_sha256
        && contract.claim_sha256 == registration.claim_sha256
        && contract.result_sha256 == registration.result_sha256
        && contract.output_sha256 == registration.output_sha256
        && contract.envelope_sha256 == registration.envelope_sha256
        && contract.candidate_set_sha256 == registration.candidate_set_sha256
        && contract.training_store_dataset_sha256 == registration.training_store_dataset_sha256
        && contract.selected_algorithm_three_seed_binding_sha256
            == registration.selected_algorithm_three_seed_binding_sha256
        && contract.sealed_holdout_split_commitment_sha256
            == registration.sealed_holdout_split_commitment_sha256
        && contract.sealed_holdout_projection_sha256
            == registration.sealed_holdout_projection_sha256
        && contract.feature_order_sha256 == registration.feature_order_sha256
        && contract.preprocessing_sha256 == registration.preprocessing_sha256
        && contract.target_id == registration.target_id
        && contract.frozen_candidate_algorithm_id == registration.frozen_candidate_algorithm_id;
    let deterministic = contract.schema_version == EXPECTED_CONTRACT_SCHEMA_VERSION
        && contract.implementation_protocol_version == EXPECTED_PROTOCOL_VERSION
        && contract.random_seeds == EXPECTED_SEEDS
        && contract.deterministic_signal_projection_function_id
            == "controlled_shadow_signal_projection_v1_pure_specification"
        && contract.deterministic_portfolio_transition_function_id
            == "controlled_shadow_long_only_cash_constrained_transition_v1_pure_specification"
        && contract.deterministic_execution_and_cost_function_id
            == "controlled_shadow_next_session_adjusted_close_25bps_v1_pure_specification"
        && contract.deterministic_comparator_synchronization_function_id
            == "controlled_shadow_spy_cash_equal_weight_frozen_rule_sync_v1_pure_specification"
        && contract.deterministic_checkpoint_and_stop_function_id
            == "controlled_shadow_21_63_126_252_checkpoint_stop_v1_pure_specification"
        && contract.canonical_future_input_schema
            == "controlled_shadow_point_in_time_read_only_input_v1_not_mounted"
        && contract.canonical_future_untrusted_output_schema
            == "controlled_shadow_checkpoint_observation_envelope_v1_no_order_intent"
        && contract.registered_not_run
        && contract.independent_implementation_review_required
        && contract.isolated_runner_registration_required;
    let point_in_time = contract.future_inputs_read_only_and_point_in_time
        && specification.execution_contract.point_in_time_data_only
        && !specification
            .execution_contract
            .lookahead_or_retroactive_revision_allowed
        && specification.universe_contract.contains("point-in-time")
        && specification.universe_contract.contains("delistings");
    let execution = specification.benchmark_symbol == "SPY"
        && specification.comparator_ids == ["cash", "equal_weight", "frozen_rule", "spy"]
        && specification.execution_contract.slippage_bps_per_side == 25
        && specification.execution_contract.commission_bps_per_side == 0
        && specification
            .execution_contract
            .adjusted_prices_and_dividends_required
        && specification.execution_contract.rebalance_frequency == "weekly";
    let portfolio = specification.portfolio_constraints.virtual_notional_usd == 1_000_000
        && specification.portfolio_constraints.long_only
        && specification.portfolio_constraints.common_stock_only
        && !specification.portfolio_constraints.options_allowed
        && !specification.portfolio_constraints.leverage_allowed
        && !specification.portfolio_constraints.shorting_allowed
        && specification
            .portfolio_constraints
            .maximum_single_name_weight_bps
            == 500
        && specification.portfolio_constraints.maximum_theme_weight_bps == 2_000
        && specification
            .portfolio_constraints
            .maximum_gross_exposure_bps
            == 6_000
        && specification.portfolio_constraints.minimum_cash_weight_bps == 4_000
        && specification.portfolio_constraints.maximum_position_count == 10;
    let observation = specification
        .observation_contract
        .minimum_forward_market_sessions
        == 252
        && specification
            .observation_contract
            .checkpoint_market_sessions
            == [21, 63, 126, 252]
        && specification
            .observation_contract
            .minimum_independent_signal_count
            == 40
        && specification
            .observation_contract
            .minimum_distinct_symbol_count
            == 12
        && specification
            .observation_contract
            .minimum_distinct_market_quarter_count
            == 4
        && !specification.observation_contract.early_promotion_allowed
        && specification.metric_contract.metric_ids
            == [
                "absolute_return",
                "benchmark_excess_return",
                "maximum_drawdown",
                "hit_rate",
                "turnover",
                "cost_drag",
            ]
        && !specification.metric_contract.composite_score_allowed
        && specification
            .metric_contract
            .all_metrics_reported_separately
        && specification
            .metric_contract
            .multiple_testing_adjustment_required
        && specification.metric_contract.transaction_costs_included
        && specification.stop_contract.stop_rule_ids.len() == 6
        && !specification
            .stop_contract
            .automatic_trade_or_position_action_allowed
        && !specification
            .stop_contract
            .stopped_design_can_be_restarted_in_place;
    let output = contract.future_output_create_once
        && contract.future_output_untrusted
        && contract.future_output_independent_validation_required
        && !contract.future_output_contains_order_intent
        && !contract.future_output_contains_broker_payload;
    let closed = authority_closed(contract, implementation);
    let checks = [
        ("implementation_record_hash", record_hash),
        ("implementation_contract_hash", contract_hash),
        ("design_review_hash", design_review_hash),
        ("design_registration_hash", registration_hash),
        ("design_specification_hash", specification_hash),
        ("stage_51_stage_76_binding", binding),
        ("deterministic_replay", deterministic),
        ("point_in_time_no_lookahead", point_in_time),
        ("execution_and_counterfactuals", execution),
        ("portfolio_boundaries", portfolio),
        ("observation_metrics_and_stop", observation),
        ("untrusted_output", output),
        ("zero_authority", closed),
    ];
    let mismatch_reasons = checks
        .into_iter()
        .filter_map(|(name, ok)| (!ok).then_some(name.to_string()))
        .collect::<Vec<_>>();
    let mut audit = ControlledShadowExperimentImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        design_review_sha256: design_review.review_sha256.clone(),
        design_registration_sha256: registration.registration_sha256.clone(),
        design_specification_sha256: specification.specification_sha256.clone(),
        implementation_record_hash_independently_reproduced: record_hash,
        implementation_contract_hash_independently_reproduced: contract_hash,
        design_review_hash_independently_reproduced: design_review_hash,
        design_registration_hash_independently_reproduced: registration_hash,
        design_specification_hash_independently_reproduced: specification_hash,
        exact_current_stage_51_through_stage_76_binding_valid: binding,
        deterministic_replay_function_and_schema_contract_valid: deterministic,
        point_in_time_universe_delisting_and_no_lookahead_contract_valid: point_in_time,
        execution_cost_dividend_rebalance_and_counterfactual_contract_valid: execution,
        long_only_caps_cash_floor_and_instrument_boundary_valid: portfolio,
        observation_checkpoint_metric_multiple_testing_and_stop_contract_valid: observation,
        create_once_untrusted_output_and_no_order_payload_contract_valid: output,
        all_runtime_store_feedback_shadow_order_broker_and_trading_authority_closed: closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn authority_closed(
    contract: &ControlledShadowExperimentImplementationContract,
    implementation: &ControlledShadowExperimentImplementationRecord,
) -> bool {
    !contract.callable_entrypoint_present
        && !contract.executable_artifact_present
        && !contract.runtime_present
        && !contract.input_mount_present
        && !contract.market_data_adapter_present
        && !contract.environment_inheritance_allowed
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.model_store_write_allowed
        && !contract.metric_store_write_allowed
        && !contract.training_feedback_allowed
        && !contract.scalar_reward_defined
        && !contract.shadow_run_allowed
        && !contract.shadow_ledger_creation_allowed
        && !contract.shadow_position_write_allowed
        && !contract.order_generation_allowed
        && !contract.broker_access_allowed
        && !contract.trading_allowed
        && !implementation.official_model_selection_authorized
        && !implementation.model_artifact_store_written
        && !implementation.metric_store_written
        && !implementation.training_feedback_authorized
        && !implementation.reward_authorized
        && !implementation.shadow_run_authorized
        && !implementation.shadow_ledger_enabled
        && !implementation.shadow_position_written
        && !implementation.order_generation_authorized
        && !implementation.broker_access_authorized
        && !implementation.trading_authorized
}

fn request_checks(request: &ReviewControlledShadowExperimentImplementationRequest) -> bool {
    request.exact_current_stage_51_through_stage_76_binding_confirmed
        && request.reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed
        && request.implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed
        && request.pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed
        && request.point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed
        && request.signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed
        && request.long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed
        && request.observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed
        && request.deterministic_stop_falsification_and_no_in_place_restart_confirmed
        && request.future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed
        && request.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && request.no_model_metric_store_training_feedback_composite_or_reward_confirmed
        && request.no_shadow_run_ledger_position_order_broker_or_trading_confirmed
        && request.approval_only_opens_future_isolated_runner_specification_registration_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn review_checks(review: &ControlledShadowExperimentImplementationReviewRecord) -> bool {
    review.exact_current_stage_51_through_stage_76_binding_confirmed
        && review.reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed
        && review.implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed
        && review.pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed
        && review.point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed
        && review.signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed
        && review.long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed
        && review.observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed
        && review.deterministic_stop_falsification_and_no_in_place_restart_confirmed
        && review.future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed
        && review.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && review.no_model_metric_store_training_feedback_composite_or_reward_confirmed
        && review.no_shadow_run_ledger_position_order_broker_or_trading_confirmed
        && review.approval_only_opens_future_isolated_runner_specification_registration_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn expected_excluded_actor_ids(
    implementation: &ControlledShadowExperimentImplementationRecord,
    prior_reviews: &[ControlledShadowExperimentImplementationReviewRecord],
) -> Vec<String> {
    let mut values = implementation.excluded_prior_actor_ids.clone();
    values.push(implementation.registered_by.clone());
    values.push(implementation.upstream_design_review.reviewer_id.clone());
    values.push(
        implementation
            .upstream_design_registration
            .registered_by
            .clone(),
    );
    values.extend(prior_reviews.iter().map(|value| value.reviewer_id.clone()));
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

pub(crate) fn validate_review(
    review: &ControlledShadowExperimentImplementationReviewRecord,
) -> Result<(), String> {
    validate_controlled_shadow_experiment_implementation_for_review(&review.implementation)?;
    let expected_audit = independently_audit(&review.implementation)?;
    let approved = review.verdict
        == ControlledShadowExperimentImplementationReviewVerdict::ApprovedForFutureIsolatedShadowRunnerSpecificationRegistration;
    let closed = !review.isolated_shadow_runner_registered
        && !review.runner_artifact_present
        && !review.callable_entrypoint_present
        && !review.runtime_present
        && !review.input_mount_present
        && !review.production_read_authorized
        && !review.production_write_authorized
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
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
        || review.review_sha256 != fingerprint_without(review, &["review_id", "review_sha256"])?
        || review.independent_audit != expected_audit
        || !review.independent_audit.mismatch_reasons.is_empty()
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || review.reviewer_id.trim().is_empty()
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &review.reviewer_id)
        || !review.reviewer_independent_from_stage_76_and_complete_prior_chain
        || review.rationale.trim().is_empty()
        || review.implementation_verification_notes.trim().is_empty()
        || review.risk_assessment.trim().is_empty()
        || review.known_limitations.trim().is_empty()
        || review.future_runner_constraints.trim().is_empty()
        || (approved && !review_checks(review))
        || review.zero_capability_implementation_independently_approved
            != (approved && review_checks(review))
        || review.future_isolated_shadow_runner_specification_registration_eligible
            != (approved && review_checks(review))
        || !closed
    {
        return Err(
            "controlled shadow implementation review is invalid or over-authorized".to_string(),
        );
    }
    if review.previous_review_id.is_some() != review.previous_review_sha256.is_some() {
        return Err("controlled shadow implementation review predecessor is invalid".to_string());
    }
    Ok(())
}

fn validate_review_chain(
    reviews: &[ControlledShadowExperimentImplementationReviewRecord],
    implementation_id: &str,
) -> Result<Vec<ControlledShadowExperimentImplementationReviewRecord>, String> {
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
                "controlled shadow implementation review replay or subject mismatch".to_string(),
            );
        }
        match &review.previous_review_id {
            Some(parent) => {
                if child_by_parent
                    .insert(parent.clone(), review.review_id.clone())
                    .is_some()
                {
                    return Err(
                        "controlled shadow implementation review chain branches".to_string()
                    );
                }
            }
            None => roots.push(review.review_id.clone()),
        }
    }
    if roots.len() != 1 {
        return Err("controlled shadow implementation review chain has no single root".to_string());
    }
    let mut ordered = Vec::with_capacity(reviews.len());
    let mut current = roots.pop().unwrap();
    let mut seen = BTreeSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err(
                "controlled shadow implementation review chain contains a cycle".to_string(),
            );
        }
        let review = by_id
            .get(&current)
            .ok_or_else(|| {
                "controlled shadow implementation review chain is disconnected".to_string()
            })?
            .clone();
        if let Some(parent_id) = &review.previous_review_id {
            let parent = by_id.get(parent_id).ok_or_else(|| {
                "controlled shadow implementation review predecessor is missing".to_string()
            })?;
            if review.previous_review_sha256.as_deref() != Some(parent.review_sha256.as_str()) {
                return Err("controlled shadow implementation predecessor hash drift".to_string());
            }
        }
        if review.excluded_prior_actor_ids
            != expected_excluded_actor_ids(&review.implementation, &ordered)
        {
            return Err(
                "controlled shadow implementation role-exclusion history mismatch".to_string(),
            );
        }
        if review.future_isolated_shadow_runner_specification_registration_eligible
            && child_by_parent.contains_key(&current)
        {
            return Err(
                "approved controlled shadow implementation review must be terminal".to_string(),
            );
        }
        ordered.push(review);
        match child_by_parent.get(&current) {
            Some(next) => current = next.clone(),
            None => break,
        }
    }
    if ordered.len() != reviews.len() {
        return Err("controlled shadow implementation review chain is disconnected".to_string());
    }
    Ok(ordered)
}

async fn read_reviews(
    state: &AppState,
    implementation_id: &str,
) -> Result<Vec<ControlledShadowExperimentImplementationReviewRecord>, String> {
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
        let review: ControlledShadowExperimentImplementationReviewRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("controlled shadow implementation review path mismatch".to_string());
        }
        reviews.push(review);
    }
    validate_review_chain(&reviews, implementation_id)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-experiment-implementation-reviews")
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
        "controlled-shadow-experiment-implementation-review-{implementation_id}.lock"
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
        .map_err(|_| "已有受控影子实现独立复核正在进行".to_string())?;
    Ok(ReviewLock { path })
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

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "受控影子实现独立复核指纹载荷无效".to_string())?;
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

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use super::super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_implementations::tests::record;

    fn request(
        implementation: &ControlledShadowExperimentImplementationRecord,
    ) -> ReviewControlledShadowExperimentImplementationRequest {
        let audit = independently_audit(implementation).unwrap();
        ReviewControlledShadowExperimentImplementationRequest {
            expected_previous_review_id: None,
            expected_previous_review_sha256: None,
            expected_implementation_sha256: implementation.implementation_sha256.clone(),
            expected_implementation_contract_sha256: implementation
                .implementation_contract
                .contract_sha256
                .clone(),
            expected_design_review_sha256: implementation
                .upstream_design_review
                .review_sha256
                .clone(),
            expected_design_registration_sha256: implementation
                .upstream_design_registration
                .registration_sha256
                .clone(),
            expected_design_specification_sha256: implementation
                .upstream_design_registration
                .design_specification
                .specification_sha256
                .clone(),
            expected_independent_audit_sha256: audit.audit_sha256,
            verdict: ControlledShadowExperimentImplementationReviewVerdict::ApprovedForFutureIsolatedShadowRunnerSpecificationRegistration,
            rationale: "independent hashes and semantics reproduced".to_string(),
            implementation_verification_notes: "pure specification only".to_string(),
            risk_assessment: "runner and execution remain separately gated".to_string(),
            known_limitations: "no real forward run".to_string(),
            future_runner_constraints: "content-addressed zero-entry runner specification only".to_string(),
            exact_current_stage_51_through_stage_76_binding_confirmed: true,
            reviewer_independent_from_stage_76_and_complete_prior_chain_confirmed: true,
            implementation_contract_design_review_registration_and_spec_hashes_independently_reproduced_confirmed: true,
            pure_specification_no_executable_artifact_entrypoint_or_runtime_confirmed: true,
            point_in_time_universe_delisting_and_no_lookahead_semantics_confirmed: true,
            signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_confirmed: true,
            long_only_caps_cash_floor_no_options_leverage_or_shorting_confirmed: true,
            observation_sample_checkpoint_separate_metrics_and_multiple_testing_confirmed: true,
            deterministic_stop_falsification_and_no_in_place_restart_confirmed: true,
            future_input_read_only_output_create_once_untrusted_validated_and_no_order_payload_confirmed: true,
            no_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_model_metric_store_training_feedback_composite_or_reward_confirmed: true,
            no_shadow_run_ledger_position_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_isolated_runner_specification_registration_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    pub(crate) fn approved_review() -> ControlledShadowExperimentImplementationReviewRecord {
        let implementation = record();
        build_review(
            &implementation,
            &[],
            None,
            "stage-77-reviewer",
            request(&implementation),
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn independent_audit_reproduces_all_five_hash_layers() {
        let audit = independently_audit(&record()).unwrap();
        assert!(audit.implementation_record_hash_independently_reproduced);
        assert!(audit.implementation_contract_hash_independently_reproduced);
        assert!(audit.design_review_hash_independently_reproduced);
        assert!(audit.design_registration_hash_independently_reproduced);
        assert!(audit.design_specification_hash_independently_reproduced);
        assert!(audit.mismatch_reasons.is_empty());
    }

    #[test]
    fn independent_audit_confirms_deterministic_semantics_and_zero_authority() {
        let audit = independently_audit(&record()).unwrap();
        assert!(audit.deterministic_replay_function_and_schema_contract_valid);
        assert!(audit.point_in_time_universe_delisting_and_no_lookahead_contract_valid);
        assert!(audit.execution_cost_dividend_rebalance_and_counterfactual_contract_valid);
        assert!(audit.long_only_caps_cash_floor_and_instrument_boundary_valid);
        assert!(audit.observation_checkpoint_metric_multiple_testing_and_stop_contract_valid);
        assert!(audit.create_once_untrusted_output_and_no_order_payload_contract_valid);
        assert!(audit.all_runtime_store_feedback_shadow_order_broker_and_trading_authority_closed);
    }

    #[test]
    fn approval_requires_every_confirmation() {
        let implementation = record();
        let mut value = request(&implementation);
        value.no_shadow_run_ledger_position_order_broker_or_trading_confirmed = false;
        assert!(
            build_review(
                &implementation,
                &[],
                None,
                "stage-77-reviewer",
                value,
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn expected_binding_drift_fails_closed() {
        let implementation = record();
        let mut value = request(&implementation);
        value.expected_design_specification_sha256 = "f".repeat(64);
        assert!(
            build_review(
                &implementation,
                &[],
                None,
                "stage-77-reviewer",
                value,
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn stage_76_registrar_cannot_review_own_implementation() {
        let implementation = record();
        assert!(
            build_review(
                &implementation,
                &[],
                None,
                &implementation.registered_by,
                request(&implementation),
                Utc::now(),
            )
            .is_err()
        );
    }

    #[test]
    fn changes_request_can_be_followed_only_by_a_new_independent_reviewer() {
        let implementation = record();
        let mut first_request = request(&implementation);
        first_request.verdict =
            ControlledShadowExperimentImplementationReviewVerdict::ChangesRequested;
        let first = build_review(
            &implementation,
            &[],
            None,
            "stage-77-reviewer-a",
            first_request,
            Utc::now(),
        )
        .unwrap();
        let mut second_request = request(&implementation);
        second_request.expected_previous_review_id = Some(first.review_id.clone());
        second_request.expected_previous_review_sha256 = Some(first.review_sha256.clone());
        assert!(
            build_review(
                &implementation,
                std::slice::from_ref(&first),
                Some(&first),
                "stage-77-reviewer-a",
                second_request,
                Utc::now(),
            )
            .is_err()
        );
        let mut final_request = request(&implementation);
        final_request.expected_previous_review_id = Some(first.review_id.clone());
        final_request.expected_previous_review_sha256 = Some(first.review_sha256.clone());
        let approved = build_review(
            &implementation,
            std::slice::from_ref(&first),
            Some(&first),
            "stage-77-reviewer-b",
            final_request,
            Utc::now(),
        )
        .unwrap();
        validate_review_chain(&[first, approved], &implementation.implementation_id).unwrap();
    }

    #[test]
    fn approved_review_opens_only_future_runner_specification_registration() {
        let value = approved_review();
        assert!(value.zero_capability_implementation_independently_approved);
        assert!(value.future_isolated_shadow_runner_specification_registration_eligible);
        assert!(!value.isolated_shadow_runner_registered);
        assert!(!value.runner_artifact_present);
        assert!(!value.callable_entrypoint_present);
        assert!(!value.runtime_present);
        assert!(!value.shadow_run_authorized);
        assert!(!value.shadow_ledger_enabled);
        assert!(!value.shadow_position_written);
        assert!(!value.order_generation_authorized);
        assert!(!value.broker_access_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn implementation_or_review_tampering_is_rejected() {
        let mut implementation = record();
        implementation.implementation_contract.shadow_run_allowed = true;
        assert!(
            !independently_audit(&implementation)
                .unwrap()
                .mismatch_reasons
                .is_empty()
        );
        let mut review = approved_review();
        review.runner_artifact_present = true;
        review.review_sha256 =
            fingerprint_without(&review, &["review_id", "review_sha256"]).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review).is_err());
    }

    #[test]
    fn review_text_is_bounded() {
        assert!(bounded_required("", "说明").is_err());
        assert!(bounded_required(&"x".repeat(MAX_TEXT_CHARS + 1), "说明").is_err());
        assert_eq!(bounded_required(" ok ", "说明").unwrap(), "ok");
    }
}
