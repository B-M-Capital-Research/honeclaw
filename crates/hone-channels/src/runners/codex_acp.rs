use async_trait::async_trait;
use hone_core::agent::{AgentMessage, AgentResponse, final_assistant_message_content};
use hone_core::config::{AgentConversationStrategy, CodexAcpConfig};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind};
use crate::mcp_bridge::hone_mcp_servers;
use crate::tool_trace::{
    PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE, persistent_side_effect_state_is_uncertain,
};

use super::acp_common::{
    ACP_PREV_PROMPT_PEAK_KEY, AcpChildGuard, AcpEventLogContext, AcpPermissionDecision,
    AcpPromptState, AcpRenderedToolStatus, AcpResponseTimeouts, AcpRunFailure, AcpToolRenderPhase,
    CliVersion, acp_failure_to_runner_result, acp_prompt_succeeded,
    configure_acp_command_process_group, create_acp_session, finalize_context_messages,
    finalize_pending_tool_calls, log_acp_prompt_stop_diagnostics, parse_cli_version,
    persist_acp_runtime_profile, resume_acp_session, select_acp_adapter_profile,
    select_acp_adapter_profile_from_initialize, wait_for_response,
    wait_for_response_with_timeouts_and_renderer, write_jsonrpc_request,
};
use super::tool_reasoning::render_runner_tool_label;
use super::types::{
    AcpAdapterKind, AcpCompatibilityStatus, AcpStreamDialect, AgentRunner, AgentRunnerEmitter,
    AgentRunnerRequest, AgentRunnerResult, NativeSkillProjection, RunnerTimeouts,
};

const CODEX_ACP_SESSION_KEY: &str = "codex_acp_session_id";
const CODEX_ACP_SESSION_MODE_KEY: &str = "codex_acp_session_mode";
const CODEX_ACP_INSTRUCTION_FINGERPRINT_KEY: &str = "codex_acp_instruction_fingerprint";
const CODEX_ACP_PERSISTENT_SESSION_MODE: &str = "native_turn_v2";
const MIN_CODEX_VERSION: CliVersion = CliVersion {
    major: 0,
    minor: 146,
    patch: 0,
};
const CODEX_ACP_TRANSIENT_SPAWN_RETRY_DELAYS_MS: &[u64] = &[200, 500];

