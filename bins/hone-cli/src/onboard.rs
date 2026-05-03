//! `hone-cli onboard` —— 首次安装后的向导式配置流程。
//!
//! 整体结构:
//! 1. 展示当前 runner 选项(multi-agent / codex_cli / codex_acp / opencode_acp)+
//!    binary 可用性,让用户选一种默认 runner
//!    ([`prompt_onboard_runner`] + [`build_runner_onboard_mutations`])
//! 2. 对每个渠道 (iMessage / Feishu / Telegram / Discord) 询问是否启用,
//!    启用的再逐字段收集必填项,展示 allow_* 默认开放的安全警告,
//!    最后询问 chat_scope。非 macOS 平台自动跳过 iMessage。
//!    ([`build_channel_onboard_mutations`])
//! 3. 根据上一步真正启用的渠道,逐个询问是否把自己加进对应 `admins.*` 白名单
//!    ([`build_admin_onboard_mutations`])
//! 4. 对每个 provider (OpenRouter / FMP / Tavily) 询问是否现在填 key
//!    ([`build_provider_onboard_mutations`])
//! 5. 把所有 mutation 一次性写入 canonical config,并重生成 effective config,
//!    打印写入字段数量摘要
//! 6. 可选运行 doctor / 直接 start
//!
//! 所有 Spec struct(`RunnerOnboardSpec` / `ChannelOnboardSpec` /
//! `ProviderOnboardSpec`) 都是 `&'static` 常量数据,放在各自的 `*_specs()`
//! 工厂函数里,方便未来改文案/加新 runner 时集中维护。
//!
//! 交互契约:任意步骤 Ctrl+C 都安全 —— mutation 只在第 5 步才真正写盘。

use std::io::IsTerminal;
use std::path::Path;

use clap::Args;
use dialoguer::theme::ColorfulTheme;
use serde_yaml::Value;

use hone_core::config::ConfigMutation;

use crate::common::load_cli_config;
use crate::discord_token::{DiscordTokenValidation, validate_discord_token};
use crate::display::{
    banner, bullet, fail_line, hint_line, ok_line, step_header, subsection, warn_line,
};
use crate::i18n::{Lang, detect_initial_lang, t, tpl};
use crate::mutations::{ChannelKind, build_provider_api_key_mutations};
use crate::prompts::{
    ProviderEmptyAction, RequiredFieldEmptyAction, RequiredFieldResolution,
    normalize_credential_value, prompt_bool, prompt_channel_recovery_action,
    prompt_provider_recovery_action, prompt_secret, prompt_select_index, prompt_text,
    prompt_visible_credential, resolve_required_secret_attempt,
};
use crate::reports::{binary_check, build_doctor_report, print_doctor_report_text};
use crate::yaml_io::{apply_message, apply_mutations_and_generate};
use crate::{CliChatScope, start};

/// Onboard 总共的步骤数,供 step_header 显示「N/TOTAL」。
/// language → runner → channels → admins → providers → notifications → apply。
const ONBOARD_TOTAL_STEPS: usize = 7;

/// `hone-cli onboard` 的命令行参数(目前为空,保留结构以便将来扩展 `--skip`、
/// `--runner` 等非交互覆盖)。
#[derive(Args, Debug, Default)]
pub(crate) struct OnboardArgs {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OnboardRunnerKind {
    /// 默认推荐:纯 OpenRouter 路由,不需要本机 CLI。
    MultiAgent,
    CodexCli,
    CodexAcp,
    OpencodeAcp,
}

impl OnboardRunnerKind {
    pub(crate) fn config_value(&self) -> &'static str {
        match self {
            Self::MultiAgent => "multi-agent",
            Self::CodexCli => "codex_cli",
            Self::CodexAcp => "codex_acp",
            Self::OpencodeAcp => "opencode_acp",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::MultiAgent => "Multi-Agent (OpenRouter)",
            Self::CodexCli => "Codex CLI",
            Self::CodexAcp => "Codex ACP",
            Self::OpencodeAcp => "OpenCode ACP",
        }
    }

    /// 对应 CLI 的 probe 指令（`codex --version` 等),用于在选择时判断本机是否已装。
    /// 返回 `None` 表示该 runner 不依赖本机 binary(例如 multi-agent 纯走 HTTP)。
    fn binary_probe(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::MultiAgent => None,
            Self::CodexCli => Some(("codex", "--version")),
            Self::CodexAcp => Some(("codex-acp", "--help")),
            Self::OpencodeAcp => Some(("opencode", "--version")),
        }
    }
}

/// `description` 与 `notes` 都是 i18n 翻译键,真正展示前需通过 `t(lang, key)` 解析。
#[derive(Clone, Copy)]
struct RunnerOnboardSpec {
    kind: OnboardRunnerKind,
    description_key: &'static str,
    note_keys: &'static [&'static str],
}

/// 渠道配置中每个必填字段的类型标签。用于统一驱动 prompt 循环。
#[derive(Clone, Copy)]
enum ChannelRequiredField {
    FeishuAppId,
    FeishuAppSecret,
    TelegramBotToken,
    DiscordBotToken,
}

/// `label_key` / `status_note_key` / `permission_note_keys` 全部是 i18n 翻译键,
/// 调用方在展示前用 `t(lang, key)` 解析。
#[derive(Clone, Copy)]
struct ChannelOnboardSpec {
    kind: ChannelKind,
    label_key: &'static str,
    /// 有些渠道在展示时需要附加警示(例如 Telegram 目前是实验性,iMessage 仅 macOS)。
    status_note_key: Option<&'static str>,
    /// 「启用前置」级别的说明,展示给用户看清楚本地需要什么。
    permission_note_keys: &'static [&'static str],
    /// 启用时必须收集的字段。
    required_fields: &'static [ChannelRequiredField],
    /// 该渠道是否需要在最后让用户选 chat_scope(iMessage 不支持,群聊模型差异)。
    supports_chat_scope: bool,
}

