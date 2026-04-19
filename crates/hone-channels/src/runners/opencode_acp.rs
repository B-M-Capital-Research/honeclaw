use async_trait::async_trait;
use hone_core::agent::{
    AgentContext, AgentMessage, AgentResponse, final_assistant_message_content,
};
use hone_core::config::OpencodeAcpConfig;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::mcp_bridge::hone_mcp_servers;

use super::acp_common::{
    AcpEventLogContext, AcpPromptState, AcpResponseTimeouts, AcpToolCallRecord, create_acp_session,
    log_acp_payload, log_acp_prompt_stop_diagnostics, log_acp_raw_parse_error,
    set_acp_session_model, timeout_message_with_stderr, wait_for_response, write_jsonrpc_request,
};
use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    RunnerTimeouts,
};

const OPENCODE_ACP_SESSION_KEY: &str = "opencode_acp_session_id";

pub struct OpencodeAcpRunner {
    config: OpencodeAcpConfig,
    timeouts: RunnerTimeouts,
}

impl OpencodeAcpRunner {
    pub fn new(config: OpencodeAcpConfig, timeouts: RunnerTimeouts) -> Self {
        Self { config, timeouts }
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
        match run_opencode_acp(&self.config, self.timeouts, request, emitter.clone()).await {
            Ok((response, updates, context_messages)) => {
                metadata_updates.extend(updates);
                AgentRunnerResult {
                    response,
                    streamed_output: true,
                    terminal_error_emitted: false,
                    session_metadata_updates: metadata_updates,
                    context_messages,
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
                    context_messages: None,
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

fn is_executable_candidate(path: &Path) -> bool {
    path.is_file()
}

fn bundled_command_path_from_env(command: &str) -> Option<PathBuf> {
    if command != "opencode" {
        return None;
    }

    env::var_os("HONE_BUNDLED_OPENCODE_BIN")
        .map(PathBuf::from)
        .filter(|path| is_executable_candidate(path))
}

fn current_exe_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let Ok(current_exe) = env::current_exe() else {
        return dirs;
    };
    let Some(parent) = current_exe.parent() else {
        return dirs;
    };

    dirs.push(parent.to_path_buf());
    if parent.file_name().and_then(|value| value.to_str()) == Some("deps") {
        if let Some(grandparent) = parent.parent() {
            dirs.push(grandparent.to_path_buf());
        }
    }
    if cfg!(target_os = "macos")
        && parent.file_name().and_then(|value| value.to_str()) == Some("MacOS")
    {
        if let Some(contents) = parent.parent() {
            let resources = contents.join("Resources");
            dirs.push(resources.clone());
            dirs.push(resources.join("binaries"));
        }
    }

    dirs
}

fn bundled_command_names(command: &str) -> Vec<String> {
    let mut names = vec![command.to_string()];
    if let Some(triple) = current_target_triple() {
        names.push(format!("{command}-{triple}"));
    }
    if cfg!(windows) {
        let mut with_ext = Vec::with_capacity(names.len() * 2);
        for name in names {
            with_ext.push(format!("{name}.exe"));
            with_ext.push(name);
        }
        return with_ext;
    }
    names
}

fn current_target_triple() -> Option<String> {
    let arch = match env::consts::ARCH {
        "aarch64" => "aarch64",
        "x86_64" => "x86_64",
        "x86" => "i686",
        other => other,
    };
    let os = match env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-msvc",
        _ => return None,
    };
    Some(format!("{arch}-{os}"))
}

fn bundled_command_path_from_current_exe(command: &str) -> Option<PathBuf> {
    for dir in current_exe_search_dirs() {
        for name in bundled_command_names(command) {
            let candidate = dir.join(&name);
            if is_executable_candidate(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn default_command_search_dirs(home_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs = vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ];

    if let Some(home) = home_dir {
        dirs.push(home.join(".local").join("bin"));
        dirs.push(home.join(".cargo").join("bin"));
        dirs.push(home.join(".bun").join("bin"));
    }

    dirs
}

pub(crate) fn resolve_command_path_with_env(
    command: &str,
    path_env: Option<&std::ffi::OsStr>,
    home_dir: Option<&Path>,
) -> PathBuf {
    let command_path = Path::new(command);
    if command_path.components().count() > 1 || command_path.is_absolute() {
        return command_path.to_path_buf();
    }

    if let Some(bundled) = bundled_command_path_from_env(command) {
        return bundled;
    }

    if let Some(path_env) = path_env {
        for entry in env::split_paths(path_env) {
            let candidate = entry.join(command);
            if is_executable_candidate(&candidate) {
                return candidate;
            }
        }
    }

    if let Some(bundled) = bundled_command_path_from_current_exe(command) {
        return bundled;
    }

    for entry in default_command_search_dirs(home_dir) {
        let candidate = entry.join(command);
        if is_executable_candidate(&candidate) {
            return candidate;
        }
    }

    command_path.to_path_buf()
}

pub(crate) fn resolve_opencode_command_path(config: &OpencodeAcpConfig) -> PathBuf {
    resolve_command_path_with_env(
        &config.command,
        env::var_os("PATH").as_deref(),
        env::var_os("HOME").as_deref().map(Path::new),
    )
}

pub(crate) fn isolated_opencode_config(config: &OpencodeAcpConfig) -> String {
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
        }
    });

    let api_base_url = config.api_base_url.trim();
    if !api_base_url.is_empty() {
        payload["provider"] = serde_json::json!({
            "openrouter": {
                "options": {
                    "baseURL": api_base_url
                }
            }
        });
    }

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
    timeouts: RunnerTimeouts,
    request: AgentRunnerRequest,
    emitter: Arc<dyn AgentRunnerEmitter>,
) -> Result<
    (
        AgentResponse,
        HashMap<String, Value>,
        Option<Vec<AgentMessage>>,
    ),
    AgentSessionError,
> {
    let acp_log = AcpEventLogContext::from_request("opencode", &request);
    let startup_timeout = timeouts.step;
    let prompt_idle_timeout = timeouts.step;
    let prompt_overall_timeout = timeouts.overall;
    let model_timeout = timeouts.step;
    let mut metadata_updates = HashMap::new();
    let mcp_servers = hone_mcp_servers(&request).map_err(|message| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message,
    })?;
    let opencode_config_path = prepare_opencode_runtime(config, &request.working_directory)?;

    let injected_openrouter_api_key = if !config.api_key.trim().is_empty() {
        Some(config.api_key.trim())
    } else {
        config
            .openrouter_api_key
            .as_deref()
            .filter(|key| !key.trim().is_empty())
    };

    // ── 日志：API key 注入状态 ──────────────────────────────────────────────────
    let api_key_status = match injected_openrouter_api_key {
        Some(key) => {
            let preview = &key[..key.len().min(8)];
            format!("injecting OPENROUTER_API_KEY={preview}…")
        }
        _ => {
            "OPENROUTER_API_KEY not injected (will inherit local opencode auth/config)".to_string()
        }
    };
    let model_status = configured_opencode_model_id(config)
        .map(|m| format!("model={m}"))
        .unwrap_or_else(|| "model=<not set, using opencode default>".to_string());
    tracing::info!(
        "[AgentRunner/opencode] session={} {api_key_status} {model_status}",
        request.session_id,
    );

    let resolved_command = resolve_opencode_command_path(config);
    if resolved_command != PathBuf::from(&config.command) {
        tracing::info!(
            "[AgentRunner/opencode] session={} resolved command '{}' -> '{}'",
            request.session_id,
            config.command,
            resolved_command.display()
        );
    }

    let mut command = tokio::process::Command::new(&resolved_command);
    command
        .args(effective_opencode_args(config, &request.working_directory))
        .current_dir(&request.working_directory)
        .env("OPENCODE_CONFIG", &opencode_config_path)
        .env("OPENCODE_DISABLE_CLAUDE_CODE", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    // 通过环境变量传递 OpenRouter API Key（opencode 的 provider.openrouter 配置不支持 apiKey 字段）
    // 若 Hone 未显式注入，则继续使用用户本机 opencode 的 auth / provider 配置。
    if let Some(api_key) = injected_openrouter_api_key {
        command.env("OPENROUTER_API_KEY", api_key);
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
        Some(&acp_log),
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
            Some(&acp_log),
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
        Some(&acp_log),
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
            model_timeout,
            stderr_buf.clone(),
            Some(&acp_log),
        )
        .await?;
        next_id += 1;
    }

    tracing::info!(
        "[AgentRunner/opencode] session={} sending session/prompt (idle_timeout={}s overall_timeout={}s)",
        request.session_id,
        prompt_idle_timeout.as_secs(),
        prompt_overall_timeout.as_secs(),
    );
    let mut opencode_state = AcpPromptState::default();
    let prompt_text = build_opencode_acp_prompt_text(
        &request.system_prompt,
        &request.runtime_input,
        Some(&request.context),
    );
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
        Some(&acp_log),
    )
    .await?;
    let prompt_result = wait_for_opencode_response_with_timeouts(
        &mut reader,
        &mut stdin,
        next_id,
        emitter.clone(),
        &mut opencode_state,
        stderr_buf.clone(),
        AcpResponseTimeouts {
            idle: prompt_idle_timeout,
            overall: prompt_overall_timeout,
        },
        &acp_log,
    )
    .await?;

    let stop_reason = prompt_result
        .get("stopReason")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let success = stop_reason != "cancelled";
    if !success {
        log_acp_prompt_stop_diagnostics(
            "opencode",
            &request.session_id,
            stop_reason,
            &prompt_result,
            &opencode_state,
            &stderr_buf,
        )
        .await;
    }

    let _ = stdin.shutdown().await;
    let _ = child.kill().await;
    if let Some(task) = stderr_task {
        task.abort();
    }
    let context_messages = finalize_opencode_context_messages(&mut opencode_state);
    let content = final_assistant_message_content(
        &context_messages,
        std::mem::take(&mut opencode_state.full_reply),
    );
    let tool_calls_made = opencode_state.finished_tool_calls.clone();

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
        Some(context_messages),
    ))
}

