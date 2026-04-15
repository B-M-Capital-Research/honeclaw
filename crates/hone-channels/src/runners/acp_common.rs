use hone_core::agent::{AgentMessage, ToolCallMade};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::runtime::resolve_tool_reasoning;

use super::types::{AgentRunnerEmitter, AgentRunnerEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpToolRenderPhase {
    Start,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AcpPermissionDecision {
    RejectOnce,
    ApproveForSession,
}

impl AcpPermissionDecision {
    fn preferred_kind(self) -> &'static str {
        match self {
            Self::RejectOnce => "reject_once",
            Self::ApproveForSession => "allow_always",
        }
    }

    fn fallback_kind(self) -> &'static str {
        match self {
            Self::RejectOnce => "reject_once",
            Self::ApproveForSession => "allow_once",
        }
    }

    fn progress_label(self) -> &'static str {
        match self {
            Self::RejectOnce => "rejected",
            Self::ApproveForSession => "approved-for-session",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AcpRenderedToolStatus {
    pub(crate) tool: String,
    pub(crate) message: Option<String>,
    pub(crate) reasoning: Option<String>,
}

pub(crate) type AcpToolStatusRenderer = fn(
    update: &Value,
    phase: AcpToolRenderPhase,
    default_tool: &str,
    default_message: Option<String>,
    default_reasoning: Option<String>,
) -> AcpRenderedToolStatus;

pub(crate) type AcpSessionUpdateTransformer = fn(&Value) -> Option<Value>;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AcpResponseTimeouts {
    pub(crate) idle: Duration,
    pub(crate) overall: Duration,
}

#[derive(Default)]
pub(crate) struct AcpPromptState {
    pub(crate) full_reply: String,
    pub(crate) pending_tool_calls: HashMap<String, AcpToolCallRecord>,
    pub(crate) finished_tool_calls: Vec<ToolCallMade>,
    pub(crate) completed_tool_call_ids: HashSet<String>,
    pub(crate) context_messages: Vec<AgentMessage>,
    pub(crate) pending_assistant_content: String,
    pub(crate) pending_assistant_tool_calls: Vec<Value>,
}

#[derive(Debug, Clone)]
pub(crate) struct AcpToolCallRecord {
    pub(crate) name: String,
    pub(crate) arguments: Value,
}

fn extract_string_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(found) = value.get(*key).and_then(|value| value.as_str()) {
            let trimmed = found.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn extract_value_field<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    for key in keys {
        if let Some(found) = value.get(*key) {
            if !found.is_null() {
                return Some(found);
            }
        }
    }
    None
}

fn parse_embedded_json(value: &Value) -> Value {
    if let Some(text) = value.as_str() {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(trimmed).unwrap_or_else(|_| Value::String(trimmed.to_string()))
        }
    } else {
        value.clone()
    }
}

fn extract_tool_call_id(update: &Value) -> Option<String> {
    extract_string_field(update, &["toolCallId", "callId", "id"]).or_else(|| {
        update
            .get("toolCall")
            .and_then(|value| extract_string_field(value, &["id", "toolCallId", "callId"]))
    })
}

fn extract_tool_name(update: &Value) -> Option<String> {
    extract_string_field(update, &["title", "name", "toolName", "kind"]).or_else(|| {
        update
            .get("toolCall")
            .and_then(|value| extract_string_field(value, &["title", "name", "toolName", "kind"]))
    })
}

fn extract_tool_arguments(update: &Value) -> Value {
    if let Some(raw) = extract_value_field(update, &["arguments", "args", "input", "parameters"]) {
        return parse_embedded_json(raw);
    }
    if let Some(tool_call) = update.get("toolCall") {
        if let Some(raw) =
            extract_value_field(tool_call, &["arguments", "args", "input", "parameters"])
        {
            return parse_embedded_json(raw);
        }
    }
    Value::Null
}

fn extract_tool_result(update: &Value) -> Option<Value> {
    if let Some(raw) = extract_value_field(update, &["result", "output", "response"]) {
        return Some(parse_embedded_json(raw));
    }
    if let Some(content) = update.get("content") {
        if let Some(raw) = extract_value_field(content, &["result", "output", "text"]) {
            return Some(parse_embedded_json(raw));
        }
        if content.is_string() || content.is_array() {
            return Some(parse_embedded_json(content));
        }
    }
    update
        .get("toolCall")
        .and_then(|tool_call| extract_value_field(tool_call, &["result", "output", "response"]))
        .map(parse_embedded_json)
}

fn extract_tool_failure(update: &Value) -> Option<Value> {
    extract_string_field(update, &["message", "detail", "description", "subtitle"])
        .map(|message| json!({ "error": message }))
}

fn capture_tool_start(state: &mut AcpPromptState, update: &Value, fallback_name: &str) {
    let Some(tool_call_id) = extract_tool_call_id(update) else {
        return;
    };
    let tool_name = extract_tool_name(update).unwrap_or_else(|| fallback_name.to_string());
    let arguments = extract_tool_arguments(update);
    state
        .pending_assistant_tool_calls
        .push(build_openai_tool_call_value(
            &tool_call_id,
            &tool_name,
            &arguments,
        ));
    state.pending_tool_calls.insert(
        tool_call_id,
        AcpToolCallRecord {
            name: tool_name,
            arguments,
        },
    );
}

fn flush_pending_assistant_message(state: &mut AcpPromptState) {
    if state.pending_assistant_content.is_empty() && state.pending_assistant_tool_calls.is_empty() {
        return;
    }

    let content = std::mem::take(&mut state.pending_assistant_content);
    let tool_calls = if state.pending_assistant_tool_calls.is_empty() {
        None
    } else {
        Some(std::mem::take(&mut state.pending_assistant_tool_calls))
    };

    state.context_messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some(content),
        tool_calls,
        tool_call_id: None,
        name: None,
        metadata: None,
    });
}

