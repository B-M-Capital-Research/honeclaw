//! Create-once registration of a zero-capability implementation specification for one exact
//! independently approved Stage 75 controlled shadow-experiment design.
//!
//! Stage 76 freezes deterministic replay semantics and future input/output envelopes only. It
//! deliberately has no callable entrypoint, runtime, data adapter, ledger, position, order,
//! broker connection, production access or trading authority. Independent implementation review
//! and every possible execution gate remain separate future stages.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registration_reviews::{
    ControlledShadowExperimentDesignRegistrationReview,
    ControlledShadowExperimentDesignRegistrationReviewVerdict,
    IndependentlyReviewedControlledShadowExperimentDesign,
    independently_reviewed_controlled_shadow_experiment_designs_for_implementation_registration,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registrations::{
    ControlledShadowExperimentDesignRegistration, ControlledShadowExperimentDesignSpecification,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-implementation-registry-v1";
const RECORD_SCHEMA_VERSION: &str = "hone-controlled-shadow-experiment-implementation-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-zero-capability-implementation-contract-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-experiment-implementation-create-once-v1-zero-capability";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-forward-replay-specification-v1-not-executable";
const STATUS: &str = "registered_not_reviewed_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowExperimentImplementationRequest {
    expected_design_review_id: String,
    expected_design_review_sha256: String,
    expected_design_registration_id: String,
    expected_design_registration_sha256: String,
    expected_design_specification_sha256: String,
    expected_adjudication_review_sha256: String,
    expected_output_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_envelope_sha256: String,
    expected_candidate_set_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    expected_sealed_holdout_projection_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_description: String,
    deterministic_replay_notes: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_current_stage_51_through_stage_75_binding_confirmed: bool,
    registrar_independent_from_stage_75_and_complete_prior_chain_confirmed: bool,
    independent_recomputation_of_design_review_registration_and_specification_confirmed: bool,
    zero_capability_specification_only_not_executable_artifact_confirmed: bool,
    point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed: bool,
    signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed: bool,
    long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed: bool,
    observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed: bool,
    deterministic_create_once_content_addressed_replay_contract_confirmed: bool,
    no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    no_model_store_metric_store_training_feedback_composite_or_reward_confirmed: bool,
    no_shadow_run_ledger_position_order_broker_or_trading_confirmed: bool,
    future_independent_implementation_review_required_before_runner_registration_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_protocol_version: String,
    pub immutable_code_revision: String,
    pub stage_75_design_review_id: String,
    pub stage_75_design_review_sha256: String,
    pub stage_74_design_registration_id: String,
    pub stage_74_design_registration_sha256: String,
    pub design_specification_sha256: String,
    pub adjudication_review_sha256: String,
    pub output_validation_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub envelope_sha256: String,
    pub candidate_set_sha256: String,
    pub training_store_dataset_sha256: String,
    pub selected_algorithm_three_seed_binding_sha256: String,
    pub sealed_holdout_split_commitment_sha256: String,
    pub sealed_holdout_projection_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub random_seeds: Vec<u64>,
    pub exact_design_specification: ControlledShadowExperimentDesignSpecification,
    pub deterministic_signal_projection_function_id: String,
    pub deterministic_portfolio_transition_function_id: String,
    pub deterministic_execution_and_cost_function_id: String,
    pub deterministic_comparator_synchronization_function_id: String,
    pub deterministic_checkpoint_and_stop_function_id: String,
    pub canonical_future_input_schema: String,
    pub canonical_future_untrusted_output_schema: String,
    pub future_inputs_read_only_and_point_in_time: bool,
    pub future_output_create_once: bool,
    pub future_output_untrusted: bool,
    pub future_output_independent_validation_required: bool,
    pub future_output_contains_order_intent: bool,
    pub future_output_contains_broker_payload: bool,
    pub registered_not_run: bool,
    pub independent_implementation_review_required: bool,
    pub isolated_runner_registration_required: bool,
    pub callable_entrypoint_present: bool,
    pub executable_artifact_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub market_data_adapter_present: bool,
    pub environment_inheritance_allowed: bool,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub training_feedback_allowed: bool,
    pub scalar_reward_defined: bool,
    pub shadow_run_allowed: bool,
    pub shadow_ledger_creation_allowed: bool,
    pub shadow_position_write_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub upstream_design_registration: ControlledShadowExperimentDesignRegistration,
    pub upstream_design_review: ControlledShadowExperimentDesignRegistrationReview,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_75_and_complete_prior_chain: bool,
    pub implementation_name: String,
    pub implementation_description: String,
    pub deterministic_replay_notes: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub implementation_contract: ControlledShadowExperimentImplementationContract,
    pub status: String,
    pub exact_current_stage_51_through_stage_75_binding_confirmed: bool,
    pub registrar_independent_from_stage_75_and_complete_prior_chain_confirmed: bool,
    pub independent_recomputation_of_design_review_registration_and_specification_confirmed: bool,
    pub zero_capability_specification_only_not_executable_artifact_confirmed: bool,
    pub point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed: bool,
    pub signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed:
        bool,
    pub long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed: bool,
    pub observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed:
        bool,
    pub deterministic_create_once_content_addressed_replay_contract_confirmed: bool,
    pub no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    pub no_model_store_metric_store_training_feedback_composite_or_reward_confirmed: bool,
    pub no_shadow_run_ledger_position_order_broker_or_trading_confirmed: bool,
    pub future_independent_implementation_review_required_before_runner_registration_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_shadow_implementation_registered: bool,
    pub future_independent_shadow_implementation_review_eligible: bool,
    pub independent_shadow_implementation_review_completed: bool,
    pub isolated_runner_registration_eligible: bool,
    pub official_model_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_run_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentImplementationEligibleDesign {
    pub design_registration: ControlledShadowExperimentDesignRegistration,
    pub design_review: ControlledShadowExperimentDesignRegistrationReview,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentImplementationItem {
    pub implementation: ControlledShadowExperimentImplementationRecord,
    pub upstream_binding_current: bool,
    pub future_independent_shadow_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_designs: Vec<ControlledShadowExperimentImplementationEligibleDesign>,
    pub items: Vec<ControlledShadowExperimentImplementationItem>,
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub callable_entrypoint_present: bool,
    pub executable_artifact_present: bool,
    pub runtime_present: bool,
    pub shadow_run_authorized: bool,
    pub shadow_ledger_enabled: bool,
    pub shadow_position_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowExperimentImplementationReadinessSummary {
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_experiment_implementations(
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
            warn!(%error, "controlled shadow implementation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "受控影子实验零能力实现登记暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_experiment_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowExperimentImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_experiment_implementation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowExperimentImplementationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(ControlledShadowExperimentImplementationReadinessSummary {
        registration_eligible_count: registry.registration_eligible_count,
        implementation_count: registry.implementation_count,
        current_binding_implementation_count: registry.current_binding_implementation_count,
        independent_implementation_review_eligible_count: registry
            .independent_implementation_review_eligible_count,
        implementation_status: registry.implementation_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn independently_reviewable_controlled_shadow_experiment_implementations(
    state: &AppState,
) -> Result<Vec<ControlledShadowExperimentImplementationRecord>, String> {
    let registry = build_registry(state).await?;
    let values = registry
        .items
        .into_iter()
        .filter(|item| item.future_independent_shadow_implementation_review_eligible)
        .map(|item| item.implementation)
        .collect::<Vec<_>>();
    for value in &values {
        validate_record(value)?;
    }
    Ok(values)
}

#[allow(dead_code)] // Consumed by the separately gated Stage 77 independent review registry.
pub(crate) fn validate_controlled_shadow_experiment_implementation_for_review(
    value: &ControlledShadowExperimentImplementationRecord,
) -> Result<(), String> {
    validate_record(value)?;
    if !value.future_independent_shadow_implementation_review_eligible
        || value.independent_shadow_implementation_review_completed
        || value.isolated_runner_registration_eligible
    {
        return Err("受控影子零能力实现不处于独立复核前状态".to_string());
    }
    Ok(())
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowExperimentImplementationRegistry, String> {
    let eligible = eligible_sources(state).await?;
    let eligible_by_review = eligible
        .iter()
        .map(|source| (source.design_review.review_id.as_str(), source))
        .collect::<HashMap<_, _>>();
    let mut items = read_records(state)
        .await?
        .into_iter()
        .map(|implementation| {
            let upstream_binding_current = eligible_by_review
                .get(implementation.upstream_design_review.review_id.as_str())
                .is_some_and(|source| record_matches_source(&implementation, source));
            ControlledShadowExperimentImplementationItem {
                future_independent_shadow_implementation_review_eligible: upstream_binding_current,
                upstream_binding_current,
                implementation,
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        right
            .implementation
            .registered_at
            .cmp(&left.implementation.registered_at)
            .then_with(|| {
                right
                    .implementation
                    .implementation_id
                    .cmp(&left.implementation.implementation_id)
            })
    });
    let registered_review_ids = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .map(|item| {
            item.implementation
                .upstream_design_review
                .review_id
                .as_str()
        })
        .collect::<BTreeSet<_>>();
    let registration_eligible_count = eligible
        .iter()
        .filter(|source| !registered_review_ids.contains(source.design_review.review_id.as_str()))
        .count();
    let current_binding_implementation_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_shadow_implementation_review_eligible)
        .count();
    let implementation_status = if current_binding_implementation_count > 0 {
        "zero_capability_shadow_implementation_registered_not_run_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "independently_approved_shadow_design_ready_for_zero_capability_implementation_registration"
    } else {
        "waiting_stage_75_independently_approved_shadow_design"
    };
    Ok(ControlledShadowExperimentImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_designs: eligible,
        registration_eligible_count,
        implementation_count: items.len(),
        current_binding_implementation_count,
        independent_implementation_review_eligible_count,
        items,
        implementation_status: implementation_status.to_string(),
        callable_entrypoint_present: false,
        executable_artifact_present: false,
        runtime_present: false,
        shadow_run_authorized: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 76 阶段只登记当前 Stage 75 独立批准设计的内容寻址、零能力实现规格。规格冻结点时输入、确定性信号/组合/成本/反事实/检查点/停止语义和未来不可信输出信封；没有可执行工件、入口、runtime、环境、网络、生产读写、影子账本、持仓、订单、券商或交易。批准后仅可进入独立实现复核，runner 与任何运行授权仍是后续独立门禁。".to_string(),
    })
}

async fn eligible_sources(
    state: &AppState,
) -> Result<Vec<ControlledShadowExperimentImplementationEligibleDesign>, String> {
    independently_reviewed_controlled_shadow_experiment_designs_for_implementation_registration(
        state,
    )
    .await?
    .into_iter()
    .map(source_from_reviewed)
    .collect()
}

fn source_from_reviewed(
    value: IndependentlyReviewedControlledShadowExperimentDesign,
) -> Result<ControlledShadowExperimentImplementationEligibleDesign, String> {
    let source = ControlledShadowExperimentImplementationEligibleDesign {
        design_registration: value.registered_design.registration,
        design_review: value.review,
    };
    validate_source(&source)?;
    Ok(source)
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    attempt_id: &str,
    request: RegisterControlledShadowExperimentImplementationRequest,
) -> Result<ControlledShadowExperimentImplementationRecord, String> {
    if registrar_id.trim().is_empty() {
        return Err("受控影子实现登记人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let eligible = eligible_sources(state).await?;
    let source = eligible
        .iter()
        .find(|source| {
            source.design_registration.attempt_id == attempt_id
                && source.design_review.review_id == request.expected_design_review_id
        })
        .ok_or_else(|| "当前 Stage 75 独立批准设计不存在，请刷新后重试".to_string())?;
    ensure_source_unregistered(&read_records(state).await?, source)?;
    let record = build_record(source, registrar_id, request, Utc::now())?;
    write_immutable_json(
        &record_root(state).join(format!("{}.json", record.implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn build_record(
    source: &ControlledShadowExperimentImplementationEligibleDesign,
    registrar_id: &str,
    request: RegisterControlledShadowExperimentImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<ControlledShadowExperimentImplementationRecord, String> {
    validate_source(source)?;
    validate_expected_bindings(source, &request)?;
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
    let deterministic_replay_notes = bounded_required(
        &request.deterministic_replay_notes,
        MAX_TEXT_CHARS,
        "确定性重放说明",
    )?;
    let known_limitations =
        bounded_required(&request.known_limitations, MAX_TEXT_CHARS, "已知局限")?;
    let future_review_constraints = bounded_required(
        &request.future_review_constraints,
        MAX_TEXT_CHARS,
        "后续复核约束",
    )?;
    if !request_checks_confirmed(&request) {
        return Err("登记前必须逐项确认完整绑定、确定性语义、零能力和职责分离".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(source);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == registrar_id)
    {
        return Err("实现登记人参与过 Stage 51–75 或完整上游，必须更换独立角色".to_string());
    }
    let implementation_contract = implementation_contract(source, &immutable_code_revision)?;
    let mut record = ControlledShadowExperimentImplementationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        upstream_design_registration: source.design_registration.clone(),
        upstream_design_review: source.design_review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_75_and_complete_prior_chain: true,
        implementation_name,
        implementation_description,
        deterministic_replay_notes,
        known_limitations,
        future_review_constraints,
        implementation_contract,
        status: STATUS.to_string(),
        exact_current_stage_51_through_stage_75_binding_confirmed: request
            .exact_current_stage_51_through_stage_75_binding_confirmed,
        registrar_independent_from_stage_75_and_complete_prior_chain_confirmed: request
            .registrar_independent_from_stage_75_and_complete_prior_chain_confirmed,
        independent_recomputation_of_design_review_registration_and_specification_confirmed:
            request.independent_recomputation_of_design_review_registration_and_specification_confirmed,
        zero_capability_specification_only_not_executable_artifact_confirmed: request
            .zero_capability_specification_only_not_executable_artifact_confirmed,
        point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed: request
            .point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed,
        signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed:
            request.signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed,
        long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed: request
            .long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed,
        observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed:
            request.observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed,
        deterministic_create_once_content_addressed_replay_contract_confirmed: request
            .deterministic_create_once_content_addressed_replay_contract_confirmed,
        no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed:
            request.no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_model_store_metric_store_training_feedback_composite_or_reward_confirmed: request
            .no_model_store_metric_store_training_feedback_composite_or_reward_confirmed,
        no_shadow_run_ledger_position_order_broker_or_trading_confirmed: request
            .no_shadow_run_ledger_position_order_broker_or_trading_confirmed,
        future_independent_implementation_review_required_before_runner_registration_confirmed:
            request.future_independent_implementation_review_required_before_runner_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_shadow_implementation_registered: true,
        future_independent_shadow_implementation_review_eligible: true,
        independent_shadow_implementation_review_completed: false,
        isolated_runner_registration_eligible: false,
        official_model_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        shadow_run_authorized: false,
        shadow_ledger_enabled: false,
        shadow_position_written: false,
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
    source: &ControlledShadowExperimentImplementationEligibleDesign,
    immutable_code_revision: &str,
) -> Result<ControlledShadowExperimentImplementationContract, String> {
    let registration = &source.design_registration;
    let review = &source.design_review;
    let mut contract = ControlledShadowExperimentImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_protocol_version: IMPLEMENTATION_PROTOCOL_VERSION.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        stage_75_design_review_id: review.review_id.clone(),
        stage_75_design_review_sha256: review.review_sha256.clone(),
        stage_74_design_registration_id: registration.registration_id.clone(),
        stage_74_design_registration_sha256: registration.registration_sha256.clone(),
        design_specification_sha256: registration
            .design_specification
            .specification_sha256
            .clone(),
        adjudication_review_sha256: registration.adjudication_review_sha256.clone(),
        output_validation_sha256: registration.output_validation_sha256.clone(),
        claim_sha256: registration.claim_sha256.clone(),
        result_sha256: registration.result_sha256.clone(),
        output_sha256: registration.output_sha256.clone(),
        envelope_sha256: registration.envelope_sha256.clone(),
        candidate_set_sha256: registration.candidate_set_sha256.clone(),
        training_store_dataset_sha256: registration.training_store_dataset_sha256.clone(),
        selected_algorithm_three_seed_binding_sha256: registration
            .selected_algorithm_three_seed_binding_sha256
            .clone(),
        sealed_holdout_split_commitment_sha256: registration
            .sealed_holdout_split_commitment_sha256
            .clone(),
        sealed_holdout_projection_sha256: registration.sealed_holdout_projection_sha256.clone(),
        feature_order_sha256: registration.feature_order_sha256.clone(),
        preprocessing_sha256: registration.preprocessing_sha256.clone(),
        target_id: registration.target_id.clone(),
        frozen_candidate_algorithm_id: registration.frozen_candidate_algorithm_id.clone(),
        random_seeds: registration.design_specification.random_seeds.clone(),
        exact_design_specification: registration.design_specification.clone(),
        deterministic_signal_projection_function_id:
            "controlled_shadow_signal_projection_v1_pure_specification".to_string(),
        deterministic_portfolio_transition_function_id:
            "controlled_shadow_long_only_cash_constrained_transition_v1_pure_specification"
                .to_string(),
        deterministic_execution_and_cost_function_id:
            "controlled_shadow_next_session_adjusted_close_25bps_v1_pure_specification".to_string(),
        deterministic_comparator_synchronization_function_id:
            "controlled_shadow_spy_cash_equal_weight_frozen_rule_sync_v1_pure_specification"
                .to_string(),
        deterministic_checkpoint_and_stop_function_id:
            "controlled_shadow_21_63_126_252_checkpoint_stop_v1_pure_specification".to_string(),
        canonical_future_input_schema:
            "controlled_shadow_point_in_time_read_only_input_v1_not_mounted".to_string(),
        canonical_future_untrusted_output_schema:
            "controlled_shadow_checkpoint_observation_envelope_v1_no_order_intent".to_string(),
        future_inputs_read_only_and_point_in_time: true,
        future_output_create_once: true,
        future_output_untrusted: true,
        future_output_independent_validation_required: true,
        future_output_contains_order_intent: false,
        future_output_contains_broker_payload: false,
        registered_not_run: true,
        independent_implementation_review_required: true,
        isolated_runner_registration_required: true,
        callable_entrypoint_present: false,
        executable_artifact_present: false,
        runtime_present: false,
        input_mount_present: false,
        market_data_adapter_present: false,
        environment_inheritance_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        training_feedback_allowed: false,
        scalar_reward_defined: false,
        shadow_run_allowed: false,
        shadow_ledger_creation_allowed: false,
        shadow_position_write_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn validate_source(
    source: &ControlledShadowExperimentImplementationEligibleDesign,
) -> Result<(), String> {
    let registration = &source.design_registration;
    let review = &source.design_review;
    let review_approved = review.verdict
        == ControlledShadowExperimentDesignRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityShadowImplementationRegistration
        && review.design_registration_independently_approved
        && review.future_zero_capability_shadow_implementation_registration_eligible;
    if registration.registration_sha256
        != fingerprint_without(registration, &["registration_id", "registration_sha256"])?
        || registration.registration_id != registration.registration_sha256[..32]
        || registration.design_specification.specification_sha256
            != fingerprint_without(
                &registration.design_specification,
                &["specification_sha256"],
            )?
        || review.review_sha256 != fingerprint_without(review, &["review_id", "review_sha256"])?
        || review.review_id != review.review_sha256[..32]
        || review.registration_id != registration.registration_id
        || review.registration_sha256 != registration.registration_sha256
        || review.design_specification_sha256
            != registration.design_specification.specification_sha256
        || review.independently_recomputed_registration_sha256 != registration.registration_sha256
        || review.independently_recomputed_design_specification_sha256
            != registration.design_specification.specification_sha256
        || review.target_id != registration.target_id
        || review.frozen_candidate_algorithm_id != registration.frozen_candidate_algorithm_id
        || !review_approved
        || review.shadow_implementation_registered
        || review.shadow_run_authorized
        || review.shadow_ledger_enabled
        || review.shadow_position_written
        || review.order_generation_authorized
        || review.broker_access_authorized
        || review.trading_authorized
    {
        return Err("Stage 74–75 受控影子设计来源无效、漂移或越权".to_string());
    }
    Ok(())
}

fn validate_expected_bindings(
    source: &ControlledShadowExperimentImplementationEligibleDesign,
    request: &RegisterControlledShadowExperimentImplementationRequest,
) -> Result<(), String> {
    let registration = &source.design_registration;
    let review = &source.design_review;
    if request.expected_design_review_id != review.review_id
        || request.expected_design_review_sha256 != review.review_sha256
        || request.expected_design_registration_id != registration.registration_id
        || request.expected_design_registration_sha256 != registration.registration_sha256
        || request.expected_design_specification_sha256
            != registration.design_specification.specification_sha256
        || request.expected_adjudication_review_sha256 != registration.adjudication_review_sha256
        || request.expected_output_validation_sha256 != registration.output_validation_sha256
        || request.expected_claim_sha256 != registration.claim_sha256
        || request.expected_result_sha256 != registration.result_sha256
        || request.expected_output_sha256 != registration.output_sha256
        || request.expected_envelope_sha256 != registration.envelope_sha256
        || request.expected_candidate_set_sha256 != registration.candidate_set_sha256
        || request.expected_training_store_dataset_sha256
            != registration.training_store_dataset_sha256
        || request.expected_selected_algorithm_three_seed_binding_sha256
            != registration.selected_algorithm_three_seed_binding_sha256
        || request.expected_sealed_holdout_split_commitment_sha256
            != registration.sealed_holdout_split_commitment_sha256
        || request.expected_sealed_holdout_projection_sha256
            != registration.sealed_holdout_projection_sha256
        || request.expected_feature_order_sha256 != registration.feature_order_sha256
        || request.expected_preprocessing_sha256 != registration.preprocessing_sha256
        || request.expected_target_id != registration.target_id
        || request.expected_frozen_candidate_algorithm_id
            != registration.frozen_candidate_algorithm_id
    {
        return Err("Stage 74–75 设计、候选或数据绑定已变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn request_checks_confirmed(
    request: &RegisterControlledShadowExperimentImplementationRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_75_binding_confirmed
        && request.registrar_independent_from_stage_75_and_complete_prior_chain_confirmed
        && request
            .independent_recomputation_of_design_review_registration_and_specification_confirmed
        && request.zero_capability_specification_only_not_executable_artifact_confirmed
        && request.point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed
        && request.signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed
        && request.long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed
        && request.observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed
        && request.deterministic_create_once_content_addressed_replay_contract_confirmed
        && request.no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && request.no_model_store_metric_store_training_feedback_composite_or_reward_confirmed
        && request.no_shadow_run_ledger_position_order_broker_or_trading_confirmed
        && request.future_independent_implementation_review_required_before_runner_registration_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn excluded_actor_ids(
    source: &ControlledShadowExperimentImplementationEligibleDesign,
) -> Vec<String> {
    let mut values = source.design_registration.excluded_prior_actor_ids.clone();
    values.push(source.design_registration.registered_by.clone());
    values.extend(source.design_review.excluded_prior_actor_ids.clone());
    values.push(source.design_review.reviewer_id.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn validate_contract(
    contract: &ControlledShadowExperimentImplementationContract,
    registration: &ControlledShadowExperimentDesignRegistration,
    review: &ControlledShadowExperimentDesignRegistrationReview,
) -> Result<(), String> {
    let hashes_valid = [
        contract.contract_sha256.as_str(),
        contract.stage_75_design_review_sha256.as_str(),
        contract.stage_74_design_registration_sha256.as_str(),
        contract.design_specification_sha256.as_str(),
        contract.adjudication_review_sha256.as_str(),
        contract.output_validation_sha256.as_str(),
        contract.claim_sha256.as_str(),
        contract.result_sha256.as_str(),
        contract.output_sha256.as_str(),
        contract.envelope_sha256.as_str(),
        contract.candidate_set_sha256.as_str(),
        contract.training_store_dataset_sha256.as_str(),
        contract
            .selected_algorithm_three_seed_binding_sha256
            .as_str(),
        contract.sealed_holdout_split_commitment_sha256.as_str(),
        contract.sealed_holdout_projection_sha256.as_str(),
        contract.feature_order_sha256.as_str(),
        contract.preprocessing_sha256.as_str(),
    ]
    .iter()
    .all(|value| valid_sha256(value));
    let zero_capability = !contract.callable_entrypoint_present
        && !contract.executable_artifact_present
        && !contract.runtime_present
        && !contract.input_mount_present
        && !contract.market_data_adapter_present
        && !contract.environment_inheritance_allowed
        && !contract.secrets_allowed
        && !contract.outbound_network_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.model_store_write_allowed
        && !contract.metric_store_write_allowed
        && !contract.training_feedback_allowed
        && !contract.scalar_reward_defined
        && !contract.shadow_run_allowed
        && !contract.shadow_ledger_creation_allowed
        && !contract.shadow_position_write_allowed
        && !contract.order_generation_allowed
        && !contract.broker_access_allowed
        && !contract.trading_allowed
        && !contract.future_output_contains_order_intent
        && !contract.future_output_contains_broker_payload;
    let functions_present = [
        &contract.deterministic_signal_projection_function_id,
        &contract.deterministic_portfolio_transition_function_id,
        &contract.deterministic_execution_and_cost_function_id,
        &contract.deterministic_comparator_synchronization_function_id,
        &contract.deterministic_checkpoint_and_stop_function_id,
        &contract.canonical_future_input_schema,
        &contract.canonical_future_untrusted_output_schema,
    ]
    .into_iter()
    .all(|value| !value.trim().is_empty());
    if contract.schema_version != CONTRACT_SCHEMA_VERSION
        || contract.contract_sha256 != fingerprint_without(contract, &["contract_sha256"])?
        || !hashes_valid
        || contract.implementation_protocol_version != IMPLEMENTATION_PROTOCOL_VERSION
        || contract.immutable_code_revision.trim().is_empty()
        || contract.stage_75_design_review_id != review.review_id
        || contract.stage_75_design_review_sha256 != review.review_sha256
        || contract.stage_74_design_registration_id != registration.registration_id
        || contract.stage_74_design_registration_sha256 != registration.registration_sha256
        || contract.design_specification_sha256
            != registration.design_specification.specification_sha256
        || contract.adjudication_review_sha256 != registration.adjudication_review_sha256
        || contract.output_validation_sha256 != registration.output_validation_sha256
        || contract.claim_sha256 != registration.claim_sha256
        || contract.result_sha256 != registration.result_sha256
        || contract.output_sha256 != registration.output_sha256
        || contract.envelope_sha256 != registration.envelope_sha256
        || contract.candidate_set_sha256 != registration.candidate_set_sha256
        || contract.training_store_dataset_sha256 != registration.training_store_dataset_sha256
        || contract.selected_algorithm_three_seed_binding_sha256
            != registration.selected_algorithm_three_seed_binding_sha256
        || contract.sealed_holdout_split_commitment_sha256
            != registration.sealed_holdout_split_commitment_sha256
        || contract.sealed_holdout_projection_sha256
            != registration.sealed_holdout_projection_sha256
        || contract.feature_order_sha256 != registration.feature_order_sha256
        || contract.preprocessing_sha256 != registration.preprocessing_sha256
        || contract.target_id != registration.target_id
        || contract.frozen_candidate_algorithm_id != registration.frozen_candidate_algorithm_id
        || contract.random_seeds != registration.design_specification.random_seeds
        || contract.exact_design_specification != registration.design_specification
        || !contract.future_inputs_read_only_and_point_in_time
        || !contract.future_output_create_once
        || !contract.future_output_untrusted
        || !contract.future_output_independent_validation_required
        || !contract.registered_not_run
        || !contract.independent_implementation_review_required
        || !contract.isolated_runner_registration_required
        || !functions_present
        || !zero_capability
    {
        return Err("受控影子实验零能力实现合同无效、漂移或越权".to_string());
    }
    Ok(())
}

fn record_checks_confirmed(value: &ControlledShadowExperimentImplementationRecord) -> bool {
    value.exact_current_stage_51_through_stage_75_binding_confirmed
        && value.registrar_independent_from_stage_75_and_complete_prior_chain_confirmed
        && value.independent_recomputation_of_design_review_registration_and_specification_confirmed
        && value.zero_capability_specification_only_not_executable_artifact_confirmed
        && value.point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed
        && value.signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed
        && value.long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed
        && value.observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed
        && value.deterministic_create_once_content_addressed_replay_contract_confirmed
        && value.no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && value.no_model_store_metric_store_training_feedback_composite_or_reward_confirmed
        && value.no_shadow_run_ledger_position_order_broker_or_trading_confirmed
        && value.future_independent_implementation_review_required_before_runner_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_record(value: &ControlledShadowExperimentImplementationRecord) -> Result<(), String> {
    let source = ControlledShadowExperimentImplementationEligibleDesign {
        design_registration: value.upstream_design_registration.clone(),
        design_review: value.upstream_design_review.clone(),
    };
    validate_source(&source)?;
    validate_contract(
        &value.implementation_contract,
        &value.upstream_design_registration,
        &value.upstream_design_review,
    )?;
    let expected_excluded = excluded_actor_ids(&source);
    let texts_valid = [
        (&value.implementation_name, MAX_NAME_CHARS),
        (&value.implementation_description, MAX_TEXT_CHARS),
        (&value.deterministic_replay_notes, MAX_TEXT_CHARS),
        (&value.known_limitations, MAX_TEXT_CHARS),
        (&value.future_review_constraints, MAX_TEXT_CHARS),
    ]
    .into_iter()
    .all(|(text, max)| !text.trim().is_empty() && text.chars().count() <= max);
    let downstream_closed = !value.independent_shadow_implementation_review_completed
        && !value.isolated_runner_registration_eligible
        && !value.official_model_selection_authorized
        && !value.model_artifact_store_written
        && !value.metric_store_written
        && !value.training_feedback_authorized
        && !value.reward_authorized
        && !value.shadow_run_authorized
        && !value.shadow_ledger_enabled
        && !value.shadow_position_written
        && !value.order_generation_authorized
        && !value.broker_access_authorized
        && !value.trading_authorized;
    if value.schema_version != RECORD_SCHEMA_VERSION
        || value.policy_version != POLICY_VERSION
        || !valid_id(&value.implementation_id)
        || !valid_sha256(&value.implementation_sha256)
        || value.implementation_sha256 != record_fingerprint(value)?
        || value.implementation_id != value.implementation_sha256[..32]
        || value.registered_by.trim().is_empty()
        || value.excluded_prior_actor_ids != expected_excluded
        || !sorted_unique(&value.excluded_prior_actor_ids)
        || value
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &value.registered_by)
        || !value.registrar_independent_from_stage_75_and_complete_prior_chain
        || !texts_valid
        || value.status != STATUS
        || !record_checks_confirmed(value)
        || !value.zero_capability_shadow_implementation_registered
        || !value.future_independent_shadow_implementation_review_eligible
        || !downstream_closed
    {
        return Err("受控影子实验零能力实现登记无效、责任重叠或越权".to_string());
    }
    Ok(())
}

fn record_matches_source(
    record: &ControlledShadowExperimentImplementationRecord,
    source: &ControlledShadowExperimentImplementationEligibleDesign,
) -> bool {
    validate_record(record).is_ok()
        && record.upstream_design_registration == source.design_registration
        && record.upstream_design_review == source.design_review
}

fn ensure_source_unregistered(
    records: &[ControlledShadowExperimentImplementationRecord],
    source: &ControlledShadowExperimentImplementationEligibleDesign,
) -> Result<(), String> {
    if records.iter().any(|record| {
        record.upstream_design_review.review_id == source.design_review.review_id
            || record.upstream_design_registration.registration_id
                == source.design_registration.registration_id
            || record.upstream_design_registration.attempt_id
                == source.design_registration.attempt_id
    }) {
        return Err("同一 Stage 75 独立批准设计只能登记一次零能力实现".to_string());
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowExperimentImplementationRecord>, String> {
    let root = record_root(state);
    let mut records = Vec::new();
    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(records),
        Err(error) => return Err(error.to_string()),
    };
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|error| error.to_string())?;
        let value =
            serde_json::from_slice::<ControlledShadowExperimentImplementationRecord>(&bytes)
                .map_err(|error| format!("{}: {error}", path.display()))?;
        validate_record(&value)?;
        if path.file_stem().and_then(|value| value.to_str())
            != Some(value.implementation_id.as_str())
        {
            return Err("受控影子实现登记文件名与记录 ID 不一致".to_string());
        }
        records.push(value);
    }
    Ok(records)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn record_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-experiment-zero-capability-implementations")
}

struct RegistrationLock {
    path: PathBuf,
}

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("controlled-shadow-experiment-implementation-registration.lock");
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
        .map_err(|_| "已有受控影子实现登记正在进行".to_string())?;
    Ok(RegistrationLock { path })
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
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

fn record_fingerprint(
    value: &ControlledShadowExperimentImplementationRecord,
) -> Result<String, String> {
    fingerprint_without(value, &["implementation_id", "implementation_sha256"])
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "受控影子实现指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, max: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > max {
        return Err(format!("{label}不能超过 {max} 个字符"));
    }
    Ok(value.to_string())
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use super::super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registrations::{
        ControlledShadowExperimentExecutionContract, ControlledShadowExperimentMetricContract,
        ControlledShadowExperimentObservationContract, ControlledShadowExperimentPortfolioConstraints,
        ControlledShadowExperimentStopContract,
    };

    fn design_specification() -> ControlledShadowExperimentDesignSpecification {
        let mut value = ControlledShadowExperimentDesignSpecification {
            schema_version: "hone-controlled-shadow-experiment-design-spec-v1".to_string(),
            specification_sha256: String::new(),
            experimental_candidate_only: true,
            target_id: "excess_return_250d".to_string(),
            frozen_candidate_algorithm_id: "ridge".to_string(),
            random_seeds: vec![17, 29, 43],
            candidate_set_sha256: "1".repeat(64),
            feature_order_sha256: "2".repeat(64),
            preprocessing_sha256: "3".repeat(64),
            benchmark_symbol: "SPY".to_string(),
            comparator_ids: vec![
                "cash".to_string(),
                "equal_weight".to_string(),
                "frozen_rule".to_string(),
                "spy".to_string(),
            ],
            universe_contract: "point-in-time common stock with delistings".to_string(),
            signal_contract: "signal at cutoff; next full session adjusted close".to_string(),
            portfolio_constraints: ControlledShadowExperimentPortfolioConstraints {
                virtual_notional_usd: 1_000_000,
                long_only: true,
                common_stock_only: true,
                options_allowed: false,
                leverage_allowed: false,
                shorting_allowed: false,
                maximum_single_name_weight_bps: 500,
                maximum_theme_weight_bps: 2_000,
                maximum_gross_exposure_bps: 6_000,
                minimum_cash_weight_bps: 4_000,
                maximum_position_count: 10,
            },
            execution_contract: ControlledShadowExperimentExecutionContract {
                signal_cutoff: "after close".to_string(),
                assumed_execution: "next full NY session adjusted close".to_string(),
                rebalance_frequency: "weekly".to_string(),
                slippage_bps_per_side: 25,
                commission_bps_per_side: 0,
                adjusted_prices_and_dividends_required: true,
                point_in_time_data_only: true,
                lookahead_or_retroactive_revision_allowed: false,
            },
            observation_contract: ControlledShadowExperimentObservationContract {
                minimum_forward_market_sessions: 252,
                checkpoint_market_sessions: vec![21, 63, 126, 252],
                minimum_independent_signal_count: 40,
                minimum_distinct_symbol_count: 12,
                minimum_distinct_market_quarter_count: 4,
                early_promotion_allowed: false,
            },
            metric_contract: ControlledShadowExperimentMetricContract {
                metric_ids: vec![
                    "absolute_return".to_string(),
                    "benchmark_excess_return".to_string(),
                    "maximum_drawdown".to_string(),
                    "hit_rate".to_string(),
                    "turnover".to_string(),
                    "cost_drag".to_string(),
                ],
                composite_score_allowed: false,
                all_metrics_reported_separately: true,
                multiple_testing_adjustment_required: true,
                transaction_costs_included: true,
            },
            stop_contract: ControlledShadowExperimentStopContract {
                stop_rule_ids: (1..=6).map(|v| format!("stop-{v}")).collect(),
                automatic_trade_or_position_action_allowed: false,
                stopped_design_can_be_restarted_in_place: false,
            },
            scalar_reward_defined: false,
            official_model_selected: false,
            model_artifact_materialized: false,
            shadow_ledger_enabled: false,
        };
        value.specification_sha256 =
            fingerprint_without(&value, &["specification_sha256"]).unwrap();
        value
    }

    fn registration() -> ControlledShadowExperimentDesignRegistration {
        let specification = design_specification();
        let mut value = ControlledShadowExperimentDesignRegistration {
            schema_version: "hone-controlled-shadow-experiment-design-registration-v1".to_string(),
            policy_version: "hone-controlled-shadow-experiment-design-create-once-v1-no-shadow-run"
                .to_string(),
            registration_id: String::new(),
            registration_sha256: String::new(),
            attempt_id: "a".repeat(32),
            adjudication_review_id: "b".repeat(32),
            adjudication_review_sha256: "4".repeat(64),
            output_validation_id: "c".repeat(32),
            output_validation_sha256: "5".repeat(64),
            claim_sha256: "6".repeat(64),
            result_sha256: "7".repeat(64),
            output_sha256: "8".repeat(64),
            envelope_sha256: "9".repeat(64),
            candidate_set_sha256: specification.candidate_set_sha256.clone(),
            training_store_dataset_sha256: "a".repeat(64),
            selected_algorithm_three_seed_binding_sha256: "b".repeat(64),
            sealed_holdout_split_commitment_sha256: "c".repeat(64),
            sealed_holdout_projection_sha256: "d".repeat(64),
            feature_order_sha256: specification.feature_order_sha256.clone(),
            preprocessing_sha256: specification.preprocessing_sha256.clone(),
            target_id: specification.target_id.clone(),
            frozen_candidate_algorithm_id: specification.frozen_candidate_algorithm_id.clone(),
            experiment_name: "test design".to_string(),
            research_hypothesis: "forward evidence".to_string(),
            economic_thesis: "not yet confirmed".to_string(),
            known_limitations: "no real run".to_string(),
            falsification_conditions: "leakage or insufficient coverage".to_string(),
            design_specification: specification,
            registered_at: Utc::now(),
            registered_by: "stage-74-registrar".to_string(),
            excluded_prior_actor_ids: vec!["stage-73-reviewer".to_string()],
            registrar_independent_from_complete_prior_chain: true,
            exact_stage_73_adjudication_and_complete_chain_confirmed: true,
            registrar_independent_from_complete_prior_chain_confirmed: true,
            experimental_candidate_not_official_model_selection_confirmed: true,
            point_in_time_forward_only_and_no_retroactive_revision_confirmed: true,
            benchmark_comparators_costs_and_rebalance_frozen_confirmed: true,
            portfolio_caps_cash_floor_and_long_only_boundary_confirmed: true,
            minimum_observation_windows_and_no_early_promotion_confirmed: true,
            separate_metrics_multiple_testing_and_no_composite_confirmed: true,
            stop_rules_and_falsification_are_frozen_confirmed: true,
            independent_design_review_required_before_any_shadow_run_request_confirmed: true,
            no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: true,
            controlled_shadow_experiment_design_registered: true,
            future_independent_design_review_eligible: true,
            design_independently_approved: false,
            official_model_selection_authorized: false,
            model_artifact_store_written: false,
            metric_store_written: false,
            training_feedback_authorized: false,
            reward_authorized: false,
            shadow_run_authorized: false,
            shadow_ledger_enabled: false,
            shadow_position_written: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        value.registration_sha256 =
            fingerprint_without(&value, &["registration_id", "registration_sha256"]).unwrap();
        value.registration_id = value.registration_sha256[..32].to_string();
        value
    }

    fn source() -> ControlledShadowExperimentImplementationEligibleDesign {
        let registration = registration();
        let mut excluded = registration.excluded_prior_actor_ids.clone();
        excluded.push(registration.registered_by.clone());
        excluded.sort();
        let mut review = ControlledShadowExperimentDesignRegistrationReview {
            schema_version: "hone-controlled-shadow-experiment-design-registration-review-v1".to_string(), policy_version: "hone-controlled-shadow-experiment-design-independent-review-v1-no-shadow-run".to_string(), review_id: String::new(), review_sha256: String::new(), previous_review_id: None, previous_review_sha256: None, attempt_id: registration.attempt_id.clone(), registration_id: registration.registration_id.clone(), registration_sha256: registration.registration_sha256.clone(), adjudication_review_id: registration.adjudication_review_id.clone(), adjudication_review_sha256: registration.adjudication_review_sha256.clone(), output_validation_id: registration.output_validation_id.clone(), output_validation_sha256: registration.output_validation_sha256.clone(), claim_sha256: registration.claim_sha256.clone(), result_sha256: registration.result_sha256.clone(), output_sha256: registration.output_sha256.clone(), envelope_sha256: registration.envelope_sha256.clone(), candidate_set_sha256: registration.candidate_set_sha256.clone(), training_store_dataset_sha256: registration.training_store_dataset_sha256.clone(), selected_algorithm_three_seed_binding_sha256: registration.selected_algorithm_three_seed_binding_sha256.clone(), sealed_holdout_split_commitment_sha256: registration.sealed_holdout_split_commitment_sha256.clone(), sealed_holdout_projection_sha256: registration.sealed_holdout_projection_sha256.clone(), feature_order_sha256: registration.feature_order_sha256.clone(), preprocessing_sha256: registration.preprocessing_sha256.clone(), design_specification_sha256: registration.design_specification.specification_sha256.clone(), independently_recomputed_registration_sha256: registration.registration_sha256.clone(), independently_recomputed_design_specification_sha256: registration.design_specification.specification_sha256.clone(), target_id: registration.target_id.clone(), frozen_candidate_algorithm_id: registration.frozen_candidate_algorithm_id.clone(), submitted_at: Utc::now(), reviewer_id: "stage-75-reviewer".to_string(), excluded_prior_actor_ids: excluded, verdict: ControlledShadowExperimentDesignRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityShadowImplementationRegistration, rationale: "approved".to_string(), risk_assessment: "bounded".to_string(), known_limitations: "unrun".to_string(), falsification_assessment: "frozen".to_string(), future_implementation_constraints: "zero capability".to_string(), reviewer_independent_from_registrar_and_complete_prior_chain: true, exact_current_stage_51_through_stage_74_binding_confirmed: true, independent_recomputation_of_registration_and_design_fingerprints_confirmed: true, reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true, experimental_candidate_not_official_model_selection_confirmed: true, point_in_time_universe_survivorship_delisting_and_no_lookahead_reviewed: true, benchmark_and_all_counterfactual_semantics_reviewed: true, signal_timing_execution_cost_dividends_and_rebalance_reviewed: true, long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed: true, minimum_windows_sample_symbol_quarter_gates_and_no_early_promotion_reviewed: true, separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed: true, stop_rules_falsification_and_no_in_place_restart_reviewed: true, no_unconfirmed_hari_or_old_wang_logic_claimed: true, approval_only_opens_future_zero_capability_shadow_implementation_registration_confirmed: true, no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: true, design_registration_independently_approved: true, future_zero_capability_shadow_implementation_registration_eligible: true, official_model_selection_authorized: false, model_artifact_store_written: false, metric_store_written: false, training_feedback_authorized: false, reward_authorized: false, shadow_implementation_registered: false, shadow_run_authorized: false, shadow_ledger_enabled: false, shadow_position_written: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        };
        review.review_sha256 =
            fingerprint_without(&review, &["review_id", "review_sha256"]).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        ControlledShadowExperimentImplementationEligibleDesign {
            design_registration: registration,
            design_review: review,
        }
    }

    fn request(
        source: &ControlledShadowExperimentImplementationEligibleDesign,
    ) -> RegisterControlledShadowExperimentImplementationRequest {
        let registration = &source.design_registration;
        let review = &source.design_review;
        RegisterControlledShadowExperimentImplementationRequest {
            expected_design_review_id: review.review_id.clone(), expected_design_review_sha256: review.review_sha256.clone(), expected_design_registration_id: registration.registration_id.clone(), expected_design_registration_sha256: registration.registration_sha256.clone(), expected_design_specification_sha256: registration.design_specification.specification_sha256.clone(), expected_adjudication_review_sha256: registration.adjudication_review_sha256.clone(), expected_output_validation_sha256: registration.output_validation_sha256.clone(), expected_claim_sha256: registration.claim_sha256.clone(), expected_result_sha256: registration.result_sha256.clone(), expected_output_sha256: registration.output_sha256.clone(), expected_envelope_sha256: registration.envelope_sha256.clone(), expected_candidate_set_sha256: registration.candidate_set_sha256.clone(), expected_training_store_dataset_sha256: registration.training_store_dataset_sha256.clone(), expected_selected_algorithm_three_seed_binding_sha256: registration.selected_algorithm_three_seed_binding_sha256.clone(), expected_sealed_holdout_split_commitment_sha256: registration.sealed_holdout_split_commitment_sha256.clone(), expected_sealed_holdout_projection_sha256: registration.sealed_holdout_projection_sha256.clone(), expected_feature_order_sha256: registration.feature_order_sha256.clone(), expected_preprocessing_sha256: registration.preprocessing_sha256.clone(), expected_target_id: registration.target_id.clone(), expected_frozen_candidate_algorithm_id: registration.frozen_candidate_algorithm_id.clone(), implementation_name: "zero capability replay specification".to_string(), immutable_code_revision: "git:stage76-test".to_string(), implementation_description: "pure deterministic contract only".to_string(), deterministic_replay_notes: "all state transitions and checkpoints are content addressed".to_string(), known_limitations: "no real forward data or runner".to_string(), future_review_constraints: "independent review before runner registration".to_string(), exact_current_stage_51_through_stage_75_binding_confirmed: true, registrar_independent_from_stage_75_and_complete_prior_chain_confirmed: true, independent_recomputation_of_design_review_registration_and_specification_confirmed: true, zero_capability_specification_only_not_executable_artifact_confirmed: true, point_in_time_universe_delisting_and_no_lookahead_semantics_preserved_confirmed: true, signal_execution_cost_dividend_rebalance_and_counterfactual_semantics_preserved_confirmed: true, long_only_caps_cash_floor_no_options_leverage_or_shorting_preserved_confirmed: true, observation_sample_checkpoint_metric_multiple_testing_and_stop_rules_preserved_confirmed: true, deterministic_create_once_content_addressed_replay_contract_confirmed: true, no_entrypoint_runtime_environment_secret_network_tool_subprocess_or_production_io_confirmed: true, no_model_store_metric_store_training_feedback_composite_or_reward_confirmed: true, no_shadow_run_ledger_position_order_broker_or_trading_confirmed: true, future_independent_implementation_review_required_before_runner_registration_confirmed: true, no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    pub(crate) fn record() -> ControlledShadowExperimentImplementationRecord {
        let source = source();
        build_record(&source, "stage-76-registrar", request(&source), Utc::now()).unwrap()
    }

    #[test]
    fn contract_binds_exact_review_registration_design_and_three_seeds() {
        let source = source();
        let contract = implementation_contract(&source, "revision").unwrap();
        assert_eq!(
            contract.stage_75_design_review_sha256,
            source.design_review.review_sha256
        );
        assert_eq!(
            contract.stage_74_design_registration_sha256,
            source.design_registration.registration_sha256
        );
        assert_eq!(
            contract.exact_design_specification,
            source.design_registration.design_specification
        );
        assert_eq!(contract.random_seeds, [17, 29, 43]);
    }

    #[test]
    fn registered_implementation_has_zero_execution_capability() {
        let value = record();
        let contract = &value.implementation_contract;
        assert!(!contract.callable_entrypoint_present);
        assert!(!contract.executable_artifact_present);
        assert!(!contract.runtime_present);
        assert!(!contract.input_mount_present);
        assert!(!contract.outbound_network_allowed);
        assert!(!contract.production_reads_allowed);
        assert!(!contract.production_writes_allowed);
        assert!(!contract.shadow_ledger_creation_allowed);
        assert!(!contract.order_generation_allowed);
        assert!(!contract.broker_access_allowed);
        assert!(!contract.trading_allowed);
    }

    #[test]
    fn deterministic_contract_preserves_design_constraints_without_reward() {
        let value = record();
        let spec = &value.implementation_contract.exact_design_specification;
        assert_eq!(spec.portfolio_constraints.virtual_notional_usd, 1_000_000);
        assert_eq!(
            spec.portfolio_constraints.maximum_single_name_weight_bps,
            500
        );
        assert_eq!(spec.portfolio_constraints.maximum_theme_weight_bps, 2_000);
        assert_eq!(spec.portfolio_constraints.maximum_gross_exposure_bps, 6_000);
        assert_eq!(spec.portfolio_constraints.minimum_cash_weight_bps, 4_000);
        assert_eq!(spec.execution_contract.slippage_bps_per_side, 25);
        assert_eq!(
            spec.observation_contract.minimum_forward_market_sessions,
            252
        );
        assert!(!value.implementation_contract.scalar_reward_defined);
    }

    #[test]
    fn every_registration_confirmation_is_required() {
        let source = source();
        let mut value = request(&source);
        assert!(request_checks_confirmed(&value));
        value.no_shadow_run_ledger_position_order_broker_or_trading_confirmed = false;
        assert!(!request_checks_confirmed(&value));
    }

    #[test]
    fn exact_design_binding_rejects_drift() {
        let source = source();
        let mut value = request(&source);
        assert!(validate_expected_bindings(&source, &value).is_ok());
        value.expected_design_specification_sha256 = "f".repeat(64);
        assert!(validate_expected_bindings(&source, &value).is_err());
    }

    #[test]
    fn registrar_must_be_outside_stage_75_and_complete_prior_chain() {
        let source = source();
        assert!(
            build_record(
                &source,
                &source.design_review.reviewer_id,
                request(&source),
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn valid_registration_only_opens_future_independent_implementation_review() {
        let value = record();
        assert!(validate_record(&value).is_ok());
        assert!(value.future_independent_shadow_implementation_review_eligible);
        assert!(!value.isolated_runner_registration_eligible);
        assert!(!value.shadow_run_authorized);
        assert!(!value.shadow_ledger_enabled);
        assert!(!value.shadow_position_written);
        assert!(!value.order_generation_authorized);
        assert!(!value.broker_access_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn tamper_authority_escalation_and_duplicate_registration_fail_closed() {
        let source = source();
        let value = record();
        assert!(ensure_source_unregistered(&[], &source).is_ok());
        assert!(ensure_source_unregistered(&[value.clone()], &source).is_err());
        let mut tampered = value;
        tampered.implementation_contract.shadow_run_allowed = true;
        tampered.implementation_contract.contract_sha256 =
            fingerprint_without(&tampered.implementation_contract, &["contract_sha256"]).unwrap();
        tampered.implementation_sha256 = record_fingerprint(&tampered).unwrap();
        tampered.implementation_id = tampered.implementation_sha256[..32].to_string();
        assert!(validate_record(&tampered).is_err());
    }

    #[test]
    fn required_text_is_bounded() {
        assert!(bounded_required("", 10, "说明").is_err());
        assert!(bounded_required(&"a".repeat(11), 10, "说明").is_err());
    }
}