pub(crate) fn persisted_codex_acp_session_id(
    session_metadata: &HashMap<String, Value>,
) -> Option<String> {
    // The persisted native ID is the identity binding. Mode and instruction
    // fingerprint are audit/migration metadata only: changing either must not
    // fork a second visible Codex task for the same logical Hone session.
    session_metadata
        .get(CODEX_ACP_SESSION_KEY)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(crate) struct CodexAcpRunner {
    config: CodexAcpConfig,
    timeouts: RunnerTimeouts,
}

impl CodexAcpRunner {
    pub(crate) fn new(config: CodexAcpConfig, timeouts: RunnerTimeouts) -> Self {
        Self { config, timeouts }
    }
}

pub(crate) fn codex_acp_effective_args(config: &CodexAcpConfig) -> Vec<String> {
    // The official codex-acp executable accepts ACP adapter arguments here.
    // Codex configuration belongs in CODEX_CONFIG and is merged into
    // thread/start or thread/resume by the adapter.
    config.args.clone()
}

pub(crate) fn codex_instruction_fingerprint(instructions: &str) -> String {
    let digest = Sha256::digest(instructions.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_codex_config_literal(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| {
        Value::String(
            raw.trim()
                .trim_matches(|ch| ch == '\'' || ch == '"')
                .to_string(),
        )
    })
}

fn insert_codex_config_path(root: &mut Map<String, Value>, path: &str, value: Value) {
    let mut segments = path
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .peekable();
    let mut current = root;
    while let Some(segment) = segments.next() {
        if segments.peek().is_none() {
            current.insert(segment.to_string(), value);
            return;
        }
        let entry = current
            .entry(segment.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        if !entry.is_object() {
            *entry = Value::Object(Map::new());
        }
        current = entry.as_object_mut().expect("inserted object");
    }
}

pub(crate) fn codex_acp_process_config(
    config: &CodexAcpConfig,
    developer_instructions: Option<&str>,
    locked_down: bool,
) -> Value {
    let mut values = Map::new();

    if let Some(model) = configured_codex_model_id(config) {
        values.insert("model".to_string(), Value::String(model));
    }
    if let Some(effort) = configured_codex_reasoning_effort(config) {
        values.insert("model_reasoning_effort".to_string(), Value::String(effort));
    }
    for override_value in &config.extra_config_overrides {
        let Some((path, raw_value)) = override_value.trim().split_once('=') else {
            continue;
        };
        insert_codex_config_path(
            &mut values,
            path.trim(),
            parse_codex_config_literal(raw_value.trim()),
        );
    }

    if locked_down {
        values.insert(
            "sandbox_mode".to_string(),
            Value::String("workspace-write".to_string()),
        );
        values.insert(
            "approval_policy".to_string(),
            Value::String("never".to_string()),
        );
        values.remove("sandbox_permissions");
    } else {
        if !config.sandbox_mode.trim().is_empty() {
            values.insert(
                "sandbox_mode".to_string(),
                Value::String(config.sandbox_mode.trim().to_string()),
            );
        }
        if !config.approval_policy.trim().is_empty() {
            values.insert(
                "approval_policy".to_string(),
                Value::String(config.approval_policy.trim().to_string()),
            );
        }
        if config.dangerously_bypass_approvals_and_sandbox {
            values.insert(
                "sandbox_mode".to_string(),
                Value::String("danger-full-access".to_string()),
            );
            values.insert(
                "approval_policy".to_string(),
                Value::String("never".to_string()),
            );
        }
        if !config.sandbox_permissions.is_empty() {
            values.insert(
                "sandbox_permissions".to_string(),
                Value::Array(
                    config
                        .sandbox_permissions
                        .iter()
                        .cloned()
                        .map(Value::String)
                        .collect(),
                ),
            );
        }
    }
    if let Some(instructions) = developer_instructions {
        values.insert(
            "developer_instructions".to_string(),
            Value::String(instructions.to_string()),
        );
    }
    Value::Object(values)
}

#[async_trait]
impl AgentRunner for CodexAcpRunner {
    fn name(&self) -> &'static str {
        "codex_acp"
    }

    fn conversation_strategy(&self) -> AgentConversationStrategy {
        AgentConversationStrategy::NativePersistent
    }

    fn acp_adapter_kind(&self) -> Option<AcpAdapterKind> {
        Some(AcpAdapterKind::CodexAcp)
    }

    fn native_skill_projection(&self) -> Option<NativeSkillProjection> {
        Some(NativeSkillProjection::CodexWorkspace)
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
                committed_visible_prefix: None,
                terminal_error_emitted: false,
                session_metadata_updates: updates,
                context_messages,
            },
            Err(failure) => acp_failure_to_runner_result(failure, emitter).await,
        }
    }
}

pub(crate) fn configured_codex_model_id(config: &CodexAcpConfig) -> Option<String> {
    let model = config.model.trim();
    if model.is_empty() {
        return None;
    }

    let (base_model, _) = split_codex_model_and_effort(model);
    Some(base_model.to_string())
}

pub(crate) fn configured_codex_reasoning_effort(config: &CodexAcpConfig) -> Option<String> {
    let variant = config.variant.trim();
    if !variant.is_empty() {
        return Some(variant.to_string());
    }
    let (_, embedded_effort) = split_codex_model_and_effort(config.model.trim());
    embedded_effort.map(ToString::to_string)
}

fn split_codex_model_and_effort(model: &str) -> (&str, Option<&str>) {
    if let Some(without_closing_bracket) = model.strip_suffix(']')
        && let Some((base, effort)) = without_closing_bracket.rsplit_once('[')
        && !base.is_empty()
        && !effort.is_empty()
    {
        return (base, Some(effort));
    }

    if let Some((base, effort)) = model.rsplit_once('/')
        && !base.is_empty()
        && matches!(
            effort,
            "low" | "medium" | "high" | "xhigh" | "max" | "ultra"
        )
    {
        return (base, Some(effort));
    }

    (model, None)
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
    if codex_version.major != MIN_CODEX_VERSION.major {
        return Err(format!(
            "codex version {codex_version} has unsupported major {}; latest validated companion version is {MIN_CODEX_VERSION}",
            codex_version.major
        ));
    }
    select_acp_adapter_profile(AcpAdapterKind::CodexAcp, adapter_version)
        .map(|_| ())
        .map_err(|message| {
            format!(
                "{message}. Update with `npm install -g @agentclientprotocol/codex-acp@latest`."
            )
        })
}

