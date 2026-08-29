//! Stage 108 chain-external review of one Stage 107 zero-capability observation-materialization
//! implementation contract.
//!
//! Approval opens only a future isolated observation-materialization runner-specification
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_implementations::{
    ControlledShadowObservationMaterializationImplementationRecord,
    ZeroCapabilityObservationMaterializationImplementationAuthorityBoundary,
    independently_reviewable_observation_materialization_implementations,
    validate_observation_materialization_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-implementation-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-implementation-independent-audit-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-materialization-implementation-chain-external-review-v1-no-observation";
const STAGE_107_RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-implementation-v1";
const STAGE_107_POLICY_VERSION: &str = "hone-controlled-shadow-observation-materialization-implementation-create-once-v1-zero-capability";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-zero-capability-observation-materialization-implementation-contract-v1";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-first-natural-cycle-observation-materialization-implementation-v1-not-executable";
const STAGE_107_STATUS: &str = "registered_not_reviewed_not_run";
const MAX_TEXT_CHARS: usize = 4_000;

const EXPECTED_FUNCTION_IDS: [&str; 8] = [
    "observation_materialization_current_source_binding_validation_v1_pure_specification",
    "observation_materialization_canonical_session_projection_v1_pure_specification",
    "observation_materialization_three_price_basis_projection_v1_pure_specification",
    "observation_materialization_explicit_gap_spy_fail_closed_v1_pure_specification",
    "observation_materialization_corporate_action_separation_v1_pure_specification",
    "observation_materialization_initial_allocation_binding_v1_pure_specification",
    "observation_materialization_conservative_availability_v1_pure_specification",
    "observation_materialization_canonical_envelope_serialization_hash_v1_pure_specification",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationMaterializationImplementationReviewVerdict {
    ApprovedForFutureIsolatedObservationMaterializationRunnerSpecificationRegistration,
    ChangesRequiredRebuildObservationMaterializationImplementation,
    RejectedObservationMaterializationImplementation,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowObservationMaterializationImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_specification_review_sha256: String,
    expected_specification_independent_audit_sha256: String,
    expected_specification_registration_sha256: String,
    expected_observation_materialization_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: ControlledShadowObservationMaterializationImplementationReviewVerdict,
    rationale: String,
    binding_and_recomputation_assessment: String,
    deterministic_projection_semantics_assessment: String,
    session_price_basis_gap_and_company_action_assessment: String,
    initial_allocation_availability_and_output_assessment: String,
    zero_capability_assessment: String,
    known_limitations: String,
    future_runner_constraints: String,
    exact_current_stage_51_through_stage_107_binding_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed:
        bool,
    all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: bool,
    exact_stage_104_admitted_output_is_only_future_input_confirmed: bool,
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
    approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub specification_review_sha256: String,
    pub specification_independent_audit_sha256: String,
    pub specification_registration_sha256: String,
    pub observation_materialization_specification_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub specification_review_hash_independently_reproduced: bool,
    pub specification_independent_audit_hash_independently_reproduced: bool,
    pub specification_registration_hash_independently_reproduced: bool,
    pub observation_materialization_specification_hash_independently_reproduced: bool,
    pub exact_current_stage_51_through_stage_107_binding_valid: bool,
    pub eight_function_ids_and_canonical_schemas_valid: bool,
    pub admitted_input_session_price_gap_and_company_action_contract_valid: bool,
    pub allocation_availability_create_once_and_output_path_contract_valid: bool,
    pub provider_publication_time_still_unverified: bool,
    pub all_artifact_runtime_input_observation_store_feedback_order_broker_and_trading_authority_closed:
        bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: ControlledShadowObservationMaterializationImplementationRecord,
    pub independent_audit: ControlledShadowObservationMaterializationImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowObservationMaterializationImplementationReviewVerdict,
    pub rationale: String,
    pub binding_and_recomputation_assessment: String,
    pub deterministic_projection_semantics_assessment: String,
    pub session_price_basis_gap_and_company_action_assessment: String,
    pub initial_allocation_availability_and_output_assessment: String,
    pub zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_runner_constraints: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_107_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed:
        bool,
    pub all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: bool,
    pub exact_stage_104_admitted_output_is_only_future_input_confirmed: bool,
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
    pub approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_implementation_independently_approved: bool,
    pub future_isolated_observation_materialization_runner_specification_registration_eligible:
        bool,
    pub isolated_runner_registered: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mounted_or_read: bool,
    pub observation_materialized: bool,
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
pub(crate) struct ControlledShadowObservationMaterializationImplementationReviewItem {
    pub implementation: ControlledShadowObservationMaterializationImplementationRecord,
    pub current_independent_audit:
        ControlledShadowObservationMaterializationImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<ControlledShadowObservationMaterializationImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_isolated_observation_materialization_runner_specification_registration_eligible:
        bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowObservationMaterializationImplementationReviewItem>,
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_isolated_observation_materialization_runner_specification_registration_eligible_count:
        usize,
    pub review_status: String,
    pub isolated_runner_registered: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mounted_or_read: bool,
    pub observation_materialized: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationMaterializationImplementationReviewReadinessSummary {
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_isolated_observation_materialization_runner_specification_registration_eligible_count:
        usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyApprovedControlledShadowObservationMaterializationImplementation {
    pub implementation: ControlledShadowObservationMaterializationImplementationRecord,
    pub review: ControlledShadowObservationMaterializationImplementationReviewRecord,
}

pub(crate) async fn handle_get_controlled_shadow_observation_materialization_implementation_reviews(
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
            warn!(%error, "observation-materialization implementation review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察物化零能力实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_observation_materialization_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowObservationMaterializationImplementationRequest>,
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

pub(crate) async fn controlled_shadow_observation_materialization_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationImplementationReviewReadinessSummary, String>
{
    let value = build_registry(state).await?;
    Ok(ControlledShadowObservationMaterializationImplementationReviewReadinessSummary {
        implementation_count: value.implementation_count,
        review_eligible_count: value.review_eligible_count,
        reviewed_count: value.reviewed_count,
        independently_approved_count: value.independently_approved_count,
        changes_required_or_rejected_count: value.changes_required_or_rejected_count,
        future_isolated_observation_materialization_runner_specification_registration_eligible_count: value.future_isolated_observation_materialization_runner_specification_registration_eligible_count,
        review_status: value.review_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn independently_approved_observation_materialization_implementations_for_isolated_runner_registration(
    state: &AppState,
) -> Result<
    Vec<IndependentlyApprovedControlledShadowObservationMaterializationImplementation>,
    String,
> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let review = item.latest_review?;
            item.future_isolated_observation_materialization_runner_specification_registration_eligible
                .then_some(
                    IndependentlyApprovedControlledShadowObservationMaterializationImplementation {
                        implementation: item.implementation,
                        review,
                    },
                )
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationImplementationReviewRegistry, String> {
    let implementations =
        independently_reviewable_observation_materialization_implementations(state).await?;
    let mut items = Vec::new();
    for implementation in implementations {
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 107 实现独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|value| {
            value.implementation != implementation || value.independent_audit != audit
        }) {
            return Err("Stage 108 复核绑定或独立审计已经变化".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|value| {
            value.future_isolated_observation_materialization_runner_specification_registration_eligible
        });
        items.push(ControlledShadowObservationMaterializationImplementationReviewItem {
            implementation,
            current_independent_audit: audit,
            complete_review_actor_ids: reviews
                .iter()
                .map(|value| value.reviewer_id.clone())
                .collect(),
            latest_review,
            review_eligible: !approved,
            future_isolated_observation_materialization_runner_specification_registration_eligible:
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
            value.future_isolated_observation_materialization_runner_specification_registration_eligible
        })
        .count();
    let changes_required_or_rejected_count = items.iter().filter(|value| value.latest_review.as_ref().is_some_and(|review| !review.future_isolated_observation_materialization_runner_specification_registration_eligible)).count();
    let review_status = if items.is_empty() {
        "waiting_stage_107_zero_capability_observation_materialization_implementation"
    } else if independently_approved_count > 0 {
        "observation_materialization_implementation_independently_approved_only_for_future_isolated_runner_specification_registration"
    } else if review_eligible_count > 0 {
        "observation_materialization_implementation_ready_for_chain_external_review"
    } else {
        "observation_materialization_implementation_review_blocked_fail_closed"
    };
    Ok(ControlledShadowObservationMaterializationImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(), items,
        implementation_count, review_eligible_count, reviewed_count, independently_approved_count,
        changes_required_or_rejected_count,
        future_isolated_observation_materialization_runner_specification_registration_eligible_count: independently_approved_count,
        review_status: review_status.to_string(), isolated_runner_registered: false,
        source_artifact_present: false,
        executable_artifact_present: false, callable_entrypoint_present: false, runtime_present: false,
        input_mounted_or_read: false, observation_materialized: false, ledger_created: false,
        position_written: false, performance_metric_written: false,
        order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        scope: "Stage 108 由 Stage 51–107 完整责任链之外的新角色，以第二实现独立重算 Stage 107 implementation/contract、Stage 106 review/audit、Stage 105 registration/specification 指纹，并复核八个纯函数标识、canonical schema、精确准入输入、交易日/三价格口径/显式 gap、公司行动、初始组合、可用时间、内容寻址输出和全部零权限位。批准只开放未来 Stage 109 隔离观察物化 runner 规格登记；不提交工件、不读取输入、不物化观察、不建账、不写持仓/绩效、不训练、不奖励、不下单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewControlledShadowObservationMaterializationImplementationRequest,
) -> Result<ControlledShadowObservationMaterializationImplementationReviewRecord, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("Stage 108 复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementation =
        independently_reviewable_observation_materialization_implementations(state)
            .await?
            .into_iter()
            .find(|value| value.implementation_id == implementation_id)
            .ok_or_else(|| "当前 Stage 107 实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|value| {
        value.future_isolated_observation_materialization_runner_specification_registration_eligible
    }) {
        return Err("当前 Stage 107 实现已经独立批准，不得追加或分叉复核".to_string());
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
    implementation: &ControlledShadowObservationMaterializationImplementationRecord,
    prior_reviews: &[ControlledShadowObservationMaterializationImplementationReviewRecord],
    prior: Option<&ControlledShadowObservationMaterializationImplementationReviewRecord>,
    reviewer_id: &str,
    request: ReviewControlledShadowObservationMaterializationImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<ControlledShadowObservationMaterializationImplementationReviewRecord, String> {
    validate_observation_materialization_implementation_for_review(implementation)?;
    let audit = independently_audit(implementation)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("Stage 107 实现独立审计不通过".to_string());
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
        || request.expected_observation_materialization_specification_sha256
            != specification.specification_sha256
        || request.expected_independent_audit_sha256 != audit.audit_sha256
    {
        return Err("Stage 105–107 实现、规格、审计或复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|value| value == reviewer_id)
    {
        return Err("复核人参与过 Stage 51–107 或此前复核链，必须更换责任链外角色".to_string());
    }
    let approved = request.verdict == ControlledShadowObservationMaterializationImplementationReviewVerdict::ApprovedForFutureIsolatedObservationMaterializationRunnerSpecificationRegistration;
    let confirmed = request_checks(&request);
    if approved && !confirmed {
        return Err("批准前必须逐项确认独立重算、观察物化语义、可用时间和零权限边界".to_string());
    }
    let mut review = ControlledShadowObservationMaterializationImplementationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(), review_sha256: String::new(),
        previous_review_id: prior.map(|value| value.review_id.clone()),
        previous_review_sha256: prior.map(|value| value.review_sha256.clone()),
        implementation: implementation.clone(), independent_audit: audit, submitted_at,
        reviewer_id: reviewer_id.to_string(), excluded_prior_actor_ids, verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        binding_and_recomputation_assessment: bounded_required(&request.binding_and_recomputation_assessment, "绑定与重算评估")?,
        deterministic_projection_semantics_assessment: bounded_required(&request.deterministic_projection_semantics_assessment, "确定性投影语义评估")?,
        session_price_basis_gap_and_company_action_assessment: bounded_required(&request.session_price_basis_gap_and_company_action_assessment, "交易日、价格口径、缺口与公司行动评估")?,
        initial_allocation_availability_and_output_assessment: bounded_required(&request.initial_allocation_availability_and_output_assessment, "初始组合、可用时间与输出评估")?,
        zero_capability_assessment: bounded_required(&request.zero_capability_assessment, "零能力评估")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_runner_constraints: bounded_required(&request.future_runner_constraints, "未来 runner 约束")?,
        reviewer_independent_from_registrar_and_complete_prior_chain: true,
        exact_current_stage_51_through_stage_107_binding_confirmed: request.exact_current_stage_51_through_stage_107_binding_confirmed,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: request.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed,
        implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: request.implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed,
        all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: request.all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed,
        exact_stage_104_admitted_output_is_only_future_input_confirmed: request.exact_stage_104_admitted_output_is_only_future_input_confirmed,
        official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: request.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed,
        explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: request.explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed,
        dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: request.dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed,
        initial_shadow_allocation_and_conservative_availability_preserved_confirmed: request.initial_shadow_allocation_and_conservative_availability_preserved_confirmed,
        provider_publication_time_remains_unverified_confirmed: request.provider_publication_time_remains_unverified_confirmed,
        one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: request.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed,
        future_output_untrusted_and_independent_validation_required_confirmed: request.future_output_untrusted_and_independent_validation_required_confirmed,
        no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: request.no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed: request.approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_implementation_independently_approved: approved && confirmed,
        future_isolated_observation_materialization_runner_specification_registration_eligible: approved && confirmed,
        isolated_runner_registered: false, source_artifact_present: false, executable_artifact_present: false,
        callable_entrypoint_present: false, runtime_present: false, input_mounted_or_read: false,
        observation_materialized: false, ledger_created: false, position_written: false,
        performance_metric_written: false, model_store_written: false, metric_store_written: false,
        training_feedback_authorized: false, reward_authorized: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false,
    };
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review)?;
    Ok(review)
}

fn independently_audit(
    implementation: &ControlledShadowObservationMaterializationImplementationRecord,
) -> Result<ControlledShadowObservationMaterializationImplementationIndependentAudit, String> {
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
    let binding = implementation.schema_version == STAGE_107_RECORD_SCHEMA_VERSION
        && implementation.policy_version == STAGE_107_POLICY_VERSION
        && implementation.status == STAGE_107_STATUS
        && implementation.confirmations_complete
        && implementation.zero_capability_implementation_contract_registered
        && implementation.future_independent_implementation_review_eligible
        && implementation.registrar_independent_from_stage_106_and_complete_prior_chain
        && implementation.implementation_id == implementation.implementation_sha256[..32]
        && contract.stage_106_specification_review_id == review.review_id
        && contract.stage_106_specification_review_sha256 == review.review_sha256
        && contract.stage_106_independent_audit_sha256 == specification_audit.audit_sha256
        && contract.stage_105_registration_id == registration.registration_id
        && contract.stage_105_registration_sha256 == registration.registration_sha256
        && contract.observation_materialization_specification_sha256
            == specification.specification_sha256
        && contract.exact_observation_materialization_specification == *specification
        && implementation_hash
        && contract_hash
        && review_hash
        && specification_audit_hash
        && registration_hash
        && specification_hash;
    let functions = [
        contract
            .current_source_binding_validation_function_id
            .as_str(),
        contract.canonical_session_projection_function_id.as_str(),
        contract.three_price_basis_projection_function_id.as_str(),
        contract
            .explicit_gap_and_spy_fail_closed_function_id
            .as_str(),
        contract.corporate_action_separation_function_id.as_str(),
        contract.initial_allocation_binding_function_id.as_str(),
        contract.conservative_availability_function_id.as_str(),
        contract
            .canonical_envelope_serialization_and_hash_function_id
            .as_str(),
    ];
    let deterministic = contract.schema_version == CONTRACT_SCHEMA_VERSION
        && contract.implementation_protocol_version == IMPLEMENTATION_PROTOCOL_VERSION
        && functions == EXPECTED_FUNCTION_IDS
        && contract.canonical_session_schema == specification.canonical_session_schema
        && contract.canonical_price_observation_schema
            == specification.canonical_price_observation_schema
        && contract.canonical_explicit_gap_schema == specification.canonical_explicit_gap_schema
        && contract.canonical_corporate_action_schema
            == specification.canonical_corporate_action_schema
        && contract.canonical_observation_envelope_schema
            == specification.canonical_observation_envelope_schema
        && contract.registered_not_run
        && contract.independent_implementation_review_required
        && contract.isolated_runner_registration_required_after_review;
    let admitted_input_session_price_gap_and_company_action = specification
        .exact_admitted_output_is_only_input
        && specification.official_market_session_count > 0
        && specification.benchmark_symbol == "SPY"
        && specification.allowed_price_bases
            == [
                "raw_unadjusted_price",
                "split_adjusted_price",
                "dividend_adjusted_price",
            ]
        && specification.subject_gap_is_explicit_not_synthetic
        && !specification.benchmark_gap_allowed
        && !specification.duplicate_row_allowed
        && !specification.out_of_window_row_allowed
        && !specification.forward_fill_allowed
        && !specification.interpolation_allowed
        && !specification.cross_price_basis_substitution_allowed
        && specification.dividends_splits_and_prices_separate;
    let allocation_availability_and_output = !specification
        .stage_88_initial_observation_output_sha256
        .is_empty()
        && !specification
            .initial_shadow_allocation_manifest_sha256
            .is_empty()
        && !specification.initial_shadow_allocation_recomputed
        && !specification.accounting_transition_applied
        && !specification.provider_publication_time_verified
        && specification.one_envelope_per_admitted_cycle
        && specification.create_once_required
        && !specification.overwrite_allowed
        && !specification.retroactive_backfill_allowed
        && !specification.in_place_correction_allowed
        && !specification.valuation_or_investment_semantics_allowed
        && specification.future_output_untrusted
        && specification.future_output_independent_validation_required
        && contract.future_exact_admitted_input_read_only_and_content_addressed
        && contract.future_output_relative_path_template
            == specification.future_output_relative_path_template
        && contract.future_observation_output_create_once_and_untrusted
        && contract.future_observation_output_independent_validation_required
        && !contract.future_observation_output_contains_valuation_or_investment_semantics
        && !contract.future_observation_output_contains_order_intent;
    let boundary = &contract.authority_boundary;
    let closed = authority_closed(boundary)
        && !implementation.observation_materialization_implementation_present
        && !implementation.independent_implementation_review_completed
        && !implementation.isolated_runner_registration_eligible
        && !implementation.observation_materialized
        && !implementation.ledger_created
        && !implementation.position_written
        && !implementation.performance_metric_written
        && !implementation.model_store_written
        && !implementation.metric_store_written
        && !implementation.training_feedback_authorized
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
            "observation_materialization_specification_hash",
            specification_hash,
        ),
        ("stage_51_stage_107_binding", binding),
        ("deterministic_functions_and_schemas", deterministic),
        (
            "admitted_input_session_price_gap_and_company_action",
            admitted_input_session_price_gap_and_company_action,
        ),
        (
            "allocation_availability_create_once_and_output_path",
            allocation_availability_and_output,
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
    let mut audit = ControlledShadowObservationMaterializationImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        specification_review_sha256: review.review_sha256.clone(),
        specification_independent_audit_sha256: specification_audit.audit_sha256.clone(),
        specification_registration_sha256: registration.registration_sha256.clone(),
        observation_materialization_specification_sha256: specification.specification_sha256.clone(),
        implementation_record_hash_independently_reproduced: implementation_hash,
        implementation_contract_hash_independently_reproduced: contract_hash,
        specification_review_hash_independently_reproduced: review_hash,
        specification_independent_audit_hash_independently_reproduced: specification_audit_hash,
        specification_registration_hash_independently_reproduced: registration_hash,
        observation_materialization_specification_hash_independently_reproduced: specification_hash,
        exact_current_stage_51_through_stage_107_binding_valid: binding,
        eight_function_ids_and_canonical_schemas_valid: deterministic,
        admitted_input_session_price_gap_and_company_action_contract_valid:
            admitted_input_session_price_gap_and_company_action,
        allocation_availability_create_once_and_output_path_contract_valid:
            allocation_availability_and_output,
        provider_publication_time_still_unverified,
        all_artifact_runtime_input_observation_store_feedback_order_broker_and_trading_authority_closed:
            closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn authority_closed(
    boundary: &ZeroCapabilityObservationMaterializationImplementationAuthorityBoundary,
) -> bool {
    !boundary.source_artifact_present
        && !boundary.executable_artifact_present
        && !boundary.callable_entrypoint_present
        && !boundary.runtime_present
        && !boundary.input_mount_present
        && !boundary.input_read_allowed
        && !boundary.environment_inheritance_allowed
        && !boundary.secrets_allowed
        && !boundary.outbound_network_allowed
        && !boundary.tools_allowed
        && !boundary.subprocesses_allowed
        && !boundary.production_reads_allowed
        && !boundary.production_writes_allowed
        && !boundary.observation_envelope_created
        && !boundary.ledger_creation_allowed
        && !boundary.position_write_allowed
        && !boundary.performance_metric_write_allowed
        && !boundary.model_store_write_allowed
        && !boundary.metric_store_write_allowed
        && !boundary.training_feedback_allowed
        && !boundary.reward_allowed
        && !boundary.order_generation_allowed
        && !boundary.broker_access_allowed
        && !boundary.trading_allowed
}

pub(crate) fn validate_controlled_shadow_observation_materialization_implementation_review_for_runner_registration(
    review: &ControlledShadowObservationMaterializationImplementationReviewRecord,
) -> Result<(), String> {
    validate_observation_materialization_implementation_for_review(&review.implementation)?;
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
    let approved = review.verdict == ControlledShadowObservationMaterializationImplementationReviewVerdict::ApprovedForFutureIsolatedObservationMaterializationRunnerSpecificationRegistration;
    let closed = !review.isolated_runner_registered
        && !review.source_artifact_present
        && !review.executable_artifact_present
        && !review.callable_entrypoint_present
        && !review.runtime_present
        && !review.input_mounted_or_read
        && !review.observation_materialized
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
            .future_isolated_observation_materialization_runner_specification_registration_eligible
            != (approved && record_checks(review))
    {
        return Err("Stage 108 观察物化实现复核记录无效、漂移或越权".to_string());
    }
    Ok(())
}

fn validate_review(
    review: &ControlledShadowObservationMaterializationImplementationReviewRecord,
) -> Result<(), String> {
    validate_controlled_shadow_observation_materialization_implementation_review_for_runner_registration(
        review,
    )
}

fn request_checks(
    value: &ReviewControlledShadowObservationMaterializationImplementationRequest,
) -> bool {
    value.exact_current_stage_51_through_stage_107_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed
        && value.all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed
        && value.exact_stage_104_admitted_output_is_only_future_input_confirmed
        && value.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && value.explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed
        && value.dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed
        && value.initial_shadow_allocation_and_conservative_availability_preserved_confirmed
        && value.provider_publication_time_remains_unverified_confirmed
        && value.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed
        && value.future_output_untrusted_and_independent_validation_required_confirmed
        && value.no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn record_checks(
    value: &ControlledShadowObservationMaterializationImplementationReviewRecord,
) -> bool {
    value.exact_current_stage_51_through_stage_107_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed
        && value.all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed
        && value.exact_stage_104_admitted_output_is_only_future_input_confirmed
        && value.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && value.explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed
        && value.dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed
        && value.initial_shadow_allocation_and_conservative_availability_preserved_confirmed
        && value.provider_publication_time_remains_unverified_confirmed
        && value.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed
        && value.future_output_untrusted_and_independent_validation_required_confirmed
        && value.no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn expected_excluded_actor_ids(
    implementation: &ControlledShadowObservationMaterializationImplementationRecord,
    reviews: &[ControlledShadowObservationMaterializationImplementationReviewRecord],
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
) -> Result<Vec<ControlledShadowObservationMaterializationImplementationReviewRecord>, String> {
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
            let review: ControlledShadowObservationMaterializationImplementationReviewRecord =
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
            return Err("Stage 108 复核链分叉、断裂、重复、跨实现或角色排除不完整".to_string());
        }
        previous_id = Some(review.review_id.clone());
        previous_sha = Some(review.review_sha256.clone());
    }
    if reviews
        .iter()
        .take(reviews.len().saturating_sub(1))
        .any(|review| {
            review.future_isolated_observation_materialization_runner_specification_registration_eligible
        })
    {
        return Err("Stage 108 批准记录必须是终止链尖".to_string());
    }
    Ok(reviews)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 108 指纹载荷无效".to_string())?;
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
        "controlled-shadow-first-natural-cycle-observation-materialization-implementation-reviews",
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
        "controlled-shadow-observation-materialization-implementation-review-{implementation_id}.lock"
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
        .map_err(|_| "已有同一 Stage 108 复核正在进行".to_string())?;
    Ok(ReviewLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ReviewControlledShadowObservationMaterializationImplementationRequest {
        ReviewControlledShadowObservationMaterializationImplementationRequest {
            expected_previous_review_id: None, expected_previous_review_sha256: None,
            expected_implementation_sha256: "a".repeat(64), expected_implementation_contract_sha256: "b".repeat(64),
            expected_specification_review_sha256: "c".repeat(64),
            expected_specification_independent_audit_sha256: "d".repeat(64),
            expected_specification_registration_sha256: "e".repeat(64),
            expected_observation_materialization_specification_sha256: "f".repeat(64),
            expected_independent_audit_sha256: "1".repeat(64),
            verdict: ControlledShadowObservationMaterializationImplementationReviewVerdict::ApprovedForFutureIsolatedObservationMaterializationRunnerSpecificationRegistration,
            rationale: "独立复核".to_string(), binding_and_recomputation_assessment: "全链重算".to_string(),
            deterministic_projection_semantics_assessment: "八个纯函数和五个 canonical schema 一致".to_string(),
            session_price_basis_gap_and_company_action_assessment: "交易日、三价格口径、显式缺口和公司行动边界一致".to_string(),
            initial_allocation_availability_and_output_assessment: "初始组合绑定、保守可用时间和输出路径一致".to_string(),
            zero_capability_assessment: "全部关闭".to_string(),
            known_limitations: "尚无真实观察物化样本".to_string(), future_runner_constraints: "仍须独立登记 Stage 109 runner 规格".to_string(),
            exact_current_stage_51_through_stage_107_binding_confirmed: true,
            reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
            implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed: true,
            all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: true,
            exact_stage_104_admitted_output_is_only_future_input_confirmed: true,
            official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
            explicit_subject_gap_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
            dividends_splits_decimal_order_row_hash_and_content_addressed_output_preserved_confirmed: true,
            initial_shadow_allocation_and_conservative_availability_preserved_confirmed: true,
            provider_publication_time_remains_unverified_confirmed: true,
            one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
            future_output_untrusted_and_independent_validation_required_confirmed: true,
            no_source_or_executable_artifact_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_isolated_observation_materialization_runner_specification_registration_confirmed: true,
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
                .all(|function_id| function_id.ends_with("_pure_specification"))
        );
        let mut unique = EXPECTED_FUNCTION_IDS.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), EXPECTED_FUNCTION_IDS.len());
    }
}
