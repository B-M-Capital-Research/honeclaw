//! Strict, point-in-time issuer operating-KPI claims.
//!
//! This is a separate claim family from GAAP facts and broad management
//! commentary.  It exists to retain the issuer's own metric name, verbatim
//! definition, measurement scope and period semantics before any causal or
//! investment interpretation. Admission is bounded by a shared six-model
//! catalog: every supported symbol maps to one first-principles model and a
//! fixed KPI allowlist. The model may extract only those IDs, and every issuer
//! name, definition, value and quote must still be verified verbatim against
//! the bounded primary source before persistence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::earnings_claim::EarningsClaimDisposition;
use crate::event::{EventKind, MarketEvent};

pub const OPERATING_KPI_POLICY_STATUS: &str = "training_only_pending_human_review";
pub const OPERATING_KPI_CLAIM_SCHEMA_VERSION: &str = "hone-operating-kpi-claim-v1";
pub const OPERATING_KPI_CATALOG_VERSION: &str =
    "hone-operating-kpi-catalog-v3-storage-nbm-rpo-source-bounded";
pub const OPERATING_KPI_SOURCE_ARTIFACT_SCHEMA_VERSION: &str =
    "hone-operating-kpi-source-artifact-v1-content-addressed";
pub const OPERATING_KPI_BACKFILL_EVENT_SCHEMA_VERSION: &str =
    "hone-operating-kpi-backfill-event-v2-source-artifact";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperatingKpiCatalogEntry {
    pub kpi_id: &'static str,
    pub label: &'static str,
    pub driver_id: &'static str,
    pub milestone_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatingKpiSourceArtifact {
    pub schema_version: String,
    pub source_sha256: String,
    pub extracted_text_sha256: String,
    pub byte_length: u64,
    pub format: String,
    pub object_path: String,
}

const STORAGE_SYMBOLS: &[&str] = &["MU", "SKHY", "SNDK", "STX", "WDC"];
const COMPUTE_SYMBOLS: &[&str] = &[
    "ALAB", "AMAT", "AMD", "ARM", "AVGO", "CBRS", "INTC", "KLAC", "LRCX", "MRVL", "NVDA", "QCOM",
    "TER", "TSM",
];
const OPTICAL_SYMBOLS: &[&str] = &[
    "AAOI", "ANET", "CIEN", "COHR", "CRDO", "CSCO", "FN", "GLW", "LITE", "NOK", "SIVEF",
];
const POWER_SYMBOLS: &[&str] = &["BE", "BWXT", "GEV", "SBET", "VRT", "VST"];
const PLATFORM_SYMBOLS: &[&str] = &[
    "AMZN", "APP", "CRWV", "DELL", "FIG", "GOOGL", "IREN", "META", "MSFT", "NBIS",
];
const APPLICATION_SYMBOLS: &[&str] = &["CAI", "RXRX", "TEM"];

const STORAGE_KPIS: &[OperatingKpiCatalogEntry] = &[
    catalog_entry(
        "nand_bit_shipments_growth",
        "NAND 位元出货增长",
        "realized_bit_demand",
        false,
    ),
    catalog_entry(
        "nand_asp_change",
        "NAND 平均售价变化",
        "inventory_price",
        false,
    ),
    catalog_entry(
        "enterprise_ssd_mix",
        "企业级 SSD 收入或出货占比",
        "share_content",
        false,
    ),
    catalog_entry(
        "enterprise_ssd_qualification",
        "企业级 SSD 认证里程碑",
        "yield_qualification",
        true,
    ),
    catalog_entry(
        "nand_capacity_utilization",
        "NAND 产能利用率",
        "wafer_bits",
        false,
    ),
    catalog_entry(
        "data_center_storage_orders",
        "数据中心存储订单或积压订单",
        "enterprise_adoption",
        true,
    ),
    catalog_entry(
        "data_center_revenue_growth",
        "数据中心收入增长",
        "enterprise_adoption",
        false,
    ),
    catalog_entry(
        "signed_storage_supply_agreements",
        "已签署存储供应协议数量",
        "enterprise_adoption",
        true,
    ),
    catalog_entry(
        "storage_nbm_remaining_performance_obligations",
        "存储 NBM 剩余履约义务",
        "enterprise_adoption",
        true,
    ),
];
const COMPUTE_KPIS: &[OperatingKpiCatalogEntry] = &[
    catalog_entry(
        "accelerator_or_asic_units",
        "加速器或 ASIC 出货量",
        "architecture_content",
        false,
    ),
    catalog_entry(
        "advanced_packaging_capacity",
        "先进封装合格产能",
        "wafer_packaging",
        false,
    ),
    catalog_entry(
        "compute_product_yield",
        "关键产品合格良率",
        "yield_lead_time",
        false,
    ),
    catalog_entry(
        "compute_backlog_lead_time",
        "积压订单与交付周期",
        "yield_lead_time",
        true,
    ),
    catalog_entry(
        "deployed_compute_power",
        "已上线算力对应电力",
        "power_deployment",
        true,
    ),
];
const OPTICAL_KPIS: &[OperatingKpiCatalogEntry] = &[
    catalog_entry(
        "high_speed_optical_mix",
        "800G/1.6T 收入或出货占比",
        "bandwidth_generation",
        false,
    ),
    catalog_entry(
        "high_speed_optical_shipments",
        "高速光模块或端口出货",
        "accelerator_ports",
        false,
    ),
    catalog_entry(
        "optical_customer_qualification",
        "高速光产品客户认证",
        "yield_qualification",
        true,
    ),
    catalog_entry(
        "optical_lead_time",
        "高速光产品交付周期",
        "deployment_inventory",
        false,
    ),
    catalog_entry(
        "laser_dsp_qualified_capacity",
        "激光器/DSP 合格产能",
        "laser_dsp_capacity",
        false,
    ),
];
const POWER_KPIS: &[OperatingKpiCatalogEntry] = &[
    catalog_entry(
        "contracted_power_mw",
        "已签约电力容量",
        "deployed_compute_mw",
        true,
    ),
    catalog_entry(
        "energized_power_mw",
        "已交付或已上电容量",
        "deployed_compute_mw",
        true,
    ),
    catalog_entry(
        "generation_equipment_backlog",
        "电力设备或项目积压订单（公司披露单位）",
        "generation_equipment",
        true,
    ),
    catalog_entry(
        "power_equipment_capacity",
        "电力设备合格交付产能",
        "generation_equipment",
        false,
    ),
    catalog_entry(
        "grid_interconnection_milestone",
        "并网与许可里程碑",
        "grid_interconnect",
        true,
    ),
    catalog_entry(
        "liquid_cooling_penetration",
        "液冷部署占比",
        "rack_density",
        false,
    ),
];
const PLATFORM_KPIS: &[OperatingKpiCatalogEntry] = &[
    catalog_entry(
        "ai_annualized_revenue",
        "AI 年化收入或 ARR",
        "paid_workloads",
        false,
    ),
    catalog_entry(
        "ai_rpo_or_bookings",
        "AI 剩余履约义务或新增订单",
        "paid_workloads",
        false,
    ),
    catalog_entry(
        "token_or_call_volume",
        "Token 或调用量",
        "usage_intensity",
        false,
    ),
    catalog_entry(
        "production_ai_customers",
        "生产级 AI 客户数",
        "paid_workloads",
        false,
    ),
    catalog_entry(
        "inference_unit_cost",
        "单位推理成本",
        "model_efficiency",
        false,
    ),
    catalog_entry(
        "capital_expenditure",
        "资本开支",
        "capacity_deployment",
        false,
    ),
];
const APPLICATION_KPIS: &[OperatingKpiCatalogEntry] = &[
    catalog_entry("application_arr", "应用 ARR", "adoption_usage", false),
    catalog_entry(
        "net_revenue_retention",
        "净收入留存率",
        "adoption_usage",
        false,
    ),
    catalog_entry(
        "production_customer_count",
        "生产客户数",
        "adoption_usage",
        false,
    ),
    catalog_entry(
        "workflow_usage_volume",
        "工作流使用量",
        "adoption_usage",
        false,
    ),
    catalog_entry(
        "verified_outcome_quality",
        "可验证结果质量",
        "outcome_quality",
        true,
    ),
    catalog_entry(
        "implementation_backlog",
        "实施积压与交付周期",
        "delivery_capacity",
        false,
    ),
];

const fn catalog_entry(
    kpi_id: &'static str,
    label: &'static str,
    driver_id: &'static str,
    milestone_allowed: bool,
) -> OperatingKpiCatalogEntry {
    OperatingKpiCatalogEntry {
        kpi_id,
        label,
        driver_id,
        milestone_allowed,
    }
}

pub fn operating_kpi_model_id_for_symbol(symbol: &str) -> Option<&'static str> {
    let symbol = normalize_symbol(symbol);
    if STORAGE_SYMBOLS.contains(&symbol.as_str()) {
        Some("ai-storage-demand-supply")
    } else if COMPUTE_SYMBOLS.contains(&symbol.as_str()) {
        Some("ai-compute-effective-capacity")
    } else if OPTICAL_SYMBOLS.contains(&symbol.as_str()) {
        Some("ai-optical-interconnect-bandwidth")
    } else if POWER_SYMBOLS.contains(&symbol.as_str()) {
        Some("ai-data-center-power-delivery")
    } else if PLATFORM_SYMBOLS.contains(&symbol.as_str()) {
        Some("ai-platform-token-economics")
    } else if APPLICATION_SYMBOLS.contains(&symbol.as_str()) {
        Some("ai-application-workflow-value")
    } else {
        None
    }
}

