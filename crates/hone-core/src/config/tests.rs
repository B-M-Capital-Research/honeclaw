use super::*;
use crate::config::ChatScope;
use serde_yaml::Value;
use std::path::{Path, PathBuf};

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("hone-config-{}-{}", prefix, uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_default_config() {
    let config = HoneConfig::default();
    assert_eq!(config.llm.provider, "openrouter");
    assert_eq!(config.llm.openrouter.model, "moonshotai/kimi-k2.5");
    assert_eq!(config.llm.openrouter.sub_model, "moonshotai/kimi-k2.5");
    assert_eq!(config.llm.auxiliary.api_key_env, "MINIMAX_API_KEY");
    assert!(config.llm.auxiliary.base_url.is_empty());
    assert_eq!(config.llm.openrouter.timeout, 120);
    assert_eq!(config.llm.openrouter.max_tokens, 32768);
}

#[test]
fn test_deserialize_minimal_yaml() {
    let yaml = r#"
llm:
  provider: openrouter
  openrouter:
    model: "test-model"
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.llm.openrouter.model, "test-model");
    assert_eq!(config.llm.openrouter.sub_model, "moonshotai/kimi-k2.5");
    assert!(config.llm.auxiliary.model.is_empty());
    assert_eq!(config.llm.openrouter.timeout, 120); // default
}

#[test]
fn test_runtime_overlay_path() {
    let path = Path::new("/tmp/config.yaml");
    let overlay = runtime_overlay_path(path);
    assert_eq!(overlay, PathBuf::from("/tmp/config.overrides.yaml"));
}

#[test]
fn test_merge_yaml_value_recursively() {
    let mut base: Value = serde_yaml::from_str(
        r#"
imessage:
  enabled: false
  target_handle: ""
  poll_interval: 2
search:
  api_keys:
    - base-a
    - base-b
logging:
  file: "./data/logs/hone.log"
custom_section:
  nested:
    keep: base
"#,
    )
    .unwrap();
    let overlay: Value = serde_yaml::from_str(
        r#"
imessage:
  enabled: true
search:
  api_keys:
    - override-a
custom_section:
  nested:
    keep: overlay
new_section:
  flag: true
"#,
    )
    .unwrap();

    merge_yaml_value(&mut base, overlay);
    let config: HoneConfig = serde_yaml::from_value(base).unwrap();

    assert!(config.imessage.enabled);
    assert_eq!(config.search.api_keys, vec!["override-a".to_string()]);
    assert_eq!(config.logging.file.as_deref(), Some("./data/logs/hone.log"));
    assert_eq!(
        config
            .extra
            .get("custom_section")
            .and_then(|v| v.as_mapping())
            .and_then(|m| m.get(Value::String("nested".to_string())))
            .and_then(|v| v.as_mapping())
            .and_then(|m| m.get(Value::String("keep".to_string())))
            .and_then(|v| v.as_str()),
        Some("overlay")
    );
    assert!(config.extra.contains_key("new_section"));
}

#[test]
fn test_read_merged_yaml_value_applies_runtime_overlay() {
    let dir = temp_test_dir("from-file");
    let config_path = dir.join("config.yaml");
    let overlay_path = runtime_overlay_path(&config_path);

    std::fs::write(
        &config_path,
        r#"
imessage:
  enabled: false
search:
  api_keys:
    - base-a
logging:
  file: "./data/logs/hone.log"
  udp_port: 9000
custom_section:
  nested:
    keep: base
"#,
    )
    .unwrap();
    std::fs::write(
        &overlay_path,
        r#"
imessage:
  enabled: true
search:
  api_keys:
    - override-a
    - override-b
logging:
  file: null
custom_section:
  nested:
    keep: overlay
"#,
    )
    .unwrap();

    let merged = read_merged_yaml_value(&config_path).unwrap();
    let config = HoneConfig::from_merged_value(merged).unwrap();
    assert!(config.imessage.enabled);
    assert_eq!(
        config.search.api_keys,
        vec!["override-a".to_string(), "override-b".to_string()]
    );
    assert_eq!(config.logging.file, None);
    assert_eq!(config.logging.udp_port, Some(9000));
    assert_eq!(
        config
            .extra
            .get("custom_section")
            .and_then(|v| v.as_mapping())
            .and_then(|m| m.get(Value::String("nested".to_string())))
            .and_then(|v| v.as_mapping())
            .and_then(|m| m.get(Value::String("keep".to_string())))
            .and_then(|v| v.as_str()),
        Some("overlay")
    );
}