/// `label_key` / `prompt_key` / `note_keys` 都是 i18n 翻译键。
#[derive(Clone, Copy)]
struct ProviderOnboardSpec {
    label_key: &'static str,
    key_path: &'static str,
    legacy_single_key_path: Option<&'static str>,
    prompt_key: &'static str,
    note_keys: &'static [&'static str],
}

fn runner_onboard_specs() -> &'static [RunnerOnboardSpec] {
    &[
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::MultiAgent,
            description_key: "runner.multi_agent.description",
            note_keys: &[
                "runner.multi_agent.note_1",
                "runner.multi_agent.note_2",
                "runner.multi_agent.note_3",
                "runner.multi_agent.note_4",
            ],
        },
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::CodexCli,
            description_key: "runner.codex_cli.description",
            note_keys: &[
                "runner.codex_cli.note_1",
                "runner.codex_cli.note_2",
                "runner.codex_cli.note_3",
                "runner.codex_cli.note_4",
            ],
        },
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::CodexAcp,
            description_key: "runner.codex_acp.description",
            note_keys: &[
                "runner.codex_acp.note_1",
                "runner.codex_acp.note_2",
                "runner.codex_acp.note_3",
                "runner.codex_acp.note_4",
                "runner.codex_acp.note_5",
            ],
        },
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::OpencodeAcp,
            description_key: "runner.opencode_acp.description",
            note_keys: &[
                "runner.opencode_acp.note_1",
                "runner.opencode_acp.note_2",
                "runner.opencode_acp.note_3",
                "runner.opencode_acp.note_4",
                "runner.opencode_acp.note_5",
                "runner.opencode_acp.note_6",
            ],
        },
    ]
}

fn channel_onboard_specs() -> &'static [ChannelOnboardSpec] {
    &[
        ChannelOnboardSpec {
            kind: ChannelKind::Imessage,
            label_key: "channel.imessage.label",
            status_note_key: Some("channel.imessage.status_note"),
            permission_note_keys: &[
                "channel.imessage.note_1",
                "channel.imessage.note_2",
                "channel.imessage.note_3",
            ],
            required_fields: &[],
            supports_chat_scope: false,
        },
        ChannelOnboardSpec {
            kind: ChannelKind::Feishu,
            label_key: "channel.feishu.label",
            status_note_key: None,
            permission_note_keys: &[
                "channel.feishu.note_1",
                "channel.feishu.note_2",
                "channel.feishu.note_3",
            ],
            required_fields: &[
                ChannelRequiredField::FeishuAppId,
                ChannelRequiredField::FeishuAppSecret,
            ],
            supports_chat_scope: true,
        },
        ChannelOnboardSpec {
            kind: ChannelKind::Telegram,
            label_key: "channel.telegram.label",
            status_note_key: Some("channel.telegram.status_note"),
            permission_note_keys: &[
                "channel.telegram.note_1",
                "channel.telegram.note_2",
                "channel.telegram.note_3",
            ],
            required_fields: &[ChannelRequiredField::TelegramBotToken],
            supports_chat_scope: true,
        },
        ChannelOnboardSpec {
            kind: ChannelKind::Discord,
            label_key: "channel.discord.label",
            status_note_key: None,
            permission_note_keys: &[
                "channel.discord.note_1",
                "channel.discord.note_2",
                "channel.discord.note_3",
            ],
            required_fields: &[ChannelRequiredField::DiscordBotToken],
            supports_chat_scope: true,
        },
    ]
}

fn provider_onboard_specs() -> &'static [ProviderOnboardSpec] {
    &[
        // OpenRouter 放最前。对 multi-agent / codex_acp / codex_cli / nano_banana 都是
        // 硬依赖,只有 opencode_acp (且用户已在 opencode 里配好 provider)可以跳过。
        // 早期版本的 onboard 完全没问这个 key,新用户跑完向导发消息立刻报
        // 「openrouter.api_key 为空」,体验很差。
        ProviderOnboardSpec {
            label_key: "provider.openrouter.label",
            key_path: "llm.openrouter.api_keys",
            legacy_single_key_path: Some("llm.openrouter.api_key"),
            prompt_key: "provider.openrouter.prompt",
            note_keys: &[
                "provider.openrouter.note_1",
                "provider.openrouter.note_2",
                "provider.openrouter.note_3",
            ],
        },
        ProviderOnboardSpec {
            label_key: "provider.fmp.label",
            key_path: "fmp.api_keys",
            legacy_single_key_path: Some("fmp.api_key"),
            prompt_key: "provider.fmp.prompt",
            note_keys: &["provider.fmp.note_1", "provider.fmp.note_2"],
        },
        ProviderOnboardSpec {
            label_key: "provider.tavily.label",
            key_path: "search.api_keys",
            legacy_single_key_path: None,
            prompt_key: "provider.tavily.prompt",
            note_keys: &["provider.tavily.note_1", "provider.tavily.note_2"],
        },
    ]
}

fn print_onboard_block(title: &str, lines: &[&str]) {
    subsection(title);
    for line in lines {
        bullet(line);
    }
}

