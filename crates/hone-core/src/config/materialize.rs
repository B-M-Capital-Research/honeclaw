//! Canonical 配置 seed / 升级 / effective 产出。
//!
//! 概念：
//! - **canonical 配置**：用户可编辑的权威 `config.yaml`（repo 根或安装后的用户目录）
//! - **legacy runtime 配置**：历史上写在 `data/runtime/config_runtime.yaml` 的残留,
//!   现在只用来一次性补齐 canonical 中仍为空/种子态的字段
//! - **effective 配置**：启动时根据 canonical 实时生成的 `data/runtime/effective-config.yaml`,
//!   供运行时进程消费（只读快照）
//!
//! 本模块就是围绕这三者流转的：
//! - `seed_canonical_config_from_source`：从 `config.example.yaml` 或历史 canonical 拷一份到目标位置（只在不存在时触发）
//! - `promote_legacy_runtime_agent_settings`：保守地把 legacy 里真正非空、canonical 仍为空的字段提升进 canonical
//! - `generate_effective_config`：canonical → effective 快照 + revision hash

use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};

use super::HoneConfig;
use super::yaml::{
    atomic_write_yaml, bool_path_is_false_or_missing, config_io_error, get_string_at_path,
    get_value_at_path, read_yaml_value, sequence_path_is_empty, set_value_at_path,
    string_path_is_blank, yaml_revision,
};

pub fn effective_config_path(runtime_dir: impl AsRef<Path>) -> PathBuf {
    runtime_dir.as_ref().join("effective-config.yaml")
}

pub fn canonical_config_candidate() -> PathBuf {
    PathBuf::from("config.yaml")
}

fn read_yaml_value_or_empty_mapping(path: &Path) -> crate::HoneResult<Value> {
    let mut value = read_yaml_value(path)?;
    if value.is_null() {
        value = Value::Mapping(Mapping::new());
    }
    Ok(value)
}

fn write_yaml_value(path: &Path, value: &Value, config_label: &str) -> crate::HoneResult<()> {
    let yaml = serde_yaml::to_string(value).map_err(|e| {
        crate::HoneError::Config(format!(
            "{config_label} 配置序列化失败 ({}): {e}",
            path.display()
        ))
    })?;
    atomic_write_yaml(path, &yaml)
}

fn copy_relative_system_prompt_asset(
    base_config_path: &Path,
    runtime_config_path: &Path,
) -> crate::HoneResult<()> {
    let base_value = read_yaml_value(base_config_path)?;
    let prompt_path = base_value
        .as_mapping()
        .and_then(|root| root.get(Value::String("agent".to_string())))
        .and_then(|agent| agent.as_mapping())
        .and_then(|agent| agent.get(Value::String("system_prompt_path".to_string())))
        .and_then(|value| value.as_str())
        .map(str::trim)
        .unwrap_or("");

    if prompt_path.is_empty() || Path::new(prompt_path).is_absolute() {
        return Ok(());
    }

    let Some(base_parent) = base_config_path.parent() else {
        return Ok(());
    };
    let Some(runtime_parent) = runtime_config_path.parent() else {
        return Ok(());
    };

    let source = base_parent.join(prompt_path);
    if !source.exists() {
        return Ok(());
    }

    let dest = runtime_parent.join(prompt_path);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| config_io_error("创建 system_prompt_path 目录失败", parent, e))?;
    }
    if !dest.exists() {
        fs::copy(&source, &dest).map_err(|e| {
            crate::HoneError::Config(format!(
                "复制 system_prompt_path 失败 ({} -> {}): {e}",
                source.display(),
                dest.display()
            ))
        })?;
    }
    Ok(())
}

pub fn seed_canonical_config_from_source(
    canonical_config_path: &Path,
    source_config_path: &Path,
) -> crate::HoneResult<()> {
    if canonical_config_path.exists() {
        return Ok(());
    }
    let Some(parent) = canonical_config_path.parent() else {
        return Err(crate::HoneError::Config(format!(
            "canonical 配置路径缺少父目录: {}",
            canonical_config_path.display()
        )));
    };
    fs::create_dir_all(parent)
        .map_err(|e| config_io_error("创建 canonical 配置目录失败", parent, e))?;
    fs::copy(source_config_path, canonical_config_path).map_err(|e| {
        crate::HoneError::Config(format!(
            "复制 canonical 配置失败 ({} -> {}): {e}",
            source_config_path.display(),
            canonical_config_path.display()
        ))
    })?;
    copy_relative_system_prompt_asset(source_config_path, canonical_config_path)?;
    Ok(())
}

/// 判断 canonical 配置里的 runner 字段是不是 seed 默认值,需要被 legacy 覆盖。
/// 只有空字符串以及历史/当前 seed 默认值会被视为「用户未显式设置」。
fn canonical_runner_looks_seeded(runner: &str) -> bool {
    matches!(
        runner.trim(),
        "" | "function_calling" | "multi-agent" | "codex_cli" | "codex_acp"
    )
}

