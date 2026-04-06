mod acp_common;
mod codex_acp;
mod gemini_acp;
mod gemini_cli;
mod multi_agent;
mod opencode_acp;
mod tool_reasoning;
mod types;

pub use codex_acp::CodexAcpRunner;
pub use gemini_acp::GeminiAcpRunner;
pub use gemini_cli::GeminiCliRunner;
#[cfg(test)]
pub(crate) use gemini_cli::stream_gemini_prompt;
pub use multi_agent::MultiAgentRunner;
pub use opencode_acp::OpencodeAcpRunner;
pub use tool_reasoning::{CodexCliReasoningRunner, FunctionCallingReasoningRunner};
pub use types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
};

#[cfg(test)]
mod tests;