fn capture_tool_finish(
    state: &mut AcpPromptState,
    update: &Value,
    fallback_name: &str,
    result: Value,
) {
    let Some(tool_call_id) = extract_tool_call_id(update) else {
        return;
    };
    if state.completed_tool_call_ids.contains(&tool_call_id) {
        return;
    }

    let pending = state.pending_tool_calls.remove(&tool_call_id);
    let tool_name = pending
        .as_ref()
        .map(|record| record.name.clone())
        .or_else(|| extract_tool_name(update))
        .unwrap_or_else(|| fallback_name.to_string());
    let arguments = pending
        .map(|record| record.arguments)
        .unwrap_or_else(|| extract_tool_arguments(update));

    state.completed_tool_call_ids.insert(tool_call_id.clone());
    state.finished_tool_calls.push(ToolCallMade {
        name: tool_name,
        arguments,
        result,
        tool_call_id: Some(tool_call_id),
    });
    flush_pending_assistant_message(state);
    state.context_messages.push(AgentMessage {
        role: "tool".to_string(),
        content: Some(stringify_tool_result(
            &state
                .finished_tool_calls
                .last()
                .map(|call| call.result.clone())
                .unwrap_or(Value::Null),
        )),
        tool_calls: None,
        tool_call_id: state
            .finished_tool_calls
            .last()
            .and_then(|call| call.tool_call_id.clone()),
        name: state
            .finished_tool_calls
            .last()
            .map(|call| call.name.clone()),
        metadata: None,
    });
}

pub(crate) fn extract_finished_tool_calls(state: AcpPromptState) -> Vec<ToolCallMade> {
    state.finished_tool_calls
}

pub(crate) fn finalize_context_messages(state: &mut AcpPromptState) -> Vec<AgentMessage> {
    flush_pending_assistant_message(state);
    state.context_messages.clone()
}

