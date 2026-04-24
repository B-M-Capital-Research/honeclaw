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

#[derive(Clone, Copy)]
struct RunnerOnboardSpec {
    kind: OnboardRunnerKind,
    description: &'static str,
    notes: &'static [&'static str],
}

/// 渠道配置中每个必填字段的类型标签。用于统一驱动 prompt 循环。
#[derive(Clone, Copy)]
enum ChannelRequiredField {
    FeishuAppId,
    FeishuAppSecret,
    TelegramBotToken,
    DiscordBotToken,
}

#[derive(Clone, Copy)]
struct ChannelOnboardSpec {
    kind: ChannelKind,
    label: &'static str,
    /// 有些渠道在展示时需要附加警示(例如 Telegram 目前是实验性,iMessage 仅 macOS)。
    status_note: Option<&'static str>,
    /// 「启用前置」级别的说明,展示给用户看清楚本地需要什么。
    permission_notes: &'static [&'static str],
    /// 启用时必须收集的字段。
    required_fields: &'static [ChannelRequiredField],
    /// 该渠道是否需要在最后让用户选 chat_scope(iMessage 不支持,群聊模型差异)。
    supports_chat_scope: bool,
}

#[derive(Clone, Copy)]
struct ProviderOnboardSpec {
    label: &'static str,
    key_path: &'static str,
    legacy_single_key_path: Option<&'static str>,
    prompt: &'static str,
    notes: &'static [&'static str],
}

fn runner_onboard_specs() -> &'static [RunnerOnboardSpec] {
    &[
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::MultiAgent,
            description: "默认推荐:search + answer 两段式,纯 HTTP 走 OpenRouter,不需要本机 CLI。",
            notes: &[
                "前置:一把可用的 OpenRouter API key(后面 Providers 环节会让你填)。",
                "原理:第一段 search 用小模型拉证据,第二段 answer 用主模型总结。",
                "适合:只有 API key、不想装 CLI 的用户。",
                "需要在本机切换模型时,之后用 `hone-cli models set ...` 即可,不必重跑 onboard。",
            ],
        },
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::CodexCli,
            description: "优先复用本机 codex CLI 登录态；适合已经能直接运行 codex 的用户。",
            notes: &[
                "前置：本机可执行 `codex --version`。",
                "优点：不需要单独填写 OpenAI-compatible base URL / API key。",
                "安装：`npm install -g @openai/codex`；已安装可用 `codex --upgrade` 更新。",
                "官方说明：https://help.openai.com/en/articles/11096431",
            ],
        },
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::CodexAcp,
            description: "通过 codex-acp 接入 ACP 协议；需要本机同时具备 codex 与 codex-acp。",
            notes: &[
                "前置：本机可执行 `codex --version` 与 `codex-acp --help`。",
                "可额外配置 model / variant / sandbox policy。",
                "安装：先装 `codex`，再装 `codex-acp`；Hone 当前最低要求是 `codex-acp >= 0.9.5`。",
                "更新：`npm install -g @zed-industries/codex-acp@latest`。",
                "官方说明：https://github.com/zed-industries/codex-acp",
            ],
        },
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::OpencodeAcp,
            description: "通过 `opencode acp` 接入本机 OpenCode；优先复用你已经在 opencode 里配好的 provider / model。",
            notes: &[
                "前置：本机可执行 `opencode --version`。",
                "默认不在 Hone 首装里填写 provider / base URL / API key。",
                "安装：`curl -fsSL https://opencode.ai/install | bash`。",
                "官方说明：https://opencode.ai/docs/",
                "请先在 `opencode` 里通过 `/connect` 或全局 `opencode.json` / `opencode.jsonc` 配好默认模型。",
                "如果需要 Hone 显式覆盖 opencode 默认模型，再用 `hone-cli models set ...`。",
            ],
        },
    ]
}

