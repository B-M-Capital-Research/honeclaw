//! Stage 115 create-once registration of an observation-to-shadow-ledger transition specification.
//!
//! This gate defines deterministic, replayable semantics for turning one exact Stage 114 admitted
//! observation envelope into a future append-only evidence event stream. It deliberately does not
//! create a ledger, infer an opening portfolio, post cash or positions, calculate NAV/performance,
//! train a model, generate an order, access a broker or trade.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_evidence_admission_reviews::{
    AdmittedControlledShadowObservationEvidence,
    admitted_controlled_shadow_observation_evidence_for_stage_115,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-specification-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-specification-registration-v1";
const SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-specification-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-ledger-transition-specification-create-once-v1-zero-capability";
const TRANSITION_PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-observation-ledger-transition-v1-specification-only-not-executable";
const STATUS: &str = "registered_waiting_stage_116_independent_specification_review";
const NEXT_GATE: &str = "stage_116_observation_ledger_transition_specification_independent_review";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowObservationLedgerTransitionSpecificationRequest {
    expected_stage_114_review_sha256: String,
    expected_stage_113_validation_sha256: String,
    expected_stage_112_result_sha256: String,
    expected_stage_112_output_sha256: String,
    expected_stage_111_claim_sha256: String,
    registration_reason: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_current_stage_51_through_stage_114_binding_confirmed: bool,
    registrar_independent_from_stage_114_and_complete_prior_chain_confirmed: bool,
    stage_114_admission_and_full_envelope_reopened_rehashed_and_reprojected_confirmed: bool,
    stage_88_binding_not_treated_as_opening_positions_confirmed: bool,
    separately_admitted_opening_portfolio_snapshot_required_confirmed: bool,
    no_default_notional_cash_positions_or_share_quantities_confirmed: bool,
    raw_close_only_for_portfolio_marks_and_adjusted_prices_not_double_counted_confirmed: bool,
    explicit_gap_blocks_nav_no_fill_interpolation_or_substitution_confirmed: bool,
    dividend_and_split_notices_require_position_and_effective_term_validation_before_posting_confirmed:
        bool,
    exact_decimal_append_only_idempotent_and_available_at_rules_confirmed: bool,
    corrections_require_new_admitted_evidence_and_never_mutate_history_confirmed: bool,
    specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: bool,
    no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    future_chain_external_specification_review_required_before_implementation_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioPrerequisite {
    pub separately_admitted_opening_portfolio_snapshot_required: bool,
    pub current_opening_portfolio_snapshot_available: bool,
    pub stage_88_binding_is_initialization_provenance_not_opening_positions: bool,
    pub default_notional_allowed: bool,
    pub default_cash_allowed: bool,
    pub infer_positions_from_subject_symbols_allowed: bool,
    pub infer_share_quantities_from_prices_or_target_weights_allowed: bool,
    pub financial_posting_before_opening_snapshot_admission_allowed: bool,
    pub missing_opening_snapshot_result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ObservationLedgerEventMappingRules {
    pub non_financial_event_type_allowlist: Vec<String>,
    pub financial_event_type_allowlist_before_opening_snapshot: Vec<String>,
    pub canonical_event_schema: String,
    pub canonical_double_entry_posting_schema: String,
    pub event_idempotency_rule: String,
    pub evidence_availability_rule: String,
    pub security_valuation_price_basis: String,
    pub benchmark_total_return_price_basis: String,
    pub split_adjusted_price_usage: String,
    pub dividend_adjusted_price_usage: String,
    pub explicit_gap_rule: String,
    pub dividend_rule: String,
    pub split_rule: String,
    pub correction_rule: String,
    pub decimal_rule: String,
    pub canonical_event_order: String,
    pub double_entry_balance_rule: String,
    pub nav_completeness_rule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityObservationLedgerTransitionAuthorityBoundary {
    pub implementation_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub admitted_evidence_mounted_or_read_for_execution: bool,
    pub opening_portfolio_snapshot_present: bool,
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
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub transition_protocol_version: String,
    pub stage_114_review_id: String,
    pub stage_114_review_sha256: String,
    pub stage_113_validation_id: String,
    pub stage_113_validation_sha256: String,
    pub stage_112_result_id: String,
    pub stage_112_result_sha256: String,
    pub stage_112_output_sha256: String,
    pub stage_111_attempt_id: String,
    pub stage_111_claim_sha256: String,
    pub stage_104_review_sha256: String,
    pub stage_102_output_sha256: String,
    pub admitted_available_at_utc: DateTime<Utc>,
    pub provider_publication_time_verified: bool,
    pub stage_88_initialization_output_sha256: String,
    pub stage_88_initialization_manifest_sha256: String,
    pub subject_symbols: Vec<String>,
    pub benchmark_symbol: String,
    pub earliest_market_session_date: NaiveDate,
    pub latest_market_session_date: NaiveDate,
    pub observed_session_count: usize,
    pub observed_price_count: usize,
    pub observed_gap_count: usize,
    pub observed_dividend_count: usize,
    pub observed_split_count: usize,
    pub exact_stage_114_admitted_envelope_is_only_observation_input: bool,
    pub opening_portfolio_prerequisite: OpeningPortfolioPrerequisite,
    pub mapping_rules: ObservationLedgerEventMappingRules,
    pub future_ledger_relative_path_template: String,
    pub future_event_stream_relative_path_template: String,
    pub create_once_required: bool,
    pub append_only_required: bool,
    pub overwrite_allowed: bool,
    pub retroactive_backfill_allowed: bool,
    pub in_place_correction_allowed: bool,
    pub financial_postings_currently_eligible: bool,
    pub nav_or_performance_currently_eligible: bool,
    pub future_independent_specification_review_required: bool,
    pub authority_boundary: ZeroCapabilityObservationLedgerTransitionAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationRegistration {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub stage_114_review_id: String,
    pub stage_114_review_sha256: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_114_and_complete_prior_chain: bool,
    pub registration_reason: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub specification: ControlledShadowObservationLedgerTransitionSpecification,
    pub status: String,
    pub confirmations_complete: bool,
    pub specification_registered: bool,
    pub future_chain_external_specification_review_eligible: bool,
    pub specification_review_completed: bool,
    pub implementation_registration_eligible: bool,
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
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationCandidate {
    pub stage_114_review_id: String,
    pub stage_114_review_sha256: String,
    pub stage_113_validation_sha256: String,
    pub stage_112_result_sha256: String,
    pub stage_112_output_sha256: String,
    pub stage_111_claim_sha256: String,
    pub admitted_available_at_utc: DateTime<Utc>,
    pub subject_symbols: Vec<String>,
    pub observed_session_count: usize,
    pub observed_price_count: usize,
    pub observed_gap_count: usize,
    pub registrar_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_endpoint_available: bool,
    pub candidates: Vec<ControlledShadowObservationLedgerTransitionSpecificationCandidate>,
    pub registrations: Vec<ControlledShadowObservationLedgerTransitionSpecificationRegistration>,
    pub admitted_observation_evidence_count: usize,
    pub registration_eligible_count: usize,
    pub registered_specification_count: usize,
    pub future_stage_116_independent_review_eligible_count: usize,
    pub opening_portfolio_snapshot_missing_count: usize,
    pub registration_status: String,
    pub next_gate: String,
    pub implementation_present: bool,
    pub opening_portfolio_snapshot_present: bool,
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
pub(crate) struct ControlledShadowObservationLedgerTransitionSpecificationReadinessSummary {
    pub admitted_observation_evidence_count: usize,
    pub registration_eligible_count: usize,
    pub registered_specification_count: usize,
    pub future_stage_116_independent_review_eligible_count: usize,
    pub opening_portfolio_snapshot_missing_count: usize,
    pub registration_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentlyReviewableControlledShadowObservationLedgerTransitionSpecification {
    pub source: AdmittedControlledShadowObservationEvidence,
    pub registration: ControlledShadowObservationLedgerTransitionSpecificationRegistration,
}

pub(crate) async fn handle_get_controlled_shadow_observation_ledger_transition_specifications(
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
            warn!(%error, "observation ledger transition specification registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察证据到账本转换规格暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_observation_ledger_transition_specification(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(review_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowObservationLedgerTransitionSpecificationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &review_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_observation_ledger_transition_specification_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionSpecificationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowObservationLedgerTransitionSpecificationReadinessSummary {
            admitted_observation_evidence_count: value.admitted_observation_evidence_count,
            registration_eligible_count: value.registration_eligible_count,
            registered_specification_count: value.registered_specification_count,
            future_stage_116_independent_review_eligible_count: value
                .future_stage_116_independent_review_eligible_count,
            opening_portfolio_snapshot_missing_count: value
                .opening_portfolio_snapshot_missing_count,
            registration_status: value.registration_status,
        },
    )
}

pub(crate) async fn independently_reviewable_observation_ledger_transition_specifications(
    state: &AppState,
) -> Result<
    Vec<IndependentlyReviewableControlledShadowObservationLedgerTransitionSpecification>,
    String,
> {
    let sources = admitted_controlled_shadow_observation_evidence_for_stage_115(state).await?;
    let source_index = sources
        .into_iter()
        .map(|source| (source.admission_review.review_id.clone(), source))
        .collect::<HashMap<_, _>>();
    let registrations = read_records(state).await?;
    let mut values = Vec::with_capacity(registrations.len());
    for registration in registrations {
        let source = source_index
            .get(&registration.stage_114_review_id)
            .ok_or_else(|| "Stage 115 登记不再绑定当前 Stage 114 准入证据".to_string())?
            .clone();
        validate_source_binding(&registration, &source)?;
        values.push(
            IndependentlyReviewableControlledShadowObservationLedgerTransitionSpecification {
                source,
                registration,
            },
        );
    }
    values.sort_by(|left, right| {
        right
            .registration
            .registered_at
            .cmp(&left.registration.registered_at)
    });
    Ok(values)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationLedgerTransitionSpecificationRegistry, String> {
    let sources = admitted_controlled_shadow_observation_evidence_for_stage_115(state).await?;
    let registrations = read_records(state).await?;
    for registration in &registrations {
        let source = sources
            .iter()
            .find(|value| value.admission_review.review_id == registration.stage_114_review_id)
            .ok_or_else(|| "Stage 115 登记的 Stage 114 准入证据已不再当前有效".to_string())?;
        validate_source_binding(registration, source)?;
    }
    let registered = registrations
        .iter()
        .map(|value| value.stage_114_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let candidates = sources
        .iter()
        .map(candidate_from_source)
        .collect::<Vec<_>>();
    let registration_eligible_count = candidates
        .iter()
        .filter(|value| !registered.contains(value.stage_114_review_id.as_str()))
        .count();
    let registered_specification_count = registrations.len();
    let registration_status = if registered_specification_count > 0 {
        "ledger_transition_specification_registered_waiting_stage_116_independent_review"
    } else if !sources.is_empty() {
        "waiting_zero_capability_ledger_transition_specification_registration"
    } else {
        "waiting_stage_114_admitted_observation_evidence"
    };
    Ok(ControlledShadowObservationLedgerTransitionSpecificationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_endpoint_available: true,
        candidates,
        registrations,
        admitted_observation_evidence_count: sources.len(),
        registration_eligible_count,
        registered_specification_count,
        future_stage_116_independent_review_eligible_count: registered_specification_count,
        opening_portfolio_snapshot_missing_count: registered_specification_count,
        registration_status: registration_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        implementation_present: false,
        opening_portfolio_snapshot_present: false,
        ledger_created: false,
        ledger_event_written: false,
        nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 115 只登记 exact Stage 114 admitted envelope 到未来 append-only 影子账本事件流的确定性规格。Stage 88 绑定明确只是初始化来源，不是开仓持仓；缺少另行独立准入的 opening portfolio snapshot 时，禁止默认本金、现金、持仓或股数。raw close 仅定义未来持仓估值口径，adjusted prices 不参与会计以避免双计，公司行动缺少有效条款或持仓时不入账，显式 gap 使 NAV 失败关闭。当前无实现、工件、runtime、账本、事件、持仓、现金、NAV/绩效、训练/RL、订单、券商或交易。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    review_id: &str,
    request: RegisterControlledShadowObservationLedgerTransitionSpecificationRequest,
) -> Result<ControlledShadowObservationLedgerTransitionSpecificationRegistration, String> {
    validate_request(review_id, &request)?;
    let _lock = acquire_lock(state, review_id).await?;
    let sources = admitted_controlled_shadow_observation_evidence_for_stage_115(state).await?;
    let source = sources
        .iter()
        .find(|value| value.admission_review.review_id == review_id)
        .ok_or_else(|| "Stage 115 未找到当前有效且已准入的精确观察证据".to_string())?;
    verify_expected_bindings(source, &request)?;
    if read_records(state)
        .await?
        .iter()
        .any(|value| value.stage_114_review_id == review_id)
    {
        return Err("该 Stage 114 准入证据已经登记过转换规格，禁止覆盖".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(source);
    let independent = actor_is_independent(registrar_id, &excluded_prior_actor_ids);
    if !independent
        || !request.registrar_independent_from_stage_114_and_complete_prior_chain_confirmed
    {
        return Err(
            "Stage 115 registrar 必须独立于 Stage 114 reviewer 与完整既有责任链".to_string(),
        );
    }
    let specification = canonical_specification(source)?;
    let mut registration = ControlledShadowObservationLedgerTransitionSpecificationRegistration {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_id: String::new(),
        registration_sha256: String::new(),
        registered_at: Utc::now(),
        registered_by: registrar_id.to_string(),
        stage_114_review_id: source.admission_review.review_id.clone(),
        stage_114_review_sha256: source.admission_review.review_sha256.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_114_and_complete_prior_chain: independent,
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
        future_chain_external_specification_review_eligible: true,
        specification_review_completed: false,
        implementation_registration_eligible: false,
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
    source: &AdmittedControlledShadowObservationEvidence,
) -> Result<ControlledShadowObservationLedgerTransitionSpecification, String> {
    let envelope = &source.envelope;
    let validation = &source.candidate.validation;
    let result = &source.candidate.result;
    let claim = &source.candidate.claim;
    let review = &source.admission_review;
    let earliest_market_session_date = envelope
        .sessions
        .first()
        .ok_or_else(|| "Stage 115 envelope 没有官方交易日".to_string())?
        .date;
    let latest_market_session_date = envelope
        .sessions
        .last()
        .ok_or_else(|| "Stage 115 envelope 没有官方交易日".to_string())?
        .date;
    let subject_symbols = subject_symbols(source);
    if subject_symbols.is_empty() {
        return Err("Stage 115 envelope 没有可识别的非 SPY 标的".to_string());
    }
    let mut specification = ControlledShadowObservationLedgerTransitionSpecification {
        schema_version: SPEC_SCHEMA_VERSION.to_string(),
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
        opening_portfolio_prerequisite: opening_portfolio_prerequisite(),
        mapping_rules: canonical_mapping_rules(),
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
        authority_boundary: closed_authority_boundary(),
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])?;
    validate_specification(&specification)?;
    Ok(specification)
}

fn opening_portfolio_prerequisite() -> OpeningPortfolioPrerequisite {
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

fn canonical_mapping_rules() -> ObservationLedgerEventMappingRules {
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

fn closed_authority_boundary() -> ZeroCapabilityObservationLedgerTransitionAuthorityBoundary {
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

fn validate_specification(
    value: &ControlledShadowObservationLedgerTransitionSpecification,
) -> Result<(), String> {
    let symbol_set = value.subject_symbols.iter().collect::<BTreeSet<_>>();
    let valid = value.schema_version == SPEC_SCHEMA_VERSION
        && value.transition_protocol_version == TRANSITION_PROTOCOL_VERSION
        && valid_sha256(&value.specification_sha256)
        && value.specification_sha256 == fingerprint_without(value, &["specification_sha256"])?
        && valid_id(&value.stage_114_review_id)
        && valid_id(&value.stage_113_validation_id)
        && valid_id(&value.stage_112_result_id)
        && valid_id(&value.stage_111_attempt_id)
        && [
            &value.stage_114_review_sha256,
            &value.stage_113_validation_sha256,
            &value.stage_112_result_sha256,
            &value.stage_112_output_sha256,
            &value.stage_111_claim_sha256,
            &value.stage_104_review_sha256,
            &value.stage_102_output_sha256,
            &value.stage_88_initialization_output_sha256,
            &value.stage_88_initialization_manifest_sha256,
        ]
        .iter()
        .all(|hash| valid_sha256(hash))
        && !value.provider_publication_time_verified
        && !value.subject_symbols.is_empty()
        && symbol_set.len() == value.subject_symbols.len()
        && value.subject_symbols.iter().all(|symbol| symbol != "SPY")
        && value.benchmark_symbol == "SPY"
        && value.earliest_market_session_date <= value.latest_market_session_date
        && value.observed_session_count > 0
        && value.observed_price_count > 0
        && value.exact_stage_114_admitted_envelope_is_only_observation_input
        && value.opening_portfolio_prerequisite == opening_portfolio_prerequisite()
        && value.mapping_rules == canonical_mapping_rules()
        && value.create_once_required
        && value.append_only_required
        && !value.overwrite_allowed
        && !value.retroactive_backfill_allowed
        && !value.in_place_correction_allowed
        && !value.financial_postings_currently_eligible
        && !value.nav_or_performance_currently_eligible
        && value.future_independent_specification_review_required
        && value.authority_boundary == closed_authority_boundary();
    valid
        .then_some(())
        .ok_or_else(|| "Stage 115 观察证据到账本转换规格偏离固定零能力合同".to_string())
}

fn validate_record(
    value: &ControlledShadowObservationLedgerTransitionSpecificationRegistration,
) -> Result<(), String> {
    validate_specification(&value.specification)?;
    let downstream_closed = !value.specification_review_completed
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
        && !value.trading_authorized;
    let valid = value.schema_version == RECORD_SCHEMA_VERSION
        && value.policy_version == POLICY_VERSION
        && valid_id(&value.registration_id)
        && valid_sha256(&value.registration_sha256)
        && value.registration_id == value.registration_sha256[..32]
        && value.registration_sha256
            == fingerprint_without(value, &["registration_id", "registration_sha256"])?
        && value.stage_114_review_id == value.specification.stage_114_review_id
        && value.stage_114_review_sha256 == value.specification.stage_114_review_sha256
        && value.registrar_independent_from_stage_114_and_complete_prior_chain
        && !value.registered_by.trim().is_empty()
        && sorted_unique(&value.excluded_prior_actor_ids)
        && actor_is_independent(&value.registered_by, &value.excluded_prior_actor_ids)
        && !value.registration_reason.trim().is_empty()
        && !value.known_limitations.trim().is_empty()
        && !value.future_review_constraints.trim().is_empty()
        && value.status == STATUS
        && value.confirmations_complete
        && value.specification_registered
        && value.future_chain_external_specification_review_eligible
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 115 规格登记记录无效、漂移或越权".to_string())
}

fn validate_source_binding(
    registration: &ControlledShadowObservationLedgerTransitionSpecificationRegistration,
    source: &AdmittedControlledShadowObservationEvidence,
) -> Result<(), String> {
    validate_record(registration)?;
    let expected = canonical_specification(source)?;
    let expected_excluded_actor_ids = excluded_actor_ids(source);
    (registration.stage_114_review_id == source.admission_review.review_id
        && registration.stage_114_review_sha256 == source.admission_review.review_sha256
        && registration.specification == expected
        && registration.excluded_prior_actor_ids == expected_excluded_actor_ids
        && actor_is_independent(
            &registration.registered_by,
            &registration.excluded_prior_actor_ids,
        ))
    .then_some(())
    .ok_or_else(|| "Stage 115 登记已与当前 Stage 114 准入证据漂移".to_string())
}

fn verify_expected_bindings(
    source: &AdmittedControlledShadowObservationEvidence,
    request: &RegisterControlledShadowObservationLedgerTransitionSpecificationRequest,
) -> Result<(), String> {
    let review = &source.admission_review;
    let candidate = &source.candidate;
    let valid = request.expected_stage_114_review_sha256 == review.review_sha256
        && request.expected_stage_113_validation_sha256 == candidate.validation.validation_sha256
        && request.expected_stage_112_result_sha256 == candidate.result.result_sha256
        && request.expected_stage_112_output_sha256 == candidate.validation.stage_112_output_sha256
        && request.expected_stage_111_claim_sha256 == candidate.claim.claim_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 115 请求与当前 Stage 111–114 精确绑定不一致".to_string())
}

fn validate_request(
    review_id: &str,
    request: &RegisterControlledShadowObservationLedgerTransitionSpecificationRequest,
) -> Result<(), String> {
    if !valid_id(review_id)
        || ![
            &request.expected_stage_114_review_sha256,
            &request.expected_stage_113_validation_sha256,
            &request.expected_stage_112_result_sha256,
            &request.expected_stage_112_output_sha256,
            &request.expected_stage_111_claim_sha256,
        ]
        .iter()
        .all(|hash| valid_sha256(hash))
    {
        return Err("Stage 115 请求绑定格式无效".to_string());
    }
    if !confirmations_complete(request) {
        return Err("Stage 115 必须逐项确认会计口径、缺失输入、责任链与全部零权限边界".to_string());
    }
    Ok(())
}

fn confirmations_complete(
    request: &RegisterControlledShadowObservationLedgerTransitionSpecificationRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_114_binding_confirmed
        && request.registrar_independent_from_stage_114_and_complete_prior_chain_confirmed
        && request.stage_114_admission_and_full_envelope_reopened_rehashed_and_reprojected_confirmed
        && request.stage_88_binding_not_treated_as_opening_positions_confirmed
        && request.separately_admitted_opening_portfolio_snapshot_required_confirmed
        && request.no_default_notional_cash_positions_or_share_quantities_confirmed
        && request.raw_close_only_for_portfolio_marks_and_adjusted_prices_not_double_counted_confirmed
        && request.explicit_gap_blocks_nav_no_fill_interpolation_or_substitution_confirmed
        && request.dividend_and_split_notices_require_position_and_effective_term_validation_before_posting_confirmed
        && request.exact_decimal_append_only_idempotent_and_available_at_rules_confirmed
        && request.corrections_require_new_admitted_evidence_and_never_mutate_history_confirmed
        && request.specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed
        && request.no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && request.future_chain_external_specification_review_required_before_implementation_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn candidate_from_source(
    source: &AdmittedControlledShadowObservationEvidence,
) -> ControlledShadowObservationLedgerTransitionSpecificationCandidate {
    ControlledShadowObservationLedgerTransitionSpecificationCandidate {
        stage_114_review_id: source.admission_review.review_id.clone(),
        stage_114_review_sha256: source.admission_review.review_sha256.clone(),
        stage_113_validation_sha256: source.candidate.validation.validation_sha256.clone(),
        stage_112_result_sha256: source.candidate.result.result_sha256.clone(),
        stage_112_output_sha256: source.candidate.validation.stage_112_output_sha256.clone(),
        stage_111_claim_sha256: source.candidate.claim.claim_sha256.clone(),
        admitted_available_at_utc: source.admission_review.admitted_available_at_utc,
        subject_symbols: subject_symbols(source),
        observed_session_count: source.envelope.sessions.len(),
        observed_price_count: source.envelope.prices.len(),
        observed_gap_count: source.envelope.explicit_gaps.len(),
        registrar_excluded_actor_ids: excluded_actor_ids(source),
    }
}

fn subject_symbols(source: &AdmittedControlledShadowObservationEvidence) -> Vec<String> {
    let mut symbols = source
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
        .collect::<Vec<_>>();
    symbols.sort();
    symbols.dedup();
    symbols
}

fn excluded_actor_ids(source: &AdmittedControlledShadowObservationEvidence) -> Vec<String> {
    let mut values = source.admission_review.excluded_prior_actor_ids.clone();
    values.push(source.admission_review.submitted_by.clone());
    values.push(source.candidate.validation.validated_by.clone());
    values.push(source.candidate.result.executed_by.clone());
    values.push(source.candidate.claim.claimed_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn actor_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|value| value != actor_id)
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationLedgerTransitionSpecificationRegistration>, String> {
    let directory = record_dir(state);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            return Err("Stage 115 登记目录含非 JSON 文件".to_string());
        }
        let metadata = tokio::fs::symlink_metadata(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.len() == 0
            || metadata.len() > 1024 * 1024
        {
            return Err("Stage 115 登记文件无效或超限".to_string());
        }
        let record: ControlledShadowObservationLedgerTransitionSpecificationRegistration =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_record(&record)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(record.registration_id.as_str())
        {
            return Err("Stage 115 登记文件名与自哈希 ID 不一致".to_string());
        }
        records.push(record);
    }
    records.sort_by(|left, right| left.registered_at.cmp(&right.registered_at));
    let unique_sources = records
        .iter()
        .map(|value| value.stage_114_review_id.as_str())
        .collect::<BTreeSet<_>>();
    let unique_ids = records
        .iter()
        .map(|value| value.registration_id.as_str())
        .collect::<BTreeSet<_>>();
    if unique_sources.len() != records.len() || unique_ids.len() != records.len() {
        return Err("Stage 115 出现重复语义登记或重复 ID".to_string());
    }
    Ok(records)
}

fn record_dir(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-controlled-shadow-observation-ledger-transition-specifications")
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

async fn acquire_lock(state: &AppState, review_id: &str) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage-115-{review_id}.lock"));
    match tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
    {
        Ok(_) => Ok(RegistrationLock(path)),
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
                Box::pin(acquire_lock(state, review_id)).await
            } else {
                Err("Stage 115 同一准入证据正在登记，请稍后重试".to_string())
            }
        }
        Err(error) => Err(error.to_string()),
    }
}

struct RegistrationLock(PathBuf);

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Stage 115 记录路径缺少父目录".to_string())?;
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
                "Stage 115 记录已存在，禁止覆盖".to_string()
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
            "Stage 115 {label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"
        ));
    }
    Ok(trimmed.to_string())
}

fn fingerprint_without<T: Serialize>(value: &T, excluded: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Stage 115 指纹对象不是 JSON object".to_string())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterControlledShadowObservationLedgerTransitionSpecificationRequest {
        RegisterControlledShadowObservationLedgerTransitionSpecificationRequest {
            expected_stage_114_review_sha256: "a".repeat(64),
            expected_stage_113_validation_sha256: "b".repeat(64),
            expected_stage_112_result_sha256: "c".repeat(64),
            expected_stage_112_output_sha256: "d".repeat(64),
            expected_stage_111_claim_sha256: "e".repeat(64),
            registration_reason: "冻结观察证据到账本事件的可复算规则。".to_string(),
            known_limitations: "当前缺少独立准入的 opening portfolio snapshot。".to_string(),
            future_review_constraints: "必须由责任链外复核者独立重建规格。".to_string(),
            exact_current_stage_51_through_stage_114_binding_confirmed: true,
            registrar_independent_from_stage_114_and_complete_prior_chain_confirmed: true,
            stage_114_admission_and_full_envelope_reopened_rehashed_and_reprojected_confirmed: true,
            stage_88_binding_not_treated_as_opening_positions_confirmed: true,
            separately_admitted_opening_portfolio_snapshot_required_confirmed: true,
            no_default_notional_cash_positions_or_share_quantities_confirmed: true,
            raw_close_only_for_portfolio_marks_and_adjusted_prices_not_double_counted_confirmed: true,
            explicit_gap_blocks_nav_no_fill_interpolation_or_substitution_confirmed: true,
            dividend_and_split_notices_require_position_and_effective_term_validation_before_posting_confirmed: true,
            exact_decimal_append_only_idempotent_and_available_at_rules_confirmed: true,
            corrections_require_new_admitted_evidence_and_never_mutate_history_confirmed: true,
            specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: true,
            no_ledger_event_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: true,
            future_chain_external_specification_review_required_before_implementation_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn opening_state_is_never_invented_from_stage_88_or_market_prices() {
        let prerequisite = opening_portfolio_prerequisite();
        assert!(prerequisite.separately_admitted_opening_portfolio_snapshot_required);
        assert!(prerequisite.stage_88_binding_is_initialization_provenance_not_opening_positions);
        assert!(!prerequisite.current_opening_portfolio_snapshot_available);
        assert!(!prerequisite.default_notional_allowed);
        assert!(!prerequisite.default_cash_allowed);
        assert!(!prerequisite.infer_positions_from_subject_symbols_allowed);
        assert!(!prerequisite.infer_share_quantities_from_prices_or_target_weights_allowed);
        assert!(!prerequisite.financial_posting_before_opening_snapshot_admission_allowed);
    }

    #[test]
    fn accounting_rules_avoid_adjusted_price_double_counting_and_fail_on_gaps() {
        let rules = canonical_mapping_rules();
        assert!(
            rules
                .security_valuation_price_basis
                .starts_with("raw_unadjusted_price")
        );
        assert!(rules.benchmark_total_return_price_basis.contains("SPY"));
        assert!(
            rules
                .split_adjusted_price_usage
                .contains("never_a_portfolio_accounting_mark")
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
                .nav_completeness_rule
                .contains("no_NAV_or_performance_fact")
        );
        assert!(
            rules
                .financial_event_type_allowlist_before_opening_snapshot
                .is_empty()
        );
    }

    #[test]
    fn specification_registration_has_zero_runtime_or_financial_authority() {
        let boundary = closed_authority_boundary();
        assert_eq!(
            boundary,
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
        );
    }

    #[test]
    fn every_accounting_and_governance_confirmation_is_required() {
        let mut value = request();
        assert!(confirmations_complete(&value));
        value.stage_88_binding_not_treated_as_opening_positions_confirmed = false;
        assert!(!confirmations_complete(&value));
        assert!(validate_request("review-115", &value).is_err());
    }
}
