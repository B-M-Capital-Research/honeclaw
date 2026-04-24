//! hone-event-engine — 主动事件引擎
//!
//! 负责：
//! 1. Pollers（纯 Rust、无 LLM）从数据源拉取市场事件
//! 2. 去重（EventStore）后发布到订阅分发层
//! 3. 按持仓/订阅分发，按 severity 分流（高优实时、低/中优先进每日摘要）
//! 4. 复用 hone-channels 的 outbound 派发渠道消息（MVP 用 LogSink，后续替换）

pub mod daily_report;
pub mod digest;
pub mod event;
pub mod fmp;
pub mod news_classifier;
pub mod polisher;
pub mod pollers;
pub mod prefs;
pub mod renderer;
pub mod router;
pub mod sinks;
pub mod source;
pub mod store;
pub mod subscription;

// ── 内部子 module:engine 主体 + spawn 模板 + 共享 pipeline ──
// 保持 crate 私有:EventEngine 通过下面的 pub use 暴露,其它三个不外露。
mod engine;
mod pipeline;
mod spawner;

#[cfg(test)]
mod tests;

pub use daily_report::DailyReport;
pub use digest::{DigestBuffer, DigestScheduler};
pub use engine::EventEngine;
pub use event::{EventKind, MarketEvent, Severity};
pub use fmp::FmpClient;
pub use hone_core::config::{EventEngineConfig, FmpConfig};
pub use news_classifier::{
    DEFAULT_IMPORTANCE_PROMPT, Importance, LlmNewsClassifier, NewsClassifier, NoopClassifier,
};
pub use polisher::{BodyPolisher, LlmPolisher, NoopPolisher, parse_polish_levels};
pub use pollers::{
    AnalystGradePoller, CorpActionPoller, EarningsPoller, EarningsSurprisePoller, MacroPoller,
    NewsPoller, PricePoller, TelegramChannelPoller, TruthSocialPoller,
};
pub use prefs::{
    AllowAllPrefs, FilePrefsStorage, NotificationPrefs, PrefsProvider, SharedPrefs, kind_tag,
};
pub use renderer::RenderFormat;
pub use router::{LogSink, NotificationRouter, OutboundSink};
pub use sinks::{DiscordSink, FeishuSink, IMessageSink, MultiChannelSink, TelegramSink};
pub use source::{EventSource, FnSource, SourceSchedule};
pub use store::EventStore;
pub use subscription::{
    GlobalSubscription, PortfolioSubscription, SharedRegistry, Subscription, SubscriptionRegistry,
    registry_from_portfolios,
};
