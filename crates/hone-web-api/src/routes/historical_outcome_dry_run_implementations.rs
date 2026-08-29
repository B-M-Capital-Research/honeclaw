//! Immutable registration for a future isolated historical-outcome dry run.
//!
//! A record here binds one reviewed labeler to one sealed price snapshot and
//! one authorization review. Registration is deliberately not execution: it
//! cannot calculate outcomes, write labels, train, reward, shadow, generate
//! orders or access a broker. A separate future review must authorize a run.

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

use super::historical_outcome_price_snapshots::{
    ApprovedHistoricalOutcomeDryRunAuthorization,
    approved_historical_outcome_dry_run_authorizations,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-implementation-registry-v1";
const IMPLEMENTATION_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-implementation-v1";
const SANDBOX_POLICY_VERSION: &str = "hone-historical-outcome-dry-run-sandbox-v1";
const INPUT_CONTRACT: &str = "只读输入：与独立授权复核精确绑定的人工批准历史状态、冻结标签器规范和不可变 FMP 复权收盘价快照；实现不得联网、补数或读取决策时点之后的非协议信息。";
const OUTPUT_CONTRACT: &str = "未来经单独授权的试运行只可向隔离临时输出写入 20/60/250 共同交易日的标的收益、SPY 收益、超额收益和最大回撤；当前登记不运行、不计算、不写结果。";
const STATUS: &str = "registered_not_run";
const MAX_NAME_CHARS: usize = 120;
const MAX_REVISION_CHARS: usize = 160;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeDryRunImplementationKind {
    DeterministicIsolatedCommonSessionReplay,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RegisterHistoricalOutcomeDryRunImplementationRequest {
    snapshot_id: String,
    expected_authorization_review_id: String,
    expected_snapshot_sha256: String,
    expected_implementation_spec_sha256: String,
    expected_protocol_sha256: String,
    implementation_name: String,
    implementation_kind: HistoricalOutcomeDryRunImplementationKind,
    code_revision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeDryRunImplementationRecord {
    pub schema_version: String,
    pub dry_run_implementation_id: String,
    pub dry_run_implementation_spec_sha256: String,
    pub registered_at: DateTime<Utc>,
    pub registered_by: String,
    pub authorization_review_id: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub reconstruction_review_id: String,
    pub labeler_implementation_id: String,
    pub labeler_implementation_spec_sha256: String,
    pub labeler_implementation_review_id: String,
    pub labeler_code_revision: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub requested_from: NaiveDate,
    pub requested_to: NaiveDate,
    pub asset_payload_sha256: String,
    pub benchmark_payload_sha256: String,
    pub asset_series_sha256: String,
    pub benchmark_series_sha256: String,
    pub common_session_count: usize,
    pub covered_horizons_market_sessions: Vec<u16>,
    pub sandbox_policy_version: String,
    pub implementation_name: String,
    pub implementation_kind: HistoricalOutcomeDryRunImplementationKind,
    pub code_revision: String,
    pub status: String,
    pub input_contract: String,
    pub output_contract: String,
    pub metrics: Vec<String>,
    pub deterministic_replay_required: bool,
    pub isolated_output_required: bool,
    pub future_information_isolation_required: bool,
    pub missing_data_fail_closed_required: bool,
    pub max_parallel_series: u8,
    pub outbound_network_allowed: bool,
    pub external_tools_allowed: bool,
    pub production_writes_allowed: bool,
    pub historical_state_mutation_allowed: bool,
    pub outcome_label_writes_allowed: bool,
    pub training_writes_allowed: bool,
    pub reward_writes_allowed: bool,
    pub shadow_writes_allowed: bool,
    pub order_generation_allowed: bool,
    pub broker_access_allowed: bool,
    pub run_authorized: bool,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunImplementationItem {
    pub implementation: HistoricalOutcomeDryRunImplementationRecord,
    pub authorization_binding_current: bool,
    pub run_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunImplementationRegistry {
    pub schema_version: String,
    pub sandbox_policy_version: String,
    pub eligible_authorizations: Vec<ApprovedHistoricalOutcomeDryRunAuthorization>,
    pub allowed_implementation_kinds: Vec<HistoricalOutcomeDryRunImplementationKind>,
    pub registration_allowed: bool,
    pub implementations: Vec<HistoricalOutcomeDryRunImplementationItem>,
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub run_authorization_review_eligible_count: usize,
    pub implementation_status: String,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeDryRunImplementationReadinessSummary {
    pub implementation_count: usize,
    pub current_binding_implementation_count: usize,
    pub run_authorization_review_eligible_count: usize,
    pub implementation_status: String,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
}

pub(crate) async fn handle_get_historical_outcome_dry_run_implementations(
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
            warn!(%error, "historical outcome dry-run implementation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线试运行实现注册表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_register_historical_outcome_dry_run_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<RegisterHistoricalOutcomeDryRunImplementationRequest>,
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
                warn!(%error, "historical outcome dry-run implementation registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线试运行实现注册表暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_dry_run_implementation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunImplementationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeDryRunImplementationReadinessSummary {
        implementation_count: registry.implementation_count,
        current_binding_implementation_count: registry.current_binding_implementation_count,
        run_authorization_review_eligible_count: registry.run_authorization_review_eligible_count,
        implementation_status: registry.implementation_status,
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
    })
}

pub(crate) async fn run_authorization_review_eligible_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeDryRunImplementationRecord>, String> {
    let eligible_authorizations = approved_historical_outcome_dry_run_authorizations(state).await?;
    let authorization_index = eligible_authorizations
        .iter()
        .map(|authorization| {
            (
                authorization.authorization_review_id.as_str(),
                authorization,
            )
        })
        .collect::<HashMap<_, _>>();
    let mut records = read_implementations(state)
        .await?
        .into_iter()
        .filter(|implementation| {
            authorization_index
                .get(implementation.authorization_review_id.as_str())
                .is_some_and(|authorization| binding_matches(implementation, authorization))
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        right.registered_at.cmp(&left.registered_at).then_with(|| {
            right
                .dry_run_implementation_id
                .cmp(&left.dry_run_implementation_id)
        })
    });
    Ok(records)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunImplementationRegistry, String> {
    let eligible_authorizations = approved_historical_outcome_dry_run_authorizations(state).await?;
    let authorization_index = eligible_authorizations
        .iter()
        .map(|authorization| {
            (
                authorization.authorization_review_id.as_str(),
                authorization,
            )
        })
        .collect::<HashMap<_, _>>();
    let records = read_implementations(state).await?;
    let mut implementations = records
        .into_iter()
        .map(|implementation| {
            let authorization_binding_current = authorization_index
                .get(implementation.authorization_review_id.as_str())
                .is_some_and(|authorization| binding_matches(&implementation, authorization));
            HistoricalOutcomeDryRunImplementationItem {
                run_authorization_review_eligible: authorization_binding_current,
                authorization_binding_current,
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
                    .dry_run_implementation_id
                    .cmp(&left.implementation.dry_run_implementation_id)
            })
    });
    let implementation_count = implementations.len();
    let current_binding_implementation_count = implementations
        .iter()
        .filter(|item| item.authorization_binding_current)
        .count();
    let run_authorization_review_eligible_count = implementations
        .iter()
        .filter(|item| item.run_authorization_review_eligible)
        .count();
    let implementation_status = if eligible_authorizations.is_empty() {
        "waiting_for_approved_dry_run_authorization"
    } else if implementation_count == 0 {
        "waiting_for_immutable_implementation_registration"
    } else if current_binding_implementation_count == 0 {
        "registered_binding_stale"
    } else {
        STATUS
    };
    Ok(HistoricalOutcomeDryRunImplementationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        sandbox_policy_version: SANDBOX_POLICY_VERSION.to_string(),
        registration_allowed: !eligible_authorizations.is_empty(),
        eligible_authorizations,
        allowed_implementation_kinds: vec![
            HistoricalOutcomeDryRunImplementationKind::DeterministicIsolatedCommonSessionReplay,
        ],
        implementations,
        implementation_count,
        current_binding_implementation_count,
        run_authorization_review_eligible_count,
        implementation_status: implementation_status.to_string(),
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
        scope: "本阶段只登记与授权复核、标签器和封存行情精确绑定的确定性隔离试运行实现；登记不等于运行，不计算结果、不写标签、不训练、不计奖励、不写影子组合、不生成订单，也不访问券商。".to_string(),
    })
}

async fn register_implementation(
    state: &AppState,
    admin_id: &str,
    request: RegisterHistoricalOutcomeDryRunImplementationRequest,
) -> Result<HistoricalOutcomeDryRunImplementationRecord, String> {
    let _lock = acquire_lock(state, "historical-outcome-dry-run-implementation").await?;
    let authorizations = approved_historical_outcome_dry_run_authorizations(state).await?;
    let authorization = authorizations
        .into_iter()
        .find(|authorization| authorization.snapshot_id == request.snapshot_id)
        .ok_or_else(|| "当前没有这条绑定有效且通过独立复核的试运行授权".to_string())?;
    if authorization.authorization_review_id != request.expected_authorization_review_id
        || authorization.snapshot_sha256 != request.expected_snapshot_sha256
        || authorization.implementation_spec_sha256 != request.expected_implementation_spec_sha256
        || authorization.protocol_sha256 != request.expected_protocol_sha256
    {
        return Err("试运行授权、行情、标签器或协议指纹已经变化，请刷新后重试".to_string());
    }
    let implementation_name =
        bounded_required(&request.implementation_name, MAX_NAME_CHARS, "实现名称")?;
    let code_revision = bounded_required(&request.code_revision, MAX_REVISION_CHARS, "代码版本")?;
    let record = new_record(
        &authorization,
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
        return Err("完全相同的离线试运行实现已经登记；规范变化时请登记新版本".to_string());
    }
    write_immutable_json(
        &implementation_root(state).join(format!("{}.json", record.dry_run_implementation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn new_record(
    authorization: &ApprovedHistoricalOutcomeDryRunAuthorization,
    admin_id: &str,
    implementation_name: String,
    implementation_kind: HistoricalOutcomeDryRunImplementationKind,
    code_revision: String,
    registered_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeDryRunImplementationRecord, String> {
    let mut record = HistoricalOutcomeDryRunImplementationRecord {
        schema_version: IMPLEMENTATION_SCHEMA_VERSION.to_string(),
        dry_run_implementation_id: String::new(),
        dry_run_implementation_spec_sha256: String::new(),
        registered_at,
        registered_by: admin_id.to_string(),
        authorization_review_id: authorization.authorization_review_id.clone(),
        snapshot_id: authorization.snapshot_id.clone(),
        snapshot_sha256: authorization.snapshot_sha256.clone(),
        reconstruction_id: authorization.reconstruction_id.clone(),
        reconstruction_sha256: authorization.reconstruction_sha256.clone(),
        reconstruction_review_id: authorization.reconstruction_review_id.clone(),
        labeler_implementation_id: authorization.implementation_id.clone(),
        labeler_implementation_spec_sha256: authorization.implementation_spec_sha256.clone(),
        labeler_implementation_review_id: authorization.implementation_review_id.clone(),
        labeler_code_revision: authorization.labeler_code_revision.clone(),
        protocol_version: authorization.protocol_version.clone(),
        protocol_sha256: authorization.protocol_sha256.clone(),
        asset_symbol: authorization.asset_symbol.clone(),
        benchmark_symbol: authorization.benchmark_symbol.clone(),
        requested_from: authorization.requested_from,
        requested_to: authorization.requested_to,
        asset_payload_sha256: authorization.asset_payload_sha256.clone(),
        benchmark_payload_sha256: authorization.benchmark_payload_sha256.clone(),
        asset_series_sha256: authorization.asset_series_sha256.clone(),
        benchmark_series_sha256: authorization.benchmark_series_sha256.clone(),
        common_session_count: authorization.common_session_count,
        covered_horizons_market_sessions: authorization.covered_horizons_market_sessions.clone(),
        sandbox_policy_version: SANDBOX_POLICY_VERSION.to_string(),
        implementation_name,
        implementation_kind,
        code_revision,
        status: STATUS.to_string(),
        input_contract: INPUT_CONTRACT.to_string(),
        output_contract: OUTPUT_CONTRACT.to_string(),
        metrics: vec![
            "asset_return".to_string(),
            "benchmark_return".to_string(),
            "excess_return".to_string(),
            "maximum_drawdown".to_string(),
        ],
        deterministic_replay_required: true,
        isolated_output_required: true,
        future_information_isolation_required: true,
        missing_data_fail_closed_required: true,
        max_parallel_series: 4,
        outbound_network_allowed: false,
        external_tools_allowed: false,
        production_writes_allowed: false,
        historical_state_mutation_allowed: false,
        outcome_label_writes_allowed: false,
        training_writes_allowed: false,
        reward_writes_allowed: false,
        shadow_writes_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        run_authorized: false,
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
    };
    record.dry_run_implementation_spec_sha256 = implementation_fingerprint(&record)?;
    record.dry_run_implementation_id = record.dry_run_implementation_spec_sha256[..32].to_string();
    validate_record(&record)?;
    Ok(record)
}

fn validate_record(record: &HistoricalOutcomeDryRunImplementationRecord) -> Result<(), String> {
    let permissions_are_closed = !record.outbound_network_allowed
        && !record.external_tools_allowed
        && !record.production_writes_allowed
        && !record.historical_state_mutation_allowed
        && !record.outcome_label_writes_allowed
        && !record.training_writes_allowed
        && !record.reward_writes_allowed
        && !record.shadow_writes_allowed
        && !record.order_generation_allowed
        && !record.broker_access_allowed
        && !record.run_authorized
        && !record.offline_dry_run_enabled
        && !record.outcome_label_generation_enabled
        && !record.decision_training_authorized
        && !record.reward_evidence_authorized
        && !record.shadow_evidence_authorized
        && !record.trading_authorized;
    if record.schema_version != IMPLEMENTATION_SCHEMA_VERSION
        || !valid_id(&record.dry_run_implementation_id)
        || !valid_sha256(&record.dry_run_implementation_spec_sha256)
        || record.dry_run_implementation_id != record.dry_run_implementation_spec_sha256[..32]
        || record.dry_run_implementation_spec_sha256 != implementation_fingerprint(record)?
        || record.registered_by.trim().is_empty()
        || !valid_id(&record.authorization_review_id)
        || !valid_id(&record.snapshot_id)
        || !valid_sha256(&record.snapshot_sha256)
        || !valid_id(&record.reconstruction_id)
        || !valid_sha256(&record.reconstruction_sha256)
        || !valid_id(&record.reconstruction_review_id)
        || !valid_id(&record.labeler_implementation_id)
        || !valid_sha256(&record.labeler_implementation_spec_sha256)
        || !valid_id(&record.labeler_implementation_review_id)
        || record.labeler_code_revision.trim().is_empty()
        || record.protocol_version.trim().is_empty()
        || !valid_sha256(&record.protocol_sha256)
        || !valid_symbol(&record.asset_symbol)
        || !valid_symbol(&record.benchmark_symbol)
        || record.requested_from > record.requested_to
        || !valid_sha256(&record.asset_payload_sha256)
        || !valid_sha256(&record.benchmark_payload_sha256)
        || !valid_sha256(&record.asset_series_sha256)
        || !valid_sha256(&record.benchmark_series_sha256)
        || record.common_session_count <= 250
        || record.covered_horizons_market_sessions != vec![20, 60, 250]
        || record.sandbox_policy_version != SANDBOX_POLICY_VERSION
        || record.implementation_name.trim().is_empty()
        || record.implementation_name.chars().count() > MAX_NAME_CHARS
        || record.code_revision.trim().is_empty()
        || record.code_revision.chars().count() > MAX_REVISION_CHARS
        || record.status != STATUS
        || record.input_contract != INPUT_CONTRACT
        || record.output_contract != OUTPUT_CONTRACT
        || record.metrics
            != vec![
                "asset_return",
                "benchmark_return",
                "excess_return",
                "maximum_drawdown",
            ]
        || !record.deterministic_replay_required
        || !record.isolated_output_required
        || !record.future_information_isolation_required
        || !record.missing_data_fail_closed_required
        || record.max_parallel_series != 4
        || !permissions_are_closed
    {
        return Err(
            "historical outcome dry-run implementation is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn binding_matches(
    record: &HistoricalOutcomeDryRunImplementationRecord,
    authorization: &ApprovedHistoricalOutcomeDryRunAuthorization,
) -> bool {
    record.authorization_review_id == authorization.authorization_review_id
        && record.snapshot_id == authorization.snapshot_id
        && record.snapshot_sha256 == authorization.snapshot_sha256
        && record.reconstruction_id == authorization.reconstruction_id
        && record.reconstruction_sha256 == authorization.reconstruction_sha256
        && record.reconstruction_review_id == authorization.reconstruction_review_id
        && record.labeler_implementation_id == authorization.implementation_id
        && record.labeler_implementation_spec_sha256 == authorization.implementation_spec_sha256
        && record.labeler_implementation_review_id == authorization.implementation_review_id
        && record.labeler_code_revision == authorization.labeler_code_revision
        && record.protocol_version == authorization.protocol_version
        && record.protocol_sha256 == authorization.protocol_sha256
        && record.asset_symbol == authorization.asset_symbol
        && record.benchmark_symbol == authorization.benchmark_symbol
        && record.requested_from == authorization.requested_from
        && record.requested_to == authorization.requested_to
        && record.asset_payload_sha256 == authorization.asset_payload_sha256
        && record.benchmark_payload_sha256 == authorization.benchmark_payload_sha256
        && record.asset_series_sha256 == authorization.asset_series_sha256
        && record.benchmark_series_sha256 == authorization.benchmark_series_sha256
        && record.common_session_count == authorization.common_session_count
        && record.covered_horizons_market_sessions == authorization.covered_horizons_market_sessions
}

fn implementation_fingerprint(
    record: &HistoricalOutcomeDryRunImplementationRecord,
) -> Result<String, String> {
    let mut value = serde_json::to_value(record).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "dry-run implementation fingerprint payload is invalid".to_string())?;
    object.remove("dry_run_implementation_id");
    object.remove("dry_run_implementation_spec_sha256");
    hash_serializable(&value)
}

fn implementation_deduplication_fingerprint(
    record: &HistoricalOutcomeDryRunImplementationRecord,
) -> Result<String, String> {
    let mut value = serde_json::to_value(record).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "dry-run implementation deduplication payload is invalid".to_string())?;
    object.remove("dry_run_implementation_id");
    object.remove("dry_run_implementation_spec_sha256");
    object.remove("registered_at");
    object.remove("registered_by");
    hash_serializable(&value)
}

async fn read_implementations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeDryRunImplementationRecord>, String> {
    let directory = implementation_root(state);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
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
        let record: HistoricalOutcomeDryRunImplementationRecord =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_record(&record)?;
        if !ids.insert(record.dry_run_implementation_id.clone())
            || !fingerprints.insert(record.dry_run_implementation_spec_sha256.clone())
        {
            return Err(
                "historical outcome dry-run implementation registry contains duplicates"
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
    decision_root(state).join("historical-outcome-dry-run-implementations")
}

struct HistoricalOutcomeDryRunImplementationLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeDryRunImplementationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<HistoricalOutcomeDryRunImplementationLock, String> {
    if !valid_id(name) {
        return Err("invalid historical outcome dry-run implementation lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("{name}.lock"));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(HistoricalOutcomeDryRunImplementationLock { path }),
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
                return Err("离线试运行实现正在由另一请求登记，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("离线试运行实现锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid immutable audit path".to_string())?;
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

    fn authorization() -> ApprovedHistoricalOutcomeDryRunAuthorization {
        ApprovedHistoricalOutcomeDryRunAuthorization {
            authorization_review_id: "authorization-review-1".to_string(),
            snapshot_id: "snapshot-1".to_string(),
            snapshot_sha256: "a".repeat(64),
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "b".repeat(64),
            reconstruction_review_id: "reconstruction-review-1".to_string(),
            implementation_id: "labeler-1".to_string(),
            implementation_spec_sha256: "c".repeat(64),
            implementation_review_id: "labeler-review-1".to_string(),
            labeler_code_revision: "oldwang@labeler123".to_string(),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "d".repeat(64),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            requested_from: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            requested_to: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            asset_payload_sha256: "e".repeat(64),
            benchmark_payload_sha256: "f".repeat(64),
            asset_series_sha256: "1".repeat(64),
            benchmark_series_sha256: "2".repeat(64),
            common_session_count: 251,
            covered_horizons_market_sessions: vec![20, 60, 250],
        }
    }

    fn record() -> HistoricalOutcomeDryRunImplementationRecord {
        new_record(
            &authorization(),
            "admin",
            "共同交易日隔离试运行实现".to_string(),
            HistoricalOutcomeDryRunImplementationKind::DeterministicIsolatedCommonSessionReplay,
            "oldwang@dryrun123".to_string(),
            DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        )
        .unwrap()
    }

    #[test]
    fn registration_is_immutable_and_never_runs() {
        let value = record();
        assert!(validate_record(&value).is_ok());
        assert_eq!(value.status, "registered_not_run");
        assert!(!value.run_authorized);
        assert!(!value.offline_dry_run_enabled);
        assert!(!value.outcome_label_generation_enabled);
        assert!(!value.decision_training_authorized);
        assert!(!value.shadow_evidence_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn fingerprint_detects_tampering() {
        let mut value = record();
        value.common_session_count = 252;
        assert!(validate_record(&value).is_err());
        let mut audit_tamper = record();
        audit_tamper.registered_by = "different-admin".to_string();
        assert!(validate_record(&audit_tamper).is_err());
    }

    #[test]
    fn authorization_binding_is_exact() {
        let value = record();
        let mut approved = authorization();
        assert!(binding_matches(&value, &approved));
        approved.asset_series_sha256 = "9".repeat(64);
        assert!(!binding_matches(&value, &approved));
    }

    #[test]
    fn duplicate_spec_is_detected_across_registration_metadata() {
        let first = record();
        let mut second = record();
        second.registered_by = "second-admin".to_string();
        second.registered_at = DateTime::parse_from_rfc3339("2026-08-22T13:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        second.dry_run_implementation_spec_sha256 = implementation_fingerprint(&second).unwrap();
        second.dry_run_implementation_id =
            second.dry_run_implementation_spec_sha256[..32].to_string();
        assert_ne!(
            first.dry_run_implementation_spec_sha256,
            second.dry_run_implementation_spec_sha256
        );
        assert_eq!(
            implementation_deduplication_fingerprint(&first).unwrap(),
            implementation_deduplication_fingerprint(&second).unwrap()
        );
    }

    #[test]
    fn over_authorized_record_is_rejected() {
        let mut value = record();
        value.order_generation_allowed = true;
        value.dry_run_implementation_spec_sha256 = implementation_fingerprint(&value).unwrap();
        value.dry_run_implementation_id =
            value.dry_run_implementation_spec_sha256[..32].to_string();
        assert!(validate_record(&value).is_err());
    }

    #[test]
    fn incomplete_protocol_horizons_are_rejected() {
        let mut value = record();
        value.covered_horizons_market_sessions = vec![20, 60];
        value.dry_run_implementation_spec_sha256 = implementation_fingerprint(&value).unwrap();
        value.dry_run_implementation_id =
            value.dry_run_implementation_spec_sha256[..32].to_string();
        assert!(validate_record(&value).is_err());
    }
}