fn build_openai_tool_call_value(tool_call_id: &str, tool_name: &str, arguments: &Value) -> Value {
    json!({
        "id": tool_call_id,
        "type": "function",
        "function": {
            "name": tool_name,
            "arguments": stringify_tool_arguments(arguments),
        }
    })
}

fn stringify_tool_arguments(arguments: &Value) -> String {
    if let Some(text) = arguments.as_str() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    serde_json::to_string(arguments).unwrap_or_else(|_| "null".to_string())
}

fn stringify_tool_result(result: &Value) -> String {
    if let Some(text) = result.as_str() {
        return text.to_string();
    }
    serde_json::to_string(result).unwrap_or_else(|_| "null".to_string())
}

fn truncate_for_log(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let total = text.chars().count();
    if total <= max_chars {
        return text.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let truncated = text.chars().take(keep).collect::<String>();
    format!("{truncated}…")
}

fn tail_for_log(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars {
        return text.to_string();
    }
    let tail = chars[chars.len() - max_chars..].iter().collect::<String>();
    format!("…{tail}")
}

fn value_excerpt_for_log(value: &Value, max_chars: usize) -> String {
    let encoded = serde_json::to_string(value).unwrap_or_else(|_| value.to_string());
    truncate_for_log(&encoded, max_chars)
}

pub(crate) fn summarize_finished_tool_calls_for_log(calls: &[ToolCallMade]) -> String {
    if calls.is_empty() {
        return "none".to_string();
    }
    let entries = calls
        .iter()
        .rev()
        .take(3)
        .map(|call| {
            let call_id = call.tool_call_id.as_deref().unwrap_or("-");
            format!("{}#{call_id}", call.name)
        })
        .collect::<Vec<_>>();
    format!("count={} recent=[{}]", calls.len(), entries.join(", "))
}

fn summarize_pending_tool_calls_for_log(state: &AcpPromptState) -> String {
    if state.pending_tool_calls.is_empty() {
        return "none".to_string();
    }
    let mut entries = state
        .pending_tool_calls
        .iter()
        .map(|(call_id, record)| format!("{}#{call_id}", record.name))
        .collect::<Vec<_>>();
    entries.sort();
    let entries = entries.into_iter().take(3).collect::<Vec<_>>();
    format!(
        "count={} recent=[{}]",
        state.pending_tool_calls.len(),
        entries.join(", ")
    )
}

pub(crate) async fn log_acp_prompt_stop_diagnostics(
    runner_label: &'static str,
    session_id: &str,
    stop_reason: &str,
    prompt_result: &Value,
    state: &AcpPromptState,
    stderr_buf: &std::sync::Arc<tokio::sync::Mutex<String>>,
) {
    let stderr_captured = stderr_buf.lock().await.clone();
    let stderr_tail = if stderr_captured.trim().is_empty() {
        "<empty>".to_string()
    } else {
        tail_for_log(&stderr_captured, 400)
    };
    tracing::warn!(
        "[AgentRunner/{runner_label}] session={} stop_reason={} reply_chars={} prompt_result={} finished_tools={} pending_tools={} stderr_tail={}",
        session_id,
        stop_reason,
        state.full_reply.chars().count(),
        value_excerpt_for_log(prompt_result, 500),
        summarize_finished_tool_calls_for_log(&state.finished_tool_calls),
        summarize_pending_tool_calls_for_log(state),
        stderr_tail,
    );
}

pub(crate) async fn create_acp_session(
    runner_label: &'static str,
    stdin: &mut tokio::process::ChildStdin,
    reader: &mut tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    request_id: u64,
    working_directory: &str,
    mcp_servers: Value,
    timeout: Duration,
    stderr_buf: std::sync::Arc<tokio::sync::Mutex<String>>,
) -> Result<String, AgentSessionError> {
    write_jsonrpc_request(
        stdin,
        request_id,
        "session/new",
        json!({
            "cwd": working_directory,
            "mcpServers": mcp_servers,
        }),
    )
    .await?;
    let result = match tokio::time::timeout(
        timeout,
        wait_for_response(
            runner_label,
            reader,
            stdin,
            request_id,
            None,
            None,
            Some(stderr_buf.clone()),
        ),
    )
    .await
    {
        Ok(result) => result?,
        Err(_) => {
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::TimeoutOverall,
                message: timeout_message_with_stderr(
                    &format!("{runner_label} acp session/new timeout"),
                    &stderr_buf,
                )
                .await,
            });
        }
    };
    result
        .get("sessionId")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .filter(|value| !value.trim().is_empty())
        .ok_or(AgentSessionError {
            kind: AgentSessionErrorKind::AgentFailed,
            message: format!("{runner_label} acp session/new returned empty sessionId"),
        })
}

