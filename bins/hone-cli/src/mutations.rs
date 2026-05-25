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
    /// Comma-separated Telegram/Discord allowlist entries.
    #[arg(long)]
    pub allow_from: Option<String>,
    /// Comma-separated Feishu email allowlist entries.
    #[arg(long)]
    pub allow_emails: Option<String>,
    /// Comma-separated Feishu mobile allowlist entries.
    #[arg(long)]
    pub allow_mobiles: Option<String>,
    /// Comma-separated Feishu open_id allowlist entries.
    #[arg(long)]
    pub allow_open_ids: Option<String>,
}

/// `hone-cli channels enable|disable <channel>` 的参数。
#[derive(Args, Debug)]
pub(crate) struct ChannelToggleArgs {
    pub channel: ChannelKind,
}

pub(crate) fn build_model_mutations(args: &ModelsSetArgs) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();

    if let Some(value) = &args.runner {
        push_string_mutation(&mut mutations, "agent.runner", value);
    }
    if let Some(value) = &args.codex_model {
        push_string_mutation(&mut mutations, "agent.codex_model", value);
    }
    if let Some(value) = &args.codex_acp_model {
        push_string_mutation(&mut mutations, "agent.codex_acp.model", value);
    }
    if let Some(value) = &args.codex_acp_variant {
        push_string_mutation(&mut mutations, "agent.codex_acp.variant", value);
    }

    // 主模型路由：同时写 opencode / multi_agent.answer 两条分支,让用户只感知
    // 「主模型」一个概念(两个字段实际由不同 runner 使用)。
    if let Some(value) = &args.base_url {
        push_string_mutations(
            &mut mutations,
            &[
                "agent.opencode.api_base_url",
                "agent.multi_agent.answer.api_base_url",
            ],
            value,
        );
    }
    if let Some(value) = &args.api_key {
        push_secret_mutations(
            &mut mutations,
            &["agent.opencode.api_key", "agent.multi_agent.answer.api_key"],
            value,
        );
    }
    if let Some(value) = &args.model {
        push_string_mutations(
            &mut mutations,
            &["agent.opencode.model", "agent.multi_agent.answer.model"],
            value,
        );
    }
    if let Some(value) = &args.variant {
        push_string_mutations(
            &mut mutations,
            &["agent.opencode.variant", "agent.multi_agent.answer.variant"],
            value,
        );
    }

    // 辅助 LLM（heartbeat / session compaction 等后台任务）。
    if let Some(value) = &args.aux_base_url {
        push_string_mutation(&mut mutations, "llm.auxiliary.base_url", value);
    }
    if let Some(value) = &args.aux_api_key {
        push_secret_mutation(&mut mutations, "llm.auxiliary.api_key", value);
    }
    if let Some(value) = &args.aux_model {
        push_string_mutation(&mut mutations, "llm.auxiliary.model", value);
        // 老字段 `openrouter.sub_model` 仍作为 auxiliary 的 fallback,同步更新。
        push_string_mutation(&mut mutations, "llm.openrouter.sub_model", value);
    }

    // Multi-agent 专属(search / answer 两阶段)的独立字段。
    if let Some(value) = &args.search_base_url {
        push_string_mutation(&mut mutations, "agent.multi_agent.search.base_url", value);
    }
    if let Some(value) = &args.search_api_key {
        push_secret_mutation(&mut mutations, "agent.multi_agent.search.api_key", value);
    }
    if let Some(value) = &args.search_model {
        push_string_mutation(&mut mutations, "agent.multi_agent.search.model", value);
    }
    if let Some(value) = args.search_max_iterations {
        push_number_mutation(
            &mut mutations,
            "agent.multi_agent.search.max_iterations",
            value,
        );
    }

    if let Some(value) = &args.answer_base_url {
        push_string_mutation(
            &mut mutations,
            "agent.multi_agent.answer.api_base_url",
            value,
        );
    }
    if let Some(value) = &args.answer_api_key {
        push_secret_mutation(&mut mutations, "agent.multi_agent.answer.api_key", value);
    }
    if let Some(value) = &args.answer_model {
        push_string_mutation(&mut mutations, "agent.multi_agent.answer.model", value);
    }
    if let Some(value) = &args.answer_variant {
        push_string_mutation(&mut mutations, "agent.multi_agent.answer.variant", value);
    }
    if let Some(value) = args.answer_max_tool_calls {
        push_number_mutation(
            &mut mutations,
            "agent.multi_agent.answer.max_tool_calls",
            value,
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

    match args.channel {
        ChannelKind::Imessage => {
            if let Some(value) = args.enabled {
                push_bool_mutation(&mut mutations, "imessage.enabled", value);
            }
            if let Some(value) = &args.target_handle {
                push_string_mutation(&mut mutations, "imessage.target_handle", value);
            }
            if let Some(value) = &args.db_path {
                push_string_mutation(&mut mutations, "imessage.db_path", value);
            }
            if let Some(value) = args.poll_interval {
                push_number_mutation(&mut mutations, "imessage.poll_interval", value);
            }
        }
        ChannelKind::Feishu => {
            if let Some(value) = args.enabled {
                push_bool_mutation(&mut mutations, "feishu.enabled", value);
            }
            if let Some(value) = &args.app_id {
                push_string_mutation(&mut mutations, "feishu.app_id", value);
            }
            if let Some(value) = &args.app_secret {
                push_secret_mutation(&mut mutations, "feishu.app_secret", value);
            }
            if let Some(value) = &args.chat_scope {
                push_chat_scope_mutation(&mut mutations, "feishu.chat_scope", value);
            }
            if let Some(value) = &args.allow_emails {
                push_csv_sequence_mutation(&mut mutations, "feishu.allow_emails", value);
            }
            if let Some(value) = &args.allow_mobiles {
                push_csv_sequence_mutation(&mut mutations, "feishu.allow_mobiles", value);
            }
            if let Some(value) = &args.allow_open_ids {
                push_csv_sequence_mutation(&mut mutations, "feishu.allow_open_ids", value);
            }
        }
        ChannelKind::Telegram => {
            if let Some(value) = args.enabled {
                push_bool_mutation(&mut mutations, "telegram.enabled", value);
            }
            if let Some(value) = &args.bot_token {
                push_secret_mutation(&mut mutations, "telegram.bot_token", value);
            }
            if let Some(value) = &args.chat_scope {
                push_chat_scope_mutation(&mut mutations, "telegram.chat_scope", value);
            }
            if let Some(value) = &args.allow_from {
                push_csv_sequence_mutation(&mut mutations, "telegram.allow_from", value);
            }
        }
        ChannelKind::Discord => {
            if let Some(value) = args.enabled {
                push_bool_mutation(&mut mutations, "discord.enabled", value);
            }
            if let Some(value) = &args.bot_token {
                push_secret_mutation(&mut mutations, "discord.bot_token", value);
            }
            if let Some(value) = &args.chat_scope {
                push_chat_scope_mutation(&mut mutations, "discord.chat_scope", value);
            }
            if let Some(value) = &args.allow_from {
                push_csv_sequence_mutation(&mut mutations, "discord.allow_from", value);
            }
        }
    }

    if mutations.is_empty() {
        return Err("至少提供一个 channels set 参数".to_string());
    }
    Ok(mutations)
}

fn push_set_mutation(mutations: &mut Vec<ConfigMutation>, path: &str, value: Value) {
    mutations.push(ConfigMutation::Set {
        path: path.to_string(),
        value,
    });
}

fn push_string_mutation(mutations: &mut Vec<ConfigMutation>, path: &str, value: &str) {
    push_set_mutation(mutations, path, Value::String(value.to_string()));
}

fn push_string_mutations(mutations: &mut Vec<ConfigMutation>, paths: &[&str], value: &str) {
    for path in paths {
        push_string_mutation(mutations, path, value);
    }
}

fn push_secret_mutation(mutations: &mut Vec<ConfigMutation>, path: &str, value: &str) {
    push_string_mutation(mutations, path, &normalize_credential_value(value));
}

fn push_secret_mutations(mutations: &mut Vec<ConfigMutation>, paths: &[&str], value: &str) {
    let normalized = normalize_credential_value(value);
    for path in paths {
        push_string_mutation(mutations, path, &normalized);
    }
}

fn push_bool_mutation(mutations: &mut Vec<ConfigMutation>, path: &str, value: bool) {
    push_set_mutation(mutations, path, Value::Bool(value));
}

fn push_number_mutation<N>(mutations: &mut Vec<ConfigMutation>, path: &str, value: N)
where
    N: Into<serde_yaml::Number>,
{
    push_set_mutation(mutations, path, Value::Number(value.into()));
}

fn push_chat_scope_mutation(mutations: &mut Vec<ConfigMutation>, path: &str, value: &CliChatScope) {
    push_string_mutation(mutations, path, value.as_config_value());
}

fn push_csv_sequence_mutation(mutations: &mut Vec<ConfigMutation>, path: &str, value: &str) {
    push_set_mutation(
        mutations,
        path,
        Value::Sequence(
            parse_csv_values(value)
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
    );
}

/// 把一串 key 压成一个 `Sequence<String>` mutation(`search.api_keys` / `fmp.api_keys` 等)。
pub(crate) fn provider_key_mutation(path: &str, keys: Vec<String>) -> ConfigMutation {
    ConfigMutation::Set {
        path: path.to_string(),
        value: Value::Sequence(keys.into_iter().map(Value::String).collect()),
    }
}

/// 同时写 `*.api_keys` 数组,并清空指定的单 key 字段,防止老字段残留值被运行时当成真 key 使用。
pub(crate) fn build_provider_api_key_mutations(
    key_path: &str,
    clear_single_key_paths: &[&str],
    keys: Vec<String>,
) -> Vec<ConfigMutation> {
    let mut mutations = vec![provider_key_mutation(key_path, keys)];
    for path in clear_single_key_paths {
        mutations.push(ConfigMutation::Set {
            path: (*path).to_string(),
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
            ..empty_model_set_args()
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
            aux_api_key: Some("  sk-aux  ".to_string()),
            search_api_key: Some("  sk-search  ".to_string()),
            answer_api_key: Some("  sk-answer  ".to_string()),
            ..empty_model_set_args()
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
            bot_token: Some("token".to_string()),
            chat_scope: Some(CliChatScope::All),
            ..empty_channel_set_args(ChannelKind::Telegram)
        };

        let mutations = build_channel_mutations(&args).unwrap();
        assert_eq!(mutations.len(), 3);
        assert!(mutations.iter().any(|mutation| matches!(mutation, ConfigMutation::Set { path, value: Value::Bool(true) } if path == "telegram.enabled")));
    }

    #[test]
    fn build_channel_mutations_trim_secret_values() {
        let telegram_args = ChannelSetArgs {
            channel: ChannelKind::Telegram,
            bot_token: Some("  tg-token  ".to_string()),
            ..empty_channel_set_args(ChannelKind::Telegram)
        };
        let feishu_args = ChannelSetArgs {
            channel: ChannelKind::Feishu,
            app_secret: Some("  fs-secret  ".to_string()),
            ..empty_channel_set_args(ChannelKind::Feishu)
        };
        let discord_args = ChannelSetArgs {
            channel: ChannelKind::Discord,
            bot_token: Some("  dc-token  ".to_string()),
            ..empty_channel_set_args(ChannelKind::Discord)
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
    fn build_channel_mutations_supports_allowlists() {
        let telegram_args = ChannelSetArgs {
            channel: ChannelKind::Telegram,
            allow_from: Some("123, 456".to_string()),
            ..empty_channel_set_args(ChannelKind::Telegram)
        };
        let mutations = build_channel_mutations(&telegram_args).unwrap();
        assert!(mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::Sequence(values) }
                if path == "telegram.allow_from" && values.len() == 2
        )));

        let feishu_args = ChannelSetArgs {
            channel: ChannelKind::Feishu,
            allow_emails: Some("a@example.com,b@example.com".to_string()),
            allow_mobiles: Some("+8613800138000".to_string()),
            allow_open_ids: Some("ou_abc".to_string()),
            ..empty_channel_set_args(ChannelKind::Feishu)
        };
        let mutations = build_channel_mutations(&feishu_args).unwrap();
        assert!(mutations.iter().any(|mutation| matches!(
            mutation,
            ConfigMutation::Set { path, value: Value::Sequence(values) }
                if path == "feishu.allow_emails" && values.len() == 2
        )));
    }

    fn empty_model_set_args() -> ModelsSetArgs {
        ModelsSetArgs {
            runner: None,
            model: None,
            variant: None,
            base_url: None,
            api_key: None,
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
        }
    }

    fn empty_channel_set_args(channel: ChannelKind) -> ChannelSetArgs {
        ChannelSetArgs {
            channel,
            enabled: None,
            target_handle: None,
            db_path: None,
            poll_interval: None,
            app_id: None,
            app_secret: None,
            bot_token: None,
            chat_scope: None,
            allow_from: None,
            allow_emails: None,
            allow_mobiles: None,
            allow_open_ids: None,
        }
    }

    #[test]
    fn build_provider_api_key_mutations_clears_legacy_fmp_single_key() {
        let mutations = build_provider_api_key_mutations(
            "fmp.api_keys",
            &["fmp.api_key"],
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
            build_provider_api_key_mutations("search.api_keys", &[], vec!["tvly-1".to_string()]);

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
