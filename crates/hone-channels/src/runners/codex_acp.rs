use async_trait::async_trait;
use hone_core::agent::{
    AgentContext, AgentMessage, AgentResponse, final_assistant_message_content,
};
use hone_core::config::CodexAcpConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::mcp_bridge::hone_mcp_servers;

use super::acp_common::{
    ACP_NEEDS_SP_RESEED_KEY, ACP_PREV_PROMPT_PEAK_KEY, AcpEventLogContext, AcpPermissionDecision,
    AcpPromptState, AcpRenderedToolStatus, AcpResponseTimeouts, AcpToolRenderPhase, CliVersion,
    acp_prompt_succeeded, build_acp_prompt_text, create_acp_session, finalize_context_messages,
    log_acp_prompt_stop_diagnostics, parse_cli_version, set_acp_session_model, wait_for_response,
    wait_for_response_with_timeouts_and_renderer, write_jsonrpc_request,
};
use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    RunnerTimeouts,
};

const CODEX_ACP_SESSION_KEY: &str = "codex_acp_session_id";
const MIN_CODEX_VERSION: CliVersion = CliVersion {
    major: 0,
    minor: 125,
    patch: 0,
};
const MIN_CODEX_ACP_VERSION: CliVersion = CliVersion {
    major: 0,
    minor: 12,
    patch: 0,
};

pub struct CodexAcpRunner {
    config: CodexAcpConfig,
    timeouts: RunnerTimeouts,
}

impl CodexAcpRunner {
    pub fn new(config: CodexAcpConfig, timeouts: RunnerTimeouts) -> Self {
        Self { config, timeouts }
    }
}

pub(crate) fn codex_acp_effective_args(config: &CodexAcpConfig, locked_down: bool) -> Vec<String> {
    let mut args = config.args.clone();
    let sandbox_mode = if locked_down {
        "workspace-write".to_string()
    } else {
        config.sandbox_mode.trim().to_string()
    };
    let approval_policy = if locked_down {
        "never".to_string()
    } else {
        config.approval_policy.trim().to_string()
    };

    if !sandbox_mode.is_empty() {
        args.push("-c".to_string());
        args.push(format!("sandbox_mode=\"{}\"", sandbox_mode));
    }

    if !approval_policy.is_empty() {
        args.push("-c".to_string());
        args.push(format!("approval_policy=\"{}\"", approval_policy));
    }

    if config.dangerously_bypass_approvals_and_sandbox && !locked_down {
        args.push("-c".to_string());
        args.push("sandbox_mode=\"danger-full-access\"".to_string());
        args.push("-c".to_string());
        args.push("approval_policy=\"never\"".to_string());
    }

    if !config.sandbox_permissions.is_empty() && !locked_down {
        let permissions = config
            .sandbox_permissions
            .iter()
            .map(|value| format!("\"{}\"", value))
            .collect::<Vec<_>>()
            .join(", ");
        args.push("-c".to_string());
        args.push(format!("sandbox_permissions=[{permissions}]"));
    }

    if let Some(effort) = configured_codex_reasoning_effort(config) {
        args.push("-c".to_string());
        args.push(format!("model_reasoning_effort=\"{}\"", effort));
    }

    for override_value in &config.extra_config_overrides {
        let trimmed = override_value.trim();
        if trimmed.is_empty() {
            continue;
        }
        args.push("-c".to_string());
        args.push(trimmed.to_string());
    }

    args
}

