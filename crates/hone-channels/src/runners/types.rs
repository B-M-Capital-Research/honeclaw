use async_trait::async_trait;
use hone_core::ActorIdentity;
use hone_core::agent::{AgentContext, AgentMessage, AgentResponse};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::agent_session::GeminiStreamOptions;
pub use crate::run_event::RunEvent as AgentRunnerEvent;

#[async_trait]
pub trait AgentRunnerEmitter: Send + Sync {
    async fn emit(&self, event: AgentRunnerEvent);
}

#[derive(Debug, Clone, Copy)]
pub struct RunnerTimeouts {
    pub step: Duration,
    pub overall: Duration,
}

#[derive(Clone)]
pub struct AgentRunnerRequest {
    pub session_id: String,
    pub actor_label: String,
    pub actor: ActorIdentity,
    pub channel_target: String,
    pub allow_cron: bool,
    pub config_path: String,
    pub runtime_dir: String,
    pub system_prompt: String,
    pub runtime_input: String,
    pub context: AgentContext,
    pub timeout: Option<Duration>,
    pub gemini_stream: GeminiStreamOptions,
    pub session_metadata: HashMap<String, Value>,
    pub working_directory: String,
    pub allowed_tools: Option<Vec<String>>,
    pub max_tool_calls: Option<u32>,
}

pub struct AgentRunnerResult {
    pub response: AgentResponse,
    pub streamed_output: bool,
    pub terminal_error_emitted: bool,
    pub session_metadata_updates: HashMap<String, Value>,
    pub context_messages: Option<Vec<AgentMessage>>,
}

#[async_trait]
pub trait AgentRunner: Send + Sync {
    fn name(&self) -> &'static str;

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult;

    /// Runner 是否自己管理对话上下文 / 历史 / 内置压缩。
    ///
    /// 返回 true 时 honeclaw 不会对其触发 SessionCompactor，也不会在每轮 prompt
    /// 里再拼接 `latest_compact_summary`，由 runner 内置的 ACP session 机制累积
    /// 与压缩。仅 ACP 系列 runner（codex_acp / opencode_acp）应当 override 为
    /// true；其它 runner（multi_agent / function_calling 等）保持默认 false。
    fn manages_own_context(&self) -> bool {
        false
    }
}
