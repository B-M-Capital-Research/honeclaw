//! NotificationRouter —— 按 severity 分流 + 多重 cap/cooldown 的事件分发器。
//!
//! 拆成 6 个 sibling module:
//!
//! | 子 module    | 职责 |
//! |--------------|------|
//! | `sink`       | `OutboundSink` trait + `LogSink` + `actor_key` / `body_preview` |
//! | `stats`      | `NewsUpgradeTickStats`(单 tick 升级可观测统计) |
//! | `config`     | `NotificationRouter` struct + `new` + 17 个 `with_*` builder + tick 计数器接口 |
//! | `classify`   | `maybe_upgrade_news` + `maybe_llm_upgrade_for_actor` + 硬信号合流 |
//! | `policy`     | 系统级 / per-actor / quiet-mode 三层 severity 调整 + 9 个查询纯函数 |
//! | `dispatch`   | `NotificationRouter::dispatch` 主管线 |
//!
//! 外部仅消费几个公开符号(`OutboundSink`, `LogSink`, `NotificationRouter`),
//! 通过下面的 `pub use` 保持 `hone_event_engine::router::*` 入口稳定。

mod classify;
mod config;
mod dispatch;
mod policy;
mod sink;
mod stats;

#[cfg(test)]
mod tests;

pub use config::NotificationRouter;
pub use sink::{LogSink, OutboundSink};

// `body_preview` 被 `digest::scheduler` 拿去打日志预览,这是唯一一个跨 module
// 而非通过 NotificationRouter 入口的 helper,所以单独 re-export。
pub(crate) use sink::body_preview;