fn channel_onboard_specs() -> &'static [ChannelOnboardSpec] {
    &[
        ChannelOnboardSpec {
            kind: ChannelKind::Imessage,
            label: "iMessage",
            status_note: Some("仅 macOS 可用。"),
            permission_notes: &[
                "需要 macOS。",
                "需要给运行 hone-cli 的终端应用授予“完全磁盘访问权限”。",
                "Hone 会轮询 `~/Library/Messages/chat.db`，并通过 AppleScript 发消息。",
            ],
            required_fields: &[],
            supports_chat_scope: false,
        },
        ChannelOnboardSpec {
            kind: ChannelKind::Feishu,
            label: "Feishu",
            status_note: None,
            permission_notes: &[
                "需要飞书开放平台应用的 `app_id` 与 `app_secret`。",
                "平台侧需要完成 Bot / 事件接入与长连接相关配置。",
                "本地只负责写入必填配置，不会替你开通平台权限。",
            ],
            required_fields: &[
                ChannelRequiredField::FeishuAppId,
                ChannelRequiredField::FeishuAppSecret,
            ],
            supports_chat_scope: true,
        },
        ChannelOnboardSpec {
            kind: ChannelKind::Telegram,
            label: "Telegram",
            status_note: Some("当前仍偏实验/placeholder 模式，不建议当成熟生产渠道使用。"),
            permission_notes: &[
                "需要 BotFather 创建的 bot token。",
                "需要把 bot 加入目标私聊或群聊。",
                "如果想处理群聊普通消息，通常还需要检查 BotFather 的 privacy mode 设置。",
            ],
            required_fields: &[ChannelRequiredField::TelegramBotToken],
            supports_chat_scope: true,
        },
        ChannelOnboardSpec {
            kind: ChannelKind::Discord,
            label: "Discord",
            status_note: None,
            permission_notes: &[
                "需要 Discord bot token。",
                "需要把 bot 邀请进目标 server/channel。",
                "至少要给 bot 查看频道、读取历史消息、发送消息等基础权限。",
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
            label: "OpenRouter",
            key_path: "llm.openrouter.api_keys",
            legacy_single_key_path: Some("llm.openrouter.api_key"),
            prompt: "OpenRouter API keys（逗号分隔）",
            notes: &[
                "LLM 主路由。multi-agent / codex_* / nano_banana 都默认走这里。",
                "如果你 runner=opencode_acp 且已在 opencode 里配好 provider,可以在下一步跳过。",
                "支持一次填写多个 key,运行时会自动 fallback。",
            ],
        },
        ProviderOnboardSpec {
            label: "FMP",
            key_path: "fmp.api_keys",
            legacy_single_key_path: Some("fmp.api_key"),
            prompt: "FMP API keys（逗号分隔）",
            notes: &[
                "用于 `data_fetch` 等金融数据能力。",
                "支持一次填写多个 key，运行时会自动 fallback。",
            ],
        },
        ProviderOnboardSpec {
            label: "Tavily",
            key_path: "search.api_keys",
            legacy_single_key_path: None,
            prompt: "Tavily API keys（逗号分隔）",
            notes: &[
                "用于 `web_search` 等联网搜索能力。",
                "支持一次填写多个 key，运行时会自动 fallback。",
            ],
        },
    ]
}

fn print_onboard_block(title: &str, lines: &[&str]) {
    println!();
    println!("{title}");
    for line in lines {
        println!("  - {line}");
    }
}

// ── Discord token-specific 恢复决策。和 prompts 里的 channel recovery 类似,
// 但选项文案针对「token 格式不合法」定制。

fn prompt_discord_token_invalid_recovery_action(
    theme: &ColorfulTheme,
    channel_label: &str,
) -> Result<RequiredFieldEmptyAction, String> {
    let items = vec![
        "重新输入 Discord bot token".to_string(),
        format!("返回并禁用 {channel_label} 渠道"),
    ];
    let idx = prompt_select_index(
        theme,
        &format!("{channel_label} 的 Discord token 格式不合法，下一步？"),
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
        match prompt_channel_recovery_action(theme, channel_label, prompt)? {
            RequiredFieldEmptyAction::Retry => {
                println!("该字段为必填项，不能为空。");
            }
            RequiredFieldEmptyAction::DisableChannel => return Ok(None),
        }
    }
}