pub(crate) fn build_opencode_acp_prompt_text(
    system_prompt: &str,
    runtime_input: &str,
    context: Option<&AgentContext>,
) -> String {
    let system = system_prompt.trim();
    let runtime = runtime_input.trim();
    let restored = context.and_then(serialize_context_for_opencode_prompt);

    let mut sections = Vec::new();
    if !system.is_empty() {
        sections.push(format!("### System Instructions ###\n{system}"));
    }
    if let Some(restored) = restored {
        sections.push(format!(
            "### Restored Conversation Transcript ###\n\
Use the following JSON transcript as the prior conversation context for this session.\n\
Messages are ordered from oldest to newest.\n\
```json\n{restored}\n```"
        ));
    }
    if !runtime.is_empty() {
        sections.push(format!("### User Input ###\n{runtime}"));
    }
    sections.join("\n\n")
}

pub(crate) fn serialize_context_for_opencode_prompt(context: &AgentContext) -> Option<String> {
    context.normalized_history_json()
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

fn finalize_opencode_context_messages(state: &mut AcpPromptState) -> Vec<AgentMessage> {
    flush_pending_assistant_message(state);
    state.context_messages.clone()
}

fn tool_call_id(update: &Value) -> Option<&str> {
    update.get("toolCallId").and_then(|value| value.as_str())
}

fn opencode_tool_name_from_start(update: &Value) -> String {
    update
        .get("title")
        .and_then(|value| value.as_str())
        .or_else(|| update.get("kind").and_then(|value| value.as_str()))
        .unwrap_or("tool")
        .to_string()
}

fn opencode_display_label(update: &Value, fallback_tool: &str) -> String {
    let raw_input = update.get("rawInput");
    let purpose_suffix = raw_input
        .and_then(|value| value.get("purpose"))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("；目的：{}", truncate_opencode_detail(value, 120)))
        .unwrap_or_default();

    let base = match fallback_tool {
        "read" => raw_input
            .and_then(|value| value.get("filePath"))
            .and_then(|value| value.as_str())
            .map(|path| format!("read {}", relativize_opencode_path(path)))
            .unwrap_or_else(|| "read".to_string()),
        "grep" => {
            let pattern = raw_input
                .and_then(|value| value.get("pattern"))
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let path = raw_input
                .and_then(|value| value.get("path"))
                .and_then(|value| value.as_str())
                .map(relativize_opencode_path);
            match (pattern, path) {
                (Some(pattern), Some(path)) => format!(
                    "grep \"{}\" in {}",
                    truncate_opencode_detail(pattern, 80),
                    truncate_opencode_detail(&path, 80)
                ),
                (Some(pattern), None) => {
                    format!("grep \"{}\"", truncate_opencode_detail(pattern, 80))
                }
                (None, Some(path)) => format!("grep in {}", truncate_opencode_detail(&path, 80)),
                (None, None) => "grep".to_string(),
            }
        }
        other => {
            if let Some(path) = raw_input
                .and_then(|value| value.get("path"))
                .and_then(|value| value.as_str())
            {
                format!("{other} {}", relativize_opencode_path(path))
            } else {
                other.to_string()
            }
        }
    };

    format!("{}{}", truncate_opencode_detail(&base, 96), purpose_suffix)
}

