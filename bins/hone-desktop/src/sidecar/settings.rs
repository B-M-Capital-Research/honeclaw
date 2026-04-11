use super::*;

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

pub(super) fn save_persisted_config(
    app: &AppHandle,
    config: &BackendConfig,
) -> Result<(), String> {
    let path = config_store_path(app)?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn atomic_write_yaml(path: &Path, yaml: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let parent = path
        .parent()
        .ok_or_else(|| format!("覆盖层路径缺少父目录: {}", path.display()))?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("config");
    let tmp_path = parent.join(format!(".{file_name}.{stamp}.tmp"));

    fs::write(&tmp_path, yaml).map_err(|e| e.to_string())?;
    match fs::rename(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(first_err) => {
            let _ = fs::remove_file(path);
            match fs::rename(&tmp_path, path) {
                Ok(()) => Ok(()),
                Err(second_err) => {
                    let _ = fs::remove_file(&tmp_path);
                    Err(format!(
                        "无法写入覆盖层 {}: {second_err}（初次重命名错误: {first_err}）",
                        path.display()
                    ))
                }
            }
        }
    }
}

fn write_overlay_patch(path: &Path, patch: Option<serde_yaml::Value>) -> Result<(), String> {
    match patch {
        None => {
            if path.exists() {
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        Some(serde_yaml::Value::Mapping(map)) if map.is_empty() => {
            if path.exists() {
                fs::remove_file(path).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        Some(value) => {
            let yaml = serde_yaml::to_string(&value).map_err(|e| e.to_string())?;
            atomic_write_yaml(path, &yaml)
        }
    }
}

pub(super) fn save_runtime_config_overlay<F>(
    config_path: &Path,
    mutate: F,
) -> Result<HoneConfig, String>
where
    F: FnOnce(&mut HoneConfig),
{
    let base_value = read_yaml_value(config_path).map_err(|e| e.to_string())?;
    let mut config = HoneConfig::from_file(config_path).map_err(|e| e.to_string())?;
    mutate(&mut config);
    let current_value = serde_yaml::to_value(&config).map_err(|e| e.to_string())?;
    let patch = diff_yaml_value(&base_value, &current_value);
    let overlay_path = runtime_overlay_path(config_path);
    write_overlay_patch(&overlay_path, patch)?;
    Ok(config)
}

pub(super) fn load_channel_settings(app: &AppHandle) -> Result<DesktopChannelSettings, String> {
    let runtime = ensure_runtime_paths(app)?;
    let config_path = runtime.config_path;
    let config = HoneConfig::from_file(&config_path).map_err(|e| e.to_string())?;
    Ok(DesktopChannelSettings {
        config_path: config_path.to_string_lossy().to_string(),
        imessage_enabled: config.imessage.enabled,
        feishu_enabled: config.feishu.enabled,
        feishu_app_id: config.feishu.app_id,
        feishu_app_secret: config.feishu.app_secret,
        telegram_enabled: config.telegram.enabled,
        telegram_bot_token: config.telegram.bot_token,
        discord_enabled: config.discord.enabled,
        discord_bot_token: config.discord.bot_token,
    })
}

pub(super) fn save_channel_settings(
    app: &AppHandle,
    settings: &DesktopChannelSettingsInput,
) -> Result<DesktopChannelSettings, String> {
    let runtime = ensure_runtime_paths(app)?;
    let config_path = runtime.config_path;
    let config = save_runtime_config_overlay(&config_path, |config| {
        config.imessage.enabled = settings.imessage_enabled;
        config.feishu.enabled = settings.feishu_enabled;
        config.feishu.app_id = settings.feishu_app_id.clone();
        config.feishu.app_secret = settings.feishu_app_secret.clone();
        config.telegram.enabled = settings.telegram_enabled;
        config.telegram.bot_token = settings.telegram_bot_token.clone();
        config.discord.enabled = settings.discord_enabled;
        config.discord.bot_token = settings.discord_bot_token.clone();
    })?;

    Ok(DesktopChannelSettings {
        config_path: config_path.to_string_lossy().to_string(),
        imessage_enabled: config.imessage.enabled,
        feishu_enabled: config.feishu.enabled,
        feishu_app_id: config.feishu.app_id,
        feishu_app_secret: config.feishu.app_secret,
        telegram_enabled: config.telegram.enabled,
        telegram_bot_token: config.telegram.bot_token,
        discord_enabled: config.discord.enabled,
        discord_bot_token: config.discord.bot_token,
    })
}
