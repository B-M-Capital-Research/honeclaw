//! YAML 读写 / overlay 合并 / path 解析 / 最小补丁等底层工具。

use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConfigPathSegment {
    Key(String),
    Index(usize),
}

/// 计算与给定配置文件同目录的覆盖层路径。
pub fn runtime_overlay_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let overlay_name = match (
        path.file_stem().and_then(|s| s.to_str()),
        path.extension().and_then(|s| s.to_str()),
    ) {
        (Some(stem), Some(ext)) => format!("{stem}.overrides.{ext}"),
        (Some(stem), None) => format!("{stem}.overrides"),
        _ => format!(
            "{}.overrides",
            path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("config")
        ),
    };
    parent.join(overlay_name)
}

/// 读取 YAML 到通用 `Value`，供合并和补丁生成使用。
pub fn read_yaml_value(path: impl AsRef<Path>) -> crate::HoneResult<Value> {
    let content = std::fs::read_to_string(path.as_ref())
        .map_err(|e| crate::HoneError::Config(format!("无法读取配置文件: {e}")))?;
    if content.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_yaml::from_str(&content)
        .map_err(|e| crate::HoneError::Config(format!("配置文件解析失败: {e}")))
}

/// 读取基础配置并叠加同目录 overlay，返回“当前有效 YAML 值”。
pub fn read_merged_yaml_value(path: impl AsRef<Path>) -> crate::HoneResult<Value> {
    let path = path.as_ref();
    let mut value = read_yaml_value(path)?;
    let overlay_path = runtime_overlay_path(path);
    if overlay_path.exists() {
        let overlay = read_yaml_value(&overlay_path)?;
        if !overlay.is_null() {
            merge_yaml_value(&mut value, overlay);
        }
    }
    Ok(value)
}

/// 将覆盖层递归合并到基础 YAML 上。
///
/// 规则：
/// - mapping 递归合并
/// - sequence / 标量 / null 直接替换
pub fn merge_yaml_value(base: &mut Value, overlay: Value) {
    match overlay {
        Value::Mapping(overlay_map) => {
            if let Value::Mapping(base_map) = base {
                for (key, overlay_value) in overlay_map {
                    match base_map.get_mut(&key) {
                        Some(base_value) => merge_yaml_value(base_value, overlay_value),
                        None => {
                            base_map.insert(key, overlay_value);
                        }
                    }
                }
            } else {
                *base = Value::Mapping(overlay_map);
            }
        }
        overlay => {
            *base = overlay;
        }
    }
}

/// 计算 `current` 相对 `base` 的最小覆盖层补丁。
///
/// - mapping 只保留有差异的子树
/// - sequence / 标量 / null 发生变化时整段保留
pub fn diff_yaml_value(base: &Value, current: &Value) -> Option<Value> {
    match (base, current) {
        (Value::Mapping(base_map), Value::Mapping(current_map)) => {
            let mut patch = Mapping::new();
            for (key, current_value) in current_map {
                match base_map.get(key) {
                    Some(base_value) => {
                        if let Some(child_patch) = diff_yaml_value(base_value, current_value) {
                            patch.insert(key.clone(), child_patch);
                        }
                    }
                    None => {
                        patch.insert(key.clone(), current_value.clone());
                    }
                }
            }
            if patch.is_empty() {
                None
            } else {
                Some(Value::Mapping(patch))
            }
        }
        (Value::Sequence(base_seq), Value::Sequence(current_seq)) => {
            if base_seq == current_seq {
                None
            } else {
                Some(current.clone())
            }
        }
        _ => {
            if base == current {
                None
            } else {
                Some(current.clone())
            }
        }
    }
}