#[test]
fn test_from_file_applies_runtime_overlay() {
    let dir = temp_test_dir("from-file-runtime-overlay");
    let config_path = dir.join("config.yaml");
    let overlay_path = runtime_overlay_path(&config_path);

    std::fs::write(
        &config_path,
        r#"
agent:
  runner: codex_cli
feishu:
  enabled: false
"#,
    )
    .unwrap();
    std::fs::write(
        &overlay_path,
        r#"
agent:
  runner: multi-agent
feishu:
  enabled: true
"#,
    )
    .unwrap();

    let config = HoneConfig::from_file(&config_path).unwrap();
    assert_eq!(config.agent.runner, "multi-agent");
    assert!(config.feishu.enabled);
}

#[test]
fn test_diff_yaml_value_keeps_only_changes() {
    let base: Value = serde_yaml::from_str(
        r#"
imessage:
  enabled: false
search:
  api_keys:
    - base-a
    - base-b
logging:
  file: "./data/logs/hone.log"
"#,
    )
    .unwrap();
    let current: Value = serde_yaml::from_str(
        r#"
imessage:
  enabled: true
search:
  api_keys:
    - override-a
logging:
  file: null
"#,
    )
    .unwrap();

    let patch = diff_yaml_value(&base, &current).expect("expected a patch");
    let patch_map = patch.as_mapping().expect("patch should be a mapping");
    assert!(patch_map.contains_key(Value::String("imessage".to_string())));
    assert!(patch_map.contains_key(Value::String("search".to_string())));
    assert!(patch_map.contains_key(Value::String("logging".to_string())));
    assert_eq!(patch_map.len(), 3);

    let logging = patch_map
        .get(Value::String("logging".to_string()))
        .and_then(|v| v.as_mapping())
        .expect("logging patch");
    assert!(matches!(
        logging.get(Value::String("file".to_string())),
        Some(Value::Null)
    ));

    let imessage = patch_map
        .get(Value::String("imessage".to_string()))
        .and_then(|v| v.as_mapping())
        .expect("imessage patch");
    assert!(matches!(
        imessage.get(Value::String("enabled".to_string())),
        Some(Value::Bool(true))
    ));

    let search = patch_map
        .get(Value::String("search".to_string()))
        .and_then(|v| v.as_mapping())
        .expect("search patch");
    assert_eq!(
        search.get(Value::String("api_keys".to_string())),
        Some(&Value::Sequence(vec![Value::String(
            "override-a".to_string()
        )]))
    );
}

#[test]
fn test_deserialize_agent_codex_model() {
    let yaml = r#"
agent:
  runner: codex_cli
  codex_model: "gpt-5.3-codex"
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.agent.runner, "codex_cli");
    assert_eq!(config.agent.codex_model, "gpt-5.3-codex");
}

#[test]
fn test_deserialize_agent_opencode_model_and_variant() {
    let yaml = r#"
agent:
  runner: opencode_acp
  opencode:
    model: "openrouter/openai/gpt-5.4"
    variant: "medium"
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.agent.runner, "opencode_acp");
    assert_eq!(config.agent.opencode.model, "openrouter/openai/gpt-5.4");
    assert_eq!(config.agent.opencode.variant, "medium");
}

#[test]
fn test_default_agent_opencode_inherits_local_config_when_unset() {
    let config = HoneConfig::default();
    assert!(config.agent.opencode.model.is_empty());
    assert!(config.agent.opencode.variant.is_empty());
    assert!(config.agent.opencode.api_base_url.is_empty());
    assert!(config.agent.opencode.api_key.is_empty());
    assert_eq!(
        config.agent.multi_agent.answer.api_base_url,
        "https://openrouter.ai/api/v1"
    );
}

#[test]
fn test_deserialize_agent_gemini_acp() {
    let yaml = r#"
agent:
  runner: gemini_acp
  gemini_acp:
    model: "gemini-2.5-pro"
    api_key_env: "GEMINI_API_KEY"
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.agent.runner, "gemini_acp");
    assert_eq!(config.agent.gemini_acp.model, "gemini-2.5-pro");
    assert_eq!(config.agent.gemini_acp.api_key_env, "GEMINI_API_KEY");
}

