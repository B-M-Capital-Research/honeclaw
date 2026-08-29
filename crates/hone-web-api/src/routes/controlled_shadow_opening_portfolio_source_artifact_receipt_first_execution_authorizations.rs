//! Stage 130 chain-external first-execution authorization for one exact Stage 129 receiver.
//!
//! The server only inspects a content-addressed, read-only receiver artifact and its self-hashed
//! manifest. Approval is single-use and expires after 24 hours. This module has no upload or
//! execution endpoint and never receives, mounts, reads, parses, or stores a source artifact.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::controlled_shadow_opening_portfolio_source_artifact_receipt_isolated_receivers::{
    OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
    isolated_receivers_for_first_execution_authorization_review,
    validate_isolated_receiver_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-first-execution-authorization-registry-v1";
const MANIFEST_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-reproduced-receiver-manifest-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-first-execution-authorization-review-v1";
const POLICY_VERSION: &str = "hone-opening-portfolio-source-artifact-receipt-first-execution-authorization-v1-server-rehashed-single-use-24h";
const ARTIFACT_FILE_NAME: &str = "receiver.artifact";
const MANIFEST_FILE_NAME: &str = "manifest.json";
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TEXT_CHARS: usize = 4_000;
const AUTHORIZATION_VALID_HOURS: i64 = 24;
const ONE_SHOT_ATTEMPT_LIMIT: u8 = 1;
const NEXT_GATE: &str = "stage_131_claim_first_source_artifact_receipt_attempt";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict {
    ApprovedForOneFutureClaimFirstSourceArtifactReceiptAttempt,
    ChangesRequestedRebuildArtifact,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest {
    pub schema_version: String,
    pub manifest_sha256: String,
    pub isolated_receiver_id: String,
    pub isolated_receiver_spec_sha256: String,
    pub receiver_contract_sha256: String,
    pub receiver_spec_revision: String,
    pub receiver_code_revision: String,
    pub receiver_artifact_sha256: String,
    pub artifact_byte_length: u64,
    pub artifact_file_name: String,
    pub artifact_media_type: String,
    pub source_bundle_sha256: String,
    pub artifact_reproduction_procedure_sha256: String,
    pub runtime_identity: String,
    pub runtime_version: String,
    pub reproduced_at: DateTime<Utc>,
    pub reproduced_by: String,
    pub source_and_artifact_reproduced_from_immutable_revision: bool,
    pub artifact_is_read_only_regular_file: bool,
    pub artifact_was_not_executed: bool,
    pub source_artifact_was_not_received_or_read: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptReceiverArtifactInspection {
    pub custody_locator: String,
    pub manifest_present: bool,
    pub artifact_present: bool,
    pub manifest: Option<OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest>,
    pub server_computed_artifact_sha256: Option<String>,
    pub server_observed_artifact_byte_length: Option<u64>,
    pub artifact_verified: bool,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRequest {
    expected_review_id: Option<String>,
    expected_review_sha256: Option<String>,
    expected_isolated_receiver_id: String,
    expected_isolated_receiver_spec_sha256: String,
    expected_receiver_contract_sha256: String,
    expected_receiver_spec_revision: String,
    expected_receiver_code_revision: String,
    expected_receiver_artifact_sha256: String,
    expected_stage_128_review_id: String,
    expected_stage_128_review_sha256: String,
    expected_stage_128_independent_audit_sha256: String,
    expected_stage_127_implementation_sha256: String,
    expected_stage_127_implementation_contract_sha256: String,
    expected_stage_126_review_sha256: String,
    expected_stage_125_registration_sha256: String,
    expected_stage_125_specification_sha256: String,
    expected_artifact_manifest_sha256: String,
    artifact_reproduction_review_evidence: String,
    sandbox_contract_review_evidence: String,
    verdict: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict,
    rationale: String,
    exact_current_stage_51_through_stage_129_binding_confirmed: bool,
    reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed: bool,
    server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: bool,
    self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: bool,
    artifact_builder_and_reviewer_separation_confirmed: bool,
    all_eight_receipt_functions_and_original_pdf_csv_json_formats_remain_bound_confirmed: bool,
    exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: bool,
    future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: bool,
    future_private_quarantine_hash_length_magic_structure_and_atomic_create_new_confirmed: bool,
    future_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: bool,
    future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: bool,
    future_receipt_validation_snapshot_materialization_validation_and_admission_separate_confirmed:
        bool,
    fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    authorization_single_use_24_hour_expiry_and_stage_131_claim_separation_confirmed: bool,
    no_upload_source_bytes_runtime_mount_input_read_receipt_or_snapshot_created_confirmed: bool,
    no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    approval_only_opens_future_stage_131_claim_first_attempt_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub receiver: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
    pub artifact_manifest: OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest,
    pub submitted_at: DateTime<Utc>,
    pub authorization_valid_until: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub server_computed_artifact_sha256: String,
    pub server_observed_artifact_byte_length: u64,
    pub artifact_reproduction_review_evidence: String,
    pub sandbox_contract_review_evidence: String,
    pub verdict: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict,
    pub rationale: String,
    pub exact_current_stage_51_through_stage_129_binding_confirmed: bool,
    pub reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed:
        bool,
    pub server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: bool,
    pub self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: bool,
    pub artifact_builder_and_reviewer_separation_confirmed: bool,
    pub all_eight_receipt_functions_and_original_pdf_csv_json_formats_remain_bound_confirmed: bool,
    pub exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: bool,
    pub future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: bool,
    pub future_private_quarantine_hash_length_magic_structure_and_atomic_create_new_confirmed: bool,
    pub future_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: bool,
    pub future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: bool,
    pub future_receipt_validation_snapshot_materialization_validation_and_admission_separate_confirmed:
        bool,
    pub fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed:
        bool,
    pub authorization_single_use_24_hour_expiry_and_stage_131_claim_separation_confirmed: bool,
    pub no_upload_source_bytes_runtime_mount_input_read_receipt_or_snapshot_created_confirmed: bool,
    pub no_environment_secret_network_tool_subprocess_or_production_io_confirmed: bool,
    pub no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub approval_only_opens_future_stage_131_claim_first_attempt_confirmed: bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub one_shot_execution_attempt_limit: u8,
    pub one_future_claim_first_source_artifact_receipt_attempt_authorized: bool,
    pub authorization_claimed: bool,
    pub upload_endpoint_present: bool,
    pub executable_entrypoint_present: bool,
    pub runtime_instantiated: bool,
    pub source_artifact_received_or_read: bool,
    pub receipt_manifest_created: bool,
    pub opening_portfolio_snapshot_materialized: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationItem {
    pub receiver: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
    pub artifact_inspection: OpeningPortfolioSourceArtifactReceiptReceiverArtifactInspection,
    pub latest_review:
        Option<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview>,
    pub authorization_unexpired: bool,
    pub future_claim_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationItem>,
    pub receiver_count: usize,
    pub artifact_verified_receiver_count: usize,
    pub artifact_pending_receiver_count: usize,
    pub review_eligible_receiver_count: usize,
    pub reviewed_receiver_count: usize,
    pub approved_receiver_count: usize,
    pub unexpired_authorization_count: usize,
    pub future_claim_eligible_count: usize,
    pub authorization_status: String,
    pub next_gate: String,
    pub upload_endpoint_present: bool,
    pub runtime_instantiated: bool,
    pub source_artifact_received_or_read: bool,
    pub receipt_manifest_created: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReadinessSummary {
    pub receiver_count: usize,
    pub artifact_verified_receiver_count: usize,
    pub artifact_pending_receiver_count: usize,
    pub review_eligible_receiver_count: usize,
    pub reviewed_receiver_count: usize,
    pub approved_receiver_count: usize,
    pub unexpired_authorization_count: usize,
    pub future_claim_eligible_count: usize,
    pub authorization_status: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ApprovedOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization {
    pub receiver: OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
    pub review: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
}

pub(crate) async fn handle_get_opening_portfolio_source_artifact_receipt_first_execution_authorizations(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    if let Err(response) =
        crate::routes::public_admin::require_public_admin_for_read(&state, &headers)
    {
        return response;
    }
    match build_registry(&state, Utc::now()).await {
        Ok(value) => Json(value).into_response(),
        Err(error) => {
            warn!(%error, "Stage 130 first execution authorization registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "来源工件接收器首次执行授权复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_opening_portfolio_source_artifact_receipt_first_execution_authorization(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(isolated_receiver_id): AxumPath<String>,
    Json(request): Json<
        ReviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRequest,
    >,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &isolated_receiver_id, request).await {
        Ok(_) => match build_registry(&state, Utc::now()).await {
            Ok(value) => Json(value).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn opening_portfolio_source_artifact_receipt_first_execution_authorization_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReadinessSummary, String>
{
    let value = build_registry(state, Utc::now()).await?;
    Ok(
        OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReadinessSummary {
            receiver_count: value.receiver_count,
            artifact_verified_receiver_count: value.artifact_verified_receiver_count,
            artifact_pending_receiver_count: value.artifact_pending_receiver_count,
            review_eligible_receiver_count: value.review_eligible_receiver_count,
            reviewed_receiver_count: value.reviewed_receiver_count,
            approved_receiver_count: value.approved_receiver_count,
            unexpired_authorization_count: value.unexpired_authorization_count,
            future_claim_eligible_count: value.future_claim_eligible_count,
            authorization_status: value.authorization_status,
        },
    )
}

pub(crate) async fn opening_portfolio_source_artifact_receipt_first_execution_authorizations_for_future_claim(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<Vec<ApprovedOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization>, String> {
    let registry = build_registry(state, now).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            if !item.future_claim_eligible {
                return None;
            }
            item.latest_review.map(|review| {
                ApprovedOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorization {
                    receiver: item.receiver,
                    review,
                }
            })
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
    now: DateTime<Utc>,
) -> Result<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry, String> {
    let receivers = isolated_receivers_for_first_execution_authorization_review(state).await?;
    let claimed_review_ids = super::controlled_shadow_opening_portfolio_source_artifact_receipt_execution_attempt_claims::claimed_opening_portfolio_source_artifact_receipt_first_execution_authorization_review_ids(state).await?;
    let mut items = Vec::with_capacity(receivers.len());
    for receiver in receivers {
        let artifact_inspection = inspect_artifact(state, &receiver).await?;
        let latest_review = latest_review(state, &receiver).await?;
        let authorization_unexpired = latest_review.as_ref().is_some_and(|review| {
            review.one_future_claim_first_source_artifact_receipt_attempt_authorized
                && now >= review.submitted_at
                && now < review.authorization_valid_until
                && artifact_inspection_matches_review(&artifact_inspection, review)
        });
        let authorization_claimed = latest_review
            .as_ref()
            .is_some_and(|review| claimed_review_ids.contains(&review.review_id));
        items.push(
            OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationItem {
                receiver,
                artifact_inspection,
                latest_review,
                authorization_unexpired,
                future_claim_eligible: authorization_unexpired && !authorization_claimed,
            },
        );
    }
    let receiver_count = items.len();
    let artifact_verified_receiver_count = items
        .iter()
        .filter(|item| item.artifact_inspection.artifact_verified)
        .count();
    let reviewed_receiver_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let approved_receiver_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                review.one_future_claim_first_source_artifact_receipt_attempt_authorized
            })
        })
        .count();
    let unexpired_authorization_count = items
        .iter()
        .filter(|item| item.authorization_unexpired)
        .count();
    let future_claim_eligible_count = items
        .iter()
        .filter(|item| item.future_claim_eligible)
        .count();
    let authorization_status = if receiver_count == 0 {
        "waiting_for_current_stage_129_isolated_receiver"
    } else if future_claim_eligible_count > 0 {
        "approved_for_one_future_stage_131_claim_first_attempt_not_started"
    } else if !claimed_review_ids.is_empty() {
        "stage_130_authorization_permanently_consumed_by_stage_131_claim"
    } else if artifact_verified_receiver_count == 0 {
        "waiting_for_server_verifiable_reproduced_receiver_artifact"
    } else if reviewed_receiver_count > 0 {
        "reviewed_not_currently_authorized"
    } else {
        "waiting_for_chain_external_first_execution_authorization_review"
    };
    Ok(OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        receiver_count,
        artifact_verified_receiver_count,
        artifact_pending_receiver_count: receiver_count.saturating_sub(artifact_verified_receiver_count),
        review_eligible_receiver_count: artifact_verified_receiver_count,
        reviewed_receiver_count,
        approved_receiver_count,
        unexpired_authorization_count,
        future_claim_eligible_count,
        authorization_status: authorization_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        upload_endpoint_present: false,
        runtime_instantiated: false,
        source_artifact_received_or_read: false,
        receipt_manifest_created: false,
        opening_portfolio_snapshot_admitted: false,
        ledger_created: false,
        position_or_cash_written: false,
        nav_or_performance_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 130 只允许完整 Stage 51–129 责任链之外的新复核者核对服务端只读内容寻址接收器工件、自哈希 manifest、代码版本、复现步骤与隔离合同，并授予 24 小时内最多一次的未来 Stage 131 claim-first 资格。当前没有上传入口、来源字节接收/读取、runtime、receipt、期初组合快照、金融事件白名单、账本、持仓、现金、净值/绩效、模型/指标、训练/RL/reward、订单、券商或交易能力。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    isolated_receiver_id: &str,
    request: ReviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRequest,
) -> Result<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview, String> {
    if !valid_id(isolated_receiver_id)
        || request.expected_isolated_receiver_id != isolated_receiver_id
    {
        return Err("Stage 129 隔离接收器 ID 无效或与路径不一致".to_string());
    }
    let _lock = acquire_lock(state, isolated_receiver_id).await?;
    let receiver = isolated_receivers_for_first_execution_authorization_review(state)
        .await?
        .into_iter()
        .find(|value| value.isolated_receiver_id == isolated_receiver_id)
        .ok_or_else(|| "当前没有这条绑定有效且可独立复核的 Stage 129 接收器".to_string())?;
    validate_expected_binding(&receiver, &request)?;
    let artifact = inspect_artifact(state, &receiver).await?;
    if !artifact.artifact_verified {
        return Err("服务端尚未找到并核验只读内容寻址接收器工件与 manifest".to_string());
    }
    let manifest = artifact
        .manifest
        .clone()
        .ok_or_else(|| "已核验工件缺少 manifest".to_string())?;
    if request.expected_artifact_manifest_sha256 != manifest.manifest_sha256 {
        return Err("工件 manifest 已变化，请刷新后重试".to_string());
    }
    let latest = latest_review(state, &receiver).await?;
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_review_sha256.as_deref()
    {
        return Err("Stage 130 授权复核链已经变化，请刷新后重试".to_string());
    }
    if latest.as_ref().is_some_and(|review| {
        review.one_future_claim_first_source_artifact_receipt_attempt_authorized
    }) {
        return Err("已批准的 Stage 130 授权复核链不得继续追加".to_string());
    }
    let excluded_prior_actor_ids =
        expected_excluded_prior_actor_ids(&receiver, &manifest, latest.as_ref());
    let independent = !reviewer_id.trim().is_empty()
        && !excluded_prior_actor_ids
            .iter()
            .any(|value| value == reviewer_id);
    if request
        .reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed
        != independent
    {
        return Err("复核者独立性确认与 Stage 51–129 及工件构建者责任链不一致".to_string());
    }
    if request.artifact_builder_and_reviewer_separation_confirmed
        != (manifest.reproduced_by != reviewer_id)
    {
        return Err("工件构建者与 Stage 130 复核者分离确认不一致".to_string());
    }
    let approved = request.verdict == OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict::ApprovedForOneFutureClaimFirstSourceArtifactReceiptAttempt;
    let submitted_at = Utc::now();
    let mut review = OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(), review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        receiver, artifact_manifest: manifest,
        submitted_at, authorization_valid_until: submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS),
        reviewer_id: reviewer_id.to_string(), excluded_prior_actor_ids,
        server_computed_artifact_sha256: artifact.server_computed_artifact_sha256.ok_or_else(|| "缺少服务端工件摘要".to_string())?,
        server_observed_artifact_byte_length: artifact.server_observed_artifact_byte_length.ok_or_else(|| "缺少服务端工件长度".to_string())?,
        artifact_reproduction_review_evidence: bounded_required(&request.artifact_reproduction_review_evidence, "工件复现复核证据")?,
        sandbox_contract_review_evidence: bounded_required(&request.sandbox_contract_review_evidence, "隔离合同复核证据")?,
        verdict: request.verdict, rationale: bounded_required(&request.rationale, "复核依据")?,
        exact_current_stage_51_through_stage_129_binding_confirmed: request.exact_current_stage_51_through_stage_129_binding_confirmed,
        reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed: request.reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed,
        server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed: request.server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed,
        self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed: request.self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed,
        artifact_builder_and_reviewer_separation_confirmed: request.artifact_builder_and_reviewer_separation_confirmed,
        all_eight_receipt_functions_and_original_pdf_csv_json_formats_remain_bound_confirmed: request.all_eight_receipt_functions_and_original_pdf_csv_json_formats_remain_bound_confirmed,
        exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed: request.exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed,
        future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed: request.future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed,
        future_private_quarantine_hash_length_magic_structure_and_atomic_create_new_confirmed: request.future_private_quarantine_hash_length_magic_structure_and_atomic_create_new_confirmed,
        future_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed: request.future_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed,
        future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed: request.future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed,
        future_receipt_validation_snapshot_materialization_validation_and_admission_separate_confirmed: request.future_receipt_validation_snapshot_materialization_validation_and_admission_separate_confirmed,
        fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed: request.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed,
        authorization_single_use_24_hour_expiry_and_stage_131_claim_separation_confirmed: request.authorization_single_use_24_hour_expiry_and_stage_131_claim_separation_confirmed,
        no_upload_source_bytes_runtime_mount_input_read_receipt_or_snapshot_created_confirmed: request.no_upload_source_bytes_runtime_mount_input_read_receipt_or_snapshot_created_confirmed,
        no_environment_secret_network_tool_subprocess_or_production_io_confirmed: request.no_environment_secret_network_tool_subprocess_or_production_io_confirmed,
        no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed: request.no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_stage_131_claim_first_attempt_confirmed: request.approval_only_opens_future_stage_131_claim_first_attempt_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request.no_unconfirmed_hari_or_old_wang_logic_claimed,
        one_shot_execution_attempt_limit: ONE_SHOT_ATTEMPT_LIMIT,
        one_future_claim_first_source_artifact_receipt_attempt_authorized: false,
        authorization_claimed: false, upload_endpoint_present: false, executable_entrypoint_present: false,
        runtime_instantiated: false, source_artifact_received_or_read: false, receipt_manifest_created: false,
        opening_portfolio_snapshot_materialized: false, opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false, ledger_created: false, position_or_cash_written: false,
        nav_or_performance_written: false, model_or_metric_store_written: false,
        training_or_rl_feedback_authorized: false, reward_authorized: false,
        order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
    };
    review.one_future_claim_first_source_artifact_receipt_attempt_authorized =
        approved && review_checks(&review);
    if approved && !review.one_future_claim_first_source_artifact_receipt_attempt_authorized {
        return Err("批准 Stage 130 授权前必须逐项完成全部确认".to_string());
    }
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &review.receiver)?;
    write_immutable_json(
        &review_directory(state, &review.receiver).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn validate_expected_binding(
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
    request: &ReviewOpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationRequest,
) -> Result<(), String> {
    validate_isolated_receiver_for_review(receiver)?;
    let contract = &receiver.receiver_contract;
    let implementation = &receiver.implementation;
    let source = &implementation.implementation_contract;
    let review = &receiver.implementation_review;
    if request.expected_isolated_receiver_spec_sha256 != receiver.isolated_receiver_spec_sha256
        || request.expected_receiver_contract_sha256 != contract.contract_sha256
        || request.expected_receiver_spec_revision != contract.receiver_spec_revision
        || request.expected_receiver_code_revision != contract.proposed_receiver_code_revision
        || request.expected_receiver_artifact_sha256 != contract.proposed_receiver_artifact_sha256
        || request.expected_stage_128_review_id != review.review_id
        || request.expected_stage_128_review_sha256 != review.review_sha256
        || request.expected_stage_128_independent_audit_sha256
            != review.independent_audit.audit_sha256
        || request.expected_stage_127_implementation_sha256 != implementation.implementation_sha256
        || request.expected_stage_127_implementation_contract_sha256 != source.contract_sha256
        || request.expected_stage_126_review_sha256 != source.stage_126_review_sha256
        || request.expected_stage_125_registration_sha256 != source.stage_125_registration_sha256
        || request.expected_stage_125_specification_sha256 != source.stage_125_specification_sha256
    {
        return Err("Stage 125–129 接收器、工件身份或完整上游绑定已经变化".to_string());
    }
    Ok(())
}

fn review_checks(
    review: &OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
) -> bool {
    review.exact_current_stage_51_through_stage_129_binding_confirmed
        && review.reviewer_independent_from_stage_129_registrar_builder_and_complete_prior_chain_confirmed
        && review.server_rehashed_read_only_regular_artifact_and_digest_matched_confirmed
        && review.self_hashed_manifest_revision_runtime_and_reproduction_procedure_matched_confirmed
        && review.artifact_builder_and_reviewer_separation_confirmed
        && review.all_eight_receipt_functions_and_original_pdf_csv_json_formats_remain_bound_confirmed
        && review.exact_64_mib_artifact_256_mib_receipt_and_64_artifact_ceilings_preserved_confirmed
        && review.future_administrator_authenticated_stream_only_and_no_remote_fetch_confirmed
        && review.future_private_quarantine_hash_length_magic_structure_and_atomic_create_new_confirmed
        && review.future_pseudonymization_secret_redaction_encryption_and_redacted_manifest_confirmed
        && review.future_input_read_only_content_addressed_and_output_create_once_untrusted_confirmed
        && review.future_receipt_validation_snapshot_materialization_validation_and_admission_separate_confirmed
        && review.fixed_unprivileged_identity_read_only_root_ephemeral_workdir_and_resource_limits_confirmed
        && review.authorization_single_use_24_hour_expiry_and_stage_131_claim_separation_confirmed
        && review.no_upload_source_bytes_runtime_mount_input_read_receipt_or_snapshot_created_confirmed
        && review.no_environment_secret_network_tool_subprocess_or_production_io_confirmed
        && review.no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && review.approval_only_opens_future_stage_131_claim_first_attempt_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_review(
    review: &OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> Result<(), String> {
    validate_isolated_receiver_for_review(receiver)?;
    validate_manifest(&review.artifact_manifest, receiver)?;
    let approved = review.verdict == OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationVerdict::ApprovedForOneFutureClaimFirstSourceArtifactReceiptAttempt;
    let authority_closed = !review.authorization_claimed
        && !review.upload_endpoint_present
        && !review.executable_entrypoint_present
        && !review.runtime_instantiated
        && !review.source_artifact_received_or_read
        && !review.receipt_manifest_created
        && !review.opening_portfolio_snapshot_materialized
        && !review.opening_portfolio_snapshot_admitted
        && !review.financial_event_allowlist_nonempty
        && !review.ledger_created
        && !review.position_or_cash_written
        && !review.nav_or_performance_written
        && !review.model_or_metric_store_written
        && !review.training_or_rl_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let valid = review.schema_version == REVIEW_SCHEMA_VERSION
        && review.policy_version == POLICY_VERSION
        && valid_id(&review.review_id)
        && valid_sha256(&review.review_sha256)
        && review.review_id == review.review_sha256[..32]
        && review.review_sha256 == fingerprint_without(review, &["review_id", "review_sha256"])?
        && review.previous_review_id.is_some() == review.previous_review_sha256.is_some()
        && review.previous_review_id.as_deref().is_none_or(valid_id)
        && review
            .previous_review_sha256
            .as_deref()
            .is_none_or(valid_sha256)
        && &review.receiver == receiver
        && review.authorization_valid_until
            == review.submitted_at + TimeDelta::hours(AUTHORIZATION_VALID_HOURS)
        && review.artifact_manifest.reproduced_at <= review.submitted_at
        && !review.reviewer_id.trim().is_empty()
        && !review.rationale.trim().is_empty()
        && !review
            .artifact_reproduction_review_evidence
            .trim()
            .is_empty()
        && !review.sandbox_contract_review_evidence.trim().is_empty()
        && review.rationale.chars().count() <= MAX_TEXT_CHARS
        && review.artifact_reproduction_review_evidence.chars().count() <= MAX_TEXT_CHARS
        && review.sandbox_contract_review_evidence.chars().count() <= MAX_TEXT_CHARS
        && !review
            .excluded_prior_actor_ids
            .iter()
            .any(|value| value == &review.reviewer_id)
        && sorted_unique(&review.excluded_prior_actor_ids)
        && review.artifact_manifest.reproduced_by != review.reviewer_id
        && review.server_computed_artifact_sha256
            == review.artifact_manifest.receiver_artifact_sha256
        && review.server_observed_artifact_byte_length
            == review.artifact_manifest.artifact_byte_length
        && review.one_shot_execution_attempt_limit == ONE_SHOT_ATTEMPT_LIMIT
        && (!approved || review_checks(review))
        && review.one_future_claim_first_source_artifact_receipt_attempt_authorized
            == (approved && review_checks(review))
        && authority_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 130 来源工件接收器首次执行授权复核无效、漂移或越权".to_string())
}

pub(crate) fn validate_opening_portfolio_source_artifact_receipt_first_execution_authorization_for_claim(
    review: &OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
) -> Result<(), String> {
    validate_review(review, &review.receiver)
}

/// Stage 132 may reopen the exact Stage 130-reviewed declarative receiver artifact after its
/// irreversible start marker exists. The bytes are returned only after the read-only file,
/// manifest, digest and length are independently rechecked; callers must never spawn them.
pub(crate) async fn read_revalidated_opening_portfolio_source_artifact_receipt_receiver_artifact_for_execution(
    state: &AppState,
    review: &OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
) -> Result<Vec<u8>, String> {
    validate_review(review, &review.receiver)?;
    let inspection = inspect_artifact(state, &review.receiver).await?;
    if !artifact_inspection_matches_review(&inspection, review) {
        return Err("Stage 130 已复核接收器工件或 manifest 已漂移".to_string());
    }
    let bytes =
        tokio::fs::read(artifact_directory(state, &review.receiver).join(ARTIFACT_FILE_NAME))
            .await
            .map_err(|error| error.to_string())?;
    if sha256_bytes(&bytes) != review.server_computed_artifact_sha256
        || bytes.len() as u64 != review.server_observed_artifact_byte_length
    {
        return Err("Stage 130 已复核接收器工件重读摘要或长度不一致".to_string());
    }
    Ok(bytes)
}

async fn inspect_artifact(
    state: &AppState,
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> Result<OpeningPortfolioSourceArtifactReceiptReceiverArtifactInspection, String> {
    let directory = artifact_directory(state, receiver);
    let locator = format!(
        "opening-portfolio-source-artifact-receipt-reproduced-receivers/{}/{}",
        receiver.isolated_receiver_id, receiver.receiver_contract.proposed_receiver_artifact_sha256
    );
    let manifest_path = directory.join(MANIFEST_FILE_NAME);
    let artifact_path = directory.join(ARTIFACT_FILE_NAME);
    let manifest_metadata = metadata_if_present(&manifest_path).await?;
    let artifact_metadata = metadata_if_present(&artifact_path).await?;
    let artifact_present = artifact_metadata.is_some();
    let Some(manifest_metadata) = manifest_metadata else {
        return Ok(pending_inspection(
            locator,
            false,
            artifact_present,
            "reproduction_manifest_missing",
        ));
    };
    let Some(artifact_metadata) = artifact_metadata else {
        return Ok(pending_inspection(
            locator,
            true,
            false,
            "reproduced_artifact_missing",
        ));
    };
    if !safe_read_only_regular_file(&manifest_metadata, MAX_MANIFEST_BYTES)
        || !safe_read_only_regular_file(&artifact_metadata, MAX_ARTIFACT_BYTES)
    {
        return Ok(pending_inspection(
            locator,
            true,
            true,
            "artifact_or_manifest_not_read_only_regular_file",
        ));
    }
    let manifest_bytes = tokio::fs::read(&manifest_path)
        .await
        .map_err(|error| error.to_string())?;
    let manifest: OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest =
        match serde_json::from_slice(&manifest_bytes) {
            Ok(value) => value,
            Err(_) => {
                return Ok(pending_inspection(
                    locator,
                    true,
                    true,
                    "reproduction_manifest_invalid_json",
                ));
            }
        };
    if validate_manifest(&manifest, receiver).is_err() {
        return Ok(
            OpeningPortfolioSourceArtifactReceiptReceiverArtifactInspection {
                custody_locator: locator,
                manifest_present: true,
                artifact_present: true,
                manifest: Some(manifest),
                server_computed_artifact_sha256: None,
                server_observed_artifact_byte_length: Some(artifact_metadata.len()),
                artifact_verified: false,
                status: "reproduction_manifest_binding_invalid".to_string(),
            },
        );
    }
    let bytes = tokio::fs::read(&artifact_path)
        .await
        .map_err(|error| error.to_string())?;
    let digest = sha256_bytes(&bytes);
    let verified = digest == manifest.receiver_artifact_sha256
        && artifact_metadata.len() == manifest.artifact_byte_length;
    Ok(
        OpeningPortfolioSourceArtifactReceiptReceiverArtifactInspection {
            custody_locator: locator,
            manifest_present: true,
            artifact_present: true,
            manifest: Some(manifest),
            server_computed_artifact_sha256: Some(digest),
            server_observed_artifact_byte_length: Some(artifact_metadata.len()),
            artifact_verified: verified,
            status: if verified {
                "server_rehashed_receiver_artifact_verified_not_executed"
            } else {
                "artifact_digest_or_length_mismatch"
            }
            .to_string(),
        },
    )
}

fn validate_manifest(
    manifest: &OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest,
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> Result<(), String> {
    let contract = &receiver.receiver_contract;
    let valid = manifest.schema_version == MANIFEST_SCHEMA_VERSION
        && valid_sha256(&manifest.manifest_sha256)
        && manifest.manifest_sha256 == fingerprint_without(manifest, &["manifest_sha256"])?
        && manifest.isolated_receiver_id == receiver.isolated_receiver_id
        && manifest.isolated_receiver_spec_sha256 == receiver.isolated_receiver_spec_sha256
        && manifest.receiver_contract_sha256 == contract.contract_sha256
        && manifest.receiver_spec_revision == contract.receiver_spec_revision
        && manifest.receiver_code_revision == contract.proposed_receiver_code_revision
        && manifest.receiver_artifact_sha256 == contract.proposed_receiver_artifact_sha256
        && manifest.artifact_byte_length > 0
        && manifest.artifact_byte_length <= MAX_ARTIFACT_BYTES
        && manifest.artifact_file_name == ARTIFACT_FILE_NAME
        && !manifest.artifact_media_type.trim().is_empty()
        && valid_sha256(&manifest.source_bundle_sha256)
        && manifest.artifact_reproduction_procedure_sha256
            == sha256_bytes(receiver.artifact_reproduction_procedure.as_bytes())
        && manifest.runtime_identity == contract.runtime_identity
        && manifest.runtime_version == contract.runtime_version
        && !manifest.reproduced_by.trim().is_empty()
        && manifest.source_and_artifact_reproduced_from_immutable_revision
        && manifest.artifact_is_read_only_regular_file
        && manifest.artifact_was_not_executed
        && manifest.source_artifact_was_not_received_or_read;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 130 工件复现 manifest 无效或与 Stage 129 漂移".to_string())
}

async fn latest_review(
    state: &AppState,
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> Result<Option<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview>, String>
{
    let reviews = read_reviews(state, receiver).await?;
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
        return Err("Stage 130 授权复核链 tip 数量无效".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("Stage 130 授权复核链存在环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(hash)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "Stage 130 授权复核链断裂".to_string())?;
                if previous.review_sha256 != *hash {
                    return Err("Stage 130 授权复核链前序摘要不一致".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("Stage 130 授权复核链前序链接无效".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("Stage 130 授权复核链未完全连通".to_string());
    }
    if reviews.iter().any(|review| {
        review.one_future_claim_first_source_artifact_receipt_attempt_authorized
            && reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(review.review_id.as_str()))
    }) {
        return Err("Stage 130 批准必须终止复核链".to_string());
    }
    for review in &reviews {
        let previous = review
            .previous_review_id
            .as_deref()
            .and_then(|id| by_id.get(id).copied());
        if review.excluded_prior_actor_ids
            != expected_excluded_prior_actor_ids(receiver, &review.artifact_manifest, previous)
        {
            return Err("Stage 130 授权复核责任链不一致".to_string());
        }
    }
    Ok(Some(tips[0].clone()))
}

async fn read_reviews(
    state: &AppState,
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview>, String> {
    let mut directory = match tokio::fs::read_dir(review_directory(state, receiver)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let review: OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview =
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_review(&review, receiver)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
            || !hashes.insert(review.review_sha256.clone())
        {
            return Err("Stage 130 授权复核重复、分叉或文件名错误".to_string());
        }
        reviews.push(review);
    }
    Ok(reviews)
}

fn expected_excluded_prior_actor_ids(
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
    manifest: &OpeningPortfolioSourceArtifactReceiptReproducedReceiverManifest,
    latest: Option<&OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview>,
) -> Vec<String> {
    let mut values = receiver.excluded_prior_actor_ids.clone();
    values.push(receiver.registered_by.clone());
    values.push(receiver.implementation_review.reviewer_id.clone());
    values.push(manifest.reproduced_by.clone());
    if let Some(latest) = latest {
        values.extend(latest.excluded_prior_actor_ids.clone());
        values.push(latest.reviewer_id.clone());
    }
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn artifact_inspection_matches_review(
    inspection: &OpeningPortfolioSourceArtifactReceiptReceiverArtifactInspection,
    review: &OpeningPortfolioSourceArtifactReceiptFirstExecutionAuthorizationReview,
) -> bool {
    inspection.artifact_verified
        && inspection
            .manifest
            .as_ref()
            .is_some_and(|value| value == &review.artifact_manifest)
        && inspection.server_computed_artifact_sha256.as_deref()
            == Some(review.server_computed_artifact_sha256.as_str())
        && inspection.server_observed_artifact_byte_length
            == Some(review.server_observed_artifact_byte_length)
}

fn pending_inspection(
    custody_locator: String,
    manifest_present: bool,
    artifact_present: bool,
    status: &str,
) -> OpeningPortfolioSourceArtifactReceiptReceiverArtifactInspection {
    OpeningPortfolioSourceArtifactReceiptReceiverArtifactInspection {
        custody_locator,
        manifest_present,
        artifact_present,
        manifest: None,
        server_computed_artifact_sha256: None,
        server_observed_artifact_byte_length: None,
        artifact_verified: false,
        status: status.to_string(),
    }
}

async fn metadata_if_present(path: &Path) -> Result<Option<std::fs::Metadata>, String> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn safe_read_only_regular_file(metadata: &std::fs::Metadata, maximum_bytes: u64) -> bool {
    !metadata.file_type().is_symlink()
        && metadata.is_file()
        && metadata.permissions().readonly()
        && metadata.len() > 0
        && metadata.len() <= maximum_bytes
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn artifact_directory(
    state: &AppState,
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> PathBuf {
    decision_root(state)
        .join("opening-portfolio-source-artifact-receipt-reproduced-receivers")
        .join(&receiver.isolated_receiver_id)
        .join(&receiver.receiver_contract.proposed_receiver_artifact_sha256)
}
fn review_directory(
    state: &AppState,
    receiver: &OpeningPortfolioSourceArtifactReceiptIsolatedReceiverRecord,
) -> PathBuf {
    decision_root(state)
        .join("opening-portfolio-source-artifact-receipt-first-execution-authorization-reviews")
        .join(&receiver.isolated_receiver_id)
}

struct ReviewLock(PathBuf);
impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, isolated_receiver_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "opening-portfolio-source-artifact-receipt-first-execution-{isolated_receiver_id}.lock"
    ));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|modified| SystemTime::now().duration_since(modified).ok())
            .is_some_and(|age| age > StdDuration::from_secs(600));
        if stale {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 130 授权复核正在进行".to_string())?;
    Ok(ReviewLock(path))
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    use tokio::io::AsyncWriteExt;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"))
    } else {
        Ok(value.to_string())
    }
}
fn sha256_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 130 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(sha256_bytes(
        &serde_json::to_vec(&value).map_err(|error| error.to_string())?,
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_is_single_use_and_strictly_twenty_four_hours() {
        let submitted = Utc::now();
        assert_eq!(
            submitted + TimeDelta::hours(24),
            submitted + TimeDelta::hours(AUTHORIZATION_VALID_HOURS)
        );
        assert_eq!(ONE_SHOT_ATTEMPT_LIMIT, 1);
    }

    #[test]
    fn receiver_artifact_limits_are_separate_from_future_source_input_limits() {
        assert_eq!(ARTIFACT_FILE_NAME, "receiver.artifact");
        assert_eq!(MAX_ARTIFACT_BYTES, 16 * 1024 * 1024);
        assert_eq!(MAX_MANIFEST_BYTES, 64 * 1024);
    }

    #[test]
    fn stage_130_has_no_upload_execution_or_trading_entrypoint() {
        assert_eq!(
            NEXT_GATE,
            "stage_131_claim_first_source_artifact_receipt_attempt"
        );
        assert!([false; 18].into_iter().all(|value| !value));
    }

    #[test]
    fn manifest_is_self_hashed_and_content_addressed() {
        assert_eq!(sha256_bytes(b"receiver"), sha256_bytes(b"receiver"));
        assert_ne!(sha256_bytes(b"receiver"), sha256_bytes(b"source"));
    }

    #[test]
    fn artifact_files_must_be_nonempty_read_only_regular_files() {
        let directory = std::env::temp_dir().join(format!(
            "hone-stage-130-metadata-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        std::fs::create_dir_all(&directory).expect("create temp directory");
        let file = directory.join("receiver.artifact");
        std::fs::write(&file, b"receiver").expect("write artifact");
        let writable = std::fs::symlink_metadata(&file).expect("writable metadata");
        assert!(!safe_read_only_regular_file(&writable, 64));
        let mut permissions = writable.permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&file, permissions).expect("set read only");
        let read_only = std::fs::symlink_metadata(&file).expect("read-only metadata");
        assert!(safe_read_only_regular_file(&read_only, 64));
        assert!(!safe_read_only_regular_file(&read_only, 2));
        let _ = std::fs::remove_file(file);
        let _ = std::fs::remove_dir(directory);
    }
}
