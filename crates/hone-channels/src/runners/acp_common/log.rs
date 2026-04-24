//! ACP 事件日志 + tracing 诊断格式化。
//!
//! 两份产物:
//! - **`acp-events.log`**:JSONL,每条写请求/响应/notification 原文 + 身份上下文。
//!   运维可以用 `grep '"identity":"Actor_…"'` 快速还原一条完整的 ACP session 流。
//! - **tracing warn**:在 prompt 超时 / stop 的时候打一条超紧凑的 summary,
//!   把 reply 长度、finished/pending tool count、stderr tail 压到一行。
//!
//! 文件级 append 用 `ACP_EVENT_LOG_LOCK` 保护,避免三个 ACP runner 并发写
//! 时互相踩到对方的行。

use chrono::Utc;
use hone_core::agent::ToolCallMade;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::io::AsyncWriteExt;

use super::state::AcpPromptState;
use crate::runners::types::AgentRunnerRequest;

const ACP_EVENT_LOG_FILENAME: &str = "acp-events.log";

static ACP_EVENT_LOG_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

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

// ── 格式化 helper ──────────────────────────────────────────────
// 这些是本 module 内部共享的纯字符串截断逻辑,外部不需要。

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
