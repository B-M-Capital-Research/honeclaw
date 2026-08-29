//! Stage 136 create-once registration of an isolated opening-portfolio snapshot materializer
//! specification.
//!
//! The record freezes one exact Stage 135 approval, a proposed future artifact identity, immutable
//! code revision, fixed unprivileged runtime contract, read-only content-addressed input and
//! create-once untrusted output. It does not create materialization source, an executable artifact, a
//! callable entrypoint, runtime, input mount/read, decrypted receipt, parsed row, snapshot
//! candidate, financial state, training, reward, order, broker or trading capability.

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

use super::controlled_shadow_opening_portfolio_snapshot_materialization_implementation_reviews::{
    IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation,
    OpeningPortfolioSnapshotMaterializationImplementationReviewRecord,
    independently_approved_opening_portfolio_snapshot_materialization_implementations_for_isolated_materializer_specification_registration,
};
use super::controlled_shadow_opening_portfolio_snapshot_materialization_implementations::{
    OpeningPortfolioSnapshotMaterializationImplementationContract,
    OpeningPortfolioSnapshotMaterializationImplementationRegistration,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-isolated-materializer-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-isolated-materializer-specification-v1";
const CONTRACT_SCHEMA_VERSION: &str = "hone-opening-portfolio-snapshot-materialization-isolated-materializer-zero-capability-contract-v1";
const POLICY_VERSION: &str = "hone-opening-portfolio-snapshot-materialization-isolated-materializer-create-once-v1-no-execution";
const STATUS: &str = "registered_not_authorized_not_run";
const NEXT_GATE: &str = "stage_137_chain_external_first_opening_portfolio_snapshot_materialization_execution_authorization_review";
const RUNTIME_IDENTITY: &str = "hone-opening-portfolio-snapshot-materializer-v1";
const RUNTIME_VERSION: &str = "v1-proposed-artifact-not-present-no-current-mount";
const FUTURE_INPUT_ENVELOPE: &str =
    "stage_133_independently_validated_encrypted_receipt_v1_read_only_content_addressed";
const FUTURE_OUTPUT_ENVELOPE: &str =
    "opening_portfolio_canonical_snapshot_candidate_v1_create_once_untrusted";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;
const MAXIMUM_MEMORY_MIB: u32 = 1_024;
const MAXIMUM_WALL_CLOCK_SECONDS: u32 = 300;
const MAXIMUM_CPU_MILLICORES: u32 = 1_000;
const MAXIMUM_PROCESS_COUNT: u32 = 1;
const MAXIMUM_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioSnapshotMaterializationIsolatedMaterializerKind {
    EphemeralDeterministicPdfCsvJsonSnapshotMaterializationSpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest {
    expected_stage_135_review_id: String,
    expected_stage_135_review_sha256: String,
    expected_stage_135_independent_audit_sha256: String,
    expected_stage_134_implementation_id: String,
    expected_stage_134_implementation_sha256: String,
    expected_stage_134_implementation_contract_sha256: String,
    expected_stage_133_validation_sha256: String,
    expected_stage_132_result_sha256: String,
    expected_stage_131_claim_sha256: String,
    expected_receipt_manifest_sha256: String,
    expected_stage_125_specification_sha256: String,
    materializer_name: String,
    materializer_kind: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerKind,
    materializer_spec_revision: String,
    proposed_materializer_code_revision: String,
    proposed_materializer_artifact_sha256: String,
    artifact_reproduction_procedure: String,
    rationale: String,
    known_limitations: String,
    future_input_constraints: String,
    future_output_constraints: String,
    exact_current_stage_51_through_stage_135_binding_confirmed: bool,
    registrar_independent_from_stage_135_and_complete_prior_chain_confirmed: bool,
    implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: bool,
    proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed:
        bool,
    all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed: bool,
    future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed:
        bool,
    complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed:
        bool,
    exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed:
        bool,
    future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed:
        bool,
    deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed: bool,
    statement_market_values_informational_and_no_nav_or_performance_confirmed: bool,
    future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed:
        bool,
    no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationIsolatedMaterializerContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub stage_135_implementation_review_id: String,
    pub stage_135_implementation_review_sha256: String,
    pub stage_135_independent_audit_sha256: String,
    pub stage_134_implementation_id: String,
    pub stage_134_implementation_sha256: String,
    pub stage_134_implementation_contract_sha256: String,
    pub stage_133_validation_sha256: String,
    pub stage_132_result_sha256: String,
    pub stage_131_claim_sha256: String,
    pub receipt_manifest_sha256: String,
    pub stage_125_specification_sha256: String,
    pub exact_approved_implementation_contract:
        OpeningPortfolioSnapshotMaterializationImplementationContract,
    pub materializer_spec_revision: String,
    pub proposed_materializer_code_revision: String,
    pub proposed_materializer_artifact_sha256: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub future_input_envelope: String,
    pub future_output_envelope: String,
    pub next_gate: String,
    pub specification_registered: bool,
    pub future_materializer_artifact_identity_bound: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_identity_bound: bool,
    pub runtime_instantiated: bool,
    pub input_mount_present: bool,
    pub input_read_allowed: bool,
    pub data_access_authorized: bool,
    pub root_filesystem_read_only_required: bool,
    pub ephemeral_working_directory_required: bool,
    pub run_as_unprivileged_required: bool,
    pub no_new_privileges_required: bool,
    pub future_input_read_only_required: bool,
    pub future_input_content_addressed_required: bool,
    pub future_input_stage_133_independent_validation_required: bool,
    pub future_decryption_ephemeral_memory_only_required: bool,
    pub plaintext_persistence_allowed: bool,
    pub remote_fetch_allowed: bool,
    pub future_output_create_once_required: bool,
    pub future_output_untrusted_required: bool,
    pub future_output_independent_validation_required: bool,
    pub future_output_investment_semantics_allowed: bool,
    pub future_output_order_intent_allowed: bool,
    pub environment_inheritance_allowed: bool,
    pub allowed_environment_variables: Vec<String>,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub output_candidate_writes_allowed: bool,
    pub opening_portfolio_snapshot_writes_allowed: bool,
    pub financial_event_allowlist_writes_allowed: bool,
    pub ledger_creation_allowed: bool,
    pub position_writes_allowed: bool,
    pub performance_metric_writes_allowed: bool,
    pub model_store_writes_allowed: bool,
    pub metric_store_writes_allowed: bool,
    pub training_feedback_allowed: bool,
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
pub(crate) struct OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub isolated_materializer_id: String,
    pub isolated_materializer_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration,
    pub implementation_review: OpeningPortfolioSnapshotMaterializationImplementationReviewRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub materializer_name: String,
    pub materializer_kind: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerKind,
    pub artifact_reproduction_procedure: String,
    pub rationale: String,
    pub known_limitations: String,
    pub future_input_constraints: String,
    pub future_output_constraints: String,
    pub materializer_contract: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerContract,
    pub status: String,
    pub confirmations_complete: bool,
    pub exact_current_stage_51_through_stage_135_binding_confirmed: bool,
    pub registrar_independent_from_stage_135_and_complete_prior_chain_confirmed: bool,
    pub implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: bool,
    pub proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed:
        bool,
    pub all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed: bool,
    pub future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed:
        bool,
    pub complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed:
        bool,
    pub exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed:
        bool,
    pub future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed:
        bool,
    pub deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed: bool,
    pub statement_market_values_informational_and_no_nav_or_performance_confirmed: bool,
    pub future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed:
        bool,
    pub no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    pub no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub first_execution_authorization_review_eligible: bool,
    pub first_execution_authorized: bool,
    pub input_accessed: bool,
    pub receipt_decrypted_or_read: bool,
    pub financial_rows_parsed: bool,
    pub output_candidate_created: bool,
    pub opening_portfolio_snapshot_materialized: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub model_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationIsolatedMaterializerItem {
    pub materializer: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord,
    pub approved_review_binding_current: bool,
    pub first_execution_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_implementations:
        Vec<IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation>,
    pub registration_eligible_count: usize,
    pub materializer_count: usize,
    pub current_binding_materializer_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub items: Vec<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerItem>,
    pub materializer_status: String,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub input_accessed: bool,
    pub receipt_decrypted_or_read: bool,
    pub financial_rows_parsed: bool,
    pub output_candidate_created: bool,
    pub opening_portfolio_snapshot_present: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationIsolatedMaterializerReadinessSummary {
    pub registration_eligible_count: usize,
    pub materializer_count: usize,
    pub current_binding_materializer_count: usize,
    pub first_execution_authorization_review_eligible_count: usize,
    pub materializer_status: String,
}

pub(crate) async fn handle_get_opening_portfolio_snapshot_materialization_isolated_materializers(
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
            warn!(%error, "opening-portfolio snapshot materialization isolated materializer registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "期初组合快照隔离物化器规格登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_opening_portfolio_snapshot_materialization_isolated_materializer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    if implementation_id != request.expected_stage_134_implementation_id {
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

pub(crate) async fn opening_portfolio_snapshot_materialization_isolated_materializer_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        OpeningPortfolioSnapshotMaterializationIsolatedMaterializerReadinessSummary {
            registration_eligible_count: value.registration_eligible_count,
            materializer_count: value.materializer_count,
            current_binding_materializer_count: value.current_binding_materializer_count,
            first_execution_authorization_review_eligible_count: value
                .first_execution_authorization_review_eligible_count,
            materializer_status: value.materializer_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn opening_portfolio_snapshot_materializers_for_first_execution_authorization_review(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord>, String> {
    let approved =
        independently_approved_opening_portfolio_snapshot_materialization_implementations_for_isolated_materializer_specification_registration(
            state,
        )
        .await?;
    let mut materializers = read_materializers(state)
        .await?
        .into_iter()
        .filter(|materializer| {
            approved
                .iter()
                .any(|value| binding_matches(materializer, value))
                && materializer.first_execution_authorization_review_eligible
        })
        .collect::<Vec<_>>();
    materializers.sort_by(|left, right| right.registered_at.cmp(&left.registered_at));
    Ok(materializers)
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry, String> {
    let approved =
        independently_approved_opening_portfolio_snapshot_materialization_implementations_for_isolated_materializer_specification_registration(
            state,
        )
        .await?;
    let mut items = read_materializers(state)
        .await?
        .into_iter()
        .map(|materializer| {
            let current = approved
                .iter()
                .any(|value| binding_matches(&materializer, value));
            OpeningPortfolioSnapshotMaterializationIsolatedMaterializerItem {
                first_execution_authorization_review_eligible: current
                    && materializer.first_execution_authorization_review_eligible,
                approved_review_binding_current: current,
                materializer,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .materializer
            .registered_at
            .cmp(&left.materializer.registered_at)
    });
    let registered_review_ids = items
        .iter()
        .map(|item| item.materializer.implementation_review.review_id.as_str())
        .collect::<BTreeSet<_>>();
    let eligible_implementations = approved
        .iter()
        .filter(|value| !registered_review_ids.contains(value.review.review_id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let registration_eligible_count = eligible_implementations.len();
    let current_binding_materializer_count = items
        .iter()
        .filter(|item| item.approved_review_binding_current)
        .count();
    let first_execution_authorization_review_eligible_count = items
        .iter()
        .filter(|item| item.first_execution_authorization_review_eligible)
        .count();
    let materializer_status = if approved.is_empty() {
        "waiting_stage_135_independently_approved_opening_portfolio_snapshot_materialization_implementation"
    } else if first_execution_authorization_review_eligible_count > 0 {
        "opening_portfolio_snapshot_materialization_isolated_materializer_registered_not_authorized_not_run"
    } else if registration_eligible_count > 0 {
        "ready_for_opening_portfolio_snapshot_materialization_isolated_materializer_specification_registration"
    } else {
        "opening_portfolio_snapshot_materialization_isolated_materializer_binding_stale_fail_closed"
    };
    Ok(OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_implementations,
        registration_eligible_count,
        materializer_count: items.len(),
        current_binding_materializer_count,
        first_execution_authorization_review_eligible_count,
        items,
        materializer_status: materializer_status.to_string(),
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_instantiated: false,
        input_accessed: false,
        receipt_decrypted_or_read: false,
        financial_rows_parsed: false,
        output_candidate_created: false,
        opening_portfolio_snapshot_present: false,
        financial_event_allowlist_nonempty: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 136 只把 Stage 135 独立批准绑定到未来物化器工件身份、不可变代码版本、固定非特权 runtime、Stage 133 独立验证且内容寻址的加密 receipt 输入，以及 create-once 不可信输出。当前没有源码、可执行工件、入口、runtime、挂载、读取、解密、解析、候选或期初快照；下一步只能进入 Stage 137 责任链外首次执行授权复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest,
) -> Result<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord, String> {
    let _lock = acquire_lock(state, &request.expected_stage_134_implementation_id).await?;
    let approved =
        independently_approved_opening_portfolio_snapshot_materialization_implementations_for_isolated_materializer_specification_registration(
            state,
        )
        .await?;
    let value = approved
        .iter()
        .find(|value| {
            value.implementation.implementation_id == request.expected_stage_134_implementation_id
        })
        .ok_or_else(|| {
            "当前没有这条绑定有效且已独立批准的 Stage 134 期初快照物化实现".to_string()
        })?;
    validate_expected_binding(value, &request)?;
    if read_materializers(state)
        .await?
        .iter()
        .any(|materializer| materializer.implementation_review.review_id == value.review.review_id)
    {
        return Err("这条 Stage 135 批准已经登记过隔离 materializer 规格".to_string());
    }
    let record = new_record(value, registrar_id, &request, Utc::now())?;
    write_immutable_json(
        &materializer_root(state).join(format!("{}.json", record.isolated_materializer_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn validate_expected_binding(
    approved: &IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation,
    request: &RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest,
) -> Result<(), String> {
    let implementation = &approved.implementation;
    let review = &approved.review;
    let contract = &implementation.implementation_contract;
    if request.expected_stage_134_implementation_id != implementation.implementation_id
        || request.expected_stage_134_implementation_sha256 != implementation.implementation_sha256
        || request.expected_stage_134_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_stage_135_review_id != review.review_id
        || request.expected_stage_135_review_sha256 != review.review_sha256
        || request.expected_stage_135_independent_audit_sha256
            != review.independent_audit.audit_sha256
        || request.expected_stage_133_validation_sha256 != contract.stage_133_validation_sha256
        || request.expected_stage_132_result_sha256 != contract.stage_132_result_sha256
        || request.expected_stage_131_claim_sha256 != contract.stage_131_claim_sha256
        || request.expected_receipt_manifest_sha256 != contract.receipt_manifest_sha256
        || request.expected_stage_125_specification_sha256
            != contract.stage_125_specification_sha256
    {
        return Err("Stage 125/131–135 期初快照物化规格、receipt、claim、result、validation、实现、审计或复核绑定已经变化".to_string());
    }
    Ok(())
}

fn new_record(
    approved: &IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation,
    registrar_id: &str,
    request: &RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest,
    registered_at: DateTime<Utc>,
) -> Result<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord, String> {
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(approved);
    if registrar_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|value| value == registrar_id)
    {
        return Err("物化器规格登记人必须独立于 Stage 135 和完整 Stage 51–135 责任链".to_string());
    }
    if !request_checks(request) {
        return Err("必须逐项确认 Stage 136 物化器规格、完整快照语义与零权限边界".to_string());
    }
    if !valid_sha256(&request.proposed_materializer_artifact_sha256) {
        return Err("未来 materializer 工件 SHA-256 无效".to_string());
    }
    let materializer_name = bounded_required(
        &request.materializer_name,
        MAX_NAME_CHARS,
        "materializer 名称",
    )?;
    let materializer_spec_revision = bounded_required(
        &request.materializer_spec_revision,
        MAX_REVISION_CHARS,
        "materializer 规格版本",
    )?;
    let proposed_materializer_code_revision = bounded_required(
        &request.proposed_materializer_code_revision,
        MAX_REVISION_CHARS,
        "未来 materializer 代码版本",
    )?;
    let mut record = OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        isolated_materializer_id: String::new(),
        isolated_materializer_spec_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        implementation: approved.implementation.clone(),
        implementation_review: approved.review.clone(),
        excluded_prior_actor_ids,
        materializer_name,
        materializer_kind: request.materializer_kind.clone(),
        artifact_reproduction_procedure: bounded_required(
            &request.artifact_reproduction_procedure,
            MAX_TEXT_CHARS,
            "工件复现步骤",
        )?,
        rationale: bounded_required(&request.rationale, MAX_TEXT_CHARS, "登记理由")?,
        known_limitations: bounded_required(
            &request.known_limitations,
            MAX_TEXT_CHARS,
            "已知局限",
        )?,
        future_input_constraints: bounded_required(
            &request.future_input_constraints,
            MAX_TEXT_CHARS,
            "未来输入约束",
        )?,
        future_output_constraints: bounded_required(
            &request.future_output_constraints,
            MAX_TEXT_CHARS,
            "未来输出约束",
        )?,
        materializer_contract: materializer_contract(
            approved,
            &materializer_spec_revision,
            &proposed_materializer_code_revision,
            &request.proposed_materializer_artifact_sha256,
        )?,
        status: STATUS.to_string(),
        confirmations_complete: true,
        exact_current_stage_51_through_stage_135_binding_confirmed: true,
        registrar_independent_from_stage_135_and_complete_prior_chain_confirmed: true,
        implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed:
            true,
        proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed:
            true,
        all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed: true,
        future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed:
            true,
        complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed: true,
        exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed:
            true,
        future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed:
            true,
        deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed: true,
        future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
            true,
        statement_market_values_informational_and_no_nav_or_performance_confirmed: true,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed:
            true,
        no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
            true,
        no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
            true,
        registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed:
            true,
        no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        first_execution_authorization_review_eligible: true,
        first_execution_authorized: false,
        input_accessed: false,
        receipt_decrypted_or_read: false,
        financial_rows_parsed: false,
        output_candidate_created: false,
        opening_portfolio_snapshot_materialized: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        position_or_cash_written: false,
        nav_or_performance_written: false,
        model_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.isolated_materializer_spec_sha256 = materializer_fingerprint(&record)?;
    record.isolated_materializer_id = record.isolated_materializer_spec_sha256[..32].to_string();
    validate_opening_portfolio_snapshot_materialization_isolated_materializer_for_review(&record)?;
    Ok(record)
}

fn materializer_contract(
    approved: &IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation,
    materializer_spec_revision: &str,
    proposed_materializer_code_revision: &str,
    proposed_materializer_artifact_sha256: &str,
) -> Result<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerContract, String> {
    let implementation = &approved.implementation;
    let source = &implementation.implementation_contract;
    let mut contract = OpeningPortfolioSnapshotMaterializationIsolatedMaterializerContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        stage_135_implementation_review_id: approved.review.review_id.clone(),
        stage_135_implementation_review_sha256: approved.review.review_sha256.clone(),
        stage_135_independent_audit_sha256: approved.review.independent_audit.audit_sha256.clone(),
        stage_134_implementation_id: implementation.implementation_id.clone(),
        stage_134_implementation_sha256: implementation.implementation_sha256.clone(),
        stage_134_implementation_contract_sha256: source.contract_sha256.clone(),
        stage_133_validation_sha256: source.stage_133_validation_sha256.clone(),
        stage_132_result_sha256: source.stage_132_result_sha256.clone(),
        stage_131_claim_sha256: source.stage_131_claim_sha256.clone(),
        receipt_manifest_sha256: source.receipt_manifest_sha256.clone(),
        stage_125_specification_sha256: source.stage_125_specification_sha256.clone(),
        exact_approved_implementation_contract: source.clone(),
        materializer_spec_revision: materializer_spec_revision.to_string(),
        proposed_materializer_code_revision: proposed_materializer_code_revision.to_string(),
        proposed_materializer_artifact_sha256: proposed_materializer_artifact_sha256
            .to_ascii_lowercase(),
        runtime_identity: RUNTIME_IDENTITY.to_string(),
        runtime_version: RUNTIME_VERSION.to_string(),
        future_input_envelope: FUTURE_INPUT_ENVELOPE.to_string(),
        future_output_envelope: FUTURE_OUTPUT_ENVELOPE.to_string(),
        next_gate: NEXT_GATE.to_string(),
        specification_registered: true,
        future_materializer_artifact_identity_bound: true,
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_identity_bound: true,
        runtime_instantiated: false,
        input_mount_present: false,
        input_read_allowed: false,
        data_access_authorized: false,
        root_filesystem_read_only_required: true,
        ephemeral_working_directory_required: true,
        run_as_unprivileged_required: true,
        no_new_privileges_required: true,
        future_input_read_only_required: true,
        future_input_content_addressed_required: true,
        future_input_stage_133_independent_validation_required: true,
        future_decryption_ephemeral_memory_only_required: true,
        plaintext_persistence_allowed: false,
        remote_fetch_allowed: false,
        future_output_create_once_required: true,
        future_output_untrusted_required: true,
        future_output_independent_validation_required: true,
        future_output_investment_semantics_allowed: false,
        future_output_order_intent_allowed: false,
        environment_inheritance_allowed: false,
        allowed_environment_variables: Vec::new(),
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        output_candidate_writes_allowed: false,
        opening_portfolio_snapshot_writes_allowed: false,
        financial_event_allowlist_writes_allowed: false,
        ledger_creation_allowed: false,
        position_writes_allowed: false,
        performance_metric_writes_allowed: false,
        model_store_writes_allowed: false,
        metric_store_writes_allowed: false,
        training_feedback_allowed: false,
        reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
        maximum_parallel_runs: 1,
        maximum_memory_mib: MAXIMUM_MEMORY_MIB,
        maximum_wall_clock_seconds: MAXIMUM_WALL_CLOCK_SECONDS,
        maximum_cpu_millicores: MAXIMUM_CPU_MILLICORES,
        maximum_process_count: MAXIMUM_PROCESS_COUNT,
        maximum_output_bytes: MAXIMUM_OUTPUT_BYTES,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn request_checks(
    request: &RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_135_binding_confirmed
        && request.registrar_independent_from_stage_135_and_complete_prior_chain_confirmed
        && request.implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed
        && request.proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed
        && request.all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed
        && request.future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed
        && request.complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed
        && request.exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed
        && request.future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed
        && request.deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed
        && request.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && request.statement_market_values_informational_and_no_nav_or_performance_confirmed
        && request.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed
        && request.no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && request.no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && request.registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

pub(crate) fn validate_opening_portfolio_snapshot_materialization_isolated_materializer_for_review(
    record: &OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord,
) -> Result<(), String> {
    let contract = &record.materializer_contract;
    let closed = !contract.source_artifact_present
        && !contract.executable_artifact_present
        && !contract.callable_entrypoint_present
        && contract.runtime_identity_bound
        && !contract.runtime_instantiated
        && !contract.input_mount_present
        && !contract.input_read_allowed
        && !contract.data_access_authorized
        && !contract.environment_inheritance_allowed
        && contract.allowed_environment_variables.is_empty()
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.output_candidate_writes_allowed
        && !contract.opening_portfolio_snapshot_writes_allowed
        && !contract.financial_event_allowlist_writes_allowed
        && !contract.ledger_creation_allowed
        && !contract.position_writes_allowed
        && !contract.performance_metric_writes_allowed
        && !contract.model_store_writes_allowed
        && !contract.metric_store_writes_allowed
        && !contract.training_feedback_allowed
        && !contract.reward_allowed
        && !contract.order_generation_allowed
        && !contract.broker_access_allowed
        && !contract.trading_allowed;
    let record_closed = !record.first_execution_authorized
        && !record.input_accessed
        && !record.receipt_decrypted_or_read
        && !record.financial_rows_parsed
        && !record.output_candidate_created
        && !record.opening_portfolio_snapshot_materialized
        && !record.financial_event_allowlist_nonempty
        && !record.ledger_created
        && !record.position_or_cash_written
        && !record.nav_or_performance_written
        && !record.model_store_written
        && !record.metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let valid = record.schema_version == RECORD_SCHEMA_VERSION
        && record.policy_version == POLICY_VERSION
        && valid_id(&record.isolated_materializer_id)
        && valid_sha256(&record.isolated_materializer_spec_sha256)
        && record.isolated_materializer_id == record.isolated_materializer_spec_sha256[..32]
        && record.isolated_materializer_spec_sha256 == materializer_fingerprint(record)?
        && record.status == STATUS
        && record.confirmations_complete
        && record.exact_current_stage_51_through_stage_135_binding_confirmed
        && record.registrar_independent_from_stage_135_and_complete_prior_chain_confirmed
        && record
            .implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed
        && record
            .proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed
        && record.all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed
        && record
            .future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed
        && record.complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed
        && record
            .exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed
        && record.future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed
        && record.deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed
        && record
            .future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && record.statement_market_values_informational_and_no_nav_or_performance_confirmed
        && record
            .fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed
        && record
            .no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && record
            .no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && record.registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed
        && record.no_unconfirmed_hari_or_old_wang_logic_claimed
        && !record
            .excluded_prior_actor_ids
            .contains(&record.registered_by)
        && sorted_unique(&record.excluded_prior_actor_ids)
        && record.first_execution_authorization_review_eligible
        && contract.schema_version == CONTRACT_SCHEMA_VERSION
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?
        && contract.next_gate == NEXT_GATE
        && contract.specification_registered
        && contract.future_materializer_artifact_identity_bound
        && valid_sha256(&contract.proposed_materializer_artifact_sha256)
        && contract.root_filesystem_read_only_required
        && contract.ephemeral_working_directory_required
        && contract.run_as_unprivileged_required
        && contract.no_new_privileges_required
        && contract.future_input_read_only_required
        && contract.future_input_content_addressed_required
        && contract.future_input_stage_133_independent_validation_required
        && contract.future_decryption_ephemeral_memory_only_required
        && !contract.plaintext_persistence_allowed
        && !contract.remote_fetch_allowed
        && contract.future_output_create_once_required
        && contract.future_output_untrusted_required
        && contract.future_output_independent_validation_required
        && closed
        && record_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 136 期初快照隔离物化器规格无效、漂移或越权".to_string())
}

fn binding_matches(
    materializer: &OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord,
    approved: &IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation,
) -> bool {
    materializer.implementation == approved.implementation
        && materializer.implementation_review == approved.review
        && validate_opening_portfolio_snapshot_materialization_isolated_materializer_for_review(
            materializer,
        )
        .is_ok()
}

fn expected_excluded_prior_actor_ids(
    approved: &IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation,
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

fn materializer_fingerprint(
    record: &OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord,
) -> Result<String, String> {
    fingerprint_without(
        record,
        &[
            "isolated_materializer_id",
            "isolated_materializer_spec_sha256",
        ],
    )
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 136 指纹载荷无效".to_string())?;
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

async fn read_materializers(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord>, String> {
    let directory = materializer_root(state);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut materializers = Vec::new();
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
            let materializer: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerRecord =
                serde_json::from_slice(
                    &tokio::fs::read(&path)
                        .await
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
            if path.file_stem().and_then(|value| value.to_str())
                != Some(materializer.isolated_materializer_id.as_str())
            {
                return Err("Stage 136 materializer 文件名与记录 ID 不一致".to_string());
            }
            validate_opening_portfolio_snapshot_materialization_isolated_materializer_for_review(
                &materializer,
            )?;
            materializers.push(materializer);
        }
    }
    materializers.sort_by(|left, right| left.registered_at.cmp(&right.registered_at));
    let mut ids = BTreeSet::new();
    let mut reviews = BTreeSet::new();
    for materializer in &materializers {
        if !ids.insert(materializer.isolated_materializer_id.clone())
            || !reviews.insert(materializer.implementation_review.review_id.clone())
        {
            return Err("Stage 136 materializer 登记重复或分叉".to_string());
        }
    }
    Ok(materializers)
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
fn materializer_root(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-snapshot-materialization-isolated-materializers")
}

struct MaterializerLock(PathBuf);
impl Drop for MaterializerLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(
    state: &AppState,
    implementation_id: &str,
) -> Result<MaterializerLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "opening-portfolio-snapshot-materialization-isolated-materializer-{implementation_id}.lock"
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
        .map_err(|_| "已有同一 Stage 136 materializer 登记正在进行".to_string())?;
    Ok(MaterializerLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest {
        RegisterOpeningPortfolioSnapshotMaterializationIsolatedMaterializerRequest {
            expected_stage_135_review_id: "r".repeat(32),
            expected_stage_135_review_sha256: "1".repeat(64),
            expected_stage_135_independent_audit_sha256: "2".repeat(64),
            expected_stage_134_implementation_id: "i".repeat(32),
            expected_stage_134_implementation_sha256: "3".repeat(64),
            expected_stage_134_implementation_contract_sha256: "4".repeat(64),
            expected_stage_133_validation_sha256: "5".repeat(64),
            expected_stage_132_result_sha256: "6".repeat(64),
            expected_stage_131_claim_sha256: "7".repeat(64),
            expected_receipt_manifest_sha256: "8".repeat(64),
            expected_stage_125_specification_sha256: "9".repeat(64),
            materializer_name: "期初组合快照隔离物化器".to_string(),
            materializer_kind: OpeningPortfolioSnapshotMaterializationIsolatedMaterializerKind::EphemeralDeterministicPdfCsvJsonSnapshotMaterializationSpecification,
            materializer_spec_revision: "v1".to_string(),
            proposed_materializer_code_revision: "immutable-revision".to_string(),
            proposed_materializer_artifact_sha256: "a".repeat(64),
            artifact_reproduction_procedure: "固定源码、依赖和构建参数后复现 SHA-256。".to_string(),
            rationale: "只冻结未来物化器身份和零权限运行边界。".to_string(),
            known_limitations: "尚无工件、入口、输入或执行授权。".to_string(),
            future_input_constraints: "只接受 Stage 133 独立验证的内容寻址加密 receipt。".to_string(),
            future_output_constraints: "只允许 create-once 不可信候选并另行验证。".to_string(),
            exact_current_stage_51_through_stage_135_binding_confirmed: true,
            registrar_independent_from_stage_135_and_complete_prior_chain_confirmed: true,
            implementation_review_audit_contract_and_specification_hashes_reproduced_confirmed: true,
            proposed_artifact_identity_code_revision_and_reproduction_procedure_bound_but_artifact_not_present_confirmed: true,
            all_ten_snapshot_materialization_functions_and_canonical_schemas_preserved_confirmed: true,
            future_input_only_stage_133_independently_validated_read_only_content_addressed_encrypted_receipt_confirmed: true,
            complete_accounts_cash_positions_options_liabilities_unsettled_and_whole_snapshot_failure_semantics_preserved_confirmed: true,
            exact_decimal_signed_quantities_identity_corporate_action_and_row_provenance_semantics_preserved_confirmed: true,
            future_decryption_only_in_isolated_ephemeral_memory_and_no_plaintext_persistence_confirmed: true,
            deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed: true,
            statement_market_values_informational_and_no_nav_or_performance_confirmed: true,
            future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
            fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_bounded_resources_confirmed: true,
            no_source_executable_entrypoint_runtime_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
            registration_only_opens_stage_137_chain_external_first_execution_authorization_review_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn complete_registration_confirmations_are_required() {
        assert!(request_checks(&request()));
        let mut incomplete = request();
        incomplete.deterministic_pdf_csv_json_parsing_and_no_remote_fetch_confirmed = false;
        assert!(!request_checks(&incomplete));
    }

    #[test]
    fn proposed_artifact_identity_must_be_sha256() {
        assert!(valid_sha256(
            &request().proposed_materializer_artifact_sha256
        ));
        assert!(!valid_sha256("not-a-sha256"));
    }

    #[test]
    fn runtime_is_bounded_single_process_and_not_parallel() {
        assert_eq!(MAXIMUM_MEMORY_MIB, 1_024);
        assert_eq!(MAXIMUM_WALL_CLOCK_SECONDS, 300);
        assert_eq!(MAXIMUM_CPU_MILLICORES, 1_000);
        assert_eq!(MAXIMUM_PROCESS_COUNT, 1);
        assert_eq!(MAXIMUM_OUTPUT_BYTES, 64 * 1024 * 1024);
    }

    #[test]
    fn envelopes_preserve_validated_encrypted_input_and_untrusted_output() {
        assert!(FUTURE_INPUT_ENVELOPE.contains("stage_133"));
        assert!(FUTURE_INPUT_ENVELOPE.contains("encrypted_receipt"));
        assert!(FUTURE_OUTPUT_ENVELOPE.contains("create_once_untrusted"));
        assert!(NEXT_GATE.contains("stage_137"));
    }

    #[test]
    fn actor_exclusion_requires_sorted_unique_chain() {
        assert!(sorted_unique(&["a".to_string(), "b".to_string()]));
        assert!(!sorted_unique(&["b".to_string(), "a".to_string()]));
        assert!(!sorted_unique(&["a".to_string(), "a".to_string()]));
    }
}
