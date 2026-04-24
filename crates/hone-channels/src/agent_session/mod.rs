//! Agent session —— 把「一次完整对话」从 user_input 到最终
//! 持久化 + outbound 的全过程组织起来。
//!
//! 按职责拆成几个子 module(一个文件一个关注点,方便定位和测试):
//!
//! | 子 module  | 职责 |
//! |------------|------|
//! | `types`    | `AgentSession*` 公开数据类型 + event helper |
//! | `progress` | `agent.run` 的 watchdog ticker |
//! | `helpers`  | 纯函数 helper(persist 过滤、overflow 判读…) |
//! | `guard`    | daily conversation 配额 RAII guard |
//! | `emitter`  | runner -> session 事件转发器(含路径脱敏) |
//! | `restore`  | `SessionStorage` -> `AgentContext` 的唯一入口 |
//! | `core`     | `AgentSession` 主体 + `run()` 管线 |
//!
//! 外部模块仍然通过 `hone_channels::agent_session::XXX` 拿到公开符号
//! (下面的 `pub use` 把它们 re-export 到本 module 根)。

mod core;
mod emitter;
mod guard;
mod helpers;
mod progress;
mod restore;
mod types;

#[cfg(test)]
mod tests;

pub use self::core::AgentSession;
pub use self::progress::{progress_watchdog_tick, run_with_progress_ticks};
pub use self::restore::restore_context;
pub use self::types::{
    AgentRunOptions, AgentRunQuotaMode, AgentSessionError, AgentSessionErrorKind,
    AgentSessionEvent, AgentSessionListener, AgentSessionResult, GeminiStreamOptions,
    MessageMetadata,
};
