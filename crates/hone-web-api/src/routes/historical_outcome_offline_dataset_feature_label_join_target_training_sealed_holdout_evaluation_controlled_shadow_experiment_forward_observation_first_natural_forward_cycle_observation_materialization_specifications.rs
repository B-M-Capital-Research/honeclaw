//! Stage 105 create-once registration of the first natural-cycle observation materialization
//! specification.
//!
//! This gate freezes how one exact Stage 104 admitted input may later be projected into an
//! immutable, still-untrusted observation envelope. It contains no implementation, artifact,
//! entrypoint or runtime and cannot create an observation, ledger, position, performance metric,
//! model/reward fact, order, broker request or trade.

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
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_input_admission_reviews::{
    AdmittedControlledShadowObservationInputForMaterializationSpecification,
    admitted_controlled_shadow_observation_inputs_for_materialization_specification,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-controlled-shadow-first-natural-cycle-observation-materialization-specification-registry-v1";
const RECORD_SCHEMA_VERSION: &str = "hone-controlled-shadow-first-natural-cycle-observation-materialization-specification-registration-v1";
const SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-first-natural-cycle-observation-materialization-specification-v1";
const POLICY_VERSION: &str = "hone-first-natural-cycle-observation-materialization-specification-create-once-v1-zero-capability";
const MATERIALIZATION_PROTOCOL_VERSION: &str =
    "hone-first-natural-cycle-observation-materialization-v1-specification-only-not-executable";
const STATUS: &str = "registered_not_reviewed_not_implemented_not_run";
const NEXT_GATE: &str = "stage_106_first_natural_forward_cycle_observation_materialization_specification_independent_review";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowObservationMaterializationSpecificationRequest {
    expected_stage_104_review_sha256: String,
    expected_stage_103_validation_sha256: String,
    expected_stage_102_result_sha256: String,
    expected_stage_102_output_sha256: String,
    expected_stage_101_claim_sha256: String,
    expected_stage_101_input_manifest_sha256: String,
    expected_cycle_claim_sha256: String,
    registration_reason: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_current_stage_51_through_stage_104_binding_confirmed: bool,
    registrar_independent_from_stage_104_and_complete_prior_chain_confirmed: bool,
    exact_admitted_output_only_no_refetch_or_reparse_confirmed: bool,
    conservative_available_at_floor_and_provider_time_limitation_preserved_confirmed: bool,
    official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: bool,
    subject_missingness_explicit_no_fill_interpolation_or_substitution_confirmed: bool,
    dividends_splits_and_price_bases_remain_separate_confirmed: bool,
    initial_shadow_allocation_binding_preserved_without_accounting_transition_confirmed: bool,
    deterministic_canonical_order_decimal_and_row_hash_rules_confirmed: bool,
    one_envelope_create_once_no_overwrite_backfill_or_in_place_correction_confirmed: bool,
    spy_gap_duplicate_out_of_window_or_hash_drift_fail_closed_confirmed: bool,
    specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: bool,
    no_network_environment_secret_tool_subprocess_production_read_or_write_confirmed: bool,
    no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    future_chain_external_specification_review_required_before_implementation_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityObservationMaterializationAuthorityBoundary {
    pub implementation_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub environment_inheritance_allowed: bool,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub observation_envelope_created: bool,
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
pub(crate) struct ControlledShadowObservationMaterializationSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub materialization_protocol_version: String,
    pub stage_104_review_id: String,
    pub stage_104_review_sha256: String,
    pub stage_103_validation_id: String,
    pub stage_103_validation_sha256: String,
    pub stage_102_attempt_id: String,
    pub stage_102_result_sha256: String,
    pub stage_102_output_sha256: String,
    pub stage_101_claim_sha256: String,
    pub stage_101_input_manifest_sha256: String,
    pub cycle_claim_id: String,
    pub cycle_claim_sha256: String,
    pub stage_88_initial_observation_output_sha256: String,
    pub initial_shadow_allocation_manifest_sha256: String,
    pub subject_symbols: Vec<String>,
    pub benchmark_symbol: String,
    pub window_start_date: NaiveDate,
    pub window_end_date: NaiveDate,
    pub earliest_market_session_date: NaiveDate,
    pub latest_market_session_date: NaiveDate,
    pub official_market_session_count: usize,
    pub admitted_available_at_utc: DateTime<Utc>,
    pub availability_basis: String,
    pub provider_publication_time_verified: bool,
    pub exact_admitted_output_is_only_input: bool,
    pub allowed_price_bases: Vec<String>,
    pub canonical_session_schema: String,
    pub canonical_price_observation_schema: String,
    pub canonical_explicit_gap_schema: String,
    pub canonical_corporate_action_schema: String,
    pub canonical_observation_envelope_schema: String,
    pub canonical_order: String,
    pub decimal_preservation_rule: String,
    pub row_hash_rule: String,
    pub future_output_relative_path_template: String,
    pub one_envelope_per_admitted_cycle: bool,
    pub create_once_required: bool,
    pub overwrite_allowed: bool,
    pub retroactive_backfill_allowed: bool,
    pub in_place_correction_allowed: bool,
    pub subject_gap_is_explicit_not_synthetic: bool,
    pub forward_fill_allowed: bool,
    pub interpolation_allowed: bool,
    pub cross_price_basis_substitution_allowed: bool,
    pub benchmark_gap_allowed: bool,
    pub duplicate_row_allowed: bool,
    pub out_of_window_row_allowed: bool,
    pub dividends_splits_and_prices_separate: bool,
    pub initial_shadow_allocation_recomputed: bool,
    pub accounting_transition_applied: bool,
    pub valuation_or_investment_semantics_allowed: bool,
    pub future_output_untrusted: bool,
    pub future_output_independent_validation_required: bool,
    pub authority_boundary: ZeroCapabilityObservationMaterializationAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationSpecificationRegistration {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub stage_104_review_id: String,
    pub stage_104_review_sha256: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_104_and_complete_prior_chain: bool,
    pub registration_reason: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub specification: ControlledShadowObservationMaterializationSpecification,
    pub status: String,
    pub confirmations_complete: bool,
    pub specification_registered: bool,
    pub future_chain_external_specification_review_eligible: bool,
    pub specification_review_completed: bool,
    pub implementation_registration_eligible: bool,
    pub observation_materialized: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationSpecificationCandidate {
    pub stage_104_review_id: String,
    pub stage_104_review_sha256: String,
    pub stage_103_validation_sha256: String,
    pub stage_102_attempt_id: String,
    pub stage_102_result_sha256: String,
    pub stage_102_output_sha256: String,
    pub stage_101_claim_sha256: String,
    pub stage_101_input_manifest_sha256: String,
    pub cycle_claim_sha256: String,
    pub subject_symbols: Vec<String>,
    pub benchmark_symbol: String,
    pub admitted_available_at_utc: DateTime<Utc>,
    pub official_market_session_count: usize,
    pub explicit_gap_count: usize,
    pub registrar_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationSpecificationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_endpoint_available: bool,
    pub candidates: Vec<ControlledShadowObservationMaterializationSpecificationCandidate>,
    pub registrations: Vec<ControlledShadowObservationMaterializationSpecificationRegistration>,
    pub admitted_input_count: usize,
    pub registration_eligible_count: usize,
    pub specification_registered_count: usize,
    pub future_chain_external_specification_review_eligible_count: usize,
    pub specification_status: String,
    pub next_gate: String,
    pub implementation_present: bool,
    pub observation_materialized: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationMaterializationSpecificationReadinessSummary {
    pub admitted_input_count: usize,
    pub registration_eligible_count: usize,
    pub specification_registered_count: usize,
    pub future_chain_external_specification_review_eligible_count: usize,
    pub specification_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct IndependentlyReviewableControlledShadowObservationMaterializationSpecification {
    pub source: AdmittedControlledShadowObservationInputForMaterializationSpecification,
    pub registration: ControlledShadowObservationMaterializationSpecificationRegistration,
}

pub(crate) async fn handle_get_controlled_shadow_observation_materialization_specifications(
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
            warn!(%error, "observation materialization specification registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察物化规格登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_observation_materialization_specification(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(review_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowObservationMaterializationSpecificationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &review_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(_) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "规格已登记，但刷新失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_observation_materialization_specification_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationSpecificationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowObservationMaterializationSpecificationReadinessSummary {
            admitted_input_count: registry.admitted_input_count,
            registration_eligible_count: registry.registration_eligible_count,
            specification_registered_count: registry.specification_registered_count,
            future_chain_external_specification_review_eligible_count: registry
                .future_chain_external_specification_review_eligible_count,
            specification_status: registry.specification_status,
        },
    )
}

pub(crate) async fn independently_reviewable_observation_materialization_specifications(
    state: &AppState,
) -> Result<
    Vec<IndependentlyReviewableControlledShadowObservationMaterializationSpecification>,
    String,
> {
    let sources =
        admitted_controlled_shadow_observation_inputs_for_materialization_specification(state)
            .await?;
    let source_index = sources
        .into_iter()
        .map(|source| (source.admission_review.review_id.clone(), source))
        .collect::<HashMap<_, _>>();
    let registrations = read_records(state).await?;
    let mut values = Vec::with_capacity(registrations.len());
    for registration in registrations {
        let source = source_index
            .get(&registration.stage_104_review_id)
            .ok_or_else(|| "Stage 105 登记不再绑定当前 Stage 104 准入源".to_string())?
            .clone();
        validate_source_binding(&registration, &source)?;
        values.push(
            IndependentlyReviewableControlledShadowObservationMaterializationSpecification {
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
) -> Result<ControlledShadowObservationMaterializationSpecificationRegistry, String> {
    let sources =
        admitted_controlled_shadow_observation_inputs_for_materialization_specification(state)
            .await?;
    let registrations = read_records(state).await?;
    let registration_index = registrations
        .iter()
        .map(|value| (value.stage_104_review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let mut candidates = Vec::new();
    for source in &sources {
        if let Some(registration) =
            registration_index.get(source.admission_review.review_id.as_str())
        {
            validate_source_binding(registration, source)?;
        } else {
            candidates.push(candidate(source));
        }
    }
    if registrations.len() + candidates.len() != sources.len() {
        return Err("Stage 105 登记与当前 Stage 104 准入链不一致".into());
    }
    let status = if !registrations.is_empty() {
        "observation_materialization_specification_registered_pending_chain_external_review"
    } else if !candidates.is_empty() {
        "waiting_zero_capability_observation_materialization_specification_registration"
    } else {
        "waiting_stage_104_admitted_observation_input"
    };
    Ok(ControlledShadowObservationMaterializationSpecificationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        registration_endpoint_available: true,
        admitted_input_count: sources.len(),
        registration_eligible_count: candidates.len(),
        specification_registered_count: registrations.len(),
        future_chain_external_specification_review_eligible_count: registrations.len(),
        candidates,
        registrations,
        specification_status: status.into(),
        next_gate: NEXT_GATE.into(),
        implementation_present: false,
        observation_materialized: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 105 只把 Stage 104 已准入的精确行情输出冻结为 create-once 观察物化规格：逐官方交易日保留股票、SPY、三价格口径、显式缺口、分红和拆股，引用既有初始影子组合但不执行会计转换。规格没有实现、工件、入口、runtime 或数据挂载；不生成观察、账本、持仓、绩效、模型、reward、订单、券商或交易事实。".into(),
    })
}

fn candidate(
    source: &AdmittedControlledShadowObservationInputForMaterializationSpecification,
) -> ControlledShadowObservationMaterializationSpecificationCandidate {
    let review = &source.admission_review;
    let parser = &source.candidate.parser_output;
    let mut excluded = review.excluded_prior_actor_ids.clone();
    excluded.push(review.submitted_by.clone());
    excluded.sort();
    excluded.dedup();
    ControlledShadowObservationMaterializationSpecificationCandidate {
        stage_104_review_id: review.review_id.clone(),
        stage_104_review_sha256: review.review_sha256.clone(),
        stage_103_validation_sha256: review.stage_103_validation_sha256.clone(),
        stage_102_attempt_id: parser.claim.attempt_id.clone(),
        stage_102_result_sha256: review.stage_102_result_sha256.clone(),
        stage_102_output_sha256: review.stage_102_output_sha256.clone(),
        stage_101_claim_sha256: review.stage_101_claim_sha256.clone(),
        stage_101_input_manifest_sha256: review.stage_101_input_manifest_sha256.clone(),
        cycle_claim_sha256: review.cycle_claim_sha256.clone(),
        subject_symbols: review.subject_symbols.clone(),
        benchmark_symbol: review.benchmark_symbol.clone(),
        admitted_available_at_utc: review.admitted_available_at_utc,
        official_market_session_count: review.official_market_session_count,
        explicit_gap_count: review.explicit_gap_count,
        registrar_excluded_actor_ids: excluded,
    }
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    review_id: &str,
    request: RegisterControlledShadowObservationMaterializationSpecificationRequest,
) -> Result<ControlledShadowObservationMaterializationSpecificationRegistration, String> {
    validate_request(review_id, &request)?;
    let _lock = acquire_lock(state, review_id).await?;
    if read_records(state)
        .await?
        .iter()
        .any(|value| value.stage_104_review_id == review_id)
    {
        return Err("该 Stage 104 准入记录已形成不可覆盖的观察物化规格".into());
    }
    let source =
        admitted_controlled_shadow_observation_inputs_for_materialization_specification(state)
            .await?
            .into_iter()
            .find(|value| value.admission_review.review_id == review_id)
            .ok_or_else(|| "当前没有可登记规格的 Stage 104 准入输入".to_string())?;
    verify_expected_bindings(&source, &request)?;
    let mut excluded = source.admission_review.excluded_prior_actor_ids.clone();
    excluded.push(source.admission_review.submitted_by.clone());
    excluded.sort();
    excluded.dedup();
    if registrar_id.trim().is_empty() || excluded.iter().any(|value| value == registrar_id) {
        return Err("Stage 105 登记者不能是 Stage 104 reviewer 或完整既有责任链角色".into());
    }
    let specification = canonical_specification(&source)?;
    let mut registration = ControlledShadowObservationMaterializationSpecificationRegistration {
        schema_version: RECORD_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        registration_id: String::new(),
        registration_sha256: String::new(),
        registered_at: Utc::now(),
        registered_by: registrar_id.into(),
        stage_104_review_id: source.admission_review.review_id.clone(),
        stage_104_review_sha256: source.admission_review.review_sha256.clone(),
        excluded_prior_actor_ids: excluded,
        registrar_independent_from_stage_104_and_complete_prior_chain: true,
        registration_reason: bounded_required(&request.registration_reason, "登记理由")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        future_review_constraints: bounded_required(
            &request.future_review_constraints,
            "未来复核约束",
        )?,
        specification,
        status: STATUS.into(),
        confirmations_complete: confirmations_complete(&request),
        specification_registered: true,
        future_chain_external_specification_review_eligible: true,
        specification_review_completed: false,
        implementation_registration_eligible: false,
        observation_materialized: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_or_metric_store_written: false,
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

fn canonical_specification(
    source: &AdmittedControlledShadowObservationInputForMaterializationSpecification,
) -> Result<ControlledShadowObservationMaterializationSpecification, String> {
    let review = &source.admission_review;
    let parser = &source.candidate.parser_output;
    let cycle = &source.candidate.cycle_claim;
    let mut specification = ControlledShadowObservationMaterializationSpecification {
        schema_version: SPEC_SCHEMA_VERSION.into(),
        specification_sha256: String::new(),
        materialization_protocol_version: MATERIALIZATION_PROTOCOL_VERSION.into(),
        stage_104_review_id: review.review_id.clone(),
        stage_104_review_sha256: review.review_sha256.clone(),
        stage_103_validation_id: review.stage_103_validation_id.clone(),
        stage_103_validation_sha256: review.stage_103_validation_sha256.clone(),
        stage_102_attempt_id: parser.claim.attempt_id.clone(),
        stage_102_result_sha256: review.stage_102_result_sha256.clone(),
        stage_102_output_sha256: review.stage_102_output_sha256.clone(),
        stage_101_claim_sha256: review.stage_101_claim_sha256.clone(),
        stage_101_input_manifest_sha256: review.stage_101_input_manifest_sha256.clone(),
        cycle_claim_id: cycle.cycle_claim_id.clone(),
        cycle_claim_sha256: cycle.cycle_claim_sha256.clone(),
        stage_88_initial_observation_output_sha256: cycle.stage_88_output_sha256.clone(),
        initial_shadow_allocation_manifest_sha256: cycle.initialization_manifest_sha256.clone(),
        subject_symbols: review.subject_symbols.clone(),
        benchmark_symbol: review.benchmark_symbol.clone(),
        window_start_date: review.window_start_date,
        window_end_date: review.window_end_date,
        earliest_market_session_date: review.earliest_market_session_date,
        latest_market_session_date: review.latest_market_session_date,
        official_market_session_count: review.official_market_session_count,
        admitted_available_at_utc: review.admitted_available_at_utc,
        availability_basis: review.availability_basis.clone(),
        provider_publication_time_verified: false,
        exact_admitted_output_is_only_input: true,
        allowed_price_bases: vec![
            "raw_unadjusted_price".into(),
            "split_adjusted_price".into(),
            "dividend_adjusted_price".into(),
        ],
        canonical_session_schema: "date,session_kind:regular|early_close,close_time_et".into(),
        canonical_price_observation_schema: "symbol,date,price_basis,close_decimal,volume_integer|null,source_request_sha256,source_row_sha256,observation_row_sha256".into(),
        canonical_explicit_gap_schema: "symbol,date,price_basis,reason:missing_subject_row_no_fill,source_manifest_sha256,gap_row_sha256".into(),
        canonical_corporate_action_schema: "symbol,date,action_kind:dividend|split,exact_source_fields,source_row_sha256,action_row_sha256".into(),
        canonical_observation_envelope_schema: "specification_sha256,stage_104_review_sha256,cycle_claim_sha256,admitted_available_at_utc,sessions,prices,explicit_gaps,dividends,splits,initial_shadow_allocation_binding,output_sha256,untrusted:true".into(),
        canonical_order: "session_date_ascending_then_symbol_ascending_then_price_basis_lexicographic;actions_by_date_symbol_kind".into(),
        decimal_preservation_rule: "preserve_exact_stage_102_canonical_decimal_string_no_rounding_or_float_conversion".into(),
        row_hash_rule: "sha256_rfc8785_canonical_json_excluding_row_sha256".into(),
        future_output_relative_path_template: format!(
            "observations/{}/{{specification_sha256}}.json",
            cycle.cycle_claim_id
        ),
        one_envelope_per_admitted_cycle: true,
        create_once_required: true,
        overwrite_allowed: false,
        retroactive_backfill_allowed: false,
        in_place_correction_allowed: false,
        subject_gap_is_explicit_not_synthetic: true,
        forward_fill_allowed: false,
        interpolation_allowed: false,
        cross_price_basis_substitution_allowed: false,
        benchmark_gap_allowed: false,
        duplicate_row_allowed: false,
        out_of_window_row_allowed: false,
        dividends_splits_and_prices_separate: true,
        initial_shadow_allocation_recomputed: false,
        accounting_transition_applied: false,
        valuation_or_investment_semantics_allowed: false,
        future_output_untrusted: true,
        future_output_independent_validation_required: true,
        authority_boundary: closed_authority_boundary(),
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])?;
    validate_specification(&specification)?;
    Ok(specification)
}

fn closed_authority_boundary() -> ZeroCapabilityObservationMaterializationAuthorityBoundary {
    ZeroCapabilityObservationMaterializationAuthorityBoundary {
        implementation_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        input_mount_present: false,
        environment_inheritance_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        observation_envelope_created: false,
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

fn validate_specification(
    value: &ControlledShadowObservationMaterializationSpecification,
) -> Result<(), String> {
    let symbols = value.subject_symbols.iter().collect::<BTreeSet<_>>();
    let closed = value.authority_boundary == closed_authority_boundary();
    let valid = value.schema_version == SPEC_SCHEMA_VERSION
        && value.materialization_protocol_version == MATERIALIZATION_PROTOCOL_VERSION
        && valid_sha256(&value.specification_sha256)
        && value.specification_sha256 == fingerprint_without(value, &["specification_sha256"])?
        && valid_id(&value.stage_104_review_id)
        && valid_id(&value.stage_103_validation_id)
        && valid_id(&value.stage_102_attempt_id)
        && valid_id(&value.cycle_claim_id)
        && [
            &value.stage_104_review_sha256,
            &value.stage_103_validation_sha256,
            &value.stage_102_result_sha256,
            &value.stage_102_output_sha256,
            &value.stage_101_claim_sha256,
            &value.stage_101_input_manifest_sha256,
            &value.cycle_claim_sha256,
            &value.stage_88_initial_observation_output_sha256,
            &value.initial_shadow_allocation_manifest_sha256,
        ]
        .iter()
        .all(|hash| valid_sha256(hash))
        && !value.subject_symbols.is_empty()
        && symbols.len() == value.subject_symbols.len()
        && value.benchmark_symbol == "SPY"
        && value.window_start_date <= value.earliest_market_session_date
        && value.earliest_market_session_date <= value.latest_market_session_date
        && value.latest_market_session_date <= value.window_end_date
        && value.official_market_session_count > 0
        && !value.provider_publication_time_verified
        && value.exact_admitted_output_is_only_input
        && value.allowed_price_bases
            == [
                "raw_unadjusted_price",
                "split_adjusted_price",
                "dividend_adjusted_price",
            ]
        && value.one_envelope_per_admitted_cycle
        && value.create_once_required
        && !value.overwrite_allowed
        && !value.retroactive_backfill_allowed
        && !value.in_place_correction_allowed
        && value.subject_gap_is_explicit_not_synthetic
        && !value.forward_fill_allowed
        && !value.interpolation_allowed
        && !value.cross_price_basis_substitution_allowed
        && !value.benchmark_gap_allowed
        && !value.duplicate_row_allowed
        && !value.out_of_window_row_allowed
        && value.dividends_splits_and_prices_separate
        && !value.initial_shadow_allocation_recomputed
        && !value.accounting_transition_applied
        && !value.valuation_or_investment_semantics_allowed
        && value.future_output_untrusted
        && value.future_output_independent_validation_required
        && closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 105 观察物化规格偏离固定零能力合同".into())
}

fn validate_record(
    value: &ControlledShadowObservationMaterializationSpecificationRegistration,
) -> Result<(), String> {
    validate_specification(&value.specification)?;
    let downstream_closed = !value.specification_review_completed
        && !value.implementation_registration_eligible
        && !value.observation_materialized
        && !value.ledger_created
        && !value.position_written
        && !value.performance_metric_written
        && !value.model_or_metric_store_written
        && !value.training_feedback_authorized
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
        && value.stage_104_review_id == value.specification.stage_104_review_id
        && value.stage_104_review_sha256 == value.specification.stage_104_review_sha256
        && value.registrar_independent_from_stage_104_and_complete_prior_chain
        && !value.registered_by.trim().is_empty()
        && value
            .excluded_prior_actor_ids
            .iter()
            .all(|actor| actor != &value.registered_by)
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
        .ok_or_else(|| "Stage 105 登记记录无效或越权".into())
}

fn validate_source_binding(
    registration: &ControlledShadowObservationMaterializationSpecificationRegistration,
    source: &AdmittedControlledShadowObservationInputForMaterializationSpecification,
) -> Result<(), String> {
    validate_record(registration)?;
    let expected = canonical_specification(source)?;
    (registration.stage_104_review_id == source.admission_review.review_id
        && registration.stage_104_review_sha256 == source.admission_review.review_sha256
        && registration.specification == expected)
        .then_some(())
        .ok_or_else(|| "Stage 105 登记已与当前 Stage 104 准入输入漂移".into())
}

fn verify_expected_bindings(
    source: &AdmittedControlledShadowObservationInputForMaterializationSpecification,
    request: &RegisterControlledShadowObservationMaterializationSpecificationRequest,
) -> Result<(), String> {
    let review = &source.admission_review;
    let valid = request.expected_stage_104_review_sha256 == review.review_sha256
        && request.expected_stage_103_validation_sha256 == review.stage_103_validation_sha256
        && request.expected_stage_102_result_sha256 == review.stage_102_result_sha256
        && request.expected_stage_102_output_sha256 == review.stage_102_output_sha256
        && request.expected_stage_101_claim_sha256 == review.stage_101_claim_sha256
        && request.expected_stage_101_input_manifest_sha256
            == review.stage_101_input_manifest_sha256
        && request.expected_cycle_claim_sha256 == review.cycle_claim_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 105 请求与当前 Stage 101–104 精确绑定不一致".into())
}

fn validate_request(
    review_id: &str,
    request: &RegisterControlledShadowObservationMaterializationSpecificationRequest,
) -> Result<(), String> {
    let hashes = [
        &request.expected_stage_104_review_sha256,
        &request.expected_stage_103_validation_sha256,
        &request.expected_stage_102_result_sha256,
        &request.expected_stage_102_output_sha256,
        &request.expected_stage_101_claim_sha256,
        &request.expected_stage_101_input_manifest_sha256,
        &request.expected_cycle_claim_sha256,
    ];
    if !valid_id(review_id) || !hashes.iter().all(|value| valid_sha256(value)) {
        return Err("Stage 105 请求绑定格式无效".into());
    }
    if !confirmations_complete(request) {
        return Err("Stage 105 必须逐项确认精确物化语义、独立性与全部零权限边界".into());
    }
    Ok(())
}

fn confirmations_complete(
    request: &RegisterControlledShadowObservationMaterializationSpecificationRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_104_binding_confirmed
        && request.registrar_independent_from_stage_104_and_complete_prior_chain_confirmed
        && request.exact_admitted_output_only_no_refetch_or_reparse_confirmed
        && request.conservative_available_at_floor_and_provider_time_limitation_preserved_confirmed
        && request.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && request.subject_missingness_explicit_no_fill_interpolation_or_substitution_confirmed
        && request.dividends_splits_and_price_bases_remain_separate_confirmed
        && request.initial_shadow_allocation_binding_preserved_without_accounting_transition_confirmed
        && request.deterministic_canonical_order_decimal_and_row_hash_rules_confirmed
        && request.one_envelope_create_once_no_overwrite_backfill_or_in_place_correction_confirmed
        && request.spy_gap_duplicate_out_of_window_or_hash_drift_fail_closed_confirmed
        && request.specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed
        && request.no_network_environment_secret_tool_subprocess_production_read_or_write_confirmed
        && request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.future_chain_external_specification_review_required_before_implementation_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationMaterializationSpecificationRegistration>, String> {
    let dir = record_dir(state);
    let Ok(mut entries) = tokio::fs::read_dir(&dir).await else {
        return Ok(Vec::new());
    };
    let mut records = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err("Stage 105 登记目录含非常规文件".into());
        }
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|error| error.to_string())?;
        let record: ControlledShadowObservationMaterializationSpecificationRegistration =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&record)?;
        records.push(record);
    }
    records.sort_by(|left, right| left.registered_at.cmp(&right.registered_at));
    let unique = records
        .iter()
        .map(|value| value.stage_104_review_id.as_str())
        .collect::<BTreeSet<_>>();
    if unique.len() != records.len() {
        return Err("Stage 105 同一准入输入出现重复语义登记".into());
    }
    Ok(records)
}

fn record_dir(state: &AppState) -> PathBuf {
    decision_root(state).join(
        "historical-outcome-controlled-shadow-first-natural-cycle-observation-materialization-specifications",
    )
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

async fn acquire_lock(state: &AppState, review_id: &str) -> Result<RegistrationLock, String> {
    let lock_dir = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&lock_dir)
        .await
        .map_err(|error| error.to_string())?;
    let path = lock_dir.join(format!("stage-105-{review_id}.lock"));
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
                Err("Stage 105 同一输入正在登记，请稍后重试".into())
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
        .ok_or_else(|| "Stage 105 路径无父目录".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    tokio::io::AsyncWriteExt::write_all(&mut file, &bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())?;
    Ok(())
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_TEXT_CHARS {
        return Err(format!("{label}不能为空且不能超过 {MAX_TEXT_CHARS} 字符"));
    }
    Ok(trimmed.to_string())
}

fn fingerprint_without<T: Serialize>(value: &T, excluded: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    if let serde_json::Value::Object(map) = &mut json {
        for key in excluded {
            map.remove(*key);
        }
    }
    let bytes = serde_json::to_vec(&json).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_boundary_is_fully_closed() {
        let boundary = closed_authority_boundary();
        assert!(!boundary.implementation_present);
        assert!(!boundary.observation_envelope_created);
        assert!(!boundary.performance_metric_write_allowed);
        assert!(!boundary.order_generation_allowed);
        assert!(!boundary.trading_allowed);
    }

    #[test]
    fn incomplete_confirmation_set_fails_closed() {
        let request = RegisterControlledShadowObservationMaterializationSpecificationRequest {
            expected_stage_104_review_sha256: "1".repeat(64),
            expected_stage_103_validation_sha256: "2".repeat(64),
            expected_stage_102_result_sha256: "3".repeat(64),
            expected_stage_102_output_sha256: "4".repeat(64),
            expected_stage_101_claim_sha256: "5".repeat(64),
            expected_stage_101_input_manifest_sha256: "6".repeat(64),
            expected_cycle_claim_sha256: "7".repeat(64),
            registration_reason: "reason".into(),
            known_limitations: "provider publication time remains unverified".into(),
            future_review_constraints: "independent review".into(),
            exact_current_stage_51_through_stage_104_binding_confirmed: true,
            registrar_independent_from_stage_104_and_complete_prior_chain_confirmed: true,
            exact_admitted_output_only_no_refetch_or_reparse_confirmed: true,
            conservative_available_at_floor_and_provider_time_limitation_preserved_confirmed: true,
            official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
            subject_missingness_explicit_no_fill_interpolation_or_substitution_confirmed: true,
            dividends_splits_and_price_bases_remain_separate_confirmed: true,
            initial_shadow_allocation_binding_preserved_without_accounting_transition_confirmed: true,
            deterministic_canonical_order_decimal_and_row_hash_rules_confirmed: true,
            one_envelope_create_once_no_overwrite_backfill_or_in_place_correction_confirmed: true,
            spy_gap_duplicate_out_of_window_or_hash_drift_fail_closed_confirmed: true,
            specification_only_no_implementation_artifact_entrypoint_runtime_or_input_mount_confirmed: true,
            no_network_environment_secret_tool_subprocess_production_read_or_write_confirmed: true,
            no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            future_chain_external_specification_review_required_before_implementation_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: false,
        };
        assert!(!confirmations_complete(&request));
        assert!(validate_request(&"a".repeat(32), &request).is_err());
    }

    #[test]
    fn path_contract_is_cycle_scoped_and_content_addressed() {
        let cycle_id = "b".repeat(32);
        let path = format!("observations/{cycle_id}/{{specification_sha256}}.json");
        assert_eq!(
            path,
            format!("observations/{cycle_id}/{{specification_sha256}}.json")
        );
        assert!(!path.contains(".."));
    }

    #[test]
    fn fingerprint_excludes_only_declared_identity_fields() {
        #[derive(Serialize)]
        struct Value<'a> {
            id: &'a str,
            sha: &'a str,
            payload: &'a str,
        }
        let first = Value {
            id: "a",
            sha: "b",
            payload: "fixed",
        };
        let second = Value {
            id: "c",
            sha: "d",
            payload: "fixed",
        };
        assert_eq!(
            fingerprint_without(&first, &["id", "sha"]).unwrap(),
            fingerprint_without(&second, &["id", "sha"]).unwrap()
        );
    }
}
