use async_trait::async_trait;
use hone_agent::{FunctionCallingAgent, FunctionCallingStreamObserver};
use hone_agent_codex_cli::CodexCliAgent;
use hone_core::agent::{Agent, AgentContext, AgentMessage};
use hone_core::{LlmAuditSink, ToolExecutionObserver};
use hone_llm::LlmProvider;
use hone_tools::ToolRegistry;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::runtime::sanitize_user_visible_output;

use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    TerminalStreamPolicy,
};

pub(crate) struct RunnerToolObserver {
    pub(crate) emitter: Arc<dyn AgentRunnerEmitter>,
}

struct RunnerStreamObserver {
    emitter: Arc<dyn AgentRunnerEmitter>,
    streamed_output: Arc<AtomicBool>,
    terminal_stream_policy: TerminalStreamPolicy,
    canonical_header_state: Mutex<CanonicalHeaderStreamState>,
    committed_visible_prefix: Arc<Mutex<Option<String>>>,
}

const CANONICAL_INVESTMENT_HEADER_START: &str = "数据时间：北京时间 ";
const CANONICAL_INVESTMENT_BASIS_SEPARATOR: &str = "；行情口径：";
const MAX_CANONICAL_INVESTMENT_HEADER_BYTES: usize = 768;
const MAX_CANONICAL_INVESTMENT_BASIS_CHARS: usize = 480;

#[derive(Debug, Default)]
enum CanonicalHeaderStreamState {
    #[default]
    Buffering,
    Candidate(String),
    Passthrough,
    Committed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanonicalHeaderDecision {
    Incomplete,
    Invalid,
    Complete { prefix_end: usize },
}

fn canonical_header_decision(buffer: &str) -> CanonicalHeaderDecision {
    if buffer.is_empty() {
        return CanonicalHeaderDecision::Incomplete;
    }
    if !CANONICAL_INVESTMENT_HEADER_START.starts_with(buffer)
        && !buffer.starts_with(CANONICAL_INVESTMENT_HEADER_START)
    {
        return CanonicalHeaderDecision::Invalid;
    }
    let Some(line_end) = buffer.find('\n') else {
        return if buffer.len() > MAX_CANONICAL_INVESTMENT_HEADER_BYTES {
            CanonicalHeaderDecision::Invalid
        } else {
            CanonicalHeaderDecision::Incomplete
        };
    };
    if line_end > MAX_CANONICAL_INVESTMENT_HEADER_BYTES {
        return CanonicalHeaderDecision::Invalid;
    }
    let line = &buffer[..line_end];
    if canonical_investment_header_is_safe(line) {
        CanonicalHeaderDecision::Complete {
            prefix_end: line_end + '\n'.len_utf8(),
        }
    } else {
        CanonicalHeaderDecision::Invalid
    }
}

fn canonical_investment_header_is_safe(line: &str) -> bool {
    if line.is_empty() || line.chars().any(char::is_control) {
        return false;
    }
    let Some(rest) = line.strip_prefix(CANONICAL_INVESTMENT_HEADER_START) else {
        return false;
    };
    let Some((timestamp, basis)) = rest.split_once(CANONICAL_INVESTMENT_BASIS_SEPARATOR) else {
        return false;
    };
    if chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M").is_err() {
        return false;
    }
    let basis_chars = basis.chars().count();
    if basis.trim() != basis
        || basis_chars == 0
        || basis_chars > MAX_CANONICAL_INVESTMENT_BASIS_CHARS
    {
        return false;
    }
    let sanitized = sanitize_user_visible_output(line);
    !sanitized.only_internal && !sanitized.removed_internal && sanitized.content == line
}

impl RunnerStreamObserver {
    fn events_for_content_delta(&self, content: &str) -> (Vec<AgentRunnerEvent>, Option<String>) {
        if self.terminal_stream_policy == TerminalStreamPolicy::Disabled {
            return (
                vec![AgentRunnerEvent::StreamDelta {
                    content: content.to_string(),
                }],
                None,
            );
        }

        let mut state = self
            .canonical_header_state
            .lock()
            .expect("canonical header stream state");
        match &mut *state {
            CanonicalHeaderStreamState::Buffering => {
                *state = CanonicalHeaderStreamState::Candidate(content.to_string());
            }
            CanonicalHeaderStreamState::Candidate(buffer) => buffer.push_str(content),
            CanonicalHeaderStreamState::Passthrough | CanonicalHeaderStreamState::Committed => {
                return (
                    vec![AgentRunnerEvent::StreamDelta {
                        content: content.to_string(),
                    }],
                    None,
                );
            }
        }

        let CanonicalHeaderStreamState::Candidate(buffer) = &mut *state else {
            unreachable!("buffering state becomes a candidate before classification");
        };
        match canonical_header_decision(buffer) {
            CanonicalHeaderDecision::Incomplete => (Vec::new(), None),
            CanonicalHeaderDecision::Invalid => {
                let content = std::mem::take(buffer);
                *state = CanonicalHeaderStreamState::Passthrough;
                (vec![AgentRunnerEvent::StreamDelta { content }], None)
            }
            CanonicalHeaderDecision::Complete { prefix_end } => {
                let complete = std::mem::take(buffer);
                let prefix = complete[..prefix_end].to_string();
                let remainder = complete[prefix_end..].to_string();
                *state = CanonicalHeaderStreamState::Committed;
                let mut events = vec![AgentRunnerEvent::CommittedStreamDelta {
                    content: prefix.clone(),
                }];
                if !remainder.is_empty() {
                    events.push(AgentRunnerEvent::StreamDelta { content: remainder });
                }
                (events, Some(prefix))
            }
        }
    }
}

