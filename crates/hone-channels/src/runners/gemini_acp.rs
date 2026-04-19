use async_trait::async_trait;
use hone_core::agent::AgentResponse;
use hone_core::config::GeminiAcpConfig;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::mcp_bridge::hone_mcp_servers;

use super::acp_common::{
    AcpEventLogContext, AcpPromptState, AcpResponseTimeouts, CliVersion, build_acp_prompt_text,
    create_acp_session, extract_finished_tool_calls, log_acp_prompt_stop_diagnostics,
    parse_cli_version, wait_for_response, wait_for_response_with_timeouts, write_jsonrpc_request,
};
use super::types::{
    AgentRunner, AgentRunnerEmitter, AgentRunnerEvent, AgentRunnerRequest, AgentRunnerResult,
    RunnerTimeouts,
};

const GEMINI_ACP_SESSION_KEY: &str = "gemini_acp_session_id";
const MIN_GEMINI_ACP_VERSION: CliVersion = CliVersion {
    major: 0,
    minor: 30,
    patch: 0,
};

pub struct GeminiAcpRunner {
    config: GeminiAcpConfig,
    timeouts: RunnerTimeouts,
}

impl GeminiAcpRunner {
    pub fn new(config: GeminiAcpConfig, timeouts: RunnerTimeouts) -> Self {
        Self { config, timeouts }
    }
}

#[async_trait]
impl AgentRunner for GeminiAcpRunner {
    fn name(&self) -> &'static str {
        "gemini_acp"
    }

    async fn run(
        &self,
        request: AgentRunnerRequest,
        emitter: Arc<dyn AgentRunnerEmitter>,
    ) -> AgentRunnerResult {
        match run_gemini_acp(&self.config, self.timeouts, request, emitter.clone()).await {
            Ok((response, updates)) => AgentRunnerResult {
                response,
                streamed_output: true,
                terminal_error_emitted: false,
                session_metadata_updates: updates,
                context_messages: None,
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

pub(crate) fn configured_gemini_api_key_env(config: &GeminiAcpConfig) -> &str {
    let env_name = config.api_key_env.trim();
    if env_name.is_empty() {
        "GEMINI_API_KEY"
    } else {
        env_name
    }
}

pub(crate) fn validate_gemini_version(version: CliVersion) -> Result<(), String> {
    if version < MIN_GEMINI_ACP_VERSION {
        return Err(format!(
            "gemini_acp requires gemini >= {MIN_GEMINI_ACP_VERSION}; found {version}. Update with `npm install -g @google/gemini-cli@latest`."
        ));
    }
    Ok(())
}

pub(crate) fn gemini_acp_effective_args(config: &GeminiAcpConfig) -> Vec<String> {
    let mut args = Vec::new();
    let mut iter = config.args.iter().peekable();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--yolo" | "-y" | "--sandbox" | "-s" => continue,
            "--approval-mode" | "--policy" | "--include-directories" => {
                let _ = iter.next();
                continue;
            }
            _ => args.push(arg.clone()),
        }
    }

    if !args
        .iter()
        .any(|arg| arg == "--acp" || arg == "--experimental-acp")
    {
        args.push("--acp".to_string());
    }
    args.push("--approval-mode".to_string());
    args.push("plan".to_string());
    args
}

async fn validate_gemini_acp_environment(
    config: &GeminiAcpConfig,
) -> Result<(), AgentSessionError> {
    let api_key_env = configured_gemini_api_key_env(config);
    let api_key = env::var(api_key_env).unwrap_or_default();
    if api_key.trim().is_empty() {
        tracing::info!(
            "[AgentRunner/gemini_acp] `{}` not set; reusing local gemini-cli login state",
            api_key_env
        );
    } else {
        tracing::info!(
            "[AgentRunner/gemini_acp] using explicit API key from `{}`",
            api_key_env
        );
    }

    let version_output = tokio::process::Command::new(&config.command)
        .arg("--version")
        .output()
        .await
        .map_err(|e| AgentSessionError {
            kind: AgentSessionErrorKind::SpawnFailed,
            message: format!(
                "failed to probe gemini version via `{}`: {e}",
                config.command
            ),
        })?;
    let version_text = String::from_utf8_lossy(&version_output.stdout)
        .trim()
        .to_string();
    let version = parse_cli_version(&version_text).ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::AgentFailed,
        message: format!(
            "gemini_acp requires a parseable `{} --version` output; got `{}`",
            config.command, version_text
        ),
    })?;
    validate_gemini_version(version).map_err(|message| AgentSessionError {
        kind: AgentSessionErrorKind::AgentFailed,
        message,
    })
}

