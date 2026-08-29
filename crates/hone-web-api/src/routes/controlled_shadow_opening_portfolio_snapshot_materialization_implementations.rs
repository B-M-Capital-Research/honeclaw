//! Stage 134 create-once registration of a zero-capability opening-portfolio snapshot
//! materialization implementation contract.
//!
//! Registration binds one independently validated Stage 133 encrypted receipt to the exact
//! Stage 125 canonical snapshot specification. It freezes deterministic parsing,
//! reconciliation, completeness, provenance and failure semantics for a future isolated
//! materializer. This module exposes no decrypt/read/parse/run endpoint and creates no financial
//! state.

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

use super::controlled_shadow_opening_portfolio_snapshot_governance_specifications::{
    OpeningPortfolioCanonicalSnapshotSchema, OpeningPortfolioSnapshotGovernanceSpecification,
    OpeningPortfolioSourceArtifactContract,
};
use super::controlled_shadow_opening_portfolio_source_artifact_receipt_validations::{
    IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt,
    OpeningPortfolioSourceArtifactReceiptValidationRecord,
    independently_validated_opening_portfolio_source_artifact_receipts_for_stage_134,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-implementation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-implementation-registration-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-zero-capability-contract-v1";
const POLICY_VERSION: &str = "hone-opening-portfolio-snapshot-materialization-implementation-create-once-v1-zero-capability-no-input-read";
const PROTOCOL_VERSION: &str = "hone-opening-portfolio-snapshot-materialization-v1-not-executable";
const STATUS: &str = "registered_not_reviewed_not_run";
const NEXT_GATE: &str =
    "stage_135_opening_portfolio_snapshot_materialization_implementation_independent_review";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_RECORD_FILE_BYTES: u64 = 2 * 1024 * 1024;
const FUTURE_MAX_INPUT_ARTIFACTS: usize = 64;
const FUTURE_MAX_INPUT_BYTES: u64 = 256 * 1024 * 1024;
const FUTURE_MAX_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;
const FUTURE_MAX_OUTPUT_ROWS: usize = 1_000_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest {
    expected_stage_133_validation_id: String,
    expected_stage_133_validation_sha256: String,
    expected_stage_132_result_sha256: String,
    expected_stage_131_claim_sha256: String,
    expected_receipt_id: String,
    expected_receipt_manifest_sha256: String,
    expected_stage_125_specification_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_description: String,
    deterministic_parser_and_adapter_semantics: String,
    account_scope_and_completeness_semantics: String,
    exact_decimal_and_signed_quantity_semantics: String,
    instrument_identity_and_corporate_action_semantics: String,
    row_provenance_and_redaction_semantics: String,
    whole_snapshot_failure_and_correction_semantics: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_current_stage_51_through_stage_133_binding_confirmed: bool,
    registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed:
        bool,
    validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed: bool,
    exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed: bool,
    future_input_only_independently_validated_content_addressed_receipt_confirmed: bool,
    future_decryption_only_inside_isolated_ephemeral_materializer_confirmed: bool,
    deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: bool,
    account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed: bool,
    exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: bool,
    instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: bool,
    no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed:
        bool,
    statement_market_values_informational_and_no_nav_or_performance_confirmed: bool,
    every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed:
        bool,
    future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed:
        bool,
    contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: bool,
    no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    future_stage_135_chain_external_independent_implementation_review_required_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityOpeningPortfolioSnapshotMaterializationAuthorityBoundary {
    pub decryption_key_access_allowed: bool,
    pub encrypted_artifact_read_allowed: bool,
    pub plaintext_persistence_allowed: bool,
    pub financial_row_parsing_allowed: bool,
    pub executable_artifact_or_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub output_candidate_present: bool,
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
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub protocol_version: String,
    pub immutable_code_revision: String,
    pub stage_133_validation_id: String,
    pub stage_133_validation_sha256: String,
    pub stage_132_result_sha256: String,
    pub stage_131_claim_sha256: String,
    pub receipt_id: String,
    pub receipt_manifest_sha256: String,
    pub stage_125_specification_sha256: String,
    pub exact_source_artifact_contract: OpeningPortfolioSourceArtifactContract,
    pub exact_canonical_snapshot_schema: OpeningPortfolioCanonicalSnapshotSchema,
    pub accepted_input_formats: Vec<String>,
    pub future_input_artifact_count_limit: usize,
    pub future_input_byte_limit: u64,
    pub future_output_byte_limit: u64,
    pub future_output_row_limit: usize,
    pub future_input_envelope_schema: String,
    pub future_output_candidate_schema: String,
    pub validate_receipt_and_specification_binding_function_id: String,
    pub decrypt_in_ephemeral_memory_function_id: String,
    pub parse_provider_pdf_csv_json_deterministically_function_id: String,
    pub normalize_account_scope_function_id: String,
    pub normalize_cash_positions_options_liabilities_and_unsettled_activity_function_id: String,
    pub reconcile_instrument_identity_and_corporate_actions_function_id: String,
    pub enforce_full_snapshot_completeness_function_id: String,
    pub canonicalize_exact_decimal_output_function_id: String,
    pub attach_source_row_provenance_function_id: String,
    pub create_once_untrusted_candidate_function_id: String,
    pub binary_floating_point_allowed: bool,
    pub manual_balance_or_position_entry_allowed: bool,
    pub missing_value_defaulting_allowed: bool,
    pub cash_position_quantity_cost_basis_or_weight_inference_allowed: bool,
    pub partial_account_scope_allowed: bool,
    pub unsupported_asset_silently_dropped: bool,
    pub statement_market_value_used_as_accounting_mark: bool,
    pub raw_account_numbers_or_credentials_in_output_logs_or_errors_allowed: bool,
    pub every_output_row_requires_artifact_sha256_and_source_locator: bool,
    pub whole_snapshot_fails_on_missing_ambiguous_unsupported_or_unreconciled_input: bool,
    pub output_create_once: bool,
    pub output_untrusted: bool,
    pub future_independent_output_validation_required: bool,
    pub future_snapshot_admission_review_required: bool,
    pub correction_requires_new_candidate: bool,
    pub registered_not_run: bool,
    pub future_independent_implementation_review_required: bool,
    pub next_gate: String,
    pub authority_boundary: ZeroCapabilityOpeningPortfolioSnapshotMaterializationAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationConfirmations {
    pub exact_current_stage_51_through_stage_133_binding_confirmed: bool,
    pub registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed:
        bool,
    pub validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed: bool,
    pub exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed: bool,
    pub future_input_only_independently_validated_content_addressed_receipt_confirmed: bool,
    pub future_decryption_only_inside_isolated_ephemeral_materializer_confirmed: bool,
    pub deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: bool,
    pub account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed: bool,
    pub exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: bool,
    pub instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: bool,
    pub no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed:
        bool,
    pub statement_market_values_informational_and_no_nav_or_performance_confirmed: bool,
    pub every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed:
        bool,
    pub future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed:
        bool,
    pub contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed:
        bool,
    pub no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub future_stage_135_chain_external_independent_implementation_review_required_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationRegistration {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub upstream_stage_133_validation: OpeningPortfolioSourceArtifactReceiptValidationRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain:
        bool,
    pub implementation_name: String,
    pub implementation_description: String,
    pub deterministic_parser_and_adapter_semantics: String,
    pub account_scope_and_completeness_semantics: String,
    pub exact_decimal_and_signed_quantity_semantics: String,
    pub instrument_identity_and_corporate_action_semantics: String,
    pub row_provenance_and_redaction_semantics: String,
    pub whole_snapshot_failure_and_correction_semantics: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub implementation_contract: OpeningPortfolioSnapshotMaterializationImplementationContract,
    pub status: String,
    pub confirmations: OpeningPortfolioSnapshotMaterializationImplementationConfirmations,
    pub confirmations_complete: bool,
    pub zero_capability_implementation_contract_registered: bool,
    pub future_stage_135_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub receipt_decrypted_or_read: bool,
    pub financial_rows_parsed: bool,
    pub output_candidate_created: bool,
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
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationCandidate {
    pub stage_131_attempt_id: String,
    pub stage_133_validation_id: String,
    pub stage_133_validation_sha256: String,
    pub stage_132_result_sha256: String,
    pub stage_131_claim_sha256: String,
    pub receipt_id: String,
    pub receipt_manifest_sha256: String,
    pub stage_125_specification_sha256: String,
    pub source_provider_name: String,
    pub portfolio_scope_alias: String,
    pub artifact_count: usize,
    pub registrar_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationItem {
    pub candidate: OpeningPortfolioSnapshotMaterializationImplementationCandidate,
    pub implementation: Option<OpeningPortfolioSnapshotMaterializationImplementationRegistration>,
    pub registration_eligible: bool,
    pub upstream_binding_current: bool,
    pub future_stage_135_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_endpoint_available: bool,
    pub items: Vec<OpeningPortfolioSnapshotMaterializationImplementationItem>,
    pub independently_validated_receipt_count: usize,
    pub registration_eligible_count: usize,
    pub implementation_contract_count: usize,
    pub current_binding_implementation_contract_count: usize,
    pub future_stage_135_independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub receipt_decrypted_or_read: bool,
    pub financial_rows_parsed: bool,
    pub output_candidate_created: bool,
    pub opening_portfolio_snapshot_present: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub next_gate: String,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationReadinessSummary {
    pub independently_validated_receipt_count: usize,
    pub registration_eligible_count: usize,
    pub implementation_contract_count: usize,
    pub current_binding_implementation_contract_count: usize,
    pub future_stage_135_independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentlyReviewableOpeningPortfolioSnapshotMaterializationImplementation {
    pub implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration,
}

pub(crate) async fn handle_get_opening_portfolio_snapshot_materialization_implementations(
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
            warn!(%error, "Stage 134 snapshot materialization implementation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "期初组合快照物化零能力实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_opening_portfolio_snapshot_materialization_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(validation_id): AxumPath<String>,
    Json(request): Json<RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &validation_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn opening_portfolio_snapshot_materialization_implementation_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        OpeningPortfolioSnapshotMaterializationImplementationReadinessSummary {
            independently_validated_receipt_count: value.independently_validated_receipt_count,
            registration_eligible_count: value.registration_eligible_count,
            implementation_contract_count: value.implementation_contract_count,
            current_binding_implementation_contract_count: value
                .current_binding_implementation_contract_count,
            future_stage_135_independent_implementation_review_eligible_count: value
                .future_stage_135_independent_implementation_review_eligible_count,
            implementation_status: value.implementation_status,
        },
    )
}

pub(crate) async fn independently_reviewable_opening_portfolio_snapshot_materialization_implementations(
    state: &AppState,
) -> Result<Vec<IndependentlyReviewableOpeningPortfolioSnapshotMaterializationImplementation>, String>
{
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            item.future_stage_135_independent_implementation_review_eligible
                .then_some(item.implementation?)
                .map(|implementation| {
                    IndependentlyReviewableOpeningPortfolioSnapshotMaterializationImplementation {
                        implementation,
                    }
                })
        })
        .collect())
}

pub(crate) fn validate_opening_portfolio_snapshot_materialization_implementation_for_review(
    value: &OpeningPortfolioSnapshotMaterializationImplementationRegistration,
) -> Result<(), String> {
    validate_record(value)
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationRegistry, String> {
    let sources =
        independently_validated_opening_portfolio_source_artifact_receipts_for_stage_134(state)
            .await?;
    let records = read_records(state).await?;
    for record in &records {
        let source = sources
            .iter()
            .find(|source| {
                source.validation.validation_id
                    == record.upstream_stage_133_validation.validation_id
            })
            .ok_or_else(|| "Stage 134 存在不再绑定当前 Stage 133 通过链的孤立登记".to_string())?;
        if !record_matches_source(record, source) {
            return Err("Stage 134 登记与当前 Stage 133/125 来源绑定漂移".to_string());
        }
    }
    let by_validation = records
        .iter()
        .map(|record| {
            (
                record.upstream_stage_133_validation.validation_id.as_str(),
                record,
            )
        })
        .collect::<HashMap<_, _>>();
    let mut items = Vec::new();
    for source in &sources {
        let candidate = candidate_from_source(source)?;
        let implementation = by_validation
            .get(source.validation.validation_id.as_str())
            .map(|value| (*value).clone());
        let upstream_binding_current = implementation
            .as_ref()
            .is_some_and(|record| record_matches_source(record, source));
        items.push(OpeningPortfolioSnapshotMaterializationImplementationItem {
            candidate,
            registration_eligible: implementation.is_none(),
            future_stage_135_independent_implementation_review_eligible: upstream_binding_current,
            upstream_binding_current,
            implementation,
        });
    }
    let registration_eligible_count = items
        .iter()
        .filter(|item| item.registration_eligible)
        .count();
    let current_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let stage_135_count = items
        .iter()
        .filter(|item| item.future_stage_135_independent_implementation_review_eligible)
        .count();
    let status = if stage_135_count > 0 {
        "zero_capability_snapshot_materialization_implementation_registered_waiting_stage_135_independent_review"
    } else if registration_eligible_count > 0 {
        "independently_validated_receipt_ready_for_stage_134_zero_capability_materialization_implementation_registration"
    } else {
        "waiting_stage_133_independently_validated_encrypted_source_artifact_receipt"
    };
    Ok(OpeningPortfolioSnapshotMaterializationImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_endpoint_available: true,
        independently_validated_receipt_count: sources.len(),
        registration_eligible_count,
        implementation_contract_count: records.len(),
        current_binding_implementation_contract_count: current_count,
        future_stage_135_independent_implementation_review_eligible_count: stage_135_count,
        items,
        implementation_status: status.to_string(),
        receipt_decrypted_or_read: false,
        financial_rows_parsed: false,
        output_candidate_created: false,
        opening_portfolio_snapshot_present: false,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        position_or_cash_written: false,
        nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        next_gate: NEXT_GATE.to_string(),
        scope: "Stage 134 只把一条 Stage 133 独立验证的加密 receipt 绑定到 Stage 125 精确来源合同与 canonical snapshot schema，并冻结未来隔离 parser/materializer 的完整性、精确十进制、逐行来源和整批失败语义。当前没有解密、来源读取、金融行解析、runtime、输出候选或真实快照；登记通过也只进入 Stage 135 责任链外实现复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    validation_id: &str,
    request: RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationRegistration, String> {
    validate_request(validation_id, &request)?;
    let _lock = acquire_lock(state, validation_id).await?;
    let sources =
        independently_validated_opening_portfolio_source_artifact_receipts_for_stage_134(state)
            .await?;
    let source = sources
        .iter()
        .find(|source| source.validation.validation_id == validation_id)
        .ok_or_else(|| "当前没有精确匹配且仍有效的 Stage 133 独立验证 receipt".to_string())?;
    validate_expected_binding(source, &request)?;
    if read_records(state)
        .await?
        .iter()
        .any(|record| record.upstream_stage_133_validation.validation_id == validation_id)
    {
        return Err("该 Stage 133 validation 已登记物化实现，禁止覆盖或重放".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(source);
    let independent = !registrar_id.trim().is_empty()
        && excluded_prior_actor_ids
            .iter()
            .all(|actor| actor != registrar_id);
    if !independent
        || !request.registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed
    {
        return Err("Stage 134 registrar 必须独立于 validator、executor、claimant 与完整既有责任链".to_string());
    }
    if !confirmations_complete(&request) {
        return Err("Stage 134 登记前必须逐项确认解析完整性、逐行来源和零能力边界".to_string());
    }
    let immutable_code_revision = bounded_required(
        &request.immutable_code_revision,
        MAX_REVISION_CHARS,
        "不可变代码版本",
    )?;
    let implementation_contract = implementation_contract(source, &immutable_code_revision)?;
    let confirmations = confirmations_from_request(&request);
    let mut record = OpeningPortfolioSnapshotMaterializationImplementationRegistration {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at: Utc::now(),
        registered_by: registrar_id.to_string(),
        upstream_stage_133_validation: source.validation.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain:
            independent,
        implementation_name: bounded_required(
            &request.implementation_name,
            MAX_NAME_CHARS,
            "实现名称",
        )?,
        implementation_description: bounded_text(&request.implementation_description, "实现说明")?,
        deterministic_parser_and_adapter_semantics: bounded_text(
            &request.deterministic_parser_and_adapter_semantics,
            "确定性解析器与适配器语义",
        )?,
        account_scope_and_completeness_semantics: bounded_text(
            &request.account_scope_and_completeness_semantics,
            "账户范围与完整性语义",
        )?,
        exact_decimal_and_signed_quantity_semantics: bounded_text(
            &request.exact_decimal_and_signed_quantity_semantics,
            "精确十进制与有符号数量语义",
        )?,
        instrument_identity_and_corporate_action_semantics: bounded_text(
            &request.instrument_identity_and_corporate_action_semantics,
            "证券身份与公司行动语义",
        )?,
        row_provenance_and_redaction_semantics: bounded_text(
            &request.row_provenance_and_redaction_semantics,
            "逐行来源与脱敏语义",
        )?,
        whole_snapshot_failure_and_correction_semantics: bounded_text(
            &request.whole_snapshot_failure_and_correction_semantics,
            "整批失败与更正语义",
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
        future_stage_135_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        receipt_decrypted_or_read: false,
        financial_rows_parsed: false,
        output_candidate_created: false,
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
        &record_directory(state).join(format!("{}.json", record.implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn implementation_contract(
    source: &IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt,
    immutable_code_revision: &str,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationContract, String> {
    let specification = exact_stage_125_specification(source);
    let mut contract = OpeningPortfolioSnapshotMaterializationImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        stage_133_validation_id: source.validation.validation_id.clone(),
        stage_133_validation_sha256: source.validation.validation_sha256.clone(),
        stage_132_result_sha256: source.result.result_sha256.clone(),
        stage_131_claim_sha256: source.claim.claim_sha256.clone(),
        receipt_id: source.receipt.receipt_id.clone(),
        receipt_manifest_sha256: source.receipt.receipt_manifest_sha256.clone(),
        stage_125_specification_sha256: specification.specification_sha256.clone(),
        exact_source_artifact_contract: specification.source_contract.clone(),
        exact_canonical_snapshot_schema: specification.canonical_snapshot_schema.clone(),
        accepted_input_formats: specification.source_contract.accepted_artifact_formats.clone(),
        future_input_artifact_count_limit: FUTURE_MAX_INPUT_ARTIFACTS,
        future_input_byte_limit: FUTURE_MAX_INPUT_BYTES,
        future_output_byte_limit: FUTURE_MAX_OUTPUT_BYTES,
        future_output_row_limit: FUTURE_MAX_OUTPUT_ROWS,
        future_input_envelope_schema: "stage_133_validation_sha256,receipt_manifest_sha256,stage_125_specification_sha256,content_addressed_encrypted_artifacts,ephemeral_decryption_key_handle".to_string(),
        future_output_candidate_schema: "candidate_id,receipt_manifest_sha256,specification_sha256,portfolio_scope_alias,reporting_currency,source_timezone,snapshot_as_of_utc,accounts,cash,positions,listed_options,liabilities,unsettled_activity,row_provenance,completeness_proof,canonical_candidate_sha256,untrusted".to_string(),
        validate_receipt_and_specification_binding_function_id:
            "opening_snapshot_validate_receipt_spec_binding_v1".to_string(),
        decrypt_in_ephemeral_memory_function_id:
            "opening_snapshot_decrypt_ephemeral_memory_only_v1".to_string(),
        parse_provider_pdf_csv_json_deterministically_function_id:
            "opening_snapshot_parse_provider_pdf_csv_json_deterministically_v1".to_string(),
        normalize_account_scope_function_id:
            "opening_snapshot_normalize_complete_account_scope_v1".to_string(),
        normalize_cash_positions_options_liabilities_and_unsettled_activity_function_id:
            "opening_snapshot_normalize_all_financial_sections_v1".to_string(),
        reconcile_instrument_identity_and_corporate_actions_function_id:
            "opening_snapshot_reconcile_instrument_identity_corporate_actions_v1".to_string(),
        enforce_full_snapshot_completeness_function_id:
            "opening_snapshot_enforce_full_completeness_fail_closed_v1".to_string(),
        canonicalize_exact_decimal_output_function_id:
            "opening_snapshot_canonicalize_exact_decimal_no_float_v1".to_string(),
        attach_source_row_provenance_function_id:
            "opening_snapshot_attach_artifact_row_provenance_v1".to_string(),
        create_once_untrusted_candidate_function_id:
            "opening_snapshot_create_once_untrusted_candidate_v1".to_string(),
        binary_floating_point_allowed: false,
        manual_balance_or_position_entry_allowed: false,
        missing_value_defaulting_allowed: false,
        cash_position_quantity_cost_basis_or_weight_inference_allowed: false,
        partial_account_scope_allowed: false,
        unsupported_asset_silently_dropped: false,
        statement_market_value_used_as_accounting_mark: false,
        raw_account_numbers_or_credentials_in_output_logs_or_errors_allowed: false,
        every_output_row_requires_artifact_sha256_and_source_locator: true,
        whole_snapshot_fails_on_missing_ambiguous_unsupported_or_unreconciled_input: true,
        output_create_once: true,
        output_untrusted: true,
        future_independent_output_validation_required: true,
        future_snapshot_admission_review_required: true,
        correction_requires_new_candidate: true,
        registered_not_run: true,
        future_independent_implementation_review_required: true,
        next_gate: NEXT_GATE.to_string(),
        authority_boundary: closed_authority_boundary(),
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    validate_contract(&contract)?;
    Ok(contract)
}

fn validate_contract(
    value: &OpeningPortfolioSnapshotMaterializationImplementationContract,
) -> Result<(), String> {
    let source = &value.exact_source_artifact_contract;
    let schema = &value.exact_canonical_snapshot_schema;
    let required_functions = [
        &value.validate_receipt_and_specification_binding_function_id,
        &value.decrypt_in_ephemeral_memory_function_id,
        &value.parse_provider_pdf_csv_json_deterministically_function_id,
        &value.normalize_account_scope_function_id,
        &value.normalize_cash_positions_options_liabilities_and_unsettled_activity_function_id,
        &value.reconcile_instrument_identity_and_corporate_actions_function_id,
        &value.enforce_full_snapshot_completeness_function_id,
        &value.canonicalize_exact_decimal_output_function_id,
        &value.attach_source_row_provenance_function_id,
        &value.create_once_untrusted_candidate_function_id,
    ];
    let valid = value.schema_version == CONTRACT_SCHEMA_VERSION
        && value.protocol_version == PROTOCOL_VERSION
        && valid_sha256(&value.contract_sha256)
        && value.contract_sha256 == fingerprint_without(value, &["contract_sha256"])?
        && valid_id(&value.stage_133_validation_id)
        && valid_id(&value.receipt_id)
        && [
            &value.stage_133_validation_sha256,
            &value.stage_132_result_sha256,
            &value.stage_131_claim_sha256,
            &value.receipt_manifest_sha256,
            &value.stage_125_specification_sha256,
        ]
        .into_iter()
        .all(|item| valid_sha256(item))
        && value.accepted_input_formats == accepted_artifact_formats()
        && value.accepted_input_formats == source.accepted_artifact_formats
        && source.original_bytes_required
        && source.content_sha256_and_byte_length_required
        && source.source_account_identifiers_must_be_pseudonymized
        && !source.raw_account_numbers_or_credentials_allowed
        && !source.manual_balance_or_position_entry_allowed
        && !source.mutable_or_overwritable_artifact_allowed
        && !schema.account_schema.trim().is_empty()
        && !schema.cash_schema.trim().is_empty()
        && !schema.position_schema.trim().is_empty()
        && !schema.listed_option_extension_schema.trim().is_empty()
        && !schema.liability_schema.trim().is_empty()
        && !schema.unsettled_activity_schema.trim().is_empty()
        && !schema.instrument_identity_precedence.is_empty()
        && !schema.supported_asset_classes.is_empty()
        && required_functions
            .into_iter()
            .all(|item| !item.trim().is_empty())
        && value.future_input_artifact_count_limit == FUTURE_MAX_INPUT_ARTIFACTS
        && value.future_input_byte_limit == FUTURE_MAX_INPUT_BYTES
        && value.future_output_byte_limit == FUTURE_MAX_OUTPUT_BYTES
        && value.future_output_row_limit == FUTURE_MAX_OUTPUT_ROWS
        && !value.binary_floating_point_allowed
        && !value.manual_balance_or_position_entry_allowed
        && !value.missing_value_defaulting_allowed
        && !value.cash_position_quantity_cost_basis_or_weight_inference_allowed
        && !value.partial_account_scope_allowed
        && !value.unsupported_asset_silently_dropped
        && !value.statement_market_value_used_as_accounting_mark
        && !value.raw_account_numbers_or_credentials_in_output_logs_or_errors_allowed
        && value.every_output_row_requires_artifact_sha256_and_source_locator
        && value.whole_snapshot_fails_on_missing_ambiguous_unsupported_or_unreconciled_input
        && value.output_create_once
        && value.output_untrusted
        && value.future_independent_output_validation_required
        && value.future_snapshot_admission_review_required
        && value.correction_requires_new_candidate
        && value.registered_not_run
        && value.future_independent_implementation_review_required
        && value.next_gate == NEXT_GATE
        && value.authority_boundary == closed_authority_boundary();
    valid
        .then_some(())
        .ok_or_else(|| "Stage 134 快照物化实现合同偏离固定完整性、来源或零能力边界".to_string())
}

fn validate_record(
    value: &OpeningPortfolioSnapshotMaterializationImplementationRegistration,
) -> Result<(), String> {
    validate_contract(&value.implementation_contract)?;
    let downstream_closed = !value.independent_implementation_review_completed
        && !value.receipt_decrypted_or_read
        && !value.financial_rows_parsed
        && !value.output_candidate_created
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
        && value.upstream_stage_133_validation.source_artifact_receipt_independently_validated
        && value
            .upstream_stage_133_validation
            .future_stage_134_snapshot_materialization_implementation_registration_eligible
        && value.implementation_contract.stage_133_validation_id
            == value.upstream_stage_133_validation.validation_id
        && value.implementation_contract.stage_133_validation_sha256
            == value.upstream_stage_133_validation.validation_sha256
        && value.implementation_contract.receipt_id
            == value.upstream_stage_133_validation.receipt_id
        && value.implementation_contract.receipt_manifest_sha256
            == value.upstream_stage_133_validation.receipt_manifest_sha256
        && value.implementation_contract.stage_125_specification_sha256
            == value.upstream_stage_133_validation.stage_125_specification_sha256
        && value.registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain
        && !value.registered_by.trim().is_empty()
        && sorted_unique(&value.excluded_prior_actor_ids)
        && value
            .excluded_prior_actor_ids
            .contains(&value.upstream_stage_133_validation.validated_by)
        && !value.excluded_prior_actor_ids.contains(&value.registered_by)
        && !value.implementation_name.trim().is_empty()
        && !value.implementation_description.trim().is_empty()
        && !value.deterministic_parser_and_adapter_semantics.trim().is_empty()
        && !value.account_scope_and_completeness_semantics.trim().is_empty()
        && !value.exact_decimal_and_signed_quantity_semantics.trim().is_empty()
        && !value
            .instrument_identity_and_corporate_action_semantics
            .trim()
            .is_empty()
        && !value.row_provenance_and_redaction_semantics.trim().is_empty()
        && !value
            .whole_snapshot_failure_and_correction_semantics
            .trim()
            .is_empty()
        && !value.known_limitations.trim().is_empty()
        && !value.future_review_constraints.trim().is_empty()
        && value.status == STATUS
        && value.confirmations_complete
        && confirmation_values_complete(&value.confirmations)
        && value.zero_capability_implementation_contract_registered
        && value.future_stage_135_independent_implementation_review_eligible
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 134 快照物化实现登记无效、漂移或越权".to_string())
}

fn record_matches_source(
    record: &OpeningPortfolioSnapshotMaterializationImplementationRegistration,
    source: &IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt,
) -> bool {
    validate_record(record).is_ok()
        && record.upstream_stage_133_validation == source.validation
        && record.implementation_contract.stage_132_result_sha256 == source.result.result_sha256
        && record.implementation_contract.stage_131_claim_sha256 == source.claim.claim_sha256
        && record.implementation_contract.receipt_manifest_sha256
            == source.receipt.receipt_manifest_sha256
        && record
            .implementation_contract
            .exact_source_artifact_contract
            == exact_stage_125_specification(source).source_contract
        && record
            .implementation_contract
            .exact_canonical_snapshot_schema
            == exact_stage_125_specification(source).canonical_snapshot_schema
}

fn candidate_from_source(
    source: &IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationCandidate, String> {
    let mut excluded = excluded_actor_ids(source);
    excluded.sort();
    excluded.dedup();
    Ok(
        OpeningPortfolioSnapshotMaterializationImplementationCandidate {
            stage_131_attempt_id: source.claim.attempt_id.clone(),
            stage_133_validation_id: source.validation.validation_id.clone(),
            stage_133_validation_sha256: source.validation.validation_sha256.clone(),
            stage_132_result_sha256: source.result.result_sha256.clone(),
            stage_131_claim_sha256: source.claim.claim_sha256.clone(),
            receipt_id: source.receipt.receipt_id.clone(),
            receipt_manifest_sha256: source.receipt.receipt_manifest_sha256.clone(),
            stage_125_specification_sha256: source
                .validation
                .stage_125_specification_sha256
                .clone(),
            source_provider_name: source.receipt.source_provider_name.clone(),
            portfolio_scope_alias: source.receipt.portfolio_scope_alias.clone(),
            artifact_count: source.receipt.artifacts.len(),
            registrar_excluded_actor_ids: excluded,
        },
    )
}

fn exact_stage_125_specification(
    source: &IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt,
) -> OpeningPortfolioSnapshotGovernanceSpecification {
    source
        .claim
        .authorization
        .receiver
        .implementation
        .implementation_contract
        .exact_stage_125_specification
        .clone()
}

fn excluded_actor_ids(
    source: &IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt,
) -> Vec<String> {
    let mut actors = source.validation.excluded_prior_actor_ids.clone();
    actors.extend([
        source.validation.validated_by.clone(),
        source.validation.stage_132_executor_id.clone(),
        source.validation.stage_131_claimant_id.clone(),
        source.claim.claimed_by.clone(),
        source.result.executed_by.clone(),
    ]);
    actors.sort();
    actors.dedup();
    actors
}

fn validate_expected_binding(
    source: &IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt,
    request: &RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest,
) -> Result<(), String> {
    let specification = exact_stage_125_specification(source);
    let valid = request.expected_stage_133_validation_id == source.validation.validation_id
        && request.expected_stage_133_validation_sha256 == source.validation.validation_sha256
        && request.expected_stage_132_result_sha256 == source.result.result_sha256
        && request.expected_stage_131_claim_sha256 == source.claim.claim_sha256
        && request.expected_receipt_id == source.receipt.receipt_id
        && request.expected_receipt_manifest_sha256 == source.receipt.receipt_manifest_sha256
        && request.expected_stage_125_specification_sha256 == specification.specification_sha256
        && source.validation.stage_125_specification_sha256 == specification.specification_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 134 请求与当前 Stage 125/131/132/133 精确绑定不一致".to_string())
}

fn validate_request(
    validation_id: &str,
    request: &RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest,
) -> Result<(), String> {
    let valid = valid_id(validation_id)
        && request.expected_stage_133_validation_id == validation_id
        && valid_id(&request.expected_stage_133_validation_id)
        && valid_id(&request.expected_receipt_id)
        && [
            &request.expected_stage_133_validation_sha256,
            &request.expected_stage_132_result_sha256,
            &request.expected_stage_131_claim_sha256,
            &request.expected_receipt_manifest_sha256,
            &request.expected_stage_125_specification_sha256,
        ]
        .into_iter()
        .all(|item| valid_sha256(item));
    valid
        .then_some(())
        .ok_or_else(|| "Stage 134 请求 ID 或摘要无效".to_string())
}

fn confirmations_complete(
    value: &RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest,
) -> bool {
    confirmation_values_complete(&confirmations_from_request(value))
}

fn confirmations_from_request(
    value: &RegisterOpeningPortfolioSnapshotMaterializationImplementationRequest,
) -> OpeningPortfolioSnapshotMaterializationImplementationConfirmations {
    OpeningPortfolioSnapshotMaterializationImplementationConfirmations {
        exact_current_stage_51_through_stage_133_binding_confirmed: value
            .exact_current_stage_51_through_stage_133_binding_confirmed,
        registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed: value.registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed,
        validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed: value.validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed,
        exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed: value.exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed,
        future_input_only_independently_validated_content_addressed_receipt_confirmed: value.future_input_only_independently_validated_content_addressed_receipt_confirmed,
        future_decryption_only_inside_isolated_ephemeral_materializer_confirmed: value.future_decryption_only_inside_isolated_ephemeral_materializer_confirmed,
        deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: value.deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed,
        account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed: value.account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed,
        exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: value.exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed,
        instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: value.instrument_identity_precedence_and_corporate_action_reconciliation_confirmed,
        no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed: value.no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed,
        statement_market_values_informational_and_no_nav_or_performance_confirmed: value.statement_market_values_informational_and_no_nav_or_performance_confirmed,
        every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed: value.every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed,
        future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed: value.future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed,
        contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: value.contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed,
        no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: value.no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed,
        future_stage_135_chain_external_independent_implementation_review_required_confirmed: value.future_stage_135_chain_external_independent_implementation_review_required_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: value.no_unconfirmed_hari_or_old_wang_logic_claimed,
    }
}

fn confirmation_values_complete(
    value: &OpeningPortfolioSnapshotMaterializationImplementationConfirmations,
) -> bool {
    value.exact_current_stage_51_through_stage_133_binding_confirmed
        && value.registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed
        && value.validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed
        && value.exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed
        && value.future_input_only_independently_validated_content_addressed_receipt_confirmed
        && value.future_decryption_only_inside_isolated_ephemeral_materializer_confirmed
        && value.deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed
        && value.account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed
        && value.exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed
        && value.instrument_identity_precedence_and_corporate_action_reconciliation_confirmed
        && value.no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed
        && value.statement_market_values_informational_and_no_nav_or_performance_confirmed
        && value.every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed
        && value.future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed
        && value.contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed
        && value.no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && value.future_stage_135_chain_external_independent_implementation_review_required_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn closed_authority_boundary()
-> ZeroCapabilityOpeningPortfolioSnapshotMaterializationAuthorityBoundary {
    ZeroCapabilityOpeningPortfolioSnapshotMaterializationAuthorityBoundary {
        decryption_key_access_allowed: false,
        encrypted_artifact_read_allowed: false,
        plaintext_persistence_allowed: false,
        financial_row_parsing_allowed: false,
        executable_artifact_or_entrypoint_present: false,
        runtime_present: false,
        input_mount_present: false,
        output_candidate_present: false,
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

fn accepted_artifact_formats() -> Vec<String> {
    vec!["csv".to_string(), "json".to_string(), "pdf".to_string()]
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSnapshotMaterializationImplementationRegistration>, String> {
    let mut output = Vec::new();
    let mut directory = match tokio::fs::read_dir(record_directory(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(output),
        Err(error) => return Err(error.to_string()),
    };
    let mut ids = BTreeSet::new();
    let mut validations = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let metadata = tokio::fs::symlink_metadata(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() == 0
            || metadata.len() > MAX_RECORD_FILE_BYTES
        {
            return Err("Stage 134 登记文件缺失、非普通文件或超限".to_string());
        }
        let bytes = tokio::fs::read(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        let record: OpeningPortfolioSnapshotMaterializationImplementationRegistration =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&record)?;
        if entry.file_name().to_string_lossy() != format!("{}.json", record.implementation_id)
            || !ids.insert(record.implementation_id.clone())
            || !validations.insert(record.upstream_stage_133_validation.validation_id.clone())
        {
            return Err("Stage 134 登记路径重复、重放或漂移".to_string());
        }
        output.push(record);
    }
    output.sort_by(|left, right| left.registered_at.cmp(&right.registered_at));
    Ok(output)
}

fn bounded_required(value: &str, maximum: usize, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > maximum {
        return Err(format!("{label}为空或超过 {maximum} 字符"));
    }
    Ok(trimmed.to_string())
}

fn bounded_text(value: &str, label: &str) -> Result<String, String> {
    bounded_required(value, MAX_TEXT_CHARS, label)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 134 fingerprint payload invalid".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(sha256_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

fn sha256_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|window| window[0] < window[1])
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())?;
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn record_directory(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("opening-portfolio-snapshot-materialization-implementation-registrations")
}

struct RegistrationLock(PathBuf);

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, validation_id: &str) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "opening-portfolio-snapshot-materialization-stage-134-{validation_id}.lock"
    ));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 134 登记正在进行".to_string())?;
    Ok(RegistrationLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_134_authority_boundary_is_fully_closed() {
        let boundary = closed_authority_boundary();
        let json = serde_json::to_value(boundary).unwrap();
        assert!(
            json.as_object()
                .unwrap()
                .values()
                .all(|value| value == false)
        );
    }

    #[test]
    fn stage_134_contract_requires_exact_decimal_provenance_and_whole_snapshot_failure() {
        let source = OpeningPortfolioSourceArtifactContract {
            source_kind: super::super::controlled_shadow_opening_portfolio_snapshot_governance_specifications::OpeningPortfolioExternalSourceKind::BrokerOrCustodianMachineExport,
            source_provider_name: "provider".to_string(),
            portfolio_scope_alias: "scope".to_string(),
            reporting_currency: "USD".to_string(),
            source_timezone: "Asia/Shanghai".to_string(),
            snapshot_as_of_utc: Utc::now(),
            expected_account_count: 1,
            accepted_artifact_formats: accepted_artifact_formats(),
            original_bytes_required: true,
            content_sha256_and_byte_length_required: true,
            provider_statement_or_export_identifier_required: true,
            provider_generated_at_or_statement_as_of_required: true,
            hone_received_at_required: true,
            source_account_identifiers_must_be_pseudonymized: true,
            raw_account_numbers_or_credentials_allowed: false,
            manual_balance_or_position_entry_allowed: false,
            mutable_or_overwritable_artifact_allowed: false,
        };
        assert_eq!(source.accepted_artifact_formats, vec!["csv", "json", "pdf"]);
        assert!(source.source_account_identifiers_must_be_pseudonymized);
        assert!(!source.manual_balance_or_position_entry_allowed);
        assert!(!closed_authority_boundary().financial_row_parsing_allowed);
    }

    #[test]
    fn stage_134_next_gate_is_review_not_execution() {
        assert!(NEXT_GATE.contains("stage_135"));
        assert!(NEXT_GATE.contains("independent_review"));
        assert!(!NEXT_GATE.contains("execute"));
        assert!(!NEXT_GATE.contains("admission"));
    }

    #[test]
    fn stage_134_limits_are_bounded_and_do_not_imply_financial_authority() {
        assert_eq!(FUTURE_MAX_INPUT_ARTIFACTS, 64);
        assert_eq!(FUTURE_MAX_INPUT_BYTES, 256 * 1024 * 1024);
        assert_eq!(FUTURE_MAX_OUTPUT_BYTES, 64 * 1024 * 1024);
        assert_eq!(FUTURE_MAX_OUTPUT_ROWS, 1_000_000);
        assert!(!closed_authority_boundary().opening_portfolio_snapshot_materialized);
        assert!(!closed_authority_boundary().trading_allowed);
    }

    #[test]
    fn stage_134_confirmations_cover_materialization_failure_modes() {
        let names = serde_json::to_value(OpeningPortfolioSnapshotMaterializationImplementationConfirmations {
            exact_current_stage_51_through_stage_133_binding_confirmed: true,
            registrar_independent_from_stage_133_validator_executor_claimant_and_complete_prior_chain_confirmed: true,
            validation_receipt_claim_result_and_specification_hashes_recomputed_confirmed: true,
            exact_stage_125_source_contract_and_canonical_snapshot_schema_preserved_confirmed: true,
            future_input_only_independently_validated_content_addressed_receipt_confirmed: true,
            future_decryption_only_inside_isolated_ephemeral_materializer_confirmed: true,
            deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: true,
            account_cash_position_option_liability_and_unsettled_activity_completeness_confirmed: true,
            exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: true,
            instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: true,
            no_default_manual_balance_or_inference_and_unsupported_asset_fails_whole_snapshot_confirmed: true,
            statement_market_values_informational_and_no_nav_or_performance_confirmed: true,
            every_output_row_bound_to_artifact_hash_and_source_locator_without_raw_account_or_secret_confirmed: true,
            future_output_create_once_untrusted_canonical_candidate_and_independent_validation_required_confirmed: true,
            contract_only_no_decrypt_read_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: true,
            no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
            future_stage_135_chain_external_independent_implementation_review_required_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }).unwrap();
        assert_eq!(names.as_object().unwrap().len(), 18);
        assert!(
            names
                .as_object()
                .unwrap()
                .values()
                .all(|value| value == true)
        );
    }
}
