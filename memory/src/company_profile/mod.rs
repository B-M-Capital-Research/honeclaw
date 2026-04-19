//! 公司画像存储 - Markdown 主文件 + 事件目录
//!
//! 目录结构：
//! ```text
//! <actor_sandbox_root>/
//!   company_profiles/
//!     <profile_id>/
//!       profile.md
//!       events/
//!         2026-04-12-earnings-q1-update.md
//! ```

mod markdown;
mod storage;
mod transfer;
mod types;

pub use types::{
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

#[cfg(test)]
mod tests;