#[test]
fn test_deserialize_agent_codex_acp_sandbox_controls() {
    let yaml = r#"
agent:
  runner: codex_acp
  codex_acp:
    model: "gpt-5.4"
    variant: "medium"
    dangerously_bypass_approvals_and_sandbox: true
    sandbox_permissions: ["disk-full-read-access"]
    extra_config_overrides: ["shell_environment_policy.inherit=all"]
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.agent.runner, "codex_acp");
    assert!(
        config
            .agent
            .codex_acp
            .dangerously_bypass_approvals_and_sandbox
    );
    assert_eq!(
        config.agent.codex_acp.sandbox_permissions,
        vec!["disk-full-read-access"]
    );
    assert_eq!(
        config.agent.codex_acp.extra_config_overrides,
        vec!["shell_environment_policy.inherit=all"]
    );
}

#[test]
fn test_deserialize_agent_multi_agent() {
    let yaml = r#"
agent:
  runner: multi-agent
  multi_agent:
    search:
      base_url: "https://api.minimaxi.com/v1"
      api_key: "sk-cp-test"
      model: "MiniMax-M2.7-highspeed"
      max_iterations: 8
    answer:
      api_base_url: "https://openrouter.ai/api/v1"
      api_key: "sk-or-test"
      model: "google/gemini-3.1-pro-preview"
      variant: "high"
      max_tool_calls: 1
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.agent.runner, "multi-agent");
    assert_eq!(
        config.agent.multi_agent.search.base_url,
        "https://api.minimaxi.com/v1"
    );
    assert_eq!(config.agent.multi_agent.search.api_key, "sk-cp-test");
    assert_eq!(
        config.agent.multi_agent.search.model,
        "MiniMax-M2.7-highspeed"
    );
    assert_eq!(config.agent.multi_agent.search.max_iterations, 8);
    assert_eq!(
        config.agent.multi_agent.answer.api_base_url,
        "https://openrouter.ai/api/v1"
    );
    assert_eq!(config.agent.multi_agent.answer.api_key, "sk-or-test");
    assert_eq!(
        config.agent.multi_agent.answer.model,
        "google/gemini-3.1-pro-preview"
    );
    assert_eq!(config.agent.multi_agent.answer.variant, "high");
    assert_eq!(config.agent.multi_agent.answer.max_tool_calls, 1);
}

#[test]
fn test_deserialize_feishu_config() {
    let yaml = r#"
feishu:
  enabled: true
  app_id: "cli_test"
  app_secret: "secret"
  allow_emails: ["alice@example.com"]
  allow_mobiles: ["+8613800138000"]
  allow_open_ids: ["ou_abc"]
  chat_scope: GROUPCHAT_ONLY
  max_message_length: 2048
  facade_url: "http://127.0.0.1:19001/rpc"
  callback_addr: "127.0.0.1:19002"
  facade_addr: "127.0.0.1:19001"
  startup_timeout_seconds: 9
admins:
  telegram_user_ids: ["8039067465"]
  feishu_emails: ["admin@example.com"]
  feishu_mobiles: ["+8613900139000"]
  feishu_open_ids: ["ou_admin"]
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.feishu.enabled);
    assert_eq!(config.feishu.app_id, "cli_test");
    assert_eq!(config.feishu.app_secret, "secret");
    assert_eq!(config.feishu.allow_emails, vec!["alice@example.com"]);
    assert_eq!(config.feishu.allow_mobiles, vec!["+8613800138000"]);
    assert_eq!(config.feishu.allow_open_ids, vec!["ou_abc"]);
    assert_eq!(config.feishu.chat_scope, ChatScope::GroupchatOnly);
    assert_eq!(config.feishu.max_message_length, 2048);
    assert_eq!(config.feishu.facade_url, "http://127.0.0.1:19001/rpc");
    assert_eq!(config.feishu.callback_addr, "127.0.0.1:19002");
    assert_eq!(config.feishu.facade_addr, "127.0.0.1:19001");
    assert_eq!(config.feishu.startup_timeout_seconds, 9);
    assert_eq!(config.admins.telegram_user_ids, vec!["8039067465"]);
    assert_eq!(config.admins.feishu_emails, vec!["admin@example.com"]);
    assert_eq!(config.admins.feishu_mobiles, vec!["+8613900139000"]);
    assert_eq!(config.admins.feishu_open_ids, vec!["ou_admin"]);
}

