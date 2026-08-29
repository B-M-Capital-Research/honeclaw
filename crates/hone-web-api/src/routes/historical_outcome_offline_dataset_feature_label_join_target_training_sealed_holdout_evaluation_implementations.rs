//! Immutable registration of a future per-target sealed-holdout evaluator implementation.
//!
//! Stage 67 binds one independently approved Stage 66 protocol to a content-addressed,
//! zero-capability implementation contract. It has no entrypoint, data mount or evaluator process.
//! Approval of the upstream protocol is not holdout access. Independent implementation review,
//! runner registration, one-shot access authorization, execution and output validation remain
//! separate future gates.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_protocol_reviews::{
    ApprovedHistoricalOutcomeSealedHoldoutEvaluationProtocol,
    HistoricalOutcomeSealedHoldoutEvaluationProtocol,
    HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
    approved_sealed_holdout_evaluation_protocols_for_implementation_registration,
    validate_approved_sealed_holdout_evaluation_protocol_for_implementation_registration,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-implementation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-implementation-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-historical-outcome-sealed-holdout-evaluation-implementation-contract-v1";
const POLICY_VERSION: &str =
    "hone-sealed-holdout-evaluation-implementation-registration-v1-zero-capability";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-sealed-holdout-one-target-three-seed-confirmatory-evaluator-v1";
