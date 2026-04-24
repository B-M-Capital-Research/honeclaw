use async_trait::async_trait;
use hone_core::agent::{
    AgentContext, AgentMessage, ToolCallMade, final_assistant_message_content,
    normalize_agent_messages,
};
use hone_core::config::{CodexAcpConfig, GeminiAcpConfig, OpencodeAcpConfig};
use hone_memory::restore_tool_message;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use super::acp_common::{
    AcpPromptState, AcpToolRenderPhase, CliVersion, extract_finished_tool_calls,
    finalize_context_messages, handle_acp_session_update, parse_cli_version,
    summarize_finished_tool_calls_for_log,
};
use super::codex_acp::{
    build_codex_acp_prompt_text, codex_acp_effective_args, configured_codex_model_id,
    configured_codex_reasoning_effort, patch_codex_session_update_params, render_codex_tool_status,
    validate_codex_version_matrix,
};
use super::gemini_acp::{
    configured_gemini_api_key_env, gemini_acp_effective_args, validate_gemini_version,
};
use super::gemini_cli::{
    GeminiCliToolRenderPhase, append_gemini_cli_tool_context_messages,
    render_gemini_cli_tool_status,
};
use super::opencode_acp::{
    build_opencode_acp_prompt_text, configured_opencode_model_id, effective_opencode_args,
    handle_opencode_session_update, isolated_opencode_config, resolve_command_path_with_env,
};
use super::tool_reasoning::{render_runner_tool_label, runner_context_messages};
use super::types::{AgentRunnerEmitter, AgentRunnerEvent};
use uuid::Uuid;

struct NoopEmitter;

#[async_trait]
impl AgentRunnerEmitter for NoopEmitter {
    async fn emit(&self, _event: AgentRunnerEvent) {}
}

#[test]
fn configured_model_id_appends_variant() {
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
fn configured_model_id_does_not_duplicate_variant_suffix() {
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
fn configured_codex_model_id_omits_variant_suffix() {
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
fn codex_acp_effective_args_include_reasoning_effort() {
    let config = CodexAcpConfig {
        variant: "high".to_string(),
        ..CodexAcpConfig::default()
    };
    let args = codex_acp_effective_args(&config, true);
    assert!(
        args.windows(2)
            .any(|w| w[0] == "-c" && w[1] == "model_reasoning_effort=\"high\""),
        "expected reasoning effort override in args, got: {args:?}"
    );
}

#[test]
fn codex_acp_effective_args_include_dangerous_bypass_overrides() {
    let config = CodexAcpConfig {
        dangerously_bypass_approvals_and_sandbox: true,
        sandbox_permissions: vec!["disk-full-read-access".to_string()],
        extra_config_overrides: vec!["shell_environment_policy.inherit=all".to_string()],
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        codex_acp_effective_args(&config, false),
        vec![
            "-c",
            "sandbox_mode=\"danger-full-access\"",
            "-c",
            "approval_policy=\"never\"",
            "-c",
            "sandbox_permissions=[\"disk-full-read-access\"]",
            "-c",
            "shell_environment_policy.inherit=all",
        ]
    );
}

#[test]
fn codex_acp_effective_args_lock_down_workspace_and_ignore_dangerous_bypass() {
    let config = CodexAcpConfig {
        dangerously_bypass_approvals_and_sandbox: true,
        sandbox_permissions: vec!["disk-full-read-access".to_string()],
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        codex_acp_effective_args(&config, true),
        vec![
            "-c",
            "sandbox_mode=\"workspace-write\"",
            "-c",
            "approval_policy=\"never\"",
        ]
    );
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
    assert_eq!(rendered.message, None);
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
    assert_eq!(done.reasoning, None);
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
    let tool_calls = messages[0]
        .tool_calls
        .as_ref()
        .expect("assistant tool calls");
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0]["id"], "gemini_cli_call_1_1");
    assert_eq!(tool_calls[0]["function"]["name"], "web_search");
    assert_eq!(
        tool_calls[0]["function"]["arguments"],
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
    assert_eq!(normalized[0].content[0].part_type, "progress");
    assert_eq!(normalized[0].content[1].part_type, "tool_call");
    assert_eq!(normalized[0].content[2].part_type, "tool_result");
    assert_eq!(normalized[0].content[3].part_type, "final");
}

#[test]
fn codex_version_matrix_accepts_minimum_validated_pair() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 115,
            patch: 0,
        },
        CliVersion {
            major: 0,
            minor: 9,
            patch: 5,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn codex_version_matrix_accepts_newer_adapter() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 115,
            patch: 0,
        },
        CliVersion {
            major: 0,
            minor: 11,
            patch: 1,
        },
    );
    assert!(result.is_ok());
}

