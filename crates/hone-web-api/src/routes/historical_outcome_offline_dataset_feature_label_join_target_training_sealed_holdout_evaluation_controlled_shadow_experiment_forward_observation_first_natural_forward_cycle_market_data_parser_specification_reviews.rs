//! Stage 96 chain-external, create-once review of one exact Stage 95 zero-capability
//! market-data parser specification.
//!
//! The reviewer independently reconstructs the fixed market-data request set, the Stage 95
//! registration/specification fingerprints and every synthetic vector hash. Approval opens only a
//! future zero-capability parser-implementation specification registration. This module has no
//! parser implementation, raw-payload mount, executable entrypoint, runtime or downstream
//! investment/trading authority.

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

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_specifications::{
    ControlledShadowMarketDataParserSpecificationRegistration,
    RegisteredControlledShadowMarketDataParserSpecification, SyntheticMarketDataParserTestVector,
    SyntheticParserExpectedOutcome, ZeroCapabilityMarketDataParserAuthorityBoundary,
    registered_market_data_parser_specifications_for_independent_review,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_receipt_attempts::CanonicalMarketDataRequestDescriptor;
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-review-v1";
const POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-chain-external-review-v1-no-parser";
const STAGE_95_RECORD_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-registration-v1";
const STAGE_95_POLICY_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-create-once-v1-zero-capability";
const STAGE_95_SPEC_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-market-data-parser-specification-v1";
const STAGE_95_PROTOCOL_VERSION: &str =
    "hone-natural-forward-market-data-parser-v1-specification-only-not-executable";
const MAX_TEXT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowMarketDataParserSpecificationReviewVerdict {
    ApprovedForFutureZeroCapabilityParserImplementationRegistration,
    ChangesRequiredRebuildParserSpecification,
    RejectedParserSpecification,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowMarketDataParserSpecificationRequest {
    expected_registration_sha256: String,
    expected_parser_specification_sha256: String,
    expected_validation_sha256: String,
    expected_receipt_sha256: String,
    expected_claim_sha256: String,
    expected_result_sha256: String,
    expected_adapter_authorization_sha256: String,
    expected_adapter_spec_sha256: String,
    expected_canonical_request_set_sha256: String,
    verdict: ControlledShadowMarketDataParserSpecificationReviewVerdict,
    rationale: String,
    source_contract_assessment: String,
    schema_and_numeric_assessment: String,
    calendar_and_reconciliation_assessment: String,
    synthetic_vector_assessment: String,
    failure_and_missing_data_assessment: String,
    known_limitations: String,
    future_implementation_constraints: String,
    exact_stage_51_through_stage_95_binding_confirmed: bool,
    reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: bool,
    independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed:
        bool,
    independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed:
        bool,
    independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed: bool,
    strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed: bool,
    duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed: bool,
    no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed: bool,
    separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed: bool,
    spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed: bool,
    source_available_at_remains_unverified_until_separate_review_confirmed: bool,
    specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed: bool,
    approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed: bool,
    no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ZeroCapabilityMarketDataParserSpecificationReviewAuthorityBoundary {
    pub parser_implementation_registered: bool,
    pub parser_implementation_present: bool,
    pub executable_artifact_present: bool,
    pub callable_entrypoint_present: bool,
    pub runtime_present: bool,
    pub raw_payload_read_or_mount_allowed: bool,
    pub environment_or_secret_access_allowed: bool,
    pub outbound_network_tool_or_subprocess_allowed: bool,
    pub production_read_or_write_allowed: bool,
    pub parsed_calendar_rows_created: bool,
    pub parsed_market_rows_created: bool,
    pub observation_write_allowed: bool,
    pub ledger_position_or_performance_write_allowed: bool,
    pub model_or_metric_store_write_allowed: bool,
    pub training_feedback_or_reward_allowed: bool,
    pub order_generation_or_broker_access_allowed: bool,
    pub trading_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowMarketDataParserSpecificationReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub registration_id: String,
    pub registration_sha256: String,
    pub parser_specification_sha256: String,
    pub validation_sha256: String,
    pub receipt_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub adapter_authorization_sha256: String,
    pub adapter_spec_sha256: String,
    pub canonical_request_set_sha256: String,
    pub independently_recomputed_validation_sha256: String,
    pub independently_recomputed_claim_sha256: String,
    pub independently_recomputed_result_sha256: String,
    pub independently_recomputed_receipt_sha256: String,
    pub independently_recomputed_registration_sha256: String,
    pub independently_recomputed_parser_specification_sha256: String,
    pub independently_reconstructed_canonical_request_set_sha256: String,
    pub independently_reconstructed_synthetic_vector_set_sha256: String,
    pub reviewed_at: DateTime<Utc>,
    pub reviewed_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowMarketDataParserSpecificationReviewVerdict,
    pub rationale: String,
    pub source_contract_assessment: String,
    pub schema_and_numeric_assessment: String,
    pub calendar_and_reconciliation_assessment: String,
    pub synthetic_vector_assessment: String,
    pub failure_and_missing_data_assessment: String,
    pub known_limitations: String,
    pub future_implementation_constraints: String,
    pub confirmations_complete: bool,
    pub exact_stage_51_through_stage_95_binding_confirmed: bool,
    pub reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed:
        bool,
    pub independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed:
        bool,
    pub independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed:
        bool,
    pub independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed: bool,
    pub strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed: bool,
    pub duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed: bool,
    pub no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed: bool,
    pub separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed: bool,
    pub spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed: bool,
    pub source_available_at_remains_unverified_until_separate_review_confirmed: bool,
    pub specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed:
        bool,
    pub approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed:
        bool,
    pub no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub validation_chain_independently_recomputed: bool,
    pub explicit_source_request_set_independently_reconstructed: bool,
    pub parser_specification_independently_recomputed: bool,
    pub synthetic_vectors_independently_reconstructed: bool,
    pub strict_fail_closed_semantics_independently_verified: bool,
    pub zero_capability_boundary_independently_verified: bool,
    pub independent_audit_passed: bool,
    pub mismatch_reasons: Vec<String>,
    pub parser_specification_independently_approved: bool,
    pub future_zero_capability_parser_implementation_registration_eligible: bool,
    pub authority_boundary: ZeroCapabilityMarketDataParserSpecificationReviewAuthorityBoundary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserSpecificationReviewItem {
    pub registration: ControlledShadowMarketDataParserSpecificationRegistration,
    pub validation_sha256: String,
    pub receipt_sha256: String,
    pub claim_sha256: String,
    pub result_sha256: String,
    pub adapter_authorization_sha256: String,
    pub adapter_spec_sha256: String,
    pub canonical_request_set_sha256: String,
    pub subject_symbols: Vec<String>,
    pub raw_payload_count: usize,
    pub latest_review: Option<ControlledShadowMarketDataParserSpecificationReview>,
    pub review_eligible: bool,
    pub independently_approved: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowMarketDataParserSpecificationReviewRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub review_endpoint_available: bool,
    pub items: Vec<ControlledShadowMarketDataParserSpecificationReviewItem>,
    pub parser_specification_registered_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_zero_capability_parser_implementation_registration_eligible_count: usize,
    pub review_status: String,
    pub parser_implementation_registered: bool,
    pub parser_implementation_present: bool,
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
pub(crate) struct ControlledShadowMarketDataParserSpecificationReviewReadinessSummary {
    pub parser_specification_registered_count: usize,
    pub review_eligible_count: usize,
    pub reviewed_count: usize,
    pub independently_approved_count: usize,
    pub changes_required_or_rejected_count: usize,
    pub future_zero_capability_parser_implementation_registration_eligible_count: usize,
    pub review_status: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct IndependentlyApprovedControlledShadowMarketDataParserSpecification {
    pub registered: RegisteredControlledShadowMarketDataParserSpecification,
    pub review: ControlledShadowMarketDataParserSpecificationReview,
}

#[derive(Debug)]
struct IndependentSpecificationAudit {
    validation_chain: bool,
    explicit_request_set: bool,
    parser_specification: bool,
    synthetic_vectors: bool,
    strict_fail_closed_semantics: bool,
    zero_capability_boundary: bool,
    validation_sha256: String,
    claim_sha256: String,
    result_sha256: String,
    receipt_sha256: String,
    registration_sha256: String,
    parser_specification_sha256: String,
    canonical_request_set_sha256: String,
    synthetic_vector_set_sha256: String,
    mismatch_reasons: Vec<String>,
}

impl IndependentSpecificationAudit {
    fn passed(&self) -> bool {
        self.validation_chain
            && self.explicit_request_set
            && self.parser_specification
            && self.synthetic_vectors
            && self.strict_fail_closed_semantics
            && self.zero_capability_boundary
            && self.mismatch_reasons.is_empty()
    }
}

pub(crate) async fn handle_get_controlled_shadow_market_data_parser_specification_reviews(
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
            warn!(%error, "market-data parser specification review registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "行情解析器规格独立复核表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_market_data_parser_specification_once(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(registration_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowMarketDataParserSpecificationRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_once(&state, &admin.user_id, &registration_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(_) => crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "复核已写入，但刷新复核表失败",
            ),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_market_data_parser_specification_review_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserSpecificationReviewReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(
        ControlledShadowMarketDataParserSpecificationReviewReadinessSummary {
            parser_specification_registered_count: registry.parser_specification_registered_count,
            review_eligible_count: registry.review_eligible_count,
            reviewed_count: registry.reviewed_count,
            independently_approved_count: registry.independently_approved_count,
            changes_required_or_rejected_count: registry.changes_required_or_rejected_count,
            future_zero_capability_parser_implementation_registration_eligible_count: registry
                .future_zero_capability_parser_implementation_registration_eligible_count,
            review_status: registry.review_status,
        },
    )
}

#[allow(dead_code)]
pub(crate) async fn independently_approved_market_data_parser_specifications_for_implementation_registration(
    state: &AppState,
) -> Result<Vec<IndependentlyApprovedControlledShadowMarketDataParserSpecification>, String> {
    let registered =
        registered_market_data_parser_specifications_for_independent_review(state).await?;
    let reviews = read_records(state).await?;
    let index = registered
        .into_iter()
        .map(|value| (value.registration.registration_id.clone(), value))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for review in reviews {
        let source = index
            .get(&review.registration_id)
            .cloned()
            .ok_or_else(|| "Stage 96 复核缺少当前 Stage 95 登记".to_string())?;
        validate_review(&review, &source)?;
        if review.parser_specification_independently_approved
            && review.future_zero_capability_parser_implementation_registration_eligible
        {
            output.push(
                IndependentlyApprovedControlledShadowMarketDataParserSpecification {
                    registered: source,
                    review,
                },
            );
        }
    }
    output.sort_by(|left, right| right.review.reviewed_at.cmp(&left.review.reviewed_at));
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowMarketDataParserSpecificationReviewRegistry, String> {
    let registered =
        registered_market_data_parser_specifications_for_independent_review(state).await?;
    let reviews = read_records(state).await?;
    let registered_index = registered
        .iter()
        .map(|value| (value.registration.registration_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let mut review_index = HashMap::new();
    for review in reviews {
        let source = registered_index
            .get(review.registration_id.as_str())
            .ok_or_else(|| "Stage 96 复核引用了不存在或已漂移的 Stage 95 登记".to_string())?;
        validate_review(&review, source)?;
        if review_index
            .insert(review.registration_id.clone(), review)
            .is_some()
        {
            return Err("同一 Stage 95 登记存在多个 Stage 96 终态复核".into());
        }
    }
    let mut items = registered
        .into_iter()
        .map(|source| {
            let review = review_index.remove(&source.registration.registration_id);
            let approved = review
                .as_ref()
                .is_some_and(|value| value.parser_specification_independently_approved);
            let receipt = source
                .source
                .attempt
                .result
                .untrusted_raw_market_data_receipt
                .as_ref()
                .expect("Stage 95 source is independently validated");
            ControlledShadowMarketDataParserSpecificationReviewItem {
                validation_sha256: source.source.validation.validation_sha256.clone(),
                receipt_sha256: receipt.receipt_sha256.clone(),
                claim_sha256: source.source.attempt.claim.claim_sha256.clone(),
                result_sha256: source.source.attempt.result.result_sha256.clone(),
                adapter_authorization_sha256: source
                    .source
                    .attempt
                    .claim
                    .adapter_authorization_sha256
                    .clone(),
                adapter_spec_sha256: source.source.attempt.claim.adapter_spec_sha256.clone(),
                canonical_request_set_sha256: source
                    .source
                    .attempt
                    .claim
                    .canonical_request_set_sha256
                    .clone(),
                registration: source.registration,
                subject_symbols: source.source.attempt.claim.subject_symbols,
                raw_payload_count: receipt.raw_payload_count,
                review_eligible: review.is_none(),
                latest_review: review,
                independently_approved: approved,
            }
        })
        .collect::<Vec<_>>();
    if !review_index.is_empty() {
        return Err("Stage 96 复核存在孤立记录".into());
    }
    items.sort_by(|left, right| {
        left.registration
            .registration_id
            .cmp(&right.registration.registration_id)
    });
    let parser_specification_registered_count = items.len();
    let review_eligible_count = items.iter().filter(|value| value.review_eligible).count();
    let reviewed_count = items
        .iter()
        .filter(|value| value.latest_review.is_some())
        .count();
    let independently_approved_count = items
        .iter()
        .filter(|value| value.independently_approved)
        .count();
    let changes_required_or_rejected_count = items
        .iter()
        .filter(|value| value.latest_review.is_some() && !value.independently_approved)
        .count();
    let review_status = if independently_approved_count > 0 {
        "parser_specification_independently_approved_waiting_zero_capability_implementation_registration"
    } else if changes_required_or_rejected_count > 0 {
        "parser_specification_changes_required_rebuild_or_rejected"
    } else if review_eligible_count > 0 {
        "waiting_chain_external_parser_specification_review"
    } else {
        "waiting_stage_95_zero_capability_parser_specification_registration"
    };
    Ok(ControlledShadowMarketDataParserSpecificationReviewRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        review_endpoint_available: true,
        parser_specification_registered_count,
        review_eligible_count,
        reviewed_count,
        independently_approved_count,
        changes_required_or_rejected_count,
        future_zero_capability_parser_implementation_registration_eligible_count:
            independently_approved_count,
        items,
        review_status: review_status.into(),
        parser_implementation_registered: false,
        parser_implementation_present: false,
        raw_payload_accessed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        forward_observation_started: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 96 由完整 Stage 51–95 责任链之外的新角色，使用第二套实现独立重算 validation/claim/result/receipt/registration/spec 摘要、五类 FMP stable 显式请求、NYSE 日历请求和八个合成向量。批准只开放未来零能力 parser 实现规格登记资格；本阶段没有 parser、载荷读取、runtime、真实解析、观察、业绩或交易权限。".into(),
    })
}

async fn review_once(
    state: &AppState,
    reviewer_id: &str,
    registration_id: &str,
    request: ReviewControlledShadowMarketDataParserSpecificationRequest,
) -> Result<ControlledShadowMarketDataParserSpecificationReview, String> {
    validate_request(registration_id, &request)?;
    let _lock = acquire_lock(state, registration_id).await?;
    if read_records(state)
        .await?
        .iter()
        .any(|value| value.registration_id == registration_id)
    {
        return Err("该 Stage 95 登记已有不可覆盖的 Stage 96 终态复核".into());
    }
    let registered = registered_market_data_parser_specifications_for_independent_review(state)
        .await?
        .into_iter()
        .find(|value| value.registration.registration_id == registration_id)
        .ok_or_else(|| "当前没有可独立复核的 Stage 95 规格登记".to_string())?;
    verify_expected_bindings(&registered, &request)?;
    let excluded_prior_actor_ids = excluded_actor_ids(&registered);
    if reviewer_id.trim().is_empty()
        || excluded_prior_actor_ids
            .iter()
            .any(|value| value == reviewer_id)
    {
        return Err(
            "Stage 96 reviewer 必须独立于 registrar、validator、executor 和完整既有责任链".into(),
        );
    }
    let audit = independently_audit(&registered)?;
    let confirmations_complete = confirmations_complete(&request);
    let approving = matches!(
        request.verdict,
        ControlledShadowMarketDataParserSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityParserImplementationRegistration
    );
    if approving && (!audit.passed() || !confirmations_complete) {
        return Err("Stage 96 独立审计或确认项未完整通过，不能批准".into());
    }
    let approved = approving && audit.passed() && confirmations_complete;
    let mut review = ControlledShadowMarketDataParserSpecificationReview {
        schema_version: REVIEW_SCHEMA_VERSION.into(),
        policy_version: POLICY_VERSION.into(),
        review_id: String::new(),
        review_sha256: String::new(),
        registration_id: registered.registration.registration_id.clone(),
        registration_sha256: registered.registration.registration_sha256.clone(),
        parser_specification_sha256: registered
            .registration
            .parser_specification
            .parser_specification_sha256
            .clone(),
        validation_sha256: registered.source.validation.validation_sha256.clone(),
        receipt_sha256: registered.registration.parser_specification.stage_93_receipt_sha256.clone(),
        claim_sha256: registered.source.attempt.claim.claim_sha256.clone(),
        result_sha256: registered.source.attempt.result.result_sha256.clone(),
        adapter_authorization_sha256: registered
            .source
            .attempt
            .claim
            .adapter_authorization_sha256
            .clone(),
        adapter_spec_sha256: registered.source.attempt.claim.adapter_spec_sha256.clone(),
        canonical_request_set_sha256: registered
            .source
            .attempt
            .claim
            .canonical_request_set_sha256
            .clone(),
        independently_recomputed_validation_sha256: audit.validation_sha256.clone(),
        independently_recomputed_claim_sha256: audit.claim_sha256.clone(),
        independently_recomputed_result_sha256: audit.result_sha256.clone(),
        independently_recomputed_receipt_sha256: audit.receipt_sha256.clone(),
        independently_recomputed_registration_sha256: audit.registration_sha256.clone(),
        independently_recomputed_parser_specification_sha256: audit.parser_specification_sha256.clone(),
        independently_reconstructed_canonical_request_set_sha256: audit
            .canonical_request_set_sha256
            .clone(),
        independently_reconstructed_synthetic_vector_set_sha256: audit
            .synthetic_vector_set_sha256
            .clone(),
        reviewed_at: Utc::now(),
        reviewed_by: reviewer_id.into(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核理由")?,
        source_contract_assessment: bounded_required(
            &request.source_contract_assessment,
            "来源合同评估",
        )?,
        schema_and_numeric_assessment: bounded_required(
            &request.schema_and_numeric_assessment,
            "schema 与数值评估",
        )?,
        calendar_and_reconciliation_assessment: bounded_required(
            &request.calendar_and_reconciliation_assessment,
            "日历与对账评估",
        )?,
        synthetic_vector_assessment: bounded_required(
            &request.synthetic_vector_assessment,
            "合成向量评估",
        )?,
        failure_and_missing_data_assessment: bounded_required(
            &request.failure_and_missing_data_assessment,
            "失败与缺失数据评估",
        )?,
        known_limitations: bounded_required(&request.known_limitations, "已知限制")?,
        future_implementation_constraints: bounded_required(
            &request.future_implementation_constraints,
            "未来实现约束",
        )?,
        confirmations_complete,
        exact_stage_51_through_stage_95_binding_confirmed: request
            .exact_stage_51_through_stage_95_binding_confirmed,
        reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed:
            request.reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed,
        independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed:
            request.independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed,
        independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed:
            request.independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed,
        independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed:
            request.independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed,
        strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed: request
            .strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed,
        duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed: request
            .duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed,
        no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed: request
            .no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed,
        separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed: request
            .separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed,
        spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed: request
            .spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed,
        source_available_at_remains_unverified_until_separate_review_confirmed: request
            .source_available_at_remains_unverified_until_separate_review_confirmed,
        specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed:
            request.specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed,
        approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed:
            request.approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed,
        no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
            request.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        validation_chain_independently_recomputed: audit.validation_chain,
        explicit_source_request_set_independently_reconstructed: audit.explicit_request_set,
        parser_specification_independently_recomputed: audit.parser_specification,
        synthetic_vectors_independently_reconstructed: audit.synthetic_vectors,
        strict_fail_closed_semantics_independently_verified: audit.strict_fail_closed_semantics,
        zero_capability_boundary_independently_verified: audit.zero_capability_boundary,
        independent_audit_passed: audit.passed(),
        mismatch_reasons: audit.mismatch_reasons,
        parser_specification_independently_approved: approved,
        future_zero_capability_parser_implementation_registration_eligible: approved,
        authority_boundary: closed_review_authority_boundary(),
    };
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].into();
    validate_review(&review, &registered)?;
    write_immutable_json(
        &record_dir(state).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn independently_audit(
    registered: &RegisteredControlledShadowMarketDataParserSpecification,
) -> Result<IndependentSpecificationAudit, String> {
    let registration = &registered.registration;
    let specification = &registration.parser_specification;
    let validation = &registered.source.validation;
    let claim = &registered.source.attempt.claim;
    let result = &registered.source.attempt.result;
    let receipt = result
        .untrusted_raw_market_data_receipt
        .as_ref()
        .ok_or_else(|| "Stage 96 上游缺少 Stage 93 原始收据".to_string())?;
    let validation_sha256 =
        fingerprint_without(validation, &["validation_id", "validation_sha256"])?;
    let claim_sha256 = fingerprint_without(claim, &["attempt_id", "claim_sha256"])?;
    let result_sha256 = fingerprint_without(result, &["result_id", "result_sha256"])?;
    let receipt_sha256 = fingerprint_without(receipt, &["receipt_sha256"])?;
    let registration_sha256 =
        fingerprint_without(registration, &["registration_id", "registration_sha256"])?;
    let parser_specification_sha256 =
        fingerprint_without(specification, &["parser_specification_sha256"])?;
    let expected_requests = independently_reconstruct_canonical_requests(
        &claim.subject_symbols,
        claim.window_start_date,
        claim.window_end_date,
    )?;
    let canonical_request_set_sha256 = fingerprint(&expected_requests)?;
    let expected_vectors = independently_reconstruct_synthetic_vectors();
    let synthetic_vector_set_sha256 = fingerprint(&expected_vectors)?;
    let validation_chain = validation_sha256 == validation.validation_sha256
        && claim_sha256 == claim.claim_sha256
        && result_sha256 == result.result_sha256
        && receipt_sha256 == receipt.receipt_sha256
        && registration_sha256 == registration.registration_sha256
        && parser_specification_sha256 == specification.parser_specification_sha256
        && validation.raw_market_data_receipt_independently_validated
        && validation.future_market_data_parser_review_eligible
        && validation.mismatch_reasons.is_empty()
        && registration.stage_94_validation_id == validation.validation_id
        && registration.stage_94_validation_sha256 == validation.validation_sha256
        && specification.stage_94_validation_id == validation.validation_id
        && specification.stage_94_validation_sha256 == validation.validation_sha256
        && specification.stage_93_claim_sha256 == claim.claim_sha256
        && specification.stage_93_result_sha256 == result.result_sha256
        && specification.stage_93_receipt_sha256 == receipt.receipt_sha256
        && specification.stage_92_adapter_authorization_sha256
            == claim.adapter_authorization_sha256
        && specification.adapter_spec_sha256 == claim.adapter_spec_sha256
        && specification.canonical_request_set_sha256 == claim.canonical_request_set_sha256;
    let explicit_request_set = claim.canonical_requests == expected_requests
        && claim.expected_request_count == expected_requests.len()
        && claim.canonical_request_set_sha256 == canonical_request_set_sha256;
    let parser_specification = registration.schema_version == STAGE_95_RECORD_SCHEMA_VERSION
        && registration.policy_version == STAGE_95_POLICY_VERSION
        && specification.schema_version == STAGE_95_SPEC_SCHEMA_VERSION
        && specification.parser_protocol_version == STAGE_95_PROTOCOL_VERSION
        && registration.parser_specification_registered
        && registration.future_chain_external_specification_review_eligible
        && !registration.specification_review_completed
        && !registration.parser_implementation_registration_eligible;
    let synthetic_vectors = specification.synthetic_test_vectors == expected_vectors
        && specification.synthetic_test_vectors.iter().all(|value| {
            value.synthetic_only_no_market_truth
                && value.input_fixture_sha256 == sha256_bytes(value.input_fixture.as_bytes())
                && value.expected_canonical_output_sha256
                    == value
                        .expected_canonical_output
                        .as_ref()
                        .map(|output| sha256_bytes(output.as_bytes()))
        });
    let strict_fail_closed_semantics = specification.source_contract_revision
        == "fmp-stable-explicit-price-dividend-split-plus-nyse-v1"
        && specification.accepted_source_kinds
            == [
                "fmp_dividend_adjusted_price",
                "fmp_dividends",
                "fmp_raw_unadjusted_price",
                "fmp_split_adjusted_price",
                "fmp_splits",
                "nyse_official_us_market_calendar",
            ]
        && specification.strict_utf8_required
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
        && !specification.inferred_dividend_or_split_allowed
        && specification.spy_required_for_every_official_session
        && specification.subject_missing_session_is_explicit_gap_not_synthetic_row
        && specification.separate_price_series_never_silently_substituted
        && specification.explicit_actions_required_for_action_adjustment_claims
        && specification.source_available_at_remains_unverified_until_separate_review
        && specification.parser_output_create_once_and_untrusted
        && specification.parser_output_independent_validation_required;
    let zero_capability_boundary = stage_95_authority_is_closed(&specification.authority_boundary)
        && registration_downstream_is_closed(registration);
    let mut mismatch_reasons = Vec::new();
    for (passed, reason) in [
        (
            validation_chain,
            "validation/claim/result/receipt/registration/spec 摘要或全链绑定不一致",
        ),
        (
            explicit_request_set,
            "五类 FMP stable 与 NYSE 规范请求集合无法独立重建",
        ),
        (
            parser_specification,
            "Stage 95 登记或规格 schema/状态不符合零能力合同",
        ),
        (
            synthetic_vectors,
            "八个合成向量或其输入/输出摘要无法独立重建",
        ),
        (
            strict_fail_closed_semantics,
            "严格 schema、缺失/重复/公司行动处理规则漂移",
        ),
        (
            zero_capability_boundary,
            "Stage 95 或 Stage 96 权限边界未保持全关闭",
        ),
    ] {
        if !passed {
            mismatch_reasons.push(reason.to_string());
        }
    }
    Ok(IndependentSpecificationAudit {
        validation_chain,
        explicit_request_set,
        parser_specification,
        synthetic_vectors,
        strict_fail_closed_semantics,
        zero_capability_boundary,
        validation_sha256,
        claim_sha256,
        result_sha256,
        receipt_sha256,
        registration_sha256,
        parser_specification_sha256,
        canonical_request_set_sha256,
        synthetic_vector_set_sha256,
        mismatch_reasons,
    })
}

fn independently_reconstruct_canonical_requests(
    symbols: &[String],
    from: chrono::NaiveDate,
    to: chrono::NaiveDate,
) -> Result<Vec<CanonicalMarketDataRequestDescriptor>, String> {
    let mut requested = symbols.to_vec();
    if !requested.iter().any(|value| value == "SPY") {
        requested.push("SPY".into());
    }
    requested.sort();
    requested.dedup();
    let mut output = Vec::new();
    for symbol in requested {
        for (source_kind, path, data_classes) in [
            (
                "split_adjusted_price",
                "/stable/historical-price-eod/full",
                vec!["ohlcv", "split_adjusted_close"],
            ),
            (
                "raw_unadjusted_price",
                "/stable/historical-price-eod/non-split-adjusted",
                vec!["ohlcv", "raw_unadjusted_close"],
            ),
            (
                "dividend_adjusted_price",
                "/stable/historical-price-eod/dividend-adjusted",
                vec!["dividend_adjusted_close", "ohlcv"],
            ),
            ("dividends", "/stable/dividends", vec!["dividend_event"]),
            ("splits", "/stable/splits", vec!["split_event"]),
        ] {
            let mut descriptor = CanonicalMarketDataRequestDescriptor {
                source_id: format!("fmp_{source_kind}_{symbol}"),
                method: "GET".into(),
                https_origin: "https://financialmodelingprep.com".into(),
                path: path.into(),
                query: vec![
                    ("apikey".into(), "REDACTED".into()),
                    ("from".into(), from.to_string()),
                    ("symbol".into(), symbol.clone()),
                    ("to".into(), to.to_string()),
                ],
                data_classes: data_classes.into_iter().map(str::to_string).collect(),
                canonical_request_sha256: String::new(),
            };
            descriptor.canonical_request_sha256 =
                fingerprint_without(&descriptor, &["canonical_request_sha256"])?;
            output.push(descriptor);
        }
    }
    let mut calendar = CanonicalMarketDataRequestDescriptor {
        source_id: "nyse_official_us_market_calendar".into(),
        method: "GET".into(),
        https_origin: "https://www.nyse.com".into(),
        path: "/markets/hours-calendars".into(),
        query: Vec::new(),
        data_classes: vec!["official_us_market_calendar".into()],
        canonical_request_sha256: String::new(),
    };
    calendar.canonical_request_sha256 =
        fingerprint_without(&calendar, &["canonical_request_sha256"])?;
    output.push(calendar);
    output.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    Ok(output)
}

fn independently_reconstruct_synthetic_vectors() -> Vec<SyntheticMarketDataParserTestVector> {
    let fixtures = [
        (
            "price-valid-two-rows",
            "fmp_split_adjusted_price",
            r#"[{"symbol":"ZZTEST","date":"2099-01-03","close":10.5,"volume":100},{"symbol":"ZZTEST","date":"2099-01-04","close":10.75,"volume":120}]"#,
            SyntheticParserExpectedOutcome::AcceptCanonicalRows,
            Some(
                r#"[{"close_decimal":"10.5","date":"2099-01-03","symbol":"ZZTEST","volume_integer":100},{"close_decimal":"10.75","date":"2099-01-04","symbol":"ZZTEST","volume_integer":120}]"#,
            ),
        ),
        (
            "price-duplicate-date-rejected",
            "fmp_raw_unadjusted_price",
            r#"[{"symbol":"ZZTEST","date":"2099-01-03","close":10.5},{"symbol":"ZZTEST","date":"2099-01-03","close":10.6}]"#,
            SyntheticParserExpectedOutcome::RejectDuplicateDate,
            None,
        ),
        (
            "price-string-decimal-rejected",
            "fmp_dividend_adjusted_price",
            r#"[{"symbol":"ZZTEST","date":"2099-01-03","close":"10.5"}]"#,
            SyntheticParserExpectedOutcome::RejectMalformedDecimal,
            None,
        ),
        (
            "price-out-of-window-rejected",
            "fmp_split_adjusted_price",
            r#"[{"symbol":"ZZTEST","date":"2098-12-31","close":10.5}]"#,
            SyntheticParserExpectedOutcome::RejectOutOfWindowDate,
            None,
        ),
        (
            "dividend-empty-accepted",
            "fmp_dividends",
            "[]",
            SyntheticParserExpectedOutcome::AcceptEmptyEventSet,
            Some("[]"),
        ),
        (
            "split-required-ratio-rejected",
            "fmp_splits",
            r#"[{"symbol":"ZZTEST","date":"2099-01-03"}]"#,
            SyntheticParserExpectedOutcome::RejectMissingRequiredField,
            None,
        ),
        (
            "nyse-synthetic-table-accepted",
            "nyse_official_us_market_calendar",
            r#"<table><thead><tr><th>Date</th><th>Session</th><th>Close ET</th></tr></thead><tbody><tr><td>2099-01-03</td><td>regular</td><td>16:00</td></tr></tbody></table>"#,
            SyntheticParserExpectedOutcome::AcceptCanonicalRows,
            Some(r#"[{"close_time_et":"16:00","date":"2099-01-03","session_kind":"regular"}]"#),
        ),
        (
            "nyse-missing-table-rejected",
            "nyse_official_us_market_calendar",
            "<html><body>synthetic fixture without calendar table</body></html>",
            SyntheticParserExpectedOutcome::RejectMissingOfficialCalendarTable,
            None,
        ),
    ];
    fixtures
        .into_iter()
        .map(
            |(vector_id, source_kind, input_fixture, expected_outcome, expected_output)| {
                SyntheticMarketDataParserTestVector {
                    vector_id: vector_id.into(),
                    source_kind: source_kind.into(),
                    input_fixture: input_fixture.into(),
                    input_fixture_sha256: sha256_bytes(input_fixture.as_bytes()),
                    expected_outcome,
                    expected_canonical_output: expected_output.map(str::to_string),
                    expected_canonical_output_sha256: expected_output
                        .map(|value| sha256_bytes(value.as_bytes())),
                    synthetic_only_no_market_truth: true,
                }
            },
        )
        .collect()
}

fn stage_95_authority_is_closed(value: &ZeroCapabilityMarketDataParserAuthorityBoundary) -> bool {
    !value.parser_implementation_present
        && !value.executable_artifact_present
        && !value.callable_entrypoint_present
        && !value.runtime_present
        && !value.raw_payload_mount_present
        && !value.environment_inheritance_allowed
        && !value.secrets_allowed
        && !value.outbound_network_allowed
        && !value.tools_allowed
        && !value.subprocesses_allowed
        && !value.production_reads_allowed
        && !value.production_writes_allowed
        && !value.calendar_rows_created
        && !value.market_rows_created
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

fn registration_downstream_is_closed(
    value: &ControlledShadowMarketDataParserSpecificationRegistration,
) -> bool {
    !value.specification_review_completed
        && !value.parser_implementation_registration_eligible
        && !value.parsed_calendar_created
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

fn closed_review_authority_boundary()
-> ZeroCapabilityMarketDataParserSpecificationReviewAuthorityBoundary {
    ZeroCapabilityMarketDataParserSpecificationReviewAuthorityBoundary {
        parser_implementation_registered: false,
        parser_implementation_present: false,
        executable_artifact_present: false,
        callable_entrypoint_present: false,
        runtime_present: false,
        raw_payload_read_or_mount_allowed: false,
        environment_or_secret_access_allowed: false,
        outbound_network_tool_or_subprocess_allowed: false,
        production_read_or_write_allowed: false,
        parsed_calendar_rows_created: false,
        parsed_market_rows_created: false,
        observation_write_allowed: false,
        ledger_position_or_performance_write_allowed: false,
        model_or_metric_store_write_allowed: false,
        training_feedback_or_reward_allowed: false,
        order_generation_or_broker_access_allowed: false,
        trading_allowed: false,
    }
}

fn excluded_actor_ids(
    registered: &RegisteredControlledShadowMarketDataParserSpecification,
) -> Vec<String> {
    let mut actors = registered.registration.excluded_prior_actor_ids.clone();
    actors.push(registered.registration.registered_by.clone());
    actors.push(registered.source.validation.validated_by.clone());
    actors.push(registered.source.attempt.claim.claimed_by.clone());
    actors.sort();
    actors.dedup();
    actors
}

fn verify_expected_bindings(
    registered: &RegisteredControlledShadowMarketDataParserSpecification,
    request: &ReviewControlledShadowMarketDataParserSpecificationRequest,
) -> Result<(), String> {
    let registration = &registered.registration;
    let claim = &registered.source.attempt.claim;
    let result = &registered.source.attempt.result;
    if request.expected_registration_sha256 != registration.registration_sha256
        || request.expected_parser_specification_sha256
            != registration
                .parser_specification
                .parser_specification_sha256
        || request.expected_validation_sha256 != registered.source.validation.validation_sha256
        || request.expected_receipt_sha256
            != registration.parser_specification.stage_93_receipt_sha256
        || request.expected_claim_sha256 != claim.claim_sha256
        || request.expected_result_sha256 != result.result_sha256
        || request.expected_adapter_authorization_sha256 != claim.adapter_authorization_sha256
        || request.expected_adapter_spec_sha256 != claim.adapter_spec_sha256
        || request.expected_canonical_request_set_sha256 != claim.canonical_request_set_sha256
    {
        return Err("Stage 96 预期 Stage 92–95 绑定已经漂移".into());
    }
    Ok(())
}

fn validate_request(
    registration_id: &str,
    request: &ReviewControlledShadowMarketDataParserSpecificationRequest,
) -> Result<(), String> {
    if !valid_id(registration_id)
        || ![
            &request.expected_registration_sha256,
            &request.expected_parser_specification_sha256,
            &request.expected_validation_sha256,
            &request.expected_receipt_sha256,
            &request.expected_claim_sha256,
            &request.expected_result_sha256,
            &request.expected_adapter_authorization_sha256,
            &request.expected_adapter_spec_sha256,
            &request.expected_canonical_request_set_sha256,
        ]
        .iter()
        .all(|value| valid_sha256(value))
        || [
            (&request.rationale, "复核理由"),
            (&request.source_contract_assessment, "来源合同评估"),
            (&request.schema_and_numeric_assessment, "schema 与数值评估"),
            (
                &request.calendar_and_reconciliation_assessment,
                "日历与对账评估",
            ),
            (&request.synthetic_vector_assessment, "合成向量评估"),
            (
                &request.failure_and_missing_data_assessment,
                "失败与缺失数据评估",
            ),
            (&request.known_limitations, "已知限制"),
            (&request.future_implementation_constraints, "未来实现约束"),
        ]
        .iter()
        .any(|(value, label)| bounded_required(value, label).is_err())
        || !confirmations_complete(request)
    {
        return Err("Stage 96 规格独立复核请求无效或确认项不完整".into());
    }
    Ok(())
}

fn confirmations_complete(
    value: &ReviewControlledShadowMarketDataParserSpecificationRequest,
) -> bool {
    value.exact_stage_51_through_stage_95_binding_confirmed
        && value.reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed
        && value.independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed
        && value.independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed
        && value.independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed
        && value.strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed
        && value.duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed
        && value.no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed
        && value.separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed
        && value.spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed
        && value.source_available_at_remains_unverified_until_separate_review_confirmed
        && value.specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed
        && value.approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed
        && value.no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && value.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn validate_review_shape(
    value: &ControlledShadowMarketDataParserSpecificationReview,
) -> Result<(), String> {
    if value.schema_version != REVIEW_SCHEMA_VERSION
        || value.policy_version != POLICY_VERSION
        || !valid_id(&value.review_id)
        || !valid_id(&value.registration_id)
        || !valid_sha256(&value.review_sha256)
        || value.review_id != value.review_sha256[..32]
        || value.review_sha256 != review_fingerprint(value)?
        || value.reviewed_by.trim().is_empty()
        || !sorted_unique(&value.excluded_prior_actor_ids)
        || value.excluded_prior_actor_ids.contains(&value.reviewed_by)
        || !value.confirmations_complete
        || value.authority_boundary != closed_review_authority_boundary()
    {
        return Err("Stage 96 规格独立复核记录无效、漂移或越权".into());
    }
    Ok(())
}

fn validate_review(
    value: &ControlledShadowMarketDataParserSpecificationReview,
    registered: &RegisteredControlledShadowMarketDataParserSpecification,
) -> Result<(), String> {
    validate_review_shape(value)?;
    let audit = independently_audit(registered)?;
    let expected_approved = matches!(
        value.verdict,
        ControlledShadowMarketDataParserSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityParserImplementationRegistration
    ) && audit.passed()
        && value.confirmations_complete;
    let expected_excluded = excluded_actor_ids(registered);
    if value.registration_id != registered.registration.registration_id
        || value.registration_sha256 != registered.registration.registration_sha256
        || value.parser_specification_sha256
            != registered
                .registration
                .parser_specification
                .parser_specification_sha256
        || value.validation_sha256 != registered.source.validation.validation_sha256
        || value.receipt_sha256
            != registered
                .registration
                .parser_specification
                .stage_93_receipt_sha256
        || value.claim_sha256 != registered.source.attempt.claim.claim_sha256
        || value.result_sha256 != registered.source.attempt.result.result_sha256
        || value.adapter_authorization_sha256
            != registered.source.attempt.claim.adapter_authorization_sha256
        || value.adapter_spec_sha256 != registered.source.attempt.claim.adapter_spec_sha256
        || value.canonical_request_set_sha256
            != registered.source.attempt.claim.canonical_request_set_sha256
        || value.excluded_prior_actor_ids != expected_excluded
        || value.independently_recomputed_validation_sha256 != audit.validation_sha256
        || value.independently_recomputed_claim_sha256 != audit.claim_sha256
        || value.independently_recomputed_result_sha256 != audit.result_sha256
        || value.independently_recomputed_receipt_sha256 != audit.receipt_sha256
        || value.independently_recomputed_registration_sha256 != audit.registration_sha256
        || value.independently_recomputed_parser_specification_sha256
            != audit.parser_specification_sha256
        || value.independently_reconstructed_canonical_request_set_sha256
            != audit.canonical_request_set_sha256
        || value.independently_reconstructed_synthetic_vector_set_sha256
            != audit.synthetic_vector_set_sha256
        || value.validation_chain_independently_recomputed != audit.validation_chain
        || value.explicit_source_request_set_independently_reconstructed
            != audit.explicit_request_set
        || value.parser_specification_independently_recomputed != audit.parser_specification
        || value.synthetic_vectors_independently_reconstructed != audit.synthetic_vectors
        || value.strict_fail_closed_semantics_independently_verified
            != audit.strict_fail_closed_semantics
        || value.zero_capability_boundary_independently_verified != audit.zero_capability_boundary
        || value.independent_audit_passed != audit.passed()
        || value.mismatch_reasons != audit.mismatch_reasons
        || value.parser_specification_independently_approved != expected_approved
        || value.future_zero_capability_parser_implementation_registration_eligible
            != expected_approved
    {
        return Err("Stage 96 独立复核与当前 Stage 95 规格或第二实现审计不一致".into());
    }
    Ok(())
}

async fn read_records(
    state: &AppState,
) -> Result<Vec<ControlledShadowMarketDataParserSpecificationReview>, String> {
    let directory = record_dir(state);
    let mut records = Vec::new();
    let mut ids = BTreeSet::new();
    let mut registrations = BTreeSet::new();
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(records),
        Err(error) => return Err(format!("读取 Stage 96 复核失败: {error}")),
    };
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| format!("遍历 Stage 96 复核失败: {error}"))?
    {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let value: ControlledShadowMarketDataParserSpecificationReview = serde_json::from_slice(
            &tokio::fs::read(&path)
                .await
                .map_err(|error| format!("读取 Stage 96 复核文件失败: {error}"))?,
        )
        .map_err(|error| format!("解析 Stage 96 复核文件失败: {error}"))?;
        validate_review_shape(&value)?;
        if path.file_stem().and_then(|value| value.to_str()) != Some(&value.review_id)
            || !ids.insert(value.review_id.clone())
            || !registrations.insert(value.registration_id.clone())
        {
            return Err("Stage 96 文件名、review ID 或 registration ID 重复".into());
        }
        records.push(value);
    }
    records.sort_by_key(|value| value.reviewed_at);
    Ok(records)
}

fn review_fingerprint(
    value: &ControlledShadowMarketDataParserSpecificationReview,
) -> Result<String, String> {
    fingerprint_without(value, &["review_id", "review_sha256"])
}

fn fingerprint<T: Serialize>(value: &T) -> Result<String, String> {
    Ok(sha256_bytes(
        &serde_json::to_vec(value).map_err(|error| error.to_string())?,
    ))
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut json = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = json
        .as_object_mut()
        .ok_or_else(|| "Stage 96 指纹载荷无效".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    fingerprint(&json)
}

fn sha256_bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn bounded_required(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT_CHARS {
        Err(format!("Stage 96 {label}为空或超过 {MAX_TEXT_CHARS} 字"))
    } else {
        Ok(value.into())
    }
}

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn record_dir(state: &AppState) -> PathBuf {
    decision_root(state).join(
        "controlled-shadow-first-natural-forward-cycle-market-data-parser-specification-reviews",
    )
}

struct ReviewLock(PathBuf);

impl Drop for ReviewLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

async fn acquire_lock(state: &AppState, registration_id: &str) -> Result<ReviewLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| format!("创建 Stage 96 锁目录失败: {error}"))?;
    let path = directory.join(format!(
        "market-data-parser-spec-review-{registration_id}.lock"
    ));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|value| SystemTime::now().duration_since(value).ok())
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
        .map_err(|_| "已有同一 Stage 96 规格复核正在进行".to_string())?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ReviewControlledShadowMarketDataParserSpecificationRequest {
        ReviewControlledShadowMarketDataParserSpecificationRequest {
            expected_registration_sha256: "1".repeat(64),
            expected_parser_specification_sha256: "2".repeat(64),
            expected_validation_sha256: "3".repeat(64),
            expected_receipt_sha256: "4".repeat(64),
            expected_claim_sha256: "5".repeat(64),
            expected_result_sha256: "6".repeat(64),
            expected_adapter_authorization_sha256: "7".repeat(64),
            expected_adapter_spec_sha256: "8".repeat(64),
            expected_canonical_request_set_sha256: "9".repeat(64),
            verdict: ControlledShadowMarketDataParserSpecificationReviewVerdict::ApprovedForFutureZeroCapabilityParserImplementationRegistration,
            rationale: "independent review".into(),
            source_contract_assessment: "explicit stable endpoints".into(),
            schema_and_numeric_assessment: "strict schemas".into(),
            calendar_and_reconciliation_assessment: "calendar and SPY synchronized".into(),
            synthetic_vector_assessment: "all vectors independently reconstructed".into(),
            failure_and_missing_data_assessment: "fail closed".into(),
            known_limitations: "no real provider semantics verified".into(),
            future_implementation_constraints: "zero capability implementation registration only".into(),
            exact_stage_51_through_stage_95_binding_confirmed: true,
            reviewer_independent_from_registrar_validator_executor_and_complete_prior_chain_confirmed: true,
            independent_recomputation_of_validation_claim_result_receipt_registration_and_specification_confirmed: true,
            independent_reconstruction_of_explicit_price_dividend_split_and_calendar_requests_confirmed: true,
            independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed: true,
            strict_utf8_json_html_date_and_bounded_numeric_rules_reviewed: true,
            duplicate_out_of_window_missing_and_malformed_fail_closed_rules_reviewed: true,
            no_deduplication_forward_fill_interpolation_fallback_or_inferred_actions_reviewed: true,
            separate_price_series_explicit_actions_and_cross_source_reconciliation_reviewed: true,
            spy_official_calendar_coverage_and_explicit_subject_gap_rules_reviewed: true,
            source_available_at_remains_unverified_until_separate_review_confirmed: true,
            specification_only_no_parser_artifact_entrypoint_runtime_or_raw_payload_access_confirmed: true,
            approval_only_opens_future_zero_capability_parser_implementation_registration_confirmed: true,
            no_parsed_rows_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: true,
            no_unconfirmed_hari_or_old_wang_logic_claimed: true,
        }
    }

    #[test]
    fn request_requires_every_independent_review_confirmation() {
        let mut value = request();
        assert!(validate_request(&"a".repeat(32), &value).is_ok());
        value
            .independent_reconstruction_of_all_synthetic_vector_input_and_output_hashes_confirmed =
            false;
        assert!(validate_request(&"a".repeat(32), &value).is_err());
    }

    #[test]
    fn second_request_builder_requires_explicit_actions_and_rejects_legacy_route() {
        let values = independently_reconstruct_canonical_requests(
            &["SNDK".into()],
            chrono::NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 8, 28).unwrap(),
        )
        .unwrap();
        assert_eq!(values.len(), 11);
        let json = serde_json::to_string(&values).unwrap();
        assert!(json.contains("/stable/dividends"));
        assert!(json.contains("/stable/splits"));
        assert!(json.contains("/stable/historical-price-eod/non-split-adjusted"));
        assert!(json.contains("/stable/historical-price-eod/dividend-adjusted"));
        assert!(!json.contains("/v3/historical-price-full"));
    }

    #[test]
    fn second_vector_builder_recomputes_all_hashes_without_market_truth() {
        let values = independently_reconstruct_synthetic_vectors();
        assert_eq!(values.len(), 8);
        assert!(values.iter().all(|value| {
            value.synthetic_only_no_market_truth
                && value.input_fixture_sha256 == sha256_bytes(value.input_fixture.as_bytes())
                && value.expected_canonical_output_sha256
                    == value
                        .expected_canonical_output
                        .as_ref()
                        .map(|output| sha256_bytes(output.as_bytes()))
        }));
    }

    #[test]
    fn review_authority_is_fully_closed() {
        let value = closed_review_authority_boundary();
        assert!(!value.parser_implementation_registered);
        assert!(!value.parser_implementation_present);
        assert!(!value.raw_payload_read_or_mount_allowed);
        assert!(!value.parsed_market_rows_created);
        assert!(!value.observation_write_allowed);
        assert!(!value.training_feedback_or_reward_allowed);
        assert!(!value.order_generation_or_broker_access_allowed);
        assert!(!value.trading_allowed);
    }
}
