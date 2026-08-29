//! Versioned, content-addressed assembly of independently admitted raw outcomes.
//!
//! This stage copies the complete current set of stage-twenty-two passing raw
//! outcome candidates into an isolated, immutable dataset object. Versions are
//! monotonic: a later version must preserve every earlier entry byte-for-byte
//! and may only append newly admitted candidates. The dataset contains no
//! features, semantic targets, split assignments, rewards or executable model
//! artifact and grants no training or trading authority.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration as StdDuration, SystemTime};

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::warn;

use super::historical_outcome_dry_run_execution_attempts::HistoricalOutcomeDryRunMetric;
use super::historical_outcome_formal_label_validations::{
    AdmittedFormalLabelOfflineTrainingDatasetCandidate,
    admitted_formal_label_offline_training_dataset_candidates,
};
use super::historical_outcome_formal_label_writes::{
    HistoricalOutcomeFormalLabelImmutableChainBindings,
    HistoricalOutcomeFormalLabelSourceProvenance,
};
use crate::state::AppState;

const REGISTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-registry-v1";
const DATASET_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-v1";
const ENTRY_SCHEMA_VERSION: &str = "hone-historical-outcome-offline-dataset-entry-v1";
const POLICY_VERSION: &str =
    "hone-versioned-content-addressed-monotonic-raw-outcome-dataset-assembly-v1";
const ASSEMBLER_IMPLEMENTATION_VERSION: &str =
    "hone-deterministic-raw-outcome-dataset-assembler-v1-no-target-or-split-inference";
const PURPOSE: &str = "historical_raw_outcome_research_only";
const SPLIT_POLICY_STATUS: &str = "not_assigned_pending_independent_dataset_governance";
const REQUIRED_HORIZONS: [u16; 3] = [20, 60, 250];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct HistoricalOutcomeOfflineDatasetCandidateBinding {
    pub label_id: String,
    pub label_sha256: String,
    pub write_claim_id: String,
    pub write_claim_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AssembleHistoricalOutcomeOfflineDatasetRequest {
    expected_candidate_set_sha256: String,
    expected_candidates: Vec<HistoricalOutcomeOfflineDatasetCandidateBinding>,
    purpose: String,
    complete_current_candidate_set_confirmed: bool,
    monotonic_version_lineage_confirmed: bool,
    point_in_time_lineage_preserved_confirmed: bool,
    no_semantic_target_or_split_inference_confirmed: bool,
    no_training_reward_shadow_order_broker_or_trading_confirmed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetEntry {
    pub schema_version: String,
    pub ordinal: usize,
    pub entry_id: String,
    pub entry_sha256: String,
    pub label_id: String,
    pub label_sha256: String,
    pub write_claim_id: String,
    pub write_claim_sha256: String,
    pub validation_id: String,
    pub validation_sha256: String,
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
    pub label_contract_sha256: String,
    pub asset_symbol: String,
    pub benchmark_symbol: String,
    pub decision_available_at: DateTime<Utc>,
    pub common_session_count: usize,
    pub raw_validated_metrics: Vec<HistoricalOutcomeDryRunMetric>,
    pub source_provenance: HistoricalOutcomeFormalLabelSourceProvenance,
    pub known_limitations: String,
    pub immutable_chain_bindings: HistoricalOutcomeFormalLabelImmutableChainBindings,
    pub formal_label_written_by: String,
    pub independently_validated_at: DateTime<Utc>,
    pub independently_validated_by: String,
    pub excluded_prior_actor_ids: Vec<String>,
    pub raw_outcome_only: bool,
    pub feature_vector_present: bool,
    pub semantic_target_assigned: bool,
    pub split_assigned: bool,
    pub reward_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoricalOutcomeOfflineDataset {
    pub schema_version: String,
    pub policy_version: String,
    pub assembler_implementation_version: String,
    pub assembler_implementation_sha256: String,
    pub dataset_id: String,
    pub dataset_version: String,
    pub version_number: usize,
    pub dataset_content_sha256: String,
    pub manifest_sha256: String,
    pub parent_dataset_id: Option<String>,
    pub parent_manifest_sha256: Option<String>,
    pub candidate_set_sha256: String,
    pub assembled_at: DateTime<Utc>,
    pub assembled_by: String,
    pub purpose: String,
    pub entry_count: usize,
    pub added_entry_count: usize,
    pub distinct_symbol_count: usize,
    pub earliest_decision_available_at: DateTime<Utc>,
    pub latest_decision_available_at: DateTime<Utc>,
    pub entries: Vec<HistoricalOutcomeOfflineDatasetEntry>,
    pub complete_candidate_set_frozen: bool,
    pub monotonic_append_only_lineage: bool,
    pub point_in_time_lineage_preserved: bool,
    pub duplicate_labels_rejected: bool,
    pub conflicting_decision_identities_rejected: bool,
    pub split_policy_status: String,
    pub copied_to_isolated_offline_dataset_store: bool,
    pub copied_to_training_store: bool,
    pub feature_join_performed: bool,
    pub semantic_targets_assigned: bool,
    pub dataset_governance_approved: bool,
    pub training_authorized: bool,
    pub training_run_allowed: bool,
    pub reward_authorized: bool,
    pub reward_written: bool,
    pub shadow_portfolio_authorized: bool,
    pub order_generation_authorized: bool,
    pub broker_access_authorized: bool,
    pub trading_authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HistoricalOutcomeOfflineDatasetRegistry {
    pub schema_version: String,
    pub policy_version: String,
    pub assembler_implementation_version: String,
    pub assembler_implementation_sha256: String,
    pub assembly_available: bool,
    pub current_candidate_count: usize,
    pub current_candidate_set_sha256: String,
    pub current_candidates: Vec<HistoricalOutcomeOfflineDatasetCandidateBinding>,
    pub dataset_count: usize,
    pub current_binding_dataset_count: usize,
    pub latest_dataset: Option<HistoricalOutcomeOfflineDataset>,
    pub datasets: Vec<HistoricalOutcomeOfflineDataset>,
    pub assembly_status: String,
    pub copied_to_training_store: bool,
    pub feature_join_performed: bool,
    pub semantic_targets_assigned: bool,
    pub dataset_governance_approved: bool,
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
pub(crate) struct HistoricalOutcomeOfflineDatasetReadinessSummary {
    pub assembly_eligible_count: usize,
    pub dataset_count: usize,
    pub current_binding_dataset_count: usize,
    pub latest_entry_count: usize,
    pub assembly_status: String,
}

pub(crate) async fn handle_get_historical_outcome_offline_datasets(
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
            warn!(%error, "historical outcome offline dataset registry read failed");
            crate::routes::json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "历史结果离线数据集注册表暂时不可用",
            )
        }
    }
}

pub(crate) async fn handle_assemble_historical_outcome_offline_dataset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<AssembleHistoricalOutcomeOfflineDatasetRequest>,
) -> Response {
    let admin =
        match crate::routes::public_admin::require_public_admin_for_mutation(&state, &headers) {
            Ok(admin) => admin,
            Err(response) => return response,
        };
    match assemble_once(&state, &admin.user_id, request).await {
        Ok(_) => match build_registry(&state).await {
            Ok(value) => (StatusCode::CREATED, Json(value)).into_response(),
            Err(error) => {
                warn!(%error, "historical outcome offline dataset reload failed");
                crate::routes::json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "历史结果离线数据集注册表暂时不可用",
                )
            }
        },
        Err(error) => crate::routes::json_error(StatusCode::BAD_REQUEST, error),
    }
}

