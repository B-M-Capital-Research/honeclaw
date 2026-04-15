//! 配置加载与验证
//!
//! 从 config.yaml 加载配置，使用 serde 反序列化。

use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

pub mod agent;
pub mod channels;
pub mod server;

pub use agent::{
    AdminConfig, AgentConfig, AuxiliaryLlmConfig, CodexAcpConfig, GeminiAcpConfig, KimiConfig,
    LlmConfig, MultiAgentAnswerConfig, MultiAgentConfig, MultiAgentSearchConfig, OpenRouterConfig,
    OpencodeAcpConfig,
};
pub use channels::{
    ChatScope, DiscordConfig, DiscordGroupReplyConfig, DiscordWatchConfig, FeishuConfig,
    GroupContextConfig, IMessageConfig, TelegramConfig, XConfig, XOAuth1Config,
};
pub use server::{
    FmpConfig, LoggingConfig, NanoBananaConfig, SearchConfig, SecurityConfig, StorageConfig,
    ToolGuardConfig, WebConfig,
};

/// 顶层配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoneConfig {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub imessage: IMessageConfig,
    #[serde(default)]
    pub feishu: FeishuConfig,
    #[serde(default)]
    pub telegram: TelegramConfig,
    #[serde(default)]
    pub discord: DiscordConfig,
    #[serde(default)]
    pub group_context: GroupContextConfig,
    #[serde(default)]
    pub x: XConfig,
    #[serde(default)]
    pub nano_banana: NanoBananaConfig,
    #[serde(default)]
    pub fmp: FmpConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    /// Agent system prompt 模板
    #[serde(default)]
    pub agent: AgentConfig,
    /// 管理员配置
    #[serde(default)]
    pub admins: AdminConfig,
    /// Web 控制台配置
    #[serde(default)]
    pub web: WebConfig,
    /// 安全策略配置
    #[serde(default)]
    pub security: SecurityConfig,
    /// 额外的未知字段
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

impl HoneConfig {
    /// 从已经完成 overlay 合并的 YAML 值加载配置。
    ///
    /// 与 `from_file()` 不同，这里不会尝试解析并内联 `system_prompt_path`
    /// 指向的文件内容，适合配置编辑流程里的“纯配置校验”。
    pub fn from_merged_value(value: Value) -> crate::HoneResult<Self> {
        let config: Self = serde_yaml::from_value(value)
            .map_err(|e| crate::HoneError::Config(format!("配置文件解析失败: {e}")))?;
        let config = config;
        config.validate()?;
        Ok(config)
    }

    /// 从 YAML 文件加载配置
    pub fn from_file(path: impl AsRef<Path>) -> crate::HoneResult<Self> {
        let path = path.as_ref();
        let value = read_yaml_value(path)?;
        let mut config = Self::from_merged_value(value)?;
        if let Err(err) = apply_system_prompt_path(&mut config, path) {
            return Err(crate::HoneError::Config(err));
        }
        Ok(config)
    }

    pub fn validate(&self) -> crate::HoneResult<()> {
        validate_channel_chat_scope("feishu", self.feishu.chat_scope)?;
        validate_channel_chat_scope("telegram", self.telegram.chat_scope)?;
        validate_channel_chat_scope("discord", self.discord.chat_scope)?;
        Ok(())
    }

    pub fn apply_runtime_overrides(
        &mut self,
        data_dir: Option<&Path>,
        skills_dir: Option<&Path>,
        config_path: Option<&Path>,
    ) {
        if let Some(data_dir) = data_dir {
            self.storage.apply_data_root(data_dir);
        }
        if let Some(skills_dir) = skills_dir {
            self.extra.insert(
                "skills_dir".to_string(),
                serde_yaml::Value::String(skills_dir.to_string_lossy().to_string()),
            );
        }
        if let Some(config_path) = config_path {
            self.extra.insert(
                "config_path".to_string(),
                serde_yaml::Value::String(config_path.to_string_lossy().to_string()),
            );
        }
    }

