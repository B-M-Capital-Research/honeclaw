//! `hone-cli models set` / `hone-cli channels set` / `hone-cli onboard` 的参数 struct
//! 以及把它们翻译成 `ConfigMutation` 序列的 pure builder。
//!
//! 设计要点：
//! - 所有 builder 都是纯函数,只接受 `&*Args` 并产出 `Vec<ConfigMutation>`;不做 IO。
//! - `models set` 有意 **镜像写** `agent.opencode.*` 和 `agent.multi_agent.answer.*`。
//!   这么做是因为用户心智模型里只有「主模型路由」一个概念,底层 runner 配置因
//!   不同 runner 而分散在不同字段,镜像能让「切一次 model 全套生效」。
//! - 敏感字段一律走 `normalize_credential_value` 去首尾空白,避免复制粘贴把
//!   意外空格带进 yaml。

use clap::{Args, ValueEnum};
use serde_yaml::Value;

use hone_core::config::ConfigMutation;

use crate::CliChatScope;
use crate::prompts::normalize_credential_value;

/// `hone-cli channels` 子命令的渠道枚举。
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChannelKind {
    Imessage,
    Feishu,
    Telegram,
    Discord,
}

/// `hone-cli models set` 的命令行参数。所有字段都是 `Option`,
/// 只有用户显式传 `--xxx` 时才会生成对应的 mutation。
#[derive(Args, Debug)]
pub(crate) struct ModelsSetArgs {
    #[arg(long)]
    pub runner: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long)]
    pub variant: Option<String>,
    #[arg(long)]
    pub base_url: Option<String>,
    #[arg(long)]
    pub api_key: Option<String>,
    #[arg(long)]
    pub codex_model: Option<String>,
    #[arg(long)]
    pub codex_acp_model: Option<String>,
    #[arg(long)]
    pub codex_acp_variant: Option<String>,
    #[arg(long)]
    pub aux_base_url: Option<String>,
    #[arg(long)]
    pub aux_api_key: Option<String>,
    #[arg(long)]
    pub aux_model: Option<String>,
    #[arg(long)]
    pub search_base_url: Option<String>,
    #[arg(long)]
    pub search_api_key: Option<String>,
    #[arg(long)]
    pub search_model: Option<String>,
    #[arg(long)]
    pub search_max_iterations: Option<u32>,
    #[arg(long)]
    pub answer_base_url: Option<String>,
    #[arg(long)]
    pub answer_api_key: Option<String>,
    #[arg(long)]
    pub answer_model: Option<String>,
    #[arg(long)]
    pub answer_variant: Option<String>,
    #[arg(long)]
    pub answer_max_tool_calls: Option<u32>,
}

/// `hone-cli channels set <channel>` 的命令行参数。字段的语义依赖 `channel`
/// 的值（iMessage 关心 db_path,Telegram/Discord 关心 bot_token,…）。
#[derive(Args, Debug)]
pub(crate) struct ChannelSetArgs {
    pub channel: ChannelKind,
    #[arg(long)]
    pub enabled: Option<bool>,
    #[arg(long)]
    pub target_handle: Option<String>,
    #[arg(long)]
    pub db_path: Option<String>,
    #[arg(long)]
    pub poll_interval: Option<u64>,
    #[arg(long)]
    pub app_id: Option<String>,
    #[arg(long)]
    pub app_secret: Option<String>,
    #[arg(long)]
    pub bot_token: Option<String>,
    #[arg(long, value_enum)]
    pub chat_scope: Option<CliChatScope>,
}

/// `hone-cli channels enable|disable <channel>` 的参数。
#[derive(Args, Debug)]
pub(crate) struct ChannelToggleArgs {
    pub channel: ChannelKind,
}

