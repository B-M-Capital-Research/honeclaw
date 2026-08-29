//! Stage 132 one-shot receipt of original provider artifacts for one exact Stage 131 claim.
//!
//! The multipart `request` field is validated before a durable start marker is written. Only
//! after that marker exists may source bytes be read. Original bytes are hashed in memory,
//! structurally screened without financial parsing, AES-256-GCM encrypted, written to a private
//! quarantine, and atomically moved to server-generated content-addressed paths. The output is a
//! create-once *untrusted* receipt; it cannot create an opening snapshot or any financial state.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use axum::Json;
use axum::extract::multipart::Field;
use axum::extract::{Multipart, Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::warn;

use super::controlled_shadow_opening_portfolio_source_artifact_receipt_execution_attempt_claims::{
    OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt,
    validate_claim,
};
use super::controlled_shadow_opening_portfolio_source_artifact_receipt_first_execution_authorizations::read_revalidated_opening_portfolio_source_artifact_receipt_receiver_artifact_for_execution;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-execution-attempt-registry-v1";
const START_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-execution-attempt-start-v1";
const RECEIPT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-manifest-v1-create-once-untrusted";
const RESULT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-execution-attempt-result-v1";
const POLICY_VERSION: &str = "hone-opening-portfolio-source-artifact-receipt-execution-v1-one-shot-encrypted-create-once-fail-closed";
const NEXT_GATE: &str = "stage_133_independent_source_artifact_receipt_validation";
const ENCRYPTION_KEY_ENV: &str = "HONE_OPENING_PORTFOLIO_RECEIPT_ENCRYPTION_KEY";
const MAX_REQUEST_BYTES: usize = 256 * 1024;
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_IDENTIFIER_CHARS: usize = 240;
const MAX_ARTIFACT_BYTES: usize = 64 * 1024 * 1024;
const MAX_RECEIPT_BYTES: usize = 256 * 1024 * 1024;
const MAX_ARTIFACT_COUNT: usize = 64;
const INTERRUPTED_RECOVERY_GRACE_SECONDS: i64 = 301;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeclaredSourceArtifact {
    declared_format: String,
    source_account_aliases: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest {
    expected_claim_sha256: String,
    expected_authorization_review_sha256: String,
    expected_isolated_receiver_spec_sha256: String,
    expected_receiver_contract_sha256: String,
    expected_receiver_artifact_sha256: String,
    expected_artifact_manifest_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_stage_125_specification_sha256: String,
    provider_statement_or_export_identifier: String,
    provider_generated_at_or_statement_as_of: DateTime<Utc>,
    artifacts: Vec<DeclaredSourceArtifact>,
    execution_reason: String,
    exact_current_stage_51_through_stage_131_binding_confirmed: bool,
    executor_independent_from_complete_prior_chain_and_stage_131_claimant_confirmed: bool,
    start_marker_consumes_claim_before_first_source_byte_confirmed: bool,
    administrator_authenticated_stream_only_no_remote_fetch_confirmed: bool,
    original_artifacts_already_account_pseudonymized_and_credentials_removed_confirmed: bool,
    format_magic_safe_structure_archive_active_content_password_symlink_and_path_rejection_confirmed:
        bool,
    streaming_sha256_length_private_quarantine_and_atomic_content_addressed_commit_confirmed: bool,
    encryption_at_rest_and_redacted_manifest_confirmed: bool,
    duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed: bool,
    receipt_create_once_untrusted_and_stage_133_independent_validation_required_confirmed: bool,
    no_financial_row_parsing_snapshot_materialization_or_snapshot_admission_confirmed: bool,
    no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct OpeningPortfolioSourceArtifactReceiptExecutionAttemptStart {
    schema_version: String,
    policy_version: String,
    start_sha256: String,
    stage_131_attempt_id: String,
    stage_131_claim_sha256: String,
    started_at: DateTime<Utc>,
    executed_by: String,
    execution_reason: String,
    declared_artifact_count: usize,
    one_shot_claim_consumed_before_first_source_byte: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UntrustedOpeningPortfolioSourceArtifactReceiptItem {
    pub artifact_id: String,
    pub artifact_sha256: String,
    pub byte_length: u64,
    pub declared_format: String,
    pub validated_format: String,
    pub source_account_aliases: Vec<String>,
    pub storage_object_sha256: String,
    pub storage_byte_length: u64,
    pub storage_relative_path: String,
    pub original_bytes_encrypted_immutable: bool,
    pub format_magic_and_safe_structure_validated_without_financial_parsing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UntrustedOpeningPortfolioSourceArtifactReceiptManifest {
    pub schema_version: String,
    pub policy_version: String,
    pub receipt_id: String,
    pub receipt_manifest_sha256: String,
    pub stage_131_attempt_id: String,
    pub stage_131_claim_sha256: String,
    pub stage_130_authorization_review_sha256: String,
    pub stage_129_isolated_receiver_spec_sha256: String,
    pub stage_127_implementation_contract_sha256: String,
    pub stage_125_specification_sha256: String,
    pub source_provider_name: String,
    pub portfolio_scope_alias: String,
    pub reporting_currency: String,
    pub source_timezone: String,
    pub snapshot_as_of_utc: DateTime<Utc>,
    pub provider_artifact_identifier_sha256: String,
    pub provider_generated_at_or_statement_as_of: DateTime<Utc>,
    pub hone_received_at: DateTime<Utc>,
    pub encryption_algorithm: String,
    pub encryption_key_fingerprint: String,
    pub artifacts: Vec<UntrustedOpeningPortfolioSourceArtifactReceiptItem>,
    pub total_original_byte_length: u64,
    pub create_once: bool,
    pub untrusted: bool,
    pub independent_receipt_validation_completed: bool,
    pub financial_rows_parsed: bool,
    pub opening_portfolio_snapshot_materialized: bool,
    pub opening_portfolio_snapshot_admitted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioSourceArtifactReceiptExecutionStatus {
    CompletedWithUntrustedReceipt,
    FailedClaimConsumed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult {
    pub schema_version: String,
    pub policy_version: String,
    pub result_id: String,
    pub result_sha256: String,
    pub stage_131_attempt_id: String,
    pub stage_131_claim_sha256: String,
    pub completed_at: DateTime<Utc>,
    pub executed_by: String,
    pub execution_reason: String,
    pub duration_millis: u64,
    pub status: OpeningPortfolioSourceArtifactReceiptExecutionStatus,
    pub bounded_error_code: Option<String>,
    pub receipt_id: Option<String>,
    pub receipt_manifest_sha256: Option<String>,
    pub receipt_manifest_relative_path: Option<String>,
    pub artifact_count: usize,
    pub total_original_byte_length: u64,
    pub claim_consumed: bool,
    pub start_marker_persisted_before_first_source_byte: bool,
    pub receiver_artifact_revalidated: bool,
    pub receiver_artifact_spawned_or_executed: bool,
    pub source_artifact_received_or_read: bool,
    pub source_artifact_may_have_been_read: bool,
    pub original_bytes_encrypted_at_rest: bool,
    pub receipt_manifest_created: bool,
    pub receipt_untrusted: bool,
    pub independent_receipt_validation_completed: bool,
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
pub(crate) struct OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub receipt_endpoint_available: bool,
    pub encryption_key_configured: bool,
    pub pending_claims: Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim>,
    pub results: Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult>,
    pub pending_claim_count: usize,
    pub terminal_result_count: usize,
    pub successful_untrusted_receipt_count: usize,
    pub failed_consumed_claim_count: usize,
    pub next_gate: String,
    pub receipt_manifest_created: bool,
    pub independent_receipt_validation_completed: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_created: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptExecutionReadinessSummary {
    pub pending_claim_count: usize,
    pub terminal_result_count: usize,
    pub successful_untrusted_receipt_count: usize,
    pub failed_consumed_claim_count: usize,
    pub encryption_key_configured: bool,
}

struct StagedArtifact {
    item: UntrustedOpeningPortfolioSourceArtifactReceiptItem,
    quarantine_path: PathBuf,
    final_path: PathBuf,
}

pub(crate) async fn handle_get_opening_portfolio_source_artifact_receipt_execution_attempts(
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
            warn!(%error, "Stage 132 source-artifact receipt registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "来源工件单次接收表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_receive_opening_portfolio_source_artifact_receipt_attempt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    mut multipart: Multipart,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    let first = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => {
            return crate::routes::json_error(
                StatusCode::BAD_REQUEST,
                "multipart 必须先提供 request 元数据字段",
            );
        }
        Err(_) => {
            return crate::routes::json_error(
                StatusCode::BAD_REQUEST,
                "multipart request 字段读取失败",
            );
        }
    };
    if first.name() != Some("request") || first.file_name().is_some() {
        return crate::routes::json_error(
            StatusCode::BAD_REQUEST,
            "第一个 multipart 字段必须是非文件 request 元数据；尚未消费 Stage 132 尝试",
        );
    }
    let request_bytes = match read_field_bounded(first, MAX_REQUEST_BYTES).await {
        Ok(value) => value,
        Err(error) => return crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    };
    let request: ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest =
        match serde_json::from_slice(&request_bytes) {
            Ok(value) => value,
            Err(_) => {
                return crate::routes::json_error(
                    StatusCode::BAD_REQUEST,
                    "request 元数据 JSON 无效或含未知字段；尚未消费 Stage 132 尝试",
                );
            }
        };
    match receive_once(&state, &admin.user_id, &attempt_id, request, multipart).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(_) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "接收结果已落盘，但刷新单次接收表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn opening_portfolio_source_artifact_receipt_execution_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptExecutionReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        OpeningPortfolioSourceArtifactReceiptExecutionReadinessSummary {
            pending_claim_count: value.pending_claim_count,
            terminal_result_count: value.terminal_result_count,
            successful_untrusted_receipt_count: value.successful_untrusted_receipt_count,
            failed_consumed_claim_count: value.failed_consumed_claim_count,
            encryption_key_configured: value.encryption_key_configured,
        },
    )
}

pub(crate) async fn started_or_terminal_opening_portfolio_source_artifact_receipt_claim_attempt_ids(
    state: &AppState,
) -> Result<BTreeSet<String>, String> {
    let mut ids = read_starts(state)
        .await?
        .into_iter()
        .map(|value| value.stage_131_attempt_id)
        .collect::<BTreeSet<_>>();
    ids.extend(
        read_results(state)
            .await?
            .into_iter()
            .map(|value| value.stage_131_attempt_id),
    );
    Ok(ids)
}

pub(crate) async fn opening_portfolio_source_artifact_receipt_results_for_independent_validation(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult>, String> {
    read_results(state).await
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry, String> {
    let claims =
        opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt(state)
            .await?;
    let mut results = read_results(state).await?;
    recover_interrupted_attempts(state, &claims, &results).await?;
    results = read_results(state).await?;
    let starts = read_starts(state).await?;
    let terminal = results
        .iter()
        .map(|value| value.stage_131_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    let started = starts
        .iter()
        .map(|value| value.stage_131_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    let pending_claims = claims
        .into_iter()
        .filter(|claim| {
            !started.contains(claim.attempt_id.as_str())
                && !terminal.contains(claim.attempt_id.as_str())
        })
        .collect::<Vec<_>>();
    let successful = results.iter().filter(|value| value.status == OpeningPortfolioSourceArtifactReceiptExecutionStatus::CompletedWithUntrustedReceipt).count();
    let failed = results.len().saturating_sub(successful);
    Ok(OpeningPortfolioSourceArtifactReceiptExecutionAttemptRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        receipt_endpoint_available: true,
        encryption_key_configured: encryption_key().is_ok(),
        pending_claim_count: pending_claims.len(),
        terminal_result_count: results.len(),
        successful_untrusted_receipt_count: successful,
        failed_consumed_claim_count: failed,
        pending_claims,
        results,
        next_gate: NEXT_GATE.to_string(),
        receipt_manifest_created: successful > 0,
        independent_receipt_validation_completed: false,
        opening_portfolio_snapshot_admitted: false,
        ledger_created: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 132 仅对尚未开始的 Stage 131 claim 接收一次管理员上传的 PDF/CSV/JSON。开始标记先于任何来源字节；原字节只以 AES-256-GCM 密文、服务端内容地址和未受信 receipt 保存。失败或中断永久消费 claim。Stage 133 独立验证之前，不得解析金融行、物化或接纳期初持仓，也不得创建账本、持仓、现金、净值、训练、订单、券商或交易状态。".to_string(),
    })
}

async fn receive_once(
    state: &AppState,
    executor_id: &str,
    attempt_id: &str,
    request: ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest,
    multipart: Multipart,
) -> Result<(), String> {
    let started_clock = Instant::now();
    let key = encryption_key()?;
    let claim =
        opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt(state)
            .await?
            .into_iter()
            .find(|value| value.attempt_id == attempt_id)
            .ok_or_else(|| "当前没有这条 Stage 131 claim".to_string())?;
    validate_claim(&claim)?;
    validate_pre_start_request(&claim, executor_id, &request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if result_path(state, attempt_id).exists() || start_path(state, attempt_id).exists() {
        return Err("这条 Stage 131 claim 已开始或已有终态，不能重试".to_string());
    }
    let started_at = Utc::now();
    let mut start = OpeningPortfolioSourceArtifactReceiptExecutionAttemptStart {
        schema_version: START_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        start_sha256: String::new(),
        stage_131_attempt_id: claim.attempt_id.clone(),
        stage_131_claim_sha256: claim.claim_sha256.clone(),
        started_at,
        executed_by: executor_id.to_string(),
        execution_reason: bounded_required(&request.execution_reason, "执行原因")?,
        declared_artifact_count: request.artifacts.len(),
        one_shot_claim_consumed_before_first_source_byte: true,
    };
    start.start_sha256 = fingerprint_without(&start, &["start_sha256"])?;
    validate_start(&start, &claim)?;
    write_immutable_json(&start_path(state, attempt_id), &start).await?;

    let outcome = receive_after_start(state, &claim, &request, multipart, &key, started_at).await;
    let result = match outcome {
        Ok(receipt) => success_result(
            &claim,
            &start,
            &receipt,
            started_clock.elapsed().as_millis() as u64,
        )?,
        Err(error) => {
            cleanup_attempt_quarantine(state, attempt_id).await;
            failure_result(
                &claim,
                &start,
                bounded_error_code(&error),
                started_clock.elapsed().as_millis() as u64,
            )?
        }
    };
    write_immutable_json(&result_path(state, attempt_id), &result).await?;
    if result.status == OpeningPortfolioSourceArtifactReceiptExecutionStatus::FailedClaimConsumed {
        return Err(format!(
            "本次 Stage 132 单次接收已终止且不可重试：{}",
            result
                .bounded_error_code
                .as_deref()
                .unwrap_or("receipt_failed")
        ));
    }
    Ok(())
}

async fn receive_after_start(
    state: &AppState,
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    request: &ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest,
    mut multipart: Multipart,
    key: &[u8; 32],
    received_at: DateTime<Utc>,
) -> Result<UntrustedOpeningPortfolioSourceArtifactReceiptManifest, String> {
    let receiver_bytes =
        read_revalidated_opening_portfolio_source_artifact_receipt_receiver_artifact_for_execution(
            state,
            &claim.authorization,
        )
        .await?;
    if receiver_bytes.is_empty() {
        return Err("receiver_artifact_empty".to_string());
    }
    let quarantine = quarantine_directory(state).join(&claim.attempt_id);
    tokio::fs::create_dir_all(&quarantine)
        .await
        .map_err(|_| "quarantine_create_failed".to_string())?;
    set_directory_private(&quarantine)?;
    let contract = &claim
        .authorization
        .receiver
        .receiver_contract
        .exact_approved_implementation_contract;
    let mut staged = Vec::new();
    let mut total = 0usize;
    let mut index = 0usize;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| "artifact_stream_failed".to_string())?
    {
        if field.name() != Some("artifact") || field.file_name().is_none() {
            return Err("unexpected_multipart_field_after_start".to_string());
        }
        let declaration = request
            .artifacts
            .get(index)
            .ok_or_else(|| "more_artifacts_than_declared".to_string())?;
        let declared_content_type = field.content_type().map(str::to_string);
        let mut bytes = read_field_bounded(field, MAX_ARTIFACT_BYTES).await?;
        if bytes.is_empty() {
            return Err("empty_source_artifact".to_string());
        }
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| "receipt_size_overflow".to_string())?;
        if total > MAX_RECEIPT_BYTES {
            return Err("receipt_exceeds_256_mib".to_string());
        }
        let validated_format = validate_safe_format(
            &bytes,
            &declaration.declared_format,
            declared_content_type.as_deref(),
        )?;
        let artifact_sha256 = sha256_bytes(&bytes);
        let encrypted = encrypt_artifact(key, &artifact_sha256, &bytes)?;
        let artifact_byte_length = bytes.len() as u64;
        bytes.fill(0);
        let storage_sha256 = sha256_bytes(&encrypted);
        let final_relative = format!(
            "opening-portfolio-source-artifacts/{}/{}/original.bin.enc",
            contract.stage_125_registration_id, artifact_sha256
        );
        let final_path = decision_root(state).join(&final_relative);
        let quarantine_path = quarantine.join(format!("artifact-{index:02}.enc"));
        write_private_create_new(&quarantine_path, &encrypted).await?;
        staged.push(StagedArtifact {
            item: UntrustedOpeningPortfolioSourceArtifactReceiptItem {
                artifact_id: artifact_sha256[..32].to_string(),
                artifact_sha256,
                byte_length: artifact_byte_length,
                declared_format: declaration.declared_format.clone(),
                validated_format,
                source_account_aliases: validated_aliases(&declaration.source_account_aliases)?,
                storage_object_sha256: storage_sha256,
                storage_byte_length: encrypted.len() as u64,
                storage_relative_path: final_relative,
                original_bytes_encrypted_immutable: true,
                format_magic_and_safe_structure_validated_without_financial_parsing: true,
            },
            quarantine_path,
            final_path,
        });
        index += 1;
    }
    if index != request.artifacts.len() {
        return Err("artifact_count_does_not_match_declaration".to_string());
    }
    let mut created = Vec::new();
    for artifact in &staged {
        if let Some(parent) = artifact.final_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|_| "content_address_parent_create_failed".to_string())?;
        }
        if artifact.final_path.exists() {
            let existing = tokio::fs::read(&artifact.final_path)
                .await
                .map_err(|_| "existing_content_address_read_failed".to_string())?;
            if sha256_bytes(&existing) != artifact.item.storage_object_sha256 {
                cleanup_created(&created).await;
                return Err("content_address_collision_or_overwrite_attempt".to_string());
            }
            let _ = tokio::fs::remove_file(&artifact.quarantine_path).await;
        } else if let Err(_) =
            tokio::fs::rename(&artifact.quarantine_path, &artifact.final_path).await
        {
            cleanup_created(&created).await;
            return Err("atomic_content_address_commit_failed".to_string());
        } else {
            created.push(artifact.final_path.clone());
            if set_file_read_only(&artifact.final_path).is_err() {
                cleanup_created(&created).await;
                return Err("content_address_permission_lock_failed".to_string());
            }
        }
    }
    let _ = tokio::fs::remove_dir_all(&quarantine).await;
    let source = &contract.exact_stage_125_specification.source_contract;
    let mut receipt = UntrustedOpeningPortfolioSourceArtifactReceiptManifest {
        schema_version: RECEIPT_SCHEMA_VERSION.to_string(), policy_version: POLICY_VERSION.to_string(),
        receipt_id: String::new(), receipt_manifest_sha256: String::new(),
        stage_131_attempt_id: claim.attempt_id.clone(), stage_131_claim_sha256: claim.claim_sha256.clone(),
        stage_130_authorization_review_sha256: claim.authorization.review_sha256.clone(),
        stage_129_isolated_receiver_spec_sha256: claim.authorization.receiver.isolated_receiver_spec_sha256.clone(),
        stage_127_implementation_contract_sha256: contract.contract_sha256.clone(),
        stage_125_specification_sha256: contract.stage_125_specification_sha256.clone(),
        source_provider_name: source.source_provider_name.clone(), portfolio_scope_alias: source.portfolio_scope_alias.clone(),
        reporting_currency: source.reporting_currency.clone(), source_timezone: source.source_timezone.clone(),
        snapshot_as_of_utc: source.snapshot_as_of_utc,
        provider_artifact_identifier_sha256: sha256_bytes(request.provider_statement_or_export_identifier.trim().as_bytes()),
        provider_generated_at_or_statement_as_of: request.provider_generated_at_or_statement_as_of,
        hone_received_at: received_at,
        encryption_algorithm: "AES-256-GCM; deterministic nonce scoped to identical plaintext digest for idempotent immutable custody".to_string(),
        encryption_key_fingerprint: sha256_bytes(key)[..16].to_string(),
        artifacts: staged.into_iter().map(|value| value.item).collect(), total_original_byte_length: total as u64,
        create_once: true, untrusted: true, independent_receipt_validation_completed: false,
        financial_rows_parsed: false, opening_portfolio_snapshot_materialized: false, opening_portfolio_snapshot_admitted: false,
    };
    receipt.receipt_manifest_sha256 =
        fingerprint_without(&receipt, &["receipt_id", "receipt_manifest_sha256"])?;
    receipt.receipt_id = receipt.receipt_manifest_sha256[..32].to_string();
    validate_receipt(&receipt, claim)?;
    let path = receipt_manifest_path(state, &claim.attempt_id, &receipt.receipt_id);
    if let Err(error) = write_immutable_json(&path, &receipt).await {
        cleanup_created(&created).await;
        return Err(format!("receipt_manifest_commit_failed:{error}"));
    }
    Ok(receipt)
}

fn validate_pre_start_request(
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    executor_id: &str,
    request: &ReceiveOpeningPortfolioSourceArtifactReceiptAttemptRequest,
) -> Result<(), String> {
    let review = &claim.authorization;
    let contract = &review
        .receiver
        .receiver_contract
        .exact_approved_implementation_contract;
    let mut excluded = claim.excluded_prior_actor_ids.clone();
    excluded.push(claim.claimed_by.clone());
    excluded.sort();
    excluded.dedup();
    let independent =
        !executor_id.trim().is_empty() && !excluded.iter().any(|value| value == executor_id);
    if request.expected_claim_sha256 != claim.claim_sha256
        || request.expected_authorization_review_sha256 != review.review_sha256
        || request.expected_isolated_receiver_spec_sha256
            != review.receiver.isolated_receiver_spec_sha256
        || request.expected_receiver_contract_sha256
            != review.receiver.receiver_contract.contract_sha256
        || request.expected_receiver_artifact_sha256 != review.server_computed_artifact_sha256
        || request.expected_artifact_manifest_sha256 != review.artifact_manifest.manifest_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_stage_125_specification_sha256
            != contract.stage_125_specification_sha256
    {
        return Err("Stage 125–131 绑定已漂移；尚未消费 Stage 132 尝试".to_string());
    }
    if independent
        != request.executor_independent_from_complete_prior_chain_and_stage_131_claimant_confirmed
    {
        return Err("Stage 132 执行人独立性确认不一致；尚未消费尝试".to_string());
    }
    if request.artifacts.is_empty() || request.artifacts.len() > MAX_ARTIFACT_COUNT {
        return Err("来源工件数量必须为 1–64；尚未消费尝试".to_string());
    }
    bounded_identifier(&request.provider_statement_or_export_identifier)?;
    bounded_required(&request.execution_reason, "执行原因")?;
    if request.provider_generated_at_or_statement_as_of > Utc::now() + Duration::minutes(5) {
        return Err("提供方生成时间不能位于未来；尚未消费尝试".to_string());
    }
    for item in &request.artifacts {
        if !contract
            .exact_source_artifact_contract
            .accepted_artifact_formats
            .contains(&item.declared_format)
        {
            return Err("声明格式不在 Stage 125 接受列表；尚未消费尝试".to_string());
        }
        validated_aliases(&item.source_account_aliases)?;
    }
    let confirmations = request.exact_current_stage_51_through_stage_131_binding_confirmed
        && request.executor_independent_from_complete_prior_chain_and_stage_131_claimant_confirmed
        && request.start_marker_consumes_claim_before_first_source_byte_confirmed
        && request.administrator_authenticated_stream_only_no_remote_fetch_confirmed
        && request.original_artifacts_already_account_pseudonymized_and_credentials_removed_confirmed
        && request.format_magic_safe_structure_archive_active_content_password_symlink_and_path_rejection_confirmed
        && request.streaming_sha256_length_private_quarantine_and_atomic_content_addressed_commit_confirmed
        && request.encryption_at_rest_and_redacted_manifest_confirmed
        && request.duplicate_content_idempotent_no_overwrite_and_correction_requires_new_artifact_confirmed
        && request.receipt_create_once_untrusted_and_stage_133_independent_validation_required_confirmed
        && request.no_financial_row_parsing_snapshot_materialization_or_snapshot_admission_confirmed
        && request.no_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && request.one_shot_failure_or_interruption_consumes_claim_and_no_retry_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    if !confirmations {
        return Err("Stage 132 必要安全确认不完整；尚未消费尝试".to_string());
    }
    Ok(())
}

fn validate_safe_format(
    bytes: &[u8],
    declared: &str,
    content_type: Option<&str>,
) -> Result<String, String> {
    let lowered = bytes
        .iter()
        .map(|byte| byte.to_ascii_lowercase())
        .collect::<Vec<_>>();
    match declared {
        "original_provider_pdf_statement" => {
            if !bytes.starts_with(b"%PDF-")
                || content_type.is_some_and(|value| value != "application/pdf")
            {
                return Err("pdf_magic_or_content_type_mismatch".to_string());
            }
            for marker in [
                b"/javascript".as_slice(),
                b"/js".as_slice(),
                b"/launch".as_slice(),
                b"/embeddedfile".as_slice(),
                b"/openaction".as_slice(),
                b"/encrypt".as_slice(),
            ] {
                if lowered.windows(marker.len()).any(|window| window == marker) {
                    return Err(
                        "pdf_active_embedded_or_password_protected_content_rejected".to_string()
                    );
                }
            }
            lopdf::Document::load_mem(bytes)
                .map_err(|_| "pdf_structure_parse_failed".to_string())?;
            Ok("pdf_magic_safe_structure_screened".to_string())
        }
        "original_provider_csv_export" => {
            if content_type.is_some_and(|value| {
                value != "text/csv"
                    && value != "application/csv"
                    && value != "application/vnd.ms-excel"
            }) {
                return Err("csv_content_type_mismatch".to_string());
            }
            let text = std::str::from_utf8(bytes).map_err(|_| "csv_not_utf8".to_string())?;
            if text.contains('\0') || text.lines().any(|line| line.len() > 1024 * 1024) {
                return Err("csv_unsafe_structure".to_string());
            }
            let header = text
                .lines()
                .next()
                .ok_or_else(|| "csv_header_missing".to_string())?;
            reject_sensitive_names(header)?;
            if text
                .lines()
                .skip(1)
                .flat_map(|line| line.split(','))
                .any(|cell| {
                    cell.trim_start_matches(&[' ', '\t', '"', '\''][..])
                        .starts_with(['=', '@'])
                })
            {
                return Err("csv_formula_active_content_rejected".to_string());
            }
            Ok("utf8_csv_safe_structure_screened".to_string())
        }
        "original_provider_json_export" => {
            if content_type.is_some_and(|value| value != "application/json") {
                return Err("json_content_type_mismatch".to_string());
            }
            let value: Value =
                serde_json::from_slice(bytes).map_err(|_| "json_invalid".to_string())?;
            if !value.is_object() && !value.is_array() {
                return Err("json_top_level_structure_rejected".to_string());
            }
            reject_sensitive_json_keys(&value)?;
            Ok("json_safe_structure_screened".to_string())
        }
        _ => Err("unsupported_declared_format".to_string()),
    }
}

fn reject_sensitive_names(value: &str) -> Result<(), String> {
    let normalized = value.to_ascii_lowercase().replace([' ', '-'], "_");
    for token in [
        "password",
        "passwd",
        "api_key",
        "secret",
        "credential",
        "access_token",
        "account_number",
        "broker_account",
        "raw_account",
    ] {
        if normalized.contains(token) {
            return Err("raw_account_or_secret_field_rejected".to_string());
        }
    }
    Ok(())
}
fn reject_sensitive_json_keys(value: &Value) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                reject_sensitive_names(key)?;
                reject_sensitive_json_keys(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                reject_sensitive_json_keys(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn encrypt_artifact(
    key: &[u8; 32],
    artifact_sha256: &str,
    plaintext: &[u8],
) -> Result<Vec<u8>, String> {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .map_err(|_| "encryption_key_invalid".to_string())?;
    mac.update(b"hone-opening-portfolio-receipt-aes-gcm-nonce-v1\0");
    mac.update(artifact_sha256.as_bytes());
    let nonce_bytes = mac.finalize().into_bytes();
    let nonce = Nonce::from_slice(&nonce_bytes[..12]);
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| "encryption_key_invalid".to_string())?;
    let mut aad = b"hone-opening-portfolio-receipt-artifact-v1\0".to_vec();
    aad.extend_from_slice(artifact_sha256.as_bytes());
    let ciphertext = cipher
        .encrypt(
            nonce,
            aes_gcm::aead::Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| "artifact_encryption_failed".to_string())?;
    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(nonce);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

fn success_result(
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    start: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptStart,
    receipt: &UntrustedOpeningPortfolioSourceArtifactReceiptManifest,
    duration_millis: u64,
) -> Result<OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult, String> {
    terminal_result(
        claim,
        start,
        duration_millis,
        OpeningPortfolioSourceArtifactReceiptExecutionStatus::CompletedWithUntrustedReceipt,
        None,
        Some(receipt),
    )
}
fn failure_result(
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    start: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptStart,
    error: String,
    duration_millis: u64,
) -> Result<OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult, String> {
    terminal_result(
        claim,
        start,
        duration_millis,
        OpeningPortfolioSourceArtifactReceiptExecutionStatus::FailedClaimConsumed,
        Some(error),
        None,
    )
}
fn terminal_result(
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    start: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptStart,
    duration_millis: u64,
    status: OpeningPortfolioSourceArtifactReceiptExecutionStatus,
    error: Option<String>,
    receipt: Option<&UntrustedOpeningPortfolioSourceArtifactReceiptManifest>,
) -> Result<OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult, String> {
    let successful = receipt.is_some();
    let mut result = OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult {
        schema_version: RESULT_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        result_id: String::new(),
        result_sha256: String::new(),
        stage_131_attempt_id: claim.attempt_id.clone(),
        stage_131_claim_sha256: claim.claim_sha256.clone(),
        completed_at: Utc::now(),
        executed_by: start.executed_by.clone(),
        execution_reason: start.execution_reason.clone(),
        duration_millis,
        status,
        bounded_error_code: error,
        receipt_id: receipt.map(|value| value.receipt_id.clone()),
        receipt_manifest_sha256: receipt.map(|value| value.receipt_manifest_sha256.clone()),
        receipt_manifest_relative_path: receipt.map(|value| {
            format!(
                "opening-portfolio-source-artifact-receipts/{}/{}/manifest.json",
                claim.attempt_id, value.receipt_id
            )
        }),
        artifact_count: receipt.map_or(0, |value| value.artifacts.len()),
        total_original_byte_length: receipt.map_or(0, |value| value.total_original_byte_length),
        claim_consumed: true,
        start_marker_persisted_before_first_source_byte: true,
        receiver_artifact_revalidated: successful,
        receiver_artifact_spawned_or_executed: false,
        source_artifact_received_or_read: successful,
        // Once the durable start marker exists, a failed stream may already have
        // yielded source bytes. Record that possibility conservatively instead of
        // presenting a failed, claim-consuming attempt as a proven no-read event.
        source_artifact_may_have_been_read: true,
        original_bytes_encrypted_at_rest: successful,
        receipt_manifest_created: successful,
        receipt_untrusted: successful,
        independent_receipt_validation_completed: false,
        opening_portfolio_snapshot_materialized: false,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        position_or_cash_written: false,
        nav_or_performance_written: false,
        model_or_metric_store_written: false,
        training_or_rl_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    result.result_sha256 = fingerprint_without(&result, &["result_id", "result_sha256"])?;
    result.result_id = result.result_sha256[..32].to_string();
    validate_result(&result, claim)?;
    Ok(result)
}

fn validate_start(
    start: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptStart,
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
) -> Result<(), String> {
    if start.schema_version != START_SCHEMA_VERSION
        || start.policy_version != POLICY_VERSION
        || !valid_sha256(&start.start_sha256)
        || start.start_sha256 != fingerprint_without(start, &["start_sha256"])?
        || start.stage_131_attempt_id != claim.attempt_id
        || start.stage_131_claim_sha256 != claim.claim_sha256
        || start.started_at < claim.claimed_at
        || start.executed_by.trim().is_empty()
        || start.execution_reason.trim().is_empty()
        || start.declared_artifact_count == 0
        || start.declared_artifact_count > MAX_ARTIFACT_COUNT
        || !start.one_shot_claim_consumed_before_first_source_byte
    {
        return Err("Stage 132 start marker 无效或漂移".to_string());
    }
    Ok(())
}
fn validate_receipt(
    receipt: &UntrustedOpeningPortfolioSourceArtifactReceiptManifest,
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
) -> Result<(), String> {
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION
        || receipt.policy_version != POLICY_VERSION
        || !valid_id(&receipt.receipt_id)
        || !valid_sha256(&receipt.receipt_manifest_sha256)
        || receipt.receipt_id != receipt.receipt_manifest_sha256[..32]
        || receipt.receipt_manifest_sha256
            != fingerprint_without(receipt, &["receipt_id", "receipt_manifest_sha256"])?
        || receipt.stage_131_attempt_id != claim.attempt_id
        || receipt.stage_131_claim_sha256 != claim.claim_sha256
        || receipt.artifacts.is_empty()
        || receipt.artifacts.len() > MAX_ARTIFACT_COUNT
        || receipt.total_original_byte_length == 0
        || receipt.total_original_byte_length > MAX_RECEIPT_BYTES as u64
        || !receipt.create_once
        || !receipt.untrusted
        || receipt.independent_receipt_validation_completed
        || receipt.financial_rows_parsed
        || receipt.opening_portfolio_snapshot_materialized
        || receipt.opening_portfolio_snapshot_admitted
    {
        return Err("Stage 132 untrusted receipt 无效、漂移或越权".to_string());
    }
    Ok(())
}
fn validate_result(
    result: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
) -> Result<(), String> {
    let success = result.status
        == OpeningPortfolioSourceArtifactReceiptExecutionStatus::CompletedWithUntrustedReceipt;
    let success_fields = result.bounded_error_code.is_none()
        && result.receipt_id.is_some()
        && result.receipt_manifest_sha256.is_some()
        && result.receipt_manifest_relative_path.is_some()
        && result.artifact_count > 0
        && result.total_original_byte_length > 0
        && result.receiver_artifact_revalidated
        && result.source_artifact_received_or_read
        && result.original_bytes_encrypted_at_rest
        && result.receipt_manifest_created
        && result.receipt_untrusted;
    let failure_fields = result.bounded_error_code.is_some()
        && result.receipt_id.is_none()
        && result.receipt_manifest_sha256.is_none()
        && result.receipt_manifest_relative_path.is_none()
        && result.artifact_count == 0
        && result.total_original_byte_length == 0
        && !result.receiver_artifact_revalidated
        && !result.source_artifact_received_or_read
        && !result.original_bytes_encrypted_at_rest
        && !result.receipt_manifest_created
        && !result.receipt_untrusted;
    let authority_closed = !result.receiver_artifact_spawned_or_executed
        && result.source_artifact_may_have_been_read
        && !result.independent_receipt_validation_completed
        && !result.opening_portfolio_snapshot_materialized
        && !result.opening_portfolio_snapshot_admitted
        && !result.financial_event_allowlist_nonempty
        && !result.ledger_created
        && !result.position_or_cash_written
        && !result.nav_or_performance_written
        && !result.model_or_metric_store_written
        && !result.training_or_rl_feedback_authorized
        && !result.reward_authorized
        && !result.order_generation_authorized
        && !result.broker_access_authorized
        && !result.trading_authorized;
    if result.schema_version != RESULT_SCHEMA_VERSION
        || result.policy_version != POLICY_VERSION
        || !valid_id(&result.result_id)
        || !valid_sha256(&result.result_sha256)
        || result.result_id != result.result_sha256[..32]
        || result.result_sha256 != fingerprint_without(result, &["result_id", "result_sha256"])?
        || result.stage_131_attempt_id != claim.attempt_id
        || result.stage_131_claim_sha256 != claim.claim_sha256
        || result.completed_at < claim.claimed_at
        || result.executed_by.trim().is_empty()
        || result.execution_reason.trim().is_empty()
        || !result.claim_consumed
        || !result.start_marker_persisted_before_first_source_byte
        || !authority_closed
        || (success && !success_fields)
        || (!success && !failure_fields)
    {
        return Err("Stage 132 terminal result 无效、漂移或越权".to_string());
    }
    Ok(())
}

async fn recover_interrupted_attempts(
    state: &AppState,
    claims: &[OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim],
    results: &[OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult],
) -> Result<(), String> {
    let claims = claims
        .iter()
        .map(|value| (value.attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let completed = results
        .iter()
        .map(|value| value.stage_131_attempt_id.as_str())
        .collect::<BTreeSet<_>>();
    for start in read_starts(state).await? {
        if completed.contains(start.stage_131_attempt_id.as_str()) {
            continue;
        }
        let claim = claims
            .get(start.stage_131_attempt_id.as_str())
            .ok_or_else(|| "Stage 132 start 缺少 Stage 131 claim".to_string())?;
        let deadline = start.started_at + Duration::seconds(INTERRUPTED_RECOVERY_GRACE_SECONDS);
        if Utc::now() <= deadline {
            continue;
        }
        cleanup_attempt_quarantine(state, &claim.attempt_id).await;
        let receipt_root = receipt_attempt_directory(state, &claim.attempt_id);
        let _ = tokio::fs::remove_dir_all(receipt_root).await;
        let result = failure_result(
            claim,
            &start,
            "receipt_interrupted_without_terminal_result".to_string(),
            (INTERRUPTED_RECOVERY_GRACE_SECONDS as u64) * 1_000,
        )?;
        write_immutable_json(&result_path(state, &claim.attempt_id), &result).await?;
    }
    Ok(())
}

async fn read_starts(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptStart>, String> {
    let mut values: Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptStart> =
        read_json_directory(start_directory(state), 512 * 1024).await?;
    let claims =
        opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt(state)
            .await?
            .into_iter()
            .map(|value| (value.attempt_id.clone(), value))
            .collect::<BTreeMap<_, _>>();
    let mut attempts = BTreeSet::new();
    for value in &values {
        let claim = claims
            .get(&value.stage_131_attempt_id)
            .ok_or_else(|| "Stage 132 start 缺少 Stage 131 claim".to_string())?;
        validate_start(value, claim)?;
        if !attempts.insert(value.stage_131_attempt_id.clone()) {
            return Err("Stage 132 start 重复或重放".to_string());
        }
    }
    values.sort_by(|left, right| left.started_at.cmp(&right.started_at));
    Ok(values)
}
async fn read_results(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult>, String> {
    let mut values: Vec<OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult> =
        read_json_directory(result_directory(state), 1024 * 1024).await?;
    let claims =
        opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt(state)
            .await?
            .into_iter()
            .map(|value| (value.attempt_id.clone(), value))
            .collect::<BTreeMap<_, _>>();
    let mut ids = BTreeSet::new();
    let mut attempts = BTreeSet::new();
    for value in &values {
        let claim = claims
            .get(&value.stage_131_attempt_id)
            .ok_or_else(|| "Stage 132 result 缺少 Stage 131 claim".to_string())?;
        validate_result(value, claim)?;
        if !ids.insert(value.result_id.clone())
            || !attempts.insert(value.stage_131_attempt_id.clone())
        {
            return Err("Stage 132 result 重复或重放".to_string());
        }
    }
    values.sort_by(|left, right| left.completed_at.cmp(&right.completed_at));
    Ok(values)
}
async fn read_json_directory<T: for<'de> Deserialize<'de>>(
    path: PathBuf,
    maximum_bytes: u64,
) -> Result<Vec<T>, String> {
    let mut directory = match tokio::fs::read_dir(path).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut values = Vec::new();
    while let Some(entry) = directory
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let metadata = entry.metadata().await.map_err(|error| error.to_string())?;
        if !metadata.is_file() || metadata.len() == 0 || metadata.len() > maximum_bytes {
            return Err("Stage 132 registry file invalid".to_string());
        }
        values.push(
            serde_json::from_slice(
                &tokio::fs::read(entry.path())
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?,
        );
    }
    Ok(values)
}
async fn read_field_bounded(mut field: Field<'_>, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field
        .chunk()
        .await
        .map_err(|_| "multipart_field_read_failed".to_string())?
    {
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err("multipart_field_exceeds_limit".to_string());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn encryption_key() -> Result<[u8; 32], String> {
    let raw = std::env::var(ENCRYPTION_KEY_ENV)
        .map_err(|_| format!("未配置 {ENCRYPTION_KEY_ENV}；尚未消费 Stage 132 尝试"))?;
    if raw.len() != 64 || !raw.bytes().all(|value| value.is_ascii_hexdigit()) {
        return Err(format!(
            "{ENCRYPTION_KEY_ENV} 必须是 64 位十六进制 AES-256 密钥；尚未消费 Stage 132 尝试"
        ));
    }
    let mut key = [0u8; 32];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&raw[index * 2..index * 2 + 2], 16)
            .map_err(|_| "加密密钥无效".to_string())?;
    }
    Ok(key)
}
fn validated_aliases(values: &[String]) -> Result<Vec<String>, String> {
    if values.is_empty() || values.len() > 64 {
        return Err("每个来源工件必须声明 1–64 个账户别名".to_string());
    }
    let mut output = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty()
            || value.len() > 80
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':'))
            || value.chars().filter(|value| value.is_ascii_digit()).count() > 8
        {
            return Err("账户别名无效或疑似包含原始账号".to_string());
        }
        output.push(value.to_string());
    }
    output.sort();
    output.dedup();
    if output.len() != values.len() {
        return Err("账户别名重复".to_string());
    }
    Ok(output)
}
fn bounded_identifier(value: &str) -> Result<(), String> {
    let value = value.trim();
    if value.is_empty()
        || value.chars().count() > MAX_IDENTIFIER_CHARS
        || value.chars().filter(|value| value.is_ascii_digit()).count() > 16
        || value.to_ascii_lowercase().contains("account")
    {
        Err("提供方导出/对账单标识为空、过长或疑似含原始账号".to_string())
    } else {
        Ok(())
    }
}
fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}为空或过长"))
    } else {
        Ok(value.to_string())
    }
}
fn bounded_error_code(value: &str) -> String {
    value
        .chars()
        .filter(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | ':'))
        .take(160)
        .collect::<String>()
        .trim_matches(':')
        .to_string()
}
fn sha256_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 132 fingerprint payload invalid".to_string())?;
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

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    write_private_create_new(path, &bytes).await?;
    set_file_read_only(path)
}
async fn write_private_create_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        options.mode(0o600);
    }
    let mut file = options
        .open(path)
        .await
        .map_err(|error| error.to_string())?;
    file.write_all(bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}
fn set_file_read_only(path: &Path) -> Result<(), String> {
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}
fn set_directory_private(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
async fn cleanup_created(paths: &[PathBuf]) {
    for path in paths {
        let _ = tokio::fs::remove_file(path).await;
    }
}
async fn cleanup_attempt_quarantine(state: &AppState, attempt_id: &str) {
    let _ = tokio::fs::remove_dir_all(quarantine_directory(state).join(attempt_id)).await;
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}
fn start_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-source-artifact-receipt-execution-attempt-starts")
}
fn result_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-source-artifact-receipt-execution-attempt-results")
}
fn quarantine_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-source-artifact-quarantine")
}
fn receipt_attempt_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("opening-portfolio-source-artifact-receipts")
        .join(attempt_id)
}
fn start_path(state: &AppState, attempt_id: &str) -> PathBuf {
    start_directory(state).join(format!("{attempt_id}.json"))
}
fn result_path(state: &AppState, attempt_id: &str) -> PathBuf {
    result_directory(state).join(format!("{attempt_id}.json"))
}
fn receipt_manifest_path(state: &AppState, attempt_id: &str, receipt_id: &str) -> PathBuf {
    receipt_attempt_directory(state, attempt_id)
        .join(receipt_id)
        .join("manifest.json")
}

struct AttemptLock(PathBuf);
impl Drop for AttemptLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<AttemptLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "opening-portfolio-source-artifact-receipt-stage-132-{attempt_id}.lock"
    ));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 132 接收尝试正在进行".to_string())?;
    Ok(AttemptLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_format_rejects_active_pdf_and_sensitive_json() {
        assert!(
            validate_safe_format(
                b"%PDF-1.7\n/JavaScript",
                "original_provider_pdf_statement",
                Some("application/pdf")
            )
            .is_err()
        );
        assert!(
            validate_safe_format(
                br#"{"account_number":"123"}"#,
                "original_provider_json_export",
                Some("application/json")
            )
            .is_err()
        );
        assert!(
            validate_safe_format(
                br#"[{"symbol":"SNDK"}]"#,
                "original_provider_json_export",
                Some("application/json")
            )
            .is_ok()
        );
    }

    #[test]
    fn safe_format_accepts_a_structurally_valid_inert_pdf() {
        use lopdf::{Document, Object, dictionary};

        let mut document = Document::with_version("1.5");
        let pages_id = document.new_object_id();
        let page_id = document.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 100.into(), 100.into()],
        });
        document.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page_id.into()],
                "Count" => 1,
            }),
        );
        let catalog_id = document.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        document.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        document.save_to(&mut bytes).expect("valid inert pdf");

        assert_eq!(
            validate_safe_format(
                &bytes,
                "original_provider_pdf_statement",
                Some("application/pdf")
            ),
            Ok("pdf_magic_safe_structure_screened".to_string())
        );
    }

    #[test]
    fn encryption_is_authenticated_and_content_idempotent() {
        let key = [7u8; 32];
        let plaintext = b"pseudonymized-provider-export";
        let digest = sha256_bytes(plaintext);
        let first = encrypt_artifact(&key, &digest, plaintext).unwrap();
        let second = encrypt_artifact(&key, &digest, plaintext).unwrap();
        assert_eq!(first, second);
        assert_ne!(&first[12..], plaintext);
        assert!(first.len() > plaintext.len());
    }

    #[test]
    fn aliases_reject_raw_account_like_values() {
        assert!(validated_aliases(&["broker_main".to_string()]).is_ok());
        assert!(validated_aliases(&["123456789012".to_string()]).is_err());
        assert!(bounded_identifier("statement-2026Q2").is_ok());
    }

    #[test]
    fn limits_match_reviewed_stage_127_contract() {
        assert_eq!(MAX_ARTIFACT_BYTES, 64 * 1024 * 1024);
        assert_eq!(MAX_RECEIPT_BYTES, 256 * 1024 * 1024);
        assert_eq!(MAX_ARTIFACT_COUNT, 64);
        assert_eq!(
            NEXT_GATE,
            "stage_133_independent_source_artifact_receipt_validation"
        );
    }
}
