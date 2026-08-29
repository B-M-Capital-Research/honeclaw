//! Stage 129 create-once registration of an isolated source-artifact receiver specification.
//!
//! This freezes one independently approved Stage 128 implementation, a proposed future artifact
//! identity, and a zero-capability runtime boundary. It does not expose an upload endpoint, create
//! or run an artifact, receive/read/write source bytes, parse financial rows, materialize an
//! opening portfolio, or create any financial, model, order, broker, or trading state.

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

use super::controlled_shadow_opening_portfolio_source_artifact_receipt_implementation_reviews::{
    IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation,
    OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord,
    independently_approved_opening_portfolio_source_artifact_receipt_implementations_for_isolated_receiver_specification_registration,
};
use super::controlled_shadow_opening_portfolio_source_artifact_receipt_implementations::{
    OpeningPortfolioSourceArtifactReceiptImplementationContract,
    OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-isolated-receiver-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-isolated-receiver-specification-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-isolated-receiver-zero-capability-contract-v1";
const POLICY_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-isolated-receiver-create-once-v1-no-execution";
const STATUS: &str = "registered_not_authorized_not_run";
const NEXT_GATE: &str =
    "chain_external_first_source_artifact_receipt_execution_authorization_review";
const RUNTIME_IDENTITY: &str = "hone-opening-portfolio-source-artifact-receipt-runtime";
const RUNTIME_VERSION: &str = "v1-proposed-artifact-not-present-no-current-input";
const FUTURE_INPUT_ENVELOPE: &str =
    "administrator_authenticated_original_provider_pdf_csv_json_stream_v1";
const FUTURE_OUTPUT_ENVELOPE: &str =
    "opening_portfolio_source_artifact_receipt_manifest_v1_create_once_untrusted";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_RECORD_FILE_BYTES: u64 = 1024 * 1024;
const MAXIMUM_MEMORY_MIB: u32 = 1_024;
const MAXIMUM_WALL_CLOCK_SECONDS: u32 = 300;
const MAXIMUM_CPU_MILLICORES: u32 = 1_000;
const MAXIMUM_PROCESS_COUNT: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioSourceArtifactReceiptIsolatedReceiverKind {
    EphemeralDeterministicStreamOnlyReceiptSpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest {
    expected_stage_128_review_id: String,
    expected_stage_128_review_sha256: String,
    expected_stage_128_independent_audit_sha256: String,
    expected_stage_127_implementation_id: String,
    expected_stage_127_implementation_sha256: String,
    expected_stage_127_implementation_contract_sha256: String,
    expected_stage_126_review_sha256: String,
    expected_stage_126_independent_audit_sha256: String,
    expected_stage_125_registration_sha256: String,
    expected_stage_125_specification_sha256: String,
    receiver_name: String,
    receiver_kind: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverKind,
    receiver_spec_revision: String,
    proposed_receiver_code_revision: String,
    proposed_receiver_artifact_sha256: String,
    artifact_reproduction_procedure: String,
    rationale: String,
    known_limitations: String,
    future_input_constraints: String,
    future_output_constraints: String,
    exact_current_stage_51_through_stage_128_binding_confirmed: bool,
    registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed: bool,
    review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed:
        bool,
    proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed: bool,
    all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed: bool,
    exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: bool,
    future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: bool,
    future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed: bool,
    future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed:
        bool,
    future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed:
        bool,
    future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: bool,
    future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed:
        bool,
    fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed:
        bool,
    no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptIsolatedReceiverContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub stage_128_review_id: String,
    pub stage_128_review_sha256: String,
    pub stage_128_independent_audit_sha256: String,
    pub stage_127_implementation_id: String,
    pub stage_127_implementation_sha256: String,
    pub stage_127_implementation_contract_sha256: String,
    pub stage_126_review_sha256: String,
    pub stage_126_independent_audit_sha256: String,
    pub stage_125_registration_sha256: String,
    pub stage_125_specification_sha256: String,
    pub exact_approved_implementation_contract:
        OpeningPortfolioSourceArtifactReceiptImplementationContract,
    pub receiver_spec_revision: String,
    pub proposed_receiver_code_revision: String,
    pub proposed_receiver_artifact_sha256: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub future_input_envelope: String,
    pub future_output_envelope: String,
    pub next_gate: String,
    pub specification_registered: bool,
    pub future_receiver_artifact_identity_bound: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub input_read_allowed: bool,
    pub data_access_authorized: bool,
    pub administrator_authentication_required: bool,
    pub stream_only_required: bool,
    pub remote_url_fetch_allowed: bool,
    pub root_filesystem_read_only_required: bool,
    pub ephemeral_working_directory_required: bool,
    pub run_as_unprivileged_required: bool,
    pub no_new_privileges_required: bool,
    pub future_input_read_only_required: bool,
    pub future_input_content_addressed_required: bool,
    pub future_output_create_once_required: bool,
    pub future_output_untrusted_required: bool,
    pub future_output_independent_validation_required: bool,
    pub environment_inheritance_allowed: bool,
    pub allowed_environment_variables: Vec<String>,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub source_artifact_storage_writes_allowed: bool,
    pub receipt_manifest_writes_allowed: bool,
    pub financial_row_parsing_allowed: bool,
    pub opening_portfolio_snapshot_writes_allowed: bool,
    pub financial_event_allowlist_writes_allowed: bool,
    pub ledger_or_position_cash_writes_allowed: bool,
    pub nav_or_performance_writes_allowed: bool,
    pub model_or_metric_store_writes_allowed: bool,
    pub training_or_rl_feedback_allowed: bool,
    pub reward_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
    pub maximum_parallel_runs: u32,
    pub maximum_memory_mib: u32,
    pub maximum_wall_clock_seconds: u32,
    pub maximum_cpu_millicores: u32,
    pub maximum_process_count: u32,
    pub maximum_output_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub isolated_receiver_id: String,
    pub isolated_receiver_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub implementation: OpeningPortfolioSourceArtifactReceiptImplementationRegistration,
    pub implementation_review: OpeningPortfolioSourceArtifactReceiptImplementationReviewRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub receiver_name: String,
    pub receiver_kind: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverKind,
    pub artifact_reproduction_procedure: String,
    pub rationale: String,
    pub known_limitations: String,
    pub future_input_constraints: String,
    pub future_output_constraints: String,
    pub receiver_contract: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverContract,
    pub status: String,
    pub confirmations_complete: bool,
    pub exact_current_stage_51_through_stage_128_binding_confirmed: bool,
    pub registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed: bool,
    pub review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed:
        bool,
    pub proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed:
        bool,
    pub all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed: bool,
    pub exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: bool,
    pub future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: bool,
    pub future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed: bool,
    pub future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed:
        bool,
    pub future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed:
        bool,
    pub future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: bool,
    pub future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed:
        bool,
    pub no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    pub no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub first_execution_authorization_review_eligible: bool,
    pub first_execution_authorized: bool,
    pub source_artifact_received_or_read: bool,
    pub receipt_manifest_created: bool,
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
pub(crate) struct OpeningPortfolioSourceArtifactReceiptIsolatedReceiverItem {
    pub receiver: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
    pub approved_review_binding_current: bool,
    pub first_execution_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_implementations:
        Vec<IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation>,
    pub registration_eligible_count: usize,
    pub isolated_receiver_count: usize,
    pub current_binding_receiver_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub items: Vec<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverItem>,
    pub receiver_status: String,
    pub upload_endpoint_present: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_accessed: bool,
    pub receipt_manifest_created: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptIsolatedReceiverReadinessSummary {
    pub registration_eligible_count: usize,
    pub isolated_receiver_count: usize,
    pub current_binding_receiver_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub receiver_status: String,
}

pub(crate) async fn handle_get_opening_portfolio_source_artifact_receipt_isolated_receivers(
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
            warn!(%error, "Stage 129 isolated receiver registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "期初组合来源工件隔离接收器规格登记暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_opening_portfolio_source_artifact_receipt_isolated_receiver(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    if implementation_id != request.expected_stage_127_implementation_id {
        return crate::routes::json_error(StatusCode::BAD_REQUEST, "实现 ID 与路径不一致");
    }
    match register_once(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn opening_portfolio_source_artifact_receipt_isolated_receiver_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        OpeningPortfolioSourceArtifactReceiptIsolatedReceiverReadinessSummary {
            registration_eligible_count: value.registration_eligible_count,
            isolated_receiver_count: value.isolated_receiver_count,
            current_binding_receiver_count: value.current_binding_receiver_count,
            first_execution_authorization_review_eligible_count: value
                .first_execution_authorization_review_eligible_count,
            receiver_status: value.receiver_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn isolated_receivers_for_first_execution_authorization_review(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord>, String> {
    let approved = independently_approved_opening_portfolio_source_artifact_receipt_implementations_for_isolated_receiver_specification_registration(state).await?;
    let mut values = read_receivers(state)
        .await?
        .into_iter()
        .filter(|receiver| {
            approved
                .iter()
                .any(|value| binding_matches(receiver, value))
                && receiver.first_execution_authorization_review_eligible
        })
        .collect::<Vec<_>>();
    values.sort_by(|left, right| right.registered_at.cmp(&left.registered_at));
    Ok(values)
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry, String> {
    let approved = independently_approved_opening_portfolio_source_artifact_receipt_implementations_for_isolated_receiver_specification_registration(state).await?;
    let mut items = read_receivers(state)
        .await?
        .into_iter()
        .map(|receiver| {
            let current = approved
                .iter()
                .any(|value| binding_matches(&receiver, value));
            OpeningPortfolioSourceArtifactReceiptIsolatedReceiverItem {
                first_execution_authorization_review_eligible: current
                    && receiver.first_execution_authorization_review_eligible,
                approved_review_binding_current: current,
                receiver,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .receiver
            .registered_at
            .cmp(&left.receiver.registered_at)
    });
    let registered_review_ids = items
        .iter()
        .map(|item| item.receiver.implementation_review.review_id.as_str())
        .collect::<BTreeSet<_>>();
    let eligible_implementations = approved
        .iter()
        .filter(|value| !registered_review_ids.contains(value.review.review_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let registration_eligible_count = eligible_implementations.len();
    let current_binding_receiver_count = items
        .iter()
        .filter(|item| item.approved_review_binding_current)
        .count();
    let first_execution_authorization_review_eligible_count = items
        .iter()
        .filter(|item| item.first_execution_authorization_review_eligible)
        .count();
    let receiver_status = if approved.is_empty() {
        "waiting_stage_128_independently_approved_source_artifact_receipt_implementation"
    } else if first_execution_authorization_review_eligible_count > 0 {
        "isolated_source_artifact_receiver_registered_not_authorized_not_run"
    } else if registration_eligible_count > 0 {
        "ready_for_isolated_source_artifact_receiver_specification_registration"
    } else {
        "isolated_source_artifact_receiver_binding_stale_fail_closed"
    };
    Ok(OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        eligible_implementations, registration_eligible_count, isolated_receiver_count: items.len(),
        current_binding_receiver_count, first_execution_authorization_review_eligible_count, items,
        receiver_status: receiver_status.to_string(), upload_endpoint_present: false,
        source_artifact_present: false, executable_artifact_present: false, callable_entrypoint_present: false,
        runtime_instantiated: false, input_accessed: false, receipt_manifest_created: false,
        opening_portfolio_snapshot_admitted: false, ledger_created: false, position_or_cash_written: false,
        nav_or_performance_written: false, order_generation_authorized: false, broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 129 只把 Stage 128 独立批准绑定到未来隔离接收器工件身份、不可变代码版本、固定非特权 runtime、管理员鉴权流式原始 PDF/CSV/JSON 输入和 create-once 未受信 receipt manifest。当前没有上传入口、来源字节、可执行工件、入口、runtime、挂载、输入读取或存储写入；登记后下一步只能进入 Stage 130 责任链外首次执行授权复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest,
) -> Result<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord, String> {
    let _lock = acquire_lock(state, &request.expected_stage_127_implementation_id).await?;
    let approved = independently_approved_opening_portfolio_source_artifact_receipt_implementations_for_isolated_receiver_specification_registration(state).await?;
    let value = approved
        .iter()
        .find(|value| {
            value.implementation.implementation_id == request.expected_stage_127_implementation_id
        })
        .ok_or_else(|| {
            "当前没有这条绑定有效且已独立批准的 Stage 127 来源工件接收实现".to_string()
        })?;
    validate_expected_binding(value, &request)?;
    if read_receivers(state)
        .await?
        .iter()
        .any(|receiver| receiver.implementation_review.review_id == value.review.review_id)
    {
        return Err("这条 Stage 128 批准已经登记过隔离接收器规格".to_string());
    }
    let record = new_record(value, registrar_id, &request, Utc::now())?;
    write_immutable_json(
        &receiver_root(state).join(format!("{}.json", record.isolated_receiver_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn validate_expected_binding(
    approved: &IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation,
    request: &RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest,
) -> Result<(), String> {
    let implementation = &approved.implementation;
    let review = &approved.review;
    let upstream = &implementation.upstream_stage_126_review;
    if request.expected_stage_128_review_id != review.review_id
        || request.expected_stage_128_review_sha256 != review.review_sha256
        || request.expected_stage_128_independent_audit_sha256
            != review.independent_audit.audit_sha256
        || request.expected_stage_127_implementation_id != implementation.implementation_id
        || request.expected_stage_127_implementation_sha256 != implementation.implementation_sha256
        || request.expected_stage_127_implementation_contract_sha256
            != implementation.implementation_contract.contract_sha256
        || request.expected_stage_126_review_sha256 != upstream.review_sha256
        || request.expected_stage_126_independent_audit_sha256
            != upstream.independent_audit.audit_sha256
        || request.expected_stage_125_registration_sha256
            != upstream.registration.registration_sha256
        || request.expected_stage_125_specification_sha256
            != upstream.registration.specification.specification_sha256
    {
        return Err("Stage 125–128 来源工件接收规格、实现、审计或复核绑定已经变化".to_string());
    }
    Ok(())
}

fn new_record(
    approved: &IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation,
    registrar_id: &str,
    request: &RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest,
    registered_at: DateTime<Utc>,
) -> Result<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord, String> {
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(approved);
    if registrar_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|value| value == registrar_id)
    {
        return Err(
            "隔离接收器规格登记人必须独立于 Stage 128 和完整 Stage 51–128 责任链".to_string(),
        );
    }
    if !request_checks(request) {
        return Err("必须逐项确认 Stage 129 隔离接收器规格与零能力边界".to_string());
    }
    if !valid_sha256(&request.proposed_receiver_artifact_sha256) {
        return Err("未来隔离接收器工件 SHA-256 无效".to_string());
    }
    let receiver_name = bounded_required(&request.receiver_name, MAX_NAME_CHARS, "接收器名称")?;
    let receiver_spec_revision = bounded_required(
        &request.receiver_spec_revision,
        MAX_REVISION_CHARS,
        "接收器规格版本",
    )?;
    let proposed_receiver_code_revision = bounded_required(
        &request.proposed_receiver_code_revision,
        MAX_REVISION_CHARS,
        "未来接收器代码版本",
    )?;
    let mut record = OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        isolated_receiver_id: String::new(), isolated_receiver_spec_sha256: String::new(), registered_at,
        registered_by: registrar_id.to_string(), implementation: approved.implementation.clone(), implementation_review: approved.review.clone(),
        excluded_prior_actor_ids, receiver_name, receiver_kind: request.receiver_kind.clone(),
        artifact_reproduction_procedure: bounded_required(&request.artifact_reproduction_procedure, MAX_TEXT_CHARS, "工件复现步骤")?,
        rationale: bounded_required(&request.rationale, MAX_TEXT_CHARS, "登记理由")?,
        known_limitations: bounded_required(&request.known_limitations, MAX_TEXT_CHARS, "已知局限")?,
        future_input_constraints: bounded_required(&request.future_input_constraints, MAX_TEXT_CHARS, "未来输入约束")?,
        future_output_constraints: bounded_required(&request.future_output_constraints, MAX_TEXT_CHARS, "未来输出约束")?,
        receiver_contract: receiver_contract(approved, &receiver_spec_revision, &proposed_receiver_code_revision, &request.proposed_receiver_artifact_sha256)?,
        status: STATUS.to_string(), confirmations_complete: true,
        exact_current_stage_51_through_stage_128_binding_confirmed: true,
        registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed: true,
        review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed: true,
        proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed: true,
        all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed: true,
        exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: true,
        future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: true,
        future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed: true,
        future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed: true,
        future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: true,
        future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: true,
        future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed: true,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
        no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
        no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
        registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed: true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        first_execution_authorization_review_eligible: true, first_execution_authorized: false,
        source_artifact_received_or_read: false, receipt_manifest_created: false, opening_portfolio_snapshot_materialized: false,
        opening_portfolio_snapshot_admitted: false, financial_event_allowlist_nonempty: false, ledger_created: false,
        position_or_cash_written: false, nav_or_performance_written: false, model_or_metric_store_written: false,
        training_or_rl_feedback_authorized: false, reward_authorized: false, order_generation_authorized: false,
        broker_access_authorized: false, trading_authorized: false,
    };
    record.isolated_receiver_spec_sha256 = receiver_fingerprint(&record)?;
    record.isolated_receiver_id = record.isolated_receiver_spec_sha256[..32].to_string();
    validate_isolated_receiver_for_review(&record)?;
    Ok(record)
}

fn receiver_contract(
    approved: &IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation,
    receiver_spec_revision: &str,
    proposed_receiver_code_revision: &str,
    proposed_receiver_artifact_sha256: &str,
) -> Result<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverContract, String> {
    let implementation = &approved.implementation;
    let source = &implementation.implementation_contract;
    let upstream = &implementation.upstream_stage_126_review;
    let mut contract = OpeningPortfolioSourceArtifactReceiptIsolatedReceiverContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        stage_128_review_id: approved.review.review_id.clone(),
        stage_128_review_sha256: approved.review.review_sha256.clone(),
        stage_128_independent_audit_sha256: approved.review.independent_audit.audit_sha256.clone(),
        stage_127_implementation_id: implementation.implementation_id.clone(),
        stage_127_implementation_sha256: implementation.implementation_sha256.clone(),
        stage_127_implementation_contract_sha256: source.contract_sha256.clone(),
        stage_126_review_sha256: upstream.review_sha256.clone(),
        stage_126_independent_audit_sha256: upstream.independent_audit.audit_sha256.clone(),
        stage_125_registration_sha256: upstream.registration.registration_sha256.clone(),
        stage_125_specification_sha256: upstream
            .registration
            .specification
            .specification_sha256
            .clone(),
        exact_approved_implementation_contract: source.clone(),
        receiver_spec_revision: receiver_spec_revision.to_string(),
        proposed_receiver_code_revision: proposed_receiver_code_revision.to_string(),
        proposed_receiver_artifact_sha256: proposed_receiver_artifact_sha256.to_ascii_lowercase(),
        runtime_identity: RUNTIME_IDENTITY.to_string(),
        runtime_version: RUNTIME_VERSION.to_string(),
        future_input_envelope: FUTURE_INPUT_ENVELOPE.to_string(),
        future_output_envelope: FUTURE_OUTPUT_ENVELOPE.to_string(),
        next_gate: NEXT_GATE.to_string(),
        specification_registered: true,
        future_receiver_artifact_identity_bound: true,
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        input_mount_present: false,
        input_read_allowed: false,
        data_access_authorized: false,
        administrator_authentication_required: true,
        stream_only_required: true,
        remote_url_fetch_allowed: false,
        root_filesystem_read_only_required: true,
        ephemeral_working_directory_required: true,
        run_as_unprivileged_required: true,
        no_new_privileges_required: true,
        future_input_read_only_required: true,
        future_input_content_addressed_required: true,
        future_output_create_once_required: true,
        future_output_untrusted_required: true,
        future_output_independent_validation_required: true,
        environment_inheritance_allowed: false,
        allowed_environment_variables: Vec::new(),
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        source_artifact_storage_writes_allowed: false,
        receipt_manifest_writes_allowed: false,
        financial_row_parsing_allowed: false,
        opening_portfolio_snapshot_writes_allowed: false,
        financial_event_allowlist_writes_allowed: false,
        ledger_or_position_cash_writes_allowed: false,
        nav_or_performance_writes_allowed: false,
        model_or_metric_store_writes_allowed: false,
        training_or_rl_feedback_allowed: false,
        reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
        maximum_parallel_runs: 1,
        maximum_memory_mib: MAXIMUM_MEMORY_MIB,
        maximum_wall_clock_seconds: MAXIMUM_WALL_CLOCK_SECONDS,
        maximum_cpu_millicores: MAXIMUM_CPU_MILLICORES,
        maximum_process_count: MAXIMUM_PROCESS_COUNT,
        maximum_output_bytes: source.future_maximum_receipt_bytes,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn request_checks(
    value: &RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest,
) -> bool {
    value.exact_current_stage_51_through_stage_128_binding_confirmed
        && value.registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed
        && value.review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed
        && value.proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed
        && value.all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed
        && value.exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed
        && value.future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed
        && value.future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed
        && value.future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed
        && value.future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed
        && value.future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed
        && value.future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed
        && value.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed
        && value.no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && value.registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn record_checks(value: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord) -> bool {
    value.exact_current_stage_51_through_stage_128_binding_confirmed
        && value.registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed
        && value.review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed
        && value.proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed
        && value.all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed
        && value.exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed
        && value.future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed
        && value.future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed
        && value.future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed
        && value.future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed
        && value.future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed
        && value.future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed
        && value.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed
        && value.no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && value.registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

pub(crate) fn validate_isolated_receiver_for_review(
    record: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> Result<(), String> {
    let contract = &record.receiver_contract;
    let source = &record.implementation.implementation_contract;
    let record_closed = !record.first_execution_authorized
        && !record.source_artifact_received_or_read
        && !record.receipt_manifest_created
        && !record.opening_portfolio_snapshot_materialized
        && !record.opening_portfolio_snapshot_admitted
        && !record.financial_event_allowlist_nonempty
        && !record.ledger_created
        && !record.position_or_cash_written
        && !record.nav_or_performance_written
        && !record.model_or_metric_store_written
        && !record.training_or_rl_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let contract_closed = !contract.source_artifact_present
        && !contract.executable_artifact_present
        && !contract.callable_entrypoint_present
        && !contract.runtime_instantiated
        && !contract.input_mount_present
        && !contract.input_read_allowed
        && !contract.data_access_authorized
        && !contract.remote_url_fetch_allowed
        && !contract.environment_inheritance_allowed
        && contract.allowed_environment_variables.is_empty()
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.source_artifact_storage_writes_allowed
        && !contract.receipt_manifest_writes_allowed
        && !contract.financial_row_parsing_allowed
        && !contract.opening_portfolio_snapshot_writes_allowed
        && !contract.financial_event_allowlist_writes_allowed
        && !contract.ledger_or_position_cash_writes_allowed
        && !contract.nav_or_performance_writes_allowed
        && !contract.model_or_metric_store_writes_allowed
        && !contract.training_or_rl_feedback_allowed
        && !contract.reward_allowed
        && !contract.order_generation_allowed
        && !contract.broker_access_allowed
        && !contract.trading_allowed;
    let valid = record.schema_version == RECORD_SCHEMA_VERSION
        && record.policy_version == POLICY_VERSION
        && record.status == STATUS
        && valid_id(&record.isolated_receiver_id)
        && valid_sha256(&record.isolated_receiver_spec_sha256)
        && record.isolated_receiver_id == record.isolated_receiver_spec_sha256[..32]
        && record.isolated_receiver_spec_sha256 == receiver_fingerprint(record)?
        && record.excluded_prior_actor_ids
            == expected_excluded_prior_actor_ids(
                &IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation {
                    implementation: record.implementation.clone(),
                    review: record.implementation_review.clone(),
                },
            )
        && !record
            .excluded_prior_actor_ids
            .contains(&record.registered_by)
        && sorted_unique(&record.excluded_prior_actor_ids)
        && record.confirmations_complete
        && record_checks(record)
        && record.first_execution_authorization_review_eligible
        && contract.schema_version == CONTRACT_SCHEMA_VERSION
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?
        && contract.stage_128_review_id == record.implementation_review.review_id
        && contract.stage_128_review_sha256 == record.implementation_review.review_sha256
        && contract.stage_128_independent_audit_sha256
            == record.implementation_review.independent_audit.audit_sha256
        && contract.stage_127_implementation_id == record.implementation.implementation_id
        && contract.stage_127_implementation_sha256 == record.implementation.implementation_sha256
        && contract.stage_127_implementation_contract_sha256 == source.contract_sha256
        && contract.exact_approved_implementation_contract == *source
        && contract.next_gate == NEXT_GATE
        && contract.specification_registered
        && contract.future_receiver_artifact_identity_bound
        && valid_sha256(&contract.proposed_receiver_artifact_sha256)
        && contract.administrator_authentication_required
        && contract.stream_only_required
        && contract.root_filesystem_read_only_required
        && contract.ephemeral_working_directory_required
        && contract.run_as_unprivileged_required
        && contract.no_new_privileges_required
        && contract.future_input_read_only_required
        && contract.future_input_content_addressed_required
        && contract.future_output_create_once_required
        && contract.future_output_untrusted_required
        && contract.future_output_independent_validation_required
        && contract.maximum_parallel_runs == 1
        && contract.maximum_memory_mib == MAXIMUM_MEMORY_MIB
        && contract.maximum_wall_clock_seconds == MAXIMUM_WALL_CLOCK_SECONDS
        && contract.maximum_cpu_millicores == MAXIMUM_CPU_MILLICORES
        && contract.maximum_process_count == MAXIMUM_PROCESS_COUNT
        && contract.maximum_output_bytes == source.future_maximum_receipt_bytes
        && source.future_maximum_artifact_bytes == 64 * 1024 * 1024
        && source.future_maximum_receipt_bytes == 256 * 1024 * 1024
        && source.future_maximum_artifact_count == 64
        && contract_closed
        && record_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 129 隔离接收器规格无效、漂移或越权".to_string())
}

fn binding_matches(
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
    approved: &IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation,
) -> bool {
    receiver.implementation == approved.implementation
        && receiver.implementation_review == approved.review
        && validate_isolated_receiver_for_review(receiver).is_ok()
}

fn expected_excluded_prior_actor_ids(
    approved: &IndependentlyApprovedOpeningPortfolioSourceArtifactReceiptImplementation,
) -> Vec<String> {
    let mut actors = approved.review.excluded_prior_actor_ids.clone();
    actors.extend(approved.implementation.excluded_prior_actor_ids.clone());
    actors.push(approved.implementation.registered_by.clone());
    actors.push(approved.review.reviewer_id.clone());
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn receiver_fingerprint(
    record: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> Result<String, String> {
    fingerprint_without(
        record,
        &["isolated_receiver_id", "isolated_receiver_spec_sha256"],
    )
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 129 指纹载荷无效".to_string())?;
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
        Err(format!("{label}不能为空且不得超过 {max_chars} 字"))
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

async fn read_receivers(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord>, String> {
    let directory = receiver_root(state);
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
        if entry
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("json")
        {
            let bytes = tokio::fs::read(&path)
                .await
                .map_err(|error| error.to_string())?;
            if bytes.len() as u64 > MAX_RECORD_FILE_BYTES {
                return Err("Stage 129 记录文件超过上限".to_string());
            }
            let receiver: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord =
                serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
            if path.file_stem().and_then(|value| value.to_str())
                != Some(receiver.isolated_receiver_id.as_str())
            {
                return Err("Stage 129 文件名与记录 ID 不一致".to_string());
            }
            validate_isolated_receiver_for_review(&receiver)?;
            values.push(receiver);
        }
    }
    values.sort_by(|left, right| left.registered_at.cmp(&right.registered_at));
    let mut ids = BTreeSet::new();
    let mut reviews = BTreeSet::new();
    for receiver in &values {
        if !ids.insert(receiver.isolated_receiver_id.clone())
            || !reviews.insert(receiver.implementation_review.review_id.clone())
        {
            return Err("Stage 129 隔离接收器规格重复或分叉".to_string());
        }
    }
    Ok(values)
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > MAX_RECORD_FILE_BYTES {
        return Err("Stage 129 记录文件超过上限".to_string());
    }
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

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn receiver_root(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-source-artifact-receipt-isolated-receivers")
}

struct ReceiverLock(PathBuf);
impl Drop for ReceiverLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, implementation_id: &str) -> Result<ReceiverLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "opening-portfolio-source-artifact-receipt-isolated-receiver-{implementation_id}.lock"
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
        .map_err(|_| "已有同一 Stage 129 登记正在进行".to_string())?;
    Ok(ReceiverLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest {
        RegisterOpeningPortfolioSourceArtifactReceiptIsolatedReceiverRequest {
            expected_stage_128_review_id: "a".repeat(32), expected_stage_128_review_sha256: "b".repeat(64), expected_stage_128_independent_audit_sha256: "c".repeat(64),
            expected_stage_127_implementation_id: "d".repeat(32), expected_stage_127_implementation_sha256: "e".repeat(64), expected_stage_127_implementation_contract_sha256: "f".repeat(64),
            expected_stage_126_review_sha256: "1".repeat(64), expected_stage_126_independent_audit_sha256: "2".repeat(64), expected_stage_125_registration_sha256: "3".repeat(64), expected_stage_125_specification_sha256: "4".repeat(64),
            receiver_name: "隔离来源工件接收器".to_string(), receiver_kind: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverKind::EphemeralDeterministicStreamOnlyReceiptSpecification,
            receiver_spec_revision: "v1".to_string(), proposed_receiver_code_revision: "rev-1".to_string(), proposed_receiver_artifact_sha256: "5".repeat(64),
            artifact_reproduction_procedure: "未来独立复现后逐位核验。".to_string(), rationale: "冻结未来接收边界。".to_string(), known_limitations: "当前没有工件、入口或来源字节。".to_string(),
            future_input_constraints: "未来只接受管理员鉴权流式原始 PDF/CSV/JSON。".to_string(), future_output_constraints: "只允许 create-once 未受信 receipt manifest。".to_string(),
            exact_current_stage_51_through_stage_128_binding_confirmed: true, registrar_independent_from_stage_128_reviewer_and_complete_prior_chain_confirmed: true,
            review_audit_implementation_contract_registration_and_specification_hashes_reproduced_confirmed: true, proposed_artifact_identity_revision_and_reproduction_bound_but_artifact_absent_confirmed: true,
            all_eight_receipt_functions_and_original_pdf_csv_json_formats_preserved_confirmed: true, exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: true,
            future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: true, future_private_quarantine_streaming_sha256_length_and_atomic_create_new_confirmed: true,
            future_magic_safe_structure_active_content_archive_password_symlink_and_path_rejection_confirmed: true, future_account_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: true,
            future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: true, future_receipt_validation_snapshot_materialization_output_validation_and_admission_separate_confirmed: true,
            fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true, no_upload_source_bytes_artifact_entrypoint_runtime_input_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
            registration_only_opens_stage_130_chain_external_first_execution_authorization_review_confirmed: true, no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn requires_every_confirmation() {
        let mut value = request();
        assert!(request_checks(&value));
        value.no_unconfirmed_hari_or_old_wang_logic_claimed = false;
        assert!(!request_checks(&value));
    }
    #[test]
    fn only_sha256_artifact_identity_is_accepted() {
        assert!(valid_sha256(&"a".repeat(64)));
        assert!(!valid_sha256(&"a".repeat(63)));
    }
    #[test]
    fn runtime_is_single_process_and_bounded() {
        assert_eq!(MAXIMUM_PROCESS_COUNT, 1);
        assert_eq!(MAXIMUM_MEMORY_MIB, 1024);
        assert_eq!(MAXIMUM_WALL_CLOCK_SECONDS, 300);
    }
    #[test]
    fn next_gate_is_first_execution_review_only() {
        assert!(NEXT_GATE.contains("first_source_artifact_receipt_execution_authorization_review"));
    }
    #[test]
    fn bounded_text_fails_closed() {
        assert!(bounded_required("", 10, "x").is_err());
        assert!(bounded_required("ok", 10, "x").is_ok());
    }
}
