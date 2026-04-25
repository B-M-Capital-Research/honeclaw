//! 全局 digest —— LLM 精读后每天 N 次的"今日全球要闻"广播。
//!
//! 与 per-actor `digest::` 完全独立的两条管道:
//! - `digest::`:ticker 命中 → buffer → flush 到单 actor;事件不挂 ticker 就推不到
//! - `global_digest::`:不挂 ticker,从 store 取候选池 → LLM 两段式精读
//!   → 渲染单条 broadcast 给所有 `prefs.global_digest_enabled=true` 的 direct actor
//!
//! Pipeline:
//! 1. `collector::CandidateCollector` —— 从 EventStore 拉 trusted-source High/Medium
//!    news,过滤掉 legal_ad / pr_wire / opinion_blog / earnings_transcript / 已广播
//! 2. (后续) `fetcher` 抓原文,`curator` Pass1+Pass2 LLM 精读,`scheduler` 按
//!    `GlobalDigestConfig.schedules` 触发,`renderer` 出最终文案
//!
//! 配置:`GlobalDigestConfig`(`hone-core::config::event_engine`)。
//! 用户开关:`NotificationPrefs.global_digest_enabled`(默认 true)。
//! 推送 channel:`delivery_log.channel = "global_digest"`。

pub mod audience;
pub mod collector;
pub mod curator;
pub mod fetcher;
pub mod renderer;
pub mod scheduler;

pub use audience::{AudienceBuilder, AudienceContext, BriefSource, CompanyBrief};
pub use collector::{CandidateCollector, GlobalDigestCandidate};
pub use curator::{
    BaselineCuratedItem, Curator, PersonalizedItem, PickCategory, RankedCandidate, ThesisRelation,
    UserThesis,
};
pub use fetcher::{ArticleBody, ArticleFetcher, ArticleSource};
pub use renderer::render_global_digest;
pub use scheduler::GlobalDigestScheduler;
