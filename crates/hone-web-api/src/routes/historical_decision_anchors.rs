//! Administrator-reviewed historical decision anchors.
//!
//! A transcript is not a decision and a model-extracted sentence is not Old
//! Wang's confirmed investment logic.  This module therefore stores exact,
//! hash-bound source excerpts as candidates and requires a separate immutable
//! human confirmation.  Even confirmed anchors remain benchmark-only until a
//! future point-in-time reconstruction protocol is independently reviewed.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Asia::Shanghai;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;
use uuid::Uuid;

use super::research_library::{ResearchUse, items_for_global_use, verifiable_global_text_source};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-decision-anchor-registry-v1";
const DISCOVERY_SCHEMA_VERSION: &str = "hone-historical-anchor-discovery-v1";
const DISCOVERY_POLICY_VERSION: &str =
    "hone-historical-anchor-discovery-v2-conservative-self-action-prefill";
const DISCOVERY_REVIEW_BATCH_POLICY_VERSION: &str =
    "hone-historical-anchor-review-batch-v1-dominant-speaker-diversity";
const LEGACY_DISCOVERY_SCREENING_SCHEMA_VERSION: &str =
    "hone-historical-anchor-discovery-screening-v1";
const DISCOVERY_SCREENING_SCHEMA_VERSION: &str =
    "hone-historical-anchor-discovery-screening-v2-correction-chain";