pub(super) fn parse_config_path(path: &str) -> crate::HoneResult<Vec<ConfigPathSegment>> {
    let raw = path.trim();
    if raw.is_empty() {
        return Err(crate::HoneError::Config("配置路径不能为空".to_string()));
    }

    let mut chars = raw.chars().peekable();
    let mut current = String::new();
    let mut segments = Vec::new();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if current.is_empty() {
                    return Err(crate::HoneError::Config(format!(
                        "非法配置路径（连续的 '.'）：{raw}"
                    )));
                }
                segments.push(ConfigPathSegment::Key(std::mem::take(&mut current)));
            }
            '[' => {
                if !current.is_empty() {
                    segments.push(ConfigPathSegment::Key(std::mem::take(&mut current)));
                }
                let mut index_buf = String::new();
                let mut closed = false;
                for next in chars.by_ref() {
                    if next == ']' {
                        closed = true;
                        break;
                    }
                    index_buf.push(next);
                }
                if !closed {
                    return Err(crate::HoneError::Config(format!(
                        "非法配置路径（缺少 ']'）：{raw}"
                    )));
                }
                let index = index_buf.parse::<usize>().map_err(|_| {
                    crate::HoneError::Config(format!("非法数组下标 '{index_buf}'：{raw}"))
                })?;
                segments.push(ConfigPathSegment::Index(index));
            }
            ']' => {
                return Err(crate::HoneError::Config(format!(
                    "非法配置路径（多余的 ']'）：{raw}"
                )));
            }
            other => current.push(other),
        }
    }

    if !current.is_empty() {
        segments.push(ConfigPathSegment::Key(current));
    }

    if segments.is_empty() {
        return Err(crate::HoneError::Config(format!("非法配置路径：{raw}")));
    }

    Ok(segments)
}

pub(super) fn get_value_at_segments<'a>(
    current: &'a Value,
    segments: &[ConfigPathSegment],
) -> crate::HoneResult<Option<&'a Value>> {
    let mut node = current;
    for segment in segments {
        match segment {
            ConfigPathSegment::Key(key) => match node {
                Value::Mapping(map) => match map.get(Value::String(key.clone())) {
                    Some(value) => node = value,
                    None => return Ok(None),
                },
                other => {
                    return Err(crate::HoneError::Config(format!(
                        "配置路径期望对象节点，但命中的是 {}",
                        yaml_kind(other)
                    )));
                }
            },
            ConfigPathSegment::Index(index) => match node {
                Value::Sequence(items) => match items.get(*index) {
                    Some(value) => node = value,
                    None => return Ok(None),
                },
                other => {
                    return Err(crate::HoneError::Config(format!(
                        "配置路径期望数组节点，但命中的是 {}",
                        yaml_kind(other)
                    )));
                }
            },
        }
    }

    Ok(Some(node))
}

pub(super) fn set_value_at_segments(
    current: &mut Value,
    segments: &[ConfigPathSegment],
    value: Value,
) -> crate::HoneResult<()> {
    if segments.is_empty() {
        *current = value;
        return Ok(());
    }

    match &segments[0] {
        ConfigPathSegment::Key(key) => {
            if !matches!(current, Value::Mapping(_)) {
                *current = Value::Mapping(Mapping::new());
            }
            let Value::Mapping(map) = current else {
                unreachable!();
            };
            let entry = map.entry(Value::String(key.clone())).or_insert(Value::Null);
            set_value_at_segments(entry, &segments[1..], value)
        }
        ConfigPathSegment::Index(index) => {
            if !matches!(current, Value::Sequence(_)) {
                *current = Value::Sequence(Vec::new());
            }
            let Value::Sequence(items) = current else {
                unreachable!();
            };
            while items.len() <= *index {
                items.push(Value::Null);
            }
            set_value_at_segments(&mut items[*index], &segments[1..], value)
        }
    }
}

pub(super) fn unset_value_at_segments(
    current: &mut Value,
    segments: &[ConfigPathSegment],
) -> crate::HoneResult<bool> {
    if segments.is_empty() {
        return Err(crate::HoneError::Config("unset 路径不能为空".to_string()));
    }

    match &segments[0] {
        ConfigPathSegment::Key(key) => match current {
            Value::Mapping(map) => {
                if segments.len() == 1 {
                    Ok(map.remove(Value::String(key.clone())).is_some())
                } else if let Some(child) = map.get_mut(Value::String(key.clone())) {
                    unset_value_at_segments(child, &segments[1..])
                } else {
                    Ok(false)
                }
            }
            other => Err(crate::HoneError::Config(format!(
                "配置路径期望对象节点，但命中的是 {}",
                yaml_kind(other)
            ))),
        },
        ConfigPathSegment::Index(index) => match current {
            Value::Sequence(items) => {
                if *index >= items.len() {
                    return Ok(false);
                }
                if segments.len() == 1 {
                    items.remove(*index);
                    Ok(true)
                } else {
                    unset_value_at_segments(&mut items[*index], &segments[1..])
                }
            }
            other => Err(crate::HoneError::Config(format!(
                "配置路径期望数组节点，但命中的是 {}",
                yaml_kind(other)
            ))),
        },
    }
}

