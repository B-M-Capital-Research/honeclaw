use chrono::Utc;
use hone_core::agent::{AgentMessage, ToolCallMade};
use regex::Regex;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{LazyLock, OnceLock};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::runtime::resolve_tool_reasoning;

use super::types::{AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest};

const ACP_EVENT_LOG_FILENAME: &str = "acp-events.log";

static ACP_EVENT_LOG_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

/// session_metadata 上记录的"上一轮 prompt 完成时 ACP runner 报告的 usage.used 峰值"。
/// 用作下一轮 compact 检测的基线（opencode 不推 compact 字面量，只能靠 used 骤降识别）。
pub(crate) const ACP_PREV_PROMPT_PEAK_KEY: &str = "acp_prev_prompt_peak_used";
/// session_metadata 上记录的"下一轮需要重新塞 system_prompt"标志。
/// 写入条件：本轮 ACP runner 报告了 compact 事件（codex 字面量 / opencode used drop）。
/// 消费方：prompt 构建层下一轮检查到 true 时，把完整 system_prompt 重新拼入 user message。
pub(crate) const ACP_NEEDS_SP_RESEED_KEY: &str = "acp_needs_sp_reseed";

/// codex-acp 在内置 compact 触发后推回的字面量 chunk（实测：
/// `agent_message_chunk text="Context compacted\n"`，单独一条）。
/// 同时也匹配 honeclaw 老 SessionCompactor 历史写入的 `Conversation compacted` 字符串。
static RE_ACP_COMPACT_STATUS_TEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^\s*(context|conversation)\s+compacted\.?\s*$").expect("valid regex")
});

/// opencode 在内置 compact 触发后会把"重启会话"的 markdown summary 拼到本轮 reply 后面，
/// 形如 `OK\n---\n## Goal\n...\n## Relevant Files\n- (none)\n---\nI don't have...`。
/// 我们用 `\n---\n## ` / `^---\n## ` 作为 compact 已发生的补充检测信号。
/// 注意：opencode 实测会把这段边界拆到多条 `agent_message_chunk` 里（如 `---\n` /
/// `## ` / ` Goal`），单条 chunk 上 regex 必然漏；因此 ingest 时必须在 **累积 buffer**
/// 上扫描。
static RE_OPENCODE_SUMMARY_BOUNDARY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)(^|\n)---\s*\n##\s+\w").expect("valid regex"));

/// 跨 chunk 扫 opencode summary boundary 时，从 buffer 尾部回看的窗口字节数。
/// 取 64 足以覆盖 `\n---\n## <heading>` 的最长合法变体（含 trailing 空白）。
const ACP_BOUNDARY_SCAN_TAIL_BYTES: usize = 64;

#[derive(Debug, Clone)]
pub(crate) struct AcpEventLogContext {
    pub(crate) runner_label: &'static str,
    pub(crate) log_path: PathBuf,
    pub(crate) session_id: String,
    pub(crate) identity: String,
    pub(crate) actor_channel: String,
    pub(crate) actor_user_id: String,
    pub(crate) actor_channel_scope: Option<String>,
}

impl AcpEventLogContext {
    pub(crate) fn from_request(runner_label: &'static str, request: &AgentRunnerRequest) -> Self {
        Self {
            runner_label,
            log_path: acp_event_log_path(&request.runtime_dir),
            session_id: request.session_id.clone(),
            identity: request.actor.session_id(),
            actor_channel: request.actor.channel.clone(),
            actor_user_id: request.actor.user_id.clone(),
            actor_channel_scope: request.actor.channel_scope.clone(),
        }
    }
}

pub(crate) fn acp_event_log_path(runtime_dir: &str) -> PathBuf {
    PathBuf::from(runtime_dir)
        .join("logs")
        .join(ACP_EVENT_LOG_FILENAME)
}

