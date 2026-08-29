//! Stage 84 create-once registration of a zero-capability implementation specification for one
//! exact Stage 83 independently approved natural-forward observation protocol.
//!
//! This stage freezes deterministic implementation semantics and future envelopes only. It has no
//! executable artifact, callable entrypoint, runtime, input mount, observation writer, ledger,
//! position, metric write, feedback/reward, order, broker connection or trading authority.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_protocol_registration_reviews::{
    ControlledShadowForwardObservationProtocolRegistrationReview,
    ControlledShadowForwardObservationProtocolRegistrationReviewVerdict,
    IndependentlyApprovedControlledShadowForwardObservationProtocol,
    independently_approved_controlled_shadow_forward_observation_protocols_for_implementation_registration,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_protocol_registrations::{
    ControlledShadowForwardObservationProtocolRegistration,
    ControlledShadowForwardObservationProtocolSpecification,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-implementation-registry-v1";
const RECORD_SCHEMA_VERSION: &str = "hone-controlled-shadow-forward-observation-implementation-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-zero-capability-implementation-contract-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-forward-observation-implementation-create-once-v1-zero-capability";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-controlled-shadow-natural-forward-observation-implementation-v1-not-executable";
const REGISTRATION_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-registration-v1";
const REGISTRATION_POLICY_VERSION: &str =
    "hone-controlled-shadow-natural-forward-only-protocol-create-once-v1-no-observation";
const PROTOCOL_SPEC_SCHEMA_VERSION: &str = "hone-controlled-shadow-forward-observation-protocol-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-registration-review-v1";
const REVIEW_POLICY_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-chain-external-review-v1-no-observation";
const DESIGN_SPEC_SCHEMA_VERSION: &str = "hone-controlled-shadow-experiment-design-spec-v1";
const STATUS: &str = "registered_not_reviewed_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowForwardObservationImplementationRequest {
    expected_protocol_review_id: String,
    expected_protocol_review_sha256: String,
    expected_protocol_registration_id: String,
    expected_protocol_registration_sha256: String,
    expected_protocol_specification_sha256: String,
    expected_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_input_manifest_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_runner_spec_sha256: String,
    expected_runner_artifact_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_design_specification_sha256: String,
    expected_candidate_set_sha256: String,
    expected_feature_order_sha256: String,
    expected_preprocessing_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_description: String,
    deterministic_observation_semantics: String,
    evidence_custody_and_correction_semantics: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_current_stage_51_through_stage_83_binding_confirmed: bool,
    registrar_independent_from_stage_83_and_complete_prior_chain_confirmed: bool,
    independent_recomputation_of_review_registration_protocol_and_design_confirmed: bool,
    zero_capability_specification_only_no_executable_artifact_confirmed: bool,
    natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: bool,
    weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed: bool,
    official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed: bool,
    next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed: bool,
    checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed:
        bool,
    deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed: bool,
    no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed:
        bool,
    no_production_read_write_observation_ledger_position_or_performance_write_confirmed: bool,
    no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed: bool,
    future_independent_implementation_review_required_before_runner_registration_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityForwardObservationAuthorityBoundary {
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
    pub observation_claim_write_allowed: bool,
    pub forward_observation_write_allowed: bool,
    pub ledger_creation_allowed: bool,
    pub position_write_allowed: bool,
    pub performance_metric_write_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub training_feedback_allowed: bool,
    pub scalar_reward_defined: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_protocol_version: String,
    pub immutable_code_revision: String,
    pub stage_83_protocol_review_id: String,
    pub stage_83_protocol_review_sha256: String,
    pub stage_82_protocol_registration_id: String,
    pub stage_82_protocol_registration_sha256: String,
    pub protocol_specification_sha256: String,
    pub validation_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub input_manifest_sha256: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub upstream_shadow_implementation_contract_sha256: String,
    pub design_specification_sha256: String,
    pub candidate_set_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub exact_protocol_specification: ControlledShadowForwardObservationProtocolSpecification,
    pub deterministic_weekly_claim_function_id: String,
    pub deterministic_market_calendar_function_id: String,
    pub deterministic_point_in_time_source_custody_function_id: String,
    pub deterministic_corporate_action_correction_function_id: String,
    pub deterministic_signal_projection_function_id: String,
    pub deterministic_portfolio_transition_function_id: String,
    pub deterministic_fill_cost_and_counterfactual_function_id: String,
    pub deterministic_checkpoint_metric_and_stop_function_id: String,
    pub canonical_future_input_manifest_schema: String,
    pub canonical_future_cycle_claim_schema: String,
    pub canonical_future_untrusted_observation_schema: String,
    pub future_inputs_read_only_point_in_time_and_content_addressed: bool,
    pub future_cycle_claim_first_required: bool,
    pub future_cycle_create_once_required: bool,
    pub future_output_create_once_and_untrusted: bool,
    pub future_output_independent_validation_required: bool,
    pub future_output_contains_order_intent: bool,
    pub future_output_contains_broker_payload: bool,
    pub registered_not_run: bool,
    pub independent_implementation_review_required: bool,
    pub isolated_runner_registration_required_after_review: bool,
    pub authority_boundary: ZeroCapabilityForwardObservationAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub upstream_protocol_registration: ControlledShadowForwardObservationProtocolRegistration,
    pub upstream_protocol_review: ControlledShadowForwardObservationProtocolRegistrationReview,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_83_and_complete_prior_chain: bool,
    pub implementation_name: String,
    pub implementation_description: String,
    pub deterministic_observation_semantics: String,
    pub evidence_custody_and_correction_semantics: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub implementation_contract: ControlledShadowForwardObservationImplementationContract,
    pub status: String,
    pub exact_current_stage_51_through_stage_83_binding_confirmed: bool,
    pub registrar_independent_from_stage_83_and_complete_prior_chain_confirmed: bool,
    pub independent_recomputation_of_review_registration_protocol_and_design_confirmed: bool,
    pub zero_capability_specification_only_no_executable_artifact_confirmed: bool,
    pub natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: bool,
    pub weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed: bool,
    pub official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed: bool,
    pub next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed: bool,
    pub checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed:
        bool,
    pub deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed: bool,
    pub no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed:
        bool,
    pub no_production_read_write_observation_ledger_position_or_performance_write_confirmed: bool,
    pub no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed: bool,
    pub future_independent_implementation_review_required_before_runner_registration_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_forward_observation_implementation_registered: bool,
    pub future_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub isolated_runner_registration_eligible: bool,
    pub forward_observation_authorized: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationImplementationItem {
    pub source: IndependentlyApprovedControlledShadowForwardObservationProtocol,
    pub implementation: Option<ControlledShadowForwardObservationImplementationRecord>,
    pub registration_eligible: bool,
    pub upstream_binding_current: bool,
    pub future_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowForwardObservationImplementationItem>,
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub callable_entrypoint_present: bool,
    pub executable_artifact_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub forward_observation_authorized: bool,
    pub forward_observation_ledger_created: bool,
    pub shadow_position_written: bool,
    pub performance_metric_written: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowForwardObservationImplementationReadinessSummary {
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_forward_observation_implementations(
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
            warn!(%error, "controlled shadow forward-observation implementation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "前向观察零能力实现登记暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_forward_observation_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(protocol_review_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowForwardObservationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, &protocol_review_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_forward_observation_implementation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationImplementationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowForwardObservationImplementationReadinessSummary {
            registration_eligible_count: value.registration_eligible_count,
            implementation_count: value.implementation_count,
            current_binding_implementation_count: value.current_binding_implementation_count,
            independent_implementation_review_eligible_count: value
                .independent_implementation_review_eligible_count,
            implementation_status: value.implementation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_reviewable_controlled_shadow_forward_observation_implementations(
    state: &AppState,
) -> Result<Vec<ControlledShadowForwardObservationImplementationRecord>, String> {
    let value = build_registry(state).await?;
    let output = value
        .items
        .into_iter()
        .filter_map(|item| {
            item.implementation.and_then(|implementation| {
                item.future_independent_implementation_review_eligible
                    .then_some(implementation)
            })
        })
        .collect::<Vec<_>>();
    for record in &output {
        validate_record(record)?;
    }
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationImplementationRegistry, String> {
    let sources =
        independently_approved_controlled_shadow_forward_observation_protocols_for_implementation_registration(
            state,
        )
        .await?;
    let records = read_records(state).await?;
    let records_by_review = records
        .iter()
        .map(|record| (record.upstream_protocol_review.review_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let mut items = Vec::new();
    for source in sources {
        let implementation = records_by_review
            .get(source.review.review_id.as_str())
            .map(|value| (*value).clone());
        let upstream_binding_current = implementation
            .as_ref()
            .is_some_and(|record| record_matches_source(record, &source));
        let registration_eligible = implementation.is_none();
        items.push(ControlledShadowForwardObservationImplementationItem {
            future_independent_implementation_review_eligible: upstream_binding_current,
            source,
            implementation,
            registration_eligible,
            upstream_binding_current,
        });
    }
    items.sort_by(|left, right| {
        right
            .source
            .review
            .submitted_at
            .cmp(&left.source.review.submitted_at)
    });
    let registration_eligible_count = items
        .iter()
        .filter(|item| item.registration_eligible)
        .count();
    let implementation_count = records.len();
    let current_binding_implementation_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .count();
    let implementation_status = if current_binding_implementation_count > 0 {
        "zero_capability_forward_observation_implementation_registered_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "independently_approved_forward_observation_protocol_ready_for_zero_capability_implementation_registration"
    } else {
        "waiting_stage_83_independently_approved_forward_observation_protocol"
    };
    Ok(ControlledShadowForwardObservationImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        registration_eligible_count,
        implementation_count,
        current_binding_implementation_count,
        independent_implementation_review_eligible_count,
        implementation_status: implementation_status.to_string(),
        callable_entrypoint_present: false,
        executable_artifact_present: false,
        runtime_present: false,
        input_mount_present: false,
        forward_observation_authorized: false,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 84 只把当前 Stage 83 独立批准协议登记为内容寻址、零能力、不可执行的观察实现规格。规格冻结周度 claim、官方日历、点时来源、公司行动更正、模型投影、组合、成本、反事实、检查点、指标与停止语义；没有工件、入口、runtime、mount、adapter、观察写入、账本、持仓、绩效、模型/指标库、反馈、reward、订单、券商或交易。下一步只能进入责任链外独立实现复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    protocol_review_id: &str,
    request: RegisterControlledShadowForwardObservationImplementationRequest,
) -> Result<ControlledShadowForwardObservationImplementationRecord, String> {
    if !valid_id(protocol_review_id) {
        return Err("Stage 83 协议复核 ID 无效".to_string());
    }
    if registrar_id.trim().is_empty() {
        return Err("Stage 84 实现登记者无效".to_string());
    }
    let _lock = acquire_lock(state, protocol_review_id).await?;
    let source =
        independently_approved_controlled_shadow_forward_observation_protocols_for_implementation_registration(
            state,
        )
        .await?
        .into_iter()
        .find(|source| source.review.review_id == protocol_review_id)
        .ok_or_else(|| "当前没有精确匹配且独立批准的 Stage 83 协议".to_string())?;
    if read_records(state)
        .await?
        .iter()
        .any(|record| record.upstream_protocol_review.review_id == protocol_review_id)
    {
        return Err("该 Stage 83 复核已登记实现规格，禁止覆盖或重放".to_string());
    }
    let record = build_record(&source, registrar_id, request, Utc::now())?;
    write_immutable_json(&record_path(state, protocol_review_id), &record).await?;
    Ok(record)
}

fn build_record(
    source: &IndependentlyApprovedControlledShadowForwardObservationProtocol,
    registrar_id: &str,
    request: RegisterControlledShadowForwardObservationImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<ControlledShadowForwardObservationImplementationRecord, String> {
    validate_source(source)?;
    validate_expected_binding(source, &request)?;
    if !all_confirmations(&request) {
        return Err("登记前必须逐项确认完整绑定、确定性语义、零能力与职责分离".to_string());
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
    let deterministic_observation_semantics = bounded_required(
        &request.deterministic_observation_semantics,
        MAX_TEXT_CHARS,
        "确定性观察语义",
    )?;
    let evidence_custody_and_correction_semantics = bounded_required(
        &request.evidence_custody_and_correction_semantics,
        MAX_TEXT_CHARS,
        "证据保管与更正语义",
    )?;
    let known_limitations =
        bounded_required(&request.known_limitations, MAX_TEXT_CHARS, "已知局限")?;
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
        || !request.registrar_independent_from_stage_83_and_complete_prior_chain_confirmed
    {
        return Err(
            "实现登记者必须独立于 Stage 83 复核者、Stage 82 登记者和完整上游责任链".to_string(),
        );
    }
    let implementation_contract = implementation_contract(source, &immutable_code_revision)?;
    let mut record = ControlledShadowForwardObservationImplementationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        upstream_protocol_registration: source.registered_protocol.registration.clone(),
        upstream_protocol_review: source.review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_83_and_complete_prior_chain: independent,
        implementation_name,
        implementation_description,
        deterministic_observation_semantics,
        evidence_custody_and_correction_semantics,
        known_limitations,
        future_review_constraints,
        implementation_contract,
        status: STATUS.to_string(),
        exact_current_stage_51_through_stage_83_binding_confirmed: request
            .exact_current_stage_51_through_stage_83_binding_confirmed,
        registrar_independent_from_stage_83_and_complete_prior_chain_confirmed: request
            .registrar_independent_from_stage_83_and_complete_prior_chain_confirmed,
        independent_recomputation_of_review_registration_protocol_and_design_confirmed: request
            .independent_recomputation_of_review_registration_protocol_and_design_confirmed,
        zero_capability_specification_only_no_executable_artifact_confirmed: request
            .zero_capability_specification_only_no_executable_artifact_confirmed,
        natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: request
            .natural_forward_no_backfill_and_observation_not_before_preserved_confirmed,
        weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed: request
            .weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed,
        official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed: request
            .official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed,
        next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed: request
            .next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed,
        checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed:
            request.checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed,
        deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed:
            request.deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed,
        no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed:
            request.no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed,
        no_production_read_write_observation_ledger_position_or_performance_write_confirmed:
            request.no_production_read_write_observation_ledger_position_or_performance_write_confirmed,
        no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed:
            request.no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed,
        future_independent_implementation_review_required_before_runner_registration_confirmed:
            request.future_independent_implementation_review_required_before_runner_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_forward_observation_implementation_registered: true,
        future_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        isolated_runner_registration_eligible: false,
        forward_observation_authorized: false,
        forward_observation_ledger_created: false,
        shadow_position_written: false,
        performance_metric_written: false,
        model_artifact_store_written: false,
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
    source: &IndependentlyApprovedControlledShadowForwardObservationProtocol,
    immutable_code_revision: &str,
) -> Result<ControlledShadowForwardObservationImplementationContract, String> {
    let registration = &source.registered_protocol.registration;
    let review = &source.review;
    let mut contract = ControlledShadowForwardObservationImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_protocol_version: IMPLEMENTATION_PROTOCOL_VERSION.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        stage_83_protocol_review_id: review.review_id.clone(),
        stage_83_protocol_review_sha256: review.review_sha256.clone(),
        stage_82_protocol_registration_id: registration.protocol_registration_id.clone(),
        stage_82_protocol_registration_sha256: registration.protocol_registration_sha256.clone(),
        protocol_specification_sha256: registration
            .protocol_specification
            .specification_sha256
            .clone(),
        validation_sha256: registration.validation_sha256.clone(),
        claim_sha256: registration.claim_sha256.clone(),
        result_sha256: registration.result_sha256.clone(),
        output_sha256: registration.output_sha256.clone(),
        input_manifest_sha256: registration.input_manifest_sha256.clone(),
        authorization_review_sha256: registration.authorization_review_sha256.clone(),
        isolated_runner_spec_sha256: registration.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: registration.runner_artifact_sha256.clone(),
        upstream_shadow_implementation_contract_sha256: registration
            .implementation_contract_sha256
            .clone(),
        design_specification_sha256: registration.design_specification_sha256.clone(),
        candidate_set_sha256: registration.candidate_set_sha256.clone(),
        feature_order_sha256: registration.feature_order_sha256.clone(),
        preprocessing_sha256: registration.preprocessing_sha256.clone(),
        target_id: registration.target_id.clone(),
        frozen_candidate_algorithm_id: registration.frozen_candidate_algorithm_id.clone(),
        exact_protocol_specification: registration.protocol_specification.clone(),
        deterministic_weekly_claim_function_id:
            "forward_observation_weekly_claim_v1_pure_specification".to_string(),
        deterministic_market_calendar_function_id:
            "forward_observation_official_us_calendar_half_day_halt_v1_pure_specification"
                .to_string(),
        deterministic_point_in_time_source_custody_function_id:
            "forward_observation_point_in_time_content_addressed_allowlist_v1_pure_specification"
                .to_string(),
        deterministic_corporate_action_correction_function_id:
            "forward_observation_raw_adjusted_dividend_split_append_only_v1_pure_specification"
                .to_string(),
        deterministic_signal_projection_function_id:
            "forward_observation_frozen_three_seed_signal_v1_pure_specification".to_string(),
        deterministic_portfolio_transition_function_id:
            "forward_observation_long_only_cash_constrained_transition_v1_pure_specification"
                .to_string(),
        deterministic_fill_cost_and_counterfactual_function_id:
            "forward_observation_next_full_session_25bps_counterfactual_v1_pure_specification"
                .to_string(),
        deterministic_checkpoint_metric_and_stop_function_id:
            "forward_observation_21_63_126_252_separate_metric_stop_v1_pure_specification"
                .to_string(),
        canonical_future_input_manifest_schema:
            "forward_observation_point_in_time_input_manifest_v1_not_mounted".to_string(),
        canonical_future_cycle_claim_schema: "forward_observation_weekly_claim_v1_not_created"
            .to_string(),
        canonical_future_untrusted_observation_schema:
            "forward_observation_checkpoint_envelope_v1_not_created_no_order_intent".to_string(),
        future_inputs_read_only_point_in_time_and_content_addressed: true,
        future_cycle_claim_first_required: true,
        future_cycle_create_once_required: true,
        future_output_create_once_and_untrusted: true,
        future_output_independent_validation_required: true,
        future_output_contains_order_intent: false,
        future_output_contains_broker_payload: false,
        registered_not_run: true,
        independent_implementation_review_required: true,
        isolated_runner_registration_required_after_review: true,
        authority_boundary: zero_capability_authority_boundary(),
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn zero_capability_authority_boundary() -> ZeroCapabilityForwardObservationAuthorityBoundary {
    ZeroCapabilityForwardObservationAuthorityBoundary {
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
        observation_claim_write_allowed: false,
        forward_observation_write_allowed: false,
        ledger_creation_allowed: false,
        position_write_allowed: false,
        performance_metric_write_allowed: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        training_feedback_allowed: false,
        scalar_reward_defined: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    }
}

fn authority_closed(value: &ZeroCapabilityForwardObservationAuthorityBoundary) -> bool {
    !value.callable_entrypoint_present
        && !value.executable_artifact_present
        && !value.runtime_present
        && !value.input_mount_present
        && !value.market_data_adapter_present
        && !value.environment_inheritance_allowed
        && !value.secrets_allowed
        && !value.outbound_network_allowed
        && !value.tools_allowed
        && !value.subprocesses_allowed
        && !value.production_reads_allowed
        && !value.production_writes_allowed
        && !value.observation_claim_write_allowed
        && !value.forward_observation_write_allowed
        && !value.ledger_creation_allowed
        && !value.position_write_allowed
        && !value.performance_metric_write_allowed
        && !value.model_store_write_allowed
        && !value.metric_store_write_allowed
        && !value.training_feedback_allowed
        && !value.scalar_reward_defined
        && !value.order_generation_allowed
        && !value.broker_access_allowed
        && !value.trading_allowed
}

fn validate_source(
    source: &IndependentlyApprovedControlledShadowForwardObservationProtocol,
) -> Result<(), String> {
    validate_source_parts(&source.registered_protocol.registration, &source.review)
}

fn validate_source_parts(
    registration: &ControlledShadowForwardObservationProtocolRegistration,
    review: &ControlledShadowForwardObservationProtocolRegistrationReview,
) -> Result<(), String> {
    let registration_sha = fingerprint_without(
        registration,
        &["protocol_registration_id", "protocol_registration_sha256"],
    )?;
    let protocol_sha = fingerprint_without(
        &registration.protocol_specification,
        &["specification_sha256"],
    )?;
    let design_sha = fingerprint_without(
        &registration
            .protocol_specification
            .exact_design_specification,
        &["specification_sha256"],
    )?;
    let review_sha = fingerprint_without(review, &["review_id", "review_sha256"])?;
    let registration_confirmations = registration
        .registrar_independent_from_stage_81_and_complete_prior_chain
        && registration.exact_stage_51_through_stage_81_binding_confirmed
        && registration.natural_forward_only_no_backfill_confirmed
        && registration.weekly_claim_first_content_addressed_observation_confirmed
        && registration.official_us_market_calendar_and_spy_synchronization_confirmed
        && registration.point_in_time_allowlisted_source_custody_confirmed
        && registration.adjusted_prices_dividends_and_append_only_corrections_confirmed
        && registration.next_full_session_fill_and_registered_costs_confirmed
        && registration.checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed
        && registration.stop_rules_fail_closed_and_no_in_place_restart_confirmed
        && registration.independent_protocol_review_required_before_observation_confirmed
        && registration.no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed
        && registration.protocol_registered
        && registration.future_independent_protocol_review_eligible;
    let review_confirmations = review
        .reviewer_independent_from_registrar_and_complete_prior_chain
        && review.exact_current_stage_51_through_stage_82_binding_confirmed
        && review.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && review.independent_recomputation_of_registration_protocol_and_design_fingerprints_confirmed
        && review.observation_not_before_and_no_retroactive_backfill_reviewed
        && review.weekly_claim_first_create_once_reviewed
        && review.official_us_market_calendar_half_days_halts_and_spy_sync_reviewed
        && review.point_in_time_allowlist_content_addressing_and_source_availability_reviewed
        && review.raw_adjusted_prices_dividends_splits_corporate_actions_and_append_only_corrections_reviewed
        && review.next_full_session_fill_25bps_cost_and_counterfactuals_reviewed
        && review.long_only_caps_cash_floor_no_options_leverage_or_shorting_reviewed
        && review.checkpoints_and_252_40_12_4_minimums_without_early_promotion_reviewed
        && review.separate_metrics_multiple_testing_no_composite_or_scalar_reward_reviewed
        && review.stop_falsification_fail_closed_and_no_in_place_restart_reviewed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
        && review.approval_only_opens_future_zero_capability_observation_implementation_registration_confirmed
        && review.no_observation_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed;
    let upstream_closed = !registration.forward_observation_authorized
        && !registration.forward_observation_ledger_created
        && !registration.shadow_position_written
        && !registration.performance_metric_written
        && !registration.model_artifact_store_written
        && !registration.metric_store_written
        && !registration.training_feedback_authorized
        && !registration.reward_authorized
        && !registration.order_generation_authorized
        && !registration.broker_access_authorized
        && !registration.trading_authorized
        && !review.forward_observation_authorized
        && !review.forward_observation_ledger_created
        && !review.shadow_position_written
        && !review.performance_metric_written
        && !review.model_artifact_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    if registration.schema_version != REGISTRATION_SCHEMA_VERSION
        || registration.policy_version != REGISTRATION_POLICY_VERSION
        || registration.protocol_registration_sha256 != registration_sha
        || registration.protocol_registration_id != registration_sha[..32]
        || registration.protocol_specification.schema_version != PROTOCOL_SPEC_SCHEMA_VERSION
        || registration.protocol_specification.specification_sha256 != protocol_sha
        || registration
            .protocol_specification
            .exact_design_specification
            .schema_version
            != DESIGN_SPEC_SCHEMA_VERSION
        || registration.design_specification_sha256 != design_sha
        || review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != REVIEW_POLICY_VERSION
        || review.review_sha256 != review_sha
        || review.review_id != review_sha[..32]
        || review.protocol_registration_id != registration.protocol_registration_id
        || review.protocol_registration_sha256 != registration.protocol_registration_sha256
        || review.protocol_specification_sha256
            != registration.protocol_specification.specification_sha256
        || review.independently_recomputed_protocol_registration_sha256 != registration_sha
        || review.independently_recomputed_protocol_specification_sha256 != protocol_sha
        || review.independently_recomputed_exact_design_specification_sha256 != design_sha
        || review.verdict
            != ControlledShadowForwardObservationProtocolRegistrationReviewVerdict::ApprovedForFutureZeroCapabilityForwardObservationImplementationRegistration
        || !review.protocol_registration_independently_approved
        || !review
            .future_zero_capability_forward_observation_implementation_registration_eligible
        || !registration_confirmations
        || !review_confirmations
        || !sorted_unique(&registration.excluded_prior_actor_ids)
        || !sorted_unique(&review.excluded_prior_actor_ids)
        || registration
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &registration.registered_by)
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &review.reviewer_id)
        || review
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &registration.registered_by)
        || !upstream_closed
    {
        return Err("Stage 82–83 前向观察协议来源无效、漂移或越权".to_string());
    }
    Ok(())
}

fn validate_expected_binding(
    source: &IndependentlyApprovedControlledShadowForwardObservationProtocol,
    request: &RegisterControlledShadowForwardObservationImplementationRequest,
) -> Result<(), String> {
    let registration = &source.registered_protocol.registration;
    let review = &source.review;
    let exact = request.expected_protocol_review_id == review.review_id
        && request.expected_protocol_review_sha256 == review.review_sha256
        && request.expected_protocol_registration_id == registration.protocol_registration_id
        && request.expected_protocol_registration_sha256
            == registration.protocol_registration_sha256
        && request.expected_protocol_specification_sha256
            == registration.protocol_specification.specification_sha256
        && request.expected_validation_sha256 == registration.validation_sha256
        && request.expected_claim_sha256 == registration.claim_sha256
        && request.expected_result_sha256 == registration.result_sha256
        && request.expected_output_sha256 == registration.output_sha256
        && request.expected_input_manifest_sha256 == registration.input_manifest_sha256
        && request.expected_authorization_review_sha256 == registration.authorization_review_sha256
        && request.expected_isolated_runner_spec_sha256 == registration.isolated_runner_spec_sha256
        && request.expected_runner_artifact_sha256 == registration.runner_artifact_sha256
        && request.expected_implementation_contract_sha256
            == registration.implementation_contract_sha256
        && request.expected_design_specification_sha256 == registration.design_specification_sha256
        && request.expected_candidate_set_sha256 == registration.candidate_set_sha256
        && request.expected_feature_order_sha256 == registration.feature_order_sha256
        && request.expected_preprocessing_sha256 == registration.preprocessing_sha256
        && request.expected_target_id == registration.target_id
        && request.expected_frozen_candidate_algorithm_id
            == registration.frozen_candidate_algorithm_id;
    exact
        .then_some(())
        .ok_or_else(|| "Stage 51–83 精确绑定已经变化，请刷新后重试".to_string())
}

fn all_confirmations(
    request: &RegisterControlledShadowForwardObservationImplementationRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_83_binding_confirmed
        && request.registrar_independent_from_stage_83_and_complete_prior_chain_confirmed
        && request.independent_recomputation_of_review_registration_protocol_and_design_confirmed
        && request.zero_capability_specification_only_no_executable_artifact_confirmed
        && request.natural_forward_no_backfill_and_observation_not_before_preserved_confirmed
        && request.weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed
        && request.official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed
        && request.next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed
        && request.checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed
        && request.deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed
        && request.no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed
        && request.no_production_read_write_observation_ledger_position_or_performance_write_confirmed
        && request.no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed
        && request.future_independent_implementation_review_required_before_runner_registration_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_contract(
    contract: &ControlledShadowForwardObservationImplementationContract,
    registration: &ControlledShadowForwardObservationProtocolRegistration,
    review: &ControlledShadowForwardObservationProtocolRegistrationReview,
) -> Result<(), String> {
    let protocol = &contract.exact_protocol_specification;
    let design = &protocol.exact_design_specification;
    let protocol_preserved = protocol.specification_sha256
        == registration.protocol_specification.specification_sha256
        && protocol == &registration.protocol_specification
        && protocol.natural_forward_only
        && !protocol.retroactive_backfill_allowed
        && protocol.per_cycle_claim_first_required
        && protocol.per_cycle_create_once_required
        && protocol.point_in_time_content_addressed_allowlisted_sources_required
        && protocol.benchmark_symbol == "SPY"
        && protocol.synchronized_security_and_benchmark_observation_required
        && protocol.corporate_action_evidence_required
        && !protocol.performance_before_natural_checkpoint_allowed
        && !protocol.forward_observation_started
        && !protocol.ledger_created
        && design.specification_sha256 == registration.design_specification_sha256
        && design.random_seeds == vec![17, 29, 43]
        && design.execution_contract.slippage_bps_per_side == 25
        && !design
            .execution_contract
            .lookahead_or_retroactive_revision_allowed
        && design.observation_contract.checkpoint_market_sessions == vec![21, 63, 126, 252]
        && design.observation_contract.minimum_forward_market_sessions == 252
        && design.observation_contract.minimum_independent_signal_count == 40
        && design.observation_contract.minimum_distinct_symbol_count == 12
        && design
            .observation_contract
            .minimum_distinct_market_quarter_count
            == 4
        && !design.observation_contract.early_promotion_allowed
        && !design.metric_contract.composite_score_allowed
        && design.metric_contract.all_metrics_reported_separately
        && !design
            .stop_contract
            .automatic_trade_or_position_action_allowed
        && !design
            .stop_contract
            .stopped_design_can_be_restarted_in_place;
    let function_ids = [
        &contract.deterministic_weekly_claim_function_id,
        &contract.deterministic_market_calendar_function_id,
        &contract.deterministic_point_in_time_source_custody_function_id,
        &contract.deterministic_corporate_action_correction_function_id,
        &contract.deterministic_signal_projection_function_id,
        &contract.deterministic_portfolio_transition_function_id,
        &contract.deterministic_fill_cost_and_counterfactual_function_id,
        &contract.deterministic_checkpoint_metric_and_stop_function_id,
    ]
    .into_iter()
    .all(|value| value.ends_with("_pure_specification"));
    if contract.schema_version != CONTRACT_SCHEMA_VERSION
        || contract.contract_sha256 != fingerprint_without(contract, &["contract_sha256"])?
        || contract.implementation_protocol_version != IMPLEMENTATION_PROTOCOL_VERSION
        || contract.immutable_code_revision.trim().is_empty()
        || contract.stage_83_protocol_review_id != review.review_id
        || contract.stage_83_protocol_review_sha256 != review.review_sha256
        || contract.stage_82_protocol_registration_id != registration.protocol_registration_id
        || contract.stage_82_protocol_registration_sha256
            != registration.protocol_registration_sha256
        || contract.protocol_specification_sha256
            != registration.protocol_specification.specification_sha256
        || contract.validation_sha256 != registration.validation_sha256
        || contract.claim_sha256 != registration.claim_sha256
        || contract.result_sha256 != registration.result_sha256
        || contract.output_sha256 != registration.output_sha256
        || contract.input_manifest_sha256 != registration.input_manifest_sha256
        || contract.authorization_review_sha256 != registration.authorization_review_sha256
        || contract.isolated_runner_spec_sha256 != registration.isolated_runner_spec_sha256
        || contract.runner_artifact_sha256 != registration.runner_artifact_sha256
        || contract.upstream_shadow_implementation_contract_sha256
            != registration.implementation_contract_sha256
        || contract.design_specification_sha256 != registration.design_specification_sha256
        || contract.candidate_set_sha256 != registration.candidate_set_sha256
        || contract.feature_order_sha256 != registration.feature_order_sha256
        || contract.preprocessing_sha256 != registration.preprocessing_sha256
        || contract.target_id != registration.target_id
        || contract.frozen_candidate_algorithm_id != registration.frozen_candidate_algorithm_id
        || !protocol_preserved
        || !function_ids
        || !contract.future_inputs_read_only_point_in_time_and_content_addressed
        || !contract.future_cycle_claim_first_required
        || !contract.future_cycle_create_once_required
        || !contract.future_output_create_once_and_untrusted
        || !contract.future_output_independent_validation_required
        || contract.future_output_contains_order_intent
        || contract.future_output_contains_broker_payload
        || !contract.registered_not_run
        || !contract.independent_implementation_review_required
        || !contract.isolated_runner_registration_required_after_review
        || !authority_closed(&contract.authority_boundary)
    {
        return Err("Stage 84 零能力观察实现合同无效、漂移或越权".to_string());
    }
    Ok(())
}

pub(crate) fn validate_controlled_shadow_forward_observation_implementation_for_review(
    record: &ControlledShadowForwardObservationImplementationRecord,
) -> Result<(), String> {
    let required = {
        let mut values = record
            .upstream_protocol_review
            .excluded_prior_actor_ids
            .clone();
        values.push(record.upstream_protocol_review.reviewer_id.clone());
        values.push(record.upstream_protocol_registration.registered_by.clone());
        values.sort();
        values.dedup();
        values
    };
    let texts_valid = [
        &record.implementation_description,
        &record.deterministic_observation_semantics,
        &record.evidence_custody_and_correction_semantics,
        &record.known_limitations,
        &record.future_review_constraints,
    ]
    .into_iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let name_and_revision_valid = !record.implementation_name.trim().is_empty()
        && record.implementation_name.chars().count() <= MAX_NAME_CHARS
        && !record
            .implementation_contract
            .immutable_code_revision
            .trim()
            .is_empty()
        && record
            .implementation_contract
            .immutable_code_revision
            .chars()
            .count()
            <= MAX_REVISION_CHARS;
    let closed = !record.forward_observation_authorized
        && !record.forward_observation_ledger_created
        && !record.shadow_position_written
        && !record.performance_metric_written
        && !record.model_artifact_store_written
        && !record.metric_store_written
        && !record.training_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || record.status != STATUS
        || !valid_id(&record.implementation_id)
        || !valid_sha256(&record.implementation_sha256)
        || !name_and_revision_valid
        || record.implementation_id != record.implementation_sha256[..32]
        || record.implementation_sha256 != record_fingerprint(record)?
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || required
            .iter()
            .any(|actor| !record.excluded_prior_actor_ids.contains(actor))
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.registered_by)
        || !record.registrar_independent_from_stage_83_and_complete_prior_chain
        || !record.exact_current_stage_51_through_stage_83_binding_confirmed
        || !record.registrar_independent_from_stage_83_and_complete_prior_chain_confirmed
        || !record.independent_recomputation_of_review_registration_protocol_and_design_confirmed
        || !record.zero_capability_specification_only_no_executable_artifact_confirmed
        || !record.natural_forward_no_backfill_and_observation_not_before_preserved_confirmed
        || !record.weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed
        || !record.official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed
        || !record.next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed
        || !record.checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed
        || !record.deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed
        || !record.no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed
        || !record.no_production_read_write_observation_ledger_position_or_performance_write_confirmed
        || !record.no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed
        || !record.future_independent_implementation_review_required_before_runner_registration_confirmed
        || !record.no_unconfirmed_hari_or_old_wang_logic_claimed
        || !record.zero_capability_forward_observation_implementation_registered
        || !record.future_independent_implementation_review_eligible
        || record.independent_implementation_review_completed
        || record.isolated_runner_registration_eligible
        || !texts_valid
        || !closed
    {
        return Err("Stage 84 零能力观察实现记录无效、漂移或越权".to_string());
    }
    validate_source_parts(
        &record.upstream_protocol_registration,
        &record.upstream_protocol_review,
    )?;
    validate_contract(
        &record.implementation_contract,
        &record.upstream_protocol_registration,
        &record.upstream_protocol_review,
    )
}

fn validate_record(
    record: &ControlledShadowForwardObservationImplementationRecord,
) -> Result<(), String> {
    validate_controlled_shadow_forward_observation_implementation_for_review(record)
}

fn record_matches_source(
    record: &ControlledShadowForwardObservationImplementationRecord,
    source: &IndependentlyApprovedControlledShadowForwardObservationProtocol,
) -> bool {
    validate_record(record).is_ok()
        && record.upstream_protocol_review.review_id == source.review.review_id
        && record.upstream_protocol_review.review_sha256 == source.review.review_sha256
        && record
            .upstream_protocol_registration
            .protocol_registration_id
            == source
                .registered_protocol
                .registration
                .protocol_registration_id
        && record
            .upstream_protocol_registration
            .protocol_registration_sha256
            == source
                .registered_protocol
                .registration
                .protocol_registration_sha256
}

fn excluded_actor_ids(
    source: &IndependentlyApprovedControlledShadowForwardObservationProtocol,
) -> Vec<String> {
    let mut actors = source.review.excluded_prior_actor_ids.clone();
    actors.push(source.review.reviewer_id.clone());
    actors.push(
        source
            .registered_protocol
            .registration
            .registered_by
            .clone(),
    );
    actors.extend(
        source
            .registered_protocol
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
    value: &ControlledShadowForwardObservationImplementationRecord,
) -> Result<String, String> {
    fingerprint_without(value, &["implementation_id", "implementation_sha256"])
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowForwardObservationImplementationRecord>, String> {
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
                let record: ControlledShadowForwardObservationImplementationRecord =
                    serde_json::from_slice(
                        &tokio::fs::read(&path)
                            .await
                            .map_err(|error| error.to_string())?,
                    )
                    .map_err(|error| error.to_string())?;
                validate_record(&record)?;
                if path.file_stem().and_then(|value| value.to_str()) != Some("implementation") {
                    return Err("Stage 84 实现记录文件名无效".to_string());
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
            || !review_ids.insert(record.upstream_protocol_review.review_id.clone())
        {
            return Err("Stage 84 实现记录重复".to_string());
        }
    }
    records.sort_by(|left, right| right.registered_at.cmp(&left.registered_at));
    Ok(records)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 84 指纹载荷无效".to_string())?;
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
    decision_root(state).join("controlled-shadow-forward-observation-implementations")
}

fn record_path(state: &AppState, protocol_review_id: &str) -> PathBuf {
    record_root(state)
        .join(protocol_review_id)
        .join("implementation.json")
}

struct RegistrationLock {
    path: PathBuf,
}

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    protocol_review_id: &str,
) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-forward-observation-implementation-{protocol_review_id}.lock"
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
        .map_err(|_| "已有同一 Stage 84 实现登记正在进行".to_string())?;
    Ok(RegistrationLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterControlledShadowForwardObservationImplementationRequest {
        RegisterControlledShadowForwardObservationImplementationRequest {
            expected_protocol_review_id: "a".repeat(32),
            expected_protocol_review_sha256: "a".repeat(64),
            expected_protocol_registration_id: "b".repeat(32),
            expected_protocol_registration_sha256: "b".repeat(64),
            expected_protocol_specification_sha256: "c".repeat(64),
            expected_validation_sha256: "d".repeat(64),
            expected_claim_sha256: "e".repeat(64),
            expected_result_sha256: "f".repeat(64),
            expected_output_sha256: "1".repeat(64),
            expected_input_manifest_sha256: "2".repeat(64),
            expected_authorization_review_sha256: "3".repeat(64),
            expected_isolated_runner_spec_sha256: "4".repeat(64),
            expected_runner_artifact_sha256: "5".repeat(64),
            expected_implementation_contract_sha256: "6".repeat(64),
            expected_design_specification_sha256: "7".repeat(64),
            expected_candidate_set_sha256: "8".repeat(64),
            expected_feature_order_sha256: "9".repeat(64),
            expected_preprocessing_sha256: "a".repeat(64),
            expected_target_id: "excess_return_252d".to_string(),
            expected_frozen_candidate_algorithm_id: "ridge".to_string(),
            implementation_name: "自然前向观察零能力实现".to_string(),
            immutable_code_revision: "revision-1".to_string(),
            implementation_description: "只冻结实现语义。".to_string(),
            deterministic_observation_semantics: "所有转换必须确定性重放。".to_string(),
            evidence_custody_and_correction_semantics: "来源内容寻址，更正只追加。".to_string(),
            known_limitations: "尚无真实前向样本。".to_string(),
            future_review_constraints: "独立复核后仍只能登记隔离 runner。".to_string(),
            exact_current_stage_51_through_stage_83_binding_confirmed: true,
            registrar_independent_from_stage_83_and_complete_prior_chain_confirmed: true,
            independent_recomputation_of_review_registration_protocol_and_design_confirmed: true,
            zero_capability_specification_only_no_executable_artifact_confirmed: true,
            natural_forward_no_backfill_and_observation_not_before_preserved_confirmed: true,
            weekly_claim_first_create_once_and_point_in_time_input_preserved_confirmed: true,
            official_market_calendar_spy_sync_and_corporate_actions_preserved_confirmed: true,
            next_full_session_25bps_cost_counterfactual_and_long_only_caps_preserved_confirmed: true,
            checkpoints_minimum_samples_separate_metrics_multiple_testing_and_stop_preserved_confirmed: true,
            deterministic_content_addressed_input_claim_output_and_correction_contract_confirmed: true,
            no_entrypoint_artifact_runtime_mount_adapter_environment_secret_network_tool_or_subprocess_confirmed: true,
            no_production_read_write_observation_ledger_position_or_performance_write_confirmed: true,
            no_model_metric_training_feedback_composite_reward_order_broker_or_trading_confirmed: true,
            future_independent_implementation_review_required_before_runner_registration_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn registration_requires_every_stage_84_confirmation() {
        let mut value = request();
        assert!(all_confirmations(&value));
        value.natural_forward_no_backfill_and_observation_not_before_preserved_confirmed = false;
        assert!(!all_confirmations(&value));
    }

    #[test]
    fn zero_capability_boundary_closes_every_authority() {
        let boundary = zero_capability_authority_boundary();
        assert!(authority_closed(&boundary));
        assert!(!boundary.executable_artifact_present);
        assert!(!boundary.forward_observation_write_allowed);
        assert!(!boundary.trading_allowed);
    }

    #[test]
    fn fingerprints_bind_implementation_semantics() {
        #[derive(Serialize)]
        struct FingerprintFixture {
            implementation_id: String,
            implementation_sha256: String,
            semantics: String,
        }
        let value = FingerprintFixture {
            implementation_id: String::new(),
            implementation_sha256: String::new(),
            semantics: "claim-first".to_string(),
        };
        let before =
            fingerprint_without(&value, &["implementation_id", "implementation_sha256"]).unwrap();
        let changed = FingerprintFixture {
            semantics: "backfill".to_string(),
            ..value
        };
        let after =
            fingerprint_without(&changed, &["implementation_id", "implementation_sha256"]).unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn ids_and_text_bounds_fail_closed() {
        assert!(valid_id(&"a".repeat(32)));
        assert!(!valid_id(&"a".repeat(31)));
        assert!(valid_sha256(&"b".repeat(64)));
        assert!(!valid_sha256("not-a-hash"));
        assert!(bounded_required("", 10, "说明").is_err());
        assert!(bounded_required("十一字字符串超长", 4, "说明").is_err());
    }
}