    pub fn ensure_runtime_dirs(&self) {
        self.storage.ensure_runtime_dirs();
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConfigPathSegment {
    Key(String),
    Index(usize),
}

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

pub fn effective_config_path(runtime_dir: impl AsRef<Path>) -> PathBuf {
    runtime_dir.as_ref().join("effective-config.yaml")
}

pub fn canonical_config_candidate() -> PathBuf {
    PathBuf::from("config.yaml")
}

fn parse_config_path(path: &str) -> crate::HoneResult<Vec<ConfigPathSegment>> {
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

fn get_value_at_segments<'a>(
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

fn set_value_at_segments(
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

fn unset_value_at_segments(
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

fn atomic_write_yaml(path: &Path, yaml: &str) -> crate::HoneResult<()> {
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

fn get_value_at_path<'a>(current: &'a Value, path: &str) -> crate::HoneResult<Option<&'a Value>> {
    let segments = parse_config_path(path)?;
    get_value_at_segments(current, &segments)
}

fn get_string_at_path(current: &Value, path: &str) -> crate::HoneResult<Option<String>> {
    Ok(get_value_at_path(current, path)?
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string()))
}

fn string_path_is_blank(current: &Value, path: &str) -> crate::HoneResult<bool> {
    Ok(get_string_at_path(current, path)?
        .map(|value| value.is_empty())
        .unwrap_or(true))
}

fn bool_path_is_false_or_missing(current: &Value, path: &str) -> crate::HoneResult<bool> {
    Ok(get_value_at_path(current, path)?
        .and_then(|value| value.as_bool())
        .map(|value| !value)
        .unwrap_or(true))
}

fn sequence_path_is_empty(current: &Value, path: &str) -> crate::HoneResult<bool> {
    Ok(match get_value_at_path(current, path)? {
        Some(Value::Sequence(items)) => items
            .iter()
            .all(|item| item.as_str().map(|s| s.trim().is_empty()).unwrap_or(true)),
        Some(Value::Null) | None => true,
        _ => false,
    })
}

fn set_value_at_path(current: &mut Value, path: &str, value: Value) -> crate::HoneResult<()> {
    let segments = parse_config_path(path)?;
    set_value_at_segments(current, &segments, value)
}

fn canonical_runner_looks_seeded(runner: &str) -> bool {
    matches!(runner.trim(), "" | "function_calling" | "codex_cli")
}

fn canonical_chat_scope_looks_seeded(scope: &str) -> bool {
    matches!(scope.trim(), "" | "DM_ONLY")
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

    if string_path_is_blank(&canonical, "agent.opencode.api_key")?
        && let Some(legacy_opencode) = get_value_at_path(&legacy, "agent.opencode")?
    {
        set_value_at_path(&mut canonical, "agent.opencode", legacy_opencode.clone())?;
        changed_paths.push("agent.opencode".to_string());
        migrated_opencode = true;
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

fn yaml_revision(value: &Value) -> crate::HoneResult<String> {
    use std::hash::{Hash, Hasher};

    let rendered = serde_yaml::to_string(value)
        .map_err(|e| crate::HoneError::Config(format!("配置序列化失败: {e}")))?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    rendered.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
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

fn apply_system_prompt_path(config: &mut HoneConfig, config_path: &Path) -> Result<(), String> {
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

fn validate_channel_chat_scope(channel: &str, chat_scope: ChatScope) -> crate::HoneResult<()> {
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

impl Default for HoneConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            imessage: IMessageConfig::default(),
            feishu: FeishuConfig::default(),
            telegram: TelegramConfig::default(),
            discord: DiscordConfig::default(),
            group_context: GroupContextConfig::default(),
            x: XConfig::default(),
            nano_banana: NanoBananaConfig::default(),
            fmp: FmpConfig::default(),
            search: SearchConfig::default(),
            logging: LoggingConfig::default(),
            storage: StorageConfig::default(),
            agent: AgentConfig::default(),
            admins: AdminConfig::default(),
            web: WebConfig::default(),
            security: SecurityConfig::default(),
            extra: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ChatScope;

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("hone-config-{}-{}", prefix, uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_default_config() {
        let config = HoneConfig::default();
        assert_eq!(config.llm.provider, "openrouter");
        assert_eq!(config.llm.openrouter.model, "moonshotai/kimi-k2.5");
        assert_eq!(config.llm.openrouter.sub_model, "moonshotai/kimi-k2.5");
        assert_eq!(config.llm.auxiliary.api_key_env, "MINIMAX_API_KEY");
        assert!(config.llm.auxiliary.base_url.is_empty());
        assert_eq!(config.llm.openrouter.timeout, 120);
        assert_eq!(config.llm.openrouter.max_tokens, 32768);
    }

    #[test]
    fn test_deserialize_minimal_yaml() {
        let yaml = r#"
llm:
  provider: openrouter
  openrouter:
    model: "test-model"
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.llm.openrouter.model, "test-model");
        assert_eq!(config.llm.openrouter.sub_model, "moonshotai/kimi-k2.5");
        assert!(config.llm.auxiliary.model.is_empty());
        assert_eq!(config.llm.openrouter.timeout, 120); // default
    }

    #[test]
    fn test_runtime_overlay_path() {
        let path = Path::new("/tmp/config.yaml");
        let overlay = runtime_overlay_path(path);
        assert_eq!(overlay, PathBuf::from("/tmp/config.overrides.yaml"));
    }

    #[test]
    fn test_merge_yaml_value_recursively() {
        let mut base: Value = serde_yaml::from_str(
            r#"
imessage:
  enabled: false
  target_handle: ""
  poll_interval: 2
search:
  api_keys:
    - base-a
    - base-b
logging:
  file: "./data/logs/hone.log"
custom_section:
  nested:
    keep: base
"#,
        )
        .unwrap();
        let overlay: Value = serde_yaml::from_str(
            r#"
imessage:
  enabled: true
search:
  api_keys:
    - override-a
custom_section:
  nested:
    keep: overlay
new_section:
  flag: true
"#,
        )
        .unwrap();

        merge_yaml_value(&mut base, overlay);
        let config: HoneConfig = serde_yaml::from_value(base).unwrap();

        assert!(config.imessage.enabled);
        assert_eq!(config.search.api_keys, vec!["override-a".to_string()]);
        assert_eq!(config.logging.file.as_deref(), Some("./data/logs/hone.log"));
        assert_eq!(
            config
                .extra
                .get("custom_section")
                .and_then(|v| v.as_mapping())
                .and_then(|m| m.get(Value::String("nested".to_string())))
                .and_then(|v| v.as_mapping())
                .and_then(|m| m.get(Value::String("keep".to_string())))
                .and_then(|v| v.as_str()),
            Some("overlay")
        );
        assert!(config.extra.contains_key("new_section"));
    }

    #[test]
    fn test_read_merged_yaml_value_applies_runtime_overlay() {
        let dir = temp_test_dir("from-file");
        let config_path = dir.join("config.yaml");
        let overlay_path = runtime_overlay_path(&config_path);

        std::fs::write(
            &config_path,
            r#"
imessage:
  enabled: false
search:
  api_keys:
    - base-a
logging:
  file: "./data/logs/hone.log"
  udp_port: 9000
custom_section:
  nested:
    keep: base
"#,
        )
        .unwrap();
        std::fs::write(
            &overlay_path,
            r#"
imessage:
  enabled: true
search:
  api_keys:
    - override-a
    - override-b
logging:
  file: null
custom_section:
  nested:
    keep: overlay
"#,
        )
        .unwrap();

        let merged = read_merged_yaml_value(&config_path).unwrap();
        let config = HoneConfig::from_merged_value(merged).unwrap();
        assert!(config.imessage.enabled);
        assert_eq!(
            config.search.api_keys,
            vec!["override-a".to_string(), "override-b".to_string()]
        );
        assert_eq!(config.logging.file, None);
        assert_eq!(config.logging.udp_port, Some(9000));
        assert_eq!(
            config
                .extra
                .get("custom_section")
                .and_then(|v| v.as_mapping())
                .and_then(|m| m.get(Value::String("nested".to_string())))
                .and_then(|v| v.as_mapping())
                .and_then(|m| m.get(Value::String("keep".to_string())))
                .and_then(|v| v.as_str()),
            Some("overlay")
        );
    }

    #[test]
    fn test_diff_yaml_value_keeps_only_changes() {
        let base: Value = serde_yaml::from_str(
            r#"
imessage:
  enabled: false
search:
  api_keys:
    - base-a
    - base-b
logging:
  file: "./data/logs/hone.log"
"#,
        )
        .unwrap();
        let current: Value = serde_yaml::from_str(
            r#"
imessage:
  enabled: true
search:
  api_keys:
    - override-a
logging:
  file: null
"#,
        )
        .unwrap();

        let patch = diff_yaml_value(&base, &current).expect("expected a patch");
        let patch_map = patch.as_mapping().expect("patch should be a mapping");
        assert!(patch_map.contains_key(Value::String("imessage".to_string())));
        assert!(patch_map.contains_key(Value::String("search".to_string())));
        assert!(patch_map.contains_key(Value::String("logging".to_string())));
        assert_eq!(patch_map.len(), 3);

        let logging = patch_map
            .get(Value::String("logging".to_string()))
            .and_then(|v| v.as_mapping())
            .expect("logging patch");
        assert!(matches!(
            logging.get(Value::String("file".to_string())),
            Some(Value::Null)
        ));

        let imessage = patch_map
            .get(Value::String("imessage".to_string()))
            .and_then(|v| v.as_mapping())
            .expect("imessage patch");
        assert!(matches!(
            imessage.get(Value::String("enabled".to_string())),
            Some(Value::Bool(true))
        ));

        let search = patch_map
            .get(Value::String("search".to_string()))
            .and_then(|v| v.as_mapping())
            .expect("search patch");
        assert_eq!(
            search.get(Value::String("api_keys".to_string())),
            Some(&Value::Sequence(vec![Value::String(
                "override-a".to_string()
            )]))
        );
    }

    #[test]
    fn test_deserialize_agent_codex_model() {
        let yaml = r#"
agent:
  runner: codex_cli
  codex_model: "gpt-5.3-codex"
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agent.runner, "codex_cli");
        assert_eq!(config.agent.codex_model, "gpt-5.3-codex");
    }

    #[test]
    fn test_deserialize_agent_opencode_model_and_variant() {
        let yaml = r#"
agent:
  runner: opencode_acp
  opencode:
    model: "openrouter/openai/gpt-5.4"
    variant: "medium"
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agent.runner, "opencode_acp");
        assert_eq!(config.agent.opencode.model, "openrouter/openai/gpt-5.4");
        assert_eq!(config.agent.opencode.variant, "medium");
    }

    #[test]
    fn test_default_agent_opencode_inherits_local_config_when_unset() {
        let config = HoneConfig::default();
        assert!(config.agent.opencode.model.is_empty());
        assert!(config.agent.opencode.variant.is_empty());
        assert!(config.agent.opencode.api_base_url.is_empty());
        assert!(config.agent.opencode.api_key.is_empty());
        assert_eq!(
            config.agent.multi_agent.answer.api_base_url,
            "https://openrouter.ai/api/v1"
        );
    }

    #[test]
    fn test_deserialize_agent_gemini_acp() {
        let yaml = r#"
agent:
  runner: gemini_acp
  gemini_acp:
    model: "gemini-2.5-pro"
    api_key_env: "GEMINI_API_KEY"
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agent.runner, "gemini_acp");
        assert_eq!(config.agent.gemini_acp.model, "gemini-2.5-pro");
        assert_eq!(config.agent.gemini_acp.api_key_env, "GEMINI_API_KEY");
    }

    #[test]
    fn test_deserialize_agent_codex_acp_sandbox_controls() {
        let yaml = r#"
agent:
  runner: codex_acp
  codex_acp:
    model: "gpt-5.4"
    variant: "medium"
    dangerously_bypass_approvals_and_sandbox: true
    sandbox_permissions: ["disk-full-read-access"]
    extra_config_overrides: ["shell_environment_policy.inherit=all"]
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agent.runner, "codex_acp");
        assert!(
            config
                .agent
                .codex_acp
                .dangerously_bypass_approvals_and_sandbox
        );
        assert_eq!(
            config.agent.codex_acp.sandbox_permissions,
            vec!["disk-full-read-access"]
        );
        assert_eq!(
            config.agent.codex_acp.extra_config_overrides,
            vec!["shell_environment_policy.inherit=all"]
        );
    }

    #[test]
    fn test_deserialize_agent_multi_agent() {
        let yaml = r#"
agent:
  runner: multi-agent
  multi_agent:
    search:
      base_url: "https://api.minimaxi.com/v1"
      api_key: "sk-cp-test"
      model: "MiniMax-M2.7-highspeed"
      max_iterations: 8
    answer:
      api_base_url: "https://openrouter.ai/api/v1"
      api_key: "sk-or-test"
      model: "google/gemini-3.1-pro-preview"
      variant: "high"
      max_tool_calls: 1
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agent.runner, "multi-agent");
        assert_eq!(
            config.agent.multi_agent.search.base_url,
            "https://api.minimaxi.com/v1"
        );
        assert_eq!(config.agent.multi_agent.search.api_key, "sk-cp-test");
        assert_eq!(
            config.agent.multi_agent.search.model,
            "MiniMax-M2.7-highspeed"
        );
        assert_eq!(config.agent.multi_agent.search.max_iterations, 8);
        assert_eq!(
            config.agent.multi_agent.answer.api_base_url,
            "https://openrouter.ai/api/v1"
        );
        assert_eq!(config.agent.multi_agent.answer.api_key, "sk-or-test");
        assert_eq!(
            config.agent.multi_agent.answer.model,
            "google/gemini-3.1-pro-preview"
        );
        assert_eq!(config.agent.multi_agent.answer.variant, "high");
        assert_eq!(config.agent.multi_agent.answer.max_tool_calls, 1);
    }

    #[test]
    fn test_deserialize_feishu_config() {
        let yaml = r#"
feishu:
  enabled: true
  app_id: "cli_test"
  app_secret: "secret"
  allow_emails: ["alice@example.com"]
  allow_mobiles: ["+8613800138000"]
  allow_open_ids: ["ou_abc"]
  chat_scope: GROUPCHAT_ONLY
  max_message_length: 2048
  facade_url: "http://127.0.0.1:19001/rpc"
  callback_addr: "127.0.0.1:19002"
  facade_addr: "127.0.0.1:19001"
  startup_timeout_seconds: 9
admins:
  feishu_emails: ["admin@example.com"]
  feishu_mobiles: ["+8613900139000"]
  feishu_open_ids: ["ou_admin"]
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.feishu.enabled);
        assert_eq!(config.feishu.app_id, "cli_test");
        assert_eq!(config.feishu.app_secret, "secret");
        assert_eq!(config.feishu.allow_emails, vec!["alice@example.com"]);
        assert_eq!(config.feishu.allow_mobiles, vec!["+8613800138000"]);
        assert_eq!(config.feishu.allow_open_ids, vec!["ou_abc"]);
        assert_eq!(config.feishu.chat_scope, ChatScope::GroupchatOnly);
        assert_eq!(config.feishu.max_message_length, 2048);
        assert_eq!(config.feishu.facade_url, "http://127.0.0.1:19001/rpc");
        assert_eq!(config.feishu.callback_addr, "127.0.0.1:19002");
        assert_eq!(config.feishu.facade_addr, "127.0.0.1:19001");
        assert_eq!(config.feishu.startup_timeout_seconds, 9);
        assert_eq!(config.admins.feishu_emails, vec!["admin@example.com"]);
        assert_eq!(config.admins.feishu_mobiles, vec!["+8613900139000"]);
        assert_eq!(config.admins.feishu_open_ids, vec!["ou_admin"]);
    }

    #[test]
    fn test_deserialize_discord_group_reply() {
        let yaml = r#"
group_context:
  pretrigger_window_enabled: false
  pretrigger_window_max_messages: 6
  pretrigger_window_max_age_seconds: 45
discord:
  group_reply:
    enabled: true
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.group_context.pretrigger_window_enabled);
        assert_eq!(config.group_context.pretrigger_window_max_messages, 6);
        assert_eq!(config.group_context.pretrigger_window_max_age_seconds, 45);
        let gr = &config.discord.group_reply;
        assert!(gr.enabled);
    }

