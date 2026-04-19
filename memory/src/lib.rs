//! Hone Memory — 会话、持仓、定时任务、草稿存储
//!
//! 使用 JSON 文件存储。

pub mod company_profile;
pub mod cron_job;
pub mod llm_audit;
pub mod portfolio;
pub mod quota;
pub mod session;
pub mod session_sqlite;

pub use company_profile::{
    AppendEventInput, CompanyProfileConflictDecision, CompanyProfileDocument,
    CompanyProfileEventDocument, CompanyProfileImportApplyInput, CompanyProfileImportApplyResult,
    CompanyProfileImportConflict, CompanyProfileImportConflictDetail, CompanyProfileImportDiffLine,
    CompanyProfileImportDiffLineKind, CompanyProfileImportEventDiff, CompanyProfileImportMode,
    CompanyProfileImportPreview, CompanyProfileImportProfileSummary,
    CompanyProfileImportResolutionInput, CompanyProfileImportResolutionResult,
    CompanyProfileImportResolutionStrategy, CompanyProfileImportSectionChangeType,
    CompanyProfileImportSectionDiff, CompanyProfileStorage, CompanyProfileTransferManifest,
    CompanyProfileTransferManifestProfile, CreateProfileInput, IndustryTemplate,
    ProfileEventMetadata, ProfileMetadata, ProfileSpaceSummary, ProfileSummary, RawProfileDocument,
    RawProfileEventDocument, RawProfileSummary, TrackingConfig,
};
pub use cron_job::CronJobStorage;
pub use llm_audit::{AuditQueryFilter, AuditRecordSummary, LlmAuditStorage};
pub use portfolio::PortfolioStorage;
pub use quota::{
    ConversationQuotaReservation, ConversationQuotaReserveResult, ConversationQuotaSnapshot,
    ConversationQuotaStorage,
};
pub use session::{
    ASSISTANT_TOOL_CALLS_METADATA_KEY, COMPACT_BOUNDARY_METADATA_KEY,
    COMPACT_SKILL_SNAPSHOT_METADATA_KEY, COMPACT_SUMMARY_METADATA_KEY, INVOKED_SKILLS_METADATA_KEY,
    InvokedSkillRecord, SLASH_SKILL_METADATA_KEY, SessionStorage,
    assistant_tool_calls_from_metadata, build_assistant_message_metadata,
    build_compact_boundary_metadata, build_compact_skill_snapshot_metadata,
    build_compact_summary_metadata, build_tool_message_metadata, build_tool_message_metadata_parts,
    find_last_compact_boundary_index, has_compact_skill_snapshot, invoked_skills_from_metadata,
    latest_compact_summary, message_is_compact_boundary, message_is_compact_skill_snapshot,
    message_is_compact_summary, message_is_slash_skill, restore_tool_message,
    select_context_messages, select_messages_after_compact_boundary,
    session_message_from_normalized, session_message_from_text, session_message_in_context,
    session_message_text, session_message_to_agent_messages, session_message_to_normalized,
};