// 让部署者明确感知到「新用户被静默订阅了什么」——目前没有可配置入口,
// 想改默认得改源码。这里只做告知,不写 mutation。
fn print_notifications_awareness_step(lang: Lang) {
    step_header(6, ONBOARD_TOTAL_STEPS, t(lang, "step.notifications"));
    print_onboard_block(
        t(lang, "notifications.defaults_title"),
        &[
            t(lang, "notifications.defaults_1"),
            t(lang, "notifications.defaults_2"),
            t(lang, "notifications.defaults_3"),
        ],
    );
    print_onboard_block(
        t(lang, "notifications.user_adjust_title"),
        &[
            t(lang, "notifications.user_adjust_1"),
            t(lang, "notifications.user_adjust_2"),
        ],
    );
    print_onboard_block(
        t(lang, "notifications.change_default_title"),
        &[
            t(lang, "notifications.change_default_1"),
            t(lang, "notifications.change_default_2"),
        ],
    );
    hint_line(t(lang, "notifications.advance_hint"));
}

// ── Discord token-specific 恢复决策。和 prompts 里的 channel recovery 类似,
// 但选项文案针对「token 格式不合法」定制。

fn prompt_discord_token_invalid_recovery_action(
    theme: &ColorfulTheme,
    lang: Lang,
    channel_label: &str,
) -> Result<RequiredFieldEmptyAction, String> {
    let items = vec![
        t(lang, "recovery.option_discord_token_retry").to_string(),
        tpl(
            t(lang, "recovery.option_disable_channel"),
            &[("label", &channel_label)],
        ),
    ];
    let idx = prompt_select_index(
        theme,
        &tpl(
            t(lang, "recovery.discord_token_invalid_prompt"),
            &[("label", &channel_label)],
        ),
        &items,
        0,
    )?;
    Ok(match idx {
        0 => RequiredFieldEmptyAction::Retry,
        _ => RequiredFieldEmptyAction::DisableChannel,
    })
}

fn prompt_onboard_required_text(
    theme: &ColorfulTheme,
    lang: Lang,
    channel_label: &str,
    prompt: &str,
    current: &str,
) -> Result<Option<String>, String> {
    loop {
        let attempted = prompt_text(theme, prompt, current)?;
        if !attempted.trim().is_empty() {
            return Ok(Some(attempted));
        }
        if !current.trim().is_empty() {
            return Ok(Some(current.to_string()));
        }
        match prompt_channel_recovery_action(theme, lang, channel_label, prompt)? {
            RequiredFieldEmptyAction::Retry => {
                println!("{}", t(lang, "channel.required_field_empty"));
            }
            RequiredFieldEmptyAction::DisableChannel => return Ok(None),
        }
    }
}

fn prompt_onboard_required_secret(
    theme: &ColorfulTheme,
    lang: Lang,
    channel_label: &str,
    prompt: &str,
    current: &str,
) -> Result<Option<String>, String> {
    loop {
        let attempted = prompt_secret(theme, lang, prompt, !current.trim().is_empty())?;
        let resolution = resolve_required_secret_attempt(attempted, current, || {
            prompt_channel_recovery_action(theme, lang, channel_label, prompt)
        })?;
        match resolution {
            RequiredFieldResolution::Value(value) => return Ok(Some(value)),
            RequiredFieldResolution::Retry => {
                println!("{}", t(lang, "channel.required_field_empty"));
            }
            RequiredFieldResolution::DisableChannel => return Ok(None),
        }
    }
}

fn prompt_onboard_required_token(
    theme: &ColorfulTheme,
    lang: Lang,
    channel_label: &str,
    prompt: &str,
    current: &str,
) -> Result<Option<String>, String> {
    loop {
        let attempted =
            prompt_visible_credential(theme, lang, prompt, !current.trim().is_empty(), current)?;
        let resolution = resolve_required_secret_attempt(attempted, current, || {
            prompt_channel_recovery_action(theme, lang, channel_label, prompt)
        })?;
        match resolution {
            RequiredFieldResolution::Value(value) => return Ok(Some(value)),
            RequiredFieldResolution::Retry => {
                println!("{}", t(lang, "channel.required_field_empty"));
            }
            RequiredFieldResolution::DisableChannel => return Ok(None),
        }
    }
}

/// Discord 专用:在通用 token prompt 之上叠加格式校验(三段 base64url、长度合理)。
/// Warn 级别允许用户继续,Invalid 级别会触发 [`prompt_discord_token_invalid_recovery_action`]。
fn prompt_onboard_required_discord_token(
    theme: &ColorfulTheme,
    lang: Lang,
    channel_label: &str,
    prompt: &str,
    current: &str,
) -> Result<Option<String>, String> {
    loop {
        let attempted =
            prompt_visible_credential(theme, lang, prompt, !current.trim().is_empty(), current)?;
        let resolution = match attempted {
            Some(value) => RequiredFieldResolution::Value(value),
            _ if !current.trim().is_empty() => {
                RequiredFieldResolution::Value(normalize_credential_value(current))
            }
            _ => match prompt_channel_recovery_action(theme, lang, channel_label, prompt)? {
                RequiredFieldEmptyAction::Retry => RequiredFieldResolution::Retry,
                RequiredFieldEmptyAction::DisableChannel => RequiredFieldResolution::DisableChannel,
            },
        };
        match resolution {
            RequiredFieldResolution::Value(value) => {
                let normalized_value = normalize_credential_value(&value);
                let len = normalized_value.len();
                match validate_discord_token(&normalized_value) {
                    DiscordTokenValidation::Valid => {
                        ok_line(&tpl(
                            t(lang, "discord_token.valid_with_len"),
                            &[("len", &len)],
                        ));
                        return Ok(Some(normalized_value));
                    }
                    DiscordTokenValidation::Warn(key) => {
                        warn_line(&tpl(
                            t(lang, "discord_token.message_with_len"),
                            &[("message", &t(lang, key)), ("len", &len)],
                        ));
                        if prompt_bool(theme, t(lang, "discord_token.confirm_use"), false)? {
                            return Ok(Some(normalized_value));
                        }
                    }
                    DiscordTokenValidation::Invalid(key) => {
                        fail_line(&tpl(
                            t(lang, "discord_token.message_with_len"),
                            &[("message", &t(lang, key)), ("len", &len)],
                        ));
                        match prompt_discord_token_invalid_recovery_action(
                            theme,
                            lang,
                            channel_label,
                        )? {
                            RequiredFieldEmptyAction::Retry => {
                                hint_line(t(lang, "recovery.discord_token_retry_hint"));
                            }
                            RequiredFieldEmptyAction::DisableChannel => return Ok(None),
                        }
                    }
                }
            }
            RequiredFieldResolution::Retry => {
                println!("{}", t(lang, "channel.required_field_empty"));
            }
            RequiredFieldResolution::DisableChannel => return Ok(None),
        }
    }
}

