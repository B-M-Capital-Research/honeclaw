//! Stage 125 create-once governance specification for an externally sourced opening portfolio.
//!
//! The specification defines what a future source artifact and canonical opening snapshot must
//! prove. It does not ingest an artifact, create a snapshot, post a ledger event, write cash or
//! positions, calculate NAV/performance, train a model, generate an order, access a broker, or
//! trade. Registration only opens Stage 126 independent specification review.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_ledger_transition_candidate_admission_reviews::{
    AdmittedNonFinancialObservationEvidenceForStage125,
    admitted_non_financial_observation_evidence_for_stage_125,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-registration-v1";
const SPECIFICATION_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-opening-portfolio-snapshot-governance-specification-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-external-source-opening-portfolio-snapshot-governance-specification-create-once-v1-zero-financial-state";
const PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-opening-portfolio-source-and-completeness-contract-v1";
const STATUS: &str = "registered_waiting_stage_126_independent_specification_review";
const NEXT_GATE: &str =
    "stage_126_opening_portfolio_snapshot_governance_specification_independent_review";
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_EXPECTED_ACCOUNT_COUNT: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioExternalSourceKind {
    BrokerOrCustodianMachineExport,
    BrokerOrCustodianStatement,
    VerifiedPortfolioAccountingSystemExport,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest {
    expected_stage_124_review_id: String,
    expected_stage_124_review_sha256: String,
    expected_stage_123_validation_sha256: String,
    expected_stage_122_candidate_sha256: String,
    expected_stage_114_review_sha256: String,
    expected_stage_112_output_sha256: String,
    source_kind: OpeningPortfolioExternalSourceKind,
    source_provider_name: String,
    portfolio_scope_alias: String,
    reporting_currency: String,
    source_timezone: String,
    snapshot_as_of_utc: DateTime<Utc>,
    expected_account_count: usize,
    registration_reason: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_current_stage_51_through_stage_124_binding_confirmed: bool,
    registrar_independent_from_stage_124_reviewer_and_complete_prior_chain_confirmed: bool,
    stage_124_admission_reopened_rehashed_and_current_confirmed: bool,
    external_source_artifact_required_and_manual_balances_forbidden_confirmed: bool,
    account_scope_complete_and_opaque_alias_contains_no_account_number_confirmed: bool,
    all_cash_positions_liabilities_and_unsettled_activity_required_confirmed: bool,
    exact_decimal_signed_quantities_and_no_default_or_inference_confirmed: bool,
    instrument_identity_and_corporate_action_reconciliation_required_confirmed: bool,
    statement_market_values_are_informational_not_accounting_marks_confirmed: bool,
    complete_independent_marks_fx_and_derivative_valuation_required_before_nav_confirmed: bool,
    source_artifact_receipt_validation_and_snapshot_admission_are_separate_future_gates_confirmed:
        bool,
    specification_only_no_artifact_upload_read_parse_or_snapshot_materialization_confirmed: bool,
    no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    future_stage_126_independent_specification_review_required_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactContract {
    pub source_kind: OpeningPortfolioExternalSourceKind,
    pub source_provider_name: String,
    pub portfolio_scope_alias: String,
    pub reporting_currency: String,
    pub source_timezone: String,
    pub snapshot_as_of_utc: DateTime<Utc>,
    pub expected_account_count: usize,
    pub accepted_artifact_formats: Vec<String>,
    pub original_bytes_required: bool,
    pub content_sha256_and_byte_length_required: bool,
    pub provider_statement_or_export_identifier_required: bool,
    pub provider_generated_at_or_statement_as_of_required: bool,
    pub hone_received_at_required: bool,
    pub source_account_identifiers_must_be_pseudonymized: bool,
    pub raw_account_numbers_or_credentials_allowed: bool,
    pub manual_balance_or_position_entry_allowed: bool,
    pub mutable_or_overwritable_artifact_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioCanonicalSnapshotSchema {
    pub account_schema: String,
    pub cash_schema: String,
    pub position_schema: String,
    pub listed_option_extension_schema: String,
    pub liability_schema: String,
    pub unsettled_activity_schema: String,
    pub instrument_identity_precedence: Vec<String>,
    pub supported_asset_classes: Vec<String>,
    pub unsupported_asset_class_result: String,
    pub exact_decimal_rule: String,
    pub signed_quantity_rule: String,
    pub duplicate_instrument_rule: String,
    pub cost_basis_rule: String,
    pub statement_market_value_rule: String,
    pub account_scope_completeness_rule: String,
    pub cash_completeness_rule: String,
    pub liabilities_and_unsettled_activity_rule: String,
    pub corporate_action_reconciliation_rule: String,
    pub missing_or_ambiguous_field_rule: String,
    pub opening_nav_rule: String,
    pub performance_inception_rule: String,
    pub correction_rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityOpeningPortfolioAuthorityBoundary {
    pub source_artifact_present: bool,
    pub source_artifact_uploaded_or_read: bool,
    pub parser_or_implementation_present: bool,
    pub executable_artifact_or_entrypoint_present: bool,
    pub runtime_present: bool,
    pub opening_portfolio_snapshot_materialized: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub ledger_event_write_allowed: bool,
    pub position_write_allowed: bool,
    pub cash_write_allowed: bool,
    pub nav_or_performance_write_allowed: bool,
    pub model_or_metric_store_write_allowed: bool,
    pub training_or_rl_feedback_allowed: bool,
    pub reward_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub protocol_version: String,
    pub stage_124_review_id: String,
    pub stage_124_review_sha256: String,
    pub stage_123_validation_sha256: String,
    pub stage_122_candidate_sha256: String,
    pub stage_114_review_sha256: String,
    pub stage_112_output_sha256: String,
    pub source_contract: OpeningPortfolioSourceArtifactContract,
    pub canonical_snapshot_schema: OpeningPortfolioCanonicalSnapshotSchema,
    pub future_source_artifact_receipt_validation_required: bool,
    pub future_canonical_snapshot_materialization_required: bool,
    pub future_independent_snapshot_output_validation_required: bool,
    pub future_opening_snapshot_admission_review_required: bool,
    pub create_once_required: bool,
    pub append_only_corrections_required: bool,
    pub overwrite_allowed: bool,
    pub default_notional_allowed: bool,
    pub infer_cash_positions_quantities_cost_basis_or_weights_allowed: bool,
    pub financial_postings_currently_eligible: bool,
    pub nav_or_performance_currently_eligible: bool,
    pub future_independent_specification_review_required: bool,
    pub authority_boundary: ZeroCapabilityOpeningPortfolioAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationRegistration {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub stage_124_review_id: String,
    pub stage_124_review_sha256: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_124_reviewer_and_complete_prior_chain: bool,
    pub registration_reason: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub specification: OpeningPortfolioSnapshotGovernanceSpecification,
    pub status: String,
    pub confirmations_complete: bool,
    pub specification_registered: bool,
    pub future_stage_126_independent_specification_review_eligible: bool,
    pub specification_review_completed: bool,
    pub source_artifact_receipt_eligible: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_created: bool,
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
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationCandidate {
    pub stage_124_review_id: String,
    pub stage_124_review_sha256: String,
    pub stage_123_validation_sha256: String,
    pub stage_122_candidate_sha256: String,
    pub stage_114_review_sha256: String,
    pub stage_112_output_sha256: String,
    pub formal_non_financial_observation_notice_count: usize,
    pub registrar_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_endpoint_available: bool,
    pub candidates: Vec<OpeningPortfolioSnapshotGovernanceSpecificationCandidate>,
    pub registrations: Vec<OpeningPortfolioSnapshotGovernanceSpecificationRegistration>,
    pub stage_124_admitted_evidence_count: usize,
    pub registration_eligible_count: usize,
    pub registered_specification_count: usize,
    pub future_stage_126_independent_specification_review_eligible_count: usize,
    pub registration_status: String,
    pub next_gate: String,
    pub source_artifact_present: bool,
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
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSnapshotGovernanceSpecificationReadinessSummary {
    pub stage_124_admitted_evidence_count: usize,
    pub registration_eligible_count: usize,
    pub registered_specification_count: usize,
    pub future_stage_126_independent_specification_review_eligible_count: usize,
    pub registration_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification {
    pub source: AdmittedNonFinancialObservationEvidenceForStage125,
    pub registration: OpeningPortfolioSnapshotGovernanceSpecificationRegistration,
}

pub(crate) async fn handle_get_opening_portfolio_snapshot_governance_specifications(
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
            warn!(%error, "Stage 125 opening portfolio governance specification registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "期初组合快照治理规格暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_opening_portfolio_snapshot_governance_specification(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(stage_124_review_id): AxumPath<String>,
    Json(request): Json<RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &stage_124_review_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn opening_portfolio_snapshot_governance_specification_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecificationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        OpeningPortfolioSnapshotGovernanceSpecificationReadinessSummary {
            stage_124_admitted_evidence_count: value.stage_124_admitted_evidence_count,
            registration_eligible_count: value.registration_eligible_count,
            registered_specification_count: value.registered_specification_count,
            future_stage_126_independent_specification_review_eligible_count: value
                .future_stage_126_independent_specification_review_eligible_count,
            registration_status: value.registration_status,
        },
    )
}

pub(crate) async fn independently_reviewable_opening_portfolio_snapshot_governance_specifications(
    state: &AppState,
) -> Result<Vec<IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification>, String> {
    let sources = admitted_non_financial_observation_evidence_for_stage_125(state).await?;
    let registrations = read_records(state).await?;
    let mut values = Vec::new();
    for registration in registrations {
        let source = sources
            .iter()
            .find(|value| value.admission_review.review_id == registration.stage_124_review_id)
            .cloned()
            .ok_or_else(|| "Stage 125 登记不再绑定当前 Stage 124 正式证据".to_string())?;
        validate_source_binding(&registration, &source)?;
        values.push(
            IndependentlyReviewableOpeningPortfolioSnapshotGovernanceSpecification {
                source,
                registration,
            },
        );
    }
    Ok(values)
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecificationRegistry, String> {
    let sources = admitted_non_financial_observation_evidence_for_stage_125(state).await?;
    let registrations = read_records(state).await?;
    for registration in &registrations {
        let source = sources
            .iter()
            .find(|value| value.admission_review.review_id == registration.stage_124_review_id)
            .ok_or_else(|| "Stage 125 登记不再绑定当前 Stage 124 正式证据".to_string())?;
        validate_source_binding(registration, source)?;
    }
    let registered = registrations
        .iter()
        .map(|value| value.stage_124_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let candidates = sources
        .iter()
        .map(candidate_from_source)
        .collect::<Vec<_>>();
    let registration_eligible_count = candidates
        .iter()
        .filter(|value| !registered.contains(value.stage_124_review_id.as_str()))
        .count();
    let registered_specification_count = registrations.len();
    let registration_status = if registered_specification_count > 0 {
        "opening_portfolio_governance_specification_registered_waiting_stage_126_independent_review"
    } else if !sources.is_empty() {
        "waiting_stage_125_opening_portfolio_governance_specification_registration"
    } else {
        "waiting_stage_124_admitted_non_financial_observation_evidence"
    };
    Ok(OpeningPortfolioSnapshotGovernanceSpecificationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_endpoint_available: true,
        candidates,
        registrations,
        stage_124_admitted_evidence_count: sources.len(),
        registration_eligible_count,
        registered_specification_count,
        future_stage_126_independent_specification_review_eligible_count:
            registered_specification_count,
        registration_status: registration_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        source_artifact_present: false,
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
        scope: "Stage 125 只登记外部来源期初组合快照的来源、账户范围、字段完整性、精确十进制、证券身份、负债/未结算活动和未来净值前置条件。当前不接收或读取券商文件，不允许手填余额，不物化或准入期初组合，不开放金融事件白名单、账本、持仓、现金、NAV/绩效、训练/RL、订单、券商或交易。登记只进入 Stage 126 责任链外独立规格复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    stage_124_review_id: &str,
    request: RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecificationRegistration, String> {
    validate_request(stage_124_review_id, &request)?;
    let _lock = acquire_lock(state, stage_124_review_id).await?;
    let sources = admitted_non_financial_observation_evidence_for_stage_125(state).await?;
    let source = sources
        .iter()
        .find(|value| value.admission_review.review_id == stage_124_review_id)
        .ok_or_else(|| "未找到当前可登记 Stage 125 规格的 Stage 124 正式证据".to_string())?;
    verify_expected_bindings(source, &request)?;
    if read_records(state)
        .await?
        .iter()
        .any(|value| value.stage_124_review_id == stage_124_review_id)
    {
        return Err("该 Stage 124 正式证据已登记期初组合治理规格".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(source);
    let independent = actor_is_independent(registrar_id, &excluded_prior_actor_ids);
    if !independent
        || !request.registrar_independent_from_stage_124_reviewer_and_complete_prior_chain_confirmed
    {
        return Err(
            "Stage 125 registrar 必须独立于 Stage 124 reviewer 与完整既有责任链".to_string(),
        );
    }
    let registered_at = Utc::now();
    if request.snapshot_as_of_utc > registered_at + Duration::minutes(5) {
        return Err("Stage 125 snapshot_as_of_utc 不得位于未来".to_string());
    }
    let specification = canonical_specification(source, &request)?;
    let mut registration = OpeningPortfolioSnapshotGovernanceSpecificationRegistration {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_id: String::new(),
        registration_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        stage_124_review_id: source.admission_review.review_id.clone(),
        stage_124_review_sha256: source.admission_review.review_sha256.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_124_reviewer_and_complete_prior_chain: independent,
        registration_reason: bounded_required(&request.registration_reason, "登记理由")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_review_constraints: bounded_required(
            &request.future_review_constraints,
            "未来复核约束",
        )?,
        specification,
        status: STATUS.to_string(),
        confirmations_complete: confirmations_complete(&request),
        specification_registered: true,
        future_stage_126_independent_specification_review_eligible: true,
        specification_review_completed: false,
        source_artifact_receipt_eligible: false,
        opening_portfolio_snapshot_admitted: false,
        ledger_created: false,
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
    registration.registration_sha256 =
        fingerprint_without(&registration, &["registration_id", "registration_sha256"])?;
    registration.registration_id = registration.registration_sha256[..32].to_string();
    validate_record(&registration)?;
    write_immutable_json(
        &record_dir(state).join(format!("{}.json", registration.registration_id)),
        &registration,
    )
    .await?;
    Ok(registration)
}

fn canonical_specification(
    source: &AdmittedNonFinancialObservationEvidenceForStage125,
    request: &RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) -> Result<OpeningPortfolioSnapshotGovernanceSpecification, String> {
    let mut specification = OpeningPortfolioSnapshotGovernanceSpecification {
        schema_version: SPECIFICATION_SCHEMA_VERSION.to_string(),
        specification_sha256: String::new(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        stage_124_review_id: source.admission_review.review_id.clone(),
        stage_124_review_sha256: source.admission_review.review_sha256.clone(),
        stage_123_validation_sha256: source.candidate.validation.validation_sha256.clone(),
        stage_122_candidate_sha256: source.candidate.candidate.candidate_sha256.clone(),
        stage_114_review_sha256: source.candidate.validation.stage_114_review_sha256.clone(),
        stage_112_output_sha256: source.candidate.validation.stage_112_output_sha256.clone(),
        source_contract: OpeningPortfolioSourceArtifactContract {
            source_kind: request.source_kind.clone(),
            source_provider_name: bounded_required(
                &request.source_provider_name,
                "外部来源提供方",
            )?,
            portfolio_scope_alias: validated_scope_alias(&request.portfolio_scope_alias)?,
            reporting_currency: validated_currency(&request.reporting_currency)?,
            source_timezone: bounded_required(&request.source_timezone, "来源时区")?,
            snapshot_as_of_utc: request.snapshot_as_of_utc,
            expected_account_count: request.expected_account_count,
            accepted_artifact_formats: vec![
                "original_provider_pdf_statement".to_string(),
                "original_provider_csv_export".to_string(),
                "original_provider_json_export".to_string(),
            ],
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
        canonical_snapshot_schema: canonical_snapshot_schema(),
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
        authority_boundary: closed_authority_boundary(),
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])?;
    validate_specification(&specification)?;
    Ok(specification)
}

fn canonical_snapshot_schema() -> OpeningPortfolioCanonicalSnapshotSchema {
    OpeningPortfolioCanonicalSnapshotSchema {
        account_schema: "portfolio_scope_alias,source_account_alias,account_type,base_currency,scope_complete,account_sha256".to_string(),
        cash_schema: "source_account_alias,currency,settled_cash_decimal,unsettled_cash_decimal,accrued_interest_decimal|null,cash_sha256".to_string(),
        position_schema: "source_account_alias,asset_class,instrument_id_type,instrument_id,symbol|null,exchange_mic|null,currency,signed_quantity_decimal,cost_basis_decimal|null,position_sha256".to_string(),
        listed_option_extension_schema: "occ_symbol,underlying_instrument_id,expiration_date,strike_decimal,call_or_put,contract_multiplier_decimal,signed_contract_quantity_decimal".to_string(),
        liability_schema: "source_account_alias,currency,liability_type,amount_decimal,accrued_amount_decimal|null,liability_sha256".to_string(),
        unsettled_activity_schema: "source_account_alias,activity_type,trade_date,settlement_date,instrument_id|null,signed_quantity_decimal|null,currency,cash_effect_decimal,activity_sha256".to_string(),
        instrument_identity_precedence: vec![
            "provider_permanent_instrument_id".to_string(),
            "figi".to_string(),
            "isin".to_string(),
            "cusip".to_string(),
            "occ_symbol_for_listed_options".to_string(),
            "ticker_plus_exchange_mic_only_if_independently_resolved".to_string(),
        ],
        supported_asset_classes: vec![
            "cash".to_string(),
            "common_equity".to_string(),
            "etf".to_string(),
            "adr".to_string(),
            "listed_option".to_string(),
        ],
        unsupported_asset_class_result:
            "fail_closed_no_partial_opening_snapshot_admission".to_string(),
        exact_decimal_rule:
            "canonical_base10_strings_only_no_binary_float_no_implicit_rounding".to_string(),
        signed_quantity_rule:
            "long_positive_short_or_written_negative_zero_positions_omitted_only_when_source_proves_zero"
                .to_string(),
        duplicate_instrument_rule:
            "aggregate_only_within_same_account_currency_and_exact_instrument_identity_while_retaining_source_rows"
                .to_string(),
        cost_basis_rule:
            "optional_source_fact_never_inferred_and_not_required_for_forward_inception_performance"
                .to_string(),
        statement_market_value_rule:
            "informational_source_fact_only_never_an_accounting_mark_or_opening_NAV_input"
                .to_string(),
        account_scope_completeness_rule:
            "every_account_in_declared_scope_must_be_present_and_reconciled_to_expected_account_count"
                .to_string(),
        cash_completeness_rule:
            "settled_and_unsettled_cash_required_for_every_source_currency_no_default_zero"
                .to_string(),
        liabilities_and_unsettled_activity_rule:
            "margin_loans_accrued_fees_interest_and_pending_trades_required_no_netting_or_omission"
                .to_string(),
        corporate_action_reconciliation_rule:
            "identifiers_and_quantities_must_reflect_source_effective_actions_as_of_snapshot_time_and_conflicts_fail_closed"
                .to_string(),
        missing_or_ambiguous_field_rule:
            "explicit_gap_with_reason_and_entire_snapshot_ineligible_no_fill_inference_or_partial_admission"
                .to_string(),
        opening_nav_rule:
            "no_NAV_until_snapshot_admitted_and_complete_independent_raw_marks_FX_and_derivative_valuation_exist_for_every_nonzero_exposure_and_liability"
                .to_string(),
        performance_inception_rule:
            "performance_starts_at_admitted_snapshot_as_of_and_never_backfills_pre_snapshot_returns"
                .to_string(),
        correction_rule:
            "new_external_artifact_and_new_independent_admission_create_append_only_superseding_snapshot_never_mutate_history"
                .to_string(),
    }
}

fn closed_authority_boundary() -> ZeroCapabilityOpeningPortfolioAuthorityBoundary {
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

fn candidate_from_source(
    source: &AdmittedNonFinancialObservationEvidenceForStage125,
) -> OpeningPortfolioSnapshotGovernanceSpecificationCandidate {
    OpeningPortfolioSnapshotGovernanceSpecificationCandidate {
        stage_124_review_id: source.admission_review.review_id.clone(),
        stage_124_review_sha256: source.admission_review.review_sha256.clone(),
        stage_123_validation_sha256: source.candidate.validation.validation_sha256.clone(),
        stage_122_candidate_sha256: source.candidate.candidate.candidate_sha256.clone(),
        stage_114_review_sha256: source.candidate.validation.stage_114_review_sha256.clone(),
        stage_112_output_sha256: source.candidate.validation.stage_112_output_sha256.clone(),
        formal_non_financial_observation_notice_count: source.candidate.candidate.notices.len(),
        registrar_excluded_actor_ids: excluded_actor_ids(source),
    }
}

fn excluded_actor_ids(source: &AdmittedNonFinancialObservationEvidenceForStage125) -> Vec<String> {
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

fn verify_expected_bindings(
    source: &AdmittedNonFinancialObservationEvidenceForStage125,
    request: &RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) -> Result<(), String> {
    let valid = request.expected_stage_124_review_id == source.admission_review.review_id
        && request.expected_stage_124_review_sha256 == source.admission_review.review_sha256
        && request.expected_stage_123_validation_sha256
            == source.candidate.validation.validation_sha256
        && request.expected_stage_122_candidate_sha256
            == source.candidate.candidate.candidate_sha256
        && request.expected_stage_114_review_sha256
            == source.candidate.validation.stage_114_review_sha256
        && request.expected_stage_112_output_sha256
            == source.candidate.validation.stage_112_output_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 125 请求与当前 Stage 112/114/122–124 精确绑定不一致".to_string())
}

fn validate_source_binding(
    registration: &OpeningPortfolioSnapshotGovernanceSpecificationRegistration,
    source: &AdmittedNonFinancialObservationEvidenceForStage125,
) -> Result<(), String> {
    validate_record(registration)?;
    let expected_excluded_actor_ids = excluded_actor_ids(source);
    let valid = registration.stage_124_review_id == source.admission_review.review_id
        && registration.stage_124_review_sha256 == source.admission_review.review_sha256
        && registration.specification.stage_123_validation_sha256
            == source.candidate.validation.validation_sha256
        && registration.specification.stage_122_candidate_sha256
            == source.candidate.candidate.candidate_sha256
        && registration.specification.stage_114_review_sha256
            == source.candidate.validation.stage_114_review_sha256
        && registration.specification.stage_112_output_sha256
            == source.candidate.validation.stage_112_output_sha256
        && registration.excluded_prior_actor_ids == expected_excluded_actor_ids
        && actor_is_independent(
            &registration.registered_by,
            &registration.excluded_prior_actor_ids,
        );
    valid
        .then_some(())
        .ok_or_else(|| "Stage 125 登记与当前 Stage 124 正式证据绑定漂移".to_string())
}

fn validate_specification(
    value: &OpeningPortfolioSnapshotGovernanceSpecification,
) -> Result<(), String> {
    let source = &value.source_contract;
    let schema = &value.canonical_snapshot_schema;
    let valid = value.schema_version == SPECIFICATION_SCHEMA_VERSION
        && value.protocol_version == PROTOCOL_VERSION
        && valid_sha256(&value.specification_sha256)
        && value.specification_sha256 == fingerprint_without(value, &["specification_sha256"])?
        && valid_id(&value.stage_124_review_id)
        && [
            &value.stage_124_review_sha256,
            &value.stage_123_validation_sha256,
            &value.stage_122_candidate_sha256,
            &value.stage_114_review_sha256,
            &value.stage_112_output_sha256,
        ]
        .into_iter()
        .all(|hash| valid_sha256(hash))
        && !source.source_provider_name.trim().is_empty()
        && validated_scope_alias(&source.portfolio_scope_alias).is_ok()
        && validated_currency(&source.reporting_currency).is_ok()
        && source.source_timezone.parse::<chrono_tz::Tz>().is_ok()
        && (1..=MAX_EXPECTED_ACCOUNT_COUNT).contains(&source.expected_account_count)
        && source.accepted_artifact_formats.len() == 3
        && source.original_bytes_required
        && source.content_sha256_and_byte_length_required
        && source.provider_statement_or_export_identifier_required
        && source.provider_generated_at_or_statement_as_of_required
        && source.hone_received_at_required
        && source.source_account_identifiers_must_be_pseudonymized
        && !source.raw_account_numbers_or_credentials_allowed
        && !source.manual_balance_or_position_entry_allowed
        && !source.mutable_or_overwritable_artifact_allowed
        && schema == &canonical_snapshot_schema()
        && value.future_source_artifact_receipt_validation_required
        && value.future_canonical_snapshot_materialization_required
        && value.future_independent_snapshot_output_validation_required
        && value.future_opening_snapshot_admission_review_required
        && value.create_once_required
        && value.append_only_corrections_required
        && !value.overwrite_allowed
        && !value.default_notional_allowed
        && !value.infer_cash_positions_quantities_cost_basis_or_weights_allowed
        && !value.financial_postings_currently_eligible
        && !value.nav_or_performance_currently_eligible
        && value.future_independent_specification_review_required
        && value.authority_boundary == closed_authority_boundary();
    valid
        .then_some(())
        .ok_or_else(|| "Stage 125 期初组合治理规格偏离固定零能力合同".to_string())
}

fn validate_record(
    value: &OpeningPortfolioSnapshotGovernanceSpecificationRegistration,
) -> Result<(), String> {
    validate_specification(&value.specification)?;
    let downstream_closed = !value.specification_review_completed
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
        && !value.trading_authorized;
    let valid = value.schema_version == RECORD_SCHEMA_VERSION
        && value.policy_version == POLICY_VERSION
        && valid_id(&value.registration_id)
        && valid_sha256(&value.registration_sha256)
        && value.registration_id == value.registration_sha256[..32]
        && value.registration_sha256
            == fingerprint_without(value, &["registration_id", "registration_sha256"])?
        && value.stage_124_review_id == value.specification.stage_124_review_id
        && value.stage_124_review_sha256 == value.specification.stage_124_review_sha256
        && value.registered_at + Duration::minutes(5)
            >= value.specification.source_contract.snapshot_as_of_utc
        && value.registrar_independent_from_stage_124_reviewer_and_complete_prior_chain
        && !value.registered_by.trim().is_empty()
        && sorted_unique(&value.excluded_prior_actor_ids)
        && actor_is_independent(&value.registered_by, &value.excluded_prior_actor_ids)
        && !value.registration_reason.trim().is_empty()
        && !value.known_limitations.trim().is_empty()
        && !value.future_review_constraints.trim().is_empty()
        && value.status == STATUS
        && value.confirmations_complete
        && value.specification_registered
        && value.future_stage_126_independent_specification_review_eligible
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 125 期初组合治理规格登记无效、漂移或越权".to_string())
}

fn validate_request(
    stage_124_review_id: &str,
    request: &RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) -> Result<(), String> {
    let valid = valid_id(stage_124_review_id)
        && request.expected_stage_124_review_id == stage_124_review_id
        && valid_id(&request.expected_stage_124_review_id)
        && [
            &request.expected_stage_124_review_sha256,
            &request.expected_stage_123_validation_sha256,
            &request.expected_stage_122_candidate_sha256,
            &request.expected_stage_114_review_sha256,
            &request.expected_stage_112_output_sha256,
        ]
        .into_iter()
        .all(|hash| valid_sha256(hash))
        && validated_scope_alias(&request.portfolio_scope_alias).is_ok()
        && validated_currency(&request.reporting_currency).is_ok()
        && !request.source_provider_name.trim().is_empty()
        && request.source_timezone.parse::<chrono_tz::Tz>().is_ok()
        && (1..=MAX_EXPECTED_ACCOUNT_COUNT).contains(&request.expected_account_count);
    if !valid {
        return Err("Stage 125 请求绑定、来源或账户范围格式无效".to_string());
    }
    if !confirmations_complete(request) {
        return Err(
            "Stage 125 必须逐项确认外部来源、完整性、估值前置门与全部零权限边界".to_string(),
        );
    }
    Ok(())
}

fn confirmations_complete(
    request: &RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_124_binding_confirmed
        && request
            .registrar_independent_from_stage_124_reviewer_and_complete_prior_chain_confirmed
        && request.stage_124_admission_reopened_rehashed_and_current_confirmed
        && request.external_source_artifact_required_and_manual_balances_forbidden_confirmed
        && request.account_scope_complete_and_opaque_alias_contains_no_account_number_confirmed
        && request.all_cash_positions_liabilities_and_unsettled_activity_required_confirmed
        && request.exact_decimal_signed_quantities_and_no_default_or_inference_confirmed
        && request.instrument_identity_and_corporate_action_reconciliation_required_confirmed
        && request.statement_market_values_are_informational_not_accounting_marks_confirmed
        && request.complete_independent_marks_fx_and_derivative_valuation_required_before_nav_confirmed
        && request.source_artifact_receipt_validation_and_snapshot_admission_are_separate_future_gates_confirmed
        && request.specification_only_no_artifact_upload_read_parse_or_snapshot_materialization_confirmed
        && request.no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && request.future_stage_126_independent_specification_review_required_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validated_scope_alias(value: &str) -> Result<String, String> {
    let value = value.trim();
    let valid = (3..=64).contains(&value.len())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
        && value.bytes().any(|byte| byte.is_ascii_lowercase());
    valid
        .then_some(value.to_string())
        .ok_or_else(|| "portfolio_scope_alias 必须是 3–64 位不含账号的英文小写别名".to_string())
}

fn validated_currency(value: &str) -> Result<String, String> {
    let value = value.trim();
    (value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase()))
        .then_some(value.to_string())
        .ok_or_else(|| "reporting_currency 必须是三位大写币种代码".to_string())
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}为空或超过 {MAX_TEXT_CHARS} 字符"))
    } else {
        Ok(value.to_string())
    }
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 125 指纹对象无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(hash_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn hash_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSnapshotGovernanceSpecificationRegistration>, String> {
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
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let metadata = tokio::fs::symlink_metadata(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() == 0
            || metadata.len() > MAX_FILE_BYTES
        {
            return Err("Stage 125 规格登记文件无效或超限".to_string());
        }
        let record: OpeningPortfolioSnapshotGovernanceSpecificationRegistration =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_record(&record)?;
        values.push(record);
    }
    values.sort_by(|left, right| right.registered_at.cmp(&left.registered_at));
    let mut stage_124_ids = BTreeSet::new();
    let mut registration_ids = BTreeSet::new();
    for value in &values {
        if !stage_124_ids.insert(value.stage_124_review_id.clone())
            || !registration_ids.insert(value.registration_id.clone())
        {
            return Err("Stage 125 规格登记出现重复或分叉".to_string());
        }
    }
    Ok(values)
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("Stage 125 规格登记文件为空或超限".to_string());
    }
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
    decision_root(state).join("opening-portfolio-snapshot-governance-specifications")
}

struct RegistrationLock(PathBuf);

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(
    state: &AppState,
    stage_124_review_id: &str,
) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage125-{stage_124_review_id}.lock"));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 125 规格登记正在进行".to_string())?;
    Ok(RegistrationLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest {
        RegisterOpeningPortfolioSnapshotGovernanceSpecificationRequest {
            expected_stage_124_review_id: "1".repeat(32),
            expected_stage_124_review_sha256: "2".repeat(64),
            expected_stage_123_validation_sha256: "3".repeat(64),
            expected_stage_122_candidate_sha256: "4".repeat(64),
            expected_stage_114_review_sha256: "5".repeat(64),
            expected_stage_112_output_sha256: "6".repeat(64),
            source_kind: OpeningPortfolioExternalSourceKind::BrokerOrCustodianMachineExport,
            source_provider_name: "test broker".to_string(),
            portfolio_scope_alias: "primary_portfolio".to_string(),
            reporting_currency: "USD".to_string(),
            source_timezone: "America/New_York".to_string(),
            snapshot_as_of_utc: Utc::now(),
            expected_account_count: 1,
            registration_reason: "test".to_string(),
            known_limitations: "no artifact yet".to_string(),
            future_review_constraints: "independent review".to_string(),
            exact_current_stage_51_through_stage_124_binding_confirmed: true,
            registrar_independent_from_stage_124_reviewer_and_complete_prior_chain_confirmed: true,
            stage_124_admission_reopened_rehashed_and_current_confirmed: true,
            external_source_artifact_required_and_manual_balances_forbidden_confirmed: true,
            account_scope_complete_and_opaque_alias_contains_no_account_number_confirmed: true,
            all_cash_positions_liabilities_and_unsettled_activity_required_confirmed: true,
            exact_decimal_signed_quantities_and_no_default_or_inference_confirmed: true,
            instrument_identity_and_corporate_action_reconciliation_required_confirmed: true,
            statement_market_values_are_informational_not_accounting_marks_confirmed: true,
            complete_independent_marks_fx_and_derivative_valuation_required_before_nav_confirmed: true,
            source_artifact_receipt_validation_and_snapshot_admission_are_separate_future_gates_confirmed: true,
            specification_only_no_artifact_upload_read_parse_or_snapshot_materialization_confirmed: true,
            no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
            future_stage_126_independent_specification_review_required_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn request_requires_exact_binding_and_all_confirmations() {
        let mut value = request();
        assert!(validate_request(&"1".repeat(32), &value).is_ok());
        value.all_cash_positions_liabilities_and_unsettled_activity_required_confirmed = false;
        assert!(validate_request(&"1".repeat(32), &value).is_err());
    }

    #[test]
    fn account_scope_alias_is_opaque_and_not_a_raw_number() {
        assert!(validated_scope_alias("primary_portfolio").is_ok());
        assert!(validated_scope_alias("123456789").is_err());
        assert!(validated_scope_alias("acct@example.com").is_err());
    }

    #[test]
    fn canonical_schema_requires_full_financial_state_but_creates_none() {
        let schema = canonical_snapshot_schema();
        assert!(schema.cash_completeness_rule.contains("no_default_zero"));
        assert!(
            schema
                .liabilities_and_unsettled_activity_rule
                .contains("margin_loans")
        );
        assert!(
            schema
                .supported_asset_classes
                .contains(&"listed_option".to_string())
        );
        assert!(
            schema
                .opening_nav_rule
                .contains("no_NAV_until_snapshot_admitted")
        );
        assert_eq!(
            closed_authority_boundary().opening_portfolio_snapshot_admitted,
            false
        );
        assert_eq!(closed_authority_boundary().position_write_allowed, false);
    }

    #[test]
    fn statement_values_cannot_become_accounting_marks() {
        let schema = canonical_snapshot_schema();
        assert!(
            schema
                .statement_market_value_rule
                .contains("never_an_accounting_mark")
        );
        assert!(schema.opening_nav_rule.contains("independent_raw_marks_FX"));
    }

    #[test]
    fn registration_only_opens_independent_specification_review() {
        assert!(NEXT_GATE.contains("stage_126"));
        assert!(NEXT_GATE.contains("independent_review"));
        assert!(!NEXT_GATE.contains("materialization"));
        assert!(!NEXT_GATE.contains("trade"));
    }
}
