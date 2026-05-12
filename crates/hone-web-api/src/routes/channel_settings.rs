use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use hone_core::config::{
    ChatScope, ConfigMutation, HoneConfig, apply_config_mutations, generate_effective_config,
};

use crate::routes::json_error;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelSettings {
    config_path: String,
    imessage_enabled: bool,
    imessage_target_handle: String,
    feishu_enabled: bool,
    feishu_app_id: String,
    feishu_app_secret: String,
    feishu_chat_scope: String,
    feishu_allow_emails: Vec<String>,
    feishu_allow_mobiles: Vec<String>,
    feishu_allow_open_ids: Vec<String>,
    telegram_enabled: bool,
    telegram_bot_token: String,
    telegram_chat_scope: String,
    telegram_allow_from: Vec<String>,
    discord_enabled: bool,
    discord_bot_token: String,
    discord_chat_scope: String,
    discord_allow_from: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelSettingsInput {
    imessage_enabled: bool,
    #[serde(default)]
    imessage_target_handle: String,
    feishu_enabled: bool,
    #[serde(default)]
    feishu_app_id: String,
    #[serde(default)]
    feishu_app_secret: String,
    #[serde(default)]
    feishu_chat_scope: String,
    #[serde(default)]
    feishu_allow_emails: Vec<String>,
    #[serde(default)]
    feishu_allow_mobiles: Vec<String>,
    #[serde(default)]
    feishu_allow_open_ids: Vec<String>,
    telegram_enabled: bool,
    #[serde(default)]
    telegram_bot_token: String,
    #[serde(default)]
    telegram_chat_scope: String,
    #[serde(default)]
    telegram_allow_from: Vec<String>,
    discord_enabled: bool,
    #[serde(default)]
    discord_bot_token: String,
    #[serde(default)]
    discord_chat_scope: String,
    #[serde(default)]
    discord_allow_from: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChannelSettingsUpdateResult {
    settings: ChannelSettings,
    restarted_bundled_backend: bool,
    message: String,
}

fn canonical_config_path() -> PathBuf {
    env::var_os("HONE_USER_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(crate::runtime::runtime_config_path()))
}

fn effective_config_path() -> PathBuf {
    PathBuf::from(crate::runtime::runtime_config_path())
}

fn chat_scope_value(scope: ChatScope) -> String {
    match scope {
        ChatScope::DmOnly => "DM_ONLY".to_string(),
        ChatScope::GroupchatOnly => "GROUPCHAT_ONLY".to_string(),
        ChatScope::All => "ALL".to_string(),
    }
}

fn string_sequence(values: &[String]) -> serde_yaml::Value {
    serde_yaml::Value::Sequence(
        values
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .map(serde_yaml::Value::String)
            .collect(),
    )
}

fn configured_or_current(value: &str, current: String) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        current
    } else {
        trimmed.to_string()
    }
}

fn channel_settings_from_config(config_path: &PathBuf, config: HoneConfig) -> ChannelSettings {
    ChannelSettings {
        config_path: config_path.to_string_lossy().to_string(),
        imessage_enabled: config.imessage.enabled,
        imessage_target_handle: config.imessage.target_handle,
        feishu_enabled: config.feishu.enabled,
        feishu_app_id: config.feishu.app_id,
        feishu_app_secret: config.feishu.app_secret,
        feishu_chat_scope: chat_scope_value(config.feishu.chat_scope),
        feishu_allow_emails: config.feishu.allow_emails,
        feishu_allow_mobiles: config.feishu.allow_mobiles,
        feishu_allow_open_ids: config.feishu.allow_open_ids,
        telegram_enabled: config.telegram.enabled,
        telegram_bot_token: config.telegram.bot_token,
        telegram_chat_scope: chat_scope_value(config.telegram.chat_scope),
        telegram_allow_from: config.telegram.allow_from,
        discord_enabled: config.discord.enabled,
        discord_bot_token: config.discord.bot_token,
        discord_chat_scope: chat_scope_value(config.discord.chat_scope),
        discord_allow_from: config.discord.allow_from,
    }
}

fn load_channel_settings_from_disk() -> Result<ChannelSettings, String> {
    let config_path = canonical_config_path();
    let config = HoneConfig::from_file(&config_path).map_err(|e| e.to_string())?;
    Ok(channel_settings_from_config(&config_path, config))
}

fn save_channel_settings_to_disk(
    settings: &ChannelSettingsInput,
) -> Result<ChannelSettings, String> {
    let config_path = canonical_config_path();
    let current = HoneConfig::from_file(&config_path).map_err(|e| e.to_string())?;
    let feishu_chat_scope = configured_or_current(
        &settings.feishu_chat_scope,
        chat_scope_value(current.feishu.chat_scope),
    );
    let telegram_chat_scope = configured_or_current(
        &settings.telegram_chat_scope,
        chat_scope_value(current.telegram.chat_scope),
    );
    let discord_chat_scope = configured_or_current(
        &settings.discord_chat_scope,
        chat_scope_value(current.discord.chat_scope),
    );

    let mutations = vec![
        ConfigMutation::Set {
            path: "imessage.enabled".to_string(),
            value: serde_yaml::Value::Bool(settings.imessage_enabled),
        },
        ConfigMutation::Set {
            path: "imessage.target_handle".to_string(),
            value: serde_yaml::Value::String(settings.imessage_target_handle.clone()),
        },
        ConfigMutation::Set {
            path: "feishu.enabled".to_string(),
            value: serde_yaml::Value::Bool(settings.feishu_enabled),
        },
        ConfigMutation::Set {
            path: "feishu.app_id".to_string(),
            value: serde_yaml::Value::String(settings.feishu_app_id.clone()),
        },
        ConfigMutation::Set {
            path: "feishu.app_secret".to_string(),
            value: serde_yaml::Value::String(settings.feishu_app_secret.clone()),
        },
        ConfigMutation::Set {
            path: "feishu.chat_scope".to_string(),
            value: serde_yaml::Value::String(feishu_chat_scope),
        },
        ConfigMutation::Set {
            path: "feishu.allow_emails".to_string(),
            value: string_sequence(&settings.feishu_allow_emails),
        },
        ConfigMutation::Set {
            path: "feishu.allow_mobiles".to_string(),
            value: string_sequence(&settings.feishu_allow_mobiles),
        },
        ConfigMutation::Set {
            path: "feishu.allow_open_ids".to_string(),
            value: string_sequence(&settings.feishu_allow_open_ids),
        },
        ConfigMutation::Set {
            path: "telegram.enabled".to_string(),
            value: serde_yaml::Value::Bool(settings.telegram_enabled),
        },
        ConfigMutation::Set {
            path: "telegram.bot_token".to_string(),
            value: serde_yaml::Value::String(settings.telegram_bot_token.clone()),
        },
        ConfigMutation::Set {
            path: "telegram.chat_scope".to_string(),
            value: serde_yaml::Value::String(telegram_chat_scope),
        },
        ConfigMutation::Set {
            path: "telegram.allow_from".to_string(),
            value: string_sequence(&settings.telegram_allow_from),
        },
        ConfigMutation::Set {
            path: "discord.enabled".to_string(),
            value: serde_yaml::Value::Bool(settings.discord_enabled),
        },
        ConfigMutation::Set {
            path: "discord.bot_token".to_string(),
            value: serde_yaml::Value::String(settings.discord_bot_token.clone()),
        },
        ConfigMutation::Set {
            path: "discord.chat_scope".to_string(),
            value: serde_yaml::Value::String(discord_chat_scope),
        },
        ConfigMutation::Set {
            path: "discord.allow_from".to_string(),
            value: string_sequence(&settings.discord_allow_from),
        },
    ];

    let result = apply_config_mutations(&config_path, &mutations).map_err(|e| e.to_string())?;
    let effective_config_path = effective_config_path();
    if effective_config_path != config_path {
        generate_effective_config(&config_path, &effective_config_path)
            .map_err(|e| e.to_string())?;
    }
    Ok(channel_settings_from_config(&config_path, result.config))
}

pub(crate) async fn handle_get_channel_settings(State(_state): State<Arc<AppState>>) -> Response {
    match load_channel_settings_from_disk() {
        Ok(settings) => Json(settings).into_response(),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to load channel settings: {e}"),
        ),
    }
}

