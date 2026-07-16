//! ACP (Agent Client Protocol) 共享层 —— 把 `codex_acp` / `opencode_acp`
//! 的公共管线按职责切成 7 个 sibling module:
//!
//! `gemini_acp` 运行时已全局禁用,但历史事件样例仍覆盖 Gemini ACP 的旧字段
//! 形状,所以部分抽取逻辑继续兼容这类 payload。
//!
//! | 子 module     | 职责 |
//! |---------------|------|
//! | `state`       | 数据类型(`AcpPromptState` / `AcpPermissionDecision` …) + 常量 + regex |
//! | `extract`     | JSON 字段抽取纯函数(tool call id/name/arguments/result/failure) |
//! | `tool_state`  | Tool call 状态机(capture start/finish + assistant/tool message pair) |
//! | `log`         | `acp-events.log` 写入 + tracing 诊断格式化 |
//! | `ingest`      | 把 `session/update` 翻译成 runner event + compact 检测 |
//! | `protocol`    | JSON-RPC 线上协议(`session/new` / `session/prompt` / permission / timeout) |
//! | `version`     | `CliVersion` 解析(每个 runner 校版本下限用) |
//!
//! 外部 runner 原来通过 `super::acp_common::{...}` 消费本 module,切完之后
//! 通过下面一整块 `pub(crate) use` 继续暴露同名符号,让 active runner 和
//! sibling runner 都不必关心内部文件拆分。

mod extract;
mod failure;
mod ingest;
mod log;
mod process;
mod protocol;
mod state;
mod tool_state;
mod version;

#[cfg(test)]
mod tests;

// ── 外部 runner 消费的符号 ──
// 这张 re-export 表就是「acp_common 模块的公共接口」,改 sibling runner 的
// 地方要跟着更新这里,否则老的 `use super::acp_common::X` 会直接 broken。

pub(crate) use failure::acp_failure_to_runner_result;
pub(crate) use ingest::{acp_prompt_succeeded, ingest_acp_message_chunk, ingest_acp_usage_update};
pub(crate) use log::{
    AcpEventLogContext, acp_diagnostic_excerpt_for_log, acp_error_detail_for_message,
    log_acp_payload, log_acp_prompt_stop_diagnostics, log_acp_raw_parse_error,
    message_with_bounded_stderr, timeout_message_with_stderr,
};
pub(crate) use process::{AcpChildGuard, configure_acp_command_process_group};
pub(crate) use protocol::{
    create_acp_session, set_acp_session_model, wait_for_response,
    wait_for_response_with_timeouts_and_renderer, write_jsonrpc_request,
};
pub(crate) use state::{
    ACP_NEEDS_SP_RESEED_KEY, ACP_PREV_PROMPT_PEAK_KEY, AcpPermissionDecision, AcpPromptState,
    AcpRenderedToolStatus, AcpResponseTimeouts, AcpRunFailure, AcpToolCallRecord,
    AcpToolRenderPhase,
};
pub(crate) use tool_state::{finalize_context_messages, finalize_pending_tool_calls};
pub(crate) use version::{CliVersion, parse_cli_version};

// ── 仅测试消费的 re-export ──
// `runners/tests.rs` 以 `use super::acp_common::{...}` 的形式直接复用 acp_common
// 的 handler / log 格式化 helper 做黑盒断言;这些符号在 lib release 路径上并不
// 被链接,因此 gate 在 `#[cfg(test)]` 之下,避免 rustc 报 unused warning。
#[cfg(test)]
pub(crate) use ingest::handle_acp_session_update;
#[cfg(test)]
pub(crate) use log::summarize_finished_tool_calls_for_log;
#[cfg(test)]
pub(crate) use tool_state::extract_finished_tool_calls;
