//! Canonical 配置 seed / 升级 / effective 产出。

use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};

use super::HoneConfig;
use super::yaml::{
    atomic_write_yaml, bool_path_is_false_or_missing, get_string_at_path, get_value_at_path,
    read_yaml_value, sequence_path_is_empty, set_value_at_path, string_path_is_blank,
    yaml_revision,
};

pub fn effective_config_path(runtime_dir: impl AsRef<Path>) -> PathBuf {
    runtime_dir.as_ref().join("effective-config.yaml")
}

pub fn canonical_config_candidate() -> PathBuf {
    PathBuf::from("config.yaml")
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
        fs::create_dir_all(parent)?;
    }
    if !dest.exists() {
        fs::copy(&source, &dest)?;
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
    fs::create_dir_all(parent)?;
    fs::copy(source_config_path, canonical_config_path)?;
    copy_relative_system_prompt_asset(source_config_path, canonical_config_path)?;
    Ok(())
}

fn canonical_runner_looks_seeded(runner: &str) -> bool {
    matches!(runner.trim(), "" | "function_calling" | "codex_cli")
}

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

    let mut canonical = read_yaml_value(canonical_config_path)?;
    if canonical.is_null() {
        canonical = Value::Mapping(Mapping::new());
    }
    let legacy = read_yaml_value(legacy_runtime_config_path)?;
    if legacy.is_null() {
        return Ok(Vec::new());
    }

    let mut changed_paths = Vec::new();
    let mut migrated_multi_agent = false;
    let mut migrated_opencode = false;

    if string_path_is_blank(&canonical, "agent.multi_agent.search.api_key")?
        && string_path_is_blank(&canonical, "agent.multi_agent.answer.api_key")?
    {
        if let Some(legacy_multi_agent) = get_value_at_path(&legacy, "agent.multi_agent")? {
            set_value_at_path(
                &mut canonical,
                "agent.multi_agent",
                legacy_multi_agent.clone(),
            )?;
            changed_paths.push("agent.multi_agent".to_string());
            migrated_multi_agent = true;
        }
    }

    if let Some(legacy_opencode) = get_value_at_path(&legacy, "agent.opencode")? {
        if canonical_opencode_block_is_blank(&canonical)? {
            set_value_at_path(&mut canonical, "agent.opencode", legacy_opencode.clone())?;
            changed_paths.push("agent.opencode".to_string());
            migrated_opencode = true;
        } else {
            for path in [
                "agent.opencode.api_base_url",
                "agent.opencode.model",
                "agent.opencode.variant",
            ] {
                if string_path_is_blank(&canonical, path)?
                    && let Some(legacy_value) = get_value_at_path(&legacy, path)?
                    && legacy_value
                        .as_str()
                        .map(|value| !value.trim().is_empty())
                        .unwrap_or(false)
                {
                    set_value_at_path(&mut canonical, path, legacy_value.clone())?;
                    changed_paths.push(path.to_string());
                    migrated_opencode = true;
                }
            }
        }
    }

    if string_path_is_blank(&canonical, "llm.auxiliary.api_key")?
        && let Some(legacy_auxiliary) = get_value_at_path(&legacy, "llm.auxiliary")?
    {
        set_value_at_path(&mut canonical, "llm.auxiliary", legacy_auxiliary.clone())?;
        changed_paths.push("llm.auxiliary".to_string());
    }

    if string_path_is_blank(&canonical, "llm.openrouter.api_key")?
        && let Some(legacy_openrouter_key) = get_value_at_path(&legacy, "llm.openrouter.api_key")?
        && legacy_openrouter_key
            .as_str()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    {
        set_value_at_path(
            &mut canonical,
            "llm.openrouter.api_key",
            legacy_openrouter_key.clone(),
        )?;
        changed_paths.push("llm.openrouter.api_key".to_string());
    }

    if sequence_path_is_empty(&canonical, "llm.openrouter.api_keys")?
        && let Some(legacy_openrouter_keys) = get_value_at_path(&legacy, "llm.openrouter.api_keys")?
    {
        set_value_at_path(
            &mut canonical,
            "llm.openrouter.api_keys",
            legacy_openrouter_keys.clone(),
        )?;
        changed_paths.push("llm.openrouter.api_keys".to_string());
    }

    for channel in ["feishu", "telegram", "discord", "imessage"] {
        let enabled_path = format!("{channel}.enabled");
        if bool_path_is_false_or_missing(&canonical, &enabled_path)?
            && let Some(Value::Bool(true)) = get_value_at_path(&legacy, &enabled_path)?
        {
            set_value_at_path(&mut canonical, &enabled_path, Value::Bool(true))?;
            changed_paths.push(enabled_path);
        }

        let chat_scope_path = format!("{channel}.chat_scope");
        let canonical_chat_scope =
            get_string_at_path(&canonical, &chat_scope_path)?.unwrap_or_default();
        if canonical_chat_scope_looks_seeded(&canonical_chat_scope) {
            let legacy_chat_scope = match get_value_at_path(&legacy, &chat_scope_path)? {
                Some(value) => Some(value.clone()),
                None => match get_value_at_path(&legacy, &format!("{channel}.dm_only"))? {
                    Some(Value::Bool(false)) => Some(Value::String("ALL".to_string())),
                    Some(Value::Bool(true)) => Some(Value::String("DM_ONLY".to_string())),
                    _ => None,
                },
            };
            if let Some(legacy_chat_scope) = legacy_chat_scope {
                set_value_at_path(&mut canonical, &chat_scope_path, legacy_chat_scope)?;
                changed_paths.push(chat_scope_path);
            }
        }

        match channel {
            "feishu" => {
                for field in ["app_id", "app_secret"] {
                    let field_path = format!("{channel}.{field}");
                    if string_path_is_blank(&canonical, &field_path)?
                        && let Some(legacy_value) = get_value_at_path(&legacy, &field_path)?
                    {
                        set_value_at_path(&mut canonical, &field_path, legacy_value.clone())?;
                        changed_paths.push(field_path);
                    }
                }
            }
            "telegram" | "discord" => {
                let token_path = format!("{channel}.bot_token");
                if string_path_is_blank(&canonical, &token_path)?
                    && let Some(legacy_token) = get_value_at_path(&legacy, &token_path)?
                {
                    set_value_at_path(&mut canonical, &token_path, legacy_token.clone())?;
                    changed_paths.push(token_path);
                }
            }
            _ => {}
        }
    }

    if sequence_path_is_empty(&canonical, "search.api_keys")?
        && let Some(legacy_search_keys) = get_value_at_path(&legacy, "search.api_keys")?
    {
        set_value_at_path(
            &mut canonical,
            "search.api_keys",
            legacy_search_keys.clone(),
        )?;
        changed_paths.push("search.api_keys".to_string());
    }
    for path in ["search.provider", "search.search_depth", "search.topic"] {
        if string_path_is_blank(&canonical, path)?
            && let Some(legacy_value) = get_value_at_path(&legacy, path)?
        {
            set_value_at_path(&mut canonical, path, legacy_value.clone())?;
            changed_paths.push(path.to_string());
        }
    }
    if get_value_at_path(&canonical, "search.max_results")?.is_none()
        && let Some(legacy_max_results) = get_value_at_path(&legacy, "search.max_results")?
    {
        set_value_at_path(
            &mut canonical,
            "search.max_results",
            legacy_max_results.clone(),
        )?;
        changed_paths.push("search.max_results".to_string());
    }

    if string_path_is_blank(&canonical, "fmp.api_key")?
        && let Some(legacy_fmp_api_key) = get_value_at_path(&legacy, "fmp.api_key")?
    {
        set_value_at_path(&mut canonical, "fmp.api_key", legacy_fmp_api_key.clone())?;
        changed_paths.push("fmp.api_key".to_string());
    }
    if sequence_path_is_empty(&canonical, "fmp.api_keys")?
        && let Some(legacy_fmp_api_keys) = get_value_at_path(&legacy, "fmp.api_keys")?
    {
        set_value_at_path(&mut canonical, "fmp.api_keys", legacy_fmp_api_keys.clone())?;
        changed_paths.push("fmp.api_keys".to_string());
    }
    if string_path_is_blank(&canonical, "fmp.base_url")?
        && let Some(legacy_fmp_base_url) = get_value_at_path(&legacy, "fmp.base_url")?
    {
        set_value_at_path(&mut canonical, "fmp.base_url", legacy_fmp_base_url.clone())?;
        changed_paths.push("fmp.base_url".to_string());
    }
    if let Some(Value::Number(_)) = get_value_at_path(&legacy, "fmp.timeout")?
        && get_value_at_path(&canonical, "fmp.timeout")?.is_none()
    {
        if let Some(legacy_fmp_timeout) = get_value_at_path(&legacy, "fmp.timeout")? {
            set_value_at_path(&mut canonical, "fmp.timeout", legacy_fmp_timeout.clone())?;
            changed_paths.push("fmp.timeout".to_string());
        }
    }

    let canonical_runner = get_string_at_path(&canonical, "agent.runner")?.unwrap_or_default();
    let legacy_runner = get_string_at_path(&legacy, "agent.runner")?.unwrap_or_default();
    let should_promote_runner = !legacy_runner.is_empty()
        && canonical_runner != legacy_runner
        && canonical_runner_looks_seeded(&canonical_runner)
        && ((legacy_runner == "multi-agent" && migrated_multi_agent)
            || (legacy_runner == "opencode_acp" && migrated_opencode)
            || canonical_runner.is_empty());

    if should_promote_runner {
        set_value_at_path(
            &mut canonical,
            "agent.runner",
            Value::String(legacy_runner.clone()),
        )?;
        changed_paths.push("agent.runner".to_string());
    }

    if changed_paths.is_empty() {
        return Ok(changed_paths);
    }

    let yaml = serde_yaml::to_string(&canonical)
        .map_err(|e| crate::HoneError::Config(format!("canonical 配置序列化失败: {e}")))?;
    atomic_write_yaml(canonical_config_path, &yaml)?;
    Ok(changed_paths)
}

pub fn generate_effective_config(
    canonical_config_path: &Path,
    effective_config_path: &Path,
) -> crate::HoneResult<String> {
    let value = read_yaml_value(canonical_config_path)?;
    HoneConfig::from_merged_value(value.clone())?;
    let yaml = serde_yaml::to_string(&value)
        .map_err(|e| crate::HoneError::Config(format!("effective 配置序列化失败: {e}")))?;
    atomic_write_yaml(effective_config_path, &yaml)?;
    copy_relative_system_prompt_asset(canonical_config_path, effective_config_path)?;
    yaml_revision(&value)
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
