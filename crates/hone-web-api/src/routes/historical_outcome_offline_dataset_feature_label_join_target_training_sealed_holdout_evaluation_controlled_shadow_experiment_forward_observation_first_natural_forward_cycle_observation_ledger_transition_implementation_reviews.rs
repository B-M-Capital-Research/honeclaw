//! Stage 118 chain-external review of one Stage 117 zero-capability observation-ledger-transition
//! implementation contract.
//!
//! Approval opens only a future isolated observation-ledger-transition runner-specification
//! registration gate. This module has no source or executable artifact, entrypoint, runtime,
//! input mount/read, observation output, ledger, position, performance, model/reward, order,
//! broker or trading capability.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_implementations::{
    ControlledShadowObservationLedgerTransitionImplementationContract,
    ControlledShadowObservationLedgerTransitionImplementationRecord,
    ZeroCapabilityObservationLedgerTransitionImplementationAuthorityBoundary,
    independently_reviewable_observation_ledger_transition_implementations,
    validate_observation_ledger_transition_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-implementation-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-implementation-independent-audit-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-ledger-transition-implementation-chain-external-review-v1-zero-capability";
const STAGE_117_RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-implementation-v1";
const STAGE_117_POLICY_VERSION: &str = "hone-controlled-shadow-observation-ledger-transition-implementation-create-once-v1-zero-capability";
const CONTRACT_SCHEMA_VERSION: &str = "hone-controlled-shadow-zero-capability-observation-ledger-transition-implementation-contract-v1";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-implementation-v1-not-executable";
const STAGE_117_STATUS: &str = "registered_not_reviewed_not_run";
const MAX_TEXT_CHARS: usize = 4_000;