fn relativize_opencode_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return trimmed.to_string();
    }
    let marker = "/hone-agent-sandboxes/";
    if let Some(index) = trimmed.find(marker) {
        let tail = &trimmed[index + marker.len()..];
        let mut parts = tail.splitn(3, '/');
        let _channel = parts.next();
        let _actor = parts.next();
        if let Some(rest) = parts.next() {
            return if rest.is_empty() {
                "workspace root".to_string()
            } else {
                rest.to_string()
            };
        }
        return "workspace root".to_string();
    }
    trimmed.to_string()
}

fn truncate_opencode_detail(text: &str, max_chars: usize) -> String {
    let trimmed = text.trim();
    let total = trimmed.chars().count();
    if total <= max_chars {
        return trimmed.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let prefix = trimmed.chars().take(keep).collect::<String>();
    format!("{prefix}…")
}

fn opencode_tool_name_for_update(state: &AcpPromptState, update: &Value) -> String {
    if let Some(call_id) = tool_call_id(update) {
        if let Some(existing) = state.pending_tool_calls.get(call_id) {
            return existing.name.clone();
        }
    }
    opencode_tool_name_from_start(update)
}

fn is_meaningful_tool_value(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Object(map) => !map.is_empty(),
        Value::Array(values) => !values.is_empty(),
        Value::String(text) => !text.trim().is_empty(),
        _ => true,
    }
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

fn opencode_extract_tool_arguments(update: &Value) -> Value {
    update
        .get("rawInput")
        .cloned()
        .filter(is_meaningful_tool_value)
        .unwrap_or(Value::Null)
}

fn opencode_extract_text_from_content(update: &Value) -> Option<String> {
    let content = update.get("content")?.as_array()?;
    for item in content {
        let text = item
            .get("content")
            .and_then(|value| value.get("text"))
            .and_then(|value| value.as_str())
            .or_else(|| item.get("text").and_then(|value| value.as_str()));
        if let Some(text) = text {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn opencode_extract_tool_result(update: &Value) -> Option<Value> {
    if let Some(output) = update
        .get("rawOutput")
        .and_then(|value| value.get("output"))
        .cloned()
        .filter(is_meaningful_tool_value)
    {
        return Some(output);
    }
    if let Some(raw_output) = update
        .get("rawOutput")
        .cloned()
        .filter(is_meaningful_tool_value)
    {
        return Some(raw_output);
    }
    opencode_extract_text_from_content(update).map(Value::String)
}

fn opencode_extract_tool_failure(update: &Value) -> Option<Value> {
    if let Some(error) = update
        .get("rawOutput")
        .and_then(|value| value.get("error"))
        .and_then(|value| value.as_str())
    {
        let trimmed = error.trim();
        if !trimmed.is_empty() {
            return Some(json!({ "error": trimmed }));
        }
    }
    opencode_extract_text_from_content(update).map(|text| json!({ "error": text }))
}

fn upsert_pending_tool_arguments(state: &mut AcpPromptState, update: &Value, tool_name: &str) {
    let Some(call_id) = tool_call_id(update) else {
        return;
    };
    let arguments = opencode_extract_tool_arguments(update);
    if !is_meaningful_tool_value(&arguments) {
        return;
    }

    state
        .pending_tool_calls
        .entry(call_id.to_string())
        .and_modify(|record| {
            if !is_meaningful_tool_value(&record.arguments) {
                record.arguments = arguments.clone();
            }
        })
        .or_insert_with(|| AcpToolCallRecord {
            name: tool_name.to_string(),
            arguments: arguments.clone(),
        });

    if let Some(entry) = state
        .pending_assistant_tool_calls
        .iter_mut()
        .find(|value| value.get("id").and_then(|value| value.as_str()) == Some(call_id))
    {
        entry["function"]["arguments"] = Value::String(stringify_tool_arguments(&arguments));
    }
}

async fn handle_opencode_tool_call(
    update: &Value,
    emitter: &Arc<dyn AgentRunnerEmitter>,
    state: &mut AcpPromptState,
) {
    let Some(call_id) = tool_call_id(update) else {
        return;
    };
    let tool_name = opencode_tool_name_from_start(update);
    let arguments = opencode_extract_tool_arguments(update);
    state
        .pending_assistant_tool_calls
        .push(build_openai_tool_call_value(
            call_id, &tool_name, &arguments,
        ));
    state.pending_tool_calls.insert(
        call_id.to_string(),
        AcpToolCallRecord {
            name: tool_name.clone(),
            arguments,
        },
    );
    if is_meaningful_tool_value(&opencode_extract_tool_arguments(update)) {
        let display_label = opencode_display_label(update, &tool_name);
        emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: display_label.clone(),
                status: "start".to_string(),
                message: None,
                reasoning: Some(format!("正在执行：{display_label}")),
            })
            .await;
    }
}

async fn handle_opencode_tool_call_update(
    update: &Value,
    emitter: &Arc<dyn AgentRunnerEmitter>,
    state: &mut AcpPromptState,
) {
    let tool_name = opencode_tool_name_for_update(state, update);
    upsert_pending_tool_arguments(state, update, &tool_name);
    let display_label = opencode_display_label(update, &tool_name);

    let status = update
        .get("status")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if status == "in_progress" && is_meaningful_tool_value(&opencode_extract_tool_arguments(update))
    {
        emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: display_label.clone(),
                status: "start".to_string(),
                message: None,
                reasoning: Some(format!("正在执行：{display_label}")),
            })
            .await;
    }
    if status == "completed" || status == "failed" {
        let Some(call_id) = tool_call_id(update).map(|value| value.to_string()) else {
            return;
        };
        if state.completed_tool_call_ids.contains(&call_id) {
            return;
        }

        let pending = state.pending_tool_calls.remove(&call_id);
        let arguments = pending
            .as_ref()
            .map(|record| record.arguments.clone())
            .filter(is_meaningful_tool_value)
            .unwrap_or_else(|| opencode_extract_tool_arguments(update));

        let result = if status == "completed" {
            opencode_extract_tool_result(update).unwrap_or(Value::Null)
        } else {
            opencode_extract_tool_failure(update)
                .unwrap_or_else(|| json!({ "error": "tool failed" }))
        };

        state.completed_tool_call_ids.insert(call_id.clone());
        state
            .finished_tool_calls
            .push(hone_core::agent::ToolCallMade {
                name: tool_name.clone(),
                arguments,
                result: result.clone(),
                tool_call_id: Some(call_id.clone()),
            });
        flush_pending_assistant_message(state);
        state.context_messages.push(AgentMessage {
            role: "tool".to_string(),
            content: Some(stringify_tool_result(&result)),
            tool_calls: None,
            tool_call_id: Some(call_id),
            name: Some(tool_name.clone()),
            metadata: None,
        });
    }

    if status == "completed" {
        emitter
            .emit(AgentRunnerEvent::ToolStatus {
                tool: display_label.clone(),
                status: "done".to_string(),
                message: Some(format!("执行完成：{display_label}")),
                reasoning: None,
            })
            .await;
    } else if status == "failed" {
        emitter
            .emit(AgentRunnerEvent::Progress {
                stage: "opencode.tool_failed",
                detail: Some(format!("tool={tool_name}")),
            })
            .await;
    }
}

