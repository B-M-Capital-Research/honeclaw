mod common;
mod repl;
mod start;

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use clap::{Args, Parser, Subcommand, ValueEnum};
use common::{load_cli_config, load_cli_core, resolve_runtime_paths};
use dialoguer::{Confirm, Input, Password, Select, theme::ColorfulTheme};
use hone_core::config::{
    ConfigApplyPlan, ConfigMutation, apply_config_mutations, generate_effective_config,
    is_sensitive_config_path, read_config_path_value, redact_sensitive_value,
};
use serde::Serialize;
use serde_yaml::Value;

#[derive(Parser, Debug)]
#[command(name = "hone-cli")]
#[command(about = "Hone CLI")]
struct Cli {
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Chat,
    #[command(visible_alias = "setup")]
    Onboard(OnboardArgs),
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Configure(ConfigureArgs),
    Models {
        #[command(subcommand)]
        command: ModelsCommands,
    },
    Channels {
        #[command(subcommand)]
        command: ChannelsCommands,
    },
    Status(StatusArgs),
    Doctor(DoctorArgs),
    Start,
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    File,
    Get(ConfigPathArgs),
    Set(ConfigSetArgs),
    Unset(ConfigPathArgs),
    Validate(ReadableArgs),
}

#[derive(Subcommand, Debug)]
enum ModelsCommands {
    Status(ReadableArgs),
    Set(ModelsSetArgs),
}

#[derive(Subcommand, Debug)]
enum ChannelsCommands {
    List(ReadableArgs),
    Set(ChannelSetArgs),
    Enable(ChannelToggleArgs),
    Disable(ChannelToggleArgs),
}

#[derive(Args, Debug)]
struct ReadableArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct StatusArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct DoctorArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct ConfigPathArgs {
    path: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
struct ConfigSetArgs {
    path: String,
    value: String,
}

#[derive(Args, Debug)]
struct ConfigureArgs {
    #[arg(long = "section", value_enum)]
    sections: Vec<ConfigureSection>,
}

#[derive(Args, Debug, Default)]
struct OnboardArgs {}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum ConfigureSection {
    Agent,
    Channels,
    Providers,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum ChannelKind {
    Imessage,
    Feishu,
    Telegram,
    Discord,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum CliChatScope {
    DmOnly,
    GroupchatOnly,
    All,
}

impl CliChatScope {
    fn as_config_value(&self) -> &'static str {
        match self {
            Self::DmOnly => "DM_ONLY",
            Self::GroupchatOnly => "GROUPCHAT_ONLY",
            Self::All => "ALL",
        }
    }

    fn label(&self) -> &'static str {
        self.as_config_value()
    }

    fn from_chat_scope(scope: hone_core::config::ChatScope) -> Self {
        match scope {
            hone_core::config::ChatScope::DmOnly => Self::DmOnly,
            hone_core::config::ChatScope::GroupchatOnly => Self::GroupchatOnly,
            hone_core::config::ChatScope::All => Self::All,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OnboardRunnerKind {
    CodexCli,
    CodexAcp,
    OpencodeAcp,
}

impl OnboardRunnerKind {
    fn config_value(&self) -> &'static str {
        match self {
            Self::CodexCli => "codex_cli",
            Self::CodexAcp => "codex_acp",
            Self::OpencodeAcp => "opencode_acp",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            Self::CodexCli => "Codex CLI",
            Self::CodexAcp => "Codex ACP",
            Self::OpencodeAcp => "OpenCode ACP",
        }
    }

    fn binary_probe(&self) -> (&'static str, &'static str) {
        match self {
            Self::CodexCli => ("codex", "--version"),
            Self::CodexAcp => ("codex-acp", "--help"),
            Self::OpencodeAcp => ("opencode", "--version"),
        }
    }
}

#[derive(Clone, Copy)]
struct RunnerOnboardSpec {
    kind: OnboardRunnerKind,
    description: &'static str,
    notes: &'static [&'static str],
}

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
    status_note: Option<&'static str>,
    permission_notes: &'static [&'static str],
    required_fields: &'static [ChannelRequiredField],
    supports_chat_scope: bool,
}

#[derive(Args, Debug)]
struct ModelsSetArgs {
    #[arg(long)]
    runner: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    variant: Option<String>,
    #[arg(long)]
    base_url: Option<String>,
    #[arg(long)]
    api_key: Option<String>,
    #[arg(long)]
    codex_model: Option<String>,
    #[arg(long)]
    codex_acp_model: Option<String>,
    #[arg(long)]
    codex_acp_variant: Option<String>,
    #[arg(long)]
    aux_base_url: Option<String>,
    #[arg(long)]
    aux_api_key: Option<String>,
    #[arg(long)]
    aux_model: Option<String>,
    #[arg(long)]
    search_base_url: Option<String>,
    #[arg(long)]
    search_api_key: Option<String>,
    #[arg(long)]
    search_model: Option<String>,
    #[arg(long)]
    search_max_iterations: Option<u32>,
    #[arg(long)]
    answer_base_url: Option<String>,
    #[arg(long)]
    answer_api_key: Option<String>,
    #[arg(long)]
    answer_model: Option<String>,
    #[arg(long)]
    answer_variant: Option<String>,
    #[arg(long)]
    answer_max_tool_calls: Option<u32>,
}

#[derive(Args, Debug)]
struct ChannelSetArgs {
    channel: ChannelKind,
    #[arg(long)]
    enabled: Option<bool>,
    #[arg(long)]
    target_handle: Option<String>,
    #[arg(long)]
    db_path: Option<String>,
    #[arg(long)]
    poll_interval: Option<u64>,
    #[arg(long)]
    app_id: Option<String>,
    #[arg(long)]
    app_secret: Option<String>,
    #[arg(long)]
    bot_token: Option<String>,
    #[arg(long, value_enum)]
    chat_scope: Option<CliChatScope>,
}

