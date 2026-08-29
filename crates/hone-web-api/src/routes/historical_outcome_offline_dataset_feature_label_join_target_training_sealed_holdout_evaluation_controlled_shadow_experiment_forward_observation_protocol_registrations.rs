//! Stage 82 create-once forward-observation protocol registration.
//!
//! This freezes how future, naturally elapsed observations may later be collected.
//! Registration does not observe a session, create a ledger or position, calculate
//! performance, or grant model-store, feedback, reward, order, broker or trading access.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_design_registrations::{
    ControlledShadowExperimentDesignSpecification, ControlledShadowExperimentExecutionContract,
    ControlledShadowExperimentMetricContract,
    ControlledShadowExperimentObservationContract, ControlledShadowExperimentPortfolioConstraints,
    ControlledShadowExperimentStopContract,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_first_execution_authorizations::exact_controlled_shadow_first_execution_authorization_for_audit;
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_output_validations::{
    ControlledShadowExperimentOutputValidationRecord,
    IndependentlyValidatedControlledShadowInitialObservation,
    independently_validated_controlled_shadow_initial_observations_for_protocol_registration,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-registration-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-forward-observation-protocol-registration-v1";
const SPEC_SCHEMA_VERSION: &str = "hone-controlled-shadow-forward-observation-protocol-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-natural-forward-only-protocol-create-once-v1-no-observation";
const MAX_TEXT_CHARS: usize = 3_600;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowForwardObservationProtocolRequest {
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
    protocol_rationale: String,
    source_custody_plan: String,
    market_calendar_plan: String,
    corporate_action_correction_policy: String,
    stop_execution_plan: String,
    exact_stage_51_through_stage_81_binding_confirmed: bool,
    registrar_independent_from_stage_81_and_complete_prior_chain_confirmed: bool,
    natural_forward_only_no_backfill_confirmed: bool,
    weekly_claim_first_content_addressed_observation_confirmed: bool,
    official_us_market_calendar_and_spy_synchronization_confirmed: bool,
    point_in_time_allowlisted_source_custody_confirmed: bool,
    adjusted_prices_dividends_and_append_only_corrections_confirmed: bool,
    next_full_session_fill_and_registered_costs_confirmed: bool,
    checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed: bool,
    stop_rules_fail_closed_and_no_in_place_restart_confirmed: bool,
    independent_protocol_review_required_before_observation_confirmed: bool,
    no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed:
        bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationProtocolSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub natural_forward_only: bool,
    pub retroactive_backfill_allowed: bool,
    pub observation_not_before: DateTime<Utc>,
    pub signal_cadence: String,
    pub first_eligible_signal_rule: String,
    pub per_cycle_claim_first_required: bool,
    pub per_cycle_create_once_required: bool,
    pub point_in_time_content_addressed_allowlisted_sources_required: bool,
    pub official_market_calendar: String,
    pub benchmark_symbol: String,
    pub synchronized_security_and_benchmark_observation_required: bool,
    pub price_basis: String,
    pub corporate_action_evidence_required: bool,
    pub correction_policy: String,
    pub source_custody_plan: String,
    pub market_calendar_plan: String,
    pub stop_execution_plan: String,
    pub exact_design_specification: ControlledShadowExperimentDesignSpecification,
    pub portfolio_constraints: ControlledShadowExperimentPortfolioConstraints,
    pub execution_contract: ControlledShadowExperimentExecutionContract,
    pub observation_contract: ControlledShadowExperimentObservationContract,
    pub metric_contract: ControlledShadowExperimentMetricContract,
    pub stop_contract: ControlledShadowExperimentStopContract,
    pub performance_before_natural_checkpoint_allowed: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowForwardObservationProtocolRegistration {
    pub schema_version: String,
    pub policy_version: String,
    pub protocol_registration_id: String,
    pub protocol_registration_sha256: String,
    pub attempt_id: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub input_manifest_sha256: String,
    pub authorization_review_sha256: String,
    pub isolated_runner_spec_sha256: String,
    pub runner_artifact_sha256: String,
    pub implementation_contract_sha256: String,
    pub design_specification_sha256: String,
    pub candidate_set_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub protocol_rationale: String,
    pub protocol_specification: ControlledShadowForwardObservationProtocolSpecification,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_81_and_complete_prior_chain: bool,
    pub exact_stage_51_through_stage_81_binding_confirmed: bool,
    pub natural_forward_only_no_backfill_confirmed: bool,
    pub weekly_claim_first_content_addressed_observation_confirmed: bool,
    pub official_us_market_calendar_and_spy_synchronization_confirmed: bool,
    pub point_in_time_allowlisted_source_custody_confirmed: bool,
    pub adjusted_prices_dividends_and_append_only_corrections_confirmed: bool,
    pub next_full_session_fill_and_registered_costs_confirmed: bool,
    pub checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed: bool,
    pub stop_rules_fail_closed_and_no_in_place_restart_confirmed: bool,
    pub independent_protocol_review_required_before_observation_confirmed: bool,
    pub no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed:
        bool,
    pub protocol_registered: bool,
    pub future_independent_protocol_review_eligible: bool,
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
pub(crate) struct ControlledShadowForwardObservationProtocolRegistrationItem {
    pub source: IndependentlyValidatedControlledShadowInitialObservation,
    pub registration: Option<ControlledShadowForwardObservationProtocolRegistration>,
    pub registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowForwardObservationProtocolRegistrationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowForwardObservationProtocolRegistrationItem>,
    pub protocol_registration_eligible_count: usize,
    pub protocol_registered_count: usize,
    pub current_binding_count: usize,
    pub future_independent_protocol_review_eligible_count: usize,
    pub protocol_registration_status: String,
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
pub(crate) struct ControlledShadowForwardObservationProtocolRegistrationReadinessSummary {
    pub protocol_registration_eligible_count: usize,
    pub protocol_registered_count: usize,
    pub current_binding_count: usize,
    pub future_independent_protocol_review_eligible_count: usize,
    pub protocol_registration_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RegisteredControlledShadowForwardObservationProtocolForIndependentReview {
    pub source: IndependentlyValidatedControlledShadowInitialObservation,
    pub registration: ControlledShadowForwardObservationProtocolRegistration,
}

pub(crate) async fn registered_controlled_shadow_forward_observation_protocols_for_independent_review(
    state: &AppState,
) -> Result<Vec<RegisteredControlledShadowForwardObservationProtocolForIndependentReview>, String> {
    let registry = build_registry(state).await?;
    let mut output = registry
        .items
        .into_iter()
        .filter_map(|item| {
            item.registration.and_then(|registration| {
                (registration.protocol_registered
                    && registration.future_independent_protocol_review_eligible)
                    .then_some(
                        RegisteredControlledShadowForwardObservationProtocolForIndependentReview {
                            source: item.source,
                            registration,
                        },
                    )
            })
        })
        .collect::<Vec<_>>();
    output.sort_by(|left, right| {
        right
            .registration
            .registered_at
            .cmp(&left.registration.registered_at)
    });
    Ok(output)
}

pub(crate) async fn handle_get_controlled_shadow_forward_observation_protocol_registrations(
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
            warn!(%error, "controlled shadow forward-observation protocol registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "前向观察协议登记暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_forward_observation_protocol(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(validation_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowForwardObservationProtocolRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match register_protocol(&state, &admin.user_id, &validation_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_forward_observation_protocol_registration_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationProtocolRegistrationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowForwardObservationProtocolRegistrationReadinessSummary {
            protocol_registration_eligible_count: value.protocol_registration_eligible_count,
            protocol_registered_count: value.protocol_registered_count,
            current_binding_count: value.current_binding_count,
            future_independent_protocol_review_eligible_count: value
                .future_independent_protocol_review_eligible_count,
            protocol_registration_status: value.protocol_registration_status,
        },
    )
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowForwardObservationProtocolRegistrationRegistry, String> {
    let sources =
        independently_validated_controlled_shadow_initial_observations_for_protocol_registration(
            state,
        )
        .await?;
    let records = read_records(state).await?;
    let mut items = Vec::new();
    for source in sources {
        let registration = records
            .iter()
            .find(|value| value.validation_id == source.validation.validation_id)
            .cloned();
        items.push(ControlledShadowForwardObservationProtocolRegistrationItem {
            registration_eligible: registration.is_none(),
            source,
            registration,
        });
    }
    let protocol_registration_eligible_count = items
        .iter()
        .filter(|value| value.registration_eligible)
        .count();
    let protocol_registered_count = items
        .iter()
        .filter(|value| value.registration.is_some())
        .count();
    let current_binding_count = records.len();
    let status = if current_binding_count != protocol_registered_count {
        "forward_observation_protocol_binding_mismatch_fail_closed"
    } else if protocol_registered_count > 0 {
        "forward_observation_protocol_registered_pending_independent_review"
    } else if protocol_registration_eligible_count > 0 {
        "waiting_forward_observation_protocol_registration"
    } else {
        "waiting_stage_81_independently_validated_initial_observation"
    };
    Ok(ControlledShadowForwardObservationProtocolRegistrationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(), items,
        protocol_registration_eligible_count, protocol_registered_count, current_binding_count,
        future_independent_protocol_review_eligible_count: protocol_registered_count,
        protocol_registration_status: status.to_string(), forward_observation_authorized: false,
        forward_observation_ledger_created: false, shadow_position_written: false,
        performance_metric_written: false, model_artifact_store_written: false,
        metric_store_written: false, training_feedback_authorized: false, reward_authorized: false,
        order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        scope: "Stage 82 只把 Stage 81 独立验证通过的零前向日初始观察，登记为自然时间向前、逐周期 claim-first、内容寻址和可独立复核的观察协议。协议冻结交易日历、SPY 同步基准、点时来源、复权与公司行动、更正留痕、成本、检查点、样本门槛和停止规则；当前不开始观察、不回填、不创建账本/持仓/绩效，不写模型/指标库，不开放反馈、reward、订单、券商或交易。".to_string(),
    })
}

async fn register_protocol(
    state: &AppState,
    registrar_id: &str,
    validation_id: &str,
    request: RegisterControlledShadowForwardObservationProtocolRequest,
) -> Result<ControlledShadowForwardObservationProtocolRegistration, String> {
    if !valid_id(validation_id) {
        return Err("Stage 81 validation ID 无效".to_string());
    }
    let _lock = acquire_lock(state, validation_id).await?;
    let source =
        independently_validated_controlled_shadow_initial_observations_for_protocol_registration(
            state,
        )
        .await?
        .into_iter()
        .find(|value| value.validation.validation_id == validation_id)
        .ok_or_else(|| "没有精确匹配且通过 Stage 81 的初始观察".to_string())?;
    if read_records(state)
        .await?
        .iter()
        .any(|value| value.validation_id == validation_id)
    {
        return Err("该 Stage 81 校验已登记协议，禁止覆盖或重放".to_string());
    }
    validate_expected_binding(&source.validation, &request)?;
    let mut excluded = source.validation.excluded_prior_actor_ids.clone();
    excluded.push(source.validation.validated_by.clone());
    excluded.push(source.validation.invoked_by.clone());
    excluded.sort();
    excluded.dedup();
    let independent =
        !registrar_id.trim().is_empty() && excluded.iter().all(|value| value != registrar_id);
    if !independent
        || !request.registrar_independent_from_stage_81_and_complete_prior_chain_confirmed
    {
        return Err("协议登记人必须独立于 Stage 81 校验者、执行者和完整上游责任链".to_string());
    }
    if !all_confirmations(&request) {
        return Err("必须确认全部前向观察与零权限边界".to_string());
    }
    let protocol_rationale = bounded_required(&request.protocol_rationale, "协议依据")?;
    let source_custody_plan = bounded_required(&request.source_custody_plan, "来源保管方案")?;
    let market_calendar_plan = bounded_required(&request.market_calendar_plan, "交易日历方案")?;
    let correction_policy = bounded_required(
        &request.corporate_action_correction_policy,
        "公司行动更正政策",
    )?;
    let stop_execution_plan = bounded_required(&request.stop_execution_plan, "停止执行方案")?;
    let authorization = exact_controlled_shadow_first_execution_authorization_for_audit(
        state,
        &source.attempt.claim.isolated_runner_id,
        &source.attempt.claim.authorization_review_id,
        &source.attempt.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "无法重开 Stage 79 授权以核对冻结设计".to_string())?;
    let design = &authorization
        .runner
        .implementation
        .implementation_contract
        .exact_design_specification;
    if design.specification_sha256 != source.validation.design_specification_sha256 {
        return Err("Stage 74 设计与 Stage 81 绑定不一致".to_string());
    }
    let registered_at = Utc::now();
    let mut specification = ControlledShadowForwardObservationProtocolSpecification {
        schema_version: SPEC_SCHEMA_VERSION.to_string(), specification_sha256: String::new(), natural_forward_only: true,
        retroactive_backfill_allowed: false, observation_not_before: registered_at,
        signal_cadence: "每周一次；每周期必须先形成不可变 claim，随后才可打开该时点输入".to_string(),
        first_eligible_signal_rule: "协议独立复核通过后，首个自然到来的美股完整周度信号截止点；不得回填登记或批准之前的任何日期".to_string(),
        per_cycle_claim_first_required: true, per_cycle_create_once_required: true,
        point_in_time_content_addressed_allowlisted_sources_required: true,
        official_market_calendar: "证券主上市交易所官方美国市场完整交易日历；半日市与停牌必须显式标记".to_string(),
        benchmark_symbol: design.benchmark_symbol.clone(), synchronized_security_and_benchmark_observation_required: true,
        price_basis: "证券与 SPY 使用同一市场时点的可复现复权收盘价；原始收盘、拆股、分红和公司行动证据必须同时内容寻址留存".to_string(),
        corporate_action_evidence_required: true, correction_policy, source_custody_plan, market_calendar_plan, stop_execution_plan,
        exact_design_specification: design.clone(),
        portfolio_constraints: design.portfolio_constraints.clone(), execution_contract: design.execution_contract.clone(),
        observation_contract: design.observation_contract.clone(), metric_contract: design.metric_contract.clone(),
        stop_contract: design.stop_contract.clone(), performance_before_natural_checkpoint_allowed: false,
        forward_observation_started: false, ledger_created: false,
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])?;
    let mut record = ControlledShadowForwardObservationProtocolRegistration {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        protocol_registration_id: String::new(),
        protocol_registration_sha256: String::new(),
        attempt_id: source.attempt.claim.attempt_id.clone(),
        validation_id: validation_id.to_string(),
        validation_sha256: source.validation.validation_sha256.clone(),
        claim_sha256: source.validation.claim_sha256.clone(),
        result_sha256: source.validation.result_sha256.clone(),
        output_sha256: source.validation.output_sha256.clone(),
        input_manifest_sha256: source.validation.input_manifest_sha256.clone(),
        authorization_review_sha256: source.validation.authorization_review_sha256.clone(),
        isolated_runner_spec_sha256: source.validation.isolated_runner_spec_sha256.clone(),
        runner_artifact_sha256: source.validation.runner_artifact_sha256.clone(),
        implementation_contract_sha256: source.validation.implementation_contract_sha256.clone(),
        design_specification_sha256: source.validation.design_specification_sha256.clone(),
        candidate_set_sha256: source.validation.candidate_set_sha256.clone(),
        feature_order_sha256: source.validation.feature_order_sha256.clone(),
        preprocessing_sha256: source.validation.preprocessing_sha256.clone(),
        target_id: source.validation.target_id.clone(),
        frozen_candidate_algorithm_id: source.validation.frozen_candidate_algorithm_id.clone(),
        protocol_rationale,
        protocol_specification: specification,
        registered_at,
        registered_by: registrar_id.to_string(),
        excluded_prior_actor_ids: excluded,
        registrar_independent_from_stage_81_and_complete_prior_chain: independent,
        exact_stage_51_through_stage_81_binding_confirmed: request
            .exact_stage_51_through_stage_81_binding_confirmed,
        natural_forward_only_no_backfill_confirmed: request
            .natural_forward_only_no_backfill_confirmed,
        weekly_claim_first_content_addressed_observation_confirmed: request
            .weekly_claim_first_content_addressed_observation_confirmed,
        official_us_market_calendar_and_spy_synchronization_confirmed: request
            .official_us_market_calendar_and_spy_synchronization_confirmed,
        point_in_time_allowlisted_source_custody_confirmed: request
            .point_in_time_allowlisted_source_custody_confirmed,
        adjusted_prices_dividends_and_append_only_corrections_confirmed: request
            .adjusted_prices_dividends_and_append_only_corrections_confirmed,
        next_full_session_fill_and_registered_costs_confirmed: request
            .next_full_session_fill_and_registered_costs_confirmed,
        checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed: request
            .checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed,
        stop_rules_fail_closed_and_no_in_place_restart_confirmed: request
            .stop_rules_fail_closed_and_no_in_place_restart_confirmed,
        independent_protocol_review_required_before_observation_confirmed: request
            .independent_protocol_review_required_before_observation_confirmed,
        no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed:
            request.no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed,
        protocol_registered: true,
        future_independent_protocol_review_eligible: true,
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
    record.protocol_registration_sha256 = fingerprint_without(
        &record,
        &["protocol_registration_id", "protocol_registration_sha256"],
    )?;
    record.protocol_registration_id = record.protocol_registration_sha256[..32].to_string();
    validate_record(&record)?;
    write_immutable_json(&record_path(state, validation_id), &record).await?;
    Ok(record)
}

fn validate_expected_binding(
    v: &ControlledShadowExperimentOutputValidationRecord,
    r: &RegisterControlledShadowForwardObservationProtocolRequest,
) -> Result<(), String> {
    if r.expected_validation_sha256 != v.validation_sha256
        || r.expected_claim_sha256 != v.claim_sha256
        || r.expected_result_sha256 != v.result_sha256
        || r.expected_output_sha256 != v.output_sha256
        || r.expected_input_manifest_sha256 != v.input_manifest_sha256
        || r.expected_authorization_review_sha256 != v.authorization_review_sha256
        || r.expected_isolated_runner_spec_sha256 != v.isolated_runner_spec_sha256
        || r.expected_runner_artifact_sha256 != v.runner_artifact_sha256
        || r.expected_implementation_contract_sha256 != v.implementation_contract_sha256
        || r.expected_design_specification_sha256 != v.design_specification_sha256
        || r.expected_candidate_set_sha256 != v.candidate_set_sha256
        || r.expected_feature_order_sha256 != v.feature_order_sha256
        || r.expected_preprocessing_sha256 != v.preprocessing_sha256
        || r.expected_target_id != v.target_id
        || r.expected_frozen_candidate_algorithm_id != v.frozen_candidate_algorithm_id
    {
        return Err("Stage 82 预期绑定与 Stage 81 不一致".to_string());
    }
    Ok(())
}

fn all_confirmations(r: &RegisterControlledShadowForwardObservationProtocolRequest) -> bool {
    r.exact_stage_51_through_stage_81_binding_confirmed && r.natural_forward_only_no_backfill_confirmed
        && r.weekly_claim_first_content_addressed_observation_confirmed && r.official_us_market_calendar_and_spy_synchronization_confirmed
        && r.point_in_time_allowlisted_source_custody_confirmed && r.adjusted_prices_dividends_and_append_only_corrections_confirmed
        && r.next_full_session_fill_and_registered_costs_confirmed && r.checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed
        && r.stop_rules_fail_closed_and_no_in_place_restart_confirmed && r.independent_protocol_review_required_before_observation_confirmed
        && r.no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed
}

fn validate_record(
    r: &ControlledShadowForwardObservationProtocolRegistration,
) -> Result<(), String> {
    let spec = &r.protocol_specification;
    let closed = !r.forward_observation_authorized
        && !r.forward_observation_ledger_created
        && !r.shadow_position_written
        && !r.performance_metric_written
        && !r.model_artifact_store_written
        && !r.metric_store_written
        && !r.training_feedback_authorized
        && !r.reward_authorized
        && !r.order_generation_authorized
        && !r.broker_access_authorized
        && !r.trading_authorized
        && !spec.forward_observation_started
        && !spec.ledger_created;
    if r.schema_version != RECORD_SCHEMA_VERSION
        || r.policy_version != POLICY_VERSION
        || !valid_id(&r.protocol_registration_id)
        || r.protocol_registration_sha256
            != fingerprint_without(
                r,
                &["protocol_registration_id", "protocol_registration_sha256"],
            )?
        || r.protocol_registration_id != r.protocol_registration_sha256[..32]
        || !sorted_unique(&r.excluded_prior_actor_ids)
        || r.excluded_prior_actor_ids
            .iter()
            .any(|value| value == &r.registered_by)
        || !r.registrar_independent_from_stage_81_and_complete_prior_chain
        || !r.exact_stage_51_through_stage_81_binding_confirmed
        || !r.natural_forward_only_no_backfill_confirmed
        || !r.weekly_claim_first_content_addressed_observation_confirmed
        || !r.official_us_market_calendar_and_spy_synchronization_confirmed
        || !r.point_in_time_allowlisted_source_custody_confirmed
        || !r.adjusted_prices_dividends_and_append_only_corrections_confirmed
        || !r.next_full_session_fill_and_registered_costs_confirmed
        || !r.checkpoints_minimum_samples_metrics_and_counterfactuals_preserved_confirmed
        || !r.stop_rules_fail_closed_and_no_in_place_restart_confirmed
        || !r.independent_protocol_review_required_before_observation_confirmed
        || !r.no_ledger_position_performance_model_metric_feedback_reward_order_broker_or_trading_confirmed
        || !r.protocol_registered
        || !r.future_independent_protocol_review_eligible
        || !closed
        || spec.schema_version != SPEC_SCHEMA_VERSION
        || spec.specification_sha256 != fingerprint_without(spec, &["specification_sha256"])?
        || !spec.natural_forward_only
        || spec.retroactive_backfill_allowed
        || !spec.per_cycle_claim_first_required
        || !spec.per_cycle_create_once_required
        || !spec.point_in_time_content_addressed_allowlisted_sources_required
        || spec.benchmark_symbol != "SPY"
        || !spec.synchronized_security_and_benchmark_observation_required
        || !spec.corporate_action_evidence_required
        || spec.exact_design_specification.specification_sha256
            != r.design_specification_sha256
        || spec.portfolio_constraints != spec.exact_design_specification.portfolio_constraints
        || spec.execution_contract != spec.exact_design_specification.execution_contract
        || spec.observation_contract != spec.exact_design_specification.observation_contract
        || spec.metric_contract != spec.exact_design_specification.metric_contract
        || spec.stop_contract != spec.exact_design_specification.stop_contract
        || spec.performance_before_natural_checkpoint_allowed
        || spec.execution_contract.slippage_bps_per_side != 25
        || spec
            .execution_contract
            .lookahead_or_retroactive_revision_allowed
        || spec.observation_contract.minimum_forward_market_sessions != 252
        || spec.observation_contract.checkpoint_market_sessions != vec![21, 63, 126, 252]
        || spec.observation_contract.minimum_independent_signal_count != 40
        || spec.observation_contract.minimum_distinct_symbol_count != 12
        || spec
            .observation_contract
            .minimum_distinct_market_quarter_count
            != 4
        || spec.observation_contract.early_promotion_allowed
        || spec.metric_contract.composite_score_allowed
        || !spec.metric_contract.all_metrics_reported_separately
        || spec
            .stop_contract
            .automatic_trade_or_position_action_allowed
        || spec.stop_contract.stopped_design_can_be_restarted_in_place
    {
        return Err("Stage 82 协议记录无效、漂移或越权".to_string());
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowForwardObservationProtocolRegistration>, String> {
    let mut records = Vec::new();
    let mut stack = vec![protocol_root(state)];
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
            let kind = entry.file_type().await.map_err(|error| error.to_string())?;
            if kind.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
                let record: ControlledShadowForwardObservationProtocolRegistration =
                    serde_json::from_slice(
                        &tokio::fs::read(&path)
                            .await
                            .map_err(|error| error.to_string())?,
                    )
                    .map_err(|error| error.to_string())?;
                validate_record(&record)?;
                records.push(record);
            }
        }
    }
    let mut ids = BTreeSet::new();
    let mut validations = BTreeSet::new();
    for r in &records {
        if !ids.insert(r.protocol_registration_id.clone())
            || !validations.insert(r.validation_id.clone())
        {
            return Err("Stage 82 协议记录重复".to_string());
        }
    }
    records.sort_by(|a, b| b.registered_at.cmp(&a.registered_at));
    Ok(records)
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"))
    } else {
        Ok(value.to_string())
    }
}
fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|e| e.to_string())?)
    ))
}
fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|v| v.is_ascii_hexdigit())
}
fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|p| p[0] < p[1])
}
fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn protocol_root(state: &AppState) -> PathBuf {
    decision_root(state).join("controlled-shadow-forward-observation-protocol-registrations")
}
fn record_path(state: &AppState, validation_id: &str) -> PathBuf {
    protocol_root(state)
        .join(validation_id)
        .join("registration.json")
}