#[async_trait]
impl FunctionCallingStreamObserver for RunnerStreamObserver {
    async fn on_content_delta(&self, content: &str) {
        if content.is_empty() {
            return;
        }
        // Tool-capable rounds remain speculative even when their text happens
        // to resemble the final answer. Deferred session emitters intentionally
        // swallow this ordinary variant.
        self.streamed_output.store(true, Ordering::Relaxed);
        self.emitter
            .emit(AgentRunnerEvent::StreamDelta {
                content: content.to_string(),
            })
            .await;
    }

    async fn on_final_content_delta(&self, content: &str) {
        if content.is_empty() {
            return;
        }
        let (events, committed_prefix) = self.events_for_content_delta(content);
        if let Some(prefix) = committed_prefix {
            *self
                .committed_visible_prefix
                .lock()
                .expect("committed visible prefix") = Some(prefix);
        }
        for event in events {
            self.streamed_output.store(true, Ordering::Relaxed);
            self.emitter.emit(event).await;
        }
    }

    fn committed_visible_prefix(&self) -> Option<String> {
        self.committed_visible_prefix
            .lock()
            .expect("committed visible prefix")
            .clone()
    }

    async fn on_content_reset(&self) {
        self.streamed_output.store(false, Ordering::Relaxed);
        self.emitter.emit(AgentRunnerEvent::StreamReset).await;
    }
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

pub(crate) struct CodexCliReasoningRunner {
    system_prompt: String,
    codex_model: Option<String>,
    tools: Arc<ToolRegistry>,
    llm_audit: Option<Arc<dyn LlmAuditSink>>,
}

impl CodexCliReasoningRunner {
    pub(crate) fn new(
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
            committed_visible_prefix: None,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages,
        }
    }
}

pub(crate) struct FunctionCallingReasoningRunner {
    llm: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    system_prompt: String,
    max_iterations: u32,
    llm_audit: Option<Arc<dyn LlmAuditSink>>,
}

impl FunctionCallingReasoningRunner {
    pub(crate) fn new(
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
        let streamed_output = Arc::new(AtomicBool::new(false));
        let committed_visible_prefix = Arc::new(Mutex::new(None));
        let observer = Arc::new(RunnerToolObserver {
            emitter: emitter.clone(),
        });
        let stream_observer = Arc::new(RunnerStreamObserver {
            emitter,
            streamed_output: streamed_output.clone(),
            terminal_stream_policy: request.terminal_stream_policy,
            canonical_header_state: Mutex::new(CanonicalHeaderStreamState::default()),
            committed_visible_prefix: committed_visible_prefix.clone(),
        });
        let original_len = request.context.messages.len();
        let agent = FunctionCallingAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            self.system_prompt.clone(),
            self.max_iterations,
            self.llm_audit.clone(),
        )
        .with_finish_research_terminal_synthesis(
            request.terminal_stream_policy == TerminalStreamPolicy::CanonicalInvestmentHeader,
        )
        .with_tool_observer(Some(observer))
        .with_stream_observer(Some(stream_observer))
        .with_tool_call_budget(
            request.max_tool_calls,
            request.tool_call_limits.clone().unwrap_or_default(),
        );

        let mut context = request.context;
        let response = agent.run(&request.runtime_input, &mut context).await;
        let context_messages = runner_context_messages(&context, original_len);
        let committed_visible_prefix = committed_visible_prefix
            .lock()
            .expect("committed visible prefix")
            .clone();

        AgentRunnerResult {
            response,
            streamed_output: streamed_output.load(Ordering::Relaxed),
            committed_visible_prefix,
            terminal_error_emitted: false,
            session_metadata_updates: HashMap::new(),
            context_messages,
        }
    }
}

#[cfg(test)]
mod terminal_stream_tests {
    use super::*;

    #[derive(Default)]
    struct CaptureEmitter {
        events: Mutex<Vec<AgentRunnerEvent>>,
    }

    #[async_trait]
    impl AgentRunnerEmitter for CaptureEmitter {
        async fn emit(&self, event: AgentRunnerEvent) {
            self.events.lock().expect("captured events").push(event);
        }
    }