#[test]
fn codex_version_matrix_rejects_old_codex() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 105,
            patch: 0,
        },
        CliVersion {
            major: 0,
            minor: 9,
            patch: 5,
        },
    );
    assert!(
        result
            .unwrap_err()
            .contains("npm install -g @openai/codex@latest")
    );
}

#[test]
fn codex_version_matrix_rejects_old_adapter() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 115,
            patch: 0,
        },
        CliVersion {
            major: 0,
            minor: 9,
            patch: 4,
        },
    );
    assert!(
        result
            .unwrap_err()
            .contains("@zed-industries/codex-acp@latest")
    );
}

#[test]
fn gemini_version_guard_rejects_old_binary() {
    let result = validate_gemini_version(CliVersion {
        major: 0,
        minor: 29,
        patch: 0,
    });
    assert!(result.unwrap_err().contains("@google/gemini-cli@latest"));
}

#[test]
fn gemini_api_key_env_defaults_to_standard_name() {
    let config = GeminiAcpConfig::default();
    assert_eq!(configured_gemini_api_key_env(&config), "GEMINI_API_KEY");
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
    assert_eq!(calls[0].result["ok"], true);
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
    assert!(summary.contains("count=2"));
    assert!(summary.contains("data_fetch#call_2"));
    assert!(summary.contains("web_search#call_1"));
    assert!(!summary.contains("AAOI"));
    assert!(!summary.contains("COHR"));
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
    assert_eq!(messages[0].tool_calls, None);
    let tool_calls = messages[1]
        .tool_calls
        .as_ref()
        .expect("assistant tool calls");
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0]["id"], "call_1");
    assert_eq!(tool_calls[0]["function"]["name"], "local_search_files");
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
fn codex_prompt_text_includes_restored_transcript_when_session_is_recreated() {
    let mut context = AgentContext::new("session-1".to_string());
    context.add_user_message("AAOI 是什么公司");
    context.add_assistant_message("我先查本地画像。", None);

    let prompt = build_codex_acp_prompt_text("SYSTEM", "新的问题", Some(&context));
    assert!(prompt.contains("### Restored Conversation Transcript ###"));
    assert!(prompt.contains("\"role\": \"user\""));
    assert!(prompt.contains("AAOI 是什么公司"));
    assert!(prompt.contains("### User Input ###\n新的问题"));
}

#[test]
fn codex_prompt_text_serializes_message_metadata() {
    let mut context = AgentContext::new("session-1".to_string());
    context.messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some("我先核验本地画像。".to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
        metadata: Some(HashMap::from([(
            "codex_acp".to_string(),
            serde_json::json!({
                "segment_kind": "progress_note",
                "channel_fields": {
                    "stream_kind": "agent_message_chunk"
                }
            }),
        )])),
    });

    let prompt = build_codex_acp_prompt_text("SYSTEM", "新的问题", Some(&context));
    assert!(prompt.contains("\"metadata\""));
    assert!(prompt.contains("\"codex_acp\""));
    assert!(prompt.contains("\"segment_kind\": \"progress_note\""));
    assert!(prompt.contains("\"stream_kind\": \"agent_message_chunk\""));
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
    assert_eq!(history[1].content.len(), 4);
    assert_eq!(history[1].content[0].part_type, "progress");
    assert_eq!(history[1].content[1].part_type, "tool_call");
    assert_eq!(history[1].content[1].name.as_deref(), Some("web_search"));
    assert_eq!(
        history[1].content[1].args,
        Some(serde_json::json!({"query":"FLNC stock price"}))
    );
    assert_eq!(history[1].content[2].part_type, "tool_result");
    assert_eq!(
        history[1].content[2].result,
        Some(serde_json::json!({"price":5.12}))
    );
    assert_eq!(history[1].content[3].part_type, "final");
    assert_eq!(
        history[1].content[3].text.as_deref(),
        Some("结论：先看订单兑现，再谈估值弹性。")
    );
}