pub(crate) fn build_acp_prompt_text(system_prompt: &str, runtime_input: &str) -> String {
    let system = system_prompt.trim();
    let runtime = runtime_input.trim();
    match (system.is_empty(), runtime.is_empty()) {
        (true, _) => runtime.to_string(),
        (_, true) => format!("### System Instructions ###\n{system}"),
        (false, false) => {
            format!("### System Instructions ###\n{system}\n\n### User Input ###\n{runtime}")
        }
    }
}

pub(crate) async fn set_acp_session_model(
    runner_label: &'static str,
    stdin: &mut tokio::process::ChildStdin,
    reader: &mut tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    request_id: u64,
    session_id: &str,
    model_id: &str,
    timeout: Duration,
    stderr_buf: std::sync::Arc<tokio::sync::Mutex<String>>,
) -> Result<(), AgentSessionError> {
    write_jsonrpc_request(
        stdin,
        request_id,
        "session/set_model",
        json!({
            "sessionId": session_id,
            "modelId": model_id,
        }),
    )
    .await?;
    match tokio::time::timeout(
        timeout,
        wait_for_response(
            runner_label,
            reader,
            stdin,
            request_id,
            None,
            None,
            Some(stderr_buf.clone()),
        ),
    )
    .await
    {
        Ok(result) => result?,
        Err(_) => {
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::TimeoutOverall,
                message: timeout_message_with_stderr(
                    &format!("{runner_label} acp session/set_model timeout for {model_id}"),
                    &stderr_buf,
                )
                .await,
            });
        }
    };
    Ok(())
}

pub(crate) async fn timeout_message_with_stderr(
    base: &str,
    stderr_buf: &std::sync::Arc<tokio::sync::Mutex<String>>,
) -> String {
    let captured = stderr_buf.lock().await.clone();
    if captured.trim().is_empty() {
        base.to_string()
    } else {
        format!("{base} stderr={captured}")
    }
}

pub(crate) async fn write_jsonrpc_request(
    stdin: &mut tokio::process::ChildStdin,
    id: u64,
    method: &str,
    params: Value,
) -> Result<(), AgentSessionError> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    let encoded = serde_json::to_string(&payload).map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to encode acp request: {e}"),
    })?;
    stdin
        .write_all(encoded.as_bytes())
        .await
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::Io,
            message: format!("failed to write acp request: {e}"),
        })?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::Io,
            message: format!("failed to write acp newline: {e}"),
        })?;
    stdin.flush().await.map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to flush acp request: {e}"),
    })?;
    Ok(())
}

pub(crate) async fn wait_for_response(
    runner_label: &'static str,
    reader: &mut tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    stdin: &mut tokio::process::ChildStdin,
    expected_id: u64,
    emitter: Option<std::sync::Arc<dyn AgentRunnerEmitter>>,
    mut state: Option<&mut AcpPromptState>,
    stderr_buf: Option<std::sync::Arc<tokio::sync::Mutex<String>>>,
) -> Result<Value, AgentSessionError> {
    while let Ok(Some(line)) = reader.next_line().await {
        if let Some(result) = process_acp_payload(
            runner_label,
            stdin,
            expected_id,
            &line,
            emitter.as_ref(),
            state.as_deref_mut(),
            stderr_buf.as_ref(),
            None,
            None,
            AcpPermissionDecision::RejectOnce,
        )
        .await?
        {
            return Ok(result);
        }
    }

    Err(AgentSessionError {
        kind: AgentSessionErrorKind::ExitFailure,
        message: format!("{runner_label} acp stream closed before response"),
    })
}

