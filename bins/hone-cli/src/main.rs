mod cleanup;
mod common;
mod discord_token;
mod mutations;
mod onboard;
mod probe;
mod prompts;
mod repl;
mod reports;
mod start;
mod yaml_io;

use cleanup::{CleanupArgs, run_cleanup};
use discord_token::prompt_optional_discord_token;
use mutations::{
    ChannelKind, ChannelSetArgs, ChannelToggleArgs, ModelsSetArgs, build_channel_mutations,
    build_model_mutations, parse_csv_values, provider_key_mutation,
};
use onboard::{OnboardArgs, run_onboard};
use prompts::{prompt_bool, prompt_secret, prompt_text, prompt_visible_credential};
use reports::{
    build_channel_reports, build_doctor_report, build_model_status, build_status_report,
    print_doctor_report_text,
};
use yaml_io::{
    apply_message, apply_mutations_and_generate, print_json, value_to_pretty_text,
    yaml_value_from_cli,
};

use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use common::{load_cli_config, load_cli_core, resolve_runtime_paths};
use dialoguer::theme::ColorfulTheme;
use hone_core::config::{
    ConfigMutation, is_sensitive_config_path, read_config_path_value, redact_sensitive_value,
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
    Cleanup(CleanupArgs),
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
    Probe(ProbeArgs),
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

#[derive(Args, Debug, Clone)]
struct ProbeArgs {
    #[arg(long)]
    channel: String,
    #[arg(long = "user-id")]
    user_id: String,
    #[arg(long)]
    scope: Option<String>,
    #[arg(long)]
    group: bool,
    #[arg(long)]
    admin: bool,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    show_events: bool,
    #[arg(long)]
    query: String,
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

/// `config set` / `config get` 的输出结构（用于 `--json` 模式序列化)。
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

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum ConfigureSection {
    Agent,
    Channels,
    Providers,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub(crate) enum CliChatScope {
    DmOnly,
    GroupchatOnly,
    All,
}

impl CliChatScope {
    pub(crate) fn as_config_value(&self) -> &'static str {
        match self {
            Self::DmOnly => "DM_ONLY",
            Self::GroupchatOnly => "GROUPCHAT_ONLY",
            Self::All => "ALL",
        }
    }

    pub(crate) fn label(&self) -> &'static str {
        self.as_config_value()
    }

    pub(crate) fn from_chat_scope(scope: hone_core::config::ChatScope) -> Self {
        match scope {
            hone_core::config::ChatScope::DmOnly => Self::DmOnly,
            hone_core::config::ChatScope::GroupchatOnly => Self::GroupchatOnly,
            hone_core::config::ChatScope::All => Self::All,
        }
    }
}

/// `trim` 后非空判定。被 main.rs 及拆出的 `reports` module 共用。
pub(crate) fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
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
                if let Some(token) = prompt_visible_credential(
                    &theme,
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

async fn run_cli() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        None | Some(Commands::Chat) => {
            let (core, paths) = load_cli_core(cli.config.as_deref()).map_err(|e| e.to_string())?;
            repl::run_chat(core, &paths.canonical_config_path.to_string_lossy()).await
        }
        Some(Commands::Onboard(args)) => run_onboard(cli.config.as_deref(), args).await,
        Some(Commands::Cleanup(args)) => run_cleanup(args),
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
        Some(Commands::Probe(args)) => {
            let (core, paths) = load_cli_core(cli.config.as_deref()).map_err(|e| e.to_string())?;
            probe::run_probe(core, &paths.canonical_config_path.to_string_lossy(), args).await
        }
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
    fn cli_parses_cleanup_command() {
        let cli = Cli::try_parse_from(["hone-cli", "cleanup", "--all", "--yes"]).unwrap();
        match cli.command {
            Some(Commands::Cleanup(args)) => {
                assert!(args.all);
                assert!(args.yes);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn cli_parses_probe_command() {
        let cli = Cli::try_parse_from([
            "hone-cli",
            "probe",
            "--channel",
            "telegram",
            "--user-id",
            "8039067465",
            "--query",
            "夜盘 aaoi 和 cohr 为什么在跌",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Probe(args)) => {
                assert_eq!(args.channel, "telegram");
                assert_eq!(args.user_id, "8039067465");
                assert_eq!(args.query, "夜盘 aaoi 和 cohr 为什么在跌");
                assert!(args.show_events);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