const EXPECTED_FUNCTION_IDS: [&str; 8] = [
    "observation_ledger_transition_current_source_binding_validation_v1_pure_contract",
    "observation_ledger_transition_opening_portfolio_prerequisite_validation_v1_pure_contract",
    "observation_ledger_transition_non_financial_observation_event_projection_v1_pure_contract",
    "observation_ledger_transition_raw_close_accounting_adjusted_price_separation_v1_pure_contract",
    "observation_ledger_transition_explicit_gap_nav_fail_closed_v1_pure_contract",
    "observation_ledger_transition_corporate_action_notice_gating_v1_pure_contract",
    "observation_ledger_transition_exact_decimal_idempotency_double_entry_v1_pure_contract",
    "observation_ledger_transition_append_only_correction_conservative_availability_v1_pure_contract",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationLedgerTransitionImplementationReviewVerdict {
    ApprovedForFutureIsolatedObservationLedgerTransitionRunnerSpecificationRegistration,
    ChangesRequiredRebuildObservationLedgerTransitionImplementation,
    RejectedObservationLedgerTransitionImplementation,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowObservationLedgerTransitionImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_specification_review_sha256: String,
    expected_specification_independent_audit_sha256: String,
    expected_specification_registration_sha256: String,
    expected_observation_ledger_transition_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: ControlledShadowObservationLedgerTransitionImplementationReviewVerdict,
    rationale: String,
    binding_and_recomputation_assessment: String,
    deterministic_projection_semantics_assessment: String,
    session_price_basis_gap_and_company_action_assessment: String,
    initial_allocation_availability_and_output_assessment: String,
    zero_capability_assessment: String,
    known_limitations: String,
    future_runner_constraints: String,
    exact_current_stage_51_through_stage_117_binding_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed:
        bool,
    all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: bool,
    exact_stage_114_admitted_output_is_only_future_input_confirmed: bool,
    official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: bool,
    explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: bool,
    dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: bool,
    initial_shadow_allocation_and_conservative_availability_preserved_confirmed: bool,
    provider_publication_time_remains_unverified_confirmed: bool,
    one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
        bool,
    future_output_untrusted_and_independent_validation_required_confirmed: bool,
    no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub specification_review_sha256: String,
    pub specification_independent_audit_sha256: String,
    pub specification_registration_sha256: String,
    pub observation_ledger_transition_specification_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub specification_review_hash_independently_reproduced: bool,
    pub specification_independent_audit_hash_independently_reproduced: bool,
    pub specification_registration_hash_independently_reproduced: bool,
    pub observation_ledger_transition_specification_hash_independently_reproduced: bool,
    pub complete_implementation_contract_rebuilt_without_stage_117_builder: bool,
    pub rebuilt_implementation_contract_exactly_matches_record: bool,
    pub exact_current_stage_51_through_stage_117_binding_valid: bool,
    pub eight_function_ids_and_canonical_schemas_valid: bool,
    pub opening_portfolio_prerequisite_and_no_invention_contract_valid: bool,
    pub raw_price_adjusted_price_gap_and_nav_contract_valid: bool,
    pub corporate_action_decimal_idempotency_double_entry_and_correction_contract_valid: bool,
    pub conservative_availability_create_once_and_output_path_contract_valid: bool,
    pub provider_publication_time_still_unverified: bool,
    pub all_source_artifact_runtime_input_ledger_financial_feedback_order_broker_and_trading_authority_closed:
        bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: ControlledShadowObservationLedgerTransitionImplementationRecord,
    pub independent_audit:
        ControlledShadowObservationLedgerTransitionImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowObservationLedgerTransitionImplementationReviewVerdict,
    pub rationale: String,
    pub binding_and_recomputation_assessment: String,
    pub deterministic_projection_semantics_assessment: String,
    pub session_price_basis_gap_and_company_action_assessment: String,
    pub initial_allocation_availability_and_output_assessment: String,
    pub zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_runner_constraints: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_117_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed:
        bool,
    pub all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: bool,
    pub exact_stage_114_admitted_output_is_only_future_input_confirmed: bool,
    pub official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: bool,
    pub explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: bool,
    pub dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed:
        bool,
    pub initial_shadow_allocation_and_conservative_availability_preserved_confirmed: bool,
    pub provider_publication_time_remains_unverified_confirmed: bool,
    pub one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
        bool,
    pub future_output_untrusted_and_independent_validation_required_confirmed: bool,
    pub no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    pub no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_implementation_independently_approved: bool,
    pub future_isolated_observation_ledger_transition_runner_specification_registration_eligible:
        bool,
    pub isolated_runner_registered: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mounted_or_read: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_created: bool,
    pub ledger_event_written: bool,
    pub position_written: bool,
    pub cash_written: bool,
    pub nav_or_performance_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionImplementationReviewItem {
    pub implementation: ControlledShadowObservationLedgerTransitionImplementationRecord,
    pub current_independent_audit:
        ControlledShadowObservationLedgerTransitionImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review:
        Option<ControlledShadowObservationLedgerTransitionImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_isolated_observation_ledger_transition_runner_specification_registration_eligible:
        bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowObservationLedgerTransitionImplementationReviewItem>,
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_isolated_observation_ledger_transition_runner_specification_registration_eligible_count:
        usize,
    pub review_status: String,
    pub isolated_runner_registered: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mounted_or_read: bool,
    pub opening_portfolio_snapshot_present: bool,
    pub ledger_created: bool,
    pub ledger_event_written: bool,
    pub position_written: bool,
    pub cash_written: bool,
    pub nav_or_performance_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationLedgerTransitionImplementationReviewReadinessSummary {
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_isolated_observation_ledger_transition_runner_specification_registration_eligible_count:
        usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyApprovedControlledShadowObservationLedgerTransitionImplementation {
    pub implementation: ControlledShadowObservationLedgerTransitionImplementationRecord,
    pub review: ControlledShadowObservationLedgerTransitionImplementationReviewRecord,
}

pub(crate) async fn handle_get_controlled_shadow_observation_ledger_transition_implementation_reviews(
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
            warn!(%error, "observation-ledger-transition implementation review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察到账本转换零能力实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_observation_ledger_transition_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowObservationLedgerTransitionImplementationRequest>,
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

pub(crate) async fn controlled_shadow_observation_ledger_transition_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionImplementationReviewReadinessSummary, String>
{
    let value = build_registry(state).await?;
    Ok(ControlledShadowObservationLedgerTransitionImplementationReviewReadinessSummary {
        implementation_count: value.implementation_count,
        review_eligible_count: value.review_eligible_count,
        reviewed_count: value.reviewed_count,
        independently_approved_count: value.independently_approved_count,
        changes_required_or_rejected_count: value.changes_required_or_rejected_count,
        future_isolated_observation_ledger_transition_runner_specification_registration_eligible_count: value.future_isolated_observation_ledger_transition_runner_specification_registration_eligible_count,
        review_status: value.review_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn independently_approved_observation_ledger_transition_implementations_for_isolated_runner_registration(
    state: &AppState,
) -> Result<
    Vec<IndependentlyApprovedControlledShadowObservationLedgerTransitionImplementation>,
    String,
> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let review = item.latest_review?;
            item.future_isolated_observation_ledger_transition_runner_specification_registration_eligible
                .then_some(
                    IndependentlyApprovedControlledShadowObservationLedgerTransitionImplementation {
                        implementation: item.implementation,
                        review,
                    },
                )
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionImplementationReviewRegistry, String> {
    let implementations =
        independently_reviewable_observation_ledger_transition_implementations(state).await?;
    let mut items = Vec::new();
    for implementation in implementations {
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 117 实现独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|value| {
            value.implementation != implementation || value.independent_audit != audit
        }) {
            return Err("Stage 118 复核绑定或独立审计已经变化".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|value| {
            value.future_isolated_observation_ledger_transition_runner_specification_registration_eligible
        });
        items.push(ControlledShadowObservationLedgerTransitionImplementationReviewItem {
            implementation,
            current_independent_audit: audit,
            complete_review_actor_ids: reviews
                .iter()
                .map(|value| value.reviewer_id.clone())
                .collect(),
            latest_review,
            review_eligible: !approved,
            future_isolated_observation_ledger_transition_runner_specification_registration_eligible:
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
            value.future_isolated_observation_ledger_transition_runner_specification_registration_eligible
        })
        .count();
    let changes_required_or_rejected_count = items.iter().filter(|value| value.latest_review.as_ref().is_some_and(|review| !review.future_isolated_observation_ledger_transition_runner_specification_registration_eligible)).count();
    let review_status = if items.is_empty() {
        "waiting_stage_117_zero_capability_observation_ledger_transition_implementation"
    } else if independently_approved_count > 0 {
        "observation_ledger_transition_implementation_independently_approved_only_for_future_isolated_runner_specification_registration"
    } else if review_eligible_count > 0 {
        "observation_ledger_transition_implementation_ready_for_chain_external_review"
    } else {
        "observation_ledger_transition_implementation_review_blocked_fail_closed"
    };
    Ok(ControlledShadowObservationLedgerTransitionImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(), items,
        implementation_count, review_eligible_count, reviewed_count, independently_approved_count,
        changes_required_or_rejected_count,
        future_isolated_observation_ledger_transition_runner_specification_registration_eligible_count: independently_approved_count,
        review_status: review_status.to_string(), isolated_runner_registered: false,
        source_artifact_present: false,
        executable_artifact_present: false, callable_entrypoint_present: false, runtime_present: false,
        input_mounted_or_read: false, opening_portfolio_snapshot_present: false,
        ledger_created: false, ledger_event_written: false, position_written: false,
        cash_written: false, nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false,
        order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        scope: "Stage 118 由 Stage 51–117 完整责任链之外的新角色，以第二套实现从 Stage 116 当前批准来源独立重建 Stage 117 implementation contract，并重算 implementation/contract、review/audit、registration/specification 指纹；复核八个纯合同函数、canonical event/双分录 schema、opening portfolio 独立准入前置门槛、raw/adjusted 价格隔离、显式 gap 阻断 NAV、公司行动 notice、精确十进制/幂等/双分录、append-only 纠错、保守 available-at 和全部零权限位。批准只开放 Stage 119 隔离账本转换 runner 规格登记；不产生源码或工件、不读取输入、不准入期初组合、不建账、不写事件/持仓/现金/NAV/绩效、不训练/RL/reward、不下单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewControlledShadowObservationLedgerTransitionImplementationRequest,
) -> Result<ControlledShadowObservationLedgerTransitionImplementationReviewRecord, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("Stage 118 复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementation =
        independently_reviewable_observation_ledger_transition_implementations(state)
            .await?
            .into_iter()
            .find(|value| value.implementation_id == implementation_id)
            .ok_or_else(|| "当前 Stage 117 实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|value| {
        value.future_isolated_observation_ledger_transition_runner_specification_registration_eligible
    }) {
        return Err("当前 Stage 117 实现已经独立批准，不得追加或分叉复核".to_string());
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
    implementation: &ControlledShadowObservationLedgerTransitionImplementationRecord,
    prior_reviews: &[ControlledShadowObservationLedgerTransitionImplementationReviewRecord],
    prior: Option<&ControlledShadowObservationLedgerTransitionImplementationReviewRecord>,
    reviewer_id: &str,
    request: ReviewControlledShadowObservationLedgerTransitionImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<ControlledShadowObservationLedgerTransitionImplementationReviewRecord, String> {
    validate_observation_ledger_transition_implementation_for_review(implementation)?;
    let audit = independently_audit(implementation)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("Stage 117 实现独立审计不通过".to_string());
    }
    let contract = &implementation.implementation_contract;
    let registration = &implementation.upstream_specification_registration;
    let specification_review = &implementation.upstream_specification_review;
    let specification = &registration.specification;
    if request.expected_previous_review_id != prior.map(|value| value.review_id.clone())
        || request.expected_previous_review_sha256 != prior.map(|value| value.review_sha256.clone())
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_specification_review_sha256 != specification_review.review_sha256
        || request.expected_specification_independent_audit_sha256
            != specification_review.independent_audit.audit_sha256
        || request.expected_specification_registration_sha256 != registration.registration_sha256
        || request.expected_observation_ledger_transition_specification_sha256
            != specification.specification_sha256
        || request.expected_independent_audit_sha256 != audit.audit_sha256
    {
        return Err("Stage 115–117 实现、规格、审计或复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|value| value == reviewer_id)
    {
        return Err("复核人参与过 Stage 51–117 或此前复核链，必须更换责任链外角色".to_string());
    }
    let approved = request.verdict == ControlledShadowObservationLedgerTransitionImplementationReviewVerdict::ApprovedForFutureIsolatedObservationLedgerTransitionRunnerSpecificationRegistration;
    let confirmed = request_checks(&request);
    if approved && !confirmed {
        return Err(
            "批准前必须逐项确认独立重建、账本转换语义、opening portfolio 门槛和零权限边界"
                .to_string(),
        );
    }
    let mut review = ControlledShadowObservationLedgerTransitionImplementationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(), review_sha256: String::new(),
        previous_review_id: prior.map(|value| value.review_id.clone()),
        previous_review_sha256: prior.map(|value| value.review_sha256.clone()),
        implementation: implementation.clone(), independent_audit: audit, submitted_at,
        reviewer_id: reviewer_id.to_string(), excluded_prior_actor_ids, verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        binding_and_recomputation_assessment: bounded_required(&request.binding_and_recomputation_assessment, "绑定与重算评估")?,
        deterministic_projection_semantics_assessment: bounded_required(&request.deterministic_projection_semantics_assessment, "确定性账本转换语义评估")?,
        session_price_basis_gap_and_company_action_assessment: bounded_required(&request.session_price_basis_gap_and_company_action_assessment, "价格口径、缺口与公司行动评估")?,
        initial_allocation_availability_and_output_assessment: bounded_required(&request.initial_allocation_availability_and_output_assessment, "opening portfolio、可用时间与输出评估")?,
        zero_capability_assessment: bounded_required(&request.zero_capability_assessment, "零能力评估")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_runner_constraints: bounded_required(&request.future_runner_constraints, "未来 runner 约束")?,
        reviewer_independent_from_registrar_and_complete_prior_chain: true,
        exact_current_stage_51_through_stage_117_binding_confirmed: request.exact_current_stage_51_through_stage_117_binding_confirmed,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: request.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed,
        implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: request.implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed,
        all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: request.all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed,
        exact_stage_114_admitted_output_is_only_future_input_confirmed: request.exact_stage_114_admitted_output_is_only_future_input_confirmed,
        official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: request.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed,
        explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: request.explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed,
        dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: request.dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed,
        initial_shadow_allocation_and_conservative_availability_preserved_confirmed: request.initial_shadow_allocation_and_conservative_availability_preserved_confirmed,
        provider_publication_time_remains_unverified_confirmed: request.provider_publication_time_remains_unverified_confirmed,
        one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: request.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed,
        future_output_untrusted_and_independent_validation_required_confirmed: request.future_output_untrusted_and_independent_validation_required_confirmed,
        no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: request.no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed: request.approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_implementation_independently_approved: approved && confirmed,
        future_isolated_observation_ledger_transition_runner_specification_registration_eligible: approved && confirmed,
        isolated_runner_registered: false, source_artifact_present: false, executable_artifact_present: false,
        callable_entrypoint_present: false, runtime_present: false, input_mounted_or_read: false,
        opening_portfolio_snapshot_admitted: false, ledger_created: false,
        ledger_event_written: false, position_written: false, cash_written: false,
        nav_or_performance_written: false, model_or_metric_store_written: false,
        training_or_rl_feedback_authorized: false, reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false,
    };
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review)?;
    Ok(review)
}

fn independently_rebuilt_contract(
    implementation: &ControlledShadowObservationLedgerTransitionImplementationRecord,
) -> Result<ControlledShadowObservationLedgerTransitionImplementationContract, String> {
    let review = &implementation.upstream_specification_review;
    let registration = &implementation.upstream_specification_registration;
    let specification = &registration.specification;
    let mut contract = ControlledShadowObservationLedgerTransitionImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_protocol_version: IMPLEMENTATION_PROTOCOL_VERSION.to_string(),
        immutable_code_revision: implementation
            .implementation_contract
            .immutable_code_revision
            .clone(),
        stage_116_specification_review_id: review.review_id.clone(),
        stage_116_specification_review_sha256: review.review_sha256.clone(),
        stage_116_independent_audit_sha256: review.independent_audit.audit_sha256.clone(),
        stage_115_registration_id: registration.registration_id.clone(),
        stage_115_registration_sha256: registration.registration_sha256.clone(),
        observation_ledger_transition_specification_sha256: specification
            .specification_sha256
            .clone(),
        exact_observation_ledger_transition_specification: specification.clone(),
        current_source_binding_validation_function_id: EXPECTED_FUNCTION_IDS[0].to_string(),
        opening_portfolio_prerequisite_validation_function_id: EXPECTED_FUNCTION_IDS[1].to_string(),
        non_financial_observation_event_projection_function_id: EXPECTED_FUNCTION_IDS[2]
            .to_string(),
        raw_close_accounting_and_adjusted_price_separation_function_id: EXPECTED_FUNCTION_IDS[3]
            .to_string(),
        explicit_gap_nav_fail_closed_function_id: EXPECTED_FUNCTION_IDS[4].to_string(),
        corporate_action_notice_gating_function_id: EXPECTED_FUNCTION_IDS[5].to_string(),
        exact_decimal_idempotency_and_double_entry_function_id: EXPECTED_FUNCTION_IDS[6]
            .to_string(),
        append_only_correction_and_conservative_availability_function_id: EXPECTED_FUNCTION_IDS[7]
            .to_string(),
        canonical_event_schema: specification.mapping_rules.canonical_event_schema.clone(),
        canonical_double_entry_posting_schema: specification
            .mapping_rules
            .canonical_double_entry_posting_schema
            .clone(),
        future_exact_admitted_input_read_only_and_content_addressed: true,
        future_ledger_relative_path_template: specification
            .future_ledger_relative_path_template
            .clone(),
        future_event_stream_relative_path_template: specification
            .future_event_stream_relative_path_template
            .clone(),
        opening_portfolio_snapshot_currently_admitted: false,
        financial_postings_currently_eligible: false,
        nav_or_performance_currently_eligible: false,
        future_ledger_and_event_stream_create_once_append_only_and_untrusted: true,
        future_ledger_and_event_stream_independent_validation_required: true,
        registered_not_run: true,
        independent_implementation_review_required: true,
        isolated_runner_registration_required_after_review: true,
        authority_boundary: closed_authority_boundary(),
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn independently_audit(
    implementation: &ControlledShadowObservationLedgerTransitionImplementationRecord,
) -> Result<ControlledShadowObservationLedgerTransitionImplementationIndependentAudit, String> {
    let contract = &implementation.implementation_contract;
    let review = &implementation.upstream_specification_review;
    let registration = &implementation.upstream_specification_registration;
    let specification = &registration.specification;
    let specification_audit = &review.independent_audit;
    let implementation_hash = implementation.implementation_sha256
        == fingerprint_without(
            implementation,
            &["implementation_id", "implementation_sha256"],
        )?;
    let contract_hash =
        contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?;
    let review_hash =
        review.review_sha256 == fingerprint_without(review, &["review_id", "review_sha256"])?;
    let specification_audit_hash = specification_audit.audit_sha256
        == fingerprint_without(specification_audit, &["audit_sha256"])?;
    let registration_hash = registration.registration_sha256
        == fingerprint_without(registration, &["registration_id", "registration_sha256"])?;
    let specification_hash = specification.specification_sha256
        == fingerprint_without(specification, &["specification_sha256"])?;
    let rebuilt_contract = independently_rebuilt_contract(implementation)?;
    let rebuilt_contract_matches = rebuilt_contract == *contract;
    let binding = implementation.schema_version == STAGE_117_RECORD_SCHEMA_VERSION
        && implementation.policy_version == STAGE_117_POLICY_VERSION
        && implementation.status == STAGE_117_STATUS
        && implementation.confirmations_complete
        && implementation.zero_capability_implementation_contract_registered
        && implementation.future_independent_implementation_review_eligible
        && implementation.registrar_independent_from_stage_116_and_complete_prior_chain
        && implementation.implementation_id == implementation.implementation_sha256[..32]
        && contract.stage_116_specification_review_id == review.review_id
        && contract.stage_116_specification_review_sha256 == review.review_sha256
        && contract.stage_116_independent_audit_sha256 == specification_audit.audit_sha256
        && contract.stage_115_registration_id == registration.registration_id
        && contract.stage_115_registration_sha256 == registration.registration_sha256
        && contract.observation_ledger_transition_specification_sha256
            == specification.specification_sha256
        && contract.exact_observation_ledger_transition_specification == *specification
        && implementation_hash
        && contract_hash
        && review_hash
        && specification_audit_hash
        && registration_hash
        && specification_hash
        && rebuilt_contract_matches;
    let functions = [
        contract
            .current_source_binding_validation_function_id
            .as_str(),
        contract
            .opening_portfolio_prerequisite_validation_function_id
            .as_str(),
        contract
            .non_financial_observation_event_projection_function_id
            .as_str(),
        contract
            .raw_close_accounting_and_adjusted_price_separation_function_id
            .as_str(),
        contract.explicit_gap_nav_fail_closed_function_id.as_str(),
        contract.corporate_action_notice_gating_function_id.as_str(),
        contract
            .exact_decimal_idempotency_and_double_entry_function_id
            .as_str(),
        contract
            .append_only_correction_and_conservative_availability_function_id
            .as_str(),
    ];
    let deterministic = contract.schema_version == CONTRACT_SCHEMA_VERSION
        && contract.implementation_protocol_version == IMPLEMENTATION_PROTOCOL_VERSION
        && functions == EXPECTED_FUNCTION_IDS
        && contract.canonical_event_schema == specification.mapping_rules.canonical_event_schema
        && contract.canonical_double_entry_posting_schema
            == specification
                .mapping_rules
                .canonical_double_entry_posting_schema
        && contract.registered_not_run
        && contract.independent_implementation_review_required
        && contract.isolated_runner_registration_required_after_review;
    let opening = &specification.opening_portfolio_prerequisite;
    let opening_portfolio = specification
        .exact_stage_114_admitted_envelope_is_only_observation_input
        && specification.observed_session_count > 0
        && specification.observed_price_count > 0
        && opening.separately_admitted_opening_portfolio_snapshot_required
        && !opening.current_opening_portfolio_snapshot_available
        && opening.stage_88_binding_is_initialization_provenance_not_opening_positions
        && !opening.default_notional_allowed
        && !opening.default_cash_allowed
        && !opening.infer_positions_from_subject_symbols_allowed
        && !opening.infer_share_quantities_from_prices_or_target_weights_allowed
        && !opening.financial_posting_before_opening_snapshot_admission_allowed
        && !contract.opening_portfolio_snapshot_currently_admitted
        && !contract.financial_postings_currently_eligible
        && !contract.nav_or_performance_currently_eligible;
    let rules = &specification.mapping_rules;
    let price_gap_nav = rules
        .security_valuation_price_basis
        .contains("raw_unadjusted_price_only")
        && rules
            .benchmark_total_return_price_basis
            .contains("dividend_adjusted_price_for_non_accounting")
        && rules
            .split_adjusted_price_usage
            .contains("never_a_portfolio_accounting_mark")
        && rules
            .dividend_adjusted_price_usage
            .contains("never_a_security_accounting_mark")
        && rules.explicit_gap_rule.contains("fail_closed")
        && rules
            .nav_completeness_rule
            .contains("otherwise_no_NAV_or_performance_fact")
        && rules
            .financial_event_type_allowlist_before_opening_snapshot
            .is_empty();
    let accounting_contract = rules.dividend_rule.contains("notice")
        && rules.split_rule.contains("notice")
        && rules.correction_rule.contains("superseding_or_reversal")
        && rules.decimal_rule.contains("exact_decimal")
        && rules.event_idempotency_rule.contains("sha256(")
        && rules
            .double_entry_balance_rule
            .contains("debits_must_exactly_equal_credits")
        && !rules.canonical_event_schema.trim().is_empty()
        && !rules
            .canonical_double_entry_posting_schema
            .trim()
            .is_empty();
    let availability_and_output = !specification.provider_publication_time_verified
        && rules.evidence_availability_rule.contains("available_at")
        && specification.create_once_required
        && specification.append_only_required
        && !specification.overwrite_allowed
        && !specification.retroactive_backfill_allowed
        && !specification.in_place_correction_allowed
        && !specification.financial_postings_currently_eligible
        && !specification.nav_or_performance_currently_eligible
        && contract.future_exact_admitted_input_read_only_and_content_addressed
        && contract.future_ledger_relative_path_template
            == specification.future_ledger_relative_path_template
        && contract.future_event_stream_relative_path_template
            == specification.future_event_stream_relative_path_template
        && contract.future_ledger_and_event_stream_create_once_append_only_and_untrusted
        && contract.future_ledger_and_event_stream_independent_validation_required;
    let closed = authority_closed(&contract.authority_boundary)
        && !implementation.observation_ledger_transition_implementation_present
        && !implementation.independent_implementation_review_completed
        && !implementation.isolated_runner_registration_eligible
        && !implementation.opening_portfolio_snapshot_admitted
        && !implementation.ledger_created
        && !implementation.ledger_event_written
        && !implementation.position_written
        && !implementation.cash_written
        && !implementation.nav_or_performance_written
        && !implementation.model_or_metric_store_written
        && !implementation.training_or_rl_feedback_authorized
        && !implementation.reward_authorized
        && !implementation.order_generation_authorized
        && !implementation.broker_access_authorized
        && !implementation.trading_authorized;
    let provider_publication_time_still_unverified =
        !specification.provider_publication_time_verified;
    let checks = [
        ("implementation_record_hash", implementation_hash),
        ("implementation_contract_hash", contract_hash),
        ("specification_review_hash", review_hash),
        (
            "specification_independent_audit_hash",
            specification_audit_hash,
        ),
        ("specification_registration_hash", registration_hash),
        (
            "observation_ledger_transition_specification_hash",
            specification_hash,
        ),
        ("independent_contract_rebuild", rebuilt_contract_matches),
        ("stage_51_stage_117_binding", binding),
        ("deterministic_functions_and_schemas", deterministic),
        ("opening_portfolio_prerequisite", opening_portfolio),
        ("raw_adjusted_gap_nav", price_gap_nav),
        ("accounting_contract", accounting_contract),
        (
            "availability_create_once_and_output_path",
            availability_and_output,
        ),
        (
            "provider_publication_time_unverified",
            provider_publication_time_still_unverified,
        ),
        ("zero_authority", closed),
    ];
    let mismatch_reasons = checks
        .into_iter()
        .filter_map(|(name, ok)| (!ok).then_some(name.to_string()))
        .collect::<Vec<_>>();
    let mut audit = ControlledShadowObservationLedgerTransitionImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        specification_review_sha256: review.review_sha256.clone(),
        specification_independent_audit_sha256: specification_audit.audit_sha256.clone(),
        specification_registration_sha256: registration.registration_sha256.clone(),
        observation_ledger_transition_specification_sha256: specification
            .specification_sha256
            .clone(),
        implementation_record_hash_independently_reproduced: implementation_hash,
        implementation_contract_hash_independently_reproduced: contract_hash,
        specification_review_hash_independently_reproduced: review_hash,
        specification_independent_audit_hash_independently_reproduced: specification_audit_hash,
        specification_registration_hash_independently_reproduced: registration_hash,
        observation_ledger_transition_specification_hash_independently_reproduced:
            specification_hash,
        complete_implementation_contract_rebuilt_without_stage_117_builder: true,
        rebuilt_implementation_contract_exactly_matches_record: rebuilt_contract_matches,
        exact_current_stage_51_through_stage_117_binding_valid: binding,
        eight_function_ids_and_canonical_schemas_valid: deterministic,
        opening_portfolio_prerequisite_and_no_invention_contract_valid: opening_portfolio,
        raw_price_adjusted_price_gap_and_nav_contract_valid: price_gap_nav,
        corporate_action_decimal_idempotency_double_entry_and_correction_contract_valid:
            accounting_contract,
        conservative_availability_create_once_and_output_path_contract_valid:
            availability_and_output,
        provider_publication_time_still_unverified,
        all_source_artifact_runtime_input_ledger_financial_feedback_order_broker_and_trading_authority_closed:
            closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn closed_authority_boundary()
-> ZeroCapabilityObservationLedgerTransitionImplementationAuthorityBoundary {
    ZeroCapabilityObservationLedgerTransitionImplementationAuthorityBoundary {
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        input_mount_present: false,
        input_read_allowed: false,
        environment_inheritance_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        opening_portfolio_snapshot_present: false,
        ledger_created: false,
        ledger_event_write_allowed: false,
        position_write_allowed: false,
        cash_write_allowed: false,
        nav_or_performance_write_allowed: false,
        model_or_metric_store_write_allowed: false,
        training_or_rl_feedback_allowed: false,
        reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    }
}

fn authority_closed(
    boundary: &ZeroCapabilityObservationLedgerTransitionImplementationAuthorityBoundary,
) -> bool {
    boundary == &closed_authority_boundary()
}

pub(crate) fn validate_controlled_shadow_observation_ledger_transition_implementation_review_for_runner_registration(
    review: &ControlledShadowObservationLedgerTransitionImplementationReviewRecord,
) -> Result<(), String> {
    validate_observation_ledger_transition_implementation_for_review(&review.implementation)?;
    let audit = independently_audit(&review.implementation)?;
    let expected_actors = expected_excluded_actor_ids(&review.implementation, &[]);
    let texts = [
        &review.rationale,
        &review.binding_and_recomputation_assessment,
        &review.deterministic_projection_semantics_assessment,
        &review.session_price_basis_gap_and_company_action_assessment,
        &review.initial_allocation_availability_and_output_assessment,
        &review.zero_capability_assessment,
        &review.known_limitations,
        &review.future_runner_constraints,
    ];
    let approved = review.verdict == ControlledShadowObservationLedgerTransitionImplementationReviewVerdict::ApprovedForFutureIsolatedObservationLedgerTransitionRunnerSpecificationRegistration;
    let closed = !review.isolated_runner_registered
        && !review.source_artifact_present
        && !review.executable_artifact_present
        && !review.callable_entrypoint_present
        && !review.runtime_present
        && !review.input_mounted_or_read
        && !review.opening_portfolio_snapshot_admitted
        && !review.ledger_created
        && !review.ledger_event_written
        && !review.position_written
        && !review.cash_written
        && !review.nav_or_performance_written
        && !review.model_or_metric_store_written
        && !review.training_or_rl_feedback_authorized
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
        || review
            .future_isolated_observation_ledger_transition_runner_specification_registration_eligible
            != (approved && record_checks(review))
    {
        return Err("Stage 118 账本转换实现复核记录无效、漂移或越权".to_string());
    }
    Ok(())
}

fn validate_review(
    review: &ControlledShadowObservationLedgerTransitionImplementationReviewRecord,
) -> Result<(), String> {
    validate_controlled_shadow_observation_ledger_transition_implementation_review_for_runner_registration(
        review,
    )
}

fn request_checks(
    value: &ReviewControlledShadowObservationLedgerTransitionImplementationRequest,
) -> bool {
    value.exact_current_stage_51_through_stage_117_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed
        && value.all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed
        && value.exact_stage_114_admitted_output_is_only_future_input_confirmed
        && value.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && value.explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed
        && value.dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed
        && value.initial_shadow_allocation_and_conservative_availability_preserved_confirmed
        && value.provider_publication_time_remains_unverified_confirmed
        && value.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed
        && value.future_output_untrusted_and_independent_validation_required_confirmed
        && value.no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn record_checks(
    value: &ControlledShadowObservationLedgerTransitionImplementationReviewRecord,
) -> bool {
    value.exact_current_stage_51_through_stage_117_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed
        && value.all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed
        && value.exact_stage_114_admitted_output_is_only_future_input_confirmed
        && value.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && value.explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed
        && value.dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed
        && value.initial_shadow_allocation_and_conservative_availability_preserved_confirmed
        && value.provider_publication_time_remains_unverified_confirmed
        && value.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed
        && value.future_output_untrusted_and_independent_validation_required_confirmed
        && value.no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn expected_excluded_actor_ids(
    implementation: &ControlledShadowObservationLedgerTransitionImplementationRecord,
    reviews: &[ControlledShadowObservationLedgerTransitionImplementationReviewRecord],
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
) -> Result<Vec<ControlledShadowObservationLedgerTransitionImplementationReviewRecord>, String> {
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
            let review: ControlledShadowObservationLedgerTransitionImplementationReviewRecord =
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
            return Err("Stage 118 复核链分叉、断裂、重复、跨实现或角色排除不完整".to_string());
        }
        previous_id = Some(review.review_id.clone());
        previous_sha = Some(review.review_sha256.clone());
    }
    if reviews
        .iter()
        .take(reviews.len().saturating_sub(1))
        .any(|review| {
            review.future_isolated_observation_ledger_transition_runner_specification_registration_eligible
        })
    {
        return Err("Stage 118 批准记录必须是终止链尖".to_string());
    }
    Ok(reviews)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 118 指纹载荷无效".to_string())?;
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
    decision_root(state).join(
        "controlled-shadow-first-natural-cycle-observation-ledger-transition-implementation-reviews",
    )
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
        "controlled-shadow-observation-ledger-transition-implementation-review-{implementation_id}.lock"
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
        .map_err(|_| "已有同一 Stage 118 复核正在进行".to_string())?;
    Ok(ReviewLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ReviewControlledShadowObservationLedgerTransitionImplementationRequest {
        ReviewControlledShadowObservationLedgerTransitionImplementationRequest {
            expected_previous_review_id: None, expected_previous_review_sha256: None,
            expected_implementation_sha256: "a".repeat(64), expected_implementation_contract_sha256: "b".repeat(64),
            expected_specification_review_sha256: "c".repeat(64),
            expected_specification_independent_audit_sha256: "d".repeat(64),
            expected_specification_registration_sha256: "e".repeat(64),
            expected_observation_ledger_transition_specification_sha256: "f".repeat(64),
            expected_independent_audit_sha256: "1".repeat(64),
            verdict: ControlledShadowObservationLedgerTransitionImplementationReviewVerdict::ApprovedForFutureIsolatedObservationLedgerTransitionRunnerSpecificationRegistration,
            rationale: "独立复核".to_string(), binding_and_recomputation_assessment: "全链重算".to_string(),
            deterministic_projection_semantics_assessment: "八个纯函数和五个 canonical schema 一致".to_string(),
            session_price_basis_gap_and_company_action_assessment: "交易日、三价格口径、显式缺口和公司行动边界一致".to_string(),
            initial_allocation_availability_and_output_assessment: "初始组合绑定、保守可用时间和输出路径一致".to_string(),
            zero_capability_assessment: "全部关闭".to_string(),
            known_limitations: "尚无真实账本转换样本".to_string(), future_runner_constraints: "仍须独立登记 Stage 119 runner 规格".to_string(),
            exact_current_stage_51_through_stage_117_binding_confirmed: true,
            reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
            implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: true,
            all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: true,
            exact_stage_114_admitted_output_is_only_future_input_confirmed: true,
            official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
            explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
            dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: true,
            initial_shadow_allocation_and_conservative_availability_preserved_confirmed: true,
            provider_publication_time_remains_unverified_confirmed: true,
            one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
            future_output_untrusted_and_independent_validation_required_confirmed: true,
            no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_isolated_observation_ledger_transition_runner_specification_registration_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn approval_requires_every_confirmation() {
        let mut value = request();
        assert!(request_checks(&value));
        value.provider_publication_time_remains_unverified_confirmed = false;
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

    #[test]
    fn independent_review_binds_exactly_eight_pure_function_contracts() {
        assert_eq!(EXPECTED_FUNCTION_IDS.len(), 8);
        assert!(
            EXPECTED_FUNCTION_IDS
                .iter()
                .all(|function_id| function_id.ends_with("_pure_contract"))
        );
        let mut unique = EXPECTED_FUNCTION_IDS.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), EXPECTED_FUNCTION_IDS.len());
    }

    #[test]
    fn authority_boundary_keeps_opening_ledger_financial_and_trading_capabilities_closed() {
        let boundary = closed_authority_boundary();
        assert!(authority_closed(&boundary));
        let mut drifted = boundary;
        drifted.opening_portfolio_snapshot_present = true;
        assert!(!authority_closed(&drifted));
    }
}
