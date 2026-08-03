use async_trait::async_trait;
use hone_core::agent::{
    AgentContext, AgentMessage, ToolCallMade, final_assistant_message_content,
    normalize_agent_messages,
};
use hone_core::config::{
    AgentConversationStrategy, CodexAcpConfig, GeminiAcpConfig, OpencodeAcpConfig,
};
use hone_core::{ActorIdentity, ToolExecutionObserver};
use hone_memory::restore_tool_message;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::acp_common::{
    AcpPromptState, AcpToolRenderPhase, CliVersion, extract_finished_tool_calls,
    finalize_context_messages, handle_acp_session_update, handle_acp_session_update_with_renderer,
    parse_cli_version, select_acp_adapter_profile_from_initialize,
    summarize_finished_tool_calls_for_log,
};
use super::codex_acp::{
    codex_acp_effective_args, codex_acp_process_config, codex_instruction_fingerprint,
    configured_codex_model_id, configured_codex_reasoning_effort,
    patch_codex_session_update_params, render_codex_tool_status, reusable_codex_acp_session_id,
    validate_codex_version_matrix,
};
use super::gemini_acp::{gemini_acp_effective_args, validate_gemini_version};
use super::gemini_cli::{
    GeminiCliToolRenderPhase, append_gemini_cli_tool_context_messages,
    render_gemini_cli_tool_status,
};
use super::opencode_acp::{
    build_opencode_acp_prompt_text, configured_opencode_model_id, effective_opencode_args,
    handle_opencode_session_update, isolated_opencode_config, opencode_api_key_log_status,
    resolve_command_path_with_env,
};
use super::tool_reasoning::{
    RunnerToolObserver, render_runner_tool_label, runner_context_messages,
};
use super::types::{
    AcpAdapterKind, AcpCompatibilityStatus, AcpStreamDialect, AgentRunner, AgentRunnerEmitter,
    AgentRunnerEvent, AgentRunnerRequest, DeliveredPushContext, DeliveredPushContextBatch,
    RunnerConversationInput, RunnerTimeouts,
};
use uuid::Uuid;

use crate::agent_session::{AgentSessionError, AgentSessionErrorKind, GeminiStreamOptions};
use crate::mcp_bridge::EMPTY_MCP_TOOL_ALLOWLIST_SENTINEL;

struct NoopEmitter;

#[async_trait]
impl AgentRunnerEmitter for NoopEmitter {
    async fn emit(&self, _event: AgentRunnerEvent) {}
}

#[derive(Debug, PartialEq, Eq)]
struct CapturedToolEvent {
    tool: String,
    status: String,
    message: Option<String>,
    reasoning: Option<String>,
}

#[derive(Default)]
struct CaptureEmitter {
    events: Mutex<Vec<AgentRunnerEvent>>,
}

#[async_trait]
impl AgentRunnerEmitter for CaptureEmitter {
    async fn emit(&self, event: AgentRunnerEvent) {
        self.events.lock().expect("events lock").push(event);
    }
}

impl CaptureEmitter {
    fn tool_events(&self) -> Vec<CapturedToolEvent> {
        self.events
            .lock()
            .expect("events lock")
            .iter()
            .filter_map(|event| match event {
                AgentRunnerEvent::ToolStatus {
                    tool,
                    status,
                    message,
                    reasoning,
                } => Some(CapturedToolEvent {
                    tool: tool.clone(),
                    status: status.clone(),
                    message: message.clone(),
                    reasoning: reasoning.clone(),
                }),
                _ => None,
            })
            .collect()
    }
}

fn assert_contains_all(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "expected text to contain {needle:?}"
        );
    }
}

fn assert_contains_none(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            !haystack.contains(needle),
            "expected text not to contain {needle:?}"
        );
    }
}

fn assert_error_contains<T, E>(result: Result<T, E>, needle: &str)
where
    T: std::fmt::Debug,
    E: std::fmt::Debug + std::fmt::Display,
{
    let err = result.expect_err("expected validation to fail");
    assert!(
        err.to_string().contains(needle),
        "expected error to contain {needle:?}, got {err:?}"
    );
}

fn assert_part_types(parts: &[hone_core::agent::NormalizedConversationPart], expected: &[&str]) {
    let actual: Vec<_> = parts.iter().map(|part| part.part_type.as_str()).collect();
    assert_eq!(actual, expected);
}

fn assert_single_tool_call<'a>(message: &'a AgentMessage, label: &str) -> &'a Value {
    let tool_calls = message.tool_calls.as_ref().expect(label);
    assert_eq!(
        tool_calls.len(),
        1,
        "expected exactly one tool call for {label}"
    );
    &tool_calls[0]
}

fn json_fence_body<'a>(text: &'a str, label: &str) -> &'a str {
    let marker = "```json\n";
    let start = text
        .find(marker)
        .unwrap_or_else(|| panic!("{label} missing opening JSON fence"))
        + marker.len();
    let end = text[start..]
        .find("\n```")
        .unwrap_or_else(|| panic!("{label} missing closing JSON fence"))
        + start;
    &text[start..end]
}

#[test]
fn configured_opencode_model_id_appends_variant() {
    let config = OpencodeAcpConfig {
        model: "openrouter/openai/gpt-5.4".to_string(),
        variant: "medium".to_string(),
        ..OpencodeAcpConfig::default()
    };
    assert_eq!(
        configured_opencode_model_id(&config).as_deref(),
        Some("openrouter/openai/gpt-5.4/medium")
    );
}

#[test]
fn configured_opencode_model_id_does_not_duplicate_variant_suffix() {
    let config = OpencodeAcpConfig {
        model: "openrouter/openai/gpt-5.4/medium".to_string(),
        variant: "medium".to_string(),
        ..OpencodeAcpConfig::default()
    };
    assert_eq!(
        configured_opencode_model_id(&config).as_deref(),
        Some("openrouter/openai/gpt-5.4/medium")
    );
}

#[test]
fn opencode_api_key_log_status_does_not_preview_secret() {
    let status = opencode_api_key_log_status(Some("sk-or-v1-secret-value"));

    assert_eq!(status, "OPENROUTER_API_KEY injected from Hone config");
    assert!(!status.contains("sk-or"));
    assert!(!status.contains("secret"));
}

#[test]
fn opencode_effective_args_replace_existing_cwd() {
    let config = OpencodeAcpConfig {
        args: vec![
            "acp".to_string(),
            "--cwd".to_string(),
            "/tmp/old".to_string(),
        ],
        ..OpencodeAcpConfig::default()
    };
    assert_eq!(
        effective_opencode_args(&config, "/tmp/new"),
        vec!["acp", "--cwd", "/tmp/new"]
    );
}

#[test]
fn isolated_opencode_config_denies_external_directory_and_bash() {
    let config = OpencodeAcpConfig {
        model: "openrouter/google/gemini-3.1-pro-preview".to_string(),
        ..OpencodeAcpConfig::default()
    };
    let payload: Value =
        serde_json::from_str(&isolated_opencode_config(&config)).expect("valid opencode json");
    assert_eq!(payload["permission"]["bash"], "deny");
    assert_eq!(payload["permission"]["external_directory"]["*"], "deny");
    assert_eq!(payload["model"], "openrouter/google/gemini-3.1-pro-preview");
}

#[test]
fn isolated_opencode_config_omits_provider_override_when_base_url_empty() {
    let config = OpencodeAcpConfig::default();
    let payload: Value =
        serde_json::from_str(&isolated_opencode_config(&config)).expect("valid opencode json");
    assert!(payload.get("provider").is_none());
    assert!(payload.get("model").is_none());
    assert_eq!(payload["permission"]["bash"], "deny");
}

