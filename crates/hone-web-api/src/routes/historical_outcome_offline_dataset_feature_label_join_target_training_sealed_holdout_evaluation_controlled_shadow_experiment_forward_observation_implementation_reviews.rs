//! Stage 85 chain-external review of a Stage 84 zero-capability forward-observation
//! implementation specification. Approval opens only a future isolated runner-specification
//! registration gate; it cannot observe, write a ledger or position, calculate performance,
//! generate an order, access a broker, or trade.

use std::collections::BTreeSet;
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_implementations::{
    ControlledShadowForwardObservationImplementationRecord,
    independently_reviewable_controlled_shadow_forward_observation_implementations,
    validate_controlled_shadow_forward_observation_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-implementation-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-implementation-independent-audit-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-forward-observation-implementation-chain-external-review-v1-no-observation";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-zero-capability-implementation-contract-v1";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-natural-forward-observation-implementation-v1-not-executable";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowForwardObservationImplementationReviewVerdict {
    ApprovedForFutureIsolatedForwardObservationRunnerSpecificationRegistration,
    ChangesRequiredRebuildForwardObservationImplementation,
    RejectedForwardObservationImplementation,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowForwardObservationImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_protocol_review_sha256: String,
    expected_protocol_registration_sha256: String,
    expected_protocol_specification_sha256: String,
    expected_design_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: ControlledShadowForwardObservationImplementationReviewVerdict,
    rationale: String,
    binding_and_recomputation_assessment: String,
    deterministic_semantics_assessment: String,
    zero_capability_assessment: String,
    known_limitations: String,
    future_runner_constraints: String,
    exact_current_stage_51_through_stage_84_binding_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed:
        bool,
    natural_forward_no_backfill_and_observation_not_before_confirmed: bool,
    weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed: bool,
    signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed: bool,
    future_schema_names_uninstantiated_confirmed: bool,
    no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed:
        bool,
    no_production_read_write_observation_ledger_position_or_performance_write_confirmed: bool,
    no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    approval_only_opens_future_isolated_runner_specification_registration_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowForwardObservationImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub protocol_review_sha256: String,
    pub protocol_registration_sha256: String,
    pub protocol_specification_sha256: String,
    pub design_specification_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub protocol_review_hash_independently_reproduced: bool,
    pub protocol_registration_hash_independently_reproduced: bool,
    pub protocol_specification_hash_independently_reproduced: bool,
    pub design_specification_hash_independently_reproduced: bool,
    pub exact_current_stage_51_through_stage_84_binding_valid: bool,
    pub deterministic_function_and_future_schema_contract_valid: bool,
    pub natural_forward_claim_calendar_custody_and_correction_contract_valid: bool,
    pub all_runtime_observation_store_feedback_order_broker_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: ControlledShadowForwardObservationImplementationRecord,
    pub independent_audit: ControlledShadowForwardObservationImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowForwardObservationImplementationReviewVerdict,
    pub rationale: String,
    pub binding_and_recomputation_assessment: String,
    pub deterministic_semantics_assessment: String,
    pub zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_runner_constraints: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_84_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed:
        bool,
    pub natural_forward_no_backfill_and_observation_not_before_confirmed: bool,
    pub weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed: bool,
    pub signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed: bool,
    pub future_schema_names_uninstantiated_confirmed: bool,
    pub no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed:
        bool,
    pub no_production_read_write_observation_ledger_position_or_performance_write_confirmed: bool,
    pub no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    pub approval_only_opens_future_isolated_runner_specification_registration_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_implementation_independently_approved: bool,
    pub future_isolated_forward_observation_runner_specification_registration_eligible: bool,
    pub isolated_runner_registered: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
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
pub(crate) struct ControlledShadowForwardObservationImplementationReviewItem {
    pub implementation: ControlledShadowForwardObservationImplementationRecord,
    pub current_independent_audit: ControlledShadowForwardObservationImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<ControlledShadowForwardObservationImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_isolated_forward_observation_runner_specification_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowForwardObservationImplementationReviewItem>,
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_isolated_forward_observation_runner_specification_registration_eligible_count: usize,
    pub review_status: String,
    pub isolated_runner_registered: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub forward_observation_authorized: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowForwardObservationImplementationReviewReadinessSummary {
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_isolated_forward_observation_runner_specification_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyApprovedControlledShadowForwardObservationImplementation {
    pub implementation: ControlledShadowForwardObservationImplementationRecord,
    pub review: ControlledShadowForwardObservationImplementationReviewRecord,
}

pub(crate) async fn handle_get_controlled_shadow_forward_observation_implementation_reviews(
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
            warn!(%error, "forward-observation implementation review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "前向观察实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_forward_observation_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowForwardObservationImplementationRequest>,
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

pub(crate) async fn controlled_shadow_forward_observation_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationImplementationReviewReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(ControlledShadowForwardObservationImplementationReviewReadinessSummary {
        implementation_count: value.implementation_count,
        review_eligible_count: value.review_eligible_count,
        reviewed_count: value.reviewed_count,
        independently_approved_count: value.independently_approved_count,
        changes_required_or_rejected_count: value.changes_required_or_rejected_count,
        future_isolated_forward_observation_runner_specification_registration_eligible_count: value.future_isolated_forward_observation_runner_specification_registration_eligible_count,
        review_status: value.review_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn independently_approved_controlled_shadow_forward_observation_implementations_for_isolated_runner_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyApprovedControlledShadowForwardObservationImplementation>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let review = item.latest_review?;
            item.future_isolated_forward_observation_runner_specification_registration_eligible
                .then_some(
                    IndependentlyApprovedControlledShadowForwardObservationImplementation {
                        implementation: item.implementation,
                        review,
                    },
                )
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationImplementationReviewRegistry, String> {
    let implementations =
        independently_reviewable_controlled_shadow_forward_observation_implementations(state)
            .await?;
    let mut items = Vec::new();
    for implementation in implementations {
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 84 实现独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|value| {
            value.implementation != implementation || value.independent_audit != audit
        }) {
            return Err("Stage 85 复核绑定或独立审计已经变化".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|value| {
            value.future_isolated_forward_observation_runner_specification_registration_eligible
        });
        items.push(ControlledShadowForwardObservationImplementationReviewItem {
            implementation,
            current_independent_audit: audit,
            complete_review_actor_ids: reviews
                .iter()
                .map(|value| value.reviewer_id.clone())
                .collect(),
            latest_review,
            review_eligible: !approved,
            future_isolated_forward_observation_runner_specification_registration_eligible:
                approved,
        });
    }
    items.sort_by(|left, right| {
        right
            .implementation
            .registered_at
            .cmp(&left.implementation.registered_at)
    });
    let implementation_count = items.len();
    let review_eligible_count = items.iter().filter(|value| value.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|value| value.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|value| {
            value.future_isolated_forward_observation_runner_specification_registration_eligible
        })
        .count();
    let changes_required_or_rejected_count = items.iter().filter(|value| value.latest_review.as_ref().is_some_and(|review| !review.future_isolated_forward_observation_runner_specification_registration_eligible)).count();
    let review_status = if items.is_empty() {
        "waiting_stage_84_zero_capability_forward_observation_implementation"
    } else if independently_approved_count > 0 {
        "forward_observation_implementation_independently_approved_only_for_future_isolated_runner_specification_registration"
    } else if review_eligible_count > 0 {
        "forward_observation_implementation_ready_for_chain_external_review"
    } else {
        "forward_observation_implementation_review_blocked_fail_closed"
    };
    Ok(ControlledShadowForwardObservationImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(), items,
        implementation_count, review_eligible_count, reviewed_count, independently_approved_count,
        changes_required_or_rejected_count,
        future_isolated_forward_observation_runner_specification_registration_eligible_count: independently_approved_count,
        review_status: review_status.to_string(), isolated_runner_registered: false,
        executable_artifact_present: false, callable_entrypoint_present: false, runtime_present: false,
        forward_observation_authorized: false, forward_observation_ledger_created: false,
        shadow_position_written: false, performance_metric_written: false,
        order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        scope: "Stage 85 由 Stage 51–84 完整责任链之外的新角色独立重算 Stage 84 实现/合同、Stage 83 复核、Stage 82 登记/协议和 Stage 74 设计指纹，并审计八个纯函数标识、三个未来 schema 名称和全部零权限位。批准只开放未来隔离 runner 规格登记；不开始观察、不建账、不写持仓/绩效、不训练、不奖励、不下单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewControlledShadowForwardObservationImplementationRequest,
) -> Result<ControlledShadowForwardObservationImplementationReviewRecord, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("Stage 85 复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementation =
        independently_reviewable_controlled_shadow_forward_observation_implementations(state)
            .await?
            .into_iter()
            .find(|value| value.implementation_id == implementation_id)
            .ok_or_else(|| "当前 Stage 84 实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|value| {
        value.future_isolated_forward_observation_runner_specification_registration_eligible
    }) {
        return Err("当前 Stage 84 实现已经独立批准，不得追加或分叉复核".to_string());
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
    implementation: &ControlledShadowForwardObservationImplementationRecord,
    prior_reviews: &[ControlledShadowForwardObservationImplementationReviewRecord],
    prior: Option<&ControlledShadowForwardObservationImplementationReviewRecord>,
    reviewer_id: &str,
    request: ReviewControlledShadowForwardObservationImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<ControlledShadowForwardObservationImplementationReviewRecord, String> {
    validate_controlled_shadow_forward_observation_implementation_for_review(implementation)?;
    let audit = independently_audit(implementation)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("Stage 84 实现独立审计不通过".to_string());
    }
    let contract = &implementation.implementation_contract;
    let registration = &implementation.upstream_protocol_registration;
    let protocol_review = &implementation.upstream_protocol_review;
    let protocol = &registration.protocol_specification;
    if request.expected_previous_review_id != prior.map(|value| value.review_id.clone())
        || request.expected_previous_review_sha256 != prior.map(|value| value.review_sha256.clone())
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_protocol_review_sha256 != protocol_review.review_sha256
        || request.expected_protocol_registration_sha256
            != registration.protocol_registration_sha256
        || request.expected_protocol_specification_sha256 != protocol.specification_sha256
        || request.expected_design_specification_sha256
            != protocol.exact_design_specification.specification_sha256
        || request.expected_independent_audit_sha256 != audit.audit_sha256
    {
        return Err("Stage 84 实现、上游协议、设计、审计或复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|value| value == reviewer_id)
    {
        return Err("复核人参与过 Stage 51–84 或此前复核链，必须更换责任链外角色".to_string());
    }
    let approved = request.verdict == ControlledShadowForwardObservationImplementationReviewVerdict::ApprovedForFutureIsolatedForwardObservationRunnerSpecificationRegistration;
    let confirmed = request_checks(&request);
    if approved && !confirmed {
        return Err("批准前必须逐项确认独立重算、自然前向、确定性语义和零权限边界".to_string());
    }
    let mut review = ControlledShadowForwardObservationImplementationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(), review_sha256: String::new(),
        previous_review_id: prior.map(|value| value.review_id.clone()),
        previous_review_sha256: prior.map(|value| value.review_sha256.clone()),
        implementation: implementation.clone(), independent_audit: audit, submitted_at,
        reviewer_id: reviewer_id.to_string(), excluded_prior_actor_ids, verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        binding_and_recomputation_assessment: bounded_required(&request.binding_and_recomputation_assessment, "绑定与重算评估")?,
        deterministic_semantics_assessment: bounded_required(&request.deterministic_semantics_assessment, "确定性语义评估")?,
        zero_capability_assessment: bounded_required(&request.zero_capability_assessment, "零能力评估")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_runner_constraints: bounded_required(&request.future_runner_constraints, "未来 runner 约束")?,
        reviewer_independent_from_registrar_and_complete_prior_chain: true,
        exact_current_stage_51_through_stage_84_binding_confirmed: request.exact_current_stage_51_through_stage_84_binding_confirmed,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: request.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed,
        implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed: request.implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed,
        natural_forward_no_backfill_and_observation_not_before_confirmed: request.natural_forward_no_backfill_and_observation_not_before_confirmed,
        weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed: request.weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed,
        signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed: request.signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed,
        future_schema_names_uninstantiated_confirmed: request.future_schema_names_uninstantiated_confirmed,
        no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed: request.no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed,
        no_production_read_write_observation_ledger_position_or_performance_write_confirmed: request.no_production_read_write_observation_ledger_position_or_performance_write_confirmed,
        no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: request.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_isolated_runner_specification_registration_confirmed: request.approval_only_opens_future_isolated_runner_specification_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_implementation_independently_approved: approved && confirmed,
        future_isolated_forward_observation_runner_specification_registration_eligible: approved && confirmed,
        isolated_runner_registered: false, executable_artifact_present: false, callable_entrypoint_present: false,
        runtime_present: false, input_mount_present: false, forward_observation_authorized: false,
        forward_observation_ledger_created: false, shadow_position_written: false,
        performance_metric_written: false, model_artifact_store_written: false, metric_store_written: false,
        training_feedback_authorized: false, reward_authorized: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false,
    };
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review)?;
    Ok(review)
}

fn independently_audit(
    implementation: &ControlledShadowForwardObservationImplementationRecord,
) -> Result<ControlledShadowForwardObservationImplementationIndependentAudit, String> {
    let contract = &implementation.implementation_contract;
    let review = &implementation.upstream_protocol_review;
    let registration = &implementation.upstream_protocol_registration;
    let protocol = &registration.protocol_specification;
    let design = &protocol.exact_design_specification;
    let implementation_hash = implementation.implementation_sha256
        == fingerprint_without(
            implementation,
            &["implementation_id", "implementation_sha256"],
        )?;
    let contract_hash =
        contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?;
    let review_hash =
        review.review_sha256 == fingerprint_without(review, &["review_id", "review_sha256"])?;
    let registration_hash = registration.protocol_registration_sha256
        == fingerprint_without(
            registration,
            &["protocol_registration_id", "protocol_registration_sha256"],
        )?;
    let protocol_hash =
        protocol.specification_sha256 == fingerprint_without(protocol, &["specification_sha256"])?;
    let design_hash =
        design.specification_sha256 == fingerprint_without(design, &["specification_sha256"])?;
    let binding = implementation.implementation_id == implementation.implementation_sha256[..32]
        && contract.stage_83_protocol_review_id == review.review_id
        && contract.stage_83_protocol_review_sha256 == review.review_sha256
        && contract.stage_82_protocol_registration_id == registration.protocol_registration_id
        && contract.stage_82_protocol_registration_sha256
            == registration.protocol_registration_sha256
        && contract.protocol_specification_sha256 == protocol.specification_sha256
        && contract.design_specification_sha256 == design.specification_sha256
        && contract.exact_protocol_specification == *protocol;
    let functions = [
        contract.deterministic_weekly_claim_function_id.as_str(),
        contract.deterministic_market_calendar_function_id.as_str(),
        contract
            .deterministic_point_in_time_source_custody_function_id
            .as_str(),
        contract
            .deterministic_corporate_action_correction_function_id
            .as_str(),
        contract
            .deterministic_signal_projection_function_id
            .as_str(),
        contract
            .deterministic_portfolio_transition_function_id
            .as_str(),
        contract
            .deterministic_fill_cost_and_counterfactual_function_id
            .as_str(),
        contract
            .deterministic_checkpoint_metric_and_stop_function_id
            .as_str(),
    ];
    let deterministic = contract.schema_version == CONTRACT_SCHEMA_VERSION
        && contract.implementation_protocol_version == IMPLEMENTATION_PROTOCOL_VERSION
        && functions
            .iter()
            .all(|value| value.ends_with("_pure_specification"))
        && contract
            .canonical_future_input_manifest_schema
            .ends_with("_not_mounted")
        && contract
            .canonical_future_cycle_claim_schema
            .ends_with("_not_created")
        && contract
            .canonical_future_untrusted_observation_schema
            .contains("not_created_no_order_intent")
        && contract.registered_not_run
        && contract.independent_implementation_review_required
        && contract.isolated_runner_registration_required_after_review;
    let natural = protocol.natural_forward_only
        && !protocol.retroactive_backfill_allowed
        && protocol.per_cycle_claim_first_required
        && protocol.per_cycle_create_once_required
        && protocol.point_in_time_content_addressed_allowlisted_sources_required
        && protocol
            .official_market_calendar
            .contains("官方美国市场完整交易日历")
        && protocol.benchmark_symbol == "SPY"
        && protocol.synchronized_security_and_benchmark_observation_required
        && protocol.corporate_action_evidence_required
        && (protocol.correction_policy.contains("追加")
            || protocol.correction_policy.contains("append-only"))
        && !protocol.performance_before_natural_checkpoint_allowed
        && !protocol.forward_observation_started
        && !protocol.ledger_created;
    let boundary = &contract.authority_boundary;
    let closed = !boundary.callable_entrypoint_present
        && !boundary.executable_artifact_present
        && !boundary.runtime_present
        && !boundary.input_mount_present
        && !boundary.market_data_adapter_present
        && !boundary.environment_inheritance_allowed
        && !boundary.secrets_allowed
        && !boundary.outbound_network_allowed
        && !boundary.tools_allowed
        && !boundary.subprocesses_allowed
        && !boundary.production_reads_allowed
        && !boundary.production_writes_allowed
        && !boundary.observation_claim_write_allowed
        && !boundary.forward_observation_write_allowed
        && !boundary.ledger_creation_allowed
        && !boundary.position_write_allowed
        && !boundary.performance_metric_write_allowed
        && !boundary.model_store_write_allowed
        && !boundary.metric_store_write_allowed
        && !boundary.training_feedback_allowed
        && !boundary.scalar_reward_defined
        && !boundary.order_generation_allowed
        && !boundary.broker_access_allowed
        && !boundary.trading_allowed
        && !implementation.forward_observation_authorized
        && !implementation.forward_observation_ledger_created
        && !implementation.shadow_position_written
        && !implementation.performance_metric_written
        && !implementation.model_artifact_store_written
        && !implementation.metric_store_written
        && !implementation.training_feedback_authorized
        && !implementation.reward_authorized
        && !implementation.order_generation_authorized
        && !implementation.broker_access_authorized
        && !implementation.trading_authorized;
    let checks = [
        ("implementation_record_hash", implementation_hash),
        ("implementation_contract_hash", contract_hash),
        ("protocol_review_hash", review_hash),
        ("protocol_registration_hash", registration_hash),
        ("protocol_specification_hash", protocol_hash),
        ("design_specification_hash", design_hash),
        ("stage_51_stage_84_binding", binding),
        ("deterministic_functions_and_schemas", deterministic),
        ("natural_forward_custody", natural),
        ("zero_authority", closed),
    ];
    let mismatch_reasons = checks
        .into_iter()
        .filter_map(|(name, ok)| (!ok).then_some(name.to_string()))
        .collect::<Vec<_>>();
    let mut audit = ControlledShadowForwardObservationImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        protocol_review_sha256: review.review_sha256.clone(),
        protocol_registration_sha256: registration.protocol_registration_sha256.clone(),
        protocol_specification_sha256: protocol.specification_sha256.clone(),
        design_specification_sha256: design.specification_sha256.clone(),
        implementation_record_hash_independently_reproduced: implementation_hash,
        implementation_contract_hash_independently_reproduced: contract_hash,
        protocol_review_hash_independently_reproduced: review_hash,
        protocol_registration_hash_independently_reproduced: registration_hash,
        protocol_specification_hash_independently_reproduced: protocol_hash,
        design_specification_hash_independently_reproduced: design_hash,
        exact_current_stage_51_through_stage_84_binding_valid: binding,
        deterministic_function_and_future_schema_contract_valid: deterministic,
        natural_forward_claim_calendar_custody_and_correction_contract_valid: natural,
        all_runtime_observation_store_feedback_order_broker_and_trading_authority_closed: closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

pub(crate) fn validate_controlled_shadow_forward_observation_implementation_review_for_runner_registration(
    review: &ControlledShadowForwardObservationImplementationReviewRecord,
) -> Result<(), String> {
    validate_controlled_shadow_forward_observation_implementation_for_review(
        &review.implementation,
    )?;
    let audit = independently_audit(&review.implementation)?;
    let expected_actors = expected_excluded_actor_ids(&review.implementation, &[]);
    let texts = [
        &review.rationale,
        &review.binding_and_recomputation_assessment,
        &review.deterministic_semantics_assessment,
        &review.zero_capability_assessment,
        &review.known_limitations,
        &review.future_runner_constraints,
    ];
    let approved = review.verdict == ControlledShadowForwardObservationImplementationReviewVerdict::ApprovedForFutureIsolatedForwardObservationRunnerSpecificationRegistration;
    let closed = !review.isolated_runner_registered
        && !review.executable_artifact_present
        && !review.callable_entrypoint_present
        && !review.runtime_present
        && !review.input_mount_present
        && !review.forward_observation_authorized
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
        || review.review_sha256 != fingerprint_without(review, &["review_id", "review_sha256"])?
        || review.review_id != review.review_sha256[..32]
        || review.independent_audit != audit
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || expected_actors
            .iter()
            .any(|actor| !review.excluded_prior_actor_ids.contains(actor))
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || texts
            .iter()
            .any(|value| value.trim().is_empty() || value.chars().count() > MAX_TEXT_CHARS)
        || !review.reviewer_independent_from_registrar_and_complete_prior_chain
        || !closed
        || review.zero_capability_implementation_independently_approved
            != (approved && record_checks(review))
        || review.future_isolated_forward_observation_runner_specification_registration_eligible
            != (approved && record_checks(review))
    {
        return Err("Stage 85 前向观察实现复核记录无效、漂移或越权".to_string());
    }
    Ok(())
}

fn validate_review(
    review: &ControlledShadowForwardObservationImplementationReviewRecord,
) -> Result<(), String> {
    validate_controlled_shadow_forward_observation_implementation_review_for_runner_registration(
        review,
    )
}

fn request_checks(value: &ReviewControlledShadowForwardObservationImplementationRequest) -> bool {
    value.exact_current_stage_51_through_stage_84_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed
        && value.natural_forward_no_backfill_and_observation_not_before_confirmed
        && value.weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed
        && value.signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed
        && value.future_schema_names_uninstantiated_confirmed
        && value.no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed
        && value.no_production_read_write_observation_ledger_position_or_performance_write_confirmed
        && value.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_isolated_runner_specification_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn record_checks(value: &ControlledShadowForwardObservationImplementationReviewRecord) -> bool {
    value.exact_current_stage_51_through_stage_84_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed
        && value.natural_forward_no_backfill_and_observation_not_before_confirmed
        && value.weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed
        && value.signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed
        && value.future_schema_names_uninstantiated_confirmed
        && value.no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed
        && value.no_production_read_write_observation_ledger_position_or_performance_write_confirmed
        && value.no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_isolated_runner_specification_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn expected_excluded_actor_ids(
    implementation: &ControlledShadowForwardObservationImplementationRecord,
    reviews: &[ControlledShadowForwardObservationImplementationReviewRecord],
) -> Vec<String> {
    let mut actors = implementation.excluded_prior_actor_ids.clone();
    actors.push(implementation.registered_by.clone());
    actors.extend(reviews.iter().map(|value| value.reviewer_id.clone()));
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

async fn read_reviews(
    state: &AppState,
    implementation_id: &str,
) -> Result<Vec<ControlledShadowForwardObservationImplementationReviewRecord>, String> {
    let directory = review_root(state).join(implementation_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        {
            let review: ControlledShadowForwardObservationImplementationReviewRecord =
                serde_json::from_slice(
                    &tokio::fs::read(entry.path())
                        .await
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
            validate_review(&review)?;
            reviews.push(review);
        }
    }
    reviews.sort_by(|left, right| {
        left.submitted_at
            .cmp(&right.submitted_at)
            .then_with(|| left.review_id.cmp(&right.review_id))
    });
    let mut previous_id = None;
    let mut previous_sha = None;
    let mut ids = BTreeSet::new();
    let mut reviewer_ids = BTreeSet::new();
    for (index, review) in reviews.iter().enumerate() {
        let expected_excluded_actor_ids =
            expected_excluded_actor_ids(&review.implementation, &reviews[..index]);
        if !ids.insert(review.review_id.clone())
            || !reviewer_ids.insert(review.reviewer_id.clone())
            || review.implementation.implementation_id != implementation_id
            || reviews.first().is_some_and(|root| {
                root.implementation != review.implementation
                    || root.independent_audit != review.independent_audit
            })
            || review.excluded_prior_actor_ids != expected_excluded_actor_ids
            || review.previous_review_id != previous_id
            || review.previous_review_sha256 != previous_sha
        {
            return Err("Stage 85 复核链分叉、断裂、重复、跨实现或角色排除不完整".to_string());
        }
        previous_id = Some(review.review_id.clone());
        previous_sha = Some(review.review_sha256.clone());
    }
    if reviews
        .iter()
        .take(reviews.len().saturating_sub(1))
        .any(|review| {
            review.future_isolated_forward_observation_runner_specification_registration_eligible
        })
    {
        return Err("Stage 85 批准记录必须是终止链尖".to_string());
    }
    Ok(reviews)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 85 指纹载荷无效".to_string())?;
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
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn review_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-forward-observation-implementation-reviews")
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
        "controlled-shadow-forward-observation-implementation-review-{implementation_id}.lock"
    ));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|value| SystemTime::now().duration_since(value).ok())
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
        .map_err(|_| "已有同一 Stage 85 复核正在进行".to_string())?;
    Ok(ReviewLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ReviewControlledShadowForwardObservationImplementationRequest {
        ReviewControlledShadowForwardObservationImplementationRequest {
            expected_previous_review_id: None, expected_previous_review_sha256: None,
            expected_implementation_sha256: "a".repeat(64), expected_implementation_contract_sha256: "b".repeat(64),
            expected_protocol_review_sha256: "c".repeat(64), expected_protocol_registration_sha256: "d".repeat(64),
            expected_protocol_specification_sha256: "e".repeat(64), expected_design_specification_sha256: "f".repeat(64),
            expected_independent_audit_sha256: "1".repeat(64),
            verdict: ControlledShadowForwardObservationImplementationReviewVerdict::ApprovedForFutureIsolatedForwardObservationRunnerSpecificationRegistration,
            rationale: "独立复核".to_string(), binding_and_recomputation_assessment: "全链重算".to_string(),
            deterministic_semantics_assessment: "八个纯函数".to_string(), zero_capability_assessment: "全部关闭".to_string(),
            known_limitations: "尚无自然前向样本".to_string(), future_runner_constraints: "仍须独立登记".to_string(),
            exact_current_stage_51_through_stage_84_binding_confirmed: true,
            reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
            implementation_contract_review_registration_protocol_and_design_hashes_independently_reproduced_confirmed: true,
            natural_forward_no_backfill_and_observation_not_before_confirmed: true,
            weekly_claim_calendar_point_in_time_custody_and_corrections_confirmed: true,
            signal_portfolio_fill_cost_counterfactual_checkpoint_and_stop_functions_confirmed: true,
            future_schema_names_uninstantiated_confirmed: true,
            no_artifact_entrypoint_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed: true,
            no_production_read_write_observation_ledger_position_or_performance_write_confirmed: true,
            no_model_metric_training_feedback_reward_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_isolated_runner_specification_registration_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn approval_requires_every_confirmation() {
        let mut value = request();
        assert!(request_checks(&value));
        value.future_schema_names_uninstantiated_confirmed = false;
        assert!(!request_checks(&value));
    }

    #[test]
    fn fingerprints_bind_semantics() {
        #[derive(Serialize)]
        struct Fixture {
            review_id: String,
            review_sha256: String,
            semantics: String,
        }
        let value = Fixture {
            review_id: String::new(),
            review_sha256: String::new(),
            semantics: "claim-first".to_string(),
        };
        let before = fingerprint_without(&value, &["review_id", "review_sha256"]).unwrap();
        let changed = Fixture {
            semantics: "backfill".to_string(),
            ..value
        };
        assert_ne!(
            before,
            fingerprint_without(&changed, &["review_id", "review_sha256"]).unwrap()
        );
    }

    #[test]
    fn reviewer_is_added_to_future_exclusion_set() {
        let mut actors = vec!["b".to_string(), "a".to_string(), "a".to_string()];
        actors.sort();
        actors.dedup();
        assert_eq!(actors, vec!["a", "b"]);
    }
}
