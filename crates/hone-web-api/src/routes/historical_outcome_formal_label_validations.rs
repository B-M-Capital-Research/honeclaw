//! Independent validation and offline-dataset-candidate admission for formal labels.
//!
//! This stage reopens one exact create-once formal raw-outcome label, its write
//! claim, the stage-twenty authorization and the complete current upstream
//! evidence chain. A validator outside the complete producer/reviewer actor set
//! independently verifies canonical hashes, the fixed eight-field payload,
//! provenance, limitations and every frozen metric bit. Passing creates only an
//! immutable offline-training-dataset *candidate admission* record. It does not
//! copy data into a training store or authorize training, reward, shadow, order,
//! broker or trading activity.

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

use super::historical_outcome_dry_run_execution_attempts::HistoricalOutcomeDryRunMetric;
use super::historical_outcome_formal_label_writes::{
    FormalLabelForIndependentTrainingAdmissionValidation, HistoricalOutcomeFormalLabel,
    HistoricalOutcomeFormalLabelWriteClaim,
    formal_labels_for_independent_training_admission_validation,
};
use super::historical_outcome_label_write_authorizations::{
    ApprovedHistoricalOutcomeLabelWriteAuthorization, FORMAL_LABEL_SCHEMA_VERSION,
    FORMAL_LABEL_SEMANTICS_VERSION, allowed_label_fields,
    exact_label_write_authorization_for_audit, label_contract_sha256,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str =
    "hone-historical-outcome-formal-label-training-admission-validation-registry-v1";
const RECORD_SCHEMA_VERSION: &str =
    "hone-historical-outcome-formal-label-training-admission-validation-v1";
const POLICY_VERSION: &str =
    "hone-independent-formal-raw-label-validation-and-offline-dataset-candidate-admission-v1";
const VALIDATOR_IMPLEMENTATION_VERSION: &str =
    "hone-independent-formal-raw-label-validator-v1-no-writer-code-reuse";
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];

