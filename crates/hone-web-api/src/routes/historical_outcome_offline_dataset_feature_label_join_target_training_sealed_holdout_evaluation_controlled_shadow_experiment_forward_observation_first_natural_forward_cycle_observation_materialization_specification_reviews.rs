//! Stage 106 chain-external review of one Stage 105 observation-materialization specification.
//!
//! The audit independently rebuilds the complete specification from the current Stage 104
//! admitted source. Approval opens only a future zero-capability implementation-registration
//! gate. This module cannot materialize an observation or create a ledger, position, performance
//! metric, model/reward fact, order, broker request or trade.

use std::collections::{BTreeMap, BTreeSet};
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_input_admission_reviews::AdmittedControlledShadowObservationInputForMaterializationSpecification;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_specifications::{
    ControlledShadowObservationMaterializationSpecification,
    ControlledShadowObservationMaterializationSpecificationRegistration,
    IndependentlyReviewableControlledShadowObservationMaterializationSpecification,
    ZeroCapabilityObservationMaterializationAuthorityBoundary,
    independently_reviewable_observation_materialization_specifications,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-specification-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-specification-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-specification-independent-audit-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-materialization-specification-chain-external-review-v1-zero-capability";
const STAGE_105_RECORD_SCHEMA_VERSION: &str = "hone-controlled-shadow-first-natural-cycle-observation-materialization-specification-registration-v1";
const STAGE_105_SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-first-natural-cycle-observation-materialization-specification-v1";
const STAGE_105_POLICY_VERSION: &str = "hone-first-natural-cycle-observation-materialization-specification-create-once-v1-zero-capability";
const MATERIALIZATION_PROTOCOL_VERSION: &str =
    "hone-first-natural-cycle-observation-materialization-v1-specification-only-not-executable";
const STAGE_105_STATUS: &str = "registered_not_reviewed_not_implemented_not_run";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationMaterializationSpecificationReviewVerdict {
    ApprovedForFutureZeroCapabilityObservationMaterializationImplementationRegistration,
    ChangesRequiredRebuildObservationMaterializationSpecification,
    RejectedObservationMaterializationSpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowObservationMaterializationSpecificationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_registration_sha256: String,
    expected_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: ControlledShadowObservationMaterializationSpecificationReviewVerdict,
    rationale: String,
    binding_and_second_implementation_assessment: String,
    session_price_basis_and_gap_assessment: String,
    corporate_action_decimal_order_and_hash_assessment: String,
    initial_allocation_and_availability_assessment: String,
    zero_capability_assessment: String,
    known_limitations: String,
    future_implementation_constraints: String,
    exact_current_stage_51_through_stage_105_binding_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    registration_and_specification_hashes_independently_reproduced_confirmed: bool,
    complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed:
        bool,
    rebuilt_specification_exactly_matches_registered_specification_confirmed: bool,
    official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: bool,
    subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: bool,
    dividends_splits_and_price_bases_remain_separate_confirmed: bool,
    decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: bool,
    initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed:
        bool,
    conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: bool,
    one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
        bool,
    future_output_untrusted_and_independent_validation_required_confirmed: bool,
    no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    approval_only_opens_future_zero_capability_implementation_registration_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationSpecificationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub specification_sha256: String,
    pub registration_hash_independently_reproduced: bool,
    pub specification_hash_independently_reproduced: bool,
    pub exact_current_stage_51_through_stage_105_binding_valid: bool,
    pub complete_specification_rebuilt_without_stage_105_builder: bool,
    pub rebuilt_specification_exactly_matches_registration: bool,
    pub session_subject_spy_three_price_basis_and_gap_contract_valid: bool,
    pub corporate_action_decimal_order_hash_and_output_path_contract_valid: bool,
    pub initial_shadow_allocation_and_availability_contract_valid: bool,
    pub all_implementation_runtime_observation_store_feedback_order_broker_and_trading_authority_closed:
        bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationSpecificationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub registration: ControlledShadowObservationMaterializationSpecificationRegistration,
    pub independent_audit: ControlledShadowObservationMaterializationSpecificationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowObservationMaterializationSpecificationReviewVerdict,
    pub rationale: String,
    pub binding_and_second_implementation_assessment: String,
    pub session_price_basis_and_gap_assessment: String,
    pub corporate_action_decimal_order_and_hash_assessment: String,
    pub initial_allocation_and_availability_assessment: String,
    pub zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_implementation_constraints: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_105_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub registration_and_specification_hashes_independently_reproduced_confirmed: bool,
    pub complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed:
        bool,
    pub rebuilt_specification_exactly_matches_registered_specification_confirmed: bool,
    pub official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: bool,
    pub subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: bool,
    pub dividends_splits_and_price_bases_remain_separate_confirmed: bool,
    pub decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: bool,
    pub initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed:
        bool,
    pub conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed:
        bool,
    pub one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
        bool,
    pub future_output_untrusted_and_independent_validation_required_confirmed: bool,
    pub no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    pub no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub approval_only_opens_future_zero_capability_implementation_registration_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub specification_independently_approved: bool,
    pub future_zero_capability_implementation_registration_eligible: bool,
    pub implementation_registered: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub observation_materialized: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationSpecificationReviewItem {
    pub registration: ControlledShadowObservationMaterializationSpecificationRegistration,
    pub current_independent_audit:
        ControlledShadowObservationMaterializationSpecificationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<ControlledShadowObservationMaterializationSpecificationReviewRecord>,
    pub review_eligible: bool,
    pub future_zero_capability_implementation_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationSpecificationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_endpoint_available: bool,
    pub items: Vec<ControlledShadowObservationMaterializationSpecificationReviewItem>,
    pub specification_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_zero_capability_implementation_registration_eligible_count: usize,
    pub review_status: String,
    pub implementation_registered: bool,
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
pub(crate) struct ControlledShadowObservationMaterializationSpecificationReviewReadinessSummary {
    pub specification_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_zero_capability_implementation_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentlyApprovedControlledShadowObservationMaterializationSpecification {
    pub review: ControlledShadowObservationMaterializationSpecificationReviewRecord,
}

pub(crate) async fn handle_get_controlled_shadow_observation_materialization_specification_reviews(
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
            warn!(%error, "observation materialization specification independent review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察物化规格独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_observation_materialization_specification(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(registration_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowObservationMaterializationSpecificationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &registration_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_observation_materialization_specification_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationSpecificationReviewReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowObservationMaterializationSpecificationReviewReadinessSummary {
            specification_count: value.specification_count,
            review_eligible_count: value.review_eligible_count,
            reviewed_count: value.reviewed_count,
            independently_approved_count: value.independently_approved_count,
            changes_required_or_rejected_count: value.changes_required_or_rejected_count,
            future_zero_capability_implementation_registration_eligible_count: value
                .future_zero_capability_implementation_registration_eligible_count,
            review_status: value.review_status,
        },
    )
}

pub(crate) async fn independently_approved_observation_materialization_specifications_for_implementation_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyApprovedControlledShadowObservationMaterializationSpecification>, String>
{
    let reviewables =
        independently_reviewable_observation_materialization_specifications(state).await?;
    let mut approved = Vec::new();
    for reviewable in reviewables {
        let expected_audit = independently_audit(&reviewable)?;
        if !expected_audit.mismatch_reasons.is_empty() {
            return Err("Stage 106 当前独立审计未通过，Stage 107 晋级关闭".to_string());
        }
        let reviews = read_reviews(state, &reviewable).await?;
        let Some(review) = reviews.last().cloned() else {
            continue;
        };
        validate_review(&reviewable.source, &review)?;
        if review.independent_audit != expected_audit
            || review.registration != reviewable.registration
        {
            return Err("Stage 106 已批准复核与当前 Stage 104–105 绑定漂移".to_string());
        }
        if review.future_zero_capability_implementation_registration_eligible {
            approved.push(
                IndependentlyApprovedControlledShadowObservationMaterializationSpecification {
                    review,
                },
            );
        }
    }
    approved.sort_by(|left, right| right.review.submitted_at.cmp(&left.review.submitted_at));
    Ok(approved)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationSpecificationReviewRegistry, String> {
    let reviewables =
        independently_reviewable_observation_materialization_specifications(state).await?;
    let mut items = Vec::new();
    for reviewable in reviewables {
        let audit = independently_audit(&reviewable)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 105 观察物化规格独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &reviewable).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|review| {
            review.registration != reviewable.registration || review.independent_audit != audit
        }) {
            return Err("Stage 106 复核绑定或独立审计已经变化".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|review| {
            review.future_zero_capability_implementation_registration_eligible
        });
        items.push(
            ControlledShadowObservationMaterializationSpecificationReviewItem {
                registration: reviewable.registration,
                current_independent_audit: audit,
                complete_review_actor_ids: reviews
                    .iter()
                    .map(|review| review.reviewer_id.clone())
                    .collect(),
                latest_review,
                review_eligible: !approved,
                future_zero_capability_implementation_registration_eligible: approved,
            },
        );
    }
    items.sort_by(|left, right| {
        right
            .registration
            .registered_at
            .cmp(&left.registration.registered_at)
    });
    let specification_count = items.len();
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|item| item.future_zero_capability_implementation_registration_eligible)
        .count();
    let changes_required_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                !review.future_zero_capability_implementation_registration_eligible
            })
        })
        .count();
    let review_status = if items.is_empty() {
        "waiting_stage_105_observation_materialization_specification"
    } else if independently_approved_count > 0 {
        "observation_materialization_specification_independently_approved_only_for_future_zero_capability_implementation_registration"
    } else if review_eligible_count > 0 {
        "observation_materialization_specification_ready_for_chain_external_review"
    } else {
        "observation_materialization_specification_review_blocked_fail_closed"
    };
    Ok(
        ControlledShadowObservationMaterializationSpecificationReviewRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_endpoint_available: true,
            items,
            specification_count,
            review_eligible_count,
            reviewed_count,
            independently_approved_count,
            changes_required_or_rejected_count,
            future_zero_capability_implementation_registration_eligible_count:
                independently_approved_count,
            review_status: review_status.to_string(),
            implementation_registered: false,
            observation_materialized: false,
            ledger_created: false,
            position_written: false,
            performance_metric_written: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "Stage 106 由责任链外角色使用第二套实现，从当前 Stage 104 准入源完整重建 Stage 105 规格并逐字段比对。批准只开放未来 Stage 107 零能力实现登记候选；没有实现、工件、入口、runtime、输入挂载、观察、账本、持仓、绩效、模型、训练、reward、订单、券商或交易能力。".to_string(),
        },
    )
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    registration_id: &str,
    request: ReviewControlledShadowObservationMaterializationSpecificationRequest,
) -> Result<ControlledShadowObservationMaterializationSpecificationReviewRecord, String> {
    validate_request(registration_id, &request)?;
    let _lock = acquire_lock(state, registration_id).await?;
    let reviewable = independently_reviewable_observation_materialization_specifications(state)
        .await?
        .into_iter()
        .find(|value| value.registration.registration_id == registration_id)
        .ok_or_else(|| "当前没有可独立复核的 Stage 105 观察物化规格".to_string())?;
    if request.expected_registration_sha256 != reviewable.registration.registration_sha256
        || request.expected_specification_sha256
            != reviewable.registration.specification.specification_sha256
    {
        return Err("Stage 105 登记或规格摘要已变化".to_string());
    }
    let audit = independently_audit(&reviewable)?;
    if request.expected_independent_audit_sha256 != audit.audit_sha256
        || !audit.mismatch_reasons.is_empty()
    {
        return Err("Stage 106 独立审计摘要不匹配或未通过".to_string());
    }
    let prior_reviews = read_reviews(state, &reviewable).await?;
    let previous = prior_reviews.last();
    if request.expected_previous_review_id.as_deref()
        != previous.map(|value| value.review_id.as_str())
        || request.expected_previous_review_sha256.as_deref()
            != previous.map(|value| value.review_sha256.as_str())
    {
        return Err("Stage 106 前序复核绑定已变化".to_string());
    }
    if previous
        .is_some_and(|value| value.future_zero_capability_implementation_registration_eligible)
    {
        return Err("Stage 106 已批准终态不得追加复核".to_string());
    }
    let excluded = expected_excluded_actor_ids(&reviewable.registration, &prior_reviews);
    if reviewer_id.trim().is_empty() || excluded.iter().any(|value| value == reviewer_id) {
        return Err("Stage 106 reviewer 必须独立于 Stage 105 登记者和完整既有责任链".to_string());
    }
    let approved = request.verdict
        == ControlledShadowObservationMaterializationSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityObservationMaterializationImplementationRegistration;
    if approved && !request_checks(&request) {
        return Err("Stage 106 批准需要完成全部独立复核确认".to_string());
    }
    let mut review = ControlledShadowObservationMaterializationSpecificationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: previous.map(|value| value.review_id.clone()),
        previous_review_sha256: previous.map(|value| value.review_sha256.clone()),
        registration: reviewable.registration,
        independent_audit: audit,
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids: excluded,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        binding_and_second_implementation_assessment: bounded_required(
            &request.binding_and_second_implementation_assessment,
            "绑定与第二实现评估",
        )?,
        session_price_basis_and_gap_assessment: bounded_required(
            &request.session_price_basis_and_gap_assessment,
            "交易日、价格口径与缺口评估",
        )?,
        corporate_action_decimal_order_and_hash_assessment: bounded_required(
            &request.corporate_action_decimal_order_and_hash_assessment,
            "公司行动、十进制、排序与哈希评估",
        )?,
        initial_allocation_and_availability_assessment: bounded_required(
            &request.initial_allocation_and_availability_assessment,
            "初始组合与可用时间评估",
        )?,
        zero_capability_assessment: bounded_required(
            &request.zero_capability_assessment,
            "零能力评估",
        )?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_implementation_constraints: bounded_required(
            &request.future_implementation_constraints,
            "未来实现约束",
        )?,
        reviewer_independent_from_registrar_and_complete_prior_chain: true,
        exact_current_stage_51_through_stage_105_binding_confirmed: request
            .exact_current_stage_51_through_stage_105_binding_confirmed,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_registrar_and_complete_prior_chain_confirmed,
        registration_and_specification_hashes_independently_reproduced_confirmed: request
            .registration_and_specification_hashes_independently_reproduced_confirmed,
        complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed:
            request.complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed,
        rebuilt_specification_exactly_matches_registered_specification_confirmed: request
            .rebuilt_specification_exactly_matches_registered_specification_confirmed,
        official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: request
            .official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed,
        subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: request
            .subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed,
        dividends_splits_and_price_bases_remain_separate_confirmed: request
            .dividends_splits_and_price_bases_remain_separate_confirmed,
        decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: request
            .decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed,
        initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed:
            request.initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed,
        conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed:
            request.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed,
        one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
            request.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed,
        future_output_untrusted_and_independent_validation_required_confirmed: request
            .future_output_untrusted_and_independent_validation_required_confirmed,
        no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed:
            request.no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
            request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_zero_capability_implementation_registration_confirmed: request
            .approval_only_opens_future_zero_capability_implementation_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        specification_independently_approved: approved,
        future_zero_capability_implementation_registration_eligible: approved,
        implementation_registered: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        input_mount_present: false,
        observation_materialized: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_or_metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&reviewable.source, &review)?;
    write_immutable_json(
        &review_root(state)
            .join(registration_id)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn independently_audit(
    reviewable: &IndependentlyReviewableControlledShadowObservationMaterializationSpecification,
) -> Result<ControlledShadowObservationMaterializationSpecificationIndependentAudit, String> {
    let registration = &reviewable.registration;
    let specification = &registration.specification;
    let registration_hash =
        fingerprint_without(registration, &["registration_id", "registration_sha256"])?;
    let specification_hash = fingerprint_without(specification, &["specification_sha256"])?;
    let registration_hash_independently_reproduced = registration_hash
        == registration.registration_sha256
        && registration.registration_id == registration_hash[..32];
    let specification_hash_independently_reproduced =
        specification_hash == specification.specification_sha256;
    let rebuilt = independently_rebuild_specification(&reviewable.source)?;
    let rebuilt_specification_exactly_matches_registration = rebuilt == *specification;
    let exact_current_stage_51_through_stage_105_binding_valid = registration.schema_version
        == STAGE_105_RECORD_SCHEMA_VERSION
        && registration.policy_version == STAGE_105_POLICY_VERSION
        && registration.status == STAGE_105_STATUS
        && registration.confirmations_complete
        && registration.specification_registered
        && registration.future_chain_external_specification_review_eligible
        && !registration.specification_review_completed
        && !registration.implementation_registration_eligible
        && registration.registrar_independent_from_stage_104_and_complete_prior_chain
        && !registration
            .excluded_prior_actor_ids
            .contains(&registration.registered_by)
        && registration.stage_104_review_id == reviewable.source.admission_review.review_id
        && registration.stage_104_review_sha256 == reviewable.source.admission_review.review_sha256
        && specification.stage_104_review_id == registration.stage_104_review_id
        && specification.stage_104_review_sha256 == registration.stage_104_review_sha256
        && registration_hash_independently_reproduced
        && specification_hash_independently_reproduced;
    let session_subject_spy_three_price_basis_and_gap_contract_valid = specification
        .allowed_price_bases
        == [
            "raw_unadjusted_price",
            "split_adjusted_price",
            "dividend_adjusted_price",
        ]
        && specification.official_market_session_count > 0
        && specification.subject_gap_is_explicit_not_synthetic
        && !specification.benchmark_gap_allowed
        && !specification.duplicate_row_allowed
        && !specification.out_of_window_row_allowed
        && !specification.forward_fill_allowed
        && !specification.interpolation_allowed
        && !specification.cross_price_basis_substitution_allowed;
    let corporate_action_decimal_order_hash_and_output_path_contract_valid = specification
        .dividends_splits_and_prices_separate
        && specification.canonical_corporate_action_schema
            == "symbol,date,action_kind:dividend|split,exact_source_fields,source_row_sha256,action_row_sha256"
        && specification.decimal_preservation_rule
            == "preserve_exact_stage_102_canonical_decimal_string_no_rounding_or_float_conversion"
        && specification.row_hash_rule == "sha256_rfc8785_canonical_json_excluding_row_sha256"
        && specification
            .future_output_relative_path_template
            .starts_with("observations/")
        && specification
            .future_output_relative_path_template
            .ends_with("/{specification_sha256}.json");
    let initial_shadow_allocation_and_availability_contract_valid = !specification
        .stage_88_initial_observation_output_sha256
        .is_empty()
        && !specification
            .initial_shadow_allocation_manifest_sha256
            .is_empty()
        && !specification.initial_shadow_allocation_recomputed
        && !specification.accounting_transition_applied
        && !specification.provider_publication_time_verified
        && specification.admitted_available_at_utc
            == reviewable.source.admission_review.admitted_available_at_utc
        && specification.availability_basis
            == reviewable.source.admission_review.availability_basis;
    let all_implementation_runtime_observation_store_feedback_order_broker_and_trading_authority_closed =
        authority_closed(&specification.authority_boundary)
            && registration_downstream_closed(registration)
            && specification.one_envelope_per_admitted_cycle
            && specification.create_once_required
            && !specification.overwrite_allowed
            && !specification.retroactive_backfill_allowed
            && !specification.in_place_correction_allowed
            && !specification.valuation_or_investment_semantics_allowed
            && specification.future_output_untrusted
            && specification.future_output_independent_validation_required;
    let checks = [
        (
            registration_hash_independently_reproduced,
            "Stage 105 registration hash mismatch",
        ),
        (
            specification_hash_independently_reproduced,
            "Stage 105 specification hash mismatch",
        ),
        (
            exact_current_stage_51_through_stage_105_binding_valid,
            "Stage 51-105 current binding invalid",
        ),
        (
            rebuilt_specification_exactly_matches_registration,
            "independent specification rebuild mismatch",
        ),
        (
            session_subject_spy_three_price_basis_and_gap_contract_valid,
            "session/subject/SPY/price-basis/gap contract invalid",
        ),
        (
            corporate_action_decimal_order_hash_and_output_path_contract_valid,
            "action/decimal/order/hash/output-path contract invalid",
        ),
        (
            initial_shadow_allocation_and_availability_contract_valid,
            "initial allocation or availability contract invalid",
        ),
        (
            all_implementation_runtime_observation_store_feedback_order_broker_and_trading_authority_closed,
            "zero-capability or downstream authority boundary is open",
        ),
    ];
    let mismatch_reasons = checks
        .iter()
        .filter_map(|(passed, reason)| (!passed).then_some((*reason).to_string()))
        .collect::<Vec<_>>();
    let mut audit = ControlledShadowObservationMaterializationSpecificationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        registration_id: registration.registration_id.clone(),
        registration_sha256: registration.registration_sha256.clone(),
        specification_sha256: specification.specification_sha256.clone(),
        registration_hash_independently_reproduced,
        specification_hash_independently_reproduced,
        exact_current_stage_51_through_stage_105_binding_valid,
        complete_specification_rebuilt_without_stage_105_builder: true,
        rebuilt_specification_exactly_matches_registration,
        session_subject_spy_three_price_basis_and_gap_contract_valid,
        corporate_action_decimal_order_hash_and_output_path_contract_valid,
        initial_shadow_allocation_and_availability_contract_valid,
        all_implementation_runtime_observation_store_feedback_order_broker_and_trading_authority_closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn independently_rebuild_specification(
    source: &AdmittedControlledShadowObservationInputForMaterializationSpecification,
) -> Result<ControlledShadowObservationMaterializationSpecification, String> {
    let review = &source.admission_review;
    let parser = &source.candidate.parser_output;
    let cycle = &source.candidate.cycle_claim;
    let mut value = ControlledShadowObservationMaterializationSpecification {
        schema_version: STAGE_105_SPEC_SCHEMA_VERSION.to_string(),
        specification_sha256: String::new(),
        materialization_protocol_version: MATERIALIZATION_PROTOCOL_VERSION.to_string(),
        stage_104_review_id: review.review_id.clone(),
        stage_104_review_sha256: review.review_sha256.clone(),
        stage_103_validation_id: review.stage_103_validation_id.clone(),
        stage_103_validation_sha256: review.stage_103_validation_sha256.clone(),
        stage_102_attempt_id: parser.claim.attempt_id.clone(),
        stage_102_result_sha256: review.stage_102_result_sha256.clone(),
        stage_102_output_sha256: review.stage_102_output_sha256.clone(),
        stage_101_claim_sha256: review.stage_101_claim_sha256.clone(),
        stage_101_input_manifest_sha256: review.stage_101_input_manifest_sha256.clone(),
        cycle_claim_id: cycle.cycle_claim_id.clone(),
        cycle_claim_sha256: cycle.cycle_claim_sha256.clone(),
        stage_88_initial_observation_output_sha256: cycle.stage_88_output_sha256.clone(),
        initial_shadow_allocation_manifest_sha256: cycle.initialization_manifest_sha256.clone(),
        subject_symbols: review.subject_symbols.clone(),
        benchmark_symbol: review.benchmark_symbol.clone(),
        window_start_date: review.window_start_date,
        window_end_date: review.window_end_date,
        earliest_market_session_date: review.earliest_market_session_date,
        latest_market_session_date: review.latest_market_session_date,
        official_market_session_count: review.official_market_session_count,
        admitted_available_at_utc: review.admitted_available_at_utc,
        availability_basis: review.availability_basis.clone(),
        provider_publication_time_verified: false,
        exact_admitted_output_is_only_input: true,
        allowed_price_bases: vec![
            "raw_unadjusted_price".to_string(),
            "split_adjusted_price".to_string(),
            "dividend_adjusted_price".to_string(),
        ],
        canonical_session_schema: "date,session_kind:regular|early_close,close_time_et".to_string(),
        canonical_price_observation_schema: "symbol,date,price_basis,close_decimal,volume_integer|null,source_request_sha256,source_row_sha256,observation_row_sha256".to_string(),
        canonical_explicit_gap_schema: "symbol,date,price_basis,reason:missing_subject_row_no_fill,source_manifest_sha256,gap_row_sha256".to_string(),
        canonical_corporate_action_schema: "symbol,date,action_kind:dividend|split,exact_source_fields,source_row_sha256,action_row_sha256".to_string(),
        canonical_observation_envelope_schema: "specification_sha256,stage_104_review_sha256,cycle_claim_sha256,admitted_available_at_utc,sessions,prices,explicit_gaps,dividends,splits,initial_shadow_allocation_binding,output_sha256,untrusted:true".to_string(),
        canonical_order: "session_date_ascending_then_symbol_ascending_then_price_basis_lexicographic;actions_by_date_symbol_kind".to_string(),
        decimal_preservation_rule: "preserve_exact_stage_102_canonical_decimal_string_no_rounding_or_float_conversion".to_string(),
        row_hash_rule: "sha256_rfc8785_canonical_json_excluding_row_sha256".to_string(),
        future_output_relative_path_template: format!(
            "observations/{}/{{specification_sha256}}.json",
            cycle.cycle_claim_id
        ),
        one_envelope_per_admitted_cycle: true,
        create_once_required: true,
        overwrite_allowed: false,
        retroactive_backfill_allowed: false,
        in_place_correction_allowed: false,
        subject_gap_is_explicit_not_synthetic: true,
        forward_fill_allowed: false,
        interpolation_allowed: false,
        cross_price_basis_substitution_allowed: false,
        benchmark_gap_allowed: false,
        duplicate_row_allowed: false,
        out_of_window_row_allowed: false,
        dividends_splits_and_prices_separate: true,
        initial_shadow_allocation_recomputed: false,
        accounting_transition_applied: false,
        valuation_or_investment_semantics_allowed: false,
        future_output_untrusted: true,
        future_output_independent_validation_required: true,
        authority_boundary: closed_authority_boundary(),
    };
    value.specification_sha256 = fingerprint_without(&value, &["specification_sha256"])?;
    Ok(value)
}

fn closed_authority_boundary() -> ZeroCapabilityObservationMaterializationAuthorityBoundary {
    ZeroCapabilityObservationMaterializationAuthorityBoundary {
        implementation_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        input_mount_present: false,
        environment_inheritance_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        observation_envelope_created: false,
        ledger_creation_allowed: false,
        position_write_allowed: false,
        performance_metric_write_allowed: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        training_feedback_allowed: false,
        reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    }
}

fn authority_closed(value: &ZeroCapabilityObservationMaterializationAuthorityBoundary) -> bool {
    value == &closed_authority_boundary()
}

fn registration_downstream_closed(
    value: &ControlledShadowObservationMaterializationSpecificationRegistration,
) -> bool {
    !value.specification_review_completed
        && !value.implementation_registration_eligible
        && !value.observation_materialized
        && !value.ledger_created
        && !value.position_written
        && !value.performance_metric_written
        && !value.model_or_metric_store_written
        && !value.training_feedback_authorized
        && !value.reward_authorized
        && !value.order_generation_authorized
        && !value.broker_access_authorized
        && !value.trading_authorized
}

fn request_checks(
    value: &ReviewControlledShadowObservationMaterializationSpecificationRequest,
) -> bool {
    value.exact_current_stage_51_through_stage_105_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.registration_and_specification_hashes_independently_reproduced_confirmed
        && value.complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed
        && value.rebuilt_specification_exactly_matches_registered_specification_confirmed
        && value.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && value.subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed
        && value.dividends_splits_and_price_bases_remain_separate_confirmed
        && value.decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed
        && value.initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed
        && value.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed
        && value.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed
        && value.future_output_untrusted_and_independent_validation_required_confirmed
        && value.no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_zero_capability_implementation_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn review_checks(
    value: &ControlledShadowObservationMaterializationSpecificationReviewRecord,
) -> bool {
    value.exact_current_stage_51_through_stage_105_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.registration_and_specification_hashes_independently_reproduced_confirmed
        && value.complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed
        && value.rebuilt_specification_exactly_matches_registered_specification_confirmed
        && value.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && value.subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed
        && value.dividends_splits_and_price_bases_remain_separate_confirmed
        && value.decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed
        && value.initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed
        && value.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed
        && value.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed
        && value.future_output_untrusted_and_independent_validation_required_confirmed
        && value.no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_zero_capability_implementation_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_review(
    source: &AdmittedControlledShadowObservationInputForMaterializationSpecification,
    review: &ControlledShadowObservationMaterializationSpecificationReviewRecord,
) -> Result<(), String> {
    let reviewable =
        IndependentlyReviewableControlledShadowObservationMaterializationSpecification {
            source: source.clone(),
            registration: review.registration.clone(),
        };
    let expected_audit = independently_audit(&reviewable)?;
    let approved = review.verdict
        == ControlledShadowObservationMaterializationSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityObservationMaterializationImplementationRegistration;
    let texts_valid = [
        &review.rationale,
        &review.binding_and_second_implementation_assessment,
        &review.session_price_basis_and_gap_assessment,
        &review.corporate_action_decimal_order_and_hash_assessment,
        &review.initial_allocation_and_availability_assessment,
        &review.zero_capability_assessment,
        &review.known_limitations,
        &review.future_implementation_constraints,
    ]
    .iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let no_downstream = !review.implementation_registered
        && !review.executable_artifact_present
        && !review.callable_entrypoint_present
        && !review.runtime_present
        && !review.input_mount_present
        && !review.observation_materialized
        && !review.ledger_created
        && !review.position_written
        && !review.performance_metric_written
        && !review.model_or_metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let valid = review.schema_version == REVIEW_SCHEMA_VERSION
        && review.policy_version == POLICY_VERSION
        && valid_id(&review.review_id)
        && valid_sha256(&review.review_sha256)
        && review.review_id == review.review_sha256[..32]
        && review.review_sha256 == fingerprint_without(review, &["review_id", "review_sha256"])?
        && review.independent_audit == expected_audit
        && review.independent_audit.mismatch_reasons.is_empty()
        && sorted_unique(&review.excluded_prior_actor_ids)
        && !review
            .excluded_prior_actor_ids
            .contains(&review.reviewer_id)
        && review
            .excluded_prior_actor_ids
            .contains(&review.registration.registered_by)
        && review.reviewer_independent_from_registrar_and_complete_prior_chain
        && texts_valid
        && (!approved || review_checks(review))
        && review.specification_independently_approved == approved
        && review.future_zero_capability_implementation_registration_eligible == approved
        && no_downstream;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 106 观察物化规格独立复核无效、漂移或越权".to_string())
}

fn validate_request(
    registration_id: &str,
    value: &ReviewControlledShadowObservationMaterializationSpecificationRequest,
) -> Result<(), String> {
    if !valid_id(registration_id)
        || !valid_sha256(&value.expected_registration_sha256)
        || !valid_sha256(&value.expected_specification_sha256)
        || !valid_sha256(&value.expected_independent_audit_sha256)
        || value
            .expected_previous_review_id
            .as_ref()
            .is_some_and(|item| !valid_id(item))
        || value
            .expected_previous_review_sha256
            .as_ref()
            .is_some_and(|item| !valid_sha256(item))
        || value.expected_previous_review_id.is_some()
            != value.expected_previous_review_sha256.is_some()
    {
        return Err("Stage 106 请求 ID 或摘要无效".to_string());
    }
    Ok(())
}

fn expected_excluded_actor_ids(
    registration: &ControlledShadowObservationMaterializationSpecificationRegistration,
    prior_reviews: &[ControlledShadowObservationMaterializationSpecificationReviewRecord],
) -> Vec<String> {
    let mut values = registration.excluded_prior_actor_ids.clone();
    values.push(registration.registered_by.clone());
    values.extend(prior_reviews.iter().map(|value| value.reviewer_id.clone()));
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

async fn read_reviews(
    state: &AppState,
    reviewable: &IndependentlyReviewableControlledShadowObservationMaterializationSpecification,
) -> Result<Vec<ControlledShadowObservationMaterializationSpecificationReviewRecord>, String> {
    let directory = review_root(state).join(&reviewable.registration.registration_id);
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
        let path = entry.path();
        if entry
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("json")
        {
            let review: ControlledShadowObservationMaterializationSpecificationReviewRecord =
                serde_json::from_slice(
                    &tokio::fs::read(&path)
                        .await
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(review.review_id.as_str())
            {
                return Err("Stage 106 复核文件名与记录 ID 不一致".to_string());
            }
            validate_review(&reviewable.source, &review)?;
            reviews.push(review);
        }
    }
    order_review_chain(reviews, &reviewable.registration)
}

fn order_review_chain(
    reviews: Vec<ControlledShadowObservationMaterializationSpecificationReviewRecord>,
    registration: &ControlledShadowObservationMaterializationSpecificationRegistration,
) -> Result<Vec<ControlledShadowObservationMaterializationSpecificationReviewRecord>, String> {
    if reviews.is_empty() {
        return Ok(reviews);
    }
    let mut by_id = BTreeMap::new();
    let mut hashes = BTreeSet::new();
    for review in reviews {
        if review.registration != *registration
            || !hashes.insert(review.review_sha256.clone())
            || by_id.insert(review.review_id.clone(), review).is_some()
        {
            return Err("Stage 106 复核重复或规格绑定不一致".to_string());
        }
    }
    let roots = by_id
        .values()
        .filter(|review| review.previous_review_id.is_none())
        .map(|review| review.review_id.clone())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("Stage 106 复核链必须恰有一个根".to_string());
    }
    let mut ordered = Vec::with_capacity(by_id.len());
    let mut next_id = Some(roots[0].clone());
    while let Some(current_id) = next_id {
        let review = by_id
            .remove(&current_id)
            .ok_or_else(|| "Stage 106 复核链断裂或包含循环".to_string())?;
        let prior = ordered.last();
        if review.previous_review_id
            != prior.map(
                |value: &ControlledShadowObservationMaterializationSpecificationReviewRecord| {
                    value.review_id.clone()
                },
            )
            || review.previous_review_sha256 != prior.map(|value| value.review_sha256.clone())
        {
            return Err("Stage 106 前序复核绑定无效".to_string());
        }
        let expected = expected_excluded_actor_ids(registration, &ordered);
        if review.excluded_prior_actor_ids != expected || expected.contains(&review.reviewer_id) {
            return Err("Stage 106 复核角色隔离无效".to_string());
        }
        let successors = by_id
            .values()
            .filter(|candidate| candidate.previous_review_id.as_deref() == Some(&current_id))
            .map(|candidate| candidate.review_id.clone())
            .collect::<Vec<_>>();
        if successors.len() > 1 {
            return Err("Stage 106 复核链发生分叉".to_string());
        }
        if review.future_zero_capability_implementation_registration_eligible
            && !successors.is_empty()
        {
            return Err("Stage 106 已批准复核必须是终态".to_string());
        }
        ordered.push(review);
        next_id = successors.into_iter().next();
    }
    if !by_id.is_empty() {
        return Err("Stage 106 复核链不连通".to_string());
    }
    Ok(ordered)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 106 指纹载荷无效".to_string())?;
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

fn review_root(state: &AppState) -> PathBuf {
    decision_root(state).join(
        "controlled-shadow-first-natural-cycle-observation-materialization-specification-reviews",
    )
}

struct ReviewLock(PathBuf);

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, registration_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-observation-materialization-specification-review-{registration_id}.lock"
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
        .map_err(|_| "已有同一 Stage 106 规格复核正在进行".to_string())?;
    Ok(ReviewLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ReviewControlledShadowObservationMaterializationSpecificationRequest {
        ReviewControlledShadowObservationMaterializationSpecificationRequest {
            expected_previous_review_id: None,
            expected_previous_review_sha256: None,
            expected_registration_sha256: "1".repeat(64),
            expected_specification_sha256: "2".repeat(64),
            expected_independent_audit_sha256: "3".repeat(64),
            verdict: ControlledShadowObservationMaterializationSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityObservationMaterializationImplementationRegistration,
            rationale: "第二实现独立重建并逐字段核对通过。".to_string(),
            binding_and_second_implementation_assessment: "Stage 51–105 绑定与摘要一致。".to_string(),
            session_price_basis_and_gap_assessment: "官方交易日、三价格口径及显式缺口合同完整。".to_string(),
            corporate_action_decimal_order_and_hash_assessment: "公司行动、十进制、排序和哈希规则完整。".to_string(),
            initial_allocation_and_availability_assessment: "初始组合仅绑定且可用时间保持保守。".to_string(),
            zero_capability_assessment: "实现、runtime、观察和全部下游权限关闭。".to_string(),
            known_limitations: "尚无真实规格记录和自然前向观察。".to_string(),
            future_implementation_constraints: "通过后只允许另行登记零能力实现。".to_string(),
            exact_current_stage_51_through_stage_105_binding_confirmed: true,
            reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
            registration_and_specification_hashes_independently_reproduced_confirmed: true,
            complete_specification_rebuilt_from_current_stage_104_source_without_stage_105_builder_confirmed: true,
            rebuilt_specification_exactly_matches_registered_specification_confirmed: true,
            official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
            subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
            dividends_splits_and_price_bases_remain_separate_confirmed: true,
            decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: true,
            initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: true,
            conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: true,
            one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
            future_output_untrusted_and_independent_validation_required_confirmed: true,
            no_implementation_artifact_entrypoint_runtime_mount_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_zero_capability_implementation_registration_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn approval_requires_every_stage_106_confirmation() {
        let mut value = request();
        assert!(request_checks(&value));
        value.rebuilt_specification_exactly_matches_registered_specification_confirmed = false;
        assert!(!request_checks(&value));
    }

    #[test]
    fn zero_capability_boundary_is_completely_closed() {
        assert!(authority_closed(&closed_authority_boundary()));
        let mut value = closed_authority_boundary();
        value.observation_envelope_created = true;
        assert!(!authority_closed(&value));
    }

    #[test]
    fn review_fingerprint_binds_verdict() {
        #[derive(Serialize)]
        struct Fixture {
            review_id: String,
            review_sha256: String,
            verdict: String,
        }
        let approved = fingerprint_without(
            &Fixture {
                review_id: String::new(),
                review_sha256: String::new(),
                verdict: "approved".to_string(),
            },
            &["review_id", "review_sha256"],
        )
        .unwrap();
        let rejected = fingerprint_without(
            &Fixture {
                review_id: String::new(),
                review_sha256: String::new(),
                verdict: "rejected".to_string(),
            },
            &["review_id", "review_sha256"],
        )
        .unwrap();
        assert_ne!(approved, rejected);
    }

    #[test]
    fn ids_and_hashes_are_strictly_content_addressed() {
        assert!(valid_id(&"a".repeat(32)));
        assert!(!valid_id(&"a".repeat(31)));
        assert!(valid_sha256(&"b".repeat(64)));
        assert!(!valid_sha256(&"g".repeat(64)));
    }
}
