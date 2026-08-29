//! Immutable registration of a future isolated feature-label join and continuous-target
//! implementation.
//!
//! Registration freezes only a declarative implementation contract after the Stage 37
//! independent review. It exposes no callable entrypoint and cannot read labels, execute a join,
//! create joined rows, copy training data, train, reward, shadow, draft orders, access a broker or
//! trade. A separate independent implementation review is the only next gate.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_spec_reviews::{
    HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
    approved_historical_outcome_feature_label_join_target_spec_reviews,
    validate_historical_outcome_feature_label_join_target_spec_review_for_implementation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-implementation-registry-v1";
const IMPLEMENTATION_SCHEMA_VERSION: &str =
    "hone-historical-outcome-feature-label-join-target-implementation-v1";
const POLICY_VERSION: &str =
    "hone-feature-label-join-target-implementation-registration-v1-no-execution";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-feature-label-join-target-isolated-implementation-contract-v1";
const JOIN_IMPLEMENTATION_ID: &str = "hone-exact-entry-feature-outcome-join-enumerator";
const JOIN_IMPLEMENTATION_VERSION: &str = "v1-one-to-one-fail-closed";
const TARGET_IMPLEMENTATION_ID: &str = "hone-raw-outcome-continuous-target-projector";
const TARGET_IMPLEMENTATION_VERSION: &str = "v1-exact-f64-bits-nine-targets";
const CANONICAL_SERIALIZER_VERSION: &str = "hone-canonical-json-serializer-v1";
const INPUT_SCHEMA_VERSION: &str = "hone-reviewed-feature-label-join-target-input-v1";
const OUTPUT_SCHEMA_VERSION: &str = "hone-unmaterialized-joined-target-envelope-v1";
const STATUS: &str = "registered_not_reviewed_not_run";
const EXPECTED_FEATURE_COUNT: u16 = 65;
const EXPECTED_TARGET_COUNT: u8 = 9;
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest {
    expected_review_id: String,
    expected_review_sha256: String,
    expected_review_contract_sha256: String,
    expected_independent_audit_sha256: String,
    expected_specification_id: String,
    expected_specification_sha256: String,
    expected_specification_body_sha256: String,
    expected_join_specification_sha256: String,
    expected_target_specification_sha256: String,
    expected_combined_artifact_sha256: String,
    expected_dataset_id: String,
    expected_dataset_content_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    exact_approved_review_specification_and_artifact_binding_confirmed: bool,
    registrar_independence_confirmed: bool,
    implementation_artifact_and_code_revision_immutable_confirmed: bool,
    exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed: bool,
    point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: bool,
    exact_nine_raw_f64_target_projection_without_transform_confirmed: bool,
    sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed: bool,
    canonical_serialization_and_fixed_input_output_schema_confirmed: bool,
    no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: bool,
    registration_review_runner_execution_and_output_validation_separation_confirmed: bool,
    no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_artifact_sha256: String,
    pub immutable_code_revision: String,
    pub join_implementation_id: String,
    pub join_implementation_version: String,
    pub target_implementation_id: String,
    pub target_implementation_version: String,
    pub canonical_serializer_version: String,
    pub input_schema_version: String,
    pub output_schema_version: String,
    pub input_contract: String,
    pub output_contract: String,
    pub exact_feature_count: u16,
    pub exact_target_count: u8,
    pub exact_horizons_market_sessions: Vec<u16>,
    pub maximum_parallel_datasets: u8,
    pub maximum_memory_mebibytes: u32,
    pub callable_entrypoint_present: bool,
    pub environment_inheritance_allowed: bool,
    pub environment_variables_allowed: bool,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub external_tools_allowed: bool,
    pub child_process_allowed: bool,
    pub label_store_reads_allowed: bool,
    pub training_store_reads_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub historical_state_mutation_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub approved_review: HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_complete_review_chain: bool,
    pub implementation_name: String,
    pub rationale: String,
    pub known_limitations: String,
    pub implementation_contract: HistoricalOutcomeFeatureLabelJoinTargetImplementationContract,
    pub status: String,
    pub exact_approved_review_specification_and_artifact_binding_confirmed: bool,
    pub registrar_independence_confirmed: bool,
    pub implementation_artifact_and_code_revision_immutable_confirmed: bool,
    pub exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed: bool,
    pub point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: bool,
    pub exact_nine_raw_f64_target_projection_without_transform_confirmed: bool,
    pub sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed: bool,
    pub canonical_serialization_and_fixed_input_output_schema_confirmed: bool,
    pub no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: bool,
    pub registration_review_runner_execution_and_output_validation_separation_confirmed: bool,
    pub no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: bool,
    pub future_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub isolated_runner_registration_eligible: bool,
    pub label_access_authorized: bool,
    pub join_execution_authorized: bool,
    pub join_executed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub semantic_target_assigned: bool,
    pub joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub output_validation_authorized: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationItem {
    pub implementation: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
    pub upstream_binding_current: bool,
    pub future_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_reviews: Vec<HistoricalOutcomeFeatureLabelJoinTargetSpecReview>,
    pub items: Vec<HistoricalOutcomeFeatureLabelJoinTargetImplementationItem>,
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub label_access_authorized: bool,
    pub join_execution_authorized: bool,
    pub join_executed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub semantic_target_assigned: bool,
    pub joined_dataset_created: bool,
    pub copied_to_training_store: bool,
    pub training_data_access_authorized: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeFeatureLabelJoinTargetImplementationReadinessSummary {
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_historical_outcome_feature_label_join_target_implementations(
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
            warn!(%error, "feature-label join/target implementation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "特征—标签连接与目标隔离实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_feature_label_join_target_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_feature_label_join_target_implementation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeFeatureLabelJoinTargetImplementationReadinessSummary {
            registration_eligible_count: registry.registration_eligible_count,
            implementation_count: registry.implementation_count,
            current_binding_implementation_count: registry.current_binding_implementation_count,
            independent_implementation_review_eligible_count: registry
                .independent_implementation_review_eligible_count,
            implementation_status: registry.implementation_status,
        },
    )
}

pub(crate) async fn independent_review_eligible_historical_outcome_feature_label_join_target_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord>, String> {
    let registry = build_registry(state).await?;
    let mut records = registry
        .items
        .into_iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .map(|item| item.implementation)
        .collect::<Vec<_>>();
    for record in &records {
        validate_record(record)?;
    }
    records.sort_by(|left, right| {
        right
            .registered_at
            .cmp(&left.registered_at)
            .then_with(|| right.implementation_id.cmp(&left.implementation_id))
    });
    Ok(records)
}

pub(crate) fn validate_historical_outcome_feature_label_join_target_implementation_for_review(
    record: &HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
) -> Result<(), String> {
    validate_record(record)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry, String> {
    let eligible_reviews =
        approved_historical_outcome_feature_label_join_target_spec_reviews(state).await?;
    let review_index = eligible_reviews
        .iter()
        .map(|review| (review.review_id.as_str(), review))
        .collect::<HashMap<_, _>>();
    let mut items = read_records(state)
        .await?
        .into_iter()
        .map(|implementation| {
            let upstream_binding_current = review_index
                .get(implementation.approved_review.review_id.as_str())
                .is_some_and(|review| implementation.approved_review == **review);
            HistoricalOutcomeFeatureLabelJoinTargetImplementationItem {
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
    let current_review_ids = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .map(|item| item.implementation.approved_review.review_id.as_str())
        .collect::<BTreeSet<_>>();
    let registration_eligible_count = eligible_reviews
        .iter()
        .filter(|review| !current_review_ids.contains(review.review_id.as_str()))
        .count();
    let implementation_count = items.len();
    let current_binding_implementation_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .count();
    let implementation_status = if current_binding_implementation_count > 0 {
        "current_join_target_implementation_registered_not_run_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "approved_join_target_specification_ready_for_isolated_implementation_registration"
    } else {
        "waiting_current_independently_approved_join_target_specification"
    };
    Ok(HistoricalOutcomeFeatureLabelJoinTargetImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_reviews,
        items,
        registration_eligible_count,
        implementation_count,
        current_binding_implementation_count,
        independent_implementation_review_eligible_count,
        implementation_status: implementation_status.to_string(),
        label_access_authorized: false,
        join_execution_authorized: false,
        join_executed: false,
        semantic_target_assignment_authorized: false,
        semantic_target_assigned: false,
        joined_dataset_created: false,
        copied_to_training_store: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_run_allowed: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 38 阶段只登记与当前独立批准 join/target 规范精确绑定的不可变隔离实现合同。合同冻结工件、代码版本、一对一连接、九维原始 f64 目标投影、序列化、输入输出与零能力沙箱边界，但没有可调用入口；下一步只允许独立实现复核，不读取标签、不执行 join、不创建 joined/training rows、不训练、不奖励、不建立影子组合、不生成订单，也不接入券商或交易。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord, String> {
    if registrar_id.trim().is_empty() {
        return Err("join/target 隔离实现登记人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let reviews = approved_historical_outcome_feature_label_join_target_spec_reviews(state).await?;
    let review = reviews
        .iter()
        .find(|review| review.review_id == request.expected_review_id)
        .ok_or_else(|| "当前独立批准 join/target 规范不存在，请刷新后重试".to_string())?;
    ensure_review_unregistered(&read_records(state).await?, review)?;
    let record = build_record(review, registrar_id, request, Utc::now())?;
    write_immutable_json(
        &record_root(state).join(format!("{}.json", record.implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn build_record(
    review: &HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord, String> {
    validate_historical_outcome_feature_label_join_target_spec_review_for_implementation(review)?;
    let specification = &review.specification;
    if request.expected_review_id != review.review_id
        || request.expected_review_sha256 != review.review_sha256
        || request.expected_review_contract_sha256 != review.review_contract.contract_sha256
        || request.expected_independent_audit_sha256 != review.independent_audit.audit_sha256
        || request.expected_specification_id != specification.specification_id
        || request.expected_specification_sha256 != specification.specification_sha256
        || request.expected_specification_body_sha256 != specification.specification_body_sha256
        || request.expected_join_specification_sha256
            != specification.join_specification.specification_sha256
        || request.expected_target_specification_sha256
            != specification.target_specification.specification_sha256
        || request.expected_combined_artifact_sha256 != specification.combined_artifact_sha256
        || request.expected_dataset_id != specification.dataset_id
        || request.expected_dataset_content_sha256 != specification.dataset_content_sha256
    {
        return Err(
            "join/target 实现绑定的复核、规范、工件或数据集已变化，请刷新后重试".to_string(),
        );
    }
    let implementation_name =
        bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    let immutable_code_revision = bounded_required(
        &request.immutable_code_revision,
        MAX_REVISION_CHARS,
        "不可变代码版本",
    )?;
    if !valid_sha256(&request.implementation_artifact_sha256) {
        return Err("实现工件 SHA-256 无效".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "登记理由")?;
    let known_limitations = bounded_required(
        &request.known_limitations,
        MAX_LIMITATIONS_CHARS,
        "已知局限",
    )?;
    let excluded_prior_actor_ids = expected_excluded_prior_actor_ids(review);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == registrar_id)
    {
        return Err("实现登记人参与过上游登记或复核链，必须更换独立角色".to_string());
    }
    let all_confirmed = request.exact_approved_review_specification_and_artifact_binding_confirmed
        && request.registrar_independence_confirmed
        && request.implementation_artifact_and_code_revision_immutable_confirmed
        && request.exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed
        && request.point_in_time_missingness_purge_embargo_and_split_isolation_confirmed
        && request.exact_nine_raw_f64_target_projection_without_transform_confirmed
        && request.sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed
        && request.canonical_serialization_and_fixed_input_output_schema_confirmed
        && request.no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed
        && request.registration_review_runner_execution_and_output_validation_separation_confirmed
        && request
            .no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed;
    if !all_confirmed {
        return Err(
            "登记 join/target 隔离实现前必须逐项确认绑定、确定性、隔离和零执行边界".to_string(),
        );
    }
    let contract = implementation_contract(
        &request.implementation_artifact_sha256,
        &immutable_code_revision,
    );
    let mut record = HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord {
        schema_version: IMPLEMENTATION_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        implementation_id: String::new(),
        implementation_sha256: String::new(),
        registered_at,
        registered_by: registrar_id.to_string(),
        approved_review: review.clone(),
        excluded_prior_actor_ids,
        registrar_independent_from_complete_review_chain: true,
        implementation_name,
        rationale,
        known_limitations,
        implementation_contract: contract,
        status: STATUS.to_string(),
        exact_approved_review_specification_and_artifact_binding_confirmed: request
            .exact_approved_review_specification_and_artifact_binding_confirmed,
        registrar_independence_confirmed: request.registrar_independence_confirmed,
        implementation_artifact_and_code_revision_immutable_confirmed: request
            .implementation_artifact_and_code_revision_immutable_confirmed,
        exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed: request
            .exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed,
        point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: request
            .point_in_time_missingness_purge_embargo_and_split_isolation_confirmed,
        exact_nine_raw_f64_target_projection_without_transform_confirmed: request
            .exact_nine_raw_f64_target_projection_without_transform_confirmed,
        sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed: request
            .sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed,
        canonical_serialization_and_fixed_input_output_schema_confirmed: request
            .canonical_serialization_and_fixed_input_output_schema_confirmed,
        no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: request
            .no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed,
        registration_review_runner_execution_and_output_validation_separation_confirmed: request
            .registration_review_runner_execution_and_output_validation_separation_confirmed,
        no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: request
            .no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed,
        future_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        isolated_runner_registration_eligible: false,
        label_access_authorized: false,
        join_execution_authorized: false,
        join_executed: false,
        semantic_target_assignment_authorized: false,
        semantic_target_assigned: false,
        joined_dataset_created: false,
        copied_to_training_store: false,
        output_validation_authorized: false,
        training_data_access_authorized: false,
        training_authorized: false,
        training_run_allowed: false,
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
    implementation_artifact_sha256: &str,
    immutable_code_revision: &str,
) -> HistoricalOutcomeFeatureLabelJoinTargetImplementationContract {
    let mut contract = HistoricalOutcomeFeatureLabelJoinTargetImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_artifact_sha256: implementation_artifact_sha256.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        join_implementation_id: JOIN_IMPLEMENTATION_ID.to_string(),
        join_implementation_version: JOIN_IMPLEMENTATION_VERSION.to_string(),
        target_implementation_id: TARGET_IMPLEMENTATION_ID.to_string(),
        target_implementation_version: TARGET_IMPLEMENTATION_VERSION.to_string(),
        canonical_serializer_version: CANONICAL_SERIALIZER_VERSION.to_string(),
        input_schema_version: INPUT_SCHEMA_VERSION.to_string(),
        output_schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        input_contract: "未来经独立实现复核、隔离 runner 登记与一次性授权后，只能读取精确绑定的正式 split manifest、65 项 feature bundle、原始结果记录与当前规范；当前登记不读取任何输入或标签。".to_string(),
        output_contract: "未来输出只能是 create-once、内容寻址、未进入训练库且须独立验证的一对一 joined-target envelope；当前登记不执行连接、不分配目标、不创建任何输出。".to_string(),
        exact_feature_count: EXPECTED_FEATURE_COUNT,
        exact_target_count: EXPECTED_TARGET_COUNT,
        exact_horizons_market_sessions: REQUIRED_HORIZONS.to_vec(),
        maximum_parallel_datasets: 1,
        maximum_memory_mebibytes: 4_096,
        callable_entrypoint_present: false,
        environment_inheritance_allowed: false,
        environment_variables_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        child_process_allowed: false,
        label_store_reads_allowed: false,
        training_store_reads_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        historical_state_mutation_allowed: false,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static join/target implementation contract is serializable");
    contract
}

fn expected_excluded_prior_actor_ids(
    review: &HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
) -> Vec<String> {
    let mut actors = review.excluded_prior_actor_ids.clone();
    actors.push(review.specification.registered_by.clone());
    actors.push(review.reviewer_id.clone());
    actors.sort();
    actors.dedup();
    actors
}

fn ensure_review_unregistered(
    records: &[HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord],
    review: &HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
) -> Result<(), String> {
    if records
        .iter()
        .any(|record| record.approved_review.review_id == review.review_id)
    {
        return Err("该独立批准 join/target 规范已经登记过隔离实现，不得重复或覆盖".to_string());
    }
    Ok(())
}

fn validate_record(
    record: &HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
) -> Result<(), String> {
    validate_historical_outcome_feature_label_join_target_spec_review_for_implementation(
        &record.approved_review,
    )?;
    let contract = &record.implementation_contract;
    let expected_contract = implementation_contract(
        &contract.implementation_artifact_sha256,
        &contract.immutable_code_revision,
    );
    let all_confirmed = record.exact_approved_review_specification_and_artifact_binding_confirmed
        && record.registrar_independence_confirmed
        && record.implementation_artifact_and_code_revision_immutable_confirmed
        && record.exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed
        && record.point_in_time_missingness_purge_embargo_and_split_isolation_confirmed
        && record.exact_nine_raw_f64_target_projection_without_transform_confirmed
        && record.sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed
        && record.canonical_serialization_and_fixed_input_output_schema_confirmed
        && record.no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed
        && record.registration_review_runner_execution_and_output_validation_separation_confirmed
        && record
            .no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !record.independent_implementation_review_completed
        && !record.isolated_runner_registration_eligible
        && !record.label_access_authorized
        && !record.join_execution_authorized
        && !record.join_executed
        && !record.semantic_target_assignment_authorized
        && !record.semantic_target_assigned
        && !record.joined_dataset_created
        && !record.copied_to_training_store
        && !record.output_validation_authorized
        && !record.training_data_access_authorized
        && !record.training_authorized
        && !record.training_run_allowed
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != IMPLEMENTATION_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || record.status != STATUS
        || !valid_id(&record.implementation_id)
        || !valid_sha256(&record.implementation_sha256)
        || record.implementation_id != record.implementation_sha256[..32]
        || record.implementation_sha256 != record_fingerprint(record)?
        || record.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(&record.approved_review)
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record.registered_by.trim().is_empty()
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.registered_by)
        || !record.registrar_independent_from_complete_review_chain
        || record.implementation_name.trim().is_empty()
        || record.implementation_name.chars().count() > MAX_NAME_CHARS
        || record.rationale.trim().is_empty()
        || record.rationale.chars().count() > MAX_RATIONALE_CHARS
        || record.known_limitations.trim().is_empty()
        || record.known_limitations.chars().count() > MAX_LIMITATIONS_CHARS
        || !valid_sha256(&contract.implementation_artifact_sha256)
        || contract.immutable_code_revision.trim().is_empty()
        || contract.immutable_code_revision.chars().count() > MAX_REVISION_CHARS
        || *contract != expected_contract
        || !all_confirmed
        || !record.future_independent_implementation_review_eligible
        || !authority_closed
    {
        return Err(
            "historical outcome join/target implementation is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord>, String> {
    let root = record_root(state);
    let mut directory = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = tokio::fs::read(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        let record: HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(record.implementation_id.as_str())
        {
            return Err("join/target implementation path mismatch".to_string());
        }
        validate_record(&record)?;
        records.push(record);
    }
    let mut identities = BTreeSet::new();
    let mut review_ids = BTreeSet::new();
    for record in &records {
        if !identities.insert(record.implementation_id.clone())
            || !review_ids.insert(record.approved_review.review_id.clone())
        {
            return Err("duplicate or replayed join/target implementation".to_string());
        }
    }
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["implementation_id", "implementation_sha256"])
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    let bytes = serde_json::to_vec(&value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn bounded_required(value: &str, max_chars: usize, field: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_chars {
        return Err(format!("{field}为空或过长"));
    }
    Ok(value.to_string())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
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

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn record_root(state: &AppState) -> PathBuf {
    decision_root(state)
        .join("historical_outcome_feature_label_join_target_implementations/records")
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
    let path = directory.join("historical-outcome-feature-label-join-target-implementation.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(RegistrationLock { path }),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let stale = tokio::fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                    .is_some_and(|age| age > StdDuration::from_secs(600));
                if stale {
                    let _ = tokio::fs::remove_file(&path).await;
                    continue;
                }
                if attempt < 7 {
                    tokio::time::sleep(StdDuration::from_millis(25)).await;
                    continue;
                }
                return Err("另一个管理员正在登记 join/target 隔离实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("join/target 隔离实现登记锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_feature_label_join_target_spec_reviews::tests::approved_review;

    fn request(
        review: &HistoricalOutcomeFeatureLabelJoinTargetSpecReview,
    ) -> RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest {
        let specification = &review.specification;
        RegisterHistoricalOutcomeFeatureLabelJoinTargetImplementationRequest {
            expected_review_id: review.review_id.clone(),
            expected_review_sha256: review.review_sha256.clone(),
            expected_review_contract_sha256: review.review_contract.contract_sha256.clone(),
            expected_independent_audit_sha256: review.independent_audit.audit_sha256.clone(),
            expected_specification_id: specification.specification_id.clone(),
            expected_specification_sha256: specification.specification_sha256.clone(),
            expected_specification_body_sha256: specification.specification_body_sha256.clone(),
            expected_join_specification_sha256: specification
                .join_specification
                .specification_sha256
                .clone(),
            expected_target_specification_sha256: specification
                .target_specification
                .specification_sha256
                .clone(),
            expected_combined_artifact_sha256: specification.combined_artifact_sha256.clone(),
            expected_dataset_id: specification.dataset_id.clone(),
            expected_dataset_content_sha256: specification.dataset_content_sha256.clone(),
            implementation_name: "isolated exact join and continuous target projector".into(),
            immutable_code_revision: "git:0123456789abcdef".into(),
            implementation_artifact_sha256: "8".repeat(64),
            rationale: "freeze the reviewed deterministic implementation without running it".into(),
            known_limitations: "implementation behavior remains independently unreviewed and no target is strategy truth".into(),
            exact_approved_review_specification_and_artifact_binding_confirmed: true,
            registrar_independence_confirmed: true,
            implementation_artifact_and_code_revision_immutable_confirmed: true,
            exact_one_to_one_join_and_fail_closed_duplicate_missing_keys_confirmed: true,
            point_in_time_missingness_purge_embargo_and_split_isolation_confirmed: true,
            exact_nine_raw_f64_target_projection_without_transform_confirmed: true,
            sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed: true,
            canonical_serialization_and_fixed_input_output_schema_confirmed: true,
            no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: true,
            registration_review_runner_execution_and_output_validation_separation_confirmed: true,
            no_label_access_join_rows_training_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn record() -> HistoricalOutcomeFeatureLabelJoinTargetImplementationRecord {
        let review = approved_review();
        build_record(
            &review,
            "join-target-implementation-registrar",
            request(&review),
            DateTime::from_timestamp(1_750_000_000, 0).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn registration_binds_exact_review_specification_artifact_and_dataset() {
        let value = record();
        let review = approved_review();
        assert_eq!(value.approved_review.review_sha256, review.review_sha256);
        assert_eq!(
            value.approved_review.specification.combined_artifact_sha256,
            review.specification.combined_artifact_sha256
        );
        assert!(validate_record(&value).is_ok());
    }

    #[test]
    fn registrar_must_be_independent_from_complete_prior_chain() {
        let review = approved_review();
        for actor in expected_excluded_prior_actor_ids(&review) {
            assert!(build_record(&review, &actor, request(&review), Utc::now()).is_err());
        }
    }

    #[test]
    fn contract_freezes_exact_join_target_serializer_schemas_and_counts() {
        let value = record();
        let contract = value.implementation_contract;
        assert_eq!(contract.join_implementation_id, JOIN_IMPLEMENTATION_ID);
        assert_eq!(contract.target_implementation_id, TARGET_IMPLEMENTATION_ID);
        assert_eq!(contract.exact_feature_count, EXPECTED_FEATURE_COUNT);
        assert_eq!(contract.exact_target_count, EXPECTED_TARGET_COUNT);
        assert_eq!(contract.exact_horizons_market_sessions, REQUIRED_HORIZONS);
        assert!(valid_sha256(&contract.contract_sha256));
    }

    #[test]
    fn registration_has_no_entrypoint_label_access_join_training_or_trading_authority() {
        let value = record();
        let contract = &value.implementation_contract;
        assert!(!contract.callable_entrypoint_present);
        assert!(!contract.label_store_reads_allowed);
        assert!(!contract.outbound_network_allowed);
        assert!(!value.label_access_authorized);
        assert!(!value.join_execution_authorized);
        assert!(!value.joined_dataset_created);
        assert!(!value.training_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn only_next_gate_is_independent_implementation_review() {
        let value = record();
        assert!(value.future_independent_implementation_review_eligible);
        assert!(!value.independent_implementation_review_completed);
        assert!(!value.isolated_runner_registration_eligible);
        assert!(!value.output_validation_authorized);
    }

    #[test]
    fn any_contract_or_authority_tamper_fails_closed() {
        let mut value = record();
        value.implementation_contract.target_implementation_version = "changed".into();
        assert!(validate_record(&value).is_err());
        let mut value = record();
        value.semantic_target_assignment_authorized = true;
        value.implementation_sha256 = record_fingerprint(&value).unwrap();
        value.implementation_id = value.implementation_sha256[..32].to_string();
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn stale_binding_duplicate_registration_or_missing_confirmation_is_rejected() {
        let review = approved_review();
        assert!(ensure_review_unregistered(&[record()], &review).is_err());
        let mut stale = request(&review);
        stale.expected_target_specification_sha256 = "0".repeat(64);
        assert!(
            build_record(
                &review,
                "join-target-implementation-registrar",
                stale,
                Utc::now()
            )
            .is_err()
        );
        let mut incomplete = request(&review);
        incomplete.sealed_holdout_labels_inaccessible_to_training_and_tuning_confirmed = false;
        assert!(
            build_record(
                &review,
                "join-target-implementation-registrar",
                incomplete,
                Utc::now()
            )
            .is_err()
        );
    }

    #[test]
    fn implementation_contract_does_not_define_action_position_threshold_rank_or_reward() {
        let json = serde_json::to_string(&record().implementation_contract).unwrap();
        for forbidden in [
            "buy_hold_sell",
            "portfolio_weight",
            "action_threshold",
            "rank_transform",
            "scalar_reward",
        ] {
            assert!(!json.contains(forbidden));
        }
    }
}