fn prompt_onboard_required_secret(
    theme: &ColorfulTheme,
    channel_label: &str,
    prompt: &str,
    current: &str,
) -> Result<Option<String>, String> {
    loop {
        let attempted = prompt_secret(theme, prompt, !current.trim().is_empty())?;
        let resolution = resolve_required_secret_attempt(attempted, current, || {
            prompt_channel_recovery_action(theme, channel_label, prompt)
        })?;
        match resolution {
            RequiredFieldResolution::Value(value) => return Ok(Some(value)),
            RequiredFieldResolution::Retry => {
                println!("该字段为必填项，不能为空。");
            }
            RequiredFieldResolution::DisableChannel => return Ok(None),
        }
    }
}

fn prompt_onboard_required_token(
    theme: &ColorfulTheme,
    channel_label: &str,
    prompt: &str,
    current: &str,
) -> Result<Option<String>, String> {
    loop {
        let attempted =
            prompt_visible_credential(theme, prompt, !current.trim().is_empty(), current)?;
        let resolution = resolve_required_secret_attempt(attempted, current, || {
            prompt_channel_recovery_action(theme, channel_label, prompt)
        })?;
        match resolution {
            RequiredFieldResolution::Value(value) => return Ok(Some(value)),
            RequiredFieldResolution::Retry => {
                println!("该字段为必填项，不能为空。");
            }
            RequiredFieldResolution::DisableChannel => return Ok(None),
        }
    }
}

