//! Stage 116 chain-external review of one Stage 115 observation-to-ledger transition specification.
//!
//! The reviewer independently rebuilds the complete specification from the current Stage 114
//! admitted evidence without calling the Stage 115 builder. Approval opens only a future
//! zero-capability implementation-registration gate. This module cannot create a ledger or event,
//! infer an opening portfolio, write cash/positions, calculate NAV/performance, train a model,
//! generate an order, access a broker or trade.

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
use tokio::io::AsyncWriteExt;
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_evidence_admission_reviews::AdmittedControlledShadowObservationEvidence;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_specifications::{
    ControlledShadowObservationLedgerTransitionSpecification,
    ControlledShadowObservationLedgerTransitionSpecificationRegistration,
    IndependentlyReviewableControlledShadowObservationLedgerTransitionSpecification,
    ObservationLedgerEventMappingRules, OpeningPortfolioPrerequisite,
    ZeroCapabilityObservationLedgerTransitionAuthorityBoundary,
    independently_reviewable_observation_ledger_transition_specifications,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-specification-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-specification-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-specification-independent-audit-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-ledger-transition-specification-chain-external-review-v1-zero-capability";
const STAGE_115_RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-specification-registration-v1";
const STAGE_115_SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-specification-v1";
const STAGE_115_POLICY_VERSION: &str = "hone-controlled-shadow-observation-ledger-transition-specification-create-once-v1-zero-capability";
const TRANSITION_PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-v1-specification-only-not-executable";
const STAGE_115_STATUS: &str = "registered_waiting_stage_116_independent_specification_review";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict {
    ApprovedForFutureZeroCapabilityLedgerTransitionImplementationRegistration,
    ChangesRequiredRebuildLedgerTransitionSpecification,
    RejectedLedgerTransitionSpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowObservationLedgerTransitionSpecificationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_registration_sha256: String,
    expected_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict,
    rationale: String,
    binding_and_second_implementation_assessment: String,
    opening_portfolio_prerequisite_assessment: String,
    price_basis_gap_and_nav_assessment: String,
    corporate_action_and_double_count_assessment: String,
    decimal_idempotency_correction_and_order_assessment: String,
    zero_capability_assessment: String,
    known_limitations: String,
    future_implementation_constraints: String,
    exact_current_stage_51_through_stage_115_binding_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    registration_and_specification_hashes_independently_reproduced_confirmed: bool,
    complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed:
        bool,
    rebuilt_specification_exactly_matches_registered_specification_confirmed: bool,
    stage_88_binding_not_opening_positions_confirmed: bool,
    separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed: bool,
    raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed: bool,
    explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed: bool,
    dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed: bool,
    exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed: bool,
    corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed: bool,
    conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: bool,
    no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed: bool,
    no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    approval_only_opens_future_zero_capability_implementation_registration_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub specification_sha256: String,
    pub registration_hash_independently_reproduced: bool,
    pub specification_hash_independently_reproduced: bool,
    pub exact_current_stage_51_through_stage_115_binding_valid: bool,
    pub complete_specification_rebuilt_without_stage_115_builder: bool,
    pub rebuilt_specification_exactly_matches_registration: bool,
    pub opening_portfolio_prerequisite_and_no_invention_contract_valid: bool,
    pub raw_price_adjusted_price_gap_and_nav_contract_valid: bool,
    pub corporate_action_no_double_count_contract_valid: bool,
    pub decimal_idempotency_append_only_correction_and_double_entry_contract_valid: bool,
    pub availability_and_provider_time_contract_valid: bool,
    pub all_implementation_ledger_financial_feedback_order_broker_and_trading_authority_closed:
        bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub registration: ControlledShadowObservationLedgerTransitionSpecificationRegistration,
    pub independent_audit: ControlledShadowObservationLedgerTransitionSpecificationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict,
    pub rationale: String,
    pub binding_and_second_implementation_assessment: String,
    pub opening_portfolio_prerequisite_assessment: String,
    pub price_basis_gap_and_nav_assessment: String,
    pub corporate_action_and_double_count_assessment: String,
    pub decimal_idempotency_correction_and_order_assessment: String,
    pub zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_implementation_constraints: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_115_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub registration_and_specification_hashes_independently_reproduced_confirmed: bool,
    pub complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed:
        bool,
    pub rebuilt_specification_exactly_matches_registered_specification_confirmed: bool,
    pub stage_88_binding_not_opening_positions_confirmed: bool,
    pub separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed: bool,
    pub raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed: bool,
    pub explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed: bool,
    pub dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed: bool,
    pub exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed: bool,
    pub corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed:
        bool,
    pub conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed:
        bool,
    pub no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed:
        bool,
    pub no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub approval_only_opens_future_zero_capability_implementation_registration_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub specification_independently_approved: bool,
    pub future_zero_capability_implementation_registration_eligible: bool,
    pub implementation_registered: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub admitted_evidence_mounted_or_read_for_execution: bool,
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
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationReviewItem {
    pub registration: ControlledShadowObservationLedgerTransitionSpecificationRegistration,
    pub current_independent_audit:
        ControlledShadowObservationLedgerTransitionSpecificationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<ControlledShadowObservationLedgerTransitionSpecificationReviewRecord>,
    pub review_eligible: bool,
    pub future_zero_capability_implementation_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_endpoint_available: bool,
    pub items: Vec<ControlledShadowObservationLedgerTransitionSpecificationReviewItem>,
    pub specification_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_zero_capability_implementation_registration_eligible_count: usize,
    pub opening_portfolio_snapshot_missing_count: usize,
    pub review_status: String,
    pub implementation_registered: bool,
    pub ledger_created: bool,
    pub ledger_event_written: bool,
    pub nav_or_performance_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationReviewReadinessSummary {
    pub specification_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_zero_capability_implementation_registration_eligible_count: usize,
    pub opening_portfolio_snapshot_missing_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentlyApprovedControlledShadowObservationLedgerTransitionSpecification {
    pub review: ControlledShadowObservationLedgerTransitionSpecificationReviewRecord,
}

pub(crate) async fn handle_get_controlled_shadow_observation_ledger_transition_specification_reviews(
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
            warn!(%error, "observation ledger transition specification independent review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察证据到账本转换规格独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_observation_ledger_transition_specification(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(registration_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowObservationLedgerTransitionSpecificationRequest>,
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

pub(crate) async fn controlled_shadow_observation_ledger_transition_specification_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionSpecificationReviewReadinessSummary, String>
{
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowObservationLedgerTransitionSpecificationReviewReadinessSummary {
            specification_count: value.specification_count,
            review_eligible_count: value.review_eligible_count,
            reviewed_count: value.reviewed_count,
            independently_approved_count: value.independently_approved_count,
            changes_required_or_rejected_count: value.changes_required_or_rejected_count,
            future_zero_capability_implementation_registration_eligible_count: value
                .future_zero_capability_implementation_registration_eligible_count,
            opening_portfolio_snapshot_missing_count: value
                .opening_portfolio_snapshot_missing_count,
            review_status: value.review_status,
        },
    )
}

pub(crate) async fn independently_approved_observation_ledger_transition_specifications_for_implementation_registration(
    state: &AppState,
) -> Result<
    Vec<IndependentlyApprovedControlledShadowObservationLedgerTransitionSpecification>,
    String,
> {
    let reviewables =
        independently_reviewable_observation_ledger_transition_specifications(state).await?;
    let mut approved = Vec::new();
    for reviewable in reviewables {
        let expected_audit = independently_audit(&reviewable)?;
        if !expected_audit.mismatch_reasons.is_empty() {
            return Err("Stage 116 当前独立审计未通过，Stage 117 晋级关闭".to_string());
        }
        let reviews = read_reviews(state, &reviewable).await?;
        let Some(review) = reviews.last().cloned() else {
            continue;
        };
        validate_review(&reviewable.source, &review)?;
        if review.independent_audit != expected_audit
            || review.registration != reviewable.registration
        {
            return Err("Stage 116 已批准复核与当前 Stage 114–115 绑定漂移".to_string());
        }
        if review.future_zero_capability_implementation_registration_eligible {
            approved.push(
                IndependentlyApprovedControlledShadowObservationLedgerTransitionSpecification {
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
) -> Result<ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry, String> {
    let reviewables =
        independently_reviewable_observation_ledger_transition_specifications(state).await?;
    let mut items = Vec::new();
    for reviewable in reviewables {
        let audit = independently_audit(&reviewable)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 115 账本转换规格独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &reviewable).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|review| {
            review.registration != reviewable.registration || review.independent_audit != audit
        }) {
            return Err("Stage 116 复核绑定或独立审计已经变化".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|review| {
            review.future_zero_capability_implementation_registration_eligible
        });
        items.push(
            ControlledShadowObservationLedgerTransitionSpecificationReviewItem {
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
        "waiting_stage_115_observation_ledger_transition_specification"
    } else if independently_approved_count > 0 {
        "ledger_transition_specification_independently_approved_only_for_future_zero_capability_implementation_registration"
    } else if review_eligible_count > 0 {
        "ledger_transition_specification_ready_for_chain_external_review"
    } else {
        "ledger_transition_specification_review_blocked_fail_closed"
    };
    Ok(
        ControlledShadowObservationLedgerTransitionSpecificationReviewRegistry {
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
            opening_portfolio_snapshot_missing_count: specification_count,
            review_status: review_status.to_string(),
            implementation_registered: false,
            ledger_created: false,
            ledger_event_written: false,
            nav_or_performance_written: false,
            training_or_rl_feedback_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "Stage 116 由完整责任链外角色使用第二实现，从当前 Stage 114 准入证据完整重建 Stage 115 会计转换规格。批准只开放未来 Stage 117 零能力实现登记；opening portfolio snapshot 仍缺失，且没有实现、工件、runtime、账本、事件、持仓、现金、NAV/绩效、训练/RL、订单、券商或交易能力。".to_string(),
        },
    )
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    registration_id: &str,
    request: ReviewControlledShadowObservationLedgerTransitionSpecificationRequest,
) -> Result<ControlledShadowObservationLedgerTransitionSpecificationReviewRecord, String> {
    validate_request(registration_id, &request)?;
    let _lock = acquire_lock(state, registration_id).await?;
    let reviewable = independently_reviewable_observation_ledger_transition_specifications(state)
        .await?
        .into_iter()
        .find(|value| value.registration.registration_id == registration_id)
        .ok_or_else(|| "当前没有可独立复核的 Stage 115 账本转换规格".to_string())?;
    if request.expected_registration_sha256 != reviewable.registration.registration_sha256
        || request.expected_specification_sha256
            != reviewable.registration.specification.specification_sha256
    {
        return Err("Stage 115 登记或规格摘要已变化".to_string());
    }
    let audit = independently_audit(&reviewable)?;
    if request.expected_independent_audit_sha256 != audit.audit_sha256
        || !audit.mismatch_reasons.is_empty()
    {
        return Err("Stage 116 独立审计摘要不匹配或未通过".to_string());
    }
    let prior_reviews = read_reviews(state, &reviewable).await?;
    let previous = prior_reviews.last();
    if request.expected_previous_review_id.as_deref()
        != previous.map(|value| value.review_id.as_str())
        || request.expected_previous_review_sha256.as_deref()
            != previous.map(|value| value.review_sha256.as_str())
    {
        return Err("Stage 116 前序复核绑定已变化".to_string());
    }
    if previous
        .is_some_and(|value| value.future_zero_capability_implementation_registration_eligible)
    {
        return Err("Stage 116 已批准终态不得追加复核".to_string());
    }
    let excluded = expected_excluded_actor_ids(&reviewable.registration, &prior_reviews);
    if reviewer_id.trim().is_empty() || excluded.iter().any(|value| value == reviewer_id) {
        return Err(
            "Stage 116 reviewer 必须独立于 Stage 115 registrar 与完整既有责任链".to_string(),
        );
    }
    let approved = request.verdict
        == ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityLedgerTransitionImplementationRegistration;
    if approved && !request_checks(&request) {
        return Err("Stage 116 批准需要完成全部独立会计规格复核确认".to_string());
    }
    let mut review = ControlledShadowObservationLedgerTransitionSpecificationReviewRecord {
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
        opening_portfolio_prerequisite_assessment: bounded_required(
            &request.opening_portfolio_prerequisite_assessment,
            "开仓组合前置条件评估",
        )?,
        price_basis_gap_and_nav_assessment: bounded_required(
            &request.price_basis_gap_and_nav_assessment,
            "价格口径、缺口与净值评估",
        )?,
        corporate_action_and_double_count_assessment: bounded_required(
            &request.corporate_action_and_double_count_assessment,
            "公司行动与双计风险评估",
        )?,
        decimal_idempotency_correction_and_order_assessment: bounded_required(
            &request.decimal_idempotency_correction_and_order_assessment,
            "十进制、幂等、纠错与排序评估",
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
        exact_current_stage_51_through_stage_115_binding_confirmed: request
            .exact_current_stage_51_through_stage_115_binding_confirmed,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_registrar_and_complete_prior_chain_confirmed,
        registration_and_specification_hashes_independently_reproduced_confirmed: request
            .registration_and_specification_hashes_independently_reproduced_confirmed,
        complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed:
            request.complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed,
        rebuilt_specification_exactly_matches_registered_specification_confirmed: request
            .rebuilt_specification_exactly_matches_registered_specification_confirmed,
        stage_88_binding_not_opening_positions_confirmed: request
            .stage_88_binding_not_opening_positions_confirmed,
        separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed: request
            .separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed,
        raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed: request
            .raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed,
        explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed: request
            .explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed,
        dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed: request
            .dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed,
        exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed: request
            .exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed,
        corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed:
            request.corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed,
        conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed:
            request.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed,
        no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed:
            request.no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed,
        no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
            request.no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed,
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
        admitted_evidence_mounted_or_read_for_execution: false,
        opening_portfolio_snapshot_admitted: false,
        ledger_created: false,
        ledger_event_written: false,
        position_written: false,
        cash_written: false,
        nav_or_performance_written: false,
        model_or_metric_store_written: false,
        training_or_rl_feedback_authorized: false,
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
    reviewable: &IndependentlyReviewableControlledShadowObservationLedgerTransitionSpecification,
) -> Result<ControlledShadowObservationLedgerTransitionSpecificationIndependentAudit, String> {
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
    let exact_current_stage_51_through_stage_115_binding_valid = registration.schema_version
        == STAGE_115_RECORD_SCHEMA_VERSION
        && registration.policy_version == STAGE_115_POLICY_VERSION
        && registration.status == STAGE_115_STATUS
        && registration.confirmations_complete
        && registration.specification_registered
        && registration.future_chain_external_specification_review_eligible
        && !registration.specification_review_completed
        && !registration.implementation_registration_eligible
        && registration.registrar_independent_from_stage_114_and_complete_prior_chain
        && !registration
            .excluded_prior_actor_ids
            .contains(&registration.registered_by)
        && registration.stage_114_review_id == reviewable.source.admission_review.review_id
        && registration.stage_114_review_sha256 == reviewable.source.admission_review.review_sha256
        && specification.stage_114_review_id == registration.stage_114_review_id
        && specification.stage_114_review_sha256 == registration.stage_114_review_sha256
        && registration_hash_independently_reproduced
        && specification_hash_independently_reproduced;
    let opening_portfolio_prerequisite_and_no_invention_contract_valid = specification
        .opening_portfolio_prerequisite
        == independent_opening_portfolio_prerequisite()
        && specification
            .mapping_rules
            .financial_event_type_allowlist_before_opening_snapshot
            .is_empty()
        && !specification.financial_postings_currently_eligible
        && !specification.nav_or_performance_currently_eligible;
    let rules = &specification.mapping_rules;
    let raw_price_adjusted_price_gap_and_nav_contract_valid = rules.security_valuation_price_basis
        == "raw_unadjusted_price_only_after_opening_positions_are_separately_admitted"
        && rules.benchmark_total_return_price_basis
            == "dividend_adjusted_price_for_non_accounting_SPY_total_return_comparison_only"
        && rules
            .split_adjusted_price_usage
            .contains("never_a_portfolio_accounting_mark")
        && rules
            .dividend_adjusted_price_usage
            .contains("never_a_security_accounting_mark")
        && rules.explicit_gap_rule.contains("no_fill")
        && rules.explicit_gap_rule.contains("fail_closed")
        && rules
            .nav_completeness_rule
            .contains("no_NAV_or_performance_fact");
    let corporate_action_no_double_count_contract_valid = rules
        .dividend_rule
        .contains("notice_only_until_opening_position")
        && rules.dividend_rule.contains("never_combine")
        && rules
            .split_rule
            .contains("notice_only_until_opening_position")
        && rules.split_rule.contains("quantity_adjustment_precedes");
    let decimal_idempotency_append_only_correction_and_double_entry_contract_valid =
        specification.mapping_rules == independent_mapping_rules()
            && specification.create_once_required
            && specification.append_only_required
            && !specification.overwrite_allowed
            && !specification.retroactive_backfill_allowed
            && !specification.in_place_correction_allowed
            && specification
                .mapping_rules
                .correction_rule
                .contains("superseding_or_reversal_events")
            && specification
                .mapping_rules
                .double_entry_balance_rule
                .contains("debits_must_exactly_equal_credits");
    let availability_and_provider_time_contract_valid = !specification
        .provider_publication_time_verified
        && specification.admitted_available_at_utc
            == reviewable.source.admission_review.admitted_available_at_utc
        && specification.exact_stage_114_admitted_envelope_is_only_observation_input;
    let all_implementation_ledger_financial_feedback_order_broker_and_trading_authority_closed =
        specification.authority_boundary == independent_closed_authority_boundary()
            && registration_downstream_closed(registration)
            && specification.future_independent_specification_review_required;
    let checks = [
        (
            registration_hash_independently_reproduced,
            "Stage 115 registration hash mismatch",
        ),
        (
            specification_hash_independently_reproduced,
            "Stage 115 specification hash mismatch",
        ),
        (
            exact_current_stage_51_through_stage_115_binding_valid,
            "Stage 51-115 current binding invalid",
        ),
        (
            rebuilt_specification_exactly_matches_registration,
            "independent specification rebuild mismatch",
        ),
        (
            opening_portfolio_prerequisite_and_no_invention_contract_valid,
            "opening portfolio prerequisite or no-invention contract invalid",
        ),
        (
            raw_price_adjusted_price_gap_and_nav_contract_valid,
            "price basis, gap or NAV fail-closed contract invalid",
        ),
        (
            corporate_action_no_double_count_contract_valid,
            "corporate-action no-double-count contract invalid",
        ),
        (
            decimal_idempotency_append_only_correction_and_double_entry_contract_valid,
            "decimal/idempotency/append-only/correction/double-entry contract invalid",
        ),
        (
            availability_and_provider_time_contract_valid,
            "availability or provider-time contract invalid",
        ),
        (
            all_implementation_ledger_financial_feedback_order_broker_and_trading_authority_closed,
            "zero-capability or downstream authority boundary is open",
        ),
    ];
    let mismatch_reasons = checks
        .iter()
        .filter_map(|(passed, reason)| (!passed).then_some((*reason).to_string()))
        .collect::<Vec<_>>();
    let mut audit = ControlledShadowObservationLedgerTransitionSpecificationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        registration_id: registration.registration_id.clone(),
        registration_sha256: registration.registration_sha256.clone(),
        specification_sha256: specification.specification_sha256.clone(),
        registration_hash_independently_reproduced,
        specification_hash_independently_reproduced,
        exact_current_stage_51_through_stage_115_binding_valid,
        complete_specification_rebuilt_without_stage_115_builder: true,
        rebuilt_specification_exactly_matches_registration,
        opening_portfolio_prerequisite_and_no_invention_contract_valid,
        raw_price_adjusted_price_gap_and_nav_contract_valid,
        corporate_action_no_double_count_contract_valid,
        decimal_idempotency_append_only_correction_and_double_entry_contract_valid,
        availability_and_provider_time_contract_valid,
        all_implementation_ledger_financial_feedback_order_broker_and_trading_authority_closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn independently_rebuild_specification(
    source: &AdmittedControlledShadowObservationEvidence,
) -> Result<ControlledShadowObservationLedgerTransitionSpecification, String> {
    let envelope = &source.envelope;
    let validation = &source.candidate.validation;
    let result = &source.candidate.result;
    let claim = &source.candidate.claim;
    let review = &source.admission_review;
    let earliest_market_session_date = envelope
        .sessions
        .iter()
        .map(|value| value.date)
        .min()
        .ok_or_else(|| "Stage 116 envelope 没有官方交易日".to_string())?;
    let latest_market_session_date = envelope
        .sessions
        .iter()
        .map(|value| value.date)
        .max()
        .ok_or_else(|| "Stage 116 envelope 没有官方交易日".to_string())?;
    let subject_symbols = independently_collect_subject_symbols(source);
    if subject_symbols.is_empty() {
        return Err("Stage 116 envelope 没有可识别的非 SPY 标的".to_string());
    }
    let mut specification = ControlledShadowObservationLedgerTransitionSpecification {
        schema_version: STAGE_115_SPEC_SCHEMA_VERSION.to_string(),
        specification_sha256: String::new(),
        transition_protocol_version: TRANSITION_PROTOCOL_VERSION.to_string(),
        stage_114_review_id: review.review_id.clone(),
        stage_114_review_sha256: review.review_sha256.clone(),
        stage_113_validation_id: validation.validation_id.clone(),
        stage_113_validation_sha256: validation.validation_sha256.clone(),
        stage_112_result_id: result.result_id.clone(),
        stage_112_result_sha256: result.result_sha256.clone(),
        stage_112_output_sha256: validation.stage_112_output_sha256.clone(),
        stage_111_attempt_id: claim.attempt_id.clone(),
        stage_111_claim_sha256: claim.claim_sha256.clone(),
        stage_104_review_sha256: validation.stage_104_review_sha256.clone(),
        stage_102_output_sha256: validation.stage_102_output_sha256.clone(),
        admitted_available_at_utc: review.admitted_available_at_utc,
        provider_publication_time_verified: false,
        stage_88_initialization_output_sha256: envelope
            .initial_shadow_allocation_binding
            .stage_88_initial_observation_output_sha256
            .clone(),
        stage_88_initialization_manifest_sha256: envelope
            .initial_shadow_allocation_binding
            .initial_shadow_allocation_manifest_sha256
            .clone(),
        subject_symbols,
        benchmark_symbol: "SPY".to_string(),
        earliest_market_session_date,
        latest_market_session_date,
        observed_session_count: envelope.sessions.len(),
        observed_price_count: envelope.prices.len(),
        observed_gap_count: envelope.explicit_gaps.len(),
        observed_dividend_count: envelope.dividends.len(),
        observed_split_count: envelope.splits.len(),
        exact_stage_114_admitted_envelope_is_only_observation_input: true,
        opening_portfolio_prerequisite: independent_opening_portfolio_prerequisite(),
        mapping_rules: independent_mapping_rules(),
        future_ledger_relative_path_template: format!(
            "shadow-ledgers/{}/{{opening_snapshot_sha256}}/ledger.json",
            review.review_id
        ),
        future_event_stream_relative_path_template: format!(
            "shadow-ledgers/{}/{{opening_snapshot_sha256}}/events/{{event_stream_sha256}}.json",
            review.review_id
        ),
        create_once_required: true,
        append_only_required: true,
        overwrite_allowed: false,
        retroactive_backfill_allowed: false,
        in_place_correction_allowed: false,
        financial_postings_currently_eligible: false,
        nav_or_performance_currently_eligible: false,
        future_independent_specification_review_required: true,
        authority_boundary: independent_closed_authority_boundary(),
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])?;
    Ok(specification)
}

fn independently_collect_subject_symbols(
    source: &AdmittedControlledShadowObservationEvidence,
) -> Vec<String> {
    source
        .envelope
        .prices
        .iter()
        .map(|value| value.symbol.clone())
        .chain(
            source
                .envelope
                .explicit_gaps
                .iter()
                .map(|value| value.symbol.clone()),
        )
        .chain(
            source
                .envelope
                .dividends
                .iter()
                .map(|value| value.symbol.clone()),
        )
        .chain(
            source
                .envelope
                .splits
                .iter()
                .map(|value| value.symbol.clone()),
        )
        .filter(|symbol| symbol != "SPY")
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn independent_opening_portfolio_prerequisite() -> OpeningPortfolioPrerequisite {
    OpeningPortfolioPrerequisite {
        separately_admitted_opening_portfolio_snapshot_required: true,
        current_opening_portfolio_snapshot_available: false,
        stage_88_binding_is_initialization_provenance_not_opening_positions: true,
        default_notional_allowed: false,
        default_cash_allowed: false,
        infer_positions_from_subject_symbols_allowed: false,
        infer_share_quantities_from_prices_or_target_weights_allowed: false,
        financial_posting_before_opening_snapshot_admission_allowed: false,
        missing_opening_snapshot_result:
            "fail_closed_waiting_separately_admitted_opening_portfolio_snapshot".to_string(),
    }
}

fn independent_mapping_rules() -> ObservationLedgerEventMappingRules {
    ObservationLedgerEventMappingRules {
        non_financial_event_type_allowlist: vec![
            "observation_evidence_anchor".to_string(),
            "market_session_fact".to_string(),
            "security_raw_close_mark_fact".to_string(),
            "benchmark_total_return_mark_fact".to_string(),
            "explicit_data_gap_fact".to_string(),
            "dividend_notice_pending_validation".to_string(),
            "split_notice_pending_validation".to_string(),
        ],
        financial_event_type_allowlist_before_opening_snapshot: Vec::new(),
        canonical_event_schema: "event_id,event_type,effective_date,available_at_utc,stage_114_review_sha256,stage_112_output_sha256,source_row_sha256|null,symbol|null,exact_decimal_fields,non_financial:true,supersedes_event_id|null,event_sha256".to_string(),
        canonical_double_entry_posting_schema: "future_only_after_opening_snapshot_admission:posting_id,event_id,account_id,debit_decimal,credit_decimal,currency,quantity_decimal|null,unit_price_decimal|null,posting_sha256".to_string(),
        event_idempotency_rule: "sha256(stage_114_review_sha256,stage_112_output_sha256,event_type,source_row_sha256_or_envelope_sha256,symbol_or_none,effective_date)".to_string(),
        evidence_availability_rule: "no_event_may_be_available_before_stage_114_admitted_available_at_utc;provider_publication_time_is_not_verified".to_string(),
        security_valuation_price_basis: "raw_unadjusted_price_only_after_opening_positions_are_separately_admitted".to_string(),
        benchmark_total_return_price_basis: "dividend_adjusted_price_for_non_accounting_SPY_total_return_comparison_only".to_string(),
        split_adjusted_price_usage: "continuity_diagnostic_only_never_a_portfolio_accounting_mark".to_string(),
        dividend_adjusted_price_usage: "benchmark_or_research_continuity_only_never_a_security_accounting_mark".to_string(),
        explicit_gap_rule: "no_fill_no_interpolation_no_cross_basis_substitution;affected_session_NAV_and_return_are_incomplete_and_fail_closed".to_string(),
        dividend_rule: "notice_only_until_opening_position_and_effective_ex_pay_terms_are_independently_admitted;never_combine_dividend_cash_with_dividend_adjusted_security_mark".to_string(),
        split_rule: "notice_only_until_opening_position_and_effective_ratio_terms_are_independently_admitted;future_quantity_adjustment_precedes_same_session_raw_close_mark".to_string(),
        correction_rule: "new_admitted_evidence_creates_append_only_superseding_or_reversal_events;historical_events_are_never_mutated_or_backfilled".to_string(),
        decimal_rule: "exact_canonical_decimal_strings_only_no_binary_float_no_implicit_rounding;presentation_rounding_never_changes_ledger_values".to_string(),
        canonical_event_order: "effective_date_ascending_then_event_precedence:evidence_anchor,market_session,corporate_action_notice,explicit_gap,price_mark_then_symbol_ascending_then_event_id".to_string(),
        double_entry_balance_rule: "future_financial_event_debits_must_exactly_equal_credits_per_currency_or_the_transition_fails_closed".to_string(),
        nav_completeness_rule: "NAV_or_return_requires_admitted_opening_snapshot_and_complete_raw_marks_for_every_nonzero_position;otherwise_no_NAV_or_performance_fact".to_string(),
    }
}

fn independent_closed_authority_boundary()
-> ZeroCapabilityObservationLedgerTransitionAuthorityBoundary {
    ZeroCapabilityObservationLedgerTransitionAuthorityBoundary {
        implementation_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        admitted_evidence_mounted_or_read_for_execution: false,
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

fn registration_downstream_closed(
    value: &ControlledShadowObservationLedgerTransitionSpecificationRegistration,
) -> bool {
    !value.specification_review_completed
        && !value.implementation_registration_eligible
        && !value.opening_portfolio_snapshot_admitted
        && !value.ledger_created
        && !value.ledger_event_written
        && !value.position_written
        && !value.cash_written
        && !value.nav_or_performance_written
        && !value.model_or_metric_store_written
        && !value.training_or_rl_feedback_authorized
        && !value.reward_authorized
        && !value.order_generation_authorized
        && !value.broker_access_authorized
        && !value.trading_authorized
}

fn request_checks(
    value: &ReviewControlledShadowObservationLedgerTransitionSpecificationRequest,
) -> bool {
    value.exact_current_stage_51_through_stage_115_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.registration_and_specification_hashes_independently_reproduced_confirmed
        && value.complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed
        && value.rebuilt_specification_exactly_matches_registered_specification_confirmed
        && value.stage_88_binding_not_opening_positions_confirmed
        && value.separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed
        && value.raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed
        && value.explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed
        && value.dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed
        && value.exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed
        && value.corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed
        && value.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed
        && value.no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed
        && value.no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_zero_capability_implementation_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn review_checks(
    value: &ControlledShadowObservationLedgerTransitionSpecificationReviewRecord,
) -> bool {
    value.exact_current_stage_51_through_stage_115_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.registration_and_specification_hashes_independently_reproduced_confirmed
        && value.complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed
        && value.rebuilt_specification_exactly_matches_registered_specification_confirmed
        && value.stage_88_binding_not_opening_positions_confirmed
        && value.separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed
        && value.raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed
        && value.explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed
        && value.dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed
        && value.exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed
        && value.corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed
        && value.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed
        && value.no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed
        && value.no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_zero_capability_implementation_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_review(
    source: &AdmittedControlledShadowObservationEvidence,
    review: &ControlledShadowObservationLedgerTransitionSpecificationReviewRecord,
) -> Result<(), String> {
    let reviewable =
        IndependentlyReviewableControlledShadowObservationLedgerTransitionSpecification {
            source: source.clone(),
            registration: review.registration.clone(),
        };
    let expected_audit = independently_audit(&reviewable)?;
    let approved = review.verdict
        == ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityLedgerTransitionImplementationRegistration;
    let texts_valid = [
        &review.rationale,
        &review.binding_and_second_implementation_assessment,
        &review.opening_portfolio_prerequisite_assessment,
        &review.price_basis_gap_and_nav_assessment,
        &review.corporate_action_and_double_count_assessment,
        &review.decimal_idempotency_correction_and_order_assessment,
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
        && !review.admitted_evidence_mounted_or_read_for_execution
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
        .ok_or_else(|| "Stage 116 账本转换规格独立复核无效、漂移或越权".to_string())
}

fn validate_request(
    registration_id: &str,
    value: &ReviewControlledShadowObservationLedgerTransitionSpecificationRequest,
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
        return Err("Stage 116 请求 ID 或摘要无效".to_string());
    }
    Ok(())
}

fn expected_excluded_actor_ids(
    registration: &ControlledShadowObservationLedgerTransitionSpecificationRegistration,
    prior_reviews: &[ControlledShadowObservationLedgerTransitionSpecificationReviewRecord],
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
    reviewable: &IndependentlyReviewableControlledShadowObservationLedgerTransitionSpecification,
) -> Result<Vec<ControlledShadowObservationLedgerTransitionSpecificationReviewRecord>, String> {
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
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            return Err("Stage 116 复核目录含非 JSON 文件".to_string());
        }
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.len() == 0
            || metadata.len() > 1024 * 1024
        {
            return Err("Stage 116 复核文件无效或超限".to_string());
        }
        let review: ControlledShadowObservationLedgerTransitionSpecificationReviewRecord =
            serde_json::from_slice(
                &tokio::fs::read(&path)
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_review(&reviewable.source, &review)?;
        if review.registration.registration_id != reviewable.registration.registration_id
            || path.file_stem().and_then(|value| value.to_str()) != Some(review.review_id.as_str())
        {
            return Err("Stage 116 复核文件路径或登记绑定不一致".to_string());
        }
        reviews.push(review);
    }
    reviews.sort_by(|left, right| left.submitted_at.cmp(&right.submitted_at));
    let mut prior: Vec<ControlledShadowObservationLedgerTransitionSpecificationReviewRecord> =
        Vec::new();
    let mut seen_ids = BTreeSet::new();
    for review in &reviews {
        let previous = prior.last();
        if review.previous_review_id.as_deref() != previous.map(|value| value.review_id.as_str())
            || review.previous_review_sha256.as_deref()
                != previous.map(|value| value.review_sha256.as_str())
            || review.excluded_prior_actor_ids
                != expected_excluded_actor_ids(&reviewable.registration, &prior)
            || !seen_ids.insert(review.review_id.clone())
            || previous.is_some_and(|value| {
                value.future_zero_capability_implementation_registration_eligible
            })
        {
            return Err("Stage 116 复核链分叉、角色排除漂移或批准后仍被追加".to_string());
        }
        prior.push(review.clone());
    }
    Ok(reviews)
}

fn review_root(state: &AppState) -> PathBuf {
    decision_root(state).join(
        "historical-outcome-controlled-shadow-observation-ledger-transition-specification-reviews",
    )
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

async fn acquire_lock(state: &AppState, registration_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage-116-{registration_id}.lock"));
    match tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
    {
        Ok(_) => Ok(ReviewLock(path)),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let stale = tokio::fs::metadata(&path)
                .await
                .ok()
                .and_then(|value| value.modified().ok())
                .and_then(|value| SystemTime::now().duration_since(value).ok())
                .is_some_and(|age| age > StdDuration::from_secs(300));
            if stale {
                tokio::fs::remove_file(&path)
                    .await
                    .map_err(|error| error.to_string())?;
                Box::pin(acquire_lock(state, registration_id)).await
            } else {
                Err("Stage 116 同一规格正在复核，请稍后重试".to_string())
            }
        }
        Err(error) => Err(error.to_string()),
    }
}

struct ReviewLock(PathBuf);

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Stage 116 复核路径缺少父目录".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "Stage 116 复核记录已存在，禁止覆盖".to_string()
            } else {
                error.to_string()
            }
        })?;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_TEXT_CHARS {
        return Err(format!(
            "Stage 116 {label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"
        ));
    }
    Ok(trimmed.to_string())
}

fn fingerprint_without<T: Serialize>(value: &T, excluded: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Stage 116 指纹对象不是 JSON object".to_string())?;
    for key in excluded {
        object.remove(*key);
    }
    let canonical = serde_json::to_vec(&json).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(canonical)))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ReviewControlledShadowObservationLedgerTransitionSpecificationRequest {
        ReviewControlledShadowObservationLedgerTransitionSpecificationRequest {
            expected_previous_review_id: None,
            expected_previous_review_sha256: None,
            expected_registration_sha256: "a".repeat(64),
            expected_specification_sha256: "b".repeat(64),
            expected_independent_audit_sha256: "c".repeat(64),
            verdict: ControlledShadowObservationLedgerTransitionSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityLedgerTransitionImplementationRegistration,
            rationale: "独立重建结果一致。".to_string(),
            binding_and_second_implementation_assessment: "完整责任链与第二实现一致。".to_string(),
            opening_portfolio_prerequisite_assessment: "未把 Stage 88 当作持仓。".to_string(),
            price_basis_gap_and_nav_assessment: "raw close 与 gap 失败关闭正确。".to_string(),
            corporate_action_and_double_count_assessment: "公司行动保持 notice，未双计。".to_string(),
            decimal_idempotency_correction_and_order_assessment: "十进制、幂等与追加纠错正确。".to_string(),
            zero_capability_assessment: "没有账本或财务写权限。".to_string(),
            known_limitations: "opening portfolio snapshot 尚未准入。".to_string(),
            future_implementation_constraints: "下一阶段仍只允许零能力实现登记。".to_string(),
            exact_current_stage_51_through_stage_115_binding_confirmed: true,
            reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
            registration_and_specification_hashes_independently_reproduced_confirmed: true,
            complete_specification_rebuilt_from_current_stage_114_evidence_without_stage_115_builder_confirmed: true,
            rebuilt_specification_exactly_matches_registered_specification_confirmed: true,
            stage_88_binding_not_opening_positions_confirmed: true,
            separate_opening_portfolio_snapshot_required_and_no_defaults_or_inference_confirmed: true,
            raw_close_only_for_security_accounting_and_adjusted_prices_non_accounting_confirmed: true,
            explicit_gap_blocks_nav_without_fill_interpolation_or_substitution_confirmed: true,
            dividends_and_splits_notice_only_until_position_and_terms_are_admitted_confirmed: true,
            exact_decimal_append_only_idempotent_event_and_double_entry_rules_confirmed: true,
            corrections_require_new_admitted_evidence_and_superseding_or_reversal_events_confirmed: true,
            conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: true,
            no_implementation_artifact_entrypoint_runtime_input_mount_or_financial_write_confirmed: true,
            no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_zero_capability_implementation_registration_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn independent_opening_contract_never_invents_a_portfolio() {
        let value = independent_opening_portfolio_prerequisite();
        assert!(value.separately_admitted_opening_portfolio_snapshot_required);
        assert!(value.stage_88_binding_is_initialization_provenance_not_opening_positions);
        assert!(!value.current_opening_portfolio_snapshot_available);
        assert!(!value.default_notional_allowed);
        assert!(!value.default_cash_allowed);
        assert!(!value.infer_positions_from_subject_symbols_allowed);
        assert!(!value.infer_share_quantities_from_prices_or_target_weights_allowed);
        assert!(!value.financial_posting_before_opening_snapshot_admission_allowed);
    }

    #[test]
    fn independent_accounting_contract_prevents_adjusted_price_double_count_and_gap_fill() {
        let rules = independent_mapping_rules();
        assert!(
            rules
                .financial_event_type_allowlist_before_opening_snapshot
                .is_empty()
        );
        assert!(
            rules
                .security_valuation_price_basis
                .starts_with("raw_unadjusted")
        );
        assert!(
            rules
                .benchmark_total_return_price_basis
                .contains("non_accounting_SPY")
        );
        assert!(
            rules
                .dividend_adjusted_price_usage
                .contains("never_a_security_accounting_mark")
        );
        assert!(rules.explicit_gap_rule.contains("no_fill"));
        assert!(rules.explicit_gap_rule.contains("fail_closed"));
        assert!(rules.dividend_rule.contains("never_combine"));
        assert!(
            rules
                .correction_rule
                .contains("superseding_or_reversal_events")
        );
    }

    #[test]
    fn review_approval_requires_every_independent_confirmation() {
        let mut value = request();
        assert!(request_checks(&value));
        value.stage_88_binding_not_opening_positions_confirmed = false;
        assert!(!request_checks(&value));
        assert!(validate_request("registration-116", &value).is_ok());
    }

    #[test]
    fn independent_review_has_zero_runtime_financial_or_trading_authority() {
        let value = independent_closed_authority_boundary();
        assert!(!value.implementation_present);
        assert!(!value.runtime_present);
        assert!(!value.opening_portfolio_snapshot_present);
        assert!(!value.ledger_event_write_allowed);
        assert!(!value.position_write_allowed);
        assert!(!value.cash_write_allowed);
        assert!(!value.nav_or_performance_write_allowed);
        assert!(!value.training_or_rl_feedback_allowed);
        assert!(!value.order_generation_allowed);
        assert!(!value.broker_access_allowed);
        assert!(!value.trading_allowed);
    }
}