fn prompt_chat_scope(
    theme: &ColorfulTheme,
    prompt: &str,
    current: hone_core::config::ChatScope,
) -> Result<CliChatScope, String> {
    let current = CliChatScope::from_chat_scope(current);
    let scopes = [
        CliChatScope::DmOnly,
        CliChatScope::GroupchatOnly,
        CliChatScope::All,
    ];
    let items = scopes
        .iter()
        .map(|scope| scope.label().to_string())
        .collect::<Vec<_>>();
    let default = scopes
        .iter()
        .position(|scope| *scope == current)
        .unwrap_or(0);
    let idx = prompt_select_index(theme, prompt, &items, default)?;
    Ok(scopes[idx].clone())
}

fn has_configured_search_keys(config: &hone_core::HoneConfig) -> bool {
    !config
        .search
        .api_keys
        .iter()
        .all(|key| key.trim().is_empty())
}

fn has_configured_provider_keys(
    spec: &ProviderOnboardSpec,
    config: &hone_core::HoneConfig,
) -> bool {
    match spec.key_path {
        "fmp.api_keys" => !config.fmp.effective_key_pool().is_empty(),
        "search.api_keys" => has_configured_search_keys(config),
        "llm.openrouter.api_keys" => !config.llm.openrouter.effective_key_pool().is_empty(),
        _ => false,
    }
}

fn prompt_onboard_provider_keys(
    theme: &ColorfulTheme,
    lang: Lang,
    provider_label: &str,
    prompt: &str,
    current_configured: bool,
) -> Result<Option<Vec<String>>, String> {
    loop {
        let attempted = prompt_secret(theme, lang, prompt, current_configured)?;
        match attempted {
            Some(raw) => {
                let keys = crate::mutations::parse_csv_values(&raw);
                if !keys.is_empty() {
                    return Ok(Some(keys));
                }
            }
            None if current_configured => return Ok(None),
            None => {}
        }

        match prompt_provider_recovery_action(theme, lang, provider_label)? {
            ProviderEmptyAction::Retry => {
                println!("{}", t(lang, "provider.keys_required_or_skip"));
            }
            ProviderEmptyAction::Skip => return Ok(None),
        }
    }
}

pub(crate) fn prompt_onboard_runner(
    theme: &ColorfulTheme,
    lang: Lang,
    config: &hone_core::HoneConfig,
) -> Result<OnboardRunnerKind, String> {
    step_header(2, ONBOARD_TOTAL_STEPS, t(lang, "step.runner"));

    let specs = runner_onboard_specs();
    // label 只放「title [badge]」,description 长文案下移到选定后再展开,
    // 否则 dialoguer 的 Select 在窄终端会把单个 item 截成多行,视觉上糊成一团。
    let labels = specs
        .iter()
        .map(|spec| {
            let badge = match spec.kind.binary_probe() {
                None => t(lang, "runner.badge_no_binary").to_string(),
                Some((binary, help_arg)) => {
                    if binary_check(binary, help_arg).available {
                        tpl(t(lang, "runner.badge_installed"), &[("binary", &binary)])
                    } else {
                        tpl(t(lang, "runner.badge_missing"), &[("binary", &binary)])
                    }
                }
            };
            format!("{} [{}]", spec.kind.title(), badge)
        })
        .collect::<Vec<_>>();
    let default = specs
        .iter()
        .position(|spec| spec.kind.config_value() == config.agent.runner.trim())
        .unwrap_or(0);

    loop {
        let idx = prompt_select_index(theme, t(lang, "runner.choose_default"), &labels, default)?;
        let selected = specs[idx];
        hint_line(t(lang, selected.description_key));
        let notes = selected
            .note_keys
            .iter()
            .map(|key| t(lang, key))
            .collect::<Vec<_>>();
        print_onboard_block(selected.kind.title(), &notes);

        // 不依赖 binary(如 multi-agent)直接通过。
        let Some((binary, help_arg)) = selected.kind.binary_probe() else {
            return Ok(selected.kind);
        };

        let status = binary_check(binary, help_arg);
        if status.available {
            ok_line(&tpl(
                t(lang, "runner.binary_detected"),
                &[("binary", &binary)],
            ));
            return Ok(selected.kind);
        }
        fail_line(&tpl(
            t(lang, "runner.binary_missing_detail"),
            &[("binary", &binary), ("detail", &status.detail)],
        ));
        // 选 true 会继续用当前 runner(配置会写入,运行时才会因缺 binary 报错);
        // 选 false 会回到 runner 选单重新挑一个(最常见路径)。
        if prompt_bool(theme, t(lang, "runner.keep_without_binary"), false)? {
            return Ok(selected.kind);
        }
    }
}