async fn append_acp_event_record(log_ctx: Option<&AcpEventLogContext>, record: Value) {
    let Some(log_ctx) = log_ctx else {
        return;
    };

    let Some(parent) = log_ctx.log_path.parent() else {
        return;
    };

    let _guard = ACP_EVENT_LOG_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await;

    if tokio::fs::create_dir_all(parent).await.is_err() {
        return;
    }

    let Ok(mut file) = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_ctx.log_path)
        .await
    else {
        return;
    };

    let Ok(mut encoded) = serde_json::to_vec(&record) else {
        return;
    };
    encoded.push(b'\n');
    let _ = file.write_all(&encoded).await;
    let _ = file.flush().await;
}

fn build_acp_event_record(
    log_ctx: &AcpEventLogContext,
    direction: &'static str,
    payload: Value,
) -> Value {
    let method = payload
        .get("method")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());
    let event_kind = if method.is_some() {
        "notification"
    } else if payload.get("id").is_some() {
        "response"
    } else {
        "message"
    };

    json!({
        "timestamp": Utc::now().to_rfc3339(),
        "runner": log_ctx.runner_label,
        "direction": direction,
        "event_kind": event_kind,
        "method": method,
        "session_id": log_ctx.session_id,
        "identity": log_ctx.identity,
        "actor_channel": log_ctx.actor_channel,
        "actor_user_id": log_ctx.actor_user_id,
        "actor_channel_scope": log_ctx.actor_channel_scope,
        "payload": payload,
    })
}

pub(crate) async fn log_acp_payload(
    log_ctx: Option<&AcpEventLogContext>,
    direction: &'static str,
    payload: &Value,
) {
    let Some(log_ctx) = log_ctx else {
        return;
    };
    append_acp_event_record(
        Some(log_ctx),
        build_acp_event_record(log_ctx, direction, payload.clone()),
    )
    .await;
}

pub(crate) async fn log_acp_raw_parse_error(
    log_ctx: Option<&AcpEventLogContext>,
    direction: &'static str,
    raw_line: &str,
    error: &str,
) {
    let Some(log_ctx) = log_ctx else {
        return;
    };
    append_acp_event_record(
        Some(log_ctx),
        json!({
            "timestamp": Utc::now().to_rfc3339(),
            "runner": log_ctx.runner_label,
            "direction": direction,
            "event_kind": "parse_error",
            "session_id": log_ctx.session_id,
            "identity": log_ctx.identity,
            "actor_channel": log_ctx.actor_channel,
            "actor_user_id": log_ctx.actor_user_id,
            "actor_channel_scope": log_ctx.actor_channel_scope,
            "error": error,
            "raw_line": raw_line,
        }),
    )
    .await;
}

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
    /// 入口由 runner 在 spawn 时从 session_metadata 读取
    /// `ACP_PREV_PROMPT_PEAK_KEY`，作为本轮 usage_update.used 骤降判定的基线。
    /// `None` 表示本 session 是第一次有 ACP 流，不做骤降判定。
    pub(crate) prev_prompt_peak_used: Option<u64>,
    /// 本轮 prompt 流中观测到的 usage.used 峰值，结束后 runner 写回 metadata。
    pub(crate) current_prompt_peak_used: u64,
    /// 本轮 prompt 流中是否检测到 ACP runner 触发了内置 compact。
    /// 触发源：codex 推 `Context compacted` 字面量 / opencode used 骤降 (>50%)。
    /// 检测后：runner 应在 metadata 写 `ACP_NEEDS_SP_RESEED_KEY=true`，下一轮重塞 SP。
    pub(crate) compact_detected: bool,
    /// 流中是否已经收到第一条 usage_update（用于"首次观测时与 prev_peak 比较"）。
    pub(crate) usage_update_seen: bool,
}

