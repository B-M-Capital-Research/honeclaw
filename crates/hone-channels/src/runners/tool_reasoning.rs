use async_trait::async_trait;
use hone_agent::FunctionCallingAgent;
use hone_agent_codex_cli::CodexCliAgent;
use hone_core::agent::Agent;
use hone_core::{LlmAuditSink, ToolExecutionObserver};
use hone_llm::LlmProvider;
use hone_tools::ToolRegistry;
use std::collections::HashMap;
use std::sync::Arc;

use crate::runtime::{get_tool_status_message, resolve_tool_reasoning};

use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
};

pub(crate) struct RunnerToolObserver {
    pub(crate) emitter: Arc<dyn AgentRunnerEmitter>,
}

#[async_trait]
impl ToolExecutionObserver for RunnerToolObserver {
    async fn on_tool_start(&self, tool_name: &str, reasoning: Option<String>) {
        let message = get_tool_status_message(tool_name, "start");
        self.emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "tool.execute",
                detail: Some(tool_name.to_string()),
            })
            .await;
        self.emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: tool_name.to_string(),
                status: "start".to_string(),
                message: if message.is_empty() {
                    None
                } else {
                    Some(message)
                },
                reasoning: resolve_tool_reasoning(tool_name, reasoning),
            })
            .await;
    }

    async fn on_tool_finish(&self, tool_name: &str, _success: bool) {
        let message = get_tool_status_message(tool_name, "done");
        self.emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: tool_name.to_string(),
                status: "done".to_string(),
                message: if message.is_empty() {
                    None
                } else {
                    Some(message)
                },
                reasoning: None,
            })
            .await;
    }
}

pub struct CodexCliReasoningRunner {
    system_prompt: String,
    codex_model: Option<String>,
    tools: Arc<ToolRegistry>,
    llm_audit: Option<Arc<dyn LlmAuditSink>>,
}

impl CodexCliReasoningRunner {
    pub fn new(
        system_prompt: String,
        codex_model: Option<String>,
        tools: Arc<ToolRegistry>,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
    ) -> Self {
        Self {
            system_prompt,
            codex_model,
            tools,
            llm_audit,
        }
    }
}

#[async_trait]
impl AgentRunner for CodexCliReasoningRunner {
    fn name(&self) -> &'static str {
        "codex_cli"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let observer = Arc::new(RunnerToolObserver { emitter });
        let agent = CodexCliAgent::new(
            self.system_prompt.clone(),
            self.codex_model.clone(),
            Some(request.working_directory.clone()),
            self.tools.clone(),
            self.llm_audit.clone(),
        )
        .with_tool_observer(Some(observer));

        let mut context = request.context;
        let response = agent.run(&request.runtime_input, &mut context).await;

        AgentRunnerResult {
            response,
            streamed_output: false,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
        }
    }
}

pub struct FunctionCallingReasoningRunner {
    llm: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    system_prompt: String,
    max_iterations: u32,
    llm_audit: Option<Arc<dyn LlmAuditSink>>,
}

impl FunctionCallingReasoningRunner {
    pub fn new(
        llm: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        system_prompt: String,
        max_iterations: u32,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
    ) -> Self {
        Self {
            llm,
            tools,
            system_prompt,
            max_iterations,
            llm_audit,
        }
    }
}

#[async_trait]
impl AgentRunner for FunctionCallingReasoningRunner {
    fn name(&self) -> &'static str {
        "function_calling"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let observer = Arc::new(RunnerToolObserver { emitter });
        let agent = FunctionCallingAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            self.system_prompt.clone(),
            self.max_iterations,
            self.llm_audit.clone(),
        )
        .with_tool_observer(Some(observer));

        let mut context = request.context;
        let response = agent.run(&request.runtime_input, &mut context).await;

        AgentRunnerResult {
            response,
            streamed_output: false,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
        }
    }
}