pub(crate) async fn wait_for_response_with_timeouts(
    runner_label: &'static str,
    reader: &mut tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    stdin: &mut tokio::process::ChildStdin,
    expected_id: u64,
    emitter: Option<std::sync::Arc<dyn AgentRunnerEmitter>>,
    state: Option<&mut AcpPromptState>,
    stderr_buf: Option<std::sync::Arc<tokio::sync::Mutex<String>>>,
    timeouts: AcpResponseTimeouts,
) -> Result<Value, AgentSessionError> {
    wait_for_response_with_timeouts_and_renderer(
        runner_label,
        reader,
        stdin,
        expected_id,
        emitter,
        state,
        stderr_buf,
        timeouts,
        None,
        None,
        AcpPermissionDecision::RejectOnce,
    )
    .await
}

pub(crate) async fn wait_for_response_with_timeouts_and_renderer(
    runner_label: &'static str,
    reader: &mut tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    stdin: &mut tokio::process::ChildStdin,
    expected_id: u64,
    emitter: Option<std::sync::Arc<dyn AgentRunnerEmitter>>,
    mut state: Option<&mut AcpPromptState>,
    stderr_buf: Option<std::sync::Arc<tokio::sync::Mutex<String>>>,
    timeouts: AcpResponseTimeouts,
    tool_status_renderer: Option<AcpToolStatusRenderer>,
    session_update_transformer: Option<AcpSessionUpdateTransformer>,
    permission_decision: AcpPermissionDecision,
) -> Result<Value, AgentSessionError> {
    let start = tokio::time::Instant::now();
    let overall_deadline = start + timeouts.overall;
    let mut idle_deadline = start + timeouts.idle;

    loop {
        let now = tokio::time::Instant::now();
        if now >= overall_deadline {
            return Err(acp_timeout_error(
                runner_label,
                "overall",
                timeouts.overall,
                stderr_buf.as_ref(),
            )
            .await);
        }
        if now >= idle_deadline {
            return Err(acp_timeout_error(
                runner_label,
                "idle",
                timeouts.idle,
                stderr_buf.as_ref(),
            )
            .await);
        }

        let deadline = std::cmp::min(idle_deadline, overall_deadline);
        let line = match tokio::time::timeout_at(deadline, reader.next_line()).await {
            Ok(Ok(Some(line))) => line,
            Ok(Ok(None)) => {
                return Err(AgentSessionError {
                    kind: AgentSessionErrorKind::ExitFailure,
                    message: format!("{runner_label} acp stream closed before response"),
                });
            }
            Ok(Err(e)) => {
                return Err(AgentSessionError {
                    kind: AgentSessionErrorKind::Io,
                    message: format!("failed to read {runner_label} acp line: {e}"),
                });
            }
            Err(_) => {
                let timed_out_on_overall = tokio::time::Instant::now() >= overall_deadline;
                let (phase, duration) = if timed_out_on_overall {
                    ("overall", timeouts.overall)
                } else {
                    ("idle", timeouts.idle)
                };
                return Err(
                    acp_timeout_error(runner_label, phase, duration, stderr_buf.as_ref()).await,
                );
            }
        };

        idle_deadline = tokio::time::Instant::now() + timeouts.idle;

        if let Some(result) = process_acp_payload(
            runner_label,
            stdin,
            expected_id,
            &line,
            emitter.as_ref(),
            state.as_deref_mut(),
            stderr_buf.as_ref(),
            tool_status_renderer,
            session_update_transformer,
            permission_decision,
        )
        .await?
        {
            return Ok(result);
        }
    }
}