pub fn operating_kpi_catalog_for_model(
    model_id: &str,
) -> Option<&'static [OperatingKpiCatalogEntry]> {
    match model_id {
        "ai-storage-demand-supply" => Some(STORAGE_KPIS),
        "ai-compute-effective-capacity" => Some(COMPUTE_KPIS),
        "ai-optical-interconnect-bandwidth" => Some(OPTICAL_KPIS),
        "ai-data-center-power-delivery" => Some(POWER_KPIS),
        "ai-platform-token-economics" => Some(PLATFORM_KPIS),
        "ai-application-workflow-value" => Some(APPLICATION_KPIS),
        _ => None,
    }
}

pub fn operating_kpi_catalog_for_symbol(
    symbol: &str,
) -> Option<&'static [OperatingKpiCatalogEntry]> {
    operating_kpi_model_id_for_symbol(symbol).and_then(operating_kpi_catalog_for_model)
}

pub fn operating_kpi_input_is_supported_for_symbol(
    symbol: &str,
    input: &OperatingKpiClaimInput,
) -> bool {
    let kpi_id = normalize_id(&input.kpi_id);
    operating_kpi_catalog_for_symbol(symbol).is_some_and(|entries| {
        entries.iter().any(|entry| {
            entry.kpi_id == kpi_id
                && (entry.milestone_allowed
                    || !matches!(input.claim_kind, OperatingKpiClaimKind::ContractMilestone))
        })
    })
}