pub(crate) fn build_runner_onboard_mutations(
    theme: &ColorfulTheme,
    lang: Lang,
    config: &hone_core::HoneConfig,
    runner: OnboardRunnerKind,
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = vec![ConfigMutation::Set {
        path: "agent.runner".to_string(),
        value: Value::String(runner.config_value().to_string()),
    }];

    match runner {
        OnboardRunnerKind::MultiAgent => {
            // Multi-agent 不需要本机 binary,也不在这里填 OpenRouter key
            // (留给统一的 Providers 环节处理,避免 key 散布在多个地方)。
            let _ = theme;
            let _ = config;
            print_onboard_block(
                t(lang, "runner.multi_agent.setup_title"),
                &[
                    t(lang, "runner.multi_agent.setup_note_1"),
                    t(lang, "runner.multi_agent.setup_note_2"),
                    t(lang, "runner.multi_agent.setup_note_3"),
                ],
            );
        }
        OnboardRunnerKind::CodexCli => {
            let codex_model = prompt_text(
                theme,
                t(lang, "runner.codex_cli.model_prompt"),
                &config.agent.codex_model,
            )?;
            mutations.push(ConfigMutation::Set {
                path: "agent.codex_model".to_string(),
                value: Value::String(codex_model),
            });
        }
        OnboardRunnerKind::CodexAcp => {
            let model = prompt_text(
                theme,
                t(lang, "runner.codex_acp.model_prompt"),
                &config.agent.codex_acp.model,
            )?;
            let variant = prompt_text(
                theme,
                t(lang, "runner.codex_acp.variant_prompt"),
                &config.agent.codex_acp.variant,
            )?;
            mutations.extend([
                ConfigMutation::Set {
                    path: "agent.codex_acp.model".to_string(),
                    value: Value::String(model),
                },
                ConfigMutation::Set {
                    path: "agent.codex_acp.variant".to_string(),
                    value: Value::String(variant),
                },
            ]);
        }
        OnboardRunnerKind::OpencodeAcp => {
            // opencode 的 provider / model 来源于用户已有的 opencode 配置,
            // Hone 不在首装里抢占 (避免把用户 opencode 里已有的 provider 登录态覆盖掉)。
            let _ = config;
            print_onboard_block(
                t(lang, "runner.opencode_acp.setup_title"),
                &[
                    t(lang, "runner.opencode_acp.setup_note_1"),
                    t(lang, "runner.opencode_acp.setup_note_2"),
                    t(lang, "runner.opencode_acp.setup_note_3"),
                ],
            );
            // 不写入任何东西,只是给用户一个"我意识到你可能还没 /connect" 的心理反馈。
            if !prompt_bool(theme, t(lang, "runner.opencode_acp.confirm_connected"), true)? {
                println!("{}", t(lang, "runner.opencode_acp.warn_not_connected"));
            }
        }
    }

    Ok(mutations)
}

