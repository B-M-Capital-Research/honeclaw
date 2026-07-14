//! `hone-cli configure` —— 按 section 驱动的交互式配置编辑器。
//!
//! 与 `hone-cli onboard` 的差别：
//! - `onboard` 是首次入门的**线性流程**,覆盖语言、runner、渠道、管理员、
//!   provider、通知默认说明和最终 apply,并把收集到的 mutation 留到最后写盘
//! - `configure` 是**按需修改**：调用方可以用 `--section agent/channels/providers`
//!   精确选择要改哪块,里面的 prompt 会把现有值作为默认提示,空值 = 保留
//!
//! 本 module 把两件事放在一起：
//! 1. [`ConfigureArgs`] + [`ConfigureSection`] clap 定义
//! 2. [`run_configure`] 的完整交互实现

use std::path::Path;

use clap::{Args, ValueEnum};
use dialoguer::theme::ColorfulTheme;
use serde_yaml::Value;

use hone_core::config::{ConfigMutation, is_sensitive_config_path};

use crate::CliChatScope;
use crate::common::load_cli_config;
use crate::discord_token::prompt_optional_discord_token;
use crate::mutations::{parse_csv_values, provider_key_mutation};
use crate::prompts::{prompt_bool, prompt_secret, prompt_text, prompt_visible_credential};
use crate::yaml_io::{apply_message, apply_mutations_and_generate};

fn csv_default(values: &[String]) -> String {
    values.join(",")
}

