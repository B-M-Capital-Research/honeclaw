//! Immutable independent authorization review for a future outcome-label materialization runner.
//!
//! Approval here is deliberately narrower than execution. It only allows a later immutable,
//! isolated materialization runner specification to be registered for another review. This
//! module never runs the implementation, writes an outcome label, infers investment semantics,
//! trains, rewards, shadows, creates orders, accesses a broker or trades.

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

use super::historical_outcome_label_materialization_implementations::{
    HistoricalOutcomeLabelMaterializationImplementationKind,
    HistoricalOutcomeLabelMaterializationImplementationRecord,
    ReviewEligibleHistoricalOutcomeLabelMaterializationImplementation,
    run_authorization_review_eligible_implementations,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-run-authorization-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-historical-outcome-label-materialization-run-authorization-review-v1";
const POLICY_VERSION: &str = "hone-historical-outcome-label-materialization-run-authorization-v1";
const MAX_RATIONALE_CHARS: usize = 2_400;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict {
    ApprovedForMaterializationRunnerRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_implementation_spec_sha256: String,
    expected_admission_review_sha256: String,
    expected_validation_sha256: String,
    expected_output_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    verdict: HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict,
    rationale: String,
    implementation_fingerprint_confirmed: bool,
    current_upstream_bindings_confirmed: bool,
    code_revision_reproducible_confirmed: bool,
    deterministic_raw_envelope_only_confirmed: bool,
    exact_metric_bit_preservation_confirmed: bool,
    provenance_and_limitations_preserved_confirmed: bool,
    create_once_isolated_output_confirmed: bool,
    missing_data_fail_closed_confirmed: bool,
    no_network_tools_or_production_access_confirmed: bool,
    no_semantic_action_position_or_reward_inference_confirmed: bool,
    no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationRunAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub materialization_implementation_id: String,
    pub materialization_implementation_spec_sha256: String,
    pub materialization_implementation_registered_by: String,
    pub implementation_name: String,
    pub implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind,
    pub code_revision: String,
    pub implementation_status: String,
    pub admission_review_id: String,
    pub admission_review_sha256: String,
    pub admission_reviewer_id: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub validated_by: String,
    pub execution_invoked_by: String,
    pub runner_registered_by: String,
    pub first_execution_authorization_reviewer_id: String,
    pub run_authorization_reviewer_id: String,
    pub output_sha256: String,
    pub snapshot_id: String,
    pub snapshot_sha256: String,
    pub protocol_version: String,
    pub protocol_sha256: String,
    pub admission_known_limitations: String,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub verdict: HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict,
    pub rationale: String,
    pub implementation_fingerprint_confirmed: bool,
    pub current_upstream_bindings_confirmed: bool,
    pub code_revision_reproducible_confirmed: bool,
    pub deterministic_raw_envelope_only_confirmed: bool,
    pub exact_metric_bit_preservation_confirmed: bool,
    pub provenance_and_limitations_preserved_confirmed: bool,
    pub create_once_isolated_output_confirmed: bool,
    pub missing_data_fail_closed_confirmed: bool,
    pub no_network_tools_or_production_access_confirmed: bool,
    pub no_semantic_action_position_or_reward_inference_confirmed: bool,
    pub no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed: bool,
    pub reviewer_independent_from_implementation_and_prior_chain: bool,
    pub materialization_runner_registration_eligible: bool,
    pub materialization_runner_registered: bool,
    pub label_materialization_run_authorized: bool,
    pub label_materialization_started: bool,
    pub outcome_label_write_allowed: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationRunAuthorizationItem {
    pub implementation: HistoricalOutcomeLabelMaterializationImplementationRecord,
    pub current_binding: bool,
    pub latest_review: Option<HistoricalOutcomeLabelMaterializationRunAuthorizationReview>,
    pub materialization_runner_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ApprovedHistoricalOutcomeLabelMaterializationRunAuthorization {
    pub implementation: HistoricalOutcomeLabelMaterializationImplementationRecord,
    pub review: HistoricalOutcomeLabelMaterializationRunAuthorizationReview,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<HistoricalOutcomeLabelMaterializationRunAuthorizationItem>,
    pub review_eligible_implementation_count: usize,
    pub reviewed_implementation_count: usize,
    pub materialization_runner_registration_eligible_count: usize,
    pub authorization_status: String,
    pub materialization_runner_registered: bool,
    pub label_materialization_run_authorized: bool,
    pub label_materialization_started: bool,
    pub outcome_label_write_allowed: bool,
    pub outcome_label_written: bool,
    pub training_authorized: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeLabelMaterializationRunAuthorizationReadinessSummary {
    pub reviewed_implementation_count: usize,
    pub materialization_runner_registration_eligible_count: usize,
    pub authorization_status: String,
    pub label_materialization_run_authorized: bool,
    pub outcome_label_written: bool,
}

#[derive(Debug, Clone)]
struct AuthorizationReviewContext {
    materialization_implementation_id: String,
    materialization_implementation_spec_sha256: String,
    materialization_implementation_registered_by: String,
    implementation_name: String,
    implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind,
    code_revision: String,
    implementation_status: String,
    admission_review_id: String,
    admission_review_sha256: String,
    admission_reviewer_id: String,
    validation_id: String,
    validation_sha256: String,
    validated_by: String,
    execution_invoked_by: String,
    runner_registered_by: String,
    first_execution_authorization_reviewer_id: String,
    run_authorization_reviewer_id: String,
    output_sha256: String,
    snapshot_id: String,
    snapshot_sha256: String,
    protocol_version: String,
    protocol_sha256: String,
    admission_known_limitations: String,
}

impl From<&ReviewEligibleHistoricalOutcomeLabelMaterializationImplementation>
    for AuthorizationReviewContext
{
    fn from(value: &ReviewEligibleHistoricalOutcomeLabelMaterializationImplementation) -> Self {
        let implementation = &value.implementation;
        let review = &value.admission.admission_review;
        let validation = &value.admission.validation;
        Self {
            materialization_implementation_id: implementation
                .materialization_implementation_id
                .clone(),
            materialization_implementation_spec_sha256: implementation
                .materialization_implementation_spec_sha256
                .clone(),
            materialization_implementation_registered_by: implementation.registered_by.clone(),
            implementation_name: implementation.implementation_name.clone(),
            implementation_kind: implementation.implementation_kind.clone(),
            code_revision: implementation.code_revision.clone(),
            implementation_status: implementation.status.clone(),
            admission_review_id: review.review_id.clone(),
            admission_review_sha256: review.review_sha256.clone(),
            admission_reviewer_id: review.reviewer_id.clone(),
            validation_id: validation.validation_id.clone(),
            validation_sha256: validation.validation_sha256.clone(),
            validated_by: validation.validated_by.clone(),
            execution_invoked_by: validation.execution_invoked_by.clone(),
            runner_registered_by: validation.runner_registered_by.clone(),
            first_execution_authorization_reviewer_id: validation
                .first_execution_authorization_reviewer_id
                .clone(),
            run_authorization_reviewer_id: validation.run_authorization_reviewer_id.clone(),
            output_sha256: implementation.output_sha256.clone(),
            snapshot_id: implementation.snapshot_id.clone(),
            snapshot_sha256: implementation.snapshot_sha256.clone(),
            protocol_version: implementation.protocol_version.clone(),
            protocol_sha256: implementation.protocol_sha256.clone(),
            admission_known_limitations: implementation.admission_known_limitations.clone(),
        }
    }
}

pub(crate) async fn handle_get_historical_outcome_label_materialization_run_authorizations(
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
            warn!(%error, "historical outcome label materialization run authorization registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果标签物化运行授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_historical_outcome_label_materialization_run_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(materialization_implementation_id): AxumPath<String>,
    Json(request): Json<ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match review_run_authorization(
        &state,
        &admin.user_id,
        &materialization_implementation_id,
        request,
    )
    .await
    {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome label materialization run authorization registry reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果标签物化运行授权复核暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_label_materialization_run_authorization_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeLabelMaterializationRunAuthorizationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        HistoricalOutcomeLabelMaterializationRunAuthorizationReadinessSummary {
            reviewed_implementation_count: registry.reviewed_implementation_count,
            materialization_runner_registration_eligible_count: registry
                .materialization_runner_registration_eligible_count,
            authorization_status: registry.authorization_status,
            label_materialization_run_authorized: false,
            outcome_label_written: false,
        },
    )
}

pub(crate) async fn materialization_runner_registration_eligible_authorizations(
    state: &AppState,
) -> Result<Vec<ApprovedHistoricalOutcomeLabelMaterializationRunAuthorization>, String> {
    let eligible = run_authorization_review_eligible_implementations(state).await?;
    let mut approved = Vec::new();
    for value in eligible {
        let context = AuthorizationReviewContext::from(&value);
        if let Some(review) = latest_review(state, &context).await?
            && review.verdict
                == HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration
            && review.materialization_runner_registration_eligible
        {
            approved.push(ApprovedHistoricalOutcomeLabelMaterializationRunAuthorization {
                implementation: value.implementation,
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
) -> Result<HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry, String> {
    let eligible = run_authorization_review_eligible_implementations(state).await?;
    let mut items = Vec::with_capacity(eligible.len());
    for value in eligible {
        let context = AuthorizationReviewContext::from(&value);
        let latest_review = latest_review(state, &context).await?;
        let materialization_runner_registration_eligible = latest_review.as_ref().is_some_and(
            |review| {
                review.verdict
                    == HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration
                    && review.materialization_runner_registration_eligible
            },
        );
        items.push(HistoricalOutcomeLabelMaterializationRunAuthorizationItem {
            implementation: value.implementation,
            current_binding: true,
            latest_review,
            materialization_runner_registration_eligible,
        });
    }
    let review_eligible_implementation_count = items.len();
    let reviewed_implementation_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let materialization_runner_registration_eligible_count = items
        .iter()
        .filter(|item| item.materialization_runner_registration_eligible)
        .count();
    let authorization_status = if items.is_empty() {
        "waiting_for_current_registered_not_run_materialization_implementation"
    } else if materialization_runner_registration_eligible_count > 0 {
        "approved_for_future_materialization_runner_registration"
    } else if reviewed_implementation_count > 0 {
        "reviewed_not_approved"
    } else {
        "waiting_for_independent_materialization_run_authorization_review"
    };
    Ok(HistoricalOutcomeLabelMaterializationRunAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        review_eligible_implementation_count,
        reviewed_implementation_count,
        materialization_runner_registration_eligible_count,
        authorization_status: authorization_status.to_string(),
        materialization_runner_registered: false,
        label_materialization_run_authorized: false,
        label_materialization_started: false,
        outcome_label_write_allowed: false,
        outcome_label_written: false,
        training_authorized: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "第十五阶段只独立复核一份精确绑定、状态为 registered_not_run 的标签物化实现。批准仅允许未来登记隔离物化 runner 规范供再次审查；当前不登记 runner、不运行、不写标签、不推断方向/评级/动作/仓位/奖励，不训练、不写影子组合、不生成订单，也不访问券商或交易。".to_string(),
    })
}

async fn review_run_authorization(
    state: &AppState,
    reviewer_id: &str,
    materialization_implementation_id: &str,
    request: ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest,
) -> Result<HistoricalOutcomeLabelMaterializationRunAuthorizationReview, String> {
    if !valid_id(materialization_implementation_id) {
        return Err("标签物化实现编号无效".to_string());
    }
    let _lock = acquire_lock(
        state,
        &format!("historical-outcome-label-materialization-run-authorization-{materialization_implementation_id}"),
    )
    .await?;
    let eligible = run_authorization_review_eligible_implementations(state)
        .await?
        .into_iter()
        .find(|value| {
            value.implementation.materialization_implementation_id
                == materialization_implementation_id
        })
        .ok_or_else(|| "实现不存在、绑定已经失效或不再具备运行授权复核资格".to_string())?;
    let context = AuthorizationReviewContext::from(&eligible);
    if request.expected_implementation_spec_sha256
        != context.materialization_implementation_spec_sha256
        || request.expected_admission_review_sha256 != context.admission_review_sha256
        || request.expected_validation_sha256 != context.validation_sha256
        || request.expected_output_sha256 != context.output_sha256
        || request.expected_snapshot_sha256 != context.snapshot_sha256
        || request.expected_protocol_sha256 != context.protocol_sha256
    {
        return Err("实现、准入、校验、输出、行情或协议指纹已经变化，请刷新后重试".to_string());
    }
    let previous = latest_review(state, &context).await?;
    if request.expected_review_id.as_deref()
        != previous.as_ref().map(|review| review.review_id.as_str())
        || request.expected_review_sha256.as_deref()
            != previous
                .as_ref()
                .map(|review| review.review_sha256.as_str())
    {
        return Err("标签物化运行授权复核链已经变化，请刷新后重试".to_string());
    }
    let rationale = bounded_required(&request.rationale, MAX_RATIONALE_CHARS, "复核依据")?;
    let approved = request.verdict
        == HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration;
    if approved && !request_all_checks(&request) {
        return Err(
            "批准前必须完成实现、绑定、原始信封、逐位保留、隔离、缺失和全部权限检查".to_string(),
        );
    }
    if approved && !reviewer_is_independent(reviewer_id, &context) {
        return Err("复核人必须独立于实现登记者、准入人、校验人和此前执行链全部角色".to_string());
    }
    let mut review = new_review(
        &context,
        previous.as_ref(),
        reviewer_id,
        request,
        rationale,
        Utc::now(),
    )?;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &context)?;
    write_immutable_json(
        &review_directory(state, &context).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn new_review(
    context: &AuthorizationReviewContext,
    previous: Option<&HistoricalOutcomeLabelMaterializationRunAuthorizationReview>,
    reviewer_id: &str,
    request: ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest,
    rationale: String,
    submitted_at: DateTime<Utc>,
) -> Result<HistoricalOutcomeLabelMaterializationRunAuthorizationReview, String> {
    let approved = request.verdict
        == HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration;
    Ok(
        HistoricalOutcomeLabelMaterializationRunAuthorizationReview {
            schema_version: REVIEW_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            review_id: String::new(),
            review_sha256: String::new(),
            previous_review_id: previous.map(|review| review.review_id.clone()),
            previous_review_sha256: previous.map(|review| review.review_sha256.clone()),
            materialization_implementation_id: context.materialization_implementation_id.clone(),
            materialization_implementation_spec_sha256: context
                .materialization_implementation_spec_sha256
                .clone(),
            materialization_implementation_registered_by: context
                .materialization_implementation_registered_by
                .clone(),
            implementation_name: context.implementation_name.clone(),
            implementation_kind: context.implementation_kind.clone(),
            code_revision: context.code_revision.clone(),
            implementation_status: context.implementation_status.clone(),
            admission_review_id: context.admission_review_id.clone(),
            admission_review_sha256: context.admission_review_sha256.clone(),
            admission_reviewer_id: context.admission_reviewer_id.clone(),
            validation_id: context.validation_id.clone(),
            validation_sha256: context.validation_sha256.clone(),
            validated_by: context.validated_by.clone(),
            execution_invoked_by: context.execution_invoked_by.clone(),
            runner_registered_by: context.runner_registered_by.clone(),
            first_execution_authorization_reviewer_id: context
                .first_execution_authorization_reviewer_id
                .clone(),
            run_authorization_reviewer_id: context.run_authorization_reviewer_id.clone(),
            output_sha256: context.output_sha256.clone(),
            snapshot_id: context.snapshot_id.clone(),
            snapshot_sha256: context.snapshot_sha256.clone(),
            protocol_version: context.protocol_version.clone(),
            protocol_sha256: context.protocol_sha256.clone(),
            admission_known_limitations: context.admission_known_limitations.clone(),
            submitted_at,
            reviewer_id: reviewer_id.to_string(),
            verdict: request.verdict,
            rationale,
            implementation_fingerprint_confirmed: request.implementation_fingerprint_confirmed,
            current_upstream_bindings_confirmed: request.current_upstream_bindings_confirmed,
            code_revision_reproducible_confirmed: request.code_revision_reproducible_confirmed,
            deterministic_raw_envelope_only_confirmed: request
                .deterministic_raw_envelope_only_confirmed,
            exact_metric_bit_preservation_confirmed: request
                .exact_metric_bit_preservation_confirmed,
            provenance_and_limitations_preserved_confirmed: request
                .provenance_and_limitations_preserved_confirmed,
            create_once_isolated_output_confirmed: request.create_once_isolated_output_confirmed,
            missing_data_fail_closed_confirmed: request.missing_data_fail_closed_confirmed,
            no_network_tools_or_production_access_confirmed: request
                .no_network_tools_or_production_access_confirmed,
            no_semantic_action_position_or_reward_inference_confirmed: request
                .no_semantic_action_position_or_reward_inference_confirmed,
            no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed: request
                .no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed,
            reviewer_independent_from_implementation_and_prior_chain: reviewer_is_independent(
                reviewer_id,
                context,
            ),
            materialization_runner_registration_eligible: approved && request_all_checks(&request),
            materialization_runner_registered: false,
            label_materialization_run_authorized: false,
            label_materialization_started: false,
            outcome_label_write_allowed: false,
            outcome_label_written: false,
            training_authorized: false,
            reward_authorized: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        },
    )
}

fn request_all_checks(
    request: &ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest,
) -> bool {
    request.implementation_fingerprint_confirmed
        && request.current_upstream_bindings_confirmed
        && request.code_revision_reproducible_confirmed
        && request.deterministic_raw_envelope_only_confirmed
        && request.exact_metric_bit_preservation_confirmed
        && request.provenance_and_limitations_preserved_confirmed
        && request.create_once_isolated_output_confirmed
        && request.missing_data_fail_closed_confirmed
        && request.no_network_tools_or_production_access_confirmed
        && request.no_semantic_action_position_or_reward_inference_confirmed
        && request.no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed
}

fn review_all_checks(review: &HistoricalOutcomeLabelMaterializationRunAuthorizationReview) -> bool {
    review.implementation_fingerprint_confirmed
        && review.current_upstream_bindings_confirmed
        && review.code_revision_reproducible_confirmed
        && review.deterministic_raw_envelope_only_confirmed
        && review.exact_metric_bit_preservation_confirmed
        && review.provenance_and_limitations_preserved_confirmed
        && review.create_once_isolated_output_confirmed
        && review.missing_data_fail_closed_confirmed
        && review.no_network_tools_or_production_access_confirmed
        && review.no_semantic_action_position_or_reward_inference_confirmed
        && review.no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed
}

fn reviewer_is_independent(reviewer_id: &str, context: &AuthorizationReviewContext) -> bool {
    !reviewer_id.trim().is_empty()
        && [
            &context.materialization_implementation_registered_by,
            &context.admission_reviewer_id,
            &context.validated_by,
            &context.execution_invoked_by,
            &context.runner_registered_by,
            &context.first_execution_authorization_reviewer_id,
            &context.run_authorization_reviewer_id,
        ]
        .into_iter()
        .all(|prior| reviewer_id != prior)
}

fn validate_review(
    review: &HistoricalOutcomeLabelMaterializationRunAuthorizationReview,
    context: &AuthorizationReviewContext,
) -> Result<(), String> {
    let approved = review.verdict
        == HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration;
    let previous_pair_valid =
        review.previous_review_id.is_some() == review.previous_review_sha256.is_some();
    let authority_closed = !review.materialization_runner_registered
        && !review.label_materialization_run_authorized
        && !review.label_materialization_started
        && !review.outcome_label_write_allowed
        && !review.outcome_label_written
        && !review.training_authorized
        && !review.reward_authorized
        && !review.shadow_portfolio_authorized
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
            .is_some_and(|value| !valid_id(value))
        || review
            .previous_review_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256(value))
        || !review_matches_context(review, context)
        || review.implementation_status != "registered_not_run"
        || review.reviewer_id.trim().is_empty()
        || review.rationale.trim().is_empty()
        || review.rationale.chars().count() > MAX_RATIONALE_CHARS
        || (approved && !review_all_checks(review))
        || review.reviewer_independent_from_implementation_and_prior_chain
            != reviewer_is_independent(&review.reviewer_id, context)
        || (approved && !review.reviewer_independent_from_implementation_and_prior_chain)
        || review.materialization_runner_registration_eligible
            != (approved && review_all_checks(review))
        || !authority_closed
    {
        return Err(
            "historical outcome label materialization run authorization is invalid or over-authorized"
                .to_string(),
        );
    }
    Ok(())
}

fn review_matches_context(
    review: &HistoricalOutcomeLabelMaterializationRunAuthorizationReview,
    context: &AuthorizationReviewContext,
) -> bool {
    review.materialization_implementation_id == context.materialization_implementation_id
        && review.materialization_implementation_spec_sha256
            == context.materialization_implementation_spec_sha256
        && review.materialization_implementation_registered_by
            == context.materialization_implementation_registered_by
        && review.implementation_name == context.implementation_name
        && review.implementation_kind == context.implementation_kind
        && review.code_revision == context.code_revision
        && review.implementation_status == context.implementation_status
        && review.admission_review_id == context.admission_review_id
        && review.admission_review_sha256 == context.admission_review_sha256
        && review.admission_reviewer_id == context.admission_reviewer_id
        && review.validation_id == context.validation_id
        && review.validation_sha256 == context.validation_sha256
        && review.validated_by == context.validated_by
        && review.execution_invoked_by == context.execution_invoked_by
        && review.runner_registered_by == context.runner_registered_by
        && review.first_execution_authorization_reviewer_id
            == context.first_execution_authorization_reviewer_id
        && review.run_authorization_reviewer_id == context.run_authorization_reviewer_id
        && review.output_sha256 == context.output_sha256
        && review.snapshot_id == context.snapshot_id
        && review.snapshot_sha256 == context.snapshot_sha256
        && review.protocol_version == context.protocol_version
        && review.protocol_sha256 == context.protocol_sha256
        && review.admission_known_limitations == context.admission_known_limitations
}

fn review_fingerprint(
    review: &HistoricalOutcomeLabelMaterializationRunAuthorizationReview,
) -> Result<String, String> {
    let mut value = serde_json::to_value(review).map_err(|error| error.to_string())?;
    let object = value.as_object_mut().ok_or_else(|| {
        "materialization run authorization fingerprint payload is invalid".to_string()
    })?;
    object.remove("review_id");
    object.remove("review_sha256");
    hash_serializable(&value)
}

async fn latest_review(
    state: &AppState,
    context: &AuthorizationReviewContext,
) -> Result<Option<HistoricalOutcomeLabelMaterializationRunAuthorizationReview>, String> {
    let directory = review_directory(state, context);
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
        let review: HistoricalOutcomeLabelMaterializationRunAuthorizationReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, context)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
        {
            return Err("materialization run authorization filename mismatch".to_string());
        }
        if !ids.insert(review.review_id.clone()) || !hashes.insert(review.review_sha256.clone()) {
            return Err("materialization run authorization contains duplicates".to_string());
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[HistoricalOutcomeLabelMaterializationRunAuthorizationReview],
) -> Result<Option<HistoricalOutcomeLabelMaterializationRunAuthorizationReview>, String> {
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
        return Err("materialization run authorization chain has invalid tips".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("materialization run authorization chain has a cycle".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(previous_id), Some(previous_sha256)) => {
                let previous = by_id.get(previous_id.as_str()).ok_or_else(|| {
                    "materialization run authorization chain is disconnected".to_string()
                })?;
                if previous.review_sha256 != *previous_sha256 {
                    return Err(
                        "materialization run authorization previous hash mismatch".to_string()
                    );
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => {
                return Err(
                    "materialization run authorization previous link is invalid".to_string()
                );
            }
        };
    }
    if visited.len() != reviews.len() {
        return Err("materialization run authorization chain is disconnected".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_directory(state: &AppState, context: &AuthorizationReviewContext) -> PathBuf {
    decision_root(state)
        .join("historical-outcome-label-materialization-run-authorization-reviews")
        .join(&context.materialization_implementation_id)
}

struct MaterializationRunAuthorizationLock {
    path: PathBuf,
}

impl Drop for MaterializationRunAuthorizationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(
    state: &AppState,
    name: &str,
) -> Result<MaterializationRunAuthorizationLock, String> {
    if !valid_id(name) {
        return Err("invalid materialization run authorization lock name".to_string());
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
        .map_err(|_| "已有标签物化运行授权正在复核".to_string())?;
    Ok(MaterializationRunAuthorizationLock { path })
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

    fn context() -> AuthorizationReviewContext {
        AuthorizationReviewContext {
            materialization_implementation_id: "materializer-1".to_string(),
            materialization_implementation_spec_sha256: "a".repeat(64),
            materialization_implementation_registered_by: "registrant".to_string(),
            implementation_name: "raw envelope".to_string(),
            implementation_kind: HistoricalOutcomeLabelMaterializationImplementationKind::DeterministicRawValidatedOutcomeEnvelope,
            code_revision: "oldwang@abc123".to_string(),
            implementation_status: "registered_not_run".to_string(),
            admission_review_id: "admission-1".to_string(),
            admission_review_sha256: "b".repeat(64),
            admission_reviewer_id: "admission-reviewer".to_string(),
            validation_id: "validation-1".to_string(),
            validation_sha256: "c".repeat(64),
            validated_by: "validator".to_string(),
            execution_invoked_by: "invoker".to_string(),
            runner_registered_by: "runner-registrant".to_string(),
            first_execution_authorization_reviewer_id: "first-reviewer".to_string(),
            run_authorization_reviewer_id: "run-reviewer".to_string(),
            output_sha256: "d".repeat(64),
            snapshot_id: "snapshot-1".to_string(),
            snapshot_sha256: "e".repeat(64),
            protocol_version: "protocol-v1".to_string(),
            protocol_sha256: "f".repeat(64),
            admission_known_limitations: "raw market observation is not an investment action".to_string(),
        }
    }

    fn request(
        verdict: HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict,
    ) -> ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest {
        let context = context();
        ReviewHistoricalOutcomeLabelMaterializationRunAuthorizationRequest {
            expected_review_id: None,
            expected_review_sha256: None,
            expected_implementation_spec_sha256: context.materialization_implementation_spec_sha256,
            expected_admission_review_sha256: context.admission_review_sha256,
            expected_validation_sha256: context.validation_sha256,
            expected_output_sha256: context.output_sha256,
            expected_snapshot_sha256: context.snapshot_sha256,
            expected_protocol_sha256: context.protocol_sha256,
            verdict,
            rationale: "independently verified".to_string(),
            implementation_fingerprint_confirmed: true,
            current_upstream_bindings_confirmed: true,
            code_revision_reproducible_confirmed: true,
            deterministic_raw_envelope_only_confirmed: true,
            exact_metric_bit_preservation_confirmed: true,
            provenance_and_limitations_preserved_confirmed: true,
            create_once_isolated_output_confirmed: true,
            missing_data_fail_closed_confirmed: true,
            no_network_tools_or_production_access_confirmed: true,
            no_semantic_action_position_or_reward_inference_confirmed: true,
            no_label_training_reward_shadow_order_broker_or_trading_authority_confirmed: true,
        }
    }

    fn signed_review(
        reviewer: &str,
        verdict: HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict,
        previous: Option<&HistoricalOutcomeLabelMaterializationRunAuthorizationReview>,
    ) -> HistoricalOutcomeLabelMaterializationRunAuthorizationReview {
        let context = context();
        let request = request(verdict);
        let rationale = request.rationale.clone();
        let mut review =
            new_review(&context, previous, reviewer, request, rationale, Utc::now()).unwrap();
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        review
    }

    #[test]
    fn approval_only_enables_future_runner_registration() {
        let context = context();
        let review = signed_review(
            "independent-reviewer",
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration,
            None,
        );
        assert!(review.materialization_runner_registration_eligible);
        assert!(!review.materialization_runner_registered);
        assert!(!review.label_materialization_run_authorized);
        assert!(!review.label_materialization_started);
        assert!(!review.outcome_label_write_allowed);
        assert!(!review.outcome_label_written);
        assert!(!review.training_authorized);
        assert!(!review.trading_authorized);
        assert!(validate_review(&review, &context).is_ok());
    }

    #[test]
    fn approval_requires_every_check() {
        let context = context();
        let mut request = request(
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration,
        );
        request.exact_metric_bit_preservation_confirmed = false;
        let rationale = request.rationale.clone();
        let mut review = new_review(
            &context,
            None,
            "independent-reviewer",
            request,
            rationale,
            Utc::now(),
        )
        .unwrap();
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review, &context).is_err());
    }

    #[test]
    fn every_prior_chain_actor_is_forbidden_from_approving() {
        let context = context();
        for reviewer in [
            "registrant",
            "admission-reviewer",
            "validator",
            "invoker",
            "runner-registrant",
            "first-reviewer",
            "run-reviewer",
        ] {
            let review = signed_review(
                reviewer,
                HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration,
                None,
            );
            assert!(validate_review(&review, &context).is_err(), "{reviewer}");
        }
    }

    #[test]
    fn exact_current_binding_is_required() {
        let review = signed_review(
            "independent-reviewer",
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ChangesRequested,
            None,
        );
        let mut changed = context();
        changed.output_sha256 = "9".repeat(64);
        assert!(validate_review(&review, &changed).is_err());
    }

    #[test]
    fn fingerprint_tampering_is_rejected() {
        let context = context();
        let mut review = signed_review(
            "independent-reviewer",
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ChangesRequested,
            None,
        );
        review.rationale = "changed after signing".to_string();
        assert!(validate_review(&review, &context).is_err());
    }

    #[test]
    fn over_authorized_label_or_trade_state_is_rejected() {
        let context = context();
        let mut review = signed_review(
            "independent-reviewer",
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration,
            None,
        );
        review.outcome_label_write_allowed = true;
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review, &context).is_err());
        let mut review = signed_review(
            "independent-reviewer",
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ApprovedForMaterializationRunnerRegistration,
            None,
        );
        review.trading_authorized = true;
        review.review_sha256 = review_fingerprint(&review).unwrap();
        review.review_id = review.review_sha256[..32].to_string();
        assert!(validate_review(&review, &context).is_err());
    }

    #[test]
    fn append_only_chain_rejects_forks_and_wrong_previous_hash() {
        let first = signed_review(
            "independent-reviewer",
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::ChangesRequested,
            None,
        );
        let second = signed_review(
            "second-independent-reviewer",
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::Rejected,
            Some(&first),
        );
        assert_eq!(
            latest_review_from_records(&[first.clone(), second.clone()])
                .unwrap()
                .unwrap()
                .review_id,
            second.review_id
        );
        let fork = signed_review(
            "third-independent-reviewer",
            HistoricalOutcomeLabelMaterializationRunAuthorizationVerdict::Rejected,
            Some(&first),
        );
        assert!(latest_review_from_records(&[first.clone(), second, fork]).is_err());
        let mut wrong = first.clone();
        wrong.previous_review_id = Some("missing-review".to_string());
        wrong.previous_review_sha256 = Some("9".repeat(64));
        assert!(latest_review_from_records(&[wrong]).is_err());
    }
}
