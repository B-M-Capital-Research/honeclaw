//! 全局 digest 内核 —— curator (Pass1+Pass2) / fetcher / audience / event_dedupe /
//! thesis_distill。从 commit 3 起,scheduler 已合并进 `unified_digest::scheduler`,
//! 本目录只保留为 unified pipeline 的 LLM 内核;`GlobalNewsSource` 在
//! `unified_digest::sources::global` 里 wrap `collector` 输出成 `UnifiedCandidate`。
//!
//! Pipeline(由 `unified_digest::scheduler` 编排):
//! 1. `collector::CandidateCollector` —— 从 EventStore 拉 trusted-source High/Medium
//!    news,过滤掉 legal_ad / pr_wire / opinion_blog / earnings_transcript / 已广播
//! 2. `event_dedupe` 折叠同事件多源 → `fetcher` 抓原文 → `curator` Pass1+Pass2 LLM 精读
//! 3. 输出经 unified scheduler per-actor fan-out,与 buffer/synth 候选池合流后渲染
//!
//! 配置:`GlobalDigestConfig`(`hone-core::config::event_engine`),仍是 LLM 子配置容器。
//! 推送 channel:`delivery_log.channel = "unified_digest"`。

pub mod audience;
pub mod collector;
pub mod curator;
pub mod event_dedupe;
pub mod fetcher;
pub mod renderer;
pub mod thesis_cron;
pub mod thesis_distill;

pub use audience::{AudienceBuilder, AudienceContext, BriefSource, CompanyBrief};
pub use collector::{CandidateCollector, GlobalDigestCandidate};
pub use curator::{
    BaselineCuratedItem, Curator, PersonalizedItem, PickCategory, RankedCandidate, ThesisRelation,
    UserThesis,
};
pub use event_dedupe::{
    ClusterAudit, DedupeStats, EventDeduper, LlmEventDeduper, PassThroughDeduper,
};
pub use fetcher::{ArticleBody, ArticleFetcher, ArticleSource};
pub use renderer::render_global_digest;
pub use thesis_cron::{
    DEFAULT_DISTILL_INTERVAL_HOURS, MIN_RETRY_INTERVAL_HOURS, TriggerReason, WEEKLY_REFRESH_HOURS,
    distill_cron_loop, distill_tick, should_trigger,
};
pub use thesis_distill::{
    DistilledTheses, LlmThesisDistiller, ProfileSource, ThesisDistiller, actor_sandbox_dir,
    distill_and_persist_one, distill_for_actor, extract_tickers, merge_into_prefs, scan_profiles,
};
