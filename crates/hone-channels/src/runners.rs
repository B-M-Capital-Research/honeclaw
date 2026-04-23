mod acp_common;
mod codex_acp;
// gemini_acp 已被全局禁用（见 core.rs 工厂层 + docs/bugs/...）。
// 模块代码保留，方便日后重新启用，因此整体允许 dead_code。
#[allow(dead_code)]
mod gemini_acp;
mod gemini_cli;
mod multi_agent;
mod opencode_acp;
mod tool_reasoning;
mod types;

pub use codex_acp::CodexAcpRunner;
pub use gemini_cli::GeminiCliRunner;
#[cfg(test)]
pub(crate) use gemini_cli::stream_gemini_prompt;
pub use multi_agent::MultiAgentRunner;
pub use opencode_acp::OpencodeAcpRunner;
pub use tool_reasoning::{CodexCliReasoningRunner, FunctionCallingReasoningRunner};
pub use types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    RunnerTimeouts,
};

#[cfg(test)]
mod tests;