async fn process_acp_payload(
    runner_label: &'static str,
    stdin: &mut tokio::process::ChildStdin,
    expected_id: u64,
    line: &str,
    emitter: Option<&std::sync::Arc<dyn AgentRunnerEmitter>>,
    mut state: Option<&mut AcpPromptState>,
    stderr_buf: Option<&std::sync::Arc<tokio::sync::Mutex<String>>>,
    tool_status_renderer: Option<AcpToolStatusRenderer>,
    session_update_transformer: Option<AcpSessionUpdateTransformer>,
    permission_decision: AcpPermissionDecision,
) -> Result<Option<Value>, AgentSessionError> {
    let payload: Value = serde_json::from_str(line).map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to parse {runner_label} acp line: {e}"),
    })?;

    if payload.get("id").and_then(|value| value.as_u64()) == Some(expected_id) {
        if let Some(error) = payload.get("error") {
            let message = error
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown acp error")
                .to_string();
            let stderr = if let Some(buf) = stderr_buf {
                let captured = buf.lock().await.clone();
                if captured.trim().is_empty() {
                    String::new()
                } else {
                    format!(" stderr={captured}")
                }
            } else {
                String::new()
            };
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::AgentFailed,
                message: format!("{runner_label} acp request failed: {message}{stderr}"),
            });
        }
        return Ok(Some(payload.get("result").cloned().unwrap_or(Value::Null)));
    }

    if let Some(method) = payload.get("method").and_then(|value| value.as_str()) {
        match method {
            "session/update" => {
                if let Some(emitter) = emitter {
                    let transformed_params = session_update_transformer.and_then(|transformer| {
                        transformer(payload.get("params").unwrap_or(&Value::Null))
                    });
                    let params = transformed_params
                        .as_ref()
                        .unwrap_or_else(|| payload.get("params").unwrap_or(&Value::Null));
                    handle_acp_session_update_with_renderer(
                        params,
                        emitter,
                        state.as_deref_mut(),
                        tool_status_renderer,
                    )
                    .await;
                }
            }
            "session/request_permission" => {
                handle_acp_permission_request(
                    runner_label,
                    stdin,
                    &payload,
                    emitter,
                    permission_decision,
                )
                .await?;
            }
            _ => {}
        }
    }

    Ok(None)
}

async fn acp_timeout_error(
    runner_label: &'static str,
    phase: &'static str,
    duration: Duration,
    stderr_buf: Option<&std::sync::Arc<tokio::sync::Mutex<String>>>,
) -> AgentSessionError {
    let base = format!(
        "{runner_label} acp session/prompt {phase} timeout ({}s)",
        duration.as_secs()
    );
    let message = if let Some(buf) = stderr_buf {
        timeout_message_with_stderr(&base, buf).await
    } else {
        base
    };
    let kind = if phase == "idle" {
        AgentSessionErrorKind::TimeoutPerLine
    } else {
        AgentSessionErrorKind::TimeoutOverall
    };
    AgentSessionError { kind, message }
}

