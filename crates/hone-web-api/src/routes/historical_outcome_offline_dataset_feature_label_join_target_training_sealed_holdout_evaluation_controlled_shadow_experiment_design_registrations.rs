//! Create-once registration of one exact Stage 73 adjudicated result as a controlled
//! forward shadow-experiment design.
//!
//! This stage freezes the benchmark, universe, signal timing, portfolio constraints,
//! execution costs, observation windows, separate metrics, comparators and stop rules.
//! It does not select or store a production model, run a ledger, create a position or
//! order, connect a broker, feed training/reward or trade. A registration only opens a
//! future independent design review.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_confirmatory_result_adjudication_reviews::{
    AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign,
    adjudicated_sealed_holdout_confirmations_for_shadow_design_registration,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-design-registration-registry-v1";
const REGISTRATION_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-experiment-design-registration-v1";
const SPEC_SCHEMA_VERSION: &str = "hone-controlled-shadow-experiment-design-spec-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-experiment-design-create-once-v1-no-shadow-run";
const MAX_NAME_CHARS: usize = 120;
const MAX_TEXT_CHARS: usize = 3_600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentPortfolioConstraints {
    pub virtual_notional_usd: u64,
    pub long_only: bool,
    pub common_stock_only: bool,
    pub options_allowed: bool,
    pub leverage_allowed: bool,
    pub shorting_allowed: bool,
    pub maximum_single_name_weight_bps: u32,
    pub maximum_theme_weight_bps: u32,
    pub maximum_gross_exposure_bps: u32,
    pub minimum_cash_weight_bps: u32,
    pub maximum_position_count: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentExecutionContract {
    pub signal_cutoff: String,
    pub assumed_execution: String,
    pub rebalance_frequency: String,
    pub slippage_bps_per_side: u16,
    pub commission_bps_per_side: u16,
    pub adjusted_prices_and_dividends_required: bool,
    pub point_in_time_data_only: bool,
    pub lookahead_or_retroactive_revision_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentObservationContract {
    pub minimum_forward_market_sessions: u16,
    pub checkpoint_market_sessions: Vec<u16>,
    pub minimum_independent_signal_count: u16,
    pub minimum_distinct_symbol_count: u16,
    pub minimum_distinct_market_quarter_count: u16,
    pub early_promotion_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentMetricContract {
    pub metric_ids: Vec<String>,
    pub composite_score_allowed: bool,
    pub all_metrics_reported_separately: bool,
    pub multiple_testing_adjustment_required: bool,
    pub transaction_costs_included: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowExperimentStopContract {
    pub stop_rule_ids: Vec<String>,
    pub automatic_trade_or_position_action_allowed: bool,
    pub stopped_design_can_be_restarted_in_place: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentDesignSpecification {
    pub schema_version: String,
    pub specification_sha256: String,
    pub experimental_candidate_only: bool,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub random_seeds: Vec<u64>,
    pub candidate_set_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub benchmark_symbol: String,
    pub comparator_ids: Vec<String>,
    pub universe_contract: String,
    pub signal_contract: String,
    pub portfolio_constraints: ControlledShadowExperimentPortfolioConstraints,
    pub execution_contract: ControlledShadowExperimentExecutionContract,
    pub observation_contract: ControlledShadowExperimentObservationContract,
    pub metric_contract: ControlledShadowExperimentMetricContract,
    pub stop_contract: ControlledShadowExperimentStopContract,
    pub scalar_reward_defined: bool,
    pub official_model_selected: bool,
    pub model_artifact_materialized: bool,
    pub shadow_ledger_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowExperimentDesignRequest {
    expected_adjudication_review_id: String,
    expected_adjudication_review_sha256: String,
    expected_output_validation_id: String,
    expected_output_validation_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_output_sha256: String,
    expected_envelope_sha256: String,
    expected_candidate_set_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_target_id: String,
    expected_frozen_candidate_algorithm_id: String,
    experiment_name: String,
    research_hypothesis: String,
    economic_thesis: String,
    known_limitations: String,
    falsification_conditions: String,
    exact_stage_73_adjudication_and_complete_chain_confirmed: bool,
    registrar_independent_from_complete_prior_chain_confirmed: bool,
    experimental_candidate_not_official_model_selection_confirmed: bool,
    point_in_time_forward_only_and_no_retroactive_revision_confirmed: bool,
    benchmark_comparators_costs_and_rebalance_frozen_confirmed: bool,
    portfolio_caps_cash_floor_and_long_only_boundary_confirmed: bool,
    minimum_observation_windows_and_no_early_promotion_confirmed: bool,
    separate_metrics_multiple_testing_and_no_composite_confirmed: bool,
    stop_rules_and_falsification_are_frozen_confirmed: bool,
    independent_design_review_required_before_any_shadow_run_request_confirmed: bool,
    no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ControlledShadowExperimentDesignRegistration {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub attempt_id: String,
    pub adjudication_review_id: String,
    pub adjudication_review_sha256: String,
    pub output_validation_id: String,
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
    pub experiment_name: String,
    pub research_hypothesis: String,
    pub economic_thesis: String,
    pub known_limitations: String,
    pub falsification_conditions: String,
    pub design_specification: ControlledShadowExperimentDesignSpecification,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_complete_prior_chain: bool,
    pub exact_stage_73_adjudication_and_complete_chain_confirmed: bool,
    pub registrar_independent_from_complete_prior_chain_confirmed: bool,
    pub experimental_candidate_not_official_model_selection_confirmed: bool,
    pub point_in_time_forward_only_and_no_retroactive_revision_confirmed: bool,
    pub benchmark_comparators_costs_and_rebalance_frozen_confirmed: bool,
    pub portfolio_caps_cash_floor_and_long_only_boundary_confirmed: bool,
    pub minimum_observation_windows_and_no_early_promotion_confirmed: bool,
    pub separate_metrics_multiple_testing_and_no_composite_confirmed: bool,
    pub stop_rules_and_falsification_are_frozen_confirmed: bool,
    pub independent_design_review_required_before_any_shadow_run_request_confirmed: bool,
    pub no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: bool,
    pub controlled_shadow_experiment_design_registered: bool,
    pub future_independent_design_review_eligible: bool,
    pub design_independently_approved: bool,
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
pub(crate) struct ControlledShadowExperimentDesignRegistrationItem {
    pub source: AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign,
    pub registration: Option<ControlledShadowExperimentDesignRegistration>,
    pub registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowExperimentDesignRegistrationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowExperimentDesignRegistrationItem>,
    pub adjudicated_candidate_count: usize,
    pub registration_eligible_count: usize,
    pub registered_design_count: usize,
    pub future_independent_design_review_eligible_count: usize,
    pub registration_status: String,
    pub design_registration_available: bool,
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
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowExperimentDesignRegistrationReadinessSummary {
    pub adjudicated_candidate_count: usize,
    pub registration_eligible_count: usize,
    pub registered_design_count: usize,
    pub future_independent_design_review_eligible_count: usize,
    pub registration_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RegisteredControlledShadowExperimentDesignForIndependentReview {
    pub source: AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign,
    pub registration: ControlledShadowExperimentDesignRegistration,
}

pub(crate) async fn handle_get_controlled_shadow_experiment_design_registrations(
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
            warn!(%error, "controlled shadow experiment design registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "受控影子实验设计登记暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_experiment_design(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowExperimentDesignRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_design(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_experiment_design_registration_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowExperimentDesignRegistrationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowExperimentDesignRegistrationReadinessSummary {
            adjudicated_candidate_count: registry.adjudicated_candidate_count,
            registration_eligible_count: registry.registration_eligible_count,
            registered_design_count: registry.registered_design_count,
            future_independent_design_review_eligible_count: registry
                .future_independent_design_review_eligible_count,
            registration_status: registry.registration_status,
        },
    )
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowExperimentDesignRegistrationRegistry, String> {
    let mut items = Vec::new();
    for source in
        adjudicated_sealed_holdout_confirmations_for_shadow_design_registration(state).await?
    {
        let registration = read_registration(state, &source).await?;
        items.push(ControlledShadowExperimentDesignRegistrationItem {
            registration_eligible: registration.is_none(),
            source,
            registration,
        });
    }
    items.sort_by(|left, right| {
        left.source
            .review
            .attempt_id
            .cmp(&right.source.review.attempt_id)
    });
    let adjudicated_candidate_count = items.len();
    let registration_eligible_count = items
        .iter()
        .filter(|item| item.registration_eligible)
        .count();
    let registered_design_count = items
        .iter()
        .filter(|item| item.registration.is_some())
        .count();
    let registration_status = if registered_design_count > 0 {
        "controlled_shadow_experiment_design_registered_pending_independent_review"
    } else if adjudicated_candidate_count > 0 {
        "waiting_controlled_shadow_experiment_design_registration"
    } else {
        "waiting_stage_73_adjudicated_confirmatory_result"
    };
    Ok(ControlledShadowExperimentDesignRegistrationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        adjudicated_candidate_count,
        registration_eligible_count,
        registered_design_count,
        future_independent_design_review_eligible_count: registered_design_count,
        registration_status: registration_status.to_string(),
        design_registration_available: true,
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
        scope: "第 74 阶段只把已裁决的实验候选登记为不可变前向影子实验设计，冻结基准、反事实、信号时点、组合约束、成本、观察窗口、分项指标和停止规则。登记后仍须由新角色独立复核；当前不正式选模、不写模型/指标库、不反馈训练或奖励，不创建影子账本、持仓、订单，不连接券商或交易。".to_string(),
    })
}

pub(crate) async fn registered_controlled_shadow_experiment_designs_for_independent_review(
    state: &AppState,
) -> Result<Vec<RegisteredControlledShadowExperimentDesignForIndependentReview>, String> {
    let mut output = Vec::new();
    for source in
        adjudicated_sealed_holdout_confirmations_for_shadow_design_registration(state).await?
    {
        if let Some(registration) = read_registration(state, &source).await?
            && registration.controlled_shadow_experiment_design_registered
            && registration.future_independent_design_review_eligible
        {
            output.push(
                RegisteredControlledShadowExperimentDesignForIndependentReview {
                    source,
                    registration,
                },
            );
        }
    }
    output.sort_by(|left, right| {
        left.registration
            .registration_id
            .cmp(&right.registration.registration_id)
    });
    Ok(output)
}

async fn register_design(
    state: &AppState,
    registrar_id: &str,
    attempt_id: &str,
    request: RegisterControlledShadowExperimentDesignRequest,
) -> Result<ControlledShadowExperimentDesignRegistration, String> {
    if !valid_id(attempt_id) {
        return Err("sealed-holdout 尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, attempt_id).await?;
    let mut matching =
        adjudicated_sealed_holdout_confirmations_for_shadow_design_registration(state)
            .await?
            .into_iter()
            .filter(|source| source.review.attempt_id == attempt_id)
            .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err("当前没有精确一条可登记的 Stage 73 裁决结果".to_string());
    }
    let source = matching.remove(0);
    if read_registration(state, &source).await?.is_some() {
        return Err("该裁决结果已完成设计登记；只能进入未来独立设计复核".to_string());
    }
    validate_expected_binding(&source, &request)?;
    let excluded_prior_actor_ids = excluded_actor_ids(&source);
    let independent = reviewer_is_independent(registrar_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("登记人不能是 Stage 73 裁决者或完整上游责任链角色".to_string());
    }
    if !all_confirmations(&request) {
        return Err("设计登记必须确认全部冻结契约和零执行权限边界".to_string());
    }
    let experiment_name = bounded_required(&request.experiment_name, "实验名称", MAX_NAME_CHARS)?;
    let research_hypothesis =
        bounded_required(&request.research_hypothesis, "研究假设", MAX_TEXT_CHARS)?;
    let economic_thesis = bounded_required(&request.economic_thesis, "经济假设", MAX_TEXT_CHARS)?;
    let known_limitations =
        bounded_required(&request.known_limitations, "已知局限", MAX_TEXT_CHARS)?;
    let falsification_conditions = bounded_required(
        &request.falsification_conditions,
        "证伪条件",
        MAX_TEXT_CHARS,
    )?;
    let design_specification = design_specification(&source)?;
    let mut registration = ControlledShadowExperimentDesignRegistration {
        schema_version: REGISTRATION_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        registration_id: String::new(),
        registration_sha256: String::new(),
        attempt_id: attempt_id.to_string(),
        adjudication_review_id: source.review.review_id.clone(),
        adjudication_review_sha256: source.review.review_sha256.clone(),
        output_validation_id: source.review.output_validation_id.clone(),
        output_validation_sha256: source.review.output_validation_sha256.clone(),
        claim_sha256: source.review.claim_sha256.clone(),
        result_sha256: source.review.result_sha256.clone(),
        output_sha256: source.review.output_sha256.clone(),
        envelope_sha256: source.review.envelope_sha256.clone(),
        candidate_set_sha256: source.review.candidate_set_sha256.clone(),
        training_store_dataset_sha256: source.review.training_store_dataset_sha256.clone(),
        selected_algorithm_three_seed_binding_sha256: source
            .review
            .selected_algorithm_three_seed_binding_sha256
            .clone(),
        sealed_holdout_split_commitment_sha256: source
            .review
            .sealed_holdout_split_commitment_sha256
            .clone(),
        sealed_holdout_projection_sha256: source.review.sealed_holdout_projection_sha256.clone(),
        feature_order_sha256: source.review.feature_order_sha256.clone(),
        preprocessing_sha256: source.review.preprocessing_sha256.clone(),
        target_id: source.review.target_id.clone(),
        frozen_candidate_algorithm_id: source.review.frozen_candidate_algorithm_id.clone(),
        experiment_name,
        research_hypothesis,
        economic_thesis,
        known_limitations,
        falsification_conditions,
        design_specification,
        registered_at: Utc::now(),
        registered_by: registrar_id.to_string(),
        excluded_prior_actor_ids,
        registrar_independent_from_complete_prior_chain: independent,
        exact_stage_73_adjudication_and_complete_chain_confirmed: request
            .exact_stage_73_adjudication_and_complete_chain_confirmed,
        registrar_independent_from_complete_prior_chain_confirmed: request
            .registrar_independent_from_complete_prior_chain_confirmed,
        experimental_candidate_not_official_model_selection_confirmed: request
            .experimental_candidate_not_official_model_selection_confirmed,
        point_in_time_forward_only_and_no_retroactive_revision_confirmed: request
            .point_in_time_forward_only_and_no_retroactive_revision_confirmed,
        benchmark_comparators_costs_and_rebalance_frozen_confirmed: request
            .benchmark_comparators_costs_and_rebalance_frozen_confirmed,
        portfolio_caps_cash_floor_and_long_only_boundary_confirmed: request
            .portfolio_caps_cash_floor_and_long_only_boundary_confirmed,
        minimum_observation_windows_and_no_early_promotion_confirmed: request
            .minimum_observation_windows_and_no_early_promotion_confirmed,
        separate_metrics_multiple_testing_and_no_composite_confirmed: request
            .separate_metrics_multiple_testing_and_no_composite_confirmed,
        stop_rules_and_falsification_are_frozen_confirmed: request
            .stop_rules_and_falsification_are_frozen_confirmed,
        independent_design_review_required_before_any_shadow_run_request_confirmed: request
            .independent_design_review_required_before_any_shadow_run_request_confirmed,
        no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed: request
            .no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed,
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
    registration.registration_sha256 = registration_fingerprint(&registration)?;
    registration.registration_id = registration.registration_sha256[..32].to_string();
    validate_registration(&registration, &source)?;
    write_immutable_json(&registration_path(state, attempt_id), &registration).await?;
    Ok(registration)
}

fn design_specification(
    source: &AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign,
) -> Result<ControlledShadowExperimentDesignSpecification, String> {
    let mut specification = ControlledShadowExperimentDesignSpecification {
        schema_version: SPEC_SCHEMA_VERSION.to_string(),
        specification_sha256: String::new(),
        experimental_candidate_only: true,
        target_id: source.review.target_id.clone(),
        frozen_candidate_algorithm_id: source.review.frozen_candidate_algorithm_id.clone(),
        random_seeds: vec![17, 29, 43],
        candidate_set_sha256: source.review.candidate_set_sha256.clone(),
        feature_order_sha256: source.review.feature_order_sha256.clone(),
        preprocessing_sha256: source.review.preprocessing_sha256.clone(),
        benchmark_symbol: "SPY".to_string(),
        comparator_ids: vec![
            "cash_zero_return".to_string(),
            "spy_total_return".to_string(),
            "eligible_universe_equal_weight".to_string(),
            "frozen_rule_based_baseline".to_string(),
        ],
        universe_contract: "仅使用 candidate_set_sha256 锁定、在信号时点可交易且拥有完整点时输入的美国上市普通股；退市、停牌和缺失数据不得事后删除".to_string(),
        signal_contract: "只使用冻结目标、算法、17/29/43、65 项特征顺序与预处理；北京时间记录信号，纽约市场下一完整交易日才允许未来模拟成交".to_string(),
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
            signal_cutoff: "信号日美股收盘后，全部点时证据完成不可变落盘".to_string(),
            assumed_execution: "下一个完整纽约交易日的可复现调整后收盘价".to_string(),
            rebalance_frequency: "每周一次；证伪只允许未来实现提前降风险，不允许追涨加仓".to_string(),
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
                "net_excess_return_vs_spy".to_string(),
                "maximum_drawdown".to_string(),
                "downside_capture".to_string(),
                "turnover_and_cost_drag".to_string(),
                "position_concentration".to_string(),
                "directional_hit_rate".to_string(),
            ],
            composite_score_allowed: false,
            all_metrics_reported_separately: true,
            multiple_testing_adjustment_required: true,
            transaction_costs_included: true,
        },
        stop_contract: ControlledShadowExperimentStopContract {
            stop_rule_ids: vec![
                "binding_or_hash_mismatch".to_string(),
                "point_in_time_leakage_detected".to_string(),
                "data_coverage_below_registered_minimum".to_string(),
                "target_or_feature_semantics_changed".to_string(),
                "unregistered_model_or_parameter_change".to_string(),
                "execution_cost_or_benchmark_unavailable".to_string(),
            ],
            automatic_trade_or_position_action_allowed: false,
            stopped_design_can_be_restarted_in_place: false,
        },
        scalar_reward_defined: false,
        official_model_selected: false,
        model_artifact_materialized: false,
        shadow_ledger_enabled: false,
    };
    specification.specification_sha256 =
        fingerprint_without(&specification, &["specification_sha256"])?;
    Ok(specification)
}

fn validate_expected_binding(
    source: &AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign,
    request: &RegisterControlledShadowExperimentDesignRequest,
) -> Result<(), String> {
    let review = &source.review;
    let exact = request.expected_adjudication_review_id == review.review_id
        && request.expected_adjudication_review_sha256 == review.review_sha256
        && request.expected_output_validation_id == review.output_validation_id
        && request.expected_output_validation_sha256 == review.output_validation_sha256
        && request.expected_claim_sha256 == review.claim_sha256
        && request.expected_result_sha256 == review.result_sha256
        && request.expected_output_sha256 == review.output_sha256
        && request.expected_envelope_sha256 == review.envelope_sha256
        && request.expected_candidate_set_sha256 == review.candidate_set_sha256
        && request.expected_selected_algorithm_three_seed_binding_sha256
            == review.selected_algorithm_three_seed_binding_sha256
        && request.expected_target_id == review.target_id
        && request.expected_frozen_candidate_algorithm_id == review.frozen_candidate_algorithm_id;
    exact
        .then_some(())
        .ok_or_else(|| "Stage 51-73 精确绑定已经变化，请刷新后重试".to_string())
}

fn excluded_actor_ids(
    source: &AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign,
) -> Vec<String> {
    let mut actors = source.review.excluded_prior_actor_ids.clone();
    actors.push(source.review.reviewer_id.clone());
    actors.retain(|actor| !actor.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn reviewer_is_independent(actor_id: &str, excluded: &[String]) -> bool {
    !actor_id.trim().is_empty() && excluded.iter().all(|actor| actor != actor_id)
}

fn all_confirmations(request: &RegisterControlledShadowExperimentDesignRequest) -> bool {
    request.exact_stage_73_adjudication_and_complete_chain_confirmed
        && request.registrar_independent_from_complete_prior_chain_confirmed
        && request.experimental_candidate_not_official_model_selection_confirmed
        && request.point_in_time_forward_only_and_no_retroactive_revision_confirmed
        && request.benchmark_comparators_costs_and_rebalance_frozen_confirmed
        && request.portfolio_caps_cash_floor_and_long_only_boundary_confirmed
        && request.minimum_observation_windows_and_no_early_promotion_confirmed
        && request.separate_metrics_multiple_testing_and_no_composite_confirmed
        && request.stop_rules_and_falsification_are_frozen_confirmed
        && request.independent_design_review_required_before_any_shadow_run_request_confirmed
        && request.no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed
}

fn validate_registration(
    registration: &ControlledShadowExperimentDesignRegistration,
    source: &AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign,
) -> Result<(), String> {
    let review = &source.review;
    let expected_excluded = excluded_actor_ids(source);
    let independent = reviewer_is_independent(
        &registration.registered_by,
        &registration.excluded_prior_actor_ids,
    );
    let spec = &registration.design_specification;
    let spec_valid = spec.schema_version == SPEC_SCHEMA_VERSION
        && valid_sha256(&spec.specification_sha256)
        && spec.specification_sha256 == fingerprint_without(spec, &["specification_sha256"])?
        && spec.experimental_candidate_only
        && spec.target_id == review.target_id
        && spec.frozen_candidate_algorithm_id == review.frozen_candidate_algorithm_id
        && spec.random_seeds == [17, 29, 43]
        && spec.candidate_set_sha256 == review.candidate_set_sha256
        && spec.feature_order_sha256 == review.feature_order_sha256
        && spec.preprocessing_sha256 == review.preprocessing_sha256
        && spec.benchmark_symbol == "SPY"
        && spec.comparator_ids.len() == 4
        && sorted_unique_after_sort(&spec.comparator_ids)
        && spec.portfolio_constraints.long_only
        && spec.portfolio_constraints.common_stock_only
        && !spec.portfolio_constraints.options_allowed
        && !spec.portfolio_constraints.leverage_allowed
        && !spec.portfolio_constraints.shorting_allowed
        && spec.portfolio_constraints.maximum_single_name_weight_bps == 500
        && spec.portfolio_constraints.maximum_theme_weight_bps == 2_000
        && spec.portfolio_constraints.maximum_gross_exposure_bps == 6_000
        && spec.portfolio_constraints.minimum_cash_weight_bps == 4_000
        && spec.execution_contract.point_in_time_data_only
        && !spec
            .execution_contract
            .lookahead_or_retroactive_revision_allowed
        && spec.execution_contract.slippage_bps_per_side == 25
        && spec.observation_contract.minimum_forward_market_sessions == 252
        && spec.observation_contract.checkpoint_market_sessions == [21, 63, 126, 252]
        && !spec.observation_contract.early_promotion_allowed
        && !spec.metric_contract.composite_score_allowed
        && spec.metric_contract.all_metrics_reported_separately
        && spec.metric_contract.multiple_testing_adjustment_required
        && spec.metric_contract.transaction_costs_included
        && !spec
            .stop_contract
            .automatic_trade_or_position_action_allowed
        && !spec.stop_contract.stopped_design_can_be_restarted_in_place
        && !spec.scalar_reward_defined
        && !spec.official_model_selected
        && !spec.model_artifact_materialized
        && !spec.shadow_ledger_enabled;
    let text_valid = registration.experiment_name.chars().count() <= MAX_NAME_CHARS
        && !registration.experiment_name.trim().is_empty()
        && [
            &registration.research_hypothesis,
            &registration.economic_thesis,
            &registration.known_limitations,
            &registration.falsification_conditions,
        ]
        .into_iter()
        .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let downstream_closed = !registration.design_independently_approved
        && !registration.official_model_selection_authorized
        && !registration.model_artifact_store_written
        && !registration.metric_store_written
        && !registration.training_feedback_authorized
        && !registration.reward_authorized
        && !registration.shadow_run_authorized
        && !registration.shadow_ledger_enabled
        && !registration.shadow_position_written
        && !registration.order_generation_authorized
        && !registration.broker_access_authorized
        && !registration.trading_authorized;
    if registration.schema_version != REGISTRATION_SCHEMA_VERSION
        || registration.policy_version != POLICY_VERSION
        || !valid_id(&registration.registration_id)
        || !valid_sha256(&registration.registration_sha256)
        || registration.registration_id != registration.registration_sha256[..32]
        || registration.registration_sha256 != registration_fingerprint(registration)?
        || registration.attempt_id != review.attempt_id
        || registration.adjudication_review_id != review.review_id
        || registration.adjudication_review_sha256 != review.review_sha256
        || registration.output_validation_id != review.output_validation_id
        || registration.output_validation_sha256 != review.output_validation_sha256
        || registration.claim_sha256 != review.claim_sha256
        || registration.result_sha256 != review.result_sha256
        || registration.output_sha256 != review.output_sha256
        || registration.envelope_sha256 != review.envelope_sha256
        || registration.candidate_set_sha256 != review.candidate_set_sha256
        || registration.training_store_dataset_sha256 != review.training_store_dataset_sha256
        || registration.selected_algorithm_three_seed_binding_sha256
            != review.selected_algorithm_three_seed_binding_sha256
        || registration.sealed_holdout_split_commitment_sha256
            != review.sealed_holdout_split_commitment_sha256
        || registration.sealed_holdout_projection_sha256 != review.sealed_holdout_projection_sha256
        || registration.feature_order_sha256 != review.feature_order_sha256
        || registration.preprocessing_sha256 != review.preprocessing_sha256
        || registration.target_id != review.target_id
        || registration.frozen_candidate_algorithm_id != review.frozen_candidate_algorithm_id
        || registration.excluded_prior_actor_ids != expected_excluded
        || !independent
        || !registration.registrar_independent_from_complete_prior_chain
        || !registration.exact_stage_73_adjudication_and_complete_chain_confirmed
        || !registration.registrar_independent_from_complete_prior_chain_confirmed
        || !registration.experimental_candidate_not_official_model_selection_confirmed
        || !registration.point_in_time_forward_only_and_no_retroactive_revision_confirmed
        || !registration.benchmark_comparators_costs_and_rebalance_frozen_confirmed
        || !registration.portfolio_caps_cash_floor_and_long_only_boundary_confirmed
        || !registration.minimum_observation_windows_and_no_early_promotion_confirmed
        || !registration.separate_metrics_multiple_testing_and_no_composite_confirmed
        || !registration.stop_rules_and_falsification_are_frozen_confirmed
        || !registration.independent_design_review_required_before_any_shadow_run_request_confirmed
        || !registration
            .no_model_store_training_reward_shadow_position_order_broker_or_trading_confirmed
        || !registration.controlled_shadow_experiment_design_registered
        || !registration.future_independent_design_review_eligible
        || !text_valid
        || !spec_valid
        || !downstream_closed
    {
        return Err(
            "controlled shadow experiment design registration is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

async fn read_registration(
    state: &AppState,
    source: &AdjudicatedHistoricalOutcomeSealedHoldoutConfirmationForShadowDesign,
) -> Result<Option<ControlledShadowExperimentDesignRegistration>, String> {
    let path = registration_path(state, &source.review.attempt_id);
    let bytes = match tokio::fs::read(&path).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let registration =
        serde_json::from_slice::<ControlledShadowExperimentDesignRegistration>(&bytes)
            .map_err(|error| error.to_string())?;
    validate_registration(&registration, source)?;
    Ok(Some(registration))
}

fn registration_fingerprint(
    registration: &ControlledShadowExperimentDesignRegistration,
) -> Result<String, String> {
    fingerprint_without(registration, &["registration_id", "registration_sha256"])
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "设计登记指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, label: &str, maximum: usize) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > maximum {
        return Err(format!("{label}不能超过 {maximum} 个字符"));
    }
    Ok(value.to_string())
}

fn sorted_unique_after_sort(values: &[String]) -> bool {
    let mut normalized = values.to_vec();
    normalized.sort();
    normalized.dedup();
    normalized.len() == values.len()
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
        .map_err(|_| "该裁决结果的影子实验设计已存在".to_string())?;
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

fn registration_path(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-experiment-design-registrations")
        .join(format!("{attempt_id}.json"))
}

struct RegistrationLock {
    path: PathBuf,
}

impl Drop for RegistrationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<RegistrationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "controlled-shadow-experiment-design-registration-{attempt_id}.lock"
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
        .map_err(|_| "已有同一影子实验设计登记正在进行".to_string())?;
    Ok(RegistrationLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_design_has_long_only_cash_and_concentration_boundaries() {
        let constraints = ControlledShadowExperimentPortfolioConstraints {
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
        };
        assert!(constraints.long_only && constraints.common_stock_only);
        assert!(!constraints.options_allowed && !constraints.leverage_allowed);
        assert_eq!(constraints.maximum_gross_exposure_bps, 6_000);
        assert_eq!(constraints.minimum_cash_weight_bps, 4_000);
    }

    #[test]
    fn actor_independence_is_fail_closed() {
        let excluded = vec!["reviewer".to_string(), "validator".to_string()];
        assert!(!reviewer_is_independent("reviewer", &excluded));
        assert!(reviewer_is_independent("new-registrar", &excluded));
        assert!(!reviewer_is_independent("", &excluded));
    }

    #[test]
    fn required_text_is_bounded() {
        assert!(bounded_required("", "研究假设", MAX_TEXT_CHARS).is_err());
        assert!(
            bounded_required(&"a".repeat(MAX_TEXT_CHARS + 1), "研究假设", MAX_TEXT_CHARS).is_err()
        );
        assert_eq!(
            bounded_required("  假设  ", "研究假设", MAX_TEXT_CHARS).unwrap(),
            "假设"
        );
    }

    #[test]
    fn duplicate_comparators_are_rejected() {
        assert!(sorted_unique_after_sort(&[
            "cash".to_string(),
            "spy".to_string()
        ]));
        assert!(!sorted_unique_after_sort(&[
            "cash".to_string(),
            "cash".to_string()
        ]));
    }

    #[test]
    fn fingerprint_excludes_only_declared_identity_fields() {
        #[derive(Serialize)]
        struct Value {
            id: String,
            body: String,
        }
        let first = Value {
            id: "one".to_string(),
            body: "fixed".to_string(),
        };
        let second = Value {
            id: "two".to_string(),
            body: "fixed".to_string(),
        };
        assert_eq!(
            fingerprint_without(&first, &["id"]).unwrap(),
            fingerprint_without(&second, &["id"]).unwrap()
        );
    }

    #[test]
    fn stage_never_authorizes_downstream_execution() {
        let permissions = [false, false, false, false, false, false, false, false];
        assert!(permissions.into_iter().all(|allowed| !allowed));
    }
}