pub(crate) fn codex_version_probe_error_is_transient_resource_unavailable(
    err: &AgentSessionError,
) -> bool {
    if err.kind != AgentSessionErrorKind::SpawnFailed {
        return false;
    }

    let message = err.message.to_ascii_lowercase();
    let from_version_probe =
        message.contains("version probe") || message.contains("failed to probe codex version");
    let resource_unavailable = message.contains("resource temporarily unavailable")
        || message.contains("os error 35")
        || message.contains("would block")
        || message.contains("resource busy")
        || message.contains("temporarily unavailable");

    from_version_probe && resource_unavailable
}

pub(crate) fn codex_resume_error_proves_missing_rollout(
    error: &AgentSessionError,
    persisted_session_id: &str,
) -> bool {
    if error.kind != AgentSessionErrorKind::AgentFailed || persisted_session_id.trim().is_empty() {
        return false;
    }

    let protocol_message = error
        .message
        .split_once(" stderr=")
        .map(|(message, _)| message)
        .unwrap_or(error.message.as_str());
    let expected_detail = format!(
        "details=no rollout found for thread id {}",
        persisted_session_id.trim()
    );

    protocol_message.starts_with("codex acp request failed:")
        && protocol_message.ends_with(&expected_detail)
}

pub(crate) fn codex_spawn_error_is_transient_resource_unavailable(err: &AgentSessionError) -> bool {
    if err.kind != AgentSessionErrorKind::SpawnFailed {
        return false;
    }

    let message = err.message.to_ascii_lowercase();
    (message.contains("failed to spawn codex acp")
        || message.contains("failed to spawn codex-acp")
        || message.contains("failed to spawn codex"))
        && (message.contains("resource temporarily unavailable")
            || message.contains("os error 35")
            || message.contains("would block")
            || message.contains("resource busy")
            || message.contains("temporarily unavailable"))
}

async fn probe_codex_cli_version(
    config: &CodexAcpConfig,
    step_timeout: Duration,
) -> Result<Option<CliVersion>, AgentSessionError> {
    let codex_output = tokio::time::timeout(
        step_timeout,
        tokio::process::Command::new(&config.codex_command)
            .arg("--version")
            .output(),
    )
    .await
    .map_err(|_| AgentSessionError {
        kind: AgentSessionErrorKind::TimeoutOverall,
        message: format!(
            "timed out probing Codex CLI version via `{}`",
            config.codex_command
        ),
    })?
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
    if codex_version < MIN_CODEX_VERSION {
        return Err(AgentSessionError {
            kind: AgentSessionErrorKind::AgentFailed,
            message: format!(
                "codex_acp requires codex >= {MIN_CODEX_VERSION}; found {codex_version}. Update with `npm install -g @openai/codex@latest`."
            ),
        });
    }
    if codex_version.major != MIN_CODEX_VERSION.major {
        return Err(AgentSessionError {
            kind: AgentSessionErrorKind::AgentFailed,
            message: format!(
                "codex version {codex_version} has unsupported major {}; latest validated companion version is {MIN_CODEX_VERSION}",
                codex_version.major
            ),
        });
    }
    Ok(Some(codex_version))
}