pub(crate) fn build_model_mutations(args: &ModelsSetArgs) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    let mut push = |path: &str, value: Value| {
        mutations.push(ConfigMutation::Set {
            path: path.to_string(),
            value,
        });
    };

    if let Some(value) = &args.runner {
        push("agent.runner", Value::String(value.clone()));
    }
    if let Some(value) = &args.codex_model {
        push("agent.codex_model", Value::String(value.clone()));
    }
    if let Some(value) = &args.codex_acp_model {
        push("agent.codex_acp.model", Value::String(value.clone()));
    }
    if let Some(value) = &args.codex_acp_variant {
        push("agent.codex_acp.variant", Value::String(value.clone()));
    }

    // 主模型路由：同时写 opencode / multi_agent.answer 两条分支,让用户只感知
    // 「主模型」一个概念(两个字段实际由不同 runner 使用)。
    if let Some(value) = &args.base_url {
        push("agent.opencode.api_base_url", Value::String(value.clone()));
        push(
            "agent.multi_agent.answer.api_base_url",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = &args.api_key {
        let normalized = normalize_credential_value(value);
        push("agent.opencode.api_key", Value::String(normalized.clone()));
        push(
            "agent.multi_agent.answer.api_key",
            Value::String(normalized),
        );
    }
    if let Some(value) = &args.model {
        push("agent.opencode.model", Value::String(value.clone()));
        push(
            "agent.multi_agent.answer.model",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = &args.variant {
        push("agent.opencode.variant", Value::String(value.clone()));
        push(
            "agent.multi_agent.answer.variant",
            Value::String(value.clone()),
        );
    }

    // 辅助 LLM（heartbeat / session compaction 等后台任务）。
    if let Some(value) = &args.aux_base_url {
        push("llm.auxiliary.base_url", Value::String(value.clone()));
    }
    if let Some(value) = &args.aux_api_key {
        push(
            "llm.auxiliary.api_key",
            Value::String(normalize_credential_value(value)),
        );
    }
    if let Some(value) = &args.aux_model {
        push("llm.auxiliary.model", Value::String(value.clone()));
        // 老字段 `openrouter.sub_model` 仍作为 auxiliary 的 fallback,同步更新。
        push("llm.openrouter.sub_model", Value::String(value.clone()));
    }

    // Multi-agent 专属(search / answer 两阶段)的独立字段。
    if let Some(value) = &args.search_base_url {
        push(
            "agent.multi_agent.search.base_url",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = &args.search_api_key {
        push(
            "agent.multi_agent.search.api_key",
            Value::String(normalize_credential_value(value)),
        );
    }
    if let Some(value) = &args.search_model {
        push(
            "agent.multi_agent.search.model",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = args.search_max_iterations {
        push(
            "agent.multi_agent.search.max_iterations",
            Value::Number(serde_yaml::Number::from(value)),
        );
    }

    if let Some(value) = &args.answer_base_url {
        push(
            "agent.multi_agent.answer.api_base_url",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = &args.answer_api_key {
        push(
            "agent.multi_agent.answer.api_key",
            Value::String(normalize_credential_value(value)),
        );
    }
    if let Some(value) = &args.answer_model {
        push(
            "agent.multi_agent.answer.model",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = &args.answer_variant {
        push(
            "agent.multi_agent.answer.variant",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = args.answer_max_tool_calls {
        push(
            "agent.multi_agent.answer.max_tool_calls",
            Value::Number(serde_yaml::Number::from(value)),
        );
    }

    if mutations.is_empty() {
        return Err("至少提供一个 models set 参数".to_string());
    }
    Ok(mutations)
}

pub(crate) fn build_channel_mutations(
    args: &ChannelSetArgs,
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    let mut push = |path: &str, value: Value| {
        mutations.push(ConfigMutation::Set {
            path: path.to_string(),
            value,
        });
    };

    match args.channel {
        ChannelKind::Imessage => {
            if let Some(value) = args.enabled {
                push("imessage.enabled", Value::Bool(value));
            }
            if let Some(value) = &args.target_handle {
                push("imessage.target_handle", Value::String(value.clone()));
            }
            if let Some(value) = &args.db_path {
                push("imessage.db_path", Value::String(value.clone()));
            }
            if let Some(value) = args.poll_interval {
                push(
                    "imessage.poll_interval",
                    Value::Number(serde_yaml::Number::from(value)),
                );
            }
        }
        ChannelKind::Feishu => {
            if let Some(value) = args.enabled {
                push("feishu.enabled", Value::Bool(value));
            }
            if let Some(value) = &args.app_id {
                push("feishu.app_id", Value::String(value.clone()));
            }
            if let Some(value) = &args.app_secret {
                push(
                    "feishu.app_secret",
                    Value::String(normalize_credential_value(value)),
                );
            }
            if let Some(value) = &args.chat_scope {
                push(
                    "feishu.chat_scope",
                    Value::String(value.as_config_value().to_string()),
                );
            }
        }
        ChannelKind::Telegram => {
            if let Some(value) = args.enabled {
                push("telegram.enabled", Value::Bool(value));
            }
            if let Some(value) = &args.bot_token {
                push(
                    "telegram.bot_token",
                    Value::String(normalize_credential_value(value)),
                );
            }
            if let Some(value) = &args.chat_scope {
                push(
                    "telegram.chat_scope",
                    Value::String(value.as_config_value().to_string()),
                );
            }
        }
        ChannelKind::Discord => {
            if let Some(value) = args.enabled {
                push("discord.enabled", Value::Bool(value));
            }
            if let Some(value) = &args.bot_token {
                push(
                    "discord.bot_token",
                    Value::String(normalize_credential_value(value)),
                );
            }
            if let Some(value) = &args.chat_scope {
                push(
                    "discord.chat_scope",
                    Value::String(value.as_config_value().to_string()),
                );
            }
        }
    }

    if mutations.is_empty() {
        return Err("至少提供一个 channels set 参数".to_string());
    }
    Ok(mutations)
}

/// 把一串 key 压成一个 `Sequence<String>` mutation(`search.api_keys` / `fmp.api_keys` 等)。
pub(crate) fn provider_key_mutation(path: &str, keys: Vec<String>) -> ConfigMutation {
    ConfigMutation::Set {
        path: path.to_string(),
        value: Value::Sequence(keys.into_iter().map(Value::String).collect()),
    }
}

/// 同时写 `*.api_keys` 数组,并在有 `legacy_single_key_path` 时清空它,防止
/// 老字段残留值被运行时当成真 key 使用。
pub(crate) fn build_provider_api_key_mutations(
    key_path: &str,
    legacy_single_key_path: Option<&str>,
    keys: Vec<String>,
) -> Vec<ConfigMutation> {
    let mut mutations = vec![provider_key_mutation(key_path, keys)];
    if let Some(path) = legacy_single_key_path {
        mutations.push(ConfigMutation::Set {
            path: path.to_string(),
            value: Value::String(String::new()),
        });
    }
    mutations
}

/// 把逗号分隔的字符串拆成非空 trim 过的条目,供 `--api-keys key1,key2` 使用。
pub(crate) fn parse_csv_values(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_model_mutations_mirrors_primary_to_multi_agent_answer() {
        let args = ModelsSetArgs {
            runner: Some("opencode_acp".to_string()),
            model: Some("openrouter/openai/gpt-5.4".to_string()),
            variant: Some("medium".to_string()),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            api_key: Some("sk-test".to_string()),
            codex_model: None,
            codex_acp_model: None,
            codex_acp_variant: None,
            aux_base_url: None,
            aux_api_key: None,
            aux_model: None,
            search_base_url: None,
            search_api_key: None,
            search_model: None,
            search_max_iterations: None,
            answer_base_url: None,
            answer_api_key: None,
            answer_model: None,
            answer_variant: None,
            answer_max_tool_calls: None,
        };

        let mutations = build_model_mutations(&args).unwrap();
        assert!(mutations.iter().any(|mutation| matches!(mutation, ConfigMutation::Set { path, .. } if path == "agent.opencode.model")));
        assert!(mutations.iter().any(|mutation| matches!(mutation, ConfigMutation::Set { path, .. } if path == "agent.multi_agent.answer.model")));
    }

    #[test]
    fn build_model_mutations_trim_secret_values() {
        let args = ModelsSetArgs {
            runner: None,
            model: None,
            variant: None,
            base_url: None,
            api_key: Some("  sk-primary  ".to_string()),
            codex_model: None,
            codex_acp_model: None,
            codex_acp_variant: None,
            aux_base_url: None,
            aux_api_key: Some("  sk-aux  ".to_string()),
            aux_model: None,
            search_base_url: None,
            search_api_key: Some("  sk-search  ".to_string()),
            search_model: None,
            search_max_iterations: None,
            answer_base_url: None,
            answer_api_key: Some("  sk-answer  ".to_string()),
            answer_model: None,
            answer_variant: None,
            answer_max_tool_calls: None,
        };

        let mutations = build_model_mutations(&args).unwrap();
        assert!(mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::String(value) }
                if path == "agent.opencode.api_key" && value == "sk-primary"
        )));
        assert!(mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::String(value) }
                if path == "llm.auxiliary.api_key" && value == "sk-aux"
        )));
        assert!(mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::String(value) }
                if path == "agent.multi_agent.search.api_key" && value == "sk-search"
        )));
        assert!(mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::String(value) }
                if path == "agent.multi_agent.answer.api_key" && value == "sk-answer"
        )));
    }

    #[test]
    fn build_channel_mutations_supports_telegram_toggle_and_token() {
        let args = ChannelSetArgs {
            channel: ChannelKind::Telegram,
            enabled: Some(true),
            target_handle: None,
            db_path: None,
            poll_interval: None,
            app_id: None,
            app_secret: None,
            bot_token: Some("token".to_string()),
            chat_scope: Some(CliChatScope::All),
        };

        let mutations = build_channel_mutations(&args).unwrap();
        assert_eq!(mutations.len(), 3);
        assert!(mutations.iter().any(|mutation| matches!(mutation, ConfigMutation::Set { path, value: Value::Bool(true) } if path == "telegram.enabled")));
    }

    #[test]
    fn build_channel_mutations_trim_secret_values() {
        let telegram_args = ChannelSetArgs {
            channel: ChannelKind::Telegram,
            enabled: None,
            target_handle: None,
            db_path: None,
            poll_interval: None,
            app_id: None,
            app_secret: None,
            bot_token: Some("  tg-token  ".to_string()),
            chat_scope: None,
        };
        let feishu_args = ChannelSetArgs {
            channel: ChannelKind::Feishu,
            enabled: None,
            target_handle: None,
            db_path: None,
            poll_interval: None,
            app_id: None,
            app_secret: Some("  fs-secret  ".to_string()),
            bot_token: None,
            chat_scope: None,
        };
        let discord_args = ChannelSetArgs {
            channel: ChannelKind::Discord,
            enabled: None,
            target_handle: None,
            db_path: None,
            poll_interval: None,
            app_id: None,
            app_secret: None,
            bot_token: Some("  dc-token  ".to_string()),
            chat_scope: None,
        };

        let telegram_mutations = build_channel_mutations(&telegram_args).unwrap();
        let feishu_mutations = build_channel_mutations(&feishu_args).unwrap();
        let discord_mutations = build_channel_mutations(&discord_args).unwrap();

        assert!(telegram_mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::String(value) }
                if path == "telegram.bot_token" && value == "tg-token"
        )));
        assert!(feishu_mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::String(value) }
                if path == "feishu.app_secret" && value == "fs-secret"
        )));
        assert!(discord_mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::String(value) }
                if path == "discord.bot_token" && value == "dc-token"
        )));
    }

    #[test]
    fn build_provider_api_key_mutations_clears_legacy_fmp_single_key() {
        let mutations = build_provider_api_key_mutations(
            "fmp.api_keys",
            Some("fmp.api_key"),
            vec!["key-a".to_string(), "key-b".to_string()],
        );

        assert_eq!(mutations.len(), 2);
        assert!(matches!(
            &mutations[0],
            ConfigMutation::Set {
                path,
                value: Value::Sequence(values),
            } if path == "fmp.api_keys"
                && values == &vec![
                    Value::String("key-a".to_string()),
                    Value::String("key-b".to_string()),
                ]
        ));
        assert!(matches!(
            &mutations[1],
            ConfigMutation::Set {
                path,
                value: Value::String(value),
            } if path == "fmp.api_key" && value.is_empty()
        ));
    }

    #[test]
    fn build_provider_api_key_mutations_sets_tavily_keys_without_legacy_field() {
        let mutations =
            build_provider_api_key_mutations("search.api_keys", None, vec!["tvly-1".to_string()]);

        assert_eq!(mutations.len(), 1);
        assert!(matches!(
            &mutations[0],
            ConfigMutation::Set {
                path,
                value: Value::Sequence(values),
            } if path == "search.api_keys"
                && values == &vec![Value::String("tvly-1".to_string())]
        ));
    }
}