pub(crate) fn build_channel_onboard_mutations(
    theme: &ColorfulTheme,
    lang: Lang,
    config: &hone_core::HoneConfig,
    enabled_channels: &mut Vec<ChannelKind>,
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    step_header(3, ONBOARD_TOTAL_STEPS, t(lang, "step.channels"));
    hint_line(t(lang, "channel.hint_skip"));

    for spec in channel_onboard_specs() {
        let label = t(lang, spec.label_key);
        // iMessage 在非 macOS 平台不可用(依赖 AppleScript + chat.db),直接 skip
        // 以免让 Linux 用户对一个铁定用不了的 channel 回答一堆问题。
        if spec.kind == ChannelKind::Imessage && !cfg!(target_os = "macos") {
            println!();
            hint_line(t(lang, "channel.imessage.skipped_non_macos"));
            continue;
        }

        let current_enabled = match spec.kind {
            ChannelKind::Imessage => config.imessage.enabled,
            ChannelKind::Feishu => config.feishu.enabled,
            ChannelKind::Telegram => config.telegram.enabled,
            ChannelKind::Discord => config.discord.enabled,
        };
        let enabled = prompt_bool(
            theme,
            &tpl(t(lang, "channel.enable_prompt"), &[("label", &label)]),
            current_enabled,
        )?;
        let enabled_path = match spec.kind {
            ChannelKind::Imessage => "imessage.enabled",
            ChannelKind::Feishu => "feishu.enabled",
            ChannelKind::Telegram => "telegram.enabled",
            ChannelKind::Discord => "discord.enabled",
        };
        let mut channel_mutations = vec![ConfigMutation::Set {
            path: enabled_path.to_string(),
            value: Value::Bool(enabled),
        }];

        if !enabled {
            mutations.extend(channel_mutations);
            continue;
        }

        if let Some(status_note_key) = spec.status_note_key {
            println!();
            println!("{}: {}", label, t(lang, status_note_key));
        }
        let permission_notes = spec
            .permission_note_keys
            .iter()
            .map(|key| t(lang, key))
            .collect::<Vec<_>>();
        print_onboard_block(
            &tpl(t(lang, "channel.prerequisites_title"), &[("label", &label)]),
            &permission_notes,
        );

        // 安全提醒:每个启用的 channel 默认的 allow_* 白名单都是空,
        // 空表等于「所有人都能触发」。onboard 不做详细白名单配置(太长),
        // 但必须让用户知道这件事,不然 bot 会对陌生人裸奔。
        println!();
        warn_line(&tpl(t(lang, "channel.allow_warn"), &[("label", &label)]));
        hint_line(t(lang, "channel.allow_hint"));

        // 必填字段循环:任一字段让用户选择「放弃整个渠道」都会把 channel_mutations
        // reset 成单条「enabled=false」并 break。
        for field in spec.required_fields {
            match field {
                ChannelRequiredField::FeishuAppId => {
                    let Some(value) = prompt_onboard_required_text(
                        theme,
                        lang,
                        label,
                        t(lang, "channel.feishu.app_id_prompt"),
                        &config.feishu.app_id,
                    )?
                    else {
                        println!(
                            "{}",
                            tpl(t(lang, "channel.disabled_via_recovery"), &[("label", &label)])
                        );
                        channel_mutations = vec![ConfigMutation::Set {
                            path: enabled_path.to_string(),
                            value: Value::Bool(false),
                        }];
                        break;
                    };
                    channel_mutations.push(ConfigMutation::Set {
                        path: "feishu.app_id".to_string(),
                        value: Value::String(value),
                    });
                }
                ChannelRequiredField::FeishuAppSecret => {
                    let Some(value) = prompt_onboard_required_secret(
                        theme,
                        lang,
                        label,
                        t(lang, "channel.feishu.app_secret_prompt"),
                        &config.feishu.app_secret,
                    )?
                    else {
                        println!(
                            "{}",
                            tpl(t(lang, "channel.disabled_via_recovery"), &[("label", &label)])
                        );
                        channel_mutations = vec![ConfigMutation::Set {
                            path: enabled_path.to_string(),
                            value: Value::Bool(false),
                        }];
                        break;
                    };
                    channel_mutations.push(ConfigMutation::Set {
                        path: "feishu.app_secret".to_string(),
                        value: Value::String(value),
                    });
                }
                ChannelRequiredField::TelegramBotToken => {
                    let Some(value) = prompt_onboard_required_token(
                        theme,
                        lang,
                        label,
                        t(lang, "channel.telegram.bot_token_prompt"),
                        &config.telegram.bot_token,
                    )?
                    else {
                        println!(
                            "{}",
                            tpl(t(lang, "channel.disabled_via_recovery"), &[("label", &label)])
                        );
                        channel_mutations = vec![ConfigMutation::Set {
                            path: enabled_path.to_string(),
                            value: Value::Bool(false),
                        }];
                        break;
                    };
                    channel_mutations.push(ConfigMutation::Set {
                        path: "telegram.bot_token".to_string(),
                        value: Value::String(value),
                    });
                }
                ChannelRequiredField::DiscordBotToken => {
                    let Some(value) = prompt_onboard_required_discord_token(
                        theme,
                        lang,
                        label,
                        t(lang, "channel.discord.bot_token_prompt"),
                        &config.discord.bot_token,
                    )?
                    else {
                        println!(
                            "{}",
                            tpl(t(lang, "channel.disabled_via_recovery"), &[("label", &label)])
                        );
                        channel_mutations = vec![ConfigMutation::Set {
                            path: enabled_path.to_string(),
                            value: Value::Bool(false),
                        }];
                        break;
                    };
                    channel_mutations.push(ConfigMutation::Set {
                        path: "discord.bot_token".to_string(),
                        value: Value::String(value),
                    });
                }
            }
        }

        // 如果循环里因为 required_field 缺失而把 channel 重置为 disabled,
        // 就跳过后续的 chat_scope / target_handle 收集。
        let channel_disabled = channel_mutations.len() == 1
            && matches!(
                channel_mutations.first(),
                Some(ConfigMutation::Set { path, value })
                    if path == enabled_path && matches!(value, Value::Bool(false))
            );
        if channel_disabled {
            mutations.extend(channel_mutations);
            continue;
        }

        // 到这里 channel 真的会启用,记下来供后续 admin 环节按 channel 询问 id。
        enabled_channels.push(spec.kind);

        if spec.supports_chat_scope {
            let current_scope = match spec.kind {
                ChannelKind::Feishu => config.feishu.chat_scope,
                ChannelKind::Telegram => config.telegram.chat_scope,
                ChannelKind::Discord => config.discord.chat_scope,
                ChannelKind::Imessage => hone_core::config::ChatScope::DmOnly,
            };
            let scope = prompt_chat_scope(
                theme,
                &tpl(t(lang, "channel.chat_scope_prompt"), &[("label", &label)]),
                current_scope,
            )?;
            let scope_path = match spec.kind {
                ChannelKind::Feishu => "feishu.chat_scope",
                ChannelKind::Telegram => "telegram.chat_scope",
                ChannelKind::Discord => "discord.chat_scope",
                ChannelKind::Imessage => unreachable!(),
            };
            channel_mutations.push(ConfigMutation::Set {
                path: scope_path.to_string(),
                value: Value::String(scope.as_config_value().to_string()),
            });
        }

        if spec.kind == ChannelKind::Imessage {
            let target_handle = prompt_text(
                theme,
                t(lang, "channel.imessage.target_handle_prompt"),
                &config.imessage.target_handle,
            )?;
            channel_mutations.push(ConfigMutation::Set {
                path: "imessage.target_handle".to_string(),
                value: Value::String(target_handle),
            });
        }

        mutations.extend(channel_mutations);
    }

    Ok(mutations)
}

/// 追加一个 `admins.<path>` 到数组:先读现有 list,去重后把 `value` 插进去,
/// 作为 Sequence 整体覆盖回去。空字符串条目被自动过滤,避免 `example: ""`
/// 这种 placeholder 被保留到最终 yaml。
fn append_admin_mutation(path: &'static str, existing: &[String], value: String) -> ConfigMutation {
    let mut items: Vec<String> = existing
        .iter()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    if !items.iter().any(|v| v == &value) {
        items.push(value);
    }
    ConfigMutation::Set {
        path: path.to_string(),
        value: Value::Sequence(items.into_iter().map(Value::String).collect()),
    }
}