async fn spawn_codex_acp_child_with_retry(
    config: &CodexAcpConfig,
    working_directory: &std::path::Path,
    developer_instructions: &str,
) -> Result<tokio::process::Child, AgentSessionError> {
    for (attempt, delay_ms) in CODEX_ACP_TRANSIENT_SPAWN_RETRY_DELAYS_MS
        .iter()
        .copied()
        .enumerate()
    {
        match spawn_codex_acp_child(config, working_directory, developer_instructions) {
            Ok(child) => return Ok(child),
            Err(err) if codex_spawn_error_is_transient_resource_unavailable(&err) => {
                tracing::warn!(
                    attempt = attempt + 1,
                    retry_in_ms = delay_ms,
                    error = %err.message,
                    "codex-acp spawn hit a transient resource limit; retrying"
                );
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
            Err(err) => return Err(err),
        }
    }

    spawn_codex_acp_child(config, working_directory, developer_instructions)
}

fn spawn_codex_acp_child(
    config: &CodexAcpConfig,
    working_directory: &std::path::Path,
    developer_instructions: &str,
) -> Result<tokio::process::Child, AgentSessionError> {
    let mut command = tokio::process::Command::new(&config.command);
    command
        .args(codex_acp_effective_args(config))
        .env("CODEX_PATH", &config.codex_command)
        .env(
            "CODEX_CONFIG",
            codex_acp_process_config(config, Some(developer_instructions), true).to_string(),
        )
        .current_dir(working_directory)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    configure_acp_command_process_group(&mut command);

    command.spawn().map_err(|e| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message: format!("failed to spawn codex acp: {e}"),
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
    AcpRunFailure,
> {
    let acp_log = AcpEventLogContext::from_request("codex", &request);
    let codex_cli_version = match probe_codex_cli_version(config, timeouts.step).await {
        Ok(version) => version,
        Err(error) if codex_version_probe_error_is_transient_resource_unavailable(&error) => {
            tracing::warn!(
                error = %error.message,
                "Codex CLI version probe hit a transient resource limit; the live adapter initialize response remains authoritative for stream selection"
            );
            None
        }
        Err(error) => return Err(error.into()),
    };
    let (developer_instructions, current_user_turn) =
        request
            .conversation
            .native_parts()
            .ok_or(AgentSessionError {
                kind: AgentSessionErrorKind::AgentFailed,
                message: "codex_acp requires native-persistent conversation input".to_string(),
            })?;
    let developer_instructions = developer_instructions.to_string();
    let current_user_turn = current_user_turn.to_string();
    let instruction_fingerprint = codex_instruction_fingerprint(&developer_instructions);

    let startup_timeout = timeouts.step;
    let prompt_idle_timeout = timeouts.step;
    let prompt_overall_timeout = timeouts.overall;
    let mut metadata_updates = HashMap::new();
    let mcp_servers = hone_mcp_servers(&request).map_err(|message| AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message,
    })?;

    let child = spawn_codex_acp_child_with_retry(
        config,
        std::path::Path::new(&request.working_directory),
        &developer_instructions,
    )
    .await?;
    let mut child_guard = AcpChildGuard::new("codex", child, None);

    let child = child_guard.child_mut().ok_or(AgentSessionError {
        kind: AgentSessionErrorKind::Io,
        message: "codex acp child unavailable".to_string(),
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

    let stderr_buffer = Arc::new(tokio::sync::Mutex::new(String::new()));
    let stderr_task = stderr.map(|stderr| {
        let stderr_buffer = stderr_buffer.clone();
        tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut guard = stderr_buffer.lock().await;
                if !guard.is_empty() {
                    guard.push('\n');
                }
                guard.push_str(&line);
            }
        })
    });
    child_guard.set_stderr_task(stderr_task);

    let mut reader = tokio::io::BufReader::new(stdout).lines();
    let mut codex_state = AcpPromptState {
        prev_prompt_peak_used: request
            .session_metadata
            .get(ACP_PREV_PROMPT_PEAK_KEY)
            .and_then(|value| value.as_u64()),
        ..AcpPromptState::default()
    };
    let run_result: Result<Value, AgentSessionError> = async {
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
        let initialize_result = tokio::time::timeout(
            startup_timeout,
            wait_for_response(
                "codex",
                &mut reader,
                &mut stdin,
                next_id,
                None,
                None,
                Some(stderr_buffer.clone()),
                Some(&acp_log),
            ),
        )
        .await
        .map_err(|_| AgentSessionError {
            kind: AgentSessionErrorKind::TimeoutOverall,
            message: "codex acp initialize timeout".to_string(),
        })??;
        let (adapter_version, adapter_profile) = select_acp_adapter_profile_from_initialize(
            AcpAdapterKind::CodexAcp,
            &initialize_result,
        )
        .map_err(|message| AgentSessionError {
            kind: AgentSessionErrorKind::AgentFailed,
            message,
        })?;
        if let Some(codex_cli_version) = codex_cli_version {
            validate_codex_version_matrix(codex_cli_version, adapter_version).map_err(
                |message| AgentSessionError {
                    kind: AgentSessionErrorKind::AgentFailed,
                    message,
                },
            )?;
        }
        let companion_codex_cli_version = codex_cli_version
            .map(|version| version.to_string())
            .unwrap_or_else(|| "probe_unavailable".to_string());
        let companion_codex_cli_compatibility = codex_cli_version
            .map(|version| {
                if version == MIN_CODEX_VERSION {
                    "validated"
                } else {
                    "compatible_newer"
                }
            })
            .unwrap_or("probe_unavailable");
        match adapter_profile.compatibility {
            AcpCompatibilityStatus::Validated => tracing::info!(
                adapter = adapter_profile.adapter.as_str(),
                detected_version = %adapter_profile.detected_version,
                companion_codex_cli_version = %companion_codex_cli_version,
                companion_codex_cli_compatibility,
                dialect = ?adapter_profile.dialect,
                compatibility = adapter_profile.compatibility.as_str(),
                "selected validated ACP stream dialect"
            ),
            AcpCompatibilityStatus::CompatibleNewer => tracing::warn!(
                adapter = adapter_profile.adapter.as_str(),
                detected_version = %adapter_profile.detected_version,
                companion_codex_cli_version = %companion_codex_cli_version,
                companion_codex_cli_compatibility,
                baseline_version = adapter_profile.baseline_version(),
                dialect = ?adapter_profile.dialect,
                compatibility = adapter_profile.compatibility.as_str(),
                "selected conservative ACP stream dialect for an unverified newer adapter"
            ),
        }
        let mut companion_versions = BTreeMap::new();
        companion_versions.insert("codex_cli".to_string(), companion_codex_cli_version);
        if let Err(error) = persist_acp_runtime_profile(
            &request.runtime_dir,
            "codex_acp",
            &adapter_profile,
            companion_versions,
        )
        .await
        {
            tracing::warn!(error = %error, "failed to persist sanitized Codex ACP runtime profile");
        }
        metadata_updates.insert(
            "codex_acp_detected_version".to_string(),
            Value::String(adapter_profile.detected_version.clone()),
        );
        metadata_updates.insert(
            "codex_acp_stream_dialect".to_string(),
            Value::String(adapter_profile.dialect.as_str().to_string()),
        );
        if adapter_profile.dialect != AcpStreamDialect::CodexAcp1_1_7 {
            return Err(AgentSessionError {
                kind: AgentSessionErrorKind::AgentFailed,
                message: format!(
                    "codex_acp selected unsupported stream dialect {:?}",
                    adapter_profile.dialect
                ),
            });
        }
        next_id += 1;

        let (codex_session_id, created_codex_session) = if let Some(session_id) =
            persisted_codex_acp_session_id(&request.session_metadata)
        {
            let previous_fingerprint = request
                .session_metadata
                .get(CODEX_ACP_INSTRUCTION_FINGERPRINT_KEY)
                .and_then(Value::as_str);
            if previous_fingerprint.is_some_and(|value| value != instruction_fingerprint) {
                tracing::info!(
                    "[AgentRunner/codex] session={} developer instructions changed; retaining native acp session={session_id}",
                    request.session_id,
                );
            }
            tracing::info!(
                "[AgentRunner/codex] session={} resuming persistent acp session={session_id}",
                request.session_id,
            );
            let resume_result = resume_acp_session(
                "codex",
                &mut stdin,
                &mut reader,
                next_id,
                &session_id,
                &request.working_directory,
                mcp_servers.clone(),
                startup_timeout,
                stderr_buffer.clone(),
                Some(&acp_log),
            )
            .await;
            match resume_result {
                Ok(()) => (session_id, false),
                Err(error) if codex_resume_error_proves_missing_rollout(&error, &session_id) => {
                    tracing::warn!(
                        "[AgentRunner/codex] session={} native acp rollout is explicitly absent; replacing binding={session_id}",
                        request.session_id,
                    );
                    next_id += 1;
                    (
                        create_acp_session(
                            "codex",
                            &mut stdin,
                            &mut reader,
                            next_id,
                            &request.working_directory,
                            mcp_servers.clone(),
                            startup_timeout,
                            stderr_buffer.clone(),
                            Some(&acp_log),
                        )
                        .await?,
                        true,
                    )
                }
                Err(error) => return Err(error),
            }
        } else {
            tracing::info!(
                "[AgentRunner/codex] session={} creating native-turn-v2 acp session",
                request.session_id,
            );
            (
                create_acp_session(
                    "codex",
                    &mut stdin,
                    &mut reader,
                    next_id,
                    &request.working_directory,
                    mcp_servers.clone(),
                    startup_timeout,
                    stderr_buffer.clone(),
                    Some(&acp_log),
                )
                .await?,
                true,
            )
        };
        next_id += 1;

        metadata_updates.insert(
            CODEX_ACP_SESSION_KEY.to_string(),
            Value::String(codex_session_id.clone()),
        );
        metadata_updates.insert(
            CODEX_ACP_SESSION_MODE_KEY.to_string(),
            Value::String(CODEX_ACP_PERSISTENT_SESSION_MODE.to_string()),
        );
        metadata_updates.insert(
            CODEX_ACP_INSTRUCTION_FINGERPRINT_KEY.to_string(),
            Value::String(instruction_fingerprint.clone()),
        );
        if created_codex_session
            && let Some(checkpoint) = request.session_metadata_checkpoint.as_ref()
        {
            checkpoint
                .persist(metadata_updates.clone())
                .map_err(|message| AgentSessionError {
                    kind: AgentSessionErrorKind::Io,
                    message,
                })?;
        }
        write_jsonrpc_request(
            &mut stdin,
            next_id,
            "session/prompt",
            serde_json::json!({
                "sessionId": codex_session_id,
                "prompt": [
                    {
                        "type": "text",
                        "text": current_user_turn,
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
            Some(stderr_buffer.clone()),
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
        Ok(prompt_result)
    }
    .await;

    let _ = stdin.shutdown().await;
    child_guard.terminate().await;
    let prompt_result = match run_result {
        Ok(prompt_result) => prompt_result,
        Err(error) => {
            metadata_updates.insert(
                ACP_PREV_PROMPT_PEAK_KEY.to_string(),
                Value::from(codex_state.current_prompt_peak_used),
            );
            return Err(AcpRunFailure {
                error,
                state: codex_state,
                metadata_updates,
            });
        }
    };

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
            &stderr_buffer,
        )
        .await;
    }

    // Peak usage remains diagnostics for native compaction detection. A
    // compact event never changes the next user payload: Codex retains its
    // developer instructions and native summary inside the thread.
    metadata_updates.insert(
        ACP_PREV_PROMPT_PEAK_KEY.to_string(),
        Value::from(codex_state.current_prompt_peak_used),
    );
    if codex_state.compact_detected {
        tracing::info!(
            "[AgentRunner/codex] session={} ACP compact detected (peak_used={}); native developer instructions remain authoritative",
            request.session_id,
            codex_state.current_prompt_peak_used
        );
    }

    finalize_pending_tool_calls(&mut codex_state, "unknown_after_missing_acp_result");
    let context_messages = finalize_context_messages(&mut codex_state);
    let content = final_assistant_message_content(
        &context_messages,
        std::mem::take(&mut codex_state.full_reply),
    );
    let tool_calls_made = codex_state.finished_tool_calls.clone();

    let state_uncertain = persistent_side_effect_state_is_uncertain(&tool_calls_made);
    let success = success && !state_uncertain;
    Ok((
        AgentResponse {
            content: if state_uncertain {
                String::new()
            } else {
                content
            },
            tool_calls_made,
            iterations: 1,
            success,
            error: if state_uncertain {
                Some(PERSISTENT_SIDE_EFFECT_UNCERTAIN_MESSAGE.to_string())
            } else if success {
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
    let rendered_command = if let Some(mcp_tool) = render_codex_mcp_tool_call(update) {
        mcp_tool
    } else if is_codex_local_command_update(update) {
        render_codex_execute_command(update).unwrap_or_else(|| "本地命令".to_string())
    } else {
        return AcpRenderedToolStatus {
            tool: default_tool.to_string(),
            message: default_message,
            reasoning: default_reasoning,
        };
    };
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

fn is_codex_local_command_update(update: &Value) -> bool {
    codex_command_value(update).is_some()
}

fn is_codex_execute_result_update(update: &Value) -> bool {
    update
        .get("kind")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind == "execute")
        || is_codex_local_command_update(update)
}

pub(crate) fn patch_codex_session_update_params(params: &Value) -> Option<Value> {
    let update = params.get("update")?;
    let session_update = update
        .get("sessionUpdate")
        .and_then(|value| value.as_str())?;
    if session_update != "tool_call_update" || !is_codex_execute_result_update(update) {
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
    let command = codex_command_value(update)?;
    if let Some(script) = command.as_str() {
        return render_shell_script_category(script);
    }

    let command = command.as_array()?;
    let command = command.iter().filter_map(Value::as_str).collect::<Vec<_>>();
    let program = command
        .first()
        .and_then(|value| command_program_name(value))?;

    if matches!(program.as_str(), "sh" | "bash" | "zsh") {
        let script = command
            .iter()
            .enumerate()
            .find(|(_, value)| matches!(**value, "-c" | "-lc"))
            .and_then(|(index, _)| command.get(index + 1))
            .copied();
        if let Some(label) = script.and_then(render_shell_script_category) {
            return Some(label);
        }
        return Some(format!("运行 shell 命令（{program}）"));
    }

    Some(render_command_category(&program, command.get(1).copied()))
}

fn codex_command_value(update: &Value) -> Option<&Value> {
    update
        .get("rawInput")
        .and_then(|value| value.get("command"))
        .or_else(|| {
            update
                .get("rawOutput")
                .and_then(|value| value.get("command"))
        })
        .filter(|command| match command {
            Value::String(command) => !command.trim().is_empty(),
            Value::Array(command) => !command.is_empty(),
            _ => false,
        })
}

fn render_codex_mcp_tool_call(update: &Value) -> Option<String> {
    let raw_input = update.get("rawInput")?;
    let server = raw_input.get("server").and_then(Value::as_str)?.trim();
    let tool = raw_input.get("tool").and_then(Value::as_str)?.trim();
    if server.is_empty() || tool.is_empty() {
        return None;
    }
    let arguments = raw_input.get("arguments").unwrap_or(&Value::Null);
    let label = render_runner_tool_label(tool, arguments);
    Some(truncate_codex_status_label(&format!("{server}/{label}")))
}

fn render_shell_script_category(script: &str) -> Option<String> {
    for line in script.lines().map(str::trim) {
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with("set ")
            || line.starts_with("export ")
            || matches!(
                line,
                "if" | "then" | "else" | "fi" | "for" | "do" | "done" | "case" | "esac"
            )
        {
            continue;
        }
        let mut words = line.split_whitespace();
        let mut program = words.next()?;
        while is_shell_assignment(program) || matches!(program, "env" | "command" | "exec") {
            program = words.next()?;
        }
        let program = command_program_name(program)?;
        return Some(render_command_category(&program, words.next()));
    }
    None
}

fn is_shell_assignment(value: &str) -> bool {
    let Some((name, _)) = value.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
}

fn command_program_name(value: &str) -> Option<String> {
    let name = value
        .trim_matches(|ch| matches!(ch, '\'' | '"'))
        .rsplit(['/', '\\'])
        .next()?
        .trim();
    (!name.is_empty()
        && name.len() <= 40
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '+' | '-')))
    .then(|| name.to_string())
}

fn safe_subcommand(value: Option<&str>) -> Option<&str> {
    value
        .map(|value| value.trim_matches(|ch| matches!(ch, '\'' | '"')))
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 24
                && value
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
        })
}

fn render_command_category(program: &str, subcommand: Option<&str>) -> String {
    match program {
        "cargo" => safe_subcommand(subcommand)
            .map(|subcommand| format!("运行 Rust 工具（cargo {subcommand}）"))
            .unwrap_or_else(|| "运行 Rust 工具（cargo）".to_string()),
        "rustfmt" => "格式化 Rust 代码（rustfmt）".to_string(),
        "git" => safe_subcommand(subcommand)
            .map(|subcommand| format!("检查 Git（git {subcommand}）"))
            .unwrap_or_else(|| "检查 Git（git）".to_string()),
        "rg" => "搜索本地内容（rg）".to_string(),
        "find" | "ls" | "sed" | "head" | "tail" | "pwd" | "wc" => {
            format!("读取本地内容（{program}）")
        }
        "curl" => "请求接口（curl）".to_string(),
        "jq" => "处理 JSON 数据（jq）".to_string(),
        "python" | "python3" => format!("运行 Python（{program}）"),
        "node" | "bun" | "npm" | "npx" => format!("运行前端工具（{program}）"),
        "launchctl" | "ps" | "pgrep" | "lsof" => format!("检查本机进程（{program}）"),
        "kill" | "killall" => format!("管理本机进程（{program}）"),
        "plutil" => "检查 macOS 配置（plutil）".to_string(),
        "cp" | "mv" | "mkdir" | "rsync" => format!("修改本地文件（{program}）"),
        "sh" | "bash" | "zsh" => format!("运行 shell 命令（{program}）"),
        _ => format!("本地命令（{program}）"),
    }
}

fn truncate_codex_status_label(text: &str) -> String {
    const MAX_CHARS: usize = 120;
    let total = text.chars().count();
    if total <= MAX_CHARS {
        return text.to_string();
    }
    let prefix = text.chars().take(MAX_CHARS - 1).collect::<String>();
    format!("{prefix}…")
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