#[test]
fn codex_acp_reuses_only_matching_native_turn_generation() {
    let fingerprint = codex_instruction_fingerprint("SYSTEM");
    let mut metadata = HashMap::new();
    metadata.insert(
        "codex_acp_session_id".to_string(),
        Value::String("old-remote-session".to_string()),
    );

    assert!(reusable_codex_acp_session_id(&metadata, &fingerprint).is_none());

    metadata.insert(
        "codex_acp_session_mode".to_string(),
        Value::String("native_turn_v2".to_string()),
    );
    metadata.insert(
        "codex_acp_instruction_fingerprint".to_string(),
        Value::String(fingerprint.clone()),
    );
    assert_eq!(
        reusable_codex_acp_session_id(&metadata, &fingerprint).as_deref(),
        Some("old-remote-session")
    );

    assert!(reusable_codex_acp_session_id(&metadata, "changed").is_none());

    metadata.insert(
        "codex_acp_session_id".to_string(),
        Value::String("  ".to_string()),
    );
    assert!(reusable_codex_acp_session_id(&metadata, &fingerprint).is_none());
}

fn make_temp_exec(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\nexit 0\n").expect("write temp executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(&path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).expect("set permissions");
    }
    path
}

fn native_codex_boundary_request(
    working_directory: &Path,
    developer_instructions: &str,
    current_user_turn: &str,
    session_metadata: HashMap<String, Value>,
) -> AgentRunnerRequest {
    AgentRunnerRequest {
        session_id: "codex-boundary-contract".to_string(),
        actor_label: "cli:codex-boundary-contract".to_string(),
        actor: ActorIdentity::new("cli", "codex-boundary-contract", None::<String>).expect("actor"),
        channel_target: "direct".to_string(),
        allow_cron: false,
        config_path: working_directory.join("config.yaml").display().to_string(),
        runtime_dir: working_directory.join("runtime").display().to_string(),
        conversation: RunnerConversationInput::NativePersistent {
            developer_instructions: developer_instructions.to_string(),
            current_user_turn: current_user_turn.to_string(),
        },
        timeout: None,
        gemini_stream: GeminiStreamOptions::default(),
        session_metadata,
        working_directory: working_directory.display().to_string(),
        allowed_tools: Some(vec![EMPTY_MCP_TOOL_ALLOWLIST_SENTINEL.to_string()]),
        max_tool_calls: None,
        tool_call_limits: None,
        agent_owned_finance_loop: false,
        service_owned_initial_prefix: None,
        terminal_stream_policy: Default::default(),
    }
}

/// External-boundary contract observed against @agentclientprotocol/codex-acp 1.1.7.
/// The double speaks real stdio JSON-RPC and emits the adapter's structured
/// contextCompaction notification between native turns. Assertions target only
/// outbound ACP roles/session lifecycle, not private prompt-builder functions.
#[cfg(unix)]
#[tokio::test]
async fn codex_acp_1_1_7_boundary_keeps_every_prompt_current_turn_only() {
    use std::os::unix::fs::PermissionsExt;

    let temp_root =
        std::env::temp_dir().join(format!("hone-codex-acp-boundary-{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_root).expect("create boundary temp dir");
    let adapter = temp_root.join("fake-codex-acp");
    let capture = temp_root.join("requests.jsonl");
    let config_capture = temp_root.join("codex-config.jsonl");
    let session_counter = temp_root.join("session-counter");
    let script = r#"#!/bin/sh
if [ "${1:-}" = "--version" ]; then
  echo "codex-cli 0.146.0"
  exit 0
fi
printf '%s\n' "$CODEX_CONFIG" >> "__CONFIG_CAPTURE__"
while IFS= read -r line; do
  printf '%s\n' "$line" >> "__CAPTURE__"
  case "$line" in
    *'"method":"initialize"'*)
      echo '{"jsonrpc":"2.0","id":1,"result":{"agentInfo":{"name":"@agentclientprotocol/codex-acp","version":"1.1.7"}}}'
      ;;
    *'"method":"session/new"'*)
      count=0
      if [ -f "__SESSION_COUNTER__" ]; then count=$(sed -n '1p' "__SESSION_COUNTER__"); fi
      count=$((count + 1))
      printf '%s\n' "$count" > "__SESSION_COUNTER__"
      printf '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fake-native-%s"}}\n' "$count"
      ;;
    *'"method":"session/resume"'*)
      echo '{"jsonrpc":"2.0","id":2,"result":{}}'
      ;;
    *'"method":"session/prompt"'*)
      case "$line" in
        *'TURN_ONE'*)
          echo '{"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"tool_call_update","_meta":{"contextCompaction":true}}}}'
          ;;
      esac
      echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
      ;;
  esac
