//! `HoneBotCore` —— 所有渠道共用的 Bot 核心。
//!
//! 按职责拆成几个 sibling module,每个文件一个关注点:
//!
//! | 子 module        | 职责 |
//! |------------------|------|
//! | `bot_core`       | `HoneBotCore` struct 定义 + 构造 + LLM / tool / runner 工厂 + session 压缩 |
//! | `intercept`      | `/register-admin` / `/report` 命令拦截 + workflow bridge HTTP |
//! | `logging`        | 启动路由日志 + 消息流日志(`log_message_*`)+ 格式化 helper |
//! | `config_loader`  | `runtime_config_path` / `load_runtime_config` 进程启动时读配置 |
//!
//! 外部仍然通过 `hone_channels::core::*` 或 `hone_channels::HoneBotCore`
//! (lib.rs 再导出)访问,内部拆分对调用方透明。

mod bot_core;
mod config_loader;
mod intercept;
mod logging;

#[cfg(test)]
mod tests;

pub use self::bot_core::{CompactSessionOutcome, HoneBotCore};
pub use self::config_loader::{load_runtime_config, runtime_config_path};
pub use self::intercept::{
    REGISTER_ADMIN_INTERCEPT_ACK, REGISTER_ADMIN_INTERCEPT_DENY_ACK,
    REGISTER_ADMIN_INTERCEPT_DISABLED_ACK, REGISTER_ADMIN_INTERCEPT_INVALID_ACK,
    REGISTER_ADMIN_INTERCEPT_PREFIX,
};
