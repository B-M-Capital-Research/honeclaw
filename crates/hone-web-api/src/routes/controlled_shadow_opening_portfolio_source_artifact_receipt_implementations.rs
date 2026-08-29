//! Stage 127 create-once registration of a zero-capability source-artifact receipt
//! implementation contract for one independently approved Stage 126 opening-portfolio
//! governance specification.
//!
//! This module stores contract metadata only. It exposes no upload endpoint, receives and reads
//! no source bytes, has no parser or executable artifact, and cannot materialize an opening
//! snapshot or create any ledger, position, cash, NAV/performance, training, order or trade state.

use std::collections::{BTreeSet, HashMap};
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

use super::controlled_shadow_opening_portfolio_snapshot_governance_specification_reviews::{
    OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord,
    independently_approved_opening_portfolio_snapshot_governance_specifications_for_source_receipt_implementation_registration,
};
use super::controlled_shadow_opening_portfolio_snapshot_governance_specifications::{
    OpeningPortfolioSnapshotGovernanceSpecification, OpeningPortfolioSourceArtifactContract,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-implementation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-implementation-registration-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-zero-capability-contract-v1";
const POLICY_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-implementation-create-once-v1-zero-capability";
const PROTOCOL_VERSION: &str = "hone-opening-portfolio-source-artifact-receipt-v1-not-executable";
const STATUS: &str = "registered_not_reviewed_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_RECORD_FILE_BYTES: u64 = 1024 * 1024;
const FUTURE_MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const FUTURE_MAX_RECEIPT_BYTES: u64 = 256 * 1024 * 1024;
const FUTURE_MAX_ARTIFACT_COUNT: usize = 64;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest {
    expected_stage_126_review_id: String,
    expected_stage_126_review_sha256: String,
    expected_stage_126_independent_audit_sha256: String,
    expected_stage_125_registration_id: String,
    expected_stage_125_registration_sha256: String,
    expected_stage_125_specification_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_description: String,
    transport_and_authentication_semantics: String,
    streaming_hash_length_and_atomic_commit_semantics: String,
    format_magic_and_active_content_rejection_semantics: String,
    pseudonymization_and_secret_redaction_semantics: String,
    quarantine_cleanup_and_idempotency_semantics: String,
    audit_and_retention_semantics: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_current_stage_51_through_stage_126_binding_confirmed: bool,
    registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed: bool,
    review_registration_specification_and_audit_hashes_recomputed_confirmed: bool,
    exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed: bool,
    original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed: bool,
    content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed:
        bool,
    archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed:
        bool,
    source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed:
        bool,
    private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed: bool,
    server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed:
        bool,
    duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: bool,
    receipt_output_untrusted_and_independent_receipt_validation_required_confirmed: bool,
    receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed:
        bool,
    contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed:
        bool,
    no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    future_stage_128_independent_implementation_review_required_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityOpeningPortfolioSourceReceiptAuthorityBoundary {
    pub upload_endpoint_present: bool,
    pub source_artifact_present: bool,
    pub source_artifact_uploaded_or_read: bool,
    pub source_artifact_storage_write_allowed: bool,
    pub executable_artifact_or_entrypoint_present: bool,
    pub runtime_present: bool,
    pub parser_present: bool,
    pub environment_inheritance_allowed: bool,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_or_subprocesses_allowed: bool,
    pub opening_portfolio_snapshot_materialized: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub position_or_cash_write_allowed: bool,
    pub nav_or_performance_write_allowed: bool,
    pub model_or_metric_store_write_allowed: bool,
    pub training_or_rl_feedback_allowed: bool,
    pub reward_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub protocol_version: String,
    pub immutable_code_revision: String,
    pub stage_126_review_id: String,
    pub stage_126_review_sha256: String,
    pub stage_126_independent_audit_sha256: String,
    pub stage_125_registration_id: String,
    pub stage_125_registration_sha256: String,
    pub stage_125_specification_sha256: String,
    pub exact_stage_125_specification: OpeningPortfolioSnapshotGovernanceSpecification,
    pub exact_source_artifact_contract: OpeningPortfolioSourceArtifactContract,
    pub future_transport_scope: String,
    pub future_maximum_artifact_bytes: u64,
    pub future_maximum_receipt_bytes: u64,
    pub future_maximum_artifact_count: usize,
    pub validate_declared_metadata_before_byte_acceptance_function_id: String,
    pub stream_private_quarantine_while_hashing_and_counting_function_id: String,
    pub validate_format_magic_and_safe_structure_without_financial_parsing_function_id: String,
    pub reject_archive_active_content_password_and_unsafe_path_function_id: String,
    pub pseudonymize_account_identity_and_redact_secrets_function_id: String,
    pub atomic_content_addressed_create_new_commit_function_id: String,
    pub append_only_redacted_receipt_manifest_function_id: String,
    pub cleanup_partial_quarantine_on_failure_function_id: String,
    pub future_private_quarantine_relative_path_template: String,
    pub future_content_addressed_artifact_relative_path_template: String,
    pub future_receipt_manifest_schema: String,
    pub original_bytes_preserved_immutable: bool,
    pub encryption_at_rest_required: bool,
    pub server_owned_received_at_required: bool,
    pub raw_account_numbers_or_credentials_in_paths_metadata_or_logs_allowed: bool,
    pub overwrite_or_mutable_artifact_allowed: bool,
    pub financial_row_parsing_allowed_in_receipt_stage: bool,
    pub future_receipt_output_untrusted: bool,
    pub future_independent_receipt_validation_required: bool,
    pub future_snapshot_materialization_separate: bool,
    pub future_snapshot_output_validation_separate: bool,
    pub future_snapshot_admission_review_separate: bool,
    pub registered_not_run: bool,
    pub independent_implementation_review_required: bool,
    pub future_isolated_receiver_registration_required_after_review: bool,
    pub authority_boundary: ZeroCapabilityOpeningPortfolioSourceReceiptAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationConfirmations {
    pub exact_current_stage_51_through_stage_126_binding_confirmed: bool,
    pub registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed: bool,
    pub review_registration_specification_and_audit_hashes_recomputed_confirmed: bool,
    pub exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed: bool,
    pub original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed: bool,
    pub content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed:
        bool,
    pub archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed:
        bool,
    pub source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed:
        bool,
    pub private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed:
        bool,
    pub server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed:
        bool,
    pub duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed:
        bool,
    pub receipt_output_untrusted_and_independent_receipt_validation_required_confirmed: bool,
    pub receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed:
        bool,
    pub contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed:
        bool,
    pub no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub future_stage_128_independent_implementation_review_required_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationRegistration {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub upstream_stage_126_review: OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_126_reviewer_and_complete_prior_chain: bool,
    pub implementation_name: String,
    pub implementation_description: String,
    pub transport_and_authentication_semantics: String,
    pub streaming_hash_length_and_atomic_commit_semantics: String,
    pub format_magic_and_active_content_rejection_semantics: String,
    pub pseudonymization_and_secret_redaction_semantics: String,
    pub quarantine_cleanup_and_idempotency_semantics: String,
    pub audit_and_retention_semantics: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub implementation_contract: OpeningPortfolioSourceArtifactReceiptImplementationContract,
    pub status: String,
    pub confirmations: OpeningPortfolioSourceArtifactReceiptImplementationConfirmations,
    pub confirmations_complete: bool,
    pub zero_capability_implementation_contract_registered: bool,
    pub future_stage_128_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub source_artifact_receipt_eligible: bool,
    pub source_artifact_present: bool,
    pub source_artifact_uploaded_or_read: bool,
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
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationItem {
    pub specification_review: OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord,
    pub implementation: Option<OpeningPortfolioSourceArtifactReceiptImplementationRegistration>,
    pub registration_eligible: bool,
    pub upstream_binding_current: bool,
    pub future_stage_128_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_endpoint_available: bool,
    pub items: Vec<OpeningPortfolioSourceArtifactReceiptImplementationItem>,
    pub independently_approved_specification_count: usize,
    pub registration_eligible_count: usize,
    pub implementation_contract_count: usize,
    pub current_binding_implementation_contract_count: usize,
    pub future_stage_128_independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub upload_endpoint_present: bool,
    pub source_artifact_present: bool,
    pub source_artifact_uploaded_or_read: bool,
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
pub(crate) struct OpeningPortfolioSourceArtifactReceiptImplementationReadinessSummary {
    pub independently_approved_specification_count: usize,
    pub registration_eligible_count: usize,
    pub implementation_contract_count: usize,
    pub current_binding_implementation_contract_count: usize,
    pub future_stage_128_independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_opening_portfolio_source_artifact_receipt_implementations(
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
            warn!(%error, "Stage 127 source artifact receipt implementation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "期初组合来源工件接收实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_opening_portfolio_source_artifact_receipt_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(stage_126_review_id): AxumPath<String>,
    Json(request): Json<RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &stage_126_review_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn opening_portfolio_source_artifact_receipt_implementation_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        OpeningPortfolioSourceArtifactReceiptImplementationReadinessSummary {
            independently_approved_specification_count: value
                .independently_approved_specification_count,
            registration_eligible_count: value.registration_eligible_count,
            implementation_contract_count: value.implementation_contract_count,
            current_binding_implementation_contract_count: value
                .current_binding_implementation_contract_count,
            future_stage_128_independent_implementation_review_eligible_count: value
                .future_stage_128_independent_implementation_review_eligible_count,
            implementation_status: value.implementation_status,
        },
    )
}

pub(crate) async fn independently_reviewable_opening_portfolio_source_artifact_receipt_implementations(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptImplementationRegistration>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            item.future_stage_128_independent_implementation_review_eligible
                .then_some(item.implementation)
                .flatten()
        })
        .collect())
}

pub(crate) fn validate_opening_portfolio_source_artifact_receipt_implementation_for_review(
    value: &OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
) -> Result<(), String> {
    validate_record(value)
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationRegistry, String> {
    let sources = independently_approved_opening_portfolio_snapshot_governance_specifications_for_source_receipt_implementation_registration(state).await?;
    let records = read_records(state).await?;
    for record in &records {
        let source = sources
            .iter()
            .find(|source| source.review_id == record.upstream_stage_126_review.review_id)
            .ok_or_else(|| {
                "Stage 127 存在不再属于当前独立批准 Stage 126 集合的孤立登记".to_string()
            })?;
        if !record_matches_source(record, source) {
            return Err("Stage 127 登记与当前 Stage 126 来源绑定漂移".to_string());
        }
    }
    let by_review = records
        .iter()
        .map(|record| (record.upstream_stage_126_review.review_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let mut items = Vec::new();
    for source in sources {
        let implementation = by_review
            .get(source.review_id.as_str())
            .map(|value| (*value).clone());
        let upstream_binding_current = implementation
            .as_ref()
            .is_some_and(|record| record_matches_source(record, &source));
        let registration_eligible = implementation.is_none();
        items.push(OpeningPortfolioSourceArtifactReceiptImplementationItem {
            specification_review: source,
            implementation,
            registration_eligible,
            upstream_binding_current,
            future_stage_128_independent_implementation_review_eligible: upstream_binding_current,
        });
    }
    items.sort_by(|left, right| {
        right
            .specification_review
            .submitted_at
            .cmp(&left.specification_review.submitted_at)
    });
    let independently_approved_specification_count = items.len();
    let registration_eligible_count = items
        .iter()
        .filter(|item| item.registration_eligible)
        .count();
    let implementation_contract_count = records.len();
    let current_binding_implementation_contract_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let future_stage_128_independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_stage_128_independent_implementation_review_eligible)
        .count();
    let implementation_status = if current_binding_implementation_contract_count > 0 {
        "zero_capability_source_artifact_receipt_implementation_registered_waiting_stage_128_independent_review"
    } else if registration_eligible_count > 0 {
        "independently_approved_opening_portfolio_governance_specification_ready_for_stage_127_implementation_registration"
    } else {
        "waiting_stage_126_independently_approved_opening_portfolio_governance_specification"
    };
    Ok(
        OpeningPortfolioSourceArtifactReceiptImplementationRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            registration_endpoint_available: true,
            items,
            independently_approved_specification_count,
            registration_eligible_count,
            implementation_contract_count,
            current_binding_implementation_contract_count,
            future_stage_128_independent_implementation_review_eligible_count,
            implementation_status: implementation_status.to_string(),
            upload_endpoint_present: false,
            source_artifact_present: false,
            source_artifact_uploaded_or_read: false,
            opening_portfolio_snapshot_present: false,
            ledger_created: false,
            position_or_cash_written: false,
            nav_or_performance_written: false,
            training_or_rl_feedback_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "Stage 127 只登记未来来源工件接收器的零能力合同，冻结流式 SHA-256/长度、私有隔离区、格式与魔数、主动内容拒绝、账号匿名化、日志脱敏、加密静态存储、内容寻址 create-new、失败清理和未受信 receipt manifest。当前没有上传入口、来源字节、parser/runtime、期初组合、账本、持仓、现金、NAV/绩效、训练/RL、订单、券商或交易；下一门只能是 Stage 128 责任链外实现复核。".to_string(),
        },
    )
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    stage_126_review_id: &str,
    request: RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationRegistration, String> {
    validate_request(stage_126_review_id, &request)?;
    let _lock = acquire_lock(state, stage_126_review_id).await?;
    let source = independently_approved_opening_portfolio_snapshot_governance_specifications_for_source_receipt_implementation_registration(state)
        .await?
        .into_iter()
        .find(|value| value.review_id == stage_126_review_id)
        .ok_or_else(|| "当前没有精确匹配且独立批准的 Stage 126 期初组合治理规格".to_string())?;
    validate_expected_binding(&source, &request)?;
    if read_records(state)
        .await?
        .iter()
        .any(|record| record.upstream_stage_126_review.review_id == stage_126_review_id)
    {
        return Err("该 Stage 126 规格复核已登记来源工件接收实现，禁止覆盖或重放".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&source);
    let independent = !registrar_id.trim().is_empty()
        && excluded_prior_actor_ids
            .iter()
            .all(|actor| actor != registrar_id);
    if !independent
        || !request.registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed
    {
        return Err(
            "Stage 127 registrar 必须独立于 Stage 126 reviewer 与完整既有责任链".to_string(),
        );
    }
    if !confirmations_complete(&request) {
        return Err("Stage 127 登记前必须逐项确认完整绑定、私密接收语义和零能力边界".to_string());
    }
    let implementation_name =
        bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    let immutable_code_revision = bounded_required(
        &request.immutable_code_revision,
        MAX_REVISION_CHARS,
        "不可变代码版本",
    )?;
    let implementation_contract = implementation_contract(&source, &immutable_code_revision)?;
    let confirmations = confirmations_from_request(&request);
    let mut record = OpeningPortfolioSourceArtifactReceiptImplementationRegistration {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at: Utc::now(),
        registered_by: registrar_id.to_string(),
        upstream_stage_126_review: source,
        excluded_prior_actor_ids,
        registrar_independent_from_stage_126_reviewer_and_complete_prior_chain: independent,
        implementation_name,
        implementation_description: bounded_text(&request.implementation_description, "实现说明")?,
        transport_and_authentication_semantics: bounded_text(
            &request.transport_and_authentication_semantics,
            "传输与鉴权语义",
        )?,
        streaming_hash_length_and_atomic_commit_semantics: bounded_text(
            &request.streaming_hash_length_and_atomic_commit_semantics,
            "流式哈希、长度与原子提交语义",
        )?,
        format_magic_and_active_content_rejection_semantics: bounded_text(
            &request.format_magic_and_active_content_rejection_semantics,
            "格式、魔数与主动内容拒绝语义",
        )?,
        pseudonymization_and_secret_redaction_semantics: bounded_text(
            &request.pseudonymization_and_secret_redaction_semantics,
            "匿名化与敏感信息脱敏语义",
        )?,
        quarantine_cleanup_and_idempotency_semantics: bounded_text(
            &request.quarantine_cleanup_and_idempotency_semantics,
            "隔离区、失败清理与幂等语义",
        )?,
        audit_and_retention_semantics: bounded_text(
            &request.audit_and_retention_semantics,
            "审计与保留语义",
        )?,
        known_limitations: bounded_text(&request.known_limitations, "已知限制")?,
        future_review_constraints: bounded_text(
            &request.future_review_constraints,
            "后续复核约束",
        )?,
        implementation_contract,
        status: STATUS.to_string(),
        confirmations,
        confirmations_complete: true,
        zero_capability_implementation_contract_registered: true,
        future_stage_128_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        source_artifact_receipt_eligible: false,
        source_artifact_present: false,
        source_artifact_uploaded_or_read: false,
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
    record.implementation_sha256 =
        fingerprint_without(&record, &["implementation_id", "implementation_sha256"])?;
    record.implementation_id = record.implementation_sha256[..32].to_string();
    validate_record(&record)?;
    write_immutable_json(
        &record_dir(state).join(format!("{}.json", record.implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn implementation_contract(
    source: &OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord,
    immutable_code_revision: &str,
) -> Result<OpeningPortfolioSourceArtifactReceiptImplementationContract, String> {
    let specification = source.registration.specification.clone();
    let mut contract = OpeningPortfolioSourceArtifactReceiptImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        stage_126_review_id: source.review_id.clone(),
        stage_126_review_sha256: source.review_sha256.clone(),
        stage_126_independent_audit_sha256: source.independent_audit.audit_sha256.clone(),
        stage_125_registration_id: source.registration.registration_id.clone(),
        stage_125_registration_sha256: source.registration.registration_sha256.clone(),
        stage_125_specification_sha256: specification.specification_sha256.clone(),
        exact_source_artifact_contract: specification.source_contract.clone(),
        exact_stage_125_specification: specification,
        future_transport_scope:
            "server_authoritative_admin_authenticated_stream_only_no_remote_fetch".to_string(),
        future_maximum_artifact_bytes: FUTURE_MAX_ARTIFACT_BYTES,
        future_maximum_receipt_bytes: FUTURE_MAX_RECEIPT_BYTES,
        future_maximum_artifact_count: FUTURE_MAX_ARTIFACT_COUNT,
        validate_declared_metadata_before_byte_acceptance_function_id:
            "opening_portfolio_receipt_validate_declared_metadata_v1".to_string(),
        stream_private_quarantine_while_hashing_and_counting_function_id:
            "opening_portfolio_receipt_stream_private_quarantine_sha256_length_v1".to_string(),
        validate_format_magic_and_safe_structure_without_financial_parsing_function_id:
            "opening_portfolio_receipt_validate_format_magic_safe_structure_v1".to_string(),
        reject_archive_active_content_password_and_unsafe_path_function_id:
            "opening_portfolio_receipt_reject_archive_active_password_unsafe_path_v1".to_string(),
        pseudonymize_account_identity_and_redact_secrets_function_id:
            "opening_portfolio_receipt_pseudonymize_identity_redact_secrets_v1".to_string(),
        atomic_content_addressed_create_new_commit_function_id:
            "opening_portfolio_receipt_atomic_content_addressed_create_new_v1".to_string(),
        append_only_redacted_receipt_manifest_function_id:
            "opening_portfolio_receipt_append_only_redacted_manifest_v1".to_string(),
        cleanup_partial_quarantine_on_failure_function_id:
            "opening_portfolio_receipt_cleanup_partial_quarantine_v1".to_string(),
        future_private_quarantine_relative_path_template:
            "opening-portfolio-source-artifact-quarantine/{stage_126_review_id}/{receipt_nonce}.part"
                .to_string(),
        future_content_addressed_artifact_relative_path_template:
            "opening-portfolio-source-artifacts/{stage_125_registration_id}/{artifact_sha256}/original.bin"
                .to_string(),
        future_receipt_manifest_schema: "receipt_id,stage_126_review_id,stage_125_registration_id,portfolio_scope_alias,artifact_sha256,byte_length,declared_format,validated_format,provider_artifact_identifier_sha256,provider_generated_at_or_statement_as_of,hone_received_at,source_account_aliases,storage_object_sha256,receipt_manifest_sha256,untrusted"
            .to_string(),
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
    validate_contract(&contract)?;
    Ok(contract)
}

fn validate_contract(
    value: &OpeningPortfolioSourceArtifactReceiptImplementationContract,
) -> Result<(), String> {
    let source = &value.exact_source_artifact_contract;
    let valid = value.schema_version == CONTRACT_SCHEMA_VERSION
        && value.protocol_version == PROTOCOL_VERSION
        && valid_sha256(&value.contract_sha256)
        && value.contract_sha256 == fingerprint_without(value, &["contract_sha256"])?
        && valid_id(&value.stage_126_review_id)
        && [
            &value.stage_126_review_sha256,
            &value.stage_126_independent_audit_sha256,
            &value.stage_125_registration_sha256,
            &value.stage_125_specification_sha256,
        ]
        .into_iter()
        .all(|item| valid_sha256(item))
        && valid_id(&value.stage_125_registration_id)
        && value.stage_125_specification_sha256
            == value.exact_stage_125_specification.specification_sha256
        && source == &value.exact_stage_125_specification.source_contract
        && source.original_bytes_required
        && source.content_sha256_and_byte_length_required
        && source.source_account_identifiers_must_be_pseudonymized
        && !source.raw_account_numbers_or_credentials_allowed
        && !source.manual_balance_or_position_entry_allowed
        && !source.mutable_or_overwritable_artifact_allowed
        && source.accepted_artifact_formats == accepted_artifact_formats()
        && value.future_transport_scope
            == "server_authoritative_admin_authenticated_stream_only_no_remote_fetch"
        && value.future_maximum_artifact_bytes == FUTURE_MAX_ARTIFACT_BYTES
        && value.future_maximum_receipt_bytes == FUTURE_MAX_RECEIPT_BYTES
        && value.future_maximum_artifact_count == FUTURE_MAX_ARTIFACT_COUNT
        && !value
            .validate_declared_metadata_before_byte_acceptance_function_id
            .is_empty()
        && !value
            .stream_private_quarantine_while_hashing_and_counting_function_id
            .is_empty()
        && !value
            .validate_format_magic_and_safe_structure_without_financial_parsing_function_id
            .is_empty()
        && !value
            .reject_archive_active_content_password_and_unsafe_path_function_id
            .is_empty()
        && !value
            .pseudonymize_account_identity_and_redact_secrets_function_id
            .is_empty()
        && !value
            .atomic_content_addressed_create_new_commit_function_id
            .is_empty()
        && !value
            .append_only_redacted_receipt_manifest_function_id
            .is_empty()
        && !value
            .cleanup_partial_quarantine_on_failure_function_id
            .is_empty()
        && value.original_bytes_preserved_immutable
        && value.encryption_at_rest_required
        && value.server_owned_received_at_required
        && !value.raw_account_numbers_or_credentials_in_paths_metadata_or_logs_allowed
        && !value.overwrite_or_mutable_artifact_allowed
        && !value.financial_row_parsing_allowed_in_receipt_stage
        && value.future_receipt_output_untrusted
        && value.future_independent_receipt_validation_required
        && value.future_snapshot_materialization_separate
        && value.future_snapshot_output_validation_separate
        && value.future_snapshot_admission_review_separate
        && value.registered_not_run
        && value.independent_implementation_review_required
        && value.future_isolated_receiver_registration_required_after_review
        && value.authority_boundary == closed_authority_boundary();
    valid
        .then_some(())
        .ok_or_else(|| "Stage 127 来源工件接收实现合同偏离固定零能力边界".to_string())
}

fn validate_record(
    value: &OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
) -> Result<(), String> {
    validate_contract(&value.implementation_contract)?;
    let review = &value.upstream_stage_126_review;
    let downstream_closed = !value.independent_implementation_review_completed
        && !value.source_artifact_receipt_eligible
        && !value.source_artifact_present
        && !value.source_artifact_uploaded_or_read
        && !value.opening_portfolio_snapshot_materialized
        && !value.opening_portfolio_snapshot_admitted
        && !value.financial_event_allowlist_nonempty
        && !value.ledger_created
        && !value.position_or_cash_written
        && !value.nav_or_performance_written
        && !value.model_or_metric_store_written
        && !value.training_or_rl_feedback_authorized
        && !value.reward_authorized
        && !value.order_generation_authorized
        && !value.broker_access_authorized
        && !value.trading_authorized;
    let valid = value.schema_version == RECORD_SCHEMA_VERSION
        && value.policy_version == POLICY_VERSION
        && valid_id(&value.implementation_id)
        && valid_sha256(&value.implementation_sha256)
        && value.implementation_id == value.implementation_sha256[..32]
        && value.implementation_sha256
            == fingerprint_without(value, &["implementation_id", "implementation_sha256"])?
        && review.specification_independently_approved
        && review.confirmations_complete
        && review
            .future_zero_capability_source_artifact_receipt_implementation_registration_eligible
        && value.implementation_contract.stage_126_review_id == review.review_id
        && value.implementation_contract.stage_126_review_sha256 == review.review_sha256
        && value
            .implementation_contract
            .stage_126_independent_audit_sha256
            == review.independent_audit.audit_sha256
        && value.implementation_contract.stage_125_registration_id
            == review.registration.registration_id
        && value.implementation_contract.stage_125_registration_sha256
            == review.registration.registration_sha256
        && value.implementation_contract.stage_125_specification_sha256
            == review.registration.specification.specification_sha256
        && value.registrar_independent_from_stage_126_reviewer_and_complete_prior_chain
        && !value.registered_by.trim().is_empty()
        && sorted_unique(&value.excluded_prior_actor_ids)
        && value.excluded_prior_actor_ids == excluded_actor_ids(review)
        && !value
            .excluded_prior_actor_ids
            .contains(&value.registered_by)
        && value.excluded_prior_actor_ids.contains(&review.reviewer_id)
        && !value.implementation_name.trim().is_empty()
        && !value.implementation_description.trim().is_empty()
        && !value
            .transport_and_authentication_semantics
            .trim()
            .is_empty()
        && !value
            .streaming_hash_length_and_atomic_commit_semantics
            .trim()
            .is_empty()
        && !value
            .format_magic_and_active_content_rejection_semantics
            .trim()
            .is_empty()
        && !value
            .pseudonymization_and_secret_redaction_semantics
            .trim()
            .is_empty()
        && !value
            .quarantine_cleanup_and_idempotency_semantics
            .trim()
            .is_empty()
        && !value.audit_and_retention_semantics.trim().is_empty()
        && !value.known_limitations.trim().is_empty()
        && !value.future_review_constraints.trim().is_empty()
        && value.status == STATUS
        && value.confirmations_complete == confirmation_values_complete(&value.confirmations)
        && value.confirmations_complete
        && value.zero_capability_implementation_contract_registered
        && value.future_stage_128_independent_implementation_review_eligible
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 127 来源工件接收实现登记无效、漂移或越权".to_string())
}

fn record_matches_source(
    record: &OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
    source: &OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord,
) -> bool {
    validate_record(record).is_ok() && &record.upstream_stage_126_review == source
}

fn validate_expected_binding(
    source: &OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord,
    request: &RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest,
) -> Result<(), String> {
    let valid = request.expected_stage_126_review_id == source.review_id
        && request.expected_stage_126_review_sha256 == source.review_sha256
        && request.expected_stage_126_independent_audit_sha256
            == source.independent_audit.audit_sha256
        && request.expected_stage_125_registration_id == source.registration.registration_id
        && request.expected_stage_125_registration_sha256
            == source.registration.registration_sha256
        && request.expected_stage_125_specification_sha256
            == source.registration.specification.specification_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 127 请求与当前 Stage 125/126 精确绑定不一致".to_string())
}

fn validate_request(
    stage_126_review_id: &str,
    request: &RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest,
) -> Result<(), String> {
    let valid = valid_id(stage_126_review_id)
        && request.expected_stage_126_review_id == stage_126_review_id
        && valid_id(&request.expected_stage_126_review_id)
        && valid_id(&request.expected_stage_125_registration_id)
        && [
            &request.expected_stage_126_review_sha256,
            &request.expected_stage_126_independent_audit_sha256,
            &request.expected_stage_125_registration_sha256,
            &request.expected_stage_125_specification_sha256,
        ]
        .into_iter()
        .all(|item| valid_sha256(item));
    valid
        .then_some(())
        .ok_or_else(|| "Stage 127 请求 ID 或摘要无效".to_string())
}

fn confirmations_complete(
    value: &RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest,
) -> bool {
    confirmation_values_complete(&confirmations_from_request(value))
}

fn confirmations_from_request(
    value: &RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest,
) -> OpeningPortfolioSourceArtifactReceiptImplementationConfirmations {
    OpeningPortfolioSourceArtifactReceiptImplementationConfirmations {
        exact_current_stage_51_through_stage_126_binding_confirmed: value
            .exact_current_stage_51_through_stage_126_binding_confirmed,
        registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed: value
            .registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed,
        review_registration_specification_and_audit_hashes_recomputed_confirmed: value
            .review_registration_specification_and_audit_hashes_recomputed_confirmed,
        exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed: value
            .exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed,
        original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed: value
            .original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed,
        content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed: value
            .content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed,
        archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed: value
            .archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed,
        source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed: value
            .source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed,
        private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed: value
            .private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed,
        server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed: value
            .server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed,
        duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: value
            .duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed,
        receipt_output_untrusted_and_independent_receipt_validation_required_confirmed: value
            .receipt_output_untrusted_and_independent_receipt_validation_required_confirmed,
        receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed: value
            .receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed,
        contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed: value
            .contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed,
        no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: value
            .no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed,
        future_stage_128_independent_implementation_review_required_confirmed: value
            .future_stage_128_independent_implementation_review_required_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: value
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
    }
}

fn confirmation_values_complete(
    value: &OpeningPortfolioSourceArtifactReceiptImplementationConfirmations,
) -> bool {
    value.exact_current_stage_51_through_stage_126_binding_confirmed
        && value.registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed
        && value.review_registration_specification_and_audit_hashes_recomputed_confirmed
        && value.exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed
        && value.original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed
        && value.content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed
        && value.archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed
        && value.source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed
        && value.private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed
        && value.server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed
        && value.duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed
        && value.receipt_output_untrusted_and_independent_receipt_validation_required_confirmed
        && value.receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed
        && value.contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed
        && value.no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && value.future_stage_128_independent_implementation_review_required_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn excluded_actor_ids(
    source: &OpeningPortfolioSnapshotGovernanceSpecificationReviewRecord,
) -> Vec<String> {
    let mut values = source.excluded_prior_actor_ids.clone();
    values.push(source.registration.registered_by.clone());
    values.push(source.reviewer_id.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn accepted_artifact_formats() -> Vec<String> {
    vec![
        "original_provider_pdf_statement".to_string(),
        "original_provider_csv_export".to_string(),
        "original_provider_json_export".to_string(),
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

fn bounded_required(value: &str, maximum: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > maximum {
        Err(format!("Stage 127 {label}不能为空且不得超过 {maximum} 字"))
    } else {
        Ok(value.to_string())
    }
}

fn bounded_text(value: &str, label: &str) -> Result<String, String> {
    bounded_required(value, MAX_TEXT_CHARS, label)
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn fingerprint_without<T: Serialize>(value: &T, excluded: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Stage 127 指纹对象不是 JSON object".to_string())?;
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

async fn read_records(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptImplementationRegistration>, String> {
    let mut entries = match tokio::fs::read_dir(record_dir(state)).await {
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
            return Err("Stage 127 实现登记目录含非 JSON 文件".to_string());
        }
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() == 0
            || metadata.len() > MAX_RECORD_FILE_BYTES
        {
            return Err("Stage 127 实现登记文件无效或超限".to_string());
        }
        let record: OpeningPortfolioSourceArtifactReceiptImplementationRegistration =
            serde_json::from_slice(
                &tokio::fs::read(&path)
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_record(&record)?;
        if path.file_stem().and_then(|value| value.to_str())
            != Some(record.implementation_id.as_str())
        {
            return Err("Stage 127 实现登记文件路径与 ID 不一致".to_string());
        }
        values.push(record);
    }
    values.sort_by(|left, right| right.registered_at.cmp(&left.registered_at));
    let mut review_ids = BTreeSet::new();
    let mut implementation_ids = BTreeSet::new();
    for value in &values {
        if !review_ids.insert(value.upstream_stage_126_review.review_id.clone())
            || !implementation_ids.insert(value.implementation_id.clone())
        {
            return Err("Stage 127 实现登记出现重复或分叉".to_string());
        }
    }
    Ok(values)
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_RECORD_FILE_BYTES {
        return Err("Stage 127 实现登记文件为空或超限".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "Stage 127 实现登记路径缺少父目录".to_string())?;
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
                "Stage 127 实现登记已存在，禁止覆盖".to_string()
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

fn record_dir(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-source-artifact-receipt-implementations")
}

struct RegistrationLock(PathBuf);

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(
    state: &AppState,
    stage_126_review_id: &str,
) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage127-{stage_126_review_id}.lock"));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 127 实现登记正在进行".to_string())?;
    Ok(RegistrationLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest {
        RegisterOpeningPortfolioSourceArtifactReceiptImplementationRequest {
            expected_stage_126_review_id: "a".repeat(32),
            expected_stage_126_review_sha256: "b".repeat(64),
            expected_stage_126_independent_audit_sha256: "c".repeat(64),
            expected_stage_125_registration_id: "d".repeat(32),
            expected_stage_125_registration_sha256: "e".repeat(64),
            expected_stage_125_specification_sha256: "f".repeat(64),
            implementation_name: "private source receipt contract".to_string(),
            immutable_code_revision: "revision-127".to_string(),
            implementation_description: "只登记不可执行合同".to_string(),
            transport_and_authentication_semantics: "管理员鉴权流式传输，不远程抓取".to_string(),
            streaming_hash_length_and_atomic_commit_semantics:
                "边接收边计算摘要与长度，通过后原子 create-new".to_string(),
            format_magic_and_active_content_rejection_semantics:
                "格式与魔数一致，拒绝主动内容".to_string(),
            pseudonymization_and_secret_redaction_semantics:
                "账号先匿名化，凭据不持久化不记录日志".to_string(),
            quarantine_cleanup_and_idempotency_semantics:
                "私有隔离区失败清理，相同内容幂等且不可覆盖".to_string(),
            audit_and_retention_semantics: "只保留脱敏 append-only receipt manifest".to_string(),
            known_limitations: "尚无上传端点、工件、解析或快照".to_string(),
            future_review_constraints: "必须通过 Stage 128 独立实现复核".to_string(),
            exact_current_stage_51_through_stage_126_binding_confirmed: true,
            registrar_independent_from_stage_126_reviewer_and_complete_prior_chain_confirmed: true,
            review_registration_specification_and_audit_hashes_recomputed_confirmed: true,
            exact_stage_125_source_contract_and_accepted_formats_preserved_confirmed: true,
            original_bytes_streamed_once_with_sha256_and_length_before_atomic_commit_confirmed: true,
            content_type_magic_utf8_structure_and_provider_metadata_checked_without_financial_parsing_confirmed: true,
            archives_active_content_password_protection_symlinks_and_path_traversal_rejected_confirmed: true,
            source_account_identifiers_pseudonymized_and_raw_accounts_credentials_never_persisted_or_logged_confirmed: true,
            private_quarantine_encryption_at_rest_create_new_and_failure_cleanup_required_confirmed: true,
            server_owned_received_time_provider_identity_and_content_addressed_manifest_required_confirmed: true,
            duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: true,
            receipt_output_untrusted_and_independent_receipt_validation_required_confirmed: true,
            receipt_snapshot_materialization_output_validation_and_snapshot_admission_remain_separate_confirmed: true,
            contract_only_no_upload_endpoint_artifact_entrypoint_runtime_network_secret_or_parser_confirmed: true,
            no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
            future_stage_128_independent_implementation_review_required_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn registration_requires_every_receipt_and_zero_capability_confirmation() {
        let mut value = request();
        assert!(confirmations_complete(&value));
        value.receipt_output_untrusted_and_independent_receipt_validation_required_confirmed =
            false;
        assert!(!confirmations_complete(&value));
    }

    #[test]
    fn request_binding_is_exact_and_hash_shaped() {
        let value = request();
        assert!(validate_request(&"a".repeat(32), &value).is_ok());
        assert!(validate_request(&"0".repeat(32), &value).is_err());
    }

    #[test]
    fn only_original_provider_artifact_formats_are_allowed() {
        assert_eq!(
            accepted_artifact_formats(),
            vec![
                "original_provider_pdf_statement",
                "original_provider_csv_export",
                "original_provider_json_export"
            ]
        );
    }

    #[test]
    fn future_receipt_resource_ceilings_are_bounded() {
        assert_eq!(FUTURE_MAX_ARTIFACT_BYTES, 64 * 1024 * 1024);
        assert_eq!(FUTURE_MAX_RECEIPT_BYTES, 256 * 1024 * 1024);
        assert_eq!(FUTURE_MAX_ARTIFACT_COUNT, 64);
        assert!(FUTURE_MAX_ARTIFACT_BYTES <= FUTURE_MAX_RECEIPT_BYTES);
    }

    #[test]
    fn current_authority_boundary_has_no_upload_or_financial_capability() {
        let value = closed_authority_boundary();
        assert!(!value.upload_endpoint_present);
        assert!(!value.source_artifact_present);
        assert!(!value.source_artifact_uploaded_or_read);
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
}