done
"#
    .replace("__CAPTURE__", &capture.display().to_string())
    .replace("__CONFIG_CAPTURE__", &config_capture.display().to_string())
    .replace("__SESSION_COUNTER__", &session_counter.display().to_string());
    fs::write(&adapter, script).expect("write fake ACP adapter");
    let mut permissions = fs::metadata(&adapter)
        .expect("adapter metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&adapter, permissions).expect("make adapter executable");

    let config = CodexAcpConfig {
        command: adapter.display().to_string(),
        codex_command: adapter.display().to_string(),
        model: "gpt-5.6-sol".to_string(),
        variant: "xhigh".to_string(),
        ..CodexAcpConfig::default()
    };
    let runner = super::CodexAcpRunner::new(
        config,
        RunnerTimeouts {
            step: Duration::from_secs(5),
            overall: Duration::from_secs(10),
        },
    );
    assert_eq!(runner.acp_adapter_kind(), Some(AcpAdapterKind::CodexAcp));
    let emitter: Arc<dyn AgentRunnerEmitter> = Arc::new(NoopEmitter);
    let legacy_metadata = HashMap::from([
        (
            "codex_acp_session_id".to_string(),
            Value::String("polluted-v1-session".to_string()),
        ),
        (
            "codex_acp_session_mode".to_string(),
            Value::String("persistent_resume_v1".to_string()),
        ),
        ("acp_needs_sp_reseed".to_string(), Value::Bool(true)),
    ]);

    let first = runner
        .run(
            native_codex_boundary_request(
                &temp_root,
                "HONE_DEVELOPER_INSTRUCTIONS",
                "TURN_ONE",
                legacy_metadata,
            ),
            emitter.clone(),
        )
        .await;
    assert!(
        first.response.success,
        "first turn: {:?}",
        first.response.error
    );
    assert_eq!(
        first.session_metadata_updates["codex_acp_session_id"],
        "fake-native-1"
    );
    assert_eq!(
        first.session_metadata_updates["codex_acp_session_mode"],
        "native_turn_v2"
    );
    assert!(
        !first
            .session_metadata_updates
            .contains_key("acp_needs_sp_reseed")
    );

    let second = runner
        .run(
            native_codex_boundary_request(
                &temp_root,
                "HONE_DEVELOPER_INSTRUCTIONS",
                RunnerConversationInput::prepare(
                    AgentConversationStrategy::NativePersistent,
                    "HONE_DEVELOPER_INSTRUCTIONS".to_string(),
                    "【当前时间】\n2026-08-03 10:00:00 (北京时间)\n\n【本轮用户输入】\nTURN_TWO"
                        .to_string(),
                    AgentContext::new("codex-boundary-contract".to_string()),
                    DeliveredPushContextBatch {
                        records: vec![DeliveredPushContext {
                            delivery_log_id: 7,
                            source_id: "push-7".to_string(),
                            delivered_at_ms: 1_775_356_740_000,
                            body: "PUSH_BEFORE_TURN_TWO".to_string(),
                        }],
                        remaining_count: 0,
                    },
                )
                .current_user_turn(),
                first.session_metadata_updates.clone(),
            ),
            emitter.clone(),
        )
        .await;
    assert!(
        second.response.success,
        "second turn: {:?}",
        second.response.error
    );

    let third = runner
        .run(
            native_codex_boundary_request(
                &temp_root,
                "CHANGED_DEVELOPER_INSTRUCTIONS",
                "TURN_THREE",
                second.session_metadata_updates.clone(),
            ),
            emitter,
        )
        .await;
    assert!(
        third.response.success,
        "third turn: {:?}",
        third.response.error
    );
    assert_eq!(
        third.session_metadata_updates["codex_acp_session_id"],
        "fake-native-2"
    );

    let requests = fs::read_to_string(&capture).expect("captured ACP requests");
    let payloads = requests
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).expect("captured JSON-RPC request"))
        .collect::<Vec<_>>();
    let session_actions = payloads
        .iter()
        .filter_map(|payload| payload.get("method").and_then(Value::as_str))
        .filter(|method| matches!(*method, "session/new" | "session/resume"))
        .collect::<Vec<_>>();
    assert_eq!(
        session_actions,
        vec!["session/new", "session/resume", "session/new"]
    );
    let prompts = payloads
        .iter()
        .filter(|payload| payload["method"] == "session/prompt")
        .map(|payload| {
            payload["params"]["prompt"][0]["text"]
                .as_str()
                .expect("text prompt")
        })
        .collect::<Vec<_>>();
    assert_eq!(prompts[0], "TURN_ONE");
    assert_eq!(prompts[2], "TURN_THREE");
    let push_pos = prompts[1]
        .find("PUSH_BEFORE_TURN_TWO")
        .expect("delivered push context in Codex ACP 1.1.7 prompt");
    let current_user_pos = prompts[1]
        .find("【本轮用户输入】\nTURN_TWO")
        .expect("current user input in Codex ACP 1.1.7 prompt");
    assert!(push_pos < current_user_pos);
    for prompt in prompts {
        for forbidden in [
            "HONE_DEVELOPER_INSTRUCTIONS",
            "System Instructions",
            "Restored Conversation Transcript",
            "tool_call",
            "tool_result",
        ] {
            assert!(
                !prompt.contains(forbidden),
                "forbidden {forbidden}: {prompt}"
            );
        }
    }

    let process_configs = fs::read_to_string(&config_capture).expect("captured CODEX_CONFIG");
    let developer_layers = process_configs
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|value| {
            value
                .get("developer_instructions")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        developer_layers,
        vec![
            "HONE_DEVELOPER_INSTRUCTIONS",
            "HONE_DEVELOPER_INSTRUCTIONS",
            "CHANGED_DEVELOPER_INSTRUCTIONS",
        ]
    );

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn resolve_opencode_command_prefers_existing_path_entry() {
    let temp_root = std::env::temp_dir().join(format!("hone-opencode-path-{}", Uuid::new_v4()));
    let bin_dir = temp_root.join("bin");
    fs::create_dir_all(&bin_dir).expect("create bin dir");
    let command_name = format!("opencode-test-{}", Uuid::new_v4());
    let binary = make_temp_exec(&bin_dir, &command_name);

    let resolved = resolve_command_path_with_env(&command_name, Some(bin_dir.as_os_str()), None);
    assert_eq!(resolved, binary);

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn resolve_opencode_command_falls_back_to_home_local_bin() {
    let temp_home = std::env::temp_dir().join(format!("hone-opencode-home-{}", Uuid::new_v4()));
    let local_bin = temp_home.join(".local").join("bin");
    fs::create_dir_all(&local_bin).expect("create local bin");
    let command_name = format!("opencode-test-{}", Uuid::new_v4());
    let binary = make_temp_exec(&local_bin, &command_name);

    let resolved = resolve_command_path_with_env(&command_name, None, Some(&temp_home));
    assert_eq!(resolved, binary);

    let _ = fs::remove_dir_all(&temp_home);
}

#[test]
fn resolve_opencode_command_prefers_bundled_env_override() {
    let temp_root = std::env::temp_dir().join(format!("hone-opencode-bundled-{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_root).expect("create temp root");
    let binary = make_temp_exec(&temp_root, "opencode");

    unsafe {
        std::env::set_var("HONE_BUNDLED_OPENCODE_BIN", &binary);
    }
    let resolved = resolve_command_path_with_env("opencode", None, None);
    assert_eq!(resolved, binary);
    unsafe {
        std::env::remove_var("HONE_BUNDLED_OPENCODE_BIN");
    }

    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn configured_codex_model_id_returns_base_model_for_process_config() {
    let config = CodexAcpConfig {
        model: "gpt-5.5".to_string(),
        variant: "high".to_string(),
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        configured_codex_model_id(&config).as_deref(),
        Some("gpt-5.5")
    );
}

#[test]
fn configured_codex_model_id_strips_legacy_variant_suffix() {
    let config = CodexAcpConfig {
        model: "gpt-5.4/medium".to_string(),
        variant: "medium".to_string(),
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        configured_codex_model_id(&config).as_deref(),
        Some("gpt-5.4")
    );
}

#[test]
fn configured_codex_model_id_strips_bracketed_effort() {
    let config = CodexAcpConfig {
        model: "gpt-5.6-sol[high]".to_string(),
        variant: "xhigh".to_string(),
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        configured_codex_model_id(&config).as_deref(),
        Some("gpt-5.6-sol")
    );
}

#[test]
fn configured_codex_model_id_uses_embedded_effort_when_variant_is_empty() {
    let config = CodexAcpConfig {
        model: "gpt-5.6-sol[xhigh]".to_string(),
        variant: String::new(),
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        configured_codex_model_id(&config).as_deref(),
        Some("gpt-5.6-sol")
    );
    assert_eq!(
        configured_codex_reasoning_effort(&config).as_deref(),
        Some("xhigh")
    );
}

#[test]
fn configured_codex_model_id_keeps_bare_model_without_effort() {
    let config = CodexAcpConfig {
        model: "gpt-5.6-sol".to_string(),
        variant: String::new(),
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        configured_codex_model_id(&config).as_deref(),
        Some("gpt-5.6-sol")
    );
}

#[test]
fn configured_codex_reasoning_effort_reads_variant() {
    let with_variant = CodexAcpConfig {
        model: "gpt-5.5".to_string(),
        variant: "high".to_string(),
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        configured_codex_reasoning_effort(&with_variant).as_deref(),
        Some("high")
    );

    let empty_variant = CodexAcpConfig {
        model: "gpt-5.5".to_string(),
        variant: String::new(),
        ..CodexAcpConfig::default()
    };
    assert!(configured_codex_reasoning_effort(&empty_variant).is_none());
}

#[test]
fn codex_acp_uses_adapter_args_and_official_codex_config_boundary() {
    let config = CodexAcpConfig {
        variant: "high".to_string(),
        args: vec!["--adapter-flag".to_string()],
        extra_config_overrides: vec!["shell_environment_policy.inherit=\"all\"".to_string()],
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        codex_acp_effective_args(&config),
        vec!["--adapter-flag".to_string()]
    );
    let process_config = codex_acp_process_config(&config, Some("HONE SYSTEM"), true);
    assert_eq!(process_config["model"], "gpt-5.6-sol");
    assert_eq!(process_config["model_reasoning_effort"], "high");
    assert_eq!(process_config["sandbox_mode"], "workspace-write");
    assert_eq!(process_config["approval_policy"], "never");
    assert_eq!(process_config["developer_instructions"], "HONE SYSTEM");
    assert_eq!(process_config["shell_environment_policy"]["inherit"], "all");
}

#[test]
fn parse_cli_version_extracts_semver() {
    assert_eq!(
        parse_cli_version("codex-cli 0.115.0"),
        Some(CliVersion {
            major: 0,
            minor: 115,
            patch: 0,
        })
    );
    assert_eq!(
        parse_cli_version("version=0.9.5"),
        Some(CliVersion {
            major: 0,
            minor: 9,
            patch: 5,
        })
    );
}

#[test]
fn gemini_cli_tool_status_renders_argument_summary_and_reasoning() {
    let rendered = render_gemini_cli_tool_status(
        "web_search",
        &serde_json::json!({
            "query": "AAOI COHR after hours move and sector sympathy"
        }),
        Some("正在搜索盘后异动背景".to_string()),
        GeminiCliToolRenderPhase::Start,
    );

    assert_eq!(
        rendered.tool,
        "web_search query=\"AAOI COHR after hours move and sector sympathy\""
    );
    assert!(rendered.message.is_none());
    assert_eq!(
        rendered.reasoning.as_deref(),
        Some(
            "正在执行：web_search query=\"AAOI COHR after hours move and sector sympathy\"；说明：正在搜索盘后异动背景"
        )
    );

    let done = render_gemini_cli_tool_status(
        "data_fetch",
        &serde_json::json!({
            "data_type": "quote",
            "symbol": "NVDA"
        }),
        None,
        GeminiCliToolRenderPhase::Done,
    );
    assert_eq!(done.tool, "data_fetch quote NVDA");
    assert_eq!(
        done.message.as_deref(),
        Some("执行完成：data_fetch quote NVDA")
    );
    assert!(done.reasoning.is_none());
}

#[test]
fn gemini_cli_tool_context_messages_capture_assistant_and_tool_entries() {
    let mut messages = Vec::new();
    append_gemini_cli_tool_context_messages(
        &mut messages,
        "gemini_cli_call_1_1",
        "我先查一下盘后新闻。",
        "web_search",
        &serde_json::json!({
            "query": "AAOI COHR after hours move"
        }),
        "{\"ok\":true}",
    );

    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, "assistant");
    assert_eq!(messages[0].content.as_deref(), Some("我先查一下盘后新闻。"));
    let tool_call = assert_single_tool_call(&messages[0], "assistant tool calls");
    assert_eq!(tool_call["id"], "gemini_cli_call_1_1");
    assert_eq!(tool_call["function"]["name"], "web_search");
    assert_eq!(
        tool_call["function"]["arguments"],
        "{\"query\":\"AAOI COHR after hours move\"}"
    );

    assert_eq!(messages[1].role, "tool");
    assert_eq!(
        messages[1].tool_call_id.as_deref(),
        Some("gemini_cli_call_1_1")
    );
    assert_eq!(messages[1].name.as_deref(), Some("web_search"));
    assert_eq!(messages[1].content.as_deref(), Some("{\"ok\":true}"));
}

#[test]
fn runner_tool_label_summarizes_arguments() {
    assert_eq!(
        render_runner_tool_label(
            "data_fetch",
            &serde_json::json!({
                "data_type": "quote",
                "symbol": "AAOI,COHR"
            })
        ),
        "data_fetch quote AAOI,COHR"
    );
    assert_eq!(
        render_runner_tool_label(
            "web_search",
            &serde_json::json!({
                "query": "AAOI COHR after hours move"
            })
        ),
        "web_search query=\"AAOI COHR after hours move\""
    );
}

#[tokio::test]
async fn runner_tool_finish_distinguishes_success_from_failure() {
    let emitter = Arc::new(CaptureEmitter::default());
    let observer = RunnerToolObserver {
        emitter: emitter.clone(),
    };
    let arguments = serde_json::json!({
        "data_type": "quote",
        "symbol": "CRWV"
    });

    observer
        .on_tool_finish("data_fetch", &arguments, true)
        .await;
    observer
        .on_tool_finish("data_fetch", &arguments, false)
        .await;

    let events = emitter.tool_events();
    assert_eq!(
        events,
        vec![
            CapturedToolEvent {
                tool: "data_fetch quote CRWV".to_string(),
                status: "done".to_string(),
                message: Some("执行完成：data_fetch quote CRWV".to_string()),
                reasoning: None,
            },
            CapturedToolEvent {
                tool: "data_fetch quote CRWV".to_string(),
                status: "failed".to_string(),
                message: Some("执行失败：data_fetch quote CRWV".to_string()),
                reasoning: None,
            },
        ]
    );
    assert!(
        !events[1]
            .message
            .as_deref()
            .expect("failure message")
            .contains("执行完成")
    );
}

#[test]
fn runner_context_messages_drop_new_user_message_and_keep_transcript_tail() {
    let mut context = AgentContext::new("session-1".to_string());
    context.add_user_message("old user");
    context.add_assistant_message("old assistant", None);
    let original_len = context.messages.len();

    context.add_user_message("new user");
    context.add_assistant_message("让我先查一下。", None);
    context.add_tool_result("tc_1", "data_fetch", "{\"ok\":true}");
    context.add_assistant_message("结论：AAOI 更弱。", None);

    let messages = runner_context_messages(&context, original_len).expect("new messages");
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].role, "assistant");
    assert_eq!(messages[0].content.as_deref(), Some("让我先查一下。"));
    assert_eq!(messages[1].role, "tool");
    assert_eq!(messages[1].tool_call_id.as_deref(), Some("tc_1"));
    assert_eq!(messages[1].name.as_deref(), Some("data_fetch"));
    assert_eq!(messages[2].role, "assistant");
    assert_eq!(messages[2].content.as_deref(), Some("结论：AAOI 更弱。"));
}

#[test]
fn codex_cli_context_messages_are_ready_for_normalized_persistence() {
    let mut context = AgentContext::new("session-1".to_string());
    context.add_user_message("old user");
    context.add_assistant_message("old assistant", None);
    let original_len = context.messages.len();

    context.add_user_message("new user");
    context.add_assistant_message(
        "先检查本地版本。",
        Some(vec![serde_json::json!({
            "id": "call_1",
            "type": "function",
            "function": {
                "name": "run_shell",
                "arguments": "{\"cmd\":\"rtk --version\"}"
            }
        })]),
    );
    context.add_tool_result("call_1", "run_shell", "rtk 0.35.0\n");
    context.add_assistant_message("VERSION=rtk 0.35.0", None);

    let messages = runner_context_messages(&context, original_len).expect("new messages");
    let normalized = normalize_agent_messages(&messages);
    assert_eq!(normalized.len(), 1);
    assert_eq!(normalized[0].role, "assistant");
    assert_part_types(
        &normalized[0].content,
        &["progress", "tool_call", "tool_result", "final"],
    );
}

#[test]
fn codex_version_matrix_accepts_minimum_validated_pair() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 146,
            patch: 0,
        },
        CliVersion {
            major: 1,
            minor: 1,
            patch: 7,
        },
    );
    result.expect("minimum Codex/Codex ACP versions should be accepted");
}