async fn run_gemini_acp(
    config: &GeminiAcpConfig,
    timeouts: RunnerTimeouts,
    request: AgentRunnerRequest,
    emitter: Arc<dyn AgentRunnerEmitter>,
) -> Result<(AgentResponse, HashMap<String, Value>), AgentSessionError> {
    let acp_log = AcpEventLogContext::from_request("gemini", &request);
    validate_gemini_acp_environment(config).await?;

    let startup_timeout = timeouts.step;
    let prompt_idle_timeout = timeouts.step;
    let prompt_overall_timeout = timeouts.overall;
    let mut metadata_updates = HashMap::new();
    let mcp_servers = hone_mcp_servers(&request).map_err(|message| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message,
    })?;

    let mut command = tokio::process::Command::new(&config.command);
    command
        .args(gemini_acp_effective_args(config))
        .current_dir(&request.working_directory)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if !config.model.trim().is_empty() {
        command.arg("--model").arg(config.model.trim());
    }

    let mut child = command.spawn().map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message: format!("failed to spawn gemini acp: {e}"),
    })?;

    let mut stdin = child.stdin.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: "gemini acp stdin unavailable".to_string(),
    })?;
    let stdout = child.stdout.take().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::StdoutUnavailable,
        message: "gemini acp stdout unavailable".to_string(),
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
            "gemini",
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
        message: "gemini acp initialize timeout".to_string(),
    })??;
    next_id += 1;

    let existing_session_id = request
        .session_metadata
        .get(GEMINI_ACP_SESSION_KEY)
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .filter(|value| !value.trim().is_empty());

    let gemini_session_id = if let Some(session_id) = existing_session_id {
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
                "gemini",
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
                session_id
            }
            Ok(Err(err)) => {
                tracing::warn!(
                    "[AgentRunner/gemini_acp] failed to load ACP session {}, creating new one: {}",
                    session_id,
                    err.message
                );
                let new_session_id = create_acp_session(
                    "gemini",
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
                new_session_id
            }
            Err(_) => {
                tracing::warn!(
                    "[AgentRunner/gemini_acp] ACP session/load timed out for {}, creating new one",
                    session_id
                );
                let new_session_id = create_acp_session(
                    "gemini",
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
                new_session_id
            }
        }
    } else {
        let new_session_id = create_acp_session(
            "gemini",
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
        new_session_id
    };

    metadata_updates.insert(
        GEMINI_ACP_SESSION_KEY.to_string(),
        Value::String(gemini_session_id.clone()),
    );

    let mut gemini_state = AcpPromptState::default();
    let prompt_text = build_acp_prompt_text(&request.system_prompt, &request.runtime_input);
    write_jsonrpc_request(
        &mut stdin,
        next_id,
        "session/prompt",
        serde_json::json!({
            "sessionId": gemini_session_id,
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
    let prompt_result = wait_for_response_with_timeouts(
        "gemini",
        &mut reader,
        &mut stdin,
        next_id,
        Some(emitter.clone()),
        Some(&mut gemini_state),
        Some(stderr_buf.clone()),
        AcpResponseTimeouts {
            idle: prompt_idle_timeout,
            overall: prompt_overall_timeout,
        },
        Some(&acp_log),
    )
    .await?;

    let stop_reason = prompt_result
        .get("stopReason")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let success = stop_reason != "cancelled";
    if !success {
        log_acp_prompt_stop_diagnostics(
            "gemini",
            &request.session_id,
            stop_reason,
            &prompt_result,
            &gemini_state,
            &stderr_buf,
        )
        .await;
    }

    let _ = stdin.shutdown().await;
    let _ = child.kill().await;
    if let Some(task) = stderr_task {
        task.abort();
    }
    let content = std::mem::take(&mut gemini_state.full_reply);
    let tool_calls_made = extract_finished_tool_calls(gemini_state);

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
                    "gemini acp prompt stopped with reason={stop_reason}"
                ))
            },
        },
        metadata_updates,
    ))
}
