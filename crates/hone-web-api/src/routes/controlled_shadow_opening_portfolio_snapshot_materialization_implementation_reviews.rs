//! Stage 135 chain-external independent review of one Stage 134 zero-capability
//! opening-portfolio snapshot materialization implementation contract.
//!
//! The reviewer rebuilds the complete contract without calling the Stage 134 builder. Approval
//! opens only a future Stage 136 isolated materializer-specification registration. This module
//! has no key or source access, performs no decryption or parsing, and creates no snapshot or
//! other financial state.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;
use tracing::warn;

use super::controlled_shadow_opening_portfolio_snapshot_materialization_implementations::{
    OpeningPortfolioSnapshotMaterializationImplementationContract,
    OpeningPortfolioSnapshotMaterializationImplementationRegistration,
    ZeroCapabilityOpeningPortfolioSnapshotMaterializationAuthorityBoundary,
    independently_reviewable_opening_portfolio_snapshot_materialization_implementations,
    validate_opening_portfolio_snapshot_materialization_implementation_for_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-implementation-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-implementation-independent-audit-v1";
const POLICY_VERSION: &str = "hone-opening-portfolio-snapshot-materialization-implementation-chain-external-review-v1-zero-capability";
const CONTRACT_SCHEMA_VERSION: &str =
    "hone-opening-portfolio-snapshot-materialization-zero-capability-contract-v1";
const PROTOCOL_VERSION: &str = "hone-opening-portfolio-snapshot-materialization-v1-not-executable";
const STAGE_134_NEXT_GATE: &str =
    "stage_135_opening_portfolio_snapshot_materialization_implementation_independent_review";
const STAGE_135_NEXT_GATE: &str =
    "stage_136_opening_portfolio_snapshot_isolated_materializer_specification_registration";
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_RECORD_FILE_BYTES: u64 = 2 * 1024 * 1024;
const FUTURE_MAX_INPUT_ARTIFACTS: usize = 64;
const FUTURE_MAX_INPUT_BYTES: u64 = 256 * 1024 * 1024;
const FUTURE_MAX_OUTPUT_BYTES: u64 = 64 * 1024 * 1024;
const FUTURE_MAX_OUTPUT_ROWS: usize = 1_000_000;

const EXPECTED_FUNCTION_IDS: [&str; 10] = [
    "opening_snapshot_validate_receipt_spec_binding_v1",
    "opening_snapshot_decrypt_ephemeral_memory_only_v1",
    "opening_snapshot_parse_provider_pdf_csv_json_deterministically_v1",
    "opening_snapshot_normalize_complete_account_scope_v1",
    "opening_snapshot_normalize_all_financial_sections_v1",
    "opening_snapshot_reconcile_instrument_identity_corporate_actions_v1",
    "opening_snapshot_enforce_full_completeness_fail_closed_v1",
    "opening_snapshot_canonicalize_exact_decimal_no_float_v1",
    "opening_snapshot_attach_artifact_row_provenance_v1",
    "opening_snapshot_create_once_untrusted_candidate_v1",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict {
    ApprovedForFutureIsolatedMaterializerSpecificationRegistration,
    ChangesRequiredRebuildMaterializationImplementation,
    RejectedMaterializationImplementation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations {
    pub exact_current_stage_51_through_stage_134_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain_confirmed:
        bool,
    pub implementation_contract_validation_result_claim_receipt_and_specification_hashes_independently_reproduced_confirmed:
        bool,
    pub complete_contract_rebuilt_without_stage_134_builder_confirmed: bool,
    pub all_stage_134_registration_confirmations_revalidated_confirmed: bool,
    pub input_only_independently_validated_content_addressed_receipt_confirmed: bool,
    pub future_decryption_only_in_isolated_ephemeral_memory_confirmed: bool,
    pub deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed: bool,
    pub complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed: bool,
    pub exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed: bool,
    pub instrument_identity_precedence_and_corporate_action_reconciliation_confirmed: bool,
    pub no_default_manual_or_inferred_financial_values_and_whole_snapshot_failure_confirmed: bool,
    pub statement_market_values_informational_and_no_nav_or_performance_confirmed: bool,
    pub every_output_row_bound_to_artifact_hash_and_source_locator_with_redaction_confirmed: bool,
    pub output_create_once_untrusted_and_separate_validation_and_admission_confirmed: bool,
    pub no_key_input_read_decrypt_parse_artifact_entrypoint_runtime_mount_or_output_confirmed: bool,
    pub no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed:
        bool,
    pub approval_only_opens_future_stage_136_isolated_materializer_specification_registration_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewOpeningPortfolioSnapshotMaterializationImplementationRequest {
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_stage_133_validation_sha256: String,
    expected_stage_132_result_sha256: String,
    expected_stage_131_claim_sha256: String,
    expected_receipt_manifest_sha256: String,
    expected_stage_125_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict,
    rationale: String,
    binding_and_recomputation_assessment: String,
    parser_schema_and_completeness_assessment: String,
    decimal_identity_and_provenance_assessment: String,
    failure_separation_and_zero_capability_assessment: String,
    known_limitations: String,
    future_materializer_constraints: String,
    confirmations: OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub stage_133_validation_sha256: String,
    pub stage_132_result_sha256: String,
    pub stage_131_claim_sha256: String,
    pub receipt_manifest_sha256: String,
    pub stage_125_specification_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub complete_contract_rebuilt_without_stage_134_builder: bool,
    pub rebuilt_contract_exactly_matches_record: bool,
    pub exact_current_stage_51_through_stage_134_binding_valid: bool,
    pub all_stage_134_registration_confirmations_valid: bool,
    pub deterministic_adapter_and_resource_contract_valid: bool,
    pub complete_financial_sections_and_whole_snapshot_failure_valid: bool,
    pub exact_decimal_identity_corporate_action_and_provenance_valid: bool,
    pub untrusted_output_validation_admission_separation_valid: bool,
    pub all_key_input_parser_financial_model_order_broker_and_trading_authority_closed: bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration,
    pub independent_audit: OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict,
    pub rationale: String,
    pub binding_and_recomputation_assessment: String,
    pub parser_schema_and_completeness_assessment: String,
    pub decimal_identity_and_provenance_assessment: String,
    pub failure_separation_and_zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_materializer_constraints: String,
    pub confirmations: OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations,
    pub confirmations_complete: bool,
    pub reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain:
        bool,
    pub zero_capability_materialization_implementation_independently_approved: bool,
    pub future_stage_136_isolated_materializer_specification_registration_eligible: bool,
    pub isolated_materializer_specification_registered: bool,
    pub decryption_key_or_input_accessed: bool,
    pub receipt_decrypted_or_read: bool,
    pub parser_artifact_entrypoint_or_runtime_present: bool,
    pub output_candidate_created: bool,
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
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationReviewItem {
    pub implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration,
    pub current_independent_audit:
        OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit,
    pub review: Option<OpeningPortfolioSnapshotMaterializationImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_stage_136_isolated_materializer_specification_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<OpeningPortfolioSnapshotMaterializationImplementationReviewItem>,
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_stage_136_isolated_materializer_specification_registration_eligible_count: usize,
    pub review_status: String,
    pub isolated_materializer_specification_registered: bool,
    pub decryption_key_or_input_accessed: bool,
    pub receipt_decrypted_or_read: bool,
    pub parser_artifact_entrypoint_or_runtime_present: bool,
    pub output_candidate_created: bool,
    pub opening_portfolio_snapshot_present: bool,
    pub financial_event_allowlist_nonempty: bool,
    pub ledger_created: bool,
    pub position_or_cash_written: bool,
    pub nav_or_performance_written: bool,
    pub training_or_rl_feedback_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub next_gate: String,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct OpeningPortfolioSnapshotMaterializationImplementationReviewReadinessSummary {
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_stage_136_isolated_materializer_specification_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation {
    pub implementation: OpeningPortfolioSnapshotMaterializationImplementationRegistration,
    pub review: OpeningPortfolioSnapshotMaterializationImplementationReviewRecord,
}

pub(crate) async fn handle_get_opening_portfolio_snapshot_materialization_implementation_reviews(
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
            warn!(%error, "Stage 135 snapshot materialization implementation review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "期初组合快照物化实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_opening_portfolio_snapshot_materialization_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewOpeningPortfolioSnapshotMaterializationImplementationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &implementation_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn opening_portfolio_snapshot_materialization_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationReviewReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        OpeningPortfolioSnapshotMaterializationImplementationReviewReadinessSummary {
            implementation_count: value.implementation_count,
            review_eligible_count: value.review_eligible_count,
            reviewed_count: value.reviewed_count,
            independently_approved_count: value.independently_approved_count,
            changes_required_or_rejected_count: value.changes_required_or_rejected_count,
            future_stage_136_isolated_materializer_specification_registration_eligible_count: value
                .future_stage_136_isolated_materializer_specification_registration_eligible_count,
            review_status: value.review_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_approved_opening_portfolio_snapshot_materialization_implementations_for_isolated_materializer_specification_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation>, String>
{
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let review = item.review?;
            item.future_stage_136_isolated_materializer_specification_registration_eligible
                .then_some(
                    IndependentlyApprovedOpeningPortfolioSnapshotMaterializationImplementation {
                        implementation: item.implementation,
                        review,
                    },
                )
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry, String> {
    let sources =
        independently_reviewable_opening_portfolio_snapshot_materialization_implementations(state)
            .await?;
    let mut items = Vec::new();
    for source in sources {
        let implementation = source.implementation;
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 134 期初快照物化实现独立审计失败，晋级关闭".to_string());
        }
        let review = read_review(state, &implementation.implementation_id).await?;
        if review.as_ref().is_some_and(|value| {
            value.implementation != implementation || value.independent_audit != audit
        }) {
            return Err("Stage 135 复核绑定或独立审计已经漂移".to_string());
        }
        let approved = review.as_ref().is_some_and(|value| {
            value.future_stage_136_isolated_materializer_specification_registration_eligible
        });
        items.push(
            OpeningPortfolioSnapshotMaterializationImplementationReviewItem {
                implementation,
                current_independent_audit: audit,
                review_eligible: review.is_none(),
                review,
                future_stage_136_isolated_materializer_specification_registration_eligible:
                    approved,
            },
        );
    }
    items.sort_by(|left, right| {
        right
            .implementation
            .registered_at
            .cmp(&left.implementation.registered_at)
    });
    let implementation_count = items.len();
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items.iter().filter(|item| item.review.is_some()).count();
    let independently_approved_count = items
        .iter()
        .filter(|item| {
            item.future_stage_136_isolated_materializer_specification_registration_eligible
        })
        .count();
    let changes_required_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.review.as_ref().is_some_and(|review| {
                review.verdict
                    != OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict::ApprovedForFutureIsolatedMaterializerSpecificationRegistration
            })
        })
        .count();
    let review_status = if independently_approved_count > 0 {
        "zero_capability_materialization_implementation_independently_approved_waiting_stage_136_isolated_materializer_specification"
    } else if changes_required_or_rejected_count > 0 {
        "materialization_implementation_review_terminal_rebuild_required"
    } else if review_eligible_count > 0 {
        "materialization_implementation_ready_for_stage_135_chain_external_independent_review"
    } else {
        "waiting_stage_134_zero_capability_materialization_implementation"
    };
    Ok(OpeningPortfolioSnapshotMaterializationImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        implementation_count,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_required_or_rejected_count,
        future_stage_136_isolated_materializer_specification_registration_eligible_count:
            independently_approved_count,
        review_status: review_status.to_string(),
        isolated_materializer_specification_registered: false,
        decryption_key_or_input_accessed: false,
        receipt_decrypted_or_read: false,
        parser_artifact_entrypoint_or_runtime_present: false,
        output_candidate_created: false,
        opening_portfolio_snapshot_present: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        position_or_cash_written: false,
        nav_or_performance_written: false,
        training_or_rl_feedback_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        next_gate: STAGE_135_NEXT_GATE.to_string(),
        scope: "Stage 135 由责任链外第二实现重建 Stage 134 期初快照物化合同，独立核对完整账户、精确十进制、证券身份、公司行动、逐行来源、整批失败和未受信输出分离。复核不取得密钥或输入，不读取或解密 receipt，不运行 parser/runtime，不生成候选或真实快照；批准只开放 Stage 136 隔离物化器规格登记。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewOpeningPortfolioSnapshotMaterializationImplementationRequest,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationReviewRecord, String> {
    validate_request(implementation_id, &request)?;
    let _lock = acquire_lock(state, implementation_id).await?;
    let source =
        independently_reviewable_opening_portfolio_snapshot_materialization_implementations(state)
            .await?
            .into_iter()
            .find(|value| value.implementation.implementation_id == implementation_id)
            .ok_or_else(|| "当前没有精确匹配且仍可复核的 Stage 134 物化实现".to_string())?;
    let implementation = source.implementation;
    if read_review(state, implementation_id).await?.is_some() {
        return Err("该 Stage 134 实现已经形成终态复核，禁止覆盖或重放".to_string());
    }
    let audit = independently_audit(&implementation)?;
    validate_expected_binding(&implementation, &audit, &request)?;
    let excluded_prior_actor_ids = expected_excluded_actor_ids(&implementation);
    let independent = !reviewer_id.trim().is_empty()
        && excluded_prior_actor_ids
            .iter()
            .all(|actor| actor != reviewer_id);
    if !independent
        || !request
            .confirmations
            .reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain_confirmed
    {
        return Err("Stage 135 reviewer 必须独立于 registrar、validator、executor、claimant 与完整既有责任链".to_string());
    }
    let confirmations_complete = confirmations_complete(&request.confirmations);
    let approved = request.verdict
        == OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict::ApprovedForFutureIsolatedMaterializerSpecificationRegistration;
    if approved && (!confirmations_complete || !audit.mismatch_reasons.is_empty()) {
        return Err("Stage 135 批准必须通过完整第二实现审计和全部逐项确认".to_string());
    }
    let mut review = OpeningPortfolioSnapshotMaterializationImplementationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        implementation,
        independent_audit: audit,
        submitted_at: Utc::now(),
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        binding_and_recomputation_assessment: bounded_required(
            &request.binding_and_recomputation_assessment,
            "绑定与重算评估",
        )?,
        parser_schema_and_completeness_assessment: bounded_required(
            &request.parser_schema_and_completeness_assessment,
            "解析/schema/完整性评估",
        )?,
        decimal_identity_and_provenance_assessment: bounded_required(
            &request.decimal_identity_and_provenance_assessment,
            "十进制/证券身份/来源评估",
        )?,
        failure_separation_and_zero_capability_assessment: bounded_required(
            &request.failure_separation_and_zero_capability_assessment,
            "失败关闭/分离/零能力评估",
        )?,
        known_limitations: bounded_required(&request.known_limitations, "已知限制")?,
        future_materializer_constraints: bounded_required(
            &request.future_materializer_constraints,
            "后续物化器约束",
        )?,
        confirmations: request.confirmations,
        confirmations_complete,
        reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain:
            independent,
        zero_capability_materialization_implementation_independently_approved: approved
            && confirmations_complete,
        future_stage_136_isolated_materializer_specification_registration_eligible: approved
            && confirmations_complete,
        isolated_materializer_specification_registered: false,
        decryption_key_or_input_accessed: false,
        receipt_decrypted_or_read: false,
        parser_artifact_entrypoint_or_runtime_present: false,
        output_candidate_created: false,
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
    review.review_sha256 = fingerprint_without(&review, &["review_id", "review_sha256"])?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review)?;
    write_immutable_json(
        &review_root(state)
            .join(implementation_id)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn independently_audit(
    implementation: &OpeningPortfolioSnapshotMaterializationImplementationRegistration,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit, String> {
    validate_opening_portfolio_snapshot_materialization_implementation_for_review(implementation)?;
    let rebuilt_contract = rebuild_contract_without_stage_134_builder(implementation)?;
    let contract = &implementation.implementation_contract;
    let implementation_record_hash_independently_reproduced = implementation.implementation_sha256
        == fingerprint_without(
            implementation,
            &["implementation_id", "implementation_sha256"],
        )?;
    let implementation_contract_hash_independently_reproduced =
        contract.contract_sha256 == fingerprint_without(contract, &["contract_sha256"])?;
    let complete_contract_rebuilt_without_stage_134_builder = true;
    let rebuilt_contract_exactly_matches_record = rebuilt_contract == *contract;
    let exact_current_stage_51_through_stage_134_binding_valid = contract.stage_133_validation_id
        == implementation.upstream_stage_133_validation.validation_id
        && contract.stage_133_validation_sha256
            == implementation
                .upstream_stage_133_validation
                .validation_sha256
        && contract.stage_132_result_sha256
            == implementation
                .upstream_stage_133_validation
                .stage_132_result_sha256
        && contract.stage_131_claim_sha256
            == implementation
                .upstream_stage_133_validation
                .stage_131_claim_sha256
        && contract.receipt_id == implementation.upstream_stage_133_validation.receipt_id
        && contract.receipt_manifest_sha256
            == implementation
                .upstream_stage_133_validation
                .receipt_manifest_sha256
        && contract.stage_125_specification_sha256
            == implementation
                .upstream_stage_133_validation
                .stage_125_specification_sha256;
    let all_stage_134_registration_confirmations_valid = implementation.confirmations_complete
        && serde_json::to_value(&implementation.confirmations)
            .ok()
            .and_then(|value| value.as_object().cloned())
            .is_some_and(|values| {
                values.len() == 18 && values.values().all(|value| value.as_bool() == Some(true))
            });
    let deterministic_adapter_and_resource_contract_valid = contract.accepted_input_formats
        == accepted_artifact_formats()
        && contract.future_input_artifact_count_limit == FUTURE_MAX_INPUT_ARTIFACTS
        && contract.future_input_byte_limit == FUTURE_MAX_INPUT_BYTES
        && contract.future_output_byte_limit == FUTURE_MAX_OUTPUT_BYTES
        && contract.future_output_row_limit == FUTURE_MAX_OUTPUT_ROWS
        && contract_function_ids(contract) == EXPECTED_FUNCTION_IDS;
    let schema = &contract.exact_canonical_snapshot_schema;
    let complete_financial_sections_and_whole_snapshot_failure_valid =
        !schema.account_schema.trim().is_empty()
            && !schema.cash_schema.trim().is_empty()
            && !schema.position_schema.trim().is_empty()
            && !schema.listed_option_extension_schema.trim().is_empty()
            && !schema.liability_schema.trim().is_empty()
            && !schema.unsettled_activity_schema.trim().is_empty()
            && !contract.partial_account_scope_allowed
            && !contract.unsupported_asset_silently_dropped
            && contract.whole_snapshot_fails_on_missing_ambiguous_unsupported_or_unreconciled_input;
    let exact_decimal_identity_corporate_action_and_provenance_valid = !contract
        .binary_floating_point_allowed
        && !contract.manual_balance_or_position_entry_allowed
        && !contract.missing_value_defaulting_allowed
        && !contract.cash_position_quantity_cost_basis_or_weight_inference_allowed
        && !schema.instrument_identity_precedence.is_empty()
        && !schema
            .corporate_action_reconciliation_rule
            .trim()
            .is_empty()
        && contract.every_output_row_requires_artifact_sha256_and_source_locator
        && !contract.raw_account_numbers_or_credentials_in_output_logs_or_errors_allowed;
    let untrusted_output_validation_admission_separation_valid = contract.output_create_once
        && contract.output_untrusted
        && contract.future_independent_output_validation_required
        && contract.future_snapshot_admission_review_required
        && contract.correction_requires_new_candidate
        && !contract.statement_market_value_used_as_accounting_mark;
    let all_key_input_parser_financial_model_order_broker_and_trading_authority_closed =
        contract.authority_boundary == closed_authority_boundary()
            && contract.registered_not_run
            && contract.future_independent_implementation_review_required;
    let mut mismatch_reasons = Vec::new();
    check(
        implementation_record_hash_independently_reproduced,
        "implementation_record_hash_mismatch",
        &mut mismatch_reasons,
    );
    check(
        implementation_contract_hash_independently_reproduced,
        "implementation_contract_hash_mismatch",
        &mut mismatch_reasons,
    );
    check(
        rebuilt_contract_exactly_matches_record,
        "independently_rebuilt_contract_mismatch",
        &mut mismatch_reasons,
    );
    check(
        exact_current_stage_51_through_stage_134_binding_valid,
        "stage_51_through_134_binding_invalid",
        &mut mismatch_reasons,
    );
    check(
        all_stage_134_registration_confirmations_valid,
        "stage_134_confirmations_incomplete",
        &mut mismatch_reasons,
    );
    check(
        deterministic_adapter_and_resource_contract_valid,
        "adapter_or_resource_contract_invalid",
        &mut mismatch_reasons,
    );
    check(
        complete_financial_sections_and_whole_snapshot_failure_valid,
        "financial_sections_or_whole_failure_contract_invalid",
        &mut mismatch_reasons,
    );
    check(
        exact_decimal_identity_corporate_action_and_provenance_valid,
        "decimal_identity_or_provenance_contract_invalid",
        &mut mismatch_reasons,
    );
    check(
        untrusted_output_validation_admission_separation_valid,
        "output_separation_contract_invalid",
        &mut mismatch_reasons,
    );
    check(
        all_key_input_parser_financial_model_order_broker_and_trading_authority_closed,
        "zero_capability_boundary_open",
        &mut mismatch_reasons,
    );
    let mut audit = OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        stage_133_validation_sha256: contract.stage_133_validation_sha256.clone(),
        stage_132_result_sha256: contract.stage_132_result_sha256.clone(),
        stage_131_claim_sha256: contract.stage_131_claim_sha256.clone(),
        receipt_manifest_sha256: contract.receipt_manifest_sha256.clone(),
        stage_125_specification_sha256: contract.stage_125_specification_sha256.clone(),
        implementation_record_hash_independently_reproduced,
        implementation_contract_hash_independently_reproduced,
        complete_contract_rebuilt_without_stage_134_builder,
        rebuilt_contract_exactly_matches_record,
        exact_current_stage_51_through_stage_134_binding_valid,
        all_stage_134_registration_confirmations_valid,
        deterministic_adapter_and_resource_contract_valid,
        complete_financial_sections_and_whole_snapshot_failure_valid,
        exact_decimal_identity_corporate_action_and_provenance_valid,
        untrusted_output_validation_admission_separation_valid,
        all_key_input_parser_financial_model_order_broker_and_trading_authority_closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn rebuild_contract_without_stage_134_builder(
    implementation: &OpeningPortfolioSnapshotMaterializationImplementationRegistration,
) -> Result<OpeningPortfolioSnapshotMaterializationImplementationContract, String> {
    let original = &implementation.implementation_contract;
    let mut contract = OpeningPortfolioSnapshotMaterializationImplementationContract {
        schema_version: CONTRACT_SCHEMA_VERSION.to_string(),
        contract_sha256: String::new(),
        protocol_version: PROTOCOL_VERSION.to_string(),
        immutable_code_revision: original.immutable_code_revision.clone(),
        stage_133_validation_id: implementation.upstream_stage_133_validation.validation_id.clone(),
        stage_133_validation_sha256: implementation.upstream_stage_133_validation.validation_sha256.clone(),
        stage_132_result_sha256: implementation.upstream_stage_133_validation.stage_132_result_sha256.clone(),
        stage_131_claim_sha256: implementation.upstream_stage_133_validation.stage_131_claim_sha256.clone(),
        receipt_id: implementation.upstream_stage_133_validation.receipt_id.clone(),
        receipt_manifest_sha256: implementation.upstream_stage_133_validation.receipt_manifest_sha256.clone(),
        stage_125_specification_sha256: implementation.upstream_stage_133_validation.stage_125_specification_sha256.clone(),
        exact_source_artifact_contract: original.exact_source_artifact_contract.clone(),
        exact_canonical_snapshot_schema: original.exact_canonical_snapshot_schema.clone(),
        accepted_input_formats: accepted_artifact_formats(),
        future_input_artifact_count_limit: FUTURE_MAX_INPUT_ARTIFACTS,
        future_input_byte_limit: FUTURE_MAX_INPUT_BYTES,
        future_output_byte_limit: FUTURE_MAX_OUTPUT_BYTES,
        future_output_row_limit: FUTURE_MAX_OUTPUT_ROWS,
        future_input_envelope_schema: "stage_133_validation_sha256,receipt_manifest_sha256,stage_125_specification_sha256,content_addressed_encrypted_artifacts,ephemeral_decryption_key_handle".to_string(),
        future_output_candidate_schema: "candidate_id,receipt_manifest_sha256,specification_sha256,portfolio_scope_alias,reporting_currency,source_timezone,snapshot_as_of_utc,accounts,cash,positions,listed_options,liabilities,unsettled_activity,row_provenance,completeness_proof,canonical_candidate_sha256,untrusted".to_string(),
        validate_receipt_and_specification_binding_function_id: EXPECTED_FUNCTION_IDS[0].to_string(),
        decrypt_in_ephemeral_memory_function_id: EXPECTED_FUNCTION_IDS[1].to_string(),
        parse_provider_pdf_csv_json_deterministically_function_id: EXPECTED_FUNCTION_IDS[2].to_string(),
        normalize_account_scope_function_id: EXPECTED_FUNCTION_IDS[3].to_string(),
        normalize_cash_positions_options_liabilities_and_unsettled_activity_function_id: EXPECTED_FUNCTION_IDS[4].to_string(),
        reconcile_instrument_identity_and_corporate_actions_function_id: EXPECTED_FUNCTION_IDS[5].to_string(),
        enforce_full_snapshot_completeness_function_id: EXPECTED_FUNCTION_IDS[6].to_string(),
        canonicalize_exact_decimal_output_function_id: EXPECTED_FUNCTION_IDS[7].to_string(),
        attach_source_row_provenance_function_id: EXPECTED_FUNCTION_IDS[8].to_string(),
        create_once_untrusted_candidate_function_id: EXPECTED_FUNCTION_IDS[9].to_string(),
        binary_floating_point_allowed: false,
        manual_balance_or_position_entry_allowed: false,
        missing_value_defaulting_allowed: false,
        cash_position_quantity_cost_basis_or_weight_inference_allowed: false,
        partial_account_scope_allowed: false,
        unsupported_asset_silently_dropped: false,
        statement_market_value_used_as_accounting_mark: false,
        raw_account_numbers_or_credentials_in_output_logs_or_errors_allowed: false,
        every_output_row_requires_artifact_sha256_and_source_locator: true,
        whole_snapshot_fails_on_missing_ambiguous_unsupported_or_unreconciled_input: true,
        output_create_once: true,
        output_untrusted: true,
        future_independent_output_validation_required: true,
        future_snapshot_admission_review_required: true,
        correction_requires_new_candidate: true,
        registered_not_run: true,
        future_independent_implementation_review_required: true,
        next_gate: STAGE_134_NEXT_GATE.to_string(),
        authority_boundary: closed_authority_boundary(),
    };
    contract.contract_sha256 = fingerprint_without(&contract, &["contract_sha256"])?;
    Ok(contract)
}

fn validate_review(
    review: &OpeningPortfolioSnapshotMaterializationImplementationReviewRecord,
) -> Result<(), String> {
    validate_opening_portfolio_snapshot_materialization_implementation_for_review(
        &review.implementation,
    )?;
    let expected_audit = independently_audit(&review.implementation)?;
    let approved = review.verdict
        == OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict::ApprovedForFutureIsolatedMaterializerSpecificationRegistration;
    let downstream_closed = !review.isolated_materializer_specification_registered
        && !review.decryption_key_or_input_accessed
        && !review.receipt_decrypted_or_read
        && !review.parser_artifact_entrypoint_or_runtime_present
        && !review.output_candidate_created
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
        && review.independent_audit == expected_audit
        && review.independent_audit.mismatch_reasons.is_empty()
        && review.excluded_prior_actor_ids == expected_excluded_actor_ids(&review.implementation)
        && !review.excluded_prior_actor_ids.contains(&review.reviewer_id)
        && review.reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain
        && review.confirmations_complete == confirmations_complete(&review.confirmations)
        && (!approved || review.confirmations_complete)
        && review.zero_capability_materialization_implementation_independently_approved
            == (approved && review.confirmations_complete)
        && review.future_stage_136_isolated_materializer_specification_registration_eligible
            == (approved && review.confirmations_complete)
        && !review.rationale.trim().is_empty()
        && !review.binding_and_recomputation_assessment.trim().is_empty()
        && !review.parser_schema_and_completeness_assessment.trim().is_empty()
        && !review.decimal_identity_and_provenance_assessment.trim().is_empty()
        && !review.failure_separation_and_zero_capability_assessment.trim().is_empty()
        && !review.known_limitations.trim().is_empty()
        && !review.future_materializer_constraints.trim().is_empty()
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 135 期初快照物化实现复核无效、漂移或越权".to_string())
}

fn validate_request(
    implementation_id: &str,
    request: &ReviewOpeningPortfolioSnapshotMaterializationImplementationRequest,
) -> Result<(), String> {
    let valid = valid_id(implementation_id)
        && [
            &request.expected_implementation_sha256,
            &request.expected_implementation_contract_sha256,
            &request.expected_stage_133_validation_sha256,
            &request.expected_stage_132_result_sha256,
            &request.expected_stage_131_claim_sha256,
            &request.expected_receipt_manifest_sha256,
            &request.expected_stage_125_specification_sha256,
            &request.expected_independent_audit_sha256,
        ]
        .into_iter()
        .all(|value| valid_sha256(value));
    valid
        .then_some(())
        .ok_or_else(|| "Stage 135 请求 ID 或摘要无效".to_string())
}

fn validate_expected_binding(
    implementation: &OpeningPortfolioSnapshotMaterializationImplementationRegistration,
    audit: &OpeningPortfolioSnapshotMaterializationImplementationIndependentAudit,
    request: &ReviewOpeningPortfolioSnapshotMaterializationImplementationRequest,
) -> Result<(), String> {
    let contract = &implementation.implementation_contract;
    let valid = request.expected_implementation_sha256 == implementation.implementation_sha256
        && request.expected_implementation_contract_sha256 == contract.contract_sha256
        && request.expected_stage_133_validation_sha256 == contract.stage_133_validation_sha256
        && request.expected_stage_132_result_sha256 == contract.stage_132_result_sha256
        && request.expected_stage_131_claim_sha256 == contract.stage_131_claim_sha256
        && request.expected_receipt_manifest_sha256 == contract.receipt_manifest_sha256
        && request.expected_stage_125_specification_sha256
            == contract.stage_125_specification_sha256
        && request.expected_independent_audit_sha256 == audit.audit_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 135 请求与当前 Stage 125/131/132/133/134 精确绑定不一致".to_string())
}

fn confirmations_complete(
    value: &OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations,
) -> bool {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .is_some_and(|values| {
            values.len() == 19 && values.values().all(|value| value.as_bool() == Some(true))
        })
}

fn expected_excluded_actor_ids(
    implementation: &OpeningPortfolioSnapshotMaterializationImplementationRegistration,
) -> Vec<String> {
    let mut values = implementation.excluded_prior_actor_ids.clone();
    values.push(implementation.registered_by.clone());
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn accepted_artifact_formats() -> Vec<String> {
    vec!["csv".to_string(), "json".to_string(), "pdf".to_string()]
}

fn contract_function_ids(
    contract: &OpeningPortfolioSnapshotMaterializationImplementationContract,
) -> [&str; 10] {
    [
        &contract.validate_receipt_and_specification_binding_function_id,
        &contract.decrypt_in_ephemeral_memory_function_id,
        &contract.parse_provider_pdf_csv_json_deterministically_function_id,
        &contract.normalize_account_scope_function_id,
        &contract.normalize_cash_positions_options_liabilities_and_unsettled_activity_function_id,
        &contract.reconcile_instrument_identity_and_corporate_actions_function_id,
        &contract.enforce_full_snapshot_completeness_function_id,
        &contract.canonicalize_exact_decimal_output_function_id,
        &contract.attach_source_row_provenance_function_id,
        &contract.create_once_untrusted_candidate_function_id,
    ]
}

fn closed_authority_boundary()
-> ZeroCapabilityOpeningPortfolioSnapshotMaterializationAuthorityBoundary {
    ZeroCapabilityOpeningPortfolioSnapshotMaterializationAuthorityBoundary {
        decryption_key_access_allowed: false,
        encrypted_artifact_read_allowed: false,
        plaintext_persistence_allowed: false,
        financial_row_parsing_allowed: false,
        executable_artifact_or_entrypoint_present: false,
        runtime_present: false,
        input_mount_present: false,
        output_candidate_present: false,
        opening_portfolio_snapshot_materialized: false,
        opening_portfolio_snapshot_admitted: false,
        financial_event_allowlist_nonempty: false,
        ledger_created: false,
        position_or_cash_write_allowed: false,
        nav_or_performance_write_allowed: false,
        model_or_metric_store_write_allowed: false,
        training_or_rl_feedback_allowed: false,
        reward_allowed: false,
        order_generation_allowed: false,
        broker_access_allowed: false,
        trading_allowed: false,
    }
}

fn check(value: bool, reason: &str, output: &mut Vec<String>) {
    if !value {
        output.push(reason.to_string());
    }
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!(
            "Stage 135 {label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"
        ))
    } else {
        Ok(value.to_string())
    }
}

fn fingerprint_without<T: Serialize>(value: &T, excluded: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Stage 135 指纹对象不是 JSON object".to_string())?;
    for key in excluded {
        object.remove(*key);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&json).map_err(|error| error.to_string())?)
    ))
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

async fn read_review(
    state: &AppState,
    implementation_id: &str,
) -> Result<Option<OpeningPortfolioSnapshotMaterializationImplementationReviewRecord>, String> {
    let directory = review_root(state).join(implementation_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let mut values = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            return Err("Stage 135 复核目录含非 JSON 文件".to_string());
        }
        let metadata = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() == 0
            || metadata.len() > MAX_RECORD_FILE_BYTES
        {
            return Err("Stage 135 复核文件无效或超限".to_string());
        }
        let review: OpeningPortfolioSnapshotMaterializationImplementationReviewRecord =
            serde_json::from_slice(
                &tokio::fs::read(&path)
                    .await
                    .map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
        validate_review(&review)?;
        if review.implementation.implementation_id != implementation_id
            || path.file_stem().and_then(|value| value.to_str()) != Some(review.review_id.as_str())
        {
            return Err("Stage 135 复核路径与绑定 ID 不一致".to_string());
        }
        values.push(review);
    }
    if values.len() > 1 {
        return Err("Stage 135 同一实现出现重复或分叉终态复核".to_string());
    }
    Ok(values.pop())
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    if bytes.is_empty() || bytes.len() as u64 > MAX_RECORD_FILE_BYTES {
        return Err("Stage 135 复核文件为空或超限".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "Stage 135 复核路径缺少父目录".to_string())?;
    tokio::fs::create_dir_all(parent)
        .await
        .map_err(|error| error.to_string())?;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "Stage 135 复核已存在，禁止覆盖".to_string()
            } else {
                error.to_string()
            }
        })?;
    file.write_all(&bytes)
        .await
        .map_err(|error| error.to_string())?;
    file.sync_all().await.map_err(|error| error.to_string())
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_root(state: &AppState) -> PathBuf {
    decision_root(state).join("opening-portfolio-snapshot-materialization-implementation-reviews")
}

struct ReviewLock(PathBuf);

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, implementation_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage135-{implementation_id}.lock"));
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 135 独立复核正在进行".to_string())?;
    Ok(ReviewLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn confirmations() -> OpeningPortfolioSnapshotMaterializationImplementationReviewConfirmations {
        serde_json::from_value(serde_json::json!({
            "exact_current_stage_51_through_stage_134_binding_confirmed": true,
            "reviewer_independent_from_registrar_validator_executor_claimant_and_complete_prior_chain_confirmed": true,
            "implementation_contract_validation_result_claim_receipt_and_specification_hashes_independently_reproduced_confirmed": true,
            "complete_contract_rebuilt_without_stage_134_builder_confirmed": true,
            "all_stage_134_registration_confirmations_revalidated_confirmed": true,
            "input_only_independently_validated_content_addressed_receipt_confirmed": true,
            "future_decryption_only_in_isolated_ephemeral_memory_confirmed": true,
            "deterministic_pdf_csv_json_adapters_and_no_remote_fetch_confirmed": true,
            "complete_accounts_cash_positions_options_liabilities_and_unsettled_activity_confirmed": true,
            "exact_decimal_strings_signed_quantities_and_no_binary_float_confirmed": true,
            "instrument_identity_precedence_and_corporate_action_reconciliation_confirmed": true,
            "no_default_manual_or_inferred_financial_values_and_whole_snapshot_failure_confirmed": true,
            "statement_market_values_informational_and_no_nav_or_performance_confirmed": true,
            "every_output_row_bound_to_artifact_hash_and_source_locator_with_redaction_confirmed": true,
            "output_create_once_untrusted_and_separate_validation_and_admission_confirmed": true,
            "no_key_input_read_decrypt_parse_artifact_entrypoint_runtime_mount_or_output_confirmed": true,
            "no_snapshot_financial_allowlist_ledger_position_cash_nav_performance_model_training_rl_reward_order_broker_or_trading_confirmed": true,
            "approval_only_opens_future_stage_136_isolated_materializer_specification_registration_confirmed": true,
            "no_unconfirmed_hari_or_old_wang_logic_claimed": true
        }))
        .expect("valid confirmations")
    }

    #[test]
    fn approval_requires_all_nineteen_independent_confirmations() {
        let value = confirmations();
        assert!(confirmations_complete(&value));
        assert_eq!(
            serde_json::to_value(value)
                .unwrap()
                .as_object()
                .unwrap()
                .len(),
            19
        );
    }

    #[test]
    fn independent_materialization_function_contract_is_frozen() {
        assert_eq!(EXPECTED_FUNCTION_IDS.len(), 10);
        assert!(
            EXPECTED_FUNCTION_IDS
                .iter()
                .all(|value| value.starts_with("opening_snapshot_"))
        );
    }

    #[test]
    fn exact_decimal_provenance_and_whole_snapshot_failure_are_required() {
        assert_eq!(accepted_artifact_formats(), vec!["csv", "json", "pdf"]);
        assert_eq!(FUTURE_MAX_INPUT_ARTIFACTS, 64);
        assert_eq!(FUTURE_MAX_INPUT_BYTES, 256 * 1024 * 1024);
        assert_eq!(FUTURE_MAX_OUTPUT_ROWS, 1_000_000);
    }

    #[test]
    fn independent_review_authority_boundary_is_fully_closed() {
        let value = closed_authority_boundary();
        assert!(!value.decryption_key_access_allowed);
        assert!(!value.encrypted_artifact_read_allowed);
        assert!(!value.financial_row_parsing_allowed);
        assert!(!value.output_candidate_present);
        assert!(!value.opening_portfolio_snapshot_materialized);
        assert!(!value.position_or_cash_write_allowed);
        assert!(!value.nav_or_performance_write_allowed);
        assert!(!value.training_or_rl_feedback_allowed);
        assert!(!value.order_generation_allowed);
        assert!(!value.broker_access_allowed);
        assert!(!value.trading_allowed);
    }

    #[test]
    fn only_explicit_approval_can_open_stage_136() {
        assert_ne!(
            OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict::ApprovedForFutureIsolatedMaterializerSpecificationRegistration,
            OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict::ChangesRequiredRebuildMaterializationImplementation
        );
        assert_ne!(
            OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict::ApprovedForFutureIsolatedMaterializerSpecificationRegistration,
            OpeningPortfolioSnapshotMaterializationImplementationReviewVerdict::RejectedMaterializationImplementation
        );
        assert!(STAGE_135_NEXT_GATE.contains("stage_136"));
    }
}
