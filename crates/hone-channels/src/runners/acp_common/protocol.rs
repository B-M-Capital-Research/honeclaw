//! ACP JSON-RPC 线上协议:请求/响应串行化、`session/new` / `session/set_model` /
//! `session/prompt` 的等待循环、`session/request_permission` 的自动决策、
//! idle/overall 超时判定、`build_acp_prompt_text` 系统提示拼装。
//!
//! 所有 ACP runner(codex / gemini / opencode)都走这里的 `wait_for_response*`
//! 入口;也就是说,`process_acp_payload` 是整个 ACP ingest 流的**唯一**驱动。
//! 保持这里是「拿到一行 stdout → 分发 → 回 Option<Value>」的简单形状,
//! 复杂的 tool/usage/summary 检测全部塞在 `super::ingest`。

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::AsyncWriteExt;

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::runners::types::AgentRunnerEmitter;

use super::ingest::handle_acp_session_update_with_renderer;
use super::log::{
    AcpEventLogContext, log_acp_payload, log_acp_raw_parse_error, timeout_message_with_stderr,
};
use super::state::{
    AcpPermissionDecision, AcpPromptState, AcpResponseTimeouts, AcpSessionUpdateTransformer,
    AcpToolStatusRenderer,
};

pub(crate) async fn create_acp_session(
    runner_label: &'static str,
    stdin: &mut tokio::process::ChildStdin,
    reader: &mut tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    request_id: u64,
    working_directory: &str,
    mcp_servers: Value,
    timeout: Duration,
    stderr_buf: Arc<tokio::sync::Mutex<String>>,
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
    stderr_buf: Arc<tokio::sync::Mutex<String>>,
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
    emitter: Option<Arc<dyn AgentRunnerEmitter>>,
    mut state: Option<&mut AcpPromptState>,
    stderr_buf: Option<Arc<tokio::sync::Mutex<String>>>,
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
    emitter: Option<Arc<dyn AgentRunnerEmitter>>,
    state: Option<&mut AcpPromptState>,
    stderr_buf: Option<Arc<tokio::sync::Mutex<String>>>,
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
    emitter: Option<Arc<dyn AgentRunnerEmitter>>,
    mut state: Option<&mut AcpPromptState>,
    stderr_buf: Option<Arc<tokio::sync::Mutex<String>>>,
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

pub(super) async fn process_acp_payload(
    runner_label: &'static str,
    stdin: &mut tokio::process::ChildStdin,
    expected_id: u64,
    line: &str,
    emitter: Option<&Arc<dyn AgentRunnerEmitter>>,
    mut state: Option<&mut AcpPromptState>,
    stderr_buf: Option<&Arc<tokio::sync::Mutex<String>>>,
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
    emitter: Option<&Arc<dyn AgentRunnerEmitter>>,
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
            .emit(crate::runners::types::AgentRunnerEvent::Progress {
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
    stderr_buf: Option<&Arc<tokio::sync::Mutex<String>>>,
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
