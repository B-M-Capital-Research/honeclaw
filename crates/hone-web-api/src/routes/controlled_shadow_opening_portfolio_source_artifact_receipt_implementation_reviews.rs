//! Stage 128 chain-external independent review of one Stage 127 zero-capability
//! opening-portfolio source-artifact receipt implementation contract.
//!
//! This module rebuilds the complete Stage 127 contract without calling its builder. Approval
//! opens only a future Stage 129 isolated receiver-specification registration. It exposes no
//! upload endpoint, receives or reads no source bytes, runs no parser, and creates no financial,
//! model, order, broker or trading state.

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

use super::controlled_shadow_opening_portfolio_source_artifact_receipt_implementations::{
    OpeningPortfolioSourceArtifactReceiptImplementationContract,
    OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
    ZeroCapabilityOpeningPortfolioSourceReceiptAuthorityBoundary,
    independently_reviewable_opening_portfolio_source_artifact_receipt_implementations,
    validate_opening_portfolio_source_artifact_receipt_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-implementation-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-implementation-independent-audit-v1";
const POLICY_VERSION: &str = "hone-opening-portfolio-source-artifact-receipt-implementation-chain-external-review-v1-zero-capability";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-zero-capability-contract-v1";
const PROTOCOL_VERSION: &str = "hone-opening-portfolio-source-artifact-receipt-v1-not-executable";
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_RECORD_FILE_BYTES: u64 = 1024 * 1024;
const FUTURE_MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const FUTURE_MAX_RECEIPT_BYTES: u64 = 256 * 1024 * 1024;
const FUTURE_MAX_ARTIFACT_COUNT: usize = 64;

const EXPECTED_FUNCTION_IDS: [&str; 8] = [
    "opening_portfolio_receipt_validate_declared_metadata_v1",
    "opening_portfolio_receipt_stream_private_quarantine_sha256_length_v1",
    "opening_portfolio_receipt_validate_format_magic_safe_structure_v1",
    "opening_portfolio_receipt_reject_archive_active_password_unsafe_path_v1",
    "opening_portfolio_receipt_pseudonymize_identity_redact_secrets_v1",
    "opening_portfolio_receipt_atomic_content_addressed_create_new_v1",
    "opening_portfolio_receipt_append_only_redacted_manifest_v1",
    "opening_portfolio_receipt_cleanup_partial_quarantine_v1",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict {
    ApprovedForFutureIsolatedSourceArtifactReceiverSpecificationRegistration,
    ChangesRequiredRebuildSourceArtifactReceiptImplementation,
    RejectedSourceArtifactReceiptImplementation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations {
    pub exact_current_stage_51_through_stage_127_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed:
        bool,
    pub complete_contract_rebuilt_without_stage_127_builder_confirmed: bool,
    pub all_stage_127_registration_confirmations_revalidated_confirmed: bool,
    pub original_provider_formats_and_resource_ceilings_preserved_confirmed: bool,
    pub administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: bool,
    pub streaming_sha256_length_private_quarantine_and_atomic_commit_confirmed: bool,
    pub format_magic_safe_structure_and_active_content_rejection_confirmed: bool,
    pub account_pseudonymization_and_secret_redaction_confirmed: bool,
    pub encryption_content_addressing_create_new_idempotency_and_failure_cleanup_confirmed: bool,
    pub server_received_time_redacted_manifest_and_untrusted_receipt_confirmed: bool,
    pub receipt_validation_materialization_output_validation_and_admission_remain_separate_confirmed:
        bool,
    pub no_upload_source_bytes_storage_write_parser_runtime_network_secret_tool_or_subprocess_confirmed:
        bool,
    pub no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub approval_only_opens_future_stage_129_isolated_receiver_specification_registration_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewOpeningPortfolioSourceArtifactReceiptImplementationRequest {
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_stage_126_review_sha256: String,
    expected_stage_126_independent_audit_sha256: String,
    expected_stage_125_registration_sha256: String,
    expected_stage_125_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict,
    rationale: String,
    binding_and_recomputation_assessment: String,
    transport_resource_and_format_assessment: String,
    privacy_storage_and_manifest_assessment: String,
    separation_and_zero_capability_assessment: String,
    known_limitations: String,
    future_receiver_constraints: String,
    confirmations: OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub stage_126_review_sha256: String,
    pub stage_126_independent_audit_sha256: String,
    pub stage_125_registration_sha256: String,
    pub stage_125_specification_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub stage_126_review_hash_independently_reproduced: bool,
    pub stage_126_independent_audit_hash_independently_reproduced: bool,
    pub stage_125_registration_hash_independently_reproduced: bool,
    pub stage_125_specification_hash_independently_reproduced: bool,
    pub complete_contract_rebuilt_without_stage_127_builder: bool,
    pub rebuilt_contract_exactly_matches_record: bool,
    pub exact_current_stage_51_through_stage_127_binding_valid: bool,
    pub all_stage_127_registration_confirmations_valid: bool,
    pub source_formats_transport_and_resource_ceilings_valid: bool,
    pub streaming_quarantine_format_and_active_content_rejection_valid: bool,
    pub privacy_encryption_content_addressing_and_failure_cleanup_valid: bool,
    pub manifest_untrusted_output_and_separation_contract_valid: bool,
    pub all_upload_source_parser_financial_model_order_broker_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub implementation: OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
    pub independent_audit: OpeningPortfolioSourceArtifactReceiptImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict,
    pub rationale: String,
    pub binding_and_recomputation_assessment: String,
    pub transport_resource_and_format_assessment: String,
    pub privacy_storage_and_manifest_assessment: String,
    pub separation_and_zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_receiver_constraints: String,
    pub confirmations: OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations,
    pub confirmations_complete: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub zero_capability_implementation_independently_approved: bool,
    pub future_stage_129_isolated_receiver_specification_registration_eligible: bool,
    pub isolated_receiver_specification_registered: bool,
    pub upload_endpoint_present: bool,
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
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationReviewItem {
    pub implementation: OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
    pub current_independent_audit:
        OpeningPortfolioSourceArtifactReceiptImplementationIndependentAudit,
    pub review: Option<OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_stage_129_isolated_receiver_specification_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<OpeningPortfolioSourceArtifactReceiptImplementationReviewItem>,
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_stage_129_isolated_receiver_specification_registration_eligible_count: usize,
    pub review_status: String,
    pub isolated_receiver_specification_registered: bool,
    pub upload_endpoint_present: bool,
    pub source_artifact_present: bool,
    pub source_artifact_uploaded_or_read: bool,
    pub parser_or_runtime_present: bool,
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
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationReviewReadinessSummary {
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_stage_129_isolated_receiver_specification_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation {
    pub implementation: OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
    pub review: OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord,
}

pub(crate) async fn handle_get_opening_portfolio_source_artifact_receipt_implementation_reviews(
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
            warn!(%error, "Stage 128 source receipt implementation review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "期初组合来源工件接收实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_opening_portfolio_source_artifact_receipt_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewOpeningPortfolioSourceArtifactReceiptImplementationRequest>,
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

pub(crate) async fn opening_portfolio_source_artifact_receipt_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationReviewReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        OpeningPortfolioSourceArtifactReceiptImplementationReviewReadinessSummary {
            implementation_count: value.implementation_count,
            review_eligible_count: value.review_eligible_count,
            reviewed_count: value.reviewed_count,
            independently_approved_count: value.independently_approved_count,
            changes_required_or_rejected_count: value.changes_required_or_rejected_count,
            future_stage_129_isolated_receiver_specification_registration_eligible_count: value
                .future_stage_129_isolated_receiver_specification_registration_eligible_count,
            review_status: value.review_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_approved_opening_portfolio_source_artifact_receipt_implementations_for_isolated_receiver_specification_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let review = item.review?;
            item.future_stage_129_isolated_receiver_specification_registration_eligible
                .then_some(
                    IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation {
                        implementation: item.implementation,
                        review,
                    },
                )
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry, String> {
    let implementations =
        independently_reviewable_opening_portfolio_source_artifact_receipt_implementations(state)
            .await?;
    let mut items = Vec::new();
    for implementation in implementations {
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 127 来源工件接收实现独立审计失败，晋级关闭".to_string());
        }
        let review = read_review(state, &implementation.implementation_id).await?;
        if review.as_ref().is_some_and(|value| {
            value.implementation != implementation || value.independent_audit != audit
        }) {
            return Err("Stage 128 复核绑定或独立审计已经漂移".to_string());
        }
        let approved = review.as_ref().is_some_and(|value| {
            value.future_stage_129_isolated_receiver_specification_registration_eligible
        });
        items.push(
            OpeningPortfolioSourceArtifactReceiptImplementationReviewItem {
                implementation,
                current_independent_audit: audit,
                review_eligible: review.is_none(),
                review,
                future_stage_129_isolated_receiver_specification_registration_eligible: approved,
            },
        );
    }
    items.sort_by(|left, right| {
        right
            .implementation
            .registered_at
            .cmp(&left.implementation.registered_at)
    });
    let implementation_count = items.len();
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items.iter().filter(|item| item.review.is_some()).count();
    let independently_approved_count = items
        .iter()
        .filter(|item| item.future_stage_129_isolated_receiver_specification_registration_eligible)
        .count();
    let changes_required_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.review.as_ref().is_some_and(|review| {
                !review.future_stage_129_isolated_receiver_specification_registration_eligible
            })
        })
        .count();
    let review_status = if items.is_empty() {
        "waiting_stage_127_source_artifact_receipt_implementation"
    } else if independently_approved_count > 0 {
        "source_artifact_receipt_implementation_independently_approved_only_for_stage_129_isolated_receiver_specification"
    } else if review_eligible_count > 0 {
        "source_artifact_receipt_implementation_ready_for_stage_128_chain_external_review"
    } else {
        "source_artifact_receipt_implementation_review_terminal_fail_closed"
    };
    Ok(OpeningPortfolioSourceArtifactReceiptImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        implementation_count,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_required_or_rejected_count,
        future_stage_129_isolated_receiver_specification_registration_eligible_count:
            independently_approved_count,
        review_status: review_status.to_string(),
        isolated_receiver_specification_registered: false,
        upload_endpoint_present: false,
        source_artifact_present: false,
        source_artifact_uploaded_or_read: false,
        parser_or_runtime_present: false,
        opening_portfolio_snapshot_present: false,
        ledger_created: false,
        position_or_cash_written: false,
        nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 128 由 Stage 51–127 完整责任链外的新角色，用不调用 Stage 127 builder 的第二实现重建完整来源工件接收合同，独立重算 implementation/contract、Stage 126 review/audit 与 Stage 125 registration/specification 摘要，并逐项复核 17 个确认、原始格式、资源上限、管理员认证流、私有隔离、流式哈希、主动内容拒绝、匿名化/脱敏、静态加密、内容寻址、失败清理和未受信 manifest。批准只开放 Stage 129 隔离接收器规格登记；当前没有上传、来源字节、parser/runtime、快照、账本、绩效、训练、订单、券商或交易能力。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewOpeningPortfolioSourceArtifactReceiptImplementationRequest,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("Stage 128 复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementation =
        independently_reviewable_opening_portfolio_source_artifact_receipt_implementations(state)
            .await?
            .into_iter()
            .find(|value| value.implementation_id == implementation_id)
            .ok_or_else(|| "当前 Stage 127 实现不存在或上游绑定失效".to_string())?;
    if read_review(state, implementation_id).await?.is_some() {
        return Err("该 Stage 127 实现已有终态独立复核，禁止覆盖、追加或分叉".to_string());
    }
    let review = build_review(&implementation, reviewer_id, request, Utc::now())?;
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
    implementation: &OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
    reviewer_id: &str,
    request: ReviewOpeningPortfolioSourceArtifactReceiptImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord, String> {
    validate_opening_portfolio_source_artifact_receipt_implementation_for_review(implementation)?;
    let audit = independently_audit(implementation)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("Stage 127 来源工件接收实现独立审计不通过".to_string());
    }
    let upstream = &implementation.upstream_stage_126_review;
    if request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256
            != implementation.implementation_contract.contract_sha256
        || request.expected_stage_126_review_sha256 != upstream.review_sha256
        || request.expected_stage_126_independent_audit_sha256
            != upstream.independent_audit.audit_sha256
        || request.expected_stage_125_registration_sha256
            != upstream.registration.registration_sha256
        || request.expected_stage_125_specification_sha256
            != upstream.registration.specification.specification_sha256
        || request.expected_independent_audit_sha256 != audit.audit_sha256
    {
        return Err("Stage 125–127 实现、合同、审计或来源绑定已变化，请刷新".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(implementation);
    if excluded_prior_actor_ids
        .iter()
        .any(|value| value == reviewer_id)
    {
        return Err(
            "Stage 128 reviewer 必须独立于 registrar、Stage 126 reviewer 与完整责任链".to_string(),
        );
    }
    let confirmations_complete = confirmations_complete(&request.confirmations);
    let approved = request.verdict
        == OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict::ApprovedForFutureIsolatedSourceArtifactReceiverSpecificationRegistration;
    if approved && !confirmations_complete {
        return Err("批准前必须逐项确认第二实现重建、接收安全语义和零能力边界".to_string());
    }
    let mut review = OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        implementation: implementation.clone(),
        independent_audit: audit,
        submitted_at,
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        binding_and_recomputation_assessment: bounded_required(
            &request.binding_and_recomputation_assessment,
            "绑定与重算评估",
        )?,
        transport_resource_and_format_assessment: bounded_required(
            &request.transport_resource_and_format_assessment,
            "传输、资源与格式评估",
        )?,
        privacy_storage_and_manifest_assessment: bounded_required(
            &request.privacy_storage_and_manifest_assessment,
            "隐私、存储与 manifest 评估",
        )?,
        separation_and_zero_capability_assessment: bounded_required(
            &request.separation_and_zero_capability_assessment,
            "分离门与零能力评估",
        )?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_receiver_constraints: bounded_required(
            &request.future_receiver_constraints,
            "未来接收器约束",
        )?,
        confirmations: request.confirmations,
        confirmations_complete,
        reviewer_independent_from_registrar_and_complete_prior_chain: true,
        zero_capability_implementation_independently_approved: approved && confirmations_complete,
        future_stage_129_isolated_receiver_specification_registration_eligible: approved
            && confirmations_complete,
        isolated_receiver_specification_registered: false,
        upload_endpoint_present: false,
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
    validate_review(&review)?;
    Ok(review)
}

fn independently_audit(
    implementation: &OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationIndependentAudit, String> {
    let upstream = &implementation.upstream_stage_126_review;
    let registration = &upstream.registration;
    let specification = &registration.specification;
    let rebuilt = independently_rebuild_contract(implementation)?;
    let implementation_record_hash_independently_reproduced = implementation.implementation_sha256
        == fingerprint_without(
            implementation,
            &["implementation_id", "implementation_sha256"],
        )?;
    let implementation_contract_hash_independently_reproduced =
        implementation.implementation_contract.contract_sha256
            == fingerprint_without(
                &implementation.implementation_contract,
                &["contract_sha256"],
            )?;
    let stage_126_review_hash_independently_reproduced =
        upstream.review_sha256 == fingerprint_without(upstream, &["review_id", "review_sha256"])?;
    let stage_126_independent_audit_hash_independently_reproduced =
        upstream.independent_audit.audit_sha256
            == fingerprint_without(&upstream.independent_audit, &["audit_sha256"])?;
    let stage_125_registration_hash_independently_reproduced = registration.registration_sha256
        == fingerprint_without(registration, &["registration_id", "registration_sha256"])?;
    let stage_125_specification_hash_independently_reproduced = specification.specification_sha256
        == fingerprint_without(specification, &["specification_sha256"])?;
    let rebuilt_contract_exactly_matches_record = rebuilt == implementation.implementation_contract;
    let exact_current_stage_51_through_stage_127_binding_valid = rebuilt.stage_126_review_id
        == upstream.review_id
        && rebuilt.stage_126_review_sha256 == upstream.review_sha256
        && rebuilt.stage_126_independent_audit_sha256 == upstream.independent_audit.audit_sha256
        && rebuilt.stage_125_registration_id == registration.registration_id
        && rebuilt.stage_125_registration_sha256 == registration.registration_sha256
        && rebuilt.stage_125_specification_sha256 == specification.specification_sha256;
    let all_stage_127_registration_confirmations_valid = implementation.confirmations_complete
        && serde_json::to_value(&implementation.confirmations)
            .ok()
            .and_then(|value| value.as_object().cloned())
            .is_some_and(|values| {
                values.len() == 17 && values.values().all(|value| value.as_bool() == Some(true))
            });
    let source_formats_transport_and_resource_ceilings_valid = rebuilt
        .exact_source_artifact_contract
        .accepted_artifact_formats
        == expected_artifact_formats()
        && rebuilt.future_transport_scope
            == "server_authoritative_admin_authenticated_stream_only_no_remote_fetch"
        && rebuilt.future_maximum_artifact_bytes == FUTURE_MAX_ARTIFACT_BYTES
        && rebuilt.future_maximum_receipt_bytes == FUTURE_MAX_RECEIPT_BYTES
        && rebuilt.future_maximum_artifact_count == FUTURE_MAX_ARTIFACT_COUNT;
    let function_ids = contract_function_ids(&rebuilt);
    let streaming_quarantine_format_and_active_content_rejection_valid = function_ids
        == EXPECTED_FUNCTION_IDS
        && rebuilt.future_private_quarantine_relative_path_template
            == "opening-portfolio-source-artifact-quarantine/{stage_126_review_id}/{receipt_nonce}.part";
    let privacy_encryption_content_addressing_and_failure_cleanup_valid = rebuilt
        .original_bytes_preserved_immutable
        && rebuilt.encryption_at_rest_required
        && rebuilt.server_owned_received_at_required
        && !rebuilt.raw_account_numbers_or_credentials_in_paths_metadata_or_logs_allowed
        && !rebuilt.overwrite_or_mutable_artifact_allowed
        && rebuilt.future_content_addressed_artifact_relative_path_template
            == "opening-portfolio-source-artifacts/{stage_125_registration_id}/{artifact_sha256}/original.bin";
    let manifest_untrusted_output_and_separation_contract_valid = rebuilt
        .future_receipt_output_untrusted
        && rebuilt.future_independent_receipt_validation_required
        && rebuilt.future_snapshot_materialization_separate
        && rebuilt.future_snapshot_output_validation_separate
        && rebuilt.future_snapshot_admission_review_separate
        && !rebuilt.financial_row_parsing_allowed_in_receipt_stage;
    let all_upload_source_parser_financial_model_order_broker_and_trading_authority_closed =
        rebuilt.authority_boundary == closed_authority_boundary();
    let checks = [
        (
            implementation_record_hash_independently_reproduced,
            "implementation_record_hash_mismatch",
        ),
        (
            implementation_contract_hash_independently_reproduced,
            "implementation_contract_hash_mismatch",
        ),
        (
            stage_126_review_hash_independently_reproduced,
            "stage_126_review_hash_mismatch",
        ),
        (
            stage_126_independent_audit_hash_independently_reproduced,
            "stage_126_audit_hash_mismatch",
        ),
        (
            stage_125_registration_hash_independently_reproduced,
            "stage_125_registration_hash_mismatch",
        ),
        (
            stage_125_specification_hash_independently_reproduced,
            "stage_125_specification_hash_mismatch",
        ),
        (
            rebuilt_contract_exactly_matches_record,
            "independent_contract_rebuild_mismatch",
        ),
        (
            exact_current_stage_51_through_stage_127_binding_valid,
            "current_binding_mismatch",
        ),
        (
            all_stage_127_registration_confirmations_valid,
            "stage_127_confirmations_incomplete",
        ),
        (
            source_formats_transport_and_resource_ceilings_valid,
            "format_transport_or_resource_mismatch",
        ),
        (
            streaming_quarantine_format_and_active_content_rejection_valid,
            "streaming_quarantine_or_rejection_mismatch",
        ),
        (
            privacy_encryption_content_addressing_and_failure_cleanup_valid,
            "privacy_storage_or_cleanup_mismatch",
        ),
        (
            manifest_untrusted_output_and_separation_contract_valid,
            "manifest_or_separation_mismatch",
        ),
        (
            all_upload_source_parser_financial_model_order_broker_and_trading_authority_closed,
            "authority_boundary_open",
        ),
    ];
    let mismatch_reasons = checks
        .into_iter()
        .filter_map(|(valid, reason)| (!valid).then_some(reason.to_string()))
        .collect();
    let mut audit = OpeningPortfolioSourceArtifactReceiptImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: implementation
            .implementation_contract
            .contract_sha256
            .clone(),
        stage_126_review_sha256: upstream.review_sha256.clone(),
        stage_126_independent_audit_sha256: upstream.independent_audit.audit_sha256.clone(),
        stage_125_registration_sha256: registration.registration_sha256.clone(),
        stage_125_specification_sha256: specification.specification_sha256.clone(),
        implementation_record_hash_independently_reproduced,
        implementation_contract_hash_independently_reproduced,
        stage_126_review_hash_independently_reproduced,
        stage_126_independent_audit_hash_independently_reproduced,
        stage_125_registration_hash_independently_reproduced,
        stage_125_specification_hash_independently_reproduced,
        complete_contract_rebuilt_without_stage_127_builder: true,
        rebuilt_contract_exactly_matches_record,
        exact_current_stage_51_through_stage_127_binding_valid,
        all_stage_127_registration_confirmations_valid,
        source_formats_transport_and_resource_ceilings_valid,
        streaming_quarantine_format_and_active_content_rejection_valid,
        privacy_encryption_content_addressing_and_failure_cleanup_valid,
        manifest_untrusted_output_and_separation_contract_valid,
        all_upload_source_parser_financial_model_order_broker_and_trading_authority_closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn independently_rebuild_contract(
    implementation: &OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationContract, String> {
    let upstream = &implementation.upstream_stage_126_review;
    let specification = upstream.registration.specification.clone();
    let mut contract = OpeningPortfolioSourceArtifactReceiptImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        immutable_code_revision: implementation
            .implementation_contract
            .immutable_code_revision
            .clone(),
        stage_126_review_id: upstream.review_id.clone(),
        stage_126_review_sha256: upstream.review_sha256.clone(),
        stage_126_independent_audit_sha256: upstream.independent_audit.audit_sha256.clone(),
        stage_125_registration_id: upstream.registration.registration_id.clone(),
        stage_125_registration_sha256: upstream.registration.registration_sha256.clone(),
        stage_125_specification_sha256: specification.specification_sha256.clone(),
        exact_source_artifact_contract: specification.source_contract.clone(),
        exact_stage_125_specification: specification,
        future_transport_scope:
            "server_authoritative_admin_authenticated_stream_only_no_remote_fetch".to_string(),
        future_maximum_artifact_bytes: FUTURE_MAX_ARTIFACT_BYTES,
        future_maximum_receipt_bytes: FUTURE_MAX_RECEIPT_BYTES,
        future_maximum_artifact_count: FUTURE_MAX_ARTIFACT_COUNT,
        validate_declared_metadata_before_byte_acceptance_function_id: EXPECTED_FUNCTION_IDS[0]
            .to_string(),
        stream_private_quarantine_while_hashing_and_counting_function_id:
            EXPECTED_FUNCTION_IDS[1].to_string(),
        validate_format_magic_and_safe_structure_without_financial_parsing_function_id:
            EXPECTED_FUNCTION_IDS[2].to_string(),
        reject_archive_active_content_password_and_unsafe_path_function_id:
            EXPECTED_FUNCTION_IDS[3].to_string(),
        pseudonymize_account_identity_and_redact_secrets_function_id:
            EXPECTED_FUNCTION_IDS[4].to_string(),
        atomic_content_addressed_create_new_commit_function_id: EXPECTED_FUNCTION_IDS[5]
            .to_string(),
        append_only_redacted_receipt_manifest_function_id: EXPECTED_FUNCTION_IDS[6].to_string(),
        cleanup_partial_quarantine_on_failure_function_id: EXPECTED_FUNCTION_IDS[7].to_string(),
        future_private_quarantine_relative_path_template:
            "opening-portfolio-source-artifact-quarantine/{stage_126_review_id}/{receipt_nonce}.part"
                .to_string(),
        future_content_addressed_artifact_relative_path_template:
            "opening-portfolio-source-artifacts/{stage_125_registration_id}/{artifact_sha256}/original.bin"
                .to_string(),
        future_receipt_manifest_schema: "receipt_id,stage_126_review_id,stage_125_registration_id,portfolio_scope_alias,artifact_sha256,byte_length,declared_format,validated_format,provider_artifact_identifier_sha256,provider_generated_at_or_statement_as_of,hone_received_at,source_account_aliases,storage_object_sha256,receipt_manifest_sha256,untrusted".to_string(),
        original_bytes_preserved_immutable: true,
        encryption_at_rest_required: true,
        server_owned_received_at_required: true,
        raw_account_numbers_or_credentials_in_paths_metadata_or_logs_allowed: false,
        overwrite_or_mutable_artifact_allowed: false,
        financial_row_parsing_allowed_in_receipt_stage: false,
        future_receipt_output_untrusted: true,
        future_independent_receipt_validation_required: true,
        future_snapshot_materialization_separate: true,
        future_snapshot_output_validation_separate: true,
        future_snapshot_admission_review_separate: true,
        registered_not_run: true,
        independent_implementation_review_required: true,
        future_isolated_receiver_registration_required_after_review: true,
        authority_boundary: closed_authority_boundary(),
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn validate_review(
    review: &OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord,
) -> Result<(), String> {
    validate_opening_portfolio_source_artifact_receipt_implementation_for_review(
        &review.implementation,
    )?;
    let expected_audit = independently_audit(&review.implementation)?;
    let approved = review.verdict
        == OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict::ApprovedForFutureIsolatedSourceArtifactReceiverSpecificationRegistration;
    let downstream_closed = !review.isolated_receiver_specification_registered
        && !review.upload_endpoint_present
        && !review.source_artifact_present
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
        && review.independent_audit == expected_audit
        && review.independent_audit.mismatch_reasons.is_empty()
        && review.excluded_prior_actor_ids == expected_excluded_actor_ids(&review.implementation)
        && !review
            .excluded_prior_actor_ids
            .contains(&review.reviewer_id)
        && review.reviewer_independent_from_registrar_and_complete_prior_chain
        && review.confirmations_complete == confirmations_complete(&review.confirmations)
        && (!approved || review.confirmations_complete)
        && review.zero_capability_implementation_independently_approved
            == (approved && review.confirmations_complete)
        && review.future_stage_129_isolated_receiver_specification_registration_eligible
            == (approved && review.confirmations_complete)
        && !review.rationale.trim().is_empty()
        && !review
            .binding_and_recomputation_assessment
            .trim()
            .is_empty()
        && !review
            .transport_resource_and_format_assessment
            .trim()
            .is_empty()
        && !review
            .privacy_storage_and_manifest_assessment
            .trim()
            .is_empty()
        && !review
            .separation_and_zero_capability_assessment
            .trim()
            .is_empty()
        && !review.known_limitations.trim().is_empty()
        && !review.future_receiver_constraints.trim().is_empty()
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 128 来源工件接收实现复核无效、漂移或越权".to_string())
}

fn confirmations_complete(
    value: &OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations,
) -> bool {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .is_some_and(|values| {
            values.len() == 17 && values.values().all(|value| value.as_bool() == Some(true))
        })
}

fn expected_excluded_actor_ids(
    implementation: &OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
) -> Vec<String> {
    let mut values = implementation.excluded_prior_actor_ids.clone();
    values.push(implementation.registered_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn expected_artifact_formats() -> Vec<String> {
    vec![
        "original_provider_pdf_statement".to_string(),
        "original_provider_csv_export".to_string(),
        "original_provider_json_export".to_string(),
    ]
}

fn contract_function_ids(
    contract: &OpeningPortfolioSourceArtifactReceiptImplementationContract,
) -> [&str; 8] {
    [
        &contract.validate_declared_metadata_before_byte_acceptance_function_id,
        &contract.stream_private_quarantine_while_hashing_and_counting_function_id,
        &contract.validate_format_magic_and_safe_structure_without_financial_parsing_function_id,
        &contract.reject_archive_active_content_password_and_unsafe_path_function_id,
        &contract.pseudonymize_account_identity_and_redact_secrets_function_id,
        &contract.atomic_content_addressed_create_new_commit_function_id,
        &contract.append_only_redacted_receipt_manifest_function_id,
        &contract.cleanup_partial_quarantine_on_failure_function_id,
    ]
}

fn closed_authority_boundary() -> ZeroCapabilityOpeningPortfolioSourceReceiptAuthorityBoundary {
    ZeroCapabilityOpeningPortfolioSourceReceiptAuthorityBoundary {
        upload_endpoint_present: false,
        source_artifact_present: false,
        source_artifact_uploaded_or_read: false,
        source_artifact_storage_write_allowed: false,
        executable_artifact_or_entrypoint_present: false,
        runtime_present: false,
        parser_present: false,
        environment_inheritance_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_or_subprocesses_allowed: false,
        opening_portfolio_snapshot_materialized: false,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        position_or_cash_write_allowed: false,
        nav_or_performance_write_allowed: false,
        model_or_metric_store_write_allowed: false,
        training_or_rl_feedback_allowed: false,
        reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    }
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!(
            "Stage 128 {label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"
        ))
    } else {
        Ok(value.to_string())
    }
}

fn fingerprint_without<T: Serialize>(value: &T, excluded: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Stage 128 指纹对象不是 JSON object".to_string())?;
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

async fn read_review(
    state: &AppState,
    implementation_id: &str,
) -> Result<Option<OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord>, String> {
    let directory = review_root(state).join(implementation_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
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
            return Err("Stage 128 复核目录含非 JSON 文件".to_string());
        }
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() == 0
            || metadata.len() > MAX_RECORD_FILE_BYTES
        {
            return Err("Stage 128 复核文件无效或超限".to_string());
        }
        let review: OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord =
            serde_json::from_slice(
                &tokio::fs::read(&path)
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_review(&review)?;
        if review.implementation.implementation_id != implementation_id
            || path.file_stem().and_then(|value| value.to_str()) != Some(review.review_id.as_str())
        {
            return Err("Stage 128 复核路径与绑定 ID 不一致".to_string());
        }
        values.push(review);
    }
    if values.len() > 1 {
        return Err("Stage 128 同一实现出现重复或分叉终态复核".to_string());
    }
    Ok(values.pop())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_RECORD_FILE_BYTES {
        return Err("Stage 128 复核文件为空或超限".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "Stage 128 复核路径缺少父目录".to_string())?;
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
                "Stage 128 复核已存在，禁止覆盖".to_string()
            } else {
                error.to_string()
            }
        })?;
    file.write_all(&bytes)
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
    decision_root(state).join("opening-portfolio-source-artifact-receipt-implementation-reviews")
}

struct ReviewLock(PathBuf);

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, implementation_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage128-{implementation_id}.lock"));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 128 独立复核正在进行".to_string())?;
    Ok(ReviewLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirmations() -> OpeningPortfolioSourceArtifactReceiptImplementationReviewConfirmations {
        serde_json::from_value(serde_json::json!({
            "exact_current_stage_51_through_stage_127_binding_confirmed": true,
            "reviewer_independent_from_registrar_and_complete_prior_chain_confirmed": true,
            "implementation_contract_review_audit_registration_and_specification_hashes_independently_reproduced_confirmed": true,
            "complete_contract_rebuilt_without_stage_127_builder_confirmed": true,
            "all_stage_127_registration_confirmations_revalidated_confirmed": true,
            "original_provider_formats_and_resource_ceilings_preserved_confirmed": true,
            "administrator_authenticated_stream_only_and_no_remote_fetch_confirmed": true,
            "streaming_sha256_length_private_quarantine_and_atomic_commit_confirmed": true,
            "format_magic_safe_structure_and_active_content_rejection_confirmed": true,
            "account_pseudonymization_and_secret_redaction_confirmed": true,
            "encryption_content_addressing_create_new_idempotency_and_failure_cleanup_confirmed": true,
            "server_received_time_redacted_manifest_and_untrusted_receipt_confirmed": true,
            "receipt_validation_materialization_output_validation_and_admission_remain_separate_confirmed": true,
            "no_upload_source_bytes_storage_write_parser_runtime_network_secret_tool_or_subprocess_confirmed": true,
            "no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed": true,
            "approval_only_opens_future_stage_129_isolated_receiver_specification_registration_confirmed": true,
            "no_unconfirmed_hari_or_old_wang_logic_claimed": true
        }))
        .expect("valid confirmations")
    }

    #[test]
    fn approval_requires_all_seventeen_independent_confirmations() {
        let value = confirmations();
        assert!(confirmations_complete(&value));
        assert_eq!(
            serde_json::to_value(value)
                .unwrap()
                .as_object()
                .unwrap()
                .len(),
            17
        );
    }

    #[test]
    fn independent_contract_function_ids_are_frozen() {
        assert_eq!(EXPECTED_FUNCTION_IDS.len(), 8);
        assert!(EXPECTED_FUNCTION_IDS.iter().all(|value| !value.is_empty()));
        assert!(
            EXPECTED_FUNCTION_IDS
                .iter()
                .all(|value| value.starts_with("opening_portfolio_receipt_"))
        );
    }

    #[test]
    fn source_formats_and_resource_ceilings_remain_bounded() {
        assert_eq!(expected_artifact_formats().len(), 3);
        assert_eq!(FUTURE_MAX_ARTIFACT_BYTES, 64 * 1024 * 1024);
        assert_eq!(FUTURE_MAX_RECEIPT_BYTES, 256 * 1024 * 1024);
        assert_eq!(FUTURE_MAX_ARTIFACT_COUNT, 64);
    }

    #[test]
    fn review_authority_boundary_has_no_upload_or_financial_capability() {
        let value = closed_authority_boundary();
        assert!(!value.upload_endpoint_present);
        assert!(!value.source_artifact_storage_write_allowed);
        assert!(!value.parser_present);
        assert!(!value.opening_portfolio_snapshot_materialized);
        assert!(!value.ledger_created);
        assert!(!value.position_or_cash_write_allowed);
        assert!(!value.nav_or_performance_write_allowed);
        assert!(!value.training_or_rl_feedback_allowed);
        assert!(!value.order_generation_allowed);
        assert!(!value.broker_access_allowed);
        assert!(!value.trading_allowed);
    }

    #[test]
    fn only_explicit_approval_can_open_stage_129() {
        assert_ne!(
            OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict::ApprovedForFutureIsolatedSourceArtifactReceiverSpecificationRegistration,
            OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict::ChangesRequiredRebuildSourceArtifactReceiptImplementation
        );
        assert_ne!(
            OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict::ApprovedForFutureIsolatedSourceArtifactReceiverSpecificationRegistration,
            OpeningPortfolioSourceArtifactReceiptImplementationReviewVerdict::RejectedSourceArtifactReceiptImplementation
        );
    }
}