#[test]
fn test_deserialize_discord_group_reply() {
    let yaml = r#"
group_context:
  pretrigger_window_enabled: false
  pretrigger_window_max_messages: 6
  pretrigger_window_max_age_seconds: 45
discord:
  group_reply:
    enabled: true
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(!config.group_context.pretrigger_window_enabled);
    assert_eq!(config.group_context.pretrigger_window_max_messages, 6);
    assert_eq!(config.group_context.pretrigger_window_max_age_seconds, 45);
    let gr = &config.discord.group_reply;
    assert!(gr.enabled);
}

#[test]
fn test_chat_scope_defaults_to_dm_only() {
    let config = HoneConfig::default();
    assert_eq!(config.feishu.chat_scope, ChatScope::DmOnly);
    assert_eq!(config.telegram.chat_scope, ChatScope::DmOnly);
    assert_eq!(config.discord.chat_scope, ChatScope::DmOnly);
}

#[test]
fn test_legacy_dm_only_false_maps_to_all() {
    let yaml = r#"
telegram:
  dm_only: false
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.telegram.chat_scope, ChatScope::All);
}

#[test]
fn test_chat_scope_overrides_legacy_dm_only() {
    let yaml = r#"
discord:
  chat_scope: GROUPCHAT_ONLY
  dm_only: true
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.discord.chat_scope, ChatScope::GroupchatOnly);
}

#[test]
fn test_read_config_path_value_supports_nested_mapping_and_sequence() {
    let dir = temp_test_dir("path-get");
    let config_path = dir.join("config.yaml");
    std::fs::write(
        &config_path,
        r#"
search:
  api_keys:
    - key-a
    - key-b
agent:
  runner: codex_cli
"#,
    )
    .unwrap();

    assert_eq!(
        read_config_path_value(&config_path, "agent.runner")
            .unwrap()
            .and_then(|value| value.as_str().map(ToString::to_string)),
        Some("codex_cli".to_string())
    );
    assert_eq!(
        read_config_path_value(&config_path, "search.api_keys[1]")
            .unwrap()
            .and_then(|value| value.as_str().map(ToString::to_string)),
        Some("key-b".to_string())
    );
    assert!(
        read_config_path_value(&config_path, "search.api_keys[3]")
            .unwrap()
            .is_none()
    );
}

#[test]
fn test_apply_config_mutations_updates_canonical_config_directly() {
    let dir = temp_test_dir("mutations");
    let config_path = dir.join("config.yaml");
    let overlay_path = runtime_overlay_path(&config_path);
    std::fs::write(
        &config_path,
        r#"
agent:
  runner: codex_cli
search:
  api_keys:
    - key-a
"#,
    )
    .unwrap();

    apply_config_mutations(
        &config_path,
        &[
            ConfigMutation::Set {
                path: "agent.runner".to_string(),
                value: Value::String("opencode_acp".to_string()),
            },
            ConfigMutation::Set {
                path: "search.api_keys[1]".to_string(),
                value: Value::String("key-b".to_string()),
            },
        ],
    )
    .unwrap();

    let base = std::fs::read_to_string(&config_path).unwrap();
    assert!(base.contains("opencode_acp"));
    assert!(base.contains("key-b"));
    assert!(!overlay_path.exists());

    let config = HoneConfig::from_file(&config_path).unwrap();
    assert_eq!(config.agent.runner, "opencode_acp");
    assert_eq!(
        config.search.api_keys,
        vec!["key-a".to_string(), "key-b".to_string()]
    );

    apply_config_mutations(
        &config_path,
        &[ConfigMutation::Unset {
            path: "search.api_keys[0]".to_string(),
        }],
    )
    .unwrap();
    let config = HoneConfig::from_file(&config_path).unwrap();
    assert_eq!(config.search.api_keys, vec!["key-b".to_string()]);
}

