use async_trait::async_trait;
use hone_core::agent::AgentResponse;
use hone_core::config::OpencodeAcpConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::mcp_bridge::hone_mcp_servers;

use super::acp_common::{
    AcpPromptState, build_acp_prompt_text, create_acp_session, extract_finished_tool_calls,
    set_acp_session_model, wait_for_response, write_jsonrpc_request,
};
use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
};

const OPENCODE_ACP_SESSION_KEY: &str = "opencode_acp_session_id";

pub struct OpencodeAcpRunner {
    config: OpencodeAcpConfig,
}

impl OpencodeAcpRunner {
    pub fn new(config: OpencodeAcpConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl AgentRunner for OpencodeAcpRunner {
    fn name(&self) -> &'static str {
        "opencode_acp"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        let mut metadata_updates = HashMap::new();
        match run_opencode_acp(&self.config, request, emitter.clone()).await {
            Ok((response, updates)) => {
                metadata_updates.extend(updates);
                AgentRunnerResult {
                    response,
                    streamed_output: true,
                    terminal_error_emitted: false,
                    session_metadata_updates: metadata_updates,
                }
            }
            Err(error) => {
                let message = error.message.clone();
                emitter.emit(AgentRunnerEvent::Error { error }).await;
                AgentRunnerResult {
                    response: AgentResponse {
                        content: String::new(),
                        tool_calls_made: Vec::new(),
                        iterations: 1,
                        success: false,
                        error: Some(message),
                    },
                    streamed_output: true,
                    terminal_error_emitted: true,
                    session_metadata_updates: HashMap::new(),
                }
            }
        }
    }
}

/// 当 `api_base_url` 指向 OpenRouter 时，opencode 要求模型 ID 以 `openrouter/` 开头，
/// 否则 opencode 会将第一段斜杠前的字符串解析为原生 provider ID（如 `google`），
/// 导致 `ProviderModelNotFoundError`。
///
/// 用户可以按 OpenRouter 的标准写法配置模型（如 `google/gemini-3.1-pro-preview`），
/// 本函数会自动补齐前缀。已经带 `openrouter/` 前缀的模型不会被重复添加。

pub(crate) fn configured_opencode_model_id(config: &OpencodeAcpConfig) -> Option<String> {
    let model = config.model.trim();
    if model.is_empty() {
        return None;
    }

    // 只要 API Key 像是 OpenRouter 的，或者 URL 包含 openrouter，就强制补全前缀
    // 同时也支持用户手动带入前缀
    let is_openrouter = config.api_base_url.contains("openrouter.ai")
        || config
            .openrouter_api_key
            .as_ref()
            .map(|k| k.starts_with("sk-or-"))
            .unwrap_or(false)
        || config.api_key.starts_with("sk-or-");

    let model = if is_openrouter && !model.starts_with("openrouter/") {
        format!("openrouter/{model}")
    } else {
        model.to_string()
    };

    let variant = config.variant.trim();
    let final_model = if variant.is_empty() {
        model
    } else {
        let suffix = format!("/{variant}");
        if model.ends_with(&suffix) {
            model
        } else {
            format!("{model}/{variant}")
        }
    };

    tracing::info!(
        "[AgentRunner/opencode] configured_model_id: input_model='{}', base_url='{}', final_model='{}'",
        config.model,
        config.api_base_url,
        final_model
    );

    Some(final_model)
}

pub(crate) fn effective_opencode_args(
    config: &OpencodeAcpConfig,
    working_directory: &str,
) -> Vec<String> {
    let mut args = Vec::new();
    let mut iter = config.args.iter().peekable();

    while let Some(arg) = iter.next() {
        if arg == "--cwd" {
            let _ = iter.next();
            continue;
        }
        args.push(arg.clone());
    }

    args.push("--cwd".to_string());
    args.push(working_directory.to_string());
    args
}

pub(crate) fn isolated_opencode_config(config: &OpencodeAcpConfig) -> String {
    // 注意：opencode 的 provider.openrouter 不支持 apiKey 字段（会导致 ConfigInvalidError 崩溃）。
    // API Key 通过 spawn 时设置 OPENROUTER_API_KEY 环境变量传递。
    let _ = config; // config 在此函数中仅用于 model/variant，api_key 不写入 JSON
    let mut payload = serde_json::json!({
        "$schema": "https://opencode.ai/config.json",
        "permission": {
            "read": "allow",
            "list": "allow",
            "glob": "allow",
            "grep": "allow",
            "edit": "deny",
            "bash": "deny",
            "webfetch": "deny",
            "websearch": "deny",
            "skill": "deny",
            "external_directory": {
                "*": "deny"
            }
        },
        "provider": {
            "openrouter": {
                "options": {
                    "baseURL": config.api_base_url.clone()
                }
            }
        }
    });

    if let Some(model) = configured_opencode_model_id(config) {
        payload["model"] = Value::String(model.clone());
        payload["agent"] = serde_json::json!({
            "plan": {
                "model": model,
                "options": {},
                "permission": {}
            },
            "build": {
                "model": model,
                "options": {},
                "permission": {}
            }
        });
    }

    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

fn prepare_opencode_runtime(
    config: &OpencodeAcpConfig,
    working_directory: &str,
) -> Result<PathBuf, AgentSessionError> {
    let runtime_root = PathBuf::from(working_directory)
        .join("runtime")
        .join("opencode");
    let config_dir = runtime_root.join("config_home").join("opencode");
    fs::create_dir_all(&config_dir).map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to create opencode config dir: {e}"),
    })?;
    let config_path = config_dir.join("opencode.jsonc");
    fs::write(&config_path, isolated_opencode_config(config)).map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to write opencode config: {e}"),
    })?;
    Ok(config_path)
}