fn yaml_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Sequence(_) => "sequence",
        Value::Mapping(_) => "mapping",
        Value::Tagged(_) => "tagged",
    }
}

pub(super) fn atomic_write_yaml(path: &Path, yaml: &str) -> crate::HoneResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let parent = path.parent().ok_or_else(|| {
        crate::HoneError::Config(format!("覆盖层路径缺少父目录: {}", path.display()))
    })?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| crate::HoneError::Config(format!("获取时间戳失败: {e}")))?
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("config");
    let tmp_path = parent.join(format!(".{file_name}.{stamp}.tmp"));

    fs::write(&tmp_path, yaml)?;
    match fs::rename(&tmp_path, path) {
        Ok(()) => Ok(()),
        Err(first_err) => {
            let _ = fs::remove_file(path);
            match fs::rename(&tmp_path, path) {
                Ok(()) => Ok(()),
                Err(second_err) => {
                    let _ = fs::remove_file(&tmp_path);
                    Err(crate::HoneError::Config(format!(
                        "无法写入覆盖层 {}: {second_err}（初次重命名错误: {first_err}）",
                        path.display()
                    )))
                }
            }
        }
    }
}

pub fn write_overlay_patch(path: &Path, patch: Option<Value>) -> crate::HoneResult<()> {
    match patch {
        None => {
            if path.exists() {
                fs::remove_file(path)?;
            }
            Ok(())
        }
        Some(Value::Mapping(map)) if map.is_empty() => {
            if path.exists() {
                fs::remove_file(path)?;
            }
            Ok(())
        }
        Some(value) => {
            let yaml = serde_yaml::to_string(&value)
                .map_err(|e| crate::HoneError::Config(format!("覆盖层序列化失败: {e}")))?;
            atomic_write_yaml(path, &yaml)
        }
    }
}

pub(super) fn get_value_at_path<'a>(
    current: &'a Value,
    path: &str,
) -> crate::HoneResult<Option<&'a Value>> {
    let segments = parse_config_path(path)?;
    get_value_at_segments(current, &segments)
}

pub(super) fn get_string_at_path(current: &Value, path: &str) -> crate::HoneResult<Option<String>> {
    Ok(get_value_at_path(current, path)?
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string()))
}

pub(super) fn string_path_is_blank(current: &Value, path: &str) -> crate::HoneResult<bool> {
    Ok(get_string_at_path(current, path)?
        .map(|value| value.is_empty())
        .unwrap_or(true))
}

pub(super) fn bool_path_is_false_or_missing(
    current: &Value,
    path: &str,
) -> crate::HoneResult<bool> {
    Ok(get_value_at_path(current, path)?
        .and_then(|value| value.as_bool())
        .map(|value| !value)
        .unwrap_or(true))
}

pub(super) fn sequence_path_is_empty(current: &Value, path: &str) -> crate::HoneResult<bool> {
    Ok(match get_value_at_path(current, path)? {
        Some(Value::Sequence(items)) => items
            .iter()
            .all(|item| item.as_str().map(|s| s.trim().is_empty()).unwrap_or(true)),
        Some(Value::Null) | None => true,
        _ => false,
    })
}

pub(super) fn set_value_at_path(
    current: &mut Value,
    path: &str,
    value: Value,
) -> crate::HoneResult<()> {
    let segments = parse_config_path(path)?;
    set_value_at_segments(current, &segments, value)
}

pub(super) fn yaml_revision(value: &Value) -> crate::HoneResult<String> {
    use std::hash::{Hash, Hasher};

    let rendered = serde_yaml::to_string(value)
        .map_err(|e| crate::HoneError::Config(format!("配置序列化失败: {e}")))?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    rendered.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}
