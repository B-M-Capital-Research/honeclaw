use std::collections::BTreeMap;
use std::path::PathBuf;

use hone_core::ActorIdentity;
use hone_core::cloud_runtime::CloudPgRuntime;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum IndustryTemplate {
    General,
    Saas,
    SemiconductorHardware,
    Consumer,
    IndustrialDefense,
    Financials,
}

impl Default for IndustryTemplate {
    fn default() -> Self {
        Self::General
    }
}

impl IndustryTemplate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Saas => "saas",
            Self::SemiconductorHardware => "semiconductor_hardware",
            Self::Consumer => "consumer",
            Self::IndustrialDefense => "industrial_defense",
            Self::Financials => "financials",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum CoverageTier {
    #[serde(alias = "A", alias = "a_core")]
    A,
    #[serde(alias = "B", alias = "b_watch")]
    B,
    #[serde(alias = "C", alias = "c_discovery")]
    C,
}

impl Default for CoverageTier {
    fn default() -> Self {
        Self::C
    }
}

impl CoverageTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackingConfig {
    #[serde(default)]
    pub enabled: bool,
    /// A=核心覆盖，B=观察覆盖，C=发现池。缺省 C，避免旧画像在没有用户授权时
    /// 被自动升级成高频深度跟踪。
    #[serde(default)]
    pub coverage_tier: CoverageTier,
    /// 用户确认的投资期限，例如 long_term / 3-5y。自由文本是为了兼容不同
    /// 投资框架；Hone 只读取，不据此静默改写主线。
    #[serde(default = "default_investment_horizon")]
    pub investment_horizon: String,
    #[serde(default = "default_tracking_cadence")]
    pub cadence: String,
    #[serde(default)]
    pub focus_metrics: Vec<String>,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            coverage_tier: CoverageTier::C,
            investment_horizon: default_investment_horizon(),
            cadence: default_tracking_cadence(),
            focus_metrics: Vec::new(),
        }
    }
}

fn default_investment_horizon() -> String {
    "long_term".to_string()
}

