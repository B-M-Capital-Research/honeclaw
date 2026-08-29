//! Human-reviewed point-in-time state reconstructions for historical anchors.
//!
//! A confirmed sentence still does not prove what information was available at
//! that time.  This module therefore freezes a second, source-bound state with
//! seven explicit components.  Missing evidence is retained as missing; later
//! documents are rejected.  Approved reconstructions remain benchmark-only and
//! do not yet trigger outcome labelling, training, reward, shadow or trading.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use chrono_tz::Asia::Shanghai;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::historical_decision_anchors::{
    ConfirmedHistoricalAnchorProjection, HistoricalAnchorAction,
    confirmed_anchors_for_reconstruction,
};
use super::research_library::verifiable_global_text_source;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-state-reconstruction-registry-v1";
const CANDIDATE_SCHEMA_VERSION: &str = "hone-historical-state-reconstruction-candidate-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-historical-state-reconstruction-review-v1";
const COMPONENT_POLICY_VERSION: &str = "hone-historical-state-components-v1-seven-layer";
const OUTCOME_PROTOCOL_VERSION: &str = "hone-historical-benchmark-outcome-protocol-v1";
const MAX_LOCATOR_CHARS: usize = 160;
const MAX_EXCERPT_CHARS: usize = 2_400;
const MAX_CLAIM_CHARS: usize = 1_000;
const MAX_REASON_CHARS: usize = 1_000;
const REQUIRED_COMPONENTS: [HistoricalStateComponentId; 7] = [
    HistoricalStateComponentId::IndustryThesis,
    HistoricalStateComponentId::CompanyFundamentals,
    HistoricalStateComponentId::FinancialVerification,
    HistoricalStateComponentId::Valuation,
    HistoricalStateComponentId::Crowding,
    HistoricalStateComponentId::MarketRegime,
    HistoricalStateComponentId::PortfolioContext,
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalStateComponentId {
    IndustryThesis,
    CompanyFundamentals,
    FinancialVerification,
    Valuation,
    Crowding,
    MarketRegime,
    PortfolioContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalStateComponentStatus {
    EvidenceBacked,
    ExplicitlyMissing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalStateReviewVerdict {
    ApprovedForBenchmark,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalStateEvidence {
    evidence_sha256: String,
    source_item_id: String,
    source_sha256: String,
    source_title: String,
    source_name: String,
    source_date: String,
    claimed_available_at: DateTime<Utc>,
    source_locator: String,
    verbatim_excerpt: String,
    normalized_claim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalStateComponent {
    component_id: HistoricalStateComponentId,
    status: HistoricalStateComponentStatus,
    evidence: Vec<HistoricalStateEvidence>,
    missing_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalStateReconstructionCandidate {
    schema_version: String,
    reconstruction_id: String,
    reconstruction_sha256: String,
    component_policy_version: String,
    anchor_candidate_id: String,
    anchor_candidate_sha256: String,
    anchor_review_id: String,
    symbol: String,
    anchor_action: HistoricalAnchorAction,
    anchor_thesis: String,
    decision_available_at: DateTime<Utc>,
    components: Vec<HistoricalStateComponent>,
    created_at: DateTime<Utc>,
    created_by: String,
    human_review_status: String,
    benchmark_state_eligible: bool,
    outcome_labeling_eligible: bool,
    decision_training_eligible: bool,
    reward_evidence_eligible: bool,
    shadow_evidence_eligible: bool,
    trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalStateReconstructionReview {
    schema_version: String,
    review_id: String,
    previous_review_id: Option<String>,
    reconstruction_id: String,
    reconstruction_sha256: String,
    submitted_at: DateTime<Utc>,
    reviewer_id: String,
    verdict: HistoricalStateReviewVerdict,
    review_statement: String,
    anchor_binding_confirmed: bool,
    source_bytes_confirmed: bool,
    availability_times_confirmed: bool,
    no_future_information_confirmed: bool,
    missingness_preserved_confirmed: bool,
    component_interpretations_confirmed: bool,
    benchmark_state_eligible: bool,
    outcome_labeling_eligible: bool,
    decision_training_eligible: bool,
    reward_evidence_eligible: bool,
    shadow_evidence_eligible: bool,
    trading_authorized: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HistoricalStateEvidenceRequest {
    source_item_id: String,
    expected_source_sha256: String,
    claimed_available_at: DateTime<Utc>,
    source_locator: String,
    verbatim_excerpt: String,
    normalized_claim: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct HistoricalStateComponentRequest {
    component_id: HistoricalStateComponentId,
    status: HistoricalStateComponentStatus,
    #[serde(default)]
    evidence: Vec<HistoricalStateEvidenceRequest>,
    missing_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateHistoricalStateReconstructionRequest {
    anchor_candidate_id: String,
    expected_anchor_candidate_sha256: String,
    expected_anchor_review_id: String,
    components: Vec<HistoricalStateComponentRequest>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalStateReconstructionRequest {
    expected_review_id: Option<String>,
    verdict: HistoricalStateReviewVerdict,
    review_statement: String,
    anchor_binding_confirmed: bool,
    source_bytes_confirmed: bool,
    availability_times_confirmed: bool,
    no_future_information_confirmed: bool,
    missingness_preserved_confirmed: bool,
    component_interpretations_confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ConfirmedAnchorSummary {
    candidate_id: String,
    candidate_sha256: String,
    review_id: String,
    symbol: String,
    final_action: HistoricalAnchorAction,
    final_thesis: String,
    decision_available_at: DateTime<Utc>,
}

impl From<&ConfirmedHistoricalAnchorProjection> for ConfirmedAnchorSummary {
    fn from(value: &ConfirmedHistoricalAnchorProjection) -> Self {
        Self {
            candidate_id: value.candidate_id.clone(),
            candidate_sha256: value.candidate_sha256.clone(),
            review_id: value.review_id.clone(),
            symbol: value.symbol.clone(),
            final_action: value.final_action.clone(),
            final_thesis: value.final_thesis.clone(),
            decision_available_at: value.decision_available_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct RequiredComponentSummary {
    component_id: HistoricalStateComponentId,
    label: &'static str,
    requirement: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeProtocol {
    pub(crate) protocol_version: String,
    pub(crate) horizons_market_sessions: Vec<u16>,
    pub(crate) asset_price_basis: String,
    pub(crate) benchmark_symbol: String,
    pub(crate) benchmark_price_basis: String,
    pub(crate) start_rule: String,
    pub(crate) metrics: Vec<String>,
    pub(crate) missing_session_rule: String,
    pub(crate) future_information_rule: String,
    pub(crate) automatic_labeling_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ReconstructionRegistryItem {
    candidate: HistoricalStateReconstructionCandidate,
    latest_review: Option<HistoricalStateReconstructionReview>,
    anchor_binding_current: bool,
    benchmark_state_ready: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalStateReconstructionRegistry {
    schema_version: String,
    component_policy_version: String,
    outcome_protocol: HistoricalOutcomeProtocol,
    confirmed_anchor_count: usize,
    reconstruction_candidate_count: usize,
    benchmark_ready_count: usize,
    stale_reconstruction_count: usize,
    confirmed_anchors: Vec<ConfirmedAnchorSummary>,
    required_components: Vec<RequiredComponentSummary>,
    reconstructions: Vec<ReconstructionRegistryItem>,
    state_reconstruction_status: String,
    automatic_reconstruction_authorized: bool,
    benchmark_outcome_labeling_enabled: bool,
    decision_training_authorized: bool,
    reward_evidence_authorized: bool,
    shadow_evidence_authorized: bool,
    trading_authorized: bool,
    scope: String,
}

pub(crate) async fn handle_get_historical_state_reconstructions(
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
            warn!(%error, "historical state reconstruction registry read failed");
            crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, "历史点时状态暂时不可用")
        }
    }
}

pub(crate) async fn handle_create_historical_state_reconstruction(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<CreateHistoricalStateReconstructionRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match create_candidate(&state, &admin.user_id, request).await {
        Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn handle_review_historical_state_reconstruction(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(reconstruction_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalStateReconstructionRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_candidate(&state, &admin.user_id, &reconstruction_id, request).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

async fn build_registry(state: &AppState) -> Result<HistoricalStateReconstructionRegistry, String> {
    let anchors = confirmed_anchors_for_reconstruction(state).await?;
    let anchor_index = anchors
        .iter()
        .map(|anchor| (anchor.candidate_id.clone(), anchor))
        .collect::<BTreeMap<_, _>>();
    let candidates = read_candidates(state).await?;
    let reviews = read_reviews(state).await?;
    let mut reconstructions = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        validate_candidate_sources(state, &candidate)?;
        let current_anchor = anchor_index.get(&candidate.anchor_candidate_id);
        let anchor_binding_current = current_anchor.is_some_and(|anchor| {
            anchor.candidate_sha256 == candidate.anchor_candidate_sha256
                && anchor.review_id == candidate.anchor_review_id
                && anchor.symbol == candidate.symbol
                && anchor.decision_available_at == candidate.decision_available_at
                && anchor.final_action == candidate.anchor_action
                && anchor.final_thesis == candidate.anchor_thesis
        });
        let latest_review = reviews.get(&candidate.reconstruction_id).cloned();
        if let Some(review) = latest_review.as_ref() {
            validate_candidate_review_binding(&candidate, review)?;
        }
        let benchmark_state_ready = anchor_binding_current
            && latest_review.as_ref().is_some_and(|review| {
                review.verdict == HistoricalStateReviewVerdict::ApprovedForBenchmark
                    && review.benchmark_state_eligible
            });
        reconstructions.push(ReconstructionRegistryItem {
            candidate,
            latest_review,
            anchor_binding_current,
            benchmark_state_ready,
        });
    }
    reconstructions.sort_by(|left, right| {
        left.candidate
            .decision_available_at
            .cmp(&right.candidate.decision_available_at)
            .then_with(|| left.candidate.symbol.cmp(&right.candidate.symbol))
            .then_with(|| {
                left.candidate
                    .reconstruction_id
                    .cmp(&right.candidate.reconstruction_id)
            })
    });
    let benchmark_ready_count = reconstructions
        .iter()
        .filter(|item| item.benchmark_state_ready)
        .count();
    let stale_reconstruction_count = reconstructions
        .iter()
        .filter(|item| !item.anchor_binding_current)
        .count();
    let state_reconstruction_status = if anchors.is_empty() {
        "waiting_for_confirmed_anchor"
    } else if benchmark_ready_count == 0 {
        "waiting_for_human_reconstruction_review"
    } else {
        "benchmark_states_ready_outcome_labeler_not_enabled"
    };
    Ok(HistoricalStateReconstructionRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        component_policy_version: COMPONENT_POLICY_VERSION.to_string(),
        outcome_protocol: outcome_protocol(),
        confirmed_anchor_count: anchors.len(),
        reconstruction_candidate_count: reconstructions.len(),
        benchmark_ready_count,
        stale_reconstruction_count,
        confirmed_anchors: anchors.iter().map(ConfirmedAnchorSummary::from).collect(),
        required_components: required_component_summaries(),
        reconstructions,
        state_reconstruction_status: state_reconstruction_status.to_string(),
        automatic_reconstruction_authorized: false,
        benchmark_outcome_labeling_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
        scope: "点时重建必须逐层保存当时已知证据或明确缺失；任何晚于判断可用时间的资料都会被拒绝。人工批准只形成历史基准状态，结果标签、训练、奖励、影子与交易仍关闭。".to_string(),
    })
}

pub(crate) async fn benchmark_ready_count_for_outcome_governance(
    state: &AppState,
) -> Result<usize, String> {
    Ok(build_registry(state).await?.benchmark_ready_count)
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalBenchmarkState {
    pub reconstruction_id: String,
    pub reconstruction_sha256: String,
    pub reconstruction_review_id: String,
    pub anchor_candidate_id: String,
    pub anchor_review_id: String,
    pub symbol: String,
    pub decision_available_at: DateTime<Utc>,
}

/// Returns only benchmark states whose source bytes, anchor binding and latest
/// human review are still current.  Future-price ingestion must bind to this
/// projection rather than trusting a reconstruction ID supplied by a client.
pub(crate) async fn approved_historical_benchmark_states(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalBenchmarkState>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .reconstructions
        .into_iter()
        .filter(|item| item.benchmark_state_ready)
        .filter_map(|item| {
            item.latest_review
                .map(|review| ApprovedHistoricalBenchmarkState {
                    reconstruction_id: item.candidate.reconstruction_id,
                    reconstruction_sha256: item.candidate.reconstruction_sha256,
                    reconstruction_review_id: review.review_id,
                    anchor_candidate_id: item.candidate.anchor_candidate_id,
                    anchor_review_id: item.candidate.anchor_review_id,
                    symbol: item.candidate.symbol,
                    decision_available_at: item.candidate.decision_available_at,
                })
        })
        .collect())
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalStateReadinessSummary {
    pub confirmed_anchor_count: usize,
    pub reconstruction_candidate_count: usize,
    pub benchmark_ready_count: usize,
    pub stale_reconstruction_count: usize,
    pub state_reconstruction_status: String,
}

pub(crate) async fn historical_state_readiness_summary(
    state: &AppState,
) -> Result<HistoricalStateReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalStateReadinessSummary {
        confirmed_anchor_count: registry.confirmed_anchor_count,
        reconstruction_candidate_count: registry.reconstruction_candidate_count,
        benchmark_ready_count: registry.benchmark_ready_count,
        stale_reconstruction_count: registry.stale_reconstruction_count,
        state_reconstruction_status: registry.state_reconstruction_status,
    })
}

async fn create_candidate(
    state: &AppState,
    admin_id: &str,
    request: CreateHistoricalStateReconstructionRequest,
) -> Result<HistoricalStateReconstructionCandidate, String> {
    if !valid_id(&request.anchor_candidate_id)
        || !valid_id(&request.expected_anchor_review_id)
        || !valid_sha256(&request.expected_anchor_candidate_sha256)
    {
        return Err("历史锚点绑定无效".to_string());
    }
    let _lock = acquire_lock(state, "candidate-registry").await?;
    let anchor = confirmed_anchors_for_reconstruction(state)
        .await?
        .into_iter()
        .find(|anchor| anchor.candidate_id == request.anchor_candidate_id)
        .ok_or_else(|| "当前没有这条已确认历史锚点".to_string())?;
    if anchor.candidate_sha256 != request.expected_anchor_candidate_sha256
        || anchor.review_id != request.expected_anchor_review_id
    {
        return Err("历史锚点已经变化，请刷新后重建".to_string());
    }
    let components = validate_component_requests(state, &anchor, request.components)?;
    let reconstruction_sha256 = reconstruction_fingerprint(&anchor, &components)?;
    if let Some(existing) = read_candidates(state)
        .await?
        .into_iter()
        .find(|candidate| candidate.reconstruction_sha256 == reconstruction_sha256)
    {
        return Ok(existing);
    }
    let candidate = HistoricalStateReconstructionCandidate {
        schema_version: CANDIDATE_SCHEMA_VERSION.to_string(),
        reconstruction_id: reconstruction_sha256[..32].to_string(),
        reconstruction_sha256,
        component_policy_version: COMPONENT_POLICY_VERSION.to_string(),
        anchor_candidate_id: anchor.candidate_id,
        anchor_candidate_sha256: anchor.candidate_sha256,
        anchor_review_id: anchor.review_id,
        symbol: anchor.symbol,
        anchor_action: anchor.final_action,
        anchor_thesis: anchor.final_thesis,
        decision_available_at: anchor.decision_available_at,
        components,
        created_at: Utc::now(),
        created_by: admin_id.to_string(),
        human_review_status: "pending".to_string(),
        benchmark_state_eligible: false,
        outcome_labeling_eligible: false,
        decision_training_eligible: false,
        reward_evidence_eligible: false,
        shadow_evidence_eligible: false,
        trading_authorized: false,
    };
    validate_candidate(&candidate)?;
    write_immutable_json(
        &candidate_directory(state).join(format!("{}.json", candidate.reconstruction_id)),
        &candidate,
    )
    .await?;
    Ok(candidate)
}

async fn review_candidate(
    state: &AppState,
    admin_id: &str,
    reconstruction_id: &str,
    request: ReviewHistoricalStateReconstructionRequest,
) -> Result<HistoricalStateReconstructionReview, String> {
    if !valid_id(reconstruction_id) {
        return Err("点时重建编号无效".to_string());
    }
    let _lock = acquire_lock(state, &format!("review-{reconstruction_id}")).await?;
    let candidate = read_candidates(state)
        .await?
        .into_iter()
        .find(|candidate| candidate.reconstruction_id == reconstruction_id)
        .ok_or_else(|| "点时重建候选不存在".to_string())?;
    validate_candidate_sources(state, &candidate)?;
    let anchor = confirmed_anchors_for_reconstruction(state)
        .await?
        .into_iter()
        .find(|anchor| anchor.candidate_id == candidate.anchor_candidate_id)
        .ok_or_else(|| "上游历史锚点已经失效".to_string())?;
    if anchor.candidate_sha256 != candidate.anchor_candidate_sha256
        || anchor.review_id != candidate.anchor_review_id
        || anchor.symbol != candidate.symbol
        || anchor.final_action != candidate.anchor_action
        || anchor.final_thesis != candidate.anchor_thesis
        || anchor.decision_available_at != candidate.decision_available_at
    {
        return Err("上游历史锚点已经变化，请重新建立点时重建".to_string());
    }
    let reviews = read_reviews(state).await?;
    let previous = reviews.get(reconstruction_id);
    if request.expected_review_id.as_deref() != previous.map(|review| review.review_id.as_str()) {
        return Err("点时重建复核已经变化，请刷新后重试".to_string());
    }
    let review_statement = bounded_required(&request.review_statement, 1_200, "复核说明")?;
    let approved = request.verdict == HistoricalStateReviewVerdict::ApprovedForBenchmark;
    let all_checks = request.anchor_binding_confirmed
        && request.source_bytes_confirmed
        && request.availability_times_confirmed
        && request.no_future_information_confirmed
        && request.missingness_preserved_confirmed
        && request.component_interpretations_confirmed;
    if approved && !all_checks {
        return Err("批准点时重建前必须完成全部来源、时点、缺失和解释检查".to_string());
    }
    let review = HistoricalStateReconstructionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        review_id: Uuid::new_v4().simple().to_string(),
        previous_review_id: previous.map(|review| review.review_id.clone()),
        reconstruction_id: candidate.reconstruction_id,
        reconstruction_sha256: candidate.reconstruction_sha256,
        submitted_at: Utc::now(),
        reviewer_id: admin_id.to_string(),
        verdict: request.verdict,
        review_statement,
        anchor_binding_confirmed: request.anchor_binding_confirmed,
        source_bytes_confirmed: request.source_bytes_confirmed,
        availability_times_confirmed: request.availability_times_confirmed,
        no_future_information_confirmed: request.no_future_information_confirmed,
        missingness_preserved_confirmed: request.missingness_preserved_confirmed,
        component_interpretations_confirmed: request.component_interpretations_confirmed,
        benchmark_state_eligible: approved,
        outcome_labeling_eligible: false,
        decision_training_eligible: false,
        reward_evidence_eligible: false,
        shadow_evidence_eligible: false,
        trading_authorized: false,
    };
    validate_review(&review)?;
    write_immutable_json(
        &review_directory(state, reconstruction_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_component_requests(
    state: &AppState,
    anchor: &ConfirmedHistoricalAnchorProjection,
    requests: Vec<HistoricalStateComponentRequest>,
) -> Result<Vec<HistoricalStateComponent>, String> {
    let mut seen = BTreeSet::new();
    let mut components = Vec::with_capacity(requests.len());
    for request in requests {
        if !seen.insert(request.component_id) {
            return Err("点时状态组件重复".to_string());
        }
        let evidence = request
            .evidence
            .into_iter()
            .map(|item| validate_evidence_request(state, anchor, request.component_id, item))
            .collect::<Result<Vec<_>, _>>()?;
        let missing_reason = match request.status {
            HistoricalStateComponentStatus::EvidenceBacked => {
                if evidence.is_empty() || request.missing_reason.is_some() {
                    return Err("有证据组件必须至少绑定一条证据，且不得填写缺失原因".to_string());
                }
                None
            }
            HistoricalStateComponentStatus::ExplicitlyMissing => {
                if !evidence.is_empty() {
                    return Err("明确缺失组件不得夹带证据".to_string());
                }
                Some(bounded_required(
                    request.missing_reason.as_deref().unwrap_or_default(),
                    MAX_REASON_CHARS,
                    "缺失原因",
                )?)
            }
        };
        components.push(HistoricalStateComponent {
            component_id: request.component_id,
            status: request.status,
            evidence,
            missing_reason,
        });
    }
    let required = REQUIRED_COMPONENTS.into_iter().collect::<BTreeSet<_>>();
    if seen != required {
        return Err("点时重建必须完整覆盖产业、公司、财务、估值、拥挤、宏观和组合七层".to_string());
    }
    components.sort_by_key(|component| component.component_id);
    validate_component_contract(&components)?;
    Ok(components)
}

fn validate_evidence_request(
    state: &AppState,
    anchor: &ConfirmedHistoricalAnchorProjection,
    component_id: HistoricalStateComponentId,
    request: HistoricalStateEvidenceRequest,
) -> Result<HistoricalStateEvidence, String> {
    let source_item_id = request.source_item_id.trim();
    let source_sha256 = request.expected_source_sha256.trim().to_ascii_lowercase();
    let source_locator = bounded_required(&request.source_locator, MAX_LOCATOR_CHARS, "证据定位")?;
    let verbatim_excerpt =
        bounded_required(&request.verbatim_excerpt, MAX_EXCERPT_CHARS, "证据原文")?;
    let normalized_claim =
        bounded_required(&request.normalized_claim, MAX_CLAIM_CHARS, "点时事实归纳")?;
    if source_item_id.is_empty() || !valid_sha256(&source_sha256) {
        return Err("点时证据来源或 SHA-256 无效".to_string());
    }
    let source = verifiable_global_text_source(state, source_item_id)?
        .ok_or_else(|| "点时证据必须来自管理员已批准的完整全局文本".to_string())?;
    if source.sha256 != source_sha256 || !source.content.contains(&verbatim_excerpt) {
        return Err("点时证据原文或文件哈希不匹配".to_string());
    }
    if request.claimed_available_at > anchor.decision_available_at
        || request.claimed_available_at > Utc::now()
        || request
            .claimed_available_at
            .with_timezone(&Shanghai)
            .date_naive()
            .to_string()
            != source.source_date
    {
        return Err("证据可用时间晚于历史判断，或与来源日期不一致".to_string());
    }
    if component_requires_symbol(component_id)
        && !source.tickers.iter().any(|ticker| ticker == &anchor.symbol)
    {
        return Err("公司层点时证据未登记到该公司".to_string());
    }
    let evidence_sha256 = evidence_fingerprint(
        component_id,
        &source.id,
        &source.sha256,
        request.claimed_available_at,
        &source_locator,
        &verbatim_excerpt,
        &normalized_claim,
    )?;
    Ok(HistoricalStateEvidence {
        evidence_sha256,
        source_item_id: source.id,
        source_sha256: source.sha256,
        source_title: source.title,
        source_name: source.source_name,
        source_date: source.source_date,
        claimed_available_at: request.claimed_available_at,
        source_locator,
        verbatim_excerpt,
        normalized_claim,
    })
}

fn validate_component_contract(components: &[HistoricalStateComponent]) -> Result<(), String> {
    let ids = components
        .iter()
        .map(|component| component.component_id)
        .collect::<BTreeSet<_>>();
    if ids != REQUIRED_COMPONENTS.into_iter().collect::<BTreeSet<_>>()
        || components.len() != REQUIRED_COMPONENTS.len()
    {
        return Err("historical state component set is invalid".to_string());
    }
    let mut evidence_ids = BTreeSet::new();
    for component in components {
        let valid_shape = match component.status {
            HistoricalStateComponentStatus::EvidenceBacked => {
                !component.evidence.is_empty() && component.missing_reason.is_none()
            }
            HistoricalStateComponentStatus::ExplicitlyMissing => {
                component.evidence.is_empty()
                    && component
                        .missing_reason
                        .as_deref()
                        .is_some_and(|reason| !reason.trim().is_empty())
            }
        };
        if !valid_shape
            || component
                .evidence
                .iter()
                .any(|evidence| !evidence_ids.insert(evidence.evidence_sha256.clone()))
        {
            return Err("historical state component contract is invalid".to_string());
        }
    }
    Ok(())
}

fn validate_candidate(candidate: &HistoricalStateReconstructionCandidate) -> Result<(), String> {
    if candidate.schema_version != CANDIDATE_SCHEMA_VERSION
        || candidate.component_policy_version != COMPONENT_POLICY_VERSION
        || !valid_id(&candidate.reconstruction_id)
        || !valid_sha256(&candidate.reconstruction_sha256)
        || !valid_id(&candidate.anchor_candidate_id)
        || !valid_id(&candidate.anchor_review_id)
        || !valid_sha256(&candidate.anchor_candidate_sha256)
        || candidate.symbol.trim().is_empty()
        || candidate.decision_available_at > candidate.created_at
        || candidate.human_review_status != "pending"
        || candidate.benchmark_state_eligible
        || candidate.outcome_labeling_eligible
        || candidate.decision_training_eligible
        || candidate.reward_evidence_eligible
        || candidate.shadow_evidence_eligible
        || candidate.trading_authorized
    {
        return Err("historical state reconstruction candidate is invalid".to_string());
    }
    validate_component_contract(&candidate.components)?;
    let anchor = ConfirmedHistoricalAnchorProjection {
        candidate_id: candidate.anchor_candidate_id.clone(),
        candidate_sha256: candidate.anchor_candidate_sha256.clone(),
        review_id: candidate.anchor_review_id.clone(),
        symbol: candidate.symbol.clone(),
        final_action: candidate.anchor_action.clone(),
        final_thesis: candidate.anchor_thesis.clone(),
        decision_available_at: candidate.decision_available_at,
    };
    let expected = reconstruction_fingerprint(&anchor, &candidate.components)?;
    if expected != candidate.reconstruction_sha256 || candidate.reconstruction_id != expected[..32]
    {
        return Err("historical state reconstruction fingerprint is invalid".to_string());
    }
    Ok(())
}

fn validate_candidate_sources(
    state: &AppState,
    candidate: &HistoricalStateReconstructionCandidate,
) -> Result<(), String> {
    validate_candidate(candidate)?;
    for component in &candidate.components {
        for evidence in &component.evidence {
            let source = verifiable_global_text_source(state, &evidence.source_item_id)?
                .ok_or_else(|| "historical reconstruction source is unavailable".to_string())?;
            let expected = evidence_fingerprint(
                component.component_id,
                &evidence.source_item_id,
                &evidence.source_sha256,
                evidence.claimed_available_at,
                &evidence.source_locator,
                &evidence.verbatim_excerpt,
                &evidence.normalized_claim,
            )?;
            if source.sha256 != evidence.source_sha256
                || source.source_date != evidence.source_date
                || !source.content.contains(&evidence.verbatim_excerpt)
                || expected != evidence.evidence_sha256
                || evidence.claimed_available_at > candidate.decision_available_at
                || evidence.claimed_available_at > candidate.created_at
                || evidence
                    .claimed_available_at
                    .with_timezone(&Shanghai)
                    .date_naive()
                    .to_string()
                    != evidence.source_date
                || (component_requires_symbol(component.component_id)
                    && !source
                        .tickers
                        .iter()
                        .any(|ticker| ticker == &candidate.symbol))
            {
                return Err("historical reconstruction source binding is invalid".to_string());
            }
        }
    }
    Ok(())
}

fn validate_review(review: &HistoricalStateReconstructionReview) -> Result<(), String> {
    let approved = review.verdict == HistoricalStateReviewVerdict::ApprovedForBenchmark;
    let all_checks = review.anchor_binding_confirmed
        && review.source_bytes_confirmed
        && review.availability_times_confirmed
        && review.no_future_information_confirmed
        && review.missingness_preserved_confirmed
        && review.component_interpretations_confirmed;
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || !valid_id(&review.review_id)
        || !valid_id(&review.reconstruction_id)
        || !valid_sha256(&review.reconstruction_sha256)
        || review.review_statement.trim().is_empty()
        || approved != (all_checks && review.benchmark_state_eligible)
        || review.outcome_labeling_eligible
        || review.decision_training_eligible
        || review.reward_evidence_eligible
        || review.shadow_evidence_eligible
        || review.trading_authorized
    {
        return Err("historical state reconstruction review is invalid".to_string());
    }
    Ok(())
}

fn validate_candidate_review_binding(
    candidate: &HistoricalStateReconstructionCandidate,
    review: &HistoricalStateReconstructionReview,
) -> Result<(), String> {
    if review.reconstruction_id != candidate.reconstruction_id
        || review.reconstruction_sha256 != candidate.reconstruction_sha256
        || review.submitted_at < candidate.created_at
    {
        return Err("historical reconstruction review binding is invalid".to_string());
    }
    Ok(())
}

fn component_requires_symbol(component: HistoricalStateComponentId) -> bool {
    matches!(
        component,
        HistoricalStateComponentId::CompanyFundamentals
            | HistoricalStateComponentId::FinancialVerification
            | HistoricalStateComponentId::Valuation
            | HistoricalStateComponentId::Crowding
    )
}

fn required_component_summaries() -> Vec<RequiredComponentSummary> {
    vec![
        RequiredComponentSummary {
            component_id: HistoricalStateComponentId::IndustryThesis,
            label: "产业第一性原理",
            requirement: "当时可见的需求、有效供给、稀缺性、关键假设和证伪条件。",
        },
        RequiredComponentSummary {
            component_id: HistoricalStateComponentId::CompanyFundamentals,
            label: "公司基本面",
            requirement: "商业模式、护城河、差异化、价值捕获和公司级证伪。",
        },
        RequiredComponentSummary {
            component_id: HistoricalStateComponentId::FinancialVerification,
            label: "财务兑现",
            requirement: "收入、毛利、现金流、库存、应收应付、资本开支、产能或订单的当时证据。",
        },
        RequiredComponentSummary {
            component_id: HistoricalStateComponentId::Valuation,
            label: "估值赔率",
            requirement: "当时价格、悲观/基准/乐观价值或明确说明无法重建。",
        },
        RequiredComponentSummary {
            component_id: HistoricalStateComponentId::Crowding,
            label: "拥挤与价格位置",
            requirement: "当时可见的涨幅、位置、关注度、杠杆或明确缺失。",
        },
        RequiredComponentSummary {
            component_id: HistoricalStateComponentId::MarketRegime,
            label: "宏观市场状态",
            requirement: "当时利率、波动、流动性、就业或市场状态证据。",
        },
        RequiredComponentSummary {
            component_id: HistoricalStateComponentId::PortfolioContext,
            label: "组合约束",
            requirement: "当时持仓、暴露、替代选择和风险预算；无法恢复时必须明确缺失。",
        },
    ]
}

pub(crate) fn outcome_protocol() -> HistoricalOutcomeProtocol {
    HistoricalOutcomeProtocol {
        protocol_version: OUTCOME_PROTOCOL_VERSION.to_string(),
        horizons_market_sessions: vec![20, 60, 250],
        asset_price_basis: "FMP adjusted close".to_string(),
        benchmark_symbol: "SPY".to_string(),
        benchmark_price_basis: "FMP adjusted close".to_string(),
        start_rule: "判断可用时间之后，标的与 SPY 共同出现的第一个完整美股交易日为起点；美东收盘后形成的判断从下一共同交易日开始。".to_string(),
        metrics: vec![
            "asset_return_percent".to_string(),
            "benchmark_return_percent".to_string(),
            "excess_return_percent".to_string(),
            "max_drawdown_percent".to_string(),
        ],
        missing_session_rule: "只按标的与 SPY 都有复权收盘价的共同交易日计数，不用日历日填充。".to_string(),
        future_information_rule: "状态重建截止判断可用时间；未来价格只写入独立结果标签，不得回流修改当时状态或人工判断。".to_string(),
        automatic_labeling_enabled: false,
    }
}

fn evidence_fingerprint(
    component_id: HistoricalStateComponentId,
    source_item_id: &str,
    source_sha256: &str,
    claimed_available_at: DateTime<Utc>,
    source_locator: &str,
    verbatim_excerpt: &str,
    normalized_claim: &str,
) -> Result<String, String> {
    hash_json(&serde_json::json!({
        "component_policy_version": COMPONENT_POLICY_VERSION,
        "component_id": component_id,
        "source_item_id": source_item_id,
        "source_sha256": source_sha256,
        "claimed_available_at": claimed_available_at,
        "source_locator": source_locator,
        "verbatim_excerpt": verbatim_excerpt,
        "normalized_claim": normalized_claim,
    }))
}

fn reconstruction_fingerprint(
    anchor: &ConfirmedHistoricalAnchorProjection,
    components: &[HistoricalStateComponent],
) -> Result<String, String> {
    hash_json(&serde_json::json!({
        "schema_version": CANDIDATE_SCHEMA_VERSION,
        "component_policy_version": COMPONENT_POLICY_VERSION,
        "anchor_candidate_id": anchor.candidate_id,
        "anchor_candidate_sha256": anchor.candidate_sha256,
        "anchor_review_id": anchor.review_id,
        "symbol": anchor.symbol,
        "anchor_action": anchor.final_action,
        "anchor_thesis": anchor.final_thesis,
        "decision_available_at": anchor.decision_available_at,
        "components": components,
    }))
}

fn hash_json(value: &serde_json::Value) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn candidate_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-state-reconstruction-candidates")
}

fn review_directory(state: &AppState, reconstruction_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-state-reconstruction-reviews")
        .join(reconstruction_id)
}

async fn read_candidates(
    state: &AppState,
) -> Result<Vec<HistoricalStateReconstructionCandidate>, String> {
    read_json_records(&candidate_directory(state), validate_candidate).await
}

async fn read_reviews(
    state: &AppState,
) -> Result<HashMap<String, HistoricalStateReconstructionReview>, String> {
    let root = decision_root(state).join("historical-state-reconstruction-reviews");
    let mut latest = HashMap::new();
    let mut directories = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(latest),
        Err(error) => return Err(error.to_string()),
    };
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
        let reconstruction_id = entry.file_name().to_string_lossy().to_string();
        if !valid_id(&reconstruction_id) {
            return Err("invalid historical reconstruction review directory".to_string());
        }
        let reviews: Vec<HistoricalStateReconstructionReview> =
            read_json_records(&entry.path(), validate_review).await?;
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
        if tips.len() > 1 {
            return Err("historical reconstruction review chain has multiple tips".to_string());
        }
        if let Some(tip) = tips.first() {
            let mut cursor = Some(*tip);
            let mut visited = BTreeSet::new();
            while let Some(review) = cursor {
                if review.reconstruction_id != reconstruction_id
                    || !visited.insert(review.review_id.clone())
                {
                    return Err("historical reconstruction review chain is invalid".to_string());
                }
                cursor = review
                    .previous_review_id
                    .as_ref()
                    .and_then(|id| by_id.get(id).copied());
            }
            if visited.len() != reviews.len() {
                return Err("historical reconstruction review chain is disconnected".to_string());
            }
            latest.insert(reconstruction_id, (*tip).clone());
        }
    }
    Ok(latest)
}

async fn read_json_records<T>(
    directory: &Path,
    validate: fn(&T) -> Result<(), String>,
) -> Result<Vec<T>, String>
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
        let value = serde_json::from_slice::<T>(&bytes).map_err(|error| error.to_string())?;
        validate(&value)?;
        output.push(value);
    }
    Ok(output)
}

struct HistoricalReconstructionLock {
    path: PathBuf,
}

impl Drop for HistoricalReconstructionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<HistoricalReconstructionLock, String> {
    if !valid_id(name) {
        return Err("invalid historical reconstruction lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("historical-reconstruction-{name}.lock"));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(HistoricalReconstructionLock { path }),
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
                return Err("点时重建正在由另一请求更新，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("点时重建锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid audit path".to_string())?;
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
        return Err(format!("{label}过长"));
    }
    Ok(trimmed.to_string())
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
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

    fn missing_components() -> Vec<HistoricalStateComponent> {
        REQUIRED_COMPONENTS
            .into_iter()
            .map(|component_id| HistoricalStateComponent {
                component_id,
                status: HistoricalStateComponentStatus::ExplicitlyMissing,
                evidence: Vec::new(),
                missing_reason: Some("当时资料无法恢复".to_string()),
            })
            .collect()
    }

    #[test]
    fn reconstruction_requires_all_seven_components_and_preserves_missingness() {
        let components = missing_components();
        assert!(validate_component_contract(&components).is_ok());
        let mut incomplete = components.clone();
        incomplete.pop();
        assert!(validate_component_contract(&incomplete).is_err());
        let mut invented = components;
        invented[0].missing_reason = None;
        assert!(validate_component_contract(&invented).is_err());
    }

    #[test]
    fn approved_reconstruction_is_benchmark_only() {
        let review = HistoricalStateReconstructionReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            review_id: "review-1".to_string(),
            previous_review_id: None,
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "a".repeat(64),
            submitted_at: Utc::now(),
            reviewer_id: "admin".to_string(),
            verdict: HistoricalStateReviewVerdict::ApprovedForBenchmark,
            review_statement: "逐项核对完成".to_string(),
            anchor_binding_confirmed: true,
            source_bytes_confirmed: true,
            availability_times_confirmed: true,
            no_future_information_confirmed: true,
            missingness_preserved_confirmed: true,
            component_interpretations_confirmed: true,
            benchmark_state_eligible: true,
            outcome_labeling_eligible: false,
            decision_training_eligible: false,
            reward_evidence_eligible: false,
            shadow_evidence_eligible: false,
            trading_authorized: false,
        };
        assert!(validate_review(&review).is_ok());
        let mut unsafe_review = review;
        unsafe_review.outcome_labeling_eligible = true;
        assert!(validate_review(&unsafe_review).is_err());
    }

    #[test]
    fn benchmark_outcome_protocol_is_fixed_but_disabled() {
        let protocol = outcome_protocol();
        assert_eq!(protocol.horizons_market_sessions, vec![20, 60, 250]);
        assert_eq!(protocol.benchmark_symbol, "SPY");
        assert!(!protocol.automatic_labeling_enabled);
    }
}