/// Discord 专用:在通用 token prompt 之上叠加格式校验(三段 base64url、长度合理)。
/// Warn 级别允许用户继续,Invalid 级别会触发 [`prompt_discord_token_invalid_recovery_action`]。
fn prompt_onboard_required_discord_token(
    theme: &ColorfulTheme,
    channel_label: &str,
    prompt: &str,
    current: &str,
) -> Result<Option<String>, String> {
    loop {
        let attempted =
            prompt_visible_credential(theme, prompt, !current.trim().is_empty(), current)?;
        let resolution = match attempted {
            Some(value) => RequiredFieldResolution::Value(value),
            _ if !current.trim().is_empty() => {
                RequiredFieldResolution::Value(normalize_credential_value(current))
            }
            _ => match prompt_channel_recovery_action(theme, channel_label, prompt)? {
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
                        println!("[✓] Token 格式有效（长度={len}）。");
                        return Ok(Some(normalized_value));
                    }
                    DiscordTokenValidation::Warn(message) => {
                        println!("[!] {message}（长度={len}）。");
                        if prompt_bool(theme, "仍然使用这个 Discord token？", false)? {
                            return Ok(Some(normalized_value));
                        }
                    }
                    DiscordTokenValidation::Invalid(message) => {
                        println!("[!] {message}（长度={len}）。");
                        match prompt_discord_token_invalid_recovery_action(theme, channel_label)? {
                            RequiredFieldEmptyAction::Retry => {
                                println!("请重新输入 Discord bot token。");
                            }
                            RequiredFieldEmptyAction::DisableChannel => return Ok(None),
                        }
                    }
                }
            }
            RequiredFieldResolution::Retry => {
                println!("该字段为必填项，不能为空。");
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
    provider_label: &str,
    prompt: &str,
    current_configured: bool,
) -> Result<Option<Vec<String>>, String> {
    loop {
        let attempted = prompt_secret(theme, prompt, current_configured)?;
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

        match prompt_provider_recovery_action(theme, provider_label)? {
            ProviderEmptyAction::Retry => {
                println!("请至少输入一个有效 key，或选择跳过。");
            }
            ProviderEmptyAction::Skip => return Ok(None),
        }
    }
}

pub(crate) fn prompt_onboard_runner(
    theme: &ColorfulTheme,
    config: &hone_core::HoneConfig,
) -> Result<OnboardRunnerKind, String> {
    let specs = runner_onboard_specs();
    let labels = specs
        .iter()
        .map(|spec| {
            // multi-agent 不依赖本机 binary,标记 "no binary" 而非 missing。
            let badge = match spec.kind.binary_probe() {
                None => "no binary needed".to_string(),
                Some((binary, help_arg)) => {
                    if binary_check(binary, help_arg).available {
                        format!("{binary} installed")
                    } else {
                        format!("{binary} missing")
                    }
                }
            };
            format!("{} [{}] - {}", spec.kind.title(), badge, spec.description)
        })
        .collect::<Vec<_>>();
    let default = specs
        .iter()
        .position(|spec| spec.kind.config_value() == config.agent.runner.trim())
        .unwrap_or(0);

    loop {
        let idx = prompt_select_index(theme, "Choose the default runner", &labels, default)?;
        let selected = specs[idx];
        print_onboard_block(selected.kind.title(), selected.notes);

        // 不依赖 binary(如 multi-agent)直接通过。
        let Some((binary, help_arg)) = selected.kind.binary_probe() else {
            return Ok(selected.kind);
        };

        let status = binary_check(binary, help_arg);
        if status.available {
            println!("检测结果：{} 可用。", binary);
            return Ok(selected.kind);
        }
        println!("检测结果：{} 未检测到（{}）。", binary, status.detail);
        // 选 true 会继续用当前 runner(配置会写入,运行时才会因缺 binary 报错);
        // 选 false 会回到 runner 选单重新挑一个(最常见路径)。
        if prompt_bool(
            theme,
            "缺少 binary,仍然保留这个 runner?(no = 返回重新选择 runner)",
            false,
        )? {
            return Ok(selected.kind);
        }
    }
}

pub(crate) fn build_runner_onboard_mutations(
    theme: &ColorfulTheme,
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
                "Multi-Agent setup",
                &[
                    "本 runner 只会写入 `agent.runner = \"multi-agent\"`;",
                    "实际跑起来需要一把 OpenRouter API key,Providers 环节会让你填。",
                    "进阶:`multi-agent.search` / `answer` 两段模型可用 `hone-cli models set ...` 微调。",
                ],
            );
        }
        OnboardRunnerKind::CodexCli => {
            let codex_model = prompt_text(
                theme,
                "Codex CLI model（留空则使用 codex 默认模型）",
                &config.agent.codex_model,
            )?;
            mutations.push(ConfigMutation::Set {
                path: "agent.codex_model".to_string(),
                value: Value::String(codex_model),
            });
        }
        OnboardRunnerKind::CodexAcp => {
            let model = prompt_text(theme, "Codex ACP model", &config.agent.codex_acp.model)?;
            let variant = prompt_text(theme, "Codex ACP variant", &config.agent.codex_acp.variant)?;
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
                "OpenCode ACP setup",
                &[
                    "Hone 首装默认只切换 runner，不在这里强行写 provider / API key / model。",
                    "请先用 `opencode` 自己完成 `/connect`、provider 选择和默认模型配置。",
                    "如果之后需要 Hone 显式覆盖 opencode 默认模型，再运行 `hone-cli models set ...`。",
                ],
            );
            // 不写入任何东西,只是给用户一个"我意识到你可能还没 /connect" 的心理反馈。
            if !prompt_bool(
                theme,
                "你已经在 opencode 里 `/connect` 并选好默认模型了吗?",
                true,
            )? {
                println!(
                    "继续写入 runner 配置;请记得稍后执行 `opencode` 并 `/connect` 配好 provider,否则 Hone 起 chat 会立刻失败。"
                );
            }
        }
    }

    Ok(mutations)
}

