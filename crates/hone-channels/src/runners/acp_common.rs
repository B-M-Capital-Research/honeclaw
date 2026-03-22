use hone_core::agent::ToolCallMade;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::runtime::resolve_tool_reasoning;

use super::types::{AgentRunnerEmitter, AgentRunnerEvent};

#[derive(Default)]
pub(crate) struct AcpPromptState {
    pub(crate) full_reply: String,
    pub(crate) pending_tool_calls: HashMap<String, AcpToolCallRecord>,
    pub(crate) finished_tool_calls: Vec<ToolCallMade>,
    pub(crate) completed_tool_call_ids: HashSet<String>,
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
    state.pending_tool_calls.insert(
        tool_call_id,
        AcpToolCallRecord {
            name: tool_name,
            arguments,
        },
    );
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
}

pub(crate) fn extract_finished_tool_calls(state: AcpPromptState) -> Vec<ToolCallMade> {
    state.finished_tool_calls
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
        let payload: Value = serde_json::from_str(&line).map_err(|e| AgentSessionError {
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
            return Ok(payload.get("result").cloned().unwrap_or(Value::Null));
        }

        if let Some(method) = payload.get("method").and_then(|value| value.as_str()) {
            match method {
                "session/update" => {
                    if let Some(emitter) = emitter.as_ref() {
                        handle_acp_session_update(
                            payload.get("params").unwrap_or(&Value::Null),
                            emitter,
                            state.as_deref_mut(),
                        )
                        .await;
                    }
                }
                "session/request_permission" => {
                    handle_acp_permission_request(runner_label, stdin, &payload, emitter.as_ref())
                        .await?;
                }
                _ => {}
            }
        }
    }

    Err(AgentSessionError {
        kind: AgentSessionErrorKind::ExitFailure,
        message: format!("{runner_label} acp stream closed before response"),
    })
}

async fn handle_acp_permission_request(
    runner_label: &'static str,
    stdin: &mut tokio::process::ChildStdin,
    payload: &Value,
    emitter: Option<&std::sync::Arc<dyn AgentRunnerEmitter>>,
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
                detail: Some(format!("{runner_label}:rejected:{tool_title}")),
            })
            .await;
    }

    let reject_option = params
        .get("options")
        .and_then(|value| value.as_array())
        .and_then(|options| {
            options.iter().find_map(|option| {
                let kind = option.get("kind").and_then(|value| value.as_str())?;
                if kind == "reject_once" {
                    option
                        .get("optionId")
                        .and_then(|value| value.as_str())
                        .map(|value| value.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| "reject".to_string());

    let response = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "result": {
            "outcome": {
                "outcome": "selected",
                "optionId": reject_option,
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

fn progress_stage_for(runner_label: &'static str, suffix: &'static str) -> &'static str {
    match (runner_label, suffix) {
        ("opencode", "tool_failed") => "opencode.tool_failed",
        ("opencode", "usage") => "opencode.usage",
        ("codex", "tool_failed") => "codex.tool_failed",
        ("codex", "usage") => "codex.usage",
        ("gemini", "tool_failed") => "gemini_acp.tool_failed",
        ("gemini", "usage") => "gemini_acp.usage",
        _ => "acp.progress",
    }
}

pub(crate) async fn handle_acp_session_update(
    params: &Value,
    emitter: &std::sync::Arc<dyn AgentRunnerEmitter>,
    mut state: Option<&mut AcpPromptState>,
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
                capture_tool_start(state, update, &tool);
            }
            emitter
                .emit(AgentRunnerEvent::ToolStatus {
                    tool: tool.clone(),
                    status: "start".to_string(),
                    message: None,
                    reasoning: resolve_tool_reasoning(&tool, extract_acp_reasoning(update)),
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
                emitter
                    .emit(AgentRunnerEvent::ToolStatus {
                        tool,
                        status: "done".to_string(),
                        message: Some("工具执行完成".to_string()),
                        reasoning: None,
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
                        stage: progress_stage_for("opencode", "tool_failed"),
                        detail: Some(format!("tool={tool}")),
                    })
                    .await;
            }
        }
        "usage_update" => {
            if let Some(used) = update.get("used").and_then(|value| value.as_u64()) {
                emitter
                    .emit(AgentRunnerEvent::Progress {
                        stage: progress_stage_for("opencode", "usage"),
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
