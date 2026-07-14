//! `hone-cli status` 和 `hone-cli doctor` 的报告数据结构 + 构建逻辑。
//!
//! 两个子命令共用大量视图：
//! - `status` 强调「当前配置读起来是怎么样」(models / channels / api_keys / binaries)
//! - `doctor` 强调「运行时依赖和文件系统状态是否健康」(路径存在 / 二进制可用 / channel auth 完整)
//!
//! 所有 Report 都实现 `Serialize`,方便 JSON 输出给其它工具消费。文本输出走
//! [`print_doctor_report_text`]；status 的人类可读输出留在调用方
//! (`run_cli` 会根据 `--json` flag 分派)。

use std::path::Path;
use std::process::Command as StdCommand;

use serde::Serialize;

use crate::common::{load_cli_config, resolve_runtime_paths};
use crate::discord_token::discord_token_doctor_check;
use crate::{non_empty, start};

const MAX_BINARY_CHECK_DETAIL_CHARS: usize = 300;

#[derive(Debug, Serialize)]
pub(crate) struct ModelStatusReport {
    pub runner: String,
    pub hone_cloud_base_url: String,
    pub hone_cloud_model: String,
    pub hone_cloud_api_key_configured: bool,
    pub codex_model: String,
    pub codex_acp_model: String,
    pub codex_acp_variant: String,
    pub opencode_base_url: String,
    pub opencode_model: String,
    pub opencode_variant: String,
    pub opencode_api_key_configured: bool,
    pub opencode_inherits_local_config: bool,
    pub auxiliary_base_url: String,
    pub auxiliary_model: String,
    pub auxiliary_api_key_configured: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ChannelStatusReport {
    pub channel: String,
    pub enabled: bool,
    pub auth_configured: bool,
    pub chat_scope: Option<String>,
    pub details: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct BinaryStatus {
    pub name: String,
    pub available: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct StatusReport {
    pub canonical_config_path: String,
    pub effective_config_path: String,
    pub data_dir: String,
    pub skills_dir: String,
    pub models: ModelStatusReport,
    pub channels: Vec<ChannelStatusReport>,
    pub api_keys: ApiKeySummary,
    pub binaries: Vec<BinaryStatus>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ApiKeySummary {
    pub hone_cloud: bool,
    pub openrouter: bool,
    pub primary_route: bool,
    pub auxiliary: bool,
    pub fmp: bool,
    pub tavily: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct DoctorCheck {
    pub name: String,
    pub status: &'static str,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DoctorReport {
    pub canonical_config_path: String,
    pub effective_config_path: String,
    pub checks: Vec<DoctorCheck>,
}

/// 真跑一下候选二进制(带 `--help` 之类的 no-op 参数),看能不能命中 PATH。
pub(crate) fn binary_check(name: &str, help_arg: &str) -> BinaryStatus {
    let output = StdCommand::new(name).arg(help_arg).output();
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
            let detail = command_output_detail(&stdout, &stderr);
            BinaryStatus {
                name: name.to_string(),
                available: true,
                detail,
            }
        }
        Err(error) => BinaryStatus {
            name: name.to_string(),
            available: false,
            detail: truncate_binary_check_detail(&error.to_string()),
        },
    }
}

fn command_output_detail(stdout: &str, stderr: &str) -> String {
    let detail = if !stdout.is_empty() {
        stdout
    } else if !stderr.is_empty() {
        stderr
    } else {
        return "命令可执行".to_string();
    };
    truncate_binary_check_detail(detail)
}

fn truncate_binary_check_detail(detail: &str) -> String {
    if detail.chars().count() <= MAX_BINARY_CHECK_DETAIL_CHARS {
        return detail.to_string();
    }
    detail
        .chars()
        .take(MAX_BINARY_CHECK_DETAIL_CHARS)
        .collect::<String>()
        + "..."
}

/// 查 hone 自己发布的 sidecar 二进制(hone-console-page / hone-mcp / 各 channel bin)
/// 是否位于 `hone-cli` 的 `start::locate_binary` 搜索路径里。
pub(crate) fn runtime_binary_status(binary: &str) -> BinaryStatus {
    match start::locate_binary(binary) {
        Some(path) => BinaryStatus {
            name: binary.to_string(),
            available: true,
            detail: path.to_string_lossy().to_string(),
        },
        None => BinaryStatus {
            name: binary.to_string(),
            available: false,
            detail: truncate_binary_check_detail(&start::missing_binary_message(binary, None)),
        },
    }
}

pub(crate) fn chat_scope_label(scope: hone_core::config::ChatScope) -> String {
    match scope {
        hone_core::config::ChatScope::DmOnly => "DM_ONLY".to_string(),
        hone_core::config::ChatScope::GroupchatOnly => "GROUPCHAT_ONLY".to_string(),
        hone_core::config::ChatScope::All => "ALL".to_string(),
    }
}

pub(crate) fn build_model_status(config: &hone_core::HoneConfig) -> ModelStatusReport {
    let opencode_inherits_local_config = config.agent.runner == "opencode_acp"
        && !non_empty(&config.agent.opencode.model)
        && !non_empty(&config.agent.opencode.variant)
        && !non_empty(&config.agent.opencode.api_base_url)
        && !non_empty(&config.agent.opencode.api_key);
    ModelStatusReport {
        runner: config.agent.runner.clone(),
        hone_cloud_base_url: config.agent.hone_cloud.base_url.clone(),
        hone_cloud_model: config.agent.hone_cloud.model.clone(),
        hone_cloud_api_key_configured: non_empty(&config.agent.hone_cloud.api_key),
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
        auxiliary_api_key_configured: non_empty(&config.llm.auxiliary.api_key),
    }
}

pub(crate) fn build_channel_reports(config: &hone_core::HoneConfig) -> Vec<ChannelStatusReport> {
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
            details: vec![
                format!(
                    "app_id={}",
                    if non_empty(&config.feishu.app_id) {
                        "<set>"
                    } else {
                        "<empty>"
                    }
                ),
                format!("allow_emails={}", config.feishu.allow_emails.len()),
                format!("allow_mobiles={}", config.feishu.allow_mobiles.len()),
                format!("allow_open_ids={}", config.feishu.allow_open_ids.len()),
            ],
        },
        ChannelStatusReport {
            channel: "telegram".to_string(),
            enabled: config.telegram.enabled,
            auth_configured: non_empty(&config.telegram.bot_token),
            chat_scope: Some(chat_scope_label(config.telegram.chat_scope)),
            details: vec![
                format!(
                    "bot_token={}",
                    if non_empty(&config.telegram.bot_token) {
                        "<set>"
                    } else {
                        "<empty>"
                    }
                ),
                format!("allow_from={}", config.telegram.allow_from.len()),
            ],
        },
        ChannelStatusReport {
            channel: "discord".to_string(),
            enabled: config.discord.enabled,
            auth_configured: non_empty(&config.discord.bot_token),
            chat_scope: Some(chat_scope_label(config.discord.chat_scope)),
            details: vec![
                format!(
                    "bot_token={}",
                    if non_empty(&config.discord.bot_token) {
                        "<set>"
                    } else {
                        "<empty>"
                    }
                ),
                format!("allow_from={}", config.discord.allow_from.len()),
            ],
        },
    ]
}

pub(crate) fn build_api_key_summary(config: &hone_core::HoneConfig) -> ApiKeySummary {
    ApiKeySummary {
        hone_cloud: non_empty(&config.agent.hone_cloud.api_key),
        openrouter: !config.llm.openrouter_key_pool().is_empty(),
        primary_route: non_empty(&config.agent.opencode.api_key),
        auxiliary: non_empty(&config.llm.auxiliary.api_key),
        fmp: !config.fmp.effective_key_pool().is_empty(),
        tavily: !config
            .search
            .api_keys
            .iter()
            .all(|key| key.trim().is_empty()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_summary_reports_hone_cloud_key() {
        let mut config = hone_core::HoneConfig::default();
        assert!(!build_api_key_summary(&config).hone_cloud);

        config.agent.hone_cloud.api_key = "hc-test".to_string();
        assert!(build_api_key_summary(&config).hone_cloud);
    }

    #[test]
    fn model_status_reports_hone_cloud_route() {
        let mut config = hone_core::HoneConfig::default();
        config.agent.runner = "hone_cloud".to_string();
        config.agent.hone_cloud.api_key = "hc-test".to_string();

        let report = build_model_status(&config);
        assert_eq!(report.runner, "hone_cloud");
        assert_eq!(report.hone_cloud_base_url, "https://hone-claw.com");
        assert_eq!(report.hone_cloud_model, "hone-cloud");
        assert!(report.hone_cloud_api_key_configured);
    }

    #[test]
    fn command_output_detail_is_bounded() {
        let detail = command_output_detail(&"x".repeat(MAX_BINARY_CHECK_DETAIL_CHARS + 10), "");
        assert_eq!(
            detail,
            format!("{}...", "x".repeat(MAX_BINARY_CHECK_DETAIL_CHARS))
        );
    }

    #[test]
    fn runtime_binary_status_missing_detail_is_actionable_and_bounded() {
        let status = runtime_binary_status("hone-definitely-missing");

        assert!(!status.available);
        assert!(status.detail.contains("已检查"));
        assert!(status.detail.contains("HONE_SOURCE_ROOT"));
        assert!(status.detail.chars().count() <= MAX_BINARY_CHECK_DETAIL_CHARS + 3);
    }
}

/// 根据 `agent.runner` 的配置值,查对应 CLI 二进制的 probe 指令。
/// `hone_cloud` 不挂本机 CLI,返回 `None`。
pub(crate) fn runner_binary_name(runner: &str) -> Option<(&'static str, &'static str)> {
    hone_core::config::AgentRunnerKind::from_config_value(runner)
        .cli_probe()
        .map(|probe| (probe.binary, probe.arg))
}

pub(crate) async fn build_status_report(
    config_path: Option<&Path>,
) -> Result<StatusReport, String> {
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
        models: build_model_status(&config),
        channels: build_channel_reports(&config),
        api_keys: build_api_key_summary(&config),
        binaries,
    })
}

pub(crate) async fn build_doctor_report(config_path: Option<&Path>) -> DoctorReport {
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

            match load_cli_config(config_path, false) {
                Ok((config, loaded_paths)) => {
                    checks.push(DoctorCheck {
                        name: "config-parse".to_string(),
                        status: "ok",
                        detail: "配置解析成功".to_string(),
                    });
                    if non_empty(&config.discord.bot_token) {
                        checks.push(discord_token_doctor_check(
                            crate::i18n::Lang::from_locale(config.language),
                            &config.discord.bot_token,
                        ));
                    }
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

pub(crate) fn print_doctor_report_text(report: DoctorReport) {
    println!("canonical_config={}", report.canonical_config_path);
    println!("effective_config={}", report.effective_config_path);
    for check in report.checks {
        println!("[{}] {} {}", check.status, check.name, check.detail);
    }
}