#[test]
fn codex_prompt_text_uses_normalized_user_assistant_history() {
    let mut context = AgentContext::new("session-1".to_string());
    context.add_user_message("AAOI 是什么公司");
    context.messages.push(AgentMessage {
        role: "assistant".to_string(),
        content: Some("我先查本地画像。".to_string()),
        tool_calls: Some(vec![serde_json::json!({
            "id": "call_1",
            "type": "function",
            "function": {
                "name": "local_search_files",
                "arguments": "{\"query\":\"AAOI\"}"
            }
        })]),
        tool_call_id: None,
        name: None,
        metadata: None,
    });
    context.messages.push(AgentMessage {
        role: "tool".to_string(),
        content: Some("{\"matches\":[\"company_profiles/aaoi.md\"]}".to_string()),
        tool_calls: None,
        tool_call_id: Some("call_1".to_string()),
        name: Some("local_search_files".to_string()),
        metadata: None,
    });
    context.add_assistant_message("AAOI 是做光模块的。", None);

    let prompt = build_codex_acp_prompt_text("SYSTEM", "新的问题", Some(&context));
    assert!(prompt.contains("\"role\": \"assistant\""));
    assert!(prompt.contains("\"type\": \"tool_call\""));
    assert!(prompt.contains("\"type\": \"tool_result\""));
    assert!(prompt.contains("\"type\": \"final\""));
    assert!(!prompt.contains("\"role\": \"tool\""));
}

#[test]
fn codex_and_opencode_prompt_transcripts_share_the_same_normalized_history() {
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

    let codex_prompt = build_codex_acp_prompt_text("SYSTEM", "新的问题", Some(&context));
    let opencode_prompt = build_opencode_acp_prompt_text("SYSTEM", "新的问题", Some(&context));

    let codex_marker = "```json\n";
    let codex_start = codex_prompt
        .find(codex_marker)
        .expect("codex transcript start")
        + codex_marker.len();
    let codex_end = codex_prompt[codex_start..]
        .find("\n```")
        .expect("codex transcript end")
        + codex_start;
    let opencode_start = opencode_prompt
        .find(codex_marker)
        .expect("opencode transcript start")
        + codex_marker.len();
    let opencode_end = opencode_prompt[opencode_start..]
        .find("\n```")
        .expect("opencode transcript end")
        + opencode_start;

    assert_eq!(
        &codex_prompt[codex_start..codex_end],
        &opencode_prompt[opencode_start..opencode_end]
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
fn codex_execute_renderer_truncates_long_command_and_appends_purpose() {
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

    assert!(rendered.tool.contains("[truncated,"));
    assert!(rendered.tool.starts_with("python - <<'PY'"));
    assert_eq!(rendered.message, None);
    assert!(
        rendered
            .reasoning
            .as_deref()
            .is_some_and(|value| value.starts_with("正在执行：python - <<'PY'"))
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

    assert_eq!(rendered.tool, "rtk ls -la uploads");
    assert_eq!(
        rendered.message.as_deref(),
        Some("执行完成：rtk ls -la uploads")
    );
    assert_eq!(rendered.reasoning, None);
}

#[tokio::test]
async fn codex_execute_completed_update_rehydrates_tool_result_from_raw_output() {
    let emitter: Arc<dyn AgentRunnerEmitter> = Arc::new(NoopEmitter);
    let mut state = AcpPromptState::default();

    let start = serde_json::json!({
        "update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "call_exec_1",
            "title": "Run rtk --version",
            "kind": "execute",
            "rawInput": {
                "command": ["/bin/zsh", "-lc", "rtk --version"]
            }
        }
    });
    handle_acp_session_update(&start, &emitter, Some(&mut state)).await;

    let completed = serde_json::json!({
        "update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "call_exec_1",
            "status": "completed",
            "kind": "execute",
            "rawOutput": {
                "stdout": "rtk 0.35.0\n",
                "formatted_output": "rtk 0.35.0\n",
                "exit_code": 0
            }
        }
    });
    let patched = patch_codex_session_update_params(&completed).expect("patched params");
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
    assert!(tool_content.contains("\"stdout\":\"rtk 0.35.0\\n\""));
    assert!(tool_content.contains("\"exit_code\":0"));
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
    let tool_calls = messages[0]
        .tool_calls
        .as_ref()
        .expect("assistant tool calls");
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0]["function"]["name"], "read");
    assert_eq!(
        tool_calls[0]["function"]["arguments"],
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
    let grep_tool_calls = messages[2].tool_calls.as_ref().expect("grep tool call");
    assert_eq!(grep_tool_calls.len(), 1);
    assert_eq!(grep_tool_calls[0]["function"]["name"], "grep");
    assert_eq!(
        grep_tool_calls[0]["function"]["arguments"],
        "{\"path\":\"/tmp/demo\",\"pattern\":\"AAOI|COHR\"}"
    );
    assert_eq!(messages[3].role, "tool");
    assert_eq!(messages[3].name.as_deref(), Some("grep"));
    assert_eq!(messages[3].tool_call_id.as_deref(), Some("call_grep_1"));
    assert_eq!(messages[3].content.as_deref(), Some("No files found"));
}