#[test]
fn test_apply_config_mutations_rejects_invalid_path_shape() {
    let dir = temp_test_dir("mutations-error");
    let config_path = dir.join("config.yaml");
    std::fs::write(
        &config_path,
        r#"
agent:
  runner: codex_cli
"#,
    )
    .unwrap();

    let error = apply_config_mutations(
        &config_path,
        &[ConfigMutation::Set {
            path: "agent.runner.value".to_string(),
            value: Value::String("x".to_string()),
        }],
    )
    .unwrap_err();
    assert!(
        error.to_string().contains("配置")
            || error.to_string().contains("invalid type")
            || error.to_string().contains("字符串")
    );
}

#[test]
fn test_apply_overlay_mutations_writes_only_to_overlay() {
    let dir = temp_test_dir("overlay-mutations");
    let config_path = dir.join("config.yaml");
    let overlay_path = runtime_overlay_path(&config_path);
    let base = r#"# user comments must survive
event_engine:
  global_digest:
    enabled: false
    schedules: ["09:00"]
    pass2_top_n: 15
"#;
    std::fs::write(&config_path, base).unwrap();

    let result = apply_overlay_mutations(
        &config_path,
        &[
            ConfigMutation::Set {
                path: "event_engine.global_digest.enabled".to_string(),
                value: Value::Bool(true),
            },
            ConfigMutation::Set {
                path: "event_engine.global_digest.schedules".to_string(),
                value: Value::Sequence(vec![
                    Value::String("09:00".to_string()),
                    Value::String("21:00".to_string()),
                ]),
            },
        ],
    )
    .unwrap();

    // base 不变,注释保留
    let base_after = std::fs::read_to_string(&config_path).unwrap();
    assert!(base_after.contains("# user comments must survive"));
    assert!(base_after.contains("enabled: false"));

    // overlay 文件存在,且内容只包含改动部分
    assert!(overlay_path.exists());
    let overlay_text = std::fs::read_to_string(&overlay_path).unwrap();
    assert!(overlay_text.contains("enabled: true"));
    assert!(overlay_text.contains("21:00"));
    assert!(!overlay_text.contains("pass2_top_n")); // 未改动的字段不该出现

    // 启动时合并后的 effective config 反映改动
    assert!(result.config.event_engine.global_digest.enabled);
    assert_eq!(
        result.config.event_engine.global_digest.schedules,
        vec!["09:00".to_string(), "21:00".to_string()]
    );
    // 未改动的字段保持 base 值
    assert_eq!(result.config.event_engine.global_digest.pass2_top_n, 15);
}

#[test]
fn test_apply_overlay_mutations_unset_removes_from_overlay() {
    let dir = temp_test_dir("overlay-unset");
    let config_path = dir.join("config.yaml");
    let overlay_path = runtime_overlay_path(&config_path);
    std::fs::write(
        &config_path,
        "event_engine:\n  global_digest:\n    enabled: false\n",
    )
    .unwrap();

    apply_overlay_mutations(
        &config_path,
        &[ConfigMutation::Set {
            path: "event_engine.global_digest.enabled".to_string(),
            value: Value::Bool(true),
        }],
    )
    .unwrap();
    assert!(overlay_path.exists());

    apply_overlay_mutations(
        &config_path,
        &[ConfigMutation::Unset {
            path: "event_engine.global_digest.enabled".to_string(),
        }],
    )
    .unwrap();
    // overlay 整个 mapping 空了 → write_overlay_patch 会删文件
    assert!(!overlay_path.exists());

    // effective 回到 base 值
    let cfg = HoneConfig::from_file(&config_path).unwrap();
    assert!(!cfg.event_engine.global_digest.enabled);
}

#[test]
fn test_apply_overlay_mutations_rejects_invalid_merged_config() {
    let dir = temp_test_dir("overlay-invalid");
    let config_path = dir.join("config.yaml");
    std::fs::write(&config_path, "feishu:\n  chat_scope: ALL\n").unwrap();

    // 写一个让 HoneConfig 解析失败的值(非法 ChatScope)
    let err = apply_overlay_mutations(
        &config_path,
        &[ConfigMutation::Set {
            path: "feishu.chat_scope".to_string(),
            value: Value::String("NOT_A_SCOPE".to_string()),
        }],
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("配置") || msg.contains("invalid") || msg.contains("解析"),
        "msg={msg}"
    );

    // overlay 不应被写入(校验失败应在 write 之前)
    let overlay_path = runtime_overlay_path(&config_path);
    assert!(!overlay_path.exists());
}

