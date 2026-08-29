//! Governance for the historical benchmark outcome protocol.
//!
//! This layer freezes how future prices may eventually be attached to an
//! already approved point-in-time reconstruction.  Approval here is deliberately
//! narrow: it may only allow a later labeler implementation to be reviewed.  It
//! never fetches a price, writes an outcome, trains a model or opens execution.

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
use uuid::Uuid;

use super::historical_state_reconstructions::{
    HistoricalOutcomeProtocol, benchmark_ready_count_for_outcome_governance, outcome_protocol,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-governance-registry-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-historical-outcome-governance-review-v1";
const MAX_RATIONALE_CHARS: usize = 1_600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeGovernanceVerdict {
    ApprovedForImplementationReview,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeGovernanceRequest {
    expected_review_id: Option<String>,
    verdict: HistoricalOutcomeGovernanceVerdict,
    rationale: String,
    protocol_frozen_pre_outcome_confirmed: bool,
    adjusted_close_source_confirmed: bool,
    common_session_rule_confirmed: bool,
    benchmark_rule_confirmed: bool,
    future_isolation_confirmed: bool,
    missing_data_fail_closed_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeGovernanceReview {
    schema_version: String,
    review_id: String,
    previous_review_id: Option<String>,
    protocol_version: String,
    protocol_sha256: String,
    submitted_at: DateTime<Utc>,
    reviewer_id: String,
    verdict: HistoricalOutcomeGovernanceVerdict,
    rationale: String,
    benchmark_state_count_at_review: usize,
    protocol_frozen_pre_outcome_confirmed: bool,
    adjusted_close_source_confirmed: bool,
    common_session_rule_confirmed: bool,
    benchmark_rule_confirmed: bool,
    future_isolation_confirmed: bool,
    missing_data_fail_closed_confirmed: bool,
    labeler_implementation_registration_eligible: bool,
    outcome_label_generation_enabled: bool,
    decision_training_authorized: bool,
    reward_evidence_authorized: bool,
    shadow_evidence_authorized: bool,
    trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeGovernanceRegistry {
    schema_version: String,
    protocol: HistoricalOutcomeProtocol,
    protocol_sha256: String,
    benchmark_ready_count: usize,
    latest_review: Option<HistoricalOutcomeGovernanceReview>,
    protocol_review_status: String,
    labeler_implementation_registration_eligible: bool,
    outcome_label_generation_enabled: bool,
    decision_training_authorized: bool,
    reward_evidence_authorized: bool,
    shadow_evidence_authorized: bool,
    trading_authorized: bool,
    scope: String,
}

pub(crate) async fn handle_get_historical_outcome_governance(
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
            warn!(%error, "historical outcome governance read failed");
            crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, "历史结果协议暂时不可用")
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_governance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<ReviewHistoricalOutcomeGovernanceRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_protocol(&state, &admin.user_id, request).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

async fn build_registry(state: &AppState) -> Result<HistoricalOutcomeGovernanceRegistry, String> {
    let protocol = outcome_protocol();
    let protocol_sha256 = protocol_fingerprint(&protocol)?;
    let benchmark_ready_count = benchmark_ready_count_for_outcome_governance(state).await?;
    let latest_review = latest_review(state).await?;
    let current_approval = latest_review.as_ref().is_some_and(|review| {
        review.protocol_version == protocol.protocol_version
            && review.protocol_sha256 == protocol_sha256
            && review.verdict == HistoricalOutcomeGovernanceVerdict::ApprovedForImplementationReview
            && review.labeler_implementation_registration_eligible
            && benchmark_ready_count > 0
    });
    let protocol_review_status = if benchmark_ready_count == 0 {
        "waiting_for_benchmark_state"
    } else if current_approval {
        "implementation_review_registration_eligible"
    } else {
        "waiting_for_protocol_review"
    };
    Ok(HistoricalOutcomeGovernanceRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        protocol,
        protocol_sha256,
        benchmark_ready_count,
        latest_review,
        protocol_review_status: protocol_review_status.to_string(),
        labeler_implementation_registration_eligible: current_approval,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
        scope: "本治理只冻结未来结果的计算口径，并最多允许后续登记独立标签器实现。它不读取行情、不生成收益标签、不回写历史状态，也不授权训练、奖励、影子或交易。".to_string(),
    })
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeReadinessSummary {
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub latest_review_id: Option<String>,
    pub protocol_review_status: String,
    pub labeler_implementation_registration_eligible: bool,
    pub outcome_label_generation_enabled: bool,
}

pub(crate) async fn historical_outcome_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeReadinessSummary {
        protocol_version: registry.protocol.protocol_version,
        protocol_sha256: registry.protocol_sha256,
        latest_review_id: registry.latest_review.map(|review| review.review_id),
        protocol_review_status: registry.protocol_review_status,
        labeler_implementation_registration_eligible: registry
            .labeler_implementation_registration_eligible,
        outcome_label_generation_enabled: registry.outcome_label_generation_enabled,
    })
}

async fn review_protocol(
    state: &AppState,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeGovernanceRequest,
) -> Result<HistoricalOutcomeGovernanceReview, String> {
    let _lock = acquire_lock(state).await?;
    let protocol = outcome_protocol();
    let protocol_sha256 = protocol_fingerprint(&protocol)?;
    let benchmark_ready_count = benchmark_ready_count_for_outcome_governance(state).await?;
    let previous = latest_review(state).await?;
    if request.expected_review_id.as_deref()
        != previous.as_ref().map(|review| review.review_id.as_str())
    {
        return Err("历史结果协议复核已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let approved =
        request.verdict == HistoricalOutcomeGovernanceVerdict::ApprovedForImplementationReview;
    let all_checks = request.protocol_frozen_pre_outcome_confirmed
        && request.adjusted_close_source_confirmed
        && request.common_session_rule_confirmed
        && request.benchmark_rule_confirmed
        && request.future_isolation_confirmed
        && request.missing_data_fail_closed_confirmed;
    if approved && benchmark_ready_count == 0 {
        return Err("至少需要一条人工批准的历史基准状态，才能批准结果标签器实现评审".to_string());
    }
    if approved && !all_checks {
        return Err("批准结果协议前必须完成全部口径、基准、时点和缺失检查".to_string());
    }
    let review = HistoricalOutcomeGovernanceReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        review_id: Uuid::new_v4().simple().to_string(),
        previous_review_id: previous.map(|review| review.review_id),
        protocol_version: protocol.protocol_version,
        protocol_sha256,
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        verdict: request.verdict,
        rationale,
        benchmark_state_count_at_review: benchmark_ready_count,
        protocol_frozen_pre_outcome_confirmed: request.protocol_frozen_pre_outcome_confirmed,
        adjusted_close_source_confirmed: request.adjusted_close_source_confirmed,
        common_session_rule_confirmed: request.common_session_rule_confirmed,
        benchmark_rule_confirmed: request.benchmark_rule_confirmed,
        future_isolation_confirmed: request.future_isolation_confirmed,
        missing_data_fail_closed_confirmed: request.missing_data_fail_closed_confirmed,
        labeler_implementation_registration_eligible: approved,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        trading_authorized: false,
    };
    validate_review(&review)?;
    write_immutable_json(
        &review_directory(state).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

pub(crate) fn protocol_fingerprint(protocol: &HistoricalOutcomeProtocol) -> Result<String, String> {
    let bytes = serde_json::to_vec(protocol).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn validate_review(review: &HistoricalOutcomeGovernanceReview) -> Result<(), String> {
    let approved =
        review.verdict == HistoricalOutcomeGovernanceVerdict::ApprovedForImplementationReview;
    let all_checks = review.protocol_frozen_pre_outcome_confirmed
        && review.adjusted_close_source_confirmed
        && review.common_session_rule_confirmed
        && review.benchmark_rule_confirmed
        && review.future_isolation_confirmed
        && review.missing_data_fail_closed_confirmed;
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.protocol_sha256)
        || review.protocol_version.trim().is_empty()
        || review.rationale.trim().is_empty()
        || approved
            != (review.benchmark_state_count_at_review > 0
                && all_checks
                && review.labeler_implementation_registration_eligible)
        || review.outcome_label_generation_enabled
        || review.decision_training_authorized
        || review.reward_evidence_authorized
        || review.shadow_evidence_authorized
        || review.trading_authorized
    {
        return Err("historical outcome governance review is invalid".to_string());
    }
    Ok(())
}

async fn latest_review(
    state: &AppState,
) -> Result<Option<HistoricalOutcomeGovernanceReview>, String> {
    let reviews = read_reviews(state).await?;
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
        return Err("historical outcome governance chain has invalid tips".to_string());
    }
    let tip = tips[0];
    let mut cursor = Some(tip);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("historical outcome governance chain has a cycle".to_string());
        }
        cursor = match review.previous_review_id.as_ref() {
            Some(id) => Some(*by_id.get(id).ok_or_else(|| {
                "historical outcome governance chain is disconnected".to_string()
            })?),
            None => None,
        };
    }
    if visited.len() != reviews.len() {
        return Err("historical outcome governance chain is disconnected".to_string());
    }
    Ok(Some(tip.clone()))
}

async fn read_reviews(state: &AppState) -> Result<Vec<HistoricalOutcomeGovernanceReview>, String> {
    let directory = review_directory(state);
    let mut output = Vec::new();
    let mut entries = match tokio::fs::read_dir(&directory).await {
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
        let review = serde_json::from_slice::<HistoricalOutcomeGovernanceReview>(&bytes)
            .map_err(|error| error.to_string())?;
        validate_review(&review)?;
        output.push(review);
    }
    Ok(output)
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("historical-outcome-governance-reviews")
}

struct HistoricalOutcomeGovernanceLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeGovernanceLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<HistoricalOutcomeGovernanceLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("historical-outcome-governance.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(HistoricalOutcomeGovernanceLock { path }),
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
                return Err("历史结果协议正在由另一请求复核，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("历史结果协议锁获取失败".to_string())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid historical outcome audit path".to_string())?;
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

    fn approved_review() -> HistoricalOutcomeGovernanceReview {
        let protocol = outcome_protocol();
        HistoricalOutcomeGovernanceReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            review_id: "review-1".to_string(),
            previous_review_id: None,
            protocol_version: protocol.protocol_version.clone(),
            protocol_sha256: protocol_fingerprint(&protocol).unwrap(),
            submitted_at: Utc::now(),
            reviewer_id: "admin".to_string(),
            verdict: HistoricalOutcomeGovernanceVerdict::ApprovedForImplementationReview,
            rationale: "协议在查看结果前冻结并逐项复核。".to_string(),
            benchmark_state_count_at_review: 1,
            protocol_frozen_pre_outcome_confirmed: true,
            adjusted_close_source_confirmed: true,
            common_session_rule_confirmed: true,
            benchmark_rule_confirmed: true,
            future_isolation_confirmed: true,
            missing_data_fail_closed_confirmed: true,
            labeler_implementation_registration_eligible: true,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            trading_authorized: false,
        }
    }

    #[test]
    fn protocol_fingerprint_is_deterministic_and_binds_every_rule() {
        let protocol = outcome_protocol();
        let first = protocol_fingerprint(&protocol).unwrap();
        assert_eq!(first, protocol_fingerprint(&protocol).unwrap());
        let mut changed = protocol;
        changed.benchmark_symbol = "QQQ".to_string();
        assert_ne!(first, protocol_fingerprint(&changed).unwrap());
    }

    #[test]
    fn approval_only_allows_future_implementation_review() {
        let review = approved_review();
        assert!(validate_review(&review).is_ok());
        let mut unsafe_review = review;
        unsafe_review.outcome_label_generation_enabled = true;
        assert!(validate_review(&unsafe_review).is_err());
    }

    #[test]
    fn approval_requires_a_benchmark_state_and_all_checks() {
        let mut review = approved_review();
        review.benchmark_state_count_at_review = 0;
        assert!(validate_review(&review).is_err());
        let mut review = approved_review();
        review.future_isolation_confirmed = false;
        assert!(validate_review(&review).is_err());
    }
}
