//! Stage 126 chain-external review of the Stage 125 opening-portfolio governance specification.
//!
//! A second implementation reconstructs and rehashes the complete source and snapshot contract.
//! Approval opens only a future zero-capability source-artifact receipt implementation contract.
//! This module cannot receive or read a broker file, materialize an opening snapshot, create a
//! ledger, write cash/positions, calculate NAV/performance, train, generate an order or trade.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::warn;

use super::controlled_shadow_opening_portfolio_snapshot_governance_specifications::{
    IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification,
    OpeningPortfolioCanonicalSnapshotSchema, OpeningPortfolioSnapshotGovernanceSpecification,
    OpeningPortfolioSnapshotGovernanceSpecificationRegistration,
    OpeningPortfolioSourceArtifactContract, ZeroCapabilityOpeningPortfolioAuthorityBoundary,
    independently_reviewable_opening_portfolio_snapshot_governance_specifications,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str = "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-independent-audit-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-chain-external-review-v1-zero-financial-state";
const STAGE_125_RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-registration-v1";
const STAGE_125_SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-v1";
const STAGE_125_POLICY_VERSION: &str = "hone-controlled-shadow-external-source-opening-portfolio-snapshot-governance-specification-create-once-v1-zero-financial-state";
const STAGE_125_PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-opening-portfolio-source-and-completeness-contract-v1";
const STAGE_125_STATUS: &str = "registered_waiting_stage_126_independent_specification_review";
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_FILE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict {
    ApprovedForFutureZeroCapabilitySourceArtifactReceiptImplementationRegistration,
    ChangesRequiredRebuildOpeningPortfolioGovernanceSpecification,
    RejectedOpeningPortfolioGovernanceSpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_registration_sha256: String,
    expected_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict,
    rationale: String,
    binding_and_second_implementation_assessment: String,
    source_artifact_and_identity_assessment: String,
    account_scope_and_snapshot_completeness_assessment: String,
    valuation_and_nav_prerequisite_assessment: String,
    zero_capability_assessment: String,
    known_limitations: String,
    future_implementation_constraints: String,
    exact_current_stage_51_through_stage_125_binding_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    registration_and_specification_hashes_independently_reproduced_confirmed: bool,
    complete_specification_rebuilt_without_stage_125_builder_confirmed: bool,
    rebuilt_specification_exactly_matches_registered_specification_confirmed: bool,
    original_external_artifact_provenance_and_pseudonymization_contract_confirmed: bool,
    complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: bool,
    exact_decimal_signed_quantity_no_default_inference_or_partial_admission_confirmed: bool,
    instrument_identity_cost_basis_and_corporate_action_contract_confirmed: bool,
    statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed: bool,
    source_receipt_snapshot_materialization_output_validation_and_admission_remain_separate_confirmed:
        bool,
    no_artifact_upload_read_parser_runtime_snapshot_or_financial_state_confirmed: bool,
    no_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    approval_only_opens_future_zero_capability_source_receipt_implementation_registration_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub specification_sha256: String,
    pub registration_hash_independently_reproduced: bool,
    pub specification_hash_independently_reproduced: bool,
    pub exact_current_stage_51_through_stage_125_binding_valid: bool,
    pub complete_specification_rebuilt_without_stage_125_builder: bool,
    pub rebuilt_specification_exactly_matches_registration: bool,
    pub external_source_artifact_and_identity_contract_valid: bool,
    pub complete_account_scope_and_snapshot_schema_contract_valid: bool,
    pub exact_decimal_no_invention_and_append_only_contract_valid: bool,
    pub statement_value_and_independent_valuation_prerequisite_contract_valid: bool,
    pub future_gates_remain_separate_and_current_financial_state_closed: bool,
    pub all_artifact_runtime_ledger_feedback_order_broker_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub registration: OpeningPortfolioSnapshotGovernanceSpecificationRegistration,
    pub independent_audit: OpeningPortfolioSnapshotGovernanceSpecificationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict,
    pub rationale: String,
    pub binding_and_second_implementation_assessment: String,
    pub source_artifact_and_identity_assessment: String,
    pub account_scope_and_snapshot_completeness_assessment: String,
    pub valuation_and_nav_prerequisite_assessment: String,
    pub zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_implementation_constraints: String,
    pub confirmations_complete: bool,
    pub specification_independently_approved: bool,
    pub future_zero_capability_source_artifact_receipt_implementation_registration_eligible: bool,
    pub source_artifact_present: bool,
    pub source_artifact_uploaded_or_read: bool,
    pub parser_or_runtime_present: bool,
    pub opening_portfolio_snapshot_materialized: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationReviewItem {
    pub registration: OpeningPortfolioSnapshotGovernanceSpecificationRegistration,
    pub current_independent_audit: OpeningPortfolioSnapshotGovernanceSpecificationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord>,
    pub review_eligible: bool,
    pub future_zero_capability_source_artifact_receipt_implementation_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_endpoint_available: bool,
    pub items: Vec<OpeningPortfolioSnapshotGovernanceSpecificationReviewItem>,
    pub specification_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count:
        usize,
    pub review_status: String,
    pub source_artifact_present: bool,
    pub opening_portfolio_snapshot_present: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationReviewReadinessSummary {
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count:
        usize,
    pub review_status: String,
}

pub(crate) async fn handle_get_opening_portfolio_snapshot_governance_specification_reviews(
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
            warn!(%error, "Stage 126 opening portfolio governance specification review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "期初组合快照治理规格独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_opening_portfolio_snapshot_governance_specification(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(registration_id): AxumPath<String>,
    Json(request): Json<ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest>,
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

pub(crate) async fn opening_portfolio_snapshot_governance_specification_review_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecificationReviewReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(OpeningPortfolioSnapshotGovernanceSpecificationReviewReadinessSummary {
        review_eligible_count: value.review_eligible_count,
        reviewed_count: value.reviewed_count,
        independently_approved_count: value.independently_approved_count,
        changes_required_or_rejected_count: value.changes_required_or_rejected_count,
        future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count: value.future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count,
        review_status: value.review_status,
    })
}

pub(crate) async fn independently_approved_opening_portfolio_snapshot_governance_specifications_for_source_receipt_implementation_registration(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord>, String> {
    let registry = build_registry(state).await?;
    let values = registry
        .items
        .into_iter()
        .filter(|item| {
            item.future_zero_capability_source_artifact_receipt_implementation_registration_eligible
        })
        .filter_map(|item| item.latest_review)
        .collect::<Vec<_>>();
    for review in &values {
        if !review.specification_independently_approved
            || !review.confirmations_complete
            || !review
                .future_zero_capability_source_artifact_receipt_implementation_registration_eligible
        {
            return Err("Stage 126 批准状态不足以登记来源工件接收实现".to_string());
        }
    }
    Ok(values)
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry, String> {
    let reviewables =
        independently_reviewable_opening_portfolio_snapshot_governance_specifications(state)
            .await?;
    let mut items = Vec::new();
    for reviewable in reviewables {
        let audit = independently_audit(&reviewable)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 126 当前独立审计失败，晋级关闭".to_string());
        }
        let reviews = read_reviews(state, &reviewable).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|review| {
            review.registration != reviewable.registration || review.independent_audit != audit
        }) {
            return Err("Stage 126 已保存复核与当前 Stage 125 绑定漂移".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|review| {
            review.future_zero_capability_source_artifact_receipt_implementation_registration_eligible
        });
        items.push(OpeningPortfolioSnapshotGovernanceSpecificationReviewItem {
            registration: reviewable.registration,
            current_independent_audit: audit,
            complete_review_actor_ids: reviews
                .iter()
                .map(|review| review.reviewer_id.clone())
                .collect(),
            latest_review,
            review_eligible: !approved,
            future_zero_capability_source_artifact_receipt_implementation_registration_eligible:
                approved,
        });
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
        .filter(|item| {
            item.future_zero_capability_source_artifact_receipt_implementation_registration_eligible
        })
        .count();
    let changes_required_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review
                .as_ref()
                .is_some_and(|review| !review.specification_independently_approved)
        })
        .count();
    let review_status = if items.is_empty() {
        "waiting_stage_125_opening_portfolio_governance_specification"
    } else if independently_approved_count > 0 {
        "opening_portfolio_governance_specification_independently_approved_only_for_future_zero_capability_source_receipt_implementation_registration"
    } else {
        "opening_portfolio_governance_specification_ready_for_chain_external_review"
    };
    Ok(OpeningPortfolioSnapshotGovernanceSpecificationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_endpoint_available: true,
        items,
        specification_count,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_required_or_rejected_count,
        future_stage_127_zero_capability_source_artifact_receipt_implementation_registration_eligible_count: independently_approved_count,
        review_status: review_status.to_string(),
        source_artifact_present: false,
        opening_portfolio_snapshot_present: false,
        ledger_created: false,
        position_or_cash_written: false,
        nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 126 由责任链外人员使用第二实现重建 Stage 125 完整外部来源与期初组合快照合同。批准只开放 Stage 127 零能力来源工件接收实现登记；当前不接收、上传或读取来源文件，不生成期初组合、账本、持仓、现金、净值/绩效、训练/RL、订单、券商或交易状态。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    registration_id: &str,
    request: ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord, String> {
    validate_request(registration_id, &request)?;
    let reviewable =
        independently_reviewable_opening_portfolio_snapshot_governance_specifications(state)
            .await?
            .into_iter()
            .find(|value| value.registration.registration_id == registration_id)
            .ok_or_else(|| "当前没有可独立复核的 Stage 125 期初组合治理规格".to_string())?;
    if request.expected_registration_sha256 != reviewable.registration.registration_sha256
        || request.expected_specification_sha256
            != reviewable.registration.specification.specification_sha256
    {
        return Err("Stage 125 登记或规格摘要已变化".to_string());
    }
    let audit = independently_audit(&reviewable)?;
    if request.expected_independent_audit_sha256 != audit.audit_sha256
        || !audit.mismatch_reasons.is_empty()
    {
        return Err("Stage 126 独立审计摘要不匹配或未通过".to_string());
    }
    let prior_reviews = read_reviews(state, &reviewable).await?;
    let previous = prior_reviews.last();
    if request.expected_previous_review_id.as_deref()
        != previous.map(|value| value.review_id.as_str())
        || request.expected_previous_review_sha256.as_deref()
            != previous.map(|value| value.review_sha256.as_str())
    {
        return Err("Stage 126 前序复核绑定已变化".to_string());
    }
    if previous.is_some_and(|value| value.specification_independently_approved) {
        return Err("Stage 126 已批准终态不得追加复核".to_string());
    }
    let excluded_prior_actor_ids =
        expected_excluded_actor_ids(&reviewable.registration, &prior_reviews);
    if reviewer_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|value| value == reviewer_id)
    {
        return Err(
            "Stage 126 reviewer 必须独立于 Stage 125 registrar 与完整既有责任链".to_string(),
        );
    }
    let approved = request.verdict == OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict::ApprovedForFutureZeroCapabilitySourceArtifactReceiptImplementationRegistration;
    let confirmations_complete = request_checks(&request);
    if approved && !confirmations_complete {
        return Err("Stage 126 批准需要完成全部独立来源、快照与零能力复核确认".to_string());
    }
    let mut review = OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: previous.map(|value| value.review_id.clone()),
        previous_review_sha256: previous.map(|value| value.review_sha256.clone()),
        registration: reviewable.registration.clone(),
        independent_audit: audit,
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        binding_and_second_implementation_assessment: bounded_required(
            &request.binding_and_second_implementation_assessment,
            "绑定与第二实现评估",
        )?,
        source_artifact_and_identity_assessment: bounded_required(
            &request.source_artifact_and_identity_assessment,
            "来源工件与身份评估",
        )?,
        account_scope_and_snapshot_completeness_assessment: bounded_required(
            &request.account_scope_and_snapshot_completeness_assessment,
            "账户范围与快照完整性评估",
        )?,
        valuation_and_nav_prerequisite_assessment: bounded_required(
            &request.valuation_and_nav_prerequisite_assessment,
            "估值与净值前置条件评估",
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
        confirmations_complete,
        specification_independently_approved: approved,
        future_zero_capability_source_artifact_receipt_implementation_registration_eligible:
            approved,
        source_artifact_present: false,
        source_artifact_uploaded_or_read: false,
        parser_or_runtime_present: false,
        opening_portfolio_snapshot_materialized: false,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        position_or_cash_written: false,
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
    validate_review(&reviewable, &review)?;
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
    reviewable: &IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecificationIndependentAudit, String> {
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
    let rebuilt = independently_rebuild_specification(reviewable)?;
    let rebuilt_specification_exactly_matches_registration = rebuilt == *specification;
    let excluded = independent_stage_125_excluded_actors(reviewable);
    let exact_current_stage_51_through_stage_125_binding_valid = registration.schema_version
        == STAGE_125_RECORD_SCHEMA_VERSION
        && registration.policy_version == STAGE_125_POLICY_VERSION
        && registration.status == STAGE_125_STATUS
        && registration.confirmations_complete
        && registration.specification_registered
        && registration.future_stage_126_independent_specification_review_eligible
        && !registration.specification_review_completed
        && !registration.source_artifact_receipt_eligible
        && registration.registrar_independent_from_stage_124_reviewer_and_complete_prior_chain
        && registration.excluded_prior_actor_ids == excluded
        && !excluded.contains(&registration.registered_by)
        && registration.stage_124_review_id == reviewable.source.admission_review.review_id
        && registration.stage_124_review_sha256 == reviewable.source.admission_review.review_sha256
        && registration_hash_independently_reproduced
        && specification_hash_independently_reproduced;
    let source = &specification.source_contract;
    let external_source_artifact_and_identity_contract_valid = source.original_bytes_required
        && source.content_sha256_and_byte_length_required
        && source.provider_statement_or_export_identifier_required
        && source.provider_generated_at_or_statement_as_of_required
        && source.hone_received_at_required
        && source.source_account_identifiers_must_be_pseudonymized
        && !source.raw_account_numbers_or_credentials_allowed
        && !source.manual_balance_or_position_entry_allowed
        && !source.mutable_or_overwritable_artifact_allowed
        && source.source_timezone.parse::<chrono_tz::Tz>().is_ok()
        && source.accepted_artifact_formats == independent_artifact_formats();
    let schema = &specification.canonical_snapshot_schema;
    let complete_account_scope_and_snapshot_schema_contract_valid = *schema
        == independent_snapshot_schema()
        && schema
            .account_scope_completeness_rule
            .contains("every_account")
        && schema.cash_completeness_rule.contains("no_default_zero")
        && schema
            .liabilities_and_unsettled_activity_rule
            .contains("pending_trades")
        && schema.listed_option_extension_schema.contains("occ_symbol")
        && schema.unsupported_asset_class_result.contains("no_partial");
    let exact_decimal_no_invention_and_append_only_contract_valid = specification
        .create_once_required
        && specification.append_only_corrections_required
        && !specification.overwrite_allowed
        && !specification.default_notional_allowed
        && !specification.infer_cash_positions_quantities_cost_basis_or_weights_allowed
        && schema.exact_decimal_rule.contains("no_binary_float")
        && schema.signed_quantity_rule.contains("written_negative")
        && schema.cost_basis_rule.contains("never_inferred")
        && schema
            .correction_rule
            .contains("append_only_superseding_snapshot");
    let statement_value_and_independent_valuation_prerequisite_contract_valid = schema
        .statement_market_value_rule
        .contains("never_an_accounting_mark")
        && schema
            .opening_nav_rule
            .contains("independent_raw_marks_FX_and_derivative_valuation")
        && !specification.nav_or_performance_currently_eligible;
    let future_gates_remain_separate_and_current_financial_state_closed = specification
        .future_source_artifact_receipt_validation_required
        && specification.future_canonical_snapshot_materialization_required
        && specification.future_independent_snapshot_output_validation_required
        && specification.future_opening_snapshot_admission_review_required
        && specification.future_independent_specification_review_required
        && !specification.financial_postings_currently_eligible;
    let all_artifact_runtime_ledger_feedback_order_broker_and_trading_authority_closed =
        specification.authority_boundary == independent_closed_authority_boundary()
            && registration_downstream_closed(registration);
    let checks = [
        (
            registration_hash_independently_reproduced,
            "Stage 125 registration hash mismatch",
        ),
        (
            specification_hash_independently_reproduced,
            "Stage 125 specification hash mismatch",
        ),
        (
            exact_current_stage_51_through_stage_125_binding_valid,
            "Stage 51-125 current binding invalid",
        ),
        (
            rebuilt_specification_exactly_matches_registration,
            "independent specification rebuild mismatch",
        ),
        (
            external_source_artifact_and_identity_contract_valid,
            "source artifact or identity contract invalid",
        ),
        (
            complete_account_scope_and_snapshot_schema_contract_valid,
            "account scope or snapshot completeness contract invalid",
        ),
        (
            exact_decimal_no_invention_and_append_only_contract_valid,
            "decimal, no-invention or append-only contract invalid",
        ),
        (
            statement_value_and_independent_valuation_prerequisite_contract_valid,
            "statement value or independent valuation prerequisite invalid",
        ),
        (
            future_gates_remain_separate_and_current_financial_state_closed,
            "future gates merged or current financial state open",
        ),
        (
            all_artifact_runtime_ledger_feedback_order_broker_and_trading_authority_closed,
            "zero capability or downstream authority boundary open",
        ),
    ];
    let mismatch_reasons = checks
        .iter()
        .filter_map(|(passed, reason)| (!passed).then_some((*reason).to_string()))
        .collect();
    let mut audit = OpeningPortfolioSnapshotGovernanceSpecificationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        registration_id: registration.registration_id.clone(),
        registration_sha256: registration.registration_sha256.clone(),
        specification_sha256: specification.specification_sha256.clone(),
        registration_hash_independently_reproduced,
        specification_hash_independently_reproduced,
        exact_current_stage_51_through_stage_125_binding_valid,
        complete_specification_rebuilt_without_stage_125_builder: true,
        rebuilt_specification_exactly_matches_registration,
        external_source_artifact_and_identity_contract_valid,
        complete_account_scope_and_snapshot_schema_contract_valid,
        exact_decimal_no_invention_and_append_only_contract_valid,
        statement_value_and_independent_valuation_prerequisite_contract_valid,
        future_gates_remain_separate_and_current_financial_state_closed,
        all_artifact_runtime_ledger_feedback_order_broker_and_trading_authority_closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn independently_rebuild_specification(
    reviewable: &IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecification, String> {
    let registered = &reviewable.registration.specification;
    let source = &reviewable.source;
    let source_contract = &registered.source_contract;
    if source_contract.source_provider_name.trim().is_empty()
        || source_contract.portfolio_scope_alias.len() < 3
        || source_contract.reporting_currency.len() != 3
        || source_contract.expected_account_count == 0
        || source_contract
            .source_timezone
            .parse::<chrono_tz::Tz>()
            .is_err()
    {
        return Err("Stage 126 独立重建发现来源变量无效".to_string());
    }
    let mut specification = OpeningPortfolioSnapshotGovernanceSpecification {
        schema_version: STAGE_125_SPEC_SCHEMA_VERSION.to_string(),
        specification_sha256: String::new(),
        protocol_version: STAGE_125_PROTOCOL_VERSION.to_string(),
        stage_124_review_id: source.admission_review.review_id.clone(),
        stage_124_review_sha256: source.admission_review.review_sha256.clone(),
        stage_123_validation_sha256: source.candidate.validation.validation_sha256.clone(),
        stage_122_candidate_sha256: source.candidate.candidate.candidate_sha256.clone(),
        stage_114_review_sha256: source.candidate.validation.stage_114_review_sha256.clone(),
        stage_112_output_sha256: source.candidate.validation.stage_112_output_sha256.clone(),
        source_contract: OpeningPortfolioSourceArtifactContract {
            source_kind: source_contract.source_kind.clone(),
            source_provider_name: source_contract.source_provider_name.clone(),
            portfolio_scope_alias: source_contract.portfolio_scope_alias.clone(),
            reporting_currency: source_contract.reporting_currency.clone(),
            source_timezone: source_contract.source_timezone.clone(),
            snapshot_as_of_utc: source_contract.snapshot_as_of_utc,
            expected_account_count: source_contract.expected_account_count,
            accepted_artifact_formats: independent_artifact_formats(),
            original_bytes_required: true,
            content_sha256_and_byte_length_required: true,
            provider_statement_or_export_identifier_required: true,
            provider_generated_at_or_statement_as_of_required: true,
            hone_received_at_required: true,
            source_account_identifiers_must_be_pseudonymized: true,
            raw_account_numbers_or_credentials_allowed: false,
            manual_balance_or_position_entry_allowed: false,
            mutable_or_overwritable_artifact_allowed: false,
        },
        canonical_snapshot_schema: independent_snapshot_schema(),
        future_source_artifact_receipt_validation_required: true,
        future_canonical_snapshot_materialization_required: true,
        future_independent_snapshot_output_validation_required: true,
        future_opening_snapshot_admission_review_required: true,
        create_once_required: true,
        append_only_corrections_required: true,
        overwrite_allowed: false,
        default_notional_allowed: false,
        infer_cash_positions_quantities_cost_basis_or_weights_allowed: false,
        financial_postings_currently_eligible: false,
        nav_or_performance_currently_eligible: false,
        future_independent_specification_review_required: true,
        authority_boundary: independent_closed_authority_boundary(),
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])?;
    Ok(specification)
}

fn independent_artifact_formats() -> Vec<String> {
    vec![
        "original_provider_pdf_statement".to_string(),
        "original_provider_csv_export".to_string(),
        "original_provider_json_export".to_string(),
    ]
}

fn independent_snapshot_schema() -> OpeningPortfolioCanonicalSnapshotSchema {
    OpeningPortfolioCanonicalSnapshotSchema {
        account_schema: "portfolio_scope_alias,source_account_alias,account_type,base_currency,scope_complete,account_sha256".to_string(),
        cash_schema: "source_account_alias,currency,settled_cash_decimal,unsettled_cash_decimal,accrued_interest_decimal|null,cash_sha256".to_string(),
        position_schema: "source_account_alias,asset_class,instrument_id_type,instrument_id,symbol|null,exchange_mic|null,currency,signed_quantity_decimal,cost_basis_decimal|null,position_sha256".to_string(),
        listed_option_extension_schema: "occ_symbol,underlying_instrument_id,expiration_date,strike_decimal,call_or_put,contract_multiplier_decimal,signed_contract_quantity_decimal".to_string(),
        liability_schema: "source_account_alias,currency,liability_type,amount_decimal,accrued_amount_decimal|null,liability_sha256".to_string(),
        unsettled_activity_schema: "source_account_alias,activity_type,trade_date,settlement_date,instrument_id|null,signed_quantity_decimal|null,currency,cash_effect_decimal,activity_sha256".to_string(),
        instrument_identity_precedence: vec!["provider_permanent_instrument_id".to_string(), "figi".to_string(), "isin".to_string(), "cusip".to_string(), "occ_symbol_for_listed_options".to_string(), "ticker_plus_exchange_mic_only_if_independently_resolved".to_string()],
        supported_asset_classes: vec!["cash".to_string(), "common_equity".to_string(), "etf".to_string(), "adr".to_string(), "listed_option".to_string()],
        unsupported_asset_class_result: "fail_closed_no_partial_opening_snapshot_admission".to_string(),
        exact_decimal_rule: "canonical_base10_strings_only_no_binary_float_no_implicit_rounding".to_string(),
        signed_quantity_rule: "long_positive_short_or_written_negative_zero_positions_omitted_only_when_source_proves_zero".to_string(),
        duplicate_instrument_rule: "aggregate_only_within_same_account_currency_and_exact_instrument_identity_while_retaining_source_rows".to_string(),
        cost_basis_rule: "optional_source_fact_never_inferred_and_not_required_for_forward_inception_performance".to_string(),
        statement_market_value_rule: "informational_source_fact_only_never_an_accounting_mark_or_opening_NAV_input".to_string(),
        account_scope_completeness_rule: "every_account_in_declared_scope_must_be_present_and_reconciled_to_expected_account_count".to_string(),
        cash_completeness_rule: "settled_and_unsettled_cash_required_for_every_source_currency_no_default_zero".to_string(),
        liabilities_and_unsettled_activity_rule: "margin_loans_accrued_fees_interest_and_pending_trades_required_no_netting_or_omission".to_string(),
        corporate_action_reconciliation_rule: "identifiers_and_quantities_must_reflect_source_effective_actions_as_of_snapshot_time_and_conflicts_fail_closed".to_string(),
        missing_or_ambiguous_field_rule: "explicit_gap_with_reason_and_entire_snapshot_ineligible_no_fill_inference_or_partial_admission".to_string(),
        opening_nav_rule: "no_NAV_until_snapshot_admitted_and_complete_independent_raw_marks_FX_and_derivative_valuation_exist_for_every_nonzero_exposure_and_liability".to_string(),
        performance_inception_rule: "performance_starts_at_admitted_snapshot_as_of_and_never_backfills_pre_snapshot_returns".to_string(),
        correction_rule: "new_external_artifact_and_new_independent_admission_create_append_only_superseding_snapshot_never_mutate_history".to_string(),
    }
}

fn independent_closed_authority_boundary() -> ZeroCapabilityOpeningPortfolioAuthorityBoundary {
    ZeroCapabilityOpeningPortfolioAuthorityBoundary {
        source_artifact_present: false,
        source_artifact_uploaded_or_read: false,
        parser_or_implementation_present: false,
        executable_artifact_or_entrypoint_present: false,
        runtime_present: false,
        opening_portfolio_snapshot_materialized: false,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false,
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
    value: &OpeningPortfolioSnapshotGovernanceSpecificationRegistration,
) -> bool {
    !value.specification_review_completed
        && !value.source_artifact_receipt_eligible
        && !value.opening_portfolio_snapshot_admitted
        && !value.ledger_created
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

fn independent_stage_125_excluded_actors(
    reviewable: &IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification,
) -> Vec<String> {
    let source = &reviewable.source;
    let mut values = source.admission_review.excluded_prior_actor_ids.clone();
    values.push(source.admission_review.submitted_by.clone());
    values.push(source.candidate.validation.validated_by.clone());
    values.push(source.candidate.result.executed_by.clone());
    values.push(source.candidate.claim.claimed_by.clone());
    values.extend(source.candidate.claim.excluded_prior_actor_ids.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn request_checks(value: &ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest) -> bool {
    value.exact_current_stage_51_through_stage_125_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.registration_and_specification_hashes_independently_reproduced_confirmed
        && value.complete_specification_rebuilt_without_stage_125_builder_confirmed
        && value.rebuilt_specification_exactly_matches_registered_specification_confirmed
        && value.original_external_artifact_provenance_and_pseudonymization_contract_confirmed
        && value.complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed
        && value.exact_decimal_signed_quantity_no_default_inference_or_partial_admission_confirmed
        && value.instrument_identity_cost_basis_and_corporate_action_contract_confirmed
        && value.statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed
        && value.source_receipt_snapshot_materialization_output_validation_and_admission_remain_separate_confirmed
        && value.no_artifact_upload_read_parser_runtime_snapshot_or_financial_state_confirmed
        && value.no_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_zero_capability_source_receipt_implementation_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_review(
    reviewable: &IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification,
    review: &OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord,
) -> Result<(), String> {
    let expected_audit = independently_audit(reviewable)?;
    let approved = review.verdict == OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict::ApprovedForFutureZeroCapabilitySourceArtifactReceiptImplementationRegistration;
    let texts_valid = [
        &review.rationale,
        &review.binding_and_second_implementation_assessment,
        &review.source_artifact_and_identity_assessment,
        &review.account_scope_and_snapshot_completeness_assessment,
        &review.valuation_and_nav_prerequisite_assessment,
        &review.zero_capability_assessment,
        &review.known_limitations,
        &review.future_implementation_constraints,
    ]
    .iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let no_downstream = !review.source_artifact_present
        && !review.source_artifact_uploaded_or_read
        && !review.parser_or_runtime_present
        && !review.opening_portfolio_snapshot_materialized
        && !review.opening_portfolio_snapshot_admitted
        && !review.financial_event_allowlist_nonempty
        && !review.ledger_created
        && !review.position_or_cash_written
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
        && review.registration == reviewable.registration
        && review.independent_audit == expected_audit
        && expected_audit.mismatch_reasons.is_empty()
        && sorted_unique(&review.excluded_prior_actor_ids)
        && !review
            .excluded_prior_actor_ids
            .contains(&review.reviewer_id)
        && review
            .excluded_prior_actor_ids
            .contains(&review.registration.registered_by)
        && texts_valid
        && (!approved || review.confirmations_complete)
        && review.specification_independently_approved == approved
        && review
            .future_zero_capability_source_artifact_receipt_implementation_registration_eligible
            == approved
        && no_downstream;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 126 期初组合治理规格独立复核无效、漂移或越权".to_string())
}

fn validate_request(
    registration_id: &str,
    value: &ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) -> Result<(), String> {
    let valid = valid_id(registration_id)
        && valid_sha256(&value.expected_registration_sha256)
        && valid_sha256(&value.expected_specification_sha256)
        && valid_sha256(&value.expected_independent_audit_sha256)
        && value
            .expected_previous_review_id
            .as_ref()
            .is_none_or(|item| valid_id(item))
        && value
            .expected_previous_review_sha256
            .as_ref()
            .is_none_or(|item| valid_sha256(item))
        && value.expected_previous_review_id.is_some()
            == value.expected_previous_review_sha256.is_some();
    valid
        .then_some(())
        .ok_or_else(|| "Stage 126 请求 ID 或摘要无效".to_string())
}

fn expected_excluded_actor_ids(
    registration: &OpeningPortfolioSnapshotGovernanceSpecificationRegistration,
    prior_reviews: &[OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord],
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
    reviewable: &IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification,
) -> Result<Vec<OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord>, String> {
    let directory = review_root(state).join(&reviewable.registration.registration_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut values = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            return Err("Stage 126 复核目录含非 JSON 文件".to_string());
        }
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() == 0
            || metadata.len() > MAX_FILE_BYTES
        {
            return Err("Stage 126 复核文件无效或超限".to_string());
        }
        let review: OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord =
            serde_json::from_slice(
                &tokio::fs::read(&path)
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_review(reviewable, &review)?;
        if path.file_stem().and_then(|value| value.to_str()) != Some(review.review_id.as_str()) {
            return Err("Stage 126 复核文件路径不一致".to_string());
        }
        values.push(review);
    }
    values.sort_by(|left, right| left.submitted_at.cmp(&right.submitted_at));
    let mut prior = Vec::new();
    let mut seen = BTreeSet::new();
    for review in &values {
        let previous = prior.last();
        if review.previous_review_id.as_deref()
            != previous.map(
                |value: &OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord| {
                    value.review_id.as_str()
                },
            )
            || review.previous_review_sha256.as_deref()
                != previous.map(|value| value.review_sha256.as_str())
            || review.excluded_prior_actor_ids
                != expected_excluded_actor_ids(&reviewable.registration, &prior)
            || !seen.insert(review.review_id.clone())
            || previous.is_some_and(|value| value.specification_independently_approved)
        {
            return Err("Stage 126 复核链分叉、角色排除漂移或批准后仍被追加".to_string());
        }
        prior.push(review.clone());
    }
    Ok(values)
}

fn review_root(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-snapshot-governance-specification-reviews")
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("Stage 126 复核文件为空或超限".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "Stage 126 复核路径缺少父目录".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "Stage 126 复核记录已存在，禁止覆盖".to_string()
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
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!(
            "Stage 126 {label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"
        ))
    } else {
        Ok(value.to_string())
    }
}

fn fingerprint_without<T: Serialize>(value: &T, excluded: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Stage 126 指纹对象不是 JSON object".to_string())?;
    for key in excluded {
        object.remove(*key);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&json).map_err(|error| error.to_string())?)
    ))
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
mod tests {
    use super::*;

    fn request() -> ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest {
        ReviewOpeningPortfolioSnapshotGovernanceSpecificationRequest {
            expected_previous_review_id: None, expected_previous_review_sha256: None,
            expected_registration_sha256: "a".repeat(64), expected_specification_sha256: "b".repeat(64), expected_independent_audit_sha256: "c".repeat(64),
            verdict: OpeningPortfolioSnapshotGovernanceSpecificationReviewVerdict::ApprovedForFutureZeroCapabilitySourceArtifactReceiptImplementationRegistration,
            rationale: "独立重建一致。".to_string(), binding_and_second_implementation_assessment: "绑定和第二实现有效。".to_string(),
            source_artifact_and_identity_assessment: "原始工件和身份链完整。".to_string(), account_scope_and_snapshot_completeness_assessment: "完整账户和字段合同有效。".to_string(),
            valuation_and_nav_prerequisite_assessment: "独立估值前置门有效。".to_string(), zero_capability_assessment: "仍无财务状态或交易能力。".to_string(),
            known_limitations: "尚未接收来源工件。".to_string(), future_implementation_constraints: "下一步只登记零能力接收实现。".to_string(),
            exact_current_stage_51_through_stage_125_binding_confirmed: true, reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
            registration_and_specification_hashes_independently_reproduced_confirmed: true, complete_specification_rebuilt_without_stage_125_builder_confirmed: true,
            rebuilt_specification_exactly_matches_registered_specification_confirmed: true, original_external_artifact_provenance_and_pseudonymization_contract_confirmed: true,
            complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: true, exact_decimal_signed_quantity_no_default_inference_or_partial_admission_confirmed: true,
            instrument_identity_cost_basis_and_corporate_action_contract_confirmed: true, statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed: true,
            source_receipt_snapshot_materialization_output_validation_and_admission_remain_separate_confirmed: true, no_artifact_upload_read_parser_runtime_snapshot_or_financial_state_confirmed: true,
            no_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_zero_capability_source_receipt_implementation_registration_confirmed: true, no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn approval_requires_every_independent_confirmation() {
        let mut value = request();
        assert!(request_checks(&value));
        value.statement_values_informational_and_independent_marks_fx_derivatives_required_confirmed = false;
        assert!(!request_checks(&value));
        assert!(validate_request(&"d".repeat(32), &value).is_ok());
    }

    #[test]
    fn independent_schema_requires_complete_accounts_options_liabilities_and_unsettled_activity() {
        let value = independent_snapshot_schema();
        assert!(
            value
                .account_scope_completeness_rule
                .contains("every_account")
        );
        assert!(value.listed_option_extension_schema.contains("occ_symbol"));
        assert!(
            value
                .liabilities_and_unsettled_activity_rule
                .contains("pending_trades")
        );
        assert!(value.unsupported_asset_class_result.contains("no_partial"));
    }

    #[test]
    fn statement_values_never_become_accounting_marks() {
        let value = independent_snapshot_schema();
        assert!(
            value
                .statement_market_value_rule
                .contains("never_an_accounting_mark")
        );
        assert!(
            value
                .opening_nav_rule
                .contains("independent_raw_marks_FX_and_derivative_valuation")
        );
    }

    #[test]
    fn source_contract_formats_remain_original_provider_artifacts() {
        assert_eq!(
            independent_artifact_formats(),
            vec![
                "original_provider_pdf_statement",
                "original_provider_csv_export",
                "original_provider_json_export"
            ]
        );
    }

    #[test]
    fn independent_review_has_zero_financial_model_or_trading_authority() {
        let value = independent_closed_authority_boundary();
        assert!(!value.source_artifact_present);
        assert!(!value.source_artifact_uploaded_or_read);
        assert!(!value.opening_portfolio_snapshot_materialized);
        assert!(!value.ledger_created);
        assert!(!value.position_write_allowed);
        assert!(!value.cash_write_allowed);
        assert!(!value.nav_or_performance_write_allowed);
        assert!(!value.training_or_rl_feedback_allowed);
        assert!(!value.order_generation_allowed);
        assert!(!value.broker_access_allowed);
        assert!(!value.trading_allowed);
    }
}
