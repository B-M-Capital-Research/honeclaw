//! Stage 95 create-once registration of a zero-capability market-data parser specification.
//!
//! The specification is bound to one exact Stage 94 independently validated raw receipt. It
//! freezes strict schemas, rejection rules, cross-source reconciliation and synthetic test vectors,
//! but contains no parser implementation, executable artifact, entrypoint, runtime, input mount,
//! network access, observation writer or downstream investment/trading authority.

use std::collections::{BTreeSet, HashMap};
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_validations::{
    IndependentlyValidatedControlledShadowMarketDataReceipt,
    independently_validated_market_data_receipts_for_parser_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-registration-v1";
const SPEC_SCHEMA_VERSION: &str = "hone-controlled-shadow-market-data-parser-specification-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-create-once-v1-zero-capability";
const PARSER_PROTOCOL_VERSION: &str =
    "hone-natural-forward-market-data-parser-v1-specification-only-not-executable";
const STATUS: &str = "registered_not_reviewed_not_implemented_not_run";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowMarketDataParserSpecificationRequest {
    expected_validation_sha256: String,
    expected_receipt_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_adapter_authorization_sha256: String,
    expected_adapter_spec_sha256: String,
    expected_canonical_request_set_sha256: String,
    registration_reason: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_stage_51_through_stage_94_binding_confirmed: bool,
    registrar_independent_from_validator_executor_stage_92_and_complete_prior_chain_confirmed: bool,
    independent_recomputation_of_validation_receipt_claim_and_request_bindings_confirmed: bool,
    explicit_price_dividend_split_and_official_calendar_sources_confirmed: bool,
    strict_utf8_json_html_schema_and_bounded_decimal_rules_confirmed: bool,
    duplicate_out_of_window_missing_and_malformed_rows_fail_closed_confirmed: bool,
    no_forward_fill_interpolation_deduplication_or_unadjusted_fallback_confirmed: bool,
    spy_calendar_sync_and_cross_source_reconciliation_required_confirmed: bool,
    synthetic_vectors_contain_no_market_fact_or_credential_confirmed: bool,
    specification_only_no_parser_code_artifact_entrypoint_or_runtime_confirmed: bool,
    no_raw_payload_read_mount_network_tool_subprocess_or_production_write_confirmed: bool,
    no_calendar_market_row_observation_ledger_position_performance_or_model_metric_created_confirmed:
        bool,
    no_training_feedback_reward_order_broker_or_trading_confirmed: bool,
    future_chain_external_specification_review_required_before_implementation_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyntheticParserExpectedOutcome {
    AcceptCanonicalRows,
    AcceptEmptyEventSet,
    RejectDuplicateDate,
    RejectMalformedDecimal,
    RejectOutOfWindowDate,
    RejectMissingRequiredField,
    RejectMissingOfficialCalendarTable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SyntheticMarketDataParserTestVector {
    pub vector_id: String,
    pub source_kind: String,
    pub input_fixture: String,
    pub input_fixture_sha256: String,
    pub expected_outcome: SyntheticParserExpectedOutcome,
    pub expected_canonical_output: Option<String>,
    pub expected_canonical_output_sha256: Option<String>,
    pub synthetic_only_no_market_truth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityMarketDataParserAuthorityBoundary {
    pub parser_implementation_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub raw_payload_mount_present: bool,
    pub environment_inheritance_allowed: bool,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub calendar_rows_created: bool,
    pub market_rows_created: bool,
    pub observation_write_allowed: bool,
    pub ledger_creation_allowed: bool,
    pub position_write_allowed: bool,
    pub performance_metric_write_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub training_feedback_allowed: bool,
    pub reward_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserSpecification {
    pub schema_version: String,
    pub parser_specification_sha256: String,
    pub parser_protocol_version: String,
    pub stage_94_validation_id: String,
    pub stage_94_validation_sha256: String,
    pub stage_93_attempt_id: String,
    pub stage_93_claim_sha256: String,
    pub stage_93_result_sha256: String,
    pub stage_93_receipt_sha256: String,
    pub stage_92_adapter_authorization_sha256: String,
    pub adapter_spec_sha256: String,
    pub canonical_request_set_sha256: String,
    pub source_contract_revision: String,
    pub external_reference_urls: Vec<String>,
    pub accepted_source_kinds: Vec<String>,
    pub canonical_calendar_row_schema: String,
    pub canonical_price_row_schema: String,
    pub canonical_dividend_row_schema: String,
    pub canonical_split_row_schema: String,
    pub canonical_parse_result_schema: String,
    pub strict_utf8_required: bool,
    pub fmp_top_level_array_required: bool,
    pub nyse_server_rendered_table_required: bool,
    pub iso_date_only_required: bool,
    pub finite_positive_price_required: bool,
    pub non_negative_volume_required: bool,
    pub duplicate_rows_rejected_not_deduplicated: bool,
    pub out_of_window_rows_rejected: bool,
    pub unknown_fields_ignored_but_not_hashed_into_canonical_rows: bool,
    pub empty_price_series_allowed: bool,
    pub empty_dividend_or_split_event_series_allowed: bool,
    pub forward_fill_allowed: bool,
    pub interpolation_allowed: bool,
    pub unadjusted_close_fallback_allowed: bool,
    pub inferred_dividend_or_split_allowed: bool,
    pub spy_required_for_every_official_session: bool,
    pub subject_missing_session_is_explicit_gap_not_synthetic_row: bool,
    pub separate_price_series_never_silently_substituted: bool,
    pub explicit_actions_required_for_action_adjustment_claims: bool,
    pub source_available_at_remains_unverified_until_separate_review: bool,
    pub parser_output_create_once_and_untrusted: bool,
    pub parser_output_independent_validation_required: bool,
    pub synthetic_test_vectors: Vec<SyntheticMarketDataParserTestVector>,
    pub authority_boundary: ZeroCapabilityMarketDataParserAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserSpecificationRegistration {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub stage_94_validation_id: String,
    pub stage_94_validation_sha256: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_validator_and_complete_prior_chain: bool,
    pub registration_reason: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub parser_specification: ControlledShadowMarketDataParserSpecification,
    pub status: String,
    pub confirmations_complete: bool,
    pub parser_specification_registered: bool,
    pub future_chain_external_specification_review_eligible: bool,
    pub specification_review_completed: bool,
    pub parser_implementation_registration_eligible: bool,
    pub parsed_calendar_created: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
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
pub(crate) struct ControlledShadowMarketDataParserSpecificationCandidate {
    pub validation_id: String,
    pub validation_sha256: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub receipt_sha256: String,
    pub adapter_authorization_sha256: String,
    pub adapter_spec_sha256: String,
    pub canonical_request_set_sha256: String,
    pub subject_symbols: Vec<String>,
    pub raw_payload_count: usize,
    pub registrar_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserSpecificationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_endpoint_available: bool,
    pub candidates: Vec<ControlledShadowMarketDataParserSpecificationCandidate>,
    pub registrations: Vec<ControlledShadowMarketDataParserSpecificationRegistration>,
    pub independently_validated_receipt_count: usize,
    pub registration_eligible_count: usize,
    pub parser_specification_registered_count: usize,
    pub future_chain_external_specification_review_eligible_count: usize,
    pub parser_specification_status: String,
    pub parser_implementation_present: bool,
    pub parsed_calendar_created: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowMarketDataParserSpecificationReadinessSummary {
    pub independently_validated_receipt_count: usize,
    pub registration_eligible_count: usize,
    pub parser_specification_registered_count: usize,
    pub future_chain_external_specification_review_eligible_count: usize,
    pub parser_specification_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct RegisteredControlledShadowMarketDataParserSpecification {
    pub source: IndependentlyValidatedControlledShadowMarketDataReceipt,
    pub registration: ControlledShadowMarketDataParserSpecificationRegistration,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_parser_specifications(
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
            warn!(%error, "market-data parser specification registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "行情解析器规格登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_market_data_parser_specification(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(validation_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowMarketDataParserSpecificationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &validation_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(_) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "规格已登记，但刷新登记表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_parser_specification_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserSpecificationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowMarketDataParserSpecificationReadinessSummary {
            independently_validated_receipt_count: registry.independently_validated_receipt_count,
            registration_eligible_count: registry.registration_eligible_count,
            parser_specification_registered_count: registry.parser_specification_registered_count,
            future_chain_external_specification_review_eligible_count: registry
                .future_chain_external_specification_review_eligible_count,
            parser_specification_status: registry.parser_specification_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn registered_market_data_parser_specifications_for_independent_review(
    state: &AppState,
) -> Result<Vec<RegisteredControlledShadowMarketDataParserSpecification>, String> {
    let sources = independently_validated_market_data_receipts_for_parser_review(state).await?;
    let registrations = read_records(state).await?;
    let index = sources
        .into_iter()
        .map(|value| (value.validation.validation_id.clone(), value))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for registration in registrations {
        let source = index
            .get(&registration.stage_94_validation_id)
            .cloned()
            .ok_or_else(|| "Stage 95 登记缺少当前 Stage 94 通过记录".to_string())?;
        validate_source_binding(&registration, &source)?;
        output.push(RegisteredControlledShadowMarketDataParserSpecification {
            source,
            registration,
        });
    }
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserSpecificationRegistry, String> {
    let sources = independently_validated_market_data_receipts_for_parser_review(state).await?;
    let registrations = read_records(state).await?;
    let registration_index = registrations
        .iter()
        .map(|value| (value.stage_94_validation_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let mut candidates = Vec::new();
    for source in &sources {
        if let Some(registration) = registration_index.get(source.validation.validation_id.as_str())
        {
            validate_source_binding(registration, source)?;
        } else {
            candidates.push(candidate(source));
        }
    }
    if registrations.len() + candidates.len() != sources.len() {
        return Err("Stage 95 规格登记与当前 Stage 94 通过链不一致".into());
    }
    let status = if !registrations.is_empty() {
        "market_data_parser_specification_registered_pending_chain_external_review"
    } else if !candidates.is_empty() {
        "waiting_zero_capability_market_data_parser_specification_registration"
    } else {
        "waiting_stage_94_independently_validated_raw_market_data_receipt"
    };
    Ok(ControlledShadowMarketDataParserSpecificationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        registration_endpoint_available: true,
        independently_validated_receipt_count: sources.len(),
        registration_eligible_count: candidates.len(),
        parser_specification_registered_count: registrations.len(),
        future_chain_external_specification_review_eligible_count: registrations.len(),
        candidates,
        registrations,
        parser_specification_status: status.into(),
        parser_implementation_present: false,
        parsed_calendar_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 95 只把一份 Stage 94 独立验证通过的原始收据绑定到零能力解析器规格。规格冻结严格 schema、拒绝规则、SPY/日历同步、显式分红拆股和合成测试向量；没有 parser 代码、工件、入口、runtime、载荷挂载或网络，不解析真实数据、不开始观察，也不开放任何训练或交易权限。".into(),
    })
}

fn candidate(
    source: &IndependentlyValidatedControlledShadowMarketDataReceipt,
) -> ControlledShadowMarketDataParserSpecificationCandidate {
    let mut excluded = source.validation.excluded_prior_actor_ids.clone();
    excluded.push(source.validation.validated_by.clone());
    excluded.sort();
    excluded.dedup();
    let receipt = source
        .attempt
        .result
        .untrusted_raw_market_data_receipt
        .as_ref()
        .expect("Stage 94 passing source always contains receipt");
    ControlledShadowMarketDataParserSpecificationCandidate {
        validation_id: source.validation.validation_id.clone(),
        validation_sha256: source.validation.validation_sha256.clone(),
        attempt_id: source.attempt.claim.attempt_id.clone(),
        claim_sha256: source.attempt.claim.claim_sha256.clone(),
        result_sha256: source.attempt.result.result_sha256.clone(),
        receipt_sha256: receipt.receipt_sha256.clone(),
        adapter_authorization_sha256: source.attempt.claim.adapter_authorization_sha256.clone(),
        adapter_spec_sha256: source.attempt.claim.adapter_spec_sha256.clone(),
        canonical_request_set_sha256: source.attempt.claim.canonical_request_set_sha256.clone(),
        subject_symbols: source.attempt.claim.subject_symbols.clone(),
        raw_payload_count: receipt.raw_payload_count,
        registrar_excluded_actor_ids: excluded,
    }
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    validation_id: &str,
    request: RegisterControlledShadowMarketDataParserSpecificationRequest,
) -> Result<ControlledShadowMarketDataParserSpecificationRegistration, String> {
    validate_request(validation_id, &request)?;
    let _lock = acquire_lock(state, validation_id).await?;
    if read_records(state)
        .await?
        .iter()
        .any(|value| value.stage_94_validation_id == validation_id)
    {
        return Err("该 Stage 94 通过记录已形成不可覆盖的解析器规格登记".into());
    }
    let source = independently_validated_market_data_receipts_for_parser_review(state)
        .await?
        .into_iter()
        .find(|value| value.validation.validation_id == validation_id)
        .ok_or_else(|| "当前没有可登记规格的 Stage 94 独立验证通过记录".to_string())?;
    verify_expected_bindings(&source, &request)?;
    let mut excluded = source.validation.excluded_prior_actor_ids.clone();
    excluded.push(source.validation.validated_by.clone());
    excluded.sort();
    excluded.dedup();
    if registrar_id.trim().is_empty() || excluded.iter().any(|value| value == registrar_id) {
        return Err("Stage 95 登记者不能是 Stage 94 validator 或完整既有责任链任一角色".into());
    }
    let parser_specification = canonical_parser_specification(&source)?;
    let mut registration = ControlledShadowMarketDataParserSpecificationRegistration {
        schema_version: RECORD_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        registration_id: String::new(),
        registration_sha256: String::new(),
        registered_at: Utc::now(),
        registered_by: registrar_id.into(),
        stage_94_validation_id: source.validation.validation_id.clone(),
        stage_94_validation_sha256: source.validation.validation_sha256.clone(),
        excluded_prior_actor_ids: excluded,
        registrar_independent_from_validator_and_complete_prior_chain: true,
        registration_reason: bounded_required(&request.registration_reason, "登记理由")?,
        known_limitations: bounded_required(&request.known_limitations, "已知限制")?,
        future_review_constraints: bounded_required(
            &request.future_review_constraints,
            "未来复核约束",
        )?,
        parser_specification,
        status: STATUS.into(),
        confirmations_complete: confirmations_complete(&request),
        parser_specification_registered: true,
        future_chain_external_specification_review_eligible: true,
        specification_review_completed: false,
        parser_implementation_registration_eligible: false,
        parsed_calendar_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    registration.registration_sha256 =
        fingerprint_without(&registration, &["registration_id", "registration_sha256"])?;
    registration.registration_id = registration.registration_sha256[..32].into();
    validate_record(&registration)?;
    write_immutable_json(
        &record_dir(state).join(format!("{}.json", registration.registration_id)),
        &registration,
    )
    .await?;
    Ok(registration)
}

fn canonical_parser_specification(
    source: &IndependentlyValidatedControlledShadowMarketDataReceipt,
) -> Result<ControlledShadowMarketDataParserSpecification, String> {
    let receipt = source
        .attempt
        .result
        .untrusted_raw_market_data_receipt
        .as_ref()
        .ok_or_else(|| "Stage 95 上游缺少原始收据".to_string())?;
    let mut specification = ControlledShadowMarketDataParserSpecification {
        schema_version: SPEC_SCHEMA_VERSION.into(),
        parser_specification_sha256: String::new(),
        parser_protocol_version: PARSER_PROTOCOL_VERSION.into(),
        stage_94_validation_id: source.validation.validation_id.clone(),
        stage_94_validation_sha256: source.validation.validation_sha256.clone(),
        stage_93_attempt_id: source.attempt.claim.attempt_id.clone(),
        stage_93_claim_sha256: source.attempt.claim.claim_sha256.clone(),
        stage_93_result_sha256: source.attempt.result.result_sha256.clone(),
        stage_93_receipt_sha256: receipt.receipt_sha256.clone(),
        stage_92_adapter_authorization_sha256: source
            .attempt
            .claim
            .adapter_authorization_sha256
            .clone(),
        adapter_spec_sha256: source.attempt.claim.adapter_spec_sha256.clone(),
        canonical_request_set_sha256: source
            .attempt
            .claim
            .canonical_request_set_sha256
            .clone(),
        source_contract_revision: "fmp-stable-explicit-price-dividend-split-plus-nyse-v1".into(),
        external_reference_urls: vec![
            "https://site.financialmodelingprep.com/developer/docs".into(),
            "https://www.nyse.com/markets/hours-calendars".into(),
        ],
        accepted_source_kinds: vec![
            "fmp_dividend_adjusted_price".into(),
            "fmp_dividends".into(),
            "fmp_raw_unadjusted_price".into(),
            "fmp_split_adjusted_price".into(),
            "fmp_splits".into(),
            "nyse_official_us_market_calendar".into(),
        ],
        canonical_calendar_row_schema:
            "date:YYYY-MM-DD,session_kind:regular|early_close,close_time_et:HH:MM".into(),
        canonical_price_row_schema:
            "symbol,date,source_kind,close_decimal,volume_integer|null,request_sha256,row_sha256"
                .into(),
        canonical_dividend_row_schema:
            "symbol,date,dividend_decimal,record_date|null,payment_date|null,declaration_date|null,row_sha256"
                .into(),
        canonical_split_row_schema:
            "symbol,date,numerator_positive_integer,denominator_positive_integer,split_type|null,row_sha256"
                .into(),
        canonical_parse_result_schema:
            "source_receipt_sha256,parser_spec_sha256,calendar_rows,price_rows,dividend_rows,split_rows,explicit_gaps,rejections,output_sha256,untrusted:true"
                .into(),
        strict_utf8_required: true,
        fmp_top_level_array_required: true,
        nyse_server_rendered_table_required: true,
        iso_date_only_required: true,
        finite_positive_price_required: true,
        non_negative_volume_required: true,
        duplicate_rows_rejected_not_deduplicated: true,
        out_of_window_rows_rejected: true,
        unknown_fields_ignored_but_not_hashed_into_canonical_rows: true,
        empty_price_series_allowed: false,
        empty_dividend_or_split_event_series_allowed: true,
        forward_fill_allowed: false,
        interpolation_allowed: false,
        unadjusted_close_fallback_allowed: false,
        inferred_dividend_or_split_allowed: false,
        spy_required_for_every_official_session: true,
        subject_missing_session_is_explicit_gap_not_synthetic_row: true,
        separate_price_series_never_silently_substituted: true,
        explicit_actions_required_for_action_adjustment_claims: true,
        source_available_at_remains_unverified_until_separate_review: true,
        parser_output_create_once_and_untrusted: true,
        parser_output_independent_validation_required: true,
        synthetic_test_vectors: synthetic_test_vectors()?,
        authority_boundary: closed_authority_boundary(),
    };
    specification.parser_specification_sha256 =
        fingerprint_without(&specification, &["parser_specification_sha256"])?;
    validate_parser_specification(&specification)?;
    Ok(specification)
}

fn synthetic_test_vectors() -> Result<Vec<SyntheticMarketDataParserTestVector>, String> {
    let fixtures = [
        (
            "price-valid-two-rows",
            "fmp_split_adjusted_price",
            r#"[{"symbol":"ZZTEST","date":"2099-01-03","close":10.5,"volume":100},{"symbol":"ZZTEST","date":"2099-01-04","close":10.75,"volume":120}]"#,
            SyntheticParserExpectedOutcome::AcceptCanonicalRows,
            Some(
                r#"[{"close_decimal":"10.5","date":"2099-01-03","symbol":"ZZTEST","volume_integer":100},{"close_decimal":"10.75","date":"2099-01-04","symbol":"ZZTEST","volume_integer":120}]"#,
            ),
        ),
        (
            "price-duplicate-date-rejected",
            "fmp_raw_unadjusted_price",
            r#"[{"symbol":"ZZTEST","date":"2099-01-03","close":10.5},{"symbol":"ZZTEST","date":"2099-01-03","close":10.6}]"#,
            SyntheticParserExpectedOutcome::RejectDuplicateDate,
            None,
        ),
        (
            "price-string-decimal-rejected",
            "fmp_dividend_adjusted_price",
            r#"[{"symbol":"ZZTEST","date":"2099-01-03","close":"10.5"}]"#,
            SyntheticParserExpectedOutcome::RejectMalformedDecimal,
            None,
        ),
        (
            "price-out-of-window-rejected",
            "fmp_split_adjusted_price",
            r#"[{"symbol":"ZZTEST","date":"2098-12-31","close":10.5}]"#,
            SyntheticParserExpectedOutcome::RejectOutOfWindowDate,
            None,
        ),
        (
            "dividend-empty-accepted",
            "fmp_dividends",
            "[]",
            SyntheticParserExpectedOutcome::AcceptEmptyEventSet,
            Some("[]"),
        ),
        (
            "split-required-ratio-rejected",
            "fmp_splits",
            r#"[{"symbol":"ZZTEST","date":"2099-01-03"}]"#,
            SyntheticParserExpectedOutcome::RejectMissingRequiredField,
            None,
        ),
        (
            "nyse-synthetic-table-accepted",
            "nyse_official_us_market_calendar",
            r#"<table><thead><tr><th>Date</th><th>Session</th><th>Close ET</th></tr></thead><tbody><tr><td>2099-01-03</td><td>regular</td><td>16:00</td></tr></tbody></table>"#,
            SyntheticParserExpectedOutcome::AcceptCanonicalRows,
            Some(r#"[{"close_time_et":"16:00","date":"2099-01-03","session_kind":"regular"}]"#),
        ),
        (
            "nyse-missing-table-rejected",
            "nyse_official_us_market_calendar",
            "<html><body>synthetic fixture without calendar table</body></html>",
            SyntheticParserExpectedOutcome::RejectMissingOfficialCalendarTable,
            None,
        ),
    ];
    fixtures
        .into_iter()
        .map(|(id, source, input, outcome, output)| {
            Ok(SyntheticMarketDataParserTestVector {
                vector_id: id.into(),
                source_kind: source.into(),
                input_fixture: input.into(),
                input_fixture_sha256: sha256_bytes(input.as_bytes()),
                expected_outcome: outcome,
                expected_canonical_output: output.map(str::to_string),
                expected_canonical_output_sha256: output
                    .map(|value| sha256_bytes(value.as_bytes())),
                synthetic_only_no_market_truth: true,
            })
        })
        .collect()
}

fn closed_authority_boundary() -> ZeroCapabilityMarketDataParserAuthorityBoundary {
    ZeroCapabilityMarketDataParserAuthorityBoundary {
        parser_implementation_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        raw_payload_mount_present: false,
        environment_inheritance_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        calendar_rows_created: false,
        market_rows_created: false,
        observation_write_allowed: false,
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

fn validate_parser_specification(
    value: &ControlledShadowMarketDataParserSpecification,
) -> Result<(), String> {
    let expected_vectors = synthetic_test_vectors()?;
    if value.schema_version != SPEC_SCHEMA_VERSION
        || value.parser_protocol_version != PARSER_PROTOCOL_VERSION
        || !valid_sha256(&value.parser_specification_sha256)
        || value.parser_specification_sha256
            != fingerprint_without(value, &["parser_specification_sha256"])?
        || !valid_id(&value.stage_94_validation_id)
        || ![
            &value.stage_94_validation_sha256,
            &value.stage_93_claim_sha256,
            &value.stage_93_result_sha256,
            &value.stage_93_receipt_sha256,
            &value.stage_92_adapter_authorization_sha256,
            &value.adapter_spec_sha256,
            &value.canonical_request_set_sha256,
        ]
        .iter()
        .all(|hash| valid_sha256(hash))
        || value.source_contract_revision != "fmp-stable-explicit-price-dividend-split-plus-nyse-v1"
        || value.external_reference_urls
            != [
                "https://site.financialmodelingprep.com/developer/docs",
                "https://www.nyse.com/markets/hours-calendars",
            ]
        || value.accepted_source_kinds
            != [
                "fmp_dividend_adjusted_price",
                "fmp_dividends",
                "fmp_raw_unadjusted_price",
                "fmp_split_adjusted_price",
                "fmp_splits",
                "nyse_official_us_market_calendar",
            ]
        || !value.strict_utf8_required
        || !value.fmp_top_level_array_required
        || !value.nyse_server_rendered_table_required
        || !value.iso_date_only_required
        || !value.finite_positive_price_required
        || !value.non_negative_volume_required
        || !value.duplicate_rows_rejected_not_deduplicated
        || !value.out_of_window_rows_rejected
        || !value.unknown_fields_ignored_but_not_hashed_into_canonical_rows
        || value.empty_price_series_allowed
        || !value.empty_dividend_or_split_event_series_allowed
        || value.forward_fill_allowed
        || value.interpolation_allowed
        || value.unadjusted_close_fallback_allowed
        || value.inferred_dividend_or_split_allowed
        || !value.spy_required_for_every_official_session
        || !value.subject_missing_session_is_explicit_gap_not_synthetic_row
        || !value.separate_price_series_never_silently_substituted
        || !value.explicit_actions_required_for_action_adjustment_claims
        || !value.source_available_at_remains_unverified_until_separate_review
        || !value.parser_output_create_once_and_untrusted
        || !value.parser_output_independent_validation_required
        || value.synthetic_test_vectors != expected_vectors
        || value.authority_boundary != closed_authority_boundary()
    {
        return Err("Stage 95 零能力行情解析器规格无效、漂移或越权".into());
    }
    Ok(())
}

fn validate_record(
    value: &ControlledShadowMarketDataParserSpecificationRegistration,
) -> Result<(), String> {
    validate_parser_specification(&value.parser_specification)?;
    let downstream_closed = !value.specification_review_completed
        && !value.parser_implementation_registration_eligible
        && !value.parsed_calendar_created
        && !value.parsed_market_rows_created
        && !value.forward_observation_started
        && !value.ledger_created
        && !value.position_written
        && !value.performance_metric_written
        && !value.model_store_written
        && !value.metric_store_written
        && !value.training_feedback_authorized
        && !value.reward_authorized
        && !value.order_generation_authorized
        && !value.broker_access_authorized
        && !value.trading_authorized;
    if value.schema_version != RECORD_SCHEMA_VERSION
        || value.policy_version != POLICY_VERSION
        || !valid_id(&value.registration_id)
        || !valid_sha256(&value.registration_sha256)
        || value.registration_sha256
            != fingerprint_without(value, &["registration_id", "registration_sha256"])?
        || value.registration_id != value.registration_sha256[..32]
        || value.stage_94_validation_id != value.parser_specification.stage_94_validation_id
        || value.stage_94_validation_sha256 != value.parser_specification.stage_94_validation_sha256
        || value.registered_by.trim().is_empty()
        || value
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &value.registered_by)
        || !sorted_unique(&value.excluded_prior_actor_ids)
        || !value.registrar_independent_from_validator_and_complete_prior_chain
        || value.registration_reason.trim().is_empty()
        || value.known_limitations.trim().is_empty()
        || value.future_review_constraints.trim().is_empty()
        || value.status != STATUS
        || !value.confirmations_complete
        || !value.parser_specification_registered
        || !value.future_chain_external_specification_review_eligible
        || !downstream_closed
    {
        return Err("Stage 95 解析器规格登记记录无效、漂移或越权".into());
    }
    Ok(())
}

fn validate_source_binding(
    registration: &ControlledShadowMarketDataParserSpecificationRegistration,
    source: &IndependentlyValidatedControlledShadowMarketDataReceipt,
) -> Result<(), String> {
    validate_record(registration)?;
    let expected = canonical_parser_specification(source)?;
    if registration.stage_94_validation_id != source.validation.validation_id
        || registration.stage_94_validation_sha256 != source.validation.validation_sha256
        || registration.parser_specification != expected
    {
        return Err("Stage 95 规格登记与当前 Stage 94 通过链绑定漂移".into());
    }
    Ok(())
}

fn verify_expected_bindings(
    source: &IndependentlyValidatedControlledShadowMarketDataReceipt,
    request: &RegisterControlledShadowMarketDataParserSpecificationRequest,
) -> Result<(), String> {
    let receipt = source
        .attempt
        .result
        .untrusted_raw_market_data_receipt
        .as_ref()
        .ok_or_else(|| "Stage 95 候选缺少原始收据".to_string())?;
    if request.expected_validation_sha256 != source.validation.validation_sha256
        || request.expected_receipt_sha256 != receipt.receipt_sha256
        || request.expected_claim_sha256 != source.attempt.claim.claim_sha256
        || request.expected_result_sha256 != source.attempt.result.result_sha256
        || request.expected_adapter_authorization_sha256
            != source.attempt.claim.adapter_authorization_sha256
        || request.expected_adapter_spec_sha256 != source.attempt.claim.adapter_spec_sha256
        || request.expected_canonical_request_set_sha256
            != source.attempt.claim.canonical_request_set_sha256
    {
        return Err("Stage 95 预期 Stage 92–94 绑定已经漂移".into());
    }
    Ok(())
}

fn validate_request(
    validation_id: &str,
    value: &RegisterControlledShadowMarketDataParserSpecificationRequest,
) -> Result<(), String> {
    if !valid_id(validation_id)
        || ![
            &value.expected_validation_sha256,
            &value.expected_receipt_sha256,
            &value.expected_claim_sha256,
            &value.expected_result_sha256,
            &value.expected_adapter_authorization_sha256,
            &value.expected_adapter_spec_sha256,
            &value.expected_canonical_request_set_sha256,
        ]
        .iter()
        .all(|hash| valid_sha256(hash))
        || bounded_required(&value.registration_reason, "登记理由").is_err()
        || bounded_required(&value.known_limitations, "已知限制").is_err()
        || bounded_required(&value.future_review_constraints, "未来复核约束").is_err()
        || !confirmations_complete(value)
    {
        return Err("Stage 95 规格登记请求无效或确认项不完整".into());
    }
    Ok(())
}

fn confirmations_complete(
    value: &RegisterControlledShadowMarketDataParserSpecificationRequest,
) -> bool {
    value.exact_stage_51_through_stage_94_binding_confirmed
        && value
            .registrar_independent_from_validator_executor_stage_92_and_complete_prior_chain_confirmed
        && value.independent_recomputation_of_validation_receipt_claim_and_request_bindings_confirmed
        && value.explicit_price_dividend_split_and_official_calendar_sources_confirmed
        && value.strict_utf8_json_html_schema_and_bounded_decimal_rules_confirmed
        && value.duplicate_out_of_window_missing_and_malformed_rows_fail_closed_confirmed
        && value.no_forward_fill_interpolation_deduplication_or_unadjusted_fallback_confirmed
        && value.spy_calendar_sync_and_cross_source_reconciliation_required_confirmed
        && value.synthetic_vectors_contain_no_market_fact_or_credential_confirmed
        && value.specification_only_no_parser_code_artifact_entrypoint_or_runtime_confirmed
        && value.no_raw_payload_read_mount_network_tool_subprocess_or_production_write_confirmed
        && value
            .no_calendar_market_row_observation_ledger_position_performance_or_model_metric_created_confirmed
        && value.no_training_feedback_reward_order_broker_or_trading_confirmed
        && value.future_chain_external_specification_review_required_before_implementation_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserSpecificationRegistration>, String> {
    let directory = record_dir(state);
    let mut records = Vec::new();
    let mut seen_ids = BTreeSet::new();
    let mut seen_validations = BTreeSet::new();
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(records),
        Err(error) => return Err(format!("读取 Stage 95 规格登记失败: {error}")),
    };
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| format!("遍历 Stage 95 规格登记失败: {error}"))?
    {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|error| format!("读取 Stage 95 规格登记文件失败: {error}"))?;
        let value: ControlledShadowMarketDataParserSpecificationRegistration =
            serde_json::from_slice(&bytes)
                .map_err(|error| format!("解析 Stage 95 规格登记文件失败: {error}"))?;
        validate_record(&value)?;
        if path.file_stem().and_then(|value| value.to_str()) != Some(&value.registration_id)
            || !seen_ids.insert(value.registration_id.clone())
            || !seen_validations.insert(value.stage_94_validation_id.clone())
        {
            return Err("Stage 95 规格登记文件名、ID 或上游 validation 重复".into());
        }
        records.push(value);
    }
    records.sort_by_key(|value| value.registered_at);
    Ok(records)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn record_dir(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-first-natural-forward-cycle-market-data-parser-specifications")
}

fn lock_path(state: &AppState, validation_id: &str) -> PathBuf {
    decision_root(state)
        .join("governance-locks")
        .join(format!("market-data-parser-spec-{validation_id}.lock"))
}

struct LockGuard(PathBuf);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, validation_id: &str) -> Result<LockGuard, String> {
    let path = lock_path(state, validation_id);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| format!("创建 Stage 95 锁目录失败: {error}"))?;
    }
    for attempt in 0..2 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(LockGuard(path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && attempt == 0 => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|value| value.modified().ok())
                    .and_then(|value| SystemTime::now().duration_since(value).ok())
                    .is_some_and(|age| age > StdDuration::from_secs(300));
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                return Err("Stage 95 规格登记正在被另一请求处理".into());
            }
            Err(error) => return Err(format!("创建 Stage 95 规格登记锁失败: {error}")),
        }
    }
    Err("Stage 95 无法取得规格登记锁".into())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Stage 95 规格登记路径无父目录".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| format!("创建 Stage 95 规格目录失败: {error}"))?;
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("序列化 Stage 95 规格登记失败: {error}"))?;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options
        .open(path)
        .await
        .map_err(|error| format!("create-once 写入 Stage 95 规格登记失败: {error}"))?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes)
        .await
        .map_err(|error| format!("写入 Stage 95 规格登记失败: {error}"))?;
    file.sync_all()
        .await
        .map_err(|error| format!("同步 Stage 95 规格登记失败: {error}"))?;
    Ok(())
}

fn fingerprint<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("Stage 95 指纹失败: {error}"))?;
    Ok(sha256_bytes(&bytes))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut json =
        serde_json::to_value(value).map_err(|error| format!("Stage 95 指纹序列化失败: {error}"))?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Stage 95 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    fingerprint(&json)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("Stage 95 {label}为空或超过 {MAX_TEXT_CHARS} 字"));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterControlledShadowMarketDataParserSpecificationRequest {
        RegisterControlledShadowMarketDataParserSpecificationRequest {
            expected_validation_sha256: "a".repeat(64),
            expected_receipt_sha256: "b".repeat(64),
            expected_claim_sha256: "c".repeat(64),
            expected_result_sha256: "d".repeat(64),
            expected_adapter_authorization_sha256: "e".repeat(64),
            expected_adapter_spec_sha256: "f".repeat(64),
            expected_canonical_request_set_sha256: "1".repeat(64),
            registration_reason: "freeze strict parser semantics".into(),
            known_limitations: "synthetic vectors do not prove provider semantics".into(),
            future_review_constraints: "independent review before implementation".into(),
            exact_stage_51_through_stage_94_binding_confirmed: true,
            registrar_independent_from_validator_executor_stage_92_and_complete_prior_chain_confirmed: true,
            independent_recomputation_of_validation_receipt_claim_and_request_bindings_confirmed: true,
            explicit_price_dividend_split_and_official_calendar_sources_confirmed: true,
            strict_utf8_json_html_schema_and_bounded_decimal_rules_confirmed: true,
            duplicate_out_of_window_missing_and_malformed_rows_fail_closed_confirmed: true,
            no_forward_fill_interpolation_deduplication_or_unadjusted_fallback_confirmed: true,
            spy_calendar_sync_and_cross_source_reconciliation_required_confirmed: true,
            synthetic_vectors_contain_no_market_fact_or_credential_confirmed: true,
            specification_only_no_parser_code_artifact_entrypoint_or_runtime_confirmed: true,
            no_raw_payload_read_mount_network_tool_subprocess_or_production_write_confirmed: true,
            no_calendar_market_row_observation_ledger_position_performance_or_model_metric_created_confirmed: true,
            no_training_feedback_reward_order_broker_or_trading_confirmed: true,
            future_chain_external_specification_review_required_before_implementation_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn synthetic_vectors_are_hashed_and_cover_accept_and_fail_closed_cases() {
        let vectors = synthetic_test_vectors().unwrap();
        assert_eq!(vectors.len(), 8);
        assert!(
            vectors
                .iter()
                .all(|value| value.synthetic_only_no_market_truth)
        );
        assert!(vectors.iter().all(|value| {
            value.input_fixture_sha256 == sha256_bytes(value.input_fixture.as_bytes())
        }));
        assert!(vectors.iter().any(|value| {
            value.expected_outcome == SyntheticParserExpectedOutcome::AcceptEmptyEventSet
        }));
        assert!(vectors.iter().any(|value| {
            value.expected_outcome == SyntheticParserExpectedOutcome::RejectDuplicateDate
        }));
        assert!(vectors.iter().any(|value| {
            value.expected_outcome
                == SyntheticParserExpectedOutcome::RejectMissingOfficialCalendarTable
        }));
    }

    #[test]
    fn closed_authority_has_no_parser_or_downstream_capability() {
        let boundary = closed_authority_boundary();
        assert!(!boundary.parser_implementation_present);
        assert!(!boundary.raw_payload_mount_present);
        assert!(!boundary.calendar_rows_created);
        assert!(!boundary.market_rows_created);
        assert!(!boundary.observation_write_allowed);
        assert!(!boundary.training_feedback_allowed);
        assert!(!boundary.order_generation_allowed);
        assert!(!boundary.trading_allowed);
    }

    #[test]
    fn registration_request_requires_every_boundary_confirmation() {
        let mut value = request();
        assert!(validate_request(&"a".repeat(32), &value).is_ok());
        value.no_forward_fill_interpolation_deduplication_or_unadjusted_fallback_confirmed = false;
        assert!(validate_request(&"a".repeat(32), &value).is_err());
        value.no_forward_fill_interpolation_deduplication_or_unadjusted_fallback_confirmed = true;
        assert!(validate_request("not-an-id", &value).is_err());
    }
}