#[test]
fn codex_version_matrix_accepts_newer_adapter() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 146,
            patch: 0,
        },
        CliVersion {
            major: 1,
            minor: 2,
            patch: 0,
        },
    );
    result.expect("newer Codex ACP adapter should be accepted");
}

#[test]
fn codex_version_matrix_rejects_old_codex() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 145,
            patch: 9,
        },
        CliVersion {
            major: 1,
            minor: 1,
            patch: 7,
        },
    );
    assert_error_contains(result, "npm install -g @openai/codex@latest");
}

#[test]
fn codex_version_matrix_rejects_unknown_codex_major() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 1,
            minor: 0,
            patch: 0,
        },
        CliVersion {
            major: 1,
            minor: 1,
            patch: 7,
        },
    );
    let error = result.expect_err("unknown Codex major must fail closed");
    assert!(error.contains("unsupported major"));
    assert!(error.contains("0.146.0"));
}

#[test]
fn codex_version_matrix_rejects_old_adapter() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 146,
            patch: 0,
        },
        CliVersion {
            major: 1,
            minor: 1,
            patch: 6,
        },
    );
    assert_error_contains(result, "@agentclientprotocol/codex-acp@latest");
}

#[test]
fn codex_version_probe_resource_limit_errors_are_bypassable() {
    let err = AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message: "failed to probe codex version via `codex`: Resource temporarily unavailable (os error 35)"
            .to_string(),
    };

    assert!(super::codex_acp::codex_version_probe_error_is_transient_resource_unavailable(&err));
}

#[test]
fn codex_version_probe_missing_binary_is_not_bypassable() {
    let err = AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message:
            "failed to probe codex version via `codex`: No such file or directory (os error 2)"
                .to_string(),
    };

    assert!(!super::codex_acp::codex_version_probe_error_is_transient_resource_unavailable(&err));
}

