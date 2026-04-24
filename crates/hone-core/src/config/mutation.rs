//! 配置变更管道：路径级 set/unset、影响面分类、敏感字段脱敏。
//!
//! Web 控制台 / CLI 的「改一行配置」请求都会走这里：先 `parse_config_path`
//! 解析点路径,`apply_config_mutations` 原地写入并重新校验整份 `HoneConfig`,
//! 再用 `classify_config_paths` 告诉调用方本次变更是能热生效、需要重启某些
//! channel、还是必须整个进程重启。
//!
//! `is_sensitive_config_path` / `redact_sensitive_value` 负责把涉及
//! `api_key` / `secret` / `token` / `password` 的字段在日志与返回值里脱敏。

use serde::Serialize;
use serde_yaml::{Mapping, Value};
use std::collections::BTreeSet;
use std::path::Path;

use super::HoneConfig;
use super::channels::ChatScope;
use super::yaml::{
    atomic_write_yaml, get_value_at_segments, parse_config_path, read_yaml_value,
    set_value_at_segments, unset_value_at_segments, yaml_revision,
};

#[derive(Debug, Clone)]
pub enum ConfigMutation {
    Set { path: String, value: Value },
    Unset { path: String },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigApplyPlan {
    pub changed_paths: Vec<String>,
    pub applied_live: bool,
    pub restarted_components: Vec<String>,
    pub restart_required: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigMutationResult {
    pub config: HoneConfig,
    pub config_revision: String,
    pub apply: ConfigApplyPlan,
}

/// 根据本次变更涉及的配置路径,推断应用策略：
/// - `restart_required = true`：进程需整体重启才能看到新值（例如 `storage.*`,
///   或除了 `logging.level` 以外的 logging 字段；这些改动会影响运行时 singletons）
/// - `restarted_components` 非空：对应 channel 子进程需要重启（改了 imessage /
///   telegram / discord / feishu 的任意字段）
/// - `applied_live = true`：可以热更新,不需要重启任何进程
///
/// 任何未知顶层字段都按最保守的「整体重启」处理,避免漏改 singleton 造成不一致。
pub fn classify_config_paths(paths: &[String]) -> ConfigApplyPlan {
    let mut restarted_components = BTreeSet::new();
    let mut restart_required = false;
    let mut applied_live = false;

    for path in paths {
        let root = path.split('.').next().unwrap_or_default();
        let logging_full_restart = root == "logging" && path.trim() != "logging.level";
        let full_restart = root == "storage"
            || path.trim() == "security.kb_actor_isolation"
            || logging_full_restart;

        if full_restart {
            restart_required = true;
            continue;
        }

        match root {
            "imessage" => {
                restarted_components.insert("imessage".to_string());
            }
            "telegram" => {
                restarted_components.insert("telegram".to_string());
            }
            "discord" => {
                restarted_components.insert("discord".to_string());
            }
            "feishu" => {
                restarted_components.insert("feishu".to_string());
            }
            "agent" | "llm" | "group_context" | "admins" | "web" | "fmp" | "search"
            | "nano_banana" | "x" => {
                applied_live = true;
            }
            "security" if path.trim_start().starts_with("security.tool_guard.") => {
                applied_live = true;
            }
            "logging" if path.trim() == "logging.level" => {
                applied_live = true;
            }
            _ => {
                restart_required = true;
            }
        }
    }

    ConfigApplyPlan {
        changed_paths: paths.to_vec(),
        applied_live: applied_live && restarted_components.is_empty() && !restart_required,
        restarted_components: restarted_components.into_iter().collect(),
        restart_required,
    }
}

pub fn read_config_path_value(config_path: &Path, path: &str) -> crate::HoneResult<Option<Value>> {
    let current = read_yaml_value(config_path)?;
    let segments = parse_config_path(path)?;
    Ok(get_value_at_segments(&current, &segments)?.cloned())
}

pub fn apply_config_mutations(
    config_path: &Path,
    mutations: &[ConfigMutation],
) -> crate::HoneResult<ConfigMutationResult> {
    let mut current = if config_path.exists() {
        read_yaml_value(config_path)?
    } else {
        Value::Mapping(Mapping::new())
    };
    if current.is_null() {
        current = Value::Mapping(Mapping::new());
    }
    let changed_paths: Vec<String> = mutations
        .iter()
        .map(|mutation| match mutation {
            ConfigMutation::Set { path, .. } | ConfigMutation::Unset { path } => path.clone(),
        })
        .collect();

    for mutation in mutations {
        match mutation {
            ConfigMutation::Set { path, value } => {
                let segments = parse_config_path(path)?;
                set_value_at_segments(&mut current, &segments, value.clone())?;
            }
            ConfigMutation::Unset { path } => {
                let segments = parse_config_path(path)?;
                unset_value_at_segments(&mut current, &segments)?;
            }
        }
    }

    HoneConfig::from_merged_value(current.clone())?;
    let yaml = serde_yaml::to_string(&current)
        .map_err(|e| crate::HoneError::Config(format!("配置序列化失败: {e}")))?;
    atomic_write_yaml(config_path, &yaml)?;
    Ok(ConfigMutationResult {
        config: HoneConfig::from_file(config_path)?,
        config_revision: yaml_revision(&current)?,
        apply: classify_config_paths(&changed_paths),
    })
}

pub fn is_sensitive_config_path(path: &str) -> bool {
    let lowered = path.to_ascii_lowercase();
    lowered.contains("api_key")
        || lowered.contains("secret")
        || lowered.contains("token")
        || lowered.contains("password")
        || lowered.contains("auth_token")
}

pub fn redact_sensitive_value(path: &str, value: &Value) -> Value {
    if !is_sensitive_config_path(path) {
        return value.clone();
    }
    match value {
        Value::Null => Value::Null,
        Value::Sequence(items) => Value::Sequence(
            items
                .iter()
                .map(|item| redact_sensitive_value(path, item))
                .collect(),
        ),
        _ => Value::String("<redacted>".to_string()),
    }
}

pub(super) fn validate_channel_chat_scope(
    channel: &str,
    chat_scope: ChatScope,
) -> crate::HoneResult<()> {
    let raw = match chat_scope {
        ChatScope::DmOnly => "DM_ONLY",
        ChatScope::GroupchatOnly => "GROUPCHAT_ONLY",
        ChatScope::All => "ALL",
    };
    if raw.trim().is_empty() {
        return Err(crate::HoneError::Config(format!(
            "{channel}.chat_scope 不能为空"
        )));
    }
    Ok(())
}