#[derive(Args, Debug)]
struct ChannelToggleArgs {
    channel: ChannelKind,
}

#[derive(Debug, Serialize)]
struct ModelStatusReport {
    runner: String,
    codex_model: String,
    codex_acp_model: String,
    codex_acp_variant: String,
    opencode_base_url: String,
    opencode_model: String,
    opencode_variant: String,
    opencode_api_key_configured: bool,
    opencode_inherits_local_config: bool,
    auxiliary_base_url: String,
    auxiliary_model: String,
    auxiliary_api_key_configured: bool,
    search_base_url: String,
    search_model: String,
    search_api_key_configured: bool,
    search_max_iterations: u32,
    answer_base_url: String,
    answer_model: String,
    answer_variant: String,
    answer_api_key_configured: bool,
    answer_max_tool_calls: u32,
}

#[derive(Debug, Serialize)]
struct ChannelStatusReport {
    channel: String,
    enabled: bool,
    auth_configured: bool,
    chat_scope: Option<String>,
    details: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BinaryStatus {
    name: String,
    available: bool,
    detail: String,
}

#[derive(Debug, Serialize)]
struct StatusReport {
    canonical_config_path: String,
    effective_config_path: String,
    data_dir: String,
    skills_dir: String,
    legacy_warning: Option<String>,
    models: ModelStatusReport,
    channels: Vec<ChannelStatusReport>,
    api_keys: ApiKeySummary,
    binaries: Vec<BinaryStatus>,
}

#[derive(Debug, Serialize)]
struct ApiKeySummary {
    openrouter: bool,
    primary_route: bool,
    auxiliary: bool,
    multi_agent_search: bool,
    multi_agent_answer: bool,
    fmp: bool,
    tavily: bool,
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    name: String,
    status: &'static str,
    detail: String,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    canonical_config_path: String,
    effective_config_path: String,
    checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
struct MutationResult {
    config_path: String,
    effective_config_path: String,
    config_revision: String,
    applied_live: bool,
    restarted_components: Vec<String>,
    restart_required: bool,
    path: String,
    value: Value,
}

fn apply_message(plan: &ConfigApplyPlan) -> String {
    if plan.restart_required {
        return "配置已保存，需重启运行时".to_string();
    }
    if !plan.restarted_components.is_empty() {
        return format!(
            "配置已保存，并需重启组件：{}",
            plan.restarted_components.join(", ")
        );
    }
    "配置已保存，已立即生效".to_string()
}

fn runner_onboard_specs() -> &'static [RunnerOnboardSpec] {
    &[
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::CodexCli,
            description: "优先复用本机 codex CLI 登录态；适合已经能直接运行 codex 的用户。",
            notes: &[
                "前置：本机可执行 `codex --version`。",
                "优点：不需要单独填写 OpenAI-compatible base URL / API key。",
            ],
        },
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::CodexAcp,
            description: "通过 codex-acp 接入 ACP 协议；需要本机同时具备 codex 与 codex-acp。",
            notes: &[
                "前置：本机可执行 `codex --version` 与 `codex-acp --help`。",
                "可额外配置 model / variant / sandbox policy。",
            ],
        },
        RunnerOnboardSpec {
            kind: OnboardRunnerKind::OpencodeAcp,
            description: "通过 `opencode acp` 接入本机 OpenCode；优先复用你已经在 opencode 里配好的 provider / model。",
            notes: &[
                "前置：本机可执行 `opencode --version`。",
                "默认不在 Hone 首装里填写 provider / base URL / API key。",
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

fn print_onboard_block(title: &str, lines: &[&str]) {
    println!();
    println!("{title}");
    for line in lines {
        println!("  - {line}");
    }
}

fn prompt_select_index(
    theme: &ColorfulTheme,
    prompt: &str,
    items: &[String],
    default: usize,
) -> Result<usize, String> {
    Select::with_theme(theme)
        .with_prompt(prompt)
        .items(items)
        .default(default.min(items.len().saturating_sub(1)))
        .interact()
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequiredFieldEmptyAction {
    Retry,
    DisableChannel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RequiredFieldResolution {
    Value(String),
    Retry,
    DisableChannel,
}

fn resolve_required_field_attempt(
    attempted: Option<String>,
    current: &str,
    on_empty: RequiredFieldEmptyAction,
) -> RequiredFieldResolution {
    if let Some(value) = attempted {
        if !value.trim().is_empty() {
            return RequiredFieldResolution::Value(value);
        }
    }
    if !current.trim().is_empty() {
        return RequiredFieldResolution::Value(current.to_string());
    }
    match on_empty {
        RequiredFieldEmptyAction::Retry => RequiredFieldResolution::Retry,
        RequiredFieldEmptyAction::DisableChannel => RequiredFieldResolution::DisableChannel,
    }
}

fn prompt_channel_recovery_action(
    theme: &ColorfulTheme,
    channel_label: &str,
    field_label: &str,
) -> Result<RequiredFieldEmptyAction, String> {
    let items = vec![
        "重试当前字段".to_string(),
        format!("返回并禁用 {channel_label} 渠道"),
    ];
    let idx = prompt_select_index(
        theme,
        &format!("{channel_label} 的必填项“{field_label}”为空，下一步？"),
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
        match resolve_required_field_attempt(
            attempted,
            current,
            prompt_channel_recovery_action(theme, channel_label, prompt)?,
        ) {
            RequiredFieldResolution::Value(value) => return Ok(Some(value)),
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

fn apply_mutations_and_generate(
    paths: &common::ResolvedRuntimePaths,
    mutations: &[ConfigMutation],
) -> Result<hone_core::config::ConfigMutationResult, String> {
    let mut result = apply_config_mutations(&paths.canonical_config_path, mutations)
        .map_err(|e| e.to_string())?;
    result.config_revision =
        generate_effective_config(&paths.canonical_config_path, &paths.effective_config_path)
            .map_err(|e| e.to_string())?;
    Ok(result)
}

fn yaml_value_from_cli(raw: &str) -> Result<Value, String> {
    serde_yaml::from_str(raw).map_err(|e| format!("无法解析配置值: {e}"))
}

fn value_to_pretty_text(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        _ => serde_yaml::to_string(value)
            .unwrap_or_else(|_| "<unable to render>".to_string())
            .trim()
            .to_string(),
    }
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let rendered = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    println!("{rendered}");
    Ok(())
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn binary_check(name: &str, help_arg: &str) -> BinaryStatus {
    let output = StdCommand::new(name).arg(help_arg).output();
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            let detail = if !stdout.is_empty() {
                stdout
            } else if !stderr.is_empty() {
                stderr
            } else {
                "命令可执行".to_string()
            };
            BinaryStatus {
                name: name.to_string(),
                available: true,
                detail,
            }
        }
        Err(error) => BinaryStatus {
            name: name.to_string(),
            available: false,
            detail: error.to_string(),
        },
    }
}

fn runtime_binary_status(binary: &str) -> BinaryStatus {
    match start::locate_binary(binary) {
        Some(path) => BinaryStatus {
            name: binary.to_string(),
            available: true,
            detail: path.to_string_lossy().to_string(),
        },
        None => BinaryStatus {
            name: binary.to_string(),
            available: false,
            detail: "未在 hone-cli 附近找到二进制".to_string(),
        },
    }
}

fn chat_scope_label(scope: hone_core::config::ChatScope) -> String {
    match scope {
        hone_core::config::ChatScope::DmOnly => "DM_ONLY".to_string(),
        hone_core::config::ChatScope::GroupchatOnly => "GROUPCHAT_ONLY".to_string(),
        hone_core::config::ChatScope::All => "ALL".to_string(),
    }
}

fn build_model_status(config: &hone_core::HoneConfig) -> ModelStatusReport {
    let opencode_inherits_local_config = config.agent.runner == "opencode_acp"
        && !non_empty(&config.agent.opencode.model)
        && !non_empty(&config.agent.opencode.variant)
        && !non_empty(&config.agent.opencode.api_base_url)
        && !non_empty(&config.agent.opencode.api_key);
    ModelStatusReport {
        runner: config.agent.runner.clone(),
        codex_model: config.agent.codex_model.clone(),
        codex_acp_model: config.agent.codex_acp.model.clone(),
        codex_acp_variant: config.agent.codex_acp.variant.clone(),
        opencode_base_url: config.agent.opencode.api_base_url.clone(),
        opencode_model: config.agent.opencode.model.clone(),
        opencode_variant: config.agent.opencode.variant.clone(),
        opencode_api_key_configured: non_empty(&config.agent.opencode.api_key),
        opencode_inherits_local_config,
        auxiliary_base_url: config.llm.auxiliary.base_url.clone(),
        auxiliary_model: config.llm.auxiliary.model.clone(),
        auxiliary_api_key_configured: !config.llm.auxiliary.resolved_api_key().is_empty(),
        search_base_url: config.agent.multi_agent.search.base_url.clone(),
        search_model: config.agent.multi_agent.search.model.clone(),
        search_api_key_configured: non_empty(&config.agent.multi_agent.search.api_key),
        search_max_iterations: config.agent.multi_agent.search.max_iterations,
        answer_base_url: config.agent.multi_agent.answer.api_base_url.clone(),
        answer_model: config.agent.multi_agent.answer.model.clone(),
        answer_variant: config.agent.multi_agent.answer.variant.clone(),
        answer_api_key_configured: non_empty(&config.agent.multi_agent.answer.api_key),
        answer_max_tool_calls: config.agent.multi_agent.answer.max_tool_calls,
    }
}

fn build_channel_reports(config: &hone_core::HoneConfig) -> Vec<ChannelStatusReport> {
    vec![
        ChannelStatusReport {
            channel: "imessage".to_string(),
            enabled: config.imessage.enabled,
            auth_configured: true,
            chat_scope: None,
            details: vec![
                format!("db_path={}", config.imessage.db_path),
                format!("poll_interval={}", config.imessage.poll_interval),
            ],
        },
        ChannelStatusReport {
            channel: "feishu".to_string(),
            enabled: config.feishu.enabled,
            auth_configured: non_empty(&config.feishu.app_id)
                && non_empty(&config.feishu.app_secret),
            chat_scope: Some(chat_scope_label(config.feishu.chat_scope)),
            details: vec![format!(
                "app_id={}",
                if non_empty(&config.feishu.app_id) {
                    "<set>"
                } else {
                    "<empty>"
                }
            )],
        },
        ChannelStatusReport {
            channel: "telegram".to_string(),
            enabled: config.telegram.enabled,
            auth_configured: non_empty(&config.telegram.bot_token),
            chat_scope: Some(chat_scope_label(config.telegram.chat_scope)),
            details: vec![format!(
                "bot_token={}",
                if non_empty(&config.telegram.bot_token) {
                    "<set>"
                } else {
                    "<empty>"
                }
            )],
        },
        ChannelStatusReport {
            channel: "discord".to_string(),
            enabled: config.discord.enabled,
            auth_configured: non_empty(&config.discord.bot_token),
            chat_scope: Some(chat_scope_label(config.discord.chat_scope)),
            details: vec![format!(
                "bot_token={}",
                if non_empty(&config.discord.bot_token) {
                    "<set>"
                } else {
                    "<empty>"
                }
            )],
        },
    ]
}

fn build_api_key_summary(config: &hone_core::HoneConfig) -> ApiKeySummary {
    ApiKeySummary {
        openrouter: !config.llm.openrouter.effective_key_pool().is_empty(),
        primary_route: non_empty(&config.agent.opencode.api_key),
        auxiliary: !config.llm.auxiliary.resolved_api_key().is_empty(),
        multi_agent_search: non_empty(&config.agent.multi_agent.search.api_key),
        multi_agent_answer: non_empty(&config.agent.multi_agent.answer.api_key),
        fmp: !config.fmp.effective_key_pool().is_empty(),
        tavily: !config
            .search
            .api_keys
            .iter()
            .all(|key| key.trim().is_empty()),
    }
}

fn runner_binary_name(runner: &str) -> Option<(&'static str, &'static str)> {
    match runner {
        "gemini_cli" | "gemini_acp" => Some(("gemini", "--version")),
        "codex_cli" => Some(("codex", "--version")),
        "codex_acp" => Some(("codex-acp", "--help")),
        "opencode_acp" | "multi-agent" => Some(("opencode", "--version")),
        _ => None,
    }
}

async fn build_status_report(config_path: Option<&Path>) -> Result<StatusReport, String> {
    let (config, paths) = load_cli_config(config_path, false).map_err(|e| e.to_string())?;
    let mut binaries = Vec::new();
    if let Some((binary, help_arg)) = runner_binary_name(config.agent.runner.trim()) {
        binaries.push(binary_check(binary, help_arg));
    }
    binaries.push(runtime_binary_status("hone-console-page"));
    binaries.push(runtime_binary_status("hone-mcp"));

    Ok(StatusReport {
        canonical_config_path: paths.canonical_config_path.to_string_lossy().to_string(),
        effective_config_path: paths.effective_config_path.to_string_lossy().to_string(),
        data_dir: paths.data_dir.to_string_lossy().to_string(),
        skills_dir: paths.skills_dir.to_string_lossy().to_string(),
        legacy_warning: paths.legacy_warning.clone(),
        models: build_model_status(&config),
        channels: build_channel_reports(&config),
        api_keys: build_api_key_summary(&config),
        binaries,
    })
}

async fn build_doctor_report(config_path: Option<&Path>) -> DoctorReport {
    let resolved = resolve_runtime_paths(config_path, false);
    let mut checks = Vec::new();

    match resolved {
        Ok(paths) => {
            checks.push(DoctorCheck {
                name: "canonical-config".to_string(),
                status: if paths.canonical_config_path.exists() {
                    "ok"
                } else {
                    "fail"
                },
                detail: paths.canonical_config_path.to_string_lossy().to_string(),
            });
            checks.push(DoctorCheck {
                name: "effective-config".to_string(),
                status: if paths.effective_config_path.exists() {
                    "ok"
                } else {
                    "warn"
                },
                detail: paths.effective_config_path.to_string_lossy().to_string(),
            });
            if let Some(warning) = &paths.legacy_warning {
                checks.push(DoctorCheck {
                    name: "legacy-runtime-config".to_string(),
                    status: "warn",
                    detail: warning.clone(),
                });
            }

            match load_cli_config(config_path, false) {
                Ok((config, loaded_paths)) => {
                    checks.push(DoctorCheck {
                        name: "config-parse".to_string(),
                        status: "ok",
                        detail: "配置解析成功".to_string(),
                    });
                    if let Some(parent) = loaded_paths.canonical_config_path.parent() {
                        let readonly = std::fs::metadata(parent)
                            .map(|m| m.permissions().readonly())
                            .unwrap_or(false);
                        checks.push(DoctorCheck {
                            name: "canonical-parent".to_string(),
                            status: if parent.exists() && !readonly {
                                "ok"
                            } else if parent.exists() {
                                "warn"
                            } else {
                                "fail"
                            },
                            detail: if readonly {
                                format!(
                                    "{} (只读权限，可能无法写 canonical config)",
                                    parent.to_string_lossy()
                                )
                            } else {
                                parent.to_string_lossy().to_string()
                            },
                        });
                    }
                    checks.push(DoctorCheck {
                        name: "runtime-dir".to_string(),
                        status: if loaded_paths.runtime_dir.exists() {
                            "ok"
                        } else {
                            "warn"
                        },
                        detail: loaded_paths.runtime_dir.to_string_lossy().to_string(),
                    });

                    checks.push(DoctorCheck {
                        name: "data-dir".to_string(),
                        status: if loaded_paths.data_dir.exists() {
                            "ok"
                        } else {
                            "warn"
                        },
                        detail: loaded_paths.data_dir.to_string_lossy().to_string(),
                    });
                    checks.push(DoctorCheck {
                        name: "skills-dir".to_string(),
                        status: if loaded_paths.skills_dir.exists() {
                            "ok"
                        } else {
                            "warn"
                        },
                        detail: loaded_paths.skills_dir.to_string_lossy().to_string(),
                    });

                    if let Some((binary, help_arg)) = runner_binary_name(config.agent.runner.trim())
                    {
                        let status = binary_check(binary, help_arg);
                        checks.push(DoctorCheck {
                            name: format!("runner-binary:{binary}"),
                            status: if status.available { "ok" } else { "fail" },
                            detail: status.detail,
                        });
                    }

                    let starter_bins = [
                        "hone-console-page",
                        "hone-mcp",
                        "hone-imessage",
                        "hone-discord",
                        "hone-feishu",
                        "hone-telegram",
                    ];
                    for binary in starter_bins {
                        let status = runtime_binary_status(binary);
                        checks.push(DoctorCheck {
                            name: format!("runtime-binary:{binary}"),
                            status: if status.available { "ok" } else { "warn" },
                            detail: status.detail,
                        });
                    }

                    for channel in build_channel_reports(&config)
                        .into_iter()
                        .filter(|channel| channel.enabled)
                    {
                        checks.push(DoctorCheck {
                            name: format!("channel-auth:{}", channel.channel),
                            status: if channel.auth_configured {
                                "ok"
                            } else {
                                "fail"
                            },
                            detail: if channel.auth_configured {
                                "已配置".to_string()
                            } else {
                                "已启用，但缺少认证字段".to_string()
                            },
                        });
                    }
                }
                Err(error) => {
                    checks.push(DoctorCheck {
                        name: "config-parse".to_string(),
                        status: "fail",
                        detail: error.to_string(),
                    });
                }
            }

            DoctorReport {
                canonical_config_path: paths.canonical_config_path.to_string_lossy().to_string(),
                effective_config_path: paths.effective_config_path.to_string_lossy().to_string(),
                checks,
            }
        }
        Err(error) => DoctorReport {
            canonical_config_path: "<unresolved>".to_string(),
            effective_config_path: "<unresolved>".to_string(),
            checks: vec![DoctorCheck {
                name: "config-path".to_string(),
                status: "fail",
                detail: error.to_string(),
            }],
        },
    }
}

fn build_model_mutations(args: &ModelsSetArgs) -> Result<Vec<ConfigMutation>, String> {
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

    if let Some(value) = &args.base_url {
        push("agent.opencode.api_base_url", Value::String(value.clone()));
        push(
            "agent.multi_agent.answer.api_base_url",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = &args.api_key {
        push("agent.opencode.api_key", Value::String(value.clone()));
        push(
            "agent.multi_agent.answer.api_key",
            Value::String(value.clone()),
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

    if let Some(value) = &args.aux_base_url {
        push("llm.auxiliary.base_url", Value::String(value.clone()));
    }
    if let Some(value) = &args.aux_api_key {
        push("llm.auxiliary.api_key", Value::String(value.clone()));
    }
    if let Some(value) = &args.aux_model {
        push("llm.auxiliary.model", Value::String(value.clone()));
        push("llm.openrouter.sub_model", Value::String(value.clone()));
    }

    if let Some(value) = &args.search_base_url {
        push(
            "agent.multi_agent.search.base_url",
            Value::String(value.clone()),
        );
    }
    if let Some(value) = &args.search_api_key {
        push(
            "agent.multi_agent.search.api_key",
            Value::String(value.clone()),
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
            Value::String(value.clone()),
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

fn build_channel_mutations(args: &ChannelSetArgs) -> Result<Vec<ConfigMutation>, String> {
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
                push("feishu.app_secret", Value::String(value.clone()));
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
                push("telegram.bot_token", Value::String(value.clone()));
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
                push("discord.bot_token", Value::String(value.clone()));
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

fn provider_key_mutation(path: &str, keys: Vec<String>) -> ConfigMutation {
    ConfigMutation::Set {
        path: path.to_string(),
        value: Value::Sequence(keys.into_iter().map(Value::String).collect()),
    }
}

fn parse_csv_values(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn prompt_text(theme: &ColorfulTheme, prompt: &str, current: &str) -> Result<String, String> {
    let mut input = Input::<String>::with_theme(theme);
    input = input.with_prompt(prompt.to_string());
    if !current.is_empty() {
        input = input.with_initial_text(current.to_string());
    }
    input.interact_text().map_err(|e| e.to_string())
}

fn prompt_bool(theme: &ColorfulTheme, prompt: &str, current: bool) -> Result<bool, String> {
    Confirm::with_theme(theme)
        .with_prompt(prompt)
        .default(current)
        .interact()
        .map_err(|e| e.to_string())
}

fn prompt_secret(
    theme: &ColorfulTheme,
    prompt: &str,
    keep_note: bool,
) -> Result<Option<String>, String> {
    let prompt = if keep_note {
        format!("{prompt}（留空保持现有值）")
    } else {
        prompt.to_string()
    };
    let value = Password::with_theme(theme)
        .with_prompt(prompt)
        .allow_empty_password(true)
        .interact()
        .map_err(|e| e.to_string())?;
    if value.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

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

fn prompt_onboard_runner(
    theme: &ColorfulTheme,
    config: &hone_core::HoneConfig,
) -> Result<OnboardRunnerKind, String> {
    let specs = runner_onboard_specs();
    let labels = specs
        .iter()
        .map(|spec| {
            let (binary, help_arg) = spec.kind.binary_probe();
            let status = binary_check(binary, help_arg);
            format!(
                "{} [{}] - {}",
                spec.kind.title(),
                if status.available {
                    "installed"
                } else {
                    "missing"
                },
                spec.description
            )
        })
        .collect::<Vec<_>>();
    let default = specs
        .iter()
        .position(|spec| spec.kind.config_value() == config.agent.runner.trim())
        .unwrap_or(0);

    loop {
        let idx = prompt_select_index(theme, "Choose the default runner", &labels, default)?;
        let selected = specs[idx];
        let (binary, help_arg) = selected.kind.binary_probe();
        let status = binary_check(binary, help_arg);
        print_onboard_block(selected.kind.title(), selected.notes);
        if status.available {
            println!("检测结果：{} 可用。", binary);
            return Ok(selected.kind);
        }
        println!("检测结果：{} 未检测到（{}）。", binary, status.detail);
        if prompt_bool(
            theme,
            "Binary not detected. Continue configuring this runner anyway?",
            false,
        )? {
            return Ok(selected.kind);
        }
    }
}

fn build_runner_onboard_mutations(
    theme: &ColorfulTheme,
    config: &hone_core::HoneConfig,
    runner: OnboardRunnerKind,
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = vec![ConfigMutation::Set {
        path: "agent.runner".to_string(),
        value: Value::String(runner.config_value().to_string()),
    }];

    match runner {
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
            let _ = theme;
            let _ = config;
            print_onboard_block(
                "OpenCode ACP setup",
                &[
                    "Hone 首装默认只切换 runner，不在这里强行写 provider / API key / model。",
                    "请先用 `opencode` 自己完成 `/connect`、provider 选择和默认模型配置。",
                    "如果之后需要 Hone 显式覆盖 opencode 默认模型，再运行 `hone-cli models set ...`。",
                ],
            );
        }
    }

    Ok(mutations)
}

fn build_channel_onboard_mutations(
    theme: &ColorfulTheme,
    config: &hone_core::HoneConfig,
) -> Result<Vec<ConfigMutation>, String> {
    let mut mutations = Vec::new();
    println!();
    println!("Channel onboarding");
    println!(
        "  - 你可以先全部跳过，之后再用 `hone-cli onboard`、`hone-cli configure` 或 `hone-cli channels ...` 修改。"
    );

    for spec in channel_onboard_specs() {
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
                    let Some(value) = prompt_onboard_required_secret(
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
                    let Some(value) = prompt_onboard_required_secret(
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

fn run_configure(config_path: Option<&Path>, args: ConfigureArgs) -> Result<(), String> {
    let (config, paths) = load_cli_config(config_path, true).map_err(|e| e.to_string())?;
    let theme = ColorfulTheme::default();
    let mut mutations = Vec::new();

    for section in sections_or_default(&args.sections) {
        match section {
            ConfigureSection::Agent => {
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
                    "Primary API key",
                    is_sensitive_config_path("agent.opencode.api_key"),
                )? {
                    mutations.push(ConfigMutation::Set {
                        path: "agent.opencode.api_key".to_string(),
                        value: Value::String(api_key),
                    });
                }

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
                mutations.push(ConfigMutation::Set {
                    path: "llm.openrouter.sub_model".to_string(),
                    value: Value::String(aux_model),
                });
                if let Some(api_key) = prompt_secret(&theme, "Auxiliary API key", true)? {
                    mutations.push(ConfigMutation::Set {
                        path: "llm.auxiliary.api_key".to_string(),
                        value: Value::String(api_key),
                    });
                }

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
                if let Some(api_key) = prompt_secret(&theme, "Multi-agent search API key", true)? {
                    mutations.push(ConfigMutation::Set {
                        path: "agent.multi_agent.search.api_key".to_string(),
                        value: Value::String(api_key),
                    });
                }

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
                if let Some(api_key) = prompt_secret(&theme, "Multi-agent answer API key", true)? {
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
                if let Some(secret) = prompt_secret(&theme, "Feishu app secret", true)? {
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
                if let Some(token) = prompt_secret(&theme, "Telegram bot token", true)? {
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
                if let Some(token) = prompt_secret(&theme, "Discord bot token", true)? {
                    mutations.push(ConfigMutation::Set {
                        path: "discord.bot_token".to_string(),
                        value: Value::String(token),
                    });
                }
            }
            ConfigureSection::Providers => {
                if let Some(keys) = prompt_secret(&theme, "OpenRouter API keys（逗号分隔）", true)?
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
                if let Some(keys) = prompt_secret(&theme, "FMP API keys（逗号分隔）", true)? {
                    mutations.push(provider_key_mutation(
                        "fmp.api_keys",
                        parse_csv_values(&keys),
                    ));
                    mutations.push(ConfigMutation::Set {
                        path: "fmp.api_key".to_string(),
                        value: Value::String(String::new()),
                    });
                }
                if let Some(keys) = prompt_secret(&theme, "Tavily API keys（逗号分隔）", true)?
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
    println!("{}", apply_message(&result.apply));
    println!(
        "config={} effective={}",
        paths.canonical_config_path.to_string_lossy(),
        paths.effective_config_path.to_string_lossy()
    );
    Ok(())
}

fn print_doctor_report_text(report: DoctorReport) {
    println!("canonical_config={}", report.canonical_config_path);
    println!("effective_config={}", report.effective_config_path);
    for check in report.checks {
        println!("[{}] {} {}", check.status, check.name, check.detail);
    }
}

async fn run_onboard(config_path: Option<&Path>, _args: OnboardArgs) -> Result<(), String> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err("`hone-cli onboard` 需要交互式终端（TTY）".to_string());
    }

    let (config, paths) = load_cli_config(config_path, true).map_err(|e| e.to_string())?;
    let theme = ColorfulTheme::default();

    println!("Hone onboarding");
    println!("  - 这个向导会写入 canonical config，并在需要时生成 runtime effective config。");
    println!("  - 任何步骤都可以先跳过，之后再通过 `hone-cli onboard` 或其他 CLI 子命令补配。");

    let runner = prompt_onboard_runner(&theme, &config)?;
    let mut mutations = build_runner_onboard_mutations(&theme, &config, runner)?;
    mutations.extend(build_channel_onboard_mutations(&theme, &config)?);

    let result = apply_mutations_and_generate(&paths, &mutations)?;
    println!();
    println!("{}", apply_message(&result.apply));
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

async fn run_cli() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        None | Some(Commands::Chat) => {
            let (core, paths) = load_cli_core(cli.config.as_deref()).map_err(|e| e.to_string())?;
            repl::run_chat(core, &paths.canonical_config_path.to_string_lossy()).await
        }
        Some(Commands::Onboard(args)) => run_onboard(cli.config.as_deref(), args).await,
        Some(Commands::Config { command }) => match command {
            ConfigCommands::File => {
                let paths = resolve_runtime_paths(cli.config.as_deref(), false)
                    .map_err(|e| e.to_string())?;
                println!("{}", paths.canonical_config_path.to_string_lossy());
                Ok(())
            }
            ConfigCommands::Get(args) => {
                let (_config, paths) =
                    load_cli_config(cli.config.as_deref(), false).map_err(|e| e.to_string())?;
                let value = read_config_path_value(&paths.canonical_config_path, &args.path)
                    .map_err(|e| e.to_string())?
                    .map(|value| redact_sensitive_value(&args.path, &value))
                    .unwrap_or(Value::Null);
                if args.json {
                    print_json(&value)
                } else {
                    println!("{}", value_to_pretty_text(&value));
                    Ok(())
                }
            }
            ConfigCommands::Set(args) => {
                let paths = resolve_runtime_paths(cli.config.as_deref(), true)
                    .map_err(|e| e.to_string())?;
                let value = yaml_value_from_cli(&args.value)?;
                let applied = apply_mutations_and_generate(
                    &paths,
                    &[ConfigMutation::Set {
                        path: args.path.clone(),
                        value,
                    }],
                )?;
                let updated = read_config_path_value(&paths.canonical_config_path, &args.path)
                    .map_err(|e| e.to_string())?
                    .map(|value| redact_sensitive_value(&args.path, &value))
                    .unwrap_or(Value::Null);
                let result = MutationResult {
                    config_path: paths.canonical_config_path.to_string_lossy().to_string(),
                    effective_config_path: paths
                        .effective_config_path
                        .to_string_lossy()
                        .to_string(),
                    config_revision: applied.config_revision,
                    applied_live: applied.apply.applied_live,
                    restarted_components: applied.apply.restarted_components.clone(),
                    restart_required: applied.apply.restart_required,
                    path: args.path,
                    value: updated,
                };
                println!("{}", apply_message(&applied.apply));
                println!("Updated {} in {}", result.path, result.config_path);
                println!("value={}", value_to_pretty_text(&result.value));
                Ok(())
            }
            ConfigCommands::Unset(args) => {
                let paths = resolve_runtime_paths(cli.config.as_deref(), true)
                    .map_err(|e| e.to_string())?;
                let applied = apply_mutations_and_generate(
                    &paths,
                    &[ConfigMutation::Unset {
                        path: args.path.clone(),
                    }],
                )?;
                let result = MutationResult {
                    config_path: paths.canonical_config_path.to_string_lossy().to_string(),
                    effective_config_path: paths
                        .effective_config_path
                        .to_string_lossy()
                        .to_string(),
                    config_revision: applied.config_revision,
                    applied_live: applied.apply.applied_live,
                    restarted_components: applied.apply.restarted_components.clone(),
                    restart_required: applied.apply.restart_required,
                    path: args.path,
                    value: Value::Null,
                };
                if args.json {
                    print_json(&result)
                } else {
                    println!("{}", apply_message(&applied.apply));
                    println!("Unset {} in {}", result.path, result.config_path);
                    Ok(())
                }
            }
            ConfigCommands::Validate(args) => {
                let (config, paths) =
                    load_cli_config(cli.config.as_deref(), false).map_err(|e| e.to_string())?;
                let response = serde_json::json!({
                    "configPath": paths.canonical_config_path,
                    "valid": true,
                    "runner": config.agent.runner,
                });
                if args.json {
                    print_json(&response)
                } else {
                    println!("Config valid: {}", response["configPath"]);
                    Ok(())
                }
            }
        },
        Some(Commands::Configure(args)) => run_configure(cli.config.as_deref(), args),
        Some(Commands::Models { command }) => match command {
            ModelsCommands::Status(args) => {
                let (config, _) =
                    load_cli_config(cli.config.as_deref(), false).map_err(|e| e.to_string())?;
                let report = build_model_status(&config);
                if args.json {
                    print_json(&report)
                } else {
                    println!("runner={}", report.runner);
                    println!(
                        "primary={} variant={} base_url={}",
                        report.opencode_model, report.opencode_variant, report.opencode_base_url
                    );
                    println!(
                        "auxiliary={} base_url={} configured={}",
                        report.auxiliary_model,
                        report.auxiliary_base_url,
                        report.auxiliary_api_key_configured
                    );
                    println!(
                        "multi-agent search={} answer={} variant={}",
                        report.search_model, report.answer_model, report.answer_variant
                    );
                    Ok(())
                }
            }
            ModelsCommands::Set(args) => {
                let paths = resolve_runtime_paths(cli.config.as_deref(), true)
                    .map_err(|e| e.to_string())?;
                let mutations = build_model_mutations(&args)?;
                let result = apply_mutations_and_generate(&paths, &mutations)?;
                println!("{}", apply_message(&result.apply));
                println!(
                    "config={} effective={}",
                    paths.canonical_config_path.to_string_lossy(),
                    paths.effective_config_path.to_string_lossy()
                );
                Ok(())
            }
        },
        Some(Commands::Channels { command }) => match command {
            ChannelsCommands::List(args) => {
                let (config, _) =
                    load_cli_config(cli.config.as_deref(), false).map_err(|e| e.to_string())?;
                let report = build_channel_reports(&config);
                if args.json {
                    print_json(&report)
                } else {
                    for channel in report {
                        println!(
                            "{} enabled={} auth_configured={}{}",
                            channel.channel,
                            channel.enabled,
                            channel.auth_configured,
                            channel
                                .chat_scope
                                .as_ref()
                                .map(|scope| format!(" chat_scope={scope}"))
                                .unwrap_or_default()
                        );
                    }
                    Ok(())
                }
            }
            ChannelsCommands::Set(args) => {
                let paths = resolve_runtime_paths(cli.config.as_deref(), true)
                    .map_err(|e| e.to_string())?;
                let mutations = build_channel_mutations(&args)?;
                let result = apply_mutations_and_generate(&paths, &mutations)?;
                println!("{}", apply_message(&result.apply));
                println!(
                    "config={} effective={}",
                    paths.canonical_config_path.to_string_lossy(),
                    paths.effective_config_path.to_string_lossy()
                );
                Ok(())
            }
            ChannelsCommands::Enable(args) => {
                let paths = resolve_runtime_paths(cli.config.as_deref(), true)
                    .map_err(|e| e.to_string())?;
                let path = match args.channel {
                    ChannelKind::Imessage => "imessage.enabled",
                    ChannelKind::Feishu => "feishu.enabled",
                    ChannelKind::Telegram => "telegram.enabled",
                    ChannelKind::Discord => "discord.enabled",
                };
                let result = apply_mutations_and_generate(
                    &paths,
                    &[ConfigMutation::Set {
                        path: path.to_string(),
                        value: Value::Bool(true),
                    }],
                )?;
                println!("{}", apply_message(&result.apply));
                println!("Enabled {path}");
                Ok(())
            }
            ChannelsCommands::Disable(args) => {
                let paths = resolve_runtime_paths(cli.config.as_deref(), true)
                    .map_err(|e| e.to_string())?;
                let path = match args.channel {
                    ChannelKind::Imessage => "imessage.enabled",
                    ChannelKind::Feishu => "feishu.enabled",
                    ChannelKind::Telegram => "telegram.enabled",
                    ChannelKind::Discord => "discord.enabled",
                };
                let result = apply_mutations_and_generate(
                    &paths,
                    &[ConfigMutation::Set {
                        path: path.to_string(),
                        value: Value::Bool(false),
                    }],
                )?;
                println!("{}", apply_message(&result.apply));
                println!("Disabled {path}");
                Ok(())
            }
        },
        Some(Commands::Status(args)) => {
            let report = build_status_report(cli.config.as_deref()).await?;
            if args.json {
                print_json(&report)
            } else {
                println!("canonical_config={}", report.canonical_config_path);
                println!("effective_config={}", report.effective_config_path);
                let primary_model = if report.models.opencode_inherits_local_config {
                    "<opencode default>"
                } else if non_empty(&report.models.opencode_model) {
                    report.models.opencode_model.as_str()
                } else {
                    "<unset>"
                };
                let primary_variant = if report.models.opencode_inherits_local_config {
                    "<inherited>"
                } else if non_empty(&report.models.opencode_variant) {
                    report.models.opencode_variant.as_str()
                } else {
                    "<unset>"
                };
                println!(
                    "runner={} primary_model={} variant={}",
                    report.models.runner, primary_model, primary_variant
                );
                if report.models.opencode_inherits_local_config {
                    println!(
                        "opencode_config_source=local-opencode (~/.config/opencode/opencode.json or opencode.jsonc)"
                    );
                }
                println!("data_dir={}", report.data_dir);
                println!("skills_dir={}", report.skills_dir);
                if let Some(warning) = &report.legacy_warning {
                    println!("legacy_warning={warning}");
                }
                let enabled = report
                    .channels
                    .iter()
                    .filter(|channel| channel.enabled)
                    .map(|channel| channel.channel.as_str())
                    .collect::<Vec<_>>();
                println!(
                    "enabled_channels={}",
                    if enabled.is_empty() {
                        "<none>".to_string()
                    } else {
                        enabled.join(",")
                    }
                );
                for binary in report.binaries {
                    println!(
                        "binary {} ready={} {}",
                        binary.name, binary.available, binary.detail
                    );
                }
                Ok(())
            }
        }
        Some(Commands::Doctor(args)) => {
            let report = build_doctor_report(cli.config.as_deref()).await;
            if args.json {
                print_json(&report)
            } else {
                print_doctor_report_text(report);
                Ok(())
            }
        }
        Some(Commands::Start) => start::run_start(cli.config.as_deref()).await,
    }
}

#[tokio::main]
async fn main() {
    if let Err(error) = run_cli().await {
        eprintln!("❌ {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_parses_config_get_command() {
        let cli = Cli::try_parse_from(["hone-cli", "config", "get", "agent.runner"]).unwrap();
        match cli.command {
            Some(Commands::Config {
                command: ConfigCommands::Get(args),
            }) => assert_eq!(args.path, "agent.runner"),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_models_set_short_form() {
        let cli = Cli::try_parse_from([
            "hone-cli",
            "models",
            "set",
            "--runner",
            "opencode_acp",
            "--model",
            "openrouter/openai/gpt-5.4",
            "--variant",
            "medium",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Models {
                command: ModelsCommands::Set(args),
            }) => {
                assert_eq!(args.runner.as_deref(), Some("opencode_acp"));
                assert_eq!(args.model.as_deref(), Some("openrouter/openai/gpt-5.4"));
                assert_eq!(args.variant.as_deref(), Some("medium"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_setup_alias_to_onboard() {
        let cli = Cli::try_parse_from(["hone-cli", "setup"]).unwrap();
        match cli.command {
            Some(Commands::Onboard(_)) => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

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
    fn resolve_required_field_attempt_disables_channel_when_empty_and_no_current_value() {
        let resolution = resolve_required_field_attempt(
            Some(String::new()),
            "",
            RequiredFieldEmptyAction::DisableChannel,
        );

        assert_eq!(resolution, RequiredFieldResolution::DisableChannel);
    }

    #[test]
    fn resolve_required_field_attempt_retries_when_empty_and_no_current_value() {
        let resolution = resolve_required_field_attempt(
            Some(String::new()),
            "",
            RequiredFieldEmptyAction::Retry,
        );

        assert_eq!(resolution, RequiredFieldResolution::Retry);
    }

    #[test]
    fn resolve_required_field_attempt_keeps_existing_value_on_empty_input() {
        let resolution = resolve_required_field_attempt(
            Some(String::new()),
            "existing-secret",
            RequiredFieldEmptyAction::DisableChannel,
        );

        assert_eq!(
            resolution,
            RequiredFieldResolution::Value("existing-secret".to_string())
        );
    }
}