#[test]
fn codex_spawn_resource_limit_errors_are_retryable() {
    let err = AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message: "failed to spawn codex acp: Resource temporarily unavailable (os error 35)"
            .to_string(),
    };

    assert!(super::codex_acp::codex_spawn_error_is_transient_resource_unavailable(&err));
}

#[test]
fn codex_spawn_missing_binary_is_not_retryable() {
    let err = AgentSessionError {
        kind: AgentSessionErrorKind::SpawnFailed,
        message: "failed to spawn codex acp: No such file or directory (os error 2)".to_string(),
    };

    assert!(!super::codex_acp::codex_spawn_error_is_transient_resource_unavailable(&err));
}

#[test]
fn gemini_version_guard_rejects_old_binary() {
    let result = validate_gemini_version(CliVersion {
        major: 0,
        minor: 29,
        patch: 0,
    });
    assert_error_contains(result, "@google/gemini-cli@latest");
}

#[test]
fn gemini_acp_effective_args_strip_sandbox_and_include_plan_mode() {
    let config = GeminiAcpConfig {
        args: vec![
            "--experimental-acp".to_string(),
            "--sandbox".to_string(),
            "--approval-mode".to_string(),
            "yolo".to_string(),
            "--include-directories".to_string(),
            "/tmp".to_string(),
            "--yolo".to_string(),
        ],
        ..GeminiAcpConfig::default()
    };
    assert_eq!(
        gemini_acp_effective_args(&config),
        vec!["--experimental-acp", "--approval-mode", "plan",]
    );
}

#[test]
fn restore_tool_message_rebuilds_context_tuple() {
    let mut metadata = HashMap::new();
    metadata.insert(
        "tool_name".to_string(),
        Value::String("web_search".to_string()),
    );
    metadata.insert(
        "tool_call_id".to_string(),
        Value::String("call_1".to_string()),
    );
    let restored = restore_tool_message(&hone_memory::session::SessionMessage {
        role: "tool".to_string(),
        content: vec![hone_core::agent::NormalizedConversationPart {
            part_type: "tool_result".to_string(),
            text: None,
            id: Some("call_1".to_string()),
            name: Some("web_search".to_string()),
            args: None,
            result: Some(serde_json::json!({"result": true})),
            metadata: None,
        }],
        status: Some("completed".to_string()),
        timestamp: "2026-04-15T00:00:00+08:00".to_string(),
        metadata: Some(metadata),
    })
    .expect("tool message");
    assert_eq!(restored.0, "call_1");
    assert_eq!(restored.1, "web_search");
    assert_eq!(restored.2, "{\"result\":true}");
}

#[test]
fn extract_finished_tool_calls_returns_collected_records() {
    let mut state = AcpPromptState::default();
    state.finished_tool_calls.push(ToolCallMade {
        name: "web_search".to_string(),
        arguments: serde_json::json!({"query": "AAPL"}),
        result: serde_json::json!({"ok": true}),
        tool_call_id: Some("call_1".to_string()),
    });

    let calls = extract_finished_tool_calls(state);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "web_search");
    assert_eq!(calls[0].result["ok"].as_bool(), Some(true));
}

#[test]
fn summarize_finished_tool_calls_for_log_limits_output_to_count_and_recent_entries() {
    let calls = vec![
        ToolCallMade {
            name: "web_search".to_string(),
            arguments: serde_json::json!({"query": "AAOI"}),
            result: serde_json::json!({"ok": true}),
            tool_call_id: Some("call_1".to_string()),
        },
        ToolCallMade {
            name: "data_fetch".to_string(),
            arguments: serde_json::json!({"ticker": "COHR"}),
            result: serde_json::json!({"ok": true}),
            tool_call_id: Some("call_2".to_string()),
        },
    ];

    let summary = summarize_finished_tool_calls_for_log(&calls);
    assert_contains_all(
        &summary,
        &["count=2", "data_fetch#call_2", "web_search#call_1"],
    );
    assert_contains_none(&summary, &["AAOI", "COHR"]);
}

#[tokio::test]
async fn acp_updates_build_restorable_transcript_sequence() {
    let emitter: Arc<dyn AgentRunnerEmitter> = Arc::new(NoopEmitter);
    let mut state = AcpPromptState::default();

    handle_acp_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "text": "先查本地画像。"
            }
        }),
        &emitter,
        Some(&mut state),
    )
    .await;
    handle_acp_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call_1",
                "title": "local_search_files",
                "arguments": {
                    "query": "AAOI",
                    "path": "company_profiles"
                }
            }
        }),
        &emitter,
        Some(&mut state),
    )
    .await;
    handle_acp_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "call_1",
                "title": "local_search_files",
                "status": "completed",
                "result": {
                    "matches": ["company_profiles/applied-optoelectronics/profile.md"]
                }
            }
        }),
        &emitter,
        Some(&mut state),
    )
    .await;
    handle_acp_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "text": "AAOI 是做光模块的。"
            }
        }),
        &emitter,
        Some(&mut state),
    )
    .await;

    let messages = finalize_context_messages(&mut state);
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0].role, "assistant");
    assert_eq!(messages[0].content.as_deref(), Some("先查本地画像。"));
    assert!(messages[0].tool_calls.is_none());
    let tool_call = assert_single_tool_call(&messages[1], "assistant tool calls");
    assert_eq!(tool_call["id"], "call_1");
    assert_eq!(tool_call["function"]["name"], "local_search_files");
    assert_eq!(messages[1].role, "assistant");
    assert_eq!(messages[1].content.as_deref(), Some(""));
    assert_eq!(messages[2].role, "tool");
    assert_eq!(messages[2].tool_call_id.as_deref(), Some("call_1"));
    assert_eq!(messages[2].name.as_deref(), Some("local_search_files"));
    assert!(
        messages[2]
            .content
            .as_deref()
            .is_some_and(|value| value.contains("applied-optoelectronics"))
    );
    assert_eq!(messages[3].role, "assistant");
    assert_eq!(messages[3].content.as_deref(), Some("AAOI 是做光模块的。"));
}

#[test]
fn normalized_history_collapses_tool_messages_into_assistant_turns() {
    let mut context = AgentContext::new("session-1".to_string());
    context.add_user_message("FLNC 现在怎么看");
    context.messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some("我先核验实体和现价。".to_string()),
        tool_calls: Some(vec![serde_json::json!({
            "id": "call_1",
            "type": "function",
            "function": {
                "name": "web_search",
                "arguments": "{\"query\":\"FLNC stock price\"}"
            }
        })]),
        tool_call_id: None,
        name: None,
        metadata: Some(HashMap::from([(
            "codex_acp".to_string(),
            serde_json::json!({ "segment_kind": "progress_note" }),
        )])),
    });
    context.messages.push(AgentMessage {
        role: "tool".to_string(),
        content: Some("{\"price\":5.12}".to_string()),
        tool_calls: None,
        tool_call_id: Some("call_1".to_string()),
        name: Some("web_search".to_string()),
        metadata: None,
    });
    context.add_assistant_message("结论：先看订单兑现，再谈估值弹性。", None);

    let history = context.normalized_history();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].role, "user");
    assert_eq!(history[1].role, "assistant");
    assert_eq!(history[1].status.as_deref(), Some("completed"));
    assert_part_types(
        &history[1].content,
        &["progress", "tool_call", "tool_result", "final"],
    );
    assert_eq!(history[1].content[1].name.as_deref(), Some("web_search"));
    assert_eq!(
        history[1].content[1].args,
        Some(serde_json::json!({"query":"FLNC stock price"}))
    );
    assert_eq!(
        history[1].content[2].result,
        Some(serde_json::json!({"price":5.12}))
    );
    assert_eq!(
        history[1].content[3].text.as_deref(),
        Some("结论：先看订单兑现，再谈估值弹性。")
    );
}