#[test]
fn test_redact_sensitive_value_masks_scalars_and_sequences() {
    assert_eq!(
        redact_sensitive_value(
            "agent.opencode.api_key",
            &Value::String("sk-123".to_string())
        ),
        Value::String("<redacted>".to_string())
    );
    assert_eq!(
        redact_sensitive_value(
            "search.api_keys",
            &Value::Sequence(vec![
                Value::String("a".to_string()),
                Value::String("b".to_string())
            ])
        ),
        Value::Sequence(vec![
            Value::String("<redacted>".to_string()),
            Value::String("<redacted>".to_string())
        ])
    );
    assert_eq!(
        redact_sensitive_value("agent.runner", &Value::String("codex_cli".to_string())),
        Value::String("codex_cli".to_string())
    );
}

#[test]
fn test_generate_effective_config_copies_relative_prompt_asset() {
    let dir = temp_test_dir("effective-config");
    let canonical = dir.join("config.yaml");
    let runtime_dir = dir.join("data/runtime");
    let effective = effective_config_path(&runtime_dir);

    std::fs::create_dir_all(&runtime_dir).unwrap();
    std::fs::write(
        &canonical,
        r#"
agent:
  system_prompt_path: "./soul.md"
  runner: codex_cli
"#,
    )
    .unwrap();
    std::fs::write(dir.join("soul.md"), "prompt").unwrap();

    let revision = generate_effective_config(&canonical, &effective).unwrap();
    assert!(!revision.is_empty());
    assert!(effective.exists());
    assert_eq!(
        std::fs::read_to_string(runtime_dir.join("soul.md")).unwrap(),
        "prompt"
    );
}