const CANDIDATE_SCHEMA_VERSION: &str = "hone-historical-decision-anchor-candidate-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-historical-decision-anchor-review-v2-available-at";
const SOURCE_POLICY_VERSION: &str = "hone-historical-anchor-source-v1-global-hash-bound";
const BENCHMARK_POLICY_VERSION: &str = "hone-historical-anchor-benchmark-only-v1";
const MAX_EXCERPT_CHARS: usize = 2_400;
const MAX_THESIS_CHARS: usize = 1_200;
const MAX_LOCATOR_CHARS: usize = 160;
const MAX_DISCOVERY_EXCERPT_CHARS: usize = 1_200;
const MAX_DISCOVERY_CONTEXT_CHARS: usize = 4_000;
const DISCOVERY_CONTEXT_RADIUS_LINES: usize = 2;
const MAX_SCREENING_CORRECTION_REASON_CHARS: usize = 400;
const MAX_DISCOVERY_SUGGESTIONS_PER_SOURCE: usize = 4;
const MAX_DISCOVERY_SUGGESTIONS: usize = 120;
const DISCOVERY_REVIEW_BATCH_SIZE: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalAnchorAction {
    Increase,
    Maintain,
    Reduce,
    Exit,
    ResearchOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalAnchorReviewVerdict {
    Confirmed,
    Revised,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalAnchorDiscoveryScreeningVerdict {
    ContinueCandidateReview,
    NotDecisionContext,
    NeedsMoreContext,
}

impl HistoricalAnchorDiscoveryScreeningVerdict {
    fn status(&self) -> &'static str {
        match self {
            Self::ContinueCandidateReview => "continue_candidate_review",
            Self::NotDecisionContext => "not_decision_context",
            Self::NeedsMoreContext => "needs_more_context",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalAnchorCandidate {
    schema_version: String,
    candidate_id: String,
    candidate_sha256: String,
    source_policy_version: String,
    source_item_id: String,
    source_sha256: String,
    source_title: String,
    source_filename: String,
    source_name: String,
    claimed_source_date: String,
    symbol: String,
    source_locator: String,
    verbatim_excerpt: String,
    candidate_action: HistoricalAnchorAction,
    candidate_thesis: String,
    candidate_origin: String,
    created_at: DateTime<Utc>,
    created_by: String,
    human_confirmation_status: String,
    benchmark_eligible: bool,
    decision_training_eligible: bool,
    reward_evidence_eligible: bool,
    shadow_evidence_eligible: bool,
    trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalAnchorReviewRecord {
    schema_version: String,
    review_id: String,
    previous_review_id: Option<String>,
    candidate_id: String,
    candidate_sha256: String,
    submitted_at: DateTime<Utc>,
    reviewer_id: String,
    verdict: HistoricalAnchorReviewVerdict,
    confirmation_statement: String,
    decision_available_at: Option<DateTime<Utc>>,
    source_time_confirmed: bool,
    speaker_identity_confirmed: bool,
    later_evidence_excluded_confirmed: bool,
    final_action: Option<HistoricalAnchorAction>,
    final_thesis: Option<String>,
    benchmark_eligible: bool,
    decision_training_eligible: bool,
    reward_evidence_eligible: bool,
    shadow_evidence_eligible: bool,
    trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalAnchorSourceSummary {
    source_item_id: String,
    source_sha256: String,
    title: String,
    filename: String,
    source_name: String,
    source_date: String,
    tickers: Vec<String>,
    parse_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalAnchorDiscoveryContextWindow {
    start_line: usize,
    end_line: usize,
    verbatim_context: String,
    context_sha256: String,
    truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalAnchorDiscoverySuggestion {
    suggestion_id: String,
    source_item_id: String,
    source_sha256: String,
    source_title: String,
    source_filename: String,
    source_name: String,
    source_date: String,
    tickers: Vec<String>,
    speaker_label: Option<String>,
    dominant_source_speaker: bool,
    personal_decision_context: bool,
    context_flags: Vec<String>,
    review_priority_reasons: Vec<String>,
    screening_status: String,
    screening_record_id: Option<String>,
    source_locator: String,
    verbatim_excerpt: String,
    context_window: HistoricalAnchorDiscoveryContextWindow,
    matched_action_cues: Vec<String>,
    suggested_action: Option<HistoricalAnchorAction>,
    interpretation_status: String,
    already_saved_candidate: bool,
    requires_manual_thesis: bool,
    requires_speaker_identity_confirmation: bool,
    requires_exact_time_confirmation: bool,
    benchmark_eligible: bool,
    decision_training_eligible: bool,
    reward_evidence_eligible: bool,
    shadow_evidence_eligible: bool,
    trading_authorized: bool,
    rank_score: u16,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalAnchorDiscoveryResponse {
    schema_version: String,
    discovery_policy_version: String,
    source_count: usize,
    matched_source_count: usize,
    suggestion_count: usize,
    suggestions: Vec<HistoricalAnchorDiscoverySuggestion>,
    active_review_batch_policy_version: String,
    active_review_batch_size: usize,
    active_review_batch: Vec<HistoricalAnchorDiscoverySuggestion>,
    screened_suggestion_count: usize,
    pending_screening_count: usize,
    shortlisted_review_count: usize,
    shortlisted_review: Vec<HistoricalAnchorDiscoverySuggestion>,
    automatic_candidate_creation_authorized: bool,
    automatic_confirmation_authorized: bool,
    benchmark_outcome_labeling_enabled: bool,
    decision_training_authorized: bool,
    reward_evidence_authorized: bool,
    shadow_evidence_authorized: bool,
    trading_authorized: bool,
    scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalAnchorDiscoveryScreeningRecord {
    schema_version: String,
    screening_id: String,
    #[serde(default)]
    previous_screening_id: Option<String>,
    suggestion_id: String,
    discovery_policy_version: String,
    review_batch_policy_version: String,
    source_item_id: String,
    source_sha256: String,
    source_locator: String,
    excerpt_sha256: String,
    verdict: HistoricalAnchorDiscoveryScreeningVerdict,
    submitted_at: DateTime<Utc>,
    submitted_by: String,
    #[serde(default)]
    correction_reason: Option<String>,
    candidate_created: bool,
    speaker_identity_confirmed: bool,
    investment_logic_confirmed: bool,
    benchmark_eligible: bool,
    decision_training_eligible: bool,
    reward_evidence_eligible: bool,
    shadow_evidence_eligible: bool,
    trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalAnchorRegistryItem {
    #[serde(flatten)]
    candidate: HistoricalAnchorCandidate,
    latest_review: Option<HistoricalAnchorReviewRecord>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalAnchorRegistryResponse {
    schema_version: String,
    source_policy_version: String,
    benchmark_policy_version: String,
    source_count: usize,
    source_symbol_count: usize,
    earliest_source_date: Option<String>,
    latest_source_date: Option<String>,
    pending_candidate_count: usize,
    confirmed_anchor_count: usize,
    rejected_candidate_count: usize,
    sources: Vec<HistoricalAnchorSourceSummary>,
    anchors: Vec<HistoricalAnchorRegistryItem>,
    automatic_extraction_authorized: bool,
    automatic_confirmation_authorized: bool,
    benchmark_outcome_labeling_enabled: bool,
    decision_training_authorized: bool,
    reward_evidence_authorized: bool,
    shadow_evidence_authorized: bool,
    trading_authorized: bool,
    scope: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CreateHistoricalAnchorCandidateRequest {
    source_item_id: String,
    expected_source_sha256: String,
    symbol: String,
    source_locator: String,
    verbatim_excerpt: String,
    candidate_action: HistoricalAnchorAction,
    candidate_thesis: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ScreenHistoricalAnchorDiscoveryRequest {
    expected_source_sha256: String,
    expected_screening_id: Option<String>,
    verdict: HistoricalAnchorDiscoveryScreeningVerdict,
    correction_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalAnchorRequest {
    expected_review_id: Option<String>,
    verdict: HistoricalAnchorReviewVerdict,
    confirmation_statement: String,
    decision_available_at: Option<DateTime<Utc>>,
    source_time_confirmed: bool,
    speaker_identity_confirmed: bool,
    later_evidence_excluded_confirmed: bool,
    revised_action: Option<HistoricalAnchorAction>,
    revised_thesis: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ConfirmedHistoricalAnchorProjection {
    pub(crate) candidate_id: String,
    pub(crate) candidate_sha256: String,
    pub(crate) review_id: String,
    pub(crate) symbol: String,
    pub(crate) final_action: HistoricalAnchorAction,
    pub(crate) final_thesis: String,
    pub(crate) decision_available_at: DateTime<Utc>,
}

pub(crate) async fn handle_get_historical_anchors(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_registry(&state).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => {
            warn!(%error, "historical decision anchor registry read failed");
            crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, "历史判断锚点暂时不可用")
        }
    }
}

pub(crate) async fn handle_get_historical_anchor_discovery(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_discovery(&state).await {
        Ok(response) => Json(response).into_response(),
        Err(error) => {
            warn!(%error, "historical anchor discovery read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史判断候选发现暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_screen_historical_anchor_discovery(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(suggestion_id): AxumPath<String>,
    Json(request): Json<ScreenHistoricalAnchorDiscoveryRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match screen_discovery_suggestion(&state, &admin.user_id, &suggestion_id, request).await {
        Ok(screening) => (StatusCode::CREATED, Json(screening)).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn handle_create_historical_anchor_candidate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<CreateHistoricalAnchorCandidateRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match create_candidate(&state, &admin.user_id, request).await {
        Ok(candidate) => (StatusCode::CREATED, Json(candidate)).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn handle_review_historical_anchor(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(candidate_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalAnchorRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_candidate(&state, &admin.user_id, &candidate_id, request).await {
        Ok(review) => Json(review).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

async fn build_registry(state: &AppState) -> Result<HistoricalAnchorRegistryResponse, String> {
    let mut sources = items_for_global_use(state, ResearchUse::Chat)?
        .into_iter()
        .filter(|item| item.parse_status == "ready" && !item.tickers.is_empty())
        .map(|item| HistoricalAnchorSourceSummary {
            source_item_id: item.id,
            source_sha256: item.sha256,
            title: item.title,
            filename: item.filename,
            source_name: item.source_name,
            source_date: item.source_date,
            tickers: item.tickers,
            parse_status: item.parse_status,
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| {
        left.source_date
            .cmp(&right.source_date)
            .then_with(|| left.filename.cmp(&right.filename))
    });
    let source_symbols = sources
        .iter()
        .flat_map(|source| source.tickers.iter().cloned())
        .collect::<BTreeSet<_>>();
    let candidates = read_candidates(state).await?;
    let reviews = read_reviews(state).await?;
    let mut anchors = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        validate_candidate_source_binding(state, &candidate)?;
        let latest_review = reviews.get(&candidate.candidate_id).cloned();
        if let Some(review) = latest_review.as_ref() {
            validate_candidate_review_binding(&candidate, review)?;
        }
        anchors.push(HistoricalAnchorRegistryItem {
            candidate,
            latest_review,
        });
    }
    anchors.sort_by(|left, right| {
        left.candidate
            .claimed_source_date
            .cmp(&right.candidate.claimed_source_date)
            .then_with(|| left.candidate.symbol.cmp(&right.candidate.symbol))
            .then_with(|| {
                left.candidate
                    .candidate_id
                    .cmp(&right.candidate.candidate_id)
            })
    });
    let confirmed_anchor_count = anchors
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                matches!(
                    review.verdict,
                    HistoricalAnchorReviewVerdict::Confirmed
                        | HistoricalAnchorReviewVerdict::Revised
                )
            })
        })
        .count();
    let rejected_candidate_count = anchors
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                matches!(review.verdict, HistoricalAnchorReviewVerdict::Rejected)
            })
        })
        .count();
    Ok(HistoricalAnchorRegistryResponse {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        source_policy_version: SOURCE_POLICY_VERSION.to_string(),
        benchmark_policy_version: BENCHMARK_POLICY_VERSION.to_string(),
        source_count: sources.len(),
        source_symbol_count: source_symbols.len(),
        earliest_source_date: sources.first().map(|source| source.source_date.clone()),
        latest_source_date: sources.last().map(|source| source.source_date.clone()),
        pending_candidate_count: anchors.len() - confirmed_anchor_count - rejected_candidate_count,
        confirmed_anchor_count,
        rejected_candidate_count,
        sources,
        anchors,
        automatic_extraction_authorized: false,
        automatic_confirmation_authorized: false,
        benchmark_outcome_labeling_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
        scope: "逐字稿只是候选来源；只有原日期、原话、说话人和未使用事后信息均经人工确认后，才成为独立历史基准锚点。当前锚点不进入决策训练、奖励门槛、影子组合或交易。".to_string(),
    })
}

async fn build_discovery(state: &AppState) -> Result<HistoricalAnchorDiscoveryResponse, String> {
    let items = items_for_global_use(state, ResearchUse::Chat)?
        .into_iter()
        .filter(|item| item.parse_status == "ready" && !item.tickers.is_empty())
        .collect::<Vec<_>>();
    let saved_candidates = read_candidates(state).await?;
    let mut suggestions = Vec::new();
    let mut matched_sources = BTreeSet::new();
    for item in &items {
        let source = verifiable_global_text_source(state, &item.id)?
            .ok_or_else(|| "全局资料来源不存在".to_string())?;
        if source.sha256 != item.sha256
            || source.source_date != item.source_date
            || source.tickers != item.tickers
        {
            return Err("全局资料发现索引与完整原文绑定不一致".to_string());
        }
        let mut source_suggestions = discover_source_suggestions(&source)?;
        for suggestion in &mut source_suggestions {
            suggestion.already_saved_candidate = saved_candidates.iter().any(|candidate| {
                candidate.source_item_id == suggestion.source_item_id
                    && candidate.source_sha256 == suggestion.source_sha256
                    && candidate.verbatim_excerpt == suggestion.verbatim_excerpt
            });
        }
        if !source_suggestions.is_empty() {
            matched_sources.insert(source.id);
        }
        suggestions.extend(source_suggestions);
    }
    suggestions.sort_by(|left, right| {
        right
            .rank_score
            .cmp(&left.rank_score)
            .then_with(|| right.source_date.cmp(&left.source_date))
            .then_with(|| left.source_filename.cmp(&right.source_filename))
            .then_with(|| left.source_locator.cmp(&right.source_locator))
            .then_with(|| left.suggestion_id.cmp(&right.suggestion_id))
    });
    suggestions.truncate(MAX_DISCOVERY_SUGGESTIONS);
    let screenings = read_discovery_screenings(state).await?;
    for suggestion in &mut suggestions {
        if let Some(screening) = screenings.get(&suggestion.suggestion_id) {
            validate_discovery_screening_binding(screening, suggestion)?;
            suggestion.screening_status = screening.verdict.status().to_string();
            suggestion.screening_record_id = Some(screening.screening_id.clone());
        }
    }
    let screened_suggestion_count = suggestions
        .iter()
        .filter(|suggestion| suggestion.screening_status != "pending")
        .count();
    let pending_screening_count = suggestions.len() - screened_suggestion_count;
    let shortlisted_review = suggestions
        .iter()
        .filter(|suggestion| suggestion.screening_status == "continue_candidate_review")
        .cloned()
        .collect::<Vec<_>>();
    let active_review_batch = select_discovery_review_batch(&suggestions);
    Ok(HistoricalAnchorDiscoveryResponse {
        schema_version: DISCOVERY_SCHEMA_VERSION.to_string(),
        discovery_policy_version: DISCOVERY_POLICY_VERSION.to_string(),
        source_count: items.len(),
        matched_source_count: matched_sources.len(),
        suggestion_count: suggestions.len(),
        suggestions,
        active_review_batch_policy_version: DISCOVERY_REVIEW_BATCH_POLICY_VERSION.to_string(),
        active_review_batch_size: active_review_batch.len(),
        active_review_batch,
        screened_suggestion_count,
        pending_screening_count,
        shortlisted_review_count: shortlisted_review.len(),
        shortlisted_review,
        automatic_candidate_creation_authorized: false,
        automatic_confirmation_authorized: false,
        benchmark_outcome_labeling_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
        scope: "本队列只在完整逐字稿中定位含明确动作词的原文片段，并保留来源哈希与行号。管理员单问筛选只判断是否值得继续建立候选；不确认说话人、动作或投资逻辑。默认小批次只按主要说话人、第一人称语境、风险排除和公司多样性降低人工噪声，不会自动保存候选、确认判断、生成结果或进入训练。".to_string(),
    })
}

fn discover_source_suggestions(
    source: &super::research_library::VerifiableGlobalResearchSource,
) -> Result<Vec<HistoricalAnchorDiscoverySuggestion>, String> {
    let speaker_counts = discovery_speaker_counts(&source.content);
    let dominant_speaker = discovery_dominant_speaker(&speaker_counts);
    let mut suggestions = Vec::new();
    for (line_index, raw_line) in source.content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let mut matches = discovery_action_cues(line);
        if matches.is_empty() {
            continue;
        }
        matches.sort_by(|left, right| left.0.cmp(&right.0));
        matches.dedup_by(|left, right| left.0 == right.0);
        let first_cue = matches
            .iter()
            .min_by_key(|(cue, _, _)| line.find(cue).unwrap_or(usize::MAX))
            .map(|(cue, _, _)| cue.as_str())
            .ok_or_else(|| "historical discovery cue is missing".to_string())?;
        let verbatim_excerpt = bounded_discovery_excerpt(line, first_cue);
        if !source.content.contains(&verbatim_excerpt) {
            return Err("historical discovery excerpt lost source binding".to_string());
        }
        let context_window =
            discovery_context_window(&source.content, line_index, &verbatim_excerpt)?;
        let unique_actions = matches
            .iter()
            .fold(Vec::new(), |mut output, (_, action, _)| {
                if !output.contains(action) {
                    output.push(action.clone());
                }
                output
            });
        let speaker_label = discovery_speaker_label(line);
        let dominant_source_speaker = speaker_label
            .as_ref()
            .zip(dominant_speaker.as_ref())
            .is_some_and(|(speaker, dominant)| speaker == dominant);
        let personal_decision_context = discovery_has_personal_decision_context(line);
        let question_context = discovery_has_question_context(line);
        let third_party_context = discovery_has_third_party_attribution(line);
        let other_company_context = discovery_has_other_company_reference(line, &source.tickers);
        let suggested_action = (unique_actions.len() == 1
            && discovery_has_direct_self_action_context(line, &unique_actions[0]))
        .then(|| unique_actions[0].clone());
        let matched_action_cues = matches
            .iter()
            .map(|(cue, _, _)| cue.clone())
            .collect::<Vec<_>>();
        let mut context_flags = Vec::new();
        if dominant_source_speaker {
            context_flags.push("dominant_source_speaker".to_string());
        }
        if personal_decision_context {
            context_flags.push("personal_decision_context".to_string());
        }
        if question_context {
            context_flags.push("audience_question_context".to_string());
        }
        if third_party_context {
            context_flags.push("third_party_attribution_context".to_string());
        }
        if other_company_context {
            context_flags.push("other_company_reference_context".to_string());
        }
        if unique_actions.len() > 1 {
            context_flags.push("conflicting_action_context".to_string());
        }
        if suggested_action.is_some() {
            context_flags.push("conservative_action_prefill".to_string());
        }
        let mut review_priority_reasons = Vec::new();
        if dominant_source_speaker {
            review_priority_reasons.push("该标签是本份逐字稿的主要说话人".to_string());
        }
        if personal_decision_context {
            review_priority_reasons.push("包含第一人称判断或持仓语境".to_string());
        }
        if suggested_action.is_some() {
            review_priority_reasons.push("单一方向满足保守动作预填".to_string());
        }
        let rank_score = (matches
            .iter()
            .map(|(_, _, score)| *score)
            .max()
            .unwrap_or(0) as i32
            + if dominant_source_speaker { 24 } else { 0 }
            + if personal_decision_context { 14 } else { 0 }
            + if suggested_action.is_some() { 28 } else { 0 }
            - if question_context { 48 } else { 0 }
            - if third_party_context { 40 } else { 0 }
            - if other_company_context { 32 } else { 0 })
        .clamp(0, u16::MAX as i32) as u16;
        let source_locator = format!("第 {} 行 · 动作词「{}」", line_index + 1, first_cue);
        let suggestion_id = discovery_fingerprint(
            &source.id,
            &source.sha256,
            &source.source_date,
            &source_locator,
            &verbatim_excerpt,
            &matched_action_cues,
        )?;
        suggestions.push(HistoricalAnchorDiscoverySuggestion {
            suggestion_id,
            source_item_id: source.id.clone(),
            source_sha256: source.sha256.clone(),
            source_title: source.title.clone(),
            source_filename: source.filename.clone(),
            source_name: source.source_name.clone(),
            source_date: source.source_date.clone(),
            tickers: source.tickers.clone(),
            speaker_label,
            dominant_source_speaker,
            personal_decision_context,
            context_flags,
            review_priority_reasons,
            screening_status: "pending".to_string(),
            screening_record_id: None,
            source_locator,
            verbatim_excerpt,
            context_window,
            matched_action_cues,
            suggested_action,
            interpretation_status: "unconfirmed_search_hit".to_string(),
            already_saved_candidate: false,
            requires_manual_thesis: true,
            requires_speaker_identity_confirmation: true,
            requires_exact_time_confirmation: true,
            benchmark_eligible: false,
            decision_training_eligible: false,
            reward_evidence_eligible: false,
            shadow_evidence_eligible: false,
            trading_authorized: false,
            rank_score,
        });
    }
    suggestions.sort_by(|left, right| {
        right
            .rank_score
            .cmp(&left.rank_score)
            .then_with(|| left.source_locator.cmp(&right.source_locator))
            .then_with(|| left.suggestion_id.cmp(&right.suggestion_id))
    });
    suggestions.truncate(MAX_DISCOVERY_SUGGESTIONS_PER_SOURCE);
    Ok(suggestions)
}

fn discovery_action_cues(line: &str) -> Vec<(String, HistoricalAnchorAction, u16)> {
    const CUES: &[(&str, &str, u16)] = &[
        ("全部卖出", "exit", 100),
        ("直接清仓", "exit", 100),
        ("清仓", "exit", 98),
        ("卖掉", "exit", 92),
        ("退出", "exit", 90),
        ("降低仓位", "reduce", 96),
        ("减少仓位", "reduce", 96),
        ("降低暴露", "reduce", 96),
        ("减仓", "reduce", 94),
        ("减掉", "reduce", 92),
        ("增加仓位", "increase", 96),
        ("增加暴露", "increase", 96),
        ("加仓", "increase", 94),
        ("买回", "increase", 92),
        ("接回", "increase", 92),
        ("增持", "increase", 90),
        ("买入", "increase", 88),
        ("继续持有", "maintain", 88),
        ("持有为主", "maintain", 88),
        ("不卖", "maintain", 84),
        ("持有", "maintain", 76),
        ("不追", "research_only", 80),
        ("不买", "research_only", 80),
        ("先观察", "research_only", 74),
        ("观望", "research_only", 70),
    ];
    CUES.iter()
        .filter(|(cue, _, _)| line.contains(cue))
        .map(|(cue, action, score)| {
            let action = match *action {
                "exit" => HistoricalAnchorAction::Exit,
                "reduce" => HistoricalAnchorAction::Reduce,
                "increase" => HistoricalAnchorAction::Increase,
                "maintain" => HistoricalAnchorAction::Maintain,
                _ => HistoricalAnchorAction::ResearchOnly,
            };
            ((*cue).to_string(), action, *score)
        })
        .collect()
}

fn discovery_speaker_label(line: &str) -> Option<String> {
    let open = line
        .char_indices()
        .find_map(|(index, character)| matches!(character, '(' | '（').then_some(index))?;
    let label = line[..open].trim();
    if label.is_empty() || label.chars().count() > 64 {
        return None;
    }
    Some(label.to_string())
}

fn discovery_speaker_counts(content: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for label in content.lines().filter_map(discovery_speaker_label) {
        *counts.entry(label).or_insert(0) += 1;
    }
    counts
}

fn discovery_dominant_speaker(counts: &BTreeMap<String, usize>) -> Option<String> {
    let maximum = counts.values().copied().max()?;
    let leaders = counts
        .iter()
        .filter(|(_, count)| **count == maximum)
        .map(|(label, _)| label.clone())
        .collect::<Vec<_>>();
    (leaders.len() == 1).then(|| leaders[0].clone())
}

fn discovery_has_question_context(line: &str) -> bool {
    const CUES: &[&str] = &[
        "我想问",
        "想问一下",
        "请问",
        "想请教",
        "你觉得",
        "适不适合",
        "能不能买",
        "要不要买",
    ];
    CUES.iter().any(|cue| line.contains(cue))
}

fn discovery_has_third_party_attribution(line: &str) -> bool {
    const CUES: &[&str] = &[
        "木头姐",
        "他说",
        "她说",
        "他们说",
        "分析师说",
        "有人说",
        "市场说",
    ];
    CUES.iter().any(|cue| line.contains(cue))
}

fn discovery_has_personal_decision_context(line: &str) -> bool {
    const CUES: &[&str] = &[
        "我自己已经",
        "我已经",
        "我当时",
        "我现在持有",
        "我现在不怎么持有",
        "我目前持有",
        "我会",
        "我准备",
        "我打算",
        "我决定",
        "我个人结论",
        "我的仓位",
        "我从上次",
        "我还是判断",
        "我还是持有",
        "给自己就定",
    ];
    CUES.iter().any(|cue| line.contains(cue))
}

fn discovery_has_other_company_reference(line: &str, source_tickers: &[String]) -> bool {
    const COMPANY_ALIASES: &[(&str, &[&str])] = &[
        ("AAOI", &["aaoi", "applied optoelectronics"]),
        ("ALAB", &["alab", "astera labs"]),
        ("AMD", &["amd", "超威"]),
        ("AMZN", &["amzn", "amazon", "亚马逊"]),
        ("APP", &["applovin"]),
        ("CRDO", &["crdo", "credo"]),
        ("GOOGL", &["googl", "google", "alphabet", "谷歌"]),
        ("META", &["meta", "facebook", "脸书"]),
        ("MSFT", &["msft", "microsoft", "微软"]),
        ("MU", &["micron", "美光"]),
        ("NVDA", &["nvda", "nvidia", "英伟达"]),
        ("SNDK", &["sndk", "sandisk", "闪迪"]),
        ("TSM", &["tsm", "tsmc", "台积电"]),
    ];
    let normalized = line.to_ascii_lowercase();
    COMPANY_ALIASES.iter().any(|(ticker, aliases)| {
        !source_tickers
            .iter()
            .any(|source_ticker| source_ticker.eq_ignore_ascii_case(ticker))
            && aliases.iter().any(|alias| normalized.contains(alias))
    })
}

fn select_discovery_review_batch(
    suggestions: &[HistoricalAnchorDiscoverySuggestion],
) -> Vec<HistoricalAnchorDiscoverySuggestion> {
    let mut selected = Vec::new();
    let mut selected_sources = BTreeSet::new();
    let mut selected_tickers = BTreeSet::new();
    for require_prefill in [true, false] {
        for suggestion in suggestions {
            if selected.len() >= DISCOVERY_REVIEW_BATCH_SIZE {
                break;
            }
            if suggestion.screening_status != "pending" {
                continue;
            }
            if require_prefill && suggestion.suggested_action.is_none() {
                continue;
            }
            if !suggestion.dominant_source_speaker || !suggestion.personal_decision_context {
                continue;
            }
            if suggestion.context_flags.iter().any(|flag| {
                matches!(
                    flag.as_str(),
                    "audience_question_context"
                        | "third_party_attribution_context"
                        | "other_company_reference_context"
                        | "conflicting_action_context"
                )
            }) {
                continue;
            }
            if selected_sources.contains(&suggestion.source_item_id)
                || suggestion
                    .tickers
                    .iter()
                    .any(|ticker| selected_tickers.contains(ticker))
            {
                continue;
            }
            selected_sources.insert(suggestion.source_item_id.clone());
            selected_tickers.extend(suggestion.tickers.iter().cloned());
            selected.push(suggestion.clone());
        }
    }
    selected
}

fn discovery_has_direct_self_action_context(line: &str, action: &HistoricalAnchorAction) -> bool {
    // A search hit can contain a third-party quote, a hypothetical example or
    // a negated action. Keep the hit for human discovery, but only prefill an
    // action when the sentence contains an explicit first-person execution
    // phrase and no obvious third-party attribution. This remains a UI hint,
    // never a confirmed interpretation.
    if discovery_has_third_party_attribution(line) || discovery_has_question_context(line) {
        return false;
    }
    let phrases: &[&str] = match action {
        HistoricalAnchorAction::Increase => &[
            "我已经买入",
            "我自己已经买入",
            "我买入",
            "我会买入",
            "我准备买入",
            "我打算买入",
            "我决定买入",
            "我会加仓",
            "我准备加仓",
            "我打算加仓",
            "我决定加仓",
            "我买回",
            "我会买回",
            "我接回",
            "我会接回",
            "我增持",
            "我会增持",
        ],
        HistoricalAnchorAction::Maintain => &[
            "我继续持有",
            "我会继续持有",
            "我会持有",
            "我还是持有",
            "我目前持有",
            "我不卖",
        ],
        HistoricalAnchorAction::Reduce => &[
            "我已经减仓",
            "我会减仓",
            "我准备减仓",
            "我打算减仓",
            "我决定减仓",
            "我先减仓",
            "我会先减仓",
            "我开始减仓",
            "我降低仓位",
            "我减少仓位",
        ],
        HistoricalAnchorAction::Exit => &[
            "我已经卖出",
            "我已经卖掉",
            "我会卖出",
            "我会卖掉",
            "我准备卖出",
            "我准备清仓",
            "我决定卖出",
            "我决定清仓",
            "我清仓",
            "我会清仓",
            "我退出",
            "我会退出",
        ],
        HistoricalAnchorAction::ResearchOnly => &[
            "我不追",
            "我不会追",
            "我不买",
            "我不会买",
            "我先观察",
            "我会观望",
            "我现在观望",
        ],
    };
    phrases.iter().any(|phrase| line.contains(phrase))
        || (matches!(action, HistoricalAnchorAction::Reduce)
            && line.contains("我从")
            && line.contains("开始减仓"))
}

fn bounded_discovery_excerpt(line: &str, cue: &str) -> String {
    let chars = line.chars().collect::<Vec<_>>();
    if chars.len() <= MAX_DISCOVERY_EXCERPT_CHARS {
        return line.to_string();
    }
    let cue_byte = line.find(cue).unwrap_or(0);
    let cue_char = line[..cue_byte].chars().count();
    let start = cue_char.saturating_sub(300);
    let end = (start + MAX_DISCOVERY_EXCERPT_CHARS).min(chars.len());
    chars[start..end]
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

fn discovery_context_window(
    content: &str,
    line_index: usize,
    verbatim_excerpt: &str,
) -> Result<HistoricalAnchorDiscoveryContextWindow, String> {
    let lines = content.lines().collect::<Vec<_>>();
    if line_index >= lines.len() || !lines[line_index].contains(verbatim_excerpt) {
        return Err("historical discovery context lost source binding".to_string());
    }
    let mut start = line_index.saturating_sub(DISCOVERY_CONTEXT_RADIUS_LINES);
    let mut end = (line_index + DISCOVERY_CONTEXT_RADIUS_LINES).min(lines.len() - 1);
    let mut verbatim_context = lines[start..=end].join("\n");
    let mut truncated = start > 0 || end + 1 < lines.len();
    while verbatim_context.chars().count() > MAX_DISCOVERY_CONTEXT_CHARS
        && (start < line_index || end > line_index)
    {
        if start < line_index {
            start += 1;
        } else if end > line_index {
            end -= 1;
        }
        verbatim_context = lines[start..=end].join("\n");
        truncated = true;
    }
    if verbatim_context.chars().count() > MAX_DISCOVERY_CONTEXT_CHARS {
        start = line_index;
        end = line_index;
        verbatim_context = verbatim_excerpt.to_string();
        truncated = true;
    }
    if verbatim_context.trim().is_empty()
        || verbatim_context.chars().count() > MAX_DISCOVERY_CONTEXT_CHARS
        || !verbatim_context.contains(verbatim_excerpt)
    {
        return Err("historical discovery context window is invalid".to_string());
    }
    Ok(HistoricalAnchorDiscoveryContextWindow {
        start_line: start + 1,
        end_line: end + 1,
        context_sha256: format!("{:x}", Sha256::digest(verbatim_context.as_bytes())),
        verbatim_context,
        truncated,
    })
}

fn discovery_fingerprint(
    source_item_id: &str,
    source_sha256: &str,
    source_date: &str,
    source_locator: &str,
    verbatim_excerpt: &str,
    matched_action_cues: &[String],
) -> Result<String, String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema_version": DISCOVERY_SCHEMA_VERSION,
        "discovery_policy_version": DISCOVERY_POLICY_VERSION,
        "source_item_id": source_item_id,
        "source_sha256": source_sha256,
        "source_date": source_date,
        "source_locator": source_locator,
        "verbatim_excerpt": verbatim_excerpt,
        "matched_action_cues": matched_action_cues,
    }))
    .map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

async fn screen_discovery_suggestion(
    state: &AppState,
    admin_id: &str,
    suggestion_id: &str,
    request: ScreenHistoricalAnchorDiscoveryRequest,
) -> Result<HistoricalAnchorDiscoveryScreeningRecord, String> {
    if !valid_id(suggestion_id) {
        return Err("历史原话筛选编号无效".to_string());
    }
    let expected_source_sha256 = request.expected_source_sha256.trim().to_ascii_lowercase();
    if !valid_sha256(&expected_source_sha256) {
        return Err("请绑定有效的完整来源 SHA-256".to_string());
    }
    let _lock = acquire_anchor_lock(state, &format!("screening-{suggestion_id}")).await?;
    let screenings = read_discovery_screenings(state).await?;
    let previous = screenings.get(suggestion_id);
    if request.expected_screening_id.as_deref()
        != previous.map(|screening| screening.screening_id.as_str())
    {
        return Err("筛选状态已经变化，请刷新后重试".to_string());
    }
    let suggestion = find_current_discovery_suggestion(state, suggestion_id)?
        .ok_or_else(|| "历史原话发现项不存在或来源已经变化".to_string())?;
    if suggestion.source_sha256 != expected_source_sha256 {
        return Err("来源文件已经变化，请重新读取后再筛选".to_string());
    }
    if previous.is_some_and(|screening| screening.verdict == request.verdict) {
        return Err("修正后的筛选结果必须发生变化".to_string());
    }
    let correction_reason = match previous {
        Some(_) => Some(bounded_required(
            request.correction_reason.as_deref().unwrap_or_default(),
            MAX_SCREENING_CORRECTION_REASON_CHARS,
            "筛选修正原因",
        )?),
        None if request
            .correction_reason
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()) =>
        {
            return Err("首次筛选不得填写修正原因".to_string());
        }
        None => None,
    };
    let screening = HistoricalAnchorDiscoveryScreeningRecord {
        schema_version: DISCOVERY_SCREENING_SCHEMA_VERSION.to_string(),
        screening_id: Uuid::new_v4().simple().to_string(),
        previous_screening_id: previous.map(|screening| screening.screening_id.clone()),
        suggestion_id: suggestion.suggestion_id.clone(),
        discovery_policy_version: DISCOVERY_POLICY_VERSION.to_string(),
        review_batch_policy_version: DISCOVERY_REVIEW_BATCH_POLICY_VERSION.to_string(),
        source_item_id: suggestion.source_item_id.clone(),
        source_sha256: suggestion.source_sha256.clone(),
        source_locator: suggestion.source_locator.clone(),
        excerpt_sha256: format!(
            "{:x}",
            Sha256::digest(suggestion.verbatim_excerpt.as_bytes())
        ),
        verdict: request.verdict,
        submitted_at: Utc::now(),
        submitted_by: admin_id.to_string(),
        correction_reason,
        candidate_created: false,
        speaker_identity_confirmed: false,
        investment_logic_confirmed: false,
        benchmark_eligible: false,
        decision_training_eligible: false,
        reward_evidence_eligible: false,
        shadow_evidence_eligible: false,
        trading_authorized: false,
    };
    validate_discovery_screening(&screening)?;
    validate_discovery_screening_binding(&screening, &suggestion)?;
    write_immutable_json(
        &discovery_screening_directory(state)
            .join(suggestion_id)
            .join(format!("{}.json", screening.screening_id)),
        &screening,
    )
    .await?;
    Ok(screening)
}

fn find_current_discovery_suggestion(
    state: &AppState,
    suggestion_id: &str,
) -> Result<Option<HistoricalAnchorDiscoverySuggestion>, String> {
    for item in items_for_global_use(state, ResearchUse::Chat)?
        .into_iter()
        .filter(|item| item.parse_status == "ready" && !item.tickers.is_empty())
    {
        let source = verifiable_global_text_source(state, &item.id)?
            .ok_or_else(|| "全局资料来源不存在".to_string())?;
        if source.sha256 != item.sha256
            || source.source_date != item.source_date
            || source.tickers != item.tickers
        {
            return Err("全局资料发现索引与完整原文绑定不一致".to_string());
        }
        if let Some(suggestion) = discover_source_suggestions(&source)?
            .into_iter()
            .find(|suggestion| suggestion.suggestion_id == suggestion_id)
        {
            return Ok(Some(suggestion));
        }
    }
    Ok(None)
}

async fn create_candidate(
    state: &AppState,
    admin_id: &str,
    request: CreateHistoricalAnchorCandidateRequest,
) -> Result<HistoricalAnchorCandidate, String> {
    let source_item_id = request.source_item_id.trim();
    let expected_source_sha256 = request.expected_source_sha256.trim().to_ascii_lowercase();
    let symbol = request.symbol.trim().to_ascii_uppercase();
    let source_locator = bounded_required(&request.source_locator, MAX_LOCATOR_CHARS, "原话定位")?;
    let verbatim_excerpt =
        bounded_required(&request.verbatim_excerpt, MAX_EXCERPT_CHARS, "原话摘录")?;
    let candidate_thesis =
        bounded_required(&request.candidate_thesis, MAX_THESIS_CHARS, "候选判断")?;
    if source_item_id.is_empty() || !valid_sha256(&expected_source_sha256) {
        return Err("请选择有效来源并绑定完整 SHA-256".to_string());
    }
    if !valid_symbol(&symbol) {
        return Err("公司代码无效".to_string());
    }
    let _lock = acquire_anchor_lock(state, "candidate-registry").await?;
    let source = verifiable_global_text_source(state, source_item_id)?
        .ok_or_else(|| "全局资料来源不存在".to_string())?;
    if source.sha256 != expected_source_sha256 {
        return Err("来源文件已经变化，请重新读取后再建立候选".to_string());
    }
    if !source.tickers.iter().any(|ticker| ticker == &symbol) {
        return Err("公司代码不在该来源的已登记范围内".to_string());
    }
    let source_date = NaiveDate::parse_from_str(&source.source_date, "%Y-%m-%d")
        .map_err(|_| "来源日期无效".to_string())?;
    if source_date > chrono::Local::now().date_naive() {
        return Err("不能使用未来日期的来源".to_string());
    }
    if !source.content.contains(&verbatim_excerpt) {
        return Err("原话摘录与完整来源文件不一致".to_string());
    }
    let candidate_sha256 = candidate_fingerprint(
        &source.id,
        &source.sha256,
        &source.source_date,
        &symbol,
        &source_locator,
        &verbatim_excerpt,
        &request.candidate_action,
        &candidate_thesis,
    )?;
    if let Some(existing) = read_candidates(state)
        .await?
        .into_iter()
        .find(|candidate| candidate.candidate_sha256 == candidate_sha256)
    {
        return Ok(existing);
    }
    let candidate = HistoricalAnchorCandidate {
        schema_version: CANDIDATE_SCHEMA_VERSION.to_string(),
        candidate_id: candidate_sha256[..32].to_string(),
        candidate_sha256,
        source_policy_version: SOURCE_POLICY_VERSION.to_string(),
        source_item_id: source.id,
        source_sha256: source.sha256,
        source_title: source.title,
        source_filename: source.filename,
        source_name: source.source_name,
        claimed_source_date: source.source_date,
        symbol,
        source_locator,
        verbatim_excerpt,
        candidate_action: request.candidate_action,
        candidate_thesis,
        candidate_origin: "administrator_transcription_candidate_not_old_wang_confirmed"
            .to_string(),
        created_at: Utc::now(),
        created_by: admin_id.to_string(),
        human_confirmation_status: "pending".to_string(),
        benchmark_eligible: false,
        decision_training_eligible: false,
        reward_evidence_eligible: false,
        shadow_evidence_eligible: false,
        trading_authorized: false,
    };
    write_immutable_json(
        &candidate_directory(state).join(format!("{}.json", candidate.candidate_id)),
        &candidate,
    )
    .await?;
    Ok(candidate)
}

async fn review_candidate(
    state: &AppState,
    admin_id: &str,
    candidate_id: &str,
    request: ReviewHistoricalAnchorRequest,
) -> Result<HistoricalAnchorReviewRecord, String> {
    if !valid_id(candidate_id) {
        return Err("历史判断候选编号无效".to_string());
    }
    let _lock = acquire_anchor_lock(state, &format!("review-{candidate_id}")).await?;
    let candidate = read_candidates(state)
        .await?
        .into_iter()
        .find(|candidate| candidate.candidate_id == candidate_id)
        .ok_or_else(|| "历史判断候选不存在".to_string())?;
    let reviews = read_reviews(state).await?;
    let previous = reviews.get(candidate_id);
    if request.expected_review_id.as_deref() != previous.map(|review| review.review_id.as_str()) {
        return Err("复核状态已经变化，请刷新后重试".to_string());
    }
    let confirmation_statement =
        bounded_required(&request.confirmation_statement, 800, "确认说明")?;
    let is_accepted = matches!(
        request.verdict,
        HistoricalAnchorReviewVerdict::Confirmed | HistoricalAnchorReviewVerdict::Revised
    );
    if is_accepted
        && !(request.source_time_confirmed
            && request.speaker_identity_confirmed
            && request.later_evidence_excluded_confirmed)
    {
        return Err("确认历史锚点前，必须确认来源时间、说话人身份和未使用事后信息".to_string());
    }
    let decision_available_at = if is_accepted {
        let value = request
            .decision_available_at
            .ok_or_else(|| "确认历史锚点前，必须填写当时判断可被市场使用的精确时间".to_string())?;
        let source_date = NaiveDate::parse_from_str(&candidate.claimed_source_date, "%Y-%m-%d")
            .map_err(|_| "来源日期无效".to_string())?;
        if value > Utc::now() || value.with_timezone(&Shanghai).date_naive() != source_date {
            return Err("判断可用时间必须是来源日期当天的真实北京时间，且不能晚于现在".to_string());
        }
        Some(value)
    } else {
        if request.decision_available_at.is_some() {
            return Err("否决候选时不得附带判断可用时间".to_string());
        }
        None
    };
    let (final_action, final_thesis) = match request.verdict {
        HistoricalAnchorReviewVerdict::Confirmed => (
            Some(candidate.candidate_action.clone()),
            Some(candidate.candidate_thesis.clone()),
        ),
        HistoricalAnchorReviewVerdict::Revised => (
            request.revised_action,
            Some(bounded_required(
                request.revised_thesis.as_deref().unwrap_or_default(),
                MAX_THESIS_CHARS,
                "修订后的判断",
            )?),
        ),
        HistoricalAnchorReviewVerdict::Rejected => (None, None),
    };
    if matches!(request.verdict, HistoricalAnchorReviewVerdict::Revised) && final_action.is_none() {
        return Err("修订时必须给出修订后的动作".to_string());
    }
    let review = HistoricalAnchorReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        review_id: Uuid::new_v4().simple().to_string(),
        previous_review_id: previous.map(|review| review.review_id.clone()),
        candidate_id: candidate.candidate_id,
        candidate_sha256: candidate.candidate_sha256,
        submitted_at: Utc::now(),
        reviewer_id: admin_id.to_string(),
        verdict: request.verdict,
        confirmation_statement,
        decision_available_at,
        source_time_confirmed: request.source_time_confirmed,
        speaker_identity_confirmed: request.speaker_identity_confirmed,
        later_evidence_excluded_confirmed: request.later_evidence_excluded_confirmed,
        final_action,
        final_thesis,
        benchmark_eligible: is_accepted,
        decision_training_eligible: false,
        reward_evidence_eligible: false,
        shadow_evidence_eligible: false,
        trading_authorized: false,
    };
    write_immutable_json(
        &review_directory(state, candidate_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

pub(crate) async fn confirmed_anchors_for_reconstruction(
    state: &AppState,
) -> Result<Vec<ConfirmedHistoricalAnchorProjection>, String> {
    let candidates = read_candidates(state).await?;
    let reviews = read_reviews(state).await?;
    let mut output = Vec::new();
    for candidate in candidates {
        validate_candidate_source_binding(state, &candidate)?;
        let Some(review) = reviews.get(&candidate.candidate_id) else {
            continue;
        };
        validate_candidate_review_binding(&candidate, review)?;
        if !matches!(
            review.verdict,
            HistoricalAnchorReviewVerdict::Confirmed | HistoricalAnchorReviewVerdict::Revised
        ) {
            continue;
        }
        output.push(ConfirmedHistoricalAnchorProjection {
            candidate_id: candidate.candidate_id,
            candidate_sha256: candidate.candidate_sha256,
            review_id: review.review_id.clone(),
            symbol: candidate.symbol,
            final_action: review
                .final_action
                .clone()
                .ok_or_else(|| "confirmed historical anchor action is missing".to_string())?,
            final_thesis: review
                .final_thesis
                .clone()
                .ok_or_else(|| "confirmed historical anchor thesis is missing".to_string())?,
            decision_available_at: review.decision_available_at.ok_or_else(|| {
                "confirmed historical anchor availability time is missing".to_string()
            })?,
        });
    }
    output.sort_by(|left, right| {
        left.decision_available_at
            .cmp(&right.decision_available_at)
            .then_with(|| left.symbol.cmp(&right.symbol))
            .then_with(|| left.candidate_id.cmp(&right.candidate_id))
    });
    Ok(output)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn candidate_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-anchor-candidates")
}

fn discovery_screening_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-anchor-discovery-screenings")
}

fn review_directory(state: &AppState, candidate_id: &str) -> PathBuf {
    decision_root(state)
        .join("historical-anchor-reviews")
        .join(candidate_id)
}

struct HistoricalAnchorLock {
    path: PathBuf,
}

impl Drop for HistoricalAnchorLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_anchor_lock(state: &AppState, name: &str) -> Result<HistoricalAnchorLock, String> {
    if !valid_id(name) {
        return Err("invalid historical anchor lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("historical-anchor-{name}.lock"));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(HistoricalAnchorLock { path }),
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
                return Err("历史判断锚点正在由另一请求更新，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("历史判断锚点锁获取失败".to_string())
}

async fn read_candidates(state: &AppState) -> Result<Vec<HistoricalAnchorCandidate>, String> {
    read_json_records(&candidate_directory(state), validate_candidate).await
}

async fn read_discovery_screenings(
    state: &AppState,
) -> Result<HashMap<String, HistoricalAnchorDiscoveryScreeningRecord>, String> {
    let root = discovery_screening_directory(state);
    let mut records = read_json_records(&root, validate_discovery_screening).await?;
    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return latest_discovery_screenings(records);
        }
        Err(error) => return Err(error.to_string()),
    };
    while let Some(entry) = entries
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
        let suggestion_id = entry.file_name().to_string_lossy().to_string();
        if !valid_id(&suggestion_id) {
            return Err("invalid historical discovery screening directory".to_string());
        }
        let mut suggestion_records =
            read_json_records(&entry.path(), validate_discovery_screening).await?;
        if suggestion_records
            .iter()
            .any(|record| record.suggestion_id != suggestion_id)
        {
            return Err("historical discovery screening directory binding is invalid".to_string());
        }
        records.append(&mut suggestion_records);
    }
    latest_discovery_screenings(records)
}

fn latest_discovery_screenings(
    records: Vec<HistoricalAnchorDiscoveryScreeningRecord>,
) -> Result<HashMap<String, HistoricalAnchorDiscoveryScreeningRecord>, String> {
    let mut grouped = BTreeMap::<String, Vec<HistoricalAnchorDiscoveryScreeningRecord>>::new();
    for record in records {
        grouped
            .entry(record.suggestion_id.clone())
            .or_default()
            .push(record);
    }
    let mut latest = HashMap::new();
    for (suggestion_id, records) in grouped {
        let mut by_id = records
            .iter()
            .map(|record| (record.screening_id.clone(), record.clone()))
            .collect::<HashMap<_, _>>();
        if by_id.len() != records.len() {
            return Err("historical discovery screening chain has duplicate ids".to_string());
        }
        let tips = records
            .iter()
            .filter(|record| {
                !records.iter().any(|other| {
                    other.previous_screening_id.as_deref() == Some(&record.screening_id)
                })
            })
            .collect::<Vec<_>>();
        if tips.len() != 1 {
            return Err("historical discovery screening chain must have one tip".to_string());
        }
        let tip = (*tips[0]).clone();
        let mut cursor_id = Some(tip.screening_id.clone());
        let mut visited = BTreeSet::new();
        while let Some(screening_id) = cursor_id {
            let screening = by_id
                .remove(&screening_id)
                .ok_or_else(|| "historical discovery screening chain is invalid".to_string())?;
            if screening.suggestion_id != suggestion_id || !visited.insert(screening_id) {
                return Err("historical discovery screening chain is invalid".to_string());
            }
            cursor_id = screening.previous_screening_id.clone();
        }
        if !by_id.is_empty() {
            return Err("historical discovery screening chain is disconnected".to_string());
        }
        latest.insert(suggestion_id, tip);
    }
    Ok(latest)
}

async fn read_reviews(
    state: &AppState,
) -> Result<HashMap<String, HistoricalAnchorReviewRecord>, String> {
    let root = decision_root(state).join("historical-anchor-reviews");
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
        let candidate_id = entry.file_name().to_string_lossy().to_string();
        if !valid_id(&candidate_id) {
            return Err("invalid historical anchor review directory".to_string());
        }
        let reviews: Vec<HistoricalAnchorReviewRecord> =
            read_json_records(&entry.path(), validate_review).await?;
        let mut by_id = reviews
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
            return Err("historical anchor review chain has multiple tips".to_string());
        }
        if let Some(tip) = tips.first() {
            let mut cursor = Some(*tip);
            let mut visited = BTreeSet::new();
            while let Some(review) = cursor {
                if review.candidate_id != candidate_id || !visited.insert(review.review_id.clone())
                {
                    return Err("historical anchor review chain is invalid".to_string());
                }
                cursor = review
                    .previous_review_id
                    .as_ref()
                    .and_then(|id| by_id.remove(id));
            }
            if visited.len() != reviews.len() {
                return Err("historical anchor review chain is disconnected".to_string());
            }
            latest.insert(candidate_id, (*tip).clone());
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

fn validate_candidate(candidate: &HistoricalAnchorCandidate) -> Result<(), String> {
    if candidate.schema_version != CANDIDATE_SCHEMA_VERSION
        || candidate.source_policy_version != SOURCE_POLICY_VERSION
        || !valid_id(&candidate.candidate_id)
        || !valid_sha256(&candidate.candidate_sha256)
        || !valid_sha256(&candidate.source_sha256)
        || !valid_symbol(&candidate.symbol)
        || candidate.human_confirmation_status != "pending"
        || candidate.benchmark_eligible
        || candidate.decision_training_eligible
        || candidate.reward_evidence_eligible
        || candidate.shadow_evidence_eligible
        || candidate.trading_authorized
    {
        return Err("historical anchor candidate contract is invalid".to_string());
    }
    let expected = candidate_fingerprint(
        &candidate.source_item_id,
        &candidate.source_sha256,
        &candidate.claimed_source_date,
        &candidate.symbol,
        &candidate.source_locator,
        &candidate.verbatim_excerpt,
        &candidate.candidate_action,
        &candidate.candidate_thesis,
    )?;
    if expected != candidate.candidate_sha256 {
        return Err("historical anchor candidate fingerprint is invalid".to_string());
    }
    Ok(())
}

fn validate_discovery_screening(
    screening: &HistoricalAnchorDiscoveryScreeningRecord,
) -> Result<(), String> {
    let legacy = screening.schema_version == LEGACY_DISCOVERY_SCREENING_SCHEMA_VERSION;
    let correction_chain = screening.schema_version == DISCOVERY_SCREENING_SCHEMA_VERSION;
    let correction_reason_valid = match (
        screening.previous_screening_id.as_deref(),
        screening.correction_reason.as_deref(),
    ) {
        (None, None) => true,
        (Some(previous), Some(reason)) => {
            valid_id(previous)
                && !reason.trim().is_empty()
                && reason.chars().count() <= MAX_SCREENING_CORRECTION_REASON_CHARS
        }
        _ => false,
    };
    if (!legacy && !correction_chain)
        || (legacy
            && (screening.previous_screening_id.is_some() || screening.correction_reason.is_some()))
        || !correction_reason_valid
        || screening.discovery_policy_version != DISCOVERY_POLICY_VERSION
        || screening.review_batch_policy_version != DISCOVERY_REVIEW_BATCH_POLICY_VERSION
        || !valid_id(&screening.screening_id)
        || !valid_id(&screening.suggestion_id)
        || screening.source_item_id.trim().is_empty()
        || !valid_sha256(&screening.source_sha256)
        || screening.source_locator.trim().is_empty()
        || !valid_sha256(&screening.excerpt_sha256)
        || screening.submitted_by.trim().is_empty()
        || screening.candidate_created
        || screening.speaker_identity_confirmed
        || screening.investment_logic_confirmed
        || screening.benchmark_eligible
        || screening.decision_training_eligible
        || screening.reward_evidence_eligible
        || screening.shadow_evidence_eligible
        || screening.trading_authorized
    {
        return Err("historical discovery screening contract is invalid".to_string());
    }
    Ok(())
}

fn validate_discovery_screening_binding(
    screening: &HistoricalAnchorDiscoveryScreeningRecord,
    suggestion: &HistoricalAnchorDiscoverySuggestion,
) -> Result<(), String> {
    let excerpt_sha256 = format!(
        "{:x}",
        Sha256::digest(suggestion.verbatim_excerpt.as_bytes())
    );
    if screening.suggestion_id != suggestion.suggestion_id
        || screening.source_item_id != suggestion.source_item_id
        || screening.source_sha256 != suggestion.source_sha256
        || screening.source_locator != suggestion.source_locator
        || screening.excerpt_sha256 != excerpt_sha256
    {
        return Err("historical discovery screening source binding is invalid".to_string());
    }
    Ok(())
}

fn validate_review(review: &HistoricalAnchorReviewRecord) -> Result<(), String> {
    let accepted = matches!(
        review.verdict,
        HistoricalAnchorReviewVerdict::Confirmed | HistoricalAnchorReviewVerdict::Revised
    );
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || !valid_id(&review.review_id)
        || !valid_id(&review.candidate_id)
        || !valid_sha256(&review.candidate_sha256)
        || review.confirmation_statement.trim().is_empty()
        || accepted
            != (review.source_time_confirmed
                && review.speaker_identity_confirmed
                && review.later_evidence_excluded_confirmed
                && review.decision_available_at.is_some()
                && review.benchmark_eligible)
        || review.decision_training_eligible
        || review.reward_evidence_eligible
        || review.shadow_evidence_eligible
        || review.trading_authorized
        || (accepted
            && (review.final_action.is_none()
                || review.final_thesis.as_deref().is_none_or(str::is_empty)))
        || (!accepted && (review.final_action.is_some() || review.final_thesis.is_some()))
        || (!accepted && review.decision_available_at.is_some())
    {
        return Err("historical anchor review contract is invalid".to_string());
    }
    Ok(())
}

fn validate_candidate_source_binding(
    state: &AppState,
    candidate: &HistoricalAnchorCandidate,
) -> Result<(), String> {
    let source = verifiable_global_text_source(state, &candidate.source_item_id)?
        .ok_or_else(|| "historical anchor source is no longer available".to_string())?;
    if source.sha256 != candidate.source_sha256
        || source.source_date != candidate.claimed_source_date
        || !source
            .tickers
            .iter()
            .any(|ticker| ticker == &candidate.symbol)
        || !source.content.contains(&candidate.verbatim_excerpt)
    {
        return Err("historical anchor source binding is invalid".to_string());
    }
    Ok(())
}

fn validate_candidate_review_binding(
    candidate: &HistoricalAnchorCandidate,
    review: &HistoricalAnchorReviewRecord,
) -> Result<(), String> {
    let expected_date = NaiveDate::parse_from_str(&candidate.claimed_source_date, "%Y-%m-%d")
        .map_err(|_| "historical anchor source date is invalid".to_string())?;
    if review.candidate_id != candidate.candidate_id
        || review.candidate_sha256 != candidate.candidate_sha256
        || review.decision_available_at.is_some_and(|available_at| {
            available_at > review.submitted_at
                || available_at.with_timezone(&Shanghai).date_naive() != expected_date
        })
    {
        return Err("historical anchor review binding is invalid".to_string());
    }
    Ok(())
}

fn candidate_fingerprint(
    source_item_id: &str,
    source_sha256: &str,
    source_date: &str,
    symbol: &str,
    source_locator: &str,
    verbatim_excerpt: &str,
    candidate_action: &HistoricalAnchorAction,
    candidate_thesis: &str,
) -> Result<String, String> {
    let bytes = serde_json::to_vec(&serde_json::json!({
        "schema_version": CANDIDATE_SCHEMA_VERSION,
        "source_policy_version": SOURCE_POLICY_VERSION,
        "source_item_id": source_item_id,
        "source_sha256": source_sha256,
        "source_date": source_date,
        "symbol": symbol,
        "source_locator": source_locator,
        "verbatim_excerpt": verbatim_excerpt,
        "candidate_action": candidate_action,
        "candidate_thesis": candidate_thesis,
    }))
    .map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
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

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_symbol(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> HistoricalAnchorCandidate {
        let action = HistoricalAnchorAction::Increase;
        let candidate_sha256 = candidate_fingerprint(
            "source-1",
            &"a".repeat(64),
            "2025-08-21",
            "BE",
            "00:27:18",
            "这是原话",
            &action,
            "AI 候选判断",
        )
        .unwrap();
        HistoricalAnchorCandidate {
            schema_version: CANDIDATE_SCHEMA_VERSION.into(),
            candidate_id: "candidate-1".into(),
            candidate_sha256,
            source_policy_version: SOURCE_POLICY_VERSION.into(),
            source_item_id: "source-1".into(),
            source_sha256: "a".repeat(64),
            source_title: "Bloom Energy".into(),
            source_filename: "be.txt".into(),
            source_name: "老王演讲逐字稿".into(),
            claimed_source_date: "2025-08-21".into(),
            symbol: "BE".into(),
            source_locator: "00:27:18".into(),
            verbatim_excerpt: "这是原话".into(),
            candidate_action: action,
            candidate_thesis: "AI 候选判断".into(),
            candidate_origin: "administrator_transcription_candidate_not_old_wang_confirmed".into(),
            created_at: Utc::now(),
            created_by: "admin".into(),
            human_confirmation_status: "pending".into(),
            benchmark_eligible: false,
            decision_training_eligible: false,
            reward_evidence_eligible: false,
            shadow_evidence_eligible: false,
            trading_authorized: false,
        }
    }

    #[test]
    fn candidate_is_hash_bound_and_never_training_eligible() {
        let mut value = candidate();
        assert!(validate_candidate(&value).is_ok());
        value.candidate_thesis.push_str("事后补充");
        assert!(validate_candidate(&value).is_err());
        let mut value = candidate();
        value.decision_training_eligible = true;
        assert!(validate_candidate(&value).is_err());
    }

    #[test]
    fn confirmed_review_is_benchmark_only() {
        let candidate = candidate();
        let review = HistoricalAnchorReviewRecord {
            schema_version: REVIEW_SCHEMA_VERSION.into(),
            review_id: "review-1".into(),
            previous_review_id: None,
            candidate_id: candidate.candidate_id.clone(),
            candidate_sha256: candidate.candidate_sha256.clone(),
            submitted_at: Utc::now(),
            reviewer_id: "admin".into(),
            verdict: HistoricalAnchorReviewVerdict::Confirmed,
            confirmation_statement: "这是我当时的真实判断。".into(),
            decision_available_at: Some(
                DateTime::parse_from_rfc3339("2025-08-21T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
            source_time_confirmed: true,
            speaker_identity_confirmed: true,
            later_evidence_excluded_confirmed: true,
            final_action: Some(HistoricalAnchorAction::Increase),
            final_thesis: Some("确认判断".into()),
            benchmark_eligible: true,
            decision_training_eligible: false,
            reward_evidence_eligible: false,
            shadow_evidence_eligible: false,
            trading_authorized: false,
        };
        assert!(validate_review(&review).is_ok());
        assert!(validate_candidate_review_binding(&candidate, &review).is_ok());
        let mut mismatched_review = review.clone();
        mismatched_review.candidate_sha256 = "b".repeat(64);
        assert!(validate_review(&mismatched_review).is_ok());
        assert!(validate_candidate_review_binding(&candidate, &mismatched_review).is_err());
        let mut wrong_day_review = review.clone();
        wrong_day_review.decision_available_at = Some(
            DateTime::parse_from_rfc3339("2025-08-22T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        );
        assert!(validate_review(&wrong_day_review).is_ok());
        assert!(validate_candidate_review_binding(&candidate, &wrong_day_review).is_err());
        let mut unsafe_review = review;
        unsafe_review.reward_evidence_eligible = true;
        assert!(validate_review(&unsafe_review).is_err());
    }

    fn discovery_source(
        content: &str,
    ) -> super::super::research_library::VerifiableGlobalResearchSource {
        super::super::research_library::VerifiableGlobalResearchSource {
            id: "source-discovery".into(),
            title: "测试逐字稿".into(),
            filename: "source.txt".into(),
            sha256: "a".repeat(64),
            source_date: "2025-08-21".into(),
            source_name: "老王演讲逐字稿".into(),
            tickers: vec!["MU".into()],
            content: content.into(),
        }
    }

    fn discovery_source_with(
        id: &str,
        ticker: &str,
        content: &str,
    ) -> super::super::research_library::VerifiableGlobalResearchSource {
        let mut source = discovery_source(content);
        source.id = id.into();
        source.title = format!("{ticker} 测试逐字稿");
        source.filename = format!("{id}.txt");
        source.tickers = vec![ticker.into()];
        source
    }

    #[test]
    fn discovery_is_exact_read_only_and_deterministic() {
        let source = discovery_source(
            "主持人(00:01:00): 下一个问题。\n老王(00:02:00): 如果单季现金流转弱，我会先减仓，后面看到新订单再买回。",
        );
        let first = discover_source_suggestions(&source).unwrap();
        let second = discover_source_suggestions(&source).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].suggestion_id, second[0].suggestion_id);
        assert!(source.content.contains(&first[0].verbatim_excerpt));
        assert_eq!(first[0].context_window.start_line, 1);
        assert_eq!(first[0].context_window.end_line, 2);
        assert!(
            first[0]
                .context_window
                .verbatim_context
                .contains(&first[0].verbatim_excerpt)
        );
        assert!(valid_sha256(&first[0].context_window.context_sha256));
        assert_eq!(first[0].interpretation_status, "unconfirmed_search_hit");
        assert!(first[0].suggested_action.is_none());
        assert!(first[0].requires_manual_thesis);
        assert!(first[0].requires_speaker_identity_confirmation);
        assert!(first[0].requires_exact_time_confirmation);
        assert!(!first[0].benchmark_eligible);
        assert!(!first[0].decision_training_eligible);
        assert!(!first[0].reward_evidence_eligible);
        assert!(!first[0].shadow_evidence_eligible);
        assert!(!first[0].trading_authorized);
    }

    #[test]
    fn discovery_prefills_only_direct_self_action_without_conflict() {
        let source = discovery_source(
            "老王(00:03:00): 这个位置我会减仓百分之十，等待下一份财报。\n老王(00:04:00): 我不会不买入，只是继续研究。",
        );
        let suggestions = discover_source_suggestions(&source).unwrap();
        let reduce = suggestions
            .iter()
            .find(|item| item.verbatim_excerpt.contains("减仓百分之十"))
            .unwrap();
        assert_eq!(
            reduce.suggested_action,
            Some(HistoricalAnchorAction::Reduce)
        );
        let ambiguous = suggestions
            .iter()
            .find(|item| item.verbatim_excerpt.contains("不会不买入"))
            .unwrap();
        assert!(ambiguous.suggested_action.is_none());
    }

    #[test]
    fn discovery_does_not_prefill_negated_or_third_party_actions() {
        let source = discovery_source(
            "老王(00:05:00): 我不会把美国股票转到韩国卖掉。\n老王(00:06:00): 木头姐说一旦管线放大，她就会不断加仓。",
        );
        let suggestions = discover_source_suggestions(&source).unwrap();
        assert_eq!(suggestions.len(), 2);
        assert!(
            suggestions
                .iter()
                .all(|suggestion| suggestion.suggested_action.is_none())
        );
    }

    #[test]
    fn discovery_does_not_prefill_audience_questions() {
        let source =
            discovery_source("观众(00:07:00): 你好，我想问一下，你觉得现在适不适合买入一部分？");
        let suggestions = discover_source_suggestions(&source).unwrap();
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].suggested_action.is_none());
    }

    #[test]
    fn discovery_caps_each_source_without_losing_source_binding() {
        let content = (0..10)
            .map(|index| format!("老王(00:{index:02}:00): 第 {index} 次明确减仓。"))
            .collect::<Vec<_>>()
            .join("\n");
        let source = discovery_source(&content);
        let suggestions = discover_source_suggestions(&source).unwrap();
        assert_eq!(suggestions.len(), MAX_DISCOVERY_SUGGESTIONS_PER_SOURCE);
        assert!(
            suggestions
                .iter()
                .all(|item| source.content.contains(&item.verbatim_excerpt))
        );
    }

    #[test]
    fn discovery_context_is_bounded_and_keeps_the_exact_hit() {
        let long_neighbor = "旁白".repeat(MAX_DISCOVERY_CONTEXT_CHARS);
        let source = discovery_source(&format!(
            "{long_neighbor}\n主持人: 继续。\n老王: 我会减仓百分之十。\n主持人: 下一个问题。\n{long_neighbor}"
        ));
        let suggestion = discover_source_suggestions(&source).unwrap().remove(0);
        assert!(
            suggestion
                .context_window
                .verbatim_context
                .contains(&suggestion.verbatim_excerpt)
        );
        assert!(
            suggestion.context_window.verbatim_context.chars().count()
                <= MAX_DISCOVERY_CONTEXT_CHARS
        );
        assert!(suggestion.context_window.truncated);
    }

    #[test]
    fn discovery_marks_only_unique_dominant_speaker_without_confirming_identity() {
        let source = discovery_source(
            "主讲人(00:01:00): 我会减仓百分之十。\n主讲人(00:02:00): 先核对订单。\n嘉宾(00:03:00): 我会买入一点。",
        );
        let suggestions = discover_source_suggestions(&source).unwrap();
        let main = suggestions
            .iter()
            .find(|item| item.speaker_label.as_deref() == Some("主讲人"))
            .unwrap();
        assert!(main.dominant_source_speaker);
        assert!(main.personal_decision_context);
        assert!(main.requires_speaker_identity_confirmation);
        let guest = suggestions
            .iter()
            .find(|item| item.speaker_label.as_deref() == Some("嘉宾"))
            .unwrap();
        assert!(!guest.dominant_source_speaker);
        assert!(guest.requires_speaker_identity_confirmation);
    }

    #[test]
    fn discovery_personal_context_rejects_generic_commentary() {
        assert!(!discovery_has_personal_decision_context(
            "我现在额度用完了，只讨论互换机制和卖掉后的溢价。"
        ));
        assert!(!discovery_has_personal_decision_context(
            "我个人觉得机构会在下跌时买入。"
        ));
        assert!(discovery_has_personal_decision_context(
            "我现在不怎么持有 META。"
        ));
        assert!(discovery_has_personal_decision_context(
            "我还是判断仓位过高，所以已经减仓。"
        ));
        assert!(discovery_has_other_company_reference(
            "我现在不怎么持有 META。",
            &["GOOGL".into()]
        ));
        assert!(!discovery_has_other_company_reference(
            "我现在不怎么持有 META。",
            &["META".into()]
        ));
    }

    #[test]
    fn discovery_review_batch_is_deterministic_risk_filtered_and_diverse() {
        let sources = [
            discovery_source_with(
                "source-a",
                "AAA",
                "主讲人(00:01:00): 我会减仓百分之十。\n主讲人(00:02:00): 继续核对订单。",
            ),
            discovery_source_with(
                "source-b",
                "BBB",
                "主讲人(00:01:00): 我当时还持有很多股票。\n主讲人(00:02:00): 继续核对订单。",
            ),
            discovery_source_with(
                "source-c",
                "CCC",
                "观众(00:01:00): 我想问一下要不要买入。\n观众(00:02:00): 谢谢。",
            ),
            discovery_source_with(
                "source-d",
                "DDD",
                "主讲人(00:01:00): 木头姐说她会加仓。\n主讲人(00:02:00): 继续核对订单。",
            ),
            discovery_source_with(
                "source-e",
                "EEE",
                "主讲人(00:01:00): 我会减仓以后再买回。\n主讲人(00:02:00): 继续核对订单。",
            ),
            discovery_source_with(
                "source-f",
                "AAA",
                "主讲人(00:01:00): 我已经买入一点。\n主讲人(00:02:00): 继续核对订单。",
            ),
            discovery_source_with(
                "source-g",
                "GGG",
                "主讲人(00:01:00): 我现在不怎么持有 META。\n主讲人(00:02:00): 继续核对订单。",
            ),
        ];
        let mut suggestions = sources
            .iter()
            .flat_map(|source| discover_source_suggestions(source).unwrap())
            .collect::<Vec<_>>();
        suggestions.sort_by(|left, right| {
            right
                .rank_score
                .cmp(&left.rank_score)
                .then_with(|| left.suggestion_id.cmp(&right.suggestion_id))
        });

        let first = select_discovery_review_batch(&suggestions);
        let second = select_discovery_review_batch(&suggestions);
        assert_eq!(
            first
                .iter()
                .map(|item| &item.suggestion_id)
                .collect::<Vec<_>>(),
            second
                .iter()
                .map(|item| &item.suggestion_id)
                .collect::<Vec<_>>()
        );
        assert_eq!(first.len(), 2);
        assert!(first.iter().all(|item| {
            item.dominant_source_speaker
                && item.personal_decision_context
                && item.context_flags.iter().all(|flag| {
                    !matches!(
                        flag.as_str(),
                        "audience_question_context"
                            | "third_party_attribution_context"
                            | "other_company_reference_context"
                            | "conflicting_action_context"
                    )
                })
        }));
        assert_eq!(
            first
                .iter()
                .map(|item| &item.source_item_id)
                .collect::<BTreeSet<_>>()
                .len(),
            first.len()
        );
        assert_eq!(
            first
                .iter()
                .flat_map(|item| item.tickers.iter())
                .collect::<BTreeSet<_>>()
                .len(),
            first.len()
        );
    }

    #[test]
    fn discovery_review_batch_excludes_already_screened_rows() {
        let source = discovery_source(
            "主讲人(00:01:00): 我会减仓百分之十。\n主讲人(00:02:00): 继续核对订单。",
        );
        let mut suggestions = discover_source_suggestions(&source).unwrap();
        assert_eq!(select_discovery_review_batch(&suggestions).len(), 1);
        suggestions[0].screening_status = "continue_candidate_review".to_string();
        assert!(select_discovery_review_batch(&suggestions).is_empty());
    }

    #[test]
    fn discovery_screening_is_hash_bound_and_non_authoritative() {
        let source = discovery_source(
            "主讲人(00:01:00): 我会减仓百分之十。\n主讲人(00:02:00): 继续核对订单。",
        );
        let suggestion = discover_source_suggestions(&source).unwrap().remove(0);
        let mut screening = HistoricalAnchorDiscoveryScreeningRecord {
            schema_version: DISCOVERY_SCREENING_SCHEMA_VERSION.to_string(),
            screening_id: "screening-1".to_string(),
            previous_screening_id: None,
            suggestion_id: suggestion.suggestion_id.clone(),
            discovery_policy_version: DISCOVERY_POLICY_VERSION.to_string(),
            review_batch_policy_version: DISCOVERY_REVIEW_BATCH_POLICY_VERSION.to_string(),
            source_item_id: suggestion.source_item_id.clone(),
            source_sha256: suggestion.source_sha256.clone(),
            source_locator: suggestion.source_locator.clone(),
            excerpt_sha256: format!(
                "{:x}",
                Sha256::digest(suggestion.verbatim_excerpt.as_bytes())
            ),
            verdict: HistoricalAnchorDiscoveryScreeningVerdict::ContinueCandidateReview,
            submitted_at: Utc::now(),
            submitted_by: "admin-1".to_string(),
            correction_reason: None,
            candidate_created: false,
            speaker_identity_confirmed: false,
            investment_logic_confirmed: false,
            benchmark_eligible: false,
            decision_training_eligible: false,
            reward_evidence_eligible: false,
            shadow_evidence_eligible: false,
            trading_authorized: false,
        };
        validate_discovery_screening(&screening).unwrap();
        validate_discovery_screening_binding(&screening, &suggestion).unwrap();

        screening.speaker_identity_confirmed = true;
        assert!(validate_discovery_screening(&screening).is_err());
        screening.speaker_identity_confirmed = false;
        screening.excerpt_sha256 = "b".repeat(64);
        assert!(validate_discovery_screening_binding(&screening, &suggestion).is_err());
    }

    #[test]
    fn discovery_screening_corrections_form_one_immutable_tip() {
        let base = HistoricalAnchorDiscoveryScreeningRecord {
            schema_version: DISCOVERY_SCREENING_SCHEMA_VERSION.to_string(),
            screening_id: "screening-1".to_string(),
            previous_screening_id: None,
            suggestion_id: "suggestion-1".to_string(),
            discovery_policy_version: DISCOVERY_POLICY_VERSION.to_string(),
            review_batch_policy_version: DISCOVERY_REVIEW_BATCH_POLICY_VERSION.to_string(),
            source_item_id: "source-1".to_string(),
            source_sha256: "a".repeat(64),
            source_locator: "第 3 行".to_string(),
            excerpt_sha256: "b".repeat(64),
            verdict: HistoricalAnchorDiscoveryScreeningVerdict::NeedsMoreContext,
            submitted_at: Utc::now(),
            submitted_by: "admin-1".to_string(),
            correction_reason: None,
            candidate_created: false,
            speaker_identity_confirmed: false,
            investment_logic_confirmed: false,
            benchmark_eligible: false,
            decision_training_eligible: false,
            reward_evidence_eligible: false,
            shadow_evidence_eligible: false,
            trading_authorized: false,
        };
        let mut correction = base.clone();
        correction.screening_id = "screening-2".to_string();
        correction.previous_screening_id = Some(base.screening_id.clone());
        correction.verdict = HistoricalAnchorDiscoveryScreeningVerdict::ContinueCandidateReview;
        correction.correction_reason = Some("结合前后原文后确认是本人仓位动作。".to_string());
        validate_discovery_screening(&base).unwrap();
        validate_discovery_screening(&correction).unwrap();
        let latest = latest_discovery_screenings(vec![base.clone(), correction.clone()]).unwrap();
        assert_eq!(
            latest.get("suggestion-1").unwrap().screening_id,
            correction.screening_id
        );

        let mut missing_reason = correction.clone();
        missing_reason.correction_reason = None;
        assert!(validate_discovery_screening(&missing_reason).is_err());

        let mut branch = correction;
        branch.screening_id = "screening-3".to_string();
        branch.correction_reason = Some("另一条冲突修正。".to_string());
        assert!(latest_discovery_screenings(vec![base, missing_reason, branch]).is_err());
    }
}
