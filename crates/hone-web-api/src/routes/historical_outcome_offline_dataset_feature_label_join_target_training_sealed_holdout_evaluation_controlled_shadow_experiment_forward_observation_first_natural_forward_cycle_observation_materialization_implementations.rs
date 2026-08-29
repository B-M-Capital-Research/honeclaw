//! Stage 107 create-once registration of a zero-capability implementation contract for one
//! independently approved Stage 105 observation-materialization specification.
//!
//! This module freezes deterministic projection semantics only. It contains no source or
//! executable artifact, callable entrypoint, runtime, input mount/read, observation writer,
//! ledger, position, performance, model/reward, order, broker or trading capability.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_specification_reviews::{
    ControlledShadowObservationMaterializationSpecificationReviewRecord,
    ControlledShadowObservationMaterializationSpecificationReviewVerdict,
    IndependentlyApprovedControlledShadowObservationMaterializationSpecification,
    independently_approved_observation_materialization_specifications_for_implementation_registration,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_observation_materialization_specifications::{
    ControlledShadowObservationMaterializationSpecification,
    ControlledShadowObservationMaterializationSpecificationRegistration,
    ZeroCapabilityObservationMaterializationAuthorityBoundary,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-implementation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-implementation-v1";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-zero-capability-observation-materialization-implementation-contract-v1";
const POLICY_VERSION: &str = "hone-controlled-shadow-observation-materialization-implementation-create-once-v1-zero-capability";
const IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-first-natural-cycle-observation-materialization-implementation-v1-not-executable";
const STAGE_105_RECORD_SCHEMA_VERSION: &str = "hone-controlled-shadow-first-natural-cycle-observation-materialization-specification-registration-v1";
const STAGE_105_SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-first-natural-cycle-observation-materialization-specification-v1";
const STAGE_105_POLICY_VERSION: &str = "hone-first-natural-cycle-observation-materialization-specification-create-once-v1-zero-capability";
const STAGE_106_REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-specification-independent-review-v1";
const STAGE_106_AUDIT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-observation-materialization-specification-independent-audit-v1";
const STAGE_106_POLICY_VERSION: &str = "hone-controlled-shadow-observation-materialization-specification-chain-external-review-v1-zero-capability";
const STATUS: &str = "registered_not_reviewed_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterControlledShadowObservationMaterializationImplementationRequest {
    expected_specification_review_id: String,
    expected_specification_review_sha256: String,
    expected_independent_audit_sha256: String,
    expected_registration_id: String,
    expected_registration_sha256: String,
    expected_specification_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_description: String,
    deterministic_projection_semantics: String,
    session_price_basis_and_gap_semantics: String,
    corporate_action_decimal_order_and_hash_semantics: String,
    initial_allocation_and_availability_semantics: String,
    error_and_missing_data_semantics: String,
    known_limitations: String,
    future_review_constraints: String,
    exact_stage_51_through_stage_106_binding_confirmed: bool,
    registrar_independent_from_stage_106_and_complete_prior_chain_confirmed: bool,
    independent_recomputation_of_review_registration_specification_and_audit_confirmed: bool,
    zero_capability_contract_only_no_source_or_executable_artifact_confirmed: bool,
    exact_stage_104_admitted_output_is_only_future_input_confirmed: bool,
    official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: bool,
    subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: bool,
    dividends_splits_and_price_bases_remain_separate_confirmed: bool,
    decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: bool,
    initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed:
        bool,
    conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: bool,
    one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
        bool,
    future_output_untrusted_and_independent_validation_required_confirmed: bool,
    no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    future_independent_implementation_review_required_before_isolated_runner_registration_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityObservationMaterializationImplementationAuthorityBoundary {
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub input_mount_present: bool,
    pub input_read_allowed: bool,
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
pub(crate) struct ControlledShadowObservationMaterializationImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_protocol_version: String,
    pub immutable_code_revision: String,
    pub stage_106_specification_review_id: String,
    pub stage_106_specification_review_sha256: String,
    pub stage_106_independent_audit_sha256: String,
    pub stage_105_registration_id: String,
    pub stage_105_registration_sha256: String,
    pub observation_materialization_specification_sha256: String,
    pub exact_observation_materialization_specification:
        ControlledShadowObservationMaterializationSpecification,
    pub current_source_binding_validation_function_id: String,
    pub canonical_session_projection_function_id: String,
    pub three_price_basis_projection_function_id: String,
    pub explicit_gap_and_spy_fail_closed_function_id: String,
    pub corporate_action_separation_function_id: String,
    pub initial_allocation_binding_function_id: String,
    pub conservative_availability_function_id: String,
    pub canonical_envelope_serialization_and_hash_function_id: String,
    pub canonical_session_schema: String,
    pub canonical_price_observation_schema: String,
    pub canonical_explicit_gap_schema: String,
    pub canonical_corporate_action_schema: String,
    pub canonical_observation_envelope_schema: String,
    pub future_exact_admitted_input_read_only_and_content_addressed: bool,
    pub future_output_relative_path_template: String,
    pub future_observation_output_create_once_and_untrusted: bool,
    pub future_observation_output_independent_validation_required: bool,
    pub future_observation_output_contains_valuation_or_investment_semantics: bool,
    pub future_observation_output_contains_order_intent: bool,
    pub registered_not_run: bool,
    pub independent_implementation_review_required: bool,
    pub isolated_runner_registration_required_after_review: bool,
    pub authority_boundary: ZeroCapabilityObservationMaterializationImplementationAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationMaterializationImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub upstream_specification_registration:
        ControlledShadowObservationMaterializationSpecificationRegistration,
    pub upstream_specification_review:
        ControlledShadowObservationMaterializationSpecificationReviewRecord,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_stage_106_and_complete_prior_chain: bool,
    pub implementation_name: String,
    pub implementation_description: String,
    pub deterministic_projection_semantics: String,
    pub session_price_basis_and_gap_semantics: String,
    pub corporate_action_decimal_order_and_hash_semantics: String,
    pub initial_allocation_and_availability_semantics: String,
    pub error_and_missing_data_semantics: String,
    pub known_limitations: String,
    pub future_review_constraints: String,
    pub implementation_contract: ControlledShadowObservationMaterializationImplementationContract,
    pub status: String,
    pub confirmations_complete: bool,
    pub exact_stage_51_through_stage_106_binding_confirmed: bool,
    pub registrar_independent_from_stage_106_and_complete_prior_chain_confirmed: bool,
    pub independent_recomputation_of_review_registration_specification_and_audit_confirmed: bool,
    pub zero_capability_contract_only_no_source_or_executable_artifact_confirmed: bool,
    pub exact_stage_104_admitted_output_is_only_future_input_confirmed: bool,
    pub official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: bool,
    pub subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: bool,
    pub dividends_splits_and_price_bases_remain_separate_confirmed: bool,
    pub decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: bool,
    pub initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed:
        bool,
    pub conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed:
        bool,
    pub one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
        bool,
    pub future_output_untrusted_and_independent_validation_required_confirmed: bool,
    pub no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
        bool,
    pub no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub future_independent_implementation_review_required_before_isolated_runner_registration_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_implementation_contract_registered: bool,
    pub observation_materialization_implementation_present: bool,
    pub future_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub isolated_runner_registration_eligible: bool,
    pub observation_materialized: bool,
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
pub(crate) struct ControlledShadowObservationMaterializationImplementationItem {
    pub specification_review: ControlledShadowObservationMaterializationSpecificationReviewRecord,
    pub specification_registration:
        ControlledShadowObservationMaterializationSpecificationRegistration,
    pub implementation: Option<ControlledShadowObservationMaterializationImplementationRecord>,
    pub registration_eligible: bool,
    pub upstream_binding_current: bool,
    pub future_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationMaterializationImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub registration_endpoint_available: bool,
    pub items: Vec<ControlledShadowObservationMaterializationImplementationItem>,
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
    pub input_mounted_or_read: bool,
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
pub(crate) struct ControlledShadowObservationMaterializationImplementationReadinessSummary {
    pub independently_approved_specification_count: usize,
    pub registration_eligible_count: usize,
    pub implementation_contract_count: usize,
    pub current_binding_implementation_contract_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_controlled_shadow_observation_materialization_implementations(
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
            warn!(%error, "observation materialization implementation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "观察物化零能力实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_controlled_shadow_observation_materialization_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(specification_review_id): AxumPath<String>,
    Json(request): Json<RegisterControlledShadowObservationMaterializationImplementationRequest>,
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

pub(crate) async fn controlled_shadow_observation_materialization_implementation_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationMaterializationImplementationReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowObservationMaterializationImplementationReadinessSummary {
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
pub(crate) async fn independently_reviewable_observation_materialization_implementations(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationMaterializationImplementationRecord>, String> {
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
) -> Result<ControlledShadowObservationMaterializationImplementationRegistry, String> {
    let sources =
        independently_approved_observation_materialization_specifications_for_implementation_registration(
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
        items.push(
            ControlledShadowObservationMaterializationImplementationItem {
                specification_registration: source.review.registration.clone(),
                specification_review: source.review,
                implementation,
                registration_eligible,
                upstream_binding_current,
                future_independent_implementation_review_eligible: upstream_binding_current,
            },
        );
    }
    items.sort_by(|left, right| {
        right
            .specification_review
            .submitted_at
            .cmp(&left.specification_review.submitted_at)
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
        "zero_capability_observation_materialization_implementation_contract_registered_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "independently_approved_observation_materialization_specification_ready_for_zero_capability_implementation_registration"
    } else {
        "waiting_stage_106_independently_approved_observation_materialization_specification"
    };
    Ok(ControlledShadowObservationMaterializationImplementationRegistry {
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
        input_mounted_or_read: false,
        observation_materialized: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 107 只把 Stage 106 独立批准的精确观察物化规格登记为内容寻址、零能力且不可执行的实现合同，冻结确定性投影函数、三价格口径、显式缺口、公司行动、初始组合绑定、可用时间和输出哈希语义。没有源码/可执行工件、入口、runtime、输入挂载或读取、观察、账本、持仓、绩效、模型/指标、训练、reward、订单、券商或交易能力；下一步只能进入 Stage 108 责任链外实现复核。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    specification_review_id: &str,
    request: RegisterControlledShadowObservationMaterializationImplementationRequest,
) -> Result<ControlledShadowObservationMaterializationImplementationRecord, String> {
    if !valid_id(specification_review_id) || registrar_id.trim().is_empty() {
        return Err("Stage 107 规格复核 ID 或登记者无效".to_string());
    }
    let _lock = acquire_lock(state, specification_review_id).await?;
    let source = independently_approved_observation_materialization_specifications_for_implementation_registration(state)
        .await?
        .into_iter()
        .find(|source| source.review.review_id == specification_review_id)
        .ok_or_else(|| "当前没有精确匹配且独立批准的 Stage 106 观察物化规格".to_string())?;
    if read_records(state)
        .await?
        .iter()
        .any(|record| record.upstream_specification_review.review_id == specification_review_id)
    {
        return Err("该 Stage 106 规格复核已登记实现合同，禁止覆盖或重放".to_string());
    }
    let record = build_record(&source, registrar_id, request, Utc::now())?;
    write_immutable_json(&record_path(state, specification_review_id), &record).await?;
    Ok(record)
}

fn build_record(
    source: &IndependentlyApprovedControlledShadowObservationMaterializationSpecification,
    registrar_id: &str,
    request: RegisterControlledShadowObservationMaterializationImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<ControlledShadowObservationMaterializationImplementationRecord, String> {
    validate_source(source)?;
    validate_expected_binding(source, &request)?;
    if !all_confirmations(&request) {
        return Err("Stage 107 登记前必须逐项确认完整绑定、投影语义、零能力和职责隔离".to_string());
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
    let deterministic_projection_semantics = bounded_required(
        &request.deterministic_projection_semantics,
        MAX_TEXT_CHARS,
        "确定性投影语义",
    )?;
    let session_price_basis_and_gap_semantics = bounded_required(
        &request.session_price_basis_and_gap_semantics,
        MAX_TEXT_CHARS,
        "交易日、价格口径与缺口语义",
    )?;
    let corporate_action_decimal_order_and_hash_semantics = bounded_required(
        &request.corporate_action_decimal_order_and_hash_semantics,
        MAX_TEXT_CHARS,
        "公司行动、十进制、排序与哈希语义",
    )?;
    let initial_allocation_and_availability_semantics = bounded_required(
        &request.initial_allocation_and_availability_semantics,
        MAX_TEXT_CHARS,
        "初始组合与可用时间语义",
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
        || !request.registrar_independent_from_stage_106_and_complete_prior_chain_confirmed
    {
        return Err("Stage 107 登记者必须独立于 Stage 106 reviewer 和完整上游责任链".to_string());
    }
    let implementation_contract = implementation_contract(source, &immutable_code_revision)?;
    let registration = source.review.registration.clone();
    let review = source.review.clone();
    let mut record = ControlledShadowObservationMaterializationImplementationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        upstream_specification_registration: registration,
        upstream_specification_review: review,
        excluded_prior_actor_ids,
        registrar_independent_from_stage_106_and_complete_prior_chain: independent,
        implementation_name,
        implementation_description,
        deterministic_projection_semantics,
        session_price_basis_and_gap_semantics,
        corporate_action_decimal_order_and_hash_semantics,
        initial_allocation_and_availability_semantics,
        error_and_missing_data_semantics,
        known_limitations,
        future_review_constraints,
        implementation_contract,
        status: STATUS.to_string(),
        confirmations_complete: all_confirmations(&request),
        exact_stage_51_through_stage_106_binding_confirmed: request
            .exact_stage_51_through_stage_106_binding_confirmed,
        registrar_independent_from_stage_106_and_complete_prior_chain_confirmed: request
            .registrar_independent_from_stage_106_and_complete_prior_chain_confirmed,
        independent_recomputation_of_review_registration_specification_and_audit_confirmed: request
            .independent_recomputation_of_review_registration_specification_and_audit_confirmed,
        zero_capability_contract_only_no_source_or_executable_artifact_confirmed: request
            .zero_capability_contract_only_no_source_or_executable_artifact_confirmed,
        exact_stage_104_admitted_output_is_only_future_input_confirmed: request
            .exact_stage_104_admitted_output_is_only_future_input_confirmed,
        official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: request
            .official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed,
        subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: request
            .subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed,
        dividends_splits_and_price_bases_remain_separate_confirmed: request
            .dividends_splits_and_price_bases_remain_separate_confirmed,
        decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: request
            .decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed,
        initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed:
            request.initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed,
        conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed:
            request.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed,
        one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed:
            request.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed,
        future_output_untrusted_and_independent_validation_required_confirmed: request
            .future_output_untrusted_and_independent_validation_required_confirmed,
        no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed:
            request.no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
            request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        future_independent_implementation_review_required_before_isolated_runner_registration_confirmed:
            request.future_independent_implementation_review_required_before_isolated_runner_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_implementation_contract_registered: true,
        observation_materialization_implementation_present: false,
        future_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        isolated_runner_registration_eligible: false,
        observation_materialized: false,
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
    source: &IndependentlyApprovedControlledShadowObservationMaterializationSpecification,
    immutable_code_revision: &str,
) -> Result<ControlledShadowObservationMaterializationImplementationContract, String> {
    let review = &source.review;
    let registration = &review.registration;
    let specification = &registration.specification;
    let function_ids = deterministic_function_ids();
    let mut contract = ControlledShadowObservationMaterializationImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_protocol_version: IMPLEMENTATION_PROTOCOL_VERSION.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        stage_106_specification_review_id: review.review_id.clone(),
        stage_106_specification_review_sha256: review.review_sha256.clone(),
        stage_106_independent_audit_sha256: review.independent_audit.audit_sha256.clone(),
        stage_105_registration_id: registration.registration_id.clone(),
        stage_105_registration_sha256: registration.registration_sha256.clone(),
        observation_materialization_specification_sha256: specification
            .specification_sha256
            .clone(),
        exact_observation_materialization_specification: specification.clone(),
        current_source_binding_validation_function_id: function_ids[0].to_string(),
        canonical_session_projection_function_id: function_ids[1].to_string(),
        three_price_basis_projection_function_id: function_ids[2].to_string(),
        explicit_gap_and_spy_fail_closed_function_id: function_ids[3].to_string(),
        corporate_action_separation_function_id: function_ids[4].to_string(),
        initial_allocation_binding_function_id: function_ids[5].to_string(),
        conservative_availability_function_id: function_ids[6].to_string(),
        canonical_envelope_serialization_and_hash_function_id: function_ids[7].to_string(),
        canonical_session_schema: specification.canonical_session_schema.clone(),
        canonical_price_observation_schema: specification
            .canonical_price_observation_schema
            .clone(),
        canonical_explicit_gap_schema: specification.canonical_explicit_gap_schema.clone(),
        canonical_corporate_action_schema: specification.canonical_corporate_action_schema.clone(),
        canonical_observation_envelope_schema: specification
            .canonical_observation_envelope_schema
            .clone(),
        future_exact_admitted_input_read_only_and_content_addressed: true,
        future_output_relative_path_template: specification
            .future_output_relative_path_template
            .clone(),
        future_observation_output_create_once_and_untrusted: true,
        future_observation_output_independent_validation_required: true,
        future_observation_output_contains_valuation_or_investment_semantics: false,
        future_observation_output_contains_order_intent: false,
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
        "observation_materialization_current_source_binding_validation_v1_pure_specification",
        "observation_materialization_canonical_session_projection_v1_pure_specification",
        "observation_materialization_three_price_basis_projection_v1_pure_specification",
        "observation_materialization_explicit_gap_spy_fail_closed_v1_pure_specification",
        "observation_materialization_corporate_action_separation_v1_pure_specification",
        "observation_materialization_initial_allocation_binding_v1_pure_specification",
        "observation_materialization_conservative_availability_v1_pure_specification",
        "observation_materialization_canonical_envelope_serialization_hash_v1_pure_specification",
    ]
}

fn closed_authority_boundary()
-> ZeroCapabilityObservationMaterializationImplementationAuthorityBoundary {
    ZeroCapabilityObservationMaterializationImplementationAuthorityBoundary {
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        input_mount_present: false,
        input_read_allowed: false,
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

fn authority_closed(
    value: &ZeroCapabilityObservationMaterializationImplementationAuthorityBoundary,
) -> bool {
    value == &closed_authority_boundary()
}

fn validate_source(
    source: &IndependentlyApprovedControlledShadowObservationMaterializationSpecification,
) -> Result<(), String> {
    validate_source_parts(&source.review.registration, &source.review)
}

fn validate_source_parts(
    registration: &ControlledShadowObservationMaterializationSpecificationRegistration,
    review: &ControlledShadowObservationMaterializationSpecificationReviewRecord,
) -> Result<(), String> {
    let registration_sha =
        fingerprint_without(registration, &["registration_id", "registration_sha256"])?;
    let specification_sha =
        fingerprint_without(&registration.specification, &["specification_sha256"])?;
    let review_sha = fingerprint_without(review, &["review_id", "review_sha256"])?;
    let audit_sha = fingerprint_without(&review.independent_audit, &["audit_sha256"])?;
    let audit = &review.independent_audit;
    let valid = registration.schema_version == STAGE_105_RECORD_SCHEMA_VERSION
        && registration.policy_version == STAGE_105_POLICY_VERSION
        && registration.registration_sha256 == registration_sha
        && registration.registration_id == registration_sha[..32]
        && registration.specification.schema_version == STAGE_105_SPEC_SCHEMA_VERSION
        && registration.specification.specification_sha256 == specification_sha
        && registration.specification_registered
        && registration.future_chain_external_specification_review_eligible
        && !registration.specification_review_completed
        && !registration.implementation_registration_eligible
        && stage_105_specification_closed(&registration.specification)
        && review.schema_version == STAGE_106_REVIEW_SCHEMA_VERSION
        && review.policy_version == STAGE_106_POLICY_VERSION
        && review.review_sha256 == review_sha
        && review.review_id == review_sha[..32]
        && review.registration == *registration
        && review.verdict
            == ControlledShadowObservationMaterializationSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityObservationMaterializationImplementationRegistration
        && review.specification_independently_approved
        && review.future_zero_capability_implementation_registration_eligible
        && !review.implementation_registered
        && !review.executable_artifact_present
        && !review.callable_entrypoint_present
        && !review.runtime_present
        && !review.input_mount_present
        && !review.observation_materialized
        && !review.ledger_created
        && !review.position_written
        && !review.performance_metric_written
        && !review.model_or_metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized
        && audit.schema_version == STAGE_106_AUDIT_SCHEMA_VERSION
        && audit.audit_sha256 == audit_sha
        && audit.registration_id == registration.registration_id
        && audit.registration_sha256 == registration.registration_sha256
        && audit.specification_sha256 == registration.specification.specification_sha256
        && audit.registration_hash_independently_reproduced
        && audit.specification_hash_independently_reproduced
        && audit.exact_current_stage_51_through_stage_105_binding_valid
        && audit.complete_specification_rebuilt_without_stage_105_builder
        && audit.rebuilt_specification_exactly_matches_registration
        && audit.session_subject_spy_three_price_basis_and_gap_contract_valid
        && audit.corporate_action_decimal_order_hash_and_output_path_contract_valid
        && audit.initial_shadow_allocation_and_availability_contract_valid
        && audit.all_implementation_runtime_observation_store_feedback_order_broker_and_trading_authority_closed
        && audit.mismatch_reasons.is_empty()
        && sorted_unique(&registration.excluded_prior_actor_ids)
        && sorted_unique(&review.excluded_prior_actor_ids)
        && !registration
            .excluded_prior_actor_ids
            .contains(&registration.registered_by)
        && !review.excluded_prior_actor_ids.contains(&review.reviewer_id)
        && review
            .excluded_prior_actor_ids
            .contains(&registration.registered_by);
    valid
        .then_some(())
        .ok_or_else(|| "Stage 105–106 观察物化规格来源无效、漂移或越权".to_string())
}

fn stage_105_specification_closed(
    value: &ControlledShadowObservationMaterializationSpecification,
) -> bool {
    value.exact_admitted_output_is_only_input
        && value.allowed_price_bases
            == [
                "raw_unadjusted_price",
                "split_adjusted_price",
                "dividend_adjusted_price",
            ]
        && value.official_market_session_count > 0
        && !value.provider_publication_time_verified
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
        && stage_105_authority_closed(&value.authority_boundary)
}

fn stage_105_authority_closed(
    value: &ZeroCapabilityObservationMaterializationAuthorityBoundary,
) -> bool {
    !value.implementation_present
        && !value.executable_artifact_present
        && !value.callable_entrypoint_present
        && !value.runtime_present
        && !value.input_mount_present
        && !value.environment_inheritance_allowed
        && !value.secrets_allowed
        && !value.outbound_network_allowed
        && !value.tools_allowed
        && !value.subprocesses_allowed
        && !value.production_reads_allowed
        && !value.production_writes_allowed
        && !value.observation_envelope_created
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

fn validate_expected_binding(
    source: &IndependentlyApprovedControlledShadowObservationMaterializationSpecification,
    request: &RegisterControlledShadowObservationMaterializationImplementationRequest,
) -> Result<(), String> {
    let review = &source.review;
    let registration = &review.registration;
    let exact = request.expected_specification_review_id == review.review_id
        && request.expected_specification_review_sha256 == review.review_sha256
        && request.expected_independent_audit_sha256 == review.independent_audit.audit_sha256
        && request.expected_registration_id == registration.registration_id
        && request.expected_registration_sha256 == registration.registration_sha256
        && request.expected_specification_sha256 == registration.specification.specification_sha256;
    exact
        .then_some(())
        .ok_or_else(|| "Stage 51–106 精确绑定已经变化，请刷新后重试".to_string())
}

fn all_confirmations(
    request: &RegisterControlledShadowObservationMaterializationImplementationRequest,
) -> bool {
    request.exact_stage_51_through_stage_106_binding_confirmed
        && request.registrar_independent_from_stage_106_and_complete_prior_chain_confirmed
        && request.independent_recomputation_of_review_registration_specification_and_audit_confirmed
        && request.zero_capability_contract_only_no_source_or_executable_artifact_confirmed
        && request.exact_stage_104_admitted_output_is_only_future_input_confirmed
        && request.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && request.subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed
        && request.dividends_splits_and_price_bases_remain_separate_confirmed
        && request.decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed
        && request.initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed
        && request.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed
        && request.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed
        && request.future_output_untrusted_and_independent_validation_required_confirmed
        && request.no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.future_independent_implementation_review_required_before_isolated_runner_registration_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_contract(
    contract: &ControlledShadowObservationMaterializationImplementationContract,
    registration: &ControlledShadowObservationMaterializationSpecificationRegistration,
    review: &ControlledShadowObservationMaterializationSpecificationReviewRecord,
) -> Result<(), String> {
    let specification = &contract.exact_observation_materialization_specification;
    let expected_function_ids = deterministic_function_ids();
    let actual_function_ids = [
        contract
            .current_source_binding_validation_function_id
            .as_str(),
        contract.canonical_session_projection_function_id.as_str(),
        contract.three_price_basis_projection_function_id.as_str(),
        contract
            .explicit_gap_and_spy_fail_closed_function_id
            .as_str(),
        contract.corporate_action_separation_function_id.as_str(),
        contract.initial_allocation_binding_function_id.as_str(),
        contract.conservative_availability_function_id.as_str(),
        contract
            .canonical_envelope_serialization_and_hash_function_id
            .as_str(),
    ];
    let valid = contract.schema_version == CONTRACT_SCHEMA_VERSION
        && contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?
        && contract.implementation_protocol_version == IMPLEMENTATION_PROTOCOL_VERSION
        && !contract.immutable_code_revision.trim().is_empty()
        && contract.stage_106_specification_review_id == review.review_id
        && contract.stage_106_specification_review_sha256 == review.review_sha256
        && contract.stage_106_independent_audit_sha256 == review.independent_audit.audit_sha256
        && contract.stage_105_registration_id == registration.registration_id
        && contract.stage_105_registration_sha256 == registration.registration_sha256
        && contract.observation_materialization_specification_sha256
            == registration.specification.specification_sha256
        && specification == &registration.specification
        && stage_105_specification_closed(specification)
        && actual_function_ids == expected_function_ids
        && contract.canonical_session_schema == specification.canonical_session_schema
        && contract.canonical_price_observation_schema
            == specification.canonical_price_observation_schema
        && contract.canonical_explicit_gap_schema == specification.canonical_explicit_gap_schema
        && contract.canonical_corporate_action_schema
            == specification.canonical_corporate_action_schema
        && contract.canonical_observation_envelope_schema
            == specification.canonical_observation_envelope_schema
        && contract.future_exact_admitted_input_read_only_and_content_addressed
        && contract.future_output_relative_path_template
            == specification.future_output_relative_path_template
        && contract.future_observation_output_create_once_and_untrusted
        && contract.future_observation_output_independent_validation_required
        && !contract.future_observation_output_contains_valuation_or_investment_semantics
        && !contract.future_observation_output_contains_order_intent
        && contract.registered_not_run
        && contract.independent_implementation_review_required
        && contract.isolated_runner_registration_required_after_review
        && authority_closed(&contract.authority_boundary);
    valid
        .then_some(())
        .ok_or_else(|| "Stage 107 零能力观察物化实现合同无效、漂移或越权".to_string())
}

pub(crate) fn validate_observation_materialization_implementation_for_review(
    record: &ControlledShadowObservationMaterializationImplementationRecord,
) -> Result<(), String> {
    let mut required = record
        .upstream_specification_review
        .excluded_prior_actor_ids
        .clone();
    required.push(record.upstream_specification_review.reviewer_id.clone());
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
        &record.deterministic_projection_semantics,
        &record.session_price_basis_and_gap_semantics,
        &record.corporate_action_decimal_order_and_hash_semantics,
        &record.initial_allocation_and_availability_semantics,
        &record.error_and_missing_data_semantics,
        &record.known_limitations,
        &record.future_review_constraints,
    ]
    .iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let no_downstream = !record.observation_materialization_implementation_present
        && !record.independent_implementation_review_completed
        && !record.isolated_runner_registration_eligible
        && !record.observation_materialized
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
        && record.registrar_independent_from_stage_106_and_complete_prior_chain
        && record.confirmations_complete
        && record.exact_stage_51_through_stage_106_binding_confirmed
        && record.registrar_independent_from_stage_106_and_complete_prior_chain_confirmed
        && record.independent_recomputation_of_review_registration_specification_and_audit_confirmed
        && record.zero_capability_contract_only_no_source_or_executable_artifact_confirmed
        && record.exact_stage_104_admitted_output_is_only_future_input_confirmed
        && record.official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed
        && record.subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed
        && record.dividends_splits_and_price_bases_remain_separate_confirmed
        && record.decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed
        && record.initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed
        && record.conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed
        && record.one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed
        && record.future_output_untrusted_and_independent_validation_required_confirmed
        && record.no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && record.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && record.future_independent_implementation_review_required_before_isolated_runner_registration_confirmed
        && record.no_unconfirmed_hari_or_old_wang_logic_claimed
        && record.zero_capability_implementation_contract_registered
        && record.future_independent_implementation_review_eligible
        && no_downstream;
    if !valid {
        return Err("Stage 107 零能力观察物化实现登记无效、漂移或越权".to_string());
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
    record: &ControlledShadowObservationMaterializationImplementationRecord,
) -> Result<(), String> {
    validate_observation_materialization_implementation_for_review(record)
}

fn record_matches_source(
    record: &ControlledShadowObservationMaterializationImplementationRecord,
    source: &IndependentlyApprovedControlledShadowObservationMaterializationSpecification,
) -> bool {
    validate_record(record).is_ok()
        && record.upstream_specification_review.review_id == source.review.review_id
        && record.upstream_specification_review.review_sha256 == source.review.review_sha256
        && record.upstream_specification_registration.registration_id
            == source.review.registration.registration_id
        && record
            .upstream_specification_registration
            .registration_sha256
            == source.review.registration.registration_sha256
}

fn excluded_actor_ids(
    source: &IndependentlyApprovedControlledShadowObservationMaterializationSpecification,
) -> Vec<String> {
    let mut actors = source.review.excluded_prior_actor_ids.clone();
    actors.push(source.review.reviewer_id.clone());
    actors.push(source.review.registration.registered_by.clone());
    actors.extend(source.review.registration.excluded_prior_actor_ids.clone());
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn record_fingerprint(
    value: &ControlledShadowObservationMaterializationImplementationRecord,
) -> Result<String, String> {
    fingerprint_without(value, &["implementation_id", "implementation_sha256"])
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationMaterializationImplementationRecord>, String> {
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
                let record: ControlledShadowObservationMaterializationImplementationRecord =
                    serde_json::from_slice(
                        &tokio::fs::read(&path)
                            .await
                            .map_err(|error| error.to_string())?,
                    )
                    .map_err(|error| error.to_string())?;
                validate_record(&record)?;
                if path.file_stem().and_then(|value| value.to_str()) != Some("implementation") {
                    return Err("Stage 107 实现登记文件名无效".to_string());
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
            return Err("Stage 107 实现登记重复".to_string());
        }
    }
    records.sort_by(|left, right| right.registered_at.cmp(&left.registered_at));
    Ok(records)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 107 指纹载荷无效".to_string())?;
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
        .join("controlled-shadow-first-natural-cycle-observation-materialization-implementations")
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
        "controlled-shadow-observation-materialization-implementation-{specification_review_id}.lock"
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
        .map_err(|_| "已有同一 Stage 107 实现登记正在进行".to_string())?;
    Ok(RegistrationLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RegisterControlledShadowObservationMaterializationImplementationRequest {
        RegisterControlledShadowObservationMaterializationImplementationRequest {
            expected_specification_review_id: "1".repeat(32),
            expected_specification_review_sha256: "1".repeat(64),
            expected_independent_audit_sha256: "2".repeat(64),
            expected_registration_id: "3".repeat(32),
            expected_registration_sha256: "3".repeat(64),
            expected_specification_sha256: "4".repeat(64),
            implementation_name: "首次自然前向周期观察物化零能力实现合同".to_string(),
            immutable_code_revision: "revision-1".to_string(),
            implementation_description: "只冻结投影实现语义，不提供代码、工件或入口。".to_string(),
            deterministic_projection_semantics: "相同准入输入与规格只能产生相同信封或相同失败。".to_string(),
            session_price_basis_and_gap_semantics: "官方交易日、三价格口径和显式缺口分别投影。".to_string(),
            corporate_action_decimal_order_and_hash_semantics: "分红拆股分离并保留精确十进制、排序和逐行摘要。".to_string(),
            initial_allocation_and_availability_semantics: "初始组合只绑定摘要，可用时间保持保守。".to_string(),
            error_and_missing_data_semantics: "SPY 缺口、重复、越界、填充或替代均失败关闭。".to_string(),
            known_limitations: "尚无真实规格复核、实现工件或自然前向观察。".to_string(),
            future_review_constraints: "Stage 108 独立实现复核通过后仍只能登记隔离 runner。".to_string(),
            exact_stage_51_through_stage_106_binding_confirmed: true,
            registrar_independent_from_stage_106_and_complete_prior_chain_confirmed: true,
            independent_recomputation_of_review_registration_specification_and_audit_confirmed: true,
            zero_capability_contract_only_no_source_or_executable_artifact_confirmed: true,
            exact_stage_104_admitted_output_is_only_future_input_confirmed: true,
            official_session_subject_spy_and_three_price_basis_matrix_preserved_confirmed: true,
            subject_gap_explicit_and_spy_gap_duplicate_out_of_window_fail_closed_confirmed: true,
            dividends_splits_and_price_bases_remain_separate_confirmed: true,
            decimal_order_row_hash_and_content_addressed_output_rules_preserved_confirmed: true,
            initial_shadow_allocation_binding_preserved_without_recomputation_or_accounting_transition_confirmed: true,
            conservative_available_at_and_unverified_provider_publication_time_preserved_confirmed: true,
            one_envelope_create_once_no_overwrite_backfill_fill_interpolation_or_substitution_confirmed: true,
            future_output_untrusted_and_independent_validation_required_confirmed: true,
            no_entrypoint_runtime_input_mount_read_environment_secret_network_tool_subprocess_or_production_io_confirmed: true,
            no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            future_independent_implementation_review_required_before_isolated_runner_registration_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn registration_requires_every_stage_107_confirmation() {
        let mut value = request();
        assert!(all_confirmations(&value));
        value.exact_stage_104_admitted_output_is_only_future_input_confirmed = false;
        assert!(!all_confirmations(&value));
    }

    #[test]
    fn implementation_authority_is_fully_closed() {
        let value = closed_authority_boundary();
        assert!(authority_closed(&value));
        assert!(!value.source_artifact_present);
        assert!(!value.input_read_allowed);
        assert!(!value.trading_allowed);
    }

    #[test]
    fn deterministic_function_contract_covers_materialization_boundaries() {
        let values = deterministic_function_ids();
        assert_eq!(values.len(), 8);
        assert!(
            values
                .iter()
                .all(|value| value.ends_with("_pure_specification"))
        );
        assert!(
            values
                .iter()
                .any(|value| value.contains("three_price_basis"))
        );
        assert!(
            values
                .iter()
                .any(|value| value.contains("initial_allocation"))
        );
        assert!(values.iter().any(|value| value.contains("availability")));
    }

    #[test]
    fn fingerprint_binds_implementation_semantics() {
        #[derive(Serialize)]
        struct Fixture {
            implementation_id: String,
            implementation_sha256: String,
            semantics: String,
        }
        let strict = fingerprint_without(
            &Fixture {
                implementation_id: String::new(),
                implementation_sha256: String::new(),
                semantics: "explicit_gap".to_string(),
            },
            &["implementation_id", "implementation_sha256"],
        )
        .unwrap();
        let fallback = fingerprint_without(
            &Fixture {
                implementation_id: String::new(),
                implementation_sha256: String::new(),
                semantics: "forward_fill".to_string(),
            },
            &["implementation_id", "implementation_sha256"],
        )
        .unwrap();
        assert_ne!(strict, fallback);
    }
}
