mod acp_common;
mod codex_acp;
// gemini_acp 已被全局禁用（见 core/bot_core.rs 工厂层 + docs/invariants.md）。
// 模块代码仅保留 legacy 参数清洗和版本校验测试夹具。
#[cfg(test)]
mod gemini_acp;
mod gemini_cli;
mod hone_cloud;
mod opencode_acp;
mod tool_reasoning;
mod types;

pub(crate) use codex_acp::CodexAcpRunner;
pub(crate) use gemini_cli::GeminiCliRunner;
#[cfg(test)]
pub(crate) use gemini_cli::stream_gemini_prompt;
pub(crate) use hone_cloud::HoneCloudRunner;
pub(crate) use opencode_acp::OpencodeAcpRunner;
pub(crate) use tool_reasoning::CodexCliReasoningRunner;
pub(crate) use types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    RunnerTimeouts,
};

#[cfg(test)]
mod tests;