async fn handle_acp_permission_request(
    runner_label: &'static str,
    stdin: &mut tokio::process::ChildStdin,
    payload: &Value,
    emitter: Option<&std::sync::Arc<dyn AgentRunnerEmitter>>,
    decision: AcpPermissionDecision,
) -> Result<(), AgentSessionError> {
    let request_id =
        payload
            .get("id")
            .and_then(|value| value.as_u64())
            .ok_or(AgentSessionError {
                kind: AgentSessionErrorKind::Io,
                message: format!("{runner_label} acp permission request missing id"),
            })?;
    let params = payload.get("params").cloned().unwrap_or(Value::Null);
    let tool_title = params
        .get("toolCall")
        .and_then(|value| value.get("title"))
        .and_then(|value| value.as_str())
        .unwrap_or("permission")
        .to_string();

    if let Some(emitter) = emitter {
        emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "acp.permission",
                detail: Some(format!(
                    "{runner_label}:{}:{tool_title}",
                    decision.progress_label()
                )),
            })
            .await;
    }

    let option_id = params
        .get("options")
        .and_then(|value| value.as_array())
        .and_then(|options| {
            select_permission_option(
                options,
                &[decision.preferred_kind(), decision.fallback_kind()],
            )
        })
        .unwrap_or_else(|| match decision {
            AcpPermissionDecision::RejectOnce => "reject".to_string(),
            AcpPermissionDecision::ApproveForSession => "approved-for-session".to_string(),
        });

    let response = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "outcome": {
                "outcome": "selected",
                "optionId": option_id,
            }
        }
    });
    let encoded = serde_json::to_string(&response).map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to encode {runner_label} permission response: {e}"),
    })?;
    stdin
        .write_all(encoded.as_bytes())
        .await
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::Io,
            message: format!("failed to write {runner_label} permission response: {e}"),
        })?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::Io,
            message: format!("failed to terminate {runner_label} permission response: {e}"),
        })?;
    stdin.flush().await.map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to flush {runner_label} permission response: {e}"),
    })?;
    Ok(())
}

fn select_permission_option(options: &[Value], preferred_kinds: &[&str]) -> Option<String> {
    for preferred_kind in preferred_kinds {
        if let Some(option_id) = options.iter().find_map(|option| {
            let kind = option.get("kind").and_then(|value| value.as_str())?;
            if kind == *preferred_kind {
                option
                    .get("optionId")
                    .and_then(|value| value.as_str())
                    .map(|value| value.to_string())
            } else {
                None
            }
        }) {
            return Some(option_id);
        }
    }
    None
}

#[allow(dead_code)]
pub(crate) async fn handle_acp_session_update(
    params: &Value,
    emitter: &std::sync::Arc<dyn AgentRunnerEmitter>,
    state: Option<&mut AcpPromptState>,
) {
    handle_acp_session_update_with_renderer(params, emitter, state, None).await;
}

