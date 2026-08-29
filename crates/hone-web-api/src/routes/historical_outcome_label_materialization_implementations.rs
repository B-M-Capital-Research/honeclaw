//! Immutable registration for a future historical outcome-label materializer.
//!
//! Registration freezes one deterministic implementation specification over
//! one exact independently admitted output. It does not run the materializer,
//! write a label, infer investment semantics, train, reward, shadow, order or
//! trade. A separate future authorization review is required before any run.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_label_admission_reviews::{
    AdmittedHistoricalOutcomeForLabelMaterialization,
    admitted_historical_outcomes_for_label_materialization,
    exact_admitted_historical_outcome_for_label_materialization,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-implementation-registry-v1";
const IMPLEMENTATION_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-implementation-v1";
const MATERIALIZATION_POLICY_VERSION: &str =
    "hone-historical-outcome-label-materialization-v1-raw-outcome-envelope-only";
const OUTPUT_LABEL_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-envelope-v1-raw-validated-market-outcome";
const STATUS: &str = "registered_not_run";
const INPUT_CONTRACT: &str = "只读输入：一条当前独立准入复核通过的精确 validation、claim/result/output、封存行情、20/60/250 共同交易日指标、冻结协议和已知局限；实现不得联网、补数、重算、覆盖或读取其它历史样本。";
const OUTPUT_CONTRACT: &str = "未来经单独授权的物化只能 create-once 写入一份原始观察结果信封，逐位保留已验证的标的收益、SPY 收益、超额收益、最大回撤、完整来源绑定和已知局限；不得推断方向、评级、买卖动作、仓位或奖励。当前登记不运行也不写标签。";
const MAX_NAME_CHARS: usize = 120;
const MAX_REVISION_CHARS: usize = 160;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeLabelMaterializationImplementationKind {
    DeterministicRawValidatedOutcomeEnvelope,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeLabelMaterializationImplementationRequest {
    attempt_id: String,
    expected_admission_review_id: String,
    expected_admission_review_sha256: String,
    expected_validation_sha256: String,
    expected_output_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    implementation_name: String,
    implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind,
    code_revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationImplementationRecord {
    pub schema_version: String,
    pub materialization_policy_version: String,
    pub materialization_implementation_id: String,
    pub materialization_implementation_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub admission_reviewer_id: String,
    pub admission_known_limitations: String,
    pub attempt_id: String,
    pub claim_sha256: String,
    pub result_id: String,
    pub result_sha256: String,
    pub output_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub common_session_count: usize,
    pub metric_horizons_market_sessions: Vec<u16>,
    pub metric_start_date: NaiveDate,
    pub metric_end_dates: Vec<NaiveDate>,
    pub recomputed_metrics_sha256: String,
    pub output_label_schema_version: String,
    pub implementation_name: String,
    pub implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind,
    pub code_revision: String,
    pub status: String,
    pub input_contract: String,
    pub output_contract: String,
    pub output_fields: Vec<String>,
    pub deterministic_projection_required: bool,
    pub exact_metric_bit_preservation_required: bool,
    pub create_once_output_required: bool,
    pub isolated_output_required: bool,
    pub known_limitations_preservation_required: bool,
    pub missing_data_fail_closed_required: bool,
    pub manual_metric_override_allowed: bool,
    pub direction_inference_allowed: bool,
    pub rating_inference_allowed: bool,
    pub investment_action_inference_allowed: bool,
    pub position_sizing_inference_allowed: bool,
    pub reward_semantics_inference_allowed: bool,
    pub outbound_network_allowed: bool,
    pub external_tools_allowed: bool,
    pub production_reads_allowed: bool,
    pub production_writes_allowed: bool,
    pub historical_state_mutation_allowed: bool,
    pub label_materialization_run_authorized: bool,
    pub outcome_label_write_allowed: bool,
    pub label_materialization_enabled: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationImplementationItem {
    pub implementation: HistoricalOutcomeLabelMaterializationImplementationRecord,
    pub admission_binding_current: bool,
    pub run_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationImplementationRegistry {
    pub schema_version: String,
    pub materialization_policy_version: String,
    pub output_label_schema_version: String,
    pub eligible_admissions: Vec<AdmittedHistoricalOutcomeProjection>,
    pub allowed_implementation_kinds: Vec<HistoricalOutcomeLabelMaterializationImplementationKind>,
    pub registration_allowed: bool,
    pub implementations: Vec<HistoricalOutcomeLabelMaterializationImplementationItem>,
    pub admitted_output_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub run_authorization_review_eligible_count: usize,
    pub implementation_status: String,
    pub label_materialization_enabled: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdmittedHistoricalOutcomeProjection {
    pub attempt_id: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub output_sha256: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub known_limitations: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeLabelMaterializationImplementationReadinessSummary {
    pub admitted_output_count: usize,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub run_authorization_review_eligible_count: usize,
    pub implementation_status: String,
    pub label_materialization_enabled: bool,
    pub outcome_label_written: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ReviewEligibleHistoricalOutcomeLabelMaterializationImplementation {
    pub implementation: HistoricalOutcomeLabelMaterializationImplementationRecord,
    pub admission: AdmittedHistoricalOutcomeForLabelMaterialization,
}

pub(crate) async fn handle_get_historical_outcome_label_materialization_implementations(
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
            warn!(%error, "historical outcome label materialization implementation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果标签物化实现注册表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_label_materialization_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeLabelMaterializationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_implementation(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome label materialization implementation registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果标签物化实现注册表暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_label_materialization_implementation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelMaterializationImplementationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeLabelMaterializationImplementationReadinessSummary {
            admitted_output_count: registry.admitted_output_count,
            implementation_count: registry.implementation_count,
            current_binding_implementation_count: registry.current_binding_implementation_count,
            run_authorization_review_eligible_count: registry
                .run_authorization_review_eligible_count,
            implementation_status: registry.implementation_status,
            label_materialization_enabled: false,
            outcome_label_written: false,
        },
    )
}

pub(crate) async fn run_authorization_review_eligible_implementations(
    state: &AppState,
) -> Result<Vec<ReviewEligibleHistoricalOutcomeLabelMaterializationImplementation>, String> {
    let admissions = admitted_historical_outcomes_for_label_materialization(state).await?;
    let admission_index = admissions
        .into_iter()
        .map(|value| (value.admission_review.review_id.clone(), value))
        .collect::<HashMap<_, _>>();
    let mut eligible = Vec::new();
    for implementation in read_implementations(state).await? {
        let Some(admission) = admission_index.get(&implementation.admission_review_id) else {
            continue;
        };
        if binding_matches(&implementation, admission) {
            eligible.push(
                ReviewEligibleHistoricalOutcomeLabelMaterializationImplementation {
                    implementation,
                    admission: admission.clone(),
                },
            );
        }
    }
    eligible.sort_by(|left, right| {
        right
            .implementation
            .registered_at
            .cmp(&left.implementation.registered_at)
            .then_with(|| {
                right
                    .implementation
                    .materialization_implementation_id
                    .cmp(&left.implementation.materialization_implementation_id)
            })
    });
    Ok(eligible)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelMaterializationImplementationRegistry, String> {
    let admissions = admitted_historical_outcomes_for_label_materialization(state).await?;
    let admission_index = admissions
        .iter()
        .map(|value| (value.admission_review.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let eligible_admissions = admissions
        .iter()
        .map(AdmittedHistoricalOutcomeProjection::from)
        .collect::<Vec<_>>();
    let mut implementations = read_implementations(state)
        .await?
        .into_iter()
        .map(|implementation| {
            let admission_binding_current = admission_index
                .get(implementation.admission_review_id.as_str())
                .is_some_and(|admission| binding_matches(&implementation, admission));
            HistoricalOutcomeLabelMaterializationImplementationItem {
                run_authorization_review_eligible: admission_binding_current,
                admission_binding_current,
                implementation,
            }
        })
        .collect::<Vec<_>>();
    implementations.sort_by(|left, right| {
        right
            .implementation
            .registered_at
            .cmp(&left.implementation.registered_at)
            .then_with(|| {
                right
                    .implementation
                    .materialization_implementation_id
                    .cmp(&left.implementation.materialization_implementation_id)
            })
    });
    let admitted_output_count = admissions.len();
    let implementation_count = implementations.len();
    let current_binding_implementation_count = implementations
        .iter()
        .filter(|item| item.admission_binding_current)
        .count();
    let run_authorization_review_eligible_count = implementations
        .iter()
        .filter(|item| item.run_authorization_review_eligible)
        .count();
    let implementation_status = if admissions.is_empty() {
        "waiting_for_admitted_output"
    } else if implementation_count == 0 {
        "waiting_for_immutable_materialization_implementation_registration"
    } else if current_binding_implementation_count == 0 {
        "registered_materialization_binding_stale"
    } else {
        STATUS
    };
    Ok(
        HistoricalOutcomeLabelMaterializationImplementationRegistry {
            schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
            materialization_policy_version: MATERIALIZATION_POLICY_VERSION.to_string(),
            output_label_schema_version: OUTPUT_LABEL_SCHEMA_VERSION.to_string(),
            eligible_admissions,
            allowed_implementation_kinds: vec![
                HistoricalOutcomeLabelMaterializationImplementationKind::DeterministicRawValidatedOutcomeEnvelope,
            ],
            registration_allowed: admitted_output_count > 0,
            implementations,
            admitted_output_count,
            implementation_count,
            current_binding_implementation_count,
            run_authorization_review_eligible_count,
            implementation_status: implementation_status.to_string(),
            label_materialization_enabled: false,
            outcome_label_written: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
            scope: "第十四阶段只登记与一条当前独立准入输出精确绑定的确定性标签物化实现规范。规范未来也只能原样封装已验证指标、来源和已知局限，不能推断方向、评级、动作、仓位或奖励；当前不运行、不写标签、不训练、不建立影子组合、不生成订单或交易。".to_string(),
        },
    )
}

async fn register_implementation(
    state: &AppState,
    admin_id: &str,
    request: RegisterHistoricalOutcomeLabelMaterializationImplementationRequest,
) -> Result<HistoricalOutcomeLabelMaterializationImplementationRecord, String> {
    if !valid_id(&request.attempt_id) {
        return Err("执行尝试 ID 无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        "historical-outcome-label-materialization-implementation",
    )
    .await?;
    let admission = exact_admitted_historical_outcome_for_label_materialization(
        state,
        &request.attempt_id,
        &request.expected_admission_review_id,
        &request.expected_admission_review_sha256,
        &request.expected_validation_sha256,
        &request.expected_output_sha256,
        &request.expected_snapshot_sha256,
        &request.expected_protocol_sha256,
    )
    .await?
    .ok_or_else(|| "当前没有这条精确绑定且独立准入的历史结果输出".to_string())?;
    let implementation_name =
        bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    let code_revision = bounded_required(&request.code_revision, MAX_REVISION_CHARS, "代码版本")?;
    let record = new_record(
        &admission,
        admin_id,
        implementation_name,
        request.implementation_kind,
        code_revision,
        Utc::now(),
    )?;
    let deduplication_fingerprint = implementation_deduplication_fingerprint(&record)?;
    if read_implementations(state)
        .await?
        .iter()
        .map(implementation_deduplication_fingerprint)
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .any(|existing| existing == &deduplication_fingerprint)
    {
        return Err("完全相同的标签物化实现已经登记；规范变化时请登记新版本".to_string());
    }
    write_immutable_json(
        &implementation_root(state)
            .join(format!("{}.json", record.materialization_implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn new_record(
    admission: &AdmittedHistoricalOutcomeForLabelMaterialization,
    admin_id: &str,
    implementation_name: String,
    implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind,
    code_revision: String,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeLabelMaterializationImplementationRecord, String> {
    let review = &admission.admission_review;
    let validation = &admission.validation;
    let snapshot = &admission.snapshot;
    let mut record = HistoricalOutcomeLabelMaterializationImplementationRecord {
        schema_version: IMPLEMENTATION_SCHEMA_VERSION.to_string(),
        materialization_policy_version: MATERIALIZATION_POLICY_VERSION.to_string(),
        materialization_implementation_id: String::new(),
        materialization_implementation_spec_sha256: String::new(),
        registered_at,
        registered_by: admin_id.to_string(),
        admission_review_id: review.review_id.clone(),
        admission_review_sha256: review.review_sha256.clone(),
        admission_reviewer_id: review.reviewer_id.clone(),
        admission_known_limitations: review.known_limitations.clone(),
        attempt_id: review.attempt_id.clone(),
        claim_sha256: review.claim_sha256.clone(),
        result_id: review.result_id.clone(),
        result_sha256: review.result_sha256.clone(),
        output_sha256: review.output_sha256.clone(),
        validation_id: review.validation_id.clone(),
        validation_sha256: review.validation_sha256.clone(),
        snapshot_id: review.snapshot_id.clone(),
        snapshot_sha256: review.snapshot_sha256.clone(),
        reconstruction_id: review.reconstruction_id.clone(),
        reconstruction_sha256: review.reconstruction_sha256.clone(),
        protocol_version: review.protocol_version.clone(),
        protocol_sha256: review.protocol_sha256.clone(),
        asset_symbol: review.asset_symbol.clone(),
        benchmark_symbol: review.benchmark_symbol.clone(),
        decision_available_at: review.decision_available_at,
        common_session_count: review.common_session_count,
        metric_horizons_market_sessions: review.metric_horizons_market_sessions.clone(),
        metric_start_date: review.metric_start_date,
        metric_end_dates: review.metric_end_dates.clone(),
        recomputed_metrics_sha256: review.recomputed_metrics_sha256.clone(),
        output_label_schema_version: OUTPUT_LABEL_SCHEMA_VERSION.to_string(),
        implementation_name,
        implementation_kind,
        code_revision,
        status: STATUS.to_string(),
        input_contract: INPUT_CONTRACT.to_string(),
        output_contract: OUTPUT_CONTRACT.to_string(),
        output_fields: vec![
            "asset_return".to_string(),
            "benchmark_return".to_string(),
            "excess_return".to_string(),
            "asset_max_drawdown".to_string(),
            "provenance".to_string(),
            "known_limitations".to_string(),
        ],
        deterministic_projection_required: true,
        exact_metric_bit_preservation_required: true,
        create_once_output_required: true,
        isolated_output_required: true,
        known_limitations_preservation_required: true,
        missing_data_fail_closed_required: true,
        manual_metric_override_allowed: false,
        direction_inference_allowed: false,
        rating_inference_allowed: false,
        investment_action_inference_allowed: false,
        position_sizing_inference_allowed: false,
        reward_semantics_inference_allowed: false,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        production_reads_allowed: false,
        production_writes_allowed: false,
        historical_state_mutation_allowed: false,
        label_materialization_run_authorized: false,
        outcome_label_write_allowed: false,
        label_materialization_enabled: false,
        outcome_label_written: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    if validation.validation_sha256 != record.validation_sha256
        || snapshot.snapshot_sha256 != record.snapshot_sha256
    {
        return Err("标签物化实现上游对象绑定不一致".to_string());
    }
    record.materialization_implementation_spec_sha256 = implementation_fingerprint(&record)?;
    record.materialization_implementation_id =
        record.materialization_implementation_spec_sha256[..32].to_string();
    validate_record(&record)?;
    Ok(record)
}

fn validate_record(
    record: &HistoricalOutcomeLabelMaterializationImplementationRecord,
) -> Result<(), String> {
    let inference_is_closed = !record.manual_metric_override_allowed
        && !record.direction_inference_allowed
        && !record.rating_inference_allowed
        && !record.investment_action_inference_allowed
        && !record.position_sizing_inference_allowed
        && !record.reward_semantics_inference_allowed;
    let authority_is_closed = !record.outbound_network_allowed
        && !record.external_tools_allowed
        && !record.production_reads_allowed
        && !record.production_writes_allowed
        && !record.historical_state_mutation_allowed
        && !record.label_materialization_run_authorized
        && !record.outcome_label_write_allowed
        && !record.label_materialization_enabled
        && !record.outcome_label_written
        && !record.training_authorized
        && !record.reward_authorized
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != IMPLEMENTATION_SCHEMA_VERSION
        || record.materialization_policy_version != MATERIALIZATION_POLICY_VERSION
        || !valid_id(&record.materialization_implementation_id)
        || !valid_sha256(&record.materialization_implementation_spec_sha256)
        || record.materialization_implementation_id
            != record.materialization_implementation_spec_sha256[..32]
        || record.materialization_implementation_spec_sha256 != implementation_fingerprint(record)?
        || record.registered_by.trim().is_empty()
        || !valid_id(&record.admission_review_id)
        || !valid_sha256(&record.admission_review_sha256)
        || record.admission_reviewer_id.trim().is_empty()
        || record.admission_known_limitations.trim().is_empty()
        || !valid_id(&record.attempt_id)
        || !valid_sha256(&record.claim_sha256)
        || !valid_id(&record.result_id)
        || !valid_sha256(&record.result_sha256)
        || !valid_sha256(&record.output_sha256)
        || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256)
        || !valid_id(&record.snapshot_id)
        || !valid_sha256(&record.snapshot_sha256)
        || !valid_id(&record.reconstruction_id)
        || !valid_sha256(&record.reconstruction_sha256)
        || record.protocol_version.trim().is_empty()
        || !valid_sha256(&record.protocol_sha256)
        || !valid_symbol(&record.asset_symbol)
        || !valid_symbol(&record.benchmark_symbol)
        || record.common_session_count <= 250
        || record.metric_horizons_market_sessions != vec![20, 60, 250]
        || record.metric_end_dates.len() != 3
        || record
            .metric_end_dates
            .iter()
            .any(|date| *date <= record.metric_start_date)
        || !valid_sha256(&record.recomputed_metrics_sha256)
        || record.output_label_schema_version != OUTPUT_LABEL_SCHEMA_VERSION
        || record.implementation_name.trim().is_empty()
        || record.implementation_name.chars().count() > MAX_NAME_CHARS
        || record.code_revision.trim().is_empty()
        || record.code_revision.chars().count() > MAX_REVISION_CHARS
        || record.status != STATUS
        || record.input_contract != INPUT_CONTRACT
        || record.output_contract != OUTPUT_CONTRACT
        || record.output_fields
            != vec![
                "asset_return",
                "benchmark_return",
                "excess_return",
                "asset_max_drawdown",
                "provenance",
                "known_limitations",
            ]
        || !record.deterministic_projection_required
        || !record.exact_metric_bit_preservation_required
        || !record.create_once_output_required
        || !record.isolated_output_required
        || !record.known_limitations_preservation_required
        || !record.missing_data_fail_closed_required
        || !inference_is_closed
        || !authority_is_closed
    {
        return Err(
            "historical outcome label materialization implementation is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn binding_matches(
    record: &HistoricalOutcomeLabelMaterializationImplementationRecord,
    admission: &AdmittedHistoricalOutcomeForLabelMaterialization,
) -> bool {
    let review = &admission.admission_review;
    record.admission_review_id == review.review_id
        && record.admission_review_sha256 == review.review_sha256
        && record.admission_reviewer_id == review.reviewer_id
        && record.admission_known_limitations == review.known_limitations
        && record.attempt_id == review.attempt_id
        && record.claim_sha256 == review.claim_sha256
        && record.result_id == review.result_id
        && record.result_sha256 == review.result_sha256
        && record.output_sha256 == review.output_sha256
        && record.validation_id == review.validation_id
        && record.validation_sha256 == review.validation_sha256
        && record.snapshot_id == review.snapshot_id
        && record.snapshot_sha256 == review.snapshot_sha256
        && record.reconstruction_id == review.reconstruction_id
        && record.reconstruction_sha256 == review.reconstruction_sha256
        && record.protocol_version == review.protocol_version
        && record.protocol_sha256 == review.protocol_sha256
        && record.asset_symbol == review.asset_symbol
        && record.benchmark_symbol == review.benchmark_symbol
        && record.decision_available_at == review.decision_available_at
        && record.common_session_count == review.common_session_count
        && record.metric_horizons_market_sessions == review.metric_horizons_market_sessions
        && record.metric_start_date == review.metric_start_date
        && record.metric_end_dates == review.metric_end_dates
        && record.recomputed_metrics_sha256 == review.recomputed_metrics_sha256
}

impl From<&AdmittedHistoricalOutcomeForLabelMaterialization>
    for AdmittedHistoricalOutcomeProjection
{
    fn from(value: &AdmittedHistoricalOutcomeForLabelMaterialization) -> Self {
        Self {
            attempt_id: value.admission_review.attempt_id.clone(),
            admission_review_id: value.admission_review.review_id.clone(),
            admission_review_sha256: value.admission_review.review_sha256.clone(),
            validation_id: value.admission_review.validation_id.clone(),
            validation_sha256: value.admission_review.validation_sha256.clone(),
            output_sha256: value.admission_review.output_sha256.clone(),
            snapshot_id: value.admission_review.snapshot_id.clone(),
            snapshot_sha256: value.admission_review.snapshot_sha256.clone(),
            protocol_version: value.admission_review.protocol_version.clone(),
            protocol_sha256: value.admission_review.protocol_sha256.clone(),
            asset_symbol: value.admission_review.asset_symbol.clone(),
            benchmark_symbol: value.admission_review.benchmark_symbol.clone(),
            decision_available_at: value.admission_review.decision_available_at,
            known_limitations: value.admission_review.known_limitations.clone(),
        }
    }
}

fn implementation_fingerprint(
    record: &HistoricalOutcomeLabelMaterializationImplementationRecord,
) -> Result<String, String> {
    let mut value = serde_json::to_value(record).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "label materialization implementation payload is invalid".to_string())?;
    object.remove("materialization_implementation_id");
    object.remove("materialization_implementation_spec_sha256");
    hash_serializable(&value)
}

fn implementation_deduplication_fingerprint(
    record: &HistoricalOutcomeLabelMaterializationImplementationRecord,
) -> Result<String, String> {
    let mut value = serde_json::to_value(record).map_err(|error| error.to_string())?;
    let object = value.as_object_mut().ok_or_else(|| {
        "label materialization implementation deduplication payload is invalid".to_string()
    })?;
    object.remove("materialization_implementation_id");
    object.remove("materialization_implementation_spec_sha256");
    object.remove("registered_at");
    object.remove("registered_by");
    hash_serializable(&value)
}

async fn read_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeLabelMaterializationImplementationRecord>, String> {
    let directory = implementation_root(state);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    while let Some(entry) = entries
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
        let record: HistoricalOutcomeLabelMaterializationImplementationRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&record)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(record.materialization_implementation_id.as_str())
        {
            return Err(
                "historical outcome label materialization implementation filename mismatch"
                    .to_string(),
            );
        }
        if !ids.insert(record.materialization_implementation_id.clone())
            || !hashes.insert(record.materialization_implementation_spec_sha256.clone())
        {
            return Err(
                "historical outcome label materialization implementation registry contains duplicates"
                    .to_string(),
            );
        }
        output.push(record);
    }
    Ok(output)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn implementation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-label-materialization-implementations")
}

struct MaterializationImplementationLock {
    path: PathBuf,
}

impl Drop for MaterializationImplementationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<MaterializationImplementationLock, String> {
    if !valid_id(name) {
        return Err("invalid label materialization implementation lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{name}.lock"));
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
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    options
        .open(&path)
        .await
        .map_err(|_| "已有标签物化实现正在登记".to_string())?;
    Ok(MaterializationImplementationLock { path })
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

fn bounded_required(value: &str, max_chars: usize, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if value.chars().count() > max_chars {
        return Err(format!("{label}不能超过 {max_chars} 个字符"));
    }
    Ok(value.to_string())
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn valid_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 24
        && value.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::historical_outcome_dry_run_execution_attempts::HistoricalOutcomeDryRunMetric;
    use crate::routes::historical_outcome_dry_run_output_validations::{
        HistoricalOutcomeDryRunOutputValidationRecord,
        HistoricalOutcomeDryRunOutputValidationVerdict,
    };
    use crate::routes::historical_outcome_label_admission_reviews::{
        HistoricalOutcomeLabelAdmissionReview, HistoricalOutcomeLabelAdmissionVerdict,
    };
    use crate::routes::historical_outcome_price_snapshots::{
        HistoricalOutcomePriceSnapshot, SealedAdjustedClosePoint,
    };

    fn admitted() -> AdmittedHistoricalOutcomeForLabelMaterialization {
        let metrics = vec![
            metric(20, 2025, 2, 3),
            metric(60, 2025, 4, 1),
            metric(250, 2026, 1, 6),
        ];
        let validation = HistoricalOutcomeDryRunOutputValidationRecord {
            schema_version: "hone-historical-outcome-dry-run-output-validation-v1".to_string(),
            policy_version: "test".to_string(),
            validation_id: "validation-1".to_string(),
            validation_sha256: "a".repeat(64),
            attempt_id: "attempt-1".to_string(),
            claim_sha256: "b".repeat(64),
            result_id: "result-1".to_string(),
            result_sha256: "c".repeat(64),
            output_sha256: "d".repeat(64),
            isolated_runner_id: "runner-1".to_string(),
            isolated_runner_spec_sha256: "e".repeat(64),
            runner_artifact_sha256: "f".repeat(64),
            authorization_review_id: "authorization-1".to_string(),
            authorization_review_sha256: "1".repeat(64),
            snapshot_id: "snapshot-1".to_string(),
            snapshot_sha256: "2".repeat(64),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "3".repeat(64),
            validator_implementation_version: "validator-v1".to_string(),
            validator_implementation_sha256: "4".repeat(64),
            validated_at: Utc::now(),
            validated_by: "validator".to_string(),
            execution_invoked_by: "invoker".to_string(),
            runner_registered_by: "runner-registrant".to_string(),
            first_execution_authorization_reviewer_id: "first-reviewer".to_string(),
            run_authorization_reviewer_id: "run-reviewer".to_string(),
            validator_independent_from_execution_and_prior_reviewers: true,
            immutable_chain_integrity_verified: true,
            current_sealed_snapshot_binding_verified: true,
            canonical_output_hash_verified: true,
            output_structure_verified: true,
            deterministic_recomputation_match: true,
            recomputed_metrics: metrics.clone(),
            mismatch_reasons: Vec::new(),
            verdict: HistoricalOutcomeDryRunOutputValidationVerdict::ValidatedDeterministicMatch,
            output_validated: true,
            outcome_label_admission_authorized: false,
            outcome_label_written: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        let points = (0..=250)
            .map(|day| SealedAdjustedClosePoint {
                date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap() + chrono::Days::new(day),
                adjusted_close: 100.0 + day as f64,
            })
            .collect::<Vec<_>>();
        let snapshot = HistoricalOutcomePriceSnapshot {
            schema_version: "test".to_string(),
            ingestion_policy_version: "test".to_string(),
            snapshot_id: validation.snapshot_id.clone(),
            snapshot_sha256: validation.snapshot_sha256.clone(),
            sealed_at: Utc::now(),
            sealed_by: "admin".to_string(),
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "5".repeat(64),
            reconstruction_review_id: "reconstruction-review-1".to_string(),
            anchor_candidate_id: "anchor-1".to_string(),
            anchor_review_id: "anchor-review-1".to_string(),
            decision_available_at: Utc::now(),
            implementation_id: "labeler-1".to_string(),
            implementation_spec_sha256: "6".repeat(64),
            implementation_review_id: "labeler-review-1".to_string(),
            governance_review_id: "governance-1".to_string(),
            protocol_version: validation.protocol_version.clone(),
            protocol_sha256: validation.protocol_sha256.clone(),
            code_revision: "oldwang@labeler".to_string(),
            provider: "test".to_string(),
            provider_endpoint_template: "test".to_string(),
            price_basis: "adjusted close".to_string(),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            requested_from: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            requested_to: NaiveDate::from_ymd_opt(2026, 1, 7).unwrap(),
            asset_payload_sha256: "7".repeat(64),
            benchmark_payload_sha256: "8".repeat(64),
            asset_series_sha256: "9".repeat(64),
            benchmark_series_sha256: "a".repeat(64),
            asset_points: points.clone(),
            benchmark_points: points,
            common_session_count: 251,
            covered_horizons_market_sessions: vec![20, 60, 250],
            all_protocol_horizons_covered: true,
            outcome_metrics_computed: false,
            label_written: false,
            historical_state_mutated: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        };
        let review = HistoricalOutcomeLabelAdmissionReview {
            schema_version: "hone-historical-outcome-label-admission-review-v1".to_string(),
            policy_version: "policy-v1".to_string(),
            review_id: "admission-review-1".to_string(),
            review_sha256: "3".repeat(64),
            previous_review_id: None,
            previous_review_sha256: None,
            attempt_id: validation.attempt_id.clone(),
            claim_sha256: validation.claim_sha256.clone(),
            result_id: validation.result_id.clone(),
            result_sha256: validation.result_sha256.clone(),
            output_sha256: validation.output_sha256.clone(),
            validation_id: validation.validation_id.clone(),
            validation_sha256: validation.validation_sha256.clone(),
            validator_implementation_version: validation.validator_implementation_version.clone(),
            validator_implementation_sha256: validation.validator_implementation_sha256.clone(),
            validated_by: validation.validated_by.clone(),
            snapshot_id: validation.snapshot_id.clone(),
            snapshot_sha256: validation.snapshot_sha256.clone(),
            reconstruction_id: snapshot.reconstruction_id.clone(),
            reconstruction_sha256: snapshot.reconstruction_sha256.clone(),
            protocol_version: validation.protocol_version.clone(),
            protocol_sha256: validation.protocol_sha256.clone(),
            asset_symbol: snapshot.asset_symbol.clone(),
            benchmark_symbol: snapshot.benchmark_symbol.clone(),
            decision_available_at: snapshot.decision_available_at,
            common_session_count: snapshot.common_session_count,
            metric_horizons_market_sessions: vec![20, 60, 250],
            metric_start_date: metrics[0].start_date,
            metric_end_dates: metrics.iter().map(|value| value.end_date).collect(),
            recomputed_metrics_sha256: hash_serializable(&metrics).unwrap(),
            submitted_at: Utc::now(),
            reviewer_id: "admission-reviewer".to_string(),
            verdict: HistoricalOutcomeLabelAdmissionVerdict::ApprovedForFutureLabelMaterialization,
            rationale: "适用性和偏差边界已审阅".to_string(),
            known_limitations: "只观察市场结果，不表示因果或正确动作".to_string(),
            reviewer_independent_from_validation_and_execution_chain: true,
            exact_validation_current_binding_confirmed: true,
            frozen_protocol_applicability_confirmed: true,
            complete_horizons_and_common_session_endpoints_confirmed: true,
            adjusted_close_and_corporate_action_basis_confirmed: true,
            benchmark_comparability_confirmed: true,
            event_time_and_future_isolation_confirmed: true,
            missingness_and_survivorship_bias_reviewed: true,
            no_manual_metric_override_confirmed: true,
            label_semantics_and_direction_not_inferred_confirmed: true,
            downstream_authority_remains_closed_confirmed: true,
            outcome_label_input_admitted: true,
            future_label_materialization_eligible: true,
            outcome_label_written: false,
            label_materialization_started: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        AdmittedHistoricalOutcomeForLabelMaterialization {
            admission_review: review,
            validation,
            snapshot,
        }
    }

    fn metric(horizon: u16, year: i32, month: u32, day: u32) -> HistoricalOutcomeDryRunMetric {
        HistoricalOutcomeDryRunMetric {
            horizon_market_sessions: horizon,
            start_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, month, day).unwrap(),
            asset_return: 0.1,
            benchmark_return: 0.04,
            excess_return: 0.06,
            asset_max_drawdown: -0.08,
        }
    }

    fn record() -> HistoricalOutcomeLabelMaterializationImplementationRecord {
        new_record(
            &admitted(),
            "materializer-registrant",
            "原始已验证结果信封物化器".to_string(),
            HistoricalOutcomeLabelMaterializationImplementationKind::DeterministicRawValidatedOutcomeEnvelope,
            "oldwang@materializer123".to_string(),
            Utc::now(),
        )
        .unwrap()
    }

    #[test]
    fn registered_materializer_is_non_running_and_semantically_narrow() {
        let value = record();
        assert_eq!(value.status, STATUS);
        assert!(value.deterministic_projection_required);
        assert!(value.exact_metric_bit_preservation_required);
        assert!(value.known_limitations_preservation_required);
        assert!(!value.label_materialization_run_authorized);
        assert!(!value.outcome_label_write_allowed);
        assert!(!value.label_materialization_enabled);
        assert!(!value.outcome_label_written);
        assert!(!value.direction_inference_allowed);
        assert!(!value.investment_action_inference_allowed);
        assert!(!value.reward_semantics_inference_allowed);
        assert!(validate_record(&value).is_ok());
    }

    #[test]
    fn exact_admission_binding_is_required() {
        let admission = admitted();
        let value = new_record(
            &admission,
            "materializer-registrant",
            "原始已验证结果信封物化器".to_string(),
            HistoricalOutcomeLabelMaterializationImplementationKind::DeterministicRawValidatedOutcomeEnvelope,
            "oldwang@materializer123".to_string(),
            Utc::now(),
        )
        .unwrap();
        assert!(binding_matches(&value, &admission));
        let mut changed = admission;
        changed.admission_review.review_sha256 = "9".repeat(64);
        assert!(!binding_matches(&value, &changed));
    }

    #[test]
    fn upstream_tampering_invalidates_fingerprint() {
        let mut value = record();
        value.output_sha256 = "9".repeat(64);
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn metric_override_or_semantic_inference_is_rejected() {
        let mut value = record();
        value.manual_metric_override_allowed = true;
        assert!(validate_record(&value).is_err());
        let mut value = record();
        value.direction_inference_allowed = true;
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn label_write_and_downstream_authority_are_rejected() {
        let mut value = record();
        value.outcome_label_write_allowed = true;
        assert!(validate_record(&value).is_err());
        let mut value = record();
        value.training_authorized = true;
        assert!(validate_record(&value).is_err());
        let mut value = record();
        value.trading_authorized = true;
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn deduplication_ignores_actor_and_time_but_not_code_revision() {
        let left = record();
        let mut right = left.clone();
        right.registered_at = right.registered_at + chrono::Duration::seconds(1);
        right.registered_by = "another-registrant".to_string();
        right.materialization_implementation_spec_sha256 =
            implementation_fingerprint(&right).unwrap();
        right.materialization_implementation_id =
            right.materialization_implementation_spec_sha256[..32].to_string();
        assert_eq!(
            implementation_deduplication_fingerprint(&left).unwrap(),
            implementation_deduplication_fingerprint(&right).unwrap()
        );
        right.code_revision = "oldwang@materializer456".to_string();
        assert_ne!(
            implementation_deduplication_fingerprint(&left).unwrap(),
            implementation_deduplication_fingerprint(&right).unwrap()
        );
    }
}
