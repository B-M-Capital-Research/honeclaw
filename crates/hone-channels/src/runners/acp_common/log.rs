//! ACP 事件日志 + tracing 诊断格式化。
//!
//! 两份产物:
//! - **`acp-events.log`**:JSONL,正常请求/响应/notification 写 payload + 身份上下文;
//!   parse error 只写脱敏后的 bounded preview,不保留完整 raw line。
//!   运维可以用 `grep '"identity":"Actor_…"'` 快速还原一条完整的 ACP session 流。
//! - **tracing warn**:在 prompt 超时 / stop 的时候打一条超紧凑的 summary,
//!   把 reply 长度、finished/pending tool count、stderr tail 压到一行。
//!
//! 文件级 append 用 `ACP_EVENT_LOG_LOCK` 保护,避免多个 ACP runner 并发写时
//! 互相踩到对方的行。

use chrono::Utc;
use hone_core::agent::ToolCallMade;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::io::AsyncWriteExt;

use super::state::AcpPromptState;
use crate::runners::types::AgentRunnerRequest;

const ACP_EVENT_LOG_FILENAME: &str = "acp-events.log";
const ACP_STDERR_DETAIL_CHARS: usize = 400;

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
            "raw_line_chars": raw_line.chars().count(),
            "raw_line_truncated": raw_line.chars().count() > ACP_STDERR_DETAIL_CHARS,
            "raw_line_preview": acp_diagnostic_excerpt_for_log(raw_line, ACP_STDERR_DETAIL_CHARS),
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
    stderr_buffer: &std::sync::Arc<tokio::sync::Mutex<String>>,
) {
    let stderr_captured = stderr_buffer.lock().await.clone();
    let stderr_tail = if stderr_captured.trim().is_empty() {
        "<empty>".to_string()
    } else {
        stderr_tail_for_log(&stderr_captured)
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
    stderr_buffer: &std::sync::Arc<tokio::sync::Mutex<String>>,
) -> String {
    message_with_bounded_stderr(base, stderr_buffer).await
}

pub(crate) async fn message_with_bounded_stderr(
    base: &str,
    stderr_buffer: &std::sync::Arc<tokio::sync::Mutex<String>>,
) -> String {
    let captured = stderr_buffer.lock().await.clone();
    let Some(stderr_detail) = stderr_detail_for_message(&captured) else {
        return base.to_string();
    };
    format!("{base} stderr={stderr_detail}")
}

fn stderr_detail_for_message(stderr: &str) -> Option<String> {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(acp_diagnostic_tail_for_log(
        trimmed,
        ACP_STDERR_DETAIL_CHARS,
    ))
}

pub(crate) fn acp_error_detail_for_message(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return "unknown acp error".to_string();
    }
    acp_diagnostic_excerpt_for_log(trimmed, ACP_STDERR_DETAIL_CHARS)
}

fn redact_common_stderr_secrets(text: &str) -> String {
    let mut output = redact_auth_scheme_tokens(text);
    for key in SENSITIVE_STDERR_MARKER_KEYS {
        output = redact_key_value(&output, key);
        output = redact_marker_value(&output, &format!("{key}:"));
        output = redact_json_string_field(&output, key);
    }
    for key in ["authorization", "Authorization"] {
        output = redact_json_string_field(&output, key);
    }
    output
}

const SENSITIVE_STDERR_MARKER_KEYS: &[&str] = &[
    "access_token",
    "accessToken",
    "api_key",
    "apiKey",
    "apikey",
    "app_secret",
    "appSecret",
    "client_secret",
    "clientSecret",
    "refresh_token",
    "refreshToken",
    "id_token",
    "idToken",
    "session_token",
    "sessionToken",
    "bot_token",
    "botToken",
    "OPENROUTER_API_KEY",
    "ANTHROPIC_API_KEY",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "TAVILY_API_KEY",
    "FMP_API_KEY",
    "HONE_CLOUD_API_KEY",
    "token",
    "secret",
    "password",
    "X-API-Key",
    "x-api-key",
];

fn redact_auth_scheme_tokens(text: &str) -> String {
    let output = redact_marker_value(text, "Bearer ");
    redact_marker_value(&output, "Basic ")
}

fn redact_key_value(text: &str, key: &str) -> String {
    redact_marker_value(text, &format!("{key}="))
}