/// Returns a short dynamic prompt appendix. It exposes only the current
/// issuer's catalog so the model cannot borrow an unrelated industry's metric.
pub fn operating_kpi_prompt_for_symbol(symbol: &str) -> String {
    let normalized = normalize_symbol(symbol);
    let Some(model_id) = operating_kpi_model_id_for_symbol(&normalized) else {
        return format!(
            "\n本次 Ticker {normalized} 不在经营 KPI 目录中；operating_kpi_claims 必须输出空数组。"
        );
    };
    let entries = operating_kpi_catalog_for_model(model_id).unwrap_or_default();
    let candidates = entries
        .iter()
        .map(|entry| format!("{}（{}）", entry.kpi_id, entry.label))
        .collect::<Vec<_>>()
        .join("、");
    format!(
        "\n本次 Ticker {normalized} 对应第一性原理模型 {model_id}。operating_kpi_claims 只能使用以下 KPI ID：{candidates}。没有逐字公司原始名称、定义、期间、值和证据摘录时必须留空；不得用行业数据、模型常识或其他公司数据补齐。目录版本：{OPERATING_KPI_CATALOG_VERSION}。"
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatingKpiClaimKind {
    ReportedFact,
    ManagementGuidance,
    ContractMilestone,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatingKpiComparisonBasis {
    YearOverYear,
    SequentialQuarter,
    PointInTime,
    PeriodTotal,
    PeriodAverage,
    PeriodEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperatingKpiClaimInput {
    pub claim_kind: OperatingKpiClaimKind,
    pub kpi_id: String,
    /// Exact metric name used by the issuer in the source material.
    pub issuer_metric_name: String,
    /// A short verbatim definition copied from the admitted source excerpt.
    pub issuer_definition: String,
    pub period: String,
    #[serde(default)]
    pub numeric_value: Option<f64>,
    #[serde(default)]
    pub unit: String,
    pub value_text: String,
    pub measurement_scope: String,
    pub comparison_basis: OperatingKpiComparisonBasis,
    #[serde(default)]
    pub speaker: String,
    pub evidence_quote: String,
    pub source_locator: String,
    /// True only when the source explicitly announces a changed definition.
    #[serde(default)]
    pub definition_changed: bool,
    #[serde(default)]
    pub disposition: EarningsClaimDisposition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperatingKpiSourceClaim {
    pub schema_version: String,
    pub claim_id: String,
    pub source_event_id: String,
    pub symbol: String,
    pub claim_kind: OperatingKpiClaimKind,
    pub kpi_id: String,
    pub issuer_metric_name: String,
    pub issuer_definition: String,
    pub definition_key: String,
    pub period: String,
    pub numeric_value: Option<f64>,
    pub unit: String,
    pub value_text: String,
    pub measurement_scope: String,
    pub comparison_basis: OperatingKpiComparisonBasis,
    pub speaker: Option<String>,
    pub evidence_quote: String,
    pub source_locator: String,
    pub source_document: String,
    pub source_name: String,
    pub source_url: String,
    pub published_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_time_precision: Option<String>,
    pub source_tier: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_artifact: Option<OperatingKpiSourceArtifact>,
    pub definition_changed: bool,
    pub policy_status: String,
    pub disposition: EarningsClaimDisposition,
}

/// Reads only explicit typed arrays created from a source-bounded review.
/// Prose, summaries and generic earnings claims are never reinterpreted as an
/// operating KPI.
pub fn operating_kpi_claims_from_event(event: &MarketEvent) -> Vec<OperatingKpiSourceClaim> {
    let Some(symbol) = event
        .symbols
        .first()
        .map(|value| normalize_symbol(value))
        .filter(|value| operating_kpi_catalog_for_symbol(value).is_some())
    else {
        return Vec::new();
    };
    let Some(source_url) = event
        .url
        .as_deref()
        .map(str::trim)
        .filter(|url| url.starts_with("https://"))
    else {
        return Vec::new();
    };
    let Some((payload_pointer, source_document, source_tier)) = claim_source(event) else {
        return Vec::new();
    };
    if !operating_kpi_source_was_verified(event) {
        return Vec::new();
    }
    let source_artifact = operating_kpi_source_artifact_from_event(event);
    let source_time_precision = event
        .payload
        .get("operating_kpi_source_time_precision")
        .and_then(|value| value.as_str())
        .filter(|value| matches!(*value, "exact" | "date_only_conservative_end_of_day"))
        .map(str::to_string);
    if event
        .payload
        .get("operating_kpi_backfill_schema_version")
        .and_then(|value| value.as_str())
        == Some(OPERATING_KPI_BACKFILL_EVENT_SCHEMA_VERSION)
        && source_artifact.is_none()
    {
        return Vec::new();
    }
    let Some(values) = event
        .payload
        .pointer(payload_pointer)
        .and_then(|value| value.as_array())
    else {
        return Vec::new();
    };

    values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let input = serde_json::from_value::<OperatingKpiClaimInput>(value.clone()).ok()?;
            valid_input(&symbol, &input).then(|| OperatingKpiSourceClaim {
                schema_version: OPERATING_KPI_CLAIM_SCHEMA_VERSION.to_string(),
                claim_id: format!("operating-kpi-claim:{}:{index}", event.id),
                source_event_id: event.id.clone(),
                symbol: symbol.clone(),
                claim_kind: input.claim_kind,
                kpi_id: normalize_id(&input.kpi_id),
                issuer_metric_name: truncate(input.issuer_metric_name.trim(), 120),
                issuer_definition: truncate(input.issuer_definition.trim(), 320),
                definition_key: definition_key(&input.issuer_definition),
                period: truncate(input.period.trim(), 100),
                numeric_value: input.numeric_value,
                unit: input.unit.trim().to_string(),
                value_text: truncate(input.value_text.trim(), 240),
                measurement_scope: truncate(input.measurement_scope.trim(), 240),
                comparison_basis: input.comparison_basis,
                speaker: (!input.speaker.trim().is_empty())
                    .then(|| truncate(input.speaker.trim(), 100)),
                evidence_quote: truncate(input.evidence_quote.trim(), 320),
                source_locator: truncate(input.source_locator.trim(), 160),
                source_document: source_document.to_string(),
                source_name: truncate(event.source.trim(), 160),
                source_url: source_url.to_string(),
                published_at: event.occurred_at,
                source_time_precision: source_time_precision.clone(),
                source_tier: source_tier.to_string(),
                source_artifact: source_artifact.clone(),
                definition_changed: input.definition_changed,
                policy_status: OPERATING_KPI_POLICY_STATUS.to_string(),
                disposition: input.disposition,
            })
        })
        .take(12)
        .collect()
}

pub fn operating_kpi_source_artifact_from_event(
    event: &MarketEvent,
) -> Option<OperatingKpiSourceArtifact> {
    let artifact = serde_json::from_value::<OperatingKpiSourceArtifact>(
        event.payload.get("operating_kpi_source_artifact")?.clone(),
    )
    .ok()?;
    operating_kpi_source_artifact_is_valid(&artifact).then_some(artifact)
}

pub fn operating_kpi_source_artifact_is_valid(artifact: &OperatingKpiSourceArtifact) -> bool {
    artifact.schema_version == OPERATING_KPI_SOURCE_ARTIFACT_SCHEMA_VERSION
        && valid_sha256(&artifact.source_sha256)
        && valid_sha256(&artifact.extracted_text_sha256)
        && (1..=5 * 1024 * 1024).contains(&artifact.byte_length)
        && matches!(artifact.format.as_str(), "pdf" | "html")
        && artifact.object_path == format!("objects/{}.{}", artifact.source_sha256, artifact.format)
}

/// Rechecks model-emitted names, definitions and evidence against the bounded
/// source body while it is still in memory.  A model cannot self-attest by
/// repeating the same invented phrase in `issuer_definition` and
/// `evidence_quote`.
pub fn operating_kpi_input_is_verbatim_in_source(
    input: &OperatingKpiClaimInput,
    source_text: &str,
) -> bool {
    let source = normalize_source_text(source_text);
    let contains = |value: &str| {
        let needle = normalize_source_text(value);
        !needle.is_empty() && source.contains(&needle)
    };
    contains(&input.issuer_metric_name)
        && contains(&input.issuer_definition)
        && contains(&input.value_text)
        && contains(&input.evidence_quote)
        && (matches!(input.claim_kind, OperatingKpiClaimKind::ReportedFact)
            || contains(&input.speaker))
}

fn operating_kpi_source_was_verified(event: &MarketEvent) -> bool {
    let key = match &event.kind {
        EventKind::EarningsCallTranscript => "earnings_transcript_operating_kpi_source_verified",
        EventKind::EarningsReleased => "earnings_quality_operating_kpi_source_verified",
        EventKind::SecFiling { form }
            if form.eq_ignore_ascii_case("10-Q")
                || form.eq_ignore_ascii_case("10-K")
                || form.eq_ignore_ascii_case("8-K") =>
        {
            "operating_kpi_source_verified"
        }
        EventKind::NewsCritical => "operating_kpi_source_verified",
        _ => return false,
    };
    event.payload.get(key).and_then(|value| value.as_bool()) == Some(true)
}

fn claim_source(event: &MarketEvent) -> Option<(&'static str, &'static str, &'static str)> {
    match &event.kind {
        EventKind::EarningsCallTranscript => Some((
            "/earnings_transcript_review/operating_kpi_claims",
            "earnings_call_transcript",
            "company_primary",
        )),
        EventKind::EarningsReleased => Some((
            "/earnings_quality_review/operating_kpi_claims",
            "earnings_release",
            "company_primary",
        )),
        EventKind::SecFiling { form }
            if form.eq_ignore_ascii_case("10-Q")
                || form.eq_ignore_ascii_case("10-K")
                || form.eq_ignore_ascii_case("8-K") =>
        {
            Some(("/operating_kpi_claims", "sec_filing", "regulatory_primary"))
        }
        EventKind::NewsCritical => {
            let metadata = event.payload.get("operating_kpi_source")?;
            let document = metadata.get("document_type")?.as_str()?;
            let tier = metadata.get("source_tier")?.as_str()?;
            match (document, tier) {
                ("investor_presentation", "company_primary") => Some((
                    "/operating_kpi_claims",
                    "investor_presentation",
                    "company_primary",
                )),
                ("official_customer_disclosure", "customer_primary") => Some((
                    "/operating_kpi_claims",
                    "official_customer_disclosure",
                    "customer_primary",
                )),
                ("regulator_disclosure", "regulatory_primary") => Some((
                    "/operating_kpi_claims",
                    "regulator_disclosure",
                    "regulatory_primary",
                )),
                _ => None,
            }
        }
        _ => None,
    }
}

fn valid_input(symbol: &str, input: &OperatingKpiClaimInput) -> bool {
    let kpi_id = normalize_id(&input.kpi_id);
    let numeric_valid = input.numeric_value.is_none_or(f64::is_finite);
    let management_speaker_valid = matches!(input.claim_kind, OperatingKpiClaimKind::ReportedFact)
        || !input.speaker.trim().is_empty();
    operating_kpi_input_is_supported_for_symbol(symbol, input)
        && !input.issuer_metric_name.trim().is_empty()
        && input.issuer_metric_name.chars().count() <= 120
        && !input.issuer_definition.trim().is_empty()
        && input.issuer_definition.chars().count() <= 320
        && verbatim_definition_is_present(input)
        && !input.period.trim().is_empty()
        && !input.value_text.trim().is_empty()
        && !input.measurement_scope.trim().is_empty()
        && !input.evidence_quote.trim().is_empty()
        && !input.source_locator.trim().is_empty()
        && numeric_valid
        && (input.numeric_value.is_none() || allowed_unit(input.unit.trim()))
        && input.numeric_value.is_none_or(|value| {
            numeric_value_appears_in_text(value, &input.value_text)
                || numeric_value_appears_in_text(value, &input.evidence_quote)
        })
        && management_speaker_valid
        && kind_matches_kpi(symbol, &input.claim_kind, &kpi_id)
}

fn kind_matches_kpi(symbol: &str, kind: &OperatingKpiClaimKind, kpi_id: &str) -> bool {
    operating_kpi_catalog_for_symbol(symbol).is_some_and(|entries| {
        entries.iter().any(|entry| {
            entry.kpi_id == kpi_id
                && (entry.milestone_allowed
                    || !matches!(kind, OperatingKpiClaimKind::ContractMilestone))
        })
    })
}

fn verbatim_definition_is_present(input: &OperatingKpiClaimInput) -> bool {
    let definition = normalize_verbatim(&input.issuer_definition);
    !definition.is_empty()
        && (normalize_verbatim(&input.evidence_quote).contains(&definition)
            || normalize_verbatim(&input.value_text).contains(&definition))
}

fn numeric_value_appears_in_text(value: f64, text: &str) -> bool {
    let normalized = text.replace([',', '，', ' '], "");
    let exact = value.to_string();
    let rounded_integer = (value.fract().abs() < f64::EPSILON).then(|| format!("{value:.0}"));
    let integer_word = rounded_integer
        .as_deref()
        .and_then(english_integer_word)
        .is_some_and(|word| {
            text.split(|character: char| !character.is_ascii_alphabetic())
                .any(|token| token.eq_ignore_ascii_case(word))
        });
    normalized.contains(&exact)
        || rounded_integer
            .as_deref()
            .is_some_and(|candidate| normalized.contains(candidate))
        || integer_word
}

fn english_integer_word(value: &str) -> Option<&'static str> {
    match value {
        "0" => Some("zero"),
        "1" => Some("one"),
        "2" => Some("two"),
        "3" => Some("three"),
        "4" => Some("four"),
        "5" => Some("five"),
        "6" => Some("six"),
        "7" => Some("seven"),
        "8" => Some("eight"),
        "9" => Some("nine"),
        "10" => Some("ten"),
        "11" => Some("eleven"),
        "12" => Some("twelve"),
        "13" => Some("thirteen"),
        "14" => Some("fourteen"),
        "15" => Some("fifteen"),
        "16" => Some("sixteen"),
        "17" => Some("seventeen"),
        "18" => Some("eighteen"),
        "19" => Some("nineteen"),
        "20" => Some("twenty"),
        _ => None,
    }
}

fn allowed_unit(unit: &str) -> bool {
    matches!(
        unit,
        "%" | "percentage_points"
            | "basis_points"
            | "USD"
            | "USD_millions"
            | "USD_billions"
            | "units"
            | "customers"
            | "agreements"
            | "days"
            | "weeks"
            | "ratio"
            | "kW"
            | "MW"
            | "GW"
            | "GB"
            | "TB"
            | "PB"
            | "EB"
            | "bits"
            | "tokens"
            | "calls"
            | "modules"
            | "ports"
            | "wafers"
            | "workflows"
            | "milestone"
    )
}

fn normalize_symbol(value: &str) -> String {
    value.trim().trim_start_matches('$').to_ascii_uppercase()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn normalize_id(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_verbatim(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn normalize_source_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn definition_key(value: &str) -> String {
    normalize_verbatim(value)
        .chars()
        .filter(|character| character.is_alphanumeric())
        .take(240)
        .collect()
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut output = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        output.push('…');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Severity;

    #[test]
    fn numeric_validation_accepts_bounded_english_integer_words_without_substring_matches() {
        assert!(numeric_value_appears_in_text(
            5.0,
            "we signed five additional agreements"
        ));
        assert!(!numeric_value_appears_in_text(
            5.0,
            "we signed fifty additional agreements"
        ));
        assert!(!numeric_value_appears_in_text(
            21.0,
            "we signed twenty-one additional agreements"
        ));
    }

    fn event(symbol: &str, payload: serde_json::Value) -> MarketEvent {
        MarketEvent {
            id: format!("{symbol}-call-2026q4"),
            kind: EventKind::EarningsCallTranscript,
            severity: Severity::Medium,
            symbols: vec![symbol.to_string()],
            occurred_at: DateTime::parse_from_rfc3339("2026-08-06T21:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            title: "call".into(),
            summary: "summary".into(),
            url: Some("https://investors.example.com/call".into()),
            source: format!("{symbol} investor relations"),
            payload,
        }
    }

    fn valid_claim() -> serde_json::Value {
        serde_json::json!({
            "claim_kind": "reported_fact",
            "kpi_id": "nand_asp_change",
            "issuer_metric_name": "NAND ASP",
            "issuer_definition": "NAND average selling price",
            "period": "FY2026 Q4",
            "numeric_value": 15.0,
            "unit": "%",
            "value_text": "NAND average selling price increased 15% sequentially",
            "measurement_scope": "company NAND realized price; sequential quarter",
            "comparison_basis": "sequential_quarter",
            "speaker": "CFO",
            "evidence_quote": "NAND average selling price increased 15% sequentially",
            "source_locator": "prepared remarks / CFO",
            "definition_changed": false,
            "disposition": "active"
        })
    }

    #[test]
    fn admits_catalog_bounded_claims_with_verbatim_definition() {
        let payload = serde_json::json!({
            "earnings_transcript_operating_kpi_source_verified": true,
            "earnings_transcript_review": {"operating_kpi_claims": [valid_claim()]}
        });
        for symbol in ["SNDK", "MU", "SKHY", "STX", "WDC"] {
            let claims = operating_kpi_claims_from_event(&event(symbol, payload.clone()));
            assert_eq!(claims.len(), 1);
            assert_eq!(claims[0].kpi_id, "nand_asp_change");
            assert_eq!(claims[0].source_tier, "company_primary");
            assert!(!claims[0].definition_key.is_empty());
        }
        assert!(operating_kpi_claims_from_event(&event("XYZ", payload)).is_empty());
    }

    #[test]
    fn v2_backfill_claims_fail_closed_without_a_valid_source_artifact() {
        let missing = serde_json::json!({
            "operating_kpi_backfill_schema_version": OPERATING_KPI_BACKFILL_EVENT_SCHEMA_VERSION,
            "operating_kpi_source_time_precision": "date_only_conservative_end_of_day",
            "earnings_transcript_operating_kpi_source_verified": true,
            "earnings_transcript_review": {"operating_kpi_claims": [valid_claim()]}
        });
        assert!(operating_kpi_claims_from_event(&event("SNDK", missing)).is_empty());

        let malformed = serde_json::json!({
            "operating_kpi_backfill_schema_version": OPERATING_KPI_BACKFILL_EVENT_SCHEMA_VERSION,
            "operating_kpi_source_time_precision": "date_only_conservative_end_of_day",
            "operating_kpi_source_artifact": {
                "schema_version": OPERATING_KPI_SOURCE_ARTIFACT_SCHEMA_VERSION,
                "source_sha256": "not-a-digest",
                "extracted_text_sha256": "b".repeat(64),
                "byte_length": 1024,
                "format": "pdf",
                "object_path": "objects/not-a-digest.pdf"
            },
            "earnings_transcript_operating_kpi_source_verified": true,
            "earnings_transcript_review": {"operating_kpi_claims": [valid_claim()]}
        });
        assert!(operating_kpi_claims_from_event(&event("SNDK", malformed)).is_empty());
    }

    #[test]
    fn six_model_catalog_is_symbol_scoped_and_rejects_cross_industry_ids() {
        assert_eq!(
            STORAGE_SYMBOLS.len()
                + COMPUTE_SYMBOLS.len()
                + OPTICAL_SYMBOLS.len()
                + POWER_SYMBOLS.len()
                + PLATFORM_SYMBOLS.len()
                + APPLICATION_SYMBOLS.len(),
            49
        );
        assert_eq!(
            STORAGE_KPIS.len()
                + COMPUTE_KPIS.len()
                + OPTICAL_KPIS.len()
                + POWER_KPIS.len()
                + PLATFORM_KPIS.len()
                + APPLICATION_KPIS.len(),
            37
        );
        let cases = [
            ("SNDK", "ai-storage-demand-supply", 9),
            ("TSM", "ai-compute-effective-capacity", 5),
            ("LITE", "ai-optical-interconnect-bandwidth", 5),
            ("GEV", "ai-data-center-power-delivery", 6),
            ("MSFT", "ai-platform-token-economics", 6),
            ("TEM", "ai-application-workflow-value", 6),
        ];
        for (symbol, model_id, expected_count) in cases {
            assert_eq!(operating_kpi_model_id_for_symbol(symbol), Some(model_id));
            assert_eq!(
                operating_kpi_catalog_for_symbol(symbol).unwrap().len(),
                expected_count
            );
        }
        assert!(operating_kpi_catalog_for_symbol("AAPL").is_none());

        let storage_claim =
            serde_json::from_value::<OperatingKpiClaimInput>(valid_claim()).unwrap();
        assert!(operating_kpi_input_is_supported_for_symbol(
            "SNDK",
            &storage_claim
        ));
        assert!(!operating_kpi_input_is_supported_for_symbol(
            "MSFT",
            &storage_claim
        ));
    }

    #[test]
    fn platform_claim_is_admitted_but_only_for_platform_symbols() {
        let claim = serde_json::json!({
            "claim_kind": "reported_fact",
            "kpi_id": "token_or_call_volume",
            "issuer_metric_name": "paid tokens",
            "issuer_definition": "paid tokens processed",
            "period": "FY2026 Q4",
            "numeric_value": 10.0,
            "unit": "%",
            "value_text": "paid tokens processed increased 10% year over year",
            "measurement_scope": "paid production API traffic; excludes free traffic",
            "comparison_basis": "year_over_year",
            "speaker": "CFO",
            "evidence_quote": "paid tokens processed increased 10% year over year",
            "source_locator": "prepared remarks / CFO",
            "definition_changed": false,
            "disposition": "active"
        });
        let payload = serde_json::json!({
            "earnings_transcript_operating_kpi_source_verified": true,
            "earnings_transcript_review": {"operating_kpi_claims": [claim]}
        });
        let admitted = operating_kpi_claims_from_event(&event("MSFT", payload.clone()));
        assert_eq!(admitted.len(), 1);
        assert_eq!(admitted[0].kpi_id, "token_or_call_volume");
        assert!(operating_kpi_claims_from_event(&event("SNDK", payload)).is_empty());
    }

    #[test]
    fn dynamic_prompt_never_exposes_another_models_kpis() {
        let platform = operating_kpi_prompt_for_symbol("msft");
        assert!(platform.contains("token_or_call_volume"));
        assert!(!platform.contains("nand_asp_change"));
        assert!(platform.contains(OPERATING_KPI_CATALOG_VERSION));

        let unsupported = operating_kpi_prompt_for_symbol("AAPL");
        assert!(unsupported.contains("必须输出空数组"));
    }

    #[test]
    fn rejects_unknown_kpi_paraphrased_definition_and_invented_number() {
        let mut unknown = valid_claim();
        unknown["kpi_id"] = serde_json::Value::String("industry_spot_price".into());
        let mut paraphrased = valid_claim();
        paraphrased["issuer_definition"] =
            serde_json::Value::String("company achieved selling price".into());
        let mut invented = valid_claim();
        invented["numeric_value"] = serde_json::Value::from(25.0);
        let payload = serde_json::json!({
            "earnings_transcript_operating_kpi_source_verified": true,
            "earnings_transcript_review": {
                "operating_kpi_claims": [unknown, paraphrased, invented]
            }
        });
        assert!(operating_kpi_claims_from_event(&event("SNDK", payload)).is_empty());
    }

    #[test]
    fn rejects_untrusted_manual_source_metadata() {
        let mut candidate = event(
            "SNDK",
            serde_json::json!({
                "operating_kpi_source": {
                    "document_type": "blog_post",
                    "source_tier": "uncertain"
                },
                "operating_kpi_claims": [valid_claim()]
            }),
        );
        candidate.kind = EventKind::NewsCritical;
        assert!(operating_kpi_claims_from_event(&candidate).is_empty());
    }

    #[test]
    fn source_verification_rejects_a_model_invented_quote_and_unverified_payload() {
        let input = serde_json::from_value::<OperatingKpiClaimInput>(valid_claim()).unwrap();
        let source =
            "CFO: NAND ASP means NAND average selling price. It increased 15% sequentially.";
        assert!(!operating_kpi_input_is_verbatim_in_source(&input, source));
        let exact_source = "CFO: NAND ASP. NAND average selling price increased 15% sequentially";
        assert!(operating_kpi_input_is_verbatim_in_source(
            &input,
            exact_source
        ));

        let payload = serde_json::json!({
            "earnings_transcript_review": {"operating_kpi_claims": [valid_claim()]}
        });
        assert!(operating_kpi_claims_from_event(&event("SNDK", payload)).is_empty());
    }
}
