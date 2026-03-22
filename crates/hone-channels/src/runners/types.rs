use async_trait::async_trait;
use hone_core::ActorIdentity;
use hone_core::agent::{AgentContext, AgentResponse};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::agent_session::{AgentSessionError, GeminiStreamOptions};

#[derive(Debug, Clone)]
pub enum AgentRunnerEvent {
    Progress {
        stage: &'static str,
        detail: Option<String>,
    },
    StreamDelta {
        content: String,
    },
    StreamThought {
        thought: String,
    },
    ToolStatus {
        tool: String,
        status: String,
        message: Option<String>,
        reasoning: Option<String>,
    },
    Error {
        error: AgentSessionError,
    },
}

#[async_trait]
pub trait AgentRunnerEmitter: Send + Sync {
    async fn emit(&self, event: AgentRunnerEvent);
}

#[derive(Clone)]
pub struct AgentRunnerRequest {
    pub session_id: String,
    pub actor_label: String,
    pub actor: ActorIdentity,
    pub channel_target: String,
    pub allow_cron: bool,
    pub config_path: String,
    pub system_prompt: String,
    pub runtime_input: String,
    pub context: AgentContext,
    pub timeout: Option<Duration>,
    pub gemini_stream: GeminiStreamOptions,
    pub session_metadata: HashMap<String, Value>,
    pub working_directory: String,
}

pub struct AgentRunnerResult {
    pub response: AgentResponse,
    pub streamed_output: bool,
    pub terminal_error_emitted: bool,
    pub session_metadata_updates: HashMap<String, Value>,
}

#[async_trait]
pub trait AgentRunner: Send + Sync {
    fn name(&self) -> &'static str;

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult;
}
