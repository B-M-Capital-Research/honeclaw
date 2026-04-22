//! Pollers：事件源适配器。
//!
//! 每个 poller 拉取一类数据并产出 `MarketEvent`。当前仅实现 earnings；
//! 后续 news / price / corp_action / macro / snapshot 将陆续加入。

pub mod analyst_grade;
pub mod corp_action;
pub mod earnings;
pub mod earnings_surprise;
pub mod macro_events;
pub mod news;
pub mod price;
pub mod social;

pub use analyst_grade::AnalystGradePoller;
pub use corp_action::CorpActionPoller;
pub use earnings::EarningsPoller;
pub use earnings_surprise::EarningsSurprisePoller;
pub use macro_events::MacroPoller;
pub use news::NewsPoller;
pub use price::PricePoller;
pub use social::{TelegramChannelPoller, TruthSocialPoller};
