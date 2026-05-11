//! Pollers：事件源适配器。
//!
//! 每个 poller 拉取一类数据并产出 `MarketEvent`。固定间隔源和 cron-aligned
//! 源都通过 `EventSource::schedule()` 交给统一 spawner 驱动。

pub mod analyst_grade;
pub mod corp_action;
pub mod earnings;
pub mod earnings_quality;
pub mod earnings_surprise;
pub mod extended_hours;
pub mod macro_events;
pub mod news;
pub mod price;
pub mod rss;
pub mod sec_enrichment;
pub mod social;

pub use analyst_grade::AnalystGradePoller;
pub use corp_action::{CorpActionCalendarPoller, SecFilingsPoller};
pub use earnings::EarningsPoller;
pub use earnings_surprise::EarningsSurprisePoller;
pub use extended_hours::ExtendedHoursPoller;
pub use macro_events::MacroPoller;
pub use news::NewsPoller;
pub use price::PricePoller;
pub use rss::RssNewsPoller;
pub use sec_enrichment::{LlmSecFilingSummarizer, NoopSecFilingSummarizer, SecFilingSummarizer};
pub use social::TelegramChannelPoller;