#[test]
fn opencode_compiled_prompt_preserves_normalized_history() {
    let mut context = AgentContext::new("session-1".to_string());
    context.add_user_message("FLNC 现在怎么看");
    context.messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some("我先查最新价格和财报。".to_string()),
        tool_calls: Some(vec![serde_json::json!({
            "id": "call_1",
            "type": "function",
            "function": {
                "name": "web_search",
                "arguments": "{\"query\":\"FLNC earnings stock price\"}"
            }
        })]),
        tool_call_id: None,
        name: None,
        metadata: None,
    });
    context.messages.push(AgentMessage {
        role: "tool".to_string(),
        content: Some("{\"price\":5.12,\"earnings_date\":\"2026-02-04\"}".to_string()),
        tool_calls: None,
        tool_call_id: Some("call_1".to_string()),
        name: Some("web_search".to_string()),
        metadata: None,
    });
    context.add_assistant_message("结论：先看订单兑现，再判断估值弹性。", None);

    let opencode_prompt = build_opencode_acp_prompt_text("SYSTEM", "新的问题", Some(&context));
    let transcript = json_fence_body(&opencode_prompt, "opencode transcript");
    assert_contains_all(
        transcript,
        &[
            "\"role\": \"assistant\"",
            "\"type\": \"tool_call\"",
            "\"type\": \"tool_result\"",
            "\"type\": \"final\"",
        ],
    );
}

#[test]
fn final_response_content_prefers_last_assistant_segment() {
    let messages = vec![
        AgentMessage {
            role: "assistant".to_string(),
            content: Some("先核验实体和现价。".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            metadata: None,
        },
        AgentMessage {
            role: "tool".to_string(),
            content: Some("{\"ok\":true}".to_string()),
            tool_calls: None,
            tool_call_id: Some("call_1".to_string()),
            name: Some("web_search".to_string()),
            metadata: None,
        },
        AgentMessage {
            role: "assistant".to_string(),
            content: Some("结论：当前价位偏交易化，需看储能订单兑现。".to_string()),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            metadata: None,
        },
    ];

    let content =
        final_assistant_message_content(&messages, "先核验实体和现价。结论：fallback".to_string());
    assert_eq!(content, "结论：当前价位偏交易化，需看储能订单兑现。");
}

#[test]
fn codex_execute_renderer_shows_safe_category_and_appends_purpose() {
    let long_script = "python - <<'PY'\n".to_string() + &"x".repeat(2400);
    let rendered = render_codex_tool_status(
        &serde_json::json!({
            "kind": "execute",
            "rawInput": {
                "command": ["/bin/zsh", "-lc", long_script],
                "purpose": "提取 runtime 目录中的 ticker 命中情况"
            }
        }),
        AcpToolRenderPhase::Start,
        "Run python",
        None,
        Some("default".to_string()),
    );

    assert_eq!(rendered.tool, "运行 Python（python）");
    assert!(rendered.message.is_none());
    assert!(
        rendered
            .reasoning
            .as_deref()
            .is_some_and(|value| value.starts_with("正在执行：运行 Python（python）"))
    );
    assert!(
        rendered
            .reasoning
            .as_deref()
            .is_some_and(|value| value.contains("；目的：提取 runtime 目录中的 ticker 命中情况"))
    );
}

#[test]
fn codex_execute_renderer_formats_done_message() {
    let rendered = render_codex_tool_status(
        &serde_json::json!({
            "kind": "execute",
            "rawInput": {
                "command": ["/bin/zsh", "-lc", "rtk ls -la uploads"]
            }
        }),
        AcpToolRenderPhase::Done,
        "Run rtk ls -la uploads",
        Some("工具执行完成".to_string()),
        None,
    );

    assert_eq!(rendered.tool, "本地命令（rtk）");
    assert_eq!(
        rendered.message.as_deref(),
        Some("执行完成：本地命令（rtk）")
    );
    assert!(rendered.reasoning.is_none());
}

/// Captured from the Codex ACP 1.1.7 execute-start stream on 2026-08-02.
/// Current adapters may send `rawInput.command` as a string rather than argv.
#[test]
fn codex_execute_renderer_summarizes_real_string_command_shape() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/acp/codex-acp-1.1.7.json"
    ))
    .expect("Codex ACP 1.1.7 fixture");
    assert_eq!(fixture["adapter"]["version"], "1.1.7");
    let rendered = render_codex_tool_status(
        &fixture["string_command_start"]["update"],
        AcpToolRenderPhase::Start,
        "git status --short",
        None,
        None,
    );

    assert_eq!(rendered.tool, "检查 Git（git status）");
    assert_eq!(
        rendered.reasoning.as_deref(),
        Some("正在执行：检查 Git（git status）")
    );
    assert!(!rendered.tool.contains("--short"));
    assert!(!rendered.tool.contains("hone-agent-sandbox"));
}

#[tokio::test]
async fn codex_execute_completion_reuses_safe_start_summary() {
    let emitter = Arc::new(CaptureEmitter::default());
    let emitter_trait: Arc<dyn AgentRunnerEmitter> = emitter.clone();
    let mut state = AcpPromptState::default();

    let start = serde_json::json!({
        "update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "exec-string-command-1",
            "kind": "execute",
            "title": "pwd",
            "rawInput": {
                "command": "pwd",
                "cwd": "/private/tmp/hone-agent-sandbox"
            }
        }
    });
    handle_acp_session_update_with_renderer(
        &start,
        &emitter_trait,
        Some(&mut state),
        Some(render_codex_tool_status),
    )
    .await;

    let completed = serde_json::json!({
        "update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "exec-string-command-1",
            "status": "completed",
            "rawOutput": {
                "formatted_output": "/private/tmp/hone-agent-sandbox\n",
                "exit_code": 0
            }
        }
    });
    handle_acp_session_update_with_renderer(
        &completed,
        &emitter_trait,
        Some(&mut state),
        Some(render_codex_tool_status),
    )
    .await;

    assert_eq!(
        emitter.tool_events(),
        vec![
            CapturedToolEvent {
                tool: "读取本地内容（pwd）".to_string(),
                status: "start".to_string(),
                message: None,
                reasoning: Some("正在执行：读取本地内容（pwd）".to_string()),
            },
            CapturedToolEvent {
                tool: "读取本地内容（pwd）".to_string(),
                status: "done".to_string(),
                message: Some("执行完成：读取本地内容（pwd）".to_string()),
                reasoning: None,
            },
        ]
    );
}

#[test]
fn codex_mcp_execute_renderer_shows_bounded_business_tool_summary() {
    let rendered = render_codex_tool_status(
        &serde_json::json!({
            "kind": "execute",
            "title": "mcp.hone.web_search",
            "_meta": {"is_mcp_tool_call": true},
            "rawInput": {
                "server": "hone",
                "tool": "web_search",
                "arguments": {
                    "query": "NVIDIA Rubin platform official 2026 specifications"
                }
            }
        }),
        AcpToolRenderPhase::Start,
        "mcp.hone.web_search",
        None,
        None,
    );

    assert_eq!(
        rendered.tool,
        "hone/web_search query=\"NVIDIA Rubin platform official 2026 specifications\""
    );
    assert_eq!(
        rendered.reasoning.as_deref(),
        Some(
            "正在执行：hone/web_search query=\"NVIDIA Rubin platform official 2026 specifications\""
        )
    );
    assert!(!rendered.tool.contains("本地命令"));
}

#[test]
fn codex_execute_renderer_never_exposes_shell_arguments_or_secrets() {
    let rendered = render_codex_tool_status(
        &serde_json::json!({
            "kind": "execute",
            "rawInput": {
                "command": "OPENROUTER_API_KEY=super-secret curl https://example.test/private?token=secret"
            }
        }),
        AcpToolRenderPhase::Start,
        "Run curl",
        None,
        None,
    );

    assert_eq!(rendered.tool, "请求接口（curl）");
    let reasoning = rendered.reasoning.expect("safe command reasoning");
    assert_eq!(reasoning, "正在执行：请求接口（curl）");
    assert!(!reasoning.contains("super-secret"));
    assert!(!reasoning.contains("example.test"));
    assert!(!reasoning.contains("token="));
}

