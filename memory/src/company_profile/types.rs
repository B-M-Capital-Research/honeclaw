use std::collections::BTreeMap;
use std::path::PathBuf;

use hone_core::ActorIdentity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrackingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_tracking_cadence")]
    pub cadence: String,
    #[serde(default)]
    pub focus_metrics: Vec<String>,
}

impl Default for TrackingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cadence: default_tracking_cadence(),
            focus_metrics: Vec::new(),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSummary {
    pub profile_id: String,
    pub company_name: String,
    pub stock_code: String,
    pub sector: String,
    pub industry_template: IndustryTemplate,
    pub status: String,
    pub tracking_enabled: bool,
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

pub struct CompanyProfileStorage {
    pub(crate) root_dir: PathBuf,
    pub(crate) actor: Option<ActorIdentity>,
}
