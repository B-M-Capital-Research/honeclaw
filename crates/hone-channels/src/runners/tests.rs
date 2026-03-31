use hone_core::agent::ToolCallMade;
use hone_core::config::{CodexAcpConfig, GeminiAcpConfig, OpencodeAcpConfig};
use hone_memory::restore_tool_message;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use super::acp_common::{
    AcpPromptState, CliVersion, extract_finished_tool_calls, parse_cli_version,
};
use super::codex_acp::{
    codex_acp_effective_args, configured_codex_model_id, validate_codex_version_matrix,
};
use super::gemini_acp::{
    configured_gemini_api_key_env, gemini_acp_effective_args, validate_gemini_version,
};
use super::opencode_acp::{
    configured_opencode_model_id, effective_opencode_args, isolated_opencode_config,
    resolve_command_path_with_env,
};
use uuid::Uuid;

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
fn configured_codex_model_id_appends_variant() {
    let config = CodexAcpConfig {
        model: "gpt-5.4".to_string(),
        variant: "medium".to_string(),
        ..CodexAcpConfig::default()
    };
    assert_eq!(
        configured_codex_model_id(&config).as_deref(),
        Some("gpt-5.4/medium")
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
fn codex_version_matrix_accepts_validated_pair() {
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
fn codex_version_matrix_rejects_unvalidated_adapter() {
    let result = validate_codex_version_matrix(
        CliVersion {
            major: 0,
            minor: 115,
            patch: 0,
        },
        CliVersion {
            major: 0,
            minor: 10,
            patch: 0,
        },
    );
    assert!(
        result
            .unwrap_err()
            .contains("@zed-industries/codex-acp@0.9.5")
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
    let restored =
        restore_tool_message("{\"result\":true}", Some(&metadata)).expect("tool message");
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
