//! Stage 104 independent admission review for one exact Stage 103 validated parser output.
//!
//! Provider publication time is not available in the retained evidence. This gate therefore uses
//! the latest custody retrieval, parser completion, independent validation and admission-review
//! timestamps as a conservative point-in-time availability floor. Approval only admits the exact
//! input for a later create-once observation-materialization specification; it does not start an
//! observation or create a ledger, position, performance, model, reward, order, broker or trade.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::America::New_York;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_claims::{
    ControlledShadowFirstNaturalForwardCycleClaim,
    claimed_first_natural_forward_cycles_for_market_data_adapter_review,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_execution_attempts::{
    UntrustedCanonicalMarketDataParseResult,
    controlled_shadow_market_data_parser_execution_custody_root,
};
use super::historical_outcome_offline_dataset_feature_label_join_target_training_sealed_holdout_evaluation_controlled_shadow_experiment_forward_observation_first_natural_forward_cycle_market_data_parser_output_validations::{
    IndependentlyValidatedMarketDataParserOutput,
    independently_validated_market_data_parser_outputs_for_observation_input_review,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-controlled-shadow-first-natural-forward-cycle-observation-input-admission-review-registry-v1";
const REVIEW_SCHEMA_VERSION: &str =
    "hone-controlled-shadow-first-natural-forward-cycle-observation-input-admission-review-v1";
const POLICY_VERSION: &str = "hone-first-natural-forward-cycle-observation-input-independent-admission-v1-custody-time-floor-no-observation";
const NEXT_GATE: &str =
    "stage_105_first_natural_forward_cycle_observation_materialization_specification_registration";
const MAX_TEXT_CHARS: usize = 4_000;
const MAX_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;
const AVAILABILITY_BASIS: &str =
    "max_custody_retrieval_parser_completion_independent_validation_and_admission_review_time";
const PROVIDER_TIME_LIMITATION: &str =
    "provider_publication_time_unverified_custody_retrieval_time_used_as_conservative_floor";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ControlledShadowObservationInputAdmissionVerdict {
    ApprovedForFutureCreateOnceObservationMaterializationSpecificationRegistration,
    ChangesRequested,
    Rejected,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReviewControlledShadowObservationInputAdmissionRequest {
    expected_previous_review_id: Option<String>,
    expected_previous_review_sha256: Option<String>,
    expected_stage_103_validation_id: String,
    expected_stage_103_validation_sha256: String,
    expected_stage_102_result_sha256: String,
    expected_stage_102_output_sha256: String,
    expected_stage_101_claim_sha256: String,
    expected_stage_101_input_manifest_sha256: String,
    expected_cycle_claim_sha256: String,
    verdict: ControlledShadowObservationInputAdmissionVerdict,
    rationale: String,
    known_limitations: String,
    exact_current_stage_51_through_stage_103_binding_confirmed: bool,
    reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed: bool,
    stage_103_full_reparse_validation_current_and_passed_confirmed: bool,
    cycle_claim_natural_forward_only_and_no_backfill_confirmed: bool,
    fixed_subject_spy_window_and_request_identities_confirmed: bool,
    every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed: bool,
    custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed:
        bool,
    admitted_rows_within_frozen_window_and_available_before_admission_confirmed: bool,
    official_sessions_and_spy_three_price_bases_complete_confirmed: bool,
    subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed: bool,
    dividends_splits_and_three_price_bases_remain_separate_confirmed: bool,
    exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed: bool,
    approval_only_opens_future_materialization_specification_registration_confirmed: bool,
    no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationInputAdmissionCandidate {
    pub cycle_claim: ControlledShadowFirstNaturalForwardCycleClaim,
    pub parser_output: IndependentlyValidatedMarketDataParserOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ControlledShadowObservationInputAdmissionReview {
    pub schema_version: String,
    pub policy_version: String,
    pub review_id: String,
    pub review_sha256: String,
    pub previous_review_id: Option<String>,
    pub previous_review_sha256: Option<String>,
    pub stage_102_attempt_id: String,
    pub stage_101_claim_sha256: String,
    pub stage_101_input_manifest_sha256: String,
    pub stage_102_result_id: String,
    pub stage_102_result_sha256: String,
    pub stage_102_output_sha256: String,
    pub stage_103_validation_id: String,
    pub stage_103_validation_sha256: String,
    pub cycle_claim_id: String,
    pub cycle_claim_sha256: String,
    pub subject_symbols: Vec<String>,
    pub benchmark_symbol: String,
    pub window_start_date: NaiveDate,
    pub window_end_date: NaiveDate,
    pub subject_symbol_set_sha256: String,
    pub time_window_sha256: String,
    pub canonical_request_set_sha256: String,
    pub raw_payload_custody_manifest_sha256: String,
    pub source_receipt_count: usize,
    pub latest_source_retrieved_at_utc: DateTime<Utc>,
    pub parser_completed_at_utc: DateTime<Utc>,
    pub independently_validated_at_utc: DateTime<Utc>,
    pub submitted_at: DateTime<Utc>,
    pub admitted_available_at_utc: DateTime<Utc>,
    pub availability_basis: String,
    pub provider_publication_time_limitation: String,
    pub provider_publication_time_verified: bool,
    pub custody_retrieval_time_verified: bool,
    pub official_market_session_count: usize,
    pub price_row_count: usize,
    pub dividend_row_count: usize,
    pub split_row_count: usize,
    pub explicit_gap_count: usize,
    pub earliest_market_session_date: NaiveDate,
    pub latest_market_session_date: NaiveDate,
    pub submitted_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub verdict: ControlledShadowObservationInputAdmissionVerdict,
    pub rationale: String,
    pub known_limitations: String,
    pub reviewer_independent_from_validator_executor_and_complete_prior_chain: bool,
    pub exact_current_stage_51_through_stage_103_binding_confirmed: bool,
    pub stage_103_full_reparse_validation_current_and_passed_confirmed: bool,
    pub cycle_claim_natural_forward_only_and_no_backfill_confirmed: bool,
    pub fixed_subject_spy_window_and_request_identities_confirmed: bool,
    pub every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed: bool,
    pub custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed:
        bool,
    pub admitted_rows_within_frozen_window_and_available_before_admission_confirmed: bool,
    pub official_sessions_and_spy_three_price_bases_complete_confirmed: bool,
    pub subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed: bool,
    pub dividends_splits_and_three_price_bases_remain_separate_confirmed: bool,
    pub exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed: bool,
    pub approval_only_opens_future_materialization_specification_registration_confirmed: bool,
    pub no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
        bool,
    pub no_unconfirmed_hari_or_old_wang_logic_claimed: bool,
    pub structural_input_audit_passed: bool,
    pub observation_input_admitted: bool,
    pub future_create_once_observation_materialization_specification_registration_eligible: bool,
    pub observation_materialization_specification_registered: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub model_or_metric_store_written: bool,
    pub training_feedback_authorized: bool,
    pub reward_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationInputAdmissionItem {
    pub candidate: ControlledShadowObservationInputAdmissionCandidate,
    pub latest_review: Option<ControlledShadowObservationInputAdmissionReview>,
    pub current_binding: bool,
    pub review_eligible: bool,
    pub observation_input_admitted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ControlledShadowObservationInputAdmissionRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub items: Vec<ControlledShadowObservationInputAdmissionItem>,
    pub independently_validated_input_candidate_count: usize,
    pub review_eligible_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_input_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_observation_materialization_specification_registration_eligible_count: usize,
    pub admission_status: String,
    pub next_gate: String,
    pub admission_review_available: bool,
    pub provider_publication_time_verified: bool,
    pub custody_retrieval_time_floor_required: bool,
    pub forward_observation_started: bool,
    pub ledger_created: bool,
    pub position_written: bool,
    pub performance_metric_written: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ControlledShadowObservationInputAdmissionReadinessSummary {
    pub independently_validated_input_candidate_count: usize,
    pub review_eligible_candidate_count: usize,
    pub reviewed_candidate_count: usize,
    pub admitted_input_count: usize,
    pub changes_requested_or_rejected_count: usize,
    pub future_observation_materialization_specification_registration_eligible_count: usize,
    pub admission_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdmittedControlledShadowObservationInputForMaterializationSpecification {
    pub candidate: ControlledShadowObservationInputAdmissionCandidate,
    pub admission_review: ControlledShadowObservationInputAdmissionReview,
}

#[derive(Debug, Clone)]
pub(crate) struct RevalidatedAdmittedControlledShadowObservationInputForMaterializationExecution {
    pub admitted: AdmittedControlledShadowObservationInputForMaterializationSpecification,
    pub exact_output: UntrustedCanonicalMarketDataParseResult,
}

pub(crate) async fn admitted_controlled_shadow_observation_inputs_for_materialization_specification(
    state: &AppState,
) -> Result<Vec<AdmittedControlledShadowObservationInputForMaterializationSpecification>, String> {
    let registry = build_registry(state).await?;
    let mut admitted = registry
        .items
        .into_iter()
        .filter_map(|item| {
            let review = item.latest_review?;
            (item.current_binding
                && item.observation_input_admitted
                && review.observation_input_admitted
                && review
                    .future_create_once_observation_materialization_specification_registration_eligible)
                .then_some(AdmittedControlledShadowObservationInputForMaterializationSpecification {
                    candidate: item.candidate,
                    admission_review: review,
                })
        })
        .collect::<Vec<_>>();
    admitted.sort_by(|left, right| {
        left.admission_review
            .submitted_at
            .cmp(&right.admission_review.submitted_at)
    });
    Ok(admitted)
}

/// Reopens and rehashes the exact Stage 102 output immediately before Stage 112. The lookup is
/// deliberately server-side and exact: callers cannot provide a path, symbol set, date window or
/// alternate payload. A previously admitted input that is no longer the current binding fails
/// closed rather than being reconstructed or substituted.
pub(crate) async fn read_revalidated_admitted_observation_input_for_materialization_execution(
    state: &AppState,
    expected_stage_104_review_sha256: &str,
    expected_stage_102_output_sha256: &str,
) -> Result<RevalidatedAdmittedControlledShadowObservationInputForMaterializationExecution, String>
{
    let admitted =
        admitted_controlled_shadow_observation_inputs_for_materialization_specification(state)
            .await?
            .into_iter()
            .find(|value| {
                value.admission_review.review_sha256 == expected_stage_104_review_sha256
                    && value.admission_review.stage_102_output_sha256
                        == expected_stage_102_output_sha256
            })
            .ok_or_else(|| {
                "Stage 112 找不到仍为当前绑定的 exact Stage 104 admitted input".to_string()
            })?;
    let exact_output = read_exact_output(state, &admitted.candidate).await?;
    validate_review(
        &admitted.admission_review,
        &admitted.candidate,
        &exact_output,
    )?;
    let audit = structural_input_audit(&admitted.candidate, &exact_output)?;
    if !audit.passed
        || !admitted.admission_review.observation_input_admitted
        || !admitted.admission_review.structural_input_audit_passed
        || exact_output.output_sha256 != expected_stage_102_output_sha256
    {
        return Err("Stage 112 exact admitted input 重哈希或结构复核失败".to_string());
    }
    Ok(
        RevalidatedAdmittedControlledShadowObservationInputForMaterializationExecution {
            admitted,
            exact_output,
        },
    )
}

#[derive(Debug)]
struct StructuralInputAudit {
    passed: bool,
    latest_source_retrieved_at_utc: DateTime<Utc>,
    official_market_session_count: usize,
    price_row_count: usize,
    dividend_row_count: usize,
    split_row_count: usize,
    explicit_gap_count: usize,
    earliest_market_session_date: NaiveDate,
    latest_market_session_date: NaiveDate,
}

pub(crate) async fn handle_get_controlled_shadow_observation_input_admission_reviews(
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
            warn!(%error, "observation-input admission registry failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "首个自然前向周期观察输入准入复核暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_review_controlled_shadow_observation_input_admission(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(attempt_id): AxumPath<String>,
    Json(request): Json<ReviewControlledShadowObservationInputAdmissionRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(value) => value,
            Err(response) => return response,
        };
    match review_admission(&state, &admin.user_id, &attempt_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => crate::routes::json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn controlled_shadow_observation_input_admission_readiness_summary(
    state: &AppState,
) -> Result<ControlledShadowObservationInputAdmissionReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(ControlledShadowObservationInputAdmissionReadinessSummary {
        independently_validated_input_candidate_count: registry
            .independently_validated_input_candidate_count,
        review_eligible_candidate_count: registry.review_eligible_candidate_count,
        reviewed_candidate_count: registry.reviewed_candidate_count,
        admitted_input_count: registry.admitted_input_count,
        changes_requested_or_rejected_count: registry.changes_requested_or_rejected_count,
        future_observation_materialization_specification_registration_eligible_count: registry
            .future_observation_materialization_specification_registration_eligible_count,
        admission_status: registry.admission_status,
    })
}

async fn build_candidates(
    state: &AppState,
) -> Result<Vec<ControlledShadowObservationInputAdmissionCandidate>, String> {
    let outputs =
        independently_validated_market_data_parser_outputs_for_observation_input_review(state)
            .await?;
    let cycle_claims = claimed_first_natural_forward_cycles_for_market_data_adapter_review(state)
        .await?
        .into_iter()
        .map(|value| (value.cycle_claim_sha256.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut candidates = Vec::with_capacity(outputs.len());
    for parser_output in outputs {
        let cycle_sha = &parser_output
            .claim
            .fixed_input_manifest
            .stage_93_claim
            .cycle_claim_sha256;
        let cycle_claim = cycle_claims
            .get(cycle_sha)
            .cloned()
            .ok_or_else(|| "Stage 104 candidate 缺少当前 Stage 91 cycle claim".to_string())?;
        candidates.push(ControlledShadowObservationInputAdmissionCandidate {
            cycle_claim,
            parser_output,
        });
    }
    candidates.sort_by(|left, right| {
        left.parser_output
            .claim
            .attempt_id
            .cmp(&right.parser_output.claim.attempt_id)
    });
    Ok(candidates)
}

async fn build_registry(
    state: &AppState,
) -> Result<ControlledShadowObservationInputAdmissionRegistry, String> {
    let candidates = build_candidates(state).await?;
    let mut items = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let latest_review = latest_review(state, &candidate).await?;
        let admitted = latest_review
            .as_ref()
            .is_some_and(|value| value.observation_input_admitted);
        items.push(ControlledShadowObservationInputAdmissionItem {
            current_binding: true,
            review_eligible: !admitted,
            observation_input_admitted: admitted,
            candidate,
            latest_review,
        });
    }
    let independently_validated_input_candidate_count = items.len();
    let review_eligible_candidate_count =
        items.iter().filter(|value| value.review_eligible).count();
    let reviewed_candidate_count = items
        .iter()
        .filter(|value| value.latest_review.is_some())
        .count();
    let admitted_input_count = items
        .iter()
        .filter(|value| value.observation_input_admitted)
        .count();
    let changes_requested_or_rejected_count = items
        .iter()
        .filter(|value| {
            value.latest_review.as_ref().is_some_and(|review| {
                review.verdict
                    != ControlledShadowObservationInputAdmissionVerdict::ApprovedForFutureCreateOnceObservationMaterializationSpecificationRegistration
            })
        })
        .count();
    let admission_status = if admitted_input_count > 0 {
        "observation_input_admitted_waiting_stage_105_materialization_specification_registration"
    } else if changes_requested_or_rejected_count > 0 {
        "observation_input_admission_changes_requested_or_rejected"
    } else if independently_validated_input_candidate_count > 0 {
        "waiting_independent_observation_input_admission_review"
    } else {
        "waiting_stage_103_independently_validated_parser_output"
    };
    Ok(ControlledShadowObservationInputAdmissionRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        items,
        independently_validated_input_candidate_count,
        review_eligible_candidate_count,
        reviewed_candidate_count,
        admitted_input_count,
        changes_requested_or_rejected_count,
        future_observation_materialization_specification_registration_eligible_count:
            admitted_input_count,
        admission_status: admission_status.to_string(),
        next_gate: NEXT_GATE.to_string(),
        admission_review_available: true,
        provider_publication_time_verified: false,
        custody_retrieval_time_floor_required: true,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "Stage 104 只接纳 Stage 103 第二实现全量重解析通过且当前绑定的输入。供应商发布时间仍未验证；准入只使用 HONE 实际保管取得、解析完成、独立校验和本次复核时间的最大值作为保守 available_at。至少一个官方交易日、SPY 三套价格完整、标的缺口显式和无回填条件全部成立后，批准也只开放 Stage 105 create-once 观察物化规格登记；不开始观察、不建账、不写持仓/绩效、不训练/reward、不生成订单、不接券商、不交易。".to_string(),
    })
}

async fn review_admission(
    state: &AppState,
    reviewer_id: &str,
    attempt_id: &str,
    request: ReviewControlledShadowObservationInputAdmissionRequest,
) -> Result<ControlledShadowObservationInputAdmissionReview, String> {
    if !valid_id(attempt_id) {
        return Err("Stage 102 attempt ID 无效".to_string());
    }
    validate_request_bindings(&request)?;
    let _lock = acquire_lock(state, attempt_id).await?;
    let candidate = build_candidates(state)
        .await?
        .into_iter()
        .find(|value| value.parser_output.claim.attempt_id == attempt_id)
        .ok_or_else(|| "当前没有精确绑定且 Stage 103 独立校验通过的观察输入候选".to_string())?;
    verify_expected_bindings(&candidate, &request)?;
    let latest = latest_review(state, &candidate).await?;
    if latest
        .as_ref()
        .is_some_and(|value| value.observation_input_admitted)
    {
        return Err("该观察输入已经准入，复核链永久冻结".to_string());
    }
    if latest.as_ref().map(|value| value.review_id.as_str())
        != request.expected_previous_review_id.as_deref()
        || latest.as_ref().map(|value| value.review_sha256.as_str())
            != request.expected_previous_review_sha256.as_deref()
    {
        return Err("Stage 104 准入复核链已经变化，请刷新后重试".to_string());
    }
    let excluded_prior_actor_ids = excluded_actor_ids(&candidate, &latest);
    let independent = reviewer_is_independent(reviewer_id, &excluded_prior_actor_ids);
    if !independent {
        return Err(
            "Stage 104 复核者不能是 Stage 103 validator、Stage 102 executor 或完整既有责任链角色"
                .to_string(),
        );
    }
    if !request.reviewer_independent_from_validator_executor_and_complete_prior_chain_confirmed {
        return Err(
            "Stage 104 复核者必须明确确认自己独立于 validator、executor 与完整既有责任链"
                .to_string(),
        );
    }
    let output = read_exact_output(state, &candidate).await?;
    let audit = structural_input_audit(&candidate, &output)?;
    let submitted_at = Utc::now();
    let admitted_available_at_utc = [
        audit.latest_source_retrieved_at_utc,
        candidate.parser_output.result.completed_at,
        candidate.parser_output.validation.validated_at,
        submitted_at,
    ]
    .into_iter()
    .max()
    .ok_or_else(|| "无法确定 Stage 104 保守可用时间".to_string())?;
    let manifest = &candidate.parser_output.claim.fixed_input_manifest;
    let mut review = ControlledShadowObservationInputAdmissionReview {
        schema_version: REVIEW_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        review_id: String::new(),
        review_sha256: String::new(),
        previous_review_id: latest.as_ref().map(|value| value.review_id.clone()),
        previous_review_sha256: latest.as_ref().map(|value| value.review_sha256.clone()),
        stage_102_attempt_id: candidate.parser_output.claim.attempt_id.clone(),
        stage_101_claim_sha256: candidate.parser_output.claim.claim_sha256.clone(),
        stage_101_input_manifest_sha256: manifest.input_manifest_sha256.clone(),
        stage_102_result_id: candidate.parser_output.result.result_id.clone(),
        stage_102_result_sha256: candidate.parser_output.result.result_sha256.clone(),
        stage_102_output_sha256: candidate
            .parser_output
            .result
            .output_sha256
            .clone()
            .unwrap_or_default(),
        stage_103_validation_id: candidate.parser_output.validation.validation_id.clone(),
        stage_103_validation_sha256: candidate.parser_output.validation.validation_sha256.clone(),
        cycle_claim_id: candidate.cycle_claim.cycle_claim_id.clone(),
        cycle_claim_sha256: candidate.cycle_claim.cycle_claim_sha256.clone(),
        subject_symbols: manifest.subject_symbols.clone(),
        benchmark_symbol: manifest.benchmark_symbol.clone(),
        window_start_date: manifest.window_start_date,
        window_end_date: manifest.window_end_date,
        subject_symbol_set_sha256: manifest.subject_symbol_set_sha256.clone(),
        time_window_sha256: manifest.time_window_sha256.clone(),
        canonical_request_set_sha256: manifest.canonical_request_set_sha256.clone(),
        raw_payload_custody_manifest_sha256: manifest.raw_payload_custody_manifest_sha256.clone(),
        source_receipt_count: manifest.raw_payload_count,
        latest_source_retrieved_at_utc: audit.latest_source_retrieved_at_utc,
        parser_completed_at_utc: candidate.parser_output.result.completed_at,
        independently_validated_at_utc: candidate.parser_output.validation.validated_at,
        submitted_at,
        admitted_available_at_utc,
        availability_basis: AVAILABILITY_BASIS.to_string(),
        provider_publication_time_limitation: PROVIDER_TIME_LIMITATION.to_string(),
        provider_publication_time_verified: false,
        custody_retrieval_time_verified: audit.passed,
        official_market_session_count: audit.official_market_session_count,
        price_row_count: audit.price_row_count,
        dividend_row_count: audit.dividend_row_count,
        split_row_count: audit.split_row_count,
        explicit_gap_count: audit.explicit_gap_count,
        earliest_market_session_date: audit.earliest_market_session_date,
        latest_market_session_date: audit.latest_market_session_date,
        submitted_by: reviewer_id.to_string(),
        excluded_prior_actor_ids,
        verdict: request.verdict,
        rationale: bounded_required(&request.rationale, "复核依据")?,
        known_limitations: bounded_required(&request.known_limitations, "已知局限")?,
        reviewer_independent_from_validator_executor_and_complete_prior_chain: independent,
        exact_current_stage_51_through_stage_103_binding_confirmed: request
            .exact_current_stage_51_through_stage_103_binding_confirmed,
        stage_103_full_reparse_validation_current_and_passed_confirmed: request
            .stage_103_full_reparse_validation_current_and_passed_confirmed,
        cycle_claim_natural_forward_only_and_no_backfill_confirmed: request
            .cycle_claim_natural_forward_only_and_no_backfill_confirmed,
        fixed_subject_spy_window_and_request_identities_confirmed: request
            .fixed_subject_spy_window_and_request_identities_confirmed,
        every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed: request
            .every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed,
        custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed:
            request.custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed,
        admitted_rows_within_frozen_window_and_available_before_admission_confirmed: request
            .admitted_rows_within_frozen_window_and_available_before_admission_confirmed,
        official_sessions_and_spy_three_price_bases_complete_confirmed: request
            .official_sessions_and_spy_three_price_bases_complete_confirmed,
        subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed: request
            .subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed,
        dividends_splits_and_three_price_bases_remain_separate_confirmed: request
            .dividends_splits_and_three_price_bases_remain_separate_confirmed,
        exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed: request
            .exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed,
        approval_only_opens_future_materialization_specification_registration_confirmed: request
            .approval_only_opens_future_materialization_specification_registration_confirmed,
        no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed:
            request.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed,
        no_unconfirmed_hari_or_old_wang_logic_claimed: request
            .no_unconfirmed_hari_or_old_wang_logic_claimed,
        structural_input_audit_passed: audit.passed,
        observation_input_admitted: false,
        future_create_once_observation_materialization_specification_registration_eligible: false,
        observation_materialization_specification_registered: false,
        forward_observation_started: false,
        ledger_created: false,
        position_written: false,
        performance_metric_written: false,
        model_or_metric_store_written: false,
        training_feedback_authorized: false,
        reward_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    let approved = review.verdict
        == ControlledShadowObservationInputAdmissionVerdict::ApprovedForFutureCreateOnceObservationMaterializationSpecificationRegistration;
    review.observation_input_admitted =
        approved && independent && audit.passed && review_all_checks(&review);
    review.future_create_once_observation_materialization_specification_registration_eligible =
        review.observation_input_admitted;
    review.review_sha256 = review_fingerprint(&review)?;
    review.review_id = review.review_sha256[..32].to_string();
    validate_review(&review, &candidate, &output)?;
    write_immutable_json(
        &review_directory(state, attempt_id).join(format!("{}.json", review.review_id)),
        &review,
    )
    .await?;
    Ok(review)
}

fn structural_input_audit(
    candidate: &ControlledShadowObservationInputAdmissionCandidate,
    output: &UntrustedCanonicalMarketDataParseResult,
) -> Result<StructuralInputAudit, String> {
    let manifest = &candidate.parser_output.claim.fixed_input_manifest;
    let validation = &candidate.parser_output.validation;
    let result = &candidate.parser_output.result;
    let raw = &manifest.raw_payloads;
    if raw.is_empty() || raw.len() != manifest.raw_payload_count {
        return Err("Stage 104 输入缺少固定 raw payload 时间证据".to_string());
    }
    let latest_source_retrieved_at_utc = raw
        .iter()
        .map(|value| value.retrieved_at_utc)
        .max()
        .ok_or_else(|| "Stage 104 无法确定最新保管取得时间".to_string())?;
    let timestamps_valid = raw.iter().all(|value| {
        value.source_available_at_basis == "retrieval_time_fallback_unverified"
            && value.source_available_at_utc == value.retrieved_at_utc
            && value.retrieved_at_utc >= manifest.stage_93_claim.claimed_at
            && value.retrieved_at_utc <= result.completed_at
    });
    let temporal_chain = candidate.cycle_claim.cycle_claim_sha256
        == manifest.stage_93_claim.cycle_claim_sha256
        && candidate.cycle_claim.claimed_at <= manifest.stage_93_claim.claimed_at
        && result.completed_at <= validation.validated_at
        && manifest.window_start_date
            >= candidate
                .cycle_claim
                .observation_eligibility_anchor
                .with_timezone(&New_York)
                .date_naive()
        && manifest.window_end_date
            <= latest_source_retrieved_at_utc
                .with_timezone(&New_York)
                .date_naive();
    let calendar_dates = output
        .calendar_rows
        .iter()
        .map(|value| value.date)
        .collect::<BTreeSet<_>>();
    if calendar_dates.is_empty() {
        return Err("Stage 104 至少需要一个官方市场交易日才能准入".to_string());
    }
    let earliest_market_session_date = *calendar_dates
        .first()
        .ok_or_else(|| "Stage 104 缺少最早交易日".to_string())?;
    let latest_market_session_date = *calendar_dates
        .last()
        .ok_or_else(|| "Stage 104 缺少最晚交易日".to_string())?;
    let calendar_valid = calendar_dates.len() == output.calendar_rows.len()
        && earliest_market_session_date >= manifest.window_start_date
        && latest_market_session_date <= manifest.window_end_date
        && output.calendar_rows.iter().all(|row| {
            matches!(row.session_kind.as_str(), "regular" | "early_close")
                && matches!(row.close_time_et.as_str(), "16:00" | "13:00")
        });
    let expected_symbols = manifest
        .subject_symbols
        .iter()
        .cloned()
        .chain(std::iter::once(manifest.benchmark_symbol.clone()))
        .collect::<BTreeSet<_>>();
    let kinds = [
        "split_adjusted_price",
        "raw_unadjusted_price",
        "dividend_adjusted_price",
    ];
    let mut price_keys = BTreeSet::new();
    let price_rows_valid = output.price_rows.iter().all(|row| {
        expected_symbols.contains(&row.symbol)
            && calendar_dates.contains(&row.date)
            && kinds.contains(&row.source_kind.as_str())
            && price_keys.insert((row.symbol.clone(), row.date, row.source_kind.clone()))
    });
    let mut gap_keys = BTreeSet::new();
    let gaps_valid = output.explicit_gaps.iter().all(|gap| {
        manifest.subject_symbols.contains(&gap.symbol)
            && calendar_dates.contains(&gap.date)
            && kinds.contains(&gap.source_kind.as_str())
            && gap.reason == "missing_subject_row_no_fill"
            && gap_keys.insert((gap.symbol.clone(), gap.date, gap.source_kind.clone()))
            && !price_keys.contains(&(gap.symbol.clone(), gap.date, gap.source_kind.clone()))
    });
    let complete_subject_matrix = manifest.subject_symbols.iter().all(|symbol| {
        calendar_dates.iter().all(|date| {
            kinds.iter().all(|kind| {
                let key = (symbol.clone(), *date, (*kind).to_string());
                price_keys.contains(&key) ^ gap_keys.contains(&key)
            })
        })
    });
    let benchmark_complete = calendar_dates.iter().all(|date| {
        kinds.iter().all(|kind| {
            price_keys.contains(&(
                manifest.benchmark_symbol.clone(),
                *date,
                (*kind).to_string(),
            ))
        })
    });
    let actions_valid = output.dividend_rows.iter().all(|row| {
        expected_symbols.contains(&row.symbol)
            && row.date >= manifest.window_start_date
            && row.date <= manifest.window_end_date
    }) && output.split_rows.iter().all(|row| {
        expected_symbols.contains(&row.symbol)
            && row.date >= manifest.window_start_date
            && row.date <= manifest.window_end_date
    });
    let exact_current_validation = validation.canonical_parse_output_independently_validated
        && validation.future_observation_input_admission_review_eligible
        && validation.complete_output_exact_match_verified
        && !validation.source_available_at_verified
        && validation.mismatch_reasons.is_empty()
        && output.output_sha256 == validation.stage_102_output_sha256
        && output.untrusted
        && !output.independent_validation_completed
        && !output.observation_started
        && !output.ledger_created
        && !output.position_written
        && !output.order_generation_authorized
        && !output.broker_access_authorized
        && !output.trading_authorized
        && output.rejections.is_empty();
    let passed = timestamps_valid
        && temporal_chain
        && calendar_valid
        && price_rows_valid
        && gaps_valid
        && complete_subject_matrix
        && benchmark_complete
        && actions_valid
        && exact_current_validation;
    Ok(StructuralInputAudit {
        passed,
        latest_source_retrieved_at_utc,
        official_market_session_count: output.calendar_rows.len(),
        price_row_count: output.price_rows.len(),
        dividend_row_count: output.dividend_rows.len(),
        split_row_count: output.split_rows.len(),
        explicit_gap_count: output.explicit_gaps.len(),
        earliest_market_session_date,
        latest_market_session_date,
    })
}

async fn read_exact_output(
    state: &AppState,
    candidate: &ControlledShadowObservationInputAdmissionCandidate,
) -> Result<UntrustedCanonicalMarketDataParseResult, String> {
    let result = &candidate.parser_output.result;
    let expected_sha = result
        .output_sha256
        .as_ref()
        .ok_or_else(|| "Stage 104 candidate 缺少 output SHA-256".to_string())?;
    let relative = result
        .output_relative_path
        .as_ref()
        .ok_or_else(|| "Stage 104 candidate 缺少 output 路径".to_string())?;
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|value| {
            matches!(
                value,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
        || relative != &format!("outputs/{expected_sha}.json")
    {
        return Err("Stage 104 output 路径越界或与摘要不一致".to_string());
    }
    let path = controlled_shadow_market_data_parser_execution_custody_root(state).join(relative);
    let metadata = tokio::fs::symlink_metadata(&path)
        .await
        .map_err(|error| error.to_string())?;
    if !metadata.file_type().is_file()
        || metadata.file_type().is_symlink()
        || metadata.len() == 0
        || metadata.len() > MAX_OUTPUT_BYTES
    {
        return Err("Stage 104 output 不是有界常规文件".to_string());
    }
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|error| error.to_string())?;
    let output: UntrustedCanonicalMarketDataParseResult =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if output.output_sha256 != *expected_sha
        || fingerprint_without(&output, &["output_sha256"])? != *expected_sha
    {
        return Err("Stage 104 output 内容或摘要漂移".to_string());
    }
    Ok(output)
}

fn review_all_checks(review: &ControlledShadowObservationInputAdmissionReview) -> bool {
    review.exact_current_stage_51_through_stage_103_binding_confirmed
        && review.reviewer_independent_from_validator_executor_and_complete_prior_chain
        && review.stage_103_full_reparse_validation_current_and_passed_confirmed
        && review.cycle_claim_natural_forward_only_and_no_backfill_confirmed
        && review.fixed_subject_spy_window_and_request_identities_confirmed
        && review.every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed
        && review.custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed
        && review.admitted_rows_within_frozen_window_and_available_before_admission_confirmed
        && review.official_sessions_and_spy_three_price_bases_complete_confirmed
        && review.subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed
        && review.dividends_splits_and_three_price_bases_remain_separate_confirmed
        && review.exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed
        && review.approval_only_opens_future_materialization_specification_registration_confirmed
        && review.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed
        && review.no_unconfirmed_hari_or_old_wang_logic_claimed
}

fn verify_expected_bindings(
    candidate: &ControlledShadowObservationInputAdmissionCandidate,
    request: &ReviewControlledShadowObservationInputAdmissionRequest,
) -> Result<(), String> {
    let parser = &candidate.parser_output;
    let manifest = &parser.claim.fixed_input_manifest;
    let valid = request.expected_stage_103_validation_id == parser.validation.validation_id
        && request.expected_stage_103_validation_sha256 == parser.validation.validation_sha256
        && request.expected_stage_102_result_sha256 == parser.result.result_sha256
        && parser.result.output_sha256.as_deref()
            == Some(request.expected_stage_102_output_sha256.as_str())
        && request.expected_stage_101_claim_sha256 == parser.claim.claim_sha256
        && request.expected_stage_101_input_manifest_sha256 == manifest.input_manifest_sha256
        && request.expected_cycle_claim_sha256 == candidate.cycle_claim.cycle_claim_sha256
        && request.expected_cycle_claim_sha256 == manifest.stage_93_claim.cycle_claim_sha256;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 104 预期绑定与当前 Stage 91/101/102/103 链不一致".to_string())
}

fn validate_request_bindings(
    request: &ReviewControlledShadowObservationInputAdmissionRequest,
) -> Result<(), String> {
    let pair = request.expected_previous_review_id.is_some()
        == request.expected_previous_review_sha256.is_some();
    let valid = pair
        && request
            .expected_previous_review_id
            .as_deref()
            .map_or(true, valid_id)
        && request
            .expected_previous_review_sha256
            .as_deref()
            .map_or(true, valid_sha256)
        && valid_id(&request.expected_stage_103_validation_id)
        && [
            &request.expected_stage_103_validation_sha256,
            &request.expected_stage_102_result_sha256,
            &request.expected_stage_102_output_sha256,
            &request.expected_stage_101_claim_sha256,
            &request.expected_stage_101_input_manifest_sha256,
            &request.expected_cycle_claim_sha256,
        ]
        .into_iter()
        .all(|value| valid_sha256(value));
    valid
        .then_some(())
        .ok_or_else(|| "Stage 104 请求绑定格式无效".to_string())
}

fn excluded_actor_ids(
    candidate: &ControlledShadowObservationInputAdmissionCandidate,
    latest: &Option<ControlledShadowObservationInputAdmissionReview>,
) -> Vec<String> {
    let parser = &candidate.parser_output;
    let mut values = parser.validation.excluded_prior_actor_ids.clone();
    values.push(parser.validation.validated_by.clone());
    values.push(parser.result.executed_by.clone());
    values.push(parser.claim.claimed_by.clone());
    values.extend(candidate.cycle_claim.excluded_prior_actor_ids.clone());
    values.push(candidate.cycle_claim.claimed_by.clone());
    if let Some(review) = latest {
        values.extend(review.excluded_prior_actor_ids.clone());
        values.push(review.submitted_by.clone());
    }
    values.retain(|value| !value.trim().is_empty());
    values.sort();
    values.dedup();
    values
}

fn reviewer_is_independent(reviewer_id: &str, excluded: &[String]) -> bool {
    !reviewer_id.trim().is_empty() && excluded.iter().all(|value| value != reviewer_id)
}

fn validate_review(
    review: &ControlledShadowObservationInputAdmissionReview,
    candidate: &ControlledShadowObservationInputAdmissionCandidate,
    output: &UntrustedCanonicalMarketDataParseResult,
) -> Result<(), String> {
    let parser = &candidate.parser_output;
    let manifest = &parser.claim.fixed_input_manifest;
    let approved = review.verdict
        == ControlledShadowObservationInputAdmissionVerdict::ApprovedForFutureCreateOnceObservationMaterializationSpecificationRegistration;
    let independent =
        reviewer_is_independent(&review.submitted_by, &review.excluded_prior_actor_ids);
    let audit = structural_input_audit(candidate, output)?;
    let expected_admitted = approved
        && independent
        && review.structural_input_audit_passed
        && review_all_checks(review);
    let downstream_closed = !review.observation_materialization_specification_registered
        && !review.forward_observation_started
        && !review.ledger_created
        && !review.position_written
        && !review.performance_metric_written
        && !review.model_or_metric_store_written
        && !review.training_feedback_authorized
        && !review.reward_authorized
        && !review.order_generation_authorized
        && !review.broker_access_authorized
        && !review.trading_authorized;
    let valid = review.schema_version == REVIEW_SCHEMA_VERSION
        && review.policy_version == POLICY_VERSION
        && valid_id(&review.review_id)
        && valid_sha256(&review.review_sha256)
        && review.review_id == review.review_sha256[..32]
        && review.review_sha256 == review_fingerprint(review)?
        && review.previous_review_id.is_some() == review.previous_review_sha256.is_some()
        && review.previous_review_id.as_deref().map_or(true, valid_id)
        && review
            .previous_review_sha256
            .as_deref()
            .map_or(true, valid_sha256)
        && review.stage_102_attempt_id == parser.claim.attempt_id
        && review.stage_101_claim_sha256 == parser.claim.claim_sha256
        && review.stage_101_input_manifest_sha256 == manifest.input_manifest_sha256
        && review.stage_102_result_id == parser.result.result_id
        && review.stage_102_result_sha256 == parser.result.result_sha256
        && parser.result.output_sha256.as_deref() == Some(review.stage_102_output_sha256.as_str())
        && review.stage_103_validation_id == parser.validation.validation_id
        && review.stage_103_validation_sha256 == parser.validation.validation_sha256
        && review.cycle_claim_id == candidate.cycle_claim.cycle_claim_id
        && review.cycle_claim_sha256 == candidate.cycle_claim.cycle_claim_sha256
        && review.subject_symbols == manifest.subject_symbols
        && review.benchmark_symbol == manifest.benchmark_symbol
        && review.window_start_date == manifest.window_start_date
        && review.window_end_date == manifest.window_end_date
        && review.subject_symbol_set_sha256 == manifest.subject_symbol_set_sha256
        && review.time_window_sha256 == manifest.time_window_sha256
        && review.canonical_request_set_sha256 == manifest.canonical_request_set_sha256
        && review.raw_payload_custody_manifest_sha256
            == manifest.raw_payload_custody_manifest_sha256
        && review.source_receipt_count == manifest.raw_payload_count
        && review.parser_completed_at_utc == parser.result.completed_at
        && review.independently_validated_at_utc == parser.validation.validated_at
        && review.admitted_available_at_utc >= review.latest_source_retrieved_at_utc
        && review.admitted_available_at_utc >= review.parser_completed_at_utc
        && review.admitted_available_at_utc >= review.independently_validated_at_utc
        && review.admitted_available_at_utc >= review.submitted_at
        && review.availability_basis == AVAILABILITY_BASIS
        && review.provider_publication_time_limitation == PROVIDER_TIME_LIMITATION
        && !review.provider_publication_time_verified
        && review.custody_retrieval_time_verified == review.structural_input_audit_passed
        && review.structural_input_audit_passed == audit.passed
        && review.latest_source_retrieved_at_utc == audit.latest_source_retrieved_at_utc
        && review.official_market_session_count == audit.official_market_session_count
        && review.price_row_count == audit.price_row_count
        && review.dividend_row_count == audit.dividend_row_count
        && review.split_row_count == audit.split_row_count
        && review.explicit_gap_count == audit.explicit_gap_count
        && review.earliest_market_session_date == audit.earliest_market_session_date
        && review.latest_market_session_date == audit.latest_market_session_date
        && !review.submitted_by.trim().is_empty()
        && !review.excluded_prior_actor_ids.is_empty()
        && sorted_unique(&review.excluded_prior_actor_ids)
        && !review.rationale.trim().is_empty()
        && review.rationale.chars().count() <= MAX_TEXT_CHARS
        && !review.known_limitations.trim().is_empty()
        && review.known_limitations.chars().count() <= MAX_TEXT_CHARS
        && independent
        && review.reviewer_independent_from_validator_executor_and_complete_prior_chain
            == independent
        && (!approved || audit.passed)
        && (!approved || review_all_checks(review))
        && review.observation_input_admitted == expected_admitted
        && review
            .future_create_once_observation_materialization_specification_registration_eligible
            == expected_admitted
        && downstream_closed;
    valid
        .then_some(())
        .ok_or_else(|| "Stage 104 观察输入准入复核记录无效、漂移或越权".to_string())
}

async fn latest_review(
    state: &AppState,
    candidate: &ControlledShadowObservationInputAdmissionCandidate,
) -> Result<Option<ControlledShadowObservationInputAdmissionReview>, String> {
    let directory = review_directory(state, &candidate.parser_output.claim.attempt_id);
    let mut entries = match tokio::fs::read_dir(&directory).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let output = read_exact_output(state, candidate).await?;
    let mut reviews = Vec::new();
    let mut ids = BTreeSet::new();
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let metadata = tokio::fs::symlink_metadata(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink()
            || !metadata.file_type().is_file()
            || metadata.len() == 0
            || metadata.len() > 256 * 1024
        {
            return Err("Stage 104 review 文件无效或超限".to_string());
        }
        let bytes = tokio::fs::read(entry.path())
            .await
            .map_err(|error| error.to_string())?;
        let review: ControlledShadowObservationInputAdmissionReview =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_review(&review, candidate, &output)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(review.review_id.as_str())
            || !ids.insert(review.review_id.clone())
        {
            return Err("Stage 104 review 文件名或唯一性无效".to_string());
        }
        reviews.push(review);
    }
    latest_review_from_records(&reviews)
}

fn latest_review_from_records(
    reviews: &[ControlledShadowObservationInputAdmissionReview],
) -> Result<Option<ControlledShadowObservationInputAdmissionReview>, String> {
    if reviews.is_empty() {
        return Ok(None);
    }
    let by_id = reviews
        .iter()
        .map(|value| (value.review_id.as_str(), value))
        .collect::<HashMap<_, _>>();
    let tips = reviews
        .iter()
        .filter(|value| {
            !reviews
                .iter()
                .any(|other| other.previous_review_id.as_deref() == Some(value.review_id.as_str()))
        })
        .collect::<Vec<_>>();
    if tips.len() != 1 {
        return Err("Stage 104 review chain 分叉或缺少唯一链尖".to_string());
    }
    let mut cursor = Some(tips[0]);
    let mut visited = BTreeSet::new();
    while let Some(review) = cursor {
        if !visited.insert(review.review_id.clone()) {
            return Err("Stage 104 review chain 存在循环".to_string());
        }
        cursor = match (&review.previous_review_id, &review.previous_review_sha256) {
            (Some(id), Some(sha)) => {
                let previous = by_id
                    .get(id.as_str())
                    .ok_or_else(|| "Stage 104 review chain 断裂".to_string())?;
                if previous.review_sha256 != *sha {
                    return Err("Stage 104 previous review SHA-256 不匹配".to_string());
                }
                Some(*previous)
            }
            (None, None) => None,
            _ => return Err("Stage 104 previous review 绑定不完整".to_string()),
        };
    }
    if visited.len() != reviews.len() {
        return Err("Stage 104 review chain 含游离记录".to_string());
    }
    Ok(Some(tips[0].clone()))
}

fn review_fingerprint(
    review: &ControlledShadowObservationInputAdmissionReview,
) -> Result<String, String> {
    fingerprint_without(review, &["review_id", "review_sha256"])
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Stage 104 fingerprint payload 无效".to_string())?;
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
        Err(format!("{label}为空或超过 {MAX_TEXT_CHARS} 字符"))
    } else {
        Ok(value.to_string())
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

async fn write_immutable_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
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

fn decision_root(state: &AppState) -> PathBuf {
    Path::new(&state.core.config.storage.session_sqlite_db_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("investment_decisions")
}

fn review_directory(state: &AppState, attempt_id: &str) -> PathBuf {
    decision_root(state)
        .join("controlled-shadow-first-natural-forward-cycle-observation-input-admission-reviews")
        .join(attempt_id)
}

struct AdmissionLock {
    path: PathBuf,
}

impl Drop for AdmissionLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, attempt_id: &str) -> Result<AdmissionLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!("stage104-{attempt_id}.lock"));
    if let Ok(metadata) = tokio::fs::metadata(&path).await {
        let stale = metadata
            .modified()
            .ok()
            .and_then(|value| SystemTime::now().duration_since(value).ok())
            .is_some_and(|value| value > StdDuration::from_secs(300));
        if stale {
            let _ = tokio::fs::remove_file(&path).await;
        }
    }
    tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .await
        .map_err(|_| "已有同一 Stage 104 准入复核正在进行".to_string())?;
    Ok(AdmissionLock { path })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conservative_available_at_uses_latest_gate_time() {
        let one = "2099-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let two = "2099-01-02T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let three = "2099-01-03T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let four = "2099-01-04T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        assert_eq!([one, two, three, four].into_iter().max(), Some(four));
        assert!(PROVIDER_TIME_LIMITATION.contains("provider_publication_time_unverified"));
    }

    #[test]
    fn provider_publication_time_is_never_claimed_verified() {
        assert_ne!(AVAILABILITY_BASIS, "provider_publication_time");
        assert!(AVAILABILITY_BASIS.contains("custody_retrieval"));
    }

    #[test]
    fn output_relative_path_rejects_escape() {
        let path = Path::new("../outputs/a.json");
        assert!(
            path.components()
                .any(|value| matches!(value, Component::ParentDir))
        );
        assert!(!Path::new("outputs/a.json").is_absolute());
    }

    #[test]
    fn reviewer_must_be_outside_complete_actor_set() {
        let actors = vec!["executor".to_string(), "validator".to_string()];
        assert!(!reviewer_is_independent("validator", &actors));
        assert!(reviewer_is_independent("stage104-reviewer", &actors));
    }

    #[test]
    fn review_chain_rejects_two_tips() {
        let mut one = sample_review("a", None);
        let mut two = sample_review("b", None);
        one.review_sha256 = "1".repeat(64);
        one.review_id = one.review_sha256[..32].to_string();
        two.review_sha256 = "2".repeat(64);
        two.review_id = two.review_sha256[..32].to_string();
        assert!(latest_review_from_records(&[one, two]).is_err());
    }

    #[test]
    fn approval_requires_all_explicit_checks() {
        let mut review = sample_review("reviewer", None);
        review.verdict = ControlledShadowObservationInputAdmissionVerdict::ApprovedForFutureCreateOnceObservationMaterializationSpecificationRegistration;
        assert!(!review_all_checks(&review));
        review.exact_current_stage_51_through_stage_103_binding_confirmed = true;
        review.reviewer_independent_from_validator_executor_and_complete_prior_chain = true;
        review.stage_103_full_reparse_validation_current_and_passed_confirmed = true;
        review.cycle_claim_natural_forward_only_and_no_backfill_confirmed = true;
        review.fixed_subject_spy_window_and_request_identities_confirmed = true;
        review.every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed = true;
        review.custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed = true;
        review.admitted_rows_within_frozen_window_and_available_before_admission_confirmed = true;
        review.official_sessions_and_spy_three_price_bases_complete_confirmed = true;
        review.subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed = true;
        review.dividends_splits_and_three_price_bases_remain_separate_confirmed = true;
        review.exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed = true;
        review.approval_only_opens_future_materialization_specification_registration_confirmed =
            true;
        review.no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed = true;
        review.no_unconfirmed_hari_or_old_wang_logic_claimed = true;
        assert!(review_all_checks(&review));
    }

    fn sample_review(
        actor: &str,
        previous: Option<(String, String)>,
    ) -> ControlledShadowObservationInputAdmissionReview {
        let time = "2099-01-04T00:00:00Z".parse().unwrap();
        ControlledShadowObservationInputAdmissionReview {
            schema_version: REVIEW_SCHEMA_VERSION.into(), policy_version: POLICY_VERSION.into(), review_id: "0".repeat(32), review_sha256: "0".repeat(64),
            previous_review_id: previous.as_ref().map(|value| value.0.clone()), previous_review_sha256: previous.map(|value| value.1),
            stage_102_attempt_id: "1".repeat(32), stage_101_claim_sha256: "2".repeat(64), stage_101_input_manifest_sha256: "3".repeat(64), stage_102_result_id: "4".repeat(32), stage_102_result_sha256: "5".repeat(64), stage_102_output_sha256: "6".repeat(64), stage_103_validation_id: "7".repeat(32), stage_103_validation_sha256: "8".repeat(64), cycle_claim_id: "9".repeat(32), cycle_claim_sha256: "a".repeat(64), subject_symbols: vec!["ZZTEST".into()], benchmark_symbol: "SPY".into(),
            window_start_date: NaiveDate::from_ymd_opt(2099, 1, 1).unwrap(), window_end_date: NaiveDate::from_ymd_opt(2099, 1, 3).unwrap(), subject_symbol_set_sha256: "b".repeat(64), time_window_sha256: "c".repeat(64), canonical_request_set_sha256: "d".repeat(64), raw_payload_custody_manifest_sha256: "e".repeat(64), source_receipt_count: 11,
            latest_source_retrieved_at_utc: time, parser_completed_at_utc: time, independently_validated_at_utc: time, submitted_at: time, admitted_available_at_utc: time, availability_basis: AVAILABILITY_BASIS.into(), provider_publication_time_limitation: PROVIDER_TIME_LIMITATION.into(), provider_publication_time_verified: false, custody_retrieval_time_verified: false,
            official_market_session_count: 1, price_row_count: 6, dividend_row_count: 0, split_row_count: 0, explicit_gap_count: 0, earliest_market_session_date: NaiveDate::from_ymd_opt(2099, 1, 2).unwrap(), latest_market_session_date: NaiveDate::from_ymd_opt(2099, 1, 2).unwrap(), submitted_by: actor.into(), excluded_prior_actor_ids: vec!["prior".into()],
            verdict: ControlledShadowObservationInputAdmissionVerdict::ChangesRequested, rationale: "test".into(), known_limitations: "provider time unverified".into(), reviewer_independent_from_validator_executor_and_complete_prior_chain: false,
            exact_current_stage_51_through_stage_103_binding_confirmed: false, stage_103_full_reparse_validation_current_and_passed_confirmed: false, cycle_claim_natural_forward_only_and_no_backfill_confirmed: false, fixed_subject_spy_window_and_request_identities_confirmed: false, every_raw_payload_custody_retrieval_timestamp_reviewed_confirmed: false, custody_retrieval_time_used_as_conservative_availability_not_provider_publication_confirmed: false, admitted_rows_within_frozen_window_and_available_before_admission_confirmed: false, official_sessions_and_spy_three_price_bases_complete_confirmed: false, subject_gaps_explicit_and_no_fill_or_cross_series_substitution_confirmed: false, dividends_splits_and_three_price_bases_remain_separate_confirmed: false, exact_output_no_rewrite_correction_or_retroactive_backfill_confirmed: false, approval_only_opens_future_materialization_specification_registration_confirmed: false, no_observation_ledger_position_performance_model_metric_training_reward_order_broker_or_trading_confirmed: false, no_unconfirmed_hari_or_old_wang_logic_claimed: false,
            structural_input_audit_passed: false, observation_input_admitted: false, future_create_once_observation_materialization_specification_registration_eligible: false, observation_materialization_specification_registered: false, forward_observation_started: false, ledger_created: false, position_written: false, performance_metric_written: false, model_or_metric_store_written: false, training_feedback_authorized: false, reward_authorized: false, order_generation_authorized: false, broker_access_authorized: false, trading_authorized: false,
        }
    }
}