    fn stream_observer(
        policy: TerminalStreamPolicy,
    ) -> (
        RunnerStreamObserver,
        Arc<CaptureEmitter>,
        Arc<Mutex<Option<String>>>,
    ) {
        let emitter = Arc::new(CaptureEmitter::default());
        let committed_visible_prefix = Arc::new(Mutex::new(None));
        (
            RunnerStreamObserver {
                emitter: emitter.clone(),
                streamed_output: Arc::new(AtomicBool::new(false)),
                terminal_stream_policy: policy,
                canonical_header_state: Mutex::new(CanonicalHeaderStreamState::default()),
                committed_visible_prefix: committed_visible_prefix.clone(),
            },
            emitter,
            committed_visible_prefix,
        )
    }

    #[tokio::test]
    async fn canonical_investment_header_commits_exact_prefix_across_chunks() {
        let (observer, emitter, committed_prefix) =
            stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);
        let header = concat!(
            "数据时间：北京时间 2026-07-18 21:05；行情口径：",
            "报价源时间：北京时间 2026-07-18 04:00（最新可得，非逐笔）\n"
        );

        observer.on_final_content_delta("数据时间：北京").await;
        observer
            .on_final_content_delta("时间 2026-07-18 21:05；行情口径：报价源时间：北京")
            .await;
        assert!(emitter.events.lock().expect("captured events").is_empty());

        observer
            .on_final_content_delta("时间 2026-07-18 04:00（最新可得，非逐笔）\n\n## 结论")
            .await;
        observer
            .on_final_content_delta("\nCRWV 与英伟达关系紧密。")
            .await;

        let events = emitter.events.lock().expect("captured events");
        assert_eq!(events.len(), 3, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentRunnerEvent::CommittedStreamDelta { content } if content == header
        ));
        assert!(matches!(
            &events[1],
            AgentRunnerEvent::StreamDelta { content } if content == "\n## 结论"
        ));
        assert!(matches!(
            &events[2],
            AgentRunnerEvent::StreamDelta { content } if content == "\nCRWV 与英伟达关系紧密。"
        ));
        assert_eq!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .as_deref(),
            Some(header)
        );
        assert_eq!(
            observer.committed_visible_prefix().as_deref(),
            Some(header),
            "the Agent must be able to read the exact irreversible prefix for terminal-only recovery"
        );
    }

    #[tokio::test]
    async fn speculative_preamble_resets_without_committing_then_next_header_can_commit() {
        let (observer, emitter, committed_prefix) =
            stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);
        observer.on_content_delta("我先继续核验工具结果").await;
        observer.on_content_reset().await;
        observer
            .on_final_content_delta(
                "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n正文",
            )
            .await;

        let events = emitter.events.lock().expect("captured events");
        assert_eq!(events.len(), 4, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentRunnerEvent::StreamDelta { content } if content == "我先继续核验工具结果"
        ));
        assert!(matches!(&events[1], AgentRunnerEvent::StreamReset));
        assert!(matches!(
            &events[2],
            AgentRunnerEvent::CommittedStreamDelta { content }
                if content == "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n"
        ));
        assert!(matches!(
            &events[3],
            AgentRunnerEvent::StreamDelta { content } if content == "正文"
        ));
        assert_eq!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .as_deref(),
            Some("数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n")
        );
    }

    #[tokio::test]
    async fn disabled_terminal_policy_preserves_ordinary_stream_deltas() {
        let (observer, emitter, committed_prefix) = stream_observer(TerminalStreamPolicy::Disabled);
        observer.on_content_delta("普通").await;
        observer.on_content_delta("流式正文").await;

        let events = emitter.events.lock().expect("captured events");
        assert_eq!(events.len(), 2, "{events:?}");
        assert!(
            events
                .iter()
                .all(|event| matches!(event, AgentRunnerEvent::StreamDelta { .. }))
        );
        assert!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .is_none()
        );
    }

    #[tokio::test]
    async fn canonical_looking_tool_round_text_remains_speculative() {
        let (observer, emitter, committed_prefix) =
            stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);
        let draft = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";

        observer.on_content_delta(draft).await;

        let events = emitter.events.lock().expect("captured events");
        assert_eq!(events.len(), 1, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentRunnerEvent::StreamDelta { content } if content == draft
        ));
        assert!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .is_none()
        );
    }

    #[tokio::test]
    async fn invalid_or_oversized_terminal_header_never_commits() {
        for invalid in [
            "---\n数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得\n".to_string(),
            format!(
                "数据时间：北京时间 2026-07-18 21:05；行情口径：{}",
                "很长".repeat(MAX_CANONICAL_INVESTMENT_HEADER_BYTES)
            ),
        ] {
            let (observer, emitter, committed_prefix) =
                stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);

            observer.on_final_content_delta(&invalid).await;

            let events = emitter.events.lock().expect("captured events");
            assert_eq!(events.len(), 1, "{events:?}");
            assert!(matches!(
                &events[0],
                AgentRunnerEvent::StreamDelta { content } if content == &invalid
            ));
            assert!(
                committed_prefix
                    .lock()
                    .expect("committed visible prefix")
                    .is_none()
            );
        }
    }
}
