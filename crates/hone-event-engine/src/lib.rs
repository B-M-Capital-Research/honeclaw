//! hone-event-engine — 主动事件引擎
//!
//! 负责：
//! 1. Pollers（纯 Rust、无 LLM）从数据源拉取市场事件
//! 2. 去重（EventStore）后发布到订阅分发层
//! 3. 按持仓/订阅分发，按 severity 分流（高优实时、低/中优先进每日摘要）
//! 4. 通过 `OutboundSink` / `MultiChannelSink` 派发到 Log、Feishu、Discord 等渠道

pub mod daily_report;
pub mod digest;
pub mod earnings_claim;
pub mod earnings_continuity;
pub mod earnings_transcript;
pub mod event;
pub mod fmp;
pub mod global_digest;
pub mod news_classifier;
pub mod operating_kpi_claim;
pub mod polisher;
pub mod pollers;
pub mod prefs;
pub mod renderer;
pub mod router;
pub mod sec_company_facts;
pub mod sinks;
pub mod source;
pub mod store;
pub mod subscription;
pub mod unified_digest;

// ── 内部子 module:engine 主体 + spawn 模板 + 共享 pipeline ──
// 保持 crate 私有:EventEngine 通过下面的 pub use 暴露,其它三个不外露。
mod earnings_document;
mod engine;
mod pipeline;
mod spawner;

#[cfg(test)]
mod tests;

pub use daily_report::DailyReport;
pub use digest::DigestBuffer;
pub use earnings_claim::{
    CLAIM_POLICY_STATUS, EarningsClaimDisposition, EarningsClaimInput, EarningsClaimKind,
    EarningsSourceClaim, LEGACY_UNSPECIFIED_METRIC_BASIS, source_claims_from_event,
};
pub use earnings_continuity::{
    EarningsContinuityOutcome, EarningsContinuityReconciler, EarningsContinuityReview,
    EarningsResearchMaterialOutcome, LlmEarningsContinuityReconciler,
};
pub use earnings_transcript::{
    EarningsTranscriptReview, EarningsTranscriptReviewer, LlmEarningsTranscriptReviewer,
    apply_earnings_transcript_review, apply_earnings_transcript_review_with_source,
};
pub use engine::EventEngine;
pub use event::{EventKind, MarketEvent, Severity};
pub use fmp::FmpClient;
pub use hone_core::config::{EventEngineConfig, FmpConfig};
pub use news_classifier::{
    DEFAULT_IMPORTANCE_PROMPT, Importance, LlmNewsClassifier, NewsClassifier, NoopClassifier,
};
pub use operating_kpi_claim::{
    OPERATING_KPI_BACKFILL_EVENT_SCHEMA_VERSION, OPERATING_KPI_CATALOG_VERSION,
    OPERATING_KPI_CLAIM_SCHEMA_VERSION, OPERATING_KPI_POLICY_STATUS,
    OPERATING_KPI_SOURCE_ARTIFACT_SCHEMA_VERSION, OperatingKpiCatalogEntry, OperatingKpiClaimInput,
    OperatingKpiClaimKind, OperatingKpiComparisonBasis, OperatingKpiSourceArtifact,
    OperatingKpiSourceClaim, operating_kpi_catalog_for_model, operating_kpi_catalog_for_symbol,
    operating_kpi_claims_from_event, operating_kpi_input_is_supported_for_symbol,
    operating_kpi_input_is_verbatim_in_source, operating_kpi_model_id_for_symbol,
    operating_kpi_prompt_for_symbol, operating_kpi_source_artifact_from_event,
    operating_kpi_source_artifact_is_valid,
};
pub use polisher::{BodyPolisher, LlmPolisher, NoopPolisher, parse_polish_levels};
pub use pollers::{
    AnalystGradePoller, CorpActionCalendarPoller, EarningsPoller, EarningsSurprisePoller,
    MacroPoller, NewsPoller, PricePoller, SecFilingsPoller, TelegramChannelPoller,
};
pub use prefs::{
    AllowAllPrefs, FilePrefsStorage, NotificationPrefs, PrefsProvider, SharedPrefs, kind_tag,
};
pub use renderer::RenderFormat;
pub use router::{LogSink, NotificationRouter, OutboundSink};
pub use sec_company_facts::{SecCompanyFactsBackfillReport, SecCompanyFactsBackfiller};
pub use sinks::{DiscordSink, FeishuSink, IMessageSink, MultiChannelSink, TelegramSink};
pub use source::{EventSource, FnSource, SourceSchedule};
pub use store::{
    DeliveredPushContextClaim, DeliveredPushContextRecord, DeliveryLogFilter, DeliveryLogRecord,
    EventStore,
};
pub use subscription::{
    CompanyProfileSubscription, GlobalSubscription, PortfolioSubscription, SharedRegistry,
    Subscription, SubscriptionRegistry, registry_from_portfolios,
    registry_from_portfolios_and_profiles,
};
pub use unified_digest::UnifiedDigestScheduler;