#[async_trait]
impl AgentRunner for CodexAcpRunner {
    fn name(&self) -> &'static str {
        "codex_acp"
    }

    fn manages_own_context(&self) -> bool {
        true
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        match run_codex_acp(&self.config, self.timeouts, request, emitter.clone()).await {
            Ok((response, updates, context_messages)) => AgentRunnerResult {
                response,
                streamed_output: true,
                terminal_error_emitted: false,
                session_metadata_updates: updates,
                context_messages,
            },
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

pub(crate) fn configured_codex_model_id(config: &CodexAcpConfig) -> Option<String> {
    let model = config.model.trim();
    if model.is_empty() {
        return None;
    }

    let variant = config.variant.trim();
    if !variant.is_empty() {
        let suffix = format!("/{variant}");
        if let Some(stripped) = model.strip_suffix(&suffix) {
            return Some(stripped.to_string());
        }
    }
    Some(model.to_string())
}

pub(crate) fn configured_codex_reasoning_effort(config: &CodexAcpConfig) -> Option<String> {
    let variant = config.variant.trim();
    if variant.is_empty() {
        None
    } else {
        Some(variant.to_string())
    }
}

pub(crate) fn validate_codex_version_matrix(
    codex_version: CliVersion,
    adapter_version: CliVersion,
) -> Result<(), String> {
    if codex_version < MIN_CODEX_VERSION {
        return Err(format!(
            "codex_acp requires codex >= {MIN_CODEX_VERSION}; found {codex_version}. Update with `npm install -g @openai/codex@latest`."
        ));
    }
    if adapter_version < MIN_CODEX_ACP_VERSION {
        return Err(format!(
            "codex_acp requires codex-acp >= {MIN_CODEX_ACP_VERSION}; found {adapter_version}. Update with `npm install -g @zed-industries/codex-acp@latest` or install the minimum validated version with `npm install -g @zed-industries/codex-acp@{MIN_CODEX_ACP_VERSION}`."
        ));
    }

    Ok(())
}

async fn validate_codex_acp_versions(
    config: &CodexAcpConfig,
    step_timeout: Duration,
) -> Result<(), AgentSessionError> {
    let codex_output = tokio::process::Command::new(&config.codex_command)
        .arg("--version")
        .output()
        .await
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::SpawnFailed,
            message: format!(
                "failed to probe codex version via `{}`: {e}",
                config.codex_command
            ),
        })?;
    let codex_text = String::from_utf8_lossy(&codex_output.stdout)
        .trim()
        .to_string();
    let codex_version = parse_cli_version(&codex_text).ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::AgentFailed,
        message: format!(
            "codex_acp requires a parseable `{} --version` output; got `{}`",
            config.codex_command, codex_text
        ),
    })?;
    validate_codex_version_matrix(
        codex_version,
        inspect_codex_acp_version(config, step_timeout).await?,
    )
    .map_err(|message| AgentSessionError {
        kind: AgentSessionErrorKind::AgentFailed,
        message,
    })
}

async fn inspect_codex_acp_version(
    config: &CodexAcpConfig,
    step_timeout: Duration,
) -> Result<CliVersion, AgentSessionError> {
    let mut command = tokio::process::Command::new(&config.command);
    command
        .args(codex_acp_effective_args(config, true))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());

    let mut child = command.spawn().map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message: format!("failed to spawn codex-acp for version probe: {e}"),
    })?;

    let mut stdin = child.stdin.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: "codex-acp version probe stdin unavailable".to_string(),
    })?;
    let stdout = child.stdout.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::StdoutUnavailable,
        message: "codex-acp version probe stdout unavailable".to_string(),
    })?;
    let mut reader = tokio::io::BufReader::new(stdout).lines();

    write_jsonrpc_request(
        &mut stdin,
        1,
        "initialize",
        serde_json::json!({
            "protocolVersion": 1,
            "clientCapabilities": {}
        }),
        None,
    )
    .await?;

    let result = tokio::time::timeout(
        step_timeout,
        wait_for_response("codex", &mut reader, &mut stdin, 1, None, None, None, None),
    )
    .await
    .map_err(|_| AgentSessionError {
        kind: AgentSessionErrorKind::TimeoutOverall,
        message: "codex-acp initialize timeout during version probe".to_string(),
    })??;

    let version_text = result
        .get("agentInfo")
        .and_then(|value| value.get("version"))
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();

    let _ = stdin.shutdown().await;
    let _ = child.kill().await;

    parse_cli_version(&version_text).ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::AgentFailed,
        message: format!(
            "codex-acp initialize returned an unparseable version: `{}`",
            version_text
        ),
    })
}

