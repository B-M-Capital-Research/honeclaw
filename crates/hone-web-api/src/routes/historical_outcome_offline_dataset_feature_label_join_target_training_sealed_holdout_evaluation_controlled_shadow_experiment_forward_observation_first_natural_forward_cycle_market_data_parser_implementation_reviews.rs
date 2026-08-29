//! Stage 98 chain-external review of one Stage 97 zero-capability market-data parser
//! implementation contract.
//!
//! Approval opens only a future isolated parser-runner specification registration gate. This
//! module has no parser source or executable artifact, entrypoint, runtime, raw-payload access,
//! parsed output, observation, ledger, position, performance, training, reward, order, broker or
//! trading capability.

use std::collections::{BTreeMap, BTreeSet};
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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_implementations::{
    ControlledShadowMarketDataParserImplementationRecord,
    ZeroCapabilityMarketDataParserImplementationAuthorityBoundary,
    independently_reviewable_market_data_parser_implementations,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-implementation-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-implementation-independent-review-v1";
const AUDIT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-implementation-independent-audit-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-implementation-chain-external-review-v1-no-parser";
const STAGE_97_RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-implementation-v1";
const STAGE_97_POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-implementation-create-once-v1-zero-capability";
const STAGE_97_CONTRACT_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-zero-capability-market-data-parser-implementation-contract-v1";
const STAGE_97_IMPLEMENTATION_PROTOCOL_VERSION: &str =
    "hone-natural-forward-market-data-parser-implementation-v1-not-executable";
const STAGE_97_STATUS: &str = "registered_not_reviewed_not_run";
const MAX_TEXT_CHARS: usize = 4_000;

const EXPECTED_FUNCTION_IDS: [&str; 8] = [
    "market_data_parser_strict_envelope_dispatch_v1_pure_specification",
    "market_data_parser_fmp_price_array_v1_pure_specification",
    "market_data_parser_fmp_dividend_events_v1_pure_specification",
    "market_data_parser_fmp_split_events_v1_pure_specification",
    "market_data_parser_nyse_calendar_table_v1_pure_specification",
    "market_data_parser_calendar_subject_spy_reconciliation_v1_pure_specification",
    "market_data_parser_canonical_row_serialization_hash_v1_pure_specification",
    "market_data_parser_eight_synthetic_vector_conformance_v1_pure_specification",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowMarketDataParserImplementationReviewVerdict {
    ApprovedForFutureIsolatedMarketDataParserRunnerSpecificationRegistration,
    ChangesRequiredRebuildMarketDataParserImplementationContract,
    RejectedMarketDataParserImplementationContract,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowMarketDataParserImplementationRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_implementation_sha256: String,
    expected_implementation_contract_sha256: String,
    expected_specification_review_sha256: String,
    expected_specification_registration_sha256: String,
    expected_parser_specification_sha256: String,
    expected_independent_audit_sha256: String,
    verdict: ControlledShadowMarketDataParserImplementationReviewVerdict,
    rationale: String,
    binding_and_recomputation_assessment: String,
    deterministic_parser_semantics_assessment: String,
    source_schema_calendar_action_and_reconciliation_assessment: String,
    failure_and_missing_data_assessment: String,
    zero_capability_assessment: String,
    known_limitations: String,
    future_runner_constraints: String,
    exact_current_stage_51_through_stage_97_binding_confirmed: bool,
    reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed:
        bool,
    all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: bool,
    explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed: bool,
    strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: bool,
    duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: bool,
    no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
        bool,
    spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed: bool,
    all_eight_synthetic_vectors_independently_reconstructed_confirmed: bool,
    source_available_at_remains_unverified_until_separate_evidence_confirmed: bool,
    future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed:
        bool,
    no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed: bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserImplementationIndependentAudit {
    pub schema_version: String,
    pub audit_sha256: String,
    pub implementation_id: String,
    pub implementation_sha256: String,
    pub implementation_contract_sha256: String,
    pub specification_review_sha256: String,
    pub specification_registration_sha256: String,
    pub parser_specification_sha256: String,
    pub implementation_record_hash_independently_reproduced: bool,
    pub implementation_contract_hash_independently_reproduced: bool,
    pub specification_review_hash_independently_reproduced: bool,
    pub specification_registration_hash_independently_reproduced: bool,
    pub parser_specification_hash_independently_reproduced: bool,
    pub exact_current_stage_51_through_stage_97_binding_valid: bool,
    pub eight_function_ids_and_canonical_schemas_valid: bool,
    pub explicit_source_calendar_action_and_reconciliation_contract_valid: bool,
    pub strict_schema_numeric_failure_and_missing_data_contract_valid: bool,
    pub eight_synthetic_vectors_bound_and_synthetic_only: bool,
    pub source_available_at_still_unverified: bool,
    pub all_artifact_runtime_raw_payload_store_feedback_order_broker_and_trading_authority_closed:
        bool,
    pub mismatch_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserImplementationReviewRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub implementation: ControlledShadowMarketDataParserImplementationRecord,
    pub independent_audit: ControlledShadowMarketDataParserImplementationIndependentAudit,
    pub submitted_at: DateTime<Utc>,
    pub reviewer_id: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowMarketDataParserImplementationReviewVerdict,
    pub rationale: String,
    pub binding_and_recomputation_assessment: String,
    pub deterministic_parser_semantics_assessment: String,
    pub source_schema_calendar_action_and_reconciliation_assessment: String,
    pub failure_and_missing_data_assessment: String,
    pub zero_capability_assessment: String,
    pub known_limitations: String,
    pub future_runner_constraints: String,
    pub reviewer_independent_from_registrar_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_97_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: bool,
    pub implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed:
        bool,
    pub all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: bool,
    pub explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed: bool,
    pub strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: bool,
    pub duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: bool,
    pub no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
        bool,
    pub spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed: bool,
    pub all_eight_synthetic_vectors_independently_reconstructed_confirmed: bool,
    pub source_available_at_remains_unverified_until_separate_evidence_confirmed: bool,
    pub future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
        bool,
    pub no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed:
        bool,
    pub no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub zero_capability_implementation_independently_approved: bool,
    pub future_isolated_parser_runner_specification_registration_eligible: bool,
    pub isolated_runner_registered: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub raw_payload_accessed: bool,
    pub parsed_calendar_rows_created: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_store_written: bool,
    pub metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserImplementationReviewItem {
    pub implementation: ControlledShadowMarketDataParserImplementationRecord,
    pub current_independent_audit: ControlledShadowMarketDataParserImplementationIndependentAudit,
    pub complete_review_actor_ids: Vec<String>,
    pub latest_review: Option<ControlledShadowMarketDataParserImplementationReviewRecord>,
    pub review_eligible: bool,
    pub future_isolated_parser_runner_specification_registration_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserImplementationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_endpoint_available: bool,
    pub items: Vec<ControlledShadowMarketDataParserImplementationReviewItem>,
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_isolated_parser_runner_specification_registration_eligible_count: usize,
    pub review_status: String,
    pub isolated_runner_registered: bool,
    pub source_artifact_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub raw_payload_accessed: bool,
    pub parsed_calendar_rows_created: bool,
    pub parsed_market_rows_created: bool,
    pub forward_observation_started: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowMarketDataParserImplementationReviewReadinessSummary {
    pub implementation_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_isolated_parser_runner_specification_registration_eligible_count: usize,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndependentlyApprovedControlledShadowMarketDataParserImplementation {
    pub implementation: ControlledShadowMarketDataParserImplementationRecord,
    pub review: ControlledShadowMarketDataParserImplementationReviewRecord,
}

pub(crate) async fn handle_get_controlled_shadow_market_data_parser_implementation_reviews(
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
            warn!(%error, "market-data parser implementation independent review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "行情解析器零能力实现独立复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_market_data_parser_implementation(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(implementation_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowMarketDataParserImplementationRequest>,
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

pub(crate) async fn controlled_shadow_market_data_parser_implementation_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserImplementationReviewReadinessSummary, String> {
    let value = build_registry(state).await?;
    Ok(
        ControlledShadowMarketDataParserImplementationReviewReadinessSummary {
            implementation_count: value.implementation_count,
            review_eligible_count: value.review_eligible_count,
            reviewed_count: value.reviewed_count,
            independently_approved_count: value.independently_approved_count,
            changes_required_or_rejected_count: value.changes_required_or_rejected_count,
            future_isolated_parser_runner_specification_registration_eligible_count: value
                .future_isolated_parser_runner_specification_registration_eligible_count,
            review_status: value.review_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_approved_market_data_parser_implementations_for_isolated_runner_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyApprovedControlledShadowMarketDataParserImplementation>, String> {
    let registry = build_registry(state).await?;
    Ok(registry
        .items
        .into_iter()
        .filter_map(|item| {
            let review = item.latest_review?;
            item.future_isolated_parser_runner_specification_registration_eligible
                .then_some(
                    IndependentlyApprovedControlledShadowMarketDataParserImplementation {
                        implementation: item.implementation,
                        review,
                    },
                )
        })
        .collect())
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserImplementationReviewRegistry, String> {
    let implementations =
        independently_reviewable_market_data_parser_implementations(state).await?;
    let mut items = Vec::new();
    for implementation in implementations {
        let audit = independently_audit(&implementation)?;
        if !audit.mismatch_reasons.is_empty() {
            return Err("Stage 97 parser 实现独立审计失败，晋级已关闭".to_string());
        }
        let reviews = read_reviews(state, &implementation.implementation_id).await?;
        let latest_review = reviews.last().cloned();
        if latest_review.as_ref().is_some_and(|review| {
            review.implementation != implementation || review.independent_audit != audit
        }) {
            return Err("Stage 98 复核绑定或独立审计已经变化".to_string());
        }
        let approved = latest_review.as_ref().is_some_and(|review| {
            review.future_isolated_parser_runner_specification_registration_eligible
        });
        items.push(ControlledShadowMarketDataParserImplementationReviewItem {
            implementation,
            current_independent_audit: audit,
            complete_review_actor_ids: reviews
                .iter()
                .map(|review| review.reviewer_id.clone())
                .collect(),
            latest_review,
            review_eligible: !approved,
            future_isolated_parser_runner_specification_registration_eligible: approved,
        });
    }
    items.sort_by(|left, right| {
        right
            .implementation
            .registered_at
            .cmp(&left.implementation.registered_at)
    });
    let implementation_count = items.len();
    let review_eligible_count = items.iter().filter(|item| item.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|item| item.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|item| item.future_isolated_parser_runner_specification_registration_eligible)
        .count();
    let changes_required_or_rejected_count = items
        .iter()
        .filter(|item| {
            item.latest_review.as_ref().is_some_and(|review| {
                !review.future_isolated_parser_runner_specification_registration_eligible
            })
        })
        .count();
    let review_status = if items.is_empty() {
        "waiting_stage_97_zero_capability_market_data_parser_implementation_contract"
    } else if independently_approved_count > 0 {
        "market_data_parser_implementation_independently_approved_only_for_future_isolated_runner_specification_registration"
    } else if review_eligible_count > 0 {
        "market_data_parser_implementation_ready_for_chain_external_review"
    } else {
        "market_data_parser_implementation_review_blocked_fail_closed"
    };
    Ok(ControlledShadowMarketDataParserImplementationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_endpoint_available: true,
        items,
        implementation_count,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_required_or_rejected_count,
        future_isolated_parser_runner_specification_registration_eligible_count:
            independently_approved_count,
        review_status: review_status.to_string(),
        isolated_runner_registered: false,
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        raw_payload_accessed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 98 由 Stage 51–97 完整责任链之外的新角色，以第二实现独立重算 Stage 97 implementation/contract、Stage 96 review、Stage 95 registration/specification 指纹，并复核八个纯函数标识、canonical schema、显式来源/公司行动/日历、严格失败关闭、八组合成向量和全部零权限位。批准只开放未来 Stage 99 隔离 parser runner 规格登记；不提交工件、不读取载荷、不解析行情、不开始观察、不建账、不写持仓/绩效、不训练、不奖励、不下单、不接券商、不交易。".to_string(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    implementation_id: &str,
    request: ReviewControlledShadowMarketDataParserImplementationRequest,
) -> Result<ControlledShadowMarketDataParserImplementationReviewRecord, String> {
    if reviewer_id.trim().is_empty() || !valid_id(implementation_id) {
        return Err("Stage 98 复核人或实现 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, implementation_id).await?;
    let implementation = independently_reviewable_market_data_parser_implementations(state)
        .await?
        .into_iter()
        .find(|value| value.implementation_id == implementation_id)
        .ok_or_else(|| "当前 Stage 97 parser 实现不存在或上游绑定失效".to_string())?;
    let reviews = read_reviews(state, implementation_id).await?;
    let prior = reviews.last();
    if prior.is_some_and(|review| {
        review.future_isolated_parser_runner_specification_registration_eligible
    }) {
        return Err("当前 Stage 97 parser 实现已经独立批准，不得追加或分叉复核".to_string());
    }
    let review = build_review(
        &implementation,
        &reviews,
        prior,
        reviewer_id,
        request,
        Utc::now(),
    )?;
    write_immutable_json(
        &review_root(state)
            .join(implementation_id)
            .join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn build_review(
    implementation: &ControlledShadowMarketDataParserImplementationRecord,
    prior_reviews: &[ControlledShadowMarketDataParserImplementationReviewRecord],
    prior: Option<&ControlledShadowMarketDataParserImplementationReviewRecord>,
    reviewer_id: &str,
    request: ReviewControlledShadowMarketDataParserImplementationRequest,
    submitted_at: DateTime<Utc>,
) -> Result<ControlledShadowMarketDataParserImplementationReviewRecord, String> {
    let audit = independently_audit(implementation)?;
    if !audit.mismatch_reasons.is_empty() {
        return Err("Stage 97 parser 实现独立审计不通过".to_string());
    }
    let contract = &implementation.implementation_contract;
    let specification_review = &implementation.upstream_specification_review;
    let registration = &implementation.upstream_specification_registration;
    let specification = &registration.parser_specification;
    if request.expected_previous_review_id != prior.map(|value| value.review_id.clone())
        || request.expected_previous_review_sha256 != prior.map(|value| value.review_sha256.clone())
        || request.expected_implementation_sha256 != implementation.implementation_sha256
        || request.expected_implementation_contract_sha256 != contract.contract_sha256
        || request.expected_specification_review_sha256 != specification_review.review_sha256
        || request.expected_specification_registration_sha256 != registration.registration_sha256
        || request.expected_parser_specification_sha256 != specification.parser_specification_sha256
        || request.expected_independent_audit_sha256 != audit.audit_sha256
    {
        return Err("Stage 95–97 实现、规格、审计或复核链已变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = expected_excluded_actor_ids(implementation, prior_reviews);
    if excluded_prior_actor_ids
        .iter()
        .any(|actor| actor == reviewer_id)
    {
        return Err("复核人参与过 Stage 51–97 或此前复核链，必须更换责任链外角色".to_string());
    }
    let approved = request.verdict
        == ControlledShadowMarketDataParserImplementationReviewVerdict::ApprovedForFutureIsolatedMarketDataParserRunnerSpecificationRegistration;
    let confirmations_complete = request_checks(&request);
    if approved && !confirmations_complete {
        return Err("批准前必须逐项确认独立重算、确定性解析语义、来源对账和零权限边界".to_string());
    }
    let mut review = ControlledShadowMarketDataParserImplementationReviewRecord {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: prior.map(|value| value.review_id.clone()),
        previous_review_sha256: prior.map(|value| value.review_sha256.clone()),
        implementation: implementation.clone(),
        independent_audit: audit,
        submitted_at,
        reviewer_id: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        binding_and_recomputation_assessment: bounded_required(
            &request.binding_and_recomputation_assessment,
            "绑定与独立重算评估",
        )?,
        deterministic_parser_semantics_assessment: bounded_required(
            &request.deterministic_parser_semantics_assessment,
            "确定性解析语义评估",
        )?,
        source_schema_calendar_action_and_reconciliation_assessment: bounded_required(
            &request.source_schema_calendar_action_and_reconciliation_assessment,
            "来源、schema、日历、公司行动与对账评估",
        )?,
        failure_and_missing_data_assessment: bounded_required(
            &request.failure_and_missing_data_assessment,
            "失败与缺失数据评估",
        )?,
        zero_capability_assessment: bounded_required(
            &request.zero_capability_assessment,
            "零能力评估",
        )?,
        known_limitations: bounded_required(&request.known_limitations, "已知限制")?,
        future_runner_constraints: bounded_required(
            &request.future_runner_constraints,
            "未来 runner 约束",
        )?,
        reviewer_independent_from_registrar_and_complete_prior_chain: true,
        exact_current_stage_51_through_stage_97_binding_confirmed: request
            .exact_current_stage_51_through_stage_97_binding_confirmed,
        reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: request
            .reviewer_independent_from_registrar_and_complete_prior_chain_confirmed,
        implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed:
            request.implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed,
        all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: request
            .all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed,
        explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed: request
            .explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed,
        strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: request
            .strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed,
        duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: request
            .duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed,
        no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed:
            request.no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed,
        spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed:
            request.spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed,
        all_eight_synthetic_vectors_independently_reconstructed_confirmed: request
            .all_eight_synthetic_vectors_independently_reconstructed_confirmed,
        source_available_at_remains_unverified_until_separate_evidence_confirmed: request
            .source_available_at_remains_unverified_until_separate_evidence_confirmed,
        future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed:
            request.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed,
        no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed:
            request.no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed,
        no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
            request.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed:
            request.approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        zero_capability_implementation_independently_approved: approved,
        future_isolated_parser_runner_specification_registration_eligible: approved,
        isolated_runner_registered: false,
        source_artifact_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        raw_payload_accessed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_store_written: false,
        metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review)?;
    Ok(review)
}

fn independently_audit(
    implementation: &ControlledShadowMarketDataParserImplementationRecord,
) -> Result<ControlledShadowMarketDataParserImplementationIndependentAudit, String> {
    let contract = &implementation.implementation_contract;
    let review = &implementation.upstream_specification_review;
    let registration = &implementation.upstream_specification_registration;
    let specification = &registration.parser_specification;
    let implementation_hash = fingerprint_without(
        implementation,
        &["implementation_id", "implementation_sha256"],
    )?;
    let contract_hash = fingerprint_without(contract, &["contract_sha256"])?;
    let review_hash = fingerprint_without(review, &["review_id", "review_sha256"])?;
    let registration_hash =
        fingerprint_without(registration, &["registration_id", "registration_sha256"])?;
    let specification_hash = fingerprint_without(specification, &["parser_specification_sha256"])?;

    let implementation_record_hash_independently_reproduced = implementation_hash
        == implementation.implementation_sha256
        && implementation.implementation_id == implementation_hash[..32];
    let implementation_contract_hash_independently_reproduced =
        contract_hash == contract.contract_sha256;
    let specification_review_hash_independently_reproduced =
        review_hash == review.review_sha256 && review.review_id == review_hash[..32];
    let specification_registration_hash_independently_reproduced = registration_hash
        == registration.registration_sha256
        && registration.registration_id == registration_hash[..32];
    let parser_specification_hash_independently_reproduced =
        specification_hash == specification.parser_specification_sha256;

    let exact_current_stage_51_through_stage_97_binding_valid = implementation.schema_version
        == STAGE_97_RECORD_SCHEMA_VERSION
        && implementation.policy_version == STAGE_97_POLICY_VERSION
        && implementation.status == STAGE_97_STATUS
        && implementation.confirmations_complete
        && implementation.zero_capability_parser_implementation_contract_registered
        && implementation.future_independent_implementation_review_eligible
        && implementation.registrar_independent_from_stage_96_and_complete_prior_chain
        && !implementation
            .excluded_prior_actor_ids
            .contains(&implementation.registered_by)
        && contract.schema_version == STAGE_97_CONTRACT_SCHEMA_VERSION
        && contract.implementation_protocol_version == STAGE_97_IMPLEMENTATION_PROTOCOL_VERSION
        && contract.stage_96_specification_review_id == review.review_id
        && contract.stage_96_specification_review_sha256 == review.review_sha256
        && contract.stage_95_registration_id == registration.registration_id
        && contract.stage_95_registration_sha256 == registration.registration_sha256
        && contract.parser_specification_sha256 == specification.parser_specification_sha256
        && contract.exact_parser_specification == *specification
        && implementation_record_hash_independently_reproduced
        && implementation_contract_hash_independently_reproduced
        && specification_review_hash_independently_reproduced
        && specification_registration_hash_independently_reproduced
        && parser_specification_hash_independently_reproduced;

    let function_ids = [
        contract.strict_envelope_dispatch_function_id.as_str(),
        contract.fmp_price_array_parser_function_id.as_str(),
        contract.fmp_dividend_event_parser_function_id.as_str(),
        contract.fmp_split_event_parser_function_id.as_str(),
        contract.nyse_calendar_table_parser_function_id.as_str(),
        contract
            .calendar_subject_spy_reconciliation_function_id
            .as_str(),
        contract
            .canonical_row_serialization_and_hash_function_id
            .as_str(),
        contract.synthetic_vector_conformance_function_id.as_str(),
    ];
    let eight_function_ids_and_canonical_schemas_valid = function_ids == EXPECTED_FUNCTION_IDS
        && contract.canonical_calendar_row_schema == specification.canonical_calendar_row_schema
        && contract.canonical_price_row_schema == specification.canonical_price_row_schema
        && contract.canonical_dividend_row_schema == specification.canonical_dividend_row_schema
        && contract.canonical_split_row_schema == specification.canonical_split_row_schema
        && contract.canonical_parse_result_schema == specification.canonical_parse_result_schema;

    let explicit_source_calendar_action_and_reconciliation_contract_valid =
        specification.accepted_source_kinds.len() >= 6
            && specification
                .external_reference_urls
                .iter()
                .all(|url| url.starts_with("https://"))
            && specification.spy_required_for_every_official_session
            && specification.subject_missing_session_is_explicit_gap_not_synthetic_row
            && specification.separate_price_series_never_silently_substituted
            && specification.explicit_actions_required_for_action_adjustment_claims;
    let strict_schema_numeric_failure_and_missing_data_contract_valid = specification
        .strict_utf8_required
        && specification.fmp_top_level_array_required
        && specification.nyse_server_rendered_table_required
        && specification.iso_date_only_required
        && specification.finite_positive_price_required
        && specification.non_negative_volume_required
        && specification.duplicate_rows_rejected_not_deduplicated
        && specification.out_of_window_rows_rejected
        && !specification.empty_price_series_allowed
        && specification.empty_dividend_or_split_event_series_allowed
        && !specification.forward_fill_allowed
        && !specification.interpolation_allowed
        && !specification.unadjusted_close_fallback_allowed
        && !specification.inferred_dividend_or_split_allowed;
    let eight_synthetic_vectors_bound_and_synthetic_only =
        synthetic_vectors_valid(&specification.synthetic_test_vectors);
    let source_available_at_still_unverified =
        specification.source_available_at_remains_unverified_until_separate_review;
    let all_artifact_runtime_raw_payload_store_feedback_order_broker_and_trading_authority_closed =
        contract.registered_not_run
            && contract.independent_implementation_review_required
            && contract.isolated_runner_registration_required_after_review
            && contract.future_raw_payload_input_read_only_and_content_addressed
            && contract.future_parser_output_create_once_and_untrusted
            && contract.future_parser_output_independent_validation_required
            && !contract.future_parser_output_contains_market_interpretation
            && !contract.future_parser_output_contains_order_intent
            && authority_closed(&contract.authority_boundary)
            && implementation_downstream_closed(implementation);

    let checks = [
        (
            implementation_record_hash_independently_reproduced,
            "implementation record hash mismatch",
        ),
        (
            implementation_contract_hash_independently_reproduced,
            "implementation contract hash mismatch",
        ),
        (
            specification_review_hash_independently_reproduced,
            "Stage 96 review hash mismatch",
        ),
        (
            specification_registration_hash_independently_reproduced,
            "Stage 95 registration hash mismatch",
        ),
        (
            parser_specification_hash_independently_reproduced,
            "parser specification hash mismatch",
        ),
        (
            exact_current_stage_51_through_stage_97_binding_valid,
            "Stage 51-97 binding invalid",
        ),
        (
            eight_function_ids_and_canonical_schemas_valid,
            "function ids or canonical schemas invalid",
        ),
        (
            explicit_source_calendar_action_and_reconciliation_contract_valid,
            "source/calendar/action/reconciliation contract invalid",
        ),
        (
            strict_schema_numeric_failure_and_missing_data_contract_valid,
            "schema/numeric/failure contract invalid",
        ),
        (
            eight_synthetic_vectors_bound_and_synthetic_only,
            "synthetic vector contract invalid",
        ),
        (
            source_available_at_still_unverified,
            "source_available_at incorrectly claimed verified",
        ),
        (
            all_artifact_runtime_raw_payload_store_feedback_order_broker_and_trading_authority_closed,
            "zero-capability boundary is open",
        ),
    ];
    let mismatch_reasons = checks
        .iter()
        .filter_map(|(passed, reason)| (!passed).then_some((*reason).to_string()))
        .collect::<Vec<_>>();
    let mut audit = ControlledShadowMarketDataParserImplementationIndependentAudit {
        schema_version: AUDIT_SCHEMA_VERSION.to_string(),
        audit_sha256: String::new(),
        implementation_id: implementation.implementation_id.clone(),
        implementation_sha256: implementation.implementation_sha256.clone(),
        implementation_contract_sha256: contract.contract_sha256.clone(),
        specification_review_sha256: review.review_sha256.clone(),
        specification_registration_sha256: registration.registration_sha256.clone(),
        parser_specification_sha256: specification.parser_specification_sha256.clone(),
        implementation_record_hash_independently_reproduced,
        implementation_contract_hash_independently_reproduced,
        specification_review_hash_independently_reproduced,
        specification_registration_hash_independently_reproduced,
        parser_specification_hash_independently_reproduced,
        exact_current_stage_51_through_stage_97_binding_valid,
        eight_function_ids_and_canonical_schemas_valid,
        explicit_source_calendar_action_and_reconciliation_contract_valid,
        strict_schema_numeric_failure_and_missing_data_contract_valid,
        eight_synthetic_vectors_bound_and_synthetic_only,
        source_available_at_still_unverified,
        all_artifact_runtime_raw_payload_store_feedback_order_broker_and_trading_authority_closed,
        mismatch_reasons,
    };
    audit.audit_sha256 = fingerprint_without(&audit, &["audit_sha256"])?;
    Ok(audit)
}

fn synthetic_vectors_valid(
    vectors: &[super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_specifications::SyntheticMarketDataParserTestVector],
) -> bool {
    if vectors.len() != 8 {
        return false;
    }
    let mut ids = BTreeSet::new();
    let mut input_hashes = BTreeSet::new();
    vectors.iter().all(|vector| {
        vector.synthetic_only_no_market_truth
            && !vector.vector_id.trim().is_empty()
            && ids.insert(vector.vector_id.clone())
            && valid_sha256(&vector.input_fixture_sha256)
            && input_hashes.insert(vector.input_fixture_sha256.clone())
            && vector
                .expected_canonical_output_sha256
                .as_ref()
                .is_none_or(|hash| valid_sha256(hash))
    })
}

fn authority_closed(value: &ZeroCapabilityMarketDataParserImplementationAuthorityBoundary) -> bool {
    !value.source_artifact_present
        && !value.executable_artifact_present
        && !value.callable_entrypoint_present
        && !value.runtime_present
        && !value.raw_payload_mount_present
        && !value.raw_payload_read_allowed
        && !value.environment_inheritance_allowed
        && !value.secrets_allowed
        && !value.outbound_network_allowed
        && !value.tools_allowed
        && !value.subprocesses_allowed
        && !value.production_reads_allowed
        && !value.production_writes_allowed
        && !value.calendar_rows_created
        && !value.market_rows_created
        && !value.parser_output_written
        && !value.observation_write_allowed
        && !value.ledger_creation_allowed
        && !value.position_write_allowed
        && !value.performance_metric_write_allowed
        && !value.model_store_write_allowed
        && !value.metric_store_write_allowed
        && !value.training_feedback_allowed
        && !value.reward_allowed
        && !value.order_generation_allowed
        && !value.broker_access_allowed
        && !value.trading_allowed
}

fn implementation_downstream_closed(
    value: &ControlledShadowMarketDataParserImplementationRecord,
) -> bool {
    !value.parser_implementation_present
        && !value.independent_implementation_review_completed
        && !value.isolated_runner_registration_eligible
        && !value.parsed_calendar_rows_created
        && !value.parsed_market_rows_created
        && !value.forward_observation_started
        && !value.ledger_created
        && !value.position_written
        && !value.performance_metric_written
        && !value.model_store_written
        && !value.metric_store_written
        && !value.training_feedback_authorized
        && !value.reward_authorized
        && !value.order_generation_authorized
        && !value.broker_access_authorized
        && !value.trading_authorized
}

fn request_checks(request: &ReviewControlledShadowMarketDataParserImplementationRequest) -> bool {
    request.exact_current_stage_51_through_stage_97_binding_confirmed
        && request.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && request.implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed
        && request.all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed
        && request.explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed
        && request.strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed
        && request.duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed
        && request.no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed
        && request.spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed
        && request.all_eight_synthetic_vectors_independently_reconstructed_confirmed
        && request.source_available_at_remains_unverified_until_separate_evidence_confirmed
        && request.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && request.no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed
        && request.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && request.approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed
        && request.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn review_checks(value: &ControlledShadowMarketDataParserImplementationReviewRecord) -> bool {
    value.exact_current_stage_51_through_stage_97_binding_confirmed
        && value.reviewer_independent_from_registrar_and_complete_prior_chain_confirmed
        && value.implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed
        && value.all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed
        && value.explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed
        && value.strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed
        && value.duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed
        && value.no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed
        && value.spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed
        && value.all_eight_synthetic_vectors_independently_reconstructed_confirmed
        && value.source_available_at_remains_unverified_until_separate_evidence_confirmed
        && value.future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed
        && value.no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed
        && value.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && value.approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_review(
    review: &ControlledShadowMarketDataParserImplementationReviewRecord,
) -> Result<(), String> {
    let expected_audit = independently_audit(&review.implementation)?;
    let approved = review.verdict
        == ControlledShadowMarketDataParserImplementationReviewVerdict::ApprovedForFutureIsolatedMarketDataParserRunnerSpecificationRegistration;
    let no_downstream = !review.isolated_runner_registered
        && !review.source_artifact_present
        && !review.executable_artifact_present
        && !review.callable_entrypoint_present
        && !review.runtime_present
        && !review.raw_payload_accessed
        && !review.parsed_calendar_rows_created
        && !review.parsed_market_rows_created
        && !review.forward_observation_started
        && !review.ledger_created
        && !review.position_written
        && !review.performance_metric_written
        && !review.model_store_written
        && !review.metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let texts_valid = [
        &review.rationale,
        &review.binding_and_recomputation_assessment,
        &review.deterministic_parser_semantics_assessment,
        &review.source_schema_calendar_action_and_reconciliation_assessment,
        &review.failure_and_missing_data_assessment,
        &review.zero_capability_assessment,
        &review.known_limitations,
        &review.future_runner_constraints,
    ]
    .iter()
    .all(|value| !value.trim().is_empty() && value.chars().count() <= MAX_TEXT_CHARS);
    let valid = review.schema_version == REVIEW_SCHEMA_VERSION
        && review.policy_version == POLICY_VERSION
        && valid_id(&review.review_id)
        && valid_sha256(&review.review_sha256)
        && review.review_id == review.review_sha256[..32]
        && review.review_sha256 == review_fingerprint(review)?
        && review.independent_audit == expected_audit
        && review.independent_audit.mismatch_reasons.is_empty()
        && sorted_unique(&review.excluded_prior_actor_ids)
        && !review
            .excluded_prior_actor_ids
            .contains(&review.reviewer_id)
        && review
            .excluded_prior_actor_ids
            .contains(&review.implementation.registered_by)
        && review.reviewer_independent_from_registrar_and_complete_prior_chain
        && texts_valid
        && (!approved || review_checks(review))
        && review.zero_capability_implementation_independently_approved == approved
        && review.future_isolated_parser_runner_specification_registration_eligible == approved
        && no_downstream;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 98 parser 实现独立复核无效、漂移或越权".to_string())
}

fn expected_excluded_actor_ids(
    implementation: &ControlledShadowMarketDataParserImplementationRecord,
    prior_reviews: &[ControlledShadowMarketDataParserImplementationReviewRecord],
) -> Vec<String> {
    let mut actors = implementation.excluded_prior_actor_ids.clone();
    actors.push(implementation.registered_by.clone());
    actors.extend(
        prior_reviews
            .iter()
            .map(|review| review.reviewer_id.clone()),
    );
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn review_fingerprint(
    value: &ControlledShadowMarketDataParserImplementationReviewRecord,
) -> Result<String, String> {
    fingerprint_without(value, &["review_id", "review_sha256"])
}

async fn read_reviews(
    state: &AppState,
    implementation_id: &str,
) -> Result<Vec<ControlledShadowMarketDataParserImplementationReviewRecord>, String> {
    let directory = review_root(state).join(implementation_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut reviews = Vec::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        if entry
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("json")
        {
            let review: ControlledShadowMarketDataParserImplementationReviewRecord =
                serde_json::from_slice(
                    &tokio::fs::read(&path)
                        .await
                        .map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
            if path.file_stem().and_then(|value| value.to_str()) != Some(review.review_id.as_str())
            {
                return Err("Stage 98 复核文件名与记录 ID 不一致".to_string());
            }
            validate_review(&review)?;
            reviews.push(review);
        }
    }
    order_review_chain(reviews, implementation_id)
}

fn order_review_chain(
    reviews: Vec<ControlledShadowMarketDataParserImplementationReviewRecord>,
    implementation_id: &str,
) -> Result<Vec<ControlledShadowMarketDataParserImplementationReviewRecord>, String> {
    if reviews.is_empty() {
        return Ok(reviews);
    }
    let mut by_id = BTreeMap::new();
    let mut hashes = BTreeSet::new();
    for review in reviews {
        if review.implementation.implementation_id != implementation_id
            || !hashes.insert(review.review_sha256.clone())
            || by_id.insert(review.review_id.clone(), review).is_some()
        {
            return Err("Stage 98 复核重复或实现绑定不一致".to_string());
        }
    }
    let roots = by_id
        .values()
        .filter(|review| review.previous_review_id.is_none())
        .map(|review| review.review_id.clone())
        .collect::<Vec<_>>();
    if roots.len() != 1 {
        return Err("Stage 98 复核链必须恰有一个根".to_string());
    }
    let mut ordered = Vec::with_capacity(by_id.len());
    let mut next_id = Some(roots[0].clone());
    while let Some(current_id) = next_id {
        let review = by_id
            .remove(&current_id)
            .ok_or_else(|| "Stage 98 复核链断裂或包含循环".to_string())?;
        let prior = ordered.last();
        if review.previous_review_id
            != prior.map(
                |value: &ControlledShadowMarketDataParserImplementationReviewRecord| {
                    value.review_id.clone()
                },
            )
            || review.previous_review_sha256 != prior.map(|value| value.review_sha256.clone())
        {
            return Err("Stage 98 复核前序绑定无效".to_string());
        }
        let expected_actors = expected_excluded_actor_ids(&review.implementation, &ordered);
        if review.excluded_prior_actor_ids != expected_actors
            || expected_actors.contains(&review.reviewer_id)
        {
            return Err("Stage 98 复核角色隔离无效".to_string());
        }
        let successors = by_id
            .values()
            .filter(|candidate| candidate.previous_review_id.as_deref() == Some(&current_id))
            .map(|candidate| candidate.review_id.clone())
            .collect::<Vec<_>>();
        if successors.len() > 1 {
            return Err("Stage 98 复核链发生分叉".to_string());
        }
        if review.future_isolated_parser_runner_specification_registration_eligible
            && !successors.is_empty()
        {
            return Err("Stage 98 已批准复核必须是终态".to_string());
        }
        ordered.push(review);
        next_id = successors.into_iter().next();
    }
    if !by_id.is_empty() {
        return Err("Stage 98 复核链不连通".to_string());
    }
    Ok(ordered)
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 98 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|error| error.to_string())?)
    ))
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("{label}不能为空且不得超过 {MAX_TEXT_CHARS} 字"))
    } else {
        Ok(value.to_string())
    }
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
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

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_root(state: &AppState) -> PathBuf {
    decision_root(state).join(
        "controlled-shadow-first-natural-forward-cycle-market-data-parser-implementation-reviews",
    )
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
    let path = directory.join(format!(
        "controlled-shadow-market-data-parser-implementation-review-{implementation_id}.lock"
    ));
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
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 98 实现复核正在进行".to_string())?;
    Ok(ReviewLock(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ReviewControlledShadowMarketDataParserImplementationRequest {
        ReviewControlledShadowMarketDataParserImplementationRequest {
            expected_previous_review_id: None,
            expected_previous_review_sha256: None,
            expected_implementation_sha256: "1".repeat(64),
            expected_implementation_contract_sha256: "2".repeat(64),
            expected_specification_review_sha256: "3".repeat(64),
            expected_specification_registration_sha256: "4".repeat(64),
            expected_parser_specification_sha256: "5".repeat(64),
            expected_independent_audit_sha256: "6".repeat(64),
            verdict: ControlledShadowMarketDataParserImplementationReviewVerdict::ApprovedForFutureIsolatedMarketDataParserRunnerSpecificationRegistration,
            rationale: "第二实现独立复算通过。".to_string(),
            binding_and_recomputation_assessment: "全部上游指纹逐位一致。".to_string(),
            deterministic_parser_semantics_assessment: "八个纯函数身份和输出 schema 一致。".to_string(),
            source_schema_calendar_action_and_reconciliation_assessment: "来源、日历和公司行动对账边界完整。".to_string(),
            failure_and_missing_data_assessment: "错误和缺失保持严格失败关闭。".to_string(),
            zero_capability_assessment: "不存在工件、入口、runtime 或数据能力。".to_string(),
            known_limitations: "尚未读取真实载荷，来源时点仍未验证。".to_string(),
            future_runner_constraints: "通过后只可登记无入口隔离 runner 规格。".to_string(),
            exact_current_stage_51_through_stage_97_binding_confirmed: true,
            reviewer_independent_from_registrar_and_complete_prior_chain_confirmed: true,
            implementation_contract_review_registration_and_specification_hashes_independently_reproduced_confirmed: true,
            all_eight_function_ids_and_canonical_schemas_independently_reviewed_confirmed: true,
            explicit_price_dividend_split_and_official_calendar_sources_preserved_confirmed: true,
            strict_utf8_json_html_date_and_bounded_numeric_rules_preserved_confirmed: true,
            duplicate_out_of_window_missing_and_malformed_fail_closed_preserved_confirmed: true,
            no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_preserved_confirmed: true,
            spy_official_calendar_subject_gap_and_cross_source_reconciliation_preserved_confirmed: true,
            all_eight_synthetic_vectors_independently_reconstructed_confirmed: true,
            source_available_at_remains_unverified_until_separate_evidence_confirmed: true,
            future_output_content_addressed_create_once_untrusted_and_independently_validated_confirmed: true,
            no_source_or_executable_artifact_entrypoint_runtime_raw_payload_mount_read_environment_secret_network_tool_or_subprocess_confirmed: true,
            no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            approval_only_opens_future_isolated_parser_runner_specification_registration_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn approval_requires_every_stage_98_confirmation() {
        let mut value = request();
        assert!(request_checks(&value));
        value.all_eight_synthetic_vectors_independently_reconstructed_confirmed = false;
        assert!(!request_checks(&value));
    }

    #[test]
    fn expected_function_ids_cover_every_parser_boundary() {
        assert_eq!(EXPECTED_FUNCTION_IDS.len(), 8);
        assert!(
            EXPECTED_FUNCTION_IDS
                .iter()
                .all(|value| value.ends_with("_pure_specification"))
        );
        assert!(
            EXPECTED_FUNCTION_IDS
                .iter()
                .any(|value| value.contains("nyse_calendar"))
        );
        assert!(
            EXPECTED_FUNCTION_IDS
                .iter()
                .any(|value| value.contains("synthetic_vector"))
        );
    }

    #[test]
    fn fingerprint_binds_review_semantics() {
        #[derive(Serialize)]
        struct Fixture {
            review_id: String,
            review_sha256: String,
            verdict: String,
        }
        let before = fingerprint_without(
            &Fixture {
                review_id: String::new(),
                review_sha256: String::new(),
                verdict: "approved".to_string(),
            },
            &["review_id", "review_sha256"],
        )
        .unwrap();
        let after = fingerprint_without(
            &Fixture {
                review_id: String::new(),
                review_sha256: String::new(),
                verdict: "rejected".to_string(),
            },
            &["review_id", "review_sha256"],
        )
        .unwrap();
        assert_ne!(before, after);
    }

    #[test]
    fn review_ids_and_hashes_are_strictly_content_addressed() {
        assert!(valid_id(&"a".repeat(32)));
        assert!(!valid_id(&"a".repeat(31)));
        assert!(valid_sha256(&"b".repeat(64)));
        assert!(!valid_sha256(&"g".repeat(64)));
    }
}