#[tokio::test]
async fn opencode_tool_status_uses_rendered_labels_from_raw_input() {
    use std::sync::Mutex;

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

    let events = emitter.events.lock().expect("events lock");
    let tool_events = events
        .iter()
        .filter_map(|event| match event {
            AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => Some((
                tool.clone(),
                status.clone(),
                message.clone(),
                reasoning.clone(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        tool_events[0],
        (
            "read uploads".to_string(),
            "start".to_string(),
            None,
            Some("正在执行：read uploads".to_string()),
        )
    );
    assert_eq!(
        tool_events[1],
        (
            "read uploads".to_string(),
            "done".to_string(),
            Some("执行完成：read uploads".to_string()),
            None,
        )
    );
    assert_eq!(
        tool_events[2],
        (
            "grep \"AAOI|COHR\" in runtime".to_string(),
            "start".to_string(),
            None,
            Some("正在执行：grep \"AAOI|COHR\" in runtime".to_string()),
        )
    );
}

#[tokio::test]
async fn opencode_tool_status_labels_workspace_root_explicitly() {
    use std::sync::Mutex;

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

    let events = emitter.events.lock().expect("events lock");
    let tool_events = events
        .iter()
        .filter_map(|event| match event {
            AgentRunnerEvent::ToolStatus {
                tool,
                status,
                message,
                reasoning,
            } => Some((
                tool.clone(),
                status.clone(),
                message.clone(),
                reasoning.clone(),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        tool_events[0],
        (
            "read workspace root".to_string(),
            "start".to_string(),
            None,
            Some("正在执行：read workspace root".to_string()),
        )
    );
    assert_eq!(
        tool_events[1],
        (
            "grep \"AAOI|COHR\" in workspace root".to_string(),
            "start".to_string(),
            None,
            Some("正在执行：grep \"AAOI|COHR\" in workspace root".to_string()),
        )
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
    assert!(prompt.contains("### Restored Conversation Transcript ###"));
    assert!(prompt.contains("\"role\": \"assistant\""));
    assert!(prompt.contains("\"type\": \"tool_call\""));
    assert!(prompt.contains("\"type\": \"tool_result\""));
    assert!(prompt.contains("\"type\": \"final\""));
    assert!(!prompt.contains("\"role\": \"tool\""));
    assert!(prompt.contains("我先检查 runtime。"));
    assert!(prompt.contains("### User Input ###\n新的问题"));
}