pub(crate) async fn handle_opencode_session_update(
    params: &Value,
    emitter: &Arc<dyn AgentRunnerEmitter>,
    state: &mut AcpPromptState,
) {
    let Some(update) = params.get("update") else {
        return;
    };
    let Some(kind) = update.get("sessionUpdate").and_then(|value| value.as_str()) else {
        return;
    };

    match kind {
        "agent_message_chunk" => {
            let text = update
                .get("content")
                .and_then(|value| value.get("text"))
                .and_then(|value| value.as_str())
                .or_else(|| update.get("text").and_then(|value| value.as_str()))
                .or_else(|| update.get("delta").and_then(|value| value.as_str()));
            let Some(text) = text else {
                return;
            };
            state.full_reply.push_str(text);
            state.pending_assistant_content.push_str(text);
            emitter
                .emit(AgentRunnerEvent::StreamDelta {
                    content: text.to_string(),
                })
                .await;
        }
        "agent_thought_chunk" => {
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
            handle_opencode_tool_call(update, emitter, state).await;
        }
        "tool_call_update" => {
            handle_opencode_tool_call_update(update, emitter, state).await;
        }
        "usage_update" => {
            if let Some(used) = update.get("used").and_then(|value| value.as_u64()) {
                emitter
                    .emit(AgentRunnerEvent::Progress {
                        stage: "opencode.usage",
                        detail: Some(format!("used={used}")),
                    })
                    .await;
            }
        }
        _ => {}
    }
}