/// Captured from the codex-acp `1.1.7` execute-completion stream shape.
/// `rawOutput` is adapter-only detail, so preserve it without imposing the
/// same event shape on OpenCode.
#[tokio::test]
async fn codex_acp_1_1_7_execute_stream_rehydrates_raw_tool_result() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/acp/codex-acp-1.1.7.json"
    ))
    .expect("Codex ACP 1.1.7 fixture");
    assert_eq!(fixture["adapter"]["captured_at"], "2026-08-02");
    let (_, profile) = select_acp_adapter_profile_from_initialize(
        AcpAdapterKind::CodexAcp,
        &fixture["initialize_result"],
    )
    .expect("Codex ACP fixture profile");
    assert_eq!(profile.dialect, AcpStreamDialect::CodexAcp1_1_7);
    let emitter: Arc<dyn AgentRunnerEmitter> = Arc::new(NoopEmitter);
    let mut state = AcpPromptState::default();
    let updates = fixture["execute_updates"]
        .as_array()
        .expect("Codex execute updates");
    handle_acp_session_update(&updates[0], &emitter, Some(&mut state)).await;
    let patched = patch_codex_session_update_params(&updates[1]).expect("patched params");
    handle_acp_session_update(&patched, &emitter, Some(&mut state)).await;

    let messages = finalize_context_messages(&mut state);
    assert_eq!(messages.len(), 2);
    let tool_calls = messages[0]
        .tool_calls
        .as_ref()
        .expect("assistant tool call");
    assert_eq!(tool_calls[0]["id"], "call_exec_1");
    assert_eq!(messages[1].role, "tool");
    assert_eq!(messages[1].tool_call_id.as_deref(), Some("call_exec_1"));
    let tool_content = messages[1].content.as_deref().expect("tool content");
    assert_contains_all(
        tool_content,
        &["\"stdout\":\"rtk 0.35.0\\n\"", "\"exit_code\":0"],
    );
}

/// Captured from a real `opencode acp` 1.18.11 exchange on 2026-08-01.
///
/// This is an external protocol fixture, not a cross-runner rendering contract:
/// OpenCode exposes thought chunks and detailed usage fields that codex-acp may
/// not expose in the same shape. Keep every detail that can be mapped safely.
#[tokio::test]
async fn opencode_1_18_11_stream_preserves_available_reasoning_answer_and_usage() {
    let runner = super::OpencodeAcpRunner::new(
        OpencodeAcpConfig::default(),
        RunnerTimeouts {
            step: Duration::from_secs(5),
            overall: Duration::from_secs(10),
        },
    );
    assert_eq!(runner.acp_adapter_kind(), Some(AcpAdapterKind::OpenCode));
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/acp/opencode-1.18.11.json"
    ))
    .expect("OpenCode 1.18.11 fixture");
    assert_eq!(fixture["adapter"]["captured_at"], "2026-08-01");
    let (_, profile) = select_acp_adapter_profile_from_initialize(
        AcpAdapterKind::OpenCode,
        &fixture["initialize_result"],
    )
    .expect("OpenCode 1.18.11 profile");
    assert_eq!(profile.dialect, AcpStreamDialect::OpenCode1_18_11);
    assert_eq!(profile.adapter.as_str(), "opencode");
    assert_eq!(profile.detected_version, "1.18.11");
    assert_eq!(profile.compatibility, AcpCompatibilityStatus::Validated);

    let emitter = Arc::new(CaptureEmitter::default());
    let emitter_trait: Arc<dyn AgentRunnerEmitter> = emitter.clone();
    let mut state = AcpPromptState::default();

    for update in fixture["updates"]
        .as_array()
        .expect("OpenCode stream updates")
    {
        handle_opencode_session_update(&update, &emitter_trait, &mut state).await;
    }

    assert_eq!(state.full_reply, "OPENCODE_ACP_OK");
    let events = emitter.events.lock().expect("events lock");
    assert!(events.iter().any(|event| matches!(
        event,
        AgentRunnerEvent::StreamThought { thought } if thought == "reasoning detail"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentRunnerEvent::StreamDelta { content } if content == "OPENCODE_A"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentRunnerEvent::StreamDelta { content } if content == "CP_OK"
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentRunnerEvent::Progress { stage, detail }
            if *stage == "opencode.usage" && detail.as_deref() == Some("used=8505")
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        AgentRunnerEvent::Progress { stage, detail }
            if *stage == "opencode.usage_detail"
                && detail.as_deref() == Some(
                    "used=8505 size=1000000 cost=0.00025996 currency=USD"
                )
    )));
}

/// Prompt-role contract paired with the observed OpenCode ACP 1.18.11 fixture
/// captured on 2026-08-01. Unlike native Codex, OpenCode starts a fresh ACP
/// session and receives delivered pushes as an assistant/context transcript
/// record; the current user turn remains byte-for-byte unchanged.
#[test]
fn opencode_1_18_11_prompt_keeps_delivered_push_out_of_current_user_input() {
    let current_user_turn =
        "【当前时间】\n2026-08-03 11:00:00 (北京时间)\n\n【本轮用户输入】\nTURN_CURRENT";
    let conversation = RunnerConversationInput::prepare(
        AgentConversationStrategy::EphemeralCompiledPrompt,
        "SYSTEM_LAYER".to_string(),
        current_user_turn.to_string(),
        AgentContext::new("opencode-1.18.11-context".to_string()),
        DeliveredPushContextBatch {
            records: vec![DeliveredPushContext {
                delivery_log_id: 11,
                source_id: "push-11".to_string(),
                delivered_at_ms: 1_775_360_400_000,
                body: "OPENCODE_PUSH_CONTEXT".to_string(),
            }],
            remaining_count: 0,
        },
    );
    let (system_prompt, projected_user_turn, context) = conversation
        .replay_parts()
        .expect("OpenCode replay conversation");
    assert_eq!(projected_user_turn, current_user_turn);
    assert_eq!(context.messages.len(), 1);
    assert_eq!(context.messages[0].role, "assistant");
    assert_eq!(
        context.messages[0]
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("subtype"))
            .and_then(Value::as_str),
        Some("delivered_push_context")
    );

    let prompt = build_opencode_acp_prompt_text(system_prompt, projected_user_turn, Some(context));
    let push_pos = prompt
        .find("OPENCODE_PUSH_CONTEXT")
        .expect("push in restored assistant context");
    let user_pos = prompt
        .find("### User Input ###")
        .expect("OpenCode current user section");
    assert!(push_pos < user_pos);
    let user_section = &prompt[user_pos..];
    assert!(user_section.contains("TURN_CURRENT"));
    assert!(!user_section.contains("OPENCODE_PUSH_CONTEXT"));
}

