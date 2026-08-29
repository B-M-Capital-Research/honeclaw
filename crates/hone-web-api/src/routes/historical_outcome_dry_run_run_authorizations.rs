//! Immutable review chain for a future historical-outcome dry-run execution.
//!
//! Approval here is deliberately narrower than execution. It only allows a
//! later isolated runner specification to be registered for another review.
//! This module never invokes the implementation, computes an outcome, creates
//! an output artifact, writes a label, trains, rewards, shadows or trades.

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

use super::historical_outcome_dry_run_implementations::{
    HistoricalOutcomeDryRunImplementationKind, HistoricalOutcomeDryRunImplementationRecord,
    run_authorization_review_eligible_implementations,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-dry-run-run-authorization-registry-v1";
const REVIEW_SCHEMA_VERSION: &str = "hone-historical-outcome-dry-run-run-authorization-review-v1";
const POLICY_VERSION: &str = "hone-historical-outcome-dry-run-run-authorization-v1";
const MAX_RATIONALE_CHARS: usize = 2_400;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeDryRunRunAuthorizationVerdict {
    ApprovedForIsolatedRunnerRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeDryRunRunAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_implementation_spec_sha256: String,
    expected_authorization_review_id: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    verdict: HistoricalOutcomeDryRunRunAuthorizationVerdict,
    rationale: String,
    implementation_fingerprint_confirmed: bool,
    current_upstream_bindings_confirmed: bool,
    code_revision_reproducible_confirmed: bool,
    sealed_input_read_only_confirmed: bool,
    deterministic_common_session_replay_confirmed: bool,
    isolated_ephemeral_output_confirmed: bool,
    resource_bounds_confirmed: bool,
    no_network_or_external_tools_confirmed: bool,
    no_production_label_training_reward_shadow_writes_confirmed: bool,
    no_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeDryRunRunAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub dry_run_implementation_id: String,
    pub dry_run_implementation_spec_sha256: String,
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
    pub sandbox_policy_version: String,
    pub implementation_name: String,
    pub implementation_kind: HistoricalOutcomeDryRunImplementationKind,
    pub code_revision: String,
    pub implementation_status: String,
    pub implementation_registered_by: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub verdict: HistoricalOutcomeDryRunRunAuthorizationVerdict,
    pub rationale: String,
    pub implementation_fingerprint_confirmed: bool,
    pub current_upstream_bindings_confirmed: bool,
    pub code_revision_reproducible_confirmed: bool,
    pub sealed_input_read_only_confirmed: bool,
    pub deterministic_common_session_replay_confirmed: bool,
    pub isolated_ephemeral_output_confirmed: bool,
    pub resource_bounds_confirmed: bool,
    pub no_network_or_external_tools_confirmed: bool,
    pub no_production_label_training_reward_shadow_writes_confirmed: bool,
    pub no_order_broker_or_trading_confirmed: bool,
    pub reviewer_independent_from_registrant: bool,
    pub isolated_runner_registration_eligible: bool,
    pub run_authorized: bool,
    pub offline_dry_run_enabled: bool,
    pub execution_started: bool,
    pub output_artifact_created: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunRunAuthorizationItem {
    pub implementation: HistoricalOutcomeDryRunImplementationRecord,
    pub current_binding: bool,
    pub latest_review: Option<HistoricalOutcomeDryRunRunAuthorizationReview>,
    pub isolated_runner_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeDryRunRunAuthorization {
    pub implementation: HistoricalOutcomeDryRunImplementationRecord,
    pub review: HistoricalOutcomeDryRunRunAuthorizationReview,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeDryRunRunAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeDryRunRunAuthorizationItem>,
    pub review_eligible_implementation_count: usize,
    pub reviewed_implementation_count: usize,
    pub isolated_runner_registration_eligible_count: usize,
    pub authorization_status: String,
    pub run_authorized: bool,
    pub offline_dry_run_enabled: bool,
    pub execution_started: bool,
    pub output_artifact_created: bool,
    pub outcome_label_generation_enabled: bool,
    pub decision_training_authorized: bool,
    pub reward_evidence_authorized: bool,
    pub shadow_evidence_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeDryRunRunAuthorizationReadinessSummary {
    pub reviewed_implementation_count: usize,
    pub isolated_runner_registration_eligible_count: usize,
    pub authorization_status: String,
    pub offline_dry_run_enabled: bool,
    pub outcome_label_generation_enabled: bool,
}

pub(crate) async fn handle_get_historical_outcome_dry_run_run_authorizations(
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
            warn!(%error, "historical outcome dry-run run authorization registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线试运行运行授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_dry_run_run_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(dry_run_implementation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeDryRunRunAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_run_authorization(&state, &admin.user_id, &dry_run_implementation_id, request)
        .await
    {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome dry-run run authorization registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线试运行运行授权复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_dry_run_run_authorization_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunRunAuthorizationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeDryRunRunAuthorizationReadinessSummary {
        reviewed_implementation_count: registry.reviewed_implementation_count,
        isolated_runner_registration_eligible_count: registry
            .isolated_runner_registration_eligible_count,
        authorization_status: registry.authorization_status,
        offline_dry_run_enabled: false,
        outcome_label_generation_enabled: false,
    })
}

pub(crate) async fn isolated_runner_registration_eligible_authorizations(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeDryRunRunAuthorization>, String> {
    let implementations = run_authorization_review_eligible_implementations(state).await?;
    let mut approved = Vec::new();
    for implementation in implementations {
        if let Some(review) = latest_review(state, &implementation).await?
            && review.verdict
                == HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration
            && review.isolated_runner_registration_eligible
        {
            approved.push(ApprovedHistoricalOutcomeDryRunRunAuthorization {
                implementation,
                review,
            });
        }
    }
    approved.sort_by(|left, right| {
        right
            .review
            .submitted_at
            .cmp(&left.review.submitted_at)
            .then_with(|| right.review.review_id.cmp(&left.review.review_id))
    });
    Ok(approved)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeDryRunRunAuthorizationRegistry, String> {
    let implementations = run_authorization_review_eligible_implementations(state).await?;
    let mut items = Vec::with_capacity(implementations.len());
    for implementation in implementations {
        let latest_review = latest_review(state, &implementation).await?;
        let isolated_runner_registration_eligible = latest_review.as_ref().is_some_and(|review| {
            review.verdict
                == HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration
                && review.isolated_runner_registration_eligible
        });
        items.push(HistoricalOutcomeDryRunRunAuthorizationItem {
            implementation,
            current_binding: true,
            latest_review,
            isolated_runner_registration_eligible,
        });
    }
    let review_eligible_implementation_count = items.len();
    let reviewed_implementation_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let isolated_runner_registration_eligible_count = items
        .iter()
        .filter(|item| item.isolated_runner_registration_eligible)
        .count();
    let authorization_status = if items.is_empty() {
        "waiting_for_current_registered_not_run_implementation"
    } else if isolated_runner_registration_eligible_count > 0 {
        "approved_for_future_isolated_runner_registration"
    } else if reviewed_implementation_count > 0 {
        "reviewed_not_approved"
    } else {
        "waiting_for_independent_run_authorization_review"
    };
    Ok(HistoricalOutcomeDryRunRunAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_implementation_count,
        reviewed_implementation_count,
        isolated_runner_registration_eligible_count,
        authorization_status: authorization_status.to_string(),
        run_authorized: false,
        offline_dry_run_enabled: false,
        execution_started: false,
        output_artifact_created: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "本阶段只复核一个精确、绑定有效且状态为 registered_not_run 的实现；实现登记者不能自批。批准只允许未来登记隔离执行器供再次审查；当前不运行代码、不计算收益、不创建输出、不写标签、不训练、不计奖励、不写影子组合、不生成订单，也不访问券商。".to_string(),
    })
}

async fn review_run_authorization(
    state: &AppState,
    reviewer_id: &str,
    dry_run_implementation_id: &str,
    request: ReviewHistoricalOutcomeDryRunRunAuthorizationRequest,
) -> Result<HistoricalOutcomeDryRunRunAuthorizationReview, String> {
    if !valid_id(dry_run_implementation_id) {
        return Err("离线试运行实现编号无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-outcome-dry-run-run-authorization-{dry_run_implementation_id}"),
    )
    .await?;
    let implementation = run_authorization_review_eligible_implementations(state)
        .await?
        .into_iter()
        .find(|implementation| {
            implementation.dry_run_implementation_id == dry_run_implementation_id
        })
        .ok_or_else(|| "实现不存在、绑定已经失效或不再具备运行授权复核资格".to_string())?;
    if request.expected_implementation_spec_sha256
        != implementation.dry_run_implementation_spec_sha256
        || request.expected_authorization_review_id != implementation.authorization_review_id
        || request.expected_snapshot_sha256 != implementation.snapshot_sha256
        || request.expected_protocol_sha256 != implementation.protocol_sha256
    {
        return Err("实现、上游授权、行情或协议指纹已经变化，请刷新后重试".to_string());
    }
    let previous = latest_review(state, &implementation).await?;
    if request.expected_review_id.as_deref()
        != previous.as_ref().map(|review| review.review_id.as_str())
        || request.expected_review_sha256.as_deref()
            != previous
                .as_ref()
                .map(|review| review.review_sha256.as_str())
    {
        return Err("运行授权复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let approved = request.verdict
        == HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration;
    if approved && !request_all_checks(&request) {
        return Err("批准前必须完成实现、绑定、复现、隔离、资源和全部权限检查".to_string());
    }
    if approved && reviewer_id == implementation.registered_by {
        return Err("实现登记者不能批准自己的运行授权；请由另一位管理员独立复核".to_string());
    }
    let mut review = HistoricalOutcomeDryRunRunAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: previous.as_ref().map(|review| review.review_id.clone()),
        previous_review_sha256: previous.as_ref().map(|review| review.review_sha256.clone()),
        dry_run_implementation_id: implementation.dry_run_implementation_id.clone(),
        dry_run_implementation_spec_sha256: implementation
            .dry_run_implementation_spec_sha256
            .clone(),
        authorization_review_id: implementation.authorization_review_id.clone(),
        snapshot_id: implementation.snapshot_id.clone(),
        snapshot_sha256: implementation.snapshot_sha256.clone(),
        reconstruction_id: implementation.reconstruction_id.clone(),
        reconstruction_sha256: implementation.reconstruction_sha256.clone(),
        reconstruction_review_id: implementation.reconstruction_review_id.clone(),
        labeler_implementation_id: implementation.labeler_implementation_id.clone(),
        labeler_implementation_spec_sha256: implementation
            .labeler_implementation_spec_sha256
            .clone(),
        labeler_implementation_review_id: implementation.labeler_implementation_review_id.clone(),
        labeler_code_revision: implementation.labeler_code_revision.clone(),
        protocol_version: implementation.protocol_version.clone(),
        protocol_sha256: implementation.protocol_sha256.clone(),
        sandbox_policy_version: implementation.sandbox_policy_version.clone(),
        implementation_name: implementation.implementation_name.clone(),
        implementation_kind: implementation.implementation_kind.clone(),
        code_revision: implementation.code_revision.clone(),
        implementation_status: implementation.status.clone(),
        implementation_registered_by: implementation.registered_by.clone(),
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        verdict: request.verdict,
        rationale,
        implementation_fingerprint_confirmed: request.implementation_fingerprint_confirmed,
        current_upstream_bindings_confirmed: request.current_upstream_bindings_confirmed,
        code_revision_reproducible_confirmed: request.code_revision_reproducible_confirmed,
        sealed_input_read_only_confirmed: request.sealed_input_read_only_confirmed,
        deterministic_common_session_replay_confirmed: request
            .deterministic_common_session_replay_confirmed,
        isolated_ephemeral_output_confirmed: request.isolated_ephemeral_output_confirmed,
        resource_bounds_confirmed: request.resource_bounds_confirmed,
        no_network_or_external_tools_confirmed: request.no_network_or_external_tools_confirmed,
        no_production_label_training_reward_shadow_writes_confirmed: request
            .no_production_label_training_reward_shadow_writes_confirmed,
        no_order_broker_or_trading_confirmed: request.no_order_broker_or_trading_confirmed,
        reviewer_independent_from_registrant: reviewer_id != implementation.registered_by,
        isolated_runner_registration_eligible: approved,
        run_authorized: false,
        offline_dry_run_enabled: false,
        execution_started: false,
        output_artifact_created: false,
        outcome_label_generation_enabled: false,
        decision_training_authorized: false,
        reward_evidence_authorized: false,
        shadow_evidence_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &implementation)?;
    write_immutable_json(
        &review_directory(state, &implementation).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn request_all_checks(request: &ReviewHistoricalOutcomeDryRunRunAuthorizationRequest) -> bool {
    request.implementation_fingerprint_confirmed
        && request.current_upstream_bindings_confirmed
        && request.code_revision_reproducible_confirmed
        && request.sealed_input_read_only_confirmed
        && request.deterministic_common_session_replay_confirmed
        && request.isolated_ephemeral_output_confirmed
        && request.resource_bounds_confirmed
        && request.no_network_or_external_tools_confirmed
        && request.no_production_label_training_reward_shadow_writes_confirmed
        && request.no_order_broker_or_trading_confirmed
}

fn review_all_checks(review: &HistoricalOutcomeDryRunRunAuthorizationReview) -> bool {
    review.implementation_fingerprint_confirmed
        && review.current_upstream_bindings_confirmed
        && review.code_revision_reproducible_confirmed
        && review.sealed_input_read_only_confirmed
        && review.deterministic_common_session_replay_confirmed
        && review.isolated_ephemeral_output_confirmed
        && review.resource_bounds_confirmed
        && review.no_network_or_external_tools_confirmed
        && review.no_production_label_training_reward_shadow_writes_confirmed
        && review.no_order_broker_or_trading_confirmed
}

fn validate_review(
    review: &HistoricalOutcomeDryRunRunAuthorizationReview,
    implementation: &HistoricalOutcomeDryRunImplementationRecord,
) -> Result<(), String> {
    let approved = review.verdict
        == HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let permissions_closed = !review.run_authorized
        && !review.offline_dry_run_enabled
        && !review.execution_started
        && !review.output_artifact_created
        && !review.outcome_label_generation_enabled
        && !review.decision_training_authorized
        && !review.reward_evidence_authorized
        && !review.shadow_evidence_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    if review.schema_version != REVIEW_SCHEMA_VERSION
        || review.policy_version != POLICY_VERSION
        || !valid_id(&review.review_id)
        || !valid_sha256(&review.review_sha256)
        || review.review_id != review.review_sha256[..32]
        || review.review_sha256 != review_fingerprint(review)?
        || !previous_pair_valid
        || review
            .previous_review_id
            .as_deref()
            .is_some_and(|id| !valid_id(id))
        || review
            .previous_review_sha256
            .as_deref()
            .is_some_and(|sha| !valid_sha256(sha))
        || review.dry_run_implementation_id != implementation.dry_run_implementation_id
        || review.dry_run_implementation_spec_sha256
            != implementation.dry_run_implementation_spec_sha256
        || review.authorization_review_id != implementation.authorization_review_id
        || review.snapshot_id != implementation.snapshot_id
        || review.snapshot_sha256 != implementation.snapshot_sha256
        || review.reconstruction_id != implementation.reconstruction_id
        || review.reconstruction_sha256 != implementation.reconstruction_sha256
        || review.reconstruction_review_id != implementation.reconstruction_review_id
        || review.labeler_implementation_id != implementation.labeler_implementation_id
        || review.labeler_implementation_spec_sha256
            != implementation.labeler_implementation_spec_sha256
        || review.labeler_implementation_review_id
            != implementation.labeler_implementation_review_id
        || review.labeler_code_revision != implementation.labeler_code_revision
        || review.protocol_version != implementation.protocol_version
        || review.protocol_sha256 != implementation.protocol_sha256
        || review.sandbox_policy_version != implementation.sandbox_policy_version
        || review.implementation_name != implementation.implementation_name
        || review.implementation_kind != implementation.implementation_kind
        || review.code_revision != implementation.code_revision
        || review.implementation_status != "registered_not_run"
        || review.implementation_status != implementation.status
        || review.implementation_registered_by != implementation.registered_by
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || (approved && !review_all_checks(review))
        || (approved && !review.reviewer_independent_from_registrant)
        || review.reviewer_independent_from_registrant
            != (review.reviewer_id != review.implementation_registered_by)
        || review.isolated_runner_registration_eligible != (approved && review_all_checks(review))
        || !permissions_closed
    {
        return Err(
            "historical outcome dry-run run authorization is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn review_fingerprint(
    review: &HistoricalOutcomeDryRunRunAuthorizationReview,
) -> Result<String, String> {
    let mut value = serde_json::to_value(review).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "run authorization fingerprint payload is invalid".to_string())?;
    object.remove("review_id");
    object.remove("review_sha256");
    hash_serializable(&value)
}

async fn latest_review(
    state: &AppState,
    implementation: &HistoricalOutcomeDryRunImplementationRecord,
) -> Result<Option<HistoricalOutcomeDryRunRunAuthorizationReview>, String> {
    let directory = review_directory(state, implementation);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
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
        let review: HistoricalOutcomeDryRunRunAuthorizationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, implementation)?;
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err(
                "historical outcome dry-run run authorization contains duplicates".to_string(),
            );
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeDryRunRunAuthorizationReview],
) -> Result<Option<HistoricalOutcomeDryRunRunAuthorizationReview>, String> {
    if reviews.is_empty() {
        return Ok(None);
    }
    let by_id = reviews
        .iter()
        .map(|review| (review.review_id.as_str(), review))
        .collect::<HashMap<_, _>>();
    let tips = reviews
        .iter()
        .filter(|review| {
            !reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
        })
        .collect::<Vec<_>>();
    if tips.len() != 1 {
        return Err(
            "historical outcome dry-run run authorization chain has invalid tips".to_string(),
        );
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err(
                "historical outcome dry-run run authorization chain has a cycle".to_string(),
            );
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(previous_id), Some(previous_sha256)) => {
                let previous = by_id.get(previous_id.as_str()).ok_or_else(|| {
                    "historical outcome dry-run run authorization chain is disconnected".to_string()
                })?;
                if previous.review_sha256 != *previous_sha256 {
                    return Err(
                        "historical outcome dry-run run authorization previous hash mismatch"
                            .to_string(),
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "historical outcome dry-run run authorization previous link is invalid"
                        .to_string(),
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err(
            "historical outcome dry-run run authorization chain is disconnected".to_string(),
        );
    }
    Ok(Some(tips[0].clone()))
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_directory(
    state: &AppState,
    implementation: &HistoricalOutcomeDryRunImplementationRecord,
) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-dry-run-run-authorization-reviews")
        .join(&implementation.dry_run_implementation_id)
}

struct HistoricalOutcomeDryRunRunAuthorizationLock {
    path: PathBuf,
}

impl Drop for HistoricalOutcomeDryRunRunAuthorizationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<HistoricalOutcomeDryRunRunAuthorizationLock, String> {
    if !valid_id(name) {
        return Err("invalid historical outcome dry-run run authorization lock name".to_string());
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
            Ok(_) => return Ok(HistoricalOutcomeDryRunRunAuthorizationLock { path }),
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
                return Err("运行授权正在由另一请求复核，请稍后重试".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("运行授权复核锁获取失败".to_string())
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

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 240
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn implementation() -> HistoricalOutcomeDryRunImplementationRecord {
        HistoricalOutcomeDryRunImplementationRecord {
            schema_version: "hone-historical-outcome-dry-run-implementation-v1".to_string(),
            dry_run_implementation_id: "implementation-1".to_string(),
            dry_run_implementation_spec_sha256: "a".repeat(64),
            registered_at: DateTime::parse_from_rfc3339("2026-08-22T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            registered_by: "admin".to_string(),
            authorization_review_id: "authorization-review-1".to_string(),
            snapshot_id: "snapshot-1".to_string(),
            snapshot_sha256: "b".repeat(64),
            reconstruction_id: "reconstruction-1".to_string(),
            reconstruction_sha256: "c".repeat(64),
            reconstruction_review_id: "reconstruction-review-1".to_string(),
            labeler_implementation_id: "labeler-1".to_string(),
            labeler_implementation_spec_sha256: "d".repeat(64),
            labeler_implementation_review_id: "labeler-review-1".to_string(),
            labeler_code_revision: "oldwang@labeler123".to_string(),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "e".repeat(64),
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            requested_from: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
            requested_to: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            asset_payload_sha256: "f".repeat(64),
            benchmark_payload_sha256: "1".repeat(64),
            asset_series_sha256: "2".repeat(64),
            benchmark_series_sha256: "3".repeat(64),
            common_session_count: 251,
            covered_horizons_market_sessions: vec![20, 60, 250],
            sandbox_policy_version: "hone-historical-outcome-dry-run-sandbox-v1".to_string(),
            implementation_name: "共同交易日隔离试运行实现".to_string(),
            implementation_kind:
                HistoricalOutcomeDryRunImplementationKind::DeterministicIsolatedCommonSessionReplay,
            code_revision: "oldwang@dryrun123".to_string(),
            status: "registered_not_run".to_string(),
            input_contract: "input".to_string(),
            output_contract: "output".to_string(),
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
        }
    }

    fn review(
        verdict: HistoricalOutcomeDryRunRunAuthorizationVerdict,
        all_checks: bool,
    ) -> HistoricalOutcomeDryRunRunAuthorizationReview {
        let value = implementation();
        let approved = verdict
            == HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration;
        let mut review = HistoricalOutcomeDryRunRunAuthorizationReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id: None,
            previous_review_sha256: None,
            dry_run_implementation_id: value.dry_run_implementation_id.clone(),
            dry_run_implementation_spec_sha256: value.dry_run_implementation_spec_sha256.clone(),
            authorization_review_id: value.authorization_review_id.clone(),
            snapshot_id: value.snapshot_id.clone(),
            snapshot_sha256: value.snapshot_sha256.clone(),
            reconstruction_id: value.reconstruction_id.clone(),
            reconstruction_sha256: value.reconstruction_sha256.clone(),
            reconstruction_review_id: value.reconstruction_review_id.clone(),
            labeler_implementation_id: value.labeler_implementation_id.clone(),
            labeler_implementation_spec_sha256: value.labeler_implementation_spec_sha256.clone(),
            labeler_implementation_review_id: value.labeler_implementation_review_id.clone(),
            labeler_code_revision: value.labeler_code_revision.clone(),
            protocol_version: value.protocol_version.clone(),
            protocol_sha256: value.protocol_sha256.clone(),
            sandbox_policy_version: value.sandbox_policy_version.clone(),
            implementation_name: value.implementation_name.clone(),
            implementation_kind: value.implementation_kind.clone(),
            code_revision: value.code_revision.clone(),
            implementation_status: value.status.clone(),
            implementation_registered_by: value.registered_by.clone(),
            submitted_at: DateTime::parse_from_rfc3339("2026-08-22T13:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            reviewer_id: "reviewer".to_string(),
            verdict,
            rationale: "复核实现、边界和全部关闭权限。".to_string(),
            implementation_fingerprint_confirmed: all_checks,
            current_upstream_bindings_confirmed: all_checks,
            code_revision_reproducible_confirmed: all_checks,
            sealed_input_read_only_confirmed: all_checks,
            deterministic_common_session_replay_confirmed: all_checks,
            isolated_ephemeral_output_confirmed: all_checks,
            resource_bounds_confirmed: all_checks,
            no_network_or_external_tools_confirmed: all_checks,
            no_production_label_training_reward_shadow_writes_confirmed: all_checks,
            no_order_broker_or_trading_confirmed: all_checks,
            reviewer_independent_from_registrant: true,
            isolated_runner_registration_eligible: approved && all_checks,
            run_authorized: false,
            offline_dry_run_enabled: false,
            execution_started: false,
            output_artifact_created: false,
            outcome_label_generation_enabled: false,
            decision_training_authorized: false,
            reward_evidence_authorized: false,
            shadow_evidence_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        review
    }

    fn rehash(review: &mut HistoricalOutcomeDryRunRunAuthorizationReview) {
        review.review_sha256 = review_fingerprint(review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
    }

    #[test]
    fn approval_only_allows_future_runner_registration_and_never_runs() {
        let value = review(
            HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration,
            true,
        );
        assert!(validate_review(&value, &implementation()).is_ok());
        assert!(value.isolated_runner_registration_eligible);
        assert!(!value.run_authorized);
        assert!(!value.offline_dry_run_enabled);
        assert!(!value.execution_started);
        assert!(!value.output_artifact_created);
        assert!(!value.outcome_label_generation_enabled);
        assert!(!value.decision_training_authorized);
        assert!(!value.shadow_evidence_authorized);
        assert!(!value.order_generation_authorized);
        assert!(!value.broker_access_authorized);
        assert!(!value.trading_authorized);
    }

    #[test]
    fn approval_without_every_check_is_rejected() {
        let value = review(
            HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration,
            false,
        );
        assert!(validate_review(&value, &implementation()).is_err());
    }

    #[test]
    fn fingerprint_detects_audit_tampering() {
        let mut value = review(
            HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration,
            true,
        );
        value.rationale.push_str("篡改");
        assert!(validate_review(&value, &implementation()).is_err());
    }

    #[test]
    fn exact_implementation_binding_is_required() {
        let value = review(
            HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration,
            true,
        );
        let mut changed = implementation();
        changed.code_revision = "oldwang@different".to_string();
        assert!(validate_review(&value, &changed).is_err());
    }

    #[test]
    fn any_execution_or_trading_authority_is_rejected() {
        let mut value = review(
            HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration,
            true,
        );
        value.run_authorized = true;
        rehash(&mut value);
        assert!(validate_review(&value, &implementation()).is_err());
    }

    #[test]
    fn registrant_cannot_self_approve_run_authority() {
        let mut value = review(
            HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration,
            true,
        );
        value.reviewer_id = value.implementation_registered_by.clone();
        value.reviewer_independent_from_registrant = false;
        rehash(&mut value);
        assert!(validate_review(&value, &implementation()).is_err());
    }

    #[test]
    fn chain_requires_exact_previous_hash_and_single_tip() {
        let first = review(
            HistoricalOutcomeDryRunRunAuthorizationVerdict::ChangesRequested,
            false,
        );
        let mut second = review(
            HistoricalOutcomeDryRunRunAuthorizationVerdict::ApprovedForIsolatedRunnerRegistration,
            true,
        );
        second.previous_review_id = Some(first.review_id.clone());
        second.previous_review_sha256 = Some(first.review_sha256.clone());
        second.submitted_at = DateTime::parse_from_rfc3339("2026-08-22T14:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        rehash(&mut second);
        assert_eq!(
            latest_review_from_records(&[first.clone(), second.clone()])
                .unwrap()
                .unwrap()
                .review_id,
            second.review_id
        );
        second.previous_review_sha256 = Some("9".repeat(64));
        rehash(&mut second);
        assert!(latest_review_from_records(&[first, second]).is_err());
    }
}
