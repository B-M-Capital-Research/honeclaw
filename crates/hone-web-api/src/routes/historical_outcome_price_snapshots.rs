//! Sealed FMP adjusted-close inputs and dry-run authorization governance.
//!
//! Price ingestion is deliberately separate from the historical labeler.  It
//! may fetch and seal normalized market series, but it never calculates an
//! outcome or mutates a training sample.  A later immutable review may allow
//! an exact labeler/snapshot pair to register an isolated dry-run execution;
//! it still does not run the labeler or enable labels, training, rewards,
//! shadow portfolios or trading.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use chrono_tz::America::New_York;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::historical_outcome_governance::protocol_fingerprint;
use super::historical_outcome_labeler_registry::{
    ApprovedHistoricalOutcomeLabeler, approved_historical_outcome_labelers,
};
use super::historical_state_reconstructions::{
    ApprovedHistoricalBenchmarkState, approved_historical_benchmark_states, outcome_protocol,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-price-snapshot-registry-v1";
const SNAPSHOT_SCHEMA_VERSION: &str = "hone-historical-outcome-price-snapshot-v1";
const INGESTION_POLICY_VERSION: &str = "hone-historical-outcome-price-ingestion-v1";
const AUTHORIZATION_REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-dry-run-authorization-registry-v1";
const AUTHORIZATION_REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-dry-run-authorization-review-v1";
const AUTHORIZATION_POLICY_VERSION: &str = "hone-historical-outcome-dry-run-authorization-v1";
const PROVIDER_NAME: &str = "Financial Modeling Prep";
const PROVIDER_ENDPOINT_TEMPLATE: &str = "/v3/historical-price-full/{symbol}?from={from}&to={to}";
const PRICE_BASIS: &str = "FMP adjusted close";
const MAX_RATIONALE_CHARS: usize = 1_600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct SealedAdjustedClosePoint {
    pub date: NaiveDate,
    pub adjusted_close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomePriceSnapshot {
    pub schema_version: String,
    pub ingestion_policy_version: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub sealed_at: DateTime<Utc>,
    pub sealed_by: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub reconstruction_review_id: String,
    pub anchor_candidate_id: String,
    pub anchor_review_id: String,
    pub decision_available_at: DateTime<Utc>,
    pub implementation_id: String,
    pub implementation_spec_sha256: String,
    pub implementation_review_id: String,
    pub governance_review_id: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub code_revision: String,
    pub provider: String,
    pub provider_endpoint_template: String,
    pub price_basis: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub requested_from: NaiveDate,
    pub requested_to: NaiveDate,
    pub asset_payload_sha256: String,
    pub benchmark_payload_sha256: String,
    pub asset_series_sha256: String,
    pub benchmark_series_sha256: String,
    pub asset_points: Vec<SealedAdjustedClosePoint>,
    pub benchmark_points: Vec<SealedAdjustedClosePoint>,
    pub common_session_count: usize,
    pub covered_horizons_market_sessions: Vec<u16>,
    pub all_protocol_horizons_covered: bool,
    pub outcome_metrics_computed: bool,
    pub label_written: bool,
    pub historical_state_mutated: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct IngestHistoricalOutcomePriceSnapshotRequest {
    reconstruction_id: String,
    expected_reconstruction_sha256: String,
    expected_reconstruction_review_id: String,
    implementation_id: String,
    expected_implementation_spec_sha256: String,
    expected_implementation_review_id: String,
    expected_protocol_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomePriceSnapshotItem {
    pub snapshot: HistoricalOutcomePriceSnapshot,
    pub reconstruction_binding_current: bool,
    pub implementation_binding_current: bool,
    pub dry_run_authorization_review_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomePriceSnapshotRegistry {
    pub schema_version: String,
    pub ingestion_policy_version: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub eligible_benchmark_states: Vec<ApprovedHistoricalBenchmarkState>,
    pub eligible_labelers: Vec<ApprovedHistoricalOutcomeLabeler>,
    pub snapshots: Vec<HistoricalOutcomePriceSnapshotItem>,
    pub current_snapshot_count: usize,
    pub fully_covered_snapshot_count: usize,
    pub price_snapshot_ingestion_enabled: bool,
    pub outcome_label_generation_enabled: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeDryRunAuthorizationVerdict {
    ApprovedForDryRunImplementationRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeDryRunAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_snapshot_sha256: String,
    expected_implementation_spec_sha256: String,
    verdict: HistoricalOutcomeDryRunAuthorizationVerdict,
    rationale: String,
    current_bindings_confirmed: bool,
    sealed_snapshot_integrity_confirmed: bool,
    provider_provenance_confirmed: bool,
    complete_common_session_coverage_confirmed: bool,
    deterministic_fixture_confirmed: bool,
    isolated_output_confirmed: bool,
    no_label_or_production_writes_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeDryRunAuthorizationReview {
    pub schema_version: String,
    pub authorization_policy_version: String,
    pub review_id: String,
    pub previous_review_id: Option<String>,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub reconstruction_review_id: String,
    pub implementation_id: String,
    pub implementation_spec_sha256: String,
    pub implementation_review_id: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub verdict: HistoricalOutcomeDryRunAuthorizationVerdict,
    pub rationale: String,
    pub current_bindings_confirmed: bool,
    pub sealed_snapshot_integrity_confirmed: bool,
    pub provider_provenance_confirmed: bool,
    pub complete_common_session_coverage_confirmed: bool,
    pub deterministic_fixture_confirmed: bool,
    pub isolated_output_confirmed: bool,
    pub no_label_or_production_writes_confirmed: bool,
    pub dry_run_implementation_registration_eligible: bool,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunAuthorizationItem {
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub asset_symbol: String,
    pub common_session_count: usize,
    pub current_binding: bool,
    pub latest_review: Option<HistoricalOutcomeDryRunAuthorizationReview>,
    pub dry_run_implementation_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunAuthorizationRegistry {
    pub schema_version: String,
    pub authorization_policy_version: String,
    pub items: Vec<HistoricalOutcomeDryRunAuthorizationItem>,
    pub reviewed_snapshot_count: usize,
    pub registration_eligible_snapshot_count: usize,
    pub authorization_status: String,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeSnapshotReadinessSummary {
    pub current_snapshot_count: usize,
    pub fully_covered_snapshot_count: usize,
    pub dry_run_implementation_registration_eligible_count: usize,
    pub authorization_status: String,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
}

/// Returns only sealed price inputs whose benchmark-state and implementation
/// bindings are still current.  Offline transformation may use this projection
/// solely to recover the frozen common-market-session index; it must not fetch
/// fresh prices or recompute an outcome.
pub(crate) async fn current_sealed_historical_outcome_price_snapshots_for_transformation(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomePriceSnapshot>, String> {
    let registry = build_price_snapshot_registry(state).await?;
    Ok(registry
        .snapshots
        .into_iter()
        .filter(|item| {
            item.reconstruction_binding_current
                && item.implementation_binding_current
                && item.snapshot.all_protocol_horizons_covered
                && !item.snapshot.outcome_metrics_computed
                && !item.snapshot.label_written
                && !item.snapshot.historical_state_mutated
        })
        .map(|item| item.snapshot)
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeDryRunAuthorization {
    pub authorization_review_id: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub reconstruction_review_id: String,
    pub implementation_id: String,
    pub implementation_spec_sha256: String,
    pub implementation_review_id: String,
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
}

pub(crate) async fn handle_get_historical_outcome_price_snapshots(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_price_snapshot_registry(&state).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => {
            warn!(%error, "historical outcome price snapshot registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果行情快照暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_ingest_historical_outcome_price_snapshot(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<IngestHistoricalOutcomePriceSnapshotRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match ingest_price_snapshot(&state, &admin.user_id, request).await {
        Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn handle_get_historical_outcome_dry_run_authorizations(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_authorization_registry(&state).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => {
            warn!(%error, "historical outcome dry-run authorization read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线试运行授权暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_dry_run_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(snapshot_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeDryRunAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_dry_run_authorization(&state, &admin.user_id, &snapshot_id, request).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_snapshot_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeSnapshotReadinessSummary, String> {
    let snapshots = build_price_snapshot_registry(state).await?;
    let authorizations = build_authorization_registry(state).await?;
    Ok(HistoricalOutcomeSnapshotReadinessSummary {
        current_snapshot_count: snapshots.current_snapshot_count,
        fully_covered_snapshot_count: snapshots.fully_covered_snapshot_count,
        dry_run_implementation_registration_eligible_count: authorizations
            .registration_eligible_snapshot_count,
        authorization_status: authorizations.authorization_status,
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
    })
}

pub(crate) async fn approved_historical_outcome_dry_run_authorizations(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeDryRunAuthorization>, String> {
    let registry = build_price_snapshot_registry(state).await?;
    let mut approved = Vec::new();
    for item in registry.snapshots {
        if !item.dry_run_authorization_review_eligible {
            continue;
        }
        let Some(review) = latest_authorization_review(state, &item.snapshot).await? else {
            continue;
        };
        if review.verdict
            != HistoricalOutcomeDryRunAuthorizationVerdict::ApprovedForDryRunImplementationRegistration
            || !review.dry_run_implementation_registration_eligible
        {
            continue;
        }
        approved.push(ApprovedHistoricalOutcomeDryRunAuthorization {
            authorization_review_id: review.review_id,
            snapshot_id: item.snapshot.snapshot_id,
            snapshot_sha256: item.snapshot.snapshot_sha256,
            reconstruction_id: item.snapshot.reconstruction_id,
            reconstruction_sha256: item.snapshot.reconstruction_sha256,
            reconstruction_review_id: item.snapshot.reconstruction_review_id,
            implementation_id: item.snapshot.implementation_id,
            implementation_spec_sha256: item.snapshot.implementation_spec_sha256,
            implementation_review_id: item.snapshot.implementation_review_id,
            labeler_code_revision: item.snapshot.code_revision,
            protocol_version: item.snapshot.protocol_version,
            protocol_sha256: item.snapshot.protocol_sha256,
            asset_symbol: item.snapshot.asset_symbol,
            benchmark_symbol: item.snapshot.benchmark_symbol,
            requested_from: item.snapshot.requested_from,
            requested_to: item.snapshot.requested_to,
            asset_payload_sha256: item.snapshot.asset_payload_sha256,
            benchmark_payload_sha256: item.snapshot.benchmark_payload_sha256,
            asset_series_sha256: item.snapshot.asset_series_sha256,
            benchmark_series_sha256: item.snapshot.benchmark_series_sha256,
            common_session_count: item.snapshot.common_session_count,
            covered_horizons_market_sessions: item.snapshot.covered_horizons_market_sessions,
        });
    }
    approved.sort_by(|left, right| {
        left.asset_symbol
            .cmp(&right.asset_symbol)
            .then_with(|| left.snapshot_id.cmp(&right.snapshot_id))
    });
    Ok(approved)
}

pub(crate) async fn current_sealed_snapshot_for_execution(
    state: &AppState,
    snapshot_id: &str,
    expected_snapshot_sha256: &str,
) -> Result<Option<HistoricalOutcomePriceSnapshot>, String> {
    let registry = build_price_snapshot_registry(state).await?;
    Ok(registry
        .snapshots
        .into_iter()
        .find(|item| {
            item.reconstruction_binding_current
                && item.implementation_binding_current
                && item.snapshot.snapshot_id == snapshot_id
                && item.snapshot.snapshot_sha256 == expected_snapshot_sha256
        })
        .map(|item| item.snapshot))
}

async fn build_price_snapshot_registry(
    state: &AppState,
) -> Result<HistoricalOutcomePriceSnapshotRegistry, String> {
    let protocol = outcome_protocol();
    let protocol_sha256 = protocol_sha256()?;
    let benchmark_states = approved_historical_benchmark_states(state).await?;
    let labelers = approved_historical_outcome_labelers(state).await?;
    let state_index = benchmark_states
        .iter()
        .map(|item| (item.reconstruction_id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let labeler_index = labelers
        .iter()
        .map(|item| (item.implementation_id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let mut items = Vec::new();
    for snapshot in read_snapshots(state).await? {
        let reconstruction_binding_current = state_index
            .get(snapshot.reconstruction_id.as_str())
            .is_some_and(|current| benchmark_state_binding_matches(&snapshot, current));
        let implementation_binding_current = labeler_index
            .get(snapshot.implementation_id.as_str())
            .is_some_and(|current| labeler_binding_matches(&snapshot, current));
        let dry_run_authorization_review_eligible = reconstruction_binding_current
            && implementation_binding_current
            && snapshot.protocol_version == protocol.protocol_version
            && snapshot.protocol_sha256 == protocol_sha256
            && snapshot.all_protocol_horizons_covered;
        items.push(HistoricalOutcomePriceSnapshotItem {
            snapshot,
            reconstruction_binding_current,
            implementation_binding_current,
            dry_run_authorization_review_eligible,
        });
    }
    items.sort_by(|left, right| {
        right
            .snapshot
            .sealed_at
            .cmp(&left.snapshot.sealed_at)
            .then_with(|| right.snapshot.snapshot_id.cmp(&left.snapshot.snapshot_id))
    });
    let current_snapshot_count = items
        .iter()
        .filter(|item| item.reconstruction_binding_current && item.implementation_binding_current)
        .count();
    let fully_covered_snapshot_count = items
        .iter()
        .filter(|item| item.dry_run_authorization_review_eligible)
        .count();
    Ok(HistoricalOutcomePriceSnapshotRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        ingestion_policy_version: INGESTION_POLICY_VERSION.to_string(),
        protocol_version: protocol.protocol_version,
        protocol_sha256,
        eligible_benchmark_states: benchmark_states,
        eligible_labelers: labelers,
        snapshots: items,
        current_snapshot_count,
        fully_covered_snapshot_count,
        price_snapshot_ingestion_enabled: true,
        outcome_label_generation_enabled: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        trading_authorized: false,
        scope: "摄取阶段只从 FMP 读取并封存复权收盘价，保存来源、截止时间、规范化载荷和序列指纹；不计算收益、不改写历史状态、不写标签，也不授权训练、奖励、影子或交易。".to_string(),
    })
}

async fn ingest_price_snapshot(
    state: &AppState,
    admin_id: &str,
    request: IngestHistoricalOutcomePriceSnapshotRequest,
) -> Result<HistoricalOutcomePriceSnapshot, String> {
    let _lock = acquire_lock(state, "price-snapshot-ingestion").await?;
    let benchmark_state = approved_historical_benchmark_states(state)
        .await?
        .into_iter()
        .find(|item| item.reconstruction_id == request.reconstruction_id)
        .ok_or_else(|| "当前没有这条人工批准且绑定有效的历史基准状态".to_string())?;
    if benchmark_state.reconstruction_sha256 != request.expected_reconstruction_sha256
        || benchmark_state.reconstruction_review_id != request.expected_reconstruction_review_id
    {
        return Err("历史点时基准状态已经变化，请刷新后重试".to_string());
    }
    let labeler = approved_historical_outcome_labelers(state)
        .await?
        .into_iter()
        .find(|item| item.implementation_id == request.implementation_id)
        .ok_or_else(|| "当前没有这条通过独立复核的标签器实现".to_string())?;
    if labeler.implementation_spec_sha256 != request.expected_implementation_spec_sha256
        || labeler.implementation_review_id != request.expected_implementation_review_id
        || labeler.protocol_sha256 != request.expected_protocol_sha256
    {
        return Err("标签器实现、复核或协议指纹已经变化，请刷新后重试".to_string());
    }
    let keys = state.core.config.fmp.effective_key_pool().keys().to_vec();
    if keys.is_empty() {
        return Err("未配置 FMP API Key，不能封存历史复权行情".to_string());
    }
    let sealed_at = Utc::now();
    let requested_to = latest_completed_new_york_market_date(sealed_at);
    let requested_from = earliest_outcome_start_date(benchmark_state.decision_available_at);
    if requested_to < requested_from {
        return Err("历史判断之后尚无已完成的美股交易日".to_string());
    }
    let asset_value = fetch_adjusted_history_value(
        state,
        &keys,
        &benchmark_state.symbol,
        requested_from,
        requested_to,
    )
    .await?;
    let protocol = outcome_protocol();
    let benchmark_value = fetch_adjusted_history_value(
        state,
        &keys,
        &protocol.benchmark_symbol,
        requested_from,
        requested_to,
    )
    .await?;
    let asset_points = adjusted_points_from_value(&asset_value, requested_from, requested_to)?;
    let benchmark_points =
        adjusted_points_from_value(&benchmark_value, requested_from, requested_to)?;
    let asset_payload_sha256 = sealed_payload_sha256(
        &benchmark_state.symbol,
        requested_from,
        requested_to,
        &asset_points,
    )?;
    let benchmark_payload_sha256 = sealed_payload_sha256(
        &protocol.benchmark_symbol,
        requested_from,
        requested_to,
        &benchmark_points,
    )?;
    let common_session_count = common_session_count(&asset_points, &benchmark_points);
    let covered_horizons_market_sessions = protocol
        .horizons_market_sessions
        .iter()
        .copied()
        .filter(|horizon| common_session_count > usize::from(*horizon))
        .collect::<Vec<_>>();
    let all_protocol_horizons_covered =
        covered_horizons_market_sessions == protocol.horizons_market_sessions;
    let asset_series_sha256 = hash_serializable(&asset_points)?;
    let benchmark_series_sha256 = hash_serializable(&benchmark_points)?;
    let mut snapshot = HistoricalOutcomePriceSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION.to_string(),
        ingestion_policy_version: INGESTION_POLICY_VERSION.to_string(),
        snapshot_id: String::new(),
        snapshot_sha256: String::new(),
        sealed_at,
        sealed_by: admin_id.to_string(),
        reconstruction_id: benchmark_state.reconstruction_id,
        reconstruction_sha256: benchmark_state.reconstruction_sha256,
        reconstruction_review_id: benchmark_state.reconstruction_review_id,
        anchor_candidate_id: benchmark_state.anchor_candidate_id,
        anchor_review_id: benchmark_state.anchor_review_id,
        decision_available_at: benchmark_state.decision_available_at,
        implementation_id: labeler.implementation_id,
        implementation_spec_sha256: labeler.implementation_spec_sha256,
        implementation_review_id: labeler.implementation_review_id,
        governance_review_id: labeler.governance_review_id,
        protocol_version: labeler.protocol_version,
        protocol_sha256: labeler.protocol_sha256,
        code_revision: labeler.code_revision,
        provider: PROVIDER_NAME.to_string(),
        provider_endpoint_template: PROVIDER_ENDPOINT_TEMPLATE.to_string(),
        price_basis: PRICE_BASIS.to_string(),
        asset_symbol: benchmark_state.symbol,
        benchmark_symbol: protocol.benchmark_symbol,
        requested_from,
        requested_to,
        asset_payload_sha256,
        benchmark_payload_sha256,
        asset_series_sha256,
        benchmark_series_sha256,
        asset_points,
        benchmark_points,
        common_session_count,
        covered_horizons_market_sessions,
        all_protocol_horizons_covered,
        outcome_metrics_computed: false,
        label_written: false,
        historical_state_mutated: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
    };
    snapshot.snapshot_sha256 = snapshot_fingerprint(&snapshot)?;
    snapshot.snapshot_id = snapshot.snapshot_sha256[..32].to_string();
    validate_snapshot(&snapshot)?;
    write_immutable_json(
        &snapshot_directory(state, &snapshot.reconstruction_id)
            .join(format!("{}.json", snapshot.snapshot_id)),
        &snapshot,
    )
    .await?;
    Ok(snapshot)
}

async fn build_authorization_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunAuthorizationRegistry, String> {
    let snapshots = build_price_snapshot_registry(state).await?;
    let mut items = Vec::with_capacity(snapshots.snapshots.len());
    for item in snapshots.snapshots {
        let latest_review = latest_authorization_review(state, &item.snapshot).await?;
        let current_binding = item.dry_run_authorization_review_eligible;
        let dry_run_implementation_registration_eligible = current_binding
            && latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    == HistoricalOutcomeDryRunAuthorizationVerdict::ApprovedForDryRunImplementationRegistration
                    && review.dry_run_implementation_registration_eligible
            });
        items.push(HistoricalOutcomeDryRunAuthorizationItem {
            snapshot_id: item.snapshot.snapshot_id,
            snapshot_sha256: item.snapshot.snapshot_sha256,
            asset_symbol: item.snapshot.asset_symbol,
            common_session_count: item.snapshot.common_session_count,
            current_binding,
            latest_review,
            dry_run_implementation_registration_eligible,
        });
    }
    let reviewed_snapshot_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let registration_eligible_snapshot_count = items
        .iter()
        .filter(|item| item.dry_run_implementation_registration_eligible)
        .count();
    let authorization_status = if items.is_empty() {
        "waiting_for_sealed_price_snapshot"
    } else if !items.iter().any(|item| item.current_binding) {
        "waiting_for_current_complete_snapshot"
    } else if registration_eligible_snapshot_count == 0 {
        "waiting_for_independent_authorization_review"
    } else {
        "dry_run_implementation_registration_eligible"
    };
    Ok(HistoricalOutcomeDryRunAuthorizationRegistry {
        schema_version: AUTHORIZATION_REGISTRY_SCHEMA_VERSION.to_string(),
        authorization_policy_version: AUTHORIZATION_POLICY_VERSION.to_string(),
        items,
        reviewed_snapshot_count,
        registration_eligible_snapshot_count,
        authorization_status: authorization_status.to_string(),
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
        scope: "授权复核只允许精确的标签器实现与封存行情快照进入下一步离线试运行实现登记；当前不运行标签器、不生成结果、不写训练样本，也不授权奖励、影子或交易。".to_string(),
    })
}

async fn review_dry_run_authorization(
    state: &AppState,
    reviewer_id: &str,
    snapshot_id: &str,
    request: ReviewHistoricalOutcomeDryRunAuthorizationRequest,
) -> Result<HistoricalOutcomeDryRunAuthorizationReview, String> {
    if !valid_id(snapshot_id) {
        return Err("行情快照编号无效".to_string());
    }
    let _lock = acquire_lock(state, &format!("dry-run-authorization-{snapshot_id}")).await?;
    let registry = build_price_snapshot_registry(state).await?;
    let item = registry
        .snapshots
        .into_iter()
        .find(|item| item.snapshot.snapshot_id == snapshot_id)
        .ok_or_else(|| "封存行情快照不存在".to_string())?;
    if request.expected_snapshot_sha256 != item.snapshot.snapshot_sha256
        || request.expected_implementation_spec_sha256 != item.snapshot.implementation_spec_sha256
    {
        return Err("行情快照或标签器实现指纹已经变化，请刷新后重试".to_string());
    }
    let previous = latest_authorization_review(state, &item.snapshot).await?;
    if request.expected_review_id.as_deref()
        != previous.as_ref().map(|review| review.review_id.as_str())
    {
        return Err("离线试运行授权复核已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let approved = request.verdict
        == HistoricalOutcomeDryRunAuthorizationVerdict::ApprovedForDryRunImplementationRegistration;
    let all_checks = authorization_request_all_checks(&request);
    if approved && !item.dry_run_authorization_review_eligible {
        return Err("行情快照绑定失效或未覆盖全部协议窗口，不能批准".to_string());
    }
    if approved && !all_checks {
        return Err("批准前必须完成绑定、完整性、来源、覆盖、确定性与隔离检查".to_string());
    }
    let review = HistoricalOutcomeDryRunAuthorizationReview {
        schema_version: AUTHORIZATION_REVIEW_SCHEMA_VERSION.to_string(),
        authorization_policy_version: AUTHORIZATION_POLICY_VERSION.to_string(),
        review_id: Uuid::new_v4().simple().to_string(),
        previous_review_id: previous.map(|review| review.review_id),
        snapshot_id: item.snapshot.snapshot_id.clone(),
        snapshot_sha256: item.snapshot.snapshot_sha256.clone(),
        reconstruction_id: item.snapshot.reconstruction_id.clone(),
        reconstruction_sha256: item.snapshot.reconstruction_sha256.clone(),
        reconstruction_review_id: item.snapshot.reconstruction_review_id.clone(),
        implementation_id: item.snapshot.implementation_id.clone(),
        implementation_spec_sha256: item.snapshot.implementation_spec_sha256.clone(),
        implementation_review_id: item.snapshot.implementation_review_id.clone(),
        protocol_version: item.snapshot.protocol_version.clone(),
        protocol_sha256: item.snapshot.protocol_sha256.clone(),
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        verdict: request.verdict,
        rationale,
        current_bindings_confirmed: request.current_bindings_confirmed,
        sealed_snapshot_integrity_confirmed: request.sealed_snapshot_integrity_confirmed,
        provider_provenance_confirmed: request.provider_provenance_confirmed,
        complete_common_session_coverage_confirmed: request
            .complete_common_session_coverage_confirmed,
        deterministic_fixture_confirmed: request.deterministic_fixture_confirmed,
        isolated_output_confirmed: request.isolated_output_confirmed,
        no_label_or_production_writes_confirmed: request.no_label_or_production_writes_confirmed,
        dry_run_implementation_registration_eligible: approved,
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
    };
    validate_authorization_review(&review, &item.snapshot)?;
    write_immutable_json(
        &authorization_review_directory(state, &item.snapshot)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn authorization_request_all_checks(
    request: &ReviewHistoricalOutcomeDryRunAuthorizationRequest,
) -> bool {
    request.current_bindings_confirmed
        && request.sealed_snapshot_integrity_confirmed
        && request.provider_provenance_confirmed
        && request.complete_common_session_coverage_confirmed
        && request.deterministic_fixture_confirmed
        && request.isolated_output_confirmed
        && request.no_label_or_production_writes_confirmed
}

fn validate_authorization_review(
    review: &HistoricalOutcomeDryRunAuthorizationReview,
    snapshot: &HistoricalOutcomePriceSnapshot,
) -> Result<(), String> {
    let approved = review.verdict
        == HistoricalOutcomeDryRunAuthorizationVerdict::ApprovedForDryRunImplementationRegistration;
    let all_checks = review.current_bindings_confirmed
        && review.sealed_snapshot_integrity_confirmed
        && review.provider_provenance_confirmed
        && review.complete_common_session_coverage_confirmed
        && review.deterministic_fixture_confirmed
        && review.isolated_output_confirmed
        && review.no_label_or_production_writes_confirmed;
    if review.schema_version != AUTHORIZATION_REVIEW_SCHEMA_VERSION
        || review.authorization_policy_version != AUTHORIZATION_POLICY_VERSION
        || !valid_id(&review.review_id)
        || review.snapshot_id != snapshot.snapshot_id
        || review.snapshot_sha256 != snapshot.snapshot_sha256
        || review.reconstruction_id != snapshot.reconstruction_id
        || review.reconstruction_sha256 != snapshot.reconstruction_sha256
        || review.reconstruction_review_id != snapshot.reconstruction_review_id
        || review.implementation_id != snapshot.implementation_id
        || review.implementation_spec_sha256 != snapshot.implementation_spec_sha256
        || review.implementation_review_id != snapshot.implementation_review_id
        || review.protocol_version != snapshot.protocol_version
        || review.protocol_sha256 != snapshot.protocol_sha256
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || approved != (all_checks && review.dry_run_implementation_registration_eligible)
        || review.offline_dry_run_enabled
        || review.outcome_label_generation_enabled
        || review.decision_training_authorized
        || review.reward_evidence_authorized
        || review.shadow_evidence_authorized
        || review.trading_authorized
    {
        return Err(
            "historical outcome dry-run authorization is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

async fn latest_authorization_review(
    state: &AppState,
    snapshot: &HistoricalOutcomePriceSnapshot,
) -> Result<Option<HistoricalOutcomeDryRunAuthorizationReview>, String> {
    let directory = authorization_review_directory(state, snapshot);
    let mut reviews =
        read_json_records::<HistoricalOutcomeDryRunAuthorizationReview>(&directory).await?;
    for review in &reviews {
        validate_authorization_review(review, snapshot)?;
    }
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
        return Err("historical outcome dry-run authorization chain has invalid tips".to_string());
    }
    let tip_id = tips[0].review_id.clone();
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("historical outcome dry-run authorization chain has a cycle".to_string());
        }
        cursor = match review.previous_review_id.as_ref() {
            Some(id) => Some(*by_id.get(id).ok_or_else(|| {
                "historical outcome dry-run authorization chain is disconnected".to_string()
            })?),
            None => None,
        };
    }
    if visited.len() != reviews.len() {
        return Err("historical outcome dry-run authorization chain is disconnected".to_string());
    }
    Ok(reviews.drain(..).find(|review| review.review_id == tip_id))
}

async fn fetch_adjusted_history_value(
    state: &AppState,
    keys: &[String],
    symbol: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<serde_json::Value, String> {
    if !valid_symbol(symbol) {
        return Err("历史行情证券代码无效".to_string());
    }
    let base = fmp_legacy_base(&state.core.config.fmp.base_url);
    let encoded_symbol = utf8_percent_encode(symbol, NON_ALPHANUMERIC).to_string();
    let mut last_error = "FMP 历史行情不可用".to_string();
    for key in keys {
        let encoded_key = utf8_percent_encode(key, NON_ALPHANUMERIC).to_string();
        let url = format!(
            "{base}/v3/historical-price-full/{encoded_symbol}?from={from}&to={to}&apikey={encoded_key}"
        );
        match super::public_finance_calendar::fetch_fmp_json_once(
            &state.http_client,
            &url,
            state.core.config.fmp.timeout,
        )
        .await
        {
            Ok(value) => {
                adjusted_points_from_value(&value, from, to)?;
                return Ok(value);
            }
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

fn adjusted_points_from_value(
    value: &serde_json::Value,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<SealedAdjustedClosePoint>, String> {
    let rows = value
        .get("historical")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "FMP 历史行情缺少 historical 数组".to_string())?;
    let mut by_date = BTreeMap::new();
    for row in rows {
        let date = row
            .get("date")
            .and_then(serde_json::Value::as_str)
            .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
            .ok_or_else(|| "FMP 历史行情包含无效交易日".to_string())?;
        let adjusted_close = row
            .get("adjClose")
            .and_then(serde_json::Value::as_f64)
            .filter(|value| value.is_finite() && *value > 0.0)
            .ok_or_else(|| "FMP 历史行情缺少有效复权收盘价".to_string())?;
        if date < from || date > to {
            return Err("FMP 历史行情返回了请求区间外的数据".to_string());
        }
        if let Some(previous) = by_date.insert(date, adjusted_close) {
            if previous != adjusted_close {
                return Err("FMP 历史行情同一交易日存在冲突复权价".to_string());
            }
            return Err("FMP 历史行情同一交易日重复".to_string());
        }
    }
    if by_date.is_empty() {
        return Err("FMP 历史行情没有可封存的复权收盘价".to_string());
    }
    Ok(by_date
        .into_iter()
        .map(|(date, adjusted_close)| SealedAdjustedClosePoint {
            date,
            adjusted_close,
        })
        .collect())
}

fn common_session_count(
    asset: &[SealedAdjustedClosePoint],
    benchmark: &[SealedAdjustedClosePoint],
) -> usize {
    let benchmark_dates = benchmark
        .iter()
        .map(|point| point.date)
        .collect::<BTreeSet<_>>();
    asset
        .iter()
        .filter(|point| benchmark_dates.contains(&point.date))
        .count()
}

fn validate_snapshot(snapshot: &HistoricalOutcomePriceSnapshot) -> Result<(), String> {
    let protocol = outcome_protocol();
    let common_count = common_session_count(&snapshot.asset_points, &snapshot.benchmark_points);
    let covered = protocol
        .horizons_market_sessions
        .iter()
        .copied()
        .filter(|horizon| common_count > usize::from(*horizon))
        .collect::<Vec<_>>();
    if snapshot.schema_version != SNAPSHOT_SCHEMA_VERSION
        || snapshot.ingestion_policy_version != INGESTION_POLICY_VERSION
        || !valid_id(&snapshot.snapshot_id)
        || !valid_sha256(&snapshot.snapshot_sha256)
        || snapshot.snapshot_id != snapshot.snapshot_sha256[..32]
        || snapshot.snapshot_sha256 != snapshot_fingerprint(snapshot)?
        || snapshot.sealed_by.trim().is_empty()
        || !valid_id(&snapshot.reconstruction_id)
        || !valid_sha256(&snapshot.reconstruction_sha256)
        || !valid_id(&snapshot.reconstruction_review_id)
        || !valid_id(&snapshot.anchor_candidate_id)
        || !valid_id(&snapshot.anchor_review_id)
        || snapshot.decision_available_at > snapshot.sealed_at
        || !valid_id(&snapshot.implementation_id)
        || !valid_sha256(&snapshot.implementation_spec_sha256)
        || !valid_id(&snapshot.implementation_review_id)
        || !valid_id(&snapshot.governance_review_id)
        || snapshot.protocol_version != protocol.protocol_version
        || snapshot.protocol_sha256 != protocol_sha256()?
        || snapshot.provider != PROVIDER_NAME
        || snapshot.provider_endpoint_template != PROVIDER_ENDPOINT_TEMPLATE
        || snapshot.price_basis != PRICE_BASIS
        || !valid_symbol(&snapshot.asset_symbol)
        || snapshot.benchmark_symbol != protocol.benchmark_symbol
        || snapshot.requested_from > snapshot.requested_to
        || !valid_sha256(&snapshot.asset_payload_sha256)
        || !valid_sha256(&snapshot.benchmark_payload_sha256)
        || snapshot.asset_payload_sha256
            != sealed_payload_sha256(
                &snapshot.asset_symbol,
                snapshot.requested_from,
                snapshot.requested_to,
                &snapshot.asset_points,
            )?
        || snapshot.benchmark_payload_sha256
            != sealed_payload_sha256(
                &snapshot.benchmark_symbol,
                snapshot.requested_from,
                snapshot.requested_to,
                &snapshot.benchmark_points,
            )?
        || snapshot.asset_series_sha256 != hash_serializable(&snapshot.asset_points)?
        || snapshot.benchmark_series_sha256 != hash_serializable(&snapshot.benchmark_points)?
        || snapshot.asset_points.iter().any(|point| {
            point.date < snapshot.requested_from
                || point.date > snapshot.requested_to
                || !point.adjusted_close.is_finite()
                || point.adjusted_close <= 0.0
        })
        || snapshot.benchmark_points.iter().any(|point| {
            point.date < snapshot.requested_from
                || point.date > snapshot.requested_to
                || !point.adjusted_close.is_finite()
                || point.adjusted_close <= 0.0
        })
        || !strictly_sorted_unique(&snapshot.asset_points)
        || !strictly_sorted_unique(&snapshot.benchmark_points)
        || snapshot.common_session_count != common_count
        || snapshot.covered_horizons_market_sessions != covered
        || snapshot.all_protocol_horizons_covered
            != (snapshot.covered_horizons_market_sessions == protocol.horizons_market_sessions)
        || snapshot.outcome_metrics_computed
        || snapshot.label_written
        || snapshot.historical_state_mutated
        || snapshot.decision_training_authorized
        || snapshot.reward_evidence_authorized
        || snapshot.shadow_evidence_authorized
        || snapshot.trading_authorized
    {
        return Err("historical outcome price snapshot is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn strictly_sorted_unique(points: &[SealedAdjustedClosePoint]) -> bool {
    !points.is_empty() && points.windows(2).all(|pair| pair[0].date < pair[1].date)
}

fn snapshot_fingerprint(snapshot: &HistoricalOutcomePriceSnapshot) -> Result<String, String> {
    hash_serializable(&serde_json::json!({
        "schema_version": snapshot.schema_version,
        "ingestion_policy_version": snapshot.ingestion_policy_version,
        "sealed_at": snapshot.sealed_at,
        "sealed_by": snapshot.sealed_by,
        "reconstruction_id": snapshot.reconstruction_id,
        "reconstruction_sha256": snapshot.reconstruction_sha256,
        "reconstruction_review_id": snapshot.reconstruction_review_id,
        "anchor_candidate_id": snapshot.anchor_candidate_id,
        "anchor_review_id": snapshot.anchor_review_id,
        "decision_available_at": snapshot.decision_available_at,
        "implementation_id": snapshot.implementation_id,
        "implementation_spec_sha256": snapshot.implementation_spec_sha256,
        "implementation_review_id": snapshot.implementation_review_id,
        "governance_review_id": snapshot.governance_review_id,
        "protocol_version": snapshot.protocol_version,
        "protocol_sha256": snapshot.protocol_sha256,
        "code_revision": snapshot.code_revision,
        "provider": snapshot.provider,
        "provider_endpoint_template": snapshot.provider_endpoint_template,
        "price_basis": snapshot.price_basis,
        "asset_symbol": snapshot.asset_symbol,
        "benchmark_symbol": snapshot.benchmark_symbol,
        "requested_from": snapshot.requested_from,
        "requested_to": snapshot.requested_to,
        "asset_payload_sha256": snapshot.asset_payload_sha256,
        "benchmark_payload_sha256": snapshot.benchmark_payload_sha256,
        "asset_series_sha256": snapshot.asset_series_sha256,
        "benchmark_series_sha256": snapshot.benchmark_series_sha256,
        "asset_points": snapshot.asset_points,
        "benchmark_points": snapshot.benchmark_points,
        "common_session_count": snapshot.common_session_count,
        "covered_horizons_market_sessions": snapshot.covered_horizons_market_sessions,
        "all_protocol_horizons_covered": snapshot.all_protocol_horizons_covered,
        "outcome_metrics_computed": snapshot.outcome_metrics_computed,
        "label_written": snapshot.label_written,
        "historical_state_mutated": snapshot.historical_state_mutated,
        "decision_training_authorized": snapshot.decision_training_authorized,
        "reward_evidence_authorized": snapshot.reward_evidence_authorized,
        "shadow_evidence_authorized": snapshot.shadow_evidence_authorized,
        "trading_authorized": snapshot.trading_authorized,
    }))
}

fn benchmark_state_binding_matches(
    snapshot: &HistoricalOutcomePriceSnapshot,
    current: &ApprovedHistoricalBenchmarkState,
) -> bool {
    snapshot.reconstruction_sha256 == current.reconstruction_sha256
        && snapshot.reconstruction_review_id == current.reconstruction_review_id
        && snapshot.anchor_candidate_id == current.anchor_candidate_id
        && snapshot.anchor_review_id == current.anchor_review_id
        && snapshot.asset_symbol == current.symbol
        && snapshot.decision_available_at == current.decision_available_at
}

fn labeler_binding_matches(
    snapshot: &HistoricalOutcomePriceSnapshot,
    current: &ApprovedHistoricalOutcomeLabeler,
) -> bool {
    snapshot.implementation_spec_sha256 == current.implementation_spec_sha256
        && snapshot.implementation_review_id == current.implementation_review_id
        && snapshot.governance_review_id == current.governance_review_id
        && snapshot.protocol_version == current.protocol_version
        && snapshot.protocol_sha256 == current.protocol_sha256
        && snapshot.code_revision == current.code_revision
}

fn protocol_sha256() -> Result<String, String> {
    protocol_fingerprint(&outcome_protocol())
}

fn sealed_payload_sha256(
    symbol: &str,
    requested_from: NaiveDate,
    requested_to: NaiveDate,
    points: &[SealedAdjustedClosePoint],
) -> Result<String, String> {
    hash_serializable(&serde_json::json!({
        "symbol": symbol,
        "requested_from": requested_from,
        "requested_to": requested_to,
        "price_basis": PRICE_BASIS,
        "points": points,
    }))
}

fn latest_completed_new_york_market_date(now: DateTime<Utc>) -> NaiveDate {
    let local = now.with_timezone(&New_York);
    if local.hour() >= 16 {
        local.date_naive()
    } else {
        local.date_naive() - Duration::days(1)
    }
}

fn earliest_outcome_start_date(decision_at: DateTime<Utc>) -> NaiveDate {
    let local = decision_at.with_timezone(&New_York);
    if local.hour() < 16 {
        local.date_naive()
    } else {
        local.date_naive() + Duration::days(1)
    }
}

fn fmp_legacy_base(raw: &str) -> String {
    raw.trim()
        .trim_end_matches('/')
        .trim_end_matches("/v3")
        .trim_end_matches("/stable")
        .trim_end_matches('/')
        .to_string()
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

async fn read_snapshots(state: &AppState) -> Result<Vec<HistoricalOutcomePriceSnapshot>, String> {
    let root = snapshot_root(state);
    let mut directories = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut fingerprints = BTreeSet::new();
    while let Some(entry) = directories
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if !entry
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        for snapshot in read_json_records::<HistoricalOutcomePriceSnapshot>(&entry.path()).await? {
            validate_snapshot(&snapshot)?;
            if !fingerprints.insert(snapshot.snapshot_sha256.clone()) {
                return Err(
                    "historical outcome price snapshot registry contains duplicates".to_string(),
                );
            }
            output.push(snapshot);
        }
    }
    Ok(output)
}

async fn read_json_records<T>(directory: &Path) -> Result<Vec<T>, String>
where
    T: for<'de> Deserialize<'de>,
{
    let mut output = Vec::new();
    let mut entries = match tokio::fs::read_dir(directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(output),
        Err(error) => return Err(error.to_string()),
    };
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
        output.push(serde_json::from_slice(&bytes).map_err(|error| error.to_string())?);
    }
    Ok(output)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn snapshot_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-price-snapshots")
}

fn snapshot_directory(state: &AppState, reconstruction_id: &str) -> PathBuf {
    snapshot_root(state).join(reconstruction_id)
}

fn authorization_review_directory(
    state: &AppState,
    snapshot: &HistoricalOutcomePriceSnapshot,
) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-dry-run-authorizations")
        .join(&snapshot.implementation_id)
        .join(&snapshot.snapshot_id)
}

struct HistoricalOutcomeSnapshotLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeSnapshotLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<HistoricalOutcomeSnapshotLock, String> {
    if !valid_id(name) {
        return Err("invalid historical outcome snapshot lock name".to_string());
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
            Ok(_) => return Ok(HistoricalOutcomeSnapshotLock { path }),
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
                return Err("历史结果行情或授权正在由另一请求变更，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("历史结果行情锁获取失败".to_string())
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

    fn points(count: usize, start: NaiveDate, offset: f64) -> Vec<SealedAdjustedClosePoint> {
        (0..count)
            .map(|index| SealedAdjustedClosePoint {
                date: start + Duration::days(index as i64),
                adjusted_close: offset + index as f64,
            })
            .collect()
    }

    fn snapshot(common_count: usize) -> HistoricalOutcomePriceSnapshot {
        let protocol = outcome_protocol();
        let start = NaiveDate::from_ymd_opt(2025, 1, 2).unwrap();
        let asset_points = points(common_count, start, 100.0);
        let benchmark_points = points(common_count, start, 500.0);
        let covered_horizons_market_sessions = protocol
            .horizons_market_sessions
            .iter()
            .copied()
            .filter(|horizon| common_count > usize::from(*horizon))
            .collect::<Vec<_>>();
        let mut value = HistoricalOutcomePriceSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION.to_string(),
            ingestion_policy_version: INGESTION_POLICY_VERSION.to_string(),
            snapshot_id: String::new(),
            snapshot_sha256: String::new(),
            sealed_at: DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            sealed_by: "admin".to_string(),
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "a".repeat(64),
            reconstruction_review_id: "reconstruction-review-1".to_string(),
            anchor_candidate_id: "anchor-1".to_string(),
            anchor_review_id: "anchor-review-1".to_string(),
            decision_available_at: DateTime::parse_from_rfc3339("2025-01-01T20:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            implementation_id: "labeler-1".to_string(),
            implementation_spec_sha256: "b".repeat(64),
            implementation_review_id: "labeler-review-1".to_string(),
            governance_review_id: "governance-review-1".to_string(),
            protocol_version: protocol.protocol_version,
            protocol_sha256: protocol_sha256().unwrap(),
            code_revision: "oldwang@abc123".to_string(),
            provider: PROVIDER_NAME.to_string(),
            provider_endpoint_template: PROVIDER_ENDPOINT_TEMPLATE.to_string(),
            price_basis: PRICE_BASIS.to_string(),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: protocol.benchmark_symbol,
            requested_from: start,
            requested_to: start + Duration::days(common_count as i64),
            asset_payload_sha256: sealed_payload_sha256(
                "SNDK",
                start,
                start + Duration::days(common_count as i64),
                &asset_points,
            )
            .unwrap(),
            benchmark_payload_sha256: sealed_payload_sha256(
                "SPY",
                start,
                start + Duration::days(common_count as i64),
                &benchmark_points,
            )
            .unwrap(),
            asset_series_sha256: hash_serializable(&asset_points).unwrap(),
            benchmark_series_sha256: hash_serializable(&benchmark_points).unwrap(),
            asset_points,
            benchmark_points,
            common_session_count: common_count,
            all_protocol_horizons_covered: covered_horizons_market_sessions
                == protocol.horizons_market_sessions,
            covered_horizons_market_sessions,
            outcome_metrics_computed: false,
            label_written: false,
            historical_state_mutated: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        };
        value.snapshot_sha256 = snapshot_fingerprint(&value).unwrap();
        value.snapshot_id = value.snapshot_sha256[..32].to_string();
        value
    }

    #[test]
    fn parser_requires_adjusted_close_and_rejects_duplicate_dates() {
        let start = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let missing = serde_json::json!({"historical": [{"date":"2026-08-01","close":10.0}]});
        assert!(adjusted_points_from_value(&missing, start, start).is_err());
        let duplicate = serde_json::json!({"historical": [
            {"date":"2026-08-01","adjClose":10.0},
            {"date":"2026-08-01","adjClose":10.0}
        ]});
        assert!(adjusted_points_from_value(&duplicate, start, start).is_err());
    }

    #[test]
    fn snapshot_is_sealed_without_computing_any_outcome() {
        let value = snapshot(251);
        assert!(validate_snapshot(&value).is_ok());
        assert!(value.all_protocol_horizons_covered);
        assert!(!value.outcome_metrics_computed);
        assert!(!value.label_written);
        assert!(!value.decision_training_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn sealed_payload_hash_binds_symbol_range_basis_and_points() {
        let mut value = snapshot(251);
        value.requested_to += Duration::days(1);
        value.snapshot_sha256 = snapshot_fingerprint(&value).unwrap();
        value.snapshot_id = value.snapshot_sha256[..32].to_string();
        assert!(validate_snapshot(&value).is_err());
    }

    #[test]
    fn snapshot_requires_all_protocol_horizons_before_authorization_review() {
        let value = snapshot(61);
        assert!(validate_snapshot(&value).is_ok());
        assert_eq!(value.covered_horizons_market_sessions, vec![20, 60]);
        assert!(!value.all_protocol_horizons_covered);
    }

    #[test]
    fn authorization_approval_opens_only_later_registration() {
        let value = snapshot(251);
        let review = HistoricalOutcomeDryRunAuthorizationReview {
            schema_version: AUTHORIZATION_REVIEW_SCHEMA_VERSION.to_string(),
            authorization_policy_version: AUTHORIZATION_POLICY_VERSION.to_string(),
            review_id: "authorization-review-1".to_string(),
            previous_review_id: None,
            snapshot_id: value.snapshot_id.clone(),
            snapshot_sha256: value.snapshot_sha256.clone(),
            reconstruction_id: value.reconstruction_id.clone(),
            reconstruction_sha256: value.reconstruction_sha256.clone(),
            reconstruction_review_id: value.reconstruction_review_id.clone(),
            implementation_id: value.implementation_id.clone(),
            implementation_spec_sha256: value.implementation_spec_sha256.clone(),
            implementation_review_id: value.implementation_review_id.clone(),
            protocol_version: value.protocol_version.clone(),
            protocol_sha256: value.protocol_sha256.clone(),
            submitted_at: Utc::now(),
            reviewer_id: "admin".to_string(),
            verdict: HistoricalOutcomeDryRunAuthorizationVerdict::ApprovedForDryRunImplementationRegistration,
            rationale: "逐项核对封存来源、覆盖和隔离边界。".to_string(),
            current_bindings_confirmed: true,
            sealed_snapshot_integrity_confirmed: true,
            provider_provenance_confirmed: true,
            complete_common_session_coverage_confirmed: true,
            deterministic_fixture_confirmed: true,
            isolated_output_confirmed: true,
            no_label_or_production_writes_confirmed: true,
            dry_run_implementation_registration_eligible: true,
            offline_dry_run_enabled: false,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        };
        assert!(validate_authorization_review(&review, &value).is_ok());
        assert!(review.dry_run_implementation_registration_eligible);
        assert!(!review.offline_dry_run_enabled);
        assert!(!review.outcome_label_generation_enabled);
    }
}