    #[test]
    fn test_chat_scope_defaults_to_dm_only() {
        let config = HoneConfig::default();
        assert_eq!(config.feishu.chat_scope, ChatScope::DmOnly);
        assert_eq!(config.telegram.chat_scope, ChatScope::DmOnly);
        assert_eq!(config.discord.chat_scope, ChatScope::DmOnly);
    }

    #[test]
    fn test_legacy_dm_only_false_maps_to_all() {
        let yaml = r#"
telegram:
  dm_only: false
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.telegram.chat_scope, ChatScope::All);
    }

    #[test]
    fn test_chat_scope_overrides_legacy_dm_only() {
        let yaml = r#"
discord:
  chat_scope: GROUPCHAT_ONLY
  dm_only: true
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.discord.chat_scope, ChatScope::GroupchatOnly);
    }

    #[test]
    fn test_read_config_path_value_supports_nested_mapping_and_sequence() {
        let dir = temp_test_dir("path-get");
        let config_path = dir.join("config.yaml");
        std::fs::write(
            &config_path,
            r#"
search:
  api_keys:
    - key-a
    - key-b
agent:
  runner: codex_cli
"#,
        )
        .unwrap();

        assert_eq!(
            read_config_path_value(&config_path, "agent.runner")
                .unwrap()
                .and_then(|value| value.as_str().map(ToString::to_string)),
            Some("codex_cli".to_string())
        );
        assert_eq!(
            read_config_path_value(&config_path, "search.api_keys[1]")
                .unwrap()
                .and_then(|value| value.as_str().map(ToString::to_string)),
            Some("key-b".to_string())
        );
        assert!(
            read_config_path_value(&config_path, "search.api_keys[3]")
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn test_apply_config_mutations_updates_canonical_config_directly() {
        let dir = temp_test_dir("mutations");
        let config_path = dir.join("config.yaml");
        let overlay_path = runtime_overlay_path(&config_path);
        std::fs::write(
            &config_path,
            r#"
agent:
  runner: codex_cli
search:
  api_keys:
    - key-a
"#,
        )
        .unwrap();

        apply_config_mutations(
            &config_path,
            &[
                ConfigMutation::Set {
                    path: "agent.runner".to_string(),
                    value: Value::String("opencode_acp".to_string()),
                },
                ConfigMutation::Set {
                    path: "search.api_keys[1]".to_string(),
                    value: Value::String("key-b".to_string()),
                },
            ],
        )
        .unwrap();

        let base = std::fs::read_to_string(&config_path).unwrap();
        assert!(base.contains("opencode_acp"));
        assert!(base.contains("key-b"));
        assert!(!overlay_path.exists());

        let config = HoneConfig::from_file(&config_path).unwrap();
        assert_eq!(config.agent.runner, "opencode_acp");
        assert_eq!(
            config.search.api_keys,
            vec!["key-a".to_string(), "key-b".to_string()]
        );

        apply_config_mutations(
            &config_path,
            &[ConfigMutation::Unset {
                path: "search.api_keys[0]".to_string(),
            }],
        )
        .unwrap();
        let config = HoneConfig::from_file(&config_path).unwrap();
        assert_eq!(config.search.api_keys, vec!["key-b".to_string()]);
    }

    #[test]
    fn test_apply_config_mutations_rejects_invalid_path_shape() {
        let dir = temp_test_dir("mutations-error");
        let config_path = dir.join("config.yaml");
        std::fs::write(
            &config_path,
            r#"
agent:
  runner: codex_cli
"#,
        )
        .unwrap();

        let error = apply_config_mutations(
            &config_path,
            &[ConfigMutation::Set {
                path: "agent.runner.value".to_string(),
                value: Value::String("x".to_string()),
            }],
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("配置")
                || error.to_string().contains("invalid type")
                || error.to_string().contains("字符串")
        );
    }

    #[test]
    fn test_redact_sensitive_value_masks_scalars_and_sequences() {
        assert_eq!(
            redact_sensitive_value(
                "agent.opencode.api_key",
                &Value::String("sk-123".to_string())
            ),
            Value::String("<redacted>".to_string())
        );
        assert_eq!(
            redact_sensitive_value(
                "search.api_keys",
                &Value::Sequence(vec![
                    Value::String("a".to_string()),
                    Value::String("b".to_string())
                ])
            ),
            Value::Sequence(vec![
                Value::String("<redacted>".to_string()),
                Value::String("<redacted>".to_string())
            ])
        );
        assert_eq!(
            redact_sensitive_value("agent.runner", &Value::String("codex_cli".to_string())),
            Value::String("codex_cli".to_string())
        );
    }

    #[test]
    fn test_generate_effective_config_copies_relative_prompt_asset() {
        let dir = temp_test_dir("effective-config");
        let canonical = dir.join("config.yaml");
        let runtime_dir = dir.join("data/runtime");
        let effective = effective_config_path(&runtime_dir);

        std::fs::create_dir_all(&runtime_dir).unwrap();
        std::fs::write(
            &canonical,
            r#"
agent:
  system_prompt_path: "./soul.md"
  runner: codex_cli
"#,
        )
        .unwrap();
        std::fs::write(dir.join("soul.md"), "prompt").unwrap();

        let revision = generate_effective_config(&canonical, &effective).unwrap();
        assert!(!revision.is_empty());
        assert!(effective.exists());
        assert_eq!(
            std::fs::read_to_string(runtime_dir.join("soul.md")).unwrap(),
            "prompt"
        );
    }

    #[test]
    fn test_promote_legacy_runtime_agent_settings_migrates_blank_multi_agent_and_runner() {
        let dir = temp_test_dir("legacy-agent-migrate");
        let canonical = dir.join("config.yaml");
        let legacy = dir.join("data/runtime/config_runtime.yaml");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(
            &canonical,
            r#"
agent:
  runner: codex_cli
  multi_agent:
    search:
      api_key: ""
    answer:
      api_key: ""
  opencode:
    api_key: ""
llm:
  auxiliary:
    api_key: ""
  openrouter:
    api_key: ""
    api_keys: []
search:
  api_keys: []
fmp:
  api_key: ""
  api_keys: []
feishu:
  enabled: false
  app_id: ""
  app_secret: ""
telegram:
  enabled: false
  bot_token: ""
  chat_scope: DM_ONLY
discord:
  enabled: false
  bot_token: ""
  chat_scope: DM_ONLY
"#,
        )
        .unwrap();
        std::fs::write(
            &legacy,
            r#"
agent:
  runner: multi-agent
  multi_agent:
    search:
      base_url: "https://api.minimaxi.com/v1"
      api_key: "legacy-search"
      model: "MiniMax-M2.7-highspeed"
      max_iterations: 8
    answer:
      api_base_url: "https://openrouter.ai/api/v1"
      api_key: "legacy-answer"
      model: "google/gemini-3.1-pro-preview"
      variant: "high"
      max_tool_calls: 1
  opencode:
    api_base_url: "https://openrouter.ai/api/v1"
    api_key: "legacy-answer"
    model: "google/gemini-3.1-pro-preview"
    variant: "high"
llm:
  auxiliary:
    base_url: "https://api.minimaxi.com/v1"
    api_key: "legacy-search"
    model: "MiniMax-M2.7-highspeed"
  openrouter:
    api_key: "legacy-openrouter"
    api_keys:
      - legacy-openrouter-1
      - legacy-openrouter-2
search:
  provider: tavily
  api_keys:
    - tvly-one
    - tvly-two
  search_depth: advanced
  topic: finance
fmp:
  api_key: "legacy-fmp"
  api_keys:
    - legacy-fmp-2
  base_url: "https://financialmodelingprep.com/api"
  timeout: 30
feishu:
  enabled: true
  app_id: "cli_test"
  app_secret: "secret"
telegram:
  enabled: true
  bot_token: "tg-token"
  dm_only: false
discord:
  enabled: true
  bot_token: "discord-token"
  dm_only: false
"#,
        )
        .unwrap();

        let changed = promote_legacy_runtime_agent_settings(&canonical, &legacy).unwrap();

        assert!(changed.contains(&"agent.multi_agent".to_string()));
        assert!(changed.contains(&"agent.opencode".to_string()));
        assert!(changed.contains(&"llm.auxiliary".to_string()));
        assert!(changed.contains(&"llm.openrouter.api_key".to_string()));
        assert!(changed.contains(&"llm.openrouter.api_keys".to_string()));
        assert!(changed.contains(&"agent.runner".to_string()));
        assert!(changed.contains(&"search.api_keys".to_string()));
        assert!(changed.contains(&"fmp.api_key".to_string()));
        assert!(changed.contains(&"fmp.api_keys".to_string()));
        assert!(changed.contains(&"feishu.enabled".to_string()));
        assert!(changed.contains(&"telegram.enabled".to_string()));
        assert!(changed.contains(&"discord.enabled".to_string()));

        let config = HoneConfig::from_file(&canonical).unwrap();
        assert_eq!(config.agent.runner, "multi-agent");
        assert_eq!(config.agent.multi_agent.search.api_key, "legacy-search");
        assert_eq!(config.agent.multi_agent.answer.api_key, "legacy-answer");
        assert_eq!(config.agent.opencode.api_key, "legacy-answer");
        assert_eq!(config.llm.auxiliary.api_key, "legacy-search");
        assert_eq!(config.llm.openrouter.api_key, "legacy-openrouter");
        assert_eq!(
            config.llm.openrouter.api_keys,
            vec![
                "legacy-openrouter-1".to_string(),
                "legacy-openrouter-2".to_string()
            ]
        );
        assert_eq!(
            config.search.api_keys,
            vec!["tvly-one".to_string(), "tvly-two".to_string()]
        );
        assert_eq!(config.fmp.api_key, "legacy-fmp");
        assert_eq!(config.fmp.api_keys, vec!["legacy-fmp-2".to_string()]);
        assert!(config.feishu.enabled);
        assert_eq!(config.feishu.app_id, "cli_test");
        assert_eq!(config.feishu.app_secret, "secret");
        assert!(config.telegram.enabled);
        assert_eq!(config.telegram.bot_token, "tg-token");
        assert_eq!(config.telegram.chat_scope, ChatScope::All);
        assert!(config.discord.enabled);
        assert_eq!(config.discord.bot_token, "discord-token");
        assert_eq!(config.discord.chat_scope, ChatScope::All);
    }

    #[test]
    fn test_promote_legacy_runtime_agent_settings_migrates_openrouter_key_pool() {
        let dir = temp_test_dir("legacy-openrouter-pool");
        let canonical = dir.join("config.yaml");
        let legacy = dir.join("data/runtime/config_runtime.yaml");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(
            &canonical,
            r#"
llm:
  openrouter:
    api_key: ""
    api_keys: []
"#,
        )
        .unwrap();
        std::fs::write(
            &legacy,
            r#"
llm:
  openrouter:
    api_key: ""
    api_keys:
      - legacy-openrouter-1
      - legacy-openrouter-2
"#,
        )
        .unwrap();

        let changed = promote_legacy_runtime_agent_settings(&canonical, &legacy).unwrap();
        assert_eq!(changed, vec!["llm.openrouter.api_keys".to_string()]);

        let config = HoneConfig::from_file(&canonical).unwrap();
        assert_eq!(config.llm.openrouter.api_key, "");
        assert_eq!(
            config.llm.openrouter.api_keys,
            vec![
                "legacy-openrouter-1".to_string(),
                "legacy-openrouter-2".to_string()
            ]
        );
        assert_eq!(
            config.llm.openrouter.effective_key_pool().keys(),
            &["legacy-openrouter-1", "legacy-openrouter-2"]
        );
    }

    #[test]
    fn test_promote_legacy_runtime_agent_settings_keeps_configured_canonical_values() {
        let dir = temp_test_dir("legacy-agent-preserve");
        let canonical = dir.join("config.yaml");
        let legacy = dir.join("data/runtime/config_runtime.yaml");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(
            &canonical,
            r#"
agent:
  runner: multi-agent
  multi_agent:
    search:
      api_key: "canonical-search"
    answer:
      api_key: "canonical-answer"
llm:
  auxiliary:
    api_key: "canonical-aux"
  openrouter:
    api_key: "canonical-openrouter"
search:
  api_keys:
    - canonical-tavily
fmp:
  api_key: "canonical-fmp"
feishu:
  enabled: true
  app_id: "canonical-app"
telegram:
  enabled: true
  bot_token: "canonical-tg"
discord:
  enabled: true
  bot_token: "canonical-discord"
"#,
        )
        .unwrap();
        std::fs::write(
            &legacy,
            r#"
agent:
  runner: codex_cli
  multi_agent:
    search:
      api_key: "legacy-search"
    answer:
      api_key: "legacy-answer"
llm:
  auxiliary:
    api_key: "legacy-aux"
  openrouter:
    api_key: "legacy-openrouter"
search:
  api_keys:
    - legacy-tavily
fmp:
  api_key: "legacy-fmp"
feishu:
  enabled: true
  app_id: "legacy-app"
telegram:
  enabled: true
  bot_token: "legacy-tg"
discord:
  enabled: true
  bot_token: "legacy-discord"
"#,
        )
        .unwrap();

        let changed = promote_legacy_runtime_agent_settings(&canonical, &legacy).unwrap();
        assert!(changed.is_empty());

        let config = HoneConfig::from_file(&canonical).unwrap();
        assert_eq!(config.agent.runner, "multi-agent");
        assert_eq!(config.agent.multi_agent.search.api_key, "canonical-search");
        assert_eq!(config.agent.multi_agent.answer.api_key, "canonical-answer");
        assert_eq!(config.llm.auxiliary.api_key, "canonical-aux");
        assert_eq!(config.llm.openrouter.api_key, "canonical-openrouter");
        assert_eq!(config.search.api_keys, vec!["canonical-tavily".to_string()]);
        assert_eq!(config.fmp.api_key, "canonical-fmp");
        assert_eq!(config.feishu.app_id, "canonical-app");
        assert_eq!(config.telegram.bot_token, "canonical-tg");
        assert_eq!(config.discord.bot_token, "canonical-discord");
    }

    #[test]
    fn test_agent_runner_timeouts_default_to_step_plus_overall() {
        let yaml = r#"
agent:
  runner: codex_acp
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agent.step_timeout_seconds, 180);
        assert_eq!(config.agent.overall_timeout_seconds, 1200);
    }

    #[test]
    fn test_agent_runner_timeout_override_preserves_explicit_values() {
        let yaml = r#"
agent:
  runner: codex_acp
  step_timeout_seconds: 120
  overall_timeout_seconds: 600
"#;
        let config: HoneConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.agent.step_timeout_seconds, 120);
        assert_eq!(config.agent.overall_timeout_seconds, 600);
    }
}