pub(crate) async fn handle_put_channel_settings(
    State(_state): State<Arc<AppState>>,
    Json(settings): Json<ChannelSettingsInput>,
) -> Response {
    match save_channel_settings_to_disk(&settings) {
        Ok(settings) => Json(ChannelSettingsUpdateResult {
            settings,
            restarted_bundled_backend: false,
            message: "已保存渠道配置；当前 CLI 启动的运行时需要重启后拉起或停止渠道监听器。"
                .to_string(),
        })
        .into_response(),
        Err(e) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to save channel settings: {e}"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
    }

    #[test]
    fn serializes_channel_settings_as_camel_case() {
        let settings = ChannelSettings {
            config_path: "/tmp/config.yaml".to_string(),
            imessage_enabled: false,
            imessage_target_handle: String::new(),
            feishu_enabled: false,
            feishu_app_id: String::new(),
            feishu_app_secret: String::new(),
            feishu_chat_scope: "DM_ONLY".to_string(),
            feishu_allow_emails: vec![],
            feishu_allow_mobiles: vec![],
            feishu_allow_open_ids: vec![],
            telegram_enabled: false,
            telegram_bot_token: String::new(),
            telegram_chat_scope: "DM_ONLY".to_string(),
            telegram_allow_from: vec![],
            discord_enabled: true,
            discord_bot_token: "token".to_string(),
            discord_chat_scope: "ALL".to_string(),
            discord_allow_from: vec!["123".to_string()],
        };

        let value = serde_json::to_value(settings).expect("settings should serialize");
        assert_eq!(value["discordEnabled"].as_bool(), Some(true));
        assert_eq!(value["discordChatScope"], "ALL");
        assert_eq!(value["discordAllowFrom"][0], "123");
        assert!(value.get("discord_enabled").is_none());
    }

    #[test]
    fn save_channel_settings_updates_canonical_and_effective_config() {
        let _guard = crate::test_env_lock().lock().unwrap();
        let root = temp_dir("hone_web_channel_settings");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("soul.md"), "system prompt").unwrap();
        let canonical = root.join("config.yaml");
        let effective = root.join("effective-config.yaml");
        std::fs::write(
            &canonical,
            r#"
system_prompt:
  path: soul.md
discord:
  enabled: false
  bot_token: old-discord-token
  chat_scope: DM_ONLY
telegram:
  enabled: false
  bot_token: old-telegram-token
  chat_scope: DM_ONLY
"#,
        )
        .unwrap();
        std::fs::write(&effective, "language: zh\n").unwrap();

        unsafe {
            env::set_var("HONE_USER_CONFIG_PATH", &canonical);
            env::set_var("HONE_CONFIG_PATH", &effective);
        }

        let saved = save_channel_settings_to_disk(&ChannelSettingsInput {
            imessage_enabled: false,
            imessage_target_handle: String::new(),
            feishu_enabled: false,
            feishu_app_id: String::new(),
            feishu_app_secret: String::new(),
            feishu_chat_scope: String::new(),
            feishu_allow_emails: vec![],
            feishu_allow_mobiles: vec![],
            feishu_allow_open_ids: vec![],
            telegram_enabled: true,
            telegram_bot_token: "new-telegram-token".to_string(),
            telegram_chat_scope: "ALL".to_string(),
            telegram_allow_from: vec![" 111 ".to_string(), String::new(), "222".to_string()],
            discord_enabled: true,
            discord_bot_token: "new-discord-token".to_string(),
            discord_chat_scope: String::new(),
            discord_allow_from: vec!["333".to_string()],
        })
        .expect("channel settings should save");

        assert!(saved.discord_enabled);
        assert_eq!(saved.discord_chat_scope, "DM_ONLY");
        assert!(saved.telegram_enabled);
        assert_eq!(saved.telegram_chat_scope, "ALL");
        assert_eq!(saved.telegram_allow_from, vec!["111", "222"]);

        let canonical_config = HoneConfig::from_file(&canonical).unwrap();
        let effective_config = HoneConfig::from_file(&effective).unwrap();
        assert!(canonical_config.discord.enabled);
        assert!(effective_config.discord.enabled);
        assert!(canonical_config.telegram.enabled);
        assert!(effective_config.telegram.enabled);

        unsafe {
            env::remove_var("HONE_USER_CONFIG_PATH");
            env::remove_var("HONE_CONFIG_PATH");
        }
        let _ = std::fs::remove_dir_all(root);
    }
}