pub(crate) fn build_channel_onboard_mutations(
    theme: &ColorfulTheme,
    config: &hone_core::HoneConfig,
    enabled_channels: &mut Vec<ChannelKind>,
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    println!();
    println!("Channel onboarding");
    println!(
        "  - 你可以先全部跳过，之后再用 `hone-cli onboard`、`hone-cli configure` 或 `hone-cli channels ...` 修改。"
    );

    for spec in channel_onboard_specs() {
        // iMessage 在非 macOS 平台不可用(依赖 AppleScript + chat.db),直接 skip
        // 以免让 Linux 用户对一个铁定用不了的 channel 回答一堆问题。
        if spec.kind == ChannelKind::Imessage && !cfg!(target_os = "macos") {
            println!();
            println!("iMessage 渠道仅 macOS 可用,当前平台跳过。");
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
            &format!("Enable {} channel?", spec.label),
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

        if let Some(status_note) = spec.status_note {
            println!();
            println!("{}: {}", spec.label, status_note);
        }
        print_onboard_block(
            &format!("{} prerequisites", spec.label),
            spec.permission_notes,
        );

        // 安全提醒:每个启用的 channel 默认的 allow_* 白名单都是空,
        // 空表等于「所有人都能触发」。onboard 不做详细白名单配置(太长),
        // 但必须让用户知道这件事,不然 bot 会对陌生人裸奔。
        println!();
        println!(
            "  ⚠ 注意:{} 渠道默认 allow 白名单为空,即所有联系人都能触发 Hone。",
            spec.label
        );
        println!(
            "     如需限定,onboard 完成后用 `hone-cli configure --section channels` 或直接编辑 config.yaml。"
        );

        // 必填字段循环:任一字段让用户选择「放弃整个渠道」都会把 channel_mutations
        // reset 成单条「enabled=false」并 break。
        for field in spec.required_fields {
            match field {
                ChannelRequiredField::FeishuAppId => {
                    let Some(value) = prompt_onboard_required_text(
                        theme,
                        spec.label,
                        "Feishu app id",
                        &config.feishu.app_id,
                    )?
                    else {
                        println!("已返回并禁用 {} 渠道。", spec.label);
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
                        spec.label,
                        "Feishu app secret",
                        &config.feishu.app_secret,
                    )?
                    else {
                        println!("已返回并禁用 {} 渠道。", spec.label);
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
                        spec.label,
                        "Telegram bot token",
                        &config.telegram.bot_token,
                    )?
                    else {
                        println!("已返回并禁用 {} 渠道。", spec.label);
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
                        spec.label,
                        "Discord bot token",
                        &config.discord.bot_token,
                    )?
                    else {
                        println!("已返回并禁用 {} 渠道。", spec.label);
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
            let scope =
                prompt_chat_scope(theme, &format!("{} chat scope", spec.label), current_scope)?;
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
                "iMessage target handle（可选；留空表示监听所有会话）",
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
    config: &hone_core::HoneConfig,
    enabled_channels: &[ChannelKind],
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    if enabled_channels.is_empty() {
        return Ok(mutations);
    }

    println!();
    println!("Admin onboarding");
    println!("  - 管理员白名单决定谁能触发 `/register-admin` / `/report` / 重启 Hone 等管理指令。");
    println!("  - 不配就没人是 admin,本机所有人都触发不到管理能力。");

    if !prompt_bool(theme, "把自己加为已启用渠道的 admin 白名单?", true)? {
        println!(
            "已跳过 admin 配置;之后可用 `hone-cli configure` 或直接编辑 config.yaml 的 `admins.*`。"
        );
        return Ok(mutations);
    }

    for kind in enabled_channels {
        match kind {
            ChannelKind::Imessage => {
                let value = prompt_text(
                    theme,
                    "iMessage admin handle(手机号带国家码,如 +8613800138000 或 Apple ID 邮箱;留空跳过)",
                    "",
                )?;
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
                let value = prompt_text(
                    theme,
                    "Telegram admin user id(数字 ID,如 8039067465;可通过 @userinfobot 获取;留空跳过)",
                    "",
                )?;
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
                let value = prompt_text(
                    theme,
                    "Discord admin user id(数字 ID,18 位数,可在 Discord 开发者模式下右键用户头像复制;留空跳过)",
                    "",
                )?;
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
                    "邮箱(admin@example.com)".to_string(),
                    "手机号(+8613800138000)".to_string(),
                    "open_id(ou_xxx)".to_string(),
                    "跳过".to_string(),
                ];
                let idx = prompt_select_index(theme, "Feishu admin 用哪种 id 添加?", &choices, 0)?;
                match idx {
                    0 => {
                        let value = prompt_text(theme, "Feishu admin 邮箱", "")?;
                        if !value.trim().is_empty() {
                            mutations.push(append_admin_mutation(
                                "admins.feishu_emails",
                                &config.admins.feishu_emails,
                                value.trim().to_string(),
                            ));
                        }
                    }
                    1 => {
                        let value = prompt_text(
                            theme,
                            "Feishu admin 手机号(推荐带国家码,如 +8613800138000)",
                            "",
                        )?;
                        if !value.trim().is_empty() {
                            mutations.push(append_admin_mutation(
                                "admins.feishu_mobiles",
                                &config.admins.feishu_mobiles,
                                value.trim().to_string(),
                            ));
                        }
                    }
                    2 => {
                        let value = prompt_text(theme, "Feishu admin open_id", "")?;
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
    config: &hone_core::HoneConfig,
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    println!();
    println!("Provider onboarding");
    println!("  - OpenRouter / FMP / Tavily 都会要求你明确选择：现在填写，或本轮跳过。");
    println!(
        "  - 跳过不会阻塞 onboarding，之后仍可用 `hone-cli configure --section providers` 补配。"
    );

    for spec in provider_onboard_specs() {
        let current_configured = has_configured_provider_keys(spec, config);
        print_onboard_block(&format!("{} API keys", spec.label), spec.notes);

        if !prompt_bool(
            theme,
            &format!("Configure {} API keys now?", spec.label),
            current_configured,
        )? {
            println!("已跳过 {} API key 配置。", spec.label);
            continue;
        }

        if let Some(keys) =
            prompt_onboard_provider_keys(theme, spec.label, spec.prompt, current_configured)?
        {
            mutations.extend(build_provider_api_key_mutations(
                spec.key_path,
                spec.legacy_single_key_path,
                keys,
            ));
        } else if current_configured {
            println!("保留现有 {} API key 配置。", spec.label);
        } else {
            println!("已跳过 {} API key 配置。", spec.label);
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

pub(crate) async fn run_onboard(
    config_path: Option<&Path>,
    _args: OnboardArgs,
) -> Result<(), String> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err("`hone-cli onboard` 需要交互式终端（TTY）".to_string());
    }

    let (config, paths) = load_cli_config(config_path, true).map_err(|e| e.to_string())?;
    let theme = ColorfulTheme::default();

    println!("Hone onboarding");
    println!("  - 约 3–5 分钟,全程键盘操作即可。");
    println!(
        "  - 任意时刻 Ctrl+C 可安全退出,已填的东西只在走到最后一步「apply」时才会写入 config.yaml。"
    );
    println!("  - 每个环节都可以跳过,之后再通过 `hone-cli onboard` 或其他 CLI 子命令补配。");

    let runner = prompt_onboard_runner(&theme, &config)?;
    let mut mutations = build_runner_onboard_mutations(&theme, &config, runner)?;

    // channel 里真正被 enable 的记一份,供 admin 环节按渠道收集对应 id。
    let mut enabled_channels: Vec<ChannelKind> = Vec::new();
    mutations.extend(build_channel_onboard_mutations(
        &theme,
        &config,
        &mut enabled_channels,
    )?);
    mutations.extend(build_admin_onboard_mutations(
        &theme,
        &config,
        &enabled_channels,
    )?);
    mutations.extend(build_provider_onboard_mutations(&theme, &config)?);

    let result = apply_mutations_and_generate(&paths, &mutations)?;
    println!();
    println!("{}", apply_message(&result.apply));
    println!(
        "  - 共写入 {} 条配置字段。",
        result.apply.changed_paths.len()
    );
    println!(
        "config={} effective={}",
        paths.canonical_config_path.to_string_lossy(),
        paths.effective_config_path.to_string_lossy()
    );

    if prompt_bool(&theme, "Run `hone-cli doctor` now?", true)? {
        println!();
        print_doctor_report_text(build_doctor_report(config_path).await);
    }

    if prompt_bool(&theme, "Start Hone now?", false)? {
        println!();
        return start::run_start(config_path).await;
    }

    println!();
    println!("后续命令：");
    println!("  - `hone-cli status`");
    println!("  - `hone-cli doctor`");
    println!("  - `hone-cli start`");
    Ok(())
}