fn floor_char_boundary(text: &str, pos: usize) -> usize {
    let mut boundary = pos.min(text.len());
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
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
    log_ctx: Option<&AcpEventLogContext>,
) -> Result<String, AgentSessionError> {
    write_jsonrpc_request(
        stdin,
        request_id,
        "session/new",
        json!({
            "cwd": working_directory,
            "mcpServers": mcp_servers,
        }),
        log_ctx,
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
            log_ctx,
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
    log_ctx: Option<&AcpEventLogContext>,
) -> Result<(), AgentSessionError> {
    write_jsonrpc_request(
        stdin,
        request_id,
        "session/set_model",
        json!({
            "sessionId": session_id,
            "modelId": model_id,
        }),
        log_ctx,
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
            log_ctx,
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
    log_ctx: Option<&AcpEventLogContext>,
) -> Result<(), AgentSessionError> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    log_acp_payload(log_ctx, "send", &payload).await;
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
    log_ctx: Option<&AcpEventLogContext>,
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
            log_ctx,
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
    log_ctx: Option<&AcpEventLogContext>,
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
        log_ctx,
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
    log_ctx: Option<&AcpEventLogContext>,
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
            log_ctx,
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
    log_ctx: Option<&AcpEventLogContext>,
) -> Result<Option<Value>, AgentSessionError> {
    let payload: Value = match serde_json::from_str(line) {
        Ok(payload) => payload,
        Err(e) => {
            let message = format!("failed to parse {runner_label} acp line: {e}");
            log_acp_raw_parse_error(log_ctx, "recv", line, &message).await;
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::Io,
                message,
            });
        }
    };
    log_acp_payload(log_ctx, "recv", &payload).await;

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
                    log_ctx,
                )
                .await?;
            }
            _ => {}
        }
        return Ok(None);
    }

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

    Ok(None)
}

