use super::*;
use hone_core::config::{ConfigMutation, apply_config_mutations, generate_effective_config};

fn chat_scope_value(scope: hone_core::config::ChatScope) -> String {
    match scope {
        hone_core::config::ChatScope::DmOnly => "DM_ONLY".to_string(),
        hone_core::config::ChatScope::GroupchatOnly => "GROUPCHAT_ONLY".to_string(),
        hone_core::config::ChatScope::All => "ALL".to_string(),
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

fn channel_settings_from_config(config_path: &Path, config: HoneConfig) -> DesktopChannelSettings {
    DesktopChannelSettings {
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

pub(super) fn seed_multi_agent_settings(config: &HoneConfig) -> MultiAgentSettings {
    let search = MultiAgentSearchSettings {
        base_url: if config.agent.multi_agent.search.base_url.trim().is_empty() {
            "https://api.minimaxi.com/v1".to_string()
        } else {
            config.agent.multi_agent.search.base_url.clone()
        },
        api_key: if config.agent.multi_agent.search.api_key.trim().is_empty() {
            config.llm.auxiliary.api_key.clone()
        } else {
            config.agent.multi_agent.search.api_key.clone()
        },
        model: if config.agent.multi_agent.search.model.trim().is_empty() {
            "MiniMax-M2.7-highspeed".to_string()
        } else {
            config.agent.multi_agent.search.model.clone()
        },
        max_iterations: if config.agent.multi_agent.search.max_iterations == 0 {
            8
        } else {
            config.agent.multi_agent.search.max_iterations
        },
    };

    let answer = MultiAgentAnswerSettings {
        base_url: if config
            .agent
            .multi_agent
            .answer
            .api_base_url
            .trim()
            .is_empty()
        {
            config.agent.opencode.api_base_url.clone()
        } else {
            config.agent.multi_agent.answer.api_base_url.clone()
        },
        api_key: if config.agent.multi_agent.answer.api_key.trim().is_empty() {
            config.agent.opencode.api_key.clone()
        } else {
            config.agent.multi_agent.answer.api_key.clone()
        },
        model: if config.agent.multi_agent.answer.model.trim().is_empty() {
            config.agent.opencode.model.clone()
        } else {
            config.agent.multi_agent.answer.model.clone()
        },
        variant: if config.agent.multi_agent.answer.variant.trim().is_empty() {
            config.agent.opencode.variant.clone()
        } else {
            config.agent.multi_agent.answer.variant.clone()
        },
        max_tool_calls: if config.agent.multi_agent.answer.max_tool_calls == 0 {
            1
        } else {
            config.agent.multi_agent.answer.max_tool_calls
        },
    };

    MultiAgentSettings { search, answer }
}

pub(super) fn seed_auxiliary_settings(config: &HoneConfig) -> AuxiliarySettings {
    let multi_search = seed_multi_agent_settings(config).search;
    let configured = &config.llm.auxiliary;

    AuxiliarySettings {
        base_url: if !configured.base_url.trim().is_empty() {
            configured.base_url.clone()
        } else if !multi_search.base_url.trim().is_empty() {
            multi_search.base_url
        } else {
            "https://api.minimaxi.com/v1".to_string()
        },
        api_key: if !configured.api_key.trim().is_empty() {
            configured.api_key.clone()
        } else if !multi_search.api_key.trim().is_empty() {
            multi_search.api_key
        } else {
            String::new()
        },
        model: if !configured.model.trim().is_empty() {
            configured.model.clone()
        } else if !multi_search.model.trim().is_empty() {
            multi_search.model
        } else {
            config.llm.openrouter.auxiliary_model().to_string()
        },
    }
}

pub(super) fn load_persisted_config(app: &AppHandle) -> Result<BackendConfig, String> {
    let path = config_store_path(app)?;
    if !path.exists() {
        return Ok(BackendConfig::default());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub(super) fn save_persisted_config(app: &AppHandle, config: &BackendConfig) -> Result<(), String> {
    let path = config_store_path(app)?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub(super) fn apply_setting_updates(
    config_path: &Path,
    effective_config_path: &Path,
    updates: Vec<(&str, serde_yaml::Value)>,
) -> Result<HoneConfig, String> {
    let mutations = updates
        .into_iter()
        .map(|(path, value)| ConfigMutation::Set {
            path: path.to_string(),
            value,
        })
        .collect::<Vec<_>>();
    let result = apply_config_mutations(config_path, &mutations).map_err(|e| e.to_string())?;
    generate_effective_config(config_path, effective_config_path).map_err(|e| e.to_string())?;
    Ok(result.config)
}

pub(super) fn load_channel_settings(app: &AppHandle) -> Result<DesktopChannelSettings, String> {
    let runtime = ensure_runtime_paths(app)?;
    let config_path = runtime.config_path;
    let config = HoneConfig::from_file(&config_path).map_err(|e| e.to_string())?;
    Ok(channel_settings_from_config(&config_path, config))
}

pub(super) fn save_channel_settings(
    app: &AppHandle,
    settings: &DesktopChannelSettingsInput,
) -> Result<DesktopChannelSettings, String> {
    let runtime = ensure_runtime_paths(app)?;
    let config_path = runtime.config_path;
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
    let config = apply_setting_updates(
        &config_path,
        &runtime.effective_config_path,
        vec![
            (
                "imessage.enabled",
                serde_yaml::Value::Bool(settings.imessage_enabled),
            ),
            (
                "imessage.target_handle",
                serde_yaml::Value::String(settings.imessage_target_handle.clone()),
            ),
            (
                "feishu.enabled",
                serde_yaml::Value::Bool(settings.feishu_enabled),
            ),
            (
                "feishu.app_id",
                serde_yaml::Value::String(settings.feishu_app_id.clone()),
            ),
            (
                "feishu.app_secret",
                serde_yaml::Value::String(settings.feishu_app_secret.clone()),
            ),
            (
                "feishu.chat_scope",
                serde_yaml::Value::String(feishu_chat_scope),
            ),
            (
                "feishu.allow_emails",
                string_sequence(&settings.feishu_allow_emails),
            ),
            (
                "feishu.allow_mobiles",
                string_sequence(&settings.feishu_allow_mobiles),
            ),
            (
                "feishu.allow_open_ids",
                string_sequence(&settings.feishu_allow_open_ids),
            ),
            (
                "telegram.enabled",
                serde_yaml::Value::Bool(settings.telegram_enabled),
            ),
            (
                "telegram.bot_token",
                serde_yaml::Value::String(settings.telegram_bot_token.clone()),
            ),
            (
                "telegram.chat_scope",
                serde_yaml::Value::String(telegram_chat_scope),
            ),
            (
                "telegram.allow_from",
                string_sequence(&settings.telegram_allow_from),
            ),
            (
                "discord.enabled",
                serde_yaml::Value::Bool(settings.discord_enabled),
            ),
            (
                "discord.bot_token",
                serde_yaml::Value::String(settings.discord_bot_token.clone()),
            ),
            (
                "discord.chat_scope",
                serde_yaml::Value::String(discord_chat_scope),
            ),
            (
                "discord.allow_from",
                string_sequence(&settings.discord_allow_from),
            ),
        ],
    )?;

    Ok(channel_settings_from_config(&config_path, config))
}
