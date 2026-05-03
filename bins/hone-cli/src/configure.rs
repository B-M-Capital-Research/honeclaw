//! `hone-cli configure` —— 按 section 驱动的交互式配置编辑器。
//!
//! 与 [`onboard`] 的差别：
//! - `onboard` 是首次入门的**线性流程**,所有默认 section 顺序跑一遍,
//!   每个字段都要用户作出「现在填 / 跳过 / 禁用」三选一的决定
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

use crate::common::load_cli_config;
use crate::discord_token::prompt_optional_discord_token;
use crate::mutations::{parse_csv_values, provider_key_mutation};
use crate::prompts::{prompt_bool, prompt_secret, prompt_text, prompt_visible_credential};
use crate::yaml_io::{apply_message, apply_mutations_and_generate};

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

                // ── Multi-agent search 阶段
                let search_base = prompt_text(
                    &theme,
                    "Multi-agent search base URL",
                    &config.agent.multi_agent.search.base_url,
                )?;
                let search_model = prompt_text(
                    &theme,
                    "Multi-agent search model",
                    &config.agent.multi_agent.search.model,
                )?;
                let search_iterations = prompt_text(
                    &theme,
                    "Multi-agent search max iterations",
                    &config.agent.multi_agent.search.max_iterations.to_string(),
                )?;
                mutations.push(ConfigMutation::Set {
                    path: "agent.multi_agent.search.base_url".to_string(),
                    value: Value::String(search_base),
                });
                mutations.push(ConfigMutation::Set {
                    path: "agent.multi_agent.search.model".to_string(),
                    value: Value::String(search_model),
                });
                mutations.push(ConfigMutation::Set {
                    path: "agent.multi_agent.search.max_iterations".to_string(),
                    value: Value::Number(serde_yaml::Number::from(
                        search_iterations
                            .parse::<u32>()
                            .map_err(|e| e.to_string())?,
                    )),
                });
                if let Some(api_key) =
                    prompt_secret(&theme, lang, "Multi-agent search API key", true)?
                {
                    mutations.push(ConfigMutation::Set {
                        path: "agent.multi_agent.search.api_key".to_string(),
                        value: Value::String(api_key),
                    });
                }

                // ── Multi-agent answer 阶段
                let answer_base = prompt_text(
                    &theme,
                    "Multi-agent answer base URL",
                    &config.agent.multi_agent.answer.api_base_url,
                )?;
                let answer_model = prompt_text(
                    &theme,
                    "Multi-agent answer model",
                    &config.agent.multi_agent.answer.model,
                )?;
                let answer_variant = prompt_text(
                    &theme,
                    "Multi-agent answer variant",
                    &config.agent.multi_agent.answer.variant,
                )?;
                let answer_tool_calls = prompt_text(
                    &theme,
                    "Multi-agent answer max tool calls",
                    &config.agent.multi_agent.answer.max_tool_calls.to_string(),
                )?;
                mutations.push(ConfigMutation::Set {
                    path: "agent.multi_agent.answer.api_base_url".to_string(),
                    value: Value::String(answer_base),
                });
                mutations.push(ConfigMutation::Set {
                    path: "agent.multi_agent.answer.model".to_string(),
                    value: Value::String(answer_model),
                });
                mutations.push(ConfigMutation::Set {
                    path: "agent.multi_agent.answer.variant".to_string(),
                    value: Value::String(answer_variant),
                });
                mutations.push(ConfigMutation::Set {
                    path: "agent.multi_agent.answer.max_tool_calls".to_string(),
                    value: Value::Number(serde_yaml::Number::from(
                        answer_tool_calls
                            .parse::<u32>()
                            .map_err(|e| e.to_string())?,
                    )),
                });
                if let Some(api_key) =
                    prompt_secret(&theme, lang, "Multi-agent answer API key", true)?
                {
                    mutations.push(ConfigMutation::Set {
                        path: "agent.multi_agent.answer.api_key".to_string(),
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
            }
            ConfigureSection::Providers => {
                // Provider keys 走 `*.api_keys` 数组格式;一次性粘贴逗号分隔的多个 key,
                // 顺手把老的 `*.api_key` 单 key 字段清空,防止残留值被运行时当真 key。
                if let Some(keys) =
                    prompt_secret(&theme, lang, "OpenRouter API keys（逗号分隔）", true)?
                {
                    mutations.push(provider_key_mutation(
                        "llm.openrouter.api_keys",
                        parse_csv_values(&keys),
                    ));
                    mutations.push(ConfigMutation::Set {
                        path: "llm.openrouter.api_key".to_string(),
                        value: Value::String(String::new()),
                    });
                }
                if let Some(keys) = prompt_secret(&theme, lang, "FMP API keys（逗号分隔）", true)?
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
                    prompt_secret(&theme, lang, "Tavily API keys（逗号分隔）", true)?
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