async fn wait_for_opencode_response_with_timeouts(
    reader: &mut tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
    stdin: &mut tokio::process::ChildStdin,
    expected_id: u64,
    emitter: Arc<dyn AgentRunnerEmitter>,
    state: &mut AcpPromptState,
    stderr_buf: Arc<tokio::sync::Mutex<String>>,
    timeouts: AcpResponseTimeouts,
    log_ctx: &AcpEventLogContext,
) -> Result<Value, AgentSessionError> {
    let overall_deadline = tokio::time::Instant::now() + timeouts.overall;
    let mut idle_deadline = tokio::time::Instant::now() + timeouts.idle;

    loop {
        let now = tokio::time::Instant::now();
        if now >= overall_deadline {
            return Err(opencode_timeout_error("overall", timeouts.overall, &stderr_buf).await);
        }
        if now >= idle_deadline {
            return Err(opencode_timeout_error("idle", timeouts.idle, &stderr_buf).await);
        }

        let deadline = std::cmp::min(idle_deadline, overall_deadline);
        let line = match tokio::time::timeout_at(deadline, reader.next_line()).await {
            Ok(Ok(Some(line))) => line,
            Ok(Ok(None)) => {
                return Err(AgentSessionError {
                    kind: AgentSessionErrorKind::ExitFailure,
                    message: "opencode acp stream closed before response".to_string(),
                });
            }
            Ok(Err(e)) => {
                return Err(AgentSessionError {
                    kind: AgentSessionErrorKind::Io,
                    message: format!("failed to read opencode acp line: {e}"),
                });
            }
            Err(_) => {
                let timed_out_on_overall = tokio::time::Instant::now() >= overall_deadline;
                let (phase, duration) = if timed_out_on_overall {
                    ("overall", timeouts.overall)
                } else {
                    ("idle", timeouts.idle)
                };
                return Err(opencode_timeout_error(phase, duration, &stderr_buf).await);
            }
        };

        idle_deadline = tokio::time::Instant::now() + timeouts.idle;

        if let Some(result) = process_opencode_payload(
            stdin,
            expected_id,
            &line,
            &emitter,
            state,
            &stderr_buf,
            log_ctx,
        )
        .await?
        {
            return Ok(result);
        }
    }
}