const STATUS: &str = "registered_not_reviewed_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest {
    expected_protocol_review_id: String,
    expected_protocol_review_sha256: String,
    expected_protocol_sha256: String,
    expected_stage_65_admission_review_sha256: String,
    expected_output_validation_sha256: String,
    expected_candidate_set_sha256: String,
    expected_training_store_dataset_sha256: String,
    expected_target_bundle_sha256: String,
    expected_recommendation_sha256: String,
    expected_selected_algorithm_three_seed_binding_sha256: String,
    expected_sealed_holdout_split_commitment_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_through_stage_66_binding_confirmed: bool,
    registrar_independent_from_stage_66_and_complete_prior_chain_confirmed: bool,
    immutable_artifact_revision_protocol_and_serialization_confirmed: bool,
    one_target_one_algorithm_three_frozen_seeds_only_confirmed: bool,
    no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed: bool,
    one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed: bool,
    fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: bool,
    no_tuning_refit_reselection_or_cross_target_composite_confirmed: bool,
    future_output_create_once_untrusted_and_independent_validation_required_confirmed: bool,
    independent_review_runner_and_one_shot_authorization_remain_separate_confirmed: bool,
    no_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_protocol_version: String,
    pub implementation_artifact_sha256: String,
    pub immutable_code_revision: String,
    pub stage_66_protocol_review_id: String,
    pub stage_66_protocol_review_sha256: String,
    pub sealed_holdout_evaluation_protocol_sha256: String,
    pub stage_65_admission_review_sha256: String,
    pub output_validation_sha256: String,
    pub candidate_set_sha256: String,
    pub training_store_dataset_sha256: String,
    pub target_bundle_sha256: String,
    pub recommendation_sha256: String,
    pub selected_algorithm_three_seed_binding_sha256: String,
    pub sealed_holdout_split_commitment_sha256: String,
    pub feature_order_sha256: String,
    pub preprocessing_sha256: String,
    pub target_id: String,
    pub frozen_candidate_algorithm_id: String,
    pub exact_random_seeds: Vec<u64>,
    pub exact_feature_count: usize,
    pub exact_target_count: usize,
    pub exact_candidate_hypothesis_count: u32,
    pub reported_metric_ids: Vec<String>,
    pub bootstrap_unit: String,
    pub bootstrap_replications: u32,
    pub bootstrap_random_seed: u64,
    pub family_wise_error_correction: String,
    pub family_wise_alpha_millionths: u32,
    pub minimum_relative_mae_improvement_ppm: u32,
    pub minimum_spearman_millionths: i32,
    pub minimum_directional_accuracy_millionths: u32,
    pub minimum_calibration_slope_millionths: u32,
    pub maximum_calibration_slope_millionths: u32,
    pub minimum_sealed_holdout_rows: u32,
    pub minimum_independent_components: u32,
    pub all_three_seeds_must_pass: bool,
    pub one_shot_evaluation_required: bool,
    pub deterministic_evaluator_function_id: String,
    pub canonical_input_projection_schema: String,
    pub canonical_untrusted_output_schema: String,
    pub future_output_create_once: bool,
    pub future_output_independent_validation_required: bool,
    pub no_feedback_reuse: bool,
    pub insufficient_sample_fails_closed: bool,
    pub no_composite_score_or_cross_target_masking: bool,
    pub independent_implementation_review_required: bool,
    pub isolated_runner_registration_required: bool,
    pub one_shot_access_authorization_required: bool,
    pub callable_entrypoint_present: bool,
    pub input_mount_present: bool,
    pub sealed_holdout_data_adapter_present: bool,
    pub sealed_holdout_features_access_allowed: bool,
    pub sealed_holdout_labels_access_allowed: bool,
    pub sealed_holdout_evaluation_allowed: bool,
    pub training_or_preprocessing_update_allowed: bool,
    pub hyperparameter_or_threshold_tuning_allowed: bool,
    pub candidate_reselection_allowed: bool,
    pub official_candidate_selection_allowed: bool,
    pub model_store_write_allowed: bool,
    pub metric_store_write_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub outbound_network_allowed: bool,
    pub environment_inheritance_allowed: bool,
    pub secrets_allowed: bool,
    pub tools_allowed: bool,
    pub subprocesses_allowed: bool,
    pub scalar_reward_defined: bool,
    pub action_position_or_ranking_semantics_defined: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub upstream_protocol: HistoricalOutcomeSealedHoldoutEvaluationProtocol,
    pub upstream_protocol_review: HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_66_and_complete_prior_chain: bool,
    pub implementation_name: String,
    pub rationale: String,
    pub known_limitations: String,
    pub implementation_contract: HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
    pub status: String,
    pub exact_current_stage_51_through_stage_66_binding_confirmed: bool,
    pub registrar_independent_from_stage_66_and_complete_prior_chain_confirmed: bool,
    pub immutable_artifact_revision_protocol_and_serialization_confirmed: bool,
    pub one_target_one_algorithm_three_frozen_seeds_only_confirmed: bool,
    pub no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed: bool,
    pub one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed: bool,
    pub fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: bool,
    pub no_tuning_refit_reselection_or_cross_target_composite_confirmed: bool,
    pub future_output_create_once_untrusted_and_independent_validation_required_confirmed: bool,
    pub independent_review_runner_and_one_shot_authorization_remain_separate_confirmed: bool,
    pub no_selection_store_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub sealed_holdout_evaluation_implementation_registered: bool,
    pub future_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub isolated_runner_registration_eligible: bool,
    pub sealed_holdout_access_authorized: bool,
    pub sealed_holdout_evaluation_authorized: bool,
    pub official_candidate_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol {
    pub protocol: HistoricalOutcomeSealedHoldoutEvaluationProtocol,
    pub protocol_review: HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationItem {
    pub implementation: HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    pub upstream_binding_current: bool,
    pub future_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_protocols:
        Vec<HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol>,
    pub items: Vec<HistoricalOutcomeSealedHoldoutEvaluationImplementationItem>,
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub callable_entrypoint_present: bool,
    pub input_mount_present: bool,
    pub sealed_holdout_access_authorized: bool,
    pub sealed_holdout_evaluation_authorized: bool,
    pub official_candidate_selection_authorized: bool,
    pub model_artifact_store_written: bool,
    pub metric_store_written: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeSealedHoldoutEvaluationImplementationReadinessSummary {
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_historical_outcome_sealed_holdout_evaluation_implementations(
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
            warn!(%error, "sealed-holdout evaluation implementation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "sealed-holdout 评估实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_sealed_holdout_evaluation_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_sealed_holdout_evaluation_implementation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationImplementationReadinessSummary {
            registration_eligible_count: registry.registration_eligible_count,
            implementation_count: registry.implementation_count,
            current_binding_implementation_count: registry.current_binding_implementation_count,
            independent_implementation_review_eligible_count: registry
                .independent_implementation_review_eligible_count,
            implementation_status: registry.implementation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independent_review_eligible_sealed_holdout_evaluation_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord>, String> {
    let registry = build_registry(state).await?;
    let values = registry
        .items
        .into_iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .map(|item| item.implementation)
        .collect::<Vec<_>>();
    for value in &values {
        validate_record(value)?;
    }
    Ok(values)
}

#[allow(dead_code)]
pub(crate) fn validate_historical_outcome_sealed_holdout_evaluation_implementation_for_review(
    value: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
) -> Result<(), String> {
    validate_record(value)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry, String> {
    let eligible = eligible_sources(state).await?;
    let eligible_by_review = eligible
        .iter()
        .map(|source| (source.protocol_review.review_id.as_str(), source))
        .collect::<HashMap<_, _>>();
    let mut items = read_records(state)
        .await?
        .into_iter()
        .map(|implementation| {
            let upstream_binding_current = eligible_by_review
                .get(implementation.upstream_protocol_review.review_id.as_str())
                .is_some_and(|source| record_matches_source(&implementation, source));
            HistoricalOutcomeSealedHoldoutEvaluationImplementationItem {
                future_independent_implementation_review_eligible: upstream_binding_current,
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
                .upstream_protocol_review
                .review_id
                .as_str()
        })
        .collect::<BTreeSet<_>>();
    let registration_eligible_count = eligible
        .iter()
        .filter(|source| !registered_review_ids.contains(source.protocol_review.review_id.as_str()))
        .count();
    let current_binding_implementation_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .count();
    let implementation_status = if current_binding_implementation_count > 0 {
        "sealed_holdout_evaluation_implementation_registered_not_run_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "approved_protocol_ready_for_sealed_holdout_evaluation_implementation_registration"
    } else {
        "waiting_stage_66_independently_approved_sealed_holdout_evaluation_protocol"
    };
    Ok(HistoricalOutcomeSealedHoldoutEvaluationImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_protocols: eligible,
        registration_eligible_count,
        implementation_count: items.len(),
        current_binding_implementation_count,
        independent_implementation_review_eligible_count,
        items,
        implementation_status: implementation_status.to_string(),
        callable_entrypoint_present: false,
        input_mount_present: false,
        sealed_holdout_access_authorized: false,
        sealed_holdout_evaluation_authorized: false,
        official_candidate_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 67 阶段只把当前 Stage 66 独立批准的逐目标协议登记为不可变、内容寻址、零能力实现合同。登记冻结一种算法、三个种子、65/1 输入输出、统计门槛、序列化和未来 create-once 不可信输出；当前没有入口、挂载、数据 adapter、sealed-holdout 访问或评估。独立实现复核、runner、单次授权、执行和输出校验继续分门，正式选模、模型/指标库、reward、shadow、order、broker 与 trading 全部关闭。".to_string(),
    })
}

async fn eligible_sources(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol>, String> {
    approved_sealed_holdout_evaluation_protocols_for_implementation_registration(state)
        .await?
        .into_iter()
        .map(source_from_approved)
        .collect()
}

fn source_from_approved(
    approved: ApprovedHistoricalOutcomeSealedHoldoutEvaluationProtocol,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol, String> {
    validate_approved_sealed_holdout_evaluation_protocol_for_implementation_registration(
        &approved.subject.protocol,
        &approved.protocol_review,
    )?;
    Ok(
        HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol {
            protocol: approved.subject.protocol,
            protocol_review: approved.protocol_review,
        },
    )
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord, String> {
    if registrar_id.trim().is_empty() {
        return Err("sealed-holdout 评估实现登记人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let eligible = eligible_sources(state).await?;
    let source = eligible
        .iter()
        .find(|source| {
            source.protocol_review.review_id == request.expected_protocol_review_id
                && source.protocol.target_id == source.protocol_review.target_id
        })
        .ok_or_else(|| "当前 Stage 66 独立批准协议不存在，请刷新后重试".to_string())?;
    ensure_protocol_unregistered(&read_records(state).await?, source)?;
    let record = build_record(source, registrar_id, request, Utc::now())?;
    write_immutable_json(
        &record_root(state).join(format!("{}.json", record.implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn build_record(
    source: &HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord, String> {
    validate_approved_sealed_holdout_evaluation_protocol_for_implementation_registration(
        &source.protocol,
        &source.protocol_review,
    )?;
    validate_expected_bindings(source, &request)?;
    let implementation_name =
        bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    let immutable_code_revision = bounded_required(
        &request.immutable_code_revision,
        MAX_REVISION_CHARS,
        "不可变代码版本",
    )?;
    if !valid_sha256(&request.implementation_artifact_sha256) {
        return Err("sealed-holdout 评估实现工件 SHA-256 无效".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "登记理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    if !request_checks_confirmed(&request) {
        return Err("登记前必须逐项确认完整绑定、零能力、一次性规则和职责分离".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&source.protocol_review);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == registrar_id)
    {
        return Err("实现登记人参与过 Stage 51–66 或完整上游，必须更换独立角色".to_string());
    }
    let implementation_contract = implementation_contract(
        source,
        &request.implementation_artifact_sha256,
        &immutable_code_revision,
    )?;
    let mut record = HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        upstream_protocol: source.protocol.clone(),
        upstream_protocol_review: source.protocol_review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_stage_66_and_complete_prior_chain: true,
        implementation_name,
        rationale,
        known_limitations,
        implementation_contract,
        status: STATUS.to_string(),
        exact_current_stage_51_through_stage_66_binding_confirmed: request
            .exact_current_stage_51_through_stage_66_binding_confirmed,
        registrar_independent_from_stage_66_and_complete_prior_chain_confirmed: request
            .registrar_independent_from_stage_66_and_complete_prior_chain_confirmed,
        immutable_artifact_revision_protocol_and_serialization_confirmed: request
            .immutable_artifact_revision_protocol_and_serialization_confirmed,
        one_target_one_algorithm_three_frozen_seeds_only_confirmed: request
            .one_target_one_algorithm_three_frozen_seeds_only_confirmed,
        no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed: request
            .no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed,
        one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed: request
            .one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed,
        fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: request
            .fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed,
        no_tuning_refit_reselection_or_cross_target_composite_confirmed: request
            .no_tuning_refit_reselection_or_cross_target_composite_confirmed,
        future_output_create_once_untrusted_and_independent_validation_required_confirmed: request
            .future_output_create_once_untrusted_and_independent_validation_required_confirmed,
        independent_review_runner_and_one_shot_authorization_remain_separate_confirmed: request
            .independent_review_runner_and_one_shot_authorization_remain_separate_confirmed,
        no_selection_store_reward_shadow_order_broker_or_trading_confirmed: request
            .no_selection_store_reward_shadow_order_broker_or_trading_confirmed,
        sealed_holdout_evaluation_implementation_registered: true,
        future_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        isolated_runner_registration_eligible: false,
        sealed_holdout_access_authorized: false,
        sealed_holdout_evaluation_authorized: false,
        official_candidate_selection_authorized: false,
        model_artifact_store_written: false,
        metric_store_written: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
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
    source: &HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol,
    implementation_artifact_sha256: &str,
    immutable_code_revision: &str,
) -> Result<HistoricalOutcomeSealedHoldoutEvaluationImplementationContract, String> {
    let protocol = &source.protocol;
    let review = &source.protocol_review;
    let mut contract = HistoricalOutcomeSealedHoldoutEvaluationImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_protocol_version: IMPLEMENTATION_PROTOCOL_VERSION.to_string(),
        implementation_artifact_sha256: implementation_artifact_sha256.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        stage_66_protocol_review_id: review.review_id.clone(),
        stage_66_protocol_review_sha256: review.review_sha256.clone(),
        sealed_holdout_evaluation_protocol_sha256: protocol.protocol_sha256.clone(),
        stage_65_admission_review_sha256: protocol.stage_65_admission_review_sha256.clone(),
        output_validation_sha256: protocol.output_validation_sha256.clone(),
        candidate_set_sha256: protocol.candidate_set_sha256.clone(),
        training_store_dataset_sha256: protocol.training_store_dataset_sha256.clone(),
        target_bundle_sha256: protocol.target_bundle_sha256.clone(),
        recommendation_sha256: protocol.recommendation_sha256.clone(),
        selected_algorithm_three_seed_binding_sha256: protocol
            .selected_algorithm_three_seed_binding_sha256
            .clone(),
        sealed_holdout_split_commitment_sha256: protocol
            .sealed_holdout_split_commitment_sha256
            .clone(),
        feature_order_sha256: protocol.feature_order_sha256.clone(),
        preprocessing_sha256: protocol.preprocessing_sha256.clone(),
        target_id: protocol.target_id.clone(),
        frozen_candidate_algorithm_id: protocol.frozen_candidate_algorithm_id.clone(),
        exact_random_seeds: protocol.exact_random_seeds.clone(),
        exact_feature_count: protocol.exact_feature_count,
        exact_target_count: protocol.exact_target_count,
        exact_candidate_hypothesis_count: protocol.exact_candidate_hypothesis_count,
        reported_metric_ids: protocol.reported_metric_ids.clone(),
        bootstrap_unit: protocol.bootstrap_unit.clone(),
        bootstrap_replications: protocol.bootstrap_replications,
        bootstrap_random_seed: protocol.bootstrap_random_seed,
        family_wise_error_correction: protocol.family_wise_error_correction.clone(),
        family_wise_alpha_millionths: protocol.family_wise_alpha_millionths,
        minimum_relative_mae_improvement_ppm: protocol.minimum_relative_mae_improvement_ppm,
        minimum_spearman_millionths: protocol.minimum_spearman_millionths,
        minimum_directional_accuracy_millionths: protocol.minimum_directional_accuracy_millionths,
        minimum_calibration_slope_millionths: protocol.minimum_calibration_slope_millionths,
        maximum_calibration_slope_millionths: protocol.maximum_calibration_slope_millionths,
        minimum_sealed_holdout_rows: protocol.minimum_sealed_holdout_rows,
        minimum_independent_components: protocol.minimum_independent_components,
        all_three_seeds_must_pass: protocol.all_three_seeds_must_pass,
        one_shot_evaluation_required: protocol.one_shot_evaluation_required,
        deterministic_evaluator_function_id:
            "sealed_holdout_one_target_three_seed_confirmatory_evaluator_v1".to_string(),
        canonical_input_projection_schema:
            "sealed_holdout_projection_v1_exact_65_features_one_target_three_artifacts".to_string(),
        canonical_untrusted_output_schema:
            "sealed_holdout_untrusted_confirmatory_metrics_v1_three_seed_no_selection".to_string(),
        future_output_create_once: true,
        future_output_independent_validation_required: true,
        no_feedback_reuse: true,
        insufficient_sample_fails_closed: true,
        no_composite_score_or_cross_target_masking: true,
        independent_implementation_review_required: true,
        isolated_runner_registration_required: true,
        one_shot_access_authorization_required: true,
        callable_entrypoint_present: false,
        input_mount_present: false,
        sealed_holdout_data_adapter_present: false,
        sealed_holdout_features_access_allowed: false,
        sealed_holdout_labels_access_allowed: false,
        sealed_holdout_evaluation_allowed: false,
        training_or_preprocessing_update_allowed: false,
        hyperparameter_or_threshold_tuning_allowed: false,
        candidate_reselection_allowed: false,
        official_candidate_selection_allowed: false,
        model_store_write_allowed: false,
        metric_store_write_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        outbound_network_allowed: false,
        environment_inheritance_allowed: false,
        secrets_allowed: false,
        tools_allowed: false,
        subprocesses_allowed: false,
        scalar_reward_defined: false,
        action_position_or_ranking_semantics_defined: false,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn validate_expected_bindings(
    source: &HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol,
    request: &RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
) -> Result<(), String> {
    let protocol = &source.protocol;
    let review = &source.protocol_review;
    if request.expected_protocol_review_id != review.review_id
        || request.expected_protocol_review_sha256 != review.review_sha256
        || request.expected_protocol_sha256 != protocol.protocol_sha256
        || request.expected_stage_65_admission_review_sha256
            != protocol.stage_65_admission_review_sha256
        || request.expected_output_validation_sha256 != protocol.output_validation_sha256
        || request.expected_candidate_set_sha256 != protocol.candidate_set_sha256
        || request.expected_training_store_dataset_sha256 != protocol.training_store_dataset_sha256
        || request.expected_target_bundle_sha256 != protocol.target_bundle_sha256
        || request.expected_recommendation_sha256 != protocol.recommendation_sha256
        || request.expected_selected_algorithm_three_seed_binding_sha256
            != protocol.selected_algorithm_three_seed_binding_sha256
        || request.expected_sealed_holdout_split_commitment_sha256
            != protocol.sealed_holdout_split_commitment_sha256
    {
        return Err("Stage 65–66 协议、候选或数据绑定已变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn request_checks_confirmed(
    request: &RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest,
) -> bool {
    request.exact_current_stage_51_through_stage_66_binding_confirmed
        && request.registrar_independent_from_stage_66_and_complete_prior_chain_confirmed
        && request.immutable_artifact_revision_protocol_and_serialization_confirmed
        && request.one_target_one_algorithm_three_frozen_seeds_only_confirmed
        && request.no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed
        && request.one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed
        && request.fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed
        && request.no_tuning_refit_reselection_or_cross_target_composite_confirmed
        && request.future_output_create_once_untrusted_and_independent_validation_required_confirmed
        && request.independent_review_runner_and_one_shot_authorization_remain_separate_confirmed
        && request.no_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn excluded_actor_ids(
    review: &HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
) -> Vec<String> {
    let mut values = review.excluded_prior_actor_ids.clone();
    values.push(review.reviewer_id.clone());
    values.sort();
    values.dedup();
    values
}

fn validate_contract(
    contract: &HistoricalOutcomeSealedHoldoutEvaluationImplementationContract,
    protocol: &HistoricalOutcomeSealedHoldoutEvaluationProtocol,
    review: &HistoricalOutcomeSealedHoldoutEvaluationProtocolReview,
) -> Result<(), String> {
    let hashes_valid = [
        contract.contract_sha256.as_str(),
        contract.implementation_artifact_sha256.as_str(),
        contract.stage_66_protocol_review_sha256.as_str(),
        contract.sealed_holdout_evaluation_protocol_sha256.as_str(),
        contract.stage_65_admission_review_sha256.as_str(),
        contract.output_validation_sha256.as_str(),
        contract.candidate_set_sha256.as_str(),
        contract.training_store_dataset_sha256.as_str(),
        contract.target_bundle_sha256.as_str(),
        contract.recommendation_sha256.as_str(),
        contract
            .selected_algorithm_three_seed_binding_sha256
            .as_str(),
        contract.sealed_holdout_split_commitment_sha256.as_str(),
        contract.feature_order_sha256.as_str(),
        contract.preprocessing_sha256.as_str(),
    ]
    .iter()
    .all(|value| valid_sha256(value));
    let permissions_closed = !contract.callable_entrypoint_present
        && !contract.input_mount_present
        && !contract.sealed_holdout_data_adapter_present
        && !contract.sealed_holdout_features_access_allowed
        && !contract.sealed_holdout_labels_access_allowed
        && !contract.sealed_holdout_evaluation_allowed
        && !contract.training_or_preprocessing_update_allowed
        && !contract.hyperparameter_or_threshold_tuning_allowed
        && !contract.candidate_reselection_allowed
        && !contract.official_candidate_selection_allowed
        && !contract.model_store_write_allowed
        && !contract.metric_store_write_allowed
        && !contract.production_reads_allowed
        && !contract.production_writes_allowed
        && !contract.outbound_network_allowed
        && !contract.environment_inheritance_allowed
        && !contract.secrets_allowed
        && !contract.tools_allowed
        && !contract.subprocesses_allowed
        && !contract.scalar_reward_defined
        && !contract.action_position_or_ranking_semantics_defined;
    if contract.schema_version != CONTRACT_SCHEMA_VERSION
        || contract.contract_sha256 != fingerprint_without(contract, &["contract_sha256"])?
        || !hashes_valid
        || contract.implementation_protocol_version != IMPLEMENTATION_PROTOCOL_VERSION
        || contract.immutable_code_revision.trim().is_empty()
        || contract.stage_66_protocol_review_id != review.review_id
        || contract.stage_66_protocol_review_sha256 != review.review_sha256
        || contract.sealed_holdout_evaluation_protocol_sha256 != protocol.protocol_sha256
        || contract.stage_65_admission_review_sha256 != protocol.stage_65_admission_review_sha256
        || contract.output_validation_sha256 != protocol.output_validation_sha256
        || contract.candidate_set_sha256 != protocol.candidate_set_sha256
        || contract.training_store_dataset_sha256 != protocol.training_store_dataset_sha256
        || contract.target_bundle_sha256 != protocol.target_bundle_sha256
        || contract.recommendation_sha256 != protocol.recommendation_sha256
        || contract.selected_algorithm_three_seed_binding_sha256
            != protocol.selected_algorithm_three_seed_binding_sha256
        || contract.sealed_holdout_split_commitment_sha256
            != protocol.sealed_holdout_split_commitment_sha256
        || contract.feature_order_sha256 != protocol.feature_order_sha256
        || contract.preprocessing_sha256 != protocol.preprocessing_sha256
        || contract.target_id != protocol.target_id
        || contract.frozen_candidate_algorithm_id != protocol.frozen_candidate_algorithm_id
        || contract.exact_random_seeds != protocol.exact_random_seeds
        || contract.exact_feature_count != 65
        || contract.exact_target_count != 1
        || contract.exact_candidate_hypothesis_count != 3
        || contract.reported_metric_ids != protocol.reported_metric_ids
        || contract.bootstrap_unit != protocol.bootstrap_unit
        || contract.bootstrap_replications != protocol.bootstrap_replications
        || contract.bootstrap_random_seed != protocol.bootstrap_random_seed
        || contract.family_wise_error_correction != protocol.family_wise_error_correction
        || contract.family_wise_alpha_millionths != protocol.family_wise_alpha_millionths
        || contract.minimum_relative_mae_improvement_ppm
            != protocol.minimum_relative_mae_improvement_ppm
        || contract.minimum_spearman_millionths != protocol.minimum_spearman_millionths
        || contract.minimum_directional_accuracy_millionths
            != protocol.minimum_directional_accuracy_millionths
        || contract.minimum_calibration_slope_millionths
            != protocol.minimum_calibration_slope_millionths
        || contract.maximum_calibration_slope_millionths
            != protocol.maximum_calibration_slope_millionths
        || contract.minimum_sealed_holdout_rows != protocol.minimum_sealed_holdout_rows
        || contract.minimum_independent_components != protocol.minimum_independent_components
        || !contract.all_three_seeds_must_pass
        || !contract.one_shot_evaluation_required
        || !contract.future_output_create_once
        || !contract.future_output_independent_validation_required
        || !contract.no_feedback_reuse
        || !contract.insufficient_sample_fails_closed
        || !contract.no_composite_score_or_cross_target_masking
        || !contract.independent_implementation_review_required
        || !contract.isolated_runner_registration_required
        || !contract.one_shot_access_authorization_required
        || contract
            .deterministic_evaluator_function_id
            .trim()
            .is_empty()
        || contract.canonical_input_projection_schema.trim().is_empty()
        || contract.canonical_untrusted_output_schema.trim().is_empty()
        || !permissions_closed
    {
        return Err("sealed-holdout 评估实现合同无效、漂移或越权".to_string());
    }
    Ok(())
}

fn record_checks_confirmed(
    value: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
) -> bool {
    value.exact_current_stage_51_through_stage_66_binding_confirmed
        && value.registrar_independent_from_stage_66_and_complete_prior_chain_confirmed
        && value.immutable_artifact_revision_protocol_and_serialization_confirmed
        && value.one_target_one_algorithm_three_frozen_seeds_only_confirmed
        && value.no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed
        && value.one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed
        && value.fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed
        && value.no_tuning_refit_reselection_or_cross_target_composite_confirmed
        && value.future_output_create_once_untrusted_and_independent_validation_required_confirmed
        && value.independent_review_runner_and_one_shot_authorization_remain_separate_confirmed
        && value.no_selection_store_reward_shadow_order_broker_or_trading_confirmed
}

fn validate_record(
    value: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
) -> Result<(), String> {
    validate_approved_sealed_holdout_evaluation_protocol_for_implementation_registration(
        &value.upstream_protocol,
        &value.upstream_protocol_review,
    )?;
    validate_contract(
        &value.implementation_contract,
        &value.upstream_protocol,
        &value.upstream_protocol_review,
    )?;
    let expected_excluded = excluded_actor_ids(&value.upstream_protocol_review);
    let permissions_closed = !value.independent_implementation_review_completed
        && !value.isolated_runner_registration_eligible
        && !value.sealed_holdout_access_authorized
        && !value.sealed_holdout_evaluation_authorized
        && !value.official_candidate_selection_authorized
        && !value.model_artifact_store_written
        && !value.metric_store_written
        && !value.reward_authorized
        && !value.shadow_portfolio_authorized
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
        || value.implementation_name.trim().is_empty()
        || value.rationale.trim().is_empty()
        || value.known_limitations.trim().is_empty()
        || value.excluded_prior_actor_ids != expected_excluded
        || !sorted_unique(&value.excluded_prior_actor_ids)
        || value
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &value.registered_by)
        || !value.registrar_independent_from_stage_66_and_complete_prior_chain
        || value.status != STATUS
        || !record_checks_confirmed(value)
        || !value.sealed_holdout_evaluation_implementation_registered
        || !value.future_independent_implementation_review_eligible
        || !permissions_closed
    {
        return Err("sealed-holdout 评估实现登记记录无效、责任重叠或越权".to_string());
    }
    Ok(())
}

fn record_matches_source(
    record: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
    source: &HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol,
) -> bool {
    validate_record(record).is_ok()
        && record.upstream_protocol == source.protocol
        && record.upstream_protocol_review == source.protocol_review
        && record
            .implementation_contract
            .stage_66_protocol_review_sha256
            == source.protocol_review.review_sha256
        && record
            .implementation_contract
            .sealed_holdout_evaluation_protocol_sha256
            == source.protocol.protocol_sha256
}

fn ensure_protocol_unregistered(
    records: &[HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord],
    source: &HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol,
) -> Result<(), String> {
    if records.iter().any(|record| {
        record.upstream_protocol_review.review_id == source.protocol_review.review_id
            || (record.upstream_protocol.attempt_id == source.protocol.attempt_id
                && record.upstream_protocol.target_id == source.protocol.target_id)
    }) {
        return Err("同一 Stage 66 逐目标协议只能登记一次实现".to_string());
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord>, String> {
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
        let value = serde_json::from_slice::<
            HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
        >(&bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
        validate_record(&value)?;
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
    decision_root(state).join("historical-outcome-sealed-holdout-evaluation-implementations")
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
    let path = directory.join("sealed-holdout-evaluation-implementation-registration.lock");
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
        .map_err(|_| "已有 sealed-holdout 评估实现登记正在进行".to_string())?;
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
    value: &HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord,
) -> Result<String, String> {
    fingerprint_without(value, &["implementation_id", "implementation_sha256"])
}

fn fingerprint_without<T: Serialize>(value: &T, keys: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "fingerprint value must be object".to_string())?;
    for key in keys {
        object.remove(*key);
    }
    hash_serializable(&json)
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max: usize, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if trimmed.chars().count() > max {
        return Err(format!("{label}不能超过 {max} 字"));
    }
    Ok(trimmed.to_string())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
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
    use super::super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_protocol_reviews::{
        HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict,
    };

    fn protocol() -> HistoricalOutcomeSealedHoldoutEvaluationProtocol {
        let mut value = HistoricalOutcomeSealedHoldoutEvaluationProtocol {
            schema_version: "hone-historical-outcome-sealed-holdout-evaluation-protocol-v1"
                .to_string(),
            protocol_version:
                "hone-sealed-holdout-per-target-three-seed-confirmatory-evaluation-v1".to_string(),
            protocol_sha256: String::new(),
            attempt_id: "attempt-67".to_string(),
            target_id: "excess_return_250_continuous".to_string(),
            stage_65_admission_review_id: "a".repeat(32),
            stage_65_admission_review_sha256: "1".repeat(64),
            output_validation_sha256: "2".repeat(64),
            candidate_set_sha256: "3".repeat(64),
            training_store_dataset_sha256: "4".repeat(64),
            rows_sha256: "5".repeat(64),
            target_commitments_sha256: "6".repeat(64),
            validation_projection_sha256: "7".repeat(64),
            target_bundle_sha256: "8".repeat(64),
            recommendation_sha256: "9".repeat(64),
            selected_algorithm_three_seed_binding_sha256: "a".repeat(64),
            sealed_holdout_split_commitment_sha256: "b".repeat(64),
            feature_order_sha256: "c".repeat(64),
            preprocessing_sha256: "d".repeat(64),
            frozen_candidate_algorithm_id: "ridge_multi_target_regression".to_string(),
            exact_random_seeds: vec![17, 29, 43],
            exact_feature_count: 65,
            exact_target_count: 1,
            target_vector_order: vec!["excess_return_250_continuous".to_string()],
            benchmark_algorithm_id: "frozen_zero_prediction_baseline".to_string(),
            reported_metric_ids: (0..7).map(|index| format!("metric-{index}")).collect(),
            bootstrap_unit: "official_split_component_id".to_string(),
            bootstrap_replications: 10_000,
            bootstrap_random_seed: 66_202_608_24,
            family_wise_error_correction:
                "holm_bonferroni_across_exact_three_frozen_seed_tests_for_one_target".to_string(),
            family_wise_alpha_millionths: 50_000,
            exact_candidate_hypothesis_count: 3,
            minimum_relative_mae_improvement_ppm: 50_000,
            minimum_spearman_millionths: 0,
            minimum_directional_accuracy_millionths: 500_000,
            minimum_calibration_slope_millionths: 500_000,
            maximum_calibration_slope_millionths: 1_500_000,
            minimum_sealed_holdout_rows: 100,
            minimum_independent_components: 20,
            all_three_seeds_must_pass: true,
            one_shot_evaluation_required: true,
            insufficient_sample_rule: "insufficient fails closed".to_string(),
            confirmatory_decision_rule: "all seeds pass".to_string(),
            no_feedback_reuse_rule: "no feedback reuse".to_string(),
            no_composite_score_or_cross_target_masking: true,
            protocol_review_only: true,
            callable_entrypoint_present: false,
            sealed_holdout_features_access_allowed: false,
            sealed_holdout_labels_access_allowed: false,
            training_or_preprocessing_update_allowed: false,
            hyperparameter_or_threshold_tuning_allowed: false,
            candidate_reselection_allowed: false,
            model_store_write_allowed: false,
            metric_store_write_allowed: false,
            production_reads_allowed: false,
            production_writes_allowed: false,
            outbound_network_allowed: false,
            secrets_allowed: false,
            scalar_reward_defined: false,
            action_position_or_ranking_semantics_defined: false,
        };
        value.protocol_sha256 = fingerprint_without(&value, &["protocol_sha256"]).unwrap();
        value
    }

    fn review(
        protocol: &HistoricalOutcomeSealedHoldoutEvaluationProtocol,
    ) -> HistoricalOutcomeSealedHoldoutEvaluationProtocolReview {
        let mut value = HistoricalOutcomeSealedHoldoutEvaluationProtocolReview {
            schema_version: "hone-historical-outcome-sealed-holdout-evaluation-protocol-review-v1".to_string(),
            policy_version: "hone-sealed-holdout-per-target-protocol-independent-review-v1-no-data-access".to_string(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id: None,
            previous_review_sha256: None,
            attempt_id: protocol.attempt_id.clone(),
            target_id: protocol.target_id.clone(),
            stage_65_admission_review_id: protocol.stage_65_admission_review_id.clone(),
            stage_65_admission_review_sha256: protocol.stage_65_admission_review_sha256.clone(),
            output_validation_sha256: protocol.output_validation_sha256.clone(),
            candidate_set_sha256: protocol.candidate_set_sha256.clone(),
            training_store_dataset_sha256: protocol.training_store_dataset_sha256.clone(),
            target_bundle_sha256: protocol.target_bundle_sha256.clone(),
            recommendation_sha256: protocol.recommendation_sha256.clone(),
            protocol_version: protocol.protocol_version.clone(),
            protocol_sha256: protocol.protocol_sha256.clone(),
            submitted_at: Utc::now(),
            reviewer_id: "stage-66-reviewer".to_string(),
            excluded_prior_actor_ids: vec!["stage-65-reviewer".to_string()],
            verdict: HistoricalOutcomeSealedHoldoutEvaluationProtocolReviewVerdict::ApprovedForFutureSealedHoldoutEvaluationImplementationRegistration,
            rationale: "independent protocol review".to_string(),
            known_limitations: "not executed".to_string(),
            reviewer_independent_from_stage_65_and_complete_prior_chain: true,
            exact_current_stage_51_through_stage_65_binding_confirmed: true,
            reviewer_independent_from_stage_65_and_complete_prior_chain_confirmed: true,
            one_target_one_algorithm_three_frozen_seeds_only_confirmed: true,
            immutable_candidate_feature_preprocessing_and_target_confirmed: true,
            sealed_holdout_single_use_and_no_feedback_reuse_confirmed: true,
            fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: true,
            all_three_seeds_must_pass_and_failures_remain_visible_confirmed: true,
            insufficient_sample_fails_closed_confirmed: true,
            no_cross_target_composite_tuning_refit_or_reselection_confirmed: true,
            protocol_review_does_not_read_mount_project_or_execute_holdout_confirmed: true,
            next_gate_is_implementation_registration_not_data_access_confirmed: true,
            no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
            protocol_independently_approved: true,
            future_sealed_holdout_evaluation_implementation_registration_eligible: true,
            official_candidate_selection_authorized: false,
            sealed_holdout_access_authorized: false,
            sealed_holdout_evaluation_authorized: false,
            model_artifact_store_written: false,
            metric_store_written: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        value.review_sha256 = fingerprint_without(&value, &["review_id", "review_sha256"]).unwrap();
        value.review_id = value.review_sha256[..32].to_string();
        value
    }

    fn source() -> HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol {
        let protocol = protocol();
        let protocol_review = review(&protocol);
        HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol {
            protocol,
            protocol_review,
        }
    }

    fn request(
        source: &HistoricalOutcomeSealedHoldoutEvaluationImplementationEligibleProtocol,
    ) -> RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest {
        RegisterHistoricalOutcomeSealedHoldoutEvaluationImplementationRequest {
            expected_protocol_review_id: source.protocol_review.review_id.clone(),
            expected_protocol_review_sha256: source.protocol_review.review_sha256.clone(),
            expected_protocol_sha256: source.protocol.protocol_sha256.clone(),
            expected_stage_65_admission_review_sha256: source
                .protocol
                .stage_65_admission_review_sha256
                .clone(),
            expected_output_validation_sha256: source.protocol.output_validation_sha256.clone(),
            expected_candidate_set_sha256: source.protocol.candidate_set_sha256.clone(),
            expected_training_store_dataset_sha256: source
                .protocol
                .training_store_dataset_sha256
                .clone(),
            expected_target_bundle_sha256: source.protocol.target_bundle_sha256.clone(),
            expected_recommendation_sha256: source.protocol.recommendation_sha256.clone(),
            expected_selected_algorithm_three_seed_binding_sha256: source
                .protocol
                .selected_algorithm_three_seed_binding_sha256
                .clone(),
            expected_sealed_holdout_split_commitment_sha256: source
                .protocol
                .sealed_holdout_split_commitment_sha256
                .clone(),
            implementation_name: "sealed holdout evaluator spec".to_string(),
            immutable_code_revision: "git:stage67-test".to_string(),
            implementation_artifact_sha256: "e".repeat(64),
            rationale: "register zero-capability evaluator before access".to_string(),
            known_limitations: "no runner, data access or execution".to_string(),
            exact_current_stage_51_through_stage_66_binding_confirmed: true,
            registrar_independent_from_stage_66_and_complete_prior_chain_confirmed: true,
            immutable_artifact_revision_protocol_and_serialization_confirmed: true,
            one_target_one_algorithm_three_frozen_seeds_only_confirmed: true,
            no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed: true,
            one_shot_no_feedback_reuse_and_fail_closed_sample_rules_preserved_confirmed: true,
            fixed_metrics_thresholds_component_bootstrap_and_three_hypothesis_holm_confirmed: true,
            no_tuning_refit_reselection_or_cross_target_composite_confirmed: true,
            future_output_create_once_untrusted_and_independent_validation_required_confirmed: true,
            independent_review_runner_and_one_shot_authorization_remain_separate_confirmed: true,
            no_selection_store_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn record() -> HistoricalOutcomeSealedHoldoutEvaluationImplementationRecord {
        let source = source();
        build_record(&source, "stage-67-registrar", request(&source), Utc::now()).unwrap()
    }

    #[test]
    fn contract_binds_one_target_one_algorithm_three_seeds_and_protocol() {
        let source = source();
        let value = implementation_contract(&source, &"e".repeat(64), "revision").unwrap();
        assert_eq!(value.target_id, source.protocol.target_id);
        assert_eq!(
            value.frozen_candidate_algorithm_id,
            "ridge_multi_target_regression"
        );
        assert_eq!(value.exact_random_seeds, [17, 29, 43]);
        assert_eq!(value.exact_candidate_hypothesis_count, 3);
        assert_eq!(
            value.sealed_holdout_evaluation_protocol_sha256,
            source.protocol.protocol_sha256
        );
    }

    #[test]
    fn implementation_registration_has_no_entrypoint_mount_adapter_or_access() {
        let value = record();
        let contract = &value.implementation_contract;
        assert!(!contract.callable_entrypoint_present);
        assert!(!contract.input_mount_present);
        assert!(!contract.sealed_holdout_data_adapter_present);
        assert!(!contract.sealed_holdout_features_access_allowed);
        assert!(!contract.sealed_holdout_labels_access_allowed);
        assert!(!contract.sealed_holdout_evaluation_allowed);
    }

    #[test]
    fn implementation_preserves_one_shot_no_feedback_and_independent_validation() {
        let value = record();
        let contract = &value.implementation_contract;
        assert!(contract.one_shot_evaluation_required);
        assert!(contract.no_feedback_reuse);
        assert!(contract.future_output_create_once);
        assert!(contract.future_output_independent_validation_required);
        assert!(contract.insufficient_sample_fails_closed);
    }

    #[test]
    fn every_registration_confirmation_is_required() {
        let source = source();
        let mut value = request(&source);
        assert!(request_checks_confirmed(&value));
        value.no_callable_entrypoint_mount_data_adapter_or_holdout_access_confirmed = false;
        assert!(!request_checks_confirmed(&value));
    }

    #[test]
    fn exact_protocol_and_candidate_binding_rejects_drift() {
        let source = source();
        let mut value = request(&source);
        assert!(validate_expected_bindings(&source, &value).is_ok());
        value.expected_selected_algorithm_three_seed_binding_sha256 = "f".repeat(64);
        assert!(validate_expected_bindings(&source, &value).is_err());
    }

    #[test]
    fn registrar_must_be_outside_stage_66_and_complete_prior_chain() {
        let source = source();
        let failure = build_record(
            &source,
            &source.protocol_review.reviewer_id,
            request(&source),
            Utc::now(),
        );
        assert!(failure.is_err());
    }

    #[test]
    fn valid_registration_only_opens_future_independent_review() {
        let value = record();
        assert!(validate_record(&value).is_ok());
        assert!(value.future_independent_implementation_review_eligible);
        assert!(!value.isolated_runner_registration_eligible);
        assert!(!value.sealed_holdout_access_authorized);
        assert!(!value.sealed_holdout_evaluation_authorized);
        assert!(!value.official_candidate_selection_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn tamper_authority_escalation_and_duplicate_registration_fail_closed() {
        let source = source();
        let value = record();
        assert!(ensure_protocol_unregistered(&[], &source).is_ok());
        assert!(ensure_protocol_unregistered(&[value.clone()], &source).is_err());

        let mut tampered = value;
        tampered
            .implementation_contract
            .sealed_holdout_labels_access_allowed = true;
        tampered.implementation_contract.contract_sha256 =
            fingerprint_without(&tampered.implementation_contract, &["contract_sha256"]).unwrap();
        tampered.implementation_sha256 = record_fingerprint(&tampered).unwrap();
        tampered.implementation_id = tampered.implementation_sha256[..32].to_string();
        assert!(validate_record(&tampered).is_err());
    }
}
