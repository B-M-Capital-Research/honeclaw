//! Hone Memory — 会话、持仓、定时任务、草稿存储
//!
//! 使用 JSON 文件存储。

pub mod billing;
pub mod company_facts;
pub mod company_profile;
pub mod cron_job;
pub mod llm_audit;
pub mod password;
pub mod portfolio;
pub mod quota;
pub mod session;
pub mod survey;
mod test_postgres;
pub mod web_auth;

pub use billing::{
    BILLING_ACCESS_ACTIVE, BILLING_ACCESS_GRACE, BILLING_ACCESS_INACTIVE, BILLING_ACCESS_PENDING,
    BILLING_ENTITLEMENT_DOMESTIC_INVITE, BILLING_ENTITLEMENT_FIXED_TERM_PURCHASE,
    BILLING_ENTITLEMENT_RECURRING_SUBSCRIPTION, BILLING_EVENT_FAILED, BILLING_EVENT_PROCESSED,
    BILLING_EVENT_PROCESSING, BILLING_EVENT_RECEIVED, BILLING_PROVIDER_DOMESTIC_INVITE,
    BILLING_PROVIDER_STRIPE, BillingEntitlement, BillingEntitlementUpsertOutcome, BillingStorage,
    BillingWebhookEvent, BillingWebhookRecordOutcome,
};
pub use company_facts::{
    BalanceSheetFacts, COMPANY_FACTS_SCHEMA_VERSION, COVER_STALE_AFTER_DAYS, CashFlowFacts,
    CompanyFacts, CompanyFactsStorage, CompanyIdentity, EarningsCadence, FACTS_STALE_AFTER_HOURS,
    FactProvenance, IncomeFacts, ShareCounts, configure_cloud_company_facts_storage,
};
pub use company_profile::{
    AppendEventInput, AppendResearchEventInput, CompanyProfileConflictDecision,
    CompanyProfileDocument, CompanyProfileEventDocument, CompanyProfileImportApplyInput,
    CompanyProfileImportApplyResult, CompanyProfileImportConflict,
    CompanyProfileImportConflictDetail, CompanyProfileImportDiffLine,
    CompanyProfileImportDiffLineKind, CompanyProfileImportEventDiff, CompanyProfileImportMode,
    CompanyProfileImportPreview, CompanyProfileImportProfileSummary,
    CompanyProfileImportResolutionInput, CompanyProfileImportResolutionResult,
    CompanyProfileImportResolutionStrategy, CompanyProfileImportSectionChangeType,
    CompanyProfileImportSectionDiff, CompanyProfileStorage, CompanyProfileTransferManifest,
    CompanyProfileTransferManifestProfile, CompanyResearchLedger, CoverageTier, CreateProfileInput,
    IndustryTemplate, ProfileEventMetadata, ProfileMetadata, ProfileSpaceSummary, ProfileSummary,
    RawProfileDocument, RawProfileEventDocument, RawProfileSummary, ResearchItemKind,
    ResearchItemStatus, ResearchLedgerItem, ResearchLedgerUpdate, TrackingConfig,
    configure_cloud_company_profile_storage, research_item_id,
};
pub use cron_job::{ChannelTargetRecord, CronJobStorage};
pub use llm_audit::{AuditQueryFilter, AuditRecordSummary, LlmAuditStorage};
pub use portfolio::{PortfolioStorage, configure_cloud_portfolio_storage};
pub use quota::{
    ConversationQuotaReservation, ConversationQuotaReserveResult, ConversationQuotaSnapshot,
    ConversationQuotaStorage,
};
pub use session::InterruptedSessionInfo;
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
pub use survey::{
    ACTIVE_SURVEY_ID, SURVEY_CLIENT_WINDOW_HOURS, SURVEY_CLIENT_WINDOW_LIMIT, SurveyResponse,
    SurveyStorage, configure_cloud_survey_storage,
};
pub use web_auth::{
    EmailVerificationResult, SESSION_TTL_DAYS_LONG, SESSION_TTL_DAYS_SHORT,
    WEB_ADMIN_DAILY_INVITE_LIMIT, WEB_IDENTITY_DOMESTIC_INVITE, WEB_IDENTITY_INTERNATIONAL_EMAIL,
    WebAdminInviteCreateOutcome, WebAdminInviteDisableOutcome, WebAdminInviteSummary,
    WebAuthStorage, WebInviteSession, WebInviteUser, WebSessionAuthResult, WebUserExternalProfile,
};