pub(crate) async fn handle_acp_session_update_with_renderer(
    params: &Value,
    emitter: &std::sync::Arc<dyn AgentRunnerEmitter>,
    mut state: Option<&mut AcpPromptState>,
    tool_status_renderer: Option<AcpToolStatusRenderer>,
) {
    let Some(update) = params.get("update") else {
        return;
    };
    let Some(kind) = update.get("sessionUpdate").and_then(|value| value.as_str()) else {
        return;
    };

    tracing::debug!("[acp] session/update kind={kind}");

    match kind {
        "agent_message_chunk" => {
            // Try nested content.text first (older protocol), then flat text/delta fields
            let text = update
                .get("content")
                .and_then(|value| value.get("text"))
                .and_then(|value| value.as_str())
                .or_else(|| update.get("text").and_then(|value| value.as_str()))
                .or_else(|| update.get("delta").and_then(|value| value.as_str()));
            let Some(text) = text else {
                tracing::debug!(
                    "[acp] agent_message_chunk: unrecognised payload format, skipping: {}",
                    update
                );
                return;
            };
            if let Some(state) = state {
                state.full_reply.push_str(text);
                state.pending_assistant_content.push_str(text);
            }
            emitter
                .emit(AgentRunnerEvent::StreamDelta {
                    content: text.to_string(),
                })
                .await;
        }
        "agent_thought_chunk" => {
            // Try nested content.text first, then flat text field
            let text = update
                .get("content")
                .and_then(|value| value.get("text"))
                .and_then(|value| value.as_str())
                .or_else(|| update.get("text").and_then(|value| value.as_str()));
            let Some(text) = text else {
                return;
            };
            emitter
                .emit(AgentRunnerEvent::StreamThought {
                    thought: text.to_string(),
                })
                .await;
        }
        "tool_call" => {
            let tool = update
                .get("title")
                .and_then(|value| value.as_str())
                .or_else(|| update.get("kind").and_then(|value| value.as_str()))
                .unwrap_or("tool")
                .to_string();
            if let Some(state) = state.as_deref_mut() {
                if !state.pending_assistant_content.is_empty() {
                    flush_pending_assistant_message(state);
                }
                capture_tool_start(state, update, &tool);
            }
            let default_reasoning = resolve_tool_reasoning(&tool, extract_acp_reasoning(update));
            let rendered = tool_status_renderer.map(|renderer| {
                renderer(
                    update,
                    AcpToolRenderPhase::Start,
                    &tool,
                    None,
                    default_reasoning.clone(),
                )
            });
            emitter
                .emit(AgentRunnerEvent::ToolStatus {
                    tool: rendered
                        .as_ref()
                        .map(|value| value.tool.clone())
                        .unwrap_or_else(|| tool.clone()),
                    status: "start".to_string(),
                    message: rendered.as_ref().and_then(|value| value.message.clone()),
                    reasoning: rendered
                        .as_ref()
                        .and_then(|value| value.reasoning.clone())
                        .or(default_reasoning),
                })
                .await;
        }
        "tool_call_update" => {
            let tool = update
                .get("title")
                .and_then(|value| value.as_str())
                .or_else(|| update.get("kind").and_then(|value| value.as_str()))
                .unwrap_or("tool")
                .to_string();
            let status = update
                .get("status")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            if status == "completed" {
                if let Some(state) = state.as_deref_mut() {
                    if let Some(result) = extract_tool_result(update) {
                        capture_tool_finish(state, update, &tool, result);
                    }
                }
                let rendered = tool_status_renderer.map(|renderer| {
                    renderer(
                        update,
                        AcpToolRenderPhase::Done,
                        &tool,
                        Some("工具执行完成".to_string()),
                        None,
                    )
                });
                emitter
                    .emit(AgentRunnerEvent::ToolStatus {
                        tool: rendered
                            .as_ref()
                            .map(|value| value.tool.clone())
                            .unwrap_or_else(|| tool.clone()),
                        status: "done".to_string(),
                        message: rendered
                            .as_ref()
                            .and_then(|value| value.message.clone())
                            .or_else(|| Some("工具执行完成".to_string())),
                        reasoning: rendered.as_ref().and_then(|value| value.reasoning.clone()),
                    })
                    .await;
            } else if status == "failed" {
                if let Some(state) = state.as_deref_mut() {
                    if let Some(result) = extract_tool_failure(update) {
                        capture_tool_finish(state, update, &tool, result);
                    }
                }
                emitter
                    .emit(AgentRunnerEvent::Progress {
                        stage: "acp.tool_failed",
                        detail: Some(format!("tool={tool}")),
                    })
                    .await;
            }
        }
        "usage_update" => {
            if let Some(used) = update.get("used").and_then(|value| value.as_u64()) {
                emitter
                    .emit(AgentRunnerEvent::Progress {
                        stage: "acp.usage",
                        detail: Some(format!("used={used}")),
                    })
                    .await;
            }
        }
        _ => {}
    }
}

fn extract_acp_reasoning(update: &Value) -> Option<String> {
    for key in ["message", "text", "detail", "description", "subtitle"] {
        if let Some(value) = update.get(key).and_then(|value| value.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    update
        .get("content")
        .and_then(|value| value.get("text"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CliVersion {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
}

impl std::fmt::Display for CliVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub(crate) fn parse_cli_version(raw: &str) -> Option<CliVersion> {
    raw.split(|ch: char| !ch.is_ascii_digit() && ch != '.')
        .find_map(|segment| {
            let mut parts = segment.split('.');
            let major = parts.next()?.parse().ok()?;
            let minor = parts.next()?.parse().ok()?;
            let patch = parts.next()?.parse().ok()?;
            Some(CliVersion {
                major,
                minor,
                patch,
            })
        })
}