#[test]
fn test_promote_legacy_runtime_agent_settings_migrates_blank_multi_agent_and_runner() {
    let dir = temp_test_dir("legacy-agent-migrate");
    let canonical = dir.join("config.yaml");
    let legacy = dir.join("data/runtime/config_runtime.yaml");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(
        &canonical,
        r#"
agent:
  runner: codex_cli
  multi_agent:
    search:
      api_key: ""
    answer:
      api_key: ""
  opencode:
    api_key: ""
llm:
  auxiliary:
    api_key: ""
  openrouter:
    api_key: ""
    api_keys: []
search:
  api_keys: []
fmp:
  api_key: ""
  api_keys: []
feishu:
  enabled: false
  app_id: ""
  app_secret: ""
telegram:
  enabled: false
  bot_token: ""
  chat_scope: DM_ONLY
discord:
  enabled: false
  bot_token: ""
  chat_scope: DM_ONLY
"#,
    )
    .unwrap();
    std::fs::write(
        &legacy,
        r#"
agent:
  runner: multi-agent
  multi_agent:
    search:
      base_url: "https://api.minimaxi.com/v1"
      api_key: "legacy-search"
      model: "MiniMax-M2.7-highspeed"
      max_iterations: 8
    answer:
      api_base_url: "https://openrouter.ai/api/v1"
      api_key: "legacy-answer"
      model: "google/gemini-3.1-pro-preview"
      variant: "high"
      max_tool_calls: 1
  opencode:
    api_base_url: "https://openrouter.ai/api/v1"
    api_key: "legacy-answer"
    model: "google/gemini-3.1-pro-preview"
    variant: "high"
llm:
  auxiliary:
    base_url: "https://api.minimaxi.com/v1"
    api_key: "legacy-search"
    model: "MiniMax-M2.7-highspeed"
  openrouter:
    api_key: "legacy-openrouter"
    api_keys:
      - legacy-openrouter-1
      - legacy-openrouter-2
search:
  provider: tavily
  api_keys:
    - tvly-one
    - tvly-two
  search_depth: advanced
  topic: finance
fmp:
  api_key: "legacy-fmp"
  api_keys:
    - legacy-fmp-2
  base_url: "https://financialmodelingprep.com/api"
  timeout: 30
feishu:
  enabled: true
  app_id: "cli_test"
  app_secret: "secret"
telegram:
  enabled: true
  bot_token: "tg-token"
  dm_only: false
discord:
  enabled: true
  bot_token: "discord-token"
  dm_only: false
"#,
    )
    .unwrap();

    let changed = promote_legacy_runtime_agent_settings(&canonical, &legacy).unwrap();

    assert!(changed.contains(&"agent.multi_agent".to_string()));
    assert!(changed.contains(&"agent.opencode".to_string()));
    assert!(changed.contains(&"llm.auxiliary".to_string()));
    assert!(changed.contains(&"llm.openrouter.api_key".to_string()));
    assert!(changed.contains(&"llm.openrouter.api_keys".to_string()));
    assert!(changed.contains(&"agent.runner".to_string()));
    assert!(changed.contains(&"search.api_keys".to_string()));
    assert!(changed.contains(&"fmp.api_key".to_string()));
    assert!(changed.contains(&"fmp.api_keys".to_string()));
    assert!(changed.contains(&"feishu.enabled".to_string()));
    assert!(changed.contains(&"telegram.enabled".to_string()));
    assert!(changed.contains(&"discord.enabled".to_string()));

    let config = HoneConfig::from_file(&canonical).unwrap();
    assert_eq!(config.agent.runner, "multi-agent");
    assert_eq!(config.agent.multi_agent.search.api_key, "legacy-search");
    assert_eq!(config.agent.multi_agent.answer.api_key, "legacy-answer");
    assert_eq!(config.agent.opencode.api_key, "legacy-answer");
    assert_eq!(config.llm.auxiliary.api_key, "legacy-search");
    assert_eq!(config.llm.openrouter.api_key, "legacy-openrouter");
    assert_eq!(
        config.llm.openrouter.api_keys,
        vec![
            "legacy-openrouter-1".to_string(),
            "legacy-openrouter-2".to_string()
        ]
    );
    assert_eq!(
        config.search.api_keys,
        vec!["tvly-one".to_string(), "tvly-two".to_string()]
    );
    assert_eq!(config.fmp.api_key, "legacy-fmp");
    assert_eq!(config.fmp.api_keys, vec!["legacy-fmp-2".to_string()]);
    assert!(config.feishu.enabled);
    assert_eq!(config.feishu.app_id, "cli_test");
    assert_eq!(config.feishu.app_secret, "secret");
    assert!(config.telegram.enabled);
    assert_eq!(config.telegram.bot_token, "tg-token");
    assert_eq!(config.telegram.chat_scope, ChatScope::All);
    assert!(config.discord.enabled);
    assert_eq!(config.discord.bot_token, "discord-token");
    assert_eq!(config.discord.chat_scope, ChatScope::All);
}

#[test]
fn test_promote_legacy_runtime_agent_settings_migrates_openrouter_key_pool() {
    let dir = temp_test_dir("legacy-openrouter-pool");
    let canonical = dir.join("config.yaml");
    let legacy = dir.join("data/runtime/config_runtime.yaml");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(
        &canonical,
        r#"
llm:
  openrouter:
    api_key: ""
    api_keys: []
"#,
    )
    .unwrap();
    std::fs::write(
        &legacy,
        r#"
llm:
  openrouter:
    api_key: ""
    api_keys:
      - legacy-openrouter-1
      - legacy-openrouter-2
"#,
    )
    .unwrap();

    let changed = promote_legacy_runtime_agent_settings(&canonical, &legacy).unwrap();
    assert_eq!(changed, vec!["llm.openrouter.api_keys".to_string()]);

    let config = HoneConfig::from_file(&canonical).unwrap();
    assert_eq!(config.llm.openrouter.api_key, "");
    assert_eq!(
        config.llm.openrouter.api_keys,
        vec![
            "legacy-openrouter-1".to_string(),
            "legacy-openrouter-2".to_string()
        ]
    );
    assert_eq!(
        config.llm.openrouter.effective_key_pool().keys(),
        &["legacy-openrouter-1", "legacy-openrouter-2"]
    );
}