fn default_tracking_cadence() -> String {
    "weekly".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileMetadata {
    pub company_name: String,
    #[serde(default)]
    pub stock_code: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sector: String,
    #[serde(default)]
    pub industry_template: IndustryTemplate,
    #[serde(default = "default_profile_status")]
    pub status: String,
    #[serde(default)]
    pub tracking: TrackingConfig,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reviewed_at: Option<String>,
}

pub(crate) fn default_profile_status() -> String {
    "active".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileEventMetadata {
    pub event_type: String,
    pub occurred_at: String,
    pub captured_at: String,
    #[serde(default = "default_mainline_impact", alias = "thesis_impact")]
    pub mainline_impact: String,
    #[serde(default)]
    pub changed_sections: Vec<String>,
    #[serde(default)]
    pub refs: Vec<String>,
    /// 同一轮财报的 release / transcript / 正式季报共享这个 key。各材料仍以
    /// append-only 事件保存，但产品层可以把它们聚合为同一张持续更新的研究卡。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub research_object_key: Option<String>,
    /// 本事件对长期研究账本造成的结构化变化。旧事件缺省为空，保持兼容。
    #[serde(default)]
    pub research_updates: Vec<ResearchLedgerUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchItemKind {
    OpenQuestion,
    ManagementCommitment,
}

impl ResearchItemKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenQuestion => "open_question",
            Self::ManagementCommitment => "management_commitment",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchItemStatus {
    Open,
    PartiallyAnswered,
    Answered,
    Confirmed,
    Contradicted,
    Expired,
}

impl ResearchItemStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Open | Self::PartiallyAnswered)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::PartiallyAnswered => "partially_answered",
            Self::Answered => "answered",
            Self::Confirmed => "confirmed",
            Self::Contradicted => "contradicted",
            Self::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchLedgerUpdate {
    pub item_id: String,
    pub kind: ResearchItemKind,
    /// 新项目必须提供 statement；后续更新允许为空，沿用首次记录的原文，防止
    /// 模型在不同季度悄悄改写问题或承诺本身。
    #[serde(default)]
    pub statement: String,
    pub status: ResearchItemStatus,
    #[serde(default)]
    pub assessment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchLedgerItem {
    pub item_id: String,
    pub kind: ResearchItemKind,
    pub statement: String,
    pub status: ResearchItemStatus,
    pub first_seen_at: String,
    pub last_reviewed_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due_at: Option<String>,
    #[serde(default)]
    pub latest_assessment: String,
    #[serde(default)]
    pub evidence: Vec<String>,
    pub latest_event_id: String,
    pub update_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyResearchLedger {
    pub profile_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of: Option<String>,
    pub items: Vec<ResearchLedgerItem>,
}

impl CompanyResearchLedger {
    pub fn active_questions(&self) -> impl Iterator<Item = &ResearchLedgerItem> {
        self.items
            .iter()
            .filter(|item| item.kind == ResearchItemKind::OpenQuestion && item.status.is_active())
    }

    pub fn active_commitments(&self) -> impl Iterator<Item = &ResearchLedgerItem> {
        self.items.iter().filter(|item| {
            item.kind == ResearchItemKind::ManagementCommitment && item.status.is_active()
        })
    }
}

/// 用 kind + 规范化原文生成稳定短 ID。ID 只负责同一研究空间里的幂等，原文
/// 始终保留在 ledger 中供审计，不能从哈希反推或替代证据。
pub fn research_item_id(kind: &ResearchItemKind, statement: &str) -> String {
    let normalized = statement
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    let digest = Sha256::digest(format!("{}:{normalized}", kind.as_str()).as_bytes());
    let suffix = digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{}-{suffix}", kind.as_str())
}

pub(crate) fn default_mainline_impact() -> String {
    "unknown".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileEventDocument {
    pub id: String,
    pub filename: String,
    pub title: String,
    pub metadata: ProfileEventMetadata,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileDocument {
    pub profile_id: String,
    pub metadata: ProfileMetadata,
    pub markdown: String,
    pub events: Vec<CompanyProfileEventDocument>,
}

impl CompanyProfileDocument {
    /// 读取一个画像二级标题的正文。这里只提供只读视图；画像主线仍只能通过
    /// 既有显式 rewrite / 用户确认路径修改。
    pub fn section(&self, title: &str) -> Option<String> {
        let wanted = title.trim();
        if wanted.is_empty() {
            return None;
        }
        let mut matched = false;
        let mut lines = Vec::new();
        for line in self.markdown.lines() {
            if let Some(section_title) = line.strip_prefix("## ") {
                if matched {
                    break;
                }
                matched = section_title.trim() == wanted;
                continue;
            }
            if matched {
                lines.push(line);
            }
        }
        let value = lines.join("\n").trim().to_string();
        (!value.is_empty()).then_some(value)
    }

    /// 从 append-only 事件折叠当前研究账本。首次记录的 kind / statement 是身份
    /// 真相源；后续事件只能更新状态和评估，不能静默改写原问题或承诺。
    pub fn research_ledger(&self) -> CompanyResearchLedger {
        let mut events = self.events.iter().collect::<Vec<_>>();
        events.sort_by(|left, right| {
            left.metadata
                .occurred_at
                .cmp(&right.metadata.occurred_at)
                .then_with(|| left.metadata.captured_at.cmp(&right.metadata.captured_at))
                .then_with(|| left.id.cmp(&right.id))
        });

        let mut items = BTreeMap::<String, ResearchLedgerItem>::new();
        let mut as_of = None;
        for event in events {
            if as_of.as_deref() < Some(event.metadata.captured_at.as_str()) {
                as_of = Some(event.metadata.captured_at.clone());
            }
            for update in &event.metadata.research_updates {
                let item_id = update.item_id.trim();
                if item_id.is_empty() {
                    continue;
                }
                if let Some(item) = items.get_mut(item_id) {
                    // 身份字段不接受模型的后续改写。即使输入异常，旧记录仍可读。
                    if item.kind != update.kind {
                        continue;
                    }
                    item.status = update.status.clone();
                    item.last_reviewed_at = event.metadata.occurred_at.clone();
                    item.latest_event_id = event.id.clone();
                    item.update_count += 1;
                    if !update.assessment.trim().is_empty() {
                        item.latest_assessment = update.assessment.trim().to_string();
                    }
                    if update.due_at.is_some() {
                        item.due_at = update.due_at.clone();
                    }
                    for evidence in &update.evidence {
                        let evidence = evidence.trim();
                        if !evidence.is_empty()
                            && !item.evidence.iter().any(|value| value == evidence)
                        {
                            item.evidence.push(evidence.to_string());
                        }
                    }
                    continue;
                }

                let statement = update.statement.trim();
                if statement.is_empty() {
                    continue;
                }
                items.insert(
                    item_id.to_string(),
                    ResearchLedgerItem {
                        item_id: item_id.to_string(),
                        kind: update.kind.clone(),
                        statement: statement.to_string(),
                        status: update.status.clone(),
                        first_seen_at: event.metadata.occurred_at.clone(),
                        last_reviewed_at: event.metadata.occurred_at.clone(),
                        due_at: update.due_at.clone(),
                        latest_assessment: update.assessment.trim().to_string(),
                        evidence: update
                            .evidence
                            .iter()
                            .map(|value| value.trim())
                            .filter(|value| !value.is_empty())
                            .map(str::to_string)
                            .collect(),
                        latest_event_id: event.id.clone(),
                        update_count: 1,
                    },
                );
            }
        }

        CompanyResearchLedger {
            profile_id: self.profile_id.clone(),
            as_of,
            items: items.into_values().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSummary {
    pub profile_id: String,
    pub company_name: String,
    pub stock_code: String,
    pub sector: String,
    pub industry_template: IndustryTemplate,
    pub status: String,
    pub tracking_enabled: bool,
    pub coverage_tier: CoverageTier,
    pub investment_horizon: String,
    pub tracking_cadence: String,
    pub updated_at: String,
    pub last_reviewed_at: Option<String>,
    pub event_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSpaceSummary {
    pub channel: String,
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_scope: Option<String>,
    pub profile_count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawProfileSummary {
    pub profile_id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub event_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawProfileEventDocument {
    pub id: String,
    pub filename: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub markdown: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawProfileDocument {
    pub profile_id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub markdown: String,
    pub events: Vec<RawProfileEventDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileTransferManifestProfile {
    pub profile_id: String,
    pub company_name: String,
    #[serde(default)]
    pub stock_code: String,
    pub event_count: usize,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileTransferManifest {
    pub version: String,
    pub exported_at: String,
    pub profile_count: usize,
    pub event_count: usize,
    pub profiles: Vec<CompanyProfileTransferManifestProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportProfileSummary {
    pub profile_id: String,
    pub company_name: String,
    #[serde(default)]
    pub stock_code: String,
    pub updated_at: String,
    pub event_count: usize,
    #[serde(
        default,
        skip_serializing_if = "String::is_empty",
        alias = "thesis_excerpt"
    )]
    pub mainline_excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportConflict {
    pub imported: CompanyProfileImportProfileSummary,
    pub existing: CompanyProfileImportProfileSummary,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompanyProfileImportMode {
    KeepExisting,
    ReplaceAll,
    Interactive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompanyProfileConflictDecision {
    Skip,
    Replace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportPreview {
    pub manifest: CompanyProfileTransferManifest,
    pub profiles: Vec<CompanyProfileImportProfileSummary>,
    pub conflicts: Vec<CompanyProfileImportConflict>,
    pub importable_count: usize,
    pub conflict_count: usize,
    pub suggested_mode: CompanyProfileImportMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompanyProfileImportApplyInput {
    pub mode: Option<CompanyProfileImportMode>,
    #[serde(default)]
    pub decisions: BTreeMap<String, CompanyProfileConflictDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportApplyResult {
    pub imported_profile_ids: Vec<String>,
    pub replaced_profile_ids: Vec<String>,
    pub skipped_profile_ids: Vec<String>,
    pub changed_profile_ids: Vec<String>,
    pub imported_count: usize,
    pub replaced_count: usize,
    pub skipped_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompanyProfileImportResolutionStrategy {
    Skip,
    Replace,
    MergeSections,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompanyProfileImportDiffLineKind {
    Context,
    Added,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportDiffLine {
    pub kind: CompanyProfileImportDiffLineKind,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompanyProfileImportSectionChangeType {
    Modified,
    ImportedOnly,
    ExistingOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportSectionDiff {
    pub section_title: String,
    pub change_type: CompanyProfileImportSectionChangeType,
    pub line_diff: Vec<CompanyProfileImportDiffLine>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub imported_excerpt: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub existing_excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CompanyProfileImportEventDiff {
    #[serde(default)]
    pub imported_only_event_ids: Vec<String>,
    #[serde(default)]
    pub existing_only_event_ids: Vec<String>,
    #[serde(default)]
    pub shared_event_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportConflictDetail {
    pub conflict: CompanyProfileImportConflict,
    #[serde(default)]
    pub available_section_titles: Vec<String>,
    #[serde(default)]
    pub section_diffs: Vec<CompanyProfileImportSectionDiff>,
    #[serde(default)]
    pub event_diff: CompanyProfileImportEventDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportResolutionInput {
    pub imported_profile_id: String,
    pub strategy: CompanyProfileImportResolutionStrategy,
    #[serde(default)]
    pub section_titles: Vec<String>,
    #[serde(default = "default_import_missing_events")]
    pub import_missing_events: bool,
}

fn default_import_missing_events() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyProfileImportResolutionResult {
    pub imported_profile_id: String,
    pub target_profile_id: String,
    pub strategy: CompanyProfileImportResolutionStrategy,
    pub created_new_profile: bool,
    pub replaced_existing_profile: bool,
    pub merged_existing_profile: bool,
    pub skipped: bool,
    #[serde(default)]
    pub changed_sections: Vec<String>,
    #[serde(default)]
    pub imported_event_ids: Vec<String>,
    #[serde(default)]
    pub skipped_event_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreateProfileInput {
    pub company_name: String,
    pub stock_code: Option<String>,
    pub sector: Option<String>,
    pub aliases: Vec<String>,
    pub industry_template: IndustryTemplate,
    pub tracking: Option<TrackingConfig>,
    pub initial_sections: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AppendEventInput {
    pub title: String,
    pub event_type: String,
    pub occurred_at: String,
    pub mainline_impact: String,
    pub changed_sections: Vec<String>,
    pub refs: Vec<String>,
    pub what_happened: String,
    pub why_it_matters: String,
    pub mainline_effect: String,
    pub evidence: String,
    pub research_log: String,
    pub follow_up: String,
}

#[derive(Debug, Clone)]
pub struct AppendResearchEventInput {
    pub event: AppendEventInput,
    pub research_object_key: Option<String>,
    pub research_updates: Vec<ResearchLedgerUpdate>,
}

pub struct CompanyProfileStorage {
    pub(crate) root_dir: PathBuf,
    pub(crate) actor: Option<ActorIdentity>,
    pub(crate) cloud: Option<CloudPgRuntime>,
}