async fn run_codex_acp(
    config: &CodexAcpConfig,
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
    let acp_log = AcpEventLogContext::from_request("codex", &request);
    validate_codex_acp_versions(config, timeouts.step).await?;

    let startup_timeout = timeouts.step;
    let prompt_idle_timeout = timeouts.step;
    let prompt_overall_timeout = timeouts.overall;
    let model_timeout = timeouts.step;
    let mut metadata_updates = HashMap::new();
    let mcp_servers = hone_mcp_servers(&request).map_err(|message| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message,
    })?;

    let mut command = tokio::process::Command::new(&config.command);
    command
        .args(codex_acp_effective_args(config, true))
        .current_dir(&request.working_directory)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = command.spawn().map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message: format!("failed to spawn codex acp: {e}"),
    })?;

    let mut stdin = child.stdin.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: "codex acp stdin unavailable".to_string(),
    })?;
    let stdout = child.stdout.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::StdoutUnavailable,
        message: "codex acp stdout unavailable".to_string(),
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
            "codex",
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
        message: "codex acp initialize timeout".to_string(),
    })??;
    next_id += 1;

    let existing_session_id = request
        .session_metadata
        .get(CODEX_ACP_SESSION_KEY)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .filter(|value| !value.trim().is_empty());

    let (codex_session_id, seeded_from_local_context) =
        if let Some(session_id) = existing_session_id {
            write_jsonrpc_request(
                &mut stdin,
                next_id,
                "session/load",
                serde_json::json!({
                    "sessionId": session_id,
                    "cwd": request.working_directory,
                    "mcpServers": mcp_servers.clone(),
                }),
                Some(&acp_log),
            )
            .await?;
            match tokio::time::timeout(
                startup_timeout,
                wait_for_response(
                    "codex",
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
            {
                Ok(Ok(_)) => {
                    next_id += 1;
                    (session_id, false)
                }
                Ok(Err(err)) => {
                    tracing::warn!(
                        "[AgentRunner/codex] failed to load ACP session {}, creating new one: {}",
                        session_id,
                        err.message
                    );
                    let new_session_id = create_acp_session(
                        "codex",
                        &mut stdin,
                        &mut reader,
                        next_id + 1,
                        &request.working_directory,
                        mcp_servers.clone(),
                        startup_timeout,
                        stderr_buf.clone(),
                        Some(&acp_log),
                    )
                    .await?;
                    next_id += 2;
                    (new_session_id, true)
                }
                Err(_) => {
                    tracing::warn!(
                        "[AgentRunner/codex] ACP session/load timed out for {}, creating new one",
                        session_id
                    );
                    let new_session_id = create_acp_session(
                        "codex",
                        &mut stdin,
                        &mut reader,
                        next_id + 1,
                        &request.working_directory,
                        mcp_servers.clone(),
                        startup_timeout,
                        stderr_buf.clone(),
                        Some(&acp_log),
                    )
                    .await?;
                    next_id += 2;
                    (new_session_id, true)
                }
            }
        } else {
            let new_session_id = create_acp_session(
                "codex",
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
            (new_session_id, true)
        };

    metadata_updates.insert(
        CODEX_ACP_SESSION_KEY.to_string(),
        Value::String(codex_session_id.clone()),
    );

    if let Some(model_id) = configured_codex_model_id(config) {
        set_acp_session_model(
            "codex",
            &mut stdin,
            &mut reader,
            next_id,
            &codex_session_id,
            &model_id,
            model_timeout,
            stderr_buf.clone(),
            Some(&acp_log),
        )
        .await?;
        next_id += 1;
    }

    let mut codex_state = AcpPromptState {
        prev_prompt_peak_used: request
            .session_metadata
            .get(ACP_PREV_PROMPT_PEAK_KEY)
            .and_then(|value| value.as_u64()),
        ..AcpPromptState::default()
    };
    let prompt_text = if seeded_from_local_context {
        build_codex_acp_prompt_text(
            &request.system_prompt,
            &request.runtime_input,
            Some(&request.context),
        )
    } else {
        build_acp_prompt_text(&request.system_prompt, &request.runtime_input)
    };
    write_jsonrpc_request(
        &mut stdin,
        next_id,
        "session/prompt",
        serde_json::json!({
            "sessionId": codex_session_id,
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
    let prompt_result = wait_for_response_with_timeouts_and_renderer(
        "codex",
        &mut reader,
        &mut stdin,
        next_id,
        Some(emitter.clone()),
        Some(&mut codex_state),
        Some(stderr_buf.clone()),
        AcpResponseTimeouts {
            idle: prompt_idle_timeout,
            overall: prompt_overall_timeout,
        },
        Some(render_codex_tool_status),
        Some(patch_codex_session_update_params),
        AcpPermissionDecision::ApproveForSession,
        Some(&acp_log),
    )
    .await?;

    let stop_reason_value = prompt_result
        .get("stopReason")
        .and_then(|value| value.as_str());
    let success = acp_prompt_succeeded(stop_reason_value);
    let stop_reason = stop_reason_value.unwrap_or("unknown");
    if !success {
        log_acp_prompt_stop_diagnostics(
            "codex",
            &request.session_id,
            stop_reason,
            &prompt_result,
            &codex_state,
            &stderr_buf,
        )
        .await;
    }

    let _ = stdin.shutdown().await;
    let _ = child.kill().await;
    if let Some(task) = stderr_task {
        task.abort();
    }
    // ACP runner 内置 compact 状态写回 session_metadata：
    //  * 总是回写本轮 used 峰值，下一轮作为 used-drop 检测基线
    //  * 若本轮检测到 compact，置 acp_needs_sp_reseed=true，下一轮 prompt 构建层
    //    把完整 system_prompt 重新拼入 user message
    metadata_updates.insert(
        ACP_PREV_PROMPT_PEAK_KEY.to_string(),
        Value::from(codex_state.current_prompt_peak_used),
    );
    if codex_state.compact_detected {
        tracing::info!(
            "[AgentRunner/codex] session={} ACP compact detected (peak_used={}); marking next turn for SP reseed",
            request.session_id,
            codex_state.current_prompt_peak_used
        );
        metadata_updates.insert(ACP_NEEDS_SP_RESEED_KEY.to_string(), Value::Bool(true));
    } else {
        // 显式清掉上一轮可能残留的 reseed 标志，确保只在 reseed 完成后才清
        // —— 实际清理由 prompt 构建层负责（看到 true → 重塞 → 写 false）。
        // 这里不主动写 false，避免覆盖 prompt 构建层尚未消费的 true。
    }

    let context_messages = finalize_context_messages(&mut codex_state);
    let content = final_assistant_message_content(
        &context_messages,
        std::mem::take(&mut codex_state.full_reply),
    );
    let tool_calls_made = codex_state.finished_tool_calls.clone();

    Ok((
        AgentResponse {
            content,
            tool_calls_made,
            iterations: 1,
            success,
            error: if success {
                None
            } else {
                Some(format!(
                    "codex acp prompt stopped with reason={stop_reason}"
                ))
            },
        },
        metadata_updates,
        Some(context_messages),
    ))
}

pub(crate) fn render_codex_tool_status(
    update: &Value,
    phase: AcpToolRenderPhase,
    default_tool: &str,
    default_message: Option<String>,
    default_reasoning: Option<String>,
) -> AcpRenderedToolStatus {
    if !is_codex_execute_update(update) {
        return AcpRenderedToolStatus {
            tool: default_tool.to_string(),
            message: default_message,
            reasoning: default_reasoning,
        };
    }

    let rendered_command = render_codex_execute_command(update)
        .unwrap_or_else(|| truncate_codex_execute_label(default_tool));
    let purpose_suffix = codex_execute_purpose(update)
        .map(|purpose| format!("；目的：{}", truncate_codex_purpose(&purpose)))
        .unwrap_or_default();

    let (message, reasoning) = match phase {
        AcpToolRenderPhase::Start => (
            None,
            Some(format!("正在执行：{rendered_command}{purpose_suffix}")),
        ),
        AcpToolRenderPhase::Done => (Some(format!("执行完成：{rendered_command}")), None),
    };

    AcpRenderedToolStatus {
        tool: rendered_command,
        message: message.or(default_message),
        reasoning: reasoning.or(default_reasoning),
    }
}

fn is_codex_execute_update(update: &Value) -> bool {
    update
        .get("kind")
        .and_then(|value| value.as_str())
        .map(|value| value == "execute")
        .unwrap_or(false)
        || update
            .get("rawInput")
            .and_then(|value| value.get("command"))
            .and_then(|value| value.as_array())
            .is_some()
        || update
            .get("rawOutput")
            .and_then(|value| value.get("command"))
            .and_then(|value| value.as_array())
            .is_some()
}

pub(crate) fn patch_codex_session_update_params(params: &Value) -> Option<Value> {
    let update = params.get("update")?;
    let session_update = update
        .get("sessionUpdate")
        .and_then(|value| value.as_str())?;
    if session_update != "tool_call_update" || !is_codex_execute_update(update) {
        return None;
    }
    if update.get("output").is_some() || update.get("result").is_some() {
        return None;
    }

    let raw_output = update.get("rawOutput")?.clone();
    let mut patched = params.clone();
    patched
        .get_mut("update")
        .and_then(|value| value.as_object_mut())
        .map(|object| object.insert("output".to_string(), raw_output));
    Some(patched)
}

fn render_codex_execute_command(update: &Value) -> Option<String> {
    let command = update
        .get("rawInput")
        .and_then(|value| value.get("command"))
        .or_else(|| {
            update
                .get("rawOutput")
                .and_then(|value| value.get("command"))
        })
        .and_then(|value| value.as_array())?;
    let parts = command
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }

    let text = if parts.len() >= 3
        && matches!(
            parts[0],
            "/bin/zsh" | "zsh" | "/bin/bash" | "bash" | "/bin/sh" | "sh"
        )
        && parts[1] == "-lc"
    {
        parts[2].to_string()
    } else {
        parts.join(" ")
    };
    Some(truncate_codex_execute_label(&text))
}

fn codex_execute_purpose(update: &Value) -> Option<String> {
    update
        .get("rawInput")
        .and_then(|value| value.get("purpose"))
        .or_else(|| {
            update
                .get("rawOutput")
                .and_then(|value| value.get("purpose"))
        })
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn truncate_codex_execute_label(text: &str) -> String {
    const MAX_CHARS: usize = 96;
    let trimmed = text.trim();
    let total = trimmed.chars().count();
    if total <= MAX_CHARS {
        return trimmed.to_string();
    }
    let keep = 56.min(MAX_CHARS.saturating_sub(1));
    let prefix = trimmed.chars().take(keep).collect::<String>();
    format!("{prefix} [truncated, {total} chars]")
}

fn truncate_codex_purpose(text: &str) -> String {
    const MAX_CHARS: usize = 120;
    let trimmed = text.trim();
    let total = trimmed.chars().count();
    if total <= MAX_CHARS {
        return trimmed.to_string();
    }
    let prefix = trimmed.chars().take(80).collect::<String>();
    format!("{prefix} [truncated, {total} chars]")
}

pub(crate) fn build_codex_acp_prompt_text(
    system_prompt: &str,
    runtime_input: &str,
    context: Option<&AgentContext>,
) -> String {
    let system = system_prompt.trim();
    let runtime = runtime_input.trim();
    let restored = context.and_then(serialize_context_for_codex_prompt);

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

pub(crate) fn serialize_context_for_codex_prompt(context: &AgentContext) -> Option<String> {
    context.normalized_history_json()
}