#[test]
fn test_promote_legacy_runtime_agent_settings_keeps_configured_canonical_values() {
    let dir = temp_test_dir("legacy-agent-preserve");
    let canonical = dir.join("config.yaml");
    let legacy = dir.join("data/runtime/config_runtime.yaml");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(
        &canonical,
        r#"
agent:
  runner: multi-agent
  multi_agent:
    search:
      api_key: "canonical-search"
    answer:
      api_key: "canonical-answer"
llm:
  auxiliary:
    api_key: "canonical-aux"
  openrouter:
    api_key: "canonical-openrouter"
search:
  api_keys:
    - canonical-tavily
fmp:
  api_key: "canonical-fmp"
feishu:
  enabled: true
  app_id: "canonical-app"
telegram:
  enabled: true
  bot_token: "canonical-tg"
discord:
  enabled: true
  bot_token: "canonical-discord"
"#,
    )
    .unwrap();
    std::fs::write(
        &legacy,
        r#"
agent:
  runner: codex_cli
  multi_agent:
    search:
      api_key: "legacy-search"
    answer:
      api_key: "legacy-answer"
llm:
  auxiliary:
    api_key: "legacy-aux"
  openrouter:
    api_key: "legacy-openrouter"
search:
  api_keys:
    - legacy-tavily
fmp:
  api_key: "legacy-fmp"
feishu:
  enabled: true
  app_id: "legacy-app"
telegram:
  enabled: true
  bot_token: "legacy-tg"
discord:
  enabled: true
  bot_token: "legacy-discord"
"#,
    )
    .unwrap();

    let changed = promote_legacy_runtime_agent_settings(&canonical, &legacy).unwrap();
    assert!(changed.is_empty());

    let config = HoneConfig::from_file(&canonical).unwrap();
    assert_eq!(config.agent.runner, "multi-agent");
    assert_eq!(config.agent.multi_agent.search.api_key, "canonical-search");
    assert_eq!(config.agent.multi_agent.answer.api_key, "canonical-answer");
    assert_eq!(config.llm.auxiliary.api_key, "canonical-aux");
    assert_eq!(config.llm.openrouter.api_key, "canonical-openrouter");
    assert_eq!(config.search.api_keys, vec!["canonical-tavily".to_string()]);
    assert_eq!(config.fmp.api_key, "canonical-fmp");
    assert_eq!(config.feishu.app_id, "canonical-app");
    assert_eq!(config.telegram.bot_token, "canonical-tg");
    assert_eq!(config.discord.bot_token, "canonical-discord");
}

#[test]
fn test_promote_legacy_runtime_agent_settings_preserves_blank_opencode_key_inheritance() {
    let dir = temp_test_dir("legacy-agent-opencode-inheritance");
    let canonical = dir.join("config.yaml");
    let legacy = dir.join("data/runtime/config_runtime.yaml");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(
        &canonical,
        r#"
agent:
  runner: opencode_acp
  opencode:
    api_base_url: "https://canonical.example/v1"
    api_key: ""
    model: "google/gemini-2.5-pro"
    variant: "high"
"#,
    )
    .unwrap();
    std::fs::write(
        &legacy,
        r#"
agent:
  runner: opencode_acp
  opencode:
    api_base_url: "https://legacy.example/v1"
    api_key: "legacy-key"
    model: "legacy-model"
    variant: "legacy-variant"
"#,
    )
    .unwrap();

    let changed = promote_legacy_runtime_agent_settings(&canonical, &legacy).unwrap();
    assert!(changed.is_empty());

    let config = HoneConfig::from_file(&canonical).unwrap();
    assert_eq!(
        config.agent.opencode.api_base_url,
        "https://canonical.example/v1"
    );
    assert_eq!(config.agent.opencode.api_key, "");
    assert_eq!(config.agent.opencode.model, "google/gemini-2.5-pro");
    assert_eq!(config.agent.opencode.variant, "high");
}

#[test]
fn test_agent_runner_timeouts_default_to_step_plus_overall() {
    let yaml = r#"
agent:
  runner: codex_acp
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.agent.step_timeout_seconds, 180);
    assert_eq!(config.agent.overall_timeout_seconds, 1200);
}

#[test]
fn test_agent_runner_timeout_override_preserves_explicit_values() {
    let yaml = r#"
agent:
  runner: codex_acp
  step_timeout_seconds: 120
  overall_timeout_seconds: 600
"#;
    let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.agent.step_timeout_seconds, 120);
    assert_eq!(config.agent.overall_timeout_seconds, 600);
}
