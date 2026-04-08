use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use hone_core::ActorIdentity;
use hone_core::agent::AgentContext;
use serde_json::Value;

use crate::HoneBotCore;
use crate::agent_session::GeminiStreamOptions;
use crate::core::runtime_config_path;
use crate::prompt_audit::{PromptAuditMetadata, write_prompt_audit};
use crate::runners::{AgentRunner, AgentRunnerRequest};
use crate::sandbox::ensure_actor_sandbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    PersistentConversation,
    TransientTask,
}

#[derive(Clone)]
pub struct ExecutionRequest {
    pub mode: ExecutionMode,
    pub session_id: String,
    pub actor: ActorIdentity,
    pub channel_target: String,
    pub allow_cron: bool,
    pub system_prompt: String,
    pub runtime_input: String,
    pub context: AgentContext,
    pub timeout: Option<Duration>,
    pub gemini_stream: GeminiStreamOptions,
    pub session_metadata: HashMap<String, Value>,
    pub model_override: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub max_tool_calls: Option<u32>,
    pub prompt_audit: Option<PromptAuditMetadata>,
}

pub struct PreparedExecution {
    pub runner_name: &'static str,
    pub runner: Box<dyn AgentRunner>,
    pub runner_request: AgentRunnerRequest,
}

pub struct ExecutionService {
    core: Arc<HoneBotCore>,
}

impl ExecutionService {
    pub fn new(core: Arc<HoneBotCore>) -> Self {
        Self { core }
    }

    pub fn prepare(&self, request: ExecutionRequest) -> Result<PreparedExecution, String> {
        if let Some(metadata) = request.prompt_audit.as_ref() {
            if let Err(err) = write_prompt_audit(
                &self.core.config,
                &request.actor,
                &request.session_id,
                metadata,
                &request.system_prompt,
                &request.runtime_input,
            ) {
                tracing::warn!(
                    "[PromptAudit] failed to write audit channel={} session_id={}: {}",
                    request.actor.channel,
                    request.session_id,
                    err
                );
            }
        }

        let tool_registry = self.core.create_tool_registry(
            Some(&request.actor),
            &request.channel_target,
            request.allow_cron,
        );
        let runner = self.core.create_runner_with_model_override(
            &request.system_prompt,
            tool_registry,
            request.model_override.as_deref(),
        )?;
        let runner_name = runner.name();
        let working_directory = ensure_actor_sandbox(&request.actor)
            .map_err(|err| format!("actor sandbox 初始化失败: {err}"))?
            .to_string_lossy()
            .to_string();
        let actor_label = match request.mode {
            ExecutionMode::PersistentConversation => request.actor.user_id.clone(),
            ExecutionMode::TransientTask => request.actor.session_id(),
        };

        Ok(PreparedExecution {
            runner_name,
            runner,
            runner_request: AgentRunnerRequest {
                session_id: request.session_id,
                actor_label,
                actor: request.actor,
                channel_target: request.channel_target,
                allow_cron: request.allow_cron,
                config_path: runtime_config_path(),
                system_prompt: request.system_prompt,
                runtime_input: request.runtime_input,
                context: request.context,
                timeout: request.timeout,
                gemini_stream: request.gemini_stream,
                session_metadata: request.session_metadata,
                working_directory,
                allowed_tools: request.allowed_tools,
                max_tool_calls: request.max_tool_calls,
            },
        })
    }
}
