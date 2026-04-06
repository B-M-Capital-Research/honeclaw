//! 配置加载与验证
//!
//! 从 config.yaml 加载配置，使用 serde 反序列化。

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod agent;
pub mod channels;
pub mod server;

pub use agent::{
    AdminConfig, AgentConfig, AuxiliaryLlmConfig, CodexAcpConfig, GeminiAcpConfig, KimiConfig,
    LlmConfig, MultiAgentAnswerConfig, MultiAgentConfig, MultiAgentSearchConfig, OpenRouterConfig,
    OpencodeAcpConfig,
};
pub use channels::{
    ChatScope, DiscordConfig, DiscordGroupReplyConfig, DiscordWatchConfig, FeishuConfig,
    GroupContextConfig, IMessageConfig, TelegramConfig, XConfig, XOAuth1Config,
};
pub use server::{
    FmpConfig, LoggingConfig, NanoBananaConfig, SearchConfig, SecurityConfig, StorageConfig,
    ToolGuardConfig, WebConfig,
};

/// 顶层配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoneConfig {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub imessage: IMessageConfig,
    #[serde(default)]
    pub feishu: FeishuConfig,
    #[serde(default)]
    pub telegram: TelegramConfig,
    #[serde(default)]
    pub discord: DiscordConfig,
    #[serde(default)]
    pub group_context: GroupContextConfig,
    #[serde(default)]
    pub x: XConfig,
    #[serde(default)]
    pub nano_banana: NanoBananaConfig,
    #[serde(default)]
    pub fmp: FmpConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    /// Agent system prompt 模板
    #[serde(default)]
    pub agent: AgentConfig,
    /// 管理员配置
    #[serde(default)]
    pub admins: AdminConfig,
    /// Web 控制台配置
    #[serde(default)]
    pub web: WebConfig,
    /// 安全策略配置
    #[serde(default)]
    pub security: SecurityConfig,
    /// 额外的未知字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

impl HoneConfig {
    /// 从 YAML 文件加载配置
    pub fn from_file(path: impl AsRef<Path>) -> crate::HoneResult<Self> {
        let path = path.as_ref();
        let mut value = read_yaml_value(path)?;
        let overlay_path = runtime_overlay_path(path);
        if overlay_path.exists() {
            let overlay = read_yaml_value(&overlay_path)?;
            if !overlay.is_null() {
                merge_yaml_value(&mut value, overlay);
            }
        }
        let config: Self = serde_yaml::from_value(value)
            .map_err(|e| crate::HoneError::Config(format!("配置文件解析失败: {e}")))?;
        let mut config = config;
        if let Err(err) = apply_system_prompt_path(&mut config, path) {
            return Err(crate::HoneError::Config(err));
        }
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> crate::HoneResult<()> {
        validate_channel_chat_scope("feishu", self.feishu.chat_scope)?;
        validate_channel_chat_scope("telegram", self.telegram.chat_scope)?;
        validate_channel_chat_scope("discord", self.discord.chat_scope)?;
        Ok(())
    }
}

/// 计算与给定配置文件同目录的覆盖层路径。
pub fn runtime_overlay_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let overlay_name = match (
        path.file_stem().and_then(|s| s.to_str()),
        path.extension().and_then(|s| s.to_str()),
    ) {
        (Some(stem), Some(ext)) => format!("{stem}.overrides.{ext}"),
        (Some(stem), None) => format!("{stem}.overrides"),
        _ => format!(
            "{}.overrides",
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("config")
        ),
    };
    parent.join(overlay_name)
}

/// 读取 YAML 到通用 `Value`，供合并和补丁生成使用。
pub fn read_yaml_value(path: impl AsRef<Path>) -> crate::HoneResult<Value> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| crate::HoneError::Config(format!("无法读取配置文件: {e}")))?;
    if content.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_yaml::from_str(&content)
        .map_err(|e| crate::HoneError::Config(format!("配置文件解析失败: {e}")))
}