async fn run_opencode_acp(
    config: &OpencodeAcpConfig,
    request: AgentRunnerRequest,
    emitter: Arc<dyn AgentRunnerEmitter>,
) -> Result<(AgentResponse, HashMap<String, Value>), AgentSessionError> {
    let startup_timeout = Duration::from_secs(config.startup_timeout_seconds.max(1));
    let request_timeout = Duration::from_secs(config.request_timeout_seconds.max(1));
    let mut metadata_updates = HashMap::new();
    let mcp_servers = hone_mcp_servers(&request).map_err(|message| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message,
    })?;
    let opencode_config_path = prepare_opencode_runtime(config, &request.working_directory)?;
    let xdg_config_home = opencode_config_path
        .parent()
        .and_then(|path| path.parent())
        .ok_or(AgentSessionError {
            kind: AgentSessionErrorKind::Io,
            message: "failed to resolve opencode XDG config home".to_string(),
        })?;

    // ── 日志：API key 注入状态 ──────────────────────────────────────────────────
    let api_key_status = match &config.openrouter_api_key {
        Some(key) if !key.trim().is_empty() => {
            let preview = &key[..key.len().min(8)];
            format!("injecting OPENROUTER_API_KEY={preview}…")
        }
        _ => "OPENROUTER_API_KEY not injected (will use env or opencode defaults)".to_string(),
    };
    let model_status = configured_opencode_model_id(config)
        .map(|m| format!("model={m}"))
        .unwrap_or_else(|| "model=<not set, using opencode default>".to_string());
    tracing::info!(
        "[AgentRunner/opencode] session={} {api_key_status} {model_status}",
        request.session_id,
    );

    let mut command = tokio::process::Command::new(&config.command);
    command
        .args(effective_opencode_args(config, &request.working_directory))
        .current_dir(&request.working_directory)
        .env("OPENCODE_CONFIG", &opencode_config_path)
        .env("OPENCODE_DISABLE_CLAUDE_CODE", "1")
        .env("XDG_CONFIG_HOME", xdg_config_home)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    // 通过环境变量传递 OpenRouter API Key（opencode 的 provider.openrouter 配置不支持 apiKey 字段）
    // 若 config_runtime.yaml 中配置了 OpenRouter API key，会在此处生效并覆盖环境变量
    if let Some(ref api_key) = config.openrouter_api_key {
        let trimmed = api_key.trim();
        if !trimmed.is_empty() {
            command.env("OPENROUTER_API_KEY", trimmed);
        }
    }

    let mut child = command.spawn().map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message: format!("failed to spawn opencode acp: {e}"),
    })?;

    let mut stdin = child.stdin.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: "opencode acp stdin unavailable".to_string(),
    })?;
    let stdout = child.stdout.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::StdoutUnavailable,
        message: "opencode acp stdout unavailable".to_string(),
    })?;
    let stderr = child.stderr.take();

    let stderr_buf = Arc::new(tokio::sync::Mutex::new(String::new()));
    let stderr_task = stderr.map(|stderr| {
        let stderr_buf = stderr_buf.clone();
        tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut guard = stderr_buf.lock().await;
                if !guard.is_empty() {
                    guard.push('\n');
                }
                guard.push_str(&line);
            }
        })
    });

    let mut reader = tokio::io::BufReader::new(stdout).lines();
    let mut next_id = 1u64;

    write_jsonrpc_request(
        &mut stdin,
        next_id,
        "initialize",
        serde_json::json!({
            "protocolVersion": 1,
            "clientCapabilities": {}
        }),
    )
    .await?;
    let _ = tokio::time::timeout(
        startup_timeout,
        wait_for_response(
            "opencode",
            &mut reader,
            &mut stdin,
            next_id,
            None,
            None,
            Some(stderr_buf.clone()),
        ),
    )
    .await
    .map_err(|_| AgentSessionError {
        kind: AgentSessionErrorKind::TimeoutOverall,
        message: "opencode acp initialize timeout".to_string(),
    })??;
    next_id += 1;

    // 始终创建新的 opencode 会话，而不是复用旧会话。
    // 原因：如果复用 session (session/load)，opencode 会在 session/prompt 响应期间
    // 异步回放旧会话的所有 agent_message_chunk 事件，这些历史片段会混入当前流式输出，
    // 导致前端 assistant_delta 包含所有历史回复，最终造成消息重复显示。
    tracing::info!(
        "[AgentRunner/opencode] session={} creating fresh acp session",
        request.session_id,
    );
    let opencode_session_id = create_acp_session(
        "opencode",
        &mut stdin,
        &mut reader,
        next_id,
        &request.working_directory,
        mcp_servers.clone(),
        startup_timeout,
        stderr_buf.clone(),
    )
    .await?;
    next_id += 1;

    metadata_updates.insert(
        OPENCODE_ACP_SESSION_KEY.to_string(),
        Value::String(opencode_session_id.clone()),
    );
    tracing::info!(
        "[AgentRunner/opencode] session={} acp_session={opencode_session_id} ready",
        request.session_id,
    );

    if let Some(model_id) = configured_opencode_model_id(config) {
        tracing::info!(
            "[AgentRunner/opencode] session={} setting model to {model_id}",
            request.session_id,
        );
        set_acp_session_model(
            "opencode",
            &mut stdin,
            &mut reader,
            next_id,
            &opencode_session_id,
            &model_id,
            request_timeout,
            stderr_buf.clone(),
        )
        .await?;
        next_id += 1;
    }

    tracing::info!(
        "[AgentRunner/opencode] session={} sending session/prompt (timeout={}s)",
        request.session_id,
        request_timeout.as_secs(),
    );
    let mut opencode_state = AcpPromptState::default();
    let prompt_text = build_acp_prompt_text(&request.system_prompt, &request.runtime_input);
    write_jsonrpc_request(
        &mut stdin,
        next_id,
        "session/prompt",
        serde_json::json!({
            "sessionId": opencode_session_id,
            "prompt": [
                {
                    "type": "text",
                    "text": prompt_text,
                }
            ]
        }),
    )
    .await?;
    let prompt_result = tokio::time::timeout(
        request_timeout,
        wait_for_response(
            "opencode",
            &mut reader,
            &mut stdin,
            next_id,
            Some(emitter.clone()),
            Some(&mut opencode_state),
            Some(stderr_buf.clone()),
        ),
    )
    .await
    .map_err(|_| AgentSessionError {
        kind: AgentSessionErrorKind::TimeoutOverall,
        message: format!(
            "opencode acp session/prompt timeout ({}s)",
            request_timeout.as_secs()
        ),
    })??;

    let stop_reason = prompt_result
        .get("stopReason")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let success = stop_reason != "cancelled";

    let _ = stdin.shutdown().await;
    let _ = child.kill().await;
    if let Some(task) = stderr_task {
        task.abort();
    }
    let content = std::mem::take(&mut opencode_state.full_reply);
    let tool_calls_made = extract_finished_tool_calls(opencode_state);

    let reply_chars = content.len();
    tracing::info!(
        "[AgentRunner/opencode] session={} stop_reason={stop_reason} success={success} reply_chars={reply_chars}",
        request.session_id,
    );

    // 若回复为空且运行"成功"，打印 stderr 帮助诊断（鉴权失败、模型未找到等）
    if reply_chars == 0 {
        let stderr_captured = stderr_buf.lock().await.clone();
        if stderr_captured.trim().is_empty() {
            tracing::warn!(
                "[AgentRunner/opencode] session={} empty reply (stop_reason={stop_reason}), no stderr captured. \
                 Possible causes: API key not set, model not found, or ACP protocol mismatch.",
                request.session_id,
            );
        } else {
            tracing::warn!(
                "[AgentRunner/opencode] session={} empty reply (stop_reason={stop_reason}). \
                 opencode stderr:\n{stderr_captured}",
                request.session_id,
            );
        }
    }

    Ok((
        AgentResponse {
            content,
            tool_calls_made,
            iterations: 1,
            success,
            error: if success {
                None
            } else {
                Some(format!("opencode prompt stopped with reason={stop_reason}"))
            },
        },
        metadata_updates,
    ))
}