pub(crate) async fn historical_outcome_offline_dataset_readiness_summary(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetReadinessSummary, String> {
    let registry = build_registry(state).await?;
    Ok(HistoricalOutcomeOfflineDatasetReadinessSummary {
        assembly_eligible_count: usize::from(registry.assembly_available),
        dataset_count: registry.dataset_count,
        current_binding_dataset_count: registry.current_binding_dataset_count,
        latest_entry_count: registry
            .latest_dataset
            .as_ref()
            .map(|dataset| dataset.entry_count)
            .unwrap_or(0),
        assembly_status: registry.assembly_status,
    })
}

pub(crate) async fn historical_outcome_offline_dataset_registry_for_governance(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetRegistry, String> {
    build_registry(state).await
}

/// Reopens one exact, still-current immutable dataset for a separately
/// authorized transformation attempt.  Callers must bind every content hash;
/// an ID alone is never sufficient because a newly admitted label can make an
/// older dataset cease to represent the complete current candidate set.
pub(crate) async fn exact_current_historical_outcome_offline_dataset_for_transformation(
    state: &AppState,
    dataset_id: &str,
    dataset_content_sha256: &str,
    manifest_sha256: &str,
    candidate_set_sha256: &str,
) -> Result<Option<HistoricalOutcomeOfflineDataset>, String> {
    let registry = build_registry(state).await?;
    Ok(registry.datasets.into_iter().find(|dataset| {
        dataset.dataset_id == dataset_id
            && dataset.dataset_content_sha256 == dataset_content_sha256
            && dataset.manifest_sha256 == manifest_sha256
            && dataset.candidate_set_sha256 == candidate_set_sha256
            && dataset.candidate_set_sha256 == registry.current_candidate_set_sha256
    }))
}

async fn build_registry(
    state: &AppState,
) -> Result<HistoricalOutcomeOfflineDatasetRegistry, String> {
    let candidates = admitted_formal_label_offline_training_dataset_candidates(state).await?;
    let current_candidates = candidate_bindings(&candidates);
    let current_candidate_set_sha256 = candidate_set_sha256(&current_candidates)?;
    let datasets = read_datasets(state).await?;
    let latest_dataset = datasets.last().cloned();
    let current_binding_dataset_count = datasets
        .iter()
        .filter(|dataset| dataset.candidate_set_sha256 == current_candidate_set_sha256)
        .count();
    let monotonic = latest_dataset
        .as_ref()
        .is_none_or(|latest| candidate_bindings_extend_dataset(&current_candidates, latest));
    let assembly_available =
        !current_candidates.is_empty() && current_binding_dataset_count == 0 && monotonic;
    let assembly_status = if current_candidates.is_empty() {
        "waiting_independently_admitted_formal_label_candidates"
    } else if !monotonic {
        "current_candidate_set_breaks_monotonic_dataset_lineage_fail_closed"
    } else if current_binding_dataset_count > 0 {
        "current_candidate_set_assembled_dataset_governance_still_pending"
    } else {
        "current_candidate_set_ready_for_versioned_offline_dataset_assembly"
    };
    Ok(HistoricalOutcomeOfflineDatasetRegistry {
        schema_version: REGISTRY_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        assembler_implementation_version: ASSEMBLER_IMPLEMENTATION_VERSION.to_string(),
        assembler_implementation_sha256: assembler_implementation_sha256(),
        assembly_available,
        current_candidate_count: current_candidates.len(),
        current_candidate_set_sha256,
        current_candidates,
        dataset_count: datasets.len(),
        current_binding_dataset_count,
        latest_dataset,
        datasets,
        assembly_status: assembly_status.to_string(),
        copied_to_training_store: false,
        feature_join_performed: false,
        semantic_targets_assigned: false,
        dataset_governance_approved: false,
        training_authorized: false,
        training_run_allowed: false,
        reward_authorized: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
        scope: "把当前全部第 22 阶段通过候选确定性装配为内容寻址、版本化、单调追加且可重放的隔离原始结果数据集。版本不得删除或改写历史条目；当前不连接特征、不分配训练/验证/留出集、不生成语义目标、不运行训练，也不产生奖励、影子、订单、券商或交易权限。".to_string(),
    })
}

async fn assemble_once(
    state: &AppState,
    assembler_id: &str,
    request: AssembleHistoricalOutcomeOfflineDatasetRequest,
) -> Result<HistoricalOutcomeOfflineDataset, String> {
    if assembler_id.trim().is_empty() {
        return Err("离线数据集装配人无效".to_string());
    }
    let _lock = acquire_lock(state).await?;
    let candidates = admitted_formal_label_offline_training_dataset_candidates(state).await?;
    if candidates.is_empty() {
        return Err("当前没有经过独立校验并准入的正式原始标签候选".to_string());
    }
    let bindings = candidate_bindings(&candidates);
    verify_request(&request, &bindings)?;
    let existing = read_datasets(state).await?;
    if existing
        .iter()
        .any(|dataset| dataset.candidate_set_sha256 == request.expected_candidate_set_sha256)
    {
        return Err("当前候选集合已经装配为不可变数据集版本，不得重放".to_string());
    }
    let latest = existing.last();
    if latest.is_some_and(|dataset| !candidate_bindings_extend_dataset(&bindings, dataset)) {
        return Err("当前候选集合删除或改写了上一版本条目，单调数据集链失败关闭".to_string());
    }
    let entries = build_entries(&candidates, latest)?;
    validate_unique_entries(&entries)?;
    let previous_entry_count = latest.map(|dataset| dataset.entry_count).unwrap_or(0);
    let version_number = latest
        .map(|dataset| dataset.version_number + 1)
        .unwrap_or(1);
    let dataset_content_sha256 = dataset_content_sha256(
        version_number,
        latest.map(|dataset| dataset.dataset_id.as_str()),
        &request.expected_candidate_set_sha256,
        &entries,
    )?;
    let dataset_id = dataset_content_sha256[..32].to_string();
    let mut dataset = HistoricalOutcomeOfflineDataset {
        schema_version: DATASET_SCHEMA_VERSION.to_string(),
        policy_version: POLICY_VERSION.to_string(),
        assembler_implementation_version: ASSEMBLER_IMPLEMENTATION_VERSION.to_string(),
        assembler_implementation_sha256: assembler_implementation_sha256(),
        dataset_id: dataset_id.clone(),
        dataset_version: format!(
            "historical-raw-outcomes-v1-{:06}-{:.12}",
            version_number, dataset_content_sha256
        ),
        version_number,
        dataset_content_sha256,
        manifest_sha256: String::new(),
        parent_dataset_id: latest.map(|dataset| dataset.dataset_id.clone()),
        parent_manifest_sha256: latest.map(|dataset| dataset.manifest_sha256.clone()),
        candidate_set_sha256: request.expected_candidate_set_sha256,
        assembled_at: Utc::now(),
        assembled_by: assembler_id.to_string(),
        purpose: PURPOSE.to_string(),
        entry_count: entries.len(),
        added_entry_count: entries.len().saturating_sub(previous_entry_count),
        distinct_symbol_count: entries
            .iter()
            .map(|entry| entry.asset_symbol.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        earliest_decision_available_at: entries
            .iter()
            .map(|entry| entry.decision_available_at)
            .min()
            .expect("non-empty candidates"),
        latest_decision_available_at: entries
            .iter()
            .map(|entry| entry.decision_available_at)
            .max()
            .expect("non-empty candidates"),
        entries,
        complete_candidate_set_frozen: true,
        monotonic_append_only_lineage: true,
        point_in_time_lineage_preserved: true,
        duplicate_labels_rejected: true,
        conflicting_decision_identities_rejected: true,
        split_policy_status: SPLIT_POLICY_STATUS.to_string(),
        copied_to_isolated_offline_dataset_store: true,
        copied_to_training_store: false,
        feature_join_performed: false,
        semantic_targets_assigned: false,
        dataset_governance_approved: false,
        training_authorized: false,
        training_run_allowed: false,
        reward_authorized: false,
        reward_written: false,
        shadow_portfolio_authorized: false,
        order_generation_authorized: false,
        broker_access_authorized: false,
        trading_authorized: false,
    };
    dataset.manifest_sha256 = manifest_fingerprint(&dataset)?;
    validate_dataset(&dataset)?;
    write_immutable_json(
        &dataset_root(state).join(format!("{}.json", dataset.dataset_id)),
        &dataset,
    )
    .await?;
    Ok(dataset)
}

fn verify_request(
    request: &AssembleHistoricalOutcomeOfflineDatasetRequest,
    current: &[HistoricalOutcomeOfflineDatasetCandidateBinding],
) -> Result<(), String> {
    let mut expected = request.expected_candidates.clone();
    expected.sort();
    if request.purpose != PURPOSE
        || !request.complete_current_candidate_set_confirmed
        || !request.monotonic_version_lineage_confirmed
        || !request.point_in_time_lineage_preserved_confirmed
        || !request.no_semantic_target_or_split_inference_confirmed
        || !request.no_training_reward_shadow_order_broker_or_trading_confirmed
        || expected != current
        || request.expected_candidate_set_sha256 != candidate_set_sha256(current)?
    {
        return Err("候选集合、用途或零训练/零交易装配边界已经变化，请刷新后重试".to_string());
    }
    Ok(())
}

fn candidate_bindings(
    candidates: &[AdmittedFormalLabelOfflineTrainingDatasetCandidate],
) -> Vec<HistoricalOutcomeOfflineDatasetCandidateBinding> {
    let mut output = candidates
        .iter()
        .map(
            |candidate| HistoricalOutcomeOfflineDatasetCandidateBinding {
                label_id: candidate.formal_label.label.label_id.clone(),
                label_sha256: candidate.formal_label.label.label_sha256.clone(),
                write_claim_id: candidate.formal_label.claim.claim_id.clone(),
                write_claim_sha256: candidate.formal_label.claim.claim_sha256.clone(),
                validation_id: candidate.validation.validation_id.clone(),
                validation_sha256: candidate.validation.validation_sha256.clone(),
            },
        )
        .collect::<Vec<_>>();
    output.sort();
    output
}

fn candidate_set_sha256(
    bindings: &[HistoricalOutcomeOfflineDatasetCandidateBinding],
) -> Result<String, String> {
    let mut canonical = bindings.to_vec();
    canonical.sort();
    hash_serializable(&serde_json::json!({
        "policy_version": POLICY_VERSION,
        "complete_current_candidate_set": canonical,
    }))
}

fn candidate_bindings_extend_dataset(
    current: &[HistoricalOutcomeOfflineDatasetCandidateBinding],
    previous: &HistoricalOutcomeOfflineDataset,
) -> bool {
    let current = current
        .iter()
        .map(|binding| (binding.label_id.as_str(), binding))
        .collect::<BTreeMap<_, _>>();
    previous.entries.iter().all(|entry| {
        current.get(entry.label_id.as_str()).is_some_and(|binding| {
            binding.label_sha256 == entry.label_sha256
                && binding.write_claim_id == entry.write_claim_id
                && binding.write_claim_sha256 == entry.write_claim_sha256
                && binding.validation_id == entry.validation_id
                && binding.validation_sha256 == entry.validation_sha256
        })
    })
}

fn build_entries(
    candidates: &[AdmittedFormalLabelOfflineTrainingDatasetCandidate],
    previous: Option<&HistoricalOutcomeOfflineDataset>,
) -> Result<Vec<HistoricalOutcomeOfflineDatasetEntry>, String> {
    let previous_label_ids = previous
        .map(|dataset| {
            dataset
                .entries
                .iter()
                .map(|entry| entry.label_id.as_str())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut ordered = candidates
        .iter()
        .filter(|candidate| {
            !previous_label_ids.contains(candidate.formal_label.label.label_id.as_str())
        })
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.formal_label
            .label
            .payload
            .decision_available_at
            .cmp(&right.formal_label.label.payload.decision_available_at)
            .then_with(|| {
                left.formal_label
                    .label
                    .payload
                    .asset_symbol
                    .cmp(&right.formal_label.label.payload.asset_symbol)
            })
            .then_with(|| {
                left.formal_label
                    .label
                    .label_id
                    .cmp(&right.formal_label.label.label_id)
            })
    });
    let mut entries = previous
        .map(|dataset| dataset.entries.clone())
        .unwrap_or_default();
    for candidate in ordered {
        entries.push(build_entry(entries.len(), candidate)?);
    }
    Ok(entries)
}

fn build_entry(
    ordinal: usize,
    candidate: &AdmittedFormalLabelOfflineTrainingDatasetCandidate,
) -> Result<HistoricalOutcomeOfflineDatasetEntry, String> {
    let label = &candidate.formal_label.label;
    let claim = &candidate.formal_label.claim;
    let validation = &candidate.validation;
    let mut entry = HistoricalOutcomeOfflineDatasetEntry {
        schema_version: ENTRY_SCHEMA_VERSION.to_string(),
        ordinal,
        entry_id: String::new(),
        entry_sha256: String::new(),
        label_id: label.label_id.clone(),
        label_sha256: label.label_sha256.clone(),
        write_claim_id: claim.claim_id.clone(),
        write_claim_sha256: claim.claim_sha256.clone(),
        validation_id: validation.validation_id.clone(),
        validation_sha256: validation.validation_sha256.clone(),
        authorization_review_sha256: validation.authorization_review_sha256.clone(),
        materialization_validation_sha256: validation.materialization_validation_sha256.clone(),
        materialization_output_sha256: validation.materialization_output_sha256.clone(),
        admission_review_sha256: validation.admission_review_sha256.clone(),
        source_validation_sha256: validation.source_validation_sha256.clone(),
        source_output_sha256: validation.source_output_sha256.clone(),
        snapshot_sha256: validation.snapshot_sha256.clone(),
        reconstruction_sha256: validation.reconstruction_sha256.clone(),
        protocol_sha256: validation.protocol_sha256.clone(),
        recomputed_metrics_sha256: validation.recomputed_metrics_sha256.clone(),
        label_contract_sha256: validation.label_contract_sha256.clone(),
        asset_symbol: label.payload.asset_symbol.clone(),
        benchmark_symbol: label.payload.benchmark_symbol.clone(),
        decision_available_at: label.payload.decision_available_at,
        common_session_count: label.payload.common_session_count,
        raw_validated_metrics: label.payload.raw_validated_metrics.clone(),
        source_provenance: label.payload.source_provenance.clone(),
        known_limitations: label.payload.known_limitations.clone(),
        immutable_chain_bindings: label.payload.immutable_chain_bindings.clone(),
        formal_label_written_by: validation.formal_label_written_by.clone(),
        independently_validated_at: validation.validated_at,
        independently_validated_by: validation.validated_by.clone(),
        excluded_prior_actor_ids: validation.excluded_prior_actor_ids.clone(),
        raw_outcome_only: true,
        feature_vector_present: false,
        semantic_target_assigned: false,
        split_assigned: false,
        reward_present: false,
    };
    entry.entry_sha256 = entry_fingerprint(&entry)?;
    entry.entry_id = entry.entry_sha256[..32].to_string();
    validate_entry(&entry)?;
    Ok(entry)
}

fn validate_unique_entries(entries: &[HistoricalOutcomeOfflineDatasetEntry]) -> Result<(), String> {
    let mut label_ids = BTreeSet::new();
    let mut entry_ids = BTreeSet::new();
    let mut decision_identities = BTreeSet::new();
    for entry in entries {
        let identity = (
            entry.asset_symbol.clone(),
            entry.decision_available_at,
            entry.snapshot_sha256.clone(),
        );
        if !label_ids.insert(entry.label_id.clone())
            || !entry_ids.insert(entry.entry_id.clone())
            || !decision_identities.insert(identity)
        {
            return Err("数据集存在重复标签、条目或冲突的点时决策身份".to_string());
        }
    }
    Ok(())
}

fn validate_entry(entry: &HistoricalOutcomeOfflineDatasetEntry) -> Result<(), String> {
    let downstream_closed = entry.raw_outcome_only
        && !entry.feature_vector_present
        && !entry.semantic_target_assigned
        && !entry.split_assigned
        && !entry.reward_present;
    let horizons = entry
        .raw_validated_metrics
        .iter()
        .map(|metric| metric.horizon_market_sessions)
        .collect::<Vec<_>>();
    if entry.schema_version != ENTRY_SCHEMA_VERSION
        || entry.ordinal > 1_000_000
        || !valid_id(&entry.entry_id)
        || !valid_sha256(&entry.entry_sha256)
        || entry.entry_id != entry.entry_sha256[..32]
        || entry.entry_sha256 != entry_fingerprint(entry)?
        || !valid_id(&entry.label_id)
        || !valid_id(&entry.write_claim_id)
        || !valid_id(&entry.validation_id)
        || !all_sha256(&[
            &entry.label_sha256,
            &entry.write_claim_sha256,
            &entry.validation_sha256,
            &entry.authorization_review_sha256,
            &entry.materialization_validation_sha256,
            &entry.materialization_output_sha256,
            &entry.admission_review_sha256,
            &entry.source_validation_sha256,
            &entry.source_output_sha256,
            &entry.snapshot_sha256,
            &entry.reconstruction_sha256,
            &entry.protocol_sha256,
            &entry.recomputed_metrics_sha256,
            &entry.label_contract_sha256,
        ])
        || entry.asset_symbol.trim().is_empty()
        || entry.benchmark_symbol.trim().is_empty()
        || entry.common_session_count < 251
        || entry.known_limitations.trim().is_empty()
        || entry.formal_label_written_by.trim().is_empty()
        || entry.independently_validated_by.trim().is_empty()
        || entry.excluded_prior_actor_ids.is_empty()
        || !sorted_unique(&entry.excluded_prior_actor_ids)
        || entry
            .excluded_prior_actor_ids
            .iter()
            .any(|actor| actor == &entry.independently_validated_by)
        || horizons != REQUIRED_HORIZONS
        || entry.raw_validated_metrics.iter().any(metric_invalid)
        || hash_serializable(&entry.raw_validated_metrics)? != entry.recomputed_metrics_sha256
        || entry.immutable_chain_bindings.recomputed_metrics_sha256
            != entry.recomputed_metrics_sha256
        || entry.immutable_chain_bindings.protocol_sha256 != entry.protocol_sha256
        || entry.immutable_chain_bindings.snapshot_sha256 != entry.snapshot_sha256
        || entry.immutable_chain_bindings.label_contract_sha256 != entry.label_contract_sha256
        || !downstream_closed
    {
        return Err(
            "historical outcome offline dataset entry is invalid or over-authorized".to_string(),
        );
    }
    Ok(())
}

fn validate_dataset(dataset: &HistoricalOutcomeOfflineDataset) -> Result<(), String> {
    let authority_closed = dataset.copied_to_isolated_offline_dataset_store
        && !dataset.copied_to_training_store
        && !dataset.feature_join_performed
        && !dataset.semantic_targets_assigned
        && !dataset.dataset_governance_approved
        && !dataset.training_authorized
        && !dataset.training_run_allowed
        && !dataset.reward_authorized
        && !dataset.reward_written
        && !dataset.shadow_portfolio_authorized
        && !dataset.order_generation_authorized
        && !dataset.broker_access_authorized
        && !dataset.trading_authorized;
    let entry_bindings = dataset
        .entries
        .iter()
        .map(|entry| HistoricalOutcomeOfflineDatasetCandidateBinding {
            label_id: entry.label_id.clone(),
            label_sha256: entry.label_sha256.clone(),
            write_claim_id: entry.write_claim_id.clone(),
            write_claim_sha256: entry.write_claim_sha256.clone(),
            validation_id: entry.validation_id.clone(),
            validation_sha256: entry.validation_sha256.clone(),
        })
        .collect::<Vec<_>>();
    if dataset.schema_version != DATASET_SCHEMA_VERSION
        || dataset.policy_version != POLICY_VERSION
        || dataset.assembler_implementation_version != ASSEMBLER_IMPLEMENTATION_VERSION
        || dataset.assembler_implementation_sha256 != assembler_implementation_sha256()
        || !valid_id(&dataset.dataset_id)
        || !valid_sha256(&dataset.dataset_content_sha256)
        || !valid_sha256(&dataset.manifest_sha256)
        || dataset.dataset_id != dataset.dataset_content_sha256[..32]
        || dataset.dataset_version
            != format!(
                "historical-raw-outcomes-v1-{:06}-{:.12}",
                dataset.version_number, dataset.dataset_content_sha256
            )
        || dataset.version_number == 0
        || dataset.manifest_sha256 != manifest_fingerprint(dataset)?
        || dataset.dataset_content_sha256
            != dataset_content_sha256(
                dataset.version_number,
                dataset.parent_dataset_id.as_deref(),
                &dataset.candidate_set_sha256,
                &dataset.entries,
            )?
        || !valid_sha256(&dataset.candidate_set_sha256)
        || candidate_set_sha256(&entry_bindings)? != dataset.candidate_set_sha256
        || dataset.assembled_by.trim().is_empty()
        || dataset.purpose != PURPOSE
        || dataset.entries.is_empty()
        || dataset.entry_count != dataset.entries.len()
        || dataset.added_entry_count == 0
        || dataset.added_entry_count > dataset.entry_count
        || dataset.distinct_symbol_count
            != dataset
                .entries
                .iter()
                .map(|entry| entry.asset_symbol.as_str())
                .collect::<BTreeSet<_>>()
                .len()
        || dataset.earliest_decision_available_at
            != dataset
                .entries
                .iter()
                .map(|entry| entry.decision_available_at)
                .min()
                .expect("non-empty")
        || dataset.latest_decision_available_at
            != dataset
                .entries
                .iter()
                .map(|entry| entry.decision_available_at)
                .max()
                .expect("non-empty")
        || !dataset.complete_candidate_set_frozen
        || !dataset.monotonic_append_only_lineage
        || !dataset.point_in_time_lineage_preserved
        || !dataset.duplicate_labels_rejected
        || !dataset.conflicting_decision_identities_rejected
        || dataset.split_policy_status != SPLIT_POLICY_STATUS
        || !authority_closed
    {
        return Err("historical outcome offline dataset is invalid or over-authorized".to_string());
    }
    for (ordinal, entry) in dataset.entries.iter().enumerate() {
        validate_entry(entry)?;
        if entry.ordinal != ordinal {
            return Err("historical outcome offline dataset entry order is invalid".to_string());
        }
    }
    validate_unique_entries(&dataset.entries)
}

fn validate_dataset_chain(datasets: &[HistoricalOutcomeOfflineDataset]) -> Result<(), String> {
    let mut prior: Option<&HistoricalOutcomeOfflineDataset> = None;
    for dataset in datasets {
        validate_dataset(dataset)?;
        let expected_version = prior.map(|value| value.version_number + 1).unwrap_or(1);
        if dataset.version_number != expected_version
            || dataset.parent_dataset_id != prior.map(|value| value.dataset_id.clone())
            || dataset.parent_manifest_sha256 != prior.map(|value| value.manifest_sha256.clone())
        {
            return Err(
                "historical outcome offline dataset lineage is forked or disconnected".to_string(),
            );
        }
        if let Some(previous) = prior {
            if dataset.entry_count <= previous.entry_count
                || dataset.added_entry_count != dataset.entry_count - previous.entry_count
                || previous
                    .entries
                    .iter()
                    .zip(&dataset.entries)
                    .any(|(left, right)| {
                        left.entry_id != right.entry_id || left.entry_sha256 != right.entry_sha256
                    })
            {
                return Err(
                    "historical outcome offline dataset lineage is not monotonic".to_string(),
                );
            }
        } else if dataset.added_entry_count != dataset.entry_count {
            return Err("first historical outcome dataset has invalid added count".to_string());
        }
        prior = Some(dataset);
    }
    Ok(())
}

fn dataset_content_sha256(
    version_number: usize,
    parent_dataset_id: Option<&str>,
    candidate_set_sha256: &str,
    entries: &[HistoricalOutcomeOfflineDatasetEntry],
) -> Result<String, String> {
    hash_serializable(&serde_json::json!({
        "schema_version": DATASET_SCHEMA_VERSION,
        "policy_version": POLICY_VERSION,
        "assembler_implementation_sha256": assembler_implementation_sha256(),
        "version_number": version_number,
        "parent_dataset_id": parent_dataset_id,
        "candidate_set_sha256": candidate_set_sha256,
        "entries": entries,
    }))
}

fn entry_fingerprint(entry: &HistoricalOutcomeOfflineDatasetEntry) -> Result<String, String> {
    fingerprint_without(entry, &["entry_id", "entry_sha256"])
}

fn manifest_fingerprint(dataset: &HistoricalOutcomeOfflineDataset) -> Result<String, String> {
    fingerprint_without(dataset, &["manifest_sha256"])
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

fn assembler_implementation_sha256() -> String {
    hash_serializable(&serde_json::json!({
        "implementation": ASSEMBLER_IMPLEMENTATION_VERSION,
        "policy": POLICY_VERSION,
        "candidate_scope": "complete_current_stage22_pass_set",
        "version_lineage": "monotonic_append_only",
        "entry_order": "decision_available_at_asset_symbol_label_id",
        "content_addressed": true,
        "split_assignment": false,
        "feature_join": false,
        "semantic_target_assignment": false,
        "training_or_execution_authority": false,
    }))
    .expect("static assembler implementation is serializable")
}

async fn read_datasets(state: &AppState) -> Result<Vec<HistoricalOutcomeOfflineDataset>, String> {
    let root = dataset_root(state);
    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut output = Vec::new();
    let mut ids = BTreeSet::new();
    let mut manifests = BTreeSet::new();
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
        let dataset: HistoricalOutcomeOfflineDataset =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        validate_dataset(&dataset)?;
        if entry.path().file_stem().and_then(|value| value.to_str())
            != Some(dataset.dataset_id.as_str())
            || !ids.insert(dataset.dataset_id.clone())
            || !manifests.insert(dataset.manifest_sha256.clone())
        {
            return Err(
                "historical outcome offline datasets contain replay or path mismatch".to_string(),
            );
        }
        output.push(dataset);
    }
    output.sort_by_key(|dataset| dataset.version_number);
    validate_dataset_chain(&output)?;
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

fn dataset_root(state: &AppState) -> PathBuf {
    decision_root(state).join("historical_outcome_offline_datasets/objects")
}

struct AssemblyLock {
    path: PathBuf,
}

impl Drop for AssemblyLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn acquire_lock(state: &AppState) -> Result<AssemblyLock, String> {
    let directory = decision_root(state).join("governance-locks");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let path = directory.join("historical-outcome-offline-dataset-assembly.lock");
    for attempt in 0..8 {
        match tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .await
        {
            Ok(_) => return Ok(AssemblyLock { path }),
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
                return Err("另一个管理员正在装配历史结果离线数据集".to_string());
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("历史结果离线数据集装配锁获取失败".to_string())
}

fn metric_invalid(metric: &HistoricalOutcomeDryRunMetric) -> bool {
    metric.end_date < metric.start_date
        || !metric.asset_return.is_finite()
        || !metric.benchmark_return.is_finite()
        || !metric.excess_return.is_finite()
        || !metric.asset_max_drawdown.is_finite()
        || metric.asset_max_drawdown > 0.0
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

    fn bindings() -> Vec<HistoricalOutcomeOfflineDatasetCandidateBinding> {
        vec![
            HistoricalOutcomeOfflineDatasetCandidateBinding {
                label_id: "b".repeat(32),
                label_sha256: "b".repeat(64),
                write_claim_id: "c".repeat(32),
                write_claim_sha256: "c".repeat(64),
                validation_id: "d".repeat(32),
                validation_sha256: "d".repeat(64),
            },
            HistoricalOutcomeOfflineDatasetCandidateBinding {
                label_id: "a".repeat(32),
                label_sha256: "a".repeat(64),
                write_claim_id: "e".repeat(32),
                write_claim_sha256: "e".repeat(64),
                validation_id: "f".repeat(32),
                validation_sha256: "f".repeat(64),
            },
        ]
    }

    fn metrics() -> Vec<HistoricalOutcomeDryRunMetric> {
        REQUIRED_HORIZONS
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
    fn candidate_set_fingerprint_is_order_independent_but_binding_sensitive() {
        let first = bindings();
        let mut reversed = first.clone();
        reversed.reverse();
        assert_eq!(
            candidate_set_sha256(&first).unwrap(),
            candidate_set_sha256(&reversed).unwrap()
        );
        reversed[0].validation_sha256 = "1".repeat(64);
        assert_ne!(
            candidate_set_sha256(&first).unwrap(),
            candidate_set_sha256(&reversed).unwrap()
        );
    }

    #[test]
    fn request_requires_the_complete_exact_candidate_set_and_closed_authority() {
        let mut current = bindings();
        current.sort();
        let request = AssembleHistoricalOutcomeOfflineDatasetRequest {
            expected_candidate_set_sha256: candidate_set_sha256(&current).unwrap(),
            expected_candidates: current.clone(),
            purpose: PURPOSE.to_string(),
            complete_current_candidate_set_confirmed: true,
            monotonic_version_lineage_confirmed: true,
            point_in_time_lineage_preserved_confirmed: true,
            no_semantic_target_or_split_inference_confirmed: true,
            no_training_reward_shadow_order_broker_or_trading_confirmed: true,
        };
        verify_request(&request, &current).unwrap();
        let mut missing = request;
        missing.expected_candidates.pop();
        assert!(verify_request(&missing, &current).is_err());
    }

    #[test]
    fn metric_vector_hash_detects_one_ulp_change() {
        let original = metrics();
        let mut changed = original.clone();
        changed[1].excess_return = f64::from_bits(changed[1].excess_return.to_bits() + 1);
        assert_ne!(
            hash_serializable(&original).unwrap(),
            hash_serializable(&changed).unwrap()
        );
    }

    #[test]
    fn duplicate_or_conflicting_decision_identity_fails_closed() {
        let now = Utc::now();
        let mut first = empty_entry(now, "SNDK", "a");
        first.entry_id = "a".repeat(32);
        first.entry_sha256 = "a".repeat(64);
        let mut second = empty_entry(now, "SNDK", "b");
        second.snapshot_sha256 = first.snapshot_sha256.clone();
        second.immutable_chain_bindings.snapshot_sha256 = first.snapshot_sha256.clone();
        second.entry_id = "b".repeat(32);
        second.entry_sha256 = "b".repeat(64);
        assert!(validate_unique_entries(&[first, second]).is_err());
    }

    #[test]
    fn dataset_lineage_must_preserve_prior_entries_and_parent_hash() {
        let now = Utc::now();
        let first_entry = empty_entry(now, "SNDK", "a");
        let first = test_dataset(1, None, vec![first_entry.clone()]);
        let second_entry = empty_entry(now - chrono::Duration::days(1), "NVDA", "b");
        let second = test_dataset(2, Some(&first), vec![first_entry, second_entry]);
        validate_dataset_chain(&[first.clone(), second.clone()]).unwrap();
        let mut fork = second;
        fork.parent_manifest_sha256 = Some("f".repeat(64));
        fork.manifest_sha256 = manifest_fingerprint(&fork).unwrap();
        assert!(validate_dataset_chain(&[first, fork]).is_err());
    }

    #[test]
    fn assembled_dataset_never_grants_training_reward_or_execution_authority() {
        let dataset = test_dataset(1, None, vec![empty_entry(Utc::now(), "SNDK", "a")]);
        validate_dataset(&dataset).unwrap();
        assert!(dataset.copied_to_isolated_offline_dataset_store);
        assert!(!dataset.copied_to_training_store);
        assert!(!dataset.feature_join_performed);
        assert!(!dataset.semantic_targets_assigned);
        assert!(!dataset.dataset_governance_approved);
        assert!(!dataset.training_authorized);
        assert!(!dataset.training_run_allowed);
        assert!(!dataset.reward_authorized);
        assert!(!dataset.shadow_portfolio_authorized);
        assert!(!dataset.trading_authorized);
    }

    fn empty_entry(
        decision_available_at: DateTime<Utc>,
        symbol: &str,
        seed: &str,
    ) -> HistoricalOutcomeOfflineDatasetEntry {
        let raw_validated_metrics = metrics();
        let recomputed_metrics_sha256 = hash_serializable(&raw_validated_metrics).unwrap();
        let mut entry = HistoricalOutcomeOfflineDatasetEntry {
            schema_version: ENTRY_SCHEMA_VERSION.to_string(),
            ordinal: 0,
            entry_id: String::new(),
            entry_sha256: String::new(),
            label_id: seed.repeat(32),
            label_sha256: seed.repeat(64),
            write_claim_id: seed.repeat(32),
            write_claim_sha256: seed.repeat(64),
            validation_id: seed.repeat(32),
            validation_sha256: seed.repeat(64),
            authorization_review_sha256: seed.repeat(64),
            materialization_validation_sha256: seed.repeat(64),
            materialization_output_sha256: seed.repeat(64),
            admission_review_sha256: seed.repeat(64),
            source_validation_sha256: seed.repeat(64),
            source_output_sha256: seed.repeat(64),
            snapshot_sha256: seed.repeat(64),
            reconstruction_sha256: seed.repeat(64),
            protocol_sha256: seed.repeat(64),
            recomputed_metrics_sha256: recomputed_metrics_sha256.clone(),
            label_contract_sha256: seed.repeat(64),
            asset_symbol: symbol.to_string(),
            benchmark_symbol: "SPY".to_string(),
            decision_available_at,
            common_session_count: 251,
            raw_validated_metrics,
            source_provenance: HistoricalOutcomeFormalLabelSourceProvenance {
                materialization_validation_id: seed.repeat(32),
                materialization_validated_at: decision_available_at,
                materialization_validated_by: "materialization-validator".to_string(),
                materialization_attempt_id: seed.repeat(32),
                source_attempt_id: seed.repeat(32),
                source_validation_id: seed.repeat(32),
                snapshot_id: seed.repeat(32),
                reconstruction_id: seed.repeat(32),
                protocol_version: "v1".to_string(),
                metric_start_date: NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
                metric_end_dates: REQUIRED_HORIZONS
                    .iter()
                    .map(|horizon| {
                        NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()
                            + chrono::Days::new(u64::from(*horizon))
                    })
                    .collect(),
            },
            known_limitations: "测试局限".to_string(),
            immutable_chain_bindings: HistoricalOutcomeFormalLabelImmutableChainBindings {
                authorization_review_id: seed.repeat(32),
                authorization_review_sha256: seed.repeat(64),
                materialization_validation_sha256: seed.repeat(64),
                materialization_claim_sha256: seed.repeat(64),
                materialization_result_sha256: seed.repeat(64),
                materialization_output_sha256: seed.repeat(64),
                admission_review_sha256: seed.repeat(64),
                source_validation_sha256: seed.repeat(64),
                source_output_sha256: seed.repeat(64),
                snapshot_sha256: seed.repeat(64),
                reconstruction_sha256: seed.repeat(64),
                protocol_sha256: seed.repeat(64),
                recomputed_metrics_sha256,
                label_contract_sha256: seed.repeat(64),
            },
            formal_label_written_by: "writer".to_string(),
            independently_validated_at: decision_available_at,
            independently_validated_by: "independent-validator".to_string(),
            excluded_prior_actor_ids: vec!["reviewer".to_string(), "writer".to_string()],
            raw_outcome_only: true,
            feature_vector_present: false,
            semantic_target_assigned: false,
            split_assigned: false,
            reward_present: false,
        };
        entry.entry_sha256 = entry_fingerprint(&entry).unwrap();
        entry.entry_id = entry.entry_sha256[..32].to_string();
        entry
    }

    fn test_dataset(
        version_number: usize,
        parent: Option<&HistoricalOutcomeOfflineDataset>,
        mut entries: Vec<HistoricalOutcomeOfflineDatasetEntry>,
    ) -> HistoricalOutcomeOfflineDataset {
        for (ordinal, entry) in entries.iter_mut().enumerate() {
            entry.ordinal = ordinal;
            entry.entry_sha256 = entry_fingerprint(entry).unwrap();
            entry.entry_id = entry.entry_sha256[..32].to_string();
        }
        let bindings = entries
            .iter()
            .map(|entry| HistoricalOutcomeOfflineDatasetCandidateBinding {
                label_id: entry.label_id.clone(),
                label_sha256: entry.label_sha256.clone(),
                write_claim_id: entry.write_claim_id.clone(),
                write_claim_sha256: entry.write_claim_sha256.clone(),
                validation_id: entry.validation_id.clone(),
                validation_sha256: entry.validation_sha256.clone(),
            })
            .collect::<Vec<_>>();
        let candidate_set_sha256 = candidate_set_sha256(&bindings).unwrap();
        let content = dataset_content_sha256(
            version_number,
            parent.map(|value| value.dataset_id.as_str()),
            &candidate_set_sha256,
            &entries,
        )
        .unwrap();
        let mut dataset = HistoricalOutcomeOfflineDataset {
            schema_version: DATASET_SCHEMA_VERSION.to_string(),
            policy_version: POLICY_VERSION.to_string(),
            assembler_implementation_version: ASSEMBLER_IMPLEMENTATION_VERSION.to_string(),
            assembler_implementation_sha256: assembler_implementation_sha256(),
            dataset_id: content[..32].to_string(),
            dataset_version: format!(
                "historical-raw-outcomes-v1-{:06}-{:.12}",
                version_number, content
            ),
            version_number,
            dataset_content_sha256: content,
            manifest_sha256: String::new(),
            parent_dataset_id: parent.map(|value| value.dataset_id.clone()),
            parent_manifest_sha256: parent.map(|value| value.manifest_sha256.clone()),
            candidate_set_sha256,
            assembled_at: Utc::now(),
            assembled_by: "assembler".to_string(),
            purpose: PURPOSE.to_string(),
            entry_count: entries.len(),
            added_entry_count: entries.len() - parent.map(|value| value.entry_count).unwrap_or(0),
            distinct_symbol_count: entries
                .iter()
                .map(|entry| entry.asset_symbol.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            earliest_decision_available_at: entries
                .iter()
                .map(|entry| entry.decision_available_at)
                .min()
                .unwrap(),
            latest_decision_available_at: entries
                .iter()
                .map(|entry| entry.decision_available_at)
                .max()
                .unwrap(),
            entries,
            complete_candidate_set_frozen: true,
            monotonic_append_only_lineage: true,
            point_in_time_lineage_preserved: true,
            duplicate_labels_rejected: true,
            conflicting_decision_identities_rejected: true,
            split_policy_status: SPLIT_POLICY_STATUS.to_string(),
            copied_to_isolated_offline_dataset_store: true,
            copied_to_training_store: false,
            feature_join_performed: false,
            semantic_targets_assigned: false,
            dataset_governance_approved: false,
            training_authorized: false,
            training_run_allowed: false,
            reward_authorized: false,
            reward_written: false,
            shadow_portfolio_authorized: false,
            order_generation_authorized: false,
            broker_access_authorized: false,
            trading_authorized: false,
        };
        dataset.manifest_sha256 = manifest_fingerprint(&dataset).unwrap();
        dataset
    }
}