struct ProtocolLock {
    path: PathBuf,
}
impl Drop for ProtocolLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
async fn acquire_lock(state: &AppState, validation_id: &str) -> Result<ProtocolLock, String> {
    let dir = protocol_root(state).join("locks");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| e.to_string())?;
    let path = dir.join(format!("{validation_id}.lock"));
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "另一个 Stage 82 登记正在处理".to_string()
            } else {
                e.to_string()
            }
        })?;
    file.sync_all().await.map_err(|e| e.to_string())?;
    Ok(ProtocolLock { path })
}
async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|e| e.to_string())?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;
    file.sync_all().await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn role_separation_is_fail_closed() {
        let excluded = ["stage-81-validator".to_string()];
        assert!(excluded.iter().any(|v| v == "stage-81-validator"));
        assert!(excluded.iter().all(|v| v != "stage-82-registrar"));
    }
    #[test]
    fn immutable_hash_changes_on_tamper() {
        #[derive(Serialize)]
        struct Sample {
            a: bool,
            b: u16,
        }
        let one = fingerprint_without(&Sample { a: true, b: 252 }, &[]).unwrap();
        let two = fingerprint_without(&Sample { a: true, b: 251 }, &[]).unwrap();
        assert_ne!(one, two);
    }
    #[test]
    fn fixed_minimum_gate_is_one_year_and_four_quarters() {
        let observation = ControlledShadowExperimentObservationContract {
            minimum_forward_market_sessions: 252,
            checkpoint_market_sessions: vec![21, 63, 126, 252],
            minimum_independent_signal_count: 40,
            minimum_distinct_symbol_count: 12,
            minimum_distinct_market_quarter_count: 4,
            early_promotion_allowed: false,
        };
        assert_eq!(observation.minimum_forward_market_sessions, 252);
        assert_eq!(observation.minimum_distinct_market_quarter_count, 4);
        assert!(!observation.early_promotion_allowed);
    }
}