#[tokio::test]
async fn opencode_updates_preserve_tool_names_and_raw_io_in_transcript() {
    let emitter: Arc<dyn AgentRunnerEmitter> = Arc::new(NoopEmitter);
    let mut state = AcpPromptState::default();

    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "agent_message_chunk",
                "content": { "type": "text", "text": "我先检查本地目录。" }
            }
        }),
        &emitter,
        &mut state,
    )
    .await;
    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call_read_1",
                "title": "read",
                "kind": "read",
                "status": "pending",
                "rawInput": {}
            }
        }),
        &emitter,
        &mut state,
    )
    .await;
    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "call_read_1",
                "status": "completed",
                "kind": "read",
                "title": "/tmp/demo/uploads",
                "rawInput": { "filePath": "/tmp/demo/uploads" },
                "rawOutput": {
                    "output": "<entries>(0 entries)</entries>"
                }
            }
        }),
        &emitter,
        &mut state,
    )
    .await;
    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call_grep_1",
                "title": "grep",
                "kind": "search",
                "status": "pending",
                "rawInput": {}
            }
        }),
        &emitter,
        &mut state,
    )
    .await;
    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "call_grep_1",
                "status": "completed",
                "kind": "search",
                "title": "AAOI|COHR",
                "rawInput": {
                    "pattern": "AAOI|COHR",
                    "path": "/tmp/demo"
                },
                "rawOutput": {
                    "output": "No files found"
                }
            }
        }),
        &emitter,
        &mut state,
    )
    .await;

    let messages = finalize_context_messages(&mut state);
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0].role, "assistant");
    let tool_call = assert_single_tool_call(&messages[0], "assistant tool calls");
    assert_eq!(tool_call["function"]["name"], "read");
    assert_eq!(
        tool_call["function"]["arguments"],
        "{\"filePath\":\"/tmp/demo/uploads\"}"
    );
    assert_eq!(messages[1].role, "tool");
    assert_eq!(messages[1].name.as_deref(), Some("read"));
    assert_eq!(messages[1].tool_call_id.as_deref(), Some("call_read_1"));
    assert_eq!(
        messages[1].content.as_deref(),
        Some("<entries>(0 entries)</entries>")
    );
    assert_eq!(messages[2].role, "assistant");
    let grep_tool_call = assert_single_tool_call(&messages[2], "grep tool call");
    assert_eq!(grep_tool_call["function"]["name"], "grep");
    assert_eq!(
        grep_tool_call["function"]["arguments"],
        "{\"path\":\"/tmp/demo\",\"pattern\":\"AAOI|COHR\"}"
    );
    assert_eq!(messages[3].role, "tool");
    assert_eq!(messages[3].name.as_deref(), Some("grep"));
    assert_eq!(messages[3].tool_call_id.as_deref(), Some("call_grep_1"));
    assert_eq!(messages[3].content.as_deref(), Some("No files found"));
}

#[tokio::test]
async fn opencode_tool_status_uses_rendered_labels_from_raw_input() {
    let emitter = Arc::new(CaptureEmitter::default());
    let emitter_trait: Arc<dyn AgentRunnerEmitter> = emitter.clone();
    let mut state = AcpPromptState::default();

    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call_read_1",
                "title": "read",
                "kind": "read",
                "status": "pending",
                "rawInput": { "filePath": "/private/tmp/hone-agent-sandboxes/telegram/direct__8039067465/uploads" }
            }
        }),
        &emitter_trait,
        &mut state,
    )
    .await;
    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "call_read_1",
                "status": "completed",
                "kind": "read",
                "title": "read",
                "rawInput": { "filePath": "/private/tmp/hone-agent-sandboxes/telegram/direct__8039067465/uploads" },
                "rawOutput": { "output": "(empty)" }
            }
        }),
        &emitter_trait,
        &mut state,
    )
    .await;
    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call_grep_1",
                "title": "grep",
                "kind": "search",
                "status": "pending",
                "rawInput": {
                    "pattern": "AAOI|COHR",
                    "path": "/private/tmp/hone-agent-sandboxes/telegram/direct__8039067465/runtime"
                }
            }
        }),
        &emitter_trait,
        &mut state,
    )
    .await;

    assert_eq!(
        emitter.tool_events(),
        vec![
            CapturedToolEvent {
                tool: "read uploads".to_string(),
                status: "start".to_string(),
                message: None,
                reasoning: Some("正在执行：read uploads".to_string()),
            },
            CapturedToolEvent {
                tool: "read uploads".to_string(),
                status: "done".to_string(),
                message: Some("执行完成：read uploads".to_string()),
                reasoning: None,
            },
            CapturedToolEvent {
                tool: "grep \"AAOI|COHR\" in runtime".to_string(),
                status: "start".to_string(),
                message: None,
                reasoning: Some("正在执行：grep \"AAOI|COHR\" in runtime".to_string()),
            },
        ]
    );
}

#[tokio::test]
async fn opencode_tool_status_labels_workspace_root_explicitly() {
    let emitter = Arc::new(CaptureEmitter::default());
    let emitter_trait: Arc<dyn AgentRunnerEmitter> = emitter.clone();
    let mut state = AcpPromptState::default();

    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call_read_root",
                "title": "read",
                "kind": "read",
                "status": "pending",
                "rawInput": {
                    "filePath": "/private/tmp/hone-agent-sandboxes/telegram/direct__8039067465"
                }
            }
        }),
        &emitter_trait,
        &mut state,
    )
    .await;
    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "call_grep_root",
                "title": "grep",
                "kind": "search",
                "status": "in_progress",
                "rawInput": {
                    "pattern": "AAOI|COHR",
                    "path": "/private/tmp/hone-agent-sandboxes/telegram/direct__8039067465"
                }
            }
        }),
        &emitter_trait,
        &mut state,
    )
    .await;

    assert_eq!(
        emitter.tool_events(),
        vec![
            CapturedToolEvent {
                tool: "read workspace root".to_string(),
                status: "start".to_string(),
                message: None,
                reasoning: Some("正在执行：read workspace root".to_string()),
            },
            CapturedToolEvent {
                tool: "grep \"AAOI|COHR\" in workspace root".to_string(),
                status: "start".to_string(),
                message: None,
                reasoning: Some("正在执行：grep \"AAOI|COHR\" in workspace root".to_string()),
            },
        ]
    );
}

#[tokio::test]
async fn opencode_tool_status_redacts_secret_values_in_labels() {
    let emitter = Arc::new(CaptureEmitter::default());
    let emitter_trait: Arc<dyn AgentRunnerEmitter> = emitter.clone();
    let mut state = AcpPromptState::default();

    handle_opencode_session_update(
        &serde_json::json!({
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call_grep_secret",
                "title": "grep",
                "kind": "search",
                "status": "pending",
                "rawInput": {
                    "pattern": "token=pattern-secret auth=Bearer bearer-secret",
                    "path": "/tmp/runtime?api_key=path-secret",
                    "purpose": "check apiKey: header-secret"
                }
            }
        }),
        &emitter_trait,
        &mut state,
    )
    .await;

    let events = emitter.tool_events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert!(event.tool.contains("token=<redacted>"));
    assert!(event.tool.contains("Bearer <redacted>"));
    assert!(event.tool.contains("api_key=<redacted>"));
    assert!(event.tool.contains("apiKey: <redacted>"));
    assert!(!event.tool.contains("pattern-secret"));
    assert!(!event.tool.contains("bearer-secret"));
    assert!(!event.tool.contains("path-secret"));
    assert!(!event.tool.contains("header-secret"));
    let expected_reasoning = format!("正在执行：{}", event.tool);
    assert_eq!(
        event.reasoning.as_deref(),
        Some(expected_reasoning.as_str())
    );
}

#[test]
fn opencode_prompt_text_includes_restored_transcript_for_fresh_sessions() {
    let mut context = AgentContext::new("session-1".to_string());
    context.add_user_message("先看本地目录");
    context.messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some("我先检查 runtime。".to_string()),
        tool_calls: Some(vec![serde_json::json!({
            "id": "call_read_1",
            "type": "function",
            "function": {
                "name": "read",
                "arguments": "{\"filePath\":\"/tmp/demo/runtime\"}"
            }
        })]),
        tool_call_id: None,
        name: None,
        metadata: None,
    });
    context.messages.push(AgentMessage {
        role: "tool".to_string(),
        content: Some("<entries>(0 entries)</entries>".to_string()),
        tool_calls: None,
        tool_call_id: Some("call_read_1".to_string()),
        name: Some("read".to_string()),
        metadata: None,
    });
    context.add_assistant_message("runtime 目录是空的。", None);

    let prompt = build_opencode_acp_prompt_text("SYSTEM", "新的问题", Some(&context));
    assert_contains_all(
        &prompt,
        &[
            "### Restored Conversation Transcript ###",
            "\"role\": \"assistant\"",
            "\"type\": \"tool_call\"",
            "\"type\": \"tool_result\"",
            "\"type\": \"final\"",
        ],
    );
    assert!(!prompt.contains("\"role\": \"tool\""));
    assert_contains_all(
        &prompt,
        &["我先检查 runtime。", "### User Input ###\n新的问题"],
    );
}
