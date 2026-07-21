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

use crate::response_finalizer::response_leaks_system_prompt;
use crate::runtime::{sanitize_agent_owned_user_visible_output, sanitize_user_visible_output};

use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    RunnerTimeouts, TerminalStreamPolicy,
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
    Rejected,
    Committed {
        pending_body: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CanonicalHeaderDecision {
    Incomplete,
    Invalid,
    Complete { header_end: usize },
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
            header_end: line_end,
        }
    } else {
        CanonicalHeaderDecision::Invalid
    }
}

fn canonical_investment_header_is_safe(line: &str) -> bool {
    if line.is_empty() || line.chars().any(char::is_control) {
        return false;
    }
    if response_leaks_system_prompt(line) {
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

fn canonical_body_line_is_safe(line: &str) -> bool {
    if line.contains('\r') {
        return false;
    }
    if line.is_empty() {
        return true;
    }
    if response_leaks_system_prompt(line) {
        return false;
    }

    // The completed response passes through this same security-only
    // sanitizer before persistence. Stream only complete lines which it would
    // preserve byte-for-byte; anything suspicious falls back to the deferred
    // final tail instead of becoming irreversible output.
    let lower = line.to_ascii_lowercase();
    if [
        "<think",
        "</think",
        "<tool_code",
        "</tool_code",
        "<tool_call",
        "</tool_call",
        "<tool_result",
        "</tool_result",
        "<tool_use",
        "</tool_use",
        "[tool_call",
        "[/tool_call",
        "[tool_result",
        "[/tool_result",
        "[tool_use",
        "[/tool_use",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return false;
    }

    let sanitized = sanitize_agent_owned_user_visible_output(line);
    !sanitized.only_internal && !sanitized.removed_internal && sanitized.content == line
}

/// Drain only through the content of a complete, nonempty, security-clean
/// line. Its terminating newline remains buffered until a later complete safe
/// line exists, so the emitted prefix can never end in whitespace that the
/// completed-response sanitizer's final `trim()` might remove.
fn drain_stable_canonical_body_prefix(pending: &mut String) -> (String, bool) {
    let mut line_start = 0;
    let mut safe_end = 0;
    for newline in pending.match_indices('\n').map(|(index, _)| index) {
        let visible_line = &pending[line_start..newline];
        if !canonical_body_line_is_safe(visible_line) {
            let safe = pending.drain(..safe_end).collect();
            return (safe, true);
        }
        if visible_line.chars().any(|ch| !ch.is_whitespace()) {
            // Include every separator before this line, but retain this line's
            // own terminator until another complete safe line follows.
            safe_end = newline;
        }
        line_start = newline + '\n'.len_utf8();
    }

    (pending.drain(..safe_end).collect(), false)
}

impl RunnerStreamObserver {
    /// Commit a typed, server-owned finance header before the first model
    /// request. Unlike a provider draft, this prefix is part of the final
    /// response contract and remains valid while later read-only research
    /// tools run. State changes only after the unique downstream sink ACKs.
    async fn commit_service_owned_prefix(&self, prefix: &str) -> bool {
        if self.terminal_stream_policy != TerminalStreamPolicy::CanonicalInvestmentHeader
            || prefix.contains(['\r', '\n'])
            || !canonical_investment_header_is_safe(prefix)
            || self.committed_visible_prefix().is_some()
        {
            return false;
        }
        if !self
            .emitter
            .emit_committed(AgentRunnerEvent::CommittedStreamDelta {
                content: prefix.to_string(),
            })
            .await
        {
            return false;
        }

        *self
            .canonical_header_state
            .lock()
            .expect("canonical header stream state") = CanonicalHeaderStreamState::Committed {
            pending_body: String::new(),
        };
        *self
            .committed_visible_prefix
            .lock()
            .expect("committed visible prefix") = Some(prefix.to_string());
        self.streamed_output.store(true, Ordering::Relaxed);
        true
    }

    fn event_for_final_content_delta(
        &self,
        content: &str,
    ) -> Option<(AgentRunnerEvent, Option<String>)> {
        if self.terminal_stream_policy == TerminalStreamPolicy::Disabled {
            return Some((
                AgentRunnerEvent::StreamDelta {
                    content: content.to_string(),
                },
                None,
            ));
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
            CanonicalHeaderStreamState::Rejected => {
                return None;
            }
            CanonicalHeaderStreamState::Committed { pending_body } => {
                pending_body.push_str(content);
                let (safe_body, rejected) = drain_stable_canonical_body_prefix(pending_body);
                if rejected {
                    *state = CanonicalHeaderStreamState::Rejected;
                }
                return (!safe_body.is_empty()).then(|| {
                    (
                        AgentRunnerEvent::CommittedStreamDelta {
                            content: safe_body.clone(),
                        },
                        Some(safe_body),
                    )
                });
            }
        }

        let CanonicalHeaderStreamState::Candidate(buffer) = &mut *state else {
            unreachable!("buffering state becomes a candidate before classification");
        };
        let leading_whitespace_bytes = buffer
            .char_indices()
            .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))
            .unwrap_or(buffer.len());
        if leading_whitespace_bytes > 0 {
            buffer.drain(..leading_whitespace_bytes);
        }
        match canonical_header_decision(buffer) {
            CanonicalHeaderDecision::Incomplete => None,
            CanonicalHeaderDecision::Invalid => {
                *state = CanonicalHeaderStreamState::Rejected;
                None
            }
            CanonicalHeaderDecision::Complete { header_end } => {
                let complete = std::mem::take(buffer);
                // Commit the canonical line without its terminator. If the
                // model stops after the header or the entire body is removed
                // by final security cleanup, the finalized response still
                // begins with these exact bytes.
                let mut committed_delta = complete[..header_end].to_string();
                let mut pending_body = complete[header_end..].to_string();
                let (safe_body, rejected) = drain_stable_canonical_body_prefix(&mut pending_body);
                committed_delta.push_str(&safe_body);
                *state = if rejected {
                    CanonicalHeaderStreamState::Rejected
                } else {
                    CanonicalHeaderStreamState::Committed { pending_body }
                };
                Some((
                    AgentRunnerEvent::CommittedStreamDelta {
                        content: committed_delta.clone(),
                    },
                    Some(committed_delta),
                ))
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
        let Some((event, committed_prefix)) = self.event_for_final_content_delta(content) else {
            return;
        };

        // A committed prefix becomes irreversible only after the downstream
        // emitter has accepted it. If this future is cancelled while the
        // emitter is blocked, neither the prefix nor streamed-output state may
        // claim that the user saw bytes which never left this observer.
        if !self.emitter.emit_committed(event).await {
            *self
                .canonical_header_state
                .lock()
                .expect("canonical header stream state") = CanonicalHeaderStreamState::Rejected;
            return;
        }
        self.streamed_output.store(true, Ordering::Relaxed);
        if let Some(delta) = committed_prefix {
            let mut committed = self
                .committed_visible_prefix
                .lock()
                .expect("committed visible prefix");
            if let Some(prefix) = committed.as_mut() {
                prefix.push_str(&delta);
            } else {
                *committed = Some(delta);
            }
        }
    }

    fn committed_visible_prefix(&self) -> Option<String> {
        self.committed_visible_prefix
            .lock()
            .expect("committed visible prefix")
            .clone()
    }

    async fn commit_service_owned_prefix(&self, prefix: &str) -> bool {
        RunnerStreamObserver::commit_service_owned_prefix(self, prefix).await
    }

    async fn on_content_reset(&self) {
        if self.terminal_stream_policy == TerminalStreamPolicy::CanonicalInvestmentHeader {
            // A committed header is irreversible. Otherwise clear any
            // incomplete/rejected final candidate so the next model round can
            // start from a fresh canonical header.
            if self.committed_visible_prefix().is_some() {
                return;
            }
            *self
                .canonical_header_state
                .lock()
                .expect("canonical header stream state") = CanonicalHeaderStreamState::default();
        }

        // An uncommitted final candidate never crossed the emitter, so its
        // reset is internal-only. Speculative ordinary deltas still receive
        // the public reset required to remove bytes they actually emitted.
        if self.streamed_output.swap(false, Ordering::Relaxed) {
            self.emitter.emit(AgentRunnerEvent::StreamReset).await;
        }
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

    async fn on_tool_finish(&self, tool_name: &str, arguments: &Value, success: bool) {
        let label = render_runner_tool_label(tool_name, arguments);
        let (status, message) = if success {
            ("done", format!("执行完成：{label}"))
        } else {
            ("failed", format!("执行失败：{label}"))
        };
        self.emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: label.clone(),
                status: status.to_string(),
                message: Some(message),
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
    timeouts: RunnerTimeouts,
}

impl FunctionCallingReasoningRunner {
    pub(crate) fn new(
        llm: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        system_prompt: String,
        max_iterations: u32,
        llm_audit: Option<Arc<dyn LlmAuditSink>>,
        timeouts: RunnerTimeouts,
    ) -> Self {
        Self {
            llm,
            tools,
            system_prompt,
            max_iterations,
            llm_audit,
            timeouts,
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
        let overall_timeout = request.timeout.unwrap_or(self.timeouts.overall);
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
        let service_owned_initial_prefix = request.service_owned_initial_prefix.clone();
        let service_owned_prefix_content = service_owned_initial_prefix
            .as_ref()
            .map(|prefix| prefix.content.clone());
        let precommitted_service_prefix = match service_owned_initial_prefix.as_ref() {
            Some(prefix) if request.agent_owned_finance_loop && prefix.commit_before_model => {
                stream_observer
                    .commit_service_owned_prefix(&prefix.content)
                    .await
                    .then(|| prefix.content.clone())
            }
            _ => None,
        };
        let original_len = request.context.messages.len();
        let agent = FunctionCallingAgent::new(
            self.llm.clone(),
            self.tools.clone(),
            self.system_prompt.clone(),
            self.max_iterations,
            self.llm_audit.clone(),
        )
        .with_agent_owned_finance_loop(request.agent_owned_finance_loop)
        .with_service_owned_initial_prefix(
            service_owned_prefix_content,
            precommitted_service_prefix,
        )
        .with_tool_observer(Some(observer))
        .with_stream_observer(Some(stream_observer))
        .with_tool_call_budget(
            request.max_tool_calls,
            request.tool_call_limits.clone().unwrap_or_default(),
        )
        .with_step_timeout(Some(self.timeouts.step))
        .with_overall_timeout(Some(overall_timeout));

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
    async fn service_owned_prefix_ack_records_exact_irreversible_bytes_once() {
        let (observer, emitter, committed_prefix) =
            stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);
        let prefix = "数据时间：北京时间 2026-07-22 10:30；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露";

        assert!(observer.commit_service_owned_prefix(prefix).await);
        assert!(!observer.commit_service_owned_prefix(prefix).await);

        let events = emitter.events.lock().expect("captured events");
        assert_eq!(events.len(), 1, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentRunnerEvent::CommittedStreamDelta { content } if content == prefix
        ));
        assert_eq!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .as_deref(),
            Some(prefix)
        );
        assert!(observer.streamed_output.load(Ordering::Relaxed));
        assert!(matches!(
            *observer
                .canonical_header_state
                .lock()
                .expect("canonical header state"),
            CanonicalHeaderStreamState::Committed { ref pending_body }
                if pending_body.is_empty()
        ));
    }

    #[tokio::test]
    async fn rejected_service_owned_prefix_ack_never_becomes_visible_state() {
        let committed_visible_prefix = Arc::new(Mutex::new(None));
        let streamed_output = Arc::new(AtomicBool::new(false));
        let observer = RunnerStreamObserver {
            emitter: Arc::new(RejectingCommitEmitter),
            streamed_output: streamed_output.clone(),
            terminal_stream_policy: TerminalStreamPolicy::CanonicalInvestmentHeader,
            canonical_header_state: Mutex::new(CanonicalHeaderStreamState::default()),
            committed_visible_prefix: committed_visible_prefix.clone(),
        };
        let prefix = "数据时间：北京时间 2026-07-22 10:30；行情口径：本轮仅使用可核验资料，具体报价时间与数据缺口在正文逐项披露";

        assert!(!observer.commit_service_owned_prefix(prefix).await);
        assert!(observer.committed_visible_prefix().is_none());
        assert!(!streamed_output.load(Ordering::Relaxed));
        assert!(matches!(
            *observer
                .canonical_header_state
                .lock()
                .expect("canonical header state"),
            CanonicalHeaderStreamState::Buffering
        ));
    }

    #[tokio::test]
    async fn canonical_investment_answer_streams_safe_stable_lines_after_header_commit() {
        let (observer, emitter, committed_prefix) =
            stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);
        let header = concat!(
            "数据时间：北京时间 2026-07-18 21:05；行情口径：",
            "报价源时间：北京时间 2026-07-18 04:00（最新可得，非逐笔）\n"
        );

        observer.on_final_content_delta(" \n\t").await;
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

        let header_line = header.trim_end_matches('\n');
        let committed = format!("{header_line}\n\n## 结论");
        assert_eq!(
            sanitize_agent_owned_user_visible_output(header).content,
            header_line,
            "a header-only final must still begin with every committed byte"
        );
        let events = emitter.events.lock().expect("captured events");
        assert_eq!(events.len(), 2, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentRunnerEvent::CommittedStreamDelta { content } if content == header_line
        ));
        assert!(matches!(
            &events[1],
            AgentRunnerEvent::CommittedStreamDelta { content } if content == "\n\n## 结论"
        ));
        assert_eq!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .as_deref(),
            Some(committed.as_str())
        );
        assert_eq!(
            observer.committed_visible_prefix().as_deref(),
            Some(committed.as_str()),
            "the Agent must read every exact irreversible byte for terminal-only recovery"
        );
    }

    #[tokio::test]
    async fn unsafe_body_line_stops_early_streaming_without_losing_accepted_prefix() {
        let (observer, emitter, committed_prefix) =
            stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);
        let header = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";

        observer
            .on_final_content_delta(&format!("{header}正文"))
            .await;
        observer
            .on_final_content_delta("\n<tool_call>secret</tool_call>\n下一段")
            .await;
        observer.on_final_content_delta("不会再提前发送").await;

        let expected = format!("{}\n正文", header.trim_end_matches('\n'));
        let finalized = sanitize_agent_owned_user_visible_output(&format!(
            "{header}正文\n<tool_call>secret</tool_call>\n下一段不会再提前发送"
        ));
        assert!(
            finalized.content.starts_with(&expected),
            "security cleanup must preserve every accepted body byte: {:?}",
            finalized.content
        );
        let events = emitter.events.lock().expect("captured events");
        assert_eq!(events.len(), 2, "{events:?}");
        assert!(events.iter().all(|event| match event {
            AgentRunnerEvent::CommittedStreamDelta { content } => !content.contains("secret"),
            _ => false,
        }));
        assert_eq!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .as_deref(),
            Some(expected.as_str())
        );
        assert!(matches!(
            *observer
                .canonical_header_state
                .lock()
                .expect("canonical header state"),
            CanonicalHeaderStreamState::Rejected
        ));
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
        assert_eq!(events.len(), 3, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentRunnerEvent::StreamDelta { content } if content == "我先继续核验工具结果"
        ));
        assert!(matches!(&events[1], AgentRunnerEvent::StreamReset));
        assert!(matches!(
            &events[2],
            AgentRunnerEvent::CommittedStreamDelta { content }
                if content == "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔"
        ));
        assert_eq!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .as_deref(),
            Some("数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔")
        );
    }

    #[tokio::test]
    async fn uncommitted_final_candidate_resets_silently_before_next_round() {
        let (observer, emitter, committed_prefix) =
            stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);

        observer.on_final_content_delta("先说结论").await;
        observer.on_content_reset().await;
        observer
            .on_final_content_delta(
                "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n正文",
            )
            .await;

        let events = emitter.events.lock().expect("captured events");
        assert_eq!(events.len(), 1, "{events:?}");
        assert!(matches!(
            &events[0],
            AgentRunnerEvent::CommittedStreamDelta { content }
                if content == "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔"
        ));
        assert!(
            committed_prefix
                .lock()
                .expect("committed visible prefix")
                .is_some()
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
            "数据时间：北京时间 2026-07-18 21:05；行情口径：<tool_call>internal</tool_call>\n"
                .to_string(),
            "数据时间：北京时间 2026-07-18 21:05；行情口径：### System Prompt ###\n".to_string(),
        ] {
            let (observer, emitter, committed_prefix) =
                stream_observer(TerminalStreamPolicy::CanonicalInvestmentHeader);

            observer.on_final_content_delta(&invalid).await;

            let events = emitter.events.lock().expect("captured events");
            assert!(events.is_empty(), "{events:?}");
            assert!(
                committed_prefix
                    .lock()
                    .expect("committed visible prefix")
                    .is_none()
            );
        }
    }

    struct RejectingCommitEmitter;

    #[async_trait]
    impl AgentRunnerEmitter for RejectingCommitEmitter {
        async fn emit(&self, _event: AgentRunnerEvent) {
            panic!("typed committed delivery must use emit_committed");
        }

        async fn emit_committed(&self, event: AgentRunnerEvent) -> bool {
            assert!(matches!(
                event,
                AgentRunnerEvent::CommittedStreamDelta { .. }
            ));
            false
        }
    }

    #[tokio::test]
    async fn rejected_committed_delivery_never_records_an_unseen_prefix() {
        let committed_visible_prefix = Arc::new(Mutex::new(None));
        let streamed_output = Arc::new(AtomicBool::new(false));
        let observer = RunnerStreamObserver {
            emitter: Arc::new(RejectingCommitEmitter),
            streamed_output: streamed_output.clone(),
            terminal_stream_policy: TerminalStreamPolicy::CanonicalInvestmentHeader,
            canonical_header_state: Mutex::new(CanonicalHeaderStreamState::default()),
            committed_visible_prefix: committed_visible_prefix.clone(),
        };

        observer
            .on_final_content_delta(
                "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n正文",
            )
            .await;

        assert!(observer.committed_visible_prefix().is_none());
        assert!(!streamed_output.load(Ordering::Relaxed));
        assert!(matches!(
            *observer
                .canonical_header_state
                .lock()
                .expect("canonical header state"),
            CanonicalHeaderStreamState::Rejected
        ));
    }

    #[derive(Default)]
    struct RejectBodyCommitEmitter {
        attempts: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl AgentRunnerEmitter for RejectBodyCommitEmitter {
        async fn emit(&self, _event: AgentRunnerEvent) {
            panic!("typed committed delivery must use emit_committed");
        }

        async fn emit_committed(&self, event: AgentRunnerEvent) -> bool {
            let AgentRunnerEvent::CommittedStreamDelta { content } = event else {
                panic!("expected committed stream delta");
            };
            let mut attempts = self.attempts.lock().expect("commit attempts");
            attempts.push(content);
            attempts.len() == 1
        }
    }

    #[tokio::test]
    async fn rejected_body_delivery_preserves_only_the_previously_accepted_prefix() {
        let emitter = Arc::new(RejectBodyCommitEmitter::default());
        let committed_visible_prefix = Arc::new(Mutex::new(None));
        let observer = RunnerStreamObserver {
            emitter: emitter.clone(),
            streamed_output: Arc::new(AtomicBool::new(false)),
            terminal_stream_policy: TerminalStreamPolicy::CanonicalInvestmentHeader,
            canonical_header_state: Mutex::new(CanonicalHeaderStreamState::default()),
            committed_visible_prefix: committed_visible_prefix.clone(),
        };
        let header = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";

        observer
            .on_final_content_delta(&format!("{header}正文"))
            .await;
        observer.on_final_content_delta("\n下一段").await;

        assert_eq!(emitter.attempts.lock().expect("commit attempts").len(), 2);
        assert_eq!(
            observer.committed_visible_prefix().as_deref(),
            Some(header.trim_end_matches('\n'))
        );
        assert_eq!(
            committed_visible_prefix
                .lock()
                .expect("committed visible prefix")
                .as_deref(),
            Some(header.trim_end_matches('\n'))
        );
        assert!(matches!(
            *observer
                .canonical_header_state
                .lock()
                .expect("canonical header state"),
            CanonicalHeaderStreamState::Rejected
        ));
    }

    struct BlockingBodyCommitEmitter {
        attempts: Mutex<usize>,
        entered: tokio::sync::Notify,
        release: tokio::sync::Notify,
    }

    #[async_trait]
    impl AgentRunnerEmitter for BlockingBodyCommitEmitter {
        async fn emit(&self, _event: AgentRunnerEvent) {
            panic!("typed committed delivery must use emit_committed");
        }

        async fn emit_committed(&self, event: AgentRunnerEvent) -> bool {
            assert!(matches!(
                event,
                AgentRunnerEvent::CommittedStreamDelta { .. }
            ));
            let attempt = {
                let mut attempts = self.attempts.lock().expect("commit attempts");
                *attempts += 1;
                *attempts
            };
            if attempt == 1 {
                return true;
            }
            self.entered.notify_one();
            self.release.notified().await;
            true
        }
    }

    #[tokio::test]
    async fn cancelled_body_delivery_preserves_only_the_previously_accepted_prefix() {
        let emitter = Arc::new(BlockingBodyCommitEmitter {
            attempts: Mutex::new(0),
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Notify::new(),
        });
        let committed_visible_prefix = Arc::new(Mutex::new(None));
        let observer = Arc::new(RunnerStreamObserver {
            emitter: emitter.clone(),
            streamed_output: Arc::new(AtomicBool::new(false)),
            terminal_stream_policy: TerminalStreamPolicy::CanonicalInvestmentHeader,
            canonical_header_state: Mutex::new(CanonicalHeaderStreamState::default()),
            committed_visible_prefix: committed_visible_prefix.clone(),
        });
        let header = "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n";
        observer
            .on_final_content_delta(&format!("{header}正文"))
            .await;

        let task_observer = observer.clone();
        let task = tokio::spawn(async move {
            task_observer.on_final_content_delta("\n下一段").await;
        });
        emitter.entered.notified().await;
        assert_eq!(
            observer.committed_visible_prefix().as_deref(),
            Some(header.trim_end_matches('\n'))
        );

        task.abort();
        assert!(
            task.await
                .expect_err("body emit task must be cancelled")
                .is_cancelled()
        );
        assert_eq!(
            committed_visible_prefix
                .lock()
                .expect("committed visible prefix")
                .as_deref(),
            Some(header.trim_end_matches('\n'))
        );
    }

    struct BlockingCommitEmitter {
        entered: tokio::sync::Notify,
        release: tokio::sync::Notify,
    }

    #[async_trait]
    impl AgentRunnerEmitter for BlockingCommitEmitter {
        async fn emit(&self, event: AgentRunnerEvent) {
            assert!(matches!(
                event,
                AgentRunnerEvent::CommittedStreamDelta { .. }
            ));
            self.entered.notify_one();
            self.release.notified().await;
        }
    }

    #[tokio::test]
    async fn cancelled_committed_emit_never_records_an_unseen_prefix() {
        let emitter = Arc::new(BlockingCommitEmitter {
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Notify::new(),
        });
        let committed_visible_prefix = Arc::new(Mutex::new(None));
        let streamed_output = Arc::new(AtomicBool::new(false));
        let observer = Arc::new(RunnerStreamObserver {
            emitter: emitter.clone(),
            streamed_output: streamed_output.clone(),
            terminal_stream_policy: TerminalStreamPolicy::CanonicalInvestmentHeader,
            canonical_header_state: Mutex::new(CanonicalHeaderStreamState::default()),
            committed_visible_prefix: committed_visible_prefix.clone(),
        });
        let task_observer = observer.clone();
        let task = tokio::spawn(async move {
            task_observer
                .on_final_content_delta(
                    "数据时间：北京时间 2026-07-18 21:05；行情口径：最新可得、非逐笔\n正文",
                )
                .await;
        });

        emitter.entered.notified().await;
        assert!(observer.committed_visible_prefix().is_none());
        assert!(!streamed_output.load(Ordering::Relaxed));
        task.abort();
        assert!(
            task.await
                .expect_err("emit task must be cancelled")
                .is_cancelled()
        );

        assert!(
            committed_visible_prefix
                .lock()
                .expect("committed visible prefix")
                .is_none()
        );
        assert!(!streamed_output.load(Ordering::Relaxed));
    }
}