/// chat_scope 在 canonical 中是不是 seed 默认值（空或 DM_ONLY）,需要被 legacy 覆盖。
fn canonical_chat_scope_looks_seeded(scope: &str) -> bool {
    matches!(scope.trim(), "" | "DM_ONLY")
}

fn opencode_field_paths() -> [&'static str; 4] {
    [
        "agent.opencode.api_base_url",
        "agent.opencode.api_key",
        "agent.opencode.model",
        "agent.opencode.variant",
    ]
}

fn canonical_opencode_block_is_blank(current: &Value) -> crate::HoneResult<bool> {
    for path in opencode_field_paths() {
        if !string_path_is_blank(current, path)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn promote_legacy_opencode(
    canonical: &mut Value,
    legacy: &Value,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<bool> {
    let Some(legacy_opencode) = get_value_at_path(legacy, "agent.opencode")? else {
        return Ok(false);
    };

    if canonical_opencode_block_is_blank(canonical)? {
        set_value_at_path(canonical, "agent.opencode", legacy_opencode.clone())?;
        changed_paths.push("agent.opencode".to_string());
        return Ok(true);
    }

    let mut migrated = false;
    for path in [
        "agent.opencode.api_base_url",
        "agent.opencode.model",
        "agent.opencode.variant",
    ] {
        if string_path_is_blank(canonical, path)?
            && let Some(legacy_value) = get_value_at_path(legacy, path)?
            && legacy_value
                .as_str()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
        {
            set_value_at_path(canonical, path, legacy_value.clone())?;
            changed_paths.push(path.to_string());
            migrated = true;
        }
    }

    Ok(migrated)
}

fn promote_legacy_llm_settings(
    canonical: &mut Value,
    legacy: &Value,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    if string_path_is_blank(canonical, "llm.auxiliary.api_key")?
        && let Some(legacy_auxiliary) = get_value_at_path(legacy, "llm.auxiliary")?
    {
        set_value_at_path(canonical, "llm.auxiliary", legacy_auxiliary.clone())?;
        changed_paths.push("llm.auxiliary".to_string());
    }

    if !openrouter_key_targets_are_blank(canonical)? {
        return Ok(());
    }

    if let Some(legacy_openrouter_keys) = get_value_at_path(legacy, "llm.openrouter.api_keys")? {
        set_value_at_path(
            canonical,
            "llm.providers.openrouter.api_keys",
            legacy_openrouter_keys.clone(),
        )?;
        changed_paths.push("llm.providers.openrouter.api_keys".to_string());
        return Ok(());
    }

    if let Some(legacy_openrouter_key) = get_value_at_path(legacy, "llm.openrouter.api_key")?
        && legacy_openrouter_key
            .as_str()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    {
        set_value_at_path(
            canonical,
            "llm.providers.openrouter.api_key",
            legacy_openrouter_key.clone(),
        )?;
        changed_paths.push("llm.providers.openrouter.api_key".to_string());
    }

    Ok(())
}

fn openrouter_key_targets_are_blank(canonical: &Value) -> crate::HoneResult<bool> {
    Ok(
        string_path_is_blank(canonical, "llm.providers.openrouter.api_key")?
            && sequence_path_is_empty(canonical, "llm.providers.openrouter.api_keys")?
            && string_path_is_blank(canonical, "llm.openrouter.api_key")?
            && sequence_path_is_empty(canonical, "llm.openrouter.api_keys")?,
    )
}

fn promote_legacy_channel_settings(
    canonical: &mut Value,
    legacy: &Value,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    for channel in ["feishu", "telegram", "discord", "imessage"] {
        promote_legacy_channel_enabled(canonical, legacy, channel, changed_paths)?;
        promote_legacy_channel_scope(canonical, legacy, channel, changed_paths)?;
        promote_legacy_channel_credentials(canonical, legacy, channel, changed_paths)?;
    }

    Ok(())
}

fn promote_legacy_channel_enabled(
    canonical: &mut Value,
    legacy: &Value,
    channel: &str,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    let enabled_path = format!("{channel}.enabled");
    if bool_path_is_false_or_missing(canonical, &enabled_path)?
        && let Some(Value::Bool(true)) = get_value_at_path(legacy, &enabled_path)?
    {
        set_value_at_path(canonical, &enabled_path, Value::Bool(true))?;
        changed_paths.push(enabled_path);
    }

    Ok(())
}

fn promote_legacy_channel_scope(
    canonical: &mut Value,
    legacy: &Value,
    channel: &str,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    let chat_scope_path = format!("{channel}.chat_scope");
    let canonical_chat_scope = get_string_at_path(canonical, &chat_scope_path)?.unwrap_or_default();

    if !canonical_chat_scope_looks_seeded(&canonical_chat_scope) {
        return Ok(());
    }

    let legacy_chat_scope = match get_value_at_path(legacy, &chat_scope_path)? {
        Some(value) => Some(value.clone()),
        None => match get_value_at_path(legacy, &format!("{channel}.dm_only"))? {
            Some(Value::Bool(false)) => Some(Value::String("ALL".to_string())),
            Some(Value::Bool(true)) => Some(Value::String("DM_ONLY".to_string())),
            _ => None,
        },
    };

    if let Some(legacy_chat_scope) = legacy_chat_scope {
        set_value_at_path(canonical, &chat_scope_path, legacy_chat_scope)?;
        changed_paths.push(chat_scope_path);
    }

    Ok(())
}

fn promote_legacy_channel_credentials(
    canonical: &mut Value,
    legacy: &Value,
    channel: &str,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    match channel {
        "feishu" => {
            for field in ["app_id", "app_secret"] {
                promote_legacy_string_path(
                    canonical,
                    legacy,
                    &format!("{channel}.{field}"),
                    changed_paths,
                )?;
            }
        }
        "telegram" | "discord" => {
            promote_legacy_string_path(
                canonical,
                legacy,
                &format!("{channel}.bot_token"),
                changed_paths,
            )?;
        }
        _ => {}
    }

    Ok(())
}

fn promote_legacy_search_settings(
    canonical: &mut Value,
    legacy: &Value,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    if sequence_path_is_empty(canonical, "search.api_keys")?
        && let Some(legacy_search_keys) = get_value_at_path(legacy, "search.api_keys")?
    {
        set_value_at_path(canonical, "search.api_keys", legacy_search_keys.clone())?;
        changed_paths.push("search.api_keys".to_string());
    }
    for path in ["search.provider", "search.search_depth", "search.topic"] {
        promote_legacy_string_path(canonical, legacy, path, changed_paths)?;
    }
    if get_value_at_path(canonical, "search.max_results")?.is_none()
        && let Some(legacy_max_results) = get_value_at_path(legacy, "search.max_results")?
    {
        set_value_at_path(canonical, "search.max_results", legacy_max_results.clone())?;
        changed_paths.push("search.max_results".to_string());
    }

    Ok(())
}

fn promote_legacy_fmp_settings(
    canonical: &mut Value,
    legacy: &Value,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    promote_legacy_string_path(canonical, legacy, "fmp.api_key", changed_paths)?;
    if sequence_path_is_empty(canonical, "fmp.api_keys")?
        && let Some(legacy_fmp_api_keys) = get_value_at_path(legacy, "fmp.api_keys")?
    {
        set_value_at_path(canonical, "fmp.api_keys", legacy_fmp_api_keys.clone())?;
        changed_paths.push("fmp.api_keys".to_string());
    }
    promote_legacy_string_path(canonical, legacy, "fmp.base_url", changed_paths)?;
    if get_value_at_path(canonical, "fmp.timeout")?.is_none()
        && let Some(Value::Number(_)) = get_value_at_path(legacy, "fmp.timeout")?
        && let Some(legacy_fmp_timeout) = get_value_at_path(legacy, "fmp.timeout")?
    {
        set_value_at_path(canonical, "fmp.timeout", legacy_fmp_timeout.clone())?;
        changed_paths.push("fmp.timeout".to_string());
    }

    Ok(())
}

fn promote_legacy_string_path(
    canonical: &mut Value,
    legacy: &Value,
    path: &str,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    if string_path_is_blank(canonical, path)?
        && let Some(legacy_value) = get_value_at_path(legacy, path)?
    {
        set_value_at_path(canonical, path, legacy_value.clone())?;
        changed_paths.push(path.to_string());
    }

    Ok(())
}

fn promote_legacy_runner(
    canonical: &mut Value,
    legacy: &Value,
    migrated_opencode: bool,
    changed_paths: &mut Vec<String>,
) -> crate::HoneResult<()> {
    let canonical_runner = get_string_at_path(canonical, "agent.runner")?.unwrap_or_default();
    let legacy_runner = get_string_at_path(legacy, "agent.runner")?.unwrap_or_default();
    let should_promote_runner = !legacy_runner.is_empty()
        && canonical_runner != legacy_runner
        && canonical_runner_looks_seeded(&canonical_runner)
        && ((legacy_runner == "opencode_acp" && migrated_opencode) || canonical_runner.is_empty());

    if should_promote_runner {
        set_value_at_path(
            canonical,
            "agent.runner",
            Value::String(legacy_runner.clone()),
        )?;
        changed_paths.push("agent.runner".to_string());
    }

    Ok(())
}

/// 从 legacy runtime config 中补迁仍未进入 canonical config 的关键用户设置字段。
///
/// 迁移策略是保守的：
/// - 只有当 canonical 对应字段仍为空或种子态时，才会提升 legacy 值
/// - 一旦 canonical 已经显式配置，后续启动不会再被 legacy 覆盖
pub fn promote_legacy_runtime_agent_settings(
    canonical_config_path: &Path,
    legacy_runtime_config_path: &Path,
) -> crate::HoneResult<Vec<String>> {
    if !canonical_config_path.exists() || !legacy_runtime_config_path.exists() {
        return Ok(Vec::new());
    }

    let mut canonical = read_yaml_value_or_empty_mapping(canonical_config_path)?;
    let legacy = read_yaml_value(legacy_runtime_config_path)?;
    if legacy.is_null() {
        return Ok(Vec::new());
    }

    let mut changed_paths = Vec::new();
    let migrated_opencode = promote_legacy_opencode(&mut canonical, &legacy, &mut changed_paths)?;
    promote_legacy_llm_settings(&mut canonical, &legacy, &mut changed_paths)?;
    promote_legacy_channel_settings(&mut canonical, &legacy, &mut changed_paths)?;
    promote_legacy_search_settings(&mut canonical, &legacy, &mut changed_paths)?;
    promote_legacy_fmp_settings(&mut canonical, &legacy, &mut changed_paths)?;
    promote_legacy_runner(
        &mut canonical,
        &legacy,
        migrated_opencode,
        &mut changed_paths,
    )?;

    if changed_paths.is_empty() {
        return Ok(changed_paths);
    }

    write_yaml_value(canonical_config_path, &canonical, "canonical")?;
    Ok(changed_paths)
}

/// Normalize long-lived rollout settings that must not be inherited from old seeded configs.
///
/// Older desktop installs can carry an explicit
/// `storage.session_sqlite_shadow_write_enabled: false` in their canonical config because that
/// value was copied from an old generated runtime snapshot. When `session_runtime_backend` is
/// still `json`, JSON session files remain the source of truth, but every runtime must dual-write
/// the SQLite mirror so recovery, listing, and bug triage do not see stale session state.
pub fn normalize_runtime_storage_rollout_settings(
    canonical_config_path: &Path,
) -> crate::HoneResult<Vec<String>> {
    if !canonical_config_path.exists() {
        return Ok(Vec::new());
    }

    let mut canonical = read_yaml_value_or_empty_mapping(canonical_config_path)?;

    let mut changed_paths = Vec::new();
    if !matches!(
        get_value_at_path(&canonical, "storage.session_sqlite_shadow_write_enabled")?,
        Some(Value::Bool(true))
    ) {
        set_value_at_path(
            &mut canonical,
            "storage.session_sqlite_shadow_write_enabled",
            Value::Bool(true),
        )?;
        changed_paths.push("storage.session_sqlite_shadow_write_enabled".to_string());
    }

    if changed_paths.is_empty() {
        return Ok(changed_paths);
    }

    write_yaml_value(canonical_config_path, &canonical, "canonical")?;
    Ok(changed_paths)
}

pub fn generate_effective_config(
    canonical_config_path: &Path,
    effective_config_path: &Path,
) -> crate::HoneResult<String> {
    let value = read_yaml_value(canonical_config_path)?;
    let mut config = HoneConfig::from_merged_value(value)?;
    let runtime_dir = effective_config_path.parent().ok_or_else(|| {
        crate::HoneError::Config(format!(
            "effective 配置路径缺少 runtime 父目录: {}",
            effective_config_path.display()
        ))
    })?;
    let data_dir = runtime_dir.parent().ok_or_else(|| {
        crate::HoneError::Config(format!(
            "effective 配置路径缺少 data 父目录: {}",
            effective_config_path.display()
        ))
    })?;
    config.storage.apply_data_root(data_dir);
    let effective_value = serde_yaml::to_value(&config)
        .map_err(|e| crate::HoneError::Config(format!("effective 配置序列化失败: {e}")))?;
    write_yaml_value(effective_config_path, &effective_value, "effective")?;
    copy_relative_system_prompt_asset(canonical_config_path, effective_config_path)?;
    yaml_revision(&effective_value)
}

pub(super) fn apply_system_prompt_path(
    config: &mut HoneConfig,
    config_path: &Path,
) -> Result<(), String> {
    let prompt_path = config.agent.system_prompt_path.trim();
    if prompt_path.is_empty() {
        return Ok(());
    }

    let resolved = if Path::new(prompt_path).is_absolute() {
        PathBuf::from(prompt_path)
    } else {
        let base_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
        base_dir.join(prompt_path)
    };

    let content = std::fs::read_to_string(&resolved)
        .map_err(|e| format!("无法读取 system_prompt_path ({})：{e}", resolved.display()))?;
    config.agent.system_prompt = content;
    Ok(())
}