#[derive(Debug, Deserialize)]
pub(crate) struct ValidateHistoricalOutcomeFormalLabelRequest {
    expected_label_sha256: String,
    expected_claim_sha256: String,
    expected_authorization_review_sha256: String,
    expected_materialization_validation_sha256: String,
    expected_materialization_output_sha256: String,
    expected_source_validation_sha256: String,
    expected_source_output_sha256: String,
    expected_snapshot_sha256: String,
    expected_protocol_sha256: String,
    expected_recomputed_metrics_sha256: String,
    expected_label_contract_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum HistoricalOutcomeFormalLabelValidationVerdict {
    AdmittedToOfflineTrainingDatasetCandidate,
    FailedIndependentValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord {
    pub schema_version: String,
    pub policy_version: String,
    pub validation_id: String,
    pub validation_sha256: String,
    pub label_id: String,
    pub label_sha256: String,
    pub label_schema_version: String,
    pub label_semantics_version: String,
    pub label_contract_sha256: String,
    pub write_claim_id: String,
    pub write_claim_sha256: String,
    pub authorization_review_id: String,
    pub authorization_review_sha256: String,
    pub materialization_validation_sha256: String,
    pub materialization_output_sha256: String,
    pub admission_review_sha256: String,
    pub source_validation_sha256: String,
    pub source_output_sha256: String,
    pub snapshot_sha256: String,
    pub reconstruction_sha256: String,
    pub protocol_sha256: String,
    pub recomputed_metrics_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub common_session_count: usize,
    pub metric_horizons_market_sessions: Vec<u16>,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validated_at: DateTime<Utc>,
    pub validated_by: String,
    pub formal_label_written_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub validator_independent_from_writer_and_complete_prior_chain: bool,
    pub current_upstream_binding_verified: bool,
    pub canonical_label_hash_verified: bool,
    pub canonical_claim_hash_verified: bool,
    pub fixed_eight_field_payload_verified: bool,
    pub exact_metric_bits_verified: bool,
    pub provenance_verified: bool,
    pub known_limitations_verified: bool,
    pub no_semantic_or_downstream_authority_verified: bool,
    pub independently_validated_metrics: Vec<HistoricalOutcomeDryRunMetric>,
    pub mismatch_reasons: Vec<String>,
    pub verdict: HistoricalOutcomeFormalLabelValidationVerdict,
    pub independently_validated_for_training_admission: bool,
    pub admitted_to_offline_training_dataset_candidate: bool,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub training_target_written: bool,
    pub reward_authorized: bool,
    pub reward_written: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFormalLabelValidationItem {
    pub formal_label: FormalLabelForIndependentTrainingAdmissionValidation,
    pub validation: Option<HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord>,
    pub validation_eligible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeFormalLabelValidationRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub validator_implementation_version: String,
    pub validator_implementation_sha256: String,
    pub validation_available: bool,
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub admitted_candidate_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
    pub items: Vec<HistoricalOutcomeFormalLabelValidationItem>,
    pub copied_to_training_store: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub reward_authorized: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub(crate) struct HistoricalOutcomeFormalLabelValidationReadinessSummary {
    pub validation_eligible_count: usize,
    pub validation_count: usize,
    pub admitted_candidate_count: usize,
    pub failed_validation_count: usize,
    pub validation_status: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AdmittedFormalLabelOfflineTrainingDatasetCandidate {
    pub formal_label: FormalLabelForIndependentTrainingAdmissionValidation,
    pub validation: HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord,
}

pub(crate) async fn handle_get_historical_outcome_formal_label_validations(
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
            warn!(%error, "historical formal label independent validation registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史正式原始标签独立校验暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_validate_historical_outcome_formal_label(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    AxumPath(label_id): AxumPath<String>,
    Json(request): Json<ValidateHistoricalOutcomeFormalLabelRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match validate_once(&state, &admin.user_id, &label_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => {
                warn!(%error, "historical formal label independent validation reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史正式原始标签独立校验暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_formal_label_validation_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeFormalLabelValidationReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeFormalLabelValidationReadinessSummary {
        validation_eligible_count: registry.validation_eligible_count,
        validation_count: registry.validation_count,
        admitted_candidate_count: registry.admitted_candidate_count,
        failed_validation_count: registry.failed_validation_count,
        validation_status: registry.validation_status,
    })
}

#[allow(dead_code)]
pub(crate) async fn admitted_formal_label_offline_training_dataset_candidates(
    state: &AppState,
) -> Result<Vec<AdmittedFormalLabelOfflineTrainingDatasetCandidate>, String> {
    let labels = formal_labels_for_independent_training_admission_validation(state).await?;
    let records = read_validations(state).await?;
    let index = records
        .into_iter()
        .filter(|record| record.admitted_to_offline_training_dataset_candidate)
        .map(|record| (record.label_id.clone(), record))
        .collect::<HashMap<_, _>>();
    let mut output = Vec::new();
    for formal_label in labels {
        let Some(validation) = index.get(&formal_label.label.label_id).cloned() else {
            continue;
        };
        if validation.label_sha256 == formal_label.label.label_sha256
            && validation.write_claim_sha256 == formal_label.claim.claim_sha256
        {
            output.push(AdmittedFormalLabelOfflineTrainingDatasetCandidate {
                formal_label,
                validation,
            });
        }
    }
    Ok(output)
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeFormalLabelValidationRegistry, String> {
    let labels = formal_labels_for_independent_training_admission_validation(state).await?;
    let validations = read_validations(state).await?;
    let index = validations
        .iter()
        .map(|record| (record.label_id.as_str(), record))
        .collect::<HashMap<_, _>>();
    let items = labels
        .into_iter()
        .map(|formal_label| {
            let validation = index
                .get(formal_label.label.label_id.as_str())
                .cloned()
                .cloned();
            HistoricalOutcomeFormalLabelValidationItem {
                validation_eligible: validation.is_none(),
                formal_label,
                validation,
            }
        })
        .collect::<Vec<_>>();
    let validation_eligible_count = items.iter().filter(|item| item.validation_eligible).count();
    let validation_count = validations.len();
    let admitted_candidate_count = validations
        .iter()
        .filter(|record| record.admitted_to_offline_training_dataset_candidate)
        .count();
    let failed_validation_count = validation_count.saturating_sub(admitted_candidate_count);
    let validation_status = if admitted_candidate_count > 0 {
        "formal_raw_labels_admitted_to_offline_dataset_candidate_training_still_closed"
    } else if failed_validation_count > 0 {
        "formal_raw_label_validation_failed_fail_closed"
    } else if validation_eligible_count > 0 {
        "waiting_independent_formal_label_validation_and_candidate_admission"
    } else {
        "waiting_create_once_formal_raw_label"
    };
    Ok(HistoricalOutcomeFormalLabelValidationRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validation_available: true,
        validation_eligible_count,
        validation_count,
        admitted_candidate_count,
        failed_validation_count,
        validation_status: validation_status.to_string(),
        items,
        copied_to_training_store: false,
        training_authorized: false,
        training_run_allowed: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "独立管理员重新打开正式标签、写入 claim、第二十阶段授权和完整当前来源链，独立核对固定八字段、canonical hash、来源、局限与 20/60/250 日指标位模式。通过只形成隔离的离线训练数据集候选准入记录；不复制到训练存储、不启动训练、不产生奖励、影子、订单、券商或交易权限。".to_string(),
    })
}

async fn validate_once(
    state: &AppState,
    validator_id: &str,
    label_id: &str,
    request: ValidateHistoricalOutcomeFormalLabelRequest,
) -> Result<HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord, String> {
    if !valid_id(label_id) {
        return Err("正式原始标签 ID 无效".to_string());
    }
    let _lock = acquire_lock(state, label_id).await?;
    if read_validations(state)
        .await?
        .iter()
        .any(|record| record.label_id == label_id)
    {
        return Err("该正式原始标签已经形成不可变独立校验记录，不得重放".to_string());
    }
    let formal_label = formal_labels_for_independent_training_admission_validation(state)
        .await?
        .into_iter()
        .find(|candidate| candidate.label.label_id == label_id)
        .ok_or_else(|| "当前找不到这条绑定有效的正式原始标签".to_string())?;
    verify_request(&formal_label, &request)?;
    let authorization = exact_label_write_authorization_for_audit(
        state,
        &formal_label.claim.authorization_review_id,
        &formal_label.claim.authorization_review_sha256,
    )
    .await?
    .ok_or_else(|| "第二十阶段授权或当前上游链已经失效".to_string())?;
    verify_authorization_binding(&formal_label, &authorization)?;
    let excluded_prior_actor_ids = excluded_prior_actor_ids(&formal_label, &authorization);
    let independent = validator_is_independent(validator_id, &excluded_prior_actor_ids);
    if !independent {
        return Err("独立校验人不能是标签写入人、授权复核人或完整上游链角色".to_string());
    }
    let mut mismatch_reasons = independent_mismatch_reasons(&formal_label, &authorization)?;
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    let canonical_label_hash_verified =
        independent_label_fingerprint(&formal_label.label)? == formal_label.label.label_sha256;
    let canonical_claim_hash_verified =
        independent_claim_fingerprint(&formal_label.claim)? == formal_label.claim.claim_sha256;
    let payload_keys = payload_keys(&formal_label.label)?;
    let fixed_eight_field_payload_verified =
        payload_keys == allowed_label_fields().into_iter().collect::<BTreeSet<_>>();
    let expected_metrics = &authorization
        .source
        .validation
        .independently_validated_metrics;
    let exact_metric_bits_verified = metrics_bitwise_equal(
        &formal_label.label.payload.raw_validated_metrics,
        expected_metrics,
    );
    let provenance_verified = provenance_matches(&formal_label, &authorization);
    let envelope = authorization
        .source
        .attempt
        .result
        .untrusted_envelope
        .as_ref()
        .ok_or_else(|| "授权来源缺少物化结果包".to_string())?;
    let known_limitations_verified =
        formal_label.label.payload.known_limitations == envelope.known_limitations;
    let no_semantic_or_downstream_authority_verified =
        semantic_and_downstream_authority_closed(&formal_label.label);
    if !canonical_label_hash_verified {
        mismatch_reasons.push("canonical_label_sha256_mismatch".to_string());
    }
    if !canonical_claim_hash_verified {
        mismatch_reasons.push("canonical_claim_sha256_mismatch".to_string());
    }
    if !fixed_eight_field_payload_verified {
        mismatch_reasons.push("formal_label_payload_fields_mismatch".to_string());
    }
    if !exact_metric_bits_verified {
        mismatch_reasons.push("exact_metric_bits_mismatch".to_string());
    }
    if !provenance_verified {
        mismatch_reasons.push("provenance_mismatch".to_string());
    }
    if !known_limitations_verified {
        mismatch_reasons.push("known_limitations_mismatch".to_string());
    }
    if !no_semantic_or_downstream_authority_verified {
        mismatch_reasons.push("semantic_or_downstream_authority_open".to_string());
    }
    mismatch_reasons.sort();
    mismatch_reasons.dedup();
    let passed = independent && mismatch_reasons.is_empty();
    let mut record = HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord {
        schema_version: RECORD_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        validation_id: String::new(),
        validation_sha256: String::new(),
        label_id: formal_label.label.label_id.clone(),
        label_sha256: formal_label.label.label_sha256.clone(),
        label_schema_version: formal_label.label.schema_version.clone(),
        label_semantics_version: formal_label.label.semantics_version.clone(),
        label_contract_sha256: formal_label.claim.label_contract_sha256.clone(),
        write_claim_id: formal_label.claim.claim_id.clone(),
        write_claim_sha256: formal_label.claim.claim_sha256.clone(),
        authorization_review_id: formal_label.claim.authorization_review_id.clone(),
        authorization_review_sha256: formal_label.claim.authorization_review_sha256.clone(),
        materialization_validation_sha256: formal_label
            .claim
            .materialization_validation_sha256
            .clone(),
        materialization_output_sha256: formal_label.claim.materialization_output_sha256.clone(),
        admission_review_sha256: formal_label.claim.admission_review_sha256.clone(),
        source_validation_sha256: formal_label.claim.source_validation_sha256.clone(),
        source_output_sha256: formal_label.claim.source_output_sha256.clone(),
        snapshot_sha256: formal_label.claim.snapshot_sha256.clone(),
        reconstruction_sha256: formal_label.claim.reconstruction_sha256.clone(),
        protocol_sha256: formal_label.claim.protocol_sha256.clone(),
        recomputed_metrics_sha256: formal_label.claim.recomputed_metrics_sha256.clone(),
        asset_symbol: formal_label.label.payload.asset_symbol.clone(),
        benchmark_symbol: formal_label.label.payload.benchmark_symbol.clone(),
        decision_available_at: formal_label.label.payload.decision_available_at,
        common_session_count: formal_label.label.payload.common_session_count,
        metric_horizons_market_sessions: formal_label
            .label
            .payload
            .raw_validated_metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect(),
        validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
        validator_implementation_sha256: validator_implementation_sha256(),
        validated_at: Utc::now(),
        validated_by: validator_id.to_string(),
        formal_label_written_by: formal_label.label.written_by.clone(),
        excluded_prior_actor_ids,
        validator_independent_from_writer_and_complete_prior_chain: independent,
        current_upstream_binding_verified: true,
        canonical_label_hash_verified,
        canonical_claim_hash_verified,
        fixed_eight_field_payload_verified,
        exact_metric_bits_verified,
        provenance_verified,
        known_limitations_verified,
        no_semantic_or_downstream_authority_verified,
        independently_validated_metrics: expected_metrics.clone(),
        mismatch_reasons,
        verdict: if passed {
            HistoricalOutcomeFormalLabelValidationVerdict::AdmittedToOfflineTrainingDatasetCandidate
        } else {
            HistoricalOutcomeFormalLabelValidationVerdict::FailedIndependentValidation
        },
        independently_validated_for_training_admission: passed,
        admitted_to_offline_training_dataset_candidate: passed,
        copied_to_training_store: false,
        training_authorized: false,
        training_run_allowed: false,
        training_target_written: false,
        reward_authorized: false,
        reward_written: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    record.validation_sha256 = validation_fingerprint(&record)?;
    record.validation_id = record.validation_sha256[..32].to_string();
    validate_record(&record)?;
    write_immutable_json(
        &validation_root(state)
            .join(&record.label_id)
            .join(format!("{}.json", record.validation_id)),
        &record,
    )
    .await?;
    Ok(record)
}

fn verify_request(
    formal_label: &FormalLabelForIndependentTrainingAdmissionValidation,
    request: &ValidateHistoricalOutcomeFormalLabelRequest,
) -> Result<(), String> {
    let claim = &formal_label.claim;
    if request.expected_label_sha256 != formal_label.label.label_sha256
        || request.expected_claim_sha256 != claim.claim_sha256
        || request.expected_authorization_review_sha256 != claim.authorization_review_sha256
        || request.expected_materialization_validation_sha256
            != claim.materialization_validation_sha256
        || request.expected_materialization_output_sha256 != claim.materialization_output_sha256
        || request.expected_source_validation_sha256 != claim.source_validation_sha256
        || request.expected_source_output_sha256 != claim.source_output_sha256
        || request.expected_snapshot_sha256 != claim.snapshot_sha256
        || request.expected_protocol_sha256 != claim.protocol_sha256
        || request.expected_recomputed_metrics_sha256 != claim.recomputed_metrics_sha256
        || request.expected_label_contract_sha256 != claim.label_contract_sha256
    {
        return Err("正式标签、claim、授权或完整上游绑定已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn verify_authorization_binding(
    formal_label: &FormalLabelForIndependentTrainingAdmissionValidation,
    authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization,
) -> Result<(), String> {
    let claim = &formal_label.claim;
    let source = &authorization.source;
    if authorization.review.review_id != claim.authorization_review_id
        || authorization.review.review_sha256 != claim.authorization_review_sha256
        || source.validation.validation_sha256 != claim.materialization_validation_sha256
        || source.validation.output_sha256 != claim.materialization_output_sha256
        || source.attempt.claim.admission_review_sha256 != claim.admission_review_sha256
        || source.attempt.claim.validation_sha256 != claim.source_validation_sha256
        || source.attempt.claim.source_output_sha256 != claim.source_output_sha256
        || source.attempt.claim.snapshot_sha256 != claim.snapshot_sha256
        || source.attempt.claim.reconstruction_sha256 != claim.reconstruction_sha256
        || source.attempt.claim.protocol_sha256 != claim.protocol_sha256
        || source.attempt.claim.recomputed_metrics_sha256 != claim.recomputed_metrics_sha256
        || authorization.review.label_contract_sha256 != claim.label_contract_sha256
    {
        return Err("正式标签与第二十阶段授权或当前来源链绑定不一致".to_string());
    }
    Ok(())
}

fn excluded_prior_actor_ids(
    formal_label: &FormalLabelForIndependentTrainingAdmissionValidation,
    authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization,
) -> Vec<String> {
    let mut actors = authorization.review.excluded_prior_actor_ids.clone();
    actors.push(authorization.review.reviewer_id.clone());
    actors.push(formal_label.claim.invoked_by.clone());
    actors.push(formal_label.label.written_by.clone());
    actors.retain(|value| !value.trim().is_empty());
    actors.sort();
    actors.dedup();
    actors
}

fn validator_is_independent(validator_id: &str, excluded: &[String]) -> bool {
    !validator_id.trim().is_empty() && !excluded.iter().any(|actor| actor == validator_id)
}

fn independent_mismatch_reasons(
    formal_label: &FormalLabelForIndependentTrainingAdmissionValidation,
    authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization,
) -> Result<Vec<String>, String> {
    let mut reasons = Vec::new();
    let label = &formal_label.label;
    let claim = &formal_label.claim;
    let envelope = authorization
        .source
        .attempt
        .result
        .untrusted_envelope
        .as_ref()
        .ok_or_else(|| "授权来源缺少物化结果包".to_string())?;
    if label.schema_version != FORMAL_LABEL_SCHEMA_VERSION
        || label.semantics_version != FORMAL_LABEL_SEMANTICS_VERSION
        || claim.formal_label_schema_version != FORMAL_LABEL_SCHEMA_VERSION
        || claim.formal_label_semantics_version != FORMAL_LABEL_SEMANTICS_VERSION
        || claim.label_contract_sha256 != label_contract_sha256()
        || label.payload.immutable_chain_bindings.label_contract_sha256 != label_contract_sha256()
    {
        reasons.push("formal_label_contract_mismatch".to_string());
    }
    let expected_label_id = independent_target_label_id(
        &claim.materialization_validation_sha256,
        &claim.materialization_output_sha256,
        &claim.label_contract_sha256,
    )?;
    if label.label_id != expected_label_id || claim.target_label_id != expected_label_id {
        reasons.push("formal_label_identity_mismatch".to_string());
    }
    if label.claim_id != claim.claim_id
        || label.claim_sha256 != claim.claim_sha256
        || label.created_at < claim.claimed_at
        || label.written_by != claim.invoked_by
    {
        reasons.push("write_claim_binding_mismatch".to_string());
    }
    let horizons = label
        .payload
        .raw_validated_metrics
        .iter()
        .map(|metric| metric.horizon_market_sessions)
        .collect::<Vec<_>>();
    if horizons != REQUIRED_HORIZONS
        || label
            .payload
            .raw_validated_metrics
            .iter()
            .any(metric_invalid)
        || label.payload.source_provenance.metric_end_dates.len() != REQUIRED_HORIZONS.len()
    {
        reasons.push("raw_metric_structure_invalid".to_string());
    }
    if hash_serializable(
        &authorization
            .source
            .validation
            .independently_validated_metrics,
    )? != claim.recomputed_metrics_sha256
    {
        reasons.push("current_source_metric_hash_mismatch".to_string());
    }
    if label.payload.asset_symbol != envelope.asset_symbol
        || label.payload.benchmark_symbol != envelope.benchmark_symbol
        || label.payload.decision_available_at != envelope.decision_available_at
        || label.payload.common_session_count != envelope.common_session_count
    {
        reasons.push("formal_label_market_identity_mismatch".to_string());
    }
    Ok(reasons)
}

fn provenance_matches(
    formal_label: &FormalLabelForIndependentTrainingAdmissionValidation,
    authorization: &ApprovedHistoricalOutcomeLabelWriteAuthorization,
) -> bool {
    let label = &formal_label.label;
    let claim = &formal_label.claim;
    let source = &authorization.source;
    let provenance = &label.payload.source_provenance;
    let bindings = &label.payload.immutable_chain_bindings;
    provenance.materialization_validation_id == source.validation.validation_id
        && provenance.materialization_validated_at == source.validation.validated_at
        && provenance.materialization_validated_by == source.validation.validated_by
        && provenance.materialization_attempt_id == source.attempt.claim.attempt_id
        && provenance.source_attempt_id == source.attempt.claim.source_attempt_id
        && provenance.source_validation_id == source.attempt.claim.validation_id
        && provenance.snapshot_id == source.attempt.claim.snapshot_id
        && provenance.reconstruction_id == source.attempt.claim.reconstruction_id
        && provenance.protocol_version == source.attempt.claim.protocol_version
        && bindings.authorization_review_id == claim.authorization_review_id
        && bindings.authorization_review_sha256 == claim.authorization_review_sha256
        && bindings.materialization_validation_sha256 == claim.materialization_validation_sha256
        && bindings.materialization_claim_sha256 == claim.materialization_claim_sha256
        && bindings.materialization_result_sha256 == claim.materialization_result_sha256
        && bindings.materialization_output_sha256 == claim.materialization_output_sha256
        && bindings.admission_review_sha256 == claim.admission_review_sha256
        && bindings.source_validation_sha256 == claim.source_validation_sha256
        && bindings.source_output_sha256 == claim.source_output_sha256
        && bindings.snapshot_sha256 == claim.snapshot_sha256
        && bindings.reconstruction_sha256 == claim.reconstruction_sha256
        && bindings.protocol_sha256 == claim.protocol_sha256
        && bindings.recomputed_metrics_sha256 == claim.recomputed_metrics_sha256
        && bindings.label_contract_sha256 == claim.label_contract_sha256
}

fn semantic_and_downstream_authority_closed(label: &HistoricalOutcomeFormalLabel) -> bool {
    label.exact_metric_bits_preserved
        && label.provenance_preserved
        && label.known_limitations_preserved
        && label.formal_label_written
        && !label.independently_validated_for_training_admission
        && !label.admitted_to_offline_training_dataset_candidate
        && !label.direction_inferred
        && !label.rating_inferred
        && !label.investment_action_inferred
        && !label.position_size_inferred
        && !label.training_target_written
        && !label.reward_written
        && !label.shadow_position_written
        && !label.order_generated
        && !label.broker_accessed
        && !label.trade_executed
}

fn metrics_bitwise_equal(
    observed: &[HistoricalOutcomeDryRunMetric],
    expected: &[HistoricalOutcomeDryRunMetric],
) -> bool {
    observed.len() == expected.len()
        && observed.iter().zip(expected).all(|(left, right)| {
            left.horizon_market_sessions == right.horizon_market_sessions
                && left.start_date == right.start_date
                && left.end_date == right.end_date
                && left.asset_return.to_bits() == right.asset_return.to_bits()
                && left.benchmark_return.to_bits() == right.benchmark_return.to_bits()
                && left.excess_return.to_bits() == right.excess_return.to_bits()
                && left.asset_max_drawdown.to_bits() == right.asset_max_drawdown.to_bits()
        })
}

fn metric_invalid(metric: &HistoricalOutcomeDryRunMetric) -> bool {
    metric.end_date < metric.start_date
        || !metric.asset_return.is_finite()
        || !metric.benchmark_return.is_finite()
        || !metric.excess_return.is_finite()
        || !metric.asset_max_drawdown.is_finite()
        || metric.asset_max_drawdown > 0.0
}

fn payload_keys(label: &HistoricalOutcomeFormalLabel) -> Result<BTreeSet<String>, String> {
    Ok(serde_json::to_value(&label.payload)
        .map_err(|error| error.to_string())?
        .as_object()
        .ok_or_else(|| "正式标签 payload 不是对象".to_string())?
        .keys()
        .cloned()
        .collect())
}

fn independent_target_label_id(
    materialization_validation_sha256: &str,
    materialization_output_sha256: &str,
    contract_sha256: &str,
) -> Result<String, String> {
    let value = serde_json::json!({
        "schema_version": FORMAL_LABEL_SCHEMA_VERSION,
        "semantics_version": FORMAL_LABEL_SEMANTICS_VERSION,
        "materialization_validation_sha256": materialization_validation_sha256,
        "materialization_output_sha256": materialization_output_sha256,
        "label_contract_sha256": contract_sha256,
    });
    Ok(hash_serializable(&value)?[..32].to_string())
}

fn independent_label_fingerprint(label: &HistoricalOutcomeFormalLabel) -> Result<String, String> {
    fingerprint_without(label, &["label_sha256"])
}

fn independent_claim_fingerprint(
    claim: &HistoricalOutcomeFormalLabelWriteClaim,
) -> Result<String, String> {
    fingerprint_without(claim, &["claim_id", "claim_sha256"])
}

fn validate_record(
    record: &HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord,
) -> Result<(), String> {
    let passed = record.validator_independent_from_writer_and_complete_prior_chain
        && record.current_upstream_binding_verified
        && record.canonical_label_hash_verified
        && record.canonical_claim_hash_verified
        && record.fixed_eight_field_payload_verified
        && record.exact_metric_bits_verified
        && record.provenance_verified
        && record.known_limitations_verified
        && record.no_semantic_or_downstream_authority_verified
        && record.mismatch_reasons.is_empty();
    let downstream_closed = !record.copied_to_training_store
        && !record.training_authorized
        && !record.training_run_allowed
        && !record.training_target_written
        && !record.reward_authorized
        && !record.reward_written
        && !record.shadow_portfolio_authorized
        && !record.order_generation_authorized
        && !record.broker_access_authorized
        && !record.trading_authorized;
    if record.schema_version != RECORD_SCHEMA_VERSION
        || record.policy_version != POLICY_VERSION
        || !valid_id(&record.validation_id)
        || !valid_sha256(&record.validation_sha256)
        || record.validation_id != record.validation_sha256[..32]
        || record.validation_sha256 != validation_fingerprint(record)?
        || !valid_id(&record.label_id)
        || !valid_sha256(&record.label_sha256)
        || record.label_schema_version != FORMAL_LABEL_SCHEMA_VERSION
        || record.label_semantics_version != FORMAL_LABEL_SEMANTICS_VERSION
        || record.label_contract_sha256 != label_contract_sha256()
        || !valid_id(&record.write_claim_id)
        || !valid_sha256(&record.write_claim_sha256)
        || !valid_id(&record.authorization_review_id)
        || !all_sha256(&[
            &record.authorization_review_sha256,
            &record.materialization_validation_sha256,
            &record.materialization_output_sha256,
            &record.admission_review_sha256,
            &record.source_validation_sha256,
            &record.source_output_sha256,
            &record.snapshot_sha256,
            &record.reconstruction_sha256,
            &record.protocol_sha256,
            &record.recomputed_metrics_sha256,
        ])
        || record.asset_symbol.trim().is_empty()
        || record.benchmark_symbol.trim().is_empty()
        || record.common_session_count < 251
        || record.metric_horizons_market_sessions != REQUIRED_HORIZONS
        || record.validator_implementation_version != VALIDATOR_IMPLEMENTATION_VERSION
        || record.validator_implementation_sha256 != validator_implementation_sha256()
        || record.validated_by.trim().is_empty()
        || record.formal_label_written_by.trim().is_empty()
        || record.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&record.excluded_prior_actor_ids)
        || record
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &record.validated_by)
        || record.independently_validated_metrics.len() != REQUIRED_HORIZONS.len()
        || record
            .independently_validated_metrics
            .iter()
            .map(|metric| metric.horizon_market_sessions)
            .collect::<Vec<_>>()
            != REQUIRED_HORIZONS
        || hash_serializable(&record.independently_validated_metrics)?
            != record.recomputed_metrics_sha256
        || record
            .independently_validated_metrics
            .iter()
            .any(metric_invalid)
        || record.independently_validated_for_training_admission != passed
        || record.admitted_to_offline_training_dataset_candidate != passed
        || (passed
            != (record.verdict
                == HistoricalOutcomeFormalLabelValidationVerdict::AdmittedToOfflineTrainingDatasetCandidate))
        || !downstream_closed
    {
        return Err("historical formal label validation record is invalid or over-authorized".to_string());
    }
    Ok(())
}

fn validation_fingerprint(
    record: &HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord,
) -> Result<String, String> {
    fingerprint_without(record, &["validation_id", "validation_sha256"])
}

fn fingerprint_without<T: Serialize>(value: &T, fields: &[&str]) -> Result<String, String> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "fingerprint payload is not an object".to_string())?;
    for field in fields {
        object.remove(*field);
    }
    hash_serializable(&value)
}

fn validator_implementation_sha256() -> String {
    hash_serializable(&serde_json::json!({
        "implementation": VALIDATOR_IMPLEMENTATION_VERSION,
        "policy": POLICY_VERSION,
        "writer_code_reused": false,
        "fixed_eight_field_payload": true,
        "metric_comparison": "ieee754_bits",
        "candidate_admission_only": true,
        "training_authorized": false,
        "reward_authorized": false,
        "shadow_order_broker_trading_authorized": false,
    }))
    .expect("static validator implementation is serializable")
}

async fn read_validations(
    state: &AppState,
) -> Result<Vec<HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord>, String> {
    let root = validation_root(state);
    let mut directories = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    let mut label_ids = BTreeSet::new();
    while let Some(directory) = directories
        .next_entry()
        .await
        .map_err(|error| error.to_string())?
    {
        if !directory
            .file_type()
            .await
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let mut entries = tokio::fs::read_dir(directory.path())
            .await
            .map_err(|error| error.to_string())?;
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
            let record: HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord =
                serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
            validate_record(&record)?;
            if entry.path().file_stem().and_then(|value| value.to_str())
                != Some(record.validation_id.as_str())
                || directory.file_name().to_string_lossy() != record.label_id
                || !ids.insert(record.validation_id.clone())
                || !hashes.insert(record.validation_sha256.clone())
                || !label_ids.insert(record.label_id.clone())
            {
                return Err(
                    "historical formal label validations contain replay or path mismatch"
                        .to_string(),
                );
            }
            output.push(record);
        }
    }
    output.sort_by(|left, right| right.validated_at.cmp(&left.validated_at));
    Ok(output)
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

fn validation_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical_outcome_formal_label_training_admission_validations")
}

struct ValidationLock {
    path: PathBuf,
}

impl Drop for ValidationLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState, label_id: &str) -> Result<ValidationLock, String> {
    if !valid_id(label_id) {
        return Err("invalid formal label validation lock name".to_string());
    }
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join(format!(
        "historical-formal-label-validation-{label_id}.lock"
    ));
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(ValidationLock { path }),
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
                return Err("同一正式标签正在由另一管理员独立校验".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("正式标签独立校验锁获取失败".to_string())
}

fn hash_serializable<T: Serialize>(value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn all_sha256(values: &[&String]) -> bool {
    values.iter().all(|value| valid_sha256(value))
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_id(value: &str) -> bool {
    value.len() == 32 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::*;

    fn metrics() -> Vec<HistoricalOutcomeDryRunMetric> {
        [20_u16, 60, 250]
            .into_iter()
            .map(|horizon| HistoricalOutcomeDryRunMetric {
                horizon_market_sessions: horizon,
                start_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()
                    + chrono::Days::new(u64::from(horizon)),
                asset_return: f64::from(horizon) / 100.0,
                benchmark_return: f64::from(horizon) / 200.0,
                excess_return: f64::from(horizon) / 200.0,
                asset_max_drawdown: -0.1,
            })
            .collect()
    }

    #[test]
    fn independent_bitwise_comparison_rejects_one_ulp() {
        let expected = metrics();
        let mut observed = expected.clone();
        observed[1].excess_return = f64::from_bits(observed[1].excess_return.to_bits() + 1);
        assert!(!metrics_bitwise_equal(&observed, &expected));
        assert!(metrics_bitwise_equal(&expected, &expected));
    }

    #[test]
    fn fixed_label_target_identity_is_contract_bound() {
        let first =
            independent_target_label_id(&"a".repeat(64), &"b".repeat(64), &"c".repeat(64)).unwrap();
        let changed =
            independent_target_label_id(&"a".repeat(64), &"b".repeat(64), &"d".repeat(64)).unwrap();
        assert!(valid_id(&first));
        assert_ne!(first, changed);
    }

    #[test]
    fn validator_must_be_outside_complete_actor_set() {
        let excluded = vec!["admission-reviewer".to_string(), "writer".to_string()];
        assert!(!validator_is_independent("writer", &excluded));
        assert!(validator_is_independent("new-validator", &excluded));
        assert!(!validator_is_independent("", &excluded));
    }

    #[test]
    fn validator_implementation_is_stable_and_non_training() {
        assert!(valid_sha256(&validator_implementation_sha256()));
        assert!(POLICY_VERSION.contains("candidate-admission"));
        assert!(VALIDATOR_IMPLEMENTATION_VERSION.contains("no-writer-code-reuse"));
    }

    #[test]
    fn admitted_record_still_grants_no_training_or_execution_authority() {
        let now = Utc::now();
        let independently_validated_metrics = metrics();
        let recomputed_metrics_sha256 =
            hash_serializable(&independently_validated_metrics).unwrap();
        let mut record = HistoricalOutcomeFormalLabelTrainingAdmissionValidationRecord {
            schema_version: RECORD_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            validation_id: String::new(),
            validation_sha256: String::new(),
            label_id: "a".repeat(32),
            label_sha256: "a".repeat(64),
            label_schema_version: FORMAL_LABEL_SCHEMA_VERSION.to_string(),
            label_semantics_version: FORMAL_LABEL_SEMANTICS_VERSION.to_string(),
            label_contract_sha256: label_contract_sha256(),
            write_claim_id: "b".repeat(32),
            write_claim_sha256: "b".repeat(64),
            authorization_review_id: "c".repeat(32),
            authorization_review_sha256: "c".repeat(64),
            materialization_validation_sha256: "d".repeat(64),
            materialization_output_sha256: "e".repeat(64),
            admission_review_sha256: "f".repeat(64),
            source_validation_sha256: "1".repeat(64),
            source_output_sha256: "2".repeat(64),
            snapshot_sha256: "3".repeat(64),
            reconstruction_sha256: "4".repeat(64),
            protocol_sha256: "5".repeat(64),
            recomputed_metrics_sha256,
            asset_symbol: "SNDK".to_string(),
            benchmark_symbol: "SPY".to_string(),
            decision_available_at: now,
            common_session_count: 251,
            metric_horizons_market_sessions: REQUIRED_HORIZONS.to_vec(),
            validator_implementation_version: VALIDATOR_IMPLEMENTATION_VERSION.to_string(),
            validator_implementation_sha256: validator_implementation_sha256(),
            validated_at: now,
            validated_by: "independent-validator".to_string(),
            formal_label_written_by: "writer".to_string(),
            excluded_prior_actor_ids: vec!["reviewer".to_string(), "writer".to_string()],
            validator_independent_from_writer_and_complete_prior_chain: true,
            current_upstream_binding_verified: true,
            canonical_label_hash_verified: true,
            canonical_claim_hash_verified: true,
            fixed_eight_field_payload_verified: true,
            exact_metric_bits_verified: true,
            provenance_verified: true,
            known_limitations_verified: true,
            no_semantic_or_downstream_authority_verified: true,
            independently_validated_metrics,
            mismatch_reasons: Vec::new(),
            verdict: HistoricalOutcomeFormalLabelValidationVerdict::AdmittedToOfflineTrainingDatasetCandidate,
            independently_validated_for_training_admission: true,
            admitted_to_offline_training_dataset_candidate: true,
            copied_to_training_store: false,
            training_authorized: false,
            training_run_allowed: false,
            training_target_written: false,
            reward_authorized: false,
            reward_written: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        record.validation_sha256 = validation_fingerprint(&record).unwrap();
        record.validation_id = record.validation_sha256[..32].to_string();
        validate_record(&record).unwrap();
        assert!(record.admitted_to_offline_training_dataset_candidate);
        assert!(!record.training_authorized);
        assert!(!record.training_run_allowed);
        assert!(!record.reward_authorized);
        assert!(!record.shadow_portfolio_authorized);
        assert!(!record.trading_authorized);
    }
}