fn sequence_mutation(path: &str, csv: &str) -> ConfigMutation {
    ConfigMutation::Set {
        path: path.to_string(),
        value: Value::Sequence(
            parse_csv_values(csv)
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    }
}

fn provider_keys_prompt(lang: crate::i18n::Lang, label: &str) -> String {
    match lang {
        crate::i18n::Lang::Zh => format!("{label} API key（逗号分隔，可填多个）"),
        crate::i18n::Lang::En => format!("{label} API keys (comma-separated)"),
    }
}

#[derive(Args, Debug)]
pub(crate) struct ConfigureArgs {
    #[arg(long = "section", value_enum)]
    pub(crate) sections: Vec<ConfigureSection>,
}

/// 可选的 configure section。CLI 默认值：全部 section 跑一遍
/// （见 [`sections_or_default`]）。
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConfigureSection {
    Agent,
    Channels,
    Providers,
}

/// 当用户没有显式传 `--section` 时,三个 section 按「agent → channels → providers」
/// 顺序全部跑一遍。
fn sections_or_default(sections: &[ConfigureSection]) -> Vec<ConfigureSection> {
    if sections.is_empty() {
        vec![
            ConfigureSection::Agent,
            ConfigureSection::Channels,
            ConfigureSection::Providers,
        ]
    } else {
        sections.to_vec()
    }
}

pub(crate) fn run_configure(config_path: Option<&Path>, args: ConfigureArgs) -> Result<(), String> {
    let (config, paths) = load_cli_config(config_path, true).map_err(|e| e.to_string())?;
    let theme = ColorfulTheme::default();
    // `configure` keeps its existing wording; we resolve the persisted
    // language only to feed the helpers that now require a `Lang` parameter.
    let lang = crate::i18n::Lang::from_locale(config.language);
    let mut mutations = Vec::new();

    for section in sections_or_default(&args.sections) {
        match section {
            ConfigureSection::Agent => {
                // ── Agent runner + Codex CLI 模型
                let runner = prompt_text(&theme, "Runner", &config.agent.runner)?;
                mutations.push(ConfigMutation::Set {
                    path: "agent.runner".to_string(),
                    value: Value::String(runner),
                });

                let codex_model =
                    prompt_text(&theme, "Codex CLI model", &config.agent.codex_model)?;
                mutations.push(ConfigMutation::Set {
                    path: "agent.codex_model".to_string(),
                    value: Value::String(codex_model),
                });

                // ── 主 LLM 路由(OpenAI-compatible 直连 / opencode)
                let opencode_url = prompt_text(
                    &theme,
                    "Primary OpenAI-compatible base URL",
                    &config.agent.opencode.api_base_url,
                )?;
                mutations.push(ConfigMutation::Set {
                    path: "agent.opencode.api_base_url".to_string(),
                    value: Value::String(opencode_url.clone()),
                });
                let opencode_model =
                    prompt_text(&theme, "Primary model", &config.agent.opencode.model)?;
                mutations.push(ConfigMutation::Set {
                    path: "agent.opencode.model".to_string(),
                    value: Value::String(opencode_model.clone()),
                });
                let opencode_variant =
                    prompt_text(&theme, "Primary variant", &config.agent.opencode.variant)?;
                mutations.push(ConfigMutation::Set {
                    path: "agent.opencode.variant".to_string(),
                    value: Value::String(opencode_variant.clone()),
                });
                if let Some(api_key) = prompt_secret(
                    &theme,
                    lang,
                    "Primary API key",
                    is_sensitive_config_path("agent.opencode.api_key"),
                )? {
                    mutations.push(ConfigMutation::Set {
                        path: "agent.opencode.api_key".to_string(),
                        value: Value::String(api_key),
                    });
                }

                // ── 辅助 LLM（heartbeat / compaction 等后台任务用)
                let aux_base =
                    prompt_text(&theme, "Auxiliary base URL", &config.llm.auxiliary.base_url)?;
                let aux_model =
                    prompt_text(&theme, "Auxiliary model", &config.llm.auxiliary.model)?;
                mutations.push(ConfigMutation::Set {
                    path: "llm.auxiliary.base_url".to_string(),
                    value: Value::String(aux_base),
                });
                mutations.push(ConfigMutation::Set {
                    path: "llm.auxiliary.model".to_string(),
                    value: Value::String(aux_model.clone()),
                });
                // 老字段 `openrouter.sub_model` 仍作 auxiliary 的 fallback,保持同步。
                mutations.push(ConfigMutation::Set {
                    path: "llm.openrouter.sub_model".to_string(),
                    value: Value::String(aux_model),
                });
                if let Some(api_key) = prompt_secret(&theme, lang, "Auxiliary API key", true)? {
                    mutations.push(ConfigMutation::Set {
                        path: "llm.auxiliary.api_key".to_string(),
                        value: Value::String(api_key),
                    });
                }
            }
            ConfigureSection::Channels => {
                let imessage_enabled =
                    prompt_bool(&theme, "Enable iMessage channel?", config.imessage.enabled)?;
                mutations.push(ConfigMutation::Set {
                    path: "imessage.enabled".to_string(),
                    value: Value::Bool(imessage_enabled),
                });
                let imessage_target = prompt_text(
                    &theme,
                    "iMessage tracked handle",
                    &config.imessage.target_handle,
                )?;
                mutations.push(ConfigMutation::Set {
                    path: "imessage.target_handle".to_string(),
                    value: Value::String(imessage_target),
                });
                let feishu_enabled =
                    prompt_bool(&theme, "Enable Feishu channel?", config.feishu.enabled)?;
                mutations.push(ConfigMutation::Set {
                    path: "feishu.enabled".to_string(),
                    value: Value::Bool(feishu_enabled),
                });
                let feishu_app_id = prompt_text(&theme, "Feishu app id", &config.feishu.app_id)?;
                mutations.push(ConfigMutation::Set {
                    path: "feishu.app_id".to_string(),
                    value: Value::String(feishu_app_id),
                });
                if let Some(secret) = prompt_secret(&theme, lang, "Feishu app secret", true)? {
                    mutations.push(ConfigMutation::Set {
                        path: "feishu.app_secret".to_string(),
                        value: Value::String(secret),
                    });
                }
                let feishu_scope = prompt_text(
                    &theme,
                    "Feishu chat scope (DM_ONLY/GROUPCHAT_ONLY/ALL)",
                    CliChatScope::from_chat_scope(config.feishu.chat_scope).label(),
                )?;
                mutations.push(ConfigMutation::Set {
                    path: "feishu.chat_scope".to_string(),
                    value: Value::String(feishu_scope),
                });
                let feishu_allow_emails = prompt_text(
                    &theme,
                    "Feishu allowed emails (comma-separated; empty means allow all)",
                    &csv_default(&config.feishu.allow_emails),
                )?;
                mutations.push(sequence_mutation(
                    "feishu.allow_emails",
                    &feishu_allow_emails,
                ));
                let feishu_allow_mobiles = prompt_text(
                    &theme,
                    "Feishu allowed mobile numbers (comma-separated; empty means allow all)",
                    &csv_default(&config.feishu.allow_mobiles),
                )?;
                mutations.push(sequence_mutation(
                    "feishu.allow_mobiles",
                    &feishu_allow_mobiles,
                ));
                let feishu_allow_open_ids = prompt_text(
                    &theme,
                    "Feishu allowed open IDs (comma-separated; empty means allow all)",
                    &csv_default(&config.feishu.allow_open_ids),
                )?;
                mutations.push(sequence_mutation(
                    "feishu.allow_open_ids",
                    &feishu_allow_open_ids,
                ));
                let telegram_enabled =
                    prompt_bool(&theme, "Enable Telegram channel?", config.telegram.enabled)?;
                mutations.push(ConfigMutation::Set {
                    path: "telegram.enabled".to_string(),
                    value: Value::Bool(telegram_enabled),
                });
                if let Some(token) = prompt_visible_credential(
                    &theme,
                    lang,
                    "Telegram bot token",
                    true,
                    &config.telegram.bot_token,
                )? {
                    mutations.push(ConfigMutation::Set {
                        path: "telegram.bot_token".to_string(),
                        value: Value::String(token),
                    });
                }
                let telegram_scope = prompt_text(
                    &theme,
                    "Telegram chat scope (DM_ONLY/GROUPCHAT_ONLY/ALL)",
                    CliChatScope::from_chat_scope(config.telegram.chat_scope).label(),
                )?;
                mutations.push(ConfigMutation::Set {
                    path: "telegram.chat_scope".to_string(),
                    value: Value::String(telegram_scope),
                });
                let telegram_allow_from = prompt_text(
                    &theme,
                    "Telegram allowed users (comma-separated; empty means allow all)",
                    &csv_default(&config.telegram.allow_from),
                )?;
                mutations.push(sequence_mutation(
                    "telegram.allow_from",
                    &telegram_allow_from,
                ));
                let discord_enabled =
                    prompt_bool(&theme, "Enable Discord channel?", config.discord.enabled)?;
                mutations.push(ConfigMutation::Set {
                    path: "discord.enabled".to_string(),
                    value: Value::Bool(discord_enabled),
                });
                if let Some(token) = prompt_optional_discord_token(
                    &theme,
                    lang,
                    "Discord bot token",
                    &config.discord.bot_token,
                    true,
                )? {
                    mutations.push(ConfigMutation::Set {
                        path: "discord.bot_token".to_string(),
                        value: Value::String(token),
                    });
                }
                let discord_scope = prompt_text(
                    &theme,
                    "Discord chat scope (DM_ONLY/GROUPCHAT_ONLY/ALL)",
                    CliChatScope::from_chat_scope(config.discord.chat_scope).label(),
                )?;
                mutations.push(ConfigMutation::Set {
                    path: "discord.chat_scope".to_string(),
                    value: Value::String(discord_scope),
                });
                let discord_allow_from = prompt_text(
                    &theme,
                    "Discord allowed users (comma-separated; empty means allow all)",
                    &csv_default(&config.discord.allow_from),
                )?;
                mutations.push(sequence_mutation("discord.allow_from", &discord_allow_from));
            }
            ConfigureSection::Providers => {
                // Provider keys 走 `*.api_keys` 数组格式;一次性粘贴逗号分隔的多个 key,
                // 顺手把老的 `*.api_key` 单 key 字段清空,防止残留值被运行时当真 key。
                if let Some(keys) = prompt_secret(
                    &theme,
                    lang,
                    &provider_keys_prompt(lang, "OpenRouter"),
                    true,
                )? {
                    mutations.push(provider_key_mutation(
                        "llm.providers.openrouter.api_keys",
                        parse_csv_values(&keys),
                    ));
                    mutations.push(ConfigMutation::Set {
                        path: "llm.providers.openrouter.api_key".to_string(),
                        value: Value::String(String::new()),
                    });
                    mutations.push(provider_key_mutation("llm.openrouter.api_keys", Vec::new()));
                    mutations.push(ConfigMutation::Set {
                        path: "llm.openrouter.api_key".to_string(),
                        value: Value::String(String::new()),
                    });
                }
                if let Some(keys) =
                    prompt_secret(&theme, lang, &provider_keys_prompt(lang, "FMP"), true)?
                {
                    mutations.push(provider_key_mutation(
                        "fmp.api_keys",
                        parse_csv_values(&keys),
                    ));
                    mutations.push(ConfigMutation::Set {
                        path: "fmp.api_key".to_string(),
                        value: Value::String(String::new()),
                    });
                }
                if let Some(keys) =
                    prompt_secret(&theme, lang, &provider_keys_prompt(lang, "Tavily"), true)?
                {
                    mutations.push(provider_key_mutation(
                        "search.api_keys",
                        parse_csv_values(&keys),
                    ));
                }
            }
        }
    }

    let result = apply_mutations_and_generate(&paths, &mutations)?;
    println!("{}", apply_message(lang, &result.apply));
    println!(
        "config={} effective={}",
        paths.canonical_config_path.to_string_lossy(),
        paths.effective_config_path.to_string_lossy()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Lang;

    #[test]
    fn provider_keys_prompt_matches_configured_language() {
        assert_eq!(
            provider_keys_prompt(Lang::Zh, "OpenRouter"),
            "OpenRouter API key（逗号分隔，可填多个）"
        );
        assert_eq!(
            provider_keys_prompt(Lang::En, "OpenRouter"),
            "OpenRouter API keys (comma-separated)"
        );
    }
}