async fn handle_acp_permission_request(
    runner_label: &'static str,
    stdin: &mut tokio::process::ChildStdin,
    payload: &Value,
    emitter: Option<&std::sync::Arc<dyn AgentRunnerEmitter>>,
    decision: AcpPermissionDecision,
    log_ctx: Option<&AcpEventLogContext>,
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
    log_acp_payload(log_ctx, "send", &response).await;
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

pub(crate) fn acp_prompt_succeeded(stop_reason: Option<&str>) -> bool {
    matches!(stop_reason, Some(reason) if reason != "cancelled")
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

/// 处理一段 ACP `agent_message_chunk` 文本（codex / opencode / gemini 共用）。
///
/// compact 发生后的文本现在按用户可见内容透传，不再在 ACP ingest 层做裁剪。
/// 这里仍保留 compact 检测，用于 runner 在本轮结束时写回
/// `ACP_NEEDS_SP_RESEED_KEY`，保证下一轮 system prompt 能正确 reseed。
pub(crate) async fn ingest_acp_message_chunk(
    text: &str,
    state: &mut AcpPromptState,
    emitter: &std::sync::Arc<dyn AgentRunnerEmitter>,
) {
    if RE_ACP_COMPACT_STATUS_TEXT.is_match(text) {
        if !state.compact_detected {
            tracing::info!(
                "[acp] runner internal compact signalled via status text: {:?}",
                text
            );
            state.compact_detected = true;
        }
    }

    let pre_full_len = state.full_reply.len();
    state.full_reply.push_str(text);
    state.pending_assistant_content.push_str(text);

    let scan_start = floor_char_boundary(
        &state.full_reply,
        pre_full_len.saturating_sub(ACP_BOUNDARY_SCAN_TAIL_BYTES),
    );
    if !state.compact_detected
        && RE_OPENCODE_SUMMARY_BOUNDARY
            .find(&state.full_reply[scan_start..])
            .is_some()
    {
        tracing::info!("[acp] opencode compact summary boundary detected in accumulated buffer");
        state.compact_detected = true;
    }

    emitter
        .emit(AgentRunnerEvent::StreamDelta {
            content: text.to_string(),
        })
        .await;
}

/// 处理一条 ACP `usage_update`。共用 peak 跟踪 + "流内首次 used 与 prev_peak 比较"
/// 的 compact 骤降识别（覆盖 opencode 不发字面量的场景）。
///
/// `progress_stage` 由调用方决定（`"acp.usage"` / `"opencode.usage"` 等）以维持
/// 现有运营 metrics 兼容。
pub(crate) async fn ingest_acp_usage_update(
    used: u64,
    state: &mut AcpPromptState,
    emitter: &std::sync::Arc<dyn AgentRunnerEmitter>,
    progress_stage: &'static str,
) {
    emitter
        .emit(AgentRunnerEvent::Progress {
            stage: progress_stage,
            detail: Some(format!("used={used}")),
        })
        .await;

    state.current_prompt_peak_used = state.current_prompt_peak_used.max(used);
    let was_first = !state.usage_update_seen;
    state.usage_update_seen = true;

    if was_first
        && !state.compact_detected
        && let Some(prev_peak) = state.prev_prompt_peak_used
        && prev_peak >= 30_000
        && used * 2 < prev_peak
    {
        tracing::info!(
            "[acp] runner internal compact signalled via usage drop: prev_peak={} now_used={}",
            prev_peak,
            used
        );
        state.compact_detected = true;
    }
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

            if let Some(state) = state.as_deref_mut() {
                ingest_acp_message_chunk(text, state, emitter).await;
            } else {
                emitter
                    .emit(AgentRunnerEvent::StreamDelta {
                        content: text.to_string(),
                    })
                    .await;
            }
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
            if let Some(used) = update.get("used").and_then(|value| value.as_u64())
                && let Some(state) = state.as_deref_mut()
            {
                ingest_acp_usage_update(used, state, emitter, "acp.usage").await;
            } else if let Some(used) = update.get("used").and_then(|value| value.as_u64()) {
                // 无 state 上下文时只投 Progress（保持现状）
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

#[cfg(test)]
mod tests {
    use super::{
        AcpEventLogContext, AcpPermissionDecision, AcpPromptState, acp_event_log_path,
        acp_prompt_succeeded, handle_acp_session_update, ingest_acp_message_chunk, log_acp_payload,
        process_acp_payload,
    };
    use crate::runners::types::{AgentRunnerEmitter, AgentRunnerEvent};
    use async_trait::async_trait;
    use serde_json::json;
    use std::process::Stdio;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    struct CollectingEmitter {
        deltas: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl AgentRunnerEmitter for CollectingEmitter {
        async fn emit(&self, event: AgentRunnerEvent) {
            if let AgentRunnerEvent::StreamDelta { content } = event {
                self.deltas.lock().await.push(content);
            }
        }
    }

    fn collecting_emitter() -> (Arc<dyn AgentRunnerEmitter>, Arc<Mutex<Vec<String>>>) {
        let deltas = Arc::new(Mutex::new(Vec::new()));
        let emitter: Arc<dyn AgentRunnerEmitter> = Arc::new(CollectingEmitter {
            deltas: deltas.clone(),
        });
        (emitter, deltas)
    }

    /// codex-acp 内置 compact 触发后单独发一条 `agent_message_chunk text="Context compacted\n"`。
    /// 现在这类文本会继续透传给用户，但仍需在 state 上标记 compact_detected=true，
    /// 供 runner 写回 metadata 触发下一轮 SP reseed。
    #[tokio::test]
    async fn handle_acp_session_update_marks_codex_compact_literal_chunk_without_dropping_output() {
        let mut state = AcpPromptState::default();
        let (emitter, deltas) = collecting_emitter();
        let params = json!({
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "content": { "text": "Context compacted\n" }
            }
        });
        handle_acp_session_update(&params, &emitter, Some(&mut state)).await;
        assert!(state.compact_detected, "compact_detected should be set");
        assert!(
            state.full_reply == "Context compacted\n",
            "compact literal should stay in full_reply, got: {:?}",
            state.full_reply
        );
        assert!(
            deltas.lock().await.clone() == vec!["Context compacted\n".to_string()],
            "compact literal should be forwarded to sink"
        );
    }

    /// opencode 不推 compact 字面量，但内置 compact 后 session 体积 >50% 骤降。
    /// 我们以"流内首次 usage_update.used 与上轮 peak 比下降超 50%"作为信号，
    /// 同样置 compact_detected=true。
    #[tokio::test]
    async fn handle_acp_session_update_detects_opencode_compact_via_usage_drop() {
        // 模拟上一轮结束时 peak_used = 200_000
        let mut state = AcpPromptState {
            prev_prompt_peak_used: Some(200_000),
            ..AcpPromptState::default()
        };
        let (emitter, _) = collecting_emitter();

        // 本轮第一条 usage_update：used=10_000，远低于 prev_peak/2 (=100_000)
        let drop_event = json!({
            "update": {
                "sessionUpdate": "usage_update",
                "used": 10_000u64,
                "size": 256_000u64
            }
        });
        handle_acp_session_update(&drop_event, &emitter, Some(&mut state)).await;
        assert!(
            state.compact_detected,
            "usage drop from 200K to 10K should signal compact"
        );
        assert!(state.usage_update_seen);
        assert_eq!(state.current_prompt_peak_used, 10_000);
    }

    /// 平稳增长的 usage_update 不应误判为 compact（避免假阳性）。
    #[tokio::test]
    async fn handle_acp_session_update_no_compact_on_normal_usage_growth() {
        let mut state = AcpPromptState {
            prev_prompt_peak_used: Some(50_000),
            ..AcpPromptState::default()
        };
        let (emitter, _) = collecting_emitter();

        let event = json!({
            "update": {
                "sessionUpdate": "usage_update",
                "used": 60_000u64,
                "size": 256_000u64
            }
        });
        handle_acp_session_update(&event, &emitter, Some(&mut state)).await;
        assert!(
            !state.compact_detected,
            "monotonic growth should not trigger compact"
        );
    }

    /// opencode compact 后会把 markdown summary 拼到本轮真实回复后面，
    /// 如 `OK\n---\n## Goal\n...`。现在不再切掉这段用户可见文本，但仍需识别
    /// compact 已发生并打上 compact_detected。
    #[tokio::test]
    async fn handle_acp_session_update_keeps_opencode_summary_after_boundary() {
        let mut state = AcpPromptState::default();
        let (emitter, deltas) = collecting_emitter();

        let event = json!({
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "content": { "text": "OK\n---\n## Goal\n- placeholder\n## Constraints\n- none" }
            }
        });
        handle_acp_session_update(&event, &emitter, Some(&mut state)).await;
        assert_eq!(
            state.full_reply, "OK\n---\n## Goal\n- placeholder\n## Constraints\n- none",
            "summary should remain in full_reply"
        );
        let captured = deltas.lock().await.clone();
        assert_eq!(
            captured,
            vec!["OK\n---\n## Goal\n- placeholder\n## Constraints\n- none".to_string()]
        );
        assert!(state.compact_detected);
    }

    /// opencode 实测会把 `\n---\n## Goal` 边界拆到多个 `agent_message_chunk` 里
    /// （e.g. `"OK\n"`, `"---\n"`, `"## "`, `" Goal\n- placeholder"`）。
    /// 单 chunk 上 regex 必漏，必须扫累积 buffer。命中后只标记 compact_detected，
    /// 不再回截 full_reply。
    /// 注意：本场景 compact_detected **未**预置，早于 usage_update 收到 summary 是常态，
    /// 必须能直接从 chunk 流自身识别。
    #[tokio::test]
    async fn ingest_acp_message_chunk_detects_boundary_split_across_chunks_without_truncating() {
        let mut state = AcpPromptState::default();
        let (emitter, deltas) = collecting_emitter();

        for piece in ["OK\n", "---\n", "## ", " Goal\n- placeholder\n"] {
            ingest_acp_message_chunk(piece, &mut state, &emitter).await;
        }

        assert_eq!(
            state.full_reply, "OK\n---\n##  Goal\n- placeholder\n",
            "cross-chunk boundary should preserve the original streamed text"
        );
        assert_eq!(
            state.pending_assistant_content, "OK\n---\n##  Goal\n- placeholder\n",
            "pending_assistant_content should stay in lockstep with full_reply"
        );
        assert!(
            state.compact_detected,
            "boundary detection must mark compact_detected for SP reseed"
        );

        let captured = deltas.lock().await.clone();
        assert_eq!(
            captured,
            vec![
                "OK\n".to_string(),
                "---\n".to_string(),
                "## ".to_string(),
                " Goal\n- placeholder\n".to_string()
            ],
            "all streamed chunks should remain visible"
        );

        ingest_acp_message_chunk("## Constraints\n- none\n", &mut state, &emitter).await;
        assert_eq!(
            state.full_reply,
            "OK\n---\n##  Goal\n- placeholder\n## Constraints\n- none\n"
        );
        let captured_after = deltas.lock().await.clone();
        assert_eq!(
            captured_after.len(),
            5,
            "post-boundary chunk should still produce a new delta"
        );
    }

    /// 回归：scan_start 采用“按字节回看窗口”时，若 full_reply 前缀包含中文等多字节 UTF-8
    /// 字符，旧实现会把切片起点落到字符中间并 panic。
    #[tokio::test]
    async fn ingest_acp_message_chunk_handles_multibyte_prefix_when_scanning_boundary_window() {
        let mut state = AcpPromptState::default();
        let (emitter, deltas) = collecting_emitter();

        let prefix = "我".repeat(30);
        ingest_acp_message_chunk(&prefix, &mut state, &emitter).await;
        ingest_acp_message_chunk("\n---\n## Goal\n- placeholder\n", &mut state, &emitter).await;

        assert_eq!(
            state.full_reply,
            format!("{prefix}\n---\n## Goal\n- placeholder\n")
        );
        assert_eq!(
            state.pending_assistant_content,
            format!("{prefix}\n---\n## Goal\n- placeholder\n")
        );
        assert!(state.compact_detected);
        assert_eq!(
            deltas.lock().await.clone(),
            vec![prefix, "\n---\n## Goal\n- placeholder\n".to_string()]
        );
    }

    #[tokio::test]
    async fn acp_event_log_records_identity_for_grep() {
        let temp_root = std::env::temp_dir().join(format!(
            "hone_acp_log_{}_{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let ctx = AcpEventLogContext {
            runner_label: "codex",
            log_path: acp_event_log_path(&temp_root.to_string_lossy()),
            session_id: "session-1".to_string(),
            identity: "Actor_feishu__group-1__alice".to_string(),
            actor_channel: "feishu".to_string(),
            actor_user_id: "alice".to_string(),
            actor_channel_scope: Some("group-1".to_string()),
        };

        log_acp_payload(
            Some(&ctx),
            "recv",
            &json!({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": { "update": { "sessionUpdate": "usage_update", "used": 1 } }
            }),
        )
        .await;

        let content = tokio::fs::read_to_string(&ctx.log_path)
            .await
            .expect("read log");
        assert!(content.contains("\"identity\":\"Actor_feishu__group-1__alice\""));
        assert!(content.contains("\"method\":\"session/update\""));

        let _ = tokio::fs::remove_dir_all(&temp_root).await;
    }

    #[tokio::test]
    async fn acp_permission_request_matching_expected_id_is_not_prompt_response() {
        let mut child = tokio::process::Command::new("cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn cat");
        let mut stdin = child.stdin.take().expect("child stdin");
        let line = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "session/request_permission",
            "params": {
                "sessionId": "session-1",
                "options": [
                    {
                        "kind": "allow_always",
                        "name": "Allow for this session",
                        "optionId": "approved-for-session"
                    }
                ],
                "toolCall": {
                    "title": "Approve MCP tool call"
                }
            }
        })
        .to_string();

        let result = process_acp_payload(
            "codex",
            &mut stdin,
            4,
            &line,
            None,
            None,
            None,
            None,
            None,
            AcpPermissionDecision::ApproveForSession,
            None,
        )
        .await
        .expect("process permission request");

        assert!(result.is_none());
        drop(stdin);
        let output = child.wait_with_output().await.expect("cat output");
        let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
        assert!(stdout.contains("\"id\":4"));
        assert!(stdout.contains("\"optionId\":\"approved-for-session\""));
    }

    #[test]
    fn acp_prompt_success_requires_explicit_non_cancelled_stop_reason() {
        assert!(acp_prompt_succeeded(Some("end_turn")));
        assert!(acp_prompt_succeeded(Some("max_tokens")));
        assert!(!acp_prompt_succeeded(Some("cancelled")));
        assert!(!acp_prompt_succeeded(None));
    }
}