/// 询问用户「要不要把自己加为 admin」,逐一根据已启用的渠道问对应 ID。
///
/// admin 名单决定谁能触发:
/// - `/register-admin`(运行时把 actor 升级成管理员)
/// - `/report`(本地私有 workflow 快捷)
/// - 重启 Hone / 危险 tool 等管理员专属工具
///
/// onboard 不做详细白名单管理(要支持列表编辑、移除、多种 id 混填的话 UI 太重)。
/// 这里只做一件事:**对每个启用的渠道,收集一个自己的 id,直接 append 到对应的
/// `admins.<field>` 数组**。用户后续可用 `hone-cli configure` 或直接改 yaml
/// 扩充或清理。
pub(crate) fn build_admin_onboard_mutations(
    theme: &ColorfulTheme,
    lang: Lang,
    config: &hone_core::HoneConfig,
    enabled_channels: &[ChannelKind],
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    if enabled_channels.is_empty() {
        return Ok(mutations);
    }

    step_header(4, ONBOARD_TOTAL_STEPS, t(lang, "step.admins"));
    hint_line(t(lang, "admin.hint_purpose"));
    hint_line(t(lang, "admin.hint_empty"));

    if !prompt_bool(theme, t(lang, "admin.add_self_prompt"), true)? {
        hint_line(t(lang, "admin.skipped_hint"));
        return Ok(mutations);
    }

    for kind in enabled_channels {
        match kind {
            ChannelKind::Imessage => {
                let value = prompt_text(theme, t(lang, "admin.imessage.handle_prompt"), "")?;
                let value = value.trim();
                if !value.is_empty() {
                    mutations.push(append_admin_mutation(
                        "admins.imessage_handles",
                        &config.admins.imessage_handles,
                        value.to_string(),
                    ));
                }
            }
            ChannelKind::Telegram => {
                let value = prompt_text(theme, t(lang, "admin.telegram.user_id_prompt"), "")?;
                let value = value.trim();
                if !value.is_empty() {
                    mutations.push(append_admin_mutation(
                        "admins.telegram_user_ids",
                        &config.admins.telegram_user_ids,
                        value.to_string(),
                    ));
                }
            }
            ChannelKind::Discord => {
                let value = prompt_text(theme, t(lang, "admin.discord.user_id_prompt"), "")?;
                let value = value.trim();
                if !value.is_empty() {
                    mutations.push(append_admin_mutation(
                        "admins.discord_user_ids",
                        &config.admins.discord_user_ids,
                        value.to_string(),
                    ));
                }
            }
            ChannelKind::Feishu => {
                // 飞书管理员识别支持 3 种 id(平台接口不一定都给全),一次收一种即可。
                let choices = vec![
                    t(lang, "admin.feishu.choice_email").to_string(),
                    t(lang, "admin.feishu.choice_mobile").to_string(),
                    t(lang, "admin.feishu.choice_open_id").to_string(),
                    t(lang, "admin.feishu.choice_skip").to_string(),
                ];
                let idx =
                    prompt_select_index(theme, t(lang, "admin.feishu.kind_prompt"), &choices, 0)?;
                match idx {
                    0 => {
                        let value = prompt_text(theme, t(lang, "admin.feishu.email_prompt"), "")?;
                        if !value.trim().is_empty() {
                            mutations.push(append_admin_mutation(
                                "admins.feishu_emails",
                                &config.admins.feishu_emails,
                                value.trim().to_string(),
                            ));
                        }
                    }
                    1 => {
                        let value = prompt_text(theme, t(lang, "admin.feishu.mobile_prompt"), "")?;
                        if !value.trim().is_empty() {
                            mutations.push(append_admin_mutation(
                                "admins.feishu_mobiles",
                                &config.admins.feishu_mobiles,
                                value.trim().to_string(),
                            ));
                        }
                    }
                    2 => {
                        let value =
                            prompt_text(theme, t(lang, "admin.feishu.open_id_prompt"), "")?;
                        if !value.trim().is_empty() {
                            mutations.push(append_admin_mutation(
                                "admins.feishu_open_ids",
                                &config.admins.feishu_open_ids,
                                value.trim().to_string(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(mutations)
}

pub(crate) fn build_provider_onboard_mutations(
    theme: &ColorfulTheme,
    lang: Lang,
    config: &hone_core::HoneConfig,
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    step_header(5, ONBOARD_TOTAL_STEPS, t(lang, "step.providers"));
    hint_line(t(lang, "provider.hint_explicit"));
    hint_line(t(lang, "provider.hint_skip_later"));

    for spec in provider_onboard_specs() {
        let label = t(lang, spec.label_key);
        let current_configured = has_configured_provider_keys(spec, config);
        let notes = spec
            .note_keys
            .iter()
            .map(|key| t(lang, key))
            .collect::<Vec<_>>();
        print_onboard_block(
            &tpl(t(lang, "provider.api_keys_title"), &[("label", &label)]),
            &notes,
        );

        if !prompt_bool(
            theme,
            &tpl(t(lang, "provider.configure_prompt"), &[("label", &label)]),
            current_configured,
        )? {
            hint_line(&tpl(t(lang, "provider.skip_message"), &[("label", &label)]));
            continue;
        }

        if let Some(keys) = prompt_onboard_provider_keys(
            theme,
            lang,
            label,
            t(lang, spec.prompt_key),
            current_configured,
        )? {
            mutations.extend(build_provider_api_key_mutations(
                spec.key_path,
                spec.legacy_single_key_path,
                keys,
            ));
            ok_line(&tpl(t(lang, "provider.saved_message"), &[("label", &label)]));
        } else if current_configured {
            hint_line(&tpl(
                t(lang, "provider.keep_existing_message"),
                &[("label", &label)],
            ));
        } else {
            hint_line(&tpl(t(lang, "provider.skip_message"), &[("label", &label)]));
        }
    }

    Ok(mutations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_admin_mutation_skips_duplicates_and_empty_placeholders() {
        let existing = vec!["".to_string(), "  ".to_string(), "alice".to_string()];
        let mutation = append_admin_mutation("admins.imessage_handles", &existing, "bob".into());
        let ConfigMutation::Set { path, value } = mutation else {
            panic!("expected Set")
        };
        assert_eq!(path, "admins.imessage_handles");
        let Value::Sequence(items) = value else {
            panic!("expected sequence")
        };
        // 空串被 filter 掉,alice 保留,bob 追加。
        assert_eq!(
            items,
            vec![Value::String("alice".into()), Value::String("bob".into()),]
        );
    }

    #[test]
    fn append_admin_mutation_is_idempotent_on_existing_value() {
        let existing = vec!["alice".to_string()];
        let mutation = append_admin_mutation("admins.feishu_emails", &existing, "alice".into());
        let ConfigMutation::Set { value, .. } = mutation else {
            panic!("expected Set")
        };
        let Value::Sequence(items) = value else {
            panic!("expected sequence")
        };
        assert_eq!(items, vec![Value::String("alice".into())]);
    }

    #[test]
    fn onboard_runner_kind_multi_agent_config_value() {
        assert_eq!(OnboardRunnerKind::MultiAgent.config_value(), "multi-agent");
        assert!(OnboardRunnerKind::MultiAgent.binary_probe().is_none());
        assert!(OnboardRunnerKind::CodexCli.binary_probe().is_some());
    }
}

/// Step 1 — pick the console + CLI default language. The choice is persisted
/// to `config.yaml.language` together with the rest of the onboard mutation
/// set; the web admin reads it from `/api/meta` to bootstrap its locale on
/// first load. Default selection comes from `LC_ALL` / `LANG`.
fn prompt_onboard_language(theme: &ColorfulTheme) -> Result<Lang, String> {
    step_header(1, ONBOARD_TOTAL_STEPS, t(Lang::En, "step.language"));
    let detected = detect_initial_lang();
    // Use the detected language for the prompt itself so the very first
    // user-facing string already matches their environment.
    let prompt = t(detected, "lang.prompt");
    let items = vec![
        t(detected, "lang.option_zh").to_string(),
        t(detected, "lang.option_en").to_string(),
    ];
    let default_idx = match detected {
        Lang::Zh => 0,
        Lang::En => 1,
    };
    let idx = prompt_select_index(theme, prompt, &items, default_idx)?;
    let chosen = if idx == 0 { Lang::Zh } else { Lang::En };
    hint_line(t(chosen, "lang.note"));
    Ok(chosen)
}

pub(crate) async fn run_onboard(
    config_path: Option<&Path>,
    _args: OnboardArgs,
) -> Result<(), String> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(t(detect_initial_lang(), "tty.required").to_string());
    }

    let (config, paths) = load_cli_config(config_path, true).map_err(|e| e.to_string())?;
    let theme = ColorfulTheme::default();

    // Step 1 lives outside the original banner so the operator picks language
    // before we commit to printing chrome strings in either locale.
    let lang = prompt_onboard_language(&theme)?;

    banner(t(lang, "banner.title"), t(lang, "banner.subtitle"));
    hint_line(t(lang, "banner.hint"));

    // The language choice persists alongside everything else collected below;
    // applying it last-minute (rather than writing it eagerly in step 1)
    // preserves the existing "Ctrl+C is safe — nothing has been written yet"
    // contract.
    let mut mutations: Vec<ConfigMutation> = vec![ConfigMutation::Set {
        path: "language".into(),
        value: Value::String(lang.as_str().to_string()),
    }];

    let runner = prompt_onboard_runner(&theme, lang, &config)?;
    mutations.extend(build_runner_onboard_mutations(&theme, lang, &config, runner)?);

    // channel 里真正被 enable 的记一份,供 admin 环节按渠道收集对应 id。
    let mut enabled_channels: Vec<ChannelKind> = Vec::new();
    mutations.extend(build_channel_onboard_mutations(
        &theme,
        lang,
        &config,
        &mut enabled_channels,
    )?);
    mutations.extend(build_admin_onboard_mutations(
        &theme,
        lang,
        &config,
        &enabled_channels,
    )?);
    mutations.extend(build_provider_onboard_mutations(&theme, lang, &config)?);

    print_notifications_awareness_step(lang);

    step_header(7, ONBOARD_TOTAL_STEPS, t(lang, "step.apply"));
    let result = apply_mutations_and_generate(&paths, &mutations)?;
    ok_line(&format!(
        "{}{}",
        apply_message(lang, &result.apply),
        tpl(
            t(lang, "apply.fields_written"),
            &[("n", &result.apply.changed_paths.len())],
        ),
    ));
    hint_line(&tpl(
        t(lang, "apply.canonical_path"),
        &[("p", &paths.canonical_config_path.to_string_lossy())],
    ));
    hint_line(&tpl(
        t(lang, "apply.effective_path"),
        &[("p", &paths.effective_config_path.to_string_lossy())],
    ));

    if prompt_bool(&theme, t(lang, "apply.run_doctor"), true)? {
        println!();
        print_doctor_report_text(build_doctor_report(config_path).await);
    }

    if prompt_bool(&theme, t(lang, "apply.start_now"), false)? {
        println!();
        return start::run_start(config_path).await;
    }

    banner(t(lang, "apply.complete"), t(lang, "apply.next_steps"));
    bullet(t(lang, "apply.tip_status"));
    bullet(t(lang, "apply.tip_doctor"));
    bullet(t(lang, "apply.tip_start"));
    Ok(())
}
