//! Immutable registration of one future isolated offline-dataset transformation implementation.
//!
//! Registration freezes a declarative implementation contract after an independent Stage 26
//! approval. It deliberately has no callable entrypoint and cannot read secrets, inherit an
//! environment, access a network or tool, generate an artifact, execute a transformation, assign
//! a target, train a model, create reward evidence, shadow a portfolio, generate an order, access
//! a broker or trade. A separate independent implementation review is the only next gate.

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

use super::historical_outcome_offline_dataset_transformation_spec_reviews::{
    HistoricalOutcomeOfflineDatasetTransformationSpecReview,
    approved_historical_outcome_offline_dataset_transformation_spec_reviews,
    validate_historical_outcome_offline_dataset_transformation_spec_review_for_implementation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-implementation-registry-v1";
const IMPLEMENTATION_SCHEMA_VERSION: &str =
    "hone-historical-outcome-offline-dataset-transformation-implementation-v1";
const POLICY_VERSION: &str =
    "hone-offline-dataset-transformation-implementation-registration-v1-no-execution";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-offline-dataset-transformation-implementation-contract-v1";
const SPLIT_IMPLEMENTATION_ID: &str = "hone-deterministic-connected-component-boundary-enumerator";
const SPLIT_IMPLEMENTATION_VERSION: &str = "v1-exact-integer-objective";
const FEATURE_IMPLEMENTATION_ID: &str = "hone-point-in-time-feature-allowlist-extractor";
const FEATURE_IMPLEMENTATION_VERSION: &str = "v1-exact-65-feature-ids";
const CANONICAL_SERIALIZER_VERSION: &str = "hone-canonical-json-serializer-v1";
const INPUT_SCHEMA_VERSION: &str = "hone-sealed-outcome-transformation-input-v1";
const OUTPUT_SCHEMA_VERSION: &str = "hone-unmaterialized-transformation-output-v1";
const STATUS: &str = "registered_not_run";
const MAX_NAME_CHARS: usize = 160;
const MAX_REVISION_CHARS: usize = 200;
const MAX_RATIONALE_CHARS: usize = 2_400;
const MAX_LIMITATIONS_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest {
    expected_review_id: String,
    expected_review_sha256: String,
    expected_review_contract_sha256: String,
    expected_transformation_spec_id: String,
    expected_transformation_spec_sha256: String,
    expected_transformation_body_sha256: String,
    expected_split_specification_sha256: String,
    expected_feature_specification_sha256: String,
    expected_dataset_content_sha256: String,
    expected_manifest_sha256: String,
    expected_candidate_set_sha256: String,
    expected_governance_review_id: String,
    expected_governance_review_sha256: String,
    implementation_name: String,
    immutable_code_revision: String,
    implementation_artifact_sha256: String,
    rationale: String,
    known_limitations: String,
    exact_approved_review_and_specification_binding_confirmed: bool,
    registrar_independence_confirmed: bool,
    implementation_artifact_and_code_revision_immutable_confirmed: bool,
    deterministic_split_and_feature_implementation_confirmed: bool,
    canonical_serialization_and_fixed_schema_confirmed: bool,
    sealed_read_only_input_and_create_once_output_confirmed: bool,
    no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: bool,
    registration_review_execution_and_output_validation_separation_confirmed: bool,
    no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationContract {
    pub schema_version: String,
    pub contract_sha256: String,
    pub implementation_artifact_sha256: String,
    pub immutable_code_revision: String,
    pub split_implementation_id: String,
    pub split_implementation_version: String,
    pub feature_implementation_id: String,
    pub feature_implementation_version: String,
    pub canonical_serializer_version: String,
    pub input_schema_version: String,
    pub output_schema_version: String,
    pub input_contract: String,
    pub output_contract: String,
    pub maximum_parallel_subjects: u8,
    pub maximum_memory_mebibytes: u32,
    pub callable_entrypoint_present: bool,
    pub environment_inheritance_allowed: bool,
    pub environment_variables_allowed: bool,
    pub secrets_allowed: bool,
    pub outbound_network_allowed: bool,
    pub external_tools_allowed: bool,
    pub child_process_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub historical_state_mutation_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub approved_review: HistoricalOutcomeOfflineDatasetTransformationSpecReview,
    pub excluded_prior_actor_ids: Vec<String>,
    pub registrar_independent_from_complete_review_chain: bool,
    pub implementation_name: String,
    pub rationale: String,
    pub known_limitations: String,
    pub implementation_contract:
        HistoricalOutcomeOfflineDatasetTransformationImplementationContract,
    pub status: String,
    pub exact_approved_review_and_specification_binding_confirmed: bool,
    pub registrar_independence_confirmed: bool,
    pub implementation_artifact_and_code_revision_immutable_confirmed: bool,
    pub deterministic_split_and_feature_implementation_confirmed: bool,
    pub canonical_serialization_and_fixed_schema_confirmed: bool,
    pub sealed_read_only_input_and_create_once_output_confirmed: bool,
    pub no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: bool,
    pub registration_review_execution_and_output_validation_separation_confirmed: bool,
    pub no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
        bool,
    pub future_independent_implementation_review_eligible: bool,
    pub independent_implementation_review_completed: bool,
    pub split_manifest_generation_authorized: bool,
    pub split_manifest_generated: bool,
    pub feature_bundle_generation_authorized: bool,
    pub feature_bundle_generated: bool,
    pub feature_join_authorized: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assignment_authorized: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationItem {
    pub implementation: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
    pub upstream_binding_current: bool,
    pub future_independent_implementation_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub eligible_reviews: Vec<HistoricalOutcomeOfflineDatasetTransformationSpecReview>,
    pub items: Vec<HistoricalOutcomeOfflineDatasetTransformationImplementationItem>,
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
    pub split_manifest_generation_authorized: bool,
    pub split_manifest_generated: bool,
    pub feature_bundle_generation_authorized: bool,
    pub feature_bundle_generated: bool,
    pub feature_join_authorized: bool,
    pub feature_join_performed: bool,
    pub semantic_target_assignment_authorized: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetTransformationImplementationReadinessSummary {
    pub registration_eligible_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub independent_implementation_review_eligible_count: usize,
    pub implementation_status: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_dataset_transformation_implementations(
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
            warn!(%error, "historical outcome transformation implementation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果隔离转换实现登记表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_offline_dataset_transformation_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_once(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome transformation implementation registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果隔离转换实现登记表暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_transformation_implementation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationImplementationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeOfflineDatasetTransformationImplementationReadinessSummary {
            registration_eligible_count: registry.registration_eligible_count,
            implementation_count: registry.implementation_count,
            current_binding_implementation_count: registry.current_binding_implementation_count,
            independent_implementation_review_eligible_count: registry
                .independent_implementation_review_eligible_count,
            implementation_status: registry.implementation_status,
        },
    )
}

pub(crate) async fn independent_review_eligible_historical_outcome_offline_dataset_transformation_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationImplementationRecord>, String> {
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

pub(crate) fn validate_historical_outcome_offline_dataset_transformation_implementation_for_review(
    record: &HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
) -> Result<(), String> {
    validate_record(record)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationImplementationRegistry, String> {
    let eligible_reviews =
        approved_historical_outcome_offline_dataset_transformation_spec_reviews(state).await?;
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
                .is_some_and(|review| binding_matches(&implementation, review));
            HistoricalOutcomeOfflineDatasetTransformationImplementationItem {
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
    let current_binding_implementation_count = items
        .iter()
        .filter(|item| item.upstream_binding_current)
        .count();
    let independent_implementation_review_eligible_count = items
        .iter()
        .filter(|item| item.future_independent_implementation_review_eligible)
        .count();
    let implementation_count = items.len();
    let implementation_status = if current_binding_implementation_count > 0 {
        "current_implementation_registered_not_run_waiting_independent_review"
    } else if registration_eligible_count > 0 {
        "approved_specification_ready_for_isolated_implementation_registration"
    } else {
        "waiting_current_independently_approved_transformation_specification"
    };
    Ok(HistoricalOutcomeOfflineDatasetTransformationImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        eligible_reviews,
        items,
        registration_eligible_count,
        implementation_count,
        current_binding_implementation_count,
        independent_implementation_review_eligible_count,
        implementation_status: implementation_status.to_string(),
        split_manifest_generation_authorized: false,
        split_manifest_generated: false,
        feature_bundle_generation_authorized: false,
        feature_bundle_generated: false,
        feature_join_authorized: false,
        feature_join_performed: false,
        semantic_target_assignment_authorized: false,
        training_authorized: false,
        training_run_allowed: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第 27 阶段只登记一份与当前独立批准转换规范精确绑定的隔离实现合同。实现工件、代码版本、确定性算法、序列化、输入输出与沙箱边界被冻结，但没有可调用入口；下一步只允许独立实现复核，不运行转换、不生成 manifest 或 bundle、不连接特征、不定义目标、不训练、不奖励、不建立影子组合、不生成订单，也不接入券商或交易。".to_string(),
    })
}

async fn register_once(
    state: &AppState,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationImplementationRecord, String> {
    if registrar_id.trim().is_empty() {
        return Err("隔离转换实现登记人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let reviews =
        approved_historical_outcome_offline_dataset_transformation_spec_reviews(state).await?;
    let review = reviews
        .iter()
        .find(|review| review.review_id == request.expected_review_id)
        .ok_or_else(|| "当前独立批准转换规范不存在，请刷新后重试".to_string())?;
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
    review: &HistoricalOutcomeOfflineDatasetTransformationSpecReview,
    registrar_id: &str,
    request: RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeOfflineDatasetTransformationImplementationRecord, String> {
    validate_historical_outcome_offline_dataset_transformation_spec_review_for_implementation(
        review,
    )?;
    if !review.future_isolated_transformation_implementation_registration_eligible {
        return Err("转换规范独立复核未批准实现登记".to_string());
    }
    let spec = &review.specification;
    if request.expected_review_id != review.review_id
        || request.expected_review_sha256 != review.review_sha256
        || request.expected_review_contract_sha256 != review.review_contract.contract_sha256
        || request.expected_transformation_spec_id != spec.transformation_spec_id
        || request.expected_transformation_spec_sha256 != spec.transformation_spec_sha256
        || request.expected_transformation_body_sha256 != spec.transformation_body_sha256
        || request.expected_split_specification_sha256
            != spec.split_manifest_specification.specification_sha256
        || request.expected_feature_specification_sha256
            != spec.feature_bundle_specification.specification_sha256
        || request.expected_dataset_content_sha256 != spec.subject.dataset_content_sha256
        || request.expected_manifest_sha256 != spec.subject.manifest_sha256
        || request.expected_candidate_set_sha256 != spec.subject.candidate_set_sha256
        || request.expected_governance_review_id != spec.governance_review_id
        || request.expected_governance_review_sha256 != spec.governance_review_sha256
    {
        return Err("转换实现绑定的复核、规范、治理或数据集已变化，请刷新后重试".to_string());
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
        return Err("实现登记人参与过数据集、治理、规范登记或独立复核链，必须更换角色".to_string());
    }
    let all_confirmed = request.exact_approved_review_and_specification_binding_confirmed
        && request.registrar_independence_confirmed
        && request.implementation_artifact_and_code_revision_immutable_confirmed
        && request.deterministic_split_and_feature_implementation_confirmed
        && request.canonical_serialization_and_fixed_schema_confirmed
        && request.sealed_read_only_input_and_create_once_output_confirmed
        && request.no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed
        && request.registration_review_execution_and_output_validation_separation_confirmed
        && request.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed;
    if !all_confirmed {
        return Err("登记隔离转换实现前必须逐项确认不可变性、确定性、沙箱和零执行边界".to_string());
    }
    let contract = implementation_contract(
        &request.implementation_artifact_sha256,
        &immutable_code_revision,
    );
    let mut record = HistoricalOutcomeOfflineDatasetTransformationImplementationRecord {
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
        exact_approved_review_and_specification_binding_confirmed: request
            .exact_approved_review_and_specification_binding_confirmed,
        registrar_independence_confirmed: request.registrar_independence_confirmed,
        implementation_artifact_and_code_revision_immutable_confirmed: request
            .implementation_artifact_and_code_revision_immutable_confirmed,
        deterministic_split_and_feature_implementation_confirmed: request
            .deterministic_split_and_feature_implementation_confirmed,
        canonical_serialization_and_fixed_schema_confirmed: request
            .canonical_serialization_and_fixed_schema_confirmed,
        sealed_read_only_input_and_create_once_output_confirmed: request
            .sealed_read_only_input_and_create_once_output_confirmed,
        no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: request
            .no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed,
        registration_review_execution_and_output_validation_separation_confirmed: request
            .registration_review_execution_and_output_validation_separation_confirmed,
        no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed:
            request.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed,
        future_independent_implementation_review_eligible: true,
        independent_implementation_review_completed: false,
        split_manifest_generation_authorized: false,
        split_manifest_generated: false,
        feature_bundle_generation_authorized: false,
        feature_bundle_generated: false,
        feature_join_authorized: false,
        feature_join_performed: false,
        semantic_target_assignment_authorized: false,
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
) -> HistoricalOutcomeOfflineDatasetTransformationImplementationContract {
    let mut contract = HistoricalOutcomeOfflineDatasetTransformationImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        implementation_artifact_sha256: implementation_artifact_sha256.to_string(),
        immutable_code_revision: immutable_code_revision.to_string(),
        split_implementation_id: SPLIT_IMPLEMENTATION_ID.to_string(),
        split_implementation_version: SPLIT_IMPLEMENTATION_VERSION.to_string(),
        feature_implementation_id: FEATURE_IMPLEMENTATION_ID.to_string(),
        feature_implementation_version: FEATURE_IMPLEMENTATION_VERSION.to_string(),
        canonical_serializer_version: CANONICAL_SERIALIZER_VERSION.to_string(),
        input_schema_version: INPUT_SCHEMA_VERSION.to_string(),
        output_schema_version: OUTPUT_SCHEMA_VERSION.to_string(),
        input_contract: "未来经独立复核与另行授权后，只可读取与批准复核精确绑定的不可变数据集内容、manifest、候选集、转换规范及其点时来源工件；当前登记不读取任何输入。".to_string(),
        output_contract: "未来输出只能是 create-once、内容寻址且另行验证的 split manifest 与 65 项点时 feature bundle；当前登记不生成、不连接、不写入任何输出。".to_string(),
        maximum_parallel_subjects: 1,
        maximum_memory_mebibytes: 2_048,
        callable_entrypoint_present: false,
        environment_inheritance_allowed: false,
        environment_variables_allowed: false,
        secrets_allowed: false,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        child_process_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        historical_state_mutation_allowed: false,
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])
        .expect("static isolated transformation implementation contract is serializable");
    contract
}

fn expected_excluded_prior_actor_ids(
    review: &HistoricalOutcomeOfflineDatasetTransformationSpecReview,
) -> Vec<String> {
    let mut actors = review.excluded_prior_actor_ids.clone();
    actors.push(review.specification.registered_by.clone());
    actors.push(review.specification.governance_reviewer_id.clone());
    actors.push(review.reviewer_id.clone());
    actors.sort();
    actors.dedup();
    actors
}

fn binding_matches(
    record: &HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
    review: &HistoricalOutcomeOfflineDatasetTransformationSpecReview,
) -> bool {
    record.approved_review == *review
}

fn ensure_review_unregistered(
    records: &[HistoricalOutcomeOfflineDatasetTransformationImplementationRecord],
    review: &HistoricalOutcomeOfflineDatasetTransformationSpecReview,
) -> Result<(), String> {
    if records
        .iter()
        .any(|record| record.approved_review.review_id == review.review_id)
    {
        return Err("该独立批准转换规范已经登记过隔离实现，不得重复或覆盖".to_string());
    }
    Ok(())
}

fn validate_record(
    record: &HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
) -> Result<(), String> {
    validate_historical_outcome_offline_dataset_transformation_spec_review_for_implementation(
        &record.approved_review,
    )?;
    let contract = &record.implementation_contract;
    let expected_contract = implementation_contract(
        &contract.implementation_artifact_sha256,
        &contract.immutable_code_revision,
    );
    let all_confirmed = record.exact_approved_review_and_specification_binding_confirmed
        && record.registrar_independence_confirmed
        && record.implementation_artifact_and_code_revision_immutable_confirmed
        && record.deterministic_split_and_feature_implementation_confirmed
        && record.canonical_serialization_and_fixed_schema_confirmed
        && record.sealed_read_only_input_and_create_once_output_confirmed
        && record.no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed
        && record.registration_review_execution_and_output_validation_separation_confirmed
        && record.no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed;
    let authority_closed = !record.independent_implementation_review_completed
        && !record.split_manifest_generation_authorized
        && !record.split_manifest_generated
        && !record.feature_bundle_generation_authorized
        && !record.feature_bundle_generated
        && !record.feature_join_authorized
        && !record.feature_join_performed
        && !record.semantic_target_assignment_authorized
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
        || !record
            .approved_review
            .future_isolated_transformation_implementation_registration_eligible
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
            "historical outcome transformation implementation is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetTransformationImplementationRecord>, String> {
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
        let record: HistoricalOutcomeOfflineDatasetTransformationImplementationRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(record.implementation_id.as_str())
        {
            return Err("transformation implementation path mismatch".to_string());
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
            return Err("duplicate or replayed transformation implementation".to_string());
        }
    }
    Ok(records)
}

fn record_fingerprint(
    record: &HistoricalOutcomeOfflineDatasetTransformationImplementationRecord,
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
        .join("historical_outcome_offline_dataset_transformation_implementations/records")
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
    let path =
        directory.join("historical-outcome-offline-dataset-transformation-implementation.lock");
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
                return Err("另一个管理员正在登记隔离转换实现".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("隔离转换实现登记锁不可用".to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::routes::historical_outcome_offline_dataset_transformation_spec_reviews::tests::approved_review;

    fn request(
        review: &HistoricalOutcomeOfflineDatasetTransformationSpecReview,
    ) -> RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest {
        let spec = &review.specification;
        RegisterHistoricalOutcomeOfflineDatasetTransformationImplementationRequest {
            expected_review_id: review.review_id.clone(),
            expected_review_sha256: review.review_sha256.clone(),
            expected_review_contract_sha256: review.review_contract.contract_sha256.clone(),
            expected_transformation_spec_id: spec.transformation_spec_id.clone(),
            expected_transformation_spec_sha256: spec.transformation_spec_sha256.clone(),
            expected_transformation_body_sha256: spec.transformation_body_sha256.clone(),
            expected_split_specification_sha256: spec
                .split_manifest_specification
                .specification_sha256
                .clone(),
            expected_feature_specification_sha256: spec
                .feature_bundle_specification
                .specification_sha256
                .clone(),
            expected_dataset_content_sha256: spec.subject.dataset_content_sha256.clone(),
            expected_manifest_sha256: spec.subject.manifest_sha256.clone(),
            expected_candidate_set_sha256: spec.subject.candidate_set_sha256.clone(),
            expected_governance_review_id: spec.governance_review_id.clone(),
            expected_governance_review_sha256: spec.governance_review_sha256.clone(),
            implementation_name: "isolated deterministic transformer".to_string(),
            immutable_code_revision: "git:0123456789abcdef".to_string(),
            implementation_artifact_sha256: "8".repeat(64),
            rationale: "freeze the independently reviewed contract without running it".to_string(),
            known_limitations: "implementation behavior has not been independently reviewed or run".to_string(),
            exact_approved_review_and_specification_binding_confirmed: true,
            registrar_independence_confirmed: true,
            implementation_artifact_and_code_revision_immutable_confirmed: true,
            deterministic_split_and_feature_implementation_confirmed: true,
            canonical_serialization_and_fixed_schema_confirmed: true,
            sealed_read_only_input_and_create_once_output_confirmed: true,
            no_entrypoint_environment_secrets_network_tools_or_child_process_confirmed: true,
            registration_review_execution_and_output_validation_separation_confirmed: true,
            no_manifest_bundle_join_target_training_reward_shadow_order_broker_or_trading_confirmed: true,
        }
    }

    pub(crate) fn record() -> HistoricalOutcomeOfflineDatasetTransformationImplementationRecord {
        let review = approved_review();
        build_record(
            &review,
            "implementation-registrar",
            request(&review),
            DateTime::from_timestamp(1_750_000_000, 0).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn registration_binds_exact_approved_review_specification_and_dataset() {
        let value = record();
        assert_eq!(value.approved_review.review_id, approved_review().review_id);
        assert!(validate_record(&value).is_ok());
    }

    #[test]
    fn registrar_must_be_independent_from_complete_chain() {
        let review = approved_review();
        for actor in [
            "registrar",
            "governance-reviewer",
            "transformation-spec-reviewer",
        ] {
            assert!(build_record(&review, actor, request(&review), Utc::now()).is_err());
        }
    }

    #[test]
    fn contract_freezes_artifact_algorithms_serializer_and_schemas() {
        let value = record();
        let contract = value.implementation_contract;
        assert_eq!(contract.split_implementation_id, SPLIT_IMPLEMENTATION_ID);
        assert_eq!(
            contract.feature_implementation_id,
            FEATURE_IMPLEMENTATION_ID
        );
        assert_eq!(
            contract.canonical_serializer_version,
            CANONICAL_SERIALIZER_VERSION
        );
        assert!(valid_sha256(&contract.contract_sha256));
    }

    #[test]
    fn registration_has_no_entrypoint_secret_network_tool_or_downstream_authority() {
        let value = record();
        let contract = &value.implementation_contract;
        assert!(!contract.callable_entrypoint_present);
        assert!(!contract.environment_variables_allowed);
        assert!(!contract.secrets_allowed);
        assert!(!contract.outbound_network_allowed);
        assert!(!contract.external_tools_allowed);
        assert!(!value.split_manifest_generated);
        assert!(!value.feature_bundle_generated);
        assert!(!value.training_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn only_next_gate_is_independent_implementation_review() {
        let value = record();
        assert!(value.future_independent_implementation_review_eligible);
        assert!(!value.independent_implementation_review_completed);
        assert!(!value.split_manifest_generation_authorized);
        assert!(!value.semantic_target_assignment_authorized);
    }

    #[test]
    fn any_content_or_authority_tamper_fails_closed() {
        let mut value = record();
        value.implementation_contract.feature_implementation_version = "changed".to_string();
        assert!(validate_record(&value).is_err());
        let mut value = record();
        value.order_generation_authorized = true;
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn stale_binding_or_missing_confirmation_is_rejected() {
        let review = approved_review();
        assert!(ensure_review_unregistered(&[record()], &review).is_err());
        let mut stale = request(&review);
        stale.expected_review_sha256 = "0".repeat(64);
        assert!(build_record(&review, "implementation-registrar", stale, Utc::now()).is_err());
        let mut incomplete = request(&review);
        incomplete.canonical_serialization_and_fixed_schema_confirmed = false;
        assert!(build_record(&review, "implementation-registrar", incomplete, Utc::now()).is_err());
    }
}