fn redact_marker_value(text: &str, marker: &str) -> String {
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(marker) {
        let value_start = index + marker.len();
        output.push_str(&remaining[..value_start]);
        let leading_whitespace = remaining[value_start..]
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .map(char::len_utf8)
            .sum::<usize>();
        output.push_str(&remaining[value_start..value_start + leading_whitespace]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start + leading_whitespace..]
            .char_indices()
            .find_map(|(idx, ch)| {
                (ch == '&'
                    || ch == ')'
                    || ch == ','
                    || ch == '"'
                    || ch == '\''
                    || ch == '}'
                    || ch == ']'
                    || ch.is_whitespace())
                .then_some(idx)
            })
            .unwrap_or(remaining[value_start + leading_whitespace..].len());
        remaining = &remaining[value_start + leading_whitespace + value_tail..];
    }
    output.push_str(remaining);
    output
}

fn redact_json_string_field(text: &str, key: &str) -> String {
    let key_marker = format!("\"{key}\"");
    let mut remaining = text;
    let mut output = String::with_capacity(text.len());
    while let Some(index) = remaining.find(&key_marker) {
        let after_key = index + key_marker.len();
        let tail = &remaining[after_key..];
        let Some((colon_offset, _)) = tail.char_indices().find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !tail[colon_offset..].starts_with(':') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let after_colon = &tail[colon_offset + 1..];
        let Some((quote_offset, _)) = after_colon
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
        else {
            break;
        };
        if !after_colon[quote_offset..].starts_with('"') {
            output.push_str(&remaining[..after_key]);
            remaining = &remaining[after_key..];
            continue;
        }
        let value_start = after_key + colon_offset + 1 + quote_offset + 1;
        output.push_str(&remaining[..value_start]);
        output.push_str("<redacted>");
        let value_tail = remaining[value_start..]
            .char_indices()
            .find_map(|(idx, ch)| (ch == '"').then_some(idx))
            .unwrap_or(remaining[value_start..].len());
        remaining = &remaining[value_start + value_tail..];
    }
    output.push_str(remaining);
    output
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

fn stderr_tail_for_log(stderr: &str) -> String {
    acp_diagnostic_tail_for_log(stderr.trim(), ACP_STDERR_DETAIL_CHARS)
}

fn value_excerpt_for_log(value: &Value, max_chars: usize) -> String {
    let encoded = serde_json::to_string(value).unwrap_or_else(|_| value.to_string());
    acp_diagnostic_excerpt_for_log(&encoded, max_chars)
}

pub(crate) fn acp_diagnostic_excerpt_for_log(text: &str, max_chars: usize) -> String {
    truncate_for_log(&redact_common_stderr_secrets(text), max_chars)
}

fn acp_diagnostic_tail_for_log(text: &str, max_chars: usize) -> String {
    tail_for_log(&redact_common_stderr_secrets(text), max_chars)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn timeout_message_omits_empty_stderr() {
        let stderr = std::sync::Arc::new(tokio::sync::Mutex::new(" \n".to_string()));
        assert_eq!(
            timeout_message_with_stderr("codex acp request timed out", &stderr).await,
            "codex acp request timed out"
        );
    }

    #[tokio::test]
    async fn timeout_message_uses_bounded_stderr_tail() {
        let stderr = std::sync::Arc::new(tokio::sync::Mutex::new(format!(
            "prefix{}",
            "x".repeat(ACP_STDERR_DETAIL_CHARS + 10)
        )));
        let message = timeout_message_with_stderr("codex acp request timed out", &stderr).await;
        assert_eq!(
            message,
            format!(
                "codex acp request timed out stderr=…{}",
                "x".repeat(ACP_STDERR_DETAIL_CHARS)
            )
        );
    }

    #[tokio::test]
    async fn timeout_message_redacts_common_stderr_secrets() {
        let stderr = std::sync::Arc::new(tokio::sync::Mutex::new(
            r#"request failed https://api.test/path?api_key=abc&token=def auth=Bearer bearer-secret apiKey: header-secret {"secret":"json-secret"}"#
                .to_string(),
        ));
        let message = timeout_message_with_stderr("codex acp request timed out", &stderr).await;
        assert_eq!(
            message,
            r#"codex acp request timed out stderr=request failed https://api.test/path?api_key=<redacted>&token=<redacted> auth=Bearer <redacted> apiKey: <redacted> {"secret":"<redacted>"}"#
        );
    }

    #[test]
    fn stop_diagnostics_stderr_tail_redacts_common_secrets() {
        let tail = stderr_tail_for_log(
            r#"request failed token: header-secret auth=Bearer bearer-secret {"api_key":"json-secret"}"#,
        );

        assert!(tail.contains("token: <redacted>"));
        assert!(tail.contains("Bearer <redacted>"));
        assert!(tail.contains("\"api_key\":\"<redacted>\""));
        assert!(!tail.contains("header-secret"));
        assert!(!tail.contains("bearer-secret"));
        assert!(!tail.contains("json-secret"));
    }

    #[test]
    fn diagnostics_redact_extended_credential_shapes() {
        let detail = acp_diagnostic_excerpt_for_log(
            r#"OPENROUTER_API_KEY=env-secret X-API-Key: header-secret Authorization: Basic basic-secret {"client_secret":"json-client","authorization":"Basic json-basic","refresh_token":"json-refresh"}"#,
            500,
        );

        assert!(detail.contains("OPENROUTER_API_KEY=<redacted>"));
        assert!(detail.contains("X-API-Key: <redacted>"));
        assert!(detail.contains("Basic <redacted>"));
        assert!(detail.contains("\"client_secret\":\"<redacted>\""));
        assert!(detail.contains("\"authorization\":\"<redacted>\""));
        assert!(detail.contains("\"refresh_token\":\"<redacted>\""));
        assert!(!detail.contains("env-secret"));
        assert!(!detail.contains("header-secret"));
        assert!(!detail.contains("basic-secret"));
        assert!(!detail.contains("json-client"));
        assert!(!detail.contains("json-basic"));
        assert!(!detail.contains("json-refresh"));
    }

    #[tokio::test]
    async fn parse_error_log_records_bounded_redacted_raw_line_preview() {
        let temp_root = std::env::temp_dir().join(format!(
            "hone_acp_parse_error_log_{}_{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or_default()
        ));
        let log_context = AcpEventLogContext {
            runner_label: "codex",
            log_path: acp_event_log_path(&temp_root.to_string_lossy()),
            session_id: "session-1".to_string(),
            identity: "Actor_discord__direct__alice".to_string(),
            actor_channel: "discord".to_string(),
            actor_user_id: "alice".to_string(),
            actor_channel_scope: Some("direct".to_string()),
        };
        let raw_line = format!(
            "{}{}",
            r#"{"token":"json-secret","message":"auth=Bearer bearer-secret ","tail":""#,
            "x".repeat(ACP_STDERR_DETAIL_CHARS + 20)
        );

        log_acp_raw_parse_error(Some(&log_context), "recv", &raw_line, "parse failed").await;

        let content = tokio::fs::read_to_string(&log_context.log_path)
            .await
            .expect("read log");
        let record = serde_json::from_str::<Value>(&content).expect("jsonl record");
        assert_eq!(record["event_kind"], "parse_error");
        assert_eq!(record["raw_line_chars"], raw_line.chars().count());
        assert_eq!(record["raw_line_truncated"], true);
        let preview = record["raw_line_preview"].as_str().expect("preview");
        assert!(preview.contains("\"token\":\"<redacted>\""));
        assert!(preview.contains("Bearer <redacted>"));
        assert!(preview.ends_with('…'));
        assert!(preview.chars().count() <= ACP_STDERR_DETAIL_CHARS);
        assert!(!record.as_object().expect("record").contains_key("raw_line"));
        assert!(!preview.contains("json-secret"));
        assert!(!preview.contains("bearer-secret"));

        let _ = tokio::fs::remove_dir_all(&temp_root).await;
    }

    #[test]
    fn prompt_result_excerpt_redacts_common_secret_shapes() {
        let detail = value_excerpt_for_log(
            &json!({
                "error": "request failed token=plain-secret auth=Bearer bearer-secret",
                "api_key": "json-secret",
                "safe": "kept"
            }),
            500,
        );

        assert!(detail.contains("token=<redacted>"));
        assert!(detail.contains("Bearer <redacted>"));
        assert!(detail.contains("\"api_key\":\"<redacted>\""));
        assert!(detail.contains("\"safe\":\"kept\""));
        assert!(!detail.contains("plain-secret"));
        assert!(!detail.contains("bearer-secret"));
        assert!(!detail.contains("json-secret"));
    }

    #[test]
    fn acp_error_detail_redacts_and_bounds_message() {
        let detail = acp_error_detail_for_message(&format!(
            "failed token=plain-secret {}",
            "x".repeat(ACP_STDERR_DETAIL_CHARS + 20)
        ));

        assert!(detail.contains("token=<redacted>"));
        assert!(!detail.contains("plain-secret"));
        assert!(detail.ends_with('…'));
        assert!(detail.chars().count() <= ACP_STDERR_DETAIL_CHARS);
    }
}
