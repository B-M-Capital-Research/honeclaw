//! Stage 133 chain-external independent validation of one Stage 132 encrypted receipt.
//!
//! The validator reopens the immutable Stage 131/132 chain, derives every custody path on the
//! server, authenticates and decrypts each AES-256-GCM object with a second implementation, and
//! independently recomputes ciphertext, plaintext, manifest, format and redaction evidence. It
//! never parses financial rows and never materializes or admits an opening portfolio snapshot.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::warn;

use super::controlled_shadow_opening_portfolio_source_artifact_receipt_execution_attempt_claims::{
    OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt, validate_claim,
};
use super::controlled_shadow_opening_portfolio_source_artifact_receipt_execution_attempts::{
    OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
    OpeningPortfolioSourceArtifactReceiptExecutionStatus,
    UntrustedOpeningPortfolioSourceArtifactReceiptManifest,
    opening_portfolio_source_artifact_receipt_results_for_independent_validation,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-independent-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-source-artifact-receipt-independent-validation-v1";
const POLICY_VERSION: &str = "hone-opening-portfolio-source-artifact-receipt-independent-validation-v1-chain-external-authenticated-decryption-no-financial-parsing";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-stage-133-independent-encrypted-receipt-validator-v1";
const NEXT_GATE: &str = "stage_134_opening_portfolio_snapshot_materialization_zero_capability_implementation_registration";
const ENCRYPTION_KEY_ENV: &str = "HONE_OPENING_PORTFOLIO_RECEIPT_ENCRYPTION_KEY";
const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RECEIPT_BYTES: u64 = 256 * 1024 * 1024;
const MAX_ARTIFACT_COUNT: usize = 64;
const MAX_MISMATCH_REASONS: usize = 32;
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ValidateOpeningPortfolioSourceArtifactReceiptRequest {
    expected_stage_131_claim_sha256: String,
    expected_stage_132_result_sha256: String,
    expected_receipt_manifest_sha256: String,
    expected_stage_130_authorization_review_sha256: String,
    expected_stage_129_isolated_receiver_spec_sha256: String,
    expected_stage_127_implementation_contract_sha256: String,
    expected_stage_125_specification_sha256: String,
    validation_reason: String,
    exact_stage_51_through_stage_132_chain_reopened_confirmed: bool,
    validator_independent_from_stage_132_executor_stage_131_claimant_and_complete_prior_chain_confirmed:
        bool,
    result_and_receipt_fingerprints_independently_recomputed_confirmed: bool,
    server_derived_manifest_and_content_addressed_paths_only_confirmed: bool,
    ciphertext_regular_read_only_size_and_sha256_recomputed_confirmed: bool,
    encryption_key_fingerprint_and_aead_authenticated_decryption_confirmed: bool,
    plaintext_length_sha256_and_content_address_independently_recomputed_confirmed: bool,
    format_magic_safe_structure_and_sensitive_field_screening_independently_repeated_confirmed:
        bool,
    receipt_redaction_and_no_original_filename_account_number_or_credential_confirmed: bool,
    terminal_create_once_validation_no_replay_confirmed: bool,
    receipt_validation_only_no_financial_row_parsing_or_snapshot_materialization_confirmed: bool,
    no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioSourceArtifactReceiptValidationVerdict {
    IndependentlyValidatedEncryptedUntrustedReceipt,
    FailedIndependentEncryptedReceiptValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub stage_131_attempt_id: String,
    pub stage_131_claim_sha256: String,
    pub stage_132_result_id: String,
    pub stage_132_result_sha256: String,
    pub receipt_id: String,
    pub receipt_manifest_sha256: String,
    pub receipt_manifest_file_sha256: String,
    pub stage_130_authorization_review_sha256: String,
    pub stage_129_isolated_receiver_spec_sha256: String,
    pub stage_127_implementation_contract_sha256: String,
    pub stage_125_specification_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub validation_reason: String,
    pub stage_132_executor_id: String,
    pub stage_131_claimant_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub exact_stage_51_through_stage_132_chain_verified: bool,
    pub validator_independent_from_executor_claimant_and_complete_prior_chain: bool,
    pub result_fingerprint_independently_verified: bool,
    pub receipt_manifest_fingerprint_independently_verified: bool,
    pub server_derived_manifest_and_content_paths_verified: bool,
    pub ciphertext_count_size_sha256_and_read_only_custody_verified: bool,
    pub encryption_key_fingerprint_verified: bool,
    pub aead_authenticated_decryption_verified: bool,
    pub plaintext_count_size_and_sha256_independently_verified: bool,
    pub format_magic_safe_structure_and_sensitive_fields_independently_verified: bool,
    pub receipt_redaction_and_secret_absence_verified: bool,
    pub no_financial_parsing_snapshot_or_downstream_authority_verified: bool,
    pub artifact_count: usize,
    pub total_plaintext_byte_length: u64,
    pub mismatch_reasons: Vec<String>,
    pub verdict: OpeningPortfolioSourceArtifactReceiptValidationVerdict,
    pub source_artifact_receipt_independently_validated: bool,
    pub future_stage_134_snapshot_materialization_implementation_registration_eligible: bool,
    pub financial_rows_parsed: bool,
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
pub(crate) struct OpeningPortfolioSourceArtifactReceiptValidationCandidate {
    pub stage_131_attempt_id: String,
    pub stage_131_claim_sha256: String,
    pub stage_132_result_sha256: String,
    pub receipt_id: String,
    pub receipt_manifest_sha256: String,
    pub stage_130_authorization_review_sha256: String,
    pub stage_129_isolated_receiver_spec_sha256: String,
    pub stage_127_implementation_contract_sha256: String,
    pub stage_125_specification_sha256: String,
    pub artifact_count: usize,
    pub total_original_byte_length: u64,
    pub stage_132_executor_id: String,
    pub validator_excluded_actor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_endpoint_available: bool,
    pub encryption_key_configured: bool,
    pub candidates: Vec<OpeningPortfolioSourceArtifactReceiptValidationCandidate>,
    pub validations: Vec<OpeningPortfolioSourceArtifactReceiptValidationRecord>,
    pub completed_untrusted_receipt_count: usize,
    pub pending_independent_validation_count: usize,
    pub independently_validated_receipt_count: usize,
    pub failed_independent_validation_count: usize,
    pub future_stage_134_snapshot_materialization_implementation_registration_eligible_count: usize,
    pub validation_status: String,
    pub financial_rows_parsed: bool,
    pub opening_portfolio_snapshot_materialized: bool,
    pub opening_portfolio_snapshot_admitted: bool,
    pub ledger_created: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub next_gate: String,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSourceArtifactReceiptValidationReadinessSummary {
    pub completed_untrusted_receipt_count: usize,
    pub pending_independent_validation_count: usize,
    pub independently_validated_receipt_count: usize,
    pub failed_independent_validation_count: usize,
    pub future_stage_134_eligible_count: usize,
    pub encryption_key_configured: bool,
    pub validation_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt {
    pub claim: OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    pub result: OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
    pub receipt: UntrustedOpeningPortfolioSourceArtifactReceiptManifest,
    pub validation: OpeningPortfolioSourceArtifactReceiptValidationRecord,
}

#[derive(Debug)]
struct IndependentAudit {
    manifest_file_sha256: String,
    exact_chain: bool,
    result_fingerprint: bool,
    manifest_fingerprint: bool,
    server_paths: bool,
    ciphertext_custody: bool,
    key_fingerprint: bool,
    authenticated_decryption: bool,
    plaintext_integrity: bool,
    safe_structure: bool,
    redaction: bool,
    downstream_closed: bool,
    artifact_count: usize,
    total_plaintext_bytes: u64,
    mismatch_reasons: Vec<String>,
}

pub(crate) async fn handle_get_opening_portfolio_source_artifact_receipt_validations(
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
            warn!(%error, "Stage 133 source-artifact receipt validation registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "来源工件 receipt 独立验证表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_opening_portfolio_source_artifact_receipt_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ValidateOpeningPortfolioSourceArtifactReceiptRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match validate_once(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => Json(value).into_response(),
            Err(_) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "验证已落盘，但刷新独立验证表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn opening_portfolio_source_artifact_receipt_validation_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        OpeningPortfolioSourceArtifactReceiptValidationReadinessSummary {
            completed_untrusted_receipt_count: registry.completed_untrusted_receipt_count,
            pending_independent_validation_count: registry.pending_independent_validation_count,
            independently_validated_receipt_count: registry.independently_validated_receipt_count,
            failed_independent_validation_count: registry.failed_independent_validation_count,
            future_stage_134_eligible_count: registry
                .future_stage_134_snapshot_materialization_implementation_registration_eligible_count,
            encryption_key_configured: registry.encryption_key_configured,
            validation_status: registry.validation_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_validated_opening_portfolio_source_artifact_receipts_for_stage_134(
    state: &AppState,
) -> Result<Vec<IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt>, String> {
    let attempts = successful_attempts(state).await?;
    let validations = read_validations(state).await?;
    let attempt_index = attempts
        .into_iter()
        .map(|(claim, result)| (claim.attempt_id.clone(), (claim, result)))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    for validation in validations.into_iter().filter(|value| {
        value.source_artifact_receipt_independently_validated
            && value.future_stage_134_snapshot_materialization_implementation_registration_eligible
    }) {
        let (claim, result) = attempt_index
            .get(&validation.stage_131_attempt_id)
            .cloned()
            .ok_or_else(|| "Stage 133 通过记录缺少当前 Stage 131/132 链".to_string())?;
        let receipt = read_exact_receipt_manifest(state, &claim, &result).await?;
        output.push(
            IndependentlyValidatedOpeningPortfolioSourceArtifactReceipt {
                claim,
                result,
                receipt,
                validation,
            },
        );
    }
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSourceArtifactReceiptValidationRegistry, String> {
    let attempts = successful_attempts(state).await?;
    let validations = read_validations(state).await?;
    let validation_index = validations
        .iter()
        .map(|value| (value.stage_131_attempt_id.as_str(), value))
        .collect::<BTreeMap<_, _>>();
    let candidates = attempts
        .iter()
        .filter(|(claim, _)| !validation_index.contains_key(claim.attempt_id.as_str()))
        .map(|(claim, result)| candidate(claim, result))
        .collect::<Result<Vec<_>, _>>()?;
    let passed = validations
        .iter()
        .filter(|value| value.source_artifact_receipt_independently_validated)
        .count();
    let failed = validations.len().saturating_sub(passed);
    let key_configured = independent_encryption_key().is_ok();
    let validation_status = if passed > 0 {
        "encrypted_source_artifact_receipt_independently_validated_waiting_stage_134"
    } else if failed > 0 {
        "encrypted_source_artifact_receipt_independent_validation_failed_terminal"
    } else if !candidates.is_empty() && !key_configured {
        "pending_source_artifact_receipt_validation_waiting_for_encryption_key"
    } else if !candidates.is_empty() {
        "waiting_for_chain_external_encrypted_source_artifact_receipt_validation"
    } else {
        "waiting_for_completed_stage_132_untrusted_source_artifact_receipt"
    };
    Ok(OpeningPortfolioSourceArtifactReceiptValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_endpoint_available: true,
        encryption_key_configured: key_configured,
        completed_untrusted_receipt_count: attempts.len(),
        pending_independent_validation_count: candidates.len(),
        independently_validated_receipt_count: passed,
        failed_independent_validation_count: failed,
        future_stage_134_snapshot_materialization_implementation_registration_eligible_count:
            passed,
        candidates,
        validations,
        validation_status: validation_status.to_string(),
        financial_rows_parsed: false,
        opening_portfolio_snapshot_materialized: false,
        opening_portfolio_snapshot_admitted: false,
        ledger_created: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        next_gate: NEXT_GATE.to_string(),
        scope: "Stage 133 只由 Stage 132 executor、Stage 131 claimant 和完整前序责任链之外的新管理员，独立重算 result/receipt/ciphertext/plaintext/路径/格式与脱敏证据。通过仍只开放 Stage 134 零能力期初快照物化实现登记；不解析财务行、不物化或准入快照，不创建账本、持仓、现金、净值、训练、订单、券商或交易权限。".to_string(),
    })
}

async fn successful_attempts(
    state: &AppState,
) -> Result<
    Vec<(
        OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
        OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
    )>,
    String,
> {
    let claims =
        opening_portfolio_source_artifact_receipt_execution_attempt_claims_for_receipt(state)
            .await?
            .into_iter()
            .map(|value| (value.attempt_id.clone(), value))
            .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    for result in
        opening_portfolio_source_artifact_receipt_results_for_independent_validation(state).await?
    {
        if result.status
            != OpeningPortfolioSourceArtifactReceiptExecutionStatus::CompletedWithUntrustedReceipt
        {
            continue;
        }
        let claim = claims
            .get(&result.stage_131_attempt_id)
            .cloned()
            .ok_or_else(|| "Stage 132 成功结果缺少 Stage 131 claim".to_string())?;
        validate_claim(&claim)?;
        if result.stage_131_claim_sha256 != claim.claim_sha256 {
            return Err("Stage 131/132 claim 摘要漂移".to_string());
        }
        output.push((claim, result));
    }
    Ok(output)
}

fn candidate(
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    result: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
) -> Result<OpeningPortfolioSourceArtifactReceiptValidationCandidate, String> {
    let contract = &claim
        .authorization
        .receiver
        .receiver_contract
        .exact_approved_implementation_contract;
    let mut excluded = claim.excluded_prior_actor_ids.clone();
    excluded.push(claim.claimed_by.clone());
    excluded.push(result.executed_by.clone());
    excluded.sort();
    excluded.dedup();
    Ok(OpeningPortfolioSourceArtifactReceiptValidationCandidate {
        stage_131_attempt_id: claim.attempt_id.clone(),
        stage_131_claim_sha256: claim.claim_sha256.clone(),
        stage_132_result_sha256: result.result_sha256.clone(),
        receipt_id: result
            .receipt_id
            .clone()
            .ok_or_else(|| "Stage 132 成功结果缺少 receipt id".to_string())?,
        receipt_manifest_sha256: result
            .receipt_manifest_sha256
            .clone()
            .ok_or_else(|| "Stage 132 成功结果缺少 receipt 摘要".to_string())?,
        stage_130_authorization_review_sha256: claim.authorization.review_sha256.clone(),
        stage_129_isolated_receiver_spec_sha256: claim
            .authorization
            .receiver
            .isolated_receiver_spec_sha256
            .clone(),
        stage_127_implementation_contract_sha256: contract.contract_sha256.clone(),
        stage_125_specification_sha256: contract.stage_125_specification_sha256.clone(),
        artifact_count: result.artifact_count,
        total_original_byte_length: result.total_original_byte_length,
        stage_132_executor_id: result.executed_by.clone(),
        validator_excluded_actor_ids: excluded,
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    attempt_id: &str,
    request: ValidateOpeningPortfolioSourceArtifactReceiptRequest,
) -> Result<OpeningPortfolioSourceArtifactReceiptValidationRecord, String> {
    validate_request(attempt_id, &request)?;
    let key = independent_encryption_key()?;
    let _lock = acquire_lock(state, attempt_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|value| value.stage_131_attempt_id == attempt_id)
    {
        return Err("该 Stage 132 receipt 已形成不可覆盖的独立验证终态".to_string());
    }
    let (claim, result) = successful_attempts(state)
        .await?
        .into_iter()
        .find(|(claim, _)| claim.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有完整且待验证的 Stage 132 未受信 receipt".to_string())?;
    verify_expected_bindings(&claim, &result, &request)?;
    let excluded = candidate(&claim, &result)?.validator_excluded_actor_ids;
    let independent =
        !validator_id.trim().is_empty() && excluded.iter().all(|value| value != validator_id);
    if !independent {
        return Err(
            "Stage 133 验证者不能是 Stage 132 executor、Stage 131 claimant 或完整前序责任链角色"
                .to_string(),
        );
    }
    if let Ok(receipt) = read_exact_receipt_manifest(state, &claim, &result).await {
        let key_fingerprint = sha256_bytes(&key)[..16].to_string();
        if receipt.encryption_key_fingerprint != key_fingerprint {
            return Err(format!(
                "当前 {ENCRYPTION_KEY_ENV} 与 receipt 密钥指纹不匹配；尚未写入 Stage 133 终态"
            ));
        }
    }
    let audit = independently_audit(state, &claim, &result, &key).await;
    let passed = independent
        && audit.exact_chain
        && audit.result_fingerprint
        && audit.manifest_fingerprint
        && audit.server_paths
        && audit.ciphertext_custody
        && audit.key_fingerprint
        && audit.authenticated_decryption
        && audit.plaintext_integrity
        && audit.safe_structure
        && audit.redaction
        && audit.downstream_closed
        && audit.mismatch_reasons.is_empty();
    let receipt_id = result
        .receipt_id
        .clone()
        .ok_or_else(|| "Stage 132 结果缺少 receipt id".to_string())?;
    let receipt_manifest_sha256 = result
        .receipt_manifest_sha256
        .clone()
        .ok_or_else(|| "Stage 132 结果缺少 receipt 摘要".to_string())?;
    let contract = &claim
        .authorization
        .receiver
        .receiver_contract
        .exact_approved_implementation_contract;
    let mut record = OpeningPortfolioSourceArtifactReceiptValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        stage_131_attempt_id: claim.attempt_id.clone(),
        stage_131_claim_sha256: claim.claim_sha256.clone(),
        stage_132_result_id: result.result_id.clone(),
        stage_132_result_sha256: result.result_sha256.clone(),
        receipt_id,
        receipt_manifest_sha256,
        receipt_manifest_file_sha256: audit.manifest_file_sha256,
        stage_130_authorization_review_sha256: claim.authorization.review_sha256.clone(),
        stage_129_isolated_receiver_spec_sha256: claim
            .authorization
            .receiver
            .isolated_receiver_spec_sha256
            .clone(),
        stage_127_implementation_contract_sha256: contract.contract_sha256.clone(),
        stage_125_specification_sha256: contract.stage_125_specification_sha256.clone(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        validation_reason: request.validation_reason.trim().to_string(),
        stage_132_executor_id: result.executed_by.clone(),
        stage_131_claimant_id: claim.claimed_by.clone(),
        excluded_prior_actor_ids: excluded,
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        exact_stage_51_through_stage_132_chain_verified: audit.exact_chain,
        validator_independent_from_executor_claimant_and_complete_prior_chain: independent,
        result_fingerprint_independently_verified: audit.result_fingerprint,
        receipt_manifest_fingerprint_independently_verified: audit.manifest_fingerprint,
        server_derived_manifest_and_content_paths_verified: audit.server_paths,
        ciphertext_count_size_sha256_and_read_only_custody_verified: audit.ciphertext_custody,
        encryption_key_fingerprint_verified: audit.key_fingerprint,
        aead_authenticated_decryption_verified: audit.authenticated_decryption,
        plaintext_count_size_and_sha256_independently_verified: audit.plaintext_integrity,
        format_magic_safe_structure_and_sensitive_fields_independently_verified: audit
            .safe_structure,
        receipt_redaction_and_secret_absence_verified: audit.redaction,
        no_financial_parsing_snapshot_or_downstream_authority_verified: audit.downstream_closed,
        artifact_count: audit.artifact_count,
        total_plaintext_byte_length: audit.total_plaintext_bytes,
        mismatch_reasons: audit.mismatch_reasons,
        verdict: if passed {
            OpeningPortfolioSourceArtifactReceiptValidationVerdict::IndependentlyValidatedEncryptedUntrustedReceipt
        } else {
            OpeningPortfolioSourceArtifactReceiptValidationVerdict::FailedIndependentEncryptedReceiptValidation
        },
        source_artifact_receipt_independently_validated: passed,
        future_stage_134_snapshot_materialization_implementation_registration_eligible: passed,
        financial_rows_parsed: false,
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
    record.validation_sha256 =
        fingerprint_without(&record, &["validation_id", "validation_sha256"])?;
    record.validation_id = record.validation_sha256[..32].to_string();
    validate_record(&record, &claim, &result)?;
    write_immutable_json(
        &validation_directory(state)
            .join(&claim.attempt_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

async fn independently_audit(
    state: &AppState,
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    result: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
    key: &[u8; 32],
) -> IndependentAudit {
    let mut reasons = Vec::new();
    let result_fingerprint = fingerprint_without(result, &["result_id", "result_sha256"])
        .is_ok_and(|value| value == result.result_sha256 && result.result_id == value[..32]);
    let expected_manifest_path = expected_receipt_manifest_path(state, claim, result);
    let manifest_bytes = read_regular_bounded(&expected_manifest_path, MAX_MANIFEST_BYTES)
        .await
        .unwrap_or_default();
    let manifest_file_sha256 = sha256_bytes(&manifest_bytes);
    let receipt = serde_json::from_slice::<UntrustedOpeningPortfolioSourceArtifactReceiptManifest>(
        &manifest_bytes,
    )
    .ok();
    let manifest_fingerprint = receipt.as_ref().is_some_and(|value| {
        fingerprint_without(value, &["receipt_id", "receipt_manifest_sha256"]).is_ok_and(|hash| {
            hash == value.receipt_manifest_sha256 && value.receipt_id == hash[..32]
        })
    });
    let contract = &claim
        .authorization
        .receiver
        .receiver_contract
        .exact_approved_implementation_contract;
    let source = &contract.exact_stage_125_specification.source_contract;
    let exact_chain = receipt.as_ref().is_some_and(|value| {
        value.stage_131_attempt_id == claim.attempt_id
            && value.stage_131_claim_sha256 == claim.claim_sha256
            && value.stage_130_authorization_review_sha256 == claim.authorization.review_sha256
            && value.stage_129_isolated_receiver_spec_sha256
                == claim.authorization.receiver.isolated_receiver_spec_sha256
            && value.stage_127_implementation_contract_sha256 == contract.contract_sha256
            && value.stage_125_specification_sha256 == contract.stage_125_specification_sha256
            && value.source_provider_name == source.source_provider_name
            && value.portfolio_scope_alias == source.portfolio_scope_alias
            && value.reporting_currency == source.reporting_currency
            && value.source_timezone == source.source_timezone
            && value.snapshot_as_of_utc == source.snapshot_as_of_utc
            && value.encryption_algorithm
                == "AES-256-GCM; deterministic nonce scoped to identical plaintext digest for idempotent immutable custody"
            && value.artifacts.iter().all(|artifact| {
                source
                    .accepted_artifact_formats
                    .contains(&artifact.declared_format)
            })
            && result.stage_131_attempt_id == claim.attempt_id
            && result.stage_131_claim_sha256 == claim.claim_sha256
            && result.receipt_id.as_deref() == Some(value.receipt_id.as_str())
            && result.receipt_manifest_sha256.as_deref()
                == Some(value.receipt_manifest_sha256.as_str())
            && result.completed_at >= value.hone_received_at
    });
    let server_paths = result.receipt_manifest_relative_path.as_deref()
        == receipt
            .as_ref()
            .map(|value| {
                format!(
                    "opening-portfolio-source-artifact-receipts/{}/{}/manifest.json",
                    claim.attempt_id, value.receipt_id
                )
            })
            .as_deref()
        && expected_manifest_path.starts_with(decision_root(state));
    let key_fingerprint = receipt
        .as_ref()
        .is_some_and(|value| value.encryption_key_fingerprint == sha256_bytes(key)[..16]);
    let redaction = receipt
        .as_ref()
        .is_some_and(independently_validate_receipt_redaction);
    let downstream_closed = receipt.as_ref().is_some_and(|value| {
        value.create_once
            && value.untrusted
            && !value.independent_receipt_validation_completed
            && !value.financial_rows_parsed
            && !value.opening_portfolio_snapshot_materialized
            && !value.opening_portfolio_snapshot_admitted
    }) && result.claim_consumed
        && result.start_marker_persisted_before_first_source_byte
        && !result.receiver_artifact_spawned_or_executed
        && result.source_artifact_received_or_read
        && result.source_artifact_may_have_been_read
        && result.original_bytes_encrypted_at_rest
        && result.receipt_manifest_created
        && result.receipt_untrusted
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

    let mut ciphertext_custody = receipt.is_some();
    let mut authenticated_decryption = receipt.is_some();
    let mut plaintext_integrity = receipt.is_some();
    let mut safe_structure = receipt.is_some();
    let mut total_plaintext_bytes = 0u64;
    let artifact_count = receipt.as_ref().map_or(0, |value| value.artifacts.len());
    if let Some(receipt) = &receipt {
        ciphertext_custody &=
            !receipt.artifacts.is_empty() && receipt.artifacts.len() <= MAX_ARTIFACT_COUNT;
        for artifact in &receipt.artifacts {
            let expected_relative = format!(
                "opening-portfolio-source-artifacts/{}/{}/original.bin.enc",
                contract.stage_125_registration_id, artifact.artifact_sha256
            );
            let path = decision_root(state).join(&expected_relative);
            let metadata = tokio::fs::symlink_metadata(&path).await.ok();
            let encrypted = read_regular_bounded(&path, MAX_ARTIFACT_BYTES + 28)
                .await
                .unwrap_or_default();
            ciphertext_custody &= artifact.storage_relative_path == expected_relative
                && path.starts_with(decision_root(state))
                && metadata.as_ref().is_some_and(|value| {
                    value.is_file()
                        && !value.file_type().is_symlink()
                        && value.permissions().readonly()
                })
                && !encrypted.is_empty()
                && encrypted.len() as u64 == artifact.storage_byte_length
                && sha256_bytes(&encrypted) == artifact.storage_object_sha256
                && artifact.storage_byte_length == artifact.byte_length.saturating_add(28)
                && artifact.original_bytes_encrypted_immutable;
            match independently_decrypt_artifact(key, &artifact.artifact_sha256, &encrypted) {
                Ok(mut plaintext) => {
                    let plaintext_hash = sha256_bytes(&plaintext);
                    total_plaintext_bytes =
                        total_plaintext_bytes.saturating_add(plaintext.len() as u64);
                    plaintext_integrity &= plaintext.len() as u64 == artifact.byte_length
                        && artifact.byte_length > 0
                        && artifact.byte_length <= MAX_ARTIFACT_BYTES
                        && plaintext_hash == artifact.artifact_sha256
                        && artifact.artifact_id == plaintext_hash[..32];
                    let independently_validated =
                        independent_safe_format(&plaintext, &artifact.declared_format);
                    safe_structure &= independently_validated.as_ref().is_ok_and(|value| {
                        value == &artifact.validated_format
                            && artifact
                                .format_magic_and_safe_structure_validated_without_financial_parsing
                    });
                    plaintext.fill(0);
                }
                Err(_) => {
                    authenticated_decryption = false;
                    plaintext_integrity = false;
                    safe_structure = false;
                }
            }
        }
        authenticated_decryption &= ciphertext_custody;
        plaintext_integrity &= total_plaintext_bytes == receipt.total_original_byte_length
            && total_plaintext_bytes == result.total_original_byte_length
            && total_plaintext_bytes <= MAX_RECEIPT_BYTES
            && artifact_count == result.artifact_count;
    }

    push_mismatch(
        &mut reasons,
        exact_chain,
        "Stage 51–132 exact chain mismatch",
    );
    push_mismatch(
        &mut reasons,
        result_fingerprint,
        "Stage 132 result fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        manifest_fingerprint,
        "receipt manifest fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        server_paths,
        "manifest or content path is not server-derived",
    );
    push_mismatch(
        &mut reasons,
        ciphertext_custody,
        "ciphertext custody, size, hash or permission mismatch",
    );
    push_mismatch(
        &mut reasons,
        key_fingerprint,
        "encryption key fingerprint mismatch",
    );
    push_mismatch(
        &mut reasons,
        authenticated_decryption,
        "AES-GCM authenticated decryption failed",
    );
    push_mismatch(
        &mut reasons,
        plaintext_integrity,
        "plaintext count, size, SHA-256 or content address mismatch",
    );
    push_mismatch(
        &mut reasons,
        safe_structure,
        "independent format or safe-structure screening mismatch",
    );
    push_mismatch(
        &mut reasons,
        redaction,
        "receipt alias, identifier or secret redaction mismatch",
    );
    push_mismatch(
        &mut reasons,
        downstream_closed,
        "financial parsing, snapshot or downstream authority is not closed",
    );

    IndependentAudit {
        manifest_file_sha256,
        exact_chain,
        result_fingerprint,
        manifest_fingerprint,
        server_paths,
        ciphertext_custody,
        key_fingerprint,
        authenticated_decryption,
        plaintext_integrity,
        safe_structure,
        redaction,
        downstream_closed,
        artifact_count,
        total_plaintext_bytes,
        mismatch_reasons: reasons,
    }
}

fn independently_decrypt_artifact(
    key: &[u8; 32],
    artifact_sha256: &str,
    encrypted: &[u8],
) -> Result<Vec<u8>, String> {
    if encrypted.len() < 28 || !valid_sha256(artifact_sha256) {
        return Err("encrypted_artifact_envelope_invalid".to_string());
    }
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .map_err(|_| "validation_key_invalid".to_string())?;
    mac.update(b"hone-opening-portfolio-receipt-aes-gcm-nonce-v1\0");
    mac.update(artifact_sha256.as_bytes());
    let expected_nonce = mac.finalize().into_bytes();
    if encrypted[..12] != expected_nonce[..12] {
        return Err("encrypted_artifact_nonce_mismatch".to_string());
    }
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|_| "validation_key_invalid".to_string())?;
    let mut aad = b"hone-opening-portfolio-receipt-artifact-v1\0".to_vec();
    aad.extend_from_slice(artifact_sha256.as_bytes());
    cipher
        .decrypt(
            Nonce::from_slice(&encrypted[..12]),
            aes_gcm::aead::Payload {
                msg: &encrypted[12..],
                aad: &aad,
            },
        )
        .map_err(|_| "encrypted_artifact_authentication_failed".to_string())
}

fn independent_safe_format(bytes: &[u8], declared: &str) -> Result<String, String> {
    let lowered = bytes
        .iter()
        .map(|byte| byte.to_ascii_lowercase())
        .collect::<Vec<_>>();
    match declared {
        "original_provider_pdf_statement" => {
            if !bytes.starts_with(b"%PDF-") {
                return Err("pdf_magic_mismatch".to_string());
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
                    return Err("pdf_active_embedded_or_encrypted_content".to_string());
                }
            }
            lopdf::Document::load_mem(bytes)
                .map_err(|_| "pdf_structure_parse_failed".to_string())?;
            Ok("pdf_magic_safe_structure_screened".to_string())
        }
        "original_provider_csv_export" => {
            let text = std::str::from_utf8(bytes).map_err(|_| "csv_not_utf8".to_string())?;
            if text.contains('\0') || text.lines().any(|line| line.len() > 1024 * 1024) {
                return Err("csv_unsafe_structure".to_string());
            }
            let header = text
                .lines()
                .next()
                .ok_or_else(|| "csv_header_missing".to_string())?;
            independent_reject_sensitive_name(header)?;
            if text
                .lines()
                .skip(1)
                .flat_map(|line| line.split(','))
                .any(|cell| {
                    cell.trim_start_matches(&[' ', '\t', '"', '\''][..])
                        .starts_with(['=', '@'])
                })
            {
                return Err("csv_formula_active_content".to_string());
            }
            Ok("utf8_csv_safe_structure_screened".to_string())
        }
        "original_provider_json_export" => {
            let value: Value =
                serde_json::from_slice(bytes).map_err(|_| "json_invalid".to_string())?;
            if !value.is_object() && !value.is_array() {
                return Err("json_top_level_structure_rejected".to_string());
            }
            independent_reject_sensitive_json_keys(&value)?;
            Ok("json_safe_structure_screened".to_string())
        }
        _ => Err("unsupported_declared_format".to_string()),
    }
}

fn independently_validate_receipt_redaction(
    receipt: &UntrustedOpeningPortfolioSourceArtifactReceiptManifest,
) -> bool {
    valid_sha256(&receipt.provider_artifact_identifier_sha256)
        && independently_redacted_label(&receipt.source_provider_name, true)
        && independently_redacted_label(&receipt.portfolio_scope_alias, false)
        && receipt.artifacts.iter().all(|artifact| {
            !artifact.source_account_aliases.is_empty()
                && artifact.source_account_aliases.len() <= 64
                && sorted_unique(&artifact.source_account_aliases)
                && artifact.source_account_aliases.iter().all(|value| {
                    !value.is_empty()
                        && value.len() <= 80
                        && value.bytes().all(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b':')
                        })
                        && value.chars().filter(|value| value.is_ascii_digit()).count() <= 8
                })
        })
}

fn independently_redacted_label(value: &str, spaces_allowed: bool) -> bool {
    let trimmed = value.trim();
    let lowered = trimmed.to_ascii_lowercase();
    !trimmed.is_empty()
        && trimmed.len() <= 160
        && !trimmed.contains(['/', '\\'])
        && ![".pdf", ".csv", ".json"]
            .iter()
            .any(|suffix| lowered.ends_with(suffix))
        && trimmed
            .chars()
            .filter(|value| value.is_ascii_digit())
            .count()
            <= 8
        && independent_reject_sensitive_name(trimmed).is_ok()
        && trimmed.chars().all(|character| {
            character.is_alphanumeric()
                || matches!(character, '_' | '-' | ':' | '.')
                || (spaces_allowed && character == ' ')
        })
}

fn independent_reject_sensitive_name(value: &str) -> Result<(), String> {
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
            return Err("raw_account_or_secret_field".to_string());
        }
    }
    Ok(())
}

fn independent_reject_sensitive_json_keys(value: &Value) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                independent_reject_sensitive_name(key)?;
                independent_reject_sensitive_json_keys(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                independent_reject_sensitive_json_keys(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn verify_expected_bindings(
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    result: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
    request: &ValidateOpeningPortfolioSourceArtifactReceiptRequest,
) -> Result<(), String> {
    let contract = &claim
        .authorization
        .receiver
        .receiver_contract
        .exact_approved_implementation_contract;
    if request.expected_stage_131_claim_sha256 != claim.claim_sha256
        || request.expected_stage_132_result_sha256 != result.result_sha256
        || result.receipt_manifest_sha256.as_deref()
            != Some(request.expected_receipt_manifest_sha256.as_str())
        || request.expected_stage_130_authorization_review_sha256
            != claim.authorization.review_sha256
        || request.expected_stage_129_isolated_receiver_spec_sha256
            != claim.authorization.receiver.isolated_receiver_spec_sha256
        || request.expected_stage_127_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_stage_125_specification_sha256
            != contract.stage_125_specification_sha256
    {
        return Err("Stage 125–132 预期绑定已经漂移".to_string());
    }
    Ok(())
}

fn validate_request(
    attempt_id: &str,
    request: &ValidateOpeningPortfolioSourceArtifactReceiptRequest,
) -> Result<(), String> {
    if !valid_id(attempt_id)
        || [
            request.expected_stage_131_claim_sha256.as_str(),
            request.expected_stage_132_result_sha256.as_str(),
            request.expected_receipt_manifest_sha256.as_str(),
            request
                .expected_stage_130_authorization_review_sha256
                .as_str(),
            request
                .expected_stage_129_isolated_receiver_spec_sha256
                .as_str(),
            request
                .expected_stage_127_implementation_contract_sha256
                .as_str(),
            request.expected_stage_125_specification_sha256.as_str(),
        ]
        .iter()
        .any(|value| !valid_sha256(value))
        || request.validation_reason.trim().is_empty()
        || request.validation_reason.chars().count() > MAX_TEXT_CHARS
    {
        return Err("Stage 133 请求 ID、摘要或验证原因无效".to_string());
    }
    let confirmed = request.exact_stage_51_through_stage_132_chain_reopened_confirmed
        && request.validator_independent_from_stage_132_executor_stage_131_claimant_and_complete_prior_chain_confirmed
        && request.result_and_receipt_fingerprints_independently_recomputed_confirmed
        && request.server_derived_manifest_and_content_addressed_paths_only_confirmed
        && request.ciphertext_regular_read_only_size_and_sha256_recomputed_confirmed
        && request.encryption_key_fingerprint_and_aead_authenticated_decryption_confirmed
        && request.plaintext_length_sha256_and_content_address_independently_recomputed_confirmed
        && request.format_magic_safe_structure_and_sensitive_field_screening_independently_repeated_confirmed
        && request.receipt_redaction_and_no_original_filename_account_number_or_credential_confirmed
        && request.terminal_create_once_validation_no_replay_confirmed
        && request.receipt_validation_only_no_financial_row_parsing_or_snapshot_materialization_confirmed
        && request.no_snapshot_admission_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed;
    if !confirmed {
        return Err("Stage 133 必要独立验证确认不完整".to_string());
    }
    Ok(())
}

fn validate_record(
    record: &OpeningPortfolioSourceArtifactReceiptValidationRecord,
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    result: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
) -> Result<(), String> {
    let passed = record.source_artifact_receipt_independently_validated;
    let all_evidence = record.exact_stage_51_through_stage_132_chain_verified
        && record.validator_independent_from_executor_claimant_and_complete_prior_chain
        && record.result_fingerprint_independently_verified
        && record.receipt_manifest_fingerprint_independently_verified
        && record.server_derived_manifest_and_content_paths_verified
        && record.ciphertext_count_size_sha256_and_read_only_custody_verified
        && record.encryption_key_fingerprint_verified
        && record.aead_authenticated_decryption_verified
        && record.plaintext_count_size_and_sha256_independently_verified
        && record.format_magic_safe_structure_and_sensitive_fields_independently_verified
        && record.receipt_redaction_and_secret_absence_verified
        && record.no_financial_parsing_snapshot_or_downstream_authority_verified
        && record.mismatch_reasons.is_empty();
    let authority_closed = !record.financial_rows_parsed
        && !record.opening_portfolio_snapshot_materialized
        && !record.opening_portfolio_snapshot_admitted
        && !record.financial_event_allowlist_nonempty
        && !record.ledger_created
        && !record.position_or_cash_written
        && !record.nav_or_performance_written
        && !record.model_or_metric_store_written
        && !record.training_or_rl_feedback_authorized
        && !record.reward_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256)
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256
            != fingerprint_without(record, &["validation_id", "validation_sha256"])?
        || record.stage_131_attempt_id != claim.attempt_id
        || record.stage_131_claim_sha256 != claim.claim_sha256
        || record.stage_132_result_id != result.result_id
        || record.stage_132_result_sha256 != result.result_sha256
        || record.stage_132_executor_id != result.executed_by
        || record.stage_131_claimant_id != claim.claimed_by
        || record.validated_by.trim().is_empty()
        || record.validation_reason.trim().is_empty()
        || record.excluded_prior_actor_ids.contains(&record.validated_by)
        || !authority_closed
        || passed != all_evidence
        || record.future_stage_134_snapshot_materialization_implementation_registration_eligible
            != passed
        || (passed
            && record.verdict
                != OpeningPortfolioSourceArtifactReceiptValidationVerdict::IndependentlyValidatedEncryptedUntrustedReceipt)
        || (!passed
            && record.verdict
                != OpeningPortfolioSourceArtifactReceiptValidationVerdict::FailedIndependentEncryptedReceiptValidation)
    {
        return Err("Stage 133 validation record 无效、漂移或越权".to_string());
    }
    Ok(())
}

async fn read_exact_receipt_manifest(
    state: &AppState,
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    result: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
) -> Result<UntrustedOpeningPortfolioSourceArtifactReceiptManifest, String> {
    let path = expected_receipt_manifest_path(state, claim, result);
    let bytes = read_regular_bounded(&path, MAX_MANIFEST_BYTES).await?;
    let receipt: UntrustedOpeningPortfolioSourceArtifactReceiptManifest =
        serde_json::from_slice(&bytes)
            .map_err(|_| "Stage 132 receipt manifest JSON 无效".to_string())?;
    let expected_relative = format!(
        "opening-portfolio-source-artifact-receipts/{}/{}/manifest.json",
        claim.attempt_id, receipt.receipt_id
    );
    if result.receipt_manifest_relative_path.as_deref() != Some(expected_relative.as_str())
        || result.receipt_id.as_deref() != Some(receipt.receipt_id.as_str())
        || result.receipt_manifest_sha256.as_deref()
            != Some(receipt.receipt_manifest_sha256.as_str())
        || receipt.receipt_manifest_sha256
            != fingerprint_without(&receipt, &["receipt_id", "receipt_manifest_sha256"])?
        || receipt.receipt_id != receipt.receipt_manifest_sha256[..32]
    {
        return Err("Stage 132 receipt manifest 路径、ID 或摘要漂移".to_string());
    }
    Ok(receipt)
}

fn expected_receipt_manifest_path(
    state: &AppState,
    claim: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptClaim,
    result: &OpeningPortfolioSourceArtifactReceiptExecutionAttemptResult,
) -> PathBuf {
    decision_root(state)
        .join("opening-portfolio-source-artifact-receipts")
        .join(&claim.attempt_id)
        .join(result.receipt_id.as_deref().unwrap_or("invalid-receipt-id"))
        .join("manifest.json")
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<OpeningPortfolioSourceArtifactReceiptValidationRecord>, String> {
    let attempts = successful_attempts(state)
        .await?
        .into_iter()
        .map(|(claim, result)| (claim.attempt_id.clone(), (claim, result)))
        .collect::<BTreeMap<_, _>>();
    let mut output = Vec::new();
    let mut root = match tokio::fs::read_dir(validation_directory(state)).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(output),
        Err(error) => return Err(error.to_string()),
    };
    let mut attempt_ids = BTreeSet::new();
    let mut validation_ids = BTreeSet::new();
    while let Some(attempt_entry) = root.next_entry().await.map_err(|error| error.to_string())? {
        if !attempt_entry
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let attempt_dir_name = attempt_entry.file_name().to_string_lossy().to_string();
        let mut directory = tokio::fs::read_dir(attempt_entry.path())
            .await
            .map_err(|error| error.to_string())?;
        while let Some(entry) = directory
            .next_entry()
            .await
            .map_err(|error| error.to_string())?
        {
            if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let bytes = read_regular_bounded(&entry.path(), MAX_MANIFEST_BYTES).await?;
            let record: OpeningPortfolioSourceArtifactReceiptValidationRecord =
                serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
            let (claim, result) = attempts
                .get(&record.stage_131_attempt_id)
                .ok_or_else(|| "Stage 133 validation 缺少当前 Stage 131/132 链".to_string())?;
            validate_record(&record, claim, result)?;
            if attempt_dir_name != record.stage_131_attempt_id
                || entry.file_name().to_string_lossy() != format!("{}.json", record.validation_id)
                || !attempt_ids.insert(record.stage_131_attempt_id.clone())
                || !validation_ids.insert(record.validation_id.clone())
            {
                return Err("Stage 133 validation 路径重复、重放或漂移".to_string());
            }
            output.push(record);
        }
    }
    output.sort_by(|left, right| left.validated_at.cmp(&right.validated_at));
    Ok(output)
}

async fn read_regular_bounded(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, String> {
    let metadata = tokio::fs::symlink_metadata(path)
        .await
        .map_err(|error| error.to_string())?;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > maximum_bytes
    {
        return Err("Stage 133 custody file missing, non-regular or outside bounds".to_string());
    }
    tokio::fs::read(path)
        .await
        .map_err(|error| error.to_string())
}

fn independent_encryption_key() -> Result<[u8; 32], String> {
    let raw = std::env::var(ENCRYPTION_KEY_ENV)
        .map_err(|_| format!("未配置 {ENCRYPTION_KEY_ENV}；Stage 133 尚未形成终态"))?;
    if raw.len() != 64 || !raw.bytes().all(|value| value.is_ascii_hexdigit()) {
        return Err(format!(
            "{ENCRYPTION_KEY_ENV} 必须是 64 位十六进制 AES-256 密钥；Stage 133 尚未形成终态"
        ));
    }
    let mut key = [0u8; 32];
    for (index, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&raw[index * 2..index * 2 + 2], 16)
            .map_err(|_| "Stage 133 encryption key invalid".to_string())?;
    }
    Ok(key)
}

fn validator_implementation_sha256() -> String {
    sha256_bytes(
        format!(
            "{VALIDATOR_IMPLEMENTATION_VERSION}|independent-result-manifest-ciphertext-aead-plaintext-format-redaction-v1"
        )
        .as_bytes(),
    )
}

fn push_mismatch(reasons: &mut Vec<String>, passed: bool, reason: &str) {
    if !passed && reasons.len() < MAX_MISMATCH_REASONS {
        reasons.push(reason.to_string());
    }
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|window| window[0] < window[1])
}

fn sha256_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 133 fingerprint payload invalid".to_string())?;
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
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())?;
    let mut permissions = std::fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn validation_directory(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-source-artifact-receipt-independent-validations")
}

struct ValidationLock(PathBuf);

impl Drop for ValidationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<ValidationLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "opening-portfolio-source-artifact-receipt-stage-133-{attempt_id}.lock"
    ));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 133 验证正在进行".to_string())?;
    Ok(ValidationLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encrypt_fixture(key: &[u8; 32], digest: &str, plaintext: &[u8]) -> Vec<u8> {
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = <HmacSha256 as Mac>::new_from_slice(key).unwrap();
        mac.update(b"hone-opening-portfolio-receipt-aes-gcm-nonce-v1\0");
        mac.update(digest.as_bytes());
        let nonce_bytes = mac.finalize().into_bytes();
        let nonce = Nonce::from_slice(&nonce_bytes[..12]);
        let cipher = Aes256Gcm::new_from_slice(key).unwrap();
        let mut aad = b"hone-opening-portfolio-receipt-artifact-v1\0".to_vec();
        aad.extend_from_slice(digest.as_bytes());
        let ciphertext = cipher
            .encrypt(
                nonce,
                aes_gcm::aead::Payload {
                    msg: plaintext,
                    aad: &aad,
                },
            )
            .unwrap();
        [nonce.as_slice(), ciphertext.as_slice()].concat()
    }

    #[test]
    fn independent_aead_decryption_authenticates_digest_and_aad() {
        let key = [7u8; 32];
        let plaintext = br#"[{"symbol":"SNDK","quantity":"10"}]"#;
        let digest = sha256_bytes(plaintext);
        let encrypted = encrypt_fixture(&key, &digest, plaintext);
        assert_eq!(
            independently_decrypt_artifact(&key, &digest, &encrypted).unwrap(),
            plaintext
        );
        let mut tampered = encrypted.clone();
        *tampered.last_mut().unwrap() ^= 1;
        assert!(independently_decrypt_artifact(&key, &digest, &tampered).is_err());
        assert!(independently_decrypt_artifact(&key, &sha256_bytes(b"other"), &encrypted).is_err());
    }

    #[test]
    fn independent_structure_screening_rejects_active_or_sensitive_content() {
        assert!(
            independent_safe_format(b"%PDF-1.7\n/Launch", "original_provider_pdf_statement")
                .is_err()
        );
        assert!(
            independent_safe_format(
                b"ticker,account_number\nSNDK,123",
                "original_provider_csv_export"
            )
            .is_err()
        );
        assert!(
            independent_safe_format(
                br#"{"symbol":"SNDK","api_key":"secret"}"#,
                "original_provider_json_export"
            )
            .is_err()
        );
        assert!(
            independent_safe_format(
                br#"[{"symbol":"SNDK","quantity":"10"}]"#,
                "original_provider_json_export"
            )
            .is_ok()
        );
    }

    #[test]
    fn stage_133_limits_and_next_gate_are_narrow() {
        assert_eq!(MAX_ARTIFACT_COUNT, 64);
        assert_eq!(MAX_ARTIFACT_BYTES, 64 * 1024 * 1024);
        assert_eq!(MAX_RECEIPT_BYTES, 256 * 1024 * 1024);
        assert!(NEXT_GATE.contains("stage_134"));
        assert!(NEXT_GATE.contains("zero_capability"));
        assert!(!NEXT_GATE.contains("admission"));
        assert!(!NEXT_GATE.contains("trade"));
    }

    #[test]
    fn validator_implementation_identity_is_stable() {
        assert_eq!(validator_implementation_sha256().len(), 64);
        assert_eq!(
            VALIDATOR_IMPLEMENTATION_VERSION,
            "hone-stage-133-independent-encrypted-receipt-validator-v1"
        );
    }

    #[test]
    fn receipt_labels_reject_filenames_paths_accounts_and_credentials() {
        assert!(independently_redacted_label("富途证券", true));
        assert!(independently_redacted_label("broker_main", false));
        assert!(!independently_redacted_label("/tmp/statement.pdf", true));
        assert!(!independently_redacted_label("account_number", false));
        assert!(!independently_redacted_label("broker_123456789", false));
    }
}
