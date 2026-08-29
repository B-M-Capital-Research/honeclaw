//! Immutable registration and human review for a future historical outcome labeler.
//!
//! A record in this module is a frozen, non-running specification.  It may not
//! fetch prices, emit labels, mutate historical states, train a model or open
//! any portfolio/trading authority.  Even an approved review only makes the
//! exact specification eligible for a later offline dry-run authorization
//! review.

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
use uuid::Uuid;

use super::historical_outcome_governance::{
    HistoricalOutcomeReadinessSummary, historical_outcome_readiness_summary,
};
use super::historical_state_reconstructions::outcome_protocol;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-labeler-registry-v1";
const IMPLEMENTATION_SCHEMA_VERSION: &str = "hone-historical-outcome-labeler-implementation-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-historical-outcome-labeler-review-v1";
const SANDBOX_POLICY_VERSION: &str = "hone-historical-outcome-labeler-sandbox-v1";
const INPUT_CONTRACT: &str = "只读输入：人工批准且绑定仍有效的历史点时状态，以及在独立摄取阶段封存、带来源与截止时间的 FMP 复权收盘价快照；实现自身不得联网。";
const OUTPUT_CONTRACT: &str = "未来离线试运行只能在隔离输出中计算 20/60/250 个共同交易日的标的收益、SPY 收益、超额收益和最大回撤；当前登记与复核不创建任何结果。";
const MAX_NAME_CHARS: usize = 120;
const MAX_RATIONALE_CHARS: usize = 1_600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeLabelerImplementationKind {
    DeterministicCommonSessionAdjustedClose,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeLabelerReviewVerdict {
    ApprovedForOfflineDryRunAuthorizationReview,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeLabelerRequest {
    expected_governance_review_id: String,
    protocol_version: String,
    protocol_sha256: String,
    implementation_name: String,
    implementation_kind: HistoricalOutcomeLabelerImplementationKind,
    code_revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelerImplementationRecord {
    schema_version: String,
    implementation_id: String,
    implementation_spec_sha256: String,
    registered_at: DateTime<Utc>,
    registered_by: String,
    governance_review_id: String,
    protocol_version: String,
    protocol_sha256: String,
    sandbox_policy_version: String,
    implementation_name: String,
    implementation_kind: HistoricalOutcomeLabelerImplementationKind,
    code_revision: String,
    status: String,
    input_contract: String,
    output_contract: String,
    price_snapshot_source: String,
    price_basis: String,
    benchmark_symbol: String,
    horizons_market_sessions: Vec<u16>,
    metrics: Vec<String>,
    common_session_rule_required: bool,
    deterministic_replay_required: bool,
    future_information_isolation_required: bool,
    missing_data_fail_closed_required: bool,
    max_parallel_series: u8,
    outbound_network_allowed: bool,
    external_tools_allowed: bool,
    production_writes_allowed: bool,
    historical_state_mutation_allowed: bool,
    label_writes_allowed: bool,
    run_authorized: bool,
    outcome_label_generation_enabled: bool,
    decision_training_authorized: bool,
    reward_evidence_authorized: bool,
    shadow_evidence_authorized: bool,
    trading_authorized: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeLabelerRequest {
    expected_review_id: Option<String>,
    verdict: HistoricalOutcomeLabelerReviewVerdict,
    rationale: String,
    implementation_fingerprint_confirmed: bool,
    protocol_binding_confirmed: bool,
    adjusted_close_and_common_sessions_confirmed: bool,
    deterministic_replay_confirmed: bool,
    future_isolation_confirmed: bool,
    missing_data_fail_closed_confirmed: bool,
    no_network_or_production_writes_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelerReview {
    schema_version: String,
    review_id: String,
    previous_review_id: Option<String>,
    implementation_id: String,
    implementation_spec_sha256: String,
    governance_review_id: String,
    protocol_version: String,
    protocol_sha256: String,
    submitted_at: DateTime<Utc>,
    reviewer_id: String,
    verdict: HistoricalOutcomeLabelerReviewVerdict,
    rationale: String,
    implementation_fingerprint_confirmed: bool,
    protocol_binding_confirmed: bool,
    adjusted_close_and_common_sessions_confirmed: bool,
    deterministic_replay_confirmed: bool,
    future_isolation_confirmed: bool,
    missing_data_fail_closed_confirmed: bool,
    no_network_or_production_writes_confirmed: bool,
    offline_dry_run_authorization_review_eligible: bool,
    offline_dry_run_enabled: bool,
    outcome_label_generation_enabled: bool,
    decision_training_authorized: bool,
    reward_evidence_authorized: bool,
    shadow_evidence_authorized: bool,
    trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelerRegistryItem {
    implementation: HistoricalOutcomeLabelerImplementationRecord,
    latest_review: Option<HistoricalOutcomeLabelerReview>,
    governance_binding_current: bool,
    offline_dry_run_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelerRegistry {
    schema_version: String,
    sandbox_policy_version: String,
    protocol_version: String,
    protocol_sha256: String,
    current_governance_review_id: Option<String>,
    registration_allowed: bool,
    allowed_implementation_kinds: Vec<HistoricalOutcomeLabelerImplementationKind>,
    implementations: Vec<HistoricalOutcomeLabelerRegistryItem>,
    current_binding_implementation_count: usize,
    reviewed_implementation_count: usize,
    labeler_review_status: String,
    offline_dry_run_enabled: bool,
    outcome_label_generation_enabled: bool,
    decision_training_authorized: bool,
    reward_evidence_authorized: bool,
    shadow_evidence_authorized: bool,
    trading_authorized: bool,
    scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeLabelerReadinessSummary {
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub reviewed_implementation_count: usize,
    pub labeler_review_status: String,
    pub offline_dry_run_authorization_review_eligible: bool,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeLabeler {
    pub implementation_id: String,
    pub implementation_spec_sha256: String,
    pub implementation_review_id: String,
    pub governance_review_id: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub code_revision: String,
}

pub(crate) async fn handle_get_historical_outcome_labelers(
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
            warn!(%error, "historical outcome labeler registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果标签器注册表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_labeler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeLabelerRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match register_implementation(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome labeler registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果标签器注册表暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn handle_review_historical_outcome_labeler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeLabelerRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_implementation(&state, &admin.user_id, &implementation_id, request).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_labeler_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelerReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeLabelerReadinessSummary {
        implementation_count: registry.implementations.len(),
        current_binding_implementation_count: registry.current_binding_implementation_count,
        reviewed_implementation_count: registry.reviewed_implementation_count,
        labeler_review_status: registry.labeler_review_status,
        offline_dry_run_authorization_review_eligible: registry
            .implementations
            .iter()
            .any(|item| item.offline_dry_run_authorization_review_eligible),
        offline_dry_run_enabled: registry.offline_dry_run_enabled,
        outcome_label_generation_enabled: registry.outcome_label_generation_enabled,
    })
}

/// Returns only exact implementations whose protocol binding and latest
/// independent review are both current.  Price sealing and later dry-run
/// authorization use this server-side projection to reject stale client data.
pub(crate) async fn approved_historical_outcome_labelers(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeLabeler>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .implementations
        .into_iter()
        .filter(|item| item.offline_dry_run_authorization_review_eligible)
        .filter_map(|item| {
            item.latest_review
                .map(|review| ApprovedHistoricalOutcomeLabeler {
                    implementation_id: item.implementation.implementation_id,
                    implementation_spec_sha256: item.implementation.implementation_spec_sha256,
                    implementation_review_id: review.review_id,
                    governance_review_id: item.implementation.governance_review_id,
                    protocol_version: item.implementation.protocol_version,
                    protocol_sha256: item.implementation.protocol_sha256,
                    code_revision: item.implementation.code_revision,
                })
        })
        .collect())
}

async fn build_registry(state: &AppState) -> Result<HistoricalOutcomeLabelerRegistry, String> {
    let governance = historical_outcome_readiness_summary(state).await?;
    let mut records = read_implementations(state).await?;
    let mut items = Vec::with_capacity(records.len());
    for record in records.drain(..) {
        let latest_review = latest_review(state, &record).await?;
        let governance_binding_current = implementation_binding_current(&record, &governance);
        let offline_dry_run_authorization_review_eligible = governance_binding_current
            && latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    == HistoricalOutcomeLabelerReviewVerdict::ApprovedForOfflineDryRunAuthorizationReview
                    && review.implementation_spec_sha256 == record.implementation_spec_sha256
                    && review.offline_dry_run_authorization_review_eligible
            });
        items.push(HistoricalOutcomeLabelerRegistryItem {
            implementation: record,
            latest_review,
            governance_binding_current,
            offline_dry_run_authorization_review_eligible,
        });
    }
    let current_binding_implementation_count = items
        .iter()
        .filter(|item| item.governance_binding_current)
        .count();
    let reviewed_implementation_count = items
        .iter()
        .filter(|item| item.offline_dry_run_authorization_review_eligible)
        .count();
    let labeler_review_status = if !governance.labeler_implementation_registration_eligible {
        "waiting_for_governance_approval"
    } else if current_binding_implementation_count == 0 {
        "waiting_for_implementation_registration"
    } else if reviewed_implementation_count == 0 {
        "waiting_for_implementation_review"
    } else {
        "offline_dry_run_authorization_review_eligible"
    };
    Ok(HistoricalOutcomeLabelerRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        sandbox_policy_version: SANDBOX_POLICY_VERSION.to_string(),
        protocol_version: governance.protocol_version,
        protocol_sha256: governance.protocol_sha256,
        current_governance_review_id: governance.latest_review_id,
        registration_allowed: governance.labeler_implementation_registration_eligible,
        allowed_implementation_kinds: vec![
            HistoricalOutcomeLabelerImplementationKind::DeterministicCommonSessionAdjustedClose,
        ],
        implementations: items,
        current_binding_implementation_count,
        reviewed_implementation_count,
        labeler_review_status: labeler_review_status.to_string(),
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
        scope: "这里只登记并人工复核确定性历史结果标签器的冻结规范。登记或批准都不会读取行情、运行实现、生成标签、改写历史状态，也不授权训练、奖励、影子或交易；批准后仅可进入独立的离线试运行授权复核。".to_string(),
    })
}

fn implementation_binding_current(
    record: &HistoricalOutcomeLabelerImplementationRecord,
    governance: &HistoricalOutcomeReadinessSummary,
) -> bool {
    governance.labeler_implementation_registration_eligible
        && governance.latest_review_id.as_deref() == Some(record.governance_review_id.as_str())
        && governance.protocol_version == record.protocol_version
        && governance.protocol_sha256 == record.protocol_sha256
}

async fn register_implementation(
    state: &AppState,
    registered_by: &str,
    request: RegisterHistoricalOutcomeLabelerRequest,
) -> Result<HistoricalOutcomeLabelerImplementationRecord, String> {
    let _lock = acquire_lock(state).await?;
    let governance = historical_outcome_readiness_summary(state).await?;
    validate_registration_request(&request, &governance)?;
    let name = bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    if name.chars().any(char::is_control) {
        return Err("实现名称不能包含控制字符".to_string());
    }
    let code_revision = valid_code_revision(&request.code_revision)?;
    let protocol = outcome_protocol();
    let registered_at = Utc::now();
    let mut record = HistoricalOutcomeLabelerImplementationRecord {
        schema_version: IMPLEMENTATION_SCHEMA_VERSION.to_string(),
        implementation_id: format!(
            "historical-outcome-labeler-{}-{}",
            registered_at.timestamp_millis(),
            Uuid::new_v4().simple()
        ),
        implementation_spec_sha256: String::new(),
        registered_at,
        registered_by: registered_by.to_string(),
        governance_review_id: request.expected_governance_review_id,
        protocol_version: request.protocol_version,
        protocol_sha256: request.protocol_sha256,
        sandbox_policy_version: SANDBOX_POLICY_VERSION.to_string(),
        implementation_name: name,
        implementation_kind: request.implementation_kind,
        code_revision,
        status: "registered_not_started".to_string(),
        input_contract: INPUT_CONTRACT.to_string(),
        output_contract: OUTPUT_CONTRACT.to_string(),
        price_snapshot_source: "sealed FMP adjusted-close snapshot".to_string(),
        price_basis: protocol.asset_price_basis,
        benchmark_symbol: protocol.benchmark_symbol,
        horizons_market_sessions: protocol.horizons_market_sessions,
        metrics: protocol.metrics,
        common_session_rule_required: true,
        deterministic_replay_required: true,
        future_information_isolation_required: true,
        missing_data_fail_closed_required: true,
        max_parallel_series: 4,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        production_writes_allowed: false,
        historical_state_mutation_allowed: false,
        label_writes_allowed: false,
        run_authorized: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
    };
    record.implementation_spec_sha256 = implementation_fingerprint(&record)?;
    validate_implementation(&record)?;
    if read_implementations(state)
        .await?
        .iter()
        .any(|current| current.implementation_spec_sha256 == record.implementation_spec_sha256)
    {
        return Err("完全相同的标签器实现规范已经登记".to_string());
    }
    write_immutable_json(
        &implementation_directory(state).join(format!("{}.json", record.implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn validate_registration_request(
    request: &RegisterHistoricalOutcomeLabelerRequest,
    governance: &HistoricalOutcomeReadinessSummary,
) -> Result<(), String> {
    if !governance.labeler_implementation_registration_eligible {
        return Err("当前历史结果协议尚未获准登记标签器实现".to_string());
    }
    if governance.latest_review_id.as_deref()
        != Some(request.expected_governance_review_id.as_str())
    {
        return Err("历史结果协议复核已经变化，请刷新后重试".to_string());
    }
    if request.protocol_version != governance.protocol_version
        || request.protocol_sha256 != governance.protocol_sha256
    {
        return Err("历史结果协议版本或指纹已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

async fn review_implementation(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewHistoricalOutcomeLabelerRequest,
) -> Result<HistoricalOutcomeLabelerReview, String> {
    if !valid_id(implementation_id) {
        return Err("标签器实现编号无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let governance = historical_outcome_readiness_summary(state).await?;
    let record = read_implementations(state)
        .await?
        .into_iter()
        .find(|record| record.implementation_id == implementation_id)
        .ok_or_else(|| "标签器实现不存在".to_string())?;
    let previous = latest_review(state, &record).await?;
    if request.expected_review_id.as_deref()
        != previous.as_ref().map(|review| review.review_id.as_str())
    {
        return Err("标签器实现复核已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let approved = request.verdict
        == HistoricalOutcomeLabelerReviewVerdict::ApprovedForOfflineDryRunAuthorizationReview;
    let all_checks = review_request_all_checks(&request);
    if approved && !implementation_binding_current(&record, &governance) {
        return Err("标签器实现与当前协议审批绑定失效，不能批准".to_string());
    }
    if approved && !all_checks {
        return Err("批准标签器实现前必须完成全部指纹、口径、隔离与失败关闭检查".to_string());
    }
    let review = HistoricalOutcomeLabelerReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        review_id: Uuid::new_v4().simple().to_string(),
        previous_review_id: previous.map(|review| review.review_id),
        implementation_id: record.implementation_id.clone(),
        implementation_spec_sha256: record.implementation_spec_sha256.clone(),
        governance_review_id: record.governance_review_id.clone(),
        protocol_version: record.protocol_version.clone(),
        protocol_sha256: record.protocol_sha256.clone(),
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        verdict: request.verdict,
        rationale,
        implementation_fingerprint_confirmed: request.implementation_fingerprint_confirmed,
        protocol_binding_confirmed: request.protocol_binding_confirmed,
        adjusted_close_and_common_sessions_confirmed: request
            .adjusted_close_and_common_sessions_confirmed,
        deterministic_replay_confirmed: request.deterministic_replay_confirmed,
        future_isolation_confirmed: request.future_isolation_confirmed,
        missing_data_fail_closed_confirmed: request.missing_data_fail_closed_confirmed,
        no_network_or_production_writes_confirmed: request
            .no_network_or_production_writes_confirmed,
        offline_dry_run_authorization_review_eligible: approved,
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
    };
    validate_review(&review, &record)?;
    write_immutable_json(
        &review_directory(state, implementation_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn review_request_all_checks(request: &ReviewHistoricalOutcomeLabelerRequest) -> bool {
    request.implementation_fingerprint_confirmed
        && request.protocol_binding_confirmed
        && request.adjusted_close_and_common_sessions_confirmed
        && request.deterministic_replay_confirmed
        && request.future_isolation_confirmed
        && request.missing_data_fail_closed_confirmed
        && request.no_network_or_production_writes_confirmed
}

fn implementation_fingerprint(
    record: &HistoricalOutcomeLabelerImplementationRecord,
) -> Result<String, String> {
    let bytes = serde_json::to_vec(&(
        (
            &record.governance_review_id,
            &record.protocol_version,
            &record.protocol_sha256,
            &record.sandbox_policy_version,
            &record.implementation_name,
            &record.implementation_kind,
            &record.code_revision,
            &record.status,
        ),
        (
            &record.input_contract,
            &record.output_contract,
            &record.price_snapshot_source,
            &record.price_basis,
            &record.benchmark_symbol,
            &record.horizons_market_sessions,
            &record.metrics,
        ),
        (
            record.common_session_rule_required,
            record.deterministic_replay_required,
            record.future_information_isolation_required,
            record.missing_data_fail_closed_required,
            record.max_parallel_series,
        ),
        (
            record.outbound_network_allowed,
            record.external_tools_allowed,
            record.production_writes_allowed,
            record.historical_state_mutation_allowed,
            record.label_writes_allowed,
            record.run_authorized,
            record.outcome_label_generation_enabled,
            record.decision_training_authorized,
            record.reward_evidence_authorized,
            record.shadow_evidence_authorized,
            record.trading_authorized,
        ),
    ))
    .map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn validate_implementation(
    record: &HistoricalOutcomeLabelerImplementationRecord,
) -> Result<(), String> {
    let protocol = outcome_protocol();
    if record.schema_version != IMPLEMENTATION_SCHEMA_VERSION
        || !valid_id(&record.implementation_id)
        || !valid_sha256(&record.implementation_spec_sha256)
        || record.implementation_spec_sha256 != implementation_fingerprint(record)?
        || !valid_id(&record.governance_review_id)
        || record.protocol_version.trim().is_empty()
        || !valid_sha256(&record.protocol_sha256)
        || record.sandbox_policy_version != SANDBOX_POLICY_VERSION
        || record.registered_by.trim().is_empty()
        || record.implementation_name.trim().is_empty()
        || record.implementation_name.chars().count() > MAX_NAME_CHARS
        || record.implementation_name.chars().any(char::is_control)
        || !valid_code_revision_value(&record.code_revision)
        || record.status != "registered_not_started"
        || record.input_contract != INPUT_CONTRACT
        || record.output_contract != OUTPUT_CONTRACT
        || record.price_snapshot_source != "sealed FMP adjusted-close snapshot"
        || record.price_basis != protocol.asset_price_basis
        || record.benchmark_symbol != protocol.benchmark_symbol
        || record.horizons_market_sessions != protocol.horizons_market_sessions
        || record.metrics != protocol.metrics
        || !record.common_session_rule_required
        || !record.deterministic_replay_required
        || !record.future_information_isolation_required
        || !record.missing_data_fail_closed_required
        || record.max_parallel_series != 4
        || record.outbound_network_allowed
        || record.external_tools_allowed
        || record.production_writes_allowed
        || record.historical_state_mutation_allowed
        || record.label_writes_allowed
        || record.run_authorized
        || record.outcome_label_generation_enabled
        || record.decision_training_authorized
        || record.reward_evidence_authorized
        || record.shadow_evidence_authorized
        || record.trading_authorized
    {
        return Err("historical outcome labeler record is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn validate_review(
    review: &HistoricalOutcomeLabelerReview,
    record: &HistoricalOutcomeLabelerImplementationRecord,
) -> Result<(), String> {
    let approved = review.verdict
        == HistoricalOutcomeLabelerReviewVerdict::ApprovedForOfflineDryRunAuthorizationReview;
    let all_checks = review.implementation_fingerprint_confirmed
        && review.protocol_binding_confirmed
        && review.adjusted_close_and_common_sessions_confirmed
        && review.deterministic_replay_confirmed
        && review.future_isolation_confirmed
        && review.missing_data_fail_closed_confirmed
        && review.no_network_or_production_writes_confirmed;
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || !valid_id(&review.review_id)
        || review.implementation_id != record.implementation_id
        || review.implementation_spec_sha256 != record.implementation_spec_sha256
        || review.governance_review_id != record.governance_review_id
        || review.protocol_version != record.protocol_version
        || review.protocol_sha256 != record.protocol_sha256
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || approved != (all_checks && review.offline_dry_run_authorization_review_eligible)
        || review.offline_dry_run_enabled
        || review.outcome_label_generation_enabled
        || review.decision_training_authorized
        || review.reward_evidence_authorized
        || review.shadow_evidence_authorized
        || review.trading_authorized
    {
        return Err("historical outcome labeler review is invalid or over-authorized".to_string());
    }
    Ok(())
}

async fn read_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeLabelerImplementationRecord>, String> {
    let directory = implementation_directory(state);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut records = Vec::new();
    let mut fingerprints = BTreeSet::new();
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
        let record = serde_json::from_slice::<HistoricalOutcomeLabelerImplementationRecord>(&bytes)
            .map_err(|error| error.to_string())?;
        validate_implementation(&record)?;
        if !fingerprints.insert(record.implementation_spec_sha256.clone()) {
            return Err(
                "historical outcome labeler registry has duplicate specifications".to_string(),
            );
        }
        records.push(record);
    }
    records.sort_by(|left, right| {
        right
            .registered_at
            .cmp(&left.registered_at)
            .then_with(|| right.implementation_id.cmp(&left.implementation_id))
    });
    Ok(records)
}

async fn latest_review(
    state: &AppState,
    record: &HistoricalOutcomeLabelerImplementationRecord,
) -> Result<Option<HistoricalOutcomeLabelerReview>, String> {
    let reviews = read_reviews(state, record).await?;
    Ok(review_chain_tip(&reviews)?.cloned())
}

fn review_chain_tip(
    reviews: &[HistoricalOutcomeLabelerReview],
) -> Result<Option<&HistoricalOutcomeLabelerReview>, String> {
    if reviews.is_empty() {
        return Ok(None);
    }
    let by_id = reviews
        .iter()
        .map(|review| (review.review_id.clone(), review))
        .collect::<HashMap<_, _>>();
    let tips = reviews
        .iter()
        .filter(|review| {
            !reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(&review.review_id))
        })
        .collect::<Vec<_>>();
    if tips.len() != 1 {
        return Err("historical outcome labeler review chain has invalid tips".to_string());
    }
    let tip = tips[0];
    let mut cursor = Some(tip);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("historical outcome labeler review chain has a cycle".to_string());
        }
        cursor = match review.previous_review_id.as_ref() {
            Some(id) => Some(*by_id.get(id).ok_or_else(|| {
                "historical outcome labeler review chain is disconnected".to_string()
            })?),
            None => None,
        };
    }
    if visited.len() != reviews.len() {
        return Err("historical outcome labeler review chain is disconnected".to_string());
    }
    Ok(Some(tip))
}

async fn read_reviews(
    state: &AppState,
    record: &HistoricalOutcomeLabelerImplementationRecord,
) -> Result<Vec<HistoricalOutcomeLabelerReview>, String> {
    let directory = review_directory(state, &record.implementation_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
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
        let review = serde_json::from_slice::<HistoricalOutcomeLabelerReview>(&bytes)
            .map_err(|error| error.to_string())?;
        validate_review(&review, record)?;
        reviews.push(review);
    }
    Ok(reviews)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn implementation_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-labeler-implementations")
}

fn review_directory(state: &AppState, implementation_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-labeler-reviews")
        .join(implementation_id)
}

struct HistoricalOutcomeLabelerLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeLabelerLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<HistoricalOutcomeLabelerLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("historical-outcome-labeler-registry.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(HistoricalOutcomeLabelerLock { path }),
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
                return Err("历史结果标签器正在由另一请求变更，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("历史结果标签器锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid historical outcome labeler audit path".to_string())?;
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
    use tokio::io::AsyncWriteExt;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

fn bounded_required(value: &str, max: usize, label: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    if trimmed.chars().count() > max {
        return Err(format!("{label}无效或过长"));
    }
    Ok(trimmed.to_string())
}

fn valid_code_revision(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if !valid_code_revision_value(trimmed) {
        return Err("代码版本只能使用字母、数字和 . - _ / : @".to_string());
    }
    Ok(trimmed.to_string())
}

fn valid_code_revision_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'/' | b':' | b'@')
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

    fn governance(eligible: bool) -> HistoricalOutcomeReadinessSummary {
        HistoricalOutcomeReadinessSummary {
            protocol_version: outcome_protocol().protocol_version,
            protocol_sha256: "a".repeat(64),
            latest_review_id: Some("governance-review-1".to_string()),
            protocol_review_status: if eligible {
                "implementation_review_registration_eligible".to_string()
            } else {
                "waiting_for_protocol_review".to_string()
            },
            labeler_implementation_registration_eligible: eligible,
            outcome_label_generation_enabled: false,
        }
    }

    fn implementation() -> HistoricalOutcomeLabelerImplementationRecord {
        let protocol = outcome_protocol();
        let mut record = HistoricalOutcomeLabelerImplementationRecord {
            schema_version: IMPLEMENTATION_SCHEMA_VERSION.to_string(),
            implementation_id: "historical-outcome-labeler-test".to_string(),
            implementation_spec_sha256: String::new(),
            registered_at: Utc::now(),
            registered_by: "admin".to_string(),
            governance_review_id: "governance-review-1".to_string(),
            protocol_version: protocol.protocol_version,
            protocol_sha256: "a".repeat(64),
            sandbox_policy_version: SANDBOX_POLICY_VERSION.to_string(),
            implementation_name: "共同交易日复权收盘价确定性标签器".to_string(),
            implementation_kind:
                HistoricalOutcomeLabelerImplementationKind::DeterministicCommonSessionAdjustedClose,
            code_revision: "oldwang@abc123".to_string(),
            status: "registered_not_started".to_string(),
            input_contract: INPUT_CONTRACT.to_string(),
            output_contract: OUTPUT_CONTRACT.to_string(),
            price_snapshot_source: "sealed FMP adjusted-close snapshot".to_string(),
            price_basis: protocol.asset_price_basis,
            benchmark_symbol: protocol.benchmark_symbol,
            horizons_market_sessions: protocol.horizons_market_sessions,
            metrics: protocol.metrics,
            common_session_rule_required: true,
            deterministic_replay_required: true,
            future_information_isolation_required: true,
            missing_data_fail_closed_required: true,
            max_parallel_series: 4,
            outbound_network_allowed: false,
            external_tools_allowed: false,
            production_writes_allowed: false,
            historical_state_mutation_allowed: false,
            label_writes_allowed: false,
            run_authorized: false,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        };
        record.implementation_spec_sha256 = implementation_fingerprint(&record).unwrap();
        record
    }

    fn approved_review(
        record: &HistoricalOutcomeLabelerImplementationRecord,
    ) -> HistoricalOutcomeLabelerReview {
        HistoricalOutcomeLabelerReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            review_id: "labeler-review-1".to_string(),
            previous_review_id: None,
            implementation_id: record.implementation_id.clone(),
            implementation_spec_sha256: record.implementation_spec_sha256.clone(),
            governance_review_id: record.governance_review_id.clone(),
            protocol_version: record.protocol_version.clone(),
            protocol_sha256: record.protocol_sha256.clone(),
            submitted_at: Utc::now(),
            reviewer_id: "admin".to_string(),
            verdict:
                HistoricalOutcomeLabelerReviewVerdict::ApprovedForOfflineDryRunAuthorizationReview,
            rationale: "逐项复核确定性、未来隔离和失败关闭边界。".to_string(),
            implementation_fingerprint_confirmed: true,
            protocol_binding_confirmed: true,
            adjusted_close_and_common_sessions_confirmed: true,
            deterministic_replay_confirmed: true,
            future_isolation_confirmed: true,
            missing_data_fail_closed_confirmed: true,
            no_network_or_production_writes_confirmed: true,
            offline_dry_run_authorization_review_eligible: true,
            offline_dry_run_enabled: false,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        }
    }

    #[test]
    fn registration_requires_current_governance_binding() {
        let mut request = RegisterHistoricalOutcomeLabelerRequest {
            expected_governance_review_id: "governance-review-1".to_string(),
            protocol_version: outcome_protocol().protocol_version,
            protocol_sha256: "a".repeat(64),
            implementation_name: "标签器".to_string(),
            implementation_kind:
                HistoricalOutcomeLabelerImplementationKind::DeterministicCommonSessionAdjustedClose,
            code_revision: "oldwang@abc123".to_string(),
        };
        assert!(validate_registration_request(&request, &governance(false)).is_err());
        assert!(validate_registration_request(&request, &governance(true)).is_ok());
        request.expected_governance_review_id = "stale-review".to_string();
        assert!(validate_registration_request(&request, &governance(true)).is_err());
    }

    #[test]
    fn implementation_fingerprint_binds_contract_and_all_authorities_stay_closed() {
        let record = implementation();
        assert!(validate_implementation(&record).is_ok());
        let first = implementation_fingerprint(&record).unwrap();
        let mut unsafe_record = record;
        unsafe_record.label_writes_allowed = true;
        unsafe_record.implementation_spec_sha256 =
            implementation_fingerprint(&unsafe_record).unwrap();
        assert_ne!(first, unsafe_record.implementation_spec_sha256);
        assert!(validate_implementation(&unsafe_record).is_err());
    }

    #[test]
    fn approval_only_opens_a_later_dry_run_authorization_review() {
        let record = implementation();
        let review = approved_review(&record);
        assert!(validate_review(&review, &record).is_ok());
        assert!(review.offline_dry_run_authorization_review_eligible);
        assert!(!review.offline_dry_run_enabled);
        assert!(!review.outcome_label_generation_enabled);
        assert!(!review.decision_training_authorized);
        assert!(!review.reward_evidence_authorized);
        assert!(!review.shadow_evidence_authorized);
        assert!(!review.trading_authorized);
    }

    #[test]
    fn approval_rejects_missing_review_check_or_runtime_authority() {
        let record = implementation();
        let mut incomplete = approved_review(&record);
        incomplete.future_isolation_confirmed = false;
        assert!(validate_review(&incomplete, &record).is_err());
        let mut unsafe_review = approved_review(&record);
        unsafe_review.offline_dry_run_enabled = true;
        assert!(validate_review(&unsafe_review, &record).is_err());
    }

    #[test]
    fn review_audit_chain_rejects_branches_and_cycles() {
        let record = implementation();
        let first = approved_review(&record);
        let mut second = approved_review(&record);
        second.review_id = "labeler-review-2".to_string();
        second.previous_review_id = Some(first.review_id.clone());
        let mut branch = approved_review(&record);
        branch.review_id = "labeler-review-3".to_string();
        branch.previous_review_id = Some(first.review_id.clone());
        assert!(review_chain_tip(&[first.clone(), second, branch]).is_err());

        let mut cycle_first = first;
        cycle_first.previous_review_id = Some("labeler-review-2".to_string());
        let mut cycle_second = approved_review(&record);
        cycle_second.review_id = "labeler-review-2".to_string();
        cycle_second.previous_review_id = Some(cycle_first.review_id.clone());
        assert!(review_chain_tip(&[cycle_first, cycle_second]).is_err());
    }
}
