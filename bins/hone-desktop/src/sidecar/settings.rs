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

pub(super) fn seed_auxiliary_settings(config: &HoneConfig) -> AuxiliarySettings {
    let configured = &config.llm.auxiliary;

    AuxiliarySettings {
        base_url: if !configured.base_url.trim().is_empty() {
            configured.base_url.clone()
        } else {
            "https://api.minimaxi.com/v1".to_string()
        },
        api_key: if !configured.api_key.trim().is_empty() {
            configured.api_key.clone()
        } else {
            String::new()
        },
        model: if !configured.model.trim().is_empty() {
            configured.model.clone()
        } else {
            config.llm.openrouter.auxiliary_model().to_string()
        },
    }
}

pub(super) fn seed_llm_profile_settings(config: &HoneConfig) -> LlmProfileSettings {
    LlmProfileSettings {
        default_profile: config.llm.default_profile.clone(),
        auxiliary_profile: config.llm.auxiliary_profile.clone(),
        polish_profile: config.event_engine.renderer.polish_llm.clone(),
        news_classifier_profile: config.event_engine.news_classifier_llm.clone(),
        filing_summary_profile: config.event_engine.sec_filings.enrichment.llm.clone(),
        earnings_quality_profile: config.event_engine.earnings.quality_review.llm.clone(),
        digest_pass1_profile: config.event_engine.global_digest.pass1_llm.clone(),
        digest_pass2_profile: config.event_engine.global_digest.pass2_llm.clone(),
        digest_event_dedupe_profile: config.event_engine.global_digest.event_dedupe_llm.clone(),
        mainline_distill_profile: config
            .event_engine
            .global_digest
            .mainline_distill_llm
            .clone(),
        profiles: LLM_PROFILE_UI_IDS
            .iter()
            .map(|id| seed_llm_profile_entry(config, id))
            .collect(),
    }
}

fn seed_llm_profile_entry(config: &HoneConfig, id: &str) -> LlmProfileEntrySettings {
    let profile = config.llm.profiles.get(id);
    let params = profile.map(|profile| &profile.params);
    LlmProfileEntrySettings {
        id: id.to_string(),
        provider: profile
            .map(|profile| profile.provider.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "openrouter".to_string()),
        model: profile
            .map(|profile| profile.model.clone())
            .unwrap_or_default(),
        max_tokens: params.and_then(|params| params.max_tokens),
        temperature: params.and_then(|params| params.temperature),
        top_p: params.and_then(|params| params.top_p),
        reasoning_effort: params
            .and_then(|params| params.reasoning.as_ref())
            .and_then(|reasoning| reasoning.effort.clone()),
        reasoning_max_tokens: params
            .and_then(|params| params.reasoning.as_ref())
            .and_then(|reasoning| reasoning.max_tokens),
        response_format_json: params
            .and_then(|params| params.response_format.as_ref())
            .and_then(|value| value.get("type"))
            .and_then(|value| value.as_str())
            == Some("json_object"),
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
    updates: Vec<(impl AsRef<str>, serde_yaml::Value)>,
) -> Result<HoneConfig, String> {
    let mutations = updates
        .into_iter()
        .map(|(path, value)| ConfigMutation::Set {
            path: path.as_ref().to_string(),
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