/// 将覆盖层递归合并到基础 YAML 上。
///
/// 规则：
/// - mapping 递归合并
/// - sequence / 标量 / null 直接替换
pub fn merge_yaml_value(base: &mut Value, overlay: Value) {
    match overlay {
        Value::Mapping(overlay_map) => {
            if let Value::Mapping(base_map) = base {
                for (key, overlay_value) in overlay_map {
                    match base_map.get_mut(&key) {
                        Some(base_value) => merge_yaml_value(base_value, overlay_value),
                        None => {
                            base_map.insert(key, overlay_value);
                        }
                    }
                }
            } else {
                *base = Value::Mapping(overlay_map);
            }
        }
        overlay => {
            *base = overlay;
        }
    }
}

/// 计算 `current` 相对 `base` 的最小覆盖层补丁。
///
/// - mapping 只保留有差异的子树
/// - sequence / 标量 / null 发生变化时整段保留
pub fn diff_yaml_value(base: &Value, current: &Value) -> Option<Value> {
    match (base, current) {
        (Value::Mapping(base_map), Value::Mapping(current_map)) => {
            let mut patch = Mapping::new();
            for (key, current_value) in current_map {
                match base_map.get(key) {
                    Some(base_value) => {
                        if let Some(child_patch) = diff_yaml_value(base_value, current_value) {
                            patch.insert(key.clone(), child_patch);
                        }
                    }
                    None => {
                        patch.insert(key.clone(), current_value.clone());
                    }
                }
            }
            if patch.is_empty() {
                None
            } else {
                Some(Value::Mapping(patch))
            }
        }
        (Value::Sequence(base_seq), Value::Sequence(current_seq)) => {
            if base_seq == current_seq {
                None
            } else {
                Some(current.clone())
            }
        }
        _ => {
            if base == current {
                None
            } else {
                Some(current.clone())
            }
        }
    }
}

fn apply_system_prompt_path(config: &mut HoneConfig, config_path: &Path) -> Result<(), String> {
    let prompt_path = config.agent.system_prompt_path.trim();
    if prompt_path.is_empty() {
        return Ok(());
    }

    let resolved = if Path::new(prompt_path).is_absolute() {
        PathBuf::from(prompt_path)
    } else {
        let base_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
        base_dir.join(prompt_path)
    };

    let content = std::fs::read_to_string(&resolved)
        .map_err(|e| format!("无法读取 system_prompt_path ({})：{e}", resolved.display()))?;
    config.agent.system_prompt = content;
    Ok(())
}

fn validate_channel_chat_scope(channel: &str, chat_scope: ChatScope) -> crate::HoneResult<()> {
    let raw = match chat_scope {
        ChatScope::DmOnly => "DM_ONLY",
        ChatScope::GroupchatOnly => "GROUPCHAT_ONLY",
        ChatScope::All => "ALL",
    };
    if raw.trim().is_empty() {
        return Err(crate::HoneError::Config(format!(
            "{channel}.chat_scope 不能为空"
        )));
    }
    Ok(())
}

impl Default for HoneConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            imessage: IMessageConfig::default(),
            feishu: FeishuConfig::default(),
            telegram: TelegramConfig::default(),
            discord: DiscordConfig::default(),
            group_context: GroupContextConfig::default(),
            x: XConfig::default(),
            nano_banana: NanoBananaConfig::default(),
            fmp: FmpConfig::default(),
            search: SearchConfig::default(),
            logging: LoggingConfig::default(),
            storage: StorageConfig::default(),
            agent: AgentConfig::default(),
            admins: AdminConfig::default(),
            web: WebConfig::default(),
            security: SecurityConfig::default(),
            extra: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ChatScope;

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("hone-config-{}-{}", prefix, uuid::Uuid::new_v4()));
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
        let path = Path::new("/tmp/config_runtime.yaml");
        let overlay = runtime_overlay_path(path);
        assert_eq!(overlay, PathBuf::from("/tmp/config_runtime.overrides.yaml"));
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
    fn test_from_file_applies_runtime_overlay() {
        let dir = temp_test_dir("from-file");
        let config_path = dir.join("config_runtime.yaml");
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

        let config = HoneConfig::from_file(&config_path).unwrap();
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
}