async fn process_opencode_payload(
    stdin: &mut tokio::process::ChildStdin,
    expected_id: u64,
    line: &str,
    emitter: &Arc<dyn AgentRunnerEmitter>,
    state: &mut AcpPromptState,
    stderr_buf: &Arc<tokio::sync::Mutex<String>>,
    log_ctx: &AcpEventLogContext,
) -> Result<Option<Value>, AgentSessionError> {
    let payload: Value = match serde_json::from_str(line) {
        Ok(payload) => payload,
        Err(e) => {
            let message = format!("failed to parse opencode acp line: {e}");
            log_acp_raw_parse_error(Some(log_ctx), "recv", line, &message).await;
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::Io,
                message,
            });
        }
    };
    log_acp_payload(Some(log_ctx), "recv", &payload).await;

    if payload.get("id").and_then(|value| value.as_u64()) == Some(expected_id) {
        if let Some(error) = payload.get("error") {
            let message = error
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown acp error")
                .to_string();
            let stderr = stderr_buf.lock().await.clone();
            let stderr = if stderr.trim().is_empty() {
                String::new()
            } else {
                format!(" stderr={stderr}")
            };
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::AgentFailed,
                message: format!("opencode acp request failed: {message}{stderr}"),
            });
        }
        return Ok(Some(payload.get("result").cloned().unwrap_or(Value::Null)));
    }

    if let Some(method) = payload.get("method").and_then(|value| value.as_str()) {
        match method {
            "session/update" => {
                handle_opencode_session_update(
                    payload.get("params").unwrap_or(&Value::Null),
                    emitter,
                    state,
                )
                .await;
            }
            "session/request_permission" => {
                handle_opencode_permission_request(stdin, &payload, emitter, log_ctx).await?;
            }
            _ => {}
        }
    }

    Ok(None)
}

