use async_trait::async_trait;
use hone_agent::FunctionCallingAgent;
use hone_agent_codex_cli::CodexCliAgent;
use hone_core::agent::{Agent, AgentContext, AgentMessage};
use hone_core::{LlmAuditSink, ToolExecutionObserver};
use hone_llm::LlmProvider;
use hone_tools::ToolRegistry;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
};

pub(crate) struct RunnerToolObserver {
    pub(crate) emitter: Arc<dyn AgentRunnerEmitter>,
}

#[async_trait]
impl ToolExecutionObserver for RunnerToolObserver {
    async fn on_tool_start(&self, tool_name: &str, arguments: &Value, reasoning: Option<String>) {
        let label = render_runner_tool_label(tool_name, arguments);
        self.emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "tool.execute",
                detail: Some(label.clone()),
            })
            .await;
        self.emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: label.clone(),
                status: "start".to_string(),
                message: None,
                reasoning: Some(render_runner_tool_reasoning(&label, reasoning)),
            })
            .await;
    }

    async fn on_tool_finish(&self, tool_name: &str, arguments: &Value, _success: bool) {
        let label = render_runner_tool_label(tool_name, arguments);
        self.emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: label.clone(),
                status: "done".to_string(),
                message: Some(format!("执行完成：{label}")),
                reasoning: None,
            })
            .await;
    }
}

pub(crate) fn render_runner_tool_label(tool_name: &str, arguments: &Value) -> String {
    let base = match tool_name {
        "web_search" => {
            if let Some(query) = tool_arg_string(arguments, &["query", "q"]) {
                format!("web_search query=\"{}\"", truncate_tool_detail(&query, 80))
            } else {
                "web_search".to_string()
            }
        }
        "data_fetch" => {
            let data_type = tool_arg_string(arguments, &["data_type"]);
            let symbol = tool_arg_string(arguments, &["symbol", "ticker"]);
            match (data_type, symbol) {
                (Some(data_type), Some(symbol)) => format!(
                    "data_fetch {} {}",
                    truncate_tool_detail(&data_type, 32),
                    truncate_tool_detail(&symbol, 32)
                ),
                (Some(data_type), None) => {
                    format!("data_fetch {}", truncate_tool_detail(&data_type, 48))
                }
                (None, Some(symbol)) => {
                    format!("data_fetch {}", truncate_tool_detail(&symbol, 48))
                }
                (None, None) => render_generic_tool_label(tool_name, arguments),
            }
        }
        "deep_research" => {
            if let Some(company) = tool_arg_string(arguments, &["company_name", "query"]) {
                format!("deep_research {}", truncate_tool_detail(&company, 72))
            } else {
                render_generic_tool_label(tool_name, arguments)
            }
        }
        "skill_tool" | "load_skill" => {
            let action = tool_arg_string(arguments, &["action"]);
            let skill = tool_arg_string(arguments, &["skill_name"]);
            match (action, skill) {
                (Some(action), Some(skill)) => format!(
                    "{tool_name} {} {}",
                    truncate_tool_detail(&action, 24),
                    truncate_tool_detail(&skill, 48)
                ),
                (Some(action), None) => {
                    format!("{tool_name} {}", truncate_tool_detail(&action, 48))
                }
                (None, Some(skill)) => {
                    format!("{tool_name} {}", truncate_tool_detail(&skill, 48))
                }
                (None, None) => render_generic_tool_label(tool_name, arguments),
            }
        }
        "portfolio" => {
            let action = tool_arg_string(arguments, &["action"]);
            let symbol = tool_arg_string(arguments, &["symbol", "ticker"]);
            match (action, symbol) {
                (Some(action), Some(symbol)) => format!(
                    "portfolio {} {}",
                    truncate_tool_detail(&action, 24),
                    truncate_tool_detail(&symbol, 24)
                ),
                _ => render_generic_tool_label(tool_name, arguments),
            }
        }
        _ => render_generic_tool_label(tool_name, arguments),
    };
    truncate_tool_detail(&base, 120)
}

fn render_generic_tool_label(tool_name: &str, arguments: &Value) -> String {
    let summary = summarize_tool_arguments(arguments);
    if summary.is_empty() {
        tool_name.to_string()
    } else {
        format!("{tool_name} {summary}")
    }
}

fn summarize_tool_arguments(arguments: &Value) -> String {
    let Value::Object(map) = arguments else {
        return String::new();
    };
    let mut pairs = Vec::new();
    for key in [
        "query",
        "q",
        "symbol",
        "ticker",
        "company_name",
        "skill_name",
        "action",
        "data_type",
        "path",
        "file_path",
        "url",
    ] {
        if let Some(value) = map.get(key) {
            let rendered = summarize_tool_argument_value(value);
            if !rendered.is_empty() {
                pairs.push(format!("{key}={rendered}"));
            }
        }
        if pairs.len() >= 2 {
            break;
        }
    }
    pairs.join(" ")
}

fn summarize_tool_argument_value(value: &Value) -> String {
    match value {
        Value::String(text) => format!("\"{}\"", truncate_tool_detail(text, 48)),
        Value::Number(number) => number.to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Array(items) => {
            if items.is_empty() {
                "[]".to_string()
            } else {
                format!("[{} items]", items.len())
            }
        }
        Value::Object(map) => format!("{{{} keys}}", map.len()),
        Value::Null => "null".to_string(),
    }
}

fn tool_arg_string(arguments: &Value, keys: &[&str]) -> Option<String> {
    let Value::Object(map) = arguments else {
        return None;
    };
    for key in keys {
        if let Some(value) = map.get(*key).and_then(|value| value.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn render_runner_tool_reasoning(label: &str, reasoning: Option<String>) -> String {
    let base = format!("正在执行：{label}");
    let note = reasoning
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| truncate_tool_detail(value, 120));
    match note {
        Some(note) if note != base => format!("{base}；说明：{note}"),
        _ => base,
    }
}

fn truncate_tool_detail(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    let total = trimmed.chars().count();
    if total <= max_chars {
        return trimmed.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let prefix = trimmed.chars().take(keep).collect::<String>();
    format!("{prefix}…")
}

pub(crate) fn runner_context_messages(
    context: &AgentContext,
    original_len: usize,
) -> Option<Vec<AgentMessage>> {
    if context.messages.len() <= original_len {
        return None;
    }
    let mut messages = context.messages[original_len..].to_vec();
    if messages
        .first()
        .is_some_and(|message| message.role == "user")
    {
        messages.remove(0);
    }
    if messages.is_empty() {
        None
    } else {
        Some(messages)
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
        let original_len = request.context.messages.len();
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
        let context_messages = runner_context_messages(&context, original_len);

        AgentRunnerResult {
            response,
            streamed_output: false,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages,
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
        let original_len = request.context.messages.len();
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
        let context_messages = runner_context_messages(&context, original_len);

        AgentRunnerResult {
            response,
            streamed_output: false,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages,
        }
    }
}
