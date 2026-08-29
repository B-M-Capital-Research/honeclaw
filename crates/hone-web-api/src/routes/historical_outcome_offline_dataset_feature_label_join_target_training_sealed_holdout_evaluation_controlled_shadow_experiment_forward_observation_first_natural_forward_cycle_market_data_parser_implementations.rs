//! Stage 97 create-once registration of a zero-capability market-data parser implementation
//! contract for one exact Stage 96 independently approved parser specification.
//!
//! This stage freezes deterministic implementation semantics only. It has no source artifact,
//! executable artifact, callable entrypoint, runtime, raw-payload mount/read, parsed output,
//! observation writer, ledger, position, metric, training, reward, order, broker or trading
//! capability.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_specification_reviews::{
    ControlledShadowMarketDataParserSpecificationReview,
    ControlledShadowMarketDataParserSpecificationReviewVerdict,
    IndependentlyApprovedControlledShadowMarketDataParserSpecification,
    ZeroCapabilityMarketDataParserSpecificationReviewAuthorityBoundary,
    independently_approved_market_data_parser_specifications_for_implementation_registration,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_specifications::{
    ControlledShadowMarketDataParserSpecification,
    ControlledShadowMarketDataParserSpecificationRegistration,
    ZeroCapabilityMarketDataParserAuthorityBoundary,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-implementation-registry-v1";
const RECORD_SCHEMA_VERSION: &str = "hone-controlled-shadow-market-data-parser-implementation-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-zero-capability-market-data-parser-implementation-contract-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-implementation-create-once-v1-zero-capability";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-natural-forward-market-data-parser-implementation-v1-not-executable";
const STAGE_95_RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-registration-v1";
const STAGE_95_POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-create-once-v1-zero-capability";
const STAGE_95_SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-v1";
const STAGE_96_REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-review-v1";
const STAGE_96_REVIEW_POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-chain-external-review-v1-no-parser";
const STATUS: &str = "registered_not_reviewed_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowMarketDataParserImplementationRequest {
    expected_specification_review_id: String,
    expected_specification_review_sha256: String,
    expected_registration_id: String,
    expected_registration_sha256: String,
    expected_parser_specification_sha256: String,
    expected_validation_sha256: String,
    expected_receipt_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_adapter_authorization_sha256: String,
    expected_adapter_spec_sha256: String,
    expected_canonical_request_set_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_description: String,
    deterministic_parser_semantics: String,
    source_schema_and_numeric_semantics: String,
    calendar_action_and_reconciliation_semantics: String,
    error_and_missing_data_semantics: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_stage_51_through_stage_96_binding_confirmed: bool,
    registrar_independent_from_stage_96_and_complete_prior_chain_confirmed: bool,
    independent_recomputation_of_review_registration_and_specification_confirmed: bool,
    zero_capability_contract_only_no_source_or_executable_artifact_confirmed: bool,
    fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed: bool,
    strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: bool,
    duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: bool,
    no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
        bool,
    spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed:
        bool,
    all_eight_synthetic_vector_hashes_bound_confirmed: bool,
    source_available_at_remains_unverified_until_separate_review_confirmed: bool,
    future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed:
        bool,
    no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    future_independent_implementation_review_required_before_isolated_runner_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityMarketDataParserImplementationAuthorityBoundary {
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub raw_payload_mount_present: bool,
    pub raw_payload_read_allowed: bool,
    pub environment_inheritance_allowed: bool,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub calendar_rows_created: bool,
    pub market_rows_created: bool,
    pub parser_output_written: bool,
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
pub(crate) struct ControlledShadowMarketDataParserImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_protocol_version: String,
    pub immutable_code_revision: String,
    pub stage_96_specification_review_id: String,
    pub stage_96_specification_review_sha256: String,
    pub stage_95_registration_id: String,
    pub stage_95_registration_sha256: String,
    pub parser_specification_sha256: String,
    pub validation_sha256: String,
    pub receipt_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub adapter_authorization_sha256: String,
    pub adapter_spec_sha256: String,
    pub canonical_request_set_sha256: String,
    pub exact_parser_specification: ControlledShadowMarketDataParserSpecification,
    pub strict_envelope_dispatch_function_id: String,
    pub fmp_price_array_parser_function_id: String,
    pub fmp_dividend_event_parser_function_id: String,
    pub fmp_split_event_parser_function_id: String,
    pub nyse_calendar_table_parser_function_id: String,
    pub calendar_subject_spy_reconciliation_function_id: String,
    pub canonical_row_serialization_and_hash_function_id: String,
    pub synthetic_vector_conformance_function_id: String,
    pub canonical_calendar_row_schema: String,
    pub canonical_price_row_schema: String,
    pub canonical_dividend_row_schema: String,
    pub canonical_split_row_schema: String,
    pub canonical_parse_result_schema: String,
    pub future_raw_payload_input_read_only_and_content_addressed: bool,
    pub future_parser_output_create_once_and_untrusted: bool,
    pub future_parser_output_independent_validation_required: bool,
    pub future_parser_output_contains_market_interpretation: bool,
    pub future_parser_output_contains_order_intent: bool,
    pub registered_not_run: bool,
    pub independent_implementation_review_required: bool,
    pub isolated_runner_registration_required_after_review: bool,
    pub authority_boundary: ZeroCapabilityMarketDataParserImplementationAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub upstream_specification_registration:
        ControlledShadowMarketDataParserSpecificationRegistration,
    pub upstream_specification_review: ControlledShadowMarketDataParserSpecificationReview,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_96_and_complete_prior_chain: bool,
    pub implementation_name: String,
    pub implementation_description: String,
    pub deterministic_parser_semantics: String,
    pub source_schema_and_numeric_semantics: String,
    pub calendar_action_and_reconciliation_semantics: String,
    pub error_and_missing_data_semantics: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub implementation_contract: ControlledShadowMarketDataParserImplementationContract,
    pub status: String,
    pub confirmations_complete: bool,
    pub exact_stage_51_through_stage_96_binding_confirmed: bool,
    pub registrar_independent_from_stage_96_and_complete_prior_chain_confirmed: bool,
    pub independent_recomputation_of_review_registration_and_specification_confirmed: bool,
    pub zero_capability_contract_only_no_source_or_executable_artifact_confirmed: bool,
    pub fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed: bool,
    pub strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: bool,
    pub duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: bool,
    pub no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
        bool,
    pub spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed:
        bool,
    pub all_eight_synthetic_vector_hashes_bound_confirmed: bool,
    pub source_available_at_remains_unverified_until_separate_review_confirmed: bool,
    pub future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    pub no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed:
        bool,
    pub no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub future_independent_implementation_review_required_before_isolated_runner_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_parser_implementation_contract_registered: bool,
    pub parser_implementation_present: bool,
    pub future_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub isolated_runner_registration_eligible: bool,
    pub parsed_calendar_rows_created: bool,
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
pub(crate) struct ControlledShadowMarketDataParserImplementationItem {
    pub specification_review: ControlledShadowMarketDataParserSpecificationReview,
    pub specification_registration: ControlledShadowMarketDataParserSpecificationRegistration,
    pub subject_symbols: Vec<String>,
    pub raw_payload_count: usize,
    pub implementation: Option<ControlledShadowMarketDataParserImplementationRecord>,
    pub registration_eligible: bool,
    pub upstream_binding_current: bool,
    pub future_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_endpoint_available: bool,
    pub items: Vec<ControlledShadowMarketDataParserImplementationItem>,
    pub independently_approved_specification_count: usize,
    pub registration_eligible_count: usize,
    pub implementation_contract_count: usize,
    pub current_binding_implementation_contract_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub raw_payload_accessed: bool,
    pub parsed_calendar_rows_created: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowMarketDataParserImplementationReadinessSummary {
    pub independently_approved_specification_count: usize,
    pub registration_eligible_count: usize,
    pub implementation_contract_count: usize,
    pub current_binding_implementation_contract_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_parser_implementations(
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
            warn!(%error, "market-data parser implementation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "行情解析器零能力实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_market_data_parser_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(specification_review_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowMarketDataParserImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &specification_review_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_parser_implementation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserImplementationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowMarketDataParserImplementationReadinessSummary {
            independently_approved_specification_count: value
                .independently_approved_specification_count,
            registration_eligible_count: value.registration_eligible_count,
            implementation_contract_count: value.implementation_contract_count,
            current_binding_implementation_contract_count: value
                .current_binding_implementation_contract_count,
            independent_implementation_review_eligible_count: value
                .independent_implementation_review_eligible_count,
            implementation_status: value.implementation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_reviewable_market_data_parser_implementations(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserImplementationRecord>, String> {
    let registry = build_registry(state).await?;
    let records = registry
        .items
        .into_iter()
        .filter_map(|item| {
            item.implementation.and_then(|implementation| {
                item.future_independent_implementation_review_eligible
                    .then_some(implementation)
            })
        })
        .collect::<Vec<_>>();
    for record in &records {
        validate_record(record)?;
    }
    Ok(records)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserImplementationRegistry, String> {
    let sources =
        independently_approved_market_data_parser_specifications_for_implementation_registration(
            state,
        )
        .await?;
    let records = read_records(state).await?;
    let by_review = records
        .iter()
        .map(|record| {
            (
                record.upstream_specification_review.review_id.as_str(),
                record,
            )
        })
        .collect::<HashMap<_, _>>();
    let mut items = Vec::new();
    for source in sources {
        let implementation = by_review
            .get(source.review.review_id.as_str())
            .map(|value| (*value).clone());
        let upstream_binding_current = implementation
            .as_ref()
            .is_some_and(|record| record_matches_source(record, &source));
        let registration_eligible = implementation.is_none();
        let receipt = source
            .registered
            .source
            .attempt
            .result
            .untrusted_raw_market_data_receipt
            .as_ref()
            .ok_or_else(|| "Stage 97 来源缺少 Stage 93 原始行情收据".to_string())?;
        items.push(ControlledShadowMarketDataParserImplementationItem {
            specification_review: source.review.clone(),
            specification_registration: source.registered.registration.clone(),
            subject_symbols: source
                .registered
                .source
                .attempt
                .claim
                .subject_symbols
                .clone(),
            raw_payload_count: receipt.raw_payload_count,
            implementation,
            registration_eligible,
            upstream_binding_current,
            future_independent_implementation_review_eligible: upstream_binding_current,
        });
    }
    items.sort_by(|left, right| {
        right
            .specification_review
            .reviewed_at
            .cmp(&left.specification_review.reviewed_at)
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
    let independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .count();
    let implementation_status = if current_binding_implementation_contract_count > 0 {
        "zero_capability_market_data_parser_implementation_contract_registered_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "independently_approved_market_data_parser_specification_ready_for_zero_capability_implementation_registration"
    } else {
        "waiting_stage_96_independently_approved_market_data_parser_specification"
    };
    Ok(ControlledShadowMarketDataParserImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_endpoint_available: true,
        items,
        independently_approved_specification_count,
        registration_eligible_count,
        implementation_contract_count,
        current_binding_implementation_contract_count,
        independent_implementation_review_eligible_count,
        implementation_status: implementation_status.to_string(),
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        raw_payload_accessed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 97 只把 Stage 96 独立通过的精确 parser 规格登记为内容寻址、零能力且不可执行的实现合同，冻结确定性函数身份、输出 schema、严格失败关闭和未来独立验证边界。没有源代码/可执行工件、入口、runtime、原始载荷读取、解析结果、观察、账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易能力；下一步只能进入责任链外实现复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    specification_review_id: &str,
    request: RegisterControlledShadowMarketDataParserImplementationRequest,
) -> Result<ControlledShadowMarketDataParserImplementationRecord, String> {
    if !valid_id(specification_review_id) || registrar_id.trim().is_empty() {
        return Err("Stage 97 规格复核 ID 或登记者无效".to_string());
    }
    let _lock = acquire_lock(state, specification_review_id).await?;
    let source =
        independently_approved_market_data_parser_specifications_for_implementation_registration(
            state,
        )
        .await?
        .into_iter()
        .find(|source| source.review.review_id == specification_review_id)
        .ok_or_else(|| "当前没有精确匹配且独立批准的 Stage 96 parser 规格".to_string())?;
    if read_records(state)
        .await?
        .iter()
        .any(|record| record.upstream_specification_review.review_id == specification_review_id)
    {
        return Err("该 Stage 96 规格复核已登记实现合同，禁止覆盖或重放".to_string());
    }
    let record = build_record(&source, registrar_id, request, Utc::now())?;
    write_immutable_json(&record_path(state, specification_review_id), &record).await?;
    Ok(record)
}

fn build_record(
    source: &IndependentlyApprovedControlledShadowMarketDataParserSpecification,
    registrar_id: &str,
    request: RegisterControlledShadowMarketDataParserImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataParserImplementationRecord, String> {
    validate_source(source)?;
    validate_expected_binding(source, &request)?;
    if !all_confirmations(&request) {
        return Err(
            "Stage 97 登记前必须逐项确认完整绑定、确定性语义、零能力和职责隔离".to_string(),
        );
    }
    let implementation_name =
        bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    let immutable_code_revision = bounded_required(
        &request.immutable_code_revision,
        MAX_REVISION_CHARS,
        "不可变代码版本",
    )?;
    let implementation_description = bounded_required(
        &request.implementation_description,
        MAX_TEXT_CHARS,
        "实现说明",
    )?;
    let deterministic_parser_semantics = bounded_required(
        &request.deterministic_parser_semantics,
        MAX_TEXT_CHARS,
        "确定性解析语义",
    )?;
    let source_schema_and_numeric_semantics = bounded_required(
        &request.source_schema_and_numeric_semantics,
        MAX_TEXT_CHARS,
        "来源 schema 与数值语义",
    )?;
    let calendar_action_and_reconciliation_semantics = bounded_required(
        &request.calendar_action_and_reconciliation_semantics,
        MAX_TEXT_CHARS,
        "日历、公司行动与对账语义",
    )?;
    let error_and_missing_data_semantics = bounded_required(
        &request.error_and_missing_data_semantics,
        MAX_TEXT_CHARS,
        "错误与缺失数据语义",
    )?;
    let known_limitations =
        bounded_required(&request.known_limitations, MAX_TEXT_CHARS, "已知限制")?;
    let future_review_constraints = bounded_required(
        &request.future_review_constraints,
        MAX_TEXT_CHARS,
        "后续复核约束",
    )?;
    let excluded_prior_actor_ids = excluded_actor_ids(source);
    let independent = excluded_prior_actor_ids
        .iter()
        .all(|actor| actor != registrar_id);
    if !independent
        || !request.registrar_independent_from_stage_96_and_complete_prior_chain_confirmed
    {
        return Err("Stage 97 登记者必须独立于 Stage 96 复核者和完整上游责任链".to_string());
    }
    let implementation_contract = implementation_contract(source, &immutable_code_revision)?;
    let mut record = ControlledShadowMarketDataParserImplementationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        upstream_specification_registration: source.registered.registration.clone(),
        upstream_specification_review: source.review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_96_and_complete_prior_chain: independent,
        implementation_name,
        implementation_description,
        deterministic_parser_semantics,
        source_schema_and_numeric_semantics,
        calendar_action_and_reconciliation_semantics,
        error_and_missing_data_semantics,
        known_limitations,
        future_review_constraints,
        implementation_contract,
        status: STATUS.to_string(),
        confirmations_complete: all_confirmations(&request),
        exact_stage_51_through_stage_96_binding_confirmed: request
            .exact_stage_51_through_stage_96_binding_confirmed,
        registrar_independent_from_stage_96_and_complete_prior_chain_confirmed: request
            .registrar_independent_from_stage_96_and_complete_prior_chain_confirmed,
        independent_recomputation_of_review_registration_and_specification_confirmed: request
            .independent_recomputation_of_review_registration_and_specification_confirmed,
        zero_capability_contract_only_no_source_or_executable_artifact_confirmed: request
            .zero_capability_contract_only_no_source_or_executable_artifact_confirmed,
        fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed: request
            .fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed,
        strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: request
            .strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed,
        duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: request
            .duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed,
        no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
            request.no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed,
        spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed:
            request.spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed,
        all_eight_synthetic_vector_hashes_bound_confirmed: request
            .all_eight_synthetic_vector_hashes_bound_confirmed,
        source_available_at_remains_unverified_until_separate_review_confirmed: request
            .source_available_at_remains_unverified_until_separate_review_confirmed,
        future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
            request.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed,
        no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed:
            request.no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed,
        no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
            request.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        future_independent_implementation_review_required_before_isolated_runner_confirmed:
            request.future_independent_implementation_review_required_before_isolated_runner_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_parser_implementation_contract_registered: true,
        parser_implementation_present: false,
        future_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        isolated_runner_registration_eligible: false,
        parsed_calendar_rows_created: false,
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
    record.implementation_sha256 = record_fingerprint(&record)?;
    record.implementation_id = record.implementation_sha256[..32].to_string();
    validate_record(&record)?;
    Ok(record)
}

fn implementation_contract(
    source: &IndependentlyApprovedControlledShadowMarketDataParserSpecification,
    immutable_code_revision: &str,
) -> Result<ControlledShadowMarketDataParserImplementationContract, String> {
    let registration = &source.registered.registration;
    let review = &source.review;
    let spec = &registration.parser_specification;
    let function_ids = deterministic_function_ids();
    let mut contract = ControlledShadowMarketDataParserImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_protocol_version: IMPLEMENTATION_PROTOCOL_VERSION.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        stage_96_specification_review_id: review.review_id.clone(),
        stage_96_specification_review_sha256: review.review_sha256.clone(),
        stage_95_registration_id: registration.registration_id.clone(),
        stage_95_registration_sha256: registration.registration_sha256.clone(),
        parser_specification_sha256: spec.parser_specification_sha256.clone(),
        validation_sha256: review.validation_sha256.clone(),
        receipt_sha256: review.receipt_sha256.clone(),
        claim_sha256: review.claim_sha256.clone(),
        result_sha256: review.result_sha256.clone(),
        adapter_authorization_sha256: review.adapter_authorization_sha256.clone(),
        adapter_spec_sha256: review.adapter_spec_sha256.clone(),
        canonical_request_set_sha256: review.canonical_request_set_sha256.clone(),
        exact_parser_specification: spec.clone(),
        strict_envelope_dispatch_function_id: function_ids[0].to_string(),
        fmp_price_array_parser_function_id: function_ids[1].to_string(),
        fmp_dividend_event_parser_function_id: function_ids[2].to_string(),
        fmp_split_event_parser_function_id: function_ids[3].to_string(),
        nyse_calendar_table_parser_function_id: function_ids[4].to_string(),
        calendar_subject_spy_reconciliation_function_id: function_ids[5].to_string(),
        canonical_row_serialization_and_hash_function_id: function_ids[6].to_string(),
        synthetic_vector_conformance_function_id: function_ids[7].to_string(),
        canonical_calendar_row_schema: spec.canonical_calendar_row_schema.clone(),
        canonical_price_row_schema: spec.canonical_price_row_schema.clone(),
        canonical_dividend_row_schema: spec.canonical_dividend_row_schema.clone(),
        canonical_split_row_schema: spec.canonical_split_row_schema.clone(),
        canonical_parse_result_schema: spec.canonical_parse_result_schema.clone(),
        future_raw_payload_input_read_only_and_content_addressed: true,
        future_parser_output_create_once_and_untrusted: true,
        future_parser_output_independent_validation_required: true,
        future_parser_output_contains_market_interpretation: false,
        future_parser_output_contains_order_intent: false,
        registered_not_run: true,
        independent_implementation_review_required: true,
        isolated_runner_registration_required_after_review: true,
        authority_boundary: closed_authority_boundary(),
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn deterministic_function_ids() -> [&'static str; 8] {
    [
        "market_data_parser_strict_envelope_dispatch_v1_pure_specification",
        "market_data_parser_fmp_price_array_v1_pure_specification",
        "market_data_parser_fmp_dividend_events_v1_pure_specification",
        "market_data_parser_fmp_split_events_v1_pure_specification",
        "market_data_parser_nyse_calendar_table_v1_pure_specification",
        "market_data_parser_calendar_subject_spy_reconciliation_v1_pure_specification",
        "market_data_parser_canonical_row_serialization_hash_v1_pure_specification",
        "market_data_parser_eight_synthetic_vector_conformance_v1_pure_specification",
    ]
}

fn closed_authority_boundary() -> ZeroCapabilityMarketDataParserImplementationAuthorityBoundary {
    ZeroCapabilityMarketDataParserImplementationAuthorityBoundary {
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        raw_payload_mount_present: false,
        raw_payload_read_allowed: false,
        environment_inheritance_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        calendar_rows_created: false,
        market_rows_created: false,
        parser_output_written: false,
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

fn authority_closed(value: &ZeroCapabilityMarketDataParserImplementationAuthorityBoundary) -> bool {
    !value.source_artifact_present
        && !value.executable_artifact_present
        && !value.callable_entrypoint_present
        && !value.runtime_present
        && !value.raw_payload_mount_present
        && !value.raw_payload_read_allowed
        && !value.environment_inheritance_allowed
        && !value.secrets_allowed
        && !value.outbound_network_allowed
        && !value.tools_allowed
        && !value.subprocesses_allowed
        && !value.production_reads_allowed
        && !value.production_writes_allowed
        && !value.calendar_rows_created
        && !value.market_rows_created
        && !value.parser_output_written
        && !value.observation_write_allowed
        && !value.ledger_creation_allowed
        && !value.position_write_allowed
        && !value.performance_metric_write_allowed
        && !value.model_store_write_allowed
        && !value.metric_store_write_allowed
        && !value.training_feedback_allowed
        && !value.reward_allowed
        && !value.order_generation_allowed
        && !value.broker_access_allowed
        && !value.trading_allowed
}

fn validate_source(
    source: &IndependentlyApprovedControlledShadowMarketDataParserSpecification,
) -> Result<(), String> {
    validate_source_parts(&source.registered.registration, &source.review)
}

fn validate_source_parts(
    registration: &ControlledShadowMarketDataParserSpecificationRegistration,
    review: &ControlledShadowMarketDataParserSpecificationReview,
) -> Result<(), String> {
    let registration_sha =
        fingerprint_without(registration, &["registration_id", "registration_sha256"])?;
    let spec_sha = fingerprint_without(
        &registration.parser_specification,
        &["parser_specification_sha256"],
    )?;
    let review_sha = fingerprint_without(review, &["review_id", "review_sha256"])?;
    let exact = registration.schema_version == STAGE_95_RECORD_SCHEMA_VERSION
        && registration.policy_version == STAGE_95_POLICY_VERSION
        && registration.registration_sha256 == registration_sha
        && registration.registration_id == registration_sha[..32]
        && registration.parser_specification.schema_version == STAGE_95_SPEC_SCHEMA_VERSION
        && registration.parser_specification.parser_specification_sha256 == spec_sha
        && registration.parser_specification_registered
        && registration.future_chain_external_specification_review_eligible
        && !registration.specification_review_completed
        && !registration.parser_implementation_registration_eligible
        && stage_95_authority_closed(&registration.parser_specification.authority_boundary)
        && stage_95_rules_closed(&registration.parser_specification)
        && review.schema_version == STAGE_96_REVIEW_SCHEMA_VERSION
        && review.policy_version == STAGE_96_REVIEW_POLICY_VERSION
        && review.review_sha256 == review_sha
        && review.review_id == review_sha[..32]
        && review.registration_id == registration.registration_id
        && review.registration_sha256 == registration.registration_sha256
        && review.parser_specification_sha256
            == registration.parser_specification.parser_specification_sha256
        && review.verdict
            == ControlledShadowMarketDataParserSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityParserImplementationRegistration
        && review.confirmations_complete
        && review.independent_audit_passed
        && review.mismatch_reasons.is_empty()
        && review.parser_specification_independently_approved
        && review.future_zero_capability_parser_implementation_registration_eligible
        && review.validation_chain_independently_recomputed
        && review.explicit_source_request_set_independently_reconstructed
        && review.parser_specification_independently_recomputed
        && review.synthetic_vectors_independently_reconstructed
        && review.strict_fail_closed_semantics_independently_verified
        && review.zero_capability_boundary_independently_verified
        && stage_96_authority_closed(&review.authority_boundary)
        && review.independently_recomputed_registration_sha256 == registration_sha
        && review.independently_recomputed_parser_specification_sha256 == spec_sha
        && sorted_unique(&registration.excluded_prior_actor_ids)
        && sorted_unique(&review.excluded_prior_actor_ids)
        && !registration
            .excluded_prior_actor_ids
            .contains(&registration.registered_by)
        && !review.excluded_prior_actor_ids.contains(&review.reviewed_by)
        && review
            .excluded_prior_actor_ids
            .contains(&registration.registered_by);
    exact
        .then_some(())
        .ok_or_else(|| "Stage 95–96 parser 规格来源无效、漂移或越权".to_string())
}

fn stage_95_rules_closed(value: &ControlledShadowMarketDataParserSpecification) -> bool {
    value.strict_utf8_required
        && value.fmp_top_level_array_required
        && value.nyse_server_rendered_table_required
        && value.iso_date_only_required
        && value.finite_positive_price_required
        && value.non_negative_volume_required
        && value.duplicate_rows_rejected_not_deduplicated
        && value.out_of_window_rows_rejected
        && !value.empty_price_series_allowed
        && value.empty_dividend_or_split_event_series_allowed
        && !value.forward_fill_allowed
        && !value.interpolation_allowed
        && !value.unadjusted_close_fallback_allowed
        && !value.inferred_dividend_or_split_allowed
        && value.spy_required_for_every_official_session
        && value.subject_missing_session_is_explicit_gap_not_synthetic_row
        && value.separate_price_series_never_silently_substituted
        && value.explicit_actions_required_for_action_adjustment_claims
        && value.source_available_at_remains_unverified_until_separate_review
        && value.parser_output_create_once_and_untrusted
        && value.parser_output_independent_validation_required
        && value.synthetic_test_vectors.len() == 8
        && value
            .synthetic_test_vectors
            .iter()
            .all(|vector| vector.synthetic_only_no_market_truth)
}

fn stage_95_authority_closed(value: &ZeroCapabilityMarketDataParserAuthorityBoundary) -> bool {
    !value.parser_implementation_present
        && !value.executable_artifact_present
        && !value.callable_entrypoint_present
        && !value.runtime_present
        && !value.raw_payload_mount_present
        && !value.environment_inheritance_allowed
        && !value.secrets_allowed
        && !value.outbound_network_allowed
        && !value.tools_allowed
        && !value.subprocesses_allowed
        && !value.production_reads_allowed
        && !value.production_writes_allowed
        && !value.calendar_rows_created
        && !value.market_rows_created
        && !value.observation_write_allowed
        && !value.ledger_creation_allowed
        && !value.position_write_allowed
        && !value.performance_metric_write_allowed
        && !value.model_store_write_allowed
        && !value.metric_store_write_allowed
        && !value.training_feedback_allowed
        && !value.reward_allowed
        && !value.order_generation_allowed
        && !value.broker_access_allowed
        && !value.trading_allowed
}

fn stage_96_authority_closed(
    value: &ZeroCapabilityMarketDataParserSpecificationReviewAuthorityBoundary,
) -> bool {
    !value.parser_implementation_registered
        && !value.parser_implementation_present
        && !value.executable_artifact_present
        && !value.callable_entrypoint_present
        && !value.runtime_present
        && !value.raw_payload_read_or_mount_allowed
        && !value.environment_or_secret_access_allowed
        && !value.outbound_network_tool_or_subprocess_allowed
        && !value.production_read_or_write_allowed
        && !value.parsed_calendar_rows_created
        && !value.parsed_market_rows_created
        && !value.observation_write_allowed
        && !value.ledger_position_or_performance_write_allowed
        && !value.model_or_metric_store_write_allowed
        && !value.training_feedback_or_reward_allowed
        && !value.order_generation_or_broker_access_allowed
        && !value.trading_allowed
}

fn validate_expected_binding(
    source: &IndependentlyApprovedControlledShadowMarketDataParserSpecification,
    request: &RegisterControlledShadowMarketDataParserImplementationRequest,
) -> Result<(), String> {
    let review = &source.review;
    let registration = &source.registered.registration;
    let exact = request.expected_specification_review_id == review.review_id
        && request.expected_specification_review_sha256 == review.review_sha256
        && request.expected_registration_id == registration.registration_id
        && request.expected_registration_sha256 == registration.registration_sha256
        && request.expected_parser_specification_sha256
            == registration
                .parser_specification
                .parser_specification_sha256
        && request.expected_validation_sha256 == review.validation_sha256
        && request.expected_receipt_sha256 == review.receipt_sha256
        && request.expected_claim_sha256 == review.claim_sha256
        && request.expected_result_sha256 == review.result_sha256
        && request.expected_adapter_authorization_sha256 == review.adapter_authorization_sha256
        && request.expected_adapter_spec_sha256 == review.adapter_spec_sha256
        && request.expected_canonical_request_set_sha256 == review.canonical_request_set_sha256;
    exact
        .then_some(())
        .ok_or_else(|| "Stage 51–96 精确绑定已经变化，请刷新后重试".to_string())
}

fn all_confirmations(
    request: &RegisterControlledShadowMarketDataParserImplementationRequest,
) -> bool {
    request.exact_stage_51_through_stage_96_binding_confirmed
        && request.registrar_independent_from_stage_96_and_complete_prior_chain_confirmed
        && request.independent_recomputation_of_review_registration_and_specification_confirmed
        && request.zero_capability_contract_only_no_source_or_executable_artifact_confirmed
        && request.fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed
        && request.strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed
        && request.duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed
        && request.no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed
        && request.spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed
        && request.all_eight_synthetic_vector_hashes_bound_confirmed
        && request.source_available_at_remains_unverified_until_separate_review_confirmed
        && request.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && request.no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed
        && request.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.future_independent_implementation_review_required_before_isolated_runner_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_contract(
    contract: &ControlledShadowMarketDataParserImplementationContract,
    registration: &ControlledShadowMarketDataParserSpecificationRegistration,
    review: &ControlledShadowMarketDataParserSpecificationReview,
) -> Result<(), String> {
    let spec = &contract.exact_parser_specification;
    let expected_function_ids = deterministic_function_ids();
    let actual_function_ids = [
        contract.strict_envelope_dispatch_function_id.as_str(),
        contract.fmp_price_array_parser_function_id.as_str(),
        contract.fmp_dividend_event_parser_function_id.as_str(),
        contract.fmp_split_event_parser_function_id.as_str(),
        contract.nyse_calendar_table_parser_function_id.as_str(),
        contract
            .calendar_subject_spy_reconciliation_function_id
            .as_str(),
        contract
            .canonical_row_serialization_and_hash_function_id
            .as_str(),
        contract.synthetic_vector_conformance_function_id.as_str(),
    ];
    let exact = contract.schema_version == CONTRACT_SCHEMA_VERSION
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?
        && contract.implementation_protocol_version == IMPLEMENTATION_PROTOCOL_VERSION
        && !contract.immutable_code_revision.trim().is_empty()
        && contract.stage_96_specification_review_id == review.review_id
        && contract.stage_96_specification_review_sha256 == review.review_sha256
        && contract.stage_95_registration_id == registration.registration_id
        && contract.stage_95_registration_sha256 == registration.registration_sha256
        && contract.parser_specification_sha256
            == registration
                .parser_specification
                .parser_specification_sha256
        && contract.validation_sha256 == review.validation_sha256
        && contract.receipt_sha256 == review.receipt_sha256
        && contract.claim_sha256 == review.claim_sha256
        && contract.result_sha256 == review.result_sha256
        && contract.adapter_authorization_sha256 == review.adapter_authorization_sha256
        && contract.adapter_spec_sha256 == review.adapter_spec_sha256
        && contract.canonical_request_set_sha256 == review.canonical_request_set_sha256
        && spec == &registration.parser_specification
        && stage_95_rules_closed(spec)
        && actual_function_ids == expected_function_ids
        && contract.canonical_calendar_row_schema == spec.canonical_calendar_row_schema
        && contract.canonical_price_row_schema == spec.canonical_price_row_schema
        && contract.canonical_dividend_row_schema == spec.canonical_dividend_row_schema
        && contract.canonical_split_row_schema == spec.canonical_split_row_schema
        && contract.canonical_parse_result_schema == spec.canonical_parse_result_schema
        && contract.future_raw_payload_input_read_only_and_content_addressed
        && contract.future_parser_output_create_once_and_untrusted
        && contract.future_parser_output_independent_validation_required
        && !contract.future_parser_output_contains_market_interpretation
        && !contract.future_parser_output_contains_order_intent
        && contract.registered_not_run
        && contract.independent_implementation_review_required
        && contract.isolated_runner_registration_required_after_review
        && authority_closed(&contract.authority_boundary);
    exact
        .then_some(())
        .ok_or_else(|| "Stage 97 零能力 parser 实现合同无效、漂移或越权".to_string())
}

pub(crate) fn validate_market_data_parser_implementation_for_review(
    record: &ControlledShadowMarketDataParserImplementationRecord,
) -> Result<(), String> {
    let mut required = record
        .upstream_specification_review
        .excluded_prior_actor_ids
        .clone();
    required.push(record.upstream_specification_review.reviewed_by.clone());
    required.push(
        record
            .upstream_specification_registration
            .registered_by
            .clone(),
    );
    required.extend(
        record
            .upstream_specification_registration
            .excluded_prior_actor_ids
            .clone(),
    );
    required.sort();
    required.dedup();
    let texts_valid = [
        &record.implementation_description,
        &record.deterministic_parser_semantics,
        &record.source_schema_and_numeric_semantics,
        &record.calendar_action_and_reconciliation_semantics,
        &record.error_and_missing_data_semantics,
        &record.known_limitations,
        &record.future_review_constraints,
    ]
    .into_iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let no_downstream = !record.parser_implementation_present
        && !record.independent_implementation_review_completed
        && !record.isolated_runner_registration_eligible
        && !record.parsed_calendar_rows_created
        && !record.parsed_market_rows_created
        && !record.forward_observation_started
        && !record.ledger_created
        && !record.position_written
        && !record.performance_metric_written
        && !record.model_store_written
        && !record.metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    let valid = record.schema_version == RECORD_SCHEMA_VERSION
        && record.policy_version == POLICY_VERSION
        && record.status == STATUS
        && valid_id(&record.implementation_id)
        && valid_sha256(&record.implementation_sha256)
        && record.implementation_id == record.implementation_sha256[..32]
        && record.implementation_sha256 == record_fingerprint(record)?
        && !record.implementation_name.trim().is_empty()
        && record.implementation_name.chars().count() <= MAX_NAME_CHARS
        && record
            .implementation_contract
            .immutable_code_revision
            .chars()
            .count()
            <= MAX_REVISION_CHARS
        && texts_valid
        && sorted_unique(&record.excluded_prior_actor_ids)
        && required
            .iter()
            .all(|actor| record.excluded_prior_actor_ids.contains(actor))
        && !record.excluded_prior_actor_ids.contains(&record.registered_by)
        && record.registrar_independent_from_stage_96_and_complete_prior_chain
        && record.confirmations_complete
        && record.exact_stage_51_through_stage_96_binding_confirmed
        && record.registrar_independent_from_stage_96_and_complete_prior_chain_confirmed
        && record.independent_recomputation_of_review_registration_and_specification_confirmed
        && record.zero_capability_contract_only_no_source_or_executable_artifact_confirmed
        && record.fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed
        && record.strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed
        && record.duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed
        && record.no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed
        && record.spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed
        && record.all_eight_synthetic_vector_hashes_bound_confirmed
        && record.source_available_at_remains_unverified_until_separate_review_confirmed
        && record.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && record.no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed
        && record.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && record.future_independent_implementation_review_required_before_isolated_runner_confirmed
        && record.no_unconfirmed_hari_or_old_wang_logic_claimed
        && record.zero_capability_parser_implementation_contract_registered
        && record.future_independent_implementation_review_eligible
        && no_downstream;
    if !valid {
        return Err("Stage 97 零能力 parser 实现登记无效、漂移或越权".to_string());
    }
    validate_source_parts(
        &record.upstream_specification_registration,
        &record.upstream_specification_review,
    )?;
    validate_contract(
        &record.implementation_contract,
        &record.upstream_specification_registration,
        &record.upstream_specification_review,
    )
}

fn validate_record(
    record: &ControlledShadowMarketDataParserImplementationRecord,
) -> Result<(), String> {
    validate_market_data_parser_implementation_for_review(record)
}

fn record_matches_source(
    record: &ControlledShadowMarketDataParserImplementationRecord,
    source: &IndependentlyApprovedControlledShadowMarketDataParserSpecification,
) -> bool {
    validate_record(record).is_ok()
        && record.upstream_specification_review.review_id == source.review.review_id
        && record.upstream_specification_review.review_sha256 == source.review.review_sha256
        && record.upstream_specification_registration.registration_id
            == source.registered.registration.registration_id
        && record
            .upstream_specification_registration
            .registration_sha256
            == source.registered.registration.registration_sha256
}

fn excluded_actor_ids(
    source: &IndependentlyApprovedControlledShadowMarketDataParserSpecification,
) -> Vec<String> {
    let mut actors = source.review.excluded_prior_actor_ids.clone();
    actors.push(source.review.reviewed_by.clone());
    actors.push(source.registered.registration.registered_by.clone());
    actors.extend(
        source
            .registered
            .registration
            .excluded_prior_actor_ids
            .clone(),
    );
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn record_fingerprint(
    value: &ControlledShadowMarketDataParserImplementationRecord,
) -> Result<String, String> {
    fingerprint_without(value, &["implementation_id", "implementation_sha256"])
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserImplementationRecord>, String> {
    let mut records = Vec::new();
    let mut stack = vec![record_root(state)];
    while let Some(directory) = stack.pop() {
        let mut entries = match tokio::fs::read_dir(&directory).await {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.to_string()),
        };
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
                .is_dir()
            {
                stack.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
                let record: ControlledShadowMarketDataParserImplementationRecord =
                    serde_json::from_slice(
                        &tokio::fs::read(&path)
                            .await
                            .map_err(|error| error.to_string())?,
                    )
                    .map_err(|error| error.to_string())?;
                validate_record(&record)?;
                if path.file_stem().and_then(|value| value.to_str()) != Some("implementation") {
                    return Err("Stage 97 实现登记文件名无效".to_string());
                }
                records.push(record);
            }
        }
    }
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    let mut review_ids = BTreeSet::new();
    for record in &records {
        if !ids.insert(record.implementation_id.clone())
            || !hashes.insert(record.implementation_sha256.clone())
            || !review_ids.insert(record.upstream_specification_review.review_id.clone())
        {
            return Err("Stage 97 实现登记重复".to_string());
        }
    }
    records.sort_by(|left, right| right.registered_at.cmp(&left.registered_at));
    Ok(records)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 97 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, maximum: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > maximum {
        Err(format!("{label}不能为空且不得超过 {maximum} 字"))
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

fn record_root(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-first-natural-forward-cycle-market-data-parser-implementations")
}

fn record_path(state: &AppState, specification_review_id: &str) -> PathBuf {
    record_root(state)
        .join(specification_review_id)
        .join("implementation.json")
}

struct RegistrationLock(PathBuf);

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(
    state: &AppState,
    specification_review_id: &str,
) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-market-data-parser-implementation-{specification_review_id}.lock"
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
        .map_err(|_| "已有同一 Stage 97 实现登记正在进行".to_string())?;
    Ok(RegistrationLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterControlledShadowMarketDataParserImplementationRequest {
        RegisterControlledShadowMarketDataParserImplementationRequest {
            expected_specification_review_id: "1".repeat(32),
            expected_specification_review_sha256: "1".repeat(64),
            expected_registration_id: "2".repeat(32),
            expected_registration_sha256: "2".repeat(64),
            expected_parser_specification_sha256: "3".repeat(64),
            expected_validation_sha256: "4".repeat(64),
            expected_receipt_sha256: "5".repeat(64),
            expected_claim_sha256: "6".repeat(64),
            expected_result_sha256: "7".repeat(64),
            expected_adapter_authorization_sha256: "8".repeat(64),
            expected_adapter_spec_sha256: "9".repeat(64),
            expected_canonical_request_set_sha256: "a".repeat(64),
            implementation_name: "自然前向行情 parser 零能力实现合同".to_string(),
            immutable_code_revision: "revision-1".to_string(),
            implementation_description: "只冻结实现语义，不提供代码或入口。".to_string(),
            deterministic_parser_semantics: "相同输入字节与规格必须产生相同结果或相同失败。".to_string(),
            source_schema_and_numeric_semantics: "严格按冻结来源 schema 和有限数值规则。".to_string(),
            calendar_action_and_reconciliation_semantics: "官方日历、SPY、标的和显式公司行动必须对账。".to_string(),
            error_and_missing_data_semantics: "重复、越界、缺失或畸形均失败关闭。".to_string(),
            known_limitations: "尚未读取真实载荷，供应商语义仍未验证。".to_string(),
            future_review_constraints: "独立实现复核通过后仍只能登记隔离 runner。".to_string(),
            exact_stage_51_through_stage_96_binding_confirmed: true,
            registrar_independent_from_stage_96_and_complete_prior_chain_confirmed: true,
            independent_recomputation_of_review_registration_and_specification_confirmed: true,
            zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
            fixed_explicit_price_dividend_split_and_calendar_sources_preserved_confirmed: true,
            strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: true,
            duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: true,
            no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: true,
            spy_official_calendar_coverage_subject_gap_and_cross_source_reconciliation_preserved_confirmed: true,
            all_eight_synthetic_vector_hashes_bound_confirmed: true,
            source_available_at_remains_unverified_until_separate_review_confirmed: true,
            future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
            no_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed: true,
            no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            future_independent_implementation_review_required_before_isolated_runner_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn registration_requires_every_stage_97_confirmation() {
        let mut value = request();
        assert!(all_confirmations(&value));
        value.all_eight_synthetic_vector_hashes_bound_confirmed = false;
        assert!(!all_confirmations(&value));
    }

    #[test]
    fn implementation_authority_is_fully_closed() {
        let value = closed_authority_boundary();
        assert!(authority_closed(&value));
        assert!(!value.source_artifact_present);
        assert!(!value.raw_payload_read_allowed);
        assert!(!value.trading_allowed);
    }

    #[test]
    fn deterministic_function_contract_covers_all_parser_boundaries() {
        let values = deterministic_function_ids();
        assert_eq!(values.len(), 8);
        assert!(
            values
                .iter()
                .all(|value| value.ends_with("_pure_specification"))
        );
        assert!(values.iter().any(|value| value.contains("nyse_calendar")));
        assert!(
            values
                .iter()
                .any(|value| value.contains("synthetic_vector"))
        );
    }

    #[test]
    fn fingerprint_binds_implementation_semantics() {
        #[derive(Serialize)]
        struct Fixture {
            implementation_id: String,
            implementation_sha256: String,
            semantics: String,
        }
        let before = fingerprint_without(
            &Fixture {
                implementation_id: String::new(),
                implementation_sha256: String::new(),
                semantics: "strict".to_string(),
            },
            &["implementation_id", "implementation_sha256"],
        )
        .unwrap();
        let after = fingerprint_without(
            &Fixture {
                implementation_id: String::new(),
                implementation_sha256: String::new(),
                semantics: "fallback".to_string(),
            },
            &["implementation_id", "implementation_sha256"],
        )
        .unwrap();
        assert_ne!(before, after);
    }
}