async fn opencode_timeout_error(
    phase: &'static str,
    duration: std::time::Duration,
    stderr_buf: &Arc<tokio::sync::Mutex<String>>,
) -> AgentSessionError {
    let base = format!(
        "opencode acp session/prompt {phase} timeout ({}s)",
        duration.as_secs()
    );
    let message = timeout_message_with_stderr(&base, stderr_buf).await;
    let kind = if phase == "idle" {
        AgentSessionErrorKind::TimeoutPerLine
    } else {
        AgentSessionErrorKind::TimeoutOverall
    };
    AgentSessionError { kind, message }
}

async fn handle_opencode_permission_request(
    stdin: &mut tokio::process::ChildStdin,
    payload: &Value,
    emitter: &Arc<dyn AgentRunnerEmitter>,
    log_ctx: &AcpEventLogContext,
) -> Result<(), AgentSessionError> {
    let request_id =
        payload
            .get("id")
            .and_then(|value| value.as_u64())
            .ok_or(AgentSessionError {
                kind: AgentSessionErrorKind::Io,
                message: "opencode acp permission request missing id".to_string(),
            })?;
    let params = payload.get("params").cloned().unwrap_or(Value::Null);
    let tool_title = params
        .get("toolCall")
        .and_then(|value| value.get("title"))
        .and_then(|value| value.as_str())
        .unwrap_or("permission")
        .to_string();

    emitter
        .emit(AgentRunnerEvent::Progress {
            stage: "acp.permission",
            detail: Some(format!("opencode:rejected:{tool_title}")),
        })
        .await;

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
    log_acp_payload(Some(log_ctx), "send", &response).await;
    let encoded = serde_json::to_string(&response).map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to encode opencode permission response: {e}"),
    })?;
    stdin
        .write_all(encoded.as_bytes())
        .await
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::Io,
            message: format!("failed to write opencode permission response: {e}"),
        })?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::Io,
            message: format!("failed to terminate opencode permission response: {e}"),
        })?;
    stdin.flush().await.map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: format!("failed to flush opencode permission response: {e}"),
    })?;
    Ok(())
}
